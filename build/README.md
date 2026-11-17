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
