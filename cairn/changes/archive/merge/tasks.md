---
cairn: tasks
change: merge
---

- [x] A `merge` verb taking base, local and remote paths plus an output path
- [x] Run the merge in process and project the merged card
- [x] Render a collision as duplicate keys, the ancestor commented above them
- [x] Render a decided or informational report line as a header comment, never as a choice
- [x] Decompose a structured collision to per-key duplicates inside one table
- [x] Note a positionally matched instance in the header, since the pairing may be wrong
- [x] Catch the duplicate-key error and name the undecided field
- [x] Write the output path only once the document parses
- [x] Test: an unresolved document does not apply
- [x] Test: keeping one line of a collision applies that value
- [x] Test: a collision on one `ADR` component renders as one key, not one table
- [x] Test: a removal against an update is a comment, not a choice
