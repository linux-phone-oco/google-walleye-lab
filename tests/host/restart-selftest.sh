#!/bin/sh
# restart-selftest: proves restart-check.sh on the two recorded journals (research walleye-c3-hang-how-it-stands): c2 left the bus at 180 s (pass), c3 never did.
# With no disconnect the journal is one witness: the verdict of phone-state.sh (a fake here, PHONE_STATE) decides - android/fastboot = restarted (the c3 case: gadget listed, adb says device), mainline-answering = no restart path, anything else = NOT KNOWN.
D=$(cd "$(dirname "$0")/../.." && pwd); M=$D/research/walleye-first-fastboot-boot/2026-11-18-c3-hang; bad=0
T=$(mktemp -d) || exit 1; trap 'rm -rf "$T"' EXIT
fake() { printf '#!/bin/sh\necho "witness adb x"\necho "verdict %s fake"\n' "$1" > "$T/ps-$1.sh"; }
for v in android fastboot mainline-answering mainline-silent not-known gone; do fake $v; done
t() { want=$1; ps=$2; shift 2; PHONE_STATE=$ps sh "$D/tests/host/restart-check.sh" "$@" >"$T/out"; rc=$?; if [ $rc -eq "$want" ]; then echo "ok   $*"; else echo "FAIL $*: exit $rc, want $want: $(cat "$T/out")"; bad=1; fi; }
NOW=$(date -d "Nov 18 2026 15:42:41" +%s)
t 0 "$T/ps-gone.sh" "$M/c2-host-journal.txt"
t 1 "$T/ps-gone.sh" "$M/c2-host-journal.txt" 100
t 1 "$T/ps-mainline-answering.sh" "$M/c3-host-journal.txt" 200 $NOW
t 0 "$T/ps-android.sh" "$M/c3-host-journal.txt" 200 $NOW
t 0 "$T/ps-fastboot.sh" "$M/c3-host-journal.txt" 200 $NOW
t 2 "$T/ps-mainline-silent.sh" "$M/c3-host-journal.txt" 200 $NOW
t 2 "$T/ps-not-known.sh" "$M/c3-host-journal.txt" 200 $NOW
t 2 "$T/ps-gone.sh" "$M/c3-host-journal.txt" 200 $NOW
t 2 /nonexistent/ps.sh "$M/c3-host-journal.txt" 200 $NOW
t 2 "$T/ps-android.sh" "$M/c3-host-journal.txt"   # no NOW: a recorded journal is never passed by today's phone
t 2 "$T/ps-android.sh" "$M/none.txt"
PHONE_STATE=$T/ps-mainline-silent.sh sh "$D/tests/host/restart-check.sh" "$M/c3-host-journal.txt" 200 $NOW | grep -q 'NOT KNOWN' || { echo "FAIL silent case does not say NOT KNOWN"; bad=1; }
exit $bad
