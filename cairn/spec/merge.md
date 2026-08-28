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

#### Scenario: A decided report line is not a choice
- GIVEN a merge where one side removed a property the other changed
- WHEN the document is projected
- THEN the surviving value is written once and the removal is said in a comment

### Requirement: A structured collision stays inside its table
A collision inside a structured value SHALL be rendered as duplicate keys within the single table that projects the instance, and SHALL NOT be rendered as a repeated array-of-tables block. Repeating such a header is valid TOML and would produce a second instance rather than a parse error, so the forcing that makes the whole convention safe would silently vanish exactly where the value is most complex.

The decomposition is not this crate's to derive. The merge report names which component of a structured value moved and which parameter changed, so a collision addresses one projected key however deep it sits, and two sides editing different components of one address never collide at all.

#### Scenario: One address, one contested key
- GIVEN two sides setting a different street on the same address
- WHEN the document is projected
- THEN one address table is written, its street contested and its other keys written once
