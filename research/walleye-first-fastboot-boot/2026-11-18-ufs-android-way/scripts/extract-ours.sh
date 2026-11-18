#!/bin/sh
# Read our tree's side at a commit into a scratch dir.  usage: extract-ours.sh KERNEL_GIT_DIR COMMIT OUT_DIR   (z13 = d49d54405 on v7.3)
set -eu
G=$1; T=$2; OUT=$3; mkdir -p "$OUT"
for f in arch/arm64/boot/dts/qcom/msm8998.dtsi drivers/ufs/host/ufs-qcom.c drivers/ufs/host/ufs-qcom.h drivers/ufs/core/ufshcd.c \
         include/ufs/ufs_quirks.h include/ufs/ufshcd.h drivers/phy/qualcomm/phy-qcom-qmp-ufs.c drivers/phy/qualcomm/phy-qcom-qmp-qserdes-com-v3.h \
         drivers/phy/qualcomm/phy-qcom-qmp-pcs-ufs-v3.h drivers/phy/qualcomm/phy-qcom-qmp-qserdes-txrx-v3.h drivers/pinctrl/qcom/pinctrl-msm8998.c; do
  git -C "$G" show "$T:$f" > "$OUT/$(basename "$f")"
done
