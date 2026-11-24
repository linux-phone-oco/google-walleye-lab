// SPDX-License-Identifier: MIT OR Apache-2.0
//! UFS try 2's new part (item walleye-ufs-try-2-one-command): the link gear and the clock read idle, DURING a bounded
//! read-only O_DIRECT read of one disk, and 2 s after it; the disk's GPT hashed before and after; the slots' flags before
//! and after. Nothing is written to the card: the read's output goes to /dev/null, nothing is mounted.

use super::{step_fits, uptime_of, Link, Speed, Ufs, UPTIME_COMMAND};
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

/// The clock the ramped try holds the UFS host at: the lowest of the node's two OPPs (research walleye-ufs-hold-the-clock-read, §1-2:
/// `freq-table-hz` core_clk 50 MHz..200 MHz; run 4's `gear-before.txt` read `clock 50000000` at idle).
pub const PIN_HZ: &str = "50000000";

/// What `cur_freq` reads for a pin in force while clock scaling is idle-suspended: `ufshcd_devfreq_target` rounds the request through
/// `clk_round_rate(GCC_UFS_AXI_CLK)`, whose mainline table starts at 100 MHz (research walleye-ufs-pin-100mhz-read, (1); runs 6 and 7 read it).
pub const ROUNDED_HZ: &str = "100000000";

/// The link gear the low state gives on this card (research walleye-ufs-pin-100mhz-read, (4); runs 6 and 7's `gear-before.txt` read it).
pub const PIN_GEAR: &str = "HS_GEAR1";

/// The command that reads the node's governor, the governors it has, the clocks it offers, its floor, ceiling and clock, and the
/// userspace governor's `set_freq` where there is one; a node that is not there says `no-governor-node`.
pub fn governor_read_command(node: &str) -> Result<String, String> {
    if !is_a_node_name(node) {
        return Err(format!("devfreq node {node:?} is not a node name"));
    }
    Ok(format!(
        "d=/sys/class/devfreq/{node}; if [ -e $d/governor ]; then echo governor=$(cat $d/governor); echo available=$(cat $d/available_governors); echo frequencies=$(cat $d/available_frequencies); echo min_freq=$(cat $d/min_freq 2>/dev/null); echo max_freq=$(cat $d/max_freq 2>/dev/null); echo cur_freq=$(cat $d/cur_freq); echo set_freq=$(cat $d/userspace/set_freq 2>/dev/null); else echo no-governor-node; fi"
    ))
}

/// One capture of the pin's evidence (research walleye-ufs-pin-100mhz-read, "the next run must capture"): the governor read, the node's
/// `trans_stat`, the link's `power_info` lines (gear, mode, lanes) and the debugfs clock `gcc_ufs_axi_clk` where debugfs is mounted
/// (`not-known-yet` where it is not). Reads only.
pub fn pin_evidence_command(node: &str) -> Result<String, String> {
    Ok(format!(
        "{}; echo trans_stat_begin; cat /sys/class/devfreq/{node}/trans_stat 2>&1; echo trans_stat_end; {}; echo gcc_ufs_axi_clk_rate=$(cat /sys/kernel/debug/clk/gcc_ufs_axi_clk/clk_rate 2>/dev/null || echo not-known-yet)",
        governor_read_command(node)?,
        super::LINK_COMMAND
    ))
}

/// The kernel log's scaling lines, read AFTER the write: the 'clock scaling is suspended, skip' line proves the rounding directly
/// (runs 6 and 7's dmesg.txt was taken at boot, before the write). An empty answer is a reading: no scaling line.
pub fn scaling_log_command() -> String {
    "dmesg | grep -i scaling | tail -n 20; echo scaling-log-read-$((6*7))".to_string()
}

/// The writes of the pin, in order, each with the name of the capture taken after it: the userspace governor, then `set_freq` at
/// [`PIN_HZ`] (the way mainline takes); a node without userspace gets `max_freq` clamped to the same clock. A runtime setting of the
/// phone's RAM kernel: nothing flashed, nothing written to the card.
pub fn clock_pin_steps(node: &str, g: &Governor) -> Result<Vec<(&'static str, String)>, String> {
    if !is_a_node_name(node) {
        return Err(format!("devfreq node {node:?} is not a node name"));
    }
    let d = format!("/sys/class/devfreq/{node}");
    if g.can_userspace() {
        Ok(vec![
            (
                "pin-after-governor-write.txt",
                format!("echo userspace > {d}/governor; echo pinned-$((6*7))"),
            ),
            (
                "pin-after-set-freq.txt",
                format!("echo {PIN_HZ} > {d}/userspace/set_freq; echo pinned-$((6*7))"),
            ),
        ])
    } else {
        Ok(vec![(
            "pin-after-max-freq.txt",
            format!("echo {PIN_HZ} > {d}/max_freq; echo pinned-$((6*7))"),
        )])
    }
}

/// What the governor read said.
#[derive(Debug, Clone, PartialEq)]
pub struct Governor {
    pub present: bool,
    pub current: Option<String>,
    pub available: Vec<String>,
    pub frequencies: Vec<String>,
    pub min_freq: Option<String>,
    pub cur_freq: Option<String>,
    pub max_freq: Option<String>,
    pub set_freq: Option<String>,
}

impl Governor {
    pub fn of(text: &str) -> Governor {
        let value = |k: &str| {
            text.lines()
                .find_map(|l| l.trim().strip_prefix(k)?.strip_prefix('='))
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        };
        let words = |k: &str| {
            value(k)
                .map(|v| v.split_whitespace().map(str::to_string).collect())
                .unwrap_or_default()
        };
        Governor {
            present: !text.contains("no-governor-node") && value("governor").is_some(),
            current: value("governor"),
            available: words("available"),
            frequencies: words("frequencies"),
            min_freq: value("min_freq"),
            cur_freq: value("cur_freq"),
            max_freq: value("max_freq"),
            set_freq: value("set_freq"),
        }
    }

    pub fn can_userspace(&self) -> bool {
        self.present && self.available.iter().any(|g| g == "userspace")
    }

    /// Whether the node offers the clock to pin at; an empty list is not known yet, so it does not refuse (the read-back does).
    pub fn offers_the_pin(&self) -> bool {
        self.frequencies.is_empty() || self.frequencies.iter().any(|f| f == PIN_HZ)
    }

    /// What is wrong with a pin read back, empty when it holds. The pin is judged by the setting that holds it and the state it gives
    /// (decision 2026-11-24-walleye-ufs-pin-asserts-the-holding-setting, research walleye-ufs-pin-100mhz-read): the userspace governor
    /// with `set_freq` at [`PIN_HZ`] (or `max_freq` at it on a node without userspace), the link gear [`PIN_GEAR`], and `cur_freq` at
    /// [`PIN_HZ`] or [`ROUNDED_HZ`] (core_clk's rounding while scaling is idle-suspended). Never `cur_freq` == 50 MHz alone, never
    /// `min_freq`. A value not read is "not known yet" and does not pass.
    pub fn pin_flaws(&self, by_userspace: bool, gear: &str) -> Vec<String> {
        let known = |v: Option<&str>| {
            v.filter(|s| !s.is_empty())
                .unwrap_or("not known yet")
                .to_string()
        };
        let mut flaws = vec![];
        if by_userspace {
            if self.current.as_deref() != Some("userspace") {
                flaws.push(format!(
                    "governor {} (wanted userspace)",
                    known(self.current.as_deref())
                ));
            }
            if self.set_freq.as_deref() != Some(PIN_HZ) {
                flaws.push(format!(
                    "userspace/set_freq {} (wanted {PIN_HZ})",
                    known(self.set_freq.as_deref())
                ));
            }
        } else if self.max_freq.as_deref() != Some(PIN_HZ) {
            flaws.push(format!(
                "max_freq {} (wanted {PIN_HZ})",
                known(self.max_freq.as_deref())
            ));
        }
        if !matches!(self.cur_freq.as_deref(), Some(PIN_HZ | ROUNDED_HZ)) {
            flaws.push(format!(
                "cur_freq {} (wanted {PIN_HZ}, or {ROUNDED_HZ} as core_clk's rounding)",
                known(self.cur_freq.as_deref())
            ));
        }
        let number = |g: &str| {
            g.trim_start_matches(|c: char| !c.is_ascii_digit())
                .to_string()
        };
        if number(gear) != number(PIN_GEAR) {
            flaws.push(format!("gear {} (wanted {PIN_GEAR})", known(Some(gear))));
        }
        flaws
    }

    pub fn line(&self) -> String {
        format!(
            "governor {}, clock {}",
            self.current.as_deref().unwrap_or("not known yet"),
            self.cur_freq.as_deref().unwrap_or("not known yet")
        )
    }

    /// Every reading of the node, for the stop line: what the next reader needs to see why a pin was refused.
    pub fn evidence(&self, gear: &str) -> String {
        let k = |v: &Option<String>| v.clone().unwrap_or_else(|| "not known yet".to_string());
        format!(
            "governor {}, available_frequencies [{}], min_freq {}, max_freq {}, cur_freq {}, set_freq {}, gear {}",
            k(&self.current),
            self.frequencies.join(" "),
            k(&self.min_freq),
            k(&self.max_freq),
            k(&self.cur_freq),
            k(&self.set_freq),
            if gear.is_empty() { "not known yet" } else { gear }
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
    /// One capture of the pin's evidence, kept whole in `name`, read as the node and the link's gear name (empty: not known yet).
    fn pin_capture(&self, sh: &mut Shell, name: &str) -> Result<(Governor, String), String> {
        let wait = Duration::from_secs(self.s.gear_wait);
        let said = sh.run(&pin_evidence_command(&self.s.devfreq)?, wait)?;
        self.lab.save(name, &said)?;
        Ok((Governor::of(&said), Link::of(&said).gear))
    }

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

    /// The UFS host's devfreq clock is held low BEFORE any load (research walleye-ufs-pin-100mhz-read, decision
    /// 2026-11-24-walleye-ufs-pin-asserts-the-holding-setting): the node, its link and the debugfs clock are captured before the write, after
    /// the governor write and after `set_freq` 50 MHz (a node without userspace: after `max_freq`), each to its own file, and the scaling
    /// lines of the kernel log after the write. The pin holds when the governor reads userspace, `set_freq` reads 50 MHz, the gear reads
    /// HS_GEAR1 and `cur_freq` reads 50 or 100 MHz (core_clk's rounding); otherwise the run stops before any load and its stop line prints
    /// every reading. Whether the 100 MHz core clock bears on the silence under load is NOT KNOWN YET.
    pub fn the_clock_is_pinned_low(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let node = &u.s.devfreq;
            let mut sh = u.lab.shell(10)?;
            let wait = Duration::from_secs(u.s.gear_wait);
            let (before, before_gear) = u.pin_capture(&mut sh, "pin-before.txt")?;
            u.lab.save("governor-before.txt", &before.line())?;
            if !before.present {
                return Err(format!(
                    "devfreq {node} has no governor node: the run stops before any load"
                ));
            }
            if !before.can_userspace() && before.available.iter().all(|g| g != "simple_ondemand") {
                return Err(format!(
                    "devfreq {node} offers no governor to clamp ({}): the run stops before any load",
                    before.available.join(" ")
                ));
            }
            if !before.offers_the_pin() {
                return Err(format!(
                    "devfreq {node} does not offer {PIN_HZ} Hz (it offers {}): the run stops before any load",
                    before.frequencies.join(" ")
                ));
            }
            let mut seen = vec![format!("before: {}", before.evidence(&before_gear))];
            let mut last = (before.clone(), before_gear);
            for (name, write) in clock_pin_steps(node, &before)? {
                let wrote = sh.run(&write, wait)?;
                u.lab
                    .save(&name.replace("pin-after", "pin-write"), &wrote)?;
                let (g, gear) = u.pin_capture(&mut sh, name)?;
                seen.push(format!(
                    "{}: {}",
                    name.trim_end_matches(".txt"),
                    g.evidence(&gear)
                ));
                last = (g, gear);
            }
            let scaling = sh.run(&scaling_log_command(), wait)?;
            u.lab.save("pin-scaling-log-after.txt", &scaling)?;
            let (after, after_gear) = last;
            u.lab.save("governor-after.txt", &after.line())?;
            let flaws = after.pin_flaws(before.can_userspace(), &after_gear);
            if !flaws.is_empty() {
                let skip = scaling
                    .lines()
                    .find(|l| l.contains("clock scaling is suspended"))
                    .unwrap_or("no 'clock scaling is suspended' line");
                return Err(format!(
                    "the pin did not take ({}); readings: {}; kernel log: {skip}; the run stops before any load",
                    flaws.join("; "),
                    seen.join(" | ")
                ));
            }
            println!(
                "  {node}: {} -> {} (gear {after_gear})",
                before.line(),
                after.line()
            );
            u.read_gear("gear-pinned.txt")?;
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
            // the image's init reboots the phone at its deadline (decision 2026-11-24-walleye-ufs-procedure-fits-the-image-not-the-image-the-procedure):
            // a step runs only when its limit ends before it; the uptime at its start is kept with the step
            let deadline = self
                .deadline
                .get()
                .ok_or("the image's reboot deadline is not known yet: no step is started")?;
            let said = match state_sh.run(UPTIME_COMMAND, Duration::from_secs(10)) {
                Ok(t) => t,
                Err(e) => {
                    // a phone that does not answer between steps is this step's silence: kept as the result, like an unanswered start
                    samples.push_str(&format!(
                        "--- the uptime before the step was not answered: {e}\n"
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
            };
            let up =
                uptime_of(&said).ok_or("the phone's uptime could not be read before the step")?;
            samples.push_str(&format!(
                "=== step {}/{} {mb} MiB at uptime {up:.1} s, image reboots at {deadline:.1} s\n",
                i + 1,
                steps.len()
            ));
            if let Err(why) = step_fits(up, deadline, self.s.load_limit) {
                samples.push_str(&format!("--- step refused: {why}\n"));
                return Err(format!(
                    "step {}/{} ({mb} MiB) refused, not started: {why}",
                    i + 1,
                    steps.len()
                ));
            }
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
    /// and let the run reach the way back, where the PON reason is read.
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

    fn ufs(mut table: Vec<(&'static str, &'static str)>, name: &str) -> Ufs {
        let mut s = Settings::stand_in(Try::Ufs);
        table.push(("/proc/uptime", "40.10 300.20")); // last: a step command also names /proc/uptime
        s.port = fake::phone(table, vec![]);
        s.out = out(name);
        let _ = std::fs::remove_dir_all(&s.out);
        s.device_wait = 2;
        let u = Ufs::new(s);
        u.deadline.set(Some(181.1)); // the matched image: petter at 1.1 s + 180 s
        u
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
                && t.contains("userspace governor"),
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
    /// What the phone printed in run 4 of try 2 (~/lab-material/walleye/ufs-try2-run4-ramped.out, saved as a fixture, research row
    /// walleye-ufs-try-2-run-4-no-powersave-governor): the governors of devfreq 1da4000.ufshc and the idle clock.
    const RUN4: &str = include_str!("../../fixtures/ufs-try2-run4-ramped.out");

    /// The governors run 4's stop line printed ("governors: userspace simple_ondemand)").
    fn run4_governors() -> String {
        RUN4.split("governors: ")
            .nth(1)
            .and_then(|t| t.split(')').next())
            .expect("run 4's stop line names the governors")
            .to_string()
    }

    /// The idle clock run 4's check 15 printed ("clock 50000000").
    fn run4_idle_clock() -> String {
        RUN4.split("; clock ")
            .nth(1)
            .and_then(|t| t.split_whitespace().next())
            .expect("run 4's check 15 names the clock")
            .to_string()
    }

    /// A governor read of the phone's node as the read command prints it: governors and idle clock are run 4's; the clock the node
    /// offers is not known yet from any capture (the first listing of the pinned try prints it), so `offers` says what a test gives it.
    fn read_text(governor: &str, offers: &str, cur: &str) -> String {
        format!(
            "governor={governor}\navailable={}\nfrequencies={offers}\ncur_freq={cur}",
            run4_governors()
        )
    }

    fn ufs(mut table: Vec<(&'static str, &'static str)>, name: &str) -> Ufs {
        let mut s = Settings::stand_in(Try::Ufs);
        table.push(("/proc/uptime", "40.10 300.20")); // last: a step command also names /proc/uptime
        s.port = fake::phone(table, vec![]);
        s.out = std::env::temp_dir().join(format!("wt-ramp-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&s.out);
        s.device_wait = 2;
        s.load_limit = 1;
        let u = Ufs::new(s);
        u.deadline.set(Some(181.1)); // the matched image: petter at 1.1 s + 180 s
        u
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
    fn the_governor_is_read_from_the_text_the_phone_printed() {
        assert_eq!(run4_governors(), "userspace simple_ondemand");
        assert_eq!(run4_idle_clock(), PIN_HZ);
        let g = Governor::of(&read_text("simple_ondemand", "", &run4_idle_clock()));
        assert!(g.can_userspace() && g.offers_the_pin());
        // the node as run 4 found it is not a pin: the governor and set_freq say so, whatever the clock reads
        assert!(
            g.pin_flaws(true, PIN_GEAR).len() >= 2,
            "{:?}",
            g.pin_flaws(true, PIN_GEAR)
        );
        assert!(!Governor::of("no-governor-node").can_userspace());
        assert!(!Governor::of("governor=a\navailable=performance\ncur_freq=1").can_userspace());
        assert!(
            !Governor::of(&read_text("simple_ondemand", "200000000", "200000000")).offers_the_pin()
        );
        assert!(Governor::of("").line().contains("not known yet"));
        let all = Governor::of(&with_set_freq(
            &read_text("userspace", "50000000 200000000", "100000000"),
            "50000000",
        ));
        assert!(
            all.evidence(PIN_GEAR).contains("set_freq 50000000")
                && all.evidence("").contains("gear not known yet")
        );
    }

    /// Run 6's captured text (~/lab-material/walleye/ufs-20261124-000728, run 7's is the same, saved as fixtures; research
    /// walleye-ufs-pin-100mhz-read): the node's governor and clock before and after the old pin, and the link's gear read idle before.
    const RUN6_GOV_AFTER: &str = include_str!("../../fixtures/ufs-try2-run6-governor-after.txt");
    const RUN6_GOV_BEFORE: &str = include_str!("../../fixtures/ufs-try2-run6-governor-before.txt");
    const RUN6_GEAR: &str = include_str!("../../fixtures/ufs-try2-run6-gear-before.txt");

    /// The governor read of run 6's `governor-after.txt` line ("governor userspace, clock 100000000"): what the old step read back.
    fn run6_after() -> Governor {
        let (gov, clock) = RUN6_GOV_AFTER
            .trim()
            .strip_prefix("governor ")
            .and_then(|t| t.split_once(", clock "))
            .expect("run 6's governor-after line");
        Governor::of(&format!("governor={gov}\ncur_freq={clock}"))
    }

    /// `text` with the userspace governor's `set_freq` line.
    fn with_set_freq(text: &str, hz: &str) -> String {
        format!("{text}\nset_freq={hz}")
    }

    #[test]
    fn run_6s_100_mhz_read_is_the_clock_passing_and_asks_for_set_freq_and_the_gear() {
        assert_eq!(
            RUN6_GOV_BEFORE.trim(),
            "governor simple_ondemand, clock 50000000"
        );
        let gear = Link::of(RUN6_GEAR).gear;
        assert_eq!(gear, PIN_GEAR); // run 6's gear before the write: the low state's gear
        let after = run6_after();
        assert_eq!(after.cur_freq.as_deref(), Some(ROUNDED_HZ));
        // the old step refused this read on the clock; under the new assertion only the set_freq read is missing (it was not captured)
        let flaws = after.pin_flaws(true, &gear);
        assert_eq!(flaws.len(), 1, "{flaws:?}");
        assert!(
            flaws[0].contains("userspace/set_freq not known yet"),
            "{flaws:?}"
        );
        // with set_freq read back at 50 MHz the same text is a pin
        let held = Governor {
            set_freq: Some(PIN_HZ.into()),
            ..after.clone()
        };
        assert!(held.pin_flaws(true, &gear).is_empty());
        // the clock passes as the lowest OPP too, and fails at anything else
        let low = Governor {
            cur_freq: Some(PIN_HZ.into()),
            ..held.clone()
        };
        assert!(low.pin_flaws(true, &gear).is_empty());
        let up = Governor {
            cur_freq: Some("200000000".into()),
            ..held.clone()
        };
        assert!(up.pin_flaws(true, &gear)[0].contains("cur_freq 200000000"));
        // a gear above the low state fails, whatever the clock
        assert!(held.pin_flaws(true, "HS_GEAR3")[0].contains("gear HS_GEAR3"));
        assert!(held.pin_flaws(true, "")[0].contains("gear not known yet"));
        // the governor that is not userspace fails
        let free = Governor {
            current: Some("simple_ondemand".into()),
            ..held
        };
        assert!(free.pin_flaws(true, &gear)[0].contains("governor simple_ondemand"));
    }

    #[test]
    fn a_node_without_userspace_is_pinned_by_max_freq_and_the_same_gear() {
        let g = Governor::of("governor=simple_ondemand\navailable=simple_ondemand\ncur_freq=100000000\nmax_freq=50000000");
        assert!(g.pin_flaws(false, PIN_GEAR).is_empty());
        let open = Governor::of("governor=simple_ondemand\navailable=simple_ondemand\ncur_freq=100000000\nmax_freq=200000000");
        assert!(open.pin_flaws(false, PIN_GEAR)[0].contains("max_freq 200000000"));
    }

    #[test]
    fn the_commands_of_the_ramp_are_shell_and_read_only() {
        let with = Governor::of(&read_text("simple_ondemand", "", "1"));
        let without =
            Governor::of("governor=simple_ondemand\navailable=simple_ondemand\ncur_freq=1");
        let node = "1da4000.ufshc";
        let userspace = clock_pin_steps(node, &with).unwrap();
        let clamp = clock_pin_steps(node, &without).unwrap();
        let mut cmds = vec![
            governor_read_command(node).unwrap(),
            pin_evidence_command(node).unwrap(),
            scaling_log_command(),
            ramp_step_command("sda", 2, 3, 256, 60).unwrap(),
            ram_log_command(),
        ];
        cmds.extend(userspace.iter().map(|(_, c)| c.clone()));
        cmds.extend(clamp.iter().map(|(_, c)| c.clone()));
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
        let step = &cmds[3];
        assert!(
            step.contains("iflag=direct")
                && step.contains("count=256")
                && step.contains("timeout 60")
        );
        assert!(
            step.find(RAM_LOG).unwrap() < step.find("dd if=").unwrap(),
            "the RAM log line is written BEFORE the read"
        );
        let d = "/sys/class/devfreq/1da4000.ufshc";
        // the governor and set_freq are two writes, each followed by its own capture
        assert_eq!(userspace.len(), 2);
        assert!(
            userspace[0]
                .1
                .contains(&format!("userspace > {d}/governor"))
                && !userspace[0].1.contains("set_freq")
        );
        assert!(userspace[1]
            .1
            .contains(&format!("{PIN_HZ} > {d}/userspace/set_freq")));
        assert_eq!(clamp.len(), 1);
        assert!(
            clamp[0].1.contains(&format!("{PIN_HZ} > {d}/max_freq"))
                && !clamp[0].1.contains("governor")
        );
        // the evidence names everything the research asks for
        let e = &cmds[1];
        for key in [
            "governor=",
            "available=",
            "frequencies=",
            "min_freq=",
            "max_freq=",
            "cur_freq=",
            "set_freq=",
            "trans_stat",
            "power_info",
            "gcc_ufs_axi_clk/clk_rate",
        ] {
            assert!(e.contains(key), "{key}");
        }
        for bad in ["", "../x", "a b", "a;b", "A"] {
            assert!(
                governor_read_command(bad).is_err() && clock_pin_steps(bad, &with).is_err(),
                "{bad}"
            );
        }
        assert!(ramp_step_command("sda1", 1, 1, 1, 1).is_err());
    }

    #[test]
    fn a_node_that_cannot_be_pinned_stops_the_run_before_any_load() {
        let u = ufs(vec![("cat $d/governor", "no-governor-node")], "nonode");
        let e = u.the_clock_is_pinned_low()().unwrap_err();
        assert!(
            e.contains("no governor node") && e.contains("before any load"),
            "{e}"
        );
        let u = ufs(
            vec![(
                "cat $d/governor",
                "governor=performance\navailable=performance\ncur_freq=1",
            )],
            "nogov",
        );
        assert!(u.the_clock_is_pinned_low()()
            .unwrap_err()
            .contains("no governor to clamp"));
        let offered =
            Box::leak(read_text("simple_ondemand", "200000000", "200000000").into_boxed_str());
        let e = ufs(vec![("cat $d/governor", &*offered)], "nooffer").the_clock_is_pinned_low()()
            .unwrap_err();
        assert!(
            e.contains("does not offer 50000000") && e.contains("before any load"),
            "{e}"
        );
    }

    /// A capture of the evidence command as the phone prints it: the governor read, then the link's lines (run 6's real
    /// `gear-before.txt`, its gear swapped when a test gives another) - the rest of the capture format is NOT KNOWN YET from a run.
    fn evidence(governor: &str, cur: &str, set_freq: Option<&str>, gear: &str) -> String {
        let node = read_text(governor, "50000000 200000000", cur);
        let node = set_freq.map_or(node.clone(), |hz| with_set_freq(&node, hz));
        format!("{node}\n{}", RUN6_GEAR.replace(PIN_GEAR, gear))
    }

    /// The step's table: the three captures in order, the scaling log, the writes' echo, the gear read after.
    fn pin_table(captures: [&str; 3], scaling: &str) -> Vec<(&'static str, &'static str)> {
        let leak = |s: String| -> &'static str { Box::leak(s.into_boxed_str()) };
        vec![
            ("cat $d/governor", leak(captures.join("|||"))),
            ("dmesg", leak(format!("{scaling}\nscaling-log-read-42"))),
            ("pinned-", "pinned-42"),
            ("power_info", RUN3_BEFORE),
        ]
    }

    #[test]
    fn the_clock_is_pinned_low_and_the_evidence_is_captured_at_each_point() {
        let before = evidence("simple_ondemand", "200000000", None, PIN_GEAR);
        let gov = evidence("userspace", "100000000", None, PIN_GEAR);
        let set = evidence("userspace", "100000000", Some(PIN_HZ), PIN_GEAR);
        let u = ufs(pin_table([&before, &gov, &set], ""), "pin");
        u.the_clock_is_pinned_low()().unwrap();
        // each point has its own file: before, after the governor write, after set_freq, and the kernel log after
        assert!(saved(&u, "pin-before.txt").contains("governor=simple_ondemand"));
        assert!(saved(&u, "pin-after-governor-write.txt").contains("governor=userspace"));
        assert!(saved(&u, "pin-after-set-freq.txt").contains("set_freq=50000000"));
        assert!(saved(&u, "pin-scaling-log-after.txt").contains("scaling-log-read-42"));
        assert!(
            saved(&u, "governor-before.txt").contains("simple_ondemand")
                && saved(&u, "governor-after.txt").contains("userspace, clock 100000000")
                && saved(&u, "gear-pinned.txt").contains("clock_1da4000.ufshc=50000000")
        );
    }

    #[test]
    fn a_pin_that_is_not_in_force_stops_the_run_before_any_load_and_prints_its_readings() {
        let before = evidence("simple_ondemand", "200000000", None, PIN_GEAR);
        let gov = evidence("userspace", "100000000", None, PIN_GEAR);
        let refused = |name: &str, set: String, scaling: &str| {
            let u = ufs(pin_table([&before, &gov, &set], scaling), name);
            u.the_clock_is_pinned_low()().unwrap_err()
        };
        // set_freq not read back: not known yet, the readings of all three points are in the line
        let e = refused(
            "noset",
            evidence("userspace", "100000000", None, PIN_GEAR),
            "",
        );
        assert!(
            e.contains("userspace/set_freq not known yet") && e.contains("before any load"),
            "{e}"
        );
        assert!(
            e.contains("before: governor simple_ondemand")
                && e.contains("pin-after-governor-write: governor userspace")
                && e.contains("pin-after-set-freq: governor userspace")
                && e.contains("no 'clock scaling is suspended' line"),
            "{e}"
        );
        // the write failed (governor unchanged): refused, naming the governor
        let e = refused(
            "unchanged",
            evidence("simple_ondemand", "50000000", Some("0"), PIN_GEAR),
            "",
        );
        assert!(
            e.contains("governor simple_ondemand (wanted userspace)") && e.contains("set_freq 0"),
            "{e}"
        );
        // the clock above the low state, and a gear above it, are each refused
        let e = refused(
            "fast",
            evidence("userspace", "200000000", Some(PIN_HZ), PIN_GEAR),
            "",
        );
        assert!(e.contains("cur_freq 200000000"), "{e}");
        let e = refused(
            "gear",
            evidence("userspace", "100000000", Some(PIN_HZ), "HS_GEAR3"),
            "",
        );
        assert!(e.contains("gear HS_GEAR3 (wanted HS_GEAR1)"), "{e}");
        // the kernel log's line is printed when it is there (the phrase is the decision's; the line itself is NOT KNOWN YET from a run)
        let e = refused(
            "logged",
            evidence("userspace", "200000000", Some(PIN_HZ), PIN_GEAR),
            "ufshcd-qcom 1da4000.ufshc: clock scaling is suspended, skip",
        );
        assert!(
            e.contains("kernel log: ufshcd-qcom 1da4000.ufshc: clock scaling is suspended, skip"),
            "{e}"
        );
    }

    #[test]
    fn a_node_without_userspace_is_clamped_and_read_back_by_max_freq() {
        let node = |max: &str| {
            format!(
                "governor=simple_ondemand\navailable=simple_ondemand\nfrequencies=50000000 200000000\ncur_freq=100000000\nmax_freq={max}\n{RUN6_GEAR}"
            )
        };
        let (open, clamped) = (node("200000000"), node(PIN_HZ));
        let t = pin_table([&open, &clamped, &clamped], "");
        let u = ufs(t, "clamp");
        u.the_clock_is_pinned_low()().unwrap();
        assert!(saved(&u, "pin-after-max-freq.txt").contains("max_freq=50000000"));
        let t = pin_table([&open, &open, &open], "");
        let e = ufs(t, "clampfailed").the_clock_is_pinned_low()().unwrap_err();
        assert!(e.contains("max_freq 200000000 (wanted 50000000)"), "{e}");
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

    /// Proof of item walleye-ufs-procedure-fits-the-180s: the image reboots at 181.1 s (petter 1.1 s + 180 s); at uptime 130 s a step
    /// that may take 60 s would end at 190 s: it is refused and named, nothing is started, and the uptime is kept.
    #[test]
    fn a_step_that_would_cross_the_reboot_deadline_is_refused_and_named() {
        // no row answers a start command: a step that started would end "did not start", not "refused"
        let t = vec![("/proc/uptime", "130.00 900.00")];
        let mut s = Settings::stand_in(Try::Ufs);
        s.port = fake::phone(t, vec![]);
        s.out = std::env::temp_dir().join(format!("wt-ramp-deadline-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&s.out);
        s.load_limit = 60;
        let u = Ufs::new(s);
        u.deadline.set(Some(181.1));
        let e = u.the_gear_is_read_under_load()().unwrap_err();
        assert!(
            e.contains("step 1/3 (1 MiB) refused, not started")
                && e.contains("130.0 s")
                && e.contains("60 s")
                && e.contains("181.1 s"),
            "{e}"
        );
        let g = saved(&u, "gear-load.txt");
        assert!(
            g.contains("=== step 1/3 1 MiB at uptime 130.0 s, image reboots at 181.1 s")
                && g.contains("--- step refused"),
            "{g}"
        );
    }

    #[test]
    fn a_ramp_without_a_known_deadline_starts_no_step() {
        let u = ufs(vec![("load-started-", "load-started-42")], "nodeadline");
        u.deadline.set(None);
        let e = u.the_gear_is_read_under_load()().unwrap_err();
        assert!(e.contains("deadline is not known yet"), "{e}");
    }

    #[test]
    fn every_step_keeps_the_uptime_at_its_start() {
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
            ("cat /tmp/ramp.log", "log-read-42"),
        ];
        let u = ufs(t, "uptimes");
        u.the_gear_is_read_under_load()().unwrap();
        let g = saved(&u, "gear-load.txt");
        assert_eq!(
            g.matches("at uptime 40.1 s, image reboots at 181.1 s")
                .count(),
            3,
            "{g}"
        );
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
