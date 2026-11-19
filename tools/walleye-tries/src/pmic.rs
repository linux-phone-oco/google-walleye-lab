// SPDX-License-Identifier: MIT OR Apache-2.0
//! The PMIC watchdog v1 try's steps (item walleye-attend-program-pmic; decision 2026-11-19-walleye-attended-tries-one-command):
//! image A (next + the six patches) from memory, B when A shows no banner; the reboot test first (arm, clean
//! reboot, Android up and no loop), the bite last (arm, never pet, the phone restarts by itself). Each method is
//! one step of tests/attended/pmic-try.rs, returns what that step runs and fails with what it saw. Nothing is
//! written to the phone's storage: the init's commands (wdt-state, wdt-reboot, wdt-bite) only read and arm.

use crate::host;
use crate::lab::Lab;
use crate::settings::Settings;
use crate::touch::{parse_uptime, redact};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// The bite's timeout (what wdt-arm sets), and how long past it the phone may keep answering the shell.
pub const BITE_S: u64 = 20;
const BITE_SLACK_S: u64 = 25;
/// A shell that went quiet sooner than this after the arming did not wait for the bite: said, not hidden.
const EARLY_S: u64 = 8;

/// What the words said to the author are, in the author's plain words (the item walleye-try-pmic-wdt-v1).
pub mod words {
    pub const TWO_TESTS: &str = "Two short tests of the Pixel 2's watchdog, about 10 minutes of you at the phone in all. Nothing is written to the phone's storage. Stay at the phone; say ok when you are there.";
    pub const REBOOT: &str = "FIRST TEST (reboot): I start the phone from my computer, arm the watchdog and reboot the phone the normal way. You will see the phone shut down and Android start by itself. PASS is that Android comes up and stays up (it does not restart again and again every half minute). Say ok and I begin.";
    pub const BITE: &str = "SECOND TEST (bite): I start the phone again from the computer and make the watchdog bite on purpose, about 20 seconds later. You may see nothing (a lit screen proves nothing); the phone goes quiet and restarts by itself. PASS means it restarts by itself. Say ok and I begin.";
    pub const HOLD_POWER: &str = "Now: if the phone powers off instead, or keeps restarting every half minute, hold the power button about 15 seconds until it starts or shows the bootloader screen - that is what happened last time and it costs only that. Say ok when the phone shows Android or the bootloader screen.";
}

/// What the init's wdt-state line says (one line, key=value words).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct State {
    pub pmic: String,
    pub s2_type: String,
    pub s2_enabled: String,
    pub firmware_type: String,
}

/// The `wdt-state ...` line among the phone's lines.
pub fn parse_state(text: &str) -> Option<State> {
    let line = text
        .lines()
        .find(|l| l.trim_start().starts_with("wdt-state "))?;
    let get = |k: &str| {
        line.split_whitespace()
            .find_map(|w| w.strip_prefix(&format!("{k}=")))
            .map(String::from)
    };
    Some(State {
        pmic: get("pmic")?,
        s2_type: get("s2-type")?,
        s2_enabled: get("s2-enabled")?,
        firmware_type: get("firmware-type").unwrap_or_default(),
    })
}

/// How the S2 type is known: read from the PON registers, or by the driver's own read-back (the probe fails
/// when the type it set does not read back, so a registered PMIC device means 7).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Know {
    Read,
    ByProbe,
}

/// The guard: the driver sets the hard type 7 and the dts cannot choose; anything else read means no arming.
pub fn judge_type(s: &State) -> Result<Know, String> {
    if s.pmic == "none" || s.pmic.is_empty() {
        return Err("no PMIC watchdog device: the driver did not probe; no bite".into());
    }
    match s.s2_type.as_str() {
        "0x07" => Ok(Know::Read),
        "unread" => Ok(Know::ByProbe),
        other => Err(format!(
            "the S2 type reads {other}, not the 0x07 the driver sets: stop, record, no bite"
        )),
    }
}

/// What wdt-reboot / wdt-bite printed: the arming line when it armed, the refusal when it did not.
pub fn arming_said(lines: &[String], goes: &str) -> Result<String, String> {
    if let Some(bad) = lines
        .iter()
        .find(|l| l.contains("FAILED") || l.contains("refused"))
    {
        return Err(format!("not armed: {bad}"));
    }
    let armed = lines
        .iter()
        .find(|l| l.contains("wdt-arm armed"))
        .ok_or_else(|| format!("no arming line; the phone said: {lines:?}"))?;
    if !lines.iter().any(|l| l.contains(goes)) {
        return Err(format!(
            "armed, but the phone never said '{goes}': {lines:?}"
        ));
    }
    Ok(armed.trim().to_string())
}

/// The uptimes read while Android was to stay up never go back: a smaller one is a restart.
pub fn never_restarted(readings: &[u64]) -> Result<(), String> {
    match readings.windows(2).find(|w| w[1] < w[0]) {
        Some(w) => Err(format!(
            "the phone restarted: uptime went from {} s to {} s",
            w[0], w[1]
        )),
        None => Ok(()),
    }
}

/// A boot that began after the arming: its uptime is no more than the time since the arming (plus a little).
pub fn fresh_boot(uptime: u64, since_arming: u64) -> bool {
    uptime <= since_arming + 5
}

/// One of the two images the try may boot: A (next + the six patches) or B (the fallback), with its path.
#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub name: &'static str,
    pub path: PathBuf,
}

#[derive(Default)]
struct Facts {
    /// boots of the image so far: the files of each boot are kept apart (b1-, b2-)
    boots: u32,
    /// the image that booted (A, or B when A showed no banner)
    chosen: Option<Image>,
    know: Option<Know>,
    armed_at: Option<Instant>,
    /// what each step learned, written to summary.txt at the end
    notes: Vec<String>,
}

/// The try: its settings and what it learned. Cheap to clone: each step takes its own.
#[derive(Clone)]
pub struct Pmic {
    s: Rc<Settings>,
    lab: Lab,
    out: Rc<PathBuf>,
    facts: Rc<RefCell<Facts>>,
}

type Done = Result<(), String>;

impl Pmic {
    /// The same try (its facts too) on other settings: the lab follows them.
    #[cfg(test)]
    fn with(&self, s: Settings) -> Pmic {
        let s = Rc::new(s);
        Pmic {
            lab: Lab::new(s.clone()),
            s,
            ..self.clone()
        }
    }

    pub fn new(s: Settings) -> Pmic {
        let out = s.out_dir("pmic-wdt");
        let s = Rc::new(s);
        Pmic {
            lab: Lab::new(s.clone()),
            s,
            out: Rc::new(out),
            facts: Rc::default(),
        }
    }

    fn save(&self, name: &str, text: &str) -> Done {
        std::fs::create_dir_all(&*self.out).map_err(|e| format!("{}: {e}", self.out.display()))?;
        let _ = std::fs::set_permissions(
            &*self.out,
            std::os::unix::fs::PermissionsExt::from_mode(0o700),
        );
        let n = self.facts.borrow().boots;
        let name = if n > 0 && name != "summary.txt" {
            format!("b{n}-{name}")
        } else {
            name.to_string()
        };
        std::fs::write(self.out.join(&name), format!("{text}\n"))
            .map_err(|e| format!("{name}: {e}"))
    }

    fn note(&self, line: String) {
        println!("  {line}");
        self.facts.borrow_mut().notes.push(line);
    }

    /// The phone's shell answers a command now (a connection alone is not an answer).
    fn answers(&self) -> bool {
        self.lab
            .shell(2)
            .and_then(|mut sh| sh.run("cat /proc/uptime", Duration::from_secs(4)))
            .is_ok()
    }

    fn tool(&self, program: &str, args: &[&str]) -> Result<host::Said, String> {
        let mut c = std::process::Command::new(program);
        c.args(["-s", &self.lab.serial()?]);
        c.args(args);
        host::run(c)
    }

    /// Android's uptime in seconds, by adb (None when adb does not see a phone answering).
    fn android_uptime(&self) -> Option<u64> {
        let said = self
            .tool(&self.s.adb, &["shell", "cat", "/proc/uptime"])
            .ok()?;
        if said.ok {
            parse_uptime(said.text.trim())
        } else {
            None
        }
    }

    /// The two images, in the order they are tried: A, then B.
    fn images(&self) -> [Image; 2] {
        [
            Image {
                name: "A",
                path: self.s.image_a.clone(),
            },
            Image {
                name: "B",
                path: self.s.image_b.clone(),
            },
        ]
    }

    /// Both images are the ones built: each sha256 is the one its SHA256SUMS line records.
    pub fn images_are_the_built_ones(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            for i in t.images() {
                host::image_is_the_built_one(&i.path)
                    .map_err(|e| format!("image {}: {e}", i.name))?;
            }
            Ok(())
        }
    }

    /// UFS is off in both images (the gate every image passes before a boot).
    pub fn ufs_is_off_in_both(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            for i in t.images() {
                let path = i.path.to_string_lossy().into_owned();
                let line = t.lab.script("ufs-off-image.sh", &[&path], &[])?;
                if !line.starts_with("pass ") {
                    return Err(format!("image {}: {line}", i.name));
                }
            }
            Ok(())
        }
    }

    fn boot(&self, image: &PathBuf) -> Done {
        let mut c = std::process::Command::new(&self.s.lpoco);
        c.args(["lab", "--device"])
            .arg(self.s.device_toml())
            .args(["fastboot", "boot", "now"])
            .arg(image);
        let said = host::run(c)?;
        if said.ok {
            Ok(())
        } else {
            Err(format!("the boot did not go: {}", said.text))
        }
    }

    /// The battery is above the lab's minimum (the shared step).
    pub fn battery_is_good(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.battery_is_good()
    }

    /// The phone is back on Android (the shared step).
    pub fn android_is_back(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.android_is_back()
    }

    /// The way back, the last step: the phone ends on Android whatever stopped the run (the shared step).
    pub fn way_back(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || {
            if std::env::var_os(crate::watch::CHILD).is_some() {
                return Ok(()); // the watching parent runs it after the child ends, whatever ended it
            }
            lab.way_back()
        }
    }

    /// The phone is in fastboot: already, or taken there from Android (the shared step).
    pub fn phone_is_in_fastboot(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || {
            crate::watch::phone_is_touched(); // the first act on the phone: the watcher then owns the way back
            lab.phone_is_in_fastboot()
        }
    }

    /// Boot image A from fastboot, from memory (never flashed); the banner's gadget is up. When A shows no
    /// banner (the gadget never comes), say so - "next does not boot walleye" - wait for the phone's own restart
    /// and boot B instead. After the first boot the same image is booted again.
    pub fn boot_image(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let chosen = t.facts.borrow().chosen.clone();
            let which: Vec<Image> = match chosen {
                Some(image) => vec![image],
                None => t.images().to_vec(),
            };
            let toml = t.s.device_toml();
            for (i, image) in which.iter().enumerate() {
                let w = image.name;
                t.boot(&image.path)?;
                let up = t.lab.script(
                    "gadget-up.sh",
                    &[&toml.to_string_lossy()],
                    &[("GADGET_WAIT", "120")],
                );
                if up.is_ok() {
                    {
                        let mut f = t.facts.borrow_mut();
                        f.chosen = Some(image.clone());
                        f.boots += 1;
                    }
                    t.note(format!("image {w} booted and shows its banner"));
                    return Ok(());
                }
                if i + 1 == which.len() {
                    return Err(format!(
                        "image {w} shows no banner within 120 s: {}",
                        up.unwrap_err()
                    ));
                }
                t.note(format!(
                    "image {w} shows no banner within 120 s: next does not boot walleye; B is used for both tests"
                ));
                t.android_is_back()()?; // the phone's own restart brings Android back
                t.phone_is_in_fastboot()()?;
            }
            Ok(())
        }
    }

    /// The gadget is up on USB and the shell answers; the phone's dmesg is saved (no serial number in it).
    pub fn shell_answers(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let toml = t.s.device_toml();
            t.lab.script(
                "gadget-up.sh",
                &[&toml.to_string_lossy()],
                &[("GADGET_WAIT", "60")],
            )?;
            let mut sh = t.lab.shell(10)?;
            let dmesg = sh.run("dmesg", Duration::from_secs(20))?;
            if dmesg.trim().is_empty() {
                return Err("the shell answers, but dmesg is empty".into());
            }
            t.save("dmesg.txt", &redact(&dmesg))
        }
    }

    fn read_state(&self) -> Result<State, String> {
        let mut sh = self.lab.shell(10)?;
        let text = sh.run("wdt-state", Duration::from_secs(20))?;
        self.save("wdt-state.txt", &text)?;
        parse_state(&text).ok_or_else(|| format!("no wdt-state line; the phone said: {text:?}"))
    }

    /// The PMIC watchdog probed and the S2 type is the one the driver sets (7): the guard of both tests.
    pub fn s2_type_is_seven(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let s = t.read_state()?;
            let know = judge_type(&s)?;
            t.facts.borrow_mut().know = Some(know);
            // hold the init's own 180 s restart (it waits while /tmp/hold exists) while the author reads the next prompt
            t.lab
                .shell(10)?
                .run("touch /tmp/hold", Duration::from_secs(10))?;
            t.note(format!(
                "{} watchdog probed; S2 type {} ({}); firmware type {}",
                s.pmic,
                s.s2_type,
                match know {
                    Know::Read => "read from the PON registers",
                    Know::ByProbe => "not read; the probe's own read-back says 7",
                },
                if s.firmware_type.is_empty() {
                    "unseen"
                } else {
                    &s.firmware_type
                }
            ));
            Ok(())
        }
    }

    /// After the clean reboot S2 reads off. Unread (the PON registers do not read) is said as not known yet:
    /// the three minutes of Android with no restart are then the proof.
    pub fn s2_is_off_after_the_reboot(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let s = t.read_state()?;
            match s.s2_enabled.as_str() {
                "no" => {
                    t.note("S2 reads OFF after the clean reboot".into());
                    Ok(())
                }
                "unread" => {
                    t.note("S2 after the clean reboot: not known yet (the PON registers do not read)".into());
                    Ok(())
                }
                other => Err(format!(
                    "S2 enabled reads {other} after the clean reboot: the reboot did not stop the watchdog"
                )),
            }
        }
    }

    fn arg(&self) -> Result<&'static str, String> {
        match self.facts.borrow().know {
            Some(Know::ByProbe) => Ok("7 probe"),
            Some(Know::Read) => Ok("7"),
            None => Err("the S2 type was not checked first: no arming".into()),
        }
    }

    /// Arm the PMIC watchdog at 20 s and reboot the phone cleanly at once (patch 2 must stop it on the way).
    pub fn arm_and_reboot(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let mut sh = t.lab.shell(10)?;
            let (lines, _) =
                sh.run_partial(&format!("wdt-reboot {}", t.arg()?), Duration::from_secs(60))?;
            let armed = arming_said(&lines, "rebooting cleanly now")?;
            t.note(format!("armed, then a clean reboot: {armed}"));
            let until = Instant::now() + Duration::from_secs(60);
            while t.answers() {
                if Instant::now() >= until {
                    return Err("the phone's shell still answers 60 s after the reboot".into());
                }
                std::thread::sleep(Duration::from_secs(2));
            }
            Ok(())
        }
    }

    fn stays_up(&self, seconds: u64, what: &str) -> Done {
        let every = (seconds / 6).clamp(1, 10);
        let until = Instant::now() + Duration::from_secs(seconds);
        let mut readings = Vec::new();
        while Instant::now() < until {
            let up = self.android_uptime().ok_or_else(|| {
                format!(
                    "adb lost the phone after {} readings {readings:?}",
                    readings.len()
                )
            })?;
            readings.push(up);
            never_restarted(&readings)?;
            std::thread::sleep(Duration::from_secs(every));
        }
        self.save("android-uptimes.txt", &format!("{what}: {readings:?}"))?;
        self.note(format!(
            "{what}: Android stayed up {seconds} s, uptime {} -> {} s, no restart",
            readings.first().copied().unwrap_or(0),
            readings.last().copied().unwrap_or(0)
        ));
        Ok(())
    }

    /// Android stays up for the wanted time (three minutes): no restart every half minute.
    pub fn android_stays_up(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || t.stays_up(t.s.stay, "after the clean reboot")
    }

    /// After the author's power hold: Android again, up for a minute.
    pub fn android_stays_up_a_minute(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || t.stays_up(t.s.stay.min(60), "after the bite and the power hold")
    }

    /// An Android property by adb: "unread" when adb does not answer or the property is empty.
    fn getprop(&self, name: &str) -> String {
        self.tool(&self.s.adb, &["shell", "getprop", name])
            .ok()
            .filter(|s| s.ok)
            .map(|s| s.text.trim().to_string())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| "unread".into())
    }

    /// After the bite pass, Android says why it booted and from which slot; both are recorded (never judged: a
    /// slot or a reason that cannot be read is said as unread, the bite's own witnesses stand).
    pub fn bootreason_and_slot_are_read(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let reason = t.getprop("ro.boot.bootreason");
            let slot = t.getprop("ro.boot.slot_suffix");
            t.save(
                "android-boot.txt",
                &format!("bootreason [{reason}] slot [{slot}]"),
            )?;
            t.note(format!(
                "Android after the bite: boot reason [{reason}], active slot [{slot}]"
            ));
            Ok(())
        }
    }

    /// Arm the watchdog at 20 s and never pet it: the bite is due.
    pub fn arm_and_bite(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let mut sh = t.lab.shell(10)?;
            let (lines, _) =
                sh.run_partial(&format!("wdt-bite {}", t.arg()?), Duration::from_secs(60))?;
            let armed = arming_said(&lines, "the bite is due in 20 s")?;
            t.facts.borrow_mut().armed_at = Some(Instant::now());
            t.note(format!("armed, not petted: {armed}"));
            Ok(())
        }
    }

    /// The phone restarts by itself: the shell goes quiet at about the bite, and the phone is seen again, on a
    /// boot that began after the arming (adb uptime), or in fastboot - never judged from one witness.
    pub fn restarts_by_itself(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let armed = t.facts.borrow().armed_at.ok_or("the bite was not armed")?;
            let secs = |i: Instant| i.duration_since(armed).as_secs();
            let quiet_by =
                Instant::now() + Duration::from_secs((BITE_S + BITE_SLACK_S).min(t.s.bite_wait));
            let lost = loop {
                let answer = t
                    .lab
                    .shell(2)
                    .and_then(|mut sh| sh.run("cat /proc/uptime", Duration::from_secs(5)));
                match answer {
                    Err(_) => break secs(Instant::now()),
                    Ok(up) if Instant::now() >= quiet_by => {
                        return Err(format!(
                            "no bite: {} s after the arming the phone still answers (uptime {})",
                            secs(Instant::now()),
                            up.trim()
                        ))
                    }
                    Ok(_) => std::thread::sleep(Duration::from_secs(1)),
                }
            };
            if lost < EARLY_S {
                t.note(format!(
                    "the shell went quiet only {lost} s after the arming, before the bite was due: not proof by itself"
                ));
            }
            let until = armed + Duration::from_secs(t.s.bite_wait);
            while Instant::now() < until {
                let now = secs(Instant::now());
                if let Some(up) = t.android_uptime() {
                    if fresh_boot(up, now) {
                        let reason = t.getprop("ro.boot.bootreason");
                        t.save("bite.txt", &format!("lost {lost} s, Android uptime {up} s at {now} s, bootreason [{reason}]"))?;
                        t.note(format!(
                            "BITE: the shell went quiet {lost} s after the arming; Android is back by itself at {now} s with uptime {up} s (a new boot); boot reason [{reason}]"
                        ));
                        return Ok(());
                    }
                } else if t.lab.in_fastboot()? {
                    t.save("bite.txt", &format!("lost {lost} s, fastboot at {now} s"))?;
                    t.note(format!(
                        "BITE: the shell went quiet {lost} s after the arming; the phone is in fastboot by itself at {now} s"
                    ));
                    return Ok(());
                }
                std::thread::sleep(Duration::from_secs(2));
            }
            Err(format!(
                "the shell went quiet {lost} s after the arming, but the phone was not seen again within {} s (powered off? hold power about 15 s, then check)",
                t.s.bite_wait
            ))
        }
    }

    /// The result is saved beside the dmesg, one line a step, and printed.
    pub fn result_is_saved(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let text = t.facts.borrow().notes.join("\n");
            t.save("summary.txt", &text)?;
            println!("  saved in {}", t.out.display());
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STATE: &str = "wdt-state pmic=watchdog1 soc=watchdog0 s2-type=0x07 s2-enabled=no src=PON firmware-type=0x01 probe=pmic-device-registered kline-type=[x] kline-poff=[y]";

    #[test]
    fn the_state_line_is_read_and_the_type_judged() {
        let s = parse_state(&format!("noise\n{STATE}\n")).unwrap();
        assert_eq!(
            (
                s.pmic.as_str(),
                s.s2_type.as_str(),
                s.s2_enabled.as_str(),
                s.firmware_type.as_str()
            ),
            ("watchdog1", "0x07", "no", "0x01")
        );
        assert_eq!(judge_type(&s), Ok(Know::Read));
        let unread = State {
            s2_type: "unread".into(),
            ..s.clone()
        };
        assert_eq!(judge_type(&unread), Ok(Know::ByProbe));
        let warm = State {
            s2_type: "0x01".into(),
            ..s.clone()
        };
        assert!(judge_type(&warm).unwrap_err().contains("no bite"));
        let none = State {
            pmic: "none".into(),
            ..s
        };
        assert!(judge_type(&none).unwrap_err().contains("did not probe"));
        assert_eq!(parse_state("nothing here"), None);
    }

    fn lines(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn arming_is_said_or_refused() {
        let ok = lines(&[
            "wdt-arm armed watchdog1 type=0x07 timeout=20",
            "wdt-bite: x; not petted, the bite is due in 20 s",
        ]);
        assert!(arming_said(&ok, "the bite is due in 20 s")
            .unwrap()
            .contains("armed watchdog1"));
        let refused = lines(&["wdt-reboot: wdt-arm refused: the S2 type reads 0x01"]);
        assert!(arming_said(&refused, "rebooting")
            .unwrap_err()
            .contains("refused"));
        let failed = lines(&["wdt-arm FAILED: the applet is not running"]);
        assert!(arming_said(&failed, "x").unwrap_err().contains("FAILED"));
        let half = lines(&["wdt-arm armed watchdog1"]);
        assert!(arming_said(&half, "rebooting cleanly now")
            .unwrap_err()
            .contains("never said"));
    }

    #[test]
    fn a_smaller_uptime_is_a_restart_and_a_fresh_boot_is_after_the_arming() {
        assert_eq!(never_restarted(&[100, 110, 125]), Ok(()));
        assert!(never_restarted(&[100, 110, 12])
            .unwrap_err()
            .contains("110 s to 12 s"));
        assert!(fresh_boot(30, 60));
        assert!(!fresh_boot(900, 60));
    }
}

#[cfg(test)]
mod walkthrough {
    //! Both tests against a stand-in phone and stand-in host programs (adb, fastboot, lpoco, the lab's gate
    //! scripts), the steps in the entry file's order - no phone, no author.
    use super::*;
    use crate::phone::fake;

    fn lab(name: &str, adb_state: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("wt-pmic-{name}-{}", std::process::id()));
        let hosts = d.join("tests/host");
        std::fs::create_dir_all(&hosts).unwrap();
        std::fs::write(
            d.join("device.toml"),
            "[connection]\naddresses = [\"127.0.0.1\"]\nserial = \"SER1\"\n",
        )
        .unwrap();
        for (script, says) in [
            ("battery-ok.sh", "pass walleye-battery-ok host 4100 mV"),
            ("ufs-off-image.sh", "pass walleye-ufs-off image"),
            ("gadget-up.sh", "pass walleye-gadget-up kernel up"),
            ("idle-android.sh", "pass walleye-idle-android host back"),
        ] {
            std::fs::write(hosts.join(script), format!("echo '{says}'\n")).unwrap();
        }
        // an adb whose uptime grows by 1 s a call (`up-now`: a fresh boot at 5 s), a fastboot that lists the phone
        crate::fixture::executable(
            &d.join("adb"),
            &format!(
                "#!/bin/sh\ncase \"$*\" in\n *uptime*) n=$(cat \"{0}/n\" 2>/dev/null || echo {1}); echo $((n+1)) > \"{0}/n\"; echo \"$n.0 1.0\";;\n *bootreason*) echo watchdog;;\n *slot_suffix*) echo _b;;\n *reboot*) : > \"{0}/bootloader\";;\n *) echo device;;\nesac\n",
                d.display(),
                adb_state
            ),
        );
        crate::fixture::executable(
            &d.join("fastboot"),
            &format!(
                "#!/bin/sh\n[ -e \"{}/bootloader\" ] && echo 'SER1 fastboot'\nexit 0\n",
                d.display()
            ),
        );
        crate::fixture::executable(&d.join("lpoco"), "#!/bin/sh\necho \"$@\" >> \"$0.calls\"\n");
        for w in ["a", "b"] {
            std::fs::write(d.join(format!("boot-wdt1-{w}.img")), w).unwrap();
        }
        let sums = std::process::Command::new("sh")
            .current_dir(&d)
            .args([
                "-c",
                "sha256sum boot-wdt1-a.img boot-wdt1-b.img > SHA256SUMS-wdt1",
            ])
            .status()
            .unwrap();
        assert!(sums.success());
        d
    }

    fn the_try(d: &std::path::Path, port: u16) -> Pmic {
        let mut s = Settings::stand_in(crate::settings::Try::Pmic);
        s.lab = d.to_path_buf();
        s.address = "127.0.0.1".into();
        s.port = port;
        s.lpoco = d.join("lpoco").to_string_lossy().into_owned();
        s.adb = d.join("adb").to_string_lossy().into_owned();
        s.fastboot = d.join("fastboot").to_string_lossy().into_owned();
        s.image_a = d.join("boot-wdt1-a.img");
        s.image_b = d.join("boot-wdt1-b.img");
        s.out_given = Some(d.join("out"));
        s.stay = 6;
        s.bite_wait = 8;
        s.back_wait = 5;
        Pmic::new(s)
    }

    fn phone(state: &'static str, bite: &'static str) -> u16 {
        fake::phone(
            vec![
                ("wdt-state", state),
                ("wdt-reboot", "!wdt-arm armed watchdog1 type=0x07 timeout=20 s2-enabled-before=no after=yes hold=90s\nwdt-reboot: x; rebooting cleanly now"),
                ("wdt-bite", bite),
                ("dmesg", "Linux version 7.3 androidboot.serialno=SECRET\npm8916_wdt probed"),
                ("uptime", "30.0 1.0"),
            ],
            vec![],
        )
    }

    const STATE_OK: &str = "wdt-state pmic=watchdog1 soc=watchdog0 s2-type=0x07 s2-enabled=no src=x firmware-type=0x01 probe=pmic-device-registered kline-type=[] kline-poff=[]";
    const STATE_WARM: &str = "wdt-state pmic=watchdog1 soc=watchdog0 s2-type=0x01 s2-enabled=no src=x firmware-type=0x01 probe=pmic-device-registered kline-type=[] kline-poff=[]";
    const BITE_OK: &str = "!wdt-arm armed watchdog1 type=0x07 timeout=20 s2-enabled-before=no after=yes hold=90s\nwdt-bite: x; not petted, the bite is due in 20 s";

    #[test]
    fn the_reboot_test_then_the_bite_pass_and_the_result_is_saved() {
        let d = lab("pass", "5");
        // the stand-in phone hangs up when it reboots, so the reboot test uses its own phone; the second one then bites
        let t = the_try(&d, phone(STATE_OK, BITE_OK));
        t.images_are_the_built_ones()().unwrap();
        t.ufs_is_off_in_both()().unwrap();
        t.battery_is_good()().unwrap();
        t.boot_image()().unwrap();
        t.shell_answers()().unwrap();
        t.s2_type_is_seven()().unwrap();
        t.arm_and_reboot()().unwrap();
        t.android_is_back()().unwrap();
        t.android_stays_up()().unwrap();
        t.phone_is_in_fastboot()().unwrap();
        // the second boot: a new stand-in phone (the first one has restarted), the same image
        let t2 = t.with(Settings {
            port: phone(STATE_OK, BITE_OK),
            ..(*t.s).clone()
        });
        t2.boot_image()().unwrap();
        t2.s2_is_off_after_the_reboot()().unwrap();
        std::fs::write(d.join("n"), "3").unwrap(); // the bite restarts the phone: a new boot
        t2.arm_and_bite()().unwrap();
        t2.restarts_by_itself()().unwrap();
        t2.result_is_saved()().unwrap();
        let summary = std::fs::read_to_string(d.join("out/summary.txt")).unwrap();
        assert!(
            summary.contains("image A booted")
                && summary.contains("BITE:")
                && summary.contains("S2 reads OFF"),
            "{summary}"
        );
        let dmesg = std::fs::read_to_string(d.join("out/b1-dmesg.txt")).unwrap();
        assert!(dmesg.contains("serialno=REDACTED") && !dmesg.contains("SECRET"));
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn image_a_not_booting_leads_to_b_and_says_so() {
        let d = lab("fallback", "5");
        // a gadget that comes up only for the image B (the last image lpoco was asked to boot)
        std::fs::write(
            d.join("tests/host/gadget-up.sh"),
            format!(
                "tail -n1 \"{}/lpoco.calls\" | grep -q boot-wdt1-b.img && echo 'pass walleye-gadget-up kernel up' && exit 0\necho 'fail walleye-gadget-up no gadget'; exit 1\n",
                d.display()
            ),
        )
        .unwrap();
        let t = the_try(&d, phone(STATE_OK, BITE_OK));
        t.boot_image()().unwrap();
        let notes = t.facts.borrow().notes.join("\n");
        assert!(
            notes.contains("image A shows no banner")
                && notes.contains("next does not boot walleye"),
            "{notes}"
        );
        assert!(notes.contains("image B booted"), "{notes}");
        let chosen = t.facts.borrow().chosen.clone().unwrap();
        assert_eq!(chosen.name, "B");
        assert_eq!(chosen.path, d.join("boot-wdt1-b.img"));
        // the second boot of the test keeps B and does not try A again
        t.boot_image()().unwrap();
        let calls = std::fs::read_to_string(d.join("lpoco.calls")).unwrap();
        let a = calls.matches("boot-wdt1-a.img").count();
        assert_eq!(a, 1, "{calls}");
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn both_images_not_booting_is_an_error_naming_the_last() {
        let d = lab("neither", "5");
        std::fs::write(
            d.join("tests/host/gadget-up.sh"),
            "echo 'fail walleye-gadget-up no gadget'; exit 1\n",
        )
        .unwrap();
        let t = the_try(&d, phone(STATE_OK, BITE_OK));
        let e = t.boot_image()().unwrap_err();
        assert!(e.contains("image B shows no banner"), "{e}");
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn the_boot_reason_and_the_slot_are_read_and_recorded() {
        let d = lab("reason", "5");
        let t = the_try(&d, phone(STATE_OK, BITE_OK));
        t.facts.borrow_mut().boots = 2; // both boots are done by this step: files are kept apart as b2-
        t.bootreason_and_slot_are_read()().unwrap();
        let said = std::fs::read_to_string(d.join("out/b2-android-boot.txt")).unwrap();
        assert!(
            said.contains("bootreason [watchdog]") && said.contains("slot [_b]"),
            "{said}"
        );
        // an adb that does not answer is said as unread, not an error
        let t = Pmic {
            s: Rc::new(Settings {
                adb: "/nonexistent/adb".into(),
                ..(*t.s).clone()
            }),
            ..t
        };
        t.bootreason_and_slot_are_read()().unwrap();
        let notes = t.facts.borrow().notes.join("\n");
        assert!(
            notes.contains("boot reason [unread], active slot [unread]"),
            "{notes}"
        );
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn a_type_that_is_not_seven_stops_before_any_arming() {
        let d = lab("warm", "5");
        let t = the_try(&d, phone(STATE_WARM, BITE_OK));
        t.shell_answers()().unwrap();
        let e = t.s2_type_is_seven()().unwrap_err();
        assert!(e.contains("no bite"), "{e}");
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn a_phone_that_keeps_answering_after_the_bite_is_no_bite() {
        let d = lab("nobite", "5");
        let t = the_try(
            &d,
            phone(
                STATE_OK,
                "wdt-arm armed watchdog1 x\nwdt-bite: x; the bite is due in 20 s",
            ),
        );
        t.s2_type_is_seven()().unwrap();
        t.arm_and_bite()().unwrap();
        let t = Pmic {
            s: Rc::new(Settings {
                bite_wait: 3,
                ..(*t.s).clone()
            }),
            ..t
        };
        let e = t.restarts_by_itself()().unwrap_err();
        assert!(e.contains("no bite"), "{e}");
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn a_refused_arming_does_not_reboot_and_a_restart_in_android_is_caught() {
        let d = lab("refused", "5");
        let t = the_try(
            &d,
            phone(
                STATE_OK,
                "wdt-bite: wdt-arm refused: the S2 type reads 0x01",
            ),
        );
        t.s2_type_is_seven()().unwrap();
        let e = t.arm_and_bite()().unwrap_err();
        assert!(e.contains("refused"), "{e}");
        assert!(never_restarted(&[100, 130, 20]).is_err());
        std::fs::remove_dir_all(d).ok();
    }

    /// A phone that has taken the S2 check and the bite: the shell then goes quiet. `adb_n` is the uptime the
    /// stand-in adb counts from ("gone": no adb answers at all).
    fn bitten(name: &str, adb_n: &str) -> (PathBuf, Pmic) {
        let d = lab(name, adb_n);
        let t = the_try(&d, phone(STATE_OK, BITE_OK));
        let t = if adb_n == "gone" {
            Pmic {
                s: Rc::new(Settings {
                    adb: "/nonexistent/adb".into(),
                    ..(*t.s).clone()
                }),
                ..t
            }
        } else {
            t
        };
        let t = Pmic {
            s: Rc::new(Settings {
                bite_wait: 3,
                ..(*t.s).clone()
            }),
            ..t
        };
        t.s2_type_is_seven()().unwrap();
        t.arm_and_bite()().unwrap();
        (d, t)
    }

    #[test]
    fn the_shell_went_quiet_and_nothing_came_back_is_not_a_pass() {
        let (d, t) = bitten("gone", "gone");
        let e = t.restarts_by_itself()().unwrap_err();
        assert!(
            e.contains("not seen again") && e.contains("hold power"),
            "{e}"
        );
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn an_android_that_was_never_restarted_is_not_the_bite() {
        let (d, t) = bitten("stale", "900");
        let e = t.restarts_by_itself()().unwrap_err();
        assert!(e.contains("not seen again"), "{e}");
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn fastboot_is_the_second_witness_too() {
        let (d, t) = bitten("fb", "gone");
        std::fs::write(d.join("bootloader"), "").unwrap();
        t.restarts_by_itself()().unwrap();
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn no_arming_without_the_type_checked() {
        let d = lab("noguard", "5");
        let t = the_try(&d, phone(STATE_OK, BITE_OK));
        assert!(t.arm_and_bite()().unwrap_err().contains("no arming"));
        std::fs::remove_dir_all(d).ok();
    }
}

#[cfg(test)]
mod the_way_back {
    //! The last step of the try, on stand-in phones: no answer from the author, a failed check and a stop all end
    //! in it; each path of it on its own.
    use super::*;
    use crate::phone::fake;

    /// A lab whose idle-android check always passes: the phone is on Android.
    fn lab_android_already(name: &str) -> (PathBuf, Pmic) {
        lab_with(name, |_| {
            "echo 'pass walleye-idle-android host back'\n".into()
        })
    }

    /// A lab whose idle-android check never passes: Android does not come back.
    fn lab_never_back(name: &str) -> (PathBuf, Pmic) {
        lab_with(name, |_| {
            "echo 'fail walleye-idle-android not yet'; exit 1\n".into()
        })
    }

    /// A lab whose idle-android check passes once the stand-in phone has been told to restart (the file `marker`
    /// is in the lab's folder): no clock decides it.
    fn lab_back_after(name: &str, marker: &str) -> (PathBuf, Pmic) {
        lab_with(name, |d| {
            format!(
                "[ -e \"{0}/{1}\" ] && echo 'pass walleye-idle-android host back' && exit 0\necho 'fail walleye-idle-android not yet'; exit 1\n",
                d.display(),
                marker
            )
        })
    }

    fn lab_with(name: &str, check: impl Fn(&std::path::Path) -> String) -> (PathBuf, Pmic) {
        let d = std::env::temp_dir().join(format!("wt-back-{name}-{}", std::process::id()));
        let hosts = d.join("tests/host");
        std::fs::create_dir_all(&hosts).unwrap();
        std::fs::write(d.join("device.toml"), "[connection]\nserial = \"SER1\"\n").unwrap();
        std::fs::write(hosts.join("idle-android.sh"), check(&d)).unwrap();
        let fastboot = d.join("fastboot");
        crate::fixture::executable(
            &fastboot,
            "#!/bin/sh\nif [ \"$3\" = reboot ]; then : > \"$0.rebooted\"; exit 0; fi; [ -e \"$0.listed\" ] && echo 'SER1 fastboot'\nexit 0\n",
        );
        let mut s = Settings::stand_in(crate::settings::Try::Pmic);
        s.lab = d.clone();
        s.address = "127.0.0.1".into();
        s.fastboot = d.join("fastboot").to_string_lossy().into_owned();
        s.back_wait = 3;
        (d, Pmic::new(s))
    }

    fn on_port(t: &Pmic, port: u16) -> Pmic {
        t.with(Settings {
            port,
            ..(*t.s).clone()
        })
    }

    #[test]
    fn a_phone_on_android_is_left_alone() {
        let (d, t) = lab_android_already("android");
        // a shell that would hang up on a command: nothing may be sent to it
        let t = on_port(&t, fake::phone(vec![("reboot", "!x")], vec![]));
        t.way_back()().unwrap();
        assert!(t
            .lab
            .shell(2)
            .and_then(|mut s| s.run("true", Duration::from_secs(2)))
            .is_ok());
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn a_test_image_is_sent_reboot_f_over_its_shell() {
        let (d, t) = lab_back_after("reboot", "restarted");
        let t = on_port(
            &t,
            fake::phone_marking(
                vec![("reboot -f", "!going")],
                vec![],
                Some(d.join("restarted")),
            ),
        );
        t.way_back()().unwrap();
        // the phone hung up on the reboot: the shell is gone
        assert!(!t.answers());
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn when_reboot_f_does_nothing_the_sysrq_trigger_is_sent() {
        // android returns when the phone restarts, which only the sysrq command makes it do: no clock decides
        let (d, t) = lab_back_after("sysrq", "restarted");
        let t = on_port(
            &t,
            fake::phone_marking(
                vec![("reboot -f", "ignored"), ("sysrq-trigger", "!going")],
                vec![],
                Some(d.join("restarted")),
            ),
        );
        t.way_back()().unwrap();
        assert!(!t.answers(), "the sysrq command was not sent");
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn a_phone_in_fastboot_is_rebooted_by_fastboot() {
        let (d, t) = lab_back_after("fastboot", "fastboot.rebooted");
        std::fs::write(d.join("fastboot.listed"), "").unwrap();
        let t = on_port(&t, fake::closed());
        t.way_back()().unwrap();
        assert!(d.join("fastboot.rebooted").exists());
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn a_silent_shell_and_no_android_says_the_power_hold_sentence() {
        let (d, t) = lab_never_back("silent");
        let t = on_port(&t, fake::closed());
        let e = t.way_back()().unwrap_err();
        assert!(e.contains("hold the power button about 15 seconds"), "{e}");
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn a_shell_that_answers_and_a_phone_that_stays_away_says_it_after_both_ways() {
        let (d, t) = lab_never_back("stays");
        let t = on_port(&t, fake::phone(vec![("reboot -f", "!x")], vec![]));
        let t0 = std::time::Instant::now();
        let e = t.way_back()().unwrap_err();
        assert!(e.contains("hold the power button"), "{e}");
        assert!(t0.elapsed().as_secs() < 30);
        std::fs::remove_dir_all(d).ok();
    }
}
