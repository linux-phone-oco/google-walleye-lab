// SPDX-License-Identifier: MIT OR Apache-2.0
//! The UFS try's steps (item walleye-attend-program-ufs; decisions 2026-11-19-walleye-attended-tries-one-command,
//! 2026-11-19-walleye-ufs-hold-second-path): the matched UFS-on image booted once from the phone's fastboot, from
//! memory, and only when the author's go is recorded on the item the manifest names; then the kernel log and the
//! card read, READ ONLY, and the way back. Each method is one step of procedures/ufs-matched-image-on-the-pixel-2.rs,
//! returns what that step runs and fails with what it saw.
//!
//! The UFS-off gate (tests/host/ufs-off-image.sh) is not weakened and not run on this image: it refuses it by design.
//! The one step that boots it, [`Ufs::boot_matched_image`], names it, and checks instead its sha256 and
//! tests/host/ufs-match-android.sh. NOTHING is written to the card: no mount, no dd, no fstrim, no reset_controller.

use crate::lab::Lab;
use crate::settings::Settings;
use crate::touch::redact;
use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

type Done = Result<(), String>;

pub mod load;

/// What the matched-image tests print last when every check is ok.
pub const MATCH_OK: &str = "ufs-match-android: all checks ok";

/// What a kernel log line has to say, in the order the kernel says it.
pub const VERSION: &str = "UFS_HW_VERSION";
pub const HOST_LEFT: &str = "UFS_HOST_LEFT HCS";
pub const PHY_TABLE: &str = "MSM8998 PHY table";
pub const PHY_LINECFG: &str = "UFS_PHY_LINECFG_DISABLE";

/// What the author is told before the step that boots the UFS-on image (the go note has the rest).
/// What the author is told before the step that boots the UFS-on image: the numbers are the settings', never typed twice.
pub fn expect(s: &Settings) -> String {
    format!("One command, about 5 minutes: the phone boots the matched UFS-on image once, from memory. It reads the card (its {} disks and their partition tables), first pins the UFS host's devfreq clock at its lowest, 50 MHz (the userspace governor) and reads it back, then reads {} in steps of {} MiB (at most {} seconds each), read only, a line in its RAM log before each step and the shell polled every second, while the link gear and the clock are noted; a step starts only when it ends before the image's own reboot (180 s after its start, read from the kernel log); the first silent step is a result, the ramp stops there; the disk's partition table is hashed before and after. NOTHING is written to the card. The glass stays dark. The phone returns to Android by itself and visits fastboot once more to read the slot flags; if it is not on Android after 4 minutes, hold the power button about 15 seconds.", s.luns, s.lun, s.ramp_mb.iter().map(u64::to_string).collect::<Vec<_>>().join(", "), s.load_limit)
}

/// Is the text the strict shape `ikb backlog-go` writes (`date -Is`: YYYY-MM-DDTHH:MM:SS+HH:MM or Z)? Nothing relative
/// ("tomorrow", "+1 hour", "2027", "@N") is a go date: `date -d` would read those as a time that never runs out.
pub fn is_strict_date(t: &str) -> bool {
    let b = t.as_bytes();
    let digit = |i: usize| b.get(i).is_some_and(u8::is_ascii_digit);
    let at = |i: usize, c: u8| b.get(i) == Some(&c);
    let head = [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18]
        .iter()
        .all(|&i| digit(i))
        && at(4, b'-')
        && at(7, b'-')
        && at(10, b'T')
        && at(13, b':')
        && at(16, b':');
    let zone = match b.len() {
        20 => at(19, b'Z'),
        25 => {
            (at(19, b'+') || at(19, b'-'))
                && digit(20)
                && digit(21)
                && at(22, b':')
                && digit(23)
                && digit(24)
        }
        _ => false,
    };
    head && zone
}

/// The epoch second of a strict date text; None when it is not one or reads none.
pub fn epoch_of(text: &str) -> Option<i64> {
    let t = text.trim();
    if !is_strict_date(t) {
        return None;
    }
    date_epoch(t)
}

/// The epoch second now.
pub fn now_epoch() -> Option<i64> {
    date_epoch("now")
}

fn date_epoch(text: &str) -> Option<i64> {
    let o = std::process::Command::new("date")
        .args(["-d", text, "+%s"])
        .output()
        .ok()?;
    if !o.status.success() {
        return None;
    }
    String::from_utf8_lossy(&o.stdout).trim().parse().ok()
}

/// The go covers THIS image: the item's row names the image's sha256, and its go_until is a strict date in the future.
/// A go given for another image (the row names another sha256) opens nothing.
pub fn go_decision(row: &str, now: i64, sha256: &str) -> Result<(), String> {
    let sha = sha256.trim().to_lowercase();
    if sha.len() != 64 || !row.to_lowercase().contains(&sha) {
        return Err(format!(
            "no go: the item does not name this image's sha256 {sha} (the go covers the image its note names)"
        ));
    }
    go_is_open(go_until_of(row).as_deref(), now)
}
/// The `go_until` of an item row as `tad q row --id ID --format csv` (JSON with doubled quotes) or plain JSON prints it:
/// the date text, None when it is null, missing or not a string.
pub fn go_until_of(row: &str) -> Option<String> {
    let key = "\"go_until\":";
    let unquoted = row.replace("\"\"", "\"");
    // plain JSON has the key as it is; the csv's doubled quotes are undone only when it has not
    let json = if row.contains(key) { row } else { &unquoted };
    let at = json.find(key)? + key.len();
    let rest = json[at..].trim_start().strip_prefix('"')?;
    let value = rest.split('"').next()?.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

/// Is the author's go open at `now`? Only a date that reads and lies in the future is a go; anything else is none.
pub fn go_is_open(go_until: Option<&str>, now: i64) -> Result<(), String> {
    let text = go_until.ok_or("no go: the item has no go_until, the author has not said go")?;
    let until = epoch_of(text).ok_or_else(|| format!("no go: go_until {text:?} is not a date"))?;
    if until > now {
        Ok(())
    } else {
        Err(format!(
            "no go: the author's go ran out at {text} (ikb backlog-go opens it again)"
        ))
    }
}

/// The text of a line of `dmesg` after a marker, None when no line has it.
pub fn line_with<'a>(dmesg: &'a str, marker: &str) -> Option<&'a str> {
    dmesg.lines().find(|l| l.contains(marker))
}

/// What the kernel log of the matched image says of the UFS probe, whole or not yet.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Probe {
    pub version: Option<String>,
    pub host_left: Option<String>,
    pub phy_table: Option<String>,
    pub linecfg: Option<String>,
    /// lines that say the probe failed or a purge was met
    pub trouble: Vec<String>,
}

impl Probe {
    pub fn of(dmesg: &str) -> Probe {
        let of = |m| line_with(dmesg, m).map(|l| l.trim().to_string());
        Probe {
            version: of(VERSION),
            host_left: of(HOST_LEFT),
            phy_table: of(PHY_TABLE),
            linecfg: of(PHY_LINECFG),
            trouble: dmesg
                .lines()
                .filter(|l| {
                    l.contains("purge refused")
                        || l.contains("unknown UFS host version")
                        || l.contains("phy initialization timed-out")
                        || l.contains("link startup failed")
                        || l.contains("Oops")
                        || l.contains("Kernel panic")
                        || l.contains("I/O error")
                })
                .map(|l| l.trim().to_string())
                .collect(),
        }
    }

    /// The names of the lines the log does not have yet.
    pub fn missing(&self) -> Vec<&'static str> {
        let mut m = vec![];
        for (have, name) in [
            (&self.version, VERSION),
            (&self.host_left, HOST_LEFT),
            (&self.phy_table, PHY_TABLE),
            (&self.linecfg, PHY_LINECFG),
        ] {
            if have.is_none() {
                m.push(name);
            }
        }
        m
    }
}

/// The `power_info` files of the host, read as `name=value` lines, as the link of the card.
/// The values are as the kernel prints them (`HS_GEAR3`, `HS_RATE_B`, `FAST_MODE`, `HIBERN8`); a bare digit is read too.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Link {
    pub gear: String,
    pub lane: String,
    pub mode: String,
    pub rate: String,
    pub link_state: String,
    pub dev_pm: String,
}

/// What the link reading says: the link at its best, up but below it, or not up.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Speed {
    /// High speed, gear 3, two lanes.
    Full,
    /// The link is up and the card answers, in a lower gear, on fewer lanes or in a slow mode.
    Slow,
    /// No link: the host reports it off, or reports nothing.
    Down,
}

impl Link {
    pub fn of(lines: &str) -> Link {
        let get = |key: &str| {
            lines
                .lines()
                .find_map(|l| l.trim().strip_prefix(&format!("{key}=")))
                .unwrap_or("")
                .trim()
                .to_string()
        };
        Link {
            gear: get("gear"),
            lane: get("lane"),
            mode: get("mode"),
            rate: get("rate"),
            link_state: get("link_state"),
            dev_pm: get("dev_pm"),
        }
    }

    /// The gear as a number: `3` and `HS_GEAR3` both say 3.
    fn gear_number(&self) -> &str {
        self.gear.trim_start_matches(|c: char| !c.is_ascii_digit())
    }

    /// The link is up: it has a mode, a lane count and the state ACTIVE or HIBERN8 (idle is up; OFF, BROKEN and the rest are down).
    pub fn is_up(&self) -> bool {
        !self.mode.is_empty()
            && !self.lane.is_empty()
            && !self.link_state.is_empty()
            && (self.link_state == "ACTIVE" || self.link_state == "HIBERN8")
    }

    pub fn speed(&self) -> Speed {
        if !self.is_up() {
            Speed::Down
        } else if self.gear_number() == "3" && self.lane == "2" && self.mode.starts_with("FAST") {
            Speed::Full
        } else {
            Speed::Slow
        }
    }

    /// One line for the log: every reading the card's link gives.
    pub fn line(&self) -> String {
        format!(
            "gear {} lanes {} rate {} mode {} link_state {} dev_pm {}",
            self.gear, self.lane, self.rate, self.mode, self.link_state, self.dev_pm
        )
    }
}

/// The names of the card's disks in `/proc/partitions` (`sda`, `sdb`, ... ), without their partitions.
pub fn luns_of(partitions: &str) -> Vec<String> {
    partitions
        .lines()
        .filter_map(|l| l.split_whitespace().nth(3))
        .filter(|n| {
            n.strip_prefix("sd")
                .is_some_and(|r| r.len() == 1 && r.chars().all(|c| c.is_ascii_lowercase()))
        })
        .map(String::from)
        .collect()
}

/// The line the matched image's init (build/initramfs-mainline/init-z14) writes to the kernel log just before it starts the
/// watchdog petter, whose 90 beats of 2 s end in `reboot -f` (research walleye-ufs-180s-is-the-init-self-reboot).
pub const PETTER_LINE: &str = "about to start the watchdog petter";

/// The init's reboot comes this many seconds after the petter starts: 90 beats of 2 s (init-z14 lines 26-34).
pub const PETTER_REBOOT_S: f64 = 180.0;

/// A ramp step ends at least this many seconds before the reboot (decision 2026-11-24-walleye-ufs-procedure-fits-the-image-not-the-image-the-procedure, (c)).
pub const DEADLINE_MARGIN_S: f64 = 10.0;

/// The command that prints the petter's kernel-log line.
pub const PETTER_COMMAND: &str = "dmesg | grep -F 'about to start the watchdog petter'";

/// The command that prints the phone's uptime: `SECONDS IDLE_SECONDS`.
pub const UPTIME_COMMAND: &str = "cat /proc/uptime";

/// The uptime at which the petter started, from a kernel-log line (`[    1.102560] walleye-banner: step 2: about to start the watchdog petter`).
pub fn petter_started_at(dmesg: &str) -> Option<f64> {
    let line = line_with(dmesg, PETTER_LINE)?;
    line.trim_start()
        .strip_prefix('[')?
        .split(']')
        .next()?
        .trim()
        .parse()
        .ok()
}

/// The uptime at which the image's init reboots the phone: the petter's start and 180 s.
pub fn reboot_deadline_of(dmesg: &str) -> Option<f64> {
    petter_started_at(dmesg).map(|t| t + PETTER_REBOOT_S)
}

/// The phone's uptime, the first field of /proc/uptime.
pub fn uptime_of(text: &str) -> Option<f64> {
    text.split_whitespace().next()?.parse().ok()
}

/// A step that takes at most `limit` seconds, started at `uptime`, ends before the deadline less the margin; else the refusal that names it.
pub fn step_fits(uptime: f64, deadline: f64, limit: u64) -> Result<(), String> {
    let ends = uptime + limit as f64;
    if ends <= deadline - DEADLINE_MARGIN_S {
        Ok(())
    } else {
        Err(format!(
            "uptime {uptime:.1} s + its limit {limit} s ends at {ends:.1} s, past the image's reboot at {deadline:.1} s less {DEADLINE_MARGIN_S:.0} s"
        ))
    }
}

/// The command that reads the host's link, one `name=value` line each.
pub const LINK_COMMAND: &str = "for f in /sys/devices/platform/*/*.ufshc/power_info/*; do echo \"$(basename $f)=$(cat $f)\"; done";

/// The command that lists the card's disks and partitions: reads only.
pub const LIST_COMMAND: &str =
    "cat /proc/partitions; ls /sys/class/scsi_disk 2>&1; lsblk 2>&1 || true";

/// The UFS try: its settings and the lab it shares with the other tries. Cheap to clone: each step takes its own.
#[derive(Clone)]
pub struct Ufs {
    s: Rc<Settings>,
    lab: Lab,
    /// The uptime at which the image's init reboots the phone; set by the step that reads the petter's line, None until then.
    deadline: Rc<Cell<Option<f64>>>,
}

impl Ufs {
    pub fn new(s: Settings) -> Ufs {
        let s = Rc::new(s);
        Ufs {
            lab: Lab::new(s.clone()),
            s,
            deadline: Rc::new(Cell::new(None)),
        }
    }

    /// The battery is above the lab's minimum (the shared step).
    pub fn battery_is_good(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.battery_is_good()
    }

    /// The image is the matched one the manifest records (its sha256), and the tree and the dtb it was built from
    /// carry what Android's UFS path carries (tests/host/ufs-match-android.sh): the gate for this image, instead of
    /// the UFS-off gate, which refuses it by design and is not touched.
    pub fn image_is_the_matched_one(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            crate::host::image_has_sha256(&u.s.image, &u.s.image_sha256)?;
            let tree = u.s.kernel_tree.to_string_lossy().into_owned();
            let dtb = u.s.dtb.to_string_lossy().into_owned();
            let line = u.lab.script("ufs-match-android.sh", &[&tree, &dtb], &[])?;
            if line.trim() == MATCH_OK {
                Ok(())
            } else {
                Err(format!("the matched-image tests ended with: {line}"))
            }
        }
    }

    /// The phone is in fastboot (the shared step); the first act on the phone, so the watcher owns the way back.
    pub fn phone_is_in_fastboot(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || {
            crate::watch::phone_is_touched();
            lab.phone_is_in_fastboot()
        }
    }

    /// The author's go covers this image now: the item's row (`tad q row`, read from the data, not from an argument) names the
    /// image's sha256 and has `go_until` in the future.
    fn go_is_open_now(&self) -> Done {
        let mut c = std::process::Command::new("tad");
        c.args(["q", "row", "--id", &self.s.go_item, "--format", "csv"])
            .current_dir(&self.s.lab);
        let said = crate::host::run(c)?;
        if !said.ok {
            return Err(format!(
                "no go: the item {} could not be read: {}",
                self.s.go_item, said.text
            ));
        }
        let now = now_epoch().ok_or("no go: the clock could not be read")?;
        go_decision(&said.text, now, &self.s.image_sha256)
    }

    /// The author's go is open on the item. Without it the run stops here, "no go", and the way back leaves the phone on Android.
    pub fn author_has_said_go(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || u.go_is_open_now()
    }

    /// The step that names the UFS-on image and boots it from the phone's fastboot, from memory: nothing is written.
    /// It is the one place the matched image is booted, and it holds its own gate: the image's sha256 and the go are
    /// checked again here, right before the boot, whatever ran before.
    pub fn boot_matched_image(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            crate::host::image_has_sha256(&u.s.image, &u.s.image_sha256)?;
            u.go_is_open_now()?;
            u.lab.boot_image(&u.s.image)
        }
    }

    /// The shell answers (the gadget is up) and the image's reboot deadline is read: the matched image's init has no hold and reboots
    /// the phone 180 s after its petter starts (research walleye-ufs-180s-is-the-init-self-reboot), so the petter's line in the kernel log
    /// gives the deadline, kept with the uptime now in deadline.txt.
    pub fn shell_answers_and_the_deadline_is_read(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let mut sh = u.lab.shell_when_up()?;
            let petter = sh.run(PETTER_COMMAND, Duration::from_secs(10))?;
            let now = sh.run(UPTIME_COMMAND, Duration::from_secs(10))?;
            let (Some(started), Some(deadline)) =
                (petter_started_at(&petter), reboot_deadline_of(&petter))
            else {
                return Err(format!(
                    "the kernel log has no line '{PETTER_LINE}': the image's reboot deadline is not known"
                ));
            };
            let up = uptime_of(&now).ok_or("the phone's uptime could not be read")?;
            u.lab.save(
                "deadline.txt",
                &format!("petter_started_s={started}\nreboot_deadline_s={deadline}\nuptime_when_read_s={up}"),
            )?;
            u.deadline.set(Some(deadline));
            println!("  the image reboots the phone at uptime {deadline:.1} s; now {up:.1} s");
            Ok(())
        }
    }

    /// The kernel log has the probe's lines: the version, the host registers, the PHY table, the line config; none of the
    /// failure lines. Polled: the probe is asynchronous. The whole log is kept (serials redacted).
    pub fn the_probe_lines_are_read(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let until = Instant::now() + Duration::from_secs(u.s.device_wait);
            let mut sh = u.lab.shell(10)?;
            let (dm, probe) = loop {
                let dm = sh.run("dmesg", Duration::from_secs(20))?;
                let probe = Probe::of(&dm);
                if probe.missing().is_empty()
                    || !probe.trouble.is_empty()
                    || Instant::now() >= until
                {
                    break (dm, probe);
                }
                std::thread::sleep(Duration::from_secs(2));
            };
            u.lab.save("dmesg.txt", &redact(&dm))?;
            for line in [
                &probe.version,
                &probe.host_left,
                &probe.phy_table,
                &probe.linecfg,
            ]
            .into_iter()
            .flatten()
            {
                println!("  {line}");
            }
            if let Some(t) = probe.trouble.first() {
                return Err(format!("the kernel log says: {t}"));
            }
            let missing = probe.missing();
            if missing.is_empty() {
                Ok(())
            } else {
                Err(format!(
                    "the kernel log has no line for {} within {} s",
                    missing.join(", "),
                    u.s.device_wait
                ))
            }
        }
    }

    /// The link is read once, not judged by its gear and not waited on for HS-G3: an idle link reads HS_GEAR1 in HIBERN8 (run 8's
    /// link.txt), so a wait for HS-G3 spends the whole device_wait of the image's 180 s (research walleye-ufs-180s-is-the-init-self-reboot).
    /// Up at any gear is "works" (HS-G3 on two lanes "works at full speed", below it "works, slow"); a link that reads down is asked
    /// again until the wait ends (the probe may not be done) and fails only then. The reading is saved to link.txt.
    pub fn the_link_is_read(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let until = Instant::now() + Duration::from_secs(u.s.device_wait);
            let mut sh = u.lab.shell(10)?;
            loop {
                let said = sh.run(LINK_COMMAND, Duration::from_secs(10))?;
                let link = Link::of(&said);
                let speed = link.speed();
                if speed != Speed::Down || Instant::now() >= until {
                    u.lab.save("link.txt", &said)?;
                    println!("  link: {}", link.line());
                    return match speed {
                        Speed::Full => {
                            println!("  works at full speed (HS-G3, 2 lanes)");
                            Ok(())
                        }
                        Speed::Slow => {
                            println!("  works, slow (below HS-G3 on 2 lanes; the link is up)");
                            Ok(())
                        }
                        Speed::Down => Err(format!(
                            "the link is down within {} s: {}",
                            u.s.device_wait,
                            link.line()
                        )),
                    };
                }
                std::thread::sleep(Duration::from_secs(2));
            }
        }
    }

    /// The card's disks are listed (reads only: /proc/partitions, scsi_disk, lsblk; nothing mounted, nothing written).
    pub fn the_luns_are_listed(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let until = Instant::now() + Duration::from_secs(u.s.device_wait);
            let mut sh = u.lab.shell(10)?;
            loop {
                let said = sh.run(LIST_COMMAND, Duration::from_secs(20))?;
                let luns = luns_of(&said);
                if !luns.is_empty() {
                    u.lab.save("luns.txt", &redact(&said))?;
                    println!("  disks listed: {}", luns.join(" "));
                    return Ok(());
                }
                if Instant::now() >= until {
                    u.lab.save("luns.txt", &redact(&said))?;
                    return Err(format!(
                        "no disk of the card is listed within {} s",
                        u.s.device_wait
                    ));
                }
                std::thread::sleep(Duration::from_secs(2));
            }
        }
    }

    /// The kernel log has no purge: nothing asked the card to purge (the guard says `purge refused` when something did).
    /// Read again after the card was listed, with everything the log has by then; kept.
    pub fn no_purge_was_asked(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let mut sh = u.lab.shell(10)?;
            let dm = sh.run("dmesg", Duration::from_secs(20))?;
            u.lab.save("dmesg-after-list.txt", &redact(&dm))?;
            match Probe::of(&dm).trouble.first() {
                None => Ok(()),
                Some(t) => Err(format!("the kernel log says: {t}")),
            }
        }
    }

    /// Say what to expect, before the boot.
    pub fn author_is_told(&self) -> impl FnOnce() -> Done + 'static {
        let text = expect(&self.s);
        move || {
            println!("  {text}");
            Ok(())
        }
    }

    /// The phone is back on Android (the shared step).
    pub fn android_is_back(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.android_is_back()
    }

    /// The way back, the last step: the phone ends on Android whatever stopped the run (the shared step); the watching
    /// parent runs it after the child ends, whatever ended it.
    pub fn way_back(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || {
            if std::env::var_os(crate::watch::CHILD).is_some() {
                return Ok(());
            }
            lab.way_back()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DMESG: &str = "[ 3.1] ufshcd-qcom 1da4000.ufshc: UFS_HW_VERSION 3.0.0\n[ 3.1] ufshcd-qcom 1da4000.ufshc: UFS_HOST_LEFT HCS 0x00000000 HCE 0x00000000\n[ 3.2] phy-qcom-qmp-ufs 1da7000.phy: UFS host 3.0.0: MSM8998 PHY table 3.0.0\n[ 3.3] phy-qcom-qmp-ufs 1da7000.phy: UFS_PHY_LINECFG_DISABLE 0x00000000\n[ 4.0] scsi host0: ufshcd";

    #[test]
    fn the_go_is_read_from_the_row_as_csv_or_json() {
        let csv = "table,row\nbacklog,\"{\"\"id\"\":\"\"x\"\",\"\"go_until\"\":\"\"2026-11-20T08:00:00+01:00\"\",\"\"rank\"\":null}\"";
        assert_eq!(
            go_until_of(csv).as_deref(),
            Some("2026-11-20T08:00:00+01:00")
        );
        assert_eq!(
            go_until_of("{\"id\":\"x\",\"go_until\":\"2026-11-20\"}").as_deref(),
            Some("2026-11-20")
        );
        assert_eq!(go_until_of("{\"go_until\":null}"), None);
        assert_eq!(go_until_of("{\"go_until\":\"\"}"), None);
        assert_eq!(go_until_of("no row"), None);
    }

    #[test]
    fn only_a_date_in_the_future_is_a_go() {
        let now = epoch_of("2026-11-19T12:00:00+00:00").unwrap();
        assert!(go_is_open(Some("2026-11-19T13:00:00+00:00"), now).is_ok());
        let past = go_is_open(Some("2026-11-19T11:00:00+00:00"), now).unwrap_err();
        assert!(
            past.starts_with("no go") && past.contains("ran out"),
            "{past}"
        );
        assert!(go_is_open(None, now).unwrap_err().starts_with("no go"));
        let junk = go_is_open(Some("soon"), now).unwrap_err();
        assert!(
            junk.starts_with("no go") && junk.contains("not a date"),
            "{junk}"
        );
    }

    #[test]
    fn the_probe_lines_are_found_and_the_missing_ones_named() {
        let p = Probe::of(DMESG);
        assert!(p.missing().is_empty() && p.trouble.is_empty());
        assert!(p.version.unwrap().ends_with("UFS_HW_VERSION 3.0.0"));
        let half = Probe::of("[ 3.1] x: UFS_HW_VERSION 3.1.0");
        assert_eq!(half.missing(), vec![HOST_LEFT, PHY_TABLE, PHY_LINECFG]);
        assert_eq!(Probe::of("").missing().len(), 4);
    }

    #[test]
    fn a_purge_or_a_refused_phy_is_trouble() {
        let dm = format!("{DMESG}\n[ 5.0] ufshcd: ufshcd_exec_dev_cmd: purge refused on this device\n[ 5.1] phy: unknown UFS host version 9.9.9, no calibration values");
        assert_eq!(Probe::of(&dm).trouble.len(), 2);
        assert!(Probe::of(DMESG).trouble.is_empty());
    }

    #[test]
    fn the_link_is_read_from_power_info_lines() {
        let full = "gear=HS_GEAR3\nlane=2\nmode=FAST_MODE\nrate=HS_RATE_B\ndev_pm=ACTIVE\nlink_state=ACTIVE";
        assert_eq!(Link::of(full).speed(), Speed::Full);
        assert_eq!(
            Link::of(&full.replace("HS_GEAR3", "3")).speed(),
            Speed::Full
        );
        // the real files of try 1: gear 1, two lanes, idle in HIBERN8: the link is up, slow
        let try1 = "dev_pm=SLEEP\ngear=HS_GEAR1\nlane=2\nlink_state=HIBERN8\nmode=FAST_MODE\nrate=HS_RATE_B";
        assert_eq!(Link::of(try1).speed(), Speed::Slow);
        for slow in [
            "gear=HS_GEAR1\nlane=2\nmode=FAST_MODE\nlink_state=ACTIVE",
            "gear=HS_GEAR3\nlane=1\nmode=FAST_MODE\nlink_state=ACTIVE",
            "gear=HS_GEAR3\nlane=2\nmode=SLOW_MODE\nlink_state=ACTIVE",
        ] {
            assert_eq!(Link::of(slow).speed(), Speed::Slow, "{slow}");
        }
        for down in [
            "gear=HS_GEAR3\nlane=2\nmode=FAST_MODE\nlink_state=OFF",
            "gear=HS_GEAR3\nlane=2\nmode=FAST_MODE\nlink_state=BROKEN",
            "",
        ] {
            assert_eq!(Link::of(down).speed(), Speed::Down, "{down}");
        }
        assert!(Link::of(try1).line().contains("rate HS_RATE_B"));
    }

    #[test]
    fn the_deadline_is_the_petters_start_and_180_s_and_a_step_must_end_before_it() {
        // the line of the matched image (research walleye-first-fastboot-boot, c2-try1 read/dmesg.txt line 312)
        let dm = "[    1.087] x: y\n[    1.102560] walleye-banner: step 2: about to start the watchdog petter\n[    2.0] z";
        assert_eq!(petter_started_at(dm), Some(1.10256));
        assert!((reboot_deadline_of(dm).unwrap() - 181.10256).abs() < 1e-9);
        assert_eq!(reboot_deadline_of("[    1.0] nothing"), None);
        assert_eq!(uptime_of("12.34 45.67\n"), Some(12.34));
        assert_eq!(uptime_of(""), None);
        let deadline = 181.1;
        assert!(step_fits(30.0, deadline, 60).is_ok());
        assert!(
            step_fits(111.1, deadline, 60).is_ok(),
            "ends exactly at the deadline less the margin"
        );
        let e = step_fits(111.2, deadline, 60).unwrap_err();
        assert!(
            e.contains("111.2") && e.contains("60 s") && e.contains("181.1"),
            "{e}"
        );
    }

    #[test]
    fn the_luns_are_the_whole_disks_of_proc_partitions() {
        let p = "major minor  #blocks  name\n 8 0 1000 sda\n 8 1 100 sda1\n 8 16 1000 sdb\n 259 0 5 mmcblk0\n";
        assert_eq!(luns_of(p), vec!["sda", "sdb"]);
        assert!(luns_of("major minor  #blocks  name\n").is_empty());
    }

    #[test]
    fn the_commands_are_valid_shell_and_write_nothing_to_the_card() {
        for c in [
            LINK_COMMAND.to_string(),
            LIST_COMMAND.to_string(),
            PETTER_COMMAND.to_string(),
            UPTIME_COMMAND.to_string(),
        ] {
            let ok = std::process::Command::new("sh")
                .args(["-n", "-c", &c])
                .status()
                .unwrap();
            assert!(ok.success(), "{c}");
            for forbidden in [
                "dd ",
                "mount",
                "fstrim",
                "reset_controller",
                "> /dev/",
                "mkfs",
            ] {
                assert!(!c.contains(forbidden), "{forbidden} in {c}");
            }
        }
    }

    #[test]
    fn a_relative_or_bare_date_is_never_a_go() {
        let now = epoch_of("2026-11-19T12:00:00+00:00").unwrap();
        for t in [
            "tomorrow",
            "+1 hour",
            "1 day",
            "10 years",
            "next week",
            "@99999999999",
            "2027",
            "2026-11-20",
        ] {
            assert!(go_is_open(Some(t), now).is_err(), "{t}");
            assert_eq!(epoch_of(t), None, "{t}");
        }
        assert!(epoch_of("2026-11-20T08:00:00Z").is_some());
        assert!(epoch_of("2026-11-20T08:00:00-05:00").is_some());
    }

    #[test]
    fn a_go_covers_only_the_image_the_item_names() {
        let sha = "461077869fb275ab8803b8bb771050d4f19724ccfd62e197dc1f3fe3a6297df5";
        let now = epoch_of("2026-11-19T12:00:00+00:00").unwrap();
        let row = |note: &str, until: &str| {
            format!("backlog,\"{{\"\"note\"\":\"\"{note}\"\",\"\"go_until\"\":\"\"{until}\"\"}}\"")
        };
        assert!(go_decision(&row(sha, "2026-11-19T13:00:00+00:00"), now, sha).is_ok());
        let other = "f8f6215e0000000000000000000000000000000000000000000000000000004f45";
        let e = go_decision(&row(other, "2026-11-19T13:00:00+00:00"), now, sha).unwrap_err();
        assert!(e.contains("does not name this image"), "{e}");
        assert!(go_decision(&row(sha, "2026-11-19T11:00:00+00:00"), now, sha).is_err());
        assert!(go_decision(&row(sha, "2026-11-19T13:00:00+00:00"), now, "4610").is_err());
        assert!(go_decision(&row(sha, "2026-11-19T13:00:00+00:00"), now, "").is_err());
    }

    #[test]
    fn the_boot_step_checks_the_image_itself_before_any_boot() {
        use crate::settings::Try;
        let dir = std::env::temp_dir().join(format!("wt-ufs-boot-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let img = dir.join("x.img");
        std::fs::write(&img, "swapped").unwrap();
        let mut s = Settings::stand_in(Try::Ufs);
        s.image = img;
        s.lpoco = "/nonexistent/lpoco-must-not-run".into();
        let e = Ufs::new(s).boot_matched_image()().unwrap_err();
        assert!(e.contains("device.toml records"), "{e}");
    }

    /// The try against a stand-in phone: the table answers by what the command contains, first row first.
    mod walkthrough {
        use super::*;
        use crate::phone::fake;
        use crate::settings::Try;

        fn out(name: &str) -> std::path::PathBuf {
            std::env::temp_dir().join(format!("wt-ufs-{name}-{}", std::process::id()))
        }

        fn ufs(table: Vec<(&'static str, &'static str)>, name: &str) -> Ufs {
            let mut s = Settings::stand_in(Try::Ufs);
            s.port = fake::phone(table, vec![]);
            s.out = out(name);
            s.device_wait = 2;
            Ufs::new(s)
        }

        const UP: &str = "gear=3\nlane=2\nmode=FAST_MODE\nrate=B\nlink_state=ACTIVE";
        const PARTS: &str = "major minor  #blocks  name\n 8 0 1000 sda\n 8 1 100 sda1";
        /// The petter line as the matched image printed it (research/walleye-first-fastboot-boot/2026-11-19-c2-try1/read/dmesg.txt line 312).
        const PETTER: &str =
            "[    1.102560] walleye-banner: step 2: about to start the watchdog petter";

        fn phone(dmesg: &'static str) -> Vec<(&'static str, &'static str)> {
            vec![
                ("watchdog petter", PETTER),
                ("power_info", UP),
                ("/proc/partitions", PARTS),
                ("/proc/uptime", "12.34 45.67"),
                ("dmesg", dmesg),
                ("echo up-", "up-42"),
            ]
        }

        #[test]
        fn the_probe_the_link_the_disks_and_no_purge_pass_on_a_phone_that_has_them() {
            let u = ufs(phone(DMESG), "pass");
            u.shell_answers_and_the_deadline_is_read()().unwrap();
            u.the_probe_lines_are_read()().unwrap();
            u.the_link_is_read()().unwrap();
            u.the_luns_are_listed()().unwrap();
            u.no_purge_was_asked()().unwrap();
            let saved = std::fs::read_to_string(out("pass").join("dmesg.txt")).unwrap();
            assert!(saved.contains("UFS_HW_VERSION 3.0.0"));
            assert!(out("pass").join("luns.txt").exists());
            let d = std::fs::read_to_string(out("pass").join("deadline.txt")).unwrap();
            assert!(
                d.contains("petter_started_s=1.10256")
                    && d.contains("reboot_deadline_s=181.10256")
                    && d.contains("uptime_when_read_s=12.34"),
                "{d}"
            );
        }

        #[test]
        fn a_log_without_the_phy_table_line_names_it() {
            let dm = "[ 3.1] x: UFS_HW_VERSION 3.0.0\n[ 3.1] x: UFS_HOST_LEFT HCS 0x0 HCE 0x0\n[ 3.3] x: UFS_PHY_LINECFG_DISABLE 0x0";
            let u = ufs(phone(dm), "nophy");
            let e = u.the_probe_lines_are_read()().unwrap_err();
            assert!(e.contains("MSM8998 PHY table"), "{e}");
        }

        #[test]
        fn a_purge_in_the_log_stops_the_probe_step_at_once() {
            let dm = "[ 3.1] x: UFS_HW_VERSION 3.0.0\n[ 4.0] ufshcd: purge refused on this device";
            let u = ufs(phone(dm), "purge");
            let e = u.the_probe_lines_are_read()().unwrap_err();
            assert!(e.contains("purge refused"), "{e}");
            let e = u.no_purge_was_asked()().unwrap_err();
            assert!(e.contains("purge refused"), "{e}");
        }

        fn read_link(power_info: &'static str, name: &str) -> Result<String, String> {
            let mut t = phone(DMESG);
            t[1] = ("power_info", power_info);
            let u = ufs(t, name);
            u.the_link_is_read()()
                .map(|()| std::fs::read_to_string(out(name).join("link.txt")).unwrap())
        }

        #[test]
        fn a_link_in_gear_one_works_slow_and_is_recorded() {
            let said = read_link("gear=HS_GEAR1\nlane=2\nmode=FAST_MODE\nrate=HS_RATE_B\ndev_pm=SLEEP\nlink_state=HIBERN8", "gear1").unwrap();
            assert!(
                said.contains("gear=HS_GEAR1") && said.contains("link_state=HIBERN8"),
                "{said}"
            );
        }

        #[test]
        fn a_link_in_gear_three_works_at_full_speed() {
            read_link("gear=HS_GEAR3\nlane=2\nmode=FAST_MODE\nrate=HS_RATE_B\ndev_pm=ACTIVE\nlink_state=ACTIVE", "gear3").unwrap();
        }

        #[test]
        fn a_link_that_is_down_fails_and_says_so() {
            let e = read_link(
                "gear=HS_GEAR1\nlane=2\nmode=FAST_MODE\nlink_state=OFF",
                "down",
            )
            .unwrap_err();
            assert!(
                e.contains("link is down") && e.contains("link_state OFF"),
                "{e}"
            );
        }

        #[test]
        fn a_card_with_no_disk_is_no_pass() {
            let mut t = phone(DMESG);
            t[2] = ("/proc/partitions", "major minor  #blocks  name\n");
            let u = ufs(t, "nolun");
            let e = u.the_luns_are_listed()().unwrap_err();
            assert!(e.contains("no disk of the card"), "{e}");
        }

        #[test]
        fn a_log_without_the_petter_line_names_the_unknown_deadline() {
            let mut t = phone(DMESG);
            t[0] = ("watchdog petter", "");
            let u = ufs(t, "nopetter");
            let e = u.shell_answers_and_the_deadline_is_read()().unwrap_err();
            assert!(
                e.contains("about to start the watchdog petter") && e.contains("not known"),
                "{e}"
            );
        }

        /// Proof of item walleye-ufs-procedure-fits-the-180s (was failing before it; research walleye-ufs-180s-is-the-init-self-reboot):
        /// the matched image's init (z14's, byte for byte) reboots the phone 90 beats of 2 s after it starts and knows no
        /// hold, and an idle link reads HS_GEAR1 in HIBERN8 (run 8's own link.txt), so waiting for HS-G3 at idle spends the
        /// whole device_wait (170 s in device.toml) and leaves the load about 4 s before the reboot. The link at idle is
        /// read, not waited on: with the manifest's device_wait, the read of an idle slow link ends within 15 s.
        #[test]
        fn an_idle_slow_link_is_read_without_spending_the_reboot_budget() {
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let mut t = phone(DMESG);
                t[1] = ("power_info", "dev_pm=SLEEP\ngear=HS_GEAR1\nlane=2\nlink_state=HIBERN8\nmode=FAST_MODE\nrate=HS_RATE_B");
                let mut s = Settings::stand_in(Try::Ufs);
                s.port = fake::phone(t, vec![]);
                s.out = out("idle");
                s.device_wait = 170; // the manifest's value, not the walkthrough's 2
                let _ = tx.send(Ufs::new(s).the_link_is_read()());
            });
            let r = rx.recv_timeout(std::time::Duration::from_secs(15));
            assert!(
                matches!(r, Ok(Ok(()))),
                "the idle link read did not end within 15 s ({r:?}): it would leave the load seconds before the init's 180 s self-reboot"
            );
        }
    }

    /// The matched-image step refuses an image whose sha256 is not the recorded one before it runs anything.
    #[test]
    fn an_image_that_is_not_the_recorded_one_is_refused() {
        use crate::settings::Try;
        let dir = std::env::temp_dir().join(format!("wt-ufs-img-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let img = dir.join("x.img");
        std::fs::write(&img, "not the image").unwrap();
        let mut s = Settings::stand_in(Try::Ufs);
        s.image = img;
        let e = Ufs::new(s).image_is_the_matched_one()().unwrap_err();
        assert!(e.contains("device.toml records"), "{e}");
    }
}
