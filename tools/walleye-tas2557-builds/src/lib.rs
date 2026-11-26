// SPDX-License-Identifier: MIT OR Apache-2.0
//! The proof walleye-tas2557-driver-builds. Each check reads the text a step printed and judges it; the texts the
//! tests judge are the ones captured on 2026-11-26 (research walleye-tas2557-v3-builds), so a judge that
//! drifts from what the tools really print fails there.

pub mod steps;

/// What one check found.
#[derive(Debug, PartialEq, Eq)]
pub enum Verdict {
    Pass(String),
    Fail(String),
    /// Not known yet: the check could not run here.
    Skip(String),
}

impl Verdict {
    pub fn word(&self) -> &'static str {
        match self {
            Verdict::Pass(_) => "pass",
            Verdict::Fail(_) => "fail",
            Verdict::Skip(_) => "skip",
        }
    }
    pub fn detail(&self) -> &str {
        match self {
            Verdict::Pass(d) | Verdict::Fail(d) | Verdict::Skip(d) => d,
        }
    }
}

fn fail<T>(why: impl Into<String>) -> Result<T, String> {
    Err(why.into())
}

/// `git log --format='%H %s' BASE..TOPIC`: two commits, each a FROMLIST one. Returns the tip's sha.
pub fn series(commits: &str) -> Result<String, String> {
    let rows: Vec<&str> = commits.lines().filter(|l| !l.trim().is_empty()).collect();
    if rows.len() != 2 {
        return fail(format!(
            "the topic is {} commits on its base, not two",
            rows.len()
        ));
    }
    for r in &rows {
        let (sha, subject) = r.split_once(' ').unwrap_or((r, ""));
        if sha.len() != 40 || !sha.bytes().all(|b| b.is_ascii_hexdigit()) {
            return fail(format!("not a commit line: {r}"));
        }
        if !subject.starts_with("FROMLIST:") {
            return fail(format!("{sha} is not a FROMLIST commit: {subject}"));
        }
    }
    Ok(rows[0].split(' ').next().unwrap_or_default().to_string())
}

/// The last `KEY=N` line of a text, N the exit status the step appended.
fn exit_of(text: &str, key: &str) -> Result<i32, String> {
    let prefix = format!("{key}=");
    let line = text.lines().rev().find(|l| l.starts_with(&prefix));
    match line.and_then(|l| l[prefix.len()..].trim().parse().ok()) {
        Some(n) => Ok(n),
        None => fail(format!("no {key}=N line: the step did not finish")),
    }
}

/// The first line with one of the words as a whole word (`strerror.o` is not an error).
fn first_with(text: &str, words: &[&str]) -> Option<String> {
    text.lines()
        .find(|l| {
            l.split(|c: char| !c.is_ascii_alphanumeric())
                .any(|w| words.iter().any(|x| w.eq_ignore_ascii_case(x)))
        })
        .map(str::to_string)
}

/// The tags make prints before a path.
const MAKE_TAGS: [&str; 10] = [
    "SCHEMA", "HOSTCC", "HOSTLD", "LEX", "YACC", "CHKDT", "LINT", "STYLE", "DTEX", "DTC",
];

/// A line make itself prints. The schema tools' complaints are any other line: they carry neither "warning"
/// nor "error", and the kernel's Makefile ends those checks with `|| true`, so the exit says nothing.
fn is_make_line(l: &str) -> bool {
    let l = l.trim();
    l.is_empty()
        || l.starts_with("make[")
        || l.starts_with("make_exit=")
        || MAKE_TAGS.iter().any(|t| {
            l.strip_prefix(t)
                .is_some_and(|r| r.starts_with(char::is_whitespace))
        })
}

/// `make DT_SCHEMA_FILES=sound/ti,tas2557.yaml dt_binding_check` in the container: exit 0, every line
/// make's own, and the schema's example compiled.
pub fn binding(text: &str) -> Result<(), String> {
    let n = exit_of(text, "make_exit")?;
    if n != 0 {
        return fail(format!("dt_binding_check exits {n}"));
    }
    if let Some(l) = text.lines().find(|l| !is_make_line(l)) {
        return fail(format!("dt_binding_check says: {}", l.trim()));
    }
    let compiled = text.lines().any(|l| {
        l.trim_start().starts_with("DTC") && l.trim_end().ends_with("ti,tas2557.example.dtb")
    });
    if !compiled {
        return fail("the schema's example (ti,tas2557.example.dtb) was not compiled");
    }
    Ok(())
}

/// `make W=1 sound/soc/codecs/tas2557.o`: exit 0, the object compiled, no warning.
pub fn driver_build(text: &str) -> Result<(), String> {
    let n = exit_of(text, "build_exit")?;
    if n != 0 {
        return fail(format!("the build exits {n}"));
    }
    if let Some(l) = first_with(text, &["warning", "warnings", "error", "errors"]) {
        return fail(format!("the build says: {}", l.trim()));
    }
    let built = text.lines().any(|l| {
        l.trim_start().starts_with("CC [M]") && l.trim_end().ends_with("sound/soc/codecs/tas2557.o")
    });
    if !built {
        return fail("sound/soc/codecs/tas2557.o was not compiled");
    }
    Ok(())
}

fn value<'a>(text: &'a str, key: &str) -> Result<&'a str, String> {
    let prefix = format!("{key}: ");
    match text.lines().find_map(|l| l.strip_prefix(&prefix)) {
        Some(v) => Ok(v.trim()),
        None => fail(format!("the read-back has no `{key}` line")),
    }
}

/// The read-back of the merged dtb (fdtget): the bus is okay, the node is ti,tas2557 at 0x4c 0x4d, its reset is
/// pm8998 gpio 6 active low, and the gpio phandle names a qcom,pm8998-gpio node.
pub fn overlay(text: &str) -> Result<(), String> {
    let sha = value(text, "ov.dtbo sha256")?;
    if sha.len() != 64 || !sha.bytes().all(|b| b.is_ascii_hexdigit()) {
        return fail(format!("the overlay's sha256 is not one: {sha}"));
    }
    let want = [
        ("i2c status", "okay"),
        ("compatible", "ti,tas2557"),
        ("reg", "4c 4d"),
    ];
    for (k, v) in want {
        let got = value(text, k)?;
        if got != v {
            return fail(format!("{k} is `{got}`, not `{v}`"));
        }
    }
    let reset: Vec<&str> = value(text, "reset-gpios (phandle gpio flags)")?
        .split_whitespace()
        .collect();
    let [phandle, gpio, flags] = reset[..] else {
        return fail("reset-gpios is not three cells");
    };
    if (gpio, flags) != ("6", "1") {
        return fail(format!(
            "reset-gpios is gpio {gpio} flags {flags}, not gpio 6 active low (1)"
        ));
    }
    let prefix = format!("phandle {phandle} is: ");
    let Some(who) = text.lines().find_map(|l| l.strip_prefix(&prefix)) else {
        return fail(format!("no line says what phandle {phandle} is"));
    };
    if !who.contains("compatible qcom,pm8998-gpio") {
        return fail(format!(
            "phandle {phandle} is not a qcom,pm8998-gpio node: {who}"
        ));
    }
    Ok(())
}

/// The four checks together, and the one line the run prints.
pub fn conclude(tip: &str, parts: &[(&str, Verdict)]) -> Verdict {
    if let Some((name, v)) = parts.iter().find(|(_, v)| matches!(v, Verdict::Fail(_))) {
        return Verdict::Fail(format!("{name}: {}", v.detail()));
    }
    if let Some((name, v)) = parts.iter().find(|(_, v)| matches!(v, Verdict::Skip(_))) {
        return Verdict::Skip(format!("{name}: {} (the other checks pass)", v.detail()));
    }
    Verdict::Pass(format!(
        "the topic's tip {tip} is two FROMLIST commits on its base, its binding checks, tas2557.o builds with W=1 without a warning, and the overlay merges onto the walleye dtb (ti,tas2557 at 0x4c 0x4d, reset pm8998 gpio 6 active low)"
    ))
}

pub fn line(v: &Verdict) -> String {
    format!(
        "{} walleye-tas2557-driver-builds host {}",
        v.word(),
        v.detail()
    )
}
