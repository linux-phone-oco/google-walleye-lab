#!/usr/bin/env rust-script
//! The modem boot as ONE command (item walleye-modem-firmware-push-procedure, for the try walleye-try-modem-mss; research walleye-modem-image-built; decision 2026-11-27-walleye-modem-firmware-is-pushed-not-packed):
//! from Android the phone is taken to its bootloader and the modem image ([images.modem]: UFS off by the gate, static rmtfs, qrtr-ns and tqftpserv, the modem's storage copies, no firmware) is booted ONCE
//! from memory - nothing flashed, no slot changed. Over the image's USB network (172.16.43.1) the lab starts `firmware-recv` on its telnet, pushes the firmware tar (the procedure table's firmware, its sha256 checked
//! first) to port 9000, runs `modem-up`, and reads every remoteproc's state, the kernel's modem lines, the daemons' logs and qrtr-lookup, all inside the init's 180 s window; the answer, whatever it is, is kept in the
//! run's folder (push.txt, modem-up.txt, read.txt, dmesg.txt, daemon-logs.txt, verdict.txt) for the research row. Then the way back is the shared one (`reboot -f` on the image's shell, `fastboot reboot`); the card,
//! the boot chain and the slots' flags are read before and after and must be equal.
//!   procedures/modem-boot-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml [procedures.modem-boot-on-the-pixel-2], [images.modem])
//!
//! ```cargo
//! [dependencies]
//! walleye-tries = { path = "steps" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use walleye_tries::lpoco_attended::prelude::*;
use walleye_tries::modem_boot::ModemBoot;
use walleye_tries::{settings::{Settings, MODEM_BOOT_PROCEDURE}, watch};


fn main() {
    let _run = procedure(MODEM_BOOT_PROCEDURE, "google-walleye");
    let b = ModemBoot::new(Settings::for_modem_boot()); // every key from device.toml, read here
    let root = b.shared();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if watch::watches(&args) {
        watch::watch_the_try(b.way_back()); // whatever ends a real run, the way back runs after
    }

    check_from("the battery is good enough", "research:walleye-rootfs-boots-run-5-passed", root.battery_is_good());
    check_from("the image is the declared modem image and UFS is off in it", "research:walleye-modem-image-built", b.the_image_is_the_declared_ufs_off_image());
    check_from("the firmware tar is the declared one", "research:walleye-modem-image-built", b.the_firmware_is_the_declared_tar());
    check_from("the stored boot chain is whole", "research:walleye-rootfs-boots-run-5-passed", b.the_stored_set_is_good());
    step("say what to expect", b.author_is_told());
    step("the phone is on Android, adbd as root", root.the_phone_is_on_android_as_root());
    check_from("adb reads the slot Android runs from", "research:walleye-rootfs-boots-run-5-passed", b.the_slot_is_read_by_adb());
    check_from("the card is read from Android: its partition tables and the root partition", "research:walleye-stop-stage1-check-17-three-answers", root.the_card_is_read_before());
    step("take the phone to fastboot", root.phone_is_in_fastboot());
    check_from("the slots' flags are read", "research:walleye-rootfs-boots-run-5-passed", root.the_slot_flags_are_read());
    step("boot the modem image from memory, nothing flashed", b.boot_the_image());
    check_from("the gadget is up and the shell answers", "research:walleye-modem-image-built", b.the_gadget_is_up());
    check_from("the firmware tar is pushed and the phone unpacked it", "research:walleye-modem-image-built", b.the_firmware_is_pushed());
    check_from("modem-up checks the set and starts qrtr-ns, tqftpserv and rmtfs", "research:walleye-modem-image-built", b.the_modem_is_started());
    check_from("the remoteproc states, the kernel's modem lines, the daemons' logs and qrtr-lookup are read and kept", "research:walleye-modem-image-built", b.the_modem_is_read());
    check_from("the modem is asked its device ids over QRTR, read only, and the answer is kept private (the IMEI never printed)", "research:walleye-modem-to-calls-how-it-stands", b.the_ids_are_read_and_kept_private());
    step("the phone is on Android (the way back, if it is not)", b.way_back());
    check_from("the phone is back on Android", "research:walleye-rootfs-boots-run-5-passed", root.android_is_back());
    step("Android answers as root again", root.the_phone_is_on_android_as_root());
    check_from("the partition tables and the root partition are as before", "research:walleye-stop-stage1-check-17-three-answers", root.the_card_is_as_before());
    check_from("the boot chain read again equals the stored set", "research:walleye-rootfs-boots-run-5-passed", root.the_boot_chain_is_as_stored());
    check_from("both slots' flags are as before", "research:walleye-rootfs-boots-run-5-passed", root.the_slot_flags_are_as_before());
}
