#!/bin/sh
# walleye-bootchain-readback-check, run on the lab host: is DIR a complete read-back of the Pixel 2's boot chain?
# DIR holds, for every name in bootchain-parts.list, NAME.img of exactly the size the phone measured for it (ls -l of /dev/block/by-name joined with
# /proc/partitions, both kept in DIR as partition-table.txt and proc-partitions.txt; blocks are 1024 bytes), mode 600 in a DIR of mode 700, and a
# line in sha256sums that matches the file. No `missing` file may exist (the reader writes one for a name the phone does not list).
# Given a second DIR2 it must pass the same, and its sha256sums must equal DIR's: a second read of the same bytes.
# XBL_SECTORS (optional, e.g. 8104) and SECTOR_BYTES (default 512): xbl_a and xbl_b must then also be XBL_SECTORS sectors; unset = that
# measure is not known yet, and the phone's own size is the one that binds.
# Prints `problem ...` lines, then one line `pass|fail $NAME host DETAIL`; exit 0 only on pass. Reads files only.
#
#   tests/host/bootchain-readback-check.sh DIR [DIR2]
HERE=$(cd "$(dirname "$0")" && pwd)
LIST=${LIST:-$HERE/bootchain-parts.list}
NAME=${NAME:-walleye-bootchain-readback-check}
SB=${SECTOR_BYTES:-512}
bad=0
problem() { echo "problem $1"; bad=$((bad + 1)); }
verdict() { if [ "$bad" -eq 0 ]; then echo "pass $NAME host $1"; exit 0; fi
  echo "fail $NAME host $bad problem(s)"; exit 1; }
[ $# -ge 1 ] && [ $# -le 2 ] || { echo "fail $NAME host usage: bootchain-readback-check.sh DIR [DIR2]"; exit 1; }
[ -r "$LIST" ] || { echo "fail $NAME host $LIST is missing"; exit 1; }
case "$XBL_SECTORS$SB" in *[!0-9]*) echo "fail $NAME host XBL_SECTORS and SECTOR_BYTES must be whole numbers"; exit 1;; esac

check_one() { # DIR: appends problems
  d=$1
  [ -d "$d" ] || { problem "$d is not a directory"; return; }
  [ "$(stat -c %a "$d")" = 700 ] || problem "$d mode is $(stat -c %a "$d"), not 700"
  [ ! -e "$d/missing" ] || problem "$d/missing exists: the phone did not list $(tr '\n' ' ' < "$d/missing")"
  for f in partition-table.txt proc-partitions.txt sha256sums; do
    [ -s "$d/$f" ] || problem "$d/$f is missing or empty"
  done
  for n in $(cat "$LIST"); do
    f=$d/$n.img
    if [ ! -f "$f" ]; then problem "$n.img is missing"; continue; fi
    [ "$(stat -c %a "$f")" = 600 ] || problem "$n.img mode is $(stat -c %a "$f"), not 600"
    have=$(stat -c %s "$f")
    # measured size: the by-name link's device, its 1 KiB blocks in /proc/partitions
    dev=$(awk -v n="$n" '{for (i = 1; i < NF; i++) if ($i == n && $(i+1) == "->") {t = $(i+2); sub(/.*\//, "", t); print t; exit}}' "$d/partition-table.txt" 2>/dev/null)
    blocks=
    [ -n "$dev" ] && blocks=$(awk -v d="$dev" '$4 == d {print $3; exit}' "$d/proc-partitions.txt" 2>/dev/null)
    if [ -z "$blocks" ]; then problem "$n: no measured size in partition-table.txt and proc-partitions.txt"
    elif [ "$have" -ne $((blocks * 1024)) ]; then problem "$n.img is $have bytes, the phone measured $((blocks * 1024))"; fi
    case $n in xbl_a|xbl_b)
      if [ -n "$XBL_SECTORS" ] && [ "$have" -ne $((XBL_SECTORS * SB)) ]; then
        problem "$n.img is $have bytes, not $XBL_SECTORS sectors of $SB"; fi;;
    esac
    want=$(awk -v f="$n.img" '$2 == f {print $1; exit}' "$d/sha256sums" 2>/dev/null)
    sum=$(sha256sum < "$f" | cut -d' ' -f1)
    if [ -z "$want" ]; then problem "$n.img has no line in sha256sums"
    elif [ "$want" != "$sum" ]; then problem "$n.img sha256 is $sum, sha256sums says $want"; fi
  done
}

if [ $# -eq 2 ] && [ "$(cd "$1" 2>/dev/null && pwd -P)" = "$(cd "$2" 2>/dev/null && pwd -P)" ]; then problem "DIR and DIR2 are the same directory: no second read"; fi
check_one "$1"
if [ $# -eq 2 ]; then
  check_one "$2"
  if [ -f "$1/sha256sums" ] && [ -f "$2/sha256sums" ]; then
    [ "$(sort "$1/sha256sums")" = "$(sort "$2/sha256sums")" ] || problem "the two sha256sums differ: the second read is not the same bytes"
  fi
fi
verdict "every partition of the list read at its measured size with a matching sha256$( [ $# -eq 2 ] && echo ', twice, the same bytes')"
