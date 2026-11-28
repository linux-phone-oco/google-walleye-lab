#!/bin/sh
# The dtb of the osfact display-and-NFC image (item walleye-osfact-image-display-and-nfc): the display image's dtb (B-r5-smmu's, device.toml [images.b-r5-smmu.recipe]) with
# dt/msm8998-google-walleye-nfc.dtso applied (fdtoverlay). Nothing here touches a phone. Every check stops the build.
#   osfact-display-nfc-dtb.sh SEED_DTB NFC_DTB KERNEL_TREE OUT_DTB
#   SEED_DTB: the display dtb (c6d840a7); NFC_DTB: the NFC image's dtb (86550c1e, device.toml [images.nfc.recipe]); KERNEL_TREE: a kernel tree (its include/ for dt-bindings)
set -eu
SEED=${1:?usage: osfact-display-nfc-dtb.sh SEED_DTB NFC_DTB KERNEL_TREE OUT_DTB}; NFC=${2:?}; K=${3:?}; OUT=${4:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
want() { awk -v n="$1" '$0 ~ "^\\[images\\." n "\\.recipe\\]" {f=1; next} f && $1 == "dtb" {gsub(/[^0-9a-f]/, "", $3); print $3; exit}' "$HERE/device.toml"; }
sum() { sha256sum "$1" | cut -d' ' -f1; }
[ "$(sum "$SEED")" = "$(want b-r5-smmu)" ] || { echo "osfact-display-nfc-dtb: SEED_DTB is not the dtb of [images.b-r5-smmu.recipe]" >&2; exit 1; }
[ "$(sum "$NFC")" = "$(want nfc)" ] || { echo "osfact-display-nfc-dtb: NFC_DTB is not the dtb of [images.nfc.recipe]" >&2; exit 1; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
cpp -nostdinc -undef -x assembler-with-cpp -I "$K/include" "$HERE/dt/msm8998-google-walleye-nfc.dtso" -o "$T/nfc.pp.dts"
dtc -@ -I dts -O dtb -q -o "$T/nfc.dtbo" "$T/nfc.pp.dts"
fdtoverlay -i "$SEED" -o "$OUT" "$T/nfc.dtbo"
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; exit 1; fi; }
N=/soc@0/i2c@c1b6000/nfc@28
for p in compatible reg interrupts enable-gpios; do ck "nfc@28 $p equals the NFC image's" "$(fdtget -t x "$OUT" $N $p 2>&1)" "$(fdtget -t x "$NFC" $N $p 2>&1)"; done
ck "the i2c bus is okay" "$(fdtget "$OUT" /soc@0/i2c@c1b6000 status)" "$(fdtget "$NFC" /soc@0/i2c@c1b6000 status)"
ck "the irq pins as the NFC image has them" "$(fdtget "$OUT" /soc@0/pinctrl@3400000/nfc-irq-state pins)" "$(fdtget "$NFC" /soc@0/pinctrl@3400000/nfc-irq-state pins)"
ck "the control pins as the NFC image has them" "$(fdtget "$OUT" /soc@0/pinctrl@3400000/nfc-ctl-state pins)" "$(fdtget "$NFC" /soc@0/pinctrl@3400000/nfc-ctl-state pins)"
for n in /soc@0/pinctrl@3400000/nfc-irq-state /soc@0/pinctrl@3400000/nfc-ctl-state $N; do
  for p in $(fdtget -p "$NFC" "$n" | grep -v -e "^phandle$" -e "^pinctrl-0$"); do ck "$n $p equals the NFC image's" "$(fdtget -t x "$OUT" "$n" "$p" 2>&1)" "$(fdtget -t x "$NFC" "$n" "$p" 2>&1)"; done
done
ck "mdss reset kept (the display seed's)" "$(fdtget -t x "$OUT" /soc@0/display-subsystem@c900000 resets)" "$(fdtget -t x "$SEED" /soc@0/display-subsystem@c900000 resets)"
ck "ramoops record size kept (as stock Android)" "$(fdtget -t x "$OUT" /reserved-memory/ramoops@a1100000 record-size)" "$(fdtget -t x "$SEED" /reserved-memory/ramoops@a1100000 record-size)"
ck "panel kept" "$(fdtget "$OUT" /soc@0/display-subsystem@c900000/dsi@c994000/panel@0 compatible)" "$(fdtget "$SEED" /soc@0/display-subsystem@c900000/dsi@c994000/panel@0 compatible)"
ck "UFS stays off" "$(fdtget "$OUT" /soc@0/ufshc@1da4000 status)" "$(fdtget "$SEED" /soc@0/ufshc@1da4000 status)"
sha256sum "$OUT"
