#!/bin/sh
# Every init of ours that sets a USB gadget serial sets walleye-first-boot, the one serial the host's udev rule knows (lab PR 175: walleye-banner made the tty root-only).
#   check-init-serials.sh [BUILD_DIR]   exit 1 when an init sets another serial, or a serialnumber line is in a form this check cannot read
# Read forms: echo "X" > .../serialnumber and stp|gst g-serial put X .../serialnumber. Any other non-comment line naming serialnumber fails (never skipped unseen).
set -eu
B=${1:-$(cd "$(dirname "$0")" && pwd)}
bad=0; n=0
for f in "$B"/initramfs/init* "$B"/initramfs-*/init*; do
  [ -f "$f" ] || continue
  lines=$(grep -v '^[[:space:]]*#' "$f" | grep -c 'serialnumber' || true)
  seen=0
  for s in $(grep -v '^[[:space:]]*#' "$f" | sed -n -e 's|^[[:space:]]*echo "\([^" ]*\)" *> *[^ ]*/strings/0x409/serialnumber[[:space:]]*$|\1|p' -e 's#^[[:space:]]*\(stp\|gst\) g-serial put \([^ ]*\) [^ ]*/strings/0x409/serialnumber[[:space:]]*$#\2#p'); do
    seen=$((seen+1)); n=$((n+1))
    if [ "$s" = walleye-first-boot ]; then echo "ok   $f: $s"; else echo "FAIL $f: serial '$s', want walleye-first-boot"; bad=$((bad+1)); fi
  done
  if [ "$seen" -ne "$lines" ]; then echo "FAIL $f: $lines serialnumber line(s), $seen read"; bad=$((bad+1)); fi
done
[ $n -gt 0 ] || { echo "FAIL no serial found under $B" >&2; exit 1; }
[ $bad -eq 0 ] || { echo "check-init-serials: $bad failure(s)" >&2; exit 1; }
echo "check-init-serials: $n serial lines, all walleye-first-boot"
