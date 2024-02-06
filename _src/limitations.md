---
sidebar_position: 24
---

# Limitations

As DotSlash aspires to do one thing well, there are inevitably many things it
does poorly. Because DotSlash is unlikely to expand beyond its original charter,
its limitations are equally unlikely to change. To that end, it is imporant to
understand the tradeoffs it makes when deciding whether DotSlash is the right
tool for the job.

## DotSlash is not a package manager

Package managers such as [RPM](https://rpm-packaging-guide.github.io/) and
[APT](<https://en.wikipedia.org/wiki/APT_(software)>) are tools in the "software
distribution" space that have a number of features that DotSlash lacks, which
may be critical to your use case. Examples include:

- A concept of "dependent packages." For example, an RPM for
  [ESLint](https://eslint.org/) could declare a dependency on the RPM for
  Node.js so that the ESLint RPM could be written assuming that `/usr/bin/node`
  is on the user's `$PATH`. By comparison, if ESLint were packaged using
  DotSlash and it used a DotSlash wrapper to invoke Node.js, then things would
  be a bit different:
  - The true `node` binary would not be downloaded until ESLint was run and
    exercised its internal DotSlash file for Node.js.
  - It would not have the side-effect of adding `node` to the user's `$PATH`.
  - Removing the ESLint DotSlash file (or even the artifact corresponding to it)
    would not provide any information to suggest that the Node.js artifact
    downloaded by DotSlash could be removed, as well.
- The ability to put files in a specific place on disk. For example, an RPM may
  write to folders such as `/etc/bash_completion.d` or `/usr/share/man` as part
  of installing a package so that the user automatically gets support for the
  corresponding Bash completions and `man` pages. A tool delivered via DotSlash
  does not have such affordances, though adding a subcommand to your CLI akin to
  [`rustup completions`](https://rust-lang.github.io/rustup/installation/index.html#enable-tab-completion-for-bash-fish-zsh-or-powershell)
  is a good workaround for the former.

## DotSlash is not a general file distribution mechanism

DotSlash is designed exclusively for fetching and running executable files. One
of our fundamental design principles has been to keep DotSlash small and to
provide minimal overhead when launching an executable. Extending it to support
general file distribution feels at odds with that.

If you are determined to use DotSlash in this way, you can always create an
executable that writes a specific data payload to a specified output folder!

## DotSlash will fail when URLs go stale

TODO(mbolin): Explain the risks you take on with respect to artifacts
expiring/no longer being available. @asuarez expressed interest in supporting
multiple URLs, so maybe wait until that is implemented before writing up this
section.

## Debug Symbols

We encourage executables to be deployed with `strip` to reduce size, but that is
undesirable for a certain class of users.

## Potential Version Skew Between Code Changes and DotSlash Changes

If you do monorepo-based development, you may have things set up such that you
prefer to build everything from source, every time. For example, at Meta, we are
heavy users of [Thrift](https://github.com/facebook/fbthrift), and the code for
the Thrift compiler lives in our repo. This means that projects that are
everyday users of Thrift have to spend some of their build cycles building the
Thrift compiler from source before they can build their own binary. (At Meta, we
leverage distributed builds and caching to mitigate the cost of having to build
common infrastructure such as Thrift.)

While this may be undesriable to the average Thrift service developer at Meta,
it is invaluable to the Thrift team and their ability to move the toolchain
forward. That is, it makes it straightforward to make a local change to the
Thrift compiler and see what effect it has by rebuilding any project of interest
in the monorepo that depends on Thrift. In the event that a service needs to be
updated as a result of a Thrift compiler change, both changes can be done as
part of the same commit such that the overall change can be landed atomically.

That said, waiting to build the Thrift compiler can be a drag, so it is tempting
to vendor the Thrift compiler (ideally one built with full Clang optimizations!)
in the monorepo as a DotSlash file, but this introduces a potential version skew
issue. That is, the repo also contains Thrift library code that is developed in
tandem with the Thrift compiler. So long as both the compiler and library are
built from source, no consistency issues arise.

By comparison, consider a scenario where the DotSlash file wrapping the Thrift
compiler were rebuilt once a week. In that world, a Thrift developer (who is
presumably building the entire Thrift toolchain from source) could easily find
themselves in a position where they need to update both the library and the
compiler together. This is problematic because landing such a commit could put
the library and the vendored Thrift compiler in an incompatible state until the
DotSlash file were rebuilt with the latest compiler changes.

While there are strategies that can be employed to circumvent this issue (such
as having two copies of the code, e.g., `dev/` and `release/` folders where a
"release" entails regenerating the DotSlash file based on the contents of `dev/`
and then copying everything over to `release/`), scenarios similar to this
Thrift example are arguably not a good fit for DotSlash.
