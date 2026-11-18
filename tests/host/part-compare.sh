#!/bin/sh
# part-compare: what does Android do for one part that our kernel does not? (decision 2026-11-19-walleye-compare-with-android-for-every-part)
# Capture a part's state on LIVE ANDROID and on OUR kernel with the same file list, normalise both, diff them. Read-only on the phone; it informs, it never decides.
#   part-compare.sh capture PART android [OUT]   adb (root where the build allows it; debugfs is mounted if missing)
#   part-compare.sh capture PART ours    [OUT]   the network shell of our image (telnet on the manifest's address)
#   part-compare.sh diff ANDROID_DIR OURS_DIR    print three groups per captured file: 'Android has, we lack' / 'we have, Android lacks' / 'same key, different value'
# The part's list is tests/host/part-compare.d/PART.list: one `NAME<TAB>COMMAND` per line (a POSIX sh one-liner that runs on the phone; # lines and blanks are skipped).
# NAME's prefix says how its answer is normalised: gpio (debugfs gpio -> chip/pin: dir level owner), pinmux, pinconf, regulator (name:state:microvolts), clk (clk_summary rows),
# dt (path=hex lines; phandle properties dropped), dmesg (timestamps dropped); any other name is a generic sorted key=value / whole-line set. A new part is a new list: see README.md.
# adb root restarts adbd on the phone (nothing else is changed; debugfs is mounted only when /proc/mounts lacks it). Answers go under OUT (default $HOME/lab-material/walleye/compare-PART-SIDE-DATE, never in git, mode 700), ids redacted.
# Environment: ADB (default adb), ADB_SERIAL (default: the only device), OURS_SHELL (default: telnet to the manifest's address, or nc), DEVICE_TOML (default ../../device.toml),
#   LISTDIR (default part-compare.d next to this script), CALL_WAIT (seconds per command on ours, default 3), SETTLE (seconds before the first send on ours, default 1).
# Exit 0 on a finished capture or diff (a diff always informs); 2 on bad usage or a transport that gave nothing at all.
HERE=$(cd "$(dirname "$0")" && pwd)
LISTDIR=${LISTDIR:-$HERE/part-compare.d}
ADB=${ADB:-adb}; CW=${CALL_WAIT:-3}; SETTLE=${SETTLE:-1}
die() { echo "part-compare: $*" >&2; exit 2; }
redact() { sed -E -e 's/\x1b\[[0-9;]*[A-Za-z]//g' -e 's/(androidboot\.serialno=)[^ ]*/\1REDACTED/g' -e 's/([0-9a-fA-F]{2}:){5}[0-9a-fA-F]{2}/XX:XX:XX:XX:XX:XX/g' -e 's/(serial(no|number)?[ =:]+)[0-9A-Za-z]{6,}/\1REDACTED/Ig'; }
clean() { sed -E 's/\x1b\[[0-9;]*[A-Za-z]//g' | tr -d '\000-\010\013-\037\377' | sed 's/\r//g'; }

# ---------- normalisers: stdin raw answer -> stdout `KEY<TAB>VALUE`, sorted, duplicate keys numbered ----------
norm() { # KIND
  case $1 in
  gpio) awk '
    function trim(s) { gsub(/^[ \t|]+|[ \t|]+$/, "", s); return s }
    /^(gpiochip[0-9]+: )?GPIOs [0-9]+-[0-9]+/ { r=$0; sub(/^(gpiochip[0-9]+: )?GPIOs /, "", r); b=r; sub(/-.*/, "", b); base=b+0; chip="chip"; sub(/^[0-9]+-[0-9]+,? */, "", r)
      if (match(r, /(parent: )?[^ ,]+\/[^ ,]+/)) { chip=substr(r, RSTART, RLENGTH); sub(/^parent: /, "", chip); sub(/^.*\//, "", chip) } else { sub(/:.*$/, "", r); if (r != "") chip=r } next }
    /^ *[A-Za-z_][A-Za-z_0-9]* *: +(in|out) / { n=$1; i=index($0, ":"); rest=substr($0, i+1); gsub(/^[ \t]+|[ \t]+$/, "", rest); gsub(/[ \t]+/, " ", rest); printf "%s/%s\t%s\n", chip, n, rest; next }
    /^ *gpio-[0-9]+ / { n=$1; sub(/gpio-/, "", n); i=index($0, "("); j=0
      for (k=length($0); k>0; k--) if (substr($0,k,1)==")") { j=k; break }
      if (!i || !j) next
      inner=substr($0, i+1, j-i-1); rest=substr($0, j+1); gsub(/^[ \t]+|[ \t]+$/, "", rest); gsub(/[ \t]+/, " ", rest)
      p=index(inner, "|"); owner=(p ? trim(substr(inner, p+1)) : trim(inner)); if (owner=="") owner="-"
      printf "%s/%d\t%s %s\n", chip, n-base, rest, owner }' ;;
  pinmux) sed -nE 's/^pin [0-9]+ \(([^)]*)\): +(.*)$/\1\t\2/p' | awk -F'\t' '$2 !~ /UNCLAIMED/' ;;
  pinconf) sed -nE 's/^pin [0-9]+ \(([^)]*)\): +(.*)$/\1\t\2/p' ;;
  regulator) awk -F: 'NF>=3 { v=$NF; s=$(NF-1); n=$1; for (i=2;i<NF-1;i++) n=n ":" $i; printf "%s\t%s:%s\n", n, s, v }' ;;
  clk) awk '$2 ~ /^[0-9]+$/ && $3 ~ /^[0-9]+$/ && NF>=6 { r=(NF==6 ? $4 : $5); printf "%s\ten=%s prep=%s rate=%s\n", $1, $2, $3, r }' ;;
  dt) awk -F= '$1 !~ /(^|\/)(linux,)?phandle$/ && NF>=1 && $0 != "" { k=$1; v=substr($0, length(k)+2); printf "%s\t%s\n", k, v }' ;;
  dmesg) sed -E -e 's/^(<[0-9]>)?\[[ 0-9.]*\] //' -e 's/^<[0-9]>//' | awk 'NF { printf "%s\t\n", $0 }' ;;
  *) awk '{ i=index($0, "="); if (i>1) printf "%s\t%s\n", substr($0,1,i-1), substr($0,i+1); else if (NF) printf "%s\t\n", $0 }' ;;
  esac | LC_ALL=C sort -t "$(printf '\t')" -k1,1 -s | awk -F'\t' '{ n[$1]++; k=($1 in n && n[$1]>1) ? $1 "#" n[$1] : $1; printf "%s\t%s\n", k, $2 }'
}
kind_of() { case $1 in gpio*) echo gpio;; pinmux*) echo pinmux;; pinconf*) echo pinconf;; regulator*) echo regulator;; clk*) echo clk;; dt*) echo dt;; dmesg*) echo dmesg;; *) echo kv;; esac; }

# ---------- capture ----------
run_android() { # COMMAND -> stdout
  if [ -n "$ADB_SERIAL" ]; then timeout 30 "$ADB" -s "$ADB_SERIAL" shell "$1" </dev/null 2>/dev/null; else timeout 30 "$ADB" shell "$1" </dev/null 2>/dev/null; fi
}
run_ours() { # COMMAND -> stdout, the answer between two markers
  ( sleep "$SETTLE"; printf '%s\n' "echo PC-BEGIN; $1; echo PC-END"; sleep "$CW"; echo exit ) | $OURS_SHELL 2>/dev/null | clean | awk '/^PC-BEGIN$/ { on=1; next } /^PC-END$/ { end=1; on=0 } on { print } END { if (on && !end) print "part-compare: no PC-END (answer may be cut)" > "/dev/stderr" }'
}
capture() { # PART SIDE [OUT]
  part=$1; side=$2; list=$LISTDIR/$part.list
  [ -r "$list" ] || die "no list for part '$part': $list (see README.md: adding a part)"
  case $side in
  android) command -v "$ADB" >/dev/null 2>&1 || die "$ADB not found"
    if [ -z "$ADB_SERIAL" ]; then n=$(timeout 10 "$ADB" devices 2>/dev/null | awk 'NR>1 && $2=="device"' | wc -l); [ "$n" -eq 1 ] || die "adb sees $n devices in state device (set ADB_SERIAL when it is not one)"; fi
    timeout 15 "$ADB" ${ADB_SERIAL:+-s "$ADB_SERIAL"} root >/dev/null 2>&1; timeout 20 "$ADB" ${ADB_SERIAL:+-s "$ADB_SERIAL"} wait-for-device >/dev/null 2>&1; for _ in 1 2 3 4 5; do [ -n "$(run_android "id -u")" ] && break; sleep 1; done
    run=run_android; run_android 'grep -q " debugfs " /proc/mounts || mount -t debugfs none /sys/kernel/debug 2>/dev/null; true' >/dev/null ;;
  ours)
    if [ -z "$OURS_SHELL" ]; then
      toml=${DEVICE_TOML:-$HERE/../../device.toml}
      addr=$(sed -n 's/^addresses *= *\["\([^"]*\)".*/\1/p' "$toml" 2>/dev/null | head -n1); [ -n "$addr" ] || die "no [connection] addresses in $toml (or set OURS_SHELL)"
      if command -v nc >/dev/null 2>&1; then OURS_SHELL="nc -w $((CW+15)) $addr 23"; else OURS_SHELL="timeout $((CW+20)) telnet $addr 23"; fi
    fi
    run=run_ours; run_ours 'grep -q " debugfs " /proc/mounts || mount -t debugfs none /sys/kernel/debug 2>/dev/null; true' >/dev/null ;;
  *) die "side must be android or ours" ;;
  esac
  OUT=${3:-$HOME/lab-material/walleye/compare-$part-$side-$(date +%Y%m%d-%H%M%S)}
  mkdir -p "$OUT" && chmod 700 "$OUT" || die "cannot make $OUT"
  got=0; tab=$(printf '\t')
  { printf 'side=%s\npart=%s\ndate=%s\n' "$side" "$part" "$(date +%Y-%m-%dT%H:%M:%S)"; echo "uname=$($run 'uname -a' | head -n1)"; } | redact > "$OUT/meta.txt"
  while IFS= read -r line || [ -n "$line" ]; do
    case $line in ''|'#'*) continue;; esac
    name=${line%%"$tab"*}; cmd=${line#*"$tab"}
    [ "$name" != "$line" ] && [ -n "$cmd" ] || die "$list: line without NAME<TAB>COMMAND: $line"
    $run "$cmd" | redact > "$OUT/$name.txt.tmp"; mv "$OUT/$name.txt.tmp" "$OUT/$name.txt"
    [ -s "$OUT/$name.txt" ] && got=$((got+1))
  done < "$list"
  [ "$got" -gt 0 ] || die "no command gave an answer on $side (saved $OUT): the transport is not up, not a verdict on the part"
  echo "part-compare: captured $got answers of $part on $side into $OUT"
}

# ---------- diff ----------
diffdir() { # ANDROID_DIR OURS_DIR
  a=$1; o=$2; [ -d "$a" ] && [ -d "$o" ] || die "diff needs two capture directories"
  tab=$(printf '\t'); T=$(mktemp -d) || die "no temp dir"; trap 'rm -rf "$T"' EXIT
  names=$( { ls "$a"; ls "$o"; } 2>/dev/null | grep '\.txt$' | grep -v '^meta\.txt$' | sort -u )
  echo "part-compare diff: android=$a ours=$o"
  for f in $names; do
    n=${f%.txt}; k=$(kind_of "$n")
    echo; echo "== $n ($k)"
    [ -f "$a/$f" ] || { echo "not captured on Android: not known yet"; continue; }
    [ -f "$o/$f" ] || { echo "not captured on ours: not known yet"; continue; }
    if [ ! -s "$a/$f" ] && [ ! -s "$o/$f" ]; then echo "empty on both sides"; continue; fi
    [ -s "$a/$f" ] || echo "(Android's answer is empty: the file or the part is absent there)"
    [ -s "$o/$f" ] || echo "(our answer is empty: the file or the part is absent here)"
    norm "$k" < "$a/$f" > "$T/a"; norm "$k" < "$o/$f" > "$T/o"
    [ -s "$a/$f" ] && [ ! -s "$T/a" ] && echo "(the normaliser matched nothing in Android's answer: unknown format, read $a/$f)"
    [ -s "$o/$f" ] && [ ! -s "$T/o" ] && echo "(the normaliser matched nothing in our answer: unknown format, read $o/$f)"
    awk -F'\t' -v A="$T/a" '
      FILENAME==A { av[$1]=$2; next }
      { ov[$1]=$2 }
      END { for (k in av) { if (!(k in ov)) print "A\t" k "\t" av[k]; else if (av[k]!=ov[k]) print "D\t" k "\tandroid=[" av[k] "] ours=[" ov[k] "]" }
            for (k in ov) if (!(k in av)) print "O\t" k "\t" ov[k] }' "$T/a" "$T/o" | LC_ALL=C sort > "$T/d"
    for g in A O D; do
      case $g in A) t='-- Android has, we lack';; O) t='-- we have, Android lacks';; D) t='-- same key, different value';; esac
      echo "$t ($(grep -c "^$g$tab" "$T/d"))"
      sed -n "s/^$g$tab//p" "$T/d" | sed "s/$tab/  /" | sed 's/^/  /'
    done
  done
}

case $1 in
  capture) [ $# -ge 3 ] || die "usage: part-compare.sh capture PART android|ours [OUT]"; capture "$2" "$3" "$4" ;;
  diff) [ $# -eq 3 ] || die "usage: part-compare.sh diff ANDROID_DIR OURS_DIR"; diffdir "$2" "$3" ;;
  *) die "usage: part-compare.sh capture PART android|ours [OUT] | diff ANDROID_DIR OURS_DIR" ;;
esac
