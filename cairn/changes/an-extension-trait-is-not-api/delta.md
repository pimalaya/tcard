---
cairn: delta
change: an-extension-trait-is-not-api
---

## ADDED Requirements

### Requirement: An extension trait is a mechanism, not API
*Folds into api.md.*

A trait with one implementation, no generic use and no dispatch SHALL be crate-private. A trait a generic function takes, or that two types implement, SHALL be public and prefixed like the rest.

Such a trait exists because Rust gives a foreign type no inherent method, so calling `card.lines(name)` rather than `lines(card, name)` costs a trait. That is the crate reaching into a type it does not own, not an extension point it offers, and publishing it asks a reader to look for a second implementation that was never coming.

The test is dispatch, not the shape of the declaration: the same keyword covers both, and only the call sites say which one this is.

#### Scenario: A trait over a foreign syntax node
- GIVEN a trait implemented once, for a type another crate owns
- WHEN the public surface is read
- THEN the trait is not in it, and the methods it carries are reached through the types that are

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
