#!/bin/sh
# walleye-touch-gpio-shared-proxy, run on the lab host: after the overlay merge, does exactly ONE node that the kernel would scan claim the touch power gpio (pmi8998 gpio 2)?
# Why: the kernel (drivers/gpio/gpiolib-shared.c, gpio_shared_of_traverse) walks the whole tree from the root and makes a GPIO "shared" - served by a gpiolib_shared.proxy device
# the consumers wait for - when two nodes refer to it by a property named gpios, gpio, *-gpios or *-gpio. It skips a node that is not available (status "disabled"/"fail") and does not
# enter its children. Research walleye-z20-tried-touch-gpio-leaf: our regulator-touch-power and the vendor overlay's synaptics_dsx@20 (synaptics,power-gpio, under the stub i2c_5) both name it.
# Passes only when, in DTB merged with the dtbo entry the bootloader picks, exactly one node reachable through available nodes only names that gpio; prints the claimants otherwise.
# Prints `pass|fail|skip walleye-touch-gpio-single-claimant kernel DETAIL`; exit 0 only on pass. Writes only a temp dir.
# Environment: DTB (our dtb, default the built z21 dtb), DTBO (dtbo.img), DTBO_IDX (default 3), LINE (the pmi8998 gpio, default 2).
W=$HOME/Projects/walleye-build
DTBO=${DTBO:-$W/control/dtbo.img}
DTB=${DTB:-$W/images-z21/msm8998-google-walleye-z21.dtb}
IDX=${DTBO_IDX:-3}
LINE=${LINE:-2}
say() { echo "$1 walleye-touch-gpio-single-claimant kernel $2"; [ "$1" = pass ]; exit $?; }
for c in fdtoverlay fdtget dtc xxd awk; do command -v "$c" >/dev/null || say skip "$c not installed"; done
[ -r "$DTBO" ] && [ -r "$DTB" ] || say skip "need DTBO=$DTBO and DTB=$DTB readable"
T=$(mktemp -d) || say skip "no temp dir"; trap 'rm -rf "$T"' EXIT
e=$(xxd -s $((32+32*IDX)) -l 8 -p "$DTBO") || say skip "cannot read the dtbo table"
size=$((0x$(echo "$e" | cut -c1-8))); off=$((0x$(echo "$e" | cut -c9-16)))
[ "$size" -gt 0 ] || say skip "dtbo entry $IDX is empty"
tail -c +$((off+1)) "$DTBO" | head -c "$size" > "$T/ov"
out=$(fdtoverlay -i "$DTB" -o "$T/m" "$T/ov" 2>&1) || say fail "overlay does not merge: $(echo "$out" | tr '\n' ' ' | cut -c1-120)"
R=$(fdtget "$T/m" /__symbols__ touch_power_regulator 2>/dev/null) || say fail "no touch_power_regulator label in the merged dtb"
set -- $(fdtget -t u "$T/m" "$R" gpios 2>/dev/null)
[ $# -eq 3 ] && [ "$2" = "$LINE" ] || say fail "$R gpios is '$*', want <phandle $LINE flags>"
P=$(printf '0x%x' "$1")
dtc -I dtb -O dts "$T/m" 2>/dev/null > "$T/m.dts" || say fail "cannot decompile the merged dtb"
# Walk the decompiled tree: a property named gpio(s) or *-gpio(s) whose first two cells are <P LINE> is a claimant, named by its node path.
awk -v p="$P" -v line="$(printf "0x%02x" "$LINE")" '
  /^[ \t]*[^ \t=;{}][^=;]*\{[ \t]*$/ { depth++; nm = $0; sub(/^[ \t]*/, "", nm); sub(/[ \t]*\{[ \t]*$/, "", nm); sub(/^[^ ]*: /, "", nm); name[depth] = nm; next }
  /^[ \t]*\};/ { depth--; next }
  /^[ \t]*[a-zA-Z0-9,._+-]*gpios? = </ {
    k = $1
    if (k == "gpio" || k == "gpios" || k ~ /-gpios?$/) {
      v = $0; sub(/^[^<]*</, "", v); sub(/>.*/, "", v); split(v, c, " ")
      if (c[1] == p && c[2] == line) { path = ""; for (i = 2; i <= depth; i++) path = path "/" name[i]; print path " (" k ")" }
    }
  }
' "$T/m.dts" > "$T/all" || say fail "awk failed"
# Then keep only the claimants no node on the path blocks (status other than okay). Not handled: a gpio-hog node (the kernel skips it; its gpios is no phandle, so it never matches) and a second <phandle line> pair in one property.
cnt=0; list=
while IFS= read -r w; do
  [ -n "$w" ] || continue
  path=${w% (*}; ok=1; cur=
  for part in $(echo "$path" | tr '/' ' '); do
    cur="$cur/$part"
    st=$(fdtget "$T/m" "$cur" status 2>/dev/null) || st=okay
    case "$st" in okay|ok) ;; *) ok=0 ;; esac
  done
  [ "$ok" = 1 ] && { cnt=$((cnt+1)); list="$list $w"; }
done < "$T/all"
[ "$cnt" -eq 1 ] || say fail "$cnt available nodes name pmi8998 gpio $LINE (a shared gpio, served by a proxy that never registers):$list"
say pass "one available node names pmi8998 gpio $LINE:$list"
