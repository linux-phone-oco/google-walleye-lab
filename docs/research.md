<!-- Generated from data/ by tad. Do not edit by hand. -->

# Research

<a id="google-walleye-bootloader-first-look"></a>
### google-walleye-bootloader-first-look

Date
: 2026-11-17

Brief
: Pixel 2 in fastboot, read only: unlocked, A/B both slots successful, stock A/B layout, old bootloader with no fastbootd; which Android sits on it is not known yet

Question
: What does the Pixel 2's bootloader say about the phone, and does Android still sit on it, judged from read-only variables only?

Found
: Measured with `fastboot getvar all` (nothing flashed, booted or set): product walleye, hw-revision MP1, boot-mode FASTBOOT, unlocked yes, secure yes, kernel uefi, bootloader mw8998-003.0085.00, baseband g8998-00034-2006052136, slot-count 2, current-slot a, both slots successful=yes unbootable=no retry-count 0, max-download-size 512 MiB, avb 1.0.0 with user-settable key set, battery 4344 mV. Partitions listed with a size: boot_a/b 32 MiB raw, system_a/b 2560 MiB ext4, vendor_a/b 500 MiB ext4, userdata about 53 GiB ext4. is-userspace, slot-suffixes, snapshot-update-status and the sizes of vbmeta, dtbo and modem are not offered: an old bootloader, no fastbootd.
Inference: both slots carry the flags Android's boot control leaves after a good boot, and the partition table is the stock A/B layout, so an Android has run on each slot; userdata of 53 GiB fits the 64 GB model. Which Android, whether it still boots and whether userdata holds data is not known yet - the bootloader cannot say.
Not known yet: whether `fastboot boot` accepts a kernel built by us, and what it does afterwards; every Android reference measurement (sleep current, audio levels, sensors, boot time), backlog google-walleye-lab/android-reference-walleye - it needs the author to boot slot a, enable USB debugging and say when the phone is at the home screen; the paragraph is in the material README.
Next: a first `fastboot boot` try needs a kernel from the research sessions and devlab's fastboot mode (backlog lab-tools/devlab-fastboot-mode). The IMEI the bootloader prints was left out of the kept output.

Material
: research/google-walleye/2026-11-17/


<a id="android-reference-walleye-measured"></a>
### android-reference-walleye-measured

Date
: 2026-11-17

Brief
: Android on the Pixel 2, read only: LineageOS 15 on stock vendor, 37 sensors, 12 s to system ready; sleep current and audio levels not known yet

Question
: What does Android do on the Pixel 2 as the author left it - build, sensors, battery, boot time, audio - measured read-only, with nothing installed or changed?

Found
: Measured with plain adb reads, then back to fastboot with the slot flags unchanged: LineageOS 15 (Android 15, userdebug, built 2025-12-05, patch 2025-11-01) on a stock Android 11 Pixel 2 vendor image, kernel 4.4.302, MSM8998 (8 cores), 3.7 GB RAM, 51 GB data, 1080x1920. Sensorservice lists 37 sensors (LSM6DSM, AK09915, TMx490x, BMP280, AK8789, Elmyra, camera sync). Battery learned 2709 mAh of 2805 mAh design (96.6 %), 922 cycles. Boot, from the phone's own event log: system server at 8.3 s, package manager ready 10.1 s, ams_ready 12.0 s, boot animation done 17.8 s (kernel start to there; the bootloader stage is not counted). Audio config: stereo speaker output at 48 kHz.
Not known yet: sleep current (the phone was charging on USB, Wi-Fi was on, no meter; batterystats' long-run counters contradict themselves so no figure is given), audio levels (no tone source, no microphone), boot time from power-on, thermal limits, the real partition table. Needs next: a USB power meter or the battery path to measure standby with radios as stated, and a tone source plus a lab microphone.
Lesson for the keeper: adb authorization on this phone was lost at reboot until the author tapped Allow again, and the lab's adb server had to be restarted to see it; no reboot should be planned before "Always allow" is confirmed.

Sources
: google-walleye-bootloader-first-look

Material
: research/android-reference/2026-11-17/


<a id="walleye-control-e-f-tries"></a>
### walleye-control-e-f-tries

Date
: 2026-11-17

Brief
: Control (LineageOS walleye boot.img) and our e, f all refused with Load Error on slot b: the cause is the phone's state, not our image

Question
: Does a real stock-format walleye image boot by fastboot boot where ours is refused?

Found
: No. The LineageOS 22.2 boot.img, boot-e and boot-f each got Load Error ('Failed to load/authenticate boot image') within a second of the transfer; the phone stayed in fastboot (18d1:4ee0 in all 90 polls each); slot flags unchanged. Files: research/walleye-first-fastboot-boot/2026-11-17-control-e-f. So our images are not the (only) cause; open: dtbo/vbmeta/slot a flags, all writes or the author's.


<a id="walleye-bootstrap-method-how-it-stands"></a>
### walleye-bootstrap-method-how-it-stands

Date
: 2026-11-17

Brief
: Best-known bootstrap: feedback channel first, vendor kernel as known-good, then mainline by fastboot boot; lk2nd is not for MSM8998/ABL; walleye skipped the vendor-kernel step

Question
: What is the best-known way to bootstrap an Android phone to mainline almost from zero (pmOS, Mobian/Halium, sdm845/sm8150 communities, MSM8998 siblings), and how does the walleye lane's work compare?

Found
: THE METHOD, in order (pmOS Porting/Mainlining guides, Qualcomm mainline porting, Halium early-init, MSM8998 pages): 1 unlock, keep the stock boot.img and a way back to Android; read offsets/ids from it. 2 Get a feedback channel BEFORE mainline: UART first, then simple-framebuffer, ramoops last (pmOS order); for a no-display phone the USB gadget shell is the channel. 3 Vendor kernel first as a known-good: Halium/UBports/Droidian boot the vendor kernel with their own initramfs (telnet 192.168.2.15, break= levels) - it proves image format, offsets, cmdline, gadget, and shows ids, rails, reserved memory. pmOS recommends a downstream port as the base but no MSM8998 sibling did it; they went straight to mainline. 4 Board dts from msm8998-mtp: fix model/compatible/board-id, delete CoreSight nodes, match regulators, clk_ignore_unused (still the normal first flag, Dybcio 2025). 5 Image.gz + dtb appended, mkbootimg base 0 page 4096, initramfs with debug-shell and USB gadget; fastboot boot (OnePlus 5, Mi 6, Pixel 3 XL with a stub dtbo; OnePlus 5T OxygenOS 9 cannot fastboot boot - flash instead). 6 First shell over USB peripheral (QMP + QUSB2 PHY, dr_mode peripheral); Pixel 2 XL reached exactly this, black screen. 7 Then UFS (not on Sony), display, touch, radios; upstream with b4.
LK2ND: not a route. Its device list stops at msm8996, msm8998 is '?' in targets.md, no Pixel, and it is an LK fork loaded by LK-based aboot; Pixel 2 runs a Qualcomm ABL/UEFI-derived bootloader. Pixel 3/sdm845 recipe is plain fastboot boot plus a stub dtbo, not lk2nd. U-Boot as second stage under ABL exists for sdm845 (FOSDEM 2024), not known for msm8998. Whether an LK image could run under ABL is not known yet.
WALLEYE LANE vs THE METHOD: done - stock ids and header read from the public LineageOS image, boot images a-f (17 checks), gadget initramfs proven in QEMU, minimal dts, control test, and the finding that Load Error follows the slot's successful flag (control accepted only from the first state). Gaps: (a) step 3 never done - no image has run a kernel on this phone, so cmdline, ids, rails, ramoops address, and the gadget on real hardware are unseen; (b) step 2 - no feedback channel chosen: UART needs a cable and an oem decision, ramoops address unverified, simplefb unlikely (command-mode DSC panel); (c) the order: the lane went to its own mainline image before any known-good boot. The control (LineageOS boot.img, vendor 4.4 kernel) is accepted from the first state but then left USB with no sign for 6+ min, so even a stock image gives no feedback on this phone yet.
NOT KNOWN YET: whether the vendor kernel in the control reaches a configfs gadget (LineageOS kernels normally do for adb); whether fastboot oem uart enable persists; walleye's dtbo overlay id; first-shell method on taimen (flash vs fastboot boot); sm8150 practice and the FOSDEM 2024 slides were not read. pmOS wiki now at wiki.nura.eco behind a bot wall; read as raw wikitext.

Sources
: pmos-porting-new-device,pmos-mainlining-guide,pmos-qualcomm-mainline-porting,pmos-msm8998,oneplus5-mainline-jamikettunen,pmos-mi6-sagit,lk2nd-devices,pmaports-crosshatch-153,halium-early-init,ramoops-doc,android-debug-cable-howto,dybcio-ignore-unused,kcxt-dtbo-wahoo,xda-fastboot-boot-ab-psa


<a id="walleye-first-fastboot-boot-tries"></a>
### walleye-first-fastboot-boot-tries

Date
: 2026-11-17

Brief
: First fastboot boot tries on the Pixel 2: try a stayed in fastboot, try b left it and Android came back on slot b with slot a flagged unbootable; our kernel's life is not known yet

Question
: What did the Pixel 2 do when given the builder's first boot images with fastboot boot, and what did it leave behind?

Found
: Try a (boot-a.img, taimen offsets): sent, the phone stayed in fastboot, no USB event on the host for 60 s. Try b (boot-b.img, LineageOS offsets): image accepted, "Booting" then the phone left USB at once; 20 s later it was Android's adb gadget (18d1:4ee7), Android on slot b (an older LineageOS build than slot a's), boot reason reboot,ota. Nothing of ours answered: no serial port, no RNDIS/ECM, 172.16.42.1 silent, pstore empty. Whether our kernel ran is not known yet.
The bootloader's A/B logic changed the state, with no write command from us: after try b the phone reports current-slot b and slot a unbootable = yes (first look: slot a current, unbootable no). Slot a's Android is presumably intact; restoring the flags is `set_active a`, a write that needs a recorded decision and the author's word. devlab called try b "refused", though the host log shows the phone left fastboot; it should read slot flags before and after a try and say what changed. Until the author decides about the slot flags no further try should be made.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-17/


<a id="walleye-slot-flip-why"></a>
### walleye-slot-flip-why

Date
: 2026-11-17

Brief
: Why a failed fastboot boot flipped the Pixel 2's slots, how to restore, how to keep tries from flipping them

Question
: Why did two fastboot boot tries change the A/B state, how do we restore it, how do we keep tries from flipping it, and should tries run on slot b?

Found
: Probable, not proven: the Pixel 2's bootloader is Qualcomm-derived ABL (Google's build is not public). In Qualcomm's ABL source (CAF pie mirror, QcomModulePkg FastbootCmds.c, PartitionTableUpdate.c, avb/VerifiedBoot.c), `fastboot boot` runs LoadImageAndAuth on the active slot; when that fails, HandleActiveSlotUnbootable sets slot a's unbootable flag (leaving successful set - the odd pair we saw), makes b active because b is successful, and cold-resets before the reply (our "Status read failed"). So our kernel probably never ran, and panic=10 is not the cause. `fastboot boot` does not decrement retry-count; a kernel crash falls back to a normal boot of the stock boot partition; a slot that is successful=yes is never counted down.
set_active X (ABL source): makes the slot active at priority 3, retry 7, clears unbootable and successful; swaps the a/b partition-type GUIDs and switches the UFS boot LUN: metadata only, undone by set_active back. a stays successful=no until Android on a marks itself good. `--slot` is ignored by `boot`.
Pixel 2 reports: loops end in "Slot Unbootable: Load Error", fixed by set_active (UBports forum 5639); after 7 unsuccessful boots the bootloader switches slot (pmaports MR 3190); the Pixel 2 XL bootloader refuses a kernel unless dtbo holds overlay id 2704 (kcxt dtbo README) - the likely real cause of the failed authentication, unconfirmed for walleye.
Not known yet: the real failure point; why try a stayed in fastboot and whether it changed flags; why b shows retry 0 after the switch; whether Google's ABL matches Qualcomm's.
Ways: leave flags and try on b (no write; if b is flagged too, set_active is the way out); set_active a now (a stays successful=no, the next failed try flips again); set_active before each try (a write per try); flash a stub dtbo to the active slot (a write, no known way back: the stock dtbo cannot be read back by fastboot); image changes (os_version 0 in our header untested).

Sources
: https://github.com/tadiphone-caf/bootable_bootloader_edk2/tree/pie/QcomModulePkg, https://android.googlesource.com/platform/system/core/+/refs/heads/main/fastboot/fastboot.cpp, https://source.android.com/docs/core/ota/ab/ab_implement, https://android.googlesource.com/platform/hardware/qcom/bootctrl/+/refs/heads/main/boot_control.cpp, https://gitlab.com/postmarketOS/pmaports/-/merge_requests/3190, https://forums.ubports.com/topic/5639/no-luck-installing-on-my-pixel-2-walleye/8, https://gitlab.com/kcxt/dtbo-google-wahoo-mainline

About
: walleye-first-fastboot-boot


<a id="walleye-bootimage-header-public"></a>
### walleye-bootimage-header-public

Date
: 2026-11-17

Brief
: Public sources: walleye/taimen boot.img is header v0, base 0, page 4096; no dtb/dtbo fields in v0; os_version is set by the AOSP build, not by porthole's deviceinfo

Question
: What do LineageOS's wahoo build and porthole's mkbootimg inputs say about header version, page size, base and os_version, and does a v0 header fit walleye?

Found
: LineageOS wahoo (lineage-15.1 BoardConfig.mk): BOARD_KERNEL_BASE 0, PAGESIZE 4096, default kernel offset, tags 0x01E00000 and ramdisk 0x02000000 for walleye and muskie (0x80000/0x02500000/0x02700000 only for the kasan builds), no header-version, dtb or recovery_dtbo argument: header v0; BOARD_USES_RECOVERY_AS_BOOT and system-as-root, dtbo in its own partition. Porthole's taimen deviceinfo: header v0, base 0, kernel 0x8000, ramdisk 0x1000000, tags 0x100, pagesize 4096 'verified against the stock taimen boot.img', kernel with the dtb appended, dtbo flashed to its own partition, no os_version set. A v0 header has no dt size or recovery_dtbo field (those arrive with v1/v2), so v0 fits walleye as far as public sources say; walleye's own stock header is still not read. The two offset sets are the two publics': taimen's stock (a) and LineageOS's (b), and try a, with the stock offsets, did not leave fastboot while b did. os_version/patch level: the AOSP build writes them into every boot.img (PLATFORM_VERSION, PLATFORM_SECURITY_PATCH); no public source for walleye gives the value, so variants c and d use 8.1.0 / 2020-01 (a pick). Not known: whether ABL reads them, and whether the dtbo overlay id (backlog walleye-dtbo-overlay-id) or the signature/AVB footer decides instead; porthole flashes a stub dtbo, which we have not.

Sources
: lineageos-kernel-google-wahoo,porthole-pmaports

About
: walleye-bootimage


<a id="walleye-first-fastboot-boot-tries-on-b"></a>
### walleye-first-fastboot-boot-tries-on-b

Date
: 2026-11-17

Brief
: Second tries on slot b: boot-a accepted (Booting OKAY) but not started, boot-b refused (Load Error); slot flags did not change; our kernel still not seen

Question
: What do boot-a.img and boot-b.img do with slot b active, with devlab's before/after slot report?

Found
: With b active (a unbootable, as left): try a - fastboot "Booting OKAY [0.052s]", the phone stays in fastboot, no USB event, nothing answers. Try b - "Booting FAILED (remote: 'Failed to load/authenticate boot image: Load Error')", stays in fastboot. Slot flags before and after were identical both times (current b; a unbootable yes, successful yes; b unbootable no, successful yes; retry-counts 0), so set-active was not needed and was not used; nothing written.
The first try b (17:37, slot a active) behaved differently: the phone left USB and Android came up on b with a flagged unbootable. Same image, so what decided is not known yet (the active slot, or something that changed in between). Our kernel has not been seen to start at all. Next: the builder's variant with os_version and patch level set; a dtbo stub stays the author's decision.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-17-on-b/

Supersedes
: walleye-first-fastboot-boot-tries


<a id="walleye-first-fastboot-boot-tries-c-d"></a>
### walleye-first-fastboot-boot-tries-c-d

Date
: 2026-11-17

Brief
: boot-c and boot-d (os_version and patch level set): both refused with Load Error, phone stays in fastboot, no USB event, flags unchanged; no image has started our kernel

Question
: Does setting os_version and patch level in the header make the bootloader start our boot image?

Found
: Try c (boot-a plus the header os word) and try d (boot-b plus it), on slot b with devlab's before/after report: both fastboot "Booting FAILED (remote: 'Failed to load/authenticate boot image: Load Error')" after the 0.368 s transfer. The phone stayed in fastboot: 18d1:4ee0 in all 90 one-second lsusb polls after each, nothing new on USB (no Linux gadget, no serial port), host kernel log empty. Slot flags identical before and after (current b, a unbootable). No write, no set-active.
So the os_version/patch level word does not make the bootloader accept our images, and it turned boot-a's silent "Booting OKAY" into a Load Error. No image has started our kernel; what the loader dislikes (header, kernel format, dtb, missing dtbo) is not known yet. Open: a dtbo stub is a write and stays the author's decision; the builder may try other header fields or a kernel with the dtb as Android expects.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-17-c-d/


<a id="walleye-stock-ids-recorded"></a>
### walleye-stock-ids-recorded

Date
: 2026-11-17

Brief
: Walleye's stock boot.img (LineageOS 22.2 20261113): header v0, offsets as our image a, one dtb with msm-id 0x124/0x20001 and no board-id/pmic-id; ids are in the dtbo table

Question
: What do walleye's stock boot header and dtb carry (page size, offsets, cmdline, os word, qcom,msm-id/board-id/pmic-id), and is a dtbo present?

Found
: Source: LineageOS official walleye boot.img, build 20261113, https://mirrorbits.lineageos.org/full/walleye/20261113/boot.img, 33554432 bytes, sha256 48dbc55c76ee47840e645707b6bf3ac322099c06358178ad8ddf2c6a53c17468 (equals the site's published value; kept at ~/Projects/walleye-build/control/boot.img, data only, never run on the host). download.lineageos.org publishes no recovery.img for walleye; on A/B the recovery is in boot. unpack_bootimg: header v0; page size 4096; kernel load 0x8000; ramdisk 0x1000000; second size 0 and address 0; tags 0x100 (base 0: taimen/blueline offsets, the same as our image a); os_version 15.0.0, os_patch_level 2026-11 (a nonzero os word; ours are 0, or 8.1.0/2020-01 in c,d); product name empty; cmdline: androidboot.hardware=walleye androidboot.console=ttyMSM0 lpm_levels.sleep_disabled=1 user_debug=31 msm_rtb.filter=0x37 ehci-hcd.park=3 service_locator.enable=1 swiotlb=2048 firmware_class.path=/vendor/firmware loop.max_part=7 raid=noautodetect usbcore.autosuspend=7 androidboot.dtbo_idx=3. Kernel: lz4 (sha256 0256983d1002752214fde5035f78ce07e088d167bad993f3f16e1e9f78162872), Linux 4.4.302 arm64, with ONE dtb appended at byte 17613814 (376259 bytes, sha256 37dd8612cab84bd96025bd5831eac194fcf1f10bc8f8f08ea6ff72717a8f5751): model 'Qualcomm Technologies, Inc. MSM8998 v2.1', compatible qcom,msm8998-mtp / qcom,msm8998 / qcom,mtp, qcom,msm-id <0x124 0x20001>; NO qcom,board-id and NO qcom,pmic-id at the root. Ramdisk is xz (sha256 a0d50bc3063f3fd60e93c067ca9b54d16e670b6171eb498decc294d658f5dcbf). So the stock dtb is matched on msm-id alone, as ours is. The board identity is in the dtbo: dtbo.img of the same build (8388608 bytes, sha256 fb09009b7946bf33d860ee94cadfabdfcf3cf63a872314bc650d9220486bb8f6, equals the site's) holds a DT table of 13 entries, ids 0x1bc (revs 0-4 and 0x80), 0x190, 0x490, 0x590, 0x690 and more; androidboot.dtbo_idx=3 names the one merged; the overlays carry no qcom,board-id. So variant f's premise (stock board-id/pmic-id to copy) does not hold: f adds qcom,board-id <8 0> only (what qcom,mtp means), a guess, no pmic-id.

Sources
: aosp-dto-multiple,inoki-abl-dt-selection

About
: walleye-loaderror-stock-ids


<a id="walleye-control-on-slot-a-try"></a>
### walleye-control-on-slot-a-try

Date
: 2026-11-17

Brief
: Control boot.img on slot a: Load Error again; the slot state is not the cause. Phone back on b, fastboot reachable

Question
: Does the LineageOS control boot.img boot by fastboot boot with slot a active (the state of the very first tries)?

Found
: No. Per decision walleye-slot-set-active-a-once: devlab fastboot set-active now a (before: current b, a unbootable yes/successful yes/retry 0, b unbootable no/successful yes/retry 0; after: current a, a unbootable no/successful no/retry 1). devlab fastboot boot now control/boot.img (sha256 48dbc55c..., LineageOS 22.2 20261113): 'Sending boot.img OKAY', then 'Booting FAILED (remote: Failed to load/authenticate boot image: Load Error)'; still in fastboot after a minute; the try changed no flags. So the refusal is not caused by slot b being active or by a being flagged unbootable: the same stock image is refused on a too. Open causes: dtbo/vbmeta state (a write, the author's or coordinator's call), or fastboot boot on this bootloader never takes an image of this kind. Restored with set-active now b: current b, a unbootable no/successful no/retry 1, b unbootable no/successful no/retry 1 (set-active resets b's successful flag; both slots bootable, retry counts 1); fastboot reachable, nothing written.

About
: walleye-first-fastboot-boot


<a id="walleye-boot-refused-why-how-it-stands"></a>
### walleye-boot-refused-why-how-it-stands

Date
: 2026-11-17

Brief
: Load Error is not the slot flags (control refused on a too); the one accepted try came with healthy Android on a, so the next test restores that state, then cold restart, host checks, UART

Question
: Why does the Pixel 2's bootloader refuse even LineageOS's official boot.img by fastboot boot (Load Error), and what makes fastboot boot work on it?

Found
: What is: the control (LineageOS 22.2 20261113 boot.img, stock header, AVB footer, own dtb) is refused with "Failed to load/authenticate boot image: Load Error" on slot b and, per walleye-control-on-slot-a-try, on slot a too; so are our b-f. Seen to be accepted: our boot-b on the very first try (slot a active, phone entered fastboot from healthy Android): the phone left USB, Android came up on b, a flagged unbootable; and boot-a, "Booting OKAY" with nothing starting, on slot b. The same image was refused later. So the refusal is not the image's format and, after the slot-a try, not the active slot or the unbootable flag alone. The same string and A/B flip are reported for walleye since 2017 (XDA PSA); on other Qualcomm ABL phones it comes "randomly" and goes with a restart, another port or cable, set_active or a stock reflash.

Ranked causes now (each by what it predicts):
1. State left by the first tries that set_active does not undo. The one accepted try came from a phone booted into Android on a, with a successful=yes (Android marks it when boot completes). Now a is successful=no, retry 1, and b's flag was reset by our set-active (b successful no). Predicts: after Android boots on a to completion (marks it successful) and the phone is put into the bootloader from that Android, the control is accepted again. Cheapest and reversible; it is the exact first state. Not tested.
2. A bootloader that has stayed up since a failed boot (the 'restart cures it' reports). Predicts: a cold power-off and start into the bootloader, then the same try, is accepted. Not tested; also covered by 1, which reboots the phone.
3. dtbo (or vbmeta) of the active slot. The bootloader picks overlays from dtbo by id and rev and refuses if none fits (AOSP DTO; the taimen bootloader wants id 2704, kcxt). Slot b's is an older Lineage build's and a's is current, yet both refuse, so a mismatch with the new dtb is unlikely; a cause only if 1 and 2 fail. A write; stays named, with no way back for the stock dtbo (fastboot cannot read it back).
4. Host: old fastboot, USB port, cable. The transfer succeeds and the refusal comes back at "booting", so least likely; recorded with each try (fastboot --version).
5. AVB and lock state: the phone reports unlocked yes, where verification errors are warnings; not measured.

Not known yet: the bootloader's own reason. Google's ABL is not public (the CAF Qualcomm ABL only shows the shape) and fastboot has no read-only oem command that prints it. The way to see it is the serial log over the Pixel 2's Android Debug Cable (Nura wiki), which needs the author's hardware.

Vector chosen: restore the first state, since it is the only state in which an image was accepted: set-active a, boot Android on a normally to completion (a's own Android, nothing written), adb reboot bootloader, the control once; then, if still refused, cold power-off start and host checks; then UART; a write last.

Sources
: xda-fastboot-boot-ab-psa,xda-load-error-restart,kcxt-dtbo-wahoo,nura-taimen-wiki,aosp-dto-compile

About
: walleye-first-fastboot-boot


<a id="walleye-first-state-control-accepted"></a>
### walleye-first-state-control-accepted

Date
: 2026-11-17

Brief
: First state restored: control ACCEPTED by fastboot boot after Android completed on a; phone then left USB entirely (no adb/fastboot) - needs a hand power-cycle

Question
: Does restoring the first state (Android completed on slot a, adb reboot bootloader) make the bootloader accept the control boot.img?

Found
: Yes. set-active a, fastboot reboot, Android up on a (sys.boot_completed 1, slot _a). After adb reboot bootloader: current-slot a, a unbootable no/successful yes/retry 0, b unbootable no/successful no/retry 1. devlab fastboot boot now control/boot.img: left fastboot after 12 s. devlab's 'came back as Linux, USB network answering' is a false match: the 18d1:d001 gadget (Qualcomm SM7150, serial postmarketOS) on the bus is the surya phone; walleye FA81N1A00790 was on no USB, no adb, no fastboot for 6+ min. So cause 1 (state left by the first tries) is confirmed: Load Error follows the slot's successful flag, not the image. Nothing written. The lab has no power control for walleye, so it is not back in fastboot: a hand hold of power 10-30 s is needed (slot a is successful, so it boots Android on a; then adb reboot bootloader). Next: boot-b/e from the same first state; fix devlab's came-back check to match the manifest serial.

About
: walleye-first-fastboot-boot


<a id="walleye-own-images-e-b-first-state"></a>
### walleye-own-images-e-b-first-state

Date
: 2026-11-17

Brief
: e and b accepted from the first state, neither started: no USB device, flags unchanged, phone stays in fastboot

Question
: Does fastboot boot of our images e and b start from the restored first state?

Found
: Both ACCEPTED (Booting OKAY) but not started: 90 s watch by serial, phone stays in fastboot, no new USB device, no 172.16.42.1 (172.16.42.2 is surya's), slot flags unchanged; acceptance is not the cause of silence

About
: walleye-first-fastboot-boot


<a id="walleye-try-s1-s2-s3"></a>
### walleye-try-s1-s2-s3

Date
: 2026-11-17

Brief
: s1 (our gadget ramdisk), s2 (our kernel), s3 (our dtb) tried from the first state: none started; s1 and s2 fell back to stock Android on a, s3 accepted but stayed in fastboot

Question
: Which stock part, swapped for ours, makes walleye's bootloader stop starting the image?

Found
: From the first state (current-slot a, a successful, b unbootable), no writes. s1: left fastboot after 12 s, Android on slot _a kernel 4.4.302 after 75 s, no 172.16.43.1, no new USB device (only the stock adb): our gadget did NOT come up; stock vendor kernel/dtb/cmdline with our ramdisk did not start (a stock boot of slot a is what ran). s2: same shape (12 s, Android on a, 4.4.302 after 72 s). s3: 'Booting OKAY' then stayed in fastboot, no USB event. Flags unchanged each time (a successful, b unbootable). Together with s0n (no start) and s0 (stock with AVB footer, accepted earlier): every image that differs from the stock footer-carrying boot.img, even ramdisk only, fails to start; s0n shows the footer or padding alone may be enough. Next: rebuild s1 with the AVB footer (algorithm NONE, padded to 32 MiB like s0) and try it.

About
: walleye-first-fastboot-boot


<a id="walleye-try-s1-gadget-up"></a>
### walleye-try-s1-gadget-up

Date
: 2026-11-17

Brief
: s1 = vendor-gadget image (footer, 32 MiB) refused with Load Error on slot a twice; gadget-up fails; earlier s1/s2 'left fastboot' were fallbacks to stock a

Question
: Does the stock kernel with our gadget initramfs (s1, AVB footer present, size 33554432 checked) start from the first state, and does 172.16.43.1 answer?

Found
: Checked: boot-s0/s1/s2/s3 are 33554432 bytes and end in AVBf; s0n is 26472448 without. Then, current-slot a (a successful, b unbootable yes retry 1): devlab fastboot boot now boot-s1.img -> 'Failed to load/authenticate boot image: Load Error', flags unchanged; again with images/vendor-gadget.img (same bytes) while GADGET_WAIT=90 gadget-up.sh (from the builder's worktree wt-walleye-builder-lab, not yet landed in the lab) ran: same Load Error, gadget-up: fail, no USB device with our strings, 2 google devices (the phone in fastboot and surya?). The earlier s1/s2 runs this session (left fastboot after 12 s, Android on a 4.4.302) were therefore fallbacks to stock, not starts; my earlier suggestion to rebuild s1 with the footer was wrong - it has it. Earlier research showed Load Error followed the slot state; b is unbootable now and tries have been run with a active this time, so the open question is whether the state (b unbootable, active a) or the image causes it. No write.

About
: walleye-first-fastboot-boot


<a id="walleye-vendor-kernel-facts-s1-panic"></a>
### walleye-vendor-kernel-facts-s1-panic

Date
: 2026-11-17

Brief
: s1 boots the stock kernel (bootreason kernel_panic, back on a after 75 s); the vendor kernel has no ACM/ECM/RNDIS, only NCM and FunctionFS, no devtmpfs - our gadget init cannot show up on it

Question
: What did the s1 try do, and what does the running vendor kernel offer our gadget init?

Found
: Bootloader first: after the earlier Load Errors both slots ended unbootable (a: unbootable yes, retry 0) and the phone stayed in fastboot after a reboot; one metadata-only 'devlab fastboot set-active now a' cleared a (unbootable no, successful no, retry 1) and Android completed on a (successful yes again). s1 re-run from that healthy first state: left fastboot after 12 s, Android on a after 75 s, ro.boot.bootreason=kernel_panic - so the stock kernel DID start with our ramdisk and panicked, then the bootloader fell back to a. (Load Error appeared only when run from the state after several refusals.) Facts, read-only from the vendor kernel 4.4.302 (adb root, debuggable): UDC a800000.dwc3. Config: USB_CONFIGFS=y, F_FS, NCM, MASS_STORAGE, MTP, PTP, ACC, AUDIO_SRC, MIDI, HID, DIAG, GSI, CDEV, QDSS = y; ACM, SERIAL, OBEX, ECM, RNDIS, EEM are NOT set; DEVTMPFS not set; PSTORE, PSTORE_CONSOLE, PSTORE_PMSG, PSTORE_RAM, ACCESS_RAMOOPS = y. Our gadget init (ACM + RNDIS/ECM) therefore could not bring up a gadget on this kernel even if init ran; it needs NCM (host sees a cdc_ncm interface) and, with no devtmpfs, mounting devtmpfs fails - our init must mknod or use mdev. Ramoops: ramoops_region reg 0xa1100000 size 0x200000; alt_ramoops_region 0xa3800000 size 0x200000; meta region 0xaffff000 size 0x1000; params record 524288, console 262144? (from /sys/module/ramoops: mem_size 2097152, record 524288, console 131072 order as listed). The pstore is encrypted by vendor ramoops (init.ramoops.sh: ramoops -D after the user unlocks, files appear in /data/vendor_ce/0/ramoops, absent here), so the s1 panic text is NOT readable from Android; /sys/fs/pstore empty. Cmdline of the vendor boot: androidboot.hardware=walleye console=ttyMSM0 ... root=PARTUUID, androidboot.dtbo_idx=3, panel mdss_dsi_samsung_s6e3fa5_1080p_cmd, serial redacted. Next: rebuild s1's init for NCM (+ mknod, no devtmpfs), keep the panic hunt by making init print to /dev/kmsg-less console only if needed; or make our init never exit (sleep loop) to see if the panic was init exiting.

About
: walleye-first-fastboot-boot


<a id="walleye-try-s0-control-healthy-state"></a>
### walleye-try-s0-control-healthy-state

Date
: 2026-11-17

Brief
: s0 control from the healthy state (a active+successful, b unbootable): accepted, left fastboot after 12 s, phone then gone from USB - needs a hand power hold

Question
: Is the footer-carrying stock control s0 accepted from the healthy state, and what does it do?

Found
: Same state as the s1 panic try. s0: left fastboot after 12 s and then no adb, no fastboot, nothing on its USB port for 3+ min (as the earlier control-accepted try). So s0 differs from s1: the stock image runs (no fallback to Android a, no panic reboot visible) but the phone then sits off USB - a hang or the vendor kernel with no adb (control cmdline and ramdisk are stock, so Android should have come up: more likely it shut off or hung). Needs hands: hold Power 10-30 s; slot a is successful so it then boots Android on a. Nothing written. Contrast: s1 (our ramdisk) panics and falls back in 75 s, s0 does not fall back.

About
: walleye-first-fastboot-boot


<a id="walleye-vendor-gadget-v1-gzip-cause"></a>
### walleye-vendor-gadget-v1-gzip-cause

Date
: 2026-11-17

Brief
: v1 vendor-gadget panicked because the stock kernel has no RD_GZIP: only an xz ramdisk unpacks

Question
: Why did s1 / vendor-gadget.img panic the stock kernel?

Found
: IKCFG data in the control's Image.raw (offset 19381680): CONFIG_RD_GZIP not set, RD_XZ=y, DEVTMPFS not set, UNIX98_PTYS y, USB_CONFIGFS_NCM y. Our v1 ramdisk was gzip, so the kernel could not unpack it, found no /init and panicked (kernel_panic bootreason). The stock ramdisk is xz CRC32. Also fixed in v2, as it would have failed next: a tmpfs mounted over /dev hid /dev/console (exec to it ends init), ACM/ECM/RNDIS absent (NCM used), init exiting. Not run on arm64 (no qemu-aarch64 installed): checked by unpack, file, sh -n, applets present.

About
: walleye-first-fastboot-boot

Supersedes
: walleye-vendor-kernel-facts-s1-panic


<a id="walleye-try-s0n-no-footer"></a>
### walleye-try-s0n-no-footer

Date
: 2026-11-17

Brief
: s0n (stock parts, no AVB footer) from the healthy state: accepted, left fastboot after 12 s, phone then off USB - same as s0, so the footer is not what matters; needs a hand power hold

Question
: Does the stock control without its AVB footer (s0n, the shape of our own images) behave as s0?

Found
: From the healthy state (a active+successful, b unbootable, retry 1) devlab fastboot boot now boot-s0n.img (sha 46b575c7...) was accepted; left fastboot after 12 s; then no adb, no fastboot, nothing on its USB port for 3+ min - exactly as s0 (walleye-try-s0-control-healthy-state). Footer and 32 MiB padding therefore do not decide whether the stock kernel starts and goes quiet on USB; the shape of our images (a-i) is not what Load Error rested on. Remaining differences to ours are the parts (s1 ramdisk panics, s2 kernel and s3 dtb refused or fall back). Phone needs hands: hold Power 10-30 s; slot a is successful so it then boots Android on a. Nothing written; slot flags not readable until it is back in fastboot.

About
: walleye-first-fastboot-boot


<a id="walleye-gadget3-third-try"></a>
### walleye-gadget3-third-try

Date
: 2026-11-17

Brief
: vendor-gadget3, third try with the watcher alive 130 s: no USB device with our strings ever; kernel_panic, Android on a after 74 s

Question
: Does vendor-gadget3 enumerate our gadget (serial walleye-first-boot) when the watcher is kept alive for the whole window?

Found
: From the first state (a active+successful, b unbootable), fastboot boot of vendor-gadget3.img: left fastboot after 12 s, Android on a after 74 s (Pixel 2 strings at t=68 s). A 1 s watcher over 130 s (its own setsid job) listed every USB device each second: port 3-2 showed only fastboot, then Android; no device with serial walleye-first-boot or product 'Pixel 2 walleye (mainline)'. The 18d1:d001 devices seen all sit on port 3-1 with serial postmarketOS = surya, not ours. ro.boot.bootreason=kernel_panic; no pstore readable without root. So the panic comes before any gadget strings, and the 240 s timer never runs. Next: an init that only writes to /dev/kmsg, to separate ramdisk unpack from gadget bring-up.

About
: walleye-first-fastboot-boot


<a id="walleye-panic-60s-how-it-stands"></a>
### walleye-panic-60s-how-it-stands

Date
: 2026-11-17

Brief
: The 60 s panic: check the system-as-root cmdline (free, read-only) first, then the panic time, then ramoops after the author's unlock; UART is the sure route

Question
: Why does the vendor 4.4.302 kernel panic about 60 s after fastboot boot with our ramdisk, and how can the panic text be read?

Found
: WHAT IS (lab base): the vendor 4.4.302 kernel started with our swapped ramdisk (xz, so not the gzip fault) and panics before any gadget: bootreason kernel_panic, USB gone at ~12 s, Android back on a at 68-74 s; the true panic time is NOT measured (the host sees USB only, and the fallback Android boot itself takes tens of seconds, so the panic may be at ~30-60 s, not 60). Stock-kernel pstore is empty from Android (vendor ramoops -D encrypts after unlock). CAUSES, ranked by what the facts support, each with the one cheap try that tells it apart. (1) Our ramdisk is not what runs: Pixel 2 is system-as-root (A/B, recovery in boot; LineageOS flashes recovery into boot), and a system-as-root cmdline is ro root=/dev/dm-0 rootwait skip_initramfs init=/init - the kernel then ignores the boot ramdisk and runs /init from the system partition (src-android-system-as-root); a bootloader can add skip_initramfs itself (src-magisk-skip-initramfs). Then our /init never ran, which fits no gadget ever, and the later panic is Android first-stage failing on a half-set-up boot. Told apart by two READ-ONLY facts: the full /proc/cmdline and /proc/device-tree/chosen/bootargs of the running Android (skip_initramfs, root=, dm=), and the control image header cmdline (host, from the file). Needs no try. If skip_initramfs is there, gadget4 (idle init) is moot for the panic and the way is an image whose cmdline/ABL path honours our ramdisk, or the system partition route; this goes first because it is free. (2) A kernel-side failure independent of userspace: a peripheral (modem/adsp/slpi via PIL) crashing or timing out with restart level SYSTEM prints subsys-restart ... Resetting the SoC and panics (src-qcom-log-analysis); the user-mode firmware fallback waits 60 s by default (src-kernel-firmware-fallback), so a missing userspace ueventd can turn into a panic about a minute in. Told apart by gadget4 (idle init that survives): the same gap with an idle init means kernel-side; the cheaper first step is to measure the panic time (see below). (3) Our init exiting or crashing: ruled out in time by gadget3 (240 s timer, no early exit seen), and gadget4 tests it again. (4) MSM apps watchdog: pet by the kernel, not userspace, so unlikely; reads as watchdog bark/bite in the text. HOW TO READ THE PANIC TEXT, cheapest first: (a) Measure the time instead of reading the text: a 1 s USB watcher over a plain adb reboot gives T_android (USB loss to Android on USB); the panic try gives the gap; panic time = gap - T_android. This needs no write. 60 s plus ~12 s would point at the firmware fallback; ~25-45 s at an early kernel fault. (b) Previous boot text in ramoops: region 0xa1100000 2 MiB is reserved, so the panic text survives the warm reset (src-kernel-ramoops-doc) but the vendor init consumes and encrypts it into /data/vendor_ce/0/ramoops once the user unlocks; so after a panic try, boot Android on a, have the author unlock the phone once (credential is the author, not ours), and read the ramoops files as root - UNKNOWN whether the file appears for a non-user-initiated reboot: one try shows it. The phone must not be rebooted again before reading, or the next boot overwrites the buffer. (c) Our own ramdisk as reader: only if (1) is false - a second try image whose init mounts pstore and only prints to a path we can read; no channel exists, so not first. (d) UART: the Android Debug Cable (USB-C passthrough breakout + FT232 TTL, a purchase of a few euros for the author, src-pixel-uart-cable) works on walleye (src-nura-serial-pixel2); fastboot oem uart enable needs the unlocked bootloader (it is) and is a bootloader setting that persists until fastboot oem uart disable - a write with a way back; the 1.8 V line needs the board to be right. It gives the bootloader log AND the kernel console to the end of the panic, so it answers causes 1-4 at once and also the Load Error question; it is the sure route, kept second only because it costs a purchase and a hand. (e) /proc/last_kmsg: not in 4.4 Android (moved to console-ramoops, Android 6+). (f) Not now: kernel rebuild with the earlycon/pstore console to the cmdline, a dtbo write.

Sources
: src-kernel-ramoops-doc,src-kernel-firmware-fallback,src-android-system-as-root,src-magisk-skip-initramfs,src-qcom-log-analysis,src-pixel-uart-cable,src-nura-serial-pixel2,src-android-wahoo-kernel

About
: walleye-first-fastboot-boot

Supersedes
: walleye-vendor-kernel-facts-s1-panic


<a id="2026-11-17-walleye-gadget4-idle-init-returns"></a>
### 2026-11-17-walleye-gadget4-idle-init-returns

Date
: 2026-11-17

Brief
: vendor-gadget4 (idle init, no gadget): back on Android a at 156 s, bootreason reboot, not kernel_panic - the panic is in the gadget bring-up

Question
: Does the panic of gadget2/3 happen without our gadget code (idle init only)?

Found
: From the first state, fastboot boot of vendor-gadget4.img: left fastboot after 15 s, back on Android a (adb) after 156 s; ro.boot.bootreason=reboot (gadget3 gave kernel_panic at ~75 s), sys.boot_completed=1, slot a active/successful, b unbootable retry 1 as before (flags unchanged). So the vendor kernel boots and our init runs to its own reboot at ~120 s; the gadget bring-up of gadget2/3 is what panics. Our init's /dev/kmsg marker is not readable from Android.

About
: walleye-first-fastboot-boot


<a id="2026-11-17-walleye-vendor-gadget5-bisect"></a>
### 2026-11-17-walleye-vendor-gadget5-bisect

Date
: 2026-11-17

Brief
: vendor-gadget5a returns at 156 s (reboot); 5b panics at 75 s (kernel_panic): the ncm function or its config symlink panics, configfs and the gadget dir do not

Question
: Which step of the gadget bring-up panics the vendor kernel: configfs+gadget dir (5a), +ncm function and config (5b), +UDC bind/network (5c)?

Found
: From the first state (Android on a successful, b unbootable retry 1), fastboot boot of each image, nothing flashed. 5a: left fastboot 12 s, Android on a at 156 s, bootreason reboot, boot_completed 1 (init ran to its own reboot at ~120 s). 5b: left fastboot 12 s, Android on a at 75 s, bootreason kernel_panic. Stopped at the first panic, so 5c not tried. Culprit: the step 5b adds over 5a: creating the ncm function and/or the config with its symlink (f_ncm in the vendor 4.4 kernel, or configfs usb_gadget function handling). UDC bind is not the cause. Slot flags before and after each try identical: a current/successful, b unbootable retry 1; phone left in fastboot. gadget-up.sh not run: it is only for a 5c that does not panic. Next: split 5b (function dir only; ncm without config; other function type such as rndis/acm) - builder.

About
: walleye-first-fastboot-boot


<a id="2026-11-17-walleye-ncm-split-b2-panics"></a>
### 2026-11-17-walleye-ncm-split-b2-panics

Date
: 2026-11-17

Brief
: ncm split: b1 (mkdir ncm.usb0 only) returns at 156 s, reboot; b2 (+ dev_addr, host_addr writes) panics at 75 s: the address writes panic

Question
: Which step of the ncm function bring-up panics the vendor msm-4.4.302 kernel (b1..b4 from the first state, stop at the first panic)?

Found
: From the first state (Android booted on a, slot a active+successful, b unbootable retry 1) by devlab fastboot boot now: vendor-gadget5b1 left fastboot after 12 s, Android back at 156 s, ro.boot.bootreason=reboot (no panic). vendor-gadget5b2 left fastboot after 12 s, back at 75 s, bootreason kernel_panic. b2 differs from b1 only by two writes: dev_addr (02:42:ac:10:2b:01) then host_addr (02:42:ac:10:2b:02) into functions/ncm.usb0. So creating the ncm function does not panic; writing dev_addr or host_addr does (or its kmsg 'about to' marker was the last line - not read, no ramoops). b3, b4 not tried: the stop rule holds. Slot flags unchanged after (a active successful, b unbootable retry 1); phone left in fastboot on a, nothing flashed.

About
: walleye-first-fastboot-boot
