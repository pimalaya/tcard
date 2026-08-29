---
cairn: log
change: list-collision-as-a-choice
date: 2026-08-29
---

# An ORG collision is put to the reader

`ORG` is component-structured upstream and a list here, so a collision on it arrived as one `ValueComponentChanged` per `;`-component, and a list field has no components to address. The choice was dropped and the reader was told, in a header note, that "a part not shown here" was contested while `organization` sat two lines below in plain sight, already settled for the local value.

`list_choice` now reads the base instance's components off the base card and moves them by every component change each side made to that instance, so the three arrays are the values the sides actually wrote rather than a per-component reconstruction that would show `[C, B]` where the side wrote `[C, D]`. The field's own key is contested with them.

`decorate` tracks which (instance, key) pairs are already contested, so the second component's collision is neither a second choice nor a spurious note. That guard is general: it holds for any field the document writes once and the report reports in parts.

## Deliberately narrow

The reconstruction applies only where both sides' actions are component changes, which is every shape `ORG` produces. Anything else falls through to the note it fell through to before, rather than being reconstructed on a guess.

This is not the sibling finding about list collisions being unioned. Where both sides rewrite a `,` list or a `TYPE` wholesale, vcard-rs merges them item by item and records no conflict at all, so nothing reaches this code to render. That half is upstream and stays open.

## Verification

The reproduction runs. The forcing laws are unchanged and still hold.

Capabilities moved: `merge`.
