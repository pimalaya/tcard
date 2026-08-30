---
cairn: spec
capability: documentation
status: current
---

# Documentation

How the crate documents itself: where the architecture is written down, what the binary's own header carries, and what shape the changelog holds. The behaviour of the tool is specified by the other capabilities; this one is about the documents around them staying true.

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

### Requirement: The changelog is one net diff
A release section SHALL carry at most one heading per kind, and each entry SHALL state the behaviour as it ends up rather than the steps that reached it.

A release section answers what moved since the previous version, and a reader answers that by reading its headings once. A second heading of the same kind turns reading into searching: the reader who takes the first Added for the additions has no way of knowing the list resumes further down, past a heading that looked like the end of it.

History is what the cairn log is for. A changelog that keeps a second copy of it keeps a worse one, since an entry later undone still reads as current.

#### Scenario: A section that grew in two sittings
- GIVEN an unreleased section holding additions written at different times
- WHEN the changelog is read
- THEN all of them are under the one Added heading

### Requirement: The product is written tCard
Prose SHALL write the product name tCard, and `tcard` SHALL be reserved for the identifier: the crate, the binary, the module path and a shell command. The document a person edits carries the prose form.

The name is the one thing a reader sees before anything else, in the header of every document the tool generates. Writing the identifier there says the tool is its own binary name, which is a detail of packaging rather than what the thing is called.

#### Scenario: The header of a generated document
- GIVEN a projected TOML form
- WHEN its header comment is read
- THEN it names tCard
