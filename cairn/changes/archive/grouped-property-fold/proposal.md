---
cairn: change
id: grouped-property-fold
status: landed
created: 2026-08-29
---

# A grouped property is written back twice

## Why

A modelled property carried by a group (`item1.EMAIL`, `item2.URL`) is projected like any other, but folding the untouched document back wrote it out a second time without its group and left the grouped original where it was. One round trip added a copy of every grouped modelled property, the next added another, without bound: apple_contacts.vcf went from 15 lines to 17, 19, 21.

Apple, iOS and Google exports use groups routinely (`item1.ADR` beside `item1.X-ABADR`), so the corpus tCard exists to edit is the corpus that grew. It also broke the foundation every other law rests on, that folding an untouched projection changes nothing, and the golden fixture that would have caught it was flagged lossy, so nothing ever folded it back.

The editor was matching a property by the whole prefix a line opens with, so `EMAIL` never matched `item1.EMAIL`, the property looked absent, and a fresh line was appended.

## What

- The editor reads a line's group apart from its name, so a property is matched by the bare name vCard actually names it by.
- The group belongs to the line rather than to the caller: a line written back carries the group of the line it replaces, and reading a line back gives it without its group, so the two are symmetric.
- The golden fixtures are folded back repeatedly and must settle, which is what would have caught the growth.
