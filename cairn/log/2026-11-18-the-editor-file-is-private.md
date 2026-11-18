---
cairn: log
change: the-editor-file-is-private
landed: 2026-11-18
---

# The editor file is private

The temporary file tCard hands the editor (`tcard-<uuid>.toml`) is created through pimalaya-cli 0.6.1's `file::private_options`: new, 0600 on Unix whatever the umask, a file or a symbolic link already at its path refused. It was written with `fs::write`, readable by every user under a 022 umask, in a directory every user shares. Cardamum's composer file got the same fix (its CAR-14) the same day; tCal owes the twin. Capability **editor** moved.
