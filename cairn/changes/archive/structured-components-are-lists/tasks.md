---
cairn: tasks
change: structured-components-are-lists
---

# Tasks

- [x] Turn `Component` from a tuple into a struct, carrying whether it holds a list.
- [x] Mark `N`'s five components and `ADR`'s street as lists, and hint the name components.
- [x] Project a list component as a TOML array, splitting the card's own component on commas.
- [x] Read a list component back from an array or a bare string, escaping each value and joining on commas.
- [x] Keep the card's bytes for any component whose value the document did not change.
- [x] Add fixtures covering a multi-valued component, and tests for the round trip and the string spelling.
- [x] Fold the delta into the spec and write the log entry.
