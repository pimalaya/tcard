---
cairn: delta
change: list-collision-as-a-choice
---

## MODIFIED Requirements

### Requirement: Only a genuine choice is rendered as one
A report entry the merge already decided SHALL be a header comment, not duplicate keys. A removal against an update is decided, the update winning whichever side it came from, so there is nothing to choose and one of the two candidates could not be written as a line in any case.

An instance matched by position rather than by `PID` SHALL be said in the header too. Two sides each editing what they think of as the second phone number can be paired into one collision by position alone, and the projection cannot tell: the choice still renders as one key, but the pairing behind it may be the wrong one, and only the reader can see that.

A collision on a key the document does write SHALL NOT be demoted to a header note. Where the report breaks such a collision into parts the projection has no key for, the parts are put back together and the field's own key is contested whole, and a key the document writes once is contested once however many parts the report reported. The catch-all note says a part not shown here was contested, which is a lie about a field written two lines further down, and it settles for the local value where a reader was available to decide.

#### Scenario: A decided report line is not a choice
- GIVEN a merge where one side removed a property the other changed
- WHEN the document is projected
- THEN the surviving value is written once and the removal is said in a comment

#### Scenario: A collision on a projected key is offered as a choice
- GIVEN two sides rewriting `ORG` wholesale
- WHEN the document is projected
- THEN `organization` is written once per side, as an array, and the document refuses to apply
