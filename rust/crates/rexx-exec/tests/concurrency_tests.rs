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

#[cfg(feature = "pinning")]
mod measured {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};

    use rayon::prelude::*;
    use rexx_exec::{Invocation, Outcome, ParkKind, PinKind, PinReport, SwitchMode, run_program};

    use super::derive;
    use super::group_runner::{fresh_copy, worktree};

    /// Runs `source` in a directory holding the external routine `extf.rex`.
    fn report_of(source: &str) -> PinReport {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("pinning-probes");
        fs::create_dir_all(&dir).expect("the probe directory");
        fs::write(
            dir.join("extf.rex"),
            "use arg n\nif n >= 3 then do\n  call SysSleep 0\n  return 'ok'\nend\nreturn extf(n + 1)\n",
        )
        .expect("the external routine");
        let outcome = run_program(
            "probe.rex",
            source.as_bytes().to_vec(),
            Invocation::none().with_directory(dir),
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
            Invocation::none().with_switch_mode(SwitchMode::AtClause(4)),
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
                Invocation::none().with_switch_mode(SwitchMode::EveryOpportunity),
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
            "DeferredReply",
            "say .c~new~m\n::class c\n::method m\n  reply 1\n  call SysSleep 0\n",
        ),
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
            let outcome: Outcome =
                run_program("probe.rex", program.into_bytes(), Invocation::none());
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
    fn an_unimplemented_wait_is_a_park_point() {
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
        let report = report_of(".mutexSemaphore~new~acquire\n");
        assert_eq!(
            frames_at(&report, ParkKind::SemaphoreWait).len(),
            1,
            "{report:?}"
        );
        let report = report_of("call SysWaitEventSem 1\n");
        assert_eq!(
            frames_at(&report, ParkKind::SysSemWait).len(),
            1,
            "{report:?}"
        );
    }

    fn run_test(run: &Path, group: &str, test: &str) -> Outcome {
        let dir = Path::new(group).parent().and_then(Path::to_str);
        fresh_copy(run, dir.expect("a group directory"));
        let driver = run.join("testOORexx.rex");
        let text = fs::read(&driver).expect("the copied driver");
        let group_file = run.join("ooRexx").join(group);
        let args = format!("-f {} -U -V 2 -t {test}", group_file.display());
        let lib = super::support::oracle::oracle_root().join("lib");
        let mut environment: Vec<(Vec<u8>, Vec<u8>)> = std::env::vars()
            .filter(|(name, _)| name != "LD_LIBRARY_PATH")
            .map(|(name, value)| (name.into_bytes(), value.into_bytes()))
            .collect();
        environment.push((
            b"LD_LIBRARY_PATH".to_vec(),
            lib.to_string_lossy().into_owned().into_bytes(),
        ));
        let invocation = Invocation::with_argument(args.into_bytes())
            .with_directory(run.to_path_buf())
            .with_environment(environment);
        let outcome = super::watchdog::run_bounded(&driver.to_string_lossy(), text, invocation);
        fs::remove_dir_all(run).unwrap_or_else(|e| panic!("cannot remove {}: {e}", run.display()));
        outcome
    }

    /// The run's outcome: the refusal, a deadline, or the summary's class.
    fn outcome_of(outcome: &Outcome) -> String {
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
        let (list, _) = derive(&worktree().join("ootest/ooRexx"));
        let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("pinning-{}", std::process::id()));
        let rows: Vec<(String, String, PinReport)> = list
            .par_iter()
            .enumerate()
            .map(|(at, row)| {
                let outcome = run_test(&base.join(at.to_string()), &row.group, &row.test);
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
        let text = format!(
            "{summary}\narrivals {all}, with a frame other than TreeEval, TreeSend or OpExec {beyond}\n\n{per_test}"
        );
        let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("pinning-table.md");
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
        GATE_ENV, Outcome, SwitchMode, TestResult, excerpt, first_difference, gate_mode,
        group_file, masked, reaching_rxapi, run_tests, source_test_names, test_names,
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

    /// The Message start tests whose outcome differs from the oracle's:
    /// `SysSleep` blocks the thread, so the halt finds the started activity
    /// before its first clause and is dropped.
    const MESSAGE_START_DIFFERING: &[&str] = &["TEST_HALT_START"];

    /// The same under `EveryOpportunity`: main's pinned yields run the
    /// started `delayValueReturn`, whose `SysSleep` blocks the thread, to its
    /// end before the test asserts it has not completed.
    const MESSAGE_START_DIFFERING_SWITCHED: &[&str] = &["TEST_START"];

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
        // One group at a time: two at once outgrow the gate's memory cap.
        static ONE_GROUP: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _one = ONE_GROUP
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let oracle = oracle::locate();
        let run = scratch(name);
        let tests: Vec<String> = source_test_names(dir, group)
            .into_iter()
            .filter(|test| chosen(test))
            .collect();
        let results = run_tests(&oracle, &run, dir, group, &tests, mode, None);
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
        fs::remove_dir_all(run.parent().expect("a parent")).expect("cannot remove the run");
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
        for test in ["TEST_SEND", "TEST_START"] {
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
}
