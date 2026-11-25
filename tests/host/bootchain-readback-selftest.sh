#!/bin/sh
# walleye-bootchain-readback-selftest: proves bootchain-readback-check.sh and bootchain-readback.sh on a fake DIR and a fake adb (the phone is never touched).
# The check is made to FAIL first (empty DIR, missing file, wrong hash, wrong size, wrong mode, differing second read), then PASS; the reader refuses and reads.
# Prints one line, `pass|fail walleye-bootchain-readback-selftest host DETAIL`; exit 0 only on pass.
HERE=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d) || exit 1; trap 'rm -rf "$T"' EXIT
fail() { echo "fail walleye-bootchain-readback-selftest host $1"; exit 1; }
CHECK=$HERE/bootchain-readback-check.sh
LIST=$HERE/bootchain-parts.list
must_fail() { # WHAT CMD...
  w=$1; shift; out=$("$@" 2>&1); [ $? -eq 1 ] || fail "$w: did not exit 1: $out"
  case $out in *"fail walleye-bootchain"*) ;; *) fail "$w: no fail line: $out";; esac; }
must_pass() { w=$1; shift; out=$("$@" 2>&1); [ $? -eq 0 ] || fail "$w: did not pass: $out"
  case $out in *"pass walleye-bootchain"*) ;; *) fail "$w: no pass line: $out";; esac; }

# the fake phone's tables: the by-name listing is text the phone printed (tests/fixtures/phone-text/partition-table-android-b-2026-11-25.cap, Android b).
# and so is /proc/partitions: Android b's root read, tests/fixtures/phone-text/proc-partitions-android-b-2026-11-23.cap (bootchain-2026-11-23-gpt-equal-run3);
# each name's row is the line the phone printed for its device, and its 1 KiB blocks give the image's size - no row is typed.
FIX=$HERE/../fixtures/phone-text
mkphone() { # DIR (a directory the fake adb reads)
  P=$1; mkdir -p "$P"; : > "$P/names"; : > "$P/procparts"; : > "$P/sizes"
  cp "$FIX/partition-table-android-b-2026-11-25.cap" "$P/byname-l"
  for n in $(cat "$LIST"); do
    dev=$(sed -n "s#^.* $n -> /dev/block/##p" "$P/byname-l")
    [ -n "$dev" ] || fail "$n is not in the captured by-name listing"
    row=$(awk -v d="$dev" '$4 == d' "$FIX/proc-partitions-android-b-2026-11-23.cap")
    [ -n "$row" ] || fail "$dev is not in the captured /proc/partitions"
    echo "$n" >> "$P/names"
    echo "$row" >> "$P/procparts"
    echo "$n $(( $(echo "$row" | awk '{print $3}') * 1024 ))" >> "$P/sizes"
  done; }
# a fake complete DIR, made without the reader
mkdirfake() { # DIR PHONEDIR
  rm -rf "$1"; mkdir -m 700 "$1"; cp "$2/byname-l" "$1/partition-table.txt"; cp "$2/procparts" "$1/proc-partitions.txt"; : > "$1/sha256sums"
  while read -r n s; do head -c "$s" /dev/zero | tr '\0' 'a' > "$1/$n.img";
    (cd "$1" && sha256sum "$n.img") >> "$1/sha256sums"; done < "$2/sizes"
  chmod 600 "$1"/*; }
mkphone "$T/phone"
[ "$(wc -l < "$LIST")" -eq 32 ] || fail "the list holds $(wc -l < "$LIST") names, not 32"

# --- the check
mkdir -m 700 "$T/empty"
must_fail "empty DIR" sh "$CHECK" "$T/empty"
must_fail "no DIR" sh "$CHECK" "$T/nothing-here"
must_fail "no argument" sh "$CHECK"
mkdirfake "$T/d1" "$T/phone"; must_pass "complete fake DIR" sh "$CHECK" "$T/d1"
rm "$T/d1/tz_a.img"; must_fail "missing file" sh "$CHECK" "$T/d1"
mkdirfake "$T/d1" "$T/phone"; printf 'b' | dd of="$T/d1/abl_a.img" bs=1 conv=notrunc 2>/dev/null; must_fail "mismatched content" sh "$CHECK" "$T/d1"
mkdirfake "$T/d1" "$T/phone"; head -c 100 "$T/d1/hyp_b.img" > "$T/x"; cat "$T/x" > "$T/d1/hyp_b.img"
(cd "$T/d1" && grep -v ' hyp_b.img' sha256sums > s && sha256sum hyp_b.img >> s && cat s > sha256sums && rm s); must_fail "wrong size, matching sha" sh "$CHECK" "$T/d1"
mkdirfake "$T/d1" "$T/phone"; grep -v ' rpm_a.img' "$T/d1/sha256sums" > "$T/s"; cat "$T/s" > "$T/d1/sha256sums"; must_fail "no sha256sums line" sh "$CHECK" "$T/d1"
mkdirfake "$T/d1" "$T/phone"; chmod 644 "$T/d1/sec.img"; must_fail "file mode 644" sh "$CHECK" "$T/d1"
mkdirfake "$T/d1" "$T/phone"; chmod 755 "$T/d1"; must_fail "dir mode 755" sh "$CHECK" "$T/d1"
mkdirfake "$T/d1" "$T/phone"; echo cdt > "$T/d1/missing"; must_fail "a missing file from the reader" sh "$CHECK" "$T/d1"
mkdirfake "$T/d1" "$T/phone"; must_pass "xbl 4052 KiB = 8104 sectors of 512 (the phone's own /proc/partitions)" env XBL_SECTORS=8104 sh "$CHECK" "$T/d1"
must_fail "xbl not 24 sectors" env XBL_SECTORS=24 sh "$CHECK" "$T/d1"
mkdirfake "$T/d2" "$T/phone"; must_pass "second read of the same bytes" sh "$CHECK" "$T/d1" "$T/d2"
printf 'b' | dd of="$T/d2/cdt.img" bs=1 conv=notrunc 2>/dev/null; (cd "$T/d2" && grep -v ' cdt.img' sha256sums > s && sha256sum cdt.img >> s && cat s > sha256sums && rm s)
must_fail "second read of other bytes" sh "$CHECK" "$T/d1" "$T/d2"

# --- the reader, with a fake adb and the real battery-ok.sh
cp "$HERE/../../device.toml" "$T/device.toml" 2>/dev/null || printf '[fastboot]\nserial = "FAKE123"\n' > "$T/device.toml"
mkdir "$T/bin"
mkadb() { # STATE VOLTAGE
  cat > "$T/bin/adb" <<ADB
#!/bin/sh
P=$T/phone
echo "\$*" >> \$P/calls
case "\$*" in
  *get-state*) echo "$1";;
  *dumpsys*) echo "  voltage: $2";;
  *"ls -l /dev/block/by-name"*) cat \$P/byname-l;;
  *"ls /dev/block/by-name"*) cat \$P/names;;
  *"cat /proc/partitions"*) cat \$P/procparts;;
  *"dd if=/dev/block/by-name/"*) a="\$*"; n=\${a#*by-name/}; n=\${n%% *}; s=\$(awk -v n="\$n" '\$1==n{print \$2}' \$P/sizes); head -c "\$s" /dev/zero | tr "\\000" a;;
  *) echo "unexpected adb call: \$*" >&2; exit 9;;
esac
ADB
  chmod +x "$T/bin/adb"; }
reader() { env PATH="$T/bin:$PATH" DEVICE_TOML="$T/device.toml" CALL_WAIT=5 sh "$HERE/bootchain-readback.sh" "$@" 2>&1; }
mkadb offline 4000; out=$(reader "$T/r0"); [ $? -eq 1 ] && case $out in *refused*not\ device*) ;; *) false;; esac || fail "adb offline: $out"
[ ! -e "$T/r0" ] || fail "a refused run made a DIR"
mkadb device 3000; out=$(reader "$T/r0"); [ $? -eq 1 ] && case $out in *refused*battery*) ;; *) false;; esac || fail "low battery: $out"
[ ! -e "$T/r0" ] || fail "a battery refusal made a DIR"
mkadb device 4000
mkdir -p "$T/occupied"; : > "$T/occupied/x"; out=$(reader "$T/occupied"); [ $? -eq 1 ] && case $out in *refused*already*) ;; *) false;; esac || fail "occupied DIR: $out"
(cd "$T" && git init -q repo) && out=$(reader "$T/repo/out"); [ $? -eq 1 ] && case $out in *refused*git*) ;; *) false;; esac || fail "DIR in a repository: $out"
out=$(reader "$T/r1"); [ $? -eq 0 ] || fail "the read failed: $out"
must_pass "reader's DIR 1 checks" sh "$CHECK" "$T/r1"
out=$(reader "$T/r2"); [ $? -eq 0 ] || fail "second read failed: $out"
must_pass "two reads of the same bytes" sh "$CHECK" "$T/r1" "$T/r2"
[ "$(stat -c %a "$T/r1")" = 700 ] && [ "$(stat -c %a "$T/r1/xbl_a.img")" = 600 ] || fail "reader modes are not 700/600"
# a name the phone does not list: reported in `missing`, the read fails, the check fails
grep -vx 'cdt' "$T/phone/names" > "$T/n"; cp "$T/phone/names" "$T/names.keep"; cat "$T/n" > "$T/phone/names"
out=$(reader "$T/r3"); [ $? -eq 1 ] && case $out in *"missing cdt"*) ;; *) false;; esac || fail "missing name: $out"
must_fail "check on a DIR with a missing name" sh "$CHECK" "$T/r3"
# the reader is read only: every adb call it made is a get-state, ls, ls -l, cat /proc/partitions, dumpsys, or a dd with if= and no of= or redirect
cp "$T/names.keep" "$T/phone/names"
[ -s "$T/phone/calls" ] || fail "no adb calls were logged"
bad=$(grep -v -e 'get-state$' -e 'dumpsys battery$' -e 'shell ls /dev/block/by-name$' -e 'shell ls -l /dev/block/by-name$' -e 'shell cat /proc/partitions$' -e 'shell id -u$' -e '^-s [^ ]* exec-out su -c dd if=/dev/block/by-name/[a-z0-9_]* bs=4096 2>/dev/null$' "$T/phone/calls" | head -n 3)
[ -z "$bad" ] || fail "the reader made a call that is not a read: $bad"
# an su that fails writes its error into the image: the reader must not say pass
mkadb device 4000; sed -i 's|^  \*"dd if=.*|  *"dd if=/dev/block/by-name/"*) echo "su: not found";;|' "$T/bin/adb"
out=$(reader "$T/r4"); [ $? -eq 1 ] && case $out in *"check on"*fails*) ;; *) false;; esac || fail "a failed su read passed: $out"
# a DIR below a missing parent inside a repository is refused too
mkadb device 4000
out=$(reader "$T/repo/new/sub/out"); [ $? -eq 1 ] && case $out in *refused*git*) ;; *) false;; esac || fail "DIR below a missing parent in a repository: $out"
[ ! -e "$T/repo/new" ] || fail "the refusal still made the DIR"
must_fail "the same DIR twice" sh "$CHECK" "$T/r1" "$T/r1"
echo "pass walleye-bootchain-readback-selftest host the reader only reads, refuses a failed su, a repository below a missing parent; the check refuses the same DIR twice"
