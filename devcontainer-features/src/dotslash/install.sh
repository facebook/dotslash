#!/usr/bin/env bash
# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under both the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree and the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree.

set -o allexport
set -o errexit
set -o noclobber
set -o nounset
set -o pipefail

ensure_dependencies() {
  apt-get update -y
  DEBIAN_FRONTEND=noninteractive apt-get -y install --no-install-recommends --no-install-suggests \
    ca-certificates \
    curl \
    tar
  apt-get clean
  rm -rf /var/lib/apt/lists/*
}

download() {
  local version="$1"
  local url
  if [ "${version}" = "latest" ]; then
    url="https://github.com/facebook/dotslash/releases/latest/download/dotslash-linux-musl.$(uname -m).tar.gz"
  else
    url="https://github.com/facebook/dotslash/releases/download/${version}/dotslash-linux-musl.$(uname -m).tar.gz"
  fi

  # First, verify the release exists!
  echo "Fetching version ${version} from ${url}..."
  local http_status
  http_status=$(curl -s -o /dev/null -w '%{http_code}' "${url}")
  if [ "${http_status}" -ne 200 ] && [ "${http_status}" -ne 302 ]; then
    echo "Failed to download version ${version}!  Does it exist?"
    return 1
  fi

  # Download and untar
  echo "Installing dotslash version ${version} to /usr/local/bin..."
  curl --silent --location --output '-' "${url}" | tar -xz -f '-' -C /usr/local/bin dotslash
}

echo "Activating feature 'dotslash' with version ${VERSION}"

if [ -z "${VERSION}" ]; then
  echo "No version specified!"
  return 1
fi

ensure_dependencies

# Remove any double quotes that might be in the version string.
VERSION="${VERSION//\"/}"

download "${VERSION}"
