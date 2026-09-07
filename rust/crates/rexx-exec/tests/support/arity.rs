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
//! list it could accept, measured against the oracle and both engines.
//!
//! One machine, two tables. `tests/collection_arity.rs` drives it over the
//! collections and `tests/introspection_arity.rs` over Phase 5i's
//! introspection classes; each driver names its own four files, refresh
//! variable and header through a [`Layout`].
//!
//! # The rule that makes a table mean anything
//!
//! **A row's argument list is real only if the ORACLE completes the send.**
//! The probe prints `SENT` after the send; an oracle run that does not reach
//! it is a harness failure for that row and not a data point, and
//! [`unsent_rows`] is what a driver turns into a test failure.
//!
//! Without the rule the instrument is defeated two ways that both read green:
//! fill in argument lists for a couple of rows and leave the rest empty, or
//! send two arguments to everything and let both sides agree on 93.902.
//!
//! An earlier version of the rule asked only for oracle exit 0, and the probe
//! traps `SYNTAX` and exits 0, so a list that *raised* satisfied it.
//! Requiring `SENT` is what closed it.
//!
//! # The verdicts
//!
//! * `agree` -- the oracle and both engines give identical three descriptors.
//! * `send-differs` -- the receiver was built on both sides and the send
//!   differs. This is the per-row signal.
//! * `setup-differs` -- one side could not build the receiver at all, so the
//!   row says nothing about its own method. **That is a phase's headline
//!   measurement rather than a harness fault**: the receiver files
//!   deliberately use the richest receiver the *oracle* can build rather than
//!   the richest both sides can.
//! * `engine-differs` -- the two engines disagree with each other, which is a
//!   defect of its own and never expected.
//! * `exempt` -- a row with a committed reason instead of a list.
//! * `no-value` -- under [`Layout::compare_values`], a send both sides
//!   completed that returned no result, so there was nothing to compare
//!   beyond the three descriptors. Deliberately not `agree`.
//! * `unstable` -- a row marked [`UNSTABLE`], whose value the oracle does not
//!   reproduce between two of its own runs.
//!
//! # What `agree` does not say
//!
//! With [`Layout::compare_values`] off the send is a bare statement whose
//! result the probe never prints, so `agree` says the three descriptors match
//! -- that neither side raised -- and not that the two sides answered the
//! same value. That is a property of this probe and holds for every table it
//! drives that way, `corpus/collection-arity.tsv` included, whose own header
//! predates the flag and does not say it. A task sizing itself from an
//! `agree` row of such a table still owes that row a witness that reads what
//! it answered.

#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::oracle;

pub const EXEMPT: &str = "EXEMPT:";

/// A row whose send the ORACLE refuses by design, where that refusal is the
/// measurement rather than a bad argument list.
///
/// `Pointer~new` and `Buffer~new` are the case it exists for: the reference
/// says instances come only from native code, the oracle answers `93.967`,
/// and no argument list makes it answer anything else -- so the rule that a
/// list is real only if the oracle completes the send has no better list to
/// ask for. The row is still measured: the three sides are compared on the
/// refusal, and a crate that refuses differently reads `send-differs`.
///
/// **The rule is inverted rather than waived**, by
/// [`refusals_the_oracle_completes`]: a row marked this way that the oracle
/// *does* complete is a failure, so the marker cannot hide a bad list. The
/// send is made with no arguments.
pub const REFUSED: &str = "REFUSED:";

/// A row whose answered value the ORACLE does not reproduce between two of
/// its own runs, so no value comparison could say anything about it.
///
/// `Object~hashCode` and `~identityHash` are the case it exists for. The
/// value is not printed for such a row and the verdict is `unstable`, which
/// is `corpus/method-bodies.txt`'s own word for an answer no verdict can rest
/// on. The send is made with no arguments.
///
/// **The rule is inverted rather than waived**, by
/// [`stable_rows_marked_unstable`]: a row marked this way whose two oracle
/// runs answer the same thing is a failure, so the marker cannot hide a
/// divergence.
pub const UNSTABLE: &str = "UNSTABLE:";

/// A one-line program the probe directory always holds, so that a row whose
/// send needs a file on disk has one to name.
///
/// `Method~newFile`, `Routine~newFile`, `Package~findProgram` and
/// `Package~loadPackage` take a file name and the oracle raises `3.1` for one
/// that is not there; the probe writes a single file and cannot be that file
/// as well, because loading the running program re-executes it.
pub const FIXTURE: &str = "source.rex";
pub const NONE: &str = "--";

/// What the probe directory is called in the evidence column.
///
/// Its real name carries a pid and a nanosecond timestamp, and
/// `Package~findProgram` answers the absolute path of [`FIXTURE`], so a row
/// quoting it verbatim would rewrite itself on every refresh.
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
    ///
    /// `corpus/collection-arity.tsv` predates the column and does not carry
    /// it; widening that file would move bytes no task asked to move.
    pub arm_column: bool,
    /// Whether the probe assigns the send's result and prints `vv~string`, so
    /// that a row completing on both sides with different answers reads
    /// `send-differs` rather than `agree`.
    ///
    /// Off for `corpus/collection-arity.tsv`, whose committed bytes are this
    /// task's control and whose verdicts are earlier phases' measurements.
    pub compare_values: bool,
    /// Whether the probe directory holds [`FIXTURE`].
    ///
    /// The oracle searches the working directory for an external routine and
    /// the probe runs there, so writing it for a driver that has no row
    /// needing a file would change that driver's environment.
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
///
/// This is a free function rather than a [`Layout`] method so that a caller
/// wanting only the receivers -- `introspection_scopes.rs` does -- does not
/// have to fabricate a layout whose write target would be an input file.
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

/// The probe for one row: build the receiver, announce it, send, announce
/// that.
///
/// The message name is quoted because `[]` and `[]=` are not symbols. Under
/// `values` the send is an assignment and the answer is printed, so a method
/// returning no result raises `91.999` at the assignment; the trap reads that
/// one code as a completed send with nothing to compare rather than as a
/// refusal, which is what keeps such a row out of the harness rule. That
/// handler builds the code from `rc` and `condition('E')` rather than from
/// `condition('O')~code`, so reaching it asks nothing of a side under test
/// that the send itself did not: a probe whose handler an engine cannot run
/// measures the probe.
///
/// A `call` receiver puts both the setup and the send inside an internal
/// routine, because a `.context` built in one activation is dead in its
/// caller (`98.981`) and a stack with one frame on it has no caller to
/// describe. The `SYNTAX` trap then sits in the main program, where the
/// condition arrives after the routine unwinds.
pub fn program(receiver: &Receiver, method: &str, arguments: &str, values: bool) -> String {
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
        "say 'SENT'\nsay 'VALUE' vv~string\nexit\n"
    } else {
        "say 'SENT'\nexit\n"
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
        text.push_str(announce);
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
        text.push_str(announce);
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
///
/// Applied to every side, so it cannot make two sides that answered
/// differently compare equal.
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
    let binary = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/release/rexx-run");
    assert!(
        binary.is_file(),
        "this test compares the release binary; build it first ({})",
        binary.display()
    );
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
        fs::write(&path, program(receiver, &method, &list, values)).expect("the probe is writable");
        let cpp = scrub(run(&mut oracle_command(&oracle, &path, &dir)), &dir);
        let mut engines = Vec::new();
        for engine in ["ir", "tree-walker"] {
            engines.push(scrub(
                run(Command::new(&binary)
                    .arg(&path)
                    .current_dir(&dir)
                    .env("REXX_ENGINE", engine)),
                &dir,
            ));
        }
        let (verdict, evidence) = classify(&cpp, &engines[0], &engines[1], unstable);
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

fn classify(cpp: &Three, ir: &Three, tree: &Three, unstable: bool) -> (String, String) {
    if ir != tree {
        return (
            "engine-differs".into(),
            format!("ir rc{} against tree-walker rc{}", ir.0, tree.0),
        );
    }
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
///
/// Under the refresh variable the table is rewritten and the answer is empty.
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
///
/// This is the inversion that keeps the marker from being an escape hatch: a
/// row is only allowed to skip the `SENT` requirement while the oracle really
/// does refuse it.
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

/// The rows marked [`UNSTABLE`] whose two oracle runs answer the same thing.
///
/// The classification is a measurement -- the oracle is run twice and its own
/// two answers compared -- rather than a sentence in a header, so a row that
/// starts reproducing loses the marker instead of hiding a divergence behind
/// it.
pub fn stable_rows_marked_unstable(layout: &Layout) -> Vec<String> {
    let receivers = layout.receivers();
    let dir = probe_dir(layout.probe_prefix);
    let path = dir.join("probe.rex");
    if layout.fixture {
        fs::write(dir.join(FIXTURE), "return 1\n").expect("the fixture is writable");
    }
    let oracle = oracle::oracle_root();
    let mut stable = Vec::new();
    for (class, method, arm, list) in layout.arguments() {
        if !list.starts_with(UNSTABLE) {
            continue;
        }
        let receiver = receivers
            .get(&(class.clone(), arm.clone()))
            .unwrap_or_else(|| panic!("{} has no setup for {class} ({arm})", layout.receivers));
        fs::write(&path, program(receiver, &method, NONE, true)).expect("the probe is writable");
        let first = run(&mut oracle_command(&oracle, &path, &dir));
        let second = run(&mut oracle_command(&oracle, &path, &dir));
        if first == second {
            stable.push(format!(
                "{class}~{method} ({arm}) is marked {UNSTABLE} and the oracle answers {:?} on \
                 two runs of its own, so the value is comparable -- drop the marker",
                last_line(&first.1)
            ));
        }
    }
    stable
}

/// The documented rows with no list at all, and the native rows whose upstream
/// arity is not zero and that are sent nothing.
///
/// This is what defeats "fill in the two control rows and leave the rest
/// empty": an empty list on a method that takes arguments is caught here
/// rather than read as agreement.
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

/// The committed rows on which the two engines answer differently.
pub fn engine_splits(layout: &Layout) -> Vec<Row> {
    let committed = layout.committed();
    if !layout.refreshing() {
        assert!(
            !committed.is_empty(),
            "{} is missing, so this reads green over nothing",
            layout.table
        );
    }
    committed
        .into_iter()
        .filter(|row| row.verdict == "engine-differs")
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{NONE, Receiver, program};

    fn plain(setup: &str) -> Receiver {
        Receiver {
            setup: setup.to_string(),
            wrapper: NONE.to_string(),
            directives: NONE.to_string(),
        }
    }

    /// The program text `corpus/collection-arity.tsv` was derived with, spelled
    /// out rather than described.
    ///
    /// Every byte of that committed table is downstream of this, so the shape
    /// is asserted rather than left to a diff someone remembers to run.
    #[test]
    fn a_receiver_with_no_wrapper_and_no_directives_writes_the_original_program() {
        assert_eq!(
            program(
                &plain("ka = 'k1' | r = .Table~new | r[ka] = 'v1'"),
                "at",
                "ka",
                false
            ),
            "ka = 'k1'\n\
             r = .Table~new\n\
             r[ka] = 'v1'\n\
             say 'SETUP-OK'\n\
             signal on syntax name oops\n\
             r~'at'(ka)\n\
             say 'SENT'\n\
             exit\n\
             oops:\n\
             say 'SYNTAX' condition('O')~code\n\
             exit 0\n"
        );
    }

    /// `--` for the argument list is a send with none, not a send of `--`.
    #[test]
    fn an_empty_argument_list_sends_no_parentheses() {
        assert!(
            program(&plain("r = .Table~new"), "items", NONE, false)
                .contains("\nr~'items'\nsay 'SENT'\n")
        );
    }

    /// Comparing values assigns the send and prints the answer, and reads the
    /// one code a method returning no result raises as a completed send.
    #[test]
    fn comparing_values_assigns_the_send_and_prints_the_answer() {
        assert_eq!(
            program(&plain("r = .Table~new"), "items", NONE, true),
            "r = .Table~new\n\
             say 'SETUP-OK'\n\
             signal on syntax name oops\n\
             vv = r~'items'\n\
             say 'SENT'\n\
             say 'VALUE' vv~string\n\
             exit\n\
             oops:\n\
             vc = rc || '.' || condition('E')\n\
             if vc = '91.999' then do\n\
             say 'SENT'\n\
             say 'NORESULT'\n\
             exit 0\n\
             end\n\
             say 'SYNTAX' vc\n\
             exit 0\n"
        );
    }

    /// The wrapper moves the setup and the send inside the call and leaves the
    /// trap in the caller, which is where the condition arrives.
    #[test]
    fn the_call_wrapper_puts_the_setup_and_the_send_inside_the_routine() {
        let text = program(
            &Receiver {
                setup: "r = .context".to_string(),
                wrapper: "call".to_string(),
                directives: NONE.to_string(),
            },
            "name",
            NONE,
            false,
        );
        assert_eq!(
            text,
            "signal on syntax name oops\n\
             call probe 'a1', 'a2'\n\
             say 'SENT'\n\
             exit\n\
             oops:\n\
             say 'SYNTAX' condition('O')~code\n\
             exit 0\n\
             probe:\n\
             r = .context\n\
             say 'SETUP-OK'\n\
             r~'name'\n\
             return\n"
        );
    }

    /// Directives are appended after the program, one per `|`-separated part.
    #[test]
    fn directives_are_appended_after_the_program() {
        let text = program(
            &Receiver {
                setup: "r = .K".to_string(),
                wrapper: NONE.to_string(),
                directives: "::class K subclass Object | ::method MM | return 2".to_string(),
            },
            "id",
            NONE,
            false,
        );
        assert!(
            text.ends_with("::class K subclass Object\n::method MM\nreturn 2\n"),
            "{text}"
        );
        assert!(text.starts_with("r = .K\nsay 'SETUP-OK'\n"), "{text}");
    }
}
