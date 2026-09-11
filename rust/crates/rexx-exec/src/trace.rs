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

use crate::Interp;
use crate::error::Raised;
use rexx_core::ObjRef;
use rexx_num::Number;
use rexx_parse::{Operator, PrefixOp};

/// The visible-output shape of the current `TRACE` setting, restricted to
/// what pure-4a code can ever produce (D18 excludes commands, so
/// `traceCommands`/`traceErrors`/`traceFailures` have nothing to show;
/// interactive debug pausing does not exist on this non-interactive runtime
/// at all).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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
    pub(crate) labels: bool,
    /// The byte `TraceSetting::toString` (`runtime/TraceSetting.cpp:62`-`119`)
    /// renders this setting as, and so the byte `TRACE()` answers.
    pub(crate) letter: u8,
}

impl TraceMode {
    /// `TraceSetting::setTraceOff`/`setTraceNormal`/`setTraceCommands`/
    /// `setTraceErrors`/`setTraceFailures` -- every one of these sets a flag
    /// this crate's own scope has nothing to show for (D18 excludes
    /// commands; errors/failures are command-condition machinery, not built
    /// here), so all five behave identically here.
    pub(crate) const OFF: TraceMode = TraceMode {
        all: false,
        results: false,
        intermediates: false,
        labels: false,
        letter: b'O',
    };
    /// `TRACE N` (`setTraceNormal`), and the setting every activation starts
    /// under. Behaves exactly as [`TraceMode::OFF`]; see that constant for
    /// why the two are separate.
    pub(crate) const NORMAL: TraceMode = TraceMode {
        letter: b'N',
        ..TraceMode::OFF
    };
    /// `TRACE C` (`setTraceCommands`). Behaves exactly as
    /// [`TraceMode::OFF`], since D18 excludes commands.
    const COMMANDS: TraceMode = TraceMode {
        letter: b'C',
        ..TraceMode::OFF
    };
    /// `TRACE E` (`setTraceErrors`). Behaves exactly as [`TraceMode::OFF`]:
    /// an `ERROR` condition is command-condition machinery.
    const ERRORS: TraceMode = TraceMode {
        letter: b'E',
        ..TraceMode::OFF
    };
    /// `TRACE F` (`setTraceFailures`). Behaves exactly as
    /// [`TraceMode::OFF`], for [`TraceMode::ERRORS`]' reason.
    const FAILURES: TraceMode = TraceMode {
        letter: b'F',
        ..TraceMode::OFF
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
        letter: b'L',
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
        letter: b'A',
    };
    /// `TRACE R` (`setTraceResults`, `traceResultsFlags`).
    const RESULTS: TraceMode = TraceMode {
        all: true,
        results: true,
        intermediates: false,
        labels: true,
        letter: b'R',
    };
    /// `TRACE I` (`setTraceIntermediates`, `traceIntermediatesFlags`).
    const INTERMEDIATES: TraceMode = TraceMode {
        all: true,
        results: true,
        intermediates: true,
        labels: true,
        letter: b'I',
    };
}

/// Everything `crate::ir::compile` is allowed to read out of a [`TraceMode`],
/// and so everything a compiled chunk's identity depends on.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct ChunkTrace(u8);

impl ChunkTrace {
    /// [`TraceMode::all`]: every stepped clause echoes.
    const CLAUSES: u8 = 1;
    /// [`TraceMode::labels`]: a `LABEL` clause echoes.
    const LABELS: u8 = 2;
    /// [`TraceMode::intermediates`]: a literal, a read, an operator's result,
    /// an argument and a call's result each echo a line of their own.
    const INTERMEDIATES: u8 = 4;

    /// [`TraceMode::results`]: a traced instruction's own top-level computed
    /// value echoes, as `>>>` or -- for a loop header -- as `>K>`.
    const RESULTS: u8 = 8;

    /// What `compile` reads out of the setting in force.
    #[inline(always)]
    pub(crate) fn of(mode: TraceMode) -> ChunkTrace {
        // **One byte and not three `bool` fields, which is a measurement.**
        // `Interp::run_ops`'s staleness check compares a whole `ChunkTrace`
        // once per promoted clause, so the type's width is on that path: as
        // three fields the comparison cost `bench-programs/emptyloop.rex`
        // 0.723% and the pinned `rexxcps` 0.372% against two, where packed it
        // costs neither.
        ChunkTrace(
            (u8::from(mode.all) * ChunkTrace::CLAUSES)
                | (u8::from(mode.labels) * ChunkTrace::LABELS)
                | (u8::from(mode.intermediates) * ChunkTrace::INTERMEDIATES)
                | (u8::from(mode.results) * ChunkTrace::RESULTS),
        )
    }

    /// Whether a `*-*` line prints for a clause of this kind.
    #[inline(always)]
    pub(crate) fn intermediates(self) -> bool {
        self.0 & ChunkTrace::INTERMEDIATES != 0
    }

    /// Whether a top-level computed value echoes -- `>>>` and the loop
    /// header's `>K>` -- which is what decides whether a chunk carries the op
    /// that emits the latter.
    #[inline(always)]
    pub(crate) fn results(self) -> bool {
        self.0 & ChunkTrace::RESULTS != 0
    }

    /// This setting with [`ChunkTrace::INTERMEDIATES`] and
    /// [`ChunkTrace::RESULTS`] masked off, which is what `Interp::run_ops`
    /// compares its chunk against per promoted clause.
    #[inline(always)]
    pub(crate) fn clause_echoes(self) -> ChunkTrace {
        ChunkTrace(self.0 & (ChunkTrace::CLAUSES | ChunkTrace::LABELS))
    }

    #[inline(always)]
    pub(crate) fn echoes(self, is_label: bool) -> bool {
        self.0 & ChunkTrace::CLAUSES != 0 || (self.0 & ChunkTrace::LABELS != 0 && is_label)
    }
}

/// Classifies a `TRACE` option string exactly like
/// `TraceSetting::parseTraceSetting` (`TraceSetting.cpp:135`-`210`): skip any
/// number of leading `?`s (a debug-pause toggle this non-interactive runtime
/// has nothing to toggle, so simply skipped rather than tracked), and the
/// first *other* byte decides, case-insensitively; everything after that one
/// byte is ignored. An empty string, or one made only of `?`s, is
/// `setTraceNormal`'s silent answer.
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
            b'C' => Ok(TraceMode::COMMANDS),
            b'E' => Ok(TraceMode::ERRORS),
            b'F' => Ok(TraceMode::FAILURES),
            b'N' => Ok(TraceMode::NORMAL),
            b'O' => Ok(TraceMode::OFF),
            _ => Err(byte),
        };
    }
    if bytes.is_empty() {
        return Ok(TraceMode::NORMAL);
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
fn make_displayable(out: &mut [u8], line_start: usize) {
    crate::error::displayable(&mut out[line_start..]);
}

/// The widest indent a `*-*` clause echo ever prints, in spaces.
pub(crate) const MAX_CLAUSE_INDENT: usize = 40;

/// `TRACE_PREFIX_CLAUSE` (`*-*`): `line`'s own clause, `text`, indented by
/// `indent` spaces (Task 11's `static_indent`, unchanged and reused, per
/// this module's own doc comment on why "when" is not this module's job),
/// clamped at [`MAX_CLAUSE_INDENT`].
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
    #[inline(always)]
    pub(crate) fn tracing_clause(&self, is_label: bool) -> bool {
        self.chunk_trace().echoes(is_label)
    }

    /// The part of the setting in force that a chunk's identity depends on
    /// ([`ChunkTrace`]).
    #[inline(always)]
    pub(crate) fn chunk_trace(&self) -> ChunkTrace {
        ChunkTrace::of(self.trace_mode())
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
    #[inline(always)]
    pub(crate) fn trace_result(&mut self, indent: usize, value: &[u8]) {
        if !self.trace_mode().results {
            return;
        }
        self.trace_result_line(indent, value);
    }

    /// The line itself, out of line: the gate above is what every call site
    /// pays when tracing is off, and it is one load and a branch.
    #[inline(never)]
    fn trace_result_line(&mut self, indent: usize, value: &[u8]) {
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
    #[inline(always)]
    pub(crate) fn trace_literal(&mut self, indent: usize, value: &[u8]) {
        if !self.trace_mode().intermediates {
            return;
        }
        self.trace_literal_line(indent, value);
    }

    /// The line itself, out of line: the gate above is what every call site
    /// pays when tracing is off, and it is one load and a branch.
    #[inline(never)]
    fn trace_literal_line(&mut self, indent: usize, value: &[u8]) {
        push_value(&mut self.trace, ">L>", indent, value);
    }

    /// One literal's own `>L>` line, from the value rather than from its text.
    #[inline(always)]
    pub(crate) fn echo_literal(&mut self, value: ObjRef) {
        if !self.tracing_intermediates() {
            return;
        }
        self.echo_literal_line(value);
    }

    /// The rendering and the line, out of line behind [`echo_literal`]'s gate.
    #[inline(never)]
    fn echo_literal_line(&mut self, value: ObjRef) {
        let indent = self.clause_state.current_value_indent;
        let text = self.to_text(value).to_vec();
        self.trace_literal(indent, &text);
    }

    /// `>V>` (`TRACE_PREFIX_VARIABLE`): a simple variable or bare stem's own
    /// read value, tagged with its own name, unquoted
    /// (`traceVariable`/`RexxActivation.hpp:341`-`342`, `quoteTag = false`).
    #[inline(always)]
    pub(crate) fn trace_variable(&mut self, indent: usize, tag: &[u8], value: &[u8]) {
        if !self.trace_mode().intermediates {
            return;
        }
        self.trace_variable_line(indent, tag, value);
    }

    /// The line itself, out of line: the gate above is what every call site
    /// pays when tracing is off, and it is one load and a branch.
    #[inline(never)]
    fn trace_variable_line(&mut self, indent: usize, tag: &[u8], value: &[u8]) {
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

    /// `>N>` (`TRACE_PREFIX_NAMESPACE`): a namespace-qualified class lookup's
    /// result, tagged with `namespace:class` and unquoted --
    /// `traceClassResolution` builds the tag as `n->concatWith(c, ':')` and
    /// passes `quoteTag` false (`RexxActivation.hpp:358`).
    pub(crate) fn trace_namespace(&mut self, indent: usize, tag: &[u8], value: &[u8]) {
        if !self.trace_mode().intermediates {
            return;
        }
        push_tagged(&mut self.trace, ">N>", indent, false, tag, " => ", value);
    }

    /// The `>O>` line one binary operator's result owes, from the value
    /// itself, at the indent the clause in force is tracing values at.
    pub(crate) fn echo_operator(&mut self, op: Operator, value: ObjRef) {
        if !self.tracing_intermediates() {
            return;
        }
        let indent = self.clause_state.current_value_indent;
        let text = self.to_text(value).to_vec();
        self.trace_operator(indent, op.spelling().as_bytes(), &text);
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

    /// The `>P>` line one prefix operator's result owes, from the value
    /// itself, at the indent the clause in force is tracing values at.
    pub(crate) fn echo_prefix_op(&mut self, op: PrefixOp, value: ObjRef) {
        if !self.tracing_intermediates() {
            return;
        }
        let indent = self.clause_state.current_value_indent;
        let text = self.to_text(value).to_vec();
        self.trace_prefix_op(indent, op.spelling().as_bytes(), &text);
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
    pub(crate) fn trace_function(&mut self, indent: usize, name: &[u8], value: &[u8]) {
        if !self.trace_mode().intermediates {
            return;
        }
        push_tagged(&mut self.trace, ">F>", indent, false, name, " => ", value);
    }

    /// `>M>` (`TRACE_PREFIX_MESSAGE`): a message send's own result, tagged
    /// with the message name, **quoted** (`traceMessage`,
    /// `RexxActivation.hpp:349`, `quoteTag = true`, unlike `>F>`).
    pub(crate) fn trace_message(&mut self, indent: usize, name: &[u8], value: &[u8]) {
        if !self.trace_mode().intermediates {
            return;
        }
        push_tagged(&mut self.trace, ">M>", indent, true, name, " => ", value);
    }

    /// `>R>` (`TRACE_PREFIX_ALIAS`): a `USE ARG >name` target has just been
    /// aliased onto the caller's variable. `tag` is the **caller's** own
    /// variable name and the value is the **callee's** target name
    /// (`UseInstruction.cpp:167`, `traceVariableAlias(reference->getName(),
    /// useRef->getName())`, that order), so `orig = 'PP'; call sub >orig`
    /// into `use arg >q` traces `>R>     "ORIG" => "Q"` -- names on both
    /// sides, no value anywhere on the line.
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

    /// `value`'s rendered bytes, **or `None` when no intermediate-value
    /// trace line would print them**.
    #[inline(always)]
    pub(crate) fn intermediate_text(&mut self, value: ObjRef) -> Option<Vec<u8>> {
        self.trace_mode()
            .intermediates
            .then(|| self.string_value_text(value))
    }

    /// `value`'s rendered bytes, or `None` when no **result-level** trace line
    /// would print them -- [`Interp::intermediate_text`]'s sibling, and its
    /// doc has the argument for both.
    #[inline(always)]
    pub(crate) fn result_text(&mut self, value: ObjRef) -> Option<Vec<u8>> {
        self.trace_mode()
            .results
            .then(|| self.string_value_text(value))
    }

    /// `>I>`/`<I<` (`TRACE_PREFIX_INVOCATION`/`_INVOCATION_EXIT`): a routine
    /// activation's own entry and exit announcement.
    /// ```text
    ///        >I> Routine "RTN" in package "/abs/path/own_a.rex".$
    ///        <I< Routine "RTN" in package "/abs/path/own_a.rex".$
    ///        >I> Method "M" with scope "K" in package "/abs/path/p5.rex".$
    ///        <I< Method "M" with scope "K" in package "/abs/path/p5.rex".$
    /// ```
    pub(crate) fn trace_invocation(&mut self, prefix: &str, subject: &Announced, package: &[u8]) {
        let line_start = self.trace.len();
        self.trace.extend(std::iter::repeat_n(b' ', 7));
        self.trace.extend_from_slice(prefix.as_bytes());
        match subject {
            Announced::Routine { name } => {
                self.trace.extend_from_slice(b" Routine \"");
                self.trace.extend_from_slice(name);
            }
            Announced::Method { name, scope } => {
                self.trace.extend_from_slice(b" Method \"");
                self.trace.extend_from_slice(name);
                self.trace.extend_from_slice(b"\" with scope \"");
                self.trace.extend_from_slice(scope);
            }
        }
        self.trace.extend_from_slice(b"\" in package \"");
        self.trace.extend_from_slice(package);
        self.trace.extend_from_slice(b"\".\n");
        make_displayable(&mut self.trace, line_start);
    }
}

/// What a `>I>`/`<I<` pair names, and which of the two messages it takes.
pub(crate) enum Announced {
    /// A `::ROUTINE`. `name` is the directive's own spelling -- upcased for
    /// the bare symbol form because the scanner upcases it, and left alone
    /// for the quoted form (measured: `::routine 'zork'` announces `"zork"`
    /// and `::routine MiXeD` announces `"MIXED"`).
    Routine { name: Vec<u8> },
    /// A `::METHOD`. `name` is the **message** name, already upcased
    /// (measured: `::method MiXeD class` announces `"MIXED"` and `::method
    /// "quoted" class` announces `"QUOTED"`), and `scope` is the defining
    /// class's `~id`, unmodified (measured: `::class 'k'` announces
    /// `with scope "k"`).
    Method { name: Vec<u8>, scope: Vec<u8> },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every formatter that finishes a line puts it through the oracle's
    /// display rule, and the rule reaches the whole line rather than the
    /// quoted value alone.
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
        // The empty setting is `setTraceNormal`, the all-`?` one is the
        // debug toggle this crate answers `OFF` for -- `mode_from_setting`'s
        // own doc has the measurement and the owner for each.
        assert_eq!(mode_from_setting(b""), Ok(TraceMode::NORMAL));
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
        // The five silent settings behave identically and are still five
        // distinct answers, because `TRACE()` reports the letter. Both
        // halves are asserted: same behaviour, different name.
        for letter in b"CEFNO" {
            let mode = mode_from_setting(&[*letter]).expect("a recognised letter");
            assert_eq!(
                (mode.all, mode.results, mode.intermediates, mode.labels),
                (false, false, false, false),
                "{} is silent",
                *letter as char
            );
            assert_eq!(
                mode.letter, *letter,
                "{} keeps its own name",
                *letter as char
            );
            assert_eq!(
                mode_from_setting(&[letter.to_ascii_lowercase()]),
                Ok(mode),
                "{} classifies case-insensitively",
                *letter as char
            );
        }
        // Every letter round-trips: what `mode_from_setting` accepts is what
        // `TRACE()` reports back, for all nine. A mapping that collapsed two
        // letters onto one constant fails here rather than only in a corpus
        // program that happens to use the second one.
        for letter in b"ACEFILNOR" {
            let mode = mode_from_setting(&[*letter]).expect("a recognised letter");
            assert_eq!(mode.letter, *letter, "{}", *letter as char);
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
    fn activate_empty(interp: &mut Interp) {
        use std::rc::Rc;

        let program = Rc::new(rexx_parse::parse_program(Vec::new()).expect("the empty program"));
        let program_id = crate::plan::ProgramId(interp.programs.len());
        interp.programs.push(Rc::clone(&program));
        let plan = interp.plan_for(
            crate::plan::BodyKey {
                program: program_id,
                directive: None,
            },
            &program.main,
            &program.symbols,
            &program.source,
        );
        let frame = interp.roots.push_slots(plan.len());
        let id = interp.next_activation_id();
        interp.push_activation(crate::Activation::new(id, program, program_id, plan, frame));
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
