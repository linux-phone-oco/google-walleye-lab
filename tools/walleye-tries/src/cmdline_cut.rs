// SPDX-License-Identifier: MIT OR Apache-2.0
//! Check C1 of the cut command line (item walleye-cmdline-cut-check-slot-a-procedure; research walleye-cmdline-tail-cut-candidates, its
//! cmdline-cut-checks.md): with slot a the CURRENT slot, the bootloader boots our debug image once from fastboot (RAM, nothing flashed) and the
//! lab reads /proc/cmdline over the image's telnet: does `androidboot.slot_suffix=_a` arrive, or is the line cut after `androidboot.pk=g-pxl-p`
//! as every flashed slot-a boot was (runs 8, 9, 12)?
//!
//! The image is the stage 2 DEBUG image ([images.ufs-root-booted-slot-debug]): it has telnet and, when the slot is missing, refuses and holds
//! (the production image has no telnet and no way to be read). The declared image is booted AS IT IS - never with a slot put on its command
//! line, which would answer nothing. The only change on the phone is the slot flag (`fastboot set_active`), made after the author's go and undone
//! by the way back, which goes through the bootloader (the static walleye-reboot tool, sent over the image's `nc` and checked by sha256 on the
//! phone) and sets Android's slot current BEFORE any `fastboot reboot`: a plain reboot with slot a current would boot our flashed slot a.
//! What the way back does on the image is NOT KNOWN YET until the first real run (the image's busybox has `nc` by its defconfig, not by a read).

use crate::lab::Lab;
use crate::rootfs_boot::{header_cmdline, root_in_image, RootfsBoot, SUFFIX_FILE};
use crate::settings::Settings;
use crate::{host, watch};
use std::io::Write;
use std::net::{TcpStream, ToSocketAddrs};
use std::process::Command;
use std::rc::Rc;
use std::time::{Duration, Instant};

type Done = Result<(), String>;

/// The word the bootloader's own words end in when its tail is cut (research walleye-cmdline-tail-cut-candidates).
pub const PK_WORD: &str = "androidboot.pk=g-pxl-p";
/// The key of the word C1 looks for.
pub const SUFFIX_KEY: &str = "androidboot.slot_suffix=";
/// The word of the debug image's header that gives it its telnet.
pub const DEBUG_WORD: &str = "walleye.debug";
/// The port the phone's `nc` listens on for the reboot tool.
pub const NC_PORT: u16 = 4444;
/// Where the tool lands on the phone (the image's tmpfs).
pub const TOOL_ON_PHONE: &str = "/tmp/walleye-reboot";
/// Kept in the run's folder: what the lab meant to do to the slot flag, written BEFORE the switch, and the slot after it.
pub const INTENT_FILE: &str = "slot-set-intent.txt";
pub const SET_FILE: &str = "slot-set.txt";
/// The one read over telnet: it holds the image up (`/tmp/stay`), prints the command line and says which of the way back's applets the image has.
pub const READ_COMMAND: &str = "touch /tmp/stay; echo ---cmdline; cat /proc/cmdline; echo ---applets; busybox --list | grep -x -e nc -e sha256sum -e chmod; echo ---end";

/// What C1 read.
#[derive(Debug, PartialEq)]
pub enum Verdict {
    /// the slot suffix arrived, and it is the current slot's
    Whole { suffix: String, length: usize },
    /// the line ends at the pk word and has no slot suffix
    Cut { length: usize },
}

impl Verdict {
    /// The line the research row records.
    pub fn said(&self, slot: &str) -> String {
        match self {
            Verdict::Whole { suffix, length } => format!("WHOLE: the bootloader's tail arrived ({length} characters) with {SUFFIX_KEY}{suffix}; with slot {slot} current the cut is not the slot-a path: the flashed normal-boot path or the image on it is the cause (go to C2)"),
            Verdict::Cut { length } => format!("CUT: the line is {length} characters and ends at {PK_WORD}, no {SUFFIX_KEY}; with slot {slot} current the slot-a path cuts whatever the image (RAM boot)"),
        }
    }
}

fn section<'a>(text: &'a str, name: &str) -> Option<Vec<&'a str>> {
    let mark = format!("---{name}");
    let mut lines = text.lines().map(str::trim_end);
    lines.find(|l| *l == mark)?;
    Some(lines.take_while(|l| !l.starts_with("---")).collect())
}

/// The command line in a telnet read ([`READ_COMMAND`]'s `---cmdline` part), or in a kernel log's `Kernel command line:` line. Exactly one non-empty line.
pub fn cmdline_in(text: &str) -> Result<String, String> {
    let lines: Vec<&str> = match section(text, "cmdline") {
        Some(l) => l.into_iter().filter(|l| !l.trim().is_empty()).collect(),
        // the kernel log prints a long line in chunks, each ending in ` \` and each led by the same words (tests/fixtures/phone-text/dmesg-ufs-root-20261125-043720.cap)
        None => {
            let mut joined = String::new();
            for chunk in text
                .lines()
                .filter_map(|l| l.split_once("Kernel command line:").map(|(_, c)| c.trim()))
            {
                joined.push_str(chunk.strip_suffix('\\').unwrap_or(chunk));
                if chunk.ends_with('\\') {
                    joined.push(' ');
                }
            }
            return if joined.is_empty() {
                Err("the read has no command line".into())
            } else {
                Ok(joined.trim().to_string())
            };
        }
    };
    match lines.as_slice() {
        [one] => Ok(one.trim().to_string()),
        [] => Err("the read has no command line".into()),
        more => Err(format!(
            "the read has {} lines where one command line was expected",
            more.len()
        )),
    }
}

/// C1's answer for the slot fastboot named current: the suffix arrived and agrees, or the line ends at the pk word without it. Anything else is NOT KNOWN YET, said with the line's end.
pub fn judge(line: &str, slot: &str) -> Result<Verdict, String> {
    let words: Vec<&str> = line.split_whitespace().collect();
    let length = line.trim().chars().count();
    let suffixes: Vec<&str> = words
        .iter()
        .filter_map(|w| w.strip_prefix(SUFFIX_KEY))
        .collect();
    match suffixes.as_slice() {
        [s] if *s == format!("_{slot}") => Ok(Verdict::Whole {
            suffix: (*s).to_string(),
            length,
        }),
        [s] => Err(format!("the bootloader gave {SUFFIX_KEY}{s} while fastboot's current slot is {slot}: not what C1 asks, nothing is concluded")),
        [] if words.last() == Some(&PK_WORD) => Ok(Verdict::Cut { length }),
        [] => Err(format!("NOT KNOWN YET: the line has no {SUFFIX_KEY} and does not end at {PK_WORD}; its last words: {}", words.iter().rev().take(4).rev().copied().collect::<Vec<_>>().join(" "))),
        more => Err(format!("the line has {} slot suffixes: {more:?}", more.len())),
    }
}

/// The image is the debug image C1 needs and nothing else: its header carries the telnet word once, no slot of its own, no word that writes the card.
pub fn header_is_the_debug_image(header: &str) -> Done {
    let words: Vec<&str> = header.split_whitespace().collect();
    match words.iter().filter(|w| **w == DEBUG_WORD).count() {
        1 => {}
        n => {
            return Err(format!(
                "the image's command line carries {DEBUG_WORD} {n} times, once is its telnet: without it nothing can be read"
            ))
        }
    }
    if words.iter().any(|w| w.starts_with(SUFFIX_KEY)) {
        return Err(format!("the image's command line already carries {SUFFIX_KEY}: C1 asks whether the BOOTLOADER gives it"));
    }
    if words.contains(&crate::rw_write::WORD) {
        return Err(format!(
            "the image's command line carries {}: this check never writes",
            crate::rw_write::WORD
        ));
    }
    Ok(())
}

/// What the parent's way back does: after a switch, a reboot is sent only from a phone that is on Android or in the bootloader with Android's slot set first.
#[derive(Debug, PartialEq)]
pub enum Plan {
    /// in the bootloader after a switch: set Android's slot, then the shared way back
    Switch,
    /// nothing was switched, or the phone is on Android: the shared way back
    Reboot,
    /// switched, and the phone is nowhere a reboot is safe: no reboot, the hand step
    Hand,
}

pub fn way_back_plan(switched: bool, on_android: bool, in_fastboot: bool) -> Plan {
    match (switched, on_android, in_fastboot) {
        (true, _, true) => Plan::Switch,
        (true, false, false) => Plan::Hand,
        _ => Plan::Reboot,
    }
}

/// The files of an earlier run of the same day that this run must not take for its own.
pub const STALE: [&str; 5] = [
    INTENT_FILE,
    SET_FILE,
    "cmdline.txt",
    "verdict.txt",
    "slots-after-way-back.txt",
];

/// `_b` and `b` name slot b.
pub fn slot_of(suffix: &str) -> Result<String, String> {
    let s = suffix.trim().trim_start_matches('_');
    match s {
        "a" | "b" => Ok(s.to_string()),
        other => Err(format!("the slot {other:?} is neither a nor b")),
    }
}

/// What the author is told before the run.
pub fn expect(luns: u64) -> String {
    format!("Check C1, about 10 minutes: the slot Android is NOT on is made current (fastboot set_active, the one change, undone by the way back), our debug image is booted once from memory with its own command line (nothing flashed, no slot on it), and /proc/cmdline is read over its telnet: does {SUFFIX_KEY} arrive, or is the line cut at {PK_WORD}? Then the phone goes to its bootloader by the reboot tool, Android's slot is made current again BEFORE the reboot, and Android is checked back with the card's {luns} disks, the boot chain and the slot flags read as before. If the tool cannot be sent over the image's nc, the way back says so and names the hand step.")
}

/// What the author is told before the rehearsal.
pub fn expect_rehearsal(luns: u64) -> String {
    format!("The rehearsal of check C1, about 5 minutes: every step but the slot switch and the boot, run for real - the debug image and its header, the stored boot chain, the reboot tool, the card read from Android ({luns} disks and the root hashed), the guard with fastboot, the slots' flags. The slot is not switched, nothing is booted, nothing is written; the phone ends on Android. No go is needed or read.")
}

fn fastboot_ok(s: &Settings, serial: &str, args: &[&str]) -> Result<String, String> {
    let mut c = Command::new(&s.fastboot);
    c.args(["-s", serial]).args(args);
    let said = host::run(c)?;
    if said.ok {
        Ok(said.text)
    } else {
        Err(format!(
            "fastboot {}: {}",
            args.join(" "),
            host::last_line(&said.text)
        ))
    }
}

/// The check: the shared steps of the root boot (the card, the guard, the flags, the go) through [`RootfsBoot`], and the steps of its own.
#[derive(Clone)]
pub struct CmdlineCut {
    s: Rc<Settings>,
    lab: Lab,
    root: RootfsBoot,
}

impl CmdlineCut {
    pub fn new(s: Settings) -> CmdlineCut {
        let root = RootfsBoot::new(s.clone());
        let s = Rc::new(s);
        CmdlineCut {
            lab: Lab::new(s.clone()),
            s,
            root,
        }
    }

    /// The root boot's steps this check shares (battery, Android as root, the slot read by adb, the card, fastboot, the guard, the flags, the go).
    pub fn shared(&self) -> &RootfsBoot {
        &self.root
    }

    fn saved(&self, name: &str) -> Result<String, String> {
        std::fs::read_to_string(self.s.out.join(name)).map_err(|e| format!("{name}: {e}"))
    }

    /// The slot Android runs from, as adb read it at the start of this run.
    fn android_slot(&self) -> Result<String, String> {
        slot_of(&self.saved(SUFFIX_FILE)?)
    }

    fn serial(&self) -> Result<String, String> {
        self.lab.serial()
    }

    /// adb reads the slot (the shared step) after the files of an earlier run of the day are forgotten: a stale switch or verdict is never this run's.
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
        let text = expect(self.s.luns);
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

    /// The image is the declared debug image (sha256, a NOT-TO-BOOT name), carries its telnet word once and names one root.
    pub fn the_image_is_the_declared_debug_image(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            host::image_has_sha256(&u.s.image, &u.s.image_sha256)?;
            if !u.s.image.to_string_lossy().contains("NOT-TO-BOOT") {
                return Err(format!("{} is not under a NOT-TO-BOOT name: a UFS-on image never lives where a queue looks", u.s.image.display()));
            }
            let bytes =
                std::fs::read(&u.s.image).map_err(|e| format!("{}: {e}", u.s.image.display()))?;
            header_is_the_debug_image(&header_cmdline(&bytes)?)?;
            println!("  the image names its root: {}", root_in_image(&bytes)?);
            Ok(())
        }
    }

    /// The stored boot chain is whole (the lab's own check) and the reboot tool is the declared one (the way back's tool).
    pub fn the_stored_set_and_the_tool_are_good(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            host::image_has_sha256(&u.s.reboot_tool, &u.s.reboot_tool_sha256)
                .map_err(|e| format!("the reboot tool is not the declared one: {e}"))?;
            let dir = u.s.bootchain.to_string_lossy().into_owned();
            u.lab
                .script("bootchain-readback-check.sh", &[&dir], &[])
                .map(|_| ())
        }
    }

    /// THE ONE CHANGE: in fastboot, the slot Android is not on is made current. The intent (and the undo) is written first, the result read back.
    pub fn the_other_slot_is_made_current(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let android = u.android_slot()?;
            let other = crate::install::other_slot(&android)?;
            let root = u.root.the_root()?;
            if root != format!("system_{other}") {
                return Err(format!("the image's root {root} is not the system of slot {other}: nothing is switched"));
            }
            u.lab.save(INTENT_FILE, &format!("fastboot set_active {other} (Android runs from {android}); the way back: fastboot set_active {android}"))?;
            let serial = u.serial()?;
            fastboot_ok(&u.s, &serial, &["set_active", other])?;
            let now = crate::install::parse_current_slot(&fastboot_ok(
                &u.s,
                &serial,
                &["getvar", "current-slot"],
            )?)?;
            if now != other {
                return Err(format!(
                    "after set_active {other} fastboot's current slot is {now}"
                ));
            }
            u.lab.save(SET_FILE, other)?;
            println!("  slot {other} is current (Android runs from {android}); the way back sets {android}");
            Ok(())
        }
    }

    /// The boot itself, from memory, the declared image as it is: the gate right before it is the root boot's (battery, sha256, the go, spent).
    pub fn boot_the_debug_image(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            u.root.the_root()?;
            u.saved(SET_FILE)?;
            u.lab.battery_is_good()?;
            host::image_has_sha256(&u.s.image, &u.s.image_sha256)?;
            u.root.go_is_open_now(true)?;
            u.lab.boot_image(&u.s.image)
        }
    }

    /// The command line, read over telnet, kept, and judged: the answer is the line, whole or cut; neither is a failure of the run.
    pub fn the_command_line_is_read(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let mut sh = u.lab.shell_when_up()?;
            let text = sh.run(READ_COMMAND, Duration::from_secs(30))?;
            u.lab.save("read.txt", &crate::touch::redact(&text))?;
            let line = cmdline_in(&text)?;
            u.lab.save("cmdline.txt", &crate::touch::redact(&line))?;
            let slot = u.saved(SET_FILE)?;
            let said = judge(&line, slot.trim())?.said(slot.trim());
            u.lab.save("verdict.txt", &said)?;
            let applets = section(&text, "applets").unwrap_or_default();
            println!("  {said}");
            println!(
                "  the image's applets for the way back: {}",
                applets.join(" ")
            );
            for need in ["nc", "sha256sum", "chmod"] {
                if !applets.contains(&need) {
                    return Err(format!("the answer is kept (verdict.txt), but the image has no {need}: the reboot tool cannot be sent, so no reboot is made; hold power and volume-down until the bootloader screen shows, then `fastboot set_active {}` and `fastboot reboot`", u.android_slot()?));
                }
            }
            Ok(())
        }
    }

    /// The reboot tool goes to the phone over its `nc` and is checked by sha256 THERE before it runs; the phone is asked for its bootloader.
    pub fn the_phone_is_sent_to_the_bootloader(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || u.send_to_the_bootloader()
    }

    fn send_to_the_bootloader(&self) -> Done {
        host::image_has_sha256(&self.s.reboot_tool, &self.s.reboot_tool_sha256)
            .map_err(|e| format!("the reboot tool is not the declared one: {e}"))?;
        let mut sh = self.lab.shell_when_up()?;
        let listen = format!("(nc -l -p {NC_PORT} > {TOOL_ON_PHONE}) > /dev/null 2>&1 &");
        sh.run(&listen, Duration::from_secs(5))?;
        std::thread::sleep(Duration::from_secs(1));
        let bytes = std::fs::read(&self.s.reboot_tool)
            .map_err(|e| format!("{}: {e}", self.s.reboot_tool.display()))?;
        let target = (self.s.address.as_str(), NC_PORT)
            .to_socket_addrs()
            .map_err(|e| e.to_string())?
            .next()
            .ok_or("no address for the phone")?;
        let mut tcp =
            TcpStream::connect_timeout(&target, Duration::from_secs(10)).map_err(|e| {
                format!(
                    "the phone's nc does not answer on {}:{NC_PORT}: {e}",
                    self.s.address
                )
            })?;
        tcp.write_all(&bytes)
            .map_err(|e| format!("sending the tool: {e}"))?;
        drop(tcp);
        let said = sh.run(
            &format!("sleep 2; sha256sum {TOOL_ON_PHONE}"),
            Duration::from_secs(30),
        )?;
        if said.split_whitespace().next() != Some(self.s.reboot_tool_sha256.as_str()) {
            return Err(format!(
                "the tool on the phone is not the declared one (wanted {}): {said}",
                self.s.reboot_tool_sha256
            ));
        }
        sh.run(
            &format!("chmod 755 {TOOL_ON_PHONE}"),
            Duration::from_secs(5),
        )?;
        // the phone hangs up as it goes down: the end of the answer is not read
        let _ = sh.run_partial(
            &format!("{TOOL_ON_PHONE} --command bootloader"),
            Duration::from_secs(10),
        );
        self.fastboot_is_reached()
    }

    fn fastboot_is_reached(&self) -> Done {
        let until = Instant::now() + Duration::from_secs(self.s.device_wait);
        while !self.lab.in_fastboot().unwrap_or(false) {
            if Instant::now() >= until {
                return Err(format!("fastboot does not list the phone within {} s of the reboot tool; by hand: hold power and volume-down until the bootloader screen shows, then `fastboot set_active {}` and `fastboot reboot`", self.s.device_wait, self.android_slot().unwrap_or_else(|_| "<Android's slot>".into())));
            }
            std::thread::sleep(Duration::from_secs(2));
        }
        Ok(())
    }

    /// In the bootloader: Android's slot is made current again (a phone already on it is not written to), the flags are read and judged as the stage 2 way back judges them.
    pub fn the_switch_is_made_back(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            u.switch_back()?;
            u.lab.fastboot_reboot(); // Android's slot is current: only now is a reboot safe (the watching parent's way back covers a stop before it)
            Ok(())
        }
    }

    fn switch_back(&self) -> Done {
        let android = self.android_slot()?;
        let serial = self.serial()?;
        let now = crate::install::parse_current_slot(&fastboot_ok(
            &self.s,
            &serial,
            &["getvar", "current-slot"],
        )?)?;
        if now != android {
            fastboot_ok(&self.s, &serial, &["set_active", &android]).map_err(|e| {
                format!("{e}; by hand: fastboot set_active {android} && fastboot reboot")
            })?;
        }
        let after = host::slot_flags(&self.s.fastboot, &serial)?;
        self.lab.save("slots-after-way-back.txt", &after)?;
        crate::flash_boot::way_back_restored(&self.saved("slots-before.txt")?, &after, &android)
    }

    /// The way back, the last step, run by the watching parent whatever ended the run. Android's slot is made current again before ANY reboot
    /// when this run switched the slot; a phone still on the image is sent to its bootloader first.
    pub fn way_back(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            if std::env::var_os(watch::CHILD).is_some() {
                return Ok(());
            }
            let switched = u.s.out.join(SET_FILE).exists() || u.s.out.join(INTENT_FILE).exists();
            if switched && !u.lab.in_fastboot().unwrap_or(false) && !u.lab.is_on_android_now() {
                if let Err(e) = u.send_to_the_bootloader() {
                    println!("way back: the image was not sent to its bootloader: {e}");
                }
            }
            let plan = way_back_plan(
                switched,
                u.lab.is_on_android_now(),
                u.lab.in_fastboot().unwrap_or(false),
            );
            match plan {
                Plan::Switch => {
                    u.switch_back()?;
                    u.lab.way_back()
                }
                Plan::Reboot => u.lab.way_back(),
                Plan::Hand => Err(format!("the slot was switched and the phone is neither on Android nor in the bootloader: NO reboot is sent (slot {} would boot our flashed slot); hold power and volume-down until the bootloader screen shows, then `fastboot set_active {}` and `fastboot reboot`", u.saved(SET_FILE).unwrap_or_else(|_| "a/b".into()).trim(), u.android_slot().unwrap_or_else(|_| "<Android's slot>".into()))),
            }
        }
    }

    /// The rehearsal's way back: nothing was switched or booted; the shared way back is all it needs.
    pub fn rehearsal_way_back(&self) -> impl FnOnce() -> Done + 'static {
        self.root.rehearsal_way_back()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Android b's own line, read from the running phone (research walleye-bootimage 2026-11-23, android-b-cmdline-and-boot-props.txt): the bootloader's whole tail, slot b.
    const ANDROID_B: &str = include_str!(
        "../../../research/walleye-bootimage/2026-11-23/android-b-cmdline-and-boot-props.txt"
    );
    /// The kernel log of a RAM boot of our image (tests/fixtures/phone-text, captured by the root boot): its `Kernel command line:` ends at the pk word.
    const CUT_DMESG: &str =
        include_str!("../../../tests/fixtures/phone-text/dmesg-ufs-root-20261125-043720.cap");

    fn whole_line() -> &'static str {
        ANDROID_B.lines().next().unwrap()
    }

    #[test]
    fn a_captured_whole_tail_is_whole_for_its_own_slot_only() {
        // source: the captured line of Android b (1401 characters); a slot-a whole line was never captured, so the branch for a is this one with the slot argument
        let v = judge(whole_line(), "b").unwrap();
        assert_eq!(
            v,
            Verdict::Whole {
                suffix: "_b".into(),
                length: 1401
            }
        );
        let e = judge(whole_line(), "a").unwrap_err();
        assert!(
            e.contains("slot_suffix=_b while fastboot's current slot is a"),
            "{e}"
        );
    }

    #[test]
    fn a_captured_cut_line_is_cut() {
        // NO slot-a cut line was ever captured whole (the evidence file holds only the ends of such lines): this is the captured joined line of a RAM boot
        // with the one word the lab itself put on its header (androidboot.slot_suffix=_b) taken out - what is left ends at the pk word, as the cut lines do.
        let captured = cmdline_in(CUT_DMESG).unwrap();
        assert!(captured.contains("androidboot.slot_suffix=_b"));
        let line = captured.replace("androidboot.slot_suffix=_b ", "");
        let v = judge(&line, "a").unwrap();
        // the length is the captured line's own, counted here
        assert_eq!(
            v,
            Verdict::Cut {
                length: line.chars().count()
            }
        );
        assert!(line.ends_with(PK_WORD));
    }

    #[test]
    fn the_read_of_the_telnet_is_one_line_between_its_marks() {
        let text = format!(
            "---cmdline\n{}\n---applets\nnc\nsha256sum\nchmod\n---end",
            whole_line()
        );
        assert_eq!(cmdline_in(&text).unwrap(), whole_line());
        assert!(cmdline_in("---cmdline\n---applets\n---end").is_err());
        assert!(cmdline_in("---cmdline\na\nb\n---end")
            .unwrap_err()
            .contains("2 lines"));
        assert_eq!(
            section(&text, "applets").unwrap(),
            ["nc", "sha256sum", "chmod"]
        );
    }

    #[test]
    fn a_line_that_is_neither_is_not_known_yet() {
        let e = judge("ro init=/init rootwait", "a").unwrap_err();
        assert!(e.contains("NOT KNOWN YET"), "{e}");
        assert!(judge(
            "x androidboot.slot_suffix=_a androidboot.slot_suffix=_a",
            "a"
        )
        .is_err());
    }

    #[test]
    fn the_image_must_carry_its_telnet_once_and_no_slot() {
        // source: the first `cmdline = ` of device.toml (the early boot image, no telnet word); the real header of the debug image is NOT captured as a fixture yet (NOT KNOWN YET)
        let manifest =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../device.toml"))
                .unwrap();
        let production = manifest
            .lines()
            .find_map(|l| {
                l.strip_prefix("cmdline = \"")
                    .and_then(|r| r.strip_suffix('"'))
            })
            .unwrap();
        assert!(header_is_the_debug_image(production)
            .unwrap_err()
            .contains("0 times"));
        let debug = format!("{production} {DEBUG_WORD}");
        header_is_the_debug_image(&debug).unwrap();
        assert!(header_is_the_debug_image(&format!("{debug} {DEBUG_WORD}")).is_err());
        assert!(
            header_is_the_debug_image(&format!("{debug} androidboot.slot_suffix=_a"))
                .unwrap_err()
                .contains("already carries")
        );
        assert!(
            header_is_the_debug_image(&format!("{debug} {}", crate::rw_write::WORD))
                .unwrap_err()
                .contains("never writes")
        );
    }

    #[test]
    fn no_reboot_is_sent_after_a_switch_from_a_phone_that_is_nowhere_safe() {
        assert_eq!(way_back_plan(true, false, false), Plan::Hand);
        assert_eq!(way_back_plan(true, false, true), Plan::Switch);
        assert_eq!(way_back_plan(true, true, false), Plan::Reboot);
        assert_eq!(way_back_plan(false, false, false), Plan::Reboot);
        assert_eq!(way_back_plan(false, false, true), Plan::Reboot);
    }

    #[test]
    fn a_suffix_names_its_slot() {
        assert_eq!(slot_of("_b\n").unwrap(), "b");
        assert_eq!(slot_of("a").unwrap(), "a");
        assert!(slot_of("_c").is_err());
    }
}
