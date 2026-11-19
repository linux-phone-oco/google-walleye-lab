// SPDX-License-Identifier: MIT OR Apache-2.0
//! The touch try's steps (item walleye-attend-program-touch): z21 once from memory, the touch reader open and
//! showing EVENTS-READY BEFORE the author is asked, the window long enough for the swipe, the way back.
//! Each method is one step of tests/attended/touch-try.rs and returns what that step runs; each fails with
//! what it saw, and the entry file's runner stops the run there. Nothing is written to the phone beyond /tmp.

use crate::host;
use crate::lab::Lab;
use crate::reader::Reader;
use crate::settings::Settings;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// z21's init pets the SoC watchdog for 90 beats of 2 s and then restarts the phone; nothing keeps it up.
pub const SELF_REBOOT_S: u64 = 180;
/// What is left of the budget for the phone to be seen and the reader to be closed.
const MARGIN_S: u64 = 8;

/// What the try learned about the phone while it ran: the touch node and the window.
#[derive(Default)]
struct Facts {
    node: String,
    window: u64,
}

/// The try: its settings, what it learned, the reader while it runs. Cheap to clone: each step takes its own.
#[derive(Clone)]
pub struct Touch {
    s: Rc<Settings>,
    lab: Lab,
    facts: Rc<RefCell<Facts>>,
    reader: Rc<RefCell<Option<Reader>>>,
}

type Done = Result<(), String>;

/// How long the reader may stay open: the wanted window, or what the phone's own restart at SELF_REBOOT_S leaves
/// after `uptime` seconds, less the margin (z21's init has no hold: research walleye-z21-init-has-no-hold).
pub fn window_for(uptime: u64, wanted: u64) -> u64 {
    wanted.min(SELF_REBOOT_S.saturating_sub(uptime + MARGIN_S))
}

/// The uptime (whole seconds) from /proc/uptime's text.
pub fn parse_uptime(text: &str) -> Option<u64> {
    text.split_whitespace()
        .next()?
        .split('.')
        .next()?
        .parse()
        .ok()
}

/// The event node (eventN) of the RMI4 / Synaptics device in /proc/bus/input/devices.
pub fn touch_node(devices: &str) -> Option<String> {
    devices.split("\n\n").find_map(|block| {
        let named = block.lines().any(|l| {
            let l = l.to_lowercase();
            l.starts_with("n: name=") && (l.contains("rmi") || l.contains("synaptics"))
        });
        if !named {
            return None;
        }
        let h = block.lines().find(|l| l.starts_with("H: Handlers="))?;
        h.trim_start_matches("H: Handlers=")
            .split_whitespace()
            .find(|w| w.starts_with("event"))
            .map(String::from)
    })
}

/// The phone's serial number (androidboot.serialno=) is not kept in what the try saves.
pub fn redact(text: &str) -> String {
    text.lines()
        .map(|l| {
            l.split(' ')
                .map(|w| {
                    if w.starts_with("androidboot.serialno=") {
                        "androidboot.serialno=REDACTED"
                    } else {
                        w
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

impl Touch {
    /// The same try on other settings: the lab follows them.
    #[cfg(test)]
    fn with(&self, s: Settings) -> Touch {
        let s = Rc::new(s);
        Touch {
            lab: Lab::new(s.clone()),
            s,
            ..self.clone()
        }
    }

    pub fn new(s: Settings) -> Touch {
        let s = Rc::new(s);
        Touch {
            lab: Lab::new(s.clone()),
            s,
            facts: Rc::default(),
            reader: Rc::default(),
        }
    }

    fn save(&self, name: &str, text: &str) -> Done {
        std::fs::write(self.s.out.join(name), format!("{text}\n"))
            .map_err(|e| format!("{name}: {e}"))
    }

    /// The battery is above the lab's minimum (the shared step).
    pub fn battery_is_good(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.battery_is_good()
    }

    /// The image is the one built: its sha256 is the one its SHA256SUMS line records.
    pub fn image_is_the_built_one(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || host::image_is_the_built_one(&t.s.image)
    }

    /// UFS is off in the image (the gate every image passes before a boot, decision 2026-11-19-walleye-ufs-off-by-a-gate).
    pub fn ufs_is_off(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let image = t.s.image.to_string_lossy().into_owned();
            let line = t.lab.script("ufs-off-image.sh", &[&image], &[])?;
            if line.starts_with("pass ") {
                Ok(())
            } else {
                Err(line)
            }
        }
    }

    /// The phone is in fastboot: from Android `adb reboot bootloader` takes it there; when fastboot lists it already,
    /// nothing is done (the shared step). Nothing is written to the phone.
    pub fn phone_is_in_fastboot(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.phone_is_in_fastboot()
    }

    /// Boot the image from the phone's fastboot, from memory: nothing is written to it.
    pub fn boot_image(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let mut c = std::process::Command::new(&t.s.lpoco);
            c.args(["lab", "--device"])
                .arg(t.s.device_toml())
                .args(["fastboot", "boot", "now"])
                .arg(&t.s.image);
            let said = host::run(c)?;
            if said.ok {
                Ok(())
            } else {
                Err(format!("the boot did not go: {}", said.text))
            }
        }
    }

    /// The gadget is up on USB, the shell answers, and the phone's dmesg is saved.
    pub fn shell_answers(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            std::fs::create_dir_all(&t.s.out).map_err(|e| format!("{}: {e}", t.s.out.display()))?;
            let _ = std::fs::set_permissions(
                &t.s.out,
                std::os::unix::fs::PermissionsExt::from_mode(0o700),
            );
            let toml = t.s.device_toml();
            t.lab.script(
                "gadget-up.sh",
                &[&toml.to_string_lossy()],
                &[("GADGET_WAIT", "60")],
            )?;
            let mut sh = t.lab.shell(10)?;
            let dmesg = sh.run("dmesg", Duration::from_secs(20))?;
            if dmesg.trim().is_empty() {
                return Err("the shell answered, but dmesg is empty".into());
            }
            t.save("dmesg.txt", &redact(&dmesg))
        }
    }

    /// The touch controller's input node is listed (the saved list names it); the window is then known.
    pub fn touch_device_is_listed(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let until = Instant::now() + Duration::from_secs(t.s.device_wait);
            let mut sh = t.lab.shell(10)?;
            loop {
                let devices = sh.run("cat /proc/bus/input/devices", Duration::from_secs(10))?;
                if let Some(node) = touch_node(&devices) {
                    t.save("input-devices.txt", &devices)?;
                    let dmesg = sh.run("dmesg", Duration::from_secs(20))?;
                    t.save("dmesg.txt", &redact(&dmesg))?;
                    let up = sh.run("cat /proc/uptime", Duration::from_secs(10))?;
                    let uptime = parse_uptime(&up).ok_or(format!("no uptime in {up:?}"))?;
                    let mut f = t.facts.borrow_mut();
                    f.node = node;
                    f.window = window_for(uptime, t.s.window);
                    return Ok(());
                }
                if Instant::now() >= until {
                    return Err(format!(
                        "no RMI4/Synaptics input device within {} s",
                        t.s.device_wait
                    ));
                }
                std::thread::sleep(Duration::from_secs(2));
            }
        }
    }

    /// The window is long enough to be worth asking the author for: said before he is asked.
    pub fn window_is_long_enough(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let f = t.facts.borrow();
            println!(
                "  the reader's window is {} s (the phone restarts by itself at {} s of uptime)",
                f.window, SELF_REBOOT_S
            );
            if f.window >= t.s.min_window {
                Ok(())
            } else {
                Err(format!(
                    "the window would be {} s (less than {} s): the author is not asked; the phone restarts by itself at {} s of uptime",
                    f.window, t.s.min_window, SELF_REBOOT_S
                ))
            }
        }
    }

    /// The reader opens the touch node for the window, on a session of its own.
    pub fn reader_is_started(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let f = t.facts.borrow();
            let node = format!("/dev/input/{}", f.node);
            let r = Reader::start(
                &t.s.address,
                t.s.port,
                &node,
                f.window,
                t.s.out.join("events.txt"),
            );
            *t.reader.borrow_mut() = Some(r);
            Ok(())
        }
    }

    /// The reader says EVENTS-READY: the node is open and nothing the author does is lost.
    pub fn reader_shows_events_ready(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let reader = t.reader.borrow();
            let r = reader.as_ref().ok_or("the reader was not started")?;
            let s = r.wait_ready(Duration::from_secs(15));
            if s.ready {
                println!(
                    "  EVENTS-READY: the reader has the touch node open for {} s",
                    t.facts.borrow().window
                );
                Ok(())
            } else {
                Err(s
                    .error
                    .unwrap_or_else(|| "the reader never said EVENTS-READY".into()))
            }
        }
    }

    /// The window closes: the reader's events are all saved.
    pub fn window_has_closed(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let r = t
                .reader
                .borrow_mut()
                .take()
                .ok_or("the reader was not started")?;
            let s = r.finish();
            println!("  the reader saw {} event lines", s.lines);
            match s.error {
                Some(e) => Err(format!("the reader ended with: {e}")),
                None => Ok(()),
            }
        }
    }

    /// The phone is back on Android (the shared step).
    pub fn android_is_back(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.android_is_back()
    }

    /// The saved files say the touch probed and a touch made events (the lab's touch-check: a `skip` - nobody
    /// touched - is not a pass, and says "not known yet").
    pub fn events_are_counted(&self) -> impl FnOnce() -> Done + 'static {
        let t = self.clone();
        move || {
            let out = t.s.out.to_string_lossy().into_owned();
            let line = t.lab.script("touch-check.sh", &[&out], &[])?;
            println!("  {line}");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEVICES: &str = "I: Bus=0000 Vendor=0000 Product=0000 Version=0000\nN: Name=\"qpnp_pon\"\nH: Handlers=kbd event0\n\nI: Bus=0018 Vendor=0000 Product=0000 Version=0000\nN: Name=\"Synaptics S3708AR\"\nH: Handlers=event1 \nB: EV=b\n";

    #[test]
    fn the_window_is_the_wanted_one_or_what_the_restart_leaves() {
        assert_eq!(window_for(107, 120), 65);
        assert_eq!(window_for(20, 120), 120);
        assert_eq!(window_for(175, 120), 0);
    }

    #[test]
    fn uptime_and_the_touch_node_are_read_from_the_phones_text() {
        assert_eq!(parse_uptime("107.42 200.1"), Some(107));
        assert_eq!(parse_uptime(""), None);
        assert_eq!(touch_node(DEVICES).as_deref(), Some("event1"));
        assert_eq!(
            touch_node("N: Name=\"qpnp_pon\"\nH: Handlers=kbd event0\n"),
            None
        );
    }

    #[test]
    fn the_serial_is_not_kept() {
        assert_eq!(
            redact("a androidboot.serialno=ABC123 b\nx"),
            "a androidboot.serialno=REDACTED b\nx"
        );
    }
}

#[cfg(test)]
mod walkthrough {
    //! The whole try against a stand-in phone and stand-in host scripts (the real touch-check is used): the
    //! steps in the entry file's order, with a synthetic touch - no phone, no author.
    use super::*;
    use crate::phone::fake;

    fn event(kind: &str) -> String {
        let mut b = vec!["00"; 24];
        b[16] = kind;
        b.join(" ")
    }

    fn lab(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("wt-lab-{name}-{}", std::process::id()));
        let hosts = d.join("tests/host");
        std::fs::create_dir_all(&hosts).unwrap();
        std::fs::write(
            d.join("device.toml"),
            "[connection]\naddresses = [\"127.0.0.1\"]\n",
        )
        .unwrap();
        for (script, says) in [
            ("battery-ok.sh", "pass walleye-battery-ok host 4100 mV"),
            ("ufs-off-image.sh", "pass walleye-ufs-off image"),
            ("gadget-up.sh", "pass walleye-gadget-up kernel up"),
            ("idle-android.sh", "pass walleye-idle-android host back"),
        ] {
            std::fs::write(hosts.join(script), format!("echo '{says}'\n")).unwrap();
        }
        let real = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/host/touch-check.sh"
        ))
        .unwrap();
        std::fs::write(hosts.join("touch-check.sh"), real).unwrap();
        d
    }

    fn the_try(name: &str, port: u16) -> (Touch, std::path::PathBuf) {
        let d = lab(name);
        let mut s = Settings::stand_in(crate::settings::Try::Touch);
        s.lab = d.clone();
        s.out = d.join("out");
        s.address = "127.0.0.1".into();
        s.port = port;
        s.window = 5;
        s.min_window = 3;
        s.device_wait = 5;
        s.back_wait = 5;
        let image = d.join("boot-z.img");
        std::fs::write(&image, "image").unwrap();
        s.image = image;
        (Touch::new(s), d)
    }

    fn phone(uptime: &'static str, events: Vec<String>) -> u16 {
        fake::phone(
            vec![
                ("input/devices", "N: Name=\"Synaptics S3708AR\"\nH: Handlers=event1\n"),
                ("uptime", uptime),
                ("dmesg", "Linux version 7.3 androidboot.serialno=SECRET\nrmi4_i2c 0-0020: found RMI device"),
            ],
            events,
        )
    }

    fn all_steps(t: &Touch) -> Done {
        t.shell_answers()()?;
        t.touch_device_is_listed()()?;
        t.window_is_long_enough()()?;
        t.reader_is_started()()?;
        t.reader_shows_events_ready()()?;
        t.window_has_closed()()?;
        t.android_is_back()()?;
        t.events_are_counted()()
    }

    #[test]
    fn a_touch_is_counted_and_nothing_secret_is_saved() {
        let (t, d) = the_try(
            "pass",
            phone("107.4 5.0", vec![event("03"), event("03"), event("00")]),
        );
        t.ufs_is_off()().unwrap();
        t.battery_is_good()().unwrap();
        all_steps(&t).unwrap();
        let dmesg = std::fs::read_to_string(d.join("out/dmesg.txt")).unwrap();
        assert!(dmesg.contains("androidboot.serialno=REDACTED") && !dmesg.contains("SECRET"));
        assert_eq!(
            std::fs::read_to_string(d.join("out/events.txt"))
                .unwrap()
                .lines()
                .count(),
            3
        );
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn nobody_touching_is_not_known_yet_not_a_pass() {
        let (t, d) = the_try("nobody", phone("107.4 5.0", vec![]));
        let e = all_steps(&t).unwrap_err();
        assert!(e.starts_with("skip walleye-touch-events"), "{e}");
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn the_try_starts_from_android_and_takes_the_phone_to_fastboot_itself() {
        let (t, d) = the_try("fb", 1);
        std::fs::write(
            d.join("device.toml"),
            "[connection]\naddresses = [\"127.0.0.1\"]\nserial = \"SER1\"\n",
        )
        .unwrap();
        let m = d.join("bootloader");
        for (n, body) in [
            (
                "adb",
                format!("case \"$*\" in *get-state*) echo device; exit 0;; esac; echo \"$*\" > \"{0}.args\"; : > \"{0}\"", m.display()),
            ),
            (
                "fastboot",
                format!("[ -e \"{}\" ] && echo 'SER1 fastboot'; exit 0", m.display()),
            ),
        ] {
            let p = d.join(n);
            std::fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
            std::fs::set_permissions(&p, std::os::unix::fs::PermissionsExt::from_mode(0o755))
                .unwrap();
        }
        let t = t.with(Settings {
            adb: d.join("adb").to_string_lossy().into_owned(),
            fastboot: d.join("fastboot").to_string_lossy().into_owned(),
            ..(*t.s).clone()
        });
        t.phone_is_in_fastboot()().unwrap();
        assert_eq!(
            std::fs::read_to_string(d.join("bootloader.args"))
                .unwrap()
                .trim(),
            "-s SER1 reboot bootloader"
        );
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn a_window_the_restart_would_cut_short_stops_before_the_author_is_asked() {
        let (t, d) = the_try("short", phone("170.0 5.0", vec![]));
        t.shell_answers()().unwrap();
        t.touch_device_is_listed()().unwrap();
        let e = t.window_is_long_enough()().unwrap_err();
        assert!(e.contains("the author is not asked"), "{e}");
        std::fs::remove_dir_all(d).ok();
    }
}
