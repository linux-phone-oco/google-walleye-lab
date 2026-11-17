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
