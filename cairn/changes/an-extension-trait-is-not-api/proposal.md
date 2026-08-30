---
cairn: change
id: an-extension-trait-is-not-api
status: landed
created: 2026-08-30
---

# A trait with one implementation is published as if it had two

## Why

`TcardCard` is a public trait with exactly one implementation, `VcardCst`, three call sites, all of them in template.rs, and no generic function taking it. A reader meeting it in the crate documentation has to work out what the second implementation is meant to be, and there is none.

What it actually is, is an extension trait: `VcardCst` belongs to vcard-rs, Rust gives a foreign type no inherent method, and a trait is the only way to write `card.lines(name)` rather than `lines(card, name)`. That is a mechanism the crate uses on itself, not a contract it offers anyone.

The sibling case tells them apart. tCal's `TcalContainer` has two implementations, a calendar and a component, and `reconcile` is generic over it, so a caller can hand it either. That trait carries dispatch and belongs in the open. This one carries none.

## What

- Make the trait crate-private and drop the prefix with the visibility, the naming rule applying to what the library ships.
- Say in the module header what the trait is for, so the next reader does not have to rediscover that a foreign type takes no inherent method.
- Keep the public surface to what a consumer actually calls: parse a stream, project it, fold a document back, merge three.
