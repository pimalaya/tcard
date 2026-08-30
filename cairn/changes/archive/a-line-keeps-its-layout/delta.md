---
cairn: delta
change: a-line-keeps-its-layout
---

## ADDED Requirements

### Requirement: A line the document leaves alone keeps its layout
*Folds into template.md.*

A line the document does not move SHALL come back with the layout it was written in: its folds, the blank lines that stood before it and its quoted-printable soft breaks. A line the document moves SHALL be written unfolded.

A layout is offsets into the line's own bytes, so an edit that changes the line's length invalidates them, and a fold put back at the wrong offset is worse than no fold at all. RFC 6350 section 3.2 recommends folding at 75 octets rather than requiring it, so writing an edited line whole is conformant.

The asymmetry is what a reader wants either way. Real exports fold every long line, so rewriting the layout of a card touched in one field yields a diff across the whole card, and an editor whose diff is that much larger than the edit is one nobody trusts over a synced or versioned file.

#### Scenario: A folded card is edited in one field
- GIVEN a card whose note is folded across two physical lines, with a blank line between two of its properties
- WHEN an untouched projection is folded back
- THEN the card comes back byte-exact, its fold and its blank line included
- AND WHEN the note is edited
- THEN only the note is rewritten, and it goes out on one line

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
