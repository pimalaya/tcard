---
cairn: spec
capability: editor
status: current
---

# Editor round trip

How a document reaches a person and comes back. The projection it is written in belongs to template, the collisions it may carry to merge; what is here is the editor tCard opens, the file it hands over, and what happens to a buffer that does not fold back.

### Requirement: The editor is `$VISUAL`, then `$EDITOR`, then nothing
tCard SHALL resolve the editor from `$VISUAL`, then `$EDITOR`, and SHALL stop there. When neither is set it SHALL fail, naming both variables and `--editor`, rather than fall back to a list of its own.

A fallback list ends in generic file openers, `xdg-open` and `open` among them, which hand the document to whatever the desktop associates with it and return before it is closed. tCard then reads back a document nobody has touched and reports a card that came out as it went in, which one layer up reads as an edit given up on. A program that never asked for a fallback SHALL not be given one.

The value SHALL be split on whitespace into a command and its arguments, so `code --wait` and `emacsclient -c` both work, and the path of the document SHALL be appended last.

#### Scenario: Neither variable is set
- GIVEN an environment carrying neither `$VISUAL` nor `$EDITOR`
- WHEN `tcard edit` is run without `--editor`
- THEN it fails naming `$VISUAL`, `$EDITOR` and `--editor`, and spawns nothing

### Requirement: An editor is named for one run
`edit` and `merge` SHALL take `--editor <COMMAND>`, which wins over both variables for that invocation. It is what makes the round trip scriptable and a broken environment recoverable without changing it, and it is the twin of the `--composer` flag of a caller spawning tCard.

### Requirement: The document is handed over as a file, never as a pipe
tCard SHALL write the document to `tcard-<uuid>.toml` in the temporary directory, spawn the editor on that path with stdin, stdout and stderr all inherited, and read the file back when the command exits. It SHALL capture none of the three: an editor handed a pipe instead of the terminal hangs, or draws where nothing reads.

A non-zero exit status SHALL abandon the round trip and report it.

### Requirement: A declined buffer is kept and named
The re-open loop SHALL survive a reader who declines it: when a document that does not fold back is not re-edited, the temporary file SHALL be kept and the error SHALL name its path. A minute of typing outlives the run that produced it, and the path is the recovery. A round trip that folded back SHALL remove the file.

An editor that could not be spawned at all SHALL remove it instead: the file then holds nothing but what tCard wrote a moment earlier.
