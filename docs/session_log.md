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
