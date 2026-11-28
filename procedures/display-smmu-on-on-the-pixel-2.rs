#!/usr/bin/env rust-script
//! The display with the SMMU on, as one command (item walleye-display-smmu-on-procedure; decisions 2026-11-28-walleye-display-smmu-on-reset-first and
//! 2026-11-28-walleye-iommu-stays-on-per-device-fix; research walleye-display-reset-smmu-on-image-built). Image B-r5-smmu is B-r5 (the MDSS reset) with the board's own
//! command line, so the SMMU stays on. The gate, the phone to fastboot, the image from memory (nothing flashed), the shell answering with the image's delay of 5 s, then the
//! signs read once the init says it is through the 13 modules: no `iommu.passthrough` on the command line, no arm-smmu context fault, no `rcg` warning, no DSI -110,
//! the DSI-1 connector connected, the panel bound, drm initialized, the CRTC's thread in ps (its debugfs state is kept, not judged: no captured text of it yet);
//! then the way back to Android, and the previous boot's console read from its pstore (kept as pstore-console.txt, judged: does it hold our kernel's lines?). NO person step: the glass is not looked at. The readings are kept in ~/lab-material/walleye/display-smmu-on-DAY/.
//! A failed step or a panic ends with the way back to Android (the phone_ready group's own), then the console read from the pstore (the owed group's); a signal runs the owed way backs too.
//!   procedures/display-smmu-on-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml, keys listed in procedures/steps/src/settings.rs)
//!
//! ```cargo
//! [dependencies]
//! walleye-tries = { path = "steps" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use walleye_tries::lpoco_attended::{prelude::*, steps};
use walleye_tries::{
    display_smmu::DisplaySmmu,
    settings::{Settings, DISPLAY_SMMU_PROCEDURE},
};

fn main() {
    let _run = procedure(DISPLAY_SMMU_PROCEDURE, "google-walleye");
    let smmu = DisplaySmmu::new(Settings::for_display_smmu());

    check_from("the battery is good enough", "research:walleye-display-reset-smmu-on-image-built", smmu.battery_is_good());
    check_from("the image is the declared one (sha256, ensured from its recipe)", "research:walleye-display-reset-smmu-on-image-built", smmu.the_image_is_the_declared_one());
    check_from("UFS is off in the image", "research:walleye-display-reset-smmu-on-image-built", smmu.ufs_is_off());
    let kernel = smmu.kernel();
    let ready = steps::phone_ready(&kernel); // built first: a kernel [kernels] test does not list ends the run before the phone is touched
    step("take the phone to fastboot", smmu.phone_is_in_fastboot());
    run_group(smmu.the_console_is_owed()); // before the phone is booted: a failed run reads the console of the boot that died in its way back, after Android is up
    run_group(ready); // boots the image the manifest names ([procedures.display-smmu-on-on-the-pixel-2] image) from memory, nothing flashed, its gadget on the bus
    step(
        "the shell answers and the image has its delay",
        smmu.shell_answers_and_the_image_has_its_delay(),
    );
    check_from(
        "the SMMU is on and the display is clean: no passthrough, no fault, no warning, connected, panel bound, drm up",
        "research:walleye-display-reset-image-b-r5-tried-signs-vanish",
        smmu.the_signs_are_read_and_judged(),
    );
    run_group(steps::leave_known_good()); // LAST: the declared way back (device.toml [kernels]), then Android answers
    step("the previous boot's console is read from Android's pstore and judged", smmu.the_console_is_read_in_the_pstore());
}
