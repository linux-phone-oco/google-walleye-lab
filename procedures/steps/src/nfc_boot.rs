// SPDX-License-Identifier: MIT OR Apache-2.0
//! The NFC boot (item walleye-nfc-boot-procedure; decision 2026-11-26-walleye-ramdisk-on-net-43; research walleye-nfc-image-built):
//! the NFC image ([images.nfc]: the ufs-off kernel with NFC built in, the dtb with the nfc@28 node on blsp2_i2c2, the roof's init at walleye's net 43) is
//! booted ONCE from memory by the library's group `steps::phone_ready("nfc")` (nothing flashed by this procedure), the gadget's serial is checked, and over
//! the image's telnet the lab runs the health check `tests/health/base/nfc-present.sh` and reads the kernel's lines on nxp-nci. The deferred devices and the clock lines are read too (said in the verdict when nxp-nci never
//! probed), because the controller's clock (Android: BBCLK3, pm8998 gpio21 clkreq) is NOT KNOWN YET.
//! Whatever the phone says is the answer, kept in the run's folder (read.txt, dmesg.txt, verdict.txt); the way back is the library's `steps::leave_known_good`.
//!
//! Where each judged word comes from (no text of the NFC image running was captured yet: the first real run is the capture):
//!  - the health line `pass|fail|skip TEST COMPONENT detail`: lab-tools health/suite/lib.sh, `say` and `check` (copied below as [`LIB`]); the test and component names
//!    `nfc-present nfc`: tests/health/base/nfc-present.sh itself, which this procedure runs from the lab's own copy;
//!  - the kernel's driver name `nxp-nci`: drivers/nfc/nxp-nci/i2c.c (`NXP_NCI_I2C_DRIVER_NAME "nxp-nci_i2c"`, the i2c driver the `nxp,nxp-nci-i2c` node binds) and
//!    drivers/nfc/nxp-nci/core.c (`nci_register_device`, the nfc0 of /sys/class/nfc);
//!  - no phone text with those words exists yet; the judge is tested against a log a phone printed that has none of them (a boot that never probed the part).

use crate::host;
use crate::lab::Lab;
use crate::settings::Settings;
use std::rc::Rc;
use std::time::Duration;

type Done = Result<(), String>;

/// The test kernel (its `[images.NAME]` and `[kernels] test` entry) the try boots: steps::phone_ready names it.
pub const KERNEL: &str = "nfc";
/// The lab's health check this procedure runs on the phone, from the lab's own copy.
pub const HEALTH_CHECK: &str = "tests/health/base/nfc-present.sh";
/// The health line this check says: its test and component.
pub const HEALTH_TEST: &str = "nfc-present";
/// The files of the run's folder; an earlier run of the day's are forgotten at the start (a stale answer is never this run's).
pub const STALE: [&str; 3] = ["read.txt", "dmesg.txt", "verdict.txt"];

/// `say` and `check` of the health suite's lib.sh (lab-tools health/suite/lib.sh), one line each: the check sources that file on a phone with the suite, and the image's shell has none.
pub const LIB: &str = "say() { printf '%s %s %s %s\\n' \"$3\" \"$1\" \"$2\" \"$(printf '%s' \"$4\" | tr '\\n' ' ')\"; }; check() { if [ \"$3\" = true ]; then say \"$1\" \"$2\" pass \"$4\"; else say \"$1\" \"$2\" fail \"$4\"; fi; }";

/// The one command that runs the health check on the image's shell: [`LIB`] in place of the line that sources it, then the check's own lines. The shell takes one line, so the lines are joined by `;`.
/// A check with no line left to run is an error: nothing would be measured.
pub fn health_command(script: &str) -> Result<String, String> {
    let body: Vec<&str> = script
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with(". "))
        .collect();
    if body.is_empty() {
        return Err(format!("{HEALTH_CHECK} has no line to run"));
    }
    Ok(format!("{LIB}; {}", body.join("; ")))
}

/// The read over telnet, after the health check: the kernel's lines on the controller and its bus; the deferred devices; the clock lines of the clock summary. Marks between the parts.
pub const READ_COMMAND: &str = "mount -t debugfs none /sys/kernel/debug 2>/dev/null; echo ---dmesg; dmesg | grep -i -e nxp -e nci -e nfc -e pn553 -e c1b6000 -e blsp_i2c8; echo ---deferred; cat /sys/kernel/debug/devices_deferred 2>&1; echo ---clock; (grep -i -e bb_clk -e bbclk -e ln_bb /sys/kernel/debug/clk/clk_summary 2>&1); echo ---end";

/// What the health line said.
#[derive(Debug, PartialEq, Clone)]
pub enum Health {
    Pass(String),
    Fail(String),
    /// the check could not measure: not known yet, never broken
    Skip(String),
}

/// The health line of nfc-present in what the phone printed: `pass|fail|skip nfc-present nfc DETAIL`. No such line is an error: nothing is concluded.
pub fn health_in(text: &str) -> Result<Health, String> {
    let line = text
        .lines()
        .map(str::trim)
        .find(|l| {
            let w: Vec<&str> = l.split_whitespace().collect();
            w.len() >= 3 && ["pass", "fail", "skip"].contains(&w[0]) && w[1] == HEALTH_TEST
        })
        .ok_or_else(|| {
            format!(
                "the phone printed no `{HEALTH_TEST}` health line; it said: {}",
                text.lines().next().unwrap_or("nothing").trim()
            )
        })?;
    let detail = line.splitn(4, ' ').nth(3).unwrap_or("").trim().to_string();
    Ok(match line.split_whitespace().next() {
        Some("pass") => Health::Pass(detail),
        Some("skip") => Health::Skip(detail),
        _ => Health::Fail(detail),
    })
}

fn section<'a>(text: &'a str, name: &str) -> Option<Vec<&'a str>> {
    let mark = format!("---{name}");
    let mut lines = text.lines().map(str::trim_end);
    lines.find(|l| *l == mark)?;
    Some(lines.take_while(|l| !l.starts_with("---")).collect())
}

/// What the kernel's lines say about the controller: a line names the nxp-nci driver, and the lines that tell of a failure about it.
#[derive(Debug, PartialEq, Default)]
pub struct Log {
    pub mentions: bool,
    pub failures: Vec<String>,
}

/// The kernel's lines (a whole log, or the grep of it that [`READ_COMMAND`] makes) judged for the controller. A failure is a line about nxp-nci, nci, nfc or pn553 that says `fail`, `error`, `timed out`, `defer` or `-ENXIO`-like `no such device`.
pub fn log_says(text: &str) -> Log {
    let mut log = Log::default();
    for line in text.lines() {
        let l = line.to_ascii_lowercase();
        if l.contains("nxp-nci") || l.contains("nxp_nci") {
            log.mentions = true;
        }
        let about = ["nxp", "nci", "nfc", "pn553"].iter().any(|w| l.contains(w));
        if about
            && [
                "fail",
                "error",
                "timed out",
                "timeout",
                "defer",
                "no such device",
                "unable",
            ]
            .iter()
            .any(|w| l.contains(w))
        {
            log.failures.push(line.trim().to_string());
        }
    }
    log
}

/// What the boot read, whole.
#[derive(Debug, PartialEq)]
pub struct Answer {
    pub health: Health,
    pub log: Log,
    /// the lines the deferred-devices list names the controller or its bus in
    /// false when the deferred list could not be opened: then nothing is said of it
    pub deferred_read: bool,
    pub deferred: Vec<String>,
    /// the clock-summary lines of the baseband clocks (empty: none listed or debugfs gave none)
    pub clocks: Vec<String>,
}

fn named(lines: Option<Vec<&str>>) -> Vec<String> {
    lines
        .unwrap_or_default()
        .into_iter()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

/// The answer of the health line and the read. Only a read that has not got its parts is an error.
pub fn judge(health_text: &str, read: &str) -> Result<Answer, String> {
    let health = health_in(health_text)?;
    let log = log_says(
        &section(read, "dmesg")
            .ok_or("the read has no dmesg part")?
            .join("\n"),
    );
    let raw = named(section(read, "deferred"));
    let deferred_read = !raw.iter().any(|l| {
        l.contains("No such file") || l.contains("can't open") || l.contains("Permission denied")
    });
    let deferred = raw
        .into_iter()
        .filter(|l| {
            ["nci", "nfc", "nxp", "c1b6000", "pn553"]
                .iter()
                .any(|w| l.to_ascii_lowercase().contains(w))
        })
        .collect();
    let clocks = named(section(read, "clock"))
        .into_iter()
        .filter(|l| !l.contains("No such file") && !l.contains("can't"))
        .collect();
    Ok(Answer {
        health,
        log,
        deferred_read,
        deferred,
        clocks,
    })
}

impl Answer {
    /// The line the research row records: what was counted, not why. The clock is read when nxp-nci did not probe or nfc0 is absent.
    pub fn said(&self) -> String {
        let health = match &self.health {
            Health::Pass(d) => format!("nfc-present passes ({d})"),
            Health::Fail(d) => format!("nfc-present FAILS ({d})"),
            Health::Skip(d) => format!("nfc-present not known yet ({d})"),
        };
        let log = if self.log.mentions {
            "the kernel names nxp-nci".to_string()
        } else {
            "the kernel never names nxp-nci".to_string()
        };
        let failures = if self.log.failures.is_empty() {
            "no failure line".to_string()
        } else {
            format!(
                "{} failure line(s), the first: {}",
                self.log.failures.len(),
                self.log.failures[0]
            )
        };
        let deferred = if !self.deferred_read {
            "the deferred list is not known yet (it could not be opened)".to_string()
        } else if self.deferred.is_empty() {
            "the controller is not on the deferred list".to_string()
        } else {
            format!("deferred: {}", self.deferred.join(" | "))
        };
        let clock = if self.needs_the_clock() {
            if self.clocks.is_empty() {
                "baseband clocks not known yet (none listed)".to_string()
            } else {
                format!("clock lines: {}", self.clocks.join(" | "))
            }
        } else {
            "clock not read (the controller probed)".to_string()
        };
        format!("{health}; {log}; {failures}; {deferred}; {clock}")
    }

    /// The clock is read when the controller did not probe: nxp-nci not named, or the health line is not a pass.
    pub fn needs_the_clock(&self) -> bool {
        !self.log.mentions || !matches!(self.health, Health::Pass(_))
    }
}

/// What the author is told before the run.
pub fn expect() -> String {
    "The NFC boot, about 6 minutes: the NFC image is booted once from memory (UFS off by the gate, nothing flashed by this procedure), the gadget's serial walleye-first-boot is checked, and over its telnet at walleye's net 43 the lab runs nfc-present and reads the kernel's lines on nxp-nci (the deferred devices and the clock lines are read too, and said when it did not probe). Whatever the phone says is the answer. The roof's init has no hold: the way back is the shared one, straight after the read.".to_string()
}

/// The try: its settings and the lab. Cheap to clone: each step takes its own.
#[derive(Clone)]
pub struct NfcBoot {
    s: Rc<Settings>,
    lab: Lab,
}

impl NfcBoot {
    pub fn new(s: Settings) -> NfcBoot {
        let s = Rc::new(s);
        NfcBoot {
            lab: Lab::new(s.clone()),
            s,
        }
    }

    /// The battery is above the lab's minimum (the shared step), after the files of an earlier run of the day are forgotten.
    pub fn battery_is_good(&self) -> impl FnOnce() -> Done + 'static {
        let n = self.clone();
        move || {
            for name in STALE {
                let _ = std::fs::remove_file(n.s.out.join(name));
            }
            n.lab.battery_is_good()
        }
    }

    pub fn author_is_told(&self) -> impl FnOnce() -> Done + 'static {
        let text = expect();
        move || {
            println!("  {text}");
            Ok(())
        }
    }

    /// The image is the declared one (its sha256 is the manifest's, and the one its SHA256SUMS line records) and UFS is off in it.
    pub fn the_image_is_the_declared_ufs_off_image(&self) -> impl FnOnce() -> Done + 'static {
        let n = self.clone();
        move || {
            host::image_has_sha256(&n.s.image, &n.s.image_sha256)?;
            host::image_is_the_built_one(&n.s.image)?;
            n.lab.ufs_is_off(&n.s.image)
        }
    }

    /// The phone is in fastboot (the shared step); nothing is written to it.
    pub fn phone_is_in_fastboot(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.phone_is_in_fastboot()
    }

    /// The gadget is up on USB under the roof's serial and the shell answers at walleye's own address (the lab's gadget-up check, waiting up to a minute).
    pub fn the_gadget_is_up(&self) -> impl FnOnce() -> Done + 'static {
        let n = self.clone();
        move || {
            let toml = n.s.device_toml();
            n.lab.script(
                "gadget-up.sh",
                &[&toml.to_string_lossy()],
                &[("GADGET_WAIT", "60")],
            )?;
            n.lab.shell_when_up().map(|_| ())
        }
    }

    /// The watching parent's way back (the Ctrl-C safety): the phone ends on Android whatever stopped the run (the shared step).
    pub fn way_back(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.way_back()
    }

    /// The health check is run on the phone, the kernel log is saved whole, then the part reads; the answer is judged and kept. Whatever the answer is, the step passes.
    pub fn the_nfc_is_read(&self) -> impl FnOnce() -> Done + 'static {
        let n = self.clone();
        move || {
            let path = n.s.lab.join(HEALTH_CHECK);
            let script =
                std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let command = health_command(&script)?;
            let mut sh = n.lab.shell_when_up()?;
            let health = sh.run(&command, Duration::from_secs(30))?;
            let read = sh.run(READ_COMMAND, Duration::from_secs(60))?;
            n.lab.save(
                "read.txt",
                &crate::touch::redact(&format!("{health}\n{read}")),
            )?;
            let dmesg = sh.run("dmesg", Duration::from_secs(20))?;
            if dmesg.trim().is_empty() {
                return Err("the shell answered, but dmesg is empty".into());
            }
            n.lab.save("dmesg.txt", &crate::touch::redact(&dmesg))?;
            let said = judge(&health, &read)?.said();
            n.lab.save("verdict.txt", &said)?;
            println!("  {said}");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The lab's own health check, as it is in this repository (the source of the test and component names).
    const CHECK: &str = include_str!("../../../tests/health/base/nfc-present.sh");
    /// A real kernel log of our image (UFS root boot, 2026-11-25): a part that never probed in a boot of ours; it has no nxp-nci line.
    const CAPTURED_LOG: &str =
        include_str!("../../../tests/fixtures/phone-text/dmesg-ufs-root-20261125-043720.cap");

    /// Run a health command on this host's own sh: the lib's `say` and `check` are the code that prints the line, whatever the host.
    fn on_host(command: &str) -> String {
        let out = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    fn read(dmesg: &str, deferred: &str, clock: &str) -> String {
        format!("---dmesg\n{dmesg}\n---deferred\n{deferred}\n---clock\n{clock}\n---end")
    }

    #[test]
    fn the_command_is_the_checks_own_lines_after_the_libs() {
        let c = health_command(CHECK).unwrap();
        assert!(c.starts_with("say() {") && c.contains("check() {"), "{c}");
        assert!(
            c.contains("check nfc-present nfc \"$(ls /sys/class/nfc/nfc0"),
            "{c}"
        );
        assert!(
            !c.contains("lib.sh") && !c.contains('\n'),
            "one line, no source line: {c}"
        );
        assert!(health_command("# only a comment\n. /x/lib.sh\n").is_err());
    }

    #[test]
    fn the_lib_prints_what_the_judge_reads_on_a_host_without_nfc() {
        // the real script text, the lib's real functions, this host's sh: no /sys/class/nfc/nfc0 here, so the line is a fail
        let h = health_in(&on_host(&health_command(CHECK).unwrap())).unwrap();
        assert_eq!(h, Health::Fail("nfc0 from nxp-nci".into()));
    }

    #[test]
    fn a_passing_condition_prints_a_pass_line() {
        let h = health_in(&on_host(&format!(
            "{LIB}; check nfc-present nfc true 'nfc0 from nxp-nci'"
        )))
        .unwrap();
        assert_eq!(h, Health::Pass("nfc0 from nxp-nci".into()));
    }

    #[test]
    fn no_health_line_is_an_error_naming_what_the_phone_said() {
        let e = health_in("sh: check: not found").unwrap_err();
        assert!(
            e.contains("no `nfc-present` health line") && e.contains("not found"),
            "{e}"
        );
        assert_eq!(
            health_in("skip nfc-present nfc no tool"),
            Ok(Health::Skip("no tool".into()))
        );
    }

    #[test]
    fn a_boot_of_ours_that_never_probed_the_part_is_said_so() {
        // source: the captured kernel log; no nxp-nci line is in it
        assert_eq!(log_says(CAPTURED_LOG), Log::default());
        let a = judge("fail nfc-present nfc nfc0 from nxp-nci", &read("", "", "")).unwrap();
        assert!(a.needs_the_clock());
        assert_eq!(
            a.said(),
            "nfc-present FAILS (nfc0 from nxp-nci); the kernel never names nxp-nci; no failure line; the controller is not on the deferred list; baseband clocks not known yet (none listed)"
        );
    }

    #[test]
    fn a_probed_controller_reads_no_clock() {
        // the kernel's driver names (i2c.c, core.c), filled by hand: NOT captured from a phone
        let a = judge(
            "pass nfc-present nfc nfc0 from nxp-nci",
            &read("nxp-nci_i2c 1-0028: NFC: nci registered", "", ""),
        )
        .unwrap();
        assert!(!a.needs_the_clock());
        assert!(
            a.said().ends_with("clock not read (the controller probed)"),
            "{}",
            a.said()
        );
    }

    #[test]
    fn a_failed_probe_and_a_deferred_controller_are_kept() {
        let a = judge(
            "fail nfc-present nfc nfc0 from nxp-nci",
            &read("nxp-nci_i2c 6-0028: probe with driver nxp-nci_i2c failed with error -110\nunrelated: error 5", "6-0028 nxp-nci_i2c: probe deferral\nother 1-0001: x", "bb_clk3 0 0 0 19200000 0 0 50000"),
        )
        .unwrap();
        assert_eq!(a.log.failures.len(), 1, "{:?}", a.log.failures);
        assert_eq!(a.deferred.len(), 1);
        assert_eq!(a.clocks.len(), 1);
        assert!(a.said().contains("clock lines: bb_clk3"), "{}", a.said());
    }

    #[test]
    fn an_unreadable_deferred_list_is_not_known_yet_and_debugfs_is_mounted_before_it_is_read() {
        let a = judge(
            "fail nfc-present nfc x",
            &read(
                "",
                "cat: can't open '/sys/kernel/debug/devices_deferred': No such file or directory",
                "",
            ),
        )
        .unwrap();
        assert!(!a.deferred_read);
        assert!(
            a.said().contains("the deferred list is not known yet"),
            "{}",
            a.said()
        );
        assert!(
            READ_COMMAND.find("mount -t debugfs").unwrap()
                < READ_COMMAND.find("devices_deferred").unwrap()
        );
    }

    #[test]
    fn the_read_command_has_the_marks_the_judge_reads() {
        for part in ["dmesg", "deferred", "clock", "end"] {
            assert!(READ_COMMAND.contains(&format!("echo ---{part}")), "{part}");
        }
        assert!(judge("pass nfc-present nfc x", "nothing").is_err());
    }

    /// Read from the entry file itself: it boots the declared image through the library's group and ends in the shared way back; it reads no go.
    #[test]
    fn the_entry_boots_the_image_and_leaves_known_good_last() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../procedures/");
        let real = std::fs::read_to_string(format!("{dir}nfc-boot-on-the-pixel-2.rs")).unwrap();
        assert!(real.contains("steps::phone_ready(KERNEL)") && real.contains("the_nfc_is_read"));
        assert!(
            real.contains("the_image_is_the_declared_ufs_off_image")
                && real.contains("Settings::for_nfc_boot()")
        );
        for banned in ["author_has_said_go", "set_active", "the_guard_holds"] {
            assert!(!real.contains(banned), "{banned}");
        }
        let last = real
            .lines()
            .rfind(|l| {
                l.trim_start().starts_with("run_group")
                    || l.trim_start().starts_with("check_from")
                    || l.trim_start().starts_with("step(")
            })
            .unwrap();
        assert!(last.contains("steps::leave_known_good()"), "{last}");
    }
}
