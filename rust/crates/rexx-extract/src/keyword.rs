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
//! spelling is blocked rather than having those calls stripped. Rewriting
//! them to `NOP` instead would admit 138 more methods carrying 169 more
//! `assertSame` calls, but only by deleting 659 real checks from bodies then
//! reported as passing. Every assertion in an extracted body is one this
//! module actually checks.

/// The exact method name this module recognises, lowercased.
const ASSERT_SAME: &str = "self~assertsame";

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
    /// The rewritten body, comments removed and every `self~assertSame`
    /// replaced by its `SAY`. Contains no `~` at all.
    pub program: String,
    /// How many `self~assertSame` calls this program checks -- statically,
    /// as written. A loop can execute one of them many times.
    pub assertions: usize,
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
    pub reason: String,
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
                reason: "occurrence inside a comment, not a call".to_string(),
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
            Err(reason) => out.blocked.push(BlockedBody {
                group: group.to_string(),
                method: method.name,
                reason,
                dropped: in_code,
            }),
        }
    }

    let calls = count_assert_same(source);
    if calls > accounted {
        out.blocked.push(BlockedBody {
            group: group.to_string(),
            method: "<outside any test-prefixed method>".to_string(),
            reason: "not inside a `test`-prefixed ::method, so `extract` never yields it"
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
fn rewrite_body(body: &str, blanked: &str) -> Result<(String, usize), String> {
    let mut program = String::new();
    let mut assertions = 0usize;
    let mut previous_continues = false;
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
            return Err(format!(
                "assertSame on a continued line, so it is not a clause of its own: {}",
                line.trim()
            ));
        }
        previous_continues = continues;

        // The message-send check runs on the *blanked* line with the calls
        // taken out, so it sees neither a `~` that is only a character in a
        // comment nor the `self~` of an assertion this module rewrites --
        // and still sees one hidden in an assertion's own argument, since
        // `blank_calls` removes only the call's name and parentheses, not
        // its operands.
        if let Some(offender) = unquoted(&blank_calls(blanked), '~') {
            return Err(format!("message send this body cannot run: {offender}"));
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
fn rewrite_line(line: &str, blanked: &str, index: &mut usize) -> Result<String, String> {
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
            return Err(format!(
                "assertSame is not a clause of its own, so a SAY cannot stand in its place: \
                 {}",
                line.trim()
            ));
        }
        let at = pos + ASSERT_SAME.len();
        let Some((left, right, consumed)) = parse_two_args(&blanked[at..], &line[at..]) else {
            return Err(format!(
                "assertSame call shape this scanner has not seen: {}",
                line.trim()
            ));
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

/// `blanked` with every exact-spelling `self~assertSame` turned to spaces,
/// so that the leftover-message-send check does not trip over the `~` of the
/// very calls this module rewrites away. Length-preserving, like
/// [`blank_comments`], and for the same reason.
fn blank_calls(blanked: &str) -> String {
    let lower = blanked.to_ascii_lowercase();
    let mut out = blanked.to_string();
    for (at, _) in lower.match_indices(ASSERT_SAME) {
        if lower[at + ASSERT_SAME.len()..]
            .chars()
            .next()
            .is_some_and(is_symbol_char)
        {
            continue;
        }
        out.replace_range(at..at + ASSERT_SAME.len(), &" ".repeat(ASSERT_SAME.len()));
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
