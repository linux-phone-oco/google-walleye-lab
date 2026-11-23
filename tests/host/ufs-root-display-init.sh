#!/bin/sh
# The proof for the UFS-root init with the display (backlog walleye-install-ufs-root-display-image), run on the lab host with no phone. Two parts:
# (1) text: build/initramfs-root-display/init is the proven root init (build/initramfs-root/init) plus exactly one difference - the display block between its DISPLAY-BLOCK-BEGIN and -END lines (with the four comment
#     lines at the top that say so); both say "handing over" before the moves of /proc /sys /dev; the block touches no block device and no mount;
# (2) behaviour: tests/host/ufs-root-init.sh, every case of it, run against the display init with a fake module list (two modules that cannot load) and a delay of 2 s: the delay is waited, both lines are tried
#     and logged, the root is still mounted read-only under the overlay, the refusals still refuse.
#   ufs-root-display-init.sh WORK_DIR        (WORK_DIR holds busybox-x86_64 and out/b, as tests/host/ufs-root-init.sh wants)
# Prints `pass|fail|skip walleye-ufs-root-display-init DETAIL`; exit 0 only on pass; a skip is not a pass.
W=${1:-}
HERE=$(cd "$(dirname "$0")/../.." && pwd)
say() { echo "$1 walleye-ufs-root-display-init $2"; [ "$1" = pass ]; exit $?; }
[ -n "$W" ] || say fail "usage: ufs-root-display-init.sh WORK_DIR"
R=$HERE/build/initramfs-root/init
D=$HERE/build/initramfs-root-display/init
[ -s "$R" ] && [ -s "$D" ] || say fail "no $R or $D"
T=$(mktemp -d) || say skip "no temp dir"; [ -n "${KEEP:-}" ] || trap 'rm -rf "$T"' EXIT
cp "$R" "$T/root.txt"
awk 'NR>=2 && NR<=5 { next } /^# the display \(DISPLAY-BLOCK-BEGIN\)/ { skip=1 } !skip { print } /^# \(DISPLAY-BLOCK-END\)/ { skip=0 }' "$D" > "$T/disp.txt"
sed -n 2,5p "$D" | grep -q -v '^#' && say fail "the first four lines after the shebang are not all comments"
[ "$(grep -c DISPLAY-BLOCK-BEGIN "$D")/$(grep -c DISPLAY-BLOCK-END "$D")" = 1/1 ] || say fail "the display block is not marked once"
cmp -s "$T/root.txt" "$T/disp.txt" || say fail "the display init is not the root init plus the display block: $(diff "$T/root.txt" "$T/disp.txt" | head -n6 | tr '\n' '|')"
for i in "$R" "$D"; do [ "$(grep -c '^say "handing over' "$i")" -eq 1 ] || say fail "$i says 'handing over' not once"; done
for i in "$R" "$D"; do [ "$(grep -n '^say "handing over' "$i" | cut -d: -f1)" -lt "$(grep -n '^mount --move /proc' "$i" | cut -d: -f1)" ] || say fail "$i: 'handing over' is said after the moves"; done
GUARD=$(grep -n "^  \[ \"\$ROOT\" != \"system\$SLOT\" \]" "$D" | cut -d: -f1); BEGIN=$(grep -n DISPLAY-BLOCK-BEGIN "$D" | cut -d: -f1); PART=$(grep -n "^# the partition: exactly one block device" "$D" | cut -d: -f1)
[ -n "$GUARD" ] && [ -n "$BEGIN" ] && [ -n "$PART" ] && [ "$GUARD" -lt "$BEGIN" ] && [ "$BEGIN" -lt "$PART" ] || say fail "the display block is not between the last guard and the wait for the partition (guard $GUARD, block $BEGIN, partition $PART)"
sed -n '/DISPLAY-BLOCK-BEGIN/,/DISPLAY-BLOCK-END/p' "$D" | grep -v '^#' | grep -q -w -e mount -e mkfs -e dd -e blockdev -e umount && say fail "the display block mounts or touches a device"
# the behaviour: every case of the root init's proof, on this init, with a fake display
mkdir -p "$T/df" && printf 'insmod /lib/modules/not-there-1.ko\ninsmod /lib/modules/not-there-2.ko\n' > "$T/df/display-modules.list" && printf '2\n' > "$T/df/display-delay"
r=$(INIT_DIR=$HERE/build/initramfs-root-display DISPLAY_FILES=$T/df sh "$HERE/tests/host/ufs-root-init.sh" "$W" | head -n1)
case $r in pass*) ;; skip*) say skip "the root init's proof skipped: $r" ;; *) say fail "the root init's proof on the display init: $r" ;; esac
say pass "the display init is the root init plus the display block, nothing else; every case of the root init's proof passes on it, with the display delay waited and a failing module line logged"
