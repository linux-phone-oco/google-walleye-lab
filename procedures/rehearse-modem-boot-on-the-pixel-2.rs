#!/usr/bin/env rust-script
//! The rehearsal of the modem boot (item walleye-modem-firmware-push-procedure): every step of procedures/modem-boot-on-the-pixel-2.rs except the boot, the push and the read - the battery, the image's sha256 and
//! the UFS-off gate, the firmware tar's sha256, the stored boot chain, the card read from Android, the trip to the bootloader and the slots' flags - and after it the card, the boot chain and the flags read again
//! and compared. Nothing is booted or written, no go is read. The phone ends on Android.
//!   procedures/rehearse-modem-boot-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml [procedures.rehearse-modem-boot-on-the-pixel-2])
//!
//! ```cargo
//! [dependencies]
//! walleye-tries = { path = "steps" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use walleye_tries::lpoco_attended::prelude::*;
use walleye_tries::modem_boot::ModemBoot;
use walleye_tries::{settings::{Settings, REHEARSE_MODEM_BOOT_PROCEDURE}, watch};


fn main() {
    let _run = procedure(REHEARSE_MODEM_BOOT_PROCEDURE, "google-walleye");
    let b = ModemBoot::new(Settings::for_rehearse_modem_boot()); // every key from device.toml, read here
    let root = b.shared();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if watch::watches(&args) {
        watch::watch_the_try(b.way_back()); // whatever ends a real run, the way back runs after
    }

    check_from("the battery is good enough", "research:walleye-rootfs-boots-run-5-passed", root.battery_is_good());
    check_from("the image is the declared modem image and UFS is off in it", "research:walleye-modem-image-built", b.the_image_is_the_declared_ufs_off_image());
    check_from("the firmware tar is the declared one", "research:walleye-modem-image-built", b.the_firmware_is_the_declared_tar());
    check_from("the ModemManager rootfs tar is the declared one", "research:walleye-mm-rootfs-nix-built", b.the_rootfs_is_the_declared_tar());
    check_from("the stored boot chain is whole", "research:walleye-rootfs-boots-run-5-passed", b.the_stored_set_is_good());
    step("say what to expect", b.rehearsal_is_told());
    step("the phone is on Android, adbd as root", root.the_phone_is_on_android_as_root());
    check_from("adb reads the slot Android runs from", "research:walleye-rootfs-boots-run-5-passed", b.the_slot_is_read_by_adb());
    check_from("the card is read from Android: its partition tables and the root partition", "research:walleye-stop-stage1-check-17-three-answers", root.the_card_is_read_before());
    step("take the phone to fastboot", root.phone_is_in_fastboot());
    check_from("the slots' flags are read", "research:walleye-rootfs-boots-run-5-passed", root.the_slot_flags_are_read());
    step("take the phone out of fastboot: Android, adbd as root", root.the_phone_is_on_android_as_root());
    check_from("the partition tables and the root partition are as before", "research:walleye-stop-stage1-check-17-three-answers", root.the_card_is_as_before());
    check_from("the boot chain read again equals the stored set", "research:walleye-rootfs-boots-run-5-passed", root.the_boot_chain_is_as_stored());
    check_from("both slots' flags are as before", "research:walleye-rootfs-boots-run-5-passed", root.the_slot_flags_are_as_before());
}
