The audit of the 29 steps of procedures/install-boot-on-the-pixel-2.rs (and the 17 of its rehearsal) for the state of a repeat, item walleye-stage2-check-16-and-repeat-audit, 2026-11-23.

The state of the phone now (from the keeper's rerun 3 and run 1's log): boot_a = e337de53 (the declared image), slot a's flags what run 1's flash left (retry 1, successful no), b untouched and successful, Android on b, the stock backup boot_a.img on file and hashing to its record 2a5cd3b2. A first run assumed: boot_a stock, slot a successful yes retry 0, no earlier flash.

"walked" = run by a test in flash_boot.rs (`every_step_that_reads_or_writes_the_slot_passes_against_the_phone_after_rerun_3`, stand-in adb and fastboot with the state above); "pure" = its decision is a function with its own tests.

| # | step | assumes a first run? |
|---|---|---|
| 1 | battery | no (the phone's charge) |
| 2 | image is the declared one | no: sizes the image against the backup file's size, which is on file in both |
| 3 | init accepts the slot | no (the image) |
| 4 | stored chain, key, reboot tool | no |
| 5 | say what to expect | no |
| 6 | go recorded | no; a go is per image sha and time, the coordinator extends it |
| 7 | phone on Android as root | no |
| 8 | adb reads the slot | no: refuses only a SWITCHED marker without RESTORED; run 3 never switched; walked |
| 9 | boot partition read now is the backup, or the image with the backup on file | YES, amended in PR 841; walked with boot_a = image |
| 10 | GPT read whole | no: whole-GPT is shape and CRCs, not bit values; the GPT checker holds no bit value (test `the_gpt_checker_takes_the_slot_bits_of_a_repeat...`) |
| 11 | phone to fastboot | no |
| 12 | fastboot agrees on the slot | no; walked |
| 13 | flags read before | no: kept as read, whatever they are; walked |
| 14 | the flash | no: the same bytes flashed again is one flash; walked |
| 15 | phone on Android | no |
| 16 | reads back as the image, no other partition changed | YES (third stop): demanded boot_a to change. Amended: boot_a reads back as the image (checked first) and no partition but boot_a may differ. Walked, and 5 tests |
| 17 | phone to fastboot | no |
| 18 | flash reset only the flashed slot's flags | YES, amended in PR 836 (any positive retry count, successful no); in the repeat the flags before are already retry 1 / no, so a bootloader that resets or leaves them passes, one that changes any other flag fails; walked |
| 19 | set_active | no: refuses unless parts-readback.txt exists and fastboot's current slot is Android's; walked |
| 20 | flags are the switch and nothing else | no: names slot a's own flags as the ones that may change (any before value), every other flag as before; walked with a's retry 1 / no before |
| 21 | new slot boots, ssh login, GPT read | first-run expectation by nature (not a repeat assumption): listed below |
| 22 | GPT changed only by the switch | pure; value-agnostic (bits, type-GUID swaps, CRCs); tested from a repeat-state GPT |
| 23 | the way back to the bootloader and set_active of Android's slot | no: a phone already on Android's slot is not written; walked from the fastboot side |
| 24 | flags back: Android's slot current, not unbootable, successful or retry above 0 | no; walked |
| 25 | phone on Android | no |
| 26 | phone back on Android | no |
| 27 | partitions as the flash left them (readback = after) | no: both read after the flash, not against "before" |
| 28 | GPT read again changed only by the switch | pure, as 22 |
| 29 | boot chain equals the stored set | no: stored set, not the flash |

The steps after set_active, with what each expects (written for the first run, not walked here because they need a booted phone): 21 expects ssh as root on our image within device_wait, kernel cmdline `androidboot.slot_suffix=_a`, PID 1 the root's /sbin/init, the card mounted read-only; 22 expects the GPT read from the new slot to differ from the one read on Android only by the switch (bits, A/B GUID swap, CRCs); 23 expects either the reboot tool pushed and RESTART2 reaching the bootloader, or the hand step; 24 expects slot b current, not unbootable, successful yes or retry above 0 - what the bootloader leaves after `set_active b` is not known yet (the stand-in models retry 7, successful no, which passes; successful yes passes too); 27 and 28 expect Android back with only the flash's partition and the switch's bits different. None of these reads a value that a repeat changes; what in them could be wrong is the kernel or the bootloader, which is what the run measures.

Rehearsal (17 steps): the rehearsal writes nothing, so every step but 9 is state-independent; its step 9 is the same function as the command's, passed on rehearsal 3 with boot_a = image.
