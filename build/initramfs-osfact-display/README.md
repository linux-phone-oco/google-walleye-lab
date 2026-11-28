# The osfact display image's ramdisk (item walleye-osfact-image-display)

The ramdisk part of `osfact pack` for the osfact walleye image that lights the glass: slpi's init (gadget, USB network,
telnet on 172.16.43.1:23, the 180 s window, the SoC watchdog's petter) plus **one line**, `/bin/display-load &`, which
loads the 13 display modules at 5 s exactly as b-a5's init did (`build/initramfs-ramroot/init`, step 1) and ends with
`dmesg -n 3` so the getty prompt stays the last row. The root (Alpine) is not in it: osfact pushes it and enters it;
`device.toml` `[install.root] binds` gives it /dev (tty1), so its inittab's `getty tty1` writes to the glass.

| File | What |
|---|---|
| `init` | `build/initramfs-slpi/init` plus the one `display-load` line (tests/host/osfact-display-ramdisk.sh refuses any other difference) |
| `display-load` | the loop of b-a5's init, on its own |
| `spec` | the cpio spec: busybox (arm64, `build/initramfs-modem/tools.sha256`), init, display-load, `/display-delay`, `/display-modules.list`, the 13 modules |

The kernel, the dtb and the delay, list and modules are b-a5's (boot-b-a5.img, sha256 83b9c4c6, the image whose rootfs v5 showed the
prompt on the glass). Build, from a folder `ROOT` that holds `busybox` (the recorded arm64 build), `init`, `display-load`, `udhcpd.conf`
(`build/initramfs-slpi/udhcpd.conf`) and b-a5's ramdisk unpacked beside (`display-delay`, `display-modules.list`, `lib/modules`):

    lpoco-bringup initramfs build --spec build/initramfs-osfact-display/spec --root ROOT --codec xz --out RAMDISK
    RAMDISK=RAMDISK BA5_RAMDISK=b-a5's ramdisk sh tests/host/osfact-display-ramdisk.sh
    osfact pack now --device device.toml --distro alpine.toml --parts PARTS --out OUT     # PARTS: kernel, ramdisk, qcom/msm8998-google-walleye.dtb

Kept outside git in `~/Projects/walleye-build/osfact-display/` (parts, SHA256SUMS).

NOT KNOWN YET: the command line. osfact packs `device.toml` `[boot] cmdline`, which lacks b-a5's `iommu.passthrough=1` (the dyndbg
words are diagnostic only). If the glass stays dark or the gadget is lost with the shell kept, that is the first suspect, and the
fix is a per-image command line in osfact (lab-tools), not a change of `[boot]`.
