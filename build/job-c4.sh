#!/bin/sh
# lpoco-lab job body for c4: the dtb from our current dts WITHOUT the PMIC-watchdog block (that is z8's change, not c2's; c4 = c2 + the tps61099 model only),
# compiled in the pristine v7.3 tree c2 was built in (its pm8998.dtsi has no pm8998_wdt), then build-c4.sh on c2's image, whose Image.gz it keeps.
set -eu
W=$HOME/Projects/walleye-build
H=$(cd "$(dirname "$0")/.." && pwd)
K=$W/linux-v7.3
O=$W/out-c4
Q=$K/arch/arm64/boot/dts/qcom
mkdir -p "$O/b"
# our dts minus the block from its comment ("The PMIC's own watchdog") to the closing of &pm8998_wdt
s=$(grep -n "^ \* The PMIC's own watchdog" "$H/dt/msm8998-google-walleye.dts" | cut -d: -f1)
e=$(grep -n '^&pm8998_wdt {' "$H/dt/msm8998-google-walleye.dts" | cut -d: -f1)
[ -n "$s" ] && [ -n "$e" ] || { echo "job-c4: the watchdog block is not found in the dts" >&2; exit 1; }
sed "$((s-1)),$((e+3))d" "$H/dt/msm8998-google-walleye.dts" > "$Q/msm8998-google-walleye.dts"
cp "$H/dt/msm8998-google-walleye-overlay-stubs.dtsi" "$H/dt/msm8998-google-walleye-f.dts" "$Q/"
! grep -q pm8998_wdt "$Q/msm8998-google-walleye.dts" || { echo "job-c4: the watchdog block is still in the dts" >&2; exit 1; }
grep -q 'msm8998-google-walleye.dtb' "$Q/Makefile" || { echo "job-c4: the tree's Makefile lacks our dtb line" >&2; exit 1; }
M="make -C $K ARCH=arm64 LLVM=1 O=$O/b"
$M defconfig >/dev/null
DTC_FLAGS=-@ $M -j"$(nproc)" qcom/msm8998-google-walleye.dtb
cp "$O/b/arch/arm64/boot/dts/qcom/msm8998-google-walleye.dtb" "$O/"
"$H/build/build-c4.sh" "$W/images-c2/boot-c2.img" "$O" "$W/images-c4"
