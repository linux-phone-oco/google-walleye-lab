#!/bin/sh
# walleye's own check: the display came up at boot with no fault in the kernel log - no SMMU context fault, no "rcg didn't update"
# on the pixel clock, no DSI command timeout, no failed autorefresh disable. Fails (or shows unknown with no dmesg) until
# walleye-display-dts-any-order works; B2 (modules loaded by the init at 2 s) shows all four, a load at 66 s none
# (research walleye-display-b2-dsi-clock-warning, decision 2026-11-19-walleye-display-early-load-splash-handover).
. /usr/lib/devlab/health/lib.sh
log=$(klog 2>/dev/null)
if [ -z "$log" ]; then
	say display-boot-clean display skip "no kernel log to read (not known yet)"
	exit 0
fi
n_smmu=$(printf '%s\n' "$log" | grep -c 'arm-smmu.*Unhandled context fault')
n_rcg=$(printf '%s\n' "$log" | grep -c "rcg didn't update its configuration")
n_dsi=$(printf '%s\n' "$log" | grep -c 'dsi_cmds2buf_tx: cmd dma tx failed')
n_ar=$(printf '%s\n' "$log" | grep -c 'disable autorefresh failed')
ok=
[ "$n_smmu$n_rcg$n_dsi$n_ar" = 0000 ] && ok=true
check display-boot-clean display "$ok" "smmu faults $n_smmu, pixel rcg warnings $n_rcg, dsi dma timeouts $n_dsi, autorefresh failures $n_ar"
