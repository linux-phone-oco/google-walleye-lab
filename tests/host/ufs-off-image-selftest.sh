#!/bin/sh
# Proves tests/host/ufs-off-image.sh on dtbs and boot images made here (decision 2026-11-19-walleye-ufs-off-by-a-gate): UFS on, with no status or enabled only by the
# phone's own overlay fails; off passes; so do a dtb with no UFS node and a command line that only carries the bootloader's androidboot.* word; a second
# appended dtb with UFS on fails the whole image; a command line that names ufs fails; no dtb in an image fails; no file skips.
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
GATE=$HERE/ufs-off-image.sh
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
want() { # NAME ARG WANTED [DTBO]
  got=$(DTBO=${4:-$T/clean.dtbo} sh "$GATE" "$2" 2>&1 | head -n1 | cut -d' ' -f1 || true)
  if [ "$got" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$got', want $3"; total=$((total+1)); fi
}
dtsfile() { # OUT HOST_STATUS PHY_STATUS (empty: no status line; "none": no UFS nodes at all)
  if [ "$2" = none ]; then
    printf '/dts-v1/;\n/ { soc@0 { #address-cells = <2>; #size-cells = <2>; serial@c1b0000 { reg = <0 0xc1b0000 0 0x1000>; }; }; };\n' > "$1"
  else
    {
      printf '/dts-v1/;\n/ { soc@0 { #address-cells = <2>; #size-cells = <2>;\n'
      printf 'ufshc@1da4000 { compatible = "qcom,msm8998-ufshc", "qcom,ufshc", "jedec,ufs-2.0"; reg = <0 0x1da4000 0 0x1000>;'
      [ -z "$2" ] || printf ' status = "%s";' "$2"
      printf ' };\nphy@1da7000 { compatible = "qcom,msm8998-qmp-ufs-phy"; reg = <0 0x1da7000 0 0x1000>;'
      [ -z "$3" ] || printf ' status = "%s";' "$3"
      printf ' };\n}; };\n'
    } > "$1"
  fi
}
mkdtb() { dtsfile "$T/$1.dts" "$2" "$3"; dtc -@ -I dts -O dtb -o "$T/$1.dtb" "$T/$1.dts" 2>/dev/null; }
mkdtbo() { # OVERLAY_DTS OUT: a dtbo table of four entries, entry 3 the overlay (entries 0-2 empty), as tests/host/touch-gpio-single-claimant.sh reads it
  dtc -I dts -O dtb -o "$T/ov.dtb" "$1" 2>/dev/null
  n=$(stat -c %s "$T/ov.dtb")
  {
    printf 'd7b7ab1e%08x' $((n + 160))
    printf '%s' 00000020 00000020 00000004 00000020 00001000 00000000
    z=0; while [ $z -lt 3 ]; do printf '%064d' 0; z=$((z+1)); done
    printf '%08x%08x%048d' "$n" 160 0
  } | xxd -r -p > "$2"
  cat "$T/ov.dtb" >> "$2"
}
cat > "$T/clean.dts" <<'D'
/dts-v1/;
/plugin/;
/ { fragment@0 { target-path = "/soc@0"; __overlay__ { touch-note = <1>; }; }; };
D
cat > "$T/ufson.dts" <<'D'
/dts-v1/;
/plugin/;
/ { fragment@0 { target-path = "/soc@0/ufshc@1da4000"; __overlay__ { status = "okay"; }; }; };
D
mkdtbo "$T/clean.dts" "$T/clean.dtbo"
mkdtbo "$T/ufson.dts" "$T/ufson.dtbo"
mkdtb off disabled disabled
mkdtb on okay okay
mkdtb nostatus "" disabled
mkdtb phyon disabled okay
mkdtb phynostatus disabled ""
mkdtb noufs none ""
# dtbs
want "both disabled passes" "$T/off.dtb" pass
want "host okay fails" "$T/on.dtb" fail
want "host with no status (enabled by default) fails" "$T/nostatus.dtb" fail
want "phy okay fails" "$T/phyon.dtb" fail
want "phy with no status fails" "$T/phynostatus.dtb" fail
want "no UFS node at all passes" "$T/noufs.dtb" pass
want "off, but the phone's own overlay enables it: fails" "$T/off.dtb" fail "$T/ufson.dtbo"
want "off with an overlay that leaves UFS alone passes" "$T/off.dtb" pass "$T/clean.dtbo"
want "no file skips" "$T/missing.dtb" skip
want "no dtbo skips" "$T/off.dtb" skip "$T/missing.dtbo"
want "a file that is neither a boot image nor a dtb fails" "$GATE" fail
# boot images
head -c 4096 /dev/zero > "$T/rd"
img() { # NAME CMDLINE DTB...
  nm=$1; cl=$2; shift 2
  printf 'FAKEKERNELIMAGEGZ' > "$T/k-$nm"
  for d in "$@"; do cat "$d" >> "$T/k-$nm"; done
  mkbootimg --header_version 0 --kernel "$T/k-$nm" --ramdisk "$T/rd" --cmdline "$cl" -o "$T/boot-$nm.img"
}
img off "console=ttyMSM0 androidboot.bootdevice=1da4000.ufshc" "$T/off.dtb"
img on "console=ttyMSM0" "$T/on.dtb"
img two "console=ttyMSM0" "$T/off.dtb" "$T/on.dtb"
img twoff "console=ttyMSM0" "$T/off.dtb" "$T/noufs.dtb"
img cmd "console=ttyMSM0 ufshcd.foo=1" "$T/off.dtb"
img nodtb "console=ttyMSM0"
want "image, UFS off, only the bootloader's androidboot word on the command line, passes" "$T/boot-off.img" pass
want "image with UFS on fails" "$T/boot-on.img" fail
want "image whose second appended dtb has UFS on fails" "$T/boot-two.img" fail
want "image with two appended dtbs, both off, passes" "$T/boot-twoff.img" pass
want "image whose command line names ufs fails" "$T/boot-cmd.img" fail
want "image with no dtb fails (UFS off cannot be shown)" "$T/boot-nodtb.img" fail
want "image off, the phone's own overlay enables UFS: fails" "$T/boot-off.img" fail "$T/ufson.dtbo"
# the review's holes: status ok, a disabled parent, a node found by its name, a merge that fails, a command line past 511 characters, a file that is no dtbo
printf '/dts-v1/;\n/ { soc@0 { #address-cells = <2>; #size-cells = <2>; ufshc@1da4000 { compatible = "x,y"; reg = <0 0x1da4000 0 0x1000>; status = "ok"; }; }; };\n' > "$T/ok.dts"
dtc -@ -I dts -O dtb -o "$T/ok.dtb" "$T/ok.dts" 2>/dev/null
printf '/dts-v1/;\n/ { soc@0 { #address-cells = <2>; #size-cells = <2>; status = "disabled"; ufshc@1da4000 { compatible = "qcom,ufshc"; reg = <0 0x1da4000 0 0x1000>; status = "okay"; }; }; };\n' > "$T/parent.dts"
dtc -@ -I dts -O dtb -o "$T/parent.dtb" "$T/parent.dts" 2>/dev/null
printf '/dts-v1/;\n/ { soc@0 { #address-cells = <2>; #size-cells = <2>; ufs@1da4000 { compatible = "x,unknown"; reg = <0 0x1da4000 0 0x1000>; }; }; };\n' > "$T/byname.dts"
dtc -@ -I dts -O dtb -o "$T/byname.dtb" "$T/byname.dts" 2>/dev/null
printf '/dts-v1/;\n/plugin/;\n/ { fragment@0 { target-path = "/nowhere"; __overlay__ { x = <1>; }; }; };\n' > "$T/nomerge.dts"
mkdtbo "$T/nomerge.dts" "$T/nomerge.dtbo"
want "status ok fails (the kernel reads ok as okay)" "$T/ok.dtb" fail
want "a disabled parent blocks the node: passes" "$T/parent.dtb" pass
want "a node named ufs@... with an unknown compatible fails" "$T/byname.dtb" fail
want "an overlay that does not merge fails" "$T/off.dtb" fail "$T/nomerge.dtbo"
want "a dtbo that is no dtbo table fails" "$T/off.dtb" fail "$T/off.dtb"
long=$(printf 'a=%0300d ' 0); img long "console=ttyMSM0 $long $long ufshcd.foo=1" "$T/off.dtb"
want "a command line past 511 characters, ufs in its second part, fails" "$T/boot-long.img" fail
# the gate helper of the build scripts: a failing image is removed, a passing one stays
cp "$T/boot-on.img" "$T/gate-on.img"; cp "$T/boot-off.img" "$T/gate-off.img"
DTBO=$T/clean.dtbo sh "$HERE/../../build/ufs-gate.sh" "$T/gate-on.img" >/dev/null 2>&1 && g1=pass || g1=fail
DTBO=$T/clean.dtbo sh "$HERE/../../build/ufs-gate.sh" "$T/gate-off.img" >/dev/null 2>&1 && g2=pass || g2=fail
DTBO=$T/missing.dtbo sh "$HERE/../../build/ufs-gate.sh" "$T/boot-off.img" >/dev/null 2>&1 && g3=pass || g3=fail
[ "$g1 $g2 $g3 $([ -e "$T/gate-on.img" ] && echo kept || echo removed)" = "fail pass fail removed" ] && echo "ok   ufs-gate.sh: UFS on fails and the image is removed, off passes, a skip is not a pass" || { echo "FAIL ufs-gate.sh: $g1 $g2 $g3"; total=$((total+1)); }
# every script that writes a boot image runs the gate (z11-z13 are the UFS-on experiments, z21, the display image and the upstream-dts image run it by their own check); build-ufs-match.sh and build-ufs-root-image.sh are UFS-on images of walleye-ufs-match-android-z: it proves the gate FAILS it and writes it only to a NOT-TO-BOOT- directory
miss=$(cd "$HERE/../../build" && for s in $(grep -l 'mkbootimg' build-*.sh); do case $s in build-z11.sh|build-z12.sh|build-z13.sh|build-z21.sh|build-display-image.sh|build-upstream-dts-image.sh|build-ufs-match.sh|build-ufs-root-image.sh) continue;; esac; grep -q 'ufs-gate.sh' "$s" || echo "$s"; done)
[ -z "$miss" ] && echo "ok   every build script that runs mkbootimg runs the UFS gate" || { echo "FAIL build scripts without the gate: $miss"; total=$((total+1)); }
[ $total -eq 0 ] || { echo "ufs-off-image-selftest: $total failed" >&2; exit 1; }
echo "pass ufs-off-image-selftest"
