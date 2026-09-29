---
cairn: change
id: an-embedded-photo-is-a-digest
status: landed
created: 2026-09-29
---

# Delta

## ADDED Requirements

### Requirement: An embedded photo is shown as its digest
A photo whose bytes the card carries, as a `data:` URI or under a base64 `ENCODING`, SHALL project as `embedded.type` and `embedded.sha256`, the full SHA-256 of its decoded bytes in lowercase hex, and never as those bytes. Any other photo SHALL project as `uri`, its value as the card wrote it.

Base64 is noise in a form meant to be read, and nobody edits a picture by hand. A digest names the bytes without showing them, and the full one can be checked against a file with `sha256sum`.

#### Scenario: An inline photo round trips untouched
- GIVEN a vCard 3.0 card holding `PHOTO;ENCODING=b;TYPE=JPEG:<base64>`
- WHEN it is projected
- THEN the form shows `embedded.type = "image/jpeg"` and the digest, and no base64
- AND WHEN that form is folded back
- THEN the card comes back byte-exact

### Requirement: An embedded photo comes back as its own line
A `[[photo]]` carrying `embedded` SHALL fold back as the card's own line holding the photo of that digest, byte for byte, wherever that line stood. A digest no photo of the card holds SHALL be refused, naming it.

### Requirement: A file is inlined at the card's version
A `[[photo]]` carrying `file` SHALL fold back as the bytes that file holds, inlined as a `data:` URI in vCard 4.0 and as base64 under `ENCODING=b` in 3.0 or `ENCODING=BASE64` in 2.1, its media type read from the bytes. The line it replaces SHALL keep every parameter but its encoding, value type and media type.

The library SHALL NOT read the file itself: the template SHALL carry a reader its caller provides, and a template without one SHALL refuse every `file`, as SHALL a reader failing, naming the path.

Where one table carries several of the three keys, `file` SHALL win over `uri`, and `uri` over `embedded`.

#### Scenario: A photo is replaced
- GIVEN a card holding an inline photo
- WHEN its `embedded` is swapped for `file = "~/me.jpg"`
- THEN the card holds the bytes of that file inline, and projecting it again shows them as `embedded`

### Requirement: A photo collision is settled, not offered
A photo both sides changed SHALL be settled by the merge and said in the header rather than written as a choice. The merged card holds one side's bytes, so a digest naming the other side's would not fold back.

## MODIFIED Requirements

### Requirement: The error enum is written by hand
The crate error SHALL implement Display and Error by hand rather than derive them from a dependency.

One enum of six variants is six match arms, and the crate below it writes its own the same way. A derive dependency earns its place where the enum is large or the source chaining is intricate, and this one is neither.

#### Scenario: The dependency list of the core
- GIVEN the library built with no features
- WHEN its dependencies are read
- THEN no error-derive crate is among them

## REMOVED Requirements

None.
