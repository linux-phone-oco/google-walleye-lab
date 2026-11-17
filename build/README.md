# Building the first walleye image

Everything here builds the first thing the phone is tried with: mainline v7.3 + our own `dt/msm8998-google-walleye.dts`, a
small initramfs with a USB gadget, a v0 boot image. Nothing here touches the phone (no fastboot, no adb); the keeper boots
what is built, once, with `devlab fastboot boot now`.

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

Walleye and surya share one host, and surya's pmOS gadget answers on 172.16.42.1, so devlab must tell the phones apart; the manifest
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
