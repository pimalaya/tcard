---
cairn: change
id: a-module-with-code-is-a-sibling
status: landed
created: 2026-08-29
---

# A module carrying code is a sibling file, not a mod.rs

## Why

The convention is content-based: a pure aggregator, holding only module declarations and re-exports, lives in foo/mod.rs, and a module carrying code of its own is a sibling foo.rs next to the foo/ folder (naming-002). The projection is the largest module in the crate and holds the whole engine, so its mod.rs is not an aggregator by any reading.

It survived because tCard and tCal were wrong the same way, and comparing the twins is how everything else here got found. tCal's own architecture document already draws the layout as template.rs beside template/, so the code was the odd one out even inside its own repository.

## What

- Move src/template/mod.rs to src/template.rs in both tools. Nothing else moves: the module path, the declarations and the code are untouched.
- The folder's other files are checked against the same test and stay where they are, each being a leaf module with no folder of its own.

While the module headers are open, the ones missing the markdown title every header opens with (inline-001) get it, so the two crates read the same from the first line of every module.
