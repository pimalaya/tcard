---
cairn: change
id: logo-sound-and-key-are-media
status: landed
created: 2026-09-29
---

# LOGO, SOUND and KEY are media

## Why

RFC 6350 gives `LOGO`, `SOUND` and `KEY` the value shape `PHOTO` has: a URI in 4.0, a `data:` one when the bytes are inline, and base64 under `ENCODING` or a `VALUE=uri` reference before. `an-embedded-photo-is-a-digest` built the `Media` kind for `PHOTO` alone, so the other three stay unmodelled: kept verbatim, but neither shown nor editable.

## What

**Three fields on the media kind**: `logo`, `sound` and `key`, each one of `embedded`, `file` or `uri` exactly as `photo` is.

**A media field names its formats.** The kind carries the top-level media type a bare legacy `TYPE` belongs to (`image`, `audio`, `application`) and the legacy names that do not spell theirs (`WAVE` is `audio/wav`, `PGP` is `application/pgp-keys`, `X509` is `application/pkix-cert`), both read and written through it. Sniffing a file learns audio and key formats.

**A text key is embedded too.** `KEY` may hold text rather than a URI (`VALUE=text` in 4.0, an armoured PGP block before), which fits none of the three keys. Its text is shown as the digest of its bytes, as inline bytes are: nobody edits a key by hand either.

**The editor re-opens on everything but a card that does not read**, rather than on a list that grows with every refusal.

## What this is not

Not tCal's `ATTACH`, and not an extraction command.
