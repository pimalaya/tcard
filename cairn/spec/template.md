---
cairn: spec
capability: template
status: current
---

# Template

Projecting a card as an ergonomic TOML form and folding the edited form back onto it. The document is an editing affordance rather than an interchange format, so the rules here are about what survives the round trip: a card comes back as it was but for what the reader actually changed. What the merge document says and what it refuses belongs to merge.

### Requirement: One reader per body
A body SHALL be read once. The reader that parses a card for the merge SHALL be the reader that parses it for the projection, so the two agree by construction rather than by serialising between them.

Two readers do not merely cost a parse. They disagree, and the disagreement is invisible: a value the first reads faithfully and the second normalises reaches the document already changed, and no test comparing the document against the second reader's output can see it.

#### Scenario: A value no reader normalises
- GIVEN a card whose list item carries an escape, and whose gender identity is a lowercase letter
- WHEN it is projected and applied unchanged
- THEN both come back byte-exact

### Requirement: A property is addressed by its bare name
The editor SHALL match a property by the name behind its group (`item1.EMAIL` is an `EMAIL`), and a line written back SHALL keep the group of the line it replaces.

A group is a label on a line, not part of what the property is: RFC 6350 section 3.3 lets any property carry one, and Apple, iOS and Google exports label addresses and URLs that way as a matter of course. Matching the whole prefix instead makes a grouped property invisible to the fold back, which then appends a group-less copy beside it and grows the card by one line per round trip.

#### Scenario: A grouped property is rewritten in place
- GIVEN a card holding `item1.EMAIL` and its `item1.X-ABLabel`
- WHEN an untouched projection is folded back
- THEN the card is unchanged, with one `EMAIL` line still carrying its group

### Requirement: A projection settles at once
Folding an untouched projection back SHALL leave the card as it was, and folding the result again SHALL change nothing further, for every card in the golden fixture set.

A card that only settles after several passes is a card that moves under the reader, and one that never settles loses or gains something on every pass. The fixtures are real exports, so they are where a normalisation nobody intended shows up first.

The values the generators exercise include the ones a second reader used to alter on the way in, escapes inside comma-separated list items and single-letter gender identities among them, which the corpus laws once ran behind a filter.

#### Scenario: A real export does not grow
- GIVEN a golden fixture carrying grouped properties
- WHEN it is folded back twice
- THEN the second fold changes nothing the first did not

### Requirement: A modelled line is patched, never rebuilt
Folding a modelled property back SHALL start from the line the value came from and replace only what the document writes, its value and the `TYPE` it shows. Every other parameter SHALL survive, and a `TYPE` the document spells as the line already carries it SHALL leave the line's bytes untouched.

The projection is an editing affordance rather than an interchange format, so anything it does not show is something the reader never chose to change. Rebuilding the line from the TOML keys instead throws away `PREF`, which decides which address a client uses, and `PID`, which is the identity a later merge pairs instances by, so an editor that rebuilds makes every merge after it worse.

#### Scenario: An unshown parameter survives
- GIVEN a card holding `EMAIL;PREF=1;TYPE=work`
- WHEN an untouched projection is folded back
- THEN the card still holds `EMAIL;PREF=1;TYPE=work`

### Requirement: A hidden component is kept
A component the projection does not show at the card's version (`pobox` and `ext` in vCard 4.0) SHALL be written back from the line it came from, at its own position.

Hiding a deprecated component discourages writing one, which is the point. Dropping it makes the document a lie: the reader was shown a card and handed back a different one, with nothing said anywhere.

#### Scenario: A post office box survives vCard 4.0
- GIVEN a vCard 4.0 card holding `ADR:PO Box 12;;1 Main St;...`
- WHEN an untouched projection is folded back
- THEN the address still holds its post office box

### Requirement: Repeated properties keep their identity
A field the projection writes as one array SHALL fold back over the lines its items came from, each line keeping as many items as it held, a surplus item opening a line of its own.

RFC 6350 keeps "one property holding several values" (`NICKNAME:a,b`) apart from "several properties" (`LANG;PREF=1:fr` twice), and the projection flattens both into one array. Joining that array into one line collapses the second case, taking the parameters that told the instances apart with it, and the next pass escapes the separator and turns the two values into one nonsense value.

#### Scenario: Two languages stay two
- GIVEN a card holding `LANG;PREF=1:fr` and `LANG;PREF=2:en`
- WHEN an untouched projection is folded back
- THEN the card still holds two `LANG` lines with their preferences

### Requirement: The projection is a sibling module, not an aggregator
The projection SHALL live in src/template.rs beside its src/template/ folder, rather than in src/template/mod.rs, because it carries the engine itself and not only the declarations of the modules under it.

The mod.rs choice is content-based. A folder whose mod.rs holds nothing but module declarations and re-exports keeps it; a module carrying code of its own is a sibling file next to the folder, so a reader can tell the two apart by the file name alone.

#### Scenario: Where the projection lives
- GIVEN the projection engine and the leaf modules it declares
- WHEN the source tree is read
- THEN the engine is src/template.rs and the leaf modules are files under src/template/
