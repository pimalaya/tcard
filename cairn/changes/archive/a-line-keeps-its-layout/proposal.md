---
cairn: change
id: a-line-keeps-its-layout
status: landed
created: 2026-08-30
---

# A card written folded came back unfolded

## Why

vcard-rs 0.3 is released, and tcard tracked its git branch to get the merge. Moving to the release is a version bump and nothing else: every type tcard names is where it was, and the whole suite passes untouched. What the release changes is underneath, and it changes what tcard hands back.

The parser now records what it resolves away. A content line carries a wire shape beside its logical bytes: where it was folded, the blank lines that stood before it, its quoted-printable soft breaks. Serialization puts all three back, so a card parsed and written out again is the card that came in, byte for byte.

That was tcard's largest known limitation, and its own header named it: a card written folded came back unfolded, the value intact and only its layout moved. Real exports fold heavily. Apple, iOS and Google fold every long line, so a reader who edited one field got a diff touching every long line in the card, which is the shape of diff that makes an editor untrustworthy for a file kept under version control or synced to a server.

The release also gives a parameter node the escaping rules of the version it was read at, which the parameter side never had. tcard builds parameter nodes by hand when it writes a line back, and stamped every one of them with the default, vCard 4.0. Nothing reads those nodes today, so nothing is visibly wrong; it is a node claiming a version its card does not declare, and it costs one argument to stop claiming it.

## What

- Depend on the released vcard-rs 0.3 and drop the git patch.
- Keep a line's wire layout where the document does not move the line, which is what the release gives for free through the byte-preserving path tcard already took.
- Say what an edited line does instead: it goes back out unfolded, since the layout is offsets into that line's own bytes and an edit that changes its length invalidates them. RFC 6350 section 3.2 recommends folding rather than requiring it.
- Stamp a line a fold-back builds, its value and its parameters alike, with the escaping rules of the version its own card declares.
- Pin the new behaviour with a crafted fixture that round trips byte-exact, folds and interior blank line included, rather than only with the imported exports that are lossy for other reasons.
