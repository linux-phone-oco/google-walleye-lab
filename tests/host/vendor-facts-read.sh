#!/bin/sh
# walleye-vendor-facts-read, run on the lab host: read the vf1 window (the phone's 8 MiB mass_storage LUN: 64-byte header "WALLEYE-VF1 LEN=<10 digits> ROUND=<2 digits>", then LEN bytes of a gzipped tar),
# check the length prefix, gunzip the tar into the output directory, write INDEX.txt (every file with its size), say which of the expected facts are missing, and refuse if a MAC or a serial is in it.
# Finds the LUN by the strings the init sets (serial walleye-first-boot, product "Pixel 2 walleye (mainline)"), never by VID:PID; reads it with O_DIRECT. Reads the host's USB tree and the block device only.
# One line at the end: `pass|fail|skip walleye-vendor-facts-read kernel DETAIL`; exit 0 only on pass (pass = the blob gunzips, the device tree and dmesg are in it, nothing identifying is). A partial dump is still a pass,
# with the missing facts named in the detail.
#
#   tests/host/vendor-facts-read.sh
# Environment: LUN_WAIT seconds to wait for the gadget (default 0: look once); LUN_DEV the block device (or VF_FILE a plain file, the self-test) instead of finding it;
#              VF_OUT the output directory (default $HOME/Projects/walleye-build/material/vf/2026-11-18-vf: hardware facts of our phone, large, kept out of git; the index is what to cite);
#              VF_TRIES reads to try when the stream does not gunzip (the init rewrites it every 10 s; default 12: the first round may take a while before any blob is written).
WAIT=${LUN_WAIT:-0}
OUT=${VF_OUT:-$HOME/Projects/walleye-build/material/vf/2026-11-18-vf}
TRIES=${VF_TRIES:-12}
say() { echo "$1 walleye-vendor-facts-read kernel $2"; [ "$1" = pass ]; exit $?; }
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
try=0; LEN=; ROUND=
while [ "$try" -lt "$TRIES" ]; do
  try=$((try+1))
  dd if="$BLK" bs=512 count=1 $DIRECT 2>/dev/null | head -c 64 | tr -d '\000' > "$T/head"
  grep -q '^WALLEYE-VF1 LEN=' "$T/head" || say fail "gadget ${USB##*/} $BLK read, but the header is not WALLEYE-VF1 (first line: $(head -n1 "$T/head" | cut -c1-60))"
  LEN=$(sed -n 's/^WALLEYE-VF1 LEN=\([0-9]*\) .*/\1/p' "$T/head" | sed 's/^0*\(.\)/\1/'); ROUND=$(sed -n 's/.*ROUND=\([0-9]*\).*/\1/p' "$T/head")
  [ "${LEN:-0}" -gt 0 ] || { sleep 5; continue; }
  [ "$LEN" -le 8388544 ] || say fail "header says LEN=$LEN, more than the 8 MiB window holds"
  N=$(( (LEN + 64 + 4095) / 4096 ))
  dd if="$BLK" bs=4096 count="$N" $DIRECT 2>/dev/null | tail -c +65 | head -c "$LEN" > "$T/vf.tgz"
  [ "$(stat -c %s "$T/vf.tgz")" -eq "$LEN" ] || { sleep 2; continue; }
  gzip -t "$T/vf.tgz" 2>/dev/null && break
  sleep 2
done
[ "${LEN:-0}" -gt 0 ] || say fail "gadget ${USB##*/} $BLK: the header says LEN=0 (the init has not written a blob yet: $try reads)"
gzip -t "$T/vf.tgz" 2>/dev/null || say fail "gadget ${USB##*/} $BLK: LEN=$LEN round $ROUND but the stream does not gunzip after $try reads"
rm -rf "$OUT"; mkdir -p "$OUT"
tar xzf "$T/vf.tgz" -C "$OUT" 2>/dev/null || { rm -rf "$OUT"; say fail "the stream gunzips but the tar does not extract into $OUT (removed)"; }
FILES=$(find "$OUT" -type f | wc -l)
# redaction: nothing identifying may be in ANY extracted file (device tree values and .dtb included, read as bytes with grep -a); a refused dump is deleted whole
bad=0
[ -d "$OUT/sys/class/net" ] && bad=$((bad+1))
for f in $(find "$OUT" -type f); do
  grep -a -q -E '([0-9a-fA-F]{2}:){5}[0-9a-fA-F]{2}|([0-9a-fA-F]{2}-){5}[0-9a-fA-F]{2}' "$f" 2>/dev/null && { echo "MAC-shaped bytes in $f" >&2; bad=$((bad+1)); }
  grep -a -q -E '(^|[^0-9])[0-9]{15}([^0-9]|$)' "$f" 2>/dev/null && { echo "IMEI-shaped digits in $f" >&2; bad=$((bad+1)); }
  grep -a -q -E '^Serial[[:space:]]*:' "$f" 2>/dev/null && { echo "serial line in $f" >&2; bad=$((bad+1)); }
  grep -a -o -E 'serialno[=: ][^ ]*' "$f" 2>/dev/null | grep -a -q -v '^serialno=REDACTED' && { echo "serialno value in $f" >&2; bad=$((bad+1)); }
done
find "$OUT/dt" -type f \( -iname '*mac-address*' -o -iname '*bdaddr*' -o -iname '*bd-addr*' -o -iname '*bd_addr*' -o -iname '*serial-number*' -o -iname '*serialno*' -o -iname '*imei*' -o -iname '*macaddr*' \) 2>/dev/null | grep -q . && { echo "an identifying device-tree property is in dt/" >&2; bad=$((bad+1)); }
[ "$bad" -eq 0 ] || { rm -rf "$OUT"; say fail "$bad identifying item(s) found in the dump (see above); the dump is deleted, nothing left the machine"; }
(cd "$OUT" && find . -type f -printf '%s %p\n' | sort -k2 > INDEX.txt)
# what is there, what is missing (a first run may be partial)
missing=
for want in dmesg iomem interrupts cmdline debugfs/clk_summary debugfs/regulator_summary debugfs/gpio debugfs/pinctrl/pinctrl-maps sys/udc-list; do [ -s "$OUT/$want" ] || missing="$missing $want"; done
[ "$(find "$OUT/dt" -type f 2>/dev/null | wc -l)" -gt 0 ] || missing="$missing dt/"
[ -n "$(ls "$OUT/debugfs" 2>/dev/null | grep -i -e dwc3 -e usb)" ] || missing="$missing debugfs/usb-or-dwc3"
{ echo "# vf1 dump, round $ROUND, $LEN bytes gzipped, $FILES files; missing: ${missing:-nothing}"; cat "$OUT/INDEX.txt"; } > "$OUT/INDEX.tmp" && mv "$OUT/INDEX.tmp" "$OUT/INDEX.txt"
case "$missing" in *dt/*|*dmesg*) say fail "round $ROUND blob read ($FILES files) but missing$missing in $OUT" ;; esac
say pass "gadget ${USB##*/} $BLK: round $ROUND, $LEN bytes, $FILES files in $OUT, missing:${missing:- nothing}"
