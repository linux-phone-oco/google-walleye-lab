#!/bin/sh
# walleye-dtb-overlay-stub-refs, run on the lab host: after the vendor dtbo overlay (entry 3) is merged into our dtb, does any
# enabled node that is OURS (it exists in the dtb before the merge) carry a property the kernel resolves before probe -
# pinctrl-N, clocks, power-domains, phys, resets, iommus - whose phandle lands under /vendor-overlay-stubs? A stub has no
# driver above it, so such a reference defers the node silently (research walleye-display-dsi-defer-pinctrl-stubs;
# walleye-overlay-stub-refs-other-nodes). Nodes only the overlay creates are not ours: nothing binds them. *-supply references
# to stubs on our nodes are listed in the detail as a note (a driver that asks for that supply would defer; not known yet which asks).
# Prints one line `pass|fail|skip walleye-dtb-overlay-stub-refs kernel DETAIL`; exit 0 only on pass. Writes only a temp dir.
# Environment: DTB (default the display late-probe L3 dtb), DTBO (dtbo.img), DTBO_IDX (default 3).
W=$HOME/Projects/walleye-build
DTB=${DTB:-$W/images-display-late-l3/l3.dtb}
DTBO=${DTBO:-$W/control/dtbo.img}
IDX=${DTBO_IDX:-3}
say() { echo "$1 walleye-dtb-overlay-stub-refs kernel $2"; [ "$1" = pass ]; exit $?; }
for c in fdtoverlay fdtget dtc xxd awk; do command -v $c >/dev/null || say skip "$c not installed: not known yet"; done
[ -r "$DTB" ] && [ -r "$DTBO" ] || say skip "need DTB=$DTB and DTBO=$DTBO readable: not known yet"
T=$(mktemp -d) || say skip "no temp dir"; trap 'rm -rf "$T"' EXIT
e=$(xxd -s $((32+32*IDX)) -l 8 -p "$DTBO") || say skip "cannot read the dtbo table"
size=$((0x$(echo "$e" | cut -c1-8))); off=$((0x$(echo "$e" | cut -c9-16)))
[ "$size" -gt 0 ] || say skip "dtbo entry $IDX is empty"
dd if="$DTBO" of="$T/ov" bs=1 skip="$off" count="$size" 2>/dev/null
fdtoverlay -i "$DTB" -o "$T/m.dtb" "$T/ov" 2>"$T/err" || say fail "overlay $IDX does not merge into $(basename "$DTB"): $(tr '\n' ' ' <"$T/err" | cut -c1-120)"
dtc -I dtb -O dts -q "$T/m.dtb" >"$T/m.dts" 2>/dev/null
# one pass over the merged dts: remember each stub node's phandle, list (node path, property, stub) for every cell that equals one
# of them, in a property of the kinds named; the cell is a phandle only in those properties, so a number alike elsewhere is not read.
awk '
function lab(s) { sub(/^[ \t]+/, "", s); sub(/ *\{$/, "", s); sub(/^[A-Za-z0-9_]+: /, "", s); return s }
{ lines[NR] = $0 }
END {
  for (i = 1; i <= NR; i++) {
    l = lines[i]
    if (l ~ /\{$/) { d++; nm[d] = lab(l); p = ""; for (k = 2; k <= d; k++) p = p "/" nm[k]; path[d] = p }
    else if (l ~ /^[ \t]*\};/) d--
    else if (l ~ /^[ \t]*phandle = </ && path[d] ~ /^\/vendor-overlay-stubs\//) { v = l; sub(/.*<0x/, "", v); sub(/>.*/, "", v); sp[tolower(v)] = path[d] }
  }
  d = 0
  for (i = 1; i <= NR; i++) {
    l = lines[i]
    if (l ~ /\{$/) { d++; nm[d] = lab(l); p = ""; for (k = 2; k <= d; k++) p = p "/" nm[k]; path[d] = p; st[d] = "" }
    else if (l ~ /^[ \t]*\};/) d--
    else if (path[d] != "" && path[d] !~ /^\/vendor-overlay-stubs/ && l ~ /= <.*>;/) {
      prop = l; sub(/^[ \t]+/, "", prop); sub(/ *=.*/, "", prop)
      if (prop !~ /^(pinctrl-[0-9]+|clocks|power-domains|phys|resets|iommus|.*-supply)$/) continue
      v = l; sub(/.*= </, "", v); sub(/>.*/, "", v); n = split(v, c, " ")
      for (j = 1; j <= n; j++) { x = c[j]; sub(/^0x/, "", x); x = tolower(x); if (x in sp) print path[d] "\t" prop "\t" sp[x] }
    }
  }
}' "$T/m.dts" | sort -u >"$T/refs.tsv"
bad=; note=
while IFS="$(printf '\t')" read -r node prop stub; do
  fdtget -p "$DTB" "$node" >/dev/null 2>&1 || continue            # the overlay's own node: nothing binds it
  [ "$(fdtget "$T/m.dtb" "$node" status 2>/dev/null)" = disabled ] && continue
  case $prop in
    *-supply) note="$note ${node##*/}:$prop";;
    *) bad="$bad ${node##*/}:$prop->${stub##*/}";;
  esac
done <"$T/refs.tsv"
[ -z "$bad" ] || say fail "$(basename "$DTB") merged with overlay $IDX: our enabled node(s) point at stubs:$bad${note:+ (supply notes:$note)}"
say pass "no enabled node of ours points a pre-probe property at a stub in $(basename "$DTB") merged with overlay $IDX${note:+ (supply notes:$note)}"
