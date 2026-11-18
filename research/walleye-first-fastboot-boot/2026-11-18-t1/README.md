# T1: sysrq c with panic=5 on the alive pk5 (built, not run on the phone)

`~/Projects/walleye-build/images-t1/boot-t1.img` = pk5 (our kernel + porthole's taimen dtb) with two changes: the dtb's ramoops node at the
phone's own region 0xa1100000 (as pk5l2) and the ramdisk = `build/initramfs-mainline/init-t1`: marker `walleye-t1-marker-7f3a` to /dev/kmsg every
second for 20 s (watchdog petted), then `echo c > /proc/sysrq-trigger`. Command line is pk5's, which already has `panic=5`.
Built by `build/build-t1.sh WORK_DIR PK5_BOOT_IMG OUT_DIR`; checks ok, rebuild byte-identical. sha256 in SHA256SUMS.
Reader after the return, phone back as Android with adb root: `tests/host/pstore-t1-read.sh` (pass = the marker in console-ramoops; expected fail = cause B).
