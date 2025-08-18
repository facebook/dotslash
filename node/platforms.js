/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

module.exports = {
  // Keep in sync with .github/workflows/release.yml - the 'npm-publish' job's dependencies
  // MUST include the build job for each artifact listed below.
  artifactsByPlatformAndArch: {
    linux: {
      arm64: {
        // Build job: 'linux-musl-arm64'
        slug: 'linux-musl.aarch64',
        binary: 'dotslash',
      },
      x64: {
        // Build job: 'linux-musl-x86_64'
        slug: 'linux-musl.x86_64',
        binary: 'dotslash',
      },
    },
    darwin: {
      '*': {
        // Build job: 'macos'
        slug: 'macos',
        binary: 'dotslash',
      },
    },
    win32: {
      arm64: {
        // Build job: 'windows-arm64'
        slug: 'windows-arm64',
        binary: 'dotslash.exe',
      },
      x64: {
        // Build job: 'windows'
        slug: 'windows',
        binary: 'dotslash.exe',
      },
    },
  },
};
