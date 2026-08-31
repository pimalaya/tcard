---
cairn: change
id: a-fold-back-is-a-verb
status: landed
created: 2026-09-01
---

# Delta

## ADDED Requirements

### Requirement: The fold back is a verb of its own
`apply` SHALL take an edited TOML document and the card it was projected from, fold the one onto the other, and write the resulting vCard. It SHALL spawn nothing.

The projection is a round trip, and only its outward half was a verb: a form edited by a script, a filter or a graphical app had no way back, though the library exposes the fold back and `edit` uses it. Who filled the form is none of tCard's business.

The document SHALL be a path or `-` for stdin, and both inputs SHALL NOT be stdin at once. `apply` SHALL take the same `--version` as `template`, being the version a card folded onto nothing is written at.

It SHALL write the source file back in place as `edit` does, `--output` sending the result elsewhere.

#### Scenario: A form edited outside tCard is folded back
- GIVEN a card projected with `template` and edited by anything at all
- WHEN `apply` is given that document and that card
- THEN the result is what `edit` would have written, byte for byte

### Requirement: A fold back with nobody to ask is an error
A document that does not parse, and one leaving a collision undecided, SHALL fail naming what could not be folded, rather than offer a re-edit. `edit` asks because a person is sitting in front of it; `apply` has nobody to ask, and a prompt in a pipeline is a hang.

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
