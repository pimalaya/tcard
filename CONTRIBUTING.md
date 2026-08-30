# Contributing guide

Thank you for investing your time in contributing to tCard.

Whether you are a human or an AI agent, read these in order before touching the code:

1. the [Pimalaya README](https://github.com/pimalaya) for what the project is and how its repositories stack;
2. the [Pimalaya CONTRIBUTING](https://github.com/pimalaya/.github/blob/master/CONTRIBUTING.md) guide (Nix environment, build and check commands, dependency overrides, commit style), which chains to the shared architecture and guidelines;
3. the inline header documentation in [src/lib.rs](./src/lib.rs): it is the architecture document of this crate, covering the projection, the modelled vocabulary, the merge and the golden fixture database;
4. the [cairn](./cairn) folder for the living specification, the in-flight proposals and the landed history, activated by [AGENTS.md](./AGENTS.md).

Everything below documents only what differs from the Pimalaya standards.

## Feature matrix

tCard is a library first, so it ships no default features: the projection and the merge are a `no_std` core over `alloc`, and everything the CLI needs (clap, the editor, the filesystem, `std` itself) sits behind the opt-in `cli` feature.

The default build is therefore the narrow one, the opposite of the io- libraries, so a change touching a feature gate or an import is built both ways before it lands:

```sh
cargo build                   # the no_std core alone, no std leak
cargo build --features cli    # the library plus the binary above it
```

tCard depends on the released vcard-rs. To build against a local checkout, pass it on the command line rather than editing Cargo.toml:

```sh
cargo test --all-features --config 'patch.crates-io.vcard-rs.path="../vcard"'
```

## Adding a fixture

tests/data is a golden database of vCards, described in the src/lib.rs header. Adding a real-world export is the fastest way to turn a bug report into a regression test:

1. drop the card in as tests/data/NAME.vcf;
2. generate the expectation with `cargo run --features cli -- template tests/data/NAME.vcf -o tests/data/NAME.all.toml`;
3. read what came out: if anything looks wrong you have found a bug, so fix the code rather than the fixture;
4. add an empty tests/data/NAME.lossy marker when the source will not round-trip byte-for-byte, the known limitations in the src/lib.rs header saying when;
5. run `cargo test`.
