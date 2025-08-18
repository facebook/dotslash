#!/usr/bin/env node
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

const { parseArgs } = require('util');
const { promises: fs } = require('fs');
const path = require('path');
const os = require('os');
const { artifactsByPlatformAndArch } = require('../platforms');
const { spawnSync } = require('child_process');

const PACKAGE_JSON_PATH = path.join(__dirname, '..', 'package.json');
const BIN_PATH = path.join(__dirname, '..', 'bin');
const GITHUB_REPO = 'facebook/dotslash';

async function main() {
  const {
    values: { tag, prerelease },
  } = parseArgs({
    options: {
      tag: {
        short: 't',
        type: 'string',
      },
      prerelease: {
        type: 'boolean',
      },
    },
  });

  if (tag == null) {
    throw new Error('Missing required argument: --tag');
  }

  await deleteOldBinaries();
  const versionInfo = getVersionInfoFromArgs(tag, prerelease);
  if (versionInfo.prerelease && !prerelease) {
    console.warn(
      `Building a prerelease version because the tag ${tag} does not seem to denote a valid semver string.`,
    );
  }
  await fetchBinaries(tag);
  await updatePackageJson(versionInfo);
}

function getVersionInfoFromArgs(tag, prerelease) {
  // https://semver.org/#is-there-a-suggested-regular-expression-regex-to-check-a-semver-string
  const SEMVER_WITH_LEADING_V =
    /^v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$/;
  if (SEMVER_WITH_LEADING_V.test(tag)) {
    return { tag, version: tag.slice(1), prerelease };
  }
  return {
    tag,
    version: '0.0.0-' + tag.replaceAll(/[^0-9a-zA-Z-]+/g, '-'),
    prerelease: true,
  };
}

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

async function fetchBinaries(tag) {
  const scratchDir = await fs.mkdtemp(path.join(os.tmpdir(), 'dotslash'));
  try {
    for (const [platform, archToArtifact] of Object.entries(
      artifactsByPlatformAndArch,
    )) {
      for (const [arch, descriptor] of Object.entries(archToArtifact)) {
        const { slug, binary } = descriptor;
        console.log(
          `Fetching ${platform} ${arch} binary (${slug} ${binary})...`,
        );
        const tarballName = `dotslash-${slug}.tar.gz`;
        const downloadURL = `https://github.com/${GITHUB_REPO}/releases/download/${tag}/${tarballName}`;
        const tarballPath = path.join(scratchDir, tarballName);
        await download(downloadURL, tarballPath);
        const extractDir = path.join(BIN_PATH, slug);
        await fs.mkdir(extractDir, { recursive: true });
        spawnSyncSafe('tar', ['-xzf', tarballPath, '-C', extractDir]);
        await fs.rm(tarballPath);
        if (!(await existsAndIsExecutable(path.join(extractDir, binary)))) {
          throw new Error(
            `Failed to extract ${binary} from ${tarballPath} to ${extractDir}`,
          );
        }
      }
    }
  } finally {
    await fs.rm(scratchDir, { force: true, recursive: true });
  }
}

async function existsAndIsExecutable(filePath) {
  try {
    await fs.access(filePath, fs.constants.R_OK | fs.constants.X_OK);
    return true;
  } catch (e) {
    return false;
  }
}

async function download(url, dest) {
  spawnSyncSafe('curl', ['-L', url, '-o', dest, '--fail-with-body'], {
    stdio: 'inherit',
  });
}

async function updatePackageJson({ version, prerelease }) {
  const packageJson = await fs.readFile(PACKAGE_JSON_PATH, 'utf8');
  const packageJsonObj = JSON.parse(packageJson);
  packageJsonObj.version = version + (prerelease ? '-' + Date.now() : '');
  await fs.writeFile(
    PACKAGE_JSON_PATH,
    JSON.stringify(packageJsonObj, null, 2) + '\n',
  );
  console.log('Updated package.json to version', packageJsonObj.version);
}

function spawnSyncSafe(command, args, options) {
  args = args ?? [];
  console.log('Running:', command, args.join(' '));
  const result = spawnSync(command, args, options);
  if (result.status != null && result.status !== 0) {
    throw new Error(`Command ${command} exited with status ${result.status}`);
  }
  if (result.error != null) {
    throw result.error;
  }
  if (result.signal != null) {
    throw new Error(
      `Command ${command} was killed with signal ${result.signal}`,
    );
  }
  return result;
}

main().catch((e) => {
  console.error(e);
  process.exitCode = 1;
});
