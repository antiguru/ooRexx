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

//! The fourth extraction mode, for `ootest/ooRexx/base/bif`, and the only one
//! that is a *reuse* of an earlier mode rather than a new shape.
//! ```text
//! ::method "test_21"                       -- C2D.testGroup:129
//!    self~expectSyntax(40.5)
//!    self~assertSame(C2D(,-1), '-1')
//! ```

use std::collections::{BTreeMap, BTreeSet};

use crate::keyword::{
    blank_comments, contains_unquoted, count_assert_same, ends_with_continuation, parse_two_args,
};
use crate::{AssertionRow, Form, RaiseExpectation};

/// Whether `c` may appear in a Rexx symbol.
fn is_symbol_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '!' | '?')
}

/// The exact method name a row comes from, lowercased. Matched
/// case-insensitively and **never by prefix**: `self~assertSameList` shares
/// the whole of it and is a different method.
const ASSERT_SAME: &str = "self~assertsame";

/// How deep a `.local~` fixture may name another before this gives up. A
/// bound rather than a cycle set because the corpus's deepest chain is one
/// hop (`OVERLAY.testGroup`'s `.vres1` names `.v256` and `.v30`) and a bound
/// terminates on a cycle too.
const FIXTURE_DEPTH: usize = 8;

/// Why a `self~assertSame` occurrence did not become an [`AssertionRow`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DropReason {
    /// Not inside a `::method` whose name begins `test`: the file's prolog, a
    /// `::class` directive's region, or a method the framework never runs as a
    /// test.
    OutsideTestMethod,
    /// Inside a `::routine` body. Reachable only via a raise from some other
    /// body, so there is no method to attribute it to. See the module doc.
    RoutineBody,
    /// Inside a `/* */` or `--` comment: text that looks like a call and is
    /// not one. [`count_assert_same`] does not know what a comment is,
    /// deliberately, so these have to be subtracted somewhere.
    InsideComment,
    /// On a line joined to its neighbour by a trailing `,` or `-`, so the call
    /// is part of a larger clause.
    ContinuedLine,
    /// Not the first thing on its line, so the state this scanner carries is
    /// not the state the call actually ran under.
    NotAClause,
    /// An argument list this scanner has not seen: no `(` after the method
    /// name, unbalanced parens, or not two or three arguments.
    UnparsedCallShape,
    /// At or after a `self~expectSyntax` in the same body. The row would state
    /// a return value where the body requires a raise; see the module doc.
    ExpectedRaise,
    /// The body sends a real message -- a `self~` attribute, or any other `~`.
    /// Needs dispatch, which is Phase 5's.
    MessageSend,
    /// The body reaches a statement that is neither an assignment nor an
    /// ooTest assertion: a loop, an `IF`, a `CALL`, a `PARSE`.
    UnsupportedStatement,
    /// A `NUMERIC DIGITS`/`FORM` whose operand is not a literal this scanner
    /// can carry (`numeric digits a`, `numeric form value`).
    NumericNotConstant,
    /// A `.NAME` the file assigns more than one distinct text to, so which
    /// value is in force depends on the order the framework ran the bodies in.
    UnresolvedFixture,
    /// The expression calls a `::routine` defined in the same file, which a
    /// lifted row does not carry.
    InFileRoutine,
    /// The file carries `::options novalue` (or `::options all`) and the
    /// expression reads a symbol nothing assigned, so in place the body raises
    /// 98.986. See the module doc.
    NoValueUnderOptions,
    /// The body sets an expectation this row schema has nowhere to carry --
    /// `self~expectCondition`, `self~assertSyntaxError`,
    /// `self~assertRuntimeError` -- each of which defers a check to whatever
    /// raises next, exactly as `expectSyntax` does.
    OtherExpectation,
    /// The call's own text carries `U+FFFD`, so the bytes it was written with
    /// did not survive being read as UTF-8.
    NonUtf8Source,
    /// An earlier `self~assertSame` in the same body changed the variable pool
    /// while its own arguments were being evaluated, so this row's state is not
    /// what its prelude says.
    SideEffectingAssertion,
    /// The call reads the clock or the random generator, so two evaluations of
    /// it need not agree (decision D11).
    ClockDependent,
}

impl DropReason {
    /// A short stable label for reports and for the tests that pin the
    /// per-reason counts.
    pub fn label(self) -> &'static str {
        match self {
            DropReason::OutsideTestMethod => "outside any test-prefixed method",
            DropReason::RoutineBody => "inside a ::routine body",
            DropReason::InsideComment => "inside a comment, not a call",
            DropReason::ContinuedLine => "on a continued line",
            DropReason::NotAClause => "not first on its line",
            DropReason::UnparsedCallShape => "unparsed call shape",
            DropReason::ExpectedRaise => "at or after a self~expectSyntax",
            DropReason::MessageSend => "body uses a message send",
            DropReason::UnsupportedStatement => "body uses an unsupported statement",
            DropReason::NumericNotConstant => "NUMERIC operand is not a literal",
            DropReason::UnresolvedFixture => "a .local fixture the file sets twice",
            DropReason::InFileRoutine => "calls a ::routine defined in the file",
            DropReason::NoValueUnderOptions => "unassigned symbol under ::options novalue",
            DropReason::OtherExpectation => "body sets another kind of expectation",
            DropReason::NonUtf8Source => "the source bytes are not UTF-8",
            DropReason::SideEffectingAssertion => "an earlier assertion assigned via VALUE",
            DropReason::ClockDependent => "reads the clock or the random generator",
        }
    }

    /// Every variant, so a caller reporting a breakdown lists the ones
    /// standing at zero too rather than only those it happened to hit.
    pub const ALL: &'static [DropReason] = &[
        DropReason::OutsideTestMethod,
        DropReason::RoutineBody,
        DropReason::InsideComment,
        DropReason::ContinuedLine,
        DropReason::NotAClause,
        DropReason::UnparsedCallShape,
        DropReason::ExpectedRaise,
        DropReason::MessageSend,
        DropReason::UnsupportedStatement,
        DropReason::NumericNotConstant,
        DropReason::UnresolvedFixture,
        DropReason::InFileRoutine,
        DropReason::NoValueUnderOptions,
        DropReason::OtherExpectation,
        DropReason::NonUtf8Source,
        DropReason::SideEffectingAssertion,
        DropReason::ClockDependent,
    ];
}

/// One body's worth of `self~assertSame` calls lost to a single reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockedBody {
    pub group: String,
    pub method: String,
    pub reason: DropReason,
    /// The offending source text, trimmed, from the *first* call lost to this
    /// reason in this body. Kept beside the category because the category is
    /// what gets counted and this is what a reader needs to find the line.
    pub detail: String,
    pub dropped: usize,
}

/// A body whose `self~expectSyntax` says something must **raise**, rather than
/// a row saying an expression must equal something.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RaiseRow {
    pub group: String,
    pub method: String,
    /// Assignment statements from earlier in the same body, verbatim, in
    /// source order, with any `.local` fixture already substituted.
    pub prelude: Vec<String>,
    pub operands: Vec<String>,
    pub digits: u32,
    pub form: Form,
    pub expect: RaiseExpectation,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BifExtraction {
    pub rows: Vec<AssertionRow>,
    pub raises: Vec<RaiseRow>,
    pub blocked: Vec<BlockedBody>,
}

impl BifExtraction {
    /// `self~assertSame` occurrences that did not become a row.
    pub fn dropped(&self) -> usize {
        self.blocked.iter().map(|b| b.dropped).sum()
    }
}

/// Every `.local~NAME =` assignment in one file, and the names that have more
/// than one answer.
#[derive(Debug, Clone, Default)]
struct Fixtures {
    /// Lowercased name to the assignment's own source text, for a name the
    /// file assigns exactly one distinct text.
    values: BTreeMap<String, String>,
    /// Lowercased names the file assigns two or more distinct texts. Not
    /// resolvable: which one is in force depends on the order the framework
    /// ran the bodies in.
    ambiguous: BTreeSet<String>,
}

/// What a `::` directive opens, as far as this extractor cares.
#[derive(Debug, Clone, PartialEq, Eq)]
enum SegmentKind {
    /// A `::method` whose name begins `test`, carrying that name.
    TestMethod(String),
    /// A `::routine`.
    Routine,
    /// Everything else: the file's prolog, a `::class`, a `::method` the
    /// framework does not run as a test.
    Other,
}

/// One directive's region: the lines from just after the directive to just
/// before the next `::` of any kind.
#[derive(Debug)]
struct Segment {
    kind: SegmentKind,
    /// Line indices into the file, `start..end`.
    start: usize,
    end: usize,
}

/// Every `self~assertSame` in `source` that could become an
/// [`AssertionRow`], plus the [`RaiseRow`]s its `expectSyntax` bodies stand
/// for and the per-reason accounting for everything else.
pub fn extract_bif(group: &str, source: &str) -> BifExtraction {
    let mut out = BifExtraction::default();
    let blanked = blank_comments(source);

    let total = count_assert_same(source);
    let in_code = count_assert_same(&blanked);
    if total > in_code {
        out.blocked.push(BlockedBody {
            group: group.to_string(),
            method: "<inside a comment>".to_string(),
            reason: DropReason::InsideComment,
            detail: "an assertSame written inside a /* */ or -- comment".to_string(),
            dropped: total - in_code,
        });
    }

    let original: Vec<&str> = source.lines().collect();
    let blank_lines: Vec<&str> = blanked.lines().collect();
    let fixtures = collect_fixtures(&original, &blank_lines);
    let routines = collect_routine_names(&blank_lines);
    let novalue = has_novalue_option(&blank_lines);

    for segment in segments(&blank_lines) {
        let calls: usize = blank_lines[segment.start..segment.end]
            .iter()
            .map(|line| count_assert_same(line))
            .sum();
        if calls == 0 {
            continue;
        }
        match &segment.kind {
            SegmentKind::TestMethod(name) => {
                let mut body = BodyScan::new(group, name, &fixtures, &routines, novalue);
                body.run(
                    &original[segment.start..segment.end],
                    &blank_lines[segment.start..segment.end],
                );
                body.finish(&mut out);
            }
            SegmentKind::Routine => out.blocked.push(BlockedBody {
                group: group.to_string(),
                method: "<a ::routine>".to_string(),
                reason: DropReason::RoutineBody,
                detail: "a condition handler's assertion, reachable only from another body"
                    .to_string(),
                dropped: calls,
            }),
            SegmentKind::Other => out.blocked.push(BlockedBody {
                group: group.to_string(),
                method: "<outside any test-prefixed method>".to_string(),
                reason: DropReason::OutsideTestMethod,
                detail: "not inside a `test`-prefixed ::method".to_string(),
                dropped: calls,
            }),
        }
    }

    out
}

/// Splits the comment-blanked file into one [`Segment`] per `::` directive,
/// plus a leading `Other` for the prolog.
fn segments(blanked: &[&str]) -> Vec<Segment> {
    let mut out = Vec::new();
    let mut kind = SegmentKind::Other;
    let mut start = 0usize;
    for (at, line) in blanked.iter().enumerate() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("::") {
            continue;
        }
        out.push(Segment {
            kind,
            start,
            end: at,
        });
        kind = directive_kind(trimmed);
        start = at + 1;
    }
    out.push(Segment {
        kind,
        start,
        end: blanked.len(),
    });
    out
}

/// What a line already known to start `::` opens.
fn directive_kind(trimmed: &str) -> SegmentKind {
    let lower = trimmed.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix("::method") {
        let name = trimmed[trimmed.len() - rest.len()..]
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_matches(['"', '\'']);
        if name.to_ascii_lowercase().starts_with("test") {
            return SegmentKind::TestMethod(name.to_string());
        }
        return SegmentKind::Other;
    }
    if lower.starts_with("::routine") {
        return SegmentKind::Routine;
    }
    SegmentKind::Other
}

/// The lowercased names of every `::routine` the file defines, so that a call
/// to one can be dropped rather than emitted into a row that does not carry
/// it.
fn collect_routine_names(blanked: &[&str]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in blanked {
        let trimmed = line.trim_start();
        let lower = trimmed.to_ascii_lowercase();
        let Some(rest) = lower.strip_prefix("::routine") else {
            continue;
        };
        let name = trimmed[trimmed.len() - rest.len()..]
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_matches(['"', '\'']);
        if !name.is_empty() {
            out.insert(name.to_ascii_lowercase());
        }
    }
    out
}

/// Whether the file carries a directive that makes reading an unassigned
/// symbol raise.
fn has_novalue_option(blanked: &[&str]) -> bool {
    blanked.iter().any(|line| {
        let lower = line.trim_start().to_ascii_lowercase();
        let Some(rest) = lower.strip_prefix("::options") else {
            return false;
        };
        rest.split_whitespace()
            .any(|word| word == "novalue" || word == "all")
    })
}

/// Every `.local~NAME =` in the file, from the whole file rather than from one
/// body: the assignment is in a `test000`-style setup method and the readers
/// are elsewhere.
fn collect_fixtures(original: &[&str], blanked: &[&str]) -> Fixtures {
    let mut out = Fixtures::default();
    for (line, blank) in original.iter().zip(blanked) {
        let trimmed = blank.trim_start();
        let lower = trimmed.to_ascii_lowercase();
        let Some(rest) = lower.strip_prefix(".local~") else {
            continue;
        };
        let name: String = rest.chars().take_while(|&c| is_symbol_char(c)).collect();
        if name.is_empty() {
            continue;
        }
        // The value is sliced out of the *original* line over positions found
        // in the blanked one, so a comment written inside the expression
        // survives while a comment written after it does not. A Rexx comment
        // ends a token without inserting a blank, so dropping one from the
        // middle of an expression changes what it means.
        let at = blank.len() - rest.len() + name.len();
        let Some(eq) = blank[at..].find('=') else {
            continue;
        };
        let value_start = at + eq + 1;
        let value = &blank[value_start..];
        let value_end = value_start + value.trim_end().len();
        let text = line[value_start..value_end].trim().to_string();
        if text.is_empty() {
            continue;
        }
        match out.values.get(&name) {
            Some(previous) if previous != &text => {
                out.ambiguous.insert(name);
            }
            _ => {
                out.values.insert(name, text);
            }
        }
    }
    for name in &out.ambiguous {
        out.values.remove(name);
    }
    out
}

/// The sequential scan of one `test`-prefixed body.
struct BodyScan<'a> {
    group: &'a str,
    method: &'a str,
    fixtures: &'a Fixtures,
    routines: &'a BTreeSet<String>,
    novalue: bool,
    digits: u32,
    form: Form,
    prelude: Vec<String>,
    /// Lowercased names this body has assigned, a stem carrying its trailing
    /// `.`. Consulted stem-first: see [`BodyScan::is_assigned`].
    assigned: BTreeSet<String>,
    expect: Option<RaiseExpectation>,
    /// Set once the body reaches a statement this scanner cannot carry. Rows
    /// emitted *before* it stand -- nothing about them was invalidated, only
    /// the state after that point stopped being trustworthy.
    blocked: Option<DropReason>,
    rows: Vec<AssertionRow>,
    raise: Option<RaiseRow>,
    losses: BTreeMap<DropReason, (usize, String)>,
}

impl<'a> BodyScan<'a> {
    fn new(
        group: &'a str,
        method: &'a str,
        fixtures: &'a Fixtures,
        routines: &'a BTreeSet<String>,
        novalue: bool,
    ) -> Self {
        BodyScan {
            group,
            method,
            fixtures,
            routines,
            novalue,
            digits: 9,
            form: Form::Scientific,
            prelude: Vec::new(),
            assigned: BTreeSet::new(),
            expect: None,
            blocked: None,
            rows: Vec::new(),
            raise: None,
            losses: BTreeMap::new(),
        }
    }

    fn lose(&mut self, reason: DropReason, count: usize, detail: &str) {
        let entry = self
            .losses
            .entry(reason)
            .or_insert_with(|| (0, detail.to_string()));
        entry.0 += count;
    }

    /// Records `reason` as what stops this body, unless something already
    /// does. The *first* blocker is the one reported, because it is the one
    /// that decides where the trustworthy prefix of the body ends.
    fn block(&mut self, reason: DropReason) {
        if self.blocked.is_none() {
            self.blocked = Some(reason);
        }
    }

    fn run(&mut self, original: &[&str], blanked: &[&str]) {
        let mut previous_continues = false;
        for (line, blank) in original.iter().zip(blanked) {
            let calls = count_assert_same(blank);
            let continues = ends_with_continuation(blank);
            if calls > 0 {
                let joined = previous_continues || continues;
                self.assertion_line(line, blank, calls, joined);
            } else {
                self.statement_line(blank);
            }
            previous_continues = continues;
        }
    }

    /// One line carrying at least one `self~assertSame`.
    fn assertion_line(&mut self, line: &str, blank: &str, calls: usize, joined: bool) {
        let detail = blank.trim().to_string();
        if let Some(reason) = self.blocked {
            self.lose(reason, calls, &detail);
            return;
        }
        if joined {
            self.block(DropReason::ContinuedLine);
            self.lose(DropReason::ContinuedLine, calls, &detail);
            return;
        }
        let lower = blank.to_ascii_lowercase();
        let trimmed_lower = lower.trim_start();
        if calls != 1 || !trimmed_lower.starts_with(ASSERT_SAME) {
            self.block(DropReason::NotAClause);
            self.lose(DropReason::NotAClause, calls, &detail);
            return;
        }

        let at = (blank.len() - trimmed_lower.len()) + ASSERT_SAME.len();
        let Some((left, right, _)) = parse_two_args(&blank[at..], &line[at..]) else {
            self.block(DropReason::UnparsedCallShape);
            self.lose(DropReason::UnparsedCallShape, 1, &detail);
            return;
        };
        let (left, right) = (left.to_string(), right.to_string());

        if let Some(expect) = self.expect {
            // The raise fires while `assertSame`'s own arguments are being
            // evaluated, so the call is never entered and `right` is not an
            // expected value. One RaiseRow per body: a second call after the
            // first raised would not run.
            if self.raise.is_none()
                && let Ok(first) = self.resolve(&left)
                && let Ok(second) = self.resolve(&right)
                && self.usable(&first, &second).is_none()
            {
                self.raise = Some(RaiseRow {
                    group: self.group.to_string(),
                    method: self.method.to_string(),
                    prelude: self.prelude.clone(),
                    operands: vec![first, second],
                    digits: self.digits,
                    form: self.form,
                    expect,
                });
            }
            self.lose(DropReason::ExpectedRaise, 1, &detail);
            return;
        }

        match (self.resolve(&left), self.resolve(&right)) {
            (Ok(expr), Ok(expected)) => {
                match self.usable(&expr, &expected) {
                    Some(reason) => self.lose(reason, 1, &detail),
                    None => self.rows.push(AssertionRow {
                        group: self.group.to_string(),
                        method: self.method.to_string(),
                        prelude: self.prelude.clone(),
                        expr,
                        expected,
                        digits: self.digits,
                        form: self.form,
                        expect_raise: None,
                    }),
                }
                // Whether or not this call became a row, evaluating its
                // arguments assigned, so nothing after it in the body stands on
                // the prelude any more. Blocking *after* emitting is the whole
                // point: `assertSame(17, value(n,'abc'))` is itself true, and it
                // is the assertions behind it that the assignment invalidates.
                if assigns_via_value(&left) || assigns_via_value(&right) {
                    self.block(DropReason::SideEffectingAssertion);
                }
            }
            (Err(reason), _) | (_, Err(reason)) => {
                // Not a blocker for the rest of the body: an unresolvable
                // operand says nothing about the state a later assertion runs
                // under.
                self.lose(reason, 1, &detail);
            }
        }
    }

    /// The reason this pair of resolved operands cannot become a row, or `None`
    /// if it can.
    fn usable(&self, expr: &str, expected: &str) -> Option<DropReason> {
        let carries = |needle: fn(&str) -> bool| {
            needle(expr) || needle(expected) || self.prelude.iter().any(|line| needle(line))
        };
        if carries(|text| text.contains(char::REPLACEMENT_CHARACTER)) {
            return Some(DropReason::NonUtf8Source);
        }
        if carries(clock_dependent) {
            return Some(DropReason::ClockDependent);
        }
        None
    }

    /// One line carrying no `self~assertSame`.
    fn statement_line(&mut self, blank: &str) {
        let trimmed = blank.trim();
        if trimmed.is_empty() {
            return;
        }
        let lower = trimmed.to_ascii_lowercase();

        if let Some(rest) = lower.strip_prefix("numeric digits") {
            match rest.trim().parse::<u32>() {
                Ok(n) if self.blocked.is_none() => self.digits = n,
                Ok(_) => {}
                Err(_) => self.block(DropReason::NumericNotConstant),
            }
            return;
        }
        if let Some(rest) = lower.strip_prefix("numeric form") {
            match rest.trim() {
                "scientific" if self.blocked.is_none() => self.form = Form::Scientific,
                "engineering" if self.blocked.is_none() => self.form = Form::Engineering,
                "scientific" | "engineering" => {}
                _ => self.block(DropReason::NumericNotConstant),
            }
            return;
        }
        if lower.starts_with("numeric fuzz") {
            // FUZZ only widens `=`/`<`/`>`'s tolerance; `assertSame` compares
            // with `==`, which FUZZ never affects, so there is no state here.
            return;
        }
        if let Some(rest) = lower.strip_prefix("self~expectsyntax") {
            match parse_raise_expectation(rest.trim()) {
                Some(expectation) if self.blocked.is_none() => self.expect = Some(expectation),
                Some(_) => {}
                None => self.block(DropReason::OtherExpectation),
            }
            return;
        }
        // Each of these defers a check to whatever raises next, exactly as
        // `expectSyntax` does, and this row schema has nowhere to carry a
        // bare condition name. Checked before the generic `self~assert*`
        // fallback below, which would otherwise treat them as inert and keep
        // emitting value rows for whatever follows -- the same gap the
        // `expectSyntax` rule exists to close.
        if lower.starts_with("self~expectcondition")
            || lower.starts_with("self~assertsyntaxerror")
            || lower.starts_with("self~assertruntimeerror")
        {
            self.block(DropReason::OtherExpectation);
            return;
        }
        if lower.starts_with("self~assert") || lower.starts_with("self~expect") {
            // Every other assertion kind: no variable assigned, no deferred
            // expectation set.
            return;
        }
        if lower.starts_with(".local~") {
            // A fixture definition. Already collected file-wide, and it binds
            // an environment symbol rather than a local, so there is nothing
            // to add to this body's prelude.
            return;
        }
        if contains_unquoted(blank, '~') {
            self.block(DropReason::MessageSend);
            return;
        }
        if self.blocked.is_some() {
            return;
        }
        match self.assignment(blank) {
            Ok(Some((name, resolved))) => {
                self.assigned.insert(name);
                self.prelude.push(resolved);
            }
            Ok(None) => self.block(DropReason::UnsupportedStatement),
            // The line *is* an assignment and its right-hand side is what
            // cannot be represented. Reporting that as an unsupported statement
            // would put an unresolvable fixture in the same bucket as a `DO`,
            // which is the one thing the per-reason breakdown exists to keep
            // apart.
            Err(reason) => self.block(reason),
        }
    }

    /// A whole-line single-clause assignment, with its right-hand side's
    /// fixtures already substituted.
    fn assignment(&self, blank: &str) -> Result<Option<(String, String)>, DropReason> {
        let is_name_char =
            |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '!' | '?');
        let text = blank.trim_end();
        let lead = text.len() - text.trim_start().len();
        let name: String = text[lead..]
            .chars()
            .take_while(|&c| is_name_char(c))
            .collect();
        if name.is_empty() {
            return Ok(None);
        }
        let after = &text[lead + name.len()..];
        let rest = after.trim_start();
        if !rest.starts_with('=') || rest.starts_with("==") {
            return Ok(None);
        }
        if contains_unquoted(text, ';') {
            return Ok(None); // a second clause this scanner cannot fold into one line
        }
        let value = rest[1..].trim();
        let resolved = self.resolve(value)?;
        Ok(Some((
            name.to_ascii_lowercase(),
            format!("{name} = {resolved}"),
        )))
    }

    /// Whether `name` -- a lowercased symbol read out of an expression -- is
    /// bound by something this body assigned.
    fn is_assigned(&self, name: &str) -> bool {
        if self.assigned.contains(name) {
            return true;
        }
        match name.find('.') {
            Some(at) => self.assigned.contains(&name[..=at]),
            None => false,
        }
    }

    /// `text` with every `.local` fixture it names substituted, or the reason
    /// it cannot become part of a row.
    fn resolve(&self, text: &str) -> Result<String, DropReason> {
        let substituted = self.substitute(text, FIXTURE_DEPTH)?;
        for run in symbol_runs(&substituted) {
            let name = substituted[run.start..run.end].to_ascii_lowercase();
            if run.is_call {
                if self.routines.contains(&name) {
                    return Err(DropReason::InFileRoutine);
                }
                continue;
            }
            if name.starts_with('.') || name.starts_with(|c: char| c.is_ascii_digit()) {
                // An environment symbol `substitute` left alone, or a constant
                // symbol. Neither is a variable read, so NOVALUE never fires
                // for one -- verified on the oracle: under `::options novalue
                // error`, `say .foo` prints `.FOO` and exits 0.
                continue;
            }
            if !self.is_assigned(&name) && self.novalue {
                return Err(DropReason::NoValueUnderOptions);
            }
        }
        Ok(substituted)
    }

    /// `text` with each `.NAME` the file's `.local` fixtures define replaced
    /// by that fixture's own source text, parenthesised.
    fn substitute(&self, text: &str, depth: usize) -> Result<String, DropReason> {
        let runs = symbol_runs(text);
        if runs
            .iter()
            .all(|run| !text[run.start..run.end].starts_with('.'))
        {
            return Ok(text.to_string());
        }
        if depth == 0 {
            return Err(DropReason::UnresolvedFixture);
        }
        let mut out = String::new();
        let mut at = 0usize;
        for run in runs {
            let token = &text[run.start..run.end];
            let Some(name) = token.strip_prefix('.') else {
                continue;
            };
            let name = name.to_ascii_lowercase();
            if name.is_empty() {
                continue;
            }
            if self.fixtures.ambiguous.contains(&name) {
                return Err(DropReason::UnresolvedFixture);
            }
            let value = match self.fixtures.values.get(&name) {
                Some(value) => value,
                None => {
                    // Not something this file sets, so it is whatever the
                    // interpreter makes of it -- and a lifted row makes the
                    // same thing of it, because nothing in the row's own
                    // program sets it either. A compound whose stem the file
                    // *does* set is a different matter: this scanner cannot
                    // say which tail is meant.
                    if let Some(stem) = name.split('.').next()
                        && stem != name
                        && (self.fixtures.values.contains_key(stem)
                            || self.fixtures.ambiguous.contains(stem))
                    {
                        return Err(DropReason::UnresolvedFixture);
                    }
                    continue;
                }
            };
            out.push_str(&text[at..run.start]);
            out.push('(');
            out.push_str(&self.substitute(value, depth - 1)?);
            out.push(')');
            at = run.end;
        }
        out.push_str(&text[at..]);
        Ok(out)
    }

    /// Folds this body's rows, its raise expectation and its per-reason losses
    /// into the file-level extraction.
    fn finish(self, out: &mut BifExtraction) {
        out.rows.extend(self.rows);
        if let Some(raise) = self.raise {
            out.raises.push(raise);
        }
        for (reason, (dropped, detail)) in self.losses {
            out.blocked.push(BlockedBody {
                group: self.group.to_string(),
                method: self.method.to_string(),
                reason,
                detail,
                dropped,
            });
        }
    }
}

/// One symbol run standing outside any string literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SymbolRun {
    start: usize,
    end: usize,
    /// The run is immediately followed by `(`, with no blank between, so it is
    /// a function call's name rather than a variable read. The blank matters:
    /// `f(x)` calls `f` while `f (x)` concatenates the *variable* `f` with
    /// `(x)`.
    is_call: bool,
}

/// The symbol runs in `text` that stand outside any string literal.
fn symbol_runs(text: &str) -> Vec<SymbolRun> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut at = 0usize;
    let mut just_closed_quote = false;
    while at < bytes.len() {
        let byte = bytes[at];
        if byte == b'\'' || byte == b'"' {
            at += 1;
            while at < bytes.len() {
                if bytes[at] == byte {
                    at += 1;
                    // A doubled quote closes and immediately reopens, which
                    // leaves the state where an escape rule would.
                    if at < bytes.len() && bytes[at] == byte {
                        at += 1;
                        continue;
                    }
                    break;
                }
                at += 1;
            }
            just_closed_quote = true;
            continue;
        }
        if !is_symbol_char(byte as char) {
            at += 1;
            just_closed_quote = false;
            continue;
        }
        let start = at;
        while at < bytes.len() && is_symbol_char(bytes[at] as char) {
            at += 1;
        }
        let is_suffix = just_closed_quote
            && at - start == 1
            && matches!(bytes[start], b'x' | b'X' | b'b' | b'B');
        just_closed_quote = false;
        if is_suffix {
            continue;
        }
        out.push(SymbolRun {
            start,
            end: at,
            is_call: bytes.get(at) == Some(&b'('),
        });
    }
    out
}

/// The top-level arguments of the call whose `(` stands at `open`, verbatim
/// and untrimmed, or `None` if the parenthesis never closes.
fn call_arguments(text: &str, open: usize) -> Option<Vec<&str>> {
    let bytes = text.as_bytes();
    if bytes.get(open) != Some(&b'(') {
        return None;
    }
    let mut args = Vec::new();
    let mut start = open + 1;
    let mut depth = 1i32;
    let mut in_str: Option<u8> = None;
    let mut at = open + 1;
    while at < bytes.len() {
        let byte = bytes[at];
        if let Some(quote) = in_str {
            if byte == quote {
                in_str = None;
            }
            at += 1;
            continue;
        }
        match byte {
            b'\'' | b'"' => in_str = Some(byte),
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    if at > open + 1 || !args.is_empty() {
                        args.push(&text[start..at]);
                    }
                    return Some(args);
                }
            }
            b',' if depth == 1 => {
                args.push(&text[start..at]);
                start = at + 1;
            }
            _ => {}
        }
        at += 1;
    }
    None
}

/// Whether `text` calls a builtin whose answer is not a function of its
/// arguments (decision D11).
fn clock_dependent(text: &str) -> bool {
    for run in symbol_runs(text) {
        if !run.is_call {
            continue;
        }
        let name = text[run.start..run.end].to_ascii_lowercase();
        let clocked = match name.as_str() {
            "random" => true,
            "date" | "time" => match call_arguments(text, run.end) {
                Some(args) => args.len() < 2 || args[1].trim().is_empty(),
                None => true,
            },
            _ => false,
        };
        if clocked {
            return true;
        }
    }
    false
}

/// Whether `text` calls `VALUE` in its **setter** form, which assigns to the
/// variable pool as a side effect of being evaluated.
fn assigns_via_value(text: &str) -> bool {
    symbol_runs(text).into_iter().any(|run| {
        run.is_call
            && text[run.start..run.end].eq_ignore_ascii_case("value")
            && call_arguments(text, run.end).is_some_and(|args| args.len() >= 2)
    })
}

/// Parses `self~expectSyntax`'s argument, already known (case-insensitively)
/// to follow that prefix: exactly `(major.sub)`.
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
