/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

//! Cross-platform approximation of execv(3) on Unix.
//! On Unix, this will use exec(2) directly.
//! On Windows, this will spawn a child process with stdin/stdout/stderr
//! inherited from the current process and will exit() with the result.

use std::io;
use std::process::Command;

#[cfg(unix)]
pub fn execv(command: &mut Command) -> io::Error {
    std::os::unix::process::CommandExt::exec(command)
}

#[cfg(windows)]
pub fn execv(command: &mut Command) -> io::Error {
    // Starting in Rust 1.62.0, batch scripts are passed to `cmd.exe /c` rather
    // than directly to `CreateProcessW`. So if a script doesn't exist, we get
    // a `cmd.exe` process error, rather than a system error. We need to be
    // able to distinguish between "Not Found" and process errors because
    // artifact downloading depends on this.
    //
    // See https://github.com/rust-lang/rust/pull/95246

    use std::path::Path;
    use std::process;
    use std::process::Child;

    fn spawn(command: &mut Command) -> io::Result<Child> {
        let program = Path::new(command.get_program());
        // This check must be done before the process is spawned, otherwise
        // we'll get a "The system cannot find the path specified."
        // printed to stderr.
        if program.extension().is_some_and(|x| {
            // .bat` and `.cmd` are the extensions checked for in std:
            // https://github.com/rust-lang/rust/blob/1.64.0/library/std/src/sys/windows/process.rs#L266-L269
            x.eq_ignore_ascii_case("bat") || x.eq_ignore_ascii_case("cmd")
        }) && !program.exists()
        {
            // Mimic the error pre-1.63.0 error.
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "The system cannot find the file specified. (os error 2)",
            ))
        } else {
            command.spawn()
        }
    }

    match spawn(command) {
        Ok(mut child) => match child.wait() {
            Ok(exit_code) => process::exit(exit_code.code().unwrap_or(1)),
            Err(e) => e,
        },
        // This could be ENOENT if the executable does not exist.
        Err(e) => e,
    }
}
