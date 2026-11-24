# build - scripts that build the Pixel 2's kernel, initramfs and boot images

They run on the lab machine and write into `~/Projects/walleye-build/` (outside git); `device.toml [images.*]`
says which image is which. One script per try grew here (about 175); once test kernels come from `int/walleye`
tags (docs decision 2026-11-23-structure-kernel-topics-and-integration), one image build replaces them.

## Building the first walleye image

Everything here builds the first thing the phone is tried with: mainline v7.3 + our own `dt/msm8998-google-walleye.dts`, a
small initramfs with a USB gadget, a v0 boot image. Nothing here touches the phone (no fastboot, no adb); the keeper boots
what is built, once, with `lpoco lab fastboot boot now`.

Builds run on the lab machine through oco-agent (one `oco-agent run` at a time per name), in `~/Projects/walleye-build`:

```sh
# the kernel tree: a worktree of the shared kernel clone at the tag (the tag fetched once from kernel.org)
git -C ~/Projects/linux-phone-oco/linux fetch --no-tags --depth=1 https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git tag v7.3
git -C ~/Projects/linux-phone-oco/linux worktree add --detach ~/Projects/walleye-build/linux-v7.3 v7.3

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
`lpoco-bringup image bisect build/series/stock-to-ours.toml --out DIR` swaps one part of the control's boot.img per image (header fields and AVB hash footer
kept: algorithm NONE, padded to 32 MiB, same properties), as the series declares: `s1` ramdisk = ours (172.16.43.1), `s2` kernel = our raw Image in an lz4 frame (as the stock one), `s3` dtb = ours.
`s0` swaps nothing (differs from the control in the header id and the footer digest only: 53 bytes) and `s0n` is s0 without footer
and padding (the shape of a-i). The command counts per image that exactly the declared part differs and the footer verifies; the five images are byte-equal to the ones first built (sha256 in `bisect/SHA256SUMS`).
What differs between stock and ours that the bootloader could read: no AVB footer and no 32 MiB padding (ours 16 MB); os version
and patch level 0 (stock 15.0.0 / 2026-11); kernel gzip (stock lz4 frame); dtb of 51 KB with no /chosen bootargs (stock 376 KB,
bootargs "rcupdate.rcu_expedited=1"); command line (stock has androidboot.* and androidboot.dtbo_idx=3); ramdisk gzip (stock xz).
Equal: header v0, offsets, page size, one appended dtb, msm-id 0x124/0x20001, no board-id/pmic-id.

## vendor-gadget.img (the vendor-kernel-first try)

`~/Projects/walleye-build/images/vendor-gadget.img` is the bisect image s1 (the control's boot.img with only the ramdisk swapped to
our gadget initramfs, 172.16.43.1; kernel, dtb, command line, header, AVB footer as the control's). `lpoco-bringup image bisect` counts that. The proof that it worked on the phone is `tests/host/gadget-up.sh` (run on the lab host; `GADGET_WAIT=90` waits for the gadget):
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

`lpoco-bringup image bisect build/series/pk456.toml --out DIR` swaps one part at a time between pk3 (porthole's kernel + taimen dtb) and ml6a
(our kernel + our dtb): `pk4` = porthole's Image.gz + OUR dtb; `pk5` = OUR Image.gz + porthole's taimen dtb; `pk6` = pk3 with only the ramdisk of ml5b (3 s per
score point). Header, command line (no earlycon) and AVB footer are pk3's; pk4 and pk5 carry ml5a's reboot-at-once init. The command counts each image and the three are byte-equal to the first build (`images-pk456/SHA256SUMS`).
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

## wp1..wp5, wpa: wa plus one group of pk2's property differences (the property-level bisect)

`build/build-property-bisect.sh WA_BOOT_IMG PK2_DTB OUT_DIR` takes wa (alive: node set and statuses of the walleye dtb) and copies in one group of what still differs from pk2's walleye dtb (dead), by property:
`wp1` root compatible + model, `wp2` battery capacity (/battery, fuel-gauge), `wp3` panel compatible removed, `wp4` `qcom,protection-domain` on the seven apr services, `wp5` leftovers in nodes disabled in both (camss supplies, pcie reg/ranges/supply, resin linux,code), `wpa` all five: every property line equals pk2's (`build/dts-norm.awk` resolves phandles to node paths so renumbered phandles do not count). Nothing differs in /memory, reserved-memory, chosen, cpus, timer, psci. 68 checks, rebuild byte-identical; diff and hashes in `research/walleye-first-fastboot-boot/2026-11-18-property-diff/`. Not run on the phone: the keeper's try `walleye-try-property-bisect`.

## g1..g3: the mainline gadget in stages on the alive base (pk5)

`build/build-pk5g.sh WORK_DIR PK5_BOOT_IMG OUT_DIR` repacks pk5 (our kernel + porthole's taimen dtb, alive) with only the ramdisk changed: one init, `initramfs-mainline/init-pk5g`, the stage is the number in `/stage` (the only file that differs).
`g1` configfs gadget dir + strings (serial walleye-first-boot, product "Pixel 2 walleye (mainline)") and nothing else; `g2` = g1 + `mass_storage.0` (2 MiB file LUN, ro) linked into config c.1, no UDC bind; `g3` = g2 + UDC bind to the first `/sys/class/udc` entry
(the host should see 1d6b:0104 within ~25 s of the jump: the first mainline light, plain `lsusb`). Every init pets /dev/watchdog each second, holds 60 s, then `reboot -f` (sysrq b behind it); no ACM, no shell loop.
Expected returns: alive (init ran) ~108 s, dead ~75 s, early panic ~45-60 s. 58 checks, rebuild byte-identical; hashes in `research/walleye-first-fastboot-boot/2026-11-18-pk5g/`. Not run on the phone: the keeper's try `walleye-try-pk5-stages`.

## z2: z1 with the GPU clock controller built in (walleye-dmesg-clean)

The first mainline dmesg ended with `arm-smmu 5040000.iommu: probe with driver arm-smmu failed with error -110`: that SMMU's power domain is gpucc's `GPU_GX_GDSC`, and
arm64 defconfig has `CONFIG_MSM_GPUCC_8998=m` while the initramfs carries no modules, so the domain never appeared. `build/walleye.config` now says `=y` (and `build-kernel.sh`
refuses a kernel without it). `build/build-z2.sh WORK_DIR Y2_BOOT_IMG KERNEL_OUT_DIR OUT_DIR` makes `boot-z2.img` = y2's image with the dtb of `dt/` now (z1's) and the new
Image.gz, ramdisk, header, command line and AVB footer y2's; 19 checks, among them that the new `.config` differs from y2's kernel's in the gpucc line only.
`tests/host/dmesg-clean-check.sh DIR` reads the shell reader's output and passes only when `devices_deferred` is empty and the dmesg has no `-110` probe timeout and no
`could not find phandle`; `dmesg-clean-selftest.sh` proves it on the z6, z7 and c2 reads a phone printed (`dmesg-clean-typed-selftest.sh` keeps the -110 and phandle cases, which no phone has printed yet). z2 does not touch the other two faults (the overlay's `serial@c16f000` waiting on the
`clock_gcc` stub, the overlay's phandles): one change per image, those come next.

## z3: z2 with the overlay's clock supplier on the real gcc (walleye-dmesg-clean)

The serial's `wait for supplier /vendor-overlay-stubs/clock_gcc` and the four `could not find phandle -...` lines (i2c@c179000, bt_wcn3990, serial@c16f000, mnh-sm) are one fault:
the stock overlay's `clocks = <&clock_gcc ID ...>` land on our empty `clock_gcc` stub, which has no `#clock-cells`, so the clock ID is read as a phandle. `build/build-z3.sh Z2_BOOT_IMG OUT_DIR`
makes `boot-z3.img` = z2 with only `/__symbols__/clock_gcc` set (fdtput) to `/soc@0/clock-controller@100000`; the overlay merge then writes gcc's phandle into those clock users.
15 checks, rebuild byte-identical; `tests/host/overlay-clock-gcc.sh` (all four clock users) fails on z2's dtb and passes on z3's. The try is `walleye-try-z3` (it supersedes z2's).

## z4: z3 with an ACM shell that survives the host (walleye-shell-reliable)

z3's try saw a live phone and an empty reader: the init ran `ash -i < /dev/ttyGS0`, which ends when the host closes the port, and the reader opened the node again for every command, so
each command hit a dead or just-respawned shell. Two changes. `build/initramfs-mainline/init-z4` (pk5l's init with only the console block changed) opens the port once (fd 4), sets it
`raw -echo -hupcl`, runs `ash -i` in its own session (`setsid`; no controlling terminal), retries a failed open, respawns at once and says every start in the kmsg (`shell on /dev/ttyGS0, start N`).
`tests/host/mainline-shell-read.sh` opens the tty once (fd 3) and never closes it before the end (no probe open, no per-command open; stty and drain go through the fd).
`build/build-z4.sh WORK_DIR Z3_BOOT_IMG OUT_DIR` makes `boot-z4.img` = z3 with only the ramdisk's init changed (packing init-pk5l first reproduces z3's ramdisk byte for byte; rebuild identical).
`tests/host/mainline-shell-selftest.sh` has a phone that hangs up when its port is closed (socat `pty-wait-slave` + a respawn loop): the one-fd reader passes it, the old reader
(`tests/host/fixture-shell-read-per-command.sh`) fails it. Not measured yet: the init's console block on the phone itself (`setsid` and `-hupcl` on ttyGS0 are read from busybox, not run); the try is `walleye-try-z4`.

## banner: y2 with a banner init (walleye-build-banner-init)

y2 and z1 were both silent to the reader. `build/build-banner.sh WORK_DIR Y2_BOOT_IMG OUT_DIR` makes `boot-banner.img` = y2's image (our kernel, our dtb, header, command line, AVB footer) with
**only the ramdisk changed**: `initramfs-mainline/init-banner` binds one ACM function and, from the moment `/dev/ttyGS0` exists, writes `WALLEYE-BANNER beat=N up=..s steps=.. last=[..]` to it every 2 s from its
own subshell (a blocked write stalls nothing), and puts one `walleye-banner: step N: ...` line on `/dev/kmsg` before each step. Watchdog petted every 2 s by its own subshell, `reboot -f` after ~180 s.
The try is a plain `cat /dev/ttyACM0 > FILE` (raw bytes, no reader), then `sh tests/host/banner-check.sh FILE`; `banner-selftest.sh` proves the check on made-up captures. If the banner bytes show under
plain `cat`, the old reader was the problem; if not, the silence is on the phone or the host (ModemManager and who holds the port: see item walleye-reader-eacces-retry).

## z20: the diagnostic kernel (walleye-build-gpio-diag-kernel)

`EXTRA_CONFIG=$PWD/build/walleye-diag.config build/build-kernel.sh TREE out-z20` builds the z15 kernel tree with dynamic debug and `CONFIG_DEBUG_GPIO`;
`build/build-z20.sh Z19_BOOT_IMG out-z20 images-z20` repacks z19 with it, adding `dyndbg="file drivers/gpio/gpiolib.c +p; file drivers/gpio/gpiolib-of.c +p; file drivers/pinctrl/core.c +p; file drivers/pinctrl/devicetree.c +p; file drivers/pinctrl/qcom/pinctrl-spmi-gpio.c +p"` to the
command line (dtb, ramdisk and the rest are z19's, byte for byte). `tests/host/gpio-diag-boot.sh` fails on z19 and passes on z20. Built only; the keeper tries it once (walleye-try-z20).

## UFS is off, and a gate keeps it off (walleye-ufs-off-gate)

Decision 2026-11-19-walleye-ufs-off-by-a-gate; research walleye-ufs-on-by-accident. UFS was on in every image from z11 and was probed against the author's word. Now `&ufshc` and `&ufsphy` are `status = "disabled"` in `dt/msm8998-google-walleye.dts`;
the z11, z12 and z13 experiments take `dt/msm8998-google-walleye-ufs.dts` (the same file with the two nodes okay, built by `build-kernel.sh` as `msm8998-google-walleye-ufs.dtb`) and nothing else does.
`tests/host/ufs-off-image.sh IMAGE|DTB` is the gate: every dtb appended to the image's kernel, alone and merged with the phone's own dtbo entry 3, must hold no UFS host or phy node that is on (okay, or with no status), and the command line must name no ufs
(the bootloader's own `androidboot.*` words aside). `build/ufs-gate.sh IMAGE...` runs it for the build scripts: every `build-*.sh` that writes a boot image stops there when UFS is on (z11, z12, z13 are the experiments that keep it on and carry no gate: their images are not to boot).
`tests/host/ufs-off-image-selftest.sh` proves the gate on made dtbs and images (CI job `ufs-off-gate`); `tests/host/ufs-off-inventory.sh` lists which images under `~/Projects/walleye-build/images*` have UFS on.
`build/build-z21.sh Z20_BOOT_IMG OUT_DIR` makes z21 = z20 with the stub container and UFS disabled in the dtb, nothing else changed (the gate fails z20 and passes z21).

## The series files: every bisect, declared (build/series/)

One command builds and counts a bisect: `lpoco-bringup image bisect build/series/NAME.toml --out DIR` (lab-tools 0.4.0, here 0.24). A series names
a control image and, per `[[image]]`, the one part each swaps (kernel, dtb, ramdisk, command line; `lz4`, `footer`); paths are relative to the
series file and point into `~/Projects/walleye-build` (`../../../../walleye-build`, the same from a worktree beside the clone). The images
of each series are byte-equal to the recorded ones (sha256 in the SHA256SUMS of the folder each series' "Run:" line names; stock-to-ours: `bisect/`, pk456: `images-pk456/`):

| series | control | images |
|---|---|---|
| stock-to-ours.toml | the stock boot.img | s0, s0n, s1, s2, s3 |
| pk456.toml | pk3 | pk4, pk5, pk6 |
| dtb-bisect.toml | pk4 | d1..d6 |
| taimen-walleye.toml | pk5 | w1..w5, wr, wa |
| property-bisect.toml | wa | wp1..wp5, wpa |
| fdt-bisect.toml | wpa | x1..x7, xa |

The parts the pk series names are `lpoco-bringup image unpack` of pk3, ml6a and ml5b into `walleye-build/parts-series/NAME/`, `ml6a/Image.gz`
being ml6a's `kernel` less the dtb that ends it. The dtbs of the last four series are the recorded ones: `build-dtb-bisect.sh`,
`build-taimen-walleye.sh`, `build-property-bisect.sh` and `build-fdt-bisect.sh` still make them (node edits, which `image bisect` does not do).

Three things to know when you run them:

- **The UFS gate does not run inside `image bisect`.** The old scripts stopped on `ufs-gate.sh`; a series writes every image it declares. Run
  `build/ufs-gate.sh DIR/boot-*.img` on the output and do not boot an image it refuses (s0..s3, pk5, pk6, w*, wp* and most x* are stock-dtb or taimen-dtb
  controls and UFS-on by nature: they carry `NOT-TO-BOOT-*.txt` beside the recorded ones). Note the gate deletes a failing image.
- **The series chain.** The controls of taimen-walleye, property-bisect and fdt-bisect are images of earlier series (pk5, wa, wpa): run pk456, then
  taimen-walleye, property-bisect, fdt-bisect in that order, into the `images-*` folders their controls name.
- **The parts of pk456 are made by hand:** `lpoco-bringup image unpack IMG --out parts-series/NAME` for pk3, ml6a, ml5b, then
  `head -c $((KERNEL_SIZE - DTB_SIZE)) ml6a/kernel > ml6a/Image.gz` (the dtb is `appended.dtb`, the tail of `kernel`). Not scripted; the s3o image
  (our dtb with overlay stubs) the old script could add is not declared.

## The display bisect images d0, d1, d2

z21's boot.img with the display kernel (`d0`), then the display dtb in stages: `d1` mdss and the display controller only, `d2` plus the DSI host and its PHY, no panel node (`d3` is `images-display/boot-display.img`, tried: no shell).
`build/build-display-bisect-dtbs.sh images-display/boot-display.img images-display-bisect` cuts Image.gz, `d1.dtb` and `d2.dtb` from that image (counting what differs),
`lpoco-bringup image bisect build/series/display-bisect.toml --out images-display-bisect` packs the three, `build/check-display-bisect.sh IMAGES Z21 PARTS BOOT_DISPLAY_IMG` counts each against z21 (and the parts against the display image)'s (ramdisk, header, size, footer, UFS-off gate) and writes `SHA256SUMS`.
Built only; the try is walleye-try-display-bisect.

## The display late probe (item walleye-display-late-probe-image)

d1 (the display nodes on, the driver built in) loses the shell and no log survives, so the display path is built as modules and loaded by hand from the shell, with the kernel log already going to the host.
`build/build-display-late.sh ~/.local/share/agent-worktrees/linux/wt-k-fg OUT` builds the kernel (`build/walleye-display-late.config`: DRM, DRM_MSM and the panel driver `=m`, NETCONSOLE and NETCONSOLE_DYNAMIC `=y`, DYNAMIC_DEBUG) and the display dtb, stages `msm.ko`, the panel driver and every module they need under `OUT/modroot` with depmod run, and writes `OUT/display-modules.list` (insmod lines, dependency order).
The kernel tree is the fuel-gauge worktree, not the older display one: the shared dts now names the label `pmi8998_fg`, which only the fuel-gauge dtsi patch defines (the driver stays off, the node unbound).
`build/build-display-late-image.sh Z21_BOOT_IMG OUT IMAGES` packs `build/initramfs-mainline/init-late` (z14's init; its self-reboot is held while `/tmp/hold` exists) with the modules, their depmod output and the list into z21's header, command line and footer, and counts the image (37 checks, the UFS-off gate among them). Set `TMPDIR` on a disk with room: /tmp is a 16 GB tmpfs.
`tests/host/display-late-probe.sh OUT_DIR` is the try: shell up, `/tmp/hold`, netconsole by configfs to 172.16.43.2:6666 and a TCP stream of /dev/kmsg on 6667 (each proved by a marker), then every list line loaded with its step written to the host first; its fail line names the step and the last line of each channel. `tests/host/display-late-probe-selftest.sh` proves it against a fake phone.

## The display dtb bisect images f0, f1, f2, f3 (item walleye-display-dtb-bisect-images)

The display death is in the dtb, not the kernel (d0 boots with the DRM drivers built in; decision 2026-11-19-walleye-display-dtb-bisect). The four images carry the late-probe image's kernel (display drivers as modules, netconsole), its ramdisk (init-late and the module list) and its header, and z21's dtb with one group of the display dtb's additions each:
`f0` nothing (the control: z21's dtb byte for byte), `f1` only the regulators (`vreg_l14a_1p8`, `vreg_l22a_3p0` under the rpm `regulators-0`, and its `vdd_l18_l22-supply` line to bob), `f2` only the three pinctrl states (`mdss_dsi_active_state`, `mdss_dsi_suspend_state`, `mdss_te_state`), `f3` only mdss, the display controller, the DSI host and its PHY set okay (no panel node, no regulators, no pinctrl states).
`build/build-display-dtb-bisect-dtbs.sh Z21_BOOT_IMG LATE_BOOT_IMG images-display-dtb-bisect` cuts Image.gz from the late-probe image, edits z21's dtb with fdtput into `f0..f3.dtb` and writes `f1.diff..f3.diff` (the decompiled dtb of z21 against each: only that group). `lpoco-bringup image bisect build/series/display-dtb-bisect.toml --out images-display-dtb-bisect` swaps each dtb into the late-probe image, `build/check-display-dtb-bisect.sh IMAGES Z21 LATE PARTS` counts each (kernel, dtb, ramdisk, header, size, footer, UFS-off gate) and writes `SHA256SUMS-display-dtb-bisect`. Set `TMPDIR` on a disk with room.
Notes: z21's display controller node has no status (enabled by default), so f3 changes mdss, the DSI host and its PHY; f3 does not carry the display dts's DSI `vdda-supply`, PHY `vdds-supply` or the DSI output endpoint (the group is the four status lines, as the item says); f1's regulators have no consumer, so the kernel may switch them off late.
Built only; the try is walleye-try-display-dtb-bisect. The new nodes carry no phandle and no `__symbols__` entry: nothing refers to them.

## L6, the stripe drawer (item walleye-display-stripes-image)

L6 is L5 (the display late probe with the panel binding, research walleye-display-late-probe-l5-tried-dsi-connected-1080x1920) plus one program in the ramdisk, `/bin/walleye-stripes`. Kernel, dtb, command line, header words and size are L5's.
`tools/walleye-stripes` (Rust, command line declared with argenv) writes four vertical stripes, red, green, blue, white, across the framebuffer `/dev/fb0` (the fbdev emulation of card0, `CONFIG_DRM_FBDEV_EMULATION=y`; it exists once `msm` has bound), pans the display, holds them `--seconds` seconds, clears to black; a log line goes to standard output and to `/dev/kmsg` at the draw and at the release, and the connector's `status`, `enabled` and `modes` (`/sys/class/drm/card0-DSI-1`) are read and logged before the draw and after the release. `cargo test` in the crate counts the frame (stripe colours at the quarter edges, channel offsets, 16 bits, padding, the kernel's struct sizes).
`build/build-stripes-bin.sh OUT` builds it static for aarch64 (std from rust-src, `aarch64-linux-gnu-gcc`, paths remapped; the same source gives the same bytes). `build/build-display-stripes-image.sh L5_BOOT_IMG LATE_DIR STRIPES_BIN OUT` packs the ramdisk the way `build-display-late-image.sh` does, proves that without the drawer it is L5's ramdisk byte for byte, swaps the ramdisk with the drawer into L5 (`lpoco-bringup image swap`), counts the image (the file list of L5's ramdisk against L6's gains only `bin/walleye-stripes`; footer; UFS-off gate) and writes `boot-l6.img`, `l6.diff`, `SHA256SUMS-display-stripes`. Set `TMPDIR` on a disk with room.
The drawer is started by `tests/host/display-late-probe.sh` after the modules are loaded and the settle (STRIPES_SECONDS, 20; 0 leaves it out; an image without the drawer, L1 to L5, is left alone); its lines are kept in `OUT_DIR/stripes.log` and a nonzero exit fails the run. Built only; the try is walleye-try-display-stripes-unattended.

## Image B2: the author's look at the screen (item walleye-display-look-b2)

`build/build-display-look-image.sh B1_BOOT_IMG LATE_DIR STRIPES_BIN OUT` makes B2 = B1 plus `/bin/walleye-stripes` (built by `build-stripes-bin.sh`; this drawer puts `/dev/tty0` in KD_GRAPHICS while the stripes are held and in KD_TEXT after, also after a failed draw, and logs `/proc/consoles` and each vtcon's name and bind). Init is B1's `init-display`, which does not start the drawer: `procedures/display-at-boot-on-the-pixel-2.rs` does, after the first look. Checks, each stopping the build: B1's recorded sha256, the ramdisk without the drawer equals B1's byte for byte, kernel, header words and size are B1's, the file list gains only `bin/walleye-stripes`, AVB, the UFS-off gate and tests, one claimant of the panel reset gpio. Writes `boot-b2.img`, `b2.diff`, `SHA256SUMS-display-look`. Set `TMPDIR` on a disk with room.

## The delay bisect images: B-d5 ... B-d50 (item walleye-display-delay-bisect-build)

`build/build-display-delay-images.sh B2_BOOT_IMG LATE_DIR OUT [DELAY...]` (default delays 5 15 25 35 50) makes one image per delay from B2: only `/init` (`build/initramfs-mainline/init-display-delay` = `init-display` plus added lines) and the new file `/display-delay` (the number of seconds of uptime) differ; the drawer, kernel, dtb, command line, header words, size, modules and list are B2's. The init waits until `/proc/uptime` has reached the number, says `walleye-display: delay N s reached at uptime T s` in the kernel log, then loads the modules as `init-display` does: the load time does not depend on the shell (the shell is reachable from the host only at about 18 s of wall time). Checks, each stopping the build: B2's recorded sha256, the ramdisk made with `init-display` and no delay file is B2's byte for byte, the init is `init-display` plus added lines only (`tests/host/display-delay-init.sh`, which advances a fake uptime and proves the wait, 0, no file and text), every image made twice comes out identical, kernel, header words and size are B2's, the file list gains only `display-delay`, AVB, the UFS-off gate and tests, one claimant of the panel reset gpio. Writes `boot-b-dN.img`, `b-dN.diff`, `SHA256SUMS-display-delay` into OUT (`~/Projects/walleye-build/images-display-delay`; each image's sha256 is in `device.toml` `[images.b-dN]`). Set `TMPDIR` on a disk with room.

`procedures/display-delay-bisect-on-the-pixel-2.rs` (lpoco-attended, no person step) boots the five in a row from fastboot (nothing flashed), reads each kernel log and counts the three signs (SMMU context faults, `rcg didn't update its configuration`, DSI -110), sends the phone back to Android between images and after any stop, and keeps the table (delay, reached, faults, warnings, -110, modules ok) in `~/lab-material/walleye/display-delay-DAY/table.txt` with the flip point. The delays are `[procedures.display-delay-bisect-on-the-pixel-2] delays`.
