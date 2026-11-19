// SPDX-License-Identifier: MIT OR Apache-2.0
//! What a try reads about its phone: every fact is a key of this lab's `device.toml`, in the table that owns it
//! (decisions 2026-11-19-attended-keys-in-the-manifest, 2026-11-19-procedure-values-from-the-named-device).
//! The entry file reads them once, at the top of `main`, through the runner's named-key reads (`dev`, `proc`), so
//! `--dry-run` fails on a missing key before any act. No device fact is read from an environment variable.
//!
//! | what | key |
//! |---|---|
//! | the phone's address and shell port | `[connection] shell_address`, `shell_port` |
//! | the host programs | `[tools] adb`, `fastboot` |
//! | waits for the phone | `[boot] device_wait`, `back_wait` |
//! | the battery floor | `[health] battery_floor_mv` (tests/host/battery-ok.sh reads it too) |
//! | the images | `[images] dir`, `[images.z21]`, `[images.wdt1-a]`, `[images.wdt1-b]` (`path`, `sha256`) |
//! | the touch try's values | `[procedures.touch-events-on-z21] window`, `min_window` |
//! | the PMIC try's values | `[procedures.pmic-watchdog-v1-on-the-pixel-2] stay`, `bite_wait` |
//! | the display try's values | `[images.l6]`, `[procedures.display-stripes-on-the-pixel-2] hold`, `module_pause` |
//! | the UFS try's values | `[images.ufs-match]` (`path`, `sha256`), `[procedures.ufs-matched-image-on-the-pixel-2]` `dtb`, `kernel_tree`, `go_item` |
//! | the display-at-boot try's values | `[images.b2]` (`path`, `sha256`), `[procedures.display-at-boot-on-the-pixel-2] modules, hold` |

use lpoco_attended::prelude::{dev, proc};
use std::path::PathBuf;
use std::process::Command;

/// The procedure names of the tries: the entry files and the `[procedures.NAME]` tables agree on them.
pub const TOUCH_PROCEDURE: &str = "touch events on z21";
pub const PMIC_PROCEDURE: &str = "PMIC watchdog v1 on the Pixel 2";
pub const DISPLAY_PROCEDURE: &str = "display stripes on the Pixel 2";
pub const DISPLAY_BOOT_PROCEDURE: &str = "display at boot on the Pixel 2";
pub const UFS_PROCEDURE: &str = "UFS matched image on the Pixel 2";

/// Which try the settings are read for: each reads only its own procedure's values and images.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Try {
    Touch,
    Pmic,
    Display,
    DisplayBoot,
    Ufs,
}

/// The settings of one try. The fields of the other try stay empty (zero, no path).
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub lab: PathBuf,
    pub image: PathBuf,
    pub out: PathBuf,
    pub window: u64,
    pub min_window: u64,
    pub device_wait: u64,
    pub back_wait: u64,
    /// the lowest battery voltage (mV) a try may start at: `[health] battery_floor_mv`
    pub battery_floor_mv: u64,
    pub address: String,
    pub port: u16,
    pub lpoco: String,
    pub adb: String,
    pub fastboot: String,
    /// the PMIC watchdog try: image A, image B, how long Android stays up, how long the bite may take
    pub image_a: PathBuf,
    pub image_b: PathBuf,
    pub stay: u64,
    pub bite_wait: u64,
    /// the display try: seconds the drawer holds the stripes, seconds between two module loads
    pub hold: u64,
    pub module_pause: u64,
    /// the display-at-boot try: how many display modules the init must say ok for, and the sha256 its image must have
    pub modules: u64,
    pub image_sha256: String,
    /// the UFS try: the dtb the matched image was built with, the kernel tree it was built from, the item the author's go is recorded on
    pub dtb: PathBuf,
    pub kernel_tree: PathBuf,
    pub go_item: String,
    /// where a try saves what it reads when a test named one, by the try's name
    pub out_given: Option<PathBuf>,
    pub day: String,
}

impl Settings {
    /// The touch try's settings, read from the device's `device.toml` (call after `procedure(...)`).
    pub fn for_touch() -> Settings {
        Settings::read(Try::Touch, &|k| dev(k).to_string(), &|k| {
            proc(k).to_string()
        })
    }

    /// The display try's settings, read from the device's `device.toml` (call after `procedure(...)`).
    pub fn for_display() -> Settings {
        Settings::read(Try::Display, &|k| dev(k).to_string(), &|k| {
            proc(k).to_string()
        })
    }

    /// The display-at-boot try's settings, read from the device's `device.toml` (call after `procedure(...)`).
    pub fn for_display_boot() -> Settings {
        Settings::read(Try::DisplayBoot, &|k| dev(k).to_string(), &|k| {
            proc(k).to_string()
        })
    }

    /// The UFS try's settings, read from the device's `device.toml` (call after `procedure(...)`).
    pub fn for_ufs() -> Settings {
        Settings::read(Try::Ufs, &|k| dev(k).to_string(), &|k| proc(k).to_string())
    }

    /// The PMIC try's settings, read from the device's `device.toml` (call after `procedure(...)`).
    pub fn for_pmic() -> Settings {
        Settings::read(Try::Pmic, &|k| dev(k).to_string(), &|k| proc(k).to_string())
    }

    /// Read every key through `dev` (a dotted key of a table) and `proc` (a key of the try's own table); a value
    /// of the wrong kind stops the program naming the key.
    pub fn read(
        which: Try,
        dev: &dyn Fn(&str) -> String,
        proc: &dyn Fn(&str) -> String,
    ) -> Settings {
        let number = |key: &str, text: String| -> u64 {
            text.trim()
                .parse()
                .unwrap_or_else(|_| panic!("{key} in device.toml is not a whole number: {text:?}"))
        };
        let lab: PathBuf = std::env::current_dir().unwrap_or_else(|_| ".".into());
        let image_dir = dev("images.dir");
        let image = |name: &str| {
            lab.join(&image_dir)
                .join(dev(&format!("images.{name}.path")))
        };
        let mut s = Settings {
            lab: lab.clone(),
            image: PathBuf::new(),
            out: PathBuf::new(),
            window: 0,
            min_window: 0,
            device_wait: number("boot.device_wait", dev("boot.device_wait")),
            back_wait: number("boot.back_wait", dev("boot.back_wait")),
            battery_floor_mv: number("health.battery_floor_mv", dev("health.battery_floor_mv")),
            address: dev("connection.shell_address"),
            port: u16::try_from(number(
                "connection.shell_port",
                dev("connection.shell_port"),
            ))
            .expect("connection.shell_port in device.toml is a port number"),
            lpoco: "lpoco".into(),
            adb: dev("tools.adb"),
            fastboot: dev("tools.fastboot"),
            image_a: PathBuf::new(),
            image_b: PathBuf::new(),
            stay: 0,
            bite_wait: 0,
            hold: 0,
            module_pause: 0,
            modules: 0,
            image_sha256: String::new(),
            dtb: PathBuf::new(),
            kernel_tree: PathBuf::new(),
            go_item: String::new(),
            out_given: None,
            day: today(),
        };
        match which {
            Try::Touch => {
                s.image = image("z21");
                s.window = number("window", proc("window"));
                s.min_window = number("min_window", proc("min_window"));
                s.out = s.out_dir("touch");
            }
            Try::Pmic => {
                s.image_a = image("wdt1-a");
                s.image_b = image("wdt1-b");
                s.stay = number("stay", proc("stay"));
                s.bite_wait = number("bite_wait", proc("bite_wait"));
            }
            Try::Display => {
                s.image = image("l6");
                s.hold = number("hold", proc("hold"));
                s.module_pause = number("module_pause", proc("module_pause"));
                s.out = s.out_dir("display");
            }
            Try::Ufs => {
                s.image = image("ufs-match");
                s.image_sha256 = dev("images.ufs-match.sha256");
                s.dtb = lab.join(&image_dir).join(proc("dtb"));
                s.kernel_tree = home_path(&proc("kernel_tree"));
                s.go_item = proc("go_item");
                s.out = s.out_dir("ufs");
            }
            Try::DisplayBoot => {
                s.image = image("b2");
                s.image_sha256 = dev("images.b2.sha256");
                s.hold = number("hold", proc("hold"));
                s.modules = number("modules", proc("modules"));
                s.out = s.out_dir("display-boot");
            }
        }
        s
    }

    /// Where the try called `name` saves what it reads: a folder of the lab's material (mode 700, never in git),
    /// unless a test named one.
    pub fn out_dir(&self, name: &str) -> PathBuf {
        self.out_given
            .clone()
            .unwrap_or_else(|| home().join(format!("lab-material/walleye/{name}-{}", self.day)))
    }

    pub fn device_toml(&self) -> PathBuf {
        self.lab.join("device.toml")
    }

    pub fn host_script(&self, name: &str) -> PathBuf {
        self.lab.join("tests/host").join(name)
    }

    /// Settings for the stand-in tests: the same keys, read by the same code, from a map of values.
    #[cfg(test)]
    pub fn stand_in(which: Try) -> Settings {
        let keys: std::collections::BTreeMap<&str, &str> = [
            ("images.dir", "images"),
            ("images.z21.path", "boot-z21.img"),
            ("images.wdt1-a.path", "boot-wdt1-a.img"),
            ("images.wdt1-b.path", "boot-wdt1-b.img"),
            ("boot.device_wait", "170"),
            ("boot.back_wait", "240"),
            ("health.battery_floor_mv", "3700"),
            ("connection.shell_address", "127.0.0.1"),
            ("connection.shell_port", "23"),
            ("tools.adb", "adb"),
            ("tools.fastboot", "fastboot"),
            ("window", "120"),
            ("min_window", "40"),
            ("stay", "180"),
            ("bite_wait", "150"),
            ("images.l6.path", "boot-l6.img"),
            ("hold", "60"),
            ("module_pause", "5"),
            ("images.b2.path", "boot-b2.img"),
            ("images.b2.sha256", "e4c8"),
            ("modules", "13"),
            ("images.ufs-match.path", "NOT-TO-BOOT-x/NOT-TO-BOOT-ufs.img"),
            ("images.ufs-match.sha256", "4610"),
            ("dtb", "NOT-TO-BOOT-x/ufs-match.dtb"),
            ("kernel_tree", "~/tree"),
            ("go_item", "walleye-try-ufs-match"),
        ]
        .into_iter()
        .collect();
        let get = |k: &str| keys[k].to_string();
        Settings::read(which, &get, &get)
    }
}

/// A path with a leading `~/` taken from the home folder.
fn home_path(text: &str) -> PathBuf {
    match text.strip_prefix("~/") {
        Some(rest) => home().join(rest),
        None => PathBuf::from(text),
    }
}

fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
}

fn today() -> String {
    Command::new("date")
        .arg("+%Y%m%d-%H%M%S")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_try_reads_its_own_values_and_images() {
        let t = Settings::stand_in(Try::Touch);
        assert_eq!((t.window, t.min_window, t.port), (120, 40, 23));
        assert_eq!(t.image, t.lab.join("images/boot-z21.img"));
        assert_eq!((t.stay, t.image_a), (0, PathBuf::new()));
        let p = Settings::stand_in(Try::Pmic);
        assert_eq!((p.stay, p.bite_wait, p.back_wait), (180, 150, 240));
        assert_eq!(p.image_b, p.lab.join("images/boot-wdt1-b.img"));
        assert_eq!(p.window, 0);
        assert_eq!((p.address.as_str(), p.adb.as_str()), ("127.0.0.1", "adb"));
    }

    #[test]
    fn the_display_at_boot_try_reads_its_image_its_sha256_and_its_module_count() {
        let b = Settings::stand_in(Try::DisplayBoot);
        assert_eq!(b.image, b.lab.join("images/boot-b2.img"));
        assert_eq!((b.modules, b.image_sha256.as_str()), (13, "e4c8"));
        assert!(b.out.ends_with(format!("display-boot-{}", b.day)));
        assert_eq!((b.hold, b.module_pause), (60, 0));
    }

    /// Every key the program reads is in this lab's real device.toml (a renamed key fails here, not at the author's dry run).
    #[test]
    fn this_labs_device_toml_has_every_key() {
        let raw: toml::Value = toml::from_str(
            &std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../device.toml"))
                .unwrap(),
        )
        .unwrap();
        let at = |path: &str| -> String {
            let v = path
                .split('.')
                .try_fold(&raw, |v, p| v.get(p))
                .unwrap_or_else(|| panic!("device.toml has no {path}"));
            v.as_str()
                .map(str::to_string)
                .unwrap_or_else(|| v.to_string())
        };
        for (which, table) in [
            (Try::Touch, TOUCH_PROCEDURE),
            (Try::Pmic, PMIC_PROCEDURE),
            (Try::Display, DISPLAY_PROCEDURE),
            (Try::DisplayBoot, DISPLAY_BOOT_PROCEDURE),
            (Try::Ufs, UFS_PROCEDURE),
        ] {
            let name: Vec<String> = table
                .split(|c: char| !c.is_ascii_alphanumeric())
                .filter(|w| !w.is_empty())
                .map(str::to_ascii_lowercase)
                .collect();
            let own = format!("procedures.{}.", name.join("-"));
            let s = Settings::read(which, &at, &|k| at(&format!("{own}{k}")));
            assert_eq!(s.port, 23);
        }
    }

    #[test]
    #[should_panic(expected = "boot.back_wait in device.toml is not a whole number")]
    fn a_value_of_the_wrong_kind_is_named() {
        let get = |k: &str| {
            match k {
                "boot.back_wait" => "soon",
                "connection.shell_port" | "boot.device_wait" => "5",
                _ => "x",
            }
            .to_string()
        };
        Settings::read(Try::Pmic, &get, &|_| "5".into());
    }
}
