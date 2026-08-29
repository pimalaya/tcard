---
cairn: log
change: date-collision-as-a-date
date: 2026-08-29
---

# A contested date reads as a date

`value_rhs` sent every non-list value through the string renderer, so a contested `BDAY` read `birthday = "19970415"` where the projection writes `birthday = 1996-04-15` two documents out of three. Nothing was lost by it, since `apply` reads either spelling back, but the reader was asked to decide a key in a syntax the document uses nowhere else, at the one moment the projection's whole job is to be legible.

`datetime::toml_datetime` reads an RFC 6350 basic date-time back into a native TOML value, the mirror of `toml_date_value` on the way out, and `Kind::Date` now renders through it. A partial value (yearless `--0203`, year only) has no native TOML form and still falls back to the quoted string, exactly as the projection does. A non-UTC offset is dropped rather than refused, which is what projecting the same value off a card already did, so the contested line and the untouched line agree.

The line search benefits too: a contested date used to fall through its value match and onto the fallback path every time, because the projected line and the choice never spelled the value the same way.

## Verification

The reproduction runs. The forcing laws still hold at 2000 cases.

Capabilities moved: `merge`.
