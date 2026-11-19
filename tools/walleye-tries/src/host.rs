// SPDX-License-Identifier: MIT OR Apache-2.0
//! The lab's host scripts (tests/host/*.sh) and programs run as checks: each prints a verdict line and exits 0
//! only on pass. A try runs them, never copies their logic.

use std::path::Path;
use std::process::Command;

/// What a host command printed (standard output and error, trimmed) and whether it exited 0.
pub struct Said {
    pub text: String,
    pub ok: bool,
}

pub fn run(mut cmd: Command) -> Result<Said, String> {
    let what = format!("{cmd:?}");
    let o = cmd
        .output()
        .map_err(|e| format!("cannot run {what}: {e}"))?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
    Ok(Said {
        text: text.trim().to_string(),
        ok: o.status.success(),
    })
}

/// Run `sh SCRIPT ARGS` with the environment given; the script's verdict line when it passes, an error saying
/// what it printed when it does not.
pub fn script(path: &Path, args: &[&str], env: &[(&str, &str)]) -> Result<String, String> {
    let mut c = Command::new("sh");
    c.arg(path).args(args);
    for (k, v) in env {
        c.env(k, v);
    }
    let said = run(c)?;
    if said.ok {
        Ok(last_line(&said.text))
    } else {
        Err(said.text)
    }
}

/// The image is the one built: its sha256 is the one its SHA256SUMS line (any SHA256SUMS* file beside it) records.
pub fn image_is_the_built_one(image: &Path) -> Result<(), String> {
    let dir = image.parent().unwrap_or(Path::new("."));
    let name = image.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let sums = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with("SHA256SUMS"))
        .filter_map(|e| std::fs::read_to_string(e.path()).ok())
        .collect::<Vec<_>>()
        .join("\n");
    let want = sums
        .lines()
        .find(|l| {
            l.split_whitespace()
                .last()
                .map(|f| f.trim_start_matches('*') == name)
                == Some(true)
        })
        .and_then(|l| l.split_whitespace().next())
        .ok_or(format!(
            "no SHA256SUMS line for {name} in {}",
            dir.display()
        ))?
        .to_string();
    let mut c = Command::new("sha256sum");
    c.arg(image);
    let said = run(c)?;
    let got = said
        .text
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string();
    if got == want {
        Ok(())
    } else {
        Err(format!("{name} is {got}, its SHA256SUMS line says {want}"))
    }
}

/// Fastboot lists the phone with this serial; a fastboot that cannot be run is an error, not "not listed".
pub fn in_fastboot(fastboot: &str, serial: &str) -> Result<bool, String> {
    let mut c = Command::new(fastboot);
    c.arg("devices");
    Ok(!serial.is_empty() && run(c)?.text.contains(serial))
}

/// The first `serial = "..."` of the manifest, as the host scripts read it.
pub fn serial_of(manifest: &str) -> Option<String> {
    manifest.lines().find_map(|l| {
        let rest = l.strip_prefix("serial")?.trim_start().strip_prefix('=')?;
        let rest = rest.trim_start().strip_prefix('"')?;
        Some(rest.split_once('"')?.0.to_string())
    })
}

/// The phone is in fastboot: already, or after `adb reboot bootloader` from Android (the phone named by its serial),
/// listed by fastboot within `wait`. Nothing is written to the phone; a phone that answers neither says so.
pub fn to_fastboot(
    adb: &str,
    fastboot: &str,
    serial: &str,
    wait: std::time::Duration,
) -> Result<(), String> {
    if serial.is_empty() {
        return Err("no serial for the phone: it could not be told from another one".into());
    }
    if in_fastboot(fastboot, serial)? {
        return Ok(());
    }
    let mut st = Command::new(adb);
    st.args(["-s", serial, "get-state"]);
    let state = run(st)?;
    if !state.ok || state.text.trim() != "device" {
        return Err(format!(
            "the phone is not in fastboot and adb does not see it on Android (adb: '{}'): it may be on the mainline gadget, unauthorized or offline",
            state.text.trim()
        ));
    }
    let mut c = Command::new(adb);
    c.args(["-s", serial]);
    c.args(["reboot", "bootloader"]);
    let said = run(c)?;
    if !said.ok {
        return Err(format!(
            "the phone is not in fastboot and adb cannot reboot it to the bootloader: {}",
            said.text
        ));
    }
    let until = std::time::Instant::now() + wait;
    while std::time::Instant::now() < until {
        if in_fastboot(fastboot, serial)? {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
    Err(format!(
        "the phone is not in fastboot within {} s of adb reboot bootloader",
        wait.as_secs()
    ))
}

pub fn last_line(text: &str) -> String {
    text.lines().last().unwrap_or("").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_serial_is_the_first_of_the_manifest() {
        let toml =
            "[connection]\nserial = \"FA81\"   # fastboot\n[x]\nserial = \"walleye-first-boot\"\n";
        assert_eq!(serial_of(toml).as_deref(), Some("FA81"));
        assert_eq!(serial_of("[x]\n"), None);
    }

    #[test]
    fn a_passing_script_gives_its_last_line_and_a_failing_one_what_it_said() {
        let d = std::env::temp_dir().join(format!("wt-host-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let ok = d.join("ok.sh");
        std::fs::write(&ok, "echo one; echo \"pass x $1 $WHO\"\n").unwrap();
        let bad = d.join("bad.sh");
        std::fs::write(&bad, "echo \"fail x nope\"; exit 1\n").unwrap();
        assert_eq!(
            script(&ok, &["a"], &[("WHO", "me")]).unwrap(),
            "pass x a me"
        );
        assert_eq!(script(&bad, &[], &[]).unwrap_err(), "fail x nope");
        std::fs::remove_dir_all(d).ok();
    }

    fn stand_in(d: &std::path::Path, adb: &str, fastboot: &str) -> (String, String) {
        std::fs::create_dir_all(d).unwrap();
        let mut out = vec![];
        for (n, body) in [("adb", adb), ("fastboot", fastboot)] {
            let p = d.join(n);
            std::fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
            std::fs::set_permissions(&p, std::os::unix::fs::PermissionsExt::from_mode(0o755))
                .unwrap();
            out.push(p.to_string_lossy().into_owned());
        }
        (out[0].clone(), out[1].clone())
    }

    #[test]
    fn a_phone_in_fastboot_is_left_alone_and_one_on_android_is_sent_there() {
        let d = std::env::temp_dir().join(format!("wt-fb-{}", std::process::id()));
        let w = std::time::Duration::from_secs(5);
        // already listed: adb must not be called (this adb fails)
        let (adb, fb) = stand_in(
            &d,
            "echo called >> \"$0.calls\"; exit 1",
            "echo 'SER1 fastboot'",
        );
        to_fastboot(&adb, &fb, "SER1", w).unwrap();
        assert!(!d.join("adb.calls").exists());
        // on Android: `reboot bootloader` is sent for that serial, then fastboot lists the phone
        let m = d.join("bootloader");
        let (adb, fb) = stand_in(
            &d,
            &format!("case \"$*\" in *get-state*) echo device; exit 0;; esac; echo \"$*\" > \"{0}.args\"; : > \"{0}\"", m.display()),
            &format!("[ -e \"{}\" ] && echo 'SER1 fastboot'; exit 0", m.display()),
        );
        to_fastboot(&adb, &fb, "SER1", w).unwrap();
        assert_eq!(
            std::fs::read_to_string(d.join("bootloader.args"))
                .unwrap()
                .trim(),
            "-s SER1 reboot bootloader"
        );
        // adb sees no phone (the gadget, unauthorized, offline): the error says so
        std::fs::remove_file(&m).unwrap();
        let (adb, fb) = stand_in(&d, "echo 'error: device offline'; exit 1", "exit 0");
        let e = to_fastboot(&adb, &fb, "SER1", w).unwrap_err();
        assert!(
            e.contains("does not see it on Android") && e.contains("device offline"),
            "{e}"
        );
        // another phone's serial in fastboot is not this phone
        let (adb, fb) = stand_in(&d, "exit 1", "echo 'OTHER fastboot'");
        assert!(!in_fastboot(&fb, "SER1").unwrap() && to_fastboot(&adb, &fb, "SER1", w).is_err());
        // adb reboots it but fastboot never lists it: the wait ends with that said
        let (adb, fb) = stand_in(
            &d,
            "case \"$*\" in *get-state*) echo device;; esac; exit 0",
            "exit 0",
        );
        let e = to_fastboot(&adb, &fb, "SER1", std::time::Duration::from_secs(1)).unwrap_err();
        assert!(e.contains("not in fastboot within 1 s"), "{e}");
        // a fastboot that cannot be run errors at once, not after the wait
        let (adb, _) = stand_in(&d, "echo device", "exit 0");
        let t0 = std::time::Instant::now();
        let e = to_fastboot(&adb, "/nonexistent/fastboot", "SER1", w).unwrap_err();
        assert!(
            e.contains("cannot run") && t0.elapsed().as_secs() < 2,
            "{e}"
        );
        // no serial: nothing is asked of any phone
        let (adb, fb) = stand_in(&d, "echo called >> \"$0.calls2\"", "exit 0");
        assert!(to_fastboot(&adb, &fb, "", w)
            .unwrap_err()
            .contains("no serial"));
        assert!(!d.join("adb.calls2").exists());
        std::fs::remove_dir_all(d).ok();
    }
}
