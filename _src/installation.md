---
sidebar_position: 8
---

# Installing DotSlash

Until `dotslash` is available in well-known package managers, the most
straightforward way to install it is via `cargo install`:

```shell
cargo install dotslash
```

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
`` Failed to find tool. Is `musl-gcc` installed? ``, then you likely need to
install the `musl-gcc` package, which `rustup` does not do for you.

On Ubuntu/Debian, you can install it with:

```shell
sudo apt install musl-tools
```
