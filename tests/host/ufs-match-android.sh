#!/bin/sh
# walleye-ufs-match-android-z, run on the lab host: does the kernel tree (and the dtb) carry what Android's UFS path carries for the Pixel 2's card?
# Proof of backlog walleye-ufs-match-android-z (decisions 2026-11-19-walleye-ufs-version-read-in-our-driver, 2026-11-19-walleye-ufs-image-builds-before-the-go; research
# walleye-first-fastboot-boot/2026-11-18-ufs-android-way and walleye-ufs-regs-reader-verdict). It reads source and a dtb: it runs nothing on a phone.
# Usage: ufs-match-android.sh KERNEL_TREE [DTB]
#   KERNEL_TREE  the tree the image is built from (topic/ufs-match-android): the quirk table, the PHY tables, the host's version read, the pinctrl gpio count
#   DTB          optional: the UFS-matched dtb (msm8998-google-walleye-ufs-match.dtb): the card reset gpio, the reserved sdc2 pads, l26 always on
# Every check prints `ok` or `FAIL`; the exit status is 1 when any failed (a check that only prints is not a check).
set -u
KT=${1:?usage: ufs-match-android.sh KERNEL_TREE [DTB]}
DTB=${2:-}
HERE=$(cd "$(dirname "$0")/../.." && pwd)
VEND=$HERE/research/walleye-first-fastboot-boot/2026-11-18-ufs-android-way/results/phy
Q=$KT/drivers/ufs/host/ufs-qcom.c
P=$KT/drivers/phy/qualcomm/phy-qcom-qmp-ufs.c
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
for f in "$Q" "$P" "$KT/drivers/pinctrl/qcom/pinctrl-msm8998.c" "$VEND/vendor-phy_cal_table_rate_A_3_0_0.txt" "$VEND/vendor-phy_cal_table_rate_A_3_1_0.txt"; do
  [ -s "$f" ] || { echo "FAIL missing $f" >&2; exit 1; }
done
W=$(mktemp -d) || exit 1; trap 'rm -rf "$W"' EXIT

# 1. the quirks of the card: SAVECONFIGTIME and the NO_PURGE guard, nothing else
TBL=$(sed -n '/^static struct ufs_dev_quirk ufs_qcom_dev_fixups\[\]/,/^};/p' "$Q")
HY=$(printf '%s\n' "$TBL" | sed -n '/UFS_VENDOR_SKHYNIX/,/},/p')
ck "one SK hynix entry in the fixups table" "$(printf '%s\n' "$TBL" | grep -c UFS_VENDOR_SKHYNIX)" 1
ck "the SK hynix entry names the model H28U74301AMR" "$(printf '%s\n' "$HY" | grep -c '\.model = "H28U74301AMR"')" 1
ck "the SK hynix entry is not any-model" "$(printf '%s\n' "$HY" | grep -c UFS_ANY_MODEL)" 0
ck "the card's quirks are HOST_PA_SAVECONFIGTIME and NO_PURGE" "$(printf '%s\n' "$HY" | grep -o 'UFS_DEVICE_QUIRK_[A-Z_0-9]*' | sort | tr '\n' ' ')" "UFS_DEVICE_QUIRK_HOST_PA_SAVECONFIGTIME UFS_DEVICE_QUIRK_NO_PURGE "
for q in DELAY_BEFORE_LPM EXTEND_SYNC_LENGTH ONE_TX_LANE; do
  ck "no $q anywhere in the SK hynix entry" "$(printf '%s\n' "$HY" | grep -c "$q")" 0
done
ck "the tree has no ONE_TX_LANE or EXTEND_SYNC_LENGTH code at all (vendor runs two lanes)" "$(grep -c -e ONE_TX_LANE -e EXTEND_SYNC_LENGTH "$Q" "$KT/drivers/ufs/host/ufs-qcom.h" "$KT/include/ufs/ufs_quirks.h" | awk -F: '{s+=$2} END {print s}')" 0
ck "the purge helper refuses set, clear and toggle" "$(sed -n '/^static bool ufshcd_purge_refused/,/^}/p' "$KT/drivers/ufs/core/ufshcd.c" | grep -c -e UPIU_QUERY_OPCODE_SET_FLAG -e UPIU_QUERY_OPCODE_CLEAR_FLAG -e UPIU_QUERY_OPCODE_TOGGLE_FLAG)" 3
ck "the purge guard is on the flag path and where the UPIU is sent" "$(grep -c 'ufshcd_purge_refused(hba, ' "$KT/drivers/ufs/core/ufshcd.c")" 2

# 2. the host reads UFS_HW_VERSION in its own init, prints it and hands it to the PHY before power on
ck "ufs-qcom prints UFS_HW_VERSION where it reads it" "$(grep -c 'UFS_HW_VERSION %u.%u.%u' "$Q")" 1
ck "ufs-qcom passes the version to the phy (phy_configure) after the mode and before power on" "$(sed -n '/^static int ufs_qcom_power_up_sequence/,/^}/p' "$Q" | grep -n -e 'phy_set_mode_ext(' -e 'phy_configure(' -e 'phy_power_on(' | cut -d'(' -f1 | sed 's/^[0-9]*://' | tr -d '\t ' | tr '\n' ' ')" "ret=phy_set_mode_ext ret=phy_configure ret=phy_power_on "
ck "ufs-qcom does not stop on a phy without configure (-EOPNOTSUPP)" "$(sed -n '/^static int ufs_qcom_power_up_sequence/,/^}/p' "$Q" | grep -c 'ret != -EOPNOTSUPP')" 1
CONF=$(sed -n '/^static int qmp_ufs_configure/,/^}/p' "$P")
ck "the phy accepts 3.0.0 and 3.1.0 and refuses any other version with -ENODEV" "$(printf '%s\n' "$CONF" | tr -d '\n\t ' | grep -c 'hw_ver_major==3&&ver->hw_ver_minor==0&&ver->hw_ver_step==0.*hw_ver_major==3&&ver->hw_ver_minor==1&&ver->hw_ver_step==0.*return-ENODEV')" 1
ck "the msm8998 phy compatible starts on the 3.1.0 config" "$(grep -A1 'qcom,msm8998-qmp-ufs-phy' "$P" | grep -c 'msm8998_ufsphy_cfg_3_1_0')" 1
ck "the 3.0.0 config writes lane 0 only (as the vendor table does), the 3.1.0 config both" "$(sed -n '/^static const struct qmp_phy_cfg msm8998_ufsphy_cfg_3_0_0/,/^};/p' "$P" | grep -c 'lane0_tbls_only	= true')/$(sed -n '/^static const struct qmp_phy_cfg msm8998_ufsphy_cfg_3_1_0/,/^};/p' "$P" | grep -c lane0_tbls_only)" 1/0

# 3. the PHY tables are the vendor's value lists: every (offset, value) of each, lane bases of qmp_ufs_offsets, both lanes unless lane 0 only
{
  echo 'struct e { unsigned off, val; };'
  echo '#define QMP_PHY_INIT_CFG(r,v) { r, v }'
  grep -hE '^#define (QSERDES_V3_COM|QSERDES_V3_TX|QSERDES_V3_RX|QPHY_V3_PCS_UFS)' "$KT"/drivers/phy/qualcomm/phy-qcom-qmp-qserdes-com-v3.h "$KT"/drivers/phy/qualcomm/phy-qcom-qmp-qserdes-txrx-v3.h "$KT"/drivers/phy/qualcomm/phy-qcom-qmp-pcs-ufs-v3.h
  awk '/^static const struct qmp_phy_init_tbl msm8998_ufsphy_/ {p=1; sub(/static const struct qmp_phy_init_tbl/,"static struct e")} p {print} /^};/ {p=0}' "$P"
  echo '#include <stdio.h>'
  echo '#define A(t,base) for (unsigned i=0;i<sizeof(t)/sizeof(t[0]);i++) printf("0x%03x 0x%02x\n", base+t[i].off, t[i].val);'
  echo 'int main(int c, char **v) { int lane1 = c > 2;'
  echo 'if (v[1][1]==0x30) { A(msm8998_ufsphy_3_0_0_serdes,0) A(msm8998_ufsphy_3_0_0_tx,0x400) A(msm8998_ufsphy_3_0_0_rx,0x600) A(msm8998_ufsphy_3_0_0_pcs,0xc00) if (lane1) { A(msm8998_ufsphy_3_0_0_tx,0x800) A(msm8998_ufsphy_3_0_0_rx,0xa00) } }'
  echo 'else { A(msm8998_ufsphy_3_1_0_serdes,0) A(msm8998_ufsphy_3_1_0_tx,0x400) A(msm8998_ufsphy_3_1_0_rx,0x600) A(msm8998_ufsphy_3_1_0_pcs,0xc00) if (lane1) { A(msm8998_ufsphy_3_1_0_tx,0x800) A(msm8998_ufsphy_3_1_0_rx,0xa00) } }'
  echo 'return 0; }'
} > "$W/t.c"
if cc -w -o "$W/t" "$W/t.c" 2> "$W/cc.err"; then
  # the vendor lists carry the power-down control (0xc04), which mainline sets in power_on
  for v in 3_0_0:30 3_1_0:31; do
    n=${v%:*}
    if [ "$n" = 3_0_0 ]; then got=$("$W/t" 30 | sort); else got=$("$W/t" 31 both | sort); fi
    want=$(grep -v '^0xc04 ' "$VEND/vendor-phy_cal_table_rate_A_$n.txt" | sort)
    ck "the msm8998 $n tables equal the vendor's list, 0xc04 apart ($(printf '%s\n' "$want" | wc -l) values)" "$(printf '%s\n' "$got" | tr '\n' ';')" "$(printf '%s\n' "$want" | tr '\n' ';')"
  done
else
  ck "the msm8998 phy tables compile against the v3 register headers" "no: $(head -n1 "$W/cc.err")" yes
fi

# 4. the card's RST_n pad (pin 153) can be named as a gpio
ck "pinctrl-msm8998 counts 154 gpios (the UFS_RESET pad is pin 153)" "$(sed -n 's/^[[:space:]]*\.ngpios = \([0-9]*\),/\1/p' "$KT/drivers/pinctrl/qcom/pinctrl-msm8998.c")" 154

# 5. the dtb
if [ -n "$DTB" ]; then
  [ -s "$DTB" ] || { echo "FAIL missing $DTB" >&2; exit 1; }
  ck "dtb: ufshc is okay and the phy is okay" "$(fdtget "$DTB" /soc@0/ufshc@1da4000 status) $(fdtget "$DTB" /soc@0/phy@1da7000 status)" "okay okay"
  TL=$(fdtget "$DTB" /soc@0/pinctrl@3400000 phandle)
  ck "dtb: the card reset is <tlmm 153 active-low> (reset-gpios)" "$(fdtget -t u "$DTB" /soc@0/ufshc@1da4000 reset-gpios | tr ' ' '\n' | tr '\n' ' ')" "$TL 153 1 "
  ck "dtb: the tlmm gpio-ranges cover pin 153" "$(fdtget -t u "$DTB" /soc@0/pinctrl@3400000 gpio-ranges)" "$TL 0 0 154"
  ck "dtb: the secure gpios and the three sdc2 pads are reserved" "$(fdtget -t u "$DTB" /soc@0/pinctrl@3400000 gpio-reserved-ranges)" "0 4 81 4 150 3"
  L26=$(fdtget "$DTB" /__symbols__ vreg_l26a_1p2 2>/dev/null || echo none)
  ck "dtb: the host's l26 rail is always on (the vendor's vddp-ref-clk)" "$([ "$L26" != none ] && fdtget "$DTB" "$L26" regulator-always-on >/dev/null 2>&1 && echo yes || echo no)" yes
  ck "dtb: the host keeps the loads of Android's rails (vcc 750000, vccq 560000, vccq2 750000 uA)" "$(for p in vcc vccq vccq2; do fdtget -t u "$DTB" /soc@0/ufshc@1da4000 "$p-max-microamp"; done | tr "\n" " ")" "750000 560000 750000 "
fi
[ "$total" -eq 0 ] || { echo "ufs-match-android: $total checks failed" >&2; exit 1; }
echo "ufs-match-android: all checks ok"
