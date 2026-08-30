# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added the `merge` verb and the `merge` module behind it, a three-way merge projected as TOML.

  `merge BASE LOCAL REMOTE --output PATH` reconciles two divergent cards against the base they came from, projects the outcome as TOML, opens `$EDITOR` on it, and writes the output path only once the edited document parses. A card that does not read refuses the merge naming the side it was given as, so the reader is told which of the three files to open. A field both sides changed is written once per surviving side, each line naming its side, with the ancestor commented above them: TOML forbids duplicate keys, so an undecided document cannot be applied, and `TcardMerged::apply` reports the duplicate-key parse error as the field left undecided rather than as a syntax error. A collision inside a structured value (`ADR`, `N`) decomposes into per-key duplicates inside the single table that projects the instance, never a repeated array-of-tables block, and one the report breaks into parts the projection has no key for (`ORG`) is put back together and contested as a whole array, once. Each value is written in the spelling the projection uses for that field, a native date included. The contest lands in the block of the instance the report names, falling back to a search by value where the report's base-card index and the merged card's order can disagree. What the merge already decided (a removal against an update, where the update wins), a collision on a part the projection does not show, an instance paired by position rather than by `PID`, and a list both sides edited, whose items are all kept since RFC 6350 gives them no order, are said in a comment at the top of the document instead, wrapped to the column the rest of the header uses.

- Offered to re-edit on a broken `edit` buffer instead of discarding it.

  When the edited TOML fails to parse, `edit` now shows the parse error and prompts to re-open `$EDITOR` seeded with the user's own buffer, looping until it parses or the user declines. JSON output stays non-interactive: the error just propagates.

- Added the `TcardTemplate` projection between a vCard file and an ergonomic TOML buffer.

  The library is `#![no_std]` (alloc only) and does just the TOML projection. The opt-in `cli` feature adds the command-line tool (clap, file/stdin I/O, std) and its `$EDITOR` round-trip; the library has no default features.

  Because vCard has a single component type, `TcardTemplate::project` flattens a single card (or a blank file) at the document root (bare keys, top-level `[name]` / `[[email]]`, no wrapper) and only emits `[[card]]` blocks for two or more cards. `TcardTemplate::apply` detects which shape the buffer is (a `[[card]]` key means blocks; otherwise a flat single card) and reconciles accordingly. Cryptic property names project to friendly kebab-cased keys (`FN` -> `full-name`, `N` -> `name`, `BDAY` -> `birthday`, `TZ` -> `timezone`, `ORG` -> `organization`, `LANG` -> `language`, `TEL` -> `phone`, `IMPP` -> `messaging`, `ADR` -> `address`). Fields are uncommented and empty (an empty value is ignored, like a removed line), prefilled when present, and carry a tab-aligned inline `#` hint (a concrete example, an enum list, or a short description) only where the value is not self-evident (`birthday`, `geo`, `timezone`, `email`, `phone`, `url`, `photo`, `messaging`, `gender`); required properties are flagged `# required` version-aware (`FN` always, `N` before 4.0). Typed properties (`email`, `phone`, `address`, `url`) list their accepted `TYPE` values in a trailing comment. The `ADR` components deprecated by RFC 6350 (`pobox`, `ext`) are hidden from the scaffold in vCard 4.0 and flagged `# deprecated` in older versions; a hidden component keeps its positional slot on apply and is written back from the card's own line rather than dropped. Date fields (`BDAY`, `ANNIVERSARY`) project as native TOML `date`/`datetime` values when complete (`1996-04-15`), falling back to a quoted RFC 6350 basic string for the partial forms TOML cannot hold (`--0415` yearless, `2009` year-only); `TcardTemplate::apply` reads either back. `UID` is not modeled: like `VERSION` it is app-managed (seeded for new cards, preserved otherwise) and cannot be set through the buffer. `TcardTemplate::apply` patches the modeled fields back onto the tree the projection walked, re-rendering only the lines that actually changed; a filled block updates or adds a card, an empty or absent block removes it, and every unmodeled property (custom `X-*`, vendor extensions) and all parameter casing and ordering are kept byte-for-byte, since the TOML is an editing affordance rather than an interchange format. A modeled line is patched rather than rebuilt, so a parameter the form does not show (`PREF`, `PID`, `LANGUAGE`, `VALUE`) survives, and a `TYPE` already spelled as the document writes it leaves the line's bytes alone. A property carried by a group (Apple's `item1.EMAIL`) is matched by the name behind the group and rewritten in place, never doubled by a group-less copy. Several properties of one repeatable name keep their identity: a list field's items fold back over the lines they came from, each keeping as many as it held, and a surplus item opens a line of its own.

- Added the `vcard` module, the one reader every verb uses.

  `TcardCards::parse` reads a whole stream into [vcard-rs](https://crates.io/crates/vcard-rs)'s byte-faithful syntax tree, so a body is read once: the tree the merge reconciles is the tree the projection walks, and no value passes through a second reader that might normalise it. `TcardCard::lines` and `TcardCard::set_lines` are the byte-preserving property edits a fold-back makes, and `TcardCards::set_count` adds or drops whole cards. An unchanged line keeps its own bytes, its parameters and its group included, and its wire layout with them: its folds, the blank lines before it and its quoted-printable soft breaks. Only a line the document moved is written anew, and such a line goes back out unfolded, the offsets a layout is recorded as no longer fitting its bytes. A line a fold-back builds is stamped with the escaping rules of the version its card declares, rather than assuming the latest.

- Added the `tcard` CLI with three verbs.

  `template [SOURCE]` prints the TOML scaffold (blank or prefilled). `edit [SOURCE]` runs the full "project → `$EDITOR` → apply" round-trip and emits the resulting vCard, writing a file source back in place. `SOURCE` resolves deterministically: `-` reads stdin, an existing file is read, otherwise the value is treated as literal vCard contents, and omitting it starts from a blank template. A `-V`/`--version` flag on each verb selects the target vCard version (the root `--version` stays the app version), and new (sourceless) cards are seeded with a fresh `urn:uuid` v4 `UID`. `merge BASE LOCAL REMOTE --output PATH` takes paths rather than a `SOURCE`, since a merge needs three cards at once. `--help` closes on the shared Pimalaya footer, the bug tracker and the sponsoring links.

- Added a golden fixture test database under tests/data: real-world and crafted vCards, NAME.vcf, each with their expected TOML projection, NAME.mode.toml, asserting projection equality and, unless a NAME.lossy marker says otherwise, byte-exact round-trip. Real cards are imported from the [ez-vcard](https://github.com/mangstadt/ez-vcard) app-export corpus (Gmail, Evolution, MS Outlook 2.1) and the [calcard](https://crates.io/crates/calcard) parser corpus, spanning vCard 2.1/3.0/4.0 and single/multi-card files.

- Added the crate architecture header on src/lib.rs and src/main.rs.

  The library's rustdoc is the architecture of the crate: its layers, the projection reading a body once, the merge writing what it could not settle as duplicate keys, and the module layout. The README stays the public presentation rather than doubling as the crate documentation, and the binary's header covers only its own wiring.

### Fixed

- A contested value was shown cut off at its first `;`.

  The document rendered a side's value by reading the first `;`-component of it, so a text value carrying an unescaped semicolon was put to the reader shorter than it is, and two sides differing only past that point read as the same value. A well-formed text value escapes its semicolons and has one component either way, so this only ever showed on a card that did not, which is exactly the card a reader most needs shown faithfully.
