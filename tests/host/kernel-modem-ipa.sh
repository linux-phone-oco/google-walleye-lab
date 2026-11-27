#!/bin/sh
# walleye-kernel-modem-ipa (item walleye-modem-net-port-ipa-node), run on the lab host or in CI: does the modem image's config (build/walleye.config + build/walleye-modem.config) set CONFIG_QCOM_IPA=y once, and does no other fragment take it back?
# With a kernel tree: does defconfig + both fragments read back QCOM_IPA=y (its dependencies kept)? Source: tests/fixtures/kernel-text/ipa-Kconfig.txt (QCOM_IPA defaults =m in defconfig; depends on NET, QCOM_SMEM, INTERCONNECT, QCOM_RPROC_COMMON, QCOM_AOSS_QMP || =n).
# Usage: kernel-modem-ipa.sh [KERNEL_TREE]   Environment: BUILD_DIR (default build). Prints `pass|fail|skip walleye-kernel-modem-ipa host DETAIL`; exit 0 only on pass.
HERE=$(cd "$(dirname "$0")/../.." && pwd)
B=${BUILD_DIR:-$HERE/build}
K=${1:-}
say() { echo "$1 walleye-kernel-modem-ipa host $2"; [ "$1" = pass ]; exit $?; }
mod=$B/walleye-modem.config
[ -r "$mod" ] || say fail "no $mod"
[ "$(grep -c "^CONFIG_QCOM_IPA=" "$mod")" = 1 ] || say fail "$mod does not set CONFIG_QCOM_IPA exactly once"
grep -qx "CONFIG_QCOM_IPA=y" "$mod" || say fail "$mod sets CONFIG_QCOM_IPA but not =y"
grep -qx "CONFIG_QCOM_Q6V5_MSS=y" "$mod" || say fail "$mod does not set CONFIG_QCOM_Q6V5_MSS=y, whose type QCOM_IPA must match"
for f in "$B"/*.config; do
  if grep -E "^(CONFIG_QCOM_IPA=[mn]|# CONFIG_QCOM_IPA is not set)" "$f" >/dev/null; then say fail "$f undoes it: $(grep -m1 QCOM_IPA "$f")"; fi
done
[ -n "$K" ] || say skip "the fragment sets it; no kernel tree given, so the merged .config is not read back: not known yet"
command -v "${CROSS_COMPILE:-aarch64-linux-gnu-}gcc" >/dev/null || command -v gcc >/dev/null || say skip "no compiler to run the kernel's kconfig: not known yet"
[ -x "$K/scripts/kconfig/merge_config.sh" ] || say skip "$K is not a kernel tree: not known yet"
T=$(mktemp -d) || say skip "no temp dir"
trap 'rm -r "$T"' EXIT INT TERM
M="make -s -C $K ARCH=arm64 O=$T"
$M defconfig >"$T/log" 2>&1 || say fail "defconfig: $(tail -n1 "$T/log")"
"$K/scripts/kconfig/merge_config.sh" -m -O "$T" "$T/.config" "$B/walleye.config" "$mod" >/dev/null 2>&1 || say fail "merge_config of the fragments failed"
$M olddefconfig >"$T/log" 2>&1 || say fail "olddefconfig: $(tail -n1 "$T/log")"
for s in QCOM_IPA QCOM_PAS QCOM_MDT_LOADER QCOM_QMI_HELPERS; do grep -qx "CONFIG_$s=y" "$T/.config" || say fail "the merged .config reads $(grep -m1 "CONFIG_${s}[ =]" "$T/.config"), not CONFIG_$s=y"; done
say pass "defconfig + walleye.config + walleye-modem.config of $K read back QCOM_IPA, QCOM_PAS, QCOM_MDT_LOADER and QCOM_QMI_HELPERS =y"
