/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

//! `Display` wrappers for pretty printing.

use std::fmt;
use std::process::Command;
use std::process::Output;

/// TODO
#[derive(Debug)]
#[must_use]
pub struct CommandDisplay<'a>(&'a Command);

impl<'a> CommandDisplay<'a> {
    pub fn new(cmd: &'a Command) -> Self {
        CommandDisplay(cmd)
    }
}

impl fmt::Display for CommandDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Properly quote when necessary.
        f.write_str(&self.0.get_program().to_string_lossy())?;
        for arg in self.0.get_args() {
            f.write_str(" ")?;
            f.write_str(&arg.to_string_lossy())?;
        }
        Ok(())
    }
}

/// TODO
#[derive(Debug)]
#[must_use]
pub struct CommandStderrDisplay<'a>(&'a Output);

impl<'a> CommandStderrDisplay<'a> {
    pub fn new(output: &'a Output) -> Self {
        CommandStderrDisplay(output)
    }
}

impl fmt::Display for CommandStderrDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.status.success() {
            write!(f, "command finished with ")?;
        } else {
            write!(f, "command failed with ")?;
        }
        if let Some(exit_code) = self.0.status.code() {
            write!(f, "exit code {} and ", exit_code)?;
        }
        write!(f, "stderr: ")?;
        if self.0.stderr.is_empty() {
            write!(f, "(empty stderr)")?;
        } else {
            // TODO: Truncate stderr.
            write!(f, "{}", String::from_utf8_lossy(&self.0.stderr).trim_end())?;
        }
        Ok(())
    }
}

/// Pretty sorted lists for use in error messages.
///
/// - expected nothing
/// - expected `a`
/// - expected `a`, `b`
/// - expected `a`, `b`, `c`
#[derive(Clone, Debug)]
pub struct ListOf<T>(Vec<T>);

impl<T> ListOf<T>
where
    T: fmt::Display + Ord,
{
    pub fn new<I>(it: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut list = it.into_iter().collect::<Vec<_>>();
        list.sort();
        ListOf(list)
    }
}

impl<T> fmt::Display for ListOf<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut it = self.0.iter();
        match it.next() {
            None => write!(f, "nothing"),
            Some(first) => {
                write!(f, "`{}`", first)?;
                for item in it {
                    write!(f, ", `{}`", item)?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_of() {
        assert_eq!(format!("{}", ListOf::new(&[] as &[&str])), "nothing");
        assert_eq!(format!("{}", ListOf::new(&["a"])), "`a`");
        assert_eq!(format!("{}", ListOf::new(&["a", "b"])), "`a`, `b`");
        assert_eq!(
            format!("{}", ListOf::new(&["c", "a", "b"])),
            "`a`, `b`, `c`",
        );
    }
}
