---
cairn: log
change: the-family-standards-settled
landed: 2026-11-15
---

# The family standards, settled

The seven decisions the-family-standards left to the owner (2026-11-14), taken on 2026-11-15 as pimalaya-70 recommended them, the first one corrected.

## What landed

- (a) A document written to a file stays silent in text and answers its receipt under `--json`; output.md already gives the composer reason, unchanged.
- (b) `cli` stays the one default feature; README.md and CONTRIBUTING.md no longer call it opt-in.
- (c) `edit` and `merge` under `--json` fail coded `usage-invalid`, exit 2, before an editor opens (src/cli/editor.rs). `editor-missing` and `editor-failed` left the code vocabulary: text output prints no code, so nothing could carry them.
- (d) `apply - -` exits 2: pimalaya-cli 3bada99 exits 2 for any failure coded `usage-invalid`, which `apply` already raised.
- (e) Folding at 75 octets stays, with its CHANGELOG line.
- (f) base64-codec's decode of non-canonical trailing bits stays, pinned by `non_canonical_trailing_bits_decode_and_are_written_canonical` (src/template/media.rs), with a CHANGELOG line.
- (g) The editor re-opens on `TcardError::is_document_refusal`, an explicit list (TOML, undecided collision, unknown digest, unreadable file the document names), where it re-opened on anything but a card that does not read. tCal takes the same rule.
- MIGRATION.md, from v0.2 to v0.3.

## Capabilities moved

- [editor](../spec/editor.md): *The editor re-opens on a document it cannot fold* made explicit; *No editor runs under JSON* added.
- [output](../spec/output.md): the two editor codes removed; *A refused command line exits 2* covers `apply - -` and an editor under JSON.
