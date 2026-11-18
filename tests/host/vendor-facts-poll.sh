#!/bin/sh
# walleye-vendor-facts-poll, run on the lab host BEFORE the vf3 boot, in its own background job: watch the phone's 4 MiB mass_storage window and keep everything it ever showed.
# (vf4 reuses it: see VF_MARK/VF_HEAD/VF_BLOCKS below; the 64 KiB text region is also kept as OUT/steps.txt.) The vf3 window is plain text: block 0 (4096 bytes) the beat "WALLEYE-VF3 BEAT=n UP=s STAGE=x BYTES=n", then from byte 4096 framed blocks "BEGIN name" ... "END name rc=N", the last line "DONE ...".
# Every POLL_SECS (default 2) it finds the LUN by the strings the init sets (serial walleye-first-boot, product "Pixel 2 walleye (mainline)"; never by VID:PID), reads the whole window with O_DIRECT,
# drops the NULs (the used part is what is left), and
#   - saves the used part as the next numbered copy OUT/win/NNN.txt when it differs from the last one, and the beat line of every read to OUT/beat.log;
#   - keeps OUT/last-complete.txt: the last copy that ends on a finished block (its last line is an END or DONE line);
#   - writes each finished block (BEGIN..END) to OUT/vf/NAME the moment it shows (name checked: letters, digits . _ @ , + - and /, no .. and no leading /), once.
# It stops at DONE, or after POLL_MAX seconds (default 150), or at once when an identifying item shows in what it wrote (MAC-shaped, 15 digits, a serialno value): then it deletes OUT, says fail.
# One line at the end: `pass|fail|skip walleye-vendor-facts-poll host DETAIL`; exit 0 only on pass (pass = at least one finished block was fetched; DONE or not is in the detail: a partial window is still a pass).
# It has no trap that kills anything but its own temp directory: started with nohup/setsid in its own job it outlives the shell that started it.
#
#   tests/host/vendor-facts-poll.sh
# Environment: VF_OUT output directory (default $HOME/Projects/walleye-build/material/vf/2026-11-18-vf3: hardware facts of our phone, large, kept out of git); POLL_SECS, POLL_MAX seconds;
#              POLL_ONCE=1 read once and stop (the self-test); LUN_DEV the block device, or VF_FILE a plain file (the self-test), instead of finding it.
OUT=${VF_OUT:-$HOME/Projects/walleye-build/material/vf/2026-11-18-vf3}
EVERY=${POLL_SECS:-2}
MAX=${POLL_MAX:-150}
ONCE=${POLL_ONCE:-0}
# the window's layout, vf3 by default; vf4 (vf2's 64 KiB text region, the stream from byte 65536): VF_MARK=WALLEYE-VF2 VF_HEAD=16 VF_BLOCKS=1040
MARK=${VF_MARK:-WALLEYE-VF3}
HEAD=${VF_HEAD:-1}
NBLK=${VF_BLOCKS:-1024}
say() { echo "$1 walleye-vendor-facts-poll host $2"; [ "$1" = pass ]; exit $?; }
find_usb() {
  for d in /sys/bus/usb/devices/*; do
    [ -r "$d/serial" ] || continue
    if [ "$(cat "$d/serial" 2>/dev/null)" = walleye-first-boot ] || [ "$(cat "$d/product" 2>/dev/null)" = "Pixel 2 walleye (mainline)" ]; then echo "$d"; return; fi
  done
}
find_blk() { for b in "$1"/*/host*/target*/*/block/*; do [ -e "$b" ] && { echo "/dev/${b##*/}"; return; }; done; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT; trap 'exit 1' INT TERM HUP
rm -rf "$OUT"; mkdir -p "$OUT/win" "$OUT/vf"
: > "$OUT/beat.log"; : > "$OUT/.done"
# fetch: write every finished, new, well-named block of $1 into OUT/vf (an awk pass; a block without its END line is left for the next read)
fetch() {
  awk -v out="$OUT/vf" -v done="$OUT/.done" '
    BEGIN { while ((getline l < done) > 0) seen[l] = 1; close(done) }
    /^BEGIN / { name = $2; buf = ""; open = 1; next }
    /^END / && open {
      if ($2 == name && !(name in seen) && name ~ /^[A-Za-z0-9][A-Za-z0-9._@,+%-]*$/ && name !~ /\.\./) {
        path = out "/" name; dir = path; sub(/\/[^\/]*$/, "", dir)
        system("mkdir -p \"" dir "\"")
        printf "%s", buf > path; printf "%s\n", $0 >> path; close(path)
        print name >> done; close(done); seen[name] = 1; n++
      }
      open = 0; next
    }
    open { buf = buf $0 "\n" }
    END { print n + 0 }
  ' "$1"
}
leaks() {
  bad=0
  for f in "$@"; do
    grep -a -q -E '([0-9a-fA-F]{2}:){5}[0-9a-fA-F]{2}|([0-9a-fA-F]{2}-){5}[0-9a-fA-F]{2}' "$f" 2>/dev/null && { echo "MAC-shaped bytes in $f" >&2; bad=$((bad+1)); }
    grep -a -q -E '(^|[^0-9])[0-9]{15}([^0-9]|$)' "$f" 2>/dev/null && { echo "IMEI-shaped digits in $f" >&2; bad=$((bad+1)); }
    grep -a -o -E 'serialno[=: ][^ ]*' "$f" 2>/dev/null | grep -a -q -v '^serialno=REDACTED' && { echo "serialno value in $f" >&2; bad=$((bad+1)); }
  done
  return $bad
}
START=$(date +%s); NCOPY=0; BLOCKS=0; SEEN_MARKER=0; DONE=0; LAST=; BEATS=0; WHERE=
while :; do
  if [ -n "$VF_FILE" ]; then BLK=$VF_FILE; DIRECT=; WHERE=file
  else
    DIRECT=iflag=direct; USB=$(find_usb); BLK=; [ -n "$USB" ] && BLK=${LUN_DEV:-$(find_blk "$USB")}
    [ -n "$BLK" ] && WHERE="${USB##*/} $BLK"
  fi
  if [ -n "$BLK" ] && [ -r "$BLK" ]; then
    if dd if="$BLK" of="$T/raw" bs=4096 count=$NBLK $DIRECT 2>/dev/null && [ -s "$T/raw" ]; then
      dd if="$T/raw" bs=4096 count=1 2>/dev/null | tr -d '\000' | head -n1 > "$T/beat"
      if grep -q "^$MARK BEAT=" "$T/beat"; then
        SEEN_MARKER=1; BEATS=$((BEATS+1)); echo "$(( $(date +%s) - START )) $(cat "$T/beat")" >> "$OUT/beat.log"
        # the contiguous text after block 0 up to the first hole: bytes read across a rewrite never glue; a torn last line is dropped
        dd if="$T/raw" bs=4096 skip=$HEAD 2>/dev/null | head -z -n1 | tr -d '\000' > "$T/used"
        [ -z "$(tail -c1 "$T/used")" ] || sed -i '$d' "$T/used"
        leaks "$T/used" || { rm -rf "$OUT"; say fail "an identifying item showed in the window (see above); the output is deleted, nothing left the machine"; }
        if ! cmp -s "$T/used" "$T/last"; then
          NCOPY=$((NCOPY+1)); cp "$T/used" "$OUT/win/$(printf %03d $NCOPY).txt"; cp "$T/used" "$T/last"
          [ "$(tail -n1 "$T/used" | cut -c1-4)" = "END " ] || [ "$(tail -n1 "$T/used" | cut -c1-4)" = "DONE" ] && cp "$T/used" "$OUT/last-complete.txt"
          fetch "$T/used" > "$T/n"
        fi
        if [ "$HEAD" -gt 1 ]; then dd if="$T/raw" bs=4096 count=$HEAD 2>/dev/null | tr -d '\000' > "$T/hdr"; leaks "$T/hdr" || { rm -rf "$OUT"; say fail "an identifying item showed in the text region (see above); the output is deleted, nothing left the machine"; }; cmp -s "$T/hdr" "$T/hdrlast" || { cp "$T/hdr" "$OUT/steps.txt"; cp "$T/hdr" "$T/hdrlast"; }; fi
        grep -q '^DONE ' "$T/used" && { DONE=1; break; }
      fi
    fi
  fi
  [ "$ONCE" = 1 ] && break
  [ $(( $(date +%s) - START )) -ge "$MAX" ] && break
  sleep "$EVERY"
done
BLOCKS=$(grep -c . "$OUT/.done")
(cd "$OUT" && find vf -type f -printf '%s %p\n' | sort -k2 > INDEX.txt)
LASTBEAT=$(tail -n1 "$OUT/beat.log" | cut -c1-80)
[ "$SEEN_MARKER" = 1 ] || { rm -rf "$OUT"; say fail "no window with our marker in $(( $(date +%s) - START )) s (${WHERE:-no USB device with our gadget strings; phone-state: $(sh "$(dirname "$0")/phone-state.sh" 2>&1 | grep "^verdict" | head -n1 | grep . || echo unavailable)}); nothing left"; }
[ "$BLOCKS" -gt 0 ] || say fail "the window showed its marker ($BEATS beats, last: $LASTBEAT) but no block ended; copies in $OUT/win"
if [ "$DONE" = 1 ]; then END=DONE; else END="no DONE, partial"; fi
say pass "$BLOCKS blocks in $OUT/vf, $NCOPY copies, $BEATS beats, $END (last beat: $LASTBEAT)"
