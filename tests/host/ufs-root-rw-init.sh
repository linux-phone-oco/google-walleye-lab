#!/bin/sh
# The proof for the rw-write init (backlog walleye-rw-write-image), run on the lab host with no phone: build/initramfs-root-rw-write/init is the production init plus RW-WRITE blocks that act only with the word walleye.rw.
#   ufs-root-rw-init.sh WORK_DIR        (WORK_DIR holds busybox-x86_64, as tests/host/ufs-root-init.sh wants; GEN: a built tree's usr/gen_init_cpio)
# (R0) text: with its RW-WRITE blocks cut the init is build/initramfs-root-production/init, line for line; the rw branch mounts `-o rw,nodiscard` and has no overlay, mkfs, fsck, discard, remount, ufs_bsg or purge.
# QEMU (x86_64 busybox stands in for the arm64 one, the init is the same file), a GPT disk whose partition system_a holds a loop-made test rootfs (mke2fs -d), each case one boot:
# (R1) the word walleye.rw: the root is ext4 rw (no overlay), the root's /sbin/init runs, a write lands in the partition (read back with debugfs from the disk after the run), /.walleye is a tmpfs (no busybox copy on the card), the decoy partition is byte for byte what it was;
# (R2) no word: the root is the read-only overlay and the whole disk is byte for byte what it was; (R3) the word with: only a decoy system_b (no partition of the name), the running slot, a name outside system_a/system_b, no slot, a root that is not the booted slot's:
# each refuses, the root's init does not run and the whole disk is byte for byte what it was.
# INIT_DIR: another init to judge, so the test is seen failing on one without the blocks. Prints `pass|fail|skip walleye-ufs-root-rw-init DETAIL`; exit 0 only on pass; a skip is not a pass.
W=${1:-}
export GEN=${GEN:-}
HERE=$(cd "$(dirname "$0")/../.." && pwd)
PDIR=${INIT_DIR:-$HERE/build/initramfs-root-rw-write}
INIT=$PDIR/init
export INIT_DIR=$PDIR
say() { echo "$1 walleye-ufs-root-rw-init $2"; [ "$1" = pass ]; exit $?; }
for c in qemu-system-x86_64 sgdisk mke2fs sha256sum zstd debugfs; do command -v "$c" >/dev/null || say skip "$c not installed: not known yet"; done
[ -n "$W" ] || say fail "usage: ufs-root-rw-init.sh WORK_DIR"
[ -x "$W/busybox-x86_64" ] || say skip "no $W/busybox-x86_64: not known yet"
[ -f "$INIT" ] || say fail "no $INIT"
KVER=$(ls /usr/lib/modules | grep -v lts | sort -V | tail -n1)
KERNEL=/usr/lib/modules/$KVER/vmlinuz
[ -f "$KERNEL" ] || say skip "no host kernel $KERNEL: not known yet"
# R0: the text
[ "$(grep -c '^# RW-WRITE-BLOCK-BEGIN' "$INIT")" = "$(grep -c '^# RW-WRITE-BLOCK-END' "$INIT")" ] && [ "$(grep -c '^# RW-WRITE-BLOCK-BEGIN' "$INIT")" -ge 1 ] || say fail "R0: the RW-WRITE blocks are not paired, or there are none"
[ "$(sed '/^# RW-WRITE-BLOCK-BEGIN/,/^# RW-WRITE-BLOCK-END/d' "$INIT")" = "$(cat "$HERE/build/initramfs-root-production/init")" ] || say fail "R0: without its RW-WRITE blocks the init is not build/initramfs-root-production/init, line for line"
RWB=$(sed -n '/^# RW-WRITE-BLOCK-BEGIN (the read-write root)/,/^# RW-WRITE-BLOCK-END/p' "$INIT" | grep -v '^#')
printf '%s\n' "$RWB" | grep -q -e 'mount -t ext4 -o rw,nodiscard "/dev/\$found" /newroot ' || say fail "R0: the rw branch does not mount the root ext4 rw,nodiscard onto /newroot"
[ "$(printf '%s\n' "$RWB" | grep -c -w -e overlay -e mkfs -e mke2fs -e fsck -e e2fsck -e dd -e fstrim -e blkdiscard -e remount -e ufs_bsg -e purge -e setro)" = 0 ] || say fail "R0: the rw branch has an overlay, mkfs, fsck, discard, remount, ufs_bsg, purge or setro"
[ "$(grep -v '^#' "$INIT" | grep -c -e 'mount -t ext4 -o rw')" = 1 ] || say fail "R0: the init has more than one read-write mount of the card"
T=$(mktemp -d) || say skip "no temp dir"; [ -n "${KEEP:-}" ] || trap 'rm -rf "$T"' EXIT
mkdir -p "$T/root/sbin" "$T/root/bin" "$T/root/proc" "$T/root/sys" "$T/root/dev" "$T/root/etc" "$T/root/run" "$T/root/tmp"
cp "$W/busybox-x86_64" "$T/root/bin/busybox"
for a in sh mount cat echo ls sync poweroff touch grep sleep head wc tr dmesg; do ln -s busybox "$T/root/bin/$a"; done
cat > "$T/root/sbin/init" <<'EOI'
#!/bin/sh
echo "ROOT-INIT-RAN pid=$$"
echo "ROOT-MOUNT $(grep ' / ' /proc/mounts | head -n1)"
echo "ROOT-WALLEYE-DIR $(grep ' /.walleye ' /proc/mounts | head -n1)"
echo "ROOT-RUN $(grep ' /run ' /proc/mounts | head -n1)"
echo written-by-the-root-init > /written-in-root && echo "ROOT-WRITE-OK" || echo "ROOT-WRITE-FAILED"
[ -e /walleye-test-marker ] && echo "ROOT-HAS-MARKER"
[ -e /.walleye/hold.sh ] && echo "ROOT-HAS-HOLD-SCRIPT"
grep -q "^/.walleye/busybox sync$" /.walleye/hold.sh && echo "ROOT-HOLD-SYNCS"
sync
poweroff -f
EOI
chmod 755 "$T/root/sbin/init"
echo marker > "$T/root/walleye-test-marker"
# the root of the image has an OpenRC shutdown level with mount-ro (Alpine: openrc); the rw init adds walleye-end and the two links
mkdir -p "$T/root/etc/init.d" "$T/root/etc/runlevels/shutdown"; printf "#!/sbin/openrc-run\n" > "$T/root/etc/init.d/mount-ro"; chmod 755 "$T/root/etc/init.d/mount-ro"
mke2fs -q -t ext4 -L testroot -d "$T/root" "$T/root.ext4" 16M || say fail "mke2fs did not make the test rootfs"
disk() { # FILE NAME: a GPT disk of 64 MiB with the test rootfs in the partition NAME and an empty decoy `userdata`
  truncate -s 64M "$1"
  sgdisk -Z -n 1:2048:+8M -c 1:userdata -n 2:0:+20M -c 2:"$2" "$1" >/dev/null || say fail "sgdisk failed"
  start=$(sgdisk -i 2 "$1" | sed -n 's/^First sector: \([0-9]*\).*/\1/p')
  dd if="$T/root.ext4" of="$1" bs=512 seek="$start" conv=notrunc status=none || say fail "dd failed"
}
disk "$T/a.img" system_a
disk "$T/b.img" system_b
[ -n "$GEN" ] || for g in "$W/out/b/usr/gen_init_cpio" "$W"/out-ufsroot/b/usr/gen_init_cpio; do [ -x "$g" ] && GEN=$g; done; export GEN
SELFTEST_KVER=$KVER "$HERE/build/build-initramfs.sh" ufs-root-selftest "$W" "$T/initramfs.cpio.gz" >"$T/build.log" 2>&1 || { tail -n5 "$T/build.log"; say fail "build-initramfs.sh ufs-root-selftest failed"; }
ACCEL=tcg; [ -w /dev/kvm ] && ACCEL=kvm
boot() { # DISK CMDLINE LOG
  timeout 240 qemu-system-x86_64 -machine accel=$ACCEL -cpu max -m 1024 -nographic -no-reboot \
    -kernel "$KERNEL" -initrd "$T/initramfs.cpio.gz" \
    -device i6300esb -action watchdog=poweroff -drive file="$1",if=virtio,format=raw \
    -append "console=ttyS0 panic=-1 loglevel=4 $2" > "$3" 2>&1 </dev/null
  tr -d '\r' < "$3" > "$3.t"
}
sha() { sha256sum "$1" | cut -d' ' -f1; }
want() { grep -aq -e "$2" "$1.t" || say fail "$3: no '$2' in $1 (tail: $(tail -n3 "$1.t" | tr '\n' '|'))"; }
refuse() { grep -aq "ROOT-INIT-RAN" "$1.t" && say fail "$2: the root's init ran"; grep -aq "walleye-ufs-root: refused.*$3" "$1.t" || say fail "$2: no refusal line (tail: $(tail -n3 "$1.t" | tr '\n' '|'))"; }
part() { # DISK N OUT: partition N of the disk as a file
  s=$(sgdisk -i "$2" "$1" | sed -n 's/^First sector: \([0-9]*\).*/\1/p'); e=$(sgdisk -i "$2" "$1" | sed -n 's/^Last sector: \([0-9]*\).*/\1/p')
  dd if="$1" of="$3" bs=512 skip="$s" count=$((e - s + 1)) status=none
}
A0=$(sha "$T/a.img"); B0=$(sha "$T/b.img")
tree() { debugfs -R "ls -p $1" "$2" 2>/dev/null | awk -F/ "NF>5 {print \$6}" | LC_ALL=C sort; }
part "$T/a.img" 2 "$T/p2pre"; TREE0=$(tree / "$T/p2pre")
part "$T/a.img" 1 "$T/decoy0"; D0=$(sha "$T/decoy0")
# R1: the word
boot "$T/a.img" "walleye.root=system_a androidboot.slot_suffix=_b walleye.rw" "$T/rw.log"
want "$T/rw.log" "walleye-init: mounts-rw (system_a" "R1 (the init says it mounts rw)"
want "$T/rw.log" "ROOT-INIT-RAN" "R1 (the root's init ran)"
want "$T/rw.log" "ROOT-MOUNT /dev/vda2 / ext4 rw" "R1 (the root is the partition itself, ext4, rw: no overlay)"
grep -a "ROOT-MOUNT" "$T/rw.log.t" | grep -aq -e overlay -e discard && say fail "R1: the root mount names an overlay or discard"
want "$T/rw.log" "ROOT-WALLEYE-DIR tmpfs /.walleye tmpfs" "R1 (/.walleye is a tmpfs: the petter's files are not written to the card)"
want "$T/rw.log" "ROOT-RUN tmpfs /run tmpfs" "R1 (/run is a tmpfs)"
want "$T/rw.log" "ROOT-WRITE-OK" "R1 (the root can write)"
want "$T/rw.log" "ROOT-HAS-HOLD-SCRIPT" "R1 (the watchdog hold is in the root)"
want "$T/rw.log" "ROOT-HOLD-SYNCS" "R1 (the hold script syncs before it resets the phone: the root's writes reach the card)"
want "$T/rw.log" "ROOT-HAS-MARKER" "R1 (the card's files show)"
part "$T/a.img" 2 "$T/p2"
[ "$(debugfs -R 'cat /written-in-root' "$T/p2" 2>/dev/null)" = written-by-the-root-init ] || say fail "R1: the write did not land in the partition (read back with debugfs)"
[ "$(tree /.walleye "$T/p2" | grep -v -x -e . -e ..)" = "" ] || say fail "R1: the card's /.walleye is not empty (the petter's files were written to the card)"
# the clean end (item walleye-rw-end-through-openrc-mount-ro): walleye-end is the overlay's file byte for byte, and the shutdown level links mount-ro and walleye-end
[ "$(debugfs -R "cat /etc/init.d/walleye-end" "$T/p2" 2>/dev/null)" = "$(cat "$HERE/rootfs/overlay/etc/init.d/walleye-end")" ] || say fail "R1: the root's etc/init.d/walleye-end is not rootfs/overlay/etc/init.d/walleye-end"
for s in mount-ro walleye-end; do debugfs -R "stat /etc/runlevels/shutdown/$s" "$T/p2" 2>/dev/null | grep -q "/etc/init.d/$s" || say fail "R1: the shutdown level has no link to /etc/init.d/$s"; done
want "$T/rw.log" "walleye-init: shutdown-level mount-ro and walleye-end" "R1 (the init says it linked them)"
[ "$(tree / "$T/p2")" = "$(printf "%s\n.walleye\nwritten-in-root\n" "$TREE0" | LC_ALL=C sort)" ] || say fail "R1: the partition's top level is not what it was plus written-in-root and the empty .walleye (a stray file on the card)"
part "$T/a.img" 1 "$T/decoy1"; [ "$(sha "$T/decoy1")" = "$D0" ] || say fail "R1: the decoy partition changed"
[ "$(sha "$T/b.img")" = "$B0" ] || say fail "R1: another disk changed"
# R1 on the declared line: the rw image's command line in device.toml (its recipe) plus the one word the run adds (androidboot.slot_suffix=_b, the slot Android is on) is NOT refused by the guard: research walleye-rw-run-1-refused-by-booted-slot (a declared walleye.booted-slot made the init refuse before any mount)
DECL=$(sed -n '/^\[images.ufs-root-booted-slot-production-rw.recipe\]/,/^$/s/^cmdline = "\(.*\)"$/\1/p' "$HERE/device.toml")
[ -n "$DECL" ] || say fail "R1 (declared line): no cmdline in [images.ufs-root-booted-slot-production-rw.recipe] of device.toml"
boot "$T/a.img" "$DECL androidboot.slot_suffix=_b" "$T/declared.log"
want "$T/declared.log" "walleye-init: mounts-rw (system_a" "R1 (the declared line: the init mounts rw, the guard did not refuse)"
want "$T/declared.log" "ROOT-INIT-RAN" "R1 (the declared line: the root's init ran)"
# R2: no word: the production behaviour, the disk untouched
cp "$T/b.img" "$T/b-keep.img"
boot "$T/b.img" "walleye.root=system_b androidboot.slot_suffix=_a" "$T/ro.log"
want "$T/ro.log" "ROOT-INIT-RAN" "R2"
want "$T/ro.log" "ROOT-MOUNT overlay / overlay .*rw" "R2 (without the word the root is the overlay)"
grep -aq "mounts-rw" "$T/ro.log.t" && say fail "R2: without the word the init says it mounts rw"
[ "$(sha "$T/b.img")" = "$B0" ] || say fail "R2: without the word the disk changed"
# R3: the guards hold with the word, and nothing is written
A1=$(sha "$T/a.img"); cp "$T/a.img" "$T/a-after.img"
chk() { [ "$(sha "$T/a.img")" = "$A1" ] || say fail "$1: the disk changed although the init refused"; }
boot "$T/b.img" "walleye.root=system_a androidboot.slot_suffix=_b walleye.rw walleye.wait=3 walleye.pause=2" "$T/decoy.log"; refuse "$T/decoy.log" "R3 (only system_b present)" "no partition named"; [ "$(sha "$T/b.img")" = "$B0" ] || say fail "R3: the disk of the missing partition changed"
boot "$T/a.img" "walleye.root=system_a androidboot.slot_suffix=_a walleye.rw walleye.pause=2" "$T/running.log"; refuse "$T/running.log" "R3 (the running slot)" "the slot Android is on"; chk "R3 running slot"
boot "$T/a.img" "walleye.root=userdata androidboot.slot_suffix=_b walleye.rw walleye.pause=2" "$T/name.log"; refuse "$T/name.log" "R3 (a name outside system_a/system_b)" "is not system_a"; chk "R3 name"
boot "$T/a.img" "walleye.root=system_a walleye.rw walleye.pause=2" "$T/nosuffix.log"; refuse "$T/nosuffix.log" "R3 (no slot: fails closed)" "no androidboot.slot_suffix"; chk "R3 no slot"
boot "$T/a.img" "walleye.root=system_a androidboot.slot_suffix=_b walleye.rw walleye.booted-slot walleye.pause=2" "$T/bootedother.log"; refuse "$T/bootedother.log" "R3 (booted-slot with a root that is not the booted slot's)" "is not the system of the booted slot"; chk "R3 booted-slot"
boot "$T/a.img" "walleye.rw walleye.pause=2" "$T/noroot.log"; refuse "$T/noroot.log" "R3 (no root named)" "no walleye.root"; chk "R3 no root"
say pass "the word walleye.rw gives a rw ext4 root whose write lands on the partition (/.walleye and /run in tmpfs, the decoy and the other disk untouched); without it the root is the read-only overlay and the disk is unchanged; every guard refuses with the word and writes nothing; with the blocks cut the init is the production one"
