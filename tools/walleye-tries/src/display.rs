// SPDX-License-Identifier: MIT OR Apache-2.0
//! The display try's steps (item walleye-attend-program-display; decision 2026-11-19-walleye-attended-tries-one-command):
//! L6 once from memory, the display modules loaded in the order of the ramdisk's /display-modules.list over the
//! mainline shell, the connector and the panel read, the stripe drawer started detached and POLLED until it has
//! drawn and until it has ended. Each method is one step of procedures/display-stripes-on-the-pixel-2.rs, returns
//! what that step runs and fails with what it saw. Nothing is written to the phone's storage; the drawer's own end
//! gives the phone back: it removes /tmp/hold, and the ramdisk's init then restarts the phone by itself.

use crate::lab::Lab;
use crate::phone::Shell;
use crate::settings::Settings;
use crate::touch::redact;
use std::rc::Rc;
use std::time::{Duration, Instant};

type Done = Result<(), String>;

/// The drawer's log and the file that keeps the ramdisk's init from restarting the phone while the drawer works.
pub const DRAWER_LOG: &str = "/tmp/stripes.log";
pub const HOLD_FILE: &str = "/tmp/hold";
/// How long one module's load may take before it counts as no answer.
const LOAD_WAIT_S: u64 = 20;
/// How long the drawer may take to reach the stripes after it starts (the second L6 try: about 12 s).
const DRAW_WAIT_S: u64 = 40;
/// How long past its hold the drawer may take to end.
const END_SLACK_S: u64 = 30;
/// How long the phone stays up after the drawer ended, for the host to read its code (the author may be slow to say ok).
pub const GRACE_S: u64 = 60;

/// What the words said to the author are.
pub const EXPECT: &str = "Four vertical stripes (red, green, blue, white) appear on the glass for about a minute. The glass may stay dark for a moment while the phone restarts into the test image; a lit screen alone proves nothing until the stripes appear. The phone returns to Android by itself at the end; if it is not there after 4 minutes, hold the power button about 15 seconds.";
pub const LOOK_FOR: &str = "four vertical stripes: red, green, blue, white";

/// The `insmod` lines of the ramdisk's /display-modules.list, in its order.
pub fn module_lines(list: &str) -> Vec<String> {
    list.lines()
        .map(str::trim)
        .filter(|l| l.starts_with("insmod /"))
        .map(String::from)
        .collect()
}

/// The command that loads one module: msm and the panel are loaded with dynamic debug on, to show their probe.
pub fn module_command(line: &str) -> String {
    let path = line.split_whitespace().last().unwrap_or("");
    let name = path.rsplit('/').next().unwrap_or("");
    if name == "msm.ko" || name.starts_with("panel-") {
        format!("{line} dyndbg=+p")
    } else {
        line.to_string()
    }
}

/// The number in `rc=N-stepK` among the lines a load printed.
pub fn rc_of(lines: &str, step: usize) -> Option<i32> {
    let tail = format!("-step{step}");
    lines.lines().find_map(|l| {
        l.trim()
            .strip_prefix("rc=")?
            .strip_suffix(&tail)?
            .parse()
            .ok()
    })
}

/// The command that sets the hold and, on the phone itself, a deadman that removes it after `deadman` seconds: a run
/// that stops (or is killed) anywhere before the drawer's own release still lets the phone restart by itself.
/// It also says whether the image has the drawer and the module list.
pub fn hold_command(deadman: u64) -> String {
    format!(
        "touch {HOLD_FILE}; setsid sh -c 'sleep {deadman}; rm -f {HOLD_FILE}' </dev/null >/dev/null 2>&1 & test -e {HOLD_FILE} && echo hold-is-$((6*7)); test -x /bin/walleye-stripes && echo drawer-is-$((6*7)); test -s /display-modules.list && echo list-is-$((6*7))"
    )
}

/// The command that starts the drawer detached (a shell session that ends must not end it): its log, its exit code
/// as `stripes-rc=N-end`, and then the hold file removed, which lets the phone restart by itself.
pub fn drawer_command(hold: u64) -> String {
    format!(
        "rm -f {DRAWER_LOG}; setsid sh -c '/bin/walleye-stripes --seconds {hold} > {DRAWER_LOG} 2>&1; r=$?; echo stripes-rc=$r-end >> {DRAWER_LOG}; sleep {GRACE_S}; rm -f {HOLD_FILE}' </dev/null >/dev/null 2>&1 & echo started-$((6*7))"
    )
}

/// The drawer's exit code, once its log says it ended.
pub fn drawer_rc(log: &str) -> Option<i32> {
    log.lines().find_map(|l| {
        l.trim()
            .strip_prefix("stripes-rc=")?
            .strip_suffix("-end")?
            .parse()
            .ok()
    })
}

/// The drawer's log says it drew the stripes and is holding them.
pub fn drawer_has_drawn(log: &str) -> bool {
    log.lines().any(|l| l.contains("draw: four stripes"))
}

/// What the drawer read of the DRM connector before it drew: status, enabled, modes.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Connector {
    pub status: String,
    pub enabled: String,
    pub modes: String,
}

/// The connector lines of the drawer's log (`connector before the draw: status = connected`, ...).
pub fn connector_of(log: &str) -> Connector {
    let get = |key: &str| {
        let want = format!("connector before the draw: {key} = ");
        log.lines()
            .find_map(|l| l.split_once(&want).map(|(_, v)| v.trim().to_string()))
            .unwrap_or_default()
    };
    Connector {
        status: get("status"),
        enabled: get("enabled"),
        modes: get("modes"),
    }
}

/// The kernel log's lines that name the panel's driver (its probe, prepare and enable lines).
pub fn panel_lines(dmesg: &str) -> Vec<&str> {
    dmesg
        .lines()
        .filter(|l| l.to_lowercase().contains("s6e3fa5"))
        .collect()
}

/// The display try: its settings and the lab it shares with the other tries. Cheap to clone: each step takes its own.
#[derive(Clone)]
pub struct Display {
    s: Rc<Settings>,
    lab: Lab,
}

impl Display {
    pub fn new(s: Settings) -> Display {
        let s = Rc::new(s);
        Display {
            lab: Lab::new(s.clone()),
            s,
        }
    }

    /// The phone's shell, tried until it answers or `device_wait` is over: the gadget on the bus is not yet telnetd.
    fn shell_when_up(&self) -> Result<Shell, String> {
        let until = Instant::now() + Duration::from_secs(self.s.device_wait);
        loop {
            let up = self.lab.shell(5).and_then(|mut sh| {
                let said = sh.run("echo up-$((6*7))", Duration::from_secs(5))?;
                if said.contains("up-42") {
                    Ok(sh)
                } else {
                    Err(format!("the shell said: {said}"))
                }
            });
            match up {
                Ok(sh) => return Ok(sh),
                Err(e) if Instant::now() >= until => {
                    return Err(format!("no shell within {} s: {e}", self.s.device_wait))
                }
                Err(_) => std::thread::sleep(Duration::from_secs(2)),
            }
        }
    }

    /// What the hold may last at most if nothing releases it: past every wait of the steps, the draw and its hold.
    fn deadman_s(&self) -> u64 {
        self.s.device_wait * 2 + self.s.hold + DRAW_WAIT_S + GRACE_S + 200
    }

    fn save(&self, name: &str, text: &str) -> Done {
        std::fs::create_dir_all(&self.s.out)
            .map_err(|e| format!("{}: {e}", self.s.out.display()))?;
        let _ = std::fs::set_permissions(
            &self.s.out,
            std::os::unix::fs::PermissionsExt::from_mode(0o700),
        );
        std::fs::write(self.s.out.join(name), format!("{text}\n"))
            .map_err(|e| format!("{name}: {e}"))
    }

    fn read_log(&self) -> Result<String, String> {
        let mut sh = self.lab.shell(10)?;
        sh.run(&format!("cat {DRAWER_LOG}"), Duration::from_secs(10))
    }

    /// The battery is above the lab's minimum (the shared step).
    pub fn battery_is_good(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.battery_is_good()
    }

    /// The image is the one built: its sha256 is the one its SHA256SUMS line records.
    pub fn image_is_the_built_one(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || crate::host::image_is_the_built_one(&d.s.image)
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

    /// Boot L6 from the phone's fastboot, from memory: nothing is written to it.
    pub fn boot_image(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || d.lab.boot_image(&d.s.image)
    }

    /// The shell answers (the gadget is up), the hold file is set so the init does not restart the phone while
    /// the modules load, and the phone has the drawer and the module list this try needs.
    pub fn shell_answers_and_holds(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || {
            let mut sh = d.shell_when_up()?;
            let said = sh.run(&hold_command(d.deadman_s()), Duration::from_secs(10))?;
            for (word, what) in [
                ("hold-is-42", "the hold file could not be set"),
                ("drawer-is-42", "the image has no /bin/walleye-stripes"),
                ("list-is-42", "the image has no /display-modules.list"),
            ] {
                if !said.contains(word) {
                    return Err(what.into());
                }
            }
            Ok(())
        }
    }

    /// Every module of the list is loaded in its order, one at a time; after each the shell is asked if it lives.
    pub fn modules_are_loaded(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || {
            let mut sh = d.lab.shell(10)?;
            let list = sh.run("cat /display-modules.list", Duration::from_secs(10))?;
            let lines = module_lines(&list);
            if lines.is_empty() {
                return Err("the phone's /display-modules.list has no insmod line".into());
            }
            let mut steps = String::new();
            for (i, line) in lines.iter().enumerate() {
                let n = i + 1;
                steps.push_str(&format!("step {n}: {line}\n"));
                let said = sh
                    .run(
                        &format!("{}; echo rc=$?-step{n}", module_command(line)),
                        Duration::from_secs(LOAD_WAIT_S),
                    )
                    .map_err(|e| format!("step {n} ({line}) was not answered: {e}"))?;
                if rc_of(&said, n) != Some(0) {
                    let _ = d.save("modules.txt", &steps);
                    return Err(format!("step {n} did not return 0: {line}: {said}"));
                }
                std::thread::sleep(Duration::from_secs(d.s.module_pause));
                let alive = sh
                    .run(
                        &format!("test -e {HOLD_FILE} && echo alive-$((6*7))"),
                        Duration::from_secs(10),
                    )
                    .map_err(|e| {
                        format!("the shell stopped answering after step {n}: {line}: {e}")
                    })?;
                if !alive.contains("alive-42") {
                    return Err(format!(
                        "the phone restarted (the hold file is gone) after step {n}: {line}"
                    ));
                }
            }
            println!("  {} modules loaded, the shell lives", lines.len());
            d.save("modules.txt", &steps)
        }
    }

    /// The DRM connector says connected (the panel path's first answer), polled: the binds are asynchronous.
    pub fn connector_is_connected(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || {
            let until = Instant::now() + Duration::from_secs(d.s.device_wait);
            let mut sh = d.lab.shell(10)?;
            let cmd = "for c in /sys/class/drm/card*-DSI-*; do echo $c $(cat $c/status) $(cat $c/enabled); done";
            loop {
                let said = sh.run(cmd, Duration::from_secs(10))?;
                if said
                    .lines()
                    .any(|l| l.split_whitespace().nth(1) == Some("connected"))
                {
                    d.save("connector.txt", &said)?;
                    println!("  {}", said.lines().next().unwrap_or(""));
                    return Ok(());
                }
                if Instant::now() >= until {
                    let dm = sh
                        .run("dmesg | tail -n 40", Duration::from_secs(20))
                        .unwrap_or_default();
                    let _ = d.save("dmesg-tail.txt", &redact(&dm));
                    return Err(format!(
                        "no DSI connector says connected within {} s (it says: {})",
                        d.s.device_wait,
                        said.lines().next().unwrap_or("nothing")
                    ));
                }
                std::thread::sleep(Duration::from_secs(2));
            }
        }
    }

    /// The drawer is started detached, with the hold it is to keep; the phone's own return follows its end.
    pub fn drawer_is_started(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || {
            let mut sh = d.lab.shell(10)?;
            let said = sh.run(&drawer_command(d.s.hold), Duration::from_secs(10))?;
            if said.contains("started-42") {
                Ok(())
            } else {
                Err(format!("the drawer did not start: {said}"))
            }
        }
    }

    /// The drawer's log says the stripes are on the glass (pan, commit, unblank returned), polled; what it read of
    /// the connector (connected, enabled, the mode) and the panel's lines of the kernel log are said and saved.
    pub fn stripes_are_drawn(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || {
            let until = Instant::now() + Duration::from_secs(DRAW_WAIT_S);
            let log = loop {
                let log = d.read_log().unwrap_or_default();
                if drawer_has_drawn(&log) || drawer_rc(&log).is_some() || Instant::now() >= until {
                    break log;
                }
                std::thread::sleep(Duration::from_secs(1));
            };
            d.save("stripes.log", &log)?;
            if !drawer_has_drawn(&log) {
                return Err(format!(
                    "the drawer did not draw within {DRAW_WAIT_S} s; its log's last line: {}",
                    crate::host::last_line(&log)
                ));
            }
            let c = connector_of(&log);
            println!(
                "  the connector: {}, {}, modes {}",
                c.status, c.enabled, c.modes
            );
            let mut sh = d.lab.shell(10)?;
            let said = sh.run("dmesg | grep -i s6e3fa5", Duration::from_secs(20))?;
            let panel = panel_lines(&said);
            d.save("panel-lines.txt", &redact(&panel.join("\n")))?;
            for l in panel.iter().take(6) {
                println!("  {l}");
            }
            if c.status != "connected" || c.enabled != "enabled" {
                return Err(format!(
                    "the connector was {} and {} when the drawer drew",
                    c.status, c.enabled
                ));
            }
            Ok(())
        }
    }

    /// Say what to expect, before the author is asked.
    pub fn author_is_told(&self) -> impl FnOnce() -> Done + 'static {
        move || {
            println!("  {EXPECT}");
            Ok(())
        }
    }

    /// The drawer has ended, with 0: its hold is over, polled until the hold and a slack past it.
    pub fn drawer_has_ended(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || {
            let until = Instant::now() + Duration::from_secs(d.s.hold + END_SLACK_S);
            loop {
                let log = d.read_log()?;
                if let Some(rc) = drawer_rc(&log) {
                    d.save("stripes.log", &log)?;
                    // best effort: the whole kernel log, while the grace keeps the phone up
                    if let Ok(mut sh) = d.lab.shell(5) {
                        if let Ok(dm) = sh.run("dmesg", Duration::from_secs(20)) {
                            let _ = d.save("dmesg.txt", &redact(&dm));
                        }
                    }
                    return if rc == 0 {
                        Ok(())
                    } else {
                        Err(format!("the drawer ended with {rc}"))
                    };
                }
                if Instant::now() >= until {
                    return Err(format!(
                        "the drawer had not ended {} s after its start",
                        d.s.hold + END_SLACK_S
                    ));
                }
                std::thread::sleep(Duration::from_secs(2));
            }
        }
    }

    /// The phone is back on Android (the shared step).
    pub fn android_is_back(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.android_is_back()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIST: &str = "insmod /lib/modules/7/kernel/drivers/gpu/drm/drm.ko\ninsmod /lib/modules/7/kernel/drivers/gpu/drm/msm/msm.ko\ninsmod /lib/modules/7/kernel/drivers/gpu/drm/panel/panel-samsung-s6e3fa5.ko\n";
    const LOG: &str = "walleye-stripes: connector before the draw: status = connected\nwalleye-stripes: connector before the draw: enabled = enabled\nwalleye-stripes: connector before the draw: modes = 1080x1920\nwalleye-stripes: draw: four stripes (red, green, blue, white) on /dev/fb0, held 60 s\n";

    #[test]
    fn the_list_gives_the_insmod_lines_in_order() {
        let l = module_lines(&format!("noise\n{LIST}"));
        assert_eq!(l.len(), 3);
        assert!(l[0].ends_with("drm.ko") && l[2].ends_with("s6e3fa5.ko"));
    }

    #[test]
    fn msm_and_the_panel_load_with_dynamic_debug_the_rest_without() {
        let l = module_lines(LIST);
        assert_eq!(module_command(&l[0]), l[0]);
        assert!(module_command(&l[1]).ends_with("msm.ko dyndbg=+p"));
        assert!(module_command(&l[2]).ends_with("s6e3fa5.ko dyndbg=+p"));
    }

    #[test]
    fn a_loads_exit_code_is_read_for_its_own_step_only() {
        assert_eq!(rc_of("rc=0-step3", 3), Some(0));
        assert_eq!(rc_of("insmod: x\nrc=1-step3", 3), Some(1));
        assert_eq!(rc_of("rc=0-step3", 4), None);
        assert_eq!(rc_of("", 1), None);
    }

    #[test]
    fn the_hold_has_a_deadman_on_the_phone_and_both_commands_are_valid_shell() {
        let h = hold_command(500);
        assert!(h.contains("sleep 500; rm -f /tmp/hold") && h.contains("setsid"));
        for c in [h, drawer_command(60)] {
            let ok = std::process::Command::new("sh")
                .args(["-n", "-c", &c])
                .status()
                .unwrap();
            assert!(ok.success(), "{c}");
        }
    }

    #[test]
    fn the_drawer_is_detached_and_gives_the_phone_back_when_it_ends() {
        let c = drawer_command(60);
        assert!(c.contains("setsid") && c.contains("--seconds 60"));
        assert!(c.contains("</dev/null") && c.trim_end().contains("& echo started-"));
        // the hold file goes after the exit code is written, whatever the code
        assert!(c.find("stripes-rc=").unwrap() < c.find("sleep 60; rm -f /tmp/hold").unwrap());
    }

    #[test]
    fn the_drawers_log_says_drawn_and_ended_separately() {
        assert!(drawer_has_drawn(LOG));
        assert_eq!(drawer_rc(LOG), None);
        let ended = format!("{LOG}stripes-rc=0-end\n");
        assert_eq!(drawer_rc(&ended), Some(0));
        assert_eq!(drawer_rc("stripes-rc=1-end"), Some(1));
        assert!(!drawer_has_drawn("walleye-stripes: FAILED: open /dev/fb0"));
    }

    #[test]
    fn the_connector_is_read_from_the_drawers_log() {
        let c = connector_of(LOG);
        assert_eq!(
            (c.status.as_str(), c.enabled.as_str(), c.modes.as_str()),
            ("connected", "enabled", "1080x1920")
        );
        assert_eq!(connector_of("nothing"), Connector::default());
    }

    #[test]
    fn the_panels_lines_are_the_ones_naming_its_driver() {
        let dm = "[ 1.0] msm: x\n[ 2.0] panel-samsung-s6e3fa5 0: prepare\n[ 3.0] other\n[ 4.0] S6E3FA5 enable";
        assert_eq!(panel_lines(dm).len(), 2);
    }

    /// The try against a stand-in phone: the table answers by what the command contains, first row first.
    mod walkthrough {
        use super::*;
        use crate::phone::fake;
        use crate::settings::Try;

        fn out(name: &str) -> std::path::PathBuf {
            std::env::temp_dir().join(format!("wt-display-{name}-{}", std::process::id()))
        }

        fn display(table: Vec<(&'static str, &'static str)>, name: &str) -> Display {
            let mut s = Settings::stand_in(Try::Display);
            s.port = fake::phone(table, vec![]);
            s.out = out(name);
            s.module_pause = 0;
            s.device_wait = 2;
            s.hold = 1;
            Display::new(s)
        }

        const DMESG: &str = "[ 1.0] msm: bound\n[ 9.0] panel-samsung-s6e3fa5 0: prepared\n[ 9.1] panel-samsung-s6e3fa5 0: enabled";
        const ENDED: &str = "walleye-stripes: connector before the draw: status = connected\nwalleye-stripes: connector before the draw: enabled = enabled\nwalleye-stripes: connector before the draw: modes = 1080x1920\nwalleye-stripes: draw: four stripes (red, green, blue, white) on /dev/fb0, held 1 s\nstripes-rc=0-end";

        fn phone(log: &'static str) -> Vec<(&'static str, &'static str)> {
            vec![
                ("hold-is-", "hold-is-42\ndrawer-is-42\nlist-is-42"),
                (
                    "cat /display-modules.list",
                    "insmod /lib/modules/7/drm.ko\ninsmod /lib/modules/7/msm.ko\ninsmod /lib/modules/7/panel-samsung-s6e3fa5.ko",
                ),
                ("-step1", "rc=0-step1"),
                ("-step2", "rc=0-step2"),
                ("-step3", "rc=0-step3"),
                ("alive-", "alive-42"),
                ("card*-DSI", "/sys/class/drm/card0-DSI-1 connected enabled"),
                ("setsid", "started-42"),
                ("cat /tmp/stripes.log", log),
                ("dmesg", DMESG),
                ("echo up-", "up-42"),
            ]
        }

        #[test]
        fn the_modules_the_connector_the_drawer_and_its_end_pass_on_a_phone_that_does_them() {
            let d = display(phone(ENDED), "pass");
            d.shell_answers_and_holds()().unwrap();
            d.modules_are_loaded()().unwrap();
            d.connector_is_connected()().unwrap();
            d.drawer_is_started()().unwrap();
            d.stripes_are_drawn()().unwrap();
            d.drawer_has_ended()().unwrap();
            let panel = std::fs::read_to_string(out("pass").join("panel-lines.txt")).unwrap();
            assert_eq!(panel.lines().filter(|l| l.contains("s6e3fa5")).count(), 2);
            let steps = std::fs::read_to_string(out("pass").join("modules.txt")).unwrap();
            assert!(steps.starts_with("step 1: insmod /lib/modules/7/drm.ko"));
        }

        #[test]
        fn a_module_that_does_not_return_zero_names_its_step_and_stops() {
            let mut t = phone(ENDED);
            t[3] = ("-step2", "insmod: Unknown symbol\nrc=1-step2");
            let d = display(t, "badmod");
            let e = d.modules_are_loaded()().unwrap_err();
            assert!(e.contains("step 2") && e.contains("msm.ko"), "{e}");
        }

        #[test]
        fn a_phone_that_restarted_between_loads_is_named() {
            let mut t = phone(ENDED);
            t[5] = ("alive-", "");
            let d = display(t, "restarted");
            let e = d.modules_are_loaded()().unwrap_err();
            assert!(e.contains("after step 1"), "{e}");
        }

        #[test]
        fn a_glass_that_is_not_connected_is_no_pass() {
            let mut t = phone(ENDED);
            t[6] = (
                "card*-DSI",
                "/sys/class/drm/card0-DSI-1 disconnected disabled",
            );
            let d = display(t, "unconnected");
            let e = d.connector_is_connected()().unwrap_err();
            assert!(e.contains("no DSI connector says connected"), "{e}");
        }

        #[test]
        fn a_drawer_that_ended_without_drawing_is_said_before_the_author_is_asked() {
            let d = display(
                phone("walleye-stripes: FAILED: open /dev/fb0\nstripes-rc=1-end"),
                "nodraw",
            );
            let e = d.stripes_are_drawn()().unwrap_err();
            assert!(e.contains("did not draw"), "{e}");
        }

        #[test]
        fn a_drawer_that_ends_with_a_code_is_no_pass() {
            let d = display(
                phone("walleye-stripes: draw: four stripes\nstripes-rc=3-end"),
                "rc3",
            );
            let e = d.drawer_has_ended()().unwrap_err();
            assert!(e.contains("ended with 3"), "{e}");
        }

        #[test]
        fn a_connector_not_enabled_when_drawn_is_no_pass() {
            let log = "walleye-stripes: connector before the draw: status = connected\nwalleye-stripes: connector before the draw: enabled = disabled\nwalleye-stripes: draw: four stripes";
            let d = display(phone(log), "disabled");
            let e = d.stripes_are_drawn()().unwrap_err();
            assert!(e.contains("connected and disabled"), "{e}");
        }

        #[test]
        fn an_image_without_the_drawer_is_named() {
            let mut t = phone(ENDED);
            t[0] = ("hold-is-", "hold-is-42\nlist-is-42");
            let d = display(t, "nodrawer");
            let e = d.shell_answers_and_holds()().unwrap_err();
            assert!(e.contains("/bin/walleye-stripes"), "{e}");
        }
    }
}
