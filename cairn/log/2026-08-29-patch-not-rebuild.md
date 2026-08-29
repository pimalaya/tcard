---
cairn: log
change: patch-not-rebuild
date: 2026-08-29
---

# A modelled line is patched rather than rebuilt

`Field::content_lines` now takes the card's own lines for the property, in the order the projection showed them, and builds each line on top of the one it came from. A new `template::patch` module takes a line apart (name, parameters, value) with quoted parameter values respected, so `ADR;GEO="geo:1,2"` splits at the right colon, and puts the prefix back with only the `TYPE` the document shows replaced.

Three findings shared that one mechanism, so they landed together rather than as three passes over the same signature.

## Parameters

Everything but `TYPE` is the line's own. `TYPE` itself is compared against what the line already carries, case-insensitively and through every spelling in the wild: `TYPE="work,voice"`, one `type=` parameter per value, and the bare vCard 2.1 `;WORK`. Where the two agree the prefix is returned untouched, which is what lets a real export round-trip byte for byte instead of being renormalised into a canonical spelling nobody asked for. Where they differ the type parameters are dropped and one `;TYPE=` is appended, every other parameter keeping its place and its order.

## Hidden components

`read_components` fills a component the document does not write from the line it came from, positionally. That covers `pobox` and `ext` in vCard 4.0, which the form hides on purpose; hiding is now a way to discourage writing one rather than a way to lose one. The property generator was widened to produce both, so the round-trip law now exercises them on every case rather than stepping around them.

## Repeated properties

A list field's items are spread over the lines they came from, each keeping as many as it held, and a surplus item opens a line of its own. `LANG;PREF=1:fr` beside `LANG;PREF=2:en` therefore stays two lines with their preferences, rather than collapsing to `LANG:fr,en` and then, on the next pass, to the single nonsense value `fr\,en`.

A `;` list is one property whose components are ordered, so it stays one line and an empty component keeps its place; `ORG:Apple Inc.;` survives as written, where before the trailing empty was filtered out and the trailing separator vanished. A `,` list is items, where an empty one says nothing and is still dropped.

## Verification

Four reproductions run: unshown parameters survive, a post office box survives vCard 4.0, two `LANG` properties stay two and settle, and every fixture settles. apple_contacts now differs from its source in one line only, `N`'s trailing empty components, which is a separate normalisation and keeps its lossy marker.

Capabilities moved: `template`.
