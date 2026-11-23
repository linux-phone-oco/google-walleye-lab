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


<a id="walleye-ncm-panic-why-how-it-stands"></a>
### walleye-ncm-panic-why-how-it-stands

Date
: 2026-11-17

Brief
: ncm panic: our init writes dev_addr/host_addr on the ncm function while opts->net is still NULL (net is made at UDC bind); mkdir and symlink are not the cause

Question
: Why does the ncm function (or its config symlink) panic the vendor msm-4.4 kernel, and which function is safe and gives a channel?

Found
: Ranked causes. 1 (source-proven, not yet counted on the phone): the two `echo > functions/ncm.usb0/{dev_addr,host_addr}` lines in every gadget init (vendor, 3, 5b, 5c). In the wahoo 4.4 tree ncm_alloc_inst leaves opts->net NULL (the net_device is made in ncm_bind at UDC bind) and refcnt is 0 until the symlink, so the attribute store calls gether_set_dev_addr(NULL,...): NULL deref, oops, panic. After the symlink the store returns -EBUSY, so no write is ever valid here. Fits all facts: 5a (no function) and gadget4 return; 5b and gadget2/3 panic; mkdir and symlink alone do not touch net (ncm_alloc only refcnt++). 2: the UDC bind (ncm_bind -> gether_register_netdev, dwc3 endpoints) could panic in 5c; unknowable until 1 is removed (Android never uses ncm: its stack is gsi/ffs, so the vendor never exercised this path). 3: the symlink itself: no code path touches net; unlikely. Safe alternatives, with the image that tells: (a) ncm WITHOUT any attribute write (random MACs; set the device side later with ip link set usb0 address): image ncm-noattr = 5c minus the two echo lines; returns at 120 s with no panic and the host sees our strings = confirmed. (b) ffs with a tiny static aarch64 helper: more work, no gain if (a) works; the fallback. (c) gsi.rndis/rmnet: need IPA driver and firmware at early init, not a first channel. (d) mass_storage read-only image: host sees a disk, no log stream; crude fallback. (e) mtp/ptp/acc: need a userspace daemon; no. Caveat: the source is the public android11 branch of the wahoo 4.4 line; the phone runs 4.4.302, the exact build is not public, so the image is the proof.

Sources
: src-wahoo-44-f-ncm,src-wahoo-44-u-ether-configfs,src-wahoo-init-hardware-usb-rc,src-kernel-gadget-configfs-doc

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-17-ncm-panic/


<a id="2026-11-17-walleye-ncm-split-b2-panics"></a>
### 2026-11-17-walleye-ncm-split-b2-panics

Date
: 2026-11-17

Brief
: ncm split: b1 (mkdir ncm.usb0 only) returns at 156 s, reboot; b2 (+ dev_addr, host_addr writes) panics at 75 s: the prediction holds

Question
: Which step of the ncm function bring-up panics the vendor msm-4.4.302 kernel (b1..b4 from the first state, stop at the first panic)?

Found
: From the first state (Android booted on a, slot a active+successful, b unbootable retry 1) by devlab fastboot boot now: vendor-gadget5b1 left fastboot after 12 s, Android back at 156 s, ro.boot.bootreason=reboot. vendor-gadget5b2 left fastboot after 12 s, back at 75 s, bootreason kernel_panic. b2 differs from b1 only by the dev_addr and host_addr writes into functions/ncm.usb0, so creating the ncm function is safe and the attribute writes panic - as predicted (walleye-ncm-panic-why). b3, b4 not tried (stop rule). Flags unchanged afterwards; phone in fastboot on a, nothing flashed.

About
: walleye-first-fastboot-boot


<a id="walleye-ncm-noattr-n1-n2"></a>
### walleye-ncm-noattr-n1-n2

Date
: 2026-11-17

Brief
: ncm-noattr: n1 returns (156 s, reboot); n2 PANICS NOT: our gadget enumerates (1d6b:0104, our strings) after 18 s, but host cdc_ncm bind fails: no MAC string

Question
: Does the ncm gadget without any attribute write bind without a panic, and does the host get a network device?

Found
: From the first state (Android on a, adb reboot bootloader), 2026-11-17: n1 (mkdir function, config, symlink, no bind): left fastboot after 12 s, Android on a at 156 s, bootreason reboot - as predicted. n2 (+ UDC bind): left fastboot after 12 s, host saw USB 1d6b:0104 after 18 s with manufacturer linux-phone-oco, product 'Pixel 2 walleye (mainline)', serial walleye-first-boot - the FIRST time our own kernel strings reach the host; no panic at bind, Android back on a at ~2 min (bootreason reboot via the init's 120 s reboot -f). So the attribute writes were the panic cause (walleye-ncm-panic-why holds). But no network device on the host: dmesg 'cdc_ncm 3-2:1.0: failed to get mac address / bind() failure'; lsusb -v shows the CDC Ethernet descriptor iMacAddress 6 (??): string 6 is not served, so the host cannot read the MAC, usb0 on the phone is unreachable, telnetd 172.16.43.1 not reachable. In the wahoo 4.4 f_ncm.c the MAC string is the static ncm_string_defs[STRING_MAC_IDX].s set in ncm_alloc to the instance's ethaddr (filled at ncm_bind from the net device) - why it is not served is not known yet. After: back in fastboot, slot a active+successful, b unbootable retry 1, nothing flashed.

Sources
: src-wahoo-44-f-ncm

About
: walleye-first-fastboot-boot


<a id="walleye-ffs-acm-helper-how-it-stands"></a>
### walleye-ffs-acm-helper-how-it-stands

Date
: 2026-11-17

Brief
: FunctionFS cannot carry CDC functional descriptors, but cdc_acm binds one class-2 interface with 3 endpoints: the helper declares that, writes ep0 before the UDC bind, bridges to a pty shell

Question
: How can FunctionFS give the host a /dev/ttyACM0 with a shell, on the vendor 4.4.302 kernel (ACM/SERIAL/ECM/RNDIS off, F_FS on, no devtmpfs), from a tiny static aarch64 C helper?

Found
: What is: the running kernel is the stock 4.4.302 with our ramdisk (research walleye-vendor-kernel-facts-s1-panic): USB_CONFIGFS and F_FS =y, ACM/SERIAL/ECM/RNDIS not set, no devtmpfs, UDC a800000.dwc3. Android itself uses ffs.adb in configfs group g1 on this phone (src-wahoo-init-hardware-usb-rc), so F_FS is exercised by the stock stack.

Finding 1 (the trap): f_fs's descriptor parser (v4.4.302 and v7.3 alike) rejects any descriptor type outside interface, endpoint, HID, OTG, interface-association and SS-companion; CS_INTERFACE (0x24, the CDC header/call-management/ACM/union descriptors) falls to default -> -EINVAL. The queued idea of declaring the full CDC functional descriptors would make the write to ep0 fail.

Finding 2 (the way): cdc_acm's probe, with no union descriptor and a class 2 / subclass 2 / protocol 1 interface with exactly three endpoints, assumes a single combined interface and picks interrupt-in, bulk-in, bulk-out with usb_find_common_endpoints. So ONE interface (class 0x02, sub 0x02, proto 0x01) with interrupt-in (10 bytes, interval 9), bulk-out and bulk-in (512 HS/SS, 64 FS) plus SS companion descriptors is enough; the host gives /dev/ttyACM0. Line coding / control-line-state are best-effort for the host.

Design: (1) init: mount configfs; gadget with ids and strings as n2; functions/ffs.acm, configs/c.1 symlink; mkdir /dev/ffs; mount -t functionfs acm /dev/ffs (no devtmpfs: nodes by cpio; ep0 appears in the ffs mount, ep1..3 after the descriptors are written; /dev/ptmx a cpio node, mount devpts). (2) helper (C, static aarch64, clang, GPL base ffs-test.c, libc only): open /dev/ffs/ep0; write the V2 descriptors blob (flags FS|HS|SS), then the strings blob (lang 0x0409); only then write the UDC name (before that the configfs bind returns -ENODEV: f_fs desc_ready). (3) ep0 loop: on ENABLE open ep1..ep3 (names follow descriptor order) and fork a pty shell (posix_openpt, /bin/sh as session leader); on DISABLE/UNBIND close and kill; answer class SETUP events (SET_LINE_CODING 0x20, GET_LINE_CODING 0x21, SET_CONTROL_LINE_STATE 0x22) with a zero-length ack or canned 115200 8N1 so cdc_acm sees no stall; any other setup gets a stall. (4) bridge: poll pty master and ep2 (bulk-out) -> ep3 (bulk-in); ep1 opened and idle. A bulk-in write blocks until the host reads, so use a thread or timeout so a missing host never hangs the shell; log to /dev/kmsg only on state changes. (5) the mass-storage log window stays the second channel.

Failure modes and how each looks: UDC write ENODEV (descriptors not written first, or parse -EINVAL: helper reports errno on /dev/kmsg and the mass-storage window); enumerates but no ttyACM (host cdc_acm rejected: host dmesg); enumerates then drops (ep0 events unanswered: host reset); panic (as 5b, an unwritten function attribute: this design writes no function attribute, only ep0 data) - the bootloader falls back to slot a as before.

Not measured yet: whether the vendor's f_fs (Android common 4.4 plus patches) behaves like stock 4.4.302 here (read, not run); the host side of the ACM bind.

Sources
: src-linux-f-fs-c,src-linux-cdc-acm-c,src-linux-ffs-test-c,src-wahoo-init-hardware-usb-rc

About
: walleye-first-fastboot-boot


<a id="walleye-n3-try-gadget-up-host-lacks-cdc-acm"></a>
### walleye-n3-try-gadget-up-host-lacks-cdc-acm

Date
: 2026-11-17

Brief
: n3 (FunctionFS ACM helper) enumerates as 1d6b:0104 after 18 s with a class-2 interface, but the host has no cdc_acm module for its running kernel 7.3.3

Question
: Does image n3 give a ttyACM and a shell answer on the host (ffs-acm-up.sh)?

Found
: devlab fastboot boot now n3 (sha 6672eb84...): left fastboot 12 s, came back as 1d6b:0104 on 3-2 after 18 s; lsusb shows iProduct 'Pixel 2 walleye (mainline)', iSerial walleye-first-boot, bInterfaceClass 2; no panic. ffs-acm-up.sh (ACM_WAIT=30) fails: interface 3-2:1.0 has no driver. The host runs 7.3.3-1-cachyos but /lib/modules holds only 7.3.4 and 6.18 lts, so modinfo cdc_acm finds nothing; cdc_acm cannot load until the host reboots into 7.3.4 (same wait as try-massstorage's usb_storage). The init's self-reboot returned the phone to Android on a after ~4 min; adb reboot bootloader; fastboot state: a current, successful, not unbootable; b unbootable, retry 1, as before the try. Nothing flashed.

About
: walleye-first-fastboot-boot


<a id="walleye-m1-retry-enumerates-unreadable"></a>
### walleye-m1-retry-enumerates-unreadable

Date
: 2026-11-17

Brief
: m1 on host 7.3.4: enumerates 1d6b:0104, usb_storage binds /dev/sda (2 MiB LUN), but root:disk 0660 and the lab user has no disk group or sudo, so the window is unread

Question
: With the host on 7.3.4, does m1's mass-storage LUN show up and can its first text be read?

Found
: Fastboot boot of vendor-gadget-m1 (sha a3cf08d3...) from slot a good/b unbootable: left fastboot 12 s, enumerated 1d6b:0104 at 18 s, usb-storage bound without modprobe, /dev/sda size 4096 sectors. massstorage-read.sh gave skip: /dev/sda is root:disk rw-rw----, user mll in wheel only, sudo needs a password. The phone fell back by itself to Android on slot a (adb up, boot completed); adb reboot bootloader returned it to fastboot, flags unchanged (a ok/successful, b unbootable retry 1). m1 is the first walleye image of ours proven to run and reach the host; only the host read is missing.

About
: walleye-first-fastboot-boot


<a id="walleye-dtbo-overlay-merge-why-how-it-stands"></a>
### walleye-dtbo-overlay-merge-why-how-it-stands

Date
: 2026-11-17

Brief
: Our dtb has no __symbols__; the dtbo overlay (idx 3) needs 137 labels, so it cannot merge: fdtoverlay fails on ours, passes on stock. Fits s3 'Booting OKAY' then fastboot.

Question
: Does the bootloader's dtbo overlay merge need __symbols__ and the nodes the overlays patch, and is that why our dtb is never started?

Found
: What is: androidboot.dtbo_idx=3 (set by the bootloader) = dtbo.img entry 3 (id 0x1bc rev 3), 147 fragments, label-targeted, __fixups__ naming 137 labels of the base dtb (cci, tlmm, soc, mdss_mdp, pm8998_gpios...). No msm-id/board-id in the overlays; the entry is picked by id/rev, then merged into the dtb of the boot image.
Measured on the host (material: dtbo-measurements.md): fdtoverlay merges entry 3 into the stock dtb (986 symbols, all 137 present) and FAILS on ours: no /__symbols__ node. AOSP's libufdt, the reference for Android's overlay merge, treats that as an error (aosp-libufdt-overlay). Even compiled with -@, our dtb would carry about 14 of the 137 labels (mainline names), and its soc is soc@0, so the fixups would still not resolve.
Fits the bisect: s0n/s0 (stock dtb) start; s2 (our kernel, stock dtb) starts the kernel and falls back; s3 (our dtb, stock kernel) alone reaches 'Booting OKAY' then stays in fastboot - the only variant whose dtb cannot take the overlay. So the dtbo merge is the likeliest cause of s3, and one that outlives every later fix: the vendor ABL will overlay any dtb we hand it. Not known yet: what Google's ABL does on a failed merge (its source is not public; libufdt returns an error, ABL may refuse or continue) - the one cheap image tells.
Does not explain the Load Error of the control (stock dtb merges fine): that stays on research walleye-boot-refused-why-how-it-stands.
Ways: (a) give our dtb the symbols: a compatibility dtsi with stub nodes for the ~123 missing labels, built with -@ - host-testable by fdtoverlay, but adds vendor stubs the mainline kernel ignores; (b) boot the stock dtb for first light and move to ours later - the kernel then runs on the vendor dtb, wrong for mainline drivers; (c) patch the stock dtb: replace its nodes with ours - same merge-safety, much larger; (d) remove the overlay from the path: only by writing dtbo (a write, not proposed); (e) ask the bootloader to skip it: no known way (androidboot.dtbo_idx is its output, not input).
Chosen: (a), narrowed: generate the stub label set from the overlay's own __fixups__ (the 137 names, each with the path it has in the stock dtb's __symbols__), build with -@, prove with tests/host/dtbo-overlay-merges.sh on the host first, then one image on the phone.

Sources
: aosp-libufdt-overlay,aosp-dto-compile,kcxt-dtbo-wahoo

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-17/


<a id="walleye-s4-overlay-symbols-try"></a>
### walleye-s4-overlay-symbols-try

Date
: 2026-11-17

Brief
: s4 (stock kernel + our dtb with overlay symbols): left fastboot at 12 s, Android on a at 36 s, bootreason reboot: the overlay merge was the s3 cause, our dtb is now taken

Question
: Does the control image with only our dtb swapped (-@, __symbols__, 130 overlay stubs) get past the bootloader's overlay merge, unlike s3?

Found
: boot-s3o.img sha256 f61e5e8c... fastboot boot once. Before: slot a active/successful, b unbootable retry 1. Result: left fastboot after 12 s (s3 stayed in fastboot), Android a with adb after 36 s, uptime 0 min, bootreason reboot, current slot a. After: adb reboot bootloader, fastboot, flags identical (a successful, b unbootable retry 1), nothing written. So the bootloader now accepts and starts the image; the s3 hang was the overlay merge. Android on a at 36 s (vs 156 s for a panic return) suggests an early reset of the started kernel with our dtb, cause not yet read; next: bring our kernel with this dtb, or read the reset reason (pstore/last_kmsg after unlock).

About
: walleye-first-fastboot-boot


<a id="walleye-ml1-ml2-no-light"></a>
### walleye-ml1-ml2-no-light

Date
: 2026-11-17

Brief
: ml1 and ml2 from the first state: accepted, left fastboot at 12-15 s, no USB device with our strings in 150 s, Android on a at 69-72 s, bootreason reboot

Question
: Does our mainline kernel start (first light: serial walleye-first-boot, product 'Pixel 2 walleye (mainline)') from boot-ml1.img (Image.gz) and boot-ml2.img (lz4)?

Found
: Both images (sha256 41d8efc0..., 63b2977f...) booted once each from the first state (a current+successful, b unbootable, fastboot). ml1: left fastboot after 15 s, Android adb back after 72 s. ml2: left after 12 s, back after 69 s. mainline-light-up.sh watched 150 s per try: fail both, no device with our strings. Return at ~70 s with bootreason reboot (not kernel_panic), the same time as the vendor-kernel panic fallback: either the panic=5/watchdog path or the bootloader fallback; which is not told by what we can read. Slot flags unchanged after both (a current, successful=yes, b unbootable, retry 1); phone left in fastboot on a. Nothing flashed. Reading it needs ramoops (author unlock) or the UART cable, per decision walleye-panic-read-order; the watchdog node address is still the unmeasured downstream value.

About
: walleye-first-fastboot-boot


<a id="2026-11-17-walleye-m1-window-read"></a>
### 2026-11-17-walleye-m1-window-read

Date
: 2026-11-17

Brief
: m1 on host 7.3.4: the window is read: vendor kernel 4.4.302 runs our init, mass_storage bound, heartbeat grows every 5 s; lpoco-host acceptance passes

Question
: With usb_storage and the uaccess rule in place, what does the phone's first written text show?

Found
: Fastboot booted vendor-gadget-m1 (sha a3cf08d3...) from slot a (a active+successful, b unbootable retry 1): left fastboot 12 s, enumerated 1d6b:0104 after 18 s. tests/host/massstorage-read.sh (LUN_WAIT=60 LUN_WATCH=20) PASSED as the lab user, no sudo: /dev/sda read O_DIRECT, 'WALLEYE-LOG m1 beat=0..4 up=1.3..21.3s', the heartbeat grows every 5 s. Window: uname Linux 4.4.302-gddfd598aee10 (the vendor kernel, our init as pid 1); cmdline is the control's (androidboot.*, verifiedbootstate=orange, dtbo_idx=3, root=PARTUUID, lpm_levels.sleep_disabled=1); mounts rootfs/proc/sys/configfs; udc a800000.dwc3; dmesg tail shows our init steps through 'gadget bound to a800000.dwc3' and configfs-gadget high-speed config #1 CONFIGURED at 3.4 s. No panic in the mass_storage path. Phone returned by itself to Android on a, then adb reboot bootloader: fastboot, flags as before (a active+successful, b unbootable retry 1); nothing flashed. lpoco-host acceptance: pass (70-lpoco-google-walleye.rules uaccess, usb_storage loaded). Full window in the material folder.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-17/


<a id="walleye-ml3-pstore-empty"></a>
### walleye-ml3-pstore-empty

Date
: 2026-11-17

Brief
: ml3 tried once: Android back on a at 72 s, bootreason reboot, /sys/fs/pstore empty though ramoops sits at 0xa1100000 - no console of ours survived

Question
: Does the stock Android's pstore hold our mainline kernel's console after ml3 (ramoops at the phone's own region)?

Found
: boot-ml3.img (sha256 5c257470...) booted from the first state with fastboot boot: left fastboot after 15 s, USB light never (no walleye-first-boot device), Android on slot a after 72 s, androidboot.bootreason=reboot. adb root works (Lineage-like build). Android's ramoops is at mem_address 2702180352 = 0xa1100000, size 2 MiB, record 524288, console 131072, pmsg 0 - it matches our node. /sys/fs/pstore has 0 files, no 'no valid data' line; /data/vendor_ce/0/ramoops holds only pub/priv pem. So no console record of ours: either our kernel dies before the pstore console registers, or the reset clears DDR (a bootloader/PMIC reset, not a warm one - bootreason is plain reboot, not kernel_panic/watchdog). Together with ml1/ml2 the 7.3 image has never shown a sign of life.

About
: walleye-first-fastboot-boot


<a id="walleye-ml4-built"></a>
### walleye-ml4-built

Date
: 2026-11-17

Brief
: ml4 built: ml3 with only /init changed (timing channel); kernel, dtb, header bytes equal ml3's; AVB footer verifies; rebuild is byte-identical

Question
: Is ml4 built so only the ramdisk differs from ml3?

Found
: build/build-ml4.sh unpacks ml3, builds the ramdisk from build/initramfs-mainline/init-ml4, repacks with ml3's header fields and AVB footer; 10 checks ok; rebuilt twice with the same hashes. boot-ml4.img sha256 105364591ce1f46a2a828caf1f80c851399487ab4a59bd777210a9a72e13ea84; initramfs-ml4.cpio.xz sha256 da11b5e005dbe541db83ba9c286e91bfbb44db1058df65bc6e920b12c61dfd19; in ~/Projects/walleye-build/images-ml4/ (SHA256SUMS there). Decision of the build: the init pets the watchdog once a second during its bounded sleep (up to 35 s at score 7) so the SoC watchdog (~30 s) does not cut the delay; before the sleep it is open and unpetted, so a hang there still resets by itself. Bin table: tests/host/ml4-bin-table.sh.

About
: walleye-first-fastboot-boot


<a id="2026-11-17-walleye-cdc-acm-needs-functional-descriptors"></a>
### 2026-11-17-walleye-cdc-acm-needs-functional-descriptors

Date
: 2026-11-17

Brief
: n3r enumerates, cdc_acm refuses it: 'Zero length descriptor references' (-22); cdc_acm needs at least one class-specific descriptor on the interface

Question
: Why does the host not bind cdc_acm to the FunctionFS single-interface console of n3r?

Found
: n3r tried once on slot b (set-active b, fastboot boot): the gadget came up as 1d6b:0104 'Pixel 2 walleye (mainline)' serial walleye-first-boot, one class-2 interface. Host dmesg: 'cdc_acm 3-2:1.0: Zero length descriptor references; probe failed with error -22'. In acm_probe the interface extra bytes (and the first endpoint's extra) are empty, so it returns -EINVAL before the union-descriptor fallback ('castrated device') is reached; that fallback only works when some extra descriptor exists. Fix to try: write a CDC header functional descriptor (plus call-management and ACM, no union required) after the interface descriptor in the FunctionFS descriptors. Contradicts the wording of decision walleye-ffs-acm-single-interface (no CDC functional descriptors), so it goes to the coordinator.

Sources
: src-linux-cdc-acm-c

About
: walleye-first-fastboot-boot


<a id="2026-11-17-walleye-ml4-two-tries"></a>
### 2026-11-17-walleye-ml4-two-tries

Date
: 2026-11-17

Brief
: ml4 twice: Android back at 75 s and 76 s; the bin table reads score 7, but the watchdog's own 69-75 s return cannot be told apart; pstore empty

Question
: Does boot-ml4.img's init run its checklist and reset itself at a score-dependent time?

Found
: boot-ml4.img (sha256 105364591ce1...ea84) fastboot-booted twice on slot b, nothing flashed: fastboot left after 15 s, adb back after 75 s (t0 23:42:58) and 76 s (t0 23:44:40). ml4-bin-table.sh reads 75 s and 76 s as delay 33/34 s = score 7 (all six checks held, including the UFS host device the dts was expected not to give). BUT the safety-watchdog return is not clearly separate: ml1/ml2/ml3 (no timing init) came back at 69-72 s, ml4 only 3-4 s later, inside one 5 s bin; so 75/76 s may equally be the watchdog firing (init not run or hung) - the two readings are not told apart. Android bootreason 'reboot'; /sys/fs/pstore empty and /data/vendor_ce/0/ramoops absent after both, no kernel log of ours survived. Slots after both: current b, a and b successful, not unbootable. Next to tell them apart: an ml5 whose init sleeps a score-independent long time (e.g. a fixed 20 s, and a score 0 variant), so a return near 72 s plus the sleep cannot be the watchdog; not decided here.

About
: walleye-first-fastboot-boot


<a id="walleye-ml5-built"></a>
### walleye-ml5-built

Date
: 2026-11-17

Brief
: ml5a (reboot at once) and ml5b (3 s per point, max 21 s) built: ml3 with only /init changed, hashes in SHA256SUMS

Question
: Can timing bins finish before the ~33 s dead-kernel reset?

Found
: ml5a and ml5b built from ml3 with only the ramdisk /init changed (build/build-ml5.sh; kernel, dtb and header bytes equal ml3's; AVB footer verifies; 20 checks ok; rebuild byte-identical). ml5a: mount proc/sys, kmsg marker, reboot -f at once (sysrq b fallback), no watchdog. ml5b: ml4's checklist with 3 s per point (max 21 s, under the ~33 s dead-kernel reset), watchdog petting kept. Bin table in tests/host/ml5-bin-table.sh: ml5a returns at BASE (expected ~40 s if init ran, ~70 s if the kernel never reached it); ml5b returns at BASE + 3*score. Images in ~/Projects/walleye-build/images-ml5/. sha256: boot-ml5a.img b937e8fba212c08db3343ada25fcfa6045f7ce903e4a5e44ea3c095e782b461e; boot-ml5b.img a0fe6f0bb02484c08a7eb0f74f095e59f12c03aba6b071c62caac4d1ae3d8ed8; initramfs-ml5a.cpio.xz 56d1f4ce247f903f2f23984b25d743f6acaea2182bb0d495a0ae611938af1ee6; initramfs-ml5b.cpio.xz 3357b2b63cdbf91337573359ca9010d221fcba84f3ba1f4e1d352569e2886b89.

About
: walleye-first-fastboot-boot


<a id="walleye-pk1-pk2-built"></a>
### walleye-pk1-pk2-built

Date
: 2026-11-17

Brief
: pk1/pk2 built: porthole's v7.2.2 + 262 patches kernel (taimen / walleye dtb) in ml5a's image, only the kernel blob differs; 21 checks ok, rebuild identical

Question
: Can porthole's known-good mainline kernel be put in our ml5a image (immediate-reboot init) for the phone?

Found
: Built from kernel.org v7.2.2 (sha512 checked) + the 262 numbered patches + config of porthole-dev/pmaports taimen-bringup @ 0d62af2 (every patch and the config checked against the APKBUILD sums; the series has no 0236). clang build of Image.gz (9.9 MB) and the taimen and walleye dtbs with -@. pk1 = taimen dtb; pk2 = walleye dtb (their patch 0029) plus qcom,msm-id <0x124 0x20001> added by us (their walleye file has none). Both dtbs carry the stub nodes for the vendor overlay's labels, and the stock dtbo entry 3 merges into each (fdtoverlay). Images = ml5a with only the kernel blob changed: header, command line (earlycon clk_ignore_unused ignore_loglevel panic=5 androidboot.*), ramdisk (init-ml5a) and AVB footer equal ml5a's, 32 MiB, footer verifies. The porthole cmdline words (qcom_wdt.arm_on_probe, watchdog.open_timeout, reboot=w) are NOT used, so that only kernel and dtb differ from ml5a. Hashes: Image.gz 852f82dc, pk1.dtb b6996001, pk2.dtb fe6bf56b, boot-pk1.img b35aab07, boot-pk2.img 9fab400f (full in SHA256SUMS). Expected: init ran = Android back at ~36-45 s; kernel never reached init = ~70-78 s (ml5a 75/75 s). pk1 back at ~40 s while ml5 was ~75 s would put the cause in our dts/config; pk1 and pk2 both ~75 s would put it in the phone, bootloader path or image.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-pk/


<a id="walleye-pk3-ml6a-built"></a>
### walleye-pk3-ml6a-built

Date
: 2026-11-18

Brief
: pk3/ml6a built: pk1 and ml5a with only earlycon removed from the command line; 22 checks ok, rebuild identical

Question
: Can pk1 and ml5a be built without earlycon on the (possibly gated) debug UART, everything else byte-equal?

Found
: build/build-nocon.sh unpacks pk1 and ml5a, drops earlycon=msm_serial_dm,0xc1b0000 (the only earlycon/console= word; androidboot.console=ttyMSM0 is stock and stays), repacks with mkbootimg and the source's AVB footer props and salt. Checks per image: kernel blob (Image.gz+dtb), ramdisk and every header word but the command line equal the source's; the cmdline differs, has no earlycon or console=, keeps clk_ignore_unused ignore_loglevel panic=5 and all androidboot.* words; size 32 MiB as the source; AVB footer verifies. Rebuild byte-identical. Now: clk_ignore_unused ignore_loglevel panic=5 androidboot.hardware=walleye androidboot.console=ttyMSM0 androidboot.dtbo_idx=3. sha256: boot-pk3.img 7f92abee656bbf3ed3637df8c7b08a7b047db21caa70c341675bf8e631f4f523; boot-ml6a.img aceb7e7a47abcf0713f2708a8f1b6a4dfa2828acb78e7ef618922437022aa02f. Images in ~/Projects/walleye-build/images-nocon. Expected: init ran = Android back at ~36-45 s; ~70-78 s = earlycon was not the hang.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-nocon/


<a id="walleye-nocon-four-returns"></a>
### walleye-nocon-four-returns

Date
: 2026-11-17

Brief
: pk3 back as Android at 63 s and 48 s, ml6a at 75 s and 75 s; slot flags unchanged; pk3 is not the no-init 75 s pattern

Question
: Without earlycon, does init run: pk3 twice, ml6a twice from the first state, return times?

Found
: Images images-nocon on slot b via devlab fastboot boot. pk3: left fastboot 12 s, Android adb back after 63 s then 48 s. ml6a: left after 15 s, Android back after 75 s and 75 s (the no-init watchdog pattern of ml5a, pk1, pk2). Flags after each: a and b successful, unbootable no, retry 0. Reading: ml6a still shows no init; pk3 returns faster than 75 s but slower and more varied than stock 36-45 s, so it is unproven that init ran - needs a log channel (ramoops/UART) to settle. No hands used; one pk3 try was lost to adb not ready (rerun).

About
: walleye-first-fastboot-boot


<a id="walleye-pk456-built"></a>
### walleye-pk456-built

Date
: 2026-11-17

Brief
: pk4/pk5/pk6 built: one swap each between porthole's kernel+dtb (pk3) and ours (ml6a), no earlycon; 31 checks ok, rebuild identical

Question
: Can porthole's kernel and ours be bisected one part at a time, all without earlycon, everything else byte-equal?

Found
: build/build-pk456.sh unpacks pk3, ml6a and ml5b and repacks with pk3's header, earlycon-free command line and AVB footer props/salt. pk4 = pk3's Image.gz + ml6a's dtb (ours, -@ with overlay stubs, init-ml5a). pk5 = ml6a's Image.gz + pk3's taimen dtb (init-ml5a). pk6 = pk3 with ml5b's ramdisk (timing init). Checks: header words, no earlycon/console=, 32 MiB size, AVB footer verifies, dtb reads and has __symbols__, each part equals the named source's and differs from the other's, three images differ. sha256: boot-pk4.img 71f199ce9bfa2bf25107a0b1991e11aeb885ac3b5674e34f6fb30d42307a656b; boot-pk5.img 6ad5ff369d0c0bc57be1dd01d2db5b675964934845def22ce1154fc95ee31327; boot-pk6.img 0c1b341e47e9624dcd0aae22bce60a94a79f9dc778e7c336e095ec30248f2c13. Images in ~/Projects/walleye-build/images-pk456. Expected: init ran (reboot at once) = Android back at about 40-65 s, no init = about 75 s; pk6 = pk3's time + 3 s per score if init ran.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-pk456/


<a id="walleye-pk-bisect-returns"></a>
### walleye-pk-bisect-returns

Date
: 2026-11-17

Brief
: pk bisect: pk3 48 s, pk5 (our kernel + porthole dtb) 42/42 s = init ran; pk4 (porthole kernel + our dtb) 75/69 s = no init: our dtb is the fault

Question
: From slot b, with no earlycon, which swap of kernel/dtb between porthole's (pk3) and ours (ml6a) stops init: return times of pk3 once, pk4/pk5/pk6 twice?

Found
: Via devlab fastboot boot, slot b, no writes. Return as Android (adb): pk3 48 s; pk4 75, 69 s; pk5 42, 42 s; pk6 66, 42 s. Bootreason reboot each, /sys/fs/pstore empty after each (shell has no su, so /data/vendor_ce/0/ramoops unread). Reading: pk5 = our Image.gz with porthole's taimen dtb returns at stock speed, so init ran and our kernel is fine; pk4 = porthole kernel with our dtb gives the no-init 70-75 s pattern, so our dtb is what stops boot. pk6 (pk3 + timing init) 66 then 42 s does not show the 3 s per point; noisy, no conclusion from it. Slot flags unchanged at the end (a,b successful, retry 0), phone left in fastboot on slot b. Next: diff our dtb against porthole's taimen/walleye dtb (memory, ramoops, reserved regions, console/uart nodes) and bisect nodes.

About
: walleye-first-fastboot-boot


<a id="walleye-dtb-bisect-built"></a>
### walleye-dtb-bisect-built

Date
: 2026-11-18

Brief
: d1..d6 built: pk4 (porthole's kernel + our dtb) with one node group removed from our dtb each; 90 checks ok, rebuild identical

Question
: Which node of OUR dtb stops boot with a kernel that reaches init with porthole's dtb?

Found
: Diff of our dtb against porthole's taimen dtb (dtc -I dtb, symbols and the vendor stubs left out): ours is the smaller (4148 vs 7206 lines). Nodes only in ours: watchdog@17817000 (porthole has the same address, compatible apss-wdt-msm8998 plus kpss-wdt and timeout-sec 30), reserved-memory ramoops@a1100000 (theirs 0xb0000000, no-map) and memory@94b00000 slpi 0x700000 (theirs 0x95800000, 0xf00000), RPM regulators-0/-1 (porthole has pm8998-regulators and pmi8998-regulators), qusb2 phy c012000 and the sleep-state pinctrl nodes. Present only in porthole's: aliases, gpio-keys, sound, fingerprint, camera/panel regulators, cpu/cpr opp tables, battery, skin thermal, zap shader region. Our gpu_mem lacks compatible shared-dma-pool. build/build-dtb-bisect.sh removes with fdtput -r from our dtb in pk4's bytes: d1 watchdog; d2 ramoops; d3 slpi carve-out; d4 RPM regulators; d5 usb@a8f8800 + phy@c012000; d6 = d1+d2+d3. spmi is not a group (the dtbo overlay needs it: it would not merge). Per image: header, command line (no earlycon), ramdisk, Image.gz equal pk4's, dtb differs and is the one written, 32 MiB, AVB footer verifies, dtb reads and has __symbols__, stock dtbo entry 3 merges. 90 checks ok, rebuild identical. sha256 in SHA256SUMS: boot-d1 b446afcc, d2 c1b89833, d3 a74f9f4d, d4 9defe73d, d5 d61531c8, d6 8da77454. Expected: init ran = Android back at about 40-65 s, no init about 75 s.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-dtb-bisect/


<a id="walleye-pk5l-built"></a>
### walleye-pk5l-built

Date
: 2026-11-18

Brief
: pk5l built: pk5 (our kernel + porthole's dtb) with a gadget init, ACM shell on ttyGS0 and the mass_storage window; 39 checks ok, rebuild identical

Question
: Can pk5 carry a first mainline userspace with a console and a log window, all else byte-equal?

Found
: build/build-pk5l.sh repacks pk5 with only the ramdisk changed (init-pk5l, busybox arm64 static). Checks: kernel+dtb bytes, header, no earlycon/console=, 32 MiB, AVB footer verifies, init in ramdisk equals source and has no bashisms, the busybox config has every applet it uses, our kernel config has USB_GADGET, CONFIGFS_ACM, CONFIGFS_MASS_STORAGE, DWC3 (+QCOM, DUAL_ROLE), QUSB2 phy, QCOM_WDT, CONFIGFS_FS, DEVTMPFS, PSTORE_RAM; porthole's dtb carries dwc3 peripheral high-speed with the qusb2 phy. The init binds one gadget with acm.GS0 and mass_storage.0, a respawned ash on /dev/ttyGS0, a 5 s log window, and pets the watchdog for 180 s (longer while /tmp/hold), then reboot -f. sha256: boot-pk5l.img a313a0c3fe927a4c6980cc320800baeb99b48ec3bd8f593de99394289e74469c; initramfs-pk5l.cpio.xz 9ab2f0945ccb9e288da45ee63f185934a4f2047d8dcd583facad62550c23cc5f. Host test tests/host/mainline-acm-up.sh. Not run on the phone.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-pk5l/


<a id="walleye-pk5l-tried"></a>
### walleye-pk5l-tried

Date
: 2026-11-18

Brief
: pk5l booted twice from fastboot: no ACM gadget ever enumerated; the phone came back as Android (adb) after 57 s and 42 s

Question
: Does pk5l (our kernel, porthole's dtb, gadget init) bring up the ACM console and the mass_storage window?

Found
: Two tries from the first state (slot b current, a and b successful). Each: left fastboot after 15 s; ACM_WAIT 60 and 40 s: no walleye-first-boot gadget, no ttyACM, no 18d1 device but fastboot; then Android adb gadget at 57 s / 42 s (the bootloader fell back to slot b's Android, i.e. the init never held 180 s nor showed a gadget). Window not read. Slot flags unchanged after (b current, both successful, retry 0). Return to fastboot by adb reboot bootloader. Suite latest not run: no devlab password (fastboot-only phone). Next: the dtb diff/bisect (walleye-try-dtb-bisect).

About
: walleye-first-fastboot-boot


<a id="walleye-pk5l2-built"></a>
### walleye-pk5l2-built

Date
: 2026-11-18

Brief
: pk5l2 built: pk5l with only the dtb's ramoops node moved to the phone's own region 0xa1100000; 20 checks ok, rebuild identical

Question
: Can pk5l's early end be read from Android's pstore: an image that differs from pk5l only in the ramoops node?

Found
: boot-pk5l2.img (sha256 222c6196...b5fd): fdtput on porthole's dtb in the kernel blob, reg 0xa1100000/2 MiB, record 0x80000, console 0x20000, pmsg 0; same dtb size, only that node differs, no overlap, kernel config has PSTORE_CONSOLE and PSTORE_RAM; kernel, ramdisk, header, cmdline, footer are pk5l's.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-pk5l2/


<a id="walleye-dtb-bisect-tried"></a>
### walleye-dtb-bisect-tried

Date
: 2026-11-18

Brief
: pk4 and d1..d6 (twice each): all 13 returned to Android at 75 s, the dead-kernel path; no group removal made init run

Question
: Which node group of our dtb stops init (d1 watchdog, d2 ramoops, d3 slpi, d4 RPM regulators, d5 usb+qusb2, d6 = d1+d2+d3)?

Found
: 13 boots from fastboot, each left fastboot after 12 s and came back as Android after 75 s (the no-init time; init would be 40-65 s). No image differed, d6 included. So none of the removed groups alone or d1+d2+d3 together is the cause. pstore not read (needs root). Slots unchanged after: b current, a and b successful. Next: diff our dtb against porthole's taimen dtb for other nodes and build d7+.

About
: walleye-first-fastboot-boot


<a id="walleye-pk5l2-tried"></a>
### walleye-pk5l2-tried

Date
: 2026-11-18

Brief
: pk5l2 booted twice from fastboot: no mainline text in pstore either time; try 1 left only the stock Android kernel's own panic (ramoops reader)

Question
: Does pk5l2 (ramoops in the phone's own region 0xa1100000) leave a mainline panic or console text in Android's pstore?

Found
: Both tries: fastboot boot, phone back as Android after 60 s and 42 s, adb root. Try 1: pstore held only the stock 4.4.302 Android kernel's boot; it itself oopsed at 17.9 s in init (persistent_ram_save_old -> __memcpy_fromio, paging fault at ffffff8009c60000, Kernel panic - not syncing: Fatal exception), rebooted, bootreason kernel_panic; no Linux version 5-9 anywhere. Try 2: pstore empty, no panic, stable 3+ min. So the mainline kernel wrote nothing Android can read at 0xa1100000: it dies before ramoops probes, or the region differs. The Android oops suggests its mapping of the old region is itself fragile (excerpt in material).

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-pk5l2-tried/


<a id="walleye-overlay-enables-what-result"></a>
### walleye-overlay-enables-what-result

Date
: 2026-11-18

Brief
: The walleye dtbo overlay does less on our dtb than on the working taimen dtb; the difference is node status in the base dtb, not the overlay

Question
: What does the stock dtbo overlay (entry 3) switch on in OUR dtb that it does not in porthole's taimen dtb?

Found
: fdtoverlay merges entry 3 into pk5's taimen dtb and into ours; the merged dtbs were flattened and diffed. Of the 137 labels the overlay names, 7 are real nodes in ours (mdss_dsi0, mdss_dsi1, mdss_mdp, pcie0, soc, spmi_bus, tlmm), 130 land on empty stubs the kernel ignores; the taimen dtb has all 137 real. On existing nodes the overlay changes in ours: i2c@c179000 (disabled to okay, compatible qcom,i2c-msm-v2, no mainline driver: inert), pcie and dsi properties (nodes stay disabled: inert), one display-controller property. Everything it adds under real parents (gpio_keys, serial@0c16f000, bt_wcn3990, tps61099, panel nodes, pmic nodes) it adds to the taimen dtb too: ours' added node set is a subset of pk5's (232 against 269). The overlay hypothesis is falsified; a label-neutralizing bisect would test almost nothing. What differs between the alive taimen dtb and the dead walleye dtbs (porthole's pk2 too, same kernel) is the base: the walleye dtbs leave 15 nodes disabled that taimen enables (camss, cci, dma-controller@17184000, iommu@5100000, video-codec, slim-ngd, five i2c masters, pcie and its phy, pon resin, pmic@3 led-controller) and lack ~100 nodes taimen has (fingerprint, gpio-keys, cam regulators, sound, skin thermal, i2c children, pin states). Ranked suspects: dma/iommu group, pcie, i2c, camera, removed nodes. Next: images w1..w5, wr, wa from pk5 (research walleye-taimen-walleye-built).

About
: walleye-first-fastboot-boot


<a id="walleye-taimen-walleye-built"></a>
### walleye-taimen-walleye-built

Date
: 2026-11-18

Brief
: w1..w5, wr, wa built: pk5 (alive) with the walleye dtb's differences put in group by group; 399 checks ok, rebuild identical

Question
: Which difference between the alive taimen dtb and the dead walleye dtbs stops init?

Found
: build-taimen-walleye.sh builds seven images from pk5 (our kernel + porthole's taimen dtb, init 42 s): w1 camera, w2 dma/iommu/codec/slim, w3 i2c, w4 pcie, w5 pmic key/led (each: status disabled, children removed), wr nodes only taimen has removed (dangling references left on purpose), wa all of them: node set and every status equal pk2's. Header, command line, ramdisk (reboot at once), Image.gz, AVB props are pk5's; the stock overlay still merges in all. Bisect plan: wa twice first (~42 s init ran, ~75 s dead): wa alive means the cause is a property, stop; wa dead then w1..w5 and wr twice each, the dead ones hold it, halve inside.

About
: walleye-first-fastboot-boot


<a id="walleye-wa-tried"></a>
### walleye-wa-tried

Date
: 2026-11-18

Brief
: wa (all walleye dtb differences on pk5) tried 3 times: returned at 42, 63, 63 s - not the 42 s alive, not the 75 s dead

Question
: Does wa (taimen dtb with every walleye difference) run init (~42 s) or die (~75 s)?

Found
: Three tries of boot-wa.img from fastboot on slot b, nothing flashed: back as Android after 42 s, 63 s, 63 s (devlab's measure, includes the 15 s in fastboot). Slots' flags unchanged (a and b successful, none unbootable). The plan's rule (alive ~42 means a property) is not met cleanly: one alive-looking, two between. 63 s twice may be a third path (kernel ran longer before reset) or timing noise; unresolved. w1..w5 and wr not tried yet. Note devlab phone needs the missing password file; plain adb used to return to fastboot.

About
: walleye-first-fastboot-boot


<a id="walleye-w1-w5-wr-tried"></a>
### walleye-w1-w5-wr-tried

Date
: 2026-11-18

Brief
: w1..w5 and wr twice each: all 12 returned at 42-63 s, ALIVE by the 66 s rule; no group of walleye dtb differences kills init

Question
: Which walleye dtb difference group, put into the alive taimen dtb, stops init?

Found
: Return times (s): w1 48,63; w2 42,63; w3 42,42; w4 63,42; w5 42,42; wr 42,42. Slot b, nothing flashed, flags unchanged. Rule from the coordinator: up to ~66 s alive, 69+ dead (all earlier dead images 69-78). With wa (42,63,63) also alive, every taimen-based image is alive, though pk2 (same nodes/statuses, porthole's walleye dtb) is dead: the cause is a property, not a node or group.

About
: walleye-first-fastboot-boot


<a id="walleye-property-diff"></a>
### walleye-property-diff

Date
: 2026-11-18

Brief
: wa and pk2 differ in only five property groups; memory, reserved-memory, chosen, cpus, timer and psci are identical

Question
: Which properties differ between wa (alive taimen-based dtb with the walleye node set and statuses) and pk2 (porthole's walleye dtb, dead)?

Found
: Decompiled both (dtc -I dtb -O dts -s), phandles resolved to node paths (build/dts-norm.awk): 31 changed property lines (12 in wa only, 13 in pk2 only, 6 raw renumbered). Groups: root compatible+model; battery capacity (0x35b600 vs 0x2932e0, two properties); panel@0 has no compatible in pk2 (wa lg,sw43402); qcom,protection-domain avs/audio + msm/adsp/audio_pd on seven apr services only in pk2; leftovers in nodes disabled in both (camss supplies, pcie reg/ranges/supply, resin linux,code). Nothing in /memory, reserved-memory, chosen, cpus, timer, psci. Not property-level: phandle numbering, node order, dtb size (wa 132369 B, pk2 128809 B) - if the all-groups image wpa lives, the cause is there, not in a property.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-property-diff/


<a id="walleye-property-bisect-built"></a>
### walleye-property-bisect-built

Date
: 2026-11-18

Brief
: wp1..wp5 and wpa built: wa with one group of pk2's property differences each; 68 checks ok, rebuild identical

Question
: Images for the property-level bisect between wa (alive) and pk2 (dead)

Found
: build/build-property-bisect.sh: six boot images, all of wa's header, command line, ramdisk, Image.gz and AVB footer, only the dtb changed by fdtput; each image changes exactly its group's property lines (checked on the normalized diff); wpa equals pk2's properties; the stock dtbo overlay merges in all; hashes in the material folder. Plan for the phone: wpa twice first (alive <=66 s means the cause is not a property: stop, look at phandle order/size); dead: wp1..wp5 twice each, the dead ones hold it, halve inside. Expected: wp4 and wp3 likeliest.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-property-diff/


<a id="walleye-wpa-tried"></a>
### walleye-wpa-tried

Date
: 2026-11-18

Brief
: wpa (taimen dtb + all property groups) booted twice from fastboot: back as Android at 60 s and 60 s, the alive class

Question
: Does adding all walleye property groups to the taimen dtb (wpa) make the boot die (>=69 s) or stay alive (<=66 s)?

Found
: wpa returned at 60 s both times (alive class, <=66 s). So no property group causes the dead boot; wp1..wp5 were not tried, being unneeded. Remaining suspects: phandle order or dtb size. Nothing flashed; slot b current, a and b successful, phone in fastboot.

About
: walleye-first-fastboot-boot


<a id="walleye-pk5g-built"></a>
### walleye-pk5g-built

Date
: 2026-11-18

Brief
: g1..g3 built: pk5 (alive) with one gadget step per ramdisk (dir+strings; +mass_storage; +UDC bind), each holds 60 s; 58 checks ok, rebuild identical

Question
: Can the mainline gadget be staged on the alive base so each image adds one step and alive and dead are 30+ s apart?

Found
: g1, g2, g3 are pk5's kernel, dtb, cmdline, header and size byte for byte; only the ramdisk differs, and the three ramdisks differ in /stage alone (one init, init-pk5g). Each pets the watchdog every second, holds 60 s, then reboot -f. No ACM, no shell loop. 58 checks ok (kernel config has the gadget, configfs, dwc3, qusb2, wdt options; AVB footer verifies), rebuild byte-identical. Not run on the phone.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-pk5g/


<a id="walleye-pstore-warm-reset-how-it-stands"></a>
### walleye-pstore-warm-reset-how-it-stands

Date
: 2026-11-18

Brief
: Empty pstore after mainline tries is not yet evidence of a cold reset: no try showed a kernel alive; and mainline never asks the PMIC for a warm reset the way the vendor kernel does

Question
: Why does our mainline kernel's console never survive in pstore, and how does an MSM8998 kernel ask for a warm reset so ramoops at 0xa1100000 keeps its text?

Found
: READ FROM SOURCE. (1) Vendor wahoo 4.4 msm_restart_prepare: the PMIC reset type is set explicitly before every restart; WARM only if download mode is on (in_panic turns it on) or the reboot command is non-empty ('reboot recovery'); a plain reboot is a PMIC HARD reset ('unless memory contents must be maintained'). It then de-asserts PS_HOLD through TZ. So on the phone, only panics and 'reboot <arg>' keep DDR; this is why stock Android's console-ramoops survives a panic but not a plain reboot. (2) Mainline: reboot=w sets REBOOT_WARM, but psci_sys_reset uses SYSTEM_RESET2 only if PSCI is 1.1+; otherwise plain SYSTEM_RESET (notifier priority 129, before qcom-wdt's 128, so the watchdog restart is never reached). The pm8998-pon driver only registers reboot-mode (soft spare reason) and never programs the PS_HOLD warm/hard reset type. Nothing in mainline does what msm_restart_prepare does; panic=5 and sysrq c therefore end in the same plain reset, whatever the PMIC default is (set by SBL/XBL: not known yet). (3) The lab's tries (ml1-ml5, pk*) never showed a kernel alive (no USB, no earlycon on the host), so an empty pstore says nothing yet about warm or cold: ramoops registers at postcore, so a kernel that dies earlier, or never starts, leaves nothing. RANKED CAUSES: (A) no kernel code ran far enough to write (most likely for ml*; test: any sign of life first, e.g. the m1 mass-storage log); (B) plain reset = PMIC hard reset, DDR lost (certain to matter once A is cleared; mainline sets no reset type); (C) the reader: stock 4.4 ramoops layout/ECC/zone count differs from the 7.x writer's, or Android init consumes the files (read with adb root right after boot; compare the 4.4 and 7.x zone maths before trusting an empty read; the clean control is the SAME kernel reading its own region after a warm reset); (D) bootloader zeroes the region on a path other than a warm reset (not shown by anything; Pixel stock Android relies on survival after panic, so XBL/ABL keep DDR on the warm path). NOT TESTED: all of it; nothing here ran on the phone. CHEAP IMAGE CHANGES, each one image, expected pstore: (T1) init marker to /dev/kmsg, 20 s wait, 'echo c > /proc/sysrq-trigger' with panic=5: console/dmesg-ramoops present only if the reset kept DDR; expected ABSENT (cause B). (T2) same, cmdline reboot=w added: expected the same, because PSCI on MSM8998 TZ is 1.0 as far as known (dmesg 'PSCIv' line to confirm) so no RESET2. (T3) init opens /dev/watchdog (kpss-wdt, already in the dts) with a short timeout and never pets it: a bite is a SoC watchdog reset which the vendor treats as a dump-capable warm path; expected console-ramoops PRESENT; this is the one image change with real hope and needs no kernel patch. (T4) kernel patch: a reboot notifier writing the PON PS_HOLD reset-type register to WARM (SPMI regmap of the pm8998 pon node; register and values to be read from the vendor qpnp-power-on source and header before writing) and arming it in the notifier; a patch worth sending upstream if T3 does not hold. READ-BACK WITHOUT PSTORE: after any return the PMIC's own record (PON_ON_REASON 0x40 = warm sequence, PON_WARM_RESET_REASON) tells which reset ran; stock Android exposes it in bootreason and in /sys/class/power_supply or qpnp pon sysfs; the live mass-storage log of init-pk5l (rewritten every 5 s) already gives the console without any pstore once a kernel is alive.

Sources
: src-wahoo-msm-poweroff,src-linux-psci-sys-reset,src-linux-qcom-pon,src-qcom-pon-reset-status,src-linux-qcom-wdt,src-openwrt-ramoops-warm,src-kernel-ramoops-doc

About
: walleye-first-fastboot-boot


<a id="walleye-pk5-stages-tried"></a>
### walleye-pk5-stages-tried

Date
: 2026-11-18

Brief
: g1, g2, g3 (pk5 + gadget steps) twice each: all back as Android; g1 42-43 s, g2 138 s, g3 42-48 s; no 1d6b gadget enumerated in 100 s, even by g3; pstore empty

Question
: Which gadget step on pk5 (dir+strings; +mass_storage; +UDC bind) first kills init, and does g3 enumerate?

Found
: Each of g1,g2,g3 booted twice from fastboot (slot b, nothing flashed), lsusb polled each second for 100 s, pstore read after each via adb root. Left fastboot at 15 s every time. Back as Android: g1 43 s,42 s; g2 138 s,138 s; g3 48 s,42 s. USB showed only fastboot then Android's 18d1:4ee7: no 1d6b device ever, so g3 (UDC bind) did not enumerate as the item expected (~25 s). pstore empty in the tries read (g1 try 1 not read). g2 returns at 138 s, outside the 42-63 alive and 75 dead classes. Battery 4377 mV read after the tries (the battery rule landed during the run).

About
: walleye-first-fastboot-boot


<a id="walleye-t1-built"></a>
### walleye-t1-built

Date
: 2026-11-18

Brief
: T1 built: pk5 with ramoops at 0xa1100000 and an init that marks /dev/kmsg, waits 20 s, sysrq c (panic=5); checks ok, rebuild identical

Question
: Can the sysrq-c try be one image that differs from pk5 only in the ramoops node and the ramdisk?

Found
: boot-t1.img sha256 435ec18473e74a95cfcb64ef6e9990400e1b073500373e60ca7bfe684d0f078d (initramfs-t1.cpio.xz f75c8c02...7720): pk5's Image.gz, header and size; only the ramoops node differs in the dtb (reg 0xa1100000/2 MiB, console 0x20000, same size, no overlap); command line has panic=5, no earlycon/console=; config has MAGIC_SYSRQ, PSTORE_CONSOLE, PSTORE_RAM, QCOM_WDT; AVB footer verifies; checks ok, rebuild identical; the init and reader share one marker (checked). Reader tests/host/pstore-t1-read.sh exits 2 (not measured) without a rooted adb, so an unrooted read is not taken for cause B. Reviewed by the programmer subagent (5 findings fixed). Not run on the phone.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-t1/


<a id="walleye-g1-tried"></a>
### walleye-g1-tried

Date
: 2026-11-18

Brief
: g1 twice from the first state (slot b, a and b successful): back as Android at ~108 s and 64 s; no 1d6b:0104 and no pstore either time

Question
: Does pk5g's first stage (gadget dir+strings) keep init alive and show a USB gadget?

Found
: Start state read first: fastboot, current slot b, both slots successful, none unbootable; the earlier keeper left no record but the state was clean. g1 try 1: left USB at ~15 s, back as adb at ~108 s (alive class), pstore empty, bootreason reboot. g1 try 2: left at 14 s, back at 64 s (neither 108 alive nor 75 dead). Neither showed a 1d6b:0104 device. Slot flags unchanged.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-pk5g-tried/


<a id="walleye-g2-try1"></a>
### walleye-g2-try1

Date
: 2026-11-18

Brief
: g2 (+mass_storage) try 1 from the first state: left USB at 15 s, back as Android at 138 s; no 1d6b:0104 seen

Question
: Does pk5g's second stage show a USB gadget?

Found
: g2 try 1: fastboot boot accepted, left USB at 15 s, no 1d6b:0104 in the per-second lsusb log (g2-try1-usb.log), back as adb at 138 s: later than the 108 s alive and 75 s dead classes. Slot flags unchanged (b current, a and b successful). Second g2 try and g3 still to run.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-pk5g-tried/


<a id="walleye-t3-built"></a>
### walleye-t3-built

Date
: 2026-11-18

Brief
: T3 built: pk5 with ramoops at 0xa1100000 and an init that opens the kpss-wdt with a 10 s timeout, pets once, never again; checks ok, rebuild identical

Question
: Can the unpetted-watchdog bite try be one image that differs from T1 only in the init?

Found
: boot-t3.img sha256 9ccc6e2787f8a7bcc7ba9cf54ce0ab3423dd18bfc9061ea49fff636a9ff88993 (initramfs-t3.cpio.xz 991cc9cd75c4a395...): same kernel, header, size and dtb edit as T1; the init writes marker walleye-t3-marker-5c1e to /dev/kmsg, then runs busybox 'watchdog -T 10 -t 3600 -F /dev/watchdog' (one ping, timeout 10 s, next ping an hour away), logs ticks for 45 s, and has no sysrq c, so a return before 45 s can only be the bite. pk5's dtb already carries watchdog@17817000 (apss-wdt-msm8998, kpss-wdt, timeout-sec 30), so the node needs no measuring to be tried: the try itself measures it (no node response = no bite = return at the 45 s reboot -f). 43 checks ok (ramoops node, no overlap, dtb differs only there, wdt node present, panic=5, config has QCOM_WDT and PSTORE_RAM, busybox has the watchdog applet, AVB footer verifies), rebuild identical. Reader tests/host/pstore-t3-read.sh passes only with the marker, a last tick under 15 and no no-bite line (a marker without a bite fails); exits 2 (not measured) without a rooted adb. Not run on the phone.

Sources
: src-linux-qcom-wdt

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-t3/


<a id="walleye-t3-tried"></a>
### walleye-t3-tried

Date
: 2026-11-18

Brief
: T3 (pk5 + ramoops 0xa1100000 + unpetted kpss-wdt, bite at ~10 s) tried once: back as Android at 42 s (alive class), pstore holds no T3 marker

Question
: Does a hardware watchdog bite leave our console in ramoops?

Found
: boot-t3.img (sha 9ccc6e27...) from fastboot on slot b; left fastboot 15 s, adb back at 42 s, same as other alive-class images; pstore-t3-read: fail, 0 console-ramoops in /sys/fs/pstore and vendor_ce, bootreason reboot. Dtb carries watchdog@17817000 and QCOM_WDT=y. Not separated: bite never happened vs bite without surviving ramoops vs init never ran; no UART/serial console.

About
: walleye-first-fastboot-boot


<a id="walleye-vf1-built"></a>
### walleye-vf1-built

Date
: 2026-11-18

Brief
: vf1 built: the control's kernel with an init that dumps the running hardware's facts into an 8 MiB read-only window; 23 checks ok, rebuild identical, reader selftest 9 ok

Question
: Can the vendor kernel be made to hand the running hardware's facts to the host through the m1 window?

Found
: boot-vf1.img (sha256 af1ca884...fffe in images-vf1/SHA256SUMS-vf1): header, kernel, footer as m1's; ramdisk init-vf1 mounts proc/sys/configfs/debugfs, binds m1's mass_storage gadget with an 8 MiB LUN, and every 10 s for 90 s rebuilds a gzipped tar (device tree per property, clk/regulator/pinctrl/gpio/usb/dwc3 debugfs, interrupts, iomem, ioports, udc, dmesg) written as 64-byte header then body, then reboot -f. Redaction in the init (no mac-address/bdaddr/serial-number/imei properties, no raw fdt, no /sys/class/net, serialno/IMEI/MAC blanked) and again in the host reader, which deletes a dump that fails. Programmer review found 1 build-failing check, the raw fdt leak, a 64-byte O_DIRECT read, a leftover raw copy and an over-broad name filter; all fixed. Not yet run on the phone.

About
: walleye-first-fastboot-boot


<a id="walleye-vf1-tried-twice"></a>
### walleye-vf1-tried-twice

Date
: 2026-11-18

Brief
: vf1 twice from the first state (slot b): gadget 1d6b:0104 up at 18 s, LUN /dev/sda readable, header never WALLEYE-VF1; no facts read

Question
: Does vf1 write the vendor-facts blob into its mass_storage window, readable on the host?

Found
: Host now on 7.3.4 with usb_storage, so the earlier module block is gone. Two tries (battery 4378 mV, slots a and b successful, boot from fastboot on b, nothing flashed): left fastboot at 12 s, enumerated 1d6b:0104 with our strings at 18 s, /dev/sda read by the reader, but the 64-byte header was empty (not WALLEYE-VF1) over 12 reads in try 1 and the full LUN_WAIT=60 VF_TRIES=60 run in try 2. Phone then left USB and came back as Android on b, flags unchanged, fastboot reachable. Not told apart: init never wrote a blob (dump step failing), host read of the LUN cached or empty, or the vendor kernel dying before the first round. Next, by the coordinator: read the vf1 init's dump step for what it writes before the first round, or add a marker round written at once; a UART remains the sure witness.

About
: walleye-first-fastboot-boot


<a id="walleye-vf2-built"></a>
### walleye-vf2-built

Date
: 2026-11-18

Brief
: vf2 built: vf1's dump with the marker at once, a 2 s beat, and a status line per step in a 64 KiB text region; 29 checks ok, rebuild identical, selftest 25 ok, reviewed

Question
: How can the window tell an empty header from a dead init?

Found
: boot-vf2.img = the control's kernel with init-vf2: the window file and the WALLEYE-VF2 BEAT marker are written before any mount and before the gadget; a beat repaints every 2 s under flock; every step goes through stp (start line before, rc=X and stderr after; a step with failed grabs gets a non-zero rc); applet list written at start; blob at offset 65536 with its own magic. The reader prints the step lines first, retries while the beat lives, and says ALIVE or DEAD/hung. sha256 boot-vf2.img 75723a23acf85e18... (full in images-vf2/SHA256SUMS-vf2 on the lab). Programmer review found 5 issues (stale overwrite between painters, marker order, rc=0 on failed grabs, reader retries, selftest beat); all fixed. Not tried on the phone.

About
: walleye-first-fastboot-boot


<a id="walleye-vf2-tried-twice"></a>
### walleye-vf2-tried-twice

Date
: 2026-11-18

Brief
: vf2 twice from the first state (slot b): init ALIVE (beat ticking 2 s, steps 0-4 ended rc=0, applets ok, step 5 g-idVendor reached by 43 s); blob not read, reader unreliable

Question
: Does the vf2 init run, and how far does it get?

Found
: Try 1: gadget 1d6b:0104 at 18 s, reader found no marker/empty text on its re-reads (phone gone from USB before any manual read). Try 2: polling the LUN every 2 s from 15 s showed WALLEYE-VF2 BEAT=2..22 (UP 3.4..43.5 s), 'start 0 init (kernel 4.4.302-...)', 21 applets ok, steps mount-sysfs/configfs/debugfs/g-mkdir rc=0, 'start 5 g-idVendor' at 43 s; my capture cut at 600 bytes so later steps and any blob header unseen; then the phone left USB. Back as Android on b both tries, left in fastboot, slots a and b successful. So the earlier empty header meant the first dump round had not been written yet, not a dead init. The init runs the VENDOR kernel 4.4.302 inside (the instrument). Reader: reads of the 64 KiB text region came back empty at times (O_DIRECT while the init rewrites every 2 s?); a plain dd poll worked.

About
: walleye-first-fastboot-boot


<a id="walleye-vf3-built"></a>
### walleye-vf3-built

Date
: 2026-11-18

Brief
: vf3 built: gadget bound first, the dump appended progressively as framed text blocks in a 4 MiB window, a host poller; 24 checks ok, rebuild identical, selftests ok, reviewed

Question
: How does the window keep what the phone shows before it leaves USB at ~45-60 s?

Found
: boot-vf3.img = the control's kernel with init-vf3: window file + WALLEYE-VF3 beat block 0 (2 s), proc/sysfs/configfs, gadget bound, THEN one source at a time (cmdline, version, udc, iomem, interrupts, clk/regulator/gpio/pinctrl, usb/dwc3, ioports, devices, applets, dmesg head/tail, device tree one node per block) framed BEGIN/END rc=N, appended from the last 4096-byte boundary and synced, ending in DONE; nothing to the console; stream budget 3.8 MB. tests/host/vendor-facts-poll.sh (start BEFORE the boot, own job): finds the LUN by our strings, copies the used part every 2 s (numbered copies, last-complete), fetches each finished block once as a flat file, stops at DONE or 150 s, refuses and deletes on a MAC/15-digit/serialno. Programmer review: nested names aborted the fetch (fixed: flat dt%a%b names), torn reads could glue across a hole (fixed: contiguous prefix, torn last line dropped), leak check now on the whole copy before any save, budget line once. sha256 boot-vf3.img cb5c7ad9bad50d80... (full in images-vf3/SHA256SUMS-vf3 on the lab). Selftest poll 25 ok, reader 25 ok.

About
: walleye-first-fastboot-boot


<a id="walleye-vf3-tried-twice"></a>
### walleye-vf3-tried-twice

Date
: 2026-11-18

Brief
: vf3 twice from the first state (slot b), poller started first: no gadget (no walleye-first-boot / Pixel 2 walleye strings) ever on the host; phone back as Android at 78 s both times

Question
: Does vf3 (gadget bound first, dump appended after) show its window and facts?

Found
: Both tries: poller (150 s, own job) saw no USB device with our strings; usb log of try 2 (1 s): fastboot until left at 12 s, then nothing on USB for ~64 s, then Android adb (18d1:4ee7) at 78 s. vf2 had the gadget up at 18 s on the same host. So vf3's reordered init did not reach the gadget (or crashed before it): the 78 s return is the dead-init class, not alive. Nothing flashed; phone left in fastboot on b, flags a and b successful, battery 4376 mV.

About
: walleye-first-fastboot-boot


<a id="walleye-vf4-built"></a>
### walleye-vf4-built

Date
: 2026-11-18

Brief
: vf4 built: vf2's init byte for byte to the gadget bind (first 86 lines), then vf3's progressive dump after it; 27 checks ok, rebuild identical, selftests ok

Question
: Which change between vf2 (alive) and vf3 (dead before the bind, twice) killed vf3, and what is the smallest step from vf2 toward a readable dump?

Found
: Diff vf2->vf3 BEFORE the bind (what vf3 did that vf2 did not, or not as vf2 did): (1) no 'say' to /dev/console and /dev/kmsg per step - vf2 wrote the console twice per step, and its gadget bind came only at 43 s; vf3 bound within seconds of boot; (2) no debugfs mount before the bind (vf2 mounted debugfs before); (3) no applet loop, no flock'd 64 KiB text region, no per-step start/step lines; (4) window 4 MiB with a 4096-byte beat block instead of 8 MiB with a 64 KiB text region. Likely cause (a hypothesis, not measured): the bind itself came too EARLY - vf2's slow console writes put it after the kernel's late unused clock/regulator cleanup (~30 s), vf3's bind ran before it, and the UDC/PHY went down or the phone reset; vf4 keeps vf2's timing so a dead vf4 would speak against this, an alive one for it. Not excluded: the missing debugfs mount or the window size. vf4 = vf2's init lines 1-86 unchanged (up to and including the 'gadget' call = UDC bind; diff vf2->vf4 first hunk at line 87, md5 of the 86 lines equal, checked by the build), then vf3's stream appended from byte 65536 of vf2's 8 MiB window (framed BEGIN/END blocks, flushed from the last 4096-byte boundary and synced, a 'chunk NAME' step line first so a hang shows in the text region, timeout 5 on debugfs reads, 3.8 MB budget, DONE, hold to 90 s then reboot -f). Poller of vf3 reused with VF_MARK=WALLEYE-VF2 VF_HEAD=16 VF_BLOCKS=1040 (reads 4259840 bytes; also keeps the 64 KiB text region as steps.txt, leak-checked). sha256 (first 16) 6184dbdc240ead5c 681b5d0858cde6e1  full in ~/Projects/walleye-build/images-vf4/SHA256SUMS-vf4 on the lab. Selftests: poll 25 ok (vf3 layout unchanged), vf4 13 ok. Programmer review (no defect on the first read) led to: POLL_MAX=400 (the dump needs more than 150 s after a 43 s bind), the gadget step lines copied into the stream as block gadget-steps, dmesg and udc before the clk/regulator/gpio reads (a wedged read cannot hide them), dbg-usb no longer a duplicate of usb. Not measured yet: the phone.

About
: walleye-first-fastboot-boot


<a id="walleye-vf4-tried-twice"></a>
### walleye-vf4-tried-twice

Date
: 2026-11-18

Brief
: vf4 twice from the first state (slot b), poller started first: no gadget ever seen, 0 blocks fetched, empty beat log; phone back as Android at 78 s both times

Question
: Does vf4 (vf2's init to the gadget bind, then the progressive dump) show the gadget, and what facts stream?

Found
: Both tries: battery 4377 mV, poller (POLL_MAX=400) started first, boot-vf4.img by devlab fastboot boot; left fastboot after 12 s, back as Android (adb) after 78 s. Poller fetched 0 blocks, no DONE, beat.log empty (0 bytes), no win copies, no steps.txt: no walleye-first-boot / Pixel 2 walleye (mainline) device ever on the host. Same as vf3 (78 s, no gadget), unlike vf2 (gadget at ~43 s). Flags not re-read after (phone left in Android on b; fastboot state was a/b successful before both). The 78 s return is the dead-kernel class. vf4 differs from vf2 only after the bind, yet no gadget: vf2's gadget may have depended on something other than the init (timing, or a first-boot-after-flash state) - not shown by these tries.

About
: walleye-first-fastboot-boot


<a id="walleye-vf-control-tried"></a>
### walleye-vf-control-tried

Date
: 2026-11-18

Brief
: vf control, no poller: vf2 gadget up twice (18-20 s, gone by 70-72 s, header empty or gone); vf4 dead twice (back as Android at 78 s, no gadget): the image, not the poller

Question
: Does vf2 repeat, and does vf4 behave like vf2 when watched the same way (no poller, no lsusb -v, LUN read only at 70 s)?

Found
: Four tries from slot b (a and b successful), no poller, plain lsusb 1 s, one LUN read at >=70 s. vf2 try1: gadget 1d6b:0104 at 20 s (counted from the boot command), gone before 70 s (read: no device), Android at 125 s. vf2 try2: gadget at 20 s, LUN /dev/sda readable at 70 s, header empty (no WALLEYE-LOG), gone at 72 s, Android at 127 s. vf4 tries 1 and 2: no gadget at all, Android back at 78-80 s (the dead path), nothing readable. So vf2 repeats (alive, gadget up ~50 s, window empty again); vf4 is dead unwatched too, though its init equals vf2's byte for byte up to the bind: the poller is cleared; what differs is vf4's later part (the progressive dump after the bind) or the image's size/layout. Note: tries 1 (vf2) and the vf2 try 1 first run (read missed by a loop timing slip, re-run as try 1b) began from fastboot after an earlier tool slip, not from Android; slot flags unchanged all along, phone back as Android on b.

About
: walleye-first-fastboot-boot


<a id="walleye-vfx-built"></a>
### walleye-vfx-built

Date
: 2026-11-18

Brief
: vfa..vfd built: m1's structure (alive) plus ONE source each put in its 2 MiB window before the bind; 23 checks each ok, rebuild identical, selftest 18 ok, reviewed

Question
: Which hardware source, read alone on m1's proven structure, works and which kills the vendor kernel (vf3/vf4's progressive dump died)?

Found
: build/initramfs-vendor5/init-vfx is m1's init line for line from the first command to the gadget bind (checked by the build), plus one block before the bind: the payload named by /etc/walleye-vf. vfa = /sys/firmware/fdt (binary, merged running tree) + /proc/iomem + /proc/interrupts + /proc/cmdline (redacted); vfb = debugfs clk_summary; vfc = debugfs regulator_summary; vfd = debugfs usb/ and the dwc3 directories. Each source is read by timeout 5 cat into the ramdisk, then put in the window file m1 makes (2 MiB): bytes 0-511 a plain header 'WALLEYE-VFx LEN= PARTS= TRUNC=' and a PART line per source (name, length, rc), the plain bytes from 512 (no tar, no gzip), a payload over the room cut with TRUNC=1; a first header with no part is written before the first read, so a kernel killed in a read shows as a header with PARTS=0. After the bind no payload write: only a 4 KiB beat block at the window's end every 5 s (m1 wrote 64 KiB every 5 s), 180 s then reboot -f. The four ramdisks differ in /etc/walleye-vf alone; header, kernel, busybox, file list and AVB footer equal m1's. Reader tests/host/window-payload-read.sh: one dd of the LUN by the header's length, parts cut out (fdt as fdt.dtb and fdt.dts through dtc), refuses an identifying string; selftest tests/host/window-payload-selftest.sh 18 ok (all four letters, truncation, missing source, killed first read, foreign window, redaction). sha256 (first 16) boot-vfa 2bb9f7214f2eef89, vfb fd90dd4d57665103, vfc dabf900a7f3bd962, vfd e8d4badc52e47908; full in ~/Projects/walleye-build/images-vfx/SHA256SUMS-vfx on the lab. Programmer review (no defect that stops a build or a good read) led to: a 200 s reboot watchdog started before anything is read (a hang before the bind used to end in no reboot), the usb dump to three levels and 64 KiB per file, the reader no longer lets one redacted MAC hide a leftover serial, a bounded LEN, stale files ignored, the fdt.dts (the serial sits in /chosen/bootargs) kept in the lab material directory and not judged. Not measured yet: the phone.

About
: walleye-first-fastboot-boot


<a id="walleye-vfx-tried"></a>
### walleye-vfx-tried

Date
: 2026-11-18

Brief
: vfa twice read (fdt, iomem, interrupts, cmdline); vfb alive but clk_summary rc=1; vfc read regulator_summary; vfd killed the init twice (no gadget)

Question
: Which single hardware source can the vendor-kernel init read into the window, and which kills it?

Found
: From the first state (slot b, a and b successful, battery 4.38 V), no poller, read at ~68 s and again near the end. vfa: gadget up at 18 s, header ours, 4 parts rc=0 (fdt 466569 B, iomem, interrupts, cmdline), both tries; the live fdt differs between tries only in 7 bytes of bootargs boottime; Android back at ~165-220 s. vfb: init alive (beat 11 at 52 s, 24 at 117 s), part clk_summary LEN=0 rc=1 - read failed, did not kill; one try. vfc: pass, regulator_summary 22515 B rc=0; one try. vfd (usb/dwc3 debugfs): no gadget ever, Android back at 87 s (try 1) and by ~136 s (try 2) - the dead path; killed twice. Flags and slot unchanged: b active, a and b successful. Material (never in git): ~/Projects/walleye-build/material/vf/2026-11-18-vfx-vf{a,b,c,d}-try*. Side finding: the fdt bootargs hold the phone's own serial unredacted, so the fdt stays in the material path only. Also: tests/host/battery-ok.sh hangs while the phone is in Android (fastboot getvar waits for a device); wrap it in timeout.

About
: walleye-first-fastboot-boot


<a id="walleye-fdt-compare"></a>
### walleye-fdt-compare

Date
: 2026-11-18

Brief
: Ours lacks, against both alive trees: restart@10ac000 (pshold), the apps watchdog, gpio-reserved-ranges; ramoops in the rtel region; the okay hardware

Question
: What does the phone's running fdt add to or need from a tree, and what do the alive taimen dtb and the running fdt have that our dtb lacks?

Found
: Three trees compared (ours 581 nodes, alive 953, running vendor tree 2284). The bootloader adds chosen bootargs/initrd, fills /memory, and rewrites reserved-memory (pil regions at other addresses than their names, bootloader log 0xa1300000, rtel 0xa1800000+32 MiB, ramoops 0xa1100000). Ours lacks restart@10ac000 qcom,pshold and the apps watchdog (in both alive trees and the running fdt), gpio-reserved-ranges <0 4>,<81 4> (both alive trees), has ramoops inside the bootloader's rtel region, and leaves display, gpu, ufshc, remoteprocs, serial and wifi disabled. 288 nodes only wpa has, 22 only ours, 45 differing and 126 only-wpa properties. The USB nodes of the running fdt equal what ours already carries; no new node is proposed.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-fdt-compare/


<a id="walleye-fdt-bisect-x-built"></a>
### walleye-fdt-bisect-x-built

Date
: 2026-11-18

Brief
: x1..x7 and xa built: wpa (alive) with one disjoint group of the differences to our dtb made equal to ours; 114 checks ok, rebuild identical

Question
: Which group of the differences between the alive dtb and ours makes init die?

Found
: build/build-fdt-bisect.sh builds boot-x1..x7 and boot-xa from boot-wpa.img and our dtb: x1 gpio-reserved-ranges, x2 chosen+reserved-memory, x3 cpu scaling, x4 restart+watchdog+imem+pmu, x5 interconnects, x6 statuses, x7 what x1..x6 leave, xa all. Only the dtb differs from wpa's image; the stock dtbo overlay still merges in each (overlay targets are emptied, not removed); the groups change disjoint lines (checked pairwise); xa differs from ours in exactly 82 pinned lines (ours-only regulators, stubs, phandle renumbering). Reviewed by the programmer subagent; its findings (x7 overlap, a hiding skip list, stale output on abort, phandle constants) fixed. Expected returns alive <= 66 s, dead >= 69 s; try order x4 x1 x2 x6 x5 x3 x7 xa, twice each.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-fdt-compare/


<a id="walleye-fdt-bisect-x-tried"></a>
### walleye-fdt-bisect-x-tried

Date
: 2026-11-18

Brief
: x1..x7, xa twice each: only x1 (gpio-reserved-ranges made ours) and xa are dead, both times; x2-x7 alive

Question
: Which disjoint group of dtb differences from the alive wpa to ours makes init die?

Found
: Returns (s), try 1/2: x4 61/64, x1 79/78, x2 43/43, x6 37/37, x5 64/64, x3 64/64, x7 64/49, xa 78/76. Alive <=66, dead >=69. x1 dead twice alone, so pinctrl@3400000 missing gpio-reserved-ranges <0 4>,<81 4> is a cause. xa dead as expected; other groups alone alive. Slot b, flags a and b successful before each; nothing flashed.

About
: walleye-first-fastboot-boot


<a id="walleye-y-built"></a>
### walleye-y-built

Date
: 2026-11-18

Brief
: y1 and y2 built: ml6a's kernel with OUR dtb now carrying gpio-reserved-ranges <0 4>, <81 4>; 49 checks ok, rebuild identical

Question
: Does our own dtb, given the property whose absence killed every dtb of ours, boot to init (y1) and to a USB gadget (y2)?

Found
: build/build-y.sh compiles dt/msm8998-google-walleye.dts (with the new &tlmm gpio-reserved-ranges = <0 4>, <81 4>) and puts it under ml6a's Image.gz with ml6a's header and command line (no earlycon). y1 carries ml5a's ramdisk (reboot at once); y2 pk5l's gadget init (ACM shell + mass_storage window). The dtb differs from ml6a's in that one property line only; the stock dtbo overlay merges into it; AVB footers verify; sizes are ml6a's. Expected: y1 alive (<= 66 s), y2 shows 1d6b:0104 on the host within ~25 s. Not measured yet: the phone; the cheap extra items of the fdt compare (restart@10ac000, the apps watchdog) are left out on purpose, the finding does not say they matter (x4 was alive), and one change keeps the effect attributable.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-y-built/


<a id="walleye-y-tried"></a>
### walleye-y-tried

Date
: 2026-11-18

Brief
: y1 twice alive (Android back at 36 s both); y2 twice: gadget 1d6b:0104 at 18 s, ttyACM0 appears; gpio-reserved-ranges in our dtb is enough

Question
: Does our own dtb with gpio-reserved-ranges <0 4>,<81 4> boot to init (y1) and to a USB gadget (y2)?

Found
: From slot b (a and b successful), battery 4379 mV. y1: left fastboot at 15 s, back as Android 36 s after the jump, both tries (alive class, <=66 s). y2: left fastboot at 15 s, 1d6b:0104 (serial walleye-first-boot, 'Pixel 2 walleye (mainline)') enumerated 18 s after the jump on both tries, /dev/ttyACM0 appeared on the host; back as Android after the init's 180 s reboot. tests/host/gadget-up.sh fails only at its ping step: y2 is ACM + mass_storage, no network function, so no ping is expected. Our own dtb now gives the first mainline USB gadget on walleye: the missing gpio-reserved-ranges property was the cause of every dead dtb of ours (x1/xa, d1..d6). Not read: ACM shell content, mass_storage window. Phone left Android on slot b, stock kernel, nothing flashed.

About
: walleye-first-fastboot-boot


<a id="2026-11-18-walleye-first-mainline-shell-map"></a>
### 2026-11-18-walleye-first-mainline-shell-map

Date
: 2026-11-18

Brief
: First mainline shell read: Linux 7.3.0 boots to init; USB udc, regulators, input, SPMI nodes present; no drm, no power_supply, arm-smmu 5040000 probe timeout

Question
: What does the dmesg of our own mainline kernel (y2) show works and what fails?

Found
: y2 booted from fastboot on slot b (nothing flashed), tty read by tests/host/mainline-shell-read.sh: dmesg 336 lines, kernel 7.3.0-dirty. Present: a800000.usb (udc), 3400000.pinctrl, 800f000.spmi, rpm regulators/clocks, ramoops, watchdog, qfprom, remoteproc nodes, event0 input, ~12 regulators. Absent: /sys/class/drm, power_supply (no charger/fuel gauge), debugfs devices_deferred (debugfs not mounted). Faults: psci PC mode -3 (firmware bug msg); spmi pm8998@0 and pmi8998@3 'does not have reg property' (-22), so PMIC children (charger, gpio, rtc) probably not bound; arm-smmu 5040000 deferred probe timeout -110; c16f000.serial waits on supplier /vendor-overlay-stubs/clock_gcc; OF: i2c@c179000 phandle not found. Next parts: PMIC reg/spmi nodes, then charger, display, UFS. Raw material in ~/lab-material/walleye/mainline-shell-20261118-082300 (redacted, not in git). Phone back via reboot -f, left in fastboot, slot b current, a and b successful.

About
: walleye-first-fastboot-boot


<a id="walleye-bringup-order-plan-made"></a>
### walleye-bringup-order-plan-made

Date
: 2026-11-18

Brief
: Walleye bring-up order: PMIC, clean probes, charger, UFS first; UFS does not wait on SPMI; fuel gauge, panel, DRV2624, TAS2557 are upstream candidates

Question
: In what order should the Pixel 2's parts come up on mainline, what does each unlock and prove, and what has mainline for these chips?

Found
: From the first mainline dmesg (y2) and the vendor tree's facts: the order is 1 PMIC on SPMI (the reg-property fault), 2 clean probes (arm-smmu timeout, serial supplier, i2c phandle), 3 charger (qcom_smbx is in mainline: a dts node and a monitored battery of 2700 mAh/4.4 V, no new driver), 4 UFS read-only (its rails L20, L26, S4 are RPM regulators already present, so it does not wait for SPMI), then USB networking, fuel gauge, touch, display, Wi-Fi/BT, audio and haptics; modem, sensors, camera, NFC, fingerprint and deep sleep get research items. Needing new drivers (upstream candidates): the S6E3FA5 panel, the PMI8998 fuel gauge (an RFC series of 2025-11-24, to test and review), DRV2624 haptics, TAS2557 amps. Has mainline: smb2 charger, rradc, UFS, DPU/DSI, RMI4 touch, WCD934x, ath10k WCN3990. Each part has its proof in the material's table; the 21 docs components map to three phases (reachable, usable, a phone); the jack is absent by hardware. Not known yet: UFS vs smmu order, the MP1 touch wiring.

Sources
: lwn-pmi8998-fg-rfc,kernel-qcom-smbx-charger,linux-msm-mainline-status

About
: walleye-first-fastboot-boot

Material
: research/walleye-bringup-order-plan/2026-11-18/


<a id="walleye-z1-built"></a>
### walleye-z1-built

Date
: 2026-11-18

Brief
: z1 built: y2 with only the dtb replaced by ours with mainline's pm8998/pmi8998 SPMI nodes; 16 checks ok, rebuild identical; the two reg-less nodes come from the dtbo overlay

Question
: How does our dtb give the SPMI bus pm8998 and pmi8998 with their children, and what is z1?

Found
: The 'does not have reg property' of the first dmesg is not our dtb: the stock dtbo overlay (idx 3, merged by the bootloader) adds qcom,pm8998@0 and qcom,pmi8998@3 under spmi@800f000 in downstream style (no reg, no compatible), while our dtb had no PMIC node at all. Mainline's OnePlus 5(T), Sony Yoshino and Mi 6 boards all include pm8998.dtsi (pmic@0, pmic@1) and pmi8998.dtsi (pmic@2, pmic@3), reg <usid SPMI_USID>; dt/msm8998-google-walleye.dts now does the same, two lines. Five overlay stub labels (pm8998_adc_tm, pm8998_gpios, pmi8998_charger, pmi8998_gpios, pmi8998_wled) are now real nodes and left the stubs (125 stubs, 12 labels ours); the overlay still merges into the new dtb (fdtoverlay pass). build/build-z.sh: y2's Image.gz, ramdisk (gadget init), header, command line and AVB footer byte for byte, only the dtb replaced; 16 checks ok, rebuild byte-identical. boot-z1.img sha256 4abbac7db9047103b93e30c79ab840c8f1772479c0ed230b7d687c259954fa2a, dtb 33fb9762d0d6c381b9a5b399ef8bdc1faf7ae20b66cd4103e96ca1b0a3895c6b (lab: ~/Projects/walleye-build/images-z). The two overlay nodes keep printing reg-less lines (no board file can remove them); tests/host/pmic-dmesg-check.sh tolerates exactly those two and demands spmi devices 0-00 0-01 0-02 0-03 registered and no probe of them failed (selftest 12 ok; a programmer review tightened it). Not known yet: whether the PMIC children bind on the phone; what the bootloader's overlay merge adds inside the live mainline PMIC nodes (the booted dtb differs from the built one); whether charger and fuel gauge defer for want of more nodes.

About
: walleye-first-fastboot-boot


<a id="walleye-z1-tried-reader-denied"></a>
### walleye-z1-tried-reader-denied

Date
: 2026-11-18

Brief
: z1 once from slot b: gadget 1d6b:0104 and ttyACM0 up at 18 s, but the reader got Permission denied on /dev/ttyACM0; no dmesg read, PMIC result unknown

Question
: Does z1 (our dtb with mainline pm8998/pmi8998 SPMI nodes) show the PMIC devices and no failed probe?

Found
: Battery 4376 mV (pass). devlab fastboot boot z1: left fastboot at 15 s, 1d6b:0104 'Pixel 2 walleye (mainline)' at 18 s, cdc_acm bound ttyACM0 (host log 08:36:39), gadget gone 08:39:35, phone back in Android (adb up), slot b, nothing flashed. mainline-shell-read.sh: Permission denied on /dev/ttyACM0; every answer empty (outputs under ~/lab-material/walleye/z1-20261118-083622, not in git); pmic-dmesg-check.sh says skip: no dmesg. So the kernel with z1's dtb is alive, but the PMIC question is unanswered. The cause (tty access from this unattended session; the uaccess tag of 70-lpoco-google-walleye.rules reaches only a seat user) must be fixed before the next z1 try.

About
: walleye-first-fastboot-boot


<a id="walleye-smmu-timeout-gpucc-module"></a>
### walleye-smmu-timeout-gpucc-module

Date
: 2026-11-18

Brief
: arm-smmu 5040000 times out because gpucc is =m in a kernel whose initramfs has no modules

Question
: Why does arm-smmu 5040000 fail with -110 on our mainline kernel?

Found
: Mainline msm8998.dtsi gives adreno_smmu (iommu@5040000) power-domains = gpucc GPU_GX_GDSC; gpucc is node clock-controller@5065000. The kernel's .config has CONFIG_MSM_GPUCC_8998=m (Kconfig: default m), our initramfs carries no modules, so no driver ever binds gpucc, the GDSC never registers, and the smmu defers until the 11 s deferred-probe timeout (-110). The gcc line 'sync_state() pending due to 5040000.iommu and 5065000.clock-controller' in the first dmesg says the same. Not yet measured on the phone: image z2 (gpucc =y) is built to test it. The two other faults (serial@c16f000 waiting on the overlay stub clock_gcc, the overlay's four phandle lines) belong to nodes the stock dtbo overlay adds, not to our dts; they are the next images.

Sources
: research 2026-11-18-walleye-first-mainline-shell-map; linux v7.3 arch/arm64/boot/dts/qcom/msm8998.dtsi and drivers/clk/qcom/Kconfig (MSM_GPUCC_8998)

About
: walleye-first-fastboot-boot


<a id="walleye-z1-tried-silent-tty"></a>
### walleye-z1-tried-silent-tty

Date
: 2026-11-18

Brief
: z1 twice from slot b: gadget 1d6b:0104 + ttyACM0 up at 18 s, tty readable (ACL held 100 s), but the shell never answered; phone fell back at ~3 min

Question
: Does z1 (our dtb + mainline PMIC nodes) show the SPMI PMICs registered?

Found
: Try 1 (reader started first): gadget at 18 s, 7 of 16 answers empty-file, dmesg empty; reader's last write got Permission denied after the phone dropped the tty. Try 2 (ACL watcher, then reader at ~110 s): getfacl showed user:mll:rw- on /dev/ttyACM0 from first appearance to 100 s, open succeeded each 2 s; reader had the tty writable yet all 16 answers were empty; tty vanished ~09:25:39, Android back on slot b by ~09:27 (a,b successful). So the earlier EACCES was the tty vanishing, not a missing ACL; the real problem is no shell answering on ttyACM0 (the ACM function opens but the init's shell is not reading it, or the kernel dies about 3 min in). PMIC result still unknown. The uaccess rule and the retry are fine; do not chase ACLs again.

About
: walleye-first-fastboot-boot


<a id="walleye-y2-z1-control-both-silent"></a>
### walleye-y2-z1-control-both-silent

Date
: 2026-11-18

Brief
: Control: y2 and z1 from slot b, same reader, same day: both enumerate at 18 s, ttyACM0 appears, reader gets 0 bytes from either; z1's silence is not the new dtb

Question
: Is z1's silent ttyACM0 the new dtb (PMIC nodes) or the reader?

Found
: y2 once (09:29) and z1 once (09:33), battery 100%, slot b, reader mainline-shell-read.sh started before each boot. Both: left fastboot at 15 s, 1d6b:0104 at 18 s (CDC ACM, CDC data, mass storage; product 'Pixel 2 walleye (mainline)'), ttyACM0 created, every command unanswered (dmesg/uname/cmdline files empty), then Permission denied on the tty near the end as the phone left. Both phones back as Android on slot b, flags a/b successful, nothing flashed. So the PMIC dtb does not silence the shell; the reader or the init's tty does. Not measured: raw bytes (the reader does not log them), host dmesg lines. Files on the lab host: ~/lab-material/walleye/y2ctl-092945 and z1ctl-093341.

About
: walleye-first-fastboot-boot


<a id="walleye-clock-gcc-stub-no-cells"></a>
### walleye-clock-gcc-stub-no-cells

Date
: 2026-11-18

Brief
: The serial wait and the four 'could not find phandle' lines have one cause: the clock_gcc stub has no #clock-cells, so the kernel reads each clock ID as a phandle

Question
: Why does c16f000.serial wait on /vendor-overlay-stubs/clock_gcc, and what are the four OF: could not find phandle lines of the first mainline dmesg?

Found
: Raw dmesg (lab material mainline-shell-20261118-082300, lines 169-173): phandle numbers -1934992561 (i2c@c179000), -1480237014 (bt_wcn3990), -943313008 (serial@c16f000), -130129017 (mnh-sm) are the vendor clock ID words (0x8caa5b4f ...) of 'clocks = <&clock_gcc ID ...>' from the stock dtbo overlay entry 3 (its __fixups__ clock_gcc names those four users). Our stub clock_gcc is an empty node: no #clock-cells, so the kernel takes the ID cell as a phandle. The serial's supplier is the stub, which never probes. Pointing /__symbols__/clock_gcc at /soc@0/clock-controller@100000 (gcc, #clock-cells 1) makes the merge write gcc's phandle (37) into all four; tests/host/overlay-clock-gcc.sh was seen failing on z2's dtb (306, the stub) and passes on z3's.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-clock-gcc-stub/


<a id="walleye-y2-old-reader-tried"></a>
### walleye-y2-old-reader-tried

Date
: 2026-11-18

Brief
: y2 with the old reader (733052d): 0 answers, but the phone's echo of one command reached the host, so the host reads; the shell does not answer

Question
: y2 with PR 155's exact reader: is the silence the reader change or the host?

Found
: Battery 4376 mV. adb reboot bootloader (devlab fastboot reboot --bootloader refused from Android), old reader started first, devlab fastboot boot y2 (sha 0579fcf4...). Left fastboot 15 s, back as 1d6b:0104 after 18 s, /dev/ttyACM0 up (ACL user:mll:rw). Old reader: every answer empty, no 'Linux version'; only devices-deferred.txt held 171 bytes, the echoed command line (tty echo, not output). So bytes do travel phone->host; the reader change is not the cause (diff 733052d..main: one added ask line); the host is not deaf; the y2 init's shell does not answer, or its output path is dead. Host: stty raw -echo 115200, fuser only the reader, ModemManager inactive, kernel log only enumeration + cdc_acm ttyACM0. The reader's last write got Permission denied (node re-created after the phone's reboot, before the ACL applied: see walleye-reader-eacces-retry). Android back on slot b, adb up, nothing flashed.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-y2-old-reader/


<a id="walleye-c1-built"></a>
### walleye-c1-built

Date
: 2026-11-18

Brief
: c1 built: z3 plus the SMB2 charger and RRADC built in and pmi8998's charger enabled with a 2700 mAh / 4.4 V simple-battery; 27 checks ok, rebuild identical

Question
: Can mainline's pmi8998 charger be switched on for walleye with no new driver, and what is the image?

Found
: Mainline v7.3 has qcom_smbx (CHARGER_QCOM_SMB2) and the rradc driver (QCOM_SPMI_RRADC), both off in defconfig; pmi8998.dtsi has the charger (disabled) and rradc nodes with the interrupts and io-channels. The driver refuses to probe without a monitored-battery and sets the float voltage from voltage-max-design, so the battery node carries the vendor GPL tree's cell facts: 2700 mAh, 4.4 V max, 3.4 V min, 50 mA termination; JEITA stays the PMIC's own. c1 = z3's ramdisk, header, command line and AVB footer with the new kernel (the two lines built in; config differs from z2's in those only) and our dtb (charger okay, battery, clock_gcc symbol on the real gcc as z3). Image ~/Projects/walleye-build/images-c1/boot-c1.img sha256 ddae19b523f2a68d902755f0e57d53c3b98c74b9b146d372bc40ea307aaee0c0, dtb dfb66904...06e3ea. Checks: build/build-c1.sh 27 ok, rebuild identical; tests/host/charger-selftest.sh 8 ok. Not measured: whether the charger probes or the cable shows online: that is the try.

Sources
: kernel-qcom-smbx-charger

About
: walleye-charger-smb2


<a id="walleye-banner-built"></a>
### walleye-banner-built

Date
: 2026-11-18

Brief
: banner built: y2 with only the ramdisk changed, an init that writes a banner to ttyGS0 every 2 s and a kmsg marker per step; 40 checks ok, rebuild identical, reviewed

Question
: Can y2's kernel and dtb carry an init whose only job is to be heard on ttyGS0, so a plain cat tells which side is silent?

Found
: images-banner/boot-banner.img sha256 3d9320f01567a9824f1371a218fa397cbedc4eb996e9409b613a458878a4a081; initramfs-banner.cpio.xz sha256 d73a17d74e1163613006aa4a0318f7e2d34db445adb9812cf9bf05f8990669ac. Kernel, dtb, header, command line, AVB footer are y2's (checked byte for byte); the ramdisk is the one change. init-banner: one acm.GS0 function, a writer subshell started before the bind that writes WALLEYE-BANNER beat=N up= steps= last=[..] every ~2 s once /dev/ttyGS0 exists, a kmsg line per step before the step, a watchdog petter subshell that reboots -f after ~180 s. The programmer review found a failed exec redirect could end init, no pet during the UDC wait, the writer started after the bind, wrong redirect order, config check accepting =m: all fixed before the build. Host check: tests/host/banner-check.sh CAPTURE (3+ banner lines with rising beats), banner-selftest.sh 7 ok. Not run on the phone.

Sources
: none

About
: walleye-first-fastboot-boot


<a id="2026-11-18-walleye-banner-seen-needs-root-on-ttyacm"></a>
### 2026-11-18-walleye-banner-seen-needs-root-on-ttyacm

Date
: 2026-11-18

Brief
: Banner image tried once from b: banner bytes ARE on the host; plain-user cat of /dev/ttyACM0 is denied (root:uucp 0660, mll not in uucp)

Question
: Banner once from slot b, plain cat of ttyACM0: are the banner bytes on the host?

Found
: boot-banner.img (sha 3d9320f0...) booted from fastboot at 10:06:35; gadget 1d6b:0104 at 18 s; ModemManager inactive; /dev/ttyACM0 crw-rw---- root:uucp, user mll groups lack uucp, so stty and cat as mll fail Permission denied (zero bytes). Read as root via lab-agent root-exec for 15 s: 7 WALLEYE-BANNER lines, beats 11..17 at up 53.9..66s, steps=8 last=[step 8 gadget bound to a800000.usb]; banner-check.sh pass. Phone reset itself ~180 s, back as Android on slot b, adb up. So the earlier empty reads were most likely the host permission (not the phone silent); kernel and gadget run 65 s+. Nothing flashed, host unchanged.

About
: walleye-first-fastboot-boot


<a id="walleye-banner2-built"></a>
### walleye-banner2-built

Date
: 2026-11-18

Brief
: banner2 built: boot-banner.img with only init-banner's USB serial changed to walleye-first-boot; 41 checks ok, rebuild identical, reviewed

Question
: Can the banner image carry the serial the host's udev rule knows, with nothing else changed?

Found
: images-banner2/boot-banner2.img sha256 680c0aa730d294e3d9c23a23c3abc4db993cfaff059e805229e4af6244ce9888 (initramfs-banner2.cpio.xz 502b1142f58bf022d24cd385acaa07f4c2bae0282e5920173ad818eaa24c9dcb). Kernel bytes equal boot-banner.img's; the ramdisk differs in one init line (serialnumber). build/check-init-serials.sh reads every init under build/initramfs*/init* (23 serial lines, all walleye-first-boot) and fails on another serial or a serialnumber line of a form it cannot read; build-banner.sh runs it (BANNER_NAME names outputs, SHA256SUMS-NAME). Seen failing first with the old serial. Programmer review applied: no silently skipped forms, plain initramfs/ included, name validated.

About
: walleye-first-fastboot-boot


<a id="walleye-banner2-tried"></a>
### walleye-banner2-tried

Date
: 2026-11-18

Brief
: banner2 once from slot b: lab-user cat of ttyACM0 got 29 banner lines, beats increasing, init alive to beat 31 (64 s); banner-check passes

Question
: Do banner bytes reach a plain lab-user cat of /dev/ttyACM0 with banner2?

Found
: Yes. fastboot boot of banner2 from slot b: left fastboot 15 s, 1d6b:0104 at 18 s, ttyACM0 root:users 0660 (uaccess ACL), cat as lab user read 2805 bytes, 29 WALLEYE-BANNER lines, beat=31 up=63.88s step 8 gadget bound to a800000.usb. Serial walleye-first-boot fixes the permission. Phone fell back to Android (adb up) by ~4 min; nothing flashed. The reader channel now works, so the z3 and c1 tries can use it.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18/


<a id="walleye-z3-try1-reader-job-not-picked-up"></a>
### walleye-z3-try1-reader-job-not-picked-up

Date
: 2026-11-18

Brief
: z3 once from slot b: gadget 1d6b:0104 at 18 s, back as Android; the reader job was never picked up by lab-agent in 600 s, so no dmesg: not a z3 result

Question
: Is z3's dmesg clean (reader as a lab job, per decision 2026-11-19-walleye-reader-runs-as-a-lab-job)?

Found
: battery-ok pass 4385 mV; z3 sha256 verified; submitted job 1794997309-walleye-z3-read1 BEFORE the boot; fastboot boot now: left fastboot 15 s, gadget 1d6b:0104 at 18 s; lab-agent wait: 'not picked up within 600s - is the agent running?' (service active since 11-17 22:18, but jobs/logs --remote show the job nowhere). The 17 zero-byte files in ~/lab-material/walleye/z3-try1 are unexplained (mtime 11:12, dmesg 0 lines): not a reading. Phone back as Android (adb) on slot b, nothing flashed. Nothing is known yet about z3's dmesg.

About
: walleye-first-fastboot-boot


<a id="walleye-z3-ttyacm-not-a-permission-problem"></a>
### walleye-z3-ttyacm-not-a-permission-problem

Date
: 2026-11-18

Brief
: z3 try 3: ttyACM0 was readable (root:users 0660 + uaccess ACL for mll, open ok each second); reader got silence for 3 min; 'Permission denied' came at the end, from the vanished node

Question
: Why did the shell reader fail on z3 with 'Permission denied' at line 50 - the node's owner, mode or ACL?

Found
: Third boot of boot-z3 (sha 9bc4ec19..) from slot b at 10:44Z, battery 4377 mV, slots b/a successful before; 1d6b:0104 at 18 s; back to Android (adb) at ~10:47:12Z, nothing flashed. Coordinator's watcher (z3-ttywatch-20261118-1042/watch.log, logs changes only): /dev/ttyACM0 first seen 10:44:15Z as crw-rw----+ root:users, ACL user:mll:rw-, open test (exec 3<>) ok on every line to the last at 10:47:12Z; udev: serial walleye-first-boot, 1d6b:0104. The reader (path used: plain) made its OUT dir at 10:44:16Z, ran each ask to its timeout, saved 16 of 17 answers EMPTY (devices-deferred holds only our own command echo); dmesg.txt empty. At 10:47:12Z the node vanished with the phone's return; the final twrite 'reboot -f' did '> /dev/ttyACM0' on a missing node and the shell printed 'Permission denied' (mll cannot create files in /dev; 2>/dev/null comes after the redirect, so the error shows). So: not a permission problem; no udev rule or group needed. The real question is why the phone answers nothing on the ACM: z3's init may run no shell on ttyGS0, or on another tty. dmesg-clean-check: skip, no dmesg.

About
: walleye-first-fastboot-boot


<a id="walleye-z4-tried-once"></a>
### walleye-z4-tried-once

Date
: 2026-11-18

Brief
: z4 once from slot b: gadget 1d6b:0104 at 18 s, one-fd reader held the port, but no shell answer (dmesg empty, 16 of 17 answers empty); phone back as Android

Question
: Does z4's ACM shell answer, with the port held open on one fd, and is the dmesg clean?

Found
: 2026-11-18 11:50Z: battery 4377 mV pass, boot-z4.img sha256 4115db3e verified, reader started first as a plain background process, slots a/b successful before; fastboot boot now left fastboot at 15 s, 1d6b:0104 at 18 s; ttyACM0 opened by the reader (path plain) but every ask returned empty (only devices-deferred kept its own command echo, 169 bytes); dmesg.txt 0 bytes, no 'Linux version'; later writes failed with Input/output error (the port went away as the phone returned). Phone back as Android (adb) on slot b by 11:53:33Z, nothing flashed. Not shown: whether init-z4's console block ran (no kmsg line read; pstore not readable by the shell user). z4 does not fix the silent shell. Proof walleye-dmesg-clean NOT passed; item stays open. Raw output: ~/lab-material/walleye/z4-try1 (not in git).

About
: walleye-first-fastboot-boot


<a id="walleye-z5-built"></a>
### walleye-z5-built

Date
: 2026-11-18

Brief
: z5 built: z4 with only the init changed to banner2's banner + z4's console + the shell's state in the banner; 49 checks ok, rebuild identical, reviewed

Question
: Can the proven banner channel report why z4's ACM shell is silent?

Found
: boot-z5.img sha256 fee768d62219fd08ce1e90618ea5a7977dd3ca1c762b6a4563f9ca92dfcc8dd9 (images-z5 on the lab). init-z5 = init-banner + z4's console block; banner line ends shell=[started n pid alive rc probe err], WALLEYE-BANNER-INFO every 10th beat from beat 3 (busybox applets, ash/sh/cttyhack/setsid/stty, tty echo/icanon/hupcl). Host checks: banner-shell-check.sh (pass = last line live shell, start count steady over 3 lines), banner-shell-selftest.sh, z5-console-selftest.sh (busybox ash, /dev/null and a pty). Programmer review found: respawn throttle was 0.2 s not 20 s (fixed), banner lines glued to the prompt (each line now starts with a newline), pass could not see a flapping shell (fixed), selftest process leak (fixed). Not fixed: the kept fd 4 means lines before the host opens the port queue and arrive stale (unlike banner2); applets are checked from the build .config, not the packed binary. Deviation from the item: the info line repeats every 10th beat so a late reader sees it.

About
: walleye-first-fastboot-boot


<a id="walleye-z5-tried"></a>
### walleye-z5-tried

Date
: 2026-11-18

Brief
: z5 once from slot b: banner says shell started, alive, probe=0, no error; the tty is full of '> ' prompts and 'ash: /: Permission denied' lines

Question
: Why is the z5 shell silent: what does a plain 60 s cat of the banner say?

Found
: boot-z5.img (sha fee768d6...) via lpoco lab fastboot boot now from slot b, battery 4376 mV: gadget 1d6b:0104 at 18 s, ttyACM0 readable by the lab user; banner-shell-check passes, last line beat=31 up=64.48s steps=8 shell=[started=yes n=1 pid=153 alive=yes rc=- probe=0 err=[none]]. The 60 s capture is 1.38 MB, 192k lines; besides banner lines: '> ' continuation prompts (79k), '~ # ' (11k), 'ash: /: Permission denied' (6k), 'ash: ash:: not found' (3.8k). Measured: the shell runs and writes prompts and errors to the tty. Hypothesis, not measured: the tty echoes the banner into the shell, which runs its own lines as commands. Head of the capture: z5-capture-head.txt in the material folder. Phone back as Android on slot b, stock kernel, nothing flashed.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18/


<a id="walleye-z5b-try-no-reader"></a>
### walleye-z5b-try-no-reader

Date
: 2026-11-18

Brief
: z5b once from slot b: gadget 1d6b:0104 up at 18 s, but the reader job was never run (lab-agent polls 401), so no tty result

Question
: Is the shell drowned by the host's echo of the banner (z5 with the host tty raw/-echo from first appearance)?

Found
: boot-z5.img (sha fee768d6...) booted via lpoco lab fastboot boot now from slot b, battery 4376 mV; gadget 1d6b:0104 at 18 s; phone came back as Android on b (adb), nothing flashed. The reader was submitted first as lab-agent job 1795004341-job (stty raw -echo -hupcl on appearance, 30 s cat, then mainline-shell-read.sh), but lab-agent's journal shows 'poll error: listing grenudi/lab-agent-jobs ... status code 401' every 5 s: the job was never picked up (lab-agent jobs does not list it). So NOT a z5b result: (a) (b) (c) unanswered. The keeper's own sandbox denies the tty open (decision 2026-11-19-walleye-reader-runs-as-a-lab-job), so there is no other sanctioned route. Needs the author's hands: renew lab-agent's GitHub token (lab-agent connect).

About
: walleye-first-fastboot-boot


<a id="walleye-z5b-tried-raw-tty"></a>
### walleye-z5b-tried-raw-tty

Date
: 2026-11-18

Brief
: z5b once from slot b, host tty raw -echo: 0 '> ' and 0 'Permission denied' in a 30 s banner capture, but the shell still answers no command

Question
: Is the shell drowned by the host's echo of the banner (z5 had '> ' prompts and 'ash: /: Permission denied' lines)?

Found
: stty raw -echo -hupcl succeeded at the first poll after the node appeared (159 polls of 0.1 s); banner capture 2332 bytes, 32 lines, 0 lines '> ', 0 'Permission denied'; banner shows tty=[echo=off,icanon=off,hupcl=off], shell started alive pid=152 probe=0 err none. So the host's echo explains the prompt/error noise. But the one-fd reader got 13 non-empty answers that are all banner beats, never a command's output (no 'Linux version'; uname answer is banner lines); one '> ' prompt appeared once the reader began writing. The banner written to ttyGS0 every 2 s shares the line with any answer, and the shell still gives no command output. Next: z6 stops the banner beats once the shell is up (or sends them to kmsg only) so the shell's answers are alone on ttyGS0. Phone back as Android on slot b (flags a and b successful, none unbootable), nothing flashed.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-z5b-try2/


<a id="walleye-z6-built"></a>
### walleye-z6-built

Date
: 2026-11-18

Brief
: z6 built: z5 with the console block replaced by a plain read/exec command loop on ttyGS0; checks ok, rebuild identical, pty selftest measures a line in gets an answer out

Question
: Can the silent interactive ash be replaced by a loop that needs no job control, no prompt and no controlling terminal, and is that shown before the phone?

Found
: init-z6 = init-z5 with only the console block replaced: fd 4 opened once on ttyGS0 (raw -echo -hupcl), then read a line, run it with sh -c (stdin /dev/null, output to /tmp/out), cat /tmp/out to fd 4; no end marker of its own - the host reader's command carries its own (CMD; echo wmsr-PID-N-END), so tests/host/mainline-shell-read.sh works with one change: banner lines (WALLEYE-BANNER) are left out of each answer. The banner keeps running and ends with loop=[started starts cmds last rc err]; a running command shows rc=running. tests/host/z6-loop-selftest.sh runs the loop under busybox ash on a real pty (script): 7 commands, answers, stderr, CRLF, marker order, rc and last command in the state; 13 of its checks fail on init-z5. banner-loop-check.sh and its selftest judge a capture. build-z6.sh: checks ok incl. the busybox options the loop uses, rebuild identical. Programmer review: no blocking bug; fixed: state before the command runs, selftest under busybox ash, config checks. Not fixed, known: a banner line can land inside an answer over 4 KB (reader drops it); a lost tty is noticed after ~10 s; each line is its own sh -c. Hashes: 71e240da2c5093092907ffd94bd831045a85b08dac33de25e67ff65566d34099  initramfs-z6.cpio.xz;7f8af938a4a5c07197f81d306e4f7122f358a6fd657db6b53898bd8bbe94a97a  boot-z6.img;

About
: walleye-first-fastboot-boot


<a id="walleye-z6-tried"></a>
### walleye-z6-tried

Date
: 2026-11-18

Brief
: z6 once from slot b: the command loop answers - dmesg in full (590 lines, Linux 7.3.0), 15 answers; banner says loop started=yes; SPMI 0-00..0-03 present

Question
: Do commands answer on z6's plain read/exec loop, and what does the first full dmesg say (PMIC)?

Found
: z6 (boot-z6.img sha256 7f8af938...) from slot b, host stty loop raw -echo -hupcl, plain background reader (decision 2026-11-19-walleye-reader-plain-background-process): gadget 1d6b:0104 at 18 s, node after 19 s; banner capture 20 s passes banner-loop-check (loop=[started=yes starts=1 cmds=1 rc=127], start count steady); the reader passed: 15 answers, dmesg 590 lines starting Linux version 7.3.0-dirty. FIRST working mainline shell on walleye. Facts: /sys/bus/spmi/devices 0-00 0-01 0-02 0-03 (pm8998, pmi8998) present; pmic-dmesg-check passes (only the overlay's 2 reg-less nodes noted); no drm, no power_supply class, no devices_deferred (no debugfs); regulators 0-11 and input0 present; udc a800000.usb; arm-smmu 5040000.iommu line in the platform list; 14 dmesg lines match deferred/fail/unknown-symbol patterns (to read next); psci Firmware Bug 'failed to set PC mode: -3'. Battery 4376 mV before, 4381 mV after. Phone back as Android on slot b by itself after the hold (adb up), then adb reboot bootloader showed flags a and b successful, none unbootable, current-slot b; left via fastboot reboot to Android. Nothing flashed.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-19-z6-try1/


<a id="walleye-z6-dmesg-mapped"></a>
### walleye-z6-dmesg-mapped

Date
: 2026-11-18

Brief
: z6 dmesg read in full: five distinct faults (gpu_gx GDSC stuck via adreno_smmu, vendor-overlay clock ids, psci, misaligned image, reg-less nodes); PMIC, RPM, USB, wdt, ramoops work

Question
: What does every warning, deferral and failure in the z6 dmesg (590 lines) mean, and in what order do we fix them?

Found
: Full map in the material (map.md). Works by lines: SPMI arbiter v3 with 0-00..0-03, RPM clock/power/regulators-0/-1 (12 regulators), USB udc and gadget bound at 1.17 s, kpss watchdog, ramoops/pstore, gpio_keys, three arm-smmu (1680000, 16c0000, cd00000) probe fine.
Faults: G1 gpu_gx GDSC "status stuck at 'off'" (six WARNs): 5040000.iommu (adreno_smmu) lists power-domains GPU_GX_GDSC, whose collapse switch needs the GFX rail nothing in our dts powers (which rail: not known yet); the smmu defers for ever ("reason unknown") and holds gcc and gpucc sync_state; the old backlog line "arm-smmu 5040000 timeout" is this. Smallest fix: our dts disables adreno_smmu until GPU work. G2 serial@c16f000 probe error -2, "qcom_cc_clk_hw_get: invalid index": a node of the vendor dtbo overlay whose clock ids are the vendor's; with c179000.i2c, bt_wcn3990 and mnh-sm it keeps gcc sync_state pending; we cannot edit the overlay, harmless with clk_ignore_unused; gone only if the bootloader merge is avoided (item walleye-dtbo-overlay-id). G3 psci "failed to set PC mode: -3": firmware does not support it, kernel falls back, harmless. G4 "Kernel image misaligned at boot": bootloader loads Image off a 2 MiB boundary, harmless. G5 two reg-less pm8998@0/pmi8998@3 nodes from the overlay; ours bind.
Not in the dmesg because not in the dtb or config yet: UFS (no node, no driver), charger (z6's dtb is z3's; main's dts has it, c1 carries it), display, touch, sensors, fuel gauge, remoteprocs. debugfs is not mounted so devices_deferred was unreadable.

Sources
: linux-gpucc-msm8998-gx

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-z6-dmesg-map/


<a id="walleye-z7-built"></a>
### walleye-z7-built

Date
: 2026-11-18

Brief
: z7 built: z6 with only the dtb changed, adreno_smmu disabled; 13 checks ok, rebuild identical, proof test seen failing on z6's dtb first

Question
: Can the GPU smmu's gpu_gx GDSC fault (G1 of the z6 dmesg map) be removed by one dtb change, shown before the phone?

Found
: images-z7/boot-z7.img (sha256 2dc6ff0e13dc042086ee2f46be91afc35944ae87c4a242e36cda28934d107b30) = z6 with only /soc@0/iommu@5040000 given status disabled (fdtput on z6's dtb, which is z3's, so a dtb of an older dts); Image.gz, ramdisk (init-z6), header, command line and AVB footer are z6's byte for byte; the dtbo overlay still merges. dt/msm8998-google-walleye.dts carries the same line (&adreno_smmu disabled) for later builds. tests/host/adreno-smmu-disabled.sh fails on z6's dtb and passes on z7's; dmesg-clean-check.sh now also counts gpu_gx stuck WARNs and tolerates only the known overlay deferrals. Not measured on the phone yet.

Sources
: linux-gpucc-msm8998-gx

About
: walleye-first-fastboot-boot


<a id="walleye-z7-tried"></a>
### walleye-z7-tried

Date
: 2026-11-18

Brief
: z7 once from slot b: dmesg-clean-check passes - no gpu_gx WARN, no smmu deferral, no phandle line; dmesg 323 lines (z6: 590)

Question
: Is the z7 dmesg (adreno_smmu disabled) free of the gpu_gx WARN and the 5040000 smmu deferral?

Found
: z7 (boot-z7.img sha256 2dc6ff0e...) once from slot b, z6 recipe (host stty loop, banner capture 20 s, plain background reader): gadget 1d6b:0104 at 18 s; banner-loop-check passes (loop started=yes starts=1); reader 15 answers, dmesg 323 lines; dmesg-clean-check pass (devices_deferred only known overlay nodes, no gpu_gx WARN, no -110, no phandle line); pmic-dmesg-check pass (spmi 0-00..0-03). Still: psci 'failed to set PC mode: -3', misaligned image Firmware Bug; no drm, no power_supply class (no debugfs, so devices_deferred was the reader's can't-open fallback). Battery 4376 mV before, 4385 mV after. Phone back as Android on slot b by itself; adb reboot bootloader showed flags a and b successful, none unbootable; left via fastboot reboot. Nothing flashed.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-19-z7-try1/


<a id="walleye-c2-built"></a>
### walleye-c2-built

Date
: 2026-11-18

Brief
: c2 built: z7 with the charger kernel (SMB2 charger, rradc built in) and the dtb with charger + 2700 mAh/4.4 V battery nodes; 29 checks ok, rebuild identical

Question
: Can the charger work of c1 be put on the z7 base, so its try is read through the command loop?

Found
: build/build-c2.sh takes z7's image (z6 command-loop ramdisk, header, cmdline, AVB footer) and changes only the kernel Image.gz (c1's: CHARGER_QCOM_SMB2=y, QCOM_SPMI_RRADC=y, config otherwise z2's) and the dtb (current dts: adreno_smmu off, charger okay, simple-battery 2700 mAh / 3.4-4.4 V / 50 mA, clock_gcc symbol on the real gcc). A check counts that the dtb differs from z7's only by the charger status, monitored-battery and the battery node. Failing first on a kernel dir without the charger (z2's out). boot-c2.img sha256 262d6a2c69658ff9167e710a61adc624a5aec7752a8b46ad68a1af6417d6d86c; dtb 40d19254033856de5af6c2289e2c927b9139baec6eafa4769e828cb9d55ed39e; rebuilt byte for byte; AVB verifies.

About
: walleye-first-fastboot-boot


<a id="walleye-c2-tried"></a>
### walleye-c2-tried

Date
: 2026-11-18

Brief
: c2 once from slot b: command loop up, dmesg 324 lines; charger@1000 deferred - supplier tps61099 has no driver; no power_supply class; charger-check fails

Question
: Does pmi8998's charger probe on c2, and is online 1 with the cable in?

Found
: c2 (boot-c2.img sha256 262d6a2c...) once from slot b via fastboot boot, z7 recipe (host stty loop, 20 s banner capture, plain background reader): gadget 1d6b:0104 at 18 s, banner-loop-check passes (starts=1), reader 15 answers. charger-check.sh FAILS: no power_supply class at all. dmesg: 'platform 800f000.spmi:pmic@2:charger@1000: deferred probe pending: platform: wait for supplier /soc@0/tps61099/ti,tps61099' - the dtb's supply link to the TPS61099 boost node (added by the stock overlay) has no driver, so the SMB2 driver never probes; rradc not checked separately. Battery 4376 mV before and after (Android). Phone back as Android on slot b by itself (adb up); fastboot showed flags a and b successful, none unbootable, current-slot b; left via fastboot reboot. Nothing flashed. Next: charger's dts must not wait on tps61099 (drop or stub the supply link / fw_devlink) - a dts choice for the coordinator, then c3.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-19-c2-try1/


<a id="walleye-c3-built"></a>
### walleye-c3-built

Date
: 2026-11-18

Brief
: c3 built: c2 with only the command line changed (fw_devlink=permissive added); 8 checks ok, rebuild identical

Question
: Can c3 be made from c2 changing nothing but the command line?

Found
: build/build-c3.sh unpacks c2, repacks with the same kernel blob, ramdisk, header fields and size, cmdline = c2's + ' fw_devlink=permissive', and the AVB footer with c2's salt and props. Checks count: kernel and ramdisk equal c2's byte for byte, header equal but the cmdline, AVB info equal but digest and sizes, footer verifies. Built through lpoco-lab job start, log read. boot-c3.img sha256 d8d975650bc570833f21989b584feda4eac5bca387972c456fb52e541011a401; rebuilt byte for byte.

About
: walleye-first-fastboot-boot


<a id="walleye-z8-tried-no-pmic-wdt"></a>
### walleye-z8-tried-no-pmic-wdt

Date
: 2026-11-18

Brief
: z8 once from slot b: command loop up, but no PMIC watchdog device: only the apps wdt 17817000 exists; pm8998 pon never bound (qcom-pon is =m, ramdisk has no modules)

Question
: Does z8's pm8998 PMIC watchdog bite unpetted (item walleye-pmic-wdt-proof)?

Found
: z8 (sha 797778f2...) booted from memory on slot b. /sys/class/watchdog has only watchdog0 = 17817000.watchdog (qcom_wdt). DT has pon@800/watchdog okay + wakeup-source; pm8916-wdt driver registered (=y) but the pon parent has no driver bound: CONFIG_POWER_RESET_QCOM_PON=m and qcom-pon.c populates its children. Fix =y (z9). Nothing armed or flashed.

About
: deep-sleep


<a id="walleye-z9-built"></a>
### walleye-z9-built

Date
: 2026-11-18

Brief
: z9 built: z8 with CONFIG_POWER_RESET_QCOM_PON=y (qcom-pon populates the PMIC watchdog child); 28 checks ok; sha256 de051922...

Question
: Why was no PMIC watchdog on z8, and what image has one?

Found
: build-kernel.sh and walleye.config now require POWER_RESET_QCOM_PON=y; build/build-z9.sh = build-z8.sh with the config-diff check allowing that line; images-z9/boot-z9.img sha256 de051922417e4f039fda4fb82ce1d9de485116f323f6009fe879ff7fbfdfe521.

About
: deep-sleep


<a id="walleye-z9-tried-pmic-wdt-bites"></a>
### walleye-z9-tried-pmic-wdt-bites

Date
: 2026-11-18

Brief
: z9 from slot b: PMIC watchdog (watchdog1, pon@800 child) bit unpetted, phone gone 20 s after arming for 20 s - but it did not come back on USB afterwards (needs hands)

Question
: Does pm8998's PMIC watchdog bite unpetted (test pmic-wdt-bites)?

Found
: tests/host/pmic-wdt-bites.sh: dry run pass (watchdog1 under spmi; kernel exposes no identity/pretimeout/bootstatus attributes, so found by sysfs path). First armed run failed (busybox watchdog without -F forks and keeps petting: no bite in 60 s, the test seen failing); with -F: pass, gone 20 s after arming for 20 s. Afterwards no USB device at port 3-2 for 10+ minutes (non-bite tries return to Android in ~75 s): the bite left the phone off or hung; fastboot and adb absent. Needs the author's hands: hold power ~15 s.

About
: deep-sleep


<a id="walleye-dtbo-overlay-id-how-it-stands"></a>
### walleye-dtbo-overlay-id-how-it-stands

Date
: 2026-11-18

Brief
: Walleye's bootloader selects dtbo entry 3 (id 0x1bc rev 3, MP1) itself; taimen's 2704 is not walleye's; whether a missing id refuses a boot is not known yet

Question
: Which dtbo overlay id does walleye's bootloader need before a mainline kernel boots, and can the vendor-overlay nodes be dropped?

Found
: Every walleye boot (control and our dtbs z6, c2, z7) carries androidboot.dtbo_idx=3 set by the bootloader: id 0x1bc rev 3, from hw-revision MP1. Taimen's 2704 (0xa90) is entry 12, another board's. Our dtb boots with that overlay merged, so the id is known and present. Not measured: boot with the id absent; only a dtbo write tests it, and the stock image is kept for restore. Vendor-overlay nodes in the dmesg go only with a replacement dtbo holding one empty 0x1bc rev 3 overlay: a keeper try.

Sources
: kcxt-dtbo-wahoo-mainline

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-19-dtbo-id/


<a id="walleye-c3-tried"></a>
### walleye-c3-tried

Date
: 2026-11-18

Brief
: c3 once from slot b: charger probed with fw_devlink=permissive, type USB, online 1 (charger-check pass); then the phone stayed in c3, silent, after the reader's reboot -f

Question
: Does pmi8998's charger probe on c3 (c2 + fw_devlink=permissive), and does the phone return by itself?

Found
: c3 (sha256 d8d97565...) booted from fastboot at 15:12, gadget 1d6b:0104 at 18 s, reader passed (16 answers, dmesg 319 lines, kept outside git in ~/lab-material/walleye/c3-try1). tests/host/charger-check.sh: pass walleye-charger-smb2 - charger probed, type USB, online 1 (one cable state only; follows-the-cable not shown). So fw_devlink=permissive is enough to probe it; next c4 (tps61099 as a fixed regulator, item walleye-charger-supplier). But the phone did not return as Android: after the reader's final reboot -f and the init's own 180 s self-reboot it was still on USB as 1d6b:0104 at 15:26, shell silent to a written command, no adb. c2 returned by itself. Nothing flashed, slot b active. Not known whether it recovers slowly (compare the z9 bite: back after more than 10 minutes) or needs the author's power hold.

About
: walleye-first-fastboot-boot


<a id="walleye-pmic-bite-reset-type-how-it-stands"></a>
### walleye-pmic-bite-reset-type-how-it-stands

Date
: 2026-11-18

Brief
: PMIC bite power-off: qcom-pon is not the cause; vendor never sets the WD S2 type; only a register read settles which of three causes

Question
: What reset type does the PM8998 watchdog bite perform and why did z9's bite power the phone off?

Found
: Question: what reset type does the PM8998 S2 stage (the watchdog's bite) perform, and why did z9's bite power the phone off?
Read from sources (no phone touched):
1. Hypothesis (b) of the coordinator, "qcom-pon overwrote the type", is ruled out: mainline qcom-pon.c v7.3 writes only 0x8f (soft reboot spare) and populates children; it never writes 0x56, 0x5a, 0x5b or 0x57.
2. The series writes S2_CTL (0x56) = 0x07 (HARD) at probe, plain write. The vendor kernel (walleye's own GPL driver) never writes 0x56 at all: it toggles only the WD enable bit in S2_CTL2 (0x57). The vendor's reset type for PS_HOLD goes through 0x5A (type) / 0x5B (enable, on gen2) with enable cleared first and a 500 us wait, no SEC_ACCESS unlock. So the vendor leaves the WD S2 type to the PMIC default / the bootloader (PBS/SBL): what it is on walleye is not in any source, only in the register.
3. Type encodings agree: WARM 0x01, SHUTDOWN 0x04, HARD 0x07 (vendor header), same as the series' constants. HARD "resets the MSM and all PMIC peripherals", WARM "the MSM but not all PMIC peripherals". Whether a HARD reset ends in a restart on PM8998 depends on a power-on trigger after the rail cycle; the sources do not say, so a HARD type that powers the phone off with nothing to turn it back on is plausible (a hard reset is a power-off followed by a re-trigger; the 'z9 off the bus' fits a PMIC that cycled and did not re-trigger, but also fits one that restarted into a boot that never enumerated USB).
4. 'Off the USB bus' is not 'powered off': the record has no current, LED or screen evidence. The second boot the phone made (if any) would show as an slot change or a POFF reason; only a register read settles which.
So: three causes stay open (write did not take or read back wrong; HARD does not self-restart on this PMIC; it restarted into a boot with no USB). One cheap, safe phone read separates them and no bite is needed: read 0x56,0x57,0x5a,0x5b,0xc5 (and 0x54,0x55) before arming (the spmi regmap debugfs, if z9 has it, else a readback in the ramdisk's command loop). The reading also shows what the bootloader set before the series wrote 0x07 (read before the driver probes is impossible from Linux; read the value in an image without the series' write, i.e. z8's dtb with the wdt node disabled, or note that the probe write comes first).
Not proven by the bite: the series bites (it does: pmic-wdt-bites, 20 s); its reset type is a board/PMIC setting the series sets to HARD as its own default for pm8916 - the 'power off' is not the series' fault, but the series' hard-coded HARD is a board choice that should become a property (qcom,s2-reset-type style) or stay for pm8916 only: an upstream question once measured.

Sources
: lineage-wahoo-qpnp-power-on,lineage-wahoo-qpnp-power-on-h,pmic-wdt-series-pm8916-wdt,mainline-qcom-pon-v7-3

About
: deep-sleep


<a id="walleye-c4-built"></a>
### walleye-c4-built

Date
: 2026-11-18

Brief
: c4 built: c2 with only the dtb changed - the TPS61099 boost as a fixed regulator on pm8998 gpio 5, the overlay's driverless /soc@0/tps61099 disabled; default fw_devlink; 26 checks ok, rebuilt same

Question
: Can the charger's supplier link be met in our dts, without fw_devlink=permissive, from the facts of the phone's running fdt?

Found
: Facts (vfa try fdt + the dtbo entry 3, read with dtc): the vendor overlay (fragment 111) adds /soc@0/tps61099 {compatible ti,tps61099; enable-gpios = pm8998_gpios 5 0; child ti,tps61099 regulator-name tps61099-vbus} and fragment 112 points the charger's external-vbus-supply at that child. The overlay merge overwrites a compatible we set on that node, so the regulator cannot sit at /soc@0/tps61099. Our dts therefore has a regulator-fixed of its own at /regulator-tps61099-vbus (gpios pm8998 5 active high, no boot-on: driven low at probe) and marks /soc@0/tps61099 status disabled (the overlay sets none); kernel drivers/of/property.c of_link_to_phandle adds no fwnode link when the supplier or an ancestor without a device is unavailable, so the charger's wait is DROPPED, not met; the charger's external-vbus-supply still names the overlay's child and qcom_smbx asks for no regulator, so the new regulator has no consumer yet. The merged dtb (fdtoverlay with the dtbo entry) keeps both. build/build-c4.sh (run by build/job-c4.sh through lpoco-lab job start, log read from the journal): c2's image with Image.gz, ramdisk, header (command line without fw_devlink), size and AVB footer equal, only the dtb changed; the dtb is built in the pristine v7.3 tree from our dts without the PMIC-watchdog block (z8's change, not c2's); 26 checks ok, the dtb diff against c2's counted to the regulator and the disabled node. boot-c4.img sha256 773fa74d4a6acd8f5c3f4518b44e254ad01f926f43877cdc141dd3d614429692; dtb 3ef2a2108b608c16d276ef60c474b2c0b4c95ea56cf2b7eb0499473160839b9c; rebuilt byte for byte (first build of the gpio spelling's twin). programmer review: claims hold; gpio spelling changed to gpios. Not shown yet: that the charger probes on it (the try).

About
: walleye-first-fastboot-boot


<a id="walleye-c3-hang-how-it-stands"></a>
### walleye-c3-hang-how-it-stands

Date
: 2026-11-18

Brief
: c3 hang: the SoC never left the USB bus, so no reset ran; the only visible change from c2 is the charger probe; the way back is a restart gate and a PMIC-side deadman

Question
: Why does c3 (c2 + fw_devlink=permissive, charger probed) not restart where c2 did, and what restart path must be proved before an image that probes the charger is used for a long try?

Found
: READ from the host journal, the c2 and c3 materials and Linux v7.3 source; nothing ran on the phone. MATERIAL: c2-host-journal.txt, c3-host-journal.txt (host `journalctl -k`, CET = Z+1).
FACTS. (1) c2 control: gadget 1d6b:0104 enumerated 14:40:17, USB disconnect 14:43:17 = exactly 180 s later (the init's own self-reboot: 90 beats of 2 s), Android (18d1:4ee7) back at 14:43:37. The same init (init-z6) and the same psci SYSTEM_RESET therefore work when the charger is deferred. (2) c3: enumerated 15:13:50, NO disconnect and no re-enumeration up to 15:42 (28+ min, device 65 still on the bus): the SoC never reset. So it is not a reboot loop (a loop shows disconnects), not a slow recovery like z9, and not 'a reset that came back wrong': nothing shows a reset was carried out. The init's 180 s self-reboot (due ~15:16:50) and the reader's reboot -f left no trace. The host's descriptors are cached, so the bus entry proves only that the USB pull-up is still on, not that the kernel answers. (3) The init is not the cause: the self-reboot is its own subshell, independent of the tty, and the identical init restarted c2. (4) c2 to c3 differs by (a) the charger probing, (b) fw_devlink=permissive. The dmesg diff (c2 vs c3) shows no new driver probe line besides the charger path (the rest is order, and the four 'sync_state() pending' lines of gcc that c3 lacks because no consumer is deferred any more); so (a) is the only visible change. (5) The smbx driver at probe (qcom_smbx.c smb_init_seq) writes: Type-C software control (the power-role mask is cleared although its comment says 'upstream facing port': UFP_EN_CMD is not set, so all role bits are 0), OTG_CFG, CHARGING_ENABLE_CMD, CHGR_CFG2, USBIN ICL options (USB51), AICL options (SUSPEND_ON_COLLAPSE cleared), pre/fast charge current (1 A cap), and its own watchdog off. It has no .shutdown, no .remove and no reboot notifier: nothing in the driver touches the reset path. c3's readings: type USB, online 1, status Full, voltage_now 0, current_now 0 (rradc gave no numbers). (6) Every reset this phone has is TZ's: psci 1.0 gives only SYSTEM_RESET (no RESET2); panic=5, reboot -f and sysrq b all end in that SMC; the kpss apps watchdog's bite is a secure-side path too and was never shown to restart this phone on its own (T3 came back as Android at 42 s, class 'alive', no marker; z9's PMIC watchdog bit and powered the phone OFF, needing hands). 'The watchdog did not bite' is therefore weak evidence: the apps wdt's bite is not proven to work here.
RANKED CAUSES (none tried). (A) The reset request stalls in TZ or the PMIC: the one thing that differs with the charger probed is the PMIC's own state (USBIN present; Type-C and charge registers written); TZ's SYSTEM_RESET programs the PMIC PON reset type over its own SPMI channel and de-asserts PS_HOLD; a wait there freezes the CPU in the SMC and, with the other CPUs stopped by the restart, leaves the USB pull-up on and the host with a silent device - exactly what is seen, and why neither watchdog helped. Needs a PMIC-side exit (hold Power ~15 s, the KPDPWR S3 reset: hands). (B) A kernel crash or hang in the charger IRQ or rradc path before the reset request, then panic=5 into the same stalled SMC; the reader's 16 answers were fine, so a late event (cable state change, AICL, bark IRQ) would be needed; open. (C) One of the Type-C role / OTG / ICL writes changes the PMIC so that a PS_HOLD reset is refused or held (charger-triggered power-on logic with an open role); a sub-case of A that the register bisect separates. REJECTED by the facts: the init blocked on a tty write (c2 same init), a slow z9-like recovery (z9 left the bus), a reboot loop (no disconnect), fw_devlink=permissive itself (no probe other than the charger appeared).
THE PSTORE can give nothing likely: a hang writes no record, and the author's power hold is a PMIC hard reset which loses DDR (research walleye-pstore-warm-reset-how-it-stands). Ask the keeper for `ls /sys/fs/pstore` once, free, but do not wait on it.
THE WAY BACK, in the order it can be proved (decision 2026-11-18-walleye-restart-gate-before-charger-try): (1) tests/host/restart-check.sh (new, host-side, from the host journal) is the gate: an image passes when the gadget leaves the bus within 200 s of enumerating; restart-selftest.sh proves it on the recorded c2 (pass) and c3 (fail) journals. (2) c5 = c4's dtb with the charger driver built as a module and NOT loaded: the control - proves c4's dtb change (tps61099 fixed regulator, pm8998 gpio 5) restarts; gate pass expected. (3) c6 = the same, the init loads the module 40 s after enumeration and the self-reboot stays at 180 s: if the gate fails here and (2) passed, the driver's probe is the cause (A or C); hands needed once. (4) only then bisect inside the driver by a module option that skips one smb_init_seq group at a time (Type-C, OTG, charging, ICL/AICL), each image gated. (5) The deadman that needs no TZ is the PMIC watchdog with S2 type HARD RESET (items walleye-pmic-bite-reset-type, walleye-pmic-wdt-readback-image, subject deep-sleep): proved on a charger-less kernel first, then armed in every charger-probing image. Until z10 reads the type, treat a bite as a power-off (z9). NOT KNOWN YET: whether TZ or the PMIC stalls; whether the kernel still answers on the bus (a root read of the usbfs control endpoint on the host, with the author's go, would tell); what the PON reason registers say after the power cycle (readable only from a kernel with the pon node bound - the z9/z10 line).

Sources
: mainline-qcom-smbx-v7-3,src-wahoo-msm-poweroff,mainline-qcom-pon-v7-3

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-c3-hang/


<a id="walleye-c3-hang-unproven"></a>
### walleye-c3-hang-unproven

Date
: 2026-11-18

Brief
: The c3 'hang' is unproven: one witness (lsusb) kept a stale gadget while Android may have been up

Question
: Did the Pixel 2 hang in c3 for 44 minutes, or did the host keep a stale enumeration of our gadget?

Found
: Correction of lab research walleye-c3-hang-why (PR 250) and of the keeper's 'hung' call (PR 246). Facts from the host journal and sysfs: the c3 gadget (dev 65, serial walleye-first-boot) enumerated at 15:13:50 (local) and stayed on the bus until 15:57:09; Android (18d1:4ee7, dev 66, adb 'device product:lineage_walleye') enumerated at 15:57:12, 3 s later; the author says he did not reboot it and saw LineageOS on the screen throughout. Three seconds is far too short for a boot into Android, so either the phone had been in Android for some time with the host keeping a stale enumeration of our gadget (a USB host only drops a device when it sees the disconnect; a device that stops answering without releasing its pull-up stays listed), or the phone restarted much earlier and Android sat in a state the host did not list until something (a touch, a mode change of the USB configuration) made it re-enumerate. The record 'c3 never left the bus for 28+ min, so the SoC never reset' is therefore NOT PROVEN: it rests on lsusb alone, one witness. What the lane knows: the gadget stayed listed 44 minutes; adb never listed the phone in that time; the author saw Android on the screen (a command-mode panel also keeps its last frame, so the screen alone proves nothing either); the phone is in Android now. What the lane does not know: when the phone left c3 and whether the reset worked at 180 s as c2's did. The restart gate (decision 2026-11-18-walleye-restart-gate-before-charger-try) stays as a cheap check but its premise 'c3 hangs the phone' is withdrawn until the gate itself (restart-check.sh reading adb, fastboot and the tty beat, not lsusb alone) measures it. Also: 18d1:d001 'Nexus 4 (fastboot)' on bus 3 port 3-1 is NOT unknown: sysfs says product 'Qualcomm SM7150 Device', serial postmarketOS - it is surya's gadget (the other lab phone, on its own lease), listed under the generic vendor id's name.

About
: walleye-first-fastboot-boot


<a id="walleye-c5-built"></a>
### walleye-c5-built

Date
: 2026-11-18

Brief
: c5/c6 built: c4 with the charger as a module (qcom_smbx.ko in the ramdisk), loaded only when the command line says walleye.charger_after=SECONDS (1..120); 40 checks ok, rebuild identical, reviewed

Question
: Can one image serve as c5 (module never loaded) and c6 (module loaded 40 s after the gadget bound), the command line deciding, so a gate fail isolates the charger probe?

Found
: build/build-c5.sh takes c4's image (dtb, header, size, AVB footer) and changes only the kernel (c2's config with CONFIG_CHARGER_QCOM_SMB2=m, built by job-c5.sh in the pristine v7.3 tree; rradc stays =y), the ramdisk (z6's packing, byte-equal to c4's with init-z6; plus /lib/modules/qcom_smbx.ko and init-c5) and, for c6 only, the command line + walleye.charger_after=40. init-c5 = init-z6 plus one block right after the bind line (checked by diff hunk): it loads the module only with the argument (1..120 s: the bind waits at most 30 s and the self-reboot is at 180 s) and only when the gadget is bound; tests/host/c5-loader-selftest.sh measures that on the host. Checks count: config differs from c2's in one line; module vermagic equals the kernel release (7.3.0-dirty), no module dependencies; Image.gz without the charger string; c5 and c6 equal but the command line; AVB verifies. boot-c5.img sha256 5a566d70b2646b9fb3a2527768b8ebbd7193ce3dea92fe15e61838f4056bd520; boot-c6.img 774b20f0ee0374bf3c7e2874067f4958442a8c8fc891356825529cbb429f334b; both rebuilt byte for byte; in ~/Projects/walleye-build/images-c5. c5's command line is c4's (no argument), so c5 differs from c4 in kernel config and ramdisk only. Reviewed by the programmer subagent: the placement check, test /tmp use and the 150 s cap were its findings, fixed.

About
: walleye-first-fastboot-boot


<a id="walleye-c5-tried"></a>
### walleye-c5-tried

Date
: 2026-11-18

Brief
: c5 once from slot b (module not loaded): gadget 1d6b:0104 up at 18 s, left the bus at exactly 180 s, Android back; restart-check passes

Question
: Does c4's dtb alone restart the phone (c5: charger module not loaded)?

Found
: boot-c5.img booted from memory from slot b, battery 4376 mV. Enumerated 1d6b:0104 (walleye-first-boot) 18 s after the boot; the init's 180 s self-reboot took the phone off the bus at 16:04:32+180 s and it returned as Android (slot b, a and b successful, nothing flashed, 4378 mV). tests/host/restart-check.sh on the host journal: pass, 180 s. So the c4 dtb (tps61099 modelled as a fixed regulator) restarts: the restart gate is met for the no-charger-probe image; c3's hang came from the charger probe. Next: c6 (module loaded 40 s after enumeration) is the step that isolates the probe.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18/


<a id="2026-11-18-walleye-phone-state-three-real-outputs"></a>
### 2026-11-18-walleye-phone-state-three-real-outputs

Date
: 2026-11-18

Brief
: phone-state.sh on the real phone: android (adb device, usb 18d1:4ee7), fastboot (listed, 18d1:4ee0), mainline-answering (banner2 gadget 1d6b:0104, tty banner)

Question
: What does phone-state.sh print on the real walleye in Android, fastboot and our gadget?

Found
: Android: witness adb device, fastboot none, usb pixel:18d1:4ee7, tty n/a -> verdict android. Fastboot (after adb reboot bootloader): adb none, fastboot listed, usb pixel:18d1:4ee0 -> verdict fastboot; slots a,b both successful, unbootable no, retry 0, current b, unlocked. Mainline (fastboot boot now of images-banner2/boot-banner2.img from slot b, nothing written): left fastboot after 15 s, back as USB 1d6b:0104 after 18 s; usb gadget, tty banner -> verdict mainline-answering. Read only on the phone.

About
: walleye-first-fastboot-boot


<a id="walleye-z10-built"></a>
### walleye-z10-built

Date
: 2026-11-18

Brief
: z10 built: z9 plus an init that reads the PON registers into the banner and a command (reads only); 27 checks ok; sha256 3485e89b...

Question
: How can the PON watchdog registers reach the log channel that works, when the debugfs reads through the command loop came back blank?

Found
: The z9 test guessed the regmap name (pmic-spmi, dir 0-00) and scanned nothing it could name; z10's init lists every regmap, reads the seven registers of each that has them, one fixed-width line each through the registers file (the kernel reads only from the file position), and prints them where the banner already works (WALLEYE-BANNER-PON beat=N [pon-regs 0x54=.. 0x55=.. 0x56=.. 0x57=.. 0x5a=.. 0x5b=.. 0xc5=.. src=[REGMAP]]), also on the command pon-regs; when none has them it says pon-regs none regmaps=[names] dbgfs=[yes|no], so a miss names the maps. Reviewed by the programmer subagent: a full 65536-register scan per poll was replaced by one-line reads, an all-XX map stopped counting as read, a stacked debugfs mount guarded. Not tried on the phone: the keeper's item walleye-pmic-wdt-z10-read-pon-regs.

Sources
: pmic-wdt-series-pm8916-wdt,lineage-wahoo-qpnp-power-on

About
: deep-sleep


<a id="walleye-z10-tried-pon-regs-not-read"></a>
### walleye-z10-tried-pon-regs-not-read

Date
: 2026-11-18

Brief
: z10 once from slot b: booted, command loop up, but banner stayed 'pon-regs not read yet' for 60+ s; test failed, nothing armed

Question
: What do the PON S2 registers read, from z10's banner?

Found
: z10 booted from memory (fastboot boot now), ttyACM0 came up, banner beats to 25 (up 60 s) all said 'WALLEYE-BANNER-PON [pon-regs not read yet]'. The test script ran only after the boot, 40 s listen, no line with registers: fail. A hand-typed pon-regs on the tty was garbled by banner output (WALLpon-regs), so the command's answer is untried. Phone fell back to Android on slot b by itself (init reboots at 180 s), adb up, flags unchanged. Nothing armed, nothing flashed. Next: find why init-z10 does not read (regmaps? read happens at which step?) - read the init's step list; run the test script in parallel with the boot, with the command sent as one write.

About
: deep-sleep


<a id="walleye-z11-built"></a>
### walleye-z11-built

Date
: 2026-11-18

Brief
: z11 built: z10 with UFS host and QMP phy built in, ufshc/ufsphy okay (l20/l26/s4 rails), init-z11 lists the LUNs and reads the boot header; reads only

Question
: Can the init list the UFS LUNs and partitions of the Pixel 2 and read its own boot.img header without a write, from one image?

Found
: Built, not tried. images-z11/boot-z11.img sha256 1d84e4675cc7e3b86b7683561c37b2a987cc98de32988838ffa439401e8aba7d (initramfs-z11.cpio.xz ead154de..., dtb 76403f05...). Kernel: v7.3 + pmic-wdt series (tree wt-k-z8), config + SCSI_UFS_QCOM=y, PHY_QCOM_QMP=y, PHY_QCOM_QMP_UFS=y (defconfig has them =m, the initramfs has no modules). Dts: rails l20 2.96 V (vcc), l26 1.2 V (vccq, hba, phy pll l2), s4 1.8 V (vccq2), phy l1 - values of mainline's OnePlus 5 file, same SoC and RPM regulators. Ramdisk init-z11 = init-z10 + /bin/ufs-lsblk (hosts, LUN vendor/model/rev, partition GPT names and sizes from sysfs) and /bin/ufs-hdr NAME (one dd of 48 bytes, the Android header fields), both banner (WALLEYE-BANNER-UFS) and commands. The init also sets every disk read-only in the kernel (blockdev --setro) and keeps refreshing the list until it is stable. Header, command line, AVB footer, size 32 MiB are z10's; ramdisk packing proved equal to z10's. 60+ build checks ok, host selftest ufs-lsblk-selftest ok. Not known yet: which UFS card the phone holds; Sony's yoshino keeps UFS off for a card that needs quirks before probe (bootloader erased after probe) - for the Pixel 2 only the wahoo GPL tree (kcxt, 5.17) is known to have used UFS; that is inference, not measured here.

Sources
: linux-msm-mainline-status

About
: walleye-first-fastboot-boot


<a id="walleye-ufs-hazard-verdict"></a>
### walleye-ufs-hazard-verdict

Date
: 2026-11-18

Brief
: Pixel 2 UFS is Hynix 64GB, the maker Sony's quirks single out; mechanism of the yoshino erase unpublished; no way back if the bootloader goes; z11 needs the author's word

Question
: Can probing the Pixel 2's UFS damage the bootloader (the Sony yoshino failure)?

Found
: Android says 64GB Hynix (model/fw unreadable without root). Sony yoshino: probe without quirks erased the bootloader; the downstream fix is NO_PURGE (two Hynix models, one Samsung), EXTEND_SYNC_LENGTH (all Hynix), AVAILTXDATALANES=1; nobody published why. Our 7.3 tree applies only HOST_PA_SAVECONFIGTIME and DELAY_BEFORE_LPM for Hynix; nothing in the driver sends purge by itself (only ufs_bsg), the probe sends reset, link startup, NOP, fDeviceInit, descriptor reads, power-mode change, SCSI scan. OnePlus 5 proves MSM8998 UFS on mainline for its card. Failure mode: bootloader gone, EDL not known reachable; a Google bootloader.img helps only while fastboot lives. Chance: not zero, unknown; not measured.

Sources
: src-sony-ufs-quirks-forward-port,src-yoshino-dtsi-ufs-disabled,src-sony-bugtracker-799,src-oneplus5-ufs-mainline,src-kumano-ufs-quirks-thread

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-ufs-hazard/


<a id="walleye-z12-built"></a>
### walleye-z12-built

Date
: 2026-11-18

Brief
: z12 built: z11 with only the kernel changed (Hynix UFS quirks); sha256 f959c448; not tried; review found the purge guard misses ufs_bsg

Question
: Can our 7.3 kernel carry Sony's Hynix UFS quirks, in a z11-identical image?

Found
: Kernel topic/ufs-hynix-quirks 12dca401a on e123d3bff: NO_PURGE, EXTEND_SYNC_LENGTH (G1-3 = 0x48), ONE_TX_LANE on SKHYNIX ANY_MODEL in ufs_qcom_dev_fixups. z12 = z11 with only Image.gz changed (build-z12.sh, checks ok); images-z12/boot-z12.img sha256 f959c44868200c9a4056e5f6ffa56679e42b24a98f64f8337c87b8f791c74f41; never run. Programmer review: (1) the purge guard sits in ufshcd_query_flag, but nothing in the tree sends fPurgeEnable except raw UPIU from userspace via ufs_bsg, which bypasses it - the guard protects nothing yet; (2) blanket for every Hynix on every ufs-qcom SoC; (3) one lane cuts TX only, link ends asymmetric, Sony's behaviour unverified; (4) PA_AVAILTXDATALANES is read-only per spec, likely a no-op, return values dropped; (5) header comment states the mechanism as fact; (6) build script greps the working tree, no check tying the Image to the commit. Lane and sync-length order is right (apply_dev_quirks runs before the power-mode change; lane_tx override takes effect).

Sources
: src-sony-ufs-quirks-forward-port

About
: walleye-first-fastboot-boot


<a id="walleye-ufs-card-identity-2026-11-18"></a>
### walleye-ufs-card-identity-2026-11-18

Date
: 2026-11-18

Brief
: Walleye UFS card: vendor Hynix known; model and firmware not obtainable without root (shell-user routes tried)

Question
: What are the model and firmware revision of walleye's UFS card, without probing it from our kernel?

Found
: ro.boot.hardware.ufs=64GB,Hynix is all a non-root read gives; sysfs scsi attrs, /proc/scsi, ufshc sysfs and dmesg are denied; no public page names the card; Google bootloader.img carries no inquiry data. Left to the keeper: fastboot getvar, or a root shell in recovery reading /sys/block/sda/device/{model,rev}. Hynix vendor alone is enough to apply Sony's Hynix quirks (sync length) first.

About
: walleye-ufs-card-identity

Material
: research/walleye-ufs-card-identity/2026-11-18/


<a id="walleye-ufs-card-identity-not-readable-stock"></a>
### walleye-ufs-card-identity-not-readable-stock

Date
: 2026-11-18

Brief
: UFS card model/firmware is not readable from stock Android shell or fastboot getvar all; only ro.boot.hardware.ufs=64GB,Hynix; needs recovery root

Question
: Can the UFS card's model and firmware be read without our kernel probing it?

Found
: Android shell (uid shell, no su): /sys/block/sda/device/{model,rev,vendor} permission denied, /proc/scsi/scsi denied, ufshc sysfs path absent; props give only ro.boot.hardware.ufs=64GB,Hynix and bootdevice 1da4000.ufshc. Fastboot (bootloader mw8998-003.0085.00) getvar all lists partitions, slots, battery, avb - no UFS model, vendor or firmware variable. Phone returned to Android slot b, nothing flashed. Remaining way: a root shell in recovery, which needs a recovery image we do not have; ask coordinator.

About
: walleye-first-fastboot-boot


<a id="walleye-z13-built"></a>
### walleye-z13-built

Date
: 2026-11-18

Brief
: z13 built: z11 with only the kernel changed (Hynix quirks, review fixes: purge refused at the UPIU send); sha256 b8348e40; not tried; programmer review done

Question
: Do the review findings of walleye-z12-built (purge guard misses ufs_bsg, unchecked writes, comments as fact, no tie to the commit) hold fixed in a z13?

Found
: Kernel topic/ufs-hynix-quirks d49d54405 (on 12dca401a): one helper ufshcd_purge_refused refuses set/clear/toggle of fPurgeEnable on ufshcd_query_flag and in ufshcd_issue_devman_upiu_cmd, judged by the UPIU's own transaction code (the programmer review showed that keying on ufs_bsg's msgcode is bypassable by NOP_OUT msgcode with a query header). Sync-length write failures are logged; the read-only PA_AVAILTXDATALANES write is dropped (lane_tx=1 in the power-mode change does it); ratelimited warning; comments say reported. build-z13.sh ties the Image to the commit (banner g d49d54405f11, -dirty only from our dts Makefile lines, ufs files clean and no untracked) and counts the guard in the source; all checks ok. images-z13/boot-z13.img sha256 b8348e40182e93f1f6c575abdcf586fc6ff91ebaed68bd0a9ecf51f2522cb89f; never run. Not measured: the guard's behaviour (checks count source text; no test calls the raw path); advanced RPMB purge (UFS_RPMB_PURGE_ENABLE) is not covered and not known to matter; blanket for every Hynix on every ufs-qcom SoC (finding 2) and Sony's lane behaviour (3) stay open; the fixup commit should be squashed into 12dca401a before any upstream.

Sources
: src-sony-ufs-quirks-forward-port

About
: walleye-first-fastboot-boot

Supersedes
: walleye-z12-built


<a id="walleye-ufs-card-identity-root"></a>
### walleye-ufs-card-identity-root

Date
: 2026-11-18

Brief
: Pixel 2 UFS card is SK hynix H28U74301AMR fw D003, 64 GB, spec 0x1002, HS-G3 2 lanes; health descriptor reads zeros

Question
: Which UFS card and firmware does the Pixel 2 carry, read-only with adb root?

Found
: sysfs: vendor SKhynix, model H28U74301AMR, rev D003; 6 LUNs (LUN0 124674048 sectors, about 64 GB; bBootEnable=1) plus 3 well-known LUNs. Device descriptor (debugfs ufshcd0/dump_device_desc): wSpecVersion 0x1002, wManufactureID 0xad01 (SK hynix), wManufactureDate 0x170a, bNumberLU 6, bNumberWLU 4. Host: ufshcd-qcom 1da4000.ufshc, QC ICE 3.0.65; vendor kernel links PWM G1 1 lane then FAST G3 2 lanes rate B; host_regs VER 0x210 (UFSHCI 2.1). Health descriptor dump read zeros (not exposed), so wear is unknown. Boot arg androidboot.hardware.ufs=64GB,Hynix. Nothing written; serial omitted. Fits decision 2026-11-18-walleye-hynix-quirks-in-the-fixups-table. The UFS-probe decision is not made here: it goes to the top coordinator via the walleye coordinator.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-19/


<a id="walleye-z14-built"></a>
### walleye-z14-built

Date
: 2026-11-18

Brief
: z14 built: z7 with only the ramdisk changed: ECM beside the ACM console, usb0 172.16.43.1, DHCP lease, telnetd; sha256 c2e616f8; reviewed; not tried

Question
: Can the proven z7 image carry USB networking (ECM) without touching the ACM console, built and checked before a try?

Found
: z14 = z7's boot image with only the ramdisk changed (kernel+dtb, header, command line, AVB footer z7's byte for byte; z7's kernel has the CDC ECM function). init-z14 = init-z6 plus an ecm.usb0 function in the ACM configuration (dev_addr/host_addr written before the bind, the mainline way), then in a subshell: usb0 172.16.43.1, udhcpd (one lease, 172.16.43.2), telnetd bound to 172.16.43.1. The ACM console and command loop are untouched, so a network failure leaves z6's shell. 28 checks ok, programmer review done (devpts mount added), rebuild identical; sha256 boot-z14.img c2e616f872328b1a28df9ff37903796579e078a7a7ea2f66195a928beee64238, initramfs-z14.cpio.xz 37e61b1b4e16ba7801d8cbb1ae6ed188e9671485bb5c770d1c558c3a664bc2d4 (~/Projects/walleye-build/images-z14). Built only, not tried. The proof walleye-gadget-up pings and talks telnet (tests/host/gadget-up.sh), not ssh: the initramfs has no sshd and no key; ssh waits for the real system.

About
: walleye-first-fastboot-boot


<a id="2026-11-18-walleye-edl-way-back"></a>
### 2026-11-18-walleye-edl-way-back

Date
: 2026-11-18

Brief
: No known way back from an erased Pixel 2 bootloader: EDL appears when the chain breaks, but no Google-signed MSM8998 firehose programmer is public

Question
: Is there a known way back from an erased bootloader on the Pixel 2 (EDL, test points, recovery)?

Found
: EDL 9008 is reached by Pixel 2s with a broken chain (several reports, Sahara HWID 0x0005e0e1, PK hash 7489217...), but recovery needs a firehose programmer signed to that hash; bkerler (2021): none released or leaked for Pixels; ~200-loader survey lists none; threads 2018-2025 end without a fix; Google's repair tool is Pixel 3+; no public test points for walleye. Verdict: no known way back, fairly sure. Preparable at no risk: read-back of xbl_a/b (4,149,248 B each, boot LUNs) and the rest of the boot chain as mode-600 files with sha256.

Sources
: src-bkerler-edl-204,src-xda-pixel2-hardbrick-3920807,src-hovatek-pixel2-26042,src-xda-edl-loaders-4525079,src-androidse-sailfish-225868,src-pixelrepair-withgoogle,src-passware-qc-testpoints,src-aleph-qualcomm-edl-2

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-edl-way-back/


<a id="walleye-z9-recovery-unproven"></a>
### walleye-z9-recovery-unproven

Date
: 2026-11-18

Brief
: z9's recovery after the PMIC bite is unproven as 'by itself' (the author's hold is a confound)

Question
: Did the phone come back by itself after the z9 PMIC bite?

Found
: Two rows of the lab say different things about the phone after the z9 PMIC bite: the z9 row says it did not come back by itself; the c3 try row (walleye-c3-tried) says z9 came back after about 10 minutes. The facts: after the z9 bite (14:54 local) the phone was off the USB bus; the keeper (session 348f6c2b) reported later that fastboot (18d1:4ee0, slot b current, successful) appeared 'a few minutes' after its last message, '>10 min', while the author held Power about 15 s in that same window (the coordinator's relay, 14:08-14:11Z, then Android 18d1:4ee7 with adb). The record does not tell which of the two recovered the phone. So the right wording for both rows: the recovery after a PMIC bite is UNPROVEN as 'by itself'; it is known only that the phone reappeared within about 15 minutes with the author's hold in the window. This research row corrects the c3 wording: wherever a row says 'z9 recovered by itself after 10 minutes', read: not proven (the hold is a confound).

About
: walleye-first-fastboot-boot


<a id="walleye-z14-tried"></a>
### walleye-z14-tried

Date
: 2026-11-18

Brief
: z14 once from slot b: gadget up at 18 s, ECM usb0 172.16.43.1 pings, telnet answers; walleye-gadget-up passes

Question
: Does z14 bring up USB networking (ECM+telnet) beside the ACM console?

Found
: boot-z14.img sha256 c2e616f8 booted from fastboot (left fastboot 15 s, gadget 1d6b:0104 at 18 s). Host got enp10s0f3u2i2 172.16.43.2; 172.16.43.1 pings; telnet 23 gives a root shell: uname 7.3.0-dirty, usb0 up, udc a800000.usb, dmesg 329 lines, no power_supply. Banner on ttyACM0 shows step 11 telnetd. First gadget-up.sh run failed only because the lab host has no nc; the script now falls back to telnet and passes. Nothing flashed; phone returned to Android.

About
: walleye-first-fastboot-boot


<a id="walleye-ufs-android-live-readback"></a>
### walleye-ufs-android-live-readback

Date
: 2026-11-18

Brief
: Android's own UFS bring-up on walleye read-only: gear 1 slow-auto at 1.82s, then HS G3 2-lane rate B at 2.05s; Hynix H28U74301AMR D003

Question
: How does Android's vendor 4.4.302 kernel bring this UFS card up (dmesg, debugfs, sysfs, DT, config, regulators)?

Found
: Host 1da4000.ufshc/phy 1da7000/ice 1db0000. dmesg: ufs_qcom_parse_reg_info warns vddp-ref-clk absent (assumed enabled); scsi host0 at 1.786; first link gear1 SLOWAUTO, then gear3 2 lanes FAST rate B at 2.051; LUNs 0-5 plus 3 well-known, SKhynix H28U74301AMR D003. rpm_lvl 3 (SLEEP/HIBERN8). Regulators: ufshc 1800/2960/1200 mV, phy 880/1200 mV, gdsc_ufs on. Vendor config UFSHCD_QCOM+ICE built in. Health desc dump reads zeros; no quirks printed by the driver. Raw files in the material folder, serial redacted.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18/


<a id="walleye-z15-built"></a>
### walleye-z15-built

Date
: 2026-11-18

Brief
: z15 built: z14 with kernel (RMI4 touch built in) and dtb (touch on blsp1_i2c5) changed; sha256 a675080f; 33 checks ok, rebuild identical, reviewed; not tried

Question
: Can the Pixel 2's Synaptics RMI4 touch controller be brought up with mainline's rmi_i2c, built and checked before a try?

Found
: z15 = z14's boot image with only kernel Image.gz and dtb replaced (header, command line, ramdisk with init-z14's ACM+ECM+telnet, AVB footer z14's). Kernel: v7.3 + pmic-wdt series, RMI4_CORE/I2C/F12 =y (F01 is in the core). Dtb: touchscreen@20 (syna,rmi4-i2c) on blsp1_i2c5, irq gpio125 falling, reset gpio89, vdd=pm8998 l28 3.008 V (from bob), vio=l6 1.808 V (from s5), values of mainline's OnePlus 5 and the wahoo GPL tree. Which board revision the lab phone is (xd adds a pmi8998 power gpio) is not known yet: if the probe times out at 0x20, look there first. Panel size not set. Test: tests/host/touch-read.sh (telnet, a person touches the panel, dmesg+input devices+od of the event node) then touch-check.sh (selftest 9 ok; no touch = skip, never fail). Programmer review done: regex anchored, no-dmesg is its own fail, zero events is skip, rpm supply wiring checked. Image sha256 a675080f42a29c25e50937da3d9ee124a387c0b7f0fe52110424c85c14f66a79 at ~/Projects/walleye-build/images-z15.

About
: walleye-first-fastboot-boot


<a id="walleye-z15-part1-tried"></a>
### walleye-z15-part1-tried

Date
: 2026-11-18

Brief
: z15 once from slot b: gadget/telnet up at 18 s, but no RMI4 touch device: i2c@c179000 has compatible qcom,i2c-msm-v2 (downstream), so no mainline driver binds

Question
: Does the RMI4 controller probe on z15 (part 1, no touch)?

Found
: z15 booted once from slot b via lpoco lab fastboot boot; walleye-gadget-up passes (ping+telnet). Absent: no rmi4/synaptics dmesg line, /sys/class/input has only pm8941_pwrkey and gpio_keys, /sys/bus/i2c/devices empty, no touch irq. rmi4_i2c driver is registered (RMI4_CORE/I2C/F12 =y, I2C_QUP=y). Cause: platform device c179000.i2c has no driver; its of_node compatible is qcom,i2c-msm-v2 (the downstream name), which mainline i2c_qup does not match, so blsp1_i2c5 never probes. Suppliers (gcc, tlmm, rpm) report available; gcc sync_state stays pending due to c179000.i2c. Next: z16 with the i2c node compatible set to mainline's qcom,i2c-qup-v2.2.1 (check the controller version) and the clocks/properties it needs. Phone returned to Android on slot b, nothing flashed. Material: ~/lab-material/walleye/z15-part1 (outside the repo).

Sources
: none

About
: walleye-first-fastboot-boot


<a id="walleye-z16-built"></a>
### walleye-z16-built

Date
: 2026-11-18

Brief
: z16 built: z15 with the QUP i2c node renamed qup@c179000 so the vendor overlay no longer overwrites its compatible; sha256 05849ba7; 26 checks ok, rebuild identical

Question
: How does z16 get the mainline i2c-qup driver onto the touch controller's bus?

Found
: z15's dtb already had compatible qcom,i2c-qup-v2.2.1 at 0xc179000; the bootloader's overlay (dtbo entry 3, fragment@142) merges its own i2c@c179000 by name into it and overwrites compatible (qcom,i2c-msm-v2), clocks, clock-names, interrupts, dmas and pinctrl (fdtoverlay on z15: merged compatible qcom,i2c-msm-v2). So changing the compatible in our dtb, as the backlog item said, would have changed nothing on the phone. z16 = z15 with the node name i2c -> qup (same length; node plus the two /__symbols__ paths blsp1_i2c5 and rmi4_f12; 9 bytes differ); the overlay then adds its own driverless i2c@c179000 beside ours, and fragment@143 lands on the vendor stub i2c_5, so touchscreen@20 stays under ours. Proof: tests/host/i2c-qup-after-merge.sh fails on z15's dtb, passes on z16's (after fdtoverlay with dtbo entry 3); build/build-z16.sh 26 checks ok, rebuild identical; boot-z16.img sha256 05849ba71282898c3fc5c77c7fa4b34ffdf64504ea1014edfd809f020c358aaa, dtb fed46c5b12110838e88c7499613d12b4f622d3cab8172955407abd4c75dc6f8f; not tried on the phone.

About
: walleye-first-fastboot-boot


<a id="walleye-z16-part1-tried"></a>
### walleye-z16-part1-tried

Date
: 2026-11-18

Brief
: z16 once from slot b: i2c bus 0 and rmi4_i2c at 0-0020 appear, but probe fails -5 (set page select failed): the controller does not answer

Question
: Does the RMI4 controller probe on z16 (part 1, nobody touching)?

Found
: z16 booted once from slot b via lpoco lab fastboot boot; gadget-up passes (gadget at 18 s). Progress over z15: i2c bus i2c-0 (c179000.qup) and device 0-0020 now exist; i2c_qup irq 70 registered; rmi4_i2c bound and ran. dmesg: 'rmi_set_page: set page failed: -5', 'Failed to set page select to 0', 'probe with driver rmi4_i2c failed with error -5' (1.09 s). So no input device from touch (only event0/1, pwrkey and gpio keys). -EIO on first read means the bus works but the chip is silent: likely unpowered or held in reset (supply regulators for the touch vdd/vio, reset gpio89, or a needed delay) - the dtb's touchscreen@20 properties not yet checked against the vendor tree. Proof of walleye-try-z16-part1 not met (rmi4 lines yes, 'no failures' no). Phone returned to Android on slot b, stock kernel, nothing flashed, slots a and b successful, fastboot reachable. Material ~/lab-material/walleye/z16-part1 (dmesg.txt, probe.txt).

About
: walleye-first-fastboot-boot


<a id="walleye-ufs-android-way-compared"></a>
### walleye-ufs-android-way-compared

Date
: 2026-11-18

Brief
: Android resets the card's RST_n and runs it with one quirk and 2 lanes; z13 never resets it and adds 4 Hynix flags (1 TX lane): not a match; abl hand-over and HW version unread

Question
: How does Android bring up this exact UFS card (vendor GPL kernel + live), and does our z13 path match it? Is a probe safe enough to take back to the author?

Found
: Read: Google kernel/msm android-msm-wahoo-4.4-android11 @52750b8f (the phone runs a LineageOS 4.4.302 fork, its UFS files not read), the keeper's live readings, ours at d49d54405 (z13). Compared with scripts (PHY tables as sorted lists, quirk tables, dts props live fdt vs our dtb) and by hand (reset, power-up, link startup, power mode). Rerun: material scripts/run-all.sh.
Android: host full reset, then a hardware RST_n pulse on the card (pinctrl ufs_reset low/high 10-15 us), then HCE: PHY reset 1 ms, tables, release, serdes start, PCS ready; link startup (G1 slow-auto, RX linecfg on, core clk 150, hw clock gating, LCC 0, REG1 mask); for THIS card (H28U74301AMR) only the quirk HOST_PA_SAVECONFIGTIME applies (vendor Hynix model entries are prefixes hB8aL1.. and do not match); HS-G3, rate B, 2 lanes each way; purge never sent (live: only fDeviceInit set).
Ours differs: (1) no card reset at all: no reset-gpios, and pinctrl-msm8998 ngpios=150 while UFS_RESET is pin 153, so it cannot be described; (2) z13 adds DELAY_BEFORE_LPM, EXTEND_SYNC_LENGTH, NO_PURGE (guard), ONE_TX_LANE: Sony's workarounds for other cards (hB8aL1/hC8aL1 D001) that Android proves this card does not need; one TX lane is a link Android never ran; (3) PHY: mainline msm8998 uses the sdm845 tables; vs vendor 3.1.0 table 10 differing lines (PLL_IVCO 0x0f vs 0x07, RX_EQU_ADAPTOR_CNTRL4 0x1d vs 0x1b, ours-only TX_RES_CODE_LANE_OFFSET and RX_MODE_00), vs 3.0.0 35 lines; which table Android ran is not known yet (HW version register unread); (4) no ICE, no phy/pll loads, plain clocks instead of hw_ctl ones, no bus votes/pm-qos, extra caps (clk scaling, WB, rpm autosuspend).
Sony's three fixes appear nowhere in the vendor sequence for this card; the erase mechanism stays unpublished, so nothing shows our path safe.
Verdict: z13 does not match Android and its extra Hynix flags move it away. Matching = SAVECONFIGTIME only (keep the NO_PURGE guard), the right PHY table, ICE/loads, and a card RST_n pulse (needs a kernel change: widen tlmm gpios or pinctrl states). Even matched, the probe is not safe enough to take to the author: abl's hand-over state is unread, the erase cause unknown, no way back (EDL). Provable before any try without our UFS: on live Android with root read UFS_HW_VERSION (0xe0), the full host regs and the PHY regs, and the ufs_reset pad state. Not known yet: abl hand-over, HW version, Lineage-fork differences.

Sources
: src-google-wahoo-msm44-android11,src-kumano-ufs-quirks-thread

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-ufs-android-way/


<a id="walleye-ufs-android-regs-read"></a>
### walleye-ufs-android-regs-read

Date
: 2026-11-18

Brief
: Live Android UFS read (read-only): UFSHCI 2.1, HS G3 2-lane rate B, Hynix 64GB; HW_VERSION and PHY regs NOT readable (no /dev/mem)

Question
: Which UFS HW version and PHY table did Android run on the Pixel 2, and what is the reset pad state?

Found
: Read as adb root on Lineage userdebug 4.4.302, phone left in Android, nothing written, no reset, no fstrim.
Read: debugfs ufshcd0/host_regs 0x00-0x9f: CAP 0x1587001f, VER(0x08) 0x00000210 (UFSHCI 2.1), HCS 0x01000000/0x00010217 region, UTRLBA 0xf5c6d000, UTMRLBA 0xf5c7a000, UTRLRDY/UTMRLRDY set. show_hba: nutrs 32, nutmrs 8, ufs_version 0x210, no errors. power_mode: gear 3/3, lanes 2/2, FAST MODE both, rate B. dmesg: init at gear 1 SLOWAUTO, then gear 3 x2 FAST rate 2 (B) at 2.3 s. Card 64GB Hynix (androidboot.hardware.ufs), bNumberLU 6, wSpecVersion 0x1002 (UFS 2.1), wManufactureID 0xad01. ICE 3.0.65. Pin 153 UFS_RESET is MUX/GPIO UNCLAIMED in pinctrl (so the pad is left to the UFS block, not driven as a gpio).
NOT answered: UFS_HW_VERSION (3.0.0 vs 3.1.0) and the PHY registers (phy_mem 1da7000-1da7da7): this Android kernel has no /dev/mem, /dev/kmem or /proc/kcore, and its debugfs does not export them; qcom/debug-regs is a driver debug dump, left unread. The proof of walleye-ufs-android-regs is not met; item stays open.
Next: a way to read 0x1da7000.. read-only (a kernel with devmem, or lab-built reader module) - needs a decision, not tried.

About
: walleye-first-fastboot-boot


<a id="walleye-z17-built"></a>
### walleye-z17-built

Date
: 2026-11-18

Brief
: z17 built: z16 with the touch controller's pmi8998 gpio 2 power switch modelled as a fixed regulator (from l28, 20 ms) that the touch vdd-supply takes; sha256 9c272493; 29 checks ok, rebuild identical

Question
: What keeps the RMI4 controller silent on z16 (probe -5 at set_page), and does a z17 that drives its power right build?

Found
: Read from the phone's own dtbo overlay (control/dtbo.img entry 3, fragments 143 and 145): the touch node has synaptics,power-gpio = pmi8998 gpio 2, power-on-state 1 (active high), reset gpio89 active low (1 ms, then 20 ms), irq gpio125, and a switch-gpio (tlmm gpio 75) whose job is not known yet. z16 powered only the rails l28 and l6; the pmi8998 gpio 2 was never driven, so the chip had no supply and answered -EIO at set_page (research walleye-z16-part1-tried). Mainline's rmi_i2c has no power or reset gpio, so the gpio becomes a regulator-fixed (pmi8998 gpio 2 active high, vin l28, startup-delay-us 20000) that the touch node's vdd-supply takes; reset stays pulled up. z17 = z16 plus that node, its symbol and one vdd-supply line; kernel, ramdisk, header, command line, AVB footer are z16's byte for byte. build/build-z17.sh: checks ok, rebuild identical; tests/host/touch-power-after-merge.sh fails on z16's dtb, passes on z17's after the overlay merge; programmer review done, findings fixed. dts source carries the same node (not compiled against a current kernel tree here). Next look if still silent: the pin's power-source/strength (spmi-gpio leaves the bootloader default), switch-gpio 75, a reset pulse.

About
: walleye-first-fastboot-boot


<a id="walleye-z17-part1-tried"></a>
### walleye-z17-part1-tried

Date
: 2026-11-18

Brief
: z17 once from slot b: gadget up at 18 s, touch still not probed: rmi4 0-0020 defers on regulator-touch-power; spmi pmi8998@3 has no reg, so no pmi8998 gpio exists

Question
: Does the RMI4 controller answer on z17 (pmi8998 gpio 2 power switch as fixed regulator), part 1, nobody touching?

Found
: z17 booted once from slot b via lpoco lab fastboot boot now; gadget 1d6b:0104 at 18 s. dmesg: 'i2c 0-0020: deferred probe pending: i2c: supplier regulator-touch-power not ready' and 'spmi spmi-0: node /soc@0/spmi@800f000/qcom,pmi8998@3 err (-22) does not have reg property'. So the fixed regulator never registers because its gpio (pmi8998 gpio controller) never exists: the pmi8998@3 node lacks its reg (SPMI usid 3 sub-address). No rmi4 probe, no input device from touch, so proof not met (nothing failed, nothing bound). Next: give pmi8998@3 a proper reg (<0x3 SPMI_USID>) in the dts so the pmi8998 gpio probes, then retry. Phone returned to Android slot b via telnet reboot -f, then adb reboot bootloader, state read (slots a,b successful, none unbootable, current b), fastboot reboot; stock kernel, nothing flashed. Material ~/lab-material/walleye/z17-part1 (dmesg.txt, input-devices.txt).

About
: walleye-first-fastboot-boot


<a id="walleye-z18-premise-falsified"></a>
### walleye-z18-premise-falsified

Date
: 2026-11-18

Brief
: z18 'give pmi8998@3 a reg' would change nothing: the reg-less node is the overlay's, the spmi core skips it; our pmic@3 and pmic@2 have reg. Why the pmi8998 gpio is absent is not known yet

Question
: Does giving spmi pmi8998@3 a reg make the pmi8998 gpio probe (backlog walleye-build-z18-pmi8998-reg)?

Found
: Read, not run. (1) z17's own dtb has pmic@2 (pmi8998_lsid0, holds gpio@c000) and pmic@3, both with reg <usid 0>; the two 'does not have reg' lines of the z17 dmesg name qcom,pm8998@0 and qcom,pmi8998@3, which the stock dtbo entry 3 adds in downstream style (fragments under spmi_bus; already known from the z1 build; tests/host/pmic-dmesg-check.sh tolerates exactly them). (2) Linux v7.3 drivers/spmi/spmi.c of_spmi_register_devices: a child without reg logs the error and 'continue's, so it blocks no other child; pm8998's pon/pwrkey (pmic@0) did probe in the same dmesg. (3) The kernel config of z15 and on has PINCTRL_QCOM_SPMI_PMIC=y, MFD_SPMI_PMIC=y, REGULATOR_FIXED_VOLTAGE=y, GPIOLIB=y. So a reg on the overlay's node cannot make the gpio appear; naming it as ours would add a second device at usid 3 or merge by name into our pmic@3. The z17 dmesg has no line at all from the pmi8998 gpio, charger or rradc, so it is NOT KNOWN YET whether pmic@2's children bound, deferred or failed quietly. Next look: a try of z17's image whose init reads facts (devices_deferred, /sys/bus/spmi/devices, gpiochips, the regulator's deferral reason), nothing in the dtb changed; or loglevel and dyndbg for spmi and pinctrl-spmi-gpio. Item walleye-overlay-collisions (host test of every by-name collision) comes first and may show one under pmic@2. Material: ~/lab-material/walleye/z17-part1/dmesg.txt lines 270-272 and 379.

About
: walleye-first-fastboot-boot


<a id="walleye-z17-spmi-gpio-facts-read"></a>
### walleye-z17-spmi-gpio-facts-read

Date
: 2026-11-18

Brief
: z17 read-only: the pmi8998 gpio chip exists (gpiochip2, 14 GPIOs, phandle matches) yet regulator-touch-power says 'can't get GPIO'

Question
: Why is the pmi8998 gpio chip absent on z17?

Found
: z17 once from slot b (fastboot boot now; gadget 1d6b:0104 at 18 s), read only over the network shell (telnet 172.16.43.1). Nothing written but debugfs mount.
Facts: the pmi8998 gpio chip EXISTS: gpiochip2 = 800f000.spmi:pmic@2:gpio@c000, 14 GPIOs (gpiochip1 = pm8998 pmic@0, 26 GPIOs; gpiochip0 = tlmm 150). spmi devices 0-00..0-03 all bound to pmic-spmi. dmesg: 'qcom,pm8998@0' and 'qcom,pmi8998@3' nodes 'err -22 does not have reg' (stray vendor-style nodes; the real nodes are pmic@N). The touch regulator's gpios = <0x60 2 0>, and phandle 0x60 is pmi8998_gpios (pmic@2 gpio@c000), i.e. the existing chip. Yet devices_deferred says 'regulator-touch-power  reg-fixed-voltage: can't get GPIO' and 'i2c 0-0020: supplier regulator-touch-power not ready'. pmic@2 gpio2 reads in/low/normal (not driven). regulators: l6, l28 disabled; touch-power not registered.
So the premise 'chip absent because pmi8998@3 lacks reg' is wrong (confirms builder's falsified premise). New question: why does gpiod_get fail for a chip that exists - probe order (regulator-fixed probed before gpiochip2 and not retried?), or gpio-ranges/pinctrl-less gpiochip lookup, or line 2 not requestable. dmesg for the gpio probe time was not captured (filter lacked timestamps of pmic@2 gpio probe); next read: full dmesg and 'cat /sys/kernel/debug/gpio' after force re-probe is NOT done (would write).
Phone returned to Android via reboot -f (see material). Material: research/walleye-first-fastboot-boot/2026-11-18/z17-spmi-gpio-facts.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-z17-spmi-gpio-facts/


<a id="walleye-z19-built"></a>
### walleye-z19-built

Date
: 2026-11-18

Brief
: z19 built: z17 with mainline's PCIe host deleted and its label a stub, so the stock dtbo overlay writes nothing of ours by name; sha256 761a3736; rebuild identical

Question
: Which nodes of our dtb does the stock dtbo overlay (entry 3) overwrite by name, and what fixes them all?

Found
: tests/host/overlay-collisions.sh merges the control dtbo's entry 3 into a dtb with fdtoverlay and lists every overwritten property: on z17 two (reg, ranges of /soc@0/pcie@1c00000, via the overlay's pcie0 fragment aimed at our label; disabled node); on a dtb built from the dts at main also i2c@c179000 (z16's rename was a dtb edit, never in the dts): ten collisions on two nodes. Fix in the dts: /delete-node/ &pcie0 with its label a stub, blsp1_i2c5 redefined as qup@c179000; z19 = z17 dtb with the pcie change by fdtput (build/build-z19.sh, rebuild identical; images-z19/boot-z19.img sha256 761a3736 on the lab). Test passes on z19 and the dts-built dtb, fails on z17 and main's old dts. Overlay: the control build's official dtbo.img, not the phone's own. CI runs only the test's selftest (dtbo is proprietary); the dts gate is build-all.sh on the lab. Not covered: same-value writes, fdtoverlay's merge standing in for the bootloader's (the z19 try is that check), vendor properties added to our nodes (164, listed not failed).

About
: walleye-first-fastboot-boot


<a id="walleye-touch-gpio-defer-leaves"></a>
### walleye-touch-gpio-defer-leaves

Date
: 2026-11-19

Brief
: Touch power gpio: our specifier is valid; the failure is a persistent -EPROBE_DEFER at one of two leaves (no gpio device matched, or no pinctrl range), not settled by source alone

Question
: Exactly why does gpiod_get of regulator-touch-power fail although the pmi8998 gpiochip2 (14 GPIOs) exists?

Found
: Read from v7.3 source and the built z17 dtb. (1) The specifier is right: dtb gpios <0x60 2 0>; 0x60 = pmic@2/gpio@c000, compatible qcom,pmi8998-gpio, #gpio-cells 2, gpio-ranges <0x60 0 0 14>; pmic_gpio_of_xlate returns 2-1 = offset 1 (PMIC_GPIO_PHYSICAL_OFFSET 1), below 14; debugfs shows gpiochip2 gpio2 as a normal line. Host proof: tests/host/touch-gpio-specifier.sh passes. So no 1-based/label/ngpios error (the driver table says 14 for pmi8998). (2) The failure is -EPROBE_DEFER (-517): devices_deferred lists only deferred probes; the string is fixed.c's dev_err_probe after gpiod_get_optional; dd.c clears the reason at every retry, so it is the last retry's, which came after pmic@2's gpio probe (late_initcall retry and every later bind): not a mere ordering race. (3) Non-defer errnos (-EINVAL from xlate, -EBUSY, pinmux errors) would not be listed: ruled out. (4) Source leaves that return -EPROBE_DEFER with the chip present: (a) of_find_gpio_device_by_xlate finds no gpio_device (its of_node is not the specifier's node, or xlate fails); (b) pinctrl_get_device_gpio_range finds no range for the line and pinctrl_ready_for_gpio_range is false (gpiochip_generic_request); (c) try_module_get on the owner: impossible, PINCTRL_QCOM_SPMI_PMIC=y. Mainline sdm845 users of the same pmi8998 dtsi work, so the source gives no static reason; which of (a)/(b) is not known yet. (5) No pin config is needed first: the line is muxed by the request itself; the overlay's switch-gpio (tlmm 75) is another pin and not on this path. (6) Our kernel has CONFIG_DYNAMIC_DEBUG off, so the gpio core's dev_dbg lines ('cannot find GPIO chip', 'gpiod_request: status -517') never print: why the z17 dmesg names no leaf. Cheapest settle: read-only z-next reads (per-pinctrl gpio-ranges in debugfs, gpiochip2 of_node vs the regulator's phandle) and a diagnostic kernel with dynamic debug and dyndbg on the command line.

Sources
: linux-v7-3-gpiolib-pinctrl

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-z17-spmi-gpio-facts/


<a id="walleye-ufs-android-regs-read2"></a>
### walleye-ufs-android-regs-read2

Date
: 2026-11-18

Brief
: UFS on live Android: ufs_version 0x210, gear 3 two-lane fast rate B, PHY driver qmp_v3; HW_VERSION and PHY registers unreadable without a memory reader

Question
: What are the UFS host and PHY registers and pad state of live Android, read only?

Found
: adb root works. debugfs ufshcd0: host_regs 0x00-0x9c read (CAP 0x1587001f, VER 0x210), power_mode gear 3/3, lanes 2/2, FAST, rate B; show_hba ufs_version 0x210, nutrs 32, nutmrs 8, no auto-hibern8. PHY driver bound: ufs_qcom_phy_qmp_v3 (not v3_660); ICE 3.0.65; rails ufshc 1.8V/2.96V/1.2V, ufsphy 0.88V/1.2V. Pinctrl lists UFS_RESET as pin 153, unclaimed, pad state not shown. qcom/debug-regs reads all zero. No /dev/mem, no /proc/kcore, so UFS_HW_VERSION (0xe0) and the PHY registers cannot be read from Android. Nothing written; dme_local_read not used. A reader needs a kernel module or a boot-from-memory try: an item for the coordinator.

About
: walleye-first-fastboot-boot


<a id="walleye-z20-built"></a>
### walleye-z20-built

Date
: 2026-11-18

Brief
: z20 built: z19 with a diagnostic kernel (dynamic debug, DEBUG_GPIO) and dyndbg for gpiolib, gpiolib-of, pinctrl core and devicetree on the command line; sha256 fe9e0cce; rebuild identical

Question
: Can one boot print which gpio/pinctrl leaf the touch power gpio defers at, and the errno?

Found
: The kernel is z15's tree (linux-phone-oco/linux topic/pmic-wdt-wake-in-suspend e123d3bff) built with build/walleye.config plus build/walleye-diag.config (CONFIG_DYNAMIC_DEBUG=y, CONFIG_DEBUG_GPIO=y; build-kernel.sh takes EXTRA_CONFIG). DEBUG_GPIO also compiles drivers/gpio with -DDEBUG: more boot output from every gpio driver, so z20 differs from z19 in that as well. build/build-z20.sh repacks z19: dtb, ramdisk, header (but sizes and command line) and AVB properties are z19's byte for byte; the command line is z19's plus dyndbg="file drivers/gpio/gpiolib.c +p; file drivers/gpio/gpiolib-of.c +p; file drivers/pinctrl/core.c +p; file drivers/pinctrl/devicetree.c +p; file drivers/pinctrl/qcom/pinctrl-spmi-gpio.c +p". Review finding kept: a dyndbg file query is a whole-string match, so the short path pinctrl/core.c matches nothing - full paths are used and the test checks them. What prints: gpiolib-of prints the errno of a failed gpio lookup (the -517 line); pinctrl core and devicetree print their steps; pinctrl-spmi-gpio has no debug lines (its +p prints nothing) and pinctrl core's defer returns are silent, so if the pinctrl leaf is the suspect and nothing prints, the fallback is the debugfs read (devices_deferred). Test tests/host/gpio-diag-boot.sh fails on z19 and on the first z20 (short paths), passes on this one; build checks ok incl. kernel headroom before the ramdisk; rebuild identical. images-z20/boot-z20.img on the lab. No dts changed. Not covered: the phone's dmesg (the keeper's try).

About
: walleye-first-fastboot-boot


<a id="walleye-z20-tried-touch-gpio-leaf"></a>
### walleye-z20-tried-touch-gpio-leaf

Date
: 2026-11-18

Brief
: z20 once from slot b: touch power defers because its pmi8998 gpio 2 is a shared GPIO served by gpiolib_shared.proxy.0, which never registers as a chip (-EPROBE_DEFER)

Question
: On the diagnostic kernel, which leaf does regulator-touch-power's gpio hit, and with what errno?

Found
: z20 booted once via lpoco lab fastboot boot now; gadget 1d6b:0104 at 18 s; dmesg 803 lines read (material ~/lab-material/walleye/z20-try1). gpiolib-of: parsed 'gpios' of /regulator-touch-power status (0), so the specifier is valid and pmic gpio exists. Then 'gpiolib_shared: Adding machine lookup entry for a shared GPIO for consumer regulator-touch-power, with key gpiolib_shared.proxy.0' and 'cannot find GPIO chip gpiolib_shared.proxy.0, deferring' repeated at every retry (1.04 s to 1.63 s+). The proxy was created at 0.670 for 800f000.spmi:pmic@2:gpio@c000 ('GPIO 2 owned by ... is shared by multiple consumers', after 'GPIO 2 at gpio@c000 is shared by multiple firmware nodes' at 0.0967): the leaf is neither no-gpio-device nor no-pinctrl-range, but a third: pmi8998 gpio 2 is claimed by more than one firmware node, so gpiolib_shared routes the consumer to an auxiliary proxy chip that never registers. Next: find the other consumer(s) of pmi8998 gpio 2 in the dts/dtbo overlay and the proxy driver's probe (gpio-shared-proxy), then remove the second claimant or make it a real shared case. Phone returned to Android slot b by the init's own 180 s fall-back; fastboot state: current b, a and b successful, none unbootable; fastboot reboot to active slot; nothing flashed.

About
: walleye-first-fastboot-boot


<a id="walleye-touch-android-compare-result"></a>
### walleye-touch-android-compare-result

Date
: 2026-11-18

Brief
: Touch, Android vs z20: Android's synaptics node (i2c, power-gpio pmi8998 gpio 2, reset tlmm 89, irq 125) is absent in ours; pmi gpio 2 is out-high on Android, in-low on ours

Question
: What does live Android set for touch that our kernel does not?

Found
: Captured with part-compare.sh (touch list) on live Android (adb root) and on z20 (fe9e0cce, network shell, one try from slot b, fell back to Android b by itself at 180 s; nothing flashed). Material ~/lab-material/walleye/compare-touch-{android-20261119-004949,ours-20261119-005129,diff.txt}, never in git. Findings: (1) Android's i2c bus carries a synaptics_dsx@20 child (irq-gpio tlmm 125, reset-gpio tlmm 89, switch-gpio tlmm 75, power-gpio pmi8998 gpio 2 with power-on-state 1, reset delay 20 ms, two pinctrl states active/suspend); input device synaptics_dsxv26 is up. Ours has no such child and no touch input device (only pwrkey, gpio-keys). (2) Pad state of the power line: Android, the pmi8998 gpio claimed by the touch driver reads out/high; ours pmi8998 gpio2 reads in/low, pull-down (no owner). (3) i2c bus pinctrl differs only in phandle numbers; its gpios (87/88) agree. (4) Regulator and clk diffs are the whole-SoC list, not touch specific; no touch regulator is enabled first on Android (tps61099-vbus disabled on both). Reading: the missing pieces are the node itself and the shared pmi8998 gpio 2 owner (z20's proxy never registers), consistent with research walleye-z20-tried-touch-gpio-leaf. Next: build item walleye-touch-gpio-shared-proxy, then add the synaptics node with these gpios. Phone left on Android slot b; fastboot flags a and b successful.

About
: walleye-first-fastboot-boot


<a id="walleye-z21-built"></a>
### walleye-z21-built

Date
: 2026-11-19

Brief
: Second claimant of pmi8998 gpio 2 found (overlay synaptics_dsx@20 under stub i2c_5); fix = /vendor-overlay-stubs disabled; a UFS-on z21 was built and is NOT to boot

Question
: Who else claims pmi8998 gpio 2 so that gpiolib_shared.proxy.0 is needed, and can our dtb stop it without touching the phone's own overlay?

Found
: After the stock dtbo overlay (entry 3) merges, two nodes name the gpio by a gpio property: our /regulator-touch-power and the overlay's synaptics_dsx@20 (synaptics,power-gpio <pmi8998 gpio 2>), which the overlay puts under the label i2c_5 - an empty stub of ours inside /vendor-overlay-stubs. drivers/gpio/gpiolib-shared.c (gpio_shared_of_traverse) skips a node whose status is not okay together with its children, and makes a gpio shared when two nodes name it; the proxy never registered on z20. The overlay cannot be edited, but its place can: status = disabled on /vendor-overlay-stubs (build/gen-overlay-stubs.sh and the dtsi, landed). tests/host/touch-gpio-single-claimant.sh merges the overlay and counts available claimants: z19/z20 dtb 2 (fail, seen first), with the change 1 (pass). build/build-z21.sh repacks z20 with that one dtb change (all checks ok, rebuild identical, sha256 58994599) - but z20 has UFS on, so that image is moved to NOT-TO-BOOT-ufs-on-images-z21 and never queued. The z21 to try is the one walleye-ufs-off-gate builds (z20 with UFS off) with this stub change in the dtsi. Programmer review: kernel reading correct; test narrowed to the kernel's property names. Not covered: the phone's dmesg.

About
: walleye-first-fastboot-boot


<a id="walleye-ufs-on-by-accident"></a>
### walleye-ufs-on-by-accident

Date
: 2026-11-18

Brief
: UFS was enabled in every image since z11 and probed at least four times against the author's 'Yes, wait'; the card and the bootloader survived

Question
: Did our images probe the UFS card after the author's 'Yes, wait', and what happened to the card?

Found
: Found 2026-11-19 by the upstreams inventory, confirmed read-only: our shared dts (dt/msm8998-google-walleye.dts) has had &ufshc and &ufsphy status = "okay" since the z11 commit (lab PR 274, 'ufs readonly z11'); every image built from main after that carries the UFS host enabled, with the UFS driver built into the kernel. The author's answer 'Yes, wait' (decision 2026-11-18-walleye-ufs-wait) was meant to keep z11 and z13 unrun, and the lane held those two tries: but the same dts line enabled UFS in the later images, which the lane did try (z14, z15, z16, z17, z19?, z20 and the shell captures): the captured dmesgs of z15, z16, z17 and z20 (lab-material z15-part1, z16-part1, z17-part1, z20-try1) show 'scsi host0: ufshcd', the SKhynix H28U74301AMR D003 enumerated (well-known LUN), and for z15/z16/z17 the other LUNs attached as sda..sdf with the partition scans ('Attached SCSI disk' lines): i.e. our kernel brought the link up and READ the card at least four times, unattended, on stock v7.3 quirks only (no z13 backport in those images: z15+ came from the touch lane's kernel; check the exact config per image), against the author's wait. Nothing was written to the card by us (no mount, read-only init), the phone still boots Android from both slots (adb device, a/b successful) and the bootloader is intact: that is REAL EVIDENCE for the risk question (the yoshino erase did not happen on this card with the plain probe: at least 4 captured probes, probably more uncaptured), and not a licence: it does not show the quirk-less probe is safe in every sequence (the erase cause is unpublished). How it happened: the ufs-readonly build item put the enable into the shared dts instead of a z11-only dts, and no check said 'UFS stays off until the author says go'; the keepers booted images by the try items, not by what their dtb enables. The author's 'Yes, wait' is withdrawn from the keepers' hands now: UFS off in every image, a host gate, and the author told plainly. ADDENDUM 2026-11-19 (item walleye-ufs-off-gate; tests/host/ufs-off-inventory.sh over ~/Projects/walleye-build/images*/boot-*.img, 97 images, z21 included): 58 are NOT TO BOOT, each marked by a NOT-TO-BOOT-IMAGE.txt beside it: 49 have UFS on, 9 (images/boot-a..i, built before the dtb carried symbols) cannot be shown off because the phone's overlay does not merge into their dtb. By images directory: images-pk456: pk5 pk6; images-z20: z20; images-z11: z11; images-vfx: vfa vfb vfc vfd; images-z12: z12; images-w: w1 w2 w3 w4 w5 wa wr; images-pk5l: pk5l; images-z13: z13; images-x: x1 x2 x3 x4 x5 x7; images: a b c d e f g h i; images-pk5l2: pk5l2; images-z15: z15; images-z16: z16; images-z17: z17; images-vf1: vf1; images-z19: z19; images-vf2: vf2; images-vf3: vf3; images-t1: t1; images-pk2: pk1 pk2; images-wp: wp1 wp2 wp3 wp4 wp5 wpa; images-vf4: vf4; images-nocon: pk3; images-t3: t3; images-pk: pk1 pk2; images-g: g1 g2 g3;  The stock-kernel vendor images (vf1-vf4, vfa-vfd) are on through the vendor dtb merged with the phone's own overlay; every other UFS-on one carries a mainline-style dtb (/soc@0/ufshc@1da4000) with status okay, from z11's shared dts on. 39 are off, among them the new z21. After this: dt/msm8998-google-walleye.dts has &ufshc and &ufsphy disabled, the z11-z13 experiments take dt/msm8998-google-walleye-ufs.dts, and tests/host/ufs-off-image.sh gates every build script that writes a boot image (a stock-kernel repack fails it by design until it has a UFS-off dtb).

About
: walleye-first-fastboot-boot


<a id="walleye-z21-tried-touch-binds"></a>
### walleye-z21-tried-touch-binds

Date
: 2026-11-19

Brief
: z21 once from slot b: no gpiolib_shared proxy deferral; rmi4 0-0020 binds, Synaptics S3708AR input registered; UFS off; phone back on Android slot b

Question
: With UFS off and the stub container disabled, does regulator-touch-power probe and rmi4 0-0020 bind?

Found
: ufs-off-image.sh passed on boot-z21.img right before the boot (sha256 matched SHA256SUMS-z21); battery 4375 mV. lpoco lab fastboot boot now: left fastboot 15 s, gadget 1d6b:0104 at 18 s, shell read by mainline-shell-read.sh (16 answers, dmesg 622 lines; material ~/lab-material/walleye/z21-try1). dmesg: 0 lines 'cannot find GPIO chip gpiolib_shared'; regulator-touch-power parses gpios status 0 at 1.017; rmi4_i2c 0-0020 registers at 1.206, 'found RMI device, manufacturer: Synaptics, product: S3708AR, fw id: 2520908', input: Synaptics S3708AR as .../0-0020/rmi4-00/input/input1. So the single-claimant fix (stub container disabled) is the cause fix for the touch power defer. Not done: tests/host/part-compare.sh capture touch ours (the reader's reboot -f ended the shell; needs a try with NOREBOOT=1 - a second z21 try, left to the coordinator) and touch-check.sh events (nobody touched the panel; skip). Phone returned to Android slot b by the reader's reboot -f; adb up, 4381 mV, nothing flashed.

About
: walleye-first-fastboot-boot


<a id="walleye-z21-touch-part-compare"></a>
### walleye-z21-touch-part-compare

Date
: 2026-11-19

Brief
: z21 second try: touch part-compare captured on ours (10 answers) and diffed against the Android capture; touch-power regulator enabled 3008 mV; phone back on Android slot b

Question
: On z21, how does the touch part's state differ from live Android?

Found
: z21 booted again from slot b (battery 4385 mV; ufs-off-image.sh passed right before; network shell 172.16.43.1 up, telnet). part-compare.sh capture touch ours: 10 answers in ~/lab-material/walleye/compare-touch-ours-20261119-021653; diff against compare-touch-android-20261119-004949 saved as ~/lab-material/walleye/compare-touch-diff-z21.txt (941 lines; the clk section reports the Android format as unmatched, so its 58 'we have' rows are not a finding). Seen: ours has regulator touch-power enabled at 3008 mV and the rmi4 device bound (previous research walleye-z21-tried-touch-binds). The gpio/pinconf lines for pmi8998 gpio 2 differ between the two captures (ours gpio.txt carries two 'gpio2' rows, in-low and out-high, from different controllers; which is the pmi8998 one is NOT settled here) - read the diff gpio section before drawing a conclusion. Touch events not tested (nobody touched). Phone fell back to Android slot b by the init's own timer; adb up, 4379 mV, nothing flashed.

About
: walleye-first-fastboot-boot


<a id="walleye-z21-touch-events-nobody-swiped"></a>
### walleye-z21-touch-events-nobody-swiped

Date
: 2026-11-19

Brief
: z21 third try: Synaptics up at 107 s, reader ran 20 s, nobody swiped; touch events not known

Question
: On z21, does the RMI4 node give EV_ABS events when a person touches the glass?

Found
: ufs-off gate passed; booted from Android slot b; Synaptics S3708AR (event1) listed at uptime 107 s; init reboots at 180 s. touch-read NOREBOOT=1 read 20 s: 1 line; touch-check skip (not known). My ready line went through SendMessage, which cannot reach another session, so no swipe. Phone back on Android slot b, adb up, nothing flashed. Next: send ready at once via send_message with the author already watching, or SECONDS_TO_READ=40.

About
: walleye-first-fastboot-boot


<a id="walleye-z21-touch-events-window-missed"></a>
### walleye-z21-touch-events-window-missed

Date
: 2026-11-19

Brief
: z21 third try: Synaptics up at 107 s; the author DID swipe but the 20 s reader had closed; touch events not known yet (the window was missed, not failed)

Question
: On z21, does the RMI4 node give EV_ABS events when a person touches the glass?

Found
: ufs-off gate passed; booted from Android slot b; Synaptics S3708AR (event1) listed at uptime 107 s; init reboots at 180 s. touch-read NOREBOOT=1 read 20 s (02:37:20-02:37:44): 1 line; touch-check skip. The keeper's ready reached the coordinator only after the read had closed; the author swiped and tapped the corners when the coordinator relayed READY NOW (his words: i did swiped and tapped corners as i could), so the swipe fell after the reader. The first record said nobody swiped: wrong, corrected here. Result: touch events NOT KNOWN YET, not failed. Retry recipe: start the event reader first (90 s, well past the swipe, or touch /tmp/hold and read 150 s), send ready the moment the device shows, go window of at least 90 minutes, only at the author's next time at the phone. Phone is back on Android slot b, nothing flashed.

About
: walleye-first-fastboot-boot

Supersedes
: walleye-z21-touch-events-nobody-swiped


<a id="walleye-display-s6e3fa5-built"></a>
### walleye-display-s6e3fa5-built

Date
: 2026-11-19

Brief
: Display kernel and image built: the S6E3FA5 driver compiles with the Pixel 2's own init read from the phone's dtbo; boot-display.img passes the UFS-off gate; not lit yet

Question
: Does the S6E3FA5 panel driver with the DPU/DSI path build into a boot image the keeper can try?

Found
: Earlier unlanded driver had lost its reset and on functions. The phone's own dtbo (entry 3, node mdss_dsi_samsung_s6e3fa5_1080p_cmd) gives the facts: on = sleep out, 120 ms, TE on, 0x53=0x20, 0xf0 unlock, 0xbb 54 00 36 20 f3, 0xf0 lock, display on; off = display off, 10 ms, sleep in, 150 ms; reset high 1 ms, low 1 ms, high 10 ms; porches h 18/10/82, v 8/2/16; 62x110 mm; vddio 1.8 V; backlight by 8-bit DCS 0x51. Programmer review: brightness changed to a one-byte write, the 16-bit read dropped, build script paths made absolute; its uncompressed-headroom check rejected (z21 boots at 43.4 MB uncompressed with the ramdisk at 16 MB). build-display.sh builds Image.gz and the display dtb (rc 0); build-display-image.sh puts them into z21's image, every check ok incl. the UFS-off gate. boot-display.img sha256 21a1134ebf99aca90fb77042697e40fa7a7b032734d706b8d2328c5b08bd09b0. NOT lit: untried. The kernel config is walleye.config plus walleye-display.config, not z21's diagnostic fragment.

About
: walleye-first-fastboot-boot


<a id="walleye-pmic-wdt-v1-images-built"></a>
### walleye-pmic-wdt-v1-images-built

Date
: 2026-11-19

Brief
: PMIC watchdog v1 images built: A on next-20261118 + the six v1 patches, B on v7.3 + topic 8b2283da3; init-wdt1 is unarmed at boot, wdt-bite and wdt-reboot arm; UFS off, rebuilds identical

Question
: Can one visit try the PMIC watchdog v1 (bite, stop-on-reboot) with a fallback image, from images that pass the UFS-off gate and arm nothing at boot?

Found
: Two boot images, same ramdisk (init-wdt1), same dtb (A and B build the identical walleye dtb), header, size and AVB footer as z21; only the kernel differs. A = linux-next next-20261118 (e8017d290, fetched here as a shallow tag; the upstreams lane's own tree is not on this machine) with the six patches of docs research/upstream-forks/2026-11-19/pmic-wdt-gen2-v1/ applied by git am, all six applied clean, head a27d60e20: so the mailed series is what boots. B = v7.3 + linux-phone-oco/linux topic/pmic-wdt-wake-in-suspend at 8b2283da3 (patches 1 and 2 = f5bb57d27, 8b2283da3, same code as v1 1 and 2). Both: arm64 defconfig + walleye.config + walleye-wdt.config (CONFIG_DYNAMIC_DEBUG only, no DEBUG_GPIO), our dts with &pm8998_wdt okay, UFS disabled, command line = z21's with its gpio dyndbg replaced by dyndbg=file drivers/watchdog/pm8916_wdt.c +p, so the banner carries the driver's two dev_dbg lines (S2 reset control was X = the firmware's type; POFF reason, printed before the type is set; the PMIC device in sysfs shows the probe passed). Init: z14 (ACM+ECM, usb0 172.16.43.1, DHCP, telnetd, command loop) with the SoC watchdog petter now found by sysfs identity qcom_wdt (never /dev/watchdog, which with the PMIC watchdog on may be the PMIC one and opening it arms it), the 180 s self-reboot now waiting while /tmp/hold exists, WALLEYE-BANNER-PON and WALLEYE-BANNER-WDT lines each beat, and two commands: wdt-bite [TYPE] (busybox watchdog -F -T 20 -t 3600 on the PMIC device: armed, one pet at the arming, then none) and wdt-reboot [TYPE] (same arming, then sync and reboot -f = reboot(2), kernel_restart, the reboot notifiers), both refused unless the PMIC device exists and S2 type reads back as TYPE (default 7). DEVIATION from the item text: the driver (v7.3 topic and v1 alike) always sets RESET_TYPE_HARD 0x07 and the dts has no property to choose a type, so 'warm 0x01 first' cannot be asked of this driver; the guard compares the read-back with 7 (the type the driver sets); a read-back of 1 means the driver did not set it and is refused unless asked for as wdt-bite 1. FINDING for the display image: tests/host/overlay-clock-gcc.sh fails on a dtb straight from build-kernel.sh and on images-display's dtb (the four overlay clock users name the empty stub, clocks[0]=321 want 36): /__symbols__/clock_gcc must be pointed at the real gcc after the build (z3's fdtput); build-pmic-wdt-v1.sh does it and checks, build-display-image.sh does not. Checks (build-pmic-wdt-v1.sh, all ok, nothing written on a FAIL): command line, busybox applets, init syntax, host selftest tests/host/pmic-wdt-v1-init-selftest.sh (seen failing 19 checks on init-z14, passing on init-wdt1), serials, packing equals z21's ramdisk from init-z14, per image 37 checks (87 ok in the whole run, 0 FAIL) (configs, wdt node qcom,pm8998-wdt okay, UFS disabled, kpss-wdt, blob, ramdisk, command line, header fields, size, headroom, AVB verify, the UFS gate build/ufs-gate.sh, and the seven dtb tests touch-gpio-single-claimant, overlay-collisions, dtbo-overlay-merges, overlay-clock-gcc, i2c-qup-after-merge, touch-power-after-merge, adreno-smmu-disabled). Rebuilt a second time: both images byte for byte equal. Not run: a QEMU run of the init (build/selftest-initramfs.sh) and any phone. Images (lab machine, ~/Projects/walleye-build/images-wdt1/, sha256): boot-wdt1-a.img 6c1dc855cc6777abf51a38dbc92dc1279396187a2bbefef42bb904118211acf8; boot-wdt1-b.img 5ff05b4f299c081567d2bc3511c5fbc0692180cb2c5be12430939f4b61f198cb; Image-a.gz 31cecab2abea68a47c0ff25eec59c99d6eac95c3062a6a90f25f2fbaddf16037; Image-b.gz 5fd9772018032bff545a696d6c4926aa940e0c3187c557030aaa7897b734bce3 (SHA256SUMS-wdt1 there). Kernel trees: ~/Projects/walleye-build/wt-k-next (A), wt-k-pmicb (B); outputs out-wdt-a, out-wdt-b. Reproduce: EXTRA_CONFIG=build/walleye-wdt.config build/build-kernel.sh TREE OUT for each, then build/build-pmic-wdt-v1.sh ~/Projects/walleye-build images-z21/boot-z21.img out-wdt-a out-wdt-b images-wdt1.

REVIEW (programmer subagent, 2026-11-19) found and these are fixed in the images above: (1) wdt-arm printed 'armed' without reading it back: now the applet must be alive (read from /proc state, since a dead unreaped child is a zombie that kill -0 still accepts: found by the new test) and S2 must read enabled afterwards, else FAILED, the applet is stopped, and wdt-reboot does not reboot (so a next banner with S2 disabled is a visible change, not an arming that never happened); (2) the 180 s self-reboot could fire between a typed wdt-bite and the bite: wdt-arm sets /tmp/hold for 90 s (only when none is set); (3) the selftest raced a backgrounded fake: now the fake writes before the fake sleep 1 returns, and a hold someone set is tested not to be removed; (4) the petter's choice of the SoC watchdog is now tested by behaviour with the PMIC watchdog at watchdog0 or watchdog1, with the SoC node missing, and with only the PMIC present (5 mutations of the init, each caught); (5) the petter step is logged when the open succeeded, not when it was only meant; (6) wdt-state prints src, the regmap the registers came from; (7) the build script checks the kernels: NOWAYOUT unset, the Linux version line (next-20261118 for A, g8b2283da3 for B), the driver's read-back string and the PM8998 compatible inside the kernel. Also found while fixing: the selftest's first expectation had hidden a bug (the banner-safe tr turned the line's newline into a question mark).
NOT PROVEN: the init's pon-regs reader has never been shown to read on the phone (z10's reader printed 'not read yet' for 60 s, research walleye-z10-tried-pon-regs-not-read; this init carries the rewritten reader of z11, never tried). If the banner shows s2-type unread, wdt-bite and wdt-reboot refuse; wdt-state says whether the PMIC device is registered (the probe passed, so the driver's own read-back equalled 0x07), and 'wdt-bite 7 probe' or 'wdt-reboot 7 probe' arms on that read-back instead, then S2 enabled after the arming cannot be confirmed by the init (it says after=unread).

About
: walleye-pmic-wdt-proof


<a id="walleye-generated-init-gaps"></a>
### walleye-generated-init-gaps

Date
: 2026-11-19

Brief
: lpoco-bringup 0.24.0 generated init (console acm, --net 43) against our z14/wdt1 init: serial, ACM+ECM, usb0, DHCP, telnetd, MACs covered; four things missing

Question
: Does the generated bring-up init cover what our own init (init-z14, init-wdt1) keeps, so the ~20 variants can be retired?

Found
: Generated from device.toml [boot.gadget] with --net 43: COVERED - serial walleye-first-boot from the manifest, configfs ACM + ECM in c.1, ECM MACs 02:42:ac:10:2b:01/02 identical to ours, usb0 172.16.43.1 with DHCP 172.16.43.2, telnetd, UDC wait, self-test. MISSING - (1) the SoC watchdog petter and the 180 s self-reboot (reboot -f, sysrq b) that waits while /tmp/hold exists; (2) the raw command loop on ttyGS0 (generated: an interactive sh respawned on ttyGS0, which research walleye-z5b-tried-raw-tty found silent on the raw tty); (3) the WALLEYE-BANNER lines on ttyGS0 and the step log /tmp/banner-steps; (4) a hook for image-specific additions (wdt-arm, wdt-bite, wdt-reboot are written into the ramdisk by init-wdt1). Decision: keep our own init; [boot.gadget] is declared in device.toml so the tool reads it; the four gaps go to orchestration.

About
: walleye-first-fastboot-boot


<a id="walleye-upstream-dts-series"></a>
### walleye-upstream-dts-series

Date
: 2026-11-19

Brief
: Pixel 2 board dts as a two-patch upstream series on v7.3: dt_binding_check, W=1 dtbs, relaxed style clean; strict 76 findings to 2; checkpatch 0 errors

Question
: Can the candidate dts of docs #573 become a clean upstream series, UFS off, and what is left

Found
: Yes: binding patch plus board dts, reviewed. APSS wdt dropped, adreno_smmu kept disabled, UFS off. Open before mailing: author and SoB, carve-out and ramoops provenance, one boot of this exact dtb.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-19-upstream-dts/


<a id="walleye-upstream-dts-image-built"></a>
### walleye-upstream-dts-image-built

Date
: 2026-11-19

Brief
: Image of the exact upstream board dts built: z21 with the series' dtb (+ overlay stubs, 2 nodes deleted by a wrapper); gates pass, rebuild identical; not tried

Question
: Can an image be built from the series' msm8998-google-walleye.dts unedited, passing the UFS-off and overlay gates?

Found
: boot-upstream-dts.img = z21 (kernel, ramdisk, header, command line, AVB footer byte for byte) with the dtb of the series' msm8998-google-walleye.dts, unedited, on v7.3. sha256 a8e4364813f3c82a35b9e5edcffb6650eac710e780119b27fa32dc2fa9521377; dtb 76467d10a9f710779f60c47ed93deabb56b07362309f9d25c7068e90fbab7f20; on the lab host in ~/Projects/walleye-build/images-upstream-dts/; built twice, same bytes. Gates pass: ufs-off-image, overlay-collisions, dtbo-overlay-merges, overlay-clock-gcc, adreno-smmu-disabled, AVB verify. Findings for the cover letter: (1) the series' dtb alone does not merge with the phone's dtbo overlay (FDT_ERR_NOTFOUND; 137 labels targeted, the board has 11), so 126 empty stub nodes are added by a wrapper, generated by gen-overlay-stubs.sh; (2) the overlay overwrites two disabled nodes of msm8998.dtsi by name, pcie@1c00000 and i2c@c179000 (11 collisions), the wrapper deletes both as the lab dts does; (3) nothing missing to reach USB: the series has gpio-reserved-ranges, UFS off, uart, ramoops. No touch expected. Not tried.

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-19-upstream-dts-image/


<a id="walleye-upstream-dts-tried-boots"></a>
### walleye-upstream-dts-tried-boots

Date
: 2026-11-19

Brief
: Upstream board dts image tried once from slot b: boots to the usb0 network shell (gadget, ping, telnet witnesses); phone back on Android b

Question
: Does the exact series dtb boot to the network shell?

Found
: boot-upstream-dts.img (sha256 a8e43648...) tried once from slot b by `lpoco lab fastboot boot now` after the UFS-off gate passed (pass) and battery 4373 mV. Booted: left fastboot after 15 s, came back as gadget 1d6b:0104 after 18 s; witnesses: phone-state.sh verdict mainline-answering (usb gadget + tty banner), ping 172.16.43.1 answered, telnet shell gave `uname -a` = Linux 7.3.0-00004-ge123d3bff47d-dirty. dmesg (604 lines, ~/lab-material/walleye/upstream-dts-20261119-081603/dmesg.txt): gadget g1 bound to a800000.usb, ECM MACs set. Only fault-class lines: psci 'failed to set PC mode: -3' (firmware bug note), msm_serial c16f000.serial probe error -2 (the known vendor-overlay serial stub), spmi pm8998@0/pmi8998@3 'does not have reg property' (-22). No -110 timeouts, no phandle errors. No touch (series has none), as expected. Not done: a probe-line diff against z21 - the saved z21 dmesg (compare-touch-ours-20261119-021653) is truncated (10 unique lines), so no diff was drawn; the dmesg-clean-check was not run (needs devices-deferred, not read). Phone fell back to Android slot b by its own 180 s timer; idle-android pass; 4369 mV; nothing flashed.

About
: walleye-upstream-dts-polish


<a id="walleye-boot-image-align-is-the-bootloaders"></a>
### walleye-boot-image-align-is-the-bootloaders

Date
: 2026-11-19

Brief
: Kernel image misaligned warning: every walleye boot image uses the stock kernel offset 0x8000; the bootloader places the Image, not our header - recorded as the bootloader's, not changed

Question
: Is the '[Firmware Bug] Kernel image misaligned at boot' line in z6/z7 dmesg ours to fix through the boot image kernel offset?

Found
: All our boot images (build-bootimg.sh a-i, build-*.sh) set base 0 and kernel_offset 0x8000, the value of taimen's deviceinfo, LineageOS wahoo's default and the stock boot.img (research walleye-bootimage-header-public, how-others-booted-wahoo). The Android bootloader loads the kernel at RAM base + offset = 0x80008000, 32 KiB past a 2 MiB boundary, which arm64 reports once at line 7 of dmesg. The kernel then boots, KASLR is on and nothing downstream warns (z7: dmesg-clean-check passes). A 2 MiB-aligned offset (0x200000) would move the load address on a bootloader that is not read and could overlap the ramdisk at 0x81000000 with a 17 MiB Image: a change that risks the only booting path to silence one cosmetic line. So: recorded as the bootloader's; no image change; if ever wanted it is one keeper try (kernel_offset 0x200000, ramdisk offset moved clear of the Image).

Sources
: walleye-bootimage-header-public

About
: walleye-first-fastboot-boot


<a id="walleye-series-reproduce-the-bisects"></a>
### walleye-series-reproduce-the-bisects

Date
: 2026-11-19

Brief
: lpoco-bringup 0.24.0 image bisect rebuilt all 35 recorded bisect images (s*, pk4-6, d1-6, w*, wp*, x*) byte for byte from six series files

Question
: Can lpoco-bringup image bisect SERIES reproduce walleye's recorded bisect images, so the old shell scripts can go?

Found
: Six series files in build/series (stock-to-ours, pk456, dtb-bisect, taimen-walleye, property-bisect, fdt-bisect) declare the control and one swapped part per image. Run on the lab host, every image's sha256 equals the recorded SHA256SUMS: s0 s0n s1 s2 s3 (5), pk4 pk5 pk6 (3), d1-d6 (6), w1-w5 wr wa (7), wp1-wp5 wpa (6), x1-x7 xa (8): 35 of 35, each counted ok by the command. One slip on the way: lpoco-bringup image unpack leaves the appended dtb inside the kernel file, so a kernel swap needs Image.gz = kernel less appended.dtb (the first pk5 differed until so); the s2 kernel is the raw Image in an lz4 frame. The dtb-making scripts (fdtput node edits) are not swaps and stay.

About
: walleye-first-fastboot-boot


<a id="walleye-fuel-gauge-built"></a>
### walleye-fuel-gauge-built

Date
: 2026-11-19

Brief
: fg1 built: v7.3 + the RFC's binding, driver and pmi8998 node, our dts enabling the node; 37 checks ok, rebuild identical; not tried

Question
: Can the PMI8998 FG gen3 driver of the RFC of 2025-11-24 be built into a gated Pixel 2 image?

Found
: Patches 1-3 of 8 (binding, pmi8998_fg.c, pmi8998.dtsi node) apply cleanly with git am -3 on v7.3 + pmic-wdt topic (e123d3bff); the other five are other boards' dts. Driver reads the gauge's own SOC, voltage, current and temperature (monitored-battery is in the binding, unused by the driver; the vendor's fg-profile-data stays in the PMIC). Probe writes only IMA interface config and clears a DMA error bit. Two things found: our generated overlay-stubs dtsi had an empty pmi8998_fg stub and the build died on a duplicate label - the stub is dropped (the vendor overlay's fragment 137 now merges its qcom,fg-* properties onto the real node; the driver ignores them); the series' soc-delta irq type is 0x3 (rising edge) in the dtsi but 0x4 in the binding example - not ours to settle, to be asked upstream. Image boot-fg1.img sha256 9f6a7033ae13e532b91874bb90e8a08803172ccfa552bbe6eb53d7f07086b99b: z21's ramdisk, header, size and AVB footer, command line with dyndbg for pmi8998_fg.c; AVB, UFS-off gate and the seven dtb tests pass; rebuilt twice, same hash.

Sources
: lwn-pmi8998-fg-rfc

About
: walleye-first-fastboot-boot


<a id="walleye-fg1-tried-capacity-matches"></a>
### walleye-fg1-tried-capacity-matches

Date
: 2026-11-19

Brief
: fg1 once from slot b: pmi8998-fg probes, qcom-battery capacity 99 %, voltage_now 4374 mV = fastboot's 4374 mV; phone back on Android slot b

Question
: Does the RFC's PMI8998 fuel gauge probe on the Pixel 2 and read the battery as fastboot does?

Found
: Battery rule passed (4373 mV adb, 4374 fastboot, cable in both); ufs-off-image.sh passed on boot-fg1.img, sha256 matched SHA256SUMS-fg1. fastboot boot now: left fastboot 15 s, gadget 1d6b:0104 at 18 s, mainline-shell-read.sh NOREBOOT=1 16 answers, dmesg 367 lines (material ~/lab-material/walleye/fg1-try1). fuel-gauge-check.sh: pass, qcom-battery type Battery, capacity 99, voltage_now 4374500 uV, 0 mV from fastboot, status Full, current_now 9765, temp 298. dmesg: pmi8998-fg fuel-gauge@4000 probed, no failures. Same cable state for both readings. Then reboot -f over the tty; Android back on slot b, adb up. Nothing flashed. Noted: online/health/usb_type files absent on qcom-battery (not the gauge's).

About
: walleye-first-fastboot-boot


<a id="walleye-ufs-regs-reader-verdict"></a>
### walleye-ufs-regs-reader-verdict

Date
: 2026-11-19

Brief
: UFS_HW_VERSION is at 0xE4 (not 0xE0), is 3.0.0 or 3.1.0 for certain, and has no read-only path on live Android: no module, read it in our own driver at the matched try

Question
: How do we read UFS_HW_VERSION and the PHY registers on live Android without /dev/mem: debugfs, a read-only module, or skip?

Found
: How to learn the UFS host's hardware version (and so the PHY table) for the Pixel 2 without /dev/mem on its live Android: the options, read from Google's published wahoo 4.4 kernel (branch android-msm-wahoo-4.4-android11, head 52750b8f, facts only, nothing kept). 2026-11-19, walleye-research. Nothing was run on the phone.

## Facts from the vendor source
- **The register is at 0xE4, not 0xE0.** `REG_UFS_HW_VERSION = 0xE4` in the host's vendor block (0xC0-0xFC: SYS1CLK_1US C0, TX_SYMBOL_CLK C4, LOCAL_PORT_ID C8, PA_ERR_CODE CC, RETRY_TIMER D0, PA_LINK_STARTUP_TIMER D8, CFG1 DC, CFG2 E0, HW_VERSION E4, TEST_BUS E8..., UNIPRO_CFG F8, AH8_CFG FC). Earlier rows (walleye-ufs-android-regs-read2, walleye-ufs-android-way-compared, the backlog proof) say 0xe0: that is CFG2. Format: major bits 31:28, minor 27:16, step 15:0.
- **The vendor PHY driver accepts only two versions.** `ufs_qcom_phy_qmp_v3_phy_calibrate` picks the rate-A table by (major, minor, step) = 3.0.0 or 3.1.0; any other value returns -ENODEV ("Unknown UFS-PHY version ... no calibration values") and the UFS would not come up. Live Android's UFS works with `ufs_qcom_phy_qmp_v3` bound, so the host is 3.0.0 or 3.1.0 - known, not guessed. Which one is one word of the register.
- **Where the vendor prints the vendor block.** debugfs `ufshcd0/host_regs` dumps only the UFSHCI space (UFSHCI_REG_SPACE_SIZE, 0x00-0x9c). The vendor block (16 words from 0xC0, so 0xE4 is inside) is dumped by `ufs_qcom_dump_dbg_regs`, called only from `ufshcd_print_host_regs` on error paths (timeouts, resets, fatal errors). `qcom/dbg_regs` (debugfs) prints the HW debug registers, not 0xC0-0xFC (measured: all zero). No debugfs file reads an arbitrary register. `dme_local_read` reads UniPro attributes, not the host block.
- **The only debugfs path that makes the dump appear is `ufshcd0/reset_controller`**: it fakes a fatal error and resets the host controller. That writes, resets the link of the disk Android runs on, and is not read-only.
- The version is not printed at boot (no dev_info), and Lineage's 4.4.302 fork was not read: a Lineage-only knob is not known.
- `androidboot.msm.hw_ver_id=2005E0E1` on the command line is the SoC id, not the UFS host version.
- Addresses from the running fdt: host `ufshc@1da4000`, PHY `phy@1da7000`.

## The options
1. **Vendor debugfs / sysfs knobs** - exhausted. host_regs and qcom/dbg_regs are read (research walleye-ufs-android-regs-read2); neither reaches 0xE4; reset_controller is a write. Result: cannot read the version this way.
2. **A read-only GPL kernel module** (insmod by adb root, ioremap 0x1da4000+0xE4 and a fixed list of PHY offsets, print, rmmod). Reads of the powered, clocked UFS host by software that runs at the same time as Android's own driver are what Android itself does; but the host's clocks are gated when the disk is idle (clock gating with hibern8 is on), and a read of a gated block does not return: the bus hangs and the phone restarts. Writes nothing; the way back is a reboot; bricking is not at stake. Costs: Lineage 4.4.302 source and config matching the running kernel (vermagic, CONFIG_MODULES and module loading unknown - `/proc/config.gz` not read), a toolchain, SELinux for insmod, and keeping the disk busy so clocks are on (or taking the driver's own ufshcd_hold, if exported - not known). Result: possible; a day of build for one word.
3. **The vendor kernel as instrument (vf)** - no: the same /dev/mem gap, and its UFS host is unclocked until its own driver runs.
4. **Skip the read.** What is lost: only which of two PHY rate-A tables to give our driver (3.1.0: 10 lines differ from mainline's; 3.0.0: 35 lines differ). The version is not needed for the erase question (it is not about analog tuning), not for the RST_n or quirk work, and not for the author's "wait".
5. **Read it in our own driver, at the UFS-on try** (new): the vendor reads it in ufs_qcom_init, after clocks are on and before the PHY is initialised; our driver can do the same read at the same place (a pr_info of REG_UFS_HW_VERSION, and the PHY table chosen by it as the vendor does), so the version is read where Android reads it, by the code that needs it, and no separate try. That is the way the matched image (item walleye-ufs-match-android-z) should carry, and it is why nothing needs to be read before the try.

## Verdict
Do not build a module (2) and do not pulse reset_controller: the loss of skipping (4) is small, and (5) closes it inside work that is queued anyway. The unreadable-now facts are bounded: version in {3.0.0, 3.1.0} (certain), the table choice (open, closed by the first read in our driver), the PHY registers after init (unread; they would show whether silicon matches the table, but a wrong table shows as a failed link startup, not an erase). The safe-before-any-try argument of research walleye-ufs-android-way-compared (a read first) stands for what it can read: host registers 0x00-0x9c (done) and pad state; the version moves into the matched image's own boot.

Sources
: src-google-wahoo-msm44-android11

About
: walleye-first-fastboot-boot

Material
: research/walleye-ufs-regs-reader/2026-11-19/


<a id="walleye-ufs-match-built"></a>
### walleye-ufs-match-built

Date
: 2026-11-19

Brief
: Matched UFS image built, never booted: z21 + kernel topic/ufs-match-android + UFS overlay; quirks, PHY table by version, RST_n pin 153; checks ok, rebuild identical, reviewed

Question
: Can a z image carry Android's UFS path for the H28U74301AMR (SAVECONFIGTIME only, version read at 0xE4, table by it, card RST_n pulse), built and reviewed, not tried?

Found
: NOT-TO-BOOT-ufs-on-ufs-match.img (sha256 f8f6215e60787b63cda7f332cc0ca7a6dc8452009a0bc86ddc54351d36aa4f45) = z21's header, command line, ramdisk and AVB footer, z21's dtb plus dt/msm8998-google-walleye-ufs-match.dtso (10 changed lines, checked exactly), and the Image of ~/Projects/walleye-build/wt-k-ufsm (topic/ufs-match-android, v7.3 + pmic wdt + fuel gauge RFC, diag config). Kernel: the SK hynix H28U74301AMR entry carries SAVECONFIGTIME + NO_PURGE only (ONE_TX_LANE, EXTEND_SYNC_LENGTH, DELAY_BEFORE_LPM gone); ufs-qcom prints UFS_HW_VERSION where it reads it (0xE4) and hands it to the QMP PHY by the new phy_configure_opts_ufs at init and before power on; the MSM8998 PHY has the vendor's 3.0.0 (lane 0 only) and 3.1.0 value lists, equal to the research lists by a compile test, and refuses any other version with -ENODEV (probe stops at init); pinctrl-msm8998 ngpios 154 so reset-gpios <tlmm 153> pulses the card (10-15 us, before HCE, as mainline's ufs_qcom_device_reset). Proof: tests/host/ufs-match-android.sh (source and dtb), its selftest (8 mutants fail), build/build-ufs-match.sh (all checks, rebuild byte-identical), and the UFS-off gate FAILS the image by design. Programmer review found 7 points: the refusal was not a stop (fixed: configure at init), the purge guard leaked the only device-management tag (fixed), weak tests (fixed, mutants), a dirty Image could ship (Image strings checked), the image could land where the inventory looks (named NOT-TO-BOOT-, OUT_DIR must be NOT-TO-BOOT-*), and two comment/scope points (order of the pulse differs from Android's; link-startup pre-change, clock scaling and host caps are NOT matched). Not matched: ICE, bus votes, PM-QoS, hw-gated clocks, pad drive/pull of ufs_reset, link-startup pre-change. Not known: whether the host is 3.0.0 or 3.1.0 (first read is in the try), whether the 3.0.0 lane-0-only table leaves lane 1 usable.

Sources
: src-google-wahoo-msm44-android11

About
: walleye-first-fastboot-boot

Material
: research/walleye-first-fastboot-boot/2026-11-18-ufs-android-way/


<a id="walleye-display-try1-author-saw"></a>
### walleye-display-try1-author-saw

Date
: 2026-11-19

Brief
: Display try 1, what the author saw: Google logo, then black, then Lineage within a minute; the black says the kernel reached the display handover

Question
: What did the panel do during display try 1, and what does it say about where the kernel stopped?

Found
: The author's words (2026-11-19, relayed by the Top Coordinator, 'i think' his): "google logo then a black screen i think then it rebooted into lineage not more than i minute after". So: logo (the bootloader's splash, shown while fastboot boot loads), then black, no stripes, then Android within a minute (host log: 49 s after leaving fastboot). Reading: the bootloader's splash stays on the panel until something takes the display; a kernel that never reaches a display driver would leave the logo up until the reset. Black before the reset therefore points at the kernel having reached the display handover (DPU/DSI probe, a panel reset or a clock or iommu change that blanks the bootloader's scan-out) and then died or hung before the gadget bound - consistent with a fault in the display path, and not with a kernel that failed earlier. It is one witness's 'i think', and what z21 and upstream-dts showed on the panel in their tries is not recorded, so the contrast is missing: d0 (the display kernel with z21's dtb, no display node) should keep the logo up if this reading holds; d1/d2 say which stage turns it black. Not a proof: black could also be the bootloader turning the panel off at its own reboot.

About
: walleye-display-s6e3fa5


<a id="walleye-display-try1-no-shell-what-is-known"></a>
### walleye-display-try1-no-shell-what-is-known

Date
: 2026-11-19

Brief
: Display try 1: the gadget never enumerated in the 49 s window; the clock_gcc fix IS in the image; the dtb differs from z21 only by the display nodes, the kernel only by DRM

Question
: Why did boot-display.img give no shell (left fastboot 11:40:18, Android back 11:41:07, nothing between)?

Found
: Host kernel log (journalctl -k, host grig): 11:39:26 fastboot 18d1:4ee0; 11:40:18 disconnect (left fastboot); 11:41:07 Android 18d1:4ee7 (49 s later). No USB device between: the gadget (1d6b:0104, z21 and upstream-dts: 18 s after leaving fastboot) never enumerated. The kernel died or hung before the gadget bound. pstore: per the records /sys/fs/pstore was empty after every mainline try (ml3, pk5l2, pk5-stages) and the ramoops file after unlock needs root which stock Android does not give; not re-read for this try (the keeper owns the phone). The mainline shell is telnet 172.16.43.1 with no password: the keeper did not need root or ssh, the shell never came up; no access change is needed. DTB: the dtb INSIDE boot-display.img (extracted at kernel blob offset 15870703, sha256 ed155326...) has clock_gcc on the real gcc (clock-controller@100000); the standalone images-display/msm8998-google-walleye-display.dtb (sha256 84c72763...) is the raw input that build-display-image.sh fixes itself (clock_gcc on /vendor-overlay-stubs/clock_gcc in the raw file, moved to the real gcc by the script; builder, PR 404): not stale, and not what booted. So the clock_gcc rebuild is not the cause. Versus z21's dtb (phandle-blind diff, node paths): only /soc@0/display-subsystem@c900000, its display-controller@c901000, dsi@c994000, phy@c994400 are 'okay' (z21: disabled), plus panel@0 (samsung,s6e3fa5) under the dsi, three pinctrl states (gpio94 x2, gpio10 TE), regulators l14 (1.8 V) and l22 (3.0 V) with vdd_l18_l22-supply; everything else equal. Kernel config versus z21's kernel (out-z20): only DRM, DRM_MSM(_DPU,_DSI,_DSI_10NM_PHY) =y, panel, fbdev/fbcon, backlight, LLCC/OCMEM/AOSS_QMP/MDT_LOADER/SMEM =y, UBWC_CONFIG m->y; the diagnostic fragment (DEBUG_GPIO, DYNAMIC_DEBUG) is dropped. So the cause is in the display path: a built-in driver probe (DRM_MSM, DPU, DSI, panel, or one of the five =y helpers) or the display nodes, seen only by a bisect since no log survives. The author's report of the screen is not in yet.

About
: walleye-display-s6e3fa5


<a id="walleye-display-bisect-d1-loses-the-shell"></a>
### walleye-display-bisect-d1-loses-the-shell

Date
: 2026-11-19

Brief
: Display bisect: d0 (display kernel, z21 dtb) boots a shell, 335 dmesg lines, no drm; d1 (mdss + display controller) loses it: Android back after 57 s

Question
: Which of d0, d1, d2 first loses the mainline shell, from slot b, unattended?

Found
: d0: left fastboot 15 s, gadget 1d6b:0104 at 18 s, reader 16 answers, dmesg 335 lines, class/drm empty (no display nodes, as built); material ~/lab-material/walleye/d0-try1. d1: gate and battery (4376 mV) passed; left fastboot 15 s, no mainline gadget, Android adb back after 57 s (dead-kernel path), reader found no ttyACM0 in 60 s; Android's boot reason property says bootloader (its own last reboot, not informative), pstore not readable without root; material ~/lab-material/walleye/d1-try1. Stopped at d1 as the item says; d2 not tried. So the mdss and display-controller nodes (d1 over d0) are what lose the shell. Slot b, nothing flashed, phone on Android slot b, adb up. The author was not asked to look.

About
: walleye-display-s6e3fa5


<a id="walleye-ufs-version-from-source"></a>
### walleye-ufs-version-from-source

Date
: 2026-11-19

Brief
: UFS host version stays 3.0.0 or 3.1.0 from Android's side; no print, no readable trace; the matched image's own read at ufs_qcom_init settles it

Question
: Can UFS_HW_VERSION (3.0.0 vs 3.1.0) be narrowed from the vendor source, Android's dmesg and the public record, without a register read?

Found
: Verdict: still two (certain: one of 3.0.0, 3.1.0). Evidence checked, 2026-11-19, nothing run on the phone. (1) Vendor selection: ufs_qcom_phy_qmp_v3_phy_calibrate takes (3,0,0) -> rate_A_3_0_0 table, (3,1,0) -> rate_A_3_1_0, else -ENODEV; Android's UFS works, so it is one of them. (2) Nothing prints the version: ufs_qcom_init stores hw_ver and passes it to the PHY, no dev_info; the only differing branch, ufs_qcom_configure_lpm (SVS2 PHY config with a hibern8 round trip), runs only on exactly 3.0.0 and leaves no log line, so Android's dmesg cannot tell the two apart. (3) The live Android dt (research walleye-first-fastboot-boot 2026-11-18-ufs-android-way live-ufs-props) has the same node for either: phy compatible qcom,ufs-phy-qmp-v3, freq table without an SVS2-only entry that could be told apart; not discriminating. (4) Public record: mainline maps qcom,msm8998-qmp-ufs-phy to sdm845_ufsphy_cfg and it is used on MSM8998 hardware; the vendor 3.1.0 table is the one within 10 lines of it (3.0.0: 35), a weak lean to 3.1.0 for a table that works; the vendor comment 'PHY attached to v3.0.0 controller supports SVS2' and v3 first appearing on MSM8998 lean the other way. No public record states MSM8998's HW_VERSION. Both leans are inference, not evidence. What the matched image settles: its pr_info of REG 0xE4 in ufs_qcom_init plus the table chosen by it, before the PHY is initialised; whichever it reads, the image already carries the choice. Nothing further to do from Android's side.

Sources
: wahoo-4-4-ufs-qmp-v3,mainlining-dev-ufs-phy-driver

About
: walleye-ufs-match-android-z


<a id="walleye-ufs-gap-link-startup-read"></a>
### walleye-ufs-gap-link-startup-read

Date
: 2026-11-19

Brief
: Android's link-startup pre-change = timers, RX linecfg on, core clk 150, HW clock gating, LCC off, SAVECONFIGTIME mask; mainline matches all but RX linecfg and the mask: do not carry

Question
: What does the GPL wahoo 4.4 ufs-qcom do before link startup (three Qualcomm-specific steps), what does mainline do for other Qualcomm hosts, and should the matched image carry the difference?

Found
: Read 2026-11-19 from the GPL source (android-msm-wahoo-4.4-android11 ufs-qcom.c, ufs-qcom.h, phy-qcom-ufs-qmp-v3.h) against the local mainline tree (surya-audio/linux, ufs-qcom.c); nothing run on the phone.
Android ufs_qcom_link_startup_pre_change, in order: (0) cfg_timers(G1, SLOWAUTO, link-startup timer) - clock-cycle registers; (1) ufs_qcom_phy_ctrl_rx_linecfg(true): clear bit1 (UFS_PHY_RX_LINECFG_DISABLE_BIT) of UFS_PHY_LINECFG_DISABLE = PHY_OFF(0x130) in the v3 PHY, readl/modify/writel, so RX LineCfg is on; (2) if QUNIPRO: DME_VS_CORE_CLK_CTRL (0xD002): MAX_CORE_CLK_1US_CYCLES (mask 0xFF) = 150, clear CORE_CLK_DIV_EN (bit 8); (3) ufs_qcom_enable_hw_clk_gating: REG_UFS_CFG2 (0xE0) |= bits 0-7 (all CGC_EN), then if QUNIPRO_CLK_GATING (hw major >= 3): DME rmw DL_VS_CLK_CFG (0xA00B) |= 0x3FF, PA_VS_CLK_CFG_REG (0x9004) |= 0x1FF, DME_VS_CORE_CLK_CTRL bit 9 (DME_HW_CGC_EN); (4) PA_LOCAL_TX_LCC_ENABLE (0x155E) = 0 unless UniPro 1.41; (5) if QUNIPRO_CLK_GATING: DME rmw PA_VS_CONFIG_REG1 (0x9000) |= SAVECONFIGTIME_MODE_MASK 0x6000. The 'three steps' of the item are: RX line config (1), the PA_VS_CONFIG_REG1 mask (5), hardware clock gating (3).
Mainline link_startup_notify PRE_CHANGE: cfg_timers; set_core_clk_ctrl (the same 0xD002 cycles/div step, scaled to the core clock rate, not a fixed 150); ufshcd_disable_host_tx_lcc (same as (4)). Hardware clock gating moved to hce_enable_notify POST_CHANGE (ufs_qcom_enable_hw_clk_gating: same CFG2 bits, same three DME rmw, unconditional on version) - before link startup, as before. Mainline has NO RX linecfg write (the v4/v6 PCS headers define LINECFG_DISABLE offsets but no code uses them; grep over drivers/phy/qualcomm and ufs-qcom.c) and NO SAVECONFIGTIME_MODE_MASK write (its PA_VS_CONFIG_REG1 use is a different, quirk-gated bit 12). The MSM8998 PHY compatible maps to sdm845_ufsphy_cfg; that, sdm845, sm8150 and the later hosts bring UFS up on mainline without those two steps.
Findings: (a) clock gating, LCC off and core clock control: the matched image has them through mainline (power and timing, none gates the link). (b) RX linecfg: the only link-relevant difference; vendor comment ties line-cfg trouble to power-mode change on some devices (UniPro 1.41 post-change disable only); whether the bit resets to enabled is not in the sources - an unknown the try can answer by one read of PHY+0x130 and needs no write. (c) PA_VS_CONFIG_REG1 0x6000: a Qualcomm-internal SaveConfigTime mode, power-mode-change time, no effect before the link is up; mainline hosts run without it.
Recommendation: do not carry. The image's mainline path equals Android's for what the link needs; the two missing steps are absent on every mainline Qualcomm host that boots from UFS, and the decision 2026-11-19-walleye-ufs-match-no-ice-no-linkstartup already bars new unreviewed hardware code. If the try's link comes up and a read of PHY+0x130 shows bit1 set, nothing is left; if link startup fails with the line-cfg bit set (disabled), the carry then is one reviewed PHY write - recorded as the fallback, not built now. The programmer's read of this was not taken: the item proof asks for it only on a carry, and the verdict is no carry; if the author wants one, it is a reading of two diffs of about 20 lines.

Sources
: wahoo-4-4-ufs-qcom-c,wahoo-4-4-ufs-qmp-v3,mainlining-dev-ufs-phy-driver

About
: walleye-ufs-match-android-z


<a id="walleye-netconsole-over-the-usb-gadget"></a>
### walleye-netconsole-over-the-usb-gadget

Date
: 2026-11-19

Brief
: netconsole over u_ether: set up works from the shell, and carries messages while the controller runs; whether it carries a hang or panic is not known yet

Question
: Can netconsole send the kernel log over the ECM gadget (usb0) to the host, and what does u_ether lack?

Found
: Read in the v7.3 tree: u_ether.c has no ndo_poll_controller (netpoll only calls one when a driver has it, net/core/netpoll.c:191), so netconsole sends through ndo_start_xmit and does not drain the queue itself. That works while the USB controller's interrupts run (ordinary log lines); a line printed with interrupts off, or a panic, may reach the queue and never the wire. Not known yet: whether the last lines before a dwc3 or display hang arrive. The probe therefore proves each channel with a marker line before loading anything, reports it (carries yes|no), refuses to load when neither carries, and adds the second channel, a TCP stream of /dev/kmsg over the same link; a bus hang can still stop both. Netconsole is built in (NETCONSOLE=y, NETCONSOLE_DYNAMIC=y) and set from the shell by configfs: dev_name, local_ip, remote_ip, remote_mac (the ECM host_addr 02:42:ac:10:2b:02), ports 6665/6666, then enabled=1.

About
: walleye-display-s6e3fa5


<a id="walleye-ufs-ice-verdict"></a>
### walleye-ufs-ice-verdict

Date
: 2026-11-19

Brief
: ICE inits after the PHY and before link startup in Android but cannot touch link or card; mainline without ICE only warns: skip it for the read-only probe

Question
: What does Android's probe do with ICE before the first UFS command, and does the card path need it?

Found
: Android: HCE pre-change = PHY power-up, lane clocks, then ICE init (own block 0x1db0000, 4 clocks, TZ config call, version/fuse/BIST reads, leaves global bypass); per-command key setup only for requests with a key, and returns at once for host major 3. ICE writes no link, PHY, reset or card registers and issues no command. Mainline v7.3: no msm8998 ICE node/compatible; without it ufs_qcom_ice_init warns and returns 0, no ICE access at all. Verdict: skip for a read-only probe; later an upstreamable port only if inline encryption is wanted. Not known yet: UFS_HW_VERSION of this host.

Sources
: src-google-wahoo-msm44-android11,src-mainline-v73-ufs-qcom-ice

About
: walleye-ufs-match-android-z

Material
: research/walleye-first-fastboot-boot/2026-11-19-ufs-ice/


<a id="walleye-display-late-probe-try1-no-gadget"></a>
### walleye-display-late-probe-try1-no-gadget

Date
: 2026-11-19

Brief
: Late-probe image tried once: no gadget at all (no 1d6b:0104), Android back after 57 s; same as d1, so the loss is not the display modules

Question
: Does the display late-probe image (display dtb, DRM/msm/panel as modules) reach the shell so the modules can be loaded by hand with the log streamed?

Found
: Gates passed first: battery 4375 mV, ufs-off-image.sh pass on boot-display-late.img, sha256 ok. adb reboot bootloader, lpoco lab fastboot boot now: left fastboot after 15 s, came back as Android slot b after 57 s. The host kernel log shows only Android's 18d1:4ee0 then 18d1:4ee7 (Pixel 2): no mainline gadget ever enumerated, so the shell, netconsole and TCP stream never existed and display-late-probe.sh could not run; no last-line finding. Same dead-kernel path as d1 (57 s). The modules are not loaded at boot in this image, so the cause lies in what d1 and this image share and d0 lacks: the display nodes (mdss, display controller, panel) in the dtb, or the DRM built-in parts. Nothing flashed; phone on Android slot b, adb up, battery 4384 mV. Item walleye-try-display-late-probe stays open: its proof is not met. Next, for the lane coordinator: a dtb with the display nodes disabled on this kernel with the late-probe ramdisk, to prove dtb nodes and not kernel; or a ramoops read of this death.

About
: walleye-display-s6e3fa5


<a id="walleye-ufs-gap-abl-handover-read"></a>
### walleye-ufs-gap-abl-handover-read

Date
: 2026-11-19

Brief
: The abl leaves the host and card running; our kernel resets the host, and only the matched image also pulses the card: four unpulsed probes read fine

Question
: What state does the bootloader leave the UFS host and card in for a RAM-booted kernel, and what does the matched image's reset and enable path do about it, from the dmesg of z15, z16, z17, z20 and the GPL and mainline sources?

Found
: Read 2026-11-19 from the captured dmesgs of the four images that had UFS on by accident (lab-material walleye z15-part1, z16-part1, z17-part1, z20-try1), the lab's dts history, and mainline's ufshcd.c and ufs-qcom.c (surya-audio/linux, 2e6c6521d); nothing run on the phone.

WHAT THE BOOTLOADER LEFT, as far as the images show. (1) The card was up and in use: the kernel command line the abl hands over carries androidboot.bootdevice=1da4000.ufshc and androidboot.hardware.ufs=64GB,Hynix, and its boottime stages (1BLL, 1BLE, 2BLL, 2BLE) are the abl reading the boot image from this card: the abl had the link up and the card initialised a moment before our kernel started. (2) Our kernel did not trust or use that state. mainline ufshcd_hba_execute_hce stops the host first when it is active (ufshcd_hba_stop, HCE 1->0) and restarts it, so the host controller is reset whatever the abl left; link startup (DME link startup, UniPro) runs from a fresh host. The vendor 4.4 driver does the same (HCE reset in its hba enable), so Android's own path also starts from a reset host. (3) The card side is NOT reset by that. The device reset line (RST_n, pin 153 UFS_RESET) is pulsed only through ufshcd_device_reset (ufshcd_init after the irq request, and link recovery), and ufs_qcom_device_reset returns -EOPNOTSUPP without a reset-gpios property. z20's dyndbg shows exactly that: 'No GPIO consumer reset found' after the failed lookups of reset-gpios and reset-gpio; and the lab's dts history (git log -S reset-gpios -- dt) shows the property first added by the matched image's commit (baa5540, walleye builder/ufs match android, #400), so z15, z16, z17 had none either (their captures have no dyndbg; this is from the dts, not the log). So in the four captured probes the card was NOT pulsed: it stayed in the state the abl left it, and still answered.
(4) What it did with that: no 'link startup failed', no retry, no timeout or UIC error line in any of the four (ignore_loglevel was on, so errors would show); 'scsi host0: ufshcd' to the first well-known LUN took about 250 ms in z15 (1.108 to 1.357 s), z16 (1.129 to 1.370) and z17, and about 207 ms in z20 (1.221 to 1.429); then all six LUNs sda..sdf attached and the partition tables read (45 partitions on sda). The card is the SK hynix H28U74301AMR firmware D003, the same in all four. That is: a host reset plus link startup on a card the abl left running (after the abl's own HS gear, if it used one) worked four times out of four captures, every one with the same outcome.

WHAT THE MATCHED IMAGE DOES DIFFERENTLY. It adds the RST_n pulse (10-15 us low then high, as Android's driver does) before the host starts, so the card is returned to its power-on state and the abl's leftover state is moot: the same sequence Android runs from its own abl, and a step none of the four accidental probes ran. That makes the matched image's state at start closer to Android's, but it is also the one step the evidence in dmesg does not cover: no capture shows a reset-pulsed probe on this card, because the first image with the pin reachable is the matched one. The pulse is short and standard; mainline hosts that boot from UFS use it; but 'worked unpulsed four times' is not 'worked pulsed'.

WHAT IS NOT KNOWN, and the try answers it. (a) The host's own state at handover (HCE, UTRLRDY, link state, power mode of the link, gear): no image printed anything before the HCE reset, and ufshcd reads none of it first; it cannot be read from Android either (research walleye-ufs-version-from-source). The matched image's first print is the version at ufs_qcom_init; a read of REG_CONTROLLER_ENABLE (0x34) and the host status (0x30) in the same place would show whether the abl left the host enabled, at the cost of two more reads in the spot where the version read already stands (whether the clocks are on there I did not check, so the bus-hang risk of these reads is not measured; not added by this research, offered as optional). (b) Whether the card, once pulsed, comes up the same: the try's read of the card identity and LUNs is the answer; failure of link startup after a pulse would show as 'link startup failed' with retries, and falls back on the decision 2026-11-19-walleye-ufs-linkstartup-do-not-carry (one reviewed PHY write).

VERDICT in plain words: the bootloader leaves the host enabled and the card running in whatever mode it read the boot image in; the kernel resets the host, and (matched image only) the card; either way the sequence has no write to the card, and the four unpulsed probes (z15, z16, z17, z20) show the card tolerates a host reset over its running state. The pulse is Android's own step and stands in the matched image. Nothing here asks for a change to it; one optional read-only print is named above. The gap 'abl handover' is closed to the point the sources allow: the rest is the try's to measure.

Sources
: wahoo-4-4-ufs-qcom-c,src-mainline-v73-ufs-qcom-ice

About
: walleye-ufs-match-android-z


<a id="walleye-ufs-gap-lane1"></a>
### walleye-ufs-gap-lane1

Date
: 2026-11-19

Brief
: Vendor 3.0.0 table programs lane 0 only (3.1.0 adds lane 1); Android needs nothing else for lane 1; a down lane 1 leaves link up and reads working, only the 2-lane step fails

Question
: What does the 3.0.0 PHY table program for lane 1 (TX/RX), does Android's HS-G3 two-lane link at version 3.0.0 rely on something outside the table, and what would the matched image show if lane 1 stayed down?

Found
: Read 2026-11-19 from Google's GPL wahoo 4.4 (android-msm-wahoo-4.4-android11, head 52750b8f; fetched with the earlier research's scripts/fetch-vendor.sh; tables counted from its results/phy lists); nothing run on the phone.

TABLE COUNT (entries by block): rate_A_3_0_0 = COM 35, TX0 1, RX0 15, PCS 6, TX1 0, RX1 0. rate_A_3_1_0 = COM 36, TX0 1, RX0 15, RX1 15, TX1 1 (QSERDES_TX1_LANE_MODE_1 = 0x06), PCS 9 (adds UFS_PHY_MULTI_LANE_CTRL1 = 0x02 among them). So the 3.0.0 table writes nothing for lane 1, no TX1 and no RX1; only 3.1.0 does. Rate B table is one entry and lane-free. The table's lane-1 entries are the same values as lane 0's (RX1 SIGDET/UCDR/EQ entries mirror RX0).

DOES ANDROID RELY ON ANYTHING ELSE FOR LANE 1? Outside the table, in the vendor code: (1) ufs_qcom_phy_calibrate only writes tbl_A (plus tbl_B for rate B) - no per-lane code; (2) the v3 PHY's set_tx_lane_enable is an empty function ('v3 PHY does not have TX_LANE_ENABLE register'); (3) the host enables the rx_lane1_sync_clk when the host has two lanes (ufs-qcom.c clock enable; our dtb lists the same clock-names, ours-ufs-props); (4) lanes-per-direction: the base msm8998.dtsi (v1 silicon, msm-id 0x0) says 1, the msm8998-v2.dtsi overlay says 2, and the live fdt reads 2 (live-ufs-props) - Walleye runs the v2/v2.1 description. Nothing else touches lane 1. So under 3.0.0 Android would have run its two-lane HS-G3 link on a PHY whose lane 1 was never programmed: either the hardware defaults suffice, or the host is not 3.0.0. Live: the first link is 'gear 1, 1 lane, SLOWAUTO' (1.82 s) and the power-mode change to 'gear 3, 2 lanes, FAST, rate B' follows at 2.05 s (research walleye-ufs-android-way README) - two lanes did work on this phone under Android's tables, whichever of the two it ran.

WHICH IT IS (still not known): the coherent picture is 3.0.0 table <-> v1 silicon, one lane (the base dtsi's lanes 1 beside a lane-0-only table) and 3.1.0 table <-> v2.x silicon, two lanes (the v2 overlay's lanes 2 beside a table with lane 1). Walleye is v2.1, which leans 3.1.0 - inference only: no public record states MSM8998's UFS_HW_VERSION per silicon revision (a search for one found none), and the vendor comment 'PHY attached to v3.0.0 controller supports SVS2' leans the other way (research walleye-ufs-version-from-source). The matched image's own read settles it.

IF THE VERSION READS 3.0.0 AND LANE 1 STAYS DOWN (expected, not run): the link comes up and reads work. Link startup and the first link run on one lane in PWM-G1 (the live 'gear 1, 1 lane' step), so a down lane 1 cannot stop link startup or the card read; it can only fail the later power-mode change to two lanes. What it would show: the 'power mode change' / 'failed setting power mode' error from ufshcd and a link left at one lane, G1 (debugfs power_mode showing 1 lane), not a hang, no card write. Ours is not Android here: mainline's sdm845 table (what MSM8998 uses) already writes both lanes' entries, so on 3.1.0 nothing differs for lane 1; only a 3.0.0 read uses the lane-0-only table.

ANSWER FOR THE TRY: lane 1 is a risk to speed, not to the card or the bootloader. Fallback if the read says 3.0.0 and the power-mode step fails: carry the 3.1.0 lane-1 entries (TX1_LANE_MODE_1 0x06, the 15 RX1 entries, MULTI_LANE_CTRL1 0x02) as one reviewed table change in a later image - a PHY lane-register write, not a card command; decide only after the try shows the version and the lane count (item backlog walleye-ufs-gap-lane1). Needs no phone work now.

Sources
: src-google-wahoo-msm44-android11,wahoo-4-4-ufs-qmp-v3,mainlining-dev-ufs-phy-driver

About
: walleye-ufs-match-android-z


<a id="walleye-display-dtb-bisect-built"></a>
### walleye-display-dtb-bisect-built

Date
: 2026-11-19

Brief
: f0..f3 built: the late-probe kernel with z21's dtb (control) and with the regulators, the pinctrl states, or mdss/dsi/phy okay alone; checks ok, rebuild identical; not tried

Question
: Which group of the display dtb's additions can be tried alone on the late-probe kernel, and are the images exactly that?

Found
: Four images on the late-probe image's kernel, ramdisk (init-late, module list) and header: f0 z21's dtb byte for byte (control), f1 + vreg_l14a_1p8, vreg_l22a_3p0 and vdd_l18_l22-supply (bob), f2 + the three mdss pinctrl states, f3 + mdss, DSI host and PHY okay (no panel, regulators, pinctrl; mdss_mdp has no status in z21, so its okay is a no-op). Each dtb diff against z21's is a file showing only that group; the added lines are compared exactly. Each image passes tests/host/ufs-off-image.sh, AVB footer, header/ramdisk/size equal to the late-probe image's; a run with z21 as the late image fails 6 checks; rebuild byte-identical; programmer review done. f3 does not carry the display dts's DSI vdda-supply, PHY vdds-supply or output endpoint; f1's regulators have no consumer. Sums: f0 57bf4a96, f1 78aeb579, f2 d9c539cc, f3 1ee2edb0. Not run on any phone.

About
: walleye-display-s6e3fa5


<a id="walleye-ufs-gap-sony-mechanism-public-record"></a>
### walleye-ufs-gap-sony-mechanism-public-record

Date
: 2026-11-19

Brief
: Sony's yoshino/tama UFS bootloader loss: mechanism not published; the SODP view is purge via the PURGE_ENABLE flag; one counter-case probed clean; our probe sends no purge

Question
: Why did Sony's yoshino phone lose its bootloader with a UFS probe lacking quirks, which command, which card, and can the matched image's path send it?

Found
: Verdict: the mechanism is NOT PUBLISHED. No mailing-list post, commit message or forum record names the command that wiped the bootloader. Sources searched this run: Sony bug tracker 799 and 802, the XDA thread asking exactly this question (no answer on record), the yoshino and tama dtsi warnings, the SODP forward-port commit, the postmarketOS wiki warnings; earlier run: the Kumano series author could not ask Sony.

What is said, by whom:
- A SODP contributor (bartcubbins, bug tracker 802): the issue is on all OEMs with similar memory chips; "only one write cycle and your memory will be purged (sort of)"; first places to look are UFS_DEVICE_QUIRK_NO_PURGE and EXTEND_SYNC_LENGTH; the main thing is to keep the driver from handling QUERY_FLAG_IDN_PURGE_ENABLE by any means. The sync-length quirk is "likely" about board or data-line faults, and its absence "might" cause false triggering of the purge flag or corruption. These are his beliefs, stated without a trace or log.
- The quirk list itself (forward port, 2019): NO_PURGE for Hynix hB8aL1/hC8aL1 fw D001, Samsung KLUCG4J1 fw 0101, any card older than UFS 2.1; sync length 0x48 for all Hynix; one TX lane.
- Recovery: a dump cannot be written back (bartcubbins); the SoC is QPROM-locked, EDL needs the xbl that lived in the wiped UFS, a swapped chip has no initialised RPMB (phodina). So: no way back, as the lane already assumed.

Counter-evidence: a yoshino XZ1 (Samsung KLUCG4J1ED-B0C1, UFS 2.1, fw 0200) enabled UFS on a v6.0-based mainline tree, booted several times, mounted rw, nothing erased. The fw 0200 FFU ships in Sony's image, so 0101 may be the affected revision; unproven. That card is Samsung, not our Hynix.

Can the matched image's path send it? The purge flag is set only by the ufs_bsg RPMB purge path, which the matched image never opens; the probe itself sends host reset, link startup, NOP, fDeviceInit, descriptor reads, power-mode change and the SCSI scan (INQUIRY, TUR, READ CAPACITY), no write, unmap or purge (research walleye-ufs-hazard-verdict). That holds if our tree equals mainline's. It does not explain the Sony loss: the loss happened with the same sort of probe, so either something outside the listed commands acts (a card-firmware reaction to the power-mode or sync-length settings, as the SODP contributor guesses, or a flag set by the abl/xbl image), or a Sony-specific path. Not known which.

Open and honest: what we do not know is whether the Pixel 2's Hynix firmware is a purge-on-fault revision; its model and firmware are not readable without root (hazard research). The matched image still lacks Sony's EXTEND_SYNC_LENGTH (all Hynix) and AVAILTXDATALANES; the SODP contributor ties the first to the mechanism.

Plain answer for the go note: unknown 4 stays unknown, counted as a finding. The matched image cannot send a purge on any path we can read, but the one thing that could differ, the sync-length setting for Hynix, is missing.

Sources
: src-sony-ufs-quirks-forward-port,src-yoshino-dtsi-ufs-disabled,src-sony-bugtracker-799,src-sony-bugtracker-802,src-xda-yoshino-ufs-erasure-origin

About
: walleye-ufs-match-android-z


<a id="walleye-display-dtb-bisect-tried"></a>
### walleye-display-dtb-bisect-tried

Date
: 2026-11-19

Brief
: f0, f1, f2 boot a shell (gadget at 18 s, 619-625 dmesg lines); f3 (mdss, display controller, DSI host and PHY okay) loses the gadget: Android back at 54 s

Question
: Which group of the display dtb's additions stops the gadget?

Found
: Each of f0..f3 once from slot b, f0 first, ufs-off gate and battery rule passed each time (4348-4381 mV), nothing flashed. f0 (z21 dtb, control), f1 (regulators only) and f2 (pinctrl states only): left fastboot 15 s, gadget 1d6b:0104 at 18 s, mainline-shell-read 16 answers, dmesg 619, 625, 619 lines, then reboot -f, Android back on slot b. f3 (mdss, display controller, dsi host and phy set okay, no panel node, no regulators, no pinctrl): no gadget, Android back after 54 s, the dead-kernel path. So the loss is the display controller group alone, not the regulators or pinctrl states, and with the display modules not loaded (late-probe kernel): the node enable itself, probed by built-in or early drivers (clocks, power domains, iommu), kills the kernel. Next: split f3 by node (mdss, dpu, dsi, phy), a builder piece. Material ~/lab-material/walleye/display-dtb-bisect-f0..f2 (f3 has none).

About
: walleye-display-s6e3fa5


<a id="walleye-ufs-gap-perf"></a>
### walleye-ufs-gap-perf

Date
: 2026-11-19

Brief
: Bus votes, PM-QoS, hardware clock gating and the ufs_reset pad drive are performance or power only; a read-only probe needs none of them, the pad drive rides with the RST_n pulse

Question
: What do Android's bus votes, PM-QoS groups, hardware-gated clocks and ufs_reset pad drive do, what breaks without them (performance, power or the link), and does a read-only probe need any?

Found
: Read 2026-11-19 from Google's GPL wahoo 4.4 (android-msm-wahoo-4.4-android11, head 52750b8f; fetched with the earlier research's scripts/fetch-vendor.sh into a scratch dir) against Linux v7.3 (drivers/ufs/host/ufs-qcom.c, drivers/pinctrl/qcom/pinctrl-msm8998.c, msm8998.dtsi) and our overlay dt/msm8998-google-walleye-ufs-match.dtso. Nothing run on the phone.

ANSWER FOR A READ-ONLY PROBE: none of the four items is needed. Each is performance or power; none can reach the link startup, the card's identity or a write.

1. BUS VOTES. Vendor: msm_bus client "ufs1" (ufs-qcom.c ufs_qcom_update_bus_bw_vote, set after every power-mode change; dts qcom,msm-bus,vectors-KBps, 22 cases on two paths: 95->512 to DDR, 1->650 config). MIN = no vote; PWM G1 L1 = 922 KB/s; HS G3 RA/RB L1 = 2097152 KB/s DDR + 102400 CNOC; HS G3 L2 = 4194304 + 204800; MAX 7643136 + 307200. The dts comment says why: at HS-G3 UFS runs at nominal voltage corner, so the vote pushes BIMC and CNOC to nominal to reach full throughput. Without a vote: the buses stay at whatever abl/RPM left, so HS-G3 throughput may be capped (a performance loss, not a failure; effect size unmeasured, inference only). Mainline v7.3: ufs_qcom_icc_init gets paths "ufs-ddr" and "cpu-ufs" through devm_of_icc_get; with no interconnects property in the node (msm8998.dtsi has none) of_icc_get returns NULL and icc_set_bw(NULL) returns 0 - no vote, no error; v7.3 has no msm8998 interconnect driver at all. A read-only probe runs the link startup and first link at PWM-G1 one lane (vendor vote 922 KB/s, effectively MIN): needs nothing. Later work, not the probe: HS speed wants an interconnect provider for msm8998 (not in mainline) - a backlog item when HS throughput matters.

2. PM-QOS. Vendor: qcom,pm-qos-cpu-groups <0x0F 0xF0>, latency 70 us, default cpu 0; per request a vote_work does pm_qos_update_request(70 us) on the cpu group of the request and an unvote_work drops it after 10 ms (UFS_QCOM_PM_QOS_UNVOTE_TIMEOUT_US). It keeps the issuing cluster out of deep idle while I/O is in flight, to cut completion-interrupt latency. Performance (I/O latency under cpuidle) only; sysfs pm_qos_enable switches it off. Mainline v7.3 has no such code in ufs-qcom.c and no walleye cpuidle states yet; a probe with a few reads is unaffected. Not carried; no backlog item until cpuidle exists on this lane.

3. HARDWARE-GATED CLOCKS. Vendor ufs_qcom_enable_hw_clk_gating (ufs-qcom.c, after link startup): REG_UFS_CFG2 |= CGC_EN_ALL, then for host version >= 3 the DL_VS/PA_VS clock-config masks and DME_VS_CORE_CLK_CTRL DME_HW_CGC_EN; saves leakage, no function. Mainline v7.3 has ufs_qcom_enable_hw_clk_gating called at hce_enable POST_CHANGE (CGC_EN_ALL, DME_HW_CGC_EN): same register effect, a little earlier. The one real difference (the DL/PA mask writes, with RX line config and the PA_VS_CONFIG_REG1 mask) is decided: research walleye-ufs-gap-link-startup-read and decision 2026-11-19-walleye-ufs-linkstartup-do-not-carry. Power only. Not needed.

4. UFS_RESET PAD DRIVE. Vendor pinctrl states ufs_dev_reset_assert / ufs_dev_reset_deassert (pin ufs_reset): output-low then output-high, 10-15 us each, bias-pull-down, drive-strength 8 (the dts comment: HDRV 3 = 3.1 mA, the power-on value, so the vendor sets the default, not a stronger drive). Mainline pad (UFS_RESET macro: pull_bit 3, drv_bit 0, out_bit 0, no oe, no mux; pin 153 on msm8998, outside ngpios = 150) is reachable only after a kernel change: our overlay sets gpio-ranges to 154 and reset-gpios <&tlmm 153 GPIO_ACTIVE_LOW> - mainline's ufs_qcom_device_reset then pulses the same pin (assert, delay, deassert). It drives the out bit and leaves pull and drive at the power-on value, which is what the vendor ends up with. So the pad drive belongs with the RST_n pulse the image already carries, and needs nothing beyond it. Open, unchanged: the order around the pulse differs (Android: core reset, pulse, host enable; ours: pulse, core reset in the host-enable pre-change, PHY, enable) and its effect on the card is not known (overlay header; walleye-ufs-gap-abl-handover-read).

VERDICT: bus votes - not needed (performance only at HS-G3; no provider exists); PM-QoS - not needed (latency under idle only); hardware-gated clocks - not needed (power only; mainline already enables the same gating, the mask difference is decided elsewhere); ufs_reset pad drive - already carried by the reset-gpios pulse, nothing more to add. The gap list for the solution table (walleye-ufs-solution-extracted) therefore loses these four rows as harmless-judged-and-now-read; the claim stays "read from source", not "proven on the phone": the probe's own read at PWM-G1 is what proves it.

Sources
: src-google-wahoo-msm44-android11,wahoo-4-4-ufs-qcom-c,src-mainline-v73-ufs-qcom-ice

About
: walleye-ufs-match-android-z


<a id="walleye-ufs-hynix-sync-length-verdict"></a>
### walleye-ufs-hynix-sync-length-verdict

Date
: 2026-11-19

Brief
: Android never writes PA_TxHsG1/2/3SyncLength or reads AvailTxDataLanes on the Pixel 2 Hynix card: Sony's 0x48 is not Android's value; verdict do not carry

Question
: What PA_TxHsG1SyncLength does Android use on the Pixel 2 Hynix card (H28U74301AMR fw D003) against Sony's 0x48, and should the matched image carry EXTEND_SYNC_LENGTH and AVAILTXDATALANES?

Found
: Verdict: do not carry either; the matched image stays as built (SAVECONFIGTIME and the NO_PURGE guard only).

Read, from Google's stock wahoo 4.4 tree (android-msm-wahoo-4.4-android11, head 52750b8f) fetched again this run: the vendor UFS driver never writes PA_TxHsG1/G2/G3SyncLength and never reads or writes PA_AvailTxDataLanes. unipro.h defines TX_HS_SYNC_LENGTH and PA_AVAILTXDATALANES, and nothing in ufshcd.c, ufs-qcom.c or the PHY files uses them; the vendor quirk table has no sync-length entry for any vendor (its SK hynix entries are the HS-G1-to-G3 switch for hB8aL1/hC8aL1/hC8aM1/hC8GL1/hC8HL1/hD8aL1/h08aM1, which do not match H28U74301AMR, and HOST_PA_SAVECONFIGTIME for any model). So Android's value on this card is the power-on default of the link, left as the card and host negotiated it, and Android runs the card at HS-G3, two lanes each way, rate B (live dmesg 2026-11-19: gear 3, lane 2/2, FAST, rate 2) with that default. Sony's 0x48 is a value written on top, by Sony's own quirk, for other cards (Hynix hB8aL1/hC8aL1 fw D001); the SODP contributor only believes it relates to the erase (research walleye-ufs-gap-sony-mechanism-public-record). AVAILTXDATALANES = 1 is the same story: Android uses both lanes and never sets it.

Correction to the item's note: "our tree lacks it for Hynix" is true of the matched image (its quirk table and tests/host/ufs-match-android.sh line 33 count zero EXTEND_SYNC_LENGTH and ONE_TX_LANE code in the tree), but the earlier z13 image (topic/ufs-hynix-quirks) carries both: that is the image the Android comparison already moved away from (research walleye-ufs-android-way-compared, difference 2).

Why not carry: the matched image's rule is equal to Android where Android is proven on this card; an added write is a difference from the only proven link, and it is a write whose purpose nobody has published. The proof that it is not carried is already a check (the test above).

Not known yet: the numeric value of the card's and host's sync-length attributes on live Android (no debugfs file shows DME attributes; a read needs a debug build or the vendor-kernel instrument, with its own risk). It would only matter if the try's link fails at HS-G3; then the first thing to read is the link state, not a quirk. The fallback stays decision 2026-11-19-walleye-ufs-linkstartup-do-not-carry (one reviewed PHY write if link fails).

Sources
: src-google-wahoo-msm44-android11,wahoo-4-4-ufs-qcom-c,src-sony-ufs-quirks-forward-port,src-sony-bugtracker-802

About
: walleye-ufs-match-android-z


<a id="walleye-display-f3-split-images-built"></a>
### walleye-display-f3-split-images-built

Date
: 2026-11-19

Brief
: g1, g2, g3 built from f3 (passthrough on the command line; no mdss iommus; mdss alone), all pass the UFS-off gate and AVB; rebuild identical; not tried

Question
: Can f3 be split three ways to test the iommu reading?

Found
: build/series/display-f3-split.toml, build/build-display-f3-split-dtbs.sh, build/check-display-f3-split.sh; images in walleye-build/images-display-f3-split. sha256: boot-g1 b64f487f..., boot-g2 171470d1..., boot-g3 f1d4ee01... (SHA256SUMS-display-f3-split has all six with the diffs against f3). z21's display controller node has no status (absent = enabled), so g3 sets it disabled explicitly. Programmer review: two check defects found and fixed. Not tried on the phone.

About
: walleye-display-s6e3fa5


<a id="walleye-audio-haptics-ways"></a>
### walleye-audio-haptics-ways

Date
: 2026-11-19

Brief
: Walleye audio: no sibling has it working; speaker needs Q6/ADSP + MI2S + new TAS2557 driver, not the WCD934x; haptics needs a new small DRV2624 driver

Question
: What is the way to drivers for the Pixel 2's haptics (DRV2624), speaker amps (TAS2557) and codec (WCD934x)?

Found
: What is (read, nothing changed): mainline 7.1 has no DRV2624 (drv260x/2665/2667 are other register maps) and no TAS2557 (tas2552/2562/2764/2770/2780/2781/2783 and tfa989x are other chips). It has wcd934x.c, but msm8998.dtsi carries no slimbus, lpass, q6 or sound nodes, and no mainline msm8998 phone has audio. The sibling Pixel 2 XL (nura wiki, calebccff linux-wahoo) boots with USB net; audio untested. Sunfish (sm7150) only has `ti,drv2624 haptic@5a` as a comment. So nobody has done either chip.
Hardware (vendor-kernel-hardware-inventory): DRV2624 at 0x5a on i2c_6, LRA, reset gpio95, PMI8998's own haptics disabled. TAS2557 stereo at 0x4c/0x4d on i2c_7, audio over quaternary MI2S (not SLIMbus), reset PM8998 gpio6, IRQs gpio7/8. WCD934x on SLIMbus carries the digital mics; the jack is disabled.
Finding that shapes the order: speaker playback does not need the WCD934x (inferred from the amps' MI2S wiring; the vendor routing read confirms it). It needs the ADSP (remoteproc, firmware from the vendor partition), APR/q6afe/q6asm, the MI2S dai, a machine driver, and a TAS2557 driver. The codec (SLIMbus NGD, mics) is the long part and comes after.
Ways: (1) new clean-room DRV2624 input force-feedback driver, structured like drv2667; (2) TAS2557 driver in bypass mode first (class-D on, DSP SmartAmp off), DSP firmware later; (3) port the vendor tas2557 driver whole with its firmware loader - rejected: large, tied to Android firmware, not upstreamable; (4) the sdm845 route (q6 sound card) as the pattern for ADSP + MI2S + machine driver; (5) wcd934x on SLIMbus: needs an msm8998 NGD node, no sibling has done it.
Unknowns, each settled by reading only: whether TAS2557 makes sound without DSP firmware and its minimum register sequence; the MI2S clock the amp needs; which ADSP firmware loads on mainline remoteproc for msm8998.
Risk to the reference phone: none for reading and drivers; each try is a second boot entry by the keeper.

Sources
: ti-drv2624-datasheet,ti-tas2557-datasheet,aosp-msm-marlin-codecs,nura-taimen-wiki,mainline-7-1-tree-read

About
: walleye-audio-haptics

Material
: research/walleye-audio-haptics/2026-11-19/


<a id="walleye-display-f3-split-tried"></a>
### walleye-display-f3-split-tried

Date
: 2026-11-19

Brief
: g1 (iommu.passthrough=1) and g2 (no mdss iommus) boot a shell; g3 (mdss alone) loses the gadget: the mdss node's iommus through the SMMU kills the boot

Question
: Which of g1, g2, g3 keeps the gadget with the display nodes on, and what does that say about the iommu reading?

Found
: Each once from slot b, g1 first; UFS-off gate and battery (4375-4385 mV) passed, nothing flashed. g1 (f3 + iommu.passthrough=1): left fastboot 15 s, gadget 1d6b:0104 at 18 s, shell read 16 answers, dmesg 624 lines. g2 (f3 without mdss iommus): same, gadget at 18 s, 624 lines. g3 (mdss alone, display controller disabled, iommus kept): no gadget, Android back after 63 s, the dead-kernel path. No mdss/dpu/drm lines and no /sys/class/drm in g1/g2: the display drivers are modules, not loaded. Reading (not yet proven): the death is the mdss iommus property probed early through the SMMU; passthrough or dropping the property avoids it. g3 dying alone says the display controller and DSI are not needed for it. Next: late-probe modules on the g1 or g2 dtb (walleye-try-display-late-probe), then choose passthrough or a fixed iommus entry. Material in ~/lab-material/walleye/display-f3-split-g1 and -g2 (g3 has none).

About
: walleye-display-s6e3fa5


<a id="walleye-audio-haptics-porthole-has-it"></a>
### walleye-audio-haptics-porthole-has-it

Date
: 2026-11-19

Brief
: Porthole's taimen kernel already has DRV2624, TAS2557, ADSP/q6/MI2S, WCD934x mics and voice; TAS2557 cannot be brought up without its firmware, so the bypass-mode plan fails

Question
: Is anyone's audio and haptics for the Pixel 2 (MSM8998, DRV2624, TAS2557, WCD934x) working on mainline, and what does the speaker need?

Found
: Yes: the porthole-dev pmaports tree (the 7.2.2 kernel the lab already boots) carries, for the sibling Pixel 2 XL, a DRV2624 input driver (patch 0018), a TAS2557 ASoC driver (0024, 0162, 0171-0174), ADSP remoteproc, q6 and the sound card (0084, 0085, 0159), SLIMbus NGD and WCD934x microphones (0076-0080, 0083, 0087, 0098-0104), and voice calls (0163-0175); its taimen dts has the drv2624 and tas2557 nodes, and its device package says calls and microphones were measured. So the earlier research row walleye-audio-haptics-ways, that no sibling has either chip, was wrong. Its first unknown is answered by the driver: the TAS2557 cannot be brought up by registers, its PLL setup and DSP program exist only in the firmware blob tas2557s_PG21_uCDSP.bin (loaded as tas2557s_uCDSP.bin), and without it the part stays mute and the sound card defers forever, so a firmware-free bypass driver is not a known way. That blob is a vendor file, shipped in porthole's firmware package from TheMuppets' mirror. Not yet read: whether walleye's amps are the same two as taimen's (same board family wahoo, porthole's own walleye dts has no audio nodes), and the licence of the blob. The porthole drivers are GPL-2.0-only Signed-off-by Giuseppe Maggio marked Assisted-by: LLM, ported from the downstream wahoo driver, so they are not clean-room of the vendor code; the lab's clean-room line for audio must be settled by the author.

Sources
: porthole-pmaports-taimen,ti-tas2557-datasheet

About
: walleye-audio-haptics

Supersedes
: walleye-audio-haptics-ways


<a id="walleye-display-late-probe-l-images-built"></a>
### walleye-display-late-probe-l-images-built

Date
: 2026-11-19

Brief
: L1 and L2 built: the late-probe image with iommu.passthrough=1 (L1) and without the mdss iommus property (L2); checks pass, rebuild identical

Question
: Can the late-probe image carry the g1 and g2 iommu fixes, each with exactly one change?

Found
: boot-l1.img (cmdline + ' iommu.passthrough=1', sha256 7a45100ca471e5406693b366999021afbcf2ed8dc8d4aa55931e8bc76f010171) and boot-l2.img (iommus <142 0> removed from mdss in the late image's full display dtb, sha256 700809f53df331f58566d9f3646514a202b8e4cbf604ddbac1143da81e4b10ba) in ~/Projects/walleye-build/images-display-late-l. Each has the late image's kernel, ramdisk and header words; AVB verifies; ufs-off-image.sh passes; the panel node stays. Scripts: build/build-display-late-l-dtb.sh, build/series/display-late-l.toml, build/check-display-late-l.sh. Note: the late dtb's mdss iommus is 142 0 (f3's is 149 0, different phandle numbering, same property); the late dtb is the full display dtb (panel node, regulators) and lacks z21's touch regulator, as the late image always did.

About
: walleye-display-s6e3fa5


<a id="walleye-z21-init-has-no-hold"></a>
### walleye-z21-init-has-no-hold

Date
: 2026-11-19

Brief
: z21's init does NOT hold on /tmp/hold: it restarts the phone at 180 s of uptime whatever is touched; the touch window is what is left after the device appears

Question
: Does touching /tmp/hold keep z21 up past its 180 s self-reboot, as the retry recipe of walleye-try-z21-touch-events says?

Found
: Unpacked boot-z21.img (unpack_bootimg, ramdisk): its /init is z14's; the petter subshell runs 'while [ $n -lt 90 ]' (90 beats of 2 s) and then reboot -f; there is no test of /tmp/hold in it. The hold lives only in init-late, init-pk5l, init-wdt1 and vendor3 (git grep tmp/hold build). So the recipe's hold step has no effect on z21; with Synaptics seen at 107 s (research walleye-z21-touch-events-window-missed) the reader has about 65 s, not 90. The touch program (tests/attended/touch-try.rs) therefore sets the window to what the restart leaves (180 s less uptime less 8 s) and refuses to ask the author when that is under 40 s.

About
: walleye-first-fastboot-boot


<a id="walleye-display-late-probe-l1-modules-load-no-drm-card"></a>
### walleye-display-late-probe-l1-modules-load-no-drm-card

Date
: 2026-11-19

Brief
: Late-probe L1 (passthrough): shell up at 2 s, all 13 display modules load with the shell alive; msm probes to the dpu mmap then stops; no drm card, no panel line

Question
: With iommu.passthrough=1 (L1), does the late-probe image reach the shell, and where does loading the display modules stop?

Found
: Gates: battery 4376 mV, ufs-off-image pass, sha256 ok. fastboot boot of boot-l1.img (nothing flashed, slot b): gadget 1d6b:0104 after 18 s, shell up 2 s later. display-late-probe.sh: log stream (TCP 6667) carried the marker, netconsole (UDP) did not. Steps 1-6 ran under the script; my own timeout 200 killed the script at step 7 (the phone was fine, held by /tmp/hold), so steps 7-13 were run by hand with the same commands and a fresh stream: all rc=0, shell alive after each, msm.ko and panel-samsung-s6e3fa5.ko with dyndbg=+p. The dmesg after msm shows: msm init, mdp and dsi registered, mdss mapped, DSI PLL0 vco 1577625878 Hz, last line 'dpu_kms_mmap_dpu mapped dpu address space' then mdss_runtime_suspend. No drm card (/sys/class/drm empty, no /dev/dri), no panel probe line, no error. The platform drivers msm_dpu, msm_dsi, msm_mdp, msm-mdss are registered. So with passthrough the dtb display nodes do not kill the boot: the failure is the SMMU path (matches g1/g2 vs g3), and the DRM master does not assemble: the component is waiting on something (panel/bridge/DSI host binding) with nothing logged. Android came back by itself on slot b, adb up, boot_completed 1, battery 4385 mV. L2 not tried (L1 did not fail). Stripes not looked for: no card to draw on. Next: read why the component master does not bind.

About
: walleye-display-s6e3fa5

Material
: research/walleye-display-s6e3fa5/2026-11-19/


<a id="walleye-display-why-dsi-defers-silently"></a>
### walleye-display-why-dsi-defers-silently

Date
: 2026-11-19

Brief
: L1: the DRM master waits for c994000.dsi, deferred with no reason; its node has vdda-supply but no vdd-supply, which msm8998's dsi config needs

Question
: On L1 after the display modules load, what does the msm component master wait on?

Found
: L1 (boot-l1.img, slot b, nothing flashed, gates pass, display-late-probe.sh pass: 13 modules, shell alive) with debugfs mounted by hand. device_component: aggregate c901000.display-controller 'not bound', its one component 'not registered'. devices_deferred: only 'c994000.dsi' with an EMPTY reason (dev_err_probe would have given one). The dsi device has no driver; its suppliers (pinctrl, pmic gpio, clock-controller, display-controller, phy c994400, glink-edge) are all 'available', waiting_for_supplier 0, no 'phy driver is not ready' line; the phy c994400 is bound (msm_dsi_phy). So the defer is a bare return: in msm_dsi_host_init (drivers/gpu/drm/msm/dsi/dsi_host.c) devm_regulator_bulk_get_const returns its error without dev_err_probe, and msm8998_dsi_cfg asks for 'vdd' and 'vdda'. The dsi node on the phone has vdda-supply, vci-supply, wqhd-vddio-supply and NO vdd-supply; a missing supply the core cannot give a dummy for defers silently. The mipi-dsi bus has no device (the panel registers only after the host), which is why no panel line appears. Not proven: the regulator error code was not read; the proof is an image whose dsi node has vdd-supply (the 0.9 V rail; vreg_l1a_0p875 on other msm8998 boards, to be read for the Pixel 2) and a dsi device that binds. Phone back on Android slot b (idle-android pass). Next: builder - L3 with vdd-supply on the dsi node, then walleye-try-display-late-probe again.

About
: walleye-display-s6e3fa5

Material
: research/walleye-display-s6e3fa5/2026-11-19/


<a id="walleye-display-late-probe-l3-built"></a>
### walleye-display-late-probe-l3-built

Date
: 2026-11-19

Brief
: L3 built: L1 plus vdd-supply = vreg_l1a_0p875 on dsi@c994000, nothing else; gate and AVB pass, rebuild identical, reviewed; not tried

Question
: Build the late-probe image L1 plus the vdd-supply on the DSI host node, the rail read from the phone's dtbo

Found
: boot-l3.img (sha256 9360c0579f1350128b8f43573d9900f8c74e08e0dac21e15089a628f20924e08, ~/Projects/walleye-build/images-display-late-l3/) = L1 with only the dtb changed: dsi@c994000 gains vdd-supply = <&vreg_l1a_0p875> (l1a, 0.88 V; diff in l3.diff, one line). Ramdisk, command line (iommu.passthrough=1), header, size and Image.gz are L1's; tests/host/ufs-off-image.sh and the AVB check pass; rebuild identical; the build and check scripts refuse an input that is not L1 (sha256 against SHA256SUMS-display-late-l). Programmer review done: input-identity check added, comment reworded.
THE RAIL IS NOT READ FROM THE PHONE: the dtbo entry 3 (~/Projects/walleye-build/control/dtbo.img) names no DSI core rail; its dsi fragments carry only the panel's vci (l22), wqhd-vddio (l14), pinctrl and gpios. Chosen by analogy: mainline msm8998_dsi_regulators wants vdd 0.9 V and vdda 1.2 V; our dts has vdda = l2a (1.2 V) and the DSI PHY's vdds = l1a (0.88 V). No mainline board in the tree supplies msm8998 DSI vdd. The try proves or refutes it; if the host still defers, other 0.9 V candidates are next.
For the try: tests/host/display-late-probe.sh needs a timeout of 400 s or more (item walleye-try-display-late-probe-l3). Build: build/build-display-late-l3-dtb.sh, build/series/display-late-l3.toml (lpoco-bringup image bisect), build/check-display-late-l3.sh.

About
: walleye-display-s6e3fa5


<a id="walleye-touch-program-unattended-run"></a>
### walleye-touch-program-unattended-run

Date
: 2026-11-19

Brief
: touch-try.rs unattended run: checks 1-9 pass, EVENTS-READY (120 s window), stops at the ask with 'nobody answered'; phone back on Android; program has no step into fastboot

Question
: Do the touch program's unattended parts pass for real, stdin from /dev/null?

Found
: First run stopped at step 4: the phone was on Android, not in fastboot (the program has no adb-reboot-bootloader step; pmic-try has back_to_fastboot). After adb reboot bootloader by the keeper, the rerun passed checks 1-3 (battery, built image, UFS off), boot from memory, shell + dmesg saved, touch controller listed, window 120 s (restart at 180 s), reader started, EVENTS-READY; stopped at ask 10 with 'nobody answered'. Phone restarted by itself; tests/host/idle-android.sh passes. Nothing flashed, slot b. Also: lpoco lab suite latest fails here - no password file ~/.config/devlab/google-walleye/password.

About
: walleye-attend-program-touch


<a id="walleye-display-late-probe-l3-tried-still-defers"></a>
### walleye-display-late-probe-l3-tried-still-defers

Date
: 2026-11-19

Brief
: L3 (vdd-supply = l1a) tried once from slot b: shell up, 13 modules load, shell alive; c994000.dsi still deferred, no DRM card, no panel line

Question
: Does the DSI host bind with vdd-supply = vreg_l1a_0p875, does a DRM card appear, what does the connector say?

Found
: Gates: ufs-off-image pass, battery 4376 mV, sha256 ok. fastboot boot of boot-l3.img (nothing flashed, slot b): gadget 1d6b:0104 after 18 s, shell 2 s later. display-late-probe.sh: all 13 modules loaded, shell alive (stream carried the marker, netconsole did not). Reads: devices_deferred lists only c994000.dsi; msm-mdss, msm_dpu, msm_dsi_phy (c994400.phy) bound; /sys/class/drm has only 'version', no /dev/dri, no panel line; all of dsi's devlink suppliers (pinctrl, pmic gpio, gcc, mdss, dpu, phy, glink) are 'available'. A manual bind of c994000.dsi (dd.c/dsi dyndbg on) shows really_probe, one mdss runtime resume/suspend, then EAGAIN with no message: the defer is silent and comes after the parent's resume, so it is in the host's own gets (regulators, clocks) or the phy/panel lookup. l1a as vdd did NOT clear it (or is not the one missing). No ftrace in this kernel (no tracefs), so the return site is not named. Phone back on Android slot b, idle-android pass. Material: /home/mll/Projects/walleye-build/runs/l3-1795095458. Next: read the dsi host's gets in the msm dsi driver source (msm_dsi_host_init: regulator bulk vdd/vdda, clocks, phy) against our dsi node and name which can defer; candidates: a 0.9 V vdd rail other than l1a, the vdda rail (l2a), a core clock from gcc/mmcc; or add pr_info/dev_err_probe in a try image.

About
: walleye-display-s6e3fa5


<a id="walleye-display-dsi-defer-pinctrl-stubs"></a>
### walleye-display-dsi-defer-pinctrl-stubs

Date
: 2026-11-19

Brief
: The dsi's silent defer is the vendor overlay's pinctrl-0/1 on dsi@c994000 pointing at our /vendor-overlay-stubs, before dsi_dev_probe runs; proven on the host

Question
: Which call in msm's DSI host probe defers silently on L3, against our dsi node and the regulators the kernel has registered?

Found
: Source read of v7.3 (drivers/gpu/drm/msm/dsi/dsi.c, dsi_host.c, dsi_manager.c, drivers/base/dd.c, drivers/pinctrl/devicetree.c) against the L3 dtb merged with the vendor dtbo overlay, and the L3 stream log.
1. The defer is not in the msm DSI driver at all. Every EPROBE_DEFER inside dsi_dev_probe/msm_dsi_host_init either prints (dsi_get_phy, DRM_DEV_ERROR) or goes through dev_err_probe (clocks, parse_dt, ioremap, OPP), which records a reason in devices_deferred; only devm_regulator_bulk_get_const returns bare. And dsi_dev_probe's own DBG("") and dsi_init's DBG("dsi probed=%p") never printed in L3's stream, where msm_dsi_register, mdss_runtime_resume and the PHY's dsi_pll_10nm_init did (same dyndbg, same macro): the driver's probe body never ran.
2. What runs before it, per defer-with-no-message: really_probe -> pinctrl_bind_pins(dev) (dd.c:684) -> then platform_probe: of_clk_set_defaults, dev_pm_domain_attach -> then dsi_dev_probe. The manual-bind read (really_probe, one mdss resume/suspend from pm_runtime_get_suppliers, then EAGAIN) fits a failure at pinctrl_bind_pins, right after the supplier get.
3. Found: the bootloader merges the vendor dtbo overlay 3 into our dtb. Its fragment@89 targets mdss_dsi0 (= our dsi@c994000, same label) and adds pinctrl-names "mdss_default","mdss_sleep", pinctrl-0 = <&mdss_dsi_active>,<&mdss_te_active>, pinctrl-1 = <&mdss_dsi_suspend>,<&mdss_te_suspend> (and lab/ibb/vci/wqhd-vddio supplies, qcom,platform-*-gpio, qcom,dsi-pref-prim-pan). Those four labels are external references (the overlay's __fixups__), resolved against our /__symbols__, where our generated overlay stubs define them as empty nodes under /vendor-overlay-stubs. fdtoverlay of l3.dtb with entry.3 (host, no phone) gives dsi@c994000 pinctrl-0 = <0x15c 0x169>, pinctrl-1 = <0x15d 0x16a>: phandles of /vendor-overlay-stubs/mdss_dsi_active, mdss_dsi_suspend, mdss_te_active, mdss_te_suspend. The phone's own listing of the running dsi node (why-no-drm-reads4.txt) shows those properties on it: pinctrl-0, pinctrl-1, pinctrl-names, lab-supply, ibb-supply, wqhd-vddio-supply, qcom,platform-*-gpio, qcom,dsi-pref-prim-pan.
4. dt_to_map_one_config walks the parents of each pinctrl-N target looking for a pin controller; a stub's parents are /vendor-overlay-stubs and the root, so it returns -ENODEV, turned into -EPROBE_DEFER because CONFIG_MODULES is on ("keep deferring if modules are enabled"). create_pinctrl returns it up through pinctrl_bind_pins; really_probe turns it into a deferral with no dev_err_probe, so devices_deferred has the dsi with an empty reason, forever (a new probe is tried whenever another driver binds, and fails the same way). That is every fact the keeper saw: suppliers 'available', phy bound, empty reason, no card, EAGAIN right after the mdss resume/suspend.
5. L3's vdd-supply = l1a is still needed (msm8998's dsi config lists vdd and vdda) but was never reached; the first defer is pinctrl.
6. Ranked list of what to check first: (1) pinctrl stubs, proven on the host by the merge above and by tests/host/dtb-display-pinctrl-labels.sh, which FAILS today; (2) after it clears, of_clk_set_defaults on assigned-clock-parents from the phy (silent -EPROBE_DEFER if the PHY's clock provider is missing; the PHY bound and printed its PLL, so unlikely); (3) dev_pm_domain_attach for power-domains rpmpd VDDCX (QCOM_RPMPD=y in the kernel config; the DPU and mdss with the same domain bound, unlikely); (4) the regulator gets (vdd now l1a, vdda l2a, both registered; would print a warning if missing); (5) later binds not read yet: dsi_bind's devm_drm_of_get_bridge for the panel endpoint (the panel's own probe), DPU's component. A trace image (the planned L4 function_graph) is not needed for (1).
7. The change for L4, in the display dts (dt/msm8998-google-walleye-display.dts), not the kernel: make the four labels name real tlmm groups instead of stubs. /delete-node/ the four stub labels (or leave them out of the generated stubs: build/gen-overlay-stubs.sh), and put the labels on the real groups: mdss_dsi_active on mdss-dsi-active-state, mdss_dsi_suspend on mdss-dsi-suspend-state, mdss_te_active and mdss_te_suspend on mdss-te-state (one group serves both TE states; a state never selected claims no pin). The overlay names its states mdss_default/mdss_sleep, not 'default', so pinctrl_bind_pins finds no default state and applies nothing: the pins stay with our panel node's own pinctrl, and no pin has two owners. Keep L3's vdd-supply.
8. Not decided by this reading, to read after L4: whether other nodes of ours that the overlay targets carry pinctrl or other phandles into stubs (the same silent defer): the test above is for the dsi only; a generic version walks every overlay fragment whose target is one of our 12 labels.

Sources
: linux-v7-3-pinctrl-devicetree,linux-v7-3-driver-core-dd

About
: walleye-display-s6e3fa5

Material
: research/walleye-display-s6e3fa5/2026-11-19/


<a id="walleye-overlay-stub-refs-other-nodes"></a>
### walleye-overlay-stub-refs-other-nodes

Date
: 2026-11-19

Brief
: Overlay 3 hangs stub references on one of our nodes only (dsi@c994000, dsi@c996000 disabled); the other 10 targets are overlay-made nodes no driver binds

Question
: Does the vendor overlay point pinctrl, supplies, clocks or gpios of any other node of ours at /vendor-overlay-stubs, the same silent defer as the dsi?

Found
: Host only: fdtoverlay of l3.dtb with dtbo entry 3, then every cell of the merged dts equal to a phandle under /vendor-overlay-stubs (list in the material). Before the merge: 0 references. After: 93 raw hits; 40 in phandle-bearing properties (pinctrl-N, *-supply, dmas, memory-region, *-source, extcon, *-gpio); the rest are numbers that equal a phandle (interrupts, opp-level, reg, qcom,smem, one interrupts-extended irq 0x186). 1. Of our nodes (they exist in l3.dtb) only the two dsi hosts carry any: dsi@c994000 pinctrl-0/1 = the four mdss_* stubs (the known defer) and vci-supply, wqhd-vddio-supply = stubs pm8998_l22, pm8998_l14; dsi@c996000 the same, status disabled so nothing binds it. msm's dsi driver asks for vdd and vdda, not vci or wqhd-vddio, so those two supplies defer nothing today; the panel driver will ask for its own. 2. The other 10 nodes with references exist only in the overlay: i2c@c179000 (qcom,i2c-msm-v2, apart from our qup node, tests/host/i2c-qup-after-merge.sh), bt_wcn3990, camera-flash@10, mnh-sm, ptn36241g, access_ramoops@0/1, qcom,ion-heap@30, mi2s-quat, rmtfs_rtel_sharedmem. Their compatibles have no mainline driver in our kernel, so nothing probes them and a stub reference defers nothing; when one of them is wanted (bluetooth wcn3990 supplies pm8998_l7/l17/l25/s3/s5 and pmi8998_bob_pin1; i2c_5 pinctrl) its references must be made real then. 3. So the silent-defer class is closed by L4's four labels: no other node of ours has it. 4. tests/host/dtb-overlay-stub-refs.sh is the generic check: our enabled nodes, pre-probe properties (pinctrl-N, clocks, power-domains, phys, resets, iommus) at stubs fail; supplies are listed as notes. It fails today on l3.dtb (dsi@c994000 four pinctrl, notes vci/wqhd-vddio) and passes once L4's labels name real groups (not run on an L4 dtb yet: not known yet).

Sources
: linux-v7-3-pinctrl-devicetree

About
: walleye-display-s6e3fa5

Material
: research/walleye-display-s6e3fa5/2026-11-19/


<a id="walleye-modem-way"></a>
### walleye-modem-way

Date
: 2026-11-19

Brief
: Modem: mainline has the MSM8998 MSS driver; OnePlus 5 runs it with qrtr/rmtfs/pd-mapper/tqftpserv, calls broken; nobody shows a Pixel 2 modem

Question
: How can the Pixel 2 modem (calls, SMS, data) work on mainline?

Found
: q6v5-mss for msm8998 is upstream (2019). Closest working phone, OnePlus 5 (same SoC): modem up, ModemManager sees it only with the ipa module held back, calls/SMS broken, remoteproc crashes after Wi-Fi disconnects. Vectors: (1) remoteproc-only proof, blobs from the factory image, userspace rmtfs/pd-mapper/tqftpserv/qrtr, ~8 h, proof MSS ready + qrtr-lookup; (2) ModemManager SMS/registration with a SIM, ~10 h; (3) data over IPA by hand, ~15 h, unstable on OnePlus 5; (4) wait for OnePlus 5 fixes. Not known yet: modem partition layout and rmtfs files on walleye, whether the Wi-Fi diag-router workaround is needed, any working Pixel 2 modem. Unread: pmOS walleye wiki (blocked by Anubis), kcxt/linux-wahoo. Chosen: vector 1 then 2, read-only until walleye-ufs-readonly settles.

Sources
: mss-msm8998-driver-1665cbd5,pmaports-oneplus5-mainline-mr3531,modemmanager-issue-386,pmos-msm8998-wiki

About
: modem-calls


<a id="walleye-sensors-way"></a>
### walleye-sensors-way

Date
: 2026-11-19

Brief
: Sensors: all sit behind the SLPI; mainline has remoteproc_slpi for msm8998 (disabled, no fastrpc node); the SDM845 libssc/hexagonrpcd recipe is the way

Question
: How can the Pixel 2 sensors work on mainline when the SLPI owns their buses?

Found
: Our vendor-kernel-hardware-inventory says the AP tree has no sensor chips, so direct i2c drivers are probably not possible (not proven). Mainline msm8998.dtsi has remoteproc_slpi at 5800000, disabled. pmaports MR 4050 gives the SDM845 stack (SLPI remoteproc, FastRPC, hexagonrpcd, libssc, iio-sensor-proxy) with firmware from the user's own phone. Vectors: (1) full stack ~20-40 h; (2) direct i2c, likely blocked; (3) SLPI + qrtr + libssc only, first milestone; (4) defer. Unknowns: SLPI boots under PAS with Pixel firmware; the memory carve-out differs (0x94b00000/7 MiB stock vs 0x94c00000/15 MiB HTC); msm8998 FastRPC node not known yet. Chosen: 3 as the first milestone toward 1.

Sources
: lkml-slpi-msm8998-pas-2019,pmaports-mr4050-sdm845-sensors,pmos-msm8998-wiki

About
: sensors


<a id="walleye-camera-way"></a>
### walleye-camera-way

Date
: 2026-11-19

Brief
: Cameras: mainline camss has no MSM8998, no IMX362/IMX378 driver, no Pixel VCM driver; far from working, sdm660 camss is the base

Question
: How can the Pixel 2 cameras (IMX362 rear, IMX378 front) work on mainline?

Found
: Nothing exists for walleye. Del Regno's SDM630/660 camss (VFE 4.8, CSID 5.0) is said to be 90% reusable for msm8998. The sdm670-mainline Pixel 3a effort has out-of-tree IMX355/363 and is blocked on a VCM driver. Vectors: (1) port msm8998 camss on the sdm660 base plus an IMX362 driver, 40-80 h, one raw frame; (2) rebase sdm670-mainline patches, 30-60 h; (3) front IMX378 only, no VCM/OIS; (4) libhybris, not researched, not upstream. Pixel Visual Core probably not needed for raw capture (inference). Chosen: vector 1 to one raw frame, first step host-only: diff the wahoo downstream camera dtsi against the sdm660 camss tables; it waits behind UFS and modem, sensors.

Sources
: camss-sdm630-660-v2,sdm670-mainline-camera-vcm-issue,imx378-rfc-2020

About
: camera-rear


<a id="walleye-nfc-way"></a>
### walleye-nfc-way

Date
: 2026-11-19

Brief
: NFC: PN553 is a mainline nxp-nci device (nxp,pn553 binding); the work is a dts node from Android's dtb; the PN80T secure element is out of reach

Question
: How can the Pixel 2 NFC work on mainline?

Found
: nxp-nci with nxp,pn553 and enable-gpios is mainline. Vector A (chosen): node with I2C address, enable/IRQ GPIOs and supplies read from the Android dtb on the host, neard/libnfc, proof a tag poll, ~4 h, upstreamable. B: secure element PN80T, closed, rejected. C: skip. Unknown: whether the PN553 needs firmware download.

Sources
: lkml-nxp-pn553-binding

About
: nfc


<a id="walleye-fingerprint-way"></a>
### walleye-fingerprint-way

Date
: 2026-11-19

Brief
: Fingerprint: the sensor model is not confirmed (CrucialTec reported); no libfprint path known; identify it first, then decide

Question
: How can the Pixel 2 fingerprint reader work on mainline?

Found
: Press report says CrucialTec module; Pixel 3 uses FPC1075; not confirmed for walleye. No mainline or libfprint support known; Android's HAL may need a TrustZone trustlet. Vectors: A identify sensor and bus from the vendor dtb, host only, ~1 h, proof a named compatible; B userspace SPI driver with libfprint, protocol probably proprietary and TEE-gated, rejected for now; C defer. Chosen: A then decide.

Sources
: biometricupdate-pixel2-crucialtec

About
: fingerprint


<a id="walleye-deep-sleep-way"></a>
### walleye-deep-sleep-way

Date
: 2026-11-19

Brief
: Deep sleep: msm8998 has PSCI idle states upstream (latency fix v5.16); whether s2idle/RPM sleep reaches deep states is not known; measure Android's suspend current first

Question
: How can the Pixel 2 reach deep sleep and low standby current on mainline?

Found
: Upstream has PSCI cpuidle states for msm8998; pmOS lists CPUidle Partial; a OnePlus 3T write-up found cpuidle-psci disabled in the config. Not known yet: s2idle vs PSCI suspend state, RPM sleep sets and XO shutdown, Android's suspend current. Vectors: A enable PSCI cpuidle + s2idle and read rpm_stats/cpuidle residency, ~6 h, risk of resume hang; B Android baseline suspend current first, needs a meter and the author's hands; C cpuidle only. Chosen: B then A, after the PMIC watchdog proof.

Sources
: lkml-cpuidle-msm8998-latency-fix,hackernoon-oneplus3t-mainline-pm,pmos-msm8998-wiki

About
: deep-sleep


<a id="walleye-display-late-probe-l4-built"></a>
### walleye-display-late-probe-l4-built

Date
: 2026-11-19

Brief
: L4 built: L3 with the four overlay pinctrl labels on real tlmm groups, their stubs removed; test passes (fails on L3), gate and AVB pass, rebuild identical, reviewed; not tried

Question
: Build L3 with mdss_dsi_active, mdss_dsi_suspend, mdss_te_active, mdss_te_suspend naming the real tlmm groups, not stubs

Found
: boot-l4.img sha256 1dfeebe2c1b6867a0f3fb9f18094e3755e9933cee668dbb177d5af27ed1750d9 (~/Projects/walleye-build/images-display-late-l4/) = L3 with only the dtb changed: /__symbols__ of the four labels now point at /soc@0/pinctrl@3400000/mdss-dsi-active-state, mdss-dsi-suspend-state and mdss-te-state (twice), and the four empty stubs under /vendor-overlay-stubs are gone (l4.diff: 4 stub nodes, 3 group lines gain labels, 4 symbol lines). dt/msm8998-google-walleye-display.dts now gives the same on a fresh build (/delete-node/ the four stubs, labels on the groups; compiled against wt-k-fg, same symbols). L3's vdd-supply, ramdisk, command line, size, Image.gz are kept. tests/host/dtb-display-pinctrl-labels.sh fails on L3's dtb and passes on L4's (and on the dtb inside boot-l4.img); dtbo-overlay-merges.sh and touch-gpio-single-claimant.sh pass; UFS-off gate and AVB pass; dtb and image rebuild identical. Programmer review: no blocker; re-diff from the images added to the check. Not checked by the reviewer: that the overlay names its states mdss_default/mdss_sleep (from research walleye-display-dsi-defer-pinctrl-stubs). gen-overlay-stubs.sh needed no change: it makes stubs only for labels our dtb lacks. Build: build/build-display-late-l4-dtb.sh, build/series/display-late-l4.toml (lpoco-bringup image bisect), build/check-display-late-l4.sh. The try is walleye-try-display-late-probe-l4.

About
: walleye-display-s6e3fa5


<a id="walleye-fingerprint-identify-dtb"></a>
### walleye-fingerprint-identify-dtb

Date
: 2026-11-19

Brief
: Fingerprint: vendor dtb names fpc,fpc1020, irq GPIO86, reset GPIO128, vdd_io S4, no SPI node; CrucialTec is the module, FPC the chip

Question
: Which sensor and which bus is the Pixel 2 fingerprint reader, from the vendor dtb?

Found
: Vendor dtbo (all walleye/taimen entries): fp_fpc1020, compatible fpc,fpc1020, irq tlmm86, reset tlmm128, vdd_io-supply pm8998_s4. No SPI bus in the node: the vendor driver (fpc1020_platform_tee.c) only does gpio and wakelock, the data path is a TrustZone trustlet. CrucialTec (press) = module maker, FPC = chip. Exact chip and SPI controller unknown: found by an SPI read of HWID. Precedent: emainline read HWID of an FPC1140C on msm8996 with a small SPI driver. libfprint has no phone-SPI FPC driver. Vectors: A SPI HWID probe from a test kernel (cheap, answers bus + chip + TZ lock); B full userspace driver on libfprint (protocol/image capture proprietary, large, after A); C defer. See research/walleye-fingerprint/2026-11-19/README.md.

Sources
: biometricupdate-pixel2-crucialtec,emainline-fingerprint-p1,fpc1020-tee-android-kernel,libfprint-supported-devices

About
: fingerprint

Material
: research/walleye-fingerprint/2026-11-19/

Supersedes
: walleye-fingerprint-way


<a id="walleye-touch-program-unattended-run-2"></a>
### walleye-touch-program-unattended-run-2

Date
: 2026-11-19

Brief
: touch-try.rs unattended run 2 from Android, no hand step: steps 1-10 pass, EVENTS-READY, stops at ask 11 'nobody answered'; phone back on Android slot b

Question
: Does touch-try.rs now run unattended from Android with the enter-fastboot step in?

Found
: Yes. Checks 1-3, step 4 (enters fastboot itself), step 5 boot from memory, checks 6-8 (window 120 s, restart at 180 s), step 9, check 10 EVENTS-READY all pass; stops at the ask with 'nobody answered'. Nothing flashed, slot b. After the 180 s self-reboot idle-android passes (slot _b, adb up). stdin /dev/null. Note: lpoco lab suite latest still needs ~/.config/devlab/google-walleye/password (absent), so the suite was not read.

About
: walleye-attend-program-touch


<a id="walleye-display-late-probe-l4-tried-dsi-binds-panel-defers-on-reset"></a>
### walleye-display-late-probe-l4-tried-dsi-binds-panel-defers-on-reset

Date
: 2026-11-19

Brief
: L4 tried once from slot b: c994000.dsi now probes and binds; the panel defers on reset-gpios (cannot find GPIO chip gpiolib_shared.proxy.6); no DRM card yet

Question
: With the four overlay pinctrl labels on real tlmm groups (L4), does the dsi probe, does a DRM card appear, does the panel bind?

Found
: Gates: ufs-off-image pass, sha256 ok, battery 4377 mV, slot b. fastboot boot of boot-l4.img (nothing flashed): gadget 1d6b:0104 after 18 s, shell 2 s later; display-late-probe.sh: all 13 modules load, shell alive (stream carried the marker, netconsole did not). dsi_dev_probe ran, 'dsi probed', msm_dsi_host_init 'Dsi Host 0 initialized': the dsi left the deferred list (the pinctrl stubs were the silent defer). Now devices_deferred lists only 'c994000.dsi.0 panel-samsung-s6e3fa5: Failed to get reset-gpios'; dmesg: 'cannot find GPIO chip gpiolib_shared.proxy.6, deferring', then 'No GPIO consumer reset found'. The panel's reset-gpios is tlmm 94 (dt/msm8998-google-walleye-display.dts:88); the shared-GPIO proxy chip it looks up does not exist. /sys/class/drm has only 'version', no /dev/dri, no connector: no 'console and connector read'. Phone back on Android slot b, idle-android pass. Material: /home/mll/Projects/walleye-build/runs/l4-1795098032. Next: find why the proxy for tlmm 94 never registers (another consumer of pin 94 in dtb or overlay, or gpiolib-shared not built) and fix it in an L5; one try.

About
: walleye-display-s6e3fa5


<a id="walleye-z21-touch-events-pass"></a>
### walleye-z21-touch-events-pass

Date
: 2026-11-19

Brief
: z21 touch on a person's swipe: Synaptics S3708AR probed, 3593 EV_ABS events read (4358 lines); phone back on Android

Question
: Does the RMI4 touch device on z21 make EV_ABS events when a person touches the glass?

Found
: Attended run of tests/attended/touch-try.rs once, at the author's Go (relayed by the Top Coordinator, go until 16:18): battery, image and UFS-off checks passed; booted z21 from slot b, reader showed EVENTS-READY for 120 s; the author swiped the whole glass and tapped the corners and said ok (relayed). The reader saw 4358 event lines, touch-check.sh: pass walleye-touch-events, 3593 EV_ABS events from 'Synaptics S3708AR'. Phone returned to Android by itself (adb device). Log: /home/mll/lab-material/walleye/touch-try-1795098876.log (not in git).

About
: walleye-first-fastboot-boot


<a id="walleye-tas2557-v3-vs-porthole"></a>
### walleye-tas2557-v3-vs-porthole

Date
: 2026-11-19

Brief
: Boiano's v3 TAS2557 (binding + driver, on the list) fits walleye's two amps and the same TI firmware format; porthole's port has no binding and a one-node hack, so start from v3

Question
: Which TAS2557 driver should walleye's series start from: Boiano's v3 on linux-sound, or porthole's port (0024, 0162, 0171-0174)?

Found
: Both drive the same part with the same TI uCDSP firmware container (both parse the 'TAS firmware' image; walleye still needs the phone's own tas2557s_PG21_uCDSP.bin, never in git).
v3 (2026-07-17, linux-sound): binding ti,tas2557 with Reviewed-by Krzysztof Kozlowski; one node, reg = <0x4c>, <0x4d> for a stereo pair (DEV_A/DEV_B by reg order), four required supplies (vbat, iovdd, avdd, dvdd), reset-gpios, interrupt, optional firmware-name ('defaults to a name derived from the silicon revision'), ti,imon/vmon-slot-nos. Driver 3067 lines: mono and dual-device stereo, fault-interrupt recovery, word length from the stream. Exercised on Xiaomi Mi A2, Redmi Note 6 Pro, Redmi 5 Plus (mono) and Mi Pad 4 (stereo, SDM660). Mark Brown's review of the driver (07-19) still open: board-specific startup GPIO table, firmware device-count check, PLL index range error, sense mute on suspend, use_pmdown_time. No v4 found by 2026-11-19. Not merged.
Porthole (taimen, 7.2.2): driver 1347 lines (0024) plus control patches, Assisted-by: LLM, no binding, no supplies; its dts describes the pair as ONE node at reg 0x4d with the driver claiming 0x4c itself and reset-gpios on pm8998 gpio index 5; picks the firmware by REV_PGID (0xa0 = PG2.1 -> tas2557s_PG21_uCDSP.bin). Measured on taimen: both addresses ACK, PG2.1.
Fit to walleye: same wahoo board family, so the two-amp pair at 0x4c/0x4d on a shared reset is the likely shape (unread on walleye: its own dts has no audio nodes; the keeper's i2c read, walleye-haptics-i2c-answers sibling, settles it). v3's binding describes that shape properly; porthole's node would not pass dt-binding review.
Unknowns: (1) v3 on the PG2.1 firmware with MI2S quaternary on msm8998 - never run on Qualcomm 8998; its startup table wires ASI2 GPIOs that may not match the pixel; (2) whether v3 selects tas2557s_PG21_uCDSP.bin by itself, or needs firmware-name in the dts; (3) the amps' supplies on walleye (vbat/iovdd/avdd/dvdd) - the vendor dts names them. All three are settled by building v3 and one boot try, none by the phone first.
Licence: v3 is Boiano's own code with a Signed-off-by; porthole's is declared ported from the downstream wahoo driver, so v3 is the cleaner base for the lab's clean-room line.

Sources
: lkml-boiano-tas2557-v3,lkml-broonie-tas2557-v3-review,porthole-pmaports-taimen

About
: walleye-audio-haptics


<a id="walleye-display-late-probe-l5-built"></a>
### walleye-display-late-probe-l5-built

Date
: 2026-11-19

Brief
: L5 built: L4 with label mdss_dsi0 on a disabled stub, so the overlay's reset-gpio <tlmm 94> no longer makes the panel's gpio shared; test fails on L4, passes on L5

Question
: Why is the gpiolib_shared proxy for the panel reset (tlmm 94) missing, and what image fixes it?

Found
: gpiolib-shared.c makes a gpio shared when two available nodes name it; the proxy is a module (GPIO_SHARED_PROXY=m, GPIO_SHARED=y) not loaded in the late-probe image. Claimants of tlmm 94 after the overlay merge on L4: the panel reset-gpios and the vendor overlay's qcom,platform-reset-gpio on dsi@c994000 (fragment 89; dsi1, fragment 90, is disabled so skipped). L5 puts the label mdss_dsi0 on a new stub under the disabled /vendor-overlay-stubs, so the overlay's properties land unscanned (lever of z21). tests/host/display-reset-gpio-single-claimant.sh: 2 claimants on L4, 1 on L5. Programmer review: the overlay's other uses of mdss_dsi0 (fragments 12, 38) are downstream-only and harmless; fixes applied. Proof is host-side fdtoverlay; the bootloader uses libufdt, so the phone try is the witness. Alternative not taken: load gpio-shared-proxy.ko. sha256 boot-l5.img fb813a9a702e11a2b3d6bd0b981dceecfb21fe0ca7947a03ebe45d62ed6c2ef5. The shared dts is not changed (label mdss_dsi0 is the dtsi's); L5 patches L4's dtb like L1..L4.

About
: walleye-display-s6e3fa5


<a id="walleye-display-late-probe-l5-tried-dsi-connected-1080x1920"></a>
### walleye-display-late-probe-l5-tried-dsi-connected-1080x1920

Date
: 2026-11-19

Brief
: L5 tried once from slot b: the panel binds, DRM card0 appears, connector DSI-1 connected 1080x1920; shell alive, 13 modules load

Question
: With the mdss_dsi0 label moved off the dsi node (L5), does the panel's reset gpio resolve, does the panel bind and a DRM card appear?

Found
: Gates: sha256 fb813a9a..., ufs-off-image pass, battery 4376 mV, slot b. fastboot boot of boot-l5.img (nothing flashed): gadget 1d6b:0104 after 18 s, shell 2 s later; display-late-probe.sh pass: all 13 modules load, shell alive; stream carried the marker, netconsole did not. The line 'cannot find GPIO chip gpiolib_shared.proxy' is gone (0 mentions in dmesg). dsi probed, Dsi Host 0 initialized, the panel c994000.dsi.0 found its reset-gpios, msm_dpu bound c994000.dsi, DRM card0 and renderD128 exist (/dev/dri), /sys/class/drm/card0-DSI-1 status connected, modes 1080x1920; the DPU atomic commit for the 1080x1920 mode was in the stream. devices_deferred empty. Not read: whether the glass lit (no eyes; the author not asked), the panel's own init sequence replies. Phone back on Android slot b by itself after /tmp/hold removed; idle-android pass. Material: /home/mll/Projects/walleye-build/runs/l5-1795100051 (not in git). Next: read the panel prepare/enable lines and a modetest or fbcon frame; ask the author to look only once the console and DRM state are read.

About
: walleye-display-s6e3fa5


<a id="walleye-display-stripes-l6-built"></a>
### walleye-display-stripes-l6-built

Date
: 2026-11-19

Brief
: L6 built: L5 plus a static stripe drawer (fbdev write, pan, forced commit); ramdisk control equals L5's, gate and AVB pass, rebuild identical, reviewed; not tried

Question
: Which image draws four stripes on the panel, unattended, with the connector read back?

Found
: L6 = L5 plus /bin/walleye-stripes in the ramdisk; kernel, dtb, command line, header words and size are L5's. The drawer (tools/walleye-stripes, Rust, argenv) writes four stripes (red, green, blue, white) to /dev/fb0 (fbdev emulation of card0), then pans, forces a set_par commit and unblanks (each result logged; a command-mode panel needs a commit), holds --seconds, clears to black (release runs even if the draw failed); log lines at draw and release go to stdout and /dev/kmsg; connector status/enabled/modes and fb0 name/modes are read before and after. Chosen over modetest (no aarch64 build) and a libdrm dumb-buffer program (needs libdrm; fbdev needs only ioctls). Started by tests/host/display-late-probe.sh after the settle (STRIPES_SECONDS, default 20; the try's proof asks for at least 60, so set STRIPES_SECONDS=60); selftest extended (drawer present, absent, 0 s, nonzero exit) and passes. Checks: cargo test (frame; kernel struct sizes 160/80), fmt, clippy clean; drawer rebuilt twice identical; image: ramdisk without the drawer equals L5's byte for byte, file list gains only bin/walleye-stripes, AVB verifies, UFS-off gate and tests pass, image rebuilt identical. Programmer review found a wrong fb_var_screeninfo size (152, not 160: a stack overrun), the unlogged pan, kmsg double writes, a draw left standing on failure: all fixed. Not known: whether the fbdev write reaches the glass; the try says. sha256 boot-l6.img 8ca3af879b4c9d87ff98f8d269282557131ca8eef5d39bc42d5a97eb5e587bad; walleye-stripes 7a4648a96352abf0bff4787c733e5606711e52b8c94fb4555fc999a9677e0f16.

About
: walleye-display-s6e3fa5


<a id="walleye-display-stripes-l6-tried-draw-calls-ok"></a>
### walleye-display-stripes-l6-tried-draw-calls-ok

Date
: 2026-11-19

Brief
: L6 tried once from slot b: fb0 msmdrmfb, connector connected/enabled 1080x1920; pan, commit, unblank return 0; panel init commands sent; whether the glass lit not known

Question
: With L6, unattended, does the stripe drawer draw and hold on the panel, and what does the panel path say?

Found
: Gates: sha256 8ca3af87..., ufs-off pass, battery 4377 mV, slot b. fastboot boot of boot-l6.img (nothing flashed): gadget 1d6b:0104 after 18 s, shell 2 s later. display-late-probe.sh with STRIPES_SECONDS=60 (material /home/mll/Projects/walleye-build/runs/l6-1795102107, not in git): stream carried the marker, netconsole did not. The drawer logged fb0 msmdrmfb 1080x1920 32bpp, connector connected/enabled/1080x1920 before the draw; FBIOPAN_DISPLAY, FBIOPUT_VSCREENINFO (force) and FBIOBLANK (unblank) each returned 0; draw line 'four stripes held 60 s' at kernel 442 s. The stream shows dsi_mgr_bridge_pre_enable, then eight dsi_cmds2buf_tx ret=50 with dsi_host_irq after each (the panel's init commands went out; no error line among them), then DPU commits with vblank counts rising through 500 s. 117 s after the draw began the shell found no walleye-stripes process and the connector still connected/enabled. The host script reported fail 'drawer did not end with 0': its telnet session for the drawer ended before the 60 s hold, so the drawer's exit code and release line were not captured (script flaw to fix: run the drawer detached with its log in a file, read after). Not known: whether the glass lit (author not asked), the drawer's exit code, whether the hold was a full 60 s. Phone back on Android slot b by itself after /tmp/hold removed; idle-android pass, battery 4333 mV. Next: a drawer run detached in the script; the attended look only after that.

About
: walleye-display-s6e3fa5


<a id="walleye-drv2624-image-built"></a>
### walleye-drv2624-image-built

Date
: 2026-11-19

Brief
: drv1 built: z21 with the DRV2624 series (driver built in) and vibrator@5a; 36 checks ok, rebuild identical, reviewed; not tried

Question
: Can the lane's test kernel carry the DRV2624 series in a UFS-off boot image?

Found
: boot-drv1.img sha256 96e85804db5478aae4525266997dfcf65f6990493cbb00ac3398114cc3501e8b: z21's ramdisk/header/size, kernel v7.3 + pmic-wdt + fuel gauge RFC + kernel/drv2624 0001-0004 as commits (tree walleye-build/wt-k-drv), dts vibrator@5a (reset gpio 95, motor values) from dt/. UFS-off gate and AVB pass, rebuild identical, tests/host/drv2624-builds.sh passes with the dtb. Programmer review: added checks for patch 0004 in the kernel and the motor values. Not booted. The keeper reads ls /sys/bus/i2c/devices, the 6-005a driver link and dmesg over the mainline shell.

About
: walleye-audio-haptics


<a id="walleye-drv1-tried-drv2624-probes"></a>
### walleye-drv1-tried-drv2624-probes

Date
: 2026-11-19

Brief
: drv1 once from slot b: DRV2624 answers at 0-005a (c17a000.i2c, i2c-0), id 0x03, input drv2624:haptics registered; phone back on Android slot b

Question
: Does the DRV2624 on the Pixel 2 answer on I2C with the drv1 image, and does the driver bind?

Found
: Gates passed right before (battery 4376 mV, sha256 of boot-drv1.img, ufs-off-image.sh, idle-android). adb reboot bootloader, lpoco lab fastboot boot now: left fastboot 15 s, gadget 1d6b:0104 at 18 s, telnet 172.16.43.1 shell, kernel 7.3.0-00011-g25cc4008afc3-dirty. dmesg: 'drv2624 0-005a: DRV2624 id 0x03' at 1.069 s, then 'input: drv2624:haptics' at .../c17a000.i2c/i2c-0/0-005a/input/input1; Synaptics rmi4 1-0020 also up. So the part is on the bus numbered i2c-0 (the dts label i2c_6 is not the Linux number): the item's 6-005a path does not exist, 0-005a does. Id register reads back 0x03 through the driver's probe. Not read: which rail powers it (the dts reset gpio 95 and the vibrator@5a node are the only supplies named), and no vibration was commanded (no force-feedback effect played). Material ~/lab-material/walleye/drv1-try1 (answers.txt). The init's 180 s timer restarted the phone though /tmp/hold was touched (as z21); Android back on slot b, adb up, 4376 mV, nothing flashed.

About
: walleye-audio-haptics


<a id="walleye-drv2"></a>
### walleye-drv2

Date
: 2026-11-19

Brief
: drv1 again: 0-005a bound to driver drv2624, name drv2624, of_node vibrator@5a, only supplier the pinctrl; no regulator linked; Android back on b

Question
: Is the DRV2624 at 0-005a bound to its driver, and what supplies it?

Found
: Second drv1 try (gates: sha256, ufs-off-image.sh, battery 4377 mV). In the shell: /sys/bus/i2c/devices/0-005a/driver -> drivers/drv2624, name drv2624, of_node soc@0/i2c@c17a000/vibrator@5a, dmesg 'DRV2624 id 0x03' at 1.16 s and input1 drv2624:haptics registered. Supplier links: only platform:3400000.pinctrl; no regulator supplier (the dts names none), so the rail is not known: the chip answers, its power source is read as always-on or shared, not measured. No effect played. Phone back on Android slot b, stock 4.4.302, adb up, 4370 mV after the init's 180 s restart; nothing flashed. Material ~/lab-material/walleye/drv1-try1/answers2.txt.

About
: walleye-audio-haptics


<a id="walleye-display-stripes-l6-tried-2-drawer-held-60s"></a>
### walleye-display-stripes-l6-tried-2-drawer-held-60s

Date
: 2026-11-19

Brief
: L6 tried a second time, drawer detached: exit 0, held 60.3 s, every ioctl 0, connector connected/enabled before and after; whether the glass lit still not known

Question
: With the drawer detached, does it end with 0 and hold the full 60 s on L6, unattended?

Found
: Gates: sha256 8ca3af87 ok, ufs-off pass, battery 4376 mV, slot b. fastboot boot (lpoco lab fastboot boot now, nothing written): gadget 1d6b:0104 after 18 s, shell 2 s later. display-late-probe.sh STRIPES_SECONDS=60 (material /home/mll/Projects/walleye-build/runs/l6b-1795105982, not in git): the script reported fail because its 70 s read came while the drawer still ran; read again ~45 s later the log ends stripes-rc=0-end. Kernel stream: draw line at 441.12 s, release pan at 501.45 s = 60.3 s hold. FBIOPAN_DISPLAY, FBIOPUT_VSCREENINFO (force), FBIOBLANK (unblank) = 0; release clears to black; fb0 msmdrmfb and connector connected/enabled 1080x1920 before and after. Script flaw: the drawer reaches its hold ~12 s after the settle and the fixed sleep of STRIPES_SECONDS+10 leaves too little margin; the read should poll for stripes-rc up to a bound. Netconsole UDP carried no marker again, TCP stream did. /tmp/hold removed, phone back on Android slot b by itself: idle-android pass, battery 4383 mV. Not known: whether the glass lit (author not asked; the attended try's question).

About
: walleye-display-s6e3fa5


<a id="walleye-modem-blobs-found"></a>
### walleye-modem-blobs-found

Date
: 2026-11-19

Brief
: Pixel 2 modem firmware found read-only: modem_b is a FAT image with image/modem.mdt+b00-b27, mba.mbn, qdsp6m.qdb; rmtfs partitions read

Question
: Where are the modem blobs and rmtfs partitions of the Pixel 2, and can they be read without writing the phone?

Found
: With Android on slot b, adb root (Android is throw-away; there is no su), dd of /dev/block/by-name to ~/lab-material/walleye/modem-2026-11-19/ (mode 700, outside git), nothing written to the phone: modem_a and modem_b (80 MiB each, identical sha 4ae0e793), fsg 1.5 MiB, fsc 1 MiB, modemst1 and modemst2 2 MiB each. modem_b is a FAT16 image: /image holds modem.mdt (8348 B) with b00-b27, mba.mbn (234152 B), qdsp6m.qdb, modemr.jsn, version.cfg (2020-06-09 build). So the remoteproc_mss firmware set (mba.mbn + modem.mdt/bNN) is on the phone's own partition and reaches the test ramdisk from ~/lab-material, never git. rmtfs reads modemst1/2, fsg, fsc: keep read-only. Still to do: a kernel image with remoteproc_mss, rmtfs memory and the qcom_q6v5_mss node enabled in the dts (research walleye-modem-way), then one test-ramdisk try.

About
: modem-calls


<a id="walleye-display-stripes-look-result"></a>
### walleye-display-stripes-look-result

Date
: 2026-11-19

Brief
: The author's look at the display procedure: the glass lights on mainline, fbcon text is visible, the stripes flashed briefly but did not hold

Question
: What did the author see on the Pixel 2's glass during the display stripes procedure?

Found
: The author's words, about 17:50 UTC 2026-11-19, relayed by the Top Coordinator, verbatim: "oh whit, i see the console output some persentage, also i saw it brifly blimp with red blue and green". Reading: (1) the glass LIGHTS on mainline, the first picture of ours on this phone; (2) fbcon text is visible on it (the console output, with a percentage, that is the kernel log on the glass); (3) the stripes showed red, blue and green (white not named) only briefly and did not hold for the 60 s the drawer held (exit 0, all ioctls 0). OPEN QUESTION: what took the framebuffer back from the drawer: fbcon repainting the console over fb0, or the drawer's own hold (item walleye-display-stripes-brief). The procedure's 'no answer' (research walleye-display-stripes-try2-no-answer) is a timing fault of the relay, not of the glass or the author: he answered at once, the Top Coordinator's relay sat queued while the keeper was blocked in its wait command and arrived after the timeout.

About
: walleye-display-s6e3fa5


<a id="walleye-display-stripes-try2-no-answer"></a>
### walleye-display-stripes-try2-no-answer

Date
: 2026-11-19

Brief
: Display stripes procedure ran for real, the drawer drew; the author's answer reached it after the timeout, not unmeasured: see walleye-display-stripes-look-result

Question
: Does the Pixel 2 show four vertical stripes (red, green, blue, white) from procedures/display-stripes-on-the-pixel-2.rs?

Found
: 2026-11-19 18:47-18:53: battery 100, image sha b70feabf, UFS-off check ok, fastboot, shell answered, 13 modules loaded, DSI connector connected, drawer drew. LOOK NOW sent to the Top Coordinator (local_12b18c70) at 18:49:30; no relayed answer by the hold plus 30 s, so 'no' was fed per the item's orders. The FAIL therefore means NO ANSWER, not that the author saw dark glass: unmeasured, not broken (measured-not-claimed). Phone back on Android (phone-state android). Log: ~/lab-material/walleye/display-stripes-1795110464.log. CORRECTION 2026-11-19: the look is NOT unmeasured. The author answered at once (about 17:50 UTC); the relay reached the keeper after its timeout because the keeper was blocked in a long wait command and a message reaches a session only at a tool round. His words and the result are in research walleye-display-stripes-look-result: the glass lights, fbcon text visible, the stripes flashed briefly.

About
: walleye-first-fastboot-boot


<a id="walleye-display-stripes-try2-author-saw"></a>
### walleye-display-stripes-try2-author-saw

Date
: 2026-11-19

Brief
: Display try 2: the author saw the console text with a percentage and a brief red, blue, green flash; no steady stripes (relayed late, after the run)

Question
: What did the author see on the glass during the display stripes try of 2026-11-19 18:49?

Found
: The author's words, relayed by the Top Coordinator after the run had ended: 'oh whit, i see the console output some persentage, also i saw it brifly blimp with red blue and green'. So the glass lights (the panel, DSI and fb0 path works) and shows the fbcon text console; the stripes appeared only briefly, not for the 60 s hold. Answer as it would have been fed: not ok - console text with a percentage and a brief flash of red, blue, green, not a steady stripe pattern. Likely cause, from the source and not measured: tools/walleye-stripes writes /dev/fb0 once then sleeps; the kernel has CONFIG_FRAMEBUFFER_CONSOLE=y and DRM_FBDEV_EMULATION=y, ignore_loglevel and dyndbg on the cmdline, and the drawer's own kmsg lines, so fbcon repaints text over the stripes as messages print. The log of this run holds no dmesg from the hold, so fbcon vs pan/unblank is NOT yet read. To measure: bring the stripes up with fbcon unbound (echo 0 > /sys/class/vtconsole/vtcon1/bind, or console loglevel 1 and no kmsg lines during the hold) and capture dmesg during the hold.

About
: walleye-first-fastboot-boot

Supersedes
: walleye-display-stripes-try2-no-answer


<a id="walleye-display-at-boot-b1-built"></a>
### walleye-display-at-boot-b1-built

Date
: 2026-11-19

Brief
: B1 built: L5 with init-display, which loads the 13 display modules itself at boot, no host script; checks pass, rebuild identical, reviewed; not tried

Question
: Can the display come up at boot with no host script?

Found
: B1 = L5's kernel, dtb, command line and modules; only /init differs (init-display = init-late plus a background subshell insmod-ing every line of /display-modules.list, each step to /dev/kmsg, failures logged with rc; a missing list is said). The self-reboot is held 600 s at boot so the glass can be looked at, then released. Gate, AVB, UFS-off tests, reset-gpio single claimant pass; rebuild identical; programmer review done (test made behavioural, missing-list and hold findings fixed). sha256 boot-b1.img 57ab1b69de78a304e0fba1b2576a1781de2b9cbc41a02c2d73b5328c67b8ebd7 in ~/Projects/walleye-build/images-display-at-boot. Kernel unchanged; fbcon takes the early log when msm loads. Not tried.

About
: walleye-display-s6e3fa5


<a id="walleye-display-stripes-brief-fbcon-repaints-the-stripes"></a>
### walleye-display-stripes-brief-fbcon-repaints-the-stripes

Date
: 2026-11-19

Brief
: The drawer's own FBIOPUT_VSCREENINFO makes fbcon repaint the console over the stripes (host, v7.3 source); the commits are not the cause; fix: KD_GRAPHICS while held

Question
: Why did the author see console output and a brief red, blue, green flash, not four held stripes, when the drawer held 60 s with every ioctl 0: fbcon repainting over fb0, a missing vtcon unbind, or the drawer's pan/commit?

Found
: Read from the v7.3 tree the image is built from; nothing tried on the phone. (1) The drawer writes the stripes into fb0 (msm fbdev, deferred damage via the drm fb helper, so the glass gets the stripes at once - the author's red-blue-green blip), then calls FBIOPUT_VSCREENINFO with activate = FB_ACTIVATE_NOW|FORCE (128). fb_set_var_from_user (fbmem.c) then calls fbcon_update_vcs; FORCE is not FB_ACTIVATE_ALL (64), so fbcon_modechanged runs, which ends in update_screen(vc): the whole visible console is drawn again cell by cell, background included, over the stripes. That is the 'console output some percentage': the stripes were wiped by the console within milliseconds of being drawn, by the drawer's own call. (2) Any later console write repaints too (scrolling is a redraw), and FBIOBLANK unblank reaches fbcon_fb_blanked -> do_unblank_screen -> fbcon_blank -> update_screen when the console was blanked; so during the 60 s hold fbcon is a second writer. (3) The pan and the commit are not at fault: every ioctl returned 0 and the blip shows a commit reaches the glass. A vtcon unbind (echo 0 > /sys/class/vtconsole/vtconN/bind, N the one named 'frame buffer device') would also stop it but detaches fbcon from every fb; the standard way is what X and Wayland compositors do: KDSETMODE KD_GRAPHICS on /dev/tty0 for the hold. In KD_GRAPHICS fbcon_modechanged, fbcon_fb_blanked and the vt's console print all return early (vc_mode != KD_TEXT), so fbcon draws nothing; entering runs do_blank_screen(1), which calls fbcon_blank with mode_switch (marks graphics; that it does not blank the panel is not proven here). KD_TEXT afterwards restores the console. The image's console= is ttyGS0 only, so where the glass's console text comes from is not known yet: the try reads /proc/consoles and the vtcon names and binds. Not known: that the glass holds the stripes once fbcon is silenced - only the attended try says.

Sources
: linux-v7-3-fbcon-fbmem

About
: walleye-display-s6e3fa5


<a id="walleye-ufs-solution-table"></a>
### walleye-ufs-solution-table

Date
: 2026-11-19

Brief
: Android's UFS path against the matched image: 7 equal, 1 carried, 7 harmless, 2 unknown (version, the pulse's effect); the erase cause stays unpublished

Question
: Step by step, how does the matched UFS image compare with Android's bring-up of H28U74301AMR after the seven gap items?

Found
: TABLE: Android's UFS bring-up of this card (SK hynix H28U74301AMR) step by step against the matched image (kernel topic/ufs-match-android 3644090c7; image sha256 461077869fb275ab8803b8bb771050d4f19724ccfd62e197dc1f3fe3a6297df5, rebuilt with the prints, identical rebuild, never booted). Read from Google's wahoo 4.4 GPL source (android-msm-wahoo-4.4-android11 @52750b8f), the live dmesg and dt, Linux v7.3 and our tree; nothing run on the phone. Marks: EQUAL / CARRIED (same effect through another route) / HARMLESS (differs, judged harmless, with the source) / UNKNOWN (the question the try answers).

1. Host reset before enable. Android: HCE 1->0 then 1. Image: mainline ufshcd_hba_execute_hce stops an active host and restarts it. EQUAL (research walleye-ufs-gap-abl-handover-read).
2. Card reset pulse, RST_n low then high 10-15 us before the host starts. Android: yes (pinctrl ufs_reset). Image: yes (pin 153 made reachable by the overlay, reset-gpios). EQUAL. UNKNOWN in effect: the one step no capture covers; four accidental unpulsed probes (z15, z16, z17, z20) read the card fine, none was ever pulsed. Pulse is short and standard on mainline UFS-boot hosts.
3. Reset pad drive/pull. Android: bias pull-down, drive 8 = HDRV 3 = the power-on value. Image: rides with the pulse; pad left at its default. HARMLESS (research walleye-ufs-gap-perf).
4. Rails. Android and image: host 1.8/2.96/1.2 V, PHY 0.88/1.2 V, l26 always on. EQUAL.
5. Clocks. Android: hardware-gated clocks. Image: plain clocks. HARMLESS (power only).
6. PHY reset 1 ms, table load, release, serdes start, PCS ready. Android: table 3.0.0 or 3.1.0 by UFS_HW_VERSION. Image: mainline's sdm845 table (what mainline uses for MSM8998; 10 lines differ from the vendor 3.1.0 table, 35 from 3.0.0), the host version read and printed at ufs_qcom_init before PHY init. UNKNOWN: which version; Android's side cannot tell (no print, no readable trace); the image's own print settles it (research walleye-ufs-version-from-source).
7. Lane 1. Android: the 3.0.0 table programs lane 0 only, 3.1.0 adds lane 1; nothing else touches lane 1; the live link reaches gear 3, 2 lanes. Image: mainline table already writes both lanes. HARMLESS: if lane 1 stays down the link still comes up and reads work; only the 2-lane step fails, a risk to speed, not to the card (research walleye-ufs-gap-lane1).
8. ICE. Android: inited after PHY, before link startup; touches no link, PHY, reset or card registers, issues no command. Image: no ICE node; mainline warns and returns. HARMLESS (research walleye-ufs-ice-verdict); an upstreamable port only if inline encryption is ever wanted (item walleye-ice-port).
9. Link-startup pre-change: timers, core clock cycles, LCC off, hardware clock gating. Android: yes. Image: yes through mainline (same registers, a fixed 150 vs scaled to the core clock rate). EQUAL / CARRIED.
10. Link-startup pre-change: RX line config on (PHY+0x130 bit 1) and the PA_VS_CONFIG_REG1 SaveConfigTime mask 0x6000. Android: yes. Image: no, and no mainline Qualcomm host that boots from UFS does either. HARMLESS (decision 2026-11-19-walleye-ufs-linkstartup-do-not-carry). The image now prints PHY+0x130 after PHY init (the value after our init, not an abl leftover) and HCS/HCE; if link startup fails with the bit set to disabled the fallback is one reviewed PHY write.
11. Card quirks. Android: only HOST_PA_SAVECONFIGTIME applies to this model (the vendor Hynix entries are prefixes that do not match). Image: SAVECONFIGTIME plus our purge guard. EQUAL in effect.
12. Sony's quirks (DELAY_BEFORE_LPM, EXTEND_SYNC_LENGTH 0x48, ONE_TX_LANE / AVAILTXDATALANES). Android: never writes or reads them on this card. Image: carries none (a host test counts zero in the tree). EQUAL (decision 2026-11-19-walleye-ufs-no-sync-length-quirk). The old z13 image carried them and was moved away from.
13. Purge. Android: never sent (live: only fDeviceInit set). Image: guard blocks it and the ufs_bsg path that sets the flag is never opened. EQUAL.
14. Link speed. Android: HS gear 3, rate B, 2 lanes each way, after a first link at gear 1 one lane. Image: mainline negotiates the same. EQUAL as far as read; shown only at the try.
15. Bus votes, PM-QoS. Android: yes. Image: no (no msm8998 interconnect provider in mainline, no vote and no error). HARMLESS: performance only; a read-only probe runs at gear 1 one lane (research walleye-ufs-gap-perf).
16. Mainline extras (clock scaling, write booster, runtime autosuspend). Image has them, Android does not. HARMLESS, standard mainline.
17. What the bootloader left. Android and image both reset the host, so the abl's host state is moot; the card was up and in use in all four accidental probes. Image adds the pulse. UNKNOWN only in the sense of item 2.
18. Commands the probe sends: host reset, RST_n pulse, link startup, NOP, fDeviceInit flag set, descriptor reads, power-mode change, SCSI scan (INQUIRY, TEST UNIT READY, READ CAPACITY). No write, unmap or purge. Android sends all of these and more.


COUNT: EQUAL 7 (items 1, 2, 4, 11, 12, 13, 14), CARRIED 1 (9), HARMLESS 7 (3, 5, 7, 8, 10, 15, 16), UNKNOWN 2 (the version, item 6; the pulse's effect on this card, items 2 and 17), 1 item is the commands list (18). The perf research drops four rows from the old gap table (bus votes, PM-QoS, gated clocks, pad drive).

WHAT NO TABLE CAN CLOSE: why Sony's yoshino (a similar Hynix card) lost its bootloader with a probe lacking quirks: the mechanism is not published (research walleye-ufs-gap-sony-mechanism-public-record); our sequence contains none of Sony's fixes because Android's own does not, and cannot send a purge on any path we can read. The Pixel 2's Hynix firmware revision is not readable without root. No EDL way back is known for walleye (research 2026-11-18-walleye-edl-way-back).

Sources
: src-google-wahoo-msm44-android11,wahoo-4-4-ufs-qcom-c,wahoo-4-4-ufs-qmp-v3,src-mainline-v73-ufs-qcom-ice,src-sony-ufs-quirks-forward-port,src-sony-bugtracker-802

About
: walleye-ufs-match-android-z


<a id="walleye-display-at-boot-b1-tried-unattended-ok-backlight-fails"></a>
### walleye-display-at-boot-b1-tried-unattended-ok-backlight-fails

Date
: 2026-11-19

Brief
: B1 tried once from fastboot, nothing flashed: shell up by itself, 13 of 13 modules said ok by the init, card0 DSI-1 connected, fb0 msmdrmfb; backlight enable fails -110; glass not looked at

Question
: Does B1 bring the display up at boot with no host script?

Found
: Unattended run of procedures/display-at-boot-on-the-pixel-2.rs (image sha 57ab1b69, UFS off, battery ok): gadget 1d6b:0104 came up, init held its self-reboot, 'walleye-display: ok' for all 13 modules, card0 and card0-DSI-1 connected, console switched to fb0 135x120. 249 display lines recorded (~/lab-material/walleye/display-at-boot-*.log). Kernel log shows dsi_err_worker status=c and dsi_cmds2buf_tx cmd dma tx failed type=0x15 data0=0x51 ret=-110, so drm_panel_enable 'failed to enable backlight: -110' (the 0x51 brightness write). The run stopped at the author's look (nobody answered); no way back was run, the init's 10-minute deadman restarted the phone and Android is back (adb device). Whether the glass lit is not known yet.

About
: walleye-display-s6e3fa5


<a id="walleye-ufs-card-revision"></a>
### walleye-ufs-card-revision

Date
: 2026-11-19

Brief
: Pixel 2 UFS card: SK hynix H28U74301AMR, firmware rev D003, six LUNs alike; vendor kernel exposes no descriptor files; risk of this revision: not known

Question
: What vendor, model and firmware revision is the Pixel 2's UFS card, and is that revision a risky one (Sony yoshino erasure)?

Found
: Read only on live Android with adb root, 2026-11-19: /sys/block/sda..sdf/device vendor 'SKhynix', model 'H28U74301AMR', rev 'D003' (all six LUNs). The vendor kernel's ufshc sysfs (/sys/devices/soc/1da4000.ufshc) holds only clkgate, rpm/spm_lvl, slowio, latency_hist: no device, string, geometry or health descriptors, no ufs_bsg touched. No writes, resets, fstrim. Raw text in ~/lab-material/walleye/ufs-card-2026-11-19/sysfs.txt (600, never git). Compared with the Sony record (XZ1 Samsung KLUCG4J1 fw 0200 probed without erase; SODP purge-flag suspicion, mechanism unpublished): this is a different maker and revision, and no public report of erasure on SK hynix D003 was found in our records, so it is NOT KNOWN to be risky and NOT KNOWN to be fine; the matched image sends no purge, which is the one mechanism suspected.

About
: walleye-ufs-match-android-z


<a id="walleye-ufs-go-note"></a>
### walleye-ufs-go-note

Date
: 2026-11-19

Brief
: Go note for the author, refreshed after the gaps: 18 steps against Android, 7 equal, 1 carried, 7 harmless, 2 unknown; erase cause unpublished, no EDL; the go is his

Question
: How close is the matched image to Android's bring-up of H28U74301AMR now, what is unknown, what could a failure cost?

Found
: NOTE FOR THE AUTHOR, refreshed 2026-11-19 after the seven gap items (plain words; the go is yours, no session lifts the hold - decision 2026-11-19-walleye-ufs-hold-second-path). The full step table is research walleye-ufs-solution-table.

WHAT THE TRY IS: boot one image (NOT-TO-BOOT-ufs-on-ufs-match.img, sha256 461077869fb2..., rebuilt with read-only prints, identical rebuild, never booted) from slot b whose kernel brings up the phone's UFS card (SK hynix H28U74301AMR, 64 GB). Nothing is written to the card by us. It prints the UFS version, the host state (HCS/HCE) and one PHY register, then reads the card's identity and partitions.

THE ANSWER FROM ANDROID, in 18 steps: 7 equal, 1 carried by mainline with the same effect, 7 differ but are judged harmless (each with its source), 2 unknown.
- Equal to Android: host reset, the card's reset pulse, the voltages, the card quirks (only SAVECONFIGTIME; our purge guard on top), Sony's three erase-related fixes absent (Android never uses them on this card), purge never sent, HS gear 3 / 2 lanes negotiated by mainline.
- Differs, harmless: clocks without hardware gating, no inline encryption (cannot touch link or card), no bus votes or PM-QoS (speed only), Android's RX-line and one timing-mask step at link start (no mainline Qualcomm host that boots from UFS does them), lane 1's table entries (a down lane 1 costs speed, not the card), mainline's extras.
- Unknown 1: the host version (3.0.0 or 3.1.0). Android cannot show it; the image prints it first thing.
- Unknown 2: the card's reset pulse on this card. Four earlier accidental probes read the card fine WITHOUT a pulse; the matched image is the first to pulse it, as Android does. Short, standard on other UFS phones, but not seen here.
So: the image matches everything that touches the card's identity, reset and link rules; it leaves out Android's speed and power tuning on purpose.

STILL NOT KNOWN AND NOT KNOWABLE FROM ANDROID: why Sony's yoshino phone (a similar Hynix card) lost its bootloader with a probe lacking quirks. Nobody has published the mechanism. Our sequence sends no write, no unmap and no purge on any path we can read, and holds none of Sony's fixes because Android's own does not. READ NOW (2026-11-19, read-only on live Android as root, nothing written; research walleye-ufs-card-revision): the card is SK hynix H28U74301AMR, firmware revision D003 on all six units. Whether D003 is a risky revision is NOT KNOWN: Sony's yoshino XZ1 that was probed without an erase carries a different maker and revision (Samsung, 0200), and no public report ties SK hynix D003 to an erase; the matched image sends no purge, the one mechanism suspected.

RISK AND COST: chance not zero, not measured. If the bootloader were erased, fastboot goes with it; no EDL way back for the Pixel 2 is known (research 2026-11-18-walleye-edl-way-back): the phone would be a brick for us and this lane would end. Lowering it: the four accidental probes read the card, nothing written, the phone still boots Android from both slots.

WHAT HAPPENS WHEN YOU SAY GO: one command (item walleye-attend-program-ufs, being made now: read-only, dry-run first) boots the image once, reads the version and the card, and takes the way back. The try itself is item walleye-try-ufs-match.

YOUR OPTIONS: (a) go on the matched image, one try, read-only; (b) wait for an EDL way back first; (c) no go.

About
: walleye-first-fastboot-boot


<a id="walleye-backlight-write-minus-110-source-read"></a>
### walleye-backlight-write-minus-110-source-read

Date
: 2026-11-19

Brief
: The 0x51 brightness write fails because it is the first DSI DMA command sent while the DPU streams; read from source, not yet proven on the phone

Question
: Why does the panel's brightness write fail with -110 on B1, and is the glass affected?

Found
: Source (v7.3) and the keeper's B1 log. (1) 'failed to enable backlight' is drm_panel_enable's backlight_enable() after panel->funcs->enable, an INFO line only: it does not stop the pipe. Our driver registers the backlight in probe (brightness 255) but writes nothing then; the first 0x51 write is this one, after prepare (reset, sleep out, TE on, 0x53=0x20, display on) succeeded with the same host and LPM. (2) -110 is dsi_cmd_dma_tx's 200 ms dma_comp timeout (DMA-done irq never came). (3) dsi_err_worker status=c = FIFO(4)|MDP_FIFO_UNDERFLOW(8): the MDP pixel path was running into the DSI FIFO when the command was sent; the worker's dsi_sw_reset then kills the pending DMA. msm_dsi_host_xfer_prepare carries the upstream TODO 'make sure dsi_cmd_mdp is idle'; DSI 6G >= 1.2 sets BLOCK_DMA_WITHIN_FRAME, so the DMA waits for the frame end, which never comes if the frame waits for a TE that does not arrive. (4) Order: pre_enable (prepare, writes ok, DPU idle), DPU encoder enable starts the first frame, then panel enable and the backlight write hits it. Hand-loaded L6 differs in order or timing (not read yet). Causes, ranked: A the write collides with the first MDP frame; B TE never arrives so the frame and the blocked DMA never end (dtbo TE/mdp_vsync pin, DPU te source); C a DPU underflow from a too-low clock or bandwidth vote. Glass: panel is sleep-out and display-on, but brightness stays at the panel's power-on level (not known; 0x51 matters), so B1's glass may be dark or dim; not known yet whether it lit. Cheapest settling by the keeper, no flash: after boot write /sys/class/backlight/*/brightness (DPU idle) - works means A, -110 again means B or C; read dmesg for dsi_err_worker repeats and DPU pp-done timeouts.

Sources
: linux-v7-3-msm-dsi-host

About
: walleye-display-at-boot


<a id="walleye-ufs-try1-card-readable"></a>
### walleye-ufs-try1-card-readable

Date
: 2026-11-19

Brief
: UFS try 1 on the matched image: the card is readable on mainline: version 3.1.0, six LUNs SKhynix H28U74301AMR D003, partition tables read, no UFS errors; the link idles at HS gear 1 on 2 lanes

Question
: What did the matched image read from the Pixel 2's UFS card, and was anything wrong?

Found
: Read from the keeper's own files (~/lab-material/walleye/ufs-20261119-210513: dmesg.txt, link.txt) by the Top Coordinator and the walleye coordinator, 2026-11-19 about 21:05 CET, one boot of boot image sha256 461077869fb2 from slot b, read only. dmesg: UFS_HW_VERSION 3.1.0 (so the 3.0.0 questions are closed); UFS_HOST_LEFT HCS 0x0000010f HCE 0x00000001 (what the bootloader left); qcom-qmp-ufs-phy: MSM8998 PHY table 3.1.0; UFS_PHY_LINECFG_DISABLE 0x00000000 (the RX line config bit is clear); inline encryption disabled as designed; scsi host0 with the well-known LUNs and six LUNs 0 to 5, each SKhynix H28U74301AMR firmware D003; the partition tables read (sda with 30 or more partitions, sdb to sdf); no UFS error lines. link.txt: gear HS_GEAR1, lanes 2, mode FAST_MODE, rate HS_RATE_B, link_state HIBERN8, dev_pm SLEEP: the link is up and works but idles at gear 1; the power-mode change to HS-G3 (Android's gear) is not seen. The phone is back on Android (adb device) after the try; nothing was written. The procedure marked FAIL only at its check 'the link is HS-G3 on two lanes', which is speed, not a fault of the card (the gap research already said so: research walleye-ufs-gap-lane1). Unknown-1 (host version) and the reset-pulse unknown are closed by this: the version is 3.1.0 and the card answered after the matched image's reset pulse.

About
: walleye-ufs-match-android-z


<a id="walleye-ufs-match-tried-version-ok-link-stays-gear1"></a>
### walleye-ufs-match-tried-version-ok-link-stays-gear1

Date
: 2026-11-19

Brief
: Matched UFS image tried once, read only: UFS_HW_VERSION 3.1.0, PHY table 3.1.0, host enabled; link stays HS_GEAR1 2 lanes HIBERN8 for 170 s, no HS-G3

Question
: Does the matched UFS image read the version, load the PHY table and reach HS-G3 on 2 lanes, probe only?

Found
: One try from slot b, nothing written, go on the item until 21:39. Kernel log: ufshcd-qcom UFS_HW_VERSION 3.1.0, HCS 0x10f HCE 1, qmp-ufs-phy MSM8998 PHY table 3.1.0, LINECFG_DISABLE 0. Check 10 failed after 170 s: gear HS_GEAR1, lanes 2, FAST_MODE, state HIBERN8, so no gear change to G3 yet; LUN listing and purge checks of the PROCEDURE not reached (it stopped at check 10). CORRECTION 2026-11-19 (from dmesg.txt of the try, ~/lab-material/walleye/ufs-20261119-210513): the KERNEL enumerated all six LUNs (sda 15584256 4096-byte blocks, 59.4 GiB, sdb to sdf) as SKhynix H28U74301AMR D003 and read their partition tables by 1.64 s (sda1 to sda22 and beyond, sdb1, sdd1 and on, sdf1 to sdf5); no UFS error line; nothing written; Android booted afterwards. See research walleye-ufs-try1-card-readable. The one try the author's go bought is spent; a second try or anything that writes needs a new go from him. Phone returned to Android (adb up). No retry, no gate edit. Log: ~/lab-material/walleye/ufs-match-try.log

About
: walleye-first-fastboot-boot


<a id="walleye-display-l7-built"></a>
### walleye-display-l7-built

Date
: 2026-11-19

Brief
: Image L7 built: L6 with the KD_GRAPHICS stripe drawer; control L6 rebuilt byte for byte, UFS-off gate and AVB pass, rebuild identical; not tried

Question
: Can the author's look run on the known-good L6 load order with the held-stripes drawer, as one command with a way back after any stop?

Found
: L7 = L6 (modules loaded by the host procedure in order) with /bin/walleye-stripes replaced by B2's drawer (aarch64 static, sha256 ee8ad01e8f62..., built twice identical; KDSETMODE KD_GRAPHICS while held, KD_TEXT after, also after a failed draw). build/build-display-l7-image.sh: 18 checks ok - L6's own sha256, the ramdisk repacked with L6's drawer equals L6's byte for byte (control; LATE_DIR out-display-late, the L4 dir fails the control), kernel/cmdline/header/size/init-late unchanged, file list unchanged, AVB footer verifies, UFS-off gate and tests pass, panel reset gpio one claimant; boot-l7.img sha256 643d1f83c6ae..., rebuilt with the second drawer build into another dir: identical (cmp). Procedure display-stripes-on-the-pixel-2.rs on lpoco-attended 0.35.1: look 1 the kernel log text, look 2 four stripes held; a real run is watched (walleye_tries::watch) so the way back (reboot -f, sysrq b, power-hold sentence) runs after any stop incl. 'nobody answered'; the hold's deadman now outlasts two 600 s answers. 117 crate tests, clippy -D warnings, fmt, lpoco-attended run --dry-run (18 steps) and check --host ok. Never booted: the keeper's unattended run for real is the open proof.

About
: walleye-display-s6e3fa5


<a id="walleye-display-l7-look1-console-seen"></a>
### walleye-display-l7-look1-console-seen

Date
: 2026-11-19

Brief
: L7 look 1: the author sees the console output on the Pixel again, on the watched procedure with the answer file

Question
: Does the author see the kernel log on the Pixel's glass during the L7 look procedure?

Found
: The author, verbatim, about 22:24 CET 2026-11-19, relayed by the Top Coordinator, who wrote 'ok' into ~/.cache/lpoco-attended/google-walleye/answer for prompt id 2717062-0 (the kernel log look): "i see the console output on pixel again". So: the glass lights again on mainline with the L7 image (L6's load order, hand-loaded by the procedure), the kernel log text is visible, and the answer file path works end to end between the Top Coordinator and the procedure. The second look, four stripes held in graphics mode, is answered the same way if he is still at the phone; its result is a separate row. Not yet known: whether the held stripes show.

About
: walleye-display-s6e3fa5


<a id="walleye-display-l7-looks-seen"></a>
### walleye-display-l7-looks-seen

Date
: 2026-11-19

Brief
: L7, both looks seen by the author: the kernel log on the glass and four held vertical stripes: the display passes on mainline

Question
: Does the author see the kernel log and the four stripes on the Pixel's glass through the L7 procedure?

Found
: The author, verbatim, relayed by the Top Coordinator, about 22:24 CET 2026-11-19: look 1 (prompt id 2717062-0): "i see the console output on pixel again"; look 2 (prompt id 2717062-1): "nice, i olso see 4 vertical color stripes !". Both answered by the Top Coordinator writing ok into ~/.cache/lpoco-attended/google-walleye/answer. So with L7 (the late-probe kernel, modules loaded by the procedure in L6's order, the held-stripes drawer in KD_GRAPHICS) the glass shows the kernel log and four vertical colour stripes that HOLD: the console repaint question (research walleye-display-stripes-brief) is answered by the KD_GRAPHICS fix. The first picture of ours on this phone. NOT yet true: the display at boot without a load order (B2 fails: pixel clock rcg warning, DSI -110, SMMU faults; research walleye-display-b2-dsi-clock-warning and item walleye-display-dts-any-order carry it). MORE OF HIS WORDS on the stripes, verbatim, relayed by the Top Coordinator: "they look perfect ! its beautiful !" - the colours and the geometry are right, no tearing or offsets noticed.

About
: walleye-display-s6e3fa5


<a id="walleye-display-b2-dsi-clock-warning"></a>
### walleye-display-b2-dsi-clock-warning

Date
: 2026-11-19

Brief
: B2's early load faults (SMMU, pclk0 rcg, DSI -110) are the bootloader splash still live at 2 s; walleye reserves no splash memory

Question
: Why B2 (display at boot) warns 'pclk0_clk_src: rcg didn't update', times out DSI commands and floods SMMU faults, when hand-loaded L5/L6 do not; and which dts dependency is missing

Found
: What the logs count (4 runs, same kernel, dtb, modules and order): msm loaded at about 2 s after boot (B, B1, B2) always gives 10 SMMU context faults at the splash buffer (iova 0x9d6f1500..0x9d771c00, cb 0, SID 0, linear scan), 4x 'pclk0_clk_src: rcg didn't update' (clk_pixel_set_rate from dsi_link_clk_set_rate_6g, run inside s6e3fa5_probe -> mipi_dsi_attach -> component bind -> first modeset), DSI DMA -110 and 'disable autorefresh failed'; msm loaded at 66 s (L6, by hand) gives none of them. So the fault is not the order of modules but the state the hardware is in at 2 s: the bootloader's display is still live.
Mainline facts (v7.3): msm8998.dtsi describes mdss/dsi/phy completely (iommus on mdss, assigned-clock-parents = phy pixel/byte PLL); OnePlus 5/5T and Fxtec Pro1 reserve the splash memory@9d400000 no-map (OnePlus also a simple-framebuffer with the mmcc clocks + MDSS_GDSC). walleye's dts reserves NOTHING at 0x9d400000, although the faulting iovas lie in it: the kernel may hand that memory out, and nothing tells the clock and SMMU code that the scan-out is in use.
Ranked hypotheses: H1 (most likely, one cause for all three signs) the bootloader's command-mode pingpong autorefresh is still scanning the splash with the SMMU stream in bypass; msm attaching its own paging domain makes every fetch fault, MDP never idles, DSI engine and pclk0 switch (the RCG update needs the running source: the bootloader's PLL is stopped by the phy reset while the rcg still sources it) fail, so DCS commands time out. By 66 s that state is gone (the bootloader's panel/PLL state is lost after the 30 s regulator late cleanup or the unused-genpd cleanup; unmeasured) so the cold start is clean. H2 the failure is only the pixel clock: the rcg is parked on the bootloader's PLL and the phy enable power-cycles it; the faults are a side show. H3 an init ordering problem (module order, phy not loaded yet): excluded by L6, which loads the same 13 modules in the same order. H4 iommu.passthrough hides a missing iommus on the dpu: excluded as the sole cause (L6 has it too and works), kept as a dts check.
Readings that tell them apart: (a) load msm at N s for N = 5, 15, 25, 35, 50 (the flip point names the cleanup that matters: 30 s = regulator_init_complete, 12 s = deferred probe timeout / sync_state, 1.3 s = genpd cleanup); (b) load at 2 s with iommu.passthrough off (faults then at attach or none?); (c) at 2 s, before loading msm, read-only: the mmcc pclk0 and byte0 RCG CFG/status, mdss registers via devmem (a keeper read) to see whether autorefresh is on; (d) the same load at 2 s with the splash region reserved (dts) and a simple-framebuffer taking the clocks.
Mainline-proper answer: the dts change that every board with a live bootloader splash carries: reserved-memory cont_splash_mem memory@9d400000 no-map (size from the vendor dts of wahoo/taimen, GPL), and where the splash must be kept alive until msm takes over, a simple-framebuffer on it with the mmcc clocks and MDSS_GDSC (the OnePlus way), so the handover is by deferred probe in any order: simpledrm/efifb is replaced by msm when it binds. No ordering list needed in init. What dts alone cannot do - a fault-free SMMU handover of an in-flight scan-out on cmd-mode autorefresh - is a driver matter (dpu: disable autorefresh and wait idle before attaching the domain, or arm-smmu keeping the boot stream mapping through reserved-memory iommu-addresses/RMR); to be settled by (a)-(d), then sent upstream as an item.

Sources
: kernel-oneplus-common-dtsi,kernel-msm8998-dtsi-mdss

About
: walleye-display-s6e3fa5

Material
: research/walleye-display-s6e3fa5/2026-11-19/


<a id="walleye-display-l7-unanswered-run-way-back-ok"></a>
### walleye-display-l7-unanswered-run-way-back-ok

Date
: 2026-11-19

Brief
: L7 run with no answer: stops at look 1 'nobody answered' after 120 s; the watcher sends the phone back; Android on slot b up ~24 s later, not the 600 s deadman

Question
: Does the L7 procedure send the phone back by its own way back when nobody answers (the path that failed on B2)?

Found
: Real run 2026-11-19 by walleye-keeper, lpoco-attended 0.37.1, --answer-limit 120, no answer file written (Top Coordinator told). Dry run 18 steps ok first. Checks 1-3 ok, phone to fastboot, L7 booted from memory (nothing flashed), gadget up, 13 modules loaded shell alive, /sys/class/drm/card0-DSI-1 connected enabled, look 1 asked at t+101 s, FAIL 'nobody answered within 120 s' at t+221 s (exit 1), prints 'the phone goes back to Android'. Phone on Android afterwards: slot _b, sys.boot_completed 1, uptime 80 s when read about 100 s after the stop, so it rebooted ~24 s after the stop, far inside the 600 s deadman. adb shows the phone, fastboot reachable. The glass was not looked at (nobody answered).

About
: walleye-display-s6e3fa5


<a id="walleye-ufs-why-gear1-ranked"></a>
### walleye-ufs-why-gear1-ranked

Date
: 2026-11-19

Brief
: Gear 1 at idle is most likely clock-scale-down (legacy path, no OPP table), not a missed G3: the link is already HS rate B, so the power-mode change was made

Question
: Why does the link idle at HS gear 1 and the change to HS-G3 is not seen: clock scaling at idle, the power-mode change not made, or the PHY table?

Found
: From try 1 files (link.txt: dev_pm=SLEEP gear=HS_GEAR1 lane=2 mode=FAST_MODE rate=HS_RATE_B state=HIBERN8; dmesg: PHY table 3.1.0, no UFS error, six LUNs read by 1.64 s) and v7.3 source (wt-k-ufsm). EXCLUDED by the files: 'power-mode change not made'. ufshcd_init_pwr_info leaves PWM G1, 1 lane, SLOWAUTO, hs_rate 0; the try shows FAST_MODE, rate B, 2 lanes, so ufshcd_config_pwr_mode ran and reached HS on both lanes; lane 1 and the PHY table (3.1.0 loaded, link up, partition tables read) are not what holds it. RANKED: (1) clock scale-down at idle, most likely: ufs-qcom sets CAP_CLK_SCALING, msm8998.dtsi has freq-table-hz (50/200 MHz core, 37.5/150 MHz unipro) and no operating-points table, so hba->use_pm_opp is false and ufs_qcom_freq_to_gear_speed returns UFS_HS_DONT_CHANGE (0); ufshcd_scale_gear then takes the legacy path and drops the gear to clk_scaling.min_gear = G1 when devfreq (polling 60 ms, down below load threshold) scales down after the boot-time partition reads; the sysfs gear is hba->pwr_info, which holds the scaled-down value; 170 s idle is long enough. (2) Gear capped below G3 by the host side: ufs_qcom_get_hs_gear returns G3 for host major <4 (3.1.0 here), so no mainline cap; a DT limit would show only if the board file sets a limit property (none seen in the match overlay): unlikely. (3) PHY table 3.1.0 against the vendor's: would show as link or read errors, not a quiet G1; unlikely, and G3 never attempted so also not proven. (4) Reading at hibern8 with dev_pm=SLEEP is only the sampling state, not a fault. Not yet known: clkscale_enable and devfreq state, power_info after a bounded read load, whether a scale-up to G3 happens under load. READ-ONLY readings for try 2: cat clkscale_enable and the devfreq node (cur_freq, available_frequencies, governor) of 1da4000.ufshc; power_info right after dd of the first MiB of sda (O_DIRECT, read only) and again after 2 s idle; a count of ufshcd power-mode / scale lines in dmesg with dyndbg on drivers/ufs/core/ufshcd.c (+p) for the try. If gear reaches 3 under load and falls back at idle: works, as Android does; if it never leaves 1 under load: look at devfreq and a missing opp (a DT operating-points-v2 table with the 150/37.5 MHz pairs mapped to gear in ufs_qcom_freq_to_gear_speed would then be the fix, an upstream-able DT change).

Sources
: src-mainline-v73-ufs-qcom-ice,src-google-wahoo-msm44-android11

About
: walleye-ufs-match-android-z


<a id="walleye-display-delay-bisect-flips-at-25s"></a>
### walleye-display-delay-bisect-flips-at-25s

Date
: 2026-11-19

Brief
: Delay bisect, one unattended run: B2's modules loaded at 5 s and 15 s show the signs; at 25, 35 and 50 s all three are zero and 13 of 13 modules ok

Question
: Between 2 s and 66 s of uptime, where do B2's three signs (SMMU faults, pclk0 rcg warning, DSI -110) stop?

Found
: Five RAM boots (B-d5..B-d50), nothing flashed, phone back on Android each time. Delay 5: SMMU faults 10, rcg 4, DSI -110 5. Delay 15: faults 10, rcg 0, -110 0. Delay 25, 35, 50: all 0. The flip is between 15 s and 25 s, so the splash handover (research walleye-display-b2-dsi-clock-warning) is over by 25 s; the SMMU faults outlast the rcg warning and -110. Table kept in ~/lab-material/walleye/display-delay-20261119-232559/table.txt. Next: narrow 15-25 s, or reserve the splash region with simple-framebuffer in the dts (decision 2026-11-19-walleye-display-early-load-splash-handover).

Sources
: walleye-display-b2-dsi-clock-warning

About
: walleye-display-s6e3fa5


<a id="walleye-display-flip-15-25-read-from-logs"></a>
### walleye-display-flip-15-25-read-from-logs

Date
: 2026-11-19

Brief
: The five delay boots show no kernel event between 12 s and 25 s: the faults are splash scanout at 0x9d6xxxxx through our new SMMU domain; the flip is the bootloader's display going idle

Question
: What happens between 15 s and 25 s that ends the SMMU faults?

Found
: From dmesg-d5/d15/d25 (~/lab-material/walleye/display-delay-20261119-232559): our kernel logs nothing between the 12.0 s sync_state() lines and the display-module load; d15 and d25 reach the same events in the same order, so no kernel event (deferred-probe timeout ~12 s, clk or regulator late init at 1.27-1.39 s, genpd) separates them: the difference is outside the kernel. The 10 faults of d15 come in one burst (15.3125-15.3135 s) right after 'dpu hardware revision' in dpu_kms_hw_init, iova 0x9d608500..0x9d68c200, level-2 translation faults, SID 0 on cd00000.iommu: reads inside the splash region (0x9d400000) by a scanout still running when msm attaches its SMMU domain. d25 binds the same dsi, same dpu init, no fault, fb0 msmdrmfb. The rcg and DSI -110 signs (d5 only) are the bootloader's DSI clocks still driven at 6 s. So two stages: DSI link clocks stop before 15 s, splash scanout before 25 s. Not in the logs: why (fastboot screen idle timeout or panel/DPU idle). Not measured: 17-23 s.

About
: walleye-display-s6e3fa5


<a id="walleye-pmic-wdt-v1-try-result"></a>
### walleye-pmic-wdt-v1-try-result

Date
: 2026-11-19

Brief
: PMIC watchdog v1 try on the Pixel 2: the reboot test passed; the bite reset the phone by itself in 20 s, but it came up in off-mode charging and Android started only after the author's power press

Question
: Does the PMIC watchdog v1 restart the Pixel 2 after a bite, and does the phone come back to Android by itself?

Found
: One real run, 2026-11-20 00:15 to 00:30 CET, procedure pmic-watchdog-v1-on-the-pixel-2 with --answer-limit 300, image A (next-based, the six v1 patches), the author at the phone, his answers written by the Top Coordinator. REBOOT TEST PASS: armed with reset type 0x07 (S2 off before, on after), a clean reboot, Android up and stayed up 180 s, S2 read OFF after it: patch 2 (stop on reboot) works. BITE PASS as a reset: 20 s after arming, unpetted, the shell went quiet and the phone restarted by itself: patch 1 (the reset type set with S2 off, read back) makes the bite a RESTART where z9, before the patch, powered the phone off. NOT as first recorded: the phone did not come back to Android by itself. The author, verbatim, about 00:31 CET: "it did restart into charging, it showed charging icon, now the screen is black", then "i pressed powerbutton, it boots android now". After the bite with the USB cable in, the bootloader boots off-mode charging and Android needs a short power press. The run's own check read the later Android. Open: item walleye-wdt-reset-boots-charger-mode.

About
: walleye-pmic-wdt-v1-image


<a id="walleye-wdt-reset-boots-charger-mode-how-it-stands"></a>
### walleye-wdt-reset-boots-charger-mode-how-it-stands

Date
: 2026-11-19

Brief
: A bite with USB in boots charger mode because the S2 reset type 7 (hard) power-cycles the PMIC and USB then counts as the power-on reason; the S2 type the firmware left (1, warm) is the one to try

Question
: Why does a PMIC-watchdog bite with USB in boot the bootloader's charger screen, and should the PON or the reset type make it warm-boot?

Found
: READ, not measured on the bite: (1) Qualcomm: the PMIC records WHY it powered on in PON_REASON1 (KPDPWR_N, CBLPWR_N cable, USB_CHG, DC_CHG, hard-reset bit) and PON_ON_REASON separates a warm reset (0x40) from a normal power-on (0x80); the vendor says WARM_RESET is for crash dumps, HARD_RESET for restarting the device. (2) Our driver writes S2 type 0x07 (hard): the PMIC then powers its rails down and up like a cold start, so with the cable in the power-on reason is the cable (USB_CHG/CBLPWR_N), not a key press or a warm reset. A bootloader that sees a charger reason and no key press shows its off-mode charging screen: that matches the author's words (charging icon, black screen, Android only after a short power press). (3) The firmware had left S2 at type 0x01 (warm), read by the driver at probe ('S2 reset control was 0x1', booted 2026-11-20 run); mainline's own constants are 0x01 warm, 0x04 shutdown, 0x07 hard. A warm type keeps the PMIC peripherals on and sets the warm-reset bit, which bootloaders of this family read as 'not a cold boot' and do not send to the charger screen (the Qualcomm LK rule, recalled, NOT read: walleye's bootloader is closed and its rule is not confirmed here). (4) Android reported boot reason [reboot] after the later power press: that is Android's reading of the second boot, and says nothing of the bite's reason. NOT KNOWN YET: the PON_REASON/POFF/ON_REASON registers right after the bite (nobody read them before the second boot: the run read them only before arming); why z9 without patch 1 powered off while the firmware type was 1 (maybe the type was never read back there); what stock Android's own watchdog bite does on this phone (stock reset type). OPTIONS: A) arm S2 with warm type 0x01 (the firmware's own): smallest change, the series already has the reset-type hook; risk: the earlier z9 power-off; B) keep hard 0x07 and accept the charger screen when USB is in (the phone does restart; user-visible only with a cable); C) a PON bit or boot-reason handover before the bite (PON_SOFT_RB_SPARE as qcom-pon.c uses for reboot modes): works only if the bootloader reads it, unproven; D) leave it to the user space: nothing. LEAN: A, tried first because it is the firmware's own value, read-only experiment first (item walleye-wdt-bite-pon-reasons-read). Does not block the series: the bite proof stands.

Sources
: qcom-pmic-pon-reset-status,qcom-pmic-boot-guide-reset-types,vendor-qpnp-power-on-c,mainline-pm8916-wdt-c,android-bootstat-reasons

About
: walleye-pmic-wdt-v1-image


<a id="walleye-display-splash-image-built"></a>
### walleye-display-splash-image-built

Date
: 2026-11-19

Brief
: Splash image B-s5 built, never booted: B-d5 plus a no-map reservation at 0x9d400000 and a simple-framebuffer (8 MDSS clocks, MDSS_GDSC) in its dtb; 17 checks ok, rebuild identical

Question
: Can the splash handover be fixed in the dts, as decision 2026-11-19-walleye-display-flip-dts-not-timing says, and built as an image to try at 5 s?

Found
: dt/msm8998-google-walleye-display-splash.dts is the source (values of mainline's OnePlus 5/5T common file and the GPL wahoo dts: 0x9d400000, 0x2400000, 1080x1920 a8r8g8b8). Today's board file does not compile in the display kernel's tree (no fuel-gauge labels), so build/build-display-splash-image.sh makes the same change on B-d5's own dtb, as L4 was made from L3: no line removed, two nodes, three /chosen properties and two symbols added. tests/host/dtb-display-splash.sh fails on B-d5's dtb and passes on the new one; the region overlaps no other reserved one. Kernel, ramdisk (init-display-delay, 5 s), header, cmdline and size are B-d5's; AVB, UFS-off gate, overlay merge, touch and panel gpio claimants pass; rebuilt identical. images-display-splash/boot-b-s5.img sha256 b7c827e9a3f55a7ee6e68752430a19bdad2de3262bc22a5cc0379cd3df80df58. Not tried: whether the faults, rcg warning and DSI -110 vanish at 5 s is the keeper's bisect.

Sources
: kernel-oneplus-common-dtsi

About
: walleye-display-s6e3fa5


<a id="walleye-display-splash-try-signs-stay"></a>
### walleye-display-splash-try-signs-stay

Date
: 2026-11-20

Brief
: B-s5 (splash reserved, simple-framebuffer) at 5 s: all three signs stay, same as B-d5: SMMU faults 10, rcg warnings 4, DSI -110 5; 13 of 13 modules ok

Question
: With the splash region reserved and a simple-framebuffer in the dtb, do the SMMU faults, the pclk0 rcg warning and the DSI -110 vanish at a 5 s load?

Found
: One unattended RAM boot of B-s5 (nothing flashed) by procedures/display-delay-bisect-on-the-pixel-2.rs with delays=5, image_prefix=b-s (device.toml put back after): sha, UFS-off and battery checks ok, 11 steps passed, phone back on Android slot b. Against B-d5 (SMMU 10, rcg 4, -110 5) and B-d25 (all 0): SMMU faults did NOT vanish (10 level-2 context faults, cb 0, iova 0x9d4e1400..0x9d56c000, inside the now-reserved 0x9d400000 region, at 6.19 s); pclk0/byte0 rcg warnings did NOT vanish (4, at 6.22-6.24 s); DSI -110 did NOT vanish (EXIT_SLEEP_MODE and Failed to initialize panel at 6.48 s, the first 0x51 backlog write -110 at 6.69 s, drm_panel_enable failed to enable backlight); fb0 msmdrmfb still registers at 8.57 s. So reserving the region and describing it as simple-framebuffer does not stop the scanout from faulting through msm's new SMMU domain nor the bootloader's DSI clocks from being re-programmed; a simple-framebuffer alone does not hand the splash over. No simple-framebuffer line in the kernel log (grep simple-fb/simplefb: none): whether it bound is not known yet. Table and dmesg kept in ~/lab-material/walleye/display-delay-b-s-20261120-011649/. Next: read whether simple-framebuffer probed (its clocks/GDSC held), then the handover in the driver (disable scanout / stop DSI clocks before msm attaches) or load after 25 s.

Sources
: walleye-display-b2-dsi-clock-warning

About
: walleye-display-s6e3fa5


<a id="walleye-display-handover-how-others-found"></a>
### walleye-display-handover-how-others-found

Date
: 2026-11-20

Brief
: The faults are msm's new SMMU domain meeting a live scan-out; no board hands over cleanly (OnePlus 5, Pixel 3 skip msm); cheapest try: reset MDSS_BCR at mdss probe

Question
: Why did the reserved splash not stop the SMMU faults at 6 s, and how do other msm platforms hand the bootloader's scan-out over?

Found
: (1) Our mmss_smmu (qcom,msm8998-smmu-v2) takes qcom_smmu_cfg_probe: the bootloader's valid SMRs are kept as BYPASS across the SMMU reset. That holds only until a device attaches a translating domain. 'preserved 0 boot mappings' in our log is the ACPI IORT RMR path and means nothing on DT. (2) msm_iommu_new does iommu_paging_domain_alloc + iommu_attach_device: the S2CR of mdss SID 0 moves to a new empty context bank, and the scan-out, still reading physical 0x9d4xxxxx, faults (SID 0x0, cb 0); first fault 5 ms after dpu_kms_hw_init. iommu-addresses direct mappings are applied by the IOMMU core to default domains only (iommu.c ~671, 3186, 3221), so msm's own domain does not get them: the walleye coordinator's lead (2) is right as far as it goes, but its mapping would not reach msm. iommu.passthrough=1 does not help either (msm allocates its own paging domain). No mainline commit found that stops the scan-out before the attach (not proof none exists). (3) B-s5's simple-framebuffer cannot have bound: no SIMPLEDRM, SYSFB_SIMPLEFB or FB_SIMPLE in arm64 defconfig or our fragments; its clocks were never held. The cmdline has clk_ignore_unused ('Not disabling unused clocks' in the log), so clk_disable_unused is not what stops the scan-out near 25 s; the earlier read (no kernel event 12-25 s) points at the bootloader side; genpd and regulator late-off timers are also ruled out for the 16-26 s window (regulators at 30 s; genpd at init, and pd is held by clk/pd state). Not known yet: what stops it. (4) Others: OnePlus 5/5T reserve cont_splash_mem and describe a simplefb but never enable mdss; Pixel 3 crosshatch disables mdss and dispcc 'so simple-framebuffer continues working'; OnePlus 6 enables mdss beside simplefb (fault-free not verified); Fxtec Pro1 reserves only. No board found loads msm over a live splash and says it is clean. The 2019 Andersson/Hugo RFC saw exactly our failure and was not merged. (5) The DSI -110 and pclk0 rcg warnings (DSI/PLL re-programmed under a live bootloader state) are what break the panel, more than the faults, which Hugo calls non-fatal. (6) msm_mdss_reset (assert, 20 ms, deassert) exists but the msm8998 mdss node has no resets property; MDSS_BCR (mmcc 0x2300) is registered and mdss-common.yaml has resets: a dts-only line would make mdss probe reset the whole MDSS block (scan-out, DSI host, pingpong) before msm attaches. Ranked hypotheses: A the bootloader's scan-out and DSI state are live and only a reset or quiesce before attach helps (try: resets = <&mmcc MDSS_BCR>); B the stop at 25 s is something outside the kernel that a read of the DPU registers at 5 s and 30 s will show; C pre-map the region in msm's domain (silences faults, leaves DSI -110: a part, not the way); D disable mdss until late like crosshatch (works, is the delay in the dts); E identity-attach msm (loses isolation, upstream-unfriendly). Material: researcher's survey, kept as this row only.

Sources
: lkml-arm-smmu-qcom-read-back-stream-mappings,lkml-arm-smmu-handoff-smr-rfc,mainline-dts-oneplus5-pixel3-splash,v73-arm-smmu-qcom-msm-iommu-source,walleye-display-splash-try-signs-stay

About
: walleye-display-s6e3fa5


<a id="walleye-pon-reasons-after-a-bite-how-to-read"></a>
### walleye-pon-reasons-after-a-bite-how-to-read

Date
: 2026-11-20

Brief
: After a bite the PON reasons can be read only where the phone first answers; a power press is a power-on that rewrites PON_REASON1, so POFF/WARM_RESET are the ones that can outlive it (not measured)

Question
: How can the PON registers be read after a PMIC watchdog bite on this phone, before anything resets them: the bootloader's charger screen has no adb, and Android's dmesg after the power press is already a second boot?

Found
: READ in this session: the driver's own constants (drivers/watchdog/pm8916_wdt.c): POFF_REASON1 is 0x0c (first generation) and 0xc5 (second generation, PM8998), PMIC_WD is its bit 2; the S2 type is the low nibble of 0x56, S2 enable is bit 7 of 0x57; the ramdisk of image A already reads those through the regmap debugfs (pon-regs). RECALLED from the downstream qpnp-power-on driver, NOT re-read: PON_REASON1 at 0x08, WARM_RESET_REASON1 at 0x0a; the offset of PON_ON_REASON on PM8998 is NOT confirmed, so the procedure reads 0x08..0x0d and 0xc0..0xc7 raw and names only 0x08, 0x0a, 0x0c and 0xc5. WHAT READS WHERE: (1) Android by itself (a warm bite may bring it back without a hand): adb gives ro.boot.bootreason and, if adb may read the kernel log, the PMIC's own reason lines; raw registers need root. (2) The bootloader by itself (fastboot lists the phone): our image booted again from memory reads the reason blocks; fastboot boot is a jump, not a PMIC reset (recalled, not measured). (3) The charger screen or an off phone: nothing answers (no adb, no fastboot), so a reading needs a hand: a power press or power+volume down; either is a power-on from off, which rewrites PON_REASON1 with the key: so for a cable-in hard bite PON_REASON1 cannot show the bite, and only the registers that outlive a power-on (POFF_REASON1 and WARM_RESET_REASON1, recalled to persist until the next power-off or warm reset, NOT measured) can. NOT KNOWN YET: which of these holds on this phone, and the PON_ON_REASON offset: the procedure saves every register raw with the path that read it (Android, our image), so the four bites decide it. DECIDED FROM THIS: the lab reads at the first place the phone answers without a hand; it asks the author only for the physical acts of the paths above, and never before it has said what it could see by itself.

Sources
: qcom-pmic-pon-reset-status,vendor-qpnp-power-on-c,mainline-pm8916-wdt-c,android-bootstat-reasons

About
: walleye-pmic-wdt-v1-image


<a id="walleye-install-stage-how-it-stands"></a>
### walleye-install-stage-how-it-stands

Date
: 2026-11-20

Brief
: Install stage: two stages (rootfs on the unused slot's system partition written from Android, kernel still from RAM; then boot flashed); ~8 build h + ~8 keeper h; no mainline UFS write needed

Question
: How far is Linux installed on the Pixel 2 and booting from its own storage, what does each step cost, what is its way back, and what must the author say yes to?

Found
: WHAT IS (measured, from the lab's own files, nothing written to the phone). UFS LUN layout read back 2026-11-19 (~/lab-material/walleye/bootchain-2026-11-19-2): sda (59.4 GiB) holds BOTH slots' abl, tz, hyp, boot (32 MiB each, sda9 boot_a, sda30 boot_b), dtbo (8 MiB), vbmeta (64 KiB), system (2.5 GiB: system_a sda22, system_b sda43), vendor (500 MiB) and userdata sda45 (53.2 GiB, shared by both slots); xbl_a/b live in the boot LUNs sdb/sdc (4 MiB); persist, misc, frp, modemst in sdd; fsg in sdf; metadata in sde. The boot-chain read-back (32 partitions, abl, xbl, tz, hyp ... ddr, cdt, sha256, twice identical, mode 600, kept off git) is the restore source for those; it does NOT include boot, dtbo, vbmeta, misc, system, vendor of either slot: they are not backed up yet (the control boot.img and dtbo.img sit in ~/Projects/walleye-build/control). Bootloader (getvar all 2026-11-17): unlocked, avb 1.0.0 user-settable key, both slots successful, no fastbootd, max-download 512 MiB; it picks dtbo entry 3 (id 0x1bc rev 3) itself and merges it onto our dtb; the lane's logs say Android runs from slot b, the brief says boot_b with slot a Android: which slot Android sits on must be read (fastboot getvar current-slot) before any write; the rule is: write the slot Android is NOT on. UFS: the matched image read the card (version 3.1.0, six LUNs, partition tables, no errors); the link idles at HS gear 1, 2 lanes; nothing has been written to the card by us. Standing decisions: Android is throw-away, the one hard line is bricking (2026-11-18-walleye-android-disposable-no-bricking: a write to one slot's boot, dtbo or vbmeta with a read-back backup and the other slot intact is the lane's call); no known way back from an erased bootloader (2026-11-18-walleye-edl-no-way-back-wait-holds); the UFS go is the author's.
HOW OTHERS INSTALL. (1) postmarketOS on Pixel A/B phones (crosshatch MR 83): one slot only is flashed; pmbootstrap flasher flash_rootfs to the system partition, or --partition userdata when system is too small, then flash_kernel (boot) and fastboot reboot. (2) The Nura/pmOS wiki for the Pixel 2 XL (taimen, same MSM8998 and UFS family): mainline boots with basic dts and USB network gives a shell; clk_ignore_unused needed; its device package was never added, so no one ships a Pixel 2/XL install; its partition map has our shape (boot_a/b, system_a/b, vendor_a/b, userdata, dtbo, vbmeta). (3) Mobian/Debian on OnePlus 6 (sdm845): fastboot flash boot, fastboot erase userdata, flash userdata (sparse, -S 100M), fastboot erase dtbo, reboot: rootfs on userdata, dtbo erased so the bootloader takes our dtb. (4) OnePlus 5 (msm8998, nearest sibling): a ramdisk mounts a loopback rootfs.img from the userdata partition root; pmaports MR 3531 carries it installed to internal storage on mainline 6.0. (5) AVB: an unlocked bootloader boots an image whose hash does not match vbmeta (orange state, a warning screen); the usual extra step on Pixels is fastboot flash --disable-verity --disable-verification vbmeta, for system/vendor verity, which our kernel does not use. Nobody has published a Pixel 2 (walleye/taimen) rootfs on its own UFS with a mainline kernel; the sibling OnePlus 5 has.
KEY FINDING. Writing the rootfs does not need mainline to write UFS. Android already writes UFS (its own proven stack). So the install splits in stages that each have a way back: STAGE 1 'rootfs on its own storage': while Android runs from slot X, dd a rootfs image onto the OTHER slot's system partition (2.5 GiB, adb root, after reading it back with sha256); boot our kernel from RAM with fastboot boot as now, UFS on, initramfs mounts that partition read-only with a tmpfs overlay: no flash, no set_active, no bootloader-visible change, no mainline UFS write; the phone returns to Android by itself (reboot). That reaches docs step rootfs-boots (root on the storage, ssh over usb-network is the proof). STAGE 2 'boots by itself': read back boot_Y, dtbo_Y, vbmeta_Y, then fastboot flash boot_Y (our image, stock dtbo), set_active Y; boot counting (retry-count) and the power key fall back to the other slot. AVB and the slot rollback are unmeasured for this phone (not known yet): the first flash of boot_Y is the cheap measurement, its way back is the read-back file and the other slot. STAGE 3 mainline UFS WRITE (ext4 rw, a persistent system) is later and the only stage with the Sony-class hazard (purge/erase/unmap): mount nodiscard, no fstrim, the purge guard, and prove after every write run that the bootloader files read back with the same sha256 as the stored set (bootchain-readback-check, twice on the same bytes).
WHERE THE ROOTFS GOES. system_<other> 2.5 GiB (the first image: fits Alpine, pmOS-style, or minimal Debian with systemd and ssh, 0.5-0.8 GiB; a phone-UI image does not, Mobian-size images need userdata). userdata 53 GiB is shared by both slots, so using it ends Android's data and Android boots to setup (disposable by decision; way back = fastboot erase userdata plus Google's public factory image): keep for stage 3. A new partition or a GPT edit is rejected: the bootloader reads the GPT, mainline would write it, no way back known. A loopback rootfs.img in userdata (OnePlus 5 way) is an option later, not now.
THE DISTRO. Postponed by the author (Debian or Nura); the earlier decision rows were not found in this base by search. Costs HERE: Debian minimal: no tooling on this lab machine today (debootstrap, mmdebstrap, qemu-user-static, pmbootstrap are not installed; mkfs.ext4, img2simg, avbtool, fastboot are), so installing them is a yes; we own the initramfs and init (lpoco-bringup generic init, usb gadget already working), so ~3 agent-h to a bootable minimal image with ssh; systemd, glibc; ~0.5 GiB; nothing proprietary in it; firmware, modem and TAS2557 data stay out of git (read from the phone into ~/lab-material, as decided). Nura (pmOS, Alpine, OpenRC, musl): pmbootstrap builds the rootfs and its own initramfs but needs a device package (deviceinfo, our kernel as a package, the dtb, boot.img rules) nobody wrote for walleye or taimen: ~6-8 agent-h and pmbootstrap (chroots, qemu) installed; it brings phone-UI images, upstream packaging and a path to contribute back. Either way the rootfs sits in its own partition and is swappable: the first image does not bind the final distro. Recommendation: Debian minimal first (fewest new moving parts to reach ssh); the author picks the final distro before a phone-UI image.
THE WAY BACK. Stage 1: nothing to undo (the Android slot untouched, reboot returns); restore system_<other> by dd of the read-back file with its sha256 checked. Stage 2: fastboot flash boot_Y from the read-back file, set_active back; retry counter and power key fall back. Not undoable: the bootloader and xbl/abl partitions (never written by us), an erase of either, a purge: no EDL way exists (research 2026-11-18-walleye-edl-way-back); those steps are in no queue.
HONEST DISTANCE. Today Linux is not installed: every boot is from RAM, nothing on the card is ours. Stage 1 = rootfs-boots: UFS-root kernel+initramfs ~4 agent-h, Debian rootfs ~3 agent-h (+tooling yes), read-back ~1 keeper-h, write ~1 keeper-h, 2-3 boot tries ~3 keeper-h: about 7-8 build hours and 5 keeper hours, 2-3 working days of the lane if the display work does not take the keeper. Stage 2 (boots by itself): +3 keeper hours, +1 day. Stage 3 (writable system, gear 3): UFS try 2 (gear, OPP table) is already queued, a write test ~4 keeper-h; ~2 more days. Not known yet: avb behaviour on flash, whether UFS at gear 1 stays stable under load (the gear try is separate), which slot Android sits on, what Android does when userdata is gone. The display does not gate rootfs-boots (ssh over usb is the proof); display-reset-image only gives the installed system a screen.

Sources
: nura-taimen-wiki,src-pmaports-crosshatch-mr83,src-pmbootstrap-install-guide,src-debian-wiki-oneplus6,src-oneplus5-mainline-jami,src-avb-readme,src-pmaports-msm8998-6-0

About
: walleye-display-s6e3fa5


<a id="walleye-install-stock-fstab-wahoo"></a>
### walleye-install-stock-fstab-wahoo

Date
: 2026-11-20

Brief
: Google's wahoo tree (android11-release): stock system is slot-selected with avb, UFS host 1da4000.ufshc, recovery.wipe wipes system/vendor/userdata then boot; supports stage 1; Elixir repartitions

Question
: What do Google's own fstab.hardware and recovery files say about the stock partitions, mounts and A/B layout, for the install stage?

Found
: Read 2026-11-20 from android.googlesource.com device/google/wahoo, branch android11-release (commit 5361539c, 2020-09-16; which Android build the phone runs is not known yet, so the match to the phone's build is not checked). Files that exist there: fstab.hardware, fstab.postinstall, init.recovery.hardware.rc, recovery.wipe, init.hardware.rc, init.insmod.sh and others; no proprietary-files lists.
fstab.hardware: (1) system mounts at / ext4 ro,barrier=1 with fs_mgr flags wait,slotselect,avb: the stock root is the SLOT-SELECTED system partition, checked by AVB, so stage 1's use of system_<other> follows the stock A/B scheme; our rootfs there needs no verity because our initramfs mounts it, not Android's fs_mgr. (2) metadata ext4 noatime,nosuid,nodev,discard,sync formattable. (3) userdata ext4 errors=panic,noatime,nosuid,nodev,barrier=1,noauto_da_alloc, latemount,wait,check,formattable, fileencryption=ice:aes-256-heh (UFS inline crypto), eraseblk=16777216, logicalblk=4096, quota, reservedsize=128M, checkpoint=block: userdata is encrypted by the UFS inline-crypto engine; our image leaves ICE out (decision 2026-11-19-walleye-ufs-match-no-ice-no-linkstartup), so old userdata is unreadable to us and needs no preserving, and the formattable flag means Android re-formats it itself. (4) misc is emmc type, no slot. (5) modem mounts at /firmware vfat ro, slotselect: the modem image is per slot, as our read of modem_b found. (6) The UFS host is soc/1da4000.ufshc, by-name under it: the controller address our dts uses. (7) zram swap 2 GiB, 8 streams. No vendor line: where Android 11 mounts vendor from is not known yet. boot, dtbo and vbmeta are not in the fstab (the bootloader reads them).
init.recovery.hardware.rc: only waits for /dev/block/platform/soc/${ro.boot.bootdevice}, symlinks it as /dev/block/bootdevice and sets sys.usb.configfs 1: nothing about slots.
recovery.wipe: Google's own recovery wipes, in order, system_a, system_b, vendor_a, vendor_b, userdata, then boot_a and boot_b last so that all partitions are wiped correctly even if interrupted: Google itself treats these slot partitions as wipeable, boot last, and the way back is Google's public factory image. This supports stage 1 writing system_<other> and stage 2 writing boot_<slot>, and gives the order to keep: never leave a slot with a new boot and no system.
ELIXIR (ProjectElixir-Devices/device_google_walleye, branch udc, 2024; relayed by the walleye coordinator, its page not read by me): its fstab.hardware and gpt-utils are its own repartitioning; the commit says walleye and taimen require repartitioning and defines product as A/B. That is not the stock layout and not our plan: the stock layout is Google's tree plus our own read-back (walleye-bootchain-readback). REPARTITIONING (a new GPT, a resized userdata or system) is a bigger step than walleye-install-stage-how-it-stands: the bootloader reads the GPT, no way back from a bad table is known, so it needs the author's word, and only then. Its proprietary-firmware.txt lists are for the later firmware step (item walleye-device-config-sources-read). Consequence: the plan is unchanged; stages 1 and 2 stand; a loopback rootfs.img in userdata stays the way to more room without a GPT edit.

Sources
: src-google-device-wahoo,src-projectelixir-device-google-walleye

About
: walleye-display-s6e3fa5


<a id="walleye-display-reset-image-b-r5-tried-signs-vanish"></a>
### walleye-display-reset-image-b-r5-tried-signs-vanish

Date
: 2026-11-20

Brief
: B-r5 (mdss resets = <&mmcc MDSS_BCR>) at 5 s: SMMU faults 0, rcg warnings 0, DSI -110 0; 13 of 13 modules ok; phone back on Android

Question
: Does resetting the MDSS block at mdss probe (B-d5 plus resets) hand the live splash over, so B-d5's three signs (faults 10, rcg 4, DSI -110 5 at 5 s) vanish?

Found
: One unattended RAM try from fastboot on slot b, nothing flashed, procedures/display-delay-bisect-on-the-pixel-2.rs with delays=5 image_prefix=b-r (device.toml put back): reached 5 s, smmu_faults 0, rcg_warnings 0, dsi_110 0, modules_ok 13, verdict clean; phone back on Android (phone-state: android). Host half passed first (tests/host/dtb-display-reset.sh). Table in ~/lab-material/walleye/display-delay-b-r-20261120-230943/table.txt. Image sha256 6dc0239e...; declared as [images.b-r5] in device.toml (the builder had not). Not measured: B-r5 at 2 s (display at boot, any order), and the glass was not looked at. Decision 2026-11-20-walleye-display-handover-reset-first stands: the reset is the handover. Next: the dts for any load order (walleye-display-dts-any-order) can carry the same resets property; the author's look at B2 is walleye-display-look-b2.

About
: walleye-display-s6e3fa5


<a id="walleye-alpine-rootfs-rootless-look-around"></a>
### walleye-alpine-rootfs-rootless-look-around

Date
: 2026-11-20

Brief
: Alpine 3.24 aarch64 rootfs builds rootless (apk.static --usermode --no-scripts, 56 pkgs, 24 MiB); gaps: no pd-mapper package, no aarch64 Rust target for lpoco-health

Question
: Can plain Alpine minimal be built on the lab machine without installing anything, and are all the packages the item names available?

Found
: apk-tools-static 3.0.9 (x86_64, from the Alpine mirror, kept under AGENT_BUILD) installs alpine-base, openssh, agetty, qrtr, rmtfs(+openrc), tqftpserv, e2fsprogs into a directory as the lab user with --usermode --initdb --no-scripts: 56 packages, 22.5 MiB, no root, no qemu. mke2fs -d (host) can make the ext4 image from that directory. Missing: pd-mapper is in no Alpine 3.24 repo (main, community; edge testing also has none) - it is postmarketOS's, build from source (aarch64-linux-gnu-gcc exists, link static); lpoco-health is ours and the host has only the x86_64 Rust std, so an aarch64 build needs a rust-std target installed (a root pacman command - for the author's hands, not done). Scripts skipped without qemu: OpenRC runlevel links, inittab getty lines, the usb gadget unit are written by the build as files.

About
: walleye-display-s6e3fa5


<a id="walleye-install-slot-readback-done-twice-same-bytes"></a>
### walleye-install-slot-readback-done-twice-same-bytes

Date
: 2026-11-20

Brief
: Both slots' boot, dtbo, vbmeta, misc, system, vendor read back over adb root twice: checked at measured sizes, identical sha256; slot b active

Question
: Can the way back of the install (both slots' partitions) be read to files, reproducibly?

Found
: Read 11 partitions (~6.3 GiB) twice via bootchain-readback.sh with LIST=slot-parts.list (reader now honours LIST), ~12 min each, to ~/lab-material/walleye/slot-readback-2026-11-20-1 and -2 (mode 600/700, outside git). slot-readback-check.sh DIR DIR2 passes: measured sizes, sha256 match, both reads the same bytes. Current slot (adb getprop ro.boot.slot_suffix) _b; fastboot getvar not read, to avoid a reboot of a healthy phone. Nothing written to the phone; adbd left as root (resets on reboot).

About
: walleye-display-s6e3fa5


<a id="walleye-alpine-rootfs-v1-built"></a>
### walleye-alpine-rootfs-v1-built

Date
: 2026-11-20

Brief
: Alpine 3.24 aarch64 rootfs v1 built rootless, reproducible, reviewed: 1 GiB sparse ext4, 57 packages, 43 checks ok, sha256 89f543eb; no ssh key yet, no pd-mapper, no lpoco-health

Question
: Does tools/walleye-rootfs build the item's v1 image (login on tty0 and ttyGS0, sshd, usb gadget net, qrtr/rmtfs/tqftpserv installed not started, no phone files, no UI) twice with the same sha256?

Found
: tools/walleye-rootfs --build now: apk.static 3.0.9 --usermode --no-scripts, Alpine keys from alpine-keys, the skipped scripts' work written by the tool (busybox and busybox-extras applet links, OpenRC links from rootfs/runlevels, inittab, the usb-gadget unit = init-z14's gadget with ECM 172.16.43.1 + udhcpd lease .2, root empty password on consoles), a tar under fakeroot, mke2fs -d tarball. Two builds, same sha256 89f543ebd4642c135af3ddcba25f37e1372ef505bb351afd598373fb83b45b54 (a directory source was not reproducible: mke2fs copies ctime). 1 GiB sparse, e2fsck clean, no file under lib/firmware, vendor, modem, no UI package. Programmer review fixed: sshd needs usb-gadget not net (conf.d/sshd), swclock+seedrng in boot, gadget unit checks usb0 and supervises udhcpd, openrc default is ::once, the image is checked before it gets its name and .sha256, an existing .sha256 is compared. NOT fixed (open, in order): ssh key required/validated; apk keys and apk.static pinned by hash, one fetch for both builds; apk db modes/owners (bbsuid is 0111, not 4111) applied; dangling-link and executable checks; started-by-nothing scan of conf.d/inittab; all four 172.16.43.1 places checked against device.toml; host-tool preflight; check logic moved into lib.rs as pure functions; getty on ttyGS0 respawns until the gadget binds. Never booted: the services' start is read from OpenRC's dependency output by the reviewer, not run on aarch64. Found: Alpine's busybox has no udhcpd (busybox-extras added); no ssh key on the lab machine: item walleye-install-rootfs-lab-key. Image ~/Projects/walleye-build/images-rootfs/walleye-alpine.ext4 (+ .sha256, packages.txt). Packages: agetty-2.42.3-r1 agetty-openrc-0.63.2-r0 alpine-baselayout-3.7.2-r1 alpine-baselayout-data-3.7.2-r1 alpine-base-3.24.4-r0 alpine-conf-3.22.0-r0 alpine-keys-2.6-r0 alpine-release-3.24.4-r0 apk-tools-3.0.9-r0 bridge-1.5-r5 busybox-1.37.0-r31 busybox-binsh-1.37.0-r31 busybox-extras-1.37.0-r31 busybox-mdev-openrc-1.37.0-r31 busybox-openrc-1.37.0-r31 busybox-suid-1.37.0-r31 ca-certificates-bundle-20261020-r0 e2fsprogs-1.47.4-r0 e2fsprogs-libs-1.47.4-r0 eudev-libs-3.2.14-r6 ifupdown-ng-0.13.0-r0 libapk-3.0.9-r0 libblkid-2.42.3-r1 libcap2-2.78-r0 libcom_err-1.47.4-r0 libcrypto3-3.5.9-r1 libeconf-0.8.3-r0 libedit-20260508.3.1-r1 libncursesw-6.6_p20260516-r0 libssl3-3.5.9-r1 libuuid-2.42.3-r1 mdev-conf-4.10-r0 musl-1.2.6-r3 musl-utils-1.2.6-r3 ncurses-terminfo-base-6.6_p20260516-r0 openrc-0.63.2-r0 openrc-user-0.63.2-r0 openssh-10.3_p1-r2 openssh-client-common-10.3_p1-r2 openssh-client-default-10.3_p1-r2 openssh-keygen-10.3_p1-r2 openssh-server-10.3_p1-r2 openssh-server-common-10.3_p1-r2 openssh-server-common-openrc-10.3_p1-r2 openssh-sftp-server-10.3_p1-r2 qrtr-1.2-r1 qrtr-libs-1.2-r1 rmtfs-1.3-r0 rmtfs-openrc-1.3-r0 scanelf-1.3.9-r1 skalibs-libs-2.15.0.0-r0 ssl_client-1.37.0-r31 tqftpserv-1.1.1-r0 tqftpserv-openrc-1.1.1-r0 utmps-libs-0.1.3.3-r0 zlib-1.4-r0 zstd-libs-1.5.7-r2 

About
: walleye-display-s6e3fa5


<a id="walleye-install-current-slot-read-b-both"></a>
### walleye-install-current-slot-read-b-both

Date
: 2026-11-20

Brief
: Android runs from slot b (adb ro.boot.slot_suffix _b = fastboot current-slot b); the other slot is a, the one stage 1 writes system_a of

Question
: Which slot does Android run from, and which is the other: adb getprop ro.boot.slot_suffix and fastboot getvar current-slot, equal?

Found
: adb getprop ro.boot.slot_suffix read _b; at a fastboot visit (adb reboot bootloader, lpoco lab fastboot getvar current-slot, reboot now) current-slot read b. The two readings are equal: Android runs from slot b, the other slot is a (stage 1 writes system_a). Nothing written; phone back on Android, slot_suffix _b after the reboot. The slot-readback directories ~/lab-material/walleye/slot-readback-2026-11-20-{1,2} each hold both slots' images named by slot (boot_a/boot_b, system_a/system_b ...), so names match slots. The lane logs (slot b) were right; the Top Coordinator's brief (boot_b, Android slot a) is the one that disagreed.

About
: walleye-display-s6e3fa5


<a id="walleye-alpine-rootfs-v1-hardened"></a>
### walleye-alpine-rootfs-v1-hardened

Date
: 2026-11-20

Brief
: Alpine v1 hardened: pinned apk.static and keys, apk dir/file modes, link/exec/start/address checks; image sha256 57cf5fd1; rootfs-boots needs ssh from the lab, not lpoco-health

Question
: Does the hardened tools/walleye-rootfs rebuild the v1 image twice with one sha256, and does step rootfs-boots need lpoco-health on the phone?

Found
: tools/walleye-rootfs, hardened (all in Rust, argenv). Image built twice, one new sha256 57cf5fd1425787eaa3399484d449e544057cac862308881069581b232fc89939 (v1 first build was 89f543eb; the change is the apk directory modes now applied), declared in device.toml [images.alpine-rootfs] at images-rootfs/walleye-alpine-v1.ext4; 54+ checks ok.
(1) rootfs/pins names apk-tools-static 3.0.9-r0 and alpine-keys 2.6-r0 with their sha256; the build downloads those exact versions once, hashes before unpacking, reuses a cached download only if its hash matches; no APKINDEX trust. (2) apk's installed db applied through fakeroot: files from `a:`, directories from `M:` (the review caught that real apk writes directory modes as M:, e.g. var/empty 555, tmp 1777, root 700); 241 paths; bin/bbsuid 4111 asserted by name from debugfs stat. (3) dangling-symlink check (399 links followed through linked dirs; run-time dirs proc sys run dev tmp allowed) and execute-bit check over bin, sbin, init.d. (4) inittab programs, conf.d needs and init.d command= of the 16 runlevel services must be in the image; start() bodies are NOT read (not known yet). (5) the 172.16.43.x places: usb-gadget and sshd 10-lab.conf must carry device.toml's address, udhcpd.conf its /24; comments are not read; device.toml shell_address = addresses[0]. (6) host-tool preflight names every missing tool. (7) the logic is pure functions in lib.rs (21 tests). (8) inittab must have a respawn getty on tty0 and ttyGS0, so ttyGS0 is retried until the gadget binds (read from the inittab; the retry rate of busybox init when /dev/ttyGS0 is absent is not measured yet).
OPEN QUESTION answered: step rootfs-boots (done_when: a distribution's root boots to its init and ssh answers; proof rootfs-ssh, a host-check) needs no lpoco-health on the phone: the lab runs it over ssh from the host. The phone's own suite is the next step health-suite-runs (proof suite-filed) and waits for item walleye-install-rootfs-health-binary.
The empty root password on tty0 and ttyGS0 is the test layer's, like the earlier test images; ttyGS0 is reachable by anyone with the USB cable. sshd takes a key only and none is in the image yet (waits for the author's yes). Not claimed: the image was never booted. Reproducibility: package versions float with the mirror (names only in rootfs/packages), so the same pins rebuilt later may give another sha; the host tools were mke2fs 1.47.4, fakeroot 1.37.2.

About
: walleye-display-s6e3fa5


<a id="walleye-ufs-root-image-built"></a>
### walleye-ufs-root-image-built

Date
: 2026-11-20

Brief
: UFS-root image built: matched UFS kernel code and dtb, overlayfs built in, our own initramfs; checks pass, rebuild identical, gate refuses it by design

Question
: Can a boot image mount the Alpine system_a read-only with a tmpfs overlay, on what exists, proven without the phone?

Found
: Image sha256 35b60fdaf0764e97e0b6de60cddc9a955a14928ba3577f7f110206387f966474 (~/Projects/walleye-build/NOT-TO-BOOT-ufs-on-images-ufs-root, 32 MiB, AVB footer verifies, header and dtb the matched image's, command line plus walleye.root=system_a, built twice identical). Kernel tree 3644090c7f6d with CONFIG_OVERLAY_FS=y (defconfig =m). tests/host/ufs-root-init.sh (QEMU, loop-made GPT disk, writable disk, sha256 compared after): root is an overlay; card partition ro, norecovery, kernel refuses writes (blockdev --setro); petter outside /run holds /dev/watchdog, survives a tmpfs on /run, the phone resets after the hold; refusals for no partition, running slot, foreign name, no name, dangling /sbin/init. Review (programmer) found: petter under /run (OpenRC remounts it), a test that could not fail (read-only QEMU disk), no kernel-side ro, a symlinked /sbin/init checked in the initramfs, unchecked numeric settings: all fixed. Open: the guard on androidboot.slot_suffix fails open when the bootloader gives none on fastboot boot (read-only mount only); only the real card shows PARTNAME and probe time: the boot try.

About
: walleye-display-s6e3fa5


<a id="walleye-ufs-try-2-built-not-booted"></a>
### walleye-ufs-try-2-built-not-booted

Date
: 2026-11-21

Brief
: UFS try 2 built as one command on lpoco-attended 0.41.0; host-side proofs pass; the UFS-on boot waits for the author's go

Question
: Is UFS try 2 one launch away, tested everywhere but the phone?

Found
: procedures/ufs-matched-image-on-the-pixel-2.rs (23 steps, 51 lines) on lpoco-attended-v0.41.0; walleye-tries: 152 unit tests incl. stand-in phone walkthroughs of the load, GPT and slot steps; clippy -D warnings, fmt clean; --dry-run passes all 23 steps; lpoco-attended check --host ok (devlab, lpoco-lab 0.41.0). NOT run on the phone, NOT booted. Author's go text: 'One command, about 5 minutes: boots the matched UFS-on image once from memory; reads the card (six disks, partition tables); reads sda read-only for at most 4 GiB or 60 s while the link gear and clock are noted; sda's partition table hashed before and after; NOTHING is written to the card; glass stays dark; the phone returns to Android by itself and visits fastboot once more to read the slot flags; if not on Android after 4 minutes, hold power ~15 s. Go covers only the image sha256 the go item names.' Keeper's run of the parts that do not boot UFS-on (dry run, gates, way back from a fake stop) is still to do.

About
: walleye-ufs-match-android-z


<a id="walleye-install-rehearsal-run-passed"></a>
### walleye-install-rehearsal-run-passed

Date
: 2026-11-21

Brief
: The install stage 1 rehearsal ran for real, unattended: 14 of 14 steps pass, slots agree (b running, a target), target equals its backup, nothing written

Question
: Does every step of the install write except the dd work on the real phone, and does the toybox dd proof hold?

Found
: 2026-11-21, procedures/rehearse-install-rootfs-on-the-pixel-2.rs on lpoco-attended 0.41.0, dry run (14 steps) first, then real: battery, backup whole, image is the declared one (sha256 57ea7152...) all pass; one fastboot visit read current-slot, adb root; image staged and its copy hashed; fastboot and adb name the same slot; the target partition read now equals its stored backup (about 5 min of the run); the dd probe file read back equal; staged copy and probe file removed; phone ends on Android slot b (adb ro.boot.slot_suffix _b), /data/local/tmp empty. No partition written. The real write (item walleye-install-write-rootfs) is now one command for the Top Coordinator to bring: procedures/install-rootfs-on-the-pixel-2.rs.

About
: walleye-display-s6e3fa5


<a id="walleye-install-stage2-android-verdict"></a>
### walleye-install-stage2-android-verdict

Date
: 2026-11-22

Brief
: Stage 2 vs Android OTA: flash boot_other plus set_active equals the OTA's own steps in kind, BUT set_active IS a GPT write (attributes, type GUIDs, UFS boot LUN): named difference, coordinator's call

Question
: Does flashing our boot into the slot Android is not on and set_active with boot counting do anything Android's own OTA does not?

Found
: VERDICT: no step of ours is outside what Android's OTA does, with ONE named fact the standing rule must see: fastboot set_active (and the OTA's setActiveBootSlot) is a write to the GPT. (1) OTA: update_engine writes boot_<other> (and the rest of that slot), the HAL then calls setSlotAsUnbootable/setActiveBootSlot; the Qualcomm HAL (hardware/qcom/bootctrl, boot_control.cpp) sets attribute bits in the GPT entries of every A/B partition on every LUN, recomputes the GPT CRCs, and on UFS switches the boot LUN (xbl_a/xbl_b) through gpt_utils_set_xbl_boot_partition; markBootSuccessful sets the successful bit of the running slot, called by Android after a boot completes (update_verifier then the HAL). (2) Fastboot set_active (CAF ABL, CmdSetActive and SwitchPtnSlots, the stand-in for walleye's proprietary ABL): boot_<x> gets active+priority+retry count 7, successful and unbootable cleared; the other slot loses active and priority; SwitchPtnSlots swaps the partition TYPE GUIDS of every _a/_b pair; on UFS it sets the boot LUN; UpdatePartitionAttributes rewrites both GPT headers and entries with CRCs. At boot FindBootableSlot decrements the retry count of an unconfirmed slot and marks it unbootable at zero, then the other slot boots: that is the fall back, and it writes the GPT every boot of a try. (3) So our one command (flash boot_<other>, set_active other) does exactly the OTA's two moves; the way back (set_active <android>) is the same call; the fall back is the bootloader's own. Named difference: the GPT write is done by the bootloader's code, not ours, but it is a GPT write, and the lane's rule says GPT never: the verdict goes to the coordinator, not a go. AVB (item 2): not read from the phone's ABL (proprietary); known from the lane's readings: unlocked, orange state allowed, our boot image carries the AVB footer and passes the lane's AVB check; whether stock vbmeta flags must be disabled is NOT KNOWN yet: read-only check: fastboot getvar slot-successful/unbootable/retry-count for both slots before and after a RAM boot (a RAM boot must change none), and getvar all twice identical. NOT READ: walleye's real ABL (Google build), its HAL build (Lineage wahoo uses qcom-caf bootctrl; stock used a Qualcomm bootctrl of the same family): behaviour measured, not source, is the proof: the ONE cheap check is the slot flags read before and after.

Sources
: src-aosp-qcom-bootctrl,src-caf-abl-partitiontableupdate,src-aosp-ab-bootloader-updating

About
: walleye-display-s6e3fa5


<a id="walleye-rootfs-boots-rehearsal-run-passed"></a>
### walleye-rootfs-boots-rehearsal-run-passed

Date
: 2026-11-22

Brief
: The root boot rehearsal ran for real, unattended: 14 of 14 steps pass; card (6 disks, system_a) and bootchain and slot flags equal before/after; nothing booted or written

Question
: Does every step of the read-only UFS-root boot except the boot itself pass on the real phone?

Found
: procedures/rehearse-rootfs-boots-on-the-pixel-2.rs, 2026-11-22, phone on Android slot b, image names root system_a (sha 35b60fda). Battery, image, stored set and key, adb slot b = fastboot slot, guard, flags read, card GPTs of 6 disks and system_a hash identical before and after, boot chain equals stored set, flags as before. Run takes ~15 min (two hashes of system_a over adb): a first run under a 580 s timeout was killed at step 12 by the keeper, not a failure of the phone. A fresh worktree needs build/images linked to ~/Projects/walleye-build (device.toml says so; git-ignored) or check 2 stops with no file. Phone ended on Android, fastboot not needed.

About
: walleye-display-s6e3fa5


<a id="walleye-install-stage3-android-ufs-comparison-verdict"></a>
### walleye-install-stage3-android-ufs-comparison-verdict

Date
: 2026-11-22

Brief
: VERDICT stage 3: with nodiscard, no fstrim, no mkfs, ufs_bsg unopened and the purge guard, our kernel sends the card no command class Android's did not; named differences are power/speed only

Question
: Does our kernel's write path to the UFS card (SK hynix H28U74301AMR D003) do anything Android's UFS path on this phone does not: UNMAP, purge, erase, quirks, write booster, power?

Found
: VERDICT: our kernel does nothing Android's did not, in command classes. Named differences are power and speed, none a command to the card. Read-only study of Google's wahoo 4.4 (android-msm-wahoo-4.4-android11 @52750b8f, GPL, clean room: facts only) against our v7.3 tree (3644090c7); nothing run on the phone.

(1) COMMAND CLASSES on a plain ext4 rw mount, write and unmount, our options (ro->rw of ONE partition, nodiscard, no fstrim, no mkfs/resize):
- WRITE(10)/READ(10) with FUA and flush: sd layer in both, same; Android's ext4/f2fs on userdata sends them all day.
- SYNCHRONIZE CACHE: from ext4 journal commits (preflush) and sd shutdown; both kernels, same.
- UNMAP: only from the filesystem (mount option discard, fstrim, mkfs, resize). Ours: none of them, the mount is nodiscard (ext4 default is off) and the procedure never runs mkfs or fstrim. Android DID send UNMAP (live counters: one UNMAP in normal operation, research walleye-ufs-android-way), we send none: fewer, not more. WRITE SAME (which could carry UNMAP, used by blkdev zeroout) is switched off by sdev->no_write_same=1 in BOTH ufshcd.c versions.
- PURGE: Android never sets fPurgeEnable (live: flags set fDeviceInit only). Ours: the only setters are the ufs_bsg RPMB purge path and the raw query path; our init never opens ufs_bsg, and both are behind ufshcd_purge_refused (UFS_DEVICE_QUIRK_NO_PURGE on this model, ufshcd.c:3465..3506 and 7692, ufs-qcom.c:1315). Equal in effect, stricter in guard.
- START STOP UNIT: runtime autosuspend of the LUN and system suspend/shutdown; Android sets sdev->autosuspend_delay and runs the same START_STOP to the device WLUN (ufshcd_set_dev_pwr_mode); mainline adds rpm_autosuspend = 1 (cap UFSHCD_CAP_RPM_AUTOSUSPEND). Same command, same card state (SLEEP/ACTIVE), different timing. HARMLESS.
- Background operations: both kernels enable auto-BKOPS only through the UFSHCD_CAP_AUTO_BKOPS_SUSPEND path (bBackgroundOpsEn attribute): same.
- Write booster: our host caps carry WB_EN and WB_WITH_CLK_SCALING, Android's 4.4 has no write-booster code. But ufshcd_wb_probe (ufshcd.c:8551) probes it only for wSpecVersion >= 0x310 or 0x220 or the SUPPORT_EXTENDED_FEATURES quirk; this card is UFS 2.1 (hba->ufs_version 0x210, device spec 0x0210) and has no such quirk (our tables: SAVECONFIGTIME for any Hynix, plus the model entry), so WB is disabled and no WB flag or attribute is ever written. EQUAL by the card's version.
- ERASE / secure erase / DISCARD with secure bit: not in either path; sd only issues it on a filesystem request. None.

(2) QUIRKS and the Sony mechanism: as each kernel applies them, this card gets HOST_PA_SAVECONFIGTIME on both (mainline's core table matches every Hynix; Android's matching entry), and ours adds the NO_PURGE guard. Sony's three quirks (DELAY_BEFORE_LPM, EXTEND_SYNC_LENGTH 0x48, one TX lane) are in neither (decision 2026-11-19-walleye-ufs-no-sync-length-quirk). Sony's mechanism is NOT PUBLISHED (research walleye-ufs-gap-sony-mechanism-public-record): the SODP view (bug 802) ties a loss to fPurgeEnable being set, with the sync-length setting a guess. A Sony-class lockup, as published, needs a purge request; our configuration cannot send one by any path read here (no ufs_bsg open, guard on the query builder, nodiscard, no fstrim, no erase path), and a plain write has no way to set it. Open honestly: the Pixel 2's card firmware D003 may react to something unlisted; Android's own year of writes on the same card with the same link settings is the evidence this is not the case here.

(3) LINK and power: Android reaches HS-G3 2 lanes rate B; mainline negotiates the same as far as read (the read-only try shows it). Android 4.4 also scales gear (ufshcd_scale_gear, ufshcd.c:9651 of the vendor tree) and clocks and has hibern8 and clock gating; ours adds no extra command, only different thresholds (no OPP table in msm8998.dtsi, no bus votes). Whether gear scaling under load behaves is exactly what item walleye-ufs-try-2 measures (power_info and devfreq under a bounded O_DIRECT read). Not known yet: nothing about writes, only speed.

(4) THE ONE TESTED COMMAND (walleye-install-rw-write-test, builder to write, keeper to run): ONE partition (system_a or system_b, the slot Android is not on, named by walleye.root and refused if it is the running slot), blockdev --setrw only that node, mount -t ext4 -o rw,nodiscard,noatime, a bounded write (a few hundred MB through dd conv=fsync and sync), umount, sync, fsck.ext4 -fn clean; the bootchain read-back (bootchain-readback.sh) before and after, bootchain-readback-check.sh must find every sha256 equal; the GPT digests before and after (read by Android and by us). Way back: the partition is rewritten from its verified backup (the install stage 1 dd and read-back); the bootloader files and GPT are never written so they need no way back; if the checks fail the phone is left charging in Android and the Top Coordinator is told. Never: erase, purge, discard, fstrim, mkfs, the GPT, xbl, abl or any bootloader partition.

(5) VERDICT: our kernel does nothing Android's did not, for this step. Named differences (none a storage command): runtime-autosuspend timing, gear-scaling thresholds, no bus votes, ICE absent, PHY table of the sdm845 family; Android sends UNMAP, we do not. NOT KNOWN YET: the Lineage fork's UFS code (the running 4.4.302; Google's stock branch was read, as in the earlier android-way research), the card's own firmware reaction to any command, and discard capability of the LUN as the SCSI layer sees it (not in the saved sysfs). None of these is a command we would send. Stop rule held: no bootloader partition, GPT, erase or purge is needed by the step.

Sources
: wahoo-4-4-ufshcd-c,wahoo-4-4-ufs-qcom-c,src-google-wahoo-msm44-android11,src-mainline-v73-ufshcd,src-sony-bugtracker-802,src-sony-ufs-quirks-forward-port

About
: walleye-display-s6e3fa5


<a id="walleye-rootfs-boots-run-1-slot-suffix-missing"></a>
### walleye-rootfs-boots-run-1-slot-suffix-missing

Date
: 2026-11-22

Brief
: Root boot run 1: UFS-on image booted, usb-network and ssh login ok (checks 13-15); stopped at check 16: bootloader gave the init no androidboot.slot_suffix, guard failed open

Question
: Does the UFS-root image boot from RAM and mount the other slot's system read-only (docs step rootfs-boots)?

Found
: Run 2026-11-22 ~20:00: checks 1-15 passed (1d6b:0104 on bus, ssh as root, kernel 7.3.0-00013-g3644090c7f6d-dirty, network and services up). Check 16 failed: the init saw no androidboot.slot_suffix on the fastboot-from-RAM boot, so its slot guard failed open and the run stopped by itself. Nothing written to the card by design. Phone back on Android slot b by itself. Not rerun, per the item. The image cannot learn the slot from the cmdline; the procedure or init needs another slot witness (e.g. pass androidboot.slot_suffix=_b in the boot cmdline from fastboot, since adb and fastboot agree on the slot).

Sources
: null

About
: walleye-display-s6e3fa5


<a id="walleye-ramroot-b-a5-unattended-run-1"></a>
### walleye-ramroot-b-a5-unattended-run-1

Date
: 2026-11-22

Brief
: B-a5 booted from RAM unattended: checks 1-15 pass (tmpfs root, 13 of 13 display modules at 5 s, three signs zero, fbcon, getty, sshd); stopped at the author's look

Question
: Does the Alpine-in-RAM image B-a5 boot UFS-off and show every readable sign of a login on the glass, before the author looks?

Found
: Run 2026-11-22 by lpoco-attended run ... --until FIRST_PROMPT: battery, sha256, UFS-off, key ok; fastboot boot from memory, usb gadget 1d6b:0104, ssh login as root; kernel 7.3.0-00007-g7430c717a94c-dirty; / is tmpfs, init said handing over to /sbin/init at 6.6 s; modules loaded at 5 s, SMMU faults 0, rcg warnings 0, DSI -110 0, 13 of 13 ok; getty on tty0 and ttyGS0, sshd up; fbcon 135x120 in the kernel log; no purge/oops/panic; deadman held for the look. Phone back on Android (adb device). Not yet known: whether the author sees walleye login: on the glass. Note for fresh worktrees: build/images is git-ignored, link it to ~/Projects/walleye-build.

About
: walleye-display-s6e3fa5


<a id="walleye-ramroot-b-a5-the-look-no-login-prompt"></a>
### walleye-ramroot-b-a5-the-look-no-login-prompt

Date
: 2026-11-22

Brief
: The author's look at B-a5: kernel log and init text on the glass, last lines 'starting sshd', 'starting local', blinking cursor; no 'walleye login:' prompt

Question
: Does Alpine's login prompt show on the Pixel's glass after B-a5 boots from RAM?

Found
: Run 2026-11-22 (log ~/lab-material/walleye/alpine-look-1795374782.log): checks 1-15 passed (ssh, tmpfs root, 13/13 modules, fbcon, no purge/oops), deadman held, the author saw the display up with boot text ending in 'starting sshd', 'starting local' and a blinking cursor, no getty login prompt on tty0. Phone back on Android slot b, nothing flashed. Open: the tty0 getty either is not started by OpenRC's default runlevel on the glass, or its prompt is printed before the later init lines and scrolled; ssh works, so the userland is up. Next: read /etc/inittab and the getty line in the image, and the getty's tty on the ssh side of the next run; not rerun without that fix.

About
: walleye-display-s6e3fa5


<a id="walleye-ramroot-b-a5-getty-tty1-rebuilt"></a>
### walleye-ramroot-b-a5-getty-tty1-rebuilt

Date
: 2026-11-22

Brief
: Rootfs v3 (getty on tty1) and B-a5 rebuilt, reproducible; host proofs pass; the screen check awaits the keeper's run

Question
: Can the lab measure the login prompt on the glass before the author looks, and is the image with tty1 built reproducibly?

Found
: Rootfs v3 sha256 886ef625690b22b1b26a3875753a02aa118e206dbf639dead3c624cfb8bbdfe2 (built twice identical, all checks ok incl. getty on tty1 and ttyGS0); B-a5 rebuilt twice identical sha256 3cb5adb8dfc9c0176e096a0318c113c53a07e00493e1d4b47697f8f5cf68e235 (old image kept as boot-b-a5-tty0.img, 575bbe43...), ramdisk checks all pass, QEMU init proof (x86 stand-in) pass, 212+22+11 unit tests, clippy -D warnings, fmt clean, procedure dry run 22 steps. NOT measured: whether tty1 puts the prompt on the glass - that is the keeper's unattended run's check 15 (needs the phone).

About
: walleye-display-s6e3fa5


<a id="walleye-rootfs-boots-rehearsal-slot-cmdline-run-passed"></a>
### walleye-rootfs-boots-rehearsal-slot-cmdline-run-passed

Date
: 2026-11-22

Brief
: The root boot rehearsal with the slot on the command line ran for real, unattended: 15 of 15 steps pass; run image made, card/bootchain/slot flags equal before/after; nothing booted or written

Question
: Does the rehearsal with the new steps (slot read from adb and fastboot, run image made with androidboot.slot_suffix) pass for real on the phone?

Found
: Image sha256 c3373d71...6c7ba9 (declared at the worktree's main 7fd9b58). Android on slot b; adb and fastboot agree; run image made and checked equal to the declared one in every byte but the command line; 6 disks and system_a hashed equal before and after; boot chain equals stored set; both slots' flags as before. Phone left on Android. Note: a fresh worktree needs the ignored link build/images -> ~/Projects/walleye-build.

About
: walleye-display-s6e3fa5


<a id="walleye-ramroot-b-a5-getty-tty1-prompt-is-mid-screen"></a>
### walleye-ramroot-b-a5-getty-tty1-prompt-is-mid-screen

Date
: 2026-11-22

Brief
: B-a5 run 204417: check 15 passes by substring, but 'walleye login:' sits at screen row 84 with init output after it; the screen ends at 'Starting local', no prompt at the foot

Question
: Is the Alpine login prompt on the glass of the Pixel 2 where the author can use it?

Found
: Keeper's unattended run 2026-11-22 20:44-20:50 of procedures/alpine-login-on-the-glass-on-the-pixel-2.rs on B-a5 3cb5adb8: checks 1-16 pass, so the lab says the prompt is on the screen (/dev/vcs of tty1, 120 rows). But screen.txt (~/lab-material/walleye/alpine-login-20261122-204417) shows 'walleye login:' at row 84 on the same line as the start of ' * Binding the usb gadget ...', after which sshd and local lines print and the rest is blank: the getty wrote its prompt early, init output followed it, and the cursor sits below. That equals the author's look (last lines starting sshd, starting local, blinking cursor). So check 15 is too weak: it should require the prompt on the last non-blank row (the cursor row), not anywhere in the text. This refines item walleye-display-alpine-getty-experiments (the getty is not silent: its prompt is overwritten in order, not missing). Phone back on Android slot b after the run (adb device), nothing written. Worktree note: build/images must link to ~/Projects/walleye-build itself, not its images/ folder.

About
: walleye-display-s6e3fa5


<a id="walleye-rootfs-boots-run-2-init-did-not-hand-over"></a>
### walleye-rootfs-boots-run-2-init-did-not-hand-over

Date
: 2026-11-22

Brief
: Root boot run 2 (slot on cmdline): booted, ssh login ok (checks 13-16); stopped at check 17: init did not hand over to the root's /sbin/init

Question
: Does the UFS-on image with the slot on its command line mount system_a read-only under the overlay and hand over?

Found
: Run 2 on 2026-11-22 (image c3373d71, run image d594dcbe): pre-checks 1-12 pass, boot, usb-network, ssh login as root on kernel 7.3.0-00013-g3644090c7f6d-dirty pass; check 17 failed: the init did not hand over to the root's /sbin/init. The slot fix is in (slot seen by init is the next thing to read). The run stopped by itself and was not rerun; the post-run checks (GPT, boot chain, slot flags equal to before) did not run, so not known yet. Phone is back on Android (adb device). Log: ~/Projects/linux-phone-oco/rootfs-boots-run-2.out. Next: read the init's console/log on the image to see why the switch_root did not happen; coordinator decides.

About
: walleye-display-s6e3fa5

Material
: research/walleye-display-s6e3fa5/2026-11-22/


<a id="walleye-rootfs-boots-run-3-handover-happened-unlogged"></a>
### walleye-rootfs-boots-run-3-handover-happened-unlogged

Date
: 2026-11-23

Brief
: Root boot run 3 stopped at check 17 again, but the evidence shows the hand-over DID happen: its 'handing over' line is lost because say() writes /dev/kmsg after /dev was moved

Question
: Why did check 17 fail again (init did not hand over) on image c3373d71, with the slot on the command line?

Found
: Run 3 (2026-11-23 05:44-06:00, PR 740's evidence read ran): checks 1-16 pass; the init logged start (slot _b seen = adb's) and 'root is /dev/sda22 (PARTNAME=system_a)', mounted it ext4 ro norecovery at /lower. Evidence after the stop: PID 1 is /sbin/init, / is the overlay (lower=/lower, upper tmpfs), /lower is the card ro, /dev /proc /sys moved into the new root, the new root's dirs listed - so switch_root ran. The missing line: build/initramfs-root/init line 94 say 'handing over' comes AFTER mount --move /dev /newroot/dev (line 91); say() writes to /dev/kmsg, which no longer exists in the initramfs's /dev, so the kernel log never gets it and rootfs_boot.rs:320 (handed_over from that line) fails. Cause is in the init's order, not in the hand-over. Fix: say 'handing over' BEFORE the mount --moves (before line 89), rebuild the image (new sha256, new go), or have the check read PID 1 / the overlay root instead. After the stop: GPT of 6 disks, system_a, boot chain and both slots' flags equal before/after (measured); phone on Android b, adb answers. Nothing flashed.

About
: walleye-display-s6e3fa5


<a id="walleye-display-any-order-b-r2-b-rp2-clean"></a>
### walleye-display-any-order-b-r2-b-rp2-clean

Date
: 2026-11-23

Brief
: B-r2 and B-rp2 (mdss reset from the kernel's own dts form) at 2 s, unattended: smmu_faults, rcg_warnings, dsi_110 all 0, 13 of 13 modules ok; phone back on Android

Question
: Does the display come up clean when its modules load at 2 s, in B-r5's order (B-r2) and with the panel before msm (B-rp2), on the dtb with the upstream-form mdss reset?

Found
: Delay bisect, delays=2, image_prefix b-r then b-rp, nothing flashed, phone back on Android slot b each time. Tables in ~/lab-material/walleye/display-delay-b-r-20261123-060903 and display-delay-b-rp-20261123-061013: 2 s reached 2 s, smmu_faults 0, rcg_warnings 0, dsi_110 0, modules_ok 13, clean. dtb test tests/host/dtb-display-reset.sh passes. Remaining for the step display-any-order: modules autoloaded by modalias with no ordering list, three cold boots, the author's look (item walleye-display-look-b2). The builder's note that the dtb also differs from B-r5 (drv2624 node, dsi pinctrl label) did not change the result.

About
: walleye-display-s6e3fa5


<a id="walleye-ramroot-b-a5-v4-prompt-respawned-kernel-noise-follows"></a>
### walleye-ramroot-b-a5-v4-prompt-respawned-kernel-noise-follows

Date
: 2026-11-23

Brief
: B-a5 on rootfs v4: the respawn works (a second 'walleye login:' at screen row 96) but kernel sync_state() lines at 21.7 s print after it; check 15 fails, as it should

Question
: With rootfs v4's local.d respawn of the tty1 getty, is 'walleye login:' the last non-blank row of the glass text?

Found
: Unattended run alpine-login-20261123-061501 (image b-a5 ad91dcd7): checks 1-14 pass; check 15 fails: screen.txt row 84 holds the first prompt (mid-screen, 'Binding the usb gadget' after it), row 96 the respawned prompt, then 4 lines '[ 21.73] gcc-msm8998 100000.clock-controller: sync_state() pending due to ...' (kernel printk at the console loglevel, the 30 s deferred-probe sync_state timeout) below it. So init output no longer follows the prompt; the kernel log does. Not yet tried: the live kill-and-respawn experiment, and lowering the console loglevel (dmesg -n 3, or loglevel= on the command line, a kernel-side header word) in 50-glass-prompt.start before the pkill; sync_state is dev_info (level 6). Phone sent back to Android slot b; nothing flashed.

About
: walleye-display-s6e3fa5


<a id="walleye-rootfs-boots-run-4-check-17-wants-noload-mounts-say-norecovery"></a>
### walleye-rootfs-boots-run-4-check-17-wants-noload-mounts-say-norecovery

Date
: 2026-11-23

Brief
: Root boot run 4: booted, ssh by the lab key, checks 1-16 pass; check 17 failed on a checker bug: /proc/mounts shows the noload option as norecovery (ext4 alias); the mount itself is right

Question
: Does the stage 1 boot (image c3373d71) pass with check 17 reading PID 1 and the overlay?

Found
: Run rootfs-boots-20261123-062054 (PR 749 main): the phone booted, the lab key logged in as root, checks 1-16 pass. Check 17 failed: '/lower is not mounted with noload: /dev/sda22 /lower ext4 ro,relatime,norecovery 0 0'. The card is mounted ro, norecovery (the kernel's /proc/mounts name for noload), under overlay on /: the mount is what the check wants; the check string is wrong (tools/walleye-tries/src/rootfs_boot.rs:380 needs ro and noload; its stand-in MOUNTS line at :1184 uses 'noload', not what the kernel prints). Fix for a builder: accept norecovery (or noload), add the kernel's real line to the stand-in test. After the stop: partition tables and root partition, boot chain, both slots' flags measured equal to before; phone on Android slot b. Nothing flashed or written; no second run without the coordinator.

About
: walleye-display-s6e3fa5


<a id="walleye-ufs-root-display-image-u-d5-built"></a>
### walleye-ufs-root-display-image-u-d5-built

Date
: 2026-11-23

Brief
: U-d5 built: one kernel with UFS on and the display, 13 modules at 5 s, the root read-only under a tmpfs overlay; 42 checks, built twice identical (sha a4558827); never booted

Question
: Can the matched UFS kernel and B-r5's display reset and modules be one image that shows the Alpine root's login on the glass and ssh at once, built and checked on the host?

Found
: Yes, on the host. Kernel = topic/ufs-match-android (3644090c7) + the MDSS reset + the panel driver (local branch wip/walleye/ufs-root-display of ~/Projects/walleye-build/linux-v7.3, ac7eefecb); DRM, DRM_MSM and the panel are 13 modules built for that release, fbcon and the DPU/DSI parts built in; dtb = the display board dtb (reset in the kernel's own dts form, the lab levers) with the matched image's UFS overlay merged: the two dtbs differ in exactly the matched image's 10-line UFS delta, and no /soc@0 child of the matched dtb is lost. Initramfs init = the proven root init plus the display block (delay 5 s, then each module line, ok or FAILED rc=N logged) and 'handing over' said before the moves; its QEMU proof runs every case of the root init's proof on it with a fake display. Image sha256 a45588272b11fed90d7d13e9ed0c3a00d1c2208b9758d01ed968587978582747 at ~/Projects/walleye-build/NOT-TO-BOOT-ufs-on-images-ufs-root-display, 32 MiB as the matched image, AVB verifies, rebuilt identical across runs and under another locale, the UFS-off gate refuses it by design. Not known yet: how the SMMU behaves with UFS on and the display reset together, and the first boot (needs system_a written with rootfs v4 and the author's go naming this sha256).

About
: walleye-display-s6e3fa5


<a id="walleye-ramroot-b-a5-v5-console-loglevel-built"></a>
### walleye-ramroot-b-a5-v5-console-loglevel-built

Date
: 2026-11-23

Brief
: Rootfs v5 (dmesg -n 3 before the getty respawn) and B-a5 83b9c4c6 rebuilt, each identical twice; the keeper's run is what shows check 15 pass

Question
: Can the late kernel sync_state() info lines be kept from printing below the respawned login prompt without touching the kernel or the header?

Found
: Yes by construction, not yet measured on the phone: /etc/local.d/50-glass-prompt.start runs dmesg -n 3 (console shows warnings and worse; the kernel log keeps everything) before the pkill that makes init respawn the tty1 getty. walleye-rootfs checks the line is there and comes before the pkill (a pattern with the loglevel after, absent, or 7 fails) and that dmesg is in the image. Rootfs v5 sha256 e893dbe1eb9f1208c00230514bda6da36a75ea459b67ac241d3df813e5a50bd3; B-a5 83b9c4c644ffb8de03f5b5f1d6eaf8c6fa72ea7121699d0cab1cf2a53034f981 (the old ad91dcd7 was overwritten in images-display-ramroot by the tool's own output: rebuild it from rootfs v4 if it is wanted as history). Not known yet: whether any warning-level line still prints below the prompt (the item walleye-display-alpine-getty-experiments fallback is loglevel= on the command line).

About
: walleye-display-s6e3fa5


<a id="walleye-rootfs-boots-run-5-passed"></a>
### walleye-rootfs-boots-run-5-passed

Date
: 2026-11-23

Brief
: Root boot run 5 (image c3373d71): all 26 steps passed - booted, ssh by the lab key, PID 1 and overlay hand-over proved, root read-only, GPT/boot chain/slot flags equal, phone on Android b

Question
: Does the UFS-root image boot from RAM, hand over to the read-only Alpine root and log in over ssh, with nothing written?

Found
: Rerun 5 on the checker fixed in PRs 749 and 756: the procedure reported all 26 steps passed. Init mounted /dev/sda22 (system_a) with slot suffix _b; checks 14-19 passed; the card's partition tables, root partition, boot chain and both slots' flags read equal after the forced reboot; phone back on Android slot b. Run log in the keeper's AGENT_BUILD rootfs-run5.log.

About
: walleye-display-s6e3fa5


<a id="walleye-install-flash-boot-procedure-built"></a>
### walleye-install-flash-boot-procedure-built

Date
: 2026-11-23

Brief
: Stage 2 as one command and its rehearsal are built and stand-in tested; the stage 1 init refuses the slot it is booted from, so the command's init gate stops a flash today

Question
: Can stage 2 (flash boot_<other>, set_active, observe, set_active back) be one tested command under the verdict's four conditions, without touching the phone?

Found
: Built: procedures/install-boot-on-the-pixel-2.rs and its rehearsal on tools/walleye-tries/src/flash_boot.rs, keys in device.toml [procedures.install-boot-on-the-pixel-2]. In code: the slot read by adb and fastboot (equal); the flash target boot_<other> built from it and flash_allow (never boot_<Android's>, never another name; argv tests); the image's walleye.root must be system_<the flashed slot> (checked before the flash); the go and the image sha256 asked again at the flash; the boot partition read now equals its backup; the flash read back whole (sha256) with no other boot/dtbo/vbmeta partition changed; set_active only after the read-back, only of that slot, only while fastboot's current slot is still Android's; the intent file written first so the watching parent runs the way back; the flags after the switch checked against the bootloader's own (current slot, new slot not successful or unbootable, retry 1-7, nothing else); the GPT of every disk read whole before and after, the checker passing only slot bits (byte 6 of the attributes of A/B entries), pairwise A/B type-GUID swaps in both the primary and the backup copy, and recomputed CRCs (seen failing on a rename, a resize, a one-sided GUID change, a stale CRC, an MBR change, a non-A/B slot bit); the way back reads current-slot first and writes set_active only if it is not Android's, never reboots into the flashed slot after a failed switch back; the boot chain as stored. 244 tests, clippy, fmt, lpoco-attended check; reviewed by the programmer subagent, its findings fixed. TWO FACTS THE HOST SHOWS: (1) the stage 1 root init (and the display one) refuses the slot it is booted from ('the slot Android runs from'), and a boot from the flashed slot's own boot partition gets androidboot.slot_suffix = that slot, so the init gate of the command FAILS on the declared ufs-root image today (checked on the real image); the gate passes only an init that does not refuse and declares 'walleye-init: mounts-the-booted-slot': stage 2 needs that init (item walleye-install-stage2-init-booted-slot). (2) NOT KNOWN YET: whether our kernel can restart into the bootloader for the way back (to_bootloader = reboot bootloader needs a reboot-mode in the pm8998 pon node); when it cannot the run ends with the one hand step and its exact command (hold power and volume-down, then fastboot set_active <Android's slot> && fastboot reboot). Also not known: whether set_active to the slot already active is a no-change write (the rehearsal does not try it).

About
: walleye-display-s6e3fa5


<a id="walleye-ramroot-b-a5-v5-run-prompt-last-row-marker-on-it"></a>
### walleye-ramroot-b-a5-v5-run-prompt-last-row-marker-on-it

Date
: 2026-11-23

Brief
: B-a5 v5 run: the prompt IS the last real row (walleye login: under 'Starting local'), check 15 fails on its own ---end marker printed on the padded last row

Question
: Does B-a5 on rootfs v5 (dmesg -n 3 before the getty respawn) end the glass text in 'walleye login:'?

Found
: Run 2026-11-23 07:28 (image 83b9c4c6, RAM boot from Android b, nothing flashed): checks 1-14 passed; check 15 FAILED, but the screen shown in its own failure text ends ' * Starting local [ ok ] / Welcome to Alpine Linux 3.24 / Kernel 7.3.0 on aarch64 (/dev/tty1) / walleye login:' - no kernel line after the prompt, so v5 works. Cause is the checker: /dev/vcs has no trailing newline, so SCREEN_COMMAND's 'echo ---end' lands at the end of the padded last row, not at a line start; sections() keeps it as a screen row and the last non-blank row is spaces+'---end'. Fix is one 'echo' before '---end' in SCREEN_COMMAND (branch fix by the programmer subagent). Phone sent back to Android (adb device, slot b), nothing written. Not known yet: check 15 passing end to end, checks 16-22 and the author's look. Note: the image is read from build/images (symlink to ~/Projects/walleye-build) in a fresh worktree.

About
: walleye-display-s6e3fa5


<a id="walleye-ramroot-b-a5-v5-run-2-check-15-passes"></a>
### walleye-ramroot-b-a5-v5-run-2-check-15-passes

Date
: 2026-11-23

Brief
: B-a5 83b9c4c6 unattended: checks 1-17 pass, the prompt is the last row of the glass text; it stops at the author's look (ask 18); phone back on Android

Question
: With the end-marker fix, does B-a5 on rootfs v5 pass check 15 and the rest before the look?

Found
: Run 2026-11-23 07:45: steps and checks 1-17 passed (13 of 13 modules, three signs zero, fbcon, getty tty1, check 15 'walleye login:' last non-blank row, kernel log clean, deadman held); ask 18 unanswered after 300 s as designed; phone sent back, Android adb device, slot b, nothing flashed. Two traps met: rust-script served a cached build of the procedure until run with --force (the fixed marker only took effect then); a fresh worktree needs build/images symlinked to ~/Projects/walleye-build. Not known yet: what the author sees on the glass (item walleye-display-look-b2 and the look for this image).

About
: walleye-display-s6e3fa5


<a id="walleye-pon-reboot-mode-already-in-the-board-dtb"></a>
### walleye-pon-reboot-mode-already-in-the-board-dtb

Date
: 2026-11-23

Brief
: The board dtb already holds mode-bootloader = <2> in the pm8998 pon node (mainline's pm8998.dtsi); no dts change is needed, only a test that shows it

Question
: Does our dtb let the kernel ask the bootloader for itself (reboot-mode in the pon node)?

Found
: Mainline v7.3 pm8998.dtsi gives pon@800 (qcom,pm8998-pon) mode-bootloader = <0x2> and mode-recovery = <0x1>; the U-d5 image's dtb has both; CONFIG_POWER_RESET_QCOM_PON=y selects REBOOT_MODE. Older dtbs fail the new test dtb-pon-reboot-mode.sh. Whether the register value reaches the bootloader is NOT shown: the keeper's try, item walleye-reboot-bootloader-from-linux-try.

About
: walleye-display-s6e3fa5


<a id="walleye-install-stage2-rehearsal-stops-at-check-2-no-boot-image"></a>
### walleye-install-stage2-rehearsal-stops-at-check-2-no-boot-image

Date
: 2026-11-23

Brief
: Stage 2 rehearsal, real run: stops at check 2 - the declared image [images.ufs-root] has no boot.img beside it; item walleye-install-stage2-boot-image (build) is still open; the phone was not touched

Question
: Can the stage 2 rehearsal (procedures/rehearse-install-boot-on-the-pixel-2.rs, no go needed) run today, as the first ready item walleye-install-flash-boot asks?

Found
: Dry run passes 17 of 17. Real run: check 1 (battery) passes, check 2 fails: build/images must link to ~/Projects/walleye-build (made in the keeper's worktree), and then the AVB footer check wants NOT-TO-BOOT-ufs-on-images-ufs-root/boot.img, which does not exist (the folder holds only the .img, NOT-TO-BOOT.txt, SHA256SUMS). The stage 2 command takes [images.ufs-root] and the rebuild with the new init (walleye.booted-slot) is item walleye-install-stage2-boot-image, open, build lane, rank 3. Nothing written, phone on Android slot b, fastboot reachable, no go used. Not known yet: checks 3-17 (not reached). Note: the item walleye-install-flash-boot lists blocked_by only rootfs-boots-try and the procedure, so it shows ready while its real predecessor is the image item; the blocker is missing from it.

About
: walleye-display-s6e3fa5


<a id="walleye-ufs-try-2-stopped-at-step-16-gear-g3-under-load"></a>
### walleye-ufs-try-2-stopped-at-step-16-gear-g3-under-load

Date
: 2026-11-23

Brief
: UFS try 2: card readable, UFS_HW_VERSION 3.1.0, 6 disks, GPT hashed; gear HS-G1 idle, HS-G3 2 lanes 200 MHz under load; the sample step timed out at 10 s, run stopped, Android back

Question
: With the matched UFS-on image: which gear does the link reach, before, under load and 2 s after?

Found
: Read only, nothing written. Probe: UFS_HW_VERSION 3.1.0, MSM8998 PHY table 3.1.0. Link at probe HS_GEAR1 2 lanes rate B, HIBERN8. 6 disks (sda-sdf), sda 45 partitions, GPT sha256 2de02f2b...57cb. Before load: HS_GEAR1 2 lanes, clock 50 MHz. Under load (8 samples): HS_GEAR3 2 lanes rate B FAST_MODE, clock 200 MHz. Run stopped at step 16: the sample command (cat /tmp/load.done plus sysfs loop) was not answered within 10 s, so the 2 s-after read, GPT hash-equal and no-purge checks (17-19) did not run; the way back ran, phone on Android (phone-state.sh adb device). GPT unchanged is therefore not yet proven by this run. Not known yet: gear 2 s after load; next step is a longer timeout on the sample step (procedure tweak) and a second run, which needs the coordinator. Log: build ufs-try2.log.

About
: walleye-first-fastboot-boot


<a id="walleye-ufs-try-2-run-1-gpt-equal-result"></a>
### walleye-ufs-try-2-run-1-gpt-equal-result

Date
: 2026-11-23

Brief
: UFS try 2 run 1: EQUAL - sda's GPT hash is 2de02f2b...57cb before (the run) and after (Android), the boot chain reads back the same bytes

Question
: Is the card's GPT unchanged after UFS try 2 run 1, which stopped at step 16 before its own hash-equal check?

Found
: EQUAL. From Android (adb root, read only, nothing written, battery 4376 mV): sha256 of sda's GPT read as rootfs_boot::gpt_command reads it (dd 6 blocks at the start and 5 at the end of /dev/block/sda, 4096 B) = 2de02f2bb790342885ebb3d70683d3db6d0cf70ead73a4b3cf738ca9b4cd57cb; the run's hash was 2de02f2b...57cb. tests/host/bootchain-readback.sh read 32 partitions to ~/lab-material/walleye/bootchain-2026-11-23-gpt-equal; bootchain-readback-check.sh against bootchain-2026-11-19-2: pass, same bytes. Active slot b (ro.boot.slot_suffix); nothing sent that could change the flags (no fastboot, no set_active). Not read this time: the slot flags by getvar (phone stays on Android). So the run harmed nothing the checks can see.

About
: walleye-first-fastboot-boot


<a id="walleye-ufs-try-2-sample-silent-cause"></a>
### walleye-ufs-try-2-sample-silent-cause

Date
: 2026-11-23

Brief
: UFS try 2 sample silent 10 s: one shell carried the load state and the sysfs gear read; the log cannot say which stalled, the step now records each read's time

Question
: Why was the sample command (cat /tmp/load.done plus the power_info and devfreq loop) not answered in 10 s in run 1?

Found
: From run 1's log and saved files: the shell is telnet on the gadget, one command at a time; 8 gear reads answered under load (so the shell was not starved), the 9th, sent together with the read of /tmp/load.done, was silent for over 10 s as the 4 GB read ended (step started 08:11, failed 15 s later). Most likely cause: a sysfs power_info read goes through the UFS host's runtime power management and waits on the link as the load ends; not proven, the log has no timings per read. Fix: the read's state is asked on its own shell with no sysfs; a gear read has gear_wait=30 s (device.toml); an unanswered read is written to gear-load.txt as not known yet and sampling goes on on a fresh shell; every sample carries its time, so run 2 shows whether the stall meets the load's end. Stand-in tests use run 1's printed text, incl. a hung read.

About
: walleye-display-s6e3fa5


<a id="walleye-rootfs-boots-run-6-passed-bootloader-not-reached"></a>
### walleye-rootfs-boots-run-6-passed-bootloader-not-reached

Date
: 2026-11-23

Brief
: Stage 1 rerun 6 (c3373d71): all 26 steps pass, but the bootloader way back was NOT reached: fastboot listed no phone within 60 s after 'reboot bootloader'; forced way back ran, all measures equal

Question
: Does 'reboot bootloader' over ssh from the booted UFS-root image bring the phone to fastboot, so the way back needs no forced reboot?

Found
: No. ssh answered, 'reboot bootloader' was sent, but fastboot listed no phone within 60 s (bootloader-way-back.txt: 'bootloader not reached'); the forced reboot then returned the phone to Android slot b. Booted: init mounted /dev/sda22 (system_a) read-only (ro, norecovery) under the overlay with suffix _b; kernel log clean; 6 disks and system_a hashes, boot chain and slot flags equal before and after. The pon mode-bootloader = <2> is in the dtb (research walleye-pon-reboot-mode-already-in-the-board-dtb), yet the reboot did not reach the bootloader - cause not known yet. Phone left on Android b, fastboot reachable.

About
: walleye-display-s6e3fa5

Material
: research/walleye-display-s6e3fa5/2026-11-23/


<a id="walleye-pd-mapper-static-from-source"></a>
### walleye-pd-mapper-static-from-source

Date
: 2026-11-23

Brief
: pd-mapper builds static for aarch64 from pinned upstream sources (pd-mapper, qrtr, xz) with aarch64-linux-gnu-gcc: 796 KB, reproducible here, BSD-3 and 0BSD

Question
: How does pd-mapper get into the Alpine image, when no Alpine repo has it?

Found
: linux-msm/pd-mapper at 5ecd2fe (BSD-3-Clause) needs libqrtr (linux-msm/qrtr 29e36ae, BSD-3-Clause) and liblzma (xz 5.8.2, 0BSD). All three pinned by archive sha256 in rootfs/pins, compiled by walleye-rootfs with the host's cross gcc and static glibc, stripped: 795656 bytes, binary sha256 9b5331e2...b268 and image sha256 22b4fed2...a73c identical across separate runs. Installed at usr/sbin/pd-mapper, linked into no runlevel; the image check proves a static aarch64 ELF. Limits (programmer review): the toolchain and glibc (LGPL-2.1 relink duty) are the host's and unpinned, so identical here, not proven elsewhere; a github by-commit archive may change gzip bytes and break a pin; a musl build would match the image but needs a musl cross compiler. Not run: needs the remoteproc firmware.

About
: walleye-display-s6e3fa5


<a id="walleye-reboot-reason-bootloader-read-finding"></a>
### walleye-reboot-reason-bootloader-read-finding

Date
: 2026-11-23

Brief
: 'reboot bootloader' never reached the kernel: busybox reboot drops the argument, so no reboot reason was written. The PMIC byte our dtb writes equals Android's; IMEM word and warm reset are unknown

Question
: Why did 'reboot bootloader' over ssh not reach fastboot on our kernel (rerun 6), and what must our kernel write?

Found
: (1) Android (wahoo msm-poweroff.c, msm_restart_prepare): for cmd 'bootloader' it writes PMIC PON 0x800+0x8F bits7:1 = 2 (qpnp_pon_set_restart_reason, PM8998 is gen2: byte 0x04) AND the IMEM word 0x77665500 at 0x146bf000+0x65c (dtsi restart_reason@65c), and any non-empty cmd gives a WARM reset (qpnp_pon_system_pwr_off WARM_RESET) then deasserts PS_HOLD. Which of the two the bootloader reads is NOT KNOWN YET (ABL is closed). (2) Our kernel: restart is PSCI SYSTEM_RESET (cold unless reboot=warm; psci.c), no msm-poweroff, no pshold node; mainline qcom-pon.c with pm8998-pon shift 1 writes the same PMIC byte (mode-bootloader <2> => 2<<1) from a reboot notifier that runs before the restart handler, but ONLY if the kernel receives the string 'bootloader'. (3) The cause found: the image is Alpine with busybox init; busybox 'reboot' ignores its arguments and signals init with SIGTERM, init calls reboot(RB_AUTOBOOT): LINUX_REBOOT_CMD_RESTART with no command. reboot-mode then finds no command and writes nothing, so the PMIC byte was never written in rerun 6 - the dtb and config were never tested by that run. (4) The kept log has one line only (bootloader-way-back.txt); no kernel line from the reboot was kept (the phone has no console on that path): the claim rests on the busybox source, not on a print. (5) Smallest change: send the command string through reboot(2) LINUX_REBOOT_CMD_RESTART2 'bootloader' (a small static tool in the rootfs, no dts change); this exercises the PMIC byte alone. If the bootloader still does not stop: add the IMEM word with a syscon-reboot-mode node (offset 0x65c, mode-bootloader 0x77665500, IMEM 0x146bf000 mapped as syscon) and/or reboot=warm so PSCI uses SYSTEM_RESET2/warm. (6) NOT KNOWN YET: whether PMIC byte alone suffices; whether a cold PSCI reset keeps the PMIC byte (Android used warm); whether IMEM is writable from EL1 under this firmware.

Sources
: src-wahoo-msm-poweroff,src-wahoo-qpnp-power-on,src-wahoo-msm8998-dtsi,src-mainline-qcom-pon,src-mainline-reboot-mode,src-busybox-halt-init

About
: walleye-bootimage


<a id="walleye-ufs-try-2-run-2-stopped-at-step-16-shell-silent-under-load"></a>
### walleye-ufs-try-2-run-2-stopped-at-step-16-shell-silent-under-load

Date
: 2026-11-23

Brief
: UFS try 2 run 2: 3.1.0, PHY table ok, GPT hash 2de02f2b...57cb, HS-G3 2 lanes 200 MHz under load for 8 samples, then the shell stopped answering at step 16; Android back

Question
: With the sample step fixed (PR 778), does the matched UFS image hold HS-G3 under a bounded read and finish the sample step?

Found
: Checks 1-15 pass. UFS_HW_VERSION 3.1.0, PHY table 3.1.0, 6 disks, sda 45 partitions, GPT hash before 2de02f2bb790342885ebb3d70683d3db6d0cf70ead73a4b3cf738ca9b4cd57cb; idle HS-G1 clock 50 MHz, under load HS_GEAR3 2 lanes 200 MHz (8 samples). Then 'cat /tmp/load.done' was not answered within 10 s at step 16 (15 s after step 16 began): the stop moved from the gear reads (run 1) to the shell itself, so the shell/USB gadget goes silent while the 4 GiB read runs, which points at the load starving the gadget or the init, not at a gear fault. The run stopped, the way back ran, phone on Android (adb device). GPT after and slot flags after were NOT read by the procedure (it stopped); not known yet. No third run without the coordinator. Log: ~/.cache/agent-build/86c262/ufs-try2-run2.log

About
: walleye-first-fastboot-boot


<a id="walleye-ufs-try-2-run-2-gpt-equal-result"></a>
### walleye-ufs-try-2-run-2-gpt-equal-result

Date
: 2026-11-23

Brief
: UFS try 2 run 2: EQUAL - sda's GPT hash is 2de02f2b...57cb before (the run) and after (Android), the boot chain reads back the same bytes

Question
: Is the card's GPT unchanged after UFS try 2 run 2, which stopped at step 16 before its own hash-equal check?

Found
: EQUAL. From Android (adb root, read only, nothing written, battery 4376 mV): sha256 of sda's GPT read as rootfs_boot::gpt_command reads it (dd 6 blocks at the start and 5 at the end of /dev/block/sda, 4096 B) = 2de02f2bb790342885ebb3d70683d3db6d0cf70ead73a4b3cf738ca9b4cd57cb; the hash the run took before is the same 2de02f2b...57cb (run 1 too). tests/host/bootchain-readback.sh read 32 partitions to ~/lab-material/walleye/bootchain-2026-11-23-gpt-equal-run2; bootchain-readback-check.sh against bootchain-2026-11-19-2: pass, same bytes. Active slot b (ro.boot.slot_suffix); nothing sent that could change the flags (no fastboot, no set_active). Not read this time: the slot flags by getvar (phone stays on Android). So run 2 harmed nothing the checks can see.

About
: walleye-display-s6e3fa5


<a id="walleye-install-boot-rehearsal-passed"></a>
### walleye-install-boot-rehearsal-passed

Date
: 2026-11-23

Brief
: Stage 2 rehearsal passed, 17 of 17, nothing written: slot b by adb and fastboot, six GPTs read twice and equal, boot chain as stored

Question
: Does every step of the stage 2 command but the flash, set_active and the switch pass on the phone on Android?

Found
: Yes after two checker fixes. Image e337de53 verifies (avbtool 1.4 wants a boot.img name beside the image; the check now links one), names root system_a. adb slot suffix _b, fastboot current-slot b, retry 0, successful yes on both, unbootable no. boot_a equals its backup. The six LUNs carry 48, 4, 4, 20, 8, 8 GPT entries (not 128; the checker reads the count from the header since); each read twice, whole, passing the switch checker unchanged. Boot chain equals the stored set. The flash that would run: fastboot flash boot_a IMAGE. fastboot printed getvar current-slot:b and slot-retry-count/successful/unbootable per slot (slots-before.txt). set_active to the active slot was not tried. Phone stays on Android slot b.

About
: walleye-bootimage

Material
: research/walleye-bootimage/2026-11-23/


<a id="walleye-stage2-forced-way-back-argument-finding"></a>
### walleye-stage2-forced-way-back-argument-finding

Date
: 2026-11-23

Brief
: Stage 2 cannot brick, but the forced way back alone leans on an unmeasured count-zero fall back (set_active gives retry 1 here, not 7); a hard hang needs a hand. Not enough to open the go alone

Question
: Can stage 2 (flash boot_a, set_active a) be made safe on the forced way back alone, with the bootloader's A/B boot-count fallback?

Found
: ARGUMENT, from what the lane measured and the sources; nothing was run on the phone for this row. (1) THE FORCED WAY BACK TODAY: from the booted image the lab sends 'reboot -f', then 'echo b > /proc/sysrq-trigger'; in rerun 6 it returned the phone to Android slot b with no hand. That was stage 1 (fastboot boot, slot b still active), so the bootloader picked b. AFTER stage 2 set_active a, a forced reboot lands on a AGAIN, our image, not Android: the forced reboot alone is not the way back; the bootloader's counter is. (2) THE COUNTER, MEASURED ON THIS PHONE: set_active X gives X priority 3, successful no, unbootable no, retry-count 1 (research walleye-first-fastboot-boot, 2026-11-17: 'after: current a, a unbootable no/successful no/retry 1'), NOT the 7 of the Qualcomm source and pmaports MR 3190: Google's ABL differs. A normal boot of an unconfirmed slot decrements it: set-active a then Android on a left a successful yes retry 0, b retry 1. The slot not made active keeps its flags (b successful yes after set-active a, seen in the 2026-11-17 state reads). Expected run: boot 1 of ours (a retry 1 to 0); one forced reboot; boot 2: a is unsuccessful at retry 0, the bootloader marks it unbootable and boots b, Android, no hand and no bootloader mode; a stays unbootable until the next set_active a. NOT MEASURED: what the bootloader does on the boot after retry reaches 0 on a slot that is unsuccessful and loaded fine. Measured only: after Load Errors a showed unbootable yes retry 0 and, with b unsuccessful too, the phone stayed in fastboot (reachable by the lab: set_active b, no hand). (3) BRANCHES. A boot_a written wrong (read-back differs): the procedure stops before set_active; b still active; nothing to undo, boot_a is the unused slot's. B set_active fails: flags read again; if not as before, set_active b by the lab's fastboot; a half-written GPT is the bootloader's atomicity, NOT KNOWN YET, the same risk as every OTA. C image refused (AVB, Load Error): the source marks a unbootable and boots b by cold reset: Android; if both end unbootable the phone waits in fastboot: set_active b by the lab. D kernel boots and panics (panic=10): reboot, counter as in (2): Android on boot 2. E kernel boots, ssh answers, run completes: forced reboot, boot 2 is Android on b; the lab reads the flags (a unbootable, b successful yes); the procedure's explicit 'set_active b from the image' cannot be used because 'reboot bootloader' does not reach fastboot (rerun 6, cause not known yet). F kernel boots but ssh does not answer (network, not a hang): the image's own self-reboot timer (decision on hang outcomes: reboot -f after 240 s unless /tmp/hold) restarts it: boot 2 as in (2); no hand. G hard hang, or reboot -f and sysrq b both ignored (the c3 case): NEEDS A HAND. The PMIC watchdog does not remove it: the v1 bite is a hard reset (type 7) and with the USB cable in the bootloader then shows its charging screen and waits for a short power press (research of the 2026-11-20 bite); warm type 1 may avoid that, unproven. After the press, boot 2 is Android by the counter. (4) NOT KNOWN YET and the smallest try: (a) the flags set_active leaves on b and the retry value on a: the rehearsal item walleye-install-flash-boot-rehearsal reads both slots' slot-retry-count, slot-successful, slot-unbootable before (read only); only the real set_active shows them after; (b) the behaviour at retry 0: only a counted boot shows it, which is stage 2 itself, whose worst case is a hand on the power key, never a state fastboot cannot repair; (c) whether Google's ABL decrements for a slot set by set_active in the same session. (5) RECOMMENDATION: the forced way back alone is NOT enough to open the go. It is safe from a brick (boot_a is the unused slot, Android's b is untouched and successful, fastboot is reachable, no bootloader partition, the GPT written only by the bootloader's own set_active) but it leans on an unmeasured count-zero fall back, and its only no-hand exit is a live kernel. Open the go after the rehearsal has read the flags and the restart2 tool (walleye-reboot-reason-restart2-tool) has been tried once in RAM: if 'reboot bootloader' then reaches fastboot, the way back is the lab's explicit set_active b and the counter is the second path; if not, the go may still be given with the Top Coordinator accepting that the residual cost is a hand on the power key (branch G), not a brick. The go stays CLOSED here.

Sources
: src-aosp-qcom-bootctrl,src-caf-abl-partitiontableupdate,src-aosp-ab-bootloader-updating,src-pmaports-mr3190-slot-switch,src-busybox-halt-init

About
: walleye-bootimage


<a id="walleye-slpi-carveout-answer"></a>
### walleye-slpi-carveout-answer

Date
: 2026-11-23

Brief
: SLPI carve-out: the image needs 15 MiB (relocatable); the phone's layout is 0x94c00000/0xf00000; our 7 MiB node is too small

Question
: Which memory carve-out was the phone's SLPI firmware made for, and what does remoteproc_slpi need in our dtb?

Found
: slpi_v2.mdt: 15 program headers, every LOAD relocatable (flag bit 27), span 0x93800000-0x94700000 = 0xf00000 (15 MiB). Walleye's own dtbo and running fdt put pil_slpi_mem at 0x94c00000/0xf00000 (other dtbo entries 0x94b00000/0xf00000, source dtsi and mainline 0x94300000/0xf00000): the size is 15 MiB everywhere; our dts holds 7 MiB at 0x94b00000, too small by 8 MiB, and the overlay never overwrites it because our pil_slpi_mem is a stub label. The block 0x94c00000-0x95b00000 swallows our inherited ipa_fw/ipa_gsi/gpu/wlan_msa nodes (0x95200000-0x957fffff): delete them, add the running layout's ipa/gpu 0x95b00000/1 MiB, shift mba to 0x94500000 and video to 0x94700000. Vendor SSC node matches mainline's remoteproc_slpi on interrupt 390, pas-id 12, px=lvs2, cx via SSCCX, smp2p pid 3; mainline reg is 0x5800000 where the vendor says 0x5c00000 (NOT KNOWN YET which is right). Firmware: slpi_v2.mdt and .b00-.b14 under /lib/firmware/qcom/msm8998/google/walleye/ from vendor_a, never git. Not known yet: that TZ accepts the image at 0x94c00000 on our boot.

Sources
: vendor-walleye-slpi-firmware-header,vendor-walleye-dtbo-pil-fragments,linux-v7-3-msm8998-dtsi-slpi,lkml-slpi-msm8998-pas-2019

About
: sensors

Material
: research/walleye-slpi-carveout-read/2026-11-23/

Supersedes
: walleye-sensors-way


<a id="walleye-rootfs-boots-run-7-bootloader-reached-android-late"></a>
### walleye-rootfs-boots-run-7-bootloader-reached-android-late

Date
: 2026-11-23

Brief
: Run 7 (RESTART2 way back): checks 1-21 passed, the bootloader WAS reached (fastboot listed the phone, current-slot b = start), but Android was not back within 240 s after fastboot reboot

Question
: Does the static reboot tool pushed to /tmp send the phone to its bootloader (RESTART2), and does the way back then return to Android?

Found
: Image c3373d71 (run image d594dcbe): stage 1 passed through check 19 (init mounted /dev/sda22 system_a ro under the overlay, slot suffix _b, kernel log clean). The way back's bootloader step worked: fastboot listed the phone, current-slot b equalled the slot at the start, fastboot reboot ran at 10:21:16. Check 22 then failed: no Android within 240 s (walleye-idle-android said the phone is in fastboot). The after-stop way back brought Android home (up since about 10:34); GPT of 6 disks and system_a, the boot chain and both slots' flags measured equal after the stop. Nothing flashed, set_active or erased. Not known yet: why fastboot reboot left the phone in fastboot for the 240 s. The tool did run from /tmp (the bootloader was reached).

About
: walleye-display-s6e3fa5

Material
: research/walleye-display-s6e3fa5/2026-11-23/


<a id="walleye-ufs-try-2-run-3-silent-shell-gpt-equal"></a>
### walleye-ufs-try-2-run-3-silent-shell-gpt-equal

Date
: 2026-11-23

Brief
: UFS try 2 run 3: 3.1.0, PHY 3.1.0, HS-G3 2 lanes 200 MHz under load (7 samples), shell silent again at step 16; GPT EQUAL after; Android back

Question
: With the silent shell kept as a reading and the wait run to the limit plus 60 s (PR 812), does the matched UFS image finish the bounded read, the 2 s-after gear and the hash-equal check?

Found
: No. Checks 1-15 pass: UFS_HW_VERSION 3.1.0, PHY table 3.1.0, 6 disks, sda 45 partitions, GPT hash before 2de02f2bb790342885ebb3d70683d3db6d0cf70ead73a4b3cf738ca9b4cd57cb, idle HS_GEAR1 clock 50 MHz, under load HS_GEAR3 2 lanes 200 MHz for 7 samples. Then the shell stopped answering 'cat /tmp/load.done' (10 s each, 6 reads unanswered) and the run stopped at step 16 after the wait to the read's limit plus 60 s: the shell did not come back. Same stop as run 2. Phone-side cause of the silence NOT KNOWN YET. Not reached: gear 2 s after load, GPT-after, no-purge check, slot flags. Way back ran: phone on Android slot b. Read again from Android (adb root, read only): GPT hash 2de02f2b...57cb EQUAL to before; bootchain-readback of 32 partitions, check against bootchain-2026-11-19-2: pass, same bytes. Log kept at ~/lab-material/walleye/ufs-try2-run3.log; no gear-load.txt was kept by the run. Per the run sheet no fourth run without the coordinator, who closes try 2 on the facts.

About
: walleye-first-fastboot-boot


<a id="walleye-install-boot-rehearsal-2-passed"></a>
### walleye-install-boot-rehearsal-2-passed

Date
: 2026-11-23

Brief
: Stage 2 rehearsal 2 after PR 817: 17 of 17 passed, nothing written; slot b, image e337de53 declared, six GPTs equal, boot chain as stored

Question
: Does the stage 2 rehearsal still pass on the code of PR 817 (RESTART2 way back, stored-set reboot tool check)?

Found
: Yes: 17 of 17 steps. Battery good; image e337de53 whole, AVB footer verifies, names system_a; init accepts the booted slot; stored chain, key and reboot tool as declared; adb slot b agrees with fastboot; boot_a equals its backup; six GPTs kept whole, read again equal and through the switch checker; slot flags equal twice; boot chain equals the stored set. The flash command was shown, not run. Phone ends on Android slot b. The run needed build/images linked to ~/Projects/walleye-build in the worktree (untracked).

About
: walleye-bootimage

Material
: research/walleye-bootimage/2026-11-23/


<a id="walleye-ufs-silent-under-load-causes-ranked"></a>
### walleye-ufs-silent-under-load-causes-ranked

Date
: 2026-11-23

Brief
: Why the shell goes silent about 4.5 s into a sustained O_DIRECT read: the first gear switch is cleared; 5 causes ranked, none settled; cause not known yet

Question
: Why do the phone's shell and usb network go silent under a sustained read of sda, after 7-8 answered samples, and what ONE try with no new risk tells the causes apart?

Found
: WHAT THE LOGS COUNT (ufs-20261123-085319 and -105319, ufs-try2-run3.log): the host went to HS_GEAR3 / 200 MHz core clock at the first sample (0.5 s) and held it for every answered sample (run 2: 8 up to 4.6 s; run 3: 7 up to 4.0 s, answered in 0.1 s), 2 lanes, rate B, link ACTIVE; then silence from the 4.6 s read on in both, six reads unanswered over 110 s. Same moment in all runs: a fixed time or volume after the load starts, not chance. Idle the same phone answered for minutes (the link read took 171 s). The first run (080827) had no load samples. After the failure the file shows dev_pm SLEEP, link HIBERN8, gear 1: the card was idle when that last read was made; whether the shell came back is NOT KNOWN YET (the run log only says no answer within limit+60 s). The phone printed nothing after the samples: no kernel text from the stall exists. Runs 1 and 2 logs are not in ~/lab-material (only ufs-try2-run3.log). THE CAUSES, ranked: (1) [cleared as stated] a gear or clock switch hangs: the up-switch at the load start finished (ufs-qcom notify = hibern8 enter, timers, core clk ctrl, hibern8 exit), so only a LATER switch is left: ufshcd's devfreq (100 ms poll, simple_ondemand) may scale down and up again when the queue drains between 1 MiB requests; each switch holds the scaling lock, waits up to 1 s for the doorbell and freezes new requests (ufshcd_clock_scaling_prepare) - a stuck second switch would stall the I/O, but not the usb gadget, unless the stall is a bus hang from touching a register while the clock is off: rank 1 for that form. (2) a stuck-asserted completion interrupt (irq 265 level-high) or a storm: all SPIs default to CPU 0 with the usb gadget's; 1 MiB direct reads give only hundreds of interrupts a second, so only a never-cleared status bit starves CPU 0; rank 2. (3) NEW, not in the item: the missing bus votes. ufshc has no interconnects; Android voted the fabric for the UFS traffic; at gear 3 the unvoted fabric (snoc/bimc left as the bootloader set it) could starve the usb gadget's DMA and the cpu's: slow, deterministic after the buffer-sized time; rank 3 - against it: the OnePlus 5 on mainline uses the same node and the same missing votes and reads its root from UFS (wiki, dts series), so the shared path is not enough to hang and the difference is OURS (the diagnostic kernel config, clk_ignore_unused, the init's watchdog petter and its starvation, the card's quirks, the RST_n pulse). (4) a watchdog bite after our petter starves: the petter is a bounded one in the root init; a starved petter would end in a warm reset and Android back by itself, which is what the phone does; rank 2 or 3 and CHEAP to test by reading the PON reason on the way back. (5) O_DIRECT memory effect (bs=1M pins 1 MiB, nothing grows) and the usb network's own bulk path: lowest; the shell is polled over the same usb that would have to die first, so (d) cannot be told from the rest by the poll alone. WHAT SETTLES WHICH: a log that survives the stall, and one variable at a time. The ramoops chase has not yet shown that a warm reset keeps our kernel's text (walleye-pk5l2-tried; the item walleye-ramoops-after-unlock waits for the author's unlock), so the log is written BEFORE each step to the phone's RAM and read back over the same link only while it answers; a second channel that survives is not known yet.

Sources
: mainline-v612-ufshcd-clk-scaling,oneplus5-mainline-ufs-dts,wahoo-4-4-ufs-qcom-c,src-mainline-v73-ufs-qcom-ice

About
: walleye-ufs-match-android-z


<a id="walleye-stage2-run-1-flash-resets-the-other-slots-flags"></a>
### walleye-stage2-run-1-flash-resets-the-other-slots-flags

Date
: 2026-11-23

Brief
: Stage 2 run 1 stopped at check 18: fastboot flash boot_a itself reset slot a (retry 0->1, successful yes->no); then way back by fastboot reboot, Android on b

Question
: Does the one fastboot flash of boot_a leave the slot flags unchanged (the procedure's check 18)?

Found
: No. Checks 1-17 passed (battery 4376 mV, boot_a read back equal, no other boot/dtbo/vbmeta changed, GPT kept). After the flash, flags read slot-retry-count:a 0->1 and slot-successful:a yes->no; b unchanged; current-slot b. The flash is the writer, not set_active. Command stopped (exit 1), its own way back ran: fastboot reboot, Android slot_suffix _b. No set_active, no second run. Open: the check forbids a change the bootloader makes on any flash; decide whether the check allows exactly this change (a decision for the coordinator).

About
: walleye-display-s6e3fa5


<a id="walleye-stage2-flash-resets-flags-documented"></a>
### walleye-stage2-flash-resets-flags-documented

Date
: 2026-11-23

Brief
: Documented: AOSP's bootloader spec says writing a slot's partition clears successful and resets the retry count; run 1's 0->1 and yes->no match set_active X's state (retry 1, successful no)

Question
: Is it the bootloader's documented behaviour that fastboot flash of a slot's boot partition resets that slot's tries and successful bit? Do rerun 7 and stage 2 run 1 agree?

Found
: DOCUMENTED. AOSP 'Implement OTA updates' (Support for slots) says: 'When modifying a partition that's part of a slot, the bootloader clears successfully booted and resets the retry count for the slot.' set_active (and the HAL's setActiveBootSlot) additionally changes the active slot and clears slot-unbootable (the only way to clear it). So the flash changing the flashed slot's flags is specified behaviour, not an anomaly; check 18, which forbade any flag change, was too strict and should allow exactly: flashed slot successful yes->no and retry reset to a positive value, nothing else (other slot, unbootable, current-slot unchanged). OBSERVED AGREES: stage 2 run 1 after flash of a: retry 0->1, successful yes->no, unbootable no, b unchanged, current b; the first-fastboot-boot material (2026-11-17, quoted in walleye-stage2-forced-way-back-argument-finding) after set_active X: retry 1, successful no, unbootable no (priority 3). Same flags after the flash as after set_active, apart from the active slot, which the flash did not change. The reset value is 1 here, not the spec's 'usually 3' (Google's ABL differs, already noted). NOT SHOWN: priority after the flash (getvar does not print it), so 'same as set_active' holds for retry, successful and unbootable only. Not read: Google's ABL source (not open for walleye) and update_engine's code; the spec page was enough. Plain sentence: documented (AOSP spec), and observed on this phone to match.

Sources
: aosp-bootloader-updating,aosp-ab-system-updates

About
: walleye-bootimage


<a id="walleye-stage2-run-1-gpt-after-flash-finding"></a>
### walleye-stage2-run-1-gpt-after-flash-finding

Date
: 2026-11-23

Brief
: Run 1's flash changed only boot_a's attribute byte (0x72 to 0x3b) and the CRCs it implies, on sda; five other disks byte equal; boot chain as flashed; the run never read the GPT after

Question
: What exactly changed in the GPT when fastboot flash boot_a reset slot a's flags, read again from Android against run 1's gpt-before.txt?

Found
: Read from Android 2026-11-23 (adb root, read only, nothing written, no fastboot verb) the six disks' GPT with the stage 2 dump command, compared with install-boot-20261123-112709/gpt-before.txt by the stage 2 checker (gpt_switch_only, PR 817 code) and byte by byte. sdb to sdf: 0 bytes differ. sda: 18 bytes differ, all classed: primary header CRC (block 1 off 16-19) and entries CRC (off 88-91); the same two CRCs in the backup header (block 10); ONE entry byte in each copy of the entries (block 2 and block 6, offset 1078 = entry 8 boot_a, byte 54, attribute bits 48-55): 0x72 to 0x3b. No other entry's bytes, no type GUID, nothing else. Checker: 1 entries' slot bits and 0 type GUIDs changed, the CRCs are right, nothing else. By the ABL layout the byte reads priority 2 to 3, retry field 6 to 7, successful 1 to 0; fastboot getvar printed retry 0 then 1, successful yes then no, so the retry field's decoding is not known yet; the raw byte is the fact. Verdict: ONLY slot a's attribute bits and the CRCs they imply differ; the Top's STOP condition did not occur. 'GPT kept' meant the table before the flash was kept (gpt-before.txt only): the run's folder has no gpt-after (check 18 stopped it before that read) and the after-flash flags came from fastboot getvar (slots-after-flash.txt). This read is after the run's way back, so it is the net change of the run; boot_b's entry is unchanged. Boot chain: boot_a e337de53 (the flashed image), boot_b, dtbo_a/b, vbmeta_a/b equal to the run's parts-readback.txt. Phone on Android slot b. Material kept in ~/lab-material/walleye/install-boot-20261123-112709/ as gpt-read-after-return.txt and parts-after-return.txt.

About
: walleye-bootimage


<a id="walleye-stage2-rerun-2-stops-at-check-9-boot-a-is-the-image"></a>
### walleye-stage2-rerun-2-stops-at-check-9-boot-a-is-the-image

Date
: 2026-11-23

Brief
: Stage 2 rerun 2 stopped at check 9: boot_a already holds e337de53 (run 1's flash), so the stored backup (2a5cd3b2) no longer equals it; nothing written, phone on Android b

Question
: Does stage 2 rerun 2 pass with boot_a already flashed by run 1?

Found
: Checks 1-8 pass (battery, image, init, chain, go, adb slot _b). Check 9 compares boot_a read now with the stored backup of the stock boot_a: read now e337de53446a, backup 2a5cd3b20ef0; it stops 'the flash could not be undone'. The coordinator allowed reflashing the same image, but check 9 refuses before any flash. No fastboot verb ran, nothing written; phone on Android slot b. Per the rule, no third run. Fix is the coordinator's: check 9 must accept boot_a equal to the declared image when the stock backup is on file from before run 1. Full output in material run.log.

About
: walleye-display-s6e3fa5

Material
: research/walleye-display-s6e3fa5/2026-11-23-stage2-rerun2/


<a id="walleye-install-boot-rehearsal-3-passed"></a>
### walleye-install-boot-rehearsal-3-passed

Date
: 2026-11-23

Brief
: Stage 2 rehearsal 3 on main after PR 841: 17 of 17 passed, nothing written; boot_a holds e337de53, its stock backup on file and hashing to its record

Question
: With check 9 accepting a boot_a that already holds the declared image, does the real rehearsal pass, in particular the boot partition check (backup path, argument order)?

Found
: Yes: 17 of 17. Check 8 (boot partition read now is its backup or the declared image with the backup on file): boot_a holds e337de53, the stock backup is on file and hashes to its record 2a5cd3b2, so the flash can be undone. adb slot b agrees with fastboot; flash target shown (boot_a), not run; six GPTs kept whole, read again equal and through the switch checker; slot flags equal twice; boot chain equals the stored set; phone back on Android slot b, no way-back needed. Log: rehearsal-3.log. Needed build/images linked to ~/Projects/walleye-build in a fresh worktree (untracked).

About
: walleye-bootimage

Material
: research/walleye-bootimage/2026-11-23/


<a id="walleye-stage2-rerun-3-stops-at-check-16-flash-changes-nothing"></a>
### walleye-stage2-rerun-3-stops-at-check-16-flash-changes-nothing

Date
: 2026-11-23

Brief
: Stage 2 rerun 3 stopped at check 16: boot_a already held e337de53, so the flash changed no partition and the check expecting boot_a to change failed; no set_active; phone on Android b

Question
: Does the stage 2 one command pass end to end with boot_a already holding the image, under the go for e337de53?

Found
: Checks 1-15 passed (battery, image sha e337de53 with AVB footer, init accepts slot, stored chain, go recorded, adb slot suffix _b, boot_a already holds the image and the stock backup hashes to its record 2a5cd3b2, 6 GPTs kept, fastboot agrees, flags read, one fastboot flash boot_a, phone back on Android as root). Check 16 failed: 'the partitions that changed with the flash are [], expected only boot_a' - the flash wrote the same bytes, so nothing changed, and the check demands a change. Same class as check 9 in rerun 2: a check assumes boot_a differs from the image before the flash. Nothing past the flash ran: no set_active, no way back needed; the phone is on Android slot b (phone-state.sh: adb device, verdict android). Not run a third time, per the item. Needs the coordinator's amendment of check 16 (accept an unchanged boot_a when it already held the image and reads back as the image), or restoring the stock boot_a first. Log: the run's own log under AGENT_BUILD stage2-run3.log.

About
: walleye-display-s6e3fa5


<a id="walleye-stage2-repeat-audit-29-steps"></a>
### walleye-stage2-repeat-audit-29-steps

Date
: 2026-11-23

Brief
: Of the 29 steps of stage 2, three assumed a first run (9, 16, 18) and are amended; every other step is state-independent or a first-run expectation of the boot itself

Question
: Which steps of the stage 2 command assume the state of a first run, against the phone's state after rerun 3?

Found
: Walked every step: 9 (amended PR 841), 16 (amended now: boot_a reads back as the image, no other partition changed, whether or not boot_a changed) and 18 (amended PR 836) were the only ones comparing against a first-run value; 8, 12-14, 19, 20, 23, 24 walked by a test against stand-in adb and fastboot with the post-run-1 flags and boot_a = image; GPT checks hold no bit value. Steps 21-28 after set_active are first-run expectations of the boot itself, listed with what each expects. What the bootloader leaves after set_active of Android's slot is not known yet.

About
: walleye-bootimage

Material
: research/walleye-bootimage/2026-11-23/


<a id="walleye-stage2-run4-stops-at-step-21-no-ssh-from-slot-a"></a>
### walleye-stage2-run4-stops-at-step-21-no-ssh-from-slot-a

Date
: 2026-11-23

Brief
: Stage 2 run 4: steps 1-20 pass (flash was a no-op, set_active a ok, flags as switch); slot a never answered ssh in 170 s; phone fell back to Android b by itself, slot a unbootable

Question
: Does the stage 2 boot image (e337de53) on boot_a, made active by set_active, boot from the card and answer ssh at 172.16.43.1?

Found
: Run 2026-11-23 13:09-13:13, log ~/.local/state/walleye-keeper/stage2-run4.log. Checks 1-20 and steps 5-19 passed: boot_a already held e337de53, boot_a read back as the image, flags changed as the switch describes. Step 21 failed: 'no ssh login within 170 s as root@172.16.43.1: ssh connect to host 172.16.43.1 port 22: Connection timed out'. The way back found Android already running ('phone is on Android already; nothing sent'), then its ask to send /tmp/walleye-reboot failed with the same ssh timeout and printed the manual hint (hold power+vol-down, fastboot set_active b && fastboot reboot). Afterwards adb showed Android slot _b, uptime 4 min, ro.boot.bootreason=watchdog (sys.boot.reason=watchdog): the new slot did not come up and the phone returned to b without our command, via a watchdog reset. fastboot (read via devlab) then: current-slot b; slot a successful no, unbootable YES, retry 0; slot b successful yes, unbootable no, retry 0. Before the run slot a was retry 1, unbootable no. So set_active a spent slot a's tries and the bootloader fell back to b. NOT KNOWN YET: whether the kernel started at all (no console/ssh evidence: usb gadget 172.16.43.1 never answered; the new-slot boot wrote nothing we read) or died before the gadget; the 170 s window may also be shorter than the retries' total. Phone ended on Android slot b, bootable; slot a unbootable (a later set_active a resets it). Not rerun, per the item's rule; lease released.

About
: walleye-display-s6e3fa5


<a id="walleye-stage2-run4-pstore-read-finding"></a>
### walleye-stage2-run4-pstore-read-finding

Date
: 2026-11-23

Brief
: pstore was empty: the phone had rebooted since run 4 (up 10 min), so slot a's console is gone; Android's normal-boot cmdline is the stock header's plus bootloader words

Question
: What did slot a's boot leave in pstore/ramoops, and what does the bootloader add to a normal boot's command line?

Found
: Read from Android b by adb root, nothing written, no reboot. /sys/fs/pstore is empty (dir made 13:13, uptime 10 min): the phone rebooted after run 4, which overwrote the console of slot a's try; nothing says whether our kernel started. /data/vendor_ce/0/ramoops holds only priv.pem/pub.pem (not read); other paths absent. Android's kernel enables pstore (console [pstore0]), so the next try can be read the same way if Android b is not rebooted first. Normal-boot cmdline = the stock boot_a header's cmdline (androidboot.hardware=walleye ... dtbo_idx=3) plus bootloader words: verifiedbootstate=orange, keymaster=1, root=PARTUUID=0000002b-..., bootdevice=1da4000.ufshc, serialno, boottime, mdss_mdp.panel, fpsensor, ddr/ufs, bootloader=mw8998-003.0085.00, bootreason, slot_suffix=_b, want_initramfs rootwait ro init=/init androidboot.force_normal_boot=1. Slot suffix by adb: _b. Phone left as found.

About
: walleye-bootimage

Material
: research/walleye-bootimage/2026-11-23/


<a id="walleye-stage2-slot-a-no-ssh-causes-ranked"></a>
### walleye-stage2-slot-a-no-ssh-causes-ranked

Date
: 2026-11-23

Brief
: Why slot a gave no ssh after a normal boot: ranked causes (bootloader-armed watchdog, cold USB, no start at all, init path) and ONE try that tells them apart: set_active a again, read pstore at once

Question
: Why did slot a not answer ssh after a normal boot of image e337de53 when fastboot boot of the same kernel (c3373d71, stage 1) did, in run 4?

Found
: FACTS (read in this lab, no phone): (1) Stage 1 from RAM: ssh login answered about 12 s after the boot started (rootfs-boots-run-7.log 31->32), so the 170 s window is not too short. (2) Same kernel, dtb and init in both; the stage 2 image differs by the word walleye.booted-slot and the bootloader's added words. (3) bootreason=watchdog (AOSP: the bootloader's reading of a hardware watchdog reset) with no panic= on our command line: something bit; a stuck kernel does not reset by itself. (4) A bite with the cable in boots the charger screen in our own earlier PMIC-wdt try (walleye-wdt-reset-boots-charger-mode), here the bootloader fell to b by itself: the fall-back came from the retry counter (slot a retry 1 before set_active, 0 and unbootable after), not a power press. (5) The dts has kpss-wdt (0x17817000) and pm8998_wdt enabled with wakeup-source but our init opens neither (the stage 2 init is the root's OpenRC), so no software of ours arms a watchdog. NOT KNOWN YET: whether our kernel started (pstore empty because the phone rebooted before it was read), when the bite came. RANKING by what each explains and its cost to test: (b-wdt) 1st: the bootloader arms the apps watchdog for a normal boot (it hands it to Android's msm_watchdog, which pets it) and does not for fastboot boot; our kernel never pets it, so it bites after tens of seconds: explains bootreason=watchdog, the fall-back, and why the same kernel is fine from RAM. Also fits a kernel that was running but the gadget never up. Test: pstore console after the next try: our init lines then a cut-off; and the stock fdt's qcom,bark-time and pet-time read from Android b (read-only). (b-usb) 2nd: cold USB: dwc3/qusb2phy from a cold start (in fastboot boot the bootloader's USB is up and handed over; a normal boot has it off). Cannot explain bootreason=watchdog alone (a missing gadget does not reset the phone) unless (b-wdt) also holds, so it ranks with (b-wdt), not above. Test: the same pstore: dwc3/udc lines and the gadget unit's lines after the root mount. (a) 3rd: the words root=PARTUUID, init=/init, want_initramfs, rootwait, ro: with an initramfs the kernel ignores root=, runs /init as the init= word says; our init reads only androidboot.slot_suffix (_a) and walleye.* words, so low. Same pstore shows it (init line mounts-the-booted-slot). (c) 4th: dtbo_idx=3 overlay on a normal boot: the same dtbo stage 1 got from the matched build; whether fastboot boot applies it is not known yet; a missing overlay changes nodes, shows in the kernel log. (d) 5th: AVB orange with algorithm NONE: an unlocked device boots a failed check with a warning (orange); the measured boottime SW:10024 ms in Android's own orange boot is the warning screen wait. A refusal would be Load Error (seen with stage-1-like images: walleye-try-s1-gadget-up) and no reset, so it does not explain the watchdog. (e) init booted-slot path never run on hardware: only a guard that fails closed (refuse), not a reset by itself. TO TELL APART with ONE try and no flash (boot_a already holds e337de53): the keeper makes slot a active again (fastboot set_active a, which gives slot a its tries back), waits for the fall-back to Android b WITHOUT any reboot of b, and reads /sys/fs/pstore/console-ramoops by adb root at once (ramoops 0xa1100000 is in our dts as in the stock one; the kernel prints into it). What it must show: NO file or no kernel lines = the kernel never started or ramoops not kept: candidates (c),(d),(e) or the region not surviving the bite (then the t3 proof walleye-pstore-t3 decides: its marker). Our lines up to the root mount and the usb-gadget unit then a cut-off = our kernel ran: (b-wdt) when the last line time is under about 60 s and no dwc3 error; (b-usb) when dwc3/qusb2 errors or no udc line appear; a refuse line = (e). Read-only, from Android b, before that: the stock fdt's watchdog nodes (/proc/device-tree wdt bark/pet), the stock kernel cmdline above, nothing else. Material: research/walleye-bootimage/2026-11-23/.

Sources
: src-android-boot-reason

About
: walleye-bootimage


<a id="walleye-install-boot-rehearsal-4-passed"></a>
### walleye-install-boot-rehearsal-4-passed

Date
: 2026-11-23

Brief
: Stage 2 rehearsal 4 on main after PR 855: 18 of 18 passed, nothing written; the pstore-first read (check 17) works on Android; pstore empty after the rehearsal's own bootloader excursion

Question
: Does the rehearsal's pstore-first read (check 17), wired for real after PR 855, run on the phone on Android, and what does it read?

Found
: Yes: 18 of 18 (checks renumbered, 18 steps). boot_a holds e337de53, stock backup on file and hashing to its record; slot b by adb and fastboot; flash target shown (boot_a), not run; six GPTs read twice and equal; slot flags equal twice; the failed-boot read ran as root over adb: pstore listing (empty, as the phone had just come back from its adb reboot bootloader excursion), uptime, id, cmdline (androidboot.bootreason=reboot, slot_suffix _b, verifiedbootstate orange), bootreason props (reboot / sys: bootloader); boot chain equals the stored set; phone ends on Android slot b. Reading of the item: its 'NO reboot' was taken as no fastboot reboot verb and no change of slot, since the rehearsal's step 10 (adb reboot bootloader and back, as rehearsals 1 to 3) is how it reads slots in fastboot; the pstore was already empty before. Log: rehearsal-4.log. Needed build/images linked to ~/Projects/walleye-build (the dir itself) in the fresh worktree.

About
: walleye-bootimage

Material
: research/walleye-bootimage/2026-11-23/


<a id="walleye-stage2-run5-stops-at-check-18-flash-resets-unbootable-too"></a>
### walleye-stage2-run5-stops-at-check-18-flash-resets-unbootable-too

Date
: 2026-11-23

Brief
: Stage 2 run 5: checks 1-17 pass; check 18 stops it: the no-op flash of boot_a reset slot-unbootable a yes->no and retry 0->1; no set_active; Android b

Question
: Does stage 2 run 5 (pstore-first code, image e337de53) reach slot a's boot?

Found
: Run 5 stopped at check 18 before set_active: with boot_a already holding e337de53 the flash still changed slot a's flags: unbootable yes->no and retry 0->1 (successful stayed no); the check allows only successful and retry count. Phone back on Android b (adb, slot _b, boot completed) by fastboot reboot; no set_active sent; step 21 and the pstore read not reached. Slot flags now: a retry 1, unbootable no, successful no; b successful yes, current. Decide: allow the unbootable bit to clear with the flash (measured here and in run 1). Logs in material.

About
: walleye-display-s6e3fa5

Material
: research/walleye-display-s6e3fa5/2026-11-23/


<a id="walleye-stage2-run-5-gpt-after-flash-finding"></a>
### walleye-stage2-run-5-gpt-after-flash-finding

Date
: 2026-11-23

Brief
: Run 5's flash changed only boot_a's attribute byte (0xb2 to 0x3b: unbootable cleared, retry 6 to 7, priority 2 to 3) and the CRCs it implies, on sda; five other disks byte equal; boot chain as before

Question
: What changed in the six GPTs when run 5's flash cleared slot a's unbootable bit and reset its retry, compared with run 5's own gpt-before?

Found
: Read from Android b (adb root, no reboot, no fastboot verb) the six disks' GPTs whole with gpt_dump_command (6 head + 5 tail blocks), compared byte by byte with install-boot-20261123-140427/gpt-before.txt (cmp -l). sdb to sdf: byte equal. sda differs at 18 bytes: entry 8 (boot_a) attribute byte 54, at 9270 in the primary entries and 25654 in the backup entries, 0xb2 to 0x3b (bits: priority 2 to 3, retry 6 to 7, unbootable 1 to 0, successful 0 and active 0 as before); the primary header CRC (4112-4115) and entries CRC (4184-4187); the backup header CRC (40976-40979) and entries CRC (41048-41051), the same new entries CRC in both. Nothing else: no other entry, no type GUID, no LBA, no boot_b byte. Verdict: ONLY slot a's attribute bits and the CRCs they imply differ, which meets the Top Coordinator's rule for check 18. The new byte 0x3b equals run 1's after-flash byte. Boot chain: sha256 of boot_a/b, dtbo_a/b, vbmeta_a/b equal to run 5's parts-before (boot_a e337de53). Slot b current, Android up 11 min. Phone left as found (adbd stays as root, which the lab sets itself). Pitfall: adb exec-out merges stderr, so hash with 2>/dev/null inside the quoted command.

About
: walleye-bootimage


<a id="walleye-stage2-walk-by-state"></a>
### walleye-stage2-walk-by-state

Date
: 2026-11-23

Brief
: Stage 2's walk passes from all four starting states (stock, after a flash, after a failed boot, after a success); check 18 allows unbootable yes to no

Question
: Does every step of stage 2 hold against each state the command can start from, and what must check 18 allow after a failed boot?

Found
: Stage 2's walk, run in order (steps 8, 9, 12, 13, 14, 16, 18, 19, 20, way back, flags back) against each state the command can start from, with a fake fastboot whose flash also clears the flashed slot's unbootable bit as run 5 printed. State | a's flags before | passes?
(i) stock | retry 0, successful yes, unbootable no, boot_a stock | passes
(ii) after one flash | retry 1, successful no, unbootable no, boot_a the image | passes
(iii) after a failed boot of a | retry 0, successful no, unbootable yes (run 5's printed before-state), boot_a the image | passes only with the amended check 18 (yes to no allowed, never no to yes); failed on the old code
(iv) after a successful stage 2 and its way back | retry 7, successful no, unbootable no, boot_a the image | passes
Check 18 now: successful ends no, retry ends positive, unbootable may stay or go yes to no, never no to yes; slot b, the current slot and the names are equal. Tests: run 5's printed text passes, no-to-yes refused, b's unbootable change refused, successful no-to-yes refused; test the_whole_walk_passes_from_each_state_the_command_can_start_from. 316 tests, clippy 0 warnings, fmt, dry run 29 steps, check --host ok. No phone touched.

About
: walleye-bootimage


<a id="walleye-stage2-run6-stops-at-check-16-adb-read-stalls"></a>
### walleye-stage2-run6-stops-at-check-16-adb-read-stalls

Date
: 2026-11-23

Brief
: Stage 2 run 6: checks 1-15 pass incl. the flash; check 16's adb exec-out dd read of boot_a stalled 25 min (transport), killed by hand; no set_active; Android b

Question
: Does stage 2 run 6 reach slot a's boot?

Found
: Run 6 (image e337de53, started 14:31): checks 1-12 and the flash (step 14) passed, Android back on b. Check 16 reads boot_a back by 'adb exec-out dd | sha256sum' and sat from 14:33 to 14:58 with no output: host adb 3573598 and sha256sum 3573599 waiting; phone dd pid 4166 in wait_woken on a full socket, adbd up, load 3.5, uptime 25 min, shell answered (a transport stall, not a UFS hang; stage2-run6-stall.txt). The coordinator asked to kill the host adb child: done at 14:58, the check failed by itself (SIGTERM), the way back found Android already on b, nothing sent, no set_active. boot_a holds e337de53 again (flash is a no-op). Earlier starts of this session failed on the keeper's own mistakes (cwd; missing build/images link to ~/Projects/walleye-build) before any phone contact. Next: one rerun inside the go to 16:26 if the coordinator records it on the item (its note says no second run); if the same read stalls, the read-back needs a bounded timeout or a chunked read.

About
: walleye-display-s6e3fa5

Material
: research/walleye-display-s6e3fa5/2026-11-23/


<a id="walleye-stage2-run7-stops-at-step-21-no-ssh-watchdog-fallback"></a>
### walleye-stage2-run7-stops-at-step-21-no-ssh-watchdog-fallback

Date
: 2026-11-23

Brief
: Stage 2 run 7 (e337de53): steps 1-20 pass incl. the read-back; slot a gave no ssh in 170 s; phone back on Android b by itself in about 1 min, bootreason watchdog, pstore empty

Question
: Does slot a boot our stage 2 image to ssh in the one rerun after run 6's adb stall?

Found
: Run 7 (15:10 to 15:15, image e337de53, under the go of PR 870): checks 1-20 passed, boot_a read back equal (flash a no-op), set_active a at 15:12:32, flags as the switch. Step 21: no ssh at 172.16.43.1 within 170 s; sending nothing, adb answered at once - Android b was already back (uptime 2 min at 15:15:35, so the reset came about 60 s after set_active). failed-boot-read.txt (whole): bootreason and sys.boot.reason both 'watchdog', slot suffix _b, verifiedbootstate orange, pstore listing EMPTY as root (no console of our kernel in it), cmdline is Android's. As run 4: the bite came about 1 min after set_active; no console recovered. Way back found the phone on Android; nothing sent; no second run.

About
: walleye-display-s6e3fa5

Material
: research/walleye-display-s6e3fa5/2026-11-23/


<a id="walleye-stage2-run7-host-usb-log-no-gadget"></a>
### walleye-stage2-run7-host-usb-log-no-gadget

Date
: 2026-11-23

Brief
: Run 7: the lab host's USB log shows no gadget of ours for 67 s after the reboot into slot a; pstore cannot help (hard reset)

Question
: Did our kernel's USB gadget come up at all during slot a's boot in stage 2 run 7, as seen by the lab host (journalctl -k, usb 3-2)?

Found
: Host kernel log (journalctl -k on the lab machine, usb 3-2) around run 7: 15:12:09 Android b's adb device 18d1:4ee7 appears; 15:12:24 disconnect; 15:12:30 the bootloader's fastboot device 18d1:4ee0 appears (the command reads the flags, set_active a); 15:12:32 disconnect (the phone reboots into slot a); then NOTHING until 15:13:39, when Android b's adb device 18d1:4ee7 appears (Android b up: uptime 2 min at 15:15:35, kernel start about 15:13:35, bootreason watchdog). So for 67 s after the reboot no USB device of ours appeared (stage 1's gadget enumerates about 12 s after kernel start): our kernel did not bring a gadget up, or did not run. The slot a attempt lasted about 35-40 s from the reboot to the reset: a watchdog the bootloader arms (about 30 s after the jump) fits; the second boot was the bootloader's retry-count fall back to b. pstore stays empty because a PMIC watchdog bite is a hard reset (research walleye-pstore-warm-reset-how-it-stands, walleye-ml3-pstore-empty): DDR content is not kept, so more pstore reads will not show our console. NOT KNOWN YET: whether the kernel ran at all (a hang before the gadget), or ran and failed to start its gadget cold, or the init failed before the Alpine root (the gadget is set up by the Alpine rootfs services after switch_root, so an init that cannot mount system_a gives exactly this picture). The host log is a free evidence channel for every try: it timestamps every USB (dis)connect.

About
: walleye-bootimage


<a id="walleye-fastboot-reboot-stays-in-fastboot-causes-ranked"></a>
### walleye-fastboot-reboot-stays-in-fastboot-causes-ranked

Date
: 2026-11-23

Brief
: fastboot reboot stayed in fastboot 240 s: ranked (a) our PMIC reason byte persists, (b) bootloader rule, (c) slot state, (d) USB; smallest try: fastboot reboot twice, timed

Question
: Why did 'fastboot reboot' leave the phone in fastboot for 240 s after our kernel reached the bootloader by RESTART2 (rerun 7), and what is the reliable last leg?

Found
: Kept facts (rerun 7, run log and out): stage 1's kernel took the phone to fastboot by RESTART2 'bootloader'; fastboot listed it, current-slot b = start; `fastboot reboot` was sent (log clock 10:24:36; the row's 10:21:16 is another clock); check 22 failed at 10:28:37, 240 s later, the host saying the phone was in fastboot. Slot flags, GPTs and boot chain equal after. The after-stop way back (lab.rs way_back: in fastboot it sends `fastboot reboot` again, then shell legs) brought Android home; its own lines were not kept, so the number of resets and the time it took are NOT KNOWN YET (the row's 'up about 10:34' is an uptime estimate).
Public source (generic LK, Google/CAF; walleye's bootloader is closed, its being LK-derived is NOT KNOWN YET): fastboot reboot = reboot_device(0), reboot-bootloader = reboot_device(FASTBOOT_MODE), continue is a separate command that boots the active slot without a reset. LK reads the IMEM restart word and zeroes it at once; it reads PON_SOFT_RB_SPARE and scrubs only bits 7:5 (value & 0x1f). Our byte (mode-bootloader 2 at shift 1 = 0x04, bit 2) lies outside bits 7:5, and the PMIC register lives through warm and cold resets; only if walleye's bootloader scrubs it too does it clear.
Ranking: (a) the reason our RESTART2 wrote persists and sends the next reset(s) back to fastboot - FIRST: it explains why the phone was in fastboot after a reset that Android's own reboot would not have sent there, and an LK-style scrub that is one boot late would also explain why a second `fastboot reboot` (after-stop way back) worked; against it: the after-stop count of resets is unmeasured. (b) fastboot reboot re-enters fastboot after a bootloader-requested entry as a bootloader rule - SECOND: indistinguishable from (a) by the rerun alone; the public LK has no such rule. (c) slot or retry state - THIRD, weak: flags equal before and after, b successful yes, and the stage 1 run flashed nothing; but retry 0 is not excluded for the slot the bootloader would pick. (d) USB or timing - LAST: 240 s of the host seeing a listed phone is not a late enumeration.
The tries, each read-only or RAM-only, one phone excursion each: (1) the same excursion, at fastboot send `fastboot continue` instead of `reboot`: Android within 240 s = the last leg that needs no reset, and if it also fails the cause is not the register (b/c/d). (2) the same excursion with `fastboot reboot` twice, each timed: Android only after the second = (a) with a one-boot-late scrub (and the way back's last leg is 'reboot twice'); both stay in fastboot = (b) or (c). (3) read PMIC PON_SOFT_RB_SPARE (0x88F) from our kernel before the RESTART2 and once more on the next boot of our image after a 'fastboot continue'/reboot (regmap read, no write): the byte 0x04 present after the first reset = (a) proved. Order: (2) first: it needs no new tool and its two timings settle (a) vs (b)/(c); then (1). The reliable last leg known to work today: a second `fastboot reboot` (what the after-stop way back did), untimed.

Sources
: src-aosp-lk-aboot,src-lk-qcom-reboot-mode-scrub,src-wahoo-qpnp-power-on,src-mainline-qcom-pon,src-wahoo-msm-poweroff

About
: walleye-bootimage

Material
: research/walleye-display-s6e3fa5/2026-11-23/


<a id="walleye-stage2-watchdog-prechecks-read"></a>
### walleye-stage2-watchdog-prechecks-read

Date
: 2026-11-23

Brief
: Kernel petting is in e337de53 and probes; stock bark 11 s/pet 10 s are Android's, not the bootloader's; our init's first echo ends kernel petting; bootloader timing NOT KNOWN YET

Question
: Does e337de53 already carry kernel-level watchdog petting and why would it fail; what does the stock fdt say; what does qcom-wdt do; how long does a kernel have?

Found
: ANSWERS, each with its line. Host reads and read-only adb on Android b (slot _b, root, nothing written, no reboot).
(1) THE MECHANISM IS IN e337de53. Image sha256 e337de53... (the file in walleye-build, unpacked with unpack_bootimg): its appended dtb has /soc@0/watchdog@17817000 compatible qcom,kpss-wdt, reg <0x17817000 0x1000>, interrupts <0 3 1> (SPI 3, edge rising), a clock (the sleep clock), no status, no timeout-sec. Kernel config of the tree's three .config (walleye-build out-ufsm, out-ufsm2, out-ufsroot; which one built e337de53 is not recorded: NOT KNOWN YET, all three agree): CONFIG_WATCHDOG=y, WATCHDOG_CORE=y, WATCHDOG_HANDLE_BOOT_ENABLED=y (line 5055), # WATCHDOG_NOWAYOUT is not set, QCOM_WDT=y (line 5095). The header's command line has panic=5 (not 'no panic=': the row walleye-stage2-slot-a-no-ssh-causes-ranked says none). PROBES: no kernel log of e337de53 or of stage 1 (c3373d71) keeps a qcom_wdt line (read.txt of rootfs-boots-20261123-101812 has none: NOT KNOWN YET for this image); the SAME driver and node in the earlier images do probe: /proc/iomem '17817000-17817fff : 17817000.watchdog', /proc/interrupts 'GICv3 35 Edge wdt_bark' (SPI 3 = 35), /sys/class/watchdog/watchdog0 under soc@0/17817000.watchdog (research walleye-first-fastboot-boot c2/z6/z7 read/, lab-material pmic-wdt-20261118-*), and dmesg 'watchdog: watchdog0: watchdog did not stop!' every time our init's echo closed /dev/watchdog. So address, clock, irq and probe are not the failure on this phone's kernel line.
(2) STOCK vs OURS (read on Android b: /proc/device-tree/soc/qcom,wdt@17817000): compatible qcom,msm-watchdog; reg 0x17817000 size 0x1000 (reg-names wdt-base); interrupts <0 3 0> and <0 4 0> (SPI 3 bark, SPI 4 bite, type 0); qcom,bark-time 0x2af8 = 11000 ms; qcom,pet-time 0x2710 = 10000 ms; qcom,ipi-ping, qcom,wakeup-enable, qcom,scandump-size 0x40000. Ours: same base and size, bark SPI 3 (type edge rising, as mainline dts use), no bite irq (mainline does not use it), clock sleep_clk (32768 Hz; the stock driver assumes WDT_HZ 32765). Register layout: the stock driver's WDT0_RST 0x04, EN 0x08, STS 0x0C, BARK 0x10, BITE 0x14 equals qcom,kpss-wdt's table (qcom-wdt.c reg_offset_data_kpss), so the layout fits. No field differs in a way that stops the driver.
THE 11 s / 10 s ARE THE ANDROID KERNEL'S, NOT THE BOOTLOADER'S: watchdog_v2.c lines 743-744 write bark = bark-time and bite = bark-time + 3 s (so 11 s / 14 s) and its probe enables the watchdog; the node says nothing about what the bootloader armed. What the bootloader arms before the kernel, and for how long, is NOT KNOWN YET: it is closed, the bootloader_log node (htc,bldr_log, two regions 0xa1300000 and 0xa1380000, 0x80000 each) cannot be read from Android (no /dev/mem). A SECURE watchdog (SCM_SVC_SEC_WDOG_DIS in the stock driver) is a second, separate possibility: the non-secure kernel cannot pet it.
(3) WHAT OUR KERNEL DOES (qcom-wdt.c v7.3, topic tree lines 346-351, 80-86; the coordinator's 331-333 are another tree): probe: if WDT_EN bit 0 is set (is_running) it calls qcom_wdt_start, which stops, then reprograms with OURS: timeout 30 s (min(max,30)), pretimeout 1 (an irq was found): bark 29 s, bite 30 s, enables, and sets WDOG_HW_RUNNING - so a bootloader-armed apps watchdog gets a FRESH 30 s from probe, its own times replaced. The core then pets it (watchdog_need_worker: timeout set, not active, hw_running) every timeout/2 = 15 s, UNTIL userspace activates it. A watchdog the bootloader did NOT arm is never armed by the driver: only an open of /dev/watchdog starts it (then bark 29 / bite 30).
THE CATCH, in our own init (build/initramfs-root/init, lines 57-65 and 86-93): its wait loop does 'echo > /dev/watchdog' once per second ONLY while the partition is not yet found. That echo OPENS the device (the core: activated, kernel petting stops), writes (a pet) and closes without the magic 'V' ('did not stop!' in every log): the watchdog is now ACTIVE with no worker, so from then on only userspace pets, 30 s per pet. The loop ends when the partition appears; between that last pet and the handover the init mounts, builds the overlay and copies busybox; after switch_root the pets exist only if someone runs /.walleye/hold.sh (the init writes it; nothing in this repository starts it: NOT KNOWN YET whether system_a's own init does). If the partition is found on the loop's first pass the device is never opened and the kernel's own petting goes on (if the watchdog was armed at all). So: kernel-level petting is IN the image, probes, and is not what fails; it covers only the time before our init first opens /dev/watchdog; after that the pets are the init's loop and hold.sh.
HOW LONG A KERNEL HAS: if the bootloader armed the apps watchdog: a fresh 30 s from our probe (a few seconds into the boot), then 15 s pets until the init's first open, then 30 s from each of the init's echoes. If it armed none, none until our open, then 30 s from the open. The bootloader's own time before probe: NOT KNOWN YET (bounded below by stage 1, where the same kernel needed about 12 s from the jump to ssh without a bite, and by the T3 try, where an unpetted node left the phone alive-class for 42 s).
FIT WITH RUN 7: reset about 35-40 s after the jump with bootreason watchdog fits 'init opened /dev/watchdog in its wait loop, the last pet came when the partition was found, nothing pets after the handover, bite 30 s later' as well as a bootloader-armed watchdog; but it does not explain why no gadget appeared in the first 12-20 s (stage 1's came about 12 s after the start). Either the kernel did not run well before the bite, or the Alpine side started late; the host's USB log is the only clock and shows nothing. The two readings tell apart by one thing the debug image already queued (walleye-stage2-debug-image) would show: a gadget and a pet from the first instant.
RECOMMENDED FOR THE TRY: (i) make the first act of our init a pet loop that lives for the whole boot (a background petter started before the partition wait, not inside it) and keep kernel petting out of the picture: after our first open the kernel stops anyway; (ii) leave the dts and the driver alone: they probe; (iii) do not assume bark 11 s: it is Android's. The first unknown it does not settle: whether a SECURE or PMIC watchdog bites instead (bootreason 'watchdog' is the bootloader's name for any of them): the debug image's USB and pet loop settle it by its own first 40 s.

Sources
: src-wahoo-msm-watchdog-v2,src-linux-qcom-wdt-v73,src-wahoo-msm8998-dtsi

About
: walleye-bootimage
