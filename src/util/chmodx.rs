/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

//! Make a file executable on Unix.

use std::io;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;

use crate::util::fs_ctx;

const DEFAULT_FILE_PERMISSIONS: u32 = 0o500;

pub fn chmodx<P: AsRef<Path>>(path: P) -> io::Result<()> {
    fn inner(path: &Path) -> io::Result<()> {
        let mut perms = fs_ctx::metadata(path)?.permissions();
        // Includes extra bits not just rwx permissions.
        // See: https://github.com/rust-lang/rust/issues/45330
        let mode = perms.mode();

        // Remove any extra bits.
        let file_permissions = mode & 0o777;

        // Only overwrite if the file isn't executable.
        if file_permissions & 0o111 == 0 {
            perms.set_mode(DEFAULT_FILE_PERMISSIONS);
            fs_ctx::set_permissions(path, perms)?;
        }

        Ok(())
    }

    inner(path.as_ref())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_chmodx() -> io::Result<()> {
        #[track_caller]
        fn t(before: u32, after: u32) -> io::Result<()> {
            let temp_path = NamedTempFile::new()?.into_temp_path();

            let mut perms = fs::metadata(&temp_path)?.permissions();
            perms.set_mode(before);
            fs::set_permissions(&temp_path, perms)?;
            assert_eq!(
                fs::metadata(&temp_path)?.permissions().mode() & 0o777,
                before,
            );

            chmodx(&temp_path)?;

            assert_eq!(
                fs::metadata(&temp_path)?.permissions().mode() & 0o777,
                after,
            );

            Ok(())
        }

        t(DEFAULT_FILE_PERMISSIONS, DEFAULT_FILE_PERMISSIONS)?;
        t(0o505, 0o505)?;
        t(0o550, 0o550)?;
        t(0o555, 0o555)?;

        t(0o100, 0o100)?;
        t(0o300, 0o300)?;
        t(0o700, 0o700)?;

        t(0o010, 0o010)?;
        t(0o030, 0o030)?;
        t(0o070, 0o070)?;

        t(0o001, 0o001)?;
        t(0o003, 0o003)?;
        t(0o007, 0o007)?;

        t(0o412, 0o412)?;

        t(0o000, DEFAULT_FILE_PERMISSIONS)?;
        t(0o200, DEFAULT_FILE_PERMISSIONS)?;
        t(0o400, DEFAULT_FILE_PERMISSIONS)?;
        t(0o600, DEFAULT_FILE_PERMISSIONS)?;

        Ok(())
    }
}
