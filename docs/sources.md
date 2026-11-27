<!-- Generated from data/ by tad. Do not edit by hand. -->

# Sources

<a id="src-kernel-ramoops-doc"></a>
### Ramoops oops/panic logger (kernel docs)

Url
: https://docs.kernel.org/admin-guide/ramoops.html

By
: walleye-research

Takes
: pstore/ramoops keeps the console and panic dump in reserved RAM across a warm reset; read from the pstore fs as console-ramoops / dmesg-ramoops-N


<a id="src-kernel-firmware-fallback"></a>
### Fallback mechanisms (kernel firmware loader docs)

Url
: https://www.kernel.org/doc/html/v4.15/driver-api/firmware/fallback-mechanisms.html

By
: walleye-research

Takes
: the user-mode-helper firmware fallback times out after 60 s by default (/sys/class/firmware/timeout)


<a id="src-android-system-as-root"></a>
### Partition layout: system-as-root (source.android.com)

Url
: https://source.android.com/docs/core/architecture/partitions/system-as-root

By
: walleye-research

Takes
: system-as-root cmdline is 'ro root=/dev/dm-0 rootwait skip_initramfs init=/init dm=...': the kernel ignores the boot ramdisk and runs /init from the system partition


<a id="src-magisk-skip-initramfs"></a>
### Magisk issues 8063 and 4307: bootloader adds skip_initramfs

Url
: https://github.com/topjohnwu/magisk/issues/8063

By
: walleye-research

Takes
: a bootloader can add skip_initramfs itself (to the cmdline or the dtb bootargs) so a swapped ramdisk is ignored


<a id="src-qcom-log-analysis"></a>
### Qualcomm Linux debug: log analysis (subsystem watchdog bark/bite, SSR)

Url
: https://docs.qualcomm.com/doc/80-88500-4/topic/71_Log_analysis.html

By
: walleye-research

Takes
: signatures: 'subsys-restart: ... restart_level = SYSTEM' followed by 'Kernel panic - not syncing: subsys-restart: Resetting the SoC - X crashed'


<a id="src-pixel-uart-cable"></a>
### Pixel 2/3/4 Android Debug Cable how-to (Peter-Easton)

Url
: https://github.com/Peter-Easton/android-debug-cable-howto

By
: walleye-research

Takes
: USB-C passthrough breakout + FT232 TTL cable (GND A1, RX A8, TX B8, VCC unused); 'fastboot oem uart enable' (bootloader unlocked) then /dev/ttyUSB0 at 115200; 'fastboot oem uart disable' after; uart stays on even if the bootloader is re-locked


<a id="src-nura-serial-pixel2"></a>
### Nura wiki: serial debugging cable schematics (Pixel 2 section)

Url
: https://wiki.nura.eco/wiki/Serial_debugging/Cable_schematics

By
: walleye-research

Takes
: Pixel 2 (walleye) tested with the Android Debug Cable; SuzyQable does not work; bootloader and kernel logs come over UART after 'fastboot oem uart enable'; UART level 1.8 V


<a id="src-android-wahoo-kernel"></a>
### Build Pixel kernels (AOSP): wahoo kernel branch

Url
: https://source.android.com/docs/setup/build/building-pixel-kernels

By
: walleye-research

Takes
: walleye/taimen kernel is android-msm-wahoo-4.4 (device/google/wahoo-kernel); the vendor GPL source to read for panic strings


<a id="src-wahoo-init-hardware-usb-rc"></a>
### device/google/wahoo init.hardware.usb.rc (crDroid mirror, branch 15.0)

Url
: https://github.com/crdroidandroid/android_device_google_wahoo/blob/15.0/init.hardware.usb.rc

By
: Google

Takes
: Android on walleye creates mass_storage, accessory, audio_source, midi, ffs.adb/mtp/ptp, diag, cser, gsi.rmnet, gsi.rndis, qdss in g1; never ncm; controller a800000.dwc3


<a id="src-kernel-gadget-configfs-doc"></a>
### Linux USB gadget configured through configfs

Url
: https://docs.kernel.org/usb/gadget_configfs.html

By
: Linux kernel

Takes
: the order: function dir, config, symlink, UDC; ncm attributes exist (dev_addr, host_addr, qmult, ifname)


<a id="src-wahoo-44-f-ncm"></a>
### kernel/msm android-msm-wahoo-4.4-android11: drivers/usb/gadget/function/f_ncm.c

Url
: https://android.googlesource.com/kernel/msm/+/refs/heads/android-msm-wahoo-4.4-android11/drivers/usb/gadget/function/f_ncm.c

By
: Google

Takes
: ncm_alloc_inst only kzallocs opts (opts->net stays NULL); the net_device is made in ncm_bind at UDC bind; ncm_alloc does refcnt++


<a id="src-wahoo-44-u-ether-configfs"></a>
### kernel/msm android-msm-wahoo-4.4-android11: u_ether_configfs.h

Url
: https://android.googlesource.com/kernel/msm/+/refs/heads/android-msm-wahoo-4.4-android11/drivers/usb/gadget/function/u_ether_configfs.h

By
: Google

Takes
: dev_addr/host_addr/qmult/ifname store: -EBUSY if refcnt, else gether_set_*(opts->net) with no NULL check


<a id="src-linux-f-fs-c"></a>
### drivers/usb/gadget/function/f_fs.c (v4.4.302 and v7.3): descriptor parser

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/drivers/usb/gadget/function/f_fs.c?h=v4.4.302

Author
: Linux kernel (GPL-2.0)

By
: Linux kernel

Takes
: ffs_do_single_desc accepts only interface, endpoint, HID, OTG, interface-association and SS companion descriptors; class-specific (0x24 CS_INTERFACE) falls to default -EINVAL in both v4.4.302 and v7.3; interface numbers are remapped by usb_interface_id (single function in a config starts at 0); a configfs bind returns -ENODEV until the descriptors are written (desc_ready)


<a id="src-linux-cdc-acm-c"></a>
### drivers/usb/class/cdc-acm.c: acm_probe without functional descriptors

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/drivers/usb/class/cdc-acm.c?h=v7.3

Author
: Linux kernel (GPL-2.0)

By
: Linux kernel

Takes
: with no union descriptor and an interface of class 2 subclass 2 with exactly three endpoints, probe assumes a single combined interface and finds interrupt-in, bulk-in, bulk-out by usb_find_common_endpoints: no CDC functional descriptors needed


<a id="src-linux-ffs-test-c"></a>
### tools/usb/ffs-test.c and Documentation/usb/functionfs.rst

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/tools/usb/ffs-test.c?h=v7.3

Author
: Samsung Electronics, Linux kernel (GPL-2.0-or-later)

By
: Linux kernel

Takes
: descriptor blob layout: header (magic V2, flags FS|HS|SS, counts), then fs, hs, ss descriptor lists; strings blob with FUNCTIONFS_STRINGS_MAGIC; ep0 event loop


<a id="aosp-libufdt-overlay"></a>
### libufdt ufdt_overlay.c (AOSP)

Url
: https://android.googlesource.com/platform/system/libufdt/+/1943dd8f54a872953f3d12c35247c7f6594ea5a6/ufdt_overlay.c

Author
: AOSP

By
: Google

Takes
: With any property in an overlay's __fixups__, a base dtb without a __symbols__ node is an error ('No node __symbols__ in main dtb'): the merge fails


<a id="bacon-pixel-uart"></a>
### Enabling serial port (console UART) on Pixel 2 and other Pixels

Url
: https://blog.bacon.dev/2020/10/enabling-serial-port-or-console-uart-on.html

Author
: bacon.dev

Year
: 2020

By
: ikb-cli

Takes
: Pixel 2 UART on USB-C SBU1/SBU2 at 1.8 V, 115200 8N1, fastboot oem uart enable; the stock cable lacks the lines; orientation differs between Pixels


<a id="xairy-pixel-kgdb"></a>
### Pixel KGDB over UART

Url
: https://xairy.io/articles/pixel-kgdb

Author
: Andrey Konovalov

By
: ikb-cli

Takes
: USB-Cereal at 1.8 V and 115200 on a Pixel 8; an FT232R supports break (KGDB sysrq), a CP2102N edition did not


<a id="google-usb-cereal"></a>
### usb-cereal

Url
: https://github.com/google/usb-cereal

Author
: Google

By
: ikb-cli

Takes
: SBU1 = TX, SBU2 = RX, DIP 1.8/3.3 V (a wrong setting can damage the target), orientation-specific, archived 2023-01-10, not an official Google product


<a id="src-linux-psci-sys-reset"></a>
### drivers/firmware/psci/psci.c psci_sys_reset (mainline)

Url
: https://github.com/torvalds/linux/blob/63fdb511c27afcebce797e3db580b3534ddfa639/drivers/firmware/psci/psci.c

By
: Linux kernel

Takes
: reboot=warm/soft uses SYSTEM_RESET2 only if psci_system_reset2_supported (PSCI 1.1); otherwise plain SYSTEM_RESET; notifier priority 129


<a id="src-linux-qcom-pon"></a>
### drivers/power/reset/qcom-pon.c (mainline)

Url
: https://github.com/torvalds/linux/blob/63fdb511c27afcebce797e3db580b3534ddfa639/drivers/power/reset/qcom-pon.c

By
: Linux kernel

Takes
: mainline pm8998-pon driver only registers reboot-mode (writes the soft spare reason); it never programs the PS_HOLD reset type (warm/hard), unlike the vendor qpnp_pon_system_pwr_off


<a id="src-qcom-pon-reset-status"></a>
### Qualcomm docs: PMIC PON reset status

Url
: https://docs.qualcomm.com/doc/80-88500-4/topic/74_PMIC_PON_reset_status.html

By
: Qualcomm

Takes
: PON_WARM_RESET_REASON / PON_ON_REASON 0x40 = warm-reset sequence; SDI runs PS_HOLD warm resets before a RAM dump: the PMIC records which kind of reset happened, readable after boot


<a id="src-linux-qcom-wdt"></a>
### drivers/watchdog/qcom-wdt.c (mainline)

Url
: https://github.com/torvalds/linux/blob/63fdb511c27afcebce797e3db580b3534ddfa639/drivers/watchdog/qcom-wdt.c

By
: Linux kernel

Takes
: qcom,kpss-wdt: restart handler (priority 128, below PSCI's 129) and a bite on an unpetted watchdog; the lab dts already has the node at 0x17817000


<a id="src-openwrt-ramoops-warm"></a>
### openwrt#5045: ramoops files survive a warm reboot, not a cold one

Url
: https://github.com/openwrt/openwrt/issues/5045

By
: walleye-research

Takes
: plain statement of the rule on a Qualcomm SoC (ipq806x): pstore survives warm, not cold reset


<a id="lwn-pmi8998-fg-rfc"></a>
### Qualcomm 3rd gen fuel gauge support (PATCH RFC 0/8)

Url
: https://lwn.net/Articles/1048226/

Author
: David Heidelberg

Year
: 2025

By
: Joel Selvaraj, Casey Connolly, Richard Acayan and others

Takes
: A PMI8998 fuel gauge driver is an RFC, not merged (2025-11-24); the dts patches name sdm845/sdm660/sdm670 boards, not walleye


<a id="kernel-qcom-smbx-charger"></a>
### drivers/power/supply/qcom_smbx.c (Qualcomm SMB2 charger)

Url
: https://github.com/torvalds/linux/blob/63fdb511c27afcebce797e3db580b3534ddfa639/drivers/power/supply/qcom_smbx.c

Author
: Casey Connolly

By
: Linux mainline

Takes
: qcom,pmi8998-charger is a mainline compatible, so the Pixel 2 charger needs a dts node, not a driver


<a id="linux-msm-mainline-status"></a>
### Qualcomm mainline status

Url
: https://linux-msm.github.io/mainline-status/

By
: linux-msm

Takes
: Where to check upstream status of Qualcomm platforms before a part is called new


<a id="linux-gpucc-msm8998-gx"></a>
### gpucc-msm8998.c and msm8998.dtsi adreno_smmu (linux-topics, v7.3)

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/drivers/clk/qcom/gpucc-msm8998.c

By
: walleye-research

Takes
: gpu_gx_gdsc has parent gpu_cx and no supply; adreno_smmu's power-domains is GPU_GX_GDSC


<a id="kcxt-dtbo-wahoo-mainline"></a>
### kcxt linux-wahoo: dtbo README (Pixel 2 XL mainline)

Url
: https://github.com/kcxt/linux-wahoo

Takes
: taimen's bootloader refuses a kernel unless dtbo holds an overlay with id 2704; recipe for a minimal dtbo


<a id="lineage-wahoo-qpnp-power-on"></a>
### qpnp-power-on.c, LineageOS android_kernel_google_wahoo (the vendor GPL kernel of walleye)

Url
: https://github.com/LineageOS/android_kernel_google_wahoo/blob/lineage-20/drivers/input/misc/qpnp-power-on.c

By
: Qualcomm / Google (GPL)

Takes
: The vendor driver writes only the WD enable bit (S2_CTL2 0x57); no S2_CTL (0x56) type write; PS_HOLD type goes through 0x5A with enable 0x5B on gen2 after clearing enable and a 500 us wait; type values WARM 1, SHUTDOWN 4, HARD 7; qcom,s3-debounce uses SEC_ACCESS


<a id="lineage-wahoo-qpnp-power-on-h"></a>
### include/linux/input/qpnp-power-on.h, android_kernel_google_wahoo

Url
: https://github.com/LineageOS/android_kernel_google_wahoo/blob/lineage-20/include/linux/input/qpnp-power-on.h

By
: Qualcomm / Google (GPL)

Takes
: enum pon_power_off_type: WARM_RESET 0x01 (reset the MSM, not all PMIC peripherals), SHUTDOWN 0x04, HARD_RESET 0x07 (reset the MSM and all PMIC peripherals)


<a id="pmic-wdt-series-pm8916-wdt"></a>
### pm8916_wdt.c with the series topic/pmic-wdt-wake-in-suspend (e123d3bff), as built into z9

Url
: https://github.com/grenudi/linux/tree/topic/pmic-wdt-wake-in-suspend

By
: upstreams lane

Takes
: probe writes S2_CTL 0x56 = 0x07 once, no unlock, no read-back; timers at 0x54/0x55, enable 0x57; poweroff reason at 0xc5 on gen2


<a id="mainline-qcom-pon-v7-3"></a>
### drivers/power/reset/qcom-pon.c, Linux v7.3

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/drivers/power/reset/qcom-pon.c?h=v7.3

By
: mainline

Takes
: qcom-pon writes only PON_SOFT_RB_SPARE (0x8f) and populates children; it never writes S2_CTL, PS_HOLD_RST_CTL or any reset type


<a id="mainline-qcom-smbx-v7-3"></a>
### drivers/power/supply/qcom_smbx.c, Linux v7.3

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/drivers/power/supply/qcom_smbx.c?h=v7.3

By
: Casey Connolly et al.

Takes
: the init sequence run at probe (Type-C software control, charging enable, ICL/AICL, watchdog off) and that it has no shutdown, remove or reboot hook


<a id="src-sony-ufs-quirks-forward-port"></a>
### Forward port Yoshino/Tama UFS quirks/hacks from OD k4.9

Url
: https://github.com/sonyxperiadev/kernel/commit/cfd148999d17ad8f1a7cbce44bb108570ad631ad

Author
: Angelo Del Regno

Year
: 2019

By
: sonyxperiadev

Takes
: the quirk set: NO_PURGE (Hynix hB8aL1/hC8aL1 D001, Samsung KLUCG4J1 0101, UFS<2.1), EXTEND_SYNC_LENGTH for all Hynix, AVAILTXDATALANES=1


<a id="src-yoshino-dtsi-ufs-disabled"></a>
### msm8998-sony-xperia-yoshino.dtsi: UFS disabled until quirks

Url
: https://raw.githubusercontent.com/torvalds/linux/master/arch/arm64/boot/dts/qcom/msm8998-sony-xperia-yoshino.dtsi

By
: mainline

Takes
: bootloader erased right after card probe without the quirks


<a id="src-sony-bugtracker-799"></a>
### Sony bug tracker 799: Tama on mainline kernel

Url
: https://github.com/sonyxperiadev/bug_tracker/issues/799

Year
: 2023

By
: Marijn Suijten

Takes
: no one dared sacrifice a device; mechanism unpublished


<a id="src-oneplus5-ufs-mainline"></a>
### OnePlus 5/5T: enable UFS (commit 47498916af)

Url
: https://github.com/torvalds/linux/commit/47498916af

Year
: 2021

By
: Jami Kettunen

Takes
: MSM8998 UFS works on mainline for that phone's card


<a id="src-kumano-ufs-quirks-thread"></a>
### Kumano devicetree thread: Sony's UFS quirks

Url
: https://lkml.iu.edu/hypermail/linux/kernel/2106.1/06123.html

Year
: 2021

By
: Angelo Del Regno

Takes
: quirks look intentional; no way to ask Sony; how to do it in mainline


<a id="src-bkerler-edl-204"></a>
### bkerler/edl issue 204: How to get or read out a Loader for Google Pixel 2?

Url
: https://github.com/bkerler/edl/issues/204

Author
: cubotron, bkerler

Year
: 2021

Takes
: A Pixel 2 that restarted into EDL: Sahara HWID 0x0005e0e100420002, PK hash 748921750a6ec931...; no loader found; the owner of the edl tool: the PK hash must match the loader, and for Pixels (except the LG-built) no loader was released or leaked


<a id="src-xda-pixel2-hardbrick-3920807"></a>
### XDA: [Hard bricked Pixel 2] qualcomm hs-usb qdloader 9008

Url
: https://xdaforums.com/t/hard-bricked-pixel-2-qualcomm-hs-usb-qdloader-9008-in-device-manager.3920807/

Year
: 2019

Takes
: Pixel 2 with no bootloader and no recovery shows 9008; QFIL needs a firehose programmer and rawprogram xml, which are proprietary; replies 2019-2023 show no solution; one reply says the device was recovered but not how


<a id="src-hovatek-pixel2-26042"></a>
### Hovatek: Potential Google pixel 2 hard brick

Url
: https://www.hovatek.com/forum/thread-26042.html

Year
: 2018

Takes
: walleye bricked by flash-all of a factory image (Nov 2018, rooted) and came up as 9008 with no logo; advice: find a working firehose programmer or boot from an SD card debrick image made on a working phone of the same model; no outcome reported


<a id="src-xda-edl-loaders-4525079"></a>
### XDA: Identifying EDL (Firehose) loaders (Renate)

Url
: https://xdaforums.com/t/identifying-edl-firehose-loaders.4525079/

Author
: Renate

Year
: 2022

Takes
: A loader is made by the phone maker from Qualcomm's xbl and is signed; with Secure Boot fused it works only on a device whose root-certificate hash matches; patching is easy, signing is not; the two public collections are bkerler/Loaders and hoplik/Firehose-Finder


<a id="src-androidse-sailfish-225868"></a>
### Android SE: Unbricking my Google Pixel (sailfish)

Url
: https://android.stackexchange.com/questions/225868/unbricking-my-google-pixel-sailfish

Year
: 2020

Takes
: Pixel (MSM8996) hard bricked by flash-all: still 9008; no firehose programmer found for sailfish; the Google driver binaries do not contain one; no solution as of 2025


<a id="src-pixelrepair-withgoogle"></a>
### Google Pixel Update and Software Repair

Url
: https://pixelrepair.withgoogle.com/

Takes
: Google's own tool works on Pixel 3 or newer running Android 10 and higher: the Pixel 2 is not covered


<a id="src-passware-qc-testpoints"></a>
### Passware: Qualcomm test-point gallery

Url
: https://support.passware.com/hc/en-us/articles/15470087921687-Qualcomm-test-point-gallery

Takes
: Public gallery of EDL test points by SoC: lists SDM845, SDM710, SM8150 and others; no Pixel 2 / MSM8998 / Google entry


<a id="src-aleph-qualcomm-edl-2"></a>
### Aleph Research: Exploiting Qualcomm EDL Programmers (2)

Url
: https://alephsecurity.com/2018/01/22/qualcomm-edl-2/

Year
: 2018

Takes
: On the Nexus 6 (shamu) adb reboot edl entered EDL; the boot chain PBL -> SBL -> ABOOT; EDL is the way back for a device whose SBL is blank - but only with a signed programmer


<a id="src-google-wahoo-msm44-android11"></a>
### Google kernel/msm, branch android-msm-wahoo-4.4-android11 (head 52750b8f): the stock Pixel 2 (wahoo) 4.4 kernel, UFS host, PHY and dts

Url
: https://android.googlesource.com/kernel/msm/+/refs/heads/android-msm-wahoo-4.4-android11

Author
: Google, Qualcomm (GPL-2.0)

Year
: 2020

By
: Google

Takes
: UFS bring-up sequence, quirk tables and PHY init values as facts (clean room: nothing copied)


<a id="linux-v7-3-gpiolib-pinctrl"></a>
### Linux v7.3 gpiolib, gpiolib-of, pinctrl core, pinctrl-spmi-gpio, regulator fixed.c (read at the tag)

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/drivers/gpio?h=v7.3

Author
: Linux kernel authors

Year
: 2026

Takes
: Where gpiod_get_optional can return -EPROBE_DEFER for a chip that exists: of_get_named_gpiod_flags (no gdev matches), pinctrl_gpio_request (no gpio range), gpiod_request (module_get)


<a id="wahoo-4-4-ufs-qmp-v3"></a>
### Google wahoo 4.4 kernel: phy-qcom-ufs-qmp-v3.c and ufs-qcom.c (android-msm-wahoo-4.4-android11)

Url
: https://android.googlesource.com/kernel/msm/+/refs/heads/android-msm-wahoo-4.4-android11/drivers/phy/phy-qcom-ufs-qmp-v3.c

By
: Google

Takes
: facts only: version-to-table selection, SVS2 branch on 3.0.0, no version print


<a id="mainlining-dev-ufs-phy-driver"></a>
### UFS phy driver - A guide to mainlining android phones

Url
: https://mainlining.dev/2021/07/04/ufs-phy-driver/

Year
: 2021

Takes
: downstream v3 tables follow mainline sdm845 values with minuscule exceptions


<a id="wahoo-4-4-ufs-qcom-c"></a>
### Google wahoo 4.4 kernel: drivers/scsi/ufs/ufs-qcom.c, ufs-qcom.h, phy-qcom-ufs-qmp-v3.h (android-msm-wahoo-4.4-android11)

Url
: https://android.googlesource.com/kernel/msm/+/refs/heads/android-msm-wahoo-4.4-android11/drivers/scsi/ufs/ufs-qcom.c

By
: Google

Takes
: link_startup_pre_change steps, register addresses and masks


<a id="src-mainline-v73-ufs-qcom-ice"></a>
### Linux v7.3: ufs-qcom.c, soc/qcom/ice.c, ICE and ufshc bindings, msm8998.dtsi

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/drivers/soc/qcom/ice.c?h=v7.3

By
: Linux kernel

Takes
: ICE-less path warns and returns 0; no msm8998 ICE compatible or node


<a id="src-sony-bugtracker-802"></a>
### Sony bug tracker 802: Tama XZ3 mainline, dreaded UFS support (bartcubbins, 2023-09 and 2025-06; phodina 2025-05)

Url
: https://github.com/sonyxperiadev/bug_tracker/issues/802

Year
: 2025

By
: Sony Open Devices contributors

Takes
: a SODP maintainer: one write cycle purges the memory 'sort of'; keep the driver from handling QUERY_FLAG_IDN_PURGE_ENABLE; sync-length quirk may relate; a dump cannot be restored; the SoC is QPROM-locked so EDL needs the UFS-held xbl


<a id="src-xda-yoshino-ufs-erasure-origin"></a>
### XDA: Looking for the origin and scope of the UFS bootloader-erasure warning (XZ1 poplar, Samsung KLUCG4J1 fw 0200)

Url
: https://xdaforums.com/t/help-looking-for-the-origin-and-scope-of-the-ufs-bootloader-erasure-warning.4800672/

Year
: 2026

By
: chendoxiu1

Takes
: a yoshino XZ1 with Samsung fw 0200 probed UFS on mainline several times with no erasure; he found no wipe path in the probe and asks the origin question nobody answers; no reply on record


<a id="ti-drv2624-datasheet"></a>
### DRV2624 haptic driver product page and datasheet

Url
: https://www.ti.com/product/DRV2624

By
: Texas Instruments

Takes
: register-level behaviour of the haptic chip (public datasheet, clean-room basis for a driver)


<a id="ti-tas2557-datasheet"></a>
### TAS2557 5.7-W Class-D mono audio amplifier with class-H boost and DSP

Url
: https://www.ti.com/lit/ds/symlink/tas2557.pdf

By
: Texas Instruments

Takes
: the amp has an on-chip DSP (SmartAmp); public datasheet is the clean-room basis for a bypass-mode driver


<a id="aosp-msm-marlin-codecs"></a>
### kernel/msm android-msm-marlin-3.18-oreo sound/soc/codecs

Url
: https://android.googlesource.com/kernel/msm/+/android-msm-marlin-3.18-oreo/sound/soc/codecs

By
: Google, Qualcomm, TI

Takes
: GPL vendor starting point: tas2557 and wcd934x sources of the wahoo/marlin-family kernels (read for register sequences, not copied wholesale)


<a id="nura-taimen-wiki"></a>
### Google Pixel 2 XL (google-taimen) - Nura Wiki

Url
: https://wiki.nura.eco/wiki/Google_Pixel_2_XL_(google-taimen)

Year
: 2026

By
: Casey (Kcxt)

Takes
: sibling Pixel 2 XL mainline status: boots, USB net, audio untested


<a id="mainline-7-1-tree-read"></a>
### Mainline 7.1 tree read for msm8998 audio, wcd934x, tas255x/tas27xx, drv26xx

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git

Year
: 2026

By
: kernel community

Takes
: what mainline has: wcd934x.c, tas2552/2562/2764/2770/2780/2781/2783, tfa989x, drv260x/2665/2667; no tas2557, no drv2624; msm8998.dtsi has no slimbus/audio/q6 nodes


<a id="porthole-pmaports-taimen"></a>
### porthole-dev/pmaports: Pixel 2 XL (taimen) port on mainline 7.2.2 (Giuseppe Maggio)

Url
: https://github.com/porthole-dev/pmaports

Author
: Giuseppe Maggio

Year
: 2026

Takes
: tas2557.c, drv2624.c, ADSP/q6/MI2S/SLIMbus/wcd934x patches 0018-0175, taimen dts audio nodes, firmware package (tas2557s_PG21_uCDSP.bin)


<a id="linux-v7-3-pinctrl-devicetree"></a>
### Linux v7.3 drivers/pinctrl/devicetree.c (dt_to_map_one_config)

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/drivers/pinctrl/devicetree.c?h=v7.3

By
: Linux kernel

Takes
: A pinctrl-N phandle whose node has no pin controller among its parents returns -EPROBE_DEFER (modules enabled), silently


<a id="linux-v7-3-driver-core-dd"></a>
### Linux v7.3 drivers/base/dd.c and drivers/base/pinctrl.c (really_probe, pinctrl_bind_pins)

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/drivers/base/dd.c?h=v7.3

By
: Linux kernel

Takes
: pinctrl_bind_pins runs before the bus probe (platform_probe) and the driver's probe; its failure defers with no message


<a id="lkml-nxp-pn553-binding"></a>
### dt-bindings: nfc: nxp,nci: document pn553

Url
: https://lkml.iu.edu/hypermail/linux/kernel/2410.2/06246.html

By
: research-lead

Takes
: nxp,pn553 compatible with nxp-nci-i2c, enable-gpios


<a id="lkml-cpuidle-msm8998-latency-fix"></a>
### msm8998 idle-state latency and residency fix (e52fecdd0c14, v5.16-rc1)

Url
: https://ayakael.net/forge/linux-uconsole/commit/e52fecdd0c14

By
: research-lead

Takes
: msm8998 PSCI idle states corrected


<a id="hackernoon-oneplus3t-mainline-pm"></a>
### How I fixed power management on a mainline OnePlus 3T

Url
: https://hackernoon.com/how-i-fixed-power-management-on-a-mainline-oneplus-3t

By
: research-lead

Takes
: cpuidle-psci was disabled in the kernel config; a sibling msm8996 phone


<a id="biometricupdate-pixel2-crucialtec"></a>
### Google Pixel 2 has integrated CrucialTec fingerprint module, sources say

Url
: https://www.biometricupdate.com/201710/google-pixel-2-has-integrated-crucialtec-fingerprint-module-sources-say

By
: research-lead

Takes
: Pixel 2 fingerprint module maker, as sources said in 2017; unconfirmed


<a id="mss-msm8998-driver-1665cbd5"></a>
### remoteproc: qcom: q6v5-mss: add msm8998 support (Jeffrey Hugo, 2019)

Url
: https://lab.nexedi.com/kirr/linux/-/commit/1665cbd5731fa7c66103ec719c57922bbc457447

By
: research-lead

Takes
: MSM8998 modem driver is upstream


<a id="pmaports-oneplus5-mainline-mr3531"></a>
### pmaports MR 3531: OnePlus 5 mainline kernel

Url
: https://gitlab.postmarketos.org/postmarketOS/pmaports/-/merge_requests/3531

By
: research-lead

Takes
: ipa blocked from autoload; closest working msm8998 modem


<a id="modemmanager-issue-386"></a>
### ModemManager issue 386: IPA v3.1 over QRTR

Url
: https://gitlab.freedesktop.org/mobile-broadband/ModemManager/-/issues/386

By
: research-lead

Takes
: msm8998 modem in ModemManager


<a id="camss-sdm630-660-v2"></a>
### SDM630/660 camss v2 (Del Regno, 2020)

Url
: https://lore.kernel.org/20201022174706.8813-1-kholk11@gmail.com/

By
: research-lead

Takes
: VFE 4.8, CSID 5.0 base, said 90% reusable for msm8998


<a id="sdm670-mainline-camera-vcm-issue"></a>
### sdm670-mainline issue 2: Pixel 3a camera, VCM driver missing

Url
: https://gitlab.com/sdm670-mainline/linux/-/issues/2

By
: research-lead

Takes
: closest Pixel camera effort; blocked on LC898212XD VCM


<a id="imx378-rfc-2020"></a>
### imx378 driver RFC (Gomez, 2020)

Url
: https://lore.kernel.org/all/20200414200151.80089-1-daniel@qtec.com/T/

By
: research-lead

Takes
: front sensor driver draft, stalled


<a id="pmos-msm8998-wiki"></a>
### postmarketOS Qualcomm Snapdragon 835 (MSM8998)

Url
: https://wiki.postmarketos.org/wiki/Qualcomm_Snapdragon_835_(MSM8998)

By
: research-lead

Takes
: SoC feature status: modem Partial, cpuidle Partial


<a id="lkml-slpi-msm8998-pas-2019"></a>
### remoteproc qcom_q6v5_pas: msm8998 ADSP and SLPI (Sankar, 2019)

Url
: https://lkml.iu.edu/hypermail/linux/kernel/1912.2/04264.html

By
: research-lead

Takes
: SLPI pas support upstream


<a id="pmaports-mr4050-sdm845-sensors"></a>
### pmaports MR 4050: SDM845 sensors via libssc, hexagonrpcd, iio-sensor-proxy

Url
: https://gitlab.com/postmarketOS/pmaports/-/merge_requests/4050

By
: research-lead

Takes
: the recipe for the SLPI sensor stack; msm8998 not mentioned


<a id="emainline-fingerprint-p1"></a>
### Fingerprint sensors on tuxified Android phones: Impossible?

Url
: https://emainline.gitlab.io/2021/12/12/fingerprint_P1.html

Author
: emainline

Year
: 2021

Takes
: Linux can read an FPC sensor's HWID over plain SPI on msm8996 once the vendor dts supplies irq, reset, supply; the later steps (image capture, matching) are the hard part


<a id="fpc1020-tee-android-kernel"></a>
### fpc1020_tee.c, Android kernel/msm

Url
: https://android.googlesource.com/kernel/msm/+/3d0a564505ab8452e7e6f52b972db386cc2f5f69/drivers/input/fingerprint/fpc1020_tee.c

By
: Google/FingerprintCard

Takes
: the vendor driver only owns irq, reset and power; the sensor protocol is in the TEE trustlet


<a id="libfprint-supported-devices"></a>
### libfprint supported devices

Url
: https://fprint.freedesktop.org/supported-devices.html

Takes
: no FPC SPI (phone-class) driver is listed; FPC entries are USB devices


<a id="lkml-boiano-tas2557-v3"></a>
### [PATCH v3 0/2] ASoC: codecs: add Texas Instruments TAS2557 smart amplifier driver (with binding, patch 1/2)

Url
: https://lkml.iu.edu/2607.2/04618.html

Author
: Gianluca Boiano

Year
: 2026

By
: Gianluca Boiano

Takes
: binding ti,tas2557 (reg array of two, four supplies, firmware-name), driver, test boards


<a id="lkml-broonie-tas2557-v3-review"></a>
### Re: [PATCH v3 2/2] ASoC: codecs: add TAS2557 mono smart amplifier driver (Mark Brown's review)

Url
: https://lkml.iu.edu/2607.2/06067.html

Author
: Mark Brown

Year
: 2026

Takes
: open review points: board-specific startup GPIO table, PLL index range, firmware device-count check, suspend mute of sense


<a id="linux-v7-3-fbcon-fbmem"></a>
### Linux v7.3: drivers/video/fbdev/core/fbmem.c, fbcon.c and drivers/tty/vt/vt.c, vt_ioctl.c (the tree the walleye image is built from)

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/drivers/video/fbdev/core/fbcon.c?h=v7.3

By
: Linux kernel

Takes
: fb_set_var_from_user calls fbcon_update_vcs, which repaints the visible console (fbcon_modechanged: update_screen); fbcon_modechanged, fbcon_fb_blanked and vt_console_print all skip when vc_mode is not KD_TEXT; KDSETMODE (vt_kdsetmode) switches that mode


<a id="linux-v7-3-msm-dsi-host"></a>
### Linux v7.3 drivers/gpu/drm/msm/dsi/dsi_host.c and drm_panel.c

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/drivers/gpu/drm/msm/dsi/dsi_host.c

Takes
: dsi_cmd_dma_tx 200 ms completion wait; dsi_err_worker; status bits FIFO=4, MDP_FIFO_UNDERFLOW=8; BLOCK_DMA_WITHIN_FRAME for DSI 6G >= 1.2; TODO 'make sure dsi_cmd_mdp is idle'; drm_panel_enable only logs the backlight failure


<a id="kernel-oneplus-common-dtsi"></a>
### msm8998-oneplus-common.dtsi (mainline): cont_splash_mem and simple-framebuffer

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/arm64/boot/dts/qcom/msm8998-oneplus-common.dtsi

By
: Linux kernel, qcom dts maintainers

Takes
: the splash region reserved no-map at 0x9d400000 and the clocks and GDSC the framebuffer pins


<a id="kernel-msm8998-dtsi-mdss"></a>
### msm8998.dtsi (mainline): mdss, dsi0, dsi0 phy, mmss_smmu nodes

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/arm64/boot/dts/qcom/msm8998.dtsi

By
: Linux kernel

Takes
: the SoC-level display description is complete: iommus on mdss, assigned-clock-parents on dsi0


<a id="qcom-pmic-pon-reset-status"></a>
### Qualcomm: PMIC PON reset status (PON_REASON1 bits, PON_ON_REASON warm vs normal)

Url
: https://docs.qualcomm.com/doc/80-88500-4/topic/74_PMIC_PON_reset_status.html

Author
: Qualcomm

Takes
: PON_REASON1 0x808 bits: KPDPWR_N 7, CBLPWR_N 6, PON1 5, USB_CHG 4, DC_CHG 3, RTC 2, hard reset 0; PON_ON_REASON 0x40 warm reset vs 0x80 normal power-on


<a id="qcom-pmic-boot-guide-reset-types"></a>
### Qualcomm Linux Boot Guide: PMIC reset-type (WARM_RESET for crash dumps, HARD_RESET to restart, SHUTDOWN)

Url
: https://docs.qualcomm.com/doc/80-70030-4/topic/pmic-developer-touchpoints.html

Author
: Qualcomm

Takes
: The vendor's own meaning of the three reset types, per reset source (KPDPWR S2 etc.)


<a id="vendor-qpnp-power-on-c"></a>
### qpnp-power-on.c, vendor msm kernel (comma msm8996 copy)

Url
: https://github.com/commaai/android_kernel_comma_msm8996/blob/master/drivers/platform/msm/qpnp-power-on.c

Takes
: S2 type written to the S2 CNTL register from qcom,s2-type, 4-bit mask; the PS_HOLD type is chosen per warm/hard/shutdown, overridable per board in DT; the boot reason is ffs(PON reason)


<a id="mainline-pm8916-wdt-c"></a>
### drivers/watchdog/pm8916_wdt.c, Linux 7.3 (our tree)

Url
: https://github.com/torvalds/linux/blob/master/drivers/watchdog/pm8916_wdt.c

Takes
: RESET_TYPE_WARM 0x01, SHUTDOWN 0x04, HARD 0x07 for the S2 control register 0x56


<a id="android-bootstat-reasons"></a>
### AOSP system/core bootstat.cpp: boot reason strings

Url
: https://android.googlesource.com/platform/system/core/+/refs/heads/main/bootstat/bootstat.cpp

Takes
: Android maps bootloader reasons such as hard_rst, usb_chg, kpdpwr, pmic_watchdog, cold,charger; sys.boot.reason is Android's reading, not the PON register


<a id="lkml-arm-smmu-qcom-read-back-stream-mappings"></a>
### iommu/arm-smmu-qcom: Read back stream mappings (v5 2/3)

Url
: https://lkml.iu.edu/hypermail/linux/kernel/2010.2/02870.html

Author
: Bjorn Andersson

Year
: 2020

Takes
: the bootloader's valid SMRs are kept as BYPASS across the SMMU reset (commit 07a7f2caaa5a); only until a device attaches a translating domain


<a id="lkml-arm-smmu-handoff-smr-rfc"></a>
### iommu: arm-smmu: Handoff SMR registers and context banks (RFC)

Url
: https://lkml.iu.edu/1906.1/05055.html

Author
: Bjorn Andersson, Jeffrey Hugo

Year
: 2019

Takes
: the same failure as ours (framebuffer works until the display driver attaches, then context faults); idea of pre-installing handover translations in the context bank; not merged


<a id="mainline-dts-oneplus5-pixel3-splash"></a>
### mainline dts at v7.3: msm8998-oneplus-common, sdm845-oneplus-common, sdm845-google-common/crosshatch, msm8998-fxtec-pro1

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/arch/arm64/boot/dts/qcom

Takes
: OnePlus 5/5T keep simplefb and do not enable mdss; Pixel 3 crosshatch disables mdss/dispcc so simple-framebuffer keeps working; no msm8998 board runs msm over a live splash


<a id="v73-arm-smmu-qcom-msm-iommu-source"></a>
### v7.3 source: arm-smmu-qcom.c, msm_iommu.c, msm_kms.c, iommu.c, msm_mdss.c, mmcc-msm8998.c

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/drivers

Takes
: msm8998 smmu takes qcom_smmu_cfg_probe; msm attaches its own paging domain (no direct mappings); msm_mdss_reset exists and MDSS_BCR is registered in mmcc-msm8998


<a id="src-pmaports-crosshatch-mr83"></a>
### pmaports MR 83: crosshatch (Pixel 3 XL) new device

Url
: https://gitlab.com/postmarketOS/pmaports/-/merge_requests/83

Takes
: Pixel A/B ports flash one slot only; pmOS on a Pixel boots from the active slot


<a id="src-pmbootstrap-install-guide"></a>
### postmarketOS installation guide with pmbootstrap (ivonblog)

Url
: https://ivonblog.com/en-us/posts/postmarketos-general-installation/

Takes
: flasher flash_rootfs (--partition userdata when system is too small), flash_kernel, fastboot reboot


<a id="src-debian-wiki-oneplus6"></a>
### Debian wiki: InstallingDebianOn/OnePlus/OnePlus6 (Mobian)

Url
: https://wiki.debian.org/InstallingDebianOn/OnePlus/OnePlus6

Takes
: Mobian flow: fastboot flash boot, erase userdata, flash userdata (sparse, -S 100M), erase dtbo


<a id="src-oneplus5-mainline-jami"></a>
### JamiKettunen/linux-mainline-oneplus5 (deprecated, moved to msm8998-mainline/linux)

Url
: https://github.com/JamiKettunen/linux-mainline-oneplus5

Takes
: OP5 (msm8998) runs a loopback rootfs.img on the userdata partition from a ramdisk


<a id="src-avb-readme"></a>
### Android Verified Boot 2.0 README (AOSP external/avb)

Url
: https://android.googlesource.com/platform/external/avb/+/master/README.md

Takes
: vbmeta, descriptors, append_vbmeta_image to a boot image; unlocked state allows a failed verification


<a id="src-pmaports-msm8998-6-0"></a>
### pmaports MR 3531: MSM8998 mainline kernel 6.0 and OnePlus 5/5T updates

Url
: https://gitlab.com/postmarketOS/pmaports/-/merge_requests/3531

Takes
: OP5/5T installed to internal storage on mainline


<a id="src-google-device-wahoo"></a>
### device/google/wahoo: Google's AOSP device configuration for the Pixel 2 and 2 XL (walleye and taimen)

Url
: https://android.googlesource.com/device/google/wahoo

Author
: Google (AOSP)

Year
: 2020

By
: Google

Takes
: PRIMARY source for the Android-side device configuration of the Pixel 2: fstab.hardware (partition names, mount options, A/B), recovery, init.hardware.*.rc (the USB gadget lines we used from crDroid's copy), init.insmod.cfg (the vendor module order), mixer_paths_tavil.xml and audio_platform_info_tavil.xml (WCD934x routing, for ALSA UCM), thermal-engine.conf and thermal_info_config.json (zones and limits), synaptics_dsxv26.idc (touch), WCNSS_qcom_cfg.ini (Wi-Fi). Read the branch or tag that matches the phone's Android build (android11-release is the last release branch listed; main is the later tidy-up). Google's tree has no proprietary-*.txt: the firmware blobs are not in it.


<a id="src-projectelixir-device-google-walleye"></a>
### ProjectElixir-Devices/device_google_walleye, branch udc: Project Elixir's Android device tree for the Pixel 2

Url
: https://github.com/ProjectElixir-Devices/device_google_walleye/tree/udc

Author
: Project Elixir (Pranav Vashi, Asriadi Rahim and the AOSP history it keeps)

Year
: 2024

By
: Project Elixir

Takes
: WHERE THE AUTHOR SAW IT (2026-11-20), not the primary: a custom-ROM tree that keeps Google's device/google/wahoo history (its commits say 'Merged-In' and AOSP Change-Ids) and adds Elixir's own. Use it for what Google's tree lacks: proprietary-firmware.txt and proprietary-files*.txt (the lists of firmware and vendor blobs and their paths: modem, adsp, venus, wlan) and extract-files.sh, and for the REPARTITIONING custom ROMs do (its fstab.hardware and gpt-utils define a product partition as A/B: 'walleye and taimen require repartitioning'); that fstab is NOT the stock layout, so partition names and mount options of the STOCK phone are read from src-google-device-wahoo. The Dolby, powerhint and audio_effects files are Elixir's, not Google's. Read-only; GitHub shows it as not a fork.


<a id="src-aosp-qcom-bootctrl"></a>
### platform/hardware/qcom/bootctrl boot_control.cpp (AOSP main)

Url
: https://android.googlesource.com/platform/hardware/qcom/bootctrl/+/refs/heads/main/boot_control.cpp

Author
: Google

By
: Google (Apache 2.0)

Takes
: What Android's boot control HAL writes on setActiveBootSlot, markBootSuccessful, setSlotAsUnbootable: GPT attribute bits of every A/B partition on every LUN, GPT CRC, and the UFS boot LUN; read only, nothing copied


<a id="src-caf-abl-partitiontableupdate"></a>
### abl_tianocore_edk2 QcomModulePkg PartitionTableUpdate.c and FastbootCmds.c (CAF, commit 797918c)

Url
: https://gitlab.com/Codeaurora/abl_tianocore_edk2/blob/797918c3bf59e2d65b4acb2acdfd6883a0605ee1/QcomModulePkg/Library/BootLib/PartitionTableUpdate.c

Author
: Qualcomm / Code Aurora

By
: Qualcomm

Takes
: What a Qualcomm ABL's fastboot set_active and its boot-time retry counting do: attributes, type-GUID swap, boot LUN, GPT write; a stand-in: the Pixel 2's own ABL is proprietary


<a id="src-aosp-ab-bootloader-updating"></a>
### Implement OTA updates (bootloader slot rules)

Url
: https://source.android.com/docs/core/architecture/bootloader/updating

Author
: Google

Takes
: The slot contract: set_active clears unbootable and successful and resets the retry count; modifying a slot's partition clears successful; retry count decrements per unconfirmed boot


<a id="wahoo-4-4-ufshcd-c"></a>
### Google wahoo 4.4 kernel: drivers/scsi/ufs/ufshcd.c, ufshcd.h, ufs_quirks.[ch] (android-msm-wahoo-4.4-android11 @52750b8f)

Url
: https://android.googlesource.com/kernel/msm/+/refs/heads/android-msm-wahoo-4.4-android11/drivers/scsi/ufs/ufshcd.c

By
: Google

Takes
: which commands the UFS core issues (START_STOP, bkops, query flags, gear scaling, no_write_same); read only, clean room


<a id="src-mainline-v73-ufshcd"></a>
### Linux v7.3 drivers/ufs/core/ufshcd.c, ufs_bsg.c, host/ufs-qcom.c and drivers/scsi/sd.c (our tree topic/ufs-match-android 3644090c7)

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/drivers/ufs/core/ufshcd.c?h=v7.3

By
: Linux kernel community, our purge guard

Takes
: write booster probe condition, purge guard, runtime autosuspend, gear scaling, quirk tables


<a id="src-wahoo-msm-poweroff"></a>
### msm-poweroff.c, android-msm-wahoo-4.4-android11

Url
: https://android.googlesource.com/kernel/msm/+/refs/heads/android-msm-wahoo-4.4-android11/drivers/power/reset/msm-poweroff.c

Author
: The Linux Foundation / Google

Year
: 2017

By
: Google

Takes
: msm_restart_prepare: for 'bootloader' it sets PMIC restart reason 2 and writes 0x77665500 to the IMEM restart_reason word, and a warm reset for any non-empty command


<a id="src-wahoo-qpnp-power-on"></a>
### qpnp-power-on.c, android-msm-wahoo-4.4-android11

Url
: https://android.googlesource.com/kernel/msm/+/refs/heads/android-msm-wahoo-4.4-android11/drivers/input/misc/qpnp-power-on.c

Author
: The Linux Foundation / Google

Year
: 2017

By
: Google

Takes
: qpnp_pon_set_restart_reason writes PON base+0x8F bits 7:1 = reason<<1 on gen2 (PM8998); enum value BOOTLOADER = 2 (include/linux/input/qpnp-power-on.h)


<a id="src-wahoo-msm8998-dtsi"></a>
### msm8998.dtsi, android-msm-wahoo-4.4-android11 (arch/arm/boot/dts/qcom)

Url
: https://android.googlesource.com/kernel/msm/+/refs/heads/android-msm-wahoo-4.4-android11/arch/arm/boot/dts/qcom/msm8998.dtsi

Author
: The Linux Foundation / Google

Year
: 2017

By
: Google

Takes
: IMEM at 0x146bf000, restart_reason@65c (qcom,msm-imem-restart_reason); pshold restart node; pm8998 pon has qcom,store-hard-reset-reason


<a id="src-mainline-qcom-pon"></a>
### drivers/power/reset/qcom-pon.c, Linux mainline

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/plain/drivers/power/reset/qcom-pon.c

Author
: Linaro

Year
: 2018

By
: Linux kernel

Takes
: reboot-mode driver: writes PON 0x8f, mask 7:shift, magic<<shift; pm8998-pon shift 1, so mode-bootloader <2> writes the same byte as Android


<a id="src-mainline-reboot-mode"></a>
### drivers/power/reset/reboot-mode.c, Linux mainline

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/plain/drivers/power/reset/reboot-mode.c

Author
: Andy Yan

Year
: 2016

By
: Linux kernel

Takes
: reboot notifier: looks the command string up in the mode-* properties; no command means no write


<a id="src-busybox-halt-init"></a>
### busybox init/halt.c and init/init.c (the version in walleye-build)

Url
: https://git.busybox.net/busybox/tree/init/halt.c

Author
: busybox

By
: busybox

Takes
: reboot applet ignores its arguments and signals init with SIGTERM; init and halt call reboot(RB_AUTOBOOT), never LINUX_REBOOT_CMD_RESTART2 with a string


<a id="src-pmaports-mr3190-slot-switch"></a>
### pmaports MR 3190 (Pixel 2 / A-B boot: after 7 unsuccessful boots the bootloader switches slot)

Url
: https://gitlab.postmarketos.org/postmarketOS/pmaports/-/merge_requests/3190

Takes
: Reported fall back after unsuccessful boots; the count 7 is the Qualcomm default, this phone measured 1


<a id="linux-v7-3-msm8998-dtsi-slpi"></a>
### Linux v7.3 arch/arm64/boot/dts/qcom/msm8998.dtsi: remoteproc_slpi, smp2p-slpi, slpi_mem

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/arm64/boot/dts/qcom/msm8998.dtsi?h=v7.3

By
: mainline (Sankar, 2019, and later)

Takes
: the remoteproc node and its properties; slpi_mem 0x94300000/0xf00000 (the vendor source's original, before the dtbo)


<a id="vendor-walleye-slpi-firmware-header"></a>
### The phone's own slpi_v2 firmware, read on the lab machine (vendor_a /firmware): ELF program headers

Url
: https://source.android.com/docs/setup/build/building-pixel-kernels

By
: Google/Qualcomm firmware, proprietary: read only, never in git

Takes
: link span 0x93800000-0x94700000 = 15 MiB; relocatable flag on every LOAD segment (the image itself is a lab-machine file)


<a id="vendor-walleye-dtbo-pil-fragments"></a>
### The phone's own dtbo and running fdt: pil_slpi_mem and the sibling PIL regions

Url
: https://android.googlesource.com/kernel/msm/+/refs/heads/android-msm-wahoo-4.4-android11/arch/arm64/boot/dts/qcom/msm8998.dtsi

By
: Google (GPL kernel dts; the dtbo is read from the phone)

Takes
: slpi base 0x94c00000 size 0xf00000 in the walleye entries, 0x94b00000 in the other seven; the SSC node's supplies, smp2p, pas-id


<a id="mainline-v612-ufshcd-clk-scaling"></a>
### Linux v6.12 drivers/ufs/core/ufshcd.c and host/ufs-qcom.c: devfreq clock scaling, gear scale, clk-scale notify

Url
: https://github.com/torvalds/linux/blob/v6.12/drivers/ufs/core/ufshcd.c

Takes
: ufshcd_devfreq_scale: prepare (1 s doorbell wait, new requests held), gear down before clocks down, gear up after clocks up; clkscale_enable=0 scales UP; ufs-qcom notify enters and leaves hibern8 around each change; devfreq polls every 100 ms with simple_ondemand


<a id="oneplus5-mainline-ufs-dts"></a>
### OnePlus 5/5T device tree series (msm8998-oneplus-common.dtsi): UFS internal storage on mainline

Url
: https://www.spinics.net/lists/linux-arm-msm/msg84453.html

Takes
: Same SoC and same msm8998.dtsi ufshc node (no iommus, no interconnects, legacy freq-table) is the UFS internal storage of a mainline phone: the shared driver path is not the first suspect; what is ours is


<a id="aosp-bootloader-updating"></a>
### Implement OTA updates (A/B bootloader requirements)

Url
: https://source.android.com/docs/core/architecture/bootloader/updating

By
: Android Open Source Project

Takes
: Under 'Support for slots': when modifying a partition that is part of a slot, the bootloader clears 'successfully booted' and resets the retry count for the slot; set_active/setActiveBootSlot clears unbootable and successful and resets the retry count


<a id="aosp-ab-system-updates"></a>
### A/B (seamless) system updates

Url
: https://source.android.com/docs/core/ota/ab

By
: Android Open Source Project

Takes
: bootable and successful attributes per slot; an unsuccessful slot after several boot attempts is marked unbootable by the bootloader


<a id="src-android-boot-reason"></a>
### Canonical boot reason (AOSP)

Url
: https://source.android.com/docs/core/architecture/bootloader/boot-reason

By
: Android Open Source Project

Takes
: ro.boot.bootreason=watchdog is the bootloader's reading of a hardware watchdog reset, in the kernel set of reasons: it says a watchdog bit, not that our kernel panicked or started


<a id="src-aosp-lk-aboot"></a>
### kernel/lk app/aboot/aboot.c (Google's LK tree): cmd_reboot, cmd_continue, check_reboot_mode

Url
: https://android.googlesource.com/kernel/lk/+/4f99a881317afabdbdb387065189e1895a47b027/app/aboot/aboot.c

By
: Google, Qualcomm CAF

Takes
: fastboot reboot is reboot_device(0), reboot-bootloader is reboot_device(FASTBOOT_MODE), continue is its own command; a generic LK, not walleye's closed bootloader


<a id="src-lk-qcom-reboot-mode-scrub"></a>
### Qualcomm platform shutdown process (quotes LK check_hard_reboot_mode / check_reboot_mode)

Url
: https://www.programmersought.com/article/18822967982/

By
: an article quoting CAF LK

Takes
: LK reads PON_SOFT_RB_SPARE, scrubs only bits 7:5 (value & 0x1f), and zeroes the IMEM restart reason word when it reads it; secondary source, CAF LK not read directly


<a id="src-wahoo-msm-watchdog-v2"></a>
### watchdog_v2.c, android-msm-wahoo-4.4-android11 (the stock Pixel 2 apps watchdog driver)

Url
: https://android.googlesource.com/kernel/msm/+/refs/heads/android-msm-wahoo-4.4-android11/drivers/soc/qcom/watchdog_v2.c

By
: Google

Takes
: registers WDT0_RST 0x04, EN 0x08, STS 0x0C, BARK 0x10, BITE 0x14 (the same offsets as qcom,kpss-wdt); WDT_HZ 32765; the KERNEL's probe writes bark = bark-time and bite = bark + 3 s and enables it (lines 743-744), pets every pet-time; a separate secure watchdog is disabled only by an SCM call (SCM_SVC_SEC_WDOG_DIS)


<a id="src-linux-qcom-wdt-v73"></a>
### drivers/watchdog/qcom-wdt.c and watchdog_dev.c, Linux v7.3 (topic tree)

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/drivers/watchdog/qcom-wdt.c

By
: Linux

Takes
: probe: is_running reads WDT_EN bit 0, then qcom_wdt_start (EN=0, RST, bark=timeout-pretimeout, bite=timeout, EN=1) and WDOG_HW_RUNNING; default timeout 30 s, pretimeout 1; the core pets a HW_RUNNING watchdog only while userspace has not activated it (watchdog_need_worker)


<a id="aosp-fastboot-cpp-android11"></a>
### AOSP system/core fastboot/fastboot.cpp (android11-release)

Url
: https://android.googlesource.com/platform/system/core/+/refs/heads/android11-release/fastboot/fastboot.cpp

By
: Google

Takes
: do_flash l.1232, flash_buf l.1044, set_active l.1246: what fastboot flash does and does not verify (behaviour only)


<a id="aosp-update-engine-download-action"></a>
### AOSP update_engine payload_consumer/download_action.cc (android11-release)

Url
: https://android.googlesource.com/platform/system/update_engine/+/refs/heads/android11-release/payload_consumer/download_action.cc

By
: Google

Takes
: l.199: the target slot is marked unbootable before the write


<a id="aosp-update-engine-postinstall"></a>
### AOSP update_engine payload_consumer/postinstall_runner_action.cc (android11-release)

Url
: https://android.googlesource.com/platform/system/update_engine/+/refs/heads/android11-release/payload_consumer/postinstall_runner_action.cc

By
: Google

Takes
: l.343-347: the target slot is made active only after success


<a id="aosp-update-engine-boot-control-android"></a>
### AOSP update_engine boot_control_android.cc (android11-release)

Url
: https://android.googlesource.com/platform/system/update_engine/+/refs/heads/android11-release/boot_control_android.cc

By
: Google

Takes
: MarkSlotUnbootable l.107, SetActiveBootSlot l.124, MarkBootSuccessfulAsync l.138


<a id="aosp-boot-hal-bootcontrol"></a>
### AOSP hardware/interfaces boot/1.0/default/BootControl.cpp (android11-release)

Url
: https://android.googlesource.com/platform/hardware/interfaces/+/refs/heads/android11-release/boot/1.0/default/BootControl.cpp

By
: Google

Takes
: the HIDL boot control: thin wrapper over the vendor module


<a id="aosp-update-engine-fs-verifier"></a>
### AOSP update_engine payload_consumer/filesystem_verifier_action.cc (android11-release)

Url
: https://android.googlesource.com/platform/system/update_engine/+/refs/heads/android11-release/payload_consumer/filesystem_verifier_action.cc

By
: Google

Takes
: l.92-100: source and target hash steps: the written slot is verified, not judged by prior content


<a id="aosp-update-attempter-android"></a>
### AOSP update_engine update_attempter_android.cc (android11-release)

Url
: https://android.googlesource.com/platform/system/update_engine/+/refs/heads/android11-release/update_attempter_android.cc

By
: Google

Takes
: l.362-372: cancel path re-sets the current slot active and successful


<a id="android-wahoo-lineage-watchdog-v2"></a>
### LineageOS android_kernel_google_wahoo lineage-22.1: watchdog_v2.c, msm8998.dtsi, msm-pm8998.dtsi

Url
: https://github.com/LineageOS/android_kernel_google_wahoo

By
: Qualcomm / The Linux Foundation (GPL-2.0)

Takes
: the apps watchdog driver and the nodes, read from the local tag reference/android-walleye/4.4.302-g113bd1cfa6f5 (android:PATH@113bd1cfa6f5)


<a id="src-ufs-lineage-wahoo-4-4-302"></a>
### LineageOS android_kernel_google_wahoo 4.4.302 drivers/scsi/ufs/ufshcd.c

Url
: https://github.com/LineageOS/android_kernel_google_wahoo

Takes
: clkscale_enable 0 scales up; simple_ondemand only; wahoo_defconfig governors


<a id="src-ufs-mainline-v7-3-ufshcd"></a>
### Mainline v7.3 drivers/ufs/core/ufshcd.c, host/ufs-qcom.c, devfreq governor_userspace.c

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/drivers/ufs/core/ufshcd.c?h=v7.3

Takes
: clkscale_enable scales up; scale-down drops gear to min_gear G1 via legacy path; userspace governor set_freq


<a id="src-caf-abl-updatecmdline"></a>
### abl_tianocore_edk2 QcomModulePkg/Library/BootLib/UpdateCmdLine.c (CAF, commit 797918c)

Url
: https://gitlab.com/Codeaurora/abl_tianocore_edk2/-/blob/797918c3bf59e2d65b4acb2acdfd6883a0605ee1/QcomModulePkg/Library/BootLib/UpdateCmdLine.c

Author
: Qualcomm / Code Aurora

By
: Qualcomm (BSD 3-clause)

Takes
: How a Qualcomm ABL builds the kernel command line: header words, verity/verifiedbootstate/keymaster, bootdevice, serialno, baseband, display, then ONE conditional block (multi-slot and no root= in the header): slot_suffix, skip_initramfs, rootwait ro init=/init, root path; a stand-in: the Pixel 2's own ABL is closed. Read only, nothing copied


<a id="src-android-wahoo-kernel-ext4-super"></a>
### android_kernel_google_wahoo (lineage-22.1, 113bd1cfa6f5) fs/ext4/super.c: tokens table and token2str

Url
: https://github.com/LineageOS/android_kernel_google_wahoo/blob/lineage-22.1/fs/ext4/super.c

By
: LineageOS

Takes
: Opt_noload has two spellings, norecovery listed first; token2str prints the first, so /proc/mounts says norecovery (l.1207-1208, 1825-1833)


<a id="aosp-init-rc-pstore"></a>
### AOSP system/core rootdir/init.rc (android11-release): pstore mount in on init

Url
: https://android.googlesource.com/platform/system/core/+/refs/heads/android11-release/rootdir/init.rc

By
: AOSP

Takes
: mount pstore at /sys/fs/pstore and chmod console-ramoops (l.357-366); init only mounts it


<a id="src-mainline-v73-gcc-msm8998-ufs-axi"></a>
### Linux v7.3 drivers/clk/qcom/gcc-msm8998.c: ftbl_ufs_axi_clk_src and gcc_ufs_axi_clk

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/drivers/clk/qcom/gcc-msm8998.c?h=v7.3

Takes
: ufs_axi_clk_src's table is 100, 200, 240 MHz - no 50 MHz entry; gcc_ufs_axi_clk (the UFS core_clk) is CLK_SET_RATE_PARENT, so a 50 MHz request rounds/sets to 100 MHz (read at our tree topic/ufs-match-android 0f3429f65ce4)


<a id="src-lineage-wahoo-4-4-302-clock-gcc-8998"></a>
### LineageOS android_kernel_google_wahoo 4.4.302 drivers/clk/msm/clock-gcc-8998.c and msm8998.dtsi ufshc

Url
: https://github.com/LineageOS/android_kernel_google_wahoo

Takes
: Android's ftbl_ufs_axi_clk_src has 50, 100, 200, 240 MHz: the 50 MHz low clock exists there; the ufshc freq-table-hz is <50000000 200000000> as ours (local tag reference/android-walleye/4.4.302-g113bd1cfa6f5)


<a id="src-mainline-v73-devfreq-core"></a>
### Linux v7.3 drivers/devfreq/devfreq.c and governor_userspace.c

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/drivers/devfreq/devfreq.c?h=v7.3

Takes
: set_freq_store -> update_devfreq -> devfreq_update_target (clamp to min/max only) -> devfreq_set_target: previous_freq = the freq the driver's target() hands back; cur_freq_show prints previous_freq when the profile has no get_cur_freq (ufs-qcom's has none)


<a id="src-walleye-init-z14"></a>
### google-walleye-lab build/initramfs-mainline/init-z14: the banner shell init, its watchdog petter that reboots after 90 beats of 2 s

Url
: https://github.com/linux-phone-oco/google-walleye-lab/blob/main/build/initramfs-mainline/init-z14

Year
: 2026

By
: linux-phone-oco

Takes
: the matched UFS image's /init, byte for byte: the 180 s self-reboot, no hold


<a id="src-walleye-ufs-procedure-link-read"></a>
### google-walleye-lab tools/walleye-tries/src/ufs.rs: the UFS try's link read waits for HS-G3 up to boot.device_wait

Url
: https://github.com/linux-phone-oco/google-walleye-lab/blob/main/tools/walleye-tries/src/ufs.rs

Year
: 2026

By
: linux-phone-oco

Takes
: the_link_is_read polls until HS-G3 or device_wait (170 s); hold_command touches /tmp/hold


<a id="src-lwn-qrtr-sensor-manager-2025"></a>
### QRTR bus and Qualcomm Sensor Manager IIO drivers (PATCH 0/3)

Url
: https://lwn.net/Articles/1016590/

Author
: Yassine Oudjana

Year
: 2025

Takes
: On MSM8996/MSM8998 the SSC (SLPI) exposes several services incl. Sensor Manager once the AP provides a Sensor Registry; no service ids are given in the cover letter


<a id="src-pmos-wiki-ssc"></a>
### Qualcomm Snapdragon Sensor Core - postmarketOS wiki

Url
: https://wiki.postmarketos.org/wiki/Qualcomm_Snapdragon_Sensor_Core

Takes
: MSM8998 SSC is a QMI-only Sensor Manager; the SLPI shows as QMI services listed with qrtr-lookup; its example lists only service 66


<a id="src-emainline-ssc-p2"></a>
### Unlocking the Qualcomm Snapdragon Sensor Core - Part 2

Url
: https://emainline.gitlab.io/2022/04/08/Unlocking_SSC_P2.html

Author
: Yassine Oudjana

Year
: 2022

Takes
: Read for service ids: it names none (no 349-362, no Sensor Manager id)


<a id="mainline-pm8941-pwrkey-c"></a>
### drivers/input/misc/pm8941-pwrkey.c (mainline)

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/drivers/input/misc/pm8941-pwrkey.c

Takes
: PON_SUBTYPE 0x05; PON_SUBTYPE_GEN2_PRIMARY 0x04, GEN2_SECONDARY 0x05


<a id="android-msm-qpnp-power-on-c-7-1-0"></a>
### drivers/platform/msm/qpnp-power-on.c (android-7.1.0_r0.2)

Url
: https://android.googlesource.com/kernel/msm/+/android-7.1.0_r0.2/drivers/platform/msm/qpnp-power-on.c

Takes
: QPNP_PON_PERPH_SUBTYPE base+0x05; PON_GEN2_PRIMARY 0x04, PON_GEN2_SECONDARY 0x05; gen2 reasons at 0xC0..


<a id="sdm845-mainline-wip-q6voice"></a>
### sdm845-mainline linux, branch wip/q6voice (ASoC qdsp6 voice: MVM/CVS/CVP)

Url
: https://gitlab.com/sdm845-mainline/linux/-/tree/wip/q6voice

Takes
: in-call audio is not mainline; a q6voice series from msm8916-mainline (Gerhold, Knecht) is carried on sdm845 and sc7280 trees


<a id="q6voiced-daemon"></a>
### q6voiced: userspace daemon opening the voice PCM on oFono call signals

Url
: https://github.com/alexander-pav/q6voiced

Takes
: with the q6voice driver, a daemon starts the hostless voice stream when a call starts; the modem routes call audio itself


<a id="sc7280-mainline-q6voice-pr27"></a>
### sc7280-mainline PR 27: ASoC q6voice for FP5 (Acayan, on v7.0)

Url
: https://github.com/sc7280-mainline/linux/pull/27

Takes
: q6voice ported to a newer kernel and SoC; VoiceMMode1 replaces CS-Voice on newer firmware


<a id="nura-msm8998-wiki"></a>
### Qualcomm Snapdragon 835 (MSM8998) wiki page (postmarketOS wiki mirror)

Url
: https://wiki.nura.eco/wiki/Qualcomm_Snapdragon_835_(MSM8998)

Takes
: modem Partial, audio Works on MSM8998; WLAN userspace needs pd-mapper, tqftpserv, qrtr, rmtfs and diag-router when MSS keeps crashing; GPS works from the modem's PDS


<a id="linux-msm-tqftpserv"></a>
### linux-msm/tqftpserv: TFTP server over QRTR

Url
: https://github.com/linux-msm/tqftpserv

Takes
: serves modem-requested files (wlanmdsp.mbn, calibration) over qrtr


<a id="modemmanager-voice-api"></a>
### ModemManager: org.freedesktop.ModemManager1.Modem.Voice

Url
: https://modemmanager.org/

Takes
: Voice interface handles calls once the modem can register; mmcli drives it


<a id="cepheus-mainline-readme"></a>
### NiHaiden/cepheus_linux_postmarketOS (Mi 9 mainline)

Url
: https://github.com/NiHaiden/cepheus_linux_postmarketOS

Takes
: a recent SoC family case: modem boots, in-kernel pd-mapper, cellular still waits on ModemManager over QMI/qrtr with a SIM


<a id="alpine-aports-modemmanager"></a>
### Alpine aports: community/modemmanager APKBUILD (1.25.95_git20261005)

Url
: https://gitlab.alpinelinux.org/alpine/aports/-/raw/master/community/modemmanager/APKBUILD

Takes
: build flags (polkit=permissive, udev via libgudev, dbus), depends: dbus, polkit, libmm-glib


<a id="modemmanager-meson-options"></a>
### ModemManager meson_options.txt

Url
: https://gitlab.freedesktop.org/mobile-broadband/ModemManager/-/raw/main/meson_options.txt

Takes
: udev, polkit (strict|permissive|no) and qrtr are build options; qrtr default on


<a id="modemmanager-base-manager-qrtr"></a>
### ModemManager src/mm-base-manager.c (QrtrBusWatcher)

Url
: https://gitlab.freedesktop.org/mobile-broadband/ModemManager/-/raw/main/src/mm-base-manager.c

Takes
: a qrtr modem is found by the QrtrBusWatcher, not by udev events; udev stays a hard dependency of the daemon


<a id="alpine-edge-aarch64-index"></a>
### Alpine edge aarch64 APKINDEX (main, community)

Url
: https://dl-cdn.alpinelinux.org/alpine/edge/community/aarch64/

Takes
: installed sizes: modemmanager 6.7 MB, libqmi 5.5 MB, libmm-glib 1.5 MB, glib 5.9 MB, dbus 0.7 MB, eudev 1.1 MB


<a id="modemmanager-mmcli-1-24-2"></a>
### ModemManager 1.24.2 cli/mmcli-manager.c and cli/mmcli-output.c

Url
: https://gitlab.freedesktop.org/mobile-broadband/ModemManager/-/tree/1.24.2/cli

Author
: ModemManager authors

Year
: 2024

By
: freedesktop.org ModemManager

Takes
: mmcli -L prints 4-space PATH [MANUFACTURER] MODEL per modem (output_modem_info, FOUND_ACTION_PREFIX); 'No modems were found' when none (dump_output_list_human); -m takes an index


<a id="mm-base-modem-1-24-2-net-port-check"></a>
### ModemManager 1.24.2 src/mm-base-modem.c organize_ports: QMI modem needs a net port

Url
: https://gitlab.freedesktop.org/mobile-broadband/ModemManager/-/raw/1.24.2/src/mm-base-modem.c

Takes
: line 1642: 'if (qmi && !data_net)' fails with 'Failed to find a net port in the QMI modem'; no switch


<a id="linux-v7-3-ipa-main-msm8998"></a>
### Linux v7.3 drivers/net/ipa/ipa_main.c: qcom,msm8998-ipa -> ipa_data_v3_1

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/drivers/net/ipa/ipa_main.c

Takes
: driver and binding know msm8998-ipa; arch/arm64/boot/dts/qcom has no ipa node for msm8998 (only ipa_fw_mem, ipa_gsi_mem reserved)


<a id="linux-v7-3-interconnect-core"></a>
### Linux v7.3 drivers/interconnect/core.c and bulk.c: no interconnects property gives a NULL path

Url
: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/drivers/interconnect/core.c?h=v7.3

Takes
: of_icc_get returns NULL when the consumer node has no interconnects property; of_icc_bulk_get fails only on an ERR_PTR; icc_set_bw and __icc_enable return 0 on a NULL path


<a id="modemmanager-1-24-2-udev-candidate-rules"></a>
### ModemManager 1.24.2 data/80-mm-candidate.rules and 77-mm-qcom-soc.rules

Url
: https://gitlab.freedesktop.org/mobile-broadband/ModemManager/-/tree/1.24.2/data

By
: ModemManager project

Takes
: SUBSYSTEM==net gets ID_MM_CANDIDATE; DRIVERS==ipa net ports get ID_MM_QCOM_SOC and ID_MM_PHYSDEV_UID=qcom-soc (read from the packaged files of modemmanager 1.24.2)
