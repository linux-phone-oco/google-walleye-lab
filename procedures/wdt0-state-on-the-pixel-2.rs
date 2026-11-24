#!/usr/bin/env rust-script
//! The WDT0 try as ONE command (item walleye-wdt0-try-procedure; image walleye-wdt0-read-image): the wdt0 image - the UFS-root image whose init, with the
//! word walleye.wdt0, reads the apps watchdog's EN, STS, BARK_TIME and BITE_TIME with devmem as its first act, read only - booted once from fastboot,
//! from memory, on the slot Android is NOT on; the lab logs in by ssh with its declared key, reads /run/walleye/wdt0 and the kernel log and judges that
//! all four registers were read (or names the one that was not and devmem's words); the readings are kept in the run's folder (wdt0.txt) for the research
//! row. Every other step is the root boot's (procedures/rootfs-boots-on-the-pixel-2.rs): the battery, the slot read two ways, the run image with the slot
//! on its command line, the author's go that names the image's sha256, the card, the boot chain and the slots' flags read before and after and EQUAL,
//! the way back through the bootloader. NOTHING is written to the card or to any register; the UFS-off gate is neither run nor weakened.
//!   procedures/wdt0-state-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml [procedures.wdt0-state-on-the-pixel-2], its image)
//!
//! ```cargo
//! [dependencies]
//! walleye-tries = { path = "../tools/walleye-tries" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use walleye_tries::lpoco_attended::{prelude::*, steps};
use walleye_tries::{rootfs_boot::RootfsBoot, settings::{Settings, WDT0_PROCEDURE}, watch};

const SOURCE: &str = "research:walleye-android-watchdog-read-fix"; // the registers and their offsets; no phone has printed the init's reading yet (the judge is tested on the init's own output, tools/walleye-tries/fixtures/wdt0-init-run-*)
const BOOT: &str = "research:walleye-rootfs-boots-run-5-passed";
const CARD: &str = "research:walleye-stop-stage1-check-17-three-answers";

fn main() {
    let _run = procedure(WDT0_PROCEDURE, "google-walleye");
    let root = RootfsBoot::wdt0(Settings::for_wdt0()); // every key from device.toml, read here
    let args: Vec<String> = std::env::args().skip(1).collect();
    if watch::watches(&args) {
        watch::watch_the_try(root.way_back()); // whatever ends a real run, the way back runs after
    }

    check_from("the battery is good enough", BOOT, root.battery_is_good());
    check_from("the image is the declared one, names one root and carries walleye.wdt0 once", BOOT, root.the_image_is_the_declared_one());
    check_from("the stored boot chain is whole and the ssh key is private", BOOT, root.the_stored_set_and_the_key_are_good());
    step("say what to expect", root.author_is_told());
    step("the phone is on Android, adbd as root", root.the_phone_is_on_android_as_root());
    check_from("adb reads the slot Android runs from", BOOT, root.the_slot_is_read_by_adb());
    check_from("the card is read from Android: its partition tables and the root partition", CARD, root.the_card_is_read_before());
    step("take the phone to fastboot", root.phone_is_in_fastboot());
    check_from("fastboot agrees on the slot and the image's root is the other one", BOOT, root.the_guard_holds());
    step("make this run's image: the declared one with the slot read on its command line", root.the_run_image_is_made());
    check_from("the slots' flags are read", BOOT, root.the_slot_flags_are_read());
    check_from("the author's go is recorded on the item", BOOT, root.author_has_said_go());
    step("boot the wdt0 image from memory, nothing flashed", root.boot_the_root_image());
    run_step(steps::wait_for_gadget(30, 5));
    step("log in over usb-network by ssh and read the phone and the watchdog's file", root.the_login_works_and_the_phone_is_read());
    check_from("it is root's login on the image, the network and the services are up", BOOT, root.the_phone_is_the_image_we_booted());
    check_from("the init read all four registers of WDT0, and the file and the kernel log say the same", SOURCE, root.the_wdt0_registers_are_read());
    check_from("the init saw the slot adb saw and mounted the named partition", BOOT, root.the_init_saw_the_slot_adb_saw());
    check_from("the card is mounted read-only under the overlay", BOOT, root.the_root_is_mounted_read_only());
    check_from("the kernel log has no purge, oops or UFS failure", BOOT, root.the_kernel_log_is_clean());
    step("send the phone back: to its bootloader first, the forced reboot if that is not reached", root.the_phone_is_sent_back());
    step("the phone is on Android (the way back, if it is not)", root.way_back());
    check_from("the phone is back on Android", BOOT, root.android_is_back());
    step("Android answers as root again", root.the_phone_is_on_android_as_root());
    check_from("the partition tables and the root partition are as before", CARD, root.the_card_is_as_before());
    check_from("the boot chain read again equals the stored set", BOOT, root.the_boot_chain_is_as_stored());
    check_from("both slots' flags are as before", BOOT, root.the_slot_flags_are_as_before());
}
