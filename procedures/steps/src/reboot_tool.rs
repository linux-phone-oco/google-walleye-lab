// SPDX-License-Identifier: MIT OR Apache-2.0
//! The way back's reboot tool, one copy for both stages (items walleye-reboot-reason-restart2-tool, walleye-install-stage2-way-back-restart2):
//! busybox's `reboot bootloader` drops its argument, so the lab pushes its static `walleye-reboot` over ssh into the phone's tmpfs
//! (`/tmp` of the overlay: nothing touches the card) and runs it there; the tool hands `bootloader` to reboot(2) RESTART2.
//! The file is the one device.toml's `[images.walleye-reboot]` names, by sha256.

use crate::host;
use crate::settings::Settings;
use std::path::Path;
use std::process::Command;

/// Where the tool is put on the phone (its tmpfs).
pub const ON_PHONE: &str = "/tmp/walleye-reboot";
/// Puts the standard input on the phone's tmpfs and makes it runnable.
pub const SEND_COMMAND: &str = "cat > /tmp/walleye-reboot && chmod 755 /tmp/walleye-reboot";
/// What the phone runs to restart into its bootloader (device.toml's `to_bootloader` of stage 2 says the same).
pub const BOOTLOADER_COMMAND: &str = "/tmp/walleye-reboot --command bootloader";

/// One command on the phone over ssh, as the declared key, `stdin` (a file) on its standard input when given; what it said, or why it failed.
/// Bounded: `timeout` ends a hung link.
pub fn ssh_from(
    s: &Settings,
    ssh: &str,
    command: &str,
    seconds: u64,
    stdin: Option<&Path>,
) -> Result<String, String> {
    let mut c = Command::new("timeout");
    if let Some(file) = stdin {
        let f = std::fs::File::open(file).map_err(|e| format!("{}: {e}", file.display()))?;
        c.stdin(f);
    }
    c.args([
        "--kill-after=5".to_string(),
        seconds.to_string(),
        ssh.to_string(),
    ])
    .args(crate::rootfs_boot::ssh_args(
        &s.identity,
        &s.ssh_user,
        &s.address,
        s.ssh_port,
        command,
    ));
    let said = host::run(c)?;
    if said.ok {
        Ok(said.text)
    } else {
        Err(format!("ssh: {}", host::last_line(&said.text)))
    }
}

/// The tool is checked against its declared sha256, sent to the phone's tmpfs over ssh, and run with `command` (which must be [`BOOTLOADER_COMMAND`]:
/// a `reboot bootloader` of busybox would end in a plain reboot). An error names the step that failed.
pub fn ask_for_the_bootloader(s: &Settings, ssh: &str, command: &str) -> Result<String, String> {
    if command != BOOTLOADER_COMMAND {
        return Err(format!(
            "the bootloader command `{command}` is not `{BOOTLOADER_COMMAND}`"
        ));
    }
    host::image_has_sha256(&s.reboot_tool, &s.reboot_tool_sha256)
        .map_err(|e| format!("the reboot tool is not the declared one: {e}"))?;
    ssh_from(s, ssh, SEND_COMMAND, 30, Some(&s.reboot_tool))
        .map_err(|e| format!("sending the reboot tool to {ON_PHONE}: {e}"))?;
    ssh_from(s, ssh, command, 10, None)
}

/// The rw root's clean end (item walleye-rw-end-through-openrc-mount-ro): the tool and a marker go to /run/walleye (tmpfs: nothing touches the card) and the phone is asked to end through init
/// (busybox `reboot`: init runs the inittab's `openrc shutdown`, whose level has mount-ro and, last, the walleye-end service that runs the tool and always reboots). Nothing remounts inside the live session.
pub const SHUTDOWN_ON_PHONE: &str = "/run/walleye/walleye-reboot";
/// Puts the standard input at [`SHUTDOWN_ON_PHONE`], makes it runnable and leaves the marker the walleye-end service looks for.
pub const SEND_FOR_SHUTDOWN_COMMAND: &str = "mkdir -p /run/walleye && cat > /run/walleye/walleye-reboot && chmod 755 /run/walleye/walleye-reboot && : > /run/walleye/to-bootloader";
/// Asks init to end: its shutdown level runs, then the service reboots (to the bootloader when the marker and the tool are there).
pub const END_COMMAND: &str = "reboot";

/// The tool is checked against its declared sha256, put with its marker on the phone's /run, and init is asked to end ([`END_COMMAND`]); the answer is not waited for as a judgement
/// (the phone hangs up as it goes down). An error names the step that failed (the way back's old forced path follows in every case).
pub fn ask_for_the_clean_end(s: &Settings, ssh: &str) -> Result<String, String> {
    host::image_has_sha256(&s.reboot_tool, &s.reboot_tool_sha256)
        .map_err(|e| format!("the reboot tool is not the declared one: {e}"))?;
    ssh_from(s, ssh, SEND_FOR_SHUTDOWN_COMMAND, 30, Some(&s.reboot_tool))
        .map_err(|e| format!("sending the reboot tool to {SHUTDOWN_ON_PHONE}: {e}"))?;
    ssh_from(s, ssh, END_COMMAND, 10, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_command_that_is_not_the_tools_is_refused_before_anything_is_sent() {
        let s = Settings::stand_in(crate::settings::Try::FlashBoot);
        for bad in [
            "reboot bootloader",
            "/tmp/walleye-reboot-x",
            "/tmp/walleye-reboot; reboot bootloader",
            "/tmp/walleye-reboot --command poweroff",
        ] {
            let said = ask_for_the_bootloader(&s, "ssh-not-run", bad).unwrap_err();
            assert!(
                said.contains("is not `/tmp/walleye-reboot --command bootloader`"),
                "{said}"
            );
        }
    }
}
