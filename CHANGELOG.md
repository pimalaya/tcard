# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Offered to re-edit on a broken `edit` buffer instead of discarding it.

  When the edited TOML fails to parse, `edit` now shows the parse error and prompts to re-open `$EDITOR` seeded with the user's own buffer, looping until it parses or the user declines. JSON output stays non-interactive: the error just propagates.

- Added the `project` / `apply` projection library between a calcard `VCard` and an ergonomic TOML buffer.

  The library is `#![no_std]` (alloc only) and does just the TOML projection. The opt-in `cli` feature adds the command-line tool (clap, file/stdin I/O, std) and its `$EDITOR` round-trip; the library has no default features.

  Because vCard has a single component type, `project` flattens a single card (or a blank file) at the document root (bare keys, top-level `[name]` / `[[email]]`, no wrapper) and only emits `[[card]]` blocks for two or more cards. `apply` detects which shape the buffer is (a `[[card]]` key means blocks; otherwise a flat single card) and reconciles accordingly. Cryptic property names project to friendly kebab-cased keys (`FN` -> `full-name`, `N` -> `name`, `BDAY` -> `birthday`, `TZ` -> `timezone`, `ORG` -> `organization`, `LANG` -> `language`, `TEL` -> `phone`, `IMPP` -> `messaging`, `ADR` -> `address`). Fields are uncommented and empty (an empty value is ignored, like a removed line), prefilled when present, and carry a tab-aligned inline `#` hint (a concrete example, an enum list, or a short description) only where the value is not self-evident (`birthday`, `geo`, `timezone`, `email`, `phone`, `url`, `photo`, `messaging`, `gender`); required properties are flagged `# required` version-aware (`FN` always, `N` before 4.0). Typed properties (`email`, `phone`, `address`, `url`) list their accepted `TYPE` values in a trailing comment. The `ADR` components deprecated by RFC 6350 (`pobox`, `ext`) are hidden from the scaffold in vCard 4.0 and flagged `# deprecated` in older versions, while their positional slot is preserved on apply. Date fields (`BDAY`, `ANNIVERSARY`), which calcard parses to a typed value with no text accessor, are rendered back in RFC 6350 basic form (`19960415`, `--0415`) instead of being dropped. `UID` is not modeled: like `VERSION` it is app-managed (seeded for new cards, preserved otherwise) and cannot be set through the buffer. `apply` patches the modeled fields back onto the original text through a format-preserving editor, re-rendering only the lines that actually changed; a filled block updates or adds a card, an empty or absent block removes it, and every unmodeled property (custom `X-*`, vendor extensions, Apple `item1.*` groups) and all folding, casing and ordering are kept byte-for-byte, since the TOML is an editing affordance rather than an interchange format.

- Added the `edit` module, a format-preserving vCard editor (the `toml_edit` analog for vCard).

  It parses a vCard stream into a tree that keeps every content line's original bytes, unfolds folded lines for matching, and re-renders only the properties a caller mutates via `Component::set_all`. `components`/`components_mut` and `properties`/`properties_mut` iterators (`.nth(i)` plus `Property::set`) address one occurrence, and `Card::set_component_count` adds or drops whole cards. The round-trip invariant is `Card::parse(s).to_string() == s`. It is `#![no_std]` (alloc only), calcard-independent, and powers `apply`'s minimal diffs.

- Added the `tcard` CLI with two verbs.

  `template [SOURCE]` prints the TOML scaffold (blank or prefilled). `edit [SOURCE]` runs the full "project → `$EDITOR` → apply" round-trip and emits the resulting vCard, writing a file source back in place. `SOURCE` resolves deterministically: `-` reads stdin, an existing file is read, otherwise the value is treated as literal vCard contents, and omitting it starts from a blank template. A `-V`/`--version` flag on each verb selects the target vCard version (the root `--version` stays the app version), and new (sourceless) cards are seeded with a fresh `urn:uuid` v4 `UID`.

- Added a golden fixture test database under `tests/data/`: real-world and crafted vCards (`<name>.vcf`) each with their expected TOML projection (`<name>.<mode>.toml`), asserting projection equality and, unless flagged `.lossy`, byte-exact round-trip. Real cards are imported from the [ez-vcard](https://github.com/mangstadt/ez-vcard) app-export corpus (Gmail, Evolution, MS Outlook 2.1) and the [calcard](https://crates.io/crates/calcard) parser corpus, spanning vCard 2.1/3.0/4.0 and single/multi-card files.
