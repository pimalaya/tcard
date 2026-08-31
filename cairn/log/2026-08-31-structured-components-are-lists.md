---
cairn: log
change: structured-components-are-lists
landed: 2026-08-31
---

# A structured component holds a list, and an untouched one keeps its bytes

tCard corrupted any card whose structured value carried a multi-valued component. RFC 6350 section 6.2.2 says an `N` component "can include multiple text values separated by the COMMA character", and its own example carries two additional names and three suffixes. Modelling each component as one string made that comma indistinguishable from a comma someone typed, so a fold-back escaped it:

```
before:  N:Stevenson;John;Philip,Paul;Dr.;Jr.,M.D.,A.C.P.
after:   N:Stevenson;Jon;Philip\,Paul;Dr.;Jr.\,M.D.\,A.C.P.
```

Only `given` was edited. `N` is one line, so changing any component re-renders all of them, and every other component paid. Two additional names became one name holding a comma; three suffixes became one. Silently, in the operation tCard exists to make safe.

**A component now declares whether it holds a list** (template/model.rs). `Component` grew from a three-tuple into a struct carrying that, which also stopped the positional reads that made the tuple hard to follow. `N`'s five hold lists. `ADR`'s street does, section 6.3.1 naming a multi-line street as the case; its six others do not, and neither do `GENDER`'s two.

A list component projects as an array and reads back joined on commas, each value escaped on its own. The projection splits the component **before** unescaping it, which is the whole of the fix: unescaping first is what lost the separator among the literals.

An absent component projects as `[]`, not `[""]`: splitting an empty string yields one item, which would have shown a slot nobody asked for.

**A bare string is accepted where an array is expected**, read as the one value it is. Reading stays liberal and only what a fold-back writes is canonical, which is the crate's posture everywhere else.

**A component the document did not change keeps the card's bytes** (template/toml.rs). This is the rule the line already follows, applied one level down, and it is what saves the components that are not lists: a `country` reading `Congo\, The Democratic Republic of the` survives an edit to the street beside it. The comparison is against what a fold-back would write rather than against the raw bytes, so a needless escape is still dropped.

**The name components got hints.** `family`, `given` and `additional` are the RFC's role names and stay, positional names being the thing that varies between cultures, but nobody guesses that `additional` means the middle names. The hint column already existed: `# last name(s)`, `# first name(s)`, `# middle name(s)`.

Verified: 33 unit tests, the merge suite and the fixtures green. Eleven golden fixtures were regenerated for the new form, and reading them back is where the change proved itself: the RFC 6350 author card projects `suffixes = ["ing. jr", "M.Sc."]`, two values the old form showed as one string and would have escaped into one.

A new crafted fixture, multi_components, pins the round trip with no lossy marker: every card already carrying a multi-valued component was lossy, so nothing asserted the round trip this bug lived in. Four unit tests cover the array projection, the edit that used to corrupt, the bare-string spelling and a comma someone actually typed.

Spec updated: `template` (ADDED: "A structured component that holds a list is typed as one", "A component the document did not change keeps its bytes", "A name component says what it is").
