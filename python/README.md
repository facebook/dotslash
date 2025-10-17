# DotSlash: simplified executable deployment

[![CI - Test](https://github.com/facebook/dotslash/actions/workflows/test-python.yml/badge.svg)](https://github.com/facebook/dotslash/actions/workflows/test-python.yml)
[![PyPI - Version](https://img.shields.io/pypi/v/dotslash.svg?logo=pypi&label=PyPI&logoColor=gold)](https://pypi.org/project/dotslash/)
[![PyPI - Downloads](https://img.shields.io/pypi/dm/dotslash.svg?color=blue&label=Downloads&logo=pypi&logoColor=gold)](https://pypi.org/project/dotslash/)
[![Built by Hatch](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/pypa/hatch/master/docs/assets/badge/v0.json)](https://github.com/pypa/hatch)
[![Ruff linting/formatting](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/astral-sh/ruff/main/assets/badge/v2.json)](https://github.com/astral-sh/ruff)

-----

[DotSlash](https://dotslash-cli.com/docs/) (`dotslash`) is a command-line tool that lets you represent a set of platform-specific, heavyweight executables with an equivalent small, easy-to-read text file. In turn, this makes it efficient to store executables in source control without hurting repository size. This paves the way for checking build toolchains and other tools directly into the repo, reducing dependencies on the host environment and thereby facilitating reproducible builds.

The `dotslash` package allows you to use DotSlash in your Python projects without having to install DotSlash globally.

***Table of Contents***

- [Using as a library](#using-as-a-library)
- [Using as a command-line tool](#using-as-a-command-line-tool)
- [Building from source](#building-from-source)
- [License](#license)

## Using as a library

The `dotslash.locate` function returns the path to the DotSlash binary that was installed by this package.

```pycon
>>> import dotslash
>>> dotslash.locate()
'/root/.local/bin/dotslash'
```

## Using as a command-line tool

The installed DotSlash binary can be invoked directly by running the `dotslash` module as a script.

```
python -m dotslash path/to/dotslash-file.json
```

## Building from source

When building or installing from this directory, the `DOTSLASH_VERSION` environment variable must be set to the version of DotSlash to use. A preceding `v` is accepted but not required.

```
DOTSLASH_VERSION=0.5.8 python -m build
```

This will use the binaries from DotSlash's [GitHub releases](https://github.com/facebook/dotslash/releases). If there is a directory of GitHub release assets, you can use that directly with the `DOTSLASH_SOURCE` environment variable.

```
DOTSLASH_VERSION=0.5.8 DOTSLASH_SOURCE=path/to/dotslash-assets python -m build
```

The DotSlash source is set to `release` by default.

## License

DotSlash is licensed under both the MIT license and Apache-2.0 license; the exact terms can be found in the [LICENSE-MIT](https://github.com/facebook/dotslash/blob/main/LICENSE-MIT) and [LICENSE-APACHE](https://github.com/facebook/dotslash/blob/main/LICENSE-APACHE) files, respectively.
