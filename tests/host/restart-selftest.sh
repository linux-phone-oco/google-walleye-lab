#!/bin/sh
# restart-selftest: proves restart-check.sh on the two recorded journals (research walleye-c3-hang-how-it-stands): c2 left the bus at 180 s (pass), c3 never did (fail with a long watch, skip with none).
D=$(cd "$(dirname "$0")/../.." && pwd); M=$D/research/walleye-first-fastboot-boot/2026-11-18-c3-hang; bad=0
t() { want=$1; shift; sh "$D/tests/host/restart-check.sh" "$@" >/dev/null; rc=$?; if [ $rc -eq "$want" ]; then echo "ok   $*"; else echo "FAIL $*: exit $rc, want $want"; bad=1; fi; }
t 0 "$M/c2-host-journal.txt"
t 1 "$M/c2-host-journal.txt" 100
t 1 "$M/c3-host-journal.txt" 200 $(date -d "Nov 18 2026 15:42:41" +%s)
t 2 "$M/c3-host-journal.txt"
t 2 "$M/none.txt"
exit $bad
