/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 2026 Rexx Language Association. All rights reserved.          */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:           */
/* https://www.oorexx.org/license.html                                        */
/*                                                                            */
/*----------------------------------------------------------------------------*/

//! Walks a checked-out ooTest suite, extracts standalone assertions from every
//! `.testGroup` file it finds, and reports the extractable fraction.

use rexx_extract::{TestMethod, extract};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let (mut suite, mut out, mut report) = (None, None, None);
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--suite" => suite = args.next().map(PathBuf::from),
            "--out" => out = args.next().map(PathBuf::from),
            "--report" => report = args.next().map(PathBuf::from),
            other => {
                eprintln!("unknown flag: {other}");
                return ExitCode::from(2);
            }
        }
    }
    let (Some(suite), Some(out), Some(report)) = (suite, out, report) else {
        eprintln!("usage: rexx-extract --suite <dir> --out <dir> --report <file>");
        return ExitCode::from(2);
    };

    let mut groups: Vec<PathBuf> = walk(&suite);
    groups.sort();
    // A mistyped or empty suite directory would otherwise report an empty
    // (and therefore vacuously "complete") table -- the same trap rexx-diff
    // guards against for an empty corpus.
    if groups.is_empty() {
        eprintln!("no .testGroup files under {}", suite.display());
        return ExitCode::from(2);
    }

    if let Err(e) = std::fs::create_dir_all(&out) {
        eprintln!("cannot create {}: {e}", out.display());
        return ExitCode::from(2);
    }

    let mut rows = Vec::new();
    let (mut total_tests, mut total_extractable) = (0usize, 0usize);
    for group_path in &groups {
        let source = match std::fs::read_to_string(group_path) {
            Ok(source) => source,
            Err(e) => {
                eprintln!("cannot read {}: {e}", group_path.display());
                return ExitCode::from(2);
            }
        };
        let group_name = group_path.file_stem().and_then(|s| s.to_str()).unwrap_or("group");
        let methods = extract(&source);
        let extractable: Vec<&TestMethod> = methods.iter().filter(|m| !m.uses_fixture).collect();

        for method in &extractable {
            let program = render(group_name, method);
            let file_name = format!("{group_name}_{}.rex", method.name);
            if let Err(e) = std::fs::write(out.join(&file_name), program) {
                eprintln!("cannot write {}: {e}", out.join(&file_name).display());
                return ExitCode::from(2);
            }
        }

        let (total, extracted) = (methods.len(), extractable.len());
        total_tests += total;
        total_extractable += extracted;
        let pct = percentage(extracted, total);
        rows.push(format!("| {} | {total} | {extracted} | {pct:.1}% |", group_path.display()));
    }

    let total_pct = percentage(total_extractable, total_tests);
    let mut report_body = String::new();
    report_body.push_str("| File | Total | Extractable | Percentage |\n");
    report_body.push_str("|---|---|---|---|\n");
    for row in &rows {
        report_body.push_str(row);
        report_body.push('\n');
    }
    report_body.push_str(&format!(
        "| **Total** | **{total_tests}** | **{total_extractable}** | **{total_pct:.1}%** |\n"
    ));

    if let Err(e) = std::fs::write(&report, report_body) {
        eprintln!("cannot write {}: {e}", report.display());
        return ExitCode::from(2);
    }

    println!(
        "{} groups, {total_tests} test methods, {total_extractable} extractable ({total_pct:.1}%)",
        groups.len()
    );
    ExitCode::SUCCESS
}

fn percentage(part: usize, whole: usize) -> f64 {
    if whole == 0 { 0.0 } else { 100.0 * part as f64 / whole as f64 }
}

/// One `::method` per name in `rexx_extract::ASSERTIONS`'s intent -- the shim
/// must define exactly the assertions the extractor is willing to recognise,
/// or a method that used a recognised-but-undefined message would be marked
/// extractable and then fail at runtime with "message not understood".
const SHIM_METHODS: &str = r#"::method assertEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertNotEquals
  use arg expected, actual
  if expected == actual then do
    say "FAIL not-expected["expected"] actual["actual"]"
    exit 1
  end
::method assertTrue
  use arg condition
  if \condition then do
    say "FAIL expected true actual["condition"]"
    exit 1
  end
::method assertFalse
  use arg condition
  if condition then do
    say "FAIL expected false actual["condition"]"
    exit 1
  end
::method assertNull
  use arg actual
  if actual \== .nil then do
    say "FAIL expected nil actual["actual"]"
    exit 1
  end
::method assertNotNull
  use arg actual
  if actual == .nil then do
    say "FAIL expected non-nil actual nil"
    exit 1
  end
::method assertSame
  use arg expected, actual
  if \(expected == actual) then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertNotSame
  use arg expected, actual
  if expected == actual then do
    say "FAIL not-expected["expected"] actual["actual"]"
    exit 1
  end
::method expectSyntax
  use arg code
  nop
::method assertListEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertArrayEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
"#;

/// Wraps a test method's body in a minimal standalone program: a `main`
/// routine running the body, plus a `shim` class defining exactly the
/// assertion messages the extractor recognises.
fn render(group_name: &str, method: &TestMethod) -> String {
    format!(
        "/* extracted from {group_name}::{} */\n::routine main public\n{}\n::class shim public\n{}",
        method.name, method.body, SHIM_METHODS
    )
}

fn walk(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        // Do not swallow this: an unreadable directory read as "empty" is how
        // a self-test reports success for work it never did.
        Err(e) => panic!("cannot read {}: {e}", dir.display()),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk(&path));
        } else if path.extension().is_some_and(|e| e == "testGroup") {
            out.push(path);
        }
    }
    out
}
