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

    /// The phone is back on Android (adb device, boot completed, not parked in fastboot).
    pub fn android_is_back(&self) -> Done {
        let until = Instant::now() + Duration::from_secs(self.s.back_wait);
        let toml = self.s.device_toml();
        loop {
            match self.script("idle-android.sh", &[&toml.to_string_lossy()], &[]) {
                Ok(_) => return Ok(()),
                Err(e) if Instant::now() >= until => {
                    return Err(format!(
                        "not back on Android within {} s: {e}",
                        self.s.back_wait
                    ))
                }
                Err(_) => std::thread::sleep(Duration::from_secs(5)),
            }
        }
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
