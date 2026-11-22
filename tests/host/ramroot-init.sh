#!/bin/sh
# The proof for the ramroot initramfs (backlog walleye-display-alpine-ramroot-image), run on the lab host with no phone: QEMU boots the ramdisk (the x86_64 busybox stands in for the phone's arm64 one, the init is
# the same file, build/initramfs-ramroot/init) whose /userland.tar is a small test userland, and no disk at all.
# Cases, each one boot: (1) the delay is waited, every line of the module list is tried and a failing one is logged and does not stop the next, the tar is unpacked into a tmpfs, removed, a setuid bit and a link
# survive, the root's init runs on a tmpfs root and the hold petter script is in it; (2) no /userland.tar: the init refuses and the root's init does not run; (3) a tar with no /sbin/init is refused; (4) a truncated
# tar is refused. The petter's text is the proven root init's (tests/host/ufs-root-init.sh proves that one running): the two hold scripts are the same lines but the log tag.
#   ramroot-init.sh WORK_DIR        (WORK_DIR holds busybox-x86_64)
# Prints `pass|fail|skip walleye-ramroot-init DETAIL`; exit 0 only on pass; a skip is not a pass.
W=${1:-}
HERE=$(cd "$(dirname "$0")/../.." && pwd)
say() { echo "$1 walleye-ramroot-init $2"; [ "$1" = pass ]; exit $?; }
for c in qemu-system-x86_64 tar lpoco-bringup; do command -v "$c" >/dev/null || say skip "$c not installed: not known yet"; done
[ -n "$W" ] || say fail "usage: ramroot-init.sh WORK_DIR"
[ -x "$W/busybox-x86_64" ] || say skip "no $W/busybox-x86_64: not known yet"
KVER=$(ls /usr/lib/modules | grep -v lts | sort -V | tail -n1)
KERNEL=/usr/lib/modules/$KVER/vmlinuz
[ -f "$KERNEL" ] || say skip "no host kernel $KERNEL: not known yet"
INIT=$HERE/build/initramfs-ramroot/init
T=$(mktemp -d) || say skip "no temp dir"; [ -n "${KEEP:-}" ] || trap 'rm -rf "$T"' EXIT
hold_block() { sed -n '/^cat > \/newroot\/.walleye\/hold.sh/,/^EOH$/p' "$1" | sed 's/walleye-ufs-root:/TAG:/; s/walleye-ramroot:/TAG:/'; }
[ -n "$(hold_block "$INIT")" ] || say fail "the ramroot init has no hold script block"
[ "$(hold_block "$INIT")" = "$(hold_block "$HERE/build/initramfs-root/init")" ] || say fail "the hold script is not the proven root init's"
# the test userland: a busybox root whose /sbin/init says what it sees and powers the machine off
mkdir -p "$T/u/sbin" "$T/u/bin" "$T/u/proc" "$T/u/sys" "$T/u/dev" "$T/u/etc" "$T/u/run" "$T/u/tmp"
cp "$W/busybox-x86_64" "$T/u/bin/busybox"
for a in sh mount cat echo ls sync poweroff touch grep sleep head wc tr readlink; do ln -s busybox "$T/u/bin/$a"; done
ln -s busybox "$T/u/bin/linkapplet"
printf '#!/bin/sh\n' > "$T/u/bin/bbsuid"; chmod 4755 "$T/u/bin/bbsuid"
cat > "$T/u/sbin/init" <<'EOI'
#!/bin/sh
echo "ROOT-INIT-RAN pid=$$"
echo "ROOT-MOUNT $(grep ' / ' /proc/mounts | head -n1)"
[ -e /userland.tar ] && echo "ROOT-HAS-TAR" || echo "ROOT-NO-TAR"
ls -l /bin/bbsuid | grep -q '^-rws' && echo "ROOT-SUID-KEPT"
[ "$(readlink /bin/linkapplet)" = busybox ] && echo "ROOT-LINK-KEPT"
[ -e /.walleye/hold.sh ] && echo "ROOT-HAS-HOLD-SCRIPT"
[ -d /run/walleye ] && echo "ROOT-HAS-RUN-WALLEYE"
[ -e /proc/cmdline ] && [ -d /sys/kernel ] && echo "ROOT-HAS-PROC-AND-SYS"
[ -e /dev/null ] && echo "ROOT-HAS-DEV"
sync
poweroff -f
EOI
chmod 755 "$T/u/sbin/init"
tar -C "$T/u" --sort=name --numeric-owner --owner=0 --group=0 -cf "$T/userland.tar" .
mkdir -p "$T/n/bin" "$T/n/etc"; cp "$W/busybox-x86_64" "$T/n/bin/busybox"; tar -C "$T/n" --numeric-owner --owner=0 --group=0 -cf "$T/noinit.tar" .
head -c $(( $(stat -c %s "$T/userland.tar") / 2 + 100 )) "$T/userland.tar" > "$T/truncated.tar"   # cut inside the busybox file data, never at an entry boundary
# the ramdisk of one case: TAR (or none) in it, a delay of 2 s, a module list of two lines that cannot load
ramdisk() { # NAME TAR|none: writes $T/NAME.xz (gzip: the host kernel reads it)
  mkdir -p "$T/rd-$1/bin" && cp "$W/busybox-x86_64" "$T/rd-$1/bin/busybox"
  printf '2\n' > "$T/rd-$1/display-delay"
  printf 'insmod /lib/modules/not-there-1.ko\ninsmod /lib/modules/not-there-2.ko\n' > "$T/rd-$1/display-modules.list"
  { for d in bin sbin etc dev proc sys tmp run lib; do echo "dir /$d 755 0 0"; done
    echo "slink /bin/sh busybox 777 0 0"; echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"
    echo "file /bin/busybox $T/rd-$1/bin/busybox 755 0 0"; echo "file /init $INIT 755 0 0"
    echo "file /display-delay $T/rd-$1/display-delay 644 0 0"; echo "file /display-modules.list $T/rd-$1/display-modules.list 644 0 0"
    [ "$2" = none ] || echo "file /userland.tar $2 644 0 0"; } > "$T/$1.spec"
  lpoco-bringup initramfs build --spec "$T/$1.spec" --root "$T/rd-$1" --codec gzip --out "$T/$1.cpio.gz" >/dev/null 2>"$T/$1.err" || say fail "lpoco-bringup initramfs build ($1): $(head -n1 "$T/$1.err")"
}
ACCEL=tcg; [ -w /dev/kvm ] && ACCEL=kvm
boot() { # NAME CMDLINE: boots $T/NAME.cpio.gz, log $T/NAME.log (carriage returns dropped in NAME.log.t)
  timeout 240 qemu-system-x86_64 -machine accel=$ACCEL -cpu max -m 1024 -nographic -no-reboot \
    -kernel "$KERNEL" -initrd "$T/$1.cpio.gz" -append "console=ttyS0 panic=-1 ignore_loglevel $2" > "$T/$1.log" 2>&1 </dev/null
  [ $? -ne 124 ] || say fail "$1: QEMU did not end by itself within the timeout (a refusal or the root init never powered off)"
  tr -d '\r' < "$T/$1.log" > "$T/$1.log.t"
}
want() { grep -aq -e "$2" "$T/$1.log.t" || say fail "$3: no '$2' in the $1 log (tail: $(tail -n3 "$T/$1.log.t" | tr '\n' '|'))"; }
refuse() { # NAME WORDS CASE
  grep -aq "ROOT-INIT-RAN" "$T/$1.log.t" && say fail "$3: the root's init ran"
  grep -aq "walleye-ramroot: refused: .*$2" "$T/$1.log.t" || say fail "$3: no refusal line '$2' (tail: $(tail -n3 "$T/$1.log.t" | tr '\n' '|'))"
}
ramdisk ok "$T/userland.tar"; boot ok ""
want ok "walleye-ramroot: start" "case 1"
want ok "walleye-display: delay 2 s reached at uptime" "case 1 (the delay is waited)"
[ "$(grep -ac "walleye-display: FAILED rc=" "$T/ok.log.t")" -eq 2 ] || say fail "case 1: both failing module lines were not tried and logged (a failure must not stop the next line)"
want ok "walleye-ramroot: display modules: done" "case 1"
want ok "walleye-ramroot: userland unpacked" "case 1"
want ok "ROOT-INIT-RAN" "case 1"
want ok "ROOT-MOUNT tmpfs / tmpfs" "case 1 (the root is a tmpfs)"
want ok "ROOT-NO-TAR" "case 1 (the tar is freed)"
want ok "ROOT-SUID-KEPT" "case 1 (the setuid bit survives the unpack)"
want ok "ROOT-LINK-KEPT" "case 1 (a link survives)"
want ok "ROOT-HAS-HOLD-SCRIPT" "case 1 (the watchdog petter is in the root)"
want ok "ROOT-HAS-RUN-WALLEYE" "case 1 (/run/walleye is there for the hold file)"
want ok "ROOT-HAS-PROC-AND-SYS" "case 1 (proc and sys are moved)"
want ok "ROOT-HAS-DEV" "case 1 (dev is moved)"
ramdisk notar none; boot notar "walleye.pause=2"; refuse notar "no /userland.tar" "case 2 (no tar)"
ramdisk noinit "$T/noinit.tar"; boot noinit "walleye.pause=2"; refuse noinit "holds no /sbin/init" "case 3 (no /sbin/init in the userland)"
ramdisk trunc "$T/truncated.tar"; boot trunc "walleye.pause=2"; refuse trunc "did not unpack" "case 4 (a truncated tar)"
say pass "the init waits the delay, tries every module line and logs a failure, unpacks the tar into a tmpfs root (setuid and links kept, the tar freed) and hands over to its /sbin/init; it refuses no tar, a tar with no /sbin/init and a truncated tar; the hold script is the proven root init's"
