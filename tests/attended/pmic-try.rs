#!/usr/bin/env rust-script
//! The PMIC watchdog v1 try (item walleye-attend-program-pmic; decision 2026-11-19-walleye-attended-tries-one-command).
//! One command: the gates, then the reboot test (arm, clean reboot, Android up and no loop), then the bite last
//! (arm, never pet, the phone restarts by itself), the plain-words prompts built in, the way back. Nothing is written to storage.
//!   tests/attended/pmic-try.rs [--dry-run] [--log FILE]      (its values: device.toml, keys listed in tools/walleye-tries/src/settings.rs)
//!
//! ```cargo
//! [dependencies]
//! lpoco-attended = { git = "https://github.com/linux-phone-oco/lab-tools", tag = "lpoco-attended-v0.32.0" }
//! walleye-tries = { path = "../../tools/walleye-tries" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use lpoco_attended::prelude::*;
use walleye_tries::{pmic::words, pmic::Pmic, settings::{Settings, PMIC_PROCEDURE}};

fn main() {
    let _run = procedure(PMIC_PROCEDURE, "google-walleye");
    let wdt = Pmic::new(Settings::for_pmic());

    check("the battery is good enough", wdt.battery_is_good());
    check("both images are the ones built", wdt.images_are_the_built_ones());
    check("UFS is off in both images", wdt.ufs_is_off_in_both());

    step("take the phone to fastboot", wdt.phone_is_in_fastboot());

    // first test: the reboot; the phone is started and read first, the author is asked after
    step("boot image A from memory (B if A shows no banner)", wdt.boot_image());
    check("the shell answers", wdt.shell_answers());
    check("the watchdog probed and S2 is type 7", wdt.s2_type_is_seven());
    ask_author(words::TWO_TESTS);
    ask_author(words::REBOOT);
    step("arm the watchdog and reboot cleanly", wdt.arm_and_reboot());
    check("Android is back", wdt.android_is_back());
    check("Android stays up, no restart loop", wdt.android_stays_up());

    // second test: the bite, last
    step("back to fastboot", wdt.phone_is_in_fastboot());
    step("boot the same image again", wdt.boot_image());
    check("the shell answers", wdt.shell_answers());
    check("the watchdog probed and S2 is type 7", wdt.s2_type_is_seven());
    check("S2 is off after the clean reboot", wdt.s2_is_off_after_the_reboot());
    ask_author(words::BITE);
    step("arm the watchdog and never pet it", wdt.arm_and_bite());
    check("the phone restarts by itself", wdt.restarts_by_itself());
    ask_author(words::HOLD_POWER);
    check("Android is back", wdt.android_is_back());
    check("Android stays up", wdt.android_stays_up_a_minute());
    step("save the result", wdt.result_is_saved());
}
