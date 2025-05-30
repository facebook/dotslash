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

echo "#!/usr/bin/env dotslash

{
  \"name\": \"buck2\",
  \"platforms\": {
    \"linux-aarch64\": {
      \"size\": 30289600,
      \"hash\": \"blake3\",
      \"digest\": \"bbb4d04da8deca8a197bffd9cf60b6057e4765a32d01dd28d495f5571dbdc96b\",
      \"format\": \"zst\",
      \"path\": \"buck2-aarch64-unknown-linux-musl\",
      \"providers\": [
        {
          \"url\": \"https://github.com/facebook/buck2/releases/download/2025-05-06/buck2-aarch64-unknown-linux-musl.zst\"
        }
      ]
    },
    \"linux-x86_64\": {
      \"size\": 31572599,
      \"hash\": \"blake3\",
      \"digest\": \"1499fa841ba87adb5cceaf3b4680db1db79967a14470bd40a344788d03e75082\",
      \"format\": \"zst\",
      \"path\": \"buck2-x86_64-unknown-linux-musl\",
      \"providers\": [
        {
          \"url\": \"https://github.com/facebook/buck2/releases/download/2025-05-06/buck2-x86_64-unknown-linux-musl.zst\"
        }
      ]
    }
  }
}" > buck2
chmod +x buck2

touch .buckconfig

check "ensure buck2 is runnable" ./buck2 --help

reportResults
