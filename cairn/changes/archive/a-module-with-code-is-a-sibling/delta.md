---
cairn: delta
change: a-module-with-code-is-a-sibling
---

## ADDED Requirements

### Requirement: The projection is a sibling module, not an aggregator
The projection SHALL live in src/template.rs beside its src/template/ folder, rather than in src/template/mod.rs, because it carries the engine itself and not only the declarations of the modules under it.

The mod.rs choice is content-based. A folder whose mod.rs holds nothing but module declarations and re-exports keeps it; a module carrying code of its own is a sibling file next to the folder, so a reader can tell the two apart by the file name alone.

#### Scenario: Where the projection lives
- GIVEN the projection engine and the leaf modules it declares
- WHEN the source tree is read
- THEN the engine is src/template.rs and the leaf modules are files under src/template/

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
