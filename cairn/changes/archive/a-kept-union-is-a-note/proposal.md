---
cairn: change
id: a-kept-union-is-a-note
status: landed
created: 2026-08-29
---

# A list both sides edited is merged silently

## Why

Where both sides rewrite a multi-valued property or a `TYPE` parameter, vcard-rs merges them item by item. Neither side changed the property as far as the merge can see: one removed some items and added others, the other did the same, and the items merge as a set, so no conflict is recorded. The merged card carries every item either side kept, the document says nothing, and the reader is never told.

Merging items as a set is right, and deliberate. It is stated in vcard-rs's merge module docs, on `Slot::Items`, and in its merge spec: RFC 6350 gives the items of a multi-valued property no order, so two sides each adding a nickname should keep both, and asking a reader to choose between them would throw one away for no reason. This is not a defect to fix in vcard-rs, and making it a conflict here would be wrong.

What is wrong is the silence. The merged value is one neither side wrote, and the reader has no way to see that it was assembled rather than chosen. The item actions are already in the report, as `ValueItemAdded` / `ValueItemRemoved` / `ParamItemAdded` / `ParamItemRemoved`; tcard's `addressed` returns `None` for them, so they reach neither a key nor the header.

## What

- Say in the header comment every list both sides edited, the way a collision the merge already settled is said, stating that the items of both were kept.
- Leave the merge itself alone: no contest, no duplicate keys, nothing for vcard-rs to change.
