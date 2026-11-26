#!/usr/bin/env rust-script
//! The rehearsal of install stage 2 (item walleye-install-flash-boot-procedure): every step of procedures/install-boot-on-the-pixel-2.rs except the flash, the
//! set_active, the boot of the new slot and the switch back - the battery, the image, the init gate, the stored boot chain, adb as root, the slot read two ways, the flash
//! target built and its command shown (not run), the boot partition read and compared with its backup, the slot flags read twice and equal, the GPT of every disk read twice,
//! equal and whole (the checker runs on them), the boot chain as stored. NOTHING is written: no flash, no set_active (whether the bootloader takes a set_active to the slot
//! already active as a no-change write is not known yet, so it is not tried). It needs no go; the phone ends on Android.
//!   procedures/rehearse-install-boot-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml [procedures.rehearse-install-boot-on-the-pixel-2], [images.NAME])
//!
//! ```cargo
//! [dependencies]
//! walleye-tries = { path = "steps" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use walleye_tries::lpoco_attended::prelude::*;
use walleye_tries::{flash_boot::FlashBoot, settings::{Settings, REHEARSE_FLASH_BOOT_PROCEDURE}, watch};

fn main() {
    let _run = procedure(REHEARSE_FLASH_BOOT_PROCEDURE, "google-walleye");
    let boot = FlashBoot::new(Settings::for_rehearse_flash_boot()); // every key from device.toml, read here
    let args: Vec<String> = std::env::args().skip(1).collect();
    if watch::watches(&args) {
        watch::watch_the_try(boot.rehearsal_way_back()); // whatever ends a real run, the way back runs after
    }

    check_from("the battery is good enough", "research:walleye-install-stage2-android-verdict", boot.battery_is_good());
    check_from("the image is the declared one, a whole boot partition, its AVB footer verifies", "research:walleye-install-flash-boot-procedure-built", boot.the_image_is_the_declared_one());
    check_from("the image's init accepts the slot it is booted from", "research:walleye-install-flash-boot-procedure-built", boot.the_init_accepts_the_slot_it_is_booted_from());
    check_from("the stored boot chain is whole, the ssh key is private and the reboot tool is the declared one", "research:walleye-install-boot-rehearsal-passed", boot.the_stored_set_and_the_key_are_good());
    step("say what to expect", boot.rehearsal_is_told());
    step("the phone is on Android, adbd as root", boot.the_phone_is_on_android_as_root());
    check_from("adb reads the slot Android runs from", "research:walleye-install-boot-rehearsal-passed", boot.the_slot_is_read_by_adb());
    check_from("the boot partition to be flashed, read now, is recorded (its backup, the declared image or other content) and the stock backup is on file and hashes to its record; the boot, dtbo and vbmeta partitions are hashed", "research:walleye-stop-stage2-checks-9-and-16-three-answers", boot.the_slot_partitions_are_read_before());
    check_from("the GPT of every disk is read whole and kept", "research:walleye-stop-stage2-check-18-three-answers", boot.the_gpt_is_read_before());
    step("take the phone to fastboot", boot.phone_is_in_fastboot());
    check_from("fastboot agrees on the slot and the flash target is the other slot's boot partition (its command is shown, not run)", "research:walleye-install-boot-rehearsal-passed", boot.the_guard_holds());
    check_from("the slot flags are read before, and the slot Android runs from is successful and bootable (the way back)", "research:walleye-stop-stage2-checks-9-and-16-three-answers", boot.the_slot_flags_are_read("slots-before.txt"));
    check_from("the slot flags read again are kept beside the first (a difference is said, not refused)", "research:walleye-stop-stage2-checks-9-and-16-three-answers", boot.the_slot_flags_are_the_same());
    step("the phone is on Android, adbd as root", boot.the_phone_is_on_android_as_root());
    check_from("the phone is back on Android", "research:walleye-install-boot-rehearsal-passed", boot.android_is_back());
    check_from("the GPT read again is the same, whole, and passes the switch checker unchanged", "research:walleye-stop-stage2-check-18-three-answers", boot.the_gpt_reads_the_same_again());
    check_from("the read of a failed boot (the pstore first, the command line, the boot reason, the slot) works on Android as it is: read only, nothing sent", "research:walleye-stop-pstore-reads-runs-4-and-7-three-answers", boot.the_failed_boot_read_works_on_android());
    check_from("the boot chain read again equals the stored set", "research:walleye-install-boot-rehearsal-passed", boot.the_boot_chain_is_as_stored());
}
