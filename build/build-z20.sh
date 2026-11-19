#!/bin/sh
# z20 (backlog walleye-build-gpio-diag-kernel): z19 with a DIAGNOSTIC kernel and a longer command line; the dtb is z19's, byte for byte.
# Why: the touch power gpio defers for good (research walleye-touch-gpio-defer-leaves) at one of two leaves nobody can tell apart by reading source. The kernel here is the same tree as z15's (linux-phone-oco/linux
# topic/pmic-wdt-wake-in-suspend, e123d3bff) with build/walleye-diag.config on top (CONFIG_DYNAMIC_DEBUG, CONFIG_DEBUG_GPIO), and the command line adds dyndbg= for gpiolib, gpiolib-of, pinctrl core and the spmi-gpio
# driver (full paths: a dyndbg file query is a whole-string match, "pinctrl/core.c" matches nothing). gpiolib-of prints the errno of a failed gpio lookup; pinctrl core and devicetree print their steps; the spmi-gpio driver has no debug lines of its own, so its +p is harmless and prints nothing. CONFIG_DEBUG_GPIO also compiles drivers/gpio with -DDEBUG: more boot output from every gpio driver, and gpiolib's own checks - z20 differs from z19 in those as well. Ramdisk (z14's init), header but the sizes, and the AVB footer's properties are z19's.
# Built only - no phone run; the keeper tries it once (queued with ikb backlog-needs).
#   build-z20.sh Z19_BOOT_IMG KERNEL_OUT_DIR OUT_DIR
# KERNEL_OUT_DIR is build-kernel.sh's OUT_DIR, run with EXTRA_CONFIG=build/walleye-diag.config (Image.gz and .config). Writes OUT_DIR/{boot-z20.img,SHA256SUMS-z20}; checks (ok/FAIL, exit 1 on a FAIL, nothing written then).
set -eu
Z19=${1:?usage: build-z20.sh Z19_BOOT_IMG KERNEL_OUT_DIR OUT_DIR}
KO=${2:?}
OUT=${3:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
DYNDBG='dyndbg="file drivers/gpio/gpiolib.c +p; file drivers/gpio/gpiolib-of.c +p; file drivers/pinctrl/core.c +p; file drivers/pinctrl/devicetree.c +p; file drivers/pinctrl/qcom/pinctrl-spmi-gpio.c +p"'
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
h() { sha256sum "$1" | cut -c1-16; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$Z19" "$KO/Image.gz" "$KO/.config"; do [ -s "$f" ] || { echo "build-z20: missing $f" >&2; exit 1; }; done
split() { # KERNEL_BLOB PREFIX: PREFIX.gz is the Image.gz, PREFIX.dtb the dtb that ends the blob
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-z20: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
echo "input $(h "$Z19")  $Z19"
INFO=$(unpack_bootimg --boot_img "$Z19" --out "$T/z19")
field() { printf "%s\n" "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split "$T/z19/kernel" "$T/k"
PSIZE=$(stat -c %s "$Z19")
AV=$(avbtool info_image --image "$Z19")
OLDCMD=$(field 'command line args')
NEWCMD="$OLDCMD $DYNDBG"
# the kernel's config
for s in DYNAMIC_DEBUG DEBUG_GPIO PINCTRL_QCOM_SPMI_PMIC PINCTRL_MSM8998 RMI4_I2C QCOM_WDT; do
  ck "the kernel's CONFIG_$s is y" "$(sed -n "s/^CONFIG_$s=//p" "$KO/.config")" y
done
ck "the z19 command line has no dyndbg" "$(printf '%s\n' "$OLDCMD" | grep -c dyndbg || true)" 0
ck "the new Image.gz is not z19's" "$(cmp -s "$KO/Image.gz" "$T/k.gz" && echo same || echo differs)" differs
cat "$KO/Image.gz" "$T/k.dtb" > "$T/blob"
hdr() { printf "%s\n" "$1" | grep -v -e '^kernel_size' -e '^ramdisk size' -e '^kernel size' -e '^command line args'; }
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(printf "%s\n" "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$T/boot-z20.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$NEWCMD" \
  --kernel "$T/blob" --ramdisk "$T/z19/ramdisk" -o "$IMG"
sh "$(dirname "$0")/ufs-gate.sh" "$IMG" || exit 1
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(printf "%s\n" "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
split "$T/u/kernel" "$T/uk"
ck "z20 Image.gz is the diag kernel's, byte for byte" "$(cmp -s "$T/uk.gz" "$KO/Image.gz" && echo same || echo differs)" same
ck "z20 dtb is z19's, byte for byte" "$(cmp -s "$T/uk.dtb" "$T/k.dtb" && echo same || echo differs)" same
ck "z20 header equals z19's but the sizes and the command line" "$(hdr "$U")" "$(hdr "$INFO")"
ck "z20 command line is z19's plus the dyndbg word" "$(printf "%s\n" "$U" | sed -n 's/^command line args: *//p' | head -n1)" "$NEWCMD"
ck "z20 ramdisk bytes equal z19's" "$(cmp -s "$T/u/ramdisk" "$T/z19/ramdisk" && echo same || echo differs)" same
ck "z20 size is z19's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "z20 AVB info equals z19's but the digest and the sizes of the kernel it hashes" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:' -e '^Original image size' -e '^VBMeta offset' -e '^ *Image Size:')" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:' -e '^Original image size' -e '^VBMeta offset' -e '^ *Image Size:')"
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
KS=$(stat -c %s "$T/blob"); KO_=$(printf %d "$(field "kernel load address")"); RO=$(printf %d "$(field "ramdisk load address")")
ck "the kernel blob ends before the ramdisk loads (headroom in bytes > 524288)" "$([ $((RO - KO_ - KS)) -gt 524288 ] && echo yes || echo "no: $((RO - KO_ - KS))")" yes
ck "z20 AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "the host test passes on z20 (tests/host/gpio-diag-boot.sh)" "$(IMG="$IMG" sh "$HERE/tests/host/gpio-diag-boot.sh" | cut -d' ' -f1)" pass
[ $total -eq 0 ] || { echo "build-z20: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-z20.img"
(cd "$OUT" && sha256sum boot-z20.img | tee SHA256SUMS-z20)
