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

//! The `PARSE` template engine: a byte string and a template in, assignments
//! out.
//!
//! Two layers, and the split is the design rather than tidiness. [`Cursor`]
//! is the whole of the *movement* rule -- the five positions below and the
//! operations that move them -- and it knows nothing about expressions,
//! tracing, variables or where its string came from, so it is unit-testable
//! against measured oracle bytes with no `Interp` in sight. [`Interp::
//! exec_parse`] is the driver: it evaluates trigger operands, emits the trace
//! lines, and assigns the targets. A *source* is one arm of
//! [`Interp::parse_strings`] and touches neither layer's logic -- including
//! the two that read a line rather than evaluating a value, `PARSE PULL` and
//! `PARSE LINEIN`, whose whole contribution here is calling one of
//! `input.rs`'s two readers and handing back what it returned.
//!
//! # The five positions
//!
//! `start`/`end` bound the section the current trigger's targets are carved
//! out of. `pattern_start`/`pattern_end` bound the *match* the last trigger
//! made -- equal for every numeric trigger, `needle.len()` apart for a string
//! one -- and they are what the next trigger measures from. `subcurrent` is
//! how far into `start..end` the word-by-word carving has got.
//!
//! Two positions rather than one is the whole reason `-n` and `<n` are
//! unrelated operations rather than variants (measured, source
//! `'abcdefghij'`):
//!
//! ```text
//! parse value 'abcdefghij' with p 5 q -2 r  ->  [abcd][efghij][cdefghij]
//! parse value 'abcdefghij' with p 5 q <2 r  ->  [abcd][cd]    [cdefghij]
//! ```
//!
//! `-n` moves the *match* position back and hands the target everything from
//! the old match to the end of the string; `<n` hands the target exactly the
//! `n` bytes ending at the match position. Same movement, unrelated
//! assignment.
//!
//! It is also why a relative trigger *after a string pattern* measures from
//! the match's start while the next target begins after the match's end:
//! measured, `p 'c' q` and `p 'c' +1 q` are identical, and `p 'c' -1 q` gives
//! `q = 'bcdefghij'`.
//!
//! # Targets belong to the trigger they precede
//!
//! `rexx-parse` accumulates targets and attaches them to the *next* trigger
//! (`instruction.rs`'s `parse_template`), emitting a trailing `End` trigger
//! for the ones with no trigger after them. So in `p 5 q -2 r`, `p` is
//! assigned when `5` fires, `q` when `-2` fires, and `r` by the `End`
//! trigger. Reading it the other way round -- `q` attached to `5` -- produces
//! a wrong answer that looks right on `p 5 q` and parts company immediately
//! after; the trace cannot tell the two apart, because an `End` trigger emits
//! no line of its own, so this was settled by value rather than by
//! transcript.

use crate::error::{Failure, Raised};
use crate::{Code, Interp, Loud};
use rexx_core::ObjRef;
use rexx_parse::{ExprKind, Parse, ParseSource, ParseTrigger, TriggerKind};

/// The platform name `PARSE SOURCE`'s first word carries.
///
/// Measured on this host: `parse source` answers `LINUX COMMAND <path>` with
/// the program's own absolute path. Which word other platforms use was not
/// measured, since one machine cannot show it.
const PLATFORM: &[u8] = b"LINUX";

/// `PARSE VERSION`'s string.
///
/// **This is the oracle's own build identity, and it is a claim about a
/// binary rather than about this crate.** Measured 2026-08-05 against the
/// `build/` present then: interpreter name and version, then the language
/// level, then the interpreter's *build date*. Nothing here can derive the
/// third field, so it is recorded verbatim from the measurement.
///
/// **`tests/parse_version_oracle.rs` is the only thing that can notice this
/// going stale, and it has to run the oracle to do it.** A rebuilt oracle
/// moves the build date with nothing in this crate changing, so that harness
/// runs `parse version` through both interpreters and compares -- gated on
/// `REXX_CORPUS_GATE`, since its whole subject is the oracle. An assertion
/// comparing this constant against a literal copy of itself would go green
/// through every rebuild there has ever been; that shape was here and is gone.
///
/// No corpus program prints it: a committed differential over a build date
/// would break on the next rebuild and say nothing about this engine, and the
/// corpus's own determinism rule excludes it anyway.
const VERSION: &[u8] = b"REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026";

/// The two bytes `PARSE` treats as whitespace when carving a section into
/// words: blank and tab, and nothing else (`RexxTarget::getWord`,
/// `instructions/ParseTarget.cpp:423`, `*scan == ' ' || *scan == '\t'`).
fn is_blank(byte: u8) -> bool {
    byte == b' ' || byte == b'\t'
}

/// Where one `PARSE` clause's templates get their strings.
///
/// **Every source but `ARG` produces exactly one string**, and carrying that
/// one string in a `Vec` cost an allocation per clause for a container that
/// was never asked to hold a second element. Measured with `heaptrack` on
/// `samples/rexxcps.rex`, whose loop parses on every iteration, removing it
/// took the program from 505,446 allocations to 466,246.
///
/// `ARG` is the one source with more than one string, and it **names the
/// argument each template will read rather than rendering them all up
/// front**. A template's string is finished with before the next one starts,
/// so at most one is live at a time and the rest of
/// [`Interp::parse_buffers`] stays available to a nested `PARSE`; a template
/// list shorter than the argument list renders nothing for the arguments it
/// never reaches; and there is no outer vector to lend, return or bound.
enum ParseStrings {
    /// A single-string source. The slot empties on the first take, so a
    /// template past the first parses the null string -- which is the rule
    /// [`Interp::next_template`] wants and gets here for free.
    One(Option<Vec<u8>>),
    /// `PARSE ARG`: the index of the argument the next template reads.
    Arg(usize),
}

/// One template's parse string and the five positions the triggers move.
///
/// Owns its string because the comma fence replaces it wholesale and because
/// `UPPER`/`LOWER` transform it on the way in, so there is nothing outside to
/// borrow from that lives long enough.
pub(crate) struct Cursor {
    string: Vec<u8>,
    length: usize,
    /// The start of the section this trigger's targets are carved from.
    start: usize,
    /// One past its end.
    end: usize,
    /// Where the last match began. Every *relative* trigger measures from
    /// here.
    pattern_start: usize,
    /// One past where the last match ended -- the same position for a numeric
    /// trigger, `needle.len()` further on for a string one. Where the next
    /// *section* and the next string search begin.
    pattern_end: usize,
    /// How far into `start..end` the word carving has got.
    subcurrent: usize,
}

impl Cursor {
    /// The string back, for a caller returning it to a pool.
    pub(crate) fn into_string(self) -> Vec<u8> {
        self.string
    }

    /// A fresh cursor over `string`, every position at the origin
    /// (`RexxTarget::next`'s own reset).
    pub(crate) fn new(string: Vec<u8>) -> Cursor {
        let length = string.len();
        Cursor {
            string,
            length,
            start: 0,
            end: 0,
            pattern_start: 0,
            pattern_end: 0,
            subcurrent: 0,
        }
    }

    /// The string being parsed, for the caller that has to trace it.
    pub(crate) fn string(&self) -> &[u8] {
        &self.string
    }

    /// The implicit trailing trigger: the section runs from the end of the
    /// last match to the end of the string.
    fn move_to_end(&mut self) {
        self.start = self.pattern_end;
        self.pattern_end = self.length;
        self.pattern_start = self.length;
        self.end = self.length;
        self.subcurrent = self.start;
    }

    /// `+n`. Forward from the last match's start, **with the Rexx
    /// no-movement rule**: if the new position is not past the old one the
    /// section runs to the end of the string instead and the match position
    /// does not move. Measured: `p +0 q` gives both the whole string.
    fn forward(&mut self, offset: usize) {
        self.start = self.pattern_start;
        self.end = self.start.saturating_add(offset).min(self.length);
        if self.end <= self.start {
            self.end = self.length;
            self.pattern_start = self.start;
        } else {
            self.pattern_start = self.end;
        }
        self.pattern_end = self.pattern_start;
        self.subcurrent = self.start;
    }

    /// `>n`. The same movement as `+n` **without** the no-movement rule, so
    /// the section is an exact slice and an offset of zero gives the null
    /// string. Measured: `p >0 q` gives `p = ''`.
    fn forward_length(&mut self, offset: usize) {
        self.start = self.pattern_start;
        self.end = self.start.saturating_add(offset).min(self.length);
        self.pattern_start = self.end;
        self.pattern_end = self.pattern_start;
        self.subcurrent = self.start;
    }

    /// `=n`, and a bare numeric symbol. Column `n` is origin one, so a zero
    /// and a one are the same position -- measured, `p 0 q` and `p 1 q` both
    /// give the whole string twice.
    ///
    /// Shares `+n`'s rule about direction: forward of the current position
    /// gives the target `[current, new)`, and anything else gives it
    /// `[current, END]`. Equal counts as backward, measured: `p 5 q 5 r`
    /// gives `r = 'efghij'`, not the null string.
    fn absolute(&mut self, column: usize) {
        let offset = column.saturating_sub(1);
        self.start = self.pattern_end;
        if offset <= self.start {
            self.end = self.length;
            self.pattern_start = offset;
        } else {
            self.end = offset.min(self.length);
            self.pattern_start = self.end;
        }
        self.pattern_end = self.pattern_start;
        self.subcurrent = self.start;
    }

    /// `-n`. The match position moves back, clamped at the origin, and the
    /// section runs from the *old* match position to the end of the string.
    /// So backward movement never assigns the null string -- measured, `p 5 q
    /// -99 r` gives `r` the whole string.
    fn backward(&mut self, offset: usize) {
        self.start = self.pattern_start;
        self.end = self.length;
        self.pattern_start = self.pattern_start.saturating_sub(offset);
        self.pattern_end = self.pattern_start;
        self.subcurrent = self.start;
    }

    /// `<n`. The `n` bytes *ending at* the match position, clamped at the
    /// origin. Not `-n` with a different sign: this is an exact slice and
    /// `p <0 q` gives `p = ''`.
    fn backward_length(&mut self, offset: usize) {
        self.start = self.pattern_start.saturating_sub(offset);
        self.end = self.pattern_start;
        self.pattern_start = self.start;
        self.pattern_end = self.start;
        self.subcurrent = self.start;
    }

    /// A literal or `(expr)` pattern. Searches from the end of the last
    /// match, so searches are non-overlapping.
    ///
    /// **An absent pattern matches at END**: the section becomes the whole
    /// remainder and the match position goes to the end, so a following
    /// target gets the null string. Measured, `p 'z' q` gives `p` the whole
    /// string and `q = ''`.
    ///
    /// **The empty pattern behaves as absent**, measured (`p '' q` is
    /// identical to `p 'z' q`) -- which is why this cannot be a plain
    /// substring search, where an empty needle matches at position zero.
    fn search(&mut self, needle: &[u8]) {
        self.match_at(
            find(&self.string, needle, self.pattern_end, false),
            needle.len(),
        );
    }

    /// [`search`] under `PARSE CASELESS`.
    ///
    /// **ASCII letters only**, verified against a byte alphabet: pattern
    /// `'e9'x` does not match a source byte `'c9'x` and the reverse does not
    /// either, `'5b'x` does not match `'7b'x`, and `'3f'x` does not match
    /// `'5f'x` -- every byte at or above `0x80`, and every non-letter, matches
    /// only itself.
    ///
    /// [`search`]: Cursor::search
    fn caseless_search(&mut self, needle: &[u8]) {
        self.match_at(
            find(&self.string, needle, self.pattern_end, true),
            needle.len(),
        );
    }

    /// The half of a string search that is not the comparison: where the two
    /// searches put the five positions once they know whether they matched.
    fn match_at(&mut self, found: Option<usize>, needle_length: usize) {
        self.start = self.pattern_end;
        match found {
            Some(at) => {
                self.end = at;
                self.pattern_start = at;
                self.pattern_end = at + needle_length;
            }
            None => {
                self.end = self.length;
                self.pattern_start = self.length;
                self.pattern_end = self.length;
            }
        }
        self.subcurrent = self.start;
    }

    /// The next blank-delimited word of the current section, as a range into
    /// [`string`], or an empty range once the section is used up.
    ///
    /// **The leading-blank skip is bounded by the whole string, not by the
    /// section's end**, which is `getWord`'s own shape rather than an
    /// oversight: the C++ scans for a non-blank relying on the string's
    /// terminating NUL and only then tests the section end
    /// (`ParseTarget.cpp:423`-`433`).
    ///
    /// [`string`]: Cursor::string
    fn next_word(&mut self) -> std::ops::Range<usize> {
        if self.subcurrent >= self.end {
            return 0..0;
        }
        let mut scan = self.subcurrent;
        while scan < self.length && is_blank(self.string[scan]) {
            scan += 1;
        }
        self.subcurrent = scan;
        if self.subcurrent >= self.end {
            return 0..0;
        }
        let word_end = (self.subcurrent..self.end).find(|&i| is_blank(self.string[i]));
        match word_end {
            // No blank before the section's end: the rest of the section is
            // the word.
            None => {
                let word = self.subcurrent..self.end;
                self.subcurrent = self.end;
                word
            }
            // The terminating blank is consumed as well, so the next scan
            // starts past it.
            Some(at) => {
                let word = self.subcurrent..at;
                self.subcurrent = at + 1;
                word
            }
        }
    }

    /// Everything left in the current section, leading blanks included -- what
    /// the *last* target of a trigger gets. Measured: `p q r` on `'a  b  c'`
    /// gives `r = ' c'`, so only the final target keeps them.
    fn remainder(&mut self) -> std::ops::Range<usize> {
        if self.subcurrent >= self.end {
            return 0..0;
        }
        let rest = self.subcurrent..self.end;
        self.subcurrent = self.end;
        rest
    }
}

/// The first offset at or after `from` where `needle` occurs in `haystack`,
/// or `None`.
///
/// An **empty needle never matches**, which is the measured behaviour of an
/// empty pattern (see [`Cursor::search`]) and the one place this differs from
/// an ordinary substring search.
fn find(haystack: &[u8], needle: &[u8], from: usize, caseless: bool) -> Option<usize> {
    if needle.is_empty() || from > haystack.len() {
        return None;
    }
    // Searched as one slice from `from` rather than by indexing `haystack` at
    // each candidate offset: `windows` and `iter` both carry their own bound,
    // where `&haystack[at..at + needle.len()]` is a range check per position.
    let tail = &haystack[from..];
    if needle.len() > tail.len() {
        return None;
    }
    let at = match (needle, caseless) {
        // A one-byte pattern is the common case and the one a byte scan can
        // do without building a window per position -- `samples/rexxcps.rex`
        // searches for a single `b` on four of its inner loop's clauses.
        // `find_byte` is `builtin::string`'s own scan, a word at a time; its
        // doc has the argument for why the lowest flagged byte is the first
        // match, and the test that runs that argument rather than resting on
        // it.
        ([byte], false) => crate::builtin::string::find_byte(tail, *byte),
        ([byte], true) => {
            let folded = byte.to_ascii_lowercase();
            tail.iter()
                .position(|candidate| candidate.to_ascii_lowercase() == folded)
        }
        (_, false) => tail.windows(needle.len()).position(|w| w == needle),
        (_, true) => tail
            .windows(needle.len())
            .position(|w| w.eq_ignore_ascii_case(needle)),
    };
    at.map(|at| at + from)
}

/// How many source buffers [`Interp::parse_buffers`] parks between `PARSE`
/// instructions, and the widest one it will park.
const PARSE_BUFFERS_KEPT: usize = 8;
const PARSE_BUFFER_BYTES_KEPT: usize = 4096;

impl Interp {
    /// An empty buffer for a `PARSE` source string. See
    /// [`Interp::parse_buffers`].
    fn take_parse_buffer(&mut self) -> Vec<u8> {
        self.parse_buffers.pop().unwrap_or_default()
    }

    /// `value`'s text in a buffer from the pool.
    ///
    /// The buffer is taken **before** the render, because
    /// [`Interp::to_text`] hands back a borrow of `self` and nothing else may
    /// touch the interpreter while it is live.
    fn rendered_into_parse_buffer(&mut self, value: ObjRef) -> Vec<u8> {
        let mut buffer = self.take_parse_buffer();
        buffer.extend_from_slice(&self.to_text(value));
        buffer
    }

    /// Hands a source buffer back, if it is one worth keeping.
    fn give_parse_buffer(&mut self, mut buffer: Vec<u8>) {
        if buffer.capacity() == 0
            || buffer.capacity() > PARSE_BUFFER_BYTES_KEPT
            || self.parse_buffers.len() >= PARSE_BUFFERS_KEPT
        {
            return;
        }
        buffer.clear();
        self.parse_buffers.push(buffer);
    }

    /// Hands back the string a template walk did not consume.
    ///
    /// `Arg` holds no buffer of its own -- it renders each argument when the
    /// template that reads it starts, into a buffer the cursor then owns -- so
    /// a walk that stopped short of the arguments has nothing here to return.
    fn give_parse_strings(&mut self, strings: ParseStrings) {
        match strings {
            ParseStrings::One(Some(string)) => self.give_parse_buffer(string),
            ParseStrings::One(None) | ParseStrings::Arg(_) => {}
        }
    }

    /// `PARSE ARG`'s string for the template at argument position `at`, in a
    /// buffer from the pool.
    ///
    /// An omitted position (`call sub 1,,3`) and a position past the last
    /// argument are both the null string, which is the rule
    /// [`Interp::parse_strings`] records the measurement for.
    ///
    /// **The value is read here rather than at the clause's start, so an
    /// argument a later template reads has to survive everything the earlier
    /// templates allocate.** It does, because an argument's value is rooted as
    /// a temporary of the *caller*, below every watermark this activation's
    /// clauses take, and so stays reachable for as long as the callee runs.
    /// `a_late_parse_arg_template_survives_collection` runs that under
    /// collect-on-every-allocation rather than arguing it; removing the
    /// caller-side `push_temp` it names makes that test panic.
    ///
    /// A trigger operand that calls a routine replaces `call_context` and puts
    /// it back, so the arguments a later template reads are still this
    /// activation's own.
    fn argument_text(&mut self, at: usize) -> Result<Vec<u8>, Failure> {
        let argument = match self.call_context.arguments.get(at) {
            Some(Some(argument)) => Some(argument.value()),
            Some(None) | None => None,
        };
        match argument {
            // `PARSE ARG` converts each argument through the same
            // `ParseTarget::init` the other sources reach, and traces no
            // `>K>` of its own to disagree with.
            Some(value) => {
                let value = self.required_string_value(value)?;
                Ok(self.rendered_into_parse_buffer(value))
            }
            None => Ok(self.take_parse_buffer()),
        }
    }

    /// One `PARSE` instruction: resolve the source, then walk the template.
    ///
    /// The trace shape, all of it measured (`trace i` unless a line is said
    /// to be results-only):
    ///
    /// | construct | lines, in order |
    /// |---|---|
    /// | source `VALUE` | `>L>` from the expression, then `>K> "VALUE" => "<src>"`, then `>>> "<src>"` |
    /// | source `VAR` | `>C>`/`>V>` from the read, then `>K> "VAR" => "<src>"`, then `>>>` |
    /// | source `SOURCE`/`VERSION` | `>K> "<kw>" => "<src>"`, then `>>>` |
    /// | source `ARG` | **no `>K>` at all** -- straight to `>>>` |
    /// | a positional trigger | `>L> "<n>"` then `>>> "<n>"`, **before** the preceding target is assigned |
    /// | `String`/`Mixed` | the same pair; caseless is not distinguishable in the trace |
    /// | an assigned target | `>=> NAME <= "<value>"`, or `>>> "<value>"` under `TRACE R` |
    /// | a `.` placeholder | `>.> "<consumed>"`, emitted even when it consumed nothing |
    /// | `End` | nothing |
    /// | the comma fence | `>>> "<next template's source>"` |
    ///
    /// **A target's own line is a choice between two prefixes, not two
    /// independent gates**, which is the one thing a `trace i` survey cannot
    /// see and this crate got told about only by measuring `trace r`:
    /// `ParseTrigger::parse` (`instructions/ParseTrigger.cpp:274`-`280`)
    /// assigns -- whose own `traceAssignment` is the `intermediates`-gated
    /// `>=>` -- and then emits `traceResult` only `if
    /// (!context->tracingIntermediates())`. Measured both ways on one
    /// program: `parse value 'abcdefghij' with p1 5 q1` under `trace i` traces
    /// `>=> P1 <= "abcd"` and `>=> Q1 <= "efghij"`, and under `trace r` traces
    /// `>>> "abcd"` and `>>> "efghij"` in their place.
    ///
    /// A trigger's operand is evaluated **before** the preceding target is
    /// assigned, measured: for `p 5 q -2 r` the order is `>L>"5"`, `>>>"5"`,
    /// `>=>P`, `>L>"2"`, `>>>"2"`, `>=>Q`, `>=>R`. The traced literal for
    /// `+3`/`-2`/`>3`/`<2` is the bare number without the sign, because that
    /// is the operand expression `rexx-parse` recorded.
    ///
    /// `evaluated` is `PARSE VALUE`'s source, for the caller that has already
    /// computed it: the compiled stream evaluates that expression into a
    /// register of its own, so [`super::ir::Op::Parse`] hands the value in
    /// where `step` passes `None` and this evaluates it. Every other source
    /// ignores the argument, having no expression to be handed.
    pub(crate) fn exec_parse(
        &mut self,
        code: &Code<'_>,
        parse: &Parse,
        evaluated: Option<ObjRef>,
    ) -> Result<(), Failure> {
        let indent = self.clause_state.current_value_indent;
        let mut strings = self.parse_strings(code, parse, indent, evaluated)?;
        let mut cursor = self.next_template(&mut strings, parse, indent)?;

        for entry in &parse.template {
            let Some(trigger) = entry else {
                // The comma fence is a template boundary and not a trigger:
                // it advances to the next parse string, which for `PARSE ARG`
                // is the next argument and for every other source is the null
                // string (`RexxTarget::next`'s own `next_argument != 1` arm).
                let next = self.next_template(&mut strings, parse, indent)?;
                let spent = std::mem::replace(&mut cursor, next);
                self.give_parse_buffer(spent.into_string());
                continue;
            };
            self.apply_trigger(code, trigger, &mut cursor, indent)?;
            self.assign_targets(code, trigger, &mut cursor, indent)?;
        }
        // Only on the way out through the bottom: a template that leaves
        // through `?` above drops its buffers instead, which is the same
        // trade every lender in this interpreter makes.
        self.give_parse_buffer(cursor.into_string());
        self.give_parse_strings(strings);
        Ok(())
    }

    /// Steps to the next parse string, applying `UPPER`/`LOWER` and tracing
    /// the result.
    ///
    /// `UPPER`/`LOWER` transform the **source** before parsing, where
    /// `CASELESS` leaves it alone and folds only the comparison. Measured
    /// over a byte alphabet, the transform is ASCII-only: `'e0'x` and `'c1'x`
    /// are unchanged by both, while `'61'x` upcases to `'41'x`.
    fn next_template(
        &mut self,
        strings: &mut ParseStrings,
        parse: &Parse,
        indent: usize,
    ) -> Result<Cursor, Failure> {
        let mut string = match strings {
            // A template past the single string parses the null string, and
            // an empty `Vec` is that with nothing taken from the pool -- one
            // of zero capacity is what `give_parse_buffer` declines to park
            // anyway.
            ParseStrings::One(string) => string.take().unwrap_or_default(),
            ParseStrings::Arg(index) => {
                let at = *index;
                *index += 1;
                self.argument_text(at)?
            }
        };
        if parse.upper {
            string.make_ascii_uppercase();
        } else if parse.lower {
            string.make_ascii_lowercase();
        }
        let cursor = Cursor::new(string);
        self.trace_result(indent, cursor.string());
        Ok(cursor)
    }

    /// The strings this `PARSE` will consume, in template order, and the
    /// source's own `>K>` line.
    ///
    /// One entry for every source but `ARG`, which contributes one per
    /// argument of the running activation -- an omitted position becoming the
    /// null string rather than closing up, measured: `call sub 'one two',
    /// 'three four', , 'five'` into `parse arg c1 , c2 , c3 , c4` gives
    /// `[one two][three four][][five]`.
    ///
    /// A template past the last string parses the null string, which
    /// [`Interp::next_template`] gets for free from the single slot emptying
    /// and from [`Interp::argument_text`] answering for a position no argument
    /// occupies.
    fn parse_strings(
        &mut self,
        code: &Code<'_>,
        parse: &Parse,
        indent: usize,
        evaluated: Option<ObjRef>,
    ) -> Result<ParseStrings, Failure> {
        // Which of the two the source produces is what decides whether the
        // required-string protocol runs at all: a source that builds its own
        // bytes never had an object to convert.
        enum Subject {
            Bytes(Vec<u8>),
            Value(ObjRef),
        }
        let (keyword, value) = match &parse.source {
            // `PARSE VALUE WITH template`, with no expression at all, is
            // legal and parses the null string.
            //
            // **And it traces a literal's `>L>` line for a null string that
            // no expression produced**, which is the parser's doing rather
            // than this instruction's: `LanguageParser`'s own `SUBKEY_VALUE`
            // arm substitutes `GlobalNames::NULLSTRING` for a missing
            // expression, so by the time `RexxInstructionParse::execute` runs
            // there is a literal to evaluate and the line is that evaluation's
            // side effect. (`execute`'s own `expression != OREF_NULL` guard is
            // unreachable for the same reason.) Measured: `parse value with a1
            // a2` under `trace i` traces `>L>   ""` and then `>K>   "VALUE" =>
            // ""`, in that order.
            ParseSource::Value(None) => {
                self.trace_literal(indent, b"");
                ("VALUE", Subject::Bytes(Vec::new()))
            }
            // **Rooted by whichever side produced it.** A value handed in is
            // already held by the register the compiled stream evaluated it
            // into, so a second temp here would root it against a frame this
            // clause does not own; a value evaluated here has nothing else
            // holding it across the `&mut self` calls the template walk makes.
            ParseSource::Value(Some(expression)) => {
                let value = match evaluated {
                    Some(value) => value,
                    None => {
                        let value = self.eval(code, expression)?;
                        self.roots.push_temp(value);
                        value
                    }
                };
                ("VALUE", Subject::Value(value))
            }
            // An ordinary variable read, with everything that implies: `>C>`
            // and `>V>` for a compound, and `NOVALUE` for an unset name --
            // measured, `signal on novalue` traps on `parse var zzunset t`.
            ParseSource::Var(id) => {
                let value = self.read_parse_var(code, *id, indent)?;
                self.roots.push_temp(value);
                ("VAR", Subject::Value(value))
            }
            // The second word is the *calling context* rather than the call
            // depth, and it is the running activation's rather than this
            // clause's: `crate::activation::CallType` carries the measured
            // table and is set where each activation is built.
            ParseSource::Source => {
                let mut source = self.take_parse_buffer();
                source.extend_from_slice(PLATFORM);
                source.push(b' ');
                source.extend_from_slice(self.activation().call_type.token());
                source.push(b' ');
                // The third word is the executable's own name, which is the
                // program's path for a program and the method's name for a
                // method compiled from source text -- measured, oracle rc 0,
                // `parse source` inside a `setMethod` body answers `LINUX
                // METHOD MM`.
                let program = self.activation().program_id;
                source.extend_from_slice(self.program_display_name(program));
                ("SOURCE", Subject::Bytes(source))
            }
            ParseSource::Version => {
                let mut version = self.take_parse_buffer();
                version.extend_from_slice(VERSION);
                ("VERSION", Subject::Bytes(version))
            }
            // No `>K>` line of any kind, and the one source with more than
            // one string (`RexxInstructionParse::execute`'s own `SUBKEY_ARG`
            // arm, which is the only one that does not call
            // `traceKeywordResult`).
            ParseSource::Arg => return Ok(ParseStrings::Arg(0)),
            // The two sources that read a line rather than evaluating a value.
            // The split between them is entirely in which reader is called --
            // `PULL` takes the queue's head when there is one, `LINEIN` never
            // consults the queue at all -- and `input.rs` owns that rule and
            // the measurements behind it.
            //
            // The `>K>` line below therefore carries the line **before** any
            // `UPPER` transform, because `next_template` is what applies the
            // transform and it runs after this. Measured, `pull n3` on a line
            // reading `skipped three`: `>K> "PULL" => "skipped three"` and
            // then `>>> "SKIPPED THREE"`, the two lines disagreeing on the
            // same instruction.
            ParseSource::Pull => ("PULL", Subject::Bytes(self.pull_line())),
            ParseSource::LineIn => ("LINEIN", Subject::Bytes(self.linein_line())),
        };
        let value = match value {
            Subject::Bytes(bytes) => {
                self.trace_keyword(indent, keyword, &bytes);
                bytes
            }
            // **`>K>` names the object and the template walks the
            // conversion.** `RexxInstructionParse::execute` traces `value`
            // and hands the same object to the target, whose own `init` calls
            // `string->requestString()` (`instructions/ParseTarget.cpp:113`).
            // Measured, `trace r` over `parse value .K with a b` with a
            // class-side `makeString` returning `'p q'`:
            // `>K>   "VALUE" => "The K class"` and then `>>>   "p q"`.
            Subject::Value(value) => {
                let traced = self.rendered_into_parse_buffer(value);
                self.trace_keyword(indent, keyword, &traced);
                let converted = self.required_string_value(value)?;
                if converted == value {
                    traced
                } else {
                    self.give_parse_buffer(traced);
                    self.rendered_into_parse_buffer(converted)
                }
            }
        };
        Ok(ParseStrings::One(Some(value)))
    }

    /// `PARSE VAR`'s source: the variable read, in all three name shapes.
    ///
    /// Mirrors `eval_node`'s own three arms rather than routing through
    /// `eval`, because the AST carries a bare `SymbolId` here and not an
    /// `Expr` to evaluate. The trace lines are the read's, not the
    /// instruction's -- measured, `parse var aa.ii p` traces `>C> AA.II =>
    /// "AA.3"` then `>V> AA.II => "one two"` before the `>K>` line, exactly
    /// as `say aa.ii` would.
    fn read_parse_var(
        &mut self,
        code: &Code<'_>,
        id: rexx_parse::SymbolId,
        indent: usize,
    ) -> Result<ObjRef, Failure> {
        // Borrowed: `SymbolTable::name` answers with the `Code`'s lifetime,
        // which is the program and not this `Interp`, so it survives the
        // `&mut self` calls below. The renderings beneath are taken only when
        // a line will print them -- `trace_variable` and
        // `trace_compound_name` both return at once unless intermediates are
        // on, and this function is on the `PARSE` path, which
        // `samples/rexxcps.rex` runs on every clause of its inner loop.
        let name = code.symbols.name(id).as_bytes();
        match crate::run::shape_of(name) {
            crate::run::NameShape::Simple => {
                let (value, novalue) = self.read(code, id);
                self.novalue_check(novalue)?;
                if self.tracing_intermediates() {
                    let text = self.to_text(value).to_vec();
                    self.trace_variable(indent, name, &text);
                }
                Ok(value)
            }
            // A bare stem read raises no `NOVALUE` (`eval_node`'s own arm has
            // the measured pair) and announces no resolved name.
            crate::run::NameShape::Stem => {
                let value = self.read_stem(name);
                if self.tracing_intermediates() {
                    let text = self.to_text(value).to_vec();
                    self.trace_variable(indent, name, &text);
                }
                Ok(value)
            }
            crate::run::NameShape::Compound => {
                let (stem_name, stem_at) = code.stem(id);
                let mut key = self.take_key_buffer();
                if let Err(failure) = self.tail_key_into(code, id, &mut key) {
                    self.give_key_buffer(key);
                    return Err(failure);
                }
                if self.tracing_intermediates() {
                    let mut resolved = stem_name.to_vec();
                    resolved.extend_from_slice(&key);
                    self.trace_compound_name(indent, name, &resolved);
                }
                let (value, novalue) = self.stem_get_at(stem_name, stem_at, &key);
                self.give_key_buffer(key);
                self.novalue_check(novalue)?;
                if self.tracing_intermediates() {
                    let text = self.to_text(value).to_vec();
                    self.trace_variable(indent, name, &text);
                }
                Ok(value)
            }
        }
    }

    /// Evaluates one trigger's operand, if it has one, and moves the cursor.
    fn apply_trigger(
        &mut self,
        code: &Code<'_>,
        trigger: &ParseTrigger,
        cursor: &mut Cursor,
        indent: usize,
    ) -> Result<(), Failure> {
        // `End` is the only kind with no operand, so every other arm below
        // can ask for one. A missing operand on a kind that needs one is not
        // representable from a program that parsed -- `parse_template`
        // (`rexx-parse`) fills `value` on every trigger it builds except
        // `End`, and refuses a trigger with nothing after it (38.901) -- and
        // it is reported as the gap it would be rather than defaulted to a
        // position.
        let operand = match trigger.kind {
            TriggerKind::End => {
                cursor.move_to_end();
                return Ok(());
            }
            _ => match &trigger.value {
                Some(operand) => operand,
                None => return Err(Loud::parse_trigger_operand().into()),
            },
        };
        let value = self.eval(code, operand)?;
        self.roots.push_temp(value);
        // `>>>` for the operand, after `eval`'s own `>L>`/`>V>` and before
        // the conversion that can fail: `integerTrigger`
        // (`ParseTrigger.cpp:143`-`153`) traces and only then converts, so
        // the operand's own value line is emitted even on the 26.4 path.
        // **Copied only where a copy is needed**, which is neither of the two
        // arms below. `trace_result` returns at once unless `results` is on,
        // and a search reads the bytes where they already are: `Cursor` is a
        // local of `exec_parse` rather than a field, so a shared borrow of
        // the value can be live while it is written to. The unconditional
        // `to_vec` here was 5.2% of every allocation `samples/rexxcps.rex`
        // made -- one per trigger, on a program whose templates are mostly
        // numeric triggers that never look at the bytes at all.
        if self.trace_mode().results {
            let rendered = self.to_text(value).to_vec();
            self.trace_result(indent, &rendered);
        }
        match trigger.kind {
            TriggerKind::String => {
                cursor.search(&self.to_text(value));
                Ok(())
            }
            TriggerKind::Mixed => {
                cursor.caseless_search(&self.to_text(value));
                Ok(())
            }
            _ => {
                // 26.4, and the conversion is bounded by the **active**
                // `NUMERIC DIGITS` rather than by a fixed width
                // (`integerTrigger`'s own `requestUnsignedNumber(result,
                // number_digits())`). Measured: `numeric digits 2; parse
                // value d with p +(100) q` is 26.4 on the oracle, as are a
                // fractional operand, a non-numeric one and a negative one.
                // The substitution is the operand's own rendering -- measured
                // `found "1E2"` for `+(1e2)`, which is D15's rule showing
                // through rather than a re-rendering here.
                let Some(offset) = self.whole_nonneg(value) else {
                    // The one place the operand's own rendering is still
                    // owned, and it is the raise's substitution.
                    return Err(Raised::syntax(26, 4, vec![self.to_text(value).to_vec()]).into());
                };
                // Clamped rather than checked: an offset past `usize` is
                // indistinguishable from one past the string's own end, and
                // every operation below treats those the same way.
                let offset = usize::try_from(offset).unwrap_or(usize::MAX);
                match trigger.kind {
                    TriggerKind::Plus => cursor.forward(offset),
                    TriggerKind::Minus => cursor.backward(offset),
                    TriggerKind::Absolute => cursor.absolute(offset),
                    TriggerKind::PlusLength => cursor.forward_length(offset),
                    TriggerKind::MinusLength => cursor.backward_length(offset),
                    // Handled above; the operand kinds and the movement
                    // kinds partition `TriggerKind`, and this arm is what
                    // keeps that a compile-time exhaustive match rather than
                    // an assumption.
                    TriggerKind::End | TriggerKind::String | TriggerKind::Mixed => {}
                }
                Ok(())
            }
        }
    }

    /// Carves the current section into this trigger's targets: a blank
    /// delimited word each, and the whole remainder for the last one.
    ///
    /// Extra targets get the null string rather than being skipped --
    /// measured, `parse value 'a b' with p q r s` gives `[a][b][][]` -- which
    /// falls out of `next_word`/`remainder` answering empty once the section
    /// is used up, with no separate case here.
    fn assign_targets(
        &mut self,
        code: &Code<'_>,
        trigger: &ParseTrigger,
        cursor: &mut Cursor,
        indent: usize,
    ) -> Result<(), Failure> {
        let last = trigger.targets.len().saturating_sub(1);
        for (index, target) in trigger.targets.iter().enumerate() {
            let piece = if index == last {
                cursor.remainder()
            } else {
                cursor.next_word()
            };
            match target {
                Some(target) => {
                    // **The value is built inside this arm because the other
                    // arm has nothing to assign it to.** A `.` consumes its
                    // field and stores nowhere, so a value built ahead of the
                    // match is created, rooted and dropped unread.
                    let value = self.text(&cursor.string()[piece.clone()]);
                    self.roots.push_temp(value);
                    // **The slot the upfront pass already bound this target
                    // to.** No compiler resolves a `PARSE` target -- nothing
                    // promotes the instruction -- but `Plan::build` walks
                    // every instruction of the body and binds the names each
                    // one writes, `note_parse` included, so the answer exists
                    // here and used to be recomputed from the name on every
                    // firing. Measured on `samples/rexxcps.rex` before this
                    // changed: of the `slot_of` calls that program makes,
                    // 2,800,003 were `PARSE` targets and every one of them
                    // resolved to the slot `by_symbol` already held.
                    //
                    // **A `Variable` target only.** A stem-shaped or
                    // compound-shaped target takes its slot from the plan's
                    // own `CompoundName` entry inside `assign_expr_target`,
                    // and those two arms assert that this argument is `None`
                    // -- the guard exists because this is the call site most
                    // likely to break it.
                    let at = match &target.kind {
                        ExprKind::Variable(id) => code.slot_for(*id),
                        _ => None,
                    };
                    // Always `Some`: this is a borrow of the parse source
                    // rather than a fresh copy of the assigned value, so
                    // there is nothing here for the `Option` to guard
                    // against -- see `assign_expr_target`'s own doc for what
                    // the other caller pays.
                    self.assign_expr_target(
                        code,
                        target,
                        value,
                        Some(&cursor.string()[piece.clone()]),
                        indent,
                        at,
                    )?;
                    // The `TRACE R` half of the pair -- see this module's own
                    // `exec_parse` doc for why it is a choice of prefix and
                    // not a second, independent line.
                    if !self.tracing_intermediates() {
                        self.trace_result(indent, &cursor.string()[piece]);
                    }
                }
                // The `.` placeholder consumes a field and assigns nothing,
                // and its own line is emitted **even when it consumed
                // nothing** -- measured, `parse value 'one two' with p . q`
                // traces `>.>   ""` between `>=> P` and `>=> Q`.
                None => self.trace_dummy(indent, &cursor.string()[piece]),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The measured oracle answers for one template, as `(template, pieces)`.
    /// Every row was taken from the oracle on 2026-08-05 (this task's report
    /// carries the transcripts); the driver below replays only the *movement*,
    /// so a row here is a claim about [`Cursor`] alone.
    fn pieces(source: &str, template: &[Trigger]) -> Vec<String> {
        let mut cursor = Cursor::new(source.as_bytes().to_vec());
        let mut out = Vec::new();
        for step in template {
            match step {
                Trigger::End => cursor.move_to_end(),
                Trigger::Plus(n) => cursor.forward(*n),
                Trigger::Minus(n) => cursor.backward(*n),
                Trigger::Absolute(n) => cursor.absolute(*n),
                Trigger::PlusLength(n) => cursor.forward_length(*n),
                Trigger::MinusLength(n) => cursor.backward_length(*n),
                Trigger::Search(needle) => cursor.search(needle.as_bytes()),
                Trigger::Caseless(needle) => cursor.caseless_search(needle.as_bytes()),
                Trigger::Targets(count) => {
                    for index in 0..*count {
                        let piece = if index + 1 == *count {
                            cursor.remainder()
                        } else {
                            cursor.next_word()
                        };
                        out.push(String::from_utf8_lossy(&cursor.string()[piece]).into_owned());
                    }
                }
            }
        }
        out
    }

    /// A template step, for [`pieces`]. `Targets(n)` is the assignment loop
    /// belonging to the trigger before it, which is where a template's
    /// targets attach (this module's own doc).
    enum Trigger {
        End,
        Plus(usize),
        Minus(usize),
        Absolute(usize),
        PlusLength(usize),
        MinusLength(usize),
        Search(&'static str),
        Caseless(&'static str),
        Targets(usize),
    }
    use Trigger::*;

    const D: &str = "abcdefghij";

    /// `-n` and `<n` are unrelated operations. The pair this crate was warned
    /// would be conflated, and the reason the two match positions exist.
    #[test]
    fn minus_and_minus_length_differ_on_the_same_movement() {
        assert_eq!(
            pieces(
                D,
                &[
                    Absolute(5),
                    Targets(1),
                    Minus(2),
                    Targets(1),
                    End,
                    Targets(1)
                ]
            ),
            ["abcd", "efghij", "cdefghij"]
        );
        assert_eq!(
            pieces(
                D,
                &[
                    Absolute(5),
                    Targets(1),
                    MinusLength(2),
                    Targets(1),
                    End,
                    Targets(1)
                ]
            ),
            ["abcd", "cd", "cdefghij"]
        );
    }

    /// `=n` and `+n`/`-n` share one rule: forward of the current position
    /// gives `[current, new)`, and anything else -- including *equal* -- gives
    /// `[current, END]`.
    #[test]
    fn absolute_and_relative_share_the_backward_rule() {
        assert_eq!(
            pieces(D, &[Absolute(5), Targets(1), End, Targets(1)]),
            ["abcd", "efghij"]
        );
        // 1 is not greater than 1, so the second target is the remainder and
        // not the null string.
        assert_eq!(
            pieces(D, &[Absolute(1), Targets(1), End, Targets(1)]),
            ["abcdefghij", "abcdefghij"]
        );
        assert_eq!(
            pieces(D, &[Absolute(11), Targets(1), End, Targets(1)]),
            ["abcdefghij", ""]
        );
        assert_eq!(
            pieces(
                D,
                &[
                    Absolute(5),
                    Targets(1),
                    Absolute(5),
                    Targets(1),
                    End,
                    Targets(1)
                ]
            ),
            ["abcd", "efghij", "efghij"]
        );
        assert_eq!(
            pieces(
                D,
                &[
                    Absolute(5),
                    Targets(1),
                    Minus(99),
                    Targets(1),
                    End,
                    Targets(1)
                ]
            ),
            ["abcd", "efghij", "abcdefghij"]
        );
        assert_eq!(
            pieces(D, &[Plus(0), Targets(1), End, Targets(1)]),
            ["abcdefghij", "abcdefghij"]
        );
    }

    /// `>n`/`<n` do **not** share it: they are exact slices, clamped at the
    /// ends, and an offset of zero gives the null string where `+0` gives the
    /// whole remainder.
    #[test]
    fn length_triggers_are_exact_slices() {
        assert_eq!(
            pieces(D, &[PlusLength(0), Targets(1), End, Targets(1)]),
            ["", "abcdefghij"]
        );
        assert_eq!(
            pieces(D, &[MinusLength(0), Targets(1), End, Targets(1)]),
            ["", "abcdefghij"]
        );
        assert_eq!(
            pieces(
                D,
                &[
                    PlusLength(3),
                    Targets(1),
                    MinusLength(2),
                    Targets(1),
                    End,
                    Targets(1)
                ]
            ),
            ["abc", "bc", "bcdefghij"]
        );
        assert_eq!(
            pieces(D, &[PlusLength(20), Targets(1), End, Targets(1)]),
            ["abcdefghij", ""]
        );
        assert_eq!(
            pieces(D, &[MinusLength(20), Targets(1), End, Targets(1)]),
            ["", "abcdefghij"]
        );
    }

    /// An absent pattern matches at END, the empty pattern behaves as absent,
    /// and a pattern at position 1 gives the null string. Searches are
    /// non-overlapping and the next one starts after the previous match.
    #[test]
    fn string_patterns_match_at_end_when_absent() {
        assert_eq!(
            pieces(D, &[Search("z"), Targets(1), End, Targets(1)]),
            ["abcdefghij", ""]
        );
        assert_eq!(
            pieces(D, &[Search(""), Targets(1), End, Targets(1)]),
            ["abcdefghij", ""]
        );
        assert_eq!(
            pieces(D, &[Search("a"), Targets(1), End, Targets(1)]),
            ["", "bcdefghij"]
        );
        assert_eq!(
            pieces(
                "aXbXc",
                &[
                    Search("X"),
                    Targets(1),
                    Search("X"),
                    Targets(1),
                    End,
                    Targets(1)
                ]
            ),
            ["a", "b", "c"]
        );
        assert_eq!(
            pieces(
                "aXXb",
                &[
                    Search("XX"),
                    Targets(1),
                    Search("X"),
                    Targets(1),
                    End,
                    Targets(1)
                ]
            ),
            ["a", "b", ""]
        );
    }

    /// After a string pattern the next *target* starts past the match, but a
    /// following *relative* trigger measures from the match's START -- the
    /// subtle one, and the reason `pattern_start` and `pattern_end` are two
    /// fields.
    #[test]
    fn a_relative_trigger_after_a_pattern_measures_from_the_match_start() {
        let plain = pieces(D, &[Search("c"), Targets(1), End, Targets(1)]);
        assert_eq!(plain, ["ab", "defghij"]);
        // `+1` moves to exactly where the match ended, so the End trigger's
        // own section is the same one it would have been with no trigger at
        // all.
        assert_eq!(
            pieces(D, &[Search("c"), Targets(1), Plus(1), End, Targets(1)]),
            plain
        );
        assert_eq!(
            pieces(D, &[Search("c"), Targets(1), Minus(1), End, Targets(1)]),
            ["ab", "bcdefghij"]
        );
        assert_eq!(
            pieces(
                D,
                &[Search("c"), Targets(1), MinusLength(1), End, Targets(1)]
            ),
            ["ab", "bcdefghij"]
        );
    }

    /// `CASELESS` folds ASCII letters and nothing else. The three
    /// non-matching rows are byte pairs exactly `0x20` apart that are not
    /// letters, plus a high byte in each direction.
    #[test]
    fn caseless_folds_ascii_letters_only() {
        assert_eq!(
            pieces("aZb", &[Caseless("z"), Targets(1), End, Targets(1)]),
            ["a", "b"]
        );
        assert_eq!(
            pieces(
                "a\u{7b}b",
                &[Caseless("\u{5b}"), Targets(1), End, Targets(1)]
            ),
            ["a{b", ""]
        );
        assert_eq!(
            pieces(
                "a\u{5f}b",
                &[Caseless("\u{3f}"), Targets(1), End, Targets(1)]
            ),
            ["a_b", ""]
        );
        // A byte alphabet, so the two high bytes are compared as bytes and
        // not as text: 0xc9 and 0xe9 are 0x20 apart and match only
        // themselves.
        let mut high = Cursor::new(vec![b'a', 0xc9, b'b']);
        high.caseless_search(&[0xe9]);
        let piece = high.remainder();
        assert_eq!(high.string()[piece], [b'a', 0xc9, b'b']);
        let mut same = Cursor::new(vec![b'a', 0xc9, b'b']);
        same.caseless_search(&[0xc9]);
        let piece = same.remainder();
        assert_eq!(same.string()[piece], [b'a']);
    }

    /// Word carving: only the final target keeps its leading blanks, extra
    /// targets get the null string, and a tab is whitespace.
    #[test]
    fn word_carving_keeps_leading_blanks_only_on_the_last_target() {
        assert_eq!(pieces("a  b  c", &[End, Targets(3)]), ["a", "b", " c"]);
        assert_eq!(pieces("a  b  c", &[End, Targets(2)]), ["a", " b  c"]);
        assert_eq!(pieces("  a b  ", &[End, Targets(2)]), ["a", "b  "]);
        assert_eq!(pieces("a b", &[End, Targets(4)]), ["a", "b", "", ""]);
        assert_eq!(pieces("a\tb", &[End, Targets(2)]), ["a", "b"]);
    }

    /// Every `TriggerKind` reaches the [`Cursor`] operation that belongs to
    /// it, and every source reaches its own string -- run through
    /// `run_program`, so the `step` arm and [`Interp::apply_trigger`]'s own
    /// dispatch are inside what is being tested.
    ///
    /// **The tests above cannot catch a dispatch swap and this was measured
    /// rather than assumed.** They call `Cursor`'s methods by name, so
    /// sending `MinusLength` to `Cursor::backward` in `apply_trigger` leaves
    /// all eight of them green. Every row below is oracle output, and the rows
    /// are chosen so that no two kinds agree on their own row: `+n` and `>n`
    /// coincide for a non-zero offset, which is what the `+0`/`>0` pair is
    /// for, and `-n` and `=n` coincide on the *second* field, which is what
    /// the third field separates.
    #[test]
    fn every_trigger_kind_and_source_reaches_its_own_operation() {
        let d = "d = 'abcdefghij'\n";
        let show3 = "say '['||p||']['||q||']['||r||']'\n";
        let show2 = "say '['||p||']['||q||']'\n";
        let rows: &[(&str, &str)] = &[
            // `+n` and `>n` agree here; `-n` and `<n` do not agree with
            // either or with each other.
            ("parse value d with p 5 q +2 r\n", "[abcd][ef][ghij]\n"),
            (
                "parse value d with p 5 q -2 r\n",
                "[abcd][efghij][cdefghij]\n",
            ),
            ("parse value d with p 5 q <2 r\n", "[abcd][cd][cdefghij]\n"),
            ("parse value d with p 5 q >2 r\n", "[abcd][ef][ghij]\n"),
        ];
        for (template, expected) in rows {
            let source = format!("{d}{template}{show3}");
            let outcome = crate::run_program(
                "/tmp/parse-dispatch.rex",
                source.into_bytes(),
                crate::Invocation::none(),
            );
            assert_eq!(
                String::from_utf8_lossy(&outcome.stdout),
                *expected,
                "{template}"
            );
        }

        let pairs: &[(&str, &str)] = &[
            // The offset that separates `+n` from `>n`: the no-movement rule
            // applies to one and not the other.
            ("parse value d with p +0 q\n", "[abcdefghij][abcdefghij]\n"),
            ("parse value d with p >0 q\n", "[][abcdefghij]\n"),
            // A bare numeric symbol is the same absolute column `=n` is.
            ("parse value d with p 2 q\n", "[a][bcdefghij]\n"),
            ("parse value d with p =2 q\n", "[a][bcdefghij]\n"),
            // `String` against `Mixed`: the same needle, and only the
            // caseless one matches.
            ("parse value 'aXb' with p 'x' q\n", "[aXb][]\n"),
            ("parse caseless value 'aXb' with p 'x' q\n", "[a][b]\n"),
            // `End`, which has no operand at all.
            ("parse value 'w1 w2 w3' with p q\n", "[w1][w2 w3]\n"),
            // The sources: each has to reach its own string rather than
            // another source's.
            ("vv = 'v1 v2'; parse var vv p q\n", "[v1][v2]\n"),
            ("parse value 'e1 e2' with p q\n", "[e1][e2]\n"),
        ];
        for (template, expected) in pairs {
            let source = format!("{d}{template}{show2}");
            let outcome = crate::run_program(
                "/tmp/parse-dispatch.rex",
                source.into_bytes(),
                crate::Invocation::none(),
            );
            assert_eq!(
                String::from_utf8_lossy(&outcome.stdout),
                *expected,
                "{template}"
            );
        }
    }

    /// A `PARSE ARG` template past the first reads an argument that every
    /// earlier template's allocations have had a chance to collect.
    ///
    /// **Run under collect-on-every-allocation**, because an ordinary run
    /// reaches the second template long before the heap is due a collection
    /// and so cannot ask the question at all. Measured to have teeth:
    /// removing `eval_traced_argument`'s `push_temp` panics this at `to_text`'s
    /// "a live value".
    ///
    /// The adjacent ordinary run is the other half: without it a mode that
    /// silently declined to collect would satisfy this on its own, and the
    /// `collections > 0` assertion is the same guard from the other side.
    #[test]
    fn a_late_parse_arg_template_survives_collection() {
        // **The arguments are built by concatenation rather than written as
        // literals**, and that is what gives this test its teeth: a literal is
        // interned where the collector cannot take it, so a template reading
        // one would answer correctly however badly it was rooted.
        let source = concat!(
            "call sub 'first' 'argument here', 'second' 'argument here'\n",
            "exit\n",
            "sub:\n",
            "parse arg a1 a2 a3, b1 b2 b3\n",
            "say '['a1']['a2']['a3']['b1']['b2']['b3']'\n",
            "return\n",
        );
        let expected = "[first][argument][here][second][argument][here]\n";
        let stressed = crate::run_program_collect_every_alloc(
            "/tmp/parse-arg-collect.rex",
            source.as_bytes().to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(String::from_utf8_lossy(&stressed.stdout), expected);
        assert_eq!(stressed.exit_code, 0);
        assert!(
            stressed.collections > 0,
            "the stress mode collected nothing, so this proves nothing"
        );
        let ordinary = crate::run_program(
            "/tmp/parse-arg-collect.rex",
            source.as_bytes().to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(String::from_utf8_lossy(&ordinary.stdout), expected);
    }

    /// `PARSE SOURCE` and `PARSE VERSION` reach their own strings, which no
    /// other source's answer can be mistaken for.
    ///
    /// The path is the one `run_program` was handed, so this pins the
    /// plumbing as well as the string: an engine that left it empty prints
    /// `LINUX COMMAND ` with nothing after it.
    ///
    /// The expected version line is built from [`VERSION`] rather than written
    /// out again. This asks whether the `Version` source reaches that constant
    /// -- which is all a test inside this crate can ask -- and leaves *whether
    /// the constant is still the oracle's answer* to the one harness that can
    /// tell, `tests/parse_version_oracle.rs`. Two copies of the string would
    /// only mean two places to edit in step, which is what a self-comparison
    /// is.
    #[test]
    fn source_and_version_carry_their_own_strings() {
        let outcome = crate::run_program(
            "/tmp/parse-source.rex",
            b"parse source s\nsay s\nparse version v\nsay v\n".to_vec(),
            crate::Invocation::none(),
        );
        let mut expected = b"LINUX COMMAND /tmp/parse-source.rex\n".to_vec();
        expected.extend_from_slice(VERSION);
        expected.push(b'\n');
        assert_eq!(outcome.stdout, expected);
    }

    /// A fractional, non-numeric or negative positional operand is 26.4, and
    /// the conversion is bounded by the **active** `NUMERIC DIGITS` rather
    /// than by a fixed width -- measured, `numeric digits 2` makes `+(100)`
    /// fail where the default 9 digits accept it.
    ///
    /// Both halves matter: the adjacent passing case is what stops the rule
    /// being read as "wide operands are refused".
    #[test]
    fn a_positional_operand_must_be_a_whole_number_within_the_active_digits() {
        let refused: &[&str] = &[
            "parse value 'abc' with p 2.5 q\n",
            "parse value 'abc' with p +('x') q\n",
            "parse value 'abc' with p +(-1) q\n",
            "numeric digits 2\nparse value 'abc' with p +(100) q\n",
        ];
        for source in refused {
            let outcome = crate::run_program(
                "/tmp/parse-26-4.rex",
                source.as_bytes().to_vec(),
                crate::Invocation::none(),
            );
            assert_eq!(outcome.exit_code, 230, "{source}");
            assert!(
                String::from_utf8_lossy(&outcome.stderr).contains(
                    "Error 26.4:  Positional pattern of PARSE template must be a whole number"
                ),
                "{source}: {}",
                String::from_utf8_lossy(&outcome.stderr)
            );
        }
        let accepted = crate::run_program(
            "/tmp/parse-26-4.rex",
            b"parse value 'abcdefghij' with p +(100) q\nsay '['||p||']'\n".to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(accepted.exit_code, 0);
        assert_eq!(
            String::from_utf8_lossy(&accepted.stdout),
            "[abcdefghij]\n",
            "an operand past the string's own end is a clamp, not an error"
        );
    }
}
