#!/bin/sh
# walleye-dmesg-clean, run on the lab host after mainline-shell-read.sh: is the dmesg of our mainline kernel free of the first map's probe faults?
# Reads the files the reader saved under DIR (dmesg.txt, devices-deferred.txt) and checks, each a line of the item's proof:
#   1. devices-deferred.txt is there and names only the known vendor-overlay nodes (serial@c16f000, c179000.i2c, bt_wcn3990, mnh-sm), never 5040000.iommu;
#   2. no "probe with driver ... failed with error -110" (the arm-smmu 5040000 deferred probe timeout);
#   3. no "OF: ... could not find phandle" (the overlay's i2c@c179000 and serial@c16f000 point at stub labels).
#   4. no "gpu_gx status stuck" WARN (the GPU smmu is disabled in the dtb, image z7).
# Prints `pass|fail|skip walleye-dmesg-clean host DETAIL` (a fail names every fault it counted); exit 0 only on pass. Reads only.
# Usage: dmesg-clean-check.sh DIR
D=${1:?usage: dmesg-clean-check.sh DIR (the output directory of the reader)}
say() { echo "$1 walleye-dmesg-clean host $2"; [ "$1" = pass ]; exit $?; }
[ -s "$D/dmesg.txt" ] || say skip "no dmesg in $D: not known yet"
[ -f "$D/devices-deferred.txt" ] || say skip "no devices-deferred.txt in $D: not known yet"
# the reader cats the file, mounts debugfs if that failed, and cats again: one "can't open" is the first try, two mean debugfs never mounted (not an empty list)
[ "$(grep -c "can't open '/sys/kernel/debug/devices_deferred'" "$D/devices-deferred.txt")" -ge 2 ] && say skip "debugfs not mounted, devices_deferred unread: not known yet"
deferred=$(grep -vE "^(cat |rnel/debug|[^	]* *echo |.*wmsr-[0-9]+)" "$D/devices-deferred.txt" | grep '	' | cut -f1 | grep -vcE '^[^ ]*(c16f000\.serial|c179000\.i2c|bt_wcn3990|mnh-sm)')
t110=$(grep -cE 'probe with driver [^ ]+ failed with error -110' "$D/dmesg.txt")
phandle=$(grep -c 'OF: .*could not find phandle' "$D/dmesg.txt")
gpux=$(grep -c 'gpu_gx.*status stuck' "$D/dmesg.txt")
bad=
[ "$deferred" -eq 0 ] || bad="$bad $deferred deferred device(s) ($(grep '	' "$D/devices-deferred.txt" | cut -f1 | grep -vE '^[^ ]*(c16f000\.serial|c179000\.i2c|bt_wcn3990|mnh-sm)' | tr '\n' ' '));"
[ "$t110" -eq 0 ] || bad="$bad $t110 probe timeout(s) -110 ($(grep -E 'failed with error -110' "$D/dmesg.txt" | sed 's/^[^]]*\] //' | head -n 2 | tr '\n' ' '));"
[ "$phandle" -eq 0 ] || bad="$bad $phandle phandle line(s) ($(grep 'could not find phandle' "$D/dmesg.txt" | sed 's/.*OF: //; s/:.*//' | tr '\n' ' '));"
[ "$gpux" -eq 0 ] || bad="$bad $gpux gpu_gx stuck WARN line(s);"
[ -z "$bad" ] || say fail "$bad"
say pass "devices_deferred only known overlay nodes, no gpu_gx WARN, no -110 probe timeout, no phandle line; dmesg $(wc -l < "$D/dmesg.txt") lines"
