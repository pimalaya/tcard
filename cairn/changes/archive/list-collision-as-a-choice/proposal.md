---
cairn: change
id: list-collision-as-a-choice
status: landed
created: 2026-08-29
---

# A collision on a key the document writes is demoted to a comment

## Why

`ORG` projects as the bare key `organization`. When both sides rewrote it the merge did record a collision, but it reported one `;`-component at a time, and a list field has no components to address, so the choice was dropped and the collision became the catch-all header note: "organization: both sides changed a part not shown here; the local value was kept".

The note is wrong on its face. `organization` is written two lines further down, in plain sight, and the document applies with the local value rather than asking. A reader is told a part they cannot see was contested while looking straight at it, which is the one thing the forcing convention exists to prevent.

## What

- A collision the report breaks into components on a field the document writes as one array is put back together and contested as one choice on that array.
- The three sides are read off the base card and moved by every component change each side made to that instance, so the arrays are the values the sides actually wrote rather than a per-component reconstruction.
- A key the document writes once is contested once, however many of its parts the report reported, so the second component's collision is neither a second choice nor a spurious note.
- The reconstruction is deliberately narrow: it applies only where both sides' actions are component changes, and anything else still falls through to the note it fell through to before.
