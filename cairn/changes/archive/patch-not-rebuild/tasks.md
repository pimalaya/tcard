---
cairn: tasks
change: patch-not-rebuild
---

- [x] Take a content line apart: name, parameters, value, quoted values included
- [x] Rebuild a line from the original's prefix, replacing only the shown `TYPE`
- [x] Leave the bytes alone when the written type is the one already carried
- [x] Read a hidden component back from the line it came from
- [x] Spread a list field's items over the lines they came from
- [x] Keep an empty component of a `;` list, drop an empty item of a `,` list
- [x] Test: a modelled property keeps `PREF`, `LANGUAGE` and `VALUE`
- [x] Test: a vCard 4.0 address keeps its post office box
- [x] Test: two `LANG` properties stay two, and settle
