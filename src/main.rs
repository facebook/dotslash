/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

mod artifact_location;
mod artifact_path;
mod config;
mod curl;
mod default_provider_factory;
mod digest;
mod dotslash_cache;
mod download;
mod execution;
mod fetch_method;
mod github_release_provider;
mod http_provider;
mod locate;
mod platform;
mod print_entry_for_url;
mod provider;
mod s3_provider;
mod subcommand;
mod util;

use std::env;
use std::process::ExitCode;

use crate::default_provider_factory::DefaultProviderFactory;

fn main() -> ExitCode {
    let args = env::args_os();
    let provider_factory = DefaultProviderFactory {};
    execution::run(args, &provider_factory)
}
