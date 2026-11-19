#!/bin/sh
# The dtb parts of the f3 split (item walleye-display-f3-split-images; decision 2026-11-19-walleye-display-iommu-reading), from the f0..f3 parts of build-display-dtb-bisect-dtbs.sh:
#   build-display-f3-split-dtbs.sh PARTS_DIR OUT_DIR      (PARTS_DIR: the f-series OUT_DIR, with f0.dtb and f3.dtb)
# g1 has no dtb of its own (it is f3 with iommu.passthrough=1 on the command line: build/series/display-f3-split.toml)
# g2 = f3 with the iommus property removed from the mdss node (display-subsystem@c900000), nothing else
# g3 = f0 (z21's dtb) with the mdss node okay and nothing else enabled: z21's display controller node has no status (absent = enabled once mdss is entered), so g3 sets it disabled; the DSI host and its PHY are disabled already
# Writes OUT_DIR/{g2.dtb,g3.dtb,g2.diff,g3.diff (the decompiled f3 against each),SHA256SUMS-display-f3-split-parts}; every check stops the build (ok/FAIL, exit 1, nothing written).
set -eu
P=${1:?usage: build-display-f3-split-dtbs.sh PARTS_DIR OUT_DIR}
OUT=${2:?}
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$P/f0.dtb" "$P/f3.dtb"; do [ -s "$f" ] || { echo "build-display-f3-split-dtbs: missing $f" >&2; exit 1; }; done
sym() { fdtget "$P/f3.dtb" /__symbols__ "$1"; }
st() { fdtget "$1" "$2" status 2>/dev/null || echo absent; }
dts() { dtc -I dtb -O dts -q "$1" 2>/dev/null; }
MDSS=$(sym mdss); MDP=$(sym mdss_mdp); DSI=$(sym mdss_dsi0); PHY=$(sym mdss_dsi0_phy)
ck "f3 has the iommus property on mdss" "$(fdtget "$P/f3.dtb" "$MDSS" iommus)" "149 0"
cp "$P/f3.dtb" "$T/g2.dtb"
fdtput -d "$T/g2.dtb" "$MDSS" iommus
cp "$P/f0.dtb" "$T/g3.dtb"
fdtput -t s "$T/g3.dtb" "$MDSS" status okay
fdtput -t s "$T/g3.dtb" "$MDP" status disabled
dts "$P/f3.dtb" > "$T/f3.dts"
for g in g2 g3; do dts "$T/$g.dtb" > "$T/$g.dts"; diff -u "$T/f3.dts" "$T/$g.dts" | sed '1,2d' > "$T/$g.diff" || true; done
changed() { grep '^[-+]' "$1" | sed 's/^\([-+]\)[[:space:]]*/\1/' | tr '\n' '|'; }
ck "g2: the diff against f3 is the iommus line, nothing else" "$(changed "$T/g2.diff")" "-iommus = <0x95 0x00>;|"
ck "g2: the mdss node has no iommus property, and is okay with the rest of f3" "$(fdtget -p "$T/g2.dtb" "$MDSS" | grep -c '^iommus$' || true) $(st "$T/g2.dtb" "$MDSS") $(st "$T/g2.dtb" "$MDP") $(st "$T/g2.dtb" "$DSI") $(st "$T/g2.dtb" "$PHY")" "0 okay okay okay okay"
ck "g3: only mdss is okay (the display controller, the DSI host and its PHY disabled)" "$(st "$T/g3.dtb" "$MDSS") $(st "$T/g3.dtb" "$MDP") $(st "$T/g3.dtb" "$DSI") $(st "$T/g3.dtb" "$PHY")" "okay disabled disabled disabled"
ck "g3: the diff against f3 is the three status lines going back to disabled, nothing else" "$(grep '^[-+]' "$T/g3.diff" | grep -vc 'status = ' || true) $(grep -c '^-.*status = "okay"' "$T/g3.diff")" "0 3"
ck "g3: against z21's dtb (f0) only mdss's status changes and the display controller gets status disabled" "$(dts "$P/f0.dtb" | diff - "$T/g3.dts" | grep "^[<>]" | sed 's/^\([<>]\)[[:space:]]*/\1/' | tr '\n' '|')" "<status = \"disabled\";|>status = \"okay\";|>status = \"disabled\";|"
ck "f3, g2, g3 are three different files" "$(for f in "$P/f3.dtb" "$T/g2.dtb" "$T/g3.dtb"; do sha256sum < "$f"; done | sort -u | wc -l)" 3
ck "the stock overlay still merges into g2 and g3 (tests/host/dtbo-overlay-merges.sh)" "$(for f in g2 g3; do DTB="$T/$f.dtb" sh "$(dirname "$0")/../tests/host/dtbo-overlay-merges.sh" | cut -d' ' -f1; done | sort -u)" pass
ck "the touch power gpio keeps one claimant in g2 and g3 (tests/host/touch-gpio-single-claimant.sh)" "$(for f in g2 g3; do DTB="$T/$f.dtb" sh "$(dirname "$0")/../tests/host/touch-gpio-single-claimant.sh" | cut -d' ' -f1; done | sort -u)" pass
[ $total -eq 0 ] || { echo "build-display-f3-split-dtbs: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$T/g2.dtb" "$T/g3.dtb" "$T/g2.diff" "$T/g3.diff" "$OUT/"
(cd "$OUT" && sha256sum g2.dtb g3.dtb g2.diff g3.diff | tee SHA256SUMS-display-f3-split-parts)
