# The osfact NFC image's ramdisk (item walleye-osfact-image-nfc)

The ramdisk part of `osfact pack` for the osfact walleye image that carries NFC: slpi's init **unchanged** (gadget, USB network, telnet on
172.16.43.1:23, the 180 s window, the SoC watchdog's petter) plus `/bin/walleye-nfc-up`, the nfc0 opener of the NFC image (research
walleye-nfc-poll-in-tool-built). The root (Alpine) is not in it: osfact pushes it and enters it. The NFC open is the host's, over the telnet shell,
as the NFC boot procedure does it (`walleye-nfc-up --index 0 --poll 10`, procedures/steps/src/nfc_boot.rs): the image only has to carry the tool.

| File | What |
|---|---|
| `init` | `build/initramfs-slpi/init`, byte for byte (tests/host/osfact-nfc-ramdisk.sh refuses any difference) |
| `spec` | the cpio spec: busybox (arm64, `build/initramfs-modem/tools.sha256`), init, `/bin/walleye-nfc-up`, udhcpd.conf; no firmware (NFC needs none) |

The other two parts are the NFC image's own: the kernel `Image-nfc.gz` (the UFS-off list, tag `int/walleye/ufs-off/v7.3/1`, NFC built in: sha256 78dbbff6,
`images.nfc.recipe` kernel) and the dtb `msm8998-google-walleye-nfc.dtb` (the nfc@28 node in the OnePlus 5 form: sha256 86550c1e, `images.nfc.recipe` dtb),
placed as `parts/kernel` and `parts/qcom/msm8998-google-walleye.dtb`. Build, from a folder `ROOT` that holds `busybox` (the recorded arm64 build), `init`,
`udhcpd.conf` (`build/initramfs-slpi/udhcpd.conf`) and `walleye-nfc-up` (static aarch64, ab2cf4c1):

    lpoco-bringup initramfs build --spec build/initramfs-osfact-nfc/spec --root ROOT --codec xz --out RAMDISK
    RAMDISK=RAMDISK NFC_RAMDISK=the NFC image's ramdisk (f4b6c1d7) sh tests/host/osfact-nfc-ramdisk.sh
    osfact pack now --device device.toml --distro alpine.toml --parts PARTS --out OUT     # PARTS: kernel, ramdisk, qcom/msm8998-google-walleye.dtb

Kept outside git in `~/Projects/walleye-build/osfact-nfc/` (parts, alpine.toml, SHA256SUMS).

NOT KNOWN YET: that nfc0 appears under osfact's chroot (the proof reads /sys/class/nfc/nfc0 on the booted image; the root's own /sys is bound by `[install.root] binds`);
the controller's clock and the pins' drive strength (research walleye-nfc-clock-and-minus-121). The tag poll stays parked (decision 2026-11-28-walleye-nfc-tag-poll-waits-no-tag).
