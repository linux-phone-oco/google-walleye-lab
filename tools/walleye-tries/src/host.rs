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

pub fn last_line(text: &str) -> String {
    text.lines().last().unwrap_or("").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
