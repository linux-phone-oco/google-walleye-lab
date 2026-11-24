// SPDX-License-Identifier: MIT OR Apache-2.0
//! The gate against small trees built here; the ids it resolves are made-up names in made-up data files, so
//! these are checks of the gate itself, not of any phone fact.

use std::fs;
use std::path::Path;
use std::process::Command;

fn tree(table: &str, scripts: &[&str], research: &[&str]) -> tempdir::T {
    let t = tempdir::T::new();
    let r = t.path();
    fs::create_dir_all(r.join("tests/host")).unwrap();
    fs::create_dir_all(r.join("data")).unwrap();
    for s in scripts {
        fs::write(r.join("tests/host").join(s), "#!/bin/sh\n").unwrap();
    }
    fs::write(r.join("tests/host/SOURCES.toml"), table).unwrap();
    let rows: String = research
        .iter()
        .map(|i| format!("{{\"id\":\"{i}\"}}\n"))
        .collect();
    fs::write(r.join("data/research.json"), rows).unwrap();
    fs::write(r.join("data/sources.json"), "{\"id\":\"src-a\"}\n").unwrap();
    t
}

fn gate(root: &Path) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_walleye-sources"))
        .args(["--root", root.to_str().unwrap()])
        .output()
        .unwrap();
    (
        o.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&o.stdout).into(),
    )
}

mod tempdir {
    use std::path::{Path, PathBuf};
    pub struct T(PathBuf);
    impl T {
        pub fn new() -> T {
            let p = std::env::temp_dir().join(format!(
                "walleye-sources-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir_all(&p).unwrap();
            T(p)
        }
        pub fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for T {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

#[test]
fn a_sourced_script_passes() {
    let t = tree(
        "[scripts.\"a.sh\"]\nsource = \"research:r1\"\n",
        &["a.sh"],
        &["r1"],
    );
    let (rc, out) = gate(t.path());
    assert_eq!(rc, 0, "{out}");
    assert!(out.contains("1 sourced"), "{out}");
}

#[test]
fn a_script_with_no_row_is_refused() {
    let t = tree("", &["a.sh"], &["r1"]);
    let (rc, out) = gate(t.path());
    assert_eq!(rc, 1, "{out}");
    assert!(out.contains("a.sh: no source"), "{out}");
}

#[test]
fn a_research_id_that_does_not_exist_is_refused() {
    let t = tree(
        "[scripts.\"a.sh\"]\nsource = \"research:gone\"\n",
        &["a.sh"],
        &["r1"],
    );
    let (rc, out) = gate(t.path());
    assert_eq!(rc, 1, "{out}");
    assert!(out.contains("research row gone does not exist"), "{out}");
}

#[test]
fn a_file_source_must_exist_and_a_source_row_is_read() {
    let t = tree(
        "[scripts.\"a.sh\"]\nsource = \"file:research/x.log\"\n[scripts.\"b.sh\"]\nsource = \"source:src-a\"\n",
        &["a.sh", "b.sh"],
        &[],
    );
    let (rc, out) = gate(t.path());
    assert_eq!(rc, 1, "{out}");
    assert!(out.contains("file research/x.log does not exist"), "{out}");
    assert!(!out.contains("src-a"), "{out}");
    fs::create_dir_all(t.path().join("research")).unwrap();
    fs::write(t.path().join("research/x.log"), "text\n").unwrap();
    assert_eq!(gate(t.path()).0, 0);
}

#[test]
fn a_row_for_a_gone_script_and_a_row_with_two_answers_are_refused() {
    let t = tree(
        "[scripts.\"gone.sh\"]\nsource = \"research:r1\"\n[scripts.\"a.sh\"]\nsource = \"research:r1\"\nunsourced = \"why\"\n",
        &["a.sh"],
        &["r1"],
    );
    let (rc, out) = gate(t.path());
    assert_eq!(rc, 1, "{out}");
    assert!(
        out.contains("gone.sh: the script is not in tests/host"),
        "{out}"
    );
    assert!(out.contains("a.sh: both source and unsourced"), "{out}");
}

#[test]
fn an_unsourced_script_passes_and_is_listed_as_not_yet_sourced() {
    let t = tree(
        "[scripts.\"a.sh\"]\nunsourced = \"the first run's text was never kept\"\n",
        &["a.sh"],
        &[],
    );
    let (rc, out) = gate(t.path());
    assert_eq!(rc, 0, "{out}");
    assert!(
        out.contains("not yet sourced: a.sh - the first run's text was never kept"),
        "{out}"
    );
    assert!(out.contains("1 not yet sourced"), "{out}");
}

#[test]
fn a_source_kind_nobody_knows_is_refused() {
    let t = tree(
        "[scripts.\"a.sh\"]\nsource = \"hunch:it-was-fine\"\n",
        &["a.sh"],
        &[],
    );
    let (rc, out) = gate(t.path());
    assert_eq!(rc, 1, "{out}");
    assert!(out.contains("a.sh: the source kind hunch"), "{out}");
}

#[test]
fn an_unsourced_row_with_no_reason_and_a_row_with_nothing_are_refused() {
    let t = tree(
        "[scripts.\"a.sh\"]\nunsourced = \" \"\n[scripts.\"b.sh\"]\n",
        &["a.sh", "b.sh"],
        &[],
    );
    let (rc, out) = gate(t.path());
    assert_eq!(rc, 1, "{out}");
    assert!(out.contains("a.sh: unsourced gives no reason"), "{out}");
    assert!(out.contains("b.sh: a row with neither"), "{out}");
}

#[test]
fn a_missing_source_row_and_a_data_line_with_no_id_are_refused() {
    let t = tree(
        "[scripts.\"a.sh\"]\nsource = \"source:nope\"\n",
        &["a.sh"],
        &[],
    );
    let (rc, out) = gate(t.path());
    assert_eq!(rc, 1, "{out}");
    assert!(out.contains("source row nope does not exist"), "{out}");
    fs::write(t.path().join("data/sources.json"), "{\"title\":\"x\"}\n").unwrap();
    let (rc, out) = gate(t.path());
    assert_eq!(rc, 1, "{out}");
    assert!(out.contains("line 1: no id"), "{out}");
}
