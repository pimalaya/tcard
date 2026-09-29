---
cairn: log
change: logo-sound-and-key-are-media
landed: 2026-09-29
---

# LOGO, SOUND and KEY are media

`an-embedded-photo-is-a-digest` built the media kind for `PHOTO` alone. RFC 6350 gives `LOGO`, `SOUND` and `KEY` the same value shape, so they join it the same day as `logo`, `sound` and `key`, each one of `embedded`, `file` or `uri`.

**A media field names its formats** (`MediaFormat` in src/template/media.rs, carried by `Kind::Media`): the top-level type a bare legacy `TYPE` falls under, and the names that do not spell theirs (`WAVE`, `PCM`, `AIFF`, `PGP`, `X509`). Legacy types are read and written through it, where the photo code had assumed `image/`. Sniffing learnt WAV, Ogg, FLAC, MP3, OpenPGP (armoured and binary) and DER certificates.

**A text key is embedded**: `VALUE=text`, or a value with no URI scheme, which is how older exports write an armoured block. Its digest is that of the unescaped text.

**The editor re-opens on every refusal but a card that does not read**, rather than on a list grown by each new one.

Capabilities moved: template (two requirements added, three generalised from photos to media), merge (the collision rule generalised), editor (one added).

Verified: every fixture gains empty `logo`, `sound` and `key` sections and nothing else, the RFC 6350 author card's `KEY` showing as its `uri`; an armoured key and a `TYPE=WAVE` sound round trip byte-exact; an `.asc` file inlined into a 3.0 card gives `KEY;ENCODING=b;TYPE=PGP:...` and re-projects with the digest `sha256sum` prints. 78 tests green on the full feature set, both builds and clippy clean.
