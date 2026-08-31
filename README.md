# tCard [![Documentation](https://img.shields.io/docsrs/tcard?style=flat&logo=docs.rs&logoColor=white)](https://docs.rs/tcard/latest/tcard) [![Matrix](https://img.shields.io/badge/chat-%23pimalaya-blue?style=flat&logo=matrix&logoColor=white)](https://matrix.to/#/#pimalaya:matrix.org) [![Mastodon](https://img.shields.io/badge/news-%40pimalaya-blue?style=flat&logo=mastodon&logoColor=white)](https://fosstodon.org/@pimalaya) [![Sponsor](https://img.shields.io/badge/sponsor-pink?style=flat&logo=github-sponsors&logoColor=white)](https://pimalaya.org/sponsor/)

Edit and merge [vCards](https://www.rfc-editor.org/rfc/rfc6350) as ergonomic TOML

```sh
tcard edit
```

```toml
full-name = "Jane Doe"
nickname = ["Janie"]
organization = ["Acme", "Engineering"]
title = "Engineer"
birthday = 1996-04-15

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

This repository ships two interfaces: a Rust library projecting a card to TOML and folding the edits back, and a CLI printing, editing and merging cards through `$EDITOR`, or folding an already edited form back with `apply`.

## Table of contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [AI policy](https://github.com/pimalaya/.github/blob/master/AI_POLICY.md)
- [License](#license)
- [Social](#social)
- [Contributing](./CONTRIBUTING.md)
- [Sponsoring](#sponsoring)

## Features

- **Ergonomic projection**: a card becomes a fillable TOML form, its cryptic property names becoming readable keys.
- **Structured** names and addresses: `N` and `ADR` expand into named components, and a typed property lists the `TYPE` values it accepts.
- **Discoverable** properties: the blank form lists every property tCard knows, with empty values, so filling one needs no reference.
- **Minimal, lossless diffs**: only the lines you changed are re-rendered, and every other line keeps the card's own bytes.
- **Verbatim passthrough**: a property tCard does not list, a parameter the form hides and a group prefix all survive an edit untouched.
- **Three-way merge**: `merge` reconciles two divergent cards against their base, writing what it cannot decide as duplicate TOML keys.
- **Interactive editing** in `$EDITOR`, or in the command `--editor` names, behind the opt-in `cli` cargo feature.

## Installation

### Pre-built binary

As root:

```sh
curl -sSL https://raw.githubusercontent.com/pimalaya/tcard/master/install.sh | sudo sh
```

As a regular user:

```sh
curl -sSL https://raw.githubusercontent.com/pimalaya/tcard/master/install.sh | PREFIX=~/.local sh
```

These commands install the latest binary from the GitHub [releases](https://github.com/pimalaya/tcard/releases) section.

For a more up-to-date version, check the [releases](https://github.com/pimalaya/tcard/actions/workflows/releases.yml) workflow and look for the *Artifacts* section: those are built from `master`, with the default cargo features.

### Cargo

The binary lives behind the `cli` feature, which is off by default so that a library consumer pays for none of it:

```sh
cargo install --locked --features cli tcard
```

The library alone is a `tcard` dependency, which pulls in none of that:

```sh
cargo add tcard
```

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

See documentation at [docs.rs](https://docs.rs/tcard/latest/tcard).

### CLI

Run `tcard --help` for the full command tree, and `tcard <command> --help` for a command's arguments and what it does with them.

A source is a path to a vCard file, `-` for stdin, or literal vCard contents; omitting it starts from a blank form. A few real command lines:

```sh
tcard template
tcard template contact.vcf
tcard template - < contact.vcf
tcard edit contact.vcf
tcard edit - < contact.vcf > updated.vcf
tcard edit --output alice.vcf
tcard edit --version 3.0 --output bob.vcf
tcard edit --editor "code --wait" contact.vcf
tcard apply form.toml contact.vcf           # fold an edited form back, no editor
tcard template contact.vcf | edit-somehow | tcard apply - contact.vcf
tcard merge base.vcf local.vcf remote.vcf --output merged.vcf
```

The editor is the one `--editor` names, then `$VISUAL`, then `$EDITOR`, and nothing after those: tCard picks none of its own, and says so when neither variable is set. It reads no configuration file, so set them in your shell. The command is spawned on the path of a temporary TOML file it edits in place, so it must block until the edit is done: use `code --wait`, not `code`.

Logs go to stderr, so they can be redirected to a file while the command output stays on stdout:

```sh
tcard template contact.vcf --log-level debug 2>/tmp/tcard.log
```

Use `--log-file <PATH>` to append them to a file directly. When `--log-level` is omitted the `RUST_LOG` environment variable is consulted, and `RUST_BACKTRACE=1` adds the full error backtrace.

## License

This project is licensed under either of:

- [MIT license](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

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
- 2026 → 2027: [NGI Zero Commons Fund](https://nlnet.nl/project/Pimalaya-pimdir/)

This program is part of Pimalaya, free software funded entirely by grants and donations. If you find it useful, consider [sponsoring](https://pimalaya.org/sponsor/) its development:

[![GitHub](https://img.shields.io/badge/-GitHub%20Sponsors-fafbfc?logo=GitHub%20Sponsors)](https://github.com/sponsors/soywod)
[![Ko-fi](https://img.shields.io/badge/-Ko--fi-ff5e5a?logo=Ko-fi&logoColor=ffffff)](https://ko-fi.com/pimalaya)
[![Buy Me a Coffee](https://img.shields.io/badge/-Buy%20Me%20a%20Coffee-ffdd00?logo=Buy%20Me%20A%20Coffee&logoColor=000000)](https://www.buymeacoffee.com/pimalaya)
[![Liberapay](https://img.shields.io/badge/-Liberapay-f6c915?logo=Liberapay&logoColor=222222)](https://liberapay.com/pimalaya)
[![thanks.dev](https://img.shields.io/badge/-thanks.dev-000000?logo=data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQuMDk3IiBoZWlnaHQ9IjE3LjU5NyIgY2xhc3M9InctMzYgbWwtMiBsZzpteC0wIHByaW50Om14LTAgcHJpbnQ6aW52ZXJ0IiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwYXRoIGQ9Ik05Ljc4MyAxNy41OTdINy4zOThjLTEuMTY4IDAtMi4wOTItLjI5Ny0yLjc3My0uODktLjY4LS41OTMtMS4wMi0xLjQ2Mi0xLjAyLTIuNjA2di0xLjM0NmMwLTEuMDE4LS4yMjctMS43NS0uNjc4LTIuMTk1LS40NTItLjQ0Ni0xLjIzMi0uNjY5LTIuMzQtLjY2OUgwVjcuNzA1aC41ODdjMS4xMDggMCAxLjg4OC0uMjIyIDIuMzQtLjY2OC40NTEtLjQ0Ni42NzctMS4xNzcuNjc3LTIuMTk1VjMuNDk2YzAtMS4xNDQuMzQtMi4wMTMgMS4wMjEtMi42MDZDNS4zMDUuMjk3IDYuMjMgMCA3LjM5OCAwaDIuMzg1djEuOTg3aC0uOTg1Yy0uMzYxIDAtLjY4OC4wMjctLjk4LjA4MmExLjcxOSAxLjcxOSAwIDAgMC0uNzM2LjMwN2MtLjIwNS4xNTYtLjM1OC4zODQtLjQ2LjY4Mi0uMTAzLjI5OC0uMTU0LjY4Mi0uMTU0IDEuMTUxVjUuMjNjMCAuODY3LS4yNDkgMS41ODYtLjc0NSAyLjE1NS0uNDk3LjU2OS0xLjE1OCAxLjAwNC0xLjk4MyAxLjMwNXYuMjE3Yy44MjUuMyAxLjQ4Ni43MzYgMS45ODMgMS4zMDUuNDk2LjU3Ljc0NSAxLjI4Ny43NDUgMi4xNTR2MS4wMjFjMCAuNDcuMDUxLjg1NC4xNTMgMS4xNTIuMTAzLjI5OC4yNTYuNTI1LjQ2MS42ODIuMTkzLjE1Ny40MzcuMjYuNzMyLjMxMi4yOTUuMDUuNjIzLjA3Ni45ODQuMDc2aC45ODVabTE0LjMxNC03LjcwNmgtLjU4OGMtMS4xMDggMC0xLjg4OC4yMjMtMi4zNC42NjktLjQ1LjQ0NS0uNjc3IDEuMTc3LS42NzcgMi4xOTVWMTQuMWMwIDEuMTQ0LS4zNCAyLjAxMy0xLjAyIDIuNjA2LS42OC41OTMtMS42MDUuODktMi43NzQuODloLTIuMzg0di0xLjk4OGguOTg0Yy4zNjIgMCAuNjg4LS4wMjcuOTgtLjA4LjI5Mi0uMDU1LjUzOC0uMTU3LjczNy0uMzA4LjIwNC0uMTU3LjM1OC0uMzg0LjQ2LS42ODIuMTAzLS4yOTguMTU0LS42ODIuMTU0LTEuMTUydi0xLjAyYzAtLjg2OC4yNDgtMS41ODYuNzQ1LTIuMTU1LjQ5Ny0uNTcgMS4xNTgtMS4wMDQgMS45ODMtMS4zMDV2LS4yMTdjLS44MjUtLjMwMS0xLjQ4Ni0uNzM2LTEuOTgzLTEuMzA1LS40OTctLjU3LS43NDUtMS4yODgtLjc0NS0yLjE1NXYtMS4wMmMwLS40Ny0uMDUxLS44NTQtLjE1NC0xLjE1Mi0uMTAyLS4yOTgtLjI1Ni0uNTI2LS40Ni0uNjgyYTEuNzE5IDEuNzE5IDAgMCAwLS43MzctLjMwNyA1LjM5NSA1LjM5NSAwIDAgMC0uOTgtLjA4MmgtLjk4NFYwaDIuMzg0YzEuMTY5IDAgMi4wOTMuMjk3IDIuNzc0Ljg5LjY4LjU5MyAxLjAyIDEuNDYyIDEuMDIgMi42MDZ2MS4zNDZjMCAxLjAxOC4yMjYgMS43NS42NzggMi4xOTUuNDUxLjQ0NiAxLjIzMS42NjggMi4zNC42NjhoLjU4N3oiIGZpbGw9IiNmZmYiLz48L3N2Zz4=)](https://thanks.dev/u/gh/soywod)
[![PayPal](https://img.shields.io/badge/-PayPal-0079c1?logo=PayPal&logoColor=ffffff)](https://www.paypal.com/paypalme/soywod)
