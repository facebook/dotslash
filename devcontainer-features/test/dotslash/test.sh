#!/usr/bin/env bash
# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under both the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree and the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree.

set -e

# shellcheck source=/dev/null
source dev-container-features-test-lib

check "ensure dotslash is installed and works" dotslash --version

reportResults
