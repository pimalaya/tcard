---
cairn: log
change: the-header-is-the-architecture
landed: 2026-08-29
---

# The header is the architecture, not the README

lib.rs carried a doc attribute including README.md, so docs.rs would have published the install instructions and the sponsor badges as the crate's documentation, and the architecture header header-001 asks for did not exist. Removing the attribute alone would have left the crate with no documentation, so the header was written rather than the line simply deleted.

## What landed

Four sections, the same in tCard and tCal and in the same order. Layers says the crate does no I/O and has no client layer, and which modules the always-compiled core is. The projection says a body is read once, that the projection walks that one tree, and that a fold-back patches a line rather than rebuilding it. The merge says how a collision becomes the same key written twice and why TOML refusing it is the forcing mechanism. Modules names each module and what it owns, the crate-private layer under the projection included.

main.rs, which had no header at all, gained one: it parses the interface, wires the logger and the printer, and hands over. It points at the library for the architecture rather than repeating it.

## What it cost

The README's Rust example was compiled as a doctest through the include, and is not any more. That is the same trade-off the sibling libraries already make: vcard-rs and ical-rs publish headers, not READMEs, and their examples are not doctested either.

The projection's submodules are crate-private, so the header names them in prose rather than linking them: a public page linking a private item is a rustdoc warning, and cargo doc has to stay clean.

## Verification

Every configuration builds, clippy is clean over all targets, cargo fmt run, the whole suite green, and cargo doc --no-deps --all-features is warning free.

Capabilities moved: documentation (ADDED: The crate header is the architecture document, The binary header carries only the binary).
