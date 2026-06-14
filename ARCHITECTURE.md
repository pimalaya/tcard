# tcard architecture

Read the [Pimalaya ARCHITECTURE](https://github.com/pimalaya/.github/blob/master/ARCHITECTURE.md) first: it describes the conventions every Pimalaya repository shares (layering, `no_std`, module and error rules, code style, licensing). This document only covers what is specific to tcard, and assumes you know that shared context.

If a statement here conflicts with the code, the code wins; please flag it.

## Where tcard fits

tcard is a **dual library/CLI** crate (org ARCHITECTURE section 4), but a small and unusual one: it does **no I/O of its own and has no protocol or storage logic**, so it has no coroutines and no `client` layer. It is a pure, total function over strings: vCard text in, TOML text out, and back. The two layers are therefore:

1. **`no_std` core** (no features): the projection between a vCard and an ergonomic TOML buffer (`vcard`, `template`, `edit`, `error`).
2. **CLI** (`cli` feature): the binary and its two verbs, plus the `$EDITOR` integration and `std`.

The "sans-I/O" principle still holds, trivially: the core never touches the filesystem, clock or network. The CLI is the only place that reads files and `$EDITOR`.

## The two directions

tcard converts between a [calcard](https://crates.io/crates/calcard) `VCard` and a TOML buffer in two directions:

- **`project`** (read): turn a vCard into a fillable, commented TOML scaffold. calcard is the reader; it parses and validates values. Because vCard has a single component type (so there is nothing to select), `project` flattens a single card (or a blank file) at the document root and only emits `[[card]]` blocks for two or more cards.
- **`apply`** (write): fold an edited TOML buffer back onto the **original vCard text**. It detects the buffer's shape (a `[[card]]` key means blocks, otherwise a flat single card).

The central decision is that **calcard is used as a reader only**. Its writer normalises folding, parameter casing (`TYPE=work` becomes `TYPE=WORK`) and property order, so re-serialising a card churns lines nobody touched. Instead `apply` patches the original bytes through `crate::edit`, an in-house format-preserving editor (the `toml_edit` analog for vCard): it keeps every content line's original bytes and re-renders only the lines whose modeled value actually changed. Its invariant is `Card::parse(s).to_string() == s` for any input; on top of it, projecting then applying an untouched buffer reproduces the source byte-for-byte. Everything tcard does not model (other properties, `UID`, `VERSION`, grouped `item1.*` properties, `X-*`, folding, casing, order) is carried through verbatim.

This is why `apply` always needs the original text, not just the edited TOML: the TOML is an editing affordance, not an interchange format.

## The modeled vocabulary

What tcard projects is described by a static `FIELDS` table in `template/model.rs`:

- A `Field { key, name, req, hint, kind }` decouples the friendly TOML `key` (`address`) from the vCard property `name` (`ADR`), so keys can be readable without touching parsing or emission. `req` marks `FN` (always required) and `N` (required before vCard 4.0 only); `hint` is the inline comment shown next to a value.
- The `Kind` enum drives both directions per field: `Scalar` (`FN`, `NOTE`), `List` (joined on a separator: `NICKNAME`, `ORG`, `CATEGORIES`), `Structured` (named, ordered components: `N`, `GENDER`), `Typed` (a repeatable `[[...]]` section with an optional `TYPE` and a single value: `EMAIL`, `TEL`, `URL`, `PHOTO`) and `TypedStructured` (typed plus components: `ADR`). Structured values expand into named keys instead of bare semicolons; typed properties list their accepted `TYPE` values inline. A structured component can be marked deprecated (RFC 6350's `ADR` `pobox` / `ext`): it is hidden from the scaffold in vCard 4.0 and flagged `# deprecated` in older versions, while its positional slot is kept on apply.

`UID` and `VERSION` are intentionally not modeled: they are app-managed, seeded for new cards and preserved otherwise.

## Layout: bare keys then sections

TOML attributes every bare key after a `[table]` / `[[array]]` header to that table. So the scalar and list fields are projected first as one aligned block, and the sectioned properties (`N`, `EMAIL`, `ADR`, ...) follow. Inline `#` hints in a block share one column, reached with tabs (a tab stop past the widest hinted line), so filling a value shifts the comments as little as possible.

## Module layout

```
src/
  lib.rs                 no_std setup, module + feature wiring
  error.rs               TcardError + Result
  vcard.rs               calcard parse adapter (text -> VCard)
  cli.rs                 [cli] binary: Cli/Command, template & edit verbs
  template/
    mod.rs               projection/apply engine + facade + unit tests
    model.rs             Kind, Field, Req, the static FIELDS table
    line.rs              Line + tab-aligned comment emission
    util.rs              TOML / escape / calcard-value helpers
  edit/
    mod.rs               module root for the format-preserving editor
    tree.rs              Card/Component/Property + Nodes DOM
    parse.rs             Parser: unfold + build the tree
    render.rs            fold content lines, detect end-of-line
```

`template/mod.rs` holds the public facade (`project`, `apply`) and the projection/apply orchestration; the submodules hold the model and the per-domain value conversions.

## The golden fixture database

`tests/data/` is a regression database of real and crafted vCards, checked by `tests/fixtures.rs`. Each `<name>.<mode>.toml` is the expected projection of `<name>.vcf` for `<mode>` (`all` projects the whole file: a single card flat at the root, two or more as `[[card]]` blocks). The runner asserts that projection equals the `.toml` for every fixture, and a byte-exact round-trip (`apply` reproduces the source) unless a `<name>.lossy` marker says the source is not already in calcard's canonical form. Real-world exports are the most valuable cases; adding one is the fastest way to turn a bug report into a test (see [CONTRIBUTING.md](./CONTRIBUTING.md)).

The imported cards come from real apps via the [ez-vcard](https://github.com/mangstadt/ez-vcard) test corpus (`ezvcard_*`: Gmail, Evolution, MS Outlook 2.1) and the [calcard](https://crates.io/crates/calcard) parser's own corpus (`calcard_*`), alongside the RFC 6350 example (`rfc6350_author`) and an Apple-style export (`apple_contacts`); these are all `.lossy`. `clean` and `two_cards` are crafted to round-trip byte-exact.

## Known limitations

These are deliberate (or pending), and explain the `.lossy` markers:

- **Structured trailing empties**: `N` / `ADR` components are joined with trailing empties dropped, so `N:Doe;John;;;` is re-emitted `N:Doe;John`.
- **Deprecated address components**: RFC 6350 deprecates `ADR`'s `pobox` and `ext`, so they are hidden from the scaffold for vCard 4.0 (and flagged `# deprecated` for older versions). Their positional slots are still preserved on apply, read back by key; but a non-conformant 4.0 card that carries data there loses it when its address line is rewritten.
- **Multi-valued structured sub-components**: each `N` / `ADR` component is modeled as one value, so an unescaped comma list (`N:Doe;John;Richter,James`, i.e. two additional names) keeps only the first; escape the comma (`Richter\,James`) to carry both as a single value.
- **Date fields**: `BDAY` / `ANNIVERSARY` are parsed by calcard to a typed date (no text accessor), so they are rendered back from its parts in RFC 6350 basic form (`19960415`, `--0415`). calcard parses an extended full date (`1996-04-15`) as year-month only, dropping the day on read, so the basic form round-trips best.
- **Grouped properties**: an Apple-style `item1.ADR` is read as `ADR` but its `item1.` group prefix is not modeled, so a rewrite emits a bare `ADR` rather than editing the grouped line in place.
- **Unmodeled parameters**: only `TYPE` is modeled on typed properties; others (`PREF`, `VALUE=uri`, ...) are dropped when that property line is rewritten.
- **Parameter casing**: a rewritten line emits `TYPE` values as calcard parsed them (lowercase), so a source written `TYPE=WORK` round-trips as `TYPE=work`.
