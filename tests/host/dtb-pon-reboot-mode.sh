#!/bin/sh
# walleye-dtb-pon-reboot-mode, run on the lab host (item walleye-install-stage2-init-booted-slot, second part): can our kernel ask the bootloader for itself on the way down?
# Passes only when the dtb's PMIC pon node (qcom,pm8998-pon, the pmic whose child is the power key) is not disabled and holds mode-bootloader = <0x2> (the value the bootloader reads from the PON register to stay in fastboot), and the
# kernel's config builds the driver that turns that into `reboot bootloader` (CONFIG_POWER_RESET_QCOM_PON=y, which selects REBOOT_MODE). Whether the register value reaches the bootloader is NOT shown here: that is the keeper's try
# (item walleye-reboot-bootloader-from-linux-try). Prints one line `pass|fail|skip walleye-dtb-pon-reboot-mode kernel DETAIL`; exit 0 only on pass. Read-only.
# Environment: DTB (the dtb to read; default the UFS root display image's dtb), CONFIG (the kernel config; default build/walleye.config of this repository).
HERE=$(cd "$(dirname "$0")/../.." && pwd)
DTB=${DTB:-$HOME/Projects/walleye-build/NOT-TO-BOOT-ufs-on-images-ufs-root-display/msm8998-google-walleye-ufs-root-display.dtb}
CONFIG=${CONFIG:-$HERE/build/walleye.config}
say() { echo "$1 walleye-dtb-pon-reboot-mode kernel $2"; [ "$1" = pass ]; exit $?; }
command -v fdtget >/dev/null || say skip "fdtget (dtc) not installed: not known yet"
[ -r "$DTB" ] || say skip "need DTB=$DTB readable: not known yet"
[ -r "$CONFIG" ] || say skip "need CONFIG=$CONFIG readable: not known yet"
N=$(basename "$DTB")
SPMI=$(fdtget -l "$DTB" /soc@0 | grep '^spmi@' | head -n1)
[ -n "$SPMI" ] || say fail "$N: no spmi bus under /soc@0"
P=/soc@0/$SPMI/pmic@0/pon@800
[ "$(fdtget "$DTB" "$P" compatible 2>/dev/null)" = qcom,pm8998-pon ] || say fail "$N: $P is not a qcom,pm8998-pon"
S=$(fdtget "$DTB" "$P" status 2>/dev/null); case $S in ""|okay|ok) ;; *) say fail "$N: $P status is $S" ;; esac
[ "$(fdtget -t u "$DTB" "$P" mode-bootloader 2>/dev/null)" = 2 ] || say fail "$N: $P has no mode-bootloader = <2>: the kernel cannot ask the bootloader for itself"
grep -qx 'CONFIG_POWER_RESET_QCOM_PON=y' "$CONFIG" || say fail "$CONFIG does not build the PON reboot-mode driver (CONFIG_POWER_RESET_QCOM_PON=y)"
say pass "$N: $P (qcom,pm8998-pon) holds mode-bootloader = <2> and the kernel config builds its driver; that the register value reaches the bootloader is not measured here"
