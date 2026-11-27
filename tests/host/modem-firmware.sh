#!/bin/sh
# walleye-modem-firmware, run on the lab host: does a firmware folder hold, for every segment of modem.mdt that has bytes, the file the kernel's mdt loader reads for it (item walleye-modem-mss-image)?
# The loader (drivers/soc/qcom/mdt_loader.c) reads modem.mdt (the ELF header and program headers), then for program header N with a file size above 0 the file modem.bNN; a header of file size 0 has no file.
# Passes only when every such file is there with exactly that size (b00 and b01 are the first two headers: the ELF header and the hash segment), mba.mbn is there, and no bNN is there beyond the headers.
# Prints one line `pass|fail|skip walleye-modem-firmware modem DETAIL`; exit 0 only on pass. Read-only. Environment: FW_DIR (the folder, no default), HEADERS (text: readelf -lW of the mdt; default: run on FW_DIR/modem.mdt; the
# selftest feeds tests/fixtures/phone-text/walleye-modem-mdt-readelf.cap this way).
say() { echo "$1 walleye-modem-firmware modem $2"; [ "$1" = pass ]; exit $?; }
[ -d "${FW_DIR:-}" ] || say skip "need FW_DIR=folder (the modem_b image's /image files): not known yet"
command -v readelf >/dev/null || say skip "readelf not installed: not known yet"
[ -r "$FW_DIR/modem.mdt" ] || say fail "$FW_DIR has no modem.mdt"
[ -r "$FW_DIR/mba.mbn" ] || say fail "$FW_DIR has no mba.mbn"
T=${HEADERS:-$(readelf -lW "$FW_DIR/modem.mdt")}
sizes=$(printf '%s\n' "$T" | awk '$1=="NULL"||$1=="LOAD" { printf "%d %d\n", n++, strtonum($5) }')
[ -n "$sizes" ] || say fail "no program headers read from the mdt text"
miss=$(printf '%s\n' "$sizes" | while read -r i sz; do
  if [ "$sz" -eq 0 ]; then [ ! -e "$(printf "%s/modem.b%02d" "$FW_DIR" "$i")" ] || echo "modem.b$(printf %02d "$i") exists, the header has no bytes"; continue; fi
  f=$(printf '%s/modem.b%02d' "$FW_DIR" "$i")
  if [ ! -f "$f" ]; then echo "modem.b$(printf %02d "$i") missing"
  elif [ "$(stat -c %s "$f")" -ne "$sz" ]; then echo "modem.b$(printf %02d "$i") is $(stat -c %s "$f") bytes, the header says $sz"; fi
done)
[ -z "$miss" ] || say fail "$(printf '%s' "$miss" | tr '\n' ';')"
n=$(printf '%s\n' "$sizes" | wc -l)
extra=$(ls "$FW_DIR" | sed -n 's/^modem\.b\([0-9][0-9]\)$/\1/p' | awk -v n="$n" '$1+0 >= n' | tr '\n' ' ')
[ -z "$extra" ] || say fail "segment files beyond the $n headers: $extra"
have=$(printf '%s\n' "$sizes" | awk '$2>0' | wc -l)
say pass "modem.mdt has $n program headers, $have with bytes, each with its modem.bNN of exactly that size; mba.mbn present"
