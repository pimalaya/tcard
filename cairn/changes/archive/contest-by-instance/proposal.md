---
cairn: change
id: contest-by-instance
status: landed
created: 2026-08-29
---

# A contest is rendered in the wrong instance's block

## Why

A contested line was found by value: the renderer walked every `[[phone]]` or `[[address]]` block and took the first whose right-hand side equalled the choice's local value. Where an uncontested sibling happened to carry that same value locally, the contest was written into the sibling's block, so the reader decided the wrong instance and the phone they never looked at was overwritten. The document was visibly self-contradictory when it happened: the commented ancestor belonged to one instance and the block around it to another.

Most of the shapes the value match was given do hold, and the fix must not throw them away. Two contested instances whose local values are equal are told apart correctly, because the line search skips a line another choice has already taken and so consumes equal values in block order. The failure needs an *uncontested* sibling carrying the contested local value, since only then is there a matching line no other choice has claimed.

The report already carries the instance the collision sits on. What it carries is that instance's position in the **base** card, and the document projects the **merged** card, so the two agree only where the pairing behind them was positional. vcard-rs pairs instances by `PID`, then by equality, then by position, and does not expose the pairing it made, so the index alone cannot address the block in every case.

## What

- The instance the report indexes is the address: the choice looks in that block first.
- Because the index is the base card's, it is trusted only when the block it names holds the value the merge kept, which is the local one wherever a choice is rendered at all. That check costs nothing and is exactly the signal that says whether the two orders agree.
- Failing that, the search falls back to what it did before: the first block holding that value, then the first block holding the key.
