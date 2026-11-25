# initramfs-slpi - the ramdisk of the SLPI image

z21's init and busybox (the USB gadget, the command loop, telnet on 172.16.43.1) plus the phone's own SLPI firmware under
`/lib/firmware/qcom/msm8998/google/walleye/` (`slpi_v2.mdt`, `.b00`-`.b14`, the path `remoteproc_slpi`'s `firmware-name` in
`dt/msm8998-google-walleye.dts` names). The firmware is NOT in git: it is read from the phone's own `vendor_a` read-back
(`debugfs -R 'dump /firmware/FILE OUT' ~/lab-material/walleye/slot-readback-2026-11-20-1/vendor_a.img`, research
walleye-slpi-carveout-read) into `~/lab-material/walleye/slpi-firmware-20261125/` (mode 700, files 600).

Build (a folder of your own, `$AGENT_BUILD/slpi-rd`, holds `init`, `udhcpd.conf`, `spec`, `busybox` (`~/Projects/walleye-build/busybox-arm64`) and `firmware/`):

```sh
lpoco-bringup initramfs build --spec spec --root $AGENT_BUILD/slpi-rd --codec xz --out slpi-ramdisk.cpio.xz
```

Before qrtr-lookup: twice, the same sha256 (348dbd45...f3b). The image is `[images.slpi]` in `device.toml`; `lpoco-bringup image ensure slpi --seed DIR` takes the ramdisk and the dtb from DIR (a folder of those two files).
Ramdisk with `/bin/qrtr-lookup` (item walleye-slpi-boot-procedure, part 1): the spec's `file /bin/qrtr-lookup qrtr-lookup` is the static aarch64 build of the qrtr tool pinned in `rootfs/pins` (commit 29e36ae1, archive sha256 checked), compiled by the host's `aarch64-linux-gnu-gcc` from `src/lookup.c` and `lib/{logging,qmi,qrtr}.c` (the files `src/meson.build` and `lib/meson.build` name) with `-O2 -static -s -ffile-prefix-map=SRC=.`; built twice equal, sha256 in `qrtr-lookup.sha256`, proof `tests/host/qrtr-lookup.sh` (it rebuilds from the pinned archive). Put it in the build folder as `qrtr-lookup` beside `busybox`: the ramdisk is then 59d0cb60...c32c (twice equal; the spec without the qrtr line gives the old 348dbd45...f3b again), the image `slpi` b9819427...65b2.
The kernel has QRTR and QRTR_SMD =y (build/walleye.config), so `qrtr-lookup` needs no module. NOT KNOWN YET: that it lists services on the phone (that is the procedure's read).
Same on this host only: the compiler and static glibc are the host's, not pinned (as rootfs/pins says); `RAMDISK=FILE sh tests/host/qrtr-lookup.sh` also compares the binary inside a ramdisk to the declared sha256.
