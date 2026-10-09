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
mod support;
mod watchdog;

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
                    mode,
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
mod group_runs {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::group_runner::{
        GATE_ENV, Outcome, Run, SwitchMode, TestResult, VERBOSITY, excerpt, first_difference,
        fresh_copy, gate_mode, group_file, masked, on_a_quiet_machine, reaching_rxapi,
        rows_in_parallel, run_crate, run_tests, source_test_names, test_names,
    };
    use super::support::oracle;

    /// Where the Message table is written when set, for the SDD record.
    const TABLE_ENV: &str = "REXX_GROUP_TABLE";

    /// Where the Object table is written when set.
    const OBJECT_TABLE_ENV: &str = "REXX_OBJECT_TABLE";

    /// Where the Message start tests' table under `EveryOpportunity` is
    /// written when set.
    const SWITCHED_TABLE_ENV: &str = "REXX_SWITCHED_TABLE";

    /// The Message start tests that do not pass, each with the method its
    /// refusal names.
    const MESSAGE_START_REFUSED: &[(&str, &str)] = &[("TEST_STARTWITH_NOT_ARRAY", "MAKEARRAY")];

    /// The Message start tests whose outcome differs from the oracle's.
    const MESSAGE_START_DIFFERING: &[&str] = &[];

    /// The same under `EveryOpportunity`.
    const MESSAGE_START_DIFFERING_SWITCHED: &[&str] = &[];

    fn scratch(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("{name}-{}", std::process::id()))
            .join("run")
    }

    fn listed(oracle: &oracle::Oracle, run: &Path, dir: &str, group: &str) -> Vec<String> {
        let not_run: BTreeSet<String> = reaching_rxapi(dir, &[group]);
        test_names(oracle, run, dir, group, &not_run)
    }

    #[test]
    fn the_runner_passes_a_passing_group_and_reports_an_altered_test_as_differing() {
        if !gate_mode() {
            eprintln!("group_runs: skipped without {GATE_ENV}");
            return;
        }
        let oracle = oracle::locate();
        let run = scratch("group-runner-self-test");
        let (dir, group) = ("API/oo", "CONVERSION");
        let tests = listed(&oracle, &run, dir, group);
        assert_eq!(
            source_test_names(dir, group),
            tests,
            "the source's test list against the oracle's listing"
        );
        let results = run_tests(&oracle, &run, dir, group, &tests, SwitchMode::None, None);
        let not_passing: Vec<String> = results
            .iter()
            .filter(|row| !matches!(row.outcome, Outcome::Pass))
            .map(|row| format!("{} {}", row.test, row.outcome.label()))
            .collect();
        assert!(not_passing.is_empty(), "not passing: {not_passing:?}");

        let altered = |run: &Path| {
            let file = group_file(run, dir, group);
            let text = fs::read_to_string(&file).expect("the copied group file");
            let from = "self~assertSame(1, .CONVERSIONTester~TestObjectToValue(1.0,";
            assert!(text.contains(from), "the altered assertion is in the group");
            fs::write(&file, text.replacen(from, &from.replace("(1,", "(2,"), 1))
                .expect("cannot rewrite the copied group file");
        };
        let name = tests
            .iter()
            .find(|name| name.eq_ignore_ascii_case("testint01"))
            .expect("testint01 is listed")
            .clone();
        let results = run_tests(
            &oracle,
            &run,
            dir,
            group,
            std::slice::from_ref(&name),
            SwitchMode::None,
            Some(&altered),
        );
        let Outcome::Differ {
            oracle: theirs,
            ours,
        } = &results[0].outcome
        else {
            panic!("the altered test is {}", results[0].outcome.label());
        };
        eprintln!(
            "altered {name}: {}",
            first_difference(&masked(&theirs.stdout), &masked(&ours.stdout))
        );
        fs::remove_dir_all(run.parent().expect("a parent")).expect("cannot remove the run");
    }

    /// Runs every test of `group` on both sides and answers the outcomes,
    /// with the table written to the file `table_env` names where set.
    fn outcome_table(
        name: &str,
        dir: &str,
        group: &str,
        table_env: &str,
        mode: SwitchMode,
        chosen: impl Fn(&str) -> bool,
    ) -> Vec<TestResult> {
        let oracle = oracle::locate();
        let tests: Vec<String> = source_test_names(dir, group)
            .into_iter()
            .filter(|test| chosen(test))
            .collect();
        let results = rows_in_parallel(
            name,
            &tests,
            |test| WALL_CLOCK.contains(&format!("{dir}/{group}.testGroup {test}").as_str()),
            |run, test| {
                let one = std::slice::from_ref(test);
                let mut results = run_tests(&oracle, run, dir, group, one, mode.clone(), None);
                results.pop().expect("one result")
            },
        );
        let mut table = String::new();
        for row in &results {
            let detail = match &row.outcome {
                Outcome::Refused { message, .. } => message.clone(),
                Outcome::Differ { oracle, ours } => format!(
                    "{}; ours stderr {:?}",
                    first_difference(&masked(&oracle.stdout), &masked(&ours.stdout)),
                    excerpt(&ours.stderr)
                ),
                _ => String::new(),
            };
            table.push_str(&format!(
                "{}\t{}\t{detail}\n",
                row.test,
                row.outcome.label()
            ));
        }
        eprintln!("{table}");
        if let Some(path) = std::env::var_os(table_env) {
            fs::write(path, &table).expect("cannot write the table");
        }
        assert_eq!(results.len(), tests.len());
        results
    }

    /// The start tests of `results` that neither pass nor are refused as
    /// `refused` says nor differ where `differing` names them.
    fn not_passing(
        results: &[TestResult],
        refused: &[(&str, &str)],
        differing: &[&str],
    ) -> Vec<String> {
        results
            .iter()
            .filter(|row| row.test.to_ascii_uppercase().contains("START"))
            .filter(|row| match &row.outcome {
                Outcome::Pass => false,
                Outcome::Refused { message, .. } => !refused.iter().any(|(test, method)| {
                    row.test.eq_ignore_ascii_case(test)
                        && message.contains(&format!("method \"{method}\""))
                }),
                Outcome::Differ { .. } => !differing
                    .iter()
                    .any(|test| row.test.eq_ignore_ascii_case(test)),
                _ => true,
            })
            .map(|row| format!("{} {}", row.test, row.outcome.label()))
            .collect()
    }

    #[test]
    fn the_outcome_table_of_the_message_group() {
        if !gate_mode() {
            eprintln!("group_runs: skipped without {GATE_ENV}");
            return;
        }
        let results = outcome_table(
            "message-table",
            "base/class",
            "Message",
            TABLE_ENV,
            SwitchMode::None,
            |_| true,
        );
        let failing = not_passing(&results, MESSAGE_START_REFUSED, MESSAGE_START_DIFFERING);
        assert!(failing.is_empty(), "not passing: {failing:?}");
        for test in ["TEST_SEND", "TEST_START", "TEST_REPLY", "TEST_NOTIFY"] {
            let row = results
                .iter()
                .find(|row| row.test.eq_ignore_ascii_case(test))
                .unwrap_or_else(|| panic!("{test} is listed"));
            assert!(
                matches!(row.outcome, Outcome::Pass),
                "{test} is {}",
                row.outcome.label()
            );
        }
    }

    /// The GUARD group's tests that pass, `TEST_WAIT_MULTIPLE` and the other
    /// `WHEN` waits among them. The rest are refused: a translation error
    /// inside the test's own `INTERPRET`.
    const GUARD_PASSING: &[&str] = &[
        "TEST_OFF",
        "TEST_ON",
        "TEST_ON_DEFAULT",
        "TEST_ON_OFF",
        "TEST_ON_OFF_CONSECUTIVE",
        "TEST_UNGUARDED",
        "TEST_WAIT_MULTIPLE",
        "TEST_WAIT_SIMPLE",
        "TEST_WAIT_SIMPLE_TRIGGER",
        "TEST_WHEN_MULTIPLE_NO_WAIT",
        "TEST_WHEN_NOT_BOOLEAN",
        "TEST_WHEN_NOVALUE",
        "TEST_WHEN_SINGLE_NO_WAIT",
        "TEST_WHEN_SINGLE_UNINITIALIZED_NO_WAIT",
        "TEST_WHEN_USE_LOCAL_NO_WAIT",
    ];

    /// [`GUARD_PASSING`] with the shipped scheduler and with a switch at every
    /// opportunity, and every other test refused.
    #[test]
    fn the_outcome_table_of_the_guard_group_in_both_modes() {
        if !gate_mode() {
            eprintln!("group_runs: skipped without {GATE_ENV}");
            return;
        }
        for (name, mode) in [
            ("guard-table", SwitchMode::None),
            ("guard-table-switched", SwitchMode::EveryOpportunity),
        ] {
            let results = outcome_table(
                name,
                "base/keyword",
                "GUARD",
                "REXX_GUARD_TABLE",
                mode,
                |_| true,
            );
            let passing: Vec<&str> = results
                .iter()
                .filter(|row| matches!(row.outcome, Outcome::Pass))
                .map(|row| row.test.as_str())
                .collect();
            assert_eq!(passing, GUARD_PASSING, "{name}");
            let other: Vec<String> = results
                .iter()
                .filter(|row| !matches!(row.outcome, Outcome::Pass | Outcome::Refused { .. }))
                .map(|row| format!("{} {}", row.test, row.outcome.label()))
                .collect();
            assert!(
                other.is_empty(),
                "{name}: neither passing nor refused: {other:?}"
            );
        }
    }

    /// Every test of the `Alarm` and `Ticker` groups passes with the shipped
    /// scheduler and with a switch at every opportunity; a [`WALL_CLOCK`] row
    /// that does not is run again once, the oracle too, and the table written
    /// to `REXX_TIMER_TABLE` names the rerun.
    #[test]
    fn the_alarm_and_ticker_groups_pass_in_both_modes() {
        if !gate_mode() {
            eprintln!("group_runs: skipped without {GATE_ENV}");
            return;
        }
        let mut not_passing = Vec::new();
        let mut table = String::from("group\tmode\ttest\toutcome\n");
        for group in ["Alarm", "Ticker"] {
            for (name, mode) in [
                ("timer-table", SwitchMode::None),
                ("timer-table-switched", SwitchMode::EveryOpportunity),
            ] {
                let results = outcome_table(
                    name,
                    "base/class",
                    group,
                    "REXX_TIMER_TABLE",
                    mode.clone(),
                    |_| true,
                );
                let oracle = oracle::locate();
                for row in &results {
                    let mut label = row.outcome.label().to_string();
                    let key = format!("base/class/{group}.testGroup {}", row.test);
                    if !matches!(row.outcome, Outcome::Pass) && WALL_CLOCK.contains(&key.as_str()) {
                        eprintln!("P48 rerun: {key} {name}: {label}");
                        let run = scratch(&format!("{name}-rerun"));
                        let one = std::slice::from_ref(&row.test);
                        let again = on_a_quiet_machine(|| {
                            run_tests(&oracle, &run, "base/class", group, one, mode.clone(), None)
                        });
                        fs::remove_dir_all(run.parent().expect("a parent"))
                            .expect("cannot remove the run");
                        label = format!("{label}, then {} (P48 rerun)", again[0].outcome.label());
                        eprintln!("P48 rerun: {key} {name}: {}", again[0].outcome.label());
                        if !matches!(again[0].outcome, Outcome::Pass) {
                            not_passing.push(format!("{group} {name} {} {label}", row.test));
                        }
                    } else if !matches!(row.outcome, Outcome::Pass) {
                        not_passing.push(format!("{group} {name} {} {label}", row.test));
                    }
                    table.push_str(&format!("{group}\t{name}\t{}\t{label}\n", row.test));
                }
            }
        }
        eprintln!("{table}");
        // Written over the last group's own table, so the file holds both
        // groups in both modes and each rerun.
        if let Some(path) = std::env::var_os("REXX_TIMER_TABLE") {
            fs::write(path, &table).expect("cannot write the table");
        }
        assert!(not_passing.is_empty(), "not passing: {not_passing:?}");
    }

    #[test]
    fn the_outcome_table_of_the_object_group() {
        if !gate_mode() {
            eprintln!("group_runs: skipped without {GATE_ENV}");
            return;
        }
        // The start tests only: `TEST_UNINIT` and `TEST_UNINIT_CLASS`
        // allocate until the memory cap stops them, on the base as here, and
        // two at once outgrow the gate's.
        let results = outcome_table(
            "object-table",
            "base/class",
            "Object",
            OBJECT_TABLE_ENV,
            SwitchMode::None,
            |test| test.to_ascii_uppercase().contains("START"),
        );
        let failing = not_passing(&results, &[], &[]);
        assert!(failing.is_empty(), "not passing: {failing:?}");
    }

    /// The Message start tests again with a switch at every clause boundary
    /// the started activities reach.
    #[test]
    fn the_message_start_tests_under_every_opportunity() {
        if !gate_mode() {
            eprintln!("group_runs: skipped without {GATE_ENV}");
            return;
        }
        let results = outcome_table(
            "message-table-switched",
            "base/class",
            "Message",
            SWITCHED_TABLE_ENV,
            SwitchMode::EveryOpportunity,
            |test| test.to_ascii_uppercase().contains("START"),
        );
        let failing = not_passing(
            &results,
            MESSAGE_START_REFUSED,
            MESSAGE_START_DIFFERING_SWITCHED,
        );
        assert!(failing.is_empty(), "not passing: {failing:?}");
    }

    /// The features of criterion 1's list that S2 covers; a row naming none
    /// of them belongs to a later stage.
    const S2_FEATURES: &[&str] = &[
        "REPLY",
        "~start",
        "Message~reply",
        "SysSleep",
        ".context~thread",
        "TraceObject field",
    ];

    /// Where the both-modes table of the S2 rows is written when set.
    const CRITERION_ONE_TABLE_ENV: &str = "REXX_CRITERION_ONE_TABLE";

    /// The S2 rows whose runs differ between the two modes, in the assertion
    /// count alone: each asserts after a `REPLY` in the continuation, which
    /// races the end of the program. The shipped scheduler ends first;
    /// `EveryOpportunity` runs the continuation first (ruling P41).
    const MODE_DIFFERING: &[&str] = &[
        "base/keyword/REPLY.testGroup TEST_REPLY_TWICE_REPLYASSERT",
        "base/keyword/REPLY.testGroup TEST_REPLY_RETURN_CODE_REPLYASSERT",
        "base/keyword/REPLY.testGroup TEST_REPLY_RETURN_CODE_SAME_REPLYASSERT",
        "base/keyword/REPLY.testGroup TEST_REPLY_EXIT_CODE_REPLYASSERT",
        "base/keyword/REPLY.testGroup TEST_REPLY_STACK_REPLYASSERT",
        "base/keyword/REPLY.testGroup TEST_REPLY_SAME_REPLYASSERT",
    ];

    /// The text of the crate's inverted-wait refusal (`lib.rs`, `inverted_wait`).
    const INVERTED_WAIT: &str = "that only an activity pinned below it can end";

    /// Whether `run` ended with the inverted-wait refusal.
    fn inverted(run: &Run) -> bool {
        String::from_utf8_lossy(&run.stderr).contains(INVERTED_WAIT)
    }

    /// The rows whose failure output prints an elapsed time, which differs
    /// between any two runs.
    const PRINTS_ELAPSED: &[&str] = &[
        "base/bif/TIME.testGroup TEST_4",
        "base/bif/TIME.testGroup TEST_5",
        "base/bif/TIME.testGroup TEST_10",
        "base/bif/TIME.testGroup TEST_11",
    ];

    /// The rows whose trace output, on stderr, comes from a `REPLY`
    /// continuation and its sender, so the lines interleave differently by
    /// mode. Allowed while both runs end in the same `rexx-exec: ` refusal, or
    /// end alike with the same stderr lines in another order.
    const TRACE_INTERLEAVES: &[&str] =
        &["base/keyword/TRACE_TraceObject.testGroup TEST_TRACEOBJECT_COLLECTOR"];

    /// The rows whose outcome on this crate depends on a wall-clock boundary or
    /// a sleep's duration, each with the lines that make it so: a failing check
    /// on one is run again once, and only a second failure counts (ruling P48).
    const WALL_CLOCK: &[&str] = &[
        // STREAM.testGroup:866-876
        "base/bif/STREAM.testGroup TEST_QUERYDIR_EXISTS",
        // DateTime.testGroup:593-596
        "base/class/DateTime.testGroup TEST_ELAPSED1",
        // SysSleep.testGroup:83-84, :107-113
        "base/rexxutil/SysSleep.testGroup TEST_SLEEP_DURATION",
        // SysSleep.testGroup:91-99
        "base/rexxutil/SysSleep.testGroup TEST_SLEEP_CONCURRENT",
        // EventSemaphore.testGroup:151-155, :162
        "base/class/EventSemaphore.testGroup TEST_WAIT_CONCURRENT",
        // MutexSemaphore.testGroup:124, :128
        "base/class/MutexSemaphore.testGroup TEST_EXCLUSION",
        // Message.testGroup:651-652, :667-668, :685-686, :958, :977
        "base/class/Message.testGroup TEST_HALT_START",
        // CALL.testGroup:424-425
        "base/keyword/CALL.testGroup TEST_4",
        // TIME.testGroup:1840, :1875 and the same midnight assertion in each
        "base/bif/TIME.testGroup TEST_2",
        "base/bif/TIME.testGroup TEST_3",
        "base/bif/TIME.testGroup TEST_4",
        "base/bif/TIME.testGroup TEST_5",
        "base/bif/TIME.testGroup TEST_8",
        "base/bif/TIME.testGroup TEST_9",
        "base/bif/TIME.testGroup TEST_10",
        "base/bif/TIME.testGroup TEST_11",
        // Ticker.testGroup:52-55, :76-79, :100-103, :123-126
        "base/class/Ticker.testGroup TEST_TICKER_TWO_ARGS_STRING_CANCEL",
        "base/class/Ticker.testGroup TEST_TICKER_TWO_ARGS_TIMESPAN_CANCEL",
        "base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_STRING_CANCEL",
        "base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_TIMESPAN_CANCEL",
        // Ticker.testGroup:62-65, :86-89, :110-113, :133-136, :164-170
        "base/class/Ticker.testGroup TEST_TICKER_TWO_ARGS_STRING_TRIGGER",
        "base/class/Ticker.testGroup TEST_TICKER_TWO_ARGS_TIMESPAN_TRIGGER",
        "base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_STRING_TRIGGER",
        "base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_TIMESPAN_TRIGGER",
        "base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_STRING_TRIGGER_MESSAGE",
        // Ticker.testGroup:147-150
        "base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_TIMESPAN_TRIGGER_MULTIPLE",
        // Alarm.testGroup:57-64, :74, :119-127, :137-146, :157-167, :201-209
        "base/class/Alarm.testGroup TEST_BASE_ALARM",
    ];

    /// The row whose run under `EveryOpportunity` hangs at the test's own
    /// race (ruling P46): main takes the mutex between the worker's
    /// `step = 6` and its last `acquire` and never releases it. The oracle
    /// hangs in the same interleaving, 3 of 3 at rc 137, on
    /// `docs/superpowers/records/2026-10-01-phase-6-s2-s5/exclusion-every-hang.rex`.
    /// Allowed only while the unswitched run passes at rc 0 and the switched
    /// one ends at the deadline.
    const EVERY_FORCES_THE_RACE: &str = "base/class/MutexSemaphore.testGroup TEST_EXCLUSION";

    /// `run`'s stdout without the lines `drop` selects.
    fn without(run: &Run, drop: impl Fn(&str) -> bool) -> String {
        String::from_utf8_lossy(&masked(&run.stdout))
            .lines()
            .filter(|line| !drop(line))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn assertions(run: &Run) -> String {
        String::from_utf8_lossy(&run.stdout)
            .lines()
            .find_map(|line| line.strip_prefix("Assertions:"))
            .map_or_else(|| "none".to_string(), |count| count.trim().to_string())
    }

    /// How two runs of this crate compare, and whether the difference, if
    /// any, is one `row` is listed for.
    fn compare_modes(row: &str, normal: &Run, every: &Run) -> (String, bool) {
        if masked(&normal.stdout) == masked(&every.stdout)
            && normal.stderr == every.stderr
            && normal.status == every.status
        {
            return ("same".to_string(), true);
        }
        let quiet = normal.stderr == every.stderr && normal.status == every.status;
        if quiet && normal.status == Some(0) && MODE_DIFFERING.contains(&row) {
            let count = |line: &str| line.starts_with("Assertions:");
            if without(normal, count) == without(every, count) {
                return (
                    format!(
                        "assertions {} then {}",
                        assertions(normal),
                        assertions(every)
                    ),
                    true,
                );
            }
        }
        if quiet && PRINTS_ELAPSED.contains(&row) {
            let timed = |line: &str| {
                let line = line.trim_start();
                line.starts_with("[failure]")
                    || line.starts_with("Expected:")
                    || line.starts_with("Actual:")
                    || line.starts_with("Message:")
            };
            if without(normal, timed) == without(every, timed) {
                return ("same apart from elapsed values".to_string(), true);
            }
        }
        let refusal = |run: &Run| {
            String::from_utf8_lossy(&run.stderr)
                .lines()
                .find(|line| line.starts_with("rexx-exec: "))
                .map(str::to_string)
        };
        let sorted_lines = |run: &Run| {
            let mut lines: Vec<String> = String::from_utf8_lossy(&run.stderr)
                .lines()
                .map(str::to_string)
                .collect();
            lines.sort();
            lines
        };
        if normal.status == every.status
            && masked(&normal.stdout) == masked(&every.stdout)
            && TRACE_INTERLEAVES.contains(&row)
        {
            if refusal(normal).is_some() && refusal(normal) == refusal(every) {
                return ("trace lines on stderr differ".to_string(), true);
            }
            if sorted_lines(normal) == sorted_lines(every) {
                return ("trace lines on stderr in another order".to_string(), true);
            }
        }
        (
            format!(
                "differs: status {:?} then {:?}",
                normal.status, every.status
            ),
            false,
        )
    }

    /// A result as a key and a detail. Two results agree when their keys do:
    /// the label, and the refusal or the status of a difference. The detail
    /// is the first line where a difference's stdout departs from the
    /// oracle's.
    fn cell(outcome: &Outcome) -> (String, String) {
        match outcome {
            Outcome::Refused { message, .. } => (format!("refused: {message}"), String::new()),
            Outcome::Differ { oracle, ours } => {
                let detail = first_difference(&masked(&oracle.stdout), &masked(&ours.stdout))
                    .replace(['\n', '\t'], " ");
                match ours.status {
                    None => ("differ: did not finish".to_string(), detail),
                    Some(status) => (format!("differ: rc {status}"), detail),
                }
            }
            other => (other.label().to_string(), String::new()),
        }
    }

    /// The features of criterion 1's list that S3 covers.
    const S3_FEATURES: &[&str] = &["GUARD", "semaphore class", "Sys*Sem", "Alarm", "Ticker"];

    /// Where the both-modes table of the S3 rows is written when set.
    const S3_TABLE_ENV: &str = "REXX_CRITERION_ONE_S3_TABLE";

    /// Every S2 row of the derived list, run on both sides with the shipped
    /// scheduler and under `EveryOpportunity`.
    #[test]
    fn the_s2_rows_of_the_derived_list_in_both_modes() {
        rows_in_both_modes("criterion-one", CRITERION_ONE_TABLE_ENV, |features| {
            features.iter().any(|f| S2_FEATURES.contains(f))
        });
    }

    /// Every row of the derived list naming an S3 feature and no S2 one, as
    /// [`the_s2_rows_of_the_derived_list_in_both_modes`] runs the others.
    #[test]
    fn the_s3_rows_of_the_derived_list_in_both_modes() {
        rows_in_both_modes("criterion-one-s3", S3_TABLE_ENV, |features| {
            !features.iter().any(|f| S2_FEATURES.contains(f))
                && features.iter().any(|f| S3_FEATURES.contains(f))
        });
    }

    /// The rows of the derived list whose features `chosen` accepts, run in
    /// the scratch directory `name`, with the table written to the file
    /// `table_env` names where set.
    fn rows_in_both_modes(
        name: &str,
        table_env: &str,
        chosen: impl Fn(&BTreeSet<&'static str>) -> bool,
    ) {
        if !gate_mode() {
            eprintln!("group_runs: skipped without {GATE_ENV}");
            return;
        }
        let (list, _) = super::derive(&super::worktree().join("ootest/ooRexx"));
        let mut groups: std::collections::BTreeMap<String, Vec<String>> = Default::default();
        for row in list.iter().filter(|row| chosen(&row.features)) {
            groups
                .entry(row.group.clone())
                .or_default()
                .push(row.test.clone());
        }
        let rows: Vec<(&String, &String)> = groups
            .iter()
            .flat_map(|(file, tests)| tests.iter().map(move |test| (file, test)))
            .collect();
        let oracle = oracle::locate();
        let results = rows_in_parallel(
            name,
            &rows,
            |(file, test)| WALL_CLOCK.contains(&format!("{file} {test}").as_str()),
            |run, (file, test)| row_in_both_modes(&oracle, run, file, test),
        );
        let mut table =
            String::from("group\ttest\tnormal against the oracle\tevery against normal\n");
        let mut differing = Vec::new();
        let mut stuck = Vec::new();
        for ((file, test), (normal, modes, hung)) in rows.iter().zip(results) {
            stuck.extend(hung);
            if !modes.1 {
                differing.push(format!("{file} {test}"));
            }
            let show = |(key, detail): &(String, String)| {
                if detail.is_empty() {
                    key.clone()
                } else {
                    format!("{key}: {detail}")
                }
            };
            table.push_str(&format!("{file}\t{test}\t{}\t{}\n", show(&normal), modes.0));
        }
        eprintln!("{table}");
        if let Some(path) = std::env::var_os(table_env) {
            fs::write(path, &table).expect("cannot write the table");
        }
        assert!(stuck.is_empty(), "an inverted wait or a hang: {stuck:?}");
        assert!(
            differing.is_empty(),
            "tests differing between the modes: {differing:?}"
        );
    }

    /// One row of [`rows_in_both_modes`] in the copy `run`: its cell against
    /// the oracle, how the two modes compare and whether that is allowed, and
    /// the modes that hung or ended in an inverted wait.
    fn row_in_both_modes(
        oracle: &oracle::Oracle,
        run: &Path,
        file: &str,
        test: &str,
    ) -> ((String, String), (String, bool), Vec<String>) {
        let (dir, group) = file
            .trim_end_matches(".testGroup")
            .rsplit_once('/')
            .expect("a group below a directory");
        let one = &[test.to_string()];
        // The runner asserts that the oracle finishes within the deadline; a
        // test it outlasts is recorded as such.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_tests(oracle, run, dir, group, one, SwitchMode::None, None)
        }));
        let normal = match result {
            Ok(results) => cell(&results[0].outcome),
            Err(payload) => {
                let deadline = payload
                    .downcast_ref::<String>()
                    .is_some_and(|m| m.starts_with("the oracle did not finish"));
                if !deadline {
                    std::panic::resume_unwind(payload);
                }
                ("oracle did not finish".to_string(), String::new())
            }
        };
        if reaching_rxapi(dir, &[group]).contains(&format!("{group}.{test}")) {
            return (normal, ("not run".to_string(), true), Vec::new());
        }
        let row = format!("{file} {test}");
        let judge = || {
            let group_path = group_file(run, dir, group);
            let args = ["-f", group_path.as_str(), "-U", "-V", VERBOSITY, "-t", test];
            fresh_copy(run, dir);
            let shipped = run_crate(run, &args, SwitchMode::None);
            fresh_copy(run, dir);
            let every = run_crate(run, &args, SwitchMode::EveryOpportunity);
            let raced = row == EVERY_FORCES_THE_RACE
                && normal.0 == "pass"
                && shipped.status == Some(0)
                && !inverted(&shipped)
                && every.status.is_none()
                && !inverted(&every);
            let mut hung = Vec::new();
            if raced {
                let modes = (
                    "every mode forces the test's own race; the oracle hangs in the \
                     same interleaving (P46)"
                        .to_string(),
                    true,
                );
                return (modes, hung);
            }
            for (name, ours) in [("normal", &shipped), ("every", &every)] {
                if inverted(ours) || ours.status.is_none() {
                    hung.push(format!("{row}: {name}"));
                }
            }
            (compare_modes(&row, &shipped, &every), hung)
        };
        let (mut judged, mut hung) = judge();
        if (!judged.1 || !hung.is_empty()) && WALL_CLOCK.contains(&row.as_str()) {
            eprintln!("P48 rerun: {row}: {} {hung:?}", judged.0);
            (judged, hung) = judge();
            judged.0.push_str(" (P48 rerun)");
        }
        (normal, judged, hung)
    }

    /// The `TRACE_TraceObject` tests that pass; the table printed beside
    /// them names what each of the others waits on.
    const TRACE_OBJECT_PASSING: &[&str] = &[
        "TEST_SETMAKESTRING_WITH_METHOD_OBJECT",
        "TEST_TRACEOBJECT_OPTION",
        "TEST_TRACEOBJECT_OPTION_INVALID",
    ];

    #[test]
    fn the_outcome_table_of_the_trace_object_group() {
        if !gate_mode() {
            eprintln!("group_runs: skipped without {GATE_ENV}");
            return;
        }
        let results = outcome_table(
            "trace-object-table",
            "base/keyword",
            "TRACE_TraceObject",
            "REXX_TRACE_OBJECT_TABLE",
            SwitchMode::None,
            |_| true,
        );
        let passing: Vec<&str> = results
            .iter()
            .filter(|row| matches!(row.outcome, Outcome::Pass))
            .map(|row| row.test.as_str())
            .collect();
        assert_eq!(passing, TRACE_OBJECT_PASSING);
    }

    /// Each group file of criterion 1's derived list in one run on the
    /// oracle and on this crate in both modes, whole and with only its tests
    /// of the derived list, the tests reaching rxapi renamed out of both.
    /// Gate-only.
    mod whole_groups {
        use std::collections::BTreeSet;
        use std::path::{Path, PathBuf};
        use std::time::{Duration, Instant};

        use super::super::group_runner::{
            GATE_ENV, Run, SwitchMode, VERBOSITY, agree, first_difference, fresh_copy, gate_mode,
            group_file, masked, outcome, reaching_rxapi, read_lossy, rows_in_parallel,
            run_crate_within, run_oracle_within, skip,
        };
        use super::super::support::oracle;
        use super::{WALL_CLOCK, inverted};

        /// Where the table is written when set.
        const TABLE_ENV: &str = "REXX_WHOLE_GROUPS_TABLE";

        /// A directory each run's descriptors are written to when set.
        const DUMP_ENV: &str = "REXX_WHOLE_GROUPS_DUMP";

        /// Where the tests each run here started are written when set.
        const STARTED_ENV: &str = "REXX_WHOLE_GROUPS_STARTED";

        /// Oracle runs per group, and per group whose oracle outcome varies or
        /// differs from ours (ruling P83).
        const ORACLE_RUNS: usize = 5;
        const ORACLE_RUNS_UNSETTLED: usize = 30;

        /// How many runs of one group run at once.
        const RUNS_AT_ONCE: usize = 5;

        /// The bound on one oracle run: `base/bif/TIME` whole takes about
        /// 94 s.
        const ORACLE_DEADLINE: Duration = Duration::from_secs(300);

        /// The bound on one run here: the oracle's slowest run times
        /// `CRATE_FACTOR`, and at least `CRATE_FLOOR`.
        const CRATE_FACTOR: u32 = 4;
        const CRATE_FLOOR: Duration = Duration::from_secs(60);

        /// Which tests of the group a run holds: all, those of the derived
        /// list, or all but the tests whose run here refuses (ruling P85).
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum Part {
            Whole,
            Derived,
            Rest,
        }

        impl Part {
            fn label(self) -> &'static str {
                match self {
                    Part::Whole => "whole",
                    Part::Derived => "derived",
                    Part::Rest => "rest",
                }
            }
        }

        /// The tests every part leaves out, on both sides, each with its reason:
        /// each allocates here until memory runs out, which takes the shared
        /// test process with it.
        const REST_LEFT_OUT: &[(&str, &str, &str)] = &[
            (
                "base/class/Class.testGroup",
                "TEST_SUBCLASSES_GC",
                "loops until a dropped class is collected; classes are never collected (D59)",
            ),
            (
                "base/class/Object.testGroup",
                "TEST_UNINIT",
                "loops until a weak reference is cleared, which here it is not before memory runs out",
            ),
            (
                "base/class/Object.testGroup",
                "TEST_UNINIT_CLASS",
                "loops until a weak reference is cleared, which here it is not before memory runs out",
            ),
        ];

        /// The runs here, by group, part and mode, that need not be an outcome
        /// the oracle produced, each with its [`key`] and reason.
        const DIFFERING: &[(&str, &str, &str, &str, &str)] = &[
            (
                "base/bif/STREAM.testGroup",
                "whole",
                "normal",
                "error, assertions 195, rc 2, last started TEST_WRONG_TOO_MANY_ARGS_S, failing [TEST_RELATIVE_FILE_EXISTS TEST_RELATIVE_FILE_EXISTS2 TEST_QUERYFILE_EXISTS_OPENED_01 TEST_QUERYFILE_EXISTS_OPENED_03 TEST_SEEK_CLOSEDFILE_2785896 TEST_SEEK_CLOSEDFILE_2787994 TEST_OPEN_WRITE_ONLY_3274050_C TEST_OPEN_WRITE_ONLY_3274050_D]",
                "tests outside the derived list that fail alone too (whole-groups/alone.txt), and TEST_RELATIVE_FILE_EXISTS and _EXISTS2, which fail on the oracle too",
            ),
            (
                "base/bif/STREAM.testGroup",
                "whole",
                "every",
                "error, assertions 195, rc 2, last started TEST_WRONG_TOO_MANY_ARGS_S, failing [TEST_RELATIVE_FILE_EXISTS TEST_RELATIVE_FILE_EXISTS2 TEST_QUERYFILE_EXISTS_OPENED_01 TEST_QUERYFILE_EXISTS_OPENED_03 TEST_SEEK_CLOSEDFILE_2785896 TEST_SEEK_CLOSEDFILE_2787994 TEST_OPEN_WRITE_ONLY_3274050_C TEST_OPEN_WRITE_ONLY_3274050_D]",
                "tests outside the derived list that fail alone too (whole-groups/alone.txt), and TEST_RELATIVE_FILE_EXISTS and _EXISTS2, which fail on the oracle too",
            ),
            (
                "base/bif/TIME.testGroup",
                "whole",
                "normal",
                "failure, assertions 506, rc 1, last started TEST_9, failing [TEST_VALIDOPT_BIGCHAR_R TEST_VALIDOPT_LITTLECHAR_R TEST_10 TEST_11 TEST_4 TEST_5]",
                "elapsed-clock defect, queued 2026-10-02-elapsed-clock-per-routine-and-reset: TEST_4, 5, 10, 11 as alone, and TEST_VALIDOPT_*_R, which pass alone, after earlier tests (whole-groups/clock)",
            ),
            (
                "base/bif/TIME.testGroup",
                "whole",
                "every",
                "failure, assertions 506, rc 1, last started TEST_9, failing [TEST_VALIDOPT_BIGCHAR_R TEST_VALIDOPT_LITTLECHAR_R TEST_10 TEST_11 TEST_4 TEST_5]",
                "elapsed-clock defect, queued 2026-10-02-elapsed-clock-per-routine-and-reset: TEST_4, 5, 10, 11 as alone, and TEST_VALIDOPT_*_R, which pass alone, after earlier tests (whole-groups/clock)",
            ),
            (
                "base/bif/TIME.testGroup",
                "derived",
                "normal",
                "failure, assertions 65, rc 1, last started TEST_9, failing [TEST_10 TEST_11 TEST_4 TEST_5]",
                "elapsed-clock defect, queued 2026-10-02-elapsed-clock-per-routine-and-reset: TEST_4, 5, 10, 11 as alone",
            ),
            (
                "base/bif/TIME.testGroup",
                "derived",
                "every",
                "failure, assertions 65, rc 1, last started TEST_9, failing [TEST_10 TEST_11 TEST_4 TEST_5]",
                "elapsed-clock defect, queued 2026-10-02-elapsed-clock-per-routine-and-reset: TEST_4, 5, 10, 11 as alone",
            ),
            (
                "base/class/Class.testGroup",
                "whole",
                "normal",
                "failure, assertions 298, rc 1, last started TEST_UNINHERIT_TWO_ARGS, failing [TEST_ACTIVATE TEST_METHODS]",
                "tests outside the derived list that fail alone too (whole-groups/alone.txt)",
            ),
            (
                "base/class/Class.testGroup",
                "whole",
                "every",
                "failure, assertions 298, rc 1, last started TEST_UNINHERIT_TWO_ARGS, failing [TEST_ACTIVATE TEST_METHODS]",
                "tests outside the derived list that fail alone too (whole-groups/alone.txt)",
            ),
            (
                "base/class/Message.testGroup",
                "whole",
                "normal",
                "rexx-exec: method \"MAKEARRAY\" of class \"Object\" is not implemented (Phase 9), rc 120, last started TEST_REPLYWITH_NOT_ARRAY, failing []",
                "TEST_REPLYWITH_NOT_ARRAY refuses, as alone (Phase 9)",
            ),
            (
                "base/class/Message.testGroup",
                "whole",
                "every",
                "rexx-exec: method \"MAKEARRAY\" of class \"Object\" is not implemented (Phase 9), rc 120, last started TEST_REPLYWITH_NOT_ARRAY, failing []",
                "TEST_REPLYWITH_NOT_ARRAY refuses, as alone (Phase 9)",
            ),
            (
                "base/class/Message.testGroup",
                "derived",
                "normal",
                "rexx-exec: method \"MAKEARRAY\" of class \"Object\" is not implemented (Phase 9), rc 120, last started TEST_REPLYWITH_NOT_ARRAY, failing []",
                "TEST_REPLYWITH_NOT_ARRAY refuses, as alone (Phase 9)",
            ),
            (
                "base/class/Message.testGroup",
                "derived",
                "every",
                "rexx-exec: method \"MAKEARRAY\" of class \"Object\" is not implemented (Phase 9), rc 120, last started TEST_REPLYWITH_NOT_ARRAY, failing []",
                "TEST_REPLYWITH_NOT_ARRAY refuses, as alone (Phase 9)",
            ),
            (
                "base/class/Method.testGroup",
                "whole",
                "normal",
                "rexx-exec: method \"NEW\" of class \"Method\" is not implemented (Phase 9), rc 120, last started TEST_NEW_CONTEXT_FLOATINGMETHOD, failing []",
                "a test outside the derived list refuses: TEST_NEW_CONTEXT_FLOATINGMETHOD, Method NEW (Phase 9)",
            ),
            (
                "base/class/Method.testGroup",
                "whole",
                "every",
                "rexx-exec: method \"NEW\" of class \"Method\" is not implemented (Phase 9), rc 120, last started TEST_NEW_CONTEXT_FLOATINGMETHOD, failing []",
                "a test outside the derived list refuses: TEST_NEW_CONTEXT_FLOATINGMETHOD, Method NEW (Phase 9)",
            ),
            (
                "base/class/Method.testGroup",
                "rest",
                "normal",
                "error, assertions 64, rc 2, last started TEST_SOURCE, failing [TEST_NEWFILE_CONTEXT_FLOATINGMETHOD TEST_NEWFILE_CONTEXT_IMPORTEDPACKAGE TEST_NEWFILE_CONTEXT_METHOD TEST_NEWFILE_CONTEXT_METHODPUBLIC TEST_NEWFILE_CONTEXT_OMITTED TEST_NEWFILE_CONTEXT_PROGRAMSCOPE TEST_NEWFILE_CONTEXT_ROUTINE TEST_NEWFILE_CONTEXT_THISPACKAGE TEST_NEW_CONTEXT_OMITTED TEST_NEW_ARRAY_FROM_FILE TEST_NEW_FILE_COMPILED]",
                "tests outside the derived list that fail alone too: Method NEW's file, array and context shapes",
            ),
            (
                "base/class/Method.testGroup",
                "rest",
                "every",
                "error, assertions 64, rc 2, last started TEST_SOURCE, failing [TEST_NEWFILE_CONTEXT_FLOATINGMETHOD TEST_NEWFILE_CONTEXT_IMPORTEDPACKAGE TEST_NEWFILE_CONTEXT_METHOD TEST_NEWFILE_CONTEXT_METHODPUBLIC TEST_NEWFILE_CONTEXT_OMITTED TEST_NEWFILE_CONTEXT_PROGRAMSCOPE TEST_NEWFILE_CONTEXT_ROUTINE TEST_NEWFILE_CONTEXT_THISPACKAGE TEST_NEW_CONTEXT_OMITTED TEST_NEW_ARRAY_FROM_FILE TEST_NEW_FILE_COMPILED]",
                "tests outside the derived list that fail alone too: Method NEW's file, array and context shapes",
            ),
            (
                "base/class/MutexSemaphore.testGroup",
                "whole",
                "every",
                "rexx-exec: the run exceeded its deadline, rc none, last started TEST_RELEASE_ONE_ARG, failing []",
                "TEST_EXCLUSION's own race: the oracle hangs in it when that schedule is forced (SysSleep 0.01 after step = 6, P46)",
            ),
            (
                "base/class/MutexSemaphore.testGroup",
                "derived",
                "every",
                "rexx-exec: the run exceeded its deadline, rc none, last started TEST_RELEASE_ONE_ARG, failing []",
                "TEST_EXCLUSION's own race: the oracle hangs in it when that schedule is forced (SysSleep 0.01 after step = 6, P46)",
            ),
            (
                "base/class/Object.testGroup",
                "whole",
                "normal",
                "rexx-exec: method \"MAKEARRAY\" of class \"Object\" is not implemented (Phase 9), rc 120, last started TEST_RUN_ARRAY_ARGUMENT, failing []",
                "a test outside the derived list refuses: TEST_RUN_ARRAY_ARGUMENT, Phase 9",
            ),
            (
                "base/class/Object.testGroup",
                "whole",
                "every",
                "rexx-exec: method \"MAKEARRAY\" of class \"Object\" is not implemented (Phase 9), rc 120, last started TEST_RUN_ARRAY_ARGUMENT, failing []",
                "a test outside the derived list refuses: TEST_RUN_ARRAY_ARGUMENT, Phase 9",
            ),
            (
                "base/class/Object.testGroup",
                "rest",
                "normal",
                "failure, assertions 249, rc 1, last started TEST_UNSETMETHOD_NO_ARG, failing [TEST_INSTANCEMETHOD TEST_RUN TEST_SETMETHOD TEST_SETMETHOD_SCOPE]",
                "tests outside the derived list that fail alone too (whole-groups/alone.txt)",
            ),
            (
                "base/class/Object.testGroup",
                "rest",
                "every",
                "failure, assertions 249, rc 1, last started TEST_UNSETMETHOD_NO_ARG, failing [TEST_INSTANCEMETHOD TEST_RUN TEST_SETMETHOD TEST_SETMETHOD_SCOPE]",
                "tests outside the derived list that fail alone too (whole-groups/alone.txt)",
            ),
            (
                "base/class/RexxContext.testGroup",
                "whole",
                "normal",
                "rexx-exec: method \"COPY\" of class \"RexxContext\" is not implemented (Phase 9), rc 120, last started TESTCOPY01, failing []",
                "a test outside the derived list refuses: TESTCOPY01, RexxContext COPY (Phase 9)",
            ),
            (
                "base/class/RexxContext.testGroup",
                "whole",
                "every",
                "rexx-exec: method \"COPY\" of class \"RexxContext\" is not implemented (Phase 9), rc 120, last started TESTCOPY01, failing []",
                "a test outside the derived list refuses: TESTCOPY01, RexxContext COPY (Phase 9)",
            ),
            (
                "base/class/RexxContext.testGroup",
                "rest",
                "normal",
                "failure, assertions 357, rc 1, last started TEST_SOURCELINE_LAST_LINE, failing [TESTRS01]",
                "tests outside the derived list that fail alone too (whole-groups/alone.txt)",
            ),
            (
                "base/class/RexxContext.testGroup",
                "rest",
                "every",
                "failure, assertions 357, rc 1, last started TEST_SOURCELINE_LAST_LINE, failing [TESTRS01]",
                "tests outside the derived list that fail alone too (whole-groups/alone.txt)",
            ),
            (
                "base/directives/ATTRIBUTE.testGroup",
                "whole",
                "normal",
                "error, assertions 276, rc 2, last started TEST_SET_RETURN, failing [TESTDELEGATE TESTMISPLACEDCLASSMETHOD]",
                "TESTDELEGATE, as alone: Method delegate attributes, outside Phase 6; TESTMISPLACEDCLASSMETHOD: `::attribute 'foo' class` with a body is 99.937 here, 99.905 on the oracle",
            ),
            (
                "base/directives/ATTRIBUTE.testGroup",
                "whole",
                "every",
                "error, assertions 276, rc 2, last started TEST_SET_RETURN, failing [TESTDELEGATE TESTMISPLACEDCLASSMETHOD]",
                "TESTDELEGATE, as alone: Method delegate attributes, outside Phase 6; TESTMISPLACEDCLASSMETHOD: `::attribute 'foo' class` with a body is 99.937 here, 99.905 on the oracle",
            ),
            (
                "base/directives/ATTRIBUTE.testGroup",
                "derived",
                "normal",
                "failure, assertions 187, rc 1, last started TESTDELEGATE, failing [TESTDELEGATE]",
                "TESTDELEGATE, as alone: Method delegate attributes, outside Phase 6",
            ),
            (
                "base/directives/ATTRIBUTE.testGroup",
                "derived",
                "every",
                "failure, assertions 187, rc 1, last started TESTDELEGATE, failing [TESTDELEGATE]",
                "TESTDELEGATE, as alone: Method delegate attributes, outside Phase 6",
            ),
            (
                "base/directives/CONSTANT.testGroup",
                "whole",
                "normal",
                "rexx-exec: method \"NEW\" of class \"Routine\" is not implemented (Phase 9), rc 120, last started TEST_EXPRESSION_ACTIVATE, failing []",
                "a test outside the derived list refuses: TEST_EXPRESSION_ACTIVATE, Routine NEW (Phase 9)",
            ),
            (
                "base/directives/CONSTANT.testGroup",
                "whole",
                "every",
                "rexx-exec: method \"NEW\" of class \"Routine\" is not implemented (Phase 9), rc 120, last started TEST_EXPRESSION_ACTIVATE, failing []",
                "a test outside the derived list refuses: TEST_EXPRESSION_ACTIVATE, Routine NEW (Phase 9)",
            ),
            (
                "base/directives/CONSTANT.testGroup",
                "rest",
                "normal",
                "error, assertions 170, rc 2, last started TEST_SUBCLASS_DEFINITION, failing [TEST_EXPRESSION_NOVALUE_ERROR]",
                "tests outside the derived list that fail alone too (whole-groups/alone.txt)",
            ),
            (
                "base/directives/CONSTANT.testGroup",
                "rest",
                "every",
                "error, assertions 170, rc 2, last started TEST_SUBCLASS_DEFINITION, failing [TEST_EXPRESSION_NOVALUE_ERROR]",
                "tests outside the derived list that fail alone too (whole-groups/alone.txt)",
            ),
            (
                "base/directives/METHOD.testGroup",
                "whole",
                "normal",
                "rexx-exec: method \"ITEMS\" of class \"StringTable\" is not implemented (Phase 9), rc 120, last started TESTPACKAGE, failing []",
                "a test outside the derived list refuses: TESTPACKAGE, StringTable ITEMS (Phase 9)",
            ),
            (
                "base/directives/METHOD.testGroup",
                "whole",
                "every",
                "rexx-exec: method \"ITEMS\" of class \"StringTable\" is not implemented (Phase 9), rc 120, last started TESTPACKAGE, failing []",
                "a test outside the derived list refuses: TESTPACKAGE, StringTable ITEMS (Phase 9)",
            ),
            (
                "base/directives/METHOD.testGroup",
                "rest",
                "normal",
                "failure, assertions 179, rc 1, last started TEST_DELEGATE_TWICE, failing [TESTDELEGATE]",
                "TESTDELEGATE, as alone: Method delegate attributes, outside Phase 6",
            ),
            (
                "base/directives/METHOD.testGroup",
                "rest",
                "every",
                "failure, assertions 179, rc 1, last started TEST_DELEGATE_TWICE, failing [TESTDELEGATE]",
                "TESTDELEGATE, as alone: Method delegate attributes, outside Phase 6",
            ),
            (
                "base/directives/METHOD.testGroup",
                "derived",
                "normal",
                "failure, assertions 67, rc 1, last started TESTGUARDEDACCESS, failing [TESTDELEGATE]",
                "TESTDELEGATE, as alone: Method delegate attributes, outside Phase 6",
            ),
            (
                "base/directives/METHOD.testGroup",
                "derived",
                "every",
                "failure, assertions 67, rc 1, last started TESTGUARDEDACCESS, failing [TESTDELEGATE]",
                "TESTDELEGATE, as alone: Method delegate attributes, outside Phase 6",
            ),
            (
                "base/keyword/CALL.testGroup",
                "derived",
                "normal",
                "failure, assertions 7, rc 1, last started TEST_4, failing [TEST_4]",
                "TEST_4, as alone: elapsed-clock defect, queued 2026-10-02-elapsed-clock-per-routine-and-reset",
            ),
            (
                "base/keyword/CALL.testGroup",
                "derived",
                "every",
                "failure, assertions 7, rc 1, last started TEST_4, failing [TEST_4]",
                "TEST_4, as alone: elapsed-clock defect, queued 2026-10-02-elapsed-clock-per-routine-and-reset",
            ),
            (
                "base/keyword/RAISE.testGroup",
                "whole",
                "normal",
                "failure, assertions 120, rc 1, last started TEST_RAISE_USER, failing [TEST_RAISE_INSERT_CRLF TEST_RAISE_PROPAGATE TEST_RAISE_SYNTAX_EXIT_02 TEST_RAISE_SYNTAX_RETURN_02]",
                "TEST_RAISE_INSERT_CRLF as alone, and tests outside the derived list that fail alone too (whole-groups/alone.txt)",
            ),
            (
                "base/keyword/RAISE.testGroup",
                "whole",
                "every",
                "failure, assertions 120, rc 1, last started TEST_RAISE_USER, failing [TEST_RAISE_INSERT_CRLF TEST_RAISE_PROPAGATE TEST_RAISE_SYNTAX_EXIT_02 TEST_RAISE_SYNTAX_RETURN_02]",
                "TEST_RAISE_INSERT_CRLF as alone, and tests outside the derived list that fail alone too (whole-groups/alone.txt)",
            ),
            (
                "base/keyword/RAISE.testGroup",
                "derived",
                "normal",
                "failure, assertions 0, rc 1, last started TEST_RAISE_INSERT_CRLF, failing [TEST_RAISE_INSERT_CRLF]",
                "TEST_RAISE_INSERT_CRLF, as alone: message text conversion, outside Phase 6",
            ),
            (
                "base/keyword/RAISE.testGroup",
                "derived",
                "every",
                "failure, assertions 0, rc 1, last started TEST_RAISE_INSERT_CRLF, failing [TEST_RAISE_INSERT_CRLF]",
                "TEST_RAISE_INSERT_CRLF, as alone: message text conversion, outside Phase 6",
            ),
            (
                "base/keyword/REPLY.testGroup",
                "whole",
                "every",
                "pass, assertions 19, rc 0, last started TEST_REPLY__CODE_RETURN, failing []",
                "every continuation runs before the program ends; the oracle races them with its end (P41, phase-4-exclusions.txt row 22)",
            ),
            (
                "base/keyword/REPLY.testGroup",
                "derived",
                "every",
                "pass, assertions 18, rc 0, last started TEST_REPLY__CODE_RETURN, failing []",
                "every continuation runs before the program ends; the oracle races them with its end (P41, phase-4-exclusions.txt row 22)",
            ),
            (
                "base/keyword/TRACE.testGroup",
                "whole",
                "normal",
                "error, assertions 129, rc 2, last started TEST_TRACE_VALUE_NUMERIC_INVALID, failing [TEST_TRACE_DROP TEST_TRACE_EXIT TEST_TRACE_EXPOSE TEST_TRACE_IGNORED TEST_TRACE_LABEL_WITH_FORWARD TEST_TRACE_OTHER_ENTRYPOINT TEST_TRACE_PROCEDURE]",
                "tests outside the derived list that fail alone too (whole-groups/alone.txt); TEST_TRACE_LABEL_WITH_FORWARD's FORWARD/REPLY trace lines, measured alone 2026-10-07",
            ),
            (
                "base/keyword/TRACE.testGroup",
                "whole",
                "every",
                "error, assertions 129, rc 2, last started TEST_TRACE_VALUE_NUMERIC_INVALID, failing [TEST_TRACE_DROP TEST_TRACE_EXIT TEST_TRACE_EXPOSE TEST_TRACE_IGNORED TEST_TRACE_LABEL_WITH_FORWARD TEST_TRACE_OTHER_ENTRYPOINT TEST_TRACE_PROCEDURE]",
                "tests outside the derived list that fail alone too (whole-groups/alone.txt); TEST_TRACE_LABEL_WITH_FORWARD's FORWARD/REPLY trace lines, measured alone 2026-10-07",
            ),
            (
                "base/keyword/TRACE_TraceObject.testGroup",
                "whole",
                "normal",
                "error, assertions 18, rc 2, last started TEST_VARIABLE, failing [TEST_CALLER_STACK_FRAME_REPLY_START TEST_OBJECT_AND_SCOPE TEST_VARIABLE TEST_CALLER_STACK_FRAME TEST_TRACEOBJECT_COLLECTOR]",
                "tests that fail alone too, measured 2026-10-07: TraceObject VARIABLE, CALLERSTACKFRAME and collected trace lines; TEST_OBJECT_AND_SCOPE's collector receives no trace lines (0 against 19)",
            ),
            (
                "base/keyword/TRACE_TraceObject.testGroup",
                "whole",
                "every",
                "error, assertions 18, rc 2, last started TEST_VARIABLE, failing [TEST_CALLER_STACK_FRAME_REPLY_START TEST_OBJECT_AND_SCOPE TEST_VARIABLE TEST_CALLER_STACK_FRAME TEST_TRACEOBJECT_COLLECTOR]",
                "tests that fail alone too, measured 2026-10-07: TraceObject VARIABLE, CALLERSTACKFRAME and collected trace lines; TEST_OBJECT_AND_SCOPE's collector receives no trace lines (0 against 19)",
            ),
            (
                "base/keyword/TRACE_TraceObject.testGroup",
                "derived",
                "normal",
                "error, assertions 2, rc 2, last started TEST_TRACEOBJECT_COLLECTOR, failing [TEST_CALLER_STACK_FRAME_REPLY_START TEST_TRACEOBJECT_COLLECTOR]",
                "tests that fail alone too, measured 2026-10-07: TraceObject CALLERSTACKFRAME THREAD entries and collected trace lines",
            ),
            (
                "base/keyword/TRACE_TraceObject.testGroup",
                "derived",
                "every",
                "error, assertions 2, rc 2, last started TEST_TRACEOBJECT_COLLECTOR, failing [TEST_CALLER_STACK_FRAME_REPLY_START TEST_TRACEOBJECT_COLLECTOR]",
                "tests that fail alone too, measured 2026-10-07: TraceObject CALLERSTACKFRAME THREAD entries and collected trace lines",
            ),
            (
                "doc/rexxref/chapter5/Section1.testGroup",
                "whole",
                "normal",
                "rexx-exec: method \"OBJECTNAME=\" of class \"Object\" is not implemented (Phase 9), rc 120, last started TEST_OBJECT_OBJECTNAMEEQUALS, failing []",
                "a test outside the derived list refuses: TEST_OBJECT_OBJECTNAMEEQUALS, Phase 9",
            ),
            (
                "doc/rexxref/chapter5/Section1.testGroup",
                "whole",
                "every",
                "rexx-exec: method \"OBJECTNAME=\" of class \"Object\" is not implemented (Phase 9), rc 120, last started TEST_OBJECT_OBJECTNAMEEQUALS, failing []",
                "a test outside the derived list refuses: TEST_OBJECT_OBJECTNAMEEQUALS, Phase 9",
            ),
        ];

        /// The line `TestCase~execute` starts each test with, in the copy's
        /// `OOREXXUNIT.CLS`, and what it becomes: the same line, which also
        /// writes `started NAME` to stderr, so line numbers are unchanged.
        const START_LINE: &str = "  aTestResult~startTest(self)       -- remember test started\n  self~setUp                        -- make sure setup is invoked before test\n";
        const START_MARKED: &str = "  aTestResult~startTest(self); .error~say('started' fName) -- remember test started\n  self~setUp                        -- make sure setup is invoked before test\n";

        fn mark_starts(at: &Path) {
            let file = at.join("framework/OOREXXUNIT.CLS");
            let text = read_lossy(&file);
            assert_eq!(
                text.matches(START_LINE).count(),
                1,
                "one test start in {}",
                file.display()
            );
            std::fs::write(&file, text.replacen(START_LINE, START_MARKED, 1))
                .expect("cannot rewrite the copied OOREXXUNIT.CLS");
        }

        /// The tests `run` started, in order.
        fn started(run: &Run) -> Vec<String> {
            String::from_utf8_lossy(&run.stderr)
                .lines()
                .filter_map(|line| line.strip_prefix("started "))
                .map(str::to_ascii_uppercase)
                .collect()
        }

        /// The tests a run's `-V 2` detail names as failing or in error, in
        /// order.
        fn failing(run: &Run) -> Vec<String> {
            let text = String::from_utf8_lossy(&run.stdout);
            let mut names = Vec::new();
            let mut detail = false;
            for line in text.lines() {
                if line.starts_with("[failure] ") || line.starts_with("[error] ") {
                    detail = true;
                } else if let Some(name) =
                    line.trim_start().strip_prefix("Test:").filter(|_| detail)
                {
                    names.push(name.trim().to_string());
                    detail = false;
                }
            }
            names
        }

        fn refusal(run: &Run) -> Option<String> {
            String::from_utf8_lossy(&run.stderr)
                .lines()
                .find(|line| line.starts_with("rexx-exec: "))
                .map(str::to_string)
        }

        /// A run's refusal or summary, its status, the last test it started
        /// and the tests failing in it.
        fn key(run: &Run) -> String {
            let status = run
                .status
                .map_or_else(|| "none".to_string(), |status| status.to_string());
            let last = started(run).pop().unwrap_or_else(|| "-".to_string());
            format!(
                "{}, rc {status}, last started {last}, failing [{}]",
                refusal(run).unwrap_or_else(|| outcome(&run.stdout)),
                failing(run).join(" ")
            )
        }

        /// `run` with its summary's assertion count masked.
        fn count_masked(run: &Run) -> Run {
            let text = String::from_utf8_lossy(&run.stdout).into_owned();
            let stdout: Vec<&str> = text
                .split('\n')
                .map(|line| {
                    if line.starts_with("Assertions:") {
                        "Assertions:"
                    } else {
                        line
                    }
                })
                .collect();
            Run {
                stdout: stdout.join("\n").into_bytes(),
                stderr: run.stderr.clone(),
                status: run.status,
            }
        }

        /// Whether the oracle's distinct outcomes `seen` differ from each other
        /// only in their assertion count (ruling P86).
        fn counted_only(seen: &[&Run]) -> bool {
            seen.len() > 1
                && seen
                    .iter()
                    .all(|run| agree(&count_masked(run), &count_masked(seen[0])))
        }

        /// `run` with the copy's path, and the copy's directory name in a
        /// path the framework abbreviates, replaced by `RUN`, so runs from
        /// different copies of one row compare.
        fn relative(run: Run, at: &Path) -> Run {
            let name = at.file_name().expect("a copy name").to_string_lossy();
            let patterns = [
                at.to_string_lossy().into_owned().into_bytes(),
                format!("/run/{name}/").into_bytes(),
            ];
            let replace = |bytes: Vec<u8>| {
                patterns.iter().fold(bytes, |bytes, pattern| {
                    let with: &[u8] = if pattern.starts_with(b"/run/") {
                        b"/run/RUN/"
                    } else {
                        b"RUN"
                    };
                    let mut out = Vec::with_capacity(bytes.len());
                    let mut rest = bytes.as_slice();
                    while !rest.is_empty() {
                        if rest.starts_with(pattern) {
                            out.extend_from_slice(with);
                            rest = &rest[pattern.len()..];
                        } else {
                            out.push(rest[0]);
                            rest = &rest[1..];
                        }
                    }
                    out
                })
            };
            Run {
                stdout: replace(run.stdout),
                stderr: replace(run.stderr),
                status: run.status,
            }
        }

        /// The copy `name` below `run` with the tests of `left_out` renamed
        /// out and each test's start marked, and the group file's path in it.
        fn copy(
            run: &Path,
            name: &str,
            dir: &str,
            group: &str,
            left_out: &BTreeSet<String>,
        ) -> (PathBuf, String) {
            let at = run.join(name);
            fresh_copy(&at, dir);
            mark_starts(&at);
            let file = group_file(&at, dir, group);
            skip(Path::new(&file), group, left_out);
            (at, file)
        }

        /// Runs `body` over `items` with at most [`RUNS_AT_ONCE`] at once,
        /// answering the results in order.
        fn at_once<I: Sync, T: Send>(items: &[I], body: impl Fn(&I) -> T + Sync) -> Vec<T> {
            let mut results = Vec::new();
            for batch in items.chunks(RUNS_AT_ONCE) {
                std::thread::scope(|scope| {
                    let handles: Vec<_> = batch
                        .iter()
                        .map(|item| scope.spawn(|| body(item)))
                        .collect();
                    results.extend(handles.into_iter().map(|handle| {
                        handle
                            .join()
                            .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
                    }));
                });
            }
            results
        }

        /// The distinct outcomes of `runs`, each with its count, most
        /// frequent first.
        fn distribution(runs: &[(Run, Duration)]) -> Vec<(&Run, usize)> {
            let mut distinct: Vec<(&Run, usize)> = Vec::new();
            for (run, _) in runs {
                match distinct.iter_mut().find(|(seen, _)| agree(seen, run)) {
                    Some((_, count)) => *count += 1,
                    None => distinct.push((run, 1)),
                }
            }
            distinct.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
            distinct
        }

        fn difference(theirs: &Run, ours: &Run) -> String {
            first_difference(&masked(&theirs.stdout), &masked(&ours.stdout))
                .replace(['\n', '\t'], " ")
        }

        /// One row's result: the table's cells, the tests each of our runs
        /// started, whether each mode agrees with the oracle, and whether
        /// each ended in an inverted wait or did not finish.
        struct Row {
            part: Part,
            cells: String,
            started: String,
            keys: [String; 2],
            agrees: [bool; 2],
            stuck: [bool; 2],
        }

        fn dump(file: &str, part: Part, side: &str, run: &Run) {
            let Some(dir) = std::env::var_os(DUMP_ENV) else {
                return;
            };
            let stem = format!("{}-{}-{side}", file.replace('/', "__"), part.label());
            let dir = Path::new(&dir);
            std::fs::create_dir_all(dir).expect("the dump directory");
            std::fs::write(dir.join(format!("{stem}.out")), &run.stdout).expect("a dump");
            std::fs::write(dir.join(format!("{stem}.err")), &run.stderr).expect("a dump");
        }

        /// The group file's directory below `ootest/ooRexx` and its name.
        fn split(file: &str) -> (&str, &str) {
            file.trim_end_matches(".testGroup")
                .rsplit_once('/')
                .expect("a group below a directory")
        }

        /// The rows of `file`'s `part`: the whole part also answers its rest
        /// row when its run here refuses.
        fn rows_of(
            oracle: &oracle::Oracle,
            run: &Path,
            file: &str,
            part: Part,
            derived: &BTreeSet<String>,
        ) -> Vec<Row> {
            let (dir, group) = split(file);
            let mut left_out = left_out_of(file, part, derived);
            let (row, ours) = checked(oracle, run, file, part, &left_out, &[]);
            let mut rows = vec![row];
            if part == Part::Whole {
                let refused = |run: &Run| {
                    refusal(run).filter(|line| !line.ends_with("the run exceeded its deadline"))
                };
                let mut refusing = Vec::new();
                let mut last = ours;
                let mut before = false;
                while let Some(line) = refused(&last) {
                    let Some(test) = started(&last).pop() else {
                        rows[0]
                            .started
                            .push_str(&format!("\trest not run: {line} before any test started"));
                        before = true;
                        break;
                    };
                    assert!(!refusing.contains(&test), "{file}: {test} refused twice");
                    left_out.insert(format!("{group}.{test}"));
                    refusing.push(test);
                    let (at, path) = copy(run, "r00", dir, group, &left_out);
                    let args = ["-f", path.as_str(), "-U", "-V", VERBOSITY];
                    last = relative(
                        run_crate_within(&at, &args, SwitchMode::None, ORACLE_DEADLINE),
                        &at,
                    );
                }
                if !refusing.is_empty() && !before {
                    rows.push(checked(oracle, run, file, Part::Rest, &left_out, &refusing).0);
                }
            }
            rows
        }

        /// The `GROUP.TEST`s every run of `file`'s `part` renames out: those
        /// reaching rxapi, [`REST_LEFT_OUT`]'s, and for the derived part each
        /// test `derived` (the derived list's tests of the group) does not
        /// name.
        fn left_out_of(file: &str, part: Part, derived: &BTreeSet<String>) -> BTreeSet<String> {
            let (dir, group) = split(file);
            let mut left_out = reaching_rxapi(dir, &[group]);
            for (_, test, _) in REST_LEFT_OUT.iter().filter(|(listed, ..)| *listed == file) {
                left_out.insert(format!("{group}.{test}"));
            }
            if part == Part::Derived {
                let path = super::super::worktree()
                    .join("ootest/ooRexx")
                    .join(dir)
                    .join(format!("{group}.testGroup"));
                left_out.extend(
                    super::super::units(&read_lossy(&path))
                        .into_iter()
                        .filter(|unit| unit.test)
                        .map(|unit| unit.name.to_ascii_uppercase())
                        .filter(|test| !derived.contains(test))
                        .map(|test| format!("{group}.{test}")),
                );
            }
            left_out
        }

        /// What one mode of `row` amounts to against the oracle and
        /// [`DIFFERING`]: `Ok` when allowed, else why not.
        fn verdict(file: &str, row: &Row, at: usize) -> Result<(), String> {
            let mode = ["normal", "every"][at];
            let listed = DIFFERING.iter().find(|(group, part, listed_mode, ..)| {
                *group == file && *part == row.part.label() && *listed_mode == mode
            });
            match listed {
                Some(_) if row.agrees[at] => Ok(()),
                Some((.., key, _)) if row.keys[at] != *key => {
                    Err(format!("listed as {key:?}, is {:?}", row.keys[at]))
                }
                Some(_) => Ok(()),
                None if row.stuck[at] => Err("an inverted wait or a hang".to_string()),
                None if !row.agrees[at] => {
                    Err(format!("not an oracle outcome: {:?}", row.keys[at]))
                }
                None => Ok(()),
            }
        }

        /// [`one_row`], run again once when a mode is not allowed and the
        /// group holds a row whose outcome depends on the wall clock (ruling
        /// P48).
        fn checked(
            oracle: &oracle::Oracle,
            run: &Path,
            file: &str,
            part: Part,
            left_out: &BTreeSet<String>,
            refusing: &[String],
        ) -> (Row, Run) {
            let first = one_row(oracle, run, file, part, left_out, refusing);
            let timed = WALL_CLOCK
                .iter()
                .any(|row| row.starts_with(&format!("{file} ")));
            if !timed || (0..2).all(|at| verdict(file, &first.0, at).is_ok()) {
                return first;
            }
            eprintln!("P48 rerun: {file} {}", part.label());
            let (mut row, normal) = one_row(oracle, run, file, part, left_out, refusing);
            row.cells.push_str(" (P48 rerun)");
            (row, normal)
        }

        /// One row: `file` with the tests of `left_out` renamed out, on the
        /// oracle and here in both modes; also answers the normal run here.
        fn one_row(
            oracle: &oracle::Oracle,
            run: &Path,
            file: &str,
            part: Part,
            left_out: &BTreeSet<String>,
            refusing: &[String],
        ) -> (Row, Run) {
            let (dir, group) = split(file);
            let oracle_run = |k: &usize| {
                let (at, path) = copy(run, &format!("o{k:02}"), dir, group, left_out);
                let args = ["-f", path.as_str(), "-U", "-V", VERBOSITY];
                let start = Instant::now();
                let theirs = run_oracle_within(oracle, &at, &args, ORACLE_DEADLINE);
                (relative(theirs, &at), start.elapsed())
            };
            let first: Vec<usize> = (0..ORACLE_RUNS).collect();
            let mut theirs = at_once(&first, oracle_run);
            let slowest = theirs.iter().map(|(_, took)| *took).max().expect("a run");
            let deadline = (slowest * CRATE_FACTOR).max(CRATE_FLOOR);
            let modes = [
                ("n00", SwitchMode::None),
                ("e00", SwitchMode::EveryOpportunity),
            ];
            let mut ours = at_once(&modes, |(name, mode)| {
                let (at, path) = copy(run, name, dir, group, left_out);
                let args = ["-f", path.as_str(), "-U", "-V", VERBOSITY];
                relative(run_crate_within(&at, &args, mode.clone(), deadline), &at)
            });
            let settled = {
                let seen = distribution(&theirs);
                seen.len() == 1 && ours.iter().all(|run| agree(seen[0].0, run))
            };
            if !settled {
                let more: Vec<usize> = (ORACLE_RUNS..ORACLE_RUNS_UNSETTLED).collect();
                theirs.extend(at_once(&more, oracle_run));
            }
            let seen = distribution(&theirs);
            dump(file, part, "oracle", seen[0].0);
            dump(file, part, "normal", &ours[0]);
            dump(file, part, "every", &ours[1]);
            let mut cells = format!("{file}\t{}\t{}\t", part.label(), theirs.len());
            for (at, (run, count)) in seen.iter().enumerate() {
                if at > 0 {
                    cells.push_str("; ");
                }
                cells.push_str(&format!("{}: {count} {}", at + 1, key(run)));
            }
            let mut started_cells =
                format!("{file}\t{}\tskipped [{}]", part.label(), refusing.join(" "));
            // Ruling P86: where the oracle's runs differ from each other only
            // in their assertion count, the count is not compared.
            let counted_only = counted_only(&seen.iter().map(|(run, _)| *run).collect::<Vec<_>>());
            let mut keys = [String::new(), String::new()];
            let mut agrees = [false; 2];
            let mut stuck = [false; 2];
            for (at, run) in ours.iter().enumerate() {
                stuck[at] = inverted(run) || run.status.is_none();
                keys[at] = key(run);
                let cell = match seen.iter().position(|(theirs, _)| agree(theirs, run)) {
                    Some(number) => {
                        agrees[at] = true;
                        format!("agrees with {}", number + 1)
                    }
                    None if counted_only && agree(&count_masked(seen[0].0), &count_masked(run)) => {
                        agrees[at] = true;
                        format!("agrees but for the assertion count (P86): {}", keys[at])
                    }
                    None => format!("differs: {}; {}", keys[at], difference(seen[0].0, run)),
                };
                cells.push_str(&format!("\t{cell}"));
                let names = started(run);
                started_cells.push_str(&format!("\t{} [{}]", names.len(), names.join(" ")));
            }
            let same = if agree(&ours[0], &ours[1]) {
                "same"
            } else {
                "differ"
            };
            cells.push_str(&format!("\t{same}"));
            let normal = ours.swap_remove(0);
            let row = Row {
                part,
                cells,
                started: started_cells,
                keys,
                agrees,
                stuck,
            };
            (row, normal)
        }

        fn summary_run(assertions: u32, failures: u32, stderr: &str, status: i32) -> Run {
            Run {
                stdout: format!(
                    "Tests ran:          1\nAssertions:         {assertions}\nFailures:           {failures}\nErrors:             0\n"
                )
                .into_bytes(),
                stderr: stderr.as_bytes().to_vec(),
                status: Some(status),
            }
        }

        #[test]
        fn a_run_agrees_but_for_the_count_only_when_the_rest_agrees() {
            let theirs = summary_run(12, 0, "", 0);
            assert!(agree(
                &count_masked(&theirs),
                &count_masked(&summary_run(10, 0, "", 0))
            ));
            assert!(!agree(
                &count_masked(&theirs),
                &count_masked(&summary_run(10, 0, "noise\n", 0))
            ));
            assert!(!agree(
                &count_masked(&theirs),
                &count_masked(&summary_run(10, 1, "", 1))
            ));
        }

        #[test]
        fn one_oracle_outcome_is_not_counted_only() {
            assert!(!counted_only(&[&summary_run(12, 0, "", 0)]));
        }

        #[test]
        fn outcomes_differing_only_in_the_count_are_counted_only() {
            assert!(counted_only(&[
                &summary_run(12, 0, "", 0),
                &summary_run(17, 0, "", 0)
            ]));
        }

        #[test]
        fn outcomes_differing_in_stderr_alone_are_not_counted_only() {
            assert!(!counted_only(&[
                &summary_run(12, 0, "", 0),
                &summary_run(12, 0, "noise\n", 0)
            ]));
        }

        #[test]
        fn outcomes_differing_in_count_and_status_are_not_counted_only() {
            assert!(!counted_only(&[
                &summary_run(12, 0, "", 0),
                &summary_run(17, 1, "", 1)
            ]));
        }

        #[test]
        fn each_group_of_the_derived_list_in_one_run_in_both_modes() {
            if !gate_mode() {
                eprintln!("group_runs: skipped without {GATE_ENV}");
                return;
            }
            let (list, _) = super::super::derive(&super::super::worktree().join("ootest/ooRexx"));
            let derived: BTreeSet<String> = list
                .iter()
                .map(|row| {
                    let group = row.group.trim_end_matches(".testGroup");
                    let group = group.rsplit_once('/').map_or(group, |(_, name)| name);
                    format!("{group}.{}", row.test)
                })
                .collect();
            let mut files: Vec<String> = list.iter().map(|row| row.group.clone()).collect();
            files.dedup();
            let parts: Vec<(&String, Part)> = files
                .iter()
                .flat_map(|file| [(file, Part::Whole), (file, Part::Derived)])
                .collect();
            let oracle = oracle::locate();
            let results = rows_in_parallel(
                "whole-groups",
                &parts,
                |(file, _)| {
                    WALL_CLOCK
                        .iter()
                        .any(|row| row.starts_with(&format!("{file} ")))
                },
                |run, (file, part)| {
                    let (_, group) = split(file);
                    let mine: BTreeSet<String> = derived
                        .iter()
                        .filter_map(|row| row.strip_prefix(&format!("{group}.")))
                        .map(str::to_string)
                        .collect();
                    rows_of(&oracle, run, file, *part, &mine)
                },
            );
            let mut table = String::from(
                "group\tpart\toracle runs\toracle outcomes\tnormal\tevery\tnormal against every\n",
            );
            let mut started = String::from("group\tpart\tskipped\tnormal started\tevery started\n");
            let mut failing = Vec::new();
            let mut listed_agreeing = Vec::new();
            for ((file, _), rows) in parts.iter().zip(&results) {
                for row in rows {
                    table.push_str(&row.cells);
                    table.push('\n');
                    started.push_str(&row.started);
                    started.push('\n');
                    for (at, mode) in ["normal", "every"].into_iter().enumerate() {
                        let name = format!("{file} {} {mode}", row.part.label());
                        let listed = DIFFERING.iter().any(|(group, part, listed_mode, ..)| {
                            group == file && *part == row.part.label() && *listed_mode == mode
                        });
                        if listed && row.agrees[at] {
                            listed_agreeing.push(name.clone());
                        }
                        if let Err(why) = verdict(file, row, at) {
                            failing.push(format!("{name}: {why}"));
                        }
                    }
                }
            }
            table.push_str(&format!(
                "\nlisted as differing, agreeing: {listed_agreeing:?}\n"
            ));
            eprintln!("{table}\n{started}");
            if let Some(path) = std::env::var_os(TABLE_ENV) {
                std::fs::write(path, &table).expect("cannot write the table");
            }
            if let Some(path) = std::env::var_os(STARTED_ENV) {
                std::fs::write(path, &started).expect("cannot write the started table");
            }
            assert!(failing.is_empty(), "{failing:#?}");
        }

        /// The seeded gate (spec 2026-10-07 section 4, "Gate"): each part of
        /// `corpus/sim-gate.tsv` in the simulation mode under the seeds its row
        /// derives, judged by the crate's own invariants and, but under `pct`
        /// and in the rows injecting failures, by the program's own checks
        /// (ruling R6). An outcome no committed oracle outcome of its part has
        /// goes to the report, never red.
        mod sim_gate {
            use std::collections::{BTreeMap, BTreeSet};
            use std::fmt::Write as _;
            use std::path::{Path, PathBuf};
            use std::sync::Mutex;
            use std::time::Duration;

            use super::super::super::group_runner::{
                Ended, GATE_ENV, ProcessRun, Run, VERBOSITY, gate_mode, masked, run_crate_process,
                run_crate_sim, run_oracle_within,
            };
            use super::super::super::support::{oracle, sidecar};
            use super::{
                DIFFERING, ORACLE_DEADLINE, ORACLE_RUNS, ORACLE_RUNS_UNSETTLED, Part, WALL_CLOCK,
                at_once, copy, failing, key, left_out_of, refusal, relative, split, started,
            };

            /// The gate's parts, relative to `rust/`.
            const TABLE: &str = "corpus/sim-gate.tsv";

            /// The reds ruled not to be defects, relative to `rust/`.
            const EXEMPT: &str = "corpus/sim-exempt.tsv";

            /// The committed oracle outcome sets, one file per part, relative
            /// to `rust/`.
            const ORACLE_DIR: &str = "corpus/sim-oracle";

            /// The seed count of every row, in place of the table's.
            const SEEDS_ENV: &str = "REXX_SIM_SEEDS";

            /// The policy of every seed, in place of the mix; `pre:D` and
            /// `pct:D` take the row's k.
            const POLICY_ENV: &str = "REXX_SIM_POLICY";

            /// One run, `GROUP:PART:SEED:POLICY`, in place of the gate's.
            const ONLY_ENV: &str = "REXX_SIM_ONLY";

            /// Set to `1`, the oracle sets are run and grown.
            const REFRESH_ENV: &str = "REXX_SIM_ORACLE_REFRESH";

            /// Where the run's report is written; else below the target's
            /// temporary directory.
            const REPORT_ENV: &str = "REXX_SIM_REPORT";

            /// How many runs go at once; [`JOBS_DEFAULT`] where unset.
            const JOBS_ENV: &str = "REXX_SIM_JOBS";
            const JOBS_DEFAULT: usize = 8;

            /// Where the calibration writes its rows; the calibration runs
            /// only where it is set.
            const CALIBRATION_ENV: &str = "REXX_SIM_CALIBRATION";

            /// The policy of seed i is entry i mod 7.
            const MIX: [&str; 7] = [
                "pre:1",
                "pre:2",
                "pre:3",
                "pct:3",
                "uniform:0.01",
                "uniform:0.2",
                "uniform:1",
            ];

            /// A run's deadline: its part's calibrated time in this build
            /// times [`DEADLINE_FACTOR`], and at least [`DEADLINE_FLOOR`].
            const DEADLINE_FACTOR: u32 = 10;
            const DEADLINE_FLOOR: Duration = Duration::from_secs(60);

            /// The calibration's bound on one run.
            const CALIBRATION_DEADLINE: Duration = Duration::from_secs(600);

            /// How a gate program's assertion line starts.
            const ASSERTION: &str = "FAIL";

            /// The directory of the gate's own programs, relative to `rust/`.
            const PROGRAMS: &str = "crates/rexx-exec/tests/sim_gate/";

            /// The refusals naming a broken invariant of the scheduler.
            const INVARIANT: &str = "rexx-exec: the scheduler found ";

            /// The refusals of a determinism breach: a post from another
            /// thread, and a replay taking a decision its trace does not hold.
            const DETERMINISM: &[&str] = &[
                "from another thread in the simulation mode is not implemented",
                "rexx-exec: the replay of ",
            ];

            /// The design-limit refusals, red unless the part's oracle set
            /// holds a run that did not end.
            const DESIGN_LIMIT: &[&str] = &[
                "that only an activity pinned below it can end",
                "a REPLY its method body runs on a nested Rust frame",
                "a wait that nothing left to run can end",
                "a wait in the simulation mode that only a signal can end",
                "a command in the simulation mode that waits longer than its bound",
                "a native call in the simulation mode that runs longer than its bound",
            ];

            fn rust_root() -> PathBuf {
                Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
            }

            /// FNV-1a 64 over `bytes`.
            fn fnv(bytes: &[u8]) -> u64 {
                bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
                    (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
                })
            }

            /// Output `i` (from 0) of the splitmix64 stream starting at `state`.
            fn splitmix(state: u64, i: u64) -> u64 {
                let mut z = state.wrapping_add((i + 1).wrapping_mul(0x9e37_79b9_7f4a_7c15));
                z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
                z ^ (z >> 31)
            }

            /// Seed `i` of a part: `splitmix(hash(group, part), i)`, the hash
            /// FNV-1a over `GROUP:PART`.
            fn seed(group: &str, part: &str, i: u64) -> u64 {
                splitmix(fnv(format!("{group}:{part}").as_bytes()), i)
            }

            /// `mix` with the part's `k` where the policy takes one.
            fn with_k(mix: &str, k: u64) -> String {
                if (mix.starts_with("pre:") || mix.starts_with("pct:")) && !mix.contains(",k=") {
                    format!("{mix},k={}", k.max(1))
                } else {
                    mix.to_string()
                }
            }

            /// One row of [`TABLE`].
            #[derive(Clone, Debug)]
            struct Row {
                /// An ooTest group file below `ootest/ooRexx`, or a program
                /// relative to `rust/`.
                group: String,
                /// `whole`, `derived`, `rest`, `single` (the one test `left_out`
                /// names, alone) or `program`.
                part: String,
                /// The contended steps of the calibration run.
                k: u64,
                /// Seeds in a release build and in a debug build; 0 leaves the
                /// row out of that build's gate.
                release_seeds: u64,
                debug_seeds: u64,
                /// The calibration run's wall time in each build.
                release_ms: u64,
                debug_ms: u64,
                /// Knobs appended to every run's mode.
                knobs: Option<String>,
                /// The tests left out beside the part's own; a single row's one
                /// test.
                left_out: Vec<String>,
            }

            impl Row {
                fn program(&self) -> bool {
                    self.part == "program"
                }

                /// Whether the row's knobs inject failures or halts
                /// (`fail=wait:K`, `halt@K`): its program's own checks are not
                /// judged and its outcomes are not compared with the oracle.
                fn injects(&self) -> bool {
                    self.knobs
                        .as_deref()
                        .is_some_and(|knobs| knobs.contains("fail=") || knobs.contains("halt@"))
                }

                fn seeds(&self) -> u64 {
                    if let Ok(count) = std::env::var(SEEDS_ENV) {
                        return count
                            .parse()
                            .unwrap_or_else(|_| panic!("{SEEDS_ENV} is not a count: {count:?}"));
                    }
                    if cfg!(debug_assertions) {
                        self.debug_seeds
                    } else {
                        self.release_seeds
                    }
                }

                fn deadline(&self) -> Duration {
                    let ms = if cfg!(debug_assertions) {
                        self.debug_ms
                    } else {
                        self.release_ms
                    };
                    (Duration::from_millis(ms) * DEADLINE_FACTOR).max(DEADLINE_FLOOR)
                }

                /// The policy of seed `i`.
                fn policy(&self, i: u64) -> String {
                    let mix = std::env::var(POLICY_ENV).unwrap_or_else(|_| {
                        MIX[usize::try_from(i % 7).expect("below 7")].to_string()
                    });
                    with_k(&mix, self.k)
                }

                fn mode(&self, seed: u64, policy: &str) -> String {
                    let mut mode = format!("sim:{seed},{policy}");
                    if let Some(knobs) = &self.knobs {
                        mode.push(',');
                        mode.push_str(knobs);
                    }
                    mode
                }

                fn oracle_file(&self) -> PathBuf {
                    rust_root().join(ORACLE_DIR).join(format!(
                        "{}.{}.tsv",
                        self.group.replace('/', "__"),
                        self.part
                    ))
                }
            }

            fn read_table() -> Vec<Row> {
                let path = rust_root().join(TABLE);
                let text = std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
                text.lines()
                    .filter(|line| !line.is_empty() && !line.starts_with('#'))
                    .map(|line| {
                        let cells: Vec<&str> = line.split('\t').collect();
                        assert_eq!(cells.len(), 9, "{TABLE}: {line:?} is not nine cells");
                        let number = |at: usize| -> u64 {
                            cells[at]
                                .parse()
                                .unwrap_or_else(|_| panic!("{TABLE}: {line:?} cell {at}"))
                        };
                        Row {
                            group: cells[0].to_string(),
                            part: cells[1].to_string(),
                            k: number(2),
                            release_seeds: number(3),
                            debug_seeds: number(4),
                            release_ms: number(5),
                            debug_ms: number(6),
                            knobs: (cells[7] != "-").then(|| cells[7].to_string()),
                            left_out: if cells[8] == "-" {
                                Vec::new()
                            } else {
                                cells[8].split(' ').map(str::to_string).collect()
                            },
                        }
                    })
                    .collect()
            }

            /// One run of the gate.
            #[derive(Clone)]
            struct Unit {
                row: Row,
                seed: u64,
                policy: String,
            }

            impl Unit {
                fn mode(&self) -> String {
                    self.row.mode(self.seed, &self.policy)
                }

                fn name(&self) -> String {
                    format!(
                        "{}:{}:{}:{}",
                        self.row.group, self.row.part, self.seed, self.policy
                    )
                }

                /// The command that runs this unit alone.
                fn replay(&self) -> String {
                    let profile = if cfg!(debug_assertions) {
                        ("", "debug")
                    } else {
                        (" --release", "release")
                    };
                    format!(
                        "REXX_CORPUS_GATE=1 {ONLY_ENV}='{}' memcap 8G cargo test -j 4{} -p \
                         rexx-exec --test concurrency_tests the_seeded_gate -- --nocapture \
                         (profile={}, stack={}, REXX_SWITCH_MODE={})",
                        self.name(),
                        profile.0,
                        profile.1,
                        rexx_exec::INTERPRETER_STACK_BYTES,
                        self.mode()
                    )
                }
            }

            /// The gate's units: each row's seeds, or where [`ONLY_ENV`] is
            /// set, `GROUP` or `GROUP:PART` its rows' seeds and
            /// `GROUP:PART:SEED:POLICY` that one run.
            fn units(rows: &[Row]) -> Vec<Unit> {
                let only = std::env::var(ONLY_ENV).ok();
                let cells: Vec<&str> = only
                    .as_deref()
                    .map_or_else(Vec::new, |only| only.splitn(4, ':').collect());
                let chosen = |row: &&Row| match cells[..] {
                    [] => true,
                    [group] => row.group == group,
                    [group, part, ..] => row.group == group && row.part == part,
                };
                if let [_, _, seed, policy] = cells[..] {
                    let row = rows
                        .iter()
                        .find(chosen)
                        .unwrap_or_else(|| panic!("{ONLY_ENV}: no row {only:?}"));
                    return vec![Unit {
                        row: row.clone(),
                        seed: seed
                            .parse()
                            .unwrap_or_else(|_| panic!("{ONLY_ENV}: {seed:?} is not a seed")),
                        policy: policy.to_string(),
                    }];
                }
                assert!(
                    cells.len() < 3,
                    "{ONLY_ENV} is GROUP, GROUP:PART or GROUP:PART:SEED:POLICY, not {only:?}"
                );
                rows.iter()
                    .filter(chosen)
                    .flat_map(|row| {
                        (0..row.seeds()).map(move |i| Unit {
                            row: row.clone(),
                            seed: seed(&row.group, &row.part, i),
                            policy: row.policy(i),
                        })
                    })
                    .collect()
            }

            /// The derived list's tests of each group file.
            fn derived_tests() -> BTreeMap<String, BTreeSet<String>> {
                let (list, _) = super::super::super::derive(
                    &super::super::super::worktree().join("ootest/ooRexx"),
                );
                let mut tests: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
                for row in list {
                    tests.entry(row.group).or_default().insert(row.test);
                }
                tests
            }

            /// The `GROUP.TEST`s a run of `row` renames out.
            fn left_out(
                row: &Row,
                derived: &BTreeMap<String, BTreeSet<String>>,
            ) -> BTreeSet<String> {
                let (_, group) = split(&row.group);
                let part = match row.part.as_str() {
                    "derived" => Part::Derived,
                    "rest" => Part::Rest,
                    _ => Part::Whole,
                };
                let mine = derived.get(&row.group).cloned().unwrap_or_default();
                let mut left_out = left_out_of(&row.group, part, &mine);
                if row.part != "single" {
                    left_out.extend(row.left_out.iter().map(|test| format!("{group}.{test}")));
                }
                left_out
            }

            /// The driver's arguments for the group file `path`: a single
            /// row's run names its one test.
            fn arguments(row: &Row, path: &str) -> Vec<String> {
                let mut args: Vec<String> = ["-f", path, "-U", "-V", VERBOSITY]
                    .map(str::to_string)
                    .into();
                if row.part == "single" {
                    args.push("-t".to_string());
                    args.extend(row.left_out.iter().cloned());
                }
                args
            }

            /// What one run of a program part reads beside its text: its
            /// path, its run directory (laid out afresh) and its sidecar.
            fn program_run(
                row: &Row,
                scratch: &Path,
            ) -> (PathBuf, PathBuf, sidecar::Sidecar, Vec<(String, String)>) {
                let path = rust_root()
                    .join(&row.group)
                    .canonicalize()
                    .unwrap_or_else(|e| panic!("{}: {e}", row.group));
                let side = row
                    .group
                    .strip_prefix("corpus/")
                    .map(|rel| sidecar::sidecar_for(&rust_root().join("corpus"), rel))
                    .unwrap_or_default();
                let dir = scratch.join("p");
                let overrides = sidecar::resolved_environment(&side, &dir);
                let cwd = sidecar::prepare_run_directory(&dir, &side);
                (path, cwd, side, overrides)
            }

            /// One run here of `row` under `mode`, its copy below `scratch`.
            fn run_here(
                row: &Row,
                derived: &BTreeMap<String, BTreeSet<String>>,
                scratch: &Path,
                mode: &str,
                deadline: Duration,
            ) -> ProcessRun {
                if row.program() {
                    let (path, cwd, side, overrides) = program_run(row, scratch);
                    let mut environment: Vec<(String, String)> = std::env::vars()
                        .filter(|(name, _)| {
                            name != "REXX_SWITCH_MODE"
                                && !overrides.iter().any(|(over, _)| over == name)
                        })
                        .collect();
                    environment.extend(overrides);
                    let mut environment: Vec<(Vec<u8>, Vec<u8>)> = environment
                        .into_iter()
                        .map(|(name, value)| (name.into_bytes(), value.into_bytes()))
                        .collect();
                    environment.sort();
                    return run_crate_process(
                        &[path.to_string_lossy().into_owned()],
                        &cwd,
                        Some(environment),
                        mode,
                        side.stdin.as_deref(),
                        deadline,
                    );
                }
                let (dir, group) = split(&row.group);
                let (at, path) = copy(scratch, "s00", dir, group, &left_out(row, derived));
                let args = arguments(row, &path);
                let args: Vec<&str> = args.iter().map(String::as_str).collect();
                let mut ran = run_crate_sim(&at, &args, mode, deadline);
                ran.run = relative(ran.run, &at);
                ran
            }

            /// One oracle run of `row`, its copy below `scratch`.
            fn run_oracle(
                oracle: &oracle::Oracle,
                row: &Row,
                derived: &BTreeMap<String, BTreeSet<String>>,
                scratch: &Path,
            ) -> Run {
                if row.program() {
                    let (path, cwd, side, overrides) = program_run(row, scratch);
                    let borrowed: Vec<(&str, &str)> = overrides
                        .iter()
                        .map(|(name, value)| (name.as_str(), value.as_str()))
                        .collect();
                    let theirs = oracle.run_in_with(&path, &cwd, &borrowed, side.stdin.as_deref());
                    let status =
                        (!oracle::did_not_finish(&theirs)).then(|| theirs.expect_exit_code());
                    return Run {
                        stdout: theirs.stdout,
                        stderr: theirs.stderr,
                        status,
                    };
                }
                let (dir, group) = split(&row.group);
                let (at, path) = copy(scratch, "o00", dir, group, &left_out(row, derived));
                let args = arguments(row, &path);
                let args: Vec<&str> = args.iter().map(String::as_str).collect();
                relative(run_oracle_within(oracle, &at, &args, ORACLE_DEADLINE), &at)
            }

            /// `stderr` without the `rexx-sim: ` lines, a child's included.
            fn sim_masked(stderr: &[u8]) -> Vec<u8> {
                let text = String::from_utf8_lossy(stderr);
                text.split_inclusive('\n')
                    .filter(|line| !line.starts_with("rexx-sim: "))
                    .collect::<String>()
                    .into_bytes()
            }

            /// The run's own `rexx-sim: ` line, its last line.
            fn sim_line(stderr: &[u8]) -> Option<String> {
                String::from_utf8_lossy(stderr)
                    .lines()
                    .last()
                    .filter(|line| line.starts_with("rexx-sim: "))
                    .map(str::to_string)
            }

            /// The value of `name=` in a `rexx-sim: ` line.
            fn sim_field<'a>(line: &'a str, name: &str) -> Option<&'a str> {
                line.split(' ')
                    .find_map(|word| word.strip_prefix(name)?.strip_prefix('='))
            }

            /// `run` as compared: stdout masked, `rexx-sim: ` lines dropped.
            fn compared(run: &Run) -> Run {
                Run {
                    stdout: masked(&run.stdout),
                    stderr: sim_masked(&run.stderr),
                    status: run.status,
                }
            }

            /// The digest of a compared run's three descriptors.
            fn digest(run: &Run) -> u64 {
                let mut bytes = run.stdout.clone();
                bytes.push(0xff);
                bytes.extend_from_slice(&run.stderr);
                bytes.push(0xff);
                bytes.extend_from_slice(
                    run.status
                        .map_or_else(|| "none".to_string(), |status| status.to_string())
                        .as_bytes(),
                );
                fnv(&bytes)
            }

            /// A compared run's key: [`key`]'s for a group part, its status
            /// and line counts for a program.
            fn outcome_key(row: &Row, run: &Run) -> String {
                if !row.program() {
                    return key(run);
                }
                let lines = |bytes: &[u8]| bytes.iter().filter(|&&b| b == b'\n').count();
                format!(
                    "rc {}, stdout {} lines, stderr {} lines{}",
                    run.status
                        .map_or_else(|| "none".to_string(), |status| status.to_string()),
                    lines(&run.stdout),
                    lines(&run.stderr),
                    refusal(run)
                        .map(|line| format!(", {line}"))
                        .unwrap_or_default()
                )
            }

            /// One outcome of a part's oracle set.
            #[derive(Clone, Debug)]
            struct Seen {
                count: usize,
                status: String,
                digest: u64,
                key: String,
            }

            fn read_oracle(row: &Row) -> Option<Vec<Seen>> {
                let text = std::fs::read_to_string(row.oracle_file()).ok()?;
                Some(
                    text.lines()
                        .filter(|line| !line.is_empty() && !line.starts_with('#'))
                        .map(|line| {
                            let cells: Vec<&str> = line.splitn(4, '\t').collect();
                            let [count, status, digest, key] = cells[..] else {
                                panic!("{}: {line:?}", row.oracle_file().display());
                            };
                            Seen {
                                count: count.parse().expect("a count"),
                                status: status.to_string(),
                                digest: u64::from_str_radix(digest, 16).expect("a digest"),
                                key: key.to_string(),
                            }
                        })
                        .collect(),
                )
            }

            fn write_oracle(row: &Row, seen: &[Seen]) {
                let runs: usize = seen.iter().map(|seen| seen.count).sum();
                let mut text = format!(
                    "# The oracle's outcomes of {} {}, {runs} runs, written under \
                     {REFRESH_ENV}=1.\n# count, status, digest of the compared \
                     descriptors, key.\n",
                    row.group, row.part
                );
                for one in seen {
                    let _ = writeln!(
                        text,
                        "{}\t{}\t{:016x}\t{}",
                        one.count, one.status, one.digest, one.key
                    );
                }
                let path = row.oracle_file();
                std::fs::create_dir_all(path.parent().expect("a parent"))
                    .expect("the oracle directory");
                std::fs::write(&path, text).expect("cannot write an oracle set");
            }

            /// The tests a key names as failing.
            fn failing_of(key: &str) -> BTreeSet<String> {
                key.rsplit_once("failing [")
                    .and_then(|(_, rest)| rest.split_once(']'))
                    .map(|(names, _)| {
                        names
                            .split(' ')
                            .filter(|name| !name.is_empty())
                            .map(str::to_string)
                            .collect()
                    })
                    .unwrap_or_default()
            }

            /// What a red is.
            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            enum Kind {
                Panic,
                Crash,
                Hang,
                Invariant,
                Determinism,
                DesignLimit,
                Check,
                NoReport,
            }

            impl Kind {
                fn label(self) -> &'static str {
                    match self {
                        Kind::Panic => "panic",
                        Kind::Crash => "crash",
                        Kind::Hang => "hang",
                        Kind::Invariant => "invariant",
                        Kind::Determinism => "determinism",
                        Kind::DesignLimit => "design-limit",
                        Kind::Check => "check",
                        Kind::NoReport => "no-report",
                    }
                }
            }

            /// One red: its kind, the test it is in (`-` for a program) and
            /// the line that shows it.
            #[derive(Clone, Debug)]
            struct Red {
                kind: Kind,
                test: String,
                line: String,
            }

            /// The reds of one run (spec section 4, "Gate"; ruling R6).
            fn judge(row: &Row, policy: &str, ran: &ProcessRun, seen: Option<&[Seen]>) -> Vec<Red> {
                let run = &ran.run;
                let text = String::from_utf8_lossy(&run.stderr).into_owned();
                let test = if row.program() {
                    "-".to_string()
                } else {
                    started(run).pop().unwrap_or_else(|| "-".to_string())
                };
                let red = |kind, line: &str| {
                    vec![Red {
                        kind,
                        test: test.clone(),
                        line: line.to_string(),
                    }]
                };
                match ran.ended {
                    Ended::Killed => {
                        return red(
                            Kind::Hang,
                            &format!("killed at the deadline, {} s", row.deadline().as_secs()),
                        );
                    }
                    Ended::Signaled(signal) => {
                        return red(Kind::Crash, &format!("ended by signal {signal}"));
                    }
                    Ended::Exited(_) => {}
                }
                if let Some(line) = text.lines().find(|line| line.contains("panicked at")) {
                    return red(Kind::Panic, line);
                }
                if run.status == Some(101) {
                    return red(Kind::Panic, "rc 101");
                }
                if let Some(line) = refusal(run) {
                    if line.starts_with(INVARIANT) {
                        return red(Kind::Invariant, &line);
                    }
                    if DETERMINISM.iter().any(|text| line.contains(text)) {
                        return red(Kind::Determinism, &line);
                    }
                    let twin_hangs =
                        seen.is_some_and(|seen| seen.iter().any(|one| one.status == "none"));
                    if DESIGN_LIMIT.iter().any(|text| line.contains(text)) && !twin_hangs {
                        return red(Kind::DesignLimit, &line);
                    }
                }
                if sim_line(&run.stderr).is_none() {
                    return red(Kind::NoReport, "no rexx-sim line ends stderr");
                }
                if row.injects() || policy.starts_with("pct:") {
                    return Vec::new();
                }
                if row.program() {
                    return String::from_utf8_lossy(&run.stdout)
                        .lines()
                        .filter(|line| line.starts_with(ASSERTION))
                        .map(|line| Red {
                            kind: Kind::Check,
                            test: test.clone(),
                            line: line.to_string(),
                        })
                        .collect();
                }
                let mut known: BTreeSet<String> = seen
                    .unwrap_or_default()
                    .iter()
                    .flat_map(|one| failing_of(&one.key))
                    .collect();
                known.extend(failing_unswitched(row));
                failing(run)
                    .into_iter()
                    .filter(|name| !known.contains(name))
                    .map(|name| Red {
                        kind: Kind::Check,
                        line: format!("{name} fails; no oracle outcome of the part fails it"),
                        test: name,
                    })
                    .collect()
            }

            /// The tests [`DIFFERING`] lists as failing in `row`'s part in the
            /// shipped scheduler's mode, where its outcome is not the oracle's.
            fn failing_unswitched(row: &Row) -> BTreeSet<String> {
                DIFFERING
                    .iter()
                    .filter(|(group, part, mode, ..)| {
                        *group == row.group && *part == row.part && *mode == "normal"
                    })
                    .flat_map(|(.., key, _)| failing_of(key))
                    .collect()
            }

            /// One row of [`EXEMPT`]: group, part, test, kind and a text the
            /// red's line holds (its refusal where it has one), each `*` for
            /// any, then the reason and its evidence.
            struct Exempt {
                cells: [String; 5],
            }

            fn read_exempt() -> Vec<Exempt> {
                let path = rust_root().join(EXEMPT);
                let text = std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
                text.lines()
                    .filter(|line| !line.is_empty() && !line.starts_with('#'))
                    .map(|line| {
                        let cells: Vec<&str> = line.split('\t').collect();
                        assert_eq!(cells.len(), 7, "{EXEMPT}: {line:?} is not seven cells");
                        assert!(
                            cells[5..].iter().all(|cell| !cell.trim().is_empty()),
                            "{EXEMPT}: {line:?} has no reason or no evidence"
                        );
                        Exempt {
                            cells: [0, 1, 2, 3, 4].map(|at| cells[at].to_string()),
                        }
                    })
                    .collect()
            }

            fn exempted(exempt: &[Exempt], row: &Row, red: &Red) -> bool {
                let wanted = [
                    row.group.as_str(),
                    row.part.as_str(),
                    red.test.as_str(),
                    red.kind.label(),
                ];
                exempt.iter().any(|one| {
                    one.cells[..4]
                        .iter()
                        .zip(wanted)
                        .all(|(cell, wanted)| cell == "*" || cell == wanted)
                        && (one.cells[4] == "*" || red.line.contains(one.cells[4].as_str()))
                })
            }

            /// Runs `body` over `items` on [`JOBS_ENV`] threads, answering the
            /// results in order.
            fn in_jobs<I: Sync, T: Send>(
                items: &[I],
                body: impl Fn(usize, &I) -> T + Sync,
            ) -> Vec<T> {
                let jobs = std::env::var(JOBS_ENV).map_or(JOBS_DEFAULT, |value| {
                    value
                        .parse()
                        .unwrap_or_else(|_| panic!("{JOBS_ENV} is not a count: {value:?}"))
                });
                let next = Mutex::new(0_usize);
                let results: Mutex<Vec<Option<T>>> =
                    Mutex::new((0..items.len()).map(|_| None).collect());
                std::thread::scope(|scope| {
                    for _ in 0..jobs.max(1) {
                        scope.spawn(|| {
                            loop {
                                let at = {
                                    let mut next = next.lock().expect("the work index");
                                    let at = *next;
                                    *next += 1;
                                    at
                                };
                                if at >= items.len() {
                                    break;
                                }
                                let result = body(at, &items[at]);
                                results.lock().expect("the results")[at] = Some(result);
                            }
                        });
                    }
                });
                results
                    .into_inner()
                    .expect("the results")
                    .into_iter()
                    .map(|result| result.expect("every item ran"))
                    .collect()
            }

            /// A fresh scratch directory for item `at` of the run `name`.
            fn scratch(name: &str, at: usize) -> PathBuf {
                PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
                    .join(format!("{name}-{}", std::process::id()))
                    .join(at.to_string())
                    .join("run")
            }

            /// The scratch directory of `unit` below the run `name`, named by
            /// the unit alone: a program may read its own path, so a replay
            /// runs where the gate ran.
            fn unit_dir(name: &str, unit: &Unit) -> PathBuf {
                PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
                    .join(name)
                    .join(format!("{:016x}", fnv(unit.name().as_bytes())))
                    .join("run")
            }

            /// Removes the item directory a [`scratch`] directory `dir` is in,
            /// and the run's directory once it is empty.
            fn remove(dir: &Path) {
                let item = dir.parent().expect("an item directory");
                if item.exists() {
                    std::fs::remove_dir_all(item).expect("cannot remove a run");
                }
                // Fails while another item of the run is still there.
                let _ = std::fs::remove_dir(item.parent().expect("a run directory"));
            }

            /// What one unit found.
            struct Found {
                key: String,
                sim: String,
                took: Duration,
                reds: Vec<Red>,
                /// `None` where not compared, else the oracle outcome it
                /// matches, by number.
                compared: Option<Option<usize>>,
                /// The rerun of a run whose reds were all checks in a test
                /// whose outcome depends on the wall clock (ruling P48), and
                /// what it found.
                rerun: Option<String>,
            }

            /// Whether every red of `reds` is a failed check in a test
            /// `WALL_CLOCK` lists for `row`'s group.
            fn wall_clock_checks(row: &Row, reds: &[Red]) -> bool {
                !reds.is_empty()
                    && reds.iter().all(|red| {
                        red.kind == Kind::Check
                            && WALL_CLOCK.contains(&format!("{} {}", row.group, red.test).as_str())
                    })
            }

            fn run_unit(
                unit: &Unit,
                derived: &BTreeMap<String, BTreeSet<String>>,
                dir: &Path,
            ) -> Found {
                let mut ran = run_here(&unit.row, derived, dir, &unit.mode(), unit.row.deadline());
                remove(dir);
                let seen = read_oracle(&unit.row);
                let mut reds = judge(&unit.row, &unit.policy, &ran, seen.as_deref());
                let mut rerun = None;
                if wall_clock_checks(&unit.row, &reds) {
                    let first: Vec<&str> = reds.iter().map(|red| red.test.as_str()).collect();
                    let first = first.join(" ");
                    ran = run_here(&unit.row, derived, dir, &unit.mode(), unit.row.deadline());
                    remove(dir);
                    reds = judge(&unit.row, &unit.policy, &ran, seen.as_deref());
                    rerun = Some(format!(
                        "{}: failed {first}, rerun {}",
                        unit.name(),
                        if reds.is_empty() {
                            "passed".to_string()
                        } else {
                            format!("failed {}", outcome_key(&unit.row, &compared(&ran.run)))
                        }
                    ));
                }
                let mine = compared(&ran.run);
                let compared = (!unit.row.injects()).then(|| {
                    let digest = digest(&mine);
                    seen.as_deref()
                        .unwrap_or_default()
                        .iter()
                        .position(|one| one.digest == digest)
                });
                Found {
                    key: outcome_key(&unit.row, &mine),
                    sim: sim_line(&ran.run.stderr).unwrap_or_else(|| "-".to_string()),
                    took: ran.took,
                    reds,
                    compared,
                    rerun,
                }
            }

            fn report_path() -> PathBuf {
                std::env::var_os(REPORT_ENV).map_or_else(
                    || PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("sim-gate-report.txt"),
                    PathBuf::from,
                )
            }

            #[test]
            fn the_seeded_gate() {
                if !gate_mode() {
                    eprintln!("sim_gate: skipped without {GATE_ENV}");
                    return;
                }
                let rows = read_table();
                let exempt = read_exempt();
                let derived = derived_tests();
                let units = units(&rows);
                assert!(!units.is_empty(), "the gate has no run in this build");
                let start = std::time::Instant::now();
                let found = in_jobs(&units, |_, unit| {
                    run_unit(unit, &derived, &unit_dir("sim-gate", unit))
                });
                let wall = start.elapsed();
                let mut report = format!(
                    "# sim gate: {} runs in {} s, profile {}, stack {}\n\
                     # group\tpart\tseed\tpolicy\tms\toracle\tkey\trexx-sim\n",
                    units.len(),
                    wall.as_secs(),
                    if cfg!(debug_assertions) {
                        "debug"
                    } else {
                        "release"
                    },
                    rexx_exec::INTERPRETER_STACK_BYTES
                );
                let mut differences = String::new();
                let reruns: Vec<&str> = found
                    .iter()
                    .filter_map(|one| one.rerun.as_deref())
                    .collect();
                let mut reds = Vec::new();
                let mut exempted_reds = Vec::new();
                for (unit, found) in units.iter().zip(&found) {
                    let oracle = match found.compared {
                        None => "not compared".to_string(),
                        Some(Some(at)) => format!("agrees with {}", at + 1),
                        Some(None) => {
                            let theirs: Vec<String> = read_oracle(&unit.row)
                                .unwrap_or_default()
                                .iter()
                                .map(|one| format!("{} x {}", one.count, one.key))
                                .collect();
                            let _ = writeln!(
                                differences,
                                "{}: ours {}; oracle [{}]",
                                unit.name(),
                                found.key,
                                theirs.join("; ")
                            );
                            "differs".to_string()
                        }
                    };
                    let _ = writeln!(
                        report,
                        "{}\t{}\t{}\t{}\t{}\t{oracle}\t{}\t{}",
                        unit.row.group,
                        unit.row.part,
                        unit.seed,
                        unit.policy,
                        found.took.as_millis(),
                        found.key,
                        found.sim
                    );
                    for red in &found.reds {
                        let line = format!(
                            "{} {} in {}: {}\n  replay: {}",
                            red.kind.label(),
                            unit.name(),
                            red.test,
                            red.line,
                            unit.replay()
                        );
                        if exempted(&exempt, &unit.row, red) {
                            exempted_reds.push(line);
                        } else {
                            reds.push(line);
                        }
                    }
                }
                let _ = write!(
                    report,
                    "\n## Differences from the oracle sets\n{differences}\n## Wall-clock reruns \
                     (ruling P48)\n{}\n\n## Reds\n{}\n\n## Exempted reds\n{}\n",
                    reruns.join("\n"),
                    reds.join("\n"),
                    exempted_reds.join("\n")
                );
                let path = report_path();
                std::fs::write(&path, &report).expect("cannot write the report");
                eprintln!(
                    "sim gate: {} runs, {} s, {} reds, {} exempted, {} differences; report {}",
                    units.len(),
                    wall.as_secs(),
                    reds.len(),
                    exempted_reds.len(),
                    differences.lines().count(),
                    path.display()
                );
                assert!(reds.is_empty(), "{}", reds.join("\n"));
            }

            /// Step 4's self-test: each row with debug seeds, seed 0 under
            /// `pre:2`, run twice, each in a process of its own, gives one
            /// trace hash and one outcome.
            #[test]
            fn a_seeded_run_repeats_in_a_fresh_process() {
                if !gate_mode() {
                    eprintln!("sim_gate: skipped without {GATE_ENV}");
                    return;
                }
                let derived = derived_tests();
                let sample: Vec<Unit> = read_table()
                    .into_iter()
                    .filter(|row| row.debug_seeds > 0)
                    .map(|row| Unit {
                        seed: seed(&row.group, &row.part, 0),
                        policy: with_k("pre:2", row.k),
                        row,
                    })
                    .collect();
                assert!(!sample.is_empty(), "no row has debug seeds");
                let pairs = in_jobs(&sample, |_, unit| {
                    [0, 1].map(|_| {
                        let dir = unit_dir("sim-self", unit);
                        let ran =
                            run_here(&unit.row, &derived, &dir, &unit.mode(), unit.row.deadline());
                        remove(&dir);
                        ran
                    })
                });
                let mut broken = Vec::new();
                for (unit, [first, second]) in sample.iter().zip(&pairs) {
                    let hash = |ran: &ProcessRun| {
                        sim_line(&ran.run.stderr)
                            .and_then(|line| sim_field(&line, "trace").map(str::to_string))
                    };
                    let same = hash(first).is_some()
                        && hash(first) == hash(second)
                        && compared(&first.run).stdout == compared(&second.run).stdout
                        && compared(&first.run).stderr == compared(&second.run).stderr
                        && first.run.status == second.run.status;
                    eprintln!(
                        "self-test {}: trace {:?} {:?}, {}",
                        unit.name(),
                        hash(first),
                        hash(second),
                        if same { "same" } else { "DIFFERENT" }
                    );
                    if !same {
                        broken.push(format!(
                            "{}: {} / {}\n  replay: {}",
                            unit.name(),
                            outcome_key(&unit.row, &compared(&first.run)),
                            outcome_key(&unit.row, &compared(&second.run)),
                            unit.replay()
                        ));
                    }
                }
                assert!(broken.is_empty(), "{}", broken.join("\n"));
            }

            /// Under [`REFRESH_ENV`], runs each row's oracle 5 times, 30 where
            /// its outcomes vary or none is the outcome of seed 0 here under
            /// `fifo`, and adds what it saw to the row's committed set.
            #[test]
            fn the_oracle_sets_grow_only_under_refresh() {
                if !gate_mode() || std::env::var(REFRESH_ENV).as_deref() != Ok("1") {
                    eprintln!("sim_gate: oracle sets not refreshed without {REFRESH_ENV}=1");
                    return;
                }
                let oracle = oracle::locate();
                let derived = derived_tests();
                let rows: Vec<Row> = read_table()
                    .into_iter()
                    .filter(|row| !row.injects())
                    .filter(|row| {
                        std::env::var(ONLY_ENV).map_or(true, |only| {
                            let cells: Vec<&str> = only.splitn(3, ':').collect();
                            cells[0] == row.group
                                && cells.get(1).is_none_or(|part| *part == row.part)
                        })
                    })
                    .collect();
                in_jobs(&rows, |at, row| {
                    let dir = scratch("sim-oracle", at);
                    let oracle_runs = |runs: std::ops::Range<usize>| {
                        at_once(&runs.collect::<Vec<_>>(), |k| {
                            let run =
                                run_oracle(&oracle, row, &derived, &dir.join(format!("o{k}")));
                            compared(&run)
                        })
                    };
                    let mut runs = oracle_runs(0..ORACLE_RUNS);
                    let mine = compared(
                        &run_here(
                            row,
                            &derived,
                            &dir.join("here"),
                            &row.mode(seed(&row.group, &row.part, 0), "fifo"),
                            CALIBRATION_DEADLINE,
                        )
                        .run,
                    );
                    let digests: BTreeSet<u64> = runs.iter().map(digest).collect();
                    if digests.len() > 1 || !digests.contains(&digest(&mine)) {
                        runs.extend(oracle_runs(ORACLE_RUNS..ORACLE_RUNS_UNSETTLED));
                    }
                    remove(&dir);
                    let mut seen = read_oracle(row).unwrap_or_default();
                    for run in &runs {
                        let digest = digest(run);
                        match seen.iter_mut().find(|one| one.digest == digest) {
                            Some(one) => one.count += 1,
                            None => seen.push(Seen {
                                count: 1,
                                status: run.status.map_or_else(
                                    || "none".to_string(),
                                    |status| status.to_string(),
                                ),
                                digest,
                                key: outcome_key(row, run),
                            }),
                        }
                    }
                    seen.sort_by_key(|one| std::cmp::Reverse(one.count));
                    write_oracle(row, &seen);
                    eprintln!(
                        "oracle set {} {}: {} outcomes",
                        row.group,
                        row.part,
                        seen.len()
                    );
                });
            }

            /// Step 1: where [`CALIBRATION_ENV`] names a file, runs each part
            /// once under `fifo` (seed 0, no preemption but the floor) and
            /// writes its contended steps, wall time and outcome; a whole part
            /// whose run refuses gets a rest part leaving the refusing tests
            /// out, as `whole_groups` derives it.
            #[test]
            fn calibration() {
                let Some(out) = std::env::var_os(CALIBRATION_ENV) else {
                    eprintln!("sim_gate: no calibration without {CALIBRATION_ENV}");
                    return;
                };
                let derived = derived_tests();
                let mut rows: Vec<Row> = Vec::new();
                let blank = |group: &str, part: &str, knobs: Option<&str>| Row {
                    group: group.to_string(),
                    part: part.to_string(),
                    k: 0,
                    release_seeds: 0,
                    debug_seeds: 0,
                    release_ms: 0,
                    debug_ms: 0,
                    knobs: knobs.map(str::to_string),
                    left_out: Vec::new(),
                };
                for group in derived.keys() {
                    rows.push(blank(group, "whole", None));
                    rows.push(blank(group, "derived", None));
                }
                let list = rust_root().join("corpus/phase-6.txt");
                for line in std::fs::read_to_string(&list).expect("phase-6.txt").lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
                        rows.push(blank(&format!("corpus/{line}"), "program", None));
                    }
                }
                for (program, knobs) in [
                    ("m11_stale_sleeper.rex", Some("fail=wait:1")),
                    ("n1_halt_ready_once.rex", Some("halt@30000")),
                    ("timer_post_wakes_every_waiter.rex", None),
                ] {
                    rows.push(blank(&format!("{PROGRAMS}{program}"), "program", knobs));
                }
                let lines = in_jobs(&rows, |at, row| {
                    let mut row = row.clone();
                    let mut out = String::new();
                    let mut refusing: Vec<String> = Vec::new();
                    loop {
                        let dir = scratch("sim-calibration", at);
                        let mode = row.mode(seed(&row.group, &row.part, 0), "fifo");
                        let ran = run_here(&row, &derived, &dir, &mode, CALIBRATION_DEADLINE);
                        remove(&dir);
                        let mine = compared(&ran.run);
                        let line = sim_line(&ran.run.stderr).unwrap_or_default();
                        let contended = sim_field(&line, "contended").unwrap_or("0");
                        let _ = writeln!(
                            out,
                            "{}\t{}\t{contended}\t{}\t{}\t{}\t{:?}\t{}",
                            row.group,
                            row.part,
                            ran.took.as_millis(),
                            row.knobs.as_deref().unwrap_or("-"),
                            if refusing.is_empty() {
                                "-".to_string()
                            } else {
                                refusing.join(" ")
                            },
                            ran.ended,
                            outcome_key(&row, &mine)
                        );
                        if row.program() || row.part == "derived" {
                            break;
                        }
                        let Some(line) = refusal(&mine) else { break };
                        if line.ends_with("the run exceeded its deadline") {
                            break;
                        }
                        let Some(test) = started(&mine).pop() else {
                            break;
                        };
                        assert!(
                            !refusing.contains(&test),
                            "{}: {test} refused twice",
                            row.group
                        );
                        refusing.push(test);
                        row.part = "rest".to_string();
                        row.left_out = refusing.clone();
                    }
                    out
                });
                std::fs::write(&out, lines.concat()).expect("cannot write the calibration");
            }

            #[test]
            fn a_seed_is_derived_by_rule_and_the_mix_repeats_every_seven() {
                assert_eq!(fnv(b""), 0xcbf2_9ce4_8422_2325);
                assert_eq!(splitmix(0, 0), 0xe220_a839_7b1d_cdaf);
                let a = seed("base/class/Alarm.testGroup", "whole", 0);
                assert_eq!(a, seed("base/class/Alarm.testGroup", "whole", 0));
                assert_ne!(a, seed("base/class/Alarm.testGroup", "whole", 1));
                assert_ne!(a, seed("base/class/Alarm.testGroup", "derived", 0));
                let row = Row {
                    group: "g".to_string(),
                    part: "whole".to_string(),
                    k: 40,
                    release_seeds: 7,
                    debug_seeds: 0,
                    release_ms: 0,
                    debug_ms: 0,
                    knobs: None,
                    left_out: Vec::new(),
                };
                if std::env::var(POLICY_ENV).is_err() {
                    let policies: Vec<String> = (0..8).map(|i| row.policy(i)).collect();
                    assert_eq!(
                        policies,
                        [
                            "pre:1,k=40",
                            "pre:2,k=40",
                            "pre:3,k=40",
                            "pct:3,k=40",
                            "uniform:0.01",
                            "uniform:0.2",
                            "uniform:1",
                            "pre:1,k=40"
                        ]
                    );
                }
                assert_eq!(with_k("pre:2", 0), "pre:2,k=1");
            }

            #[test]
            fn a_run_is_compared_without_its_sim_lines() {
                let stderr = b"rexx-sim: seed=9 policy=fifo\nstarted X\nrexx-sim: seed=1 \
                    policy=fifo steps=3 contended=2 switches=1 trace=00000000000000ff \
                    profile=release stack=1\n";
                assert_eq!(sim_masked(stderr), b"started X\n");
                let line = sim_line(stderr).expect("the run's own line");
                assert_eq!(sim_field(&line, "contended"), Some("2"));
                assert_eq!(sim_field(&line, "trace"), Some("00000000000000ff"));
                assert_eq!(sim_line(b"rexx-sim: x\nrexx-exec: y\n"), None);
            }

            #[test]
            fn a_key_names_its_failing_tests() {
                assert_eq!(
                    failing_of("failure, assertions 3, rc 1, last started B, failing [A B]"),
                    BTreeSet::from(["A".to_string(), "B".to_string()])
                );
                assert!(
                    failing_of("pass, assertions 3, rc 0, last started B, failing []").is_empty()
                );
            }

            /// Every row of the table is a part the gate names: each group
            /// file of the derived list whole and derived, each program of
            /// `corpus/phase-6.txt`, and the gate's own programs; a rest or
            /// single row names a group with a whole row. Every other row has an
            /// oracle set, and every exempt row names a table row.
            #[test]
            fn the_table_holds_the_parts_the_spec_names() {
                let rows = read_table();
                let have: BTreeSet<(String, String)> = rows
                    .iter()
                    .filter(|row| {
                        !matches!(row.part.as_str(), "rest" | "single")
                            && !row.group.starts_with(PROGRAMS)
                    })
                    .map(|row| (row.group.clone(), row.part.clone()))
                    .collect();
                let mut want = BTreeSet::new();
                for group in derived_tests().keys() {
                    want.insert((group.clone(), "whole".to_string()));
                    want.insert((group.clone(), "derived".to_string()));
                }
                let list = rust_root().join("corpus/phase-6.txt");
                for line in std::fs::read_to_string(&list).expect("phase-6.txt").lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
                        want.insert((format!("corpus/{line}"), "program".to_string()));
                    }
                }
                assert_eq!(have, want);
                for row in rows.iter().filter(|row| row.part == "single") {
                    assert!(
                        have.contains(&(row.group.clone(), "whole".to_string())),
                        "{} single has no whole row",
                        row.group
                    );
                    assert_eq!(row.left_out.len(), 1, "{} single names one test", row.group);
                }
                for row in rows.iter().filter(|row| row.part == "rest") {
                    assert!(
                        have.contains(&(row.group.clone(), "whole".to_string())),
                        "{} rest has no whole row",
                        row.group
                    );
                    assert!(
                        !row.left_out.is_empty(),
                        "{} rest leaves nothing out",
                        row.group
                    );
                }
                for row in rows.iter().filter(|row| !row.injects()) {
                    assert!(
                        read_oracle(row).is_some_and(|seen| !seen.is_empty()),
                        "{} {} has no committed oracle set",
                        row.group,
                        row.part
                    );
                }
                assert!(rows.iter().any(|row| row.release_seeds > 0));
                assert!(rows.iter().any(|row| row.debug_seeds > 0));
                for one in read_exempt() {
                    assert!(
                        rows.iter().any(|row| {
                            (one.cells[0] == "*" || one.cells[0] == row.group)
                                && (one.cells[1] == "*" || one.cells[1] == row.part)
                        }),
                        "{EXEMPT} row {:?} names no table row",
                        one.cells
                    );
                }
            }
        }
    }
}
