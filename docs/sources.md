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


<a id="src-wahoo-msm-poweroff"></a>
### msm-poweroff.c, kernel/msm android-msm-wahoo-4.4-android11 (vendor restart path)

Url
: https://android.googlesource.com/kernel/msm/+/refs/heads/android-msm-wahoo-4.4-android11/drivers/power/reset/msm-poweroff.c

By
: Google/Qualcomm

Takes
: msm_restart_prepare: PMIC reset type is WARM only if download mode is on (panic sets it) or the reboot command is non-empty; a plain reboot is a PMIC HARD reset, 'unless memory contents must be maintained'; panic path can bite the watchdog; PS_HOLD de-asserted through TZ


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
