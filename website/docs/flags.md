---
sidebar_position: 14
---

# Command Line Flags

Because the usage of DotSlash is:

```shell
dotslash DOTSLASH_FILE [OPTIONS]
```

where `[OPTIONS]` is forwarded to the executable represented by `DOTSLASH_FILE`,
DotSlash's own command line flags must be able to be disambiguated from
`DOTSLASH_FILE`. In practice, that means any flag recognized by DotSlash is an
unsupported DotSlash file name. For this reason, the set of supported flags is
fairly limited.

## Supported Flags

<!-- markdownlint-disable MD033 -->

| flag                     | description                                                                                                                                           |
| ------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--help`                 | prints basic usage info, as well as the _platform_ it was compiled for (which is the entry it will use from the `"platforms"` map in a DotSlash file) |
| <nobr>`--version`</nobr> | prints the DotSlash version number and exits                                                                                                          |

<!-- markdownlint-enable MD033 -->

## Experimental Commands

Experimental commands are special flags that we are not committed to supporting,
and whose output format should be considered unstable. These commands are
"hidden" behind `--` (using `--` as the first argument to `dotslash` tells it to
use a special argument parser) and are used like so:

```shell
$ dotslash -- cache-dir
/Users/mbolin/Library/Caches/dotslash
```

| command                | description                                                                          |
| ---------------------- | ------------------------------------------------------------------------------------ |
| `b3sum FILE`           | prints the BLAKE3 hash of `FILE`                                                     |
| `cache-dir`            | prints the absolute path to the user's DotSlash cache and exits                      |
| `create-url-entry URL` | generates the DotSlash JSON snippet for the artifact at the URL                      |
| `fetch DOTSLASH_FILE`  | fetches the artifact identified by `DOTSLASH_FILE` if it is not already in the cache |
| `parse DOTSLASH_FILE`  | parses `DOTSLASH_FILE` and prints the data as pure JSON to stdout                    |
| `sha256 FILE`          | prints the SHA-256 hash of `FILE`                                                    |

## Environment Variables

The `DOTSLASH_CACHE` environment variable can be used to override the default
location of the DotSlash cache. By default, the DotSlash cache resides at:

| platform | path                                                 |
| -------- | ---------------------------------------------------- |
| Linux    | `$XDG_CACHE_HOME/dotslash` or `$HOME/.cache/dotslash |
| macOS    | `$HOME/Library/Caches/dotslash`                      |
| Windows  | `{FOLDERID_LocalAppData}/dotslash`                   |

DotSlash relies on
[`dirs::cache_dir()`](https://docs.rs/dirs/5.0.1/dirs/fn.cache_dir.html) to use
the appropriate default directory on each platform.
