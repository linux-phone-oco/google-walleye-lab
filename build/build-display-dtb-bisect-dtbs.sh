#!/bin/sh
# The parts of the display dtb bisect (item walleye-display-dtb-bisect-images): the late-probe image's kernel (Image.gz) and four dtbs, each z21's dtb plus ONE group of the display dtb's additions.
#   build-display-dtb-bisect-dtbs.sh Z21_BOOT_IMG LATE_BOOT_IMG OUT_DIR      (LATE_BOOT_IMG: build-display-late-image.sh's boot-display-late.img)
# f0 = z21's dtb, untouched (the control)
# f1 = + the regulators: vreg_l14a_1p8 and vreg_l22a_3p0 under the rpm regulators-0, and its vdd_l18_l22-supply line (to the bob regulator)
# f2 = + the three pinctrl states under the tlmm: mdss_dsi_active_state, mdss_dsi_suspend_state, mdss_te_state
# f3 = + mdss, the display controller, the DSI host and its PHY set okay (no panel node, no regulators, no pinctrl states)
# The values are those of dt/msm8998-google-walleye-display.dts. Writes OUT_DIR/{Image.gz,f0.dtb..f3.dtb,f1.diff..f3.diff (the decompiled dtb of z21 against each),SHA256SUMS-display-dtb-bisect-parts};
# every check stops the build (ok/FAIL, exit 1, nothing written). Set TMPDIR on a disk with room.
set -eu
Z21=${1:?usage: build-display-dtb-bisect-dtbs.sh Z21_BOOT_IMG LATE_BOOT_IMG OUT_DIR}
LATE=${2:?}
OUT=${3:?}
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$Z21" "$LATE"; do [ -s "$f" ] || { echo "build-display-dtb-bisect-dtbs: missing $f" >&2; exit 1; }; done
lpoco-bringup image unpack "$Z21" --out "$T/z" >/dev/null
lpoco-bringup image unpack "$LATE" --out "$T/l" >/dev/null
KS=$(stat -c %s "$T/l/kernel"); DS=$(stat -c %s "$T/l/appended.dtb")
head -c $((KS - DS)) "$T/l/kernel" > "$T/Image.gz"
ck "the late image's kernel file is Image.gz then its dtb, byte for byte" "$(cat "$T/Image.gz" "$T/l/appended.dtb" | cmp -s - "$T/l/kernel" && echo same || echo differs)" same
ck "Image.gz is a gzip file" "$(head -c 2 "$T/Image.gz" | od -An -tx1 | tr -d ' \n')" 1f8b
Z=$T/z/appended.dtb
sym() { fdtget "$Z" /__symbols__ "$1"; }
st() { fdtget "$1" "$2" status 2>/dev/null || echo absent; }
RPM=$(sym rpm_requests); TLMM=$(sym tlmm); BOB=$(sym vreg_bob)
MDSS=$(sym mdss); MDP=$(sym mdss_mdp); DSI=$(sym mdss_dsi0); PHY=$(sym mdss_dsi0_phy)
REG=$RPM/regulators-0
ck "z21's dtb has none of the display additions (no l14, no l22, no mdss pinctrl state)" "$(fdtget -l "$Z" "$REG" | grep -c -e '^l14$' -e '^l22$' || true) $(fdtget -l "$Z" "$TLMM" | grep -c '^mdss' || true)" "0 0"
ck "z21's dtb has mdss, the DSI host and its PHY disabled" "$(st "$Z" "$MDSS") $(st "$Z" "$DSI") $(st "$Z" "$PHY")" "disabled disabled disabled"
cp "$Z" "$T/f0.dtb"
# f1
cp "$Z" "$T/f1.dtb"
BOBPH=$(fdtget -t x "$Z" "$BOB" phandle)
fdtput -t x "$T/f1.dtb" "$REG" vdd_l18_l22-supply "$BOBPH"
for r in l14:1800000:vreg_l14a_1p8 l22:3000000:vreg_l22a_3p0; do
  n=${r%%:*}; v=${r#*:}; v=${v%%:*}
  fdtput -c "$T/f1.dtb" "$REG/$n"
  fdtput -t i "$T/f1.dtb" "$REG/$n" regulator-min-microvolt "$v"
  fdtput -t i "$T/f1.dtb" "$REG/$n" regulator-max-microvolt "$v"
  fdtput "$T/f1.dtb" "$REG/$n" regulator-allow-set-load
done
# f2
cp "$Z" "$T/f2.dtb"
pin() { # NAME PIN FUNCTION DRIVE BIAS
  fdtput -c "$T/f2.dtb" "$TLMM/$1"
  fdtput -t s "$T/f2.dtb" "$TLMM/$1" pins "$2"
  fdtput -t s "$T/f2.dtb" "$TLMM/$1" function "$3"
  fdtput -t i "$T/f2.dtb" "$TLMM/$1" drive-strength "$4"
  fdtput "$T/f2.dtb" "$TLMM/$1" "$5"
}
pin mdss-dsi-active-state gpio94 gpio 8 bias-disable
pin mdss-dsi-suspend-state gpio94 gpio 2 bias-pull-down
pin mdss-te-state gpio10 mdp_vsync_a 2 bias-pull-down
# f3
cp "$Z" "$T/f3.dtb"
for n in "$MDSS" "$MDP" "$DSI" "$PHY"; do fdtput -t s "$T/f3.dtb" "$n" status okay; done
# what each is
dts() { dtc -I dtb -O dts -q "$1" 2>/dev/null; }
dts "$Z" > "$T/z.dts"
for f in f0 f1 f2 f3; do dts "$T/$f.dtb" > "$T/$f.dts"; done
for f in f1 f2 f3; do
  diff -u "$T/z.dts" "$T/$f.dts" | sed '1,2d' > "$T/$f.diff" || true
  ck "$f: differs from z21's dtb" "$([ -s "$T/$f.diff" ] && echo yes || echo no)" yes
  ck "$f: the diff takes no line away except status lines" "$(grep '^-' "$T/$f.diff" | grep -vc 'status = "disabled"' || true)" 0
done
ck "f1: the diff adds the two regulators and the supply line, nothing else (nodes added)" "$(grep -c '^+.*{$' "$T/f1.diff")" 2
ck "f1: the supply line is the bob regulator's phandle" "$(grep '^+.*vdd_l18_l22-supply' "$T/f1.diff" | sed 's/^+[[:space:]]*//')" "vdd_l18_l22-supply = <0x$BOBPH>;"
ck "f2: the diff adds the three pinctrl states (nodes added)" "$(grep -c '^+.*{$' "$T/f2.diff")" 3
added() { grep '^+' "$1" | sed 's/^+[[:space:]]*//' | grep -v '^$' | sort | tr '\n' '|'; }
ck "f1: the added lines are exactly the two regulators (1.8 V, 3.0 V, allow-set-load) and the supply line" "$(added "$T/f1.diff")" "};|};|l14 {|l22 {|regulator-allow-set-load;|regulator-allow-set-load;|regulator-max-microvolt = <0x1b7740>;|regulator-max-microvolt = <0x2dc6c0>;|regulator-min-microvolt = <0x1b7740>;|regulator-min-microvolt = <0x2dc6c0>;|vdd_l18_l22-supply = <0x$BOBPH>;|"
ck "f2: the added lines are exactly the three pinctrl states (pins, function, drive, bias as in the display dts)" "$(added "$T/f2.diff")" "};|};|};|bias-disable;|bias-pull-down;|bias-pull-down;|drive-strength = <0x02>;|drive-strength = <0x02>;|drive-strength = <0x08>;|function = \"gpio\";|function = \"gpio\";|function = \"mdp_vsync_a\";|mdss-dsi-active-state {|mdss-dsi-suspend-state {|mdss-te-state {|pins = \"gpio10\";|pins = \"gpio94\";|pins = \"gpio94\";|"
ck "f3: the diff holds status lines only" "$(grep '^[-+]' "$T/f3.diff" | grep -vc 'status = ' || true)" 0
ck "f3: mdss, the display controller, the DSI host and its PHY are okay" "$(st "$T/f3.dtb" "$MDSS") $(st "$T/f3.dtb" "$MDP") $(st "$T/f3.dtb" "$DSI") $(st "$T/f3.dtb" "$PHY")" "okay okay okay okay"
ck "f3: the diff adds no node" "$(grep -c '^[-+].*{$' "$T/f3.diff" || true)" 0
ck "f3: no panel node" "$(fdtget -l "$T/f3.dtb" "$DSI" | grep -c panel || true)" 0
ck "f1 and f2 leave mdss, the DSI host and its PHY disabled" "$(for f in f1 f2; do st "$T/$f.dtb" "$MDSS"; st "$T/$f.dtb" "$DSI"; st "$T/$f.dtb" "$PHY"; done | sort -u)" disabled
ck "f0, f1, f2, f3 are four different files" "$(for f in f0 f1 f2 f3; do sha256sum < "$T/$f.dtb"; done | sort -u | wc -l)" 4
ck "the stock overlay still merges into f0..f3 (tests/host/dtbo-overlay-merges.sh)" "$(for f in f0 f1 f2 f3; do DTB="$T/$f.dtb" sh "$(dirname "$0")/../tests/host/dtbo-overlay-merges.sh" | cut -d' ' -f1; done | sort -u)" pass
ck "the touch power gpio keeps one claimant in f0..f3 (tests/host/touch-gpio-single-claimant.sh)" "$(for f in f0 f1 f2 f3; do DTB="$T/$f.dtb" sh "$(dirname "$0")/../tests/host/touch-gpio-single-claimant.sh" | cut -d' ' -f1; done | sort -u)" pass
[ $total -eq 0 ] || { echo "build-display-dtb-bisect-dtbs: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$T/Image.gz" "$T/f0.dtb" "$T/f1.dtb" "$T/f2.dtb" "$T/f3.dtb" "$T/f1.diff" "$T/f2.diff" "$T/f3.diff" "$OUT/"
(cd "$OUT" && sha256sum Image.gz f0.dtb f1.dtb f2.dtb f3.dtb f1.diff f2.diff f3.diff | tee SHA256SUMS-display-dtb-bisect-parts)
