---
cairn: tasks
change: parse-once
---

- [x] Confirm vcard-rs's model covers every property the projection shows, across 2.1, 3.0 and 4.0
- [x] Port template/model.rs and template/mod.rs onto vcard-rs's model
- [x] Fold back through vcard-rs's tree layer, retiring src/edit
- [x] Drop the calcard dependency
- [x] Un-ignore the escape and gender reproductions, which close with the reader that caused them
- [x] Drop the list-item filter the projection generators carry for the escape bug, widening the laws again
- [x] Verify projection equality and byte-exact round-trip across the whole fixture corpus
- [x] Verify the four build configurations and that the crate stays no_std
