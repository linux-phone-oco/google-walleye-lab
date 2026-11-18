#!/bin/sh
# walleye-overlay-collisions, run on the lab host: which nodes of OUR dtb does the vendor dtbo overlay (the entry the bootloader picks, androidboot.dtbo_idx=3)
# write BY NAME? The bootloader merges every fragment into its target node by name: a node of the overlay with the name of a node of ours overwrites the properties
# it names there (z16: i2c@c179000 overwrote our qup controller's compatible; z17: the overlay's own pmi8998 nodes). A node of another name stays ours.
# The overlay is merged into DTB with fdtoverlay (the same by-name merge, fragment targets resolved through the dtb's /__symbols__ and the overlay's __fixups__);
# the merged and the plain dtb are then compared node by node, property by property:
#   overwrite  a property our dtb has now has another value     -> a collision (exit 1) unless ALLOW names it
#   add        a property our dtb did not have, on a node we have -> listed, not a collision (a pure addition)
# A property the overlay writes with the very value we have is not visible (and harmless). Prints every collision (our path, property, our value, the overlay's)
# and a count, then one line `pass|fail|skip walleye-overlay-collisions kernel DETAIL`; exit 0 only on pass. Writes only a temp dir.
# The overlay is the official dtbo.img of the build in the control directory (DTBO), not the phone's own: the phone's would need one read over adb root.
# ALLOW (default: overlay-collisions.allow beside this script): one line per intended overwrite, "PATH PROP", or "PATH *" for every property of that node;
# a line is a reason too: after the two words, free text. Blank lines and lines starting # are ignored.
# Environment: DTB (our dtb, default the newest built z image's), DTBO (dtbo.img), DTBO_IDX (default 3), ALLOW, QUIET (1: only the last line).
HERE=$(cd "$(dirname "$0")" && pwd)
W=$HOME/Projects/walleye-build
DTBO=${DTBO:-$W/control/dtbo.img}
DTB=${DTB:-$W/out/msm8998-google-walleye.dtb}
IDX=${DTBO_IDX:-3}
ALLOW=${ALLOW:-$HERE/overlay-collisions.allow}
say() { echo "$1 walleye-overlay-collisions kernel $2"; [ "$1" = pass ]; exit $?; }
for c in fdtoverlay dtc xxd awk; do command -v $c >/dev/null || say skip "$c not installed"; done
[ -r "$DTBO" ] && [ -r "$DTB" ] || say skip "need DTBO=$DTBO and DTB=$DTB readable"
T=$(mktemp -d) || say skip "no temp dir"; trap 'rm -rf "$T"' EXIT
e=$(xxd -s $((32+32*IDX)) -l 8 -p "$DTBO") || say skip "cannot read the dtbo table"
size=$((0x$(echo "$e" | cut -c1-8))); off=$((0x$(echo "$e" | cut -c9-16)))
[ "$size" -gt 0 ] || say skip "dtbo entry $IDX is empty"
tail -c +$((off+1)) "$DTBO" | head -c "$size" > "$T/ov"
out=$(fdtoverlay -i "$DTB" -o "$T/merged" "$T/ov" 2>&1) || say fail "overlay $IDX does not merge into $(basename "$DTB"): $(echo "$out" | tr '\n' ' ' | cut -c1-160)"
# flat: "N<TAB>path" per node, "P<TAB>path<TAB>prop<TAB>value" per property (a value spanning lines joined), phandle and the overlay bookkeeping left out
flat() {
  dtc -I dtb -O dts "$1" 2>/dev/null | awk '
    function pathnow(  i,p){p="";for(i=2;i<=d;i++)p=p "/" st[i];return p==""?"/":p}
    { line=$0; sub(/^[ \t]+/,"",line)
      if(acc!=""){acc=acc " " line; if(line !~ /;$/) next; line=acc; acc=""}
      else if(line=="" || line ~ /^\/dts-v1\/;|^\/plugin\/;/) next
      else if(line ~ /\{$/ && line !~ /=/){ s=line; sub(/ *\{$/,"",s); sub(/^[A-Za-z_][A-Za-z0-9_]*: /,"",s); if(s=="/")s=""; st[++d]=s; p=pathnow(); if(p !~ /^\/__(symbols|fixups|local_fixups)__/) print "N\t" p; next }
      else if(line ~ /^\};$/){d--;next}
      else if(line !~ /;$/){acc=line; next}
      p=pathnow(); if(p ~ /^\/__(symbols|fixups|local_fixups)__/) next
      s=line; sub(/;$/,"",s); name=s; sub(/ *=.*/,"",name); val=""; if(s ~ /=/){val=s; sub(/^[^=]*= */,"",val)}
      if(name=="phandle"||name=="linux,phandle") next
      print "P\t" p "\t" name "\t" val }'
}
flat "$DTB" > "$T/base.flat"; flat "$T/merged" > "$T/merged.flat"
[ -s "$T/base.flat" ] || say skip "cannot decompile $DTB"
[ -r "$ALLOW" ] && cp "$ALLOW" "$T/allow" || : > "$T/allow"
awk -F'\t' -v allow="$T/allow" '
  BEGIN { while ((getline l < allow) > 0) { if (l ~ /^[ \t]*(#|$)/) continue; split(l,a," "); ok[a[1] SUBSEP a[2]]=1 } }
  FNR==NR { if($1=="N") node[$2]=1; else base[$2 SUBSEP $3]=$4; next }
  $1=="P" && ($2 in node) {
    k=$2 SUBSEP $3
    if(!(k in base)) { adds++; add[adds]=$2 " " $3; next }
    if(base[k]!=$4) {
      if(ok[k] || ok[$2 SUBSEP "*"]) { allowed++; al[allowed]=$2 " " $3; next }
      n++; col[n]=$2 "\t" $3 "\t" base[k] "\t" $4
    }
  }
  END {
    for(i=1;i<=n;i++){ split(col[i],c,"\t"); printf "collision %s  %s: ours %s  overlay %s\n", c[1], c[2], substr(c[3],1,60), substr(c[4],1,60); nodes[c[1]]=1 }
    for(i=1;i<=allowed;i++) printf "allowed   %s\n", al[i]
    for(i=1;i<=adds;i++){split(add[i],q," "); ap[q[1]]=ap[q[1]] " " q[2]; ao[q[1]]=1} for(x in ao) printf "add       %s:%s\n", x, ap[x]
    m=0; for(x in nodes) m++
    printf "count %d collisions on %d nodes, %d allowed, %d pure additions on nodes we have\n", n, m, allowed+0, adds+0
    exit n>0
  }' "$T/base.flat" "$T/merged.flat" > "$T/report"
rc=$?
if [ "${QUIET:-0}" = 1 ]; then tail -n 1 "$T/report"; else cat "$T/report"; fi
sum=$(tail -n 1 "$T/report" | sed 's/^count //')
[ "$rc" -eq 0 ] && say pass "overlay $IDX ($(sha256sum "$DTBO" | cut -c1-8)) writes no node of $(basename "$DTB") by name with another value: $sum"
say fail "overlay $IDX overwrites properties of nodes of $(basename "$DTB"): $sum"
