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

use crate::util::fs_ctx;

fn make_tree_entries_impl(folder: &Path, read_only: bool) -> io::Result<()> {
    for entry in fs_ctx::read_dir(folder)? {
        let entry = entry?;
        let metadata = fs_ctx::symlink_metadata(entry.path())?;

        if metadata.is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            make_tree_entries_impl(&entry.path(), read_only)?;
        }

        let mut perms = metadata.permissions();
        perms.set_readonly(read_only);
        fs_ctx::set_permissions(entry.path(), perms)?;
    }

    Ok(())
}

/// Makes all entries within the specified `folder` read-only.
///
/// Takes the specified `folder` (which must point to a directory) and
/// recursively makes all entries within it read-only, but it does *not* change
/// the permissions on the folder itself. Symlinks are not followed and no
/// attempt is made to change their permissions.
pub fn make_tree_entries_read_only(folder: &Path) -> io::Result<()> {
    make_tree_entries_impl(folder, true)
}

/// Like `make_tree_entries_read_only` but does the reverse - makes the
/// entries writable.
pub fn make_tree_entries_writable(folder: &Path) -> io::Result<()> {
    make_tree_entries_impl(folder, false)
}
