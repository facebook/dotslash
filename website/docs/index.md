---
sidebar_position: 0
---

# Introduction

DotSlash (`dotslash`) is a command-line tool that is designed to facilitate
fetching an executable, verifying it, and then running it. It maintains a local
cache of fetched executables so that subsequent invocations are fast.

DotSlash helps keeps heavyweight binaries out of your repo while ensuring your
developers seamlessly get the tools they need, ensuring consistent builds across
platforms. At Meta, DotSlash is executed _hundreds of millions of times per day_
to deliver a mix of first-party and third-party tools to end-user developers as
well as hermetic build environments.

While the page on [Motivation](./motivation) details the benefits of DotSlash
and the thinking behind its design, here we will try to illustrate the value
with a concrete example:

## Example: Vendoring Node.js in a Repo (Traditional)

Suppose you have a project that depends on Node.js. To ensure that everyone on
your team uses the same version of Node.js, traditionally, you would add the
following files to your repo and ask contributors to reference `scripts/node`
from your repo instead of assuming `node` is on the `$PATH`:

- `scripts/node-mac-v18.19.0` the universal macOS binary for Node.js
- `scripts/node-linux-v18.19.0` the x86_64 Linux binary for Node.js
- `scripts/node` a shell script with the following contents:

```bash
#!/bin/bash

# Copied from https://stackoverflow.com/a/246128.
DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

if [ "$(uname)" == "Darwin" ]; then
  # In this example, assume node-mac-v18.19.0 is a universal macOS binary.
  "$DIR/node-mac-v18.19.0" "$@"
else
  "$DIR/node-linux-v18.19.0" "$@"
fi

exit $?
```

Downsides with this approach:

- Binary files are checked into the repo, making `git clone` more expensive.
- Further, every user has to pay the cost of downloading an executable they are
  guaranteed not to use because it is for a different operating system.
- Three files are being used to represent one executable, making it all too easy
  to update one of the files but not the others.
- The Bash script has to execute additional processes (for `dirname`, `uname`,
  and `[`) before it ultimately runs Node.js.

## Example: Vendoring Node.js in a Repo (with DotSlash!)

To solve the above problem with DotSlash, do the following:

- Compress each platform-specific executable (or `.tar` file containing the
  executable) with `gzip` or [`zstd`](https://facebook.github.io/zstd/) and
  record the resulting size, in bytes, as well as the
  [BLAKE3](<https://en.wikipedia.org/wiki/BLAKE_(hash_function)#BLAKE3>) or
  [SHA-256](https://en.wikipedia.org/wiki/SHA-2) hash.
- Upload each artifact to a URL accessible to your target audience. For example,
  an internal-only executable might be served from a URL that is restricted to
  users on a VPN.
- Rewrite the shell script at `scripts/node` with this information, structured
  as follows:

```bash
#!/usr/bin/env dotslash

// The URLs in this file were taken from https://nodejs.org/dist/v18.19.0/

{
  "name": "node-v18.19.0",
  "platforms": {
    "macos-aarch64": {
      "size": 40660307,
      "hash": "blake3",
      "digest": "6e2ca33951e586e7670016dd9e503d028454bf9249d5ff556347c3d98c347c34",
      "format": "tar.gz",
      "path": "node-v18.19.0-darwin-arm64/bin/node",
      "providers": [
        {
          "url": "https://nodejs.org/dist/v18.19.0/node-v18.19.0-darwin-arm64.tar.gz"
        }
      ]
    },
    "macos-x86_64": {
      "size": 42202872,
      "hash": "blake3",
      "digest": "37521058114e7f71e0de3fe8042c8fa7908305e9115488c6c29b514f9cd2a24c",
      "format": "tar.gz",
      "path": "node-v18.19.0-darwin-x64/bin/node",
      "providers": [
        {
          "url": "https://nodejs.org/dist/v18.19.0/node-v18.19.0-darwin-x64.tar.gz"
        }
      ]
    },
    "linux-x86_64": {
      "size": 44694523,
      "hash": "blake3",
      "digest": "72b81fc3a30b7bedc1a09a3fafc4478a1b02e5ebf0ad04ea15d23b3e9dc89212",
      "format": "tar.gz",
      "path": "node-v18.19.0-linux-x64/bin/node",
      "providers": [
        {
          "url": "https://nodejs.org/dist/v18.19.0/node-v18.19.0-linux-x64.tar.gz"
        }
      ]
    }
  }
}
```

Note that in the above example, we leverage DotSlash to distribute
_architecture_-specific executables for macOS so users can download smaller,
more specific binaries. If the archive contained a universal macOS binary, there
would still be individual entries for both `"macos-x86_64"` and
`"macos-aarch64"` in the DotSlash file, but the values would be the same.

Assuming `dotslash` is on your `$PATH` and you remembered to
`chmod +x ./scripts/node` to mark it as executable, you should be able to run
your Node.js wrapper exactly as you did before:

```shell
$ ./scripts/node --version
v18.19.0
```

The first time you run `./scripts/node --version`, you will likely experience a
small delay while DotSlash fetches, decompresses, and verifies the appropriate
`.zst`, but subsequent invocations should be instantaneous.

To understand what is happening under the hood, see
[How DotSlash Works](./execution).
