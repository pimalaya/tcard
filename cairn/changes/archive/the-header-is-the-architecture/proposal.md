---
cairn: change
id: the-header-is-the-architecture
status: landed
created: 2026-08-29
---

# The header is the architecture, not the README

## Why

lib.rs opened with a doc attribute including README.md, so the crate's rustdoc was its public presentation: the badges, the install instructions, the sponsors. header-003 forbids exactly that, because the README and the header are two different documents by design, and header-001 says what should have stood there instead. The lib.rs header is the crate's architecture document, the one thing a reader cannot recover by reading the code.

Deleting the attribute on its own would leave the crate documented by nothing at all, which is worse than documented by the wrong thing. header-004 points the same way: the architecture header is compressed, never shortened by dropping what it is for.

tCard and tCal are one design written twice, so a reader who knows one should recognise the other on sight. Two headers written independently would lose that, and the projection, the fold-back and the merge forcing are the same three ideas in both.

## What

Remove the include and write the architecture header each crate should have had, in four sections: the layers, the projection pipeline, the merge and its forcing mechanism, and the module layout. The header opens by naming what the tool is and points at cairn/spec for the behaviour behind it.

The two headers are siblings: the same section titles in the same order, differing only where vCard and the other format genuinely differ. main.rs gets its own header carrying what the binary itself does, which is parsing, logging and printing, and points at the library for everything else, so the two files never restate each other.
