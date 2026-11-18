#!/bin/sh
# lpoco-lab job body for c5 (backlog walleye-build-c5): c2's kernel config with ONE line changed, CONFIG_CHARGER_QCOM_SMB2=m (rradc stays =y),
# built in the pristine v7.3 tree c2 was built in; Image.gz plus the one module, qcom_smbx.ko.
#   job-c5.sh   then build-c5.sh WORK_DIR C4_BOOT_IMG out-c5 OUT_DIR
set -eu
W=$HOME/Projects/walleye-build
K=$W/linux-v7.3
O=$W/out-c5
mkdir -p "$O/b"
[ -s "$W/out-c2/.config" ] || { echo "job-c5: c2's .config is missing" >&2; exit 1; }
sed 's/^CONFIG_CHARGER_QCOM_SMB2=y$/CONFIG_CHARGER_QCOM_SMB2=m/' "$W/out-c2/.config" > "$O/b/.config"
M="make -C $K ARCH=arm64 LLVM=1 O=$O/b"
$M olddefconfig >/dev/null
diff "$W/out-c2/.config" "$O/b/.config" || true
grep -q '^CONFIG_CHARGER_QCOM_SMB2=m$' "$O/b/.config" || { echo "job-c5: the charger is not =m" >&2; exit 1; }
grep -q '^CONFIG_QCOM_SPMI_RRADC=y$' "$O/b/.config" || { echo "job-c5: rradc is not =y" >&2; exit 1; }
$M -j"$(nproc)" Image.gz
$M -j"$(nproc)" drivers/power/supply/qcom_smbx.ko
cp "$O/b/arch/arm64/boot/Image.gz" "$O/b/.config" "$O/b/drivers/power/supply/qcom_smbx.ko" "$O/"
ls -l "$O/Image.gz" "$O/qcom_smbx.ko"
