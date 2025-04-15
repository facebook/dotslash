/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::env::ArgsOs;
use std::ffi::OsStr;
use std::fs;
use std::io;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;
use std::process::ExitCode;

use anyhow::Context as _;

#[cfg(unix)]
use crate::config::Arg0;
use crate::dotslash_cache::DotslashCache;
use crate::download::download_artifact;
use crate::locate::locate_artifact;
use crate::provider::ProviderFactory;
use crate::subcommand::Subcommand;
use crate::subcommand::SubcommandError;
use crate::subcommand::run_subcommand;
use crate::util;

pub fn run<P: ProviderFactory>(mut args: ArgsOs, provider_factory: &P) -> ExitCode {
    // If there is an argument, check whether it is a valid DotSlash file.
    // If so, there is no need to parse any args: just run it!
    let err = if let Some(file_arg) = args.nth(1) {
        match run_dotslash_file(&file_arg, args, provider_factory) {
            Ok(()) => return ExitCode::SUCCESS,
            Err(err) if err.is::<SubcommandError>() => err,
            Err(err) => err.context(format!(
                "problem with `{}`",
                dunce::canonicalize(&file_arg)
                    .unwrap_or_else(|_| dunce::simplified(file_arg.as_ref()).to_owned())
                    .display(),
            )),
        }
    } else {
        anyhow::format_err!("must specify the path to a DotSlash file")
    };

    eprintln!("dotslash error: {}", err);
    for cause in err.chain().skip(1) {
        eprintln!("caused by: {}", cause);
    }

    ExitCode::FAILURE
}

fn run_dotslash_file<P: ProviderFactory>(
    file_arg: &OsStr,
    mut args: ArgsOs,
    provider_factory: &P,
) -> anyhow::Result<()> {
    let dotslash_data = match fs::read_to_string(file_arg) {
        Ok(data) => data,
        Err(err) => {
            if err.kind() == io::ErrorKind::NotFound {
                match try_parse_file_arg_as_flag(file_arg, &mut args) {
                    DotslashFlagResult::Success => return Ok(()),
                    DotslashFlagResult::Failure(err) => return Err(err.into()),
                    DotslashFlagResult::NoMatch => {}
                }
            }
            return Err(err).context("failed to read DotSlash file");
        }
    };

    let dotslash_cache = DotslashCache::new();
    let (artifact_entry, artifact_location) = locate_artifact(&dotslash_data, &dotslash_cache)?;

    let mut command = Command::new(&artifact_location.executable);
    command.args(args);

    #[cfg(unix)]
    match artifact_location.arg0 {
        Arg0::DotslashFile => {
            command.arg0(file_arg);
        }
        Arg0::UnderlyingExecutable => {
            // This is what the OS already does...
        }
    }

    let error = util::execv(&mut command);

    if !util::is_not_found_error(&error) {
        return Err(error).context(format!(
            "failed to execute `{}`",
            artifact_location.executable.display()
        ));
    }

    download_artifact(&artifact_entry, &artifact_location, provider_factory).with_context(
        || {
            format!(
                "failed to download artifact into cache `{}` artifact location `{}`",
                dotslash_cache.cache_dir().display(),
                artifact_location.artifact_directory.display()
            )
        },
    )?;

    // Since we just unpacked the executable for the first time, we can
    // afford to pay the macOS cost mentioned above.
    if cfg!(unix) {
        let _ = util::update_mtime(&artifact_location.executable);
    }

    // Now that we have fetched the artifact, try to execv again.
    let execv_error = util::execv(&mut command);

    let executable = Path::new(command.get_program());

    let err_context = if util::is_not_found_error(&execv_error) {
        if executable.exists() {
            // On Unix, if an interpreter in a shebang does not exist, the
            // exec returns ENOENT. It is unclear under what other
            // circumstances this happens, so the message here should not
            // be too specific.
            format!(
                "failed to execute `{}` even though it exists (interpreter problem)",
                executable.display(),
            )
        } else {
            format!(
                "failed to execute `{}` because it was not found",
                executable.display(),
            )
        }
    } else {
        format!("failed to execute `{}`", executable.display())
    };

    Err(execv_error).context(err_context)
}

enum DotslashFlagResult {
    /// Arguments were well-formed and the flag was handled successfully.
    Success,
    /// Arguments were ill-formed or there was an error processing the subcommand.
    Failure(SubcommandError),
    /// The file arg did not match a known subcommand.
    NoMatch,
}

/// Called when opening file_arg returns ENOENT in [run_dotslash_file()]. In general, we do not
/// attempt to support sophisticated arg parsing in DotSlash itself because normally arguments
/// should be passed transparently to the underlying executable.
fn try_parse_file_arg_as_flag(file_arg: &OsStr, args: &mut ArgsOs) -> DotslashFlagResult {
    let subcommand = match file_arg.as_encoded_bytes() {
        b"--help" => Subcommand::Help,
        b"--version" => Subcommand::Version,
        b"--" => {
            if let Some(subcommand_arg) = args.next() {
                match subcommand_arg.to_string_lossy().parse::<Subcommand>() {
                    Ok(subcommand) => subcommand,
                    Err(err) => return DotslashFlagResult::Failure(err),
                }
            } else {
                return DotslashFlagResult::Failure(SubcommandError::MissingCommand);
            }
        }
        _ => return DotslashFlagResult::NoMatch,
    };

    if let Err(err) = run_subcommand(subcommand, args) {
        DotslashFlagResult::Failure(err)
    } else {
        DotslashFlagResult::Success
    }
}
