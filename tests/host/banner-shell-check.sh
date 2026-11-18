#!/bin/sh
# banner-shell-check (backlog walleye-banner-shell, z5): does the z5 banner say a live shell is running on the ACM tty?
#   sh tests/host/banner-shell-check.sh [CAPTURE_FILE]
# With a file it reads that raw capture; without one it captures /dev/ttyACM0 with a plain `cat` for 60 s (nothing is ever sent to the phone) into a file it names.
# Prints the verdict line, then the last banner line.
# pass: banner-check.sh passes (>= 3 banner lines, beats increasing) AND the LAST banner line says shell=[started=yes ... alive=yes ...]. Always prints the last banner line.
# fail (exit 1): what the last line said (not started / dead / no shell field); skip (exit 2): no capture file or no tty - not known yet.
H=$(cd "$(dirname "$0")" && pwd)
F=${1:-}
if [ -z "$F" ]; then
  TTY=${TTY:-/dev/ttyACM0}
  [ -c "$TTY" ] || { echo "skip: no $TTY (the phone's gadget is not up; not known yet)"; exit 2; }
  F=$(mktemp "${TMPDIR:-/tmp}/banner-shell-capture.XXXXXX")
  echo "capturing $TTY for 60 s into $F"
  timeout 60 cat "$TTY" > "$F" 2>/dev/null || true
fi
[ -f "$F" ] || { echo "skip: no capture file $F (not known yet)"; exit 2; }
LAST=$(tr -d '\r' < "$F" | grep '^WALLEYE-BANNER beat=' | tail -n1)
if ! B=$(sh "$H/banner-check.sh" "$F"); then V="fail: no banner to judge the shell by - $B"; R=1
else
  case $LAST in
    *"shell=[started=yes "*"alive=yes "*)
      NS=$(tr -d "\r" < "$F" | grep "^WALLEYE-BANNER beat=" | tail -n3 | sed -n "s/.* shell=\[started=[a-z]* n=\([0-9]*\) .*/\1/p" | sort -u | wc -l)
      if [ "$NS" -le 1 ]; then V="pass: the banner shows a live shell, start count steady ($B)"; R=0; else V="fail: the shell is alive but restarting (the start count changed within the last 3 banner lines)"; R=1; fi;;
    *"shell=[started=yes "*) V="fail: the shell was started but is not alive in the last banner line"; R=1;;
    *"shell=[started=no"*) V="fail: the shell was never started (the console block did not run, or the tty never opened)"; R=1;;
    *) V="fail: the last banner line has no shell state (a banner without the z5 shell field)"; R=1;;
  esac
fi
echo "$V"
echo "last banner: ${LAST:-none}"
exit $R
