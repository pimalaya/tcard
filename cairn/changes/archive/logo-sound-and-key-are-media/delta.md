---
cairn: change
id: logo-sound-and-key-are-media
status: landed
created: 2026-09-29
---

# Delta

## ADDED Requirements

### Requirement: A text key is embedded
A `KEY` whose value is text rather than a URI, declared `VALUE=text` or carrying no URI scheme, SHALL project as `embedded`, its digest being that of the text's bytes, and fold back as its own line.

#### Scenario: An armoured key round trips
- GIVEN a card holding `KEY;TYPE=PGP:-----BEGIN PGP PUBLIC KEY BLOCK-----...`
- WHEN it is projected and folded back untouched
- THEN the form shows a digest and no key text, and the card comes back byte-exact

### Requirement: A legacy media type is read and written through the field's formats
A bare `TYPE` on a media line before vCard 4.0 SHALL be read as a media type through the field's formats, a name the field aliases taking its alias and any other falling under the field's top-level type. A `file` inlined before 4.0 SHALL write its media type back the same way.

#### Scenario: A WAVE sound
- GIVEN a vCard 3.0 card holding `SOUND;ENCODING=b;TYPE=WAVE:...`
- WHEN it is projected
- THEN the form shows `embedded.type = "audio/wav"`

### Requirement: The editor re-opens on a document it cannot fold
`edit` and `merge` SHALL offer to re-open the editor on every refusal to fold the document back, and on none about the card it came from.

## MODIFIED Requirements

### Requirement: A media collision is settled, not offered
A media value (`PHOTO`, `LOGO`, `SOUND`, `KEY`) both sides changed SHALL be settled by the merge and said in the header rather than written as a choice. The merged card holds one side's bytes, so a digest naming the other side's would not fold back.

### Requirement: An embedded photo is shown as its digest
A media value (`PHOTO`, `LOGO`, `SOUND`, `KEY`) whose bytes the card carries, as a `data:` URI or under a base64 `ENCODING`, SHALL project as `embedded.type` and `embedded.sha256`, the full SHA-256 of its decoded bytes in lowercase hex, and never as those bytes. Any other value SHALL project as `uri`, its value as the card wrote it.

Base64 is noise in a form meant to be read, and nobody edits a picture, a sound or a key by hand. A digest names the bytes without showing them, and the full one can be checked against a file with `sha256sum`.

#### Scenario: An inline photo round trips untouched
- GIVEN a vCard 3.0 card holding `PHOTO;ENCODING=b;TYPE=JPEG:<base64>`
- WHEN it is projected
- THEN the form shows `embedded.type = "image/jpeg"` and the digest, and no base64
- AND WHEN that form is folded back
- THEN the card comes back byte-exact

### Requirement: An embedded photo comes back as its own line
A media table carrying `embedded` SHALL fold back as the card's own line holding the value of that digest, byte for byte, wherever that line stood. A digest no line of that property holds SHALL be refused, naming it.

### Requirement: A file is inlined at the card's version
A media table carrying `file` SHALL fold back as the bytes that file holds, inlined as a `data:` URI in vCard 4.0 and as base64 under `ENCODING=b` in 3.0 or `ENCODING=BASE64` in 2.1, its media type read from the bytes. The line it replaces SHALL keep every parameter but its encoding, value type and media type.

The library SHALL NOT read the file itself: the template SHALL carry a reader its caller provides, and a template without one SHALL refuse every `file`, as SHALL a reader failing, naming the path.

Where one table carries several of the three keys, `file` SHALL win over `uri`, and `uri` over `embedded`.

#### Scenario: A photo is replaced
- GIVEN a card holding an inline photo
- WHEN its `embedded` is swapped for `file = "~/me.jpg"`
- THEN the card holds the bytes of that file inline, and projecting it again shows them as `embedded`

## REMOVED Requirements

None.
