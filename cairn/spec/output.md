---
cairn: spec
capability: output
status: current
---

# Output

What the binary prints on stdout and how it exits, under the family contract pimalaya-cli specifies (its cairn/spec output.md and error.md): every value a described type, `--json` keys in camelCase, a failure object with a stable code, exit 1 on a failure and 2 on a refused command line. What is here is how tCard, an editor with no account and no backend, fits that contract.

### Requirement: A document prints as itself, or as its contents under JSON
A verb that produces a document (the TOML form for `template`, the vCard for `edit`, `apply` and `merge`) and has no file to write it to SHALL print it on stdout: in text as itself, byte for byte, and under `--json` as `{"contents": <document>}`.

Text output stays the document so that a pipe can take it on, `tcard template contact.vcf | … | tcard apply - contact.vcf` among them.

#### Scenario: A template under JSON
- GIVEN a card
- WHEN `tcard --json template <card>` runs
- THEN stdout holds one object whose `contents` is the form `tcard template <card>` prints, and the exit code is 0

### Requirement: A document written to a file answers its receipt under JSON
A verb that writes its document to a file, the one `--output` names or the source edited in place, SHALL print under `--json` the receipt `{"path": <file>, "size": <bytes written>}`, and SHALL print nothing in text.

tCard runs as the composer of other programs (`composer = "tcard edit"` in Cardamum), which hand it a file, inherit its stdout and may be piping their own stdout onwards as the card. A text receipt would land in that stream; a JSON caller asked for one and gets it.

#### Scenario: A template written to a file
- GIVEN a card and a target path
- WHEN `tcard template <card> -o <path>` runs, then the same under `--json`
- THEN the first prints nothing, and the second prints the path and the size of the file written

### Requirement: A failure carries a stable code
A failure SHALL print the pimalaya-cli failure object, whose `code` is one of tCard's kebab-case codes when the failure belongs to a class a caller acts on, and `null` otherwise, and SHALL exit 1. The codes SHALL be:

- `card-unreadable`: a card does not read as a vCard, the source or one of a merge's three;
- `template-unreadable`: the TOML document does not parse;
- `collision-undecided`: the document still holds a merge collision;
- `embedded-unknown`: the document names an embedded digest the card does not hold;
- `file-unreadable`: a file the document names cannot be read;
- `editor-missing`: neither `--editor`, `$VISUAL` nor `$EDITOR` names an editor;
- `editor-failed`: the editor could not be spawned or exited with a failure.

The library stays free of pimalaya-cli: it fails with `TcardError`, and the CLI gives the chain the code of the variant it carries, every message kept.

#### Scenario: A form that does not parse
- GIVEN a TOML document holding `name = ` and nothing after it
- WHEN `tcard --json apply <document> <card>` runs
- THEN stdout holds the failure object coded `template-unreadable`, and the exit code is 1

### Requirement: A refused command line exits 2
The binary SHALL parse its arguments through pimalaya-cli's `clap::parse`, so a command line the grammar refuses exits 2, printing the failure object coded `usage-invalid` under `--json`. `apply` given `-` for both its document and its source SHALL fail coded `usage-invalid`, since one stdin cannot carry both.

#### Scenario: A mistyped flag under JSON
- GIVEN no command
- WHEN `tcard --json --frobnicate` runs
- THEN stdout holds the failure object coded `usage-invalid`, and the exit code is 2

### Requirement: Every verb has a schema
`tcard json-schema` SHALL write the JSON Schema of what each verb prints, `tcard-template`, `tcard-edit`, `tcard-apply` and `tcard-merge` describing the document or its receipt and `tcard-error` the failure object with its codes, and a test SHALL check the registry against the command tree, every runnable command described and every key naming one. The package SHALL install the schemas under share/schemas.
