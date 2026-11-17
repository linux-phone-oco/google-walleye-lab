#!/bin/sh
# What each bisect image must be, counted: against the control, exactly the parts named differ and no other. Per image:
# header fields (offsets, page size, os word, command line) equal the control's; kernel lz4 frame, dtb and ramdisk
# equal the control's unless the image swaps that part (s1 ramdisk, s2 kernel, s3 dtb); the AVB footer verifies
# (s0n has none); the size is the control's (footer) or the unpadded size (s0n).
#
#   check-bisect.sh CONTROL_BOOT_IMG OUT_DIR       (OUT_DIR as build-bisect.sh wrote it)
# Prints "ok"/"FAIL" per check; exits 1 on any failure.
set -u
CTRL=${1:?usage: check-bisect.sh CONTROL_BOOT_IMG OUT_DIR}
OUT=${2:?}
P=$OUT/parts
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
pass=0; fail=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; pass=$((pass+1)); else echo "FAIL $1: got '$2', want '$3'"; fail=$((fail+1)); fi; }
h() { sha256sum "$1" | cut -c1-16; }
CI=$(unpack_bootimg --boot_img "$CTRL" --out "$T/c")
hdr() { echo "$1" | grep -v '^boot magic' ; }
CSIZE=$(stat -c %s "$CTRL")
split() {  # split KERNEL_BLOB PREFIX: the lz4 frame and the dtb that ends it
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  head -c "${o:-0}" "$1" > "$2.lz4"; tail -c +$((${o:-0}+1)) "$1" > "$2.dtb"
}
split "$T/c/kernel" "$T/c/k"
for i in s0 s0n s1 s2 s3; do
  IMG=$OUT/boot-$i.img
  echo "-- $i"
  UI=$(unpack_bootimg --boot_img "$IMG" --out "$T/$i" 2>&1)
  ck "$i header equals the control's (offsets, page size, os word, command line)" "$(hdr "$UI" | grep -v -e 'kernel_size' -e 'ramdisk size')" "$(hdr "$CI" | grep -v -e 'kernel_size' -e 'ramdisk size')"
  split "$T/$i/kernel" "$T/$i/k"
  want_rd=c; want_k=c; want_d=c
  case $i in s1) want_rd=our ;; s2) want_k=our ;; s3) want_d=our ;; esac
  if [ $want_rd = c ]; then ck "$i ramdisk is the control's" "$(h "$T/$i/ramdisk")" "$(h "$T/c/ramdisk")"; else ck "$i ramdisk is ours, not the control's" "$(h "$T/$i/ramdisk")" "$(h "$P/our-ramdisk")"; fi
  if [ $want_k = c ]; then ck "$i kernel lz4 frame is the control's" "$(h "$T/$i/k.lz4")" "$(h "$T/c/k.lz4")"; else ck "$i kernel lz4 frame is ours" "$(h "$T/$i/k.lz4")" "$(h "$P/our-kernel.lz4")"; fi
  if [ $want_d = c ]; then ck "$i dtb is the control's" "$(h "$T/$i/k.dtb")" "$(h "$T/c/k.dtb")"; else ck "$i dtb is ours" "$(h "$T/$i/k.dtb")" "$(h "$P/our.dtb")"; fi
  ck "$i kernel frame decompresses" "$(lz4 -dc "$T/$i/k.lz4" 2>/dev/null | head -c 2)" "MZ"
  ck "$i dtb reads" "$(fdtdump "$T/$i/k.dtb" >/dev/null 2>&1 && echo yes || echo no)" yes
  if [ $i = s0n ]; then
    ck "$i has no AVB footer" "$(avbtool info_image --image "$IMG" >/dev/null 2>&1 && echo yes || echo no)" no
    ck "$i size fits the partition ($(stat -c %s "$IMG"))" "$([ "$(stat -c %s "$IMG")" -le "$CSIZE" ] && echo yes || echo no)" yes
  else
    ck "$i size is the control's ($CSIZE)" "$(stat -c %s "$IMG")" "$CSIZE"
    mkdir -p "$T/v$i"; cp "$IMG" "$T/v$i/boot.img"   # verify_image looks the partition up by file name
    ck "$i AVB footer verifies" "$(avbtool verify_image --image "$T/v$i/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
    ck "$i AVB partition name boot, algorithm NONE" "$(avbtool info_image --image "$IMG" | grep -c -e 'Partition Name: *boot' -e 'Algorithm: *NONE')" 2
  fi
done
echo "bisect images: $pass ok, $fail failed"
[ $fail -eq 0 ]
