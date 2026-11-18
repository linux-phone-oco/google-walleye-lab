#!/bin/sh
# Extract the QMP UFS PHY init tables of both trees as sorted "absolute-offset value" lists, then diff them.
# usage: phy-tables.sh VENDOR_DIR OURS_DIR OUT_DIR     (VENDOR_DIR from fetch-vendor.sh; OURS_DIR from extract-ours.sh)
# Reading only: the vendor files are compiled in a scratch dir to resolve macro offsets; nothing of them is kept.
set -eu
V=$1; O=$2; OUT=$3; mkdir -p "$OUT"; W=$(mktemp -d); trap 'rm -rf "$W"' EXIT
CC=${CC:-cc}

# ---- vendor: defines of phy-qcom-ufs-qmp-v3.h + the named tables -> C -> numbers
{
  echo 'struct e { unsigned off, val; const char *n; };'
  echo '#define UFS_QCOM_PHY_CAL_ENTRY(r,v) { r, v, #r }'
  grep -E '^#define' "$V/phy-qcom-ufs-qmp-v3.h" | grep -v 'UFS_QCOM_PHY_QMP_V3_H_'
  # tables: from "static struct ufs_qcom_phy_calibration NAME[] = {" to "};"
  awk '/^static struct ufs_qcom_phy_calibration/ {p=1; sub(/static struct ufs_qcom_phy_calibration/,"static struct e")} p {print} /^};/ {p=0}' "$V/phy-qcom-ufs-qmp-v3.h"
  echo '#include <stdio.h>'
  echo '#define P(t) for (unsigned i=0;i<sizeof(t)/sizeof(t[0]);i++) printf("%s 0x%03x 0x%02x %s\n", #t, t[i].off, t[i].val, t[i].n);'
  echo 'int main(void){'
  grep -oE 'phy_cal_table_[A-Za-z0-9_]+\[\]' "$V/phy-qcom-ufs-qmp-v3.h" | tr -d '[]' | sed 's/.*/P(&)/'
  echo 'return 0;}'
} > "$W/v.c"
$CC -w -o "$W/v" "$W/v.c"
"$W/v" > "$OUT/vendor-phy-all.txt"
for t in $(awk '{print $1}' "$OUT/vendor-phy-all.txt" | sort -u); do
  awk -v t="$t" '$1==t {print $2, $3}' "$OUT/vendor-phy-all.txt" | sort > "$OUT/vendor-$t.txt"
done

# ---- ours: mainline's sdm845 tables (what msm8998 uses) with the block bases of qmp_ufs_offsets
{
  echo 'struct e { unsigned off, val; const char *n; };'
  echo '#define QMP_PHY_INIT_CFG(r,v) { r, v, #r }'
  grep -hE '^#define (QSERDES_V3_COM|QSERDES_V3_TX|QSERDES_V3_RX|QPHY_V3_PCS_UFS)' "$O"/phy-qcom-qmp-qserdes-com-v3.h "$O"/phy-qcom-qmp-qserdes-txrx-v3.h "$O"/phy-qcom-qmp-pcs-ufs-v3.h
  awk '/^static const struct qmp_phy_init_tbl sdm845_ufsphy_/ {p=1; sub(/static const struct qmp_phy_init_tbl/,"static struct e")} p {print} /^};/ {p=0}' "$O/phy-qcom-qmp-ufs.c"
  echo '#include <stdio.h>'
  echo '#define P(t,base) for (unsigned i=0;i<sizeof(t)/sizeof(t[0]);i++) printf("%s 0x%03x 0x%02x %s\n", #t, base+t[i].off, t[i].val, t[i].n);'
  echo 'int main(void){'
  echo 'P(sdm845_ufsphy_serdes,0)  P(sdm845_ufsphy_hs_b_serdes,0) P(sdm845_ufsphy_pcs,0xc00)'
  echo 'P(sdm845_ufsphy_tx,0x400) P(sdm845_ufsphy_tx,0x800) P(sdm845_ufsphy_rx,0x600) P(sdm845_ufsphy_rx,0xa00)'
  echo 'return 0;}'
} > "$W/o.c"
$CC -w -o "$W/o" "$W/o.c"
"$W/o" > "$OUT/ours-phy-all.txt"
for t in serdes hs_b_serdes; do awk -v t="sdm845_ufsphy_$t" '$1==t {print $2,$3}' "$OUT/ours-phy-all.txt" | sort > "$OUT/ours-$t.txt"; done
awk '$1 ~ /_(tx|rx|pcs)$/ {print $2,$3}' "$OUT/ours-phy-all.txt" | sort > "$OUT/ours-lanes-pcs.txt"
awk '$1 == "sdm845_ufsphy_serdes" {print $2,$3}' "$OUT/ours-phy-all.txt" > /dev/null
{ awk '$1 != "sdm845_ufsphy_hs_b_serdes" {print $2,$3}' "$OUT/ours-phy-all.txt"; } | sort > "$OUT/ours-rateA.txt"

# ---- the differences, by offset, with register names: "OFFSET vendor ours NAME" (- = not written by that side)
for t in phy_cal_table_rate_A_3_0_0 phy_cal_table_rate_A_3_1_0; do
  awk -v t="$t" '$1==t {print $2, $3, $4}' "$OUT/vendor-phy-all.txt" | sort -u > "$W/vn"
  awk '$1 != "sdm845_ufsphy_hs_b_serdes" {print $2, $3, $4}' "$OUT/ours-phy-all.txt" | sort -u > "$W/on"
  awk 'NR==FNR {v[$1]=$2; n[$1]=$3; next} {o[$1]=$2; n[$1]=$3; seen[$1]=1}
       END {for (k in v) seen[k]=1; for (k in seen) { a=(k in v)?v[k]:"-"; b=(k in o)?o[k]:"-"; if (a!=b) print k, a, b, n[k] } }' "$W/vn" "$W/on" | sort > "$OUT/diff-$t.txt"
done
echo "vendor-only / differing / ours-only entries: see $OUT/diff-*.txt (columns: offset vendor ours name)"
