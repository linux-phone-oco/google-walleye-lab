#!/bin/sh
# walleye-kernel-slpi-pas (item walleye-slpi-image-kernel-pas), run on the lab host or in CI: does every kernel config fragment the walleye images are built from set the options remoteproc_slpi needs =y,
# and, when a kernel tree is given, does the .config of defconfig + build/walleye.config (as build/build-kernel.sh merges them) read back =y for them and for what they pull in?
# Source: tests/fixtures/kernel-text/slpi-pas-Kconfig.txt (Linux v7.3's Kconfig: QCOM_Q6V5_PAS depends on `QCOM_SYSMON || QCOM_SYSMON=n` and `RPMSG_QCOM_GLINK_SMEM || ...=n`, so each is capped at the PAS driver's own
# value and must be =y with it; QCOM_MDT_LOADER has no prompt and is selected by the PAS driver). The initramfs carries no modules: research walleye-slpi-image-kernel-pas.
# Usage: kernel-slpi-pas.sh [KERNEL_TREE]   Environment: BUILD_DIR (the folder of the fragments, default build), CONFIG (a built .config to read back too)
# Prints `pass|fail|skip walleye-kernel-slpi-pas host DETAIL`; exit 0 only on pass.
HERE=$(cd "$(dirname "$0")/../.." && pwd)
B=${BUILD_DIR:-$HERE/build}
K=${1:-}
SET="QCOM_Q6V5_PAS QCOM_SYSMON RPMSG_QCOM_GLINK_SMEM"
PULLED="QCOM_MDT_LOADER"  # QCOM_PDR_HELPERS is selected only by QCOM_PMIC_GLINK and QCOM_APR and used by nothing under drivers/remoteproc (git grep v7.3, 2026-11-25): the SLPI start does not need it, so it stays as defconfig has it
say() { echo "$1 walleye-kernel-slpi-pas host $2"; [ "$1" = pass ]; exit $?; }
base=$B/walleye.config
[ -r "$base" ] || say fail "no $base"
for s in $SET; do
  [ "$(grep -c "^CONFIG_$s=" "$base")" = 1 ] || say fail "$base does not set CONFIG_$s exactly once"
  grep -qx "CONFIG_$s=y" "$base" || say fail "$base sets CONFIG_$s but not =y"
done
alt=$(echo "$SET" | tr ' ' '|')
for f in "$B"/*.config; do
  [ "$f" = "$base" ] && continue
  if grep -E "^(CONFIG_($alt)=[mn]|# CONFIG_($alt) is not set)" "$f" >/dev/null; then say fail "$f undoes it: $(grep -m1 -E "$alt" "$f")"; fi
done
if [ -n "${CONFIG:-}" ]; then
  [ -r "$CONFIG" ] || say fail "no built config at $CONFIG"
  for s in $SET $PULLED; do grep -qx "CONFIG_$s=y" "$CONFIG" || say fail "$CONFIG does not read CONFIG_$s=y"; done
fi
[ -n "$K" ] || say skip "the fragments set it; no kernel tree given, so the merged .config is not read back: not known yet"
command -v "${CROSS_COMPILE:-aarch64-linux-gnu-}gcc" >/dev/null || command -v gcc >/dev/null || say skip "no compiler to run the kernel's kconfig: not known yet"
[ -x "$K/scripts/kconfig/merge_config.sh" ] || say skip "$K is not a kernel tree: not known yet"
T=$(mktemp -d) || say skip "no temp dir"
trap 'rm -r "$T"' EXIT INT TERM
M="make -s -C $K ARCH=arm64 O=$T"
$M defconfig >"$T/log" 2>&1 || say fail "defconfig: $(tail -n1 "$T/log")"
"$K/scripts/kconfig/merge_config.sh" -m -O "$T" "$T/.config" "$base" >/dev/null 2>&1 || say fail "merge_config of $base failed"
$M olddefconfig >"$T/log" 2>&1 || say fail "olddefconfig: $(tail -n1 "$T/log")"
for s in $SET $PULLED; do grep -qx "CONFIG_$s=y" "$T/.config" || say fail "the merged .config reads $(grep -m1 "CONFIG_${s}[ =]" "$T/.config"), not CONFIG_$s=y"; done
say pass "defconfig + walleye.config of $K read back $SET $PULLED =y"
