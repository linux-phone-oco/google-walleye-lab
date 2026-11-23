#!/bin/sh
# Pack the walleye initramfs (busybox + build/initramfs/*) as a gzip cpio, with the kernel's own gen_init_cpio
# (no root, fixed timestamps, so the same inputs give the same bytes).
#
#   build-initramfs.sh phone    WORK_DIR OUT_CPIO_GZ     arm64 busybox, for the phone
#   build-initramfs.sh selftest WORK_DIR OUT_CPIO_GZ     x86_64 busybox + the host's gadget modules and iproute2,
#                                                        for the QEMU self-test (build/selftest-initramfs.sh)
# NET=43 (default 42): the phone answers on 172.16.NET.1 and leases 172.16.NET.2; the init, udhcpd.conf and selftest.sh are
# rewritten into WORK_DIR/net43/ (172.16.42 -> 172.16.43, the ECM MAC byte 2a -> 2b); at 42 the files are used as they are,
# so every image built before stays byte for byte.
# WORK_DIR holds busybox-arm64 / busybox-x86_64 (build-busybox.sh) and out/b (build-kernel.sh).
set -eu
MODE=${1:?usage: build-initramfs.sh phone|selftest WORK_DIR OUT}
W=${2:?usage: build-initramfs.sh phone|selftest WORK_DIR OUT}
OUT=${3:?usage: build-initramfs.sh phone|selftest WORK_DIR OUT}
HERE=$(cd "$(dirname "$0")" && pwd)
case $MODE in phone|ufs-root) BB=$W/busybox-arm64 ;; selftest|ufs-root-selftest) BB=$W/busybox-x86_64 ;; *) echo "mode $MODE" >&2; exit 2 ;; esac
GEN=${GEN:-$W/out/b/usr/gen_init_cpio}   # GEN: the usr/gen_init_cpio of any built kernel tree
[ -x "$GEN" ] || make -C "$W/linux-v7.3" ARCH=arm64 LLVM=1 O="$W/out/b" usr/gen_init_cpio >/dev/null
NET=${NET:-42}
SRC=$HERE/initramfs; SUF=
case $MODE in ufs-root|ufs-root-selftest) SRC=${INIT_DIR:-$HERE/initramfs-root} ;; esac   # INIT_DIR: another init (build/initramfs-root-display, the UFS root with the display)
if [ "$NET" != 42 ] && [ "${MODE#ufs-root}" = "$MODE" ]; then
  SRC=$W/net$NET; SUF=-$NET; rm -rf "$SRC"; mkdir -p "$SRC"
  MAC=$(printf %02x "$NET")
  for n in init udhcpd.conf selftest.sh udhcpc.script; do
    sed -e "s/172\.16\.42\./172.16.$NET./g" -e "s/ac:10:2a:/ac:10:$MAC:/g" "$HERE/initramfs/$n" > "$SRC/$n"
  done
  grep -q "172.16.$NET.1" "$SRC/init" && ! grep -q "172\.16\.42" "$SRC/init" "$SRC/udhcpd.conf" "$SRC/selftest.sh" || { echo "build-initramfs: address rewrite left 42 behind" >&2; exit 1; }
fi
SPEC=$W/initramfs-$MODE$SUF.spec
{
  echo "dir /bin 755 0 0"; echo "dir /sbin 755 0 0"; echo "dir /etc 755 0 0"; echo "dir /dev 755 0 0"
  echo "dir /proc 755 0 0"; echo "dir /sys 755 0 0"; echo "dir /tmp 1777 0 0"; echo "dir /run 755 0 0"
  echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"
  echo "file /bin/busybox $BB 755 0 0"
  echo "slink /bin/sh busybox 777 0 0"
  echo "file /init $SRC/init 755 0 0"
  [ "${MODE#ufs-root}" != "$MODE" ] || echo "file /etc/udhcpd.conf $SRC/udhcpd.conf 644 0 0"
} > "$SPEC"
# DISPLAY_FILES (ufs-root and ufs-root-selftest only): a directory holding display-modules.list, display-delay and lib/ (the display modules, depmod run); packed as the display init (INIT_DIR) reads them
if [ -n "${DISPLAY_FILES:-}" ]; then
  case $MODE in ufs-root|ufs-root-selftest) ;; *) echo "build-initramfs: DISPLAY_FILES is for ufs-root and ufs-root-selftest" >&2; exit 2 ;; esac
  for f in display-modules.list display-delay; do [ -s "$DISPLAY_FILES/$f" ] || { echo "build-initramfs: no $DISPLAY_FILES/$f" >&2; exit 1; }; done
  {
    echo "file /display-modules.list $DISPLAY_FILES/display-modules.list 644 0 0"
    echo "file /display-delay $DISPLAY_FILES/display-delay 644 0 0"
    if [ -d "$DISPLAY_FILES/lib" ] && [ "$MODE" = ufs-root ]; then
      (cd "$DISPLAY_FILES" && find lib -type d | LC_ALL=C sort | awk '{ print "dir /" $0 " 755 0 0" }')
      (cd "$DISPLAY_FILES" && find lib -type f | LC_ALL=C sort | awk -v r="$DISPLAY_FILES" '{ print "file /" $0 " " r "/" $0 " 644 0 0" }')
    fi
  } >> "$SPEC"
fi
if [ $MODE = selftest ]; then
  KVER=${SELFTEST_KVER:?selftest needs SELFTEST_KVER, the release of the host kernel QEMU boots (its modules are packed)}
  T=$W/selftest-files; rm -rf "$T"; mkdir -p "$T/mod" "$T/ip"
  : > "$T/modules.list"
  for d in /opt /opt/ip /lib /lib/modules /lib64 /usr /usr/lib; do echo "dir $d 755 0 0"; done >> "$SPEC"
  for m in dummy_hcd usb_f_acm usb_f_ecm usb_f_rndis cdc_ether cdc_acm; do
    modprobe -S "$KVER" --show-depends "$m" | sed -n 's/^insmod //p'
  done | awk '!seen[$1]++' | while read -r ko; do
    b=$(basename "$ko" .zst); b=${b%.ko}
    case $ko in *.zst) zstd -dqc "$ko" > "$T/mod/$b.ko" ;; *) cp "$ko" "$T/mod/$b.ko" ;; esac
    echo "/lib/modules/$b.ko" >> "$T/modules.list"
  done
  while read -r f; do echo "file $f $T/mod/$(basename "$f") 644 0 0"; done < "$T/modules.list" >> "$SPEC"
  echo "file /modules.list $T/modules.list 644 0 0" >> "$SPEC"
  echo "file /selftest.sh $SRC/selftest.sh 755 0 0" >> "$SPEC"
  echo "file /udhcpc.script $SRC/udhcpc.script 755 0 0" >> "$SPEC"
  IPBIN=$(command -v ip)
  echo "file /opt/ip/ip $IPBIN 755 0 0" >> "$SPEC"
  ldd "$IPBIN" | awk '/=>/ {print $3} /^[ \t]*\/lib/ {print $1}' | sort -u | while read -r so; do
    [ -f "$so" ] || continue
    echo "file $so $so 755 0 0"
  done > "$T/libs.spec"
  # libraries go where the loader looks: /usr/lib and /lib64 (Arch's /lib is a link to /usr/lib)
  awk '{n=split($2,a,"/"); print "file /usr/lib/" a[n] " " $3 " " $4 " " $5 " " $6}' "$T/libs.spec" >> "$SPEC"
  awk '{n=split($2,a,"/"); if (a[n] ~ /^ld-linux/) print "slink /lib64/" a[n] " /usr/lib/" a[n] " 777 0 0"}' "$T/libs.spec" >> "$SPEC"
fi
if [ $MODE = ufs-root-selftest ]; then
  KVER=${SELFTEST_KVER:?ufs-root-selftest needs SELFTEST_KVER, the release of the host kernel QEMU boots (its modules are packed)}
  T=$W/ufs-root-selftest-files; rm -rf "$T"; mkdir -p "$T/mod"
  : > "$T/modules.list"
  for d in /lib /lib/modules; do echo "dir $d 755 0 0"; done >> "$SPEC"
  for m in virtio_pci virtio_blk crc32c_generic ext4 overlay i6300esb; do
    modprobe -S "$KVER" --show-depends "$m" | sed -n "s/^insmod //p"
  done | awk "!seen[\$1]++" | while read -r ko; do
    b=$(basename "$ko" .zst); b=${b%.ko}
    case $ko in *.zst) zstd -dqc "$ko" > "$T/mod/$b.ko" ;; *) cp "$ko" "$T/mod/$b.ko" ;; esac
    echo "/lib/modules/$b.ko" >> "$T/modules.list"
  done
  while read -r f; do echo "file $f $T/mod/$(basename "$f") 644 0 0"; done < "$T/modules.list" >> "$SPEC"
  echo "file /modules.list $T/modules.list 644 0 0" >> "$SPEC"
fi
"$GEN" -t 0 "$SPEC" | gzip -9n > "$OUT"
ls -l "$OUT"
