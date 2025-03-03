# 🧙 Acolyte

> _[Would you like to know the secret to eternal happiness?](https://youtu.be/M_FAL8nVT40?t=25)_

Acolyte is a lightweight resource monitor for Kubernetes containers.

The planned flow is:

1. start a container
2. `exec` Acolyte to the container, and make sure it keeps on running after `exec` termination
3. continuously record stats on a shared volume, another worker reads them from there
4. Acolyte dies with the container

## Development

```bash
cargo test

RUST_LOG=debug cargo run
```

# Build

For release, you should build it as a [`musl`](https://en.wikipedia.org/wiki/Musl) static binary:

```bash
rustup target add x86_64-unknown-linux-musl

# https://rust-lang.github.io/rfcs/1721-crt-static.html#specifying-dynamicstatic-c-runtime-linkage
# the flag is enabled by default for this specific target but 🤷
RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target x86_64-unknown-linux-musl

RUST_LOG=debug target/x86_64-unknown-linux-musl/release/acolyte
```
