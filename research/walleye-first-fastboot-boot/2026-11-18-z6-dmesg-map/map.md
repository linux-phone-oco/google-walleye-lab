# z6 dmesg map (590 lines, Linux 7.3.0-dirty, saved in ../2026-11-19-z6-try1/read/dmesg.txt)

Read from the text with grep for warn/fail/defer/error/timeout/-N codes; cause as far as our dts, walleye.config
and the linux-topics source (drivers/clk/qcom/gpucc-msm8998.c, gdsc.c, arch/arm64/boot/dts/qcom/msm8998.dtsi) show.
Not measured here: the vendor gfx-rail wiring (why GX stays off) - marked "not known yet".

## What works, by lines
- SPMI: `spmi_pmic_arb 800f000.spmi: PMIC arbiter version v3`; /sys/bus/spmi: 0-00..0-03; pm8998 adc/gpio/pon/rtc/temp-alarm and pmi8998 labibb/adc/gpio bound as platform devices.
- RPM: remoteproc:glink-edge rpm-requests clock-controller, power-controller, regulators-0/-1; 12 regulators in class/regulator.
- USB: a800000.usb udc, gadget bound (banner step 8) at 1.17 s.
- Watchdog 17817000 (kpss-wdt, petted by the banner), ramoops a1100000 (pstore registered), gpio_keys input0.
- smmu 1680000 / 16c0000 / cd00000 probe fine; clocks (gcc, gpucc, mmcc nodes) present.

## Groups of warnings

### G1 clocks/power: gpu_gx GDSC "status stuck at 'off'" (6 WARNINGs: 1.0095, 1.0565, 1.0686, 1.0866, 1.1787, 11.24 s - one fault, repeated per probe retry)
Trace: platform_probe -> dev_pm_domain_attach -> genpd_dev_pm_attach -> genpd_power_on -> gdsc_enable ->
gdsc_toggle_logic -> WARN (gdsc.c:192, status poll timed out, x0 = -110). Who attaches: 5040000.iommu (adreno_smmu in
msm8998.dtsi: `power-domains = <&gpucc GPU_GX_GDSC>`; comment says GX is enabled there on purpose). GX's parent is gpu_cx
(rpm-less VOTABLE gdsc); GX needs its rail and clamps. Cause: the GX collapse switch is powered from the GFX rail, which
nothing in our dts turns on (vendor: pm8005 / CPR-managed vdd_gfx); not known yet which rail on walleye.
Effect: 5040000.iommu defers for ever ("deferred probe pending: (reason unknown)", line 584 - the "-110 smmu timeout"
of the old backlog text is this, not a bus timeout), and gcc/gpucc sync_state() stay pending due to 5040000.iommu (lines 585-586).
Fix (smallest): our board dts sets `&adreno_smmu { status = "disabled"; }` (and gpu/gmu nodes, already disabled) until GPU work: nothing
else uses it; gives no WARN and no deferral. Alternative later: a gfx rail node + `gpu_gx` supply. Rank: dmesg-clean, first.

### G2 serial: msm_serial port #1 "probe ... with error -2" (line 289), `qcom_cc_clk_hw_get: invalid index 3351654288` (288)
Port #0 c1b0000.serial (our debug UART, blsp2_uart1) is fine. Port #1 is serial@c16f000, a node of the vendor dtbo overlay
merged by the bootloader (androidboot.dtbo_idx=3): its `clocks = <&clock_gcc ID ...>` carry the vendor's clock ids; z3 already
pointed /__symbols__/clock_gcc at the real gcc (removes the old "could not find phandle"), but the vendor id (0xC7..., not an index of
mainline gcc-msm8998) is not valid in mainline gcc -> -ENOENT. Same overlay nodes keep gcc sync_state() pending:
c179000.i2c, soc@0:bt_wcn3990, c16f000.serial, soc@0:mnh-sm (lines 587-591). Cause: vendor overlay's clock ids; we cannot edit the overlay
(dtbo partition not ours). Fix choices: (a) accept, boot with `fw_devlink=permissive` is not enough (sync_state is by design) - harmless with clk_ignore_unused,
note it as known noise; (b) after walleye-dtbo-overlay-id is answered, boot without the overlay (dtb complete on its own) -> the nodes vanish; (c) a dtb carrying a
translation is not possible. Recommend (a) now, (b) as its own item.

### G3 psci: `[Firmware Bug]: failed to set PC mode: -3` (line 34)
Platform-coordinated mode request not supported by this firmware (-3 = NOT_SUPPORTED) after OSI mode supported; the kernel falls back. Harmless; known on msm8998
mainline boards. Fix: none needed (do not use cpuidle OSI/domain-idle nodes on walleye until deep-sleep research); note only.

### G4 early boot noise
`[Firmware Bug]: Kernel image misaligned at boot` (line 7): the bootloader loads Image at an address not 2 MiB aligned; our boot image header kernel offset. Fix: build-bootimg kernel_offset 0x8000 vs 0x80000 check - low rank, harmless.
`efi: UEFI not found`, `DMI: not present` (147): normal, no fix. `qcom_scm: SHM Bridge not supported`, `qseecom: untested machine, skipping`: normal (no TZ apps wanted).

### G5 SPMI overlay nodes: `spmi spmi-0: node .../qcom,pm8998@0 err (-22) does not have 'reg' property` and pmi8998@3 (lines 265-266)
The vendor overlay's two reg-less PMIC nodes; ours (pm8998.dtsi / pmi8998.dtsi) bind (0-00..0-03). pmic-dmesg-check notes them. Same fate as G2 (b).

### G6 absent, not warned (read from the platform list and config)
- UFS: no ufshc/ufsphy node in the platform list, no UFS driver in walleye.config (no SCSI_UFS_QCOM, no QMP UFS phy): nothing probes, nothing is read.
- Charger: main's dts has &pmi8998_charger and walleye.config has CHARGER_QCOM_SMB2, but z6's dtb is z3's (no charger node): no smb2 line, class/power_supply empty. c1 (walleye-try-c1) carries the node; it must be rebuilt on the z6 init.
- Display (no drm), touch (no input beyond gpio_keys), sensors, fuel gauge, remoteprocs (smp2p lines only, no modem/adsp start): not in the dts yet.
- debugfs not mounted -> devices_deferred unreadable (the keeper's reader must `mount -t debugfs none /sys/kernel/debug`).

## Counting
The 14 pattern lines are one fault repeated (G1, six WARNs with their Tainted/Workqueue lines) plus G2 (2 lines), G3, G4, G5 (2 lines). Distinct problems: five. Nothing in PMIC/regulators, USB, watchdog or ramoops fails.
