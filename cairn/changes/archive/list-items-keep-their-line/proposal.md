---
cairn: change
id: list-items-keep-their-line
status: landed
created: 2026-08-31
---

# A list item goes back to the line it came from

## Why

A repeatable property's items are shown as one array, however many lines carried them, so a fold-back has to give them back. It gave each line as many items as it *held*, counted off the front of the array. Remove one item anywhere but the end and everything behind it slides onto the line before, taking that line's parameters with it:

```
before:  NICKNAME;PREF=1:Jim,Jimmy
         NICKNAME;PREF=2:Big Tuna
after:   NICKNAME;PREF=1:Jim,Big Tuna
```

Deleting `Jimmy` made `Big Tuna` the preferred nickname and deleted the line that said otherwise. Nothing in the form suggested either would happen.

The same shape put every added item on a line of its own, so filling a blank form with two nicknames wrote two `NICKNAME` lines rather than one. tCal's README documents the single-line spelling for its twin of this field, which makes that half a documented behaviour rather than a preference.

Found by an audit of tCal that carried back: tCal had the identical bug, in `CATEGORIES` and `FREEBUSY`, where a free afternoon came back marked busy.

## What

**An item belongs to the line whose value held it**, matched by value rather than by position, because a line's parameters describe the items that line carried. An item no line held fills the room a line lost, in document order, which is how renaming an item rewrites its own line rather than opening a second.

**Whatever is left over shares one new line.** Which line's parameters those items should carry is the question several lines make unanswerable, so they carry none, together.

**At most one line leaves nothing to disambiguate**, so its items are the array in document order. That is the case the README documents, and it is also what makes an added item join the line it belongs to, parameters and all.

`ORG` is untouched: a `;` joins one property's own components rather than several properties, so it was already one line by construction and stays there.

## What this is not

Not a change to what the form shows. The array is the same array; only which line each item returns to has changed.

Not a merge of lines. Two properties of one name still keep their own lines and their own parameters, which is the behaviour this rests on rather than one it revisits.
