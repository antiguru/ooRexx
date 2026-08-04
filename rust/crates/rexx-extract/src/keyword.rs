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

//! The third extraction mode: whole method **bodies** from
//! `ootest/ooRexx/base/keyword`, rewritten into standalone programs.
//!
//! # Why neither existing mode transfers
//!
//! [`crate::extract`] lifts a method and wraps it as a routine; [`crate::
//! extract_assertions`] emits one row per `self~assertSame`, carrying an
//! *assignment prelude* forward from earlier in the same method. The second
//! is modelled on `base/expressions`, where an assertion's meaning is fixed
//! by a few preceding assignments and the `NUMERIC` settings.
//!
//! `base/keyword` tests **statements**. Its assertions sit after loops,
//! `IF`s, `SIGNAL`s and `PARSE`s -- exactly the statements
//! `extract_assertions`'s `simple_assignment` rejects by construction -- so
//! there is no prelude to carry, and the state an assertion depends on is
//! the whole body that ran before it. Measured, pointing the
//! `base/expressions` extractor at this group: **54 rows out of 2,561
//! prefix-matched calls**, and its own conservation assertion panics on the
//! second file (`ASSIGNMENT.testGroup: 0 rows + 39 dropped != 265`). Hence a
//! body-shaped extractor rather than a generalisation of the row-shaped one.
//!
//! # Two blind spots this mode does not inherit
//!
//! `extract_assertions` recognises an assertion only via
//! `trimmed.starts_with("self~assertsame")`, which misses every call that is
//! not first on its line. `base/expressions` never writes one; `base/keyword`
//! writes 403 of them, as multi-clause lines (`id=0010; ABBREV =0010;
//! self~assertSame(abbrev, id)`) and as `THEN`/`ELSE` targets (`When i=0
//! Then self~assertSame(...)`). This module finds a call anywhere on a line,
//! outside strings and comments, and requires only that it stand at a clause
//! boundary (see [`clause_boundary`]).
//!
//! That same prefix test also matches `self~assertSameList`, a **different**
//! method taking a list. `base/expressions` contains none;
//! `base/keyword` contains 120. [`count_assert_same`] and the scanner both
//! test the exact spelling, so an `assertSameList` is never mistaken for an
//! `assertSame` -- it simply leaves a `~` behind, which blocks its body like
//! any other message send.
//!
//! # What a rewritten body looks like
//!
//! Each `self~assertSame(A, B)` becomes one ordinary `SAY`:
//!
//! ```text
//! say '@@ASSERTSAME 3' ((A) == (B))
//! ```
//!
//! `OOREXXUNIT.CLS`'s `assertSame` is `if \ (expected == actual) then …
//! fail`, and Rexx `==` is exact-string identity, so the comparison is the
//! assertion, unchanged. Emitting it **unconditionally** rather than only on
//! failure is what makes the number of assertions that actually *executed*
//! observable: an assertion inside a loop prints once per pass, and a body
//! whose assertions never run prints nothing, which a consumer can tell
//! apart from a body that ran and passed. A conditional `IF` would report
//! neither, and would additionally rebind a following `ELSE` to itself when
//! the call it replaced was a `THEN` target.
//!
//! The two operands are parenthesised individually because concatenation
//! binds tighter than comparison in Rexx: appending to raw text that itself
//! contains a top-level comparison would regroup the expression instead of
//! comparing it. The whole comparison is parenthesised again so the blank
//! concatenation with the marker literal cannot capture part of it, and the
//! marker is a *string literal* rather than a symbol so the `(` that follows
//! can never be read as a function call's argument list.
//!
//! This needs no interpreter feature beyond `SAY`, `==` and the body's own
//! statements: no message dispatch, no `self`, and nothing from Phase 5.
//!
//! # What is excluded, and why nothing is silently deleted
//!
//! A body is extracted only when the **only** `~` left after rewriting is
//! none at all. In particular a body containing any *other* `self~assert*`
//! spelling is blocked rather than having those calls stripped: rewriting
//! them to `NOP` would admit more bodies only by deleting the checks they
//! were written to make, and then reporting them as passing. Every
//! assertion in an extracted body is one this module actually checks.
//!
//! That decision has a price, and it is measured rather than asserted in
//! prose: it is exactly [`DropReason::OtherAssertion`]'s own column in the
//! drop table, pinned by `tests/extract_keyword.rs`'s
//! `the_drop_reasons_account_for_every_call_outside_the_population`.

/// The exact method name this module recognises, lowercased.
const ASSERT_SAME: &str = "self~assertsame";

/// The near-miss that shares the whole of [`ASSERT_SAME`] as a prefix. A
/// different method taking a list, not modelled here, and counted under its
/// own [`DropReason`] so the shortfall it causes is visible rather than
/// merged into the general message-send bucket.
const ASSERT_SAME_LIST: &str = "self~assertsamelist";

/// The literal every rewritten assertion prints, followed by its 1-based
/// index within its own body and then `0` or `1`. A consumer reads one such
/// line per assertion *execution*, so a loop yields several.
pub const ASSERTION_MARKER: &str = "@@ASSERTSAME";

/// One `test`-prefixed method body, rewritten into a program that can run on
/// its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeywordBody {
    pub group: String,
    pub method: String,
    /// The method's own source, verbatim, with every `self~assertSame`
    /// replaced by its `SAY`. Comments are **kept** -- see [`rewrite_body`]
    /// for the measurement that says removing one changes what the code
    /// around it means.
    ///
    /// No `~` survives as *code*: a body where one would is blocked rather
    /// than extracted. One can still appear inside a string literal or a
    /// comment, where it is data.
    pub program: String,
    /// How many `self~assertSame` calls this program checks -- statically,
    /// as written. A loop can execute one of them many times.
    pub assertions: usize,
}

/// Why a `self~assertSame` occurrence is outside the extracted population.
///
/// A closed set rather than a free-text string, because these are the
/// accounting for `calls - rows`: reporting that difference as one bucket
/// says only that something was lost, while a counted breakdown says what
/// kind of thing and how much of each, and a category that starts growing
/// is then visible on its own rather than hidden inside a total that was
/// always going to be large.
///
/// Some variants stand at zero against `base/keyword` today. They are kept
/// and counted rather than dropped: a category pinned at zero fails loudly
/// the first time the corpus grows one, which is exactly when a reader
/// needs to know. Which ones are at zero is asserted in
/// `tests/extract_keyword.rs` rather than stated here, since that is a fact
/// about a checkout that can move under `svn up`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DropReason {
    /// In a `::method` whose name does not begin `test`, so [`crate::extract`]
    /// never yields its body and nothing downstream can see it at all.
    OutsideTestMethod,
    /// Inside a `/* */` or `--` comment: text that looks like a call and is
    /// not one. [`count_assert_same`] does not know what a comment is,
    /// deliberately, so these have to be subtracted somewhere.
    InsideComment,
    /// On a line joined to its neighbour by a trailing `,` or `-`, so the
    /// call is part of a larger clause and a `SAY` cannot stand in its
    /// place.
    ContinuedLine,
    /// The enclosing body's strongest blocker is `self~assertSameList`, a
    /// different method this module does not model. Split out because it is
    /// the shape that silently poisoned the row-shaped extractor -- there,
    /// a prefix test claimed each one and then rejected it, blocking every
    /// later assertion in the same method.
    ///
    /// **This reads zero against `base/keyword` for a reason worth knowing,
    /// and it is not "no body mixes the two".** Five methods do contain both
    /// an exact `assertSame` and an `assertSameList` (`DoOver`'s
    /// `test_do_over`, `DoWith`'s `test_do_with`, `LoopOver`'s
    /// `test_loop_over`, `LoopWith`'s `test_loop_with`, `REPLY`'s
    /// `test_reply_same_replyAssert`), and every one of them also sends a
    /// real message, so [`DropReason::MessageSend`] claims them first.
    AssertSameList,
    /// The enclosing body's only other sends are ooTest assertions of some
    /// **other** spelling -- `assertTrue`, `assertEquals`,
    /// `assertSyntaxError` and the rest. Distinct from
    /// [`DropReason::MessageSend`] because these bodies are plain classic
    /// Rexx otherwise, and are exactly the population that could be admitted
    /// by rewriting those calls to `NOP`. That is measured and declined: it
    /// would report a body as passing after deleting the checks it was
    /// written to make. This category is what that decision costs, stated as
    /// a number rather than left as a claim.
    OtherAssertion,
    /// The enclosing body sends a real message, which this module cannot run
    /// and will not delete. Unblocking these needs message dispatch, which
    /// is Phase 5's.
    MessageSend,
    /// An argument list this scanner has not seen -- not two or three
    /// arguments, or not closed on its own line.
    UnparsedCallShape,
    /// Used as an operand rather than standing as a clause of its own
    /// (`x = self~assertSame(...)`), so a `SAY` cannot replace it.
    NotAClause,
}

impl DropReason {
    /// A short stable label for reports and for the tests that pin the
    /// per-reason counts.
    pub fn label(self) -> &'static str {
        match self {
            DropReason::OutsideTestMethod => "outside any test-prefixed method",
            DropReason::InsideComment => "inside a comment, not a call",
            DropReason::ContinuedLine => "on a continued line",
            DropReason::AssertSameList => "body's only send is assertSameList",
            DropReason::OtherAssertion => "body uses another assert* spelling",
            DropReason::MessageSend => "body uses a message send",
            DropReason::UnparsedCallShape => "unparsed call shape",
            DropReason::NotAClause => "not a clause of its own",
        }
    }

    /// Every variant, so a caller reporting a breakdown lists the ones
    /// standing at zero too rather than only those it happened to hit.
    pub const ALL: &'static [DropReason] = &[
        DropReason::OutsideTestMethod,
        DropReason::InsideComment,
        DropReason::ContinuedLine,
        DropReason::AssertSameList,
        DropReason::OtherAssertion,
        DropReason::MessageSend,
        DropReason::UnparsedCallShape,
        DropReason::NotAClause,
    ];
}

/// One method (or one file's worth of stray text) whose `self~assertSame`
/// calls could not become a runnable body, and why.
///
/// Unlike [`crate::BlockedMethod`], this is **all or nothing**: a body runs
/// as one program, so a message send anywhere in it stops the assertions
/// before it from being checkable too.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockedBody {
    pub group: String,
    pub method: String,
    pub reason: DropReason,
    /// The offending source text, trimmed. Kept beside the category rather
    /// than folded into it: the category is what gets counted, and this is
    /// what a reader needs to find the line again.
    pub detail: String,
    pub dropped: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KeywordExtraction {
    pub bodies: Vec<KeywordBody>,
    pub blocked: Vec<BlockedBody>,
}

impl KeywordExtraction {
    /// `self~assertSame` calls carried by an extracted body. The unit the
    /// conservation law counts in, so that it is comparable with
    /// [`count_assert_same`] -- **not** the number of bodies.
    pub fn rows(&self) -> usize {
        self.bodies.iter().map(|b| b.assertions).sum()
    }

    /// `self~assertSame` occurrences that did not become part of any body.
    pub fn dropped(&self) -> usize {
        self.blocked.iter().map(|b| b.dropped).sum()
    }

    /// Occurrences dropped for one particular reason. Summing this over
    /// [`DropReason::ALL`] gives [`KeywordExtraction::dropped`] exactly,
    /// which is what makes the breakdown an accounting rather than a
    /// sample.
    pub fn dropped_for(&self, reason: DropReason) -> usize {
        self.blocked
            .iter()
            .filter(|b| b.reason == reason)
            .map(|b| b.dropped)
            .sum()
    }
}

/// The number of `self~assertSame` occurrences in `source`, case-insensitive
/// and **exact-spelling**: an occurrence followed by another symbol
/// character is a different method (`assertSameList`) and is not counted.
///
/// Deliberately a substring count that does not parse Rexx, does not know
/// what a method body is, and does not know what a comment is. It is the
/// independent denominator the conservation law is stated against, so it
/// must not share any judgement with the scanner it checks -- an occurrence
/// this counts and the scanner cannot use has to show up as a drop with a
/// reason, which is the whole point.
pub fn count_assert_same(source: &str) -> usize {
    let lower = source.to_ascii_lowercase();
    lower
        .match_indices(ASSERT_SAME)
        .filter(|(at, _)| {
            let after = &lower[at + ASSERT_SAME.len()..];
            !after.chars().next().is_some_and(is_symbol_char)
        })
        .count()
}

fn is_symbol_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Every `test`-prefixed method in `source` that carries at least one
/// `self~assertSame`, either as a rewritten [`KeywordBody`] or as a
/// [`BlockedBody`] accounting for the calls lost.
///
/// `group` is a caller-supplied label (typically the `.testGroup` file's
/// stem) copied verbatim into every entry; this function has no notion of
/// files.
///
/// Guarantees `rows() + dropped() == count_assert_same(source)`. The two
/// populations that make that more than an identity, both absent from
/// `base/expressions` and both present here, get their own drop reasons: an
/// occurrence inside a comment, and an occurrence in a method whose name
/// does not begin `test` (`GUARD.testGroup`'s `waiter_multiple` has two).
pub fn extract_keyword(group: &str, source: &str) -> KeywordExtraction {
    let mut out = KeywordExtraction::default();
    let mut accounted = 0usize;

    for method in crate::extract(source) {
        let raw = count_assert_same(&method.body);
        if raw == 0 {
            continue;
        }
        accounted += raw;

        let blanked = blank_comments(&method.body);
        let in_code = count_assert_same(&blanked);
        if in_code < raw {
            // Not calls at all: `TRACE.testGroup` quotes two of them inside
            // a block comment showing that method's own expected trace
            // output. Counted by `count_assert_same`, which does not know
            // what a comment is, so they need somewhere to go.
            out.blocked.push(BlockedBody {
                group: group.to_string(),
                method: method.name.clone(),
                reason: DropReason::InsideComment,
                detail: "an assertSame written inside a comment".to_string(),
                dropped: raw - in_code,
            });
        }
        if in_code == 0 {
            continue;
        }

        match rewrite_body(&method.body, &blanked) {
            Ok((program, assertions)) => out.bodies.push(KeywordBody {
                group: group.to_string(),
                method: method.name,
                program,
                assertions,
            }),
            Err((reason, detail)) => out.blocked.push(BlockedBody {
                group: group.to_string(),
                method: method.name,
                reason,
                detail,
                dropped: in_code,
            }),
        }
    }

    let calls = count_assert_same(source);
    if calls > accounted {
        out.blocked.push(BlockedBody {
            group: group.to_string(),
            method: "<outside any test-prefixed method>".to_string(),
            reason: DropReason::OutsideTestMethod,
            detail: "not inside a `test`-prefixed ::method, so `extract` never yields it"
                .to_string(),
            dropped: calls - accounted,
        });
    }

    out
}

/// Rewrites a method body into a standalone program, or says why it cannot
/// be one.
///
/// All-or-nothing, unlike the row-shaped extractor's per-assertion
/// blocking: the result runs as a single program, so one unrunnable line
/// takes the whole body with it.
///
/// **Two views of the same bytes.** `blanked` ([`blank_comments`]) is what
/// every structural decision is made against -- where the calls are, where
/// their arguments end, whether a `~` is code -- and `body` is what is
/// *emitted*, verbatim apart from the substitutions. The comments therefore
/// survive into the program, which is not cosmetic: a Rexx comment ends a
/// token **without** inserting a blank, so deleting one joins two tokens and
/// replacing one with a space concatenates them with a blank instead of
/// abutting them, and neither is what the source meant. Measured on the
/// oracle: `say '['1/**/05']'` prints `[105]` while `say '['1 /**/ 05']'`
/// prints `[1 05]`, and `zz = 1; say '['zz/**/05']'` prints `[105]`, so the
/// comment separates the tokens and contributes nothing between them.
/// `ITERATE.testGroup`'s `test_11` and `LEAVE.testGroup`'s `test_10` both
/// turn on exactly this (`(11/**/ 1/**irrelevant**/05  10/*...*/)`), and an
/// earlier draft that stripped comments to spaces made both of them
/// disagree with the oracle -- caught by running the rewritten programs
/// under both interpreters, not by reading the code.
///
/// The two views stay byte-aligned because `blank_comments` replaces each
/// comment byte with a space and leaves everything else, newlines included,
/// exactly where it was.
fn rewrite_body(body: &str, blanked: &str) -> Result<(String, usize), (DropReason, String)> {
    let mut program = String::new();
    let mut assertions = 0usize;
    let mut previous_continues = false;
    let all_blanked = blanked;
    for (line, blanked) in body.lines().zip(blanked.lines()) {
        // A call on a continued line is not a clause of its own, whichever
        // side the join is on: `clause_boundary` would see an empty prefix
        // and wave through a `SAY` spliced into the middle of someone else's
        // clause, and a call whose own line continues would swallow the next
        // one into its `SAY`. Zero calls in this group sit on either side of
        // a join today (measured) -- this is the guard that keeps that from
        // being an assumption the extractor silently depends on.
        let continues = ends_with_continuation(blanked);
        if count_assert_same(blanked) > 0 && (previous_continues || continues) {
            return Err((DropReason::ContinuedLine, line.trim().to_string()));
        }
        previous_continues = continues;

        // The message-send check runs on the *blanked* line with the calls
        // taken out, so it sees neither a `~` that is only a character in a
        // comment nor the `self~` of an assertion this module rewrites --
        // and still sees one hidden in an assertion's own argument, since
        // `blank_calls` removes only the call's name and parentheses, not
        // its operands.
        if unquoted(&blank_calls(blanked, ASSERT_SAME), '~').is_some() {
            // Which *kind* of send is asked of the whole body, not of this
            // line, even though this line is the one named in `detail`. The
            // question the category answers is "what would it take to
            // unblock this body", and the answer is the strongest blocker
            // anywhere in it: a body whose first offending line is an
            // `assertTrue` but which sends a real message ten lines later
            // is not unblocked by modelling assertions.
            return Err((classify_sends(all_blanked), line.trim().to_string()));
        }
        program.push_str(&rewrite_line(line, blanked, &mut assertions)?);
        program.push('\n');
    }
    Ok((program, assertions))
}

/// Rewrites every `self~assertSame` call on one line, advancing `index`.
///
/// `blanked` is `line` with its comments turned to spaces and is what every
/// position is found in; the emitted text always comes from `line`, so an
/// argument that contains a comment keeps it. See [`rewrite_body`] for why
/// that matters.
///
/// Line at a time because a Rexx clause is: a clause ends at end of line
/// unless continued, no string spans a line, and [`blank_comments`] has
/// already resolved every block comment that does. Measured across this
/// group: **zero** `assertSame` calls sit on a continued line, so nothing
/// needs a multi-line parse, and a call whose closing paren is not on its
/// own line is a shape this has never seen -- reported rather than guessed
/// at.
fn rewrite_line(
    line: &str,
    blanked: &str,
    index: &mut usize,
) -> Result<String, (DropReason, String)> {
    let lower = blanked.to_ascii_lowercase();
    let mut out = String::new();
    let mut emitted = 0usize;
    let mut pos = 0usize;
    let mut in_str: Option<char> = None;

    while pos < blanked.len() {
        let c = blanked[pos..]
            .chars()
            .next()
            .expect("pos is a char boundary inside blanked");
        let width = c.len_utf8();
        if let Some(quote) = in_str {
            // A doubled quote closes and immediately reopens, which leaves
            // the state exactly where an escape rule would; only in/out
            // matters here.
            if c == quote {
                in_str = None;
            }
            pos += width;
            continue;
        }
        if c == '\'' || c == '"' {
            in_str = Some(c);
            pos += width;
            continue;
        }
        if !lower[pos..].starts_with(ASSERT_SAME)
            || lower[pos + ASSERT_SAME.len()..]
                .chars()
                .next()
                .is_some_and(is_symbol_char)
        {
            pos += width;
            continue;
        }

        if !clause_boundary(&blanked[..pos]) {
            return Err((DropReason::NotAClause, line.trim().to_string()));
        }
        let at = pos + ASSERT_SAME.len();
        let Some((left, right, consumed)) = parse_two_args(&blanked[at..], &line[at..]) else {
            return Err((DropReason::UnparsedCallShape, line.trim().to_string()));
        };

        *index += 1;
        out.push_str(&line[emitted..pos]);
        out.push_str(&format!(
            "say '{ASSERTION_MARKER} {index}' (({left}) == ({right}))"
        ));
        pos = at + consumed;
        emitted = pos;
    }

    out.push_str(&line[emitted..]);
    Ok(out)
}

/// `blanked` with every exact-spelling occurrence of `name` turned to
/// spaces, so that the leftover-message-send check does not trip over the
/// `~` of a call the caller has already accounted for. Length-preserving,
/// like [`blank_comments`], and for the same reason.
///
/// "Exact-spelling" matters in both directions here. Blanking
/// [`ASSERT_SAME`] must not consume an `assertSameList`, or the two
/// categories would collapse into one; blanking [`ASSERT_SAME_LIST`] on a
/// line where [`ASSERT_SAME`] was already blanked is unaffected, since
/// nothing of the shorter name is left to match.
fn blank_calls(blanked: &str, name: &str) -> String {
    let lower = blanked.to_ascii_lowercase();
    let mut out = blanked.to_string();
    for (at, _) in lower.match_indices(name) {
        if lower[at + name.len()..]
            .chars()
            .next()
            .is_some_and(is_symbol_char)
        {
            continue;
        }
        out.replace_range(at..at + name.len(), &" ".repeat(name.len()));
    }
    out
}

/// Whether `before` -- the text preceding a call on its own line -- leaves
/// that call standing as a clause in its own right, so that replacing it
/// with a `SAY` instruction is legal where replacing a sub-expression would
/// not be.
///
/// Measured over all 2,441 exact-spelling calls in this group, every one is
/// preceded by exactly one of these: nothing (2,038), `;` (263), `THEN`
/// (128), a label's `:` (8), `ELSE` (2). The list is checked rather than
/// assumed, so a call used as an operand (`x = self~assertSame(...)`) blocks
/// its body instead of being turned into a syntactically invalid program.
fn clause_boundary(before: &str) -> bool {
    let before = before.trim_end();
    if before.is_empty() {
        return true;
    }
    if before.ends_with(';') || before.ends_with(':') {
        return true;
    }
    let last = before
        .rsplit(|c: char| c.is_whitespace())
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    matches!(last.as_str(), "then" | "else" | "otherwise")
}

/// Splits `(expected, actual)` or `(expected, actual, msg)` -- `rest` begins
/// at the `(` -- into the first two positional arguments verbatim, plus the
/// byte length consumed through the closing paren.
///
/// The signature is `use strict arg expected, actual, msg = ""`
/// (`framework/OOREXXUNIT.CLS`), so a third argument is legal and is a
/// failure-report message only: it is read and discarded here, never
/// compared. Two calls in this group pass one.
///
/// `None` for anything else: no `(` immediately after the method name (a
/// blank there means it is not an argument list at all), unbalanced parens
/// to the end of the line, fewer than two arguments, or more than three --
/// `use strict arg` would itself reject a fourth, so a call with one is not
/// a shape to guess at.
///
/// `blanked` is where the structure is found -- so a comma or a paren
/// inside a comment cannot be mistaken for one in the argument list -- and
/// `text` is the byte-aligned original the returned slices come from, so an
/// argument keeps any comment written inside it.
fn parse_two_args<'a>(blanked: &str, text: &'a str) -> Option<(&'a str, &'a str, usize)> {
    let mut chars = blanked.char_indices();
    if chars.next()?.1 != '(' {
        return None;
    }
    let mut depth = 1i32;
    let mut in_str: Option<char> = None;
    let mut commas: Vec<usize> = Vec::new();
    for (at, c) in chars {
        if let Some(quote) = in_str {
            if c == quote {
                in_str = None;
            }
            continue;
        }
        match c {
            '\'' | '"' => in_str = Some(c),
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    let first = *commas.first()?;
                    let second = commas.get(1).copied().unwrap_or(at);
                    return Some((
                        text[1..first].trim(),
                        text[first + 1..second].trim(),
                        at + c.len_utf8(),
                    ));
                }
            }
            ',' if depth == 1 => {
                if commas.len() == 2 {
                    return None; // a fourth argument
                }
                commas.push(at);
            }
            _ => {}
        }
    }
    None
}

/// Whether `line` ends with a Rexx line-continuation character, so that the
/// clause it belongs to does not end here.
///
/// **Both** `,` and `-`: ooRexx accepts each, differing only in whether a
/// blank is inserted at the join. Verified against the oracle rather than
/// read off documentation -- `say "a" -` / `"b"` prints `a b`, and `say 1 -`
/// / `+ 2` prints `3`, so a trailing `-` is a continuation and not a dangling
/// operator.
///
/// Testing the last character after trimming is enough to stay out of
/// strings: a line whose final token is a string literal ends with that
/// literal's own closing quote.
fn ends_with_continuation(line: &str) -> bool {
    matches!(line.trim_end().chars().last(), Some(',') | Some('-'))
}

/// Which kind of send blocks a whole body, given its comment-blanked text.
///
/// The strongest blocker present anywhere wins, because the category exists
/// to answer "what would unblock this body" and the weaker ones are moot
/// while a stronger one stands. In order:
///
/// * [`DropReason::MessageSend`] -- a real message send. Needs dispatch,
///   which is Phase 5's, and no amount of assertion modelling touches it.
/// * [`DropReason::AssertSameList`] -- otherwise, a `self~assertSameList`.
/// * [`DropReason::OtherAssertion`] -- otherwise, some other `self~assert*`
///   spelling, and nothing else. These bodies are plain classic Rexx apart
///   from assertions this module does not model.
///
/// Only reached once a `~` is known to survive the `assertSame` rewrite, so
/// the fall-through is a body with a send this cascade did not name; it
/// takes `MessageSend`, the conservative end.
fn classify_sends(blanked: &str) -> DropReason {
    let mut saw_assert_same_list = false;
    for line in blanked.lines() {
        let without_calls = blank_calls(line, ASSERT_SAME);
        if unquoted(&without_calls, '~').is_none() {
            continue;
        }
        let without_list = blank_calls(&without_calls, ASSERT_SAME_LIST);
        if unquoted(&without_list, '~').is_none() {
            saw_assert_same_list = true;
            continue;
        }
        if unquoted(&blank_self_assertions(&without_list), '~').is_some() {
            return DropReason::MessageSend;
        }
    }
    if saw_assert_same_list {
        DropReason::AssertSameList
    } else {
        DropReason::OtherAssertion
    }
}

/// `line` with every `self~assert…`/`self~expect…` send turned to spaces,
/// message name included, so that what is left is only the sends that are
/// *not* ooTest assertions.
///
/// The names are not enumerated. Anything after `self~` beginning `assert`
/// or `expect` counts, which is deliberate: `OOREXXUNIT.CLS` defines the
/// set and this repository cannot see it change, so a list here would be an
/// exhaustiveness claim over an enumeration living somewhere else. The
/// prefix rule needs no list and cannot go stale as the framework adds an
/// assertion.
fn blank_self_assertions(line: &str) -> String {
    const SELF: &str = "self~";
    let lower = line.to_ascii_lowercase();
    let mut out = line.to_string();
    for (at, _) in lower.match_indices(SELF) {
        let message = &lower[at + SELF.len()..];
        if !message.starts_with("assert") && !message.starts_with("expect") {
            continue;
        }
        let name_len: usize = message
            .chars()
            .take_while(|&c| is_symbol_char(c))
            .map(char::len_utf8)
            .sum();
        let end = at + SELF.len() + name_len;
        out.replace_range(at..end, &" ".repeat(end - at));
    }
    out
}

/// The first clause of `line` containing `target` outside any string, or
/// `None`. Used to name the offending text in a block reason rather than
/// only reporting that something offended.
fn unquoted(line: &str, target: char) -> Option<&str> {
    let mut in_str: Option<char> = None;
    for c in line.chars() {
        if let Some(quote) = in_str {
            if c == quote {
                in_str = None;
            }
            continue;
        }
        match c {
            '\'' | '"' => in_str = Some(c),
            _ if c == target => return Some(line.trim()),
            _ => {}
        }
    }
    None
}

/// `body` with every comment byte replaced by a space, newlines excepted.
///
/// Both forms: `/* … */`, which **nests** in Rexx, and `--` to end of line.
///
/// Blanking rather than removing, and byte for byte, so that the result is
/// exactly as long as its input and every offset found in it addresses the
/// same character of the original. That is what lets one pass decide
/// structure from this view and emit text from the untouched one, which
/// [`rewrite_body`] needs because a comment is not something a rewriter may
/// silently resolve on the interpreter's behalf.
///
/// Since every replaced byte becomes an ASCII space and every other byte is
/// left alone, the result is valid UTF-8 whatever the comment contained.
/// Works on an owned byte copy because `String` has no safe mutable byte
/// view (`str::as_bytes_mut` is `unsafe`, which this workspace forbids). The
/// `from_utf8` at the end cannot fail: a comment region is blanked in full,
/// so no multi-byte sequence is ever half-replaced, and every byte written
/// is an ASCII space.
fn blank_comments(body: &str) -> String {
    let source = body.as_bytes();
    let mut out = source.to_vec();
    let mut depth = 0usize;
    let mut in_str: Option<u8> = None;
    let mut at = 0usize;

    while at < source.len() {
        let byte = source[at];
        let next = source.get(at + 1).copied();
        if depth > 0 {
            let step = match (byte, next) {
                (b'*', Some(b'/')) => {
                    depth -= 1;
                    2
                }
                (b'/', Some(b'*')) => {
                    depth += 1;
                    2
                }
                _ => 1,
            };
            for blank_at in at..(at + step).min(source.len()) {
                if source[blank_at] != b'\n' {
                    out[blank_at] = b' ';
                }
            }
            at += step;
            continue;
        }
        match (in_str, byte, next) {
            (Some(quote), _, _) => {
                if byte == quote {
                    in_str = None;
                }
                at += 1;
            }
            (None, b'\'' | b'"', _) => {
                in_str = Some(byte);
                at += 1;
            }
            (None, b'/', Some(b'*')) => {
                depth = 1;
                out[at] = b' ';
                out[at + 1] = b' ';
                at += 2;
            }
            (None, b'-', Some(b'-')) => {
                while at < source.len() && source[at] != b'\n' {
                    out[at] = b' ';
                    at += 1;
                }
            }
            _ => at += 1,
        }
    }
    String::from_utf8(out).expect("only whole comment regions are replaced, and only by spaces")
}
