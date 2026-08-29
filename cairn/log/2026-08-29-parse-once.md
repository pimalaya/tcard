---
cairn: log
change: parse-once
date: 2026-08-29
---

# Two libraries read every card, and the second one lost bytes

Every body went through both vcard-rs and calcard: the merge reconciled a syntax tree, serialised it, and calcard parsed the result again so the projection had a typed card to walk. One body, two readers, two models, and the second one normalised what it read. `CATEGORIES:a\,  b` came back `a\, b`, `GENDER:F;m` came back `GENDER:F;M`, and the loss happened before `template::project` saw anything, which was handed a parsed card rather than the source bytes.

## What landed

- **calcard is gone.** `crate::vcard` is the one reader: `parse` reads a whole stream into vcard-rs's `VcardCst`, and the projection, the merge and the fold-back all walk that same tree. The dependency is dropped and `merge` no longer pulls a second vCard library in, so the feature now gates only the reconciliation.

- **src/edit is gone with it**, 789 lines of in-house format-preserving editor replaced by vcard-rs's tree, which is the same design maintained next door and now fuzzed. What is left is 242 lines in src/vcard.rs: the stream type, the two edits a fold-back makes (`Card::set_lines`, `Cards::set_count`) and the line building behind them. An unchanged line is left where it stands, so its parameter casing, its group and its ending are the source's own; a line the document moved is written anew.

- **A file's other cards survive.** vcard-rs's single-card `parse` stops at the first `END` and documents that it does, so `crate::vcard::parse` reads with `parse_many`, which is what the retired `parse_all` was for. A bare RFC 2425 record, which has no envelope for `parse_many` to split on, still goes through the single-card reader.

- **A merged card is projected as the merge left it.** `merge::project` no longer serialises the merged tree and reads it back; the tree goes straight to the projection, and its three inputs are the trees the merge was given.

- **A line is read through the grammar that patches it.** `template/patch.rs` already owned the content-line grammar for the fold-back: where the head ends, which parameters are types, how an escaped separator binds. The projection now reads through it too, so what the form shows and what a fold-back writes agree by construction. vcard-rs's own `value_colon` is quote-aware since `quoted-parameter-values`, so the logical-line workaround tcal needed was not needed here.

## What the model swap cost

The typed model calcard supplied was only ever used to re-render a value the card had already written, so reading the raw value is both simpler and more faithful. Seven of the ten golden projections changed, and every change but one is a value calcard had been losing:

- `BDAY;VALUE=DATE:1963-09-21` projected `"1963-09"`, the day dropped on read. It now projects the native `1963-09-21`.
- `TZ:-0500` projected `""`, the typed value having no text accessor. It now projects `"-0500"`.
- `N:Perreault;Simon;;;ing. jr,M.Sc.` projected `suffixes = "ing. jr"`, and `N:Doe;John;Richter,James;...` projected `additional = "Richter"`: an unescaped comma inside a component truncated it. The whole component is now shown, and the ARCHITECTURE limitation saying otherwise is retired.
- `EMAIL;PREF;INTERNET` projected `type = ""` while the fold-back, reading the same line through `patch`, would have rewritten it to a bare `EMAIL` and dropped both. Both now show and both survive.
- `PHOTO;TYPE=JPEG;ENCODING=BASE64` projected `value = ""`, so the fold-back deleted the photo. The value is now shown as written, which is long but true; the same pass stopped a property whose projection lists no type at all from having its own types cleared by a document that never showed them.

The one deliberate change is casing: a `TYPE` is now shown as the card spelled it (`TYPE=HOME` shows `HOME`) rather than lowercased against calcard's known vocabulary, which lowercased `HOME` and left `INTERNET` alone in the same value. Matching is case-insensitive on the way back, so an untouched form still leaves the line's bytes alone.

One thing did get worse. vcard-rs unfolds a line on parse and keeps no record of where the folds were, so a card written with folded lines comes back unfolded. No fixture changed side over it, every affected one being `.lossy` already, but it is a real regression against the retired editor: the sibling ical-rs keeps a wire shape for exactly this and vcard-rs does not yet. It is recorded under the ARCHITECTURE known limitations.

## Verification

- The whole suite green: 28 lib, 14 merge forcing (2 still ignored, addressed by the list-union note), 10 projection laws, the fixture database, the doctest. All four configurations build (`--all-features`, `--no-default-features`, `--no-default-features --features merge`, still `no_std`), `clippy --all-features --all-targets` is clean, `cargo fmt` run.
- `an_escape_in_a_list_item_keeps_the_space_behind_it` and `a_one_letter_gender_identity_keeps_its_case` are no longer ignored, and the projection generators dropped the two filters that kept escapes out of list items and one-letter identities out of `GENDER`, so every law now runs on them.
- The `.lossy` set is unchanged: the same eight fixtures are lossy, for the same reasons plus folding, and `clean` and `two_cards` still round-trip byte for byte.

Capabilities moved: template (ADDED: One reader per body; MODIFIED: A projection settles at once); merge (prose only, the retired editor no longer being a place rules live).
