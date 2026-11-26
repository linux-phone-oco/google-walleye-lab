#!/usr/bin/env rust-script
//! The rehearsal of the transport run (item walleye-transport-procedure): the reads on Android only - adb as root, the library's `read_slot(phone_on("android"))`, adb's
//! own serial against the declared one - and no trip to the bootloader. It writes nothing and needs no go. The phone never leaves Android.
//!   procedures/rehearse-transport-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml [procedures.rehearse-transport-on-the-pixel-2])
//!
//! ```cargo
//! [dependencies]
//! walleye-tries = { path = "../tools/walleye-tries" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use walleye_tries::lpoco_attended::{phone_on, prelude::*, steps};
use walleye_tries::transport::Transport;
use walleye_tries::{settings::{Settings, REHEARSE_TRANSPORT_PROCEDURE}, watch};


fn main() {
    let _run = procedure(REHEARSE_TRANSPORT_PROCEDURE, "google-walleye");
    let t = Transport::new(Settings::for_rehearse_transport()); // every key from device.toml, read here
    let args: Vec<String> = std::env::args().skip(1).collect();
    if watch::watches(&args) {
        watch::watch_the_try(t.way_back()); // whatever ends a real run, the way back runs after
    }

    check_from("the battery is good enough", "research:walleye-rootfs-boots-run-5-passed", t.battery_is_good());
    step("say what to expect", t.author_is_told(true));
    step("the phone is on Android, adbd as root", t.the_phone_is_on_android_as_root());
    run_step(steps::read_slot(phone_on("android"))); // the library over [connections.android]
    check_from("adb reads the slot Android runs from", "research:walleye-rootfs-boots-run-5-passed", t.the_slot_is_read_by_adb());
    check_from("adb's own serial is the one [fastboot] declares", "file:docs/backlog.md", t.the_adb_serial_is_the_declared_one());
    check_from("Android runs from the slot it began on", "research:walleye-rootfs-boots-run-5-passed", t.the_slot_is_the_one_it_began_on());
}
