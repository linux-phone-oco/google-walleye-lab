#!/bin/sh
# banner-loop-check (backlog walleye-command-loop, z6): does the z6 banner say the command loop is open on the ACM tty?
#   sh tests/host/banner-loop-check.sh [CAPTURE_FILE]
# With a file it reads that raw capture; without one it captures /dev/ttyACM0 with a plain `cat` for 60 s (nothing is sent to the phone) into a file it names. Prints the verdict line, then the last banner line.
# pass: banner-check.sh passes (>= 3 banner lines, beats increasing) AND the last banner line says loop=[started=yes starts=N ...] and the start count N is steady over the last 3 lines. Prints cmds=, last= and rc= as the banner says them.
# fail (exit 1): the loop not started, restarting, or no loop field; skip (exit 2): no capture file or no tty - not known yet.
H=$(cd "$(dirname "$0")" && pwd)
F=${1:-}
if [ -z "$F" ]; then
  TTY=${TTY:-/dev/ttyACM0}
  [ -c "$TTY" ] || { echo "skip: no $TTY (the phone's gadget is not up; not known yet)"; exit 2; }
  F=$(mktemp "${TMPDIR:-/tmp}/banner-loop-capture.XXXXXX"); echo "capturing $TTY for 60 s into $F"
  timeout 60 cat "$TTY" > "$F" 2>/dev/null || true
fi
[ -f "$F" ] || { echo "skip: no capture file $F (not known yet)"; exit 2; }
LAST=$(tr -d '\r' < "$F" | grep '^WALLEYE-BANNER beat=' | tail -n1)
if ! B=$(sh "$H/banner-check.sh" "$F"); then V="fail: no banner to judge the loop by - $B"; R=1
else
  case $LAST in
    *" loop=[started=yes starts="*)
      NS=$(tr -d '\r' < "$F" | grep '^WALLEYE-BANNER beat=' | tail -n3 | sed -n 's/.* loop=\[started=[a-z]* starts=\([0-9]*\) .*/\1/p' | sort -u | wc -l)
      if [ "$NS" -le 1 ]; then V="pass: the banner shows the command loop open, start count steady (${LAST#* loop=}) ($B)"; R=0; else V="fail: the loop is open but reopening (the start count changed within the last 3 banner lines)"; R=1; fi;;
    *" loop=[started=no"*) V="fail: the loop never opened the tty (the console block did not run, or the open failed)"; R=1;;
    *) V="fail: the last banner line has no loop state (a banner that is not z6's)"; R=1;;
  esac
fi
echo "$V"
echo "last banner: ${LAST:-none}"
exit $R
