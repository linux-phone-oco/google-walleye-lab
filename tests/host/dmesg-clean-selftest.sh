#!/bin/sh
# Proves tests/host/dmesg-clean-check.sh on text our phone printed (tests/fixtures/phone-text/dmesg-clean-*.cap, copies of the z7, z6 and c2 try reads), never touching the phone:
# the clean read passes, each fault alone fails, missing files skip. Faults no phone has printed (-110, phandle) are in dmesg-clean-typed-selftest.sh.
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
F="$HERE/../fixtures/phone-text"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
want() { # NAME EXPECTED_WORD DIR
  got=$(sh "$HERE/dmesg-clean-check.sh" "$3" | cut -d' ' -f1 || true)
  if [ "$got" = "$2" ]; then echo "ok   $1"; else echo "FAIL $1: got $got, want $2"; total=$((total+1)); fi
}
mk() { # NAME DMESG_TRY DEFERRED_TRY - a read directory of two captured files
  mkdir -p "$T/$1"; cp "$F/dmesg-clean-$2-dmesg.cap" "$T/$1/dmesg.txt"; cp "$F/dmesg-clean-$3-devices-deferred.cap" "$T/$1/devices-deferred.txt"; }
mk clean z7-try1 z7-try1;    want "the z7 read (clean dmesg, devices_deferred unreadable once) passes" pass "$T/clean"
mk both z6-try1 z6-try1;     want "the z6 read (smmu deferred, gpu_gx stuck) fails" fail "$T/both"
mk defer z7-try1 z6-try1;    want "z6's deferred smmu with a clean dmesg fails" fail "$T/defer"
mk gpux z6-try1 z7-try1;     want "z6's gpu_gx WARN with nothing deferred fails" fail "$T/gpux"
mk charger c2-try1 c2-try1;  want "the c2 read (a charger waiting for its supplier) fails" fail "$T/charger"
mk nodebugfs z7-try1 z7-try1; cat "$F/dmesg-clean-z7-try1-devices-deferred.cap" "$F/dmesg-clean-z7-try1-devices-deferred.cap" > "$T/nodebugfs/devices-deferred.txt"
want "debugfs never mounted (the can't-open line twice) skips" skip "$T/nodebugfs"
mkdir "$T/empty"; want "no files skip" skip "$T/empty"
[ $total -eq 0 ] || { echo "dmesg-clean-selftest: $total failed" >&2; exit 1; }
echo "pass dmesg-clean-selftest"
