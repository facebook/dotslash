/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::path::Path;

use anyhow::Context as _;
use serde::Deserialize;
use serde_jsonrc::value::Value;

use crate::config::ArtifactEntry;
use crate::curl::CurlCommand;
use crate::curl::FetchContext;
use crate::provider::Provider;
use crate::util::FileLock;

pub struct HttpProvider {}

#[derive(Deserialize, Debug)]
struct HttpProviderConfig {
    url: String,
}

impl Provider for HttpProvider {
    fn fetch_artifact(
        &self,
        provider_config: &Value,
        destination: &Path,
        _fetch_lock: &FileLock,
        artifact_entry: &ArtifactEntry,
    ) -> anyhow::Result<()> {
        let HttpProviderConfig { url } = <_>::deserialize(provider_config)?;
        let curl_cmd = CurlCommand::new(url.as_ref());
        // Currently, we always disable the progress bar, but we plan to add a
        // configuration option to enable it.
        let show_progress = false;
        let fetch_context = FetchContext {
            artifact_name: url.as_str(),
            content_length: artifact_entry.size,
            show_progress,
        };
        curl_cmd
            .get_request(destination, &fetch_context)
            .with_context(|| format!("failed to fetch `{}`", url))?;
        Ok(())
    }
}
