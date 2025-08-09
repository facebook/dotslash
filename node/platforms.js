/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

module.exports = {
  artifactsByPlatformAndArch: {
    linux: {
      arm64: {
        slug: 'linux-musl.aarch64',
        binary: 'dotslash',
      },
      x64: {
        slug: 'linux-musl.x86_64',
        binary: 'dotslash',
      },
    },
    darwin: {
      '*': {
        slug: 'macos',
        binary: 'dotslash',
      },
    },
    win32: {
      '*': {
        slug: 'windows',
        binary: 'dotslash.exe',
      },
    },
  },
};
