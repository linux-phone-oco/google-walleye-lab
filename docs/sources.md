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
