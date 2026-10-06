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
                let mut results = run_tests(&oracle, run, dir, group, one, mode, None);
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
    /// inside the test's own `INTERPRET`, or `USE LOCAL` in a method.
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
                let results =
                    outcome_table(name, "base/class", group, "REXX_TIMER_TABLE", mode, |_| {
                        true
                    });
                let oracle = oracle::locate();
                for row in &results {
                    let mut label = row.outcome.label().to_string();
                    let key = format!("base/class/{group}.testGroup {}", row.test);
                    if !matches!(row.outcome, Outcome::Pass) && WALL_CLOCK.contains(&key.as_str()) {
                        eprintln!("P48 rerun: {key} {name}: {label}");
                        let run = scratch(&format!("{name}-rerun"));
                        let one = std::slice::from_ref(&row.test);
                        let again = on_a_quiet_machine(|| {
                            run_tests(&oracle, &run, "base/class", group, one, mode, None)
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

    /// The rows refused in both modes whose trace output, on stderr, comes
    /// from a `REPLY` continuation and its sender: the lines interleave
    /// differently and the refused run ends after a different number. Allowed
    /// only while both runs end in the same `rexx-exec: ` refusal.
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
        if normal.status == every.status
            && masked(&normal.stdout) == masked(&every.stdout)
            && refusal(normal).is_some()
            && refusal(normal) == refusal(every)
            && TRACE_INTERLEAVES.contains(&row)
        {
            return ("trace lines on stderr differ".to_string(), true);
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

        /// Which tests of the group a run holds.
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum Part {
            Whole,
            Derived,
        }

        impl Part {
            fn label(self) -> &'static str {
                match self {
                    Part::Whole => "whole",
                    Part::Derived => "derived",
                }
            }
        }

        /// The runs here, by group, part and mode, that are not an outcome
        /// the oracle produced, each with its [`key`] and reason.
        const DIFFERING: &[(&str, &str, &str, &str, &str)] = &[
            (
                "base/bif/STREAM.testGroup",
                "whole",
                "normal",
                "error, assertions 195, rc 2",
                "tests outside the derived list that fail alone too, outside Phase 6",
            ),
            (
                "base/bif/STREAM.testGroup",
                "whole",
                "every",
                "error, assertions 195, rc 2",
                "tests outside the derived list that fail alone too, outside Phase 6",
            ),
            (
                "base/bif/TIME.testGroup",
                "whole",
                "normal",
                "failure, assertions 506, rc 1",
                "elapsed-clock defect, queued 2026-10-02-elapsed-clock-per-routine-and-reset",
            ),
            (
                "base/bif/TIME.testGroup",
                "whole",
                "every",
                "failure, assertions 506, rc 1",
                "elapsed-clock defect, queued 2026-10-02-elapsed-clock-per-routine-and-reset",
            ),
            (
                "base/bif/TIME.testGroup",
                "derived",
                "normal",
                "failure, assertions 65, rc 1",
                "elapsed-clock defect, queued 2026-10-02-elapsed-clock-per-routine-and-reset",
            ),
            (
                "base/bif/TIME.testGroup",
                "derived",
                "every",
                "failure, assertions 65, rc 1",
                "elapsed-clock defect, queued 2026-10-02-elapsed-clock-per-routine-and-reset",
            ),
            (
                "base/class/Class.testGroup",
                "whole",
                "normal",
                "rexx-exec: method \"TEST1\" of class \"TESTDEFINE1\" is not implemented (Phase 9)",
                "refused at a test outside the derived list, outside Phase 6",
            ),
            (
                "base/class/Class.testGroup",
                "whole",
                "every",
                "rexx-exec: method \"TEST1\" of class \"TESTDEFINE1\" is not implemented (Phase 9)",
                "refused at a test outside the derived list, outside Phase 6",
            ),
            (
                "base/class/DateTime.testGroup",
                "whole",
                "normal",
                "rexx-exec: DO is not implemented",
                "refused at a test outside the derived list, outside Phase 6",
            ),
            (
                "base/class/DateTime.testGroup",
                "whole",
                "every",
                "rexx-exec: DO is not implemented",
                "refused at a test outside the derived list, outside Phase 6",
            ),
            (
                "base/class/Message.testGroup",
                "whole",
                "normal",
                "rexx-exec: method \"MAKEARRAY\" of class \"Object\" is not implemented (Phase 9)",
                "TEST_STARTWITH_NOT_ARRAY's refusal, as in its own run",
            ),
            (
                "base/class/Message.testGroup",
                "whole",
                "every",
                "rexx-exec: method \"MAKEARRAY\" of class \"Object\" is not implemented (Phase 9)",
                "TEST_STARTWITH_NOT_ARRAY's refusal, as in its own run",
            ),
            (
                "base/class/Message.testGroup",
                "derived",
                "normal",
                "rexx-exec: method \"MAKEARRAY\" of class \"Object\" is not implemented (Phase 9)",
                "TEST_STARTWITH_NOT_ARRAY's refusal, as in its own run",
            ),
            (
                "base/class/Message.testGroup",
                "derived",
                "every",
                "rexx-exec: method \"MAKEARRAY\" of class \"Object\" is not implemented (Phase 9)",
                "TEST_STARTWITH_NOT_ARRAY's refusal, as in its own run",
            ),
            (
                "base/class/Method.testGroup",
                "whole",
                "normal",
                "rexx-exec: DO is not implemented",
                "TESTDIRECTIVES' refusal, as in its own run",
            ),
            (
                "base/class/Method.testGroup",
                "whole",
                "every",
                "rexx-exec: DO is not implemented",
                "TESTDIRECTIVES' refusal, as in its own run",
            ),
            (
                "base/class/Method.testGroup",
                "derived",
                "normal",
                "rexx-exec: DO is not implemented",
                "TESTDIRECTIVES' refusal, as in its own run",
            ),
            (
                "base/class/Method.testGroup",
                "derived",
                "every",
                "rexx-exec: DO is not implemented",
                "TESTDIRECTIVES' refusal, as in its own run",
            ),
            (
                "base/class/MethodArgs.testGroup",
                "whole",
                "normal",
                "rexx-exec: DO is not implemented",
                "the TEST_REQUEST_STRING refusal, as in their own runs",
            ),
            (
                "base/class/MethodArgs.testGroup",
                "whole",
                "every",
                "rexx-exec: DO is not implemented",
                "the TEST_REQUEST_STRING refusal, as in their own runs",
            ),
            (
                "base/class/MethodArgs.testGroup",
                "derived",
                "normal",
                "rexx-exec: DO is not implemented",
                "the TEST_REQUEST_STRING refusal, as in their own runs",
            ),
            (
                "base/class/MethodArgs.testGroup",
                "derived",
                "every",
                "rexx-exec: DO is not implemented",
                "the TEST_REQUEST_STRING refusal, as in their own runs",
            ),
            (
                "base/class/MutexSemaphore.testGroup",
                "whole",
                "every",
                "rexx-exec: the run exceeded its deadline",
                "TEST_EXCLUSION's own race, which the oracle hangs in too (P46)",
            ),
            (
                "base/class/MutexSemaphore.testGroup",
                "derived",
                "every",
                "rexx-exec: the run exceeded its deadline",
                "TEST_EXCLUSION's own race, which the oracle hangs in too (P46)",
            ),
            (
                "base/class/Object.testGroup",
                "whole",
                "normal",
                "rexx-exec: method \"MAKEARRAY\" of class \"Object\" is not implemented (Phase 9)",
                "refused at a test outside the derived list, outside Phase 6",
            ),
            (
                "base/class/Object.testGroup",
                "whole",
                "every",
                "rexx-exec: method \"MAKEARRAY\" of class \"Object\" is not implemented (Phase 9)",
                "refused at a test outside the derived list, outside Phase 6",
            ),
            (
                "base/class/RexxContext.testGroup",
                "whole",
                "normal",
                "rexx-exec: CONDITION option \"O\" answers a Directory, which is not implemented",
                "refused at a test outside the derived list, outside Phase 6",
            ),
            (
                "base/class/RexxContext.testGroup",
                "whole",
                "every",
                "rexx-exec: CONDITION option \"O\" answers a Directory, which is not implemented",
                "refused at a test outside the derived list, outside Phase 6",
            ),
            (
                "base/directives/ATTRIBUTE.testGroup",
                "whole",
                "normal",
                "rexx-exec: test does not parse here: 25.925: Invalid subkeyword found. is not implemented (Phase 5)",
                "refused at a test outside the derived list, outside Phase 6",
            ),
            (
                "base/directives/ATTRIBUTE.testGroup",
                "whole",
                "every",
                "rexx-exec: test does not parse here: 25.925: Invalid subkeyword found. is not implemented (Phase 5)",
                "refused at a test outside the derived list, outside Phase 6",
            ),
            (
                "base/directives/ATTRIBUTE.testGroup",
                "derived",
                "normal",
                "failure, assertions 187, rc 1",
                "TESTDELEGATE, as in its own run: Method delegate attributes, outside Phase 6",
            ),
            (
                "base/directives/ATTRIBUTE.testGroup",
                "derived",
                "every",
                "failure, assertions 187, rc 1",
                "TESTDELEGATE, as in its own run: Method delegate attributes, outside Phase 6",
            ),
            (
                "base/directives/CONSTANT.testGroup",
                "whole",
                "normal",
                "rexx-exec: constant_TestGroup does not parse here: 19.916: String or symbol expected as ::CONSTANT value. is not implemented (Phase 5)",
                "refused at a test outside the derived list, outside Phase 6",
            ),
            (
                "base/directives/CONSTANT.testGroup",
                "whole",
                "every",
                "rexx-exec: constant_TestGroup does not parse here: 19.916: String or symbol expected as ::CONSTANT value. is not implemented (Phase 5)",
                "refused at a test outside the derived list, outside Phase 6",
            ),
            (
                "base/directives/METHOD.testGroup",
                "whole",
                "normal",
                "rexx-exec: test does not parse here: 25.902: Invalid subkeyword found. is not implemented (Phase 5)",
                "refused at a test outside the derived list, outside Phase 6",
            ),
            (
                "base/directives/METHOD.testGroup",
                "whole",
                "every",
                "rexx-exec: test does not parse here: 25.902: Invalid subkeyword found. is not implemented (Phase 5)",
                "refused at a test outside the derived list, outside Phase 6",
            ),
            (
                "base/directives/METHOD.testGroup",
                "derived",
                "normal",
                "failure, assertions 67, rc 1",
                "TESTDELEGATE, as in its own run: Method delegate attributes, outside Phase 6",
            ),
            (
                "base/directives/METHOD.testGroup",
                "derived",
                "every",
                "failure, assertions 67, rc 1",
                "TESTDELEGATE, as in its own run: Method delegate attributes, outside Phase 6",
            ),
            (
                "base/keyword/CALL.testGroup",
                "whole",
                "normal",
                "rexx-exec: test does not parse here: 19.2: String or symbol expected after CALL keyword. is not implemented (Phase 5)",
                "refused at a test outside the derived list, outside Phase 6",
            ),
            (
                "base/keyword/CALL.testGroup",
                "whole",
                "every",
                "rexx-exec: test does not parse here: 19.2: String or symbol expected after CALL keyword. is not implemented (Phase 5)",
                "refused at a test outside the derived list, outside Phase 6",
            ),
            (
                "base/keyword/CALL.testGroup",
                "derived",
                "normal",
                "failure, assertions 7, rc 1",
                "elapsed-clock defect, queued 2026-10-02-elapsed-clock-per-routine-and-reset",
            ),
            (
                "base/keyword/CALL.testGroup",
                "derived",
                "every",
                "failure, assertions 7, rc 1",
                "elapsed-clock defect, queued 2026-10-02-elapsed-clock-per-routine-and-reset",
            ),
            (
                "base/keyword/GUARD.testGroup",
                "whole",
                "normal",
                "rexx-exec: test does not parse here: 25.913: Invalid subkeyword found. is not implemented (Phase 5)",
                "refused at a test outside the derived list, outside Phase 6",
            ),
            (
                "base/keyword/GUARD.testGroup",
                "whole",
                "every",
                "rexx-exec: test does not parse here: 25.913: Invalid subkeyword found. is not implemented (Phase 5)",
                "refused at a test outside the derived list, outside Phase 6",
            ),
            (
                "base/keyword/GUARD.testGroup",
                "derived",
                "normal",
                "rexx-exec: USE LOCAL in a ::METHOD body is not implemented (Phase 5)",
                "TEST_WHEN_USE_LOCAL_NO_WAIT's refusal, as in its own run",
            ),
            (
                "base/keyword/GUARD.testGroup",
                "derived",
                "every",
                "rexx-exec: USE LOCAL in a ::METHOD body is not implemented (Phase 5)",
                "TEST_WHEN_USE_LOCAL_NO_WAIT's refusal, as in its own run",
            ),
            (
                "base/keyword/RAISE.testGroup",
                "whole",
                "normal",
                "failure, assertions 120, rc 1",
                "TEST_RAISE_INSERT_CRLF as in its own run, and tests outside the derived list that fail alone too, outside Phase 6",
            ),
            (
                "base/keyword/RAISE.testGroup",
                "whole",
                "every",
                "failure, assertions 120, rc 1",
                "TEST_RAISE_INSERT_CRLF as in its own run, and tests outside the derived list that fail alone too, outside Phase 6",
            ),
            (
                "base/keyword/RAISE.testGroup",
                "derived",
                "normal",
                "failure, assertions 0, rc 1",
                "TEST_RAISE_INSERT_CRLF, as in its own run: message text conversion, outside Phase 6",
            ),
            (
                "base/keyword/RAISE.testGroup",
                "derived",
                "every",
                "failure, assertions 0, rc 1",
                "TEST_RAISE_INSERT_CRLF, as in its own run: message text conversion, outside Phase 6",
            ),
            (
                "base/keyword/REPLY.testGroup",
                "whole",
                "every",
                "pass, assertions 19, rc 0",
                "every continuation runs before the program ends; the oracle races them with its end (P41)",
            ),
            (
                "base/keyword/REPLY.testGroup",
                "derived",
                "every",
                "pass, assertions 18, rc 0",
                "every continuation runs before the program ends; the oracle races them with its end (P41)",
            ),
            (
                "base/keyword/TRACE.testGroup",
                "whole",
                "normal",
                "rexx-exec: DO is not implemented",
                "refused at a test outside the derived list, outside Phase 6",
            ),
            (
                "base/keyword/TRACE.testGroup",
                "whole",
                "every",
                "rexx-exec: DO is not implemented",
                "refused at a test outside the derived list, outside Phase 6",
            ),
            (
                "base/keyword/TRACE_TraceObject.testGroup",
                "whole",
                "normal",
                "rexx-exec: DO is not implemented",
                "the TEST_TRACEOBJECT_COLLECTOR and TEST_CALLER_STACK_FRAME_REPLY_START refusal, as in their own runs",
            ),
            (
                "base/keyword/TRACE_TraceObject.testGroup",
                "whole",
                "every",
                "rexx-exec: DO is not implemented",
                "the TEST_TRACEOBJECT_COLLECTOR and TEST_CALLER_STACK_FRAME_REPLY_START refusal, as in their own runs",
            ),
            (
                "base/keyword/TRACE_TraceObject.testGroup",
                "derived",
                "normal",
                "rexx-exec: DO is not implemented",
                "the TEST_TRACEOBJECT_COLLECTOR and TEST_CALLER_STACK_FRAME_REPLY_START refusal, as in their own runs",
            ),
            (
                "base/keyword/TRACE_TraceObject.testGroup",
                "derived",
                "every",
                "rexx-exec: DO is not implemented",
                "the TEST_TRACEOBJECT_COLLECTOR and TEST_CALLER_STACK_FRAME_REPLY_START refusal, as in their own runs",
            ),
            (
                "doc/rexxref/chapter5/Section1.testGroup",
                "whole",
                "normal",
                "rexx-exec: method \"OBJECTNAME=\" of class \"Object\" is not implemented (Phase 9)",
                "refused at a test outside the derived list, outside Phase 6",
            ),
            (
                "doc/rexxref/chapter5/Section1.testGroup",
                "whole",
                "every",
                "rexx-exec: method \"OBJECTNAME=\" of class \"Object\" is not implemented (Phase 9)",
                "refused at a test outside the derived list, outside Phase 6",
            ),
        ];

        /// A run's refusal, or its summary.
        fn key(run: &Run) -> String {
            String::from_utf8_lossy(&run.stderr)
                .lines()
                .find(|line| line.starts_with("rexx-exec: "))
                .map_or_else(|| summary(run), str::to_string)
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
        /// out, and the group file's path in it.
        fn copy(
            run: &Path,
            name: &str,
            dir: &str,
            group: &str,
            left_out: &BTreeSet<String>,
        ) -> (PathBuf, String) {
            let at = run.join(name);
            fresh_copy(&at, dir);
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
            distinct.sort_by(|a, b| b.1.cmp(&a.1));
            distinct
        }

        fn summary(run: &Run) -> String {
            match run.status {
                Some(status) => format!("{}, rc {status}", outcome(&run.stdout)),
                None => "did not finish".to_string(),
            }
        }

        fn difference(theirs: &Run, ours: &Run) -> String {
            first_difference(&masked(&theirs.stdout), &masked(&ours.stdout))
                .replace(['\n', '\t'], " ")
        }

        /// One row's result: the table's cells, whether each mode agrees with
        /// the oracle, and whether each ended in an inverted wait or did not
        /// finish.
        struct Row {
            cells: String,
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

        fn one_row(
            oracle: &oracle::Oracle,
            run: &Path,
            file: &str,
            part: Part,
            derived: &BTreeSet<String>,
        ) -> Row {
            let (dir, group) = file
                .trim_end_matches(".testGroup")
                .rsplit_once('/')
                .expect("a group below a directory");
            let mut left_out = reaching_rxapi(dir, &[group]);
            if part == Part::Derived {
                let path = super::super::worktree()
                    .join("ootest/ooRexx")
                    .join(dir)
                    .join(format!("{group}.testGroup"));
                let text = read_lossy(&path);
                left_out.extend(
                    text.lines()
                        .filter_map(|line| {
                            let line = line.trim_start();
                            let rest = line
                                .get(..8)?
                                .eq_ignore_ascii_case("::method")
                                .then(|| &line[8..])?;
                            let name = rest.split_whitespace().next()?.trim_matches(['\'', '"']);
                            Some(name.to_ascii_uppercase())
                        })
                        .filter(|name| name.starts_with("TEST") && !derived.contains(name))
                        .map(|test| format!("{group}.{test}")),
                );
            }
            let left_out = &left_out;
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
            let ours = at_once(&modes, |(name, mode)| {
                let (at, path) = copy(run, name, dir, group, left_out);
                let args = ["-f", path.as_str(), "-U", "-V", VERBOSITY];
                relative(run_crate_within(&at, &args, *mode, deadline), &at)
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
                if at == 0 {
                    cells.push_str(&format!("1: {count} {}", summary(run)));
                } else {
                    let detail = difference(seen[0].0, run);
                    cells.push_str(&format!("; {}: {count} {}, {detail}", at + 1, summary(run)));
                }
            }
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
                    None => format!("differs: {}, {}", keys[at], difference(seen[0].0, run)),
                };
                cells.push_str(&format!("\t{cell}"));
            }
            let same = if agree(&ours[0], &ours[1]) {
                "same"
            } else {
                "differ"
            };
            cells.push_str(&format!("\t{same}"));
            Row {
                cells,
                keys,
                agrees,
                stuck,
            }
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
            let rows: Vec<(&String, Part)> = files
                .iter()
                .flat_map(|file| [(file, Part::Whole), (file, Part::Derived)])
                .collect();
            let oracle = oracle::locate();
            let results = rows_in_parallel(
                "whole-groups",
                &rows,
                |(file, _)| {
                    WALL_CLOCK
                        .iter()
                        .any(|row| row.starts_with(&format!("{file} ")))
                },
                |run, (file, part)| {
                    let (_, group) = file
                        .trim_end_matches(".testGroup")
                        .rsplit_once('/')
                        .expect("a group below a directory");
                    let mine: BTreeSet<String> = derived
                        .iter()
                        .filter_map(|row| row.strip_prefix(&format!("{group}.")))
                        .map(str::to_string)
                        .collect();
                    one_row(&oracle, run, file, *part, &mine)
                },
            );
            let mut table = String::from(
                "group\tpart\toracle runs\toracle outcomes\tnormal\tevery\tnormal against every\n",
            );
            let mut failing = Vec::new();
            for ((file, part), row) in rows.iter().zip(&results) {
                table.push_str(&row.cells);
                table.push('\n');
                for (at, mode) in ["normal", "every"].into_iter().enumerate() {
                    let listed = DIFFERING
                        .iter()
                        .find(|(group, listed_part, listed_mode, ..)| {
                            group == file && *listed_part == part.label() && *listed_mode == mode
                        });
                    let name = format!("{file} {} {mode}", part.label());
                    match listed {
                        Some((.., key, _)) if row.agrees[at] || row.keys[at] != *key => {
                            failing
                                .push(format!("{name}: listed as {key:?}, is {:?}", row.keys[at]));
                        }
                        Some(_) => {}
                        None if row.stuck[at] => {
                            failing.push(format!("{name}: an inverted wait or a hang"));
                        }
                        None if !row.agrees[at] => {
                            failing.push(format!("{name}: not an oracle outcome"));
                        }
                        None => {}
                    }
                }
            }
            eprintln!("{table}");
            if let Some(path) = std::env::var_os(TABLE_ENV) {
                std::fs::write(path, &table).expect("cannot write the table");
            }
            assert!(failing.is_empty(), "{failing:#?}");
        }
    }
}
