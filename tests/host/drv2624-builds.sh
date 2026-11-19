#!/bin/sh
# walleye-haptics-driver-builds (item walleye-drv2624-driver), run on the lab host: do the four kernel patches of kernel/drv2624/ (binding, driver, the driver's fixes)
# apply to kernel.org v7.3 and does the driver build as a module for arm64 without a warning (W=1)? And, when DTB is given, does that dtb carry the vibrator node
# (compatible ti,drv2624, reg 0x5a, a reset gpio) on the QUP6 controller? Checks the pinned checksums first. Writes a temp dir; the kernel tree gets a throwaway git worktree (removed at the end) and nothing else; it is read through
# a throwaway git worktree. Prints `pass|fail|skip walleye-haptics-driver-builds host DETAIL`; exit 0 only on pass.
# Usage: drv2624-builds.sh KERNEL_GIT_TREE   Environment: BASE (default v7.3), DTB (a built walleye dtb), CROSS_COMPILE (default aarch64-linux-gnu-)
HERE=$(cd "$(dirname "$0")/../.." && pwd)
K=${1:?usage: drv2624-builds.sh KERNEL_GIT_TREE}
BASE=${BASE:-v7.3}
CC=${CROSS_COMPILE:-aarch64-linux-gnu-}
say() { echo "$1 walleye-haptics-driver-builds host $2"; [ "$1" = pass ]; exit $?; }
command -v "${CC}gcc" >/dev/null || say skip "no ${CC}gcc: not known yet"
git -C "$K" rev-parse --verify -q "$BASE^{commit}" >/dev/null || say skip "$K has no $BASE: not known yet"
(cd "$HERE/kernel/drv2624" && sha256sum -c SHA256SUMS >/dev/null 2>&1) || say fail "the patches do not match kernel/drv2624/SHA256SUMS"
T=$(mktemp -d) || say skip "no temp dir"
W=$T/tree
cleanup() { git -C "$K" worktree remove --force "$W" >/dev/null 2>&1; rm -r "$T"; }
trap cleanup EXIT INT TERM
git -C "$K" worktree add -q --detach "$W" "$BASE" || say skip "cannot make a worktree of $BASE in $K"
for p in "$HERE"/kernel/drv2624/000[1234]-*.patch; do
  git -C "$W" -c user.name=check -c user.email=check@example.invalid am -q "$p" >"$T/am.log" 2>&1 || say fail "$(basename "$p") does not apply to $BASE: $(tr '\n' ' ' <"$T/am.log" | cut -c1-120)"
done
O=$T/out
make -s -C "$W" ARCH=arm64 CROSS_COMPILE="$CC" O="$O" defconfig >"$T/cfg.log" 2>&1 || say fail "defconfig: $(tail -n1 "$T/cfg.log")"
echo CONFIG_INPUT_DRV2624_HAPTICS=m >>"$O/.config"
make -s -C "$W" ARCH=arm64 CROSS_COMPILE="$CC" O="$O" olddefconfig >"$T/cfg.log" 2>&1 || say fail "olddefconfig: $(tail -n1 "$T/cfg.log")"
grep -q '^CONFIG_INPUT_DRV2624_HAPTICS=m' "$O/.config" || say fail "the Kconfig symbol INPUT_DRV2624_HAPTICS did not take"
make -C "$W" ARCH=arm64 CROSS_COMPILE="$CC" O="$O" W=1 drivers/input/misc/drv2624.o >"$T/build.log" 2>&1 || say fail "build: $(grep -m1 -i error "$T/build.log")"
w=$(grep -c 'drv2624.*warning' "$T/build.log")
[ "$w" = 0 ] || say fail "the driver builds with $w warning(s): $(grep -m1 'drv2624.*warning' "$T/build.log")"
[ -s "$O/drivers/input/misc/drv2624.o" ] || say fail "no drv2624.o after the build"
# the dts patch is the dt/ file's own change: applied in reverse to a copy of it, it must apply cleanly
cp "$HERE/dt/msm8998-google-walleye.dts" "$T/walleye.dts"
mkdir -p "$T/p/arch/arm64/boot/dts/qcom" && cp "$T/walleye.dts" "$T/p/arch/arm64/boot/dts/qcom/msm8998-google-walleye.dts"
patch -R -s -p1 --dry-run -d "$T/p" <"$(ls "$HERE"/kernel/drv2624/0005-*.patch)" >"$T/dts.log" 2>&1 || say fail "0005 does not match dt/msm8998-google-walleye.dts: $(tr '\n' ' ' <"$T/dts.log" | cut -c1-120)"
[ -n "${DTB:-}" ] || say skip "the four patches apply to $BASE and drv2624.o builds, the dts patch matches dt/; no DTB given, so the node in a built dtb is not checked"
command -v fdtget >/dev/null || say skip "fdtget not installed; the module builds, the node is not checked"
B=/soc@0/i2c@c17a000
[ "$(fdtget "$DTB" $B status 2>/dev/null)" = okay ] || say fail "$DTB: $B is not okay"
N=$(fdtget -l "$DTB" $B 2>/dev/null | grep '^vibrator@')
[ "$N" = vibrator@5a ] || say fail "$DTB has no vibrator@5a under $B (children: $N)"
P=$B/$N
[ "$(fdtget "$DTB" "$P" compatible)" = ti,drv2624 ] || say fail "the vibrator's compatible is not ti,drv2624"
[ "$(fdtget -t x "$DTB" "$P" reg)" = 5a ] || say fail "the vibrator's reg is not 0x5a"
[ "$(fdtget -t u "$DTB" "$P" reset-gpios | awk '{print $2, $3}')" = "95 1" ] || say fail "the vibrator's reset-gpios is not gpio 95, active low (got: $(fdtget -t u "$DTB" "$P" reset-gpios))"
say pass "the four patches apply to $BASE, drv2624.o builds as a module without a warning, the dts patch matches dt/, and $(basename "$DTB") has vibrator@5a (ti,drv2624, reset gpio95 active low) on a QUP6 that is okay"
