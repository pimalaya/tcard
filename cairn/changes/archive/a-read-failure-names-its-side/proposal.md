---
cairn: change
id: a-read-failure-names-its-side
status: landed
created: 2026-08-29
---

# A read failure names the side it came from

## Why

A merge reads three files, and any of them can fail to parse. `merge::parse` maps all three onto `ParseVcard`, which carries only what vcard-rs said, so a person is told the merge failed without being told which of the three files to open. The paths are theirs and only they can tell base from local from remote, so naming the side is the whole difference between a report and a puzzle.

tCal already names it. The same three-file verb answers `Cannot read the base calendar: ...` there and `Cannot parse vCard: ...` here, and the two tools should meet a reader the same way.

## What

- Add `ReadCard { side, message }`, carrying the side the unreadable card was given as and what vcard-rs made of it, with tCal's field names.
- Have the merge read through it, one call per side.
- Leave `ParseVcard` to the reader every other verb goes through, where there is one body and no side to name, exactly as tCal keeps `ParseICalendar` beside `ReadCalendar`.
