/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

use crate::github_release_provider::GitHubReleaseProvider;
use crate::http_provider::HttpProvider;
use crate::provider::Provider;
use crate::provider::ProviderFactory;
use crate::s3_provider::S3Provider;

pub struct DefaultProviderFactory;

impl ProviderFactory for DefaultProviderFactory {
    fn get_provider(&self, provider_type: &str) -> anyhow::Result<Box<dyn Provider>> {
        match provider_type {
            "http" => Ok(Box::new(HttpProvider {})),
            "github-release" => Ok(Box::new(GitHubReleaseProvider {})),
            "s3" => Ok(Box::new(S3Provider {})),
            _ => Err(anyhow::format_err!(
                "unknown provider type: `{provider_type}`",
            )),
        }
    }
}
