---
cairn: log
change: a-kept-union-is-a-note
date: 2026-08-29
---

# A list both sides edited is merged silently

Where both sides rewrote a multi-valued property or a `TYPE`, the merged card came out carrying every item either side kept, and the document said nothing. `NICKNAME:a,b` against `c,d` and `e,f` merged to `c,d,e,f`, a value neither side wrote; `TEL;TYPE=home` against `work` and `cell` came out `TYPE=work,cell`, a number that is now both a work line and a mobile.

## What landed

- **The union is a header note.** `note_unions` reads the item actions of both sides out of the report and, where they name the same list on the same instance, says so: `nickname: both sides changed its list; the items of both were kept`, or `phone: both sides changed its TYPE; the values of both were kept`. It is the same header a settled collision and a positional pairing already use, so the reader meets it where they already look.

- **The merge itself is untouched.** The items still merge as a set, no contest is written, and the document still applies as it stands. There is nothing to choose: RFC 6350 gives the items of a multi-valued property no order, so both sides' additions and removals all apply, and putting them to a reader would throw one of two nicknames away for no reason. That is documented, deliberate vcard-rs behaviour, stated in its merge module docs, on `Slot::Items` and in its merge spec, and it was right.

- **The finding was wrong about the remedy** and is corrected: the union is not a defect and neither vcard-rs nor tcard's `addressed` needed changing. What was missing is the note, which is now there.

## Verification

The two reproductions are no longer ignored: `a_list_union_is_said_in_the_header` and `a_type_union_is_said_in_the_header` assert the merged value, the header line, and that the document applies. The merge forcing suite is 14 tests with none ignored. All four configurations build, clippy is clean, `cargo fmt` run.

Capabilities moved: merge (ADDED: A union is said in the header).
