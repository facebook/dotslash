<div class="title-block" style="text-align: center;" align="center">

# DotSlash: simplified executable deployment

![License] [![Build Status]][CI]

[License]:
  https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blueviolet.svg
[Build Status]:
  https://github.com/facebook/dotslash/actions/workflows/build.yml/badge.svg?branch=main
[CI]: https://github.com/facebook/dotslash/actions/workflows/build.yml

</div>

DotSlash (`dotslash`) is a command-line tool that lets you represent a set of
platform-specific, heavyweight executables with an equivalent small,
easy-to-read text file. In turn, this makes it efficient to store executables in
source control without hurting repository size. This paves the way for checking
build toolchains and other tools directly into the repo, reducing dependencies
on the host environment and thereby facilitating reproducible builds.

We will illustrate this with
[an example taken from the DotSlash website](https://dotslash-cli.com/docs/).
Traditionally, if you want to vendor a specific version of Node.js into your
project and you want to support both macOS and Linux, you likely need at least
two binaries (one for macOS and one for Linux) as well as a shell script like
this:

```shell
#!/bin/bash

# Copied from https://stackoverflow.com/a/246128.
DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

if [ "$(uname)" == "Darwin" ]; then
  # In this example, assume node-mac-v18.16.0 is a universal macOS binary.
  "$DIR/node-mac-v18.16.0" "$@"
else
  "$DIR/node-linux-v18.16.0" "$@"
fi

exit $?
```

With DotSlash, the shell script and the binaries can be replaced with a single
file named `node`:

```jsonc
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
    // Note that with DotSlash, it is straightforward to specify separate
    // binaries for different platforms, such as x86 vs. arm64 on macOS.
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

Assuming `dotslash` is on your `$PATH` and you remembered to `chmod +x node` to
mark it as executable, you can now run your Node.js wrapper exactly as you did
before:

```shell
$ ./node --version
v18.16.0
```

The first time you run `./node --version`, you will likely experience a small
delay while DotSlash fetches, decompresses, and verifies the appropriate
`.tar.gz`, but subsequent invocations should be instantaneous.

To understand what is happening under the hood, read the article on
[how DotSlash works](https://dotslash-cli.com/docs/execution/).

## Installing DotSlash

See the [installation instructions](https://dotslash-cli.com/docs/installation/)
on the DotSlash website.

## License

DotSlash is licensed under both the MIT license and Apache-2.0 license; the
exact terms can be found in the [LICENSE-MIT](LICENSE-MIT) and
[LICENSE-APACHE](LICENSE-APACHE) files, respectively.
