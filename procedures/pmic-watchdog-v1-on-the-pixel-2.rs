#!/usr/bin/env rust-script
//! The PMIC watchdog v1 try (item walleye-attend-program-pmic; decision 2026-11-19-walleye-attended-tries-one-command).
//! One command: the gates, then the reboot test (arm, clean reboot, Android up and no loop), then the bite last
//! (arm, never pet, the phone restarts by itself), the plain-words prompts built in. Each test is its own step group:
//! steps::phone_ready(KERNEL) boots the image once from memory, steps::leave_known_good ends the test on Android
//! (lpoco-attended 0.46.4); a failed check or an unanswered ask in between runs the way back of the group that is open.
//! Nothing is written to storage.
//!   procedures/pmic-watchdog-v1-on-the-pixel-2.rs [--dry-run] [--log FILE]      (its values: device.toml, keys listed in tools/walleye-tries/src/settings.rs)
//!
//! ```cargo
//! [dependencies]
//! walleye-tries = { path = "../tools/walleye-tries" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use walleye_tries::lpoco_attended::{prelude::*, steps};
use walleye_tries::{pmic::words, pmic::{Pmic, KERNEL}, settings::{Settings, PMIC_PROCEDURE}};

fn main() {
    let _run = procedure(PMIC_PROCEDURE, "google-walleye");
    let wdt = Pmic::new(Settings::for_pmic());

    check_from("the battery is good enough", "research:walleye-pmic-wdt-v1-try-result", wdt.battery_is_good());
    check_from("the image is the one built", "research:walleye-pmic-wdt-v1-images-built", wdt.image_is_the_built_one());
    check_from("UFS is off in the image", "research:walleye-pmic-wdt-v1-images-built", wdt.ufs_is_off());

    // first test: the reboot; the phone is started and read first, the author is asked after
    step("take the phone to fastboot", wdt.phone_is_in_fastboot());
    run_group(steps::phone_ready(KERNEL)); // boots [images.wdt1-a] from memory, its gadget on the bus, the shell answering
    check_from("the phone's dmesg is saved", "research:walleye-pmic-wdt-v1-try-result", wdt.dmesg_is_saved());
    check_from("the watchdog probed and S2 is type 7", "research:walleye-pmic-bite-reset-type-how-it-stands", wdt.s2_type_is_seven());
    ask_author(words::TWO_TESTS);
    ask_author(words::REBOOT);
    step("arm the watchdog and reboot cleanly", wdt.arm_and_reboot());
    check_from("Android is back", "research:walleye-pmic-wdt-v1-try-result", wdt.android_is_back());
    check_from("Android stays up, no restart loop", "research:walleye-pmic-wdt-v1-try-result", wdt.android_stays_up());
    run_group(steps::leave_known_good()); // the first test ends on Android by the declared way back

    // second test: the bite, last
    step("back to fastboot", wdt.phone_is_in_fastboot());
    run_group(steps::phone_ready(KERNEL));
    check_from("the phone's dmesg is saved", "research:walleye-pmic-wdt-v1-try-result", wdt.dmesg_is_saved());
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
    run_group(steps::leave_known_good()); // LAST: the declared way back, then Android answers
}
