#!/usr/bin/env rust-script
//! The lab's way back to Android as a command of its own: what device.toml `[kernels] way_back` names, so the library's step groups
//! (phone_ready, leave_known_good of lpoco-attended 0.46.2) end a run the way walleye ends it today (item walleye-procedures-on-step-groups;
//! decision 2026-11-26-walleye-way-back-is-a-procedure). The phone may be on Android already (nothing is sent), in its bootloader
//! (`fastboot reboot`, a second only if it still lists) or on a test image's shell (`reboot -f`, then the sysrq reboot); it ends on Android,
//! as `bin/attended run` reports. Nothing is flashed, set or written; no image is booted, so no go is read.
//!   procedures/way-back-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: the shared keys of device.toml; [procedures.way-back-on-the-pixel-2] has none of its own)
//!
//! ```cargo
//! [dependencies]
//! walleye-tries = { path = "../tools/walleye-tries" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::rc::Rc;
use walleye_tries::lab::Lab;
use walleye_tries::lpoco_attended::prelude::*;
use walleye_tries::settings::{Settings, WAY_BACK_PROCEDURE};

fn main() {
    let _run = procedure(WAY_BACK_PROCEDURE, "google-walleye");
    let lab = Lab::new(Rc::new(Settings::for_way_back())); // every key from device.toml, read here

    let way = lab.clone();
    step(
        "the phone comes back to Android by the lab's way back",
        move || way.way_back(),
    );
    check_from(
        "the phone is back on Android",
        "research:walleye-fastboot-reboot-stays-in-fastboot-causes-ranked",
        move || lab.android_is_back(),
    );
}
