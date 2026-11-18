#!/bin/sh
# The quirks each side applies, as sorted lists "VENDOR MODEL FLAG" (device quirks) and "CONDITION FLAG" (host quirks), and what
# applies to THIS card (SK hynix, model string H28U74301AMR): vendor matches model by PREFIX (STR_PRFX_EQUAL), ours the same.
# usage: quirks.sh VENDOR_DIR OURS_DIR OUT_DIR
set -eu
V=$1; O=$2; OUT=$3; mkdir -p "$OUT"
CARD=H28U74301AMR
# vendor: UFS_FIX(vendor, model, flag) possibly over several lines, in ufs_quirks.c
tr '\n' ' ' < "$V/ufs_quirks.c" | grep -oE 'UFS_FIX\([^)]*\)' | sed -E 's/UFS_FIX\(([^,]*),[ ]*([^,]*),[ ]*([^)]*)\)/\1 \2 \3/; s/[ ]+/ /g' | sort > "$OUT/vendor-device-quirks.txt"
# ours: { .wmanufacturerid = V, .model = M, .quirk = A | B }, both tables (core ufs_fixups, ufs-qcom dev fixups)
for f in "$O/ufshcd.c" "$O/ufs-qcom.c"; do
  tr '\n\t' '  ' < "$f" | grep -oE '\{ \.wmanufacturerid = [A-Z_0-9]+,[ ]*\.model = [^,]*,[ ]*\.quirk = [^}]*\}' | sed -E 's/\{ \.wmanufacturerid = ([A-Z_0-9]+),[ ]*\.model = ("[^"]*"|[A-Z_]+)[^,]*,[ ]*\.quirk = ([^}]*) \}/\1 \2 \3/; s/[ ]+/ /g'
done | awk '{v=$1; m=$2; q=$0; sub(/^[^ ]+ [^ ]+ /,"",q); n=split(q,a,/ \| /); for(i=1;i<=n;i++){gsub(/^ +| +$/,"",a[i]); print v, m, a[i]}}' | sort -u > "$OUT/ours-device-quirks.txt"
# which entries match the card: manufacturer is SK hynix (vendor name HYNIX 0x1AD, ours SKHYNIX), model ANY or prefix of the card's model string
for s in vendor ours; do
  awk -v card="$CARD" '$1 ~ /HYNIX/ { m=$2; gsub(/"/,"",m); if (m=="UFS_ANY_MODEL" || index(card,m)==1) print $3 }' "$OUT/$s-device-quirks.txt" | sort -u > "$OUT/$s-quirks-this-card.txt"
done
# host quirks: the flags UFSHCD_QUIRK_* and UFSHCD_CAP_* each ufs-qcom sets, with the condition in the source line context (by name only)
grep -oE 'UFSHCD_(QUIRK|CAP)_[A-Z0-9_]+' "$V/ufs-qcom.c" | sort -u > "$OUT/vendor-host-flags.txt"
grep -oE 'UFSHCD_(QUIRK|CAP)_[A-Z0-9_]+' "$O/ufs-qcom.c" | sort -u > "$OUT/ours-host-flags.txt"
{ echo "== device quirks applying to $CARD (< vendor Android, > ours z13)"; diff "$OUT/vendor-quirks-this-card.txt" "$OUT/ours-quirks-this-card.txt" || true; } > "$OUT/diff-quirks.txt"
{ echo "== host flags named in ufs-qcom.c (< vendor, > ours; a name is not a condition: the source says when)"; diff "$OUT/vendor-host-flags.txt" "$OUT/ours-host-flags.txt" || true; } >> "$OUT/diff-quirks.txt"
