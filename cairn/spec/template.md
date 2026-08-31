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

### Requirement: A line the document leaves alone keeps its layout
A line the document does not move SHALL come back with the layout it was written in: its folds, the blank lines that stood before it and its quoted-printable soft breaks. A line the document moves SHALL be written unfolded.

A layout is offsets into the line's own bytes, so an edit that changes the line's length invalidates them, and a fold put back at the wrong offset is worse than no fold at all. RFC 6350 section 3.2 recommends folding at 75 octets rather than requiring it, so writing an edited line whole is conformant.

The asymmetry is what a reader wants either way. Real exports fold every long line, so rewriting the layout of a card touched in one field yields a diff across the whole card, and an editor whose diff is that much larger than the edit is one nobody trusts over a synced or versioned file.

#### Scenario: A folded card is edited in one field
- GIVEN a card whose note is folded across two physical lines, with a blank line between two of its properties
- WHEN an untouched projection is folded back
- THEN the card comes back byte-exact, its fold and its blank line included
- AND WHEN the note is edited
- THEN only the note is rewritten, and it goes out on one line

### Requirement: The projection is a sibling module, not an aggregator
The projection SHALL live in src/template.rs beside its src/template/ folder, rather than in src/template/mod.rs, because it carries the engine itself and not only the declarations of the modules under it.

The mod.rs choice is content-based. A folder whose mod.rs holds nothing but module declarations and re-exports keeps it; a module carrying code of its own is a sibling file next to the folder, so a reader can tell the two apart by the file name alone.

#### Scenario: Where the projection lives
- GIVEN the projection engine and the leaf modules it declares
- WHEN the source tree is read
- THEN the engine is src/template.rs and the leaf modules are files under src/template/

### Requirement: A structured component that holds a list is typed as one
A component of a structured value SHALL declare whether it holds several values, and a component that does SHALL project as a TOML array and read back as the comma-separated list RFC 6350 section 6.2.2 defines, each value escaped on its own.

`N`'s five components hold lists. `ADR`'s street does, section 6.3.1 naming a multi-line street as the case; its six others do not, the RFC allowing them "where it makes semantic sense" and no sense being served by an array of postal codes. `GENDER`'s two do not, its sex being one code and its identity free-form text.

An absent component SHALL project as an empty array rather than an array holding one empty value. A bare string SHALL be accepted where an array is expected, and read as the one value it is: reading stays liberal, and only what a fold-back writes is canonical.

### Requirement: A component the document did not change keeps its bytes
When the value a document holds for a component still means what the card's own component meant, that component SHALL go back out as the card wrote it rather than being re-escaped from the form.

This is the rule the line already follows, applied one level down, and it is what a scalar component depends on: a structured value is one line, so changing any component re-renders every component, and a comma the card used as a separator would otherwise be escaped into the value on the way past. It is also what lets a needless escape be dropped, the comparison being against what a fold-back would write rather than against the raw bytes.

### Requirement: A name component says what it is
The `N` components SHALL keep the RFC's role names, `family` and `given` naming what a name is rather than where it is written, which is what varies between cultures. Each SHALL carry an inline hint saying which name it holds, `additional` above all, whose meaning nobody guesses.

### Requirement: Inline comments share one column across the card
The inline `#` hints SHALL align on one column measured over the whole card, not one per block: the first tab stop past the widest hinted left side anywhere in it, so every hinted line reaches it with at least one tab.

A column per section makes the comments step in and out as the reader scrolls, each block setting its own by whatever its widest value happens to be. One column reads as one column.

The card is the unit rather than the file, which is what tCal measures per component: a long value on one card would otherwise push every other card's comments out with it. Reading a form is the same job in both crates, so they align the same way.

### Requirement: A list item goes back to the line it came from
An item of a repeatable property SHALL be given back to the line whose value held it, matched by value rather than by position. A line's parameters describe the items that line carried, so counting items off the front of the array hands each line whatever has room and relabels every item behind a removed one.

An item no line held SHALL fill the room a line lost, in document order, so renaming an item rewrites its own line. Whatever is left over SHALL share one new line, which carries no parameters: which line's it should have carried is the question several lines make unanswerable. A line left with no items SHALL be removed.

Matching SHALL be on the values rather than on their escaping, the escaping being applied on the way out, so an item is the same item however the card happened to spell it.

### Requirement: One line leaves nothing to disambiguate
A property holding at most one line SHALL take the array as that line's items, in the order the document wrote them. There is no second line to attribute an item to, so an added item joins the line and its parameters rather than opening a bare second one.

A `;`-joined property (`ORG`) SHALL likewise stay one line: the separator joins one property's own components rather than several properties.
