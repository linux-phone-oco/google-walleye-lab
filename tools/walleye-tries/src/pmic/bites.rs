// SPDX-License-Identifier: MIT OR Apache-2.0
//! The four PMIC bites' steps (item walleye-wdt-pon-bites-procedure): the watchdog bites with the S2 type 0x07 (hard)
//! and 0x01 (warm), each with the USB cable in and with it out, and what the PON registers say afterwards.
//!
//! What the lab decides without the author: after a bite with the cable in, the shell goes quiet; then adb up by itself
//! is "Android came back", fastboot listed is "the bootloader", neither within `charger_wait` is "the charger screen or
//! off" ([`Seen`]). Only in that last case is the author asked, and only for a physical act. The registers are read where
//! the phone first answers: in Android (what it says of the reason it booted) or in our own image booted again from
//! fastboot (the init's pon-regs reads the reason blocks; that boot is a jump, not a reset). What was read is said raw:
//! a register is named only where `[procedures.*] pon_names` names it (not known yet is not "missing").

use super::*;

/// The S2 reset type the image's dts names: hard, the driver's own (0x07), or warm (0x01).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum S2 {
    Hard,
    Warm,
}

impl S2 {
    /// The value of the low nibble of PON_PMIC_WD_RESET_S2_CTL.
    pub fn code(self) -> u8 {
        match self {
            S2::Hard => 7,
            S2::Warm => 1,
        }
    }
}

/// Where the USB cable is while the watchdog bites.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cable {
    In,
    Out,
}

/// One bite: the type it is armed with and the cable's place.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bite {
    pub s2: S2,
    pub cable: Cable,
}

impl Bite {
    pub const HARD_IN: Bite = Bite {
        s2: S2::Hard,
        cable: Cable::In,
    };
    pub const HARD_OUT: Bite = Bite {
        s2: S2::Hard,
        cable: Cable::Out,
    };
    pub const WARM_IN: Bite = Bite {
        s2: S2::Warm,
        cable: Cable::In,
    };
    pub const WARM_OUT: Bite = Bite {
        s2: S2::Warm,
        cable: Cable::Out,
    };

    /// Said to the author and logged: "S2 type 0x07, cable in".
    pub fn name(self) -> String {
        format!(
            "S2 type 0x{:02x}, cable {}",
            self.s2.code(),
            match self.cable {
                Cable::In => "in",
                Cable::Out => "out",
            }
        )
    }

    /// The prefix of this bite's files: `hard-in`, `warm-out`.
    pub fn slug(self) -> String {
        format!(
            "{}-{}",
            match self.s2 {
                S2::Hard => "hard",
                S2::Warm => "warm",
            },
            match self.cable {
                Cable::In => "in",
                Cable::Out => "out",
            }
        )
    }
}

/// What the lab sees after a bite, before anyone does anything.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Seen {
    /// adb answers on a boot that began after the arming
    Android,
    /// fastboot lists the phone
    Fastboot,
    /// neither: the bootloader's charger screen, or the phone is off (the lab cannot tell which)
    Dark,
}

/// The words said to the author: physical acts only, in the plain words of the item.
pub mod words {
    pub const READY: &str = "stay at the phone with a finger on its power button: the watchdog is about to bite, and the phone may stay dark (it did last time) until the power button is pressed / say ok when you are there";
    pub const CABLE_OUT: &str = "the USB cable is out of the phone within the seconds this step says, before the bite (the bite follows by itself; the screen may not show it)";
    pub const CABLE_BACK: &str = "the USB cable is in the phone again, and the computer lists the phone, or the screen shows the charging picture, or Android";
    pub const LOOK: &str =
        "the phone's screen: a charging picture, a dark screen, or Android (say which)";
    pub const BUTTONS: &str = "power + volume down";
    pub const BOOTLOADER: &str = "the bootloader screen (fastboot) shows, or Android starts";
}

/// The PON registers `pon-regs` printed: the `0xRR=VV` words, in the order printed. `??` (not read) stays `??`.
pub fn parse_pon_regs(text: &str) -> Vec<(String, String)> {
    let Some(line) = text
        .lines()
        .find(|l| l.trim_start().starts_with("pon-regs "))
    else {
        return Vec::new();
    };
    line.split_whitespace()
        .filter_map(|w| w.strip_prefix("0x")?.split_once('='))
        .map(|(r, v)| (r.to_ascii_lowercase(), v.to_string()))
        .collect()
}

/// The registers said one per line: the offset, the name where `names` gives one, the value - read, or `not read`.
pub fn say_regs(regs: &[(String, String)], names: &[(String, String)]) -> String {
    if regs.is_empty() {
        return "the PON registers: not known yet (pon-regs printed none)".into();
    }
    regs.iter()
        .map(|(r, v)| {
            let name = names
                .iter()
                .find(|(n, _)| n == r)
                .map_or("(not named)", |(_, n)| n.as_str());
            let v = if v == "??" { "not read" } else { v.as_str() };
            format!("0x8{r} {name} = {v}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The lines of an Android dmesg (or of the init's) that tell why the phone booted: power-on and power-off
/// reasons, resets, the PMIC's own lines. A serial number never stays (the caller redacts), forty lines at most.
pub fn reason_lines(dmesg: &str) -> Vec<String> {
    const WORDS: [&str; 7] = [
        "power-on",
        "poff",
        "pon ",
        "reset type",
        "bootreason",
        "boot reason",
        "s2 reset",
    ];
    dmesg
        .lines()
        .filter(|l| {
            let l = l.to_ascii_lowercase();
            WORDS.iter().any(|w| l.contains(w))
        })
        .take(40)
        .map(str::to_string)
        .collect()
}

/// The guard before a bite: the PMIC watchdog probed and the S2 type is the one this image's dts names
/// (read from the PON registers, or by the driver's own read-back when those do not read: the probe fails when the type it set does not read back).
pub fn judge_bite_type(s: &State, want: u8) -> Result<Know, String> {
    if s.pmic == "none" || s.pmic.is_empty() {
        return Err("no PMIC watchdog device: the driver did not probe; no bite".into());
    }
    match s.s2_type.as_str() {
        t if t == format!("0x{want:02x}") => Ok(Know::Read),
        "unread" => Ok(Know::ByProbe),
        other => Err(format!(
            "the S2 type reads {other}, not the 0x{want:02x} this image names: stop, record, no bite"
        )),
    }
}

impl Pmic {
    /// The try of the four bites: its two images are the hard one and the warm one, its files go to their own folder.
    pub fn for_bites(s: Settings) -> Pmic {
        let out = s.out_dir("pmic-wdt-bites");
        let s = Rc::new(s);
        Pmic {
            lab: Lab::new(s.clone()),
            s,
            out: Rc::new(out),
            facts: Rc::default(),
            names: ["hard", "warm"],
        }
    }

    fn bite_image(&self, b: Bite) -> Image {
        let [hard, warm] = self.images();
        match b.s2 {
            S2::Hard => hard,
            S2::Warm => warm,
        }
    }

    /// The bite in hand: its files are named by it from now on.
    fn begin(&self, b: Bite) {
        let mut f = self.facts.borrow_mut();
        f.bite = Some(b);
        f.know = None;
        f.seen = None;
        f.armed_at = None;
    }

    /// Boot this bite's image from fastboot, from memory; the banner's gadget is up.
    pub fn bite_image_boots(&self, b: Bite) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            t.begin(b);
            let image = t.bite_image(b);
            t.boot(&image.path)?;
            t.lab
                .script(
                    "gadget-up.sh",
                    &[&t.s.device_toml().to_string_lossy()],
                    &[("GADGET_WAIT", "120")],
                )
                .map_err(|e| format!("image {} shows no banner within 120 s: {e}", image.name))?;
            t.facts.borrow_mut().chosen = Some(image.clone());
            t.note(format!(
                "{}: image {} booted and shows its banner",
                b.name(),
                image.name
            ));
            Ok(())
        }
    }

    /// The watchdog probed, the shell answers, and the S2 type is the one the bite is for.
    pub fn bite_type_reads(&self, b: Bite) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let s = t.read_state()?;
            let know = judge_bite_type(&s, b.s2.code())?;
            t.facts.borrow_mut().know = Some(know);
            // hold the init's own 180 s restart while the author reads the next prompt
            t.lab
                .shell(10)?
                .run("touch /tmp/hold", Duration::from_secs(10))?;
            t.note(format!(
                "{}: {} watchdog probed; S2 type {} ({}); firmware type {}",
                b.name(),
                s.pmic,
                s.s2_type,
                match know {
                    Know::Read => "read from the PON registers",
                    Know::ByProbe => "not read; the probe's own read-back says it",
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

    /// Seconds the watchdog is armed for: its 20 s with the cable in, the author's time to pull the cable with it out.
    fn bite_seconds(&self, b: Bite) -> u64 {
        match b.cable {
            Cable::In => BITE_S,
            Cable::Out => self.s.cable_out_s,
        }
    }

    /// Arm the watchdog for this bite and never pet it.
    pub fn bite_is_armed(&self, b: Bite) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let know = t
                .facts
                .borrow()
                .know
                .ok_or("the S2 type was not checked first: no arming")?;
            let seconds = t.bite_seconds(b);
            let trust = match know {
                Know::ByProbe => "probe",
                Know::Read => "read",
            };
            let mut sh = t.lab.shell(10)?;
            let (lines, _) = sh.run_partial(
                &format!("wdt-bite {} {trust} {seconds}", b.s2.code()),
                Duration::from_secs(60),
            )?;
            let armed = arming_said(&lines, &format!("the bite is due in {seconds} s"))?;
            t.facts.borrow_mut().armed_at = Some(Instant::now());
            t.note(format!("{}: armed, not petted: {armed}", b.name()));
            Ok(())
        }
    }

    /// With the cable in: the shell goes quiet at the bite.
    pub fn the_phone_goes_quiet(&self, b: Bite) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let armed = t.facts.borrow().armed_at.ok_or("the bite was not armed")?;
            let lost = t.shell_goes_quiet(armed)?;
            t.note(format!(
                "{}: the shell went quiet {lost} s after the arming: the bite",
                b.name()
            ));
            Ok(())
        }
    }

    /// With the cable out: the host cannot reach the phone any more (the author pulled the cable in time).
    pub fn the_host_lost_the_phone(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            if t.answers() {
                return Err("the phone's shell still answers: the cable is not out (the bite comes in time anyway, but with the cable in)".into());
            }
            Ok(())
        }
    }

    /// With the cable out: the lab waits out the bite - the armed seconds and the slack - blind, as the host sees nothing.
    pub fn the_bite_passes(&self, b: Bite) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let armed = t.facts.borrow().armed_at.ok_or("the bite was not armed")?;
            let slack = BITE_SLACK_S.min(t.s.bite_wait.saturating_sub(t.bite_seconds(b)));
            let until = armed + Duration::from_secs(t.bite_seconds(b) + slack);
            while Instant::now() < until {
                std::thread::sleep(Duration::from_millis(500));
            }
            t.note(format!(
                "{}: waited out the bite ({} s armed, {} s slack); the host saw nothing, the cable being out",
                b.name(),
                t.bite_seconds(b),
                slack
            ));
            Ok(())
        }
    }

    /// What the lab sees now (never judged as pass or fail): Android on a boot that began after the arming, the
    /// bootloader, or nothing within `charger_wait` seconds (the charger screen, or off).
    pub fn what_the_lab_sees(&self, b: Bite) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let armed = t.facts.borrow().armed_at.ok_or("the bite was not armed")?;
            let until = Instant::now() + Duration::from_secs(t.s.charger_wait);
            let seen = loop {
                let since = Instant::now().duration_since(armed).as_secs();
                match t.android_uptime() {
                    Some(up) if fresh_boot(up, since) => break Seen::Android,
                    Some(_) => {}
                    None if t.lab.in_fastboot()? => break Seen::Fastboot,
                    None => {}
                }
                if Instant::now() >= until {
                    break Seen::Dark;
                }
                std::thread::sleep(Duration::from_secs(2));
            };
            t.facts.borrow_mut().seen = Some(seen);
            let said = match seen {
                Seen::Android => "Android answers by itself on a new boot".to_string(),
                Seen::Fastboot => "the phone is in the bootloader by itself".to_string(),
                Seen::Dark => format!(
                    "no adb and no fastboot within {} s: the charger screen, or the phone is off",
                    t.s.charger_wait
                ),
            };
            t.save("seen.txt", &said)?;
            t.note(format!("{}: {said}", b.name()));
            Ok(())
        }
    }

    /// The phone showed neither Android nor the bootloader (also true in a dry run, where nothing was looked at).
    pub fn the_phone_is_dark(&self) -> bool {
        !matches!(
            self.facts.borrow().seen,
            Some(Seen::Android) | Some(Seen::Fastboot)
        )
    }

    /// Read what the phone says of the reason, at the first place it answers: Android's own record, or the reason
    /// blocks of our image booted again from the bootloader.
    pub fn the_reason_is_read(&self, b: Bite) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let seen = t.facts.borrow().seen;
            match seen {
                Some(Seen::Android) => t.read_in_android(b),
                Some(Seen::Fastboot) => t.read_in_our_image(b),
                Some(Seen::Dark) | None => {
                    t.save(
                        "reason.txt",
                        "not known yet: the phone showed neither Android nor the bootloader",
                    )?;
                    t.note(format!(
                        "{}: the reason is not known yet: the phone showed neither Android nor the bootloader",
                        b.name()
                    ));
                    Ok(())
                }
            }
        }
    }

    /// Android's record (its boot reason, its slot, the reason lines of its kernel log when adb may read them).
    fn read_in_android(&self, b: Bite) -> Done {
        let reason = self.getprop("ro.boot.bootreason");
        let slot = self.getprop("ro.boot.slot_suffix");
        let dmesg = self
            .tool(&self.s.adb, &["shell", "dmesg"])
            .ok()
            .filter(|d| d.ok)
            .map(|d| reason_lines(&redact(&d.text)));
        let lines = match &dmesg {
            Some(l) if !l.is_empty() => l.join("\n"),
            Some(_) => "the kernel log has no reason line".into(),
            None => "unread (adb may not read the kernel log)".into(),
        };
        self.save(
            "reason.txt",
            &format!(
                "Android: bootreason [{reason}] slot [{slot}]\nkernel log reason lines:\n{lines}"
            ),
        )?;
        self.note(format!(
            "{}: Android says boot reason [{reason}], slot [{slot}]; kernel log: {}",
            b.name(),
            lines.lines().next().unwrap_or("")
        ));
        Ok(())
    }

    /// Our image booted again from the bootloader: its pon-regs and the driver's two lines, said raw.
    fn read_in_our_image(&self, b: Bite) -> Done {
        let image = self.bite_image(b);
        self.boot(&image.path)?;
        self.lab
            .script(
                "gadget-up.sh",
                &[&self.s.device_toml().to_string_lossy()],
                &[("GADGET_WAIT", "120")],
            )
            .map_err(|e| format!("image {} shows no banner within 120 s: {e}", image.name))?;
        let mut sh = self.lab.shell(10)?;
        let regs = sh.run("pon-regs", Duration::from_secs(30))?;
        let dmesg = sh.run("dmesg", Duration::from_secs(20))?;
        self.save("pon-regs.txt", &regs)?;
        let said = say_regs(&parse_pon_regs(&regs), &self.s.pon_names);
        let driver = reason_lines(&redact(&dmesg)).join("\n");
        self.save("reason.txt", &format!("{said}\ndriver lines:\n{driver}"))?;
        self.note(format!(
            "{}: read in image {}:\n{said}",
            b.name(),
            image.name
        ));
        Ok(())
    }

    /// The phone goes back to Android by its own way back, after every bite (the shared way back, also in the watched child).
    pub fn the_phone_is_back_on_android(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.way_back()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_four_bites_are_named_and_their_files_kept_apart() {
        let all = [Bite::HARD_IN, Bite::HARD_OUT, Bite::WARM_IN, Bite::WARM_OUT];
        let slugs: Vec<_> = all.iter().map(|b| b.slug()).collect();
        assert_eq!(slugs, ["hard-in", "hard-out", "warm-in", "warm-out"]);
        assert_eq!(Bite::WARM_OUT.name(), "S2 type 0x01, cable out");
        assert_eq!(Bite::HARD_IN.name(), "S2 type 0x07, cable in");
    }

    #[test]
    fn the_registers_are_read_from_the_line_and_said_with_names_only_where_given() {
        let line = "noise\npon-regs 0x08=80 0x0a=?? 0x0c=04 0xc0=02 src=[spmi]";
        let regs = parse_pon_regs(line);
        assert_eq!(regs.len(), 4);
        let names = vec![("08".to_string(), "PON_REASON1".to_string())];
        let said = say_regs(&regs, &names);
        assert!(said.contains("0x808 PON_REASON1 = 80"), "{said}");
        assert!(said.contains("0x80a (not named) = not read"), "{said}");
        assert!(said.contains("0x8c0 (not named) = 02"), "{said}");
        assert!(say_regs(&[], &names).contains("not known yet"));
        assert!(parse_pon_regs("pon-regs none regmaps=[] dbgfs=[no]").is_empty());
    }

    #[test]
    fn only_the_lines_that_tell_the_reason_are_kept() {
        let log = "[0.1] cpu up\n[0.5] qcom,pmic PMIC@SID0: Power-on reason: Triggered from CBL\n[0.6] x\n[0.7] pm8916_wdt: POFF reason: 0x4 0x0";
        let l = reason_lines(log);
        assert_eq!(l.len(), 2, "{l:?}");
        assert!(reason_lines("nothing").is_empty());
    }

    #[test]
    fn the_guard_takes_the_type_the_image_names_and_no_other() {
        let s = |t: &str| State {
            pmic: "watchdog1".into(),
            s2_type: t.into(),
            s2_enabled: "no".into(),
            firmware_type: String::new(),
        };
        assert_eq!(judge_bite_type(&s("0x01"), 1), Ok(Know::Read));
        assert_eq!(judge_bite_type(&s("0x07"), 7), Ok(Know::Read));
        assert_eq!(judge_bite_type(&s("unread"), 1), Ok(Know::ByProbe));
        let e = judge_bite_type(&s("0x07"), 1).unwrap_err();
        assert!(
            e.contains("not the 0x01 this image names") && e.contains("no bite"),
            "{e}"
        );
        let none = State {
            pmic: "none".into(),
            ..s("0x01")
        };
        assert!(judge_bite_type(&none, 1)
            .unwrap_err()
            .contains("did not probe"));
    }
}

#[cfg(test)]
mod walkthrough {
    //! The bites' steps against a stand-in phone and stand-in host programs (adb, fastboot, lpoco, the gate scripts): no phone, no author.
    use super::*;
    use crate::pmic::walkthrough::{lab, phone, BITE_OK, STATE_OK};
    use std::path::{Path, PathBuf};

    const STATE_WARM: &str = "wdt-state pmic=watchdog1 soc=watchdog0 s2-type=0x01 s2-enabled=no src=x firmware-type=0x01 probe=pmic-device-registered kline-type=[] kline-poff=[]";
    const BITE_WARM: &str = "!wdt-arm armed watchdog1 type=0x01 timeout=20 s2-enabled-before=no after=yes hold=90s\nwdt-bite: x; not petted, the bite is due in 20 s";
    const REGS: &str = "pon-regs 0x08=02 0x0a=00 0x0c=04 0xc0=01 0xc5=04 src=[spmi]";

    /// The lab of the v1 try plus the two images of the bites with their checksums.
    fn bites_lab(name: &str, adb_state: &str) -> PathBuf {
        let d = lab(name, adb_state);
        for w in ["hard", "warm"] {
            std::fs::write(d.join(format!("boot-wdt2-{w}.img")), w).unwrap();
        }
        let sums = std::process::Command::new("sh")
            .current_dir(&d)
            .args([
                "-c",
                "sha256sum boot-wdt2-hard.img boot-wdt2-warm.img > SHA256SUMS-wdt2",
            ])
            .status()
            .unwrap();
        assert!(sums.success());
        d
    }

    fn the_bites(d: &Path, port: u16) -> Pmic {
        let mut s = Settings::stand_in(crate::settings::Try::Bites);
        s.lab = d.to_path_buf();
        s.address = "127.0.0.1".into();
        s.port = port;
        s.lpoco = d.join("lpoco").to_string_lossy().into_owned();
        s.adb = d.join("adb").to_string_lossy().into_owned();
        s.fastboot = d.join("fastboot").to_string_lossy().into_owned();
        s.image_a = d.join("boot-wdt2-hard.img");
        s.image_b = d.join("boot-wdt2-warm.img");
        s.out_given = Some(d.join("out"));
        s.bite_wait = 4;
        s.back_wait = 5;
        s.charger_wait = 4;
        s.cable_out_s = 1;
        Pmic::for_bites(s)
    }

    /// The same try with no adb: the phone is not in Android.
    fn without_adb(t: Pmic) -> Pmic {
        Pmic {
            s: Rc::new(Settings {
                adb: "/nonexistent/adb".into(),
                ..(*t.s).clone()
            }),
            ..t
        }
    }

    #[test]
    fn a_hard_bite_with_the_cable_in_and_android_back_by_itself_is_read_from_android() {
        let d = bites_lab("hard-in", "5");
        let t = the_bites(&d, phone(STATE_OK, BITE_OK));
        let b = Bite::HARD_IN;
        t.images_are_the_built_ones()().unwrap();
        t.bite_image_boots(b)().unwrap();
        t.bite_type_reads(b)().unwrap();
        t.bite_is_armed(b)().unwrap();
        t.the_phone_goes_quiet(b)().unwrap();
        t.what_the_lab_sees(b)().unwrap();
        assert!(!t.the_phone_is_dark());
        t.the_reason_is_read(b)().unwrap();
        let calls = std::fs::read_to_string(d.join("lpoco.calls")).unwrap();
        assert!(calls.contains("boot-wdt2-hard.img"), "{calls}");
        let said = std::fs::read_to_string(d.join("out/hard-in-reason.txt")).unwrap();
        assert!(said.contains("bootreason [watchdog]"), "{said}");
        let notes = t.facts.borrow().notes.join("\n");
        assert!(
            notes.contains("S2 type 0x07, cable in: Android answers by itself"),
            "{notes}"
        );
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn the_warm_image_is_booted_for_the_warm_bite_and_a_hard_reading_stops_it() {
        let d = bites_lab("warm-guard", "5");
        let t = the_bites(&d, phone(STATE_OK, BITE_OK));
        t.bite_image_boots(Bite::WARM_IN)().unwrap();
        let calls = std::fs::read_to_string(d.join("lpoco.calls")).unwrap();
        assert!(calls.contains("boot-wdt2-warm.img"), "{calls}");
        // the phone reads 0x07 but this bite is for 0x01: no arming
        let e = t.bite_type_reads(Bite::WARM_IN)().unwrap_err();
        assert!(e.contains("not the 0x01 this image names"), "{e}");
        let e = t.bite_is_armed(Bite::WARM_IN)().unwrap_err();
        assert!(e.contains("no arming"), "{e}");
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn a_warm_bite_arms_with_its_type_and_says_the_time() {
        let d = bites_lab("warm-in", "5");
        let t = the_bites(&d, phone(STATE_WARM, BITE_WARM));
        let b = Bite::WARM_IN;
        t.bite_image_boots(b)().unwrap();
        t.bite_type_reads(b)().unwrap();
        t.bite_is_armed(b)().unwrap();
        let notes = t.facts.borrow().notes.join("\n");
        assert!(
            notes.contains("S2 type 0x01, cable in: armed, not petted"),
            "{notes}"
        );
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn nothing_seen_is_the_dark_phone_and_its_reason_is_not_known_yet() {
        let d = bites_lab("dark", "5");
        let t = without_adb(the_bites(&d, phone(STATE_OK, BITE_OK)));
        let b = Bite::HARD_IN;
        t.begin(b);
        t.facts.borrow_mut().armed_at = Some(Instant::now());
        assert!(
            t.the_phone_is_dark(),
            "before anything is seen the phone counts as dark"
        );
        t.what_the_lab_sees(b)().unwrap();
        assert_eq!(t.facts.borrow().seen, Some(Seen::Dark));
        assert!(t.the_phone_is_dark());
        t.the_reason_is_read(b)().unwrap();
        let said = std::fs::read_to_string(d.join("out/hard-in-reason.txt")).unwrap();
        assert!(said.contains("not known yet"), "{said}");
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn the_bootloader_by_itself_boots_our_image_again_and_reads_its_registers() {
        let d = bites_lab("fastboot", "5");
        let t = the_bites(&d, phone_with_regs());
        let t = without_adb(t);
        std::fs::write(d.join("bootloader"), "").unwrap(); // the stand-in fastboot lists the phone
        let b = Bite::WARM_OUT;
        t.begin(b);
        t.facts.borrow_mut().armed_at = Some(Instant::now());
        t.what_the_lab_sees(b)().unwrap();
        assert_eq!(t.facts.borrow().seen, Some(Seen::Fastboot));
        t.the_reason_is_read(b)().unwrap();
        let calls = std::fs::read_to_string(d.join("lpoco.calls")).unwrap();
        assert!(calls.contains("boot-wdt2-warm.img"), "{calls}");
        let said = std::fs::read_to_string(d.join("out/warm-out-reason.txt")).unwrap();
        assert!(said.contains("0x808 PON_REASON1 = 02"), "{said}");
        assert!(said.contains("0x8c0 (not named) = 01"), "{said}");
        assert!(said.contains("POFF reason"), "{said}");
        std::fs::remove_dir_all(d).ok();
    }

    fn phone_with_regs() -> u16 {
        crate::phone::fake::phone(
            vec![
                ("pon-regs", REGS),
                ("dmesg", "pm8916_wdt: POFF reason: 0x4 0x0\nunrelated line"),
            ],
            vec![],
        )
    }

    #[test]
    fn with_the_cable_out_the_host_must_have_lost_the_phone_and_the_bite_is_waited_out() {
        let d = bites_lab("cable-out", "5");
        let b = Bite::HARD_OUT;
        // the cable is still in: the shell answers
        let t = the_bites(&d, phone(STATE_OK, BITE_OK));
        assert!(t.the_host_lost_the_phone()()
            .unwrap_err()
            .contains("cable is not out"));
        // the cable is out: nothing answers
        let t = the_bites(&d, crate::phone::fake::closed());
        t.begin(b);
        t.facts.borrow_mut().armed_at = Some(Instant::now());
        t.the_host_lost_the_phone()().unwrap();
        let before = Instant::now();
        t.the_bite_passes(b)().unwrap();
        assert!(before.elapsed() >= Duration::from_secs(1));
        assert!(t
            .facts
            .borrow()
            .notes
            .join("\n")
            .contains("waited out the bite"));
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn the_images_are_named_hard_and_warm_in_what_is_said() {
        let d = bites_lab("names", "5");
        std::fs::write(d.join("boot-wdt2-warm.img"), "changed").unwrap();
        let t = the_bites(&d, crate::phone::fake::closed());
        let e = t.images_are_the_built_ones()().unwrap_err();
        assert!(e.starts_with("image warm:"), "{e}");
        std::fs::remove_dir_all(d).ok();
    }
}
