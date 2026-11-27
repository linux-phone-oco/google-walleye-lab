#!/bin/sh
# Proof of tests/host/kernel-modem-ipa.sh, no kernel tree: it passes this repository's fragments and fails each way they can lose the symbol. Every failing case is an edit of the real build/*.config (copied).
# Source of what the symbol needs: tests/fixtures/kernel-text/ipa-Kconfig.txt.
HERE=$(cd "$(dirname "$0")/../.." && pwd)
C=$HERE/tests/host/kernel-modem-ipa.sh
T=$(mktemp -d) || exit 1
trap 'rm -r "$T"' EXIT INT TERM
fresh() { rm -rf "$T/b"; mkdir "$T/b"; cp "$HERE"/build/*.config "$T/b/"; }
want() { out=$(BUILD_DIR=$T/b sh "$C"); rc=$?
  [ "$rc" = 1 ] || { echo "fail walleye-kernel-modem-ipa-selftest $1: wanted $2, got exit $rc: $out"; exit 1; }
  case "$out" in "fail "*"$2"*) ;; *) echo "fail walleye-kernel-modem-ipa-selftest $1: wanted a failure with '$2', got: $out"; exit 1 ;; esac
}
fresh; out=$(BUILD_DIR=$T/b sh "$C"); case "$out" in "skip walleye-kernel-modem-ipa host the fragment sets it"*) ;; *) echo "fail walleye-kernel-modem-ipa-selftest: $out"; exit 1 ;; esac
fresh; sed -i '/^CONFIG_QCOM_IPA=y/d' "$T/b/walleye-modem.config"; want "ipa missing" "exactly once"
fresh; sed -i 's/^CONFIG_QCOM_IPA=y/CONFIG_QCOM_IPA=m/' "$T/b/walleye-modem.config"; want "ipa =m" "not =y"
fresh; echo "CONFIG_QCOM_IPA=y" >>"$T/b/walleye-modem.config"; want "ipa set twice" "exactly once"
fresh; sed -i '/^CONFIG_QCOM_Q6V5_MSS=y/d' "$T/b/walleye-modem.config"; want "the modem driver missing" "whose type"
fresh; echo "# CONFIG_QCOM_IPA is not set" >>"$T/b/walleye-diag.config"; want "another fragment takes it back" "undoes it"
fresh; echo "CONFIG_QCOM_IPA=m" >>"$T/b/walleye-display.config"; want "another fragment sets it =m" "undoes it"
fresh; rm "$T/b/walleye-modem.config"; want "no modem fragment" "no "
echo "pass walleye-kernel-modem-ipa-selftest the fragments pass and each of seven edits fails"
