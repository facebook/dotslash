/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

use std::fmt;

use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DigestError {
    #[error("invalid hash characters `{0}`")]
    InvalidHashCharacters(String),

    #[error("invalid hash length `{0}`")]
    InvalidHashLength(String),
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
#[serde(try_from = "String")]
pub struct Digest(String);

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for Digest {
    type Error = DigestError;

    fn try_from(hash: String) -> Result<Self, Self::Error> {
        if !hash.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')) {
            Err(DigestError::InvalidHashCharacters(hash))
        } else if hash.len() != 64 {
            Err(DigestError::InvalidHashLength(hash))
        } else {
            Ok(Digest(hash))
        }
    }
}

impl Digest {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use super::*;

    #[test]
    fn test_digest_try_from_string_invalid() {
        assert_matches!(
            Digest::try_from("".to_owned()),
            Err(DigestError::InvalidHashLength(x)) if x.is_empty()
        );
        assert_matches!(
            Digest::try_from("z".to_owned()),
            Err(DigestError::InvalidHashCharacters(x)) if x == "z"
        );
        assert_matches!(
            Digest::try_from("7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d906".to_owned()),
            Err(DigestError::InvalidHashLength(x))
            if x == "7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d906"
        );
        assert_matches!(
            Digest::try_from("7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d90690".to_owned()),
            Err(DigestError::InvalidHashLength(x))
            if x == "7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d90690"
        );
    }

    #[test]
    fn test_digest_try_from_string_valid() {
        let digest = Digest::try_from(
            "7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069".to_owned(),
        )
        .unwrap();
        let expected =
            Digest("7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069".to_owned());
        assert_eq!(digest, expected);
        assert_eq!(
            format!("{}", digest),
            "7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069",
        );
    }
}
