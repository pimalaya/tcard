---
cairn: change
id: the-merge-is-not-a-build-option
status: landed
created: 2026-08-29
---

# The merge is not a build option

## Why

The `merge` cargo feature gates one thing, the `pub mod merge` declaration. It pulls no crate into the build: vcard-rs stopped being optional when calcard went, and it is now a plain dependency every configuration compiles.

That is the guideline's own test failing. A cargo feature is justified only when it pulls additional crates into the build (crate-003), and gating code that changes nothing about the crate set is a switch that costs a build configuration and buys nothing. The feature arrived when the library was optional, and dropping calcard made it not.

It is not free either. It is a fourth configuration to build and test, a `#[cfg]` on the test file, a `cli` feature that has to name it, and a paragraph in the README, the spec and the architecture document each saying the merge is opt-in when nothing about the build changes when it is opted out of.

The same removal landed in neverest this morning, for the same reason. This is tCard's half of it, done with tCard and tCal together so the two stay one shape.

## What

- Delete the `merge` feature from Cargo.toml, and stop `cli` naming it.
- Ship `merge` unconditionally: no `#[cfg]` on the module, none on the merge test file.
- Say so in the README, the architecture document and the spec, which all describe the capability as opt-in.

## Not changed

No behaviour moves for any build anyone would have made. The `no_std` core keeps its shape, the merge module allocating no more than the projection beside it, and every build that carried the merge still carries it.
