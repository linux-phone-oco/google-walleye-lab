#!/bin/sh
# The proof for the rw root's clean end (backlog walleye-rw-clean-end-who-holds-root, run 3 of the rw write: `mount -o remount,ro /` failed 'Resource busy', e2fsck -fn from Android: 'Free blocks count wrong'), host only, no phone:
#   rw-root-clean-end.sh WORK_DIR        (WORK_DIR holds busybox-x86_64, as tests/host/ufs-root-rw-init.sh wants)
# QEMU: the rw-write init mounts a loop-made rootfs ext4 rw; the root's busybox init starts a writer that holds /var/log/messages open for writing (syslogd's case), then ends the way the image ends: busybox reboot -> init runs
# the inittab's `::shutdown:/sbin/openrc shutdown`. The stub /sbin/openrc here takes the services of the SHUTDOWN level from the image's own rootfs/runlevels (killprocs: end every process but init and itself; mount-ro: sync, remount ro;
# savecache: nothing; a service the stub does not know: said and skipped). After the guest powers off, e2fsck -fn on the partition must find it clean: no "wrong" line, no "skipping journal recovery" (the flag of an rw root cut off).
# It fails while rootfs/runlevels has no `shutdown mount-ro` (the root is then cut off rw) and passes when Alpine's own path (killprocs, then mount-ro) is in the level. INIT_DIR / RUNLEVELS: other files to judge.
# Prints `pass|fail|skip walleye-rw-root-clean-end DETAIL`; exit 0 only on pass; a skip is not a pass.
W=${1:-}
export GEN=${GEN:-}
HERE=$(cd "$(dirname "$0")/../.." && pwd)
PDIR=${INIT_DIR:-$HERE/build/initramfs-root-rw-write}
RUNLEVELS=${RUNLEVELS:-$HERE/rootfs/runlevels}
say() { echo "$1 walleye-rw-root-clean-end $2"; [ "$1" = pass ]; exit $?; }
for c in qemu-system-x86_64 sgdisk mke2fs e2fsck sha256sum zstd; do command -v "$c" >/dev/null || say skip "$c not installed: not known yet"; done
[ -n "$W" ] || say fail "usage: rw-root-clean-end.sh WORK_DIR"
[ -x "$W/busybox-x86_64" ] || say skip "no $W/busybox-x86_64: not known yet"
[ -f "$PDIR/init" ] || say fail "no $PDIR/init"
[ -f "$RUNLEVELS" ] || say fail "no $RUNLEVELS"
KVER=$(ls /usr/lib/modules | grep -v lts | sort -V | tail -n1)
KERNEL=/usr/lib/modules/$KVER/vmlinuz
[ -f "$KERNEL" ] || say skip "no host kernel $KERNEL: not known yet"
T=$(mktemp -d) || say skip "no temp dir"; [ -n "${KEEP:-}" ] || trap 'rm -rf "$T"' EXIT
R=$T/root
mkdir -p "$R/sbin" "$R/bin" "$R/proc" "$R/sys" "$R/dev" "$R/etc" "$R/run" "$R/tmp" "$R/var/log"
cp "$W/busybox-x86_64" "$R/bin/busybox"
for a in sh mount cat echo sync reboot kill sleep grep tr head; do ln -s busybox "$R/bin/$a"; done
ln -s ../bin/busybox "$R/sbin/init"
# the image's own inittab lines that matter: sysinit, a once, and the shutdown action (rootfs/overlay/etc/inittab)
grep '^::shutdown:' "$HERE/rootfs/overlay/etc/inittab" > "$R/etc/inittab" || say fail "rootfs/overlay/etc/inittab has no ::shutdown: line"
grep '^shutdown ' "$RUNLEVELS" > "$R/etc/shutdown-level"
cat >> "$R/etc/inittab" <<'EOI'
::sysinit:/bin/sh /etc/rc.boot
::once:/bin/sh /etc/rc.end
EOI
cat > "$R/etc/rc.boot" <<'EOI'
mount -t proc proc /proc 2>/dev/null
mount -t sysfs sys /sys 2>/dev/null
# syslogd's case: a process that holds a file of the root open for writing, as long as it runs
( exec 3>>/var/log/messages; while :; do echo tick >&3; sleep 1; done ) &
echo "ROOT-BOOT $(grep ' / ' /proc/mounts | head -n1)"
EOI
cat > "$R/etc/rc.end" <<'EOI'
sleep 3
echo "ROOT-END-BEGIN"
reboot
EOI
cat > "$R/sbin/openrc" <<'EOI'
#!/bin/sh
# the stub: the services of the level named $1, in the order of /etc/shutdown-level
[ "$1" = shutdown ] || exit 0
while read -r lvl svc; do
  case $svc in
    killprocs) for sig in TERM KILL; do for d in /proc/[0-9]*; do p=${d#/proc/}; [ "$p" = 1 ] || [ "$p" = $$ ] || [ -z "$(tr -d "\000" < $d/cmdline 2>/dev/null)" ] || kill -$sig "$p" 2>/dev/null; done; sleep 2; done; echo "OPENRC killprocs" ;;
    mount-ro) sync; if mount -o remount,ro / 2>/run/mro; then echo "OPENRC mount-ro ok"; else echo "OPENRC mount-ro FAILED $(cat /run/mro)"; fi ;;
    savecache) echo "OPENRC savecache" ;;
    *) echo "OPENRC unknown service $svc skipped" ;;
  esac
done < /etc/shutdown-level
echo "OPENRC shutdown done, root: $(grep ' / ' /proc/mounts | head -n1)"
EOI
chmod 755 "$R/sbin/openrc" "$R/etc/rc.boot" "$R/etc/rc.end"
mke2fs -q -t ext4 -L testroot -d "$R" "$T/root.ext4" 16M || say fail "mke2fs did not make the test rootfs"
truncate -s 64M "$T/a.img"
sgdisk -Z -n 1:2048:+8M -c 1:userdata -n 2:0:+20M -c 2:system_a "$T/a.img" >/dev/null || say fail "sgdisk failed"
start=$(sgdisk -i 2 "$T/a.img" | sed -n 's/^First sector: \([0-9]*\).*/\1/p')
dd if="$T/root.ext4" of="$T/a.img" bs=512 seek="$start" conv=notrunc status=none || say fail "dd failed"
[ -n "$GEN" ] || for g in "$W/out/b/usr/gen_init_cpio" "$W"/out-ufsroot/b/usr/gen_init_cpio; do [ -x "$g" ] && GEN=$g; done; export GEN
SELFTEST_KVER=$KVER INIT_DIR=$PDIR "$HERE/build/build-initramfs.sh" ufs-root-selftest "$W" "$T/initramfs.cpio.gz" >"$T/build.log" 2>&1 || { tail -n5 "$T/build.log"; say fail "build-initramfs.sh ufs-root-selftest failed"; }
ACCEL=tcg; [ -w /dev/kvm ] && ACCEL=kvm
timeout 240 qemu-system-x86_64 -machine accel=$ACCEL -cpu max -m 1024 -nographic -no-reboot \
  -kernel "$KERNEL" -initrd "$T/initramfs.cpio.gz" -device i6300esb -action watchdog=poweroff -drive file="$T/a.img",if=virtio,format=raw \
  -append "console=ttyS0 panic=-1 loglevel=4 walleye.root=system_a androidboot.slot_suffix=_b walleye.rw walleye.hold=60" > "$T/run.log" 2>&1 </dev/null
tr -d '\r' < "$T/run.log" > "$T/run.t"
grep -aq "ROOT-BOOT /dev/vda2 / ext4 rw" "$T/run.t" || say fail "the root did not boot rw (tail: $(tail -n3 "$T/run.t" | tr '\n' '|'))"
grep -aq "ROOT-END-BEGIN" "$T/run.t" || say fail "the root's end did not begin (tail: $(tail -n3 "$T/run.t" | tr '\n' '|'))"
s=$(sgdisk -i 2 "$T/a.img" | sed -n 's/^First sector: \([0-9]*\).*/\1/p'); e=$(sgdisk -i 2 "$T/a.img" | sed -n 's/^Last sector: \([0-9]*\).*/\1/p')
dd if="$T/a.img" of="$T/p2" bs=512 skip="$s" count=$((e - s + 1)) status=none
e2fsck -fn "$T/p2" > "$T/fsck.txt" 2>&1; rc=$?
OP=$(grep -a '^OPENRC' "$T/run.t" | tr '\n' '|')
[ "$rc" = 0 ] || say fail "e2fsck -fn rc=$rc: $(grep -a -e wrong -e journal -e clean "$T/fsck.txt" | head -n3 | tr '\n' '|') (the end said: $OP)"
grep -aq -e wrong -e 'skipping journal' "$T/fsck.txt" && say fail "e2fsck -fn is not clean: $(grep -a -e wrong -e 'skipping journal' "$T/fsck.txt" | head -n3 | tr '\n' '|') (the end said: $OP)"
say pass "the root, held open for writing by a process, ends clean through the shutdown level $(tr '\n' ' ' < "$R/etc/shutdown-level"): $OP e2fsck -fn clean"
