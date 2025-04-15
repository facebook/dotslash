/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::borrow::Cow;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context as _;
use digest::Digest as _;
use serde_jsonrc::value::Value;
use sha2::Sha256;

use crate::artifact_location::ArtifactLocation;
use crate::config::ArtifactEntry;
use crate::config::HashAlgorithm;
use crate::digest::Digest;
use crate::fetch_method::ArtifactFormat;
use crate::provider::ProviderFactory;
use crate::util;
use crate::util::FileLock;
use crate::util::FileLockError;
use crate::util::fs_ctx;
use crate::util::unarchive;

pub const DEFAULT_PROVDIER_TYPE: &str = "http";

/// This function is responsible for:
/// 1. Downloading an artifact into a temp location within $DOTSLASH_CACHE.
/// 2. Verifying that the size and digest match the ArtifactEntry.
/// 3. Decompressing the artifact, as appropriate.
/// 4. Atomically moving it from its temp location to its final location.
pub fn download_artifact<P: ProviderFactory>(
    artifact_entry: &ArtifactEntry,
    artifact_location: &ArtifactLocation,
    provider_factory: &P,
) -> anyhow::Result<()> {
    let artifact_parent_dir = artifact_location
        .artifact_directory
        .parent()
        .with_context(|| {
            format!(
                "no parent for artifact_directory `{}`",
                artifact_location.artifact_directory.display()
            )
        })?;

    fs_ctx::create_dir_all(artifact_parent_dir)?;

    // We must maintain a reference to the FileLock until the download is complete.
    let file_lock = acquire_download_lock_for_artifact(artifact_location)
        .context("failed to get artifact lock")?;

    // Record warnings: only reported if no provider succeeds.
    let mut warnings = vec![];
    for provider_config in &artifact_entry.providers {
        // This must be a sibling to the final artifact_location so that we can
        // atomically move it into place.
        let temp_dir_to_mv = fs_ctx::tempdir_in(artifact_parent_dir)?;
        let fetch_destination: PathBuf = {
            let fetch_destination = fs_ctx::namedtempfile_new_in(artifact_parent_dir)
                .context("failed to create fetch temp path")?
                .into_temp_path();
            // fetch_destination is dropped after this and is removed from
            // disk. This is deliberate since we want a unique path and not
            // necessarily the file.
            fetch_destination.to_path_buf()
        };

        let provider_type = get_provider_type(provider_config)?;
        let provider = provider_factory.get_provider(provider_type)?;
        match provider.fetch_artifact(
            provider_config,
            &fetch_destination,
            &file_lock,
            artifact_entry,
        ) {
            Ok(()) => match verify_artifact(&fetch_destination, artifact_entry) {
                Ok(()) => {
                    unpack_verified_artifact(
                        &fetch_destination,
                        temp_dir_to_mv.path(),
                        artifact_entry.format,
                        artifact_entry.path.as_str(),
                    )?;
                    if artifact_entry.readonly {
                        util::make_tree_entries_read_only(temp_dir_to_mv.path())?;
                    }
                    util::mv_no_clobber(&temp_dir_to_mv, &artifact_location.artifact_directory)?;
                    if artifact_entry.readonly {
                        // Note the following appears to work on Linux but not
                        // macOS:
                        //
                        // ```
                        // /tmp$ mkdir foo
                        // /tmp$ chmod -w foo
                        // /tmp$ mv foo bar
                        // ```
                        //
                        // so we have to do the final `chmod -w` after the `mv`.
                        // While we could also do the full `chmod -R -w` after
                        // the `mv`, that is a bit riskier because a
                        // simultaneous invocation of the DotSlash file would be
                        // able to use the artifact before `chmod -R -w`
                        // finishes.
                        let metadata =
                            fs_ctx::symlink_metadata(&artifact_location.artifact_directory)?;
                        let mut perms = metadata.permissions();
                        perms.set_readonly(true);
                        fs_ctx::set_permissions(&artifact_location.artifact_directory, perms)?;
                    }
                    return Ok(());
                }
                Err(e) => warnings.push(format!("warning: failed to verify artifact {:?}", e)),
            },
            Err(e) => warnings.push(format!("failed to fetch artifact: {:?}", e)),
        }
    }

    Err(anyhow::format_err!(
        "no providers succeeded. warnings:\n{}",
        warnings.join("\n")
    ))
}

fn get_provider_type(provider_config: &Value) -> anyhow::Result<&str> {
    match provider_config.get("type") {
        Some(v) => v.as_str().context("type must map to a string"),
        None => Ok(DEFAULT_PROVDIER_TYPE),
    }
}

fn verify_artifact(
    artifact_temp_location: &Path,
    artifact_entry: &ArtifactEntry,
) -> anyhow::Result<()> {
    // First, verify the hash and digest.
    let mut file = File::options()
        .read(true)
        .open(artifact_temp_location)
        .with_context(|| {
            format!(
                "failed to open fetched artifact `{}`",
                artifact_temp_location.display()
            )
        })?;
    let (size_in_bytes, digest) = match artifact_entry.hash {
        HashAlgorithm::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            io::copy(&mut file, &mut hasher).map(|size_in_bytes| {
                let digest = format!("{:x}", hasher.finalize());
                (size_in_bytes, digest)
            })
        }
        HashAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            io::copy(&mut file, &mut hasher).map(|size_in_bytes| {
                let digest = format!("{:x}", hasher.finalize());
                (size_in_bytes, digest)
            })
        }
    }
    .with_context(|| {
        format!(
            "failed to calculate digest for fetched archive `{}`",
            artifact_temp_location.display()
        )
    })?;
    drop(file);

    if size_in_bytes != artifact_entry.size {
        return Err(anyhow::format_err!(
            "fetched artifact `{}` has incorrect size: {} bytes vs expected {} bytes",
            artifact_temp_location.display(),
            size_in_bytes,
            artifact_entry.size
        ));
    }

    let digest = Digest::try_from(digest)?;
    if digest != artifact_entry.digest {
        return Err(anyhow::format_err!(
            "fetched artifact `{}` has incorrect digest: {} vs expected {}",
            artifact_temp_location.display(),
            digest,
            artifact_entry.digest
        ));
    }

    Ok(())
}

/// Unpacks the verified artifact. When this function exits, the contents of
/// `temp_dir_to_mv` should be ready to be moved into the final location.
fn unpack_verified_artifact(
    fetched_artifact: &Path,
    temp_dir_to_mv: &Path,
    format: ArtifactFormat,
    artifact_entry_path: &str,
) -> anyhow::Result<()> {
    // Container artifacts get unarchived into directories.
    // Non-container artifacts get written directly to a file.
    let final_artifact_path = if format.is_container() {
        Cow::Borrowed(temp_dir_to_mv)
    } else {
        let final_artifact_path = temp_dir_to_mv.join(artifact_entry_path);
        let parent = final_artifact_path.parent().unwrap();
        if parent != Path::new("") {
            fs_ctx::create_dir_all(parent)?;
        }
        Cow::Owned(final_artifact_path)
    };

    if let Some(archive_type) = format.as_archive_type() {
        let reader = BufReader::new(fs_ctx::file_open(fetched_artifact)?);
        unarchive::unarchive(reader, &final_artifact_path, archive_type)?;
    } else {
        fs_ctx::rename(fetched_artifact, &final_artifact_path)?;
    }

    if !format.is_container() {
        #[cfg(unix)]
        util::chmodx(final_artifact_path).context("failed to make path executable")?;
    };

    Ok(())
}

/// Attempts to acquire an advisory lock for a lock file in the DotSlash cache
/// that corresponds to the artifact specified by `scheme`, creating the file
/// if necessary. This should be done before download_artifact() is called in
/// order to help prevent concurrent fetches.
///
/// Note that this system is not 100% foolproof: it serves as a "best effort" to
/// avoid concurrent fetches, but it is not guaranteed to prevent them.
///
/// Fortunately, download_artifact() is designed to be resilient in the face of
/// concurrent fetches, so locking is only a performance optimization, not a
/// required safeguard.
pub fn acquire_download_lock_for_artifact(
    artifact_location: &ArtifactLocation,
) -> anyhow::Result<FileLock> {
    if let Some(lock_dir) = artifact_location.lock_path.parent() {
        if fs_ctx::create_dir_all(lock_dir).is_ok() {
            match FileLock::acquire(&artifact_location.lock_path) {
                Ok(file_lock) => return Ok(file_lock),
                Err(err @ FileLockError::LockExclusive(..)) => return Err(err.into()),
                Err(FileLockError::Create(..)) => {}
            }
        }
    }
    Ok(FileLock::default())
}
