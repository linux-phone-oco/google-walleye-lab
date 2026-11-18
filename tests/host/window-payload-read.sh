#!/bin/sh
# walleye-window-payload-read, run on the lab host: read the vfa..vfd window (the phone's 2 MiB mass_storage LUN, m1's) and take its payload out by its header.
# The window: bytes 0-511 "WALLEYE-VFx LEN=<10 digits> PARTS=<n> TRUNC=<0|1>" and a "PART name LEN=<n> RC=<rc>" line per part; from byte 512 the parts' plain bytes in that order; the last 4096 bytes the init's beat
# ("WALLEYE-VFx BEAT=<n> UP=<s>"). One dd of the LUN takes header and payload together; the parts are cut from it by the lengths. Finds the LUN by the strings the init sets (serial walleye-first-boot, product
# "Pixel 2 walleye (mainline)"), never by VID:PID; reads with O_DIRECT; changes nothing (the LUN is read-only for the host).
# Each part goes to OUT/NAME (fdt as fdt.dtb, and fdt.dts through dtc -I dtb -O dts when dtc is there); OUT/header.txt keeps the header, OUT/beat.txt the beat block. The text parts are checked for a serial or a MAC-shaped string
# (the init redacts them; a hit makes this a fail). The device tree is hardware description of the phone and stays in the lab's material directory, never in git.
# One line at the end: `pass|fail|skip walleye-window-payload-read kernel DETAIL`; exit 0 only on pass (pass = the header is ours, the lengths add up, every part has the bytes its header says, and no part has rc other than 0).
#
#   tests/host/window-payload-read.sh
# Environment: LUN_WAIT seconds to wait for the gadget (default 0: look once); LUN_DEV the block device (or VF_FILE a plain file, the self-test) instead of finding it;
#              VF_OUT the output directory (default $HOME/Projects/walleye-build/material/vf/2026-11-18-vfx, then the image letter is added: ...-vfa).
WAIT=${LUN_WAIT:-0}
say() { echo "$1 walleye-window-payload-read kernel $2"; [ "$1" = pass ]; exit $?; }
find_usb() {
  for d in /sys/bus/usb/devices/*; do
    [ -r "$d/serial" ] || continue
    if [ "$(cat "$d/serial" 2>/dev/null)" = walleye-first-boot ] || [ "$(cat "$d/product" 2>/dev/null)" = "Pixel 2 walleye (mainline)" ]; then echo "$d"; return; fi
  done
}
find_blk() { for b in "$1"/*/host*/target*/*/block/*; do [ -e "$b" ] && { echo "/dev/${b##*/}"; return; }; done; }
if [ -n "$VF_FILE" ]; then BLK=$VF_FILE; DIRECT=; USB=file
else
  DIRECT=iflag=direct; n=0; USB=; BLK=
  while :; do
    USB=$(find_usb); [ -n "$USB" ] && BLK=${LUN_DEV:-$(find_blk "$USB")}
    [ -n "$BLK" ] && break
    [ "$n" -ge "$WAIT" ] && break
    sleep 1; n=$((n+1))
  done
  [ -n "$USB" ] || say fail "no USB device with our gadget strings (the phone is in fastboot, off, or hung): $(lsusb 2>/dev/null | grep -ci google) google device(s) on the bus"
  [ -n "$BLK" ] || say fail "gadget ${USB##*/} enumerated but no block device under it (usb-storage not bound, or the LUN has no medium yet)"
  [ -r "$BLK" ] || say skip "gadget ${USB##*/} is $BLK but it is not readable by $(id -un) (disk group or sudo needed)"
fi
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
# the beat first (the init's last 4 KiB), so a dead init shows even when the payload is not there
dd if="$BLK" bs=4096 skip=511 count=1 $DIRECT 2>/dev/null | tr -d '\000' > "$T/beat"
dd if="$BLK" bs=512 count=1 $DIRECT 2>/dev/null | tr -d '\000' > "$T/head"
H1=$(head -n1 "$T/head")
case "$H1" in
  WALLEYE-VF[abcd]\ LEN=*) ;;
  *) say fail "gadget ${USB##*/} $BLK read, but the header is not ours: '$(echo "$H1" | cut -c1-60)' (the init wrote nothing, or it is not ours); beat: '$(head -n1 "$T/beat" | cut -c1-60)'" ;;
esac
L=$(echo "$H1" | sed -n 's/^WALLEYE-VF\([abcd]\) .*/\1/p')
LEN=$(echo "$H1" | sed -n 's/.* LEN=\([0-9]*\) .*/\1/p' | sed 's/^0*\([0-9]\)/\1/')
NPARTS=$(echo "$H1" | sed -n 's/.* PARTS=\([0-9]*\) .*/\1/p')
TRUNC=$(echo "$H1" | sed -n 's/.* TRUNC=\([01]\).*/\1/p')
OUT=${VF_OUT:-$HOME/Projects/walleye-build/material/vf/2026-11-18-vfx}
[ -n "$VF_OUT" ] || OUT=${OUT%vfx}vf$L
mkdir -p "$OUT" && cp "$T/head" "$OUT/header.txt" && cp "$T/beat" "$OUT/beat.txt"
BEAT=$(head -n1 "$T/beat" | cut -c1-60)
[ -n "$LEN" ] && [ -n "$NPARTS" ] || say fail "header unreadable: '$H1'"
[ "$LEN" -le 2092544 ] || say fail "vf$L: the header says LEN=$LEN, more than the window holds"
[ "$LEN" -le 2092544 ] || say fail "vf$L: the header says LEN=$LEN, more than the window holds"
# a header whose LEN is still 0 and no parts: the init was killed inside its first read (the header is written before it), or the source was empty
if [ "$NPARTS" -eq 0 ]; then say fail "vf$L: the init started its payload (header written) but no part ended: killed or hung in the first read; beat: '$BEAT'"; fi
# header, then LEN payload bytes, in one read of the LUN
BLOCKS=$(( (LEN + 511) / 512 + 1 ))
dd if="$BLK" bs=512 count="$BLOCKS" $DIRECT 2>/dev/null > "$T/all"
dd if="$T/all" bs=512 skip=1 2>/dev/null | head -c "$LEN" > "$T/payload"
GOT=$(stat -c %s "$T/payload")
[ "$GOT" -eq "$LEN" ] || say fail "vf$L: the header says $LEN payload bytes, the LUN gave $GOT"
SUM=0; OFF=0; BAD=; NP=0
for ln in $(grep '^PART ' "$T/head" | tr ' ' '|'); do
  name=$(echo "$ln" | cut -d'|' -f2); plen=$(echo "$ln" | sed -n 's/.*LEN=\([0-9]*\).*/\1/p'); prc=$(echo "$ln" | sed -n 's/.*RC=\([0-9]*\).*/\1/p')
  [ -n "$plen" ] && [ -n "$prc" ] || { BAD="$BAD $name:bad-part-line"; continue; }
  NP=$((NP+1))
  f=$name; [ "$name" = fdt ] && f=fdt.dtb
  tail -c +$((OFF + 1)) "$T/payload" | head -c "$plen" > "$OUT/$f"
  [ "$TRUNC" = 1 ] || [ "$(stat -c %s "$OUT/$f")" -eq "$plen" ] || BAD="$BAD $name:short"
  [ "$prc" -eq 0 ] || BAD="$BAD $name:rc=$prc"
  [ "$name" = fdt ] && [ "$plen" -gt 0 ] && [ "$(head -c 4 "$OUT/$f" | od -An -tx1 | tr -d ' \n')" != d00dfeed ] && BAD="$BAD fdt:no-dtb-magic"
  OFF=$((OFF + plen)); SUM=$((SUM + plen))
done
[ "$NP" -eq "$NPARTS" ] || BAD="$BAD parts:$NP-of-$NPARTS"
[ "$TRUNC" = 1 ] || [ "$SUM" -eq "$LEN" ] || BAD="$BAD lengths:$SUM-of-$LEN"
# the readable form of the device tree
DTSNOTE=
if [ -s "$OUT/fdt.dtb" ]; then
  if command -v dtc >/dev/null 2>&1; then dtc -I dtb -O dts -o "$OUT/fdt.dts" "$OUT/fdt.dtb" 2>"$T/dtc.err" && DTSNOTE="; fdt.dts $(wc -l < "$OUT/fdt.dts") lines" || DTSNOTE="; dtc failed: $(head -n1 "$T/dtc.err" | cut -c1-60)"; fi
fi
# identifying strings in a text part: a serial or a MAC-shaped string that the init should have redacted
for pn in $(grep "^PART " "$T/head" | cut -d" " -f2); do f=$OUT/$pn
  case "${f##*/}" in fdt.dtb|fdt.dts|header.txt|beat.txt) continue;; esac
  if grep -v REDACTED "$f" 2>/dev/null | grep -q -i -E "serialno[= :]|([0-9a-f]{2}[:-]){5}[0-9a-f]{2}"; then BAD="$BAD ${f##*/}:identifying"; fi
done
[ -z "$BAD" ] || say fail "vf$L: $NP part(s) of $NPARTS read, problems:$BAD; beat: '$BEAT'"
say pass "vf$L: $NP part(s), $LEN bytes$([ "$TRUNC" = 1 ] && echo ' (TRUNCATED to the window)') in $OUT: $(ls "$OUT" | tr '\n' ' ')$DTSNOTE; beat: '$BEAT'"
