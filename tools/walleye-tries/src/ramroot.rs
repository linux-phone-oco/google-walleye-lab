// SPDX-License-Identifier: MIT OR Apache-2.0
//! The Alpine login prompt on the Pixel's glass as ONE command (item walleye-display-alpine-ramroot-image; the author's word of
//! 2026-11-22: "i want to see the thing on pixel"): image B-a5 (B-r5's kernel, dtb and 13 display modules, the Alpine userland unpacked
//! into a tmpfs root by the image's own init, UFS off) booted once from the phone's fastboot, from memory. The lab logs in over
//! usb-network by ssh with its declared key and proves, from what the phone says, that the display modules loaded at the delay with none
//! of the three signs, that a getty runs on the glass (tty1) and on the gadget serial (ttyGS0), that sshd runs, that the kernel log has
//! fbcon, and that the root is a tmpfs; then the author looks at the glass. Each method is one step of
//! procedures/alpine-login-on-the-glass-on-the-pixel-2.rs.
//!
//! What the image's init must say (the proof reads these, so they are the contract between the image and this program):
//!   - to the kernel log, `walleye-display: delay N s reached at uptime T s`, then per module `walleye-display: insmod ...` and
//!     `walleye-display: ok` (B-r5's own words), then a line ending `display modules: done`;
//!   - to the kernel log, lines tagged `walleye-ramroot:` (INIT_TAG);
//!   - the watchdog petter keeps the phone up while `/run/walleye/hold` exists (the root init's hold) and the deadman restarts it to Android
//!     by itself when it has not.
//!
//! The phone is never written: the boot is from memory, the hold is a file in the tmpfs, the way back is a reboot.

use crate::display_boot::LOOP_DONE;
use crate::display_delay::{score, Row};
use crate::lab::Lab;
use crate::rootfs_boot::{identity_is_private, sections, ssh_args, RootfsBoot};
use crate::settings::Settings;
use crate::touch::redact;
use crate::{host, watch};
use std::process::Command;
use std::rc::Rc;
use std::time::{Duration, Instant};

type Done = Result<(), String>;

/// The tag the image's init puts on every line it says to the kernel log.
pub const INIT_TAG: &str = "walleye-ramroot:";
/// The file whose presence keeps the image's deadman off (the root init's hold).
pub const HOLD_FILE: &str = "/run/walleye/hold";
/// How long a hold set by this program lasts at most, whatever happens to the lab: the phone's own shell removes it.
pub const HOLD_SECONDS: u64 = 300;

/// What is said to the author before the first act on the phone.
pub const EXPECT: &str = "The run boots the phone from memory (nothing is written to it): the screen shows the kernel log's white text on black, and at about 5 s the display driver starts; then Alpine Linux starts from RAM and the screen should show a login prompt, `walleye login:` (the root's login has no password on the glass). The usb cable stays in: ssh comes over it (172.16.43.1, the lab's key). The run checks everything it can read first and then asks you whether you see the prompt. The phone restarts into Android by itself about five minutes after it booted unless the run holds it for your look, and the run sends it back when it is done.";
/// What the author is asked to look for.
pub const LOOK_FOR: &str = "do you see an Alpine login prompt on the screen? (`walleye login:` after the boot text; say what you see instead if it is blank, dark or something else)";

/// The VT the glass's getty runs on: Alpine's own default (VT 1 is the console fbcon shows at boot). Busybox getty on /dev/tty0 printed no prompt in
/// the look at B-a5 (research walleye-ramroot-b-a5-the-look-no-login-prompt); the lab's screen check below says whether this line is right.
pub const GLASS_TTY: &str = "tty1";
/// The tty of the gadget's serial, the lab's second console.
pub const GADGET_TTY: &str = "ttyGS0";
/// The most the lab waits for the prompt in the screen's text, seconds: the getty is up before this step, and the deadman (300 s from the hand-over) must not be reached.
pub const PROMPT_WAIT: u64 = 30;

/// The one read the lab makes after the login, read-only and kept whole in read.txt: the VT the glass shows (the active one), the consoles, every process
/// with its tty, the inittab, the kernel's command line and the hostname - what the next try needs if the screen's check fails.
pub const READ_COMMAND: &str = "echo ---id; id -u; echo ---kernel; uname -r; echo ---mounts; cat /proc/mounts; echo ---ps; (ps -o pid,args 2>&1 || ps 2>&1); echo ---pstty; (ps -o pid,tty,args 2>&1 || true); echo ---active; cat /sys/class/tty/tty0/active 2>&1; echo ---consoles; cat /proc/consoles 2>&1; echo ---inittab; cat /etc/inittab 2>&1; echo ---cmdline; cat /proc/cmdline 2>&1; echo ---hostname; hostname; echo ---dmesg; dmesg; echo ---services; rc-status --crashed 2>&1 || true; echo ---net; ip -o addr show usb0 2>&1; echo ---end";

/// The screen's text, read by the lab: the hostname, the VT the glass shows, then that VT's text (/dev/vcs is the active VT's), one line per screen row.
pub const SCREEN_COMMAND: &str = "echo ---hostname; hostname; echo ---active; cat /sys/class/tty/tty0/active 2>&1; echo ---screen; c=$(stty size </dev/tty0 2>/dev/null | cut -d' ' -f2); if [ \"${c:-0}\" -gt 0 ] && fold -w \"$c\" /dev/vcs 2>/dev/null; then :; else cat /dev/vcs 2>&1; fi; echo ---end";

/// The screen's text holds the login prompt of this host: `HOSTNAME login:` on one row (white space between the words counts as one space).
pub fn screen_has_login_prompt(screen: &str, hostname: &str) -> bool {
    let want = format!("{} login:", hostname.trim());
    !hostname.trim().is_empty()
        && screen.lines().any(|l| {
            l.split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .contains(&want)
        })
}

/// What the lab saw instead, for the log: the VT the glass shows and the last non-blank rows of its text.
pub fn what_the_screen_shows(active: &str, screen: &str) -> String {
    let rows: Vec<&str> = screen
        .lines()
        .map(str::trim_end)
        .filter(|l| !l.is_empty())
        .collect();
    let tail = &rows[rows.len().saturating_sub(6)..];
    format!(
        "the glass shows {}; its last rows:\n{}",
        active.trim(),
        tail.join("\n")
    )
}

/// The hold: the file is made and removed again by the phone's own shell after [`HOLD_SECONDS`], so a lab that dies leaves the deadman in the end.
pub fn hold_command() -> String {
    format!(
        "mkdir -p /run/walleye && touch {HOLD_FILE} && (setsid sh -c 'sleep {HOLD_SECONDS}; rm -f {HOLD_FILE}' >/dev/null 2>&1 </dev/null &)"
    )
}

/// The hold is released.
pub fn release_command() -> String {
    format!("rm -f {HOLD_FILE}")
}

fn section<'a>(parts: &'a [(String, String)], name: &str) -> Result<&'a str, String> {
    parts
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, t)| t.as_str())
        .ok_or_else(|| format!("the read has no {name} section"))
}

/// The root is a tmpfs (the Alpine userland unpacked into RAM), not a card: the line of `/` in /proc/mounts.
pub fn root_is_tmpfs(proc_mounts: &str) -> Result<String, String> {
    let root = proc_mounts
        .lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>())
        .find(|f| f.len() >= 3 && f[1] == "/")
        .ok_or("no line of / in /proc/mounts")?;
    if root[2] == "tmpfs" {
        Ok(root[0].to_string())
    } else {
        Err(format!(
            "the root is {} of type {}, not a tmpfs",
            root[0], root[2]
        ))
    }
}

/// A getty runs on `tty` (a whole word of its line: tty0 is not ttyGS0).
pub fn getty_on(ps: &str, tty: &str) -> bool {
    ps.lines()
        .any(|l| l.contains("getty") && l.split_whitespace().any(|w| w == tty))
}

/// sshd runs (its own process, not this lab's ssh).
pub fn sshd_runs(ps: &str) -> bool {
    ps.lines().any(|l| {
        l.split_whitespace()
            .any(|w| w == "sshd" || w.ends_with("/sshd"))
    })
}

/// The kernel log says the console is on the frame buffer: fbcon took the VTs.
pub fn fbcon_said(dmesg: &str) -> Option<String> {
    dmesg
        .lines()
        .find(|l| {
            l.contains("fbcon:") || l.contains("Console: switching to colour frame buffer device")
        })
        .map(|l| l.trim().to_string())
}

/// The init's own lines, tagged.
pub fn init_lines(dmesg: &str) -> Vec<String> {
    dmesg
        .lines()
        .filter(|l| l.contains(INIT_TAG))
        .map(|l| l.trim().to_string())
        .collect()
}

/// Lines of the log that say a purge, an oops or a panic.
pub fn trouble(dmesg: &str) -> Vec<String> {
    dmesg
        .lines()
        .filter(|l| {
            l.contains("purge refused")
                || l.contains("Oops")
                || l.contains("Kernel panic")
                || l.contains("BUG:")
        })
        .map(|l| l.trim().to_string())
        .collect()
}

/// The three signs and the load, said in one line.
pub fn row_says(r: &Row) -> String {
    format!(
        "loaded at {} s; SMMU faults {}, rcg warnings {}, DSI -110 {}; {} of {} modules ok{}",
        r.reached.map_or("?".to_string(), |t| t.to_string()),
        r.smmu_faults,
        r.rcg_warnings,
        r.dsi_110,
        r.modules_ok,
        r.modules_want,
        if r.finished {
            ""
        } else {
            " (the list is not done)"
        }
    )
}

/// The try: its settings, the lab and the ssh program (`ssh`; a test names a stand-in).
#[derive(Clone)]
pub struct Ramroot {
    s: Rc<Settings>,
    lab: Lab,
    boot: RootfsBoot,
    ssh: String,
}

impl Ramroot {
    pub fn new(s: Settings) -> Ramroot {
        Ramroot::with_ssh(s, "ssh")
    }

    pub fn with_ssh(s: Settings, ssh: &str) -> Ramroot {
        let boot = RootfsBoot::with_ssh(s.clone(), ssh);
        let s = Rc::new(s);
        Ramroot {
            lab: Lab::new(s.clone()),
            boot,
            s,
            ssh: ssh.to_string(),
        }
    }

    fn delay(&self) -> Result<u64, String> {
        self.s
            .delays
            .first()
            .copied()
            .ok_or_else(|| "device.toml names no delay".to_string())
    }

    /// One command on the phone over ssh as the declared key, bounded.
    fn ssh(&self, command: &str, seconds: u64) -> Result<String, String> {
        let mut c = Command::new("timeout");
        c.args([
            "--kill-after=5".to_string(),
            seconds.to_string(),
            self.ssh.clone(),
        ])
        .args(ssh_args(
            &self.s.identity,
            &self.s.ssh_user,
            &self.s.address,
            self.s.ssh_port,
            command,
        ));
        let said = host::run(c)?;
        if said.ok {
            Ok(said.text)
        } else {
            Err(format!("ssh: {}", host::last_line(&said.text)))
        }
    }

    fn read_section(&self, name: &str) -> Result<String, String> {
        let text = std::fs::read_to_string(self.s.out.join("read.txt"))
            .map_err(|e| format!("read.txt: {e}"))?;
        section(&sections(&text), name).map(str::to_string)
    }

    /// The battery is above the lab's minimum (the shared step).
    pub fn battery_is_good(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.battery_is_good()
    }

    /// The image is the declared one: its sha256 is its SHA256SUMS line's and the one device.toml records.
    pub fn the_image_is_the_declared_one(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            host::image_is_the_built_one(&u.s.image)?;
            host::image_has_sha256(&u.s.image, &u.s.image_sha256)
        }
    }

    /// UFS is off in the image (the gate every image passes before a boot).
    pub fn ufs_is_off(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || u.lab.ufs_is_off(&u.s.image)
    }

    /// The key the lab logs in with is a private file.
    pub fn the_key_is_private(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || identity_is_private(&u.s.identity)
    }

    /// Say what to expect, before the first act on the phone.
    pub fn author_is_told(&self) -> impl FnOnce() -> Done + 'static {
        move || {
            println!("  {EXPECT}");
            Ok(())
        }
    }

    /// The phone is in fastboot; from here the watching parent knows the phone was touched.
    pub fn phone_is_in_fastboot(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || {
            watch::phone_is_touched();
            lab.phone_is_in_fastboot()
        }
    }

    /// Boot the image from the phone's fastboot, from memory: nothing is written. Its gate runs again right before the boot.
    pub fn boot_the_image(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            u.lab.battery_is_good()?;
            host::image_has_sha256(&u.s.image, &u.s.image_sha256)?;
            u.lab.ufs_is_off(&u.s.image)?;
            u.lab.boot_image(&u.s.image)
        }
    }

    /// The lab logs in by ssh as the declared key and reads the phone, polled until `device_wait` is over (the gadget, the network and sshd
    /// come up one after the other) and until the init has said its display list is done. The read is kept, serials redacted.
    pub fn the_login_works_and_the_phone_is_read(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let until = Instant::now() + Duration::from_secs(u.s.device_wait);
            let mut last: Result<String, String>;
            loop {
                match u.ssh(READ_COMMAND, 40) {
                    Ok(text) if text.contains("---end") => {
                        let secs = sections(&text);
                        let said = |name: &str, f: &dyn Fn(&str) -> bool| {
                            secs.iter().any(|(n, t)| n == name && f(t))
                        };
                        // the modules are through, and both gettys run (busybox init starts them after OpenRC's default level, sshd answers inside it)
                        let done = said("dmesg", &|t| {
                            t.lines().any(|l| l.trim_end().ends_with(LOOP_DONE))
                        }) && said("ps", &|t| {
                            getty_on(t, GLASS_TTY) && getty_on(t, GADGET_TTY)
                        });
                        last = Ok(text);
                        if done {
                            break;
                        }
                    }
                    Ok(_) => last = Err("the read did not finish".into()),
                    Err(e) => last = Err(e),
                }
                if Instant::now() >= until {
                    break;
                }
                std::thread::sleep(Duration::from_secs(3));
            }
            match last {
                Ok(text) => u.lab.save("read.txt", &redact(&text)),
                Err(e) => Err(format!(
                    "no ssh login within {} s as {}@{}: {e}",
                    u.s.device_wait, u.s.ssh_user, u.s.address
                )),
            }
        }
    }

    /// The login is root's, usb0 has the lab's address, and no service of the root crashed.
    pub fn the_phone_is_the_image_we_booted(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let id = u.read_section("id")?;
            if id.trim() != "0" {
                return Err(format!("the login is uid {:?}, not root's 0", id.trim()));
            }
            let crashed = u.read_section("services")?;
            if !crashed.trim().is_empty() {
                return Err(format!("services crashed: {}", crashed.trim()));
            }
            let net = u.read_section("net")?;
            if !net.contains(&u.s.address) {
                return Err(format!(
                    "usb0 has not the address {}: {}",
                    u.s.address,
                    net.trim()
                ));
            }
            println!("  kernel {}", u.read_section("kernel")?.trim());
            Ok(())
        }
    }

    /// The image's init spoke: its tagged lines are in the kernel log, and the root it made is a tmpfs.
    pub fn the_root_is_a_tmpfs_the_init_made(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let lines = init_lines(&u.read_section("dmesg")?);
            if lines.is_empty() {
                return Err(format!(
                    "the kernel log has no line of the init ({INIT_TAG})"
                ));
            }
            let dev = root_is_tmpfs(&u.read_section("mounts")?)?;
            println!(
                "  / is {dev}, a tmpfs; the init said {} lines, the last: {}",
                lines.len(),
                lines.last().map_or("", |l| l.as_str())
            );
            Ok(())
        }
    }

    /// The display modules loaded at the delay, every one said ok, and none of the three signs is in the log.
    pub fn the_modules_loaded_and_the_three_signs_are_zero(
        &self,
    ) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let delay = u.delay()?;
            let row = score(delay, u.s.modules as usize, &u.read_section("dmesg")?);
            println!("  {}", row_says(&row));
            let reached = row.reached.ok_or(format!(
                "the init never said 'delay {delay} s reached': it is not the image's init"
            ))?;
            if reached < delay {
                return Err(format!(
                    "the init said it reached the delay of {delay} s at uptime {reached} s"
                ));
            }
            if !row.loaded() {
                return Err(format!(
                    "the load did not finish with every module ok: {}",
                    row_says(&row)
                ));
            }
            if !row.is_clean() {
                return Err(format!(
                    "a sign of trouble is in the log: {}",
                    row_says(&row)
                ));
            }
            Ok(())
        }
    }

    /// A getty runs on the glass (tty1) and on the gadget serial (ttyGS0), and sshd runs.
    pub fn the_getties_and_sshd_run(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let ps = u.read_section("ps")?;
            for tty in [GLASS_TTY, GADGET_TTY] {
                if !getty_on(&ps, tty) {
                    return Err(format!("no getty runs on {tty}:\n{}", ps.trim()));
                }
            }
            if !sshd_runs(&ps) {
                return Err(format!("sshd does not run:\n{}", ps.trim()));
            }
            Ok(())
        }
    }

    /// The SCREEN's text holds the login prompt: /dev/vcs of the VT the glass shows, read over ssh until `HOSTNAME login:` is on a row (the getty is
    /// respawned until it is there), kept in screen.txt. The author is asked to look only after this passes: a look is asked when the lab measured the prompt.
    pub fn the_screen_holds_the_login_prompt(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let until = Instant::now() + Duration::from_secs(u.s.device_wait.min(PROMPT_WAIT));
            loop {
                let read = u.ssh(SCREEN_COMMAND, 20);
                let seen = read.as_ref().map(|t| sections(t)).unwrap_or_default();
                let part = |n: &str| section(&seen, n).unwrap_or_default().to_string();
                let (host, active, screen) = (part("hostname"), part("active"), part("screen"));
                if screen_has_login_prompt(&screen, &host) {
                    u.lab.save("screen.txt", &redact(&screen))?;
                    println!(
                        "  the screen's text ({}) holds `{} login:`",
                        active.trim(),
                        host.trim()
                    );
                    return Ok(());
                }
                if Instant::now() >= until {
                    u.lab.save("screen.txt", &redact(&screen))?;
                    return Err(match read {
                        Err(e) => format!("the screen could not be read: {e}"),
                        Ok(_) => format!(
                            "the login prompt `{} login:` is not in the screen's text after {} s; {}",
                            host.trim(),
                            u.s.device_wait.min(PROMPT_WAIT),
                            what_the_screen_shows(&active, &screen)
                        ),
                    });
                }
                std::thread::sleep(Duration::from_secs(3));
            }
        }
    }

    /// The kernel log has fbcon: the console is on the frame buffer, so the glass is where the login prompt goes.
    pub fn the_kernel_log_has_fbcon(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let said = fbcon_said(&u.read_section("dmesg")?)
                .ok_or("the kernel log has no fbcon line: the console is not on the glass")?;
            println!("  {said}");
            Ok(())
        }
    }

    /// The kernel log has no purge, oops or panic.
    pub fn the_kernel_log_is_clean(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || match trouble(&u.read_section("dmesg")?).first() {
            Some(t) => Err(format!("the kernel log says: {t}")),
            None => Ok(()),
        }
    }

    /// The deadman is held for the look: the hold file is made (and removes itself after [`HOLD_SECONDS`]).
    pub fn the_deadman_is_held(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || u.ssh(&hold_command(), 15).map(|_| ())
    }

    /// The hold is released after the look.
    pub fn the_deadman_is_released(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || u.ssh(&release_command(), 15).map(|_| ())
    }

    /// The phone is sent away from the image: busybox's forced reboot over ssh.
    pub fn the_phone_is_sent_back(&self) -> impl FnOnce() -> Done + 'static {
        self.boot.the_phone_is_sent_back()
    }

    /// The phone is back on Android (the shared step).
    pub fn android_is_back(&self) -> impl FnOnce() -> Done + 'static {
        self.boot.android_is_back()
    }

    /// The way back, run by the watching parent after the child ends, whatever ended it (an unanswered ask included).
    pub fn way_back(&self) -> impl FnOnce() -> Done + 'static {
        self.boot.way_back()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::executable;
    use crate::settings::Try;
    use std::os::unix::fs::PermissionsExt;

    const MOUNTS: &str = "tmpfs / tmpfs rw,relatime,size=1048576k 0 0\nproc /proc proc rw 0 0\n";
    const PS: &str = "  PID COMMAND\n    1 {init} /sbin/init\n  210 /usr/sbin/sshd\n  240 /sbin/getty 38400 tty1\n  241 /sbin/getty -L 115200 ttyGS0 vt100\n";
    const DMESG: &str = "[    1.0] walleye-ramroot: start\n[    5.1] walleye-display: delay 5 s reached at uptime 5 s\n[    5.2] walleye-display: insmod /lib/modules/m1.ko\n[    5.3] walleye-display: ok\n[    5.2] walleye-display: insmod /lib/modules/m2.ko\n[    5.3] walleye-display: ok\n[    5.2] walleye-display: insmod /lib/modules/m3.ko\n[    5.3] walleye-display: ok\n[    5.2] walleye-display: insmod /lib/modules/m4.ko\n[    5.3] walleye-display: ok\n[    5.2] walleye-display: insmod /lib/modules/m5.ko\n[    5.3] walleye-display: ok\n[    5.2] walleye-display: insmod /lib/modules/m6.ko\n[    5.3] walleye-display: ok\n[    5.2] walleye-display: insmod /lib/modules/m7.ko\n[    5.3] walleye-display: ok\n[    5.2] walleye-display: insmod /lib/modules/m8.ko\n[    5.3] walleye-display: ok\n[    5.2] walleye-display: insmod /lib/modules/m9.ko\n[    5.3] walleye-display: ok\n[    5.2] walleye-display: insmod /lib/modules/m10.ko\n[    5.3] walleye-display: ok\n[    5.2] walleye-display: insmod /lib/modules/m11.ko\n[    5.3] walleye-display: ok\n[    5.2] walleye-display: insmod /lib/modules/m12.ko\n[    5.3] walleye-display: ok\n[    5.2] walleye-display: insmod /lib/modules/m13.ko\n[    5.3] walleye-display: ok\n[    5.9] fbcon: msm-drmdrmfb (fb0) is primary device\n[    6.0] Console: switching to colour frame buffer device 135x120\n[    6.1] walleye-ramroot: display modules: done\n";
    const NET: &str = "5: usb0    inet 172.16.43.1/24 brd 172.16.43.255 scope global usb0";

    fn read_of(dmesg: &str, mounts: &str, ps: &str, services: &str) -> String {
        format!("---id\n0\n---kernel\n7.3.0\n---mounts\n{mounts}---ps\n{ps}---dmesg\n{dmesg}---services\n{services}---net\n{NET}\n---end\n")
    }

    #[test]
    fn the_root_is_a_tmpfs_or_it_is_named() {
        assert_eq!(root_is_tmpfs(MOUNTS).unwrap(), "tmpfs");
        let card = "/dev/sda13 / ext4 ro 0 0\n";
        assert!(root_is_tmpfs(card).unwrap_err().contains("not a tmpfs"));
        assert!(root_is_tmpfs("proc /proc proc rw 0 0\n").is_err());
    }

    #[test]
    fn a_getty_is_found_by_its_whole_tty_word() {
        assert!(getty_on(PS, "tty1") && getty_on(PS, "ttyGS0"));
        let only_gs = "  241 /sbin/getty -L 115200 ttyGS0 vt100\n";
        assert!(!getty_on(only_gs, "tty1"), "tty1 is not ttyGS0");
        assert!(!getty_on("  300 /sbin/agetty tty0\n", "tty1"));
    }

    const SCREEN: &str = "---hostname\nwalleye\n---active\ntty1\n---screen\n[    6.0] Console: switching to colour frame buffer device 135x120\n * Starting local ...\n\nwalleye login:                \n---end\n";

    #[test]
    fn the_prompt_is_a_hostname_and_login_on_one_row() {
        let on = "boot text\nwalleye   login: \n";
        assert!(screen_has_login_prompt(on, "walleye\n"));
        assert!(
            !screen_has_login_prompt(on, "other"),
            "another host's prompt"
        );
        assert!(!screen_has_login_prompt(
            " * Starting local ... [ ok ]\n_\n",
            "walleye"
        ));
        assert!(
            !screen_has_login_prompt("login:\n", ""),
            "no hostname, no proof"
        );
        let shown = what_the_screen_shows("tty1\n", "a\n\n  \nb\nc\nd\ne\nf\ng\nh\n");
        assert!(
            shown.contains("tty1") && shown.ends_with("h") && !shown.contains("\na\n"),
            "{shown}"
        );
    }

    #[test]
    fn the_screen_check_passes_on_a_prompt_and_stops_without_one() {
        let r = rig("screen-ok", SCREEN);
        (r.the_screen_holds_the_login_prompt())().unwrap();
        let kept = std::fs::read_to_string(r.s.out.join("screen.txt")).unwrap();
        assert!(kept.contains("walleye login:"));
        let c = calls(&r);
        assert!(c.contains("/dev/vcs"), "{c}");
        let none = SCREEN.replace("walleye login:", "_");
        let r = rig("screen-none", &none);
        let e = (r.the_screen_holds_the_login_prompt())().unwrap_err();
        assert!(
            e.contains("walleye login:") && e.contains("tty1") && e.contains("Starting local"),
            "{e}"
        );
        let r = rig("screen-empty", "---end\n");
        assert!((r.the_screen_holds_the_login_prompt())().is_err());
    }

    #[test]
    fn the_read_asks_for_the_vt_the_consoles_and_the_screen() {
        for want in [
            "/sys/class/tty/tty0/active",
            "/proc/consoles",
            "ps -o pid,tty,args",
            "/etc/inittab",
            "/proc/cmdline",
        ] {
            assert!(READ_COMMAND.contains(want), "{want}");
        }
        assert!(SCREEN_COMMAND.contains("/dev/vcs") && !SCREEN_COMMAND.contains(" > "));
        assert_eq!((GLASS_TTY, GADGET_TTY), ("tty1", "ttyGS0"));
    }

    #[test]
    fn sshd_is_found_as_a_process() {
        assert!(sshd_runs(PS));
        assert!(sshd_runs(
            "  300 sshd: /usr/sbin/sshd [listener] 0 of 10-100 startups\n"
        ));
        assert!(!sshd_runs("  300 sshd: root@notty\n"));
        assert!(!sshd_runs("  240 /sbin/getty 38400 tty1\n  1 ssh-keygen\n"));
    }

    #[test]
    fn fbcon_and_the_init_and_trouble_are_read_from_the_log() {
        assert!(fbcon_said(DMESG).unwrap().contains("fbcon:"));
        assert!(fbcon_said("[ 1.0] nothing\n").is_none());
        assert!(fbcon_said("Console: switching to colour frame buffer device 1x1").is_some());
        assert_eq!(init_lines(DMESG).len(), 2);
        assert!(trouble(DMESG).is_empty());
        assert_eq!(
            trouble("[ 2.0] Oops: x\n[ 3.0] purge refused on this device\n").len(),
            2
        );
    }

    #[test]
    fn the_hold_is_bounded_and_the_release_removes_it() {
        let h = hold_command();
        assert!(h.contains(HOLD_FILE) && h.contains("sleep 300"));
        assert_eq!(release_command(), "rm -f /run/walleye/hold");
    }

    #[test]
    fn the_settings_name_the_image_the_delay_and_the_login() {
        let s = Settings::stand_in(Try::Ramroot);
        assert_eq!(
            s.image,
            s.lab.join("images/images-display-ramroot/boot-b-a5.img")
        );
        assert_eq!(
            (s.image_sha256.as_str(), s.delays.clone(), s.modules),
            ("a5a5", vec![5], 13)
        );
        assert_eq!((s.ssh_user.as_str(), s.ssh_port), ("root", 22));
        assert!(s.out.ends_with(format!("alpine-login-{}", s.day)));
    }

    /// A lab whose ssh is a stand-in that answers every command with one file, as the phone would.
    fn rig(name: &str, read: &str) -> Ramroot {
        let d = std::env::temp_dir().join(format!("wt-rr-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("read.out"), read).unwrap();
        let ssh = d.join("ssh");
        executable(
            &ssh,
            &format!(
                "#!/bin/sh\necho \"$@\" >> {d}/ssh.calls\ncat {d}/read.out\n",
                d = d.display()
            ),
        );
        let key = d.join("key");
        std::fs::write(&key, "k").unwrap();
        std::fs::set_permissions(&key, PermissionsExt::from_mode(0o600)).unwrap();
        let mut s = Settings::stand_in(Try::Ramroot);
        s.out = d.join("out");
        s.identity = key;
        s.address = "172.16.43.1".into();
        s.device_wait = 1;
        std::fs::create_dir_all(&s.out).unwrap();
        Ramroot::with_ssh(s, &ssh.to_string_lossy())
    }

    fn calls(r: &Ramroot) -> String {
        std::fs::read_to_string(r.s.out.parent().unwrap().join("ssh.calls")).unwrap_or_default()
    }

    #[test]
    fn a_good_read_passes_every_step_after_the_login() {
        let r = rig("good", &read_of(DMESG, MOUNTS, PS, ""));
        (r.the_login_works_and_the_phone_is_read())().unwrap();
        (r.the_phone_is_the_image_we_booted())().unwrap();
        (r.the_root_is_a_tmpfs_the_init_made())().unwrap();
        (r.the_modules_loaded_and_the_three_signs_are_zero())().unwrap();
        (r.the_getties_and_sshd_run())().unwrap();
        (r.the_kernel_log_has_fbcon())().unwrap();
        (r.the_kernel_log_is_clean())().unwrap();
        (r.the_deadman_is_held())().unwrap();
        (r.the_deadman_is_released())().unwrap();
        let c = calls(&r);
        assert!(
            c.contains("root@172.16.43.1") && c.contains("BatchMode=yes"),
            "{c}"
        );
        assert!(
            c.contains("touch /run/walleye/hold") && c.contains("rm -f /run/walleye/hold"),
            "{c}"
        );
        let kept = std::fs::read_to_string(r.s.out.join("read.txt")).unwrap();
        assert!(kept.contains("---dmesg"));
    }

    #[test]
    fn each_failing_read_stops_its_own_step() {
        let bad = |name: &str, dmesg: &str, mounts: &str, ps: &str| {
            let r = rig(name, &read_of(dmesg, mounts, ps, ""));
            (r.the_login_works_and_the_phone_is_read())().unwrap();
            r
        };
        // a sign of trouble after the load
        let smmu = format!("{DMESG}[ 6.5] arm-smmu: Unhandled context fault: fsr=0x402\n");
        let r = bad("smmu", &smmu, MOUNTS, PS);
        assert!((r.the_modules_loaded_and_the_three_signs_are_zero())()
            .unwrap_err()
            .contains("SMMU faults 1"));
        // a module that did not say ok
        let short = DMESG.replacen("walleye-display: ok", "walleye-display: FAILED rc=1", 1);
        let r = bad("short", &short, MOUNTS, PS);
        assert!((r.the_modules_loaded_and_the_three_signs_are_zero())().is_err());
        // no getty on the glass
        let r = bad(
            "nogetty",
            DMESG,
            MOUNTS,
            "  210 /usr/sbin/sshd\n  241 /sbin/getty ttyGS0\n",
        );
        assert!((r.the_getties_and_sshd_run())()
            .unwrap_err()
            .contains("tty1"));
        // no fbcon
        let r = bad("nofb", "[ 1.0] walleye-ramroot: x\n", MOUNTS, PS);
        assert!((r.the_kernel_log_has_fbcon())().is_err());
        // a root that is a card
        let r = bad("card", DMESG, "/dev/sda13 / ext4 ro 0 0\n", PS);
        assert!((r.the_root_is_a_tmpfs_the_init_made())()
            .unwrap_err()
            .contains("not a tmpfs"));
        // an oops
        let r = bad("oops", &format!("{DMESG}[ 9.0] Oops: bad\n"), MOUNTS, PS);
        assert!((r.the_kernel_log_is_clean())().is_err());
        // a crashed service
        let r = rig("crashed", &read_of(DMESG, MOUNTS, PS, "sshd crashed"));
        (r.the_login_works_and_the_phone_is_read())().unwrap();
        assert!((r.the_phone_is_the_image_we_booted())()
            .unwrap_err()
            .contains("crashed"));
    }

    #[test]
    fn no_login_within_the_wait_is_said() {
        let r = rig("nologin", "no answer, no end marker\n");
        let e = (r.the_login_works_and_the_phone_is_read())().unwrap_err();
        assert!(
            e.contains("no ssh login within 1 s as root@172.16.43.1"),
            "{e}"
        );
    }
}

#[cfg(test)]
mod command_quoting {
    use super::*;

    #[test]
    fn the_commands_the_phone_runs_are_valid_shell() {
        for c in [READ_COMMAND, SCREEN_COMMAND] {
            let ok = std::process::Command::new("sh")
                .args(["-n", "-c", c])
                .status()
                .unwrap();
            assert!(ok.success(), "{c}");
        }
    }
}
