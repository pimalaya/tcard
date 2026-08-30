---
cairn: spec
capability: api
status: current
---

# Public surface

The shape of what the crate exposes: the types the entry points hang off, how the source tree is divided, and what the crate refuses to depend on. What each of them does belongs to template and merge.

### Requirement: An entry point is a type that names its inputs
Every public entry point SHALL be a method on a type carrying the inputs as named fields, never a free function over positional arguments. A private helper of a few lines stays a function.

Three strings in a row say nothing about which is which, and a merge given its sides the wrong way round compiles and merges backwards. A type names them at the call site, and the name survives every later argument.

vcard-rs settled the same question one version below, `VcardMerge { base, left, right }.merge()` replacing a positional `merge`, and a layer that reads the other way makes the reader change idiom halfway down the stack.

#### Scenario: A merge is given its three cards
- GIVEN a base card and the two sides that diverged from it
- WHEN a merge is asked for
- THEN each card is passed by name, and no order of the three is silently valid

### Requirement: Both directions belong to one value
Projecting a form and folding one back SHALL be two methods on the same value, holding the cards once, rather than two functions each taking the card again.

The fold-back patches the tree the form was projected from, which is the one-reader-per-body requirement stated in the type: a caller cannot hand the second half a different card than the first half read, because it does not hand it a card at all.

#### Scenario: A round trip through the editor
- GIVEN a card read once
- WHEN it is projected, edited and folded back
- THEN the same value serves both directions and the card is parsed once

### Requirement: A verb is a module
Each command of the CLI SHALL live in its own module under cli, over shared modules for the arguments several verbs take and for the editor round trip. A module of the merge SHALL likewise hold one part of it.

A file holding every verb of a CLI, or every part of a merge, is navigable by scrolling and by nothing else. The rule is the one the rest of Pimalaya already follows, and it makes the name of the thing you are looking for the name of the file it is in.

#### Scenario: Looking for what a verb does
- GIVEN the source tree
- WHEN the behaviour of one command is looked for
- THEN it is in the module named after that command

### Requirement: The error enum is written by hand
The crate error SHALL implement Display and Error by hand rather than derive them from a dependency.

One enum of four variants is four match arms, and the crate below it writes its own the same way. A derive dependency earns its place where the enum is large or the source chaining is intricate, and this one is neither.

#### Scenario: The dependency list of the core
- GIVEN the library built with no features
- WHEN its dependencies are read
- THEN no error-derive crate is among them
