#!/bin/sh
# z6 (backlog walleye-command-loop): z5 with ONE change, the ramdisk's /init: initramfs-mainline/init-z6 (z5's banner and gadget, the console block REPLACED by a plain read/exec command loop on ttyGS0, no interactive ash)
# instead of init-z5. Kernel, dtb, header, command line and AVB footer are z5's (which are z3's). Host only; nothing touches the phone.
#   build-z6.sh WORK_DIR Z5_BOOT_IMG OUT_DIR
# Writes OUT_DIR/{boot-z6.img,initramfs-z6.cpio.xz,SHA256SUMS}; checks (ok/FAIL, exit 1 on a FAIL, nothing written then).
# The ramdisk is packed the way build-z5.sh packs it; the first check packs init-z5 and finds z5's ramdisk byte for byte, so the only difference is the init.
set -eu
W=${1:?usage: build-z6.sh WORK_DIR Z5_BOOT_IMG OUT_DIR}
Z5=${2:?}
OUT=${3:?}
HERE=$(cd "$(dirname "$0")" && pwd)
BB=$W/busybox-arm64
GEN=$W/out/b/usr/gen_init_cpio
I=$HERE/initramfs-mainline/init-z6
OLD=$HERE/initramfs-mainline/init-z5
BANNER=$HERE/initramfs-mainline/init-banner
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$GEN" "$BB" "$Z5" "$I" "$OLD" "$BANNER" "$W/busybox-x86_64"; do [ -s "$f" ] || { echo "build-z6: missing $f" >&2; exit 1; }; done
echo "input $(sha256sum "$Z5" | cut -c1-16)  $Z5"
INFO=$(unpack_bootimg --boot_img "$Z5" --out "$T/z5")
field() { printf '%s\n' "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
PSIZE=$(stat -c %s "$Z5")
AV=$(avbtool info_image --image "$Z5")
pack() { # INIT OUT_XZ
  {
    for d in /bin /sbin /etc /dev /proc /sys /tmp /run; do echo "dir $d 755 0 0"; done
    echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"
    echo "file /bin/busybox $BB 755 0 0"; echo "slink /bin/sh busybox 777 0 0"; echo "file /init $1 755 0 0"
  } > "$T/spec"
  "$GEN" -t 0 "$T/spec" > "$T/rd.cpio"
  xz --check=crc32 --lzma2=preset=6e -c "$T/rd.cpio" > "$2"
}
pack "$OLD" "$T/old.xz"
ck "packing init-z5 gives z5's ramdisk byte for byte (the packing is the same)" "$(cmp -s "$T/old.xz" "$T/z5/ramdisk" && echo same || echo differs)" same
mkdir -p "$T/o"; pack "$I" "$T/o/initramfs-z6.cpio.xz"
IMG=$T/boot-z6.img
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(printf '%s\n' "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/z5/kernel" --ramdisk "$T/o/initramfs-z6.cpio.xz" -o "$IMG"
sh "$(dirname "$0")/ufs-gate.sh" "$IMG" || exit 1
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(printf '%s\n' "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
hdr() { printf '%s\n' "$1" | grep -v -e 'ramdisk size'; }
ck "kernel (Image.gz + dtb) bytes equal z5's" "$(cmp -s "$T/z5/kernel" "$T/u/kernel" && echo same || echo differs)" same
ck "header equals z5's but the ramdisk size" "$(hdr "$U")" "$(hdr "$INFO")"
ck "command line equals z5's" "$(field 'command line args')" "$(printf '%s\n' "$U" | sed -n 's/^command line args: *//p' | head -n1)"
ck "ramdisk differs from z5's (the one change)" "$(cmp -s "$T/z5/ramdisk" "$T/u/ramdisk" && echo same || echo differs)" differs
ck "ramdisk is this build's xz" "$(cmp -s "$T/u/ramdisk" "$T/o/initramfs-z6.cpio.xz" && echo same || echo differs)" same
mkdir "$T/x" "$T/y" && (cd "$T/x" && xz -dc "$T/u/ramdisk" | cpio -id --quiet 2>/dev/null || true) && (cd "$T/y" && xz -dc "$T/z5/ramdisk" | cpio -id --quiet 2>/dev/null || true)
ck "the init in the ramdisk is init-z6" "$(cmp -s "$T/x/init" "$I" && echo same || echo differs)" same
ck "the busybox in it is z5's" "$(cmp -s "$T/x/bin/busybox" "$T/y/bin/busybox" && echo same || echo differs)" same
ck "ramdisk lists the same files as z5's" "$(xz -dc "$T/u/ramdisk" | cpio -t 2>/dev/null | sort | tr '\n' ' ')" "$(xz -dc "$T/z5/ramdisk" | cpio -t 2>/dev/null | sort | tr '\n' ' ')"
ck "busybox in it is aarch64 static" "$(file -b "$T/x/bin/busybox" | cut -d, -f1-2)" "ELF 64-bit LSB executable, ARM aarch64"
for a in ash mount mkdir ls ln cut head tail wc uname reboot sync sleep true kill mv tr stty grep cat; do
  ck "busybox has the applet $a (arm64 build config)" "$(grep -cx "CONFIG_$(echo "$a" | tr a-z A-Z)=y" "$W/busybox-arm64.src/.config")" 1
done
for o in FEATURE_FANCY_SLEEP ASH_CMDCMD ASH_PRINTF; do ck "busybox config has $o (the loop relies on it)" "$(grep -cx "CONFIG_$o=y" "$W/busybox-arm64.src/.config")" 1; done
ck "busybox --list exists (the installer is in; the info line needs it)" "$(grep -cx 'CONFIG_FEATURE_INSTALLER=y' "$W/busybox-arm64.src/.config")" 1
ck "init has no bashisms (busybox ash -n)" "$("$W/busybox-x86_64" ash -n "$I" 2>&1 || echo syntax-error)" ""
MISS=0; while IFS= read -r l; do grep -qxF -- "$l" "$I" || { MISS=$((MISS+1)); echo "  not in init-z6: $l"; }; done <<EOL
$(grep -E '^(echo|mkdir|ln|true|mount)[^#]*(\$G|/proc|/sys|/dev|BANNERLOG)' "$BANNER" | grep -v -e 'strings/0x409/product' -e '^echo "walleye-banner' )
EOL
NL=$(grep -E '^(echo|mkdir|ln|true|mount)[^#]*(\$G|/proc|/sys|/dev|BANNERLOG)' "$BANNER" | grep -v -e 'strings/0x409/product' -e '^echo "walleye-banner' | wc -l)
ck "init-banner's gadget and mount lines are all in init-z6 unchanged (the product string aside; at least 10 compared)" "$MISS $([ "$NL" -ge 10 ] && echo enough || echo few)" "0 enough"
ck "the banner line keeps init-banner's fields (beat up steps last)" "$(grep -c 'WALLEYE-BANNER beat=\$b up=\$(cut -d. . -f1 /proc/uptime)s steps=\$(wc -l < \$BANNERLOG) last=\[\$(tail -n1 \$BANNERLOG)\] loop=\[' "$I")" 1
ck "the banner line ends with the loop's state (started starts cmds last rc err)" "$(grep -c 'loop=\[started=\$S starts=\$N cmds=\$C last=\[\$LC\] rc=\$LR err=\[\$E\]\]' "$I")" 1
ck "the banner has no end marker of any kind (no END in a banner line)" "$(grep -E 'printf .*WALLEYE-BANNER' "$I" | grep -c 'END')" 0
ck "the console is a command loop: it reads a line from fd 4 and runs it with sh -c, answers on fd 4" "$(grep -c 'IFS= read -r line <&4' "$I")$(grep -c 'sh -c "\$line" < /dev/null > /tmp/out 2>&1' "$I")$(grep -c 'cat /tmp/out >&4' "$I")" 111
ck "the console sets the tty raw, no echo, no hangup" "$(grep -c 'stty -F \$TTY raw -echo -hupcl' "$I")" 1
ck "the console keeps the port open once per open (command exec 4<>, stderr to sh.err)" "$(grep -c '{ command exec 4<> \$TTY; } 2>> \$SH.err' "$I")" 1
ck "no interactive shell, no job control, no cttyhack, no setsid in the code lines" "$(grep -v '^[[:space:]]*#' "$I" | grep -c -e 'ash -i' -e 'cttyhack' -e 'setsid')" 0
ck "no gadget but acm.GS0 (banner2's one function)" "$(grep -c 'mass_storage' "$I")" 0
ck "every init of ours writes the serial walleye-first-boot (check-init-serials.sh)" "$(sh "$HERE/check-init-serials.sh" "$HERE" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "the host selftest of the command loop under a pty passes (z6-loop-selftest.sh, 'a line in gets an answer out')" "$(sh "$HERE/../tests/host/z6-loop-selftest.sh" "$W/busybox-x86_64" 2>&1 | tail -n1)" "z6-loop-selftest: all ok"
ck "the host selftest of the banner-loop check passes" "$(sh "$HERE/../tests/host/banner-loop-selftest.sh" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "the host selftest of the reader passes (it leaves banner lines out)" "$(sh "$HERE/../tests/host/mainline-shell-selftest.sh" 2>&1 | tail -n1 | cut -c1-41)" "pass walleye-mainline-shell-selftest host"
ck "size is z5's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "AVB info equals z5's but the digest and the image sizes" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')"
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "build-z6: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-z6.img"; cp "$T/o/initramfs-z6.cpio.xz" "$OUT/"
(cd "$OUT" && sha256sum initramfs-z6.cpio.xz boot-z6.img | tee SHA256SUMS)
