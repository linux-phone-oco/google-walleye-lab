Stock Android's SMMU setup for the display and the modem, read off the host (2026-11-28).
Source of the dts pieces: the dtb appended to boot_a.img in ~/lab-material/walleye/slot-readback-2026-11-20-1 (the phone's own stock boot image; `adb shell` showed the same node names under /proc/device-tree, values not readable as shell).
The .c files are excerpts of LineageOS android_kernel_google_wahoo (GPL) and of kernel.org master, kept to cite lines.
