#!/bin/sh
# Porthole's kernel for build-pk.sh (backlog walleye-porthole-known-good): kernel.org v7.2.2 plus the 262 numbered patches and the config of
# porthole-dev/pmaports branch taimen-bringup (device/testing/linux-postmarketos-qcom-msm8998-7.2), both GPL-2.0-only, kept as published.
#   pk-kernel.sh WORK_DIR PMAPORTS_COMMIT
# Makes WORK_DIR/{pm,linux-7.2.2,out}; Image.gz and the two dtbs (built -@) are in WORK_DIR/out/arch/arm64/boot. Host only.
# Checks the tarball and every patch and the config against the sha512sums of the APKBUILD; stops on any difference.
set -eu
W=${1:?usage: pk-kernel.sh WORK_DIR PMAPORTS_COMMIT}
COMMIT=${2:?}
mkdir -p "$W"; cd "$W"
[ -d pm ] || git clone -q --filter=blob:none -b taimen-bringup https://github.com/porthole-dev/pmaports.git pm
git -C pm checkout -q "$COMMIT"
P=$W/pm/device/testing/linux-postmarketos-qcom-msm8998-7.2
[ -s linux-7.2.2.tar.xz ] || curl -fsSL -O https://cdn.kernel.org/pub/linux/kernel/v7.x/linux-7.2.2.tar.xz
sed -n '/^sha512sums="/,/^"/p' "$P/APKBUILD" | grep -E '^[0-9a-f]{128}  ' > sums.txt
grep ' linux-7.2.2.tar.xz$' sums.txt | sha512sum -c - >/dev/null || { echo "pk-kernel: tarball sha512 differs" >&2; exit 1; }
(cd "$P" && grep -v ' linux-7.2.2.tar.xz$' "$W/sums.txt" | sha512sum -c --quiet -) || { echo "pk-kernel: a patch or the config differs from the APKBUILD's sums" >&2; exit 1; }
rm -rf linux-7.2.2; tar xf linux-7.2.2.tar.xz
cd linux-7.2.2
n=0
for p in $(sed -n '/^source="/,/^"/p' "$P/APKBUILD" | grep -E '^[[:space:]]+[0-9]{4}-.*\.patch'); do
  patch -p1 -s --no-backup-if-mismatch < "$P/$p" || { echo "pk-kernel: $p does not apply" >&2; exit 1; }
  n=$((n+1))
done
[ "$n" -eq 262 ] || { echo "pk-kernel: applied $n patches, want 262" >&2; exit 1; }
mkdir -p ../out
cp "$P/config-postmarketos-qcom-msm8998-7.2.aarch64" ../out/.config
M="make ARCH=arm64 LLVM=1 O=../out"
$M olddefconfig >/dev/null
for s in BLK_DEV_INITRD RD_XZ; do grep -q "^CONFIG_$s=y" ../out/.config || { echo "pk-kernel: CONFIG_$s is not =y" >&2; exit 1; }; done
DTC_FLAGS=-@ $M -j"$(nproc)" KBUILD_BUILD_VERSION=84-postmarketos-qcom-msm8998-7.2 Image.gz qcom/msm8998-google-taimen.dtb qcom/msm8998-google-walleye.dtb
ls -l ../out/arch/arm64/boot/Image.gz
