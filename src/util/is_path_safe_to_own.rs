/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::io;
use std::os::unix::fs::MetadataExt as _;
use std::path::Path;

use nix::unistd;

use crate::util;

/// A path is considered "safe to own" if:
/// (1) it exists and we own it, or,
/// (2) it doesn't exist and we own the nearest parent that does exist.
#[must_use]
pub fn is_path_safe_to_own(path: &Path) -> bool {
    for ancestor in path.ancestors() {
        // Use `symlink_metadata` and not `metadata` because we're not
        // interested in following symlinks. If the path is a broken
        // symlink we want to still check the owner on that, instead of
        // treating it like a "NotFound".
        match ancestor.symlink_metadata() {
            Ok(meta) => {
                return unistd::getuid().as_raw() == meta.uid();
            }
            Err(ref e) if util::is_not_found_error(e) => {
                continue;
            }
            Err(ref e) if e.kind() == io::ErrorKind::PermissionDenied => {
                return false;
            }
            // Not sure how this can happen.
            Err(_) => {
                return false;
            }
        }
    }

    false
}
