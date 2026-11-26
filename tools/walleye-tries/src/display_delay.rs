// SPDX-License-Identifier: MIT OR Apache-2.0
//! The display delay bisect's steps (item walleye-display-delay-bisect-build; research walleye-display-b2-dsi-clock-warning): the same
//! kernel, dtb and modules load at 2 s of uptime with three signs of trouble (SMMU context faults at the bootloader's splash buffer,
//! `pclk0_clk_src: rcg didn't update its configuration`, DSI commands timing out with -110) and at 66 s with none. One image per delay
//! (b-d5 ... b-d50, or the images `image_prefix` names: B2 whose init waits the number of seconds in its own /display-delay before the first insmod); this program boots
//! them from memory one after the other and scores the three signs of each from the kernel log. Nothing is written to the phone and
//! nobody is asked: the glass is not looked at, the console and the log are read (decision 2026-11-19-walleye-display-console-before-eyes).
//! Each method is one step of procedures/display-delay-bisect-on-the-pixel-2.rs.

use crate::display_boot::{loop_is_done, ok_lines};
use crate::lab::Lab;
use crate::settings::{DelayImage, Settings};
use crate::touch::redact;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

type Done = Result<(), String>;

/// What the init says to the kernel log when the delay is over, before the first insmod.
const REACHED: &str = "walleye-display: delay ";
/// The sign words, as the research names them.
const RCG_WORDS: &str = "rcg didn't update its configuration";
/// The three signs of trouble in one kernel log, and how the load went.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// the delay baked into the image, seconds of uptime
    pub delay: u64,
    /// the uptime the init said it had reached when it started to load, `None` when it said nothing
    pub reached: Option<u64>,
    pub smmu_faults: usize,
    pub rcg_warnings: usize,
    pub dsi_110: usize,
    /// `walleye-display: ok` lines in the log, and whether the init said it was through the list
    pub modules_ok: usize,
    /// how many modules the init must say ok for
    pub modules_want: usize,
    pub finished: bool,
}

impl Row {
    /// None of the three signs is there.
    pub fn is_clean(&self) -> bool {
        self.smmu_faults + self.rcg_warnings + self.dsi_110 == 0
    }

    /// The init went through its list and every module said ok: a row without it says nothing about the signs.
    pub fn loaded(&self) -> bool {
        self.finished && self.modules_ok == self.modules_want
    }
}

/// An SMMU context fault line: the arm-smmu driver's words for a fault of a stream the kernel gave a domain.
pub fn is_smmu_fault(line: &str) -> bool {
    let l = line.to_lowercase();
    l.contains("context fault") || (l.contains("smmu") && l.contains("fault"))
}

/// The `pclk0_clk_src` warning of the DSI link clock.
pub fn is_rcg_warning(line: &str) -> bool {
    line.contains(RCG_WORDS)
}

/// A DSI command that timed out: -110 on a line of the display path.
pub fn is_dsi_timeout(line: &str) -> bool {
    let l = line.trim_end();
    !l.contains("walleye-")
        && (l.ends_with("ret=-110") || l.ends_with(": -110") || l.ends_with(" -110"))
}

/// The uptime the init said it reached for `delay`: `walleye-display: delay 15 s reached at uptime 16 s`.
pub fn reached_at(dmesg: &str, delay: u64) -> Option<u64> {
    dmesg.lines().find_map(|l| {
        let said = l.split_once(REACHED)?.1;
        let rest = said.strip_prefix(&format!("{delay} s reached at uptime "))?;
        rest.split_whitespace().next()?.parse().ok()
    })
}

/// Score one kernel log: the three signs counted by line, the modules said ok, the init through its list.
pub fn score(delay: u64, modules_want: usize, dmesg: &str) -> Row {
    // only what the kernel said from the moment the init started to load counts
    let from = dmesg.lines().position(|l| l.contains(REACHED)).unwrap_or(0);
    let count = |is: fn(&str) -> bool| dmesg.lines().skip(from).filter(|l| is(l)).count();
    Row {
        delay,
        reached: reached_at(dmesg, delay),
        smmu_faults: count(is_smmu_fault),
        rcg_warnings: count(is_rcg_warning),
        dsi_110: count(is_dsi_timeout),
        modules_ok: ok_lines(dmesg),
        modules_want,
        finished: loop_is_done(dmesg),
    }
}

/// The result table: one line per delay, the three signs counted, what the load did.
pub fn table(rows: &[Row]) -> String {
    let mut out = String::from(
        "delay_s reached_s smmu_faults rcg_warnings dsi_110 modules_ok finished verdict\n",
    );
    for r in rows {
        let reached = r.reached.map_or("-".to_string(), |s| s.to_string());
        let verdict = match (r.is_clean(), r.loaded()) {
            (true, true) => "clean",
            (true, false) => "clean-but-not-loaded",
            (false, _) => "signs",
        };
        out.push_str(&format!(
            "{} {} {} {} {} {} {} {}\n",
            r.delay,
            reached,
            r.smmu_faults,
            r.rcg_warnings,
            r.dsi_110,
            r.modules_ok,
            if r.finished { "yes" } else { "no" },
            verdict
        ));
    }
    out
}

/// The registers the image's init read (item walleye-display-scanout-registers): `walleye-regs: T=30 INTF1_STATUS 0x0c96ba6c 0x00000001`, one value per sample time.
/// A read that failed (`... FAILED devmem: ...`) is a value of `FAILED`.
pub fn register_values(dmesg: &str) -> Vec<(String, String, u64, String)> {
    dmesg
        .lines()
        .filter_map(|l| {
            let said = l.split_once("walleye-regs: T=")?.1;
            let mut w = said.split_whitespace();
            let sample: u64 = w.next()?.parse().ok()?;
            let name = w.next()?.to_string();
            let addr = w.next()?.to_string();
            let value = w.next()?.to_string();
            Some((name, addr, sample, value))
        })
        .collect()
}

/// The init has begun sampling registers but not said it is done (`registers: done`): the log is read again later. A log from an image without the regs block never waits.
pub fn registers_pending(dmesg: &str) -> bool {
    dmesg.contains("walleye-regs: sample") && !dmesg.contains("registers: done")
}

/// The table as kept: a log whose init never said `registers: done` (a read hung, or the wait ran out) says so first, so a table of dashes is not taken for a result.
pub fn regs_table_text(dmesg: &str) -> String {
    let note = if registers_pending(dmesg) {
        "incomplete: registers: done not seen\n"
    } else {
        ""
    };
    format!("{note}{}", register_table(dmesg))
}

/// The register table: one line per register, its value at each sample time, and `changes` when two samples differ (the registers that move between the samples are what to look at).
/// A register not read at a sample shows `-`.
pub fn register_table(dmesg: &str) -> String {
    let values = register_values(dmesg);
    let mut samples: Vec<u64> = values.iter().map(|v| v.2).collect();
    samples.sort_unstable();
    samples.dedup();
    let mut regs: Vec<(String, String)> = Vec::new();
    for (name, addr, _, _) in &values {
        if !regs.iter().any(|r| &r.0 == name) {
            regs.push((name.clone(), addr.clone()));
        }
    }
    let mut out = String::from("register address");
    for s in &samples {
        out.push_str(&format!(" at_{s}s"));
    }
    out.push_str(" verdict\n");
    for (name, addr) in &regs {
        out.push_str(&format!("{name} {addr}"));
        let at: Vec<String> = samples
            .iter()
            .map(|s| {
                values
                    .iter()
                    .rev()
                    .find(|v| &v.0 == name && v.2 == *s)
                    .map_or("-".to_string(), |v| v.3.clone())
            })
            .collect();
        for v in &at {
            out.push_str(&format!(" {v}"));
        }
        let seen: Vec<&String> = at.iter().filter(|v| v.as_str() != "-").collect();
        let moves = seen.iter().any(|v| *v != seen[0]);
        out.push_str(if seen.len() < 2 {
            " single\n"
        } else if moves {
            " changes\n"
        } else {
            " same\n"
        });
    }
    out
}

/// The delay at which the signs stop: the first delay whose row is clean and finished, after which every later row is too.
/// `None` when no row is clean, or a clean one is followed by one with signs (the flip is then not a point).
pub fn flip_point(rows: &[Row]) -> Option<u64> {
    let first = rows.iter().position(|r| r.is_clean() && r.loaded())?;
    rows[first..]
        .iter()
        .all(|r| r.is_clean() && r.loaded())
        .then_some(rows[first].delay)
}

/// What the table says in one sentence, for the log.
pub fn summary(rows: &[Row]) -> String {
    match (flip_point(rows), rows.iter().all(Row::is_clean)) {
        (_, true) => {
            "no delay shows a sign: the signs did not appear at any of these delays".into()
        }
        (Some(d), _) => {
            format!("the signs stop at a delay of {d} s: every delay from {d} s on is clean")
        }
        (None, _) => {
            "no single flip point: the clean rows are not all after the rows with signs".into()
        }
    }
}

/// The bisect: its settings, the lab it shares with the other tries and the rows read so far. Cheap to clone: each step takes its own.
#[derive(Clone)]
pub struct DisplayDelay {
    s: Rc<Settings>,
    lab: Lab,
    rows: Rc<RefCell<Vec<Row>>>,
}

impl DisplayDelay {
    pub fn new(s: Settings) -> DisplayDelay {
        let s = Rc::new(s);
        DisplayDelay {
            lab: Lab::new(s.clone()),
            s,
            rows: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// The delays in the order they are tried.
    pub fn delays(&self) -> Vec<u64> {
        self.s.delays.clone()
    }

    /// The test kernel the image of `delay` is (its `[images.NAME]` and `[kernels] test` entry): steps::phone_ready names it.
    pub fn kernel(&self, delay: u64) -> Result<String, String> {
        self.image(delay).map(|i| i.name)
    }

    fn image(&self, delay: u64) -> Result<DelayImage, String> {
        self.s
            .delay_images
            .iter()
            .find(|i| i.delay == delay)
            .cloned()
            .ok_or(format!("no image for a delay of {delay} s in device.toml"))
    }

    /// The battery is above the lab's minimum (the shared step).
    pub fn battery_is_good(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.battery_is_good()
    }

    /// Every image is the one built: its sha256 is its SHA256SUMS line's and the one device.toml records.
    pub fn every_image_is_the_built_one(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || {
            for i in &d.s.delay_images {
                crate::host::image_is_the_built_one(&i.path)
                    .and_then(|_| crate::host::image_has_sha256(&i.path, &i.sha256))
                    .map_err(|e| format!("{}: {e}", i.name))?;
            }
            Ok(())
        }
    }

    /// UFS is off in every image (the gate every image passes before a boot).
    pub fn ufs_is_off_in_every_image(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || {
            for i in &d.s.delay_images {
                d.lab
                    .ufs_is_off(&i.path)
                    .map_err(|e| format!("{}: {e}", i.name))?;
            }
            Ok(())
        }
    }

    /// The phone is in fastboot (the shared step); from here the watching parent knows the phone was touched.
    pub fn phone_is_in_fastboot(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || {
            crate::watch::phone_is_touched();
            lab.phone_is_in_fastboot()
        }
    }

    /// The shell answers by itself and the image has the delay: `/display-delay` holds the number and the init is the delay init.
    pub fn shell_answers_and_the_image_has_its_delay(
        &self,
        delay: u64,
    ) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || {
            let mut sh = d.lab.shell_when_up()?;
            let said = sh.run("cat /display-delay", Duration::from_secs(10))?;
            if said.trim() != delay.to_string() {
                return Err(format!(
                    "the image says its delay is {:?}, not {delay}: it is not {}",
                    said.trim(),
                    d.image(delay)?.name
                ));
            }
            Ok(())
        }
    }

    /// The init has loaded the display modules (or `device_wait` is over), the kernel log is read and scored; the row is kept and the
    /// table so far is saved. The log is kept whole (the serial redacted) in the lab's material. A load that did not finish is a row, not a stop.
    pub fn modules_load_and_the_signs_are_scored(
        &self,
        delay: u64,
    ) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || {
            let mut sh = d.lab.shell(10)?;
            let until = Instant::now() + Duration::from_secs(d.s.device_wait);
            // the signs come after the init says it is through the list (the first faults 30 ms, the DSI timeouts 0.6 s later):
            // read again every 2 s until two reads in a row count the same (and, on an image whose init samples registers, until it says it is done)
            let (dmesg, row) = {
                let mut last: Option<Row> = None;
                loop {
                    let dmesg = redact(&sh.run("dmesg", Duration::from_secs(20))?);
                    let row = score(delay, d.s.modules as usize, &dmesg);
                    let settled =
                        row.finished && last.as_ref() == Some(&row) && !registers_pending(&dmesg);
                    if settled || Instant::now() >= until {
                        break (dmesg, row);
                    }
                    last = Some(row);
                    std::thread::sleep(Duration::from_secs(2));
                }
            };
            let name = d.image(delay)?.name;
            d.lab.save(&format!("dmesg-d{delay}.txt"), &dmesg)?;
            if !register_values(&dmesg).is_empty() {
                d.lab
                    .save(&format!("regs-d{delay}.txt"), &register_table(&dmesg))?;
            }
            let reached = row.reached.ok_or(format!(
                "the init never said 'delay {delay} s reached': the image's init is not the delay init"
            ))?;
            if reached < delay {
                return Err(format!(
                    "the init said it reached the delay of {delay} s at uptime {reached} s"
                ));
            }
            println!(
                "  {name}: loaded at {reached} s; SMMU faults {}, rcg warnings {}, DSI -110 {}; {} of {} modules ok{}",
                row.smmu_faults,
                row.rcg_warnings,
                row.dsi_110,
                row.modules_ok,
                row.modules_want,
                if row.loaded() { "" } else { " (NOT loaded: the init did not finish the list with every module ok)" }
            );
            let mut rows = d.rows.borrow_mut();
            rows.push(row);
            d.lab.save("table.txt", &table(&rows))
        }
    }

    /// The result table is said and kept in a file of the lab's material: delay, faults, warnings, -110, and where the signs stop.
    pub fn the_table_is_written(&self) -> impl FnOnce() -> Done + 'static {
        let d = self.clone();
        move || {
            let rows = d.rows.borrow();
            if rows.len() != d.s.delays.len() {
                return Err(format!(
                    "{} rows for {} delays",
                    rows.len(),
                    d.s.delays.len()
                ));
            }
            let text = format!("{}{}\n", table(&rows), summary(&rows));
            d.lab.save("table.txt", &text)?;
            for l in text.lines() {
                println!("  {l}");
            }
            println!("  kept in {}", d.s.out.join("table.txt").display());
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Try;

    const SMMU: &str = "[    2.5] arm-smmu cd00000.iommu: Unhandled context fault: fsr=0x402, iova=0x9d6f1500, fsynr=0x1, cbfrsynra=0x0, cb=0";
    const RCG: &str = "[    2.9] pclk0_clk_src: rcg didn't update its configuration.";
    const DSI: &str = "[    2.6] msm_dsi ae94000.dsi.0: [drm:dsi_cmds2buf_tx] *ERROR* cmd dma tx failed, type=0x5, data0=0x28, len=4, ret=-110";
    const BACKLIGHT_OTHER: &str = "[   40.0] walleye-display: insmod /lib/modules/7.3.0-110/kernel/drivers/gpu/drm/msm/msm.ko";

    fn log(delay: u64, reached: u64, signs: &[&str], ok: usize, done: bool) -> String {
        let mut out = String::from("[    1.0] kernel: boot\n");
        out.push_str(&format!(
            "[{reached}.0] walleye-display: delay {delay} s reached at uptime {reached} s\n"
        ));
        for s in signs {
            out.push_str(s);
            out.push('\n');
        }
        for _ in 0..ok {
            out.push_str("[   9.0] walleye-display: ok\n");
        }
        if done {
            out.push_str("[   9.9] walleye-banner: step 12: display modules: done\n");
        }
        out
    }

    #[test]
    fn the_three_signs_are_counted_by_their_own_words() {
        let r = score(
            5,
            13,
            &log(5, 5, &[SMMU, SMMU, RCG, DSI, DSI, DSI], 13, true),
        );
        assert_eq!((r.smmu_faults, r.rcg_warnings, r.dsi_110), (2, 1, 3));
        assert_eq!((r.modules_ok, r.finished, r.reached), (13, true, Some(5)));
        assert!(!r.is_clean());
    }

    #[test]
    fn a_log_without_the_signs_is_clean_and_a_minus_110_off_the_display_path_is_not_a_sign() {
        let r = score(50, 13, &log(50, 51, &[BACKLIGHT_OTHER], 13, true));
        assert_eq!((r.smmu_faults, r.rcg_warnings, r.dsi_110), (0, 0, 0));
        assert!(r.is_clean());
        assert_eq!(r.reached, Some(51));
    }

    #[test]
    fn the_delay_the_init_said_is_the_one_asked_for() {
        let l = log(15, 16, &[], 0, false);
        assert_eq!(reached_at(&l, 15), Some(16));
        assert_eq!(reached_at(&l, 5), None);
        assert_eq!(reached_at("[1] nothing", 15), None);
        let r = score(15, 13, &l);
        assert!(!r.finished);
    }

    #[test]
    fn a_load_where_a_module_failed_is_not_a_clean_row_and_never_the_flip() {
        let l = format!(
            "{}[ 9.5] walleye-display: FAILED rc=1\n",
            log(25, 25, &[], 3, true)
        );
        let r = score(25, 13, &l);
        assert!(r.is_clean() && !r.loaded());
        assert_eq!(flip_point(std::slice::from_ref(&r)), None);
        assert!(table(&[r]).contains("clean-but-not-loaded"));
    }

    fn rows(spec: &[(u64, usize, bool)]) -> Vec<Row> {
        spec.iter()
            .map(|&(delay, faults, finished)| Row {
                delay,
                reached: Some(delay),
                smmu_faults: faults,
                rcg_warnings: 0,
                dsi_110: 0,
                modules_ok: 13,
                modules_want: 13,
                finished,
            })
            .collect()
    }

    #[test]
    fn the_flip_point_is_the_first_clean_delay_when_every_later_one_is_clean_too() {
        let r = rows(&[
            (5, 10, true),
            (15, 10, true),
            (25, 0, true),
            (35, 0, true),
            (50, 0, true),
        ]);
        assert_eq!(flip_point(&r), Some(25));
        assert!(summary(&r).contains("25 s"));
    }

    #[test]
    fn no_single_flip_point_is_said_when_a_clean_row_is_followed_by_signs() {
        let r = rows(&[(5, 10, true), (15, 0, true), (25, 4, true)]);
        assert_eq!(flip_point(&r), None);
        assert!(summary(&r).contains("no single flip point"));
    }

    #[test]
    fn all_clean_is_said_as_such_and_an_unfinished_load_is_not_a_flip() {
        let all = rows(&[(5, 0, true), (15, 0, true)]);
        assert!(summary(&all).contains("no delay shows a sign"));
        let unfinished = rows(&[(5, 3, true), (15, 0, false)]);
        assert_eq!(flip_point(&unfinished), None);
        assert!(table(&unfinished).contains("clean-but-not-loaded"));
    }

    const REGS_LOG: &str = "[    4.1] walleye-regs: sample 4 s at uptime 4 s
[    4.1] walleye-regs: read CTL0_TOP 0x0c902014
[    4.1] walleye-regs: T=4 CTL0_TOP 0x0c902014 0x00000001
[    4.2] walleye-regs: T=4 INTF1_STATUS 0x0c96ba6c 0x00000001
[   10.1] walleye-regs: T=10 CTL0_TOP 0x0c902014 0x00000001
[   10.2] walleye-regs: T=10 INTF1_STATUS 0x0c96ba6c 0x00000001
[   30.1] walleye-regs: T=30 CTL0_TOP 0x0c902014 0x00000001
[   30.2] walleye-regs: T=30 INTF1_STATUS 0x0c96ba6c 0x00000000
[   30.3] walleye-regs: T=30 PP0_AUTOREFRESH_CONFIG 0x0c971030 FAILED devmem: mmap: Operation not permitted
";

    #[test]
    fn the_registers_are_read_off_the_log_by_sample_and_a_failed_read_is_a_value() {
        let v = register_values(REGS_LOG);
        assert_eq!(v.len(), 7);
        assert_eq!(
            v[0],
            (
                "CTL0_TOP".into(),
                "0x0c902014".into(),
                4,
                "0x00000001".into()
            )
        );
        assert_eq!(v[6].3, "FAILED");
        assert!(register_values("[ 1.0] walleye-regs: read CTL0_TOP 0x0c902014\n").is_empty());
    }

    #[test]
    fn the_register_table_names_the_registers_that_change_between_samples() {
        let t = register_table(REGS_LOG);
        let lines: Vec<&str> = t.lines().collect();
        assert_eq!(lines[0], "register address at_4s at_10s at_30s verdict");
        assert_eq!(
            lines[1],
            "CTL0_TOP 0x0c902014 0x00000001 0x00000001 0x00000001 same"
        );
        assert_eq!(
            lines[2],
            "INTF1_STATUS 0x0c96ba6c 0x00000001 0x00000001 0x00000000 changes"
        );
        assert_eq!(
            lines[3],
            "PP0_AUTOREFRESH_CONFIG 0x0c971030 - - FAILED single"
        );
        assert_eq!(lines.len(), 4);
    }

    #[test]
    fn a_table_of_a_log_that_never_said_done_says_it_is_incomplete() {
        let t = regs_table_text("[ 4.0] walleye-regs: sample 4 s at uptime 4 s\n[ 4.1] walleye-regs: T=4 CTL0_TOP 0x0c902014 0x1\n");
        assert!(
            t.starts_with("incomplete: registers: done not seen\nregister address at_4s verdict\n")
        );
        assert!(t.contains("CTL0_TOP 0x0c902014 0x1 single"));
        assert!(!regs_table_text(&format!(
            "{REGS_LOG}[ 31.0] walleye-banner: step 14: registers: done\n"
        ))
        .starts_with("incomplete"));
    }

    #[test]
    fn the_log_is_read_again_while_the_init_samples_registers_and_never_for_an_image_without_them()
    {
        assert!(registers_pending(
            "[ 4.0] walleye-regs: sample 4 s at uptime 4 s\n"
        ));
        assert!(!registers_pending(&format!(
            "{REGS_LOG}[ 31.0] walleye-banner: step 14: registers: done\n"
        )));
        assert!(!registers_pending(
            "[ 9.9] walleye-banner: step 12: display modules: done\n"
        ));
    }

    #[test]
    fn the_table_has_a_header_and_a_line_per_delay() {
        let t = table(&rows(&[(5, 10, true), (50, 0, true)]));
        let lines: Vec<&str> = t.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with("delay_s reached_s smmu_faults rcg_warnings dsi_110"));
        assert_eq!(lines[1], "5 5 10 0 0 13 yes signs");
        assert_eq!(lines[2], "50 50 0 0 0 13 yes clean");
    }

    #[test]
    fn an_image_prefix_names_the_splash_image_at_five_seconds() {
        let base = |k: &str| match k {
            "images.dir" => "images".to_string(),
            "images.b-s5.path" => "boot-b-s5.img".into(),
            "images.b-s5.sha256" => "5b5b".into(),
            "boot.device_wait" | "boot.back_wait" | "health.battery_floor_mv" => "5".into(),
            "connection.shell_port" => "23".into(),
            "delays" => "5".into(),
            "image_prefix" => "b-s".into(),
            "modules" => "13".into(),
            _ => "x".into(),
        };
        let s = Settings::read(Try::DisplayDelay, &base, &base);
        assert_eq!(s.delay_images.len(), 1);
        assert_eq!(s.delay_images[0].name, "b-s5");
        assert_eq!(s.delay_images[0].path, s.lab.join("images/boot-b-s5.img"));
        assert_eq!(s.delay_images[0].sha256, "5b5b");
        let d = Settings::stand_in(Try::DisplayDelay);
        assert_eq!(d.delay_images[0].name, "b-d5");
    }

    #[test]
    fn the_settings_name_an_image_for_every_delay() {
        let s = Settings::stand_in(Try::DisplayDelay);
        assert_eq!(s.delays, vec![5, 15, 25, 35, 50]);
        assert_eq!(s.delay_images.len(), 5);
        assert_eq!(s.delay_images[1].path, s.lab.join("images/boot-b-d15.img"));
        assert_eq!(s.delay_images[4].sha256, "d50d");
        let d = DisplayDelay::new(s);
        assert!(d.image(25).is_ok());
        assert!(d.image(7).unwrap_err().contains("7 s"));
    }

    #[test]
    fn the_table_step_refuses_a_table_with_missing_rows() {
        let mut s = Settings::stand_in(Try::DisplayDelay);
        s.out = std::env::temp_dir().join(format!("wt-delay-table-{}", std::process::id()));
        let d = DisplayDelay::new(s);
        let e = (d.the_table_is_written())().unwrap_err();
        assert!(e.contains("0 rows for 5 delays"), "{e}");
        for n in d.delays() {
            d.rows
                .borrow_mut()
                .push(score(n, 13, &log(n, n, &[], 13, true)));
        }
        (d.the_table_is_written())().unwrap();
        let text = std::fs::read_to_string(d.s.out.join("table.txt")).unwrap();
        assert!(text.contains("no delay shows a sign"), "{text}");
        let _ = std::fs::remove_dir_all(&d.s.out);
    }

    /// The steps against a stand-in phone: the table answers by what the command contains, first row first.
    mod walkthrough {
        use super::*;
        use crate::phone::fake;

        fn out(name: &str) -> std::path::PathBuf {
            std::env::temp_dir().join(format!("wt-display-delay-{name}-{}", std::process::id()))
        }

        fn leak(s: String) -> &'static str {
            Box::leak(s.into_boxed_str())
        }

        fn bisect(table: Vec<(&'static str, &'static str)>, name: &str) -> DisplayDelay {
            let mut s = Settings::stand_in(Try::DisplayDelay);
            s.port = fake::phone(table, vec![]);
            s.out = out(name);
            s.device_wait = 2;
            DisplayDelay::new(s)
        }

        #[test]
        fn a_load_with_the_signs_is_a_row_and_the_log_is_kept() {
            let dmesg = log(5, 5, &[SMMU, RCG, DSI], 13, true);
            let d = bisect(vec![("dmesg", leak(dmesg)), ("echo up-", "up-42")], "signs");
            (d.modules_load_and_the_signs_are_scored(5))().unwrap();
            let rows = d.rows.borrow();
            assert_eq!(rows.len(), 1);
            assert_eq!(
                (rows[0].smmu_faults, rows[0].rcg_warnings, rows[0].dsi_110),
                (1, 1, 1)
            );
            let kept = std::fs::read_to_string(out("signs").join("table.txt")).unwrap();
            assert!(kept.contains("5 5 1 1 1 13 yes signs"), "{kept}");
            assert!(out("signs").join("dmesg-d5.txt").exists());
            let _ = std::fs::remove_dir_all(out("signs"));
        }

        #[test]
        fn an_unfinished_load_is_a_row_not_a_stop() {
            let dmesg = log(15, 16, &[SMMU], 4, false);
            let d = bisect(vec![("dmesg", leak(dmesg))], "unfinished");
            (d.modules_load_and_the_signs_are_scored(15))().unwrap();
            assert!(!d.rows.borrow()[0].finished);
            let _ = std::fs::remove_dir_all(out("unfinished"));
        }

        #[test]
        fn an_image_whose_init_is_not_the_delay_init_stops_the_run() {
            let dmesg = "[ 5.0] walleye-display: ok\n[ 9.9] walleye-banner: step 12: display modules: done\n".to_string();
            let d = bisect(vec![("dmesg", leak(dmesg))], "plain");
            let e = (d.modules_load_and_the_signs_are_scored(5))().unwrap_err();
            assert!(e.contains("not the delay init"), "{e}");
            let _ = std::fs::remove_dir_all(out("plain"));
        }

        #[test]
        fn a_load_before_its_delay_stops_the_run() {
            let dmesg = log(25, 12, &[], 13, true);
            let d = bisect(vec![("dmesg", leak(dmesg))], "early");
            let e = (d.modules_load_and_the_signs_are_scored(25))().unwrap_err();
            assert!(e.contains("reached the delay of 25 s at uptime 12"), "{e}");
            let _ = std::fs::remove_dir_all(out("early"));
        }

        #[test]
        fn the_shell_says_its_delay_and_another_image_is_named() {
            let right = bisect(
                vec![("cat /display-delay", "35"), ("echo up-", "up-42")],
                "right",
            );
            (right.shell_answers_and_the_image_has_its_delay(35))().unwrap();
            let e = (right.shell_answers_and_the_image_has_its_delay(5))().unwrap_err();
            assert!(e.contains("is not b-d5"), "{e}");
        }

        #[test]
        fn the_five_in_a_row_make_the_table_with_the_flip_point() {
            let d = bisect(vec![], "row");
            for (n, signs) in [(5, 3), (15, 3), (25, 0), (35, 0), (50, 0)] {
                let found: Vec<&str> = [SMMU; 3][..signs].to_vec();
                d.rows
                    .borrow_mut()
                    .push(score(n, 13, &log(n, n, &found, 13, true)));
            }
            (d.the_table_is_written())().unwrap();
            let text = std::fs::read_to_string(out("row").join("table.txt")).unwrap();
            assert!(text.contains("the signs stop at a delay of 25 s"), "{text}");
            assert_eq!(text.lines().filter(|l| l.ends_with(" signs")).count(), 2);
            let _ = std::fs::remove_dir_all(out("row"));
        }
    }
}
