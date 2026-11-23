Stage 2 (procedures/install-boot-on-the-pixel-2.rs, 29 steps; rehearsal 17) judged by what safety rests on, and by how fastboot, update_engine and boot_control write a slot. Item walleye-stage2-checks-should-they-exist, 2026-11-23, walleye-research. No phone touched. Questions (1) and (2); (3) is the last section.

## The invariants safety rests on (the item's four)
- I1 boot_b and every other partition untouched (the target boot_X is the one write).
- I2 the GPT changes only in the slot bits of entries, the A/B type GUIDs and the CRCs they imply.
- I3 the written image is verified by read-back.
- I4 a way back is proven (backup on file and hashing to its record; `set_active` Android's slot) and the bootloader's retry counter is the second path.
A check protects something only if its failure means one of these is, or will be, false. A check that asserts what a slot HELD or WHICH FLAGS IT HAD before an overwrite protects none of them: the overwrite destroys that state whatever it was.

## How the working system does it (read for behaviour only; AOSP android11-release, Apache-2.0)
- fastboot `flash` (system/core/fastboot/fastboot.cpp): `do_flash` l.1232-1244 loads the file, resizes only a logical partition, calls `flash_buf` l.1044-1085, which for boot/boot_a/boot_b copies the AVB footer (l.1047) and sends the buffer with `FlashPartition`. It reads nothing of the target before, asserts nothing about its old content, and does not read back: the protocol has no read. `set_active` l.1246-1258 only sends SetActive; its comment says it "clears slot-unbootable".
- update_engine writes the INACTIVE slot whatever it held: `DownloadAction` marks the target slot unbootable first, "Marking new slot as unbootable" (payload_consumer/download_action.cc l.199-200 -> `BootControlAndroid::MarkSlotUnbootable`, boot_control_android.cc l.107-120); the payload is written; `FilesystemVerifierAction` hashes the TARGET and compares (payload_consumer/filesystem_verifier_action.cc l.92-100 kVerifySourceHash / kVerifyTargetHash; source hashes are of the CURRENT slot, per delta operation); only if postinstall succeeded `SetActiveBootSlot(target)` (postinstall_runner_action.cc l.343-347, `BootControlAndroid::SetActiveBootSlot` l.124-137). A cancelled update leaves the target slot unbootable and never switches.
- boot_control (hardware/interfaces/boot/1.0/default/BootControl.cpp l.40-85): thin HIDL over the vendor module: `markBootSuccessful`, `setActiveBootSlot`, `setSlotAsUnbootable`, `isSlotBootable`, `isSlotMarkedSuccessful`. No flag value is asserted anywhere in update_engine except after a switch back: `UPDATED_NEED_REBOOT` cancel path re-asserts the current slot active and marked successful (update_attempter_android.cc l.362-372), because "marking it as active may reset the successful bit" (l.368-369).
- So the system judges a slot by (a) verification of what was written, (b) the boot counter and unbootable/successful bits, never by its prior contents; and every flag state of the TARGET slot is legal before the switch: update_engine itself puts it in "unbootable" first.

## Table (keep / relax / DROP; why; what it protects)
| # | step | protects | verdict |
|---|---|---|---|
| 1 | battery | power loss mid-flash (I3, I4) | KEEP |
| 2 | image declared: sha, NOT-TO-BOOT name, size, AVB footer | the right bytes written (I3); host only | KEEP |
| 3 | image's init accepts the slot | phone time (a boot to a refusal), not safety; host only, free | KEEP host-side, not a gate on the phone |
| 4 | stored chain, key, reboot tool | I4 (what we put back) | KEEP |
| 5 | say what to expect | witness, no state | KEEP |
| 6 | go recorded | policy, not state | KEEP |
| 7 | on Android, adbd root | precondition of reads | KEEP |
| 8 | adb reads slot; clears out folder; REFUSES on host marker SWITCHED without RESTORED | the slot read protects I1 (target = the other slot); the marker is the host's memory of an intention, the phone's flags and GPT are the truth, read at 10/13 | KEEP the read; DROP the marker refusal (state it as a note; the way back already acts on the phone's real state). Its stop text itself was not read: not known yet |
| 9 | target boot read now == backup, or declared image with backup on file | I4 only through "backup on file and hashing to its record"; what boot_a held before protects nothing (overwritten either way; update_engine and fastboot ask nothing) | RELAX to: backup file on file and hashes to its record; the live boot_X hash is recorded (and its 32 MiB kept by adb pull when it is neither), never asserted |
| 10 | GPT of every disk read whole, kept | I2 baseline | KEEP |
| 11 | phone to fastboot | step | KEEP |
| 12 | fastboot slot == adb slot; target = other slot's boot via allow-list; image's root is that slot's system | I1 (never Android's slot, never boot_b): the one pre-check that protects the main invariant | KEEP, test hardest |
| 13 | slot flags read before | baseline for 18/20/24 | KEEP as a record, no assertion |
| 14 | the one fastboot flash | the write | KEEP |
| 15 | back on Android, root | read-back needs adb | KEEP |
| 16 | target reads back as the image; no other boot/dtbo/vbmeta changed | I3 and I1; the only real verification | KEEP. Never require that boot_X changed (rerun 3 stopped on that; the flash of equal bytes changes nothing, fastboot does not care) |
| 17 | phone to fastboot | step | KEEP |
| 18 | flash changed only flashed slot's successful/retry (and unbootable yes to no) | for the TARGET slot: nothing (bootloader may reset any of its flags, update_engine itself sets unbootable; three stops came from asserting a value); for ANDROID'S slot and current-slot: I4, and that part is real | DROP the assertion on the flashed slot's flags; KEEP "Android's slot's flags and current-slot equal their before" |
| 19 | the bootloader's set_active | the switch | KEEP |
| 20 | flags are the switch and nothing else | current-slot == target, target not unbootable, retry > 0 is the goal itself; the claim "every other flag is what it was" rests on one observed run, not on a source | RELAX to: current slot is target, target not unbootable, retry > 0; Android's slot's bits recorded and compared at 24 only |
| 21 | new slot boots, ssh logs in; GPT read there | the goal; a failure falls to the way back and to the counter (I4 second path) | KEEP |
| 22 | GPT changed only by the switch | I2 | KEEP |
| 23 | the way back: set_active Android's slot | I4 | KEEP |
| 24 | Android's slot current, bootable | I4 | KEEP (current, not unbootable; successful or retry > 0) |
| 25, 26 | phone on Android, root | read | KEEP |
| 27 | boot/dtbo/vbmeta as the flash left them | I1 after the booted image ran | KEEP |
| 28 | GPT read again changed only by switch | I2 | KEEP |
| 29 | boot chain equals stored set | I1; overlaps 27 (one nature, two routes, practice 32) | RELAX: fold into 27, one read |
Nature found twice (practice 32): the slot flags (13, 18, 20, 24) are GPT attribute bits, which 10/22/28 already read whole and check by `gpt_switch_only`; fastboot's `getvar slot-*` is a second route to the same bits. Whether the flag steps can be dropped in favour of the GPT read alone is for the builder's state walk to settle on the phone's real dumps (not known yet).
Net: of 29 steps, 3 assertions are dropped or cut back (the marker refusal at 8, the live-content assertion at 9, the flashed-slot flag assertion at 18), 2 relaxed (20, 29), the rest kept; none of the dropped ones guards I1-I4. Rehearsal: its "flags read again equal the first" is not an invariant (Android may mark successful on its own): record, do not require; its GPT-same, boot-chain and failed-boot-read steps stay.

## (3) What let each stop through
- 18 (first stop, flags after flash): expectation without a source: written from one observed run's printed flags (a typed stand-in), not from AOSP's spec; the rehearsal writes nothing so could never see a flag a flash changes.
- 9: assumed boot_a held the stock image, the state of a first run; protected nothing (we overwrite and keep a backup). A review asking "what does this assert about state this command did not create?" catches it; the rehearsal ran on a phone that had never been flashed.
- 16: demanded that the flash change boot_a; also first-run state. Repeat of the same image reads equal.
- 18 again (unbootable yes to no after a failed boot, run 5): a state the audit did not walk: the repeat audit walked a repeat of a successful flash, not of a failed boot.
- 8: a host marker file standing in for the phone's state; the walk "refuses only a SWITCHED marker without RESTORED" (stage2-repeat-audit.md) classed it "no first-run assumption" because the audit asked only about the phone. Exact stop not read here.
Common cause: each check asserted the VALUE of a state the command itself, or an earlier run, changes, rather than the invariant. What would have caught them without the phone: a state walk over every state each flag/partition can legally be in (the item walleye-stage2-state-model-walk), and the rule "a check names its source" (item walleye-checks-name-their-sources).

## Not known yet
Which vendor boot_control the Pixel 2 (walleye) ABL uses and exactly what it does to other slot's flags on set_active (AOSP's HAL is an interface; the behaviour is the vendor module's: observed in run 1 only). The exact text of stop 8.
