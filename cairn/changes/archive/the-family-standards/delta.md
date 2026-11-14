---
cairn: change
id: the-family-standards
status: landed
created: 2026-11-14
---

# Delta

## ADDED Requirements

Into spec/output.md, a new capability: A document prints as itself, or as its contents under JSON; A document written to a file answers its receipt under JSON; A failure carries a stable code; A refused command line exits 2; Every verb has a schema. Their text is the one spec/output.md holds.

Into spec/api.md:

### Requirement: Base64 runs on the family codec
Media bytes SHALL be encoded and decoded through base64-codec, the codec every Pimalaya crate shares, and the binary SHALL install the detected CPU token first thing in `main`. Decoding SHALL refuse a value holding a stray byte, so such a value reads as the reference it then is, as it did under the strict decoder before.

## MODIFIED Requirements

In spec/template.md:

### Requirement: A line the document leaves alone keeps its layout
A line the document moves SHALL be written anew by vcard-rs: in vCard 3.0 and 4.0 folded at 75 octets, keeping the folds that stood before the first byte the edit changed, and in vCard 2.1 or under quoted-printable on one line. It was written unfolded. The first scenario now ends on one line while it fits in 75 octets, and a scenario for a line growing past 75 octets is added.

## REMOVED Requirements

None.
