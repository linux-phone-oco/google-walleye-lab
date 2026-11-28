// SPDX-License-Identifier: MIT OR Apache-2.0
//! The SMMU-on display try's steps (item walleye-display-smmu-on-procedure; decisions 2026-11-28-walleye-display-smmu-on-reset-first and
//! 2026-11-28-walleye-iommu-stays-on-per-device-fix; research walleye-display-reset-smmu-on-image-built): image B-r5-smmu, B-r5 with the board's own
//! command line (no `iommu.passthrough=1`), booted once from memory. The MDSS reset (B-r5's dtb) removes the bootloader's scan-out, so the SMMU can stay on;
//! the try reads whether it did: the kernel command line has no passthrough word, no arm-smmu context fault, no `rcg` warning and no DSI -110 after the
//! init loaded the 13 display modules, the DSI-1 connector reads connected, the panel is bound, drm is initialized, and the CRTC's thread is there.
//! Nothing is written to the phone and nobody is asked: the glass is not looked at (decision 2026-11-19-walleye-display-console-before-eyes).
//! Each method is one step of procedures/display-smmu-on-on-the-pixel-2.rs. The checks' expected words come from captured text (tests/fixtures/phone-text/SOURCES.toml).

use crate::android::Android;
use crate::display_boot::{drm_command, drm_of, drm_problem, loop_is_done};
use crate::display_delay::{score, Row};
use crate::lab::Lab;
use crate::settings::Settings;
use crate::touch::redact;
use std::rc::Rc;
use std::time::{Duration, Instant};

type Done = Result<(), String>;

/// The word that turns the SMMU's translation off on the command line (the bring-up shortcut B-r5-smmu drops).
const PASSTHROUGH: &str = "iommu.passthrough";
/// What the kernel prints before each line of its command line (a long one is printed in chunks, each led by this).
const CMDLINE: &str = "Kernel command line:";
/// The DSI host bound to the display controller (`msm_dpu c901000.display-controller: bound c994000.dsi (ops dsi_ops [msm])`).
const DSI_BOUND: &str = "bound c994000.dsi";
/// The panel driver's own lines carry its device (`panel-samsung-s6e3fa5 c994000.dsi.0: ...`).
const PANEL: &str = "panel-samsung-s6e3fa5 c994000.dsi.0";
/// A failed bind of the panel driver prints `probe with driver panel-samsung-s6e3fa5 failed with error N`.
const PANEL_FAILED: &str = "probe with driver panel-samsung-s6e3fa5 failed";
/// `[drm] Initialized msm 1.13.0 for c901000.display-controller on minor 0`.
const DRM_INITIALIZED: &str = "[drm] Initialized msm";
/// The kernel thread of the first CRTC, as `ps` lists it (`[card0-crtc0]`).
const CRTC_THREAD: &str = "card0-crtc0";
/// The phone's own state of the DRM device, saved for the next reading and not judged yet (no captured text of it).
pub const DRM_STATE_COMMAND: &str =
    "mount -t debugfs none /sys/kernel/debug 2>/dev/null; cat /sys/kernel/debug/dri/0/state 2>&1";

/// The command line the kernel printed, in one string: its chunks of the `Kernel command line:` lines of the log.
pub fn kernel_command_line(dmesg: &str) -> String {
    dmesg
        .lines()
        .filter_map(|l| {
            l.split_once(CMDLINE)
                .map(|(_, rest)| rest.trim().trim_end_matches('\\').trim())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The command line turns the SMMU's translation off (`iommu.passthrough=1`, or any value of the word).
pub fn has_passthrough(cmdline: &str) -> bool {
    cmdline
        .split_whitespace()
        .any(|w| w.starts_with(PASSTHROUGH))
}

/// Whether the log's command line has the passthrough word; `None` when the log has no command line.
pub fn command_line_has_passthrough(dmesg: &str) -> Option<bool> {
    let line = kernel_command_line(dmesg);
    (!line.is_empty()).then(|| has_passthrough(&line))
}

/// The display controller's own failure lines (`[dpu error]enc35 pp0 disable autorefresh failed`), the fourth sign of tests/health/base/display-boot-clean.sh.
pub fn autorefresh_failures(dmesg: &str) -> usize {
    dmesg
        .lines()
        .filter(|l| l.contains("disable autorefresh failed"))
        .count()
}

/// The panel is bound: the DSI host is bound to the display controller, the panel driver printed its lines and did not say its probe failed.
pub fn panel_is_bound(dmesg: &str) -> bool {
    dmesg.contains(DSI_BOUND)
        && dmesg.lines().any(|l| l.contains(PANEL))
        && !dmesg.contains(PANEL_FAILED)
}

/// drm said it initialized msm and the framebuffer device came up on it.
pub fn drm_is_initialized(dmesg: &str) -> bool {
    dmesg.contains(DRM_INITIALIZED) && dmesg.contains("msmdrmfb frame buffer device")
}

/// The first CRTC's thread is in `ps`'s listing.
pub fn crtc_thread_is_listed(ps: &str) -> bool {
    ps.lines().any(|l| l.contains(CRTC_THREAD))
}

/// Everything one reading of the phone says about the SMMU-on display, and the one sentence it makes.
#[derive(Debug, Clone, PartialEq)]
pub struct Signs {
    pub row: Row,
    /// `None`: the log has no kernel command line to read, which is not a pass
    pub passthrough: Option<bool>,
    pub autorefresh_failures: usize,
    pub connected: Option<String>,
    pub panel_bound: bool,
    pub drm_initialized: bool,
    pub crtc_listed: bool,
}

/// One reading judged: every reason the boot is not display-boot-clean with the SMMU on, none when it is.
pub fn problems(s: &Signs) -> Vec<String> {
    let mut out = Vec::new();
    if s.passthrough.is_none() {
        out.push(
            "the kernel log has no command line to read: whether the SMMU is on is not known"
                .to_string(),
        );
    }
    if s.passthrough == Some(true) {
        out.push("the kernel command line still has iommu.passthrough: the SMMU is off, this is not the SMMU-on image".to_string());
    }
    if !s.row.loaded() {
        out.push(format!(
            "the init did not finish the list with every module ok ({} of {} ok, finished {})",
            s.row.modules_ok, s.row.modules_want, s.row.finished
        ));
    }
    for (n, what) in [
        (s.row.smmu_faults, "arm-smmu context fault lines"),
        (
            s.row.rcg_warnings,
            "rcg didn't update its configuration warnings",
        ),
        (s.row.dsi_110, "DSI -110 timeouts"),
        (s.autorefresh_failures, "disable autorefresh failed lines"),
    ] {
        if n > 0 {
            out.push(format!("{n} {what}"));
        }
    }
    if let Some(why) = &s.connected {
        out.push(why.clone());
    }
    for (ok, what) in [
        (s.panel_bound, "the panel is not bound (no 'bound c994000.dsi' with the panel driver's lines, or its probe failed)"),
        (s.drm_initialized, "drm did not initialize msm with its framebuffer device"),
        (s.crtc_listed, "no card0-crtc0 thread in ps"),
    ] {
        if !ok {
            out.push(what.to_string());
        }
    }
    out
}

/// The signs as they are kept: one line each, said and saved.
pub fn report(s: &Signs) -> String {
    let yes = |b: bool| if b { "yes" } else { "no" };
    format!(
        "passthrough_on_cmdline {}\nsmmu_faults {}\nrcg_warnings {}\ndsi_110 {}\nmodules_ok {} of {}\nconnector {}\npanel_bound {}\ndrm_initialized {}\ncrtc_thread {}\ncrtc_active not known yet (debugfs state kept in drm-state.txt, no captured text to judge it by)\nverdict {}\n",
        s.passthrough.map_or("not known", yes),
        s.row.smmu_faults,
        s.row.rcg_warnings,
        s.row.dsi_110,
        s.row.modules_ok,
        s.row.modules_want,
        s.connected.as_deref().unwrap_or("connected"),
        yes(s.panel_bound),
        yes(s.drm_initialized),
        yes(s.crtc_listed),
        if problems(s).is_empty() { "display-boot-clean (crtc active not judged)" } else { "not clean" },
    )
}

/// The kernel version lines of a console that are not Android's (`Linux version` 5 and up (Android's own is 4.4)): the proof that the pstore holds the mainline boot, not only Android's own 4.4 one.
pub fn mainline_boot_lines(console: &str) -> Vec<&str> {
    console
        .lines()
        .filter(|l| {
            l.split("Linux version ")
                .nth(1)
                .and_then(|v| v.split('.').next())
                .and_then(|major| major.parse::<u32>().ok())
                .is_some_and(|major| major >= 5)
        })
        .collect()
}

/// What the previous boot's console says, judged: the mainline kernel's own lines are there or they are not (no failure: the way back has already passed).
pub fn console_verdict(console: &str) -> String {
    let lines = mainline_boot_lines(console);
    match lines.first() {
        Some(first) => format!("console-ramoops holds the mainline boot ({} version line(s): {})", lines.len(), first.trim()),
        None => "console-ramoops holds no mainline kernel line (no 'Linux version' 5 or higher): our kernel's console did not survive, or Android's pstore could not read it".to_string(),
    }
}

/// The SMMU-on try: its settings and the lab it shares with the other tries. Cheap to clone: each step takes its own.
#[derive(Clone)]
pub struct DisplaySmmu {
    s: Rc<Settings>,
    lab: Lab,
    android: Android,
}

impl DisplaySmmu {
    pub fn new(s: Settings) -> DisplaySmmu {
        let s = Rc::new(s);
        let lab = Lab::new(s.clone());
        DisplaySmmu {
            lab: lab.clone(),
            android: Android::new(s.clone(), lab.clone()),
            s,
        }
    }

    /// The test kernel the try boots (its `[images.NAME]` and `[kernels] test` entry): steps::phone_ready names it.
    pub fn kernel(&self) -> String {
        self.s.image_name.clone()
    }

    /// The delay the image's init waits before it loads the modules (seconds of uptime).
    fn delay(&self) -> u64 {
        self.s.delays[0]
    }

    /// The battery is above the lab's minimum (the shared step).
    pub fn battery_is_good(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.battery_is_good()
    }

    /// The image is the declared one: a recipe image (no build file, no SHA256SUMS line) is taken from `image ensure` by its sha256 and put where the build files
    /// live (the shared step of the root boot), then its sha256 is the one device.toml records. Nothing of the phone is touched.
    pub fn the_image_is_the_declared_one(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || {
            if let Some(dir) = d.s.image.parent() {
                std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
            }
            println!(
                "  {}",
                crate::host::the_image_is_ensured(
                    &d.s.bringup,
                    &d.s.image_name,
                    &d.s.device_toml(),
                    &d.s.image,
                    &d.s.image_sha256
                )?
            );
            crate::host::image_has_sha256(&d.s.image, &d.s.image_sha256)
        }
    }

    /// UFS is off in the image (the gate every image passes before a boot).
    pub fn ufs_is_off(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || d.lab.ufs_is_off(&d.s.image)
    }

    /// The phone is in fastboot (the shared step).
    pub fn phone_is_in_fastboot(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.phone_is_in_fastboot()
    }

    /// The shell answers by itself and the image has the delay: `/display-delay` holds the number (the delay init B-r5's ramdisk has).
    pub fn shell_answers_and_the_image_has_its_delay(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || {
            let mut sh = d.lab.shell_when_up()?;
            let said = sh.run("cat /display-delay", Duration::from_secs(10))?;
            if said.trim() != d.delay().to_string() {
                return Err(format!(
                    "the image says its delay is {:?}, not {}: it is not the delay init",
                    said.trim(),
                    d.delay()
                ));
            }
            Ok(())
        }
    }

    /// The init loaded the modules (or `device_wait` is over), the log, the DRM class, `ps` and the DRM state are read, kept (the serial redacted) and judged.
    /// The log is read again every 2 s until two reads in a row count the same, as the delay bisect does. A reading with problems fails the step with all of them.
    pub fn the_signs_are_read_and_judged(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || {
            let mut sh = d.lab.shell(10)?;
            let until = Instant::now() + Duration::from_secs(d.s.device_wait);
            let (dmesg, row) = {
                let mut last: Option<Row> = None;
                loop {
                    let dmesg = redact(&sh.run("dmesg", Duration::from_secs(20))?);
                    let row = score(d.delay(), d.s.modules as usize, &dmesg);
                    let settled = loop_is_done(&dmesg) && last.as_ref() == Some(&row);
                    if settled || Instant::now() >= until {
                        break (dmesg, row);
                    }
                    last = Some(row);
                    std::thread::sleep(Duration::from_secs(2));
                }
            };
            let drm = sh.run(&drm_command(), Duration::from_secs(10))?;
            let ps = sh.run("(ps -o pid,args 2>&1 || ps 2>&1)", Duration::from_secs(10))?;
            let state = sh.run(DRM_STATE_COMMAND, Duration::from_secs(10))?;
            d.lab.save("dmesg.txt", &dmesg)?;
            d.lab.save("drm.txt", &drm)?;
            d.lab.save("ps.txt", &ps)?;
            d.lab.save("drm-state.txt", &redact(&state))?;
            let signs = Signs {
                row,
                passthrough: command_line_has_passthrough(&dmesg),
                autorefresh_failures: autorefresh_failures(&dmesg),
                connected: drm_problem(&drm_of(&drm)),
                panel_bound: panel_is_bound(&dmesg),
                drm_initialized: drm_is_initialized(&dmesg),
                crtc_listed: crtc_thread_is_listed(&ps),
            };
            let said = report(&signs);
            d.lab.save("signs.txt", &said)?;
            for l in said.lines() {
                println!("  {l}");
            }
            let found = problems(&signs);
            if found.is_empty() {
                Ok(())
            } else {
                Err(found.join("; "))
            }
        }
    }

    /// After the way back, on Android: the previous boot's console is read (`adb root`, then [`crate::rw_write::PSTORE_COMMAND`], read only), kept as `pstore-console.txt` in the
    /// run's folder and judged ([`console_verdict`]). The ramoops node of B-r5-smmu has stock Android's layout (decision 2026-11-28-walleye-display-try-needs-its-console-first), so a
    /// try that dies before its gadget still leaves its last lines here. A read that cannot be made is said as not known yet; this step never fails the run.
    pub fn the_console_is_read_in_the_pstore(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || {
            let _ = d.android.adb(&["root"]);
            std::thread::sleep(Duration::from_secs(3));
            match d.android.adb(&["shell", crate::rw_write::PSTORE_COMMAND]) {
                Ok(text) if !text.contains("Permission denied") => {
                    let text = redact(&text);
                    if let Err(e) = d.lab.save(crate::rw_write::PSTORE_FILE, &text) {
                        println!("NOT KNOWN YET: the console could not be kept: {e}");
                    }
                    println!("  {}", console_verdict(&text));
                }
                Ok(_) => println!("NOT KNOWN YET: the pstore is closed to adb (no root)"),
                Err(e) => println!("NOT KNOWN YET: the pstore could not be read: {e}"),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// B-r5's kernel log at its 5 s load (iommu.passthrough=1; the signs vanished): tests/fixtures/phone-text/SOURCES.toml names its source.
    const CLEAN: &str =
        include_str!("../../../tests/fixtures/phone-text/display-b-r5-clean-dmesg-d5-20261120.cap");
    /// B-d5's kernel log at 5 s: the arm-smmu context faults.
    const SIGNS: &str =
        include_str!("../../../tests/fixtures/phone-text/display-b-d5-signs-dmesg-d5-20261126.cap");
    /// B-a5's `ps`: the card0-crtc0 thread.
    const PS: &str =
        include_str!("../../../tests/fixtures/phone-text/display-b-a5-ps-20261123.cap");
    /// Android's own command line (no passthrough word): procedures/steps/fixtures/android-b-cmdline.txt, read by the phone's own /proc/cmdline.
    const ANDROID_CMDLINE: &str = include_str!("../fixtures/android-b-cmdline.txt");

    fn row_of(dmesg: &str) -> Row {
        score(5, 13, dmesg)
    }

    fn signs_of(dmesg: &str, ps: &str, cmdline: &str, connected: &str) -> Signs {
        Signs {
            row: row_of(dmesg),
            passthrough: Some(has_passthrough(cmdline)),
            autorefresh_failures: autorefresh_failures(dmesg),
            connected: drm_problem(&drm_of(connected)),
            panel_bound: panel_is_bound(dmesg),
            drm_initialized: drm_is_initialized(dmesg),
            crtc_listed: crtc_thread_is_listed(ps),
        }
    }

    const DRM_OK: &str = "card0\ncard0-DSI-1\nrenderD128\nversion\ndsi=connected\n"; // the shape of ~/lab-material/walleye/display-boot-20261126-084113/drm.txt

    /// Android's own 4.4.302 oops text, read from its pstore (research walleye-first-fastboot-boot, 2026-11-18-pk5l2-tried/android-oops-excerpt.txt).
    const ANDROID_PSTORE: &str = include_str!("../../../research/walleye-first-fastboot-boot/2026-11-18-pk5l2-tried/android-oops-excerpt.txt");

    #[test]
    fn a_console_with_the_mainline_boot_is_said_to_hold_it() {
        assert_eq!(mainline_boot_lines(CLEAN).len(), 1);
        assert!(console_verdict(CLEAN).contains("holds the mainline boot"));
    }

    #[test]
    fn android_s_own_console_holds_no_mainline_line() {
        assert!(mainline_boot_lines(ANDROID_PSTORE).is_empty());
        assert!(console_verdict(ANDROID_PSTORE).contains("holds no mainline kernel line"));
    }

    #[test]
    fn the_captured_b_r5_log_has_passthrough_on_its_command_line() {
        assert_eq!(command_line_has_passthrough(CLEAN), Some(true));
    }

    #[test]
    fn android_s_own_command_line_has_no_passthrough() {
        assert!(!has_passthrough(ANDROID_CMDLINE));
    }

    #[test]
    fn the_captured_clean_log_is_clean_in_every_sign_but_the_command_line() {
        let s = signs_of(CLEAN, PS, ANDROID_CMDLINE, DRM_OK);
        assert_eq!(
            (s.row.smmu_faults, s.row.rcg_warnings, s.row.dsi_110),
            (0, 0, 0)
        );
        assert!(
            s.row.loaded(),
            "13 of 13 modules ok and the init through its list: {:?}",
            s.row
        );
        assert!(s.panel_bound && s.drm_initialized && s.crtc_listed);
        assert_eq!(problems(&s), Vec::<String>::new());
        assert!(report(&s).contains("verdict display-boot-clean"));
    }

    #[test]
    fn a_command_line_with_passthrough_is_not_the_smmu_on_image() {
        let s = signs_of(CLEAN, PS, &kernel_command_line(CLEAN), DRM_OK);
        assert_eq!(problems(&s).len(), 1);
        assert!(problems(&s)[0].contains("iommu.passthrough"));
    }

    #[test]
    fn the_captured_log_with_signs_counts_the_smmu_faults() {
        let s = signs_of(SIGNS, PS, ANDROID_CMDLINE, DRM_OK);
        assert!(s.row.smmu_faults > 0, "{:?}", s.row);
        let p = problems(&s);
        assert!(
            p.iter().any(|l| l.contains("arm-smmu context fault")),
            "{p:?}"
        );
        assert!(report(&s).contains("verdict not clean"));
    }

    #[test]
    fn a_connector_that_is_not_connected_is_said() {
        let s = signs_of(
            CLEAN,
            PS,
            ANDROID_CMDLINE,
            "card0\ncard0-DSI-1\ndsi=disconnected\n",
        );
        let p = problems(&s);
        assert_eq!(p.len(), 1);
        assert!(p[0].contains("disconnected"), "{p:?}");
    }

    #[test]
    fn a_log_cut_before_the_bind_has_no_panel_and_no_drm() {
        let cut: String = CLEAN
            .lines()
            .take_while(|l| !l.contains("c994000.dsi.0"))
            .map(|l| format!("{l}\n"))
            .collect();
        assert!(!panel_is_bound(&cut));
        assert!(!drm_is_initialized(&cut));
    }

    #[test]
    fn a_failed_panel_probe_is_not_a_bound_panel() {
        let probe = CLEAN
            .lines()
            .find(|l| l.contains("probe with driver msm_serial failed"))
            .expect("the capture has a failed probe line"); // a real line, the driver named by the panel's
        let failed = format!(
            "{CLEAN}{}\n",
            probe
                .replace("msm_serial c16f000.serial", PANEL)
                .replace("msm_serial", "panel-samsung-s6e3fa5")
        );
        assert!(!panel_is_bound(&failed));
    }

    #[test]
    fn ps_without_the_crtc_thread_lists_none() {
        let without: String = PS
            .lines()
            .filter(|l| !l.contains(CRTC_THREAD))
            .map(|l| format!("{l}\n"))
            .collect();
        assert!(crtc_thread_is_listed(PS));
        assert!(!crtc_thread_is_listed(&without));
    }

    #[test]
    fn a_log_without_its_command_line_is_not_a_pass() {
        let cut: String = CLEAN
            .lines()
            .filter(|l| !l.contains(CMDLINE))
            .map(|l| format!("{l}\n"))
            .collect();
        assert_eq!(command_line_has_passthrough(&cut), None);
        let mut s = signs_of(&cut, PS, ANDROID_CMDLINE, DRM_OK);
        s.passthrough = command_line_has_passthrough(&cut);
        assert!(problems(&s)
            .iter()
            .any(|l| l.contains("no command line to read")));
    }

    #[test]
    fn autorefresh_failures_are_counted_from_the_captured_log_with_signs() {
        assert_eq!(autorefresh_failures(SIGNS), 1);
        assert_eq!(autorefresh_failures(CLEAN), 0);
        assert!(problems(&signs_of(SIGNS, PS, ANDROID_CMDLINE, DRM_OK))
            .iter()
            .any(|l| l.contains("autorefresh")));
    }

    #[test]
    fn the_drm_state_command_mounts_debugfs_first() {
        assert!(DRM_STATE_COMMAND.starts_with("mount -t debugfs none /sys/kernel/debug"));
        assert!(DRM_STATE_COMMAND.contains("/sys/kernel/debug/dri/0/state"));
    }

    #[test]
    fn the_report_says_the_crtc_state_is_not_known_yet_rather_than_missing() {
        let s = signs_of(CLEAN, PS, ANDROID_CMDLINE, DRM_OK);
        assert!(report(&s).contains("crtc_active not known yet"));
    }
}
