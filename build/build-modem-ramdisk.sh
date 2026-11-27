#!/bin/sh
# The ramdisk of the modem image (item walleye-modem-mss-image) and the tar of the phone's own modem firmware that the host pushes to it: the ramdisk holds the slpi ramdisk's init and busybox, the static modem tools,
# and read-only copies of its four modem storage partitions, packed with the roof's lpoco-bringup initramfs build (gen_init_cpio, xz crc32). Nothing here touches the phone.
#   build-modem-ramdisk.sh WORK_DIR OUT_CPIO_XZ OUT_FIRMWARE_TAR
# WORK_DIR is a folder of your own (mode 700) that this script fills and the cpio spec names; ENV: MATERIAL (default ~/lab-material/walleye/modem-2026-11-19, the phone's reads
# with its sha256sums), BUSYBOX (default ~/Projects/walleye-build/busybox-arm64), RMTFS, QRTR_NS, QRTR_LOOKUP, TQFTPSERV, QMICLI (the nix outputs of build/modem-tools.nix:
# `nix build -f build/modem-tools.nix NAME -o OUT-NAME` for NAME rmtfs, qrtr, tqftpserv and qmicli: the binaries are OUT-NAME/bin/rmtfs, qrtr-ns, qrtr-lookup, tqftpserv, qmicli). The partitions' contents stay in WORK_DIR and the ramdisk, never in git.
# Every check stops the build (docs practice 24): a source whose sha256 is not the recorded one, a missing firmware file, a ramdisk that differs on a second pack.
set -eu
W=${1:?usage: build-modem-ramdisk.sh WORK_DIR OUT_CPIO_XZ OUT_FIRMWARE_TAR}
OUT=${2:?usage: build-modem-ramdisk.sh WORK_DIR OUT_CPIO_XZ OUT_FIRMWARE_TAR}
TAR=${3:?usage: build-modem-ramdisk.sh WORK_DIR OUT_CPIO_XZ OUT_FIRMWARE_TAR}
HERE=$(cd "$(dirname "$0")" && pwd)
MATERIAL=${MATERIAL:-$HOME/lab-material/walleye/modem-2026-11-19}
VENDOR=${VENDOR:-$HOME/lab-material/walleye/vendor-firmware-2026-11-24}   # the phone's /vendor/firmware pull with its sha256sums: ipa_fws.* comes from it (the dts ipa node: qcom,gsi-loader self, firmware-name .../walleye/ipa_fws.mdt)
BUSYBOX=${BUSYBOX:-$HOME/Projects/walleye-build/busybox-arm64}
TOOLS=${TOOLS:-$HERE/initramfs-modem/tools.sha256}
umask 077
trap 'rm -f "$OUT.again" "$TAR.again"' EXIT
case $(realpath -m "$W") in "$(git -C "$HERE" rev-parse --show-toplevel)"/*) echo "build-modem-ramdisk: WORK_DIR is inside the git tree: the phone storage must stay out of it" >&2; exit 1;; esac
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
want() { sed -n "s/^\([0-9a-f]\{64\}\)  $1\$/\1/p" "$TOOLS"; }
sum() { sha256sum "$1" | cut -d' ' -f1; }
mkdir -p "$W/firmware" "$W/storage"
for t in rmtfs qrtr-ns qrtr-lookup tqftpserv qmicli; do
  var=$(printf '%s' "$t" | tr 'a-z-' 'A-Z_'); eval "src=\${$var:?$var: set to the nix output binary}"
  ck "$t is the recorded static build (tools.sha256)" "$(sum "$src")" "$(want "$t")"
  cp "$src" "$W/$t"; chmod 755 "$W/$t"
done
"$HERE/check-qmicli-qrtr.sh" "$W/qmicli" >&2 || total=$((total+1))
ck "busybox is the recorded arm64 build" "$(sum "$BUSYBOX")" "$(want busybox-arm64)"
cp "$BUSYBOX" "$W/busybox"; chmod 755 "$W/busybox"
cp "$HERE/initramfs-slpi/init" "$HERE/initramfs-slpi/udhcpd.conf" "$HERE/initramfs-modem/modem-up" "$HERE/initramfs-modem/firmware-recv" "$HERE/initramfs-modem/rootfs-recv" "$HERE/initramfs-modem/mm-up" "$W/"
chmod 755 "$W/init" "$W/modem-up" "$W/firmware-recv" "$W/rootfs-recv" "$W/mm-up"
# the phone's reads: the material's own sha256sums name each one
for p in modemst1 modemst2 fsg fsc modem_b; do
  ck "$p.img is the phone's read (sha256sums of the material)" "$(sum "$MATERIAL/$p.img")" "$(sed -n "s/^\([0-9a-f]\{64\}\)  $p.img\$/\1/p" "$MATERIAL/sha256sums")"
done
cp "$MATERIAL/modemst1.img" "$W/storage/modem_fs1"; cp "$MATERIAL/modemst2.img" "$W/storage/modem_fs2"
cp "$MATERIAL/fsg.img" "$W/storage/modem_fsg"; cp "$MATERIAL/fsc.img" "$W/storage/modem_fsc"
# the firmware set out of modem_b's FAT /image: the files the dtb's firmware-name and tqftpserv read (mba.mbn, modem.mdt, the b?? segments and qdsp6m.qdb; the jsn is the modem's own config)
rm -f "$W"/firmware/*
mcopy -i "$MATERIAL/modem_b.img" ::image/mba.mbn ::image/qdsp6m.qdb ::image/modemr.jsn "$W/firmware/"
mcopy -i "$MATERIAL/modem_b.img" ::image/modem.mdt "$W/firmware/"
for i in $(seq 0 27); do
  n=$(printf 'b%02d' "$i")
  # b15 and b19 are not in the partition (checked: mdir lists 28 of b00-b27 less these two; the mdt's program headers decide whether the loader needs them: tests/host/modem-firmware.sh)
  mcopy -i "$MATERIAL/modem_b.img" "::image/modem.$n" "$W/firmware/" || echo "note: modem.$n is not in modem_b/image"
done
ck "the firmware folder is whole by the mdt (tests/host/modem-firmware.sh)" "$(FW_DIR="$W/firmware" sh "$HERE/../tests/host/modem-firmware.sh" | cut -d' ' -f1)" pass
ck "qdsp6m.qdb is there (tqftpserv serves it)" "$([ -s "$W/firmware/qdsp6m.qdb" ] && echo yes || echo no)" yes
# ipa_fws: the AP loads it itself (qcom,gsi-loader self; firmware-name qcom/msm8998/google/walleye/ipa_fws.mdt), so it sits beside the modem's files; the pull's own sha256sums name each file
for e in mdt b00 b01 b02 b03 b04; do
  ck "ipa_fws.$e is the phone's pull (sha256sums of the vendor material)" "$(sum "$VENDOR/firmware/ipa_fws.$e")" "$(sed -n "s|^\([0-9a-f]\{64\}\)  ./firmware/ipa_fws.$e\$|\1|p" "$VENDOR/sha256sums")"
  cp "$VENDOR/firmware/ipa_fws.$e" "$W/firmware/"
done
ck "the ipa_fws files are whole by their mdt (tests/host/modem-firmware.sh)" "$(NAME=ipa_fws FW_DIR="$W/firmware" sh "$HERE/../tests/host/modem-firmware.sh" | cut -d' ' -f1)" pass
# wlanmdsp.mbn: the modem asks tqftpserv for it (/readonly/firmware/image/wlanmdsp.mbn, then /readonly/vendor/firmware/..., a prefix the pinned tqftpserv build does not know; run modem-boot-20261127-165116, research walleye-modem-tqftpserv-serves);
# on Android it is a file of /vendor/firmware, so the phone's pull has it; the pinned build serves /readonly/firmware/image/NAME from /lib/firmware/<the mss firmware's folder>/NAME, where the tar puts it
ck "wlanmdsp.mbn is the phone's pull (sha256sums of the vendor material)" "$(sum "$VENDOR/firmware/wlanmdsp.mbn")" "$(sed -n "s|^\([0-9a-f]\{64\}\)  ./firmware/wlanmdsp.mbn\$|\1|p" "$VENDOR/sha256sums")"
cp "$VENDOR/firmware/wlanmdsp.mbn" "$W/firmware/"
# the firmware is NOT in the ramdisk: 61 MB (34 MB xz) with the kernel does not fit the 32 MiB boot partition (decision 2026-11-27-walleye-modem-firmware-is-pushed-not-packed).
# It goes to the phone as one tar over the USB network (firmware-recv), with its own sha256 list that modem-up checks.
(cd "$W/firmware" && rm -f SHA256SUMS && sha256sum -- * > SHA256SUMS)
rm -f "$TAR"
tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --mode=u=rw,go=r -c -C "$W/firmware" -f "$TAR" .
tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --mode=u=rw,go=r -c -C "$W/firmware" -f "$TAR.again" .
ck "a second tar gives the same bytes" "$(sum "$TAR.again")" "$(sum "$TAR")"
ck "the tar holds what the modem asked for and was refused (tests/host/modem-firmware-served.sh)" "$(TAR="$TAR" VENDOR="$VENDOR" sh "$HERE/../tests/host/modem-firmware-served.sh" | cut -d' ' -f1)" pass
rm -f "$TAR.again"
# the spec (built here: one line per file, the names rmtfs reads)
{
  for d in /bin /sbin /etc /dev /proc /sys /tmp /run /lib /lib/firmware /lib/firmware/qcom /lib/firmware/qcom/msm8998 /lib/firmware/qcom/msm8998/google /lib/firmware/qcom/msm8998/google/walleye /rmtfs; do echo "dir $d 755 0 0"; done
  echo "slink /bin/sh busybox 777 0 0"
  echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"
  for b in busybox rmtfs qrtr-ns qrtr-lookup tqftpserv qmicli modem-up firmware-recv rootfs-recv mm-up; do echo "file /bin/$b $b 755 0 0"; done
  echo "file /init init 755 0 0"; echo "file /etc/udhcpd.conf udhcpd.conf 644 0 0"
  for f in modem_fs1 modem_fs2 modem_fsg modem_fsc; do echo "file /rmtfs/$f storage/$f 444 0 0"; done
} > "$W/spec"
lpoco-bringup initramfs build --spec "$W/spec" --root "$W" --codec xz --out "$OUT"
lpoco-bringup initramfs build --spec "$W/spec" --root "$W" --codec xz --out "$OUT.again"
ck "a second pack gives the same bytes" "$(sum "$OUT.again")" "$(sum "$OUT")"
rm -f "$OUT.again"
[ $total -eq 0 ] || { echo "build-modem-ramdisk: $total checks failed" >&2; rm -f "$OUT" "$TAR"; exit 1; }
sha256sum "$OUT" "$TAR"
