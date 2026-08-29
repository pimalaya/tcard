---
cairn: delta
change: parse-once
---

## ADDED Requirements

### Requirement: One reader per body
*Folds into template.md.*

A body SHALL be read once. The reader that parses a card for the merge SHALL be the reader that parses it for the projection, so the two agree by construction rather than by serialising between them.

Two readers do not merely cost a parse. They disagree, and the disagreement is invisible: a value the first reads faithfully and the second normalises reaches the document already changed, and no test comparing the document against the second reader's output can see it.

#### Scenario: A value no reader normalises
- GIVEN a card whose list item carries an escape, and whose gender identity is a lowercase letter
- WHEN it is projected and applied unchanged
- THEN both come back byte-exact

## MODIFIED Requirements

### Requirement: Projection round trip
Unchanged in what it requires: applying an unedited projection SHALL yield the card it came from. What changes is that it now holds for values the previous reader altered on the way in, escapes inside comma-separated list items and single-letter gender identities among them, which the corpus laws exercised only behind a filter.

## REMOVED Requirements

None.
