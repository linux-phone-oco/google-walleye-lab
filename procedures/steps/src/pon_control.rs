// SPDX-License-Identifier: MIT OR Apache-2.0
//! The PON control read's steps (item walleye-wdt-pon-control-procedure; decision 2026-11-26-pon-reasons-read-in-our-image-after-a-control; research
//! walleye-pon-read-after-bite-the-way): the phone is powered off by adb (a known cause, no bite), the author holds volume-down and power once, the bootloader
//! lists the phone, the image `pon-read` is booted from memory (steps::phone_ready, nothing flashed) and its /bin/pon-block prints the PON block 0xC0..0xCB raw.
//! The raw lines are kept in the run's folder; what each register means is not said here (the control counts what a key press and the bootloader leave).

use crate::host;
use crate::lab::Lab;
use crate::settings::Settings;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};

type Done = Result<(), String>;

/// The test kernel the control boots (its `[images.NAME]` and `[kernels] test` entry): wdt1-a with /bin/pon-block in its ramdisk.
pub const KERNEL: &str = "pon-read";
/// The file of the run's folder holding the raw lines.
pub const BLOCK_FILE: &str = "pon-block.txt";

/// What the words said to the author are, in the author's plain words.
pub mod words {
    pub const ONE_HOLD: &str = "The PON control read, about 5 minutes, one act of yours at the phone. Nothing is written to the phone's storage and nothing is made to bite. I switch the phone off from my computer; when its screen is dark I ask you for one thing: hold volume-down and power together until the bootloader screen (white text, no Android) shows, then let go. Stay at the phone; say ok when you are there.";
    pub const HOLD: &str = "The phone is off. Now hold volume-down and power together until the bootloader screen shows (white text, 'fastboot'), then let go. Say ok when you see it.";
}

/// One line of /bin/pon-block: the twelve registers 0xc0..0xcb as read (None where the kernel's line said XX, shown ?? by the reader), and the regmap's name.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub regs: Vec<Option<u8>>,
    pub src: String,
}

/// The `pon-block ...` lines among the phone's lines. A `pon-block none ...` line is an error naming what it saw.
pub fn parse_block(text: &str) -> Result<Vec<Line>, String> {
    let mut lines = vec![];
    for l in text
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("pon-block "))
    {
        if l.starts_with("pon-block none") {
            return Err(format!("the reader found no PON registers: {l}"));
        }
        let regs: Vec<Option<u8>> = (0xc0u32..=0xcb)
            .map(|r| {
                l.split_whitespace()
                    .find_map(|w| w.strip_prefix(&format!("0x{r:02x}=")))
                    .and_then(|v| u8::from_str_radix(v, 16).ok())
            })
            .collect();
        let src = l
            .split("src=[")
            .nth(1)
            .and_then(|s| s.strip_suffix(']'))
            .ok_or_else(|| format!("no src=[...] at the end of {l:?}"))?
            .to_string();
        lines.push(Line { regs, src });
    }
    if lines.is_empty() {
        return Err(format!("no pon-block line; the phone said: {text:?}"));
    }
    Ok(lines)
}

/// A reading is a line that holds PON_REASON1 (0xc0) and POFF_REASON1 (0xc5): a line of `??` is a read that failed, not a zero.
pub fn is_a_reading(l: &Line) -> bool {
    l.regs[0].is_some() && l.regs[5].is_some()
}

/// The control: its settings, its folder.
#[derive(Clone)]
pub struct PonControl {
    s: Rc<Settings>,
    lab: Lab,
    out: Rc<PathBuf>,
}

impl PonControl {
    pub fn new(s: Settings) -> PonControl {
        let out = s.out_dir("pon-control");
        let s = Rc::new(s);
        PonControl {
            lab: Lab::new(s.clone()),
            s,
            out: Rc::new(out),
        }
    }

    pub fn battery_is_good(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.battery_is_good()
    }

    pub fn image_is_the_built_one(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || host::image_is_the_built_one(&t.s.image)
    }

    pub fn ufs_is_off(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || t.lab.ufs_is_off(&t.s.image)
    }

    /// Android answers now: the control starts from a phone that is on and up.
    pub fn the_phone_is_on_android(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.android_is_back()
    }

    fn adb(&self, args: &[&str]) -> Result<host::Said, String> {
        let mut c = std::process::Command::new(&self.s.adb);
        c.args(["-s", &self.lab.serial()?]);
        c.args(args);
        host::run(c)
    }

    /// `adb shell reboot -p`, then adb and fastboot list nothing within `off_wait`: the phone is off by a known cause (no bite).
    pub fn power_off_by_adb(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let said = t.adb(&["shell", "reboot", "-p"])?;
            if !said.ok {
                return Err(format!("adb could not send `reboot -p`: {}", said.text));
            }
            let until = Instant::now() + Duration::from_secs(t.s.off_wait);
            let mut quiet = 0;
            loop {
                let state = t.adb(&["get-state"])?;
                let on = state.ok && state.text.trim() == "device";
                quiet = if !on && !t.lab.in_fastboot()? {
                    quiet + 1
                } else {
                    0
                };
                if quiet >= 2 {
                    return Ok(());
                }
                if Instant::now() >= until {
                    return Err(format!(
                        "the phone is not off {} s after `reboot -p` (adb says {:?})",
                        t.s.off_wait,
                        state.text.trim()
                    ));
                }
                std::thread::sleep(Duration::from_secs(2));
            }
        }
    }

    /// The bootloader lists the phone, within `hold_wait` of the author's ok.
    pub fn the_bootloader_lists_the_phone(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let until = Instant::now() + Duration::from_secs(t.s.hold_wait);
            loop {
                if t.lab.in_fastboot()? {
                    return Ok(());
                }
                if Instant::now() >= until {
                    return Err(format!(
                        "fastboot lists no phone {} s after the hold: the phone is off or on Android, not in its bootloader. THE PHONE MAY BE OFF: hold power about 15 seconds until Android or the bootloader screen shows",
                        t.s.hold_wait
                    ));
                }
                std::thread::sleep(Duration::from_secs(2));
            }
        }
    }

    fn save(&self, name: &str, text: &str) -> Done {
        std::fs::create_dir_all(&*self.out).map_err(|e| format!("{}: {e}", self.out.display()))?;
        let _ = std::fs::set_permissions(
            &*self.out,
            std::os::unix::fs::PermissionsExt::from_mode(0o700),
        );
        std::fs::write(self.out.join(name), format!("{text}\n")).map_err(|e| format!("{name}: {e}"))
    }

    /// The image's shell runs `pon-block`; its raw lines are saved as they came and must hold a reading.
    pub fn the_block_is_read_and_saved(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let mut sh = t.lab.shell(10)?;
            let text = sh.run("pon-block", Duration::from_secs(30))?;
            t.save(BLOCK_FILE, &text)?;
            let lines = parse_block(&text)?;
            match lines.iter().find(|l| is_a_reading(l)) {
                Some(l) => {
                    println!("  PON block read from regmap {} ({} registers)", l.src, l.regs.iter().flatten().count());
                    Ok(())
                }
                None => Err(format!("pon-block printed {} line(s), none with PON_REASON1 and POFF_REASON1 read: {text:?}", lines.len())),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The reader's own output (tests/host/pon-block-selftest.sh's tree through build/initramfs-mainline/pon-block), kept as it printed.
    const READER: &str = include_str!("../fixtures/pon-block-reader-output.txt");

    #[test]
    fn the_readers_own_line_parses_to_twelve_registers() {
        let lines = parse_block(READER).unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].src, "spmi-0");
        assert_eq!(lines[0].regs.len(), 12);
        assert_eq!(lines[0].regs[0], Some(0x01));
        assert_eq!(lines[0].regs[2], Some(0x80));
        assert_eq!(lines[0].regs[5], Some(0x04));
        assert_eq!(lines[0].regs[7], Some(0x02));
        assert!(is_a_reading(&lines[0]));
    }

    #[test]
    fn a_none_line_and_an_absent_line_are_errors_naming_what_was_said() {
        let e = parse_block("pon-block none regmaps=[other] dbgfs=[yes]").unwrap_err();
        assert!(e.contains("regmaps=[other]"), "{e}");
        let e = parse_block("sh: pon-block: not found").unwrap_err();
        assert!(e.contains("not found"), "{e}");
    }

    #[test]
    fn a_line_of_failed_reads_is_not_a_reading() {
        let l = parse_block("pon-block 0xc0=?? 0xc1=?? 0xc2=?? 0xc3=?? 0xc4=?? 0xc5=?? 0xc6=?? 0xc7=02 0xc8=?? 0xc9=?? 0xca=?? 0xcb=?? src=[x]").unwrap();
        assert!(!is_a_reading(&l[0]));
        assert_eq!(l[0].regs[7], Some(2));
    }
}

#[cfg(test)]
mod walkthrough {
    //! The control's steps in the entry file's order against a stand-in adb, fastboot and phone: the phone goes off by `reboot -p` (adb lists it no more),
    //! fastboot lists it only once the "hold" file is there, the shell answers `pon-block` with the reader's own line. No phone, no author.
    use super::*;
    use crate::phone::fake;

    fn lab(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("wt-pon-{name}-{}", std::process::id()));
        let hosts = d.join("tests/host");
        std::fs::create_dir_all(&hosts).unwrap();
        std::fs::write(
            d.join("device.toml"),
            "[connection]\naddresses = [\"127.0.0.1\"]\nserial = \"SER1\"\n",
        )
        .unwrap();
        for (script, says) in [
            ("battery-ok.sh", "pass walleye-battery-ok host 4100 mV"),
            ("ufs-off-image.sh", "pass walleye-ufs-off image"),
        ] {
            std::fs::write(hosts.join(script), format!("echo '{says}'\n")).unwrap();
        }
        // adb: `get-state` says device until `reboot -p` was sent (then it fails, as adb does with no phone)
        crate::fixture::executable(
            &d.join("adb"),
            &format!(
                "#!/bin/sh\ncase \"$*\" in\n *reboot*) : > \"{0}/off\";;\n *get-state*) [ -e \"{0}/off\" ] && {{ echo 'error: no devices/emulators found'; exit 1; }}; echo device;;\n *) echo device;;\nesac\n",
                d.display()
            ),
        );
        crate::fixture::executable(
            &d.join("fastboot"),
            &format!(
                "#!/bin/sh\n[ -e \"{}/held\" ] && echo 'SER1 fastboot'\nexit 0\n",
                d.display()
            ),
        );
        std::fs::write(d.join("boot-pon-read.img"), "a").unwrap();
        d
    }

    fn the_control(d: &std::path::Path, port: u16) -> PonControl {
        let mut s = Settings::stand_in(crate::settings::Try::PonControl);
        s.lab = d.to_path_buf();
        s.address = "127.0.0.1".into();
        s.port = port;
        s.adb = d.join("adb").to_string_lossy().into_owned();
        s.fastboot = d.join("fastboot").to_string_lossy().into_owned();
        s.image = d.join("boot-pon-read.img");
        s.out_given = Some(d.join("out"));
        s.off_wait = 6;
        s.hold_wait = 6;
        PonControl::new(s)
    }

    #[test]
    fn off_then_the_hold_then_the_block_is_read_and_saved() {
        let d = lab("pass");
        let port = fake::phone(vec![("pon-block", READER_ANSWER)], vec![]);
        let t = the_control(&d, port);
        t.battery_is_good()().unwrap();
        t.ufs_is_off()().unwrap();
        t.power_off_by_adb()().unwrap();
        // the author has not held yet: the bootloader does not list the phone
        assert!(t.the_bootloader_lists_the_phone()()
            .unwrap_err()
            .contains("not in its bootloader"));
        std::fs::write(d.join("held"), "").unwrap();
        t.the_bootloader_lists_the_phone()().unwrap();
        t.the_block_is_read_and_saved()().unwrap();
        let saved = std::fs::read_to_string(d.join("out").join(BLOCK_FILE)).unwrap();
        assert!(
            saved.contains("0xc5=04") && saved.contains("src=[spmi-0]"),
            "{saved}"
        );
    }

    /// The reader's own line (the same text the fixture holds, kept in one place for the fake phone's table).
    const READER_ANSWER: &str = include_str!("../fixtures/pon-block-reader-output.txt");

    #[test]
    fn a_phone_that_is_still_on_after_the_power_off_is_not_off() {
        let d = lab("on");
        // an adb that never goes away
        crate::fixture::executable(&d.join("adb"), "#!/bin/sh\necho device\n");
        let t = the_control(&d, fake::closed());
        let e = t.power_off_by_adb()().unwrap_err();
        assert!(e.contains("is not off"), "{e}");
    }

    #[test]
    fn a_block_with_no_reading_stops_the_run_and_keeps_what_was_said() {
        let d = lab("none");
        let port = fake::phone(
            vec![("pon-block", "pon-block none regmaps=[a,b] dbgfs=[yes]")],
            vec![],
        );
        let t = the_control(&d, port);
        let e = t.the_block_is_read_and_saved()().unwrap_err();
        assert!(e.contains("regmaps=[a,b]"), "{e}");
        assert!(std::fs::read_to_string(d.join("out").join(BLOCK_FILE))
            .unwrap()
            .contains("none"));
    }
}
