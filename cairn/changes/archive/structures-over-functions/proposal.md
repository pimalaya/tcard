---
cairn: change
id: structures-over-functions
status: landed
created: 2026-08-30
---

# The library is a bag of functions, and the reader has to hold the arguments

## Why

Every entry point of the crate is a free function over positional arguments: `template::project(cards, version)`, `template::apply(original, edited)`, `merge::project(base, local, remote)`, `merge::apply(vcard, edited)`, `vcard::parse(input)`. Nothing says which of those arguments belong together, and two of them are three strings of the same type in a row, where swapping two compiles and merges the wrong way round.

vcard-rs answered the same question one version ago. Its free `merge(base, left, right)` became `VcardMerge { base, left, right }.merge()`, so a caller names its three cards rather than positioning them, and the sibling ical-rs followed. tCard is the layer directly above and still reads the old way.

The pairing is the other half of it. `template::apply` takes the original text back because the projection did not keep it, so the two halves of one round trip are two functions that a caller has to remember to give the same card twice. A type holding the cards makes that structural: the tree the form was projected from is the tree the fold-back patches, which is what the one-reader-per-body requirement asks for anyway.

The same reading applies to the two big modules. cli.rs carried three commands, their shared arguments, the editor round trip and the file writing in one file, and merge.rs carried the choice, the document, the notes and the wrapping in another, at 960 lines. Neither is navigable by the name of the thing you are looking for.

Two smaller things ride along, both of them things a reader trips over. thiserror is a dependency for one enum of four variants whose Display is four lines by hand, and vcard-rs already writes its own. And the product is tCard, while `tcard` is the identifier: the document a person edits says "edited by tcard", which is the binary's name pretending to be the product's.

## What

- Give every entry point a type that names its inputs: `Template { cards, version }`, `Merge { base, local, remote }`, `Merged`, `Cards::parse`.
- Fold `template::apply` onto the template it was projected from, so a round trip is two methods on one value rather than two functions over the same card twice.
- Split cli.rs into one module per verb over shared argument and editor modules, and merge.rs into choice, document and note under a facade.
- Drop thiserror and write Display and Error by hand.
- Write the product name as tCard everywhere it is prose, keeping `tcard` for the crate, the binary and the module.
- Cut the inline comments that narrate what the code says, keeping the ones that carry a why nothing else records.
