#!/usr/bin/env rust-script
//! The PON control read as ONE command (item walleye-wdt-pon-control-procedure; decision 2026-11-26-pon-reasons-read-in-our-image-after-a-control; research
//! walleye-pon-read-after-bite-the-way): no bite, a known power-off. The phone is switched off by adb (`reboot -p`), the author does his one act (volume-down and power
//! held to the bootloader), the image pon-read is booted from memory (steps::phone_ready, nothing flashed) and its /bin/pon-block prints the PON block 0xC0..0xCB raw; the lines
//! are kept in the run's folder. It counts what a key press and the bootloader leave of the registers; it names no register's meaning. The way back is the declared one (steps::leave_known_good).
//!   procedures/pon-control-read-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml [procedures.pon-control-read-on-the-pixel-2])
//!
//! ```cargo
//! [dependencies]
//! walleye-tries = { path = "steps" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use walleye_tries::lpoco_attended::{prelude::*, steps};
use walleye_tries::pon_control::{words, PonControl, KERNEL};
use walleye_tries::settings::{Settings, PON_CONTROL_PROCEDURE};

fn main() {
    let _run = procedure(PON_CONTROL_PROCEDURE, "google-walleye");
    let t = PonControl::new(Settings::for_pon_control()); // every key from device.toml, read here

    check_from("the battery is good enough", "research:walleye-pmic-wdt-v1-try-result", t.battery_is_good());
    check_from("the image is the one built", "research:walleye-pon-read-after-bite-the-way", t.image_is_the_built_one());
    check_from("UFS is off in the image", "research:walleye-pon-read-after-bite-the-way", t.ufs_is_off());
    ask_author(words::ONE_HOLD);
    check_from("the phone is on Android", "research:walleye-pon-read-after-bite-the-way", t.the_phone_is_on_android());
    step("power the phone off by adb (a known cause, no bite)", t.power_off_by_adb());
    ask_author(words::HOLD); // the author's one act
    check_from("the bootloader lists the phone", "research:walleye-pon-read-after-bite-the-way", t.the_bootloader_lists_the_phone());
    run_group(steps::phone_ready(KERNEL)); // boots [images.pon-read] from memory, its gadget on the bus, the shell answering
    check_from("the PON block 0xC0..0xCB is read and saved raw", "research:walleye-pon-read-after-bite-the-way", t.the_block_is_read_and_saved());
    run_group(steps::leave_known_good()); // LAST: the declared way back, then Android answers
}
