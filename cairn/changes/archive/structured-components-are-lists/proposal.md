---
cairn: change
id: structured-components-are-lists
status: landed
created: 2026-08-31
---

# A structured component holds a list, and an untouched one keeps its bytes

## Why

RFC 6350 section 6.2.2 says of `N` that "the text components are separated by the SEMICOLON character" and that "individual text components can include multiple text values separated by the COMMA character". Its own example carries two additional names and three suffixes:

```
N:Stevenson;John;Philip,Paul;Dr.;Jr.,M.D.,A.C.P.
```

tCard models every component as one string, so that comma is read as literal text. On the way back the whole component is escaped, and the card comes out changed:

```
before:  N:Stevenson;John;Philip,Paul;Dr.;Jr.,M.D.,A.C.P.
after:   N:Stevenson;Jon;Philip\,Paul;Dr.;Jr.\,M.D.\,A.C.P.
```

Two additional names became one name containing a comma, and three suffixes became one. Nothing in the form was touched but `given`: because `N` is a single line, changing any part of it re-renders all of it, and every component pays.

That is silent data loss in the one operation tCard exists to make safe, and no test caught it because the golden fixtures carry no multi-valued component.

The form cannot tell the two apart while a component is a string: a comma the person typed and a comma the card used as a separator look identical. Typing the component as a list removes the ambiguity rather than guessing at it, which is also how the form already treats `nickname`, `organization`, `categories` and `language`.

## What

**A component declares whether it holds a list.** `N`'s five all do, per section 6.2.2. `ADR`'s `street` does, section 6.3.1 naming a street with multiple lines as the case. `GENDER`'s two do not, its sex being one letter and its identity free-form text.

A list component projects as a TOML array and reads back joined on commas, each value escaped on its own:

```toml
[name]
family = ["Stevenson"]
given = ["John"]
additional = ["Philip", "Paul"]
prefixes = ["Dr."]
suffixes = ["Jr.", "M.D.", "A.C.P."]
```

A bare string is accepted where an array is expected, so `given = "John"` still applies. Reading is liberal; only what tCard writes is canonical.

**A component the document did not change keeps the card's own bytes.** This is tCard's own rule, which the line already follows, applied one level down: when the value the form holds is what the card's component already meant, the component goes back out exactly as the card wrote it. That is what saves the scalar components, so a `country` reading `Congo, The Democratic Republic of the` survives an edit to the street beside it.

**The name components get hints.** `family`, `given` and `additional` are the RFC's role names and stay, positional names being the thing that varies between cultures, but nobody guesses that `additional` means the middle names. The hint column the form already has says so.

## What this is not

`ADR`'s six other components are not lists. The RFC allows them "where it makes semantic sense", and the sense is a multi-line street; making a postal code an array would cost every reader something to buy a case nobody has. They are covered by the untouched-component rule instead.

The projection is not otherwise renumbered: a component keeps its position, a hidden one still round-trips, and a card carrying no multi-valued component projects and folds back exactly as before.
