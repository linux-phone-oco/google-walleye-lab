#!/bin/sh
# banner-check (backlog walleye-try-banner): does a raw capture of /dev/ttyACM0 (plain `cat`, nothing else) hold the banner init's lines?
#   sh tests/host/banner-check.sh CAPTURE_FILE
# pass when the capture has at least 3 WALLEYE-BANNER lines whose beat numbers strictly increase; fail (exit 1) with what it saw otherwise; skip (exit 2) when there is no capture file.
# 0 bytes: the reader or the host is silent, not the banner image proven bad - see the fail line.
F=${1:?usage: banner-check.sh CAPTURE_FILE}
[ -f "$F" ] || { echo "skip: no capture file $F (not known yet)"; exit 2; }
BYTES=$(wc -c < "$F")
N=$(tr -d '\r' < "$F" | grep -c '^WALLEYE-BANNER beat=[0-9][0-9]* ')
UP=$(tr -d '\r' < "$F" | sed -n 's/^WALLEYE-BANNER beat=\([0-9][0-9]*\) .*/\1/p' | awk 'NR>1 && $1<=p {bad=1} {p=$1} END {print bad+0}')
if [ "$N" -ge 3 ] && [ "$UP" -eq 0 ]; then
  echo "pass: $N banner lines in $BYTES bytes, beats increasing; last: $(tr -d '\r' < "$F" | grep '^WALLEYE-BANNER' | tail -n1)"
  exit 0
fi
echo "fail: $N banner lines in $BYTES bytes (need >= 3 with increasing beats; non-increasing=$UP)"
exit 1
