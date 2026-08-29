---
cairn: delta
change: date-collision-as-a-date
---

## ADDED Requirements

### Requirement: A collision is written in the projection's own spelling
The values of a collision SHALL be rendered as the projection renders that field: an array for a list field, a native date for a date field where the value is complete, a quoted string elsewhere.

A document that contests a key in a syntax it uses nowhere else asks the reader to learn a second spelling at the moment they are least able to, and a reader editing a contested line and an untouched one should be editing the same thing. The two spellings fold back to the same card, so this costs nothing and buys the legibility the projection exists for.

#### Scenario: A contested date reads as a date
- GIVEN two sides setting a different `BDAY`
- WHEN the document is projected
- THEN each side's line reads `birthday = 1997-04-15`, not a quoted basic string
