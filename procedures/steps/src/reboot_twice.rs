// SPDX-License-Identifier: MIT OR Apache-2.0
//! The timed fastboot-reboot excursion (item walleye-fastboot-reboot-twice-procedure, for the try walleye-fastboot-reboot-twice-timed-try;
//! research walleye-fastboot-reboot-stays-in-fastboot-causes-ranked): the phone is taken from Android to its bootloader, `fastboot reboot` is sent
//! and timed, a second `fastboot reboot` only when the phone still lists in fastboot after the first, then a second excursion that sends
//! `fastboot continue`, timed. Each call's own line and its time are kept in the run's folder (legs.txt) for the research row.
//!
//! Nothing is flashed, set or written: the legs are reboots out of the bootloader, the card, the boot chain and the slots' flags are read before and
//! after and must be equal, and the way back is the shared one (Android, adbd as root). It boots no image, so it needs no go: the way back of every
//! other try sends the same `fastboot reboot` (decision in the item's session-log row).
//! What the phone does after `fastboot continue` from this bootloader is NOT KNOWN YET until the first real run.

use crate::host;
use crate::lab::Lab;
use crate::rootfs_boot::RootfsBoot;
use crate::settings::Settings;
use std::process::Command;
use std::rc::Rc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

type Done = Result<(), String>;

/// The file of the run's folder holding every leg's line.
pub const LEGS_FILE: &str = "legs.txt";
/// The file holding what the legs add up to.
pub const VERDICT_FILE: &str = "verdict.txt";
/// The files of an earlier run of the same day that this run must not take for its own.
pub const STALE: [&str; 2] = [LEGS_FILE, VERDICT_FILE];

/// One call of fastboot that leaves the bootloader, with the time it took Android to answer after it.
#[derive(Debug, Clone, PartialEq)]
pub struct Leg {
    /// `reboot` or `continue`
    pub word: String,
    /// fastboot's exit status was success
    pub accepted: bool,
    /// fastboot's last line
    pub said: String,
    /// when it was sent, seconds since the epoch (the host clock)
    pub at: u64,
    /// how long Android took to answer after it, or how long it was waited for in vain
    pub waited: Duration,
    pub back: bool,
}

impl Leg {
    /// The line kept: the call, fastboot's own words, the time sent, and what followed.
    pub fn line(&self, n: usize) -> String {
        let after = if self.back {
            format!(
                "Android answered {} s after it was sent",
                self.waited.as_secs()
            )
        } else {
            format!(
                "Android did not answer {} s after it was sent",
                self.waited.as_secs()
            )
        };
        let rc = if self.accepted { "accepted" } else { "refused" };
        format!(
            "leg {n}: `fastboot {}` {rc} at {} (epoch s), fastboot said {:?}; {after}",
            self.word, self.at, self.said
        )
    }
}

/// Which call comes next: the first `fastboot reboot`; one more only when the phone still lists in fastboot after it; then one `fastboot continue`
/// (from the bootloader, so a phone that is on Android is taken there first). Nothing more is sent: no third reboot, no second continue.
pub fn next_word(legs: &[Leg], on_android: bool, in_fastboot: bool) -> Option<&'static str> {
    let reboots = legs.iter().filter(|l| l.word == "reboot").count();
    let continued = legs.iter().any(|l| l.word == "continue");
    match legs.last() {
        None => Some("reboot"),
        Some(last) if last.word == "reboot" && reboots == 1 && !last.back && in_fastboot => {
            Some("reboot")
        }
        _ if !continued && (on_android || in_fastboot) => Some("continue"),
        _ => None,
    }
}

/// What the legs add up to, for the research row: which call brought Android back, after how long. It says what was counted, not why.
pub fn verdict(legs: &[Leg]) -> String {
    if legs.is_empty() {
        return "no leg was sent: not known yet".into();
    }
    let said: Vec<String> = legs
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let what = if !l.accepted {
                "was refused by fastboot".to_string()
            } else if l.back {
                format!("brought Android back after {} s", l.waited.as_secs())
            } else {
                format!("did not bring Android back within {} s", l.waited.as_secs())
            };
            format!("leg {} (`fastboot {}`) {what}", i + 1, l.word)
        })
        .collect();
    let mut text = said.join("; ");
    if !legs.iter().any(|l| l.word == "continue") {
        text.push_str("; no `fastboot continue` was sent: the phone was neither on Android nor in fastboot, or a call was refused (not known yet)");
    }
    text
}

/// What the author is told before the run.
pub fn expect(luns: u64, back_wait: u64) -> String {
    format!("The timed fastboot excursion, up to {} minutes: the phone goes to its bootloader, `fastboot reboot` is sent and Android is waited for up to {back_wait} s; if the phone still lists in fastboot a second `fastboot reboot` is sent, timed the same; then the phone goes to the bootloader once more and `fastboot continue` is sent, timed. Nothing is flashed or set. The card's {luns} disks, the boot chain and the slots' flags are read before and after and must be equal; every call's line and time are kept. If the phone stays in fastboot the way back sends its own reboots and names the hand step.", (back_wait * 3 + 600) / 60)
}

/// What the author is told before the rehearsal.
pub fn expect_rehearsal(luns: u64) -> String {
    format!("The rehearsal of the timed fastboot excursion, about 5 minutes: every read of the run for real (the stored boot chain, the card's {luns} disks and the root, the slots' flags in fastboot) and the phone's trips to the bootloader and back, but no leg is sent. Nothing is written; the phone ends on Android. No go is needed or read.")
}

/// The try: the shared steps of the root boot (battery, Android as root, the card, fastboot, the flags) through [`RootfsBoot`], and the legs.
#[derive(Clone)]
pub struct RebootTwice {
    s: Rc<Settings>,
    lab: Lab,
    root: RootfsBoot,
}

impl RebootTwice {
    pub fn new(s: Settings) -> RebootTwice {
        let root = RootfsBoot::new(s.clone());
        let s = Rc::new(s);
        RebootTwice {
            lab: Lab::new(s.clone()),
            s,
            root,
        }
    }

    pub fn shared(&self) -> &RootfsBoot {
        &self.root
    }

    /// adb reads the slot (the shared step) after the files of an earlier run of the day are forgotten: a stale leg is never this run's.
    pub fn the_slot_is_read_by_adb(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        let shared = self.root.the_slot_is_read_by_adb();
        move || {
            for name in STALE {
                let _ = std::fs::remove_file(u.s.out.join(name));
            }
            shared()
        }
    }

    pub fn author_is_told(&self) -> impl FnOnce() -> Done + 'static {
        let text = expect(self.s.luns, self.s.back_wait);
        move || {
            println!("  {text}");
            Ok(())
        }
    }

    pub fn rehearsal_is_told(&self) -> impl FnOnce() -> Done + 'static {
        let text = expect_rehearsal(self.s.luns);
        move || {
            println!("  {text}");
            Ok(())
        }
    }

    /// The stored boot chain is whole (the lab's own check).
    pub fn the_stored_set_is_good(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let dir = u.s.bootchain.to_string_lossy().into_owned();
            u.lab
                .script("bootchain-readback-check.sh", &[&dir], &[])
                .map(|_| ())
        }
    }

    /// One call: `fastboot -s SERIAL WORD`, sent now and timed, then Android is waited for up to `back_wait`.
    fn send(&self, word: &str) -> Result<Leg, String> {
        let mut c = Command::new(&self.s.fastboot);
        c.args(["-s", &self.lab.serial()?, word]);
        let at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let sent = Instant::now();
        let said = host::run(c)?;
        let back = said.ok
            && self
                .lab
                .android_within(self.s.back_wait, Duration::from_secs(1))
                .is_ok(); // a refused call is not waited on
        Ok(Leg {
            word: word.into(),
            accepted: said.ok,
            said: host::last_line(&said.text),
            at,
            waited: sent.elapsed(),
            back,
        })
    }

    /// The legs, one after the other as [`next_word`] says, each line kept (and printed) as soon as the call is answered.
    /// Every call is sent from the bootloader only (a phone on Android is taken there first); a call fastboot refuses stops the run, its line kept.
    pub fn the_legs_are_sent(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let mut legs: Vec<Leg> = Vec::new();
            let mut kept = String::new();
            loop {
                let in_fastboot = u.lab.in_fastboot()?;
                let Some(word) = next_word(&legs, u.lab.is_on_android_now(), in_fastboot) else {
                    break;
                };
                if !in_fastboot {
                    u.lab.phone_is_in_fastboot()?;
                }
                let leg = u.send(word)?;
                let line = leg.line(legs.len() + 1);
                println!("  {line}");
                kept.push_str(&line);
                kept.push('\n');
                u.lab.save(LEGS_FILE, kept.trim_end())?;
                let refused = !leg.accepted;
                legs.push(leg);
                if refused {
                    u.lab.save(VERDICT_FILE, &verdict(&legs))?;
                    return Err(format!(
                        "fastboot refused `{word}`: {}",
                        legs.last().map_or("", |l| l.said.as_str())
                    ));
                }
            }
            let said = verdict(&legs);
            u.lab.save(VERDICT_FILE, &said)?;
            println!("  {said}");
            Ok(())
        }
    }

    /// The way back, the last step of the run, the shared one: the phone ends on Android whatever stopped the run (only the watching parent runs it).
    pub fn way_back(&self) -> impl FnOnce() -> Done + 'static {
        self.root.rehearsal_way_back()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Try;

    fn leg(word: &str, back: bool, secs: u64) -> Leg {
        Leg {
            word: word.into(),
            accepted: true,
            said: "Rebooting...".into(),
            at: 1_000,
            waited: Duration::from_secs(secs),
            back,
        }
    }

    #[test]
    fn the_first_call_is_a_reboot() {
        assert_eq!(next_word(&[], false, true), Some("reboot"));
    }

    #[test]
    fn a_second_reboot_is_sent_only_when_the_phone_still_lists_in_fastboot() {
        let one = [leg("reboot", false, 240)];
        assert_eq!(next_word(&one, false, true), Some("reboot"));
        // not listed and not on Android: the phone is on its way, nothing is sent
        assert_eq!(next_word(&one, false, false), None);
        // back on Android: the second excursion's call
        let back = [leg("reboot", true, 31)];
        assert_eq!(next_word(&back, true, false), Some("continue"));
    }

    #[test]
    fn no_third_reboot_and_one_continue_only() {
        let two = [leg("reboot", false, 240), leg("reboot", false, 240)];
        assert_eq!(next_word(&two, false, true), Some("continue"));
        let all = [leg("reboot", true, 31), leg("continue", true, 40)];
        assert_eq!(next_word(&all, true, false), None);
        assert_eq!(next_word(&all, false, true), None);
    }

    #[test]
    fn a_leg_keeps_the_call_fastboots_words_the_time_and_the_wait() {
        let l = leg("continue", false, 7);
        assert_eq!(
            l.line(3),
            "leg 3: `fastboot continue` accepted at 1000 (epoch s), fastboot said \"Rebooting...\"; Android did not answer 7 s after it was sent"
        );
        assert!(leg("reboot", true, 31)
            .line(1)
            .ends_with("Android answered 31 s after it was sent"));
    }

    #[test]
    fn the_verdict_counts_each_leg_and_says_not_known_when_none_was_sent() {
        assert_eq!(verdict(&[]), "no leg was sent: not known yet");
        assert_eq!(
            verdict(&[leg("reboot", false, 240), leg("reboot", true, 33)]),
            "leg 1 (`fastboot reboot`) did not bring Android back within 240 s; leg 2 (`fastboot reboot`) brought Android back after 33 s; no `fastboot continue` was sent: the phone was neither on Android nor in fastboot, or a call was refused (not known yet)"
        );
    }

    /// A lab whose fastboot lists the phone until `back_after` calls were logged, whose idle-android check passes then, and which answers each call with a line.
    fn lab(name: &str, back_after: u32) -> (RebootTwice, std::path::PathBuf) {
        let d = std::env::temp_dir().join(format!("wt-rt-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("tests/host")).unwrap();
        std::fs::write(d.join("device.toml"), "[connection]\nserial = \"SER1\"\n").unwrap();
        let log = d.join("fastboot.log");
        crate::fixture::executable(
            &d.join("tests/host/idle-android.sh"),
            &format!(
                "[ \"$(cat '{0}' 2>/dev/null | wc -l)\" -ge {back_after} ] && echo 'pass walleye-idle-android host back' && exit 0\necho 'fail not yet'; exit 1\n",
                log.display()
            ),
        );
        crate::fixture::executable(
            &d.join("fastboot"),
            &format!(
                "#!/bin/sh\ncase \"$3\" in reboot|continue) echo \"$3\" >> '{0}'; echo 'Rebooting...'; exit 0;; esac\necho 'SER1 fastboot'\n",
                log.display()
            ),
        );
        let mut s = Settings::stand_in(Try::Touch);
        s.lab = d.clone();
        s.out = d.join("out");
        s.fastboot = d.join("fastboot").to_string_lossy().into_owned();
        s.back_wait = 1;
        (RebootTwice::new(s), log)
    }

    #[test]
    fn a_call_fastboot_refuses_stops_the_run_with_its_line_kept_and_no_wait() {
        let (t, log) = lab("refused", 99);
        let d = t.s.lab.clone();
        crate::fixture::executable(
            &d.join("fastboot"),
            &format!("#!/bin/sh\ncase \"$3\" in reboot|continue) echo \"$3\" >> '{0}'; echo 'FAILED (remote: no)'; exit 1;; esac\necho 'SER1 fastboot'\n", log.display()),
        );
        let e = t.the_legs_are_sent()().unwrap_err();
        assert!(e.contains("fastboot refused `reboot`"), "{e}");
        assert_eq!(calls(&log), ["reboot"]);
        let said = std::fs::read_to_string(t.s.out.join(VERDICT_FILE)).unwrap();
        assert!(said.contains("was refused by fastboot"), "{said}");
    }

    /// Read from the entry files themselves: the run sends legs and reads no go or image; the rehearsal sends none and keeps its own folder.
    #[test]
    fn the_entries_send_legs_only_in_the_real_run_and_read_no_go_or_image() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../procedures/");
        let real = std::fs::read_to_string(format!("{dir}fastboot-reboot-twice-on-the-pixel-2.rs"))
            .unwrap();
        let rehearsal = std::fs::read_to_string(format!(
            "{dir}rehearse-fastboot-reboot-twice-on-the-pixel-2.rs"
        ))
        .unwrap();
        assert!(real.contains("the_legs_are_sent") && !rehearsal.contains("the_legs_are_sent"));
        for text in [&real, &rehearsal] {
            for banned in [
                "author_has_said_go",
                "boot_the_",
                "the_run_image_is_made",
                "the_guard_holds",
            ] {
                assert!(!text.contains(banned), "{banned}");
            }
            assert!(text.contains("t.way_back()"));
        }
        assert!(rehearsal.contains("Settings::for_rehearse_reboot_twice()"));
        assert!(real.contains("Settings::for_reboot_twice()"));
    }

    fn calls(log: &std::path::Path) -> Vec<String> {
        std::fs::read_to_string(log)
            .map(|t| t.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    #[test]
    fn a_phone_that_stays_after_the_first_reboot_gets_a_second_and_then_one_continue() {
        let (t, log) = lab("stays", 2);
        t.the_legs_are_sent()().unwrap();
        assert_eq!(calls(&log), ["reboot", "reboot", "continue"]);
        let kept = std::fs::read_to_string(t.s.out.join(LEGS_FILE)).unwrap();
        assert_eq!(kept.lines().count(), 3, "{kept}");
        assert!(kept.contains("leg 2: `fastboot reboot`"), "{kept}");
        let said = std::fs::read_to_string(t.s.out.join(VERDICT_FILE)).unwrap();
        assert!(
            said.contains("leg 1 (`fastboot reboot`) did not bring Android back"),
            "{said}"
        );
    }

    #[test]
    fn a_phone_that_is_back_after_the_first_reboot_is_sent_no_second() {
        let (t, log) = lab("once", 1);
        // the stand-in lists the phone in fastboot always, so the one continue needs no trip through adb
        t.the_legs_are_sent()().unwrap();
        assert_eq!(calls(&log), ["reboot", "continue"]);
    }
}
