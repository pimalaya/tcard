---
cairn: change
id: an-embedded-photo-is-a-digest
status: landed
created: 2026-09-29
---

# An embedded photo is a digest

## Why

Issue #1: a photo inlined in a card projects as its base64, kilobytes of noise in a form meant to be read, and nothing in the form can replace it short of pasting base64 back. Creating a card could take a URI, editing one could not take a picture at all.

The plan was agreed on the issue with its reporter.

## What

A `[[photo]]` holds exactly one of three keys:

- **`embedded`**, an inline photo the card already carries: `embedded.type` (its media type) and `embedded.sha256` (the full digest of its bytes, checkable with `sha256sum`). Its bytes are never shown. Coming back unchanged, the original line is kept byte for byte, whichever instance it sits at. A digest the card does not hold is an error.
- **`file`**, a local path. On apply its bytes are read and inlined, as a `data:` URI in vCard 4.0 or as `ENCODING=b` (3.0) / `ENCODING=BASE64` (2.1). The next projection shows it as `embedded`, so replacing a photo is swapping its `embedded` for a `file`.
- **`uri`**, a reference, kept as written, `file:` included.

The library does no I/O, so reading a `file` goes through a reader the caller hands the template, the CLI's being `fs::read` behind the usual path expansion. A library caller handing none has every `file` refused.

## What this is not

Not an extraction mode. Writing an embedded photo out to disk is a separate command, later.

Not LOGO, SOUND, KEY or tCal's ATTACH yet. The same shape applies to them, and each is a table entry away once the kind exists, KEY first needing an answer for its text values.
