# part-compare lists

One file per part: `PART.list`, used by `tests/host/part-compare.sh capture PART android|ours` and compared by `part-compare.sh diff`.

Adding a part: copy `touch.list`, change the commands. A line is `NAME<TAB>COMMAND` (a real tab); the command is a POSIX sh one-liner that runs on the phone (Android's toybox and our busybox: no bashisms), `#` lines and blank lines are skipped.
The NAME's prefix picks the normaliser (see the header of part-compare.sh): `gpio`, `pinmux`, `pinconf`, `regulator`, `clk`, `dt`, `dmesg`; any other name is a generic sorted `key=value` / whole-line set.
Rules for a list: read-only commands only; filter with `grep` on the phone so the capture stays small; give both sides the same command (the same file on both kernels).
Known limit: a device-tree property that holds a phandle (`gpios`, `*-supply`) is a number that differs between two trees, so it shows as 'different value' even when both point to the same thing; read those by hand against the `regulator` and `gpio` answers.
