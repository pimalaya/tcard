---
cairn: tasks
change: parse-once
---

- [ ] Confirm vcard-rs's model covers every property the projection shows, across 2.1, 3.0 and 4.0
- [ ] Port template/model.rs and template/mod.rs onto vcard-rs's model
- [ ] Fold back through vcard-rs's tree layer, retiring src/edit
- [ ] Drop the calcard dependency
- [ ] Un-ignore the escape and gender reproductions, which close with the reader that caused them
- [ ] Drop the list-item filter the projection generators carry for the escape bug, widening the laws again
- [ ] Verify projection equality and byte-exact round-trip across the whole fixture corpus
- [ ] Verify the four build configurations and that the crate stays no_std
