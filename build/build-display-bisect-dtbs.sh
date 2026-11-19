#!/bin/sh
# The parts of the display bisect (item walleye-display-bisect-images): the display kernel's Image.gz and the dtbs of d1 and d2, cut from the display image itself, so they are the dtb that image boots (clock_gcc on the real gcc) and not the raw built one.
#   build-display-bisect-dtbs.sh BOOT_DISPLAY_IMG OUT_DIR      (BOOT_DISPLAY_IMG: build-display-image.sh's boot-display.img)
# d1 = the display dtb with mdss and its display controller okay, the DSI host and its PHY disabled, no panel node (and the DSI output's remote-endpoint, which named it, dropped).
# d2 = d1 with the DSI host and its PHY okay again; the panel node stays absent. d3 = the display image as it is. Every other node is the display dtb's.
# Writes OUT_DIR/{Image.gz,d1.dtb,d2.dtb,SHA256SUMS-display-bisect-parts}; every check stops the build (ok/FAIL, exit 1, nothing written).
set -eu
IMG=${1:?usage: build-display-bisect-dtbs.sh BOOT_DISPLAY_IMG OUT_DIR}
OUT=${2:?}
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
[ -s "$IMG" ] || { echo "build-display-bisect-dtbs: missing $IMG" >&2; exit 1; }
lpoco-bringup image unpack "$IMG" --out "$T/u" >/dev/null
DTB=$T/u/appended.dtb
KS=$(stat -c %s "$T/u/kernel"); DS=$(stat -c %s "$DTB")
head -c $((KS - DS)) "$T/u/kernel" > "$T/Image.gz"
ck "the kernel file is Image.gz then the dtb, byte for byte" "$(cat "$T/Image.gz" "$DTB" | cmp -s - "$T/u/kernel" && echo same || echo differs)" same
ck "Image.gz is a gzip file" "$(head -c 2 "$T/Image.gz" | od -An -tx1 | tr -d ' \n')" 1f8b
node() { fdtget "$DTB" /__symbols__ "$1"; }
MDSS=$(node mdss); MDP=$(node mdss_mdp); DSI=$(node mdss_dsi0); PHY=$(node mdss_dsi0_phy); OUTEP=$(node mdss_dsi0_out)
st() { fdtget "$1" "$2" status 2>/dev/null || echo absent; }
for n in "$MDSS" "$MDP" "$DSI" "$PHY"; do ck "the display dtb has $n okay" "$(st "$DTB" "$n")" okay; done
ck "the display dtb has the panel node" "$(fdtget -l "$DTB" "$DSI" | grep -c '^panel@0$')" 1
cp "$DTB" "$T/d1.dtb"
fdtput -t s "$T/d1.dtb" "$DSI" status disabled
fdtput -t s "$T/d1.dtb" "$PHY" status disabled
fdtput -r "$T/d1.dtb" "$DSI/panel@0"
fdtput -d "$T/d1.dtb" "$OUTEP" remote-endpoint
cp "$T/d1.dtb" "$T/d2.dtb"
fdtput -t s "$T/d2.dtb" "$DSI" status okay
fdtput -t s "$T/d2.dtb" "$PHY" status okay
for d in d1 d2; do
  ck "$d: mdss and the display controller are okay" "$(st "$T/$d.dtb" "$MDSS") $(st "$T/$d.dtb" "$MDP")" "okay okay"
  ck "$d: the panel node is absent" "$(fdtget -l "$T/$d.dtb" "$DSI" | grep -c '^panel@0$' || true)" 0
done
ck "d1: the DSI host and its PHY are disabled" "$(st "$T/d1.dtb" "$DSI") $(st "$T/d1.dtb" "$PHY")" "disabled disabled"
ck "d2: the DSI host and its PHY are okay" "$(st "$T/d2.dtb" "$DSI") $(st "$T/d2.dtb" "$PHY")" "okay okay"
# every other node as in the display dtb: the decompiled sources differ by those edits only
dts() { dtc -I dtb -O dts -q "$1" 2>/dev/null | sed 's/phandle = <0x[0-9a-f]*>;/phandle;/'; }
dts "$DTB" > "$T/f.dts"; dts "$T/d1.dtb" > "$T/d1.dts"; dts "$T/d2.dtb" > "$T/d2.dts"
added=$(diff "$T/f.dts" "$T/d1.dts" | grep "^>" | grep -vc "status = " || true)
ck "d1 adds no line but status lines" "$added" 0
ck "d1 loses exactly the panel, its port and its endpoint (3 nodes)" "$(( $(grep -c "{" "$T/f.dts") - $(grep -c "{" "$T/d1.dts") ))" 3
ck "d2 differs from d1 in the two status lines only" "$(diff "$T/d1.dts" "$T/d2.dts" | grep '^[<>]' | grep -vc 'status = ' || true)" 0
ck "the stock overlay still merges into d1 and d2 (tests/host/dtbo-overlay-merges.sh)" "$(for d in d1 d2; do DTB="$T/$d.dtb" sh "$(dirname "$0")/../tests/host/dtbo-overlay-merges.sh" | cut -d' ' -f1; done | sort -u)" pass
[ $total -eq 0 ] || { echo "build-display-bisect-dtbs: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$T/Image.gz" "$T/d1.dtb" "$T/d2.dtb" "$OUT/"
(cd "$OUT" && sha256sum Image.gz d1.dtb d2.dtb | tee SHA256SUMS-display-bisect-parts)
