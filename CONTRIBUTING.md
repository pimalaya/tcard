# Contributing guide

Thank you for investing your time in contributing to tcard.

Whether you are a human or an AI agent, read these in order before touching the code:

1. the [Pimalaya README](https://github.com/pimalaya) for what the project is and how its repositories stack;
2. the [Pimalaya ARCHITECTURE](https://github.com/pimalaya/.github/blob/master/ARCHITECTURE.md) for the conventions every repository shares (layering, `no_std`, modules, errors, code style, licensing, notes for AI agents);
3. this guide, for how to build, test and submit changes here;
4. the repo [ARCHITECTURE](./ARCHITECTURE.md) for how tcard in particular is designed.

This document stays operational; the design lives in [ARCHITECTURE.md](./ARCHITECTURE.md).

## Development environment

The environment is managed by [Nix](https://nixos.org/download.html). `nix develop` spawns a shell with the right toolchain; every cargo command below assumes it (or prefix them with `nix develop --command`).

Without Nix, install a recent stable toolchain via [rustup](https://rust-lang.github.io/rustup/) (`rustup update`); the crate needs Rust matching the `rust-version` in [Cargo.toml](./Cargo.toml).

## Build

tcard is a `#![no_std]` library with an optional CLI behind a single `cli` feature (not enabled by default):

```sh
cargo build                      # no_std core library
cargo build --features cli       # library + binary (pulls in std)
cargo build --release --features cli
```

When touching feature gates or imports, check both the core and the CLI build, so no `std`-only code leaks into the `no_std` core.

## Lint, test, audit

```sh
cargo test                       # unit + integration + doc tests
cargo test --features cli        # also exercises the CLI-only code paths
cargo clippy --all-targets       # keep clean for core and --features cli
cargo fmt                        # CI checks `cargo fmt --check`
```

Before opening a PR, make sure `cargo test`, `cargo clippy` and `cargo fmt --check` pass.

### Adding a fixture

`tests/data/` is a golden database of vCards (see [ARCHITECTURE.md](./ARCHITECTURE.md#the-golden-fixture-database)); adding a real-world card is the fastest way to turn a bug report into a regression test:

1. drop the card in as `tests/data/<name>.vcf`;
2. generate the expectation: `cargo run --features cli -- template tests/data/<name>.vcf -o tests/data/<name>.all.toml`;
3. eyeball the generated `.toml`; if anything looks wrong, you have found a bug, fix the code rather than the fixture;
4. if the source will not round-trip byte-for-byte (see the limitations in ARCHITECTURE), add an empty `tests/data/<name>.lossy` marker;
5. run `cargo test`.

## Commit style

tcard follows the [conventional commits specification](https://www.conventionalcommits.org/en/v1.0.0/#summary). Keep the subject imperative and scoped; describe the *why* in the body when it is not obvious.
