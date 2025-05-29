#!/usr/bin/env bash

set -e

source dev-container-features-test-lib

check "ensure dotslash is installed and works" dotslash --version

reportResults
