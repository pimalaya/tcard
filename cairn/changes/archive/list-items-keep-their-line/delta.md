---
cairn: change
id: list-items-keep-their-line
status: landed
created: 2026-08-31
---

# Delta

## ADDED Requirements

### Requirement: A list item goes back to the line it came from
An item of a repeatable property SHALL be given back to the line whose value held it, matched by value rather than by position. A line's parameters describe the items that line carried, so counting items off the front of the array hands each line whatever has room and relabels every item behind a removed one.

An item no line held SHALL fill the room a line lost, in document order, so renaming an item rewrites its own line. Whatever is left over SHALL share one new line, which carries no parameters: which line's it should have carried is the question several lines make unanswerable. A line left with no items SHALL be removed.

Matching SHALL be on the values rather than on their escaping, the escaping being applied on the way out, so an item is the same item however the card happened to spell it.

### Requirement: One line leaves nothing to disambiguate
A property holding at most one line SHALL take the array as that line's items, in the order the document wrote them. There is no second line to attribute an item to, so an added item joins the line and its parameters rather than opening a bare second one.

A `;`-joined property (`ORG`) SHALL likewise stay one line: the separator joins one property's own components rather than several properties.

## MODIFIED Requirements

## REMOVED Requirements
