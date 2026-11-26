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
pub const STALE: [&str; 4] = ["read.txt", "dmesg.txt", "open.txt", "verdict.txt"];

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

/// The pins the node and Android's dtb name: gpio12 (Android's enable, nq-ven), gpio116 (the OnePlus 5's enable, and this node's), gpio92 (the interrupt), gpio93 (firmware download, nq-firm).
pub const GPIOS: [u32; 4] = [12, 116, 92, 93];
/// The tool that opens nfc0 (research walleye-nfc-clock-and-minus-121: `nfctool -d nfc0 -1`). The ramdisk's busybox has none and no other tool: when it is not there the open is said not made (not known yet), never guessed.
pub const OPEN_TOOL: &str = "nfctool -d nfc0 -1";

/// The open read over telnet, one line for the shell, marks between the parts: the four pins before the open; the open (its output and `rc=`, or `no-tool`); the four pins after; i2cdetect on the bus of c1b6000
/// after the open (nfctool has exited by then: whether nfc0 stays powered is NOT KNOWN YET; `no-bus` when sysfs names none); the clock summary's ln_bb_clk3 lines.
pub const OPEN_COMMAND: &str = "mount -t debugfs none /sys/kernel/debug 2>/dev/null; P='^ *gpio(12|116|92|93) +:'; echo ---gpio-before; grep -E \"$P\" /sys/kernel/debug/gpio; echo ---open; if command -v nfctool >/dev/null; then timeout 20 nfctool -d nfc0 -1 2>&1; echo rc=$?; else echo no-tool; fi; echo ---gpio-after; grep -E \"$P\" /sys/kernel/debug/gpio; echo ---i2c; B=$(ls /sys/bus/platform/devices/c1b6000.i2c 2>/dev/null | grep '^i2c-' | head -1 | cut -c5-); if [ -n \"$B\" ]; then i2cdetect -y $B 2>&1; else echo no-bus; fi; echo ---clock; grep -i ln_bb_clk3 /sys/kernel/debug/clk/clk_summary 2>/dev/null; echo ---end";

/// The pins of [`GPIOS`] in a pin table (the msm pinctrl's `gpioN : in|out VALUE DRIVE PULL`), each as its number and its words in single spaces; a pin not listed is not in the answer.
pub fn pin_lines(text: &str) -> Vec<(u32, String)> {
    let mut found: Vec<(u32, String)> = text
        .lines()
        .filter_map(|l| {
            let (name, rest) = l.trim().split_once(':')?;
            let n: u32 = name.trim().strip_prefix("gpio")?.parse().ok()?;
            GPIOS
                .contains(&n)
                .then(|| (n, rest.split_whitespace().collect::<Vec<_>>().join(" ")))
        })
        .collect();
    found.sort_by_key(|(n, _)| GPIOS.iter().position(|g| g == n));
    found
}

/// What the answers of the open point to (research walleye-nfc-clock-and-minus-121: no ACK at 0x28 means VEN; an ACK and no NCI reply means the clock; firmware framing means gpio93 polarity).
#[derive(Debug, PartialEq, Clone)]
pub enum Cause {
    /// nfc0 was not opened: no tool in the ramdisk, or the read has none of it. Not known yet.
    NotOpened,
    /// no ACK at 0x28, or the open's own answer is a write to a chip that is not there
    Ven,
    /// the chip is on the bus (an ACK, or the driver holds the address) and no NCI reply came
    Clock,
    /// the open's answer speaks of firmware framing
    Gpio93,
    /// nfc0 opened and the chip answered: none of the three
    Answers,
    /// the open failed in a way none of the rules names; its first line is kept
    Unnamed(String),
}

/// The answer of the open read: the pins before and after, the cell of i2cdetect at 0x28, the cause.
#[derive(Debug, PartialEq)]
pub struct OpenAnswer {
    pub cause: Cause,
    pub before: Vec<(u32, String)>,
    pub after: Vec<(u32, String)>,
    /// the pins whose line changed with the open
    pub moved: Vec<u32>,
    /// the cell of the row 20: at column 8 (0x28): `--` no ACK, `UU` the address is held by a driver, `28` an ACK; None: the bus was not found or the table is unreadable
    pub cell: Option<String>,
    pub clocks: Vec<String>,
}

/// The cell for 0x28 in busybox's i2cdetect table (its rows: `20: ` then one cell per address, `--`, `UU` or the address in hex).
fn cell_28(part: &[&str]) -> Option<String> {
    let row = part.iter().find(|l| l.trim_start().starts_with("20:"))?;
    row.trim_start()
        .strip_prefix("20:")?
        .split_whitespace()
        .nth(8)
        .map(str::to_string)
}

fn the_cause(opened: &[&str], cell: &Option<String>) -> Cause {
    let said = opened.join(" ").to_ascii_lowercase();
    if said.trim() == "no-tool" || said.trim().is_empty() {
        return Cause::NotOpened;
    }
    if said.contains("rc=0") {
        return Cause::Answers;
    }
    // the bus first: no ACK at 0x28 is VEN whatever the open said
    if cell.as_deref() == Some("--") {
        return Cause::Ven;
    }
    let acked = matches!(cell.as_deref(), Some("28") | Some("UU"));
    if said.contains("firmware") {
        return Cause::Gpio93;
    }
    // `timeout 20` ends a hung nfctool with 124 (143 on some busybox): a chip that ACKs and never answers NCI
    let hung = said.contains("rc=124") || said.contains("rc=143");
    if hung || said.contains("timed out") || said.contains("timeout") || said.contains("-110") {
        return Cause::Clock;
    }
    // without a table of the bus the open's own words are all there is
    let ven_words = ["no such device", "remote i/o", "-121", "-6)"];
    if !acked && ven_words.iter().any(|w| said.contains(w)) {
        return Cause::Ven;
    }
    if acked {
        return Cause::Clock;
    }
    Cause::Unnamed(
        opened
            .first()
            .map(|l| l.trim().to_string())
            .unwrap_or_default(),
    )
}

/// The open read judged. Only a read that has not got its marks is an error; whatever it says is the answer.
pub fn open_judge(read: &str) -> Result<OpenAnswer, String> {
    let part = |name: &str| section(read, name).ok_or(format!("the open read has no {name} part"));
    let (before, open, after, i2c, clock) = (
        part("gpio-before")?,
        part("open")?,
        part("gpio-after")?,
        part("i2c")?,
        part("clock")?,
    );
    let before_pins = pin_lines(&before.join("\n"));
    let after_pins = pin_lines(&after.join("\n"));
    let moved = after_pins
        .iter()
        .filter(|(n, now)| before_pins.iter().any(|(b, was)| b == n && was != now))
        .map(|(n, _)| *n)
        .collect();
    let cell = cell_28(&i2c);
    Ok(OpenAnswer {
        cause: the_cause(&open, &cell),
        before: before_pins,
        after: after_pins,
        moved,
        cell,
        clocks: clock
            .iter()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && !l.contains("No such file") && !l.contains("can't"))
            .collect(),
    })
}

impl OpenAnswer {
    /// The line the research row records: what was counted, then the cause the answers point to.
    pub fn said(&self) -> String {
        let pins = |p: &[(u32, String)]| {
            if p.is_empty() {
                "not known yet (no pin table)".to_string()
            } else {
                p.iter()
                    .map(|(n, s)| format!("gpio{n} {s}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        };
        let cell = match self.cell.as_deref() {
            Some("--") => "no ACK at 0x28".to_string(),
            Some("UU") => {
                "0x28 is held by the driver (UU: i2cdetect cannot tell its ACK)".to_string()
            }
            Some(c) => format!("0x28 answers ({c})"),
            None => "0x28 not known yet (no i2cdetect table)".to_string(),
        };
        let moved = if self.moved.is_empty() {
            "no pin moved with the open".to_string()
        } else {
            format!(
                "moved with the open: {}",
                self.moved
                    .iter()
                    .map(|n| format!("gpio{n}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let clock = if self.clocks.is_empty() {
            "ln_bb_clk3 not known yet (not listed)".to_string()
        } else {
            format!("clock: {}", self.clocks.join(" | "))
        };
        let cause = match &self.cause {
            Cause::NotOpened => "cause not known yet: nfc0 was not opened (no tool in the ramdisk)".to_string(),
            Cause::Ven => "points to VEN (no ACK: the enable pin does not switch the chip on; the pins that moved with the open say which of gpio12 and gpio116 is it)".to_string(),
            Cause::Clock => "points to the clock (the chip is there and gave no NCI reply)".to_string(),
            Cause::Gpio93 => "points to gpio93 (firmware framing)".to_string(),
            Cause::Answers => "nfc0 opened and the chip answered: neither VEN, clock nor gpio93".to_string(),
            Cause::Unnamed(l) => format!("cause not known yet: the open said `{l}`"),
        };
        format!(
            "before: {}; after: {}; {moved}; {cell}; {clock}; {cause}",
            pins(&self.before),
            pins(&self.after)
        )
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

    /// nfc0 is opened and the pins, the bus and the clock are read around it (the open read, [`OPEN_COMMAND`]); the answer is kept in open.txt and said. Whatever it says, the step passes:
    /// no tool to open nfc0 is the answer "not known yet", never a failed try.
    pub fn the_nfc0_is_opened_and_the_pins_read(&self) -> impl FnOnce() -> Done + 'static {
        let n = self.clone();
        move || {
            let mut sh = n.lab.shell_when_up()?;
            let read = sh.run(OPEN_COMMAND, Duration::from_secs(60))?;
            n.lab.save("open.txt", &crate::touch::redact(&read))?;
            // a read without its marks is a cut read: said not known yet, so the way back still runs
            let said = match open_judge(&read) {
                Ok(a) => a.said(),
                Err(e) => format!("open read not known yet ({e}); the raw read is in open.txt"),
            };
            let path = n.s.out.join("verdict.txt");
            let first =
                std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            n.lab
                .save("verdict.txt", &format!("{}\n{said}", first.trim_end()))?;
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
        assert!(
            real.contains("steps::phone_ready(KERNEL)")
                && real.contains("the_nfc_is_read")
                && real.contains("the_nfc0_is_opened_and_the_pins_read")
        );
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

    /// The phone's own pin table (Android, slot b, 2026-11-25, tests/fixtures/phone-text/SOURCES.toml): the msm pinctrl's line format, the one /sys/kernel/debug/gpio prints on mainline too (drivers/pinctrl/qcom/pinctrl-msm.c, msm_gpio_dbg_show_one).
    const GPIO_CAPTURE_BYTES: &[u8] = include_bytes!(
        "../../../tests/fixtures/phone-text/part-compare-touch-android-b-gpio-2026-11-25.cap"
    );

    /// The open read, in the marks of [`OPEN_COMMAND`]. The i2cdetect rows follow busybox's own format (`%02x: ` then `-- `, `UU ` or the address): a stand-in, NOT captured from a phone.
    fn open_read(before: &str, open: &str, after: &str, i2c28: &str) -> String {
        format!("---gpio-before\n{before}\n---open\n{open}\n---gpio-after\n{after}\n---i2c\n     0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f\n20: -- -- -- -- -- -- -- -- {i2c28} -- -- -- -- -- -- --\n---clock\n ln_bb_clk3 0 0 0 19200000 0 0 50000\n---end")
    }

    fn capture() -> String {
        String::from_utf8_lossy(GPIO_CAPTURE_BYTES).to_string() // the capture holds a few bytes that are no utf-8 (a gpio-1002 name read as it was)
    }

    fn pins(text: &str) -> String {
        text.lines()
            .filter(|l| {
                ["gpio12 ", "gpio116 ", "gpio92 ", "gpio93 "]
                    .iter()
                    .any(|p| l.trim_start().starts_with(p))
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn the_pin_lines_are_read_in_the_phones_own_format() {
        let p = pin_lines(&capture());
        assert_eq!(p.len(), 4, "{p:?}");
        assert_eq!(p[0].0, 12);
        assert!(p[0].1.starts_with("out 0 2mA pull down"), "{p:?}");
        assert_eq!(p.iter().map(|x| x.0).collect::<Vec<_>>(), GPIOS.to_vec());
        // gpio1 is not gpio12, gpio11 not gpio116: a number is matched whole
        assert!(pin_lines("gpio120 : in 0 2mA no pull\ngpio1  : in 0 2mA no pull").is_empty());
    }

    #[test]
    fn the_open_command_reads_the_pins_before_and_after_and_names_the_bus_of_the_node() {
        for part in ["gpio-before", "open", "gpio-after", "i2c", "clock", "end"] {
            assert!(OPEN_COMMAND.contains(&format!("echo ---{part}")), "{part}");
        }
        assert!(
            OPEN_COMMAND.find("---gpio-before").unwrap() < OPEN_COMMAND.find("---open").unwrap()
        );
        assert!(
            OPEN_COMMAND.find("---open").unwrap() < OPEN_COMMAND.find("---gpio-after").unwrap()
        );
        assert!(
            OPEN_COMMAND.contains("nfctool -d nfc0 -1") && OPEN_COMMAND.contains("c1b6000.i2c")
        );
        assert!(OPEN_COMMAND.contains("i2cdetect") && OPEN_COMMAND.contains("ln_bb_clk3"));
        for n in GPIOS {
            assert!(OPEN_COMMAND.contains(&n.to_string()), "{n}");
        }
        assert!(!OPEN_COMMAND.contains('\n'));
    }

    #[test]
    fn no_tool_is_not_known_yet_and_never_a_cause() {
        let before = pins(&capture());
        let a = open_judge(&open_read(&before, "no-tool", &before, "UU")).unwrap();
        assert_eq!(a.cause, Cause::NotOpened);
        assert!(a.said().contains("not known yet"), "{}", a.said());
    }

    #[test]
    fn no_ack_at_28_points_to_ven() {
        let before = pins(&capture());
        let a = open_judge(&open_read(
            &before,
            "nfctool: Power on failed: No such device or address\nrc=1",
            &before,
            "--",
        ))
        .unwrap();
        assert_eq!(a.cause, Cause::Ven);
    }

    #[test]
    fn an_ack_and_no_nci_reply_points_to_the_clock() {
        let before = pins(&capture());
        let a = open_judge(&open_read(
            &before,
            "nfctool: Power on failed: Connection timed out\nrc=1",
            &before,
            "28",
        ))
        .unwrap();
        assert_eq!(a.cause, Cause::Clock);
        // with the driver bound i2cdetect prints UU: the bus ACK is not its to tell, the open's own answer is
        let b = open_judge(&open_read(
            &before,
            "nfctool: Power on failed: Connection timed out\nrc=1",
            &before,
            "UU",
        ))
        .unwrap();
        assert_eq!(b.cause, Cause::Clock);
        assert!(b.said().contains("UU"), "{}", b.said());
    }

    #[test]
    fn a_firmware_answer_points_to_gpio93_and_an_open_that_works_to_none() {
        let before = pins(&capture());
        let a = open_judge(&open_read(
            &before,
            "nfctool: firmware download mode answered\nrc=1",
            &before,
            "UU",
        ))
        .unwrap();
        assert_eq!(a.cause, Cause::Gpio93);
        let ok = open_judge(&open_read(&before, "rc=0", &before, "UU")).unwrap();
        assert_eq!(ok.cause, Cause::Answers);
    }

    #[test]
    fn the_pins_that_moved_with_the_open_are_named() {
        let before = pins(&capture());
        let after = before.replace("gpio116 : in  0", "gpio116 : out 1");
        assert_ne!(
            before, after,
            "the capture has a gpio116 line that reads in 0"
        );
        let a = open_judge(&open_read(&before, "rc=0", &after, "UU")).unwrap();
        assert_eq!(a.moved, vec![116]);
        assert!(
            a.said().contains("moved with the open: gpio116"),
            "{}",
            a.said()
        );
    }

    #[test]
    fn a_read_without_its_marks_is_an_error() {
        assert!(open_judge("nothing").is_err());
    }

    #[test]
    fn the_open_command_is_a_shell_line_that_parses() {
        let ok = std::process::Command::new("sh")
            .arg("-n")
            .arg("-c")
            .arg(OPEN_COMMAND)
            .status()
            .unwrap();
        assert!(ok.success());
    }

    #[test]
    fn an_ack_outranks_ven_words_and_a_hung_tool_is_the_clock() {
        let before = pins(&capture());
        let a = open_judge(&open_read(
            &before,
            "nfctool: Remote I/O error\nrc=1",
            &before,
            "28",
        ))
        .unwrap();
        assert_eq!(a.cause, Cause::Clock);
        let h = open_judge(&open_read(&before, "rc=124", &before, "UU")).unwrap();
        assert_eq!(h.cause, Cause::Clock);
        let w = open_judge(&open_read(
            &before,
            "nfctool: Remote I/O error\nrc=1",
            &before,
            "--",
        ))
        .unwrap();
        assert_eq!(w.cause, Cause::Ven);
    }

    #[test]
    fn without_a_bus_table_the_open_words_speak_and_the_rest_is_unnamed() {
        let before = pins(&capture());
        let no_bus =
            |r: String| r.replace(r.lines().find(|l| l.starts_with("20:")).unwrap(), "no-bus");
        let v = open_judge(&no_bus(open_read(
            &before,
            "nfctool: No such device or address\nrc=1",
            &before,
            "--",
        )))
        .unwrap();
        assert_eq!(v.cell, None);
        assert_eq!(v.cause, Cause::Ven);
        let u = open_judge(&no_bus(open_read(&before, "rc=127", &before, "--"))).unwrap();
        assert_eq!(u.cause, Cause::Unnamed("rc=127".into()));
        assert!(
            u.said()
                .contains("cause not known yet: the open said `rc=127`"),
            "{}",
            u.said()
        );
        assert!(u.said().contains("0x28 not known yet"), "{}", u.said());
    }
}
