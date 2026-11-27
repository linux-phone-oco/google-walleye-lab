#!/bin/sh
# Proof of tests/host/kernel-modem-mss.sh, no kernel tree: it passes this repository's fragments and fails each way they can lose a symbol. Every failing case is an edit of the real build/*.config (copied).
# Source of what the symbols need: tests/fixtures/kernel-text/modem-mss-Kconfig.txt.
HERE=$(cd "$(dirname "$0")/../.." && pwd)
C=$HERE/tests/host/kernel-modem-mss.sh
T=$(mktemp -d) || exit 1
trap 'rm -r "$T"' EXIT INT TERM
fresh() { rm -rf "$T/b"; mkdir "$T/b"; cp "$HERE"/build/*.config "$T/b/"; }
want() { # WHAT REASON (a substring the failure must carry)
  out=$(BUILD_DIR=$T/b sh "$C") ; rc=$?
  case "$rc" in 1) ;; *) echo "fail walleye-kernel-modem-mss-selftest $1: wanted $2, got exit $rc: $out"; exit 1 ;; esac
  case "$out" in "fail "*"$2"*) ;; *) echo "fail walleye-kernel-modem-mss-selftest $1: wanted a failure with '$2', got: $out"; exit 1 ;; esac
}
fresh; out=$(BUILD_DIR=$T/b sh "$C"); case "$out" in "skip walleye-kernel-modem-mss host the fragments set it"*) ;; *) echo "fail walleye-kernel-modem-mss-selftest: $out"; exit 1 ;; esac
fresh; sed -i '/^CONFIG_QCOM_Q6V5_MSS=y/d' "$T/b/walleye-modem.config"; want "the modem driver missing" "exactly once"
fresh; sed -i 's/^CONFIG_QCOM_Q6V5_MSS=y/CONFIG_QCOM_Q6V5_MSS=m/' "$T/b/walleye-modem.config"; want "the modem driver =m" "not =y"
fresh; sed -i 's/^CONFIG_QCOM_RMTFS_MEM=y/CONFIG_QCOM_RMTFS_MEM=m/' "$T/b/walleye-modem.config"; want "rmtfs memory =m" "not =y"
fresh; echo "CONFIG_QCOM_RMTFS_MEM=y" >>"$T/b/walleye-modem.config"; want "rmtfs memory set twice" "exactly once"
fresh; sed -i 's/^CONFIG_QCOM_SYSMON=y/CONFIG_QCOM_SYSMON=m/' "$T/b/walleye.config"; want "SYSMON =m, which would make the driver =m" "which the modem needs"
fresh; echo "# CONFIG_QCOM_Q6V5_MSS is not set" >>"$T/b/walleye-diag.config"; want "another fragment takes it back" "undoes it"
fresh; rm "$T/b/walleye-modem.config"; want "no modem fragment" "no "
fresh; echo "CONFIG_QCOM_SYSMON=m" >>"$T/b/walleye-modem.config"; want "the modem fragment takes SYSMON back" "undoes it"
fresh; echo "# CONFIG_QCOM_Q6V5_MSS is not set" >>"$T/b/walleye-modem.config"; want "the modem fragment says not set beside =y" "undoes it"
fresh; sed -i "/^CONFIG_QRTR_SMD=y/d" "$T/b/walleye.config"; want "QRTR_SMD missing" "which the modem needs"
fresh; sed -i "/^CONFIG_RPMSG_QCOM_GLINK_SMEM=y/d" "$T/b/walleye.config"; want "GLINK_SMEM missing" "which the modem needs"
fresh; echo "CONFIG_QCOM_RMTFS_MEM=m" >>"$T/b/walleye-diag.config"; want "another fragment sets rmtfs memory =m" "undoes it"
echo "pass walleye-kernel-modem-mss-selftest the fragments pass and each of eleven edits fails"
