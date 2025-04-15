/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::env;
use std::ffi::OsStr;
use std::io;
use std::iter;
use std::path::Path;
use std::path::PathBuf;

use snapbox::Data;
use snapbox::data::DataFormat;

pub fn current_dir() -> io::Result<PathBuf> {
    env::current_dir()
}

pub fn dotslash_bin() -> PathBuf {
    env!("CARGO_BIN_EXE_dotslash").into()
}

pub fn envs() -> impl IntoIterator<Item = (impl AsRef<OsStr>, impl AsRef<OsStr>)> {
    iter::empty::<(&str, &str)>()
}

pub fn snapshot_file(name: &str) -> Data {
    let path = Path::new(".").join("tests").join("snapshots").join(name);
    Data::read_from(&path, Some(DataFormat::Text))
}
