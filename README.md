# tCard [![Documentation](https://img.shields.io/docsrs/tcard?style=flat&logo=docs.rs&logoColor=white)](https://docs.rs/tcard/latest/tcard) [![Matrix](https://img.shields.io/badge/chat-%23pimalaya-blue?style=flat&logo=matrix&logoColor=white)](https://matrix.to/#/#pimalaya:matrix.org) [![Mastodon](https://img.shields.io/badge/news-%40pimalaya-blue?style=flat&logo=mastodon&logoColor=white)](https://fosstodon.org/@pimalaya)

CLI & lib to edit [vCards](https://www.rfc-editor.org/rfc/rfc6350) as ergonomic TOML.

```sh
$ tcard edit
```

```toml
full-name = "Jane Doe"
nickname = ["Janie"]
organization = ["Acme", "Engineering"]
title = "Engineer"
birthday = "19960415"

[[email]]
type = "work"
value = "jane@acme.example"

[[phone]]
type = "cell"
value = "+1-555-0100"
```

Output:

```vcf
BEGIN:VCARD
VERSION:4.0
UID:urn:uuid:1f34e439-ca07-446f-af28-f5b7d3afcfc8
FN:Jane Doe
NICKNAME:Janie
ORG:Acme;Engineering
TITLE:Engineer
BDAY:19960415
EMAIL;TYPE=work:jane@acme.example
TEL;TYPE=cell:+1-555-0100
END:VCARD
```

This repository ships two interfaces:

- Rust **library** to generate vCard from/to TOML projection
- **CLI** to print and/or edit TOML template using `$EDITOR`

## Table of contents

- [Features](#features)
- [Installation](#installation)
  - [Pre-built binary](#pre-built-binary)
  - [Cargo](#cargo)
  - [Nix](#nix)
  - [Sources](#sources)
- [Usage](#usage)
  - [Library](#library)
  - [CLI](#cli)
- [FAQ](#faq)
- [License](#license)
- [AI disclosure](#ai-disclosure)
- [Contributing](CONTRIBUTING.md)
- [Social](#social)
- [Sponsoring](#sponsoring)

## Features

- Partial `no_std` support
- vCard from/to TOML **projection**, backed by [calcard](https://crates.io/crates/calcard) (RFC 6350).
- **Friendly** keys and values: cryptic property names become readable TOML keys.
- **Structured** names and addresses: `N` and `ADR` expand into named components; typed properties (`email`, `tel`) list their accepted `TYPE` values.
- **Discoverable** properties: prints all available properties with empty values by default, fill the ones you need.
- **Minimal, lossless diffs**: `apply` patches the original text through a format-preserving editor, re-rendering only the lines you changed.

## Installation

### Pre-built binary

The CLI binary `tcard` can be installed from the latest [GitHub release](https://github.com/pimalaya/tcard/releases) using the install script:

*As root:*

```sh
curl -sSL https://raw.githubusercontent.com/pimalaya/tcard/master/install.sh | sudo sh
```

*As a regular user:*

```sh
curl -sSL https://raw.githubusercontent.com/pimalaya/tcard/master/install.sh | PREFIX=~/.local sh
```

For a more up-to-date version, check out the [pre-releases](https://github.com/pimalaya/tcard/actions/workflows/pre-releases.yml) GitHub workflow: pick the latest run and grab the artifact matching your OS. These are built from the `master` branch.

> [!NOTE]
> Pre-built binaries are built with the default cargo features. If you need a different feature set, use another installation method.

### Cargo

```sh
cargo install tcard --locked --features cli
```

You can also use the git repository for a more up-to-date (but less stable) version:

```sh
cargo install --locked --git https://github.com/pimalaya/tcard.git
```

To use `tcard` as a library, add it to your `Cargo.toml`:

```toml
[dependencies]
tcard = "0.0.1"
```

The library has no default features: it is a slim `no_std` (plus `alloc`) build with no clap, no editor integration, just the `project` / `apply` projection over a calcard `VCard`. The CLI lives behind the opt-in `cli` feature (enabled above with `cargo install --features cli`).

### Nix

If you have the [Flakes](https://nixos.wiki/wiki/Flakes) feature enabled:

```sh
nix profile install github:pimalaya/tcard
```

Or run without installing:

```sh
nix run github:pimalaya/tcard -- template < contact.vcf
```

### Sources

```sh
git clone https://github.com/pimalaya/tcard
cd tcard
nix run
```

## Usage

### Library

Project a vCard file to TOML, then fold edits back:

```rust
use tcard::{template, vcard};

let input = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Ada Lovelace\r\nEND:VCARD\r\n";

// A file may hold several cards; project them all.
let cards = vcard::parse_all(input).unwrap();
let version = cards[0].version().unwrap();

// Emit the prefilled scaffold: a single card flattens at the root, two or
// more become [[card]] blocks.
let scaffold = template::project(&cards, version);
assert!(scaffold.contains("full-name = \"Ada Lovelace\""));

// After the user edits the scaffold, fold it back onto the original text:
// only changed lines are re-rendered, everything else stays byte-for-byte.
let edited = scaffold.replace("Ada Lovelace", "Ada King");
let updated = template::apply(input, &edited).unwrap();
assert!(updated.contains("FN:Ada King"));
```

### CLI

Print a blank, fully-documented template:

```sh
tcard template
```

Project an existing vCard to TOML (path, stdin via `-`, or literal contents):

```sh
tcard template contact.vcf
tcard template - < contact.vcf
```

Edit a vCard in `$EDITOR`. With a file source, the result is written back in place; otherwise it goes to stdout (or `--output`):

```sh
tcard edit contact.vcf
tcard edit - < contact.vcf > updated.vcf
tcard template | $EDITOR /dev/stdin   # inspect the scaffold first
```

Start a new card from scratch and write it out:

```sh
tcard edit --output alice.vcf
tcard edit --version 3.0 --output bob.vcf
```

## FAQ

### How does `tcard edit` pick the editor?

The [edit](https://crates.io/crates/edit) crate resolves `$VISUAL` first, then `$EDITOR`, then an OS default. tcard does not expose a config override: set `VISUAL` / `EDITOR` in your shell rc file.

### Will tcard reformat my whole card on edit?

No. `apply` patches the original text through a format-preserving editor (the vCard analog of toml_edit): only the lines of modeled fields you actually changed are re-rendered, so the diff is minimal. Folding, parameter casing (`TYPE=work` stays `TYPE=work`), property order and line endings of every untouched line are kept byte-for-byte.

### What happens to properties tcard does not list?

They are kept verbatim. The scaffold only surfaces the modeled vocabulary, but `apply` carries every other property (custom `X-*`, Apple `item1.*` groups, vendor extensions) straight from the original card into the result.

### How do I debug the CLI?

Use `--log <level>` where `<level>` is one of `off`, `error`, `warn`, `info`, `debug`, `trace`:

```sh
tcard --log trace template contact.vcf
```

The `RUST_LOG` environment variable, when set, overrides `--log` and supports per-target filters (see the [env_logger](https://docs.rs/env_logger/latest/env_logger/#enabling-logging) documentation). `RUST_BACKTRACE=1` enables full error backtraces. Logs are written to `stderr`.

## License

This project is licensed under either of:

- [MIT license](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.

## AI disclosure

This project is developed with AI assistance. This section documents how, so users and downstream packagers can make informed decisions.

- **Tools**: Claude Code (Anthropic), Opus 4.8, invoked locally with a persistent project-scoped memory and a small set of repo-specific rules.
- **Used for**: Refactors, mechanical multi-file edits, boilerplate (feature gates, error enums, derive macros, trait impls), test scaffolding, doc polish, exploratory design conversations.
- **Not used for**: Engineering, critical code, git manipulation (commit, merge, rebase…), real-world tests.
- **Verification**: Every AI-assisted change is read, compiled, tested, and formatted before commit (`nix develop --command cargo check / cargo test / cargo fmt`). Behavioural correctness is verified against the relevant RFC or upstream spec, not assumed from the model output. Tests are never adjusted to fit AI-generated code; the code is adjusted to fit correct behaviour.
- **Limitations**: AI models occasionally produce code that compiles and passes tests but is subtly wrong: off-by-one errors, missed edge cases, plausible but nonexistent APIs, stale RFC references. The verification workflow catches most of this; it does not catch all of it. Bug reports are welcome and taken seriously.
- **Last reviewed**: 14/06/2026

## Social

- Chat on [Matrix](https://matrix.to/#/#pimalaya:matrix.org)
- News on [Mastodon](https://fosstodon.org/@pimalaya) or [RSS](https://fosstodon.org/@pimalaya.rss)
- Mail at [pimalaya.org@posteo.net](mailto:pimalaya.org@posteo.net)

## Sponsoring

[![nlnet](https://nlnet.nl/logo/banner-160x60.png)](https://nlnet.nl/)

Special thanks to the [NLnet foundation](https://nlnet.nl/) and the [European Commission](https://www.ngi.eu/) that have been financially supporting the project for years:

- 2022 → 2023: [NGI Assure](https://nlnet.nl/project/Himalaya/)
- 2023 → 2024: [NGI Zero Entrust](https://nlnet.nl/project/Pimalaya/)
- 2024 → 2026: [NGI Zero Core](https://nlnet.nl/project/Pimalaya-PIM/)
- *2027 in preparation…*

If you appreciate the project, feel free to donate using one of the following providers:

[![GitHub](https://img.shields.io/badge/-GitHub%20Sponsors-fafbfc?logo=GitHub%20Sponsors)](https://github.com/sponsors/soywod)
[![Ko-fi](https://img.shields.io/badge/-Ko--fi-ff5e5a?logo=Ko-fi&logoColor=ffffff)](https://ko-fi.com/soywod)
[![Buy Me a Coffee](https://img.shields.io/badge/-Buy%20Me%20a%20Coffee-ffdd00?logo=Buy%20Me%20A%20Coffee&logoColor=000000)](https://www.buymeacoffee.com/soywod)
[![Liberapay](https://img.shields.io/badge/-Liberapay-f6c915?logo=Liberapay&logoColor=222222)](https://liberapay.com/soywod)
[![thanks.dev](https://img.shields.io/badge/-thanks.dev-000000?logo=data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQuMDk3IiBoZWlnaHQ9IjE3LjU5NyIgY2xhc3M9InctMzYgbWwtMiBsZzpteC0wIHByaW50Om14LTAgcHJpbnQ6aW52ZXJ0IiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwYXRoIGQ9Ik05Ljc4MyAxNy41OTdINy4zOThjLTEuMTY4IDAtMi4wOTItLjI5Ny0yLjc3My0uODktLjY4LS41OTMtMS4wMi0xLjQ2Mi0xLjAyLTIuNjA2di0xLjM0NmMwLTEuMDE4LS4yMjctMS43NS0uNjc4LTIuMTk1LS40NTItLjQ0Ni0xLjIzMi0uNjY5LTIuMzQtLjY2OUgwVjcuNzA1aC41ODdjMS4xMDggMCAxLjg4OC0uMjIyIDIuMzQtLjY2OC40NTEtLjQ0Ni42NzctMS4xNzcuNjc3LTIuMTk1VjMuNDk2YzAtMS4xNDQuMzQtMi4wMTMgMS4wMjEtMi42MDZDNS4zMDUuMjk3IDYuMjMgMCA3LjM5OCAwaDIuMzg1djEuOTg3aC0uOTg1Yy0uMzYxIDAtLjY4OC4wMjctLjk4LjA4MmExLjcxOSAxLjcxOSAwIDAgMC0uNzM2LjMwN2MtLjIwNS4xNTYtLjM1OC4zODQtLjQ2LjY4Mi0uMTAzLjI5OC0uMTU0LjY4Mi0uMTU0IDEuMTUxVjUuMjNjMCAuODY3LS4yNDkgMS41ODYtLjc0NSAyLjE1NS0uNDk3LjU2OS0xLjE1OCAxLjAwNC0xLjk4MyAxLjMwNXYuMjE3Yy44MjUuMyAxLjQ4Ni43MzYgMS45ODMgMS4zMDUuNDk2LjU3Ljc0NSAxLjI4Ny43NDUgMi4xNTR2MS4wMjFjMCAuNDcuMDUxLjg1NC4xNTMgMS4xNTIuMTAzLjI5OC4yNTYuNTI1LjQ2MS42ODIuMTkzLjE1Ny40MzcuMjYuNzMyLjMxMi4yOTUuMDUuNjIzLjA3Ni45ODQuMDc2aC45ODVabTE0LjMxNC03LjcwNmgtLjU4OGMtMS4xMDggMC0xLjg4OC4yMjMtMi4zNC42NjktLjQ1LjQ0NS0uNjc3IDEuMTc3LS42NzcgMi4xOTVWMTQuMWMwIDEuMTQ0LS4zNCAyLjAxMy0xLjAyIDIuNjA2LS42OC41OTMtMS42MDUuODktMi43NzQuODloLTIuMzg0di0xLjk4OGguOTg0Yy4zNjIgMCAuNjg4LS4wMjcuOTgtLjA4LjI5Mi0uMDU1LjUzOC0uMTU3LjczNy0uMzA4LjIwNC0uMTU3LjM1OC0uMzg0LjQ2LS42ODIuMTAzLS4yOTguMTU0LS42ODIuMTU0LTEuMTUydi0xLjAyYzAtLjg2OC4yNDgtMS41ODYuNzQ1LTIuMTU1LjQ5Ny0uNTcgMS4xNTgtMS4wMDQgMS45ODMtMS4zMDV2LS4yMTdjLS44MjUtLjMwMS0xLjQ4Ni0uNzM2LTEuOTgzLTEuMzA1LS40OTctLjU3LS43NDUtMS4yODgtLjc0NS0yLjE1NXYtMS4wMmMwLS40Ny0uMDUxLS44NTQtLjE1NC0xLjE1Mi0uMTAyLS4yOTgtLjI1Ni0uNTI2LS40Ni0uNjgyYTEuNzE5IDEuNzE5IDAgMCAwLS43MzctLjMwNyA1LjM5NSA1LjM5NSAwIDAgMC0uOTgtLjA4MmgtLjk4NFYwaDIuMzg0YzEuMTY5IDAgMi4wOTMuMjk3IDIuNzc0Ljg5LjY4LjU5MyAxLjAyIDEuNDYyIDEuMDIgMi42MDZ2MS4zNDZjMCAxLjAxOC4yMjYgMS43NS42NzggMi4xOTUuNDUxLjQ0NiAxLjIzMS42NjggMi4zNC42NjhoLjU4N3oiIGZpbGw9IiNmZmYiLz48L3N2Zz4=)](https://thanks.dev/soywod)
[![PayPal](https://img.shields.io/badge/-PayPal-0079c1?logo=PayPal&logoColor=ffffff)](https://www.paypal.com/paypalme/soywod)
