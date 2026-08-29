---
cairn: log
change: grouped-property-fold
date: 2026-08-29
---

# A grouped property is rewritten in place

The format-preserving editor now reads a content line's group apart from its property name. `Property` carries the group as originally spelled, matching happens on the bare uppercased name, `Property::set` puts the group back in front of the line it is handed, and `Component::get_all` hands a line back without it, so the reading and writing ends of the editor speak the same form.

Nothing above the editor changed. `template::apply` still asks for `EMAIL`, and it is the editor that decides `item1.EMAIL` is one: the group is a label on a line rather than part of what the property is, which is the reading RFC 6350 section 3.3 supports and the one that keeps the projection free of group keys nobody wants to edit.

## What it cost before

`set_all("EMAIL", ...)` found no line named `EMAIL`, concluded the property was absent, appended a group-less copy and left the grouped original alone. Every round trip added one copy of every grouped modelled property, so apple_contacts.vcf went 15 lines to 17 to 19 to 21, and the golden fixture that would have caught it carried a lossy marker, so it was never folded back at all. That marker is now the exception rather than the rule for this failure: every fixture is folded twice and has to settle.

## Verification

The two reproductions run: a grouped `EMAIL` survives once rather than twice, and every fixture settles after one round trip. The three projection laws (fold changes nothing, project-fold-project is identical, an unmodelled property survives verbatim) still hold at 3000 cases.

Capabilities moved: `template` (new).
