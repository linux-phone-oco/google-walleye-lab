#!/bin/sh
# walleye-bootchain-readback, run on the lab host BY THE KEEPER with Android up and adb root allowed (Android is throw-away): reads the Pixel 2's boot chain
# into DIR as files, a restore source kept outside every repository. READ ONLY: the only things sent to the phone are `ls`, `cat` and
# `dd if=... of=stdout`; nothing writes to it. For each name in bootchain-parts.list, `adb exec-out su -c 'dd if=/dev/block/by-name/NAME bs=4096'` goes to
# NAME.img (mode 600, in a DIR of mode 700); a name `ls /dev/block/by-name` does not list is reported and written to `missing`, never guessed.
# Also kept: partition-table.txt (`ls -l /dev/block/by-name`), proc-partitions.txt (`cat /proc/partitions`), sha256sums.
# It refuses to run: when `adb get-state` for the manifest serial is not `device`; when battery-ok.sh (the lab's battery rule) does not pass; when DIR
# is inside a git repository or already holds anything. Exit 0 when every name was read (then run bootchain-readback-check.sh DIR, and again on a
# second read: `check.sh DIR DIR2`); 1 on a refusal, a missing name or a failed read. Prints `pass|fail walleye-bootchain-readback host DETAIL`.
#
#   tests/host/bootchain-readback.sh [DIR]      (default: ~/lab-material/walleye/bootchain-DATE/, DATE = today)
# Environment: DEVICE_TOML (default: ../../device.toml), CALL_WAIT seconds for each short adb call (default 15).
HERE=$(cd "$(dirname "$0")" && pwd)
TOML=${DEVICE_TOML:-$HERE/../../device.toml}
LIST=$HERE/bootchain-parts.list
CW=${CALL_WAIT:-15}
DIR=${1:-$HOME/lab-material/walleye/bootchain-$(date +%Y-%m-%d)}
say() { printf "%s walleye-bootchain-readback host %s\n" "$1" "$2"; [ "$1" = pass ] && exit 0; exit 1; }
case $CW in *[!0-9]*|'') say fail "CALL_WAIT must be whole seconds";; esac
command -v timeout >/dev/null && command -v sha256sum >/dev/null || say fail "timeout(1) and sha256sum are needed"
SER=$(sed -n 's/^serial *= *"\([^"]*\)".*/\1/p' "$TOML" 2>/dev/null | head -n1)
[ -n "$SER" ] || say fail "no serial in $TOML"
adbq() { timeout -k 1 "$CW" adb -s "$SER" "$@"; }

state=$(adbq get-state 2>/dev/null | tr -d '\r')
[ "$state" = device ] || say fail "refused: adb shows ${state:-nothing} for $SER, not device: nothing read"
bat=$(sh "$HERE/battery-ok.sh" "$TOML"); [ $? -eq 0 ] || say fail "refused: the battery rule did not pass: $bat"
p=$DIR; while [ ! -d "$p" ]; do p=$(dirname "$p"); done   # the first ancestor that exists
if git -C "$p" rev-parse --git-dir >/dev/null 2>&1; then say fail "refused: $DIR is inside a git repository; the read-back stays outside every repository"; fi
if [ -e "$DIR" ] && [ -n "$(ls -A "$DIR" 2>/dev/null)" ]; then say fail "refused: $DIR already holds files; never overwritten"; fi

umask 077
mkdir -p "$DIR" && chmod 700 "$DIR" || say fail "cannot create $DIR"
names=$(adbq shell ls /dev/block/by-name | tr -d '\r') || say fail "ls /dev/block/by-name failed"
[ -n "$names" ] || say fail "the phone listed no names in /dev/block/by-name"
adbq shell ls -l /dev/block/by-name | tr -d '\r' > "$DIR/partition-table.txt" && [ -s "$DIR/partition-table.txt" ] || say fail "ls -l of by-name failed"
adbq shell cat /proc/partitions | tr -d '\r' > "$DIR/proc-partitions.txt" && [ -s "$DIR/proc-partitions.txt" ] || say fail "cat /proc/partitions failed"

# adbd as root (adb root) has no su: read directly; as shell, through su
if [ "$(adbq shell id -u 2>/dev/null | tr -d "\r")" = 0 ]; then SU=""; else SU="su -c"; fi
bad=0
: > "$DIR/sha256sums"
for n in $(cat "$LIST"); do
  if ! printf '%s\n' "$names" | grep -qx "$n"; then echo "missing $n: not in /dev/block/by-name"; echo "$n" >> "$DIR/missing"; bad=$((bad + 1)); continue; fi
  # dd's own report goes to /dev/null on the phone: exec-out would mix stderr into the image
  if ! timeout -k 1 600 adb -s "$SER" exec-out $SU "dd if=/dev/block/by-name/$n bs=4096 2>/dev/null" > "$DIR/$n.img"; then
    echo "failed $n: the read did not finish"; rm -f "$DIR/$n.img"; bad=$((bad + 1)); continue; fi
  (cd "$DIR" && sha256sum "$n.img") >> "$DIR/sha256sums"
  echo "read $n $(stat -c %s "$DIR/$n.img") bytes"
done
chmod 600 "$DIR"/* 2>/dev/null
[ "$bad" -eq 0 ] || say fail "$bad name(s) missing or failed; $DIR kept for the record, not complete"
chk=$(sh "$HERE/bootchain-readback-check.sh" "$DIR") || { echo "$chk"; say fail "read, but the check on $DIR fails (a failed su writes its error into the image)"; }
say pass "$(wc -l < "$LIST") partitions read into $DIR and checked at their measured sizes"
