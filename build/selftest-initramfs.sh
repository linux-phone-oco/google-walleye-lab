#!/bin/sh
# The proof for walleye-initramfs-usb-gadget: boot the initramfs under QEMU with the host kernel's dummy_hcd (a USB
# host controller and a device controller wired to each other), and let the init reach its own gadget through the
# host side: lease, ping and telnet on 172.16.42.1. The x86_64 busybox stands in for the phone's arm64 one - the init
# and its scripts are the same files. Needs qemu-system-x86_64; /dev/kvm makes it fast, without it it is slow.
#
#   selftest-initramfs.sh WORK_DIR
# NET=43: the same proof with the initramfs at 172.16.43.1 (default 42).
# Prints the SELFTEST lines; exits 0 only if the last is "SELFTEST PASS".
set -eu
W=${1:?usage: selftest-initramfs.sh WORK_DIR}
HERE=$(cd "$(dirname "$0")" && pwd)
KVER=$(ls /usr/lib/modules | grep -v lts | sort -V | tail -n1)
KERNEL=/usr/lib/modules/$KVER/vmlinuz
[ -f "$KERNEL" ] || { echo "no host kernel $KERNEL" >&2; exit 1; }
NET=${NET:-42}
SELFTEST_KVER=$KVER NET=$NET "$HERE/build-initramfs.sh" selftest "$W" "$W/initramfs-selftest.cpio.gz" >/dev/null
ACCEL=tcg; [ -w /dev/kvm ] && ACCEL=kvm
timeout 240 qemu-system-x86_64 -machine accel=$ACCEL -cpu max -m 1024 -nographic -no-reboot \
  -kernel "$KERNEL" -initrd "$W/initramfs-selftest.cpio.gz" \
  -append "console=ttyS0 panic=-1 loglevel=4 bringup.selftest=1" > "$W/selftest.log" 2>&1 || true
grep -a "walleye-init\|SELFTEST" "$W/selftest.log" | tr -d '\r'
tail -n1 "$W/selftest.log" >/dev/null
grep -aq "^SELFTEST PASS" "$W/selftest.log"
