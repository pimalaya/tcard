---
cairn: log
change: prefix-the-library
date: 2026-08-30
---

# The library carries its prefix, the CLI does not

`Cards`, `Template`, `Merge`, `Merged` and the `Result` alias became `TcardCards`, `TcardTemplate`, `TcardMerge`, `TcardMerged` and `TcardResult`, joining the `TcardError` that already carried it. Nothing under cli moved: `Cli`, `Command`, the three `*Command`s, `SourceArg`, `VersionArg`, `CardVersion`, `Editor` and `Output` are bare, which is the override cli-001 grants and not an oversight.

The line the rule draws is the `cli` feature gate, which makes it checkable rather than a matter of taste: what ships to a library consumer is prefixed, what only the binary sees is not.

## What the rename walked into

A mechanical rename over `Merge` and `Template` catches the English words too. Six module headers and doc sentences opened with one, "# Merge", "Merge the three cards into a document to decide", "Merge two divergent vCards", and came out prefixed; all are back to prose. The module names themselves never moved, so `crate::merge::TcardMerge` is the path, and the header of that module is still "# Merge".

The intra-doc links moved with the types, in the crate header and in the module ones, and the changelog entries naming the API were updated with them.

## Verification

The suite is green unchanged in what it asserts, 56 tests, plus clippy, rustdoc with no broken link and both feature builds.

Capabilities moved: `api`.
