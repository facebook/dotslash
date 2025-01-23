/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::borrow::Cow;
use std::fmt::Write;
use std::path::Path;
use std::path::PathBuf;

use crate::artifact_path::ArtifactPath;
use crate::config::Arg0;
use crate::config::ArtifactEntry;
use crate::config::HashAlgorithm;
use crate::dotslash_cache::DotslashCache;
use crate::fetch_method::ArtifactFormat;

/// We limit the number of bytes of the BLAKE3 hash to try to keep the path
/// lengths in $DOTSLASH_CACHE down so that we don't exceed $PATH_MAX.
/// Risk of collision is still quite low despite not using the full hash.
const NUM_HASH_BYTES_FOR_PATH: usize = 20;

/// Paths of interest for reading/writing the artifact for use by a Provider.
/// All paths are expected to be absolute.
pub struct ArtifactLocation {
    /// Directory where the final and intermediate objects for this artifact
    /// are kept.
    pub artifact_directory: PathBuf,
    /// Path to this artifact's executable.
    pub executable: PathBuf,
    /// The path to use for advisory locking while downloading the specified
    /// artifact.
    pub lock_path: PathBuf,
    /// Determines what arg0 (`argv[0]`) gets set to.
    #[cfg_attr(windows, expect(dead_code))]
    pub arg0: Arg0,
}

/// In terms of the computing the path within the artifact_directory, it is a
/// hash of the artifact's content (size, hash algorithm, digest) as well as how
/// it was decompressed (fetch_method). By design, it is *independent* of the
/// method used to fetch it.
///
/// There are effectively two cases:
///
/// - compressed archive: artifact_directory contains the extracted file(s).
///   (Note the archive might contain only a single file.)
/// - uncompressed single file: artifact_directory contains a single file whose
///   name matches the specified `filename` property in the DotSlash file.
pub fn determine_location(
    artifact_entry: &ArtifactEntry,
    dotslash_cache: &DotslashCache,
) -> ArtifactLocation {
    let ArtifactEntry {
        size,
        hash,
        digest,
        format,
        path,
        providers: _,
        arg0,
        readonly,
    } = artifact_entry;

    let artifact_hash = blake3::Hasher::new()
        .update(size.to_string().as_bytes())
        .update(b"\0")
        .update(create_key_for_hash_algorithm(*hash))
        .update(b"\0")
        .update(digest.as_str().as_bytes())
        .update(b"\0")
        .update(create_key_for_format(*format, path).as_bytes())
        .update(b"\0")
        .update(if *readonly { b"1" } else { b"0" })
        .finalize();
    let artifact_key = artifact_hash.as_bytes()[..NUM_HASH_BYTES_FOR_PATH]
        .iter()
        .fold(
            String::with_capacity(NUM_HASH_BYTES_FOR_PATH * 2),
            |mut output, b| {
                let _ = write!(output, "{b:02x}");
                output
            },
        );

    let (key_prefix, key_rest) = artifact_key.split_at(2);
    let artifact_directory = dotslash_cache
        .artifacts_dir()
        .join(key_prefix)
        .join(key_rest);

    let mut executable = artifact_directory.clone();
    executable.extend(Path::new(path.as_str()));
    let lock_path = dotslash_cache.locks_dir(key_prefix).join(key_rest);

    ArtifactLocation {
        artifact_directory,
        executable,
        lock_path,
        arg0: *arg0,
    }
}

fn create_key_for_hash_algorithm(hash: HashAlgorithm) -> &'static [u8] {
    match hash {
        HashAlgorithm::Blake3 => b"blake3",
        HashAlgorithm::Sha256 => b"sha256",
    }
}

fn create_key_for_format(format: ArtifactFormat, path: &ArtifactPath) -> Cow<'static, str> {
    match format {
        // For a non-container artifact, the `path` must be part of the cache
        // key. The key has a prefix to distinguish it from the cache keys
        // for archive artifacts.
        ArtifactFormat::Plain => Cow::Owned(format!("file:{}", path)),
        ArtifactFormat::Gz => Cow::Owned(format!("file.gz:{}", path)),
        ArtifactFormat::Xz => Cow::Owned(format!("file.xz:{}", path)),
        ArtifactFormat::Zstd => Cow::Owned(format!("file.zst:{}", path)),

        // For a container artifact, the type of archive is sufficient
        // to distinguish it.
        ArtifactFormat::Tar => Cow::Borrowed("tar"),
        ArtifactFormat::TarGz => Cow::Borrowed("tar.gz"),
        ArtifactFormat::TarXz => Cow::Borrowed("tar.xz"),
        ArtifactFormat::TarZstd => Cow::Borrowed("tar.zst"),
        ArtifactFormat::Zip => Cow::Borrowed("zip"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::digest::Digest;

    #[test]
    fn paths_for_extract_case() {
        let artifact_entry = ArtifactEntry {
            size: 8675309,
            hash: HashAlgorithm::Blake3,
            digest: Digest::try_from(
                "7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069".to_owned(),
            )
            .unwrap(),
            format: ArtifactFormat::TarGz,
            path: "bin/sapling".parse().unwrap(),
            providers: vec![],
            arg0: Arg0::DotslashFile,
            readonly: true,
        };
        let dotslash_cache = DotslashCache::default();
        let location = determine_location(&artifact_entry, &dotslash_cache);

        assert_eq!(
            location.artifact_directory,
            dotslash_cache
                .artifacts_dir()
                .join("0c")
                .join("7cc25be015e0ab6855aaa7bfea49d5dffe5e4c")
        );
        assert_eq!(
            location.executable,
            dotslash_cache
                .artifacts_dir()
                .join("0c")
                .join("7cc25be015e0ab6855aaa7bfea49d5dffe5e4c")
                .join("bin/sapling"),
        );
        assert_eq!(
            location.lock_path,
            dotslash_cache
                .locks_dir("0c")
                .join("7cc25be015e0ab6855aaa7bfea49d5dffe5e4c")
        );
    }

    #[test]
    fn paths_for_rename_case() {
        let artifact_entry = ArtifactEntry {
            size: 381654729,
            hash: HashAlgorithm::Sha256,
            digest: Digest::try_from(
                "7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069".to_owned(),
            )
            .unwrap(),
            format: ArtifactFormat::Plain,
            path: "minesweeper.exe".parse().unwrap(),
            providers: vec![],
            arg0: Arg0::DotslashFile,
            readonly: true,
        };
        let dotslash_cache = DotslashCache::default();
        let location = determine_location(&artifact_entry, &dotslash_cache);

        assert_eq!(
            location.artifact_directory,
            dotslash_cache
                .artifacts_dir()
                .join("0d")
                .join("fd21d5ac7f30378d523758d64d902698559d72")
        );
        assert_eq!(
            location.executable,
            dotslash_cache
                .artifacts_dir()
                .join("0d")
                .join("fd21d5ac7f30378d523758d64d902698559d72")
                .join("minesweeper.exe"),
        );
        assert_eq!(
            location.lock_path,
            dotslash_cache
                .locks_dir("0d")
                .join("fd21d5ac7f30378d523758d64d902698559d72")
        );
    }
}
