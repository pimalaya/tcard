---
cairn: delta
change: the-header-is-the-architecture
---

## ADDED Requirements

### Requirement: The crate header is the architecture document
The lib.rs header SHALL be the crate's architecture document, structured by sections, and the README SHALL NOT be included as the crate's rustdoc.

The README is the public presentation, written for someone deciding whether to install the tool. The header is written for someone about to read or change the code, and it is the only place the shape of the crate is stated: which layer reads, which projects, which reconciles, and why the document a person edits is TOML. Publishing the first as the second leaves the second unwritten.

A section is compressed rather than dropped when the header grows too long, since the test of a cut is whether a reader who does not know the crate still learns the same architectural fact from it.

#### Scenario: The crate's rustdoc
- GIVEN the library built with any feature set
- WHEN its rustdoc is generated
- THEN the crate page is the architecture header, and holds no install instructions

### Requirement: The binary header carries only the binary
main.rs SHALL open with its own header, naming what the binary itself does and pointing at the library for the architecture.

The binary is the parser, the logger and the printer, and nothing else: every verb it runs lives in the library below it. A header restating the library's architecture would be a second copy to keep true, and the copy that rots is the one nobody publishes.

#### Scenario: The two headers side by side
- GIVEN lib.rs and main.rs
- WHEN both headers are read
- THEN the architecture is stated once, and the binary's header names only its own wiring
