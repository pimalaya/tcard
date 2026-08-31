---
cairn: log
change: one-comment-column
landed: 2026-08-31
---

# The inline comments share one column across a card

The hints aligned per block: the bare keys picked a column from their widest line, `[name]` picked another, `[[email]]` another. Scrolling the form, the comments stepped in and out at every section, each block having set its own by whatever value it happened to hold.

They now align on one column measured across the whole card, the file holding several being measured one card at a time so a long value on one does not push the others out. tCal already did this for a component, with a NOTE saying so, so this is the twins converging rather than a new idea: reading a form is the same job in both.

**The column moved out of the block** (template/line.rs). `Lines::column` was a private method the block used on itself; it is now a free `column` over any iterator of lines, and `Lines::emit` takes the column rather than computing one. That is tCal's shape, `comment_column` and `emit_lines`, arrived at from the other side.

**The projection collects before it writes** (template.rs). `project_card` returns its blocks instead of pushing them into the output, and one `emit` measures across the lot and writes them with a blank line between. It could not have been done any other way: a column over a whole card is not knowable until every block is built.

That moved the `[[card]]` header into the block that follows it rather than leaving it a block of its own, the blank line belonging before a header rather than after it. The multi-card layout is unchanged, which the two_cards fixture pins.

Verified: 33 unit tests, the merge suite and the fixtures green, the eleven golden fixtures regenerated for the new column. `hints_are_tab_aligned` now expands the tabs and asserts every `#` lands at the same offset, across the bare keys and every section, where before it only asserted each was tab-padded.

Spec updated: `template` (ADDED: "Inline comments share one column across the document").
