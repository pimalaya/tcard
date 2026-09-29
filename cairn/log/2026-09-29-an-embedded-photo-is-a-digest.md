---
cairn: log
change: an-embedded-photo-is-a-digest
landed: 2026-09-29
---

# An embedded photo is a digest

Issue #1: an inline photo projected as its base64, and nothing in the form could replace it. The shape was agreed on the issue with its reporter.

**A `[[photo]]` is one of three keys** (src/template/media.rs, a new `Media` kind in model). `embedded.type` and `embedded.sha256` name inline bytes, read from a `data:` URI or from base64 under `ENCODING`, and fold back as the card's own line holding that digest, wherever it stood; an unknown digest is refused. `file` inlines a path's bytes at the card's version, a `data:` URI in 4.0 and `ENCODING=b` / `ENCODING=BASE64` in 3.0 / 2.1, the media type sniffed from the bytes and every other parameter kept. `uri` is kept as written, no longer escaped.

**The library still reads nothing.** `TcardTemplate::files` and `TcardMerged::files` carry a `TcardFiles` reader; none refuses every `file`. The CLI hands every verb one over `path_parser` and `fs::read`, and the editor re-opens on both new refusals.

**A photo collision is settled, not offered**: the merged card holds one side's bytes, so the other digest could not fold back.

**Dead code went with it**: no `Typed` field lists an empty type set any more, so the `shown` helper and the empty branch of the type line are gone.

Not done: LOGO, SOUND and KEY (a table entry each, KEY needing an answer for text values), tCal's ATTACH, and an extraction command.

Capabilities moved: template (three requirements added), merge (one added), api (error enum wording modified).

Verified: the Outlook 2.1 fixture's digest equals `sha256sum` of its decoded bytes; `file` replacing a `PREF=1` URI photo gives `PHOTO;PREF=1:data:image/jpeg;base64,...`, re-projects as `embedded` and re-applies byte-exact; a bogus digest fails naming it. Fixtures regenerated, their hint column narrower now that photo lines carry none. 75 tests green on the full feature set, both builds and clippy clean.
