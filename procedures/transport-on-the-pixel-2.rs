#!/usr/bin/env rust-script
//! The declared transports on the real Pixel 2 as ONE command (item walleye-transport-procedure, for the run walleye-transport-real-phone-run; decision
//! 2026-11-26-walleye-android-connection-user-root): the phone is on Android, then the library (it asks adbd for root itself, lab-tools 0.53.5): `read_slot(phone_on("android"))` - the first live use of
//! `[connections.android]` - then adb's own serial against the one `[fastboot]` declares, `adb reboot bootloader` honoured with fastboot listing the phone on the slot adb read,
//! and `fastboot reboot` back to Android on that slot. What each call printed is kept in the run's folder (transport.txt) for the research row. READ ONLY: nothing is
//! written, flashed or set and no image is booted, so no go is read; the way back is the shared one and the phone ends on Android.
//!   procedures/transport-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml [procedures.transport-on-the-pixel-2])
//!
//! ```cargo
//! [dependencies]
//! walleye-tries = { path = "steps" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use walleye_tries::lpoco_attended::{phone_on, prelude::*, steps};
use walleye_tries::transport::Transport;
use walleye_tries::{settings::{Settings, TRANSPORT_PROCEDURE}, watch};


fn main() {
    let _run = procedure(TRANSPORT_PROCEDURE, "google-walleye");
    let t = Transport::new(Settings::for_transport()); // every key from device.toml, read here
    let args: Vec<String> = std::env::args().skip(1).collect();
    if watch::watches(&args) {
        watch::watch_the_try(t.way_back()); // whatever ends a real run, the way back runs after
    }

    check_from("the battery is good enough", "research:walleye-rootfs-boots-run-5-passed", t.battery_is_good());
    step("say what to expect", t.author_is_told(false));
    step("the phone is on Android (the library asks adbd for root)", t.the_phone_is_on_android());
    run_step(steps::read_slot(phone_on("android"))); // the library over [connections.android]
    check_from("adb reads the slot Android runs from", "research:walleye-rootfs-boots-run-5-passed", t.the_slot_is_read_by_adb());
    check_from("adb's own serial is the one [fastboot] declares", "research:walleye-transport-serial-check-source", t.the_adb_serial_is_the_declared_one());
    step("take the phone to its bootloader by `adb reboot bootloader`", t.the_phone_is_taken_to_the_bootloader());
    check_from("fastboot lists the phone and its current-slot is the slot adb read", "research:walleye-rootfs-boots-run-5-passed", t.fastboot_lists_the_slot_adb_read());
    step("`fastboot reboot`: Android answers", t.fastboot_reboot_returns_android());
    step("the phone is on Android (the way back, if it is not)", t.way_back());
    check_from("the phone is back on Android", "research:walleye-fastboot-reboot-stays-in-fastboot-causes-ranked", t.android_is_back());
    check_from("Android runs from the slot it began on", "research:walleye-rootfs-boots-run-5-passed", t.the_slot_is_the_one_it_began_on());
}
