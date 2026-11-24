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

/// The sha256 of a file, as `sha256sum` prints it.
pub fn sha256_of(file: &Path) -> Result<String, String> {
    let mut c = Command::new("sha256sum");
    c.arg(file);
    let said = run(c)?;
    if !said.ok {
        return Err(format!("sha256sum {}: {}", file.display(), said.text));
    }
    Ok(said
        .text
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string())
}

/// The sha256 of the first `bytes` bytes of a file (all of it when it is shorter is an error: the bytes asked for must exist).
pub fn sha256_of_head(file: &Path, bytes: u64) -> Result<String, String> {
    use std::io::{Read, Write};
    let mut head = Vec::new();
    std::fs::File::open(file)
        .and_then(|f| f.take(bytes).read_to_end(&mut head))
        .map_err(|e| format!("{}: {e}", file.display()))?;
    if head.len() as u64 != bytes {
        return Err(format!("{} has fewer than {bytes} bytes", file.display()));
    }
    let mut c = Command::new("sha256sum")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot run sha256sum: {e}"))?;
    c.stdin
        .take()
        .ok_or("sha256sum has no input")?
        .write_all(&head)
        .map_err(|e| format!("sha256sum: {e}"))?;
    let out = c.wait_with_output().map_err(|e| e.to_string())?;
    String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .next()
        .map(str::to_string)
        .ok_or_else(|| "sha256sum gave no sum".to_string())
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
    let got = sha256_of(image)?;
    if got == want {
        Ok(())
    } else {
        Err(format!("{name} is {got}, its SHA256SUMS line says {want}"))
    }
}

/// The path `lpoco-bringup image ensure NAME` names: the last line of a real run is the store path alone, the last line of a `--dry-run` ends in
/// `(PATH)` (captured text of lpoco-bringup 0.47.1 in research/walleye-bootimage/2026-11-23/ensure-z21.txt, and of 0.48.4 in research walleye-image-recipe-z21-rebuilt-equal).
pub fn ensured_path(said: &str) -> Option<String> {
    let last = said.lines().rev().map(str::trim).find(|l| !l.is_empty())?;
    if last.starts_with('/') && !last.contains(' ') {
        return Some(last.to_string());
    }
    let open = last.rfind('(')?;
    let path = last[open + 1..].strip_suffix(')')?;
    path.starts_with('/').then(|| path.to_string())
}

/// A run takes its image from `image ensure` by sha256, never from a file in walleye-build that may be stale (item walleye-run-uses-ensured-image-not-build-file):
/// the build file already has the declared sha256 - nothing is asked; otherwise `bringup image ensure NAME --device DEVICE_TOML` names a store path, that
/// file's sha256 must equal the declared one, and it replaces the build file (written beside it, then renamed). Anything else is an error that names what was seen,
/// and the run stops before any phone step. The sentence says what was done.
pub fn the_image_is_ensured(
    bringup: &str,
    name: &str,
    device_toml: &Path,
    image: &Path,
    want: &str,
) -> Result<String, String> {
    if image_has_sha256(image, want).is_ok() {
        return Ok(format!(
            "{name}: the file is the declared one; nothing ensured"
        ));
    }
    let stale = image_has_sha256(image, want).unwrap_err();
    let mut c = Command::new(bringup);
    c.args(["image", "ensure", name, "--device"])
        .arg(device_toml);
    let said = run(c).map_err(|e| format!("{stale}; image ensure could not be run: {e}"))?;
    if !said.ok {
        return Err(format!(
            "{stale}; image ensure {name} failed: {}",
            last_line(&said.text)
        ));
    }
    let path = ensured_path(&said.text).ok_or_else(|| {
        format!(
            "{stale}; image ensure {name} named no store path: {}",
            last_line(&said.text)
        )
    })?;
    let store = Path::new(&path);
    image_has_sha256(store, want)
        .map_err(|e| format!("{stale}; the ensured file is not the declared one: {e}"))?;
    let tmp = image.with_extension("ensuring");
    std::fs::copy(store, &tmp).map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, image).map_err(|e| format!("{}: {e}", image.display()))?;
    Ok(format!(
        "{name}: the build file was stale ({stale}); replaced by the ensured {path}"
    ))
}

/// The image's sha256 is the one the manifest records.
pub fn image_has_sha256(image: &Path, want: &str) -> Result<(), String> {
    let name = image.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let want = want.trim().to_lowercase();
    if want.is_empty() {
        return Err(format!("no sha256 recorded for {name} in device.toml"));
    }
    let got = sha256_of(image)?;
    if got == want {
        Ok(())
    } else {
        Err(format!("{name} is {got}, device.toml records {want}"))
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

/// A fresh directory that is removed when it is dropped, whichever way the work in it ends.
struct Scratch(std::path::PathBuf);

impl Scratch {
    fn new(what: &str) -> Result<Self, String> {
        // `create_dir` refuses a name that exists, so the directory is this run's own
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let d = std::env::temp_dir().join(format!("{what}-{}-{nanos}", std::process::id()));
        std::fs::create_dir(&d).map_err(|e| format!("{}: {e}", d.display()))?;
        Ok(Scratch(d))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The image's AVB footer verifies (`avbtool verify_image`). avbtool reads the hash descriptor's partition name and looks for
/// `<name>.img` beside the image unless the image is itself so named, so an image under any other name (a NOT-TO-BOOT one)
/// is verified through a link named `boot.img` in a directory of its own, removed afterwards.
pub fn avb_footer_verifies(avbtool: &str, image: &Path) -> Result<(), String> {
    let real = image
        .canonicalize()
        .map_err(|e| format!("{}: {e}", image.display()))?;
    let dir = Scratch::new("walleye-avb")?;
    let link = dir.0.join("boot.img");
    std::os::unix::fs::symlink(&real, &link).map_err(|e| format!("{}: {e}", link.display()))?;
    let mut c = Command::new(avbtool);
    c.args(["verify_image", "--image"]).arg(&link);
    let said = run(c)?;
    if said.ok {
        Ok(())
    } else {
        Err(format!(
            "the image's AVB footer does not verify: {}",
            last_line(&said.text)
        ))
    }
}

pub fn last_line(text: &str) -> String {
    text.lines().last().unwrap_or("").to_string()
}

/// The lines of `fastboot getvar all` that say which slot the phone is on and each slot's flags (successful, unbootable,
/// retry count), sorted, so two reads of an unchanged phone are equal text. Nothing else (no serial) is kept.
pub fn slot_lines(getvar_all: &str) -> String {
    let mut lines: Vec<String> = getvar_all
        .lines()
        .map(|l| {
            l.trim()
                .trim_start_matches("(bootloader)")
                .trim()
                .to_string()
        })
        .filter(|l| {
            l.starts_with("current-slot")
                || l.starts_with("slot-successful")
                || l.starts_with("slot-unbootable")
                || l.starts_with("slot-retry-count")
        })
        .collect();
    lines.sort();
    lines.dedup();
    lines.join("\n")
}

/// The slot flags of the phone in fastboot (`fastboot -s SERIAL getvar all`); an answer with no slot line is an error, not "unchanged".
pub fn slot_flags(fastboot: &str, serial: &str) -> Result<String, String> {
    let mut c = Command::new(fastboot);
    c.args(["-s", serial, "getvar", "all"]);
    let said = run(c)?;
    let lines = slot_lines(&said.text);
    if lines.contains("current-slot") && lines.contains("slot-successful") {
        Ok(lines)
    } else {
        Err(format!(
            "fastboot told no slot flags (current-slot, slot-successful): {}",
            last_line(&said.text)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const ENSURE_Z21: &str =
        include_str!("../../../research/walleye-bootimage/2026-11-23/ensure-z21.txt");

    #[test]
    fn the_ensured_path_is_read_off_the_text_lpoco_bringup_printed() {
        // captured: research/walleye-bootimage/2026-11-23/ensure-z21.txt (a real run) and the 2026-11-24 --dry-run line of lpoco-bringup 0.48.4
        let run: String = ENSURE_Z21.lines().take(5).collect::<Vec<_>>().join("\n");
        assert_eq!(
            ensured_path(&run).as_deref(),
            Some("/nix/store/6vmx4fp4hbhsfam89mfa8ixks7rlj2n0-boot-z21.img")
        );
        let dry = "ufs-root-booted-slot-production-rw: in the store (/nix/store/h6d3g53lpqbiqw1nfkwh6s2q8mv4ky0b-NOT-TO-BOOT-ufs-on-ufs-root-booted-slot-production-rw.img)\n";
        assert_eq!(ensured_path(dry).as_deref(), Some("/nix/store/h6d3g53lpqbiqw1nfkwh6s2q8mv4ky0b-NOT-TO-BOOT-ufs-on-ufs-root-booted-slot-production-rw.img"));
        assert_eq!(ensured_path("z21: no recipe\n"), None);
    }

    fn ensure_rig(
        name: &str,
        build: &str,
        store: &str,
        stub: &str,
    ) -> (PathBuf, PathBuf, String, PathBuf) {
        let d = std::env::temp_dir().join(format!("wt-ensure-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let (image, store_file) = (d.join("NOT-TO-BOOT-x.img"), d.join("store.img"));
        std::fs::write(&image, build).unwrap();
        std::fs::write(&store_file, store).unwrap();
        let want = sha256_of(&store_file).unwrap();
        let program = d.join("lpoco-bringup");
        crate::fixture::executable(
            &program,
            &stub.replace("STORE", &store_file.to_string_lossy()),
        );
        (image, program, want, d)
    }

    #[test]
    fn a_stale_build_file_is_replaced_by_the_ensured_image_and_a_current_one_asks_nothing() {
        let (image, program, want, d) = ensure_rig(
            "replace",
            "old build",
            "the declared build",
            "#!/bin/sh\necho 'x: in the store'\necho STORE\n",
        );
        let said = the_image_is_ensured(
            &program.to_string_lossy(),
            "x",
            &d.join("device.toml"),
            &image,
            &want,
        )
        .unwrap();
        assert!(
            said.contains("stale") && said.contains("replaced"),
            "{said}"
        );
        assert_eq!(
            std::fs::read_to_string(&image).unwrap(),
            "the declared build"
        );
        // current now: a program that fails is never run
        let failing = d.join("failing");
        crate::fixture::executable(&failing, "#!/bin/sh\nexit 1\n");
        assert!(the_image_is_ensured(
            &failing.to_string_lossy(),
            "x",
            &d.join("device.toml"),
            &image,
            &want
        )
        .unwrap()
        .contains("nothing ensured"));
    }

    #[test]
    fn a_stale_build_file_the_ensure_cannot_mend_stops_the_run_naming_both_shas() {
        let (image, failing, want, d) = ensure_rig(
            "fails",
            "old build",
            "the declared build",
            "#!/bin/sh\necho 'x: no recipe' >&2\nexit 1\n",
        );
        let e = the_image_is_ensured(
            &failing.to_string_lossy(),
            "x",
            &d.join("device.toml"),
            &image,
            &want,
        )
        .unwrap_err();
        assert!(
            e.contains("device.toml records") && e.contains("no recipe"),
            "{e}"
        );
        assert_eq!(std::fs::read_to_string(&image).unwrap(), "old build");
        let (image, wrong, want, d) = ensure_rig(
            "wrong",
            "old build",
            "the declared build",
            "#!/bin/sh\necho '/etc/hostname'\n",
        );
        let e = the_image_is_ensured(
            &wrong.to_string_lossy(),
            "x",
            &d.join("device.toml"),
            &image,
            &want,
        )
        .unwrap_err();
        assert!(e.contains("not the declared one"), "{e}");
        assert_eq!(std::fs::read_to_string(&image).unwrap(), "old build");
    }

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

    /// A stand-in avbtool with the real one's habit: it looks for `<partition name>.img` in the image's directory
    /// unless the image is itself so named (the partition here is `boot`), and writes the path it was given to `$0.saw`.
    fn avbtool_like_the_real_one(d: &std::path::Path) -> String {
        std::fs::create_dir_all(d).unwrap();
        let p = d.join("avbtool");
        crate::fixture::executable(
            &p,
            "#!/bin/sh\n\
             [ \"$1\" = verify_image ] || exit 2\n\
             img=$3\n\
             echo \"$img\" > \"$0.saw\"\n\
             [ \"${img##*/}\" = boot.img ] || { echo \"FileNotFoundError: ${img%/*}/boot.img\"; exit 1; }\n\
             grep -q good \"$img\" || { echo \"Hash of data does not match digest\"; exit 1; }\n",
        );
        p.to_string_lossy().into_owned()
    }

    #[test]
    fn an_image_not_named_for_its_partition_verifies_through_a_boot_img_link_removed_after() {
        let d = std::env::temp_dir().join(format!("wt-avb-{}", std::process::id()));
        let avbtool = avbtool_like_the_real_one(&d);
        let saw = || {
            std::path::PathBuf::from(
                std::fs::read_to_string(d.join("avbtool.saw"))
                    .unwrap()
                    .trim(),
            )
        };
        let good = d.join("NOT-TO-BOOT-x.img");
        std::fs::write(&good, "good").unwrap();
        avb_footer_verifies(&avbtool, &good).unwrap();
        assert_eq!(saw().file_name().unwrap(), "boot.img");
        assert!(
            !saw().parent().unwrap().exists(),
            "the temporary directory stays"
        );
        // a failing verify says what avbtool said, and still leaves nothing behind
        let bad = d.join("NOT-TO-BOOT-y.img");
        std::fs::write(&bad, "bad").unwrap();
        assert_eq!(
            avb_footer_verifies(&avbtool, &bad).unwrap_err(),
            "the image's AVB footer does not verify: Hash of data does not match digest"
        );
        assert!(
            !saw().parent().unwrap().exists(),
            "the temporary directory stays after a failure"
        );
        // an image that is not there is refused before avbtool is asked
        assert!(avb_footer_verifies(&avbtool, &d.join("missing.img")).is_err());
        std::fs::remove_dir_all(d).ok();
    }

    fn stand_in(d: &std::path::Path, adb: &str, fastboot: &str) -> (String, String) {
        std::fs::create_dir_all(d).unwrap();
        let mut out = vec![];
        for (n, body) in [("adb", adb), ("fastboot", fastboot)] {
            let p = d.join(n);
            crate::fixture::executable(&p, &format!("#!/bin/sh\n{body}\n"));
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

    #[test]
    fn an_image_is_the_one_whose_sha256_the_manifest_records() {
        let d = std::env::temp_dir().join(format!("wt-sha-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let img = d.join("boot-x.img");
        std::fs::write(&img, "abc").unwrap();
        // sha256 of "abc"
        let abc = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        image_has_sha256(&img, abc).unwrap();
        image_has_sha256(&img, &abc.to_uppercase()).unwrap();
        let e = image_has_sha256(&img, "57ab").unwrap_err();
        assert!(e.contains(abc) && e.contains("57ab"), "{e}");
        assert!(image_has_sha256(&img, "")
            .unwrap_err()
            .contains("no sha256"));
        std::fs::remove_dir_all(d).ok();
    }
}
