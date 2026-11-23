#!/usr/bin/env rust-script
//! Install stage 2 as ONE command (item walleye-install-flash-boot-procedure; decisions 2026-11-22-walleye-author-research-then-proceed,
//! 2026-11-19-walleye-attended-tries-one-command; research walleye-install-stage2-android-verdict): our boot image, named by its [images.NAME] row, flashed to
//! the boot partition of the slot Android is NOT on (read by adb and fastboot, equal; one allow-listed name, never the other slot's), read back whole, that slot made
//! active by the bootloader's own `fastboot set_active` (the call Android's OTA makes: a GPT write by the bootloader, with the slot flags and the GPT of every disk
//! read before and after and only the slot bits, the A/B type GUIDs and the CRCs allowed to differ), the phone observed coming up from the card on ssh, and the way
//! back `fastboot set_active <Android's slot>` made and checked in the same run; the phone ends on Android. No bootloader partition, no GPT bytes of ours, no erase,
//! no purge. A keeper runs it only under the coordinator's go (item walleye-install-flash-boot-run), after the stage 1 proof and UFS try 2.
//!   procedures/install-boot-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml [procedures.install-boot-on-the-pixel-2], [images.NAME])
//!
//! ```cargo
//! [dependencies]
//! lpoco-attended = { git = "https://github.com/linux-phone-oco/lab-tools", tag = "lpoco-attended-v0.44.3" }
//! walleye-tries = { path = "../tools/walleye-tries" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use lpoco_attended::prelude::*;
use walleye_tries::{flash_boot::FlashBoot, settings::{Settings, FLASH_BOOT_PROCEDURE}, watch};

fn main() {
    let _run = procedure(FLASH_BOOT_PROCEDURE, "google-walleye");
    let boot = FlashBoot::new(Settings::for_flash_boot()); // every key from device.toml, read here
    let args: Vec<String> = std::env::args().skip(1).collect();
    if watch::watches(&args) {
        watch::watch_the_try(boot.way_back()); // whatever ends a real run, the way back runs after
    }

    check("the battery is good enough", boot.battery_is_good());
    check("the image is the declared one, a whole boot partition, its AVB footer verifies", boot.the_image_is_the_declared_one());
    check("the image's init accepts the slot it is booted from", boot.the_init_accepts_the_slot_it_is_booted_from());
    check("the stored boot chain is whole, the ssh key is private and the reboot tool is the declared one", boot.the_stored_set_and_the_key_are_good());
    step("say what to expect", boot.author_is_told());
    check("the coordinator's go is recorded on the item for this image", boot.go_is_open());
    step("the phone is on Android, adbd as root", boot.the_phone_is_on_android_as_root());
    check("adb reads the slot Android runs from", boot.the_slot_is_read_by_adb());
    check("the boot partition to be flashed, read now, is recorded (its backup, the declared image or other content) and the stock backup is on file and hashes to its record; the boot, dtbo and vbmeta partitions are hashed", boot.the_slot_partitions_are_read_before());
    check("the GPT of every disk is read whole and kept", boot.the_gpt_is_read_before());
    step("take the phone to fastboot", boot.phone_is_in_fastboot());
    check("fastboot agrees on the slot and the flash target is the other slot's boot partition", boot.the_guard_holds());
    check("the slot flags are read before, and the slot Android runs from is successful and bootable (the way back)", boot.the_slot_flags_are_read("slots-before.txt"));
    step("flash the boot partition of the slot Android is not on: one fastboot flash", boot.the_boot_image_is_flashed());
    step("the phone is on Android, adbd as root (Android's slot is still the active one)", boot.the_phone_is_on_android_as_root());
    check("the partition reads back as the image, and no other boot, dtbo or vbmeta partition changed", boot.the_partition_reads_back_as_the_image());
    step("take the phone to fastboot", boot.phone_is_in_fastboot());
    check("the flash left Android's slot, the current slot and every other flag as they were (the flashed slot's own flags are the bootloader's)", boot.the_flash_changed_no_flag());
    step("make the flashed slot active: the bootloader's own set_active", boot.the_slot_is_made_active());
    check("the current slot is the new one, not unbootable, with tries left", boot.the_flags_are_the_switch_and_nothing_else());
    step("the new slot boots from the card and the lab logs in by ssh; its GPT is read", boot.the_new_slot_boots_and_the_lab_logs_in());
    check("the GPT changed only by the slot switch (bits, A/B type GUIDs, CRCs)", boot.the_gpt_changed_only_by_the_switch("gpt-new-slot.txt"));
    step("the way back: the phone to the bootloader, set_active Android's slot", boot.the_switch_is_made_back());
    check("Android's slot is current again and bootable", boot.the_flags_are_back());
    step("the phone is on Android, adbd as root", boot.the_phone_is_on_android_as_root());
    check("the phone is back on Android", boot.android_is_back());
    check("the boot, dtbo and vbmeta partitions are as the flash left them; the GPT is read again", boot.the_partitions_are_as_the_flash_left_them());
    check("the GPT read again changed only by the slot switch", boot.the_gpt_changed_only_by_the_switch("gpt-after.txt"));
    check("the boot chain read again equals the stored set", boot.the_boot_chain_is_as_stored());
}
