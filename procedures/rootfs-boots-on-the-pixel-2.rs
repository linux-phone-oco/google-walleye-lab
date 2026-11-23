#!/usr/bin/env rust-script
//! The read-only UFS-on boot of our kernel on the Alpine root as ONE command (item walleye-install-boot-procedure; run by item
//! walleye-install-rootfs-boots-try; decisions 2026-11-20-walleye-install-stage-1-go, 2026-11-19-walleye-attended-tries-one-command):
//! the UFS-root image booted once from fastboot, from memory; its init mounts the card's system partition of the slot Android is NOT on
//! read-only under a tmpfs overlay; the lab logs in over usb-network by ssh with its declared key and reads the mounts and the kernel log
//! (docs step rootfs-boots). Before the boot: adb's slot and fastboot's slot agree and the image's root is the other slot's system
//! (a fastboot boot from memory gives the kernel no androidboot.slot_suffix: this command puts the slot it read on the command line of THIS run's image, made by lpoco-bringup image swap and checked equal to the declared one in every byte but the command line; the init fails closed without it), and the author's go is
//! recorded on the item the manifest names for this image's sha256. After: the GPT of every disk, the root partition and the boot chain
//! are read from Android and equal to before, and the slots' flags too. NOTHING is written to the card; the way back to Android runs on any stop.
//! The UFS-off gate refuses this image by design and is neither run, weakened nor bypassed.
//! When a check after the boot fails (check 17 and later) and the phone is still on the image, the way back FIRST reads the evidence over ssh,
//! read-only, into this run's log and its folder's evidence.txt (the kernel log's tail and the init's lines, the command line, mounts, processes,
//! the root and the new root, the init's logs, the card's PARTNAMEs, df; item walleye-rootfs-boots-run-2-diagnose), THEN sends the phone away; after
//! it the card, the boot chain and the slots' flags are measured against what was read before when the run stopped before its own measures.
//! The way back goes through the bootloader first (item walleye-way-back-reboot-bootloader-first): `reboot bootloader` over ssh, fastboot lists the phone within 60 s,
//! its current-slot is read and said against the slot at the start, then `fastboot reboot`; when fastboot does not list the phone the log says "bootloader not reached" and the forced reboot runs.
//!   procedures/rootfs-boots-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml [procedures.rootfs-boots-on-the-pixel-2], [images.ufs-root])
//!
//! ```cargo
//! [dependencies]
//! lpoco-attended = { git = "https://github.com/linux-phone-oco/lab-tools", tag = "lpoco-attended-v0.41.0" }
//! walleye-tries = { path = "../tools/walleye-tries" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use lpoco_attended::{prelude::*, steps};
use walleye_tries::{rootfs_boot::RootfsBoot, settings::{Settings, ROOTFS_BOOT_PROCEDURE}, watch};

fn main() {
    let _run = procedure(ROOTFS_BOOT_PROCEDURE, "google-walleye");
    let root = RootfsBoot::new(Settings::for_rootfs_boot()); // every key from device.toml, read here
    let args: Vec<String> = std::env::args().skip(1).collect();
    if watch::watches(&args) {
        watch::watch_the_try(root.way_back()); // whatever ends a real run, the way back runs after
    }

    check("the battery is good enough", root.battery_is_good());
    check("the image is the declared one and names one root", root.the_image_is_the_declared_one());
    check("the stored boot chain is whole and the ssh key is private", root.the_stored_set_and_the_key_are_good());
    step("say what to expect", root.author_is_told());
    step("the phone is on Android, adbd as root", root.the_phone_is_on_android_as_root());
    check("adb reads the slot Android runs from", root.the_slot_is_read_by_adb());
    check("the card is read from Android: its partition tables and the root partition", root.the_card_is_read_before());
    step("take the phone to fastboot", root.phone_is_in_fastboot());
    check("fastboot agrees on the slot and the image's root is the other one", root.the_guard_holds());
    step("make this run's image: the declared one with the slot read on its command line", root.the_run_image_is_made());
    check("the slots' flags are read", root.the_slot_flags_are_read());
    check("the author's go is recorded on the item", root.author_has_said_go());
    step("boot the UFS-root image from memory, nothing flashed", root.boot_the_root_image());
    run_step(steps::wait_for_gadget(30, 5));
    step("log in over usb-network by ssh and read the phone", root.the_login_works_and_the_phone_is_read());
    check("it is root's login on the image, the network and the services are up", root.the_phone_is_the_image_we_booted());
    check("the init saw the slot adb saw and mounted the named partition", root.the_init_saw_the_slot_adb_saw());
    check("the card is mounted read-only under the overlay", root.the_root_is_mounted_read_only());
    check("the kernel log has no purge, oops or UFS failure", root.the_kernel_log_is_clean());
    step("send the phone back: to its bootloader first, the forced reboot if that is not reached", root.the_phone_is_sent_back());
    step("the phone is on Android (the way back, if it is not)", root.way_back());
    check("the phone is back on Android", root.android_is_back());
    step("Android answers as root again", root.the_phone_is_on_android_as_root());
    check("the partition tables and the root partition are as before", root.the_card_is_as_before());
    check("the boot chain read again equals the stored set", root.the_boot_chain_is_as_stored());
    check("both slots' flags are as before", root.the_slot_flags_are_as_before());
}
