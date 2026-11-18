#!/bin/sh
# ufs-lsblk-selftest (backlog walleye-ufs-readonly): proves init-z11's `ufs-lsblk` and `ufs-hdr` readers and the banner's UFS line on the host, never touching the phone.
#   sh tests/host/ufs-lsblk-selftest.sh   (busybox ash if ~/Projects/walleye-build/busybox-x86_64 exists, else sh)
# The two scripts are cut out of init-z11 (the heredocs between "cat > /bin/NAME" and EOS) and run against a fake sysfs (SYS=...) shaped like the kernel's: class/block/sdX{,N}/{size,uevent,partition,device/{vendor,model,rev}}
# and a fake /dev (DEVDIR=..., ANYFILE=1: a plain file stands in for the block node). Checks: disks and partitions listed with their GPT names, no disk says luns=0, the header fields read from one partition, a missing
# partition and a short read say so, nothing in the trees changes (reads only), init-z11 mounts nothing of the disks and writes to no disk.
set -u
H=$(cd "$(dirname "$0")" && pwd)
INIT=$H/../../build/initramfs-mainline/init-z11
BBX=$HOME/Projects/walleye-build/busybox-x86_64
SH=sh; [ -x "$BBX" ] && SH="$BBX ash"
[ -f "$INIT" ] || { echo "FAIL no $INIT"; exit 1; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
for n in ufs-lsblk ufs-hdr; do sed -n "/^cat > \\/bin\\/$n <<.EOS.\$/,/^EOS\$/p" "$INIT" | sed '1d;$d' > "$T/$n"; done
ck "both readers were found in init-z11" "$([ "$(wc -l < "$T/ufs-lsblk")" -gt 5 ] && [ "$(wc -l < "$T/ufs-hdr")" -gt 5 ] && echo yes || echo no)" yes
S=$T/sys/class; mkdir -p "$S/block" "$S/scsi_host/host0" "$T/dev"
disk() { mkdir -p "$S/block/$1/device"; echo "$2" > "$S/block/$1/size"; printf '%s\n' "$3" > "$S/block/$1/device/vendor"; printf '%s\n' "$4" > "$S/block/$1/device/model"; printf '%s\n' "$5" > "$S/block/$1/device/rev"; }
part() { mkdir -p "$S/block/$1"; echo 1 > "$S/block/$1/partition"; echo "$2" > "$S/block/$1/size"; printf 'MAJOR=8\nMINOR=1\nDEVNAME=%s\nPARTNAME=%s\n' "$1" "$3" > "$S/block/$1/uevent"; }
OUT0=$(SYS=$T/sys $SH "$T/ufs-lsblk")
ck "no disk: luns=0 and says so (not known yet, not an error)" "$OUT0" "ufs-lsblk hosts=[host0] luns=0 parts=0"
disk sda 125829120 "SAMSUNG " "KLUBG4G1CE-B0B1" "0100"; disk sdb 8192 "SAMSUNG " "KLUBG4G1CE-B0B1" "0100"
part sda1 2048 modem_a; part sda2 131072 boot_a; part sda3 131072 boot_b; part sdb1 4096 xbl_a
touch "$T/mark"; sleep 1
OUT=$(SYS=$T/sys $SH "$T/ufs-lsblk")
ck "summary line: two LUNs, four partitions" "$(printf '%s\n' "$OUT" | sed -n 1p)" "ufs-lsblk hosts=[host0] luns=2 parts=4"
ck "disk line carries size, vendor, model, rev" "$(printf '%s\n' "$OUT" | grep '^ufs-lsblk disk=sda ')" "ufs-lsblk disk=sda size=125829120 vendor=[SAMSUNG] model=[KLUBG4G1CE-B0B1] rev=[0100]"
ck "partition line carries the GPT name" "$(printf '%s\n' "$OUT" | grep '^ufs-lsblk part=sda2 ')" "ufs-lsblk part=sda2 name=boot_a size=131072"
ck "one line per disk and partition plus the summary" "$(printf '%s\n' "$OUT" | wc -l | tr -d ' ')" 7
# the header reader
printf 'ANDROID!' > "$T/dev/sda2"
for w in 00ab0000 00800000 00200000 01000000 00000000 00f00000 00000100 00001000 00000000 0f000000 00000000 00000000; do printf '%s' "$w" | sed 's/\(..\)\(..\)\(..\)\(..\)/\4\3\2\1/' | tr -d '\n' | sed 's/\(..\)/\\x\1/g' > "$T/w"; printf "$(cat "$T/w")" >> "$T/dev/sda2.tail"; done
cat "$T/dev/sda2.tail" >> "$T/dev/sda2"; rm "$T/dev/sda2.tail"
HD=$(TMPF=$T/hdr SYS=$T/sys DEVDIR=$T/dev ANYFILE=1 $SH "$T/ufs-hdr" boot_a)
ck "ufs-hdr: the header fields in order (each word as the kernel reads it, hex)" "$HD" "ufs-hdr part=boot_a dev=$T/dev/sda2 magic=ANDROID! kernel_size=00ab0000 kernel_addr=00800000 ramdisk_size=00200000 ramdisk_addr=01000000 second_size=00000000 second_addr=00f00000 tags_addr=00000100 page_size=00001000 header_version=00000000"
ck "ufs-hdr: a name that no partition has" "$(SYS=$T/sys DEVDIR=$T/dev ANYFILE=1 $SH "$T/ufs-hdr" nosuch)" "ufs-hdr none nosuch (no partition of that name)"
ck "ufs-hdr: no block node for the partition (the init has not got /dev/sda3)" "$(SYS=$T/sys DEVDIR=$T/dev $SH "$T/ufs-hdr" boot_b)" "ufs-hdr none boot_b (no block node $T/dev/sda3)"
: > "$T/dev/sda3"
ck "ufs-hdr: a short read says so" "$(TMPF=$T/hdr SYS=$T/sys DEVDIR=$T/dev ANYFILE=1 $SH "$T/ufs-hdr" boot_b)" "ufs-hdr none boot_b (read gave 0 words)"
ck "the readers changed no file of the fake sysfs (reads only)" "$(cd "$T/sys" && find . -type f -newer "$T/mark" | wc -l | tr -d ' ')" 0
# the banner part
sed -n '/^      U=\$(cat \$UFSF/,/WALLEYE-BANNER-UFS/p' "$INIT" | sed 's/^      //' > "$T/bannerpart.sh"
ck "the banner part was found in init-z11 (2 lines)" "$(wc -l < "$T/bannerpart.sh" | tr -d ' ')" 2
printf '%s\n' "ufs-lsblk hosts=[host0] luns=1 parts=1" "ufs-lsblk part=sda1 name=boot_a size=1" > "$T/ufs"
L=$( ( TTY=/dev/stdout; b=4; UFSF=$T/ufs; . "$T/bannerpart.sh" ) | grep WALLEYE-BANNER-UFS )
ck "the banner line: starts WALLEYE-BANNER-UFS, one line, ends with ]" "$L" "WALLEYE-BANNER-UFS beat=4 [ufs-lsblk hosts=[host0] luns=1 parts=1|ufs-lsblk part=sda1 name=boot_a size=1]"
L=$( ( TTY=/dev/stdout; b=1; UFSF=$T/none; . "$T/bannerpart.sh" ) | grep WALLEYE-BANNER-UFS )
ck "no reading yet: the banner says not read yet" "$L" "WALLEYE-BANNER-UFS beat=1 [ufs-lsblk not read yet]"
# the init writes nothing to a disk
ck "init-z11 mounts no disk and writes to none (no mount of sd*, no > /dev/sd, no dd of=, no mkfs, no blkdiscard, no sg)" "$(grep -v '^[[:space:]]*#' "$INIT" | grep -c -e 'mount[^|]* /dev/sd' -e '> */dev/sd' -e 'dd [^|]*of=/dev' -e 'dd [^|]*of="\$dev' -e 'mkfs' -e 'blkdiscard' -e 'sg_' -e '/sys/class/block/[^ ]* *>' -e 'echo [^ ]* *> */sys/class/scsi' -e 'delete')" 0
ck "init-z11 sets every disk read-only (blockdev --setro)" "$(grep -v "^[[:space:]]*#" "$INIT" | grep -c "blockdev --setro")" 1
ck "init-z11 keeps the apps watchdog petter and the 90-beat self-reboot" "$(grep -c -e 'while \[ \$n -lt 90 \]' -e 'self-reboot now' "$INIT")" 2
ck "init-z11 busybox ash -n" "$($SH -n "$INIT" 2>&1 || echo syntax-error)" ""
[ $total -eq 0 ] || { echo "ufs-lsblk-selftest: $total FAIL"; exit 1; }
echo "ufs-lsblk-selftest: all ok"
