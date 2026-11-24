// SPDX-License-Identifier: MIT OR Apache-2.0
//! The gate that every check names its source. One step each: read the files, read the table, read the rows a
//! source may cite, judge every file, say what is refused and what is not yet sourced.

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

/// Where to look, all relative to `root` except `root` itself.
pub struct Gate {
    pub root: String,
    pub dir: String,
    pub table: String,
    pub ext: String,
}

/// One row of the table: a source, or the reason there is none yet. Exactly one of the two.
#[derive(Deserialize, Default)]
struct Entry {
    source: Option<String>,
    unsourced: Option<String>,
}

#[derive(Deserialize, Default)]
struct Table {
    #[serde(default)]
    scripts: BTreeMap<String, Entry>,
}

/// The ids a source may cite.
struct Known {
    research: BTreeSet<String>,
    sources: BTreeSet<String>,
}

/// What the gate found.
struct Report {
    problems: Vec<String>,
    unsourced: Vec<(String, String)>,
    sourced: usize,
}

/// The text the gate prints and whether it passed.
pub fn run_gate(gate: &Gate, root: &Path) -> (String, bool) {
    let report = judge_all(gate, root);
    let ok = report.problems.is_empty();
    (render(&report, gate), ok)
}

fn judge_all(gate: &Gate, root: &Path) -> Report {
    let mut r = Report {
        problems: vec![],
        unsourced: vec![],
        sourced: 0,
    };
    let files = match list_files(&root.join(&gate.dir), &gate.ext) {
        Ok(f) => f,
        Err(e) => return fail(r, e),
    };
    let table = match read_table(&root.join(&gate.table)) {
        Ok(t) => t,
        Err(e) => return fail(r, e),
    };
    let known = match read_known(root) {
        Ok(k) => k,
        Err(e) => return fail(r, e),
    };
    for f in &files {
        match table.scripts.get(f) {
            None => r
                .problems
                .push(format!("{f}: no source (add a row to {})", gate.table)),
            Some(e) => judge_entry(&mut r, f, e, &known, root),
        }
    }
    for name in table.scripts.keys().filter(|n| !files.contains(*n)) {
        r.problems
            .push(format!("{name}: the script is not in {}", gate.dir));
    }
    r
}

fn fail(mut r: Report, why: String) -> Report {
    r.problems.push(why);
    r
}

fn judge_entry(r: &mut Report, file: &str, e: &Entry, known: &Known, root: &Path) {
    match (&e.source, &e.unsourced) {
        (Some(_), Some(_)) => r
            .problems
            .push(format!("{file}: both source and unsourced")),
        (None, None) => r
            .problems
            .push(format!("{file}: a row with neither source nor unsourced")),
        (None, Some(why)) if why.trim().is_empty() => r
            .problems
            .push(format!("{file}: unsourced gives no reason")),
        (None, Some(why)) => r.unsourced.push((file.to_string(), why.clone())),
        (Some(s), None) => match resolve(s, known, root) {
            Ok(()) => r.sourced += 1,
            Err(why) => r.problems.push(format!("{file}: {why}")),
        },
    }
}

/// A source is `research:ID` (data/research.json), `source:ID` (data/sources.json) or `file:PATH` (under the root).
fn resolve(source: &str, known: &Known, root: &Path) -> Result<(), String> {
    let Some((kind, what)) = source.split_once(':') else {
        return Err(format!("the source {source:?} is not KIND:WHAT"));
    };
    match kind {
        "research" if known.research.contains(what) => Ok(()),
        "research" => Err(format!("research row {what} does not exist")),
        "source" if known.sources.contains(what) => Ok(()),
        "source" => Err(format!("source row {what} does not exist")),
        "file" if root.join(what).is_file() => Ok(()),
        "file" => Err(format!("file {what} does not exist")),
        other => Err(format!(
            "the source kind {other} is none of research, source, file"
        )),
    }
}

fn list_files(dir: &Path, ext: &str) -> Result<BTreeSet<String>, String> {
    let rd = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let suffix = format!(".{ext}");
    let mut names = BTreeSet::new();
    for entry in rd {
        let entry = entry.map_err(|e| format!("{}: {e}", dir.display()))?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|n| format!("{}: a name that is not UTF-8: {n:?}", dir.display()))?;
        if entry.path().is_file() && name.ends_with(&suffix) {
            names.insert(name);
        }
    }
    Ok(names)
}

fn read_table(path: &Path) -> Result<Table, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

fn read_known(root: &Path) -> Result<Known, String> {
    Ok(Known {
        research: ids_of(&root.join("data/research.json"))?,
        sources: ids_of(&root.join("data/sources.json"))?,
    })
}

/// The `id` of each line of a json-lines data file.
fn ids_of(path: &Path) -> Result<BTreeSet<String>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut ids = BTreeSet::new();
    for (n, line) in text
        .lines()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty())
    {
        let v: serde_json::Value = serde_json::from_str(line)
            .map_err(|e| format!("{} line {}: {e}", path.display(), n + 1))?;
        let id = v
            .get("id")
            .and_then(|i| i.as_str())
            .ok_or_else(|| format!("{} line {}: no id", path.display(), n + 1))?;
        ids.insert(id.to_string());
    }
    Ok(ids)
}

fn render(r: &Report, gate: &Gate) -> String {
    let mut out = String::new();
    for p in &r.problems {
        out.push_str(&format!("refused {p}\n"));
    }
    for (f, why) in &r.unsourced {
        out.push_str(&format!("not yet sourced: {f} - {why}\n"));
    }
    out.push_str(&format!(
        "walleye-sources {}: {} sourced, {} not yet sourced, {} refused\n",
        gate.dir,
        r.sourced,
        r.unsourced.len(),
        r.problems.len()
    ));
    out
}
