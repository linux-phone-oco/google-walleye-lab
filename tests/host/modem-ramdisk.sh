#!/bin/sh
# walleye-modem-ramdisk, run on the lab host: is the modem image's ramdisk what the item says (item walleye-modem-mss-image)?
# Passes only when the ramdisk (an xz cpio) holds: the five static tools (rmtfs, qrtr-ns, qrtr-lookup, tqftpserv, qmicli: item walleye-modem-qmicli-in-image) and busybox with the sha256 of build/initramfs-modem/tools.sha256 (each from its own record: the nix build's research walleye-rmtfs-static-nix-ok);
# the four scripts modem-up, firmware-recv, rootfs-recv and mm-up equal to the files of build/initramfs-modem; /init equal to the slpi ramdisk's init; the four storage copies /rmtfs/modem_fs1, modem_fs2, modem_fsg and modem_fsc
# byte for byte the phone's reads modemst1, modemst2, fsg, fsc (the sha256sums beside them), read-only (mode 444); and NO firmware file and no partition image of any other name (the firmware goes by firmware-recv; the
# 32 MiB image has no room for it). Read-only; extracts to a temp folder (mode 700) that it removes. Prints `pass|fail|skip walleye-modem-ramdisk modem DETAIL`; exit 0 only on pass.
# Environment: RAMDISK (the file, no default), MATERIAL (default ~/lab-material/walleye/modem-2026-11-19).
HERE=$(cd "$(dirname "$0")/../.." && pwd)
MATERIAL=${MATERIAL:-$HOME/lab-material/walleye/modem-2026-11-19}
say() { echo "$1 walleye-modem-ramdisk modem $2"; [ "$1" = pass ]; exit $?; }
[ -r "${RAMDISK:-}" ] || say skip "need RAMDISK=file: not known yet"
RAMDISK=$(realpath "$RAMDISK")
[ -r "$MATERIAL/sha256sums" ] || say skip "no $MATERIAL/sha256sums (the phone's reads): not known yet"
for c in xz cpio sha256sum; do command -v $c >/dev/null || say skip "$c not installed: not known yet"; done
umask 077; T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
# cpio cannot make the device nodes as a user and says so: judged by what came out, not by its exit
(cd "$T" && xz -dc "$RAMDISK" | cpio -id --quiet 2>/dev/null)
[ -f "$T/init" ] || say fail "$RAMDISK is not an xz cpio with an /init"
sum() { [ -f "$1" ] && sha256sum "$1" | cut -d' ' -f1; }
tool() { sed -n "s/^\([0-9a-f]\{64\}\)  $1\$/\1/p" "$HERE/build/initramfs-modem/tools.sha256"; }
for t in rmtfs qrtr-ns qrtr-lookup tqftpserv qmicli; do [ -n "$(tool $t)" ] && [ "$(sum "$T/bin/$t")" = "$(tool $t)" ] || say fail "/bin/$t is not the recorded static build"; done
[ -n "$(tool busybox-arm64)" ] && [ "$(sum "$T/bin/busybox")" = "$(tool busybox-arm64)" ] || say fail "/bin/busybox is not the recorded arm64 build"
for s in modem-up firmware-recv rootfs-recv mm-up; do cmp -s "$T/bin/$s" "$HERE/build/initramfs-modem/$s" || say fail "/bin/$s differs from build/initramfs-modem/$s"; done
# /init is slpi's with exactly one line changed - the end of the 180 s window - and the new line is build/initramfs-modem/window-end (decision 2026-11-27-walleye-modem-image-ends-by-sysrq-b-with-ipa)
d=$(diff "$HERE/build/initramfs-slpi/init" "$T/init")
[ "$(printf '%s\n' "$d" | sed -n 's/^> //p')" = "$(cat "$HERE/build/initramfs-modem/window-end")" ] || say fail "/init's window end is not build/initramfs-modem/window-end"
[ "$(printf '%s\n' "$d" | grep -c '^[<>]')" = 2 ] || say fail "/init differs from build/initramfs-slpi/init in more than one line"
[ "$(printf '%s\n' "$d" | sed -n 's/^< //p')" = '  sync; reboot -f; sleep 5; echo b > /proc/sysrq-trigger' ] || say fail "/init changed a line other than slpi's window end"
[ "$(grep -n 'sysrq-trigger' "$T/init" | head -n1 | grep -c 'echo b > /proc/sysrq-trigger; sleep 5; reboot -f')" = 1 ] || say fail "/init does not end with sync, sysrq b, then reboot -f as the fallback"
for pr in modem_fs1:modemst1 modem_fs2:modemst2 modem_fsg:fsg modem_fsc:fsc; do
  f=${pr%%:*}; p=${pr#*:}
  want=$(sed -n "s/^\([0-9a-f]\{64\}\)  $p.img\$/\1/p" "$MATERIAL/sha256sums")
  [ -n "$want" ] && [ "$(sum "$T/rmtfs/$f")" = "$want" ] || say fail "/rmtfs/$f is not the phone's $p read"
  [ "$(stat -c %a "$T/rmtfs/$f")" = 444 ] || say fail "/rmtfs/$f is not read-only (mode $(stat -c %a "$T/rmtfs/$f"))"
done
[ "$(ls "$T/rmtfs" | wc -l)" = 4 ] || say fail "/rmtfs holds more than the four storage copies"
fw=$(find "$T/lib/firmware" -type f | wc -l)
[ "$fw" = 0 ] || say fail "$fw firmware files are in the ramdisk (they go by firmware-recv)"
# nothing else: the files of the ramdisk are exactly these (no other partition image, no firmware, anywhere)
want="bin/busybox bin/firmware-recv bin/mm-up bin/modem-up bin/qmicli bin/qrtr-lookup bin/qrtr-ns bin/rmtfs bin/rootfs-recv bin/tqftpserv etc/udhcpd.conf init rmtfs/modem_fs1 rmtfs/modem_fs2 rmtfs/modem_fsc rmtfs/modem_fsg"
got=$(cd "$T" && find . -type f | sed 's|^./||' | sort | tr '\n' ' ' | sed 's/ $//')
[ "$got" = "$want" ] || say fail "the ramdisk's files are not exactly the expected ones: $got"
say pass "the five static tools (qmicli among them), busybox, modem-up, firmware-recv, rootfs-recv, mm-up and the slpi init are the recorded ones; /rmtfs holds the phone's modemst1, modemst2, fsg and fsc, read-only; no firmware inside"
