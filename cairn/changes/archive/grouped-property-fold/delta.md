---
cairn: delta
change: grouped-property-fold
---

## ADDED Requirements

### Requirement: A property is addressed by its bare name
The editor SHALL match a property by the name behind its group (`item1.EMAIL` is an `EMAIL`), and a line written back SHALL keep the group of the line it replaces.

A group is a label on a line, not part of what the property is: RFC 6350 section 3.3 lets any property carry one, and Apple, iOS and Google exports label addresses and URLs that way as a matter of course. Matching the whole prefix instead makes a grouped property invisible to the fold back, which then appends a group-less copy beside it and grows the card by one line per round trip.

#### Scenario: A grouped property is rewritten in place
- GIVEN a card holding `item1.EMAIL` and its `item1.X-ABLabel`
- WHEN an untouched projection is folded back
- THEN the card is unchanged, with one `EMAIL` line still carrying its group

### Requirement: A projection settles at once
Folding an untouched projection back SHALL leave the card as it was, and folding the result again SHALL change nothing further, for every card in the golden fixture set.

A card that only settles after several passes is a card that moves under the reader, and one that never settles loses or gains something on every pass. The fixtures are real exports, so they are where a normalisation nobody intended shows up first.

#### Scenario: A real export does not grow
- GIVEN a golden fixture carrying grouped properties
- WHEN it is folded back twice
- THEN the second fold changes nothing the first did not
