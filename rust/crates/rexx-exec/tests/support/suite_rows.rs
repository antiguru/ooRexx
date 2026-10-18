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

//! The three files of tests the suite gate does not require to pass (spec
//! 2026-10-10 section 2), and each owner's check. A row is
//! `dir/group<TAB>TEST<TAB>owner<TAB>reason`; `#` starts a comment line, and
//! each file holds one `# ootest revision N` line naming the revision its rows
//! were derived at. Declared with `#[path]` beside `group_runner`.

// Pulled in by more than one test binary, each of which uses part of it.
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::group_runner::{
    reaching_rxapi_in, read_lossy, source_classes, source_test_names_in, test_classes, worktree,
};

/// The Phase 10 rows, relative to the worktree; Phase 10 deletes the file.
pub const EXCLUSIONS: &str = "docs/superpowers/plans/phase-9-exclusions.txt";
/// The Deviation, Environment and Flaky rows.
pub const LICENCES: &str = "rust/corpus/suite-licences.txt";
/// The tests the framework does not run on this host.
pub const NOT_RUN: &str = "rust/corpus/suite-not-run.txt";
pub const FILES: [&str; 3] = [EXCLUSIONS, LICENCES, NOT_RUN];

/// The ootest revision the files must name, relative to the worktree.
pub const REVISION: &str = "rust/corpus/suite-revision.txt";

/// Where an Environment row's oracle run record lives, relative to the
/// worktree.
pub const ORACLE_RUNS: &str = "rust/corpus/suite-oracle-runs/";

/// The file of tests that fail on every CI platform, which a Flaky row cites.
pub const COMMON_FAILURES: &str = ".github/known-test-failures/common.txt";

/// The file whose `DEVIATIONS -- permanent, chosen` section a Deviation row
/// cites.
pub const DEVIATIONS: &str = "docs/superpowers/plans/phase-4-exclusions.txt";

/// Where the suite gate writes its table; the oracle-difference report, the
/// per-test record and the verdicts go beside it.
pub const TABLE_ENV: &str = "REXX_SUITE_TABLE";

/// The per-test record's name beside the table: `group`, `test`, the row's
/// owner or `-`, and each side's verdict.
pub const TESTS_FILE: &str = "tests.tsv";

/// The per-test record's line naming a run of every group; a run restricted
/// by `REXX_SUITE_GROUPS` writes `# groups ` and the names instead.
pub const ALL_GROUPS_LINE: &str = "# groups all";

/// Whether the per-test record `text` is of a run of every group at the ootest
/// revision `revision`.
pub fn full_run_record(text: &str, revision: &str) -> Result<(), String> {
    let wanted = format!("# ootest revision {revision}");
    if !text.lines().any(|line| line == wanted) {
        return Err(format!("the record does not say `{wanted}`"));
    }
    if !text.lines().any(|line| line == ALL_GROUPS_LINE) {
        return Err(format!(
            "the record is not of every group (no `{ALL_GROUPS_LINE}`)"
        ));
    }
    Ok(())
}

/// Where the table goes: [`TABLE_ENV`]'s path, else `target/suite/table.tsv`.
pub fn table_path() -> PathBuf {
    std::env::var_os(TABLE_ENV).map_or_else(
        || {
            PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
                .parent()
                .expect("the target directory")
                .join("suite/table.tsv")
        },
        PathBuf::from,
    )
}

/// Why a test is Phase 10's: spec D3's closed set, the one the refusals
/// print.
pub use rexx_core::Deferral;

/// A row's owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Phase10(Deferral),
    Deviation(u32),
    Environment,
    Flaky,
    NotRun,
}

impl Owner {
    pub fn parse(text: &str) -> Option<Owner> {
        if let Some(reason) = text.strip_prefix("Phase 10 ") {
            return Deferral::named(reason).map(Owner::Phase10);
        }
        if let Some(number) = text.strip_prefix("Deviation ") {
            return number
                .parse()
                .ok()
                .filter(|_| number.bytes().all(|byte| byte.is_ascii_digit()))
                .map(Owner::Deviation);
        }
        match text {
            "Environment" => Some(Owner::Environment),
            "Flaky" => Some(Owner::Flaky),
            "NotRun" => Some(Owner::NotRun),
            _ => None,
        }
    }

    /// The file a row of this owner belongs in.
    pub fn file(self) -> &'static str {
        match self {
            Owner::Phase10(_) => EXCLUSIONS,
            Owner::Deviation(_) | Owner::Environment | Owner::Flaky => LICENCES,
            Owner::NotRun => NOT_RUN,
        }
    }
}

/// One row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// The file it is in, relative to the worktree.
    pub file: String,
    pub line: usize,
    /// `dir/group`.
    pub group: String,
    /// Upper case.
    pub test: String,
    pub owner: Owner,
    pub reason: String,
}

impl Row {
    /// Where the row is, for a message.
    pub fn at(&self) -> String {
        format!("{}:{}", self.file, self.line)
    }

    /// The refusal a `refusal: TEXT` reason records.
    pub fn refusal(&self) -> Option<&str> {
        self.reason.strip_prefix("refusal: ")
    }

    /// Whether the harness renames this row's test out on both sides: an
    /// Environment row whose reason says the test hangs.
    pub fn renamed_out(&self) -> bool {
        self.owner == Owner::Environment && self.reason.starts_with("hangs")
    }
}

/// The rows of `text`, the file `file`, its `# ootest revision` and what is
/// malformed in it.
pub fn parse_rows(file: &str, text: &str) -> (Option<String>, Vec<Row>, Vec<String>) {
    let mut revision = None;
    let mut rows = Vec::new();
    let mut problems = Vec::new();
    for (at, line) in text.lines().enumerate() {
        let line_number = at + 1;
        if let Some(comment) = line.strip_prefix('#') {
            if let Some(found) = comment.trim().strip_prefix("ootest revision ")
                && revision.replace(found.trim().to_string()).is_some()
            {
                problems.push(format!("{file}:{line_number}: a second revision line"));
            }
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        let [group, test, owner, reason] = fields.as_slice() else {
            problems.push(format!(
                "{file}:{line_number}: not four tab-separated fields: {line:?}"
            ));
            continue;
        };
        let Some(owner) = Owner::parse(owner) else {
            problems.push(format!("{file}:{line_number}: no owner {owner:?}"));
            continue;
        };
        if owner.file() != file {
            problems.push(format!(
                "{file}:{line_number}: a {owner:?} row belongs in {}",
                owner.file()
            ));
        }
        if test.to_ascii_uppercase() != *test || reason.trim().is_empty() {
            problems.push(format!(
                "{file}:{line_number}: the test is not upper case or the reason is empty"
            ));
        }
        rows.push(Row {
            file: file.to_string(),
            line: line_number,
            group: (*group).to_string(),
            test: (*test).to_string(),
            owner,
            reason: (*reason).to_string(),
        });
    }
    if revision.is_none() {
        problems.push(format!("{file}: no `# ootest revision` line"));
    }
    (revision, rows, problems)
}

/// Every row of the three files below `worktree`, and what is malformed,
/// including a file whose revision is not [`REVISION`]'s.
pub fn read_rows_in(worktree: &Path) -> (Vec<Row>, Vec<String>) {
    let recorded = fs::read_to_string(worktree.join(REVISION)).expect("the recorded revision");
    let mut rows = Vec::new();
    let mut problems = Vec::new();
    for file in FILES {
        let text = fs::read_to_string(worktree.join(file))
            .unwrap_or_else(|e| panic!("cannot read {file}: {e}"));
        let (revision, more, malformed) = parse_rows(file, &text);
        if revision
            .as_deref()
            .is_some_and(|found| found != recorded.trim())
        {
            problems.push(format!(
                "{file} names revision {revision:?}, {REVISION} {:?}",
                recorded.trim()
            ));
        }
        rows.extend(more);
        problems.extend(malformed);
    }
    (rows, problems)
}

/// [`read_rows_in`] for this worktree.
pub fn read_rows() -> (Vec<Row>, Vec<String>) {
    read_rows_in(&worktree())
}

/// `rows` by `(group, test)`.
pub fn by_test(rows: &[Row]) -> BTreeMap<(String, String), &Row> {
    rows.iter()
        .map(|row| ((row.group.clone(), row.test.clone()), row))
        .collect()
}

/// The groups of the native-API partition's row 10, as `dir/group`.
pub const ROW_10_GROUPS: &[&str] = &[
    "API/classic/CLASSIC",
    "API/oo/INVOCATION",
    "API/oo/ProcessInvocation",
    "API/oo/ProcessRexxStart",
    "API/oo/RexxStart",
];

/// The `utilities` groups ruling R1 makes Phase 10's.
pub const R1_GROUPS: &[&str] = &[
    "utilities/rxapi",
    "utilities/rxqueue/rxQueue",
    "utilities/rxsubcom/rxsubcom",
];

/// The `extensions/` directories of pure-Rexx packages, which Moritz ruled
/// Phase 9's (2026-10-17).
pub const PHASE_9_EXTENSION_DIRS: &[&str] = &[
    "extensions/dateparser/",
    "extensions/json/",
    "extensions/yaml/",
];

/// The reason a group the roadmap makes Phase 10's is, or `None`.
pub fn phase_10_group(group: &str) -> Option<Deferral> {
    if group.starts_with("extensions/")
        && !PHASE_9_EXTENSION_DIRS
            .iter()
            .any(|dir| group.starts_with(dir))
    {
        Some(Deferral::Extension)
    } else if ROW_10_GROUPS.contains(&group) {
        Some(Deferral::Row10NativeApi)
    } else if R1_GROUPS.contains(&group) {
        Some(Deferral::Rxapi)
    } else {
        None
    }
}

/// Whether a captured refusal `text` carries a row owned by Phase 10 for
/// `reason`: it names Phase 10 with that reason in parentheses, as
/// `rexx_core::Owner::label` prints it. The one place the check of a refusal
/// row's reason is decided.
pub fn refusal_carries(text: &str, reason: Deferral) -> bool {
    text.contains(&format!("({})", rexx_core::Owner::Phase10(reason).label()))
}

/// What the owner checks need besides the rows.
pub struct Context {
    /// The directory the groups' `dir`s are below.
    pub root: PathBuf,
    /// Every group, as `dir/group`.
    pub groups: BTreeSet<String>,
    /// `reaching_rxapi` over every group, as `(dir/group, TEST)`.
    pub rxapi: BTreeSet<(String, String)>,
    /// The numbers of [`DEVIATIONS`]' section, withdrawn ones left out.
    pub deviations: BTreeSet<u32>,
    /// [`COMMON_FAILURES`]' entries, upper case.
    pub flaky: BTreeSet<String>,
    /// The test types the framework does not run by default.
    pub skipped_types: BTreeSet<String>,
    /// The worktree, for an Environment row's record.
    pub worktree: PathBuf,
}

impl Context {
    /// The context of `groups` (as `(dir, group)`) below `root`.
    pub fn new(root: &Path, groups: &[(String, String)]) -> Context {
        let worktree = worktree();
        let mut rxapi = BTreeSet::new();
        for (dir, group) in groups {
            let path = group_path(dir, group);
            for member in reaching_rxapi_in(root, dir, &[group.as_str()]) {
                let test = member
                    .strip_prefix(&format!("{group}."))
                    .expect("a GROUP.TEST name");
                rxapi.insert((path.clone(), test.to_string()));
            }
        }
        Context {
            root: root.to_path_buf(),
            groups: groups
                .iter()
                .map(|(dir, group)| group_path(dir, group))
                .collect(),
            rxapi,
            deviations: deviations(&read_lossy(&worktree.join(DEVIATIONS))),
            flaky: common_failures(&read_lossy(&worktree.join(COMMON_FAILURES))),
            skipped_types: skipped_types(&read_lossy(&worktree.join("ootest/ooTest.frm"))),
            worktree,
        }
    }

    fn group_file(&self, group: &str) -> PathBuf {
        self.root.join(format!("{group}.testGroup"))
    }
}

/// `dir/group`, or `group` for a group at the top.
pub fn group_path(dir: &str, group: &str) -> String {
    if dir.is_empty() {
        group.to_string()
    } else {
        format!("{dir}/{group}")
    }
}

/// The numbers under `text`'s `DEVIATIONS -- permanent, chosen` heading,
/// up to the next heading at the margin, other than those `WITHDRAWN`.
pub fn deviations(text: &str) -> BTreeSet<u32> {
    let mut lines = text
        .lines()
        .skip_while(|line| *line != "DEVIATIONS -- permanent, chosen")
        .skip(2);
    let mut numbers = BTreeSet::new();
    for line in lines.by_ref() {
        if !line.is_empty() && !line.starts_with(' ') {
            break;
        }
        let trimmed = line.trim_start();
        let Some((number, rest)) = trimmed.split_once(". ") else {
            continue;
        };
        if line.len() - trimmed.len() <= 2
            && !number.is_empty()
            && number.bytes().all(|byte| byte.is_ascii_digit())
            && !rest.starts_with("WITHDRAWN")
        {
            numbers.insert(number.parse().expect("a number"));
        }
    }
    numbers
}

/// The `GROUP.testGroup/TEST` entries of a known-failures file, upper case.
pub fn common_failures(text: &str) -> BTreeSet<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_ascii_uppercase)
        .collect()
}

/// The `ooTestTypes` constants of `ooTest.frm`'s `text` that its
/// `defaultTestSet` leaves out, upper case.
pub fn skipped_types(text: &str) -> BTreeSet<String> {
    let mut in_types = false;
    let mut types = BTreeSet::new();
    for line in text.lines() {
        let lower = line.trim_start().to_ascii_lowercase();
        if lower.starts_with("::class") {
            in_types = lower.contains("'ootesttypes'");
        } else if in_types
            && let Some(rest) = lower.strip_prefix("::constant")
            && let Some(name) = rest.split_whitespace().next()
            && name.ends_with("_test")
        {
            types.insert(name.to_ascii_uppercase());
        }
    }
    let start = text
        .find("tests = .set~of(")
        .expect("ooTest.frm's defaultTestSet");
    let end = start + text[start..].find(")\n").expect("the end of the set");
    let default: BTreeSet<String> = text[start..end]
        .split("self~")
        .skip(1)
        .map(|word| {
            word.trim_end_matches([',', ' ', '-', '\n', '\r'])
                .trim()
                .to_ascii_uppercase()
        })
        .collect();
    assert!(!default.is_empty(), "defaultTestSet names no type");
    types.retain(|name| !default.contains(name));
    types
}

/// What is wrong with `rows` against `context`: each row's owner check, a
/// test with more than one row, and the Rxapi rows against
/// `reaching_rxapi`'s output.
pub fn check_rows(rows: &[Row], context: &Context) -> Vec<String> {
    let mut problems = Vec::new();
    let mut seen: BTreeMap<(&str, &str), &Row> = BTreeMap::new();
    for row in rows {
        if let Some(first) = seen.insert((&row.group, &row.test), row) {
            problems.push(format!(
                "{}: {} {} has a row at {} too",
                row.at(),
                row.group,
                row.test,
                first.at()
            ));
        }
        if let Err(problem) = check_row(row, context) {
            problems.push(format!(
                "{}: {} {}: {problem}",
                row.at(),
                row.group,
                row.test
            ));
        }
    }
    let renamed: BTreeSet<(String, String)> = rows
        .iter()
        .filter(|row| row.reason == "reaching_rxapi")
        .map(|row| (row.group.clone(), row.test.clone()))
        .collect();
    for (group, test) in context.rxapi.difference(&renamed) {
        problems.push(format!(
            "{group} {test}: reaching_rxapi renames it out, and it has no reaching_rxapi row"
        ));
    }
    problems
}

/// One row's owner check.
pub fn check_row(row: &Row, context: &Context) -> Result<(), String> {
    if !context.groups.contains(&row.group) {
        return Err("no such group".to_string());
    }
    let text = read_lossy(&context.group_file(&row.group));
    let classes = source_classes(&text);
    let defined = classes
        .iter()
        .any(|class| class.methods.contains(&row.test));
    if !defined {
        return Err("the group file defines no such method".to_string());
    }
    match row.owner {
        Owner::Phase10(reason) => check_phase_10(row, reason, context),
        Owner::Deviation(number) => {
            if context.deviations.contains(&number) {
                Ok(())
            } else {
                Err(format!(
                    "{DEVIATIONS} has no Deviation {number} that is not withdrawn"
                ))
            }
        }
        Owner::Environment => check_environment(row, context),
        Owner::Flaky => {
            let stem = row.group.rsplit('/').next().expect("a group name");
            let entry = row
                .reason
                .strip_prefix("common.txt: ")
                .ok_or("the reason is not `common.txt: ENTRY`")?
                .to_ascii_uppercase();
            if !context.flaky.contains(&entry) {
                Err(format!("{COMMON_FAILURES} has no entry {entry}"))
            } else if entry != format!("{stem}.TESTGROUP/{}", row.test).to_ascii_uppercase() {
                Err(format!("{entry} is not this row's test"))
            } else {
                Ok(())
            }
        }
        Owner::NotRun => check_not_run(row, &text, context),
    }
}

fn check_phase_10(row: &Row, reason: Deferral, context: &Context) -> Result<(), String> {
    if row.reason == "reaching_rxapi" {
        return if reason != Deferral::Rxapi {
            Err("a reaching_rxapi row is owned by Phase 10 Rxapi".to_string())
        } else if context
            .rxapi
            .contains(&(row.group.clone(), row.test.clone()))
        {
            Ok(())
        } else {
            Err("reaching_rxapi does not rename it out".to_string())
        };
    }
    if row.reason == "group" {
        return match phase_10_group(&row.group) {
            Some(found) if found == reason => Ok(()),
            Some(found) => Err(format!("the group is Phase 10's for {}", found.name())),
            None => Err("the roadmap does not make the group Phase 10's".to_string()),
        };
    }
    match row.refusal() {
        Some(text) if refusal_carries(text, reason) => Ok(()),
        Some(text) => Err(format!(
            "the refusal does not carry Phase 10 {}: {text}",
            reason.name()
        )),
        None => Err("the reason is not reaching_rxapi, group or `refusal: TEXT`".to_string()),
    }
}

fn check_environment(row: &Row, context: &Context) -> Result<(), String> {
    let record = row
        .reason
        .split("; ")
        .find_map(|part| part.strip_prefix("record: "))
        .ok_or("no `record: PATH`")?;
    if !record.starts_with(ORACLE_RUNS) {
        return Err(format!("the record is not under {ORACLE_RUNS}"));
    }
    let tracked = Command::new("git")
        .arg("-C")
        .arg(&context.worktree)
        .args(["ls-files", "--error-unmatch", record])
        .output()
        .expect("cannot run git");
    if !tracked.status.success() {
        return Err(format!("{record} is not committed"));
    }
    if !read_lossy(&context.worktree.join(record)).contains(&row.test) {
        return Err(format!("{record} does not name the test"));
    }
    let ruled = row
        .reason
        .split("; ")
        .find_map(|part| part.strip_prefix("ruled: Moritz "))
        .ok_or("no `ruled: Moritz DATE`")?;
    let date: Vec<&str> = ruled.split('-').collect();
    if date.len() == 3
        && [4, 2, 2]
            .iter()
            .zip(&date)
            .all(|(len, part)| part.len() == *len && part.bytes().all(|b| b.is_ascii_digit()))
    {
        Ok(())
    } else {
        Err(format!("{ruled:?} is not a date"))
    }
}

fn check_not_run(row: &Row, text: &str, context: &Context) -> Result<(), String> {
    if let Some(line) = row.reason.strip_prefix("restrictOS line ") {
        let number: usize = line.parse().map_err(|_| "not a line number")?;
        let source = text
            .lines()
            .nth(number.wrapping_sub(1))
            .ok_or("no such line")?;
        let lower = source.to_ascii_lowercase();
        let at = lower
            .find("restrictos(")
            .ok_or("the line holds no restrictOS")?;
        let argument = source[at + "restrictos(".len()..]
            .trim_start()
            .trim_start_matches(['\'', '"']);
        let argument = argument
            .split(['\'', '"'])
            .next()
            .unwrap_or("")
            .to_ascii_uppercase();
        let words: Vec<&str> = argument.split_whitespace().collect();
        return if words.is_empty() || words == ["UNIX"] || words.contains(&"LINUX") {
            Err(format!("restrictOS({argument:?}) admits LINUX"))
        } else {
            Ok(())
        };
    }
    if let Some(name) = row.reason.strip_prefix("test type ") {
        if !context.skipped_types.contains(name) {
            return Err(format!("{name} is a type the framework runs by default"));
        }
        let classes = source_classes(text);
        let tests = test_classes(&classes);
        let assignment = format!("~{}", name.to_ascii_lowercase());
        let typed = classes.iter().any(|class| {
            tests.contains(&class.name)
                && class.methods.contains(&row.test)
                && class.body.contains("ootesttype")
                && class.body.contains(&assignment)
        });
        return if typed {
            Ok(())
        } else {
            Err(format!("no test class defining it sets its type to {name}"))
        };
    }
    Err("the reason is not `restrictOS line N` or `test type NAME`".to_string())
}

/// The rows the group files alone determine, one line each as the files hold
/// them: every test `reaching_rxapi` renames out; every other test of a
/// group the roadmap makes Phase 10's; every other test of a group whose
/// `restrictOS` leaves this host out; and every other test of a test class
/// whose type the framework skips by default.
pub fn derived_rows(context: &Context) -> Vec<String> {
    let mut lines = Vec::new();
    for group in &context.groups {
        let (dir, name) = group.rsplit_once('/').unwrap_or(("", group));
        let text = read_lossy(&context.group_file(group));
        let mut tests: BTreeSet<String> = source_test_names_in(&context.root, dir, name)
            .into_iter()
            .collect();
        for (rxapi_group, test) in &context.rxapi {
            if rxapi_group == group {
                tests.remove(test);
                lines.push(format!("{group}\t{test}\tPhase 10 Rxapi\treaching_rxapi"));
            }
        }
        if let Some(reason) = phase_10_group(group) {
            for test in &tests {
                lines.push(format!(
                    "{group}\t{test}\tPhase 10 {}\tgroup",
                    reason.name()
                ));
            }
            continue;
        }
        let restricted = text.lines().enumerate().find_map(|(at, line)| {
            let probe = Row {
                file: NOT_RUN.to_string(),
                line: 0,
                group: group.clone(),
                test: String::new(),
                owner: Owner::NotRun,
                reason: format!("restrictOS line {}", at + 1),
            };
            (line.to_ascii_lowercase().contains("restrictos(")
                && check_not_run(&probe, &text, context).is_ok())
            .then_some(at + 1)
        });
        if let Some(line) = restricted {
            for test in &tests {
                lines.push(format!("{group}\t{test}\tNotRun\trestrictOS line {line}"));
            }
            continue;
        }
        let classes = source_classes(&text);
        let test_class_names = test_classes(&classes);
        for class in classes
            .iter()
            .filter(|class| test_class_names.contains(&class.name))
        {
            let Some(name) = context.skipped_types.iter().find(|name| {
                class.body.contains("ootesttype")
                    && class
                        .body
                        .contains(&format!("~{}", name.to_ascii_lowercase()))
            }) else {
                continue;
            };
            for test in class.methods.iter().filter(|test| tests.contains(*test)) {
                lines.push(format!("{group}\t{test}\tNotRun\ttest type {name}"));
            }
        }
    }
    lines
}
