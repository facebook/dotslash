# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is dual-licensed under either the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree or the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree. You may select, at your option, one of the
# above-listed licenses.
if __name__ == "__main__":
    import sys

    from dotslash import locate

    dotslash = locate()

    if sys.platform == "win32":
        import subprocess

        process = subprocess.run([dotslash, *sys.argv[1:]])
        sys.exit(process.returncode)
    else:
        import os

        os.execvp(dotslash, [dotslash, *sys.argv[1:]])
