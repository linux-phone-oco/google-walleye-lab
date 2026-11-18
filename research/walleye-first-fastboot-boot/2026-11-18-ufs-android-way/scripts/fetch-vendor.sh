#!/bin/sh
# Read Google's published GPL wahoo kernel (msm-4.4) UFS files into a scratch dir (never into a repository).
# usage: fetch-vendor.sh OUT_DIR [BRANCH]       default branch android-msm-wahoo-4.4-android11 (Google's last stock wahoo 4.4)
set -eu
OUT=$1; B=${2:-android-msm-wahoo-4.4-android11}; mkdir -p "$OUT"
U=https://android.googlesource.com/kernel/msm/+/refs/heads/$B
curl -fs "https://android.googlesource.com/kernel/msm/+log/refs/heads/$B?format=JSON&n=1" | grep -m1 '"commit"' | tee "$OUT/COMMIT"
for f in drivers/scsi/ufs/ufshcd.c drivers/scsi/ufs/ufshcd.h drivers/scsi/ufs/ufs-qcom.c drivers/scsi/ufs/ufs-qcom.h \
         drivers/scsi/ufs/ufs_quirks.h drivers/scsi/ufs/ufs_quirks.c drivers/scsi/ufs/ufshcd-pltfrm.c drivers/scsi/ufs/unipro.h drivers/scsi/ufs/ufs.h \
         drivers/phy/phy-qcom-ufs.c drivers/phy/phy-qcom-ufs-i.h drivers/phy/phy-qcom-ufs-qmp-v3.c drivers/phy/phy-qcom-ufs-qmp-v3.h; do
  curl -fs "$U/$f?format=TEXT" | base64 -d > "$OUT/$(basename "$f")"
done
# the device tree: base soc file, v2 overlay, board rail file, pins (32-bit tree path in this kernel)
for f in msm8998.dtsi msm8998-v2.dtsi msm8998-v2.1.dtsi msm8998-pinctrl.dtsi msm8998-mtp.dtsi; do
  curl -fs "$U/arch/arm/boot/dts/qcom/$f?format=TEXT" | base64 -d > "$OUT/dts-qcom_$f"
done
