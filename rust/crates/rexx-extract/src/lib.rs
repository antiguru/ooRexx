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

//! Lifts individual test methods out of ooTest `.testGroup` files so they can
//! run as standalone programs long before the ooTest framework itself works.
//!
//! This is a heuristic, and deliberately conservative: a method that touches
//! fixture state set up by `setUp` cannot stand alone, so it is flagged and
//! skipped rather than mis-extracted into a silently-passing test.
//!
//! [`extract_assertions`] is a second, unrelated extraction mode for
//! `base/expressions`: wrapping a whole method as `::routine main public`
//! produces a program whose prolog is empty (nothing precedes the directive,
//! so nothing calls it), which runs and asserts nothing at all. That mode
//! emits one *row* per `self~assertSame` call instead -- the two expressions
//! it compares, verbatim and unparsed, plus the `NUMERIC DIGITS`/`FORM`
//! settings in force when it runs and any assignment prelude the same method
//! established first -- so a later harness with a real evaluator can check
//! each one directly, without ever running the extracted text as a program.

/// The set of `self~` messages that are assertions rather than fixture access.
const ASSERTIONS: &[&str] = &[
    "assertequals",
    "assertnotequals",
    "asserttrue",
    "assertfalse",
    "assertnull",
    "assertnotnull",
    "assertsame",
    "assertnotsame",
    "expectsyntax",
    "assertlistequals",
    "assertarrayequals",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestMethod {
    pub name: String,
    pub body: String,
    /// True when the body reads or writes `self~<something>` that is not an
    /// assertion, meaning it depends on fixture state and cannot stand alone.
    pub uses_fixture: bool,
}

pub fn extract(source: &str) -> Vec<TestMethod> {
    let mut out = Vec::new();
    let mut current: Option<(String, Vec<&str>)> = None;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = strip_directive(trimmed, "::method") {
            if let Some((name, body)) = current.take() {
                push_if_test(&mut out, name, body);
            }
            // A quoted `::method` name may use either quote character --
            // `::method 'test_15_bit'` in MULTIPLICATION.testGroup is single
            // -quoted, unlike every other name in the suite. Stripping only
            // `"` left the quotes in the name, which then failed the
            // "starts with test" filter below and silently dropped the
            // method: 8 whole methods and 864 assertSame calls in that one
            // file (measured 2026-07-30, extracting base/expressions).
            let name = rest
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_matches(['"', '\'']);
            current = Some((name.to_string(), Vec::new()));
        } else if trimmed.starts_with("::") {
            if let Some((name, body)) = current.take() {
                push_if_test(&mut out, name, body);
            }
        } else if let Some((_, body)) = current.as_mut() {
            body.push(line);
        }
    }
    if let Some((name, body)) = current.take() {
        push_if_test(&mut out, name, body);
    }
    out
}

fn strip_directive<'a>(line: &'a str, directive: &str) -> Option<&'a str> {
    let lower = line.to_ascii_lowercase();
    lower
        .starts_with(directive)
        .then(|| line[directive.len()..].trim_start())
}

fn push_if_test(out: &mut Vec<TestMethod>, name: String, body: Vec<&str>) {
    if !name.to_ascii_lowercase().starts_with("test") {
        return;
    }
    let body = body.join("\n");
    out.push(TestMethod {
        uses_fixture: touches_fixture(&body),
        name,
        body,
    });
}

fn touches_fixture(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    let mut rest = lower.as_str();
    while let Some(at) = rest.find("self~") {
        rest = &rest[at + "self~".len()..];
        let message: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '.')
            .collect();
        if !ASSERTIONS.contains(&message.as_str()) {
            return true;
        }
    }
    false
}

/// `NUMERIC FORM` at the point an assertion runs. Defaults to `Scientific`,
/// exactly as a fresh method activation does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Form {
    Scientific,
    Engineering,
}

/// A condition an assertion's expression is expected to **raise**, in place
/// of a value it is expected to equal.
///
/// `self~expectSyntax(major.sub)` does not check anything itself -- it sets
/// a flag (`OOREXXUNIT.CLS`: `self~conditionExpected = .true; self~
/// conditionName = "SYNTAX"; self~conditionCode = major.sub`) that the
/// framework's own per-method condition trap consults *later*, whenever
/// something in the rest of that method actually raises. So a
/// `self~assertSame` seen after one is not testing "expr equals expected"
/// at all: it is testing "evaluating expr raises major.sub", and `expected`
/// (its second argument) is never even reached under the oracle, because
/// Rexx evaluates a message send's arguments left to right and the raise
/// happens while evaluating the first one.
///
/// Measured, `DIVISION.testGroup`'s `test_262`:
///
/// ```text
/// self~expectSyntax(26.11)
/// Numeric Digits 5
/// self~assertSame("-5678932" % "-37", 1)
/// ```
///
/// running `"-5678932" % "-37"` under `DIGITS 5` raises the oracle's own
/// Error 26.11 ("Result of % operation did not result in a whole number"),
/// which is what this row is actually checking -- not that the quotient
/// equals `1`. Note `expectSyntax` is not even the *immediately* preceding
/// line here; `NUMERIC DIGITS` sits between it and the assertion, which is
/// why this is carried as state (below) rather than checked one line back.
///
/// **Carried sequentially, exactly like `digits`/`form`**, and for the same
/// reason: a second `self~expectSyntax` in the same method (none occur in
/// `base/expressions` today -- checked -- but nothing about the mechanism
/// is specific to there being only one) would set a new expectation for
/// whatever follows it, the same way a second `NUMERIC DIGITS` would. This
/// struct is `Copy` so carrying it costs nothing extra in the scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RaiseExpectation {
    pub major: u32,
    pub sub: u32,
}

/// One `self~assertSame` call, together with everything about its method's
/// prior statements that decides what it means: `NUMERIC DIGITS`/`FORM` in
/// force, any assignment lines earlier in the same method, and whether a
/// `self~expectSyntax` earlier in the method turns this row into a
/// raise-expectation rather than a value comparison.
///
/// `expr` and `expected` are `assertSame`'s two positional arguments,
/// verbatim source text, unparsed. Both are ordinary Rexx expressions --
/// method-call arguments always are, so a quoted literal and a bare signed
/// number are both just expressions that happen to be constant -- and
/// deciding what either one evaluates to needs a real evaluator, which is a
/// later harness's job, not this extractor's. When `expect_raise` is
/// `Some`, `expected` is still the raw text `assertSame` was called with,
/// kept for provenance, but it is not meaningful to compare against: see
/// `RaiseExpectation`'s own doc for why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssertionRow {
    pub group: String,
    pub method: String,
    /// Assignment statements from earlier in the same method, verbatim and
    /// in source order. Empty for every assertion except the few that need
    /// one (`CONCATENATION`'s `a`..`g` variables, as of this writing).
    pub prelude: Vec<String>,
    pub expr: String,
    pub expected: String,
    pub digits: u32,
    pub form: Form,
    /// `Some` when a `self~expectSyntax` earlier in this method means this
    /// row tests a raise rather than a value. See [`RaiseExpectation`].
    pub expect_raise: Option<RaiseExpectation>,
}

/// One method whose remaining `assertSame` calls could not become rows, and
/// why, and how many were dropped.
///
/// Assertions from *before* the blocking statement in the same method still
/// become rows: nothing about them was actually invalidated, only the
/// state after that point stopped being trustworthy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockedMethod {
    pub group: String,
    pub method: String,
    pub reason: String,
    pub dropped: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AssertionExtraction {
    pub rows: Vec<AssertionRow>,
    pub blocked: Vec<BlockedMethod>,
}

/// Scans every `test`-prefixed `::method` in `source` for `self~assertSame`
/// calls and turns each into an [`AssertionRow`], carrying `NUMERIC`
/// settings and any assignment prelude sequentially through the method
/// exactly as the interpreter would.
///
/// `group` is a caller-supplied label (typically the `.testGroup` file's
/// stem, e.g. `"PRECEDENCE"`) copied verbatim into every row and blocked
/// entry; this function has no notion of files.
pub fn extract_assertions(group: &str, source: &str) -> AssertionExtraction {
    let mut out = AssertionExtraction::default();
    for method in extract(source) {
        scan_method_for_assertions(group, &method.name, &method.body, &mut out);
    }
    out
}

fn scan_method_for_assertions(
    group: &str,
    method: &str,
    body: &str,
    out: &mut AssertionExtraction,
) {
    let mut digits: u32 = 9;
    let mut form = Form::Scientific;
    let mut prelude: Vec<String> = Vec::new();
    let mut expect_raise: Option<RaiseExpectation> = None;
    let mut blocked_reason: Option<String> = None;
    let mut dropped = 0usize;

    let block = |reason: &mut Option<String>, line: &str| {
        if reason.is_none() {
            *reason = Some(format!("unsupported statement: {}", line.trim()));
        }
    };

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("--") {
            continue;
        }
        let lower = trimmed.to_ascii_lowercase();

        if let Some(rest) = lower.strip_prefix("numeric digits") {
            match rest.trim().parse::<u32>() {
                Ok(n) => {
                    if blocked_reason.is_none() {
                        digits = n;
                    }
                }
                Err(_) => block(&mut blocked_reason, trimmed),
            }
            continue;
        }
        if let Some(rest) = lower.strip_prefix("numeric form") {
            match rest.trim() {
                "scientific" => {
                    if blocked_reason.is_none() {
                        form = Form::Scientific;
                    }
                }
                "engineering" => {
                    if blocked_reason.is_none() {
                        form = Form::Engineering;
                    }
                }
                _ => block(&mut blocked_reason, trimmed),
            }
            continue;
        }
        if lower.starts_with("numeric fuzz") {
            // FUZZ only widens `=`/`<`/`>`'s tolerance; assertSame compares
            // with `==`, which FUZZ never affects, so there is no state to
            // carry here.
            continue;
        }
        if lower.starts_with("self~assertsame") {
            let parsed = parse_assert_same(trimmed);
            if blocked_reason.is_none() {
                match parsed {
                    Some((expr, expected)) => out.rows.push(AssertionRow {
                        group: group.to_string(),
                        method: method.to_string(),
                        prelude: prelude.clone(),
                        expr,
                        expected,
                        digits,
                        form,
                        expect_raise,
                    }),
                    None => {
                        block(&mut blocked_reason, trimmed);
                        dropped += 1;
                    }
                }
            } else {
                dropped += 1;
            }
            continue;
        }
        // `self~expectSyntax(major.sub)` defers a raise-check to whatever
        // in the rest of this method actually raises -- see
        // `RaiseExpectation`'s own doc. Checked, and blocked on parse
        // failure, *before* the generic `self~expect` fallback just below,
        // which would otherwise treat this exact prefix as inert and
        // silently keep emitting ordinary value-comparison rows for
        // whatever `self~assertSame` follows.
        if let Some(rest) = lower.strip_prefix("self~expectsyntax") {
            match parse_raise_expectation(rest.trim()) {
                Some(expectation) => {
                    if blocked_reason.is_none() {
                        expect_raise = Some(expectation);
                    }
                }
                None => block(&mut blocked_reason, trimmed),
            }
            continue;
        }
        // `self~expectCondition(name)` defers the same way, but for an
        // arbitrary named condition rather than a numbered `SYNTAX` one --
        // this row schema has nowhere to carry a bare condition name, so
        // rather than silently fall into the generic inert branch below
        // (which would keep emitting ordinary value rows exactly as
        // wrongly as the `expectSyntax` gap this struct exists to close),
        // block. Zero occurrences of `self~expectCondition` anywhere near
        // an `assertSame` in `base/expressions` today -- checked -- so
        // this is a forward guard against the corpus growing, not a
        // measured case.
        if lower.starts_with("self~expectcondition") {
            block(&mut blocked_reason, trimmed);
            continue;
        }
        // `self~assertSyntaxError`/`self~assertRuntimeError` call
        // `self~expectSyntax` internally (`OOREXXUNIT.CLS:1203`/`:1213`) and
        // then attempt the risky statement themselves -- but neither checks
        // the raise *locally*. There is no local catch inside either
        // method: if the attempt raises, it escapes exactly the way a bare
        // `self~expectSyntax` followed by a raising statement would, all
        // the way out to the same per-method trap. So a `self~assertSame`
        // is safe to leave un-converted after one of these two not because
        // they are self-contained -- they are not -- but only because none
        // of the 33 occurrences (all in `Literals.testGroup`, all wrapping
        // calls like `self~assertSyntaxError((15.1, 1), self~hex(" "))`)
        // has a `self~assertSame` anywhere later in the same method
        // (checked, 0 occurrences). That is an empirical fact about this
        // corpus, not a structural guarantee this scanner can rely on --
        // block here too, the same forward guard `self~expectCondition`
        // above has, rather than trusting the generic "other assertion
        // kind, inert" fallback below to be safe here by luck.
        if lower.starts_with("self~assertsyntaxerror")
            || lower.starts_with("self~assertruntimeerror")
        {
            block(&mut blocked_reason, trimmed);
            continue;
        }
        if lower.starts_with("self~assert") || lower.starts_with("self~expect") {
            // Every other assertion kind (`assertTrue`, `assertEquals`,
            // `assertNull`, ...): no variables assigned, and no deferred
            // expectation set either.
            continue;
        }
        if blocked_reason.is_none()
            && let Some(assign) = simple_assignment(trimmed)
        {
            prelude.push(assign.to_string());
            continue;
        }
        block(&mut blocked_reason, trimmed);
    }

    if dropped > 0 {
        out.blocked.push(BlockedMethod {
            group: group.to_string(),
            method: method.to_string(),
            reason: blocked_reason.unwrap_or_default(),
            dropped,
        });
    }
}

/// Splits a `self~assertSame(...)` clause into its two positional arguments,
/// verbatim apart from surrounding whitespace. `line` must already be known
/// (case-insensitively) to start with `self~assertSame`.
///
/// `None` means the shape this scanner requires does not hold: no opening
/// paren immediately after the message name, unbalanced parens, a comma
/// count at the call's own nesting depth other than exactly one, or text
/// trailing after the closing paren. None of the corpus's 4,269 calls hit
/// this (checked before writing this scanner), so `None` here means a
/// method this scanner has never seen the shape of, not an expected case.
fn parse_assert_same(line: &str) -> Option<(String, String)> {
    let prefix_len = "self~assertsame".len();
    let rest = line.get(prefix_len..)?.trim_start();
    let mut it = rest.char_indices();
    let (open, ch) = it.next()?;
    if ch != '(' {
        return None;
    }

    let mut depth = 1i32;
    let mut in_str: Option<char> = None;
    let mut top_comma: Option<usize> = None;
    let mut close: Option<usize> = None;
    let mut chars = rest[open + 1..].char_indices().peekable();
    while let Some((rel, c)) = chars.next() {
        let i = open + 1 + rel;
        if let Some(q) = in_str {
            if c == q {
                if chars.peek().map(|&(_, next)| next) == Some(q) {
                    chars.next(); // doubled quote: an escaped quote, stay in the string
                } else {
                    in_str = None;
                }
            }
            continue;
        }
        match c {
            '\'' | '"' => in_str = Some(c),
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(i);
                }
            }
            ',' if depth == 1 => {
                if top_comma.is_some() {
                    return None; // more than one top-level comma
                }
                top_comma = Some(i);
            }
            _ => {}
        }
        if close.is_some() {
            break;
        }
    }
    let close = close?;
    if !rest[close + 1..].trim().is_empty() {
        return None; // trailing text after the call
    }
    let comma = top_comma?;
    let expr = rest[open + 1..comma].trim().to_string();
    let expected = rest[comma + 1..close].trim().to_string();
    Some((expr, expected))
}

/// Parses `self~expectSyntax`'s argument, already known (case-insensitively)
/// to follow that prefix. `rest` is everything after `self~expectSyntax`,
/// trimmed -- expected to be exactly `(major.sub)` with nothing else, which
/// is the shape every one of the 184 `self~assertSame` calls this scanner
/// converts to a raise expectation follows.
///
/// `None` for anything else, including `expectSyntax`'s own array form
/// (`(major.sub, message inserts...)`, which the method also accepts but
/// none of these 184 rows' own markers use) or a trailing `msg` argument.
/// Rather than guess which comma-separated item is the code, or discard the
/// inserts, this blocks the method -- the same conservative choice
/// `parse_assert_same` makes for an `assertSame` shape it has not seen
/// either.
fn parse_raise_expectation(rest: &str) -> Option<RaiseExpectation> {
    let inner = rest.strip_prefix('(')?.strip_suffix(')')?.trim();
    if inner.contains(',') {
        return None;
    }
    let (major, sub) = inner.split_once('.')?;
    let major = major.trim().parse::<u32>().ok()?;
    let sub = sub.trim().parse::<u32>().ok()?;
    Some(RaiseExpectation { major, sub })
}

/// A whole-line, single-clause assignment: a symbol (letters, digits, `_`,
/// `.`, `!`, `?` -- covers plain and compound variable names) followed by
/// `=` (not `==`). Rejects a line carrying an unquoted `;`, since that is a
/// second clause this scanner cannot safely fold into one prelude line
/// verbatim. Returns `line` verbatim on a match.
fn simple_assignment(line: &str) -> Option<&str> {
    let is_symbol_char = |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '!' | '?');
    let ident_len: usize = line
        .char_indices()
        .take_while(|&(_, c)| is_symbol_char(c))
        .map(|(_, c)| c.len_utf8())
        .sum();
    if ident_len == 0 {
        return None;
    }
    let mut rest = line[ident_len..].trim_start().chars();
    if rest.next() != Some('=') {
        return None;
    }
    if rest.next() == Some('=') {
        return None; // `==`, not an assignment
    }
    if has_unquoted(line, ';') {
        return None;
    }
    Some(line)
}

/// Whether `target` occurs outside any `'...'`/`"..."` string, with the same
/// doubled-quote-is-an-escaped-quote rule `parse_assert_same` uses.
fn has_unquoted(line: &str, target: char) -> bool {
    let mut in_str: Option<char> = None;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if let Some(q) = in_str {
            if c == q {
                if chars.peek() == Some(&q) {
                    chars.next();
                } else {
                    in_str = None;
                }
            }
        } else if c == '\'' || c == '"' {
            in_str = Some(c);
        } else if c == target {
            return true;
        }
    }
    false
}

/// Every `.testGroup` file under `dir`, recursively, sorted for a
/// deterministic walk order.
pub fn find_test_groups(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => panic!("cannot read {}: {e}", dir.display()),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(find_test_groups(&path));
        } else if path.extension().is_some_and(|e| e == "testGroup") {
            out.push(path);
        }
    }
    out.sort();
    out
}
