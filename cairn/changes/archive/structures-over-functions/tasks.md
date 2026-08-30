---
cairn: tasks
change: structures-over-functions
---

- [x] Replace the free entry points with `Cards::parse`, `Template`, `Merge` and `Merged`
- [x] Give `Template` both directions, so a fold-back patches the tree its form came from
- [x] Split src/cli.rs into args, editor and one module per verb
- [x] Split src/merge.rs into choice, document and note under the facade
- [x] Drop thiserror, writing Display and Error by hand
- [x] Write the product name as tCard in prose, including the document's own header
- [x] Cut the inline comments that narrate, keeping the whys
- [x] Verify the suite, clippy, rustdoc and both feature builds
