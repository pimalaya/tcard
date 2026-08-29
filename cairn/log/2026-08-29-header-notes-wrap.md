---
cairn: log
change: header-notes-wrap
landed: 2026-08-29
---

# A header note wraps like the rest of the document

The projected document's own header comment is written to a column, and so are tCal's merge notes. tCard's ran to one line however long they were, so `X-FOO: both sides changed a part not shown here; the local value was kept` was a single 74-column line under a header wrapped at 66.

## What landed

`WRAP`, the column, and `comment`, the wrapper, are tCal's, lifted whole and adapted to the `Vec<String>` the tCard decorator works in rather than tCal's string buffer. A note is folded at 66 including the `# ` prefix, and a continuation line is indented two spaces so it sits under the text of its bullet rather than under the dash.

## What the tests had to give up

Four assertions matched a note as a whole line of the document, which a note long enough to fold no longer is. They read the header instead, through a `notes` helper that collapses the leading comment block into one line, which is what tCal's suite already did and is the more honest assertion either way: what is being claimed is that the header says something, not that it says it on one line.

A new law, `a_long_note_wraps_under_itself`, holds the column itself: no header line passes it, and a note that needs a second line gets an indented one.

## Verification

Both configurations build, clippy is clean over all targets, `cargo fmt` run. The merge forcing suite is 15 tests, the whole suite green.

Capabilities moved: merge (ADDED: A header note wraps at the document's column).
