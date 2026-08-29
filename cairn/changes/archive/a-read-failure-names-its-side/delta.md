---
cairn: delta
change: a-read-failure-names-its-side
---

## ADDED Requirements

### Requirement: A read failure names the side it came from
Where one of a merge's three cards does not parse, the refusal SHALL name the side it was given as, beside what the reader made of it.

A merge is the one verb reading more than one body, and its three paths are the user's. A refusal naming none of them says only that the merge failed, leaving the reader to open all three to find out which.

#### Scenario: An unreadable remote card
- GIVEN a merge whose remote card is not a vCard
- WHEN it is projected
- THEN it is refused, naming the remote side

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
