---
cairn: delta
change: a-kept-union-is-a-note
---

## ADDED Requirements

### Requirement: A union is said in the header
*Folds into merge.md.*

Where both sides edited the items of one multi-valued property or one list parameter, the document SHALL say so in its header comment, stating that the items of both were kept. It SHALL NOT contest them.

The items of such a value merge as a set, RFC 6350 giving them no order, so both sides' additions and removals all apply and nothing collides. That is the right outcome: two sides each adding a nickname should keep both, and putting them to a reader would throw one away for no reason. The silence is what is wrong, since the merged value is then one neither side wrote and nobody was told it was assembled.

#### Scenario: Both sides rewrite a list
- GIVEN a base holding `NICKNAME:a,b`, a local holding `NICKNAME:c,d` and a remote holding `NICKNAME:e,f`
- WHEN they are merged
- THEN the card holds all four, the header says the items of both were kept, and the document applies as it stands

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
