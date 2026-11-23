#!/bin/sh
# The proof for the stage 2 PRODUCTION init (backlog walleye-stage2-production-image), on the lab host with no phone: build/initramfs-root-production/init is the UFS-root init plus two PRODUCTION-BLOCKs (the petters, the hold script that ends them).
#   ufs-root-production-init.sh WORK_DIR        (WORK_DIR holds busybox-x86_64, as tests/host/ufs-root-init.sh wants)
# (P0) text: without its two blocks and its header the init is build/initramfs-root/init, line for line; the first block (the petters) runs before the slot, the root name or the card is looked at; nothing of a debug init is in it:
#      no gadget, configfs, telnet, udhcpd, debug word, debug log or dyndbg.
# (P1) the whole proof of the root init (tests/host/ufs-root-init.sh: QEMU, read-only mount, overlay, every refusal, the petter held open after the hand-over, the restart after the hold) passes with this init in its place.
# INIT_DIR (default build/initramfs-root-production): another init to judge, so the test is seen failing on one that lacks the blocks.
# Prints `pass|fail|skip walleye-ufs-root-production-init DETAIL`; exit 0 only on pass; a skip is not a pass.
W=${1:-}
export GEN=${GEN:-}
HERE=$(cd "$(dirname "$0")/../.." && pwd)
PDIR=${INIT_DIR:-$HERE/build/initramfs-root-production}
PINIT=$PDIR/init
say() { echo "$1 walleye-ufs-root-production-init $2"; [ "$1" = pass ]; exit $?; }
[ -f "$PINIT" ] || say fail "no $PINIT"
[ "$(grep -c '^# PRODUCTION-BLOCK-BEGIN' "$PINIT")" = 2 ] && [ "$(grep -c '^# PRODUCTION-BLOCK-END' "$PINIT")" = 2 ] || say fail "P0: the init does not have exactly two PRODUCTION blocks"
STRIPPED=$(sed '/^# PRODUCTION-BLOCK-BEGIN/,/^# PRODUCTION-BLOCK-END/d' "$PINIT")
[ "$(sed -n '2,4p' "$PINIT" | grep -c '^#')" = 3 ] || say fail "P0: lines 2 to 4 of the init are not its three header comment lines"
[ "$(printf '%s\n' "$STRIPPED" | sed 1,4d)" = "$(sed 1d "$HERE/build/initramfs-root/init")" ] || say fail "P0: without its PRODUCTION blocks and its header the init is not build/initramfs-root/init, line for line"
B2=$(sed -n '/^# PRODUCTION-BLOCK-BEGIN (2/,/^# PRODUCTION-BLOCK-END/p' "$PINIT")
printf '%s\n' "$B2" | grep -q 'kill \$(cat /tmp/wd.supervisor)' || say fail "P0: block 2 does not end the supervisor"
printf '%s\n' "$B2" | grep -q 'busybox kill -9 ' || say fail "P0: the hold script does not end the petters with SIGKILL (SIGTERM disarms the watchdog)"
printf '%s\n' "$B2" | grep -q 'kill -0 .* || refuse' || say fail "P0: block 2 does not refuse to hand over with a dead petter"
sed -n '/^# PRODUCTION-BLOCK-BEGIN (1/,/^# PRODUCTION-BLOCK-END/p' "$PINIT" | grep -q 'kill -9 \$(cat /tmp/wd.pids)' || say fail "P0: the petting has no deadline (the supervisor never ends the petters)"
[ "$(grep -v '^#' "$PINIT" | grep -c -e usb_gadget -e configfs -e telnetd -e udhcpd -e walleye.debug -e walleye-debug -e dynamic_debug -e ttyGS)" = 0 ] || say fail "P0: the init carries debug code (gadget, telnet, debug word or log)"
FIRST=$(grep -n '^wd_supervise &' "$PINIT" | head -n1 | cut -d: -f1)
[ -n "$FIRST" ] || say fail "P0: the init never starts the petters"
for later in 'ROOT=\$(arg walleye.root)' 'say "start, kernel' 'mount -t ext4' 'blockdev --setro'; do
  L=$(grep -n -e "$later" "$PINIT" | head -n1 | cut -d: -f1)
  [ -n "$L" ] && [ "$FIRST" -lt "$L" ] || say fail "P0: the petters (line $FIRST) do not start before '$later' (line $L)"
done
WANT_OK="walleye-ufs-root: petting /dev/watchdog0" INIT_DIR=$PDIR sh "$HERE/tests/host/ufs-root-init.sh" "$W" > "${TMPDIR:-/tmp}/walleye-production-p1.$$" 2>&1; rc=$?
P1=$(tail -n1 "${TMPDIR:-/tmp}/walleye-production-p1.$$"); rm -f "${TMPDIR:-/tmp}/walleye-production-p1.$$"
[ $rc -eq 0 ] || say fail "P1: the root init's proof does not pass with this init: $P1"
say pass "the production init is the root init plus the petters (before the slot, the name and the card) and the hold that ends them, with no gadget, telnet or debug word; the root init's whole proof passes with it in its place"
