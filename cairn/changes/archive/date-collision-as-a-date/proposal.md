---
cairn: change
id: date-collision-as-a-date
status: landed
created: 2026-08-29
---

# A contested date is written in a syntax the projection never uses

## Why

A date field projects as a native TOML date (`birthday = 1996-04-15`), but a collision on one was rendered as the quoted RFC 6350 string the card carries (`birthday = "19970415"`). Every behavioural law still held, since both spellings fold back to the same card, but the reader was asked to decide a key in a syntax the document uses nowhere else, at the one moment the projection's job is to be legible.

It also cost the line search its value match, since the projected line reads `1996-04-15` and the choice read `"19970415"`, so a contested date always took the fallback path rather than the addressed one.

## What

- A date is contested in the spelling the projection writes it in: native where the value is complete, the quoted RFC 6350 string where it is partial (yearless, year only) and TOML has no form for it.
- Reading an RFC 6350 basic date-time back into a native value drops a non-UTC offset, which is what projecting the same value off a card already does.
