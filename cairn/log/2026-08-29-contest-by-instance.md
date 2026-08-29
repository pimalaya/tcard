---
cairn: log
change: contest-by-instance
date: 2026-08-29
---

# A contest is addressed by instance, with the value behind it

A `Choice` now carries the instance the merge report names, and `locate` looks in that instance's block before anything else. The block is taken when it holds the value the merge kept; failing that the old search stands behind it, first for the block holding that value, then for the first block holding the key.

## Why the index alone is not enough

The report's path indexes the instance in the **base** card. The document projects the **merged** card, which is the local one with the remote's actions replayed, and vcard-rs pairs base to local by `PID`, then by equality, then by position. That pairing is computed inside `merge` and is not on `VcardMergeReport`, which carries the merged card, each side's actions and the conflicts, and nothing that says which local instance a base index landed on. So the index addresses the block exactly while the pairing was positional and no structural change shifted it, and cannot be trusted blind.

Reimplementing the pairing here was the other option and was refused: it would duplicate upstream logic that can change under us without a compile error, and the check that makes the index safe is already free. A collision only renders as a choice where the merge kept the local value, so the block at the reported index must hold that value; when it does not, the two orders have diverged and the value search is the better answer.

## What the value search was getting wrong

Taking the first block whose value matched let an *uncontested* sibling carrying the same local value steal the contest. Keeping the remote side then decided the wrong phone and overwrote the one the reader never looked at, and the document was visibly self-contradictory while it happened, the commented ancestor belonging to one instance and the block around it to another.

The shapes it got right are kept: two contested instances holding equal local values are still told apart, because the search skips a line another choice has taken and so consumes equal values in block order.

## Verification

The reproduction runs. The five forcing laws still hold at 2000 cases, and the equal-values case that pinned the old mechanism still passes.

Capabilities moved: `merge`.
