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
//! ```text
//! say '@@ASSERTSAME 3' ((A) == (B))
//! ```

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
    pub program: String,
    /// How many `self~assertSame` calls this program checks -- statically,
    /// as written. A loop can execute one of them many times.
    pub assertions: usize,
}

/// Why a `self~assertSame` occurrence is outside the extracted population.
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
}

/// The number of `self~assertSame` occurrences in `source`, case-insensitive
/// and **exact-spelling**: an occurrence followed by another symbol
/// character is a different method (`assertSameList`) and is not counted.
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

pub(crate) fn is_symbol_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Every `test`-prefixed method in `source` that carries at least one
/// `self~assertSame`, either as a rewritten [`KeywordBody`] or as a
/// [`BlockedBody`] accounting for the calls lost.
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
        // one into its `SAY`. This guard **fires** against the corpus rather
        // than standing by for a case that never comes -- see
        // [`DropReason::ContinuedLine`]'s own count in the committed drop
        // table.
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
        if contains_unquoted(&blank_calls(blanked, ASSERT_SAME), '~') {
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
pub(crate) fn parse_two_args<'a>(
    blanked: &str,
    text: &'a str,
) -> Option<(&'a str, &'a str, usize)> {
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
pub(crate) fn ends_with_continuation(line: &str) -> bool {
    matches!(line.trim_end().chars().last(), Some(',') | Some('-'))
}

/// Which kind of send blocks a whole body, given its comment-blanked text.
fn classify_sends(blanked: &str) -> DropReason {
    let mut saw_assert_same_list = false;
    for line in blanked.lines() {
        let without_calls = blank_calls(line, ASSERT_SAME);
        if !contains_unquoted(&without_calls, '~') {
            continue;
        }
        let without_list = blank_calls(&without_calls, ASSERT_SAME_LIST);
        if !contains_unquoted(&without_list, '~') {
            saw_assert_same_list = true;
            continue;
        }
        if contains_unquoted(&blank_self_assertions(&without_list), '~') {
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

/// Whether `line` contains `target` outside any string literal.
pub(crate) fn contains_unquoted(line: &str, target: char) -> bool {
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
            _ if c == target => return true,
            _ => {}
        }
    }
    false
}

/// `body` with every comment byte replaced by a space, newlines excepted.
pub(crate) fn blank_comments(body: &str) -> String {
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
