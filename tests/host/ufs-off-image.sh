#!/bin/sh
# walleye-ufs-off-gate, run on the lab host: is UFS off in a boot image of ours? (decision 2026-11-19-walleye-ufs-off-by-a-gate; research walleye-ufs-on-by-accident)
# Why: UFS was enabled in every image since z11 and probed against the author's word; the bootloader and the card survived by luck. No image of ours boots with UFS on again.
# Usage: ufs-off-image.sh IMAGE|DTB. An image is unpacked (unpack_bootimg) and EVERY dtb appended to its kernel is taken - the bootloader may pick any of them; a dtb file is taken as it is.
# Each dtb is judged as it is and merged with the dtbo entry the bootloader picks (the phone's own overlay could enable a node): a node whose compatible names the UFS host
# controller or the UFS phy (ufshc, ufs-phy, qmp-ufs) fails when its status is okay or absent (absent means enabled), unless a parent is disabled (the kernel does not enter it). A dtb with no
# UFS node passes. For an image the kernel command line also fails when a word names ufs, except the bootloader's own androidboot.* words. An image with no dtb fails: off cannot be shown.
# An overlay that does not merge FAILS (UFS off cannot be shown), as does a dtbo whose table is not a dtbo (magic d7b7ab1e). A node is a UFS node by its compatible (ufshc, ufs-phy, qmp-ufs) or its unit address (@1da4000, @1da7000; the empty vendor stubs named ufs1 and the like are not UFS nodes). The gate is passed only on the line `pass`; a `skip` is not a pass.
# Prints `pass|fail|skip walleye-ufs-off DETAIL`; exit 0 only on pass. Writes only a temp dir.
# Environment: DTBO (the dtbo.img, default the control's), DTBO_IDX (default 3).
W=$HOME/Projects/walleye-build
DTBO=${DTBO:-$W/control/dtbo.img}
IDX=${DTBO_IDX:-3}
IN=${1:-}
say() { echo "$1 walleye-ufs-off $2"; [ "$1" = pass ]; exit $?; }
for c in fdtget fdtoverlay dtc xxd awk unpack_bootimg; do command -v "$c" >/dev/null || say skip "$c not installed: not known yet"; done
[ -n "$IN" ] || say fail "usage: ufs-off-image.sh IMAGE|DTB"
[ -r "$IN" ] || say skip "no file at $IN: not known yet"
[ -r "$DTBO" ] || say skip "no dtbo at $DTBO: not known yet"
[ "$(od -An -tx1 -N4 "$DTBO" | tr -d " \n")" = d7b7ab1e ] || say fail "$DTBO is not a dtbo table (no d7b7ab1e magic)"
T=$(mktemp -d) || say skip "no temp dir"; trap 'rm -rf "$T"' EXIT
e=$(xxd -s $((32+32*IDX)) -l 8 -p "$DTBO") || say skip "cannot read the dtbo table"
size=$((0x$(echo "$e" | cut -c1-8))); off=$((0x$(echo "$e" | cut -c9-16)))
[ "$size" -gt 0 ] || say skip "dtbo entry $IDX is empty"
tail -c +$((off+1)) "$DTBO" | head -c "$size" > "$T/ov"
magic=$(head -c 8 "$IN" | tr -d '\000')
n=0
if [ "$magic" = "ANDROID!" ]; then
  KIND=image
  INFO=$(unpack_bootimg --boot_img "$IN" --out "$T/u" 2>&1) || say fail "unpack_bootimg cannot read $IN"
  K=$T/u/kernel
  [ -s "$K" ] || say fail "no kernel in $IN"
  cmd=$(printf '%s\n' "$INFO" | sed -n 's/^\(additional \)\{0,1\}command line args: *//p' | tr '\n' ' ')
  set -f; for w in $cmd; do
    case $w in androidboot.*) ;; *) case $(printf %s "$w" | tr A-Z a-z) in *ufs*) say fail "the command line names ufs: $w" ;; esac ;; esac
  done
  # every dtb appended to the kernel: a d00dfeed whose total size fits the file and that decompiles; one inside a taken dtb is skipped
  ks=$(stat -c %s "$K"); skipto=0
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$K" | cut -d: -f1); do
    [ "$x" -ge "$skipto" ] || continue
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$K" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ "$ts" -ge 40 ] && [ $((x+ts)) -le "$ks" ] || continue
    tail -c +$((x+1)) "$K" | head -c "$ts" > "$T/d$n.dtb"
    dtc -I dtb -O dts "$T/d$n.dtb" >/dev/null 2>&1 || continue
    n=$((n+1)); skipto=$((x+ts))
  done
  [ "$n" -gt 0 ] || say fail "no dtb appended to the kernel of $IN: UFS off cannot be shown"
else
  KIND=dtb
  [ "$(od -An -tx1 -N4 "$IN" | tr -d ' ')" = d00dfeed ] || say fail "$IN is neither a boot image nor a dtb"
  cp "$IN" "$T/d0.dtb"; n=1
fi
# judge DTB LABEL: print the nodes that name ufs and are on
judge() {
  dtc -I dtb -O dts "$1" 2>/dev/null > "$1.dts" || { echo "$2: cannot decompile"; return; }
  awk '
    /^[ \t]*[^ \t=;{}][^=;]*\{[ \t]*$/ { depth++; nm = $0; sub(/^[ \t]*/, "", nm); sub(/[ \t]*\{[ \t]*$/, "", nm); sub(/^[^ ]*: /, "", nm); name[depth] = nm; if (nm ~ /@1da[47]000$/) { path = ""; for (i = 2; i <= depth; i++) path = path "/" name[i]; print path }; next }
    /^[ \t]*\};/ { depth--; next }
    /^[ \t]*compatible = / && ($0 ~ /ufshc/ || $0 ~ /ufs-phy/ || $0 ~ /qmp-ufs/) { path = ""; for (i = 2; i <= depth; i++) path = path "/" name[i]; print path }
  ' "$1.dts" | sort -u | while IFS= read -r p; do
    [ -n "$p" ] || continue
    cur=; blocked=0; st=
    for part in $(echo "$p" | tr '/' ' '); do
      cur="$cur/$part"
      st=$(fdtget "$1" "$cur" status 2>/dev/null) || st=
      case "$st" in ''|okay|ok) ;; *) blocked=1 ;; esac
    done
    [ "$blocked" = 0 ] && echo "$2: $p status '${st:-absent (enabled by default)}'"
  done
}
bad=; notes=; i=0
while [ "$i" -lt "$n" ]; do
  d=$T/d$i.dtb
  r=$(judge "$d" "dtb $i")
  [ -z "$r" ] || bad="$bad $r;"
  if fdtoverlay -i "$d" -o "$T/m$i.dtb" "$T/ov" >/dev/null 2>&1; then
    r=$(judge "$T/m$i.dtb" "dtb $i merged with dtbo entry $IDX")
    [ -z "$r" ] || bad="$bad $r;"
  else
    bad="$bad dtb $i takes no merge of dtbo entry $IDX (the bootloader merges it: UFS off cannot be shown);"
  fi
  i=$((i+1))
done
[ -z "$bad" ] || say fail "UFS is on:$(printf %s "$bad" | tr "\n" " ")"
say pass "$KIND: $n dtb(s), no UFS node is on, alone or merged with dtbo entry $IDX;${cmd:+ command line names no ufs;}$notes"
