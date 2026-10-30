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

//! The arity probe: what a documented method does when it is sent an argument
//! list it could accept, measured against the oracle.

#![allow(dead_code)]
#![expect(
    clippy::disallowed_methods,
    reason = "this harness times or bounds real runs"
)]

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::oracle;

fn interpreter_binary() -> PathBuf {
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_rexx"));
    assert!(
        binary.is_file(),
        "arity interpreter is missing at {}; run cargo build -p rexx-exec --bin rexx in this target directory",
        binary.display()
    );
    binary
}

pub const EXEMPT: &str = "EXEMPT:";

/// A row whose send the ORACLE refuses by design, where that refusal is the
/// measurement rather than a bad argument list.
pub const REFUSED: &str = "REFUSED:";

/// A row whose answered value the ORACLE does not reproduce between two of
/// its own runs, so no value comparison could say anything about it.
pub const UNSTABLE: &str = "UNSTABLE:";

/// A one-line program the probe directory always holds, so that a row whose
/// send needs a file on disk has one to name.
pub const FIXTURE: &str = "source.rex";
pub const NONE: &str = "--";

/// What the probe directory is called in the evidence column.
const PROBE_DIR: &str = "<probe directory>";

/// The four files one table is derived from and written to, plus what the
/// driver calls itself.
pub struct Layout {
    /// Receivers: `class<TAB>setup` without [`Layout::arm_column`], and
    /// `class<TAB>arm<TAB>setup<TAB>wrapper<TAB>directives` with it.
    pub receivers: &'static str,
    /// Argument lists: `class<TAB>method<TAB>arguments`, with `arm` inserted
    /// third when [`Layout::arm_column`] is set.
    pub arguments: &'static str,
    /// The derived scope table [`rows_missing_arguments`] reads the `arity`
    /// column out of.
    pub scopes: &'static str,
    /// The table this layout writes.
    pub table: &'static str,
    pub refresh_env: &'static str,
    pub header: &'static str,
    /// Whether every file carries an `arm` column and both arms are measured.
    pub arm_column: bool,
    /// Whether the probe assigns the send's result and prints `vv~string`, so
    /// that a row completing on both sides with different answers reads
    /// `send-differs` rather than `agree`.
    pub compare_values: bool,
    /// Whether the probe directory holds [`FIXTURE`].
    pub fixture: bool,
    /// Distinguishes this driver's probe directories from another's.
    pub probe_prefix: &'static str,
}

/// The arm a row with no `arm` column is measured on.
const IMPLIED_ARM: &str = "instance";

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Row {
    pub class: String,
    pub method: String,
    pub arm: String,
    pub verdict: String,
    pub evidence: String,
}

/// How a class's receiver is built.
#[derive(Clone, Debug)]
pub struct Receiver {
    /// Statements separated by `|`, which must leave `r` holding the receiver.
    pub setup: String,
    /// `call` puts the setup and the send inside an internal routine called
    /// from the main program; [`NONE`] leaves both at the top level.
    pub wrapper: String,
    /// Directive text, `|`-separated lines, appended after the program.
    pub directives: String,
}

impl Receiver {
    fn wrapped(&self) -> bool {
        self.wrapper == "call"
    }
}

pub fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../corpus")
}

pub fn read_table(path: &Path, fields: usize) -> Vec<Vec<String>> {
    let text =
        fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let mut rows = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let row: Vec<String> = line.split('\t').map(str::to_string).collect();
        assert_eq!(
            row.len(),
            fields,
            "{}: row {line:?} has the wrong number of fields",
            path.display()
        );
        rows.push(row);
    }
    assert!(
        !rows.is_empty(),
        "{} named no rows -- that is a defect in the row file, not an empty pass",
        path.display()
    );
    rows
}

/// Every receiver in a receivers file, keyed by (class, arm).
pub fn read_receivers(file: &str, arm_column: bool) -> HashMap<(String, String), Receiver> {
    let fields = if arm_column { 5 } else { 2 };
    read_table(&corpus_root().join(file), fields)
        .into_iter()
        .map(|row| {
            if arm_column {
                (
                    (row[0].clone(), row[1].clone()),
                    Receiver {
                        setup: row[2].clone(),
                        wrapper: row[3].clone(),
                        directives: row[4].clone(),
                    },
                )
            } else {
                (
                    (row[0].clone(), IMPLIED_ARM.to_string()),
                    Receiver {
                        setup: row[1].clone(),
                        wrapper: NONE.to_string(),
                        directives: NONE.to_string(),
                    },
                )
            }
        })
        .collect()
}

impl Layout {
    fn path(&self, name: &str) -> PathBuf {
        corpus_root().join(name)
    }

    /// Every class's receiver, keyed by (class, arm).
    pub fn receivers(&self) -> HashMap<(String, String), Receiver> {
        read_receivers(self.receivers, self.arm_column)
    }

    /// Every (class, method, arm, argument list) the probe sends.
    pub fn arguments(&self) -> Vec<(String, String, String, String)> {
        let fields = if self.arm_column { 4 } else { 3 };
        read_table(&self.path(self.arguments), fields)
            .into_iter()
            .map(|row| {
                if self.arm_column {
                    (
                        row[0].clone(),
                        row[1].clone(),
                        row[2].clone(),
                        row[3].clone(),
                    )
                } else {
                    (
                        row[0].clone(),
                        row[1].clone(),
                        IMPLIED_ARM.to_string(),
                        row[2].clone(),
                    )
                }
            })
            .collect()
    }

    fn line(&self, row: &Row) -> String {
        if self.arm_column {
            format!(
                "{}\t{}\t{}\t{}\t{}\n",
                row.class, row.method, row.arm, row.verdict, row.evidence
            )
        } else {
            format!(
                "{}\t{}\t{}\t{}\n",
                row.class, row.method, row.verdict, row.evidence
            )
        }
    }

    pub fn committed(&self) -> Vec<Row> {
        let path = self.path(self.table);
        if !path.exists() {
            return Vec::new();
        }
        let fields = if self.arm_column { 5 } else { 4 };
        let mut rows: Vec<Row> = read_table(&path, fields)
            .into_iter()
            .map(|row| {
                if self.arm_column {
                    Row {
                        class: row[0].clone(),
                        method: row[1].clone(),
                        arm: row[2].clone(),
                        verdict: row[3].clone(),
                        evidence: row[4].clone(),
                    }
                } else {
                    Row {
                        class: row[0].clone(),
                        method: row[1].clone(),
                        arm: IMPLIED_ARM.to_string(),
                        verdict: row[2].clone(),
                        evidence: row[3].clone(),
                    }
                }
            })
            .collect();
        rows.sort();
        rows
    }

    pub fn refreshing(&self) -> bool {
        matches!(std::env::var(self.refresh_env), Ok(value) if !value.is_empty() && value != "0")
    }

    fn write(&self, rows: &[Row]) {
        let mut text = String::from(self.header);
        for row in rows {
            text.push_str(&self.line(row));
        }
        fs::write(self.path(self.table), text).expect("the table is writable");
    }
}

/// Rows whose answer is a path naming the interpreter that ran, so neither
/// side's value can match the other's: their `VALUE` line is the answer's
/// class and whether the file it names exists.
const PATH_VALUED: &[(&str, &str)] = &[("RexxInfo", "executable"), ("RexxInfo", "libraryPath")];

/// What a row's `VALUE` line prints of the answer `vv`.
pub fn rendering(class: &str, method: &str) -> &'static str {
    if PATH_VALUED.contains(&(class, method)) {
        "vv~class vv~exists"
    } else {
        "vv~string"
    }
}

/// The probe for one row: build the receiver, announce it, send, announce
/// that.
pub fn program(receiver: &Receiver, method: &str, arguments: &str, values: bool) -> String {
    program_rendering(receiver, method, arguments, values, "vv~string")
}

/// [`program`] with the `VALUE` line printing `rendering`.
pub fn program_rendering(
    receiver: &Receiver,
    method: &str,
    arguments: &str,
    values: bool,
    rendering: &str,
) -> String {
    let call = if arguments == NONE {
        format!("r~'{method}'")
    } else {
        format!("r~'{method}'({arguments})")
    };
    let send = if values {
        format!("vv = {call}\n")
    } else {
        format!("{call}\n")
    };
    let announce = if values {
        format!("say 'SENT'\nsay 'VALUE' {rendering}\nexit\n")
    } else {
        "say 'SENT'\nexit\n".to_string()
    };
    let trap = if values {
        "oops:\nvc = rc || '.' || condition('E')\nif vc = '91.999' then do\nsay 'SENT'\n\
         say 'NORESULT'\nexit 0\nend\nsay 'SYNTAX' vc\nexit 0\n"
    } else {
        "oops:\nsay 'SYNTAX' condition('O')~code\nexit 0\n"
    };
    let mut setup = String::new();
    for statement in receiver.setup.split('|') {
        setup.push_str(statement.trim());
        setup.push('\n');
    }
    let mut text = String::new();
    if receiver.wrapped() {
        text.push_str("signal on syntax name oops\n");
        text.push_str("call probe 'a1', 'a2'\n");
        text.push_str(&announce);
        text.push_str(trap);
        text.push_str("probe:\n");
        text.push_str(&setup);
        text.push_str("say 'SETUP-OK'\n");
        text.push_str(&send);
        text.push_str("return\n");
    } else {
        text.push_str(&setup);
        text.push_str("say 'SETUP-OK'\n");
        text.push_str("signal on syntax name oops\n");
        text.push_str(&send);
        text.push_str(&announce);
        text.push_str(trap);
    }
    if receiver.directives != NONE {
        for line in receiver.directives.split('|') {
            text.push_str(line.trim());
            text.push('\n');
        }
    }
    text
}

/// Exit status, stdout and stderr, read as three separate descriptors.
type Three = (i32, String, String);

fn run(command: &mut Command) -> Three {
    let out = command.output().expect("the probe runs");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// The probe directory's own name, out of both streams and under one name.
fn scrub(three: Three, dir: &Path) -> Three {
    let dir = dir.display().to_string();
    (
        three.0,
        three.1.replace(&dir, PROBE_DIR),
        three.2.replace(&dir, PROBE_DIR),
    )
}

/// The oracle wrapped as `rust/CLAUDE.md` requires: its own address-space cap,
/// its own library path, and the probe directory as the working directory.
fn oracle_command(oracle: &Path, program: &Path, dir: &Path) -> Command {
    let mut command = Command::new("bash");
    command
        .arg("-c")
        .arg(format!(
            "ulimit -v 1048576; exec {} {}",
            oracle.join("bin/rexx").display(),
            program.display()
        ))
        .current_dir(dir)
        .env("LD_LIBRARY_PATH", oracle.join("lib"));
    command
}

fn probe_dir(prefix: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the clock is after the epoch")
            .as_nanos()
    ));
    fs::create_dir_all(&base).expect("a private probe directory");
    base
}

/// The three sides for every row, plus the oracle's stdout so the harness
/// rule can be checked separately from the verdict.
pub fn measured(layout: &Layout) -> Vec<(Row, String)> {
    let receivers = layout.receivers();
    let dir = probe_dir(layout.probe_prefix);
    let path = dir.join("probe.rex");
    if layout.fixture {
        fs::write(dir.join(FIXTURE), "return 1\n").expect("the fixture is writable");
    }
    let oracle = oracle::oracle_root();
    let binary = interpreter_binary();
    let mut rows = Vec::new();
    for (class, method, arm, list) in layout.arguments() {
        if let Some(reason) = list.strip_prefix(EXEMPT) {
            rows.push((
                Row {
                    class,
                    method,
                    arm,
                    verdict: "exempt".into(),
                    evidence: reason.to_string(),
                },
                String::new(),
            ));
            continue;
        }
        let receiver = receivers
            .get(&(class.clone(), arm.clone()))
            .unwrap_or_else(|| panic!("{} has no setup for {class} ({arm})", layout.receivers));
        let unstable = list.starts_with(UNSTABLE);
        let list = if list.starts_with(REFUSED) || unstable {
            NONE.to_string()
        } else {
            list
        };
        let values = layout.compare_values && !unstable;
        let text = program_rendering(receiver, &method, &list, values, rendering(&class, &method));
        fs::write(&path, text).expect("the probe is writable");
        let cpp = scrub(run(&mut oracle_command(&oracle, &path, &dir)), &dir);
        // **The same `LD_LIBRARY_PATH` the oracle child is given**, so a row
        // whose method loads one of the oracle's own extensions is measuring
        // the two interpreters and not the difference between a child that
        // was told where the libraries are and one that was not. Set on the
        // child, never on this process.
        let ours = scrub(
            run(Command::new(&binary)
                .arg(&path)
                .current_dir(&dir)
                .env("LD_LIBRARY_PATH", oracle.join("lib"))),
            &dir,
        );
        let (verdict, evidence) = classify(&cpp, &ours, unstable);
        rows.push((
            Row {
                class,
                method,
                arm,
                verdict,
                evidence,
            },
            cpp.1,
        ));
    }
    rows.sort();
    rows
}

fn classify(cpp: &Three, ir: &Three, unstable: bool) -> (String, String) {
    if ir == cpp {
        if unstable {
            return (
                "unstable".into(),
                format!("rc{}, the oracle does not reproduce its own answer", cpp.0),
            );
        }
        if cpp.1.contains("NORESULT") {
            return (
                "no-value".into(),
                format!("rc{}, the send returned no result to compare", cpp.0),
            );
        }
        return ("agree".into(), format!("rc{}", cpp.0));
    }
    if !ir.1.contains("SETUP-OK") {
        return (
            "setup-differs".into(),
            format!("crate rc{}: {}", ir.0, first_line(&ir.2)),
        );
    }
    (
        "send-differs".into(),
        format!(
            "oracle rc{} {}; crate rc{} {}",
            cpp.0,
            last_line(&cpp.1),
            ir.0,
            if ir.2.trim().is_empty() {
                last_line(&ir.1)
            } else {
                first_line(&ir.2)
            }
        ),
    )
}

/// A control character in an answer would put a second tab in the row and
/// break the table's own shape, so evidence carries none.
fn printable(line: &str, width: usize) -> String {
    line.chars()
        .take(width)
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}

fn first_line(text: &str) -> String {
    printable(text.trim().lines().next().unwrap_or(""), 90)
}

fn last_line(text: &str) -> String {
    printable(text.trim().lines().next_back().unwrap_or(""), 60)
}

// ---- what the drivers assert ----

/// Refresh the table, or answer the rows that moved against the committed one.
pub fn table_disagreements(layout: &Layout) -> Vec<(Row, Row)> {
    let measured: Vec<Row> = measured(layout).into_iter().map(|(row, _)| row).collect();
    if layout.refreshing() {
        layout.write(&measured);
        return Vec::new();
    }
    let committed = layout.committed();
    assert!(
        !committed.is_empty(),
        "{} is missing. Create it with `{}=1 cargo test --release -p rexx-exec --test <driver>`",
        layout.table,
        layout.refresh_env
    );
    assert_eq!(
        measured.len(),
        committed.len(),
        "{} has {} rows against the interpreters' {}",
        layout.table,
        committed.len(),
        measured.len()
    );
    measured
        .into_iter()
        .zip(committed)
        .filter(|(a, b)| a != b)
        .collect()
}

/// **The harness rule.** The rows whose send the oracle does not complete,
/// which are a failure of this instrument for that row and never a data point.
pub fn unsent_rows(layout: &Layout) -> Vec<String> {
    let refused = refused_keys(layout);
    measured(layout)
        .iter()
        .filter(|(row, stdout)| {
            row.verdict != "exempt"
                && !refused.contains(&(row.class.clone(), row.method.clone(), row.arm.clone()))
                && !stdout.contains("SENT")
        })
        .map(|(row, stdout)| {
            format!(
                "{}~{} ({}): the oracle stopped at {:?}",
                row.class,
                row.method,
                row.arm,
                stdout.trim().lines().next_back().unwrap_or("nothing")
            )
        })
        .collect()
}

/// The rows marked [`REFUSED`] whose send the oracle does in fact complete.
pub fn refusals_the_oracle_completes(layout: &Layout) -> Vec<String> {
    let refused = refused_keys(layout);
    measured(layout)
        .iter()
        .filter(|(row, stdout)| {
            refused.contains(&(row.class.clone(), row.method.clone(), row.arm.clone()))
                && stdout.contains("SENT")
        })
        .map(|(row, _)| {
            format!(
                "{}~{} ({}) is marked {REFUSED} and the oracle completes the send, so the \
                 refusal is not the measurement -- give it a real argument list",
                row.class, row.method, row.arm
            )
        })
        .collect()
}

fn refused_keys(layout: &Layout) -> std::collections::HashSet<(String, String, String)> {
    layout
        .arguments()
        .into_iter()
        .filter(|(_, _, _, list)| list.starts_with(REFUSED))
        .map(|(class, method, arm, _)| (class, method, arm))
        .collect()
}

mod unstable;
pub use unstable::stable_rows_marked_unstable;

/// The documented rows with no list at all, and the native rows whose upstream
/// arity is not zero and that are sent nothing.
pub fn rows_missing_arguments(layout: &Layout) -> Vec<String> {
    let arms: &[&str] = if layout.arm_column {
        &["instance", "class"]
    } else {
        &[IMPLIED_ARM]
    };
    let scopes: HashMap<(String, String, String), (String, String)> =
        read_table(&corpus_root().join(layout.scopes), 7)
            .into_iter()
            .filter(|row| arms.contains(&row[2].as_str()))
            .map(|row| {
                (
                    (row[0].clone(), row[1].clone(), row[2].clone()),
                    (row[4].clone(), row[6].clone()),
                )
            })
            .collect();
    let lists: HashMap<(String, String, String), String> = layout
        .arguments()
        .into_iter()
        .map(|(class, method, arm, list)| ((class, method, arm), list))
        .collect();
    let mut wrong = Vec::new();
    for (key, (kind, arity)) in &scopes {
        let Some(list) = lists.get(key) else {
            wrong.push(format!(
                "{}~{} ({}) has no argument list at all",
                key.0, key.1, key.2
            ));
            continue;
        };
        if kind == "native" && arity != "0" && (list == NONE || list.starts_with(UNSTABLE)) {
            wrong.push(format!(
                "{}~{} ({}) is native at arity {arity} and is sent nothing",
                key.0, key.1, key.2
            ));
        }
    }
    wrong.sort();
    wrong
}

/// An exemption carries its reason, so declining a row is a sentence and not
/// a blank.
pub fn exemptions_without_a_reason(layout: &Layout) -> Vec<String> {
    layout
        .arguments()
        .into_iter()
        .filter_map(|(class, method, arm, list)| {
            let reason = list
                .strip_prefix(EXEMPT)
                .or_else(|| list.strip_prefix(REFUSED))
                .or_else(|| list.strip_prefix(UNSTABLE))?;
            (reason.len() <= 20).then(|| {
                format!(
                    "{class}~{method} ({arm}) is exempt with no reason worth reading: {reason:?}"
                )
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod binary_selection_tests {
    #[test]
    fn arity_runs_the_cargo_owned_binary() {
        let selected = super::interpreter_binary()
            .canonicalize()
            .expect("selected interpreter");
        let cargo_binary = std::path::Path::new(env!("CARGO_BIN_EXE_rexx"))
            .canonicalize()
            .expect("Cargo interpreter");
        assert_eq!(selected, cargo_binary);
        let target_profile = std::env::current_exe()
            .expect("test executable")
            .parent()
            .expect("deps")
            .parent()
            .expect("profile")
            .to_path_buf();
        assert_eq!(selected.parent(), Some(target_profile.as_path()));
    }
}
