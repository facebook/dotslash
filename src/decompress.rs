/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::io;
use std::io::Read;
use std::path::Path;

use crate::util::fs_ctx;

/// Attempts to extract the tar archive into the specified directory.
/// To extract, it uses the tar crate (https://crates.io/crates/tar) directly.
/// Those who create compressed artifacts for DotSlash are responsible for
/// ensuring they can be decompressed with its version of tar.
pub fn untar(tar_file: &Path, destination_dir: &Path, is_tar_gz: bool) -> io::Result<()> {
    // The destination dir is canonicalized for the benefit of Windows, but we
    // do it on all platforms for consistency of behavior.
    //
    // Windows has a path length limit of 255 chars. "Extended-length paths"[1]
    // are paths starting with `\\?\`. These are not subject to the length
    // limit, but have other issues: they cannot use forward slashes.
    //
    // `fs::canonicalize` will both prefix the path with `\\?\` and normalize
    // the slashes[2]. This is important because we don't know the depth of the
    // tarball file structure (so we need to avoid possible path length
    // limits), and we don't know if the destination path is mixing slashes.
    //
    // We only use extended-length paths here and not earlier because you
    // can't exec `.bat` files with `\\?\` (although `.exe` files are ok).
    //
    // We canonicalize for all platforms because `fs::canonicalize` can
    // error[3] and not everyone can test on Windows.
    //
    // [1] https://docs.microsoft.com/en-us/windows/desktop/FileIO/naming-a-file#maxpath
    // [2] https://doc.rust-lang.org/std/fs/fn.canonicalize.html#platform-specific-behavior
    // [3] https://doc.rust-lang.org/std/fs/fn.canonicalize.html#errors

    fs_ctx::create_dir_all(destination_dir)?;
    let destination_dir = fs_ctx::canonicalize(destination_dir)?;
    let file = fs_ctx::file_open(tar_file)?;
    if is_tar_gz {
        let decoder = flate2::read::GzDecoder::new(file);
        let archive = tar::Archive::new(decoder);
        unpack(archive, &destination_dir)
    } else {
        let archive = tar::Archive::new(file);
        unpack(archive, &destination_dir)
    }
}

pub fn unpack<R: Read>(mut archive: tar::Archive<R>, destination_dir: &Path) -> io::Result<()> {
    archive.set_preserve_permissions(true);
    archive.set_preserve_mtime(true);
    archive.unpack(destination_dir)
}
