---
cairn: delta
change: one-section-per-kind
---

## ADDED Requirements

### Requirement: The changelog is one net diff
A release section SHALL carry at most one heading per kind, and each entry SHALL state the behaviour as it ends up rather than the steps that reached it.

A release section answers what moved since the previous version, and a reader answers that by reading its headings once. A second heading of the same kind turns reading into searching: the reader who takes the first Added for the additions has no way of knowing the list resumes further down, past a heading that looked like the end of it.

History is what the cairn log is for. A changelog that keeps a second copy of it keeps a worse one, since an entry later undone still reads as current.

#### Scenario: A section that grew in two sittings
- GIVEN an unreleased section holding additions written at different times
- WHEN the changelog is read
- THEN all of them are under the one Added heading
