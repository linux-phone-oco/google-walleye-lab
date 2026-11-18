# T3: an unpetted kpss-wdt bite on the alive pk5 (built, not run on the phone)

`~/Projects/walleye-build/images-t3/boot-t3.img` = pk5 (our kernel + porthole's taimen dtb) with two changes: the dtb's ramoops node at the
phone's own region 0xa1100000 (as T1) and the ramdisk = `build/initramfs-mainline/init-t3`: marker `walleye-t3-marker-5c1e` to /dev/kmsg, then
busybox `watchdog -T 10 -t 3600 -F /dev/watchdog` (10 s timeout, one ping, the next an hour away), so the SoC watchdog bites at ~10 s. No sysrq c.
The dtb already has `watchdog@17817000` (qcom,apss-wdt-msm8998, kpss-wdt, timeout-sec 30), so no dtb node is added. Command line is pk5's (panic=5).
Built by `build/build-t3.sh WORK_DIR PK5_BOOT_IMG OUT_DIR`; checks ok, rebuild byte-identical. sha256 in SHA256SUMS.
Reader after the return, phone back as Android with adb root: `tests/host/pstore-t3-read.sh` (pass = the marker in console-ramoops; hoped pass: a bite is a warm path).
