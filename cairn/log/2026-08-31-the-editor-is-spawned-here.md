---
cairn: log
change: the-editor-is-spawned-here
landed: 2026-08-31
---

# The editor is spawned here, and named when it cannot be found

tCard opened `$EDITOR` through the [edit](https://crates.io/crates/edit) crate. The resolution was the problem, not the round trip: with neither `$VISUAL` nor `$EDITOR` set, that crate walks a fallback list ending in `xdg-open`, `gnome-open`, `kde-open` and a bare `open` (edit-0.1.5/src/lib.rs:49-72). Those are file openers, not editors. They hand the `.toml` to whatever the desktop associates with it and return while the window is still up, so tCard read back a document nobody had touched and wrote the card out exactly as it went in.

Nothing reported that, and one layer up it is worse than nothing: cardamum now reads a card handed back untouched as an edit given up on, so an unset `$EDITOR` on a desktop became a silently abandoned edit in a program that never asked for a fallback.

**The resolution is three lines and a refusal** (src/cli/editor.rs): `--editor`, then `$VISUAL`, then `$EDITOR`, then `No editor found; set $VISUAL or $EDITOR, or pass --editor <COMMAND>`. A legible failure beats a mystery edit, and tCard picks no editor on anyone's behalf.

**`--editor <COMMAND>`** (src/cli/args.rs) is a shared argument on `edit` and `merge`, the twin of cardamum's `--composer`: it makes the round trip scriptable, and a broken `$EDITOR` recoverable without touching the environment.

**The spawn is ours**: `tcard-<uuid>.toml` in the temporary directory, the command line split on whitespace so `code --wait` carries its argument, the path appended last, the three streams inherited, the file read back on exit. The `edit` dependency and its feature entry are gone.

**The buffer outlives the run that could not use it**: a document that does not fold back and is not re-edited keeps its file, named in the error as `Cannot fold back <path>`, and so does an editor exiting non-zero. An editor that could not be spawned at all removes it, the file holding nothing but what tCard wrote a moment earlier, and a fold that succeeded removes it too.

Capabilities moved: editor, a new capability file, four requirements. Nothing in template, merge or api changed.

Verified with `--editor` winning over `$VISUAL`, `$VISUAL` winning over `$EDITOR`, `$EDITOR` alone, an editor carrying arguments, an unset pair failing and spawning nothing, a missing program removing the file, a non-zero exit keeping and naming it, and a successful fold leaving the temporary directory clean. 65 tests green on the full feature set, clippy clean.
