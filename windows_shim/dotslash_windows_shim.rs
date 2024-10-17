/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

//! This is the "DotSlash Windows Shim" using Rust's standard library.
//! Unfortunately even with an optimized libstd, the resulting binary is
//! too large for comfortably checking into source control.
//!
//! This implementation serves as a "Reference Implementation" for the
//! upcoming no_std pure-Windows-API version.

use std::env;
use std::io;
use std::process::Command;

fn main() {
    let exe_path =
        env::current_exe().expect("dotslash-windows-shim: could not get module filename.");
    let filename = exe_path.file_stem().unwrap();
    let exe_path = exe_path.with_file_name(filename);

    let mut command = Command::new("dotslash");
    command.arg(exe_path);
    command.args(env::args_os().skip(1));

    match command.status() {
        Ok(exit_code) => std::process::exit(exit_code.code().unwrap_or(1)),
        Err(err) => {
            if err.kind() == io::ErrorKind::NotFound {
                eprintln!("dotslash-windows-shim: dotslash executable not found.");
            } else {
                eprintln!("dotslash-windows-shim: `{}`.", err);
            };
            std::process::exit(1);
        }
    }
}
