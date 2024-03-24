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
use std::path::Path;
use std::process::Command;
use std::process::ExitCode;

use anyhow::format_err;
use anyhow::Context as _;

use crate::dotslash_cache::DotslashCache;
use crate::download::download_artifact;
use crate::locate::locate_artifact;
use crate::locate::update_artifact_mtime;
use crate::provider::ProviderFactory;
use crate::subcommand::run_subcommand;
use crate::subcommand::Subcommand;
use crate::subcommand::SubcommandError;
use crate::util::execv;

pub fn run<P: ProviderFactory>(mut args: ArgsOs, provider_factory: &P) -> ExitCode {
    // If there is an argument, check whether it is a valid DotSlash file.
    // If so, there is no need to parse any args: just run it!
    let err = if let Some(file_arg) = args.nth(1) {
        match run_dotslash_file(&file_arg, args, provider_factory) {
            Ok(()) => return ExitCode::SUCCESS,
            Err(err) if err.is::<SubcommandError>() => err,
            Err(err) => err.context(format_err!(
                "problem with `{}`",
                dunce::canonicalize(&file_arg)
                    .unwrap_or_else(|_| dunce::simplified(file_arg.as_ref()).to_owned())
                    .display(),
            )),
        }
    } else {
        format_err!("must specify the path to a DotSlash file")
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
                    DotSlashFlagResult::Success => return Ok(()),
                    DotSlashFlagResult::Failure(err) => return Err(err.into()),
                    DotSlashFlagResult::NoMatch => {}
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
    std::os::unix::process::CommandExt::arg0(&mut command, file_arg);

    let error = execv::execv(&mut command);

    if !is_file_not_found_error(&error) {
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
    #[cfg(unix)]
    update_artifact_mtime(&artifact_location.executable);

    // Now that we have fetched the artifact, try to execv again.
    let execv_error = execv::execv(&mut command);

    let executable = Path::new(command.get_program());

    let err_context = if is_file_not_found_error(&execv_error) {
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

    Err(format_err!(execv_error).context(err_context))
}

fn is_file_not_found_error(error_from_execv: &io::Error) -> bool {
    // If execv fails with ENOENT, that means we need to fetch the artifact.
    // This is the most likely error returned by execv.
    if error_from_execv.kind() == io::ErrorKind::NotFound {
        return true;
    }

    // On Windows, the following is already covered by the NotFound check above.
    #[cfg(unix)]
    if let Some(raw_os_error) = error_from_execv.raw_os_error() {
        // Note that this can happen if the program passed to execv is:
        //
        // ~/.cache/dotslash/obj/ha/xx/abc/extract/my_tool
        //
        // but the following is a regular file:
        //
        // ~/.cache/dotslash/obj/ha/xx/abc/extract
        //
        // This could happen if a previous release of DotSlash wrote this entry in the cache in
        // a different way that is not consistent with the current directory structure. We
        // should attempt to fetch the artifact again in this case.
        if raw_os_error == nix::errno::Errno::ENOTDIR as i32 {
            return true;
        }
    }

    false
}

enum DotSlashFlagResult {
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
fn try_parse_file_arg_as_flag(file_arg: &OsStr, args: &mut ArgsOs) -> DotSlashFlagResult {
    let subcommand = match file_arg.as_encoded_bytes() {
        b"--help" => Subcommand::Help,
        b"--version" => Subcommand::Version,
        b"--" => {
            if let Some(subcommand_arg) = args.next() {
                match subcommand_arg.to_string_lossy().parse::<Subcommand>() {
                    Ok(subcommand) => subcommand,
                    Err(err) => return DotSlashFlagResult::Failure(err),
                }
            } else {
                return DotSlashFlagResult::Failure(SubcommandError::MissingCommand);
            }
        }
        _ => return DotSlashFlagResult::NoMatch,
    };

    if let Err(err) = run_subcommand(subcommand, args) {
        DotSlashFlagResult::Failure(err)
    } else {
        DotSlashFlagResult::Success
    }
}
