---
cairn: change
id: header-notes-wrap
status: landed
created: 2026-08-29
---

# A header note wraps like the rest of the document

## Why

The document's own header comment is written to a column, and so are the notes tCal adds under it, at 66 including the `# ` prefix. tCard's notes are written as one line each however long they run, so a note trails off the width the rest of the document keeps and, in a narrow terminal, off the screen.

It is presentational, and it is the document a person reads. Two tools projecting the same kind of note should lay it out the same way.

## What

- Lift tCal's wrapping into tCard, at the same column and through the same `WRAP` constant, a continuation line of a bullet indented under its text.
- Assert the notes through the header rather than through a whole line, since a note long enough to wrap is no longer one line to match.
