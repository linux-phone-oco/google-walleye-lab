// SPDX-License-Identifier: MIT OR Apache-2.0
//! The plumbing both tries share: this lab's shell, its host scripts, the phone's serial, and the steps that
//! every try starts or ends with (battery, fastboot, Android back). A try holds one `Lab`, never a copy.

use crate::{host, phone::Shell, settings::Settings};
use std::{
    rc::Rc,
    time::{Duration, Instant},
};

type Done = Result<(), String>;

/// How long the phone has to be listed by fastboot after `adb reboot bootloader`.
const FASTBOOT_WAIT: Duration = Duration::from_secs(90);

/// The two ways to restart a test image that has no self-reboot, over its telnet shell (PMIC run 3 of the keeper).
pub const REBOOT_NOW: &str = "reboot -f";
pub const SYSRQ_REBOOT: &str = "echo b > /proc/sysrq-trigger";
/// What is said when the phone does not return by itself: the one hand step left.
pub const POWER_HOLD: &str =
    "hold the power button about 15 seconds until the phone starts or shows the bootloader screen";

#[derive(Clone)]
pub struct Lab {
    pub s: Rc<Settings>,
}

impl Lab {
    pub fn new(s: Rc<Settings>) -> Lab {
        Lab { s }
    }

    pub fn shell(&self, wait: u64) -> Result<Shell, String> {
        Shell::open(&self.s.address, self.s.port, Duration::from_secs(wait))
    }

    /// The phone's shell, tried until it answers or `device_wait` is over: the gadget on the bus is not yet telnetd.
    pub fn shell_when_up(&self) -> Result<Shell, String> {
        let until = Instant::now() + Duration::from_secs(self.s.device_wait);
        loop {
            let up = self.shell(5).and_then(|mut sh| {
                let said = sh.run("echo up-$((6*7))", Duration::from_secs(5))?;
                if said.contains("up-42") {
                    Ok(sh)
                } else {
                    Err(format!("the shell said: {said}"))
                }
            });
            match up {
                Ok(sh) => return Ok(sh),
                Err(e) if Instant::now() >= until => {
                    return Err(format!("no shell within {} s: {e}", self.s.device_wait))
                }
                Err(_) => std::thread::sleep(Duration::from_secs(2)),
            }
        }
    }

    /// Keep what a try read in its folder of the lab's material (mode 700, never in git).
    pub fn save(&self, name: &str, text: &str) -> Done {
        std::fs::create_dir_all(&self.s.out)
            .map_err(|e| format!("{}: {e}", self.s.out.display()))?;
        let _ = std::fs::set_permissions(
            &self.s.out,
            std::os::unix::fs::PermissionsExt::from_mode(0o700),
        );
        std::fs::write(self.s.out.join(name), format!("{text}\n"))
            .map_err(|e| format!("{name}: {e}"))
    }

    pub fn script(
        &self,
        name: &str,
        args: &[&str],
        env: &[(&str, &str)],
    ) -> Result<String, String> {
        host::script(&self.s.host_script(name), args, env)
    }

    /// The phone's serial, the first of device.toml; an unreadable file or one without a serial is named.
    pub fn serial(&self) -> Result<String, String> {
        let path = self.s.device_toml();
        let toml =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        host::serial_of(&toml).ok_or_else(|| format!("no serial in {}", path.display()))
    }

    /// The battery is above the lab's minimum (a skip - nothing answers - stops the run too: not known yet).
    pub fn battery_is_good(&self) -> Done {
        let toml = self.s.device_toml();
        self.script("battery-ok.sh", &[&toml.to_string_lossy()], &[])
            .map(|_| ())
    }

    /// The phone is in fastboot: already, or taken there from Android by `adb reboot bootloader`.
    pub fn phone_is_in_fastboot(&self) -> Done {
        host::to_fastboot(
            &self.s.adb,
            &self.s.fastboot,
            &self.serial()?,
            FASTBOOT_WAIT,
        )
    }

    /// Fastboot lists this lab's phone now.
    pub fn in_fastboot(&self) -> Result<bool, String> {
        host::in_fastboot(&self.s.fastboot, &self.serial()?)
    }

    /// UFS is off in the image (the gate every image passes before a boot, decision 2026-11-19-walleye-ufs-off-by-a-gate).
    pub fn ufs_is_off(&self, image: &std::path::Path) -> Done {
        let image = image.to_string_lossy().into_owned();
        let line = self.script("ufs-off-image.sh", &[&image], &[])?;
        if line.starts_with("pass ") {
            Ok(())
        } else {
            Err(line)
        }
    }

    /// Boot the image from the phone's fastboot, from memory: nothing is written to it.
    pub fn boot_image(&self, image: &std::path::Path) -> Done {
        let mut c = std::process::Command::new(&self.s.lpoco);
        c.args(["lab", "--device"])
            .arg(self.s.device_toml())
            .args(["fastboot", "boot", "now"])
            .arg(image);
        let said = host::run(c)?;
        if said.ok {
            Ok(())
        } else {
            Err(format!("the boot did not go: {}", said.text))
        }
    }

    /// Android answers now (one look, no waiting).
    pub fn is_on_android_now(&self) -> bool {
        self.android_within(0, Duration::from_secs(1)).is_ok()
    }

    /// The phone is back on Android (adb device, boot completed, not parked in fastboot).
    pub fn android_is_back(&self) -> Done {
        self.android_within(self.s.back_wait, Duration::from_secs(5))
    }

    /// Android answers within `secs` (one look when 0), looked at again every `poll`.
    fn android_within(&self, secs: u64, poll: Duration) -> Done {
        let until = Instant::now() + Duration::from_secs(secs);
        let toml = self.s.device_toml();
        loop {
            match self.script("idle-android.sh", &[&toml.to_string_lossy()], &[]) {
                Ok(_) => return Ok(()),
                Err(e) if Instant::now() >= until => {
                    return Err(format!("not back on Android within {secs} s: {e}"))
                }
                Err(_) => std::thread::sleep(poll),
            }
        }
    }

    /// The way back, the last step of an attended try however it ended: the phone ends on Android without a hand step.
    /// Already on Android: nothing is sent. In fastboot: `fastboot reboot`. On a test image's shell: `reboot -f`, and
    /// when Android does not come, the sysrq trigger. A phone that does not return is said so, with the power-hold
    /// sentence ([`POWER_HOLD`]) - the only hand step left.
    pub fn way_back(&self) -> Done {
        let poll = Duration::from_secs(1);
        if self.android_within(0, poll).is_ok() {
            println!("way back: the phone is on Android already; nothing sent");
            return Ok(());
        }
        if self.in_fastboot().unwrap_or(false) {
            println!(
                "way back: leg 1, `fastboot reboot`, then Android within {} s",
                self.s.back_wait
            );
            let mut c = std::process::Command::new(&self.s.fastboot);
            c.args(["-s", &self.serial().unwrap_or_default(), "reboot"]);
            let _ = host::run(c);
            if self.android_within(self.s.back_wait, poll).is_ok() {
                return Ok(());
            }
        }
        let mut shell_answered = false;
        for command in [REBOOT_NOW, SYSRQ_REBOOT] {
            let Ok(mut sh) = self.shell(3) else { continue };
            shell_answered = true;
            println!(
                "way back: leg on the image's shell, `{command}`, then Android within {} s",
                self.s.back_wait
            );
            // the phone hangs up as it goes down: its answer is not read
            let _ = sh.run_partial(command, Duration::from_secs(3));
            if self.android_within(self.s.back_wait, poll).is_ok() {
                return Ok(());
            }
        }
        // no shell: the phone may be restarting by itself; look once more for as long as the wait
        if !shell_answered && self.android_within(self.s.back_wait, poll).is_ok() {
            return Ok(());
        }
        Err(format!(
            "the phone did not come back to Android within {} s of each way: {POWER_HOLD}",
            self.s.back_wait
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lab_in(name: &str, device_toml: Option<&str>) -> Lab {
        let d = std::env::temp_dir().join(format!("wt-lab-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        if let Some(t) = device_toml {
            std::fs::write(d.join("device.toml"), t).unwrap();
        }
        let mut s = Settings::stand_in(crate::settings::Try::Touch);
        s.lab = d;
        Lab::new(Rc::new(s))
    }

    #[test]
    fn the_serial_is_read_and_a_missing_file_or_serial_is_named() {
        let ok = lab_in("ok", Some("[connection]\nserial = \"SER1\"\n"));
        assert_eq!(ok.serial().unwrap(), "SER1");
        let none = lab_in("none", Some("[x]\n"));
        assert!(none.serial().unwrap_err().contains("no serial in"));
        let gone = lab_in("gone", None);
        let e = gone.serial().unwrap_err();
        assert!(e.contains("device.toml"), "{e}");
        assert!(gone
            .phone_is_in_fastboot()
            .unwrap_err()
            .contains("device.toml"));
    }
}
