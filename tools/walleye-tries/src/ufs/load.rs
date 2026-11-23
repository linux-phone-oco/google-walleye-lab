// SPDX-License-Identifier: MIT OR Apache-2.0
//! UFS try 2's new part (item walleye-ufs-try-2-one-command): the link gear and the clock read idle, DURING a bounded
//! read-only O_DIRECT read of one disk, and 2 s after it; the disk's GPT hashed before and after; the slots' flags before
//! and after. Nothing is written to the card: the read's output goes to /dev/null, nothing is mounted.

use super::{Link, Speed, Ufs};
use crate::phone::Shell;
use crate::touch::redact;
use std::time::{Duration, Instant};

type Done = Result<(), String>;

/// The clock nodes of devfreq, one `clock_NAME=HZ` line each; absent nodes print nothing (the clock is then not known yet).
pub const CLOCK_COMMAND: &str =
    "for f in /sys/class/devfreq/*/cur_freq; do [ -e $f ] || continue; echo \"clock_$(basename $(dirname $f))=$(cat $f)\"; done 2>/dev/null";

/// The disk is one `sd` letter name: the only thing a command is built from.
pub fn lun_is_a_disk(lun: &str) -> bool {
    lun.strip_prefix("sd")
        .is_some_and(|r| r.len() == 1 && r.chars().all(|c| c.is_ascii_lowercase()))
}

/// The gear command: the link's `power_info` lines and the clock lines.
pub fn gear_command() -> String {
    format!("{}; {CLOCK_COMMAND}", super::LINK_COMMAND)
}

/// The link and the clock of one reading.
#[derive(Debug, Clone, PartialEq)]
pub struct Gear {
    pub link: Link,
    /// `(devfreq name, Hz text)`
    pub clocks: Vec<(String, String)>,
}

impl Gear {
    pub fn of(text: &str) -> Gear {
        let clocks = text
            .lines()
            .filter_map(|l| l.trim().strip_prefix("clock_")?.split_once('='))
            .map(|(n, v)| (n.to_string(), v.trim().to_string()))
            .collect();
        Gear {
            link: Link::of(text),
            clocks,
        }
    }

    /// The UFS host's clock, when devfreq has a node named for it; None is "not known yet".
    pub fn ufs_clock(&self) -> Option<&str> {
        self.clocks
            .iter()
            .find(|(n, _)| n.contains("ufs"))
            .map(|(_, v)| v.as_str())
    }

    pub fn line(&self) -> String {
        format!(
            "{}; clock {}",
            self.link.line(),
            self.ufs_clock().unwrap_or("not known yet")
        )
    }
}

/// The command that starts the bounded read in the background: O_DIRECT, to /dev/null, under `timeout`, its exit code in /tmp/load.done.
pub fn load_start_command(lun: &str, mb: u64, limit: u64) -> Result<String, String> {
    if !lun_is_a_disk(lun) {
        return Err(format!("lun {lun:?} is not a disk name like sda"));
    }
    Ok(format!(
        "rm -f /tmp/load.done /tmp/load.out; (timeout {limit} dd if=/dev/{lun} of=/dev/null bs=1M count={mb} iflag=direct 2>/tmp/load.out; echo rc=$? >/tmp/load.done) </dev/null >/dev/null 2>&1 & echo load-started-$((6*7))"
    ))
}

/// How long past the read's own limit a silent shell is waited for before the step stops (runs 1 and 2 stopped at 10 s).
pub const SILENCE_MARGIN_S: u64 = 60;

/// The command that reads only the load's state (no sysfs): it must answer even when a gear read is stuck, so it has its own shell.
pub fn load_state_command() -> &'static str {
    "cat /tmp/load.done 2>/dev/null; echo state-$((6*7))"
}

/// Run 1 (research walleye-ufs-try-2-stopped-at-step-16-gear-g3-under-load): eight gear reads answered under load, the ninth,
/// sent on the one shell together with the read of /tmp/load.done, was not answered in 10 s. A sysfs read of power_info goes
/// through the UFS host's runtime power management, so it can wait for the link as the load ends. Hence the read's state has
/// a shell of its own and no sysfs, a gear read has `gear_wait` seconds (device.toml), and one not answered is written down
/// as not known yet. What the stall was is read from gear-load.txt of the next run: each sample carries its time.
/// One gear sample, written into gear-load.txt: when it was asked, how long it took, and what it said or that it was not answered.
pub fn sample_line(at: Duration, took: Duration, said: Option<&str>) -> String {
    match said {
        Some(t) => format!(
            "--- sample at {:.1} s, answered in {:.1} s\n{t}\n",
            at.as_secs_f32(),
            took.as_secs_f32()
        ),
        None => format!(
            "--- sample at {:.1} s, not answered in {:.1} s (not known yet)\n",
            at.as_secs_f32(),
            took.as_secs_f32()
        ),
    }
}

/// The command that hashes the primary and the backup GPT of the disk, reading only.
pub fn gpt_command(lun: &str) -> Result<String, String> {
    if !lun_is_a_disk(lun) {
        return Err(format!("lun {lun:?} is not a disk name like sda"));
    }
    Ok(format!(
        "sz=$(cat /sys/block/{lun}/size); (dd if=/dev/{lun} bs=4096 count=6 iflag=direct 2>/dev/null; dd if=/dev/{lun} bs=4096 skip=$((sz/8-5)) count=5 iflag=direct 2>/dev/null) | sha256sum"
    ))
}

/// The sha256 of no input at all: a read that gave nothing.
pub const EMPTY_INPUT: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

/// The 64-hex digest a `sha256sum` answer holds.
pub fn digest_of(text: &str) -> Option<String> {
    text.split_whitespace()
        .find(|w| w.len() == 64 && w.chars().all(|c| c.is_ascii_hexdigit()))
        .map(str::to_lowercase)
}

/// The exit code of the read (`rc=N` in /tmp/load.done), None while it runs.
pub fn load_rc(sample: &str) -> Option<i32> {
    sample
        .lines()
        .find_map(|l| l.trim().strip_prefix("rc=")?.trim().parse().ok())
}

/// Does a command write to a disk? `of=` anything but /dev/null, a redirection to /dev, mount, mkfs, fstrim, a reset.
pub fn writes_to_the_card(cmd: &str) -> bool {
    let of = cmd
        .split_whitespace()
        .any(|w| w.starts_with("of=") && w != "of=/dev/null");
    let redirect = cmd.split('>').skip(1).any(|r| {
        let t = r.trim_start_matches('>').trim_start();
        t.starts_with("/dev/") && !t.starts_with("/dev/null")
    });
    of || redirect
        || ["mount", "mkfs", "fstrim", "reset_controller", "blkdiscard"]
            .iter()
            .any(|w| cmd.contains(w))
}

/// The phone's RAM log of the ramp: one line before each step and one after it, in a file under /tmp the lab reads at once
/// (decision 2026-11-23-walleye-ufs-silent-one-ramped-pinned-try: the log is written BEFORE the step, no channel is known to survive a reset).
pub const RAM_LOG: &str = "/tmp/ramp.log";

/// The seconds between two polls of the shell, and the pause between two steps; a test does not wait that long.
#[cfg(not(test))]
const POLL_MS: u64 = 1000;
#[cfg(test)]
const POLL_MS: u64 = 5;

/// A devfreq node name is the only thing a path is built from: lower-case hex, letters, dots, dashes, underscores.
pub fn is_a_node_name(n: &str) -> bool {
    !n.is_empty()
        && n.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '-' | '_'))
}

/// The command that reads the node's governor, the governors it has and its clock; a node that is not there says `no-governor-node`.
pub fn governor_read_command(node: &str) -> Result<String, String> {
    if !is_a_node_name(node) {
        return Err(format!("devfreq node {node:?} is not a node name"));
    }
    Ok(format!(
        "d=/sys/class/devfreq/{node}; if [ -e $d/governor ]; then echo governor=$(cat $d/governor); echo available=$(cat $d/available_governors); echo cur_freq=$(cat $d/cur_freq); else echo no-governor-node; fi"
    ))
}

/// The one sysfs write of the try: the governor of the UFS host's devfreq node set to powersave (a runtime setting of the phone's RAM kernel: nothing flashed, nothing written to the card).
pub fn governor_pin_command(node: &str) -> Result<String, String> {
    if !is_a_node_name(node) {
        return Err(format!("devfreq node {node:?} is not a node name"));
    }
    Ok(format!(
        "echo powersave > /sys/class/devfreq/{node}/governor; echo pinned-$((6*7))"
    ))
}

/// What the governor read said.
#[derive(Debug, Clone, PartialEq)]
pub struct Governor {
    pub present: bool,
    pub current: Option<String>,
    pub available: Vec<String>,
    pub cur_freq: Option<String>,
}

impl Governor {
    pub fn of(text: &str) -> Governor {
        let value = |k: &str| {
            text.lines()
                .find_map(|l| l.trim().strip_prefix(k)?.strip_prefix('='))
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        };
        Governor {
            present: !text.contains("no-governor-node") && value("governor").is_some(),
            current: value("governor"),
            available: value("available")
                .map(|v| v.split_whitespace().map(str::to_string).collect())
                .unwrap_or_default(),
            cur_freq: value("cur_freq"),
        }
    }

    pub fn can_powersave(&self) -> bool {
        self.present && self.available.iter().any(|g| g == "powersave")
    }

    pub fn line(&self) -> String {
        format!(
            "governor {}, clock {}",
            self.current.as_deref().unwrap_or("not known yet"),
            self.cur_freq.as_deref().unwrap_or("not known yet")
        )
    }
}

/// The command that starts ONE step of the ramp in the background: first one line in the RAM log (step, size, uptime), then the
/// bounded O_DIRECT read to /dev/null under `timeout`, its exit code in /tmp/load.done and a line after it in the RAM log.
pub fn ramp_step_command(
    lun: &str,
    step: usize,
    steps: usize,
    mb: u64,
    limit: u64,
) -> Result<String, String> {
    if !lun_is_a_disk(lun) {
        return Err(format!("lun {lun:?} is not a disk name like sda"));
    }
    let up = "$(cut -d' ' -f1 /proc/uptime)";
    Ok(format!(
        "rm -f /tmp/load.done /tmp/load.out; echo \"step {step}/{steps} start {mb} MiB uptime {up}\" >> {RAM_LOG}; (timeout {limit} dd if=/dev/{lun} of=/dev/null bs=1M count={mb} iflag=direct 2>/tmp/load.out; rc=$?; echo rc=$rc >/tmp/load.done; echo \"step {step}/{steps} end rc=$rc uptime {up}\" >> {RAM_LOG}) </dev/null >/dev/null 2>&1 & echo load-started-$((6*7))"
    ))
}

/// The command that prints the RAM log.
pub fn ram_log_command() -> String {
    format!("cat {RAM_LOG}; echo log-read-$((6*7))")
}

/// How one step of the ramp went on the host's side.
#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    /// The read ended with `rc`; `silent` state reads were not answered on the way (the first at `first_silent` seconds), `inside` gear reads answered.
    Ended {
        rc: i32,
        inside: u32,
        silent: u32,
        first_silent: Option<f32>,
    },
    /// The shell never came back within the step's limit and the margin.
    NoEnd { silent: u32, first_silent: f32 },
}

/// The result file of the ramp, kept as a result: how far it got and the first silent step with its gear and clock.
pub fn ramp_result(done: &[(u64, Step)], steps: &[u64], gear: &str) -> String {
    let mut t = String::new();
    for (n, (mb, how)) in done.iter().enumerate() {
        match how {
            Step::Ended { rc, silent: 0, .. } => t.push_str(&format!(
                "step {}/{} {mb} MiB: ended rc={rc}, the shell answered every poll\n",
                n + 1,
                steps.len()
            )),
            Step::Ended { rc, silent, first_silent, .. } => t.push_str(&format!(
                "step {}/{} {mb} MiB: FIRST SILENT STEP: {silent} polls not answered from {:.1} s, then the read ended rc={rc}; last gear read: {gear}\n",
                n + 1,
                steps.len(),
                first_silent.unwrap_or(0.0)
            )),
            Step::NoEnd { silent, first_silent } => t.push_str(&format!(
                "step {}/{} {mb} MiB: FIRST SILENT STEP: {silent} polls not answered from {first_silent:.1} s and the read did not end; last gear read: {gear}\n",
                n + 1,
                steps.len()
            )),
        }
    }
    if done.len() == steps.len()
        && done
            .iter()
            .all(|(_, h)| matches!(h, Step::Ended { silent: 0, .. }))
    {
        t.push_str("no silent step: the shell answered every poll through the whole ramp\n");
    } else if let Some(last) = steps.get(done.len()..).filter(|r| !r.is_empty()) {
        t.push_str(&format!(
            "steps not started after the silence: {last:?} MiB\n"
        ));
    }
    t
}

impl Ufs {
    fn read_gear(&self, name: &str) -> Result<Gear, String> {
        let mut sh = self.lab.shell(10)?;
        let said = sh.run(&gear_command(), Duration::from_secs(self.s.gear_wait))?;
        let gear = Gear::of(&said);
        self.lab.save(name, &said)?;
        println!("  {name}: {}", gear.line());
        if gear.link.speed() == Speed::Down {
            return Err(format!("the link is down: {}", gear.line()));
        }
        Ok(gear)
    }

    /// The gear and the clock, idle, before any load.
    pub fn the_gear_is_read_before_load(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || u.read_gear("gear-before.txt").map(|_| ())
    }

    /// The governor of the UFS host's devfreq node is read and set to powersave, BEFORE any load: no node, or no powersave among its
    /// governors, stops the run here; what was read before and after is saved, and the clock it holds is said.
    pub fn the_governor_is_pinned_to_powersave(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let node = &u.s.devfreq;
            let mut sh = u.lab.shell(10)?;
            let wait = Duration::from_secs(u.s.gear_wait);
            let before = Governor::of(&sh.run(&governor_read_command(node)?, wait)?);
            u.lab.save("governor-before.txt", &before.line())?;
            if !before.can_powersave() {
                return Err(format!(
                    "devfreq {node} has no powersave to pin ({}; governors: {}): the run stops before any load",
                    if before.present { "node there" } else { "no governor node" },
                    before.available.join(" ")
                ));
            }
            sh.run(&governor_pin_command(node)?, wait)?;
            let after = Governor::of(&sh.run(&governor_read_command(node)?, wait)?);
            u.lab.save("governor-after.txt", &after.line())?;
            if after.current.as_deref() != Some("powersave") {
                return Err(format!(
                    "the governor did not take powersave: {}",
                    after.line()
                ));
            }
            println!("  {node}: {} -> {}", before.line(), after.line());
            Ok(())
        }
    }

    /// The ramped, bounded, read-only read of the disk (decision 2026-11-23-walleye-ufs-silent-one-ramped-pinned-try): each step of
    /// `ramp_mb` is one O_DIRECT read to /dev/null with a line in the phone's RAM log before it, the shell polled every second and the gear
    /// sampled in between. The first silent step is kept as a RESULT (ramp-result.txt: its size, the second it began, the last gear and
    /// clock), not an error: the ramp stops there and the way back runs. A step that ends with an error code, or a phone that hung up, is
    /// an error. At least one gear read must have been answered in the whole ramp.
    pub fn the_gear_is_read_under_load(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let mut state_sh = u.lab.shell(10)?;
            let (mut samples, mut done, mut inside_all) = (String::new(), Vec::new(), 0);
            let mut last_gear = String::from("not known yet");
            let ramped = u.ramp(
                &mut state_sh,
                &mut samples,
                &mut done,
                &mut inside_all,
                &mut last_gear,
            );
            u.lab.save("gear-load.txt", &redact(&samples))?;
            let result = ramp_result(&done, &u.s.ramp_mb, &last_gear);
            u.lab.save("ramp-result.txt", &result)?;
            print!("{result}");
            u.save_the_ram_log();
            ramped?;
            if inside_all == 0
                && !done
                    .iter()
                    .any(|(_, h)| matches!(h, Step::NoEnd { .. } | Step::Ended { silent: 1.., .. }))
            {
                return Err(
                    "the ramp ended before the gear could be read under load: not known yet".into(),
                );
            }
            Ok(())
        }
    }

    /// The RAM log, read at once on a fresh shell; a phone that does not answer is said so (not known yet), never an error.
    fn save_the_ram_log(&self) {
        let said = self
            .lab
            .shell(10)
            .and_then(|mut sh| sh.run(&ram_log_command(), Duration::from_secs(10)));
        let text = match said {
            Ok(t) => t,
            Err(e) => {
                format!("not known yet: the phone did not answer the read of {RAM_LOG}: {e}\n")
            }
        };
        println!(
            "  RAM log: {}",
            text.lines()
                .rev()
                .find(|l| !l.starts_with("log-read-"))
                .unwrap_or("empty")
        );
        let _ = self.lab.save("ramp-log.txt", &text);
    }

    fn ramp(
        &self,
        state_sh: &mut Shell,
        samples: &mut String,
        done: &mut Vec<(u64, Step)>,
        inside_all: &mut u32,
        last_gear: &mut String,
    ) -> Done {
        let steps = &self.s.ramp_mb;
        for (i, &mb) in steps.iter().enumerate() {
            let start = ramp_step_command(&self.s.lun, i + 1, steps.len(), mb, self.s.load_limit)?;
            samples.push_str(&format!("=== step {}/{} {mb} MiB\n", i + 1, steps.len()));
            let began = Instant::now();
            match state_sh.run(&start, Duration::from_secs(10)) {
                Ok(t) if t.contains("load-started-42") => {}
                Ok(_) => return Err(format!("step {} ({mb} MiB) did not start", i + 1)),
                Err(e) => {
                    // a phone that does not answer the start is this step's silence: kept as the result
                    samples.push_str(&format!(
                        "--- the start of the step was not answered: {e}\n"
                    ));
                    done.push((
                        mb,
                        Step::NoEnd {
                            silent: 1,
                            first_silent: 0.0,
                        },
                    ));
                    return Ok(());
                }
            }
            let how = self.sample_until_the_read_ends(state_sh, began, samples, last_gear);
            let (rc, inside, silent) = match &how {
                Step::Ended {
                    rc, inside, silent, ..
                } => (Some(*rc), *inside, *silent),
                Step::NoEnd { silent, .. } => (None, 0, *silent),
            };
            *inside_all += inside;
            done.push((mb, how));
            match rc {
                Some(0) | Some(124) => {}
                Some(other) => {
                    let out = state_sh
                        .run("cat /tmp/load.out", Duration::from_secs(10))
                        .unwrap_or_default();
                    return Err(format!(
                        "step {} ({mb} MiB) ended with {other}: {out}",
                        i + 1
                    ));
                }
                None => return Ok(()), // the shell never came back: the result is kept, the way back runs
            }
            if silent > 0 {
                return Ok(()); // the first silent step is the result: no bigger step after it
            }
            std::thread::sleep(Duration::from_millis(POLL_MS * 2));
        }
        Ok(())
    }

    /// Ask the step's state on its own shell every second until it ends (at most its limit and SILENCE_MARGIN_S, checked between polls; a
    /// gear read in flight may run on to `gear_wait` after that), read the gear in between. A poll not answered is a reading kept with its
    /// time; the shell is then asked on a fresh connection.
    fn sample_until_the_read_ends(
        &self,
        state_sh: &mut Shell,
        began: Instant,
        samples: &mut String,
        last_gear: &mut String,
    ) -> Step {
        let until = began + Duration::from_secs(self.s.load_limit + SILENCE_MARGIN_S);
        let state_wait = Duration::from_secs(self.s.gear_wait.min(10));
        let Ok(mut gear_sh) = self.lab.shell(10) else {
            samples.push_str("--- no shell for the gear reads: the phone does not answer\n");
            return Step::NoEnd {
                silent: 1,
                first_silent: 0.0,
            };
        };
        let (mut inside, mut unanswered, mut silent, mut hung_up) = (0, 0, 0u32, 0);
        let mut first_silent: Option<f32> = None;
        let rc = loop {
            if Instant::now() >= until {
                samples.push_str(&format!(
                    "--- no end of the read within its limit and {SILENCE_MARGIN_S} s ({silent} silent state reads)\n"
                ));
                return Step::NoEnd {
                    silent,
                    first_silent: first_silent.unwrap_or(0.0),
                };
            }
            std::thread::sleep(Duration::from_millis(POLL_MS));
            let (at, asked) = (began.elapsed(), Instant::now());
            match state_sh.run(load_state_command(), state_wait) {
                Ok(said) => {
                    hung_up = 0;
                    if let Some(rc) = load_rc(&said) {
                        break rc;
                    }
                }
                Err(e) => {
                    // runs 1 to 3: the shell went silent while the read ran. That is a reading, kept with its time; the shell is
                    // asked again on a fresh connection until the step's own limit and the margin are over.
                    silent += 1;
                    first_silent.get_or_insert(at.as_secs_f32());
                    if asked.elapsed() < Duration::from_secs(1) {
                        hung_up += 1; // an answer at once is the phone hanging up, not a silence
                        if hung_up >= 5 {
                            samples.push_str(
                                "--- the shell hung up five times at once: the phone is gone\n",
                            );
                            return Step::NoEnd {
                                silent,
                                first_silent: first_silent.unwrap_or(0.0),
                            };
                        }
                    }
                    println!("  under load: the shell is silent ({e}); waiting for it");
                    samples.push_str(&format!(
                        "--- state read at {:.1} s, not answered in {:.1} s (the shell is silent under load)\n",
                        at.as_secs_f32(),
                        asked.elapsed().as_secs_f32()
                    ));
                    if let Ok(fresh) = self.lab.shell(10) {
                        *state_sh = fresh;
                    }
                    continue; // no gear read on a silent shell: it would only wait too
                }
            }
            match gear_sh.run(&gear_command(), Duration::from_secs(self.s.gear_wait)) {
                Ok(said) => {
                    inside += 1;
                    let gear = Gear::of(&said);
                    println!("  under load: {}", gear.line());
                    *last_gear = format!("{} at {:.1} s", gear.line(), at.as_secs_f32());
                    samples.push_str(&sample_line(at, asked.elapsed(), Some(&said)));
                    if asked.elapsed() >= Duration::from_secs(3) {
                        silent += 1; // a gear read that held the poll for 3 s or more is a silence of the shell
                        first_silent.get_or_insert(at.as_secs_f32());
                    }
                }
                Err(e) => {
                    unanswered += 1;
                    println!(
                        "  under load: a gear read was not answered in {} s (not known yet): {e}",
                        self.s.gear_wait
                    );
                    samples.push_str(&sample_line(at, asked.elapsed(), None));
                    silent += 1; // a stalled gear read is silence too (run 1)
                    first_silent.get_or_insert(at.as_secs_f32());
                    match self.lab.shell(10) {
                        Ok(fresh) => gear_sh = fresh,
                        Err(_) => {
                            return Step::NoEnd {
                                silent,
                                first_silent: first_silent.unwrap_or(0.0),
                            }
                        }
                    }
                }
            }
        };
        samples.push_str(&format!(
            "--- the read ended at {:.1} s with rc={rc}; {inside} gear reads answered, {unanswered} not, {silent} state reads silent\n",
            began.elapsed().as_secs_f32()
        ));
        Step::Ended {
            rc,
            inside,
            silent,
            first_silent,
        }
    }

    /// On the way back, once Android answers: why the phone booted, read only by adb (the boot reason properties and the kernel log's
    /// reset lines when adb may read them). A watchdog bite or a warm reset shows here or it is "not known yet": nothing is judged or written.
    pub fn the_pon_reason_is_read(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let android = crate::android::Android::new(u.s.clone(), u.lab.clone());
            let mut text = String::new();
            for prop in [
                "ro.boot.bootreason",
                "sys.boot.reason",
                "sys.boot.reason.last",
            ] {
                let said = android
                    .adb(&["shell", "getprop", prop])
                    .map(|t| t.trim().to_string())
                    .ok()
                    .filter(|t| !t.is_empty())
                    .unwrap_or_else(|| "not known yet".into());
                text.push_str(&format!("{prop}={said}\n"));
            }
            let kernel = android
                .adb(&["shell", "dmesg 2>/dev/null | grep -iwE 'pon|poff|qpnp-pon|reset reason|warm reset|watchdog' | head -20"])
                .ok()
                .filter(|t| !t.trim().is_empty())
                .unwrap_or_else(|| "kernel log lines: not known yet (adb may not read dmesg)".into());
            text.push_str(&kernel);
            println!("  PON reason: {}", text.lines().next().unwrap_or(""));
            u.lab.save("pon-reason.txt", &redact(&text))
        }
    }

    /// Did the ramp end on a silent step (ramp-result.txt)? Then the phone is not known to answer: the steps after the load say "not known yet"
    /// and let the run reach the hold release and the way back, where the PON reason is read.
    fn the_ramp_was_silent(&self) -> bool {
        std::fs::read_to_string(self.s.out.join("ramp-result.txt"))
            .is_ok_and(|t| t.contains("FIRST SILENT STEP"))
    }

    /// The gear and the clock again, 2 s after the read ended.
    pub fn the_gear_is_read_two_seconds_after(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            if u.the_ramp_was_silent() {
                println!("  the ramp ended silent: the gear after the read is not known yet");
                return u.lab.save(
                    "gear-after.txt",
                    "not known yet: the ramp ended on a silent step\n",
                );
            }
            std::thread::sleep(Duration::from_secs(2));
            u.read_gear("gear-after.txt").map(|_| ())
        }
    }

    fn gpt_read(&self, name: &str) -> Done {
        let mut sh = self.lab.shell(10)?;
        let said = sh.run(&gpt_command(&self.s.lun)?, Duration::from_secs(20))?;
        let digest = digest_of(&said).ok_or_else(|| format!("no digest in: {said}"))?;
        if digest == EMPTY_INPUT {
            return Err("the GPT read hashed nothing: the disk could not be read".into());
        }
        println!("  {name}: {digest}");
        self.lab.save(name, &digest)
    }

    /// The GPT of the disk is hashed, before the read.
    pub fn the_gpt_is_hashed_before(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || u.gpt_read("gpt-before.txt")
    }

    /// The GPT is hashed again and equals the first digest.
    pub fn the_gpt_is_unchanged(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            if u.the_ramp_was_silent() {
                println!("  the ramp ended silent: the GPT after the read is not known yet");
                return u.lab.save(
                    "gpt-after.txt",
                    "not known yet: the ramp ended on a silent step\n",
                );
            }
            u.gpt_read("gpt-after.txt")?;
            let read =
                |n: &str| std::fs::read_to_string(u.s.out.join(n)).map_err(|e| format!("{n}: {e}"));
            let (a, b) = (read("gpt-before.txt")?, read("gpt-after.txt")?);
            if a.trim() == b.trim() {
                Ok(())
            } else {
                Err(format!("the GPT changed: {} then {}", a.trim(), b.trim()))
            }
        }
    }

    /// The card has its disks, each with a name, and the first one's partitions: identity and tables as in try 1.
    pub fn the_card_is_identified(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let mut sh = u.lab.shell(10)?;
            let said = sh.run(
                "for d in /sys/class/scsi_device/*/device; do echo \"$(cat $d/vendor) $(cat $d/model) $(cat $d/rev)\"; done; cat /proc/partitions",
                Duration::from_secs(20),
            )?;
            u.lab.save("card.txt", &redact(&said))?;
            let luns = super::luns_of(&said);
            let parts = said
                .lines()
                .filter(|l| {
                    l.split_whitespace()
                        .nth(3)
                        .is_some_and(|n| n.len() > u.s.lun.len() && n.starts_with(u.s.lun.as_str()))
                })
                .count();
            if luns.len() as u64 != u.s.luns {
                return Err(format!(
                    "{} disks listed, device.toml says {}: {}",
                    luns.len(),
                    u.s.luns,
                    luns.join(" ")
                ));
            }
            if parts == 0 {
                return Err(format!("{} has no partition listed", u.s.lun));
            }
            println!("  {} disks, {} has {parts} partitions", luns.len(), u.s.lun);
            Ok(())
        }
    }

    fn slots(&self, name: &str) -> Done {
        let flags = crate::host::slot_flags(&self.s.fastboot, &self.lab.serial()?)?;
        self.lab.save(name, &flags)
    }

    /// The slots' flags, read in fastboot before the boot.
    pub fn the_slot_flags_are_read(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || u.slots("slots-before.txt")
    }

    /// Back on Android: the phone goes to fastboot once more, the flags are read and equal the first, and it goes back to Android.
    pub fn the_slot_flags_are_as_before(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            u.lab.phone_is_in_fastboot()?;
            let read = u.slots("slots-after.txt");
            let back = u.lab.way_back();
            read?;
            back?;
            let get =
                |n: &str| std::fs::read_to_string(u.s.out.join(n)).map_err(|e| format!("{n}: {e}"));
            let (a, b) = (get("slots-before.txt")?, get("slots-after.txt")?);
            if a == b {
                Ok(())
            } else {
                Err(format!("the slot flags changed:\n{a}\nthen\n{b}"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phone::fake;
    use crate::settings::{Settings, Try};

    const G3: &str = "gear=HS_GEAR3\nlane=2\nmode=FAST_MODE\nrate=HS_RATE_B\nlink_state=ACTIVE\nclock_1d84000.ufshc=300000000";
    const G1: &str = "gear=HS_GEAR1\nlane=2\nmode=FAST_MODE\nrate=HS_RATE_B\nlink_state=HIBERN8";
    const HEX: &str = "461077869fb275ab8803b8bb771050d4f19724ccfd62e197dc1f3fe3a6297df5";

    fn out(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("wt-load-{name}-{}", std::process::id()))
    }

    fn ufs(table: Vec<(&'static str, &'static str)>, name: &str) -> Ufs {
        let mut s = Settings::stand_in(Try::Ufs);
        s.port = fake::phone(table, vec![]);
        s.out = out(name);
        let _ = std::fs::remove_dir_all(&s.out);
        s.device_wait = 2;
        Ufs::new(s)
    }

    #[test]
    fn only_a_disk_name_is_built_into_a_command() {
        assert!(lun_is_a_disk("sda") && lun_is_a_disk("sdf"));
        for bad in ["", "sd", "sda1", "sdA", "sda; rm", "mmcblk0", "/dev/sda"] {
            assert!(!lun_is_a_disk(bad), "{bad}");
            assert!(load_start_command(bad, 1, 1).is_err() && gpt_command(bad).is_err());
        }
    }

    #[test]
    fn the_gear_and_the_clock_are_read_and_a_missing_clock_is_not_known_yet() {
        let g = Gear::of(G3);
        assert_eq!(g.link.speed(), Speed::Full);
        assert_eq!(g.ufs_clock(), Some("300000000"));
        assert!(g.line().contains("clock 300000000"));
        assert!(Gear::of(G1).line().ends_with("clock not known yet"));
    }

    #[test]
    fn the_commands_are_shell_read_only_and_bounded() {
        let cmds = [
            gear_command(),
            load_start_command("sda", 4096, 60).unwrap(),
            load_state_command().to_string(),
            gpt_command("sda").unwrap(),
        ];
        for c in &cmds {
            let ok = std::process::Command::new("sh")
                .args(["-n", "-c", c])
                .status()
                .unwrap();
            assert!(ok.success(), "{c}");
            assert!(!writes_to_the_card(c), "{c}");
        }
        let start = &cmds[1];
        assert!(
            start.contains("iflag=direct")
                && start.contains("timeout 60")
                && start.contains("count=4096")
        );
        for bad in [
            "dd if=/dev/zero of=/dev/sda",
            "echo x > /dev/sda",
            "mount /dev/sda1 /m",
            "fstrim /",
            "mkfs.ext4 x",
        ] {
            assert!(writes_to_the_card(bad), "{bad}");
        }
    }

    #[test]
    fn digests_and_the_reads_exit_code_are_read() {
        assert_eq!(digest_of(&format!("{HEX}  -")).as_deref(), Some(HEX));
        assert_eq!(digest_of("sha256sum: not found"), None);
        assert_eq!(load_rc("rc=0\ngear=3"), Some(0));
        assert_eq!(load_rc("gear=3"), None);
    }

    #[test]
    fn slot_flags_are_kept_sorted_and_without_the_serial() {
        let a = "(bootloader) slot-successful:b:yes\n(bootloader) current-slot:b\n(bootloader) serial:XYZ\n(bootloader) slot-successful:a:no\n";
        let b = "(bootloader) current-slot:b\n(bootloader) slot-successful:a:no\n(bootloader) slot-successful:b:yes\n";
        assert_eq!(crate::host::slot_lines(a), crate::host::slot_lines(b));
        assert!(!crate::host::slot_lines(a).contains("XYZ"));
    }

    /// What the phone printed under load in run 1 (research walleye-ufs-try-2-stopped-at-step-16-gear-g3-under-load).
    const RUN1_G3: &str = "dev_pm=ACTIVE\ngear=HS_GEAR3\nlane=2\nlink_state=ACTIVE\nmode=FAST_MODE\nrate=HS_RATE_B\nclock_1da4000.ufshc=200000000";
    const NOT_DONE: &str = "state-42";
    const DONE0: &str = "rc=0\nstate-42";

    #[test]
    fn a_read_that_is_sampled_under_load_passes() {
        let t = vec![
            ("load-started-", "load-started-42"),
            (
                "echo state-",
                Box::leak(format!("{NOT_DONE}|||{NOT_DONE}|||{DONE0}").into_boxed_str()),
            ),
            ("power_info", RUN1_G3),
            (
                "cat /tmp/load.out",
                "4096+0 records out\n4294967296 bytes (4.0GB) copied, 12.1 seconds, 338.5MB/s",
            ),
        ];
        let u = ufs(t, "load");
        u.the_gear_is_read_under_load()().unwrap();
        let saved = std::fs::read_to_string(out("load").join("gear-load.txt")).unwrap();
        assert!(
            out("load").join("ramp-result.txt").exists()
                && out("load").join("ramp-log.txt").exists()
        );
        assert!(
            saved.contains("answered in") && saved.contains("2 gear reads answered, 0 not"),
            "{saved}"
        );
    }

    #[test]
    fn a_gear_read_that_is_not_answered_is_written_down_and_the_read_still_ends_the_step() {
        let t = vec![
            ("load-started-", "load-started-42"),
            (
                "echo state-",
                Box::leak(
                    format!("{NOT_DONE}|||{NOT_DONE}|||{NOT_DONE}|||{DONE0}").into_boxed_str(),
                ),
            ),
            ("power_info", "?"),
            ("cat /tmp/load.out", "x"),
        ];
        let u = ufs(t, "silent");
        u.the_gear_is_read_under_load()().unwrap(); // a stalled gear read is a recorded silence, not an error
        let r = std::fs::read_to_string(out("silent").join("ramp-result.txt")).unwrap();
        assert!(r.contains("FIRST SILENT STEP"), "{r}");
        let saved = std::fs::read_to_string(out("silent").join("gear-load.txt")).unwrap();
        assert!(
            saved.contains("not answered") && saved.contains("0 gear reads answered, 3 not"),
            "{saved}"
        );
    }

    #[test]
    fn a_hung_gear_read_is_dropped_and_sampling_goes_on_a_fresh_shell() {
        let t = vec![
            ("load-started-", "load-started-42"),
            (
                "echo state-",
                Box::leak(format!("{NOT_DONE}|||{NOT_DONE}|||{DONE0}").into_boxed_str()),
            ),
            (
                "power_info",
                Box::leak(format!("?once={RUN1_G3}").into_boxed_str()),
            ),
            ("cat /tmp/load.out", "x"),
        ];
        ufs(t, "hung").the_gear_is_read_under_load()().unwrap();
        let saved = std::fs::read_to_string(out("hung").join("gear-load.txt")).unwrap();
        assert!(
            saved.contains("not answered in") && saved.contains("1 gear reads answered, 1 not"),
            "{saved}"
        );
    }

    #[test]
    fn a_silent_shell_during_the_read_is_a_reading_and_the_wait_goes_on_a_fresh_shell() {
        // run 2 of try 2: after eight answered gear reads the shell stopped answering `cat /tmp/load.done` (step 16 failed at 10 s)
        let t = vec![
            ("load-started-", "load-started-42"),
            (
                "echo state-",
                Box::leak(format!("?once={NOT_DONE}|||{NOT_DONE}|||{DONE0}").into_boxed_str()),
            ),
            ("power_info", RUN1_G3),
            ("cat /tmp/load.out", "x"),
        ];
        let u = ufs(t, "silent-state");
        u.the_gear_is_read_under_load()().unwrap();
        let saved = std::fs::read_to_string(out("silent-state").join("gear-load.txt")).unwrap();
        assert!(
            saved.contains("the shell is silent under load")
                && saved.contains("1 state reads silent"),
            "{saved}"
        );
    }

    #[test]
    fn a_read_that_ended_before_any_sample_is_not_known_yet() {
        let t = vec![
            ("load-started-", "load-started-42"),
            ("echo state-", DONE0),
            ("cat /tmp/load.out", "x"),
        ];
        let e = ufs(t, "fast").the_gear_is_read_under_load()().unwrap_err();
        assert!(e.contains("not known yet"), "{e}");
    }

    #[test]
    fn a_read_with_an_error_exit_is_a_failure_and_a_timeout_is_bounded_not_failed() {
        let run = |rc: &'static str, name: &str| {
            let t = vec![
                ("load-started-", "load-started-42"),
                (
                    "echo state-",
                    Box::leak(format!("{NOT_DONE}|||rc={rc}\nstate-42").into_boxed_str()),
                ),
                ("power_info", RUN1_G3),
                ("cat /tmp/load.out", "dd: I/O error"),
            ];
            ufs(t, name).the_gear_is_read_under_load()()
        };
        assert!(run("124", "to").is_ok());
        assert!(run("1", "err").unwrap_err().contains("I/O error"));
    }

    #[test]
    fn the_whole_after_load_part_is_reachable_in_the_stand_in() {
        // checks 16-19 of run 1: the gear 2 s after, the GPT hashed again and equal
        let t = vec![
            ("power_info", RUN1_G3),
            (
                "sha256sum",
                Box::leak(format!("{HEX}  -").into_boxed_str()) as &'static str,
            ),
        ];
        let u = ufs(t, "after");
        u.the_gpt_is_hashed_before()().unwrap();
        u.the_gear_is_read_two_seconds_after()().unwrap();
        u.the_gpt_is_unchanged()().unwrap();
        assert!(out("after").join("gear-after.txt").exists());
    }

    #[test]
    fn a_link_that_is_down_under_the_read_fails_the_gear_step() {
        let t = vec![(
            "power_info",
            "gear=HS_GEAR1\nlane=2\nmode=FAST_MODE\nlink_state=OFF",
        )];
        let e = ufs(t, "down").the_gear_is_read_before_load()().unwrap_err();
        assert!(e.contains("link is down"), "{e}");
    }

    #[test]
    fn the_gpt_before_and_after_are_compared() {
        let same = vec![(
            "sha256sum",
            Box::leak(format!("{HEX}  -").into_boxed_str()) as &'static str,
        )];
        let u = ufs(same, "gpt");
        u.the_gpt_is_hashed_before()().unwrap();
        u.the_gpt_is_unchanged()().unwrap();
        let u = ufs(
            vec![(
                "sha256sum",
                "0000000000000000000000000000000000000000000000000000000000000001  -",
            )],
            "gptdiff",
        );
        std::fs::create_dir_all(&u.s.out).unwrap();
        std::fs::write(u.s.out.join("gpt-before.txt"), format!("{HEX}\n")).unwrap();
        assert!(u.the_gpt_is_unchanged()()
            .unwrap_err()
            .contains("GPT changed"));
    }

    #[test]
    fn the_card_needs_all_its_disks_and_the_first_ones_partitions() {
        let six = "x\n 8 0 1000 sda\n 8 1 100 sda1\n 8 16 1 sdb\n 8 32 1 sdc\n 8 48 1 sdd\n 8 64 1 sde\n 8 80 1 sdf";
        assert!(ufs(vec![("scsi_device", six)], "six").the_card_is_identified()().is_ok());
        let e = ufs(vec![("scsi_device", " 8 0 1 sda\n 8 1 1 sda1")], "one")
            .the_card_is_identified()()
        .unwrap_err();
        assert!(e.contains("disks listed"), "{e}");
        let e = ufs(
            vec![(
                "scsi_device",
                " 8 0 1 sda\n 8 16 1 sdb\n 8 32 1 sdc\n 8 48 1 sdd\n 8 64 1 sde\n 8 80 1 sdf",
            )],
            "nopart",
        )
        .the_card_is_identified()()
        .unwrap_err();
        assert!(e.contains("no partition"), "{e}");
    }
}

#[cfg(test)]
mod more_tests {
    use super::*;
    use crate::settings::{Settings, Try};

    #[test]
    fn the_told_text_carries_the_settings_numbers() {
        let s = Settings::stand_in(Try::Ufs);
        let t = crate::ufs::expect(&s);
        assert!(
            t.contains("6 disks")
                && t.contains("sda in steps of 1, 2, 4 MiB (at most 20 seconds each)")
                && t.contains("governor to powersave"),
            "{t}"
        );
    }

    #[test]
    fn a_gpt_read_that_hashed_nothing_is_no_pass() {
        let mut s = Settings::stand_in(Try::Ufs);
        s.port = crate::phone::fake::phone(
            vec![(
                "sha256sum",
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  -",
            )],
            vec![],
        );
        s.out = std::env::temp_dir().join(format!("wt-empty-{}", std::process::id()));
        let e = Ufs::new(s).the_gpt_is_hashed_before()().unwrap_err();
        assert!(e.contains("hashed nothing"), "{e}");
    }

    #[test]
    #[should_panic(expected = "not a disk name")]
    fn a_lun_that_is_no_disk_stops_the_dry_run() {
        crate::settings::Settings::stand_in_with(Try::Ufs, &[("lun", "sda1")]);
    }

    #[test]
    fn slot_flags_of_a_fastboot_without_slot_lines_is_an_error() {
        let d = std::env::temp_dir().join(format!("wt-fb-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let fb = d.join("fastboot");
        std::fs::write(&fb, "#!/bin/sh\necho '(bootloader) product:walleye' >&2\n").unwrap();
        std::process::Command::new("chmod")
            .arg("+x")
            .arg(&fb)
            .status()
            .unwrap();
        let e = crate::host::slot_flags(fb.to_str().unwrap(), "SER").unwrap_err();
        assert!(e.contains("no slot flags"), "{e}");
        std::fs::write(&fb, "#!/bin/sh\necho '(bootloader) current-slot:b' >&2\necho '(bootloader) slot-successful:b:yes' >&2\n").unwrap();
        assert!(crate::host::slot_flags(fb.to_str().unwrap(), "SER")
            .unwrap()
            .contains("current-slot:b"));
    }
}

#[cfg(test)]
mod ramp_tests {
    use super::*;
    use crate::phone::fake;
    use crate::settings::{Settings, Try};

    /// What the phone printed under load in run 3 of try 2 (~/lab-material/walleye/ufs-20261123-105319/gear-load.txt, saved as fixtures)
    const RUN3_LOAD: &str = include_str!("../../fixtures/ufs-try2-run3-gear-load.txt");
    const RUN3_BEFORE: &str = include_str!("../../fixtures/ufs-try2-run3-gear-before.txt");
    const NOT_DONE: &str = "state-42";
    const DONE0: &str = "rc=0\nstate-42";
    /// ASSUMED text, not phone-printed: no governor read has come from the phone yet (the first run prints the real one); the clocks are run 3's idle 50 MHz and load 200 MHz at best
    /// ASSUMED text, not phone-printed: no governor read has come from the phone yet (the first run prints the real one)
    const POWERSAVE: &str = "governor=simple_ondemand\navailable=powersave performance simple_ondemand\ncur_freq=300000000";
    const PINNED: &str =
        "governor=powersave\navailable=powersave performance simple_ondemand\ncur_freq=50000000";

    fn ufs(table: Vec<(&'static str, &'static str)>, name: &str) -> Ufs {
        let mut s = Settings::stand_in(Try::Ufs);
        s.port = fake::phone(table, vec![]);
        s.out = std::env::temp_dir().join(format!("wt-ramp-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&s.out);
        s.device_wait = 2;
        s.load_limit = 1;
        Ufs::new(s)
    }

    fn saved(u: &Ufs, name: &str) -> String {
        std::fs::read_to_string(u.s.out.join(name)).unwrap_or_default()
    }

    /// The blocks of run 3's gear-load.txt that carry a gear read, as the host's reading of them.
    fn run3_samples() -> Vec<Gear> {
        RUN3_LOAD
            .split("--- ")
            .filter(|b| b.starts_with("sample at"))
            .map(|b| Gear::of(b.split_once('\n').unwrap().1))
            .collect()
    }

    #[test]
    fn the_text_run_3_printed_is_read_as_gear_3_at_200_mhz_then_silence() {
        let g = run3_samples();
        assert_eq!(g.len(), 7);
        assert!(g
            .iter()
            .all(|x| x.link.speed() == Speed::Full && x.ufs_clock() == Some("200000000")));
        let silent = RUN3_LOAD.matches("not answered in 10.1 s").count();
        assert_eq!(silent, 6); // run 3: six state reads unanswered over 110 s
        assert_eq!(Gear::of(RUN3_BEFORE).ufs_clock(), Some("50000000"));
        assert_ne!(Gear::of(RUN3_BEFORE).link.speed(), Speed::Full);
    }

    #[test]
    fn the_first_silent_step_is_a_result_kept_with_its_gear_and_clock() {
        let steps = [64, 256, 1024];
        let silent = Step::NoEnd {
            silent: 4,
            first_silent: 4.6,
        };
        let r = ramp_result(
            &[
                (
                    64,
                    Step::Ended {
                        rc: 0,
                        inside: 1,
                        silent: 0,
                        first_silent: None,
                    },
                ),
                (256, silent),
            ],
            &steps,
            "gear 3 200 MHz at 4.0 s",
        );
        assert!(
            r.contains("step 2/3 256 MiB: FIRST SILENT STEP: 4 polls not answered from 4.6 s"),
            "{r}"
        );
        assert!(
            r.contains("last gear read: gear 3 200 MHz at 4.0 s")
                && r.contains("steps not started after the silence: [1024] MiB"),
            "{r}"
        );
        let all = ramp_result(
            &steps.map(|m| {
                (
                    m,
                    Step::Ended {
                        rc: 0,
                        inside: 1,
                        silent: 0,
                        first_silent: None,
                    },
                )
            }),
            &steps,
            "x",
        );
        assert!(all.contains("no silent step"), "{all}");
    }

    #[test]
    fn the_governor_is_read_and_a_missing_node_or_powersave_is_said() {
        let g = Governor::of(POWERSAVE);
        assert!(
            g.can_powersave()
                && g.current.as_deref() == Some("simple_ondemand")
                && g.cur_freq.as_deref() == Some("300000000")
        );
        assert!(!Governor::of("no-governor-node").can_powersave());
        assert!(!Governor::of("governor=a\navailable=performance\ncur_freq=1").can_powersave());
        assert!(Governor::of("").line().contains("not known yet"));
    }

    #[test]
    fn the_commands_of_the_ramp_are_shell_and_read_only() {
        let cmds = [
            governor_read_command("1da4000.ufshc").unwrap(),
            governor_pin_command("1da4000.ufshc").unwrap(),
            ramp_step_command("sda", 2, 3, 256, 60).unwrap(),
            ram_log_command(),
        ];
        for c in &cmds {
            assert!(
                std::process::Command::new("sh")
                    .args(["-n", "-c", c])
                    .status()
                    .unwrap()
                    .success(),
                "{c}"
            );
            assert!(!writes_to_the_card(c), "{c}");
        }
        let step = &cmds[2];
        assert!(
            step.contains("iflag=direct")
                && step.contains("count=256")
                && step.contains("timeout 60")
        );
        assert!(
            step.find(RAM_LOG).unwrap() < step.find("dd if=").unwrap(),
            "the RAM log line is written BEFORE the read"
        );
        assert!(cmds[1].contains("powersave > /sys/class/devfreq/1da4000.ufshc/governor"));
        for bad in ["", "../x", "a b", "a;b", "A"] {
            assert!(
                governor_read_command(bad).is_err() && governor_pin_command(bad).is_err(),
                "{bad}"
            );
        }
        assert!(ramp_step_command("sda1", 1, 1, 1, 1).is_err());
    }

    #[test]
    fn a_node_without_powersave_stops_the_run_before_any_load() {
        let u = ufs(vec![("cat $d/governor", "no-governor-node")], "nonode");
        let e = u.the_governor_is_pinned_to_powersave()().unwrap_err();
        assert!(
            e.contains("no powersave to pin") && e.contains("before any load"),
            "{e}"
        );
        let u = ufs(
            vec![(
                "cat $d/governor",
                "governor=performance\navailable=performance\ncur_freq=1",
            )],
            "nops",
        );
        assert!(u.the_governor_is_pinned_to_powersave()()
            .unwrap_err()
            .contains("no powersave"));
    }

    #[test]
    fn the_governor_is_pinned_and_read_back() {
        // the first read says simple_ondemand, the second (after the pin) powersave at 50 MHz
        let t = vec![
            (
                "cat $d/governor",
                Box::leak(format!("{POWERSAVE}|||{PINNED}").into_boxed_str()) as &'static str,
            ),
            ("pinned-", "pinned-42"),
        ];
        let u = ufs(t, "pin");
        u.the_governor_is_pinned_to_powersave()().unwrap();
        assert!(
            saved(&u, "governor-before.txt").contains("simple_ondemand")
                && saved(&u, "governor-after.txt").contains("powersave")
        );
        // a pin that did not take is an error
        let t = vec![("cat $d/governor", POWERSAVE), ("pinned-", "pinned-42")];
        assert!(ufs(t, "notaken").the_governor_is_pinned_to_powersave()()
            .unwrap_err()
            .contains("did not take powersave"));
    }

    #[test]
    fn a_ramp_with_an_answering_shell_runs_every_step_and_reads_the_ram_log() {
        let t = vec![
            ("load-started-", "load-started-42"),
            (
                "echo state-",
                Box::leak(format!("{NOT_DONE}|||{DONE0}").into_boxed_str()) as &'static str,
            ),
            (
                "power_info",
                "gear=HS_GEAR1\nlane=2\nlink_state=ACTIVE\nclock_1da4000.ufshc=50000000",
            ),
            (
                "cat /tmp/ramp.log",
                "step 1/3 start 1 MiB uptime 40.1\nstep 1/3 end rc=0 uptime 40.4\nlog-read-42",
            ),
        ];
        let u = ufs(t, "all");
        u.the_gear_is_read_under_load()().unwrap();
        let r = saved(&u, "ramp-result.txt");
        assert!(
            r.contains("no silent step") && r.matches("ended rc=0").count() == 3,
            "{r}"
        );
        assert!(saved(&u, "ramp-log.txt").contains("step 1/3 start 1 MiB"));
        assert!(saved(&u, "gear-load.txt").contains("=== step 3/3 4 MiB"));
    }

    #[test]
    fn a_silent_step_is_a_result_not_an_error_and_stops_the_ramp() {
        // the first state read of step 2 is never answered (like run 3 at 4.6 s); the shell comes back on a fresh connection
        let t = vec![
            ("load-started-", "load-started-42"),
            (
                "echo state-",
                Box::leak(
                    format!("{NOT_DONE}|||{DONE0}|||?once={NOT_DONE}|||{DONE0}").into_boxed_str(),
                ) as &'static str,
            ),
            (
                "power_info",
                "gear=HS_GEAR3\nlane=2\nlink_state=ACTIVE\nclock_1da4000.ufshc=200000000",
            ),
            (
                "cat /tmp/ramp.log",
                "step 2/3 start 2 MiB uptime 40.1\nlog-read-42",
            ),
        ];
        let u = ufs(t, "silent");
        u.the_gear_is_read_under_load()().unwrap();
        let r = saved(&u, "ramp-result.txt");
        assert!(
            r.contains("FIRST SILENT STEP: 1 polls not answered") && r.contains("clock 200000000"),
            "{r}"
        );
        assert!(
            !r.contains("no silent step") && r.contains("steps not started after the silence"),
            "{r}"
        );
    }

    #[test]
    fn a_step_that_ends_with_an_error_code_is_an_error() {
        let t = vec![
            ("load-started-", "load-started-42"),
            ("echo state-", "rc=1\nstate-42"),
            ("cat /tmp/load.out", "dd: I/O error"),
            ("power_info", "gear=HS_GEAR1\nlink_state=ACTIVE"),
        ];
        let e = ufs(t, "err").the_gear_is_read_under_load()().unwrap_err();
        assert!(e.contains("ended with 1") && e.contains("I/O error"), "{e}");
    }

    #[test]
    fn the_pon_reason_is_saved_even_when_adb_says_nothing() {
        let mut s = Settings::stand_in(Try::Ufs);
        s.adb = "/nonexistent/adb".into();
        s.out = std::env::temp_dir().join(format!("wt-pon-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&s.out);
        let u = Ufs::new(s);
        u.the_pon_reason_is_read()().unwrap();
        let t = saved(&u, "pon-reason.txt");
        assert!(t.contains("ro.boot.bootreason=not known yet"), "{t}");
    }
}

#[cfg(test)]
mod after_silence_tests {
    use super::*;
    use crate::phone::fake;
    use crate::settings::{Settings, Try};

    fn ufs(name: &str, silent: bool) -> Ufs {
        let mut s = Settings::stand_in(Try::Ufs);
        s.port = fake::phone(vec![("sha256sum", "no digest")], vec![]);
        s.out = std::env::temp_dir().join(format!("wt-after-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&s.out);
        std::fs::create_dir_all(&s.out).unwrap();
        if silent {
            std::fs::write(
                s.out.join("ramp-result.txt"),
                "step 2/3 256 MiB: FIRST SILENT STEP: 4 polls\n",
            )
            .unwrap();
        }
        Ufs::new(s)
    }

    #[test]
    fn after_a_silent_ramp_the_load_steps_say_not_known_yet_and_the_run_goes_on() {
        let u = ufs("silent", true);
        u.the_gear_is_read_two_seconds_after()().unwrap();
        u.the_gpt_is_unchanged()().unwrap();
        for n in ["gear-after.txt", "gpt-after.txt"] {
            let t = std::fs::read_to_string(u.s.out.join(n)).unwrap();
            assert!(t.starts_with("not known yet"), "{n}: {t}");
        }
    }

    #[test]
    fn after_a_ramp_that_never_went_silent_a_failed_read_is_still_an_error() {
        let u = ufs("answering", false);
        assert!(u.the_gpt_is_unchanged()().is_err());
    }

    #[test]
    fn a_phone_that_hangs_up_under_load_is_a_result_naming_its_step() {
        // the stand-in has no listener for the shell: every connection is refused at once
        let mut s = Settings::stand_in(Try::Ufs);
        s.port = 1; // nothing listens
        s.out = std::env::temp_dir().join(format!("wt-gone-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&s.out);
        s.device_wait = 1;
        let u = Ufs::new(s);
        // no shell at the start of the ramp is an error of the run (the phone was not there), never a silent step
        assert!(u.the_gear_is_read_under_load()().is_err());
    }
}
