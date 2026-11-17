#!/bin/sh
# What a walleye boot image must be, counted: magic ANDROID!, header v0, the offsets asked for, size within the 32 MiB
# boot partition (getvar boot_a 0x2000000), a device tree inside the kernel blob with qcom,msm-id <0x124 0x20001>
# and no /chosen/bootargs, the command line present, and the ramdisk a gzip cpio holding /init.
#
#   check-bootimg.sh BOOT_IMG RAMDISK_OFFSET TAGS_OFFSET [OS_VERSION OS_PATCH_LEVEL]   (none given: the header holds none)
# Prints one "ok"/"FAIL" line per check and a count; exits 1 on any failure.
set -u
IMG=${1:?usage: check-bootimg.sh BOOT_IMG RAMDISK_OFFSET TAGS_OFFSET}
RDO=${2:?}
TGO=${3:?}
OSV=${4:-}
OSP=${5:-}
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
pass=0; fail=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1: $2"; pass=$((pass+1)); else echo "FAIL $1: got '$2', want '$3'"; fail=$((fail+1)); fi; }
ck magic "$(head -c 8 "$IMG")" 'ANDROID!'
SIZE=$(stat -c %s "$IMG")
ck "size fits 32 MiB ($SIZE bytes)" "$([ "$SIZE" -le 33554432 ] && echo yes || echo no)" yes
INFO=$(unpack_bootimg --boot_img "$IMG" --out "$T/u" 2>&1)
field() { echo "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
ck header_version "$(field 'boot image header version')" 0
ck "os version" "$(field 'os version')" "${OSV:-None}"
ck "os patch level" "$(field 'os patch level')" "${OSP:-None}"
ck "page size" "$(field 'page size')" 4096
ck "kernel load address" "$(field 'kernel load address')" 0x00008000
ck "ramdisk load address" "$(field 'ramdisk load address')" "$(printf '0x%08x' "$RDO")"
ck "tags load address" "$(field 'kernel tags load address')" "$(printf '0x%08x' "$TGO")"
ck cmdline-has-earlycon "$(field 'command line args' | grep -c 'earlycon=msm_serial_dm,0xc1b0000')" 1
# the dtb: find the flattened device tree magic inside the kernel blob and read it back
K=$T/u/kernel
OFF=$(grep -abo "$(printf '\320\015\376\355')" "$K" | tail -n1 | cut -d: -f1)
if [ -n "$OFF" ]; then tail -c +$((OFF+1)) "$K" > "$T/appended.dtb"; else : > "$T/appended.dtb"; fi
ck "dtb appended after the kernel" "$([ -s "$T/appended.dtb" ] && fdtdump "$T/appended.dtb" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "qcom,msm-id" "$(fdtget -t u "$T/appended.dtb" / qcom,msm-id 2>/dev/null)" "292 131073"
ck "msm-id is 0x124/0x20001" "$(printf '0x%x 0x%x' 292 131073)" "0x124 0x20001"
ck "compatible" "$(fdtget "$T/appended.dtb" / compatible 2>/dev/null | cut -d' ' -f1)" google,walleye
ck "no bootargs in the dtb" "$(dtc -I dtb -O dts "$T/appended.dtb" 2>/dev/null | grep -c bootargs)" 0
ck "kernel is a gzip" "$(head -c 2 "$K" | od -An -tx1 | tr -d ' ')" 1f8b
ck "ramdisk has /init" "$(zcat "$T/u/ramdisk" 2>/dev/null | cpio -t 2>/dev/null | grep -cx init)" 1
echo "$IMG: $pass ok, $fail failed"
[ $fail -eq 0 ]
