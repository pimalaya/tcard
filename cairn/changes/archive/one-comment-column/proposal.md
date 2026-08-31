---
cairn: change
id: one-comment-column
status: landed
created: 2026-08-31
---

# The inline comments share one column across a card

## Why

The hints aligned per block, so the bare keys picked one column, `[name]` picked another and `[[email]]` another. Scrolling the form, the comments step in and out at every section for no reason a reader can see: each block set its own column by whatever its widest value happened to be.

tCal already aligns a whole component on one column, and says so in a NOTE. Reading a form is the same job in both, so the two should agree.

## What

One column, measured over every line of a card, still the first tab stop past the widest hinted left side. The card is the unit rather than the file, matching what tCal measures per component.

`Lines::column` becomes a free `column` over any iterator of lines, and `Lines::emit` takes the column rather than computing its own, which is the shape tCal reached from the other side. `project_card` returns its blocks instead of writing them, and one `emit` measures across the lot: a column over a whole card is not knowable until every block is built.

The `[[card]]` header joins the block below it rather than standing alone, the blank line belonging before a header rather than after one. The multi-card layout is unchanged.
