---
cairn: change
id: prefix-the-library
status: landed
created: 2026-08-30
---

# Half the public surface carries the domain prefix, half does not

## Why

`TcardError` carries it and `Cards`, `Card`, `Template`, `Merge`, `Merged` and the `Result` alias do not, which is not a distinction anyone drew: it is where the naming rule was applied and where it was forgotten.

naming-007 asks every pub item of a library to carry its domain prefix, and gives two exemptions, neither of which fits: a type re-exported from a foreign crate, and the shared std toolkit crates the crate name already namespaces. tCard is a library first, published as `tcard` and consumed as `tcard::template::Template`, where the prefix is what tells a reader whose `Template` they are looking at once it sits in a `use` list beside a dozen others.

The counterweight is cli-001, which overrides the rule for the cli subtree: nothing there is meant to be consumed as a library and the binary already names itself, so `Cli`, `Command`, `TemplateCommand`, `SourceArg` and `Output` stay bare. That is the line, and it happens to fall exactly where the `cli` feature does.

## What

- Prefix every pub item the library ships: `TcardCards`, `TcardTemplate`, `TcardMerge`, `TcardMerged`, `TcardResult`, next to the `TcardError` that already had it.
- Leave the cli subtree bare, cli-001 overriding the rule there.
- Keep the module headers and the prose reading as English: the module is still `merge`, its header is still "# Merge", and a doc sentence still starts with the word rather than with the type.
