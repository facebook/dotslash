/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::path::Path;
use std::process::Command;

use anyhow::Context as _;
use serde::Deserialize;
use serde_jsonrc::value::Value;

use crate::config::ArtifactEntry;
use crate::provider::Provider;
use crate::util::CommandDisplay;
use crate::util::CommandStderrDisplay;
use crate::util::FileLock;

pub struct GitHubReleaseProvider {}

#[derive(Deserialize, Debug)]
struct GitHubReleaseProviderConfig {
    tag: String,
    repo: String,
    name: String,
}

impl Provider for GitHubReleaseProvider {
    fn fetch_artifact(
        &self,
        provider_config: &Value,
        destination: &Path,
        _fetch_lock: &FileLock,
        _artifact_entry: &ArtifactEntry,
    ) -> anyhow::Result<()> {
        let GitHubReleaseProviderConfig { tag, repo, name } = <_>::deserialize(provider_config)?;
        let mut command = Command::new("gh");
        command
            .arg("release")
            .arg("download")
            .arg(tag)
            .arg("--repo")
            .arg(repo)
            .arg("--pattern")
            // We want to match an a release by name, but unfortunately,
            // `gh release download` only supports --pattern, which takes a
            // regex. Adding ^ and $ as anchors only seems to break things.
            .arg(regex_escape(&name))
            .arg("--output")
            .arg(destination);

        let output = command
            .output()
            .with_context(|| format!("{}", CommandDisplay::new(&command)))
            .context("failed to run the GitHub CLI")?;

        if !output.status.success() {
            return Err(anyhow::format_err!(
                "{}",
                CommandStderrDisplay::new(&output)
            ))
            .with_context(|| format!("{}", CommandDisplay::new(&command)))
            .context("the GitHub CLI failed");
        }

        Ok(())
    }
}

/// We want the functionality comparable to regex::escape() without pulling in
/// the entire crate.
fn regex_escape(s: &str) -> String {
    s.chars().fold(
        // Releases filenames likely have at least one `.` in there that needs
        // to be escaped, so add some padding, by default.
        String::with_capacity(s.len() + 4),
        |mut output, c| {
            if let '\\' | '.' | '+' | '*' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '^'
            | '$' = c
            {
                output.push('\\');
            };
            output.push(c);
            output
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regex_escape_no_quotable_chars() {
        assert_eq!("foo", regex_escape("foo"));
        assert_eq!("FOO", regex_escape("FOO"));
        // Spaces do not get escaped.
        assert_eq!("foo bar", regex_escape("foo bar"));
        // Angle brackets do not get escaped.
        assert_eq!("<abc>", regex_escape("<abc>"));
        // Slashes do not get escaped.
        assert_eq!("foo/bar", regex_escape("foo/bar"));
    }

    #[test]
    fn regex_escape_punctuation() {
        assert_eq!("abc\\.tar\\.gz", regex_escape("abc.tar.gz"));
        assert_eq!("what\\?!\\?", regex_escape("what?!?"));
    }

    #[test]
    fn regex_escape_brackets() {
        assert_eq!("\\[abc\\]", regex_escape("[abc]"));
        assert_eq!("\\{abc\\}", regex_escape("{abc}"));
        assert_eq!("\\(abc\\)", regex_escape("(abc)"));
    }

    #[test]
    fn regex_escape_anchors() {
        assert_eq!("\\^foobarbaz\\$", regex_escape("^foobarbaz$"));
    }

    #[test]
    fn regex_escape_quantifiers() {
        assert_eq!("https\\?://", regex_escape("https?://"));
        assert_eq!("foo\\+foo\\+", regex_escape("foo+foo+"));
        assert_eq!("foo\\*foo\\*", regex_escape("foo*foo*"));
    }

    #[test]
    fn regex_escape_alternation() {
        assert_eq!("foo\\|bar", regex_escape("foo|bar"));
    }
}
