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

Twice, the same sha256 (348dbd45...f3b). The image is `[images.slpi]` in `device.toml`; `lpoco-bringup image ensure slpi --seed DIR` takes the ramdisk and the dtb from DIR (a folder of those two files).
Not here yet: the qrtr tools (`qrtr-lookup`): the first try reads `/sys/class/remoteproc/*/state` and the kernel log from the shell.
