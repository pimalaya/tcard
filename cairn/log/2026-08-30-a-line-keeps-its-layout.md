---
cairn: log
change: a-line-keeps-its-layout
date: 2026-08-30
---

# A card written folded comes back folded

tcard now depends on the released vcard-rs 0.3 rather than its git branch, and the git patch is gone from Cargo.toml. The bump needed no code: every type tcard names sits where it sat, and the suite passed before anything else was touched. What moved is underneath.

## The layout

vcard-rs 0.3 records what its tokeniser resolves away, as offsets into the line's logical bytes: its folds, the blank lines before it, its quoted-printable soft breaks. Serialization puts them back, and drops the whole shape rather than misapplying it once an edit changes the line's length.

tcard already took the byte-preserving path, keeping a line the document did not move rather than rebuilding it, so a kept line now keeps its layout with its bytes. The limitation the src/lib.rs header carried, that a card written folded came back unfolded, is retired and replaced by what is true instead: only a line the document moved goes out unfolded.

ezvcard_evolution shows the size of it. Seventeen folded lines, and the fold-back now differs from the source in the one line the date normalisation rewrites, where it used to rewrite every one of them.

## The escaping rules

0.3 gives a parameter node the escaping rules of the version it was read at, which the parameter side never carried. `built` takes an escaper and stamps it on both the value node and every parameter, taken from the card's own `VERSION` rather than from the default, so a 2.1 or 3.0 card no longer holds nodes claiming 4.0. A parameter's values are split by vcard-rs's own parser now, a comma inside a quoted value not counting, which retires the split tcard kept for it.

Nothing reads those nodes on the way out, since tcard writes a line through its own content-line grammar, so this fixes no visible defect. It stops the tree from saying something about the card that the card does not say.

## Verification

A crafted fixture, tests/data/folded.vcf, carries a folded note and a blank line between two properties, and round trips byte-exact with no lossy marker. A projection law asserts both halves on one card: untouched, it comes back with its fold; with the note edited, the note is rewritten on a single line. The eight imported exports keep their lossy markers, the reasons being the ones they already had, trailing empty components, date normalisation and escape spelling.

Capabilities moved: `template`.
