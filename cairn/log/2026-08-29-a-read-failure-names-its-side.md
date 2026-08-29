---
cairn: log
change: a-read-failure-names-its-side
landed: 2026-08-29
---

# A read failure names the side it came from

A merge reads three files, and any of them can fail to parse. All three went through one `parse` that answered `Cannot parse vCard: ...`, so a person was told the merge failed without being told which of their three paths to open.

## What landed

`TcardError::ReadCard { side, message }` carries the side the unreadable card was given as, base, local or remote, beside what vcard-rs made of it, and `merge::read` reads each of the three through it. `ParseVcard` stays for the reader every other verb goes through, where there is one body and no side to name.

The variant is tCal's `ReadCalendar` with the domain word swapped: the same two fields under the same names, the same message shape, and the same division of labour beside a whole-body parse error. The two tools now refuse an unreadable merge the same way.

## Verification

`merge::tests::an_unreadable_side_is_named` projects a merge whose remote card is not a vCard and asserts the refusal names the remote side. Both configurations build, clippy is clean, `cargo fmt` run, the whole suite green.

Capabilities moved: merge (ADDED: A read failure names the side it came from).
