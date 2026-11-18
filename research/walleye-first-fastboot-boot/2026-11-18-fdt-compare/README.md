Three trees compared: OUR dtb (`out/msm8998-google-walleye.dtb`, dead), the alive taimen dtb (pk5 = pk1.dtb, 143 KB) and its walleye-property twin `wpa` (alive, 60/60 s),
and the phone's RUNNING fdt (`vfa` try 1, material `~/Projects/walleye-build/material/vf/2026-11-18-vfx-vfa-try1/fdt.dtb`, 466 KB). Clean room: only hardware facts of the GPL trees and of what ran.
Files here: `nodes-only-wpa.txt` (288 nodes wpa has, ours lacks), `nodes-only-ours.txt` (22), `props-only-wpa.txt` (126 properties of nodes both have, only wpa has), `props-only-ours.txt` (14),
`props-differing.txt` (45 lines: path|property, wpa's value, ours; phandles resolved to node paths by `build/dts-norm.awk`), `SHA256SUMS` (the x-images).

## Structure

| | ours | alive taimen (pk1/pk5) | wpa | running fdt (vendor 4.4 tree) |
|---|---|---|---|---|
| dtb size | 83 821 | 143 297 | 132 588 | 466 569 |
| nodes / phandles | 581 / 399 | 953 / 606 | 846 nodes | 2284 / 1097 |
| root compatible | google,walleye google,wahoo qcom,msm8998 | google,taimen google,wahoo qcom,msm8998 | + qcom,msm8998-mtp | qcom,msm8998-mtp qcom,msm8998 qcom,mtp |
| `qcom,msm-id` | <0x124 0x20001> | same | same | same; no board-id, no pmic-id anywhere |
| /memory | memory@80000000, reg <0 0x80000000 0 0> (size 0) | same | same | `memory` (no unit), the bootloader fills it: 0x80000000+0x80000000, 0x100000000+0x7e4c0000 (3.97 GiB) |
| /chosen | EMPTY node | #address-cells #size-cells ranges | same | the bootloader adds bootargs (the image's command line appended to the stock tree's), linux,initrd-start/end, kaslr-seed 0, stdout-path serial0, a `radio` subnode (htc smem facts) |
| psci, timer | identical in all four (arm,psci-1.0 smc; the four timer interrupts) | | | psci identical |

What the bootloader ADDS to a tree it hands the kernel (present in the running fdt, absent in the stock source trees): /chosen bootargs, initrd range, kaslr-seed, stdout-path; /memory reg filled
(our size-0 reg is the placeholder form); `bootloader_log` (htc,bldr_log, 0xa1300000 and 0xa1380000, 512 KiB each: the ABL's own log), `calibration_data` (audio, wifi, bt, gyro blobs; device-specific, not recorded here),
`ramoops` (0xa1100000, 2 MiB, plus `alt_ramoops_region` 0xa3800000 and `ramoops_meta_region` 0xaffff000), a rewritten reserved-memory (below). Not changed by it: msm-id, psci, timer.

## /reserved-memory, three ways (address, size)

| region | running fdt | alive taimen | ours |
|---|---|---|---|
| removed_regions | 0x85800000 0x3700000 | 85800000 6M, 85e00000 1M, smem 86000000 2M, 86200000 0x2d00000, 88f00000 2M | the same set |
| spss | 0x8ab00000 0x700000 | same | same |
| adsp / modem | 0x8b200000 0x1a00000 / 0x8cc00000 0x7800000 | same | same |
| mba | 0x94500000 0x200000 (named 94100000: the bootloader moved it) | 94900000 0x200000 | 94900000 0x200000 |
| video | 0x94700000 0x500000 (named 93c00000) | 94400000 0x500000 | 94400000 0x500000 |
| slpi | 0x94c00000 0xf00000 (named 94300000) | memory@95800000 0xf00000 | memory@94b00000 0x700000 |
| ipa/gpu | 0x95b00000 0x100000 (named 95200000) | 95200000 64K, 95210000 20K, 95600000, 95700000 1M | same four |
| easel | 0x95c00000 0x4000000 | absent | absent |
| splash | 0x9d400000 0x2400000 (named 9d600000: the bootloader's framebuffer) | absent | absent |
| htc_reserve (bootloader log) | 0xa1300000 0x500000 | absent | absent |
| ramoops | 0xa1100000 0x200000 | b0000000 | **a1810000** |
| htc_rtel | 0xa1800000 0x2000000 | absent | absent: **our ramoops@a1810000 lies inside it** |
| dynamic | cma 32M, secure 92M, qseecom 20M, adsp 8M, sp 8M (alloc below 4 GiB) | | |

Facts: the running tree's pil regions carry the names of the ORIGINAL layout but the bootloader's current addresses (a fixup), so a source tree's pil addresses are not what the running phone uses;
ours and the alive tree differ from each other and from the running fdt in the pil block. Our ramoops sits inside the bootloader's 32 MiB runtime-event region; the pk5l2/T1/T3 images already moved it to 0xa1100000.

## What the alive trees (taimen dtb, wpa) and the running fdt have, and ours lacks, in order of suspicion

1. `restart@10ac000` (compatible `qcom,pshold`, regs pshold-base and tcsr-boot-misc-detect) and the apps watchdog `watchdog@17817000` (alive: qcom,apss-wdt-msm8998 / kpss-wdt; running: qcom,msm-watchdog, bark 11 s, pet 10 s): in BOTH alive trees and in the running fdt, not in ours. A dead image returns at ~75 s, a bite-class time; an alive one reboots at once. Image x4.
2. `pinctrl@3400000 gpio-reserved-ranges = <0 4>, <81 4>`: both alive taimen and porthole's pk2 have it, ours has none (the running vendor tree has no such property: the vendor kernel never touches those pins). Image x1.
3. /chosen empty and /reserved-memory as above (ramoops in the bootloader's rtel region; the modem remoteproc memory-region pointing at a different slpi block). Image x2.
4. Disabled hardware the bootloader leaves running: display-subsystem + dsi + phy, gpu, ufshc + its phy, three remoteprocs, serial@c171000, wifi are `okay` in both alive trees, `disabled` in ours. Image x6.
5. Seven `interconnect@` providers (1000000 1500000 1620000 1660000 1700000 1740000 17900000) and the `interconnects` of display, gpu, ufshc, usb, video-codec. Image x5.
6. CPU scaling: cpu-gold/silver/cpr opp tables, cpufreq-hw@17914800, two power-managers, power-controller@179c8000, two lmh, per-cpu operating-points-v2/power-domains/qcom,freq-domain, `system-pc` idle state, capacity-dmips-mhz (alive 0x209/0x400, ours 0x400/0x600). Image x3.
7. The rest: imem@146bf000, pmu@1008000 (in x4), qfprom cells, pmic children, thermal extras, apr services, camss, cci, panel, regulator node names (alive `pm8998-regulators`, ours `regulators-0`), sram@778000, bias-pull-up vs bias-disable in the i2c sleep states, gpu opp-levels. Image x7.

## The images (host only; `build/build-fdt-bisect.sh WPA_BOOT_IMG OURS_DTB OUT_DIR`, 114 checks, rebuild byte-identical; on the lab: `~/Projects/walleye-build/images-x/`)

x1..x7 = wpa with ONE disjoint group made equal to ours (checked pairwise: no two groups change the same line, but a property whose target another group removed: x3/x7 one, x4/x5 one), `xa` = all seven; kernel, command line, ramdisk (reboot at once) and AVB footer are wpa's. A node the stock dtbo overlay targets (ramoops@b0000000, cci, three pmics) is emptied, not removed, so every image still merges the overlay
(the check). After xa the normalized dtb differs from ours in exactly the 82 lines of `xa-left-over-ours.txt` (checked by hash): the ours-only rpm regulators-0/-1 blocks and overlay stubs, the emptied overlay targets, the supplies of consumers of removed regulators, renumbered phandle lists (c8c0000 clocks, pcie iommu-map, rpm-msg-ram). If xa is alive and ours dead, the cause is among those lines: build the next image from them.
Expected returns (the keeper's rule): alive <= 66 s, dead >= 69 s. Reading: if one single image x_n is dead, its group is a cause; if only xa is dead, groups interact (bisect xa minus one group); if xa is alive, the dtb is not the fault and the kernel config or the command line is.
Order to try: x4, x1, x2, x6, x5, x3, x7, xa, twice each.

## Proposal for dt/msm8998-google-walleye.dts: USB peripheral, read from the running fdt

The running fdt's `ssusb@a800000` (qcom,dwc-usb3-msm, downstream glue) with `dwc3@a800000` (snps,dwc3) and `qusb@c012000`, `ssphy@c010000`:
- glue regs 0xa800000 + 0xf8c00 and ahb2phy 0xc016000; interrupts hs_phy 0x15b, ss_phy 0xf3, pwr_event 0xb4 (SPI numbers); dwc3 core irq SPI 0x83; clocks core, iface, bus_aggr, utmi, sleep, xo; core-clk-rate 120 MHz, 60 MHz in high speed (ours assigns 60 MHz: equal).
- dwc3 quirks: snps,disable-clk-gating, snps,has-lpm-erratum, snps,hird-threshold 0x10, snps,nominal-elastic-buffer, tx-fifo-resize (tx fifo 0x5328 in the glue). Mainline has the same dwc3 properties by the same names; add them only if the first enumeration fails.
- USB 2 PHY qusb2-v2 (reg 0xc012000, efuse-bit-pos 16, 4 bits, `skip_efuse_reg`): vdd = pm8998 l1, vdda12 = l2, vdda18 = l12, vdda33 = l24; init sequence (value, register offset): 0x13@0x04, 0x7c@0x18c, 0x80@0x2c, 0x0a@0x184, 0xf5@0x23c, 0x0b@0x240, 0xe4@0x248, 0x19@0xb4. Ours has vdd=l1, pll=l12, dpdm=l24 (the same rails); the l2 (1.2 V) rail is only the SS PHY's core-supply. Start with mainline's defaults; if the gadget does not enumerate, apply TUNE1 0xf5 (0x23c), TUNE2 0x0b (0x240), TUNE4 0xe4 (0x248) through the qusb2 phy's `qcom,...-tune` properties.
- SS PHY QMP v2 (reg 0xc010000, `qcom,vbus-valid-override`, core-supply l2, vdd l1): we drop it (high-speed only), as the proposal's `usb3_dwc3 { phys = <&qusb2phy>; maximum-speed = "high-speed"; }` already does.
- VBUS / role: the vendor tree's extcon is the pmi8998 USB-PD PHY (`qcom,usb-pdphy@1700` on pmi8998@2); mainline has no driver for it, so `dr_mode = "peripheral"` stays, and `qcom,select-utmi-as-pipe-clk` with it.
- gdsc: `USB3_GDSC-supply` = gdsc_usb30@10f004 (mainline: the gcc USB_30 GDSC, power-domains in the glue node: already in msm8998.dtsi).
Net: our `&usb3`/`&usb3_dwc3`/`&qusb2phy` nodes already carry what the running fdt says; no new node is needed for the first gadget. What our tree lacks around them (the x-groups above) is the open question.
