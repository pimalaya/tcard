# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-08-30

### Added

- Added `TcardTemplate`, the projection of a vCard as an ergonomic TOML form.

  Cryptic names become readable keys (`FN` as `full-name`, `ADR` as named components), a blank form lists every modelled property so it doubles as documentation, and a date projects as a native TOML value. `UID` and `VERSION` are app-managed: hidden, seeded for a new card, preserved for every other one.

- Added the fold-back, which rewrites only the lines the form changed.

  A modelled line is patched rather than rebuilt, so a parameter the form hides, a group prefix, the folds and the casing of every other line stay as the card wrote them. A card is read once, through [vcard-rs](https://crates.io/crates/vcard-rs)'s byte-faithful tree.

- Added `TcardMerge`, a three-way merge projected as that same form.

  What both sides changed is written once per side under one key, which TOML refuses, so an undecided document cannot be applied and the refusal names the field. What the merge settled on its own is said in the document's header instead.

- Added the `tcard` CLI behind the opt-in `cli` feature: `template`, `edit` and `merge`.

  A source is a file, `-` for stdin, literal vCard contents, or nothing for a blank form. `edit` opens `$EDITOR` and offers to re-open a buffer that does not fold back. `-V`/`--version` picks the vCard version a new card is written at.

- Added a `no_std` core over `alloc`, so a library consumer pays for none of the CLI.

- Added the golden fixture database under tests/data: real and crafted cards asserting the projection and, where the source is already in the form a fold-back writes, a byte-exact round trip.

[unreleased]: https://github.com/pimalaya/tcard/compare/v0.1.0..HEAD
[0.1.0]: https://github.com/pimalaya/tcard/compare/root..v0.1.0
