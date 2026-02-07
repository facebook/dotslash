/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

use std::path::Path;
use std::process::Command;

use anyhow::Context as _;
use serde::Deserialize;
use serde_json::Value;

use crate::config::ArtifactEntry;
use crate::provider::Provider;
use crate::util::CommandDisplay;
use crate::util::CommandStderrDisplay;
use crate::util::FileLock;

pub struct GcsProvider {}

#[derive(Deserialize, Debug)]
struct GcsProviderConfig {
    bucket: String,
    object: String,
}

/// Environment variables checked for a bearer token, in order.
const TOKEN_ENV_VARS: &[&str] = &[
    "GOOGLE_AUTH_TOKEN",
    "CLOUDSDK_AUTH_ACCESS_TOKEN",
    "GOOGLE_OAUTH_ACCESS_TOKEN",
];

fn get_bearer_token() -> Option<String> {
    // First, check environment variables.
    let from_env = TOKEN_ENV_VARS
        .iter()
        .find_map(|var| std::env::var(var).ok())
        .filter(|t| !t.is_empty());
    if from_env.is_some() {
        return from_env;
    }

    // Fall back to gcloud, if available.
    let output = Command::new("gcloud")
        .args(["auth", "print-access-token"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let token = String::from_utf8(output.stdout).ok()?;
    let token = token.trim().to_owned();
    if token.is_empty() { None } else { Some(token) }
}

impl Provider for GcsProvider {
    fn fetch_artifact(
        &self,
        provider_config: &Value,
        destination: &Path,
        _fetch_lock: &FileLock,
        _: &ArtifactEntry,
    ) -> anyhow::Result<()> {
        let GcsProviderConfig { bucket, object } = <_>::deserialize(provider_config)?;
        let url = format!("https://storage.googleapis.com/{}/{}", bucket, object,);
        let output_arg = destination.to_str().unwrap();

        let mut command = Command::new("curl");
        command.args(["--location", "--retry", "3"]);
        command.args(["--fail", "--silent", "--show-error"]);
        if let Some(token) = get_bearer_token() {
            command.args(["-H", &format!("Authorization: Bearer {}", token)]);
        }
        command.args(["--output", output_arg]);
        command.arg(&url);

        let output = command
            .output()
            .with_context(|| format!("{}", CommandDisplay::new(&command)))
            .context("failed to run curl for GCS download")?;

        if !output.status.success() {
            return Err(anyhow::format_err!(
                "{}",
                CommandStderrDisplay::new(&output)
            ))
            .with_context(|| format!("{}", CommandDisplay::new(&command)))
            .context("curl failed to download from GCS");
        }
        Ok(())
    }
}
