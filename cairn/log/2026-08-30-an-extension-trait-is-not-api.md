---
cairn: log
change: an-extension-trait-is-not-api
date: 2026-08-30
---

# A trait with one implementation is not a contract

`TcardCard` is `pub(crate) trait Card` again, prefix dropped with the visibility, since the naming rule applies to what the library ships. Its two methods are unchanged, and so are its three call sites in template.rs.

It was published as if a caller might implement it. Nothing does but `VcardCst`, no function is generic over it, and a reader meeting a one-implementation trait in the documentation spends the effort of looking for the second one. What it really is, is the only way to hang a method on a type vcard-rs owns, and the module header now says so rather than leaving it to be rediscovered.

The public surface is now what a consumer calls and nothing else: `TcardCards` to read a stream, `TcardTemplate` to project one and fold a document back, `TcardMerge` and `TcardMerged` to reconcile three, `TcardError` and `TcardResult` to handle a refusal.

## The case that is not this one

tCal's `TcalContainer` looks the same and is not: a calendar and a component both implement it, and `reconcile` is generic over it, so a caller genuinely hands it either. That trait keeps its place and its prefix. `TcalComponent` and `TcalProp` sit closer to the tCard case and are worth the same look, which is a separate change in that repository rather than a silent one here.

## Verification

The suite is green unchanged, 56 tests, plus clippy and rustdoc with no link left pointing into a private item.

Capabilities moved: `api`.
