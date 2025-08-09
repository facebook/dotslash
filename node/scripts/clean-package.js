#!/usr/bin/env node
/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

'use strict';

const { promises: fs } = require('fs');
const path = require('path');

const PACKAGE_JSON_PATH = path.join(__dirname, '..', 'package.json');
const BIN_PATH = path.join(__dirname, '..', 'bin');
const DEFAULT_VERSION = '0.0.0-dev';

async function deleteOldBinaries() {
  const entries = await fs.readdir(BIN_PATH, { withFileTypes: true });
  for (const entry of entries) {
    if (!entry.isDirectory()) {
      continue;
    }
    await fs.rm(path.join(BIN_PATH, entry.name), {
      recursive: true,
      force: true,
    });
  }
}

async function cleanPackageJson() {
  const packageJson = await fs.readFile(PACKAGE_JSON_PATH, 'utf8');
  const packageJsonObj = JSON.parse(packageJson);
  packageJsonObj.version = DEFAULT_VERSION;
  await fs.writeFile(
    PACKAGE_JSON_PATH,
    JSON.stringify(packageJsonObj, null, 2) + '\n',
  );
}

async function main() {
  await deleteOldBinaries();
  await cleanPackageJson();
}

module.exports = { deleteOldBinaries };

if (require.main === module) {
  main().catch((err) => {
    console.error(err);
    process.exitCode = 1;
  });
}
