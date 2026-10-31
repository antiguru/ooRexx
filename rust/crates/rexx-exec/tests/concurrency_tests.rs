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

//! Phase 6 exit criterion 1's test list, derived per test method from the
//! ooTest sources and held equal to the one in
//! `docs/superpowers/plans/phase-6-pinning.md`; with the `pinning` feature, the
//! would-be park points each of those tests reaches.

#![expect(
    clippy::disallowed_methods,
    reason = "this harness times or bounds real runs"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Overrides the directory the derivation walks, for the negative control.
const ROOT_ENV: &str = "REXX_CONCURRENCY_ROOT";

fn worktree() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn ootest_root() -> PathBuf {
    env::var_os(ROOT_ENV).map_or_else(|| worktree().join("ootest/ooRexx"), PathBuf::from)
}

fn pinning_doc() -> PathBuf {
    worktree().join("docs/superpowers/plans/phase-6-pinning.md")
}

/// Directories whose groups are excluded whole, with the reason.
const EXCLUDED_DIRS: &[(&str, &str)] = &[
    ("extensions/", "extension Phase 10 recompiles"),
    ("samples/", "sample Phase 10 recompiles"),
];

/// A `::METHOD`, `::ROUTINE` or `::RESOURCE` of a group file.
#[derive(Debug)]
struct Unit {
    name: String,
    class: Option<String>,
    test: bool,
    /// A class method `ACTIVATE` or `INIT`, which runs whichever test runs.
    activation: bool,
    /// Lower case, comments removed, string contents removed.
    code: String,
    /// Lower case, comments removed.
    text: String,
}

/// One derived test: the group's path below the root, the method name upper
/// case, and the features its body or a helper uses.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Derived {
    group: String,
    test: String,
    features: BTreeSet<&'static str>,
}

/// `line` with comments removed, and string contents too when `strings` is
/// false; `depth` carries an open block comment between lines.
fn lex_line(line: &str, depth: &mut usize, strings: bool) -> String {
    let bytes = line.as_bytes();
    let mut out = String::new();
    let mut at = 0;
    while at < bytes.len() {
        if *depth > 0 {
            if bytes[at..].starts_with(b"*/") {
                *depth -= 1;
                at += 2;
            } else if bytes[at..].starts_with(b"/*") {
                *depth += 1;
                at += 2;
            } else {
                at += 1;
            }
            continue;
        }
        if bytes[at..].starts_with(b"/*") {
            *depth += 1;
            at += 2;
            out.push(' ');
            continue;
        }
        if bytes[at..].starts_with(b"--") {
            break;
        }
        let c = bytes[at];
        if c == b'\'' || c == b'"' {
            let close = bytes[at + 1..].iter().position(|&b| b == c);
            let end = close.map_or(bytes.len(), |close| at + 1 + close + 1);
            if strings {
                out.push_str(&line[at..end]);
            } else {
                out.push_str("''");
            }
            at = end;
            continue;
        }
        out.push(char::from(c).to_ascii_lowercase());
        at += 1;
    }
    out
}

/// The name a directive line gives, lower case, quotes removed.
fn directive_name(rest: &str) -> String {
    let rest = rest.trim_start();
    let name = match rest.chars().next() {
        Some(quote @ ('\'' | '"')) => rest[1..].split(quote).next().unwrap_or(""),
        _ => rest.split_whitespace().next().unwrap_or(""),
    };
    name.to_ascii_lowercase()
}

/// Every unit of a group file's text.
fn units(text: &str) -> Vec<Unit> {
    let mut units = Vec::new();
    let mut current: Option<Unit> = None;
    let mut class: Option<String> = None;
    let mut tests: BTreeSet<String> = BTreeSet::new();
    let mut resource_end: Option<String> = None;
    let (mut depth_code, mut depth_text) = (0, 0);
    for line in text.lines() {
        let trimmed = line.trim_start();
        let lower = trimmed.to_ascii_lowercase();
        let in_comment = depth_code > 0;
        let code = lex_line(line, &mut depth_code, false);
        let text = lex_line(line, &mut depth_text, true);
        if let Some(end) = &resource_end {
            if lower.starts_with(end.as_str()) {
                resource_end = None;
                units.extend(current.take());
                (depth_code, depth_text) = (0, 0);
                continue;
            }
        } else if let Some(directive) = lower.strip_prefix("::").filter(|_| !in_comment) {
            units.extend(current.take());
            let rest = &trimmed[2..];
            let keyword = directive.split_whitespace().next().unwrap_or("");
            let after = rest.trim_start()[keyword.len()..].to_string();
            let name = directive_name(&after);
            match keyword {
                "class" => {
                    let words: Vec<&str> = directive.split_whitespace().collect();
                    let parent = words
                        .iter()
                        .position(|word| *word == "subclass")
                        .and_then(|at| words.get(at + 1))
                        .map(|word| word.trim_matches(['\'', '"']).to_string());
                    if parent
                        .as_deref()
                        .is_some_and(|parent| parent == "ootestcase" || tests.contains(parent))
                    {
                        tests.insert(name.clone());
                    }
                    class = Some(name);
                }
                "method" | "routine" | "resource" => {
                    let test = keyword == "method"
                        && name.starts_with("test")
                        && class.as_ref().is_some_and(|class| tests.contains(class));
                    if keyword == "resource" {
                        let words: Vec<&str> = directive.split_whitespace().collect();
                        let end = words
                            .iter()
                            .position(|word| *word == "end")
                            .and_then(|at| words.get(at + 1))
                            .map_or("::end".to_string(), |word| {
                                word.trim_matches(['\'', '"']).to_string()
                            });
                        resource_end = Some(end);
                    }
                    let activation = keyword == "method"
                        && (name == "activate" || name == "init")
                        && directive
                            .split_whitespace()
                            .skip(2)
                            .any(|word| word == "class");
                    current = Some(Unit {
                        name,
                        class: class.clone().filter(|_| keyword == "method"),
                        test,
                        activation,
                        code: String::new(),
                        text: String::new(),
                    });
                }
                _ => {}
            }
            continue;
        }
        if let Some(unit) = current.as_mut() {
            unit.code.push_str(&code);
            unit.code.push('\n');
            unit.text.push_str(&text);
            unit.text.push('\n');
        }
    }
    units.extend(current);
    units
}

fn words(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '!' | '?')))
        .filter(|word| !word.is_empty())
}

/// Whether a clause of `code` starts with `keyword`, directly or after
/// `THEN`, `ELSE`, `OTHERWISE` or a label.
fn instruction(code: &str, keyword: &str) -> bool {
    code.split(['\n', ';']).any(|clause| {
        let tokens: Vec<&str> = clause
            .split(|c: char| c.is_ascii_whitespace() || c == ':')
            .filter(|token| !token.is_empty())
            .collect();
        tokens.iter().enumerate().any(|(at, token)| {
            *token == keyword
                && (at == 0 || matches!(tokens[at - 1], "then" | "else" | "otherwise"))
                && !tokens.get(at + 1).is_some_and(|next| next.starts_with('='))
        })
    })
}

/// Whether `code` sends `name` (lower case) to anything.
fn sends(code: &str, name: &str) -> bool {
    code.match_indices('~').any(|(at, _)| {
        let rest = code[at + 1..].trim_start_matches('~');
        rest.strip_prefix(name).is_some_and(|after| {
            !after.starts_with(|c: char| {
                c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '!' | '?')
            })
        })
    })
}

/// The criterion's features `unit` uses itself.
fn features(unit: &Unit) -> BTreeSet<&'static str> {
    let code = unit.code.as_str();
    let mut found = BTreeSet::new();
    if instruction(code, "reply") {
        found.insert("REPLY");
    }
    if instruction(code, "guard") {
        found.insert("GUARD");
    }
    if sends(code, "start") || sends(code, "startwith") {
        found.insert("~start");
    }
    if sends(code, "reply") || sends(code, "replywith") {
        found.insert("Message~reply");
    }
    let word_set: BTreeSet<&str> = words(code).collect();
    if word_set.contains(".mutexsemaphore") || word_set.contains(".eventsemaphore") {
        found.insert("semaphore class");
    }
    if word_set
        .iter()
        .any(|word| word.starts_with("sys") && word.ends_with("sem"))
    {
        found.insert("Sys*Sem");
    }
    if word_set.contains(".alarm") {
        found.insert("Alarm");
    }
    if word_set.contains(".ticker") {
        found.insert("Ticker");
    }
    if word_set.contains("syssleep") {
        found.insert("SysSleep");
    }
    if code.contains(".context~thread") {
        found.insert(".context~thread");
    }
    const FIELDS: &[&str] = &["thread", "isguarded", "hasscopelock", "scopelockcount"];
    if FIELDS.iter().any(|field| {
        sends(code, field)
            || unit.text.contains(&format!("'{field}'"))
            || unit.text.contains(&format!("\"{field}\""))
    }) {
        found.insert("TraceObject field");
    }
    found
}

/// Whether `unit` uses a named queue or a registry, rxapi's persistent state,
/// and whether that is `RexxQueue`.
fn rxapi(unit: &Unit) -> Option<&'static str> {
    const CALLS: &[&str] = &["rxqueue", "rxfuncadd", "rxfuncdrop", "rxfuncquery"];
    let code = unit.code.as_str();
    if code.contains(".rexxqueue") {
        return Some("RexxQueue, Phase 10");
    }
    let word_set: BTreeSet<&str> = words(code).collect();
    if word_set.iter().any(|word| word.contains("rexxmacro"))
        || CALLS.iter().any(|call| word_set.contains(call))
    {
        return Some("rxapi persistent state");
    }
    None
}

/// Each test unit of `text` with the features it and the units it reaches
/// use, and the rxapi reason if one of those reaches rxapi.
fn derive_group(text: &str) -> Vec<(String, BTreeSet<&'static str>, Option<&'static str>)> {
    let units = units(text);
    let own: Vec<BTreeSet<&'static str>> = units.iter().map(features).collect();
    let refs: Vec<BTreeSet<&str>> = units
        .iter()
        .map(|unit| words(&unit.code).collect())
        .collect();
    // A class named is a class whose `INIT` a `~new` runs.
    let reaches = |from: usize, to: &Unit| {
        let names = &refs[from];
        names.contains(to.name.as_str())
            || to.name == "init"
                && to.class.as_ref().is_some_and(|class| {
                    names.contains(class.as_str()) || names.contains(format!(".{class}").as_str())
                })
    };
    let mut out = Vec::new();
    for (at, unit) in units.iter().enumerate() {
        if !unit.test {
            continue;
        }
        let mut seen: BTreeSet<usize> = units
            .iter()
            .enumerate()
            .filter(|(_, unit)| unit.activation)
            .map(|(at, _)| at)
            .chain([at])
            .collect();
        let mut frontier: Vec<usize> = seen.iter().copied().collect();
        while let Some(from) = frontier.pop() {
            for (to, target) in units.iter().enumerate() {
                if !seen.contains(&to) && !target.test && reaches(from, target) {
                    seen.insert(to);
                    frontier.push(to);
                }
            }
        }
        let found: BTreeSet<&'static str> = seen.iter().flat_map(|at| own[*at].clone()).collect();
        let reason = seen.iter().find_map(|at| rxapi(&units[*at]));
        out.push((unit.name.to_ascii_uppercase(), found, reason));
    }
    out
}

fn group_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .map(|entry| entry.expect("a directory entry").path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            group_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "testGroup") {
            out.push(path);
        }
    }
}

/// The derived list and the exclusions, each exclusion with its reason.
fn derive(root: &Path) -> (Vec<Derived>, Vec<(Derived, &'static str)>) {
    let mut files = Vec::new();
    group_files(root, &mut files);
    let mut list = Vec::new();
    let mut excluded = Vec::new();
    for file in files {
        let group = file
            .strip_prefix(root)
            .expect("under the root")
            .to_string_lossy()
            .into_owned();
        let bytes = fs::read(&file).unwrap_or_else(|e| panic!("cannot read {group}: {e}"));
        let text = String::from_utf8_lossy(&bytes);
        let dir_reason = EXCLUDED_DIRS
            .iter()
            .find(|(prefix, _)| group.starts_with(prefix))
            .map(|(_, reason)| *reason);
        for (test, features, rxapi) in derive_group(&text) {
            if features.is_empty() {
                continue;
            }
            let row = Derived {
                group: group.clone(),
                test,
                features,
            };
            match dir_reason.or(rxapi) {
                Some(reason) => excluded.push((row, reason)),
                None => list.push(row),
            }
        }
    }
    (list, excluded)
}

fn render(list: &[Derived]) -> String {
    list.iter()
        .map(|row| {
            let features: Vec<&str> = row.features.iter().copied().collect();
            format!("{} {} {}\n", row.group, row.test, features.join(","))
        })
        .collect()
}

fn render_excluded(excluded: &[(Derived, &str)]) -> String {
    excluded
        .iter()
        .map(|(row, reason)| format!("{} {} -- {reason}\n", row.group, row.test))
        .collect()
}

/// The fenced block following `heading` in the pinning document.
fn committed_block(heading: &str) -> String {
    let text = fs::read_to_string(pinning_doc()).expect("the pinning document");
    let after = text
        .split_once(&format!("\n{heading}\n"))
        .unwrap_or_else(|| panic!("no {heading:?} in the pinning document"))
        .1;
    let open = after.find("```").expect("a fenced block after the heading");
    let body = &after[open..];
    let body = &body[body.find('\n').expect("an opening fence line") + 1..];
    body[..body.find("```").expect("a closing fence")].to_string()
}

#[test]
fn the_derived_list_is_the_committed_one() {
    let (list, excluded) = derive(&ootest_root());
    let fresh = render(&list);
    let fresh_excluded = render_excluded(&excluded);
    if env::var_os(ROOT_ENV).is_some() {
        println!("derived:\n{fresh}excluded:\n{fresh_excluded}");
        return;
    }
    assert_eq!(
        committed_block("## Derived list"),
        fresh,
        "the derived list; the fresh one is printed above"
    );
    assert_eq!(
        committed_block("## Exclusions"),
        fresh_excluded,
        "the exclusions"
    );
}

#[test]
fn the_derived_list_holds_the_groups_the_spec_names() {
    const NAMED: &[&str] = &[
        "base/keyword/GUARD.testGroup",
        "base/keyword/REPLY.testGroup",
        "base/class/Message.testGroup",
        "base/class/EventSemaphore.testGroup",
        "base/class/MutexSemaphore.testGroup",
        "base/class/Alarm.testGroup",
        "base/class/Ticker.testGroup",
        "regressions/bug2003_guard_when.testGroup",
        "base/rexxutil/SysSleep.testGroup",
        "base/keyword/TRACE_TraceObject.testGroup",
        "base/class/RexxContext.testGroup",
        "base/keyword/TRACE.testGroup",
        "base/keyword/RAISE.testGroup",
        "base/class/Object.testGroup",
        "base/directives/METHOD.testGroup",
        "base/special.variables/RESULT_RC_SIGL.testGroup",
        "doc/rexxref/chapter5/Section1.testGroup",
    ];
    let (list, _) = derive(&worktree().join("ootest/ooRexx"));
    let groups: BTreeSet<&str> = list.iter().map(|row| row.group.as_str()).collect();
    let missing: Vec<&&str> = NAMED
        .iter()
        .filter(|name| !groups.contains(**name))
        .collect();
    assert!(
        missing.is_empty(),
        "named groups the derivation misses: {missing:?}"
    );
}

#[test]
fn the_detector_tells_an_instruction_from_a_name_that_only_looks_like_one() {
    let group = "::class t subclass ooTestCase\n\
                 ::method test_reply\n  if a then reply 1\n\
                 ::method test_helper\n  call helper\n\
                 ::method test_quoted\n  say 'reply guard on ~start'\n\
                 ::method test_assigned\n  reply = 1; guard = 2 -- reply\n\
                 ::method test_message\n  m = o~start('x')\n\
                 ::method test_class\n  x = .worker~new\n\
                 ::routine helper\n  call syssleep 1\n\
                 ::class worker\n::method init\n  guard on when a\n\
                 ::class timers\n::method activate class\n  a = .alarm~new(1, .nil)\n";
    let derived: BTreeMap<String, BTreeSet<&str>> = derive_group(group)
        .into_iter()
        .map(|(name, features, _)| (name, features))
        .collect();
    let expect = |name: &str, want: &[&str]| {
        assert_eq!(
            derived[name],
            want.iter().copied().collect::<BTreeSet<_>>(),
            "{name}"
        );
    };
    expect("TEST_REPLY", &["Alarm", "REPLY"]);
    expect("TEST_HELPER", &["Alarm", "SysSleep"]);
    expect("TEST_QUOTED", &["Alarm"]);
    expect("TEST_ASSIGNED", &["Alarm"]);
    expect("TEST_MESSAGE", &["Alarm", "~start"]);
    expect("TEST_CLASS", &["Alarm", "GUARD"]);
}

#[path = "support/group_runner.rs"]
mod group_runner;
#[path = "support/suite_rows/mod.rs"]
mod suite_rows;
mod support;
mod watchdog;

/// The group runner's copies keep every byte they do not rewrite: a group
/// file or `ooTest.frm` need not be UTF-8.
mod byte_exact_copies {
    use std::collections::BTreeSet;
    use std::fs;

    use super::group_runner::{mark_starts, skip, without_rxfuncquery, worktree};

    /// A scratch directory of this test's own under the target directory.
    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("byte-exact-copies");
        fs::create_dir_all(&dir).expect("the scratch directory");
        dir.join(name)
    }

    /// The lines of `bytes`, split on `\n` as the runner splits them.
    fn lines(bytes: &[u8]) -> Vec<&[u8]> {
        bytes.split(|&byte| byte == b'\n').collect()
    }

    /// `skip` over a copy of `C2X.testGroup`, which holds a literal `0xAA`,
    /// changes the renamed method's line and no other byte.
    #[test]
    fn skip_changes_only_the_renamed_methods_line() {
        let original = fs::read(worktree().join("ootest/ooRexx/base/bif/C2X.testGroup"))
            .expect("C2X.testGroup");
        assert!(
            std::str::from_utf8(&original).is_err(),
            "C2X.testGroup is expected to hold a byte that is not UTF-8"
        );
        let copy = scratch("C2X.testGroup");
        fs::write(&copy, &original).expect("the copy");
        let before = lines(&original);
        let renamed = before
            .iter()
            .position(|line| {
                line.trim_ascii_start()
                    .to_ascii_lowercase()
                    .starts_with(b"::method")
            })
            .expect("a test method");
        let name =
            String::from_utf8_lossy(before[renamed].trim_ascii_start()[8..].trim_ascii_start())
                .split_whitespace()
                .next()
                .expect("a method name")
                .trim_matches(['\'', '"'])
                .to_string();
        let member = format!("C2X.{}", name.to_ascii_uppercase());
        skip(&copy, "C2X", &BTreeSet::from([member]));
        let written = fs::read(&copy).expect("the rewritten copy");
        let after = lines(&written);
        assert_eq!(before.len(), after.len(), "the line count");
        for (at, (was, now)) in before.iter().zip(&after).enumerate() {
            if at == renamed {
                let marker = b"SKIPPED_";
                let inserted = now
                    .windows(marker.len())
                    .position(|window| window == marker)
                    .expect("the renamed line carries the marker");
                let restored = [&now[..inserted], &now[inserted + marker.len()..]].concat();
                assert_eq!(restored.as_slice(), *was, "the renamed line");
            } else {
                assert_eq!(was, now, "line {} changed", at + 1);
            }
        }
        fs::remove_file(&copy).expect("remove the copy");
    }

    /// `without_rxfuncquery` over a copy of `ooTest.frm` with a `0xFF` in a
    /// comment does not panic and keeps the byte.
    #[test]
    fn without_rxfuncquery_keeps_a_byte_that_is_not_utf_8() {
        let mut text = fs::read(worktree().join("ootest/ooTest.frm")).expect("ooTest.frm");
        let at = text
            .windows(2)
            .position(|pair| pair == b"--")
            .expect("a comment");
        text.insert(at + 2, 0xFF);
        let copy = scratch("ooTest.frm");
        fs::write(&copy, &text).expect("the copy");
        without_rxfuncquery(&copy);
        let written = fs::read(&copy).expect("the rewritten copy");
        assert!(written.contains(&0xFF), "the 0xFF byte was lost");
        assert!(
            !written.windows(12).any(|window| window == b"rxfuncquery("),
            "an rxfuncquery test is left"
        );
        fs::remove_file(&copy).expect("remove the copy");
    }

    /// `mark_starts` keeps a byte that is not UTF-8 elsewhere in the file.
    #[test]
    fn mark_starts_keeps_a_byte_that_is_not_utf_8() {
        let at = scratch("mark-starts");
        fs::create_dir_all(at.join("framework")).expect("the framework directory");
        let mut text =
            fs::read(worktree().join("ootest/framework/OOREXXUNIT.CLS")).expect("OOREXXUNIT.CLS");
        text.insert(0, 0xFF);
        fs::write(at.join("framework/OOREXXUNIT.CLS"), &text).expect("the copy");
        mark_starts(&at);
        let written = fs::read(at.join("framework/OOREXXUNIT.CLS")).expect("the rewritten copy");
        assert_eq!(written[0], 0xFF, "the leading 0xFF was lost");
        assert!(
            written
                .windows(20)
                .any(|window| window == b".error~say('started'"),
            "the start line is not marked"
        );
        fs::remove_dir_all(&at).expect("remove the copy");
    }
}

/// One test of criterion 1's derived list, run in process, for the
/// instruments that read an `Outcome` field a feature adds.
#[cfg(any(feature = "pinning", feature = "sharing"))]
mod derived_runs {
    use std::fs;
    use std::path::Path;

    use rexx_exec::{Invocation, Outcome, SwitchMode};

    use super::group_runner::fresh_copy;

    pub(super) fn run_test(
        run: &Path,
        group: &str,
        test: Option<&str>,
        mode: Option<SwitchMode>,
    ) -> Outcome {
        let dir = Path::new(group).parent().and_then(Path::to_str);
        fresh_copy(run, dir.expect("a group directory"));
        let driver = run.join("testOORexx.rex");
        let text = fs::read(&driver).expect("the copied driver");
        let group_file = run.join("ooRexx").join(group);
        let args = match test {
            Some(test) => format!("-f {} -U -V 2 -t {test}", group_file.display()),
            None => format!("-f {} -U -V 2", group_file.display()),
        };
        let lib = super::support::oracle::oracle_root().join("lib");
        let mut environment: Vec<(Vec<u8>, Vec<u8>)> = std::env::vars()
            .filter(|(name, _)| name != "LD_LIBRARY_PATH")
            .map(|(name, value)| (name.into_bytes(), value.into_bytes()))
            .collect();
        environment.push((
            b"LD_LIBRARY_PATH".to_vec(),
            lib.to_string_lossy().into_owned().into_bytes(),
        ));
        let invocation = Invocation::with_argument(args.into_bytes());
        let invocation = match mode {
            Some(mode) => invocation.with_switch_mode(mode),
            None => invocation,
        }
        .with_directory(run.to_path_buf())
        .with_environment(environment);
        let outcome = super::watchdog::run_bounded(&driver.to_string_lossy(), text, invocation);
        fs::remove_dir_all(run).unwrap_or_else(|e| panic!("cannot remove {}: {e}", run.display()));
        outcome
    }

    /// The run's outcome: the refusal, a deadline, or the summary's class.
    pub(super) fn outcome_of(outcome: &Outcome) -> String {
        let stderr = String::from_utf8_lossy(&outcome.stderr);
        if let Some(line) = stderr.lines().find(|line| line.starts_with("rexx-exec: ")) {
            return format!("refused at {}", &line["rexx-exec: ".len()..]);
        }
        if super::watchdog::did_not_finish(outcome) {
            return "did not finish".to_string();
        }
        let stdout = String::from_utf8_lossy(&outcome.stdout);
        let value = |label: &str| {
            stdout
                .lines()
                .find_map(|line| line.strip_prefix(label))
                .map(|rest| rest.trim().to_string())
        };
        let class = match (value("Errors:"), value("Failures:"), value("Tests ran:")) {
            (_, _, Some(ran)) if ran == "0" => "no test ran",
            (Some(errors), _, _) if errors != "0" => "error",
            (_, Some(failures), _) if failures != "0" => "failure",
            (Some(_), Some(_), _) => "pass",
            _ => "no summary",
        };
        format!("{class}, rc {}", outcome.exit_code)
    }
}

/// Criterion 6's sharing fraction (spec 2026-09-29 section 9).
#[cfg(feature = "sharing")]
mod sharing {
    use std::fs;
    use std::path::{Path, PathBuf};

    use rayon::prelude::*;
    use rexx_exec::{Invocation, SharingCount, SharingReport, run_program};

    use super::derive;
    use super::derived_runs::{outcome_of, run_test};
    use super::group_runner::{reaching_rxapi, worktree};

    fn sharing_of(source: &str) -> (String, SharingReport) {
        let outcome = run_program(
            "probe.rex",
            source.as_bytes().to_vec(),
            Invocation::none().with_deadline(super::support::oracle::RUN_DEADLINE),
        );
        (
            String::from_utf8_lossy(&outcome.stdout).into_owned(),
            outcome.sharing,
        )
    }

    /// One activity shares nothing, however much it allocates and reads.
    #[test]
    fn one_activity_shares_nothing() {
        let (stdout, sharing) =
            sharing_of("a = .array~new\ndo i = 1 to 50\n  a[i] = i * 2\nend\nsay a[50]\n");
        assert_eq!(stdout, "100\n");
        assert!(sharing.program.objects > 0, "{sharing:?}");
        assert_eq!(
            sharing.bootstrap.shared + sharing.program.shared,
            0,
            "{sharing:?}"
        );
    }

    /// An array the main activity made and a started one reads is shared.
    #[test]
    fn an_object_read_by_a_started_activity_is_shared() {
        let source = "a = .array~of('x', 'y')\nm = .t~new~start('look', a)\nsay m~result\n\
                      ::class t\n::method look\n  use arg a\n  return a[2]\n";
        let (stdout, sharing) = sharing_of(source);
        assert_eq!(stdout, "y\n");
        assert!(sharing.program.shared > 0, "{sharing:?}");
    }

    /// Objects with an `UNINIT` that a started activity never names are not
    /// shared: the heap's registry of them is not a touch.
    #[test]
    fn uninit_objects_another_activity_never_names_are_not_shared() {
        let source = "a = .array~new\ndo i = 1 to 100\n  a[i] = .u~new\nend\n\
                      say .t~new~start('other')~result\nsay a~items\n\
                      ::class u\n::method uninit\n\
                      ::class t\n::method other\n  return 'ok'\n";
        let (stdout, sharing) = sharing_of(source);
        assert_eq!(stdout, "ok\n100\n");
        assert!(sharing.program.shared < 100, "{sharing:?}");
    }

    /// Semaphores the main activity made are not shared when a started
    /// activity collects: the collector's prune of the semaphore table is not
    /// a touch.
    #[test]
    fn a_collection_in_another_activity_shares_nothing_it_prunes() {
        let source = "s = .array~new\ndo i = 1 to 100\n  m = .mutexSemaphore~new\n  \
                      m~acquire\n  m~release\n  s[i] = m\nend\n\
                      say .t~new~start('other')~result\nsay s~items\n\
                      ::class t\n::method other\n  call GC 'Force'\n  return 'ok'\n";
        let (stdout, sharing) = sharing_of(source);
        assert_eq!(stdout, "ok\n100\n");
        assert!(sharing.program.shared < 50, "{sharing:?}");
    }

    /// A stem cleared in a started activity does not share the exposer
    /// stems of returned `PROCEDURE EXPOSE` calls the exposer table still
    /// reaches.
    #[test]
    fn a_stem_cleared_in_another_activity_shares_no_dead_exposer() {
        let source = "h. = 0\ndo i = 1 to 100\n  call p\nend\n\
                      say .t~new~start('clear', h.)~result\nsay h.1\nexit\n\
                      p: procedure expose h.1\n  h.1 = h.1 + 1\n  return\n\
                      ::class t\n::method clear\n  use arg s.\n  s.~empty\n  return 'ok'\n";
        let (stdout, sharing) = sharing_of(source);
        assert_eq!(stdout, "ok\n0\n");
        assert!(sharing.program.shared < 50, "{sharing:?}");
    }

    fn add(total: &mut SharingReport, one: SharingReport) {
        for (sum, count) in [
            (&mut total.bootstrap, one.bootstrap),
            (&mut total.program, one.program),
        ] {
            sum.objects += count.objects;
            sum.shared += count.shared;
        }
    }

    fn cells(count: SharingCount) -> String {
        format!("{} | {}", count.objects, count.shared)
    }

    const HEADER: &str =
        "| bootstrap objects | bootstrap shared | program objects | program shared |";

    fn summary(label: &str, runs: usize, total: SharingReport) -> String {
        format!(
            "{label} {runs}\n\n| | objects | shared |\n|---|---|---|\n\
             | bootstrap | {} |\n| program | {} |\n",
            cells(total.bootstrap),
            cells(total.program)
        )
    }

    fn write(file: &str, text: &str) {
        let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join(file);
        fs::write(&out, text).expect("cannot write the table");
        println!("{text}\nwritten to {}", out.display());
    }

    #[test]
    fn sharing_fraction_over_the_derived_list() {
        let (list, _) = derive(&worktree().join("ootest/ooRexx"));
        let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("sharing-{}", std::process::id()));
        let rows: Vec<(String, String, SharingReport)> = list
            .par_iter()
            .enumerate()
            .map(|(at, row)| {
                let outcome = run_test(
                    &base.join(at.to_string()),
                    &row.group,
                    Some(&row.test),
                    None,
                );
                (
                    format!("{} {}", row.group, row.test),
                    outcome_of(&outcome),
                    outcome.sharing,
                )
            })
            .collect();
        let mut table = format!("| test | outcome {HEADER}\n|---|---|---|---|---|---|\n");
        let mut total = SharingReport::default();
        for (test, outcome, sharing) in &rows {
            add(&mut total, *sharing);
            table.push_str(&format!(
                "| {test} | {outcome} | {} | {} |\n",
                cells(sharing.bootstrap),
                cells(sharing.program)
            ));
        }
        write(
            "sharing-derived.md",
            &format!("{}\n{table}", summary("tests", rows.len(), total)),
        );
    }

    /// Every ooTest group file run in process until its end or its first
    /// refusal or deadline, but a group with a test that reaches rxapi
    /// (`group_runner::reaching_rxapi`).
    #[test]
    #[ignore = "most of 8 GB; run alone with RAYON_NUM_THREADS=4 and --ignored"]
    fn sharing_fraction_over_every_ootest_group() {
        let root = worktree().join("ootest/ooRexx");
        let mut groups = Vec::new();
        let mut skipped = Vec::new();
        let mut pending = vec![root.clone()];
        while let Some(dir) = pending.pop() {
            for entry in fs::read_dir(&dir).expect("an ooTest directory").flatten() {
                let path = entry.path();
                if path.is_dir() {
                    pending.push(path);
                    continue;
                }
                let Some(stem) = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .and_then(|name| name.strip_suffix(".testGroup"))
                else {
                    continue;
                };
                let relative = path.strip_prefix(&root).expect("below the root");
                let group = relative.to_str().expect("a UTF-8 path").to_string();
                let dir = relative
                    .parent()
                    .and_then(Path::to_str)
                    .expect("a directory");
                if reaching_rxapi(dir, &[stem]).is_empty() {
                    groups.push(group);
                } else {
                    skipped.push(group);
                }
            }
        }
        groups.sort();
        skipped.sort();
        let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("sharing-groups-{}", std::process::id()));
        let rows: Vec<(String, String, SharingReport)> = groups
            .par_iter()
            .enumerate()
            .map(|(at, group)| {
                let outcome = run_test(&base.join(at.to_string()), group, None, None);
                (group.clone(), outcome_of(&outcome), outcome.sharing)
            })
            .collect();
        let mut table = format!("| group | outcome {HEADER}\n|---|---|---|---|---|---|\n");
        let mut total = SharingReport::default();
        for (group, outcome, sharing) in &rows {
            add(&mut total, *sharing);
            table.push_str(&format!(
                "| {group} | {outcome} | {} | {} |\n",
                cells(sharing.bootstrap),
                cells(sharing.program)
            ));
        }
        let skipped: String = skipped.iter().map(|group| format!("- {group}\n")).collect();
        write(
            "sharing-groups.md",
            &format!(
                "{}\nnot run, reaching rxapi:\n\n{skipped}\n{table}",
                summary("groups", rows.len(), total)
            ),
        );
    }
}

#[cfg(feature = "pinning")]
mod measured {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};

    use rayon::prelude::*;
    use rexx_exec::{Invocation, Outcome, ParkKind, PinKind, PinReport, SwitchMode, run_program};

    use super::derive;
    use super::derived_runs::{outcome_of, run_test};
    use super::group_runner::worktree;

    /// The external routine every probe of [`report_of`] can call.
    const EXTF: &str =
        "use arg n\nif n >= 3 then do\n  call SysSleep 0\n  return 'ok'\nend\nreturn extf(n + 1)\n";

    /// Runs `source` in a directory holding the external routine `extf.rex`.
    /// The tests run in parallel, and test processes share the directory, so
    /// the routine is written under a name of the process's and thread's own
    /// and renamed into place: a probe never reads it half written.
    fn report_of(source: &str) -> PinReport {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("pinning-probes");
        fs::create_dir_all(&dir).expect("the probe directory");
        let extf = dir.join("extf.rex");
        if fs::read(&extf).ok().as_deref() != Some(EXTF.as_bytes()) {
            let staged = dir.join(format!(
                "extf.{}.{:?}.staged",
                std::process::id(),
                std::thread::current().id()
            ));
            fs::write(&staged, EXTF).expect("the external routine");
            fs::rename(&staged, &extf).expect("the external routine in place");
        }
        let outcome = run_program(
            "probe.rex",
            source.as_bytes().to_vec(),
            Invocation::none()
                .with_deadline(super::support::oracle::RUN_DEADLINE)
                .with_directory(dir),
        );
        assert_eq!(outcome.pinning.unbalanced, 0, "a pinned frame left pushed");
        outcome.pinning
    }

    fn frames_at(report: &PinReport, park: ParkKind) -> Vec<Vec<PinKind>> {
        report
            .parks
            .keys()
            .filter(|(kind, _)| *kind == park)
            .map(|(_, frames)| frames.clone())
            .collect()
    }

    /// A slice that ends inside a sort comparator is deferred, and counted
    /// once with the comparator's frame, however many pinned clauses follow
    /// before the sort returns.
    #[test]
    fn a_slice_deferred_inside_a_sort_comparator_is_counted_once() {
        let source = "b = .t~new~start('other')\narr = .array~of(3, 1, 2)\n\
                      arr~sortWith(.cmp~new)\nsay 'sorted'\n\
                      ::class t\n::method other\n  say 'other ran'\n\
                      ::class cmp\n::method compare\n  use arg l, r\n  return l - r\n";
        let outcome = run_program(
            "probe.rex",
            source.as_bytes().to_vec(),
            Invocation::none()
                .with_deadline(super::support::oracle::RUN_DEADLINE)
                .with_switch_mode(SwitchMode::AtClause(4)),
        );
        assert_eq!(
            String::from_utf8_lossy(&outcome.stdout),
            "other ran\nsorted\n"
        );
        let deferred = &outcome.pinning.deferred_slices;
        assert_eq!(deferred.values().sum::<u64>(), 1, "{deferred:?}");
        assert!(
            deferred
                .keys()
                .all(|frames| frames.contains(&PinKind::SortComparator)),
            "{deferred:?}"
        );
    }

    /// A `REPLY` in a labelled block is refused and counted with the
    /// `NestedLoop` frame; one whose pins all lie below its method body's
    /// driver moves and is not counted.
    #[test]
    fn an_immovable_reply_is_counted_with_its_frames() {
        let report = report_of(
            "say .c~new~m\n::class c\n::method m\n  do label l\n    reply 1\n    leave l\n  end\n",
        );
        let immovable = &report.immovable_replies;
        assert_eq!(immovable.values().sum::<u64>(), 1, "{immovable:?}");
        assert!(
            immovable
                .keys()
                .all(|frames| frames.contains(&PinKind::NestedLoop)),
            "{immovable:?}"
        );
        let report =
            report_of(".c~new~~m\nsay 'main'\n::class c\n::method m\n  reply\n  say 'rest'\n");
        assert!(
            report.immovable_replies.is_empty(),
            "{:?}",
            report.immovable_replies
        );
    }

    /// Under every opportunity main defers a slice at its sort comparator's
    /// first clause, whose `~result` then parks it pinned. The activity it
    /// waits on sorts with a one-clause comparator while a third is ready:
    /// its first slice is deferred and its second takes a pinned yield, so
    /// the slice main deferred is not carried over to it. The one-clause
    /// comparator gives it an odd count of pinned visits, which per-key
    /// totals need to see a carried slice: with an even count, the shift by
    /// one visit leaves both totals unchanged.
    #[test]
    fn a_slice_deferred_before_a_pinned_park_stays_with_its_activity() {
        let source = ".local['ARR'] = .array~of(2, 1)\n\
                      .local~~put(.array~of(3, 1, 2)~start('sortWith', .cmpb~new), 'M')\
                      ~~put(.t~new~start('run'), 'C')~at('ARR')~sortWith(.cmpa~new)\n\
                      say 'done'\n\
                      ::class cmpa\n::method compare\n  \
                      return .m~result~items * 0 + arg(1) - arg(2)\n\
                      ::class cmpb\n::method compare\n  return arg(1) - arg(2)\n\
                      ::class t\n::method run\n  do i = 1 to 3\n    say 'c ran' i\n  end\n";
        let outcome = run_program(
            "probe.rex",
            source.as_bytes().to_vec(),
            Invocation::none()
                .with_deadline(super::support::oracle::RUN_DEADLINE)
                .with_switch_mode(SwitchMode::EveryOpportunity),
        );
        assert_eq!(
            (
                outcome.exit_code,
                String::from_utf8_lossy(&outcome.stdout).as_ref(),
                String::from_utf8_lossy(&outcome.stderr).as_ref()
            ),
            (0, "c ran 1\ndone\nc ran 2\nc ran 3\n", "")
        );
        let sorting = vec![PinKind::Native("SORTWITH".into()), PinKind::SortComparator];
        let parking = [vec![PinKind::TreeSend], sorting.clone()].concat();
        let deferred = &outcome.pinning.deferred_slices;
        assert_eq!(deferred.get(&parking), Some(&1), "{deferred:?}");
        assert_eq!(deferred.get(&sorting), Some(&2), "{deferred:?}");
        let yields = &outcome.pinning.pinned_yields;
        assert_eq!(yields.get(&sorting), Some(&1), "{yields:?}");
    }

    /// A busy-wait inside `INTERPRET` takes pinned yields, counted with the
    /// `Interpret` frame, and no pinned yield is counted where nothing pins.
    #[test]
    fn a_pinned_busy_wait_counts_its_pinned_yields() {
        let source = |wait: &str| {
            format!(
                "f = .flag~new\na = f~start('waitForFlag')\nf~start('setFlag')\na~wait\n\
                 ::class flag\n::attribute done unguarded\n::method init\n  expose done\n  \
                 done = 0\n::method waitForFlag unguarded\n  {wait}\n  say 'ended'\n\
                 ::method setFlag unguarded\n  self~done = 1\n"
            )
        };
        let run = |wait: &str| {
            run_program(
                "probe.rex",
                source(wait).into_bytes(),
                Invocation::none()
                    .with_deadline(super::support::oracle::RUN_DEADLINE)
                    .with_switch_mode(SwitchMode::EveryOpportunity),
            )
        };
        let pinned = run("interpret \"do while \\self~done; end\"");
        assert_eq!(String::from_utf8_lossy(&pinned.stdout), "ended\n");
        let yields = &pinned.pinning.pinned_yields;
        assert!(yields.values().sum::<u64>() >= 1, "{yields:?}");
        assert!(
            yields
                .keys()
                .all(|frames| frames.contains(&PinKind::Interpret)),
            "{yields:?}"
        );
        let unpinned = run("do while \\self~done; end");
        assert_eq!(String::from_utf8_lossy(&unpinned.stdout), "ended\n");
        assert!(
            unpinned.pinning.pinned_yields.is_empty(),
            "{:?}",
            unpinned.pinning.pinned_yields
        );
    }

    /// Two pinned busy-waiters that need each other: `a` spins until `b`
    /// sets `x`, then sets `y`; `b`, run inside `a`'s pinned yields, sets `x`
    /// and spins until `y`. `b`'s yields can never run `a`, buried below, so
    /// each round of them is an inverted yield; the oracle completes (ruling
    /// P31). Bounded by a deadline, so the count is asserted, not the end.
    #[test]
    fn pinned_busy_waiters_that_need_each_other_count_inverted_yields() {
        let source = "g = .gate~new\na = .t~new~start('a', g)\nb = .t~new~start('b', g)\n\
                      say a~result\nsay b~result\n::class gate\n::attribute x unguarded\n\
                      ::attribute y unguarded\n::method init\n  expose x y\n  x = 0\n  y = 0\n\
                      ::class t\n::method a unguarded\n  use arg g\n  \
                      interpret 'do while \\g~x; end'\n  g~y = 1\n  return 'A ended'\n\
                      ::method b unguarded\n  use arg g\n  \
                      interpret 'g~x = 1; do while \\g~y; end'\n  return 'B ended'\n";
        let outcome = run_program(
            "probe.rex",
            source.as_bytes().to_vec(),
            Invocation::none()
                .with_switch_mode(SwitchMode::EveryOpportunity)
                .with_deadline(std::time::Duration::from_secs(2)),
        );
        assert_eq!(outcome.exit_code, rexx_exec::DEADLINE_EXIT);
        assert_eq!(String::from_utf8_lossy(&outcome.stdout), "");
        let inverted = &outcome.pinning.inverted_yields;
        assert!(inverted.values().sum::<u64>() > 0, "{inverted:?}");
        assert!(
            inverted
                .keys()
                .all(|frames| frames.contains(&PinKind::Interpret)),
            "{inverted:?}"
        );
    }

    #[test]
    fn a_park_records_the_pinned_frames_above_it() {
        let report = report_of("call SysSleep 0\n");
        assert_eq!(
            frames_at(&report, ParkKind::SysSleep),
            [Vec::<PinKind>::new()]
        );

        let report = report_of("interpret 'call SysSleep 0'\n");
        let frames = frames_at(&report, ParkKind::SysSleep);
        assert!(
            frames.len() == 1 && frames[0].contains(&PinKind::Interpret),
            "{frames:?}"
        );

        let report = report_of(
            "a = .array~of(2, 1)\na~sortWith(.c~new)\n\
             ::class c\n::method compare\n  use arg l, r\n  call SysSleep 0\n  return l - r\n",
        );
        let frames = frames_at(&report, ParkKind::SysSleep);
        assert!(
            !frames.is_empty()
                && frames.iter().all(|frames| {
                    frames.contains(&PinKind::SortComparator)
                        && frames.contains(&PinKind::Native("SORTWITH".into()))
                }),
            "{frames:?}"
        );

        let report = report_of(
            ".c~new~m\n::class c\n::method m\n  expose a\n  a = 1\n  guard on when a = 1\n",
        );
        assert_eq!(frames_at(&report, ParkKind::GuardWhen).len(), 1);
    }

    /// `SysSleep` inside `IF ... THEN DO`, a loop and a function argument
    /// parks its activity at the driver: no pinned frame above it and no
    /// pinned wait; inside a sort comparator it is a pinned wait under the
    /// comparator's frame.
    #[test]
    fn a_sleep_is_a_pinned_wait_only_under_a_pinned_frame() {
        let mut pinned = Vec::new();
        for program in [
            "if 1 then do\n  call SysSleep 0\n  say 'a'\nend\n",
            "do i = 1 to 2\n  call SysSleep 0\nend\n",
            "say length(SysSleep(0))\n",
            "x = SysSleep(0)\n",
        ] {
            let report = report_of(program);
            if frames_at(&report, ParkKind::SysSleep) != [Vec::<PinKind>::new()]
                || !report.pinned_parks.is_empty()
            {
                pinned.push(format!("{program:?}: {report:?}"));
            }
        }
        assert!(pinned.is_empty(), "{pinned:#?}");

        let report = report_of(
            "a = .array~of(2, 1)\na~sortWith(.c~new)\n\
             ::class c\n::method compare\n  use arg l, r\n  call SysSleep 0\n  return l - r\n",
        );
        let arrivals: u64 = report
            .parks
            .iter()
            .filter(|((kind, _), _)| *kind == ParkKind::SysSleep)
            .map(|(_, count)| count)
            .sum();
        let parks = &report.pinned_parks;
        assert!(
            arrivals > 0
                && parks.values().sum::<u64>() == arrivals
                && parks.keys().all(|(kind, frames)| {
                    *kind == ParkKind::SysSleep && frames.contains(&PinKind::SortComparator)
                }),
            "{report:?}"
        );
    }

    /// A `GUARD WHEN` wait at a level the driver entered stacklessly parks
    /// its activity with no pinned wait; inside a sort comparator it is a
    /// pinned wait under the comparator's frame.
    #[test]
    fn a_guard_when_is_a_pinned_wait_only_under_a_pinned_frame() {
        const SETTER: &str = "o = .k~new\nm = o~start('waiter')\ncall SysSleep 0.05\no~set\nm~wait\n\
                              ::class k\n::method set unguarded\n  expose v\n  v = 1\n";
        let report = report_of(&format!(
            "{SETTER}::method waiter unguarded\n  expose v\n  guard off when v = 1\n"
        ));
        assert!(
            !frames_at(&report, ParkKind::GuardWhen).is_empty() && report.pinned_parks.is_empty(),
            "{report:?}"
        );
        let report = report_of(&format!(
            "{SETTER}::method waiter unguarded\n  a = .array~of(2, 1)\n  a~sortWith(self)\n\
             ::method compare unguarded\n  expose v\n  use arg l, r\n  guard off when v = 1\n  \
             return l - r\n"
        ));
        let parks = &report.pinned_parks;
        assert!(
            parks.keys().any(|(kind, frames)| {
                *kind == ParkKind::GuardWhen && frames.contains(&PinKind::SortComparator)
            }) && parks.keys().all(|(kind, _)| *kind == ParkKind::GuardWhen),
            "{report:?}"
        );
    }

    /// A sleep in a PARSE template position or a SELECT CASE `WHEN` value is
    /// evaluated on the Rust stack, so it is a pinned wait under `TreeEval`.
    #[test]
    fn a_sleep_in_a_tree_evaluated_expression_is_a_pinned_wait() {
        for program in [
            "parse value 'ab' with x (SysSleep(0)) y\n",
            "select case 1\n  when SysSleep(0) + 1 then nop\nend\n",
        ] {
            let report = report_of(program);
            let parks: Vec<_> = report.pinned_parks.iter().collect();
            assert!(
                parks.len() == 1
                    && parks[0].0.0 == ParkKind::SysSleep
                    && parks[0].0.1.contains(&PinKind::TreeEval),
                "{program:?}: {report:?}"
            );
        }
    }

    /// A pinned sleeper due while a later sleeper's loop runs above its own
    /// is set aside until that loop returns, and counted as a late wake.
    #[test]
    fn a_buried_sleeper_due_first_is_a_late_wake() {
        let report = report_of(
            "b = .t~new~start('srt', 1.0, 0.05)\na = .t~new~start('srt', 0.2, 0)\n\
             a~wait\nb~wait\n::class t\n::method srt\n  use arg secs, pre\n  \
             call SysSleep pre\n  x = .array~of(2, 1)\n  x~sortWith(.c~new(secs))\n\
             ::class c\n::method init\n  expose s\n  use arg s\n\
             ::method compare\n  expose s\n  use arg l, r\n  call SysSleep s\n  return l - r\n",
        );
        let late = &report.late_wakes;
        assert!(
            !late.is_empty()
                && late.keys().all(|(kind, frames)| {
                    *kind == ParkKind::SysSleep && frames.contains(&PinKind::SortComparator)
                }),
            "{report:?}"
        );
        let report = report_of(
            "a = .array~of(2, 1)\na~sortWith(.c~new)\n\
             ::class c\n::method compare\n  use arg l, r\n  call SysSleep 0\n  return l - r\n",
        );
        assert!(report.late_wakes.is_empty(), "{report:?}");
    }

    /// One program per frame kind reachable from Rexx, each reaching `SysSleep`
    /// under that kind.
    const FRAME_PROBES: &[(&str, &str)] = &[
        (
            "Unknown",
            ".c~new~foo\n::class c\n::method unknown\n  call SysSleep 0\n",
        ),
        (
            "Forward",
            ".c~new~m\n::class c\n::method m\n  forward message('N')\n::method n\n  call SysSleep 0\n",
        ),
        (
            "Delegate",
            ".k~new~m\n::class k\n::method init\n  expose d\n  d = .c~new\n::method m delegate d\n\
             ::class c\n::method m\n  call SysSleep 0\n",
        ),
        (
            "Operator",
            "y = .c~new + 1\n::class c\n::method '+'\n  call SysSleep 0\n  return 1\n",
        ),
        (
            "TrapHandler",
            "call on error name h\naddress system 'false'\nexit\nh:\n  call SysSleep 0\n  return\n",
        ),
        (
            "NovalueHook",
            ".local~novalue = .c~new\nsay zz\n::class c\n::method novalue\n  call SysSleep 0\n  return 1\n",
        ),
        (
            "LoopHeader",
            "do i = 1 to 2 while f()\nend\nexit\nf:\n  call SysSleep 0\n  return 1\n",
        ),
        ("NestedLoop", "do label l\n  call SysSleep 0\nend\n"),
        (
            "Conversion",
            "do i over .c~new\nend\n::class c\n::method makearray\n  call SysSleep 0\n  return .array~new\n",
        ),
        (
            "TreeEval",
            "x = .c~new~~m\n::class c\n::method m\n  call SysSleep 0\n",
        ),
        (
            "TreeEval",
            "x = f(.c~new~~m)\nexit\nf: return 1\n::class c\n::method m\n  call SysSleep 0\n",
        ),
        (
            "TreeEval",
            "call f .c~new~~m\nexit\nf: return\n::class c\n::method m\n  call SysSleep 0\n",
        ),
        (
            "TreeEval",
            "x = length(.c~new~~m)\n::class c\n::method m\n  call SysSleep 0\n",
        ),
        (
            "TreeSend",
            ".c~new~~m\n::class c\n::method m\n  call SysSleep 0\n",
        ),
        (
            "Program",
            "say f(0)\n::routine f\n  use arg n\n  return extf(n + 1)\n",
        ),
        ("OpExec", "interpret 'call SysSleep 0'\n"),
        (
            "Uninit",
            "u = .c~new\n::class c\n::method uninit\n  call SysSleep 0\n",
        ),
        ("Interpret", "interpret 'call SysSleep 0'\n"),
        (
            "OutputWrapper",
            ".output~destination(.c~new)\nsay 'x'\n::class c\n::method say\n  call SysSleep 0\n",
        ),
        (
            "PullWrapper",
            ".input~destination(.c~new)\nparse pull line\n::class c\n::method linein\n  call SysSleep 0\n  return 'x'\n",
        ),
        (
            "TraceWrapper",
            ".traceOutput~destination(.c~new)\ntrace r\nx = 1\n::class c\n::method lineout\n  call SysSleep 0\n",
        ),
        (
            "SecurityManager",
            "r = .routine~new('r', \"x = stream('/nonexistent/dir/x', 'S')\")\n\
             r~setSecurityManager(.sm~new)\nr~call\n\
             ::class sm\n::method unknown\n  call SysSleep 0\n  return 0\n",
        ),
        (
            "Notification",
            "m = .message~new('abc', 'length')\nm~notify(.n~new)\nm~send\n\
             ::class n inherit MessageNotification\n::method messageComplete\n  \
             call SysSleep 0\n",
        ),
        (
            "RedirectWrapper",
            "a = .d~new\naddress system 'echo hi' with output using (a)\n\
             ::class d subclass array\n::method append\n  call SysSleep 0\n  forward class (super)\n",
        ),
    ];

    #[test]
    fn a_park_under_each_frame_kind_records_it() {
        let mut missing = Vec::new();
        for (kind, program) in FRAME_PROBES {
            let report = report_of(program);
            let seen = frames_at(&report, ParkKind::SysSleep)
                .iter()
                .flatten()
                .any(|frame| format!("{frame:?}") == *kind);
            if !seen {
                missing.push(format!("{kind}: {:?}", report.parks));
            }
        }
        assert!(missing.is_empty(), "{missing:#?}");
    }

    /// Calls and sends in argument positions, sends, the natives that run a
    /// Rexx body, and calls inside an unlabelled plain `DO` block run on the driver's
    /// frame, so a park inside them has no pinned frame above it.
    #[test]
    fn a_park_inside_a_stackless_entry_is_under_no_pinned_frame() {
        const CLASS: &str = "::class c\n::method m\n  call SysSleep 0\n  return 1\n\
                             ::method init\n  if arg() > 0 then call SysSleep 0\n";
        let mut pinned = Vec::new();
        for program in [
            "x = f(g())\nexit\nf: return 1\ng:\n  call SysSleep 0\n  return 2\n",
            "call f g()\nexit\nf: return\ng:\n  call SysSleep 0\n  return 2\n",
            "x = abs(g())\nexit\ng:\n  call SysSleep 0\n  return 2\n",
            "x = (g(), 1)\nexit\ng:\n  call SysSleep 0\n  return 2\n",
            "if .c~new~m then nop\n",
            ".c~new~m\n",
            "x = f(.c~new~m)\nexit\nf: return 1\n",
            "x = .c~new(1)\n",
            "x = .message~new(.c~new, 'M')~send\n",
            "x = .c~new~send('M')\n",
            "x = .c~new~start('M')\n",
            "x = .context~package~findRoutine('R')~call\n::routine r\n  call SysSleep 0\n",
            "if 1 then do\n  x = g()\n  y = .c~new~m\n  call g\nend\nexit\ng:\n  call SysSleep 0\n  return 2\n",
            "do i = 1 to 2\n  do\n    x = g()\n  end\nend\nexit\ng:\n  call SysSleep 0\n  return 2\n",
        ] {
            let source = format!("{program}{CLASS}");
            let report = report_of(&source);
            let frames = frames_at(&report, ParkKind::SysSleep);
            if frames != [Vec::<PinKind>::new()] {
                pinned.push(format!("{program:?}: {frames:?}"));
            }
        }
        assert!(pinned.is_empty(), "{pinned:#?}");
    }

    /// A `DELEGATE` method and the `OF` factories still run the Rexx body on
    /// the Rust stack, so a park inside it is pinned.
    #[test]
    fn a_park_inside_a_recursing_native_is_under_a_pinned_frame() {
        const CLASS: &str = "::class c\n::method m\n  call SysSleep 0\n  return 1\n";
        const INIT: &str = "::method init\n  call SysSleep 0\n";
        let mut unpinned = Vec::new();
        for program in [
            "x = .kd~new~m\n::class kd\n::method init\n  expose d\n  d = .c~new\n\
             ::method m delegate d\n"
                .to_string(),
            format!("x = .aa~of(1)\n::class aa subclass array\n{INIT}"),
            format!("x = .bb~of(1)\n::class bb subclass bag\n{INIT}"),
            format!("x = .ss~of(1)\n::class ss subclass set\n{INIT}"),
            format!("x = .qq~of(1)\n::class qq subclass queue\n{INIT}"),
        ] {
            let source = format!("{program}{CLASS}");
            let report = report_of(&source);
            let frames = frames_at(&report, ParkKind::SysSleep);
            if frames.is_empty() || frames.iter().any(Vec::is_empty) {
                unpinned.push(format!("{program:?}: {frames:?}"));
            }
        }
        assert!(unpinned.is_empty(), "{unpinned:#?}");
    }

    /// `~new` into a subclass's `INIT`, for every class a subclass reaches
    /// `INIT` through: either a park inside `INIT` is under no pinned frame
    /// and `INIT` reaching `~new` again adds no native stack per level, or
    /// the park is pinned. The classes whose own `NEW` sends `INIT` are the
    /// first kind.
    #[test]
    fn a_new_into_init_is_stackless_or_pinned() {
        let sends_init = [
            ("Object", ""),
            ("EventSemaphore", ""),
            ("MutexSemaphore", ""),
            ("List", ""),
            ("Queue", ""),
            ("Supplier", ""),
            ("Array", ""),
            ("Directory", ""),
            ("StringTable", ""),
            ("Table", ""),
            ("IdentityTable", ""),
            ("Set", ""),
            ("Bag", ""),
            ("Relation", ""),
            ("Class", "'Z'"),
            ("MutableBuffer", ""),
            ("WeakReference", "1"),
        ];
        let stack = |class: &str, args: &str, depth: usize| {
            let program = format!(
                ".local~n = {depth}\nx = .k~new({args})\nsay .local~n\n\
                 ::class k subclass {class}\n::method init\n  .local~n = .local~n - 1\n  \
                 if .local~n > 0 then y = .k~new({args})\n"
            );
            let outcome: Outcome = run_program(
                "probe.rex",
                program.into_bytes(),
                Invocation::none().with_deadline(super::support::oracle::RUN_DEADLINE),
            );
            assert_eq!(outcome.exit_code, 0, "{class}: {:?}", outcome.stderr);
            assert_eq!(String::from_utf8_lossy(&outcome.stdout), "0\n", "{class}");
            outcome.stack.bytes
        };
        let mut wrong = Vec::new();
        for (class, args) in sends_init.iter().copied().chain([("CircularQueue", "5")]) {
            let report = report_of(&format!(
                "x = .k~new({args})\n::class k subclass {class}\n::method init\n  \
                 call SysSleep 0\n"
            ));
            let frames = frames_at(&report, ParkKind::SysSleep);
            let (shallow, deep) = (stack(class, args, 50), stack(class, args, 500));
            let flat = deep <= shallow + 1024;
            let unpinned = frames == [Vec::<PinKind>::new()];
            let pinned = !frames.is_empty() && frames.iter().all(|frames| !frames.is_empty());
            let stackless = sends_init.contains(&(class, args));
            if !(unpinned && flat || pinned) || stackless && !(unpinned && flat) {
                wrong.push(format!(
                    "{class}: {frames:?}, {shallow} for 50, {deep} for 500"
                ));
            }
        }
        assert!(wrong.is_empty(), "{wrong:#?}");
    }

    #[test]
    fn a_park_point_native_is_not_a_frame_above_itself() {
        let report = report_of("m = .message~new('abc', 'LENGTH')\nm~send\nm~result\n");
        let frames = frames_at(&report, ParkKind::MessageResult);
        assert!(
            frames.len() == 1
                && !frames[0]
                    .iter()
                    .any(|frame| matches!(frame, PinKind::Native(_))),
            "{frames:?}"
        );
    }

    /// Main waits pinned on `m0`; `s1` waits pinned on `m1`; `s3` wakes main,
    /// which `s1`'s loop sets aside; `s2` then waits pinned on `m2` with
    /// `$WAIT`, which only main would send.
    const HIDDEN_INVERSION: &str = "c = .w~new\n\
        m0 = .message~new(c, 'val', 'I', 0)\nm1 = .message~new(c, 'val', 'I', 1)\n\
        m2 = .message~new(c, 'val', 'I', 2)\nc~start('s1', m0, m1, m2)\n\
        interpret \"say 'main got' m0~result\"\nm2~send\n\
        ::class w\n::method val unguarded; use arg x; return x*10\n\
        ::method s1 unguarded\n  use arg m0, m1, m2\n  .w~new~start('s3', m0, m1, m2)\n\
        \x20 interpret 'say m1~result'\n\
        ::method s3 unguarded\n  use arg m0, m1, m2\n  m0~send\n  .w~new~start('s2', m1, m2)\n\
        ::method s2 unguarded\n  use arg m1, m2\n  interpret '$WAIT'\n  m1~send\n";

    /// An inverted pinned wait is counted under its own park kind and the
    /// frames that pinned it, whichever loop set the ready activity aside; a
    /// pinned wait with nothing ready anywhere is not counted.
    #[test]
    fn an_inverted_wait_is_counted_with_its_kind_and_frames() {
        for (wait, kind) in [
            ("say m2~result", ParkKind::MessageResult),
            ("m2~wait", ParkKind::MessageWait),
        ] {
            let report = report_of(&HIDDEN_INVERSION.replace("$WAIT", wait));
            let inverted: Vec<_> = report.inverted.iter().collect();
            assert!(
                inverted.len() == 1
                    && inverted[0].0.0 == kind
                    && inverted[0].0.1.contains(&PinKind::Interpret)
                    && *inverted[0].1 == 1,
                "{wait}: {report:?}"
            );
        }
        let report = report_of(
            "c = .w~new\nm1 = .message~new(c, 'val', 'I', 1)\nm2 = .message~new(c, 'val', 'I', 2)\n\
             c~start('s1', m1, m2)\ninterpret 'say m1~result'\n\
             ::class w\n::method val unguarded; use arg x; return x*10\n\
             ::method s1 unguarded\n  use arg m1, m2\n  .w~new~start('s2', m1)\n\
             \x20 interpret 'say m2~result'\n\
             ::method s2 unguarded\n  use arg m1\n  m1~send\n",
        );
        assert_eq!(
            report.inverted.get(&(
                ParkKind::MessageResult,
                vec![PinKind::OpExec, PinKind::Interpret]
            )),
            Some(&1),
            "{report:?}"
        );
        let report = report_of("m = .message~new('abc', 'length')\ninterpret 'say m~result'\n");
        assert!(report.inverted.is_empty(), "{report:?}");
    }

    #[test]
    fn a_wait_is_a_park_point() {
        let report = report_of(".message~new(1, 'X')~wait\n");
        assert_eq!(
            frames_at(&report, ParkKind::MessageWait).len(),
            1,
            "{report:?}"
        );
        let report = report_of(".eventSemaphore~new~wait\n");
        assert_eq!(
            frames_at(&report, ParkKind::SemaphoreWait).len(),
            1,
            "{report:?}"
        );
        let report = report_of(
            "m = .mutexSemaphore~new\nm~acquire\n.message~new(m, 'acquire')~start\n\
             call SysSleep 0.05\nm~release\n",
        );
        assert_eq!(
            frames_at(&report, ParkKind::SemaphoreWait).len(),
            1,
            "{report:?}"
        );
        let report = report_of("h = SysCreateEventSem()\ncall SysWaitEventSem h\n");
        assert_eq!(
            frames_at(&report, ParkKind::SysSemWait).len(),
            1,
            "{report:?}"
        );
    }

    fn frames_text(frames: &[PinKind]) -> String {
        if frames.is_empty() {
            return "-".to_string();
        }
        frames
            .iter()
            .map(|frame| match frame {
                PinKind::Native(name) => format!("Native ~{name}"),
                other => format!("{other:?}"),
            })
            .collect::<Vec<_>>()
            .join(" > ")
    }

    /// Whether `frames` holds a kind other than the tree-evaluated and
    /// `Op::Exec` frames.
    fn pinned_beyond_tree(frames: &[PinKind]) -> bool {
        frames.iter().any(|frame| {
            !matches!(
                frame,
                PinKind::TreeEval | PinKind::TreeSend | PinKind::OpExec
            )
        })
    }

    #[test]
    fn pinned_parks_over_the_derived_list() {
        pinning_table(None, "pinning-table.md");
    }

    /// The same table with a switch at every clause boundary.
    #[test]
    fn pinned_parks_over_the_derived_list_under_every_opportunity() {
        pinning_table(
            Some(SwitchMode::EveryOpportunity),
            "pinning-table-every-opportunity.md",
        );
    }

    fn pinning_table(mode: Option<SwitchMode>, file: &str) {
        let (list, _) = derive(&worktree().join("ootest/ooRexx"));
        let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "pinning-{}-{}",
            std::process::id(),
            file.trim_end_matches(".md")
        ));
        let rows: Vec<(String, String, PinReport)> = list
            .par_iter()
            .enumerate()
            .map(|(at, row)| {
                let outcome = run_test(
                    &base.join(at.to_string()),
                    &row.group,
                    Some(&row.test),
                    mode.clone(),
                );
                (
                    format!("{} {}", row.group, row.test),
                    outcome_of(&outcome),
                    outcome.pinning,
                )
            })
            .collect();

        let mut per_test =
            String::from("| test | outcome | park | frames | arrivals |\n|---|---|---|---|---|\n");
        let mut totals: BTreeMap<(ParkKind, String), (usize, u64)> = BTreeMap::new();
        let mut unbalanced = Vec::new();
        for (test, outcome, report) in &rows {
            if report.unbalanced != 0 {
                unbalanced.push(test.clone());
            }
            if report.parks.is_empty() {
                per_test.push_str(&format!(
                    "| {test} | {outcome} | no park reached in-process | | |\n"
                ));
            }
            for ((park, frames), count) in &report.parks {
                let frames = frames_text(frames);
                per_test.push_str(&format!(
                    "| {test} | {outcome} | {park:?} | {frames} | {count} |\n"
                ));
                let total = totals.entry((*park, frames)).or_default();
                total.0 += 1;
                total.1 += count;
            }
        }
        let mut summary = String::from("| park | frames | tests | arrivals |\n|---|---|---|---|\n");
        for ((park, frames), (tests, arrivals)) in &totals {
            summary.push_str(&format!("| {park:?} | {frames} | {tests} | {arrivals} |\n"));
        }
        let beyond: u64 = rows
            .iter()
            .flat_map(|(_, _, report)| &report.parks)
            .filter(|((_, frames), _)| pinned_beyond_tree(frames))
            .map(|(_, count)| count)
            .sum();
        let all: u64 = rows
            .iter()
            .flat_map(|(_, _, report)| report.parks.values())
            .sum();
        let mut waits: BTreeMap<(&str, String, String), u64> = BTreeMap::new();
        for (_, _, report) in &rows {
            for (what, map) in [
                ("pinned", &report.pinned_parks),
                ("inverted", &report.inverted),
                ("late wake", &report.late_wakes),
            ] {
                for ((park, frames), count) in map {
                    *waits
                        .entry((what, format!("{park:?}"), frames_text(frames)))
                        .or_default() += count;
                }
            }
            for (what, map) in [
                ("immovable", &report.immovable_replies),
                ("deferred slice", &report.deferred_slices),
                ("pinned yield", &report.pinned_yields),
                ("inverted yield", &report.inverted_yields),
            ] {
                for (frames, count) in map {
                    let park = if what == "immovable" { "Reply" } else { "-" };
                    *waits
                        .entry((what, park.to_string(), frames_text(frames)))
                        .or_default() += count;
                }
            }
        }
        let mut by_kind = String::from("| wait | park | frames | count |\n|---|---|---|---|\n");
        for ((what, park, frames), count) in &waits {
            by_kind.push_str(&format!("| {what} | {park} | {frames} | {count} |\n"));
        }
        let text = format!(
            "{summary}\narrivals {all}, with a frame other than TreeEval, TreeSend or OpExec {beyond}\n\n{by_kind}\n{per_test}"
        );
        let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(file);
        fs::write(&out, &text).expect("cannot write the table");
        println!("{text}\nwritten to {}", out.display());
        assert!(
            unbalanced.is_empty(),
            "runs ending with a frame pushed: {unbalanced:?}"
        );
    }
}

/// The both-sides group runner: its self-test, and the outcome tables of
/// `base/class/Message`'s tests and of `base/class/Object`'s start tests.
/// Gate-only.
mod group_runs;
