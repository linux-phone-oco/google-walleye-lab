// SPDX-License-Identifier: MIT OR Apache-2.0
//! What a try reads from its environment, declared once with argenv. The runner (lpoco-attended) owns the
//! arguments (`--dry-run`, `--log FILE`, `--help`); everything here is a variable, so the two never clash.

use argenv_resolve::*;
use std::path::PathBuf;
use std::process::Command;

contract! {
    IMAGE: String = Input {
        key: "image",
        ty: Type::String,
        env: Some(Env::new("WALLEYE_TRY_IMAGE")),
        summary: "The boot image to try (default: images-z21/boot-z21.img of ~/Projects/walleye-build).",
        ..Input::EMPTY
    };

    OUT: String = Input {
        key: "out",
        ty: Type::String,
        env: Some(Env::new("WALLEYE_TRY_OUT")),
        summary: "Where the try saves what it reads (default: ~/lab-material/walleye/touch-DATE; mode 700, never in git).",
        ..Input::EMPTY
    };

    WINDOW: u64 = Input {
        key: "window",
        ty: Type::Uint,
        default: Some(120),
        env: Some(Env::new("WALLEYE_TRY_WINDOW")),
        summary: "Seconds the touch reader stays open at most (the phone restarts by itself at 180 s of uptime: the window is cut to what is left).",
        ..Input::EMPTY
    };

    MIN_WINDOW: u64 = Input {
        key: "min_window",
        ty: Type::Uint,
        default: Some(40),
        env: Some(Env::new("WALLEYE_TRY_MIN_WINDOW")),
        summary: "The shortest window worth asking the author for; a shorter one stops the run before he is asked.",
        ..Input::EMPTY
    };

    DEVICE_WAIT: u64 = Input {
        key: "device_wait",
        ty: Type::Uint,
        default: Some(170),
        env: Some(Env::new("WALLEYE_TRY_DEVICE_WAIT")),
        summary: "Seconds to wait for the touch input device to appear after the shell answers.",
        ..Input::EMPTY
    };

    BACK_WAIT: u64 = Input {
        key: "back_wait",
        ty: Type::Uint,
        default: Some(240),
        env: Some(Env::new("WALLEYE_TRY_BACK_WAIT")),
        summary: "Seconds to wait for the phone to be back on Android after the try.",
        ..Input::EMPTY
    };

    ADDRESS: String = Input {
        key: "address",
        ty: Type::String,
        env: Some(Env::new("WALLEYE_TRY_ADDRESS")),
        summary: "The phone's address (default: the first of [connection] addresses in device.toml).",
        ..Input::EMPTY
    };

    PORT: u64 = Input {
        key: "port",
        ty: Type::Uint,
        default: Some(23),
        env: Some(Env::new("WALLEYE_TRY_PORT")),
        summary: "The phone's telnet port; a test points it at a stand-in phone.",
        ..Input::EMPTY
    };

    LPOCO: String = Input {
        key: "lpoco",
        ty: Type::String,
        env: Some(Env::new("WALLEYE_TRY_LPOCO")),
        summary: "The lpoco program that boots the image (lpoco when none is given); a test points it at a stand-in.",
        ..Input::EMPTY
    };

    LAB: String = Input {
        key: "lab",
        ty: Type::String,
        env: Some(Env::new("WALLEYE_TRY_LAB")),
        summary: "This lab's folder, where device.toml and tests/host are (default: the folder the program runs in).",
        ..Input::EMPTY
    };

    IMAGE_A: String = Input {
        key: "image_a",
        ty: Type::String,
        env: Some(Env::new("WALLEYE_TRY_IMAGE_A")),
        summary: "PMIC watchdog try, image A (next + the six patches; default: images-wdt1/boot-wdt1-a.img of ~/Projects/walleye-build).",
        ..Input::EMPTY
    };

    IMAGE_B: String = Input {
        key: "image_b",
        ty: Type::String,
        env: Some(Env::new("WALLEYE_TRY_IMAGE_B")),
        summary: "PMIC watchdog try, image B, the fallback when A shows no banner (v7.3 + the patches; default: images-wdt1/boot-wdt1-b.img).",
        ..Input::EMPTY
    };

    STAY: u64 = Input {
        key: "stay",
        ty: Type::Uint,
        default: Some(180),
        env: Some(Env::new("WALLEYE_TRY_STAY")),
        summary: "Seconds Android must stay up after the clean reboot with no restart (the test's PASS: three minutes).",
        ..Input::EMPTY
    };

    BITE_WAIT: u64 = Input {
        key: "bite_wait",
        ty: Type::Uint,
        default: Some(150),
        env: Some(Env::new("WALLEYE_TRY_BITE_WAIT")),
        summary: "Seconds to see the phone restart by itself after the bite is armed (20 s timeout; a restart loop of about 20 s is expected after).",
        ..Input::EMPTY
    };

    ADB: String = Input {
        key: "adb",
        ty: Type::String,
        env: Some(Env::new("WALLEYE_TRY_ADB")),
        summary: "The adb program (adb when none is given); a test points it at a stand-in.",
        ..Input::EMPTY
    };

    FASTBOOT: String = Input {
        key: "fastboot",
        ty: Type::String,
        env: Some(Env::new("WALLEYE_TRY_FASTBOOT")),
        summary: "The fastboot program (fastboot when none is given); a test points it at a stand-in.",
        ..Input::EMPTY
    };
}

/// The settings of one try, every default filled in.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub lab: PathBuf,
    pub image: PathBuf,
    pub out: PathBuf,
    pub window: u64,
    pub min_window: u64,
    pub device_wait: u64,
    pub back_wait: u64,
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
    /// where a try saves what it reads when the author named none (OUT), by the try's name
    pub out_given: Option<PathBuf>,
    pub day: String,
}

impl Settings {
    /// Read the variables of the process's environment; a wrong one stops the program (as argenv does everywhere).
    pub fn from_env() -> Settings {
        let day = Command::new("date")
            .arg("+%Y%m%d-%H%M%S")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        let r = Invocation::from_env(&ProcessEnv).resolve(&Contract::records());
        handle_problems(&lint(&Contract::records(), &r), OnProblems::FailOnError);
        Settings::from_resolution(&r, &day)
    }

    /// The settings a resolution gives; `day` names the default output folder.
    pub fn from_resolution(r: &Resolution, day: &str) -> Settings {
        let home = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()));
        let lab = Contract::LAB
            .get_from(r)
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| ".".into()));
        let address = Contract::ADDRESS.get_from(r).unwrap_or_else(|| {
            let text = std::fs::read_to_string(lab.join("device.toml")).unwrap_or_default();
            first_address(&text).unwrap_or_default()
        });
        Settings {
            image: Contract::IMAGE
                .get_from(r)
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join("Projects/walleye-build/images-z21/boot-z21.img")),
            out: Contract::OUT
                .get_from(r)
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(format!("lab-material/walleye/touch-{day}"))),
            window: Contract::WINDOW.get_from(r).unwrap_or(120),
            min_window: Contract::MIN_WINDOW.get_from(r).unwrap_or(40),
            device_wait: Contract::DEVICE_WAIT.get_from(r).unwrap_or(170),
            back_wait: Contract::BACK_WAIT.get_from(r).unwrap_or(240),
            address,
            port: u16::try_from(Contract::PORT.get_from(r).unwrap_or(23))
                .expect("WALLEYE_TRY_PORT is a port number"),
            lpoco: Contract::LPOCO
                .get_from(r)
                .unwrap_or_else(|| "lpoco".into()),
            adb: Contract::ADB.get_from(r).unwrap_or_else(|| "adb".into()),
            fastboot: Contract::FASTBOOT
                .get_from(r)
                .unwrap_or_else(|| "fastboot".into()),
            image_a: Contract::IMAGE_A
                .get_from(r)
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join("Projects/walleye-build/images-wdt1/boot-wdt1-a.img")),
            image_b: Contract::IMAGE_B
                .get_from(r)
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join("Projects/walleye-build/images-wdt1/boot-wdt1-b.img")),
            stay: Contract::STAY.get_from(r).unwrap_or(180),
            bite_wait: Contract::BITE_WAIT.get_from(r).unwrap_or(150),
            out_given: Contract::OUT.get_from(r).map(PathBuf::from),
            day: day.to_string(),
            lab,
        }
    }

    /// Where the try called `name` saves what it reads: the author's OUT, else a folder of the lab's material (mode 700, never in git).
    pub fn out_dir(&self, name: &str) -> PathBuf {
        self.out_given.clone().unwrap_or_else(|| {
            let home = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()));
            home.join(format!("lab-material/walleye/{name}-{}", self.day))
        })
    }

    pub fn device_toml(&self) -> PathBuf {
        self.lab.join("device.toml")
    }

    pub fn host_script(&self, name: &str) -> PathBuf {
        self.lab.join("tests/host").join(name)
    }
}

/// The first of `[connection] addresses = ["..", ..]` in a manifest's text.
pub fn first_address(manifest: &str) -> Option<String> {
    let line = manifest
        .lines()
        .find(|l| l.trim_start().starts_with("addresses") && l.contains('['))?;
    let after = line.split_once('[')?.1;
    let inner = after.split_once('"')?.1;
    Some(inner.split_once('"')?.0.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    struct Env(BTreeMap<String, String>);
    impl EnvSource for Env {
        fn get(&self, name: &str) -> Option<String> {
            self.0.get(name).cloned()
        }
        fn names(&self) -> Vec<String> {
            self.0.keys().cloned().collect()
        }
    }

    fn settings(pairs: &[(&str, &str)]) -> Settings {
        let env = Env(pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect());
        let r = Invocation::from_env(&env).resolve(&Contract::records());
        Settings::from_resolution(&r, "DAY")
    }

    #[test]
    fn the_declaration_holds() {
        assert!(
            Contract::problems().is_empty(),
            "{:#?}",
            Contract::problems()
        );
    }

    #[test]
    fn defaults_and_overrides() {
        let s = settings(&[
            ("WALLEYE_TRY_ADDRESS", "10.0.0.1"),
            ("WALLEYE_TRY_WINDOW", "75"),
        ]);
        assert_eq!((s.window, s.min_window, s.port), (75, 40, 23));
        assert_eq!(s.address, "10.0.0.1");
        assert!(s.out.ends_with("lab-material/walleye/touch-DAY"));
        assert!(s.image.ends_with("images-z21/boot-z21.img"));
        assert_eq!(s.lpoco, "lpoco");
    }

    #[test]
    fn the_address_is_the_first_of_the_manifest() {
        let toml = "[connection]\nuser = \"user\"\naddresses = [\"172.16.43.1\", \"x\"]   # note\n";
        assert_eq!(first_address(toml).as_deref(), Some("172.16.43.1"));
        assert_eq!(first_address("[device]\n"), None);
    }
}
