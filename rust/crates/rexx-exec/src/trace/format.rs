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

//! The mode a `TRACE` setting parses into, and the byte layout of each
//! prefix's line: pure functions on bytes, which `trace.rs`'s emission calls.

use super::{Number, Raised, Trace};

/// The visible-output shape of the current `TRACE` setting, and whether
/// interactive debug is on. The separate `pauseInstructions`/`pauseLabels`/
/// `pauseCommands` flags have no field: a pause follows every clause the
/// letter in force traced, so [`TraceMode::debug`] and the letter decide it
/// together.
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
    /// `TraceSetting::tracingCommands`: a command clause echoes `*-*` and its
    /// evaluated string as `>>>`, before the command runs. Measured under
    /// `TRACE C`, where a command that succeeds still shows both and nothing
    /// else in the program shows anything.
    pub(crate) commands: bool,
    /// `TraceSetting::tracingErrors`: a command that raised `ERROR` echoes
    /// `*-*` and `>>>` **after** it has run, where the setting did not
    /// already echo them.
    pub(crate) errors: bool,
    /// `TraceSetting::tracingFailures`, the same for `FAILURE`. Set under
    /// `TRACE N`, which is why a failing command traces itself in a program
    /// carrying no `TRACE` instruction at all.
    pub(crate) failures: bool,
    /// The byte `TraceSetting::toString` (`runtime/TraceSetting.cpp:62`-`119`)
    /// renders this setting as, and so the byte `TRACE()` answers.
    pub(crate) letter: u8,
    /// `TraceSetting::traceDebug`, the `?` prefix: every traced clause is
    /// followed by a pause. `TRACE()` answers it as a `?` before the letter.
    pub(crate) debug: bool,
}

impl TraceMode {
    /// `TraceSetting::setTraceOff`: every flag clear, and the one setting
    /// under which a failing command shows nothing.
    pub(crate) const OFF: TraceMode = TraceMode {
        all: false,
        results: false,
        intermediates: false,
        labels: false,
        commands: false,
        errors: false,
        failures: false,
        letter: b'O',
        debug: false,
    };
    /// `TRACE N` (`setTraceNormal`), and the setting every activation starts
    /// under. `defaultTraceFlags` is `traceNormal` **and `traceFailures`**
    /// (`TraceSetting.cpp:49`), so this is not [`TraceMode::OFF`] with
    /// another letter: a command that raises `FAILURE` traces itself here.
    pub(crate) const NORMAL: TraceMode = TraceMode {
        failures: true,
        letter: b'N',
        ..TraceMode::OFF
    };
    /// `TRACE C` (`setTraceCommands`): the single instruction type, so a
    /// command clause echoes and nothing else in the program does.
    const COMMANDS: TraceMode = TraceMode {
        commands: true,
        letter: b'C',
        ..TraceMode::OFF
    };
    /// `TRACE E` (`setTraceErrors`), which sets `traceFailures` beside
    /// `traceErrors` -- "just errors includes failures", `TraceSetting.hpp`.
    const ERRORS: TraceMode = TraceMode {
        errors: true,
        failures: true,
        letter: b'E',
        ..TraceMode::OFF
    };
    /// `TRACE F` (`setTraceFailures`): failures and, unlike
    /// [`TraceMode::ERRORS`], not errors. Measured -- `exit 127` traces and
    /// `exit 3` does not.
    const FAILURES: TraceMode = TraceMode {
        failures: true,
        letter: b'F',
        ..TraceMode::OFF
    };
    /// `TRACE L` (`setTraceLabels`, which resets every other flag and sets
    /// `traceLabels` alone). The one mode where `labels` decides anything:
    /// every executed `LABEL` clause echoes and nothing else does.
    /// `tests/trace_oracle/trace_labels.rex` carries the probed routes and
    /// says why they are examples rather than an enumeration.
    pub(super) const LABELS: TraceMode = TraceMode {
        all: false,
        results: false,
        intermediates: false,
        labels: true,
        commands: false,
        errors: false,
        failures: false,
        letter: b'L',
        debug: false,
    };
    /// `TRACE A` (`setTraceAll`, `traceAllFlags`): every clause echoes, but
    /// `traceAllFlags` deliberately omits `traceResults` -- measured
    /// nowhere in this task's own corpus, but stated here because
    /// `mode_from_setting` needs a real answer for the letter, not a
    /// silent fallback that would misreport `TRACE A x = 1` as producing no
    /// `*-*` line at all.
    pub(super) const ALL: TraceMode = TraceMode {
        all: true,
        results: false,
        intermediates: false,
        labels: true,
        // `traceAllFlags` carries `traceCommands` and neither of the other
        // two (`TraceSetting.cpp:52`), so a command's `>>>` here comes from
        // this flag rather than from `results`.
        commands: true,
        errors: false,
        failures: false,
        letter: b'A',
        debug: false,
    };
    /// `TRACE R` (`setTraceResults`, `traceResultsFlags`). Also the half of
    /// `setExternalTrace` that is not the debug flag, which is what
    /// `RXTRACE=ON` starts a program under.
    pub(crate) const RESULTS: TraceMode = TraceMode {
        all: true,
        results: true,
        intermediates: false,
        labels: true,
        commands: true,
        errors: false,
        failures: false,
        letter: b'R',
        debug: false,
    };
    /// `TRACE I` (`setTraceIntermediates`, `traceIntermediatesFlags`).
    pub(crate) const INTERMEDIATES: TraceMode = TraceMode {
        all: true,
        results: true,
        intermediates: true,
        labels: true,
        commands: true,
        errors: false,
        failures: false,
        letter: b'I',
        debug: false,
    };

    /// `TraceSetting::isTraceOff`, the one setting that refuses the debug
    /// flag. The letter is the flag: `setTraceOff` clears every other one.
    pub(crate) fn is_off(self) -> bool {
        self.letter == b'O'
    }
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

    /// [`TraceMode::commands`]: a command clause echoes. Part of a chunk's
    /// identity because it decides whether the chunk carries the echo op at
    /// all, which is the same reason [`ChunkTrace::CLAUSES`] is.
    const COMMANDS: u8 = 16;

    /// [`TraceMode::debug`]: every traced clause is followed by a pause.
    /// Part of a chunk's identity so that entering debug makes every chunk
    /// stale, which routes the clause echo back through the run-time gate --
    /// the pause rides that decision instead of costing the compiled path a
    /// branch of its own.
    const DEBUG: u8 = 32;

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
                | (u8::from(mode.debug) * ChunkTrace::DEBUG)
                | (u8::from(mode.labels) * ChunkTrace::LABELS)
                | (u8::from(mode.intermediates) * ChunkTrace::INTERMEDIATES)
                | (u8::from(mode.results) * ChunkTrace::RESULTS)
                | (u8::from(mode.commands) * ChunkTrace::COMMANDS),
        )
    }

    /// Whether interactive debug is on, which is what puts a line typed at a
    /// pause -- and so a setting no reading of the source fixes -- in reach.
    #[inline(always)]
    pub(crate) fn debugging(self) -> bool {
        self.0 & ChunkTrace::DEBUG != 0
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
        ChunkTrace(self.0 & (ChunkTrace::CLAUSES | ChunkTrace::LABELS | ChunkTrace::COMMANDS))
    }

    #[inline(always)]
    pub(crate) fn echoes(self, is_label: bool, is_command: bool) -> bool {
        self.0 & ChunkTrace::CLAUSES != 0
            || (self.0 & ChunkTrace::LABELS != 0 && is_label)
            || (self.0 & ChunkTrace::COMMANDS != 0 && is_command)
    }

    /// [`ChunkTrace::of`] applied to [`TraceMode::OFF`]: no bit set.
    pub(crate) const OFF: ChunkTrace = ChunkTrace(0);
}

/// A [`TraceMode`] beside the [`ChunkTrace`] the trace sink obeys under it --
/// its packing, or [`ChunkTrace::OFF`] while a debug pause runs -- so that
/// both are paid where the setting or the pause changes rather than at every
/// clause that reads them.
///
/// **The byte cannot drift from the mode**: [`TraceCache::of`] is the only way
/// to build one, and it derives the byte.
#[derive(Copy, Clone, Debug)]
pub(crate) struct TraceCache {
    mode: TraceMode,
    chunk: ChunkTrace,
}

impl TraceCache {
    #[inline(always)]
    pub(crate) fn of(mode: TraceMode, paused: bool) -> TraceCache {
        TraceCache {
            mode,
            chunk: if paused {
                ChunkTrace::OFF
            } else {
                ChunkTrace::of(mode)
            },
        }
    }

    #[inline(always)]
    pub(crate) fn mode(self) -> TraceMode {
        self.mode
    }

    #[inline(always)]
    pub(crate) fn chunk(self) -> ChunkTrace {
        self.chunk
    }
}

/// What one instruction does to the `TRACE` setting in force -- the
/// optimizing function of `crate::ir::trace_flow`'s forward analysis, one
/// entry per instruction in [`crate::plan::Plan::trace_events`].
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum TraceEvent {
    /// The setting leaves this instruction as it arrived.
    Keeps,
    /// The setting this instruction puts in force, fixed by its own text.
    Sets(ChunkTrace),
    /// The instruction can put a setting in force that no reading of the
    /// source fixes.
    Unknown,
}

/// What a `TRACE` instruction whose setting is a literal puts in force,
/// following `Interp::exec_trace`'s own arms.
pub(crate) fn literal_trace_event(setting: &Trace) -> TraceEvent {
    let mode = match setting {
        Trace::Default => TraceMode::NORMAL,
        Trace::Setting(bytes) => {
            let Ok(request) = parse_trace_request(bytes) else {
                return TraceEvent::Unknown;
            };
            // A setting made only of `?`s toggles the flag on whatever is
            // already in force, so its answer is the incoming pool rather
            // than this instruction's text.
            if request.setting.is_none() {
                return TraceEvent::Unknown;
            }
            applied(TraceMode::OFF, &request)
        }
        // `TRACE n` raises 24.901 outside a pause, and `TRACE VALUE`
        // computes its setting at run time.
        Trace::Skip(_) | Trace::Value(_) => return TraceEvent::Unknown,
    };
    // A line typed at an interactive-debug pause changes the setting from
    // outside the body, which no analysis of the body can see.
    if mode.debug {
        return TraceEvent::Unknown;
    }
    TraceEvent::Sets(ChunkTrace::of(mode))
}

/// Classifies a `TRACE` option string exactly like
/// What one `TRACE` setting asks for: `TraceSetting::parseTraceSetting`
/// (`TraceSetting.cpp:135`-`210`) reads any number of `?`s and then one
/// letter, case-insensitively, ignoring everything after it.
pub(crate) struct TraceRequest {
    /// The letter's own mode, or `None` for a setting that named no letter.
    pub(crate) setting: Option<TraceMode>,
    /// How many `?`s it carried. An even number asks for no change.
    pub(crate) toggles: usize,
}

/// [`TraceRequest`] for one setting, or the byte that is not a letter.
pub(crate) fn parse_trace_request(bytes: &[u8]) -> Result<TraceRequest, u8> {
    let toggles = bytes.iter().filter(|byte| **byte == b'?').count();
    let letter = bytes.iter().find(|byte| **byte != b'?');
    let setting = match letter {
        None => None,
        Some(byte) => Some(mode_of_letter(*byte)?),
    };
    Ok(TraceRequest { setting, toggles })
}

/// The setting in force after `request` is applied to `current`.
///
/// **A setting naming a letter replaces the debug flag; one made only of
/// `?`s toggles it and keeps the letter.** Measured: `trace ?r` traces
/// results *and* pauses, `trace ?` from a pause ends debug and leaves `R` in
/// force, and a bare `trace` ends debug and sets Normal.
///
/// `OFF` is the exception in both halves: `parseTraceSetting`
/// (`TraceSetting.cpp:225`-`238`) skips `setDebug` when the letter is `O`,
/// and `RexxActivation::setTrace` (`RexxActivation.cpp:1010`-`1017`) applies
/// a bare toggle only while the setting in force is not `OFF`. So no
/// spelling of `TRACE` reaches interactive debug from `OFF`, which is what
/// `base/keyword`'s `TRACE::test_trace_?o` asserts.
pub(crate) fn applied(current: TraceMode, request: &TraceRequest) -> TraceMode {
    match (request.setting, request.toggles) {
        (Some(mode), toggles) => TraceMode {
            debug: toggles % 2 == 1 && !mode.is_off(),
            ..mode
        },
        (None, 0) => TraceMode::NORMAL,
        (None, toggles) => TraceMode {
            debug: if toggles % 2 == 1 && !current.is_off() {
                !current.debug
            } else {
                current.debug
            },
            ..current
        },
    }
}

/// `parseTraceSetting`'s answer for a caller with no setting in force to
/// merge against: the letter alone, with `?` recorded as the debug flag. An
/// empty string is `setTraceNormal`'s silent answer, and one made only of
/// `?`s keeps the older reading of `OFF` for the letter.
pub(crate) fn mode_from_setting(bytes: &[u8]) -> Result<TraceMode, u8> {
    let toggles = bytes.iter().filter(|byte| **byte == b'?').count();
    let answer = |mode: TraceMode| {
        Ok(TraceMode {
            debug: toggles % 2 == 1,
            ..mode
        })
    };
    for &byte in bytes {
        if byte == b'?' {
            continue;
        }
        let mode = mode_of_letter(byte)?;
        // `setTraceOff` is unconditional, so `::options trace ?o` is `OFF`
        // and not a paused `OFF`, exactly as the instruction is. The
        // all-`?` fallback below is not covered by this: what it stands in
        // for is `setDebugToggle`, not `setTraceOff`.
        if mode.is_off() {
            return Ok(mode);
        }
        return answer(mode);
    }
    if bytes.is_empty() {
        return answer(TraceMode::NORMAL);
    }
    answer(TraceMode::OFF)
}

/// The mode one `TRACE` letter names, case-insensitively, or the byte itself
/// when it names none.
fn mode_of_letter(byte: u8) -> Result<TraceMode, u8> {
    match byte.to_ascii_uppercase() {
        b'A' => Ok(TraceMode::ALL),
        b'R' => Ok(TraceMode::RESULTS),
        b'I' => Ok(TraceMode::INTERMEDIATES),
        // `L` was in the silent group below until 4b Task 9's review round 1
        // measured what it actually does. It echoes every executed `LABEL`
        // clause -- see `TraceMode::labels`.
        b'L' => Ok(TraceMode::LABELS),
        // `C`/`E`/`F`/`N`/`O`: every letter `check_trace_setting` accepts is
        // recognised here. Each of the four besides `O` decides whether a
        // command clause echoes, so none of them is a setting this crate has
        // nothing to show for.
        b'C' => Ok(TraceMode::COMMANDS),
        b'E' => Ok(TraceMode::ERRORS),
        b'F' => Ok(TraceMode::FAILURES),
        b'N' => Ok(TraceMode::NORMAL),
        b'O' => Ok(TraceMode::OFF),
        _ => Err(byte),
    }
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
/// unreachable from it; `run/settings.rs`'s own `Trace::Setting` call site
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
pub(super) fn make_displayable(out: &mut [u8], line_start: usize) {
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
