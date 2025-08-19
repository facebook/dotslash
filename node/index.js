/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

'use strict';

const os = require('os');
const path = require('path');
const { artifactsByPlatformAndArch } = require('./platforms');

const artifacts = artifactsByPlatformAndArch[os.platform()];
const { slug, binary } = artifacts[os.arch()] ?? artifacts['*'];

module.exports = path.join(__dirname, 'bin', slug, binary);
