/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::env::ArgsOs;
use std::ffi::OsString;
use std::fmt;
use std::io;
use std::str::FromStr;

use anyhow::Context as _;
use digest::Digest as _;
use sha2::Sha256;
use thiserror::Error;

use crate::config::REQUIRED_HEADER;
use crate::config::parse_file;
use crate::default_provider_factory::DefaultProviderFactory;
use crate::dotslash_cache::DotslashCache;
use crate::download::download_artifact;
use crate::locate::locate_artifact;
use crate::platform::SUPPORTED_PLATFORM;
use crate::print_entry_for_url::print_entry_for_url;
use crate::util;
use crate::util::fs_ctx;

#[derive(Debug)]
pub enum Subcommand {
    /// Similar to running `b3sum`, though takes exactly one argument and prints
    /// only the hash
    B3Sum,

    /// Clean the cache directory
    Clean,

    /// Create a the artifact entry for DotSlash file from a URL
    CreateUrlEntry,

    /// Print the cache directory
    CacheDir,

    /// Fetch and cache an artifact but do not execute it
    Fetch,

    /// Parse a DotSlash file and print its data as JSON
    Parse,

    /// Similar to running `shasum -a 256`, though takes exactly one argument
    /// and prints only the hash
    Sha256,

    /// Version
    Version,

    /// Help
    Help,
}

impl fmt::Display for Subcommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::B3Sum => "b3sum",
            Self::Clean => "clean",
            Self::CreateUrlEntry => "create-url-entry",
            Self::CacheDir => "cache-dir",
            Self::Fetch => "fetch",
            Self::Parse => "parse",
            Self::Sha256 => "sha256",
            Self::Version => "version",
            Self::Help => "help",
        })
    }
}

impl FromStr for Subcommand {
    type Err = SubcommandError;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "b3sum" => Ok(Subcommand::B3Sum),
            "clean" => Ok(Subcommand::Clean),
            "create-url-entry" => Ok(Subcommand::CreateUrlEntry),
            "cache-dir" => Ok(Subcommand::CacheDir),
            "fetch" => Ok(Subcommand::Fetch),
            "parse" => Ok(Subcommand::Parse),
            "sha256" => Ok(Subcommand::Sha256),
            "version" => Ok(Subcommand::Version),
            "help" => Ok(Subcommand::Help),
            _ => Err(SubcommandError::UnknownCommand(name.to_owned())),
        }
    }
}

#[derive(Error, Debug)]
pub enum SubcommandError {
    #[error(
        "no subcommand passed to '--'

See `dotslash --help` for more information."
    )]
    MissingCommand,

    #[error(
        "unknown subcommand passed to '--': `{0}`

See `dotslash --help` for more information."
    )]
    UnknownCommand(String),

    #[error("'{0}' command failed")]
    Other(Subcommand, #[source] anyhow::Error),
}

pub fn run_subcommand(subcommand: Subcommand, args: &mut ArgsOs) -> Result<(), SubcommandError> {
    run_subcommand_impl(&subcommand, args).map_err(|x| SubcommandError::Other(subcommand, x))
}

fn run_subcommand_impl(subcommand: &Subcommand, args: &mut ArgsOs) -> anyhow::Result<()> {
    match subcommand {
        Subcommand::B3Sum => {
            let file_arg = take_exactly_one_arg(args)?;
            // TODO: read from stdin if file_arg is `-`
            let mut file = fs_ctx::file_open(file_arg)?;
            let mut hasher = blake3::Hasher::new();
            io::copy(&mut file, &mut hasher)?;
            let hex_digest = format!("{:x}", hasher.finalize());
            println!("{}", hex_digest);
        }

        Subcommand::Clean => {
            if args.next().is_some() {
                return Err(anyhow::format_err!(
                    "expected no arguments but received some",
                ));
            }

            let dotslash_cache = DotslashCache::new();
            eprintln!("Cleaning `{}`", dotslash_cache.cache_dir().display());
            // Make sure nothing is read-only first.
            let _ = util::make_tree_entries_writable(dotslash_cache.cache_dir());
            // Then delete the contents.
            fs_ctx::remove_dir_all(dotslash_cache.cache_dir())?;
        }

        Subcommand::CreateUrlEntry => {
            let url = take_exactly_one_arg(args)?;
            print_entry_for_url(&url)?;
        }

        Subcommand::CacheDir => {
            if args.next().is_some() {
                return Err(anyhow::format_err!(
                    "expected no arguments but received some",
                ));
            }

            let dotslash_cache = DotslashCache::new();
            println!("{}", dotslash_cache.cache_dir().display());
        }

        Subcommand::Fetch => {
            let file_arg = take_exactly_one_arg(args)?;
            let dotslash_data = fs_ctx::read_to_string(file_arg)?;
            let dotslash_cache = DotslashCache::new();
            let (artifact_entry, artifact_location) =
                locate_artifact(&dotslash_data, &dotslash_cache)?;
            if !artifact_location.executable.exists() {
                let provider_factory = DefaultProviderFactory {};
                download_artifact(&artifact_entry, &artifact_location, &provider_factory)?;
            }
            println!("{}", artifact_location.executable.display());
        }

        Subcommand::Parse => {
            let file_arg = take_exactly_one_arg(args)?;
            let dotslash_data = fs_ctx::read_to_string(file_arg)?;
            let (original_json, _config_file) =
                parse_file(&dotslash_data).context("failed to parse file")?;
            let json =
                serde_jsonrc::to_string(&original_json).context("failed to serialize value")?;
            println!("{json}");
        }

        Subcommand::Version => {
            if args.next().is_some() {
                return Err(anyhow::format_err!(
                    "expected no arguments but received some",
                ));
            }

            println!("DotSlash {}", env!("CARGO_PKG_VERSION"));
        }

        Subcommand::Help => {
            if args.next().is_some() {
                return Err(anyhow::format_err!(
                    "expected no arguments but received some",
                ));
            }

            eprint!(
                r##"usage: dotslash DOTSLASH_FILE [OPTIONS]

DOTSLASH_FILE must be a file that starts with `{}`
and contains a JSON body tells DotSlash how to fetch and run the executable
that DOTSLASH_FILE represents.

All OPTIONS will be forwarded directly to the executable identified by
DOTSLASH_FILE.

Supported platform: {}

Your DotSlash cache is: {}

dotslash also has these special experimental commands:
  dotslash --help                   Print this message
  dotslash --version                Print the version of dotslash
  dotslash -- b3sum FILE            Compute blake3 hash
  dotslash -- clean                 Clean dotslash cache
  dotslash -- create-url-entry URL  Generate "http" provider entry
  dotslash -- cache-dir             Print path to the cache directory
  dotslash -- fetch DOTSLASH_FILE   Prepare for execution, but print exe path
                                    instead of executing
  dotslash -- parse DOTSLASH_FILE   Parse the dotslash file
  dotslash -- sha256 FILE           Compute sha256 sum of the file

Learn more at {}
"##,
                REQUIRED_HEADER,
                SUPPORTED_PLATFORM,
                DotslashCache::new().cache_dir().display(),
                env!("CARGO_PKG_HOMEPAGE"),
            );
        }

        Subcommand::Sha256 => {
            let file_arg = take_exactly_one_arg(args)?;
            // TODO: read from stdin if file_arg is `-`
            let mut file = fs_ctx::file_open(file_arg)?;
            let mut hasher = Sha256::new();
            io::copy(&mut file, &mut hasher)?;
            let hex_digest = format!("{:x}", hasher.finalize());
            println!("{}", hex_digest);
        }
    };

    Ok(())
}

fn take_exactly_one_arg(args: &mut ArgsOs) -> anyhow::Result<OsString> {
    match (args.next(), args.next()) {
        (None, _) => Err(anyhow::format_err!(
            "expected exactly one argument but received none"
        )),
        (Some(_), Some(_)) => Err(anyhow::format_err!(
            "expected exactly one argument but received more"
        )),
        (Some(arg), None) => Ok(arg),
    }
}
