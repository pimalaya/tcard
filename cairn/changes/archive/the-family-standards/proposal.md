---
cairn: change
id: the-family-standards
status: landed
created: 2026-11-14
---

# The family standards

## Why

tCard joins the 2026-11 Pimalaya family release beside Cardamum, Calendula and Comodoro, which took the shared kit (pimalaya-cli 0.4) and its output and error contract on 2026-11-12. tCard was still on pimalaya-cli 0.2 and vcard-rs 0.5: under `--json` it printed the raw TOML or vCard, a failure carried no code, a refused command line printed nothing on stdout, and no command wrote the schemas of what it prints.

## What

**Dependencies.** vcard-rs 0.6 and pimalaya-cli 0.4 (through a path patch to ../cli until the family pins it), `rust-version` 1.89, every dependency declared with `default-features = false` and the features it uses, clap without `wrap_help` and `env`. Media base64 moves to base64-codec, the family codec, and the binary installs the detected CPU token.

**Output.** A verb prints its document as a `DocumentOutput`: in text the document itself, so pipes keep working; under `--json` `{contents}`, or the receipt `{path, size}` when it went to a file, which prints nothing in text since tCard runs as the composer of programs piping their own stdout.

**Failures.** A binary-local `ErrorCode` (`card-unreadable`, `template-unreadable`, `collision-undecided`, `embedded-unknown`, `file-unreadable`, `editor-missing`, `editor-failed`), given to a library failure from the `TcardError` it carries and raised by the editor round trip; `ErrorReport::eval_coded` prints it. Arguments go through `pimalaya_cli::clap::parse`, so a refused command line exits 2 coded `usage-invalid`; `apply - -` fails coded the same.

**Schemas.** `json-schema` writes one schema per verb and `tcard-error`; a test checks the registry against the command tree; the package installs them under share/schemas.

**vcard-rs 0.6 folds.** An edited 3.0 or 4.0 line longer than 75 octets now goes out folded; the template spec and the lib header say so.

## Judged not applicable

No REPL and no MCP server: tCard holds no account or connection a session would keep open, and its verbs open an editor. No configuration, so no `ConfigFlags`, no account flag, no welcome and no shipped sample. No events and no partial outcome (exit 3): every verb produces one document or fails. No `Columns`, `Fields` or `sanitize::text`: tCard prints no record or list, only the document it was asked for, as data.

## What this is not

Not the owed media extraction command, and not a version bump.
