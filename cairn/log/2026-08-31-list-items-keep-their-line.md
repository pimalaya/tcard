---
cairn: log
change: list-items-keep-their-line
landed: 2026-08-31
---

# A list item goes back to the line it came from

A repeatable property's items are shown as one array however many lines carried them. Giving them back counted them off the front, so removing one slid every item behind it onto the line before, parameters and all:

```
before:  NICKNAME;PREF=1:Jim,Jimmy
         NICKNAME;PREF=2:Big Tuna
after:   NICKNAME;PREF=1:Jim,Big Tuna
```

Deleting `Jimmy` made `Big Tuna` the preferred nickname and deleted the line that said otherwise. Nothing in the form suggested either would happen, and the test suite was green throughout: every existing test exercised one line.

Found in tCal first, by an audit carried across from this crate's own component work. There it hit `CATEGORIES` and `FREEBUSY`, where dropping one busy period reported a free afternoon as busy. The two crates had the same `spread`, so they had the same bug.

**An item now belongs to the line whose value held it** (template/model.rs), matched by value. An item no line held fills the room a line lost, in document order, which is what keeps a rename on its own line rather than opening a second. A line left with no items is dropped.

**Leftover items share one new line** rather than taking one each. Which line's parameters they should carry is the question several lines make unanswerable, so they carry none, together.

**At most one line is the array**, in document order. That is the common case and it has nothing to disambiguate, so an added item joins the line and its parameters. It is also what tCal's README documents for its twin of this field, where the one-line-each behaviour had made the front-page example untrue.

**Matching is on the values, not their escaping.** The call site used to escape before spreading, which compared a card's own spelling against ours; it now spreads the values and escapes on the way out.

`ORG` is untouched, its `;` joining one property's own components rather than several properties, and a test pins that a third unit stays a third component rather than becoming a second `ORG`.

Verified: 38 unit tests, the merge suite and the fixtures green, five of them new over the removal, the rename, the join, the shared new line and `ORG`. The corruption was reproduced against the built binary before and after.

Spec updated: `template` (ADDED: "A list item goes back to the line it came from", "One line leaves nothing to disambiguate").
