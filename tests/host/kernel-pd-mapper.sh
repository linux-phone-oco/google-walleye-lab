#!/bin/sh
# walleye-kernel-pd-mapper (item of that name), run on the lab host or in CI: does every kernel config fragment the walleye images are built from set CONFIG_QCOM_PD_MAPPER=y,
# and, when a kernel tree is given, does the .config of defconfig + build/walleye.config (as build/build-kernel.sh merges them) read back =y for it and the QRTR it depends on?
# Source: tests/fixtures/kernel-text/pd-mapper-Kconfig.txt (the symbol is a tristate that `depends on NET && QRTR`; defconfig has both =m; plain Alpine has no userspace pd-mapper).
# Usage: kernel-pd-mapper.sh [KERNEL_TREE]   Environment: BUILD_DIR (the folder of the fragments, default build), CONFIG (a built .config to read back too)
# Prints `pass|fail|skip walleye-kernel-pd-mapper host DETAIL`; exit 0 only on pass.
HERE=$(cd "$(dirname "$0")/../.." && pwd)
B=${BUILD_DIR:-$HERE/build}
K=${1:-}
say() { echo "$1 walleye-kernel-pd-mapper host $2"; [ "$1" = pass ]; exit $?; }
# the base fragment every kernel build merges first (build/build-kernel.sh and the build-display*.sh scripts)
base=$B/walleye.config
[ -r "$base" ] || say fail "no $base"
for s in QRTR QRTR_SMD QCOM_PD_MAPPER; do
  [ "$(grep -c "^CONFIG_$s=" "$base")" = 1 ] || say fail "$base does not set CONFIG_$s exactly once"
  grep -qx "CONFIG_$s=y" "$base" || say fail "$base sets CONFIG_$s but not =y"
done
# no other fragment takes it back
for f in "$B"/*.config; do
  [ "$f" = "$base" ] && continue
  if grep -E "^(CONFIG_(QRTR|QRTR_SMD|QCOM_PD_MAPPER)=[mn]|# CONFIG_(QRTR|QRTR_SMD|QCOM_PD_MAPPER) is not set)" "$f" >/dev/null; then say fail "$f undoes it: $(grep -m1 -E 'QRTR|PD_MAPPER' "$f")"; fi
done
if [ -n "${CONFIG:-}" ]; then
  [ -r "$CONFIG" ] || say fail "no built config at $CONFIG"
  for s in QRTR QCOM_PD_MAPPER; do grep -qx "CONFIG_$s=y" "$CONFIG" || say fail "$CONFIG does not read CONFIG_$s=y"; done
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
for s in QRTR QRTR_SMD QCOM_PD_MAPPER; do grep -qx "CONFIG_$s=y" "$T/.config" || say fail "the merged .config reads $(grep -m1 "CONFIG_$s[ =]" "$T/.config"), not CONFIG_$s=y"; done
say pass "defconfig + walleye.config of $K read back QRTR, QRTR_SMD and QCOM_PD_MAPPER =y"
