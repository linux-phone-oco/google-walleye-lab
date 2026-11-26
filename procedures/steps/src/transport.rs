// SPDX-License-Identifier: MIT OR Apache-2.0
//! The declared transports on the real Pixel 2, read only (item walleye-transport-procedure, for the run walleye-transport-real-phone-run;
//! decision 2026-11-26-walleye-android-connection-user-root): the phone is on Android, then the library, which asks adbd for root itself (lab-tools 0.53.5): the
//! `read_slot(phone_on("android"))` (the first live use of `[connections.android]`, which the entry file runs), then what this
//! file adds - the adb serial equals the `[fastboot]` serial, `adb reboot bootloader` is honoured and fastboot lists the phone with the
//! slot adb read, `fastboot reboot` returns Android on that slot. Every text each call printed is kept in the run's folder (transport.txt)
//! for the research row.
//!
//! Nothing is written, flashed or set: the calls are reads, one reboot to the bootloader and one out of it. The phone ends on Android,
//! on the slot it began on (the slot is read, never typed; the run's proof expects `b`, which the folder's text shows).

use crate::android::Android;
use crate::host;
use crate::install::{parse_current_slot, slots_agree};
use crate::lab::Lab;
use crate::settings::Settings;
use crate::watch;
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;

type Done = Result<(), String>;

/// The file of the run's folder holding what each call printed.
pub const READINGS_FILE: &str = "transport.txt";

/// The serial `adb get-serialno` printed is the one `[fastboot]` declares. adb prints the serial alone on a line; a text with
/// more than one line, or none, is a doubt (two phones on the bus, an error), never a match.
pub fn serial_matches(printed: &str, declared: &str) -> Result<(), String> {
    let printed = printed.trim();
    if declared.is_empty() {
        return Err("no serial is declared in device.toml [fastboot]".into());
    }
    if printed == declared {
        Ok(())
    } else {
        Err(format!(
            "adb's own serial is {printed:?}, device.toml [fastboot] declares {declared:?}: the two do not name the same phone"
        ))
    }
}

/// What the author is told before the run.
pub fn expect() -> String {
    "The transport run, about 5 minutes, read only: the slot read through the library's android connection, adb's own serial against the declared one, then the phone goes to its bootloader by `adb reboot bootloader` (fastboot must list it with the slot adb read) and out again by `fastboot reboot`. Nothing is written, flashed or set; the phone ends on Android on the slot it began on.".into()
}

/// What the author is told before the rehearsal.
pub fn expect_rehearsal() -> String {
    "The rehearsal of the transport run, about 2 minutes: the slot read through the library's android connection, adb's own serial against the declared one. The phone never leaves Android; nothing is written.".into()
}

#[derive(Clone)]
pub struct Transport {
    s: Rc<Settings>,
    lab: Lab,
    android: Android,
    /// the slot suffix adb read first (`_b`): what fastboot and the later reads must say again
    first: Rc<RefCell<Option<String>>>,
}

impl Transport {
    pub fn new(s: Settings) -> Transport {
        let s = Rc::new(s);
        let lab = Lab::new(s.clone());
        Transport {
            android: Android::new(s.clone(), lab.clone()),
            lab,
            s,
            first: Rc::new(RefCell::new(None)),
        }
    }

    pub fn battery_is_good(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.battery_is_good()
    }

    /// What each call printed, one line each, added to the run's folder (a stale file of an earlier run of the day is forgotten first).
    fn keep(&self, line: &str) -> Done {
        let file = self.s.out.join(READINGS_FILE);
        let before = std::fs::read_to_string(&file).unwrap_or_default();
        self.lab
            .save(READINGS_FILE, format!("{before}{line}").trim_end())
    }

    pub fn author_is_told(&self, rehearsal: bool) -> impl FnOnce() -> Done + 'static {
        let (text, file) = (
            if rehearsal {
                expect_rehearsal()
            } else {
                expect()
            },
            self.s.out.join(READINGS_FILE),
        );
        move || {
            let _ = std::fs::remove_file(file); // a stale reading is never this run's
            println!("  {text}");
            Ok(())
        }
    }

    /// The phone is on Android (out of fastboot if it is there); the library's next call asks adbd for root (`[connections.android]` user = root, once per Phone).
    pub fn the_phone_is_on_android(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            watch::phone_is_touched();
            u.android.is_up()
        }
    }

    /// adb's own serial (`adb get-serialno`, no `-s`: one phone on the bus) is the declared one.
    pub fn the_adb_serial_is_the_declared_one(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let mut c = Command::new(&u.s.adb);
            c.arg("get-serialno");
            let said = host::run(c)?;
            if !said.ok {
                return Err(format!("adb get-serialno: {}", said.text));
            }
            u.keep(&format!("adb get-serialno said: {}", said.text.trim()))?;
            serial_matches(&said.text, &u.lab.serial()?)
        }
    }

    /// adb reads the slot suffix Android runs from (kept; the later reads must say it again).
    pub fn the_slot_is_read_by_adb(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let suffix = u.android.slot_suffix()?.trim().to_string();
            u.keep(&format!("adb getprop ro.boot.slot_suffix said: {suffix}"))?;
            if suffix != "_a" && suffix != "_b" {
                return Err(format!("adb told no slot _a or _b: {suffix:?}"));
            }
            *u.first.borrow_mut() = Some(suffix);
            Ok(())
        }
    }

    /// The phone is in fastboot by `adb reboot bootloader` (honoured: Android's init reads the argument).
    pub fn the_phone_is_taken_to_the_bootloader(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || {
            watch::phone_is_touched();
            lab.phone_is_in_fastboot()
        }
    }

    /// Fastboot lists the phone with a `current-slot` equal to the slot adb read.
    pub fn fastboot_lists_the_slot_adb_read(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let suffix = u.first_suffix()?;
            let mut c = Command::new(&u.s.fastboot);
            c.args(["-s", &u.lab.serial()?, "getvar", "current-slot"]);
            let text = host::run(c)?.text;
            u.keep(&format!(
                "fastboot getvar current-slot said: {}",
                text.replace('\n', " | ")
            ))?;
            slots_agree(&suffix, &parse_current_slot(&text)?).map(|_| ())
        }
    }

    /// `fastboot reboot` is sent and Android answers within `back_wait` (getprop needs no root).
    pub fn fastboot_reboot_returns_android(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            u.lab.fastboot_reboot();
            u.keep("fastboot reboot sent")?;
            u.lab.android_is_back()
        }
    }

    /// Android runs from the slot it began on.
    pub fn the_slot_is_the_one_it_began_on(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let again = u.android.slot_suffix()?.trim().to_string();
            u.keep(&format!(
                "adb getprop ro.boot.slot_suffix said again: {again}"
            ))?;
            let first = u.first_suffix()?;
            if again == first {
                Ok(())
            } else {
                Err(format!(
                    "Android began on slot {first} and runs from {again} now"
                ))
            }
        }
    }

    fn first_suffix(&self) -> Result<String, String> {
        self.first
            .borrow()
            .clone()
            .ok_or_else(|| "the slot adb reads is not read yet".to_string())
    }

    pub fn android_is_back(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.android_is_back()
    }

    /// The way back, the shared one: the phone ends on Android whatever stopped the run (only the watching parent runs it).
    pub fn way_back(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.way_back()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::executable;
    use crate::settings::Try;

    /// `fastboot getvar all`'s slot lines as the phone printed them in the check C1 run (research/walleye-bootimage/2026-11-25-c1-run/slots-before.txt).
    const SLOTS_B: &str =
        include_str!("../../../tests/fixtures/phone-text/fastboot-slots-android-b-2026-11-25.cap");

    #[test]
    fn the_slot_fastboot_printed_in_the_c1_run_is_b_and_agrees_with_adbs_suffix() {
        let slot = parse_current_slot(SLOTS_B).unwrap();
        assert_eq!(slot, "b");
        assert_eq!(slots_agree("_b", &slot).unwrap(), "b");
        assert!(slots_agree("_a", &slot).is_err());
    }

    #[test]
    fn the_serial_must_equal_the_declared_one_exactly() {
        assert!(serial_matches("FA81N1A00790\n", "FA81N1A00790").is_ok());
        assert!(serial_matches("FA81N1A00791", "FA81N1A00790").is_err());
        assert!(serial_matches("", "FA81N1A00790").is_err());
        // two phones on the bus: adb says so and prints no serial
        assert!(serial_matches("error: more than one device/emulator", "FA81N1A00790").is_err());
        assert!(serial_matches("FA81N1A00790", "").is_err());
    }

    /// A lab whose adb answers `get-serialno` with `serial` and whose device.toml declares `declared`.
    fn lab(name: &str, serial: &str, declared: &str) -> Transport {
        let d = std::env::temp_dir().join(format!("wt-tr-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(
            d.join("device.toml"),
            format!("[fastboot]\nserial = \"{declared}\"\n"),
        )
        .unwrap();
        executable(
            &d.join("adb"),
            &format!(
                "#!/bin/sh\ncase \"$1\" in get-serialno) echo '{serial}'; exit 0;; esac\nexit 1\n"
            ),
        );
        let mut s = Settings::stand_in(Try::Touch);
        s.lab = d.clone();
        s.out = d.join("out");
        s.adb = d.join("adb").to_string_lossy().into_owned();
        Transport::new(s)
    }

    /// Source of the stand-in adb: it prints the serial alone, as `adb get-serialno` is documented to; the phone's own text of it is NOT captured yet (not known yet): the first real run keeps it in transport.txt.
    #[test]
    fn the_serial_step_passes_on_the_declared_serial_and_keeps_what_adb_said() {
        let t = lab("same", "FA81N1A00790", "FA81N1A00790");
        t.the_adb_serial_is_the_declared_one()().unwrap();
        let kept = std::fs::read_to_string(t.s.out.join(READINGS_FILE)).unwrap();
        assert!(
            kept.contains("adb get-serialno said: FA81N1A00790"),
            "{kept}"
        );
    }

    #[test]
    fn the_serial_step_fails_naming_both_serials_when_they_differ() {
        let t = lab("other", "ZZZ", "FA81N1A00790");
        let e = t.the_adb_serial_is_the_declared_one()().unwrap_err();
        assert!(e.contains("ZZZ") && e.contains("FA81N1A00790"), "{e}");
    }

    #[test]
    fn a_later_slot_read_before_the_first_is_refused() {
        let t = lab("noslot", "x", "x");
        let e = t.fastboot_lists_the_slot_adb_read()().unwrap_err();
        assert!(e.contains("not read yet"), "{e}");
    }
}

#[cfg(test)]
mod fastboot_tests {
    use super::*;
    use crate::fixture::executable;
    use crate::settings::Try;

    /// A lab whose fastboot prints the captured slot lines (tests/fixtures/phone-text, a cut of research/walleye-bootimage/2026-11-25-c1-run/slots-before.txt) and whose adb read `first`.
    /// Fastboot's raw `getvar current-slot` text alone is not captured yet (not known yet): the first run keeps it in transport.txt.
    fn lab(name: &str, first: &str) -> Transport {
        let d = std::env::temp_dir().join(format!("wt-trf-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("device.toml"), "[fastboot]\nserial = \"SER1\"\n").unwrap();
        let cap = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/phone-text/fastboot-slots-android-b-2026-11-25.cap"
        );
        executable(&d.join("fastboot"), &format!("#!/bin/sh\ncat '{cap}'\n"));
        let mut s = Settings::stand_in(Try::Touch);
        s.lab = d.clone();
        s.out = d.join("out");
        s.fastboot = d.join("fastboot").to_string_lossy().into_owned();
        let t = Transport::new(s);
        *t.first.borrow_mut() = Some(first.to_string());
        t
    }

    #[test]
    fn fastboot_agreeing_with_adb_passes_and_disagreeing_fails() {
        lab("same", "_b").fastboot_lists_the_slot_adb_read()().unwrap();
        let e = lab("other", "_a").fastboot_lists_the_slot_adb_read()().unwrap_err();
        assert!(e.contains("disagree"), "{e}");
    }
}
