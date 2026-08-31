# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-09-01

### Added

- Added `--editor <COMMAND>` to `edit` and `merge`, naming the editor for one run, ahead of `$VISUAL` and `$EDITOR`.

  It is spawned on the path of a temporary TOML file it edits in place, so it must block until the edit is done: use `--editor "code --wait"`, not `--editor code`.

- A buffer that does not fold back is now kept and named when you decline to fix it, and when the editor exits non-zero.

  The error carries the path, which is the recovery: what you typed outlives the run that could not use it. A round trip that folded back still removes the file.

- Added inline hints to the name components, `additional` above all: the RFC role names say what a name is rather than where it is written, which is what varies between cultures, but nobody guesses that one means the middle names.

### Changed

- **BREAKING**: the editor is now `$VISUAL`, then `$EDITOR`, and nothing after those, where an unset pair used to fall through to a list of platform defaults.

  That list ended in `xdg-open`, `gnome-open`, `kde-open` and a bare `open`, which are file openers rather than editors: they hand the document to whatever the desktop associates with `.toml` and return before it is closed. tCard then read back a document nobody had touched yet and wrote the card out unchanged, which a caller spawning tCard reads as an edit given up on. Neither variable set is now a failure naming both of them and `--editor`.

  The [edit](https://crates.io/crates/edit) dependency is gone with it: what is left is a temporary file, a spawn with the three streams inherited, and a read back.

- The inline comments share one column across a card, where each block used to pick its own.

  Scrolling the form, the comments stepped in and out at every section. The card is the unit rather than the file, which is what tCal measures per component, so the two now agree.

### Fixed

- A list item now goes back to the line it came from, where the items used to be counted off the front of the array.

  A line's parameters describe the items that line carried, so counting instead handed each line whatever had room. Removing one item slid every item behind it onto the line before: deleting `Jimmy` from `NICKNAME;PREF=1:Jim,Jimmy` beside `NICKNAME;PREF=2:Big Tuna` left `NICKNAME;PREF=1:Jim,Big Tuna`, making `Big Tuna` the preferred nickname and deleting the line that said otherwise.

- Items added to a property that holds one line now join that line instead of each opening a bare line of its own.

  One line has nothing to disambiguate, so an added item can only belong to it, parameters and all. Where several lines make it unanswerable, the added items share one new line between them rather than taking one each. `ORG` is unaffected, its `;` joining one property's own components.

- **BREAKING**: a structured component that holds several values is an array in the form, where it used to be one string.

  `N`'s five components and `ADR`'s street are lists: RFC 6350 section 6.2.2 says a component "can include multiple text values separated by the COMMA character", and section 6.3.1 names a multi-line street as the case. Typed as one string, tCard could not tell that comma from one someone typed, and escaped it on the way out: editing any part of a name turned `Philip,Paul` into `Philip\,Paul`, two additional names into one. A bare string is still accepted where an array is expected.

- A component the document did not change now keeps the card's own bytes.

  A structured value is one line, so changing any component re-renders every component. Without this the same escaping reached the components that are not lists, and a `country` reading `Congo\, The Democratic Republic of the` came back changed by an edit to the street beside it.

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

[unreleased]: https://github.com/pimalaya/tcard/compare/v0.2.0..HEAD
[0.2.0]: https://github.com/pimalaya/tcard/compare/v0.1.0..v0.2.0
[0.1.0]: https://github.com/pimalaya/tcard/compare/root..v0.1.0
