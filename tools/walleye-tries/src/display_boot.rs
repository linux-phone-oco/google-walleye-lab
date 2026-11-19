// SPDX-License-Identifier: MIT OR Apache-2.0
//! The display-at-boot try's steps (item walleye-display-at-boot): B1 once from memory; its init, not a host script,
//! loads the display modules; the shell answers, every module of the ramdisk's /display-modules.list has said
//! `walleye-display: ok` in the kernel log, card0 and its DSI-1 connector read connected, the display lines of the
//! kernel log are recorded, the author is asked to look at the glass, and the phone goes back to Android.
//! Each method is one step of procedures/display-at-boot-on-the-pixel-2.rs, returns what that step runs and fails
//! with what it saw. Nothing is written to the phone's storage and no module is loaded from here: the init holds the
//! self-reboot itself for 600 s after it starts (init-display, the deadman the image carries), then the phone
//! restarts by itself.

use crate::display::{
    drawer_has_drawn, drawer_rc, module_file, module_lines, DRAWER_LOG, DRAW_WAIT_S, END_SLACK_S,
    HOLD_FILE,
};
use crate::lab::Lab;
use crate::settings::Settings;
use crate::touch::redact;
use std::rc::Rc;
use std::time::{Duration, Instant};

type Done = Result<(), String>;

/// The init's own words in the kernel log: `walleye-display: insmod PATH`, then `walleye-display: ok` or `FAILED rc=N`.
const SAYS: &str = "walleye-display: ";
/// What the init says to the kernel log (as a banner step) when it has been through the whole list.
pub const LOOP_DONE: &str = "display modules: done";
/// The card and the connector the proof names.
const CARD: &str = "card0";
const CONNECTOR: &str = "DSI-1";

/// What the words said to the author are.
pub const EXPECT: &str = "The phone's own init loads the display modules at boot; then the run writes the backlight once. The glass may show the kernel log (white text on black) or the init's console, stay lit and blank, or stay dark: each is a finding, say which. A second look follows with four stripes. The phone restarts into Android by itself about 10 minutes after it booted, and the run sends it back when it is done.";
pub const LOOK_FOR: &str = "the kernel log's text or the init's console on the glass (say what you see, or that it is blank or dark)";
pub const EXPECT_STRIPES: &str = "Now the run draws four vertical stripes, red, green, blue and white, from left to right, and holds them. Look at the glass at once: they stay for the time said below and then the screen goes black.";
pub const LOOK_FOR_STRIPES: &str = "four vertical stripes, red, green, blue, white, HELD on the glass (say what you see: the stripes, the kernel log text over them, blank or dark)";

/// What one module's lines in the kernel log say.
#[derive(Debug, Clone, PartialEq)]
pub enum Load {
    Ok,
    Failed(i32),
    /// announced (`insmod PATH`), no result after it
    Unfinished,
}

/// What the init said to the kernel log, the text after `walleye-display: `, of each line that has it.
fn says(dmesg: &str) -> impl Iterator<Item = &str> {
    dmesg
        .lines()
        .filter_map(|l| l.split_once(SAYS).map(|(_, said)| said.trim()))
}

/// What the init said about each module it announced, in the order it said it.
pub fn loads(dmesg: &str) -> Vec<(String, Load)> {
    let mut seen: Vec<(String, Load)> = Vec::new();
    for said in says(dmesg) {
        if said.starts_with("insmod ") {
            seen.push((module_file(said).to_string(), Load::Unfinished));
        } else if let Some(last) = seen.last_mut().filter(|l| l.1 == Load::Unfinished) {
            if said == "ok" {
                last.1 = Load::Ok;
            } else if let Some(rc) = said.strip_prefix("FAILED rc=").and_then(|n| n.parse().ok()) {
                last.1 = Load::Failed(rc);
            }
        }
    }
    seen
}

/// How many `walleye-display: ok` lines the kernel log has.
pub fn ok_lines(dmesg: &str) -> usize {
    says(dmesg).filter(|said| *said == "ok").count()
}

/// The init has been through the whole list (it says so after the last line).
pub fn loop_is_done(dmesg: &str) -> bool {
    dmesg.lines().any(|l| l.trim_end().ends_with(LOOP_DONE))
}

/// The module files of the phone's /display-modules.list, in its order.
pub fn module_names(list: &str) -> Vec<String> {
    module_lines(list)
        .iter()
        .map(|l| module_file(l).to_string())
        .collect()
}

/// The modules of `names` that did not say ok, each with what the log says of it.
pub fn not_ok(names: &[String], dmesg: &str) -> Vec<String> {
    let seen = loads(dmesg);
    names
        .iter()
        .filter_map(
            |n| match seen.iter().find(|(m, _)| m == n).map(|(_, l)| l) {
                Some(Load::Ok) => None,
                Some(Load::Failed(rc)) => Some(format!("{n} (FAILED rc={rc})")),
                Some(Load::Unfinished) => Some(format!("{n} (started, no result)")),
                None => Some(format!("{n} (not started)")),
            },
        )
        .collect()
}

/// All `want` modules of the list said ok, or what is missing, named.
pub fn every_module_said_ok(names: &[String], want: u64, dmesg: &str) -> Done {
    if names.len() as u64 != want {
        return Err(format!(
            "the phone's /display-modules.list has {} insmod lines, the procedure expects {want}",
            names.len()
        ));
    }
    let ok = ok_lines(dmesg);
    let missing = not_ok(names, dmesg);
    if missing.is_empty() && ok as u64 == want {
        return Ok(());
    }
    if missing.is_empty() {
        return Err(format!(
            "{ok} 'walleye-display: ok' lines, the list has {want} modules"
        ));
    }
    Err(format!(
        "{ok} of {want} display modules said ok; not ok: {}",
        missing.join(", ")
    ))
}

/// The kernel log's lines of the display path (the init's own, drm, msm, the DSI host, the panel, the console on it).
pub fn display_lines(dmesg: &str) -> Vec<&str> {
    const WORDS: [&str; 9] = [
        "walleye-display",
        "drm",
        "msm",
        "dsi",
        "panel",
        "s6e3fa5",
        "fbcon",
        "frame buffer",
        "framebuffer",
    ];
    dmesg
        .lines()
        .filter(|l| {
            let l = l.to_lowercase();
            WORDS.iter().any(|w| l.contains(w))
        })
        .collect()
}

/// The command that reads the DRM class: what is listed, and what the DSI-1 connector of card0 says.
pub fn drm_command() -> String {
    format!(
        "for e in /sys/class/drm/*; do echo ${{e##*/}}; done; echo dsi=$(cat /sys/class/drm/{CARD}-{CONNECTOR}/status 2>/dev/null)"
    )
}

/// The command that writes the panel's backlight once through sysfs (item walleye-display-backlight-sysfs-write): half of
/// max_brightness to the first /sys/class/backlight/*/brightness, and what it did said as `backlight-wrote=DIR=VALUE`.
pub fn backlight_command() -> String {
    "for d in /sys/class/backlight/*; do [ -e $d/brightness ] || continue; m=$(cat $d/max_brightness); v=$((m/2)); echo $v > $d/brightness 2>/dev/null; echo backlight-wrote=${d##*/}=$v-rc=$?; break; done; ls /sys/class/backlight".into()
}

/// The backlight was written: the value, or what stands between the command and a write.
pub fn backlight_written(said: &str) -> Result<String, String> {
    said.lines()
        .find_map(|l| l.trim().strip_prefix("backlight-wrote=").map(String::from))
        .ok_or_else(|| {
            format!(
                "no backlight was written (/sys/class/backlight lists: {})",
                said.split_whitespace().collect::<Vec<_>>().join(" ")
            )
        })
}

/// The kernel log's lines about the backlight and the -110 timeout the first write may bring (research walleye-display-backlight-timeout).
pub fn backlight_lines(dmesg: &str) -> Vec<&str> {
    dmesg
        .lines()
        .filter(|l| {
            let l = l.to_lowercase();
            l.contains("backlight") || l.contains("-110")
        })
        .collect()
}

/// The command that starts the stripe drawer detached (a shell session that ends must not end it): its log and its exit
/// code as `stripes-rc=N-end`. The init's own hold (600 s from boot) keeps the phone up; nothing is released from here.
pub fn stripes_command(hold: u64) -> String {
    format!(
        "rm -f {DRAWER_LOG}; setsid sh -c '/bin/walleye-stripes --seconds {hold} > {DRAWER_LOG} 2>&1; echo stripes-rc=$?-end >> {DRAWER_LOG}' </dev/null >/dev/null 2>&1 & echo started-$((6*7))"
    )
}

/// The drawer's log says the console took `mode` (KD_GRAPHICS or KD_TEXT) with 0.
pub fn console_mode_is(log: &str, mode: &str) -> Done {
    let want = format!("KDSETMODE {mode} on ");
    match log.lines().find(|l| l.contains(&want)) {
        Some(l) if l.trim_end().ends_with("= 0") => Ok(()),
        Some(l) => Err(format!("the console did not take {mode}: {}", l.trim())),
        None => match log
            .lines()
            .find(|l| l.contains(&format!("KDSETMODE {mode}:")))
        {
            Some(l) => Err(l.trim().to_string()),
            None => Err(format!("the drawer's log has no KDSETMODE {mode} line")),
        },
    }
}

/// The drawer's log says the console went to KD_GRAPHICS before the draw and back to KD_TEXT after, each with 0.
pub fn console_went_graphics_and_back(log: &str) -> Done {
    console_mode_is(log, "KD_GRAPHICS")?;
    console_mode_is(log, "KD_TEXT")
}

/// What the drawer said of the console: /proc/consoles, each vtcon's name and bind, and the KDSETMODE results.
pub fn console_lines(log: &str) -> Vec<&str> {
    log.lines()
        .filter(|l| l.contains("/proc/consoles") || l.contains("vtcon") || l.contains("KDSETMODE"))
        .collect()
}

/// What `drm_command` printed.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Drm {
    pub entries: Vec<String>,
    pub status: String,
}

/// Drops terminal colour sequences (`ESC [ ... letter`) the phone's `ls` may print.
fn without_colour(s: &str) -> String {
    let mut out = String::new();
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\x1b' {
            if it.peek() == Some(&'[') {
                it.next();
                for d in it.by_ref() {
                    if d.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub fn drm_of(said: &str) -> Drm {
    let said = &without_colour(said);
    let mut d = Drm::default();
    for l in said.lines().map(str::trim) {
        match l.strip_prefix("dsi=") {
            Some(status) => d.status = status.trim().to_string(),
            None => d.entries.extend(l.split_whitespace().map(String::from)),
        }
    }
    d
}

/// What stands between the DRM class and "card0 and DSI-1 connected", none when it is there.
pub fn drm_problem(d: &Drm) -> Option<String> {
    if !d.entries.iter().any(|e| e == CARD) {
        return Some(format!(
            "{CARD} is not in /sys/class/drm (it lists: {})",
            d.entries.join(" ")
        ));
    }
    if d.status != "connected" {
        return Some(format!(
            "{CARD}-{CONNECTOR} says {:?}, not connected",
            d.status
        ));
    }
    None
}

/// The command that says the init's hold is on and the list is in the image.
pub fn hold_command() -> String {
    format!(
        "test -e {HOLD_FILE} && echo hold-is-$((6*7)); test -s /display-modules.list && echo list-is-$((6*7))"
    )
}

/// The display-at-boot try: its settings and the lab it shares with the other tries. Cheap to clone: each step takes its own.
#[derive(Clone)]
pub struct DisplayBoot {
    s: Rc<Settings>,
    lab: Lab,
}

impl DisplayBoot {
    pub fn new(s: Settings) -> DisplayBoot {
        let s = Rc::new(s);
        DisplayBoot {
            lab: Lab::new(s.clone()),
            s,
        }
    }

    /// The battery is above the lab's minimum (the shared step).
    pub fn battery_is_good(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.battery_is_good()
    }

    /// The image is the one built: its sha256 is its SHA256SUMS line's and the one device.toml records.
    pub fn image_is_the_built_one(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || {
            crate::host::image_is_the_built_one(&d.s.image)?;
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

    /// Boot B1 from the phone's fastboot, from memory: nothing is written to it.
    pub fn boot_image(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || d.lab.boot_image(&d.s.image)
    }

    /// The shell answers by itself (no host script has loaded anything), the init's hold of the self-reboot is on
    /// and the image has the module list.
    pub fn shell_answers_and_the_hold_is_on(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || {
            let mut sh = d.lab.shell_when_up()?;
            let said = sh.run(&hold_command(), Duration::from_secs(10))?;
            for (word, what) in [
                (
                    "hold-is-42",
                    "the init's hold is not on (the init is not init-display: the phone restarts at 180 s)",
                ),
                ("list-is-42", "the image has no /display-modules.list"),
            ] {
                if !said.contains(word) {
                    return Err(what.into());
                }
            }
            Ok(())
        }
    }

    /// Every module of the list said ok in the kernel log, polled until the init has been through the list or
    /// `device_wait` is over; what the log said is kept either way.
    pub fn every_module_said_ok(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || {
            let mut sh = d.lab.shell(10)?;
            let list = sh.run("cat /display-modules.list", Duration::from_secs(10))?;
            let names = module_names(&list);
            let until = Instant::now() + Duration::from_secs(d.s.device_wait);
            let (dmesg, finished) = loop {
                let dmesg = sh.run("dmesg", Duration::from_secs(20))?;
                if loop_is_done(&dmesg) {
                    break (dmesg, true);
                }
                if Instant::now() >= until {
                    break (dmesg, false);
                }
                std::thread::sleep(Duration::from_secs(2));
            };
            let verdict = every_module_said_ok(&names, d.s.modules, &dmesg);
            if verdict.is_err() {
                let _ = d.lab.save("dmesg.txt", &redact(&dmesg));
            }
            verdict.map_err(|e| {
                if finished {
                    e
                } else {
                    format!(
                        "{e} (the init had not said '{LOOP_DONE}' {} s after the shell answered)",
                        d.s.device_wait
                    )
                }
            })?;
            println!(
                "  {} of {} display modules said ok",
                ok_lines(&dmesg),
                d.s.modules
            );
            Ok(())
        }
    }

    /// The DRM class lists card0 and its DSI-1 connector says connected, polled: the binds are asynchronous.
    pub fn connector_is_connected(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || {
            let until = Instant::now() + Duration::from_secs(d.s.device_wait);
            let mut sh = d.lab.shell(10)?;
            loop {
                let said = sh.run(&drm_command(), Duration::from_secs(10))?;
                let problem = drm_problem(&drm_of(&said));
                if problem.is_none() {
                    d.lab.save("drm.txt", &said)?;
                    println!("  {CARD} and {CARD}-{CONNECTOR}: connected");
                    return Ok(());
                }
                if Instant::now() >= until {
                    let _ = d.lab.save("drm.txt", &said);
                    return Err(format!(
                        "{} (within {} s)",
                        problem.unwrap_or_default(),
                        d.s.device_wait
                    ));
                }
                std::thread::sleep(Duration::from_secs(2));
            }
        }
    }

    /// The kernel log is kept whole (the serial redacted) and its display lines apart; the last few are said.
    pub fn display_lines_are_recorded(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || {
            let mut sh = d.lab.shell(10)?;
            let dmesg = sh.run("dmesg", Duration::from_secs(20))?;
            let dmesg = redact(&dmesg);
            d.lab.save("dmesg.txt", &dmesg)?;
            let lines = display_lines(&dmesg);
            if lines.is_empty() {
                return Err("the kernel log has no display line".into());
            }
            d.lab.save("display-lines.txt", &lines.join("\n"))?;
            println!("  {} display lines recorded; the last:", lines.len());
            for l in lines.iter().rev().take(6).rev() {
                println!("  {l}");
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

    /// The backlight is written once through sysfs, then the kernel log is read again: the value and every backlight line
    /// (the -110 the first write may bring) are kept and said; a log line is a finding here, not a failure.
    pub fn backlight_is_written(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || {
            let mut sh = d.lab.shell(10)?;
            let said = sh.run(&backlight_command(), Duration::from_secs(10))?;
            let wrote = backlight_written(&said)?;
            std::thread::sleep(Duration::from_secs(2));
            let dmesg = redact(&sh.run("dmesg", Duration::from_secs(20))?);
            d.lab.save("dmesg-after-backlight.txt", &dmesg)?;
            let lines = backlight_lines(&dmesg);
            d.lab.save("backlight-lines.txt", &lines.join("\n"))?;
            println!(
                "  wrote {wrote}; {} backlight lines in the kernel log:",
                lines.len()
            );
            for l in lines.iter().rev().take(4).rev() {
                println!("  {l}");
            }
            Ok(())
        }
    }

    /// Say what the second look is for, before the drawer starts.
    pub fn stripes_are_announced(&self) -> impl FnOnce() -> Done + 'static {
        let hold = self.s.hold;
        move || {
            println!("  {EXPECT_STRIPES} (held {hold} s)");
            Ok(())
        }
    }

    /// The drawer is started detached; its log says the stripes are drawn and the console went to KD_GRAPHICS with 0.
    pub fn stripes_are_drawn_held(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || {
            let mut sh = d.lab.shell(10)?;
            let said = sh.run(&stripes_command(d.s.hold), Duration::from_secs(10))?;
            if !said.contains("started-42") {
                return Err(format!("the drawer did not start: {said}"));
            }
            let until = Instant::now() + Duration::from_secs(DRAW_WAIT_S);
            let log = loop {
                let log = sh.run(&format!("cat {DRAWER_LOG}"), Duration::from_secs(10))?;
                if drawer_has_drawn(&log) || drawer_rc(&log).is_some() || Instant::now() >= until {
                    break log;
                }
                std::thread::sleep(Duration::from_secs(1));
            };
            d.lab.save("stripes.log", &log)?;
            if !drawer_has_drawn(&log) {
                return Err(format!(
                    "the drawer did not draw within {DRAW_WAIT_S} s; its log's last line: {}",
                    crate::host::last_line(&log)
                ));
            }
            for l in console_lines(&log) {
                println!("  {l}");
            }
            console_mode_is(&log, "KD_GRAPHICS")
        }
    }

    /// The drawer ended with 0 after its hold and put the console back to KD_TEXT with 0; the whole log is kept.
    pub fn stripes_have_ended(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || {
            let until = Instant::now() + Duration::from_secs(d.s.hold + END_SLACK_S);
            let mut sh = d.lab.shell(10)?;
            loop {
                let log = sh.run(&format!("cat {DRAWER_LOG}"), Duration::from_secs(10))?;
                if let Some(rc) = drawer_rc(&log) {
                    d.lab.save("stripes.log", &log)?;
                    if rc != 0 {
                        return Err(format!("the drawer ended with {rc}"));
                    }
                    return console_went_graphics_and_back(&log);
                }
                if Instant::now() >= until {
                    return Err(format!(
                        "the drawer had not ended {} s after the step began (the init's own 600 s hold may have run out and the phone restarted: the author answered too late)",
                        d.s.hold + END_SLACK_S
                    ));
                }
                std::thread::sleep(Duration::from_secs(2));
            }
        }
    }

    /// The way back: the phone ends on Android (the shared step; `reboot -f` over the shell when it is not there yet).
    pub fn way_back(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.way_back()
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

    const NAMES: [&str; 13] = [
        "drm.ko",
        "drm_kms_helper.ko",
        "drm_client_lib.ko",
        "cec.ko",
        "drm_display_helper.ko",
        "drm_dp_aux_bus.ko",
        "gpu-sched.ko",
        "drm_exec.ko",
        "drm_gpuvm.ko",
        "mdt_loader.ko",
        "ubwc_config.ko",
        "msm.ko",
        "panel-samsung-s6e3fa5.ko",
    ];

    fn list() -> String {
        NAMES
            .iter()
            .map(|n| format!("insmod /lib/modules/7.3.0/kernel/drivers/x/{n}\n"))
            .collect()
    }

    fn names() -> Vec<String> {
        NAMES.iter().map(|n| n.to_string()).collect()
    }

    /// The kernel log as the init writes it: the announce line, then ok or FAILED, per module; `fail` names the ones that fail.
    fn dmesg_with(fail: &[(&str, i32)], stop_after: usize) -> String {
        let mut out = String::from("[    1.0] msm: something else\n[    5.0] walleye-banner: step 9: display modules: loading\n");
        for (i, n) in NAMES.iter().enumerate().take(stop_after) {
            out.push_str(&format!(
                "[ {:6.3}] walleye-display: insmod /lib/modules/7.3.0/kernel/drivers/x/{n}\n",
                6.0 + i as f64
            ));
            match fail.iter().find(|(f, _)| f == n) {
                Some((_, rc)) => {
                    out.push_str(&format!("[    7.0] walleye-display: FAILED rc={rc}\n"))
                }
                None => out.push_str("[    7.0] walleye-display: ok\n"),
            }
        }
        if stop_after == NAMES.len() {
            out.push_str("[   30.0] walleye-banner: step 12: display modules: done\n");
        }
        out
    }

    #[test]
    fn the_13_modules_of_the_list_are_named_in_order() {
        let n = module_names(&format!("noise\n{}", list()));
        assert_eq!(n.len(), 13);
        assert_eq!(
            (n[0].as_str(), n[12].as_str()),
            ("drm.ko", "panel-samsung-s6e3fa5.ko")
        );
        assert!(module_names("").is_empty());
    }

    #[test]
    fn the_ok_lines_are_counted_whatever_stands_before_them() {
        assert_eq!(ok_lines(&dmesg_with(&[], 13)), 13);
        assert_eq!(ok_lines(&dmesg_with(&[("msm.ko", 1)], 13)), 12);
        assert_eq!(
            ok_lines("walleye-display: ok\r\n[ 1.0] walleye-display: ok"),
            2
        );
        // the announce lines, the failures and other drivers' ok are not ok lines of the init
        assert_eq!(ok_lines("walleye-display: insmod /x/ok.ko\nwalleye-display: FAILED rc=1\nmmc: ok\nwalleye-display: okay"), 0);
        assert_eq!(ok_lines(""), 0);
    }

    #[test]
    fn each_announced_module_has_its_own_result() {
        let l = loads(&dmesg_with(&[("msm.ko", 1)], 13));
        assert_eq!(l.len(), 13);
        assert_eq!(l[0], ("drm.ko".to_string(), Load::Ok));
        assert_eq!(l[11], ("msm.ko".to_string(), Load::Failed(1)));
        let half = loads(&format!(
            "{}walleye-display: insmod /x/hang.ko\n",
            dmesg_with(&[], 2)
        ));
        assert_eq!(
            half.last().unwrap(),
            &("hang.ko".to_string(), Load::Unfinished)
        );
    }

    #[test]
    fn the_loop_is_done_when_the_init_says_so() {
        assert!(loop_is_done(&dmesg_with(&[], 13)));
        assert!(!loop_is_done(&dmesg_with(&[], 5)));
    }

    #[test]
    fn all_13_ok_passes_and_nothing_else_does() {
        every_module_said_ok(&names(), 13, &dmesg_with(&[], 13)).unwrap();
    }

    #[test]
    fn a_failed_module_a_silent_one_and_an_unfinished_one_are_named() {
        let e = every_module_said_ok(
            &names(),
            13,
            &dmesg_with(&[("msm.ko", 1), ("drm_exec.ko", 127)], 13),
        )
        .unwrap_err();
        assert!(
            e.contains("11 of 13")
                && e.contains("msm.ko (FAILED rc=1)")
                && e.contains("drm_exec.ko (FAILED rc=127)"),
            "{e}"
        );
        let e = every_module_said_ok(&names(), 13, &dmesg_with(&[], 11)).unwrap_err();
        assert!(
            e.contains("11 of 13")
                && e.contains("msm.ko (not started)")
                && e.contains("panel-samsung-s6e3fa5.ko (not started)"),
            "{e}"
        );
        assert!(!e.contains("drm.ko ("), "{e}");
        let hung = format!(
            "{}walleye-display: insmod /x/drm_exec.ko\n",
            dmesg_with(&[], 7)
        );
        assert!(not_ok(&names(), &hung).contains(&"drm_exec.ko (started, no result)".to_string()));
    }

    #[test]
    fn a_list_that_is_not_13_long_is_no_pass_even_when_every_line_said_ok() {
        let short = names()[..12].to_vec();
        let e = every_module_said_ok(&short, 13, &dmesg_with(&[], 12)).unwrap_err();
        assert!(
            e.contains("12") && e.contains("13") && e.contains("list"),
            "{e}"
        );
    }

    #[test]
    fn more_ok_lines_than_modules_is_no_pass() {
        let twice = format!("{}walleye-display: ok\n", dmesg_with(&[], 13));
        let e = every_module_said_ok(&names(), 13, &twice).unwrap_err();
        assert!(e.contains("14") && e.contains("13"), "{e}");
    }

    #[test]
    fn the_display_lines_are_the_ones_of_the_display_path() {
        let dm = "[ 1.0] mmc0: new card\n[ 2.0] walleye-display: ok\n[ 3.0] [drm] Initialized msm 1.13.0\n[ 4.0] panel-samsung-s6e3fa5 ae94000.dsi.0: enabled\n[ 5.0] Console: switching to colour frame buffer device 135x120\n[ 6.0] usb0: link up";
        let l = display_lines(dm);
        assert_eq!(l.len(), 4, "{l:?}");
        assert!(!l.iter().any(|x| x.contains("mmc0") || x.contains("usb0")));
        assert!(display_lines("").is_empty());
    }

    #[test]
    fn the_drm_class_is_read_for_card0_and_the_connector() {
        let up = drm_of("card0\ncard0-DSI-1\nrenderD128\nversion\ndsi=connected");
        assert_eq!(up.status, "connected");
        assert!(up.entries.contains(&"card0".to_string()));
        assert_eq!(drm_problem(&up), None);
        // the connector reads, but there is no card0 listed by itself
        let nocard = drm_of("card0-DSI-1\ndsi=connected");
        assert!(drm_problem(&nocard).unwrap().contains("card0"));
        // card0 and the connector are there, the panel is not seen
        for said in ["card0\ncard0-DSI-1\ndsi=disconnected", "card0\ndsi="] {
            let p = drm_problem(&drm_of(said)).unwrap();
            assert!(p.contains("DSI-1") && p.contains("connected"), "{p}");
        }
        let coloured = drm_of("\x1b[1;36mcard0\x1b[m \x1b[1;36mcard0-DSI-1\x1b[m \x1b[0;0mversion\x1b[m\ndsi=connected");
        assert_eq!(drm_problem(&coloured), None);
        assert_eq!(drm_of(""), Drm::default());
        assert!(drm_problem(&drm_of("")).is_some());
    }

    #[test]
    fn the_phones_commands_are_valid_shell_and_ask_for_what_the_proof_names() {
        for c in [drm_command(), hold_command()] {
            let ok = std::process::Command::new("sh")
                .args(["-n", "-c", &c])
                .status()
                .unwrap();
            assert!(ok.success(), "{c}");
        }
        assert!(drm_command().contains("/sys/class/drm/card0-DSI-1/status"));
        assert!(!drm_command().contains("ls "));
        assert!(hold_command().contains("/tmp/hold"));
        for c in [backlight_command(), stripes_command(60)] {
            let ok = std::process::Command::new("sh")
                .arg("-n")
                .arg("-c")
                .arg(&c)
                .status()
                .unwrap();
            assert!(ok.success(), "{c}");
        }
    }

    #[test]
    fn the_backlight_write_is_read_from_what_the_phone_said() {
        assert_eq!(
            backlight_written("backlight-wrote=panel0-backlight=127-rc=0\npanel0-backlight")
                .unwrap(),
            "panel0-backlight=127-rc=0"
        );
        let e = backlight_written("panel0-backlight\n").unwrap_err();
        assert!(
            e.contains("no backlight was written") && e.contains("panel0-backlight"),
            "{e}"
        );
        assert!(backlight_written("").is_err());
    }

    #[test]
    fn a_rejected_write_is_a_finding_kept_not_a_failure() {
        assert_eq!(
            backlight_written("backlight-wrote=panel0-backlight=127-rc=1\npanel0-backlight")
                .unwrap(),
            "panel0-backlight=127-rc=1"
        );
    }

    #[test]
    fn a_console_that_could_not_be_opened_is_named() {
        let e = console_mode_is(
            "walleye-stripes: KDSETMODE KD_GRAPHICS: /dev/tty0 could not be opened\n",
            "KD_GRAPHICS",
        )
        .unwrap_err();
        assert!(e.contains("could not be opened"), "{e}");
    }
    #[test]
    fn the_backlight_timeout_line_is_found_in_the_kernel_log() {
        let dmesg = "[1] msm: x\n[9.1] panel-samsung-s6e3fa5 0: failed to set brightness: -110\n[9.2] backlight: ok\n";
        assert_eq!(backlight_lines(dmesg).len(), 2);
        assert!(backlight_lines("[1] nothing\n").is_empty());
    }

    #[test]
    fn the_drawer_keeps_the_phone_up_by_the_inits_hold_and_releases_nothing() {
        let c = stripes_command(60);
        assert!(c.contains("--seconds 60") && c.contains("setsid") && c.contains("stripes-rc="));
        assert!(!c.contains("rm -f /tmp/hold"), "{c}");
    }

    #[test]
    fn the_console_must_have_gone_graphics_and_back_with_zero() {
        let good = "walleye-stripes: KDSETMODE KD_GRAPHICS on /dev/tty0 = 0\nwalleye-stripes: KDSETMODE KD_TEXT on /dev/tty0 = 0\n";
        console_went_graphics_and_back(good).unwrap();
        let e = console_went_graphics_and_back(
            "walleye-stripes: KDSETMODE KD_GRAPHICS on /dev/tty0 = 0\n",
        )
        .unwrap_err();
        assert!(e.contains("KD_TEXT"), "{e}");
        let e = console_went_graphics_and_back(
            "walleye-stripes: KDSETMODE KD_GRAPHICS on /dev/tty0 = Inappropriate ioctl for device\nwalleye-stripes: KDSETMODE KD_TEXT on /dev/tty0 = 0\n",
        )
        .unwrap_err();
        assert!(
            e.contains("KD_GRAPHICS") && e.contains("Inappropriate"),
            "{e}"
        );
        assert!(console_went_graphics_and_back("").is_err());
        let log = format!("{good}walleye-stripes: /proc/consoles = tty0 | ttyMSM0\nwalleye-stripes: vtcon1: name = frame buffer device | bind = 1\nwalleye-stripes: plan: x\n");
        assert_eq!(console_lines(&log).len(), 4);
    }

    /// The try against a stand-in phone: the table answers by what the command contains, first row first.
    mod walkthrough {
        use super::*;
        use crate::phone::fake;
        use crate::settings::Try;

        fn out(name: &str) -> std::path::PathBuf {
            std::env::temp_dir().join(format!("wt-display-boot-{name}-{}", std::process::id()))
        }

        fn display_boot(table: Vec<(&'static str, &'static str)>, name: &str) -> DisplayBoot {
            let mut s = Settings::stand_in(Try::DisplayBoot);
            s.port = fake::phone(table, vec![]);
            s.out = out(name);
            s.device_wait = 2;
            DisplayBoot::new(s)
        }

        fn leak(s: String) -> &'static str {
            Box::leak(s.into_boxed_str())
        }

        fn phone(dmesg: String, drm: &'static str) -> Vec<(&'static str, &'static str)> {
            vec![
                ("hold-is-", "hold-is-42\nlist-is-42"),
                ("cat /display-modules.list", leak(list())),
                ("card0-DSI-1", drm),
                ("dmesg", leak(dmesg)),
                ("echo up-", "up-42"),
            ]
        }

        const DRM_UP: &str = "card0\ncard0-DSI-1\nversion\ndsi=connected";
        const DMESG_TAIL: &str =
            "[ 9.0] [drm] Initialized msm 1.13.0\n[ 9.1] panel-samsung-s6e3fa5 0: enabled\n";

        fn good_dmesg() -> String {
            format!("{}{DMESG_TAIL}", dmesg_with(&[], 13))
        }

        const GOOD_STRIPES: &str = "walleye-stripes: KDSETMODE KD_GRAPHICS on /dev/tty0 = 0\nwalleye-stripes: draw: four stripes (red, green, blue, white) on /dev/fb0, held 60 s\nwalleye-stripes: KDSETMODE KD_TEXT on /dev/tty0 = 0\nstripes-rc=0-end\n";

        #[test]
        fn the_backlight_is_written_once_and_the_timeout_line_is_kept() {
            let dmesg = format!(
                "{}[ 9.5] panel-samsung-s6e3fa5 0: backlight write failed: -110\n",
                good_dmesg()
            );
            let mut table = phone(dmesg, DRM_UP);
            table.insert(
                0,
                (
                    "backlight-wrote=",
                    "backlight-wrote=panel0-backlight=127-rc=0\npanel0-backlight",
                ),
            );
            let d = display_boot(table, "backlight");
            d.backlight_is_written()().unwrap();
            let kept =
                std::fs::read_to_string(out("backlight").join("backlight-lines.txt")).unwrap();
            assert!(kept.contains("-110"), "{kept}");
        }

        #[test]
        fn a_phone_without_a_backlight_is_named() {
            let mut table = phone(good_dmesg(), DRM_UP);
            table.insert(0, ("backlight-wrote=", ""));
            let e = display_boot(table, "nobacklight").backlight_is_written()().unwrap_err();
            assert!(e.contains("no backlight was written"), "{e}");
        }

        #[test]
        fn the_stripes_are_drawn_and_end_with_the_console_back_in_text() {
            let mut table = phone(good_dmesg(), DRM_UP);
            table.insert(0, ("cat /tmp/stripes.log", GOOD_STRIPES));
            table.insert(0, ("walleye-stripes --seconds", "started-42"));
            let d = display_boot(table, "stripes");
            d.stripes_are_drawn_held()().unwrap();
            d.stripes_have_ended()().unwrap();
            assert!(out("stripes").join("stripes.log").exists());
        }

        #[test]
        fn a_console_that_refused_graphics_is_named_at_the_draw() {
            let log = GOOD_STRIPES.replace(
                "KD_GRAPHICS on /dev/tty0 = 0",
                "KD_GRAPHICS on /dev/tty0 = Permission denied",
            );
            let mut table = phone(good_dmesg(), DRM_UP);
            table.insert(0, ("cat /tmp/stripes.log", leak(log)));
            table.insert(0, ("walleye-stripes --seconds", "started-42"));
            let e = display_boot(table, "refused").stripes_are_drawn_held()().unwrap_err();
            assert!(
                e.contains("KD_GRAPHICS") && e.contains("Permission denied"),
                "{e}"
            );
        }

        #[test]
        fn a_drawer_that_ended_with_an_error_is_named() {
            let log = GOOD_STRIPES.replace("stripes-rc=0-end", "stripes-rc=1-end");
            let mut table = phone(good_dmesg(), DRM_UP);
            table.insert(0, ("cat /tmp/stripes.log", leak(log)));
            let e = display_boot(table, "rc1").stripes_have_ended()().unwrap_err();
            assert!(e.contains("ended with 1"), "{e}");
        }
        #[test]
        fn the_hold_the_13_oks_the_connector_and_the_record_pass_on_a_phone_that_did_it_itself() {
            let d = display_boot(phone(good_dmesg(), DRM_UP), "pass");
            d.shell_answers_and_the_hold_is_on()().unwrap();
            d.every_module_said_ok()().unwrap();
            d.connector_is_connected()().unwrap();
            d.display_lines_are_recorded()().unwrap();
            let lines = std::fs::read_to_string(out("pass").join("display-lines.txt")).unwrap();
            assert!(lines.contains("Initialized msm") && lines.contains("s6e3fa5"));
            assert_eq!(lines.matches("walleye-display: ok").count(), 13);
            assert!(out("pass").join("dmesg.txt").exists() && out("pass").join("drm.txt").exists());
        }

        #[test]
        fn a_missing_module_is_named_and_the_log_is_kept() {
            let d = display_boot(phone(dmesg_with(&[("msm.ko", 1)], 13), DRM_UP), "missing");
            let e = d.every_module_said_ok()().unwrap_err();
            assert!(
                e.contains("12 of 13") && e.contains("msm.ko (FAILED rc=1)"),
                "{e}"
            );
            assert!(out("missing").join("dmesg.txt").exists());
        }

        #[test]
        fn an_init_that_never_finishes_the_list_says_so_after_the_wait() {
            let d = display_boot(phone(dmesg_with(&[], 5), DRM_UP), "unfinished");
            let e = d.every_module_said_ok()().unwrap_err();
            assert!(
                e.contains("5 of 13") && e.contains("display modules: done"),
                "{e}"
            );
        }

        #[test]
        fn a_connector_that_says_disconnected_is_no_pass() {
            let d = display_boot(
                phone(good_dmesg(), "card0\ncard0-DSI-1\ndsi=disconnected"),
                "disc",
            );
            let e = d.connector_is_connected()().unwrap_err();
            assert!(e.contains("DSI-1") && e.contains("disconnected"), "{e}");
        }

        #[test]
        fn an_image_whose_init_holds_nothing_is_named() {
            let mut t = phone(good_dmesg(), DRM_UP);
            t[0] = ("hold-is-", "list-is-42");
            let e = display_boot(t, "nohold").shell_answers_and_the_hold_is_on()().unwrap_err();
            assert!(e.contains("hold"), "{e}");
        }

        #[test]
        fn an_image_without_the_list_is_named() {
            let mut t = phone(good_dmesg(), DRM_UP);
            t[0] = ("hold-is-", "hold-is-42");
            let e = display_boot(t, "nolist").shell_answers_and_the_hold_is_on()().unwrap_err();
            assert!(e.contains("/display-modules.list"), "{e}");
        }

        #[test]
        fn a_kernel_log_without_display_lines_is_no_record() {
            let d = display_boot(phone("[ 1.0] mmc0: new card".into(), DRM_UP), "nolines");
            let e = d.display_lines_are_recorded()().unwrap_err();
            assert!(e.contains("no display line"), "{e}");
        }

        #[test]
        fn the_image_must_be_the_one_device_toml_records() {
            let dir =
                std::env::temp_dir().join(format!("wt-display-boot-img-{}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("boot-b1.img"), "abc").unwrap();
            let abc = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
            std::fs::write(dir.join("SHA256SUMS-x"), format!("{abc}  boot-b1.img\n")).unwrap();
            let mut s = Settings::stand_in(Try::DisplayBoot);
            s.image = dir.join("boot-b1.img");
            s.image_sha256 = abc.into();
            DisplayBoot::new(s.clone()).image_is_the_built_one()().unwrap();
            s.image_sha256 = "57ab".into();
            let e = DisplayBoot::new(s).image_is_the_built_one()().unwrap_err();
            assert!(e.contains("57ab"), "{e}");
            std::fs::remove_dir_all(dir).ok();
        }
    }
}
