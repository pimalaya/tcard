---
cairn: spec
capability: merge
status: current
---

# Merge

Reconciling two divergent cards is a verb of its own beside template and edit, because the thing tCard is for is putting a vCard in front of a person, and a merge is exactly the moment where a card needs a person. The rules here are about what the document says and what it refuses; the projection it is written in belongs to template, and the byte-preserving fold back belongs to edit.

### Requirement: Merging is a verb over three files
`merge` SHALL take a base, a local and a remote card as paths, plus the path to write, run the three-way merge in process, and project the result as TOML for editing. It SHALL write the output path only once the edited document parses, and SHALL leave it untouched otherwise.

Taking the three rather than a pre-merged body with markers is what keeps the document a card. Line markers are how a line-oriented merge shows an unresolved region, and a vCard is not lines: a marker in one would break every parser downstream, including this one. The merge is a pure function over bodies already at hand, so running it here rather than receiving its output costs nothing and invents no format.

#### Scenario: The output is written only when the document is decided
- GIVEN a merge whose document still holds an undecided collision
- WHEN the editor exits
- THEN the output path is not written

### Requirement: A collision is duplicate keys
A field both sides changed SHALL be written once per surviving side, each line naming its side, with the ancestor above them as a comment. Resolving SHALL be deleting the lines that are not wanted, or replacing all of them with a value of the user's own.

TOML forbids duplicate keys, so an undecided document does not parse and cannot be applied. The forcing is the format's rather than a rule of ours, which means there is nothing to enforce, nothing to name, and no way to save a decision that was never made. Commenting the alternatives out instead would leave the field absent, and absence is how a user deletes a property, so an overlooked collision would drop a field and look deliberate.

The ancestor is a comment because keeping it is never the resolution to a collision: both sides moved away from it, and offering it as a live third line invites discarding two edits at once. Commented, it still says what the field was, which is what makes the two live lines mean anything.

#### Scenario: An undecided document is refused
- GIVEN a merged document holding a collision as written
- WHEN it is applied
- THEN it is refused, naming the field left undecided rather than reporting a syntax error

#### Scenario: Deleting the other line decides it
- GIVEN the same document with one of the two lines removed
- WHEN it is applied
- THEN the card carries the surviving value

### Requirement: Only a genuine choice is rendered as one
A report entry the merge already decided SHALL be a header comment, not duplicate keys. A removal against an update is decided, the update winning whichever side it came from, so there is nothing to choose and one of the two candidates could not be written as a line in any case.

An instance matched by position rather than by `PID` SHALL be said in the header too. Two sides each editing what they think of as the second phone number can be paired into one collision by position alone, and the projection cannot tell: the choice still renders as one key, but the pairing behind it may be the wrong one, and only the reader can see that.

A collision on a key the document does write SHALL NOT be demoted to a header note. Where the report breaks such a collision into parts the projection has no key for, the parts are put back together and the field's own key is contested whole, and a key the document writes once is contested once however many parts the report reported. The catch-all note says a part not shown here was contested, which is a lie about a field written two lines further down, and it settles for the local value where a reader was available to decide.

#### Scenario: A decided report line is not a choice
- GIVEN a merge where one side removed a property the other changed
- WHEN the document is projected
- THEN the surviving value is written once and the removal is said in a comment

#### Scenario: A collision on a projected key is offered as a choice
- GIVEN two sides rewriting `ORG` wholesale
- WHEN the document is projected
- THEN `organization` is written once per side, as an array, and the document refuses to apply

### Requirement: A structured collision stays inside its table
A collision inside a structured value SHALL be rendered as duplicate keys within the single table that projects the instance, and SHALL NOT be rendered as a repeated array-of-tables block. Repeating such a header is valid TOML and would produce a second instance rather than a parse error, so the forcing that makes the whole convention safe would silently vanish exactly where the value is most complex.

The decomposition is not this crate's to derive. The merge report names which component of a structured value moved and which parameter changed, so a collision addresses one projected key however deep it sits, and two sides editing different components of one address never collide at all.

#### Scenario: One address, one contested key
- GIVEN two sides setting a different street on the same address
- WHEN the document is projected
- THEN one address table is written, its street contested and its other keys written once

### Requirement: A contest is rendered in its own instance's block
A collision on a repeatable property SHALL be written into the block of the instance the merge report names, and SHALL fall back to a search by value only where that block cannot be trusted to be the right one.

The report indexes the instance in the base card while the document projects the merged one, and the pairing between them (by `PID`, then by equality, then by position) is not exposed, so the index is authoritative only while that pairing was positional. The block it names is therefore taken when it holds the value the merge kept, which is the local one wherever a choice is rendered at all, and the value search stands behind it for the rest.

Addressing by value alone lets an uncontested sibling carrying the same local value steal the contest, and then the reader decides one phone number and overwrites another.

#### Scenario: An uncontested sibling does not steal the contest
- GIVEN two phones where only the second collides, both reading the same number locally
- WHEN the document is projected and the remote side is kept
- THEN the second phone carries the remote number and the first is untouched

### Requirement: A collision is written in the projection's own spelling
The values of a collision SHALL be rendered as the projection renders that field: an array for a list field, a native date for a date field where the value is complete, a quoted string elsewhere.

A document that contests a key in a syntax it uses nowhere else asks the reader to learn a second spelling at the moment they are least able to, and a reader editing a contested line and an untouched one should be editing the same thing. The two spellings fold back to the same card, so this costs nothing and buys the legibility the projection exists for.

#### Scenario: A contested date reads as a date
- GIVEN two sides setting a different `BDAY`
- WHEN the document is projected
- THEN each side's line reads `birthday = 1997-04-15`, not a quoted basic string
