#!/bin/sh
# walleye-kernel-modem-mss (item walleye-modem-mss-image), run on the lab host or in CI: does the modem image's config (build/walleye.config + build/walleye-modem.config) set the modem subsystem driver and the rmtfs memory driver =y,
# once each, with the symbols it needs =y (QRTR, QRTR_SMD, QCOM_PD_MAPPER, QCOM_SYSMON, RPMSG_QCOM_GLINK_SMEM), and does no other fragment take them back? With a kernel tree: does defconfig + both fragments read back =y?
# Source: tests/fixtures/kernel-text/modem-mss-Kconfig.txt (v7.3's Kconfig blocks; both symbols default =m, QCOM_Q6V5_MSS depends on `QCOM_SYSMON || =n` and `RPMSG_QCOM_GLINK_SMEM || =n`).
# Usage: kernel-modem-mss.sh [KERNEL_TREE]   Environment: BUILD_DIR (default build), CONFIG (a built .config to read back too)
# Prints `pass|fail|skip walleye-kernel-modem-mss host DETAIL`; exit 0 only on pass.
HERE=$(cd "$(dirname "$0")/../.." && pwd)
B=${BUILD_DIR:-$HERE/build}
K=${1:-}
say() { echo "$1 walleye-kernel-modem-mss host $2"; [ "$1" = pass ]; exit $?; }
base=$B/walleye.config; mod=$B/walleye-modem.config
[ -r "$base" ] || say fail "no $base"
[ -r "$mod" ] || say fail "no $mod"
for s in QCOM_Q6V5_MSS QCOM_RMTFS_MEM; do
  [ "$(grep -c "^CONFIG_$s=" "$mod")" = 1 ] || say fail "$mod does not set CONFIG_$s exactly once"
  grep -qx "CONFIG_$s=y" "$mod" || say fail "$mod sets CONFIG_$s but not =y"
done
for s in QRTR QRTR_SMD QCOM_PD_MAPPER QCOM_SYSMON RPMSG_QCOM_GLINK_SMEM; do grep -qx "CONFIG_$s=y" "$base" || say fail "$base does not set CONFIG_$s=y, which the modem needs"; done
for f in "$B"/*.config; do
  if grep -E "^(CONFIG_(QCOM_Q6V5_MSS|QCOM_RMTFS_MEM|QRTR|QRTR_SMD|QCOM_PD_MAPPER|QCOM_SYSMON|RPMSG_QCOM_GLINK_SMEM)=[mn]|# CONFIG_(QCOM_Q6V5_MSS|QCOM_RMTFS_MEM|QRTR|QRTR_SMD|QCOM_PD_MAPPER|QCOM_SYSMON|RPMSG_QCOM_GLINK_SMEM) is not set)" "$f" >/dev/null; then say fail "$f undoes it: $(grep -m1 -E 'Q6V5_MSS|RMTFS|QRTR|PD_MAPPER|SYSMON|GLINK_SMEM' "$f")"; fi
done
if [ -n "${CONFIG:-}" ]; then
  [ -r "$CONFIG" ] || say fail "no built config at $CONFIG"
  for s in QCOM_Q6V5_MSS QCOM_RMTFS_MEM; do grep -qx "CONFIG_$s=y" "$CONFIG" || say fail "$CONFIG does not read CONFIG_$s=y"; done
fi
[ -n "$K" ] || say skip "the fragments set it; no kernel tree given, so the merged .config is not read back: not known yet"
command -v "${CROSS_COMPILE:-aarch64-linux-gnu-}gcc" >/dev/null || command -v gcc >/dev/null || say skip "no compiler to run the kernel's kconfig: not known yet"
[ -x "$K/scripts/kconfig/merge_config.sh" ] || say skip "$K is not a kernel tree: not known yet"
T=$(mktemp -d) || say skip "no temp dir"
trap 'rm -r "$T"' EXIT INT TERM
M="make -s -C $K ARCH=arm64 O=$T"
$M defconfig >"$T/log" 2>&1 || say fail "defconfig: $(tail -n1 "$T/log")"
"$K/scripts/kconfig/merge_config.sh" -m -O "$T" "$T/.config" "$base" "$mod" >/dev/null 2>&1 || say fail "merge_config of the fragments failed"
$M olddefconfig >"$T/log" 2>&1 || say fail "olddefconfig: $(tail -n1 "$T/log")"
for s in QCOM_Q6V5_MSS QCOM_RMTFS_MEM QRTR QCOM_PD_MAPPER; do grep -qx "CONFIG_$s=y" "$T/.config" || say fail "the merged .config reads $(grep -m1 "CONFIG_${s}[ =]" "$T/.config"), not CONFIG_$s=y"; done
say pass "defconfig + walleye.config + walleye-modem.config of $K read back QCOM_Q6V5_MSS, QCOM_RMTFS_MEM, QRTR and QCOM_PD_MAPPER =y"
