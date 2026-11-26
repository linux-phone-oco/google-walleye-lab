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
//! | the display try's values | `[images.l7]`, `[procedures.display-stripes-on-the-pixel-2] hold`, `module_pause` |
//! | the UFS try's values | `[images.ufs-match]` (`path`, `sha256`), `[procedures.ufs-matched-image-on-the-pixel-2]` `dtb`, `kernel_tree`, `go_item`, `lun`, `load_mb`, `load_limit`, `luns`, `ramp_mb`, `devfreq` |
//! | the install write's values | `[images.alpine-rootfs]` (`path`, `sha256`), `[procedures.install-rootfs-on-the-pixel-2]` `backup`, `stage_dir`, `by_name` |
//! | the root boot's values | `[images.ufs-root]` (`path`, `sha256`), `[procedures.rootfs-boots-on-the-pixel-2]` `go_item`, `identity`, `ssh_user`, `ssh_port`, `bootchain`, `by_name`, `luns`, `fixtures` |
//! | the rw write's values | `[images.ufs-root-booted-slot-production-rw]` (`path`, `sha256`; named by `image`), `[procedures.rw-write-on-the-pixel-2]` `image`, `go_item`, `identity`, `ssh_user`, `ssh_port`, `bootchain`, `by_name`, `luns`, `write_mib` |
//! | the display-at-boot try's values | `[images.b2]` (`path`, `sha256`), `[procedures.display-at-boot-on-the-pixel-2] modules, hold` |
//! | the Alpine-in-RAM login try's values | `[procedures.alpine-login-on-the-glass-on-the-pixel-2]` `image` (the name of its `[images.NAME]`: `path`, `sha256`), `delay`, `modules`, `identity`, `ssh_user`, `ssh_port` |
//! | the display delay bisect's values | `[images.b-d5]` ... `[images.b-d50]` (or the images `image_prefix` names: `b-s` and delays 5 give `[images.b-s5]`) (`path`, `sha256`), `[procedures.display-delay-bisect-on-the-pixel-2] delays`, `image_prefix`, `modules` |

use lpoco_attended::prelude::{dev, proc};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The procedure names of the tries: the entry files and the `[procedures.NAME]` tables agree on them.
pub const TOUCH_PROCEDURE: &str = "touch events on z21";
pub const PMIC_PROCEDURE: &str = "PMIC watchdog v1 on the Pixel 2";
pub const DISPLAY_PROCEDURE: &str = "display stripes on the Pixel 2";
pub const DISPLAY_BOOT_PROCEDURE: &str = "display at boot on the Pixel 2";
pub const UFS_PROCEDURE: &str = "UFS matched image on the Pixel 2";
pub const DISPLAY_DELAY_PROCEDURE: &str = "display delay bisect on the Pixel 2";
pub const INSTALL_PROCEDURE: &str = "install rootfs on the Pixel 2";
pub const REHEARSE_INSTALL_PROCEDURE: &str = "rehearse install rootfs on the Pixel 2";
pub const ROOTFS_BOOT_PROCEDURE: &str = "rootfs boots on the Pixel 2";
pub const REHEARSE_ROOTFS_BOOT_PROCEDURE: &str = "rehearse rootfs boots on the Pixel 2";
pub const RW_WRITE_PROCEDURE: &str = "rw write on the Pixel 2";
pub const REHEARSE_RW_WRITE_PROCEDURE: &str = "rehearse rw write on the Pixel 2";
pub const WDT0_PROCEDURE: &str = "wdt0 state on the Pixel 2";
pub const REHEARSE_WDT0_PROCEDURE: &str = "rehearse wdt0 state on the Pixel 2";
pub const CMDLINE_CUT_PROCEDURE: &str = "cmdline cut c1 on the Pixel 2";
pub const REHEARSE_CMDLINE_CUT_PROCEDURE: &str = "rehearse cmdline cut c1 on the Pixel 2";
pub const REBOOT_TWICE_PROCEDURE: &str = "fastboot reboot twice on the Pixel 2";
pub const REHEARSE_REBOOT_TWICE_PROCEDURE: &str = "rehearse fastboot reboot twice on the Pixel 2";
pub const SLPI_BOOT_PROCEDURE: &str = "slpi boot on the Pixel 2";
pub const REHEARSE_SLPI_BOOT_PROCEDURE: &str = "rehearse slpi boot on the Pixel 2";
pub const RAMROOT_PROCEDURE: &str = "alpine login on the glass on the Pixel 2";
pub const FLASH_BOOT_PROCEDURE: &str = "install boot on the Pixel 2";
pub const REHEARSE_FLASH_BOOT_PROCEDURE: &str = "rehearse install boot on the Pixel 2";
pub const TRANSPORT_PROCEDURE: &str = "transport on the Pixel 2";
pub const REHEARSE_TRANSPORT_PROCEDURE: &str = "rehearse transport on the Pixel 2";

/// The image of the root boot: the name of its `[images.NAME]` table.
pub const ROOT_IMAGE: &str = "ufs-root";
/// The `[images.NAME]` row of the static reboot tool the way back pushes to the phone (path under the images folder, sha256).
pub const REBOOT_TOOL: &str = "walleye-reboot";

/// Which try the settings are read for: each reads only its own procedure's values and images.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Try {
    Touch,
    Pmic,
    Display,
    DisplayBoot,
    DisplayDelay,
    Ufs,
    Install,
    RootfsBoot,
    RwWrite,
    Wdt0,
    CmdlineCut,
    RebootTwice,
    SlpiBoot,
    Ramroot,
    FlashBoot,
    Transport,
}

/// One image of the delay bisect: the seconds of uptime its init waits, its file and the sha256 `device.toml` records.
#[derive(Clone, Debug, PartialEq)]
pub struct DelayImage {
    /// the image's name in `[images.NAME]`: the procedure's `image_prefix` and the delay (b-d5, b-s5)
    pub name: String,
    pub delay: u64,
    pub path: PathBuf,
    pub sha256: String,
}

/// The settings of one try. The fields of the other try stay empty (zero, no path).
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub lab: PathBuf,
    pub image: PathBuf,
    /// the `[images.NAME]` this try boots, when `image ensure` may replace a stale build file (empty: no ensure)
    pub image_name: String,
    pub out: PathBuf,
    pub window: u64,
    pub min_window: u64,
    pub device_wait: u64,
    pub back_wait: u64,
    /// how long stage 2 waits, sending the phone nothing, for Android to answer adb after the new slot gave no ssh: `fallback_wait` of the stage 2 procedure's table
    pub fallback_wait: u64,
    /// the rw write: how many MiB of zeros the one bounded write puts in its test file (`write_mib` of the rw write procedure's table; 1 to `rw_write::MAX_WRITE_MIB`)
    pub write_mib: u64,
    /// the words that pick the phone's lines out of the lab host's kernel log (`3-2`: the port, which also picks the driver-bind lines (cdc_ether 3-2:1.0) the phone is cabled to): `usb_log_match` of the stage 2 procedure's table
    pub usb_log_match: String,
    /// the lowest battery voltage (mV) a try may start at: `[health] battery_floor_mv`
    pub battery_floor_mv: u64,
    pub address: String,
    pub port: u16,
    pub lpoco: String,
    /// the boot-image tool (`lpoco-bringup image swap` makes the run image: the declared one with only its command line changed)
    pub bringup: String,
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
    /// the display delay bisect: the delays (seconds of uptime) in the order they are tried, each with its image and the sha256 device.toml records
    pub delays: Vec<u64>,
    pub delay_images: Vec<DelayImage>,
    /// the UFS try: the dtb the matched image was built with, the kernel tree it was built from, the item the author's go is recorded on
    pub dtb: PathBuf,
    pub kernel_tree: PathBuf,
    pub go_item: String,
    /// where the spent gos are kept: a go a run used is refused to the next (`ufs::go_gate`); a test points it at its own folder
    pub go_state: PathBuf,
    /// the UFS try's read of the card: the disk it reads (`lun`, one lower-case letter after `sd`), how many MiB at most (`load_mb`), the most seconds it may take (`load_limit`), the disks the card has (`luns`)
    pub lun: String,
    pub load_mb: u64,
    pub load_limit: u64,
    /// the most seconds one gear read may take (`gear_wait`): a sysfs read can wait on the link's power management as the load ends
    pub gear_wait: u64,
    pub luns: u64,
    /// the ramped read (`ramp_mb`, "64,256,1024"): the MiB of each step, rising, the last is `load_mb`; the devfreq node of the UFS host (`devfreq`) is held at 50 MHz first
    pub ramp_mb: Vec<u64>,
    pub devfreq: String,
    /// the install write: the folder of the slots' read-back (the way back), the folder on the phone the image is staged in,
    /// the folder of the phone's partition names
    pub backup: PathBuf,
    pub stage_dir: String,
    pub by_name: String,
    /// the root's boot try: the key the lab logs in with, the user and port of the phone's sshd, the stored boot chain it is compared with
    pub identity: PathBuf,
    pub ssh_user: String,
    pub ssh_port: u16,
    pub bootchain: PathBuf,
    /// the root boot only: the folder the capture step writes our kernel's text into as fixtures with provenance (item walleye-kernel-text-capture-procedure)
    pub fixtures: PathBuf,
    /// the way back's tool: the static walleye-reboot the lab pushes to the phone's tmpfs, and the sha256 it must have
    pub reboot_tool: PathBuf,
    pub reboot_tool_sha256: String,
    /// stage 2: the partitions the flash may name (`boot_a boot_b`; the run picks the one that is not Android's) and the command that takes our slot's root to the bootloader
    pub flash_allow: String,
    pub to_bootloader: String,
    /// where a try saves what it reads when a test named one, by the try's name
    pub out_given: Option<PathBuf>,
    pub day: String,
}

impl Settings {
    /// The touch try's settings, read from the device's `device.toml` (call after `procedure(...)`).
    pub fn for_touch() -> Settings {
        Settings::read_linked(Try::Touch, &|k| dev(k).to_string(), &|k| {
            proc(k).to_string()
        })
    }

    /// The display try's settings, read from the device's `device.toml` (call after `procedure(...)`).
    pub fn for_display() -> Settings {
        Settings::read_linked(Try::Display, &|k| dev(k).to_string(), &|k| {
            proc(k).to_string()
        })
    }

    /// The display-at-boot try's settings, read from the device's `device.toml` (call after `procedure(...)`).
    pub fn for_display_boot() -> Settings {
        Settings::read_linked(Try::DisplayBoot, &|k| dev(k).to_string(), &|k| {
            proc(k).to_string()
        })
    }

    /// The display delay bisect's settings, read from the device's `device.toml` (call after `procedure(...)`).
    pub fn for_display_delay() -> Settings {
        Settings::read_linked(Try::DisplayDelay, &|k| dev(k).to_string(), &|k| {
            proc(k).to_string()
        })
    }

    /// The UFS try's settings, read from the device's `device.toml` (call after `procedure(...)`).
    pub fn for_ufs() -> Settings {
        Settings::read_linked(Try::Ufs, &|k| dev(k).to_string(), &|k| proc(k).to_string())
    }

    /// The install write's settings, read from the device's `device.toml` (call after `procedure(...)`).
    pub fn for_install() -> Settings {
        Settings::read_linked(Try::Install, &|k| dev(k).to_string(), &|k| {
            proc(k).to_string()
        })
    }

    /// The root boot's rehearsal reads the boot's own keys, from its own table (call after `procedure(...)`).
    pub fn for_rehearse_rootfs_boot() -> Settings {
        let mut s = Settings::for_rootfs_boot();
        s.out = s.out_dir("rootfs-boots-rehearsal"); // never the real boot's folder: its boot chain read-back refuses a folder with files
        s
    }

    /// The root's boot try's settings, read from the device's `device.toml` (call after `procedure(...)`).
    pub fn for_rootfs_boot() -> Settings {
        Settings::read_linked(Try::RootfsBoot, &|k| dev(k).to_string(), &|k| {
            proc(k).to_string()
        })
    }

    /// The WDT0 try's settings (item walleye-wdt0-try-procedure), read from the device's `device.toml` (call after `procedure(...)`).
    pub fn for_wdt0() -> Settings {
        Settings::read_linked(Try::Wdt0, &|k| dev(k).to_string(), &|k| proc(k).to_string())
    }

    /// The WDT0 try's rehearsal reads the try's own keys, from its own table, and keeps its own folder (the real run's refuses a folder with files).
    pub fn for_rehearse_wdt0() -> Settings {
        let mut s = Settings::for_wdt0();
        s.out = s.out_dir("wdt0-state-rehearsal");
        s
    }

    /// Check C1's settings (item walleye-cmdline-cut-check-slot-a-procedure), read from the device's `device.toml` (call after `procedure(...)`).
    pub fn for_cmdline_cut() -> Settings {
        Settings::read_linked(Try::CmdlineCut, &|k| dev(k).to_string(), &|k| {
            proc(k).to_string()
        })
    }

    /// Check C1's rehearsal reads the check's own keys, from its own table, and keeps its own folder (the real run's refuses a folder with files).
    pub fn for_rehearse_cmdline_cut() -> Settings {
        let mut s = Settings::for_cmdline_cut();
        s.out = s.out_dir("cmdline-cut-c1-rehearsal");
        s
    }

    /// The timed fastboot excursion's settings (item walleye-fastboot-reboot-twice-procedure), read from the device's `device.toml` (call after `procedure(...)`).
    pub fn for_reboot_twice() -> Settings {
        Settings::read_linked(Try::RebootTwice, &|k| dev(k).to_string(), &|k| {
            proc(k).to_string()
        })
    }

    /// The excursion's rehearsal reads the excursion's own keys, from its own table, and keeps its own folder (the real run's refuses a folder with files).
    pub fn for_rehearse_reboot_twice() -> Settings {
        let mut s = Settings::for_reboot_twice();
        s.out = s.out_dir("fastboot-reboot-twice-rehearsal");
        s
    }

    /// The transport run's settings (item walleye-transport-procedure), read from the device's `device.toml` (call after `procedure(...)`).
    pub fn for_transport() -> Settings {
        Settings::read_linked(Try::Transport, &|k| dev(k).to_string(), &|k| {
            proc(k).to_string()
        })
    }

    /// The transport rehearsal keeps its own folder (the real run's refuses a folder with files).
    pub fn for_rehearse_transport() -> Settings {
        let mut s = Settings::for_transport();
        s.out = s.out_dir("transport-rehearsal");
        s
    }

    /// The SLPI boot's settings (item walleye-slpi-boot-procedure), read from the device's `device.toml` (call after `procedure(...)`).
    pub fn for_slpi_boot() -> Settings {
        Settings::read_linked(Try::SlpiBoot, &|k| dev(k).to_string(), &|k| {
            proc(k).to_string()
        })
    }

    /// The SLPI boot's rehearsal reads the boot's own keys, from its own table, and keeps its own folder (the real run's refuses a folder with files).
    pub fn for_rehearse_slpi_boot() -> Settings {
        let mut s = Settings::for_slpi_boot();
        s.out = s.out_dir("slpi-boot-rehearsal");
        s
    }

    /// The rw write's settings, read from the device's `device.toml` (call after `procedure(...)`).
    pub fn for_rw_write() -> Settings {
        Settings::read_linked(Try::RwWrite, &|k| dev(k).to_string(), &|k| {
            proc(k).to_string()
        })
    }

    /// The rw write's rehearsal reads the write's own keys, from its own table, and keeps its own folder (the real run's refuses a folder with files).
    pub fn for_rehearse_rw_write() -> Settings {
        let mut s = Settings::for_rw_write();
        s.out = s.out_dir("rw-write-rehearsal");
        s
    }

    /// Install stage 2's settings (flash the boot partition of the slot Android is not on, set_active it, the way back), read from the device's `device.toml`.
    pub fn for_flash_boot() -> Settings {
        Settings::read_linked(Try::FlashBoot, &|k| dev(k).to_string(), &|k| {
            proc(k).to_string()
        })
    }

    /// The rehearsal of stage 2 reads the command's own keys, from its own table, and keeps its own folder.
    pub fn for_rehearse_flash_boot() -> Settings {
        let mut s = Settings::for_flash_boot();
        s.out = s.out_dir("install-boot-rehearsal");
        s
    }

    /// The Alpine-in-RAM login try's settings, read from the device's `device.toml` (call after `procedure(...)`).
    pub fn for_ramroot() -> Settings {
        Settings::read_linked(Try::Ramroot, &|k| dev(k).to_string(), &|k| {
            proc(k).to_string()
        })
    }

    /// The PMIC try's settings, read from the device's `device.toml` (call after `procedure(...)`).
    pub fn for_pmic() -> Settings {
        Settings::read_linked(Try::Pmic, &|k| dev(k).to_string(), &|k| proc(k).to_string())
    }

    /// Read every key through `dev` (a dotted key of a table) and `proc` (a key of the try's own table); a value
    /// of the wrong kind stops the program naming the key.
    /// `read`, then the images folder linked as AGENTS.md says a fresh worktree needs (see `link_images`): what the `for_*` constructors call.
    fn read_linked(
        which: Try,
        dev: &dyn Fn(&str) -> String,
        proc: &dyn Fn(&str) -> String,
    ) -> Settings {
        let s = Settings::read(which, dev, proc);
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_default();
        if let Err(e) = link_images(&s.lab, &dev("images.dir"), &home) {
            panic!("{e}");
        }
        s
    }

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
            bringup: "lpoco-bringup".into(),
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
            image_name: String::new(),
            delays: Vec::new(),
            delay_images: Vec::new(),
            dtb: PathBuf::new(),
            kernel_tree: PathBuf::new(),
            go_item: String::new(),
            go_state: crate::ufs::go_state_dir(),
            lun: String::new(),
            load_mb: 0,
            load_limit: 0,
            gear_wait: 0,
            fallback_wait: 0,
            write_mib: 0,
            usb_log_match: String::new(),
            luns: 0,
            ramp_mb: Vec::new(),
            devfreq: String::new(),
            backup: PathBuf::new(),
            stage_dir: String::new(),
            by_name: String::new(),
            identity: PathBuf::new(),
            ssh_user: String::new(),
            ssh_port: 0,
            bootchain: PathBuf::new(),
            fixtures: PathBuf::new(),
            reboot_tool: PathBuf::new(),
            reboot_tool_sha256: String::new(),
            flash_allow: String::new(),
            to_bootloader: String::new(),
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
                s.image = image("l7");
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
                s.lun = proc("lun");
                assert!(
                    crate::ufs::load::lun_is_a_disk(&s.lun),
                    "lun in device.toml is not a disk name like sda: {:?}",
                    s.lun
                );
                s.load_mb = number("load_mb", proc("load_mb"));
                s.load_limit = number("load_limit", proc("load_limit"));
                s.gear_wait = number("gear_wait", proc("gear_wait"));
                s.luns = number("luns", proc("luns"));
                s.ramp_mb = proc("ramp_mb")
                    .split(',')
                    .map(|n| number("ramp_mb", n.trim().to_string()))
                    .collect();
                s.devfreq = proc("devfreq");
                assert!(
                    crate::ufs::load::is_a_node_name(&s.devfreq),
                    "devfreq in device.toml is not a node name like 1da4000.ufshc: {:?}",
                    s.devfreq
                );
                assert!(
                    s.ramp_mb.windows(2).all(|w| w[0] < w[1])
                        && s.ramp_mb.last() == Some(&s.load_mb),
                    "ramp_mb in device.toml rises and ends at load_mb: {:?}",
                    s.ramp_mb
                );
                assert!(
                    s.load_mb >= 1 && s.load_limit >= 1 && s.gear_wait >= 1 && s.luns >= 1,
                    "load_mb, load_limit, gear_wait and luns in device.toml are at least 1"
                );
                s.out = s.out_dir("ufs");
            }
            Try::Install => {
                s.image = image("alpine-rootfs");
                s.image_sha256 = dev("images.alpine-rootfs.sha256");
                s.backup = home_path(&proc("backup"));
                s.stage_dir = proc("stage_dir");
                s.by_name = proc("by_name");
                assert!(
                    s.stage_dir.starts_with('/') && s.by_name.starts_with('/'),
                    "stage_dir and by_name in device.toml are absolute paths on the phone"
                );
                s.out = s.out_dir("install");
            }
            Try::RootfsBoot => {
                s.image = image(ROOT_IMAGE);
                s.image_name = ROOT_IMAGE.into();
                s.image_sha256 = dev(&format!("images.{ROOT_IMAGE}.sha256"));
                s.go_item = proc("go_item");
                s.identity = home_path(&proc("identity"));
                s.ssh_user = proc("ssh_user");
                s.ssh_port = u16::try_from(number("ssh_port", proc("ssh_port")))
                    .expect("ssh_port in device.toml is a port number");
                s.bootchain = home_path(&proc("bootchain"));
                s.by_name = proc("by_name");
                s.luns = number("luns", proc("luns"));
                s.reboot_tool = image(REBOOT_TOOL);
                s.reboot_tool_sha256 = dev(&format!("images.{REBOOT_TOOL}.sha256"));
                assert!(
                    s.by_name.starts_with('/') && (1..=26).contains(&s.luns),
                    "by_name in device.toml is an absolute path on the phone, luns is 1 to 26"
                );
                s.fixtures = lab.join(proc("fixtures"));
                s.out = s.out_dir("rootfs-boots");
            }
            Try::RwWrite => {
                // the rw write: the image is named by its [images.NAME] row (the sha256 the go names), the root is the system partition of the slot Android is not on
                let name = proc("image");
                s.image = image(&name);
                s.image_name = name.clone();
                s.image_sha256 = dev(&format!("images.{name}.sha256"));
                s.go_item = proc("go_item");
                s.identity = home_path(&proc("identity"));
                s.ssh_user = proc("ssh_user");
                s.ssh_port = u16::try_from(number("ssh_port", proc("ssh_port")))
                    .expect("ssh_port in device.toml is a port number");
                s.bootchain = home_path(&proc("bootchain"));
                s.by_name = proc("by_name");
                s.luns = number("luns", proc("luns"));
                s.write_mib = number("write_mib", proc("write_mib"));
                s.reboot_tool = image(REBOOT_TOOL);
                s.reboot_tool_sha256 = dev(&format!("images.{REBOOT_TOOL}.sha256"));
                assert!(
                    s.by_name.starts_with('/') && (1..=26).contains(&s.luns),
                    "by_name in device.toml is an absolute path on the phone, luns is 1 to 26"
                );
                assert!(
                    (1..=crate::rw_write::MAX_WRITE_MIB).contains(&s.write_mib),
                    "write_mib in device.toml is 1 to {} (the bound of the one write)",
                    crate::rw_write::MAX_WRITE_MIB
                );
                s.out = s.out_dir("rw-write");
            }
            Try::Wdt0 => {
                // the WDT0 try: the root boot's keys, the image named by its [images.NAME] row (the sha256 the go names)
                let name = proc("image");
                s.image = image(&name);
                s.image_name = name.clone();
                s.image_sha256 = dev(&format!("images.{name}.sha256"));
                s.go_item = proc("go_item");
                s.identity = home_path(&proc("identity"));
                s.ssh_user = proc("ssh_user");
                s.ssh_port = u16::try_from(number("ssh_port", proc("ssh_port")))
                    .expect("ssh_port in device.toml is a port number");
                s.bootchain = home_path(&proc("bootchain"));
                s.by_name = proc("by_name");
                s.luns = number("luns", proc("luns"));
                s.reboot_tool = image(REBOOT_TOOL);
                s.reboot_tool_sha256 = dev(&format!("images.{REBOOT_TOOL}.sha256"));
                assert!(
                    s.by_name.starts_with('/') && (1..=26).contains(&s.luns),
                    "by_name in device.toml is an absolute path on the phone, luns is 1 to 26"
                );
                s.out = s.out_dir("wdt0-state");
            }
            Try::CmdlineCut => {
                // check C1: the debug image named by its [images.NAME] row (the sha256 the go names); no ssh - the image's telnet is read - and the way back's reboot tool
                let name = proc("image");
                s.image = image(&name);
                s.image_sha256 = dev(&format!("images.{name}.sha256"));
                s.go_item = proc("go_item");
                s.bootchain = home_path(&proc("bootchain"));
                s.by_name = proc("by_name");
                s.luns = number("luns", proc("luns"));
                s.reboot_tool = image(REBOOT_TOOL);
                s.reboot_tool_sha256 = dev(&format!("images.{REBOOT_TOOL}.sha256"));
                assert!(
                    s.by_name.starts_with('/') && (1..=26).contains(&s.luns),
                    "by_name in device.toml is an absolute path on the phone, luns is 1 to 26"
                );
                s.out = s.out_dir("cmdline-cut-c1");
            }
            Try::Transport => {
                // the transport run: read only, no image, no go, no ssh, no card read - only the shared keys (adb, fastboot, back_wait, the serial)
                s.out = s.out_dir("transport");
            }
            Try::RebootTwice => {
                // the timed fastboot excursion: no image, no go, no ssh - reboots out of the bootloader, the card and the boot chain read before and after
                s.bootchain = home_path(&proc("bootchain"));
                s.by_name = proc("by_name");
                s.luns = number("luns", proc("luns"));
                assert!(
                    s.by_name.starts_with('/') && (1..=26).contains(&s.luns),
                    "by_name in device.toml is an absolute path on the phone, luns is 1 to 26"
                );
                s.out = s.out_dir("fastboot-reboot-twice");
            }
            Try::SlpiBoot => {
                // the SLPI boot: the image named by its [images.NAME] row, booted from RAM; no ssh, no go (nothing is flashed, the image is UFS-off by the gate)
                let name = proc("image");
                s.image = image(&name);
                s.image_name = name.clone();
                s.image_sha256 = dev(&format!("images.{name}.sha256"));
                s.bootchain = home_path(&proc("bootchain"));
                s.by_name = proc("by_name");
                s.luns = number("luns", proc("luns"));
                assert!(
                    s.by_name.starts_with('/') && (1..=26).contains(&s.luns),
                    "by_name in device.toml is an absolute path on the phone, luns is 1 to 26"
                );
                s.out = s.out_dir("slpi-boot");
            }
            Try::Ramroot => {
                let name = proc("image");
                s.image = image(&name);
                s.image_sha256 = dev(&format!("images.{name}.sha256"));
                s.delays = vec![number("delay", proc("delay"))];
                s.modules = number("modules", proc("modules"));
                s.identity = home_path(&proc("identity"));
                s.ssh_user = proc("ssh_user");
                s.ssh_port = u16::try_from(number("ssh_port", proc("ssh_port")))
                    .expect("ssh_port in device.toml is a port number");
                s.out = s.out_dir("alpine-login");
            }
            Try::FlashBoot => {
                // stage 2: the boot image is named by its [images.NAME] row, never typed here (the ufs-root image is rebuilt with a new sha256 by other items)
                let name = proc("image");
                s.image = image(&name);
                s.image_sha256 = dev(&format!("images.{name}.sha256"));
                s.go_item = proc("go_item");
                s.identity = home_path(&proc("identity"));
                s.ssh_user = proc("ssh_user");
                s.ssh_port = u16::try_from(number("ssh_port", proc("ssh_port")))
                    .expect("ssh_port in device.toml is a port number");
                s.bootchain = home_path(&proc("bootchain"));
                s.backup = home_path(&proc("backup"));
                s.by_name = proc("by_name");
                s.luns = number("luns", proc("luns"));
                s.flash_allow = proc("flash_allow");
                s.to_bootloader = proc("to_bootloader");
                s.fallback_wait = number("fallback_wait", proc("fallback_wait"));
                s.usb_log_match = proc("usb_log_match");
                assert!(
                    !s.usb_log_match.trim().is_empty(),
                    "usb_log_match in device.toml names the host port of the phone"
                );
                s.reboot_tool = image(REBOOT_TOOL);
                s.reboot_tool_sha256 = dev(&format!("images.{REBOOT_TOOL}.sha256"));
                assert!(
                    s.by_name.starts_with('/') && (1..=26).contains(&s.luns),
                    "by_name in device.toml is an absolute path on the phone, luns is 1 to 26"
                );
                assert!(
                    !s.flash_allow.trim().is_empty() && !s.to_bootloader.trim().is_empty(),
                    "flash_allow and to_bootloader in device.toml are not empty"
                );
                assert!(
                    s.to_bootloader == crate::reboot_tool::BOOTLOADER_COMMAND,
                    "to_bootloader in device.toml is the reboot tool's command (busybox reboot drops the bootloader argument)"
                );
                s.out = s.out_dir("install-boot");
            }
            Try::DisplayBoot => {
                s.image = image("b2");
                s.image_sha256 = dev("images.b2.sha256");
                s.hold = number("hold", proc("hold"));
                s.modules = number("modules", proc("modules"));
                s.out = s.out_dir("display-boot");
            }
            Try::DisplayDelay => {
                s.delays = proc("delays")
                    .split_whitespace()
                    .map(|n| {
                        n.parse().unwrap_or_else(|_| {
                            panic!("delays in device.toml is a list of whole seconds: {n:?}")
                        })
                    })
                    .collect();
                assert!(!s.delays.is_empty(), "delays in device.toml names no delay");
                let prefix = proc("image_prefix");
                s.delay_images = s
                    .delays
                    .iter()
                    .map(|&n| DelayImage {
                        name: format!("{prefix}{n}"),
                        delay: n,
                        path: image(&format!("{prefix}{n}")),
                        sha256: dev(&format!("images.{prefix}{n}.sha256")),
                    })
                    .collect();
                s.modules = number("modules", proc("modules"));
                s.out = s.out_dir(&format!("display-delay-{prefix}"));
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
        Settings::stand_in_with(which, &[])
    }

    /// The stand-in keys with some values replaced.
    #[cfg(test)]
    pub fn stand_in_with(which: Try, over: &[(&'static str, &'static str)]) -> Settings {
        let mut keys: std::collections::BTreeMap<&str, &str> = [
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
            ("images.l7.path", "boot-l7.img"),
            ("hold", "60"),
            ("module_pause", "5"),
            ("images.b2.path", "boot-b2.img"),
            ("images.b2.sha256", "e4c8"),
            ("modules", "13"),
            ("delays", "5 15 25 35 50"),
            ("image_prefix", "b-d"),
            ("images.b-d5.path", "boot-b-d5.img"),
            ("images.b-d5.sha256", "d5d5"),
            ("images.b-d15.path", "boot-b-d15.img"),
            ("images.b-d15.sha256", "d15d"),
            ("images.b-d25.path", "boot-b-d25.img"),
            ("images.b-d25.sha256", "d25d"),
            ("images.b-d35.path", "boot-b-d35.img"),
            ("images.b-d35.sha256", "d35d"),
            ("images.b-d50.path", "boot-b-d50.img"),
            ("images.b-d50.sha256", "d50d"),
            ("images.ufs-match.path", "NOT-TO-BOOT-x/NOT-TO-BOOT-ufs.img"),
            ("images.ufs-match.sha256", "4610"),
            ("dtb", "NOT-TO-BOOT-x/ufs-match.dtb"),
            ("kernel_tree", "~/tree"),
            ("go_item", "walleye-try-ufs-match"),
            ("lun", "sda"),
            ("load_mb", "4"),
            ("load_limit", "20"),
            ("gear_wait", "1"),
            ("luns", "6"),
            ("ramp_mb", "1, 2, 4"),
            ("devfreq", "1da4000.ufshc"),
            ("images.ufs-root.path", "NOT-TO-BOOT-x/NOT-TO-BOOT-root.img"),
            ("images.ufs-root.sha256", "35b6"),
            (
                "images.walleye-reboot.path",
                "walleye-reboot/walleye-reboot",
            ),
            ("images.walleye-reboot.sha256", "r3b0"),
            ("identity", "~/key"),
            ("ssh_user", "root"),
            ("ssh_port", "22"),
            ("bootchain", "~/bootchain"),
            ("fixtures", "tests/fixtures/phone-text"),
            ("image", "b-a5"),
            ("delay", "5"),
            ("images.b-a5.path", "images-display-ramroot/boot-b-a5.img"),
            ("images.b-a5.sha256", "a5a5"),
            ("images.alpine-rootfs.path", "images-rootfs/rootfs.ext4"),
            ("images.alpine-rootfs.sha256", "a1b2"),
            ("backup", "~/backup"),
            ("stage_dir", "/data/local/tmp"),
            ("by_name", "/dev/block/by-name"),
            ("flash_allow", "boot_a boot_b"),
            ("to_bootloader", "/tmp/walleye-reboot --command bootloader"),
            ("fallback_wait", "420"),
            ("write_mib", "16"),
            ("usb_log_match", "3-2"),
        ]
        .into_iter()
        .collect();
        keys.extend(over.iter().copied());
        let get = |k: &str| keys[k].to_string();
        let mut s = Settings::read(which, &get, &get);
        // a test never spends a go of the lab machine: each stand-in has its own folder of spent gos
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        s.go_state = std::env::temp_dir().join(format!("wt-spent-gos-{}-{n}", std::process::id()));
        s
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
            (Try::DisplayDelay, DISPLAY_DELAY_PROCEDURE),
            (Try::Install, INSTALL_PROCEDURE),
            (Try::Install, REHEARSE_INSTALL_PROCEDURE),
            (Try::RootfsBoot, ROOTFS_BOOT_PROCEDURE),
            (Try::RootfsBoot, REHEARSE_ROOTFS_BOOT_PROCEDURE),
            (Try::RwWrite, RW_WRITE_PROCEDURE),
            (Try::RwWrite, REHEARSE_RW_WRITE_PROCEDURE),
            (Try::Wdt0, WDT0_PROCEDURE),
            (Try::Wdt0, REHEARSE_WDT0_PROCEDURE),
            (Try::CmdlineCut, CMDLINE_CUT_PROCEDURE),
            (Try::CmdlineCut, REHEARSE_CMDLINE_CUT_PROCEDURE),
            (Try::RebootTwice, REBOOT_TWICE_PROCEDURE),
            (Try::RebootTwice, REHEARSE_REBOOT_TWICE_PROCEDURE),
            (Try::SlpiBoot, SLPI_BOOT_PROCEDURE),
            (Try::SlpiBoot, REHEARSE_SLPI_BOOT_PROCEDURE),
            (Try::Ramroot, RAMROOT_PROCEDURE),
            (Try::FlashBoot, FLASH_BOOT_PROCEDURE),
            (Try::FlashBoot, REHEARSE_FLASH_BOOT_PROCEDURE),
            (Try::Transport, TRANSPORT_PROCEDURE),
            (Try::Transport, REHEARSE_TRANSPORT_PROCEDURE),
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

/// A fresh worktree has no `build/images` (it is ignored, outside git): the lab's images live in `~/Projects/walleye-build`, which
/// `images.dir` of device.toml names relative to the worktree. When the folder is absent and `HOME/Projects/walleye-build` is there,
/// `images.dir` becomes a symlink to it (inside the worktree, nothing written outside it). Present (a folder or a link that
/// resolves): left as it is. Where the target is absent (a CI runner), nothing is done: the image checks say what they miss.
/// A link in the way that points nowhere, or a file, is refused with the command that fixes it.
pub fn link_images(lab: &Path, dir: &str, home: &Path) -> Result<(), String> {
    let at = lab.join(dir);
    let target = home.join("Projects/walleye-build");
    if at.exists() {
        return Ok(());
    }
    if at.symlink_metadata().is_ok() {
        return Err(format!(
            "{} is a link that points nowhere (or a file): fix it with `ln -sfn {} {}`",
            at.display(),
            target.display(),
            at.display()
        ));
    }
    if !target.is_dir() {
        return Ok(());
    }
    if let Some(parent) = at.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot make {}: {e}", parent.display()))?;
    }
    std::os::unix::fs::symlink(&target, &at).map_err(|e| {
        format!(
            "cannot link {} to {}: {e}; fix it with `ln -sfn {} {}`",
            at.display(),
            target.display(),
            target.display(),
            at.display()
        )
    })
}

#[cfg(test)]
mod link_tests {
    use super::link_images;
    use std::fs;

    fn scratch(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("walleye-link-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    /// AGENTS.md and the session log of 2026-11-24 (run 6, keeper PR 964): a fresh worktree needs build/images linked to ~/Projects/walleye-build.
    #[test]
    fn a_fresh_worktree_gets_the_link_to_walleye_build_itself() {
        let (lab, home) = (scratch("lab"), scratch("home"));
        fs::create_dir_all(home.join("Projects/walleye-build/images-z21")).unwrap();
        link_images(&lab, "build/images", &home).unwrap();
        let at = lab.join("build/images");
        assert_eq!(
            fs::read_link(&at).unwrap(),
            home.join("Projects/walleye-build")
        );
        assert!(at.join("images-z21").is_dir());
        // present: left alone
        link_images(&lab, "build/images", &home).unwrap();
        assert_eq!(
            fs::read_link(&at).unwrap(),
            home.join("Projects/walleye-build")
        );
    }

    #[test]
    fn a_folder_already_there_is_left_and_a_dead_link_is_refused_with_the_fix() {
        let (lab, home) = (scratch("lab2"), scratch("home2"));
        fs::create_dir_all(home.join("Projects/walleye-build")).unwrap();
        fs::create_dir_all(lab.join("build/images")).unwrap();
        link_images(&lab, "build/images", &home).unwrap();
        assert!(!lab.join("build/images").is_symlink());
        let (lab, home) = (scratch("lab3"), scratch("home3"));
        fs::create_dir_all(home.join("Projects/walleye-build")).unwrap();
        fs::create_dir_all(lab.join("build")).unwrap();
        std::os::unix::fs::symlink("/nowhere/at/all", lab.join("build/images")).unwrap();
        let e = link_images(&lab, "build/images", &home).unwrap_err();
        assert!(e.contains("ln -sfn") && e.contains("walleye-build"), "{e}");
    }

    #[test]
    fn without_walleye_build_nothing_is_made() {
        let (lab, home) = (scratch("lab4"), scratch("home4"));
        link_images(&lab, "build/images", &home).unwrap();
        assert!(!lab.join("build").exists());
    }
}
