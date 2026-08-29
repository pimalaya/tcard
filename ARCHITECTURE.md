# tcard architecture

Read the [Pimalaya ARCHITECTURE](https://github.com/pimalaya/.github/blob/master/ARCHITECTURE.md) first: it describes the conventions every Pimalaya repository shares (layering, `no_std`, module and error rules, code style, licensing). This document only covers what is specific to tcard, and assumes you know that shared context.

If a statement here conflicts with the code, the code wins; please flag it.

## Where tcard fits

tcard is a **dual library/CLI** crate (org ARCHITECTURE section 4), but a small and unusual one: it does **no I/O of its own and has no protocol or storage logic**, so it has no coroutines and no `client` layer. It is a pure, total function over strings: vCard text in, TOML text out, and back. The two layers are therefore:

1. **`no_std` core** (no features): the projection between a vCard and an ergonomic TOML buffer (`vcard`, `template`, `error`), plus the three-way merge over it (`merge`).
2. **CLI** (`cli` feature): the binary and its three verbs, plus the `$EDITOR` integration and `std`.

The "sans-I/O" principle still holds, trivially: the core never touches the filesystem, clock or network. The CLI is the only place that reads files and `$EDITOR`.

## The two directions

tcard converts between a [vcard-rs](https://crates.io/crates/vcard-rs) syntax tree and a TOML buffer in two directions:

- **`project`** (read): turn a vCard into a fillable, commented TOML scaffold. Because vCard has a single record type (so there is nothing to select), `project` flattens a single card (or a blank file) at the document root and only emits `[[card]]` blocks for two or more cards.
- **`apply`** (write): fold an edited TOML buffer back onto the **original vCard text**. It detects the buffer's shape (a `[[card]]` key means blocks, otherwise a flat single card).

The central decision is that **a body is read once**. The tree the merge reconciles is the tree the projection walks, so no value passes through a second reader that might normalise it, and `apply` rewrites only the lines whose modeled value actually changed.

Everything tcard does not model (other properties, `UID`, `VERSION`, `X-*`, parameter casing, order) is carried through verbatim, and a modeled line is patched rather than rebuilt, so the parameters and hidden components the form does not show survive too. A group (Apple's `item1.EMAIL`) is a label on a line rather than part of the property, so a line is matched by the name behind it and keeps the group it carried.

The boundary is that **tcard owns the content-line grammar it patches, vcard-rs owns the card structure and the bytes**. `template/patch.rs` is that grammar: it splits a line at the colon its parameters do not quote, reads its `TYPE` values, and writes a value back behind the prefix the line already carried. The projection reads through the same grammar, so what the form shows and what a fold-back writes agree by construction.

This is why `apply` always needs the original text, not just the edited TOML: the TOML is an editing affordance, not an interchange format.

## The modeled vocabulary

What tcard projects is described by a static `FIELDS` table in `template/model.rs`:

- A `Field { key, name, req, hint, kind }` decouples the friendly TOML `key` (`address`) from the vCard property `name` (`ADR`), so keys can be readable without touching parsing or emission. `req` marks `FN` (always required) and `N` (required before vCard 4.0 only); `hint` is the inline comment shown next to a value.
- The `Kind` enum drives both directions per field: `Scalar` (`FN`, `NOTE`), `List` (joined on a separator: `NICKNAME`, `ORG`, `CATEGORIES`), `Structured` (named, ordered components: `N`, `GENDER`), `Typed` (a repeatable `[[...]]` section with an optional `TYPE` and a single value: `EMAIL`, `TEL`, `URL`, `PHOTO`) and `TypedStructured` (typed plus components: `ADR`). Structured values expand into named keys instead of bare semicolons; typed properties list their accepted `TYPE` values inline. A structured component can be marked deprecated (RFC 6350's `ADR` `pobox` / `ext`): it is hidden from the scaffold in vCard 4.0 and flagged `# deprecated` in older versions, while on apply it keeps its positional slot and is written back from the card's own line, since hiding a component is no licence to drop it.

`UID` and `VERSION` are intentionally not modeled: they are app-managed, seeded for new cards and preserved otherwise.

## Layout: bare keys then sections

TOML attributes every bare key after a `[table]` / `[[array]]` header to that table. So the scalar and list fields are projected first as one aligned block, and the sectioned properties (`N`, `EMAIL`, `ADR`, ...) follow. Inline `#` hints in a block share one column, reached with tabs (a tab stop past the widest hinted line), so filling a value shifts the comments as little as possible.

## Module layout

```
src/
  lib.rs                 the architecture header, no_std setup, module wiring
  main.rs                [cli] binary entry point: parse, log, print, dispatch
  error.rs               TcardError + Result
  vcard.rs               the one reader: Cards, Card, the fold-back edits
  merge.rs               three-way merge, decided as a document
  cli.rs                 [cli] Cli/Command, the three verbs, the editor round trip
  template.rs            projection/apply engine + facade + unit tests
  template/
    model.rs             Kind, Field, Req, the static FIELDS table
    patch.rs             the content-line grammar the fold-back patches
    datetime.rs          dates between the wire form and native TOML
    line.rs              Line + tab-aligned comment emission
    util.rs              TOML rendering, escaping, reading a line's value
```

template.rs holds the public facade (`project`, `apply`) and the projection/apply orchestration; the submodules hold the model and the per-domain value conversions.

## The golden fixture database

`tests/data/` is a regression database of real and crafted vCards, checked by `tests/fixtures.rs`. Each `<name>.<mode>.toml` is the expected projection of `<name>.vcf` for `<mode>` (`all` projects the whole file: a single card flat at the root, two or more as `[[card]]` blocks). The runner asserts that projection equals the `.toml` for every fixture, and a byte-exact round-trip (`apply` reproduces the source) unless a `<name>.lossy` marker says the source is not already in the form a fold-back writes. Real-world exports are the most valuable cases; adding one is the fastest way to turn a bug report into a test (see [CONTRIBUTING.md](./CONTRIBUTING.md)).

The imported cards come from real apps via the [ez-vcard](https://github.com/mangstadt/ez-vcard) test corpus (`ezvcard_*`: Gmail, Evolution, MS Outlook 2.1) and the [calcard](https://crates.io/crates/calcard) parser's own corpus (`calcard_*`), alongside the RFC 6350 example (`rfc6350_author`) and an Apple-style export (`apple_contacts`); these are all `.lossy`. `clean` and `two_cards` are crafted to round-trip byte-exact.

## Known limitations

These are deliberate (or pending), and explain the `.lossy` markers:

- **Structured trailing empties**: `N` / `ADR` components are joined with trailing empties dropped, so `N:Doe;John;;;` is re-emitted `N:Doe;John`.
- **Folding**: vcard-rs unfolds a line on parse and does not record where the folds were, so a card written with folded lines comes back unfolded. The value is intact; only its layout moves. The sibling ical-rs keeps a wire shape for exactly this, and vcard-rs does not yet.
- **Date fields**: `BDAY` / `ANNIVERSARY` are read as the card wrote them and written back in RFC 6350 basic form, so an extended date (`1996-04-15`) comes back `19960415`.
- **Escaping**: a text value is written back escaped per RFC 6350 section 3.4, so an unescaped comma a card carried (`FN:Doe, John`) comes back escaped, and a needless escape (`http\://x`) comes back without it.
- **Parameters on a retyped line**: a line whose `TYPE` the document changed is rewritten with the document's spelling and its type parameters gathered into one `TYPE=`. A line whose types are unchanged, however they are spelled (`;WORK`, `TYPE="work,voice"`, one parameter per value), keeps its bytes.
