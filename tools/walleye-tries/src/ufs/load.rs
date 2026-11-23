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

    /// The bounded read-only read of the disk, the gear and the clock sampled while it runs (at least one sample must fall inside it).
    /// The read's state is asked on a shell of its own, so a gear read that waits cannot hide the read's end; a gear read that is
    /// not answered is written down with its time and the sampling goes on on a fresh shell. The samples are saved however the step ends.
    pub fn the_gear_is_read_under_load(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let start = load_start_command(&u.s.lun, u.s.load_mb, u.s.load_limit)?;
            let mut state_sh = u.lab.shell(10)?;
            let began = Instant::now();
            if !state_sh
                .run(&start, Duration::from_secs(10))?
                .contains("load-started-42")
            {
                return Err("the read did not start".into());
            }
            let mut samples = String::new();
            let sampled = u.sample_until_the_read_ends(&mut state_sh, began, &mut samples);
            u.lab.save("gear-load.txt", &redact(&samples))?;
            let (rc, inside) = sampled?;
            let out = state_sh.run("cat /tmp/load.out", Duration::from_secs(10))?;
            u.lab.save("load.txt", &out)?;
            println!("  read: {}", out.lines().last().unwrap_or(""));
            match rc {
                0 | 124 if inside > 0 => Ok(()),
                0 | 124 => Err(
                    "the read ended before the gear could be read under load: not known yet".into(),
                ),
                other => Err(format!("the read ended with {other}: {out}")),
            }
        }
    }

    /// Ask the read's state on its own shell until it ends (at most its limit and 15 s, checked between samples; a gear read in
    /// flight may run on to `gear_wait` after that), read the gear in between. Returns the exit code and the answered gear reads.
    fn sample_until_the_read_ends(
        &self,
        state_sh: &mut Shell,
        began: Instant,
        samples: &mut String,
    ) -> Result<(i32, u32), String> {
        let until = began + Duration::from_secs(self.s.load_limit + 15);
        let mut gear_sh = self.lab.shell(10)?;
        let (mut inside, mut unanswered) = (0, 0);
        let rc = loop {
            if Instant::now() >= until {
                return Err("the read did not end within its time limit and 15 s".into());
            }
            std::thread::sleep(Duration::from_millis(500));
            if let Some(rc) = load_rc(&state_sh.run(load_state_command(), Duration::from_secs(10))?)
            {
                break rc;
            }
            let (at, asked) = (began.elapsed(), Instant::now());
            match gear_sh.run(&gear_command(), Duration::from_secs(self.s.gear_wait)) {
                Ok(said) => {
                    inside += 1;
                    println!("  under load: {}", Gear::of(&said).line());
                    samples.push_str(&sample_line(at, asked.elapsed(), Some(&said)));
                }
                Err(e) => {
                    unanswered += 1;
                    println!(
                        "  under load: a gear read was not answered in {} s (not known yet): {e}",
                        self.s.gear_wait
                    );
                    samples.push_str(&sample_line(at, asked.elapsed(), None));
                    gear_sh = self.lab.shell(10)?;
                }
            }
        };
        samples.push_str(&format!(
            "--- the read ended at {:.1} s with rc={rc}; {inside} gear reads answered, {unanswered} not\n",
            began.elapsed().as_secs_f32()
        ));
        Ok((rc, inside))
    }

    /// The gear and the clock again, 2 s after the read ended.
    pub fn the_gear_is_read_two_seconds_after(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
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
        assert!(out("load").join("load.txt").exists());
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
        let e = ufs(t, "silent").the_gear_is_read_under_load()().unwrap_err();
        assert!(e.contains("not known yet"), "{e}");
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
            t.contains("6 disks") && t.contains("sda for at most 4 MiB or 20 seconds"),
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
