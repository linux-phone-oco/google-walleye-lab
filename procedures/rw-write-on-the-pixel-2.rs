#!/usr/bin/env rust-script
//! The rw write on the Pixel 2 as ONE command (item walleye-install-rw-write-procedure; image walleye-rw-write-image, plumbing walleye-rw-write-plumbing in tools/walleye-tries/src/rw_write.rs):
//! the rw-write image booted once from fastboot, from memory, with its root - the system partition of the slot Android is NOT on - mounted READ-WRITE (ext4 rw,nodiscard, no overlay; the
//! init's word walleye.rw); the lab logs in over usb-network by ssh, writes ONE bounded file of zeros into that root, syncs, reads it back, checks the filesystem without repairing it,
//! removes the file, checks the filesystem (read-only, the answer kept), reads the kernel log and sends the phone back. The gates before the boot are the root boot's (battery, the image's sha256 and its command line - it must carry walleye.rw once -, the stored boot chain
//! and the key, adb's and fastboot's slot agreeing and the image's root being the other slot's system, the slots' flags) and the author's go on the item the manifest names, which must name THIS image's FULL sha256
//! (a short one, or none, is "no go" and nothing is booted). After: every disk's partition table and the boot chain read from Android and equal to before, the slots' flags too, and the ROOT PARTITION CHANGED (a write that
//! did not reach the card is a failure). The way back to Android runs on any stop, as the root boot's does. NOT KNOWN YET (the run captures them: mounts in read.txt, fsck.txt): the rw lines of /proc/mounts and what fsck.ext4 -n
//! prints on a mounted root. The UFS-off gate refuses this image by design and is neither run, weakened nor bypassed. Never: bootloader partitions, the GPT, erase, purge, fstrim, mkfs.
//!   procedures/rw-write-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml [procedures.rw-write-on-the-pixel-2], [images.ufs-root-booted-slot-production-rw])
//!
//! ```cargo
//! [dependencies]
//! walleye-tries = { path = "../tools/walleye-tries" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use walleye_tries::lpoco_attended::{prelude::*, steps};
use walleye_tries::{rootfs_boot::RootfsBoot, settings::{Settings, RW_WRITE_PROCEDURE}, watch};

fn main() {
    let _run = procedure(RW_WRITE_PROCEDURE, "google-walleye");
    let rw = RootfsBoot::rw_write(Settings::for_rw_write()); // every key from device.toml, read here
    let args: Vec<String> = std::env::args().skip(1).collect();
    if watch::watches(&args) {
        watch::watch_the_try(rw.way_back()); // whatever ends a real run, the way back runs after
    }

    check_from("the battery is good enough", "research:walleye-rootfs-boots-run-5-passed", rw.battery_is_good());
    check_from("the image is the declared one, names one root and carries walleye.rw once", "research:walleye-rw-write-image-built", rw.the_image_is_the_declared_one());
    check_from("the stored boot chain is whole and the ssh key is private", "research:walleye-rootfs-boots-run-5-passed", rw.the_stored_set_and_the_key_are_good());
    step("say what to expect", rw.author_is_told());
    step("the phone is on Android, adbd as root", rw.the_phone_is_on_android_as_root());
    check_from("adb reads the slot Android runs from", "research:walleye-rootfs-boots-run-5-passed", rw.the_slot_is_read_by_adb());
    check_from("the card is read from Android: its partition tables and the root partition", "research:walleye-stop-stage1-check-17-three-answers", rw.the_card_is_read_before());
    step("take the phone to fastboot", rw.phone_is_in_fastboot());
    check_from("fastboot agrees on the slot and the image's root is the other one", "research:walleye-rootfs-boots-run-5-passed", rw.the_guard_holds());
    step("make this run's image: the declared one with the slot read on its command line", rw.the_run_image_is_made());
    check_from("the slots' flags are read", "research:walleye-rootfs-boots-run-5-passed", rw.the_slot_flags_are_read());
    check_from("the author's go is recorded on the item and names this image's full sha256", "research:walleye-rw-write-plumbing-built", rw.author_has_said_go());
    step("boot the rw-write image from memory, nothing flashed", rw.boot_the_root_image());
    run_step(steps::wait_for_gadget(30, 5));
    step("log in over usb-network by ssh and read the phone", rw.the_login_works_and_the_phone_is_read());
    check_from("it is root's login on the image, the network and the services are up", "research:walleye-rootfs-boots-run-5-passed", rw.the_phone_is_the_image_we_booted());
    check_from("the init saw the slot adb saw and mounted the named partition", "research:walleye-rootfs-boots-run-5-passed", rw.the_init_saw_the_slot_adb_saw());
    check_from("the card is mounted read-write as the root: no overlay, no discard, /.walleye in a tmpfs", "research:walleye-rw-write-image-built", rw.the_root_is_mounted_read_write());
    step("write ONE bounded file of zeros into the root, synced", rw.the_bounded_write_is_made());
    check_from("the file reads back whole: its size, all zeros", "research:walleye-rw-write-plumbing-built", rw.the_file_reads_back_whole());
    step("remove the test file and sync", rw.the_test_file_is_removed());
    step("check the filesystem without repairing it (fsck.ext4 -n)", rw.the_filesystem_is_checked());
    check_from("the kernel log has the rw init's line and no purge, oops, UFS failure or EXT4 error", "research:walleye-rw-write-plumbing-built", rw.the_kernel_log_after_the_write_is_clean());
    step("send the phone back: to its bootloader first, the forced reboot if that is not reached", rw.the_phone_is_sent_back());
    step("the phone is on Android (the way back, if it is not)", rw.way_back());
    check_from("the phone is back on Android", "research:walleye-rootfs-boots-run-5-passed", rw.android_is_back());
    step("Android answers as root again", rw.the_phone_is_on_android_as_root());
    check_from("every partition table is as before and the root partition changed", "research:walleye-rw-write-plumbing-built", rw.the_card_is_as_before());
    check_from("the boot chain read again equals the stored set", "research:walleye-rootfs-boots-run-5-passed", rw.the_boot_chain_is_as_stored());
    check_from("both slots' flags are as before", "research:walleye-rootfs-boots-run-5-passed", rw.the_slot_flags_are_as_before());
}
