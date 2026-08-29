---
cairn: change
id: parse-once
status: active
created: 2026-08-29
---

# Two libraries read every card, and the second one loses bytes

## Why

tcard depends on calcard and on vcard-rs at once, and every body goes through both. A merge parses the three sides with vcard-rs, runs the reconciliation, serialises the result, and calcard then parses that result again so the projection has a typed card to walk. One body, two readers, two models.

The second read is where two confirmed defects enter, and neither is fixable from here. A comma-separated list item loses exactly one space after an escape, so `CATEGORIES:a\,  b` comes back `a\, b`. A one-letter gender identity comes back uppercased, `GENDER:F;m` as `GENDER:F;M`. Both happen inside calcard 0.3.13 before the projection sees anything, and `template::project` is handed an already-parsed card rather than the source bytes, so there is nothing here to work around with. They are recorded as tcard-tcal-escape-eats-a-space and tcard-gender-identity-uppercased, each with a reproduction the suite carries ignored.

The editor exists for the same reason. Its own header says calcard is a normalising reader and writer, churning line folding, parameter casing and property order even where nothing changed, and that this module instead keeps every content line's original bytes. It then says it is calcard-independent and could move to its own crate, shared with the iCalendar sibling. That crate exists already: it is the tree layer of vcard-rs and ical-rs, maintained next door, fuzzed, and now carrying the merge as well.

Nothing calcard supplies is missing from vcard-rs, which carries the version-agnostic model, properties, parameters, values, the byte-faithful CST, jCard and JSContact.

## What

- Drop the calcard dependency and project from vcard-rs's own model.
- Retire src/edit in favour of vcard-rs's tree layer rather than maintaining a second implementation of the same idea, which is what the module header already anticipates.
- Read each body once. The merge and the projection then agree by construction instead of by round trip.
- The two calcard reproductions close because the reader that lost those bytes is gone. Un-ignore both.

## Order

After vcard-rs's current fix round is released, so the port targets the fixed merge rather than the one being replaced under it. tcard goes before tcal: the vCard model is the simpler of the two, and tcal follows the shape tcard settles on, as merge and prefer-local already did.

## What it costs, honestly

template/model.rs is written against calcard's typed model and is the real work; the rest is deletion. Before committing, confirm vcard-rs's decoded model covers every property the projection shows across vCard 2.1, 3.0 and 4.0, since calcard's coverage is what the current model was built on and a gap would surface as a silently dropped field rather than a compile error.

The golden fixture corpus is the safety net. Projection equality and byte-exact round-trip must hold across it before and after, and apple_contacts must not regain the drift the grouped-property fix removed.
