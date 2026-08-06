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

//! `TRACE` (D17): the mode, the byte-level formatting of each prefix
//! reachable from the code this crate runs -- 4a's own ten (`*-*`, `>>>`,
//! `>=>`, `>L>`, `>V>`, `>O>`, `>K>`, `>C>`, `>P>`, `>E>`) plus the three
//! 4b's calls add (`>A>`, `>F>`, `>R>`, Task 9) -- and the classification a
//! `TRACE`/`TRACE VALUE` setting goes through to become one.
//!
//! **What lives here is formatting and classification, never *when* to
//! call it.** `run.rs`'s `step_in_temps_frame` (the clause echo, and the
//! loop drivers' own re-echo) and `eval.rs`'s `eval` (the post-order
//! intermediate-value hook) own the call sites, because they are the two
//! places D17 names as already having the one-insertion-point shape this
//! task needs to reuse rather than re-derive. This module owns turning
//! "prefix, tag, value, indent" into the oracle's exact bytes, and nothing
//! about when that tuple becomes available.
//!
//! **All format constants below are read from `RexxActivation.cpp:3565`-
//! `3611`, not inferred from output**: `trace_prefix_table` (the 19
//! three-byte prefixes), `LINENUMBER = 6`, `PREFIX_OFFSET = 7`,
//! `PREFIX_LENGTH = 3`, `INDENT_SPACING = 2`, `QUOTES_OVERHEAD = 2`,
//! `TRACE_OVERHEAD = 15`, `VALUE_MARKER = " => "`,
//! `ASSIGNMENT_MARKER = " <= "`. Every byte offset below is that source's
//! arithmetic, re-derived rather than copied as a magic number, and cross-
//! checked against `cat -A` transcripts in this task's own report.

use crate::Interp;
use crate::error::Raised;
use rexx_num::Number;

/// The visible-output shape of the current `TRACE` setting, restricted to
/// what pure-4a code can ever produce (D18 excludes commands, so
/// `traceCommands`/`traceErrors`/`traceFailures` have nothing to show;
/// interactive debug pausing does not exist on this non-interactive runtime
/// at all).
///
/// **A four-field struct, not the oracle's `FlagSet<TraceFlag, 32>`.**
/// `TraceSetting.cpp:49`-`54`'s own flag combinations reduce to four
/// observable questions for a program this crate can run: is every clause
/// echoed (`all`, `TRACE_PREFIX_CLAUSE`), is a traced instruction's own
/// computed value shown (`results`, `TRACE_PREFIX_RESULT`/`_KEYWORD`), is
/// *every* intermediate step of evaluating it shown too (`intermediates`,
/// every other value prefix plus `TRACE_PREFIX_ASSIGNMENT`), and is a
/// `LABEL` clause echoed (`labels`)? `results` is true whenever
/// `intermediates` is (measured: `TRACE I`'s own flag set is `TRACE R`'s
/// plus one more bit, `traceIntermediatesFlags` a strict superset of
/// `traceResultsFlags`) and `labels` is true whenever `all` is, so these are
/// not four independent booleans in practice, but naming an invariant type
/// over fields that are always checked together would be D16's own
/// `Novalue` shape solving a problem this struct does not have.
///
/// **It said "a three-field struct... exactly three observable questions
/// for a program 4a can run" and that was right for 4a and wrong here**:
/// 4b's Task 9 review round 1 measured the fourth, `TRACE L`, which this
/// crate answered with silence. The count in a sentence like that is a
/// claim about the language, and it goes stale the way a table does.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub(crate) struct TraceMode {
    /// `TRACE_PREFIX_CLAUSE` (`*-*`): every stepped instruction's own clause
    /// is echoed. `TraceSetting::tracingAll`/`tracingInstructions`.
    pub(crate) all: bool,
    /// `TRACE_PREFIX_RESULT`/`_KEYWORD` (`>>>`/`>K>`): a traced instruction's
    /// own top-level computed value is shown once. `tracingResults`.
    pub(crate) results: bool,
    /// Every other value prefix (`>L>`/`>V>`/`>O>`/`>P>`/`>C>`) plus
    /// `>=>` (`TRACE_PREFIX_ASSIGNMENT`): every step of *evaluating* a
    /// traced instruction's expression is shown, not only its final value.
    /// `tracingIntermediates`.
    pub(crate) intermediates: bool,
    /// `TraceSetting::tracingLabels`: a `LABEL` clause is echoed, with the
    /// ordinary `*-*` prefix. **A fourth field, added at 4b Task 9's review
    /// round 1 (F8) because 4b measured a fourth observable question this
    /// struct's own "three for a program 4a can run" argument did not
    /// cover.** `RexxInstructionLabel::execute` (`LabelInstruction.cpp`,
    /// read directly) traces through `traceLabel` and nothing else, and
    /// `traceLabel`'s gate is `tracingLabels()` -- which
    /// `TraceSetting.cpp:52`-`54` sets in `traceAllFlags`,
    /// `traceResultsFlags` and `traceIntermediatesFlags` as well as in
    /// `setTraceLabels`. So this is `true` under `A`/`R`/`I` too, where
    /// `all` already covers it, and the only mode it decides anything in is
    /// `L`.
    ///
    /// **The condition is "the `LABEL` instruction executed", not any list
    /// of ways control can arrive**, and that distinction cost a wrong
    /// sentence here: this said `trace l` echoes "a fallen-through label, a
    /// `CALL` target and a `SIGNAL` target", which reads as a closed set and
    /// is not one -- an internal *function* call reaches a label too. The
    /// C++ site above enumerates the whole condition in one line; nothing
    /// enumerates the routes, so nothing here should count them.
    pub(crate) labels: bool,
}

impl TraceMode {
    /// `TraceSetting::setTraceOff`/`setTraceNormal`/`setTraceCommands`/
    /// `setTraceErrors`/`setTraceFailures`, and the initial state before any
    /// `TRACE` instruction runs at all -- every one of these sets a flag
    /// this crate's own scope has nothing to show for (D18 excludes
    /// commands; errors/failures are command-condition machinery, not built
    /// here), so all five collapse to this crate's one silent answer.
    /// `#[derive(Default)]` picks this automatically (all four `false`),
    /// which is also `Interp::new`'s own starting state.
    ///
    /// **`setTraceLabels` used to be the sixth name in that list and is
    /// not any more**: `TRACE L` has something to show -- the label clauses
    /// it echoes -- and [`TraceMode::LABELS`] is where it goes now. Round 1
    /// changed the count in the sentence above and left the name in it, so
    /// the two contradicted each other; the re-review (NEW-6) caught that
    /// and this is the corrected pair.
    pub(crate) const OFF: TraceMode = TraceMode {
        all: false,
        results: false,
        intermediates: false,
        labels: false,
    };
    /// `TRACE L` (`setTraceLabels`, which resets every other flag and sets
    /// `traceLabels` alone). The one mode where `labels` decides anything:
    /// every executed `LABEL` clause echoes and nothing else does.
    /// `tests/trace_oracle/trace_labels.rex` carries the probed routes and
    /// says why they are examples rather than an enumeration.
    const LABELS: TraceMode = TraceMode {
        all: false,
        results: false,
        intermediates: false,
        labels: true,
    };
    /// `TRACE A` (`setTraceAll`, `traceAllFlags`): every clause echoes, but
    /// `traceAllFlags` deliberately omits `traceResults` -- measured
    /// nowhere in this task's own corpus, but stated here because
    /// `mode_from_setting` needs a real answer for the letter, not a
    /// silent fallback that would misreport `TRACE A x = 1` as producing no
    /// `*-*` line at all.
    const ALL: TraceMode = TraceMode {
        all: true,
        results: false,
        intermediates: false,
        labels: true,
    };
    /// `TRACE R` (`setTraceResults`, `traceResultsFlags`).
    const RESULTS: TraceMode = TraceMode {
        all: true,
        results: true,
        intermediates: false,
        labels: true,
    };
    /// `TRACE I` (`setTraceIntermediates`, `traceIntermediatesFlags`).
    const INTERMEDIATES: TraceMode = TraceMode {
        all: true,
        results: true,
        intermediates: true,
        labels: true,
    };
}

/// Classifies a `TRACE` option string exactly like
/// `TraceSetting::parseTraceSetting` (`TraceSetting.cpp:135`-`210`): skip any
/// number of leading `?`s (a debug-pause toggle this non-interactive runtime
/// has nothing to toggle, so simply skipped rather than tracked), and the
/// first *other* byte decides, case-insensitively; everything after that one
/// byte is ignored. An empty string, or one made only of `?`s, is
/// `setTraceNormal`'s silent answer.
///
/// Returns the offending byte, verbatim (not uppercased), on anything not in
/// `"ACEFILNOR"` -- the nine letters `rexx-inventory`'s own 24.1 message
/// names. Used for **both** `Trace::Setting` (already validated at parse
/// time by `rexx-parse`'s own `check_trace_setting`, so its call site can
/// `.expect()` this always returning `Ok`) and `Trace::Value` (computed at
/// run time from an arbitrary Rexx expression, never validated by anything
/// before this call), which is why this returns a `Result` at all rather
/// than assuming a valid letter the way a `Trace::Setting`-only version
/// could.
pub(crate) fn mode_from_setting(bytes: &[u8]) -> Result<TraceMode, u8> {
    for &byte in bytes {
        if byte == b'?' {
            continue;
        }
        return match byte.to_ascii_uppercase() {
            b'A' => Ok(TraceMode::ALL),
            b'R' => Ok(TraceMode::RESULTS),
            b'I' => Ok(TraceMode::INTERMEDIATES),
            // `L` was in the silent group below until 4b Task 9's review
            // round 1 measured what it actually does. It echoes every
            // executed `LABEL` clause -- see `TraceMode::labels`.
            b'L' => Ok(TraceMode::LABELS),
            // `C`/`E`/`F`/`N`/`O`: all nine of `check_trace_setting`'s
            // accepted letters are recognised here, not only the ones with
            // a visible effect in this crate's scope -- `TRACE C x = 1` must
            // not be treated as an unrecognised setting, it must be treated
            // as "recognised, and this crate has nothing to show for it".
            b'C' | b'E' | b'F' | b'N' | b'O' => Ok(TraceMode::OFF),
            _ => Err(byte),
        };
    }
    Ok(TraceMode::OFF)
}

/// The same precision `rexx-parse`'s own `TRACE_DIGITS` uses for the
/// skip-count forms (`trace 5`, `trace -3`) -- duplicated as a constant
/// rather than imported, because `rexx-parse::instruction::TRACE_DIGITS` is
/// private to that module and this crate cannot reach it. Not a second
/// number-parsing *algorithm*: `whole_number` (`rexx-parse::convert`) is
/// `Number::parse(text)?.whole_value(digits)`, the same two `rexx-num`
/// calls below, so only the digits bound is restated, not the arithmetic.
const TRACE_DIGITS: usize = 9;

/// Whether `text` (a `TRACE VALUE` expression's rendered result) is a whole
/// number under `TRACE_DIGITS` -- the same question `rexx-parse`'s parser
/// asks of `TRACE`'s own literal-number form, asked again here because
/// `TRACE VALUE`'s text is not known until run time and never passes
/// through the parser's own check at all. Measured: `trace value 5` raises
/// 24.901 exactly like `trace 5` (this task's own report has the
/// transcript), so a numeric-looking `VALUE` result is a skip count, not a
/// setting string, even though nothing parsed it as one.
pub(crate) fn is_whole_number(text: &[u8]) -> bool {
    std::str::from_utf8(text)
        .ok()
        .and_then(Number::parse)
        .and_then(|n| n.whole_value(TRACE_DIGITS))
        .is_some()
}

/// 24.901, "Numeric TRACE requests are valid only from interactive
/// debugging." -- unconditional, regardless of the number (measured: `trace
/// 0` raises it exactly like `trace 5`), because this runtime has no
/// interactive debugging at all for a nonzero skip count to be valid *from*.
/// No substitution value, matching the catalogue's `(24, 901)` entry.
///
/// Through `Raised::syntax` rather than a bare struct literal, which is what
/// this and its neighbour were until 4b's Task 7 gave `Raised` a field no
/// raiser cares about (`Delivery`). That constructor's own doc comment has
/// the argument; the short version is that twenty-one copies of `condition:
/// "SYNTAX"` each had to name the new field, and one call does not.
pub(crate) fn raised_numeric_trace_interactive_only() -> Raised {
    Raised::syntax(24, 901, Vec::new())
}

/// 24.1, "TRACE request letter must be one of \"ACEFILNOR\"; found \"&1\"."
/// -- `mode_from_setting`'s `Err` case, reachable only through `TRACE
/// VALUE` (a `Trace::Setting`'s own text is pre-validated, so this arm is
/// unreachable from it; `run.rs`'s own `Trace::Setting` call site
/// `.expect()`s that rather than routing through this at all). `found` is
/// the offending byte exactly as `mode_from_setting` returned it -- not
/// uppercased, matching `TraceSetting.cpp`'s own `badOption = value->
/// getChar(pos)`, which records the character as typed.
pub(crate) fn raised_invalid_trace_letter(found: u8) -> Raised {
    Raised::syntax(24, 1, vec![vec![found]])
}

// ---- byte-level formatting ----

/// Appends `indent` spaces to `out` -- the one place every formatter below
/// applies Task 11's own quantity, so a caller never repeats
/// `std::iter::repeat_n(b' ', indent)` at five call sites.
fn push_indent(out: &mut Vec<u8>, indent: usize) {
    out.extend(std::iter::repeat_n(b' ', indent));
}

/// Runs the oracle's display rule over the trace line that starts at
/// `line_start`, which is the last thing each formatter below does.
///
/// **This is `RexxActivation::processTraceInfo`'s own
/// `traceLine->stringTrace()`** (`execution/RexxActivation.cpp:5249`), which
/// every live trace line passes through on its way to `traceOutput`. The
/// rule itself and its 256-value measurement live in `error.rs`'s
/// `displayable`, beside the other route to the same C++ function.
///
/// Applied per completed line rather than once over `self.trace`, because
/// the trace buffer is appended to across a whole run and a single pass at
/// the end would have no moment to run at. The three formatters that finish
/// a line each call this; `push_operator` does not, since it delegates to
/// `push_tagged`, which does.
///
/// Measured, and the reason this exists separately from the report's own
/// application: `trace r` over `say 'p'||'02'x||'q'` prints `"p?q"` on the
/// oracle's `>>>` line and agrees with us on stdout, where the raw byte
/// belongs.
fn make_displayable(out: &mut [u8], line_start: usize) {
    crate::error::displayable(&mut out[line_start..]);
}

/// The widest indent a `*-*` clause echo ever prints, in spaces.
///
/// **The cap is on the `*-*` echo alone, and it is on the total printed
/// indent rather than on any of the quantities that add up to it.** Measured
/// against the oracle with plain nested `DO`s around a failing clause and no
/// call anywhere (4b Task 2's report has the programs): depth 18 prints 36,
/// depth 19 prints 38, depth 20 prints 40, and depths 21, 25 and 30 all
/// print 40 as well. Measured the same way with a fragment on top of a
/// nest -- 20 `DO`s around `interpret "do jj = 1 to 1; say 1/0; end"` -- the
/// fragment's own clause would sit at 42 and prints 40, so the cap applies
/// after the activation base is added, not to either part alone.
///
/// **A value line is not capped**, which is what rules out putting this
/// inside `static_indent` or inside [`push_indent`]: measured under `trace
/// r` at nesting depth 25, the `*-*` echo prints 40 while the `>>>` value
/// line for the same clause prints its full 50 ([`push_prefixed_blanks`]'s
/// own `3 + indent`, so 53 blanks after the prefix). One clamp, at the one
/// formatter that has it, and nowhere upstream of that.
pub(crate) const MAX_CLAUSE_INDENT: usize = 40;

/// `TRACE_PREFIX_CLAUSE` (`*-*`): `line`'s own clause, `text`, indented by
/// `indent` spaces (Task 11's `static_indent`, unchanged and reused, per
/// this module's own doc comment on why "when" is not this module's job),
/// clamped at [`MAX_CLAUSE_INDENT`].
///
/// **The only `*-*` formatter in the crate.** `error.rs`'s `Raised::report`
/// used to hold a second copy of these four lines, byte-identical to this
/// one and documented as such -- "one quantity with two formatters, not two
/// quantities", the second formatter D17's own retrofit note names. 4b's
/// Task 2 had to clamp that one quantity and found the cheapest way to keep
/// two formatters agreeing is to have one of them; `report` calls this now,
/// so the clamp is applied once because there is one place to apply it.
pub(crate) fn push_clause(out: &mut Vec<u8>, line: usize, indent: usize, text: &[u8]) {
    let line_start = out.len();
    out.extend_from_slice(format!("{line:>6} *-* ").as_bytes());
    push_indent(out, indent.min(MAX_CLAUSE_INDENT));
    out.extend_from_slice(text);
    out.push(b'\n');
    make_displayable(out, line_start);
}

/// `TRACE_PREFIX_RESULT` (`>>>`) or `TRACE_PREFIX_LITERAL`/`_VARIABLE`/
/// `_PREFIX` when called with the matching `prefix` and no tag: an
/// untagged, quoted value alone -- `push_prefixed_blanks` builds the header
/// and its own fixed-plus-indent gap, this just quotes `value` after it.
///
/// Measured (this task's report, Step 2, `trace_output.rex`): `>L>   "1"`
/// is 7 blanks, `>L>`, 3 blanks, `"1"`.
pub(crate) fn push_value(out: &mut Vec<u8>, prefix: &str, indent: usize, value: &[u8]) {
    let line_start = out.len();
    push_prefixed_blanks(out, prefix, indent);
    push_quoted(out, value);
    make_displayable(out, line_start);
}

/// The shared header every formatter in this module builds first: 7 blanks
/// (the unused six-wide line-number field plus its trailing space,
/// `PREFIX_OFFSET = 7`), then `prefix`, then a further blank run out to
/// where the quoted value or tag starts.
///
/// That trailing run is `3 + indent`, **not** `indent` alone -- read off
/// `RexxActivation.cpp`'s own `dataOffset = TRACE_OVERHEAD +
/// indent_levels*INDENT_SPACING - 2`: with `indent` here already the
/// doubled quantity `static_indent` returns (spaces, not levels), that is
/// `(15 + indent - 2) - 10 = 3 + indent` bytes after the prefix ends at
/// byte 10. Measured the same way, indent 0: `>L>   "1"` is the prefix then
/// exactly 3 blanks then the quote, and `>=>   X <= "2"`'s 3 blanks before
/// `X` are the identical run -- the fixed 3 is shared by every value-
/// bearing line regardless of whether what follows is a bare quote or a
/// tag, only `indent` on top of it moves.
fn push_prefixed_blanks(out: &mut Vec<u8>, prefix: &str, indent: usize) {
    debug_assert_eq!(prefix.len(), 3, "every trace prefix is exactly 3 bytes");
    out.extend(std::iter::repeat_n(b' ', 7));
    out.extend_from_slice(prefix.as_bytes());
    push_indent(out, 3 + indent);
}

/// `"value"`, quoted -- the trailing half of every value-bearing trace line.
fn push_quoted(out: &mut Vec<u8>, value: &[u8]) {
    out.push(b'"');
    out.extend_from_slice(value);
    out.push(b'"');
    out.push(b'\n');
}

/// `TRACE_PREFIX_VARIABLE`/`_ASSIGNMENT`/`_KEYWORD` (`>V>`/`>=>`/`>K>`), the
/// `tag marker "value"` shape `traceTaggedValue` builds. `quote_tag`
/// controls whether `tag` itself is wrapped in quotes -- measured
/// (`traceKeywordResult`'s own call passes `quoteTag = true`, `>K>   "TO"
/// => "2"`; `traceVariable`/`traceAssignment` both pass `false`, `>V>   X
/// => "2"` and `>=>   X <= "2"`, no quotes around the variable name).
pub(crate) fn push_tagged(
    out: &mut Vec<u8>,
    prefix: &str,
    indent: usize,
    quote_tag: bool,
    tag: &[u8],
    marker: &str,
    value: &[u8],
) {
    let line_start = out.len();
    push_prefixed_blanks(out, prefix, indent);
    if quote_tag {
        push_quoted_tag(out, tag);
    } else {
        out.extend_from_slice(tag);
    }
    out.extend_from_slice(marker.as_bytes());
    push_quoted(out, value);
    make_displayable(out, line_start);
}

/// `"tag"` with no trailing newline -- `push_tagged`'s own quoted-tag case,
/// split out because `push_quoted` always terminates its own line and a tag
/// is never the last thing on one.
fn push_quoted_tag(out: &mut Vec<u8>, tag: &[u8]) {
    out.push(b'"');
    out.extend_from_slice(tag);
    out.push(b'"');
}

/// `TRACE_PREFIX_OPERATOR`/`_PREFIX` (`>O>`/`>P>`): `traceOperatorValue`'s
/// own shape, `"op" => "value"` -- always a quoted tag (measured: `>O>
/// "+" => "2"`, unlike `>V>`/`>=>`), so this is `push_tagged` with
/// `quote_tag` fixed to `true` and the arithmetic/comparison/logical
/// operator's own spelling as the tag.
pub(crate) fn push_operator(
    out: &mut Vec<u8>,
    prefix: &str,
    indent: usize,
    op: &[u8],
    value: &[u8],
) {
    push_tagged(out, prefix, indent, true, op, " => ", value);
}

impl Interp {
    /// Whether a `*-*` line would print for a clause of this kind -- the
    /// **one** decision behind both formatters below, and behind the
    /// `run.rs` call site's own guard against building a clause's text when
    /// nothing will print it.
    ///
    /// `all` covers every clause; `labels` covers a `LABEL` clause only, and
    /// is the only field that decides anything under `TRACE L`. The `||`
    /// reduces to `all` in every other mode, because `labels` is true
    /// wherever `all` is (`TraceMode::labels`'s own doc comment has the
    /// flag sets that make that so).
    pub(crate) fn tracing_clause(&self, is_label: bool) -> bool {
        let mode = self.trace_mode();
        mode.all || (mode.labels && is_label)
    }

    /// Appends `*-*`'s own line for a clause that is **never** a `LABEL`:
    /// the loop drivers' own re-echo of a `DO`/`LOOP`/`END`, and a
    /// `WHEN`/`OTHERWISE` header (see `run.rs`'s own doc comments on those
    /// call sites for why a `DO`/`LOOP` needs a second echo and nothing else
    /// built so far does). A label cannot appear at any of them -- one
    /// inside a `DO` block is error 47.2 at parse time, measured.
    pub(crate) fn trace_clause(&mut self, line: usize, indent: usize, text: &[u8]) {
        self.trace_stepped_clause(false, line, indent, text);
    }

    /// The same line for a *stepped* instruction, which is the one place a
    /// `LABEL` reaches the sink: `run.rs`'s `step_in_temps_frame`. Under
    /// `TRACE L` this is the only echo the whole run produces.
    pub(crate) fn trace_stepped_clause(
        &mut self,
        is_label: bool,
        line: usize,
        indent: usize,
        text: &[u8],
    ) {
        if !self.tracing_clause(is_label) {
            return;
        }
        push_clause(&mut self.trace, line, indent, text);
    }

    /// `>>>`, an instruction's own top-level computed value -- gated on
    /// `trace_mode.results`, true for both `TRACE R` and `TRACE I`.
    /// `indent` is the *value line's* own indent, which is the traced
    /// instruction's clause indent (this crate never nests a value line
    /// under anything deeper than its own clause, matching every
    /// transcript in the report: a `say`'s own `>L>`/`>>>` sit at the same
    /// indent as the `say` clause itself, never one level further in).
    pub(crate) fn trace_result(&mut self, indent: usize, value: &[u8]) {
        if !self.trace_mode().results {
            return;
        }
        push_value(&mut self.trace, ">>>", indent, value);
    }

    /// `>K>`, a keyword sub-clause's own value (`DO`'s `TO`/`BY`/`FOR`/
    /// `WHILE`/`UNTIL`, `SELECT CASE`'s `CASE`) -- gated on
    /// `trace_mode.results` like `>>>`, **not** `intermediates`: measured,
    /// `trace r` alone already shows `>K>   "TO" => "2"` with no other
    /// intermediate line anywhere in the same transcript.
    pub(crate) fn trace_keyword(&mut self, indent: usize, keyword: &str, value: &[u8]) {
        if !self.trace_mode().results {
            return;
        }
        push_tagged(
            &mut self.trace,
            ">K>",
            indent,
            true,
            keyword.as_bytes(),
            " => ",
            value,
        );
    }

    /// `>L>`/`>V>`/`>O>`/`>P>` -- `eval.rs`'s own single post-order insertion
    /// point, gated on `trace_mode.intermediates` (`TRACE I` only).
    pub(crate) fn tracing_intermediates(&self) -> bool {
        self.trace_mode().intermediates
    }

    /// `>L>` (`TRACE_PREFIX_LITERAL`): a literal's own value, untagged.
    /// `eval.rs`'s `eval` calls this for `ExprKind::Literal` and
    /// `ExprKind::Constant` alike -- a constant symbol's own upcased
    /// spelling is not distinguished from a quoted literal here, matching
    /// how little either one is: **not independently oracle-probed for
    /// `Constant`** (this task's report says so), reasoned from `Literal`'s
    /// own measured shape rather than a second transcript.
    pub(crate) fn trace_literal(&mut self, indent: usize, value: &[u8]) {
        if !self.trace_mode().intermediates {
            return;
        }
        push_value(&mut self.trace, ">L>", indent, value);
    }

    /// `>V>` (`TRACE_PREFIX_VARIABLE`): a simple variable or bare stem's own
    /// read value, tagged with its own name, unquoted
    /// (`traceVariable`/`RexxActivation.hpp:341`-`342`, `quoteTag = false`).
    pub(crate) fn trace_variable(&mut self, indent: usize, tag: &[u8], value: &[u8]) {
        if !self.trace_mode().intermediates {
            return;
        }
        push_tagged(&mut self.trace, ">V>", indent, false, tag, " => ", value);
    }

    /// `>E>` (`TRACE_PREFIX_DOTVARIABLE`): `.NIL`/`.TRUE`/`.FALSE`, tagged
    /// with the environment symbol's own spelling, unquoted. **Not named in
    /// the design spec's own "measured reachable from pure-4a code" list --
    /// a correction this task found and reports**: measured, `trace i` /
    /// `say .nil` shows `>E>   .NIL => "The NIL object"` *and* `>>>`, both
    /// on pure 4a code (`ExprKind::DotVariable`'s three admissible names are
    /// 4a's own, D15).
    pub(crate) fn trace_dotvar(&mut self, indent: usize, tag: &[u8], value: &[u8]) {
        if !self.trace_mode().intermediates {
            return;
        }
        push_tagged(&mut self.trace, ">E>", indent, false, tag, " => ", value);
    }

    /// `>O>` (`TRACE_PREFIX_OPERATOR`): a binary operator's own result,
    /// tagged with the operator's own spelling, quoted
    /// (`traceOperatorValue` always quotes its tag, unlike `>V>`/`>=>`).
    pub(crate) fn trace_operator(&mut self, indent: usize, op: &[u8], value: &[u8]) {
        if !self.trace_mode().intermediates {
            return;
        }
        push_operator(&mut self.trace, ">O>", indent, op, value);
    }

    /// `>P>` (`TRACE_PREFIX_PREFIX`): a prefix operator's own result, the
    /// identical shape to `>O>` with its own prefix byte
    /// (`tracePrefix`/`RexxActivation.hpp:353`-`354` calls the same
    /// `traceOperatorValue` `traceOperator` does, differing only in which
    /// `TracePrefix` it passes).
    pub(crate) fn trace_prefix_op(&mut self, indent: usize, op: &[u8], value: &[u8]) {
        if !self.trace_mode().intermediates {
            return;
        }
        push_operator(&mut self.trace, ">P>", indent, op, value);
    }

    /// `>=>` (`TRACE_PREFIX_ASSIGNMENT`): a variable was just written.
    /// `tag` is unquoted (`traceAssignment`/`ExpressionVariable.cpp:299`
    /// pass `quoteTag = false`, unlike a keyword's own tag) -- the read
    /// site's own name for a simple variable or a bare stem, or the whole
    /// compound's own source spelling for a compound (`run.rs`'s
    /// `Assignment` arm is the one caller, and its own doc comment says
    /// which for each target shape). Gated on `intermediates`: measured,
    /// `trace r` alone shows `>>>` for an assignment's own value but never
    /// `>=>` (this task's report, `trace_results.rex`'s own transcript).
    pub(crate) fn trace_assignment(&mut self, indent: usize, tag: &[u8], value: &[u8]) {
        if !self.trace_mode().intermediates {
            return;
        }
        push_tagged(&mut self.trace, ">=>", indent, false, tag, " <= ", value);
    }

    /// `>A>` (`TRACE_PREFIX_ARGUMENT`): one call argument's own evaluated
    /// value, untagged, emitted at the *call site* once the argument
    /// expression has produced a value (`RexxInstruction::evaluateArguments`,
    /// `RexxInstruction.cpp:144`-`162`, read directly: `traceArgument` right
    /// after each `evaluate`, and `traceArgument(NULLSTRING)` -- an empty
    /// value line, not a skipped one -- for an omitted position).
    ///
    /// Gated on `intermediates` (`traceArgument`'s own
    /// `if (settings.intermediateTrace)`), measured both ways: `trace i` /
    /// `call sub 1,,3` shows `>A>   "1"`, `>A>   ""`, `>A>   "3"`, and the
    /// same program under `trace r` shows none of the three.
    pub(crate) fn trace_argument(&mut self, indent: usize, value: &[u8]) {
        if !self.trace_mode().intermediates {
            return;
        }
        push_value(&mut self.trace, ">A>", indent, value);
    }

    /// `>F>` (`TRACE_PREFIX_FUNCTION`): a **function-form** call's own
    /// returned value, tagged with the routine's name, unquoted
    /// (`traceFunction`/`RexxActivation.hpp:347`-`348`, `quoteTag = false`,
    /// like `>V>` and unlike `>K>`).
    ///
    /// **The instruction form has no equivalent line**, which is measured
    /// rather than inferred from the C++ alone: `zz = sub(1, 2)` traces
    /// `>F>   SUB => "3"` after the callee's own two `>>>` lines, while
    /// `call sub 1, 2` traces no `>F>` anywhere -- the oracle's own
    /// `traceFunction` call sits in `ExpressionFunction::evaluate`
    /// (`ExpressionFunction.cpp:228`), which the `CALL` instruction does not
    /// go through at all.
    ///
    /// `indent` is the **caller's** own clause indent, not the callee's:
    /// measured, the `>F>` line above sits at the enclosing assignment's
    /// indent while the callee's `>>>` lines sit two further in.
    pub(crate) fn trace_function(&mut self, indent: usize, name: &[u8], value: &[u8]) {
        if !self.trace_mode().intermediates {
            return;
        }
        push_tagged(&mut self.trace, ">F>", indent, false, name, " => ", value);
    }

    /// `>R>` (`TRACE_PREFIX_ALIAS`): a `USE ARG >name` target has just been
    /// aliased onto the caller's variable. `tag` is the **caller's** own
    /// variable name and the value is the **callee's** target name
    /// (`UseInstruction.cpp:167`, `traceVariableAlias(reference->getName(),
    /// useRef->getName())`, that order), so `orig = 'PP'; call sub >orig`
    /// into `use arg >q` traces `>R>     "ORIG" => "Q"` -- names on both
    /// sides, no value anywhere on the line.
    ///
    /// **Gated on `results`, not `intermediates`** (`traceVariableAlias`'s
    /// own `if (tracingResults())`, unlike every other prefix this task
    /// added): measured, the same program under `trace r` still shows the
    /// `>R>` line with no other value line around it, and under `trace l`
    /// shows **no `>R>`** -- not "nothing at all", which this sentence said
    /// and which the same task's own measurements falsify (review round 1,
    /// F5): under `trace l` both sides echo the callee's `sub:` label
    /// clause and neither emits a value line of any kind.
    pub(crate) fn trace_alias(&mut self, indent: usize, reference: &[u8], target: &[u8]) {
        if !self.trace_mode().results {
            return;
        }
        push_tagged(
            &mut self.trace,
            ">R>",
            indent,
            true,
            reference,
            " => ",
            target,
        );
    }

    /// `>.>` (`TRACE_PREFIX_DUMMY`): what a `PARSE` template's `.`
    /// placeholder just consumed, untagged, with no assignment behind it.
    ///
    /// Gated on `intermediates` (`ParseTrigger.cpp:285` calls
    /// `traceIntermediate`, whose own body is `if
    /// (settings.intermediateTrace)`, `RexxActivation.hpp:339`), which is
    /// measured rather than taken from the C++ alone: `parse value 'one two'
    /// with p . q` under `trace i` emits `>.>   ""` between the two `>=>`
    /// lines, and the same program under `trace r` emits no `>.>` at all --
    /// two `>>>` lines for `P` and `Q` and nothing for the placeholder.
    ///
    /// **Emitted even when the placeholder consumed nothing**, which is why
    /// the line above is `>.>   ""` and not absent.
    pub(crate) fn trace_dummy(&mut self, indent: usize, value: &[u8]) {
        if !self.trace_mode().intermediates {
            return;
        }
        push_value(&mut self.trace, ">.>", indent, value);
    }

    /// `>C>` (`TRACE_PREFIX_COMPOUND`): announces which fully-resolved
    /// compound name a read or write just used, before `>V>`/`>=>` shows
    /// what is actually stored there -- `tag` is the compound's own
    /// *unresolved* source spelling (`code.symbols.name(id)`, e.g. `A.I`),
    /// `resolved` is the fully-resolved name (`A.1`) `derived_tail_name`'s
    /// own shape builds (measured, `evaluateLocalCompoundVariable`,
    /// `RexxActivation.cpp:4791`-`4802`: `traceCompoundName` then
    /// `traceCompound`/`traceAssignment`, always both, whether or not the
    /// tail actually resolves to a stored value). Gated on `intermediates`
    /// like every other value-prefix line.
    pub(crate) fn trace_compound_name(&mut self, indent: usize, tag: &[u8], resolved: &[u8]) {
        if !self.trace_mode().intermediates {
            return;
        }
        push_tagged(&mut self.trace, ">C>", indent, false, tag, " => ", resolved);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every formatter that finishes a line puts it through the oracle's
    /// display rule, and the rule reaches the whole line rather than the
    /// quoted value alone.
    ///
    /// Measured: `trace r` over `say 'p'||'02'x||'q'` prints `>>>   "p?q"`
    /// on the oracle, with the raw byte still on stdout where it belongs.
    /// The clause echo carries source bytes and obeys the same rule; the
    /// tag does too, which is why `push_tagged` is checked separately from
    /// `push_value` rather than assumed to follow from it.
    #[test]
    fn every_completed_trace_line_is_made_displayable() {
        let mut out = Vec::new();
        push_value(&mut out, ">>>", 0, b"p\x02q");
        assert_eq!(out, b"       >>>   \"p?q\"\n");

        out.clear();
        push_clause(&mut out, 2, 0, b"say 'p'\x02'q'");
        assert_eq!(out, b"     2 *-* say 'p'?'q'\n");

        out.clear();
        push_tagged(&mut out, ">V>", 0, false, b"Z\x03Z", " => ", b"a\x04b");
        assert_eq!(out, b"       >V>   Z?Z => \"a?b\"\n");

        out.clear();
        push_operator(&mut out, ">O>", 0, b"|\x05|", b"a\x06b");
        assert_eq!(out, b"       >O>   \"|?|\" => \"a?b\"\n");

        // A byte at or above 0x80 is not sanitised, and neither are the
        // three control bytes the rule spares. Without these the rule could
        // be "every byte outside printable ASCII" and every line above would
        // still pass.
        out.clear();
        push_value(&mut out, ">>>", 0, b"\xff\x09\x0d\x80");
        assert_eq!(out, b"       >>>   \"\xff\x09\x0d\x80\"\n");
    }

    /// `mode_from_setting`'s own nine-letter table plus the two silent
    /// defaults (empty, all-`?`) -- every arm this function has, asserted
    /// once each rather than only through the corpus's own two programs.
    #[test]
    fn every_recognised_letter_classifies_and_unrecognised_ones_report_the_byte() {
        assert_eq!(mode_from_setting(b""), Ok(TraceMode::OFF));
        assert_eq!(mode_from_setting(b"?"), Ok(TraceMode::OFF));
        assert_eq!(mode_from_setting(b"??"), Ok(TraceMode::OFF));
        assert_eq!(mode_from_setting(b"a"), Ok(TraceMode::ALL));
        assert_eq!(mode_from_setting(b"?R"), Ok(TraceMode::RESULTS));
        assert_eq!(mode_from_setting(b"i"), Ok(TraceMode::INTERMEDIATES));
        assert_eq!(mode_from_setting(b"results"), Ok(TraceMode::RESULTS));
        // `L` left this group at review round 1 (F8): it is the one letter
        // here that has something to show, and this assertion is what went
        // red when it moved. Both spellings, since `TRACE VALUE 'l'` reaches
        // the same classifier as `TRACE ?L` does.
        assert_eq!(mode_from_setting(b"L"), Ok(TraceMode::LABELS));
        assert_eq!(mode_from_setting(b"?l"), Ok(TraceMode::LABELS));
        assert_eq!(
            (TraceMode::LABELS.labels, TraceMode::LABELS.all),
            (true, false),
            "L echoes labels and nothing else"
        );
        // `labels` is set wherever `all` is, so the `||` in `tracing_clause`
        // reduces to `all` in every mode but `L` -- the property that keeps
        // this a change to one letter's behaviour rather than to four.
        for mode in [TraceMode::ALL, TraceMode::RESULTS, TraceMode::INTERMEDIATES] {
            assert!(mode.labels && mode.all, "{mode:?}");
        }
        for letter in b"CEFNO" {
            assert_eq!(
                mode_from_setting(&[*letter]),
                Ok(TraceMode::OFF),
                "{}",
                *letter as char
            );
        }
        assert_eq!(mode_from_setting(b"z"), Err(b'z'));
        assert_eq!(mode_from_setting(b"?z"), Err(b'z'));
    }

    /// `is_whole_number`'s own oracle-measured pair: `trace value 5` raises
    /// 24.901 exactly like `trace 5` (a digit string is a skip count), and
    /// `trace value 'R'` behaves exactly like `trace r` (a letter is a
    /// setting) -- this task's report has both transcripts.
    #[test]
    fn a_digit_string_is_a_whole_number_and_a_letter_is_not() {
        assert!(is_whole_number(b"5"));
        assert!(is_whole_number(b"0"));
        assert!(is_whole_number(b"-3"));
        assert!(!is_whole_number(b"R"));
        assert!(!is_whole_number(b""));
        assert!(!is_whole_number(b"5x"));
    }

    /// The formatting functions, checked against the exact bytes this
    /// task's report captured from the oracle (`cat -A`, `trace_output.rex`
    /// under `TRACE I`) -- one assertion per prefix shape, independent of
    /// whichever `run.rs`/`eval.rs` call site ends up calling each one.
    #[test]
    fn every_formatter_matches_its_own_oracle_transcript() {
        let mut out = Vec::new();
        push_clause(&mut out, 2, 0, b"x = 1 + 1");
        assert_eq!(out, b"     2 *-* x = 1 + 1\n");

        let mut out = Vec::new();
        push_value(&mut out, ">L>", 0, b"1");
        assert_eq!(out, b"       >L>   \"1\"\n");

        let mut out = Vec::new();
        push_operator(&mut out, ">O>", 0, b"+", b"2");
        assert_eq!(out, b"       >O>   \"+\" => \"2\"\n");

        let mut out = Vec::new();
        push_value(&mut out, ">>>", 0, b"2");
        assert_eq!(out, b"       >>>   \"2\"\n");

        let mut out = Vec::new();
        push_tagged(&mut out, ">=>", 0, false, b"X", " <= ", b"2");
        assert_eq!(out, b"       >=>   X <= \"2\"\n");

        let mut out = Vec::new();
        push_tagged(&mut out, ">V>", 0, false, b"X", " => ", b"2");
        assert_eq!(out, b"       >V>   X => \"2\"\n");

        // Indent 4, the say-inside-a-matched-THEN shape:
        // `       >L>       "big"` -- one clause's own value lines sit at
        // the clause's own indent, never deeper.
        let mut out = Vec::new();
        push_value(&mut out, ">L>", 4, b"big");
        assert_eq!(out, b"       >L>       \"big\"\n");

        // 4b's three, each from this task's own oracle transcript.
        // `>A>`, indent 0: `call sub 1,,3` under `trace i`.
        let mut out = Vec::new();
        push_value(&mut out, ">A>", 0, b"1");
        assert_eq!(out, b"       >A>   \"1\"\n");

        // The omitted position's own line, which is empty rather than
        // absent -- the same formatter with no value.
        let mut out = Vec::new();
        push_value(&mut out, ">A>", 0, b"");
        assert_eq!(out, b"       >A>   \"\"\n");

        // `>F>`, indent 0, tag *unquoted*: `zz = sub(1, 2)` under `trace i`.
        let mut out = Vec::new();
        push_tagged(&mut out, ">F>", 0, false, b"SUB", " => ", b"3");
        assert_eq!(out, b"       >F>   SUB => \"3\"\n");

        // `>R>`, indent 2, tag *quoted* -- the callee's own indent, since a
        // `USE ARG` sits inside a called routine by construction.
        let mut out = Vec::new();
        push_tagged(&mut out, ">R>", 2, true, b"ORIG", " => ", b"Q");
        assert_eq!(out, b"       >R>     \"ORIG\" => \"Q\"\n");
    }

    /// The 4b prefixes' own gates, which are **not** all the same one:
    /// `>A>`/`>F>` are `intermediates` and `>R>` is `results`
    /// (`traceArgument`/`traceFunction` against `traceVariableAlias`,
    /// `RexxActivation.hpp:340`/`:347`/`:370`, and measured -- `trace r` on
    /// an aliasing `USE ARG >q` shows `>R>` and nothing else).
    ///
    /// Written as three modes against three prefixes rather than one
    /// assertion per prefix, because the failure this catches is a gate
    /// copied from the neighbouring method: under `RESULTS` a wrongly
    /// `intermediates`-gated `>R>` disappears, and under `RESULTS` a wrongly
    /// `results`-gated `>A>`/`>F>` appear.
    #[test]
    fn the_two_argument_prefixes_are_intermediates_and_the_alias_prefix_is_results() {
        let mut interp = Interp::new();
        activate_empty(&mut interp);
        interp.set_trace_mode(TraceMode::RESULTS);
        interp.trace_argument(0, b"1");
        interp.trace_function(0, b"SUB", b"3");
        interp.trace_alias(0, b"ORIG", b"Q");
        assert_eq!(
            interp.trace, b"       >R>   \"ORIG\" => \"Q\"\n",
            "under RESULTS only the alias line is shown"
        );

        let mut interp = Interp::new();
        activate_empty(&mut interp);
        interp.set_trace_mode(TraceMode::INTERMEDIATES);
        interp.trace_argument(0, b"1");
        interp.trace_function(0, b"SUB", b"3");
        interp.trace_alias(0, b"ORIG", b"Q");
        assert_eq!(
            interp.trace,
            b"       >A>   \"1\"\n       >F>   SUB => \"3\"\n       >R>   \"ORIG\" => \"Q\"\n",
            "under INTERMEDIATES all three are shown"
        );

        let mut interp = Interp::new();
        activate_empty(&mut interp);
        interp.trace_argument(0, b"1");
        interp.trace_function(0, b"SUB", b"3");
        interp.trace_alias(0, b"ORIG", b"Q");
        assert!(interp.trace.is_empty(), "TraceMode::OFF is silent");
    }

    /// `Interp::trace_clause`/`trace_result`/`trace_keyword` are each
    /// gated correctly -- silent under `TraceMode::OFF`, and each fires
    /// only under the mode that names it (`>>>`/`>K>` under `results`,
    /// **not** requiring `intermediates`, matching `trace r` alone already
    /// showing both).
    /// Pushes the one activation these gates read their mode from.
    ///
    /// Needed since Task 3 moved `trace_mode` from `Interp` onto
    /// `Activation`: `Interp::trace_mode` is the *running* activation's, so
    /// there has to be one before anything can be traced or set. The program
    /// is empty because nothing here runs it -- the tests below call the
    /// three sink functions directly.
    fn activate_empty(interp: &mut Interp) {
        use std::rc::Rc;

        let program = Rc::new(rexx_parse::parse_program(Vec::new()).expect("the empty program"));
        let id = crate::plan::ProgramId(interp.programs.len());
        interp.programs.push(Rc::clone(&program));
        let plan = interp.plan_for(
            crate::plan::BodyKey {
                program: id,
                directive: None,
            },
            &program.main,
            &program.symbols,
        );
        let frame = interp.roots.push_slots(plan.len());
        let id = interp.next_activation_id();
        interp
            .activations
            .push(crate::Activation::new(id, program, plan, frame));
    }

    #[test]
    fn the_three_gates_fire_under_exactly_the_modes_that_should_show_them() {
        let mut interp = Interp::new();
        activate_empty(&mut interp);
        interp.trace_clause(1, 0, b"say 1");
        interp.trace_result(0, b"1");
        interp.trace_keyword(0, "TO", b"2");
        assert!(interp.trace.is_empty(), "TraceMode::OFF is silent");

        let mut interp = Interp::new();
        activate_empty(&mut interp);
        interp.set_trace_mode(TraceMode::RESULTS);
        interp.trace_clause(1, 0, b"say 1");
        interp.trace_result(0, b"1");
        interp.trace_keyword(0, "TO", b"2");
        assert_eq!(
            interp.trace,
            b"     1 *-* say 1\n       >>>   \"1\"\n       >K>   \"TO\" => \"2\"\n"
        );
    }
}
