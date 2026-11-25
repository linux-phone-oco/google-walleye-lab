#!/bin/sh
# walleye-slpi-sensor-service-selftest: runs tests/health/base/slpi-sensor-service.sh on the text the phone printed at the SLPI boot of 2026-11-25
# (tests/fixtures/phone-text/slpi-boot-qrtr-read.cap, source research/walleye-slpi-boot-qrtr/read.txt): the remoteproc part becomes a sysfs tree, the qrtr part
# the output of a stand-in qrtr-lookup; nothing else is typed. Needs the suite's lib.sh (HEALTH_LIB, or lab-tools' health/suite/lib.sh next to this repository): without it, skip.
# Prints one line, `pass|fail|skip walleye-slpi-sensor-service-selftest host DETAIL`; exit 0 only on pass.
HERE=$(cd "$(dirname "$0")/../.." && pwd)
CAP=$HERE/tests/fixtures/phone-text/slpi-boot-qrtr-read.cap
S=$HERE/tests/health/base/slpi-sensor-service.sh
say() { echo "$1 walleye-slpi-sensor-service-selftest host $2"; [ "$1" = pass ]; exit $?; }
LIB=${HEALTH_LIB:-}
[ -n "$LIB" ] || for c in "$HERE"/../lab-tools/health/suite/lib.sh /usr/lib/devlab/health/lib.sh; do [ -r "$c" ] && LIB=$c && break; done
[ -r "$LIB" ] || say skip "no health suite lib.sh (give HEALTH_LIB): not known yet"
T=$(mktemp -d) || say fail "no temp dir"; trap 'rm -rf "$T"' EXIT INT TERM
part() { awk -v p="---$1" '/^---/{f=0} f{print} $0==p{f=1}' "$CAP"; }
tree() { # DIR: a remoteproc sysfs tree from the capture's `NAME STATE` lines
  n=0; mkdir -p "$1"
  part remoteproc | while read -r name state; do mkdir "$1/remoteproc$n"; echo "$name" >"$1/remoteproc$n/name"; echo "$state" >"$1/remoteproc$n/state"; n=$((n+1)); done; }
sed '/^\. \/etc\/lpoco-health/d' "$LIB" >"$T/lib.sh"; LIB=$T/lib.sh  # the suite reads /etc/lpoco-health/health.env, absent here: a failed `.` ends a POSIX shell
tree "$T/rp"
mkdir "$T/bin"; part qrtr >"$T/qrtr.txt"
printf '#!/bin/sh\ncat "%s"\n' "$T/qrtr.txt" >"$T/bin/qrtr-lookup"; chmod +x "$T/bin/qrtr-lookup"
run() { HEALTH_LIB=$LIB REMOTEPROC_ROOT=$1 PATH="$T/bin:$PATH" sh "$S" 2>&1; }
line() { printf '%s\n' "$1" | grep "^[a-z]* $2 "; }
out=$(run "$T/rp")
case $(line "$out" slpi-running) in "pass slpi-running "*) ;; *) say fail "the captured running slpi did not pass slpi-running: $out" ;; esac
# 23 services are listed and none is named sensor (ids 349-362 print <unknown>): that is not known yet, never a pass and never a fail
case $(line "$out" sensor-service) in "skip sensor-service "*) ;; *) say fail "unnamed services were not a skip: $out" ;; esac
# the same capture with the slpi offline: slpi-running fails
for d in "$T"/rp/*; do [ "$(cat "$d/name")" = slpi ] && echo offline >"$d/state"; done
out=$(run "$T/rp"); case $(line "$out" slpi-running) in "fail slpi-running "*) ;; *) say fail "an offline slpi did not fail: $out" ;; esac
# a table with a service named sensor passes (the name is added to one row of the capture's own table)
awk '!d && /<unknown>/{sub(/<unknown>/,"Sensor service"); d=1} {print}' "$T/qrtr.txt" >"$T/q2"; mv "$T/q2" "$T/qrtr.txt"
out=$(run "$T/rp"); case $(line "$out" sensor-service) in "pass sensor-service "*) ;; *) say fail "a named sensor service did not pass: $out" ;; esac
# no table at all: fail
: >"$T/qrtr.txt"; out=$(run "$T/rp"); case $(line "$out" sensor-service) in "fail sensor-service "*) ;; *) say fail "an empty table did not fail: $out" ;; esac
say pass "the captured SLPI boot read passes slpi-running and gives sensor-service skip (23 unnamed services); offline, a named service and an empty table each judged"
