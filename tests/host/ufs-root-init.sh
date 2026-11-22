#!/bin/sh
# The proof for the UFS-root initramfs (backlog walleye-install-ufs-root-image), run on the lab host with no phone: QEMU boots the initramfs (build/build-initramfs.sh ufs-root-selftest:
# the x86_64 busybox stands in for the phone's arm64 one, the init is the same file) on a GPT disk whose partition `system_a` holds a loop-made test rootfs (mke2fs -d, no root needed).
# Cases, each one boot: (1) the named partition is there: the init mounts it read-only, puts a tmpfs overlay on it, the root's /sbin/init runs, a write lands in the overlay and not on the disk;
# (2) the disk is read-only to QEMU and its sha256 is the same after the run; (3) only a decoy `system_b` is there and `walleye.root=system_a` is asked: the init refuses and nothing is mounted;
# (4) the running slot (androidboot.slot_suffix=_a) is asked for as the root: the init refuses even though the partition is there; (5) a name that is not system_a or system_b is refused; (9) no androidboot.slot_suffix: refused (fails closed); (10) a suffix that is not _a or _b: refused.
#   ufs-root-init.sh WORK_DIR        (WORK_DIR holds busybox-x86_64 and out/b, as build/build-initramfs.sh wants)
# Prints `pass|fail|skip walleye-ufs-root-init DETAIL`; exit 0 only on pass; a skip is not a pass.
W=${1:-}
export GEN=${GEN:-}
HERE=$(cd "$(dirname "$0")/../.." && pwd)
say() { echo "$1 walleye-ufs-root-init $2"; [ "$1" = pass ]; exit $?; }
for c in qemu-system-x86_64 sgdisk mke2fs sha256sum zstd modprobe; do command -v "$c" >/dev/null || say skip "$c not installed: not known yet"; done
[ -n "$W" ] || say fail "usage: ufs-root-init.sh WORK_DIR"
[ -x "$W/busybox-x86_64" ] || say skip "no $W/busybox-x86_64: not known yet"
KVER=$(ls /usr/lib/modules | grep -v lts | sort -V | tail -n1)
KERNEL=/usr/lib/modules/$KVER/vmlinuz
[ -f "$KERNEL" ] || say skip "no host kernel $KERNEL: not known yet"
T=$(mktemp -d) || say skip "no temp dir"; [ -n "${KEEP:-}" ] || trap 'rm -rf "$T"' EXIT
# the test rootfs: a busybox root whose /sbin/init says what it sees and powers the machine off
mkdir -p "$T/root/sbin" "$T/root/bin" "$T/root/proc" "$T/root/sys" "$T/root/dev" "$T/root/etc" "$T/root/run" "$T/root/tmp"
cp "$W/busybox-x86_64" "$T/root/bin/busybox"
for a in sh mount cat echo ls sync poweroff touch grep sleep head wc tr;  do ln -s busybox "$T/root/bin/$a"; done
cat > "$T/root/sbin/init" <<'EOI'
#!/bin/sh
echo "ROOT-INIT-RAN pid=$$"
echo "ROOT-MOUNT $(grep ' / ' /proc/mounts | head -n1)"
echo "ROOT-LOWER $(grep ' /lower ' /proc/mounts | head -n1)"
touch /written-in-overlay 2>/dev/null && echo "ROOT-WRITE-OK" || echo "ROOT-WRITE-FAILED"
[ -e /walleye-test-marker ] && echo "ROOT-HAS-MARKER"
[ -e /.walleye/hold.sh ] && echo "ROOT-HAS-HOLD-SCRIPT"
mount -t tmpfs tmpfs /run; sleep 1   # OpenRC mounts a fresh tmpfs on /run: the petter must not live there
(echo x > /dev/vda2) 2>/dev/null && echo ROOT-DEV-WRITABLE || echo ROOT-DEV-READONLY
sleep 2
[ "$(ls -l /proc/[0-9]*/fd 2>/dev/null | grep -c /dev/watchdog)" -ge 1 ] && echo "ROOT-WATCHDOG-HELD"
grep -q walleye.test=linger /proc/cmdline && { echo "ROOT-LINGERS"; sleep 120; }
sync
poweroff -f
EOI
chmod 755 "$T/root/sbin/init"
echo marker > "$T/root/walleye-test-marker"
mke2fs -q -t ext4 -L testroot -d "$T/root" "$T/root.ext4" 16M || say fail "mke2fs did not make the test rootfs"
# the disk: GPT, a decoy first, then the partitions the cases ask for
disk() { # FILE NAME: a GPT disk of 64 MiB with the test rootfs in the partition NAME and an empty decoy `userdata`
  truncate -s 64M "$1"
  sgdisk -Z -n 1:2048:+8M -c 1:userdata -n 2:0:+20M -c 2:"$2" "$1" >/dev/null || say fail "sgdisk failed"
  start=$(sgdisk -i 2 "$1" | sed -n 's/^First sector: \([0-9]*\).*/\1/p')
  dd if="$T/root.ext4" of="$1" bs=512 seek="$start" conv=notrunc status=none || say fail "dd failed"
}
disk "$T/a.img" system_a
# a root whose /sbin/init is a symlink to nothing (the init must refuse it, not hand over)
mkdir -p "$T/dangling/sbin" && ln -s /nonexistent "$T/dangling/sbin/init" && mke2fs -q -t ext4 -d "$T/dangling" "$T/dangling.ext4" 16M || say fail "no dangling-init rootfs"
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
A0=$(sha "$T/a.img")
boot "$T/a.img" "walleye.root=system_a androidboot.slot_suffix=_b" "$T/ok.log"
want "$T/ok.log" "ROOT-INIT-RAN" "case 1"
want "$T/ok.log" "ROOT-MOUNT overlay / overlay .*rw" "case 1 (the root is an overlay)"
want "$T/ok.log" "ROOT-LOWER /dev/vda2 /lower ext4 ro,.*norecovery" "case 1 (the card's partition is read-only, no journal replay)"
grep -a "ROOT-LOWER" "$T/ok.log.t" | grep -aq "discard" && say fail "case 1: the lower mount names discard"
grep -q -e "-o ro,nodiscard,noload" "$HERE/build/initramfs-root/init" || say fail "case 1: the init's ext4 mount does not say ro,nodiscard,noload (/proc/mounts shows no word for nodiscard: discard is simply absent)"
want "$T/ok.log" "ROOT-DEV-READONLY" "case 2 (the kernel refuses writes to the card partition: blockdev --setro)"
want "$T/ok.log" "ROOT-WRITE-OK" "case 1 (a write lands in the overlay)"
want "$T/ok.log" "ROOT-HAS-MARKER" "case 1 (the lower's files show)"
want "$T/ok.log" "ROOT-HAS-HOLD-SCRIPT" "case 1 (the watchdog petter is in the root)"
want "$T/ok.log" "ROOT-WATCHDOG-HELD" "case 1 (a process in the root holds /dev/watchdog open after the hand-over)"
[ "$(sha "$T/a.img")" = "$A0" ] || say fail "case 2: the disk image changed"
boot "$T/b.img" "walleye.root=system_a androidboot.slot_suffix=_b walleye.wait=3 walleye.pause=2" "$T/decoy.log"; refuse "$T/decoy.log" "case 3 (only system_b present)" "no partition named"
boot "$T/a.img" "walleye.root=system_a androidboot.slot_suffix=_a walleye.pause=2" "$T/running.log"; refuse "$T/running.log" "case 4 (the running slot)" "slot Android runs from"
boot "$T/a.img" "walleye.root=userdata androidboot.slot_suffix=_b walleye.pause=2" "$T/name.log"; refuse "$T/name.log" "case 5 (a name that is not a system slot)" "is not system_a"
boot "$T/a.img" "androidboot.slot_suffix=_b walleye.pause=2" "$T/none.log"; refuse "$T/none.log" "case 6 (no root asked)" "no walleye.root"
boot "$T/a.img" "walleye.root=system_a walleye.pause=2" "$T/nosuffix.log"; refuse "$T/nosuffix.log" "case 9 (no androidboot.slot_suffix: the bootloader of a fastboot boot gives none; the init fails CLOSED)" "no androidboot.slot_suffix"
boot "$T/a.img" "walleye.root=system_a androidboot.slot_suffix=_c walleye.pause=2" "$T/badsuffix.log"; refuse "$T/badsuffix.log" "case 10 (a suffix that is not _a or _b)" "is not _a or _b"
cp "$T/a.img" "$T/d.img"; start=$(sgdisk -i 2 "$T/d.img" | sed -n "s/^First sector: \([0-9]*\).*/\1/p"); dd if="$T/dangling.ext4" of="$T/d.img" bs=512 seek="$start" conv=notrunc status=none
boot "$T/d.img" "walleye.root=system_a androidboot.slot_suffix=_b walleye.pause=2" "$T/dangling.log"; refuse "$T/dangling.log" "case 8 (/sbin/init is a symlink to nothing)" "symlink to nothing"
boot "$T/a.img" "walleye.root=system_a androidboot.slot_suffix=_b walleye.hold=6 walleye.test=linger" "$T/hold.log"
want "$T/hold.log" "ROOT-LINGERS" "case 7"
want "$T/hold.log" "walleye-ufs-root: hold over, the phone resets itself" "case 7 (after the hold the root is left to the watchdog: the phone restarts by itself)"
say pass "the init mounts system_a read-only (nodiscard) under a tmpfs overlay and hands over to the root's init; the petter ends with the hold and the phone resets itself; it refuses a missing partition, a missing or foreign slot suffix, the running slot, a foreign name, no name and a root without a working /sbin/init; the disk is unchanged ($A0)"
