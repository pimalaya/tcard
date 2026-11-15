# Migration guide

## From v0.2 to v0.3

v0.3 takes the output and error contract the Pimalaya CLIs share, and changes how the form shows a photo. The CHANGELOG lists every change; these are the ones to act on.

### The form

- **A photo is `embedded`, `file` or `uri`**, where v0.2 wrote its value. An inline photo shows as `embedded.type` and `embedded.sha256` and comes back byte for byte when left alone; to replace it, swap its `embedded` for `file = "~/me.jpg"`. Project the card again with `tcard template` rather than reuse a form v0.2 wrote. `LOGO`, `SOUND` and `KEY` take the same three shapes.
- **Long lines are folded.** An edited vCard 3.0 or 4.0 line longer than 75 octets goes out folded, as RFC 6350 section 3.2 asks, where v0.2 wrote it on one line. A script comparing a card byte for byte against an unfolded one unfolds both first.

### JSON output and exit codes

- **A document prints as `{"contents": ...}`** under `--json`, where v0.2 printed the raw TOML or vCard, and as the receipt `{"path": ..., "size": ...}` when it went to a file. Text output is unchanged: the document as itself, nothing for a file.
- **A failure prints the failure object** under `--json`, with a stable `code` (`card-unreadable`, `template-unreadable`, `collision-undecided`, `embedded-unknown`, `file-unreadable`, `usage-invalid`, or `null`), and exits 1. Read `code` rather than matching the message.
- **A refused command line exits 2**, printing that object coded `usage-invalid` under `--json`: a mistyped flag, `apply - -` (one stdin cannot carry both inputs), and `edit` or `merge` under `--json`.
- **`edit` and `merge` no longer run under `--json`**: the editor drew on the stdout a JSON caller reads. A program wanting JSON projects the form with `tcard --json template`, edits it itself, and folds it back with `tcard --json apply`.
- `tcard json-schema` writes the schema of what each command prints; packages install them under share/schemas.

### Building

- The minimum supported Rust version is 1.89.
- The binary is the `cli` feature, on by default: `cargo install tcard` builds it, and a library consumer turns it off with `default-features = false`.
