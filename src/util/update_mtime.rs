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

use filetime::FileTime;

/// DotSlash can unpack old artifacts which can be reaped by tools like
/// tmpwatch or tmpreaper. Those tools work better using the mtime rather than
/// atime which is why we update the mtime. But this doesn't work on
/// Windows sometimes.
pub fn update_mtime(executable: &Path) -> io::Result<()> {
    filetime::set_file_mtime(executable, FileTime::now())
}
