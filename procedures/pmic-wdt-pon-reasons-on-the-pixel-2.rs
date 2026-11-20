#!/usr/bin/env rust-script
//! The four PMIC bites (item walleye-wdt-pon-bites-procedure): the watchdog bites with the S2 reset type 0x07 (hard)
//! and 0x01 (warm), each with the USB cable in and with it out, and the PON registers are read after each, before
//! anything resets them. The lab decides every part it can: the shell goes quiet at the bite; then adb up by itself is
//! "Android came back", fastboot listed is "the bootloader", neither is "the charger screen, or off". The author is
//! asked only for physical acts (the cable, a look at the screen, the buttons), and before the first bite, so a run
//! nobody answers ends before any bite, the phone back on Android by the shared way back.
//!   procedures/pmic-wdt-pon-reasons-on-the-pixel-2.rs [--dry-run] [--log FILE]      (its values: device.toml, keys listed in tools/walleye-tries/src/settings.rs)
//!
//! ```cargo
//! [dependencies]
//! lpoco-attended = { git = "https://github.com/linux-phone-oco/lab-tools", tag = "lpoco-attended-v0.39.0" }
//! walleye-tries = { path = "../tools/walleye-tries" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use lpoco_attended::prelude::*;
use lpoco_attended::steps;
use walleye_tries::{
    pmic::bites::{words, Bite, Cable},
    pmic::Pmic,
    settings::{Settings, Try, BITES_PROCEDURE},
    watch,
};

fn main() {
    let _run = procedure(BITES_PROCEDURE, "google-walleye");
    // the keys are read through this entry's own runner (0.39), so the plumbing may stay on the 0.35 the older entries use
    let wdt = Pmic::for_bites(Settings::read(Try::Bites, &|k| dev(k).to_string(), &|k| {
        proc(k).to_string()
    }));
    // a real run is watched: whatever ends it (a failed check, an unanswered ask, Ctrl-C), the way back runs after
    let args: Vec<String> = std::env::args().skip(1).collect();
    if watch::watches(&args) {
        watch::watch_the_try(wdt.way_back());
    }

    check("the battery is good enough", wdt.battery_is_good());
    check(
        "both images are the ones built",
        wdt.images_are_the_built_ones(),
    );
    check("UFS is off in both images", wdt.ufs_is_off_in_both());

    for bite in [Bite::HARD_IN, Bite::HARD_OUT, Bite::WARM_IN, Bite::WARM_OUT] {
        one_bite(&wdt, bite);
    }
    step("save the result", wdt.result_is_saved());
    step(
        "the phone is on Android (the way back, if it is not)",
        wdt.way_back(),
    );
}

/// One bite as groups: ready, the bite, what the lab sees, what the author is asked when the phone shows nothing, the reading.
#[rustfmt::skip]
fn one_bite(wdt: &Pmic, b: Bite) {
    let name = b.name();
    run_group(group(&format!("{name}: get ready"), [
        Step::new(Kind::Check, "the battery is good enough", wdt.battery_is_good()),
        Step::new(Kind::Do, "take the phone to fastboot", wdt.phone_is_in_fastboot()),
        Step::new(Kind::Do, "boot this bite's image from memory", wdt.bite_image_boots(b)),
        Step::new(Kind::Check, "the shell answers", wdt.shell_answers()),
        Step::new(Kind::Check, "the watchdog probed and the S2 type is this bite's", wdt.bite_type_reads(b)),
        steps::confirm(words::READY),
    ]));
    let arm = Step::new(Kind::Do, "arm the watchdog and never pet it", wdt.bite_is_armed(b));
    run_group(match b.cable {
        Cable::In => group(&format!("{name}: the bite"), [
            arm,
            Step::new(Kind::Check, "the phone goes quiet at the bite", wdt.the_phone_goes_quiet(b)),
        ]),
        Cable::Out => group(&format!("{name}: the bite"), [
            arm,
            steps::unplug_usb(words::CABLE_OUT),
            Step::new(Kind::Check, "the host no longer reaches the phone", wdt.the_host_lost_the_phone()),
            Step::new(Kind::Do, "wait out the bite", wdt.the_bite_passes(b)),
            steps::replug_usb(words::CABLE_BACK),
        ]),
    });
    step("what the lab sees: Android, the bootloader or nothing", wdt.what_the_lab_sees(b));
    if wdt.the_phone_is_dark() {
        run_group(group(&format!("{name}: the phone shows nothing"), [
            steps::look_for(words::LOOK),
            steps::hold_button(words::BUTTONS, words::BOOTLOADER),
            Step::new(Kind::Do, "what the lab sees now", wdt.what_the_lab_sees(b)),
        ]));
    }
    run_group(group(&format!("{name}: the reading"), [
        Step::new(Kind::Do, "read the reason where the phone first answers", wdt.the_reason_is_read(b)),
        Step::new(Kind::Do, "the phone is back on Android", wdt.the_phone_is_back_on_android()),
    ]));
}
