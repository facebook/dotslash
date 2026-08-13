/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

//! Size-capped LRU eviction for the DotSlash artifact cache.
//!
//! Design notes:
//!
//! - There is no catalog of entries. Each `$CACHE/<2-hex>/<rest>/` directory
//!   is one cache entry. Recency is that directory's mtime.
//! - A running total in `$CACHE/usage` is the miss-path gate. If the total is
//!   at or under the limit, GC does not walk the cache. The counter is allowed
//!   to run high (external deletes are not subtracted); eviction recounts and
//!   writes the true sum. A missing or unreadable file means "unknown" and
//!   forces one validate walk — it is never treated as zero.
//! - The hot path (cache hit → exec) only updates the artifact directory
//!   mtime (one `utime` syscall; ignored if it fails).
//! - Eviction runs after a download when `DOTSLASH_CACHE_MAX_SIZE` is set, or
//!   when the user runs `dotslash -- clean --size SIZE`. Auto-GC deletes
//!   down to 80% of the limit (hysteresis); `clean --size` deletes until
//!   the cache is `<= SIZE`. If another process holds the GC lock, auto-GC
//!   skips without walking.
//! - Sizing and deletes are single-threaded. Parallel walks are a possible
//!   later optimization; this path must finish before `execv`.

use std::env;
use std::fs;
use std::io;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Once;
use std::time::SystemTime;

use anyhow::Context as _;

use crate::artifact_location::ARTIFACT_HASH_PREFIX_LEN;
use crate::artifact_location::ARTIFACT_HASH_REST_LEN;
use crate::dotslash_cache::DOTSLASH_CACHE_MAX_SIZE_ENV;
use crate::dotslash_cache::DotslashCache;
use crate::util;
use crate::util::FileLock;
use crate::util::fs_ctx;

/// After exceeding `max_bytes`, auto-GC deletes until the cache is at most
/// this fraction of the limit so a few subsequent downloads do not
/// immediately re-trigger GC. Manual `clean --size` does not use hysteresis.
const GC_TARGET_NUMERATOR: u64 = 4;
const GC_TARGET_DENOMINATOR: u64 = 5;

/// Suffix for directories renamed out of the artifact hash path before
/// deletion. Not a valid artifact rest name, so a crash mid-delete cannot
/// be mistaken for a cache entry.
const GC_TRASH_SUFFIX: &str = ".gc";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GcStats {
    pub bytes_before: u64,
    pub bytes_after: u64,
    pub artifacts_before: usize,
    pub artifacts_evicted: usize,
    pub bytes_evicted: u64,
}

struct CachedArtifact {
    path: PathBuf,
    size: u64,
    mtime: SystemTime,
}

/// Parse a byte size like `10G`, `512MiB`, or `1048576`.
///
/// Suffixes are binary (k = 1024). `B` / `iB` after k/m/g/t is optional.
pub fn parse_byte_size(input: &str) -> anyhow::Result<u64> {
    let s = input.trim();
    if s.is_empty() {
        return Err(anyhow::format_err!("empty size"));
    }

    let n_end = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    if n_end == 0 {
        return Err(anyhow::format_err!("invalid size `{input}`"));
    }

    let (num, suffix) = s.split_at(n_end);
    let n: u64 = num
        .parse()
        .with_context(|| format!("invalid size `{input}`"))?;
    let multiplier: u64 = match suffix.trim().to_ascii_lowercase().as_str() {
        "" | "b" => 1,
        "k" | "kb" | "kib" => 1024,
        "m" | "mb" | "mib" => 1024 * 1024,
        "g" | "gb" | "gib" => 1024 * 1024 * 1024,
        "t" | "tb" | "tib" => 1024u64.pow(4),
        _ => {
            return Err(anyhow::format_err!(
                "unknown size suffix `{suffix}` in `{input}`"
            ));
        }
    };

    n.checked_mul(multiplier)
        .ok_or_else(|| anyhow::format_err!("size `{input}` overflow"))
}

/// Returns `None` if automatic GC is disabled (unset, empty, or invalid).
/// `0` is a real cap of zero bytes. Invalid values are reported once to
/// stderr and treated as unset so they cannot break execution.
pub fn configured_max_size() -> Option<u64> {
    match env::var(DOTSLASH_CACHE_MAX_SIZE_ENV) {
        Err(env::VarError::NotPresent) => None,
        Err(env::VarError::NotUnicode(_)) => {
            warn_invalid_max_size(&format!(
                "dotslash: {DOTSLASH_CACHE_MAX_SIZE_ENV} is not valid Unicode; ignoring"
            ));
            None
        }
        Ok(s) if s.trim().is_empty() => None,
        Ok(s) => match parse_byte_size(&s) {
            Ok(n) => Some(n),
            Err(err) => {
                warn_invalid_max_size(&format!(
                    "dotslash: invalid {DOTSLASH_CACHE_MAX_SIZE_ENV} value `{s}` ({err}); ignoring"
                ));
                None
            }
        },
    }
}

fn warn_invalid_max_size(message: &str) {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let _ = writeln!(io::stderr(), "{message}");
    });
}

/// If `DOTSLASH_CACHE_MAX_SIZE` is set, record this download in the running
/// total and evict only when that total exceeds the limit. Never evicts
/// `protect` (the artifact just downloaded). Failures are ignored so a GC
/// problem cannot break execution.
pub fn maybe_gc_after_download(cache: &DotslashCache, protect: &Path) {
    let Some(max_bytes) = configured_max_size() else {
        return;
    };
    let _ = record_download_and_maybe_gc(cache, protect, max_bytes);
}

/// Scan the cache and evict least-recently-used artifacts until the total
/// size is at most `max_bytes`. Unlike auto-GC, this does not apply the 80%
/// hysteresis watermark.
///
/// `protect`, if set, is never deleted.
pub fn gc_cache(
    cache: &DotslashCache,
    max_bytes: u64,
    protect: Option<&Path>,
) -> anyhow::Result<GcStats> {
    ensure_lock_dir(cache)?;

    let _gc_lock =
        FileLock::acquire(cache.gc_lock_path()).context("failed to lock cache for gc")?;
    let _usage_lock =
        FileLock::acquire(cache.usage_lock_path()).context("failed to lock cache usage")?;
    gc_cache_locked(cache, max_bytes, protect, /*hysteresis*/ false)
}

pub fn format_bytes(n: u64) -> String {
    const K: f64 = 1024.0;
    let n_f = n as f64;
    if n >= 1024u64.pow(4) {
        format!("{:.1} TiB", n_f / K.powi(4))
    } else if n >= 1024u64.pow(3) {
        format!("{:.1} GiB", n_f / K.powi(3))
    } else if n >= 1024u64.pow(2) {
        format!("{:.1} MiB", n_f / K.powi(2))
    } else if n >= 1024 {
        format!("{:.1} KiB", n_f / K)
    } else {
        format!("{n} B")
    }
}

fn skipped_stats() -> GcStats {
    GcStats {
        bytes_before: 0,
        bytes_after: 0,
        artifacts_before: 0,
        artifacts_evicted: 0,
        bytes_evicted: 0,
    }
}

fn record_download_and_maybe_gc(
    cache: &DotslashCache,
    protect: &Path,
    max_bytes: u64,
) -> anyhow::Result<GcStats> {
    let new_size = dir_size(protect)?;
    ensure_lock_dir(cache)?;

    let need_gc = {
        let _usage_lock =
            FileLock::acquire(cache.usage_lock_path()).context("failed to lock cache usage")?;
        match read_usage(cache)? {
            Some(total) => {
                let new_total = total.saturating_add(new_size);
                write_usage(cache, new_total)?;
                new_total > max_bytes
            }
            // Missing/invalid usage: never treat as 0 (that would hide an
            // already-populated cache). Force a validate walk.
            None => true,
        }
    };

    if !need_gc {
        return Ok(skipped_stats());
    }

    let Some(_gc_lock) =
        FileLock::try_acquire(cache.gc_lock_path()).context("failed to lock cache for gc")?
    else {
        return Ok(skipped_stats());
    };
    let _usage_lock =
        FileLock::acquire(cache.usage_lock_path()).context("failed to lock cache usage")?;
    gc_cache_locked(cache, max_bytes, Some(protect), /*hysteresis*/ true)
}

fn eviction_target(max_bytes: u64, hysteresis: bool) -> u64 {
    if hysteresis {
        max_bytes.saturating_mul(GC_TARGET_NUMERATOR) / GC_TARGET_DENOMINATOR
    } else {
        max_bytes
    }
}

fn gc_cache_locked(
    cache: &DotslashCache,
    max_bytes: u64,
    protect: Option<&Path>,
    hysteresis: bool,
) -> anyhow::Result<GcStats> {
    let mut artifacts = collect_artifacts(cache)?;
    let bytes_before: u64 = artifacts.iter().map(|a| a.size).sum();
    let artifacts_before = artifacts.len();
    let target = eviction_target(max_bytes, hysteresis);

    let stats = if bytes_before <= max_bytes {
        GcStats {
            bytes_before,
            bytes_after: bytes_before,
            artifacts_before,
            artifacts_evicted: 0,
            bytes_evicted: 0,
        }
    } else {
        // Oldest first. Equal mtimes fall back to path for determinism.
        artifacts.sort_by(|a, b| a.mtime.cmp(&b.mtime).then_with(|| a.path.cmp(&b.path)));

        let mut bytes_after = bytes_before;
        let mut artifacts_evicted = 0;
        let mut bytes_evicted: u64 = 0;

        for artifact in artifacts {
            if bytes_after <= target {
                break;
            }
            if protect.is_some_and(|p| p == artifact.path.as_path()) {
                continue;
            }
            // Skip if a fetch holds the download lock, or if we cannot take it.
            // Never delete unlocked on lock errors.
            let Some(_artifact_lock) = try_lock_artifact_for_delete(cache, &artifact.path) else {
                continue;
            };
            if remove_cached_artifact(&artifact.path).is_ok() {
                bytes_after = bytes_after.saturating_sub(artifact.size);
                bytes_evicted = bytes_evicted.saturating_add(artifact.size);
                artifacts_evicted += 1;
            }
        }

        GcStats {
            bytes_before,
            bytes_after,
            artifacts_before,
            artifacts_evicted,
            bytes_evicted,
        }
    };

    write_usage(cache, stats.bytes_after)?;
    Ok(stats)
}

fn collect_artifacts(cache: &DotslashCache) -> anyhow::Result<Vec<CachedArtifact>> {
    let mut artifacts = Vec::new();
    let root = cache.artifacts_dir();
    let entries = match fs_ctx::read_dir(root) {
        Ok(entries) => entries,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(artifacts),
        Err(err) => return Err(err.into()),
    };

    for entry in entries {
        let Some(entry) = skip_not_found(entry)? else {
            continue;
        };
        let name = entry.file_name();
        let Some(prefix) = name.to_str() else {
            continue;
        };
        if prefix.len() != ARTIFACT_HASH_PREFIX_LEN || !is_lowercase_hex(prefix) {
            continue;
        }
        let Some(file_type) = skip_not_found(entry.file_type())? else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }

        let prefix_entries = match fs_ctx::read_dir(entry.path()) {
            Ok(entries) => entries,
            Err(err) if err.kind() == io::ErrorKind::NotFound => continue,
            Err(err) => return Err(err.into()),
        };
        for artifact_entry in prefix_entries {
            let Some(artifact_entry) = skip_not_found(artifact_entry)? else {
                continue;
            };
            let rest = artifact_entry.file_name();
            let Some(rest) = rest.to_str() else {
                continue;
            };
            if is_gc_trash_name(rest) {
                let _ = delete_tree(&artifact_entry.path());
                continue;
            }
            if rest.len() != ARTIFACT_HASH_REST_LEN || !is_lowercase_hex(rest) {
                continue;
            }
            let Some(file_type) = skip_not_found(artifact_entry.file_type())? else {
                continue;
            };
            if !file_type.is_dir() {
                continue;
            }

            let path = artifact_entry.path();
            let Some(metadata) = skip_not_found(fs_ctx::symlink_metadata(&path))? else {
                continue;
            };
            let mtime = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let size = match dir_size(&path) {
                Ok(size) => size,
                Err(err) if err.kind() == io::ErrorKind::NotFound => continue,
                Err(err) => return Err(err.into()),
            };
            artifacts.push(CachedArtifact { path, size, mtime });
        }
    }

    Ok(artifacts)
}

fn skip_not_found<T>(result: io::Result<T>) -> io::Result<Option<T>> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err),
    }
}

fn is_lowercase_hex(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn is_gc_trash_name(name: &str) -> bool {
    name.starts_with('.') && name.ends_with(GC_TRASH_SUFFIX)
}

fn dir_size(path: &Path) -> io::Result<u64> {
    let mut size = 0;
    let entries = match fs_ctx::read_dir(path) {
        Ok(entries) => entries,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(err) => return Err(err),
    };
    for entry in entries {
        let Some(entry) = skip_not_found(entry)? else {
            continue;
        };
        let Some(metadata) = skip_not_found(fs_ctx::symlink_metadata(entry.path()))? else {
            continue;
        };
        if metadata.is_dir() {
            size += dir_size(&entry.path())?;
        } else if metadata.is_file() {
            size += metadata.len();
        }
    }
    Ok(size)
}

fn artifact_lock_path(cache: &DotslashCache, artifact_dir: &Path) -> Option<PathBuf> {
    let rest = artifact_dir.file_name()?;
    let prefix = artifact_dir.parent()?.file_name()?.to_str()?;
    Some(cache.locks_dir(prefix).join(rest))
}

fn try_lock_artifact_for_delete(cache: &DotslashCache, artifact_dir: &Path) -> Option<FileLock> {
    let lock_path = artifact_lock_path(cache, artifact_dir)?;
    if let Some(parent) = lock_path.parent() {
        fs_ctx::create_dir_all(parent).ok()?;
    }
    FileLock::try_acquire(&lock_path).ok().flatten()
}

fn trash_path(artifact_dir: &Path) -> Option<PathBuf> {
    let rest = artifact_dir.file_name()?.to_str()?;
    Some(
        artifact_dir
            .parent()?
            .join(format!(".{rest}{GC_TRASH_SUFFIX}")),
    )
}

fn delete_tree(path: &Path) -> io::Result<()> {
    let _ = make_writable(path);
    let _ = util::make_tree_entries_writable(path);
    fs_ctx::remove_dir_all(path)
}

/// Rename the artifact out of its hash path first so a partial delete cannot
/// leave a broken entry that `mv_no_clobber` will refuse to replace. If the
/// subsequent delete fails, the next walk reaps the leftover trash directory.
fn remove_cached_artifact(path: &Path) -> io::Result<()> {
    let Some(trash) = trash_path(path) else {
        return delete_tree(path);
    };
    if trash.exists() {
        let _ = delete_tree(&trash);
    }
    let _ = make_writable(path);
    fs_ctx::rename(path, &trash)?;
    let _ = delete_tree(&trash);
    if let Some(parent) = path.parent() {
        let _ = fs::remove_dir(parent);
    }
    Ok(())
}

fn make_writable(path: &Path) -> io::Result<()> {
    let metadata = fs_ctx::symlink_metadata(path)?;
    let mut perms = metadata.permissions();
    if perms.readonly() {
        #[expect(clippy::permissions_set_readonly_false)]
        perms.set_readonly(false);
        fs_ctx::set_permissions(path, perms)?;
    }
    Ok(())
}

fn ensure_lock_dir(cache: &DotslashCache) -> anyhow::Result<()> {
    if let Some(lock_dir) = cache.gc_lock_path().parent() {
        fs_ctx::create_dir_all(lock_dir)?;
    }
    Ok(())
}

fn read_usage(cache: &DotslashCache) -> anyhow::Result<Option<u64>> {
    match fs::read_to_string(cache.usage_path()) {
        Ok(s) => match s.trim().parse::<u64>() {
            Ok(n) => Ok(Some(n)),
            Err(_) => Ok(None),
        },
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).context("failed to read cache usage"),
    }
}

fn write_usage(cache: &DotslashCache, bytes: u64) -> anyhow::Result<()> {
    fs::write(cache.usage_path(), format!("{bytes}\n")).context("failed to write cache usage")
}

#[cfg(test)]
mod tests {
    use super::*;
    use filetime::FileTime;
    use std::fs::File;

    fn hex_name(ch: char, len: usize) -> String {
        String::from(ch).repeat(len)
    }

    fn write_artifact(cache: &Path, prefix: &str, rest_ch: char, nbytes: usize) -> PathBuf {
        let dir = cache
            .join(prefix)
            .join(hex_name(rest_ch, ARTIFACT_HASH_REST_LEN));
        fs::create_dir_all(&dir).unwrap();
        let mut file = File::create(dir.join("blob")).unwrap();
        file.write_all(&vec![0u8; nbytes]).unwrap();
        dir
    }

    fn set_mtime(path: &Path, secs_ago: i64) {
        let time = FileTime::from_unix_time(1_700_000_000 - secs_ago, 0);
        filetime::set_file_mtime(path, time).unwrap();
    }

    #[test]
    fn parse_byte_size_plain() {
        assert_eq!(parse_byte_size("0").unwrap(), 0);
        assert_eq!(parse_byte_size("123").unwrap(), 123);
        assert_eq!(parse_byte_size("  1048576  ").unwrap(), 1048576);
    }

    #[test]
    fn parse_byte_size_suffixes() {
        assert_eq!(parse_byte_size("1k").unwrap(), 1024);
        assert_eq!(parse_byte_size("1K").unwrap(), 1024);
        assert_eq!(parse_byte_size("1KB").unwrap(), 1024);
        assert_eq!(parse_byte_size("1KiB").unwrap(), 1024);
        assert_eq!(parse_byte_size("2M").unwrap(), 2 * 1024 * 1024);
        assert_eq!(parse_byte_size("10G").unwrap(), 10 * 1024 * 1024 * 1024);
        assert_eq!(parse_byte_size("1T").unwrap(), 1024u64.pow(4));
        assert_eq!(parse_byte_size("1 b").unwrap(), 1);
    }

    #[test]
    fn parse_byte_size_errors() {
        assert!(parse_byte_size("").is_err());
        assert!(parse_byte_size("G").is_err());
        assert!(parse_byte_size("10Q").is_err());
        assert!(parse_byte_size("-1").is_err());
        assert!(parse_byte_size("1.5G").is_err());
    }

    #[test]
    fn collect_skips_locks_and_temp_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = DotslashCache::new_in(tmp.path());
        write_artifact(tmp.path(), "aa", 'a', 10);
        fs::create_dir_all(tmp.path().join("locks").join("aa")).unwrap();
        fs::create_dir_all(tmp.path().join("aa").join(".tmpXXXX")).unwrap();
        fs::create_dir_all(tmp.path().join("not-hex")).unwrap();
        fs::write(tmp.path().join("usage"), "10\n").unwrap();

        let artifacts = collect_artifacts(&cache).unwrap();
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].size, 10);
    }

    #[test]
    fn gc_evicts_oldest_until_under_target() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = DotslashCache::new_in(tmp.path());
        let old = write_artifact(tmp.path(), "aa", 'a', 1000);
        let mid = write_artifact(tmp.path(), "bb", 'b', 1000);
        let new = write_artifact(tmp.path(), "cc", 'c', 1000);
        set_mtime(&old, 300);
        set_mtime(&mid, 200);
        set_mtime(&new, 100);

        // 3000 > 2500, so evict oldest (1000) → 2000, which is already <= 2500.
        let stats = gc_cache(&cache, 2500, None).unwrap();
        assert_eq!(stats.artifacts_before, 3);
        assert_eq!(stats.artifacts_evicted, 1);
        assert_eq!(stats.bytes_before, 3000);
        assert_eq!(stats.bytes_after, 2000);
        assert!(!old.exists());
        assert!(mid.exists());
        assert!(new.exists());
        assert_eq!(read_usage(&cache).unwrap(), Some(2000));
    }

    #[test]
    fn gc_never_evicts_protected_artifact() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = DotslashCache::new_in(tmp.path());
        let old = write_artifact(tmp.path(), "aa", 'a', 1000);
        let new = write_artifact(tmp.path(), "bb", 'b', 1000);
        set_mtime(&old, 200);
        set_mtime(&new, 100);

        // Limit 1 byte would evict everything, but `new` is protected so only
        // `old` goes away.
        let stats = gc_cache(&cache, 1, Some(&new)).unwrap();
        assert_eq!(stats.artifacts_evicted, 1);
        assert!(!old.exists());
        assert!(new.exists());
        assert_eq!(stats.bytes_after, 1000);
    }

    #[test]
    fn gc_zero_evicts_all() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = DotslashCache::new_in(tmp.path());
        write_artifact(tmp.path(), "aa", 'a', 100);
        write_artifact(tmp.path(), "bb", 'b', 100);

        let stats = gc_cache(&cache, 0, None).unwrap();
        assert_eq!(stats.artifacts_evicted, 2);
        assert_eq!(stats.bytes_after, 0);
        assert!(collect_artifacts(&cache).unwrap().is_empty());
        assert_eq!(read_usage(&cache).unwrap(), Some(0));
    }

    #[test]
    fn gc_under_limit_is_noop() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = DotslashCache::new_in(tmp.path());
        write_artifact(tmp.path(), "aa", 'a', 100);

        let stats = gc_cache(&cache, 1000, None).unwrap();
        assert_eq!(stats.artifacts_evicted, 0);
        assert_eq!(stats.bytes_after, 100);
        assert_eq!(read_usage(&cache).unwrap(), Some(100));
    }

    #[test]
    fn gc_deletes_readonly_artifact_tree() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = DotslashCache::new_in(tmp.path());
        let dir = write_artifact(tmp.path(), "aa", 'a', 50);
        util::make_tree_entries_read_only(&dir).unwrap();
        let mut perms = fs::symlink_metadata(&dir).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&dir, perms).unwrap();

        let stats = gc_cache(&cache, 0, None).unwrap();
        assert_eq!(stats.artifacts_evicted, 1);
        assert!(!dir.exists());
    }

    #[test]
    fn record_download_under_limit_does_not_evict() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = DotslashCache::new_in(tmp.path());
        let old = write_artifact(tmp.path(), "aa", 'a', 1000);
        let new = write_artifact(tmp.path(), "bb", 'b', 1000);
        write_usage(&cache, 1000).unwrap();

        let stats = record_download_and_maybe_gc(&cache, &new, 2500).unwrap();
        assert_eq!(stats.artifacts_evicted, 0);
        assert!(old.exists());
        assert!(new.exists());
        assert_eq!(read_usage(&cache).unwrap(), Some(2000));
    }

    #[test]
    fn record_download_over_limit_evicts_and_heals_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = DotslashCache::new_in(tmp.path());
        let old = write_artifact(tmp.path(), "aa", 'a', 1000);
        let new = write_artifact(tmp.path(), "bb", 'b', 1000);
        set_mtime(&old, 200);
        set_mtime(&new, 100);
        write_usage(&cache, 1000).unwrap();

        let stats = record_download_and_maybe_gc(&cache, &new, 1500).unwrap();
        assert_eq!(stats.artifacts_evicted, 1);
        assert!(!old.exists());
        assert!(new.exists());
        assert_eq!(read_usage(&cache).unwrap(), Some(1000));
    }

    #[test]
    fn missing_usage_forces_validate_walk() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = DotslashCache::new_in(tmp.path());
        let old = write_artifact(tmp.path(), "aa", 'a', 1000);
        let new = write_artifact(tmp.path(), "bb", 'b', 1000);
        set_mtime(&old, 200);
        set_mtime(&new, 100);
        assert!(read_usage(&cache).unwrap().is_none());

        // Unknown usage + over limit → walk, evict oldest, persist true sum.
        let stats = record_download_and_maybe_gc(&cache, &new, 1500).unwrap();
        assert_eq!(stats.artifacts_evicted, 1);
        assert!(!old.exists());
        assert!(new.exists());
        assert_eq!(read_usage(&cache).unwrap(), Some(1000));
    }

    #[test]
    fn high_usage_false_positive_heals_without_delete() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = DotslashCache::new_in(tmp.path());
        let only = write_artifact(tmp.path(), "aa", 'a', 100);
        // Counter thinks we are way over; walk shows we are not.
        write_usage(&cache, 10_000).unwrap();

        let stats = record_download_and_maybe_gc(&cache, &only, 1000).unwrap();
        assert_eq!(stats.artifacts_evicted, 0);
        assert!(only.exists());
        assert_eq!(read_usage(&cache).unwrap(), Some(100));
    }

    #[test]
    fn missing_usage_under_limit_writes_true_sum() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = DotslashCache::new_in(tmp.path());
        let only = write_artifact(tmp.path(), "aa", 'a', 100);
        assert!(read_usage(&cache).unwrap().is_none());

        let stats = record_download_and_maybe_gc(&cache, &only, 10_000).unwrap();
        assert_eq!(stats.artifacts_evicted, 0);
        assert!(only.exists());
        assert_eq!(read_usage(&cache).unwrap(), Some(100));
    }

    #[test]
    fn gc_clean_stops_at_limit_not_hysteresis() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = DotslashCache::new_in(tmp.path());
        let a = write_artifact(tmp.path(), "aa", 'a', 1000);
        let b = write_artifact(tmp.path(), "bb", 'b', 1000);
        let c = write_artifact(tmp.path(), "cc", 'c', 1000);
        let d = write_artifact(tmp.path(), "dd", 'd', 1000);
        set_mtime(&a, 400);
        set_mtime(&b, 300);
        set_mtime(&c, 200);
        set_mtime(&d, 100);

        // 4000 > 3500; clean trims to <= 3500 (one eviction), not to 80% (2800).
        let stats = gc_cache(&cache, 3500, None).unwrap();
        assert_eq!(stats.artifacts_evicted, 1);
        assert_eq!(stats.bytes_after, 3000);
        assert!(!a.exists());
        assert!(b.exists());
        assert!(c.exists());
        assert!(d.exists());
    }

    #[test]
    fn record_download_uses_hysteresis() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = DotslashCache::new_in(tmp.path());
        let a = write_artifact(tmp.path(), "aa", 'a', 1000);
        let b = write_artifact(tmp.path(), "bb", 'b', 1000);
        let c = write_artifact(tmp.path(), "cc", 'c', 1000);
        let new = write_artifact(tmp.path(), "dd", 'd', 1000);
        set_mtime(&a, 400);
        set_mtime(&b, 300);
        set_mtime(&c, 200);
        set_mtime(&new, 100);
        write_usage(&cache, 3000).unwrap();

        // 4000 > 3500; auto-GC target is 2800, so two oldest go.
        let stats = record_download_and_maybe_gc(&cache, &new, 3500).unwrap();
        assert_eq!(stats.artifacts_evicted, 2);
        assert_eq!(stats.bytes_after, 2000);
        assert!(!a.exists());
        assert!(!b.exists());
        assert!(c.exists());
        assert!(new.exists());
    }

    #[test]
    fn gc_skips_locked_artifact() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = DotslashCache::new_in(tmp.path());
        let locked = write_artifact(tmp.path(), "aa", 'a', 100);
        let other = write_artifact(tmp.path(), "bb", 'b', 100);
        let lock_path = artifact_lock_path(&cache, &locked).unwrap();
        fs::create_dir_all(lock_path.parent().unwrap()).unwrap();
        let _lock = FileLock::acquire(&lock_path).unwrap();

        let stats = gc_cache(&cache, 0, None).unwrap();
        assert_eq!(stats.artifacts_evicted, 1);
        assert!(locked.exists());
        assert!(!other.exists());
    }

    #[test]
    fn gc_reaps_leftover_trash() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = DotslashCache::new_in(tmp.path());
        let dir = write_artifact(tmp.path(), "aa", 'a', 10);
        let trash = trash_path(&dir).unwrap();
        fs::create_dir_all(&trash).unwrap();
        File::create(trash.join("leftover")).unwrap();

        let stats = gc_cache(&cache, 10_000, None).unwrap();
        assert_eq!(stats.artifacts_evicted, 0);
        assert!(dir.exists());
        assert!(!trash.exists());
    }

    #[test]
    fn mtime_updates_on_readonly_artifact_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = write_artifact(tmp.path(), "aa", 'a', 10);
        util::make_tree_entries_read_only(&dir).unwrap();
        let mut perms = fs::symlink_metadata(&dir).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&dir, perms).unwrap();
        let before = fs::symlink_metadata(&dir).unwrap().modified().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        util::update_mtime(&dir).unwrap();
        let after = fs::symlink_metadata(&dir).unwrap().modified().unwrap();
        assert!(
            after > before,
            "readonly artifact dir mtime must be updatable for LRU"
        );
    }

    #[test]
    fn format_bytes_units() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MiB");
    }
}
