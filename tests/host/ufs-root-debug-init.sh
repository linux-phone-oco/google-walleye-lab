#!/bin/sh
# The proof for the stage 2 DEBUG init (backlog walleye-stage2-debug-image), on the lab host with no phone: build/initramfs-root-debug/init is the UFS-root init plus blocks that act only with the word walleye.debug.
#   ufs-root-debug-init.sh WORK_DIR        (WORK_DIR holds busybox-x86_64 and out/b, as build/build-initramfs.sh wants)
# (D0) text: the init without its DEBUG blocks is build/initramfs-root/init, line for line (after the header); every call of a block is behind the word.
# (D1) QEMU, the word given, system_a there: the watchdog device (QEMU's i6300esb) is petted, the gadget comes up on the host kernel's dummy_hcd BEFORE the card is mounted, the host side of it leases, pings and speaks telnet to
#      172.16.43.1 and reads /tmp/walleye-debug.log there (the selftest of the initramfs), then the init goes on: the root's init runs, the debug copy of the log is in the root, the gadget is released first; the disk is unchanged.
# (D2) QEMU, the word given, the root partition NEVER appears: the init refuses and HOLDS (the gadget answers over the hold, no restart, the root's init never runs).
# (D3) the word NOT given: the whole proof of the root init (tests/host/ufs-root-init.sh) passes with this init in its place.
# Prints `pass|fail|skip walleye-ufs-root-debug-init DETAIL`; exit 0 only on pass; a skip is not a pass.
W=${1:-}
export GEN=${GEN:-}
HERE=$(cd "$(dirname "$0")/../.." && pwd)
DINIT=$HERE/build/initramfs-root-debug/init
say() { echo "$1 walleye-ufs-root-debug-init $2"; [ "$1" = pass ]; exit $?; }
[ -f "$DINIT" ] || say fail "no $DINIT"
# D0: the text
STRIPPED=$(sed '/^# DEBUG-BLOCK-BEGIN/,/^# DEBUG-BLOCK-END/d' "$DINIT")
ROOTN=$(( $(wc -l < "$HERE/build/initramfs-root/init") - 1 ))
[ "$(printf '%s\n' "$STRIPPED" | tail -n "$ROOTN")" = "$(tail -n "$ROOTN" "$HERE/build/initramfs-root/init")" ] || say fail "D0: without its DEBUG blocks the debug init is not build/initramfs-root/init"
[ "$(grep -c '^# DEBUG-BLOCK-BEGIN' "$DINIT")" = 2 ] && [ "$(grep -c '^# DEBUG-BLOCK-END' "$DINIT")" = 2 ] || say fail "D0: the init does not have exactly two DEBUG blocks"
[ "$(printf '%s\n' "$STRIPPED" | tail -n "$ROOTN" | grep -c 'walleye-debug\|wd_start\|debug_gadget\|walleye\.debug')" = 0 ] || say fail "D0: the debug code is outside its blocks"
for c in qemu-system-x86_64 sgdisk mke2fs sha256sum zstd modprobe unshare nsenter; do command -v "$c" >/dev/null || say skip "$c not installed: not known yet"; done
[ -n "$W" ] || say fail "usage: ufs-root-debug-init.sh WORK_DIR"
[ -x "$W/busybox-x86_64" ] || say skip "no $W/busybox-x86_64: not known yet"
KVER=$(ls /usr/lib/modules | grep -v lts | sort -V | tail -n1)
KERNEL=/usr/lib/modules/$KVER/vmlinuz
[ -f "$KERNEL" ] || say skip "no host kernel $KERNEL: not known yet"
modprobe -S "$KVER" --show-depends dummy_hcd >/dev/null 2>&1 || say skip "the host kernel has no dummy_hcd: not known yet"
T=$(mktemp -d) || say skip "no temp dir"; [ -n "${KEEP:-}" ] || trap 'rm -rf "$T"' EXIT
mkdir -p "$T/root/sbin" "$T/root/bin" "$T/root/proc" "$T/root/sys" "$T/root/dev" "$T/root/etc" "$T/root/run" "$T/root/tmp"
cp "$W/busybox-x86_64" "$T/root/bin/busybox"
for a in sh mount cat echo ls sync poweroff touch grep sleep head wc tr dmesg; do ln -s busybox "$T/root/bin/$a"; done
cat > "$T/root/sbin/init" <<'EOI'
#!/bin/sh
echo "ROOT-INIT-RAN pid=$$"
[ -s /.walleye/init-debug.log ] && echo "ROOT-HAS-DEBUG-LOG"
[ -s /.walleye/wd.pids ] && echo "ROOT-HAS-WD-PIDS"
[ ! -e /sys/kernel/config/usb_gadget/g1 ] && echo "ROOT-GADGET-RELEASED"
n1=$(dmesg | grep -c "walleye-debug: petting"); sleep 4; n2=$(dmesg | grep -c "walleye-debug: petting"); [ "$n1" = "$n2" ] && echo "ROOT-NO-PETTER-RESPAWN"
sync
poweroff -f
EOI
chmod 755 "$T/root/sbin/init"
mke2fs -q -t ext4 -L testroot -d "$T/root" "$T/root.ext4" 16M || say fail "mke2fs did not make the test rootfs"
disk() { # FILE NAME
  truncate -s 64M "$1"
  sgdisk -Z -n 1:2048:+8M -c 1:userdata -n 2:0:+20M -c 2:"$2" "$1" >/dev/null || say fail "sgdisk failed"
  start=$(sgdisk -i 2 "$1" | sed -n 's/^First sector: \([0-9]*\).*/\1/p')
  dd if="$T/root.ext4" of="$1" bs=512 seek="$start" conv=notrunc status=none || say fail "dd failed"
}
disk "$T/a.img" system_a
disk "$T/b.img" system_b
[ -n "$GEN" ] || for g in "$W/out/b/usr/gen_init_cpio" "$W"/out-ufsroot/b/usr/gen_init_cpio; do [ -x "$g" ] && GEN=$g; done; export GEN
INIT_DIR=$HERE/build/initramfs-root-debug DEBUG_GADGET=1 SELFTEST_KVER=$KVER "$HERE/build/build-initramfs.sh" ufs-root-selftest "$W" "$T/initramfs.cpio.gz" >"$T/build.log" 2>&1 || { tail -n5 "$T/build.log"; say fail "build-initramfs.sh ufs-root-selftest failed"; }
ACCEL=tcg; [ -w /dev/kvm ] && ACCEL=kvm
boot() { # DISK CMDLINE LOG
  timeout 300 qemu-system-x86_64 -machine accel=$ACCEL -cpu max -m 1024 -nographic -no-reboot \
    -kernel "$KERNEL" -initrd "$T/initramfs.cpio.gz" \
    -device i6300esb -action watchdog=poweroff -drive file="$1",if=virtio,format=raw \
    -append "console=ttyS0 panic=-1 loglevel=4 $2" > "$3" 2>&1 </dev/null
  tr -d '\r' < "$3" > "$3.t"
}
sha() { sha256sum "$1" | cut -d' ' -f1; }
want() { grep -aq -e "$2" "$1.t" || say fail "$3: no '$2' in $1 (tail: $(tail -n4 "$1.t" | tr '\n' '|'))"; }
not() { ! grep -aq -e "$2" "$1.t" || say fail "$3: '$2' is in $1"; }
A0=$(sha "$T/a.img")
boot "$T/a.img" "walleye.root=system_a androidboot.slot_suffix=_a walleye.booted-slot walleye.debug walleye.debugwait=2 walleye.selftest=1" "$T/d1.log"
want "$T/d1.log" "walleye-debug: petting /dev/watchdog0" "D1 (the watchdog device is petted)"
want "$T/d1.log" "walleye-debug: gadget bound to" "D1 (the gadget is bound)"
want "$T/d1.log" "SELFTEST PASS" "D1 (the host side leases, pings, speaks telnet and reads the log)"
GB=$(grep -an "walleye-debug: gadget bound to" "$T/d1.log.t" | head -n1 | cut -d: -f1); RT=$(grep -an "walleye-init: mounts-the-booted-slot" "$T/d1.log.t" | head -n1 | cut -d: -f1)
[ -n "$GB" ] && [ -n "$RT" ] && [ "$GB" -lt "$RT" ] || say fail "D1: the gadget is not bound before the init reaches the card (gadget line $GB, root line $RT)"
want "$T/d1.log" "walleye-debug: the debug gadget is released" "D1 (the gadget is released before the hand-over)"
want "$T/d1.log" "ROOT-INIT-RAN" "D1 (the init goes on to the root)"
want "$T/d1.log" "ROOT-HAS-DEBUG-LOG" "D1 (the log is copied to the root)"
want "$T/d1.log" "ROOT-HAS-WD-PIDS" "D1 (the petters' ends are handed to the root's hold script)"
want "$T/d1.log" "ROOT-GADGET-RELEASED" "D1 (the root finds no gadget of ours)"
want "$T/d1.log" "ROOT-NO-PETTER-RESPAWN" "D1 (the supervisor is gone at the hand-over: no petter is started in the root)"
[ "$(sha "$T/a.img")" = "$A0" ] || say fail "D1: the disk image changed"
boot "$T/b.img" "walleye.root=system_a androidboot.slot_suffix=_a walleye.booted-slot walleye.debug walleye.wait=3 walleye.hold=300 walleye.selftest=1" "$T/d2.log"
want "$T/d2.log" "walleye-ufs-root: refused: no partition named system_a" "D2 (the refusal)"
want "$T/d2.log" "walleye-debug: holding up to 300s" "D2 (a refusal holds)"
want "$T/d2.log" "SELFTEST PASS" "D2 (the gadget answers over the hold)"
not "$T/d2.log" "ROOT-INIT-RAN" "D2 (the root never appeared)"
INIT_DIR=$HERE/build/initramfs-root-debug sh "$HERE/tests/host/ufs-root-init.sh" "$W" > "$T/d3.out" 2>&1
grep -q '^pass ' "$T/d3.out" || say fail "D3: the root init's own proof does not pass with the debug init in its place (without the word): $(tail -n2 "$T/d3.out" | tr '\n' '|')"
say pass "the debug init is the root init plus two blocks behind walleye.debug; with the word the watchdog is petted and the gadget (172.16.43.1, telnet, the log) is up before the card is read, a refusal holds with the gadget answering, the gadget is released before the hand-over; without the word the root init's proof passes; the disk is unchanged ($A0)"
