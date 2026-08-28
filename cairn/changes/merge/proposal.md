---
cairn: change
id: merge
status: landed
created: 2026-08-28
---

# A collision has nowhere to be shown

## Why

vcard-rs merges two divergent cards against their common base and reports what collided. Something has to put that report in front of a person, and tCard is already the thing that puts a vCard in front of a person: the TOML projection exists so that a card can be read and edited by someone who does not want to think about folded lines and semicolon-separated components.

A sync tool is the immediate caller. It merges in the background, resolves everything the two sides did not both touch, and is then holding one card, one base, one remote body and a short list of fields where the two sides disagree. It must not decide those itself and it must not open an editor from an unattended run, so it hands the three bodies to a program and takes back one. tCard is the natural program, and the piece it lacks is small: it already renders and parses the document, and the merge is a function it already links.

The interesting question is how an unresolved collision looks in TOML, because the obvious answers are all quietly lossy. Commenting the alternatives out leaves the field absent from the document, and absence is already how a user deletes a property, so an ignored conflict silently drops a field and looks exactly like an intended deletion. A separate block listing the candidates is unambiguous but needs a rule that something has to enforce.

TOML enforces one already: duplicate keys are a parse error. Writing the same key once per surviving side makes an unresolved document one that cannot be applied at all, with no vocabulary to invent and no rule to police, and resolution is deleting the lines you do not want.

## What

- A `merge` verb taking base, local and remote paths plus an output path, running the merge in process and projecting the result.
- A collision rendered as duplicate keys, one live line per side, the ancestor above them as a comment so reverting stays possible and never accidental.
- Only a genuine choice rendered that way. A removal against an update is already decided by the merge, the update winning, and has nothing to pick between; it is said in a comment instead.
- A collision inside a structured value rendered as duplicate keys within the one table, never as a repeated array-of-tables block, which is legal TOML and would silently make a second address rather than an error. The decomposition is the merge report's already: it names which component of an `ADR` or `N` moved, and which parameter changed, so every choice lands on exactly one projected key.
- The duplicate-key parse error caught and reported as the field left undecided, rather than as a syntax error, reusing the reprompt loop the editor path already has.
