---
cairn: log
change: the-merge-is-not-a-build-option
landed: 2026-08-29
---

# The merge is not a build option

The `merge` cargo feature gated one thing, the `pub mod merge` declaration, and pulled no crate into the build: vcard-rs stopped being optional when calcard went, and is a plain dependency every configuration compiles.

## What landed

The feature is gone from the manifest, `cli` no longer names it, `pub mod merge` ships unconditionally, and the merge forcing suite drops the `#![cfg]` that used to make it disappear from a build without the feature. The README, the architecture document and the merge spec no longer describe the capability as opt-in.

## Why the feature could not earn its keep

A cargo feature is justified only when it pulls additional crates into the build (crate-003). This one changed nothing about the crate set: the same vcard-rs was compiled either way, and the only thing the switch bought was one module missing from the public API. It arrived when the library was optional, and dropping calcard made it not.

It was not free. It was a fourth build configuration to keep green, a `#[cfg]` on the test file that silently ran fourteen fewer tests when it was off, a `cli` feature that had to remember to name it, and three documents each explaining an option that changed nothing.

## Not changed

No behaviour moves for any build anyone would have made. `default` was empty and `cli` pulled the feature in, so every binary already carried the merge. The `no_std` core keeps its shape: the merge module allocates no more than the projection beside it.

The same removal landed in neverest this morning and in tCal today, under the same change id.

## Verification

`--all-features` and `--no-default-features` both build, clippy is clean over all targets, `cargo fmt` run. The whole suite passes in both configurations, merge forcing included, which is the point: without the feature it used to compile to nothing.

Capabilities moved: merge (MODIFIED: Merging is a verb over three files).
