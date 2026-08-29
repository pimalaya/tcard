---
cairn: log
change: a-module-with-code-is-a-sibling
landed: 2026-08-29
---

# A module carrying code is a sibling file, not a mod.rs

src/template/mod.rs held the whole projection engine, 490 lines of it, under a file name reserved for a folder's aggregator.

## What landed

The file moved to src/template.rs, beside the src/template/ folder it declares. Nothing else moved: the module path, the declarations, the code and the tests are byte for byte what they were, and the compiler saw no difference at all. The architecture document, which already drew the layout this way in one of the two repositories, now matches the tree in both.

The folder's other files were checked against the same test and stay where they are: each is a leaf module with no folder under it, so mod.rs is not in question for any of them. Neither repository carries a second module folder.

## Why it survived this long

tCard and tCal were wrong in the same way, and comparing the twins is how nearly everything else here was found. Two identical mistakes look like a convention.

## Riding along

The module headers missing the markdown title every header opens with got it, so both crates now read the same from the first line of every module.

## Verification

Both configurations build, clippy is clean over all targets, `cargo fmt` run, the whole suite green.

Capabilities moved: template (ADDED: The projection is a sibling module, not an aggregator).
