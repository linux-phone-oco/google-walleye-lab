#!/bin/sh
# walleye-adsp-q6-builds (item walleye-adsp-q6-mi2s), run on the lab host: do the eight kernel patches of kernel/audio-adsp-q6/ apply to kernel.org v7.3, do the msm8998
# sound drivers (the machine driver, lpass-msm8998, q6afe) build without a warning, and does the walleye dts patch (0009), applied to dt/, give a dtb whose ADSP is okay with
# its firmware name, whose APR services (core, afe, asm, adm, mvm, cvs, cvp) are there, whose LPASS SMMU is okay, and whose MultiMedia and Quaternary MI2S dais are described?
# Checks the pinned checksums first. Writes a temp dir; the kernel tree gets a throwaway git worktree (removed at the end). dt/'s nodes owned by other topics (the PMIC watchdog,
# the fuel gauge) are cut from the test copy of the dts, because those topics are not in this series.
# Prints `pass|fail|skip walleye-adsp-q6-builds host DETAIL`; exit 0 only on pass.
# Usage: audio-adsp-q6-builds.sh KERNEL_GIT_TREE   Environment: BASE (default v7.3), CROSS_COMPILE (default aarch64-linux-gnu-)
HERE=$(cd "$(dirname "$0")/../.." && pwd)
K=${1:?usage: audio-adsp-q6-builds.sh KERNEL_GIT_TREE}
BASE=${BASE:-v7.3}
CC=${CROSS_COMPILE:-aarch64-linux-gnu-}
S=$HERE/kernel/audio-adsp-q6
say() { echo "$1 walleye-adsp-q6-builds host $2"; [ "$1" = pass ]; exit $?; }
command -v "${CC}gcc" >/dev/null || say skip "no ${CC}gcc: not known yet"
command -v fdtget >/dev/null || say skip "no fdtget: not known yet"
git -C "$K" rev-parse --verify -q "$BASE^{commit}" >/dev/null || say skip "$K has no $BASE: not known yet"
(cd "$S" && sha256sum -c SHA256SUMS >/dev/null 2>&1) || say fail "the patches do not match kernel/audio-adsp-q6/SHA256SUMS"
T=$(mktemp -d) || say skip "no temp dir"
W=$T/tree
cleanup() { git -C "$K" worktree remove --force "$W" >/dev/null 2>&1; rm -r "$T"; }
trap cleanup EXIT
trap 'exit 130' INT TERM
git -C "$K" worktree add -q --detach "$W" "$BASE" || say skip "cannot make a worktree of $BASE in $K"
for p in "$S"/000[1-8]-*.patch; do
  git -C "$W" -c user.name=check -c user.email=check@example.invalid am -q "$p" >"$T/am.log" 2>&1 || say fail "$(basename "$p") does not apply to $BASE: $(tr '\n' ' ' <"$T/am.log" | cut -c1-120)"
done
Q=$W/arch/arm64/boot/dts/qcom
cp "$HERE/dt/msm8998-google-walleye.dts" "$HERE/dt/msm8998-google-walleye-overlay-stubs.dtsi" "$Q/"
for p9 in "$S"/0009-*.patch; do :; done
(cd "$W" && patch -p1 -s --no-backup-if-mismatch <"$p9") >"$T/dts.log" 2>&1 || say fail "0009 does not apply to dt/msm8998-google-walleye.dts: $(tr '\n' ' ' <"$T/dts.log" | cut -c1-120)"
awk '/^&pm8998_wdt \{/{s=1} /^&pmi8998_fg \{/{s=1} s&&/^\};/{s=0;next} !s' "$Q/msm8998-google-walleye.dts" >"$T/w.dts" && cp "$T/w.dts" "$Q/msm8998-google-walleye.dts"
O=$T/out
make -s -C "$W" ARCH=arm64 CROSS_COMPILE="$CC" O="$O" defconfig >"$T/cfg.log" 2>&1 || say fail "defconfig: $(tail -n1 "$T/cfg.log")"
printf 'CONFIG_SND_SOC_MSM8998=m\nCONFIG_SND_SOC_LPASS_MSM8998=m\n' >>"$O/.config"
make -s -C "$W" ARCH=arm64 CROSS_COMPILE="$CC" O="$O" olddefconfig >"$T/cfg.log" 2>&1 || say fail "olddefconfig: $(tail -n1 "$T/cfg.log")"
for s in CONFIG_SND_SOC_MSM8998 CONFIG_SND_SOC_LPASS_MSM8998; do grep -q "^$s=m" "$O/.config" || say fail "the Kconfig symbol $s did not take"; done
make -C "$W" ARCH=arm64 CROSS_COMPILE="$CC" O="$O" W=1 sound/soc/qcom/msm8998.o sound/soc/qcom/lpass-msm8998.o sound/soc/qcom/qdsp6/q6afe.o >"$T/build.log" 2>&1 || say fail "build: $(grep -m1 -i error "$T/build.log")"
w=$(grep -cE '(msm8998|q6afe)\.c.*warning' "$T/build.log")
[ "$w" = 0 ] || say fail "the sound objects build with $w warning(s): $(grep -m1 -E '(msm8998|q6afe)\.c.*warning' "$T/build.log")"
make -s -C "$W" ARCH=arm64 CROSS_COMPILE="$CC" O="$O" qcom/msm8998-google-walleye.dtb >"$T/dtb.log" 2>&1 || say fail "dtb: $(grep -m1 -E 'ERROR|Error' "$T/dtb.log")"
B=$O/arch/arm64/boot/dts/qcom/msm8998-google-walleye.dtb
R=/soc@0/remoteproc@17300000
[ "$(fdtget "$B" $R status)" = okay ] || say fail "the ADSP ($R) is not okay"
[ "$(fdtget -t s "$B" $R firmware-name)" = qcom/msm8998/wahoo/adsp.mbn ] || say fail "the ADSP's firmware-name is not qcom/msm8998/wahoo/adsp.mbn"
[ "$(fdtget "$B" /soc@0/iommu@5100000 status)" = okay ] || say fail "the LPASS SMMU is not okay"
n=$(fdtget -l "$B" $R/glink-edge/apr | wc -l)
[ "$n" = 7 ] || say fail "the APR node has $n services, not 7 (core, afe, asm, adm, mvm, cvs, cvp)"
for s in 3 4 7 8 9 a b; do
  fdtget -p "$B" $R/glink-edge/apr/service@$s | grep -qx qcom,protection-domain && say fail "APR service@$s still names a protection domain"
done
for d in 0 1 2 3; do fdtget -l "$B" $R/glink-edge/apr/service@7/dais | grep -qx "dai@$d" || say fail "q6asm has no dai@$d (MultiMedia$((d+1)))"; done
for l in quat-mi2s-active quat-mi2s-sd1-active; do fdtget -l "$B" /soc@0/pinctrl@3400000 | grep -qx "$l" || say fail "no pin group $l"; done
fdtget -l "$B" $R/glink-edge/apr/service@4/dais | grep -qx 'dai@22' || say fail "q6afe has no dai@22 (Quaternary MI2S RX)"
[ "$(fdtget -t u "$B" $R/glink-edge/apr/service@4/dais/dai@22 qcom,sd-lines)" = 1 ] || say fail "Quaternary MI2S RX does not claim only SD1"
say pass "the eight patches apply to $BASE, msm8998.o, lpass-msm8998.o and q6afe.o build without a warning, and the walleye dtb (0009 on dt/) has the ADSP okay on its own firmware name, 7 APR services, the LPASS SMMU okay and Quaternary MI2S RX on SD1"
