#!/bin/sh
# Normalise the ufshc and ufsphy node properties of (a) the live Android fdt dump (dt.txt of the keeper's read) and (b) our compiled dtb
# into sorted "node prop = value" lists, and diff them.  Phandle-valued properties are listed by name only (the numbers differ by tree).
# usage: dt-props.sh LIVE_DT_TXT OUR_DTB OUT_DIR       needs: dtc, awk
set -eu
L=$1; D=$2; OUT=$3; mkdir -p "$OUT"
PH='^(clocks|phys|resets|pinctrl-[0-9]+|ufs-qcom-crypto|power-domains|phandle|vcc-supply|vccq-supply|vccq2-supply|vdd-hba-supply|vdda-phy-supply|vdda-pll-supply|vddp-ref-clk-supply|vdd-hba-fixed-regulator|interrupt-parent)$'
# live: "-- ufshc@1da4000/prop:  hex bytes  | ascii"
awk -v ph="$PH" '
/^-- (ufshc|ufsphy)@/ {
  line=$0; sub(/^-- /,"",line); split(line,a,":"); path=a[1]; n=split(path,p,"/"); node=p[1]; prop=p[2]
  sub(/@.*/,"",node)
  hex=line; sub(/^[^:]*:[ ]*/,"",hex); sub(/[ ]*\|.*$/,"",hex); nb=split(hex,b," ")
  if (prop ~ ph) { print node, prop, "= (phandle or reference)"; next }
  if (prop=="name") next
  str=(nb>0 && b[nb]=="00"); nz=0; for(i=1;i<=nb;i++){ v=strtonum("0x" b[i]); if (v!=0) nz=1; if (!(v==0 || (v>=32 && v<127))) str=0 } if (!nz) str=0
  if (nb==0) { print node, prop, "= (empty)"; next }
  if (str) { s=""; cur=""; for(i=1;i<=nb;i++){ v=strtonum("0x" b[i]); if(v==0){ s=s (s==""?"":",") "\"" cur "\""; cur="" } else cur=cur sprintf("%c",v) } print node, prop, "=", s }
  else { s=""; for(i=1;i+3<=nb;i+=4){ s=s sprintf("%s0x%s%s%s%s", (s==""?"":" "), b[i],b[i+1],b[i+2],b[i+3]) } gsub(/0x0+/,"0x",s); gsub(/0x /,"0x0 ",s); sub(/0x$/,"0x0",s); print node, prop, "= <" s ">" }
}' "$L" | sort > "$OUT/live-ufs-props.txt"
# ours: dtc -> the two nodes
dtc -q -I dtb -O dts "$D" 2>/dev/null | awk -v ph="$PH" '
/^\t\t([a-z0-9_]+: )?(ufshc|phy)@1da[47]000 \{/ { n=($0 ~ /ufshc@/) ? "ufshc" : "ufsphy"; node=n; inn=1; next }
inn && /^\t\t\};/ { inn=0 }
inn && /^\t\t\t[a-z#]/ {
  l=$0; sub(/^\t+/,"",l); sub(/;$/,"",l); p=l; sub(/ =.*/,"",p); v=l; sub(/^[^=]*= ?/,"",v); if (l !~ /=/) v="(empty)"
  if (p ~ ph) { print node, p, "= (phandle or reference)"; next }
  gsub(/", "/,"\",\"",v)
  print node, p, "=", v
}' | sort > "$OUT/ours-ufs-props.txt"
for f in live ours; do sed -E -e "s/0x0+([0-9a-f])/0x\1/g" -e "s/\"ok\"/\"okay\"/" "$OUT/$f-ufs-props.txt" | sort > "$OUT/$f-ufs-props.tmp" && mv "$OUT/$f-ufs-props.tmp" "$OUT/$f-ufs-props.txt"; done
diff "$OUT/live-ufs-props.txt" "$OUT/ours-ufs-props.txt" > "$OUT/diff-ufs-props.txt" || true
echo "live props: $(wc -l < "$OUT/live-ufs-props.txt"), ours: $(wc -l < "$OUT/ours-ufs-props.txt"); diff in $OUT/diff-ufs-props.txt (< live Android, > ours)"
