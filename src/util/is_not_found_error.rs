/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::io;

/// Determine if an `io::Error` means that a path does not exist.
///
/// A file cannot be the child path of another file. For example,
/// given that `/a/b` is a file, a path `/a/b/c` is an error because
/// `c` cannot exist as a child of the file `/a/b`.
///
/// On Windows, this is an `io::ErrorKind::NotFound` error.
/// On Unix, this is an `io::ErrorKind::NotADirectory`.
///
/// Note on execv:
///
/// If execv fails with ENOENT, that means we need to fetch the artifact.
/// This is the most likely error returned by execv.
///
/// ENOTDIR can happen if the program passed to execv is:
///
/// ~/.cache/dotslash/obj/ha/xx/abc/extract/my_tool
///
/// but the following is a regular file:
///
/// ~/.cache/dotslash/obj/ha/xx/abc/extract
///
/// This could happen if a previous release of DotSlash wrote this entry in
/// the cache in a different way that is not consistent with the current
/// directory structure. We should attempt to fetch the artifact again in
/// this case.
#[must_use]
pub fn is_not_found_error(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::NotFound
        || (cfg!(unix) && err.kind() == io::ErrorKind::NotADirectory)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::NamedTempFile;
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_is_not_found_error_not_found() {
        let temp_dir = TempDir::with_prefix("dotslash-").unwrap();
        let err = fs::read(temp_dir.path().join("fake_file.txt")).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
        assert!(is_not_found_error(&err));
    }

    #[test]
    fn test_is_not_found_error_enotdir() {
        let temp_file = NamedTempFile::with_prefix("dotslash-").unwrap();
        let err = fs::read(temp_file.path().join("fake_file.txt")).unwrap_err();
        assert_eq!(
            err.kind().to_string(),
            if cfg!(windows) {
                "entity not found"
            } else {
                "not a directory"
            },
        );
        assert!(is_not_found_error(&err));
    }
}
