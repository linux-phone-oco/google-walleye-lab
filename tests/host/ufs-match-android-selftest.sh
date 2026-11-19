#!/bin/sh
# Proves tests/host/ufs-match-android.sh can fail where it matters: the real tree passes, and each mutant of the driver (a scratch copy of the files it reads) fails it.
# Usage: ufs-match-android-selftest.sh KERNEL_TREE
set -eu
KT=${1:?usage: ufs-match-android-selftest.sh KERNEL_TREE}
HERE=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
FILES="drivers/ufs/host/ufs-qcom.c drivers/ufs/host/ufs-qcom.h drivers/ufs/core/ufshcd.c include/ufs/ufs_quirks.h drivers/phy/qualcomm/phy-qcom-qmp-ufs.c drivers/pinctrl/qcom/pinctrl-msm8998.c drivers/phy/qualcomm/phy-qcom-qmp-qserdes-com-v3.h drivers/phy/qualcomm/phy-qcom-qmp-qserdes-txrx-v3.h drivers/phy/qualcomm/phy-qcom-qmp-pcs-ufs-v3.h"
fresh() { rm -rf "$T/t"; for f in $FILES; do mkdir -p "$T/t/$(dirname "$f")"; cp "$KT/$f" "$T/t/$f"; done; }
want() { # NAME WANTED(pass|fail)
  got=$(sh "$HERE/ufs-match-android.sh" "$T/t" >/dev/null 2>&1 && echo pass || echo fail)
  if [ "$got" = "$2" ]; then echo "ok   $1"; else echo "FAIL $1: got $got, want $2"; total=$((total+1)); fi
}
mut() { # NAME FILE SED
  fresh; sed -i "$3" "$T/t/$2"
  cmp -s "$T/t/$2" "$KT/$2" && { echo "FAIL mutant '$1' changed nothing"; total=$((total+1)); return; }
  want "mutant fails: $1" fail
}
fresh; want "the tree as it is passes" pass
mut "lane 1 written for 3.0.0" drivers/phy/qualcomm/phy-qcom-qmp-ufs.c 's/ && !cfg->lane0_tbls_only//'
mut "3.0.0 and 3.1.0 swapped" drivers/phy/qualcomm/phy-qcom-qmp-ufs.c 's/qmp->cfg = &msm8998_ufsphy_cfg_3_0_0;$/qmp->cfg = \&msm8998_ufsphy_cfg_X;/; s/qmp->cfg = &msm8998_ufsphy_cfg_3_1_0;$/qmp->cfg = \&msm8998_ufsphy_cfg_3_0_0;/; s/msm8998_ufsphy_cfg_X/msm8998_ufsphy_cfg_3_1_0/'
mut "purge guard returns false" drivers/ufs/core/ufshcd.c '/^static bool ufshcd_purge_refused/,/^}/s/return true;/return false;/'
mut "refused UPIU leaks its tag" drivers/ufs/core/ufshcd.c 's/^\t\tgoto put_dev_mgmt_cmd;\n//; /ufshcd_purge_refused(hba, req_upiu->qr.opcode/,/^\t}/s/goto put_dev_mgmt_cmd;/return -EPERM;/'
mut "a PHY table value changed" drivers/phy/qualcomm/phy-qcom-qmp-ufs.c '/msm8998_ufsphy_3_0_0_serdes\[\]/,/^};/s/QSERDES_V3_COM_PLL_IVCO, 0x0f/QSERDES_V3_COM_PLL_IVCO, 0x07/'
mut "one TX lane quirk back" drivers/ufs/host/ufs-qcom.c 's/UFS_DEVICE_QUIRK_NO_PURGE },/UFS_DEVICE_QUIRK_NO_PURGE | UFS_DEVICE_QUIRK_DELAY_BEFORE_LPM },/'
mut "version not handed over at init" drivers/ufs/host/ufs-qcom.c '/^static int ufs_qcom_init(/,/^}/s/err = ufs_qcom_phy_configure(host);/err = 0;/'
mut "ngpios back to 150" drivers/pinctrl/qcom/pinctrl-msm8998.c 's/\.ngpios = 154,/.ngpios = 150,/'
[ $total -eq 0 ] || { echo "ufs-match-android-selftest: $total failed" >&2; exit 1; }
echo "ufs-match-android-selftest: all ok"
