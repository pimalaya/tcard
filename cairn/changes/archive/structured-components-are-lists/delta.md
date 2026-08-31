---
cairn: change
id: structured-components-are-lists
status: landed
created: 2026-08-31
---

# Delta

## ADDED Requirements

### Requirement: A structured component that holds a list is typed as one
A component of a structured value SHALL declare whether it holds several values, and a component that does SHALL project as a TOML array and read back as the comma-separated list RFC 6350 section 6.2.2 defines, each value escaped on its own.

`N`'s five components hold lists. `ADR`'s street does, section 6.3.1 naming a multi-line street as the case; its six others do not, the RFC allowing them "where it makes semantic sense" and no sense being served by an array of postal codes. `GENDER`'s two do not.

A bare string SHALL be accepted where an array is expected, and read as the one value it is. Reading stays liberal; only what a fold-back writes is canonical.

### Requirement: A component the document did not change keeps its bytes
When the value a document holds for a component is what the card's own component already meant, that component SHALL go back out as the card wrote it, rather than being re-escaped from the form.

This is the rule the line already follows, applied one level down, and it is what a scalar component depends on: a structured value is one line, so changing any component re-renders every component, and a comma the card used as a separator would otherwise be escaped into the value on the way past.

### Requirement: A name component says what it is
The `N` components SHALL keep the RFC's role names, `family` and `given` naming what a name is rather than where it is written, which is what varies between cultures. Each SHALL carry an inline hint saying which name it holds, `additional` above all, whose meaning nobody guesses.

## MODIFIED Requirements

## REMOVED Requirements
