#!/usr/bin/env rust-script
//! The osfact display-and-NFC image, both parts read in ONE boot (item walleye-osfact-image-display-and-nfc; decision 2026-11-28-walleye-osfact-parts-join-in-one-build-and-run; the image is the one
//! `osfact pack now` made: the identity-client kernel with NFC built in, B-r5-smmu's dtb with the nfc@28 node, slpi's init with the display-load line and /bin/walleye-nfc-up). The gate, the phone to fastboot, the image from memory
//! (nothing flashed), the shell answering with the image's delay of 5 s; then the DISPLAY's signs, read as display-smmu-on reads them (13 modules ok, no iommu.passthrough, no SMMU fault, no rcg warning, no DSI -110,
//! DSI-1 connected, drm initialized, the CRTC's thread) and the NFC's, read as nfc-boot reads them (tests/health/base/nfc-present.sh, the nxp-nci lines, nfc0 opened by walleye-nfc-up, the pins), kept in the run's folder
//! (the NFC's in its nfc/ folder). Then the way back to Android and the previous boot's console from the pstore. No person step: the glass is not looked at. The root is not pushed by this procedure: both parts are in the ramdisk.
//!   procedures/osfact-display-nfc-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml [procedures.osfact-display-nfc-on-the-pixel-2], [images.osfact-display-nfc])
//!
//! ```cargo
//! [dependencies]
//! walleye-tries = { path = "steps" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use walleye_tries::lpoco_attended::{prelude::*, steps};
use walleye_tries::{
    display_smmu::DisplaySmmu,
    nfc_boot::NfcBoot,
    settings::{Settings, OSFACT_DISPLAY_NFC_PROCEDURE},
};

fn main() {
    let _run = procedure(OSFACT_DISPLAY_NFC_PROCEDURE, "google-walleye");
    let display = DisplaySmmu::new(Settings::for_osfact_display_nfc());
    let mut nfc_settings = Settings::for_osfact_display_nfc();
    nfc_settings.out = nfc_settings.out.join("nfc"); // the two halves keep files of the same names (dmesg.txt, verdict.txt): each in its own folder
    let nfc = NfcBoot::new(nfc_settings);

    let out = Settings::for_osfact_display_nfc().out;
    step("the display half's files of an earlier run are forgotten", move || -> Result<(), String> {
        for name in ["dmesg.txt", "drm.txt", "ps.txt", "drm-state.txt", "panel-driver.txt", "signs.txt", "pstore-console.txt"] {
            let _ = std::fs::remove_file(out.join(name));
        }
        Ok(())
    });
    check_from("the battery is good enough (the NFC half's files of an earlier run are forgotten)", "research:walleye-osfact-display-nfc-image-packed-equal", nfc.battery_is_good());
    check_from("the image is the declared one (sha256)", "research:walleye-osfact-display-nfc-image-packed-equal", display.the_image_is_the_declared_one());
    check_from("UFS is off in the image", "research:walleye-osfact-display-nfc-image-packed-equal", display.ufs_is_off());
    let kernel = display.kernel();
    let ready = steps::phone_ready(&kernel); // built first: a kernel [kernels] test does not list ends the run before the phone is touched
    step("take the phone to fastboot", display.phone_is_in_fastboot());
    run_group(display.the_console_is_owed()); // before the phone is booted: a failed run reads the console of the boot that died in its way back
    run_group(ready); // boots the image the manifest names from memory, nothing flashed, its gadget on the bus
    step("the shell answers and the image has its delay", display.shell_answers_and_the_image_has_its_delay());
    // the NFC half is read first: its steps never fail on the answer, so a display that fails its signs still leaves the NFC verdict of the same boot
    check_from("nfc-present is run and the kernel's nxp-nci lines are read and kept", "research:walleye-nfc-image-built", nfc.the_nfc_is_read());
    check_from(
        "nfc0 is opened, the pins, the bus and the clock are read around it",
        "research:walleye-nfc-clock-and-minus-121",
        nfc.the_nfc0_is_opened_and_the_pins_read(),
    );
    check_from(
        "the display is clean on this kernel: no passthrough, no fault, no warning, connected, drm up",
        "research:walleye-display-reset-image-b-r5-tried-signs-vanish",
        display.the_signs_are_read_and_judged(),
    );
    run_group(steps::leave_known_good()); // LAST: the declared way back (device.toml [kernels]), then Android answers
    step("the previous boot's console is read from Android's pstore and judged", display.the_console_is_read_in_the_pstore());
}
