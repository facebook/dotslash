/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

mod artifact_location;
mod artifact_path;
mod config;
mod curl;
mod decompress;
mod digest;
mod dotslash_cache;
mod download;
mod execution;
mod fetch_method;
mod github_release_provider;
mod http_provider;
mod platform;
mod print_entry_for_url;
mod progress;
mod provider;
mod subcommand;
mod util;

use std::env;
use std::process::ExitCode;

use anyhow::format_err;

use crate::github_release_provider::GitHubReleaseProvider;
use crate::http_provider::HttpProvider;
use crate::provider::Provider;
use crate::provider::ProviderFactory;

fn main() -> ExitCode {
    let args = env::args_os();
    let provider_factory = DefaultProviderFactory {};
    execution::run(args, &provider_factory)
}

struct DefaultProviderFactory;

impl ProviderFactory for DefaultProviderFactory {
    fn get_provider(&self, provider_type: &str) -> anyhow::Result<Box<dyn Provider>> {
        match provider_type {
            "http" => Ok(Box::new(HttpProvider {})),
            "github-release" => Ok(Box::new(GitHubReleaseProvider {})),
            _ => Err(format_err!("unknown provider type: `{}`", provider_type)),
        }
    }
}
