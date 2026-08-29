---
cairn: delta
change: header-notes-wrap
---

## ADDED Requirements

### Requirement: A header note wraps at the document's column
A note written into the document header SHALL wrap at the same column the header itself uses, its `#` prefix included, and a continuation line SHALL be indented under the text of its bullet rather than under the bullet mark.

The header is prose a person reads before anything else, and a line running past the width the rest of the document keeps is the one part of the document that can leave the screen.

#### Scenario: A note longer than the column
- GIVEN a note whose text passes the wrapping column
- WHEN the document is projected
- THEN it is written over two comment lines, the second indented under the first line's text

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
