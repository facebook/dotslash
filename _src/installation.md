---
sidebar_position: 8
---

<FbInternalOnly>

:::info

If you're a Meta Employee trying to use DotSlash on your Meta laptop, don't
follow this doc.

See the
[internal Wiki page](https://www.internalfb.com/intern/wiki/?fbid=1082156404104844)
instead.

:::

</FbInternalOnly>

# Installing DotSlash

We provide a number of ways to install DotSlash.

:::note

On macOS, we **strongly** recommend running DotSlash as a
[Universal Binary](https://developer.apple.com/documentation/apple-silicon/building-a-universal-macos-binary)
rather than an x86 or ARM64 binary. If an x86 binary is running under
[Rosetta](https://developer.apple.com/documentation/apple-silicon/about-the-rosetta-translation-environment)
on Apple Silicon and ends up spawning `dotslash`, then for consistency with the
parent process, this will ensure that the `macos-x86_64` artifact will be run.

:::

## Prebuilt Binaries

We provide prebuilt binaries for macOS, [Ubuntu] Linux, and Windows on GitHub:

<https://github.com/facebook/dotslash/releases/latest>

Once you have downloaded DotSlash, you must ensure that `dotslash` (or
`dotslash.exe` on Windows) is on your `PATH`. You can test that it is setup
correctly on Mac or Linux by running:

```shell
/usr/bin/env dotslash --help
```

### Linux

```shell
curl -LSfs "https://github.com/facebook/dotslash/releases/latest/download/dotslash-ubuntu-22.04.$(uname -m).tar.gz" | tar fxz - -C YOUR_BIN_PATH
```

### macOS

For the reasons explained above, the macOS release is a Universal Binary.

```shell
curl -LSfs https://github.com/facebook/dotslash/releases/latest/download/dotslash-macos.tar.gz | tar fxz - -C YOUR_BIN_PATH
```

:::warning

Downloading the `.tar.gz` using a web browser instead of something like `curl`
will cause macOS to tag DotSlash as "untrusted" and the security manager will
prevent you from running it. You can remove this annotation as follows:

```shell
xattr -r -d com.apple.quarantine ~/Downloads/dotslash-macos.*.tar.gz
```

:::

### Windows

```cmd
cmd /c 'curl.exe -LSfs https://github.com/facebook/dotslash/releases/latest/download/dotslash-windows.tar.gz | tar fxz - -C YOUR_BIN_PATH'
```

## Developer Container

We provide a feature that you can add to your `devcontainer.json` that will
automatically install the latest version of DotSlash to `/usr/local/bin`.

```json
"features": {
  "ghcr.io/facebook/devcontainers/features/dotslash:latest": {}
}
```

## GitHub Actions

We provide a GitHub Action to install dotslash for a workflow:

<https://github.com/facebook/install-dotslash>

It can be used from GitHub Actions workflows like so:

```
name: test suite
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: facebook/install-dotslash@latest
      - run: ./some_dotslash_file
```

## cargo install

If you are familiar with the Rust toolchain, you can also build and install
DotSlash using `cargo`:

```shell
cargo install dotslash
```

Assuming you already have `.cargo/bin` on your `PATH`, you should not have to
update any environment variables to get DotSlash to work.

Though note that `cargo install` does not create a universal binary, so you may
be better off [building from source](#build-from-source).

## Install from npm

To use DotSlash in Node.js projects, you can install it as a dependency:

```shell
npm install --save fb-dotslash
```

This will install a suitable `dotslash` binary in `node_modules/.bin` and ensure
it's on the `PATH` when executing any `package.json` scripts, as well as during
`npm exec`, etc.

:::note

You can also use `npx fb-dotslash ./some_dotslash_file` to run a DotSlash file
from the command line without installing anything.

:::

For more advanced use cases, import the `fb-dotslash` package directly in your
code:

```js
const dotslash = require('fb-dotslash');
const {spawnSync} = require('child_process');
spawnSync(dotslash, ['./some_dotslash_file'], {stdio: 'inherit']);
```

## Build from source

The short version of the build process is:

```shell
git clone https://github.com/facebook/dotslash
cd dotslash
cargo build --release
```

Or with [Sapling](https://sapling-scm.com/docs/introduction/):

```shell
sl clone https://github.com/facebook/dotslash
cd dotslash
cargo build --release
```

And then you can copy `./target/release/dotslash` (or `dotslash.exe` on Windows)
to your `PATH`.

### macOS

Building a Universal Binary on macOS entails some extra steps:

```shell
git clone https://github.com/facebook/dotslash
cd dotslash
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
lipo -create -output dotslash target/aarch64-apple-darwin/release/dotslash target/x86_64-apple-darwin/release/dotslash
```

And then adding the `dotslash` file created by `lipo` to your `PATH`.

### musl

On Linux, consider building DotSlash from source using
[musl](https://musl.libc.org/) for an even more lightweight version of the
`dotslash` executable:

```shell
$ git clone https://github.com/facebook/dotslash
$ cd dotslash
$ rustup target add x86_64-unknown-linux-musl
$ cargo build --release --target=x86_64-unknown-linux-musl
$ target/x86_64-unknown-linux-musl/release/dotslash --help
usage: dotslash DOTSLASH_FILE [OPTIONS]
...
```

Note that if `cargo build` fails with an error like
``Failed to find tool. Is `musl-gcc` installed?``, then you likely need to
install the `musl-gcc` package, which `rustup` does not do for you.

On Ubuntu/Debian, you can install it with:

```shell
sudo apt install musl-tools
```

## GitHub CLI

If you have the [GitHub CLI (`gh`)](https://cli.github.com/) installed, you can
use `gh release download` to fetch the `.tar.gz` for a release instead of `curl`
by running a command of the form:

```shell
gh release download --repo facebook/dotslash TAG --pattern PATTERN
```

where `TAG` is the name of the release (such as `v0.2.0`) and `PATTERN` is used
to match the platform that matches the artifact's name (such as `'*windows*'`)
like so:

```shell
gh release download --repo facebook/dotslash v0.2.0 --pattern '*windows*'
```
