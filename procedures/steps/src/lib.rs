// SPDX-License-Identifier: MIT OR Apache-2.0
//! The Pixel 2 lab's attended tries. An entry file (procedures/*.rs, run by rust-script) reads like a
//! checklist: `use lpoco_attended::prelude::*;` for the runner, `use walleye_tries::touch::Touch;` for
//! this phone's steps. What a step does, parses and retries lives here, tested without a phone.

/// The runner every attended procedure shares, re-exported so its tag is pinned once, in this crate's Cargo.toml.
pub use lpoco_attended;

pub mod android;
pub mod cmdline_cut;
pub mod display;
pub mod display_boot;
pub mod display_delay;
pub mod display_smmu;
#[cfg(test)]
mod fixture;
pub mod flash_boot;
pub mod host;
pub mod install;
pub mod lab;
pub mod modem_boot;
pub mod nfc_boot;
pub mod phone;
pub mod pmic;
pub mod pon_control;
pub mod ramroot;
pub mod reader;
pub mod reboot_tool;
pub mod reboot_twice;
pub mod rootfs_boot;
pub mod rw_write;
pub mod settings;
pub mod slpi_boot;
pub mod touch;
pub mod transport;
pub mod ufs;
pub mod watch;
pub mod wdt0;

#[cfg(test)]
mod tests_never_notify {
    /// `.cargo/config.toml` pins the notify file to nowhere for every `cargo test` here (item walleye-tests-never-reach-the-authors-notify).
    #[test]
    fn the_notify_file_a_test_sees_is_nowhere() {
        let p = std::env::var("LPOCO_ATTENDED_NOTIFY")
            .expect("cargo's [env] pins LPOCO_ATTENDED_NOTIFY");
        assert!(!std::path::Path::new(&p).exists(), "{p} exists");
    }
}
