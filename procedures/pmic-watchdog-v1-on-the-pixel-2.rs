#!/usr/bin/env rust-script
//! The PMIC watchdog v1 try (item walleye-attend-program-pmic; decision 2026-11-19-walleye-attended-tries-one-command).
//! One command: the gates, then the reboot test (arm, clean reboot, Android up and no loop), then the bite last
//! (arm, never pet, the phone restarts by itself), the plain-words prompts built in. However the run ends (a failed check,
//! an unanswered ask, Ctrl-C) the way back runs last: reboot -f, the sysrq trigger, then the power-hold sentence.
//! Nothing is written to storage.
//!   procedures/pmic-watchdog-v1-on-the-pixel-2.rs [--dry-run] [--log FILE]      (its values: device.toml, keys listed in tools/walleye-tries/src/settings.rs)
//!
//! ```cargo
//! [dependencies]
//! walleye-tries = { path = "../tools/walleye-tries" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use walleye_tries::lpoco_attended::prelude::*;
use walleye_tries::{pmic::words, pmic::Pmic, settings::{Settings, PMIC_PROCEDURE}, watch};

fn main() {
    let _run = procedure(PMIC_PROCEDURE, "google-walleye");
    let wdt = Pmic::new(Settings::for_pmic());
    // a real run is watched: whatever ends it (a failed check, an unanswered ask, Ctrl-C), the way back runs after
    let args: Vec<String> = std::env::args().skip(1).collect();
    if watch::watches(&args) {
        watch::watch_the_try(wdt.way_back());
    }

    check_from("the battery is good enough", "research:walleye-pmic-wdt-v1-try-result", wdt.battery_is_good());
    check_from("both images are the ones built", "research:walleye-pmic-wdt-v1-images-built", wdt.images_are_the_built_ones());
    check_from("UFS is off in both images", "research:walleye-pmic-wdt-v1-images-built", wdt.ufs_is_off_in_both());

    step("take the phone to fastboot", wdt.phone_is_in_fastboot());

    // first test: the reboot; the phone is started and read first, the author is asked after
    step("boot image A from memory (B if A shows no banner)", wdt.boot_image());
    check_from("the shell answers", "research:walleye-pmic-wdt-v1-try-result", wdt.shell_answers());
    check_from("the watchdog probed and S2 is type 7", "research:walleye-pmic-bite-reset-type-how-it-stands", wdt.s2_type_is_seven());
    ask_author(words::TWO_TESTS);
    ask_author(words::REBOOT);
    step("arm the watchdog and reboot cleanly", wdt.arm_and_reboot());
    check_from("Android is back", "research:walleye-pmic-wdt-v1-try-result", wdt.android_is_back());
    check_from("Android stays up, no restart loop", "research:walleye-pmic-wdt-v1-try-result", wdt.android_stays_up());

    // second test: the bite, last
    step("back to fastboot", wdt.phone_is_in_fastboot());
    step("boot the same image again", wdt.boot_image());
    check_from("the shell answers", "research:walleye-pmic-wdt-v1-try-result", wdt.shell_answers());
    check_from("the watchdog probed and S2 is type 7", "research:walleye-pmic-bite-reset-type-how-it-stands", wdt.s2_type_is_seven());
    check_from("S2 is off after the clean reboot", "research:walleye-pmic-bite-reset-type-how-it-stands", wdt.s2_is_off_after_the_reboot());
    ask_author(words::BITE);
    step("arm the watchdog and never pet it", wdt.arm_and_bite());
    check_from("the phone restarts by itself", "research:walleye-pmic-wdt-v1-try-result", wdt.restarts_by_itself());
    ask_author(words::HOLD_POWER);
    check_from("Android is back", "research:walleye-pmic-wdt-v1-try-result", wdt.android_is_back());
    check_from("Android stays up", "research:walleye-pmic-wdt-v1-try-result", wdt.android_stays_up_a_minute());
    check_from("Android's boot reason and active slot are read", "research:walleye-pmic-wdt-v1-try-result", wdt.bootreason_and_slot_are_read());
    step("save the result", wdt.result_is_saved());
    step("the phone is on Android (the way back, if it is not)", wdt.way_back());
}
