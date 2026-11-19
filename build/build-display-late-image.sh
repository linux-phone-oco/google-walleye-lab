#!/bin/sh
# The boot image of the display late probe (item walleye-display-late-probe-image): z21's header, command line and AVB footer, the late-probe kernel (display path as modules, netconsole built in) with the display dtb,
# and a ramdisk of init-late (z14's init, its self-reboot held while /tmp/hold exists) with the display modules, their depmod output and /display-modules.list (insmod lines, dependency order). Nothing display-related probes at boot:
# the host script tests/host/display-late-probe.sh loads the modules one at a time with the log streamed. UFS is off in the dtb (the gate, checked here on the finished image). Built only - no phone run; the keeper tries it once (walleye-try-display-late-probe).
#   build-display-late-image.sh Z21_BOOT_IMG LATE_DIR OUT_DIR      (LATE_DIR: build-display-late.sh's OUT_DIR; WORK_DIR with busybox-arm64 and a built gen_init_cpio: environment WORK, default ~/Projects/walleye-build)
# The dtb in the image gets clock_gcc at the real gcc (fdtput), checked before and after. Writes OUT_DIR/{boot-display-late.img,SHA256SUMS-display-late-img}; every check stops the build (ok/FAIL, exit 1, nothing written).
set -eu
Z21=${1:?usage: build-display-late-image.sh Z21_BOOT_IMG LATE_DIR OUT_DIR}
D=${2:?}
OUT=${3:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
W=${WORK:-$HOME/Projects/walleye-build}
BB=$W/busybox-arm64
GEN=${GEN:-$D/b/usr/gen_init_cpio}
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$Z21" "$D/Image.gz" "$D/msm8998-google-walleye-display.dtb" "$D/display-modules.list" "$D/.config" "$D/System.map" "$BB" "$GEN"; do [ -s "$f" ] || [ -x "$f" ] || { echo "build-display-late-image: missing $f" >&2; exit 1; }; done
INFO=$(unpack_bootimg --boot_img "$Z21" --out "$T/z21")
field() { printf "%s\n" "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
PSIZE=$(stat -c %s "$Z21")
AV=$(avbtool info_image --image "$Z21")
GCC=/soc@0/clock-controller@100000
DTB=$T/display.dtb
cp "$D/msm8998-google-walleye-display.dtb" "$DTB"
ck "the built display dtb has clock_gcc at the empty stub (the fix is needed)" "$(fdtget "$DTB" /__symbols__ clock_gcc)" /vendor-overlay-stubs/clock_gcc
fdtput -t s "$DTB" /__symbols__ clock_gcc "$GCC"
ck "clock_gcc is the real gcc in the image dtb" "$(fdtget "$DTB" /__symbols__ clock_gcc)" "$GCC"
# the dtb is the display dtb: mdss, the display controller, the DSI host, its PHY okay, the panel node present
st() { fdtget "$DTB" "$(fdtget "$DTB" /__symbols__ "$1")" status 2>/dev/null || echo absent; }
ck "mdss, display controller, DSI host and PHY are okay in the dtb" "$(st mdss) $(st mdss_mdp) $(st mdss_dsi0) $(st mdss_dsi0_phy)" "okay okay okay okay"
ck "the panel node is in the dtb" "$(fdtget -l "$DTB" "$(fdtget "$DTB" /__symbols__ mdss_dsi0)" | grep -c '^panel@0$')" 1
# the kernel: the config, and that the display driver is a module, not in the image
for s in NETCONSOLE NETCONSOLE_DYNAMIC DYNAMIC_DEBUG USB_CONFIGFS_ECM CONFIGFS_FS DRM_MSM_DPU; do ck "CONFIG_$s is y" "$(sed -n "s/^CONFIG_$s=//p" "$D/.config")" y; done
for s in DRM DRM_MSM DRM_PANEL_SAMSUNG_S6E3FA5; do ck "CONFIG_$s is m" "$(sed -n "s/^CONFIG_$s=//p" "$D/.config")" m; done
zcat "$D/Image.gz" > "$T/kernel"
ck "the kernel holds no display driver: dpu_kms_init is not in System.map" "$(grep -c " dpu_kms_init" "$D/System.map" || true)" 0
ck "the kernel image holds netconsole" "$(grep -ac 'netconsole' "$T/kernel" | sed 's/^[1-9][0-9]*$/yes/')" yes
# the modules: every one of the list is a file of the staging root, and every file of the root is in the list (nothing else goes into the ramdisk)
ROOT=$D/modroot
KVER=$(ls "$ROOT/lib/modules")
ck "the kernel's version string is the modules' ($KVER)" "$(grep -ao "Linux version $KVER" "$T/kernel" | head -n1 | sed 's/Linux version //')" "$KVER"
ck "the modules' depmod output is in the root" "$(ls "$ROOT/lib/modules/$KVER" | grep -c '^modules.dep$')" 1
ck "the list holds every .ko of the root exactly once" "$(sed -n 's/^insmod //p' "$D/display-modules.list" | sort | cmp -s - "$(cd "$ROOT" && find . -name '*.ko' | sed 's#^\.##' | sort > "$T/kos"; echo "$T/kos")" && echo same || echo differs)" same
ck "every module comes after all the modules it needs (modules.dep of the root against the list's order): violations" "$(sed -n 's#^insmod /lib/modules/[^/]*/##p' "$D/display-modules.list" | awk -v dep="$ROOT/lib/modules/$KVER/modules.dep" 'BEGIN { while ((getline l < dep) > 0) { split(l, a, ": *"); d[a[1]] = a[2] } } { pos[$0] = NR } END { for (m in pos) { n = split(d[m], x, " "); for (j = 1; j <= n; j++) if (!(x[j] in pos) || pos[x[j]] > pos[m]) bad++ } print bad + 0 }')" 0
# the ramdisk
{
  for d in /bin /sbin /etc /dev /proc /sys /tmp /run; do echo "dir $d 755 0 0"; done
  echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"
  echo "file /bin/busybox $BB 755 0 0"; echo "slink /bin/sh busybox 777 0 0"; echo "file /init $HERE/build/initramfs-mainline/init-late 755 0 0"
  echo "file /etc/udhcpd.conf $HERE/build/initramfs-mainline/udhcpd-z14.conf 644 0 0"
  echo "file /display-modules.list $D/display-modules.list 644 0 0"
  (cd "$ROOT" && find lib -type d -printf 'dir /%p 755 0 0\n')
  (cd "$ROOT" && find lib -type f -printf "file /%p $ROOT/%p 644 0 0\n")
} > "$T/spec"
"$GEN" -t 0 "$T/spec" > "$T/rd.cpio"
xz --check=crc32 --lzma2=preset=6e -c "$T/rd.cpio" > "$T/ramdisk.xz"
cat "$D/Image.gz" "$DTB" > "$T/blob"
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(printf "%s\n" "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$T/boot-display-late.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/blob" --ramdisk "$T/ramdisk.xz" -o "$IMG"
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(printf "%s\n" "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
ck "the kernel blob is Image.gz then the display dtb, byte for byte" "$(cmp -s "$T/u/kernel" "$T/blob" && echo same || echo differs)" same
ck "the ramdisk is the packed one, byte for byte" "$(cmp -s "$T/u/ramdisk" "$T/ramdisk.xz" && echo same || echo differs)" same
ck "the command line is z21's" "$(printf "%s\n" "$U" | sed -n 's/^command line args: *//p')" "$(field 'command line args')"
ck "the header words (offsets, page size, os version and patch level) are z21's" "$(printf '%s\n' "$U" | grep -e 'load address' -e 'page size' -e 'os ' | cmp -s - "$(printf '%s\n' "$INFO" | grep -e 'load address' -e 'page size' -e 'os ' > "$T/hz"; echo "$T/hz")" && echo same || echo differs)" same
ck "the size is z21's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
KS=$(stat -c %s "$T/blob"); KO_=$(printf %d "$(field "kernel load address")"); RO=$(printf %d "$(field "ramdisk load address")")
ck "the kernel blob ends before the ramdisk loads (headroom in bytes > 524288)" "$([ $((RO - KO_ - KS)) -gt 524288 ] && echo yes || echo "no: $((RO - KO_ - KS))")" yes
# what the ramdisk holds, read back from the image
mkdir "$T/rd" && (cd "$T/rd" && xz -dc "$T/u/ramdisk" | cpio -id 2>/dev/null || true)  # cpio cannot mknod as a user: the two device nodes are not checked here
ck "the ramdisk's /init is init-late, byte for byte" "$(cmp -s "$T/rd/init" "$HERE/build/initramfs-mainline/init-late" && echo same || echo differs)" same
ck "the ramdisk's /init holds its self-reboot while /tmp/hold exists" "$(grep -c 'while \[ \$n -lt 90 \] || \[ -e /tmp/hold \]' "$T/rd/init")" 1
ck "the ramdisk's /display-modules.list is the list" "$(cmp -s "$T/rd/display-modules.list" "$D/display-modules.list" && echo same || echo differs)" same
ck "every module of the list is in the ramdisk, byte for byte" "$(sed -n 's/^insmod //p' "$D/display-modules.list" | while IFS= read -r p; do cmp -s "$T/rd$p" "$ROOT$p" || echo differs; done | sort -u | tr -d '\n')" ""
ck "the ramdisk has the modules' depmod output" "$(cmp -s "$T/rd/lib/modules/$KVER/modules.dep" "$ROOT/lib/modules/$KVER/modules.dep" && echo same || echo differs)" same
ck "the ramdisk's busybox has insmod, nc, telnetd and setsid" "$(strings -n2 "$T/rd/bin/busybox" | grep -x -e insmod -e nc -e telnetd -e setsid | sort -u | wc -l)" 4
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "the AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "the UFS-off gate passes the image (tests/host/ufs-off-image.sh)" "$(sh "$HERE/tests/host/ufs-off-image.sh" "$IMG" | cut -d" " -f1 || true)" pass
ck "the dtb keeps the touch power gpio single-claimant after the overlay merge" "$(DTB="$DTB" sh "$HERE/tests/host/touch-gpio-single-claimant.sh" | cut -d' ' -f1)" pass
ck "the overlay still merges into the dtb" "$(DTB="$DTB" sh "$HERE/tests/host/dtbo-overlay-merges.sh" | cut -d' ' -f1)" pass
ck "after the overlay merge the overlay's clock users name gcc (tests/host/overlay-clock-gcc.sh)" "$(DTB="$DTB" sh "$HERE/tests/host/overlay-clock-gcc.sh" | cut -d' ' -f1)" pass
ck "every init of ours writes the serial walleye-first-boot (check-init-serials.sh)" "$(sh "$HERE/build/check-init-serials.sh" "$HERE/build" >/dev/null 2>&1 && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "build-display-late-image: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-display-late.img"
(cd "$OUT" && sha256sum boot-display-late.img | tee SHA256SUMS-display-late-img)
