#!/usr/bin/env rust-script
//! The NFC boot as ONE command (item walleye-nfc-boot-procedure, for the try walleye-nfc-boot; research walleye-nfc-image-built;
//! decisions 2026-11-26-walleye-ramdisk-on-net-43, 2026-11-19-walleye-attended-tries-one-command): from Android the phone is taken to its bootloader and the NFC
//! image ([images.nfc]: UFS off by the gate, the nfc@28 node, NFC built in, the roof's init at walleye's net 43) is booted ONCE from memory - nothing flashed by this
//! procedure (steps::phone_ready(nfc), `fastboot boot`). Over the image's telnet the lab runs tests/health/base/nfc-present.sh and reads the kernel's lines on nxp-nci, and the deferred devices
//! and the clock lines (said in the verdict when it did not probe); the answer, whatever it is, is kept in the run's folder (read.txt, dmesg.txt, verdict.txt) for the research row.
//! Then nfc0 is opened (nfctool -d nfc0 -1 where the ramdisk has it; else said not known yet), the pins gpio12, 116, 92 and 93 are read before and after, i2cdetect runs on the node's bus and ln_bb_clk3 is read (open.txt): the answers say VEN (no ACK), the clock (ACK, no NCI reply) or gpio93 (firmware framing) - research walleye-nfc-clock-and-minus-121.
//! The roof's init has no hold and no watchdog petter (research walleye-generated-init-gaps): the read is short. Then steps::leave_known_good, last.
//!   procedures/nfc-boot-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml [procedures.nfc-boot-on-the-pixel-2], [images.nfc])
//!
//! ```cargo
//! [dependencies]
//! walleye-tries = { path = "steps" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use walleye_tries::lpoco_attended::{prelude::*, steps};
use walleye_tries::nfc_boot::{NfcBoot, KERNEL};
use walleye_tries::{settings::{Settings, NFC_BOOT_PROCEDURE}, watch};

fn main() {
    let _run = procedure(NFC_BOOT_PROCEDURE, "google-walleye");
    let b = NfcBoot::new(Settings::for_nfc_boot()); // every key from device.toml, read here
    // a real run is watched: whatever ends it (an unread shell, a failed check, Ctrl-C), the way back runs after, so the phone never stays on the RAM image
    let args: Vec<String> = std::env::args().skip(1).collect();
    if watch::watches(&args) {
        watch::watch_the_try(b.way_back());
    }

    check_from("the battery is good enough", "research:walleye-rootfs-boots-run-5-passed", b.battery_is_good());
    check_from("the image is the declared one, built, and UFS is off in it", "research:walleye-nfc-image-built", b.the_image_is_the_declared_ufs_off_image());
    step("say what to expect", b.author_is_told());
    step("take the phone to fastboot", b.phone_is_in_fastboot());
    run_group(steps::phone_ready(KERNEL)); // boots [images.nfc] from memory, nothing flashed, its gadget on the bus, the shell answering
    check_from("the gadget is up under the roof's serial and the shell answers at walleye's address", "research:walleye-generated-init-gaps", b.the_gadget_is_up());
    check_from("nfc-present is run, the kernel's nxp-nci lines are read and kept", "research:walleye-nfc-image-built", b.the_nfc_is_read());
    check_from("nfc0 is opened, the pins, the bus and the clock are read around it, and the cause the answers point to is said", "research:walleye-nfc-clock-and-minus-121", b.the_nfc0_is_opened_and_the_pins_read());
    run_group(steps::leave_known_good()); // LAST: the declared way back (device.toml [kernels]), then Android answers
}
