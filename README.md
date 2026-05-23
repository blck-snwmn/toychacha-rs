# toychacha-rs
[![test suite](https://github.com/blck-snwmn/toychacha-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/blck-snwmn/toychacha-rs/actions/workflows/ci.yml)

Toy implementation of chacha20 poly1305 written in Rust.

## Test
```bash
cargo test
```

## Development

CLI tools (`lefthook`) are managed by [aqua](https://aquaproj.github.io/) with versions pinned in [aqua.yaml](aqua.yaml).

### Install tools

Install aqua itself first (see the [aqua installation guide](https://aquaproj.github.io/docs/install)), then install the pinned tools:

```bash
aqua install
```

### Set up git hooks

[lefthook](lefthook.yml) runs format, clippy, and test checks on staged `*.rs` files before each commit. Register the hooks once after cloning:

```bash
lefthook install
```
