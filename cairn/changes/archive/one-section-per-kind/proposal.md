---
cairn: change
id: one-section-per-kind
status: landed
created: 2026-08-29
---

# One section per kind, because a release is a net diff

## Why

changelog-002 says a release section reports the net changes against the previous version rather than a history of how it got there. Neither tool has shipped, so the unreleased section is the whole of the first release, and it is read once, top to bottom, by someone deciding what the tool does.

tCal's had grown two Added headings, one above a Changed heading and one below it. A reader who takes the first as the additions stops at Changed and never sees seven more of them, and nothing in the document tells them to keep going. The interior churn that produced the split is exactly what the cairn log already records.

## What

Keep one heading per kind in the unreleased section, merging what was split and preserving every entry, and write the rule down so neither repository drifts back into it. tCard's section is checked against the same rule, so the two hold one shape as well as one spec.
