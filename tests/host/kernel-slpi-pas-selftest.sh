#!/bin/sh
# Proof of tests/host/kernel-slpi-pas.sh, no kernel tree: it passes this repository's fragments and fails each way they can lose the symbol.
# Every failing case is an edit of the real build/*.config (copied), as the banner selftests edit captured text. Source of what the symbol needs: tests/fixtures/kernel-text/pd-mapper-Kconfig.txt.
HERE=$(cd "$(dirname "$0")/../.." && pwd)
C=$HERE/tests/host/kernel-slpi-pas.sh
T=$(mktemp -d) || exit 1
trap 'rm -r "$T"' EXIT INT TERM
fresh() { rm -rf "$T/b"; mkdir "$T/b"; cp "$HERE"/build/*.config "$T/b/"; }
want() { # WHAT EXPECTED-WORD
  out=$(BUILD_DIR=$T/b sh "$C") ; rc=$?
  case "$2:$rc" in pass:0|fail:1|skip:1) ;; *) echo "fail walleye-kernel-slpi-pas-selftest $1: wanted $2, got exit $rc: $out"; exit 1 ;; esac
  case "$out" in "$2 "*|"skip "*) ;; *) echo "fail walleye-kernel-slpi-pas-selftest $1: wanted $2, got: $out"; exit 1 ;; esac
}
fresh; out=$(BUILD_DIR=$T/b sh "$C"); case "$out" in "skip walleye-kernel-slpi-pas host the fragments set it"*) ;; *) echo "fail walleye-kernel-slpi-pas-selftest: $out"; exit 1 ;; esac
fresh; sed -i '/^CONFIG_QCOM_Q6V5_PAS=y/d' "$T/b/walleye.config"; want "the symbol missing" fail
fresh; sed -i 's/^CONFIG_QCOM_Q6V5_PAS=y/CONFIG_QCOM_Q6V5_PAS=m/' "$T/b/walleye.config"; want "the symbol =m" fail
fresh; sed -i 's/^CONFIG_QRTR=y/CONFIG_QRTR=m/' "$T/b/walleye.config"; want "QRTR =m, which would make the symbol =m" fail
fresh; echo "# CONFIG_QCOM_Q6V5_PAS is not set" >>"$T/b/walleye-diag.config"; want "another fragment takes it back" fail
fresh; echo "CONFIG_QCOM_Q6V5_PAS=y" >>"$T/b/walleye.config"; want "the symbol set twice" fail
echo "pass walleye-kernel-slpi-pas-selftest the fragments pass and each of five edits fails"
