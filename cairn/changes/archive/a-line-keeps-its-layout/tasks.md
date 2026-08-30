---
cairn: tasks
change: a-line-keeps-its-layout
---

- [x] Depend on vcard-rs 0.3 from crates.io and drop the git patch
- [x] Stamp a built line's value and parameters with the escaping rules of its card's version
- [x] Add a crafted folded fixture that round trips byte-exact, with no lossy marker
- [x] Assert both halves: an untouched folded line keeps its folds, an edited one goes out unfolded
- [x] Retire the unfolding limitation from the src/lib.rs header
- [x] Correct the CONTRIBUTING note that says Cargo.toml patches vcard-rs to git
