---
cairn: change
id: the-editor-is-spawned-here
status: landed
created: 2026-08-31
---

# The editor is spawned here, and named when it cannot be found

## Why

tCard opens `$EDITOR` through the [edit](https://crates.io/crates/edit) crate, which resolves an editor, writes a temporary file, spawns and reads it back. The resolution is the problem. When neither `$VISUAL` nor `$EDITOR` is set, the crate walks a list of fallbacks, and that list ends in `xdg-open`, `gnome-open` and `kde-open` on Linux and in a bare `open` on macOS: generic file openers, not editors. They hand the `.toml` to whatever the desktop associates with it, and they do not reliably block. The command returns while the window is still open, tCard reads back a document nobody has touched yet, and the card comes out exactly as it went in.

Nothing reports that. A round trip that changed nothing is indistinguishable from an edit someone thought better of, which is precisely the reading cardamum now takes: a composer handing back the seed untouched means no. So an unset `$EDITOR` on a desktop turns into a silently abandoned edit, one layer up, in a program that never asked for a fallback.

There is no way out of it either. tCard reads no configuration and offers no flag, so the chain is the whole of the policy.

The rest of what the crate does is a temporary file, a spawn with the streams inherited and a read back, which is thirty lines against `std`. Cardamum already writes them for the same purpose, and the two programs are now two halves of the same round trip.

## What

**Resolve `$VISUAL`, then `$EDITOR`, and stop.** No fallback list, no file opener. When neither is set the command SHALL say so and name the two variables and the flag, which is a legible failure rather than a mystery edit.

**`--editor <COMMAND>`**, on `edit` and on `merge`, naming the command for one invocation. It is what makes tCard scriptable, what makes a broken `$EDITOR` recoverable without touching the environment, and the twin of cardamum's `--composer`.

**The spawn is ours**: a `tcard-<uuid>.toml` in the temporary directory, the command spawned on its path with stdin, stdout and stderr inherited, and the file read back when it exits. Capturing nothing is the point, the same as it is in cardamum: an editor handed a pipe hangs or writes where nothing reads.

**A buffer that cannot be folded back is kept and named.** The re-open loop stays, and the file it looped over survives a reader who declines it: what someone typed is worth more than a temporary path.

**The `edit` dependency goes**, and with it the only thing standing between tCard and the editor it was told to run.

## What this is not

This is not a configuration file. tCard still reads none: the environment names the editor, and a flag overrides it for one run.

It is not editor detection either. A command that does not block is still a command that does not block, and `code --wait` is still the caller's business. What changes is that tCard no longer picks such a command on someone's behalf.
