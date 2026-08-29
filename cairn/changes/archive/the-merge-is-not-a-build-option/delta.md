---
cairn: delta
change: the-merge-is-not-a-build-option
---

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: Merging is a verb over three files
`merge` SHALL take a base, a local and a remote card as paths, plus the path to write, run the three-way merge in process, and project the result as TOML for editing. It SHALL write the output path only once the edited document parses, and SHALL leave it untouched otherwise.

The capability SHALL be built unconditionally. vcard-rs is a plain dependency of every configuration, so gating the merge changes nothing about the crate set and a cargo feature has nothing left to buy.

Taking the three rather than a pre-merged body with markers is what keeps the document a card. Line markers are how a line-oriented merge shows an unresolved region, and a vCard is not lines: a marker in one would break every parser downstream, including this one. The merge is a pure function over bodies already at hand, so running it here rather than receiving its output costs nothing and invents no format.
