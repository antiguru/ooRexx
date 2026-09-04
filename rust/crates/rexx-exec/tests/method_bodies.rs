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

//! The method-body table (D76): whether each documented method **works**,
//! where gate table C only asks whether `hasMethod` answers for it.
//!
//! One row per (class, method, arm) of `corpus/docs/class-methods.txt`, each
//! classified by sending the documented name with no arguments to a real
//! receiver and comparing this crate against the oracle. The classification
//! is a property of that one send, not of the method at every arity.
//!
//! # The gate, and it is red in every mode
//!
//! A row that was not diverging may not start, and a row that was answering
//! may not stop -- [`regressed`] is the whole rule. A refusal that turns into
//! a wrong answer is the failure this table exists to catch, and it is not a
//! fact about any phase, so no gate mode relaxes it. A count of implemented
//! bodies is not a criterion here and is not asserted on.
//!
//! # Two passes, and only the second needs the oracle
//!
//! The refusal precedes argument handling -- `Interp::invoke` asks
//! `Interp::invocable` before the seam and before any arity check -- so a
//! zero-argument send classifies [`Body::Loud`] exactly, with no signature
//! source. Pass one is crate-only over every row; pass two runs the oracle
//! only for the rows pass one did not classify.
//!
//! # No verdict is kept unless it turns on the two interpreters alone
//!
//! A comparison says nothing where the answer behind it moves on its own, so
//! a second pass holds the crate's answer against the oracle under three
//! environments and asks the oracle to repeat itself before any divergence
//! is called. A row whose verdict turns on the clock, the zone, or the
//! oracle's own irreproducibility is [`Body::Unstable`] -- derived from the
//! runs, never from a list of names.
//!
//! # What [`Body::Answers`] is worth
//!
//! A method needing arguments is sent none, so both sides raise and agree on
//! the raise: `answers` says the two interpreters agree on this send, which
//! for such a row is agreement about an arity error rather than about a
//! result. The gated rule does not rest on it.
//!
//! # Refreshing it
//!
//! `REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec --test
//! method_bodies` rewrites `corpus/method-bodies.txt` from the run -- and
//! refuses to write a regression, so the baseline cannot be laundered by
//! regenerating it. Any other difference between the committed file and the
//! run, its header included, fails, so the table cannot go stale unnoticed.

mod gate_tables;
mod support;
mod watchdog;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use gate_tables::{
    Descriptors, Report, Structural, assert_no_structural_failures, compare_raw, excerpt, is_loud,
    refused_construct, verdict,
};
use rexx_exec::{Engine, Invocation, Outcome};
use support::oracle::{CppOutcome, did_not_finish, wrapped_exit_code};

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus")
}

/// The committed table, beside `corpus/builtin-status.txt` rather than under
/// `corpus/docs/`: that directory is asserted to hold exactly what
/// `rexx-extract-docs` writes, and this table is derived by running both
/// interpreters, which is what `builtin-status.txt` is too.
const TABLE_FILE: &str = "method-bodies.txt";

/// The env var that rewrites [`TABLE_FILE`] from the run.
const REFRESH_ENV: &str = "REXX_METHOD_BODIES_REFRESH";

/// The field separator the committed table shares with every other row set
/// here.
const FIELDS: usize = 5;

/// The placeholder a row with no evidence to carry writes, spelled as
/// `corpus/docs/`'s own.
const NO_EVIDENCE: &str = "-";

/// Which side's own answer moved, for an [`Body::Unstable`] row's evidence.
const UNSTABLE_CRATE: &str = "this crate";
const UNSTABLE_ORACLE: &str = "the oracle";

/// Two zones twenty-six hours apart, so that no instant puts them on the same
/// calendar date.
///
/// **That gap is what makes a row reading the wall clock score the same every
/// hour of every day.** This crate's `.DateTime~today` answers the UTC date
/// where the oracle answers the local one, so against this machine's zone
/// alone the two agree for twenty-two hours a day and differ for two -- a
/// verdict that turns on when the sweep ran. Against a pair that is never on
/// one date, the row can never match all three, and `diverge` is what it
/// reads at every hour.
const SHIFTED_ZONES: (&str, &str) = ("Etc/GMT+12", "Etc/GMT-14");

// ------------------------------------------------------------------ verdicts

/// What one row's zero-argument send did.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Body {
    /// This crate refused at `NOT_IMPLEMENTED_EXIT` with its own marker,
    /// rather than answering. The evidence column names the construct.
    Loud,
    /// The send reached a body and agreed with the oracle on all three
    /// descriptors. The evidence column is the status both sides gave.
    Answers,
    /// The send answered and disagreed with the oracle. The evidence column
    /// is which channels moved.
    Diverge,
    /// One side's own answer moved between two runs of it, so there is
    /// nothing reproducible to compare and neither `answers` nor `diverge`
    /// would be a claim the run made. The evidence column names the side.
    Unstable,
}

impl Body {
    fn label(self) -> &'static str {
        match self {
            Body::Loud => "loud",
            Body::Answers => "answers",
            Body::Diverge => "diverge",
            Body::Unstable => "unstable",
        }
    }

    /// Every cell there is, for a caller that enumerates them without knowing
    /// how many there are.
    fn all() -> &'static [Body] {
        &[Body::Loud, Body::Answers, Body::Diverge, Body::Unstable]
    }

    fn parse(text: &str) -> Option<Body> {
        Body::all()
            .iter()
            .copied()
            .find(|cell| cell.label() == text)
    }
}

/// Whether a row moving from `was` to `now` is the regression this table
/// gates.
///
/// Sixteen literal arms over the square, so exhaustiveness and non-overlap
/// are the compiler's rather than a claim in a comment. Two rules are in
/// here: a row that was not diverging may not start, and a row that was
/// answering may not stop.
fn regressed(was: Body, now: Body) -> bool {
    match (was, now) {
        (Body::Loud, Body::Loud) => false,
        (Body::Loud, Body::Answers) => false,
        (Body::Loud, Body::Diverge) => true,
        (Body::Loud, Body::Unstable) => false,
        (Body::Answers, Body::Loud) => true,
        (Body::Answers, Body::Answers) => false,
        (Body::Answers, Body::Diverge) => true,
        (Body::Answers, Body::Unstable) => true,
        (Body::Diverge, Body::Loud) => false,
        (Body::Diverge, Body::Answers) => false,
        (Body::Diverge, Body::Diverge) => false,
        (Body::Diverge, Body::Unstable) => false,
        (Body::Unstable, Body::Loud) => false,
        (Body::Unstable, Body::Answers) => false,
        (Body::Unstable, Body::Diverge) => true,
        (Body::Unstable, Body::Unstable) => false,
    }
}

// ---------------------------------------------------------------- the row set

/// Reads one committed row file as tab-separated fields, dropping comments
/// and blank lines and panicking on a row of the wrong width.
fn read_table(path: &Path, fields: usize) -> Vec<Vec<String>> {
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

/// One documented class, from `class-set.txt`.
struct ClassRow {
    name: String,
    /// The committed construction expression, or `None` where the row set
    /// offers no route to an instance.
    construction: Option<String>,
    /// The directive the probe carries below the send, for an expression that
    /// reads one.
    directive: Option<String>,
}

fn read_classes(corpus: &Path) -> Vec<ClassRow> {
    read_table(&corpus.join("docs/class-set.txt"), 9)
        .into_iter()
        .map(|row| ClassRow {
            name: row[0].clone(),
            construction: (row[6] != NO_EVIDENCE).then(|| row[6].clone()),
            directive: (row[7] != NO_EVIDENCE).then(|| row[7].clone()),
        })
        .collect()
}

/// One documented (class, method, arm), from `class-methods.txt`.
#[derive(Clone)]
struct MethodRow {
    class: String,
    method: String,
    arm: String,
}

impl MethodRow {
    fn key(&self) -> (String, String, String) {
        (self.class.clone(), self.method.clone(), self.arm.clone())
    }
}

fn read_method_rows(corpus: &Path) -> Vec<MethodRow> {
    read_table(&corpus.join("docs/class-methods.txt"), 7)
        .into_iter()
        .map(|row| MethodRow {
            class: row[0].clone(),
            method: row[1].clone(),
            arm: row[2].clone(),
        })
        .collect()
}

/// One row of the committed table.
struct Committed {
    verdict: Body,
    evidence: String,
}

/// The committed table, or an empty baseline when it does not exist yet and
/// this run is the one creating it.
fn read_committed(
    corpus: &Path,
    refreshing: bool,
) -> BTreeMap<(String, String, String), Committed> {
    let path = corpus.join(TABLE_FILE);
    if !path.is_file() {
        assert!(
            refreshing,
            "{} does not exist, so this table has no baseline and no move of a verdict can be \
             a regression. Create it with `{REFRESH_ENV}=1 cargo test --release -p rexx-exec \
             --test method_bodies`",
            path.display()
        );
        return BTreeMap::new();
    }
    read_table(&path, FIELDS)
        .into_iter()
        .map(|row| {
            let verdict = Body::parse(&row[3]).unwrap_or_else(|| {
                panic!(
                    "{TABLE_FILE}: row {:?} {:?} {:?} carries the verdict {:?}, which is not one \
                     of {:?}",
                    row[0],
                    row[1],
                    row[2],
                    row[3],
                    Body::all().iter().map(|c| c.label()).collect::<Vec<_>>()
                )
            });
            (
                (row[0].clone(), row[1].clone(), row[2].clone()),
                Committed {
                    verdict,
                    evidence: row[4].clone(),
                },
            )
        })
        .collect()
}

// ----------------------------------------------------------------- the probes

/// The two method names with no printable spelling, written in the row set as
/// the placeholders the class tables use.
///
/// Measured: the names are the empty string and a single blank, and
/// `.Object~method("")` and `.Object~method(" ")` both answer `The Method
/// class`. A name this does not recognise is passed through, which is right
/// for every other name in the row set.
fn method_name_literal(name: &str) -> &str {
    match name {
        "(abuttal)" => "",
        "(blank)" => " ",
        other => other,
    }
}

/// A class whose `class-set.txt` construction expression cannot be this
/// table's receiver, with the expression that replaces it.
///
/// `.DateTime~new` is the instant it is evaluated at, so a value-returning
/// method of that instance answers differently on two runs and the row's
/// verdict turns on the clock. Measured under the documented expression: this
/// crate's own two engines disagree on the `DateTime` rows whose answer
/// carries the time, and a row flips between two sweeps.
const RECEIVER_OVERRIDES: &[(&str, &str)] = &[(
    "DateTime",
    ".DateTime~fromIsoDate('2020-01-02T03:04:05.678901')",
)];

/// The expression a row's send is made to, or `None` for the class arm, which
/// sends to the class object itself.
fn instance_receiver(class: &ClassRow) -> String {
    if let Some((_, expression)) = RECEIVER_OVERRIDES
        .iter()
        .find(|(name, _)| *name == class.name)
    {
        return (*expression).to_string();
    }
    match &class.construction {
        Some(program) => program.clone(),
        None => format!(".{}~new", class.name),
    }
}

/// The text of one row's probe program.
///
/// This function is the definition of the program: nothing is committed for
/// it to drift from, and the receiver and the name both come from the row.
fn probe_text(class: &ClassRow, row: &MethodRow) -> String {
    let receiver = match row.arm.as_str() {
        "class" => format!(".{}", class.name),
        _ => instance_receiver(class),
    };
    let mut text = format!(
        "/* Method-body row: {} {} ({} arm). The documented name is sent with no\n\
         \x20  arguments to `{receiver}`. Derived by\n\
         \x20  crates/rexx-exec/tests/method_bodies.rs on every run. */\n",
        class.name, row.method, row.arm
    );
    text.push_str(&format!("o = {receiver}\n"));
    text.push_str(&format!("say o~'{}'()\n", method_name_literal(&row.method)));
    if row.arm != "class"
        && let Some(directive) = &class.directive
    {
        text.push_str(&format!("{directive}\n"));
    }
    text
}

// -------------------------------------------------------------- the two sides

/// What this crate did with one probe, and whether its own answer was
/// reproducible.
struct CrateSide {
    outcome: Outcome,
    stable: bool,
}

/// Runs one probe on both engines, bounded, and reports whether the two agreed.
///
/// A disagreement is re-measured on one engine before it is believed: two
/// runs of the same engine that also disagree mean the program's own answer
/// moves between runs, which is a row this table cannot classify rather than
/// an engine defect. Panics when the two engines disagree while two runs of
/// one engine agree, and when either arm refused a body to the other --
/// neither is a fact about the oracle and no mode relaxes them.
fn run_both_engines(abs: &Path, text: &[u8]) -> CrateSide {
    let path = abs
        .to_str()
        .unwrap_or_else(|| panic!("probe path {} is not valid UTF-8", abs.display()));
    let run = |engine| -> Outcome {
        watchdog::run_bounded(path, text.to_vec(), Invocation::none().with_engine(engine))
    };
    let tree_walker = run(Engine::TreeWalker);
    let ir = run(Engine::Ir);
    assert!(
        ir.chunks_refused == 0 && tree_walker.chunks_refused == 0,
        "a body of {path} did not run on the engine it was attributed to: the ir arm refused \
         {} bodies to the tree-walker and the tree-walker arm counted {}, where it compiles \
         nothing to refuse",
        ir.chunks_refused,
        tree_walker.chunks_refused,
    );
    if same_outcome(&tree_walker, &ir) {
        return CrateSide {
            outcome: tree_walker,
            stable: true,
        };
    }
    let again = run(Engine::TreeWalker);
    assert!(
        !same_outcome(&tree_walker, &again),
        "the two engines disagree on {path}, which is a structural failure and not a verdict \
         -- two runs of the tree-walker agree with each other.\n  tree-walker: exit={} \
         stdout={} stderr={}\n  ir:          exit={} stdout={} stderr={}",
        wrapped_exit_code(tree_walker.exit_code),
        excerpt(&tree_walker.stdout),
        excerpt(&tree_walker.stderr),
        wrapped_exit_code(ir.exit_code),
        excerpt(&ir.stdout),
        excerpt(&ir.stderr),
    );
    CrateSide {
        outcome: tree_walker,
        stable: false,
    }
}

/// Whether two crate runs agree on all three descriptors.
fn same_outcome(left: &Outcome, right: &Outcome) -> bool {
    wrapped_exit_code(left.exit_code) == wrapped_exit_code(right.exit_code)
        && left.stdout == right.stdout
        && left.stderr == right.stderr
}

/// A row this crate answered, held between the oracle's first run and the
/// second pass that decides what its verdict is allowed to be.
struct Pending {
    key: (String, String, String),
    subject: String,
    abs: PathBuf,
    crate_side: Outcome,
    oracle: CppOutcome,
}

/// The channels a `diverge` row moved on **in any of the three
/// environments**, which is the comparison its verdict rests on.
///
/// **Against this machine's zone alone the answer turns on the hour.**
/// `DateTime today` answers the UTC date here and the local one on the
/// oracle, so the two agree in this zone for twenty-two hours a day: measured
/// 2026-09-04, that row derives `agree` at 07:03 CEST and `diverge-stdout`
/// around local midnight, while its verdict is `diverge` at every hour. A
/// committed evidence value that moves by itself is a daily gate failure, so
/// the union is what the table carries.
fn channels_that_moved_anywhere(row: &Pending, shifted: &[(&str, CppOutcome)]) -> Descriptors {
    let mut moved = compare_raw(&row.crate_side, &row.oracle);
    for (_, out) in shifted {
        let also = compare_raw(&row.crate_side, out);
        moved.status |= also.status;
        moved.stdout |= also.stdout;
        moved.stderr |= also.stderr;
    }
    moved
}

/// Whether two oracle runs of one probe produced the same three descriptors.
fn same_oracle(left: &CppOutcome, right: &CppOutcome) -> bool {
    left.termination == right.termination
        && left.stdout == right.stdout
        && left.stderr == right.stderr
}

/// One finished row.
struct Measured {
    verdict: Body,
    evidence: String,
    /// Both sides rendered, for the rows the report prints in full: a
    /// divergence, and a row whose own answer moved.
    detail: Option<String>,
}

// ------------------------------------------------------------------- the sweep

/// Where the derived probes are written, one directory per row so that an
/// oracle run's own working directory holds exactly the program it runs.
///
/// Under Cargo's per-target temporary directory and named with the pid and
/// the clock, as `builtin_status.rs`'s `fresh_run_root` is and for its
/// reason: a shared fixed path would let one run's probes appear in another
/// run's external-routine search path.
fn staging_root() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("method-bodies-{}-{nanos}", std::process::id()))
}

/// Every `class-set.txt` name a [`RECEIVER_OVERRIDES`] entry claims exists.
fn check_overrides(classes: &[ClassRow], structural: &mut Vec<Structural>) {
    let names: BTreeSet<&str> = classes.iter().map(|row| row.name.as_str()).collect();
    for (name, _) in RECEIVER_OVERRIDES {
        if !names.contains(name) {
            structural.push(Structural {
                subject: format!("RECEIVER_OVERRIDES entry {name}"),
                detail: "names a class class-set.txt does not carry, so the override applies to \
                         no row and the class it was written for is being sent the documented \
                         expression after all"
                    .to_string(),
            });
        }
    }
}

/// Every instance receiver this table sends to is the one gate table C's
/// committed probe constructs, or an override.
///
/// The two tables read the same `construction` column through two
/// derivations, and a receiver that drifted apart would leave this table
/// classifying a method on an object table C never asks about.
fn check_receivers_match_table_c(
    corpus: &Path,
    classes: &[ClassRow],
    arms: &BTreeSet<(String, String)>,
    structural: &mut Vec<Structural>,
) {
    let overridden: BTreeSet<&str> = RECEIVER_OVERRIDES.iter().map(|(name, _)| *name).collect();
    for class in classes {
        if overridden.contains(class.name.as_str())
            || !arms.contains(&(class.name.clone(), "instance".to_string()))
        {
            continue;
        }
        let probe = corpus.join(format!(
            "gate-tables/methods/{}__instance.rex",
            class.name.to_ascii_lowercase()
        ));
        let Ok(committed) = fs::read_to_string(&probe) else {
            structural.push(Structural {
                subject: format!("class-set.txt row {}", class.name),
                detail: format!(
                    "gate table C has instance-arm rows for it but {} cannot be read, so this \
                     table's receiver cannot be held against the one table C constructs",
                    probe.display()
                ),
            });
            continue;
        };
        let theirs = committed
            .lines()
            .find_map(|line| line.strip_prefix("o = "))
            .unwrap_or("<no `o = ` line>");
        let ours = instance_receiver(class);
        if theirs != ours {
            structural.push(Structural {
                subject: format!("class-set.txt row {}", class.name),
                detail: format!(
                    "this table sends to `{ours}` and gate table C's committed probe constructs \
                     `{theirs}`. The two derive the receiver from the same column, so a \
                     difference means one of them is asking about an object the other never \
                     sees"
                ),
            });
        }
    }
}

/// The committed table's row set is the documented row set, both directions.
fn check_row_set(
    rows: &[MethodRow],
    committed: &BTreeMap<(String, String, String), Committed>,
    structural: &mut Vec<Structural>,
) {
    let derived: BTreeSet<(String, String, String)> = rows.iter().map(MethodRow::key).collect();
    let held: BTreeSet<(String, String, String)> = committed.keys().cloned().collect();
    for missing in derived.difference(&held) {
        structural.push(Structural {
            subject: format!("{} {} ({})", missing.0, missing.1, missing.2),
            detail: format!(
                "class-methods.txt documents this row and {TABLE_FILE} does not carry it, so \
                 nothing holds a baseline for it and no move of its verdict can be a regression"
            ),
        });
    }
    for extra in held.difference(&derived) {
        structural.push(Structural {
            subject: format!("{} {} ({})", extra.0, extra.1, extra.2),
            detail: format!(
                "{TABLE_FILE} carries a row class-methods.txt does not document, so nothing \
                 runs it and its verdict is about nothing"
            ),
        });
    }
}

/// The committed table's text, header included.
fn render(rows: &[MethodRow], measured: &BTreeMap<(String, String, String), Measured>) -> String {
    let mut text = String::from(TABLE_HEADER);
    for row in rows {
        let Some(cell) = measured.get(&row.key()) else {
            continue;
        };
        writeln!(
            &mut text,
            "{}\t{}\t{}\t{}\t{}",
            row.class,
            row.method,
            row.arm,
            cell.verdict.label(),
            cell.evidence
        )
        .unwrap();
    }
    text
}

/// The committed table's header, which is where a reader of the data meets
/// the four verdicts and the rule they are gated by.
const TABLE_HEADER: &str = "\
# The method-body table (D76): one row per (class, method, arm) of
# corpus/docs/class-methods.txt, classified by sending the documented name
# with NO ARGUMENTS to a real receiver -- the class object for the class arm,
# and class-set.txt's construction expression for the instance arm.
#
# One `class<TAB>method<TAB>arm<TAB>verdict<TAB>evidence` per line, in
# class-methods.txt's own order.
#
# `loud`     this crate refused rather than answering, and `evidence` is the
#            construct its message named. This is the safety property the
#            gate protects: an unimplemented method is loud, never wrong.
# `answers`  the send reached a body and agreed with the oracle on all three
#            descriptors, and `evidence` is the status both sides gave. A
#            method needing arguments is sent none, so for such a row this is
#            agreement about an arity error rather than about a result.
# `diverge`  the send answered and disagreed, and `evidence` names the
#            channels that moved in ANY of the three environments below --
#            a union, because a row that agrees in this machine's zone and
#            disagrees in another would otherwise carry an evidence value
#            that changes with the hour.
# `unstable` the answer a verdict would have rested on is not reproducible,
#            so neither `answers` nor `diverge` would be a claim the run
#            made. `evidence` names what moved: `this crate` (its two engine
#            runs differ) or `the oracle` (its own two runs differ).
#
# `answers` is agreement under THREE environments -- this machine's zone and
# two more twenty-six hours apart, so that no instant puts them on one
# calendar date. Anything less is `diverge`: the crate answers one string, so
# a zone it fails to match is a zone it is wrong under, and a row that read
# the wall clock would otherwise agree for twenty-two hours a day and diverge
# for two.
#
# THE GATE IS ONE RULE, and it is red in every mode: a row that was not
# diverging may not start, and a row that was answering may not stop. A count
# of implemented bodies is not a criterion.
# crates/rexx-exec/tests/method_bodies.rs is the derivation; refresh with
#   REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec \\
#       --test method_bodies
# which refuses to write a regression rather than laundering one.
#
# The classification is of the zero-argument send alone. A `loud` row may
# answer at another arity, and an `answers` row may refuse at one.
#
# It is also a measurement on this machine. Some `DateTime` rows turn on the
# local timezone: measured, `utcIsoDate` reads `answers` under TZ=UTC and
# `diverge` under CEST+0200, because the oracle applies the offset and this
# crate does not.
";

#[test]
fn no_row_started_diverging_or_stopped_answering() {
    let oracle = support::oracle::locate();
    let corpus = corpus_dir();
    let mut structural = Vec::new();

    let refreshing =
        matches!(std::env::var(REFRESH_ENV), Ok(value) if !value.is_empty() && value != "0");
    let classes = read_classes(&corpus);
    let rows = read_method_rows(&corpus);
    let committed = read_committed(&corpus, refreshing);
    let committed_text = fs::read_to_string(corpus.join(TABLE_FILE)).unwrap_or_default();

    let by_name: BTreeMap<&str, &ClassRow> =
        classes.iter().map(|row| (row.name.as_str(), row)).collect();
    let arms: BTreeSet<(String, String)> = rows
        .iter()
        .map(|row| (row.class.clone(), row.arm.clone()))
        .collect();
    check_overrides(&classes, &mut structural);
    check_receivers_match_table_c(&corpus, &classes, &arms, &mut structural);
    // Not while refreshing: a row set that has moved is exactly what the
    // refresh is for, and reporting it as structural would leave the table
    // unwritable by the one command that fixes it.
    if !refreshing {
        check_row_set(&rows, &committed, &mut structural);
    }

    let staging = staging_root();
    let mut measured: BTreeMap<(String, String, String), Measured> = BTreeMap::new();
    let mut pending: Vec<Pending> = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        let Some(class) = by_name.get(row.class.as_str()) else {
            structural.push(Structural {
                subject: format!("{} {} ({})", row.class, row.method, row.arm),
                detail: "class-methods.txt names a class class-set.txt does not carry, so no \
                         receiver can be derived for it"
                    .to_string(),
            });
            continue;
        };
        let text = probe_text(class, row);
        let dir = staging.join(format!("{index:04}"));
        fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
        let file = dir.join("probe.rex");
        fs::write(&file, &text).unwrap_or_else(|e| panic!("cannot write {}: {e}", file.display()));
        let abs = fs::canonicalize(&file)
            .unwrap_or_else(|e| panic!("cannot canonicalize {}: {e}", file.display()));

        let subject = format!("{} {} ({})", row.class, row.method, row.arm);
        let crate_side = run_both_engines(&abs, text.as_bytes());
        if !crate_side.stable {
            measured.insert(
                row.key(),
                Measured {
                    verdict: Body::Unstable,
                    evidence: UNSTABLE_CRATE.to_string(),
                    detail: Some(format!(
                        "two runs of one engine differ; the last was rc={} out={} err={}",
                        wrapped_exit_code(crate_side.outcome.exit_code),
                        excerpt(&crate_side.outcome.stdout),
                        excerpt(&crate_side.outcome.stderr),
                    )),
                },
            );
            continue;
        }
        if is_loud(&crate_side.outcome) {
            measured.insert(
                row.key(),
                Measured {
                    verdict: Body::Loud,
                    evidence: refused_construct(&crate_side.outcome.stderr)
                        .unwrap_or_else(|| NO_EVIDENCE.to_string()),
                    detail: None,
                },
            );
            continue;
        }
        let cpp: CppOutcome = oracle.run(&abs);
        if did_not_finish(&cpp) {
            structural.push(Structural {
                subject: subject.clone(),
                detail: format!(
                    "the oracle did not finish: {:?}. Whatever it left on its descriptors is \
                     where it was interrupted, not an answer to compare",
                    cpp.termination
                ),
            });
            continue;
        }
        pending.push(Pending {
            key: row.key(),
            subject,
            abs,
            crate_side: crate_side.outcome,
            oracle: cpp,
        });
    }

    // **A second pass, which asks whether each row's verdict turns on
    // anything but the two interpreters.**
    //
    // A comparison is worth nothing where the answer behind it moves on its
    // own, and two things move it here. The oracle's answer can be
    // irreproducible -- `Object~identityHash` is derived from an address. And
    // the comparison can turn on the clock and the zone, which is worse,
    // because it makes a row's verdict a fact about when the sweep ran.
    //
    // So the crate's one answer is held against the oracle under this
    // machine's zone and under [`SHIFTED_ZONES`], and only agreement with all
    // three is `answers`. Anything less is a divergence: the crate's answer
    // is one string, so a zone it fails to match is a zone it is wrong under.
    //
    // **What this cannot see, and it will matter later.** The environment is
    // shifted on the oracle's side only, because the crate runs in process
    // and `TZ` is read per process. A body that correctly answers differently
    // per zone would therefore match at most one of the three and read
    // `diverge` here. Nothing in this crate does that today -- its
    // `DateTime` bodies ignore the offset, which is the defect above -- and
    // the task that lands one has to shift both sides rather than trust this.
    for row in &pending {
        let shifted = [
            (
                SHIFTED_ZONES.0,
                oracle.run_in_zone(&row.abs, SHIFTED_ZONES.0),
            ),
            (
                SHIFTED_ZONES.1,
                oracle.run_in_zone(&row.abs, SHIFTED_ZONES.1),
            ),
        ];
        if let Some((zone, unfinished)) = shifted.iter().find(|(_, out)| did_not_finish(out)) {
            structural.push(Structural {
                subject: row.subject.clone(),
                detail: format!(
                    "the oracle did not finish under TZ={zone}: {:?}, so whether this row's \
                     verdict turns on the environment has no answer",
                    unfinished.termination
                ),
            });
            continue;
        }
        let agrees_with = |cpp: &CppOutcome| {
            verdict(compare_raw(&row.crate_side, cpp)) == gate_tables::Verdict::Agree
        };
        if agrees_with(&row.oracle) && shifted.iter().all(|(_, out)| agrees_with(out)) {
            measured.insert(
                row.key.clone(),
                Measured {
                    verdict: Body::Answers,
                    evidence: format!("rc {}", wrapped_exit_code(row.crate_side.exit_code)),
                    detail: None,
                },
            );
            continue;
        }
        // The oracle is asked to repeat itself before this run is allowed to
        // call a divergence, and only here, where a divergence is what it
        // would otherwise call.
        let again: CppOutcome = oracle.run(&row.abs);
        if did_not_finish(&again) {
            structural.push(Structural {
                subject: row.subject.clone(),
                detail: format!(
                    "the oracle did not finish its repeat run: {:?}, so whether it reproduces \
                     this row is unknown and no divergence can rest on it",
                    again.termination
                ),
            });
            continue;
        }
        measured.insert(
            row.key.clone(),
            if same_oracle(&row.oracle, &again) {
                Measured {
                    verdict: Body::Diverge,
                    evidence: verdict(channels_that_moved_anywhere(row, &shifted))
                        .label()
                        .to_string(),
                    detail: Some(format!(
                        "oracle rc={} out={} err={}\n            crate  rc={} out={} err={}\n\
                         \x20           under TZ={} the oracle answers {} and under TZ={} {}",
                        row.oracle.expect_exit_code(),
                        excerpt(&row.oracle.stdout),
                        excerpt(&row.oracle.stderr),
                        wrapped_exit_code(row.crate_side.exit_code),
                        excerpt(&row.crate_side.stdout),
                        excerpt(&row.crate_side.stderr),
                        shifted[0].0,
                        excerpt(&shifted[0].1.stdout),
                        shifted[1].0,
                        excerpt(&shifted[1].1.stdout),
                    )),
                }
            } else {
                Measured {
                    verdict: Body::Unstable,
                    evidence: UNSTABLE_ORACLE.to_string(),
                    detail: Some(format!(
                        "the oracle answered {} and then {}",
                        excerpt(&row.oracle.stdout),
                        excerpt(&again.stdout),
                    )),
                }
            },
        );
    }

    // ---------------------------------------------------------------- report

    let mut report = Report::new(
        "rexx-exec method-body table (D76) -- whether each documented method works \
         (corpus/method-bodies.txt)",
    );
    report.line(&format!(
        "{} row(s) over corpus/docs/class-methods.txt, one zero-argument send each. \
         The oracle ran {} time(s): the rows pass one did not classify.",
        rows.len(),
        oracle.invocations(),
    ));

    let mut by_verdict: BTreeMap<&str, usize> = BTreeMap::new();
    for cell in measured.values() {
        *by_verdict.entry(cell.verdict.label()).or_insert(0) += 1;
    }
    report.line("");
    report.line("verdicts, over every row of this table:");
    for (label, count) in &by_verdict {
        report.line(&format!("  {label}: {count}"));
    }

    report.line("");
    report.line("by (class, arm) -- rows, and what they did:");
    let mut groups: BTreeMap<(String, String), BTreeMap<&str, usize>> = BTreeMap::new();
    for row in &rows {
        if let Some(cell) = measured.get(&row.key()) {
            *groups
                .entry((row.class.clone(), row.arm.clone()))
                .or_default()
                .entry(cell.verdict.label())
                .or_insert(0) += 1;
        }
    }
    for ((class, arm), counts) in &groups {
        let total: usize = counts.values().sum();
        let summary: Vec<String> = counts
            .iter()
            .map(|(label, count)| format!("{label}={count}"))
            .collect();
        report.line(&format!(
            "  {class:<28} {arm:<9} {total:<4} row(s): {}",
            summary.join(" ")
        ));
    }

    report.line("");
    report.line("every row this crate answered without agreeing, and every unstable row:");
    for row in &rows {
        let Some(cell) = measured.get(&row.key()) else {
            continue;
        };
        let Some(detail) = &cell.detail else { continue };
        report.line(&format!(
            "  {:<9} {} {} ({} arm) [{}]",
            cell.verdict.label(),
            row.class,
            row.method,
            row.arm,
            cell.evidence
        ));
        report.line(&format!("            {detail}"));
    }

    // Every difference between the committed table and this run, whether or
    // not it is a regression: a baseline that no longer describes the tree
    // asserts `loud` over rows nobody has looked at since.
    let mut drifted: Vec<String> = Vec::new();
    let mut regressions: Vec<String> = Vec::new();
    for row in &rows {
        let (Some(cell), Some(was)) = (measured.get(&row.key()), committed.get(&row.key())) else {
            continue;
        };
        if regressed(was.verdict, cell.verdict) {
            regressions.push(format!(
                "{} {} ({} arm): {} -> {} [{}]",
                row.class,
                row.method,
                row.arm,
                was.verdict.label(),
                cell.verdict.label(),
                cell.evidence
            ));
        } else if was.verdict != cell.verdict || was.evidence != cell.evidence {
            drifted.push(format!(
                "{} {} ({} arm): {} [{}] -> {} [{}]",
                row.class,
                row.method,
                row.arm,
                was.verdict.label(),
                was.evidence,
                cell.verdict.label(),
                cell.evidence
            ));
        }
    }

    report.line("");
    report.line(&format!(
        "regressions this run: {}. other drift from the committed table: {}.",
        regressions.len(),
        drifted.len()
    ));
    for line in regressions.iter().chain(drifted.iter()).take(40) {
        report.line(&format!("  {line}"));
    }

    gate_tables::emit_uncaptured(&report.finish());

    // The regression rule first and unconditionally, and before the refresh
    // writes anything: a baseline that could be rewritten over a regression
    // would be a gate that regenerating turns off.
    assert!(
        regressions.is_empty(),
        "{} row(s) of the method-body table regressed -- a row that refused now answers \
         something the oracle does not, or a row that answered has stopped. This is red in \
         every mode and {REFRESH_ENV} will not write it:\n  {}",
        regressions.len(),
        regressions.join("\n  "),
    );

    assert_no_structural_failures(&structural);

    let derived_text = render(&rows, &measured);
    if refreshing {
        let path = corpus.join(TABLE_FILE);
        fs::write(&path, &derived_text)
            .unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
    }

    // The whole file rather than its rows alone, so the header -- this
    // table's only definition of what its four verdicts mean -- cannot move
    // without a run saying so.
    assert!(
        refreshing || (drifted.is_empty() && committed_text == derived_text),
        "{TABLE_FILE} is not what this run derives, so the committed table no longer describes \
         the tree: {} row(s) drifted{}. Refresh it with `{REFRESH_ENV}=1 cargo test --release \
         -p rexx-exec --test method_bodies` and read the diff:\n  {}",
        drifted.len(),
        if drifted.is_empty() {
            ", and its text still differs, so what moved is the header"
        } else {
            ""
        },
        drifted.join("\n  "),
    );

    let _ = fs::remove_dir_all(&staging);
}

/// [`regressed`] is the two rules its doc states, checked over the whole
/// square from outside the `match`.
#[test]
fn the_regression_rule_is_the_two_rules_it_states() {
    for &was in Body::all() {
        for &now in Body::all() {
            let starts_diverging = now == Body::Diverge && was != Body::Diverge;
            let stops_answering = was == Body::Answers && now != Body::Answers;
            assert_eq!(
                regressed(was, now),
                starts_diverging || stops_answering,
                "{was:?} -> {now:?}"
            );
        }
    }
}

/// Every verdict round-trips through the label the committed table carries,
/// so a cell cannot be written under a name the reader does not know.
#[test]
fn every_verdict_parses_back_from_its_label() {
    for &cell in Body::all() {
        assert_eq!(Body::parse(cell.label()), Some(cell));
    }
    assert_eq!(Body::parse("agree"), None);
}
