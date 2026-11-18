# Building the first walleye image

Everything here builds the first thing the phone is tried with: mainline v7.3 + our own `dt/msm8998-google-walleye.dts`, a
small initramfs with a USB gadget, a v0 boot image. Nothing here touches the phone (no fastboot, no adb); the keeper boots
what is built, once, with `lpoco lab fastboot boot now`.

Builds run on the lab machine through oco-agent (one `oco-agent run` at a time per name), in `~/Projects/walleye-build`:

```sh
# the kernel tree: a worktree of the shared kernel clone at the tag (the tag fetched once from kernel.org)
git -C ~/Projects/surya-audio/linux fetch --no-tags --depth=1 https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git tag v7.3
git -C ~/Projects/surya-audio/linux worktree add --detach ~/Projects/walleye-build/linux-v7.3 v7.3

build/build-kernel.sh   ~/Projects/walleye-build/linux-v7.3 ~/Projects/walleye-build/out   # Image.gz + dtb, clang
build/build-busybox.sh  arm64  ~/Projects/walleye-build                                    # static busybox 1.36.1
build/build-initramfs.sh phone ~/Projects/walleye-build ~/Projects/walleye-build/initramfs-phone.cpio.gz
build/build-bootimg.sh  out/Image.gz out/msm8998-google-walleye.dtb initramfs-phone.cpio.gz images   # boot-a.img, boot-b.img
```

`build/selftest-initramfs.sh WORK_DIR` runs the init under QEMU (x86_64 stand-in, the host kernel's `dummy_hcd` as the USB
wire) and passes only if the gadget gets a lease, answers a ping and a telnet command on 172.16.42.1.

Two boot images, because walleye's own stock boot.img header is not read yet: `boot-a.img` has taimen's offsets (ramdisk
0x1000000, tags 0x100), `boot-b.img` LineageOS's (ramdisk 0x2000000, tags 0x1e00000); base 0, kernel 0x8000, page 4096 in both.
`build/check-bootimg.sh` counts what each must be (magic, size within 32 MiB, offsets, dtb with `qcom,msm-id` 0x124/0x20001
and no bootargs). Once the keeper has read the stock header, the set that matches it is the one to try.

## Variants c and d (after the first two tries)

`boot-c.img` and `boot-d.img` are `a` and `b` with the header's os_version 8.1.0 and os_patch_level 2020-01 (the first two carry
none); only those four header bytes differ, kernel, dtb, initramfs and command line are the same. Rebuild only the images with
`KEEP_KERNEL=1 build/build-all.sh` (the old images stay byte for byte). `images/DIFFERENCES.txt` has the one-line table.

## Variants e and f (after the stock boot.img was read)

LineageOS 22.2's own walleye boot.img (20261113, sha256 48dbc55c...7468) has header v0, base 0, kernel 0x8000, ramdisk 0x1000000,
tags 0x100, second 0, page 4096, one dtb appended to an lz4 kernel with `qcom,msm-id <0x124 0x20001>` and no `qcom,board-id`
or `qcom,pmic-id` (those live in the dtbo table, id 0x1bc; the kernel is told `androidboot.dtbo_idx=3`). Image `a` already has
those offsets. `boot-e.img` = a with the command line cut to earlycon + console (its `--second_offset 0` changes nothing:
mkbootimg writes second address 0 for an empty second, a's header has it). `boot-f.img` = e with `dt/msm8998-google-walleye-f.dts`
(`qcom,board-id <8 0>`, what the stock dtb's own `qcom,mtp` means; the stock dtb has no board-id at all). The os word stays 0.
Pass the f dtb as the fifth argument of `build-bootimg.sh` (`build-all.sh` does).

## Variants g, h and i (walleye's own address)

Walleye and surya share one host, and surya's pmOS gadget answers on 172.16.42.1, so lpoco lab must tell the phones apart; the manifest
`[connection] addresses` already says 172.16.43.1. `boot-g.img` = e and `boot-h.img` = f with the initramfs built `NET=43`
(`build-initramfs.sh` rewrites 172.16.42 to 172.16.43 in the init, udhcpd.conf and the self-test, and the ECM MAC byte 2a to 2b,
into `WORK_DIR/net43/`; at the default NET=42 the files are used as they are, so a-f stay byte for byte). Only the ramdisk differs
from e, b and f. `check-bootimg.sh` now also counts `ifconfig usb0 172.16.NET.1` in /init and the lease in udhcpd.conf (`NET=43` for g, h, i).
`NET=43 build/selftest-initramfs.sh WORK_DIR` is the QEMU proof at 172.16.43.1.

## The bisect images s0, s0n, s1, s2, s3 (stock to ours, one swap each)

e, b (and g, h, i) are accepted by the bootloader ("Booting OKAY") from the first state but never start; the stock control boots.
`build/build-bisect.sh CONTROL_BOOT_IMG OUR_IMAGE OUR_DTB OUR_RAMDISK_43 OUT_DIR` unpacks the control and packs its own parts
again with its own header fields and its AVB hash footer (avbtool, algorithm NONE, padded to 32 MiB, same properties), swapping
one part: `s1` ramdisk = ours (172.16.43.1), `s2` kernel = our raw Image in an lz4 frame (as the stock one), `s3` dtb = ours.
`s0` swaps nothing (differs from the control in the header id and the footer digest only: 53 bytes) and `s0n` is s0 without footer
and padding (the shape of a-i). `build/check-bisect.sh` counts per image that exactly the swapped part differs and the footer verifies.
What differs between stock and ours that the bootloader could read: no AVB footer and no 32 MiB padding (ours 16 MB); os version
and patch level 0 (stock 15.0.0 / 2026-11); kernel gzip (stock lz4 frame); dtb of 51 KB with no /chosen bootargs (stock 376 KB,
bootargs "rcupdate.rcu_expedited=1"); command line (stock has androidboot.* and androidboot.dtbo_idx=3); ramdisk gzip (stock xz).
Equal: header v0, offsets, page size, one appended dtb, msm-id 0x124/0x20001, no board-id/pmic-id.

## vendor-gadget.img (the vendor-kernel-first try)

`~/Projects/walleye-build/images/vendor-gadget.img` is the bisect image s1 (the control's boot.img with only the ramdisk swapped to
our gadget initramfs, 172.16.43.1; kernel, dtb, command line, header, AVB footer as the control's). `build/check-bisect.sh` counts
that. The proof that it worked on the phone is `tests/host/gadget-up.sh` (run on the lab host; `GADGET_WAIT=90` waits for the gadget):
it passes only when a USB device with our gadget strings enumerates and 172.16.43.1 answers ping and telnet; with the phone in
fastboot it fails.

## vendor-gadget2.img (why vendor-gadget.img panicked, and the fix)

The stock kernel's own config (read from its IKCFG data in the control's Image) has `CONFIG_RD_GZIP` **not set**, `RD_XZ=y`,
`DEVTMPFS` not set, `USB_CONFIGFS_NCM=y` and no ACM/ECM/RNDIS. v1's ramdisk was gzip, so the kernel could not unpack it and had
no `/init`: the panic. `build/build-vendor-gadget.sh WORK_DIR CONTROL_BOOT_IMG OUT_DIR` makes `initramfs-vendor.cpio.xz` (xz, CRC32,
as the stock ramdisk; init and nodes from `build/initramfs-vendor/`: no mount over /dev, /dev nodes from the cpio, one NCM function,
172.16.43.1, telnetd, an init that never ends) and `vendor-gadget2.img` (the control with only that ramdisk swapped, AVB footer
as the control's), and counts ten checks. Proof on the phone is again `tests/host/gadget-up.sh`. Not run on arm64 (no qemu-aarch64 on
the lab machine; installing one is a change): checked by unpack, `file`, `sh -n`, and the applets present in busybox-arm64.

## vendor-gadget3.img (vendor-gadget2 plus a way back without hands)

Only the ramdisk's `/init` changes (`build/initramfs-vendor3/init`); `build/build-vendor-gadget3.sh WORK_DIR CONTROL_BOOT_IMG OUT_DIR
[VENDOR2_RAMDISK]` makes `initramfs-vendor3.cpio.xz` and `vendor-gadget3.img` and counts fourteen checks, one of them that every other
file in the ramdisk is vendor-gadget2's (given its xz). New in init: a marker written to `/dev/kmsg` before the first mount (the node is in
the cpio), and a background timer started first: `sleep 240`, skipped if `/tmp/hold` exists (`touch /tmp/hold` from the telnet shell keeps
the phone up), then `reboot -f`, and `echo b > /proc/sysrq-trigger` five seconds later if that returned. The stock kernel's config has
`CONFIG_MAGIC_SYSRQ=y` with `MAGIC_SYSRQ_DEFAULT_ENABLE=0x1` (all functions), `PSTORE`, `PSTORE_RAM` and `PSTORE_CONSOLE`, so the sysrq
reboot works and a later boot's init lists `/sys/fs/pstore` (it already did). Not run on the phone: that is the keeper's try, proof
`tests/host/gadget-up.sh`, and the phone must return to fastboot/Android by itself after 240 s.

## vendor-gadget4.img (an idle init: does the vendor kernel panic without our gadget?)

Only the ramdisk's `/init` changes (`build/initramfs-vendor4/init`): a kmsg marker before any mount, `mount proc`, an "alive N s" marker every 30 s,
at 120 s `reboot -f` then `echo b > /proc/sysrq-trigger`. No configfs, USB, network or telnetd. `build/build-vendor-gadget4.sh WORK_DIR CONTROL_BOOT_IMG
OUT_DIR [VENDOR3_RAMDISK]` makes `initramfs-vendor4.cpio.xz` and `vendor-gadget4.img` and counts fourteen checks (header, kernel blob, size and AVB
footer are the control's; the ramdisk differs from vendor-gadget3's only in `/init` and the unused `etc/udhcpd.conf`). Not run on the phone: the keeper's try.

## vendor-gadget5a/b/c.img (the gadget bring-up bisect: which step panics?)

gadget4's idle init returns by itself at 120 s, gadget2/3 (the full gadget) panic about 75 s in, so the panic is in the gadget bring-up. Three images, each
gadget4's init plus one more step, everything else byte-identical (the control's kernel, dtb, command line, header, AVB footer; ramdisk differs from gadget4's only in `/init`):
`5a` mounts sysfs and configfs and creates the gadget dir with idVendor/idProduct and strings (serial `walleye-first-boot`, product `Pixel 2 walleye (mainline)`), no function;
`5b` adds the `ncm.usb0` function and the config symlink, no UDC bind; `5c` adds the UDC bind (first of `/sys/class/udc`), `usb0` at 172.16.43.1 and `telnetd`.
Each init keeps gadget4's kmsg marker before any mount, alive markers every 30 s and the self-reboot at 120 s (`reboot -f`, then sysrq b). `build/build-vendor-gadget5.sh WORK_DIR
CONTROL_BOOT_IMG OUT_DIR [VENDOR4_RAMDISK]` builds all three and counts fourteen checks each (as gadget4's, plus: the init holds exactly its step's parts). Not run on the phone: the keeper's tries.

## First light images ml1 and ml2 (our mainline kernel)

`build/build-kernel.sh KERNEL_TREE OUT_DIR` (into a new OUT_DIR, e.g. `~/Projects/walleye-build/out-ml`: the older images stay reproducible from `out`) now also
builds in the SoC watchdog (`QCOM_WDT`; the dts has a `watchdog@17817000` node, address and bark interrupt from the downstream MSM8998 tree, not measured on this phone).
`build/build-mainline-light.sh WORK_DIR CONTROL_BOOT_IMG KERNEL_OUT_DIR OUT_DIR` packs the control's header and AVB footer around our kernel, our -@ dtb and
`build/initramfs-mainline/init-ml` (mass_storage window, serial walleye-first-boot, product "Pixel 2 walleye (mainline)"; opens /dev/watchdog and never pets it).
`boot-ml1.img` has the kernel as Image.gz, `boot-ml2.img` as an lz4 frame like the stock one. Command line: `earlycon=msm_serial_dm,0xc1b0000 clk_ignore_unused
ignore_loglevel panic=5` plus the stock `androidboot.*` words. `tests/host/mainline-light-up.sh` is the host proof: a USB device with our strings.

## pk1 and pk2: porthole's kernel on this phone (known-good step)

`build/pk-kernel.sh WORK_DIR 0d62af2af5e72333a4d0e635f8bf338f191d4cc3` fetches porthole-dev/pmaports (branch taimen-bringup, that commit) and kernel.org
v7.2.2, checks the tarball, the 262 numbered patches and the config against the APKBUILD's sha512sums, applies the series and builds
`Image.gz` and the taimen and walleye dtbs (`-@`) with clang. `build/build-pk.sh KERNEL_TREE KERNEL_OUT ML5A_BOOT_IMG DTBO_IMG OUT_DIR` then packs
them into ml5a's image: **only the kernel blob differs** (header, command line, ramdisk with init-ml5a, AVB footer are ml5a's). `pk1` = porthole's
taimen dtb, `pk2` = their walleye dtb (patch 0029) plus `qcom,msm-id <0x124 0x20001>` (their walleye file has none; the control dtb's id). Both carry
the empty stub nodes for the vendor overlay's labels (as ours, `gen-overlay-stubs.sh`), so the bootloader's dtbo entry 3 merges (checked with
`fdtoverlay`). Not taken from porthole: their command line (ml5a's is kept, so only kernel and dtb differ), their stub dtbo (never written).
Licences: GPL-2.0-only kernel and patches, kept as published; nothing of them is copied into this repository.

## pk3 and ml6a: pk1 and ml5a without earlycon (is the gated debug UART the hang?)

`build/build-nocon.sh PK1_BOOT_IMG ML5A_BOOT_IMG OUT_DIR` repacks pk1 and ml5a with **only the kernel command line changed**: `earlycon=msm_serial_dm,0xc1b0000`
removed (no `console=` word was there; `androidboot.console=ttyMSM0` is a stock androidboot word and stays). Kernel blob, ramdisk, header words and AVB footer
props are the source's (11 checks per image). Rationale and expected return times: research `walleye-pk3-ml6a-built`. Not run on the phone: the keeper's try.

## pk4, pk5, pk6: bisect porthole's kernel against ours (one swap each, no earlycon)

`build/build-pk456.sh PK3_BOOT_IMG ML6A_BOOT_IMG ML5B_BOOT_IMG OUT_DIR` swaps one part at a time between pk3 (porthole's kernel + taimen dtb) and ml6a
(our kernel + our dtb): `pk4` = porthole's Image.gz + OUR dtb; `pk5` = OUR Image.gz + porthole's taimen dtb; `pk6` = pk3 with only the ramdisk of ml5b (3 s per
score point). Header, command line (no earlycon) and AVB footer are pk3's; pk4 and pk5 carry ml5a's reboot-at-once init. 31 checks, rebuild byte-identical.
Not run on the phone: the keeper's try `walleye-try-bisect-pk`.

## d1..d6: which node of our dtb stops boot (pk4 with one node group removed)

`build/build-dtb-bisect.sh PK4_BOOT_IMG PK5_BOOT_IMG DTBO_IMG OUT_DIR` takes pk4 (porthole's kernel + our dtb, dead) and removes one group of nodes from our dtb with `fdtput -r`:
`d1` watchdog, `d2` ramoops, `d3` slpi carve-out, `d4` RPM regulators, `d5` usb@a8f8800 + qusb2 phy, `d6` = d1+d2+d3. Everything else is pk4's; the stock dtbo overlay still merges
(spmi is not a group: the overlay needs it). 90 checks, rebuild byte-identical. Not run on the phone: the keeper's try `walleye-try-dtb-bisect`.

## pk5l: first mainline userspace (ACM console + log window)

`build/build-pk5l.sh WORK_DIR PK5_BOOT_IMG OUT_DIR` repacks pk5 (our kernel + porthole's taimen dtb) with **only the ramdisk changed**: `initramfs-mainline/init-pk5l` binds one configfs gadget
with `acm.GS0` (a respawned `busybox ash` on /dev/ttyGS0) and `mass_storage.0` (a 2 MiB window rewritten every 5 s), pets the SoC watchdog for 180 s (longer while `/tmp/hold` exists) and reboots.
39 checks incl. the kernel config's gadget options. Host test: `tests/host/mainline-acm-up.sh`. Not run on the phone: the keeper's try `walleye-try-pk5l`.

## pk5l2 (ramoops in the phone's own region)

`build/build-pk5l2.sh PK5L_BOOT_IMG OUT_DIR` edits only the ramoops node of the dtb in pk5l's kernel blob (`fdtput`: reg 0xa1100000/2 MiB, record 0x80000, console 0x20000, pmsg 0), so a panic that ends pk5l early is readable from Android's pstore. 20 checks; hashes in `research/walleye-first-fastboot-boot/2026-11-18-pk5l2/`.

## w1..w5, wr, wa: pk5 made walleye-like, group by group (the bisect from the alive side)

`build/build-taimen-walleye.sh PK5_BOOT_IMG PK2_DTB OUT_DIR` takes pk5 (our kernel + porthole's TAIMEN dtb, reaches init) and puts in what the walleye dtbs (pk2 and ours) have
that the taimen dtb does not: `w1` camss + cci disabled and their camera children removed, `w2` dma-controller@17184000, iommu@5100000, video-codec, slim-ngd disabled (slim@1 removed),
`w3` the five i2c masters disabled and their children (touchscreen, vibrator, codec, nfc, pmic, proximity) removed, `w4` pcie + its phy disabled, `w5` the pon resin key and the
pmic@3 led-controller disabled (led-0 removed), `wr` the nodes only taimen has removed (fingerprint, gpio-keys, seven camera regulators, sound, skin thermal, the audio/camera pin states and dais;
four pinctrl states the overlay needs become empty stubs, as in pk2), `wa` all of them: its node set and every status equal pk2's (checked). Everything else is pk5's; the stock dtbo overlay still merges.
399 checks, rebuild byte-identical. Why these and not overlay labels: research `walleye-overlay-enables-what-result`. Not run on the phone: the keeper's try `walleye-try-taimen-walleye`.
