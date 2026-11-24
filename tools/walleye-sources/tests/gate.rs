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

/// A tree with one procedure, `procedures/p.rs`, holding `body`; the research rows are `research`.
fn procedures_tree(body: &str, research: &[&str]) -> tempdir::T {
    let t = tree("", &[], research);
    fs::create_dir_all(t.path().join("procedures")).unwrap();
    fs::write(t.path().join("procedures/p.rs"), body).unwrap();
    t
}

fn procedures_gate(root: &Path) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_walleye-sources"))
        .args([
            "--root",
            root.to_str().unwrap(),
            "--procedures",
            "procedures",
        ])
        .output()
        .unwrap();
    (
        o.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&o.stdout).into(),
    )
}

#[test]
fn a_procedure_check_citing_a_research_row_passes() {
    let t = procedures_tree(
        "fn main() {\n    check_from(\"the slot is read\", \"research:r1\", x.slot());\n    step(\"go\", x.go());\n}\n",
        &["r1"],
    );
    let (rc, out) = procedures_gate(t.path());
    assert_eq!(rc, 0, "{out}");
    assert!(out.contains("1 sourced"), "{out}");
}

#[test]
fn a_procedure_check_citing_a_research_row_that_does_not_exist_is_refused() {
    let t = procedures_tree(
        "check_from(\"the slot is read\", \"research:gone\", x.slot());\n",
        &["r1"],
    );
    let (rc, out) = procedures_gate(t.path());
    assert_eq!(rc, 1, "{out}");
    assert!(
        out.contains("p.rs:1: research row gone does not exist"),
        "{out}"
    );
}

#[test]
fn a_procedure_check_with_no_source_is_refused() {
    let t = procedures_tree(
        "step(\"a\", x.a());\ncheck(\"the slot is read\", x.slot());\n",
        &["r1"],
    );
    let (rc, out) = procedures_gate(t.path());
    assert_eq!(rc, 1, "{out}");
    assert!(
        out.contains("p.rs:2: check(..) names no source (use check_from)"),
        "{out}"
    );
}

#[test]
fn a_check_from_with_a_source_that_is_not_kind_and_what_is_refused() {
    let t = procedures_tree("check_from(\"x\", \"it-was-fine\", x.y());\n", &[]);
    let (rc, out) = procedures_gate(t.path());
    assert_eq!(rc, 1, "{out}");
    assert!(
        out.contains("p.rs:1: the source \"it-was-fine\" is not KIND:WHAT"),
        "{out}"
    );
}

#[test]
fn a_check_from_over_several_lines_is_read_whole() {
    let body = "check_from(\n    &format!(\"B-d{d}: loads\"),\n    \"research:r1\",\n    b.x(d),\n);\ncheck_from(\n    \"y\",\n    \"research:gone\",\n    b.y(),\n);\n";
    let t = procedures_tree(body, &["r1"]);
    let (rc, out) = procedures_gate(t.path());
    assert_eq!(rc, 1, "{out}");
    assert!(
        out.contains("p.rs:6: research row gone does not exist"),
        "{out}"
    );
    assert!(out.contains("1 sourced"), "{out}");
}

fn refused(body: &str, research: &[&str], want: &str) {
    let t = procedures_tree(body, research);
    let (rc, out) = procedures_gate(t.path());
    assert_eq!(rc, 1, "{out}");
    assert!(out.contains(want), "{out}");
}

fn passes(body: &str, research: &[&str]) {
    let t = procedures_tree(body, research);
    let (rc, out) = procedures_gate(t.path());
    assert_eq!(rc, 0, "{out}");
}

#[test]
fn the_source_is_the_second_argument_not_the_second_literal() {
    passes("check_from(\"w\", \"research:r1\", x.a(\"b\"));\n", &["r1"]);
    refused(
        "check_from(\"w\", SRC, x.a(\"research:r1\"));\n",
        &["r1"],
        "p.rs:1: the source must be a string literal",
    );
    refused(
        "check_from(what, \"research:gone\", f());\n",
        &["r1"],
        "p.rs:1: research row gone does not exist",
    );
}

#[test]
fn comments_strings_and_char_literals_hide_nothing_and_show_nothing() {
    passes(
        "step(\"a\", x.a()); // then check(\"x\", y)\n/* check(\"z\", w) */\ncheck_from(\"uses check( inside\", \"research:r1\", f('\"'));\nlet s = r#\"check(\"q\", r)\"#;\n",
        &["r1"],
    );
    refused(
        "let c = '\"';\ncheck(\"x\", f());\n",
        &[],
        "p.rs:2: check(..) names no source",
    );
}

#[test]
fn a_spaced_bare_check_is_refused_and_a_method_or_definition_is_not_a_call() {
    refused(
        "check (\"x\", f());\n",
        &[],
        "p.rs:1: check(..) names no source",
    );
    passes(
        "fn check_from(a: &str, b: &str) {}\nfn check(a: u8) {}\nx.check(1);\nx.check_from(1);\n",
        &[],
    );
}

#[test]
fn an_unknown_kind_and_source_and_file_rows_are_judged_in_procedures_too() {
    refused(
        "check_from(\"w\", \"hunch:x\", f());\n",
        &[],
        "the source kind hunch",
    );
    passes("check_from(\"w\", \"source:src-a\", f());\n", &[]);
    refused(
        "check_from(\"w\", \"file:no/such\", f());\n",
        &[],
        "file no/such does not exist",
    );
}

#[test]
fn a_missing_procedures_folder_and_options_of_the_other_mode_are_refused() {
    let t = tree("", &[], &[]);
    let (rc, out) = procedures_gate(t.path());
    assert_eq!(rc, 1, "{out}");
    let o = Command::new(env!("CARGO_BIN_EXE_walleye-sources"))
        .args([
            "--root",
            t.path().to_str().unwrap(),
            "--procedures",
            "procedures",
            "--ext",
            "cap",
        ])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("--procedures stands alone"));
}
