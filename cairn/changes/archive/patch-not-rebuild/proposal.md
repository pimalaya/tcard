---
cairn: change
id: patch-not-rebuild
status: landed
created: 2026-08-29
---

# A modelled line is rebuilt from the document, so everything the document does not show is lost

## Why

The projection's own header promises that what tCard does not model is kept verbatim. That held for unmodelled *properties* and not for the parts of a modelled one, because the fold back rebuilt each modelled line out of the TOML keys rather than editing the line the value came from. Three losses followed from that one mechanism.

A parameter the projection does not show was dropped: `EMAIL;PREF=1;TYPE=work` came back `EMAIL;TYPE=work`. `PREF` decides which address a client actually uses, and the lost `PID` is the identity the merge pairs instances by, so the editor quietly made every later merge worse. It sits in the interactive conflict path too, so a person settling a conflict by hand dropped them without being told.

A component the version hides was dropped: `pobox` and `ext` are hidden in vCard 4.0, and an inbound 4.0 address came back without its post office box. Hiding a component from a form is a fair way to discourage writing it; dropping it is claiming the card the reader was shown is the card they had.

Several properties of one repeatable name collapsed into one: `LANG;PREF=1:fr` and `LANG;PREF=2:en` became `LANG:fr,en`, losing the preference order RFC 6350 section 6.4.4 exists for, and a second pass escaped the separator and turned two languages into one value called `fr,en`.

## What

- The fold back is handed the card's own lines for a property and patches them: the name and parameters are the line's, and only the value and the `TYPE` the document writes are replaced.
- An unchanged `TYPE` leaves the line's bytes alone, whatever case or spelling it was written in (`;WORK`, `TYPE="work,voice"`, one parameter per type).
- A component the document does not write is taken from the line it came from rather than left empty.
- A field's items are spread over the lines they came from, each keeping as many as it held, so repeated properties keep their identity; a surplus item opens a line of its own.
- An empty item is dropped from a `,` list, where it says nothing, and kept in a `;` list, where components are ordered and an empty one holds a place.
