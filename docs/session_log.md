<!-- Generated from data/ by tad. Do not edit by hand. -->

# Session Log

<a id="2026-11-17-device-base"></a>
### The device base, from device-template

Entry date
: 2026-11-17

Done
: google-walleye-lab gets its own open tad base from linux-phone-oco/device-template (#4, #5): tables.toml, idealkb.toml (0.9.1), devicebase.toml (parts = 1), generic, records/, queries/, Makefile, bin/install-tad.sh and install-ikb.sh, the tad CI; AGENTS.md and oco.toml now point recording here. device.toml is unchanged (devlab's). Rows of this lane move here from docs next, under the keeper lease (docs decision 2026-11-17-commons-and-device-bases).

Considered
: Recording this in docs as before; the base is the place from now on, so its first row is its own making.

---


<a id="2026-11-17-the-first-walleye-images-built"></a>
### The first walleye images built

Entry date
: 2026-11-17

Done
: Built v7.3 + own msm8998-google-walleye.dts (clang, defconfig + fragment), a busybox gadget initramfs with a QEMU x86_64 self-test, and two v0 boot images (taimen and Lineage offset sets) with a counting check; all through oco-agent jobs, nothing touched the phone. Lab PR 3. Images and SHA256SUMS in ~/Projects/walleye-build/images; reproduce with build/build-all.sh.

Considered
: qemu-system-aarch64 for the initramfs proof (not installed; installing is a change to the lab machine); porthole's cmdline pieces qcom_wdt.arm_on_probe and watchdog.open_timeout (they need porthole's patches, mainline ignores them).

Rejected
: a new kernel clone (reused surya's shared clone as a worktree, as surya does); python for the boot image check (shell + mkbootimg/unpack_bootimg/fdtget)

About
: walleye-kernel-dts

---


<a id="2026-11-17-walleye-builder-stock-ids"></a>
### Stock boot.img read; variants e and f built

Entry date
: 2026-11-17

Done
: Downloaded LineageOS walleye boot.img and dtbo.img (the author's word, decision walleye-control-image-download), verified sha256, unpacked on the host, read header and dtb ids (research walleye-stock-ids-recorded). Built e and f; check-bootimg counts 20 ok for each of a-f (lab PR 6).

Considered
: Building f with the stock board-id/pmic-id as asked: none exist in the stock dtb; chose qcom,board-id <8 0> from its qcom,mtp compatible, no pmic-id, labelled a guess.

Rejected
: qcom,board-id from the dtbo table id 0x1bc: Google's overlay id, not a Qualcomm platform id. Changing the os word: a was the only silent OKAY.

About
: walleye-loaderror-stock-ids

---


<a id="2026-11-17-1900-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: Control, e and f tried on slot b by fastboot boot: all Load Error, flags unchanged. Phone in fastboot, slot b current and good, a untouched reserve, stock Android unchanged, nothing written.

Considered
: next: the author's word on a dtbo stub (a write) or on slot a flags; nothing more to try from the host side without it

---


<a id="2026-11-17-walleye-how-the-phone-stands-2"></a>
### How walleye stands

Entry date
: 2026-11-17

Done
: Control boot.img refused with Load Error on slot a too (slot state is not the cause); phone back on slot b via set-active, both slots bootable (retry 1 each), fastboot reachable, nothing written; lease released

Considered
: next: walleye-boot-refused-why - remaining causes need a dtbo/vbmeta write (walleye-loaderror-dtbo-stub waits on the author's word); never repeat the slot-a control

---


<a id="2026-11-17-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: First state restored and control accepted; phone in fastboot, unlocked, slot a active and successful, b untouched (retry 1); nothing flashed

Considered
: next: walleye-own-images-first-state (our image e, then b, by fastboot boot from this state); lease released

---


<a id="2026-11-17-walleye-images-g-h-i"></a>
### Images g, h, i: the gadget at 172.16.43.1

Entry date
: 2026-11-17

Done
: build-initramfs.sh takes NET (default 42: files used as they are, so a-f stay byte for byte; 43 rewrites 172.16.42 to 172.16.43 and ECM MAC byte 2a to 2b into WORK_DIR/net43). Built g=e, h=b, i=f; check-bootimg counts the address in /init and the lease; selftest PASS at NET 42 and 43. sha256 g cc3cc1af..., h 3090bbcd..., i 5fdcd4ca...; images/SHA256SUMS and DIFFERENCES.txt updated.

Considered
: Editing init in place to 43; an address file inside the ramdisk

Rejected
: Both: they change the a-f ramdisk bytes while a keeper is trying e and b

About
: walleye-initramfs-usb-gadget

---


<a id="2026-11-17-builder-vendor-gadget2-image"></a>
### Builder: vendor-gadget2 image

Entry date
: 2026-11-17

Done
: Found the panic cause in the stock kernel's config, built an xz ramdisk + init for the vendor kernel, vendor-gadget2.img (control with only the ramdisk swapped, AVB footer), 10 checks ok, landed google-walleye-lab PR 12, queued the try.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2030-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: s0n (no footer) tried from the healthy state: left fastboot, off USB like s0; the phone is OFF USB and needs a hand power hold (Power 10-30 s), then boots Android on a (a active+successful); nothing flashed

Considered
: next: after the hand reset, the parts bisect (s2 kernel, s3 dtb) one at a time; the footer is ruled out

---


<a id="2026-11-17-walleye-vendor-gadget3"></a>
### vendor-gadget3 built

Entry date
: 2026-11-17

Done
: vendor-gadget3.img: vendor-gadget2 with only /init changed (early kmsg marker, 240 s self-reboot timer cancelled by /tmp/hold, reboot -f then sysrq b). Checked the stock config: MAGIC_SYSRQ=y, enable mask 0x1, PSTORE_RAM y. 14 checks ok incl. ramdisk equal to gadget2's apart from /init. Landed google-walleye-lab 0f14a92. Try queued.

Considered
: timer in the telnet shell only (dies with no shell); a watchdog (none known in the vendor config)

Rejected
: putting the timer after the gadget setup: a failed gadget would then never return

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2035-walleye-gadget3-try"></a>
### vendor-gadget3 tried twice from the first state: kernel_panic, back on a, gadget not shown

Entry date
: 2026-11-17

Done
: devlab fastboot boot now vendor-gadget3.img (sha 608afa37...) from the first state (a current+successful, b unbootable, retry 1), twice. Try 1: left fastboot after 12 s, Android on a after 107 s. Try 2 (gadget-up watcher and lsusb log running): left after 12 s, Android on a after 76 s. ro.boot.bootreason=kernel_panic after both; no 240 s timer reboot. walleye-gadget-up not passed: its watcher output was lost when my cleanup killed the shell, so no pass is claimed; the lsusb log shows 18d1:d001 fastboot-class devices at 20:33:41-20:34:41 (strings not read: not known yet whether ours). After: a active, Android booted (sys.boot_completed 1). Nothing flashed.

Considered
: next: a third try with the watcher kept alive (GADGET_WAIT=120, own job, output to a file) and the d001 device serial/product read at once; panic cause unseen - a deeper question if a clean watcher also shows nothing.

---


<a id="2026-11-17-rows-moved-from-docs"></a>
### Walleye's rows moved here from docs

Entry date
: 2026-11-17

Done
: Under the lease orchestration-move: 41 backlog items (device google-walleye or project google-walleye-lab, lab-tools' own items stay in docs), 20 research, 7 decisions, 10 log entries and their research/ material, replayed from docs through this base's records; 21 parts seeded from docs' support_status of google-walleye (all unknown). Kept as they were, except: an about naming the device or a docs subject (android-reference) is left empty (the base is the device's; across bases: docs about-across-bases); six decision ids gain their date (2026-11-17-...) as this base's ids require. Added queries/lane-next.sql (--needs phone|research|build) for the research and build launchers. From now on walleye records here, through ikb; docs keeps what helps every device.

Considered
: Moving lab-tools' items with device google-walleye too: they are the conveyor's, helping every device.

---


<a id="2026-11-17-2055-walleye-gadget3-third-try"></a>
### vendor-gadget3 third try: panic before the gadget

Entry date
: 2026-11-17

Done
: Took the lease, watcher first (own job, 130 s), fastboot boot of vendor-gadget3 from the first state; read the result; phone back on Android a, fastboot reachable, slot a good.

Considered
: next: a builder makes a kmsg-only init image (separates ramdisk unpack from gadget bring-up)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-vendor-gadget4"></a>
### vendor-gadget4 built

Entry date
: 2026-11-17

Done
: vendor-gadget4.img: the control with only the ramdisk swapped, an idle init (kmsg marker, proc, alive markers, reboot -f at 120 s, sysrq b fallback; no gadget). 14 checks ok, AVB verifies, size 33554432. sha256 vendor-gadget4.img db23f804a975d4077568cba458814a8af834846a59b2f8d90662ec21e6abbf1b; initramfs-vendor4.cpio.xz 0900a9db50731a4b0fb9e0a9428e3f91e4d8b993a69178d4605b480f9d09bcd4 (in ~/Projects/walleye-build/images, SHA256SUMS). Expected: a panic at ~75 s means not our gadget; a return at ~120 s means the gadget bring-up panics.

Considered
: alive markers every 30 s (pstore/console trail if it panics)

Rejected
: mounting configfs/pstore in the idle init: it would blur the split

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2105-walleye-how-the-phone-stands-3"></a>
### gadget4 try: the panic is the gadget bring-up

Entry date
: 2026-11-17

Done
: Took the lease; from the first state fastboot boot of vendor-gadget4.img: left fastboot 15 s, back on Android a at 156 s, bootreason reboot, boot_completed 1. Left in fastboot, slot a active and successful, b unbootable retry 1 (unchanged); nothing flashed.

Considered
: next: split the gadget bring-up of gadget3 into steps (configfs, functions, UDC bind), a try for each

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-vendor-gadget5"></a>
### vendor-gadget5a/b/c built

Entry date
: 2026-11-17

Done
: Three images for the gadget bring-up bisect, each the control with only the ramdisk swapped, gadget4's init (kmsg marker, 30 s alive markers, reboot -f at 120 s, sysrq b) plus one step: 5a configfs + gadget dir (ids, strings); 5b + ncm function and config symlink; 5c + UDC bind, usb0 172.16.43.1, telnetd. build/build-vendor-gadget5.sh: 14 checks each, all ok, AVB verifies, size 33554432, ramdisk differs from gadget4's only in /init. Files in ~/Projects/walleye-build/images, SHA256SUMS. sha256: vendor-gadget5a.img a89fc3d027fe50c7990d6ba6de13bd7976000d63a5d6dd7a6be92db60fd92a87; vendor-gadget5b.img 2e19c15c8ce00f0f24b2477f5a4617948b4e27a6e079832c49772548bffdc26e; vendor-gadget5c.img d09280cce8860a45c8c76701f319686b40948fa2d264341145255d6f31746662; initramfs-vendor5a.cpio.xz eeb644653738c001611363c6b7c8f542a96dbe4537b18cccbdea0880a101a81c; 5b 163c6ef6940e25b13e8dc89bee745459ec59588de6560866b0628f40b9734ad8; 5c 5a9f77fb0ff759b8e4e2218c9fc0d46506de5f15ad1e4d6b8d1f8fc003217073. Outcome table (tries from the first state, each in order a, b, c, stop at the first panic): panic = bootreason kernel_panic, back on Android a in ~75 s; no panic = phone returns by itself at ~120 s with bootreason reboot. First panic at 5a: configfs mount or gadget dir creation; at 5b: the ncm function or config; at 5c: the UDC bind (the dwc3 controller) or the network/telnetd part. 5c without a panic: the gadget is up and gadget-up.sh (read-only ping/telnet to 172.16.43.1) can pass during the 120 s. No hands expected in any outcome: the init returns by itself, a panic falls back by itself.

Considered
: one script for all three (a loop), one init file per step so each image's source is readable alone

Rejected
: a single image with a step chosen at run time (no way to pass it: the kernel command line is the control's and must stay)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2125-walleye-gadget-bisect"></a>
### Gadget bisect: 5b panics

Entry date
: 2026-11-17

Done
: Took the lease; tried vendor-gadget5a (no panic, back at 156 s) and 5b (kernel_panic, back at 75 s) from the first state on the active slot a as the earlier gadget tries did; stopped at the first panic. Phone left in fastboot, a active+successful, b unbootable retry 1 (unchanged), nothing flashed.

Considered
: next: builder splits 5b: mkdir ncm function only, then the config symlink; or acm/rndis instead of ncm

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-vendor-gadget5b-split"></a>
### vendor-gadget5b1..b4 built

Entry date
: 2026-11-17

Done
: Four images, each the control with only the ramdisk swapped, gadget5a's init plus one ncm step: b1 mkdir functions/ncm.usb0; b2 + dev_addr, host_addr; b3 + configs/c.1 and its string; b4 + symlink (= 5b). build/build-vendor-gadget5b.sh, 15 checks each ok (header = control's, kernel = control's, xz ramdisk, AVB verifies, 33554432 bytes, ramdisk differs from gadget4's only in /init, exact step parts). sha256: 5b1 a50624ada0e1c1a2 5b2 2d1bacd3056d28ba 5b3 95f5a5ec86925374 5b4 b923eeb243d3cb50 (full in images/SHA256SUMS). Closed walleye-kmsg-only-init: gadget4 was already built and tried.

Considered
: three images as the backlog said

Rejected
: Three images b1..b3 with config dir and symlink together: a panic in b3 would not say which of the two.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-vendor-gadget-n"></a>
### vendor-gadget-n1, n2 built

Entry date
: 2026-11-17

Done
: Two images, each the stock control with only the ramdisk swapped (xz), vendor-gadget5a's init plus the ncm step without any write to the function's settings (walleye-ncm-panic-why): n1 = 5b4 minus the dev_addr/host_addr writes (function, config with its string, symlink; no bind); n2 = n1 + UDC bind, ip link set usb0 address, telnetd on 172.16.43.1. build/build-vendor-gadget-n.sh, 17 checks each ok (header and kernel = control's, AVB verifies, 33554432 bytes, ramdisk differs from gadget4's only in /init, exact step parts, grep for dev_addr|host_addr|qmult|ifname finds none). All earlier images (a-i, gadget1-5c, 5b1-b4, s0-s3) byte for byte as before (sha256sum -c: 0 differ); rebuilt twice with the same hashes. Files in ~/Projects/walleye-build/images, in SHA256SUMS there. sha256: vendor-gadget-n1.img b2e96b420e566d35acc415e964690feed9ecf7d67d0fbcebc8dbe467c67793b2, vendor-gadget-n2.img b007d1e3b54c9da90ed8dc5cb44ee098c9c06776a3ffc39ea64d98a0e4282a31; ramdisks initramfs-vendor-n1.cpio.xz f64aa41a5bfcd792ebb6cf06d4b52257ede6c7e2b77c843d027102bbacdc128d, initramfs-vendor-n2.cpio.xz d10d20db747081618fc4d1e01c75524ed82405387a1620a1c48fdf2c170ccfa4.

Considered
: n2's bind is announced inside its if (not at a line start), so the check counts 4 announcements, not 5. The usb0 address is set by ip link (busybox has CONFIG_IP and FEATURE_IP_LINK); ifconfig hw ether is the fallback if ip is refused.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2230-walleye-ncm-split"></a>
### ncm split: b2 panics

Entry date
: 2026-11-17

Done
: Took the lease; tried vendor-gadget5b1 (returns, reboot) and 5b2 (kernel_panic at 75 s) by fastboot boot from the first state; phone left in fastboot, a active, nothing flashed

Considered
: b3, b4 not tried (stop at the first panic); next: the noattr try with vendor-gadget-n1, n2

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2240-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: Fastboot, unlocked, slot a active and successful, b unbootable retry 1; nothing flashed; ncm split done (b2 panics), lease released

Considered
: next: walleye-try-ncm-noattr (n1 then n2, from the first state: Android on a, then adb reboot bootloader)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2155-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: n1 returned; n2 enumerated with our strings and no panic; host cdc_ncm bind failed (MAC string not served). Fastboot, unlocked, slot a active+successful, b unbootable retry 1; nothing flashed; lease to be released

Considered
: next: walleye-gadget-host-bind (ecm / rndis images, builder), then ffs-helper if those fail

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-vendor-gadget-m1"></a>
### vendor-gadget-m1 built: the mass-storage log window

Entry date
: 2026-11-17

Done
: Built vendor-gadget-m1.img (the stock control's boot.img, only the ramdisk swapped; init-m1): a mass_storage gadget (functions/mass_storage.0, lun.0 ro=1 removable=1 cdrom=0 nofua=0, file = a 2 MiB file in the ramdisk), no ncm and no network, every lun attribute written only if it exists and each configfs step announced in kmsg first; every 5 s the first 64 KiB of the file is rewritten in place (beat, uname, cmdline, mounts, udc, dmesg tail; NUL padded); reboot -f at 180 s then sysrq b. build/build-vendor-gadget-m.sh: 22 checks ok. tests/host/massstorage-read.sh finds the LUN by our USB strings (sysfs), reads 64 KiB with O_DIRECT, prints the text, one pass/fail/skip line. sha256 vendor-gadget-m1.img a3cf08d388ab72c5e1532e7ae60c87c48e54524fda6ca2d9be7bf4aeb81f360a, initramfs-vendor-m1.cpio.xz dd59bce15a988dac7488b079deb6558fba2da24b620aa543ea4801f5ecfbc29e, in ~/Projects/walleye-build/images and its SHA256SUMS. Not measured yet: the host reader against a real LUN (the phone was not touched), whether this kernel exposes every lun attribute.

Considered
: Writing the window by dd in place with NUL padding (no stale tail) against rewriting the file whole (the LUN would be reopened).

Rejected
: An inquiry_string attribute to name the LUN: not in the brief's attribute list, the host finds the LUN by the usb strings instead.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-builder-vendor-gadget-n3-image"></a>
### Builder: vendor-gadget-n3 image (FunctionFS ACM console)

Entry date
: 2026-11-17

Done
: Built the static aarch64 helper build/ffs-acm/ffs-acm.c (descriptors V2 FS/HS/SS then strings to ep0, only then the UDC bind; one class 2/2/1 interface with 3 endpoints; pty shell bridged to ep2/ep3 by two forked pumps; setup requests for line coding answered, others stalled; no function attribute written), init build/initramfs-vendor5/init-n3 (functions/ffs.acm, devpts, functionfs mount, helper in the background, self-reboot at 180 s) and build/build-vendor-gadget-n3.sh (21 checks, all ok). Images in ~/Projects/walleye-build/images, added to SHA256SUMS: ffs-acm fea51dd8fc5057ced8ac358607210676612c68209edb4a86a61c64086fef1f04; initramfs-vendor-n3.cpio.xz 0be5a7f05f878a8be608a333d410a5619fc10940bd85cbe1635444a196337a9e; vendor-gadget-n3.img 6672eb844fb5435ebb336e33a0b59db069bc2e62a6e01f98d760f5646b1d214c. tests/host/ffs-acm-up.sh fails without the phone, as it must. Not run on the phone.

Considered
: CORRECTION (walleye-coordinator, 2026-11-17): the builder wrote here that the lab has no aarch64 Rust target - that is wrong. The lab has one: lpoco-health is built static for aarch64-unknown-linux-musl with the toolchain in ~/.local/share/devlab-cross (rustup + aarch64-linux-gnu-gcc as the linker; see lab-tools health/build.sh). The C helper of this image is therefore a PROBE only; before the helper is kept or reused it is rewritten in Rust (backlog walleye-ffs-helper-rust). The helper takes no arguments and reads no environment, every path is a constant, so there is nothing for argenv to declare. Threads: rejected for two fork pumps.

Rejected
: Mass-storage window in n3: n3 changes one thing (the channel) from n2. Not tested under qemu: FunctionFS needs a UDC.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2155-walleye-m1-try"></a>
### m1 try: gadget enumerates as mass storage, the host cannot read it

Entry date
: 2026-11-17

Done
: Took the lease; fastboot booted vendor-gadget-m1 (sha a3cf08d3...) from the first state: left fastboot at 12 s, enumerated 1d6b:0104 with our strings at 18 s, no panic, class 08 interface up, phone returned by itself to Android on a. Read of the log window failed: no usb_storage module on the host for kernel 7.3.3 (modules dir has 7.3.4 and 6.18.60-lts). Phone put back in fastboot (adb reboot bootloader): a active+successful, b unbootable retry 1, unchanged; nothing flashed.

Considered
: next: the host loads usb_storage (reboot into 7.3.4 - a host change, not mine), then repeat the m1 try and read the window; or the FunctionFS ACM helper, which needs no host storage driver

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2200-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: Phone in fastboot, unlocked; slot a active and successful, b unbootable retry 1 (unchanged); stock Android on a; nothing flashed. m1 enumerates as our mass-storage gadget with no panic; its log window is unread because the lab host has no usb_storage module for kernel 7.3.3.

Considered
: next: walleye-try-massstorage again once the host has usb_storage (reboot into 7.3.4), or the FunctionFS ACM helper

---


<a id="2026-11-17-2210-walleye-how-the-phone-stands-4"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: n3 tried once: enumerates 1d6b:0104 with ACM class interface after 18 s, no panic; host lacks cdc_acm for running kernel 7.3.3 so ffs-acm-up fails; phone back on Android a by self-reboot, now in fastboot, slot a good, nothing flashed

Considered
: next: reboot the lab host into 7.3.4 (host owner), then retry n3 and the ACM test

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-builder-n3r"></a>
### Builder: vendor-gadget-n3r (Rust FunctionFS ACM helper)

Entry date
: 2026-11-17

Done
: Rewrote the helper in Rust (build/ffs-acm-rs, libc crate, panic=abort, static musl, linked by aarch64-linux-gnu-gcc as lab-tools health/build.sh does): same steps as the C probe, descriptors and strings checked byte for byte against the probe's by a unit test (tests/c-probe-blobs.bin). build-vendor-gadget-n3r.sh makes vendor-gadget-n3r.img: n3 with only /bin/ffs-acm swapped (init byte for byte, checked against initramfs-vendor-n3); 21 checks ok; second build gave identical hashes. Hashes in images/SHA256SUMS. No arguments or environment read, so no argenv. Phone not touched.

Considered
: nix crate instead of libc: more code for the same fork/ioctl calls

Rejected
: Replacing n3's C helper: it stays as the probe; n3r is a separate image so the try can compare

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2230-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: m1 tried once from the first state: runs, enumerates, usb-storage binds /dev/sda; host read refused (no disk group). Phone back in fastboot, unlocked, slot a active+successful, b unbootable retry 1; nothing flashed

Considered
: next: host read access then retry m1; rejected: changing host groups or using sudo myself (system setting, the author's)

---


<a id="2026-11-17-2245-walleye-overlay-stubs-build"></a>
### Overlay stubs build: s3o image

Entry date
: 2026-11-17

Done
: Generated the 130-stub compat dtsi from the dtbo entry 3 fixups, built our dtb with -@, host overlay-merge test passes, built image s3o (control + our new dtb) with checksums, queued the phone try.

Considered
: Stubs at the stock paths (would need a /soc node beside our soc@0); flat stubs under one parent node chosen: fdtoverlay and libufdt need only the symbol and a node.

Rejected
: Replacing s3 with the new dtb: s3's result stays reproducible, so the new image is s3o beside it.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2310-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: s4 tried once: overlay symbols fix the s3 hang (left fastboot, Android a at 36 s, reboot). Phone in fastboot, unlocked, slot a active+successful, b unbootable retry 1; nothing flashed; lease released

Considered
: next: our kernel with the symbols dtb (vendor-gadget with s3o dtb); read why the stock kernel resets with our dtb

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-builder-mainline-light"></a>
### Builder: ml1 and ml2 images (first light of our mainline kernel)

Entry date
: 2026-11-17

Done
: Built v7.3 + walleye.config (+QCOM_WDT, MASS_STORAGE, RD_XZ) into out-ml; added the watchdog node to the dts, build/initramfs-mainline/init-ml, build/build-mainline-light.sh (32 checks, rebuild gives the same hashes) and tests/host/mainline-light-up.sh. Images in ~/Projects/walleye-build/images-ml: boot-ml1.img 41d8efc0c6e5c3ed3086e66f605c39ddc4a0bcb8a0e266a2cbbd010738cc636b, boot-ml2.img 63b2977f1f21c5de40b2c8efcbb0f033860ad5fae2c3ee9eca052f77f5979add, initramfs-mainline-light.cpio.xz 9feb844211c3e5b636b2481eb920d36cdf698eaf4441090574edd943a69b7e01 (SHA256SUMS beside them). Not run: the init under QEMU (syntax checked only), dtc W=1 on the new node. An early hang (before the watchdog opens) still needs the author's power hold.

Considered
: ml2 pack: raw Image in an lz4 frame (-9 -BI -B7) as build-bisect.sh does

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-keeper-ml-light"></a>
### ml1 and ml2 first-light tries

Entry date
: 2026-11-17

Done
: ml1 then ml2, each once from the first state; both fail the light test; phone back in fastboot, slot a current and successful, nothing written

Considered
: next: read ramoops (author unlock) or UART; watchdog node address check

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-how-the-phone-stands-ml"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: Phone in fastboot (FA81N1A00790), slot a current+successful, b unbootable retry 1; stock Android on a is what runs; no kernel kept

Considered
: next: panic read by ramoops or UART, decided by the walleye coordinator

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-builder-rust-ci"></a>
### Builder: Rust CI for ffs-acm-rs

Entry date
: 2026-11-17

Done
: Added a rust job to check.yml (fmt --check, clippy --all-targets -D warnings, test in build/ffs-acm-rs); all three pass on the lab first. Split walleye-code-hygiene-rust-ci: CI part done; C probe removal left open, blocked by walleye-ffs-rust-try because the coordinator's note says remove it only after n3r is tried and that try has not happened. No image built, nothing queued for the phone, phone not touched.

Considered
: removing the C probe now: n3 retry and n3r try are still open

Rejected
: Removing the C probe now: it would break the order the coordinator set

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-builder-ml3"></a>
### Builder: ml3, ramoops at the phone's own region

Entry date
: 2026-11-17

Done
: Moved the dts ramoops node to 0xa1100000 (2 MiB, record 0x80000, console 0x20000, no pmsg); rebuilt the dtb (build-kernel.sh, out-ml); build/build-ml3.sh made ~/Projects/walleye-build/images-ml3/boot-ml3.img with 10 checks ok: node by fdtget, old node gone, no carve-out overlap, PSTORE_CONSOLE and PSTORE_RAM =y, ramdisk and kernel bytes equal ml1's, only the dtb part differs. sha256 boot-ml3.img 5c2574706882f5c8f40b134bdf501e22c092093e494b5aad25efdabe4ba105c4; initramfs 9feb844211c3e5b636b2481eb920d36cdf698eaf4441090574edd943a69b7e01 (same as ml1); ml1 41d8efc0..., ml2 63b2977f... unchanged. Queued walleye-try-ml3-ramoops.

Considered
: A new kernel build dir: not needed, the kernel is unchanged and only the dtb is rebuilt.

Rejected
: Reading ramoops parameters from the phone: builders never touch it.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-keeper-m1-window"></a>
### m1 window read: lpoco-host acceptance passes

Entry date
: 2026-11-17

Done
: Took the lease; one m1 try from slot a; read the window on host 7.3.4 as the lab user (pass). Phone back in fastboot via adb reboot bootloader: a active+successful, b unbootable retry 1; nothing flashed.

Considered
: next: walleye-ffs-rust-try (n3r, ttyACM readable now); then ffs-helper-try n3 may close

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-how-the-phone-stands-m1"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: Phone in fastboot (FA81N1A00790), unlocked, slot a active+successful, b unbootable retry 1; stock Android on a; no kernel kept; nothing flashed. m1 window readable by the lab user; the vendor kernel runs our init.

Considered
: next: walleye-ffs-rust-try (n3r), then the vendor kernel's reserved-memory/ramoops facts for mainline

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2340-walleye-ml3-try"></a>
### ml3 try: pstore empty

Entry date
: 2026-11-17

Done
: Took the lease; booted boot-ml3.img once from the first state (slot a, b unbootable by the first state, nothing flashed); no USB light; Android back on a after 72 s; adb root read /sys/fs/pstore (empty) and the ramoops parameters (0xa1100000, matches); wrote tests/host/ml3-ramoops-read.sh (fails); left the phone in fastboot on slot a, good, lease to release.

Considered
: Reading raw DDR at 0xa1100000 (no /dev/mem on this kernel); a ml4 with earlycon on the msm serial (blind without UART).

Rejected
: Re-trying ml3: the result is deterministic; ml1/ml2/ml3 all end alike.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2330-walleye-ml4-build"></a>
### ml4 built

Entry date
: 2026-11-17

Done
: build/build-ml4.sh, build/initramfs-mainline/init-ml4, tests/host/ml4-bin-table.sh; images-ml4 with checksums; walleye-try-ml4 queued for the phone

Considered
: reusing build-mainline-light.sh (its checks assert ml1's init, so a separate script)

Rejected
: not petting the watchdog in the sleep: scores 6-7 would be cut by the 30 s reset and bins would saturate

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2345-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: n3r tried once on slot b: gadget enumerates (1d6b:0104, walleye-first-boot) but cdc_acm refuses it: Zero length descriptor references (-22); ffs-acm-up fails. Phone fell back to Android by itself, rebooted to fastboot: unlocked, slot b active+successful, a good; nothing flashed; lease released

Considered
: next: n3r2 with CDC functional descriptors (header, call-mgmt, ACM) in the helper; amends walleye-ffs-acm-single-interface - asked of the coordinator

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2350-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: ml4 tried twice from the first state (slot b, nothing flashed): back as Android at 75 s and 76 s, pstore empty. Phone in fastboot, unlocked, current slot b, a and b successful, lease walleye-keeper released at landing.

Considered
: next: tell score 7 from the watchdog (ml5 with a long score-independent sleep) - for the walleye coordinator

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-ml5-build"></a>
### ml5 built

Entry date
: 2026-11-17

Done
: build-ml5.sh, init-ml5a, init-ml5b, ml5-bin-table.sh; images in walleye-build/images-ml5 with SHA256SUMS; try walleye-try-ml5 queued

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0010-walleye-ml5-try"></a>
### ml5 try: kernel never reaches init

Entry date
: 2026-11-17

Done
: Took the lease; booted ml5a twice (Android back after 75 s, 75 s) and ml5b twice (75 s, 78 s) by fastboot boot from slot b (both slots good, nothing flashed). ml5a base 75 s is the ~70 s bin: the mainline kernel never reached init, as ml1-ml3; ml5b - ml5a = 0 and 3 s, so no init ran. Phone left in fastboot, slot b active and successful, a successful; lease released.

Considered
: UART/earlycon before init; a vendor-kernel-side check, for the coordinator.

Rejected
: Re-trying ml5: four tries agree, deterministic.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-pk-build"></a>
### pk1 and pk2 built

Entry date
: 2026-11-17

Done
: build/pk-kernel.sh, build/build-pk.sh; images in walleye-build/images-pk with SHA256SUMS (research/walleye-first-fastboot-boot/2026-11-18-pk); try walleye-try-pk queued

Considered
: porthole's own cmdline and stub dtbo: not taken, to keep ml5a's image the only other difference

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0030-walleye-pk-try"></a>
### pk1/pk2 try: kernel never reaches init

Entry date
: 2026-11-17

Done
: Booted pk1 twice then pk2 twice by fastboot boot from slot b (both slots good, nothing flashed). Android back after 75, 75, 75, 75 s: the ~70-78 s no-init bin, same as ml5a (75 s). porthole's v7.2.2+262-patch kernel (taimen/walleye dtb) does not reach init either; pk1 = pk2 so the dtb makes no difference. Returned to fastboot by adb reboot bootloader after each.

Considered
: Four tries agree, deterministic. Next for the coordinator: the failure is before init for every mainline kernel and both dtbs, so UART/earlycon or ramoops read (decision walleye-panic-read-order).

Rejected
: Re-trying pk1/pk2: four identical times

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0035-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: Phone in fastboot, unlocked, slot b current, a and b both successful, retry 0; nothing flashed; lease released

Considered
: next: coordinator decides the no-init vector (ramoops read / UART) for the mainline kernels

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-nocon-build"></a>
### pk3 and ml6a built

Entry date
: 2026-11-18

Done
: build/build-nocon.sh; images in walleye-build/images-nocon with SHA256SUMS (research/walleye-first-fastboot-boot/2026-11-18-nocon); try walleye-try-no-earlycon queued

Considered
: keeping androidboot.console=ttyMSM0: it is a stock androidboot word, not a kernel console=, and the item says keep the stock androidboot pieces

Rejected
: building the images twice with different tools: one script on the recorded images keeps every other byte equal

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1200-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: Ran the no-earlycon tries; phone in fastboot, unlocked, slot b current, a and b successful, retry 0; nothing flashed; lease to be released

Considered
: next: coordinator decides the log channel (ramoops read / UART) for pk3 vs ml6a

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-pk456-build"></a>
### pk4, pk5, pk6 built

Entry date
: 2026-11-17

Done
: build/build-pk456.sh: one swap each between pk3 and ml6a, no earlycon; 31 checks ok, rebuild identical; hashes in research/walleye-first-fastboot-boot/2026-11-18-pk456; try walleye-try-bisect-pk queued for the keeper

Considered
: Splitting each kernel blob at the dtb that ends it (as check-bisect.sh) and repacking with pk3's header and AVB props

Rejected
: Rebuilding the kernels or dtbs: the swap uses the recorded images' own bytes, so nothing else can differ

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-dtb-bisect-build"></a>
### d1..d6 built (our dtb bisect)

Entry date
: 2026-11-18

Done
: build/build-dtb-bisect.sh: six images, each pk4 with a node group removed from our dtb by fdtput; 90 checks ok, rebuild identical; hashes in research/walleye-first-fastboot-boot/2026-11-18-dtb-bisect; try walleye-try-dtb-bisect queued for the keeper

Considered
: Diffing the decompiled dtbs by path and property first, so each group is a node we have and porthole lacks or places differently

Rejected
: Removing spmi@800f000 as a group: the vendor dtbo overlay needs it, so that image would fail at the bootloader's merge and test nothing; adding our nodes to porthole's dtb (pk5 + group) in this round: removal from the dead dtb needs no kernel or dts rebuild and the reverse direction waits for a null result

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-pk5l-build"></a>
### pk5l built (first mainline userspace image)

Entry date
: 2026-11-18

Done
: build/build-pk5l.sh, init-pk5l, tests/host/mainline-acm-up.sh; 39 checks ok, rebuild identical; hashes in research/walleye-first-fastboot-boot/2026-11-18-pk5l; try walleye-try-pk5l queued rank 1

Considered
: One gadget config with both functions so a single bind gives console and window; petting the watchdog with a held-open fd 3 and a hold file to stay up

Rejected
: Two images (ACM only, window only): a failure of one function would then hide the other; a getty/login: a respawned ash on the tty is the least that answers

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1700-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: pk5l tried twice from fastboot, no ACM, fell to Android both times (57 s, 42 s); phone in fastboot, slot b current, a and b successful, nothing flashed, stock kernel

Considered
: next: walleye-try-dtb-bisect (d1..d6); pk5l's init may not start at all, a console-less sign is the return time only

---


<a id="2026-11-18-walleye-pk5l2-build"></a>
### pk5l2 built (ramoops in the phone's own region)

Entry date
: 2026-11-18

Done
: build/build-pk5l2.sh: pk5l's kernel blob split, the dtb's ramoops node edited with fdtput to 0xa1100000/2 MiB (record 0x80000, console 0x20000, pmsg 0), reassembled with pk5l's ramdisk, header and footer; 20 checks ok, rebuild identical; try walleye-try-pk5l2 queued rank 1

Considered
: dtc round trip to rename the node to ramoops@a1100000

Rejected
: dtc dts round trip: it re-emits the whole dtb, so more than the ramoops node would change; fdtput keeps size and every other byte

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1800-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: pk4 and d1..d6 twice each, all back as Android at 75 s; phone in fastboot, slot b current, a and b successful, nothing flashed, stock kernel; lease released

Considered
: next: research/build item to diff our dtb vs porthole's and build d7+

---


<a id="2026-11-18-walleye-taimen-walleye-build"></a>
### Overlay analysed, taimen-to-walleye images built

Entry date
: 2026-11-18

Done
: Merged the dtbo overlay into pk5's and our dtb and diffed: inert on ours beyond one i2c node. Diffed porthole's taimen and walleye dtbs: 15 nodes disabled, ~100 absent. Built w1..w5, wr, wa (399 checks, identical rebuild; programmer review fixed phandle check, header, stale sums).

Considered
: Overlay-label neutralizing images L1..Ln as the item said

Rejected
: L1..Ln: the analysis falsified their premise

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-2000-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: wa tried 3 times (42, 63, 63 s); phone in fastboot, slot b current, a and b successful, nothing flashed, stock kernel; lease released

Considered
: next: decide whether 63 s is a new return class (a ramoops read, or wa x2 more); then w1..w5, wr twice each as planned

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-2100-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: w1..w5, wr x2 each tried (42-63 s, all alive); phone in fastboot, slot b current, a and b successful, nothing flashed, stock kernel; lease released

Considered
: next: bisect properties between taimen dtb and pk2's dtb (not nodes)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-property-bisect-build"></a>
### wp1..wp5 and wpa built (property bisect)

Entry date
: 2026-11-18

Done
: Diffed wa and pk2 property by property (phandles resolved): five groups, nothing in memory/reserved-memory/chosen/cpus/timer/psci. Built wp1..wp5, wpa with build/build-property-bisect.sh (68 checks ok, rebuild identical); queued walleye-try-property-bisect for the keeper

Considered
: wa vs pk2 differ in phandle numbering, node order and dtb size too; wpa tests whether any property matters. Mirror images from our dtb after the first results

Rejected
: Binary halves: five groups, singles are as cheap

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-2200-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: wpa x2 tried (60 s each, alive); phone in fastboot, slot b current, a and b successful, nothing flashed, stock kernel; lease released

Considered
: next: bisect phandle order and dtb size between taimen dtb and pk2's dtb (not properties)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-pk5g-build"></a>
### g1..g3 built (mainline gadget staged on pk5)

Entry date
: 2026-11-18

Done
: build/build-pk5g.sh and init-pk5g: three images on pk5's kernel/dtb/header, ramdisks differing in /stage; 58 checks ok, rebuild identical; try walleye-try-pk5-stages queued

Considered
: three separate inits (rejected: one init + /stage keeps the diff to one file); 60 s holds so alive and dead are 30+ s apart

Rejected
: ACM and a shell loop in these steps: pk5l had them and ended early

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-2300-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: phone is gone from USB (no fastboot, no adb; devlab fastboot state: off or needs hands); pk5g stages g1..g3 (backlog walleye-try-pk5-stages) not started; nothing flashed, stock kernel, last known slot b current, a and b successful

Considered
: next: a hand power-cycle (or cable reseat) brings it back to Android/fastboot; then take walleye-try-pk5-stages

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-battery-watch-built"></a>
### walleye battery watch built

Entry date
: 2026-11-18

Done
: tests/host/battery-ok.sh (fastboot getvar battery-voltage or adb dumpsys battery; exit 1 below 3700 mV, skip when unread), tests/host/idle-android.sh (end state: Android booted, not in fastboot), tests/host/battery-selftest.sh (fake fastboot/adb, seen failing first, then pass); keeper rule added to AGENTS.md. Phone not touched.

Considered
: reading the phone for real in this run

Rejected
: reading the live phone now: builders never touch a phone; the selftest uses fake fastboot and adb

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-2330-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: g1,g2,g3 tried twice each (research walleye-pk5-stages-tried); phone back in Android on slot b (idle-android pass, battery 4377 mV), nothing flashed, stock kernel; lease released

Considered
: next: g3 never enumerated and g2 returns at 138 s; find what differs from expectation (UDC name, ordering, dtb usb node)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-t1-build"></a>
### T1 image built

Entry date
: 2026-11-18

Done
: build/build-t1.sh, init-t1, tests/host/pstore-t1-read.sh; boot-t1.img built from pk5, queued for the keeper

Considered
: Basing T1 on pk5l2 (its ramdisk is the ACM gadget; pk5 is the alive base)

Rejected
: Basing on pk5l2: sysrq c gives a deliberate end only if the init ran, and pk5l never showed it

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-2345-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: walleye-try-pk5-stages was already done and recorded by an earlier keeper (2330, #114); I found the phone off USB, then in Android, read fastboot state (b current, a and b successful), and ran g1 x2 and g2 x1 again: same result, no 1d6b:0104. Phone in Android on b, nothing flashed, stock kernel, lease released

Considered
: next: walleye-pstore-t1-try says run only after a kernel has shown life; none has, so it is a turning point for the coordinator

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0400-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: T1 tried on b (fastboot boot only, nothing flashed): back in Android after 159 s; battery 4384 mV/100% before. /sys/fs/pstore empty, no ramoops dir: our marker and panic text did not survive (cause B). Phone in Android on b, stock kernel, adb root, slots a and b good; lease released

Considered
: next: coordinator decides what follows cause B (pstore not retained across the sysrq c reset; another way to see the kernel's output)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-t3-build"></a>
### T3 image built

Entry date
: 2026-11-18

Done
: build/build-t3.sh, init-t3, tests/host/pstore-t3-read.sh; boot-t3.img built from pk5, queued for the keeper (backlog walleye-pstore-t3-wdog-bite). Phone not touched.

Considered
: Setting the 10 s timeout with busybox watchdog -T and one ping (-t 3600); the dtb's timeout-sec 30 alone

Rejected
: Opening /dev/watchdog with 'exec 3>' in sh: leaves the default 30 s (dtb) and a failed open would end init; busybox watchdog has the ioctl for the timeout

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-2359-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: T3 tried once: pstore empty, back as Android on b at 42 s, nothing flashed, stock kernel, fastboot reachable (slots a,b successful)

Considered
: next: turning point for the coordinator - empty pstore after an alive kernel with an unpetted watchdog does not tell no-bite from lost region; needs UART or another witness of init before more tries

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-vf1-build"></a>
### vf1 built

Entry date
: 2026-11-18

Done
: Built vf1 (init-vf1, build-vendor-facts.sh, vendor-facts-read.sh, selftest), reviewed by the programmer subagent, fixed, queued walleye-try-vf1 for the keeper.

Considered
: Copying the raw /sys/firmware/fdt for dtc: rejected, it carries the properties we drop by name; the per-property copy is enough.

Rejected
: A tar of live /proc and debugfs paths (sizes read as 0).

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-0430-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: vf1 tried twice from the first state: gadget up, LUN readable, header empty both times; phone back as Android on b, stock kernel, flags a and b successful, fastboot reachable, nothing flashed; host on 7.3.4 with usb_storage

Considered
: next: turning point for the coordinator - a first-round marker written at once by the vf1 init (or the dump step read) so an empty header can be told from a dead init; meanwhile the queue's other items wait on the author (unlock, UART)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-vf2-build"></a>
### vf2 built

Entry date
: 2026-11-18

Done
: init-vf2, build-vendor-facts-vf2.sh, reader and selftest extended; programmer review's five findings fixed; try queued

Considered
: a second 2 MiB window for the text (rejected: the LUN size cannot change after bind; one 8 MiB window with a 64 KiB text region)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0510-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: vf2 tried twice from the first state: init alive, steps run, beat ticks; phone left in fastboot on slot b, a and b successful, stock kernel, nothing flashed

Considered
: next: turning point for the coordinator - a poller that saves the whole 64 KiB text region every 2 s to files (not only the end) and fetches the blob the moment its header appears, since the phone leaves USB after ~45-60 s; reader's single-shot read is racy

---


<a id="2026-11-18-walleye-vf3-build"></a>
### vf3 built

Entry date
: 2026-11-18

Done
: init-vf3, build-vendor-facts-vf3.sh, vendor-facts-poll.sh and its selftest; programmer review's findings fixed; try queued

Considered
: appending with dd bs=1 seek bytes (rejected: slow; 4096-aligned rewrite of the last partial block instead); one nested dt/ directory tree in the poller (rejected: a block named dt/soc blocks dt/soc/spi; flat names with %)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-2400-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: vf3 twice from the first state: no gadget ever, no blocks, back as Android at 78 s both times (dead-init class). Phone left in fastboot on slot b, flags a and b successful, stock kernel, nothing flashed, battery 4377 mV

Considered
: next: turning point for the coordinator - vf2 init was alive, vf3 (gadget first, then progressive dump) is not; diff vf2 vs vf3 init, find what kills it before the gadget bind

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-vf4-build"></a>
### vf4 built

Entry date
: 2026-11-18

Done
: init-vf4 (vf2's first 86 lines byte for byte to the bind, then vf3's progressive dump from byte 65536), build-vendor-facts-vf4.sh, poller parametrised (VF_MARK/VF_HEAD/VF_BLOCKS, steps.txt), vendor-facts-poll-selftest-vf4.sh; try queued

Considered
: Keeping vf2's blob code after the bind as dead code so the diff has additions only: rejected, the diff is checked as 'identical up to the bind, first hunk after it'. A second poller for vf4: rejected, one poller with a layout by environment.

Rejected
: Moving the gadget earlier again (vf3's order) before knowing whether the early bind killed it.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0535-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: vf4 tried twice (poller first): no gadget, 0 blocks, empty beat log, back as Android at 78 s both; phone in Android on b, stock kernel, nothing flashed, fastboot reachable earlier (a and b successful)

Considered
: next: turning point for the coordinator - vf4 (vf2's first 86 lines + progressive dump) died like vf3 though vf2 lived: diff vf2 init vs vf4 init before the bind, or retry vf2 to see whether its 43 s gadget repeats; other queue items wait on the author (unlock, UART)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0610-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: vf control done: vf2 twice alive (gadget up, empty header), vf4 twice dead unwatched; phone back as Android on b, stock kernel, flags a and b successful, nothing flashed, fastboot reachable

Considered
: next: turning point for the coordinator - vf4 dies without the poller, so its progressive dump after the bind is the suspect; bisect the dump (vf4 with a vf2-like tail) or write a first-round marker at once

---


<a id="2026-11-18-walleye-vfx-build"></a>
### vfa..vfd built (one hardware source per image)

Entry date
: 2026-11-18

Done
: init-vfx (m1 plus one payload before the bind, chosen by /etc/walleye-vf), build-vendor-facts-vfx.sh (4 images, 23 checks each, rebuild identical), window-payload-read.sh and its selftest (18 ok); try queued as walleye-try-vfa

Considered
: Four init files: one init and a one-word file instead, so the four images differ in that file alone and the diff against m1 is one block. The beat in the window's last 4 KiB so a running init is told from a hung one without touching the payload. Header written first with no part, so a read that kills the kernel is told from a never-started init.

Rejected
: Keeping m1's 64 KiB text rewrite every 5 s: it would overwrite the payload. A tar or gzip of the payload, progressive appends and a poller: the dump after the bind is the suspect (vf3, vf4 dead).

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0650-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: vfa twice, vfb, vfc once, vfd twice tried from the first state through devlab fastboot boot; phone is Android on slot b, stock kernel, a and b successful, nothing flashed, fastboot reachable; the live fdt, iomem, interrupts, cmdline and regulator summary are in the material path

Considered
: next: build mainline USB nodes from the vfa fdt (porthole/taimen dtb bisect can now be compared against the live walleye fdt); vfb's clk_summary needs a look at why the read fails (debugfs path); fix battery-ok.sh to time out while Android runs; queue's other items wait on the author (unlock, UART)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-fdt-compare-build"></a>
### fdt compare and x1..xa built

Entry date
: 2026-11-18

Done
: Decompiled and normalized our, the alive, wpa and the running fdt; listed what ours lacks; built x1..x7 and xa with build-fdt-bisect.sh (114 checks, rebuild identical, programmer-reviewed); wrote the USB read of the running fdt; queued the try.

Considered
: A phandle-rewriting merge tool; splicing at source level.

Rejected
: Both: see the decision 2026-11-18-walleye-bisect-from-alive-by-fdtput.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-battery-ok-timeout-built"></a>
### battery-ok.sh timeout built

Entry date
: 2026-11-18

Done
: battery-ok.sh reads adb first, then fastboot, every call bounded (timeout -k 1 6); neither answers: skip, exit 2, 14 s worst. Selftest has hanging fake adb/fastboot. Programmer review: fixed exec bit, -k, missing timeout(1), selftest outer guard.

Rejected
: fastboot first: hangs the Android case the first 7 s for nothing; a longer timeout: keeps total under the 15 s bound

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-0700-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: fdt bisect x1..xa twice each from slot b: x1 and xa dead, others alive; phone back as Android on b, stock kernel, nothing flashed, fastboot reachable via adb reboot bootloader

Considered
: next: our dtb plus gpio-reserved-ranges <0 4>,<81 4>, try twice (alive expected); the earlier vf1 empty-header turning point may be this same cause

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-y-build"></a>
### y1 and y2 built (our dts with gpio-reserved-ranges)

Entry date
: 2026-11-18

Done
: Added &tlmm gpio-reserved-ranges <0 4>, <81 4> to our dts; build/build-y.sh builds boot-y1.img and boot-y2.img (49 checks, rebuild identical, hashes in the research material); programmer review; try queued as walleye-try-y.

Considered
: Taking the pshold/watchdog nodes along; rebuilding the whole kernel (only the dtb changes, Image.gz is ml6a's byte for byte).

Rejected
: Both, see decision 2026-11-18-walleye-our-dts-gets-gpio-reserved-only.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0800-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Tried y1 twice (alive, Android back 36 s) and y2 twice (gadget 1d6b:0104 at 18 s, ttyACM0): our dtb with gpio-reserved-ranges boots mainline to USB. Phone is Android on slot b, stock kernel, nothing flashed, fastboot reachable via adb reboot bootloader.

Considered
: next: read the ACM shell on y2 (dmesg, the facts the vendor dump was after); give mainline USB its nodes properly; consider restart@10ac000 and the apps watchdog from the fdt compare; upstream the gpio-reserved-ranges commit

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1900-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: walleye-keeper wrote tests/host/mainline-shell-read.sh (reads y2's ttyACM: dmesg first, facts, device lists, touch /tmp/hold, redacted, saved outside git) and mainline-shell-selftest.sh (fake phone on a socat pty; 8 ok); host only, phone untouched: Android on slot b, stock kernel, nothing flashed, fastboot reachable via adb reboot bootloader. Item walleye-first-mainline-shell stays open: the real read needs a y2 try

Considered
: next: try y2 (our dtb + gpio-reserved-ranges) twice from slot b with WAIT=60 mainline-shell-read.sh running, then write the research row of what the dmesg shows

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0830-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Reader fixed (echo-matching marker, ANSI strip) and run on y2: first mainline dmesg read. Phone in fastboot, slot b current, a/b successful, stock kernel, nothing flashed.

Considered
: next: PMIC spmi reg nodes (pm8998/pmi8998), smmu timeout, then charger/display

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-research-bringup-order"></a>
### walleye-research: the bring-up order plan

Entry date
: 2026-11-18

Done
: Read the first mainline map, the vendor hardware inventory and the mainline path; checked public sources for the charger, fuel gauge, haptics, amps and panel; wrote the plan material, one research row, one decision and eleven queued items with proofs.

Considered
: UFS gated by the PMIC (the brief's guess) - the vendor tree puts its rails on the RPM, so no. Display first.

Rejected
: Queuing a phone try: the phone is off USB (walleye-phone-needs-hands); the first tries come from the builds.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-builder-pmic"></a>
### walleye builder: PMIC nodes, z1 built

Entry date
: 2026-11-18

Done
: walleye-pmic-spmi: dts includes pm8998/pmi8998, build-z.sh builds z1 (16 checks, rebuild identical), pmic-dmesg-check with selftest (12 ok; programmer review fixed the loose overlay filter, added 0-01 and the probe-failed line, the check says registered, not working), the reader also lists /sys/bus/spmi/devices; queued walleye-try-z for the keeper.

Considered
: Own pmic nodes named like the overlay's; zero-line proof.

Rejected
: See decision 2026-11-18-walleye-pmic-from-mainline-dtsi.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0900-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: z1 tried once from slot b: alive, ttyACM0 up, reader denied the tty so the PMIC dmesg was not read; phone back as Android on b, stock kernel, nothing flashed, adb up, fastboot reachable on demand

Considered
: next: fix tty access for the keeper session (getfacl during the try; uaccess tag only reaches the seat user), then try z1 again with the reader waiting; coordinator to decide who fixes access (host prepare is lab-tools')

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-builder-dmesg-clean-z2"></a>
### walleye-builder: dmesg-clean, z2 built

Entry date
: 2026-11-18

Done
: Found the smmu -110 cause (gpucc =m, no modules), set it =y, rebuilt the kernel, built boot-z2.img with 19 checks, wrote dmesg-clean-check.sh and its selftest (seen failing on the first map), queued walleye-try-z2 for the keeper.

Considered
: Pointing the overlay's clock_gcc label at the real gcc by fdtput for the serial fault.

Rejected
: Putting all three fixes in one image: one change per image.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0930-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: z1 tried twice from slot b with the reader: gadget and ttyACM0 up at 18 s, ACL held, shell silent; phone back as Android on slot b, stock kernel, nothing flashed, fastboot reachable via adb reboot bootloader; walleye-try-z back to open

Considered
: next: does y2's ttyACM0 answer at all (y2 once with the reader)? else read facts through the mass_storage window; walleye-try-z2 after

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0945-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: y2 once then z1 once with the same reader: both silent (gadget 18 s, ttyACM0, no bytes); phone back as Android on slot b, stock kernel, nothing flashed, adb up, fastboot via adb reboot bootloader

Considered
: next: builder makes a banner init (ttyGS0 every 2 s, kmsg marker per step) on y2 first then z1; keeper reads with plain cat before any reader; control item walleye-try-y2-control done

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-builder-dmesg-clean-z3"></a>
### walleye-builder: dmesg-clean, z3 built

Entry date
: 2026-11-18

Done
: Read the first dmesg: the four phandle lines and the serial wait are one cause (the clock_gcc stub has no #clock-cells). Wrote tests/host/overlay-clock-gcc.sh (all four users; seen failing on z2's dtb), build/build-z3.sh (z2 + the clock_gcc symbol on the real gcc, 15 checks, rebuild identical, reviewed by the programmer agent and its findings fixed), queued walleye-try-z3.

Considered
: Pointing the symbol at gcc by fdtput after the compile; the shell being silent on the phone, so the try may learn nothing until the banner init exists.

Rejected
: Disabling overlay nodes; a fake #clock-cells on the stub; folding other fixes into z3.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0950-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: y2 tried with the old reader: silent too, only the tty echo arrives; phone back as Android on slot b, stock kernel, nothing flashed, fastboot reachable via adb reboot bootloader

Considered
: next: banner init (builder) and a raw cat of ttyACM0; devlab suite/phone unavailable: no password file at ~/.config/devlab/google-walleye/password (not created)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-builder-charger-c1"></a>
### walleye-builder: charger c1 built

Entry date
: 2026-11-18

Done
: Enabled pmi8998's charger and a simple-battery (2700 mAh, 4.4 V) in dt/, built the two drivers in, wrote build/build-c1.sh (z3 + new kernel + our dtb, 27 checks, rebuild identical), tests/host/charger-check.sh with its selftest, and one reader step that saves each power supply's type/online/voltage; queued the try.

Considered
: Building c1 on z3, which still waits for its own try (the silent shell): the charger try reads the same way, so both wait on the banner init and reader work.

Rejected
: Modules for the charger (the initramfs carries none); a fuel-gauge node now (capacity stays not known yet); changing the PMIC's JEITA bounds.

About
: walleye-charger-smb2

---


<a id="2026-11-18-walleye-builder-banner"></a>
### walleye-builder: the banner image

Entry date
: 2026-11-18

Done
: Built boot-banner.img (y2 + init-banner), build/build-banner.sh with 40 checks, tests/host/banner-check.sh and banner-selftest.sh; queued walleye-try-banner for the keeper; marked walleye-build-banner-init done.

Considered
: Building on z3 instead of y2 (the item says y2: one change per image, and the banner tells reader from phone first).

Rejected
: A shell on ttyGS0 in the banner image: the banner must not depend on anything reading it. A held-open tty fd: a plain open-write-close per line is simpler and survives the host reopening the port.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1012-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: banner try once from b: banner seen (root read); phone back as Android on b, stock kernel, nothing flashed, fastboot reachable via adb reboot bootloader

Considered
: next: re-read the earlier empty tries as a permission problem; redo the dtb gpio-reserved-ranges try with a reader that can open ttyACM0 (root-exec, or mll into uucp / a udev rule, the author's host change)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-builder-reader-eacces"></a>
### walleye-builder: reader retries denied tty, root-exec fallback

Entry date
: 2026-11-18

Done
: mainline-shell-read.sh retries the tty open while Permission denied (ACCESS_WAIT 20 s, 0.5 s steps); on lasting EACCES only, falls back to lab-agent root-exec for that one resolved /dev/ttyACM* node (stty, one-command writes, reads), prints the path used, logs each root use in OUT/root-exec.log. Selftest 21 ok (9 fail on old reader). Programmer review applied: positional node arg, EACCES-only, resolved-node check, timeouts, same stty on both paths, uid-0 guard.

Considered
: root cat in background as reader (tighter, not done)

Rejected
: chmod/chown/usermod, rule change: lab-prepare stays the author's; block readers unchanged: they skip on -r, not the same open

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-banner-serial-fix-built"></a>
### banner2 built: the serial fix

Entry date
: 2026-11-18

Done
: init-banner serial walleye-banner -> walleye-first-boot; build/check-init-serials.sh (every init, no unread form skipped); build-banner.sh runs it and takes BANNER_NAME; boot-banner2.img built twice identical, 41 checks; programmer review applied; walleye-banner-serial-fix done, walleye-try-banner2 queued for the keeper.

Considered
: check inside build-banner.sh only for init-banner

Rejected
: a name-guess grep over serial forms: it silently skipped gst/unquoted forms; now any unread serialnumber line fails

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1030-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: banner2 tried once from slot b: banner read as lab user, proof passes. Phone back as Android on b, stock kernel, nothing flashed, fastboot reachable via adb reboot bootloader

Considered
: next: z3 and c1 tries with the reader (our dtb plus gpio-reserved-ranges) twice each

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1120-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: z3 tried twice from slot b (fastboot boot from memory, nothing flashed): both times the image came up as USB 1d6b:0104 at 18 s, but mainline-shell-read.sh got Permission denied opening /dev/ttyACM0 and saved an empty dmesg, so nothing is learned about the dmesg. Inside this keeper session (sandboxed Bash) a plain open of /dev/ttyACM0 is denied although getfacl shows user:mll:rw-; probably the sandbox, not the udev ACL. Phone is back as Android on slot b, stock kernel, both slots good, battery 4357 mV

Considered
: next: run the z3 reader with the sandbox off (needs the author's or coordinator's ok), or from a terminal tab of the user; walleye-try-z3 stays open. Not done: no third try, no root-exec (withdrawn)

---


<a id="2026-11-18-1135-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: z3 try 1 booted once from b (gadget at 18 s); reader job never picked up by lab-agent, so no dmesg; phone Android on b, stock kernel, nothing flashed, fastboot reachable via adb reboot bootloader

Considered
: next: the coordinator fixes why submitted jobs are not picked up (agent running, jobs not seen), then z3 twice, c1 twice; z3-try stays open

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1140-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: z3 booted once from slot b: gadget up at 18 s, back in Android on slot b, stock kernel, nothing flashed, fastboot reachable via adb reboot bootloader; reader denied /dev/ttyACM0 so no dmesg

Considered
: next: make /dev/ttyACM0 readable for mll (udev rule/group; host change, coordinator's), fresh reader, then z3 twice and c1 twice

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1148-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: walleye-keeper took walleye-try-z3 and booted z3 once from slot b (fastboot boot, nothing flashed) while the coordinator's tty watcher ran: the node was rw for mll all along; the reader saw silence; the Permission denied was the final write to the vanished node. Phone back as Android on b via adb, stock kernel, flags b/a successful, fastboot reachable via adb reboot bootloader. Item left open with the finding.

Considered
: next: read what z3's ramdisk init does on ttyGS0 (banner, shell, another tty) before any further try; a plain cat of the node during a try would show whether anything is sent. Earlier next (dtb plus gpio-reserved-ranges, twice) still stands.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-builder-reader-drop-root-exec"></a>
### walleye-builder: reader drops the root-exec fallback

Entry date
: 2026-11-18

Done
: mainline-shell-read.sh no longer calls lab-agent root-exec (grep finds none in tests/host and build); the 20 s retry on Permission denied stays; a still-denied tty now fails with owner, group, ACL, the usb parent's serial and who holds the port. Selftest: root-exec cases replaced by a still-denied case (fails cleanly, lab-agent never called, nothing written to the phone); 19 checks ok, seen failing first.

Considered
: keeping root-exec as a logged last resort

Rejected
: root-exec fallback: the no-root decision stands (2026-11-18-walleye-banner-serial-not-root-exec), the cause was the image's serial

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-z4-the-acm-shell-that-answers"></a>
### z4: the ACM shell that answers

Entry date
: 2026-11-18

Done
: init-z4, build-z4.sh, one-fd reader, hang-up fake in the selftest, README; programmer review fixed three bugs; boot-z4.img 4115db3e2c678919cb3d3bd725d5065bbbbac1e6026732989c1eca3590043b20; try queued

Considered
: cttyhack, setsid -c

Rejected
: cttyhack (review)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1155-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Android on slot b, stock kernel, nothing flashed, fastboot reachable via adb reboot bootloader, slots a/b successful; z4 tried once: gadget up at 18 s but the ACM shell stayed silent (dmesg empty)

Considered
: next: z4's console block not seen on the phone; read the kmsg counter / pstore as root-less route, or decide vector with coordinator; item walleye-try-z4 left open

---


<a id="2026-11-18-z5-banner-reports-shell"></a>
### z5: the banner reports the shell's state

Entry date
: 2026-11-18

Done
: Built z5 (init-z5, build-z5.sh, banner-shell-check and two selftests), reviewed by the programmer agent, fixed its findings, queued walleye-try-z5.

Considered
: Redirecting the shell's own stderr to a file (loses the prompt and errors on the tty); a probe start whose output goes to a file instead

Rejected
: Having the banner run stty itself each beat: extra tty ioctls on the banner path; the console block records the tty flags once after its own stty

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-0800-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: z5 tried once from slot b (research walleye-z5-tried): shell alive, tty full of prompts and errors; phone back as Android on b, stock kernel, nothing flashed, fastboot reachable via adb reboot bootloader, battery 4376 mV

Considered
: next: z6 = z5 with the tty set raw/no-echo before the shell starts (hypothesis: the banner echoes into the shell); the earlier x1 gpio-reserved-ranges item stays open

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1325-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: z5b tried once from slot b: gadget up at 18 s; reader job never ran (lab-agent GitHub token 401). Phone is Android on slot b, stock kernel, nothing flashed, fastboot reachable via adb reboot bootloader, battery 4376 mV

Considered
: next: author renews lab-agent token (lab-agent connect), then z5b again with the same job script; the x1 gpio-reserved-ranges item stays open

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-z6-command-loop-built"></a>
### z6: a command loop instead of the silent ash

Entry date
: 2026-11-18

Done
: Built init-z6 (z5 with the console block replaced by a read/exec loop on one fd), build-z6.sh, the pty selftest z6-loop-selftest, banner-loop-check and its selftest; the reader leaves banner lines out. Reviewed by the programmer agent. Queued walleye-try-z6.

Considered
: Reading with < /dev/ttyGS0 per line as the item sketched; an end marker printed by the loop.

Rejected
: Reopening the tty per line (bytes arriving between closes can be lost, and a close with hangup could end the session): the port is opened once, as z4 decided. An own end marker (__END_n__): the reader's command already carries its marker, a second one would only add a line to each answer.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-0900-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: z6 tried once from slot b (research walleye-z6-tried): FIRST working mainline shell - commands answer, dmesg 590 lines (Linux 7.3.0), SPMI pm8998/pmi8998 devices 0-00..0-03 present, pmic-dmesg-check passes; no drm, no power_supply class. Phone is Android on slot b, stock kernel, nothing flashed, flags a and b successful, fastboot reachable via adb reboot bootloader, battery 4381 mV.

Considered
: next: read the 14 deferred/fail lines of the z6 dmesg (research/.../2026-11-19-z6-try1/read/dmesg.txt) and the PMIC children (charger, fuel gauge); the shell makes dmesg-driven bring-up possible: next item by the bring-up order (PMIC, clean probes, charger, UFS read-only). The earlier x1 gpio-reserved-ranges item is answered by y/y2 (in our dtb). Procedure that worked: stty loop + banner capture + one-fd reader as plain background processes started before fastboot boot now IMAGE.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-research-z6-dmesg-map"></a>
### walleye-research: z6 dmesg map

Entry date
: 2026-11-18

Done
: Read the z6 dmesg, grouped five faults, recorded research walleye-z6-dmesg-mapped, decision 2026-11-18-walleye-z6-dmesg-fix-order, reranked dmesg-clean/try-c1/ufs-readonly, queued dtbo-overlay-id (PR 218).

Considered
: fw_devlink=permissive; powering the GFX rail.

Rejected
: Both: hide or premature.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-z7-dmesg-clean-built"></a>
### z7: the GPU smmu off

Entry date
: 2026-11-18

Done
: Built z7 (build-z7.sh), the dts line, adreno-smmu-disabled check and selftest, dmesg-clean-check's gpu_gx count and known-overlay tolerance. Queued walleye-try-z7.

Considered
: Building z7 from the dts (would also bring the charger node, a second change).

Rejected
: A second change in the same image; one change per image keeps a try readable.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-1000-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: z7 tried once from slot b: dmesg clean of the gpu_gx WARN and smmu deferral (323 lines). Phone is Android on slot b, stock kernel, nothing flashed, flags a and b successful, fastboot reachable via adb reboot bootloader, battery 4385 mV.

Considered
: next: per decision 2026-11-18-walleye-z6-dmesg-fix-order - c1 on the z7 init with debugfs and facts, then UFS read-only, USB net, touch, display; the psci PC-mode and misaligned-image Firmware Bug lines remain (vendor-overlay noise left)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-c2-built"></a>
### c2 built

Entry date
: 2026-11-18

Done
: build/build-c2.sh: z7 + the charger kernel + the charger/battery dtb; images-c2 hashed; try queued, try-c1 closed as superseded

Considered
: re-applying the c1 diff by hand on z7's sources: the dts already carries both the charger and the smmu line, so the dtb is a plain build of the dts

Rejected
: a dtb diff against z7 by line count: z7's dtb is an older dts's and phandles shift; the check normalises values and compares the sorted lines

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-1100-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: c2 tried once from slot b: command loop ok, charger deferred on tps61099 supplier (no driver), no power_supply class. Phone is Android on slot b, stock kernel, nothing flashed, flags a and b successful, fastboot reachable via adb reboot bootloader, battery 4376 mV.

Considered
: next: item walleye-charger-wait-tps61099 (dts choice: drop the supply link, stub tps61099, or fw_devlink=permissive) to the walleye coordinator; then c3; else the z7 order (UFS read-only, USB net, touch, display)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-z8-built-c2-plus-pmic-watchdog"></a>
### z8 built: c2 plus PMIC watchdog

Entry date
: 2026-11-18

Done
: Built kernel (v7.3 + e123d3bff, PM8916_WATCHDOG=y, dts pm8998_wdt okay wakeup-source), build/build-z8.sh with all checks ok (rebuilt twice, same hashes), programmer review applied. boot-z8.img sha256 797778f27d7aa0c8e93cbe6d101678132da3f94bc5536c672f4f740767dce0b5, dtb 1cb054698d37202a64c0408975023374ee429605cd08b04727f491a62c43946b, in ~/Projects/walleye-build/images-z8/. Not tried on the phone.

Considered
: tying Image.gz to the commit: only the compatible string is checked; not known yet beyond that.

About
: deep-sleep

---


<a id="2026-11-18-1500-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Took the lease; phone is Android on slot b, stock kernel, adb up (FA81N1A00790), nothing changed this run. phone-next offered only walleye-debug-without-display, which is blocked_by walleye-first-fastboot-boot, has no built image or test, and the pstore chase is paused by decision 2026-11-18-walleye-vendor-kernel-as-instrument; unattended, so not started. lpoco lab suite latest fails: no devlab password file for google-walleye (needs PHONE_PASS or ~/.config/devlab/google-walleye/password).

Considered
: next: walleye-charger-wait-tps61099 (needs research; decision 2026-11-18-walleye-charger-supplier-permissive-then-modelled already says permissive devlink first, then a modelled fixed regulator) - build c3 with fw_devlink=permissive, then a try on slot b; the queue's phone item should be that try once c3 is built. Also: phone-next lists a blocked item; its blocked_by is not honoured by the query.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-c3-built"></a>
### c3 built

Entry date
: 2026-11-18

Done
: build/build-c3.sh (failed first on a missing input), built through lpoco-lab job start, log read; try queued as walleye-try-c3; walleye-charger-supplier stays open for c4.

Considered
: lpoco-bringup image swap --cmdline (would work; c2's own script pattern kept for its AVB salt/props checks)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1510-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: PMIC watchdog: z8 had none (qcom-pon =m); z9 (=y) bit unpetted, gone 20 s after arming (pmic-wdt-bites pass, landed PR 236). Phone is OFF/not on USB since 14:54 after the bite: not Android, not fastboot - needs the author's hands (hold power ~15 s). Slots untouched (b current, a and b successful as of 14:49). Nothing flashed.

Considered
: next: author power cycle (item walleye-needs-power-cycle-after-pmic-bite), then check fastboot; lane coordinator to decide the PMIC bite's reset type before another bite; c3 try (c2 + fw_devlink=permissive, PR 235) still queued; wake-in-suspend waits for walleye to suspend at all

---


<a id="2026-11-18-1530-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: c3 tried once from slot b: charger probes with fw_devlink=permissive (charger-check pass). Walleye is HUNG in c3 (silent, USB 1d6b:0104, no adb/fastboot) since 15:13; may recover slowly as after z9, else needs the author's power hold. Nothing flashed, slot b active.

Considered
: next: item walleye-c3-hung-wait-then-power-cycle first; then c4 (tps61099 fixed regulator, walleye-charger-supplier). PMIC bite proof stays barred by decision 2026-11-18-walleye-no-pmic-bite-until-reset-type

---


<a id="2026-11-18-1535-c4-built"></a>
### c4 built

Entry date
: 2026-11-18

Done
: walleye-builder: read c3's try (charger probes with fw_devlink=permissive), read the tps61099 facts from the vfa fdt and the dtbo, modelled it in our dts, built boot-c4.img through lpoco-lab job start (log read from the journal; 26 checks ok, rebuilt identical), programmer review, queued walleye-try-c4.

Considered
: A regulator-fixed at the overlay's node; the build in the v7.3+wdt tree (it would add z8's watchdog to the dtb).

Rejected
: Both - see decision 2026-11-18-walleye-tps61099-fixed-regulator-supplier-disabled.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-builder-c5-and-c6-built"></a>
### walleye builder: c5 and c6 built

Entry date
: 2026-11-18

Done
: Built c5/c6 as one image (charger as a module, loaded late by command line): build-c5.sh, job-c5.sh, init-c5, c5-loader-selftest; 40 checks ok, rebuild identical, programmer review fixed.

Considered
: Building c5 and c6 as two inits; one image keeps them equal but the command line.

Rejected
: Cap of 150 s for the delay: the 180 s self-reboot could race a late bind; 120 s used.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-phone-state-check-over-several-witnesses"></a>
### Phone-state check over several witnesses

Entry date
: 2026-11-18

Done
: Built tests/host/phone-state.sh: verdict android|fastboot|mainline-answering|mainline-silent|not-known|gone over adb, fastboot, sysfs usb, the gadget's own tty; never 'hung'. Selftest with fakes, programmer review fixed (tty echo/hupcl, tty node bound to the gadget, adb recovery states, age dropped). AGENTS.md rule added.

Considered
: Age of the listing from sysfs mtime

Rejected
: Age witness: sysfs mtimes are lazy and say nothing of enumeration time, so it is shown as not known yet; usb reset probe left out of the first version as the item says

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1610-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Took walleye-try-c5: tried once from slot b, restart gate passed (180 s). Phone is Android on slot b, stock kernel, nothing flashed, flags a and b successful, fastboot reachable by adb reboot bootloader, battery 4378 mV.

Considered
: next: walleye-try-c6 (charger module loaded 40 s after enumeration) - the gate passed, so it may run; it needs a PMIC-side deadman decision first if its try is longer than 180 s (decision restart-gate-before-charger-try); else walleye-dtbo-minimal-try

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-wire-phone-state"></a>
### Wire phone-state into restart-check and readers

Entry date
: 2026-11-18

Done
: restart-check.sh consults phone-state.sh when the journal has no disconnect; five readers print its verdict not 'hung'; selftest 11 ok

Considered
: consult by default without NOW

Rejected
: consulting phone-state against a recorded journal: today's phone proves nothing about it (programmer review)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-google-walleye-lab-follows-docs-device-layer"></a>
### google-walleye-lab follows docs' device layer

Entry date
: 2026-11-18

Done
: Copied device-template #26: devicebase.toml pin (docs 0.17.0), device:begin/end markers around the device tables, generic list removed, idealkb.toml why text, AGENTS.md line. follow.sh against docs v0.17.0 found the device part and every device statement already identical, so nothing else moved.

Considered
: Running follow.sh first with no markers: it would append a second copy of the device tables at the file's end.

Rejected
: Keeping our own generic list: docs' device-layer/ is the one owner.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1840-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Phone is Android on slot b, stock kernel, nothing flashed, adb up. Took walleye-pmic-wdt-read-s2-regs: wrote tests/host/pmic-wdt-s2-regs-read.sh (read only), booted z9 from memory via lpoco lab fastboot boot now; it answered (1d6b:0104, uname 7.3.0-00004-ge123d3bff47d-dirty; spmi regmaps 0-00..0-03 named pmic-spmi under debugfs). First run: no PON register lines read. Second run (NOREBOOT) answers came back as blank lines only. Registers NOT yet read; item stays open. Phone left the mainline image by itself after about 15 min (off USB for part), back on Android b.

Considered
: next: debug the tty reader (answers blank; compare with mainline-shell-read.sh which worked; check the regmap registers format); retry z9; walleye-pmic-wdt-readback-image is the fallback; dtbo item stays held

About
: deep-sleep

---


<a id="2026-11-18-walleye-builder-z10"></a>
### walleye-builder: z10

Entry date
: 2026-11-18

Done
: Built z10 from z9 (build/build-z10.sh, 27 checks), init-z10 with /bin/pon-regs, the selftest and the phone test pmic-wdt-pon-regs-banner; programmer review applied; queued the keeper's try.

Considered
: A kernel change (S2 type by dt property with read-back) as the backlog first said: needs a kernel build and a push of the series, and the registers must be read before any type is chosen.

Rejected
: Kernel route now: the coordinator's order was the registers in the banner, reads only; the property comes after the values are known.

About
: deep-sleep

---


<a id="2026-11-18-1915-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Phone is Android on slot b, stock kernel, adb up, nothing flashed, flags unchanged. z10 tried once from memory: no PON register values (banner 'not read yet'); item walleye-pmic-wdt-z10-read-pon-regs stays open.

Considered
: next: why init-z10 never read the registers; walleye-pmic-wdt-read-s2-regs; walleye-dtbo-minimal-try question to coordinator still open

About
: deep-sleep

---


<a id="2026-11-19-walleye-builder-z11"></a>
### walleye-builder: z11, UFS read-only

Entry date
: 2026-11-18

Done
: Took walleye-ufs-readonly (first ready row). dts: l20/l26 rails, ufshc and ufsphy okay; config: UFS host and QMP UFS phy =y; init-z11 (z10 + ufs-lsblk, ufs-hdr, WALLEYE-BANNER-UFS); build-z11.sh with its checks; ufs-lsblk-selftest.sh and ufs-lsblk-check.sh (test walleye-ufs-lsblk). Built images-z11/boot-z11.img b4399ea3...; queued walleye-try-z11 for the keeper.

Considered
: Bringing the dtb change as a z9-style dtb-only image (rejected: the UFS host is =m in z9's kernel, so the kernel must change too).

Rejected
: Anything that writes or mounts: not in this image.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-z12"></a>
### walleye-builder: z12, Hynix UFS quirks

Entry date
: 2026-11-18

Done
: Took walleye-ufs-hynix-quirks. Kernel topic/ufs-hynix-quirks 12dca401a; build-z12.sh (15 checks); z12 sha256 f959c448 built, not run.

Considered
: Sony's pre-link-startup lane limit via Kconfig

Rejected
: Model-specific tables: the card's model is unknown

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-2330-how-the-phone-stands-ufs"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Android on slot b, stock kernel, nothing flashed, adb up. Item walleye-ufs-card-identity closed on the getvar-none branch (one fastboot visit, rebooted back).

Considered
: next: coordinator to say whether the exact UFS model needs a recovery root shell, since z11/z12 wait on it; walleye-pmic-wdt-z10-read-pon-regs is parked. Note: the earlier 2300 entry says the phone was off USB; it is back on Android now.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-builder-z13"></a>
### Hynix quirks review fixes, z13 built

Entry date
: 2026-11-18

Done
: Fixed findings 1,4,5,6 of walleye-z12-built; a second programmer review caught that the raw-path guard keyed on msgcode was bypassable, moved it to ufshcd_issue_devman_upiu_cmd by transaction code; z13 built, checks ok.

Considered
: Propagating DME write errors (would fail the UFS probe)

Rejected
: Keeping the PA_AVAILTXDATALANES write: read-only per spec, would warn every boot. Guarding in ufs_bsg alone: bypassable.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-1300-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Phone is Android (LineageOS userdebug) on slot b, stock kernel, nothing flashed or written; adb root was enabled (adbd restarted as root, disposable per decision 2026-11-18-walleye-android-disposable-no-bricking). Item walleye-ufs-card-identity-root done: UFS is SK hynix H28U74301AMR fw D003 64 GB (research walleye-ufs-card-identity-root).

Considered
: next: coordinator takes the UFS identity to the top coordinator for the UFS-probe decision; then walleye-pmic-wdt-read-s2-regs (read only); walleye-charger-wait-tps61099 dts choice; walleye-pmic-wdt-z10 stays parked

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-builder-z14"></a>
### walleye builder: z14 USB networking built

Entry date
: 2026-11-18

Done
: Built z14 (z7 + ECM/usb0/udhcpd/telnetd in the init), 26 checks, rebuild identical; queued walleye-try-z14 for the keeper; walleye-usb-net in progress until the try

Considered
: Dropbear static for ssh (no key on the lab host, no sshd in earlier images); NCM (panicked on the vendor kernel; ECM is in this kernel and is what the first images used)

Rejected
: ssh in the initramfs now: needs a key and a second build; telnet on the private cable is what gadget-up.sh proves

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-steps-claimed"></a>
### Walleye's steps claimed on the bring-up chain

Entry date
: 2026-11-18

Done
: Added steps = 1 to devicebase.toml; wrote build/claim-steps.sh (ikb step per step) and ran it: 113 claims (17 done, 3 partial, 1 failing, 92 open), each draft claim of docs checked against this base's research, backlog and decisions. Corrected two: charger-online partial (c3 online 1, restart gate open), watchdog-way-back failing (z9 bite, no way back). In a scratch copy of docs the feed reads them, bringup-chain-walleye.sh passes, daily-phone.sh google-walleye fails honestly (66 of 82 not done). Programmer review: no blocking problem; fixed the claim date (re-run identical) and the charger note. Open: research row walleye-c3-tried says z9 came back after 10 min, the z9 row says not - the c3 row's wording is for the keeper to correct.

Rejected
: unknown for untried steps (the chain test refuses it); copying the draft whole (two claims contradicted the records)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-1400-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: walleye-try-z14 done: z14 once from slot b, USB networking works (172.16.43.1 ping + telnet root shell); gadget-up.sh falls back to telnet when host lacks nc. Phone back as Android on slot b, stock kernel, nothing flashed, fastboot reachable.

Considered
: next: walleye-ufs-android-live / walleye-bootchain-readback (read only); z14 gives a network shell for probe reading

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-2330-how-the-phone-stands-ufs-readback"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: walleye-ufs-android-live done: read-only adb root readback of Android's UFS bring-up recorded. Phone: Android on slot b, stock kernel, nothing flashed, fastboot reachable.

Considered
: next: walleye-bootchain-readback (read only, needs a test written first)

---


<a id="2026-11-19-1600-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: walleye-try-z15 part 1: z15 once from slot b over the network shell; touch probe absent because i2c@c179000 keeps the downstream compatible qcom,i2c-msm-v2. Phone back as Android on slot b, stock kernel, nothing flashed, fastboot reachable, slots a and b successful.

Considered
: next: walleye-build-z16-i2c-compatible (builder); z15 part 2 waits for the author; then walleye-ufs-android-live / walleye-bootchain-readback (read only)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-builder-z16"></a>
### walleye-builder: z16 built

Entry date
: 2026-11-18

Done
: Read z15 part 1; found the overlay overwrites the qup node by name; wrote tests/host/i2c-qup-after-merge.sh (fails on z15, passes on z16) and build/build-z16.sh (26 checks, rebuild identical); recorded research, decision, the phone try queued.

Considered
: compatible edit, dtbo edit, soc symbol repoint, dtc round trip

Rejected
: all four, see the decision

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-1700-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: z16 part 1 once from slot b: i2c bus and rmi4_i2c bind, probe fails -5 (controller silent). Phone back as Android on slot b, stock kernel, nothing flashed, fastboot reachable, slots a and b successful.

Considered
: next: walleye-build-z17 touch power/reset (builder): check vdd/vio supplies, reset gpio89, delay against vendor tree; z15/z16 part 2 waits for the author; then walleye-ufs-android-live / walleye-bootchain-readback (read only)

---


<a id="2026-11-18-walleye-research-ufs-android-way"></a>
### walleye-research: UFS the Android way

Entry date
: 2026-11-18

Done
: Compared Google's wahoo 4.4 UFS/PHY/dts with z13 and the live readings by extracted lists (PHY tables, quirks, dts props) and by hand (reset, power-up, link startup). Finding: Android resets the card and uses one quirk and 2 lanes; z13 does neither. Queued walleye-ufs-android-regs (phone, read-only) and walleye-ufs-match-android-z (build).

Considered
: Reading msm-4.4 against v7.3 line by line (the author said no: diverged too far); the Lineage fork's UFS files (not read: Google's stock branch is the pairing with the stock bootloader).

Rejected
: Claiming a match or a safe probe: abl's hand-over, the HW version and the erase cause are unread.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-1800-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Android on slot b, stock kernel, nothing flashed, fastboot reachable; read UFS host regs read-only on live Android (research walleye-ufs-android-regs-read); HW_VERSION and PHY unreadable, no /dev/mem

Considered
: next: walleye-bootchain-readback (read only, rank 4); a reader for 0x1da7000 PHY; walleye-build-z17 touch power (builder)

---


<a id="2026-11-18-walleye-builder-z17"></a>
### walleye-builder: z17 built

Entry date
: 2026-11-18

Done
: Read z16's probe -5 and the phone's own dtbo overlay: the touch power switch is pmi8998 gpio 2, never driven by z16. Wrote tests/host/touch-power-after-merge.sh (fails on z16, passes on z17) and build/build-z17.sh (rebuild identical), the same regulator in the dts; programmer review fixed; research walleye-z17-built, try walleye-try-z17-part1 queued.

Considered
: Rebuilding the dtb from the dts: the kernel tree clone is stale; z16 was a dtb edit too, so z17 edits z16's dtb with fdtput. Driving gpio 75: its job is not known yet.

Rejected
: A gpio hog instead of a regulator: the supply belongs to the touch node's vdd so the driver's own order and delay apply.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-1900-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Android on slot b, stock kernel, nothing flashed, fastboot reachable, slots a/b successful. Tried z17 once: touch still silent, rmi4 defers on regulator-touch-power because pmi8998@3 lacks reg (research walleye-z17-part1-tried)

Considered
: next: build z18 (pmi8998@3 reg, builder); walleye-ufs-android-regs and walleye-bootchain-readback remain

---


<a id="2026-11-18-walleye-builder-z18-stopped"></a>
### walleye-builder: z18 not built, its premise is false

Entry date
: 2026-11-18

Done
: Read z17's dtb, the overlay merge, the dmesg and v7.3 spmi.c: pmi8998@3 of ours has reg; the reg-less lines are the overlay's nodes and are skipped. Recorded research walleye-z18-premise-falsified and a decision; the item stays open with the next step.

Considered
: Building z18 as the item says; a facts-reading init on z17's image next.

Rejected
: A no-effect build and a phone try.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-2130-how-the-phone-stands-z17-facts"></a>
### How the phone stands (after z17 facts)

Entry date
: 2026-11-18

Done
: Android on slot b, stock kernel, nothing flashed, a/b successful, fastboot reachable. z17 facts read once (no write): pmi8998 gpiochip2 exists, touch regulator still 'can't get GPIO'

Considered
: next: why gpiod_get fails on an existing chip (full dmesg order: pmic@2 gpio probe vs regulator-fixed); walleye-ufs-android-regs needs a reader decision; bootchain-readback remains

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-builder-overlay-collisions"></a>
### walleye-builder: overlay collisions test, fix, z19

Entry date
: 2026-11-18

Done
: Wrote the collision test and its selftest, found pcie0 (and the dts lacking z16's rename), fixed both in the dts, built z19 from z17 (rebuild identical), CI selftest job, build-all gate; programmer review fixed the gate's path.

Considered
: Comparing fragments by hand (fdtoverlay plus a flat diff is the same by-name merge); the phone's own dtbo (needs adb root; a builder never touches the phone).

Rejected
: Allow-listing pcie.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-research-touch-gpio"></a>
### walleye-research: touch gpio defer

Entry date
: 2026-11-18

Done
: Read v7.3 gpiolib, pinctrl and spmi-gpio source against the built z17 dtb: the specifier is valid (tests/host/touch-gpio-specifier.sh passes), the failure is a persistent -EPROBE_DEFER at an unnamed leaf; queued a read-only phone read and a dyndbg diagnostic build.

Considered
: dts hog, pinctrl-0 node, label rename, replacing the regulator, unbind/bind re-probe

Rejected
: all guesses or writes: see decision 2026-11-19-walleye-touch-gpio-diag-first

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-2000-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Android on slot b, stock kernel, nothing flashed or booted; read UFS via debugfs (read only); fastboot reachable per last stand, adb root on

Considered
: next: walleye-try-z19, then walleye-bootchain-readback; UFS HW_VERSION/PHY need a reader (coordinator)

---


<a id="2026-11-19-walleye-builder-gpio-diag"></a>
### walleye-builder: gpio diag kernel, z20

Entry date
: 2026-11-18

Done
: Built z20: z19 with a dynamic-debug and DEBUG_GPIO kernel (EXTRA_CONFIG hook in build-kernel.sh, build/walleye-diag.config), command line plus dyndbg by full kernel paths; test first (fails on z19), checks ok, rebuild identical; the programmer review found the short path pinctrl/core.c matching nothing - fixed, test made exact; queued walleye-try-z20 for the keeper.

Considered
: drivers/base/dd.c +p (every deferral of every device: too loud); module-style short file names

Rejected
: A debugfs leaf read first (the coordinator's fallback, redundant once the diag kernel prints the gpio errno); a new kernel branch for the config (the tree is z15's, only a config fragment differs).

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-2100-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Android on slot b, stock kernel, nothing flashed; z20 tried once and done (touch gpio leaf = shared-gpio proxy, research walleye-z20-tried-touch-gpio-leaf); a and b successful, none unbootable

Considered
: next: walleye-touch-gpio-shared-proxy (build), then walleye-try-z19 and walleye-bootchain-readback; UFS HW_VERSION/PHY still need a reader (coordinator)

---


<a id="2026-11-18-walleye-part-compare-tool-built"></a>
### walleye: part-compare tool built

Entry date
: 2026-11-18

Done
: Built tests/host/part-compare.sh with per-part lists (touch first), selftest on fake captures and transports; programmer review fixed 10 defects before landing; no phone touched.

Considered
: One telnet session per command (kept: simple, same as touch-read.sh); a dt phandle resolver (left as a documented limit).

Rejected
: grep -P and bash-only constructs: host and phone shells are dash/busybox/toybox; Python: forbidden.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-0100-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Android on slot b, stock kernel, nothing flashed; z20 tried a second time for the touch compare (fell back by itself at 180 s; a and b successful, none unbootable); touch-android-compare done (research walleye-touch-android-compare-result)

Considered
: next: walleye-touch-gpio-shared-proxy (build) then add the synaptics node (tlmm 125/89/75, pmi gpio 2); walleye-ufs-android-regs still needs a mem reader (coordinator); walleye-try-z19 and walleye-bootchain-readback remain

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-z21"></a>
### walleye-builder: one claimant for the touch gpio

Entry date
: 2026-11-19

Done
: Found the second claimant of pmi8998 gpio 2 (overlay synaptics_dsx@20 under our stub i2c_5); test first (fails 2 on z19/z20 dtbs); stubs container disabled in generator and dtsi; a UFS-on z21 repack built, then withdrawn (NOT-TO-BOOT) when main's UFS gate arrived; no phone try queued; programmer review applied.

Considered
: Queuing the UFS-on z21; folding the UFS gate into this change

Rejected
: Booting any UFS-on image; the gate is its own item (walleye-ufs-off-gate).

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-ufs-off-gate"></a>
### walleye-builder: UFS off by a gate, then z21

Entry date
: 2026-11-19

Done
: dts: ufshc/ufsphy disabled, variant dts for z11-z13; gate tests/host/ufs-off-image.sh (every appended dtb, alone and merged with dtbo entry 3, plus command line incl. additional args) with selftest and CI job; build/ufs-gate.sh wired into 56 build scripts and build-all; inventory and NOT-TO-BOOT marks on 58 images; z21 rebuilt UFS-off, gate fails z20 and passes z21; programmer review fixes: extra_cmdline, merge failure fails, unit-address match, image removed on failure.

Considered
: Gate on stock-kernel repacks (vendor-gadget, vendor-facts, bisect s0): they fail by design, they need a UFS-off dtb before they build again - not done here.

Rejected
: Matching empty vendor stub nodes named ufs* (they are not devices: first try made 96 images ON); renaming the images directories (scripts refer to them), markers beside the images instead.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-2200-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up, 4381 mV; z21 tried once and done: touch binds (rmi4 0-0020, Synaptics S3708AR input1), no gpiolib_shared deferral

Considered
: next: touch events need a touch (touch-read.sh with a person) and part-compare capture touch ours (z21 again with NOREBOOT=1); walleye-ufs-android-regs still needs a mem reader (coordinator); then walleye-try-z19/bootchain-readback

---


<a id="2026-11-19-2230-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, 4379 mV; z21 tried a second time for the touch part-compare capture, captured and diffed

Considered
: next: read the gpio section of compare-touch-diff-z21.txt (pmi8998 gpio 2 on ours vs Android) ; walleye-ufs-android-regs still needs a mem reader (coordinator); then z19, bootchain-readback

---


<a id="2026-11-19-licences-for-google-walleye-lab"></a>
### Licences for google-walleye-lab

Entry date
: 2026-11-19

Done
: Copied device-template #33's model; annotated per path; SPDX dual licence on our own dts and the stub generator; licence-check fails on main (no REUSE.toml), passes on the branch; programmer review found the C probe relicensed against its own GPL header - fixed.

Considered
: build-pk.sh's generated dts wrap porthole's board files: left GPL-2.0-only.

Rejected
: Writing our own coverage check: orchestration's order is the shared script.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-0235-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Nothing changed. Android on slot b, stock kernel, nothing flashed (as the 22:30 hand-over). Keeper run 02:32 took no item: walleye-try-z21-touch-events needs a person at the phone and the run is unattended; lpoco lab cannot reach the phone: ~/.config/devlab/google-walleye/password is missing (no google-walleye dir at all). walleye-ufs-android-regs still waits on a mem reader.

Considered
: next: coordinator provides the walleye password file (mode 600) and, if the author is at the phone, a fresh go for z21 touch events; then z19, bootchain-readback

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-night-pause"></a>
### walleye lane: paused for the night, touch events still not known

Entry date
: 2026-11-19

Done
: The author paused for the night (relayed by the top coordinator): no new keeper, builder or research session until he or the top coordinator says resume; scheduled runs are disabled; no builder was running. Pixel 2 left on Android slot b, nothing flashed. Touch events on z21: Synaptics S3708AR bound (event1); the author swiped after the 20 s reader had closed; record corrected (research walleye-z21-touch-events-window-missed); go closed; the retry recipe sits in the item walleye-try-z21-touch-events (reader first and 90 s or more, /tmp/hold, ready at once, go of 90 minutes or more).

Considered
: Briefing a keeper directly as a subagent for the retry: held, since the author is away.

Rejected
: Reopening the go tonight: the author is not at the phone.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-display"></a>
### walleye-builder: the display driver compiled from the phone's own panel facts, image built

Entry date
: 2026-11-19

Done
: Took the unlanded display branch of earlier runs; it did not compile (reset and on functions deleted). Read the S6E3FA5 node of the phone's dtbo and restored them with its sequences; built Image.gz and the display dtb, then boot-display.img on z21's ramdisk; every check ok incl. the UFS-off gate. Queued walleye-try-display (witness: person). Not lit: untried.

Rejected
: Lighting the panel with the OnePlus 5 init: not the Pixel 2's own.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-pmic-wdt-v1-images"></a>
### walleye-builder: the PMIC watchdog v1 images, A on next and B on v7.3

Entry date
: 2026-11-19

Done
: Rebuilt the upstreams lane's next tree from the tag next-20261118 plus the six patches (clean), built kernel A there and kernel B on v7.3 + topic 8b2283da3, wrote init-wdt1 (unarmed at boot; wdt-state, wdt-bite, wdt-reboot; SoC petter by identity; banner lines PON and WDT), its host selftest (seen failing on init-z14, passing on init-wdt1) and build/build-pmic-wdt-v1.sh (81 checks, rebuild identical). Found that the clock_gcc symbol must be fixed after the build and that the display image lacks it (backlog walleye-display-image-clock-gcc). Hashes in the try item and in research walleye-pmic-wdt-v1-images-built. A programmer review found seven issues (arming not read back, the self-reboot racing the bite, a racy selftest, a petter check by text only, and more); all fixed, the selftest now catches five mutations of the init, and the images were rebuilt (87 checks ok, rebuild identical).

Considered
: Waiting for the lane's own next tree (not on this machine); arming from the init after a delay; a dts property for the reset type (the driver always sets hard 0x07).

Rejected
: Using /dev/watchdog for the petter (may be the PMIC device, opening arms it). Calling the guard 'the dts type' (there is none).

About
: walleye-pmic-wdt-proof

---


<a id="2026-11-19-display-image-rebuilt-with-the-real-gcc"></a>
### Display image rebuilt with the real gcc

Entry date
: 2026-11-19

Done
: build-display-image.sh now sets /__symbols__/clock_gcc to /soc@0/clock-controller@100000 on the dtb it packs, as build-pmic-wdt-v1.sh does, checked before (the empty stub) and after, and runs overlay-clock-gcc.sh on it; the test was seen failing on the old image's dtb first. New hash b70feabf44a81fc3ad17c9905305b3c8379197d4d824581de2272acec2515cc8 (old 21a1134e retired); walleye-try-display note updated. No phone touched.

Considered
: fixing in build-display.sh (the dtb is built there): the image script already owns the packing and the checks, and the pmic script does it the same way

Rejected
: none

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-layer-declared"></a>
### walleye's device layer declared in device.toml

Entry date
: 2026-11-19

Done
: Added [layer] (name google-walleye, no needs, kernel linux-phone-oco/linux topic/pmic-wdt-wake-in-suspend with build/walleye.config and the walleye dtb, boot android-bootimg, firmware extract-from-partitions) and [boot] (header v0, base 0, kernel 0x8000, ramdisk 0x1000000, tags 0x100, page 4096, 32 MiB partition, msm-id, dtb appended, AVB footer). Checked with lpoco-manifest's own parser (Manifest::parse from lab-tools main 51e5e87, a scratch example): ok; a bad boot method is refused.

Considered
: The cmdline is the plain Android one of the z-line without the diagnostic dyndbg words.

Rejected
: Declaring UCM or udev files: walleye has none, none invented. Naming firmware partition files: the schema holds a method only, so the sources are said in a comment. Declaring [boot.gadget] here: that is walleye-init-from-roof.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-init-gaps"></a>
### walleye-builder: the generated init compared with ours

Entry date
: 2026-11-19

Done
: Declared [boot.gadget] in device.toml; generated the init and files with lpoco-bringup 0.24.0 and compared them with init-z14 and init-wdt1; recorded the gaps (research walleye-generated-init-gaps); sent them to the coordinator for orchestration

Considered
: Switching build/ to the generated init with the missing parts bolted on in walleye's own scripts

Rejected
: Retiring the init variants now: the generated init has no petter, no hold, no raw command loop and no banner, so the phone tries would lose their 180 s self-reboot; item stays open until the roof covers them

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-upstream-dts"></a>
### walleye-builder: the Pixel 2 board dts as an upstream series

Entry date
: 2026-11-19

Done
: Polished the candidate into a two-patch series on v7.3; binding, W=1 dtbs and relaxed style clean, strict 76 to 2, checkpatch 0 errors; programmer review applied in part; nothing mailed.

Considered
: A separate msm8998.dtsi wdt patch

Rejected
: Wdt patch without a bite proof; dual licence

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-upstream-dts-image"></a>
### walleye-builder: image of the exact upstream board dts

Entry date
: 2026-11-19

Done
: Built boot-upstream-dts.img (sha256 a8e4364813f3c82a35b9e5edcffb6650eac710e780119b27fa32dc2fa9521377): z21 with the dtb of the series' dts unedited plus a wrapper with overlay stubs and two deleted nodes; build/build-upstream-dts-image.sh, 31 checks, rebuilt identical, programmer review applied (tree-clean check, FDT_ERR_NOTFOUND check, tool failures stop the build). No phone touched.

Considered
: Editing the series' dts to carry the stubs

Rejected
: Editing the series' dts: the stubs and deletions are the stock bootloader's merge, not upstream's; kept in a wrapper

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-0820-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, 4369 mV; upstream board dts image tried once, booted to the usb0 shell, back on Android

Considered
: next: z19 try, bootchain-readback (read only), c4; read z21 gpio diff; z21 dmesg capture was truncated - recapture if a probe diff is wanted

---


<a id="2026-11-19-walleye-builder-bootchain-readback-scripts"></a>
### walleye-builder: boot-chain read-back reader, check and selftest

Entry date
: 2026-11-19

Done
: Wrote bootchain-readback.sh (read only, refuses without adb device, battery rule, clean DIR outside repositories, then runs its check), bootchain-readback-check.sh (32 names, measured size, sha256, modes, second read equal), bootchain-readback-selftest.sh with a fake adb that logs every call; CI job. Programmer review: five findings fixed (repository below a missing parent, reader trusting a failed su, call log, same DIR twice, echo of phone text).

Considered
: Sizes from the phone's own by-name and /proc/partitions rather than a hard-coded table; xbl 8104 sectors optional because its sector size is not recorded.

Rejected
: Hard-coded sizes: only xbl is measured. Plain adb exec-out dd: the brief names su -c.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-2330-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adbd back to shell; bootchain read back (32 partitions, twice, identical sha256) and item walleye-bootchain-readback done

Considered
: next: z21 touch gpio section of compare-touch-diff-z21.txt; walleye-ufs-android-regs needs a mem reader (coordinator); pmic-wdt items stay parked

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-boot-image-align"></a>
### walleye-builder: the misaligned-image line is the bootloader's

Entry date
: 2026-11-19

Done
: Took walleye-boot-image-align (rank 9). Read the z7 dmesg and build-bootimg.sh: our kernel offset 0x8000 is the stock/taimen/LineageOS value, so the warning comes from where the bootloader loads the Image. Item closed as recorded-as-the-bootloader's; no phone touched, no image built. Left for the next builder: ffs-acm retirement, bisect series, fuel gauge, wifi.

Considered
: Whether a 2 MiB-aligned kernel offset is worth one keeper try.

Rejected
: Moving the offset to 0x200000: changes the load address on an unread bootloader and may overlap the ramdisk at 0x81000000, to silence one cosmetic line.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-retire-ffs-acm"></a>
### walleye-builder: the lab's own FunctionFS helpers retired

Entry date
: 2026-11-19

Done
: Removed the C probe, the Rust helper, their build scripts, the n3/n3r image scripts and init-n3, the rust CI job and the GPL-2.0 REUSE annotation for the probe; README licence line now has no exception. Checks pass.

Considered
: Taking the helper from the lpoco-ffs-acm crate for some image: no current image needs one (mainline uses native configfs ACM/NCM/ECM). Keeping tests/host/ffs-acm-up.sh.

Rejected
: Keeping init-n3 without its scripts: nothing builds from it; git history holds it. Removing ffs-acm-up.sh: it is the proof of a done row.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-series-files"></a>
### walleye-builder: the bisects as lpoco-bringup series files

Entry date
: 2026-11-19

Done
: Six series files in build/series; lpoco-bringup 0.24.0 image bisect reproduced 35 of 35 recorded images byte for byte; removed build-bisect.sh, check-bisect.sh, build-pk456.sh; README names the series. Not run on any phone.

Considered
: Leaving the dtb scripts whole

Rejected
: Deleting the dtb scripts: they make the dtbs

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-fuel-gauge"></a>
### walleye-builder: the fuel gauge image

Entry date
: 2026-11-19

Done
: Fetched the RFC, applied patches 1-3 on v7.3 plus the wdt topic, enabled the node, wrote the check and its selftest, built fg1 (37 checks, rebuild identical), dropped the pmi8998_fg overlay stub, queued the keeper's try (walleye-try-fuel-gauge).

Considered
: All eight patches; a table of the vendor profile.

Rejected
: Both, see the decision.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-20-0030-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adbd up; fg1 tried once: fuel gauge pass (capacity 99 %, 4374 mV = fastboot); item walleye-try-fuel-gauge done

Considered
: next: z21 touch gpio section of compare-touch-diff-z21.txt; walleye-ufs-android-regs needs a mem reader (coordinator); pmic-wdt items stay parked; upstream the fuel gauge result to the RFC thread is the author's

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-unattended-night"></a>
### walleye lane: the unattended night, 2026-11-19

Entry date
: 2026-11-19

Done
: Landed with the lane's builders and keepers (lab PRs 357-392): touch record corrected (the author swiped, window missed; retry recipe in the item); PMIC watchdog v1 images A (next) and B (v7.3 + topic 8b2283da3) built and gated, attended try queued with its plain-words ask, type 0x07 as the series sets; display image rebuilt with the clock_gcc fix (try waits for the author); device layer declared in device.toml; own ffs helpers and bisect scripts retired for the shared tools; the exact upstream board dts booted once to the usb0 shell (research walleye-upstream-dts-tried-boots), finding: the series' dtb needs overlay stubs to merge with the phone's dtbo; boot chain read back (32 partitions twice, identical sha256, files outside git); fuel gauge fg1 proven (capacity 99, 4374 mV = fastboot); boot image alignment is the bootloader's; four generated-init gaps written for orchestration (init adoption waits). The phone ended on Android slot b, nothing flashed. Waits for the author: touch retry, display try, PMIC v1 try (reboot test then bite), UFS (his go).

Considered
: Retrying the touch run at once: the author was gone; the retry has its recipe.

Rejected
: Booting any UFS-on image or the old z19/c4 tries: gate and decisions say no.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-research-ufs-reader"></a>
### walleye-research: how to read UFS_HW_VERSION on live Android

Entry date
: 2026-11-19

Done
: Read the vendor wahoo 4.4 UFS source: register is 0xE4 (earlier rows said 0xE0), version is 3.0.0 or 3.1.0 for certain, no read-only path on Android. Decided: no module; the matched image reads it in its own probe. Closed items walleye-ufs-regs-reader and walleye-ufs-android-regs.

Considered
: A read-only module, debugfs reset_controller, vendor kernel as instrument, skipping

Rejected
: module and reset_controller (cost, bus-hang, write)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-ufs-match"></a>
### walleye-builder: the matched UFS image, built and never booted

Entry date
: 2026-11-19

Done
: Built walleye-ufs-match-android-z: kernel topic/ufs-match-android (quirks, PHY tables by host version, RST_n gpio 153), z21 plus a UFS overlay, image NOT-TO-BOOT-ufs-on-ufs-match.img. The programmer review found 7 points; stop-at-init, the leaked tag, the weak tests, the dirty-Image and queue-name points are fixed, the scope points recorded. The kernel branch is local in ~/Projects/walleye-build/wt-k-ufsm (not pushed). Nothing touched a phone; the try is walleye-try-ufs-match, waiting for the author's go.

Considered
: pinctrl states for the reset; a board-file variant dts instead of an overlay (z21's dtb had drifted from the board file)

Rejected
: a full match of Android's path (link-startup pre-change, ICE not carried)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-display-bisect-images"></a>
### walleye-builder: the display bisect images d0, d1, d2

Entry date
: 2026-11-19

Done
: Three boot images on z21's boot.img: d0 the display kernel with z21's dtb, d1 mdss and the display controller only, d2 plus DSI and its PHY, no panel node; a series file, a parts script and a count script (21+ checks ok; a z21-kernel negative run fails it), UFS-off gate passes on each, sha256 recorded. A programmer review found the count script too trusting; it now reads the display kernel and the stage dtbs itself. Not run on any phone.

Considered
: Whether the stale-looking dtb in images-display is read: build-display-image.sh reads it as raw input and fixes clock_gcc itself

Rejected
: Rebuilding the dtbs from source: the image's own bytes are the proof of the stage

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-1200-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, flags clean. Display try 1 (keeper local_a27ce56b, on the Top Coordinator's go): boot-display.img left fastboot at 11:40:18, Android was back at 11:41:07 (49 s by the host log, 63 s by the keeper), no gadget ever enumerated, no shell, no pattern. The author saw the Google logo, then black, then Lineage within a minute (research walleye-display-try1-author-saw). Cause hunt: research walleye-display-try1-no-shell-what-is-known (clock_gcc fix is in the image; only the display nodes and DRM differ from z21). Display bisect images d0, d1, d2 built (PR 404); the try walleye-try-display-bisect runs next, unattended, read from the console (decision 2026-11-19-walleye-display-console-before-eyes).

Considered
: The keeper's line 'no root, no mainline shell password' is not the gap: the mainline shell is telnet without password and the kernel never reached the gadget, so the keeper's branch keeper/walleye-display-try-1 (which replaces this entry with that wording) is not merged; ramoops needs unlock and root and every mainline try left pstore empty.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-2350-how-the-phone-stands"></a>
### How the phone stands (display bisect)

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up; display bisect: d0 shell ok, d1 loses it (Android in 57 s), d2 not tried; item walleye-try-display-bisect done

Considered
: next: split d1's mdss vs display controller nodes (builder); z21 touch gpio section; walleye-ufs-android-regs needs a mem reader; pmic-wdt items parked

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-walleye-builder-display-late-probe-image"></a>
### walleye-builder: the display late-probe image and its host script

Entry date
: 2026-11-19

Done
: Image boot-display-late.img on z21's header, command line and footer: display path (drm, msm, the S6E3FA5 panel driver and their helpers, 9 modules) as modules with depmod output and an ordered /display-modules.list in the ramdisk, netconsole built in, d1's dtb with the display nodes on (clock_gcc on the real gcc), init-late (z14's init, self-reboot held by /tmp/hold). build-display-late.sh and build-display-late-image.sh (every check stops the build, 36+ checks, UFS-off gate and AVB footer among them). tests/host/display-late-probe.sh loads the list one module at a time, writes each step to the host before the load, streams the log by netconsole and a TCP /dev/kmsg stream, proves each channel by a marker; its selftest runs against a fake phone (19 checks, including a phone that dies at msm.ko). Not run on any phone. Sums in the keeper's folder: boot-display-late.img 673c9582...aff4ed.

Considered
: A new init with the loader inside (the host drives the steps instead); netconsole only (the TCP stream is the second channel); the older display worktree as the kernel tree

Rejected
: The older display worktree: its dts no longer compiles (pmi8998_fg label). /tmp for the work: it is a full 16 GB tmpfs, TMPDIR goes on /home

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-1305-walleye-keeper-late-probe-stands"></a>
### How the phone stands (late probe tried)

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adbd up, battery 4384 mV; display late-probe image tried once: no mainline gadget, Android back at 57 s (research walleye-display-late-probe-try1-no-gadget); item walleye-try-display-late-probe stays open

Considered
: next: late-probe kernel with the display nodes disabled in the dtb to tell dtb from kernel; ramoops read of the death; z21 touch gpio section; UFS needs a mem reader (coordinator)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-display-dtb-bisect-images"></a>
### walleye-builder: the display dtb bisect images f0..f3

Entry date
: 2026-11-19

Done
: build-display-dtb-bisect-dtbs.sh (Image.gz cut from the late-probe image; f0..f3 dtbs by fdtput on z21's dtb; f1..f3.diff), series build/series/display-dtb-bisect.toml, check-display-dtb-bisect.sh, README section. Programmer review: loose value checks became exact added-line comparisons, parts sums verified by the check, scripts 755. Sums in ~/Projects/walleye-build/images-display-dtb-bisect/SHA256SUMS-display-dtb-bisect. Not run on any phone.

Considered
: Taking the groups from the late-probe image's own dtb (lacks z21's touch-power regulator, shifted phandles); cutting from z21's dtb keeps f0 byte for byte z21's

Rejected
: Rebuilding the dtbs from the display dts: phandles shift and f0 would not be z21's bytes

About
: walleye-display-s6e3fa5

---


<a id="2026-11-20-0100-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adbd up (idle-android pass); display dtb bisect tried once each: f0,f1,f2 shell up, f3 no gadget; item walleye-try-display-dtb-bisect done

Considered
: next: walleye-display-f3-node-split (builder), then its tries; z21 touch gpio section; walleye-ufs-android-regs needs a mem reader (coordinator); pmic-wdt items parked; upstream the fuel gauge result is the author's

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-walleye-builder-display-f3-split-images"></a>
### walleye-builder: the f3 split images g1..g3

Entry date
: 2026-11-19

Done
: Series display-f3-split.toml, parts script and check; g1 cmdline +iommu.passthrough=1, g2 f3 without mdss iommus, g3 mdss alone (display controller explicitly disabled); gate and AVB pass, rebuild identical, programmer review fixed two check gaps

Rejected
: leaving the display controller without status in g3: absent means enabled, which would not be mdss alone

About
: walleye-display-s6e3fa5

---


<a id="2026-11-20-0200-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up (boot_completed 1); display f3 split tried once each: g1 and g2 shell up, g3 no gadget; item walleye-try-display-f3-split done

Considered
: next: builder - late-probe image with the g2 dtb or g1 cmdline (then walleye-try-display-late-probe); z21 touch gpio section; walleye-ufs-android-regs needs a mem reader (coordinator); pmic-wdt items parked

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-walleye-builder-display-late-probe-l-images"></a>
### walleye-builder: late-probe images L1 and L2

Entry date
: 2026-11-19

Done
: Built L1 (late image + iommu.passthrough=1) and L2 (late image, mdss iommus removed) with lpoco-bringup image bisect; build-display-late-l-dtb.sh, display-late-l.toml, check-display-late-l.sh; all checks pass, rebuild identical; try item walleye-try-display-late-probe now unblocked.

Considered
: Deriving from f3 (z21's dtb plus display nodes): rejected, f3 lacks the panel node the late probe needs.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-walleye-builder-touch-program"></a>
### walleye-builder: the touch try as one program

Entry date
: 2026-11-19

Done
: tests/attended/touch-try.rs (13 named steps) over tools/walleye-tries; found z21's init has no hold; window follows the restart; reader open and EVENTS-READY before the author is asked; Android awaited before the count; CI job runs the crate's tests; unattended run queued for the keeper.

Considered
: A keeper-side shell recipe; an unattended switch in the program; a hold with a deadman (removed after review).

Rejected
: All three: the one program on the shared runner covers it.

About
: walleye-attend-program-touch

---


<a id="2026-11-19-1415-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up (boot_completed 1, 4385 mV). Display late-probe L1 tried once: shell up, all 13 display modules load with the shell alive, msm probes to the dpu mmap, no drm card, no panel line (research walleye-display-late-probe-l1-modules-load-no-drm-card); item walleye-try-display-late-probe done. My own timeout 200 cut display-late-probe.sh at step 7; steps 7-13 by hand; use 400+ next time.

Considered
: next: walleye-display-why-no-drm-card (read-only reads on L1); then walleye-tas2557-firmware-source and walleye-haptics-i2c-answers (ready, read-only from Android); L2 not needed; builder: z21 touch gpio section; walleye-ufs-android-regs needs a mem reader (coordinator)

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-walleye-builder-pmic-program"></a>
### walleye-builder: the PMIC watchdog try as one program

Entry date
: 2026-11-19

Done
: tests/attended/pmic-try.rs (23 steps) over tools/walleye-tries/src/pmic.rs: gates, reboot test first, bite last, prompts in the author's plain words, A then B fallback; shared phone/host helpers extended (run_partial, image_is_the_built_one); 29 tests with stand-in phone and host programs.

Considered
: Holding the init's restart with /tmp/hold at the S2 check (chosen after review); judging the bite by bootreason (recorded only: format unknown).

Rejected
: Copying touch's plumbing is accepted for now; a shared Lab type is queued (walleye-tries-shared-lab-plumbing).

About
: walleye-attend-program-pmic

---


<a id="2026-11-20-0230-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up (idle-android pass); L1 tried once (nothing flashed): the DRM master waits for c994000.dsi, deferred silently, its node has no vdd-supply; item walleye-display-why-no-drm-card done

Considered
: next: builder - L3 = L1 + vdd-supply on the dsi node (the 0.9 V rail read from the phone), then walleye-try-display-late-probe; touch gpio z21 section; walleye-ufs-android-regs needs a mem reader; pmic-wdt parked; ready for a keeper: walleye-attend-touch-unattended-run, walleye-tas2557-firmware-source, walleye-haptics-i2c-answers

About
: walleye-display-s6e3fa5

---


<a id="2026-11-20-walleye-builder-display-late-probe-l3-image"></a>
### walleye-builder: late-probe image L3 (vdd-supply on the dsi node)

Entry date
: 2026-11-19

Done
: Built L3 = L1 + vdd-supply (l1a) on dsi@c994000 with lpoco-bringup image bisect; build-display-late-l3-dtb.sh, display-late-l3.toml, check-display-late-l3.sh; gate, AVB pass, rebuild identical; programmer review applied. sha256 9360c0579f1350128b8f43573d9900f8c74e08e0dac21e15089a628f20924e08.

Considered
: A rail read from the dtbo: it names none; l1a chosen by analogy (PHY vdds, driver's 0.9 V).

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-drv2624-ported-from-porthole"></a>
### DRV2624 ported from porthole

Entry date
: 2026-11-19

Done
: Took porthole's patch 0018 onto v7.3 as an atomized series: binding (written here), driver (authorship kept), two fix commits after a programmer review (unbind work cancel, range checks, READ_ONCE, GPIOLIB), walleye dts node whose values match the stock overlay; per-chip record; host proof test seen failing and passing; no phone touched.

Considered
: Building a boot image and a try in this item (left to the next item, the part's i2c answer is read first).

Rejected
: Keeping porthole's patch verbatim with all fixes in the same commit: authorship and our changes must be separate commits. Adding a haptics table to device.toml now: the manifest model is lab-tools' (docs #638), not invented here.

About
: walleye-audio-haptics

---


<a id="2026-11-19-2330-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up (idle-android pass). L3 tried once from memory: shell up, 13 display modules load, c994000.dsi still deferred silently, no DRM card. Item walleye-try-display-late-probe-l3 done; lease released.

Considered
: next: builder/research - which msm dsi host get defers (source read, then L4 with dev_err_probe or another vdd rail); keeper next: walleye-attend-pmic-unattended-run, walleye-tas2557-firmware-source, walleye-haptics-i2c-answers

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-touch-program-enters-fastboot-itself"></a>
### touch program enters fastboot itself

Entry date
: 2026-11-19

Done
: Added host::to_fastboot and Touch::phone_is_in_fastboot and the step in touch-try.rs; after programmer review: no serial fails fast, adb state is checked before the reboot, the timeout and no-serial cases have tests.

Considered
: Reusing pmic.rs back_to_fastboot directly (it is private to Pmic; the shared-plumbing item will move both onto host.rs).

Rejected
: Reading the serial with unwrap_or_default: an empty serial reboots whatever adb sees.

About
: walleye-attend-program-touch

---


<a id="2026-11-20-walleye-builder-display-late-probe-l4-image"></a>
### walleye-builder: late-probe image L4 (pinctrl labels on real tlmm groups)

Entry date
: 2026-11-19

Done
: Built L4 from L3 (dtb: four overlay labels repointed to tlmm groups, stubs removed; display dts changed to match); scripts, series, check; test fails on L3 and passes on L4; gate, AVB pass; rebuild identical; programmer review applied. sha256 1dfeebe2c1b6867a0f3fb9f18094e3755e9933cee668dbb177d5af27ed1750d9.

Considered
: Changing gen-overlay-stubs.sh: not needed, it stubs only labels our dtb lacks; the display dts deletes the four generic stubs instead (the main dts keeps them).

Rejected
: A full kernel/dtb rebuild for the image: the L1..L3 line patches the previous image's dtb, so L4 differs from L3 in the dtb only.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-1535-walleye-keeper-pmic-unattended-run"></a>
### PMIC program unattended run

Entry date
: 2026-11-19

Done
: Ran tests/attended/pmic-try.rs --log with stdin /dev/null on the lab machine: checks 1-3 (battery good, both images are the built ones, UFS off in both) pass; the run stopped at ask 4 (TWO_TESTS words) with 'nobody answered' before any boot, arming or bite. Phone untouched; idle-android passes afterwards (Android, adb device).

Considered
: Finding: the program asks the author BEFORE booting image A, so the shell, S2 type read and /tmp/hold steps (the item's 'must pass') do not run unattended; only the gates do. To exercise them unattended the program needs the first ask moved after the boot-A/shell/S2 checks, or a --no-ask-until-arm mode: builder work (tools/walleye-tries). Touch item walleye-try-z21-touch-events skipped: attended, its go expired (04:04 on 2026-11-19), author not present on this scheduled run.

About
: walleye-attend-program-pmic

---


<a id="2026-11-19-1540-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up, idle-android pass. Only the PMIC program's gates ran (no boot).

Considered
: next: builder reorders pmic-try asks (boot A before the first ask) then keeper reruns walleye-attend-pmic-unattended-run; walleye-tas2557-firmware-source, walleye-haptics-i2c-answers; touch needs a fresh go

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-adsp-q6-and-quaternary-mi2s-ported-from-porthole"></a>
### ADSP, q6 and Quaternary MI2S ported from porthole

Entry date
: 2026-11-19

Done
: Took porthole 0023, 0043, 0075, 0081-0083, 0085, 0159 onto v7.3 as eight patches with authorship kept (0006-0008 refitted at fuzz 3) and a ninth, the walleye dts (ADSP on, protection domain deleted on all seven services, LPASS SMMU, MultiMedia dais, Quaternary MI2S RX on SD1, pin groups); kernel/audio-adsp-q6/ with the part's record; host proof test tests/host/audio-adsp-q6-builds.sh seen failing (bad checksum) and passing; programmer review fixed: missing voice-service deletes, lost credits, test gaps, trap. No phone touched, dt/ unchanged.

Considered
: Porting 0084 whole (sound card and WCD934x codec nodes too); changing dt/ now.

Rejected
: The sound card node here: it names the TAS2557 amp and the codec, whose items come first. Porthole's pin labels: they collide with the overlay stubs, renamed. Marking the item done: its proof is a card in /proc/asound/cards on the phone, not reached.

About
: walleye-audio-haptics

---


<a id="2026-11-20-walleye-builder-display-late-probe-l5-image"></a>
### walleye-builder: late-probe image L5 (tlmm 94 reset gpio single claimant)

Entry date
: 2026-11-19

Done
: Found the second claimant of tlmm 94 (the vendor overlay's qcom,platform-reset-gpio on dsi0), wrote tests/host/display-reset-gpio-single-claimant.sh (fails on L4, passes on L5), built L5 by moving the mdss_dsi0 label onto a disabled stub; gate and AVB pass, rebuild identical, programmer review applied. sha256 fb813a9a702e11a2b3d6bd0b981dceecfb21fe0ca7947a03ebe45d62ed6c2ef5.

Considered
: Loading gpio-shared-proxy.ko from the shell: would work around, not remove, the second claimant.

Rejected
: Disabling dsi0: the panel is its child. Editing the overlay: the bootloader merges the phone's own.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-walleye-builder-pmic-first-ask"></a>
### pmic-try: first ask after boot

Entry date
: 2026-11-19

Done
: Moved the TWO_TESTS ask in tests/attended/pmic-try.rs to after boot image A, the shell and the S2 type read; the reboot-test-first, bite-last order and the words are unchanged.

Considered
: Moving the REBOOT ask too: no, it already follows the checks.

Rejected
: Splitting TWO_TESTS wording: it reads fine after boot.

About
: walleye-attend-program-pmic

---


<a id="2026-11-19-walleye-builder-image-l6-stripe-drawer"></a>
### walleye-builder: image L6 (stripe drawer)

Entry date
: 2026-11-19

Done
: Built the stripe drawer and image L6 from L5; programmer review fixed a struct size, logging and release; selftest, gate, rebuild identical.

Considered
: modetest; a libdrm dumb-buffer program

Rejected
: modetest: no aarch64 build; libdrm program: a library to cross-build where fbdev needs only ioctls

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-walleye-drv2624-image-built"></a>
### DRV2624 test image built

Entry date
: 2026-11-19

Done
: build/build-drv2624.sh and walleye-drv2624.config; boot-drv1.img built from z21 with the series applied in a new kernel tree, checks and review done, no phone touched.

Considered
: Driver as a module in the ramdisk.

Rejected
: A module: the initramfs carries no modules, so built in as for every image.

About
: walleye-audio-haptics

---


<a id="2026-11-20-0100-walleye-keeper-pmic-run-two"></a>
### PMIC program run 2: boot A and shell pass, check 6 fails

Entry date
: 2026-11-19

Done
: Ran tests/attended/pmic-try.rs unattended (stdin /dev/null) after the builder moved the first ask. First attempt stopped at step 4: phone idle on Android, not in fastboot; I ran adb reboot bootloader and reran. Gates 1-3, boot image A, shell answers pass; check 6 stopped: wdt-state pmic=none. Read from the shell: pm8916-wdt probed, /dev/watchdog0/1 exist, but the watchdog class devices have no identity attribute (no WATCHDOG_SYSFS), so the identity match fails. Nothing armed, nothing flashed. The image's own 180 s self-reboot returned the phone to Android slot b, boot_completed 1, adb up.

Considered
: next: builder walleye-wdt-state-without-sysfs-identity, then rerun; program could enter fastboot itself from Android

About
: walleye-attend-program-pmic

---


<a id="2026-11-19-walleye-tries-one-lab-type-for-the-shared"></a>
### walleye-tries: one Lab type for the shared plumbing

Entry date
: 2026-11-19

Done
: Moved shell, script, serial, battery, android-back and to-fastboot into walleye_tries::lab::Lab; touch and pmic hold one. host::in_fastboot returns Result so an unrunnable fastboot fails at once; serial_of moved to host; unreadable device.toml named. pmic-try.rs starts with a phone-is-in-fastboot step (no adb reboot bootloader by hand). 32 tests pass, clippy clean.

Considered
: Closure-returning methods on Lab itself (would change both entry files)

Rejected
: Kept thin delegating step methods on Pmic and Touch so entry files stay unchanged

About
: walleye-attend-program-pmic

---


<a id="2026-11-19-wdt-state-and-the-petter-find-watchdogs-by"></a>
### wdt-state and the petter find watchdogs by device link when sysfs has no identity

Entry date
: 2026-11-19

Done
: init-wdt1: the petter and wdt-state take the identity file when present, else the device link (*:watchdog = PMIC pon@800, *.watchdog = qcom_wdt SoC). Host selftest +4 cases without identity files, incl. PMIC-only chooses nothing (all pass); target busybox has CONFIG_READLINK. Programmer review: no blocking findings; open risk: the real readlink names are assumed from the item note, to be confirmed on the phone. Images rebuilt into ~/Projects/walleye-build/images-wdt1-sysfsless: A 9dfea09637a813fa677449780ac475a3b859e81135a056fadd8811dadc2f40b4, B 15f5e97d1ac4185fd4a3a5dae3b874cd5ee0267e694a4b454f9889d7efefe933 (earlier images-wdt1 untouched). Item stays open: its proof is the phone try.

Rejected
: enabling WATCHDOG_SYSFS: a ramdisk change is smaller and keeps the kernel as mailed

About
: walleye-attend-program-pmic

---


<a id="2026-11-19-tries-read-device-toml"></a>
### Tries read device.toml

Entry date
: 2026-11-19

Done
: walleye-try-settings-into-device-toml: settings.rs rewritten over dev()/proc(), entry files on lpoco-attended v0.32.0, device.toml keys added, stand-in tests on Settings::stand_in; 31 tests, clippy, fmt, both --dry-run pass, a missing key fails the dry run naming it. The real run needs build/images linked to ~/Projects/walleye-build in the lab clone.

Considered
: Checking the declared sha256 in code too: the SHA256SUMS check already verifies; left.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-battery-floor-into-device-toml"></a>
### Battery floor into device.toml

Entry date
: 2026-11-19

Done
: Declared [health] battery_floor_mv and battery_source; programs read the floor with dev(); battery-ok.sh reads floor and tool names from the manifest, BATTERY_MIN_MV gone.

Rejected
: Keeping a 3700 default in the script: a missing floor would pass blind; it is skip now. Rewriting battery-ok.sh in Rust: the author said shell stays.

About
: walleye-attend-program-touch

---


<a id="2026-11-20-0300-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up (idle-android pass). Touch program's unattended run done (z21 booted once from memory, EVENTS-READY, no author, phone back on Android). Gap: touch-try.rs lacks an enter-fastboot step (item walleye-touch-program-enter-fastboot). lpoco lab suite latest does not apply: this phone has no login.

Considered
: next: builder - touch-program-enter-fastboot, L3 dsi vdd-supply, z21-hold-init; keeper next: walleye-attend-pmic-unattended-run, walleye-tas2557-firmware-source, walleye-haptics-i2c-answers

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-1600-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: L4 tried once from slot b, nothing flashed: dsi binds, panel defers on reset-gpios (shared-GPIO proxy), no DRM card. Phone on Android slot b, stock kernel, idle-android pass.

Considered
: next: touch try (go open 2026-11-19); an L5 for the tlmm 94 proxy (walleye-display-late-probe-l5-gpio-shared-proxy); then walleye-attend-pmic-unattended-run. Suite not read: no login (no Linux userland on this phone, so no password file is needed).

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-2000-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up. Tried walleye-tas2557-firmware-source: shell user cannot read the vendor firmware (permission denied); stopped, recorded on the item. Suite not read: no login (no Linux userland on this phone, so no password file is needed).

Considered
: next: tas2557 firmware from the factory image's vendor.img on the host (no phone); keeper walleye-attend-pmic-unattended-run; walleye-haptics-i2c-answers

About
: walleye-first-fastboot-boot

---


<a id="2026-11-20-0100-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, idle-android pass. L5 tried once from memory: panel binds, DRM card0 with DSI-1 connected 1080x1920, shell alive, 13 modules load. Item walleye-try-display-late-probe-l5 done; lease released. Suite not read: no login (no Linux userland on this phone, so no password file is needed).

Considered
: next: builder/coordinator - panel enable and a frame (modetest or fbcon) one try, then ask the author to look; keeper next: walleye-modem-mss-proof, walleye-attend-pmic-unattended-run, walleye-tas2557-firmware-source, walleye-haptics-i2c-answers

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-2330-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Re-read the TAS2557 firmware pull on the host: sha256 ffadec7e...b8107b0 and 101481 bytes still match the 2026-11-19 note; no porthole copy exists on this host, so the hash-match proof of walleye-tas2557-firmware-source still has nothing to compare to. Phone not touched this run: lpoco lab phone and suite do not apply (this phone has no login, no password file is needed). Phone as the last hand-over left it: Android on slot b, stock kernel, nothing flashed.

Considered
: next: walleye-try-display-stripes-unattended (rank 2, unattended) and walleye-attend-pmic-unattended-run; the firmware item waits on a porthole hash to compare

About
: walleye-audio-haptics

---


<a id="2026-11-20-0430-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up (idle-android pass), battery 4333 mV; L6 tried once: stripes drawn by fbdev, connector connected/enabled, panel init commands sent without error; item walleye-try-display-stripes-unattended done in part (glass, drawer exit code unread). Suite not read: no login (no Linux userland on this phone, so no password file is needed).

Considered
: next: builder walleye-display-stripes-script-detach, then rerun the L6 try for the exit code and hold; attended look only after; keeper walleye-attend-pmic-unattended-run waits on the builder's ask move

About
: walleye-display-s6e3fa5

---


<a id="2026-11-20-0110-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up (boot_completed 1). PMIC try run 2 booted image A from memory and ended by its self-reboot; no arming. Suite not read: no login (no Linux userland on this phone, so no password file is needed).

Considered
: next: builder walleye-wdt-state-without-sysfs-identity (and walleye-display-stripes-script-detach), then keeper reruns walleye-attend-pmic-unattended-run; L6 stripes exit code still unread

About
: walleye-first-fastboot-boot

---


<a id="2026-11-20-0400-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up, idle-android pass. Touch program unattended run 2 clean (no hand step). Suite not read: no login (no Linux userland on this phone, so no password file is needed).

Considered
: next: keeper walleye-attend-pmic-unattended-run, walleye-tas2557-firmware-source, walleye-haptics-i2c-answers; builder z21-hold-init, L3 dsi vdd-supply; no password file is needed: this phone has no login for lpoco lab suite or phone (AGENTS.md, lab PR 505)

About
: walleye-attend-program-touch

---


<a id="2026-11-19-1540-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up. Touch try on z21 attended run passed (3593 EV_ABS events, Synaptics S3708AR). Suite not read: no login (no Linux userland on this phone, so no password file is needed).

Considered
: next: keeper walleye-attend-pmic-unattended-run (needs builder to move the TWO_TESTS ask), walleye-tas2557-firmware-source and walleye-haptics-i2c-answers (read over adb and the mainline shell, no password file); builder z21-hold-init; password file for lpoco lab

---


<a id="2026-11-19-1610-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up and now left rooted-debugging as the author set it. TAS2557 PG21 firmware pulled read-only to ~/lab-material/walleye/tas2557 (mode 600, never git). Suite not read: no login (no Linux userland on this phone, so no password file is needed).

Considered
: next: walleye-attend-pmic-unattended-run, walleye-haptics-i2c-answers, walleye-tas2557-driver; builder z21-hold-init; no password file is needed: this phone has no login for lpoco lab suite or phone (AGENTS.md, lab PR 505)

About
: walleye-audio-haptics

---


<a id="2026-11-20-0600-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up, 4370 mV. drv1 tried a second time: DRV2624 bound at 0-005a, id 0x03; item walleye-haptics-i2c-answers done. Suite not read: no login (no Linux userland on this phone, so no password file is needed).

Considered
: next: builder walleye-display-stripes-script-detach, then L6 rerun (walleye-try-display-stripes-unattended-2, unattended); the haptics effect try (play an effect, rail unmeasured) wants a builder row; no password file is needed: this phone has no login for lpoco lab suite or phone (AGENTS.md, lab PR 505)

About
: walleye-audio-haptics

---


<a id="2026-11-19-1830-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up, 4383 mV, idle-android pass. Took walleye-try-display-stripes-unattended-2: L6 booted by fastboot from memory, drawer detached ran 60.3 s, exit 0, done. Suite not read: no login (no Linux userland on this phone, so no password file is needed).

Considered
: next: builder walleye-display-late-probe-poll-stripes-rc; then walleye-attend-program-display (attended look); the PMIC unattended run (walleye-attend-pmic-unattended-run) is ready; no password file is needed: this phone has no login for lpoco lab suite or phone (AGENTS.md, lab PR 505)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-2000-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up, battery 4376 mV. drv1 tried once: DRV2624 answers at 0-005a (i2c-0), id 0x03, input registered. Item walleye-haptics-i2c-answers in progress (test itself not run). Suite not read: no login (no Linux userland on this phone, so no password file is needed).

Considered
: next: close the haptics item on the dmesg evidence or one more try that lists 0-005a/driver and the supply; then the next ready row is walleye-modem-mss-proof. The drv1 init restarts at 180 s even with /tmp/hold touched.

About
: walleye-audio-haptics

---


<a id="2026-11-19-late-probe-poll-stripes-rc"></a>
### display-late-probe polls for stripes-rc

Entry date
: 2026-11-19

Done
: display-late-probe.sh waits STRIPES_SECONDS then polls the drawer log every STRIPES_POLL (3) s for up to STRIPES_GRACE (40) real s for stripes-rc; selftest has a late drawer (seen failing before the change) and a never-reporting one (bounded fail). Real L6 run queued for the keeper.

Considered
: a longer fixed sleep

Rejected
: a longer fixed sleep: still a guess, and slow on every pass

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-walleye-keeper-pmic-run-three"></a>
### PMIC program run 3: steps 1-7 pass on the rebuilt pair

Entry date
: 2026-11-19

Done
: Ran tests/attended/pmic-try.rs for real with the rebuilt images (images-wdt1-sysfsless, sha256 checked; a build/images link to ~/Projects/walleye-build made in the worktree, untracked). Checks 1-3 pass; step 4 fastboot, step 5 image A booted with its banner, check 6 shell answers, check 7 pass: watchdog1 probed, S2 type 0x07, firmware type 0x1. Run stopped at ask 8 (nobody answered), as designed. wdt-state in the ramdisk read pmic=watchdog1 soc=watchdog0 s2-enabled=no without sysfs identity: the fix works on the phone. Way back: the ramdisk has no self-reboot and plain reboot did not leave it; reboot -f then sysrq b over the telnet shell took the phone to Android slot b.

Considered
: Image A's ramdisk stays up until told: the program's end after an unanswered ask leaves the phone in the test ramdisk, off adb and fastboot; a way back (reboot -f / sysrq b via the shell) is not in the program: a builder row. lpoco lab suite has no password file (no login on this phone).

About
: walleye-attend-program-pmic

---


<a id="2026-11-20-1200-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up, 4385 mV, boot completed. PMIC program run 3: steps 1-7 pass on the rebuilt images; items walleye-attend-pmic-unattended-run and walleye-wdt-state-without-sysfs-identity done.

Considered
: next: the author's one command tests/attended/pmic-try.rs (reboot and bite tests); a builder row so the program takes the phone out of the test ramdisk when an ask goes unanswered (reboot -f then sysrq b over the shell); then the display stripes poll and L6 rerun; next ready: walleye-modem-mss-proof (20), walleye-slpi-boot-qrtr (21)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-20-1300-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed or written; adb root restarted adbd (Android is throw-away). Read-only pull of modem_a/b, fsg, fsc, modemst1/2 into ~/lab-material/walleye/modem-2026-11-19; modem_b holds mba.mbn and modem.mdt+b00-b27. walleye-modem-mss-proof host step done.

Considered
: next: builder row for a remoteproc_mss test image (dts node, rmtfs, firmware in ramdisk), then one try on slot b; suite latest needs the phone password file at ~/.config/devlab/google-walleye/password (not present), so no suite run this time; next ready: walleye-slpi-boot-qrtr (21)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-display-procedure"></a>
### Display try as one Rust procedure

Entry date
: 2026-11-19

Done
: procedures/display-stripes-on-the-pixel-2.rs on lpoco-attended v0.32.1 with steps in tools/walleye-tries/src/display.rs: gate, fastboot, boot L6, wait_for_gadget (library), shell polled and hold set with an on-phone deadman, modules loaded in list order with rc and liveness each, connector poll, expectation said, detached drawer polled to 'drew', look_for, drawer end rc 0 (60 s grace before the hold is released), Android back. Phone not touched. 48 stand-in tests, clippy, fmt, lpoco-attended check and --dry-run pass; programmer review found 5 defects, fixed.

Considered
: library boot_once/back_to_known_good/read_dmesg/check_probe go through devlab phone, which needs a login this phone lacks: used wait_for_gadget and look_for only

Rejected
: tests/host/display-late-probe.sh (reads the drawer result too early); a fixed sleep for the hold

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-2130-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Real unattended run of procedures/display-stripes-on-the-pixel-2.rs: steps 1-12 passed, stopped at look_for with 'nobody answered'; phone restarted by the image's deadman into Android on slot b (adb device up). Stock kernel, nothing flashed or written. Log: ~/Projects/walleye-build/display-real.log (first try failed at once on a --log path with AGENT_BUILD unset; nothing ran).

Considered
: next: the program's stop at look_for leaves the phone in the test image until the deadman - it should reboot itself back when nobody answers (for the lane coordinator); suite latest needs ~/.config/devlab/google-walleye/password; next ready: walleye-slpi-boot-qrtr (21)

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-pmic-program-the-way-back-as-the-last-act"></a>
### PMIC program: the way back as the last act

Entry date
: 2026-11-19

Done
: tools/walleye-tries watch.rs: a real run of pmic-try.rs is watched by a parent that re-runs the program as a child, then runs Lab::way_back (already Android: nothing; fastboot: fastboot reboot; test shell: reboot -f, then echo b > /proc/sysrq-trigger; else the power-hold sentence, exit non-zero) and exits with the child's code, only if the child acted on the phone (marker file). 61 stand-in tests, clippy, fmt, dry-run lists the step. Programmer review applied: marker, no duplicate in the child, serial error falls through, signal exit codes.

Considered
: library back_to_known_good (systemd userland: does not fit the telnet shell); atexit hook; a library on_stop hook (lab-tools, not mine)

Rejected
: in-process wrapping of steps: the runner calls process::exit on a failed step or unanswered ask and Ctrl-C kills the process, nothing runs after

About
: walleye-attend-program-pmic

---


<a id="2026-11-19-walleye-pmic-program-ab-and-witnesses"></a>
### PMIC program: Image type, A-to-B fallback test, boot reason and slot read

Entry date
: 2026-11-19

Done
: pmic.rs: an Image type (A, B) replaces loose paths; a stand-in test that A not showing a banner leads to B and says so (and keeps B for the second boot; both failing names image B); a new step after the bite pass reads ro.boot.bootreason and ro.boot.slot_suffix from Android and records them (unread when adb gives nothing). 64 tests pass, clippy -D warnings and fmt clean, --dry-run passes (26 steps). Default images already are the rebuilt sysfsless pair via device.toml [images.wdt1-a/b].

Rejected
: a failing step when the slot or reason is unread: the bite's own witnesses stand, so it is said, not judged

About
: walleye-attend-program-pmic

---


<a id="2026-11-19-1855-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed. Ran the display stripes procedure once under the go; stripes drawn, no witness answer came back, so the procedure itself timed out waiting for the relay (research walleye-display-stripes-try2-no-answer); the author did look and answered at once, his words in research walleye-display-stripes-look-result: the glass lights, fbcon text visible, the stripes flashed briefly.

Considered
: next: re-run when the author is confirmed at the glass and a relay is live (ask him to answer straight to the keeper); else walleye-slpi-boot-qrtr (21)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-display-at-boot"></a>
### Display at boot image B1 built

Entry date
: 2026-11-19

Done
: init-display, build-display-at-boot-image.sh, tests/host/display-at-boot-init.sh (behavioural, mutants fail); B1 built, rebuild identical, reviewed

Considered
: building the modules into the kernel (a new kernel build)

Rejected
: built-in DRM: it moves the display probe before the shell, which is what lost the shell in d1; init-loaded modules keep the shell alive

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-walleye-builder-ufs-match-prints"></a>
### walleye-builder: the matched UFS image prints the host and PHY registers, read-only

Entry date
: 2026-11-19

Done
: walleye-ufs-match-prints: HCS/HCE at the version read, PCS+0x130 after PCS ready; image rebuilt (sha256 in the backlog note), identical rebuild, build-ufs-match.sh counts the two new strings (5). Programmer review clean; its one point (the PHY value is post-init, not a leftover) is in the commit message and the note. Never booted.

Considered
: reading the PHY before phy_init (clocks are off there: a fault risk)

Rejected
: none beyond that

About
: walleye-ufs-match-android-z

---


<a id="2026-11-19-walleye-research-ufs-solution-extracted"></a>
### walleye-research: the UFS solution extracted from Android

Entry date
: 2026-11-19

Done
: Closing table of Android's UFS path against the matched image after the gaps (walleye-ufs-solution-table); go note refreshed with the rebuilt image's sha and all gap findings.

Considered
: Naming the image 'sha pending': not needed, the prints build landed (#555).

About
: walleye-ufs-match-android-z

---


<a id="2026-11-19-walleye-builder-the-ufs-try-as-one-rust"></a>
### walleye-builder: the UFS try as one Rust procedure

Entry date
: 2026-11-19

Done
: walleye-attend-program-ufs: procedures/ufs-matched-image-on-the-pixel-2.rs over lpoco-attended v0.32.1, steps in tools/walleye-tries/src/ufs.rs, values in device.toml ([images.ufs-match], [procedures.ufs-matched-image-on-the-pixel-2]). The UFS-on boot step holds its own gate (sha256 and go rechecked right before the boot); the go is read from the item's row and must name the image's sha256; read only, watcher runs the way back. Not yet run for real: queued for the keeper without a go.

Considered
: go read by tad row parsing; a watcher parent for the way back (as the PMIC try)

Rejected
: date -d on go_until (reads 'tomorrow' as a go that never ends: strict date shape only); weakening or skipping tests/host/ufs-off-image.sh (untouched); the go check only as an earlier step (the boot step rechecks)

About
: walleye-ufs-match-android-z

---


<a id="2026-11-19-2130-walleye-keeper-banner-shell-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Took walleye-attend-program-ufs. lpoco-attended run ufs-matched-image-on-the-pixel-2 (no go) stopped at check 1 (battery: no voltage from adb or fastboot), phone untouched. The phone is NOT on Android as the last hand-over said: USB shows 1d6b:0104 'Pixel 2 walleye (mainline banner shell)' serial walleye-first-boot (enp10s0f3u2i2 172.16.43.2), not fastboot, no adb. Nothing flashed or written. Suite cannot run (no ~/.config/devlab/google-walleye/password).

Considered
: next: whoever left the banner-shell image on brings the phone back to Android by its way back, then re-run the UFS no-go run; item stays in-progress.

About
: walleye-attend-program-ufs

---


<a id="2026-11-19-2100-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: walleye-attend-program-ufs proven: real unattended run of the UFS procedure stopped at 'no go' and the phone came back to Android; nothing flashed, slot b, stock kernel, Android running (adb up)

Considered
: next: walleye-ufs-card-fw-rev-read (1, read-only adb root), walleye-display-at-boot try (4), pmic unattended run 4 (3); suite latest needs the phone password file (not present)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-2300-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Took walleye-tas2557-firmware-source (rank 1). Re-checked: pulled blob ~/lab-material/walleye/tas2557/tas2557s_PG21_uCDSP.bin sha256 ffadec7e...06b8107b0 intact. Porthole's tree on the host carries only patches and an APKBUILD that installs the file from a package source dir that is not on the host and pins no hash for it, so the hash-match proof still has no reference. Phone untouched: Android on slot b, stock kernel, nothing flashed.

Considered
: next: item stays open; the proof needs porthole's TAS2557 firmware hash (its docs/BLOBS.md, not on host) or a second pull of tas2557s_uCDSP.bin; the driver try can use the pulled file meanwhile. Next keeper item: walleye-attend-pmic-unattended-run waits on a builder change; then walleye-modem-mss-proof.

About
: walleye-audio-haptics

---


<a id="2026-11-20-1400-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed; B1 tried once unattended from fastboot (13/13 display modules ok, DSI-1 connected, backlight write fails -110, glass not looked at); phone returned by the init's deadman. Tooling note: the run needs build/images linked to ~/Projects/walleye-build and the phone password file is still missing (suite and phone root unavailable).

Considered
: next: author's look at B1 (one command), then the backlight -110 on the dsi cmd dma (research row); next ready: walleye-ufs-card-fw-rev-read (1), walleye-attend-pmic-unattended-run-4 (3)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-2045-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed or changed. Read the UFS card read-only with adb root: SK hynix H28U74301AMR rev D003 (research walleye-ufs-card-revision); risk not known. walleye-ufs-card-fw-rev-read done.

Considered
: next: add the card line to research walleye-ufs-go-note's risk paragraph (coordinator or next keeper); walleye-display-at-boot waits on the author's look (one command); next ready: walleye-attend-pmic-unattended-run-4 (3). Phone password file still missing (suite unavailable).

About
: walleye-first-fastboot-boot
