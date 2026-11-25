// SPDX-License-Identifier: MIT OR Apache-2.0
//! The check that every `[images.NAME.recipe]` command line of the manifest stays within the header budget.
//!
//! Where the budget comes from: run 8's captured /proc/cmdline was cut at 1363 bytes, the bootloader
//! appending about 1087 characters after our header, so the header must stay under about 270 (research
//! walleye-stage2-cmdline-budget-and-slot-sources); the lab keeps it to 160, a margin of 110. The retired
//! build-ufs-root scripts checked the length; nothing did after them (item walleye-recipe-cmdline-budget-check).
//!
//! An image over the budget may stand in the exceptions table only while it cannot be booted (its path
//! starts with NOT-TO-BOOT), with a reason; a row for an image that is gone or no longer over is refused,
//! so the table cannot outlive what it excuses.

use std::fs;
use toml::{Table, Value};

/// The most characters a recipe's command line may have.
pub const DEFAULT_MAX: usize = 160;

const NAME: &str = "walleye-cmdline-budget";
// The budget is about the bootloader's cut line, so it is counted in bytes.
const UNBOOTABLE: &str = "NOT-TO-BOOT";

/// Read the manifest and the exceptions, and say what each recipe's line measures: `(text, ok)`.
pub fn check(device: &str, exceptions: &str, max: usize) -> (String, bool) {
    let mut out = String::new();
    let mut ok = true;
    let mut say = |good: bool, text: String| {
        out.push_str(&format!(
            "{} {NAME} {text}\n",
            if good { "pass" } else { "fail" }
        ));
        ok &= good;
    };
    let manifest = match read_table(device) {
        Ok(t) => t,
        Err(e) => return (format!("fail {NAME} {e}\n"), false),
    };
    let allowed = match read_table(exceptions) {
        Ok(t) => t,
        Err(e) => return (format!("fail {NAME} {e}\n"), false),
    };
    let images = manifest.get("images").and_then(Value::as_table);
    let over = allowed.get("over").and_then(Value::as_table);
    let mut measured = 0;
    for (name, image) in images.into_iter().flatten() {
        let Some(cmdline) = image.get("recipe").and_then(|r| r.get("cmdline")) else {
            continue;
        };
        let Some(line) = cmdline.as_str() else {
            say(
                false,
                format!("{name}: the recipe's cmdline is not a string"),
            );
            continue;
        };
        measured += 1;
        let n = line.len();
        let excused = over.is_some_and(|o| o.contains_key(name));
        if n <= max {
            say(true, format!("{name}: {n} bytes, within {max}"));
        } else if excused {
            say(
                true,
                format!("{name}: {n} bytes, over {max} and excused by {exceptions}"),
            );
        } else {
            say(
                false,
                format!(
                    "{name}: {n} bytes, over {max} (the bootloader cuts what follows our header)"
                ),
            );
        }
    }
    for (name, row) in over.into_iter().flatten() {
        let why = row.get("why").and_then(Value::as_str).unwrap_or("");
        let image = images.and_then(|i| i.get(name));
        let line = image
            .and_then(|i| i.get("recipe"))
            .and_then(|r| r.get("cmdline"))
            .and_then(Value::as_str);
        let path = image
            .and_then(|i| i.get("path"))
            .and_then(Value::as_str)
            .unwrap_or("");
        if why.trim().is_empty() {
            say(
                false,
                format!("{name}: the exception has no why in {exceptions}"),
            );
        }
        match line {
            None => say(false, format!("{name}: excused in {exceptions} but {device} has no recipe command line for it")),
            Some(l) if l.len() <= max => say(false, format!("{name}: excused in {exceptions} but no longer over {max}: remove the row")),
            Some(_) if !path.starts_with(UNBOOTABLE) => say(false, format!("{name}: excused in {exceptions} but its path {path} can be booted: only a {UNBOOTABLE} image may be over")),
            Some(_) => {}
        }
    }
    if measured == 0 {
        say(
            false,
            format!("{device} has no [images.NAME.recipe] command line to measure"),
        );
    }
    (out, ok)
}

fn read_table(path: &str) -> Result<Table, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    text.parse::<Table>()
        .map_err(|e| format!("cannot parse {path}: {e}"))
}
