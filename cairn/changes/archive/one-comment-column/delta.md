---
cairn: change
id: one-comment-column
status: landed
created: 2026-08-31
---

# Delta

## ADDED Requirements

### Requirement: Inline comments share one column across the card
The inline `#` hints SHALL align on one column measured over the whole card, not one per block: the first tab stop past the widest hinted left side anywhere in it, so every hinted line reaches it with at least one tab.

A column per section makes the comments step in and out as the reader scrolls, each block setting its own by whatever its widest value happens to be. One column reads as one column.

The card is the unit rather than the file, which is what tCal measures per component: a long value on one card would otherwise push every other card's comments out with it. Reading a form is the same job in both crates, so they align the same way.

## MODIFIED Requirements

## REMOVED Requirements
