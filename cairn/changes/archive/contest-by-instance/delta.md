---
cairn: delta
change: contest-by-instance
---

## ADDED Requirements

### Requirement: A contest is rendered in its own instance's block
A collision on a repeatable property SHALL be written into the block of the instance the merge report names, and SHALL fall back to a search by value only where that block cannot be trusted to be the right one.

The report indexes the instance in the base card while the document projects the merged one, and the pairing between them (by `PID`, then by equality, then by position) is not exposed, so the index is authoritative only while that pairing was positional. The block it names is therefore taken when it holds the value the merge kept, which is the local one wherever a choice is rendered at all, and the value search stands behind it for the rest.

Addressing by value alone lets an uncontested sibling carrying the same local value steal the contest, and then the reader decides one phone number and overwrites another.

#### Scenario: An uncontested sibling does not steal the contest
- GIVEN two phones where only the second collides, both reading the same number locally
- WHEN the document is projected and the remote side is kept
- THEN the second phone carries the remote number and the first is untouched
