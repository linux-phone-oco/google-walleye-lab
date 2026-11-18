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
