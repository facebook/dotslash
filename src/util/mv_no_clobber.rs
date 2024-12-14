/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::io;
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::util::fs_ctx;

/// Move a file or directory only if destination does not exist.
///
/// This is conceptually equivalent to `mv --no-clobber`, though like `mv -n`,
/// this is susceptible to a TOCTTOU issue because another process could create
/// the file at the destination between the initial check for the destination and
/// the write.
///
/// If no move is performed because the destination already exists, this function
/// returns Ok, not Err.
///
/// TODO(T57290904): When possible, use platform-specific syscalls to make this
/// atomic. Specifically, the following should be available in newer OS versions:
///   * Linux: renameat2 with RENAME_NOREPLACE flag
///   * macOS: renamex_np with RENAME_EXCL flag
pub fn mv_no_clobber<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<()> {
    fn mv_no_clobber_impl(from: &Path, to: &Path) -> io::Result<()> {
        // If the destination already exists, do nothing.
        if to.exists() {
            return Ok(());
        }

        match fs_ctx::rename(from, to) {
            Ok(()) => Ok(()),
            Err(error) => {
                // If the rename failed, but the destination exists now,
                // assume we hit the TOCTTOU case and return Ok.
                if to.exists() { Ok(()) } else { Err(error) }
            }
        }
    }

    if cfg!(windows) {
        // It is a mystery why we get a permission error on Windows, that
        // then quickly clears up. This seems to happen when the system is
        // under heavy load.
        for wait in [1, 4, 9] {
            match mv_no_clobber_impl(from.as_ref(), to.as_ref()) {
                Err(err) if err.kind() == io::ErrorKind::PermissionDenied => {
                    thread::sleep(Duration::from_secs(wait));
                }
                ret => return ret,
            }
        }
    }

    mv_no_clobber_impl(from.as_ref(), to.as_ref())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_file() {
        let temp_path_1 = NamedTempFile::new().unwrap().into_temp_path();
        let temp_path_2 = NamedTempFile::new().unwrap().into_temp_path();

        assert!(temp_path_1.exists());
        assert!(temp_path_2.exists());

        // Should not fail just because dest exists.
        mv_no_clobber(&temp_path_1, &temp_path_2).unwrap();

        assert!(temp_path_1.exists());
        assert!(temp_path_2.exists());

        fs::remove_file(&temp_path_2).unwrap();

        assert!(temp_path_1.exists());
        assert!(!temp_path_2.exists());

        mv_no_clobber(&temp_path_1, &temp_path_2).unwrap();

        assert!(!temp_path_1.exists());
        assert!(temp_path_2.exists());
    }

    #[test]
    fn test_directory() {
        let temp_dir_1 = tempfile::tempdir().unwrap();
        let temp_dir_2 = tempfile::tempdir().unwrap();

        assert!(temp_dir_1.path().exists());
        assert!(temp_dir_2.path().exists());

        // Should not fail just because dest exists.
        mv_no_clobber(&temp_dir_1, &temp_dir_2).unwrap();

        assert!(temp_dir_1.path().exists());
        assert!(temp_dir_2.path().exists());

        fs::remove_dir_all(&temp_dir_2).unwrap();

        assert!(temp_dir_1.path().exists());
        assert!(!temp_dir_2.path().exists());

        mv_no_clobber(&temp_dir_1, &temp_dir_2).unwrap();

        assert!(!temp_dir_1.path().exists());
        assert!(temp_dir_2.path().exists());
    }
}
