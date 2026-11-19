#!/bin/sh
# walleye-dtb-display-splash, run on the lab host: does the dtb hand the bootloader's splash over properly (item walleye-display-splash-region-dts)?
# Passes only when the dtb reserves 0x9d400000 + 0x2400000 no-map, overlapping no other reserved region, and /chosen holds a simple-framebuffer
# of that address, 1080x1920 a8r8g8b8 stride 4320, with the eight MDSS clocks of the mmcc and its MDSS_GDSC power domain (research
# walleye-display-flip-15-25-read-from-logs). Prints one line `pass|fail|skip walleye-dtb-display-splash kernel DETAIL`; exit 0 only on pass. Read-only.
# Environment: DTB (the dtb to read; default the splash image's dtb).
DTB=${DTB:-$HOME/Projects/walleye-build/images-display-splash/splash.dtb}
say() { echo "$1 walleye-dtb-display-splash kernel $2"; [ "$1" = pass ]; exit $?; }
command -v fdtget >/dev/null || say skip "fdtget (dtc) not installed: not known yet"
[ -r "$DTB" ] || say skip "need DTB=$DTB readable: not known yet"
N=$(basename "$DTB")
R=/reserved-memory/memory@9d400000
[ "$(fdtget -t u "$DTB" "$R" reg 2>/dev/null)" = "0 2638217216 0 37748736" ] || say fail "$N: $R is not reg 0x9d400000 0x2400000"
fdtget -p "$DTB" "$R" | grep -qx no-map || say fail "$N: $R is not no-map"
for c in $(fdtget -l "$DTB" /reserved-memory); do
  [ "$c" = memory@9d400000 ] && continue
  set -- $(fdtget -t u "$DTB" "/reserved-memory/$c" reg 2>/dev/null) || continue
  [ $# -eq 4 ] || continue
  s=$(( $1 * 4294967296 + $2 )); e=$(( s + $3 * 4294967296 + $4 ))
  [ "$e" -le 2638217216 ] || [ "$s" -ge 2675965952 ] || say fail "$N: /reserved-memory/$c overlaps the splash region"
done
F=/chosen/framebuffer@9d400000
[ "$(fdtget "$DTB" "$F" compatible 2>/dev/null)" = simple-framebuffer ] || say fail "$N: no simple-framebuffer at $F"
[ "$(fdtget -t u "$DTB" "$F" reg)" = "0 2638217216 0 37748736" ] || say fail "$N: $F's reg is not the splash region"
[ "$(for p in width height stride; do fdtget -t u "$DTB" "$F" $p; done | tr "\n" " ")" = "1080 1920 4320 " ] || say fail "$N: $F is not 1080x1920 stride 4320"
[ "$(fdtget -t s "$DTB" "$F" format)" = a8r8g8b8 ] || say fail "$N: $F's format is not a8r8g8b8"
MMCC=$(fdtget -t u "$DTB" "$(fdtget "$DTB" /__symbols__ mmcc)" phandle) || say fail "$N: no mmcc"
set -- $(fdtget -t u "$DTB" "$F" clocks)
[ $# -eq 16 ] || say fail "$N: $F has $(( $# / 2 )) clocks, not 8"
ids=
while [ $# -gt 0 ]; do
  [ "$1" = "$MMCC" ] || say fail "$N: a clock of $F is not the mmcc's"
  ids="$ids $2"; shift 2
done
[ "$ids" = " 62 64 70 67 72 82 65 74" ] || say fail "$N: $F's clock ids are$ids, not the eight MDSS clocks"
[ "$(fdtget -t u "$DTB" "$F" power-domains)" = "$MMCC 4" ] || say fail "$N: $F's power domain is not the mmcc's MDSS_GDSC"
say pass "$N reserves the splash and holds it with a simple-framebuffer (8 clocks, MDSS_GDSC)"
