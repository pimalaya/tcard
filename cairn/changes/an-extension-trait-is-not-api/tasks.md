---
cairn: tasks
change: an-extension-trait-is-not-api
---

- [x] Make the trait `pub(crate)` and rename it back to `Card`
- [x] Say in the vcard module header why it is a trait at all
- [x] Drop it from the changelog and from the naming change that had prefixed it
- [x] Verify the suite, clippy and rustdoc with no link into a private item
