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

//! The builtins that read interpreter state rather than transform a value:
//! `ADDRESS`, `ARG`, `CONDITION`, `DIGITS`, `ERRORTEXT`, `FORM`, `FUZZ`,
//! `GC`, `QUEUED`, `SOURCELINE` and `TRACE`.
//!
//! # They read the *running activation*, which is not the calling clause's
//!
//! Every reader below takes its answer from `Interp::activation()`, and a
//! builtin adds no activation of its own (`builtin.rs`'s module doc has
//! the three measured observables for that). So `DIGITS()` inside an
//! internal routine reports the routine's own `NUMERIC DIGITS`, not the
//! caller's, and `CONDITION('S')` reports the trap table of whichever frame
//! asked.
//!
//! # A maximum of zero is not the same as ignoring an extra
//!
//! `ADDRESS`, `DIGITS`, `FORM`, `FUZZ` and `QUEUED` open with `check_args`
//! and a maximum of 0 in `BuiltinFunctions.cpp`, so one argument is 40.4 --
//! measured, `say address(1)` is `Too many arguments in invocation of
//! ADDRESS; maximum expected is 0.` at rc 216. `builtin.rs`'s table is
//! where each row's own `(min, max)` lives; nothing here restates it.
//!
//! # An option letter is one byte, upcased, and the null string is not one
//!
//! `ARG`'s second argument and `CONDITION`'s first are both read as
//! `Utilities::toUpper(option->getChar(0))`, so only the first byte decides
//! and case does not matter -- measured, `arg(1,'exists')` and `arg(1,'e')`
//! are both `1`, and `condition('cond')` behaves as `condition('C')`. An
//! empty option is rejected rather than treated as omitted: `condition('')`
//! and `arg(1,'')` are both 40.904 with `found ""`.
//!
//! **The rejection belongs to the switch, not to reading the argument**, and
//! `ARG` is where that shows: it has checks in front of the switch, and an
//! empty option loses to both of them (`arg(,'')` is 40.5, `arg(0,'')` is
//! 40.14). `arg`'s own doc has the four-row order.
//!
//! # What is not here, and why it is loud rather than approximated
//!
//! Three option letters answer an object this crate's value model has no
//! representation for, and each fails loudly at the call: `ARG(n,'A')` and
//! `CONDITION('A')` answer an `Array`, `CONDITION('O')` a `Directory`.
//! Measured, inside a `SIGNAL ON SYNTAX` handler after `say substr('abc')`:
//! `condition('A')~class` is `The Array class` and `condition('A')~items` is
//! 2, and `condition('O')~items` is 14. The null string would be right for
//! an empty `Array` and wrong for every other shape, which is exactly the
//! silent-wrong-answer trade `Loud::builtin_option_object` exists to refuse.
//!
//! `CONDITION('D')` is answered from `Raised::description`, which carries a
//! `RAISE ... DESCRIPTION` value and does not carry `NOVALUE`'s variable
//! name -- that one pair is loud too, for the same reason and no other.

use rexx_core::ObjRef;

use super::{Args, optional_string, whole_number};
use crate::Interp;
use crate::Loud;
use crate::activation::TrappedCondition;
use crate::error::{Failure, Raised};

/// The environment `ADDRESS()` names before any `ADDRESS` instruction has
/// run, and after a bare `ADDRESS` swaps back to it.
///
/// **A compile-time platform constant, which is why it is spelled here at
/// all.** Both platforms' `SystemInterpreter::getDefaultAddressName()` are
/// the identical one line, `return GlobalNames::INITIALADDRESS;`, and
/// `memory/GlobalNames.h:124` builds that name from `SYSINITIALADDRESS`.
/// **The split is in `platform/<os>/PlatformDefinitions.h`**: `"sh"` at
/// unix line 66, `"CMD"` at windows line 65, both read directly. (There is
/// a second `#define SYSINITIALADDRESS "sh"` in
/// `platform/unix/SystemCommands.cpp:68`; it is local to that translation
/// unit and is **not** the one `GlobalNames.h` expands, so citing it -- as
/// an earlier version of this comment did -- names a definition that could
/// change without changing the answer.) The value is fixed when the
/// interpreter is built, not read from the environment or the process, and
/// measured on this host: `say address()` as a program's only clause prints
/// `sh`.
///
/// So this is the same kind of value as `PARSE SOURCE`'s first two words --
/// `LINUX COMMAND` here, also platform-fixed, also written into the corpus
/// as a measured constant -- and not the kind `PARSE VERSION`'s build date
/// is, which moves under a rebuild and is therefore pinned by a gated
/// differential test instead of by a corpus program. Naming the default
/// environment is not the same job as *issuing a command to* it, which is
/// what D18 defers.
#[cfg(unix)]
const DEFAULT_ENVIRONMENT: &[u8] = b"sh";
#[cfg(not(unix))]
const DEFAULT_ENVIRONMENT: &[u8] = b"CMD";

/// The first byte of an option argument, upcased, or `None` when the
/// argument was not supplied at all.
///
/// `Err` is the 40.904 the null string gets: an option that is present and
/// empty is not an omitted option. Only for a builtin whose option is the
/// *first* thing validated -- `ARG` has two checks in front of its switch
/// and reads its own option inline for that reason.
fn option_letter(
    interp: &mut Interp,
    name: &[u8],
    args: Args<'_>,
    position: usize,
    valid: &str,
) -> Result<Option<u8>, Failure> {
    let Some(text) = optional_string(interp, args, position) else {
        return Ok(None);
    };
    match text.first() {
        Some(byte) => Ok(Some(byte.to_ascii_uppercase())),
        None => Err(Raised::argument_not_in_list(name, position, valid, &text).into()),
    }
}

/// `ADDRESS()`: the environment a command clause would be sent to.
///
/// `context->getAddress()` and nothing else. `None` on the activation is the
/// platform default -- see [`DEFAULT_ENVIRONMENT`] for why that name is
/// written down here rather than deferred with the rest of the command
/// layer.
///
/// Measured, and the swap needs three toggles to be told from a pop:
///
/// ```text
/// say address()   ->  sh
/// address envA
/// address envB
/// say address()   ->  ENVB
/// address ; say address()  ->  ENVA
/// address ; say address()  ->  ENVB
/// address ; say address()  ->  ENVA
/// ```
pub(crate) fn address(
    interp: &mut Interp,
    _name: &[u8],
    _args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let current = interp.activation().address.current.clone();
    let bytes = match &current {
        Some(name) => &name[..],
        None => DEFAULT_ENVIRONMENT,
    };
    Ok(interp.text(bytes))
}

/// `DIGITS()`: the running activation's own `NUMERIC DIGITS`.
pub(crate) fn digits(
    interp: &mut Interp,
    _name: &[u8],
    _args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let digits = interp.activation().settings.digits();
    Ok(interp.text(digits.to_string().as_bytes()))
}

/// `FUZZ()`: the running activation's own `NUMERIC FUZZ`.
pub(crate) fn fuzz(interp: &mut Interp, _name: &[u8], _args: Args<'_>) -> Result<ObjRef, Failure> {
    let fuzz = interp.activation().settings.fuzz();
    Ok(interp.text(fuzz.to_string().as_bytes()))
}

/// `FORM()`: `SCIENTIFIC` or `ENGINEERING`, upper case.
///
/// The oracle returns one of two `GlobalNames` constants rather than
/// formatting anything, so there is no third answer -- measured, `numeric
/// form` with no expression resets to `SCIENTIFIC`.
pub(crate) fn form(interp: &mut Interp, _name: &[u8], _args: Args<'_>) -> Result<ObjRef, Failure> {
    let text: &[u8] = match interp.activation().settings.form() {
        rexx_num::Form::Scientific => b"SCIENTIFIC",
        rexx_num::Form::Engineering => b"ENGINEERING",
    };
    Ok(interp.text(text))
}

/// `QUEUED()`: how many lines the external data queue holds.
///
/// **Single-program only**, which is the `QUEUED` row of
/// `phase-4-exclusions.txt` rather than a limitation of this function: the
/// oracle's queue is served by a live `rxapi` and shared across processes,
/// so a differential run that wrote in one process and counted in another
/// could never agree. `queue.rs`'s module doc has the measurement.
pub(crate) fn queued(
    interp: &mut Interp,
    _name: &[u8],
    _args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let count = interp.queue.len();
    Ok(interp.text(count.to_string().as_bytes()))
}

/// `GC()` / `GC('Force')`: whether a collection was run.
///
/// `0` with no argument and `1` with one, measured -- the oracle only
/// collects unconditionally in a debug build, and this is a release one.
/// The argument is not an option *letter*: `BUILTIN(GC)` tests
/// `forceData[0] != 'f' && != 'F'` and rejects everything else, so
/// `gc('fnord')` is `1` and `gc('x')` is 40.904 naming the four spellings
/// **with their own quotes inside the message** -- `GC argument 1 must be
/// one of "force", "Force", "f", "F"; found "x".`
pub(crate) fn gc(interp: &mut Interp, name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    const VALID: &str = "\"force\", \"Force\", \"f\", \"F\"";
    let Some(text) = optional_string(interp, args, 1) else {
        return Ok(interp.text(b"0"));
    };
    if !matches!(text.first(), Some(b'f' | b'F')) {
        return Err(Raised::argument_not_in_list(name, 1, VALID, &text).into());
    }
    // Really collect, the way `memoryObject.collectAndUninit` does, rather
    // than report `1` for something that did not happen. Safe at exactly the
    // point every allocation is: this crate's own collect-stress mode
    // (`Interp::alloc_with`) runs a collection immediately before every
    // allocation, so a caller's values are already rooted by the time any
    // builtin can be entered.
    //
    // **Through `Interp::collect_now` and not `Heap::collect` directly**,
    // because the root set the collector is handed is not `Interp::roots`
    // alone: an activation's context object is named by nothing in it and is
    // swept in there. A build that reaches past it frees the running
    // activation's own `.CONTEXT` -- measured against one, `say
    // .context~objectName` either side of a forced collection answers
    // `a RexxContext` twice at rc 0 on the oracle and refuses the second
    // send at rc 120 here. Going through the same door every allocation goes
    // through also keeps `collect_at` adjusted, which a bare `Heap::collect`
    // leaves where it was.
    interp.collect_now();
    Ok(interp.text(b"1"))
}

/// `ERRORTEXT(n)`: the major error message for `n`, or the null string when
/// the catalogue has no entry.
///
/// Only the major message, which is why the range is `0..=99` and why no
/// substitution ever happens: those live on the `n.m` sub-entries.
/// Measured, rc 216 either side -- `errortext(-1)` and `errortext(100)` are
/// both 40.903, `ERRORTEXT argument 1 must be in the range 0-99; found
/// "..."` -- and inside the range a number with no catalogue entry is the
/// null string rather than an error: `errortext(0)` prints nothing.
pub(crate) fn errortext(
    interp: &mut Interp,
    name: &[u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let number = whole_number(interp, name, args, 1)?.expect("check_arity admitted argument 1");
    if !(0..=99).contains(&number) {
        return Err(Raised::argument_out_of_range(name, 1, converted(number).as_bytes()).into());
    }
    let text = u16::try_from(number)
        .ok()
        .and_then(|major| rexx_inventory::errors::lookup(major, 0))
        .map(|message| message.text)
        .unwrap_or_default();
    Ok(interp.text(text.as_bytes()))
}

/// `SOURCELINE()`: the program's line count, or `SOURCELINE(n)`: line `n`.
///
/// The *program's*, never the fragment's: `getEffectivePackageObject` reads
/// through an interpret context to its caller, and this crate's `INTERPRET`
/// runs in the enclosing activation, so `activation().program` is already
/// the right one.
///
/// Both failures are the BIF wrapper's 40.x at rc 216, measured against a
/// one-line program: `sourceline(0)` is 40.14 `SOURCELINE argument 1 must be
/// positive; found "0".`, and `sourceline(99)` is 40.34 naming both the
/// request and the count.
pub(crate) fn sourceline(
    interp: &mut Interp,
    name: &[u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let source = &interp.activation().program.source;
    let lines = source.line_count();
    let Some(number) = whole_number(interp, name, args, 1)? else {
        return Ok(interp.text(lines.to_string().as_bytes()));
    };
    if number <= 0 {
        return Err(Raised::argument_not_positive(name, 1, converted(number).as_bytes()).into());
    }
    let requested = usize::try_from(number).unwrap_or(usize::MAX);
    if requested > lines {
        return Err(Raised::sourceline_out_of_range(converted(number).as_bytes(), lines).into());
    }
    let line = interp
        .activation()
        .program
        .source
        .line(requested)
        .expect("a line number within the count")
        .to_vec();
    Ok(interp.text_built(line))
}

/// `TRACE()` / `TRACE(setting)`: the setting in force, and optionally a new
/// one.
///
/// **The old setting is what comes back, not the new one** -- the oracle
/// reads `traceSetting()` before calling `setTrace`, measured: `trace l`
/// then `say trace('O')` prints `L` and leaves the setting at `O`.
///
/// The answer is one byte, `TraceSetting::toString`'s own rendering of the
/// stored flags, and all nine letters are distinguishable --
/// `TraceMode::letter` carries the measurements, including the initial `N`
/// and the `?` prefix this crate does not reproduce.
///
/// A new setting goes through `parseTraceSetting`, so `trace('Z')` is 24.1
/// at rc 232 rather than a 40.x. **A digit string is 24.1 here and 24.901
/// from the instruction**, which is a real difference and not a slip:
/// `RexxInstructionTrace::execute` tests for a whole number before parsing a
/// setting, and `BUILTIN(TRACE)` goes straight to
/// `RexxActivation::setTrace(RexxString *)`, which does not. Measured, rc
/// 232 for both:
///
/// ```text
/// trace value 5     24.901  Numeric TRACE requests are valid only from interactive debugging.
/// say trace('5')    24.1    TRACE request letter must be one of "ACEFILNOR"; found "5".
/// ```
///
/// The rejected byte is the first one, not the whole string: `trace('-3')`
/// reports `found "-"`.
pub(crate) fn trace(interp: &mut Interp, _name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    let previous = interp.trace_mode().letter;
    if let Some(text) = optional_string(interp, args, 1) {
        let mode = crate::trace::mode_from_setting(&text)
            .map_err(crate::trace::raised_invalid_trace_letter)?;
        interp.set_trace_mode(mode);
    }
    Ok(interp.text(&[previous]))
}

/// `ARG()`, `ARG(n)` and `ARG(n, option)`: the current routine's own
/// argument list.
///
/// **The list is the *call's*, not the activation's**, which is why this
/// reads `Interp::call_context` -- at the top level that is the one
/// command-line argument, and inside a routine it is that call's own
/// positions with an interior omission left in place. Measured, both:
/// a program run with one argument reports `arg()` as 1, and `call sub
/// 'p1',,'p3'` reports 3 with position 2 omitted.
///
/// The four options, all measured against that call:
///
/// ```text
/// arg(2)        ''      an omitted position reads as the null string
/// arg(2,'E')    0       ... and 'E'xists says so
/// arg(2,'O')    1       'O'mitted is its complement
/// arg(4,'E')    0       past the end is neither existing ...
/// arg(4,'O')    1       ... nor supplied
/// arg(2,'N')    ''      'N'ormal is `arg(n)` again
/// ```
///
/// **The three checks run in the C++'s own order, and every pair of them
/// can be told apart.** `optional_integer` first, then the no-position test,
/// then `positive_integer`, and only then the option's own letter:
///
/// ```text
/// arg('x','q')   40.12  argument 1 must be a whole number
/// arg(,'')       40.5   argument 1 is required     (an empty option is still an option)
/// arg(0,'')      40.14  argument 1 must be positive
/// arg(1,'')      40.904 argument 2 must be one of AENO; found ""
/// ```
///
/// All four measured, rc 216. The third and fourth are the pair that places
/// the emptiness check: it belongs to the option *switch*, which a bad
/// position never reaches.
pub(crate) fn arg(interp: &mut Interp, name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    const VALID: &str = "AENO";
    let position = whole_number(interp, name, args, 1)?;
    // Read but not validated here: `optional_string` is all the C++ does
    // with it until the switch below.
    let option = optional_string(interp, args, 2);
    let Some(position) = position else {
        // No position at all: any option, empty or not, is 40.5, and
        // otherwise this is the count.
        if option.is_some() {
            return Err(Raised::missing_argument(name, 1).into());
        }
        let count = interp.call_context.arguments.len();
        return Ok(interp.text(count.to_string().as_bytes()));
    };
    if position <= 0 {
        return Err(Raised::argument_not_positive(name, 1, converted(position).as_bytes()).into());
    }
    let index = usize::try_from(position).unwrap_or(usize::MAX);
    let supplied = interp
        .call_context
        .arguments
        .get(index - 1)
        .cloned()
        .flatten();
    let letter = option
        .as_ref()
        .map(|text| text.first().copied().unwrap_or(0).to_ascii_uppercase());
    match letter {
        None | Some(b'N') => Ok(match supplied {
            Some(argument) => argument.value(),
            None => interp.text(b""),
        }),
        Some(b'E') => Ok(interp.text(if supplied.is_some() { b"1" } else { b"0" })),
        Some(b'O') => Ok(interp.text(if supplied.is_some() { b"0" } else { b"1" })),
        Some(b'A') => Err(Loud::builtin_option_object("ARG", b'A', "an Array").into()),
        Some(_) => {
            let found = option.unwrap_or_default();
            Err(Raised::argument_not_in_list(name, 2, VALID, &found).into())
        }
    }
}

/// `CONDITION()` and `CONDITION(option)`: what the handler now running was
/// entered for.
///
/// **The bare form is `CONDITION('I')`, not `'C'`** -- `BUILTIN(CONDITION)`
/// opens `int style = 'I'`, and measured, `say condition()` inside a `SIGNAL
/// ON SYNTAX` handler prints `SIGNAL`.
///
/// The whole option table, measured from *inside* a live handler in each
/// case, because outside one every answer is the null string or `.NIL` and a
/// stub returning `''` is indistinguishable from a correct implementation:
///
/// ```text
///          SYNTAX (1/0)   NOVALUE      USER via SIGNAL   USER via CALL   no condition
///  bare    SIGNAL         SIGNAL       SIGNAL            CALL            ''
///  A       <Array 0>      .NIL         the ADDITIONAL    same            .NIL
///  C       SYNTAX         NOVALUE      USER UC           same            ''
///  D       ''             ZUNSETVAR    the DESCRIPTION   same            ''
///  E       3              ''           ''                same            ''
///  I       SIGNAL         SIGNAL       SIGNAL            CALL            ''
///  O       <Directory 14> <Dir 9>      <Dir 10>          same            .NIL
///  S       OFF            OFF          OFF               DELAY           ''
/// ```
///
/// **`I` and `S` are not one bit, and this is the case that shows it.** `I`
/// is stored on the condition object when the trap fires; `S` is looked up
/// *live* in the running activation's trap table
/// (`RexxActivation::trapState`). Measured, inside a `SIGNAL ON SYNTAX`
/// handler:
///
/// ```text
/// before rearm  I SIGNAL  S OFF
/// signal on syntax name sh1
/// after  rearm  I SIGNAL  S ON
/// signal off syntax
/// after  off    I SIGNAL  S OFF
/// ```
///
/// and the mirror inside a `CALL ON USER UC` handler, where `signal on user
/// uc` leaves `I` at `CALL` while `S` becomes `ON`.
///
/// `R` clears the condition and answers the null string, and it clears
/// **this activation's** copy only -- measured, a subroutine called from the
/// handler that resets sees `''` afterwards while the handler itself still
/// reports `SYNTAX` when it resumes.
pub(crate) fn condition(
    interp: &mut Interp,
    name: &[u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    const VALID: &str = "ACDEIORS";
    let style = option_letter(interp, name, args, 1, VALID)?.unwrap_or(b'I');
    // Resolved before the option is answered, because two of the options
    // have a distinct answer when there is no condition at all and the rest
    // share one.
    let active: Option<TrappedCondition> = interp.activation().condition.clone();
    match (style, &active) {
        (b'A', _) => Err(Loud::builtin_option_object("CONDITION", b'A', "an Array or .NIL").into()),
        (b'O', _) => Err(Loud::builtin_option_object("CONDITION", b'O', "a Directory").into()),
        (b'R', _) => {
            interp.activation_mut().condition = None;
            Ok(interp.text(b""))
        }
        (b'C' | b'D' | b'E' | b'I' | b'S', None) => Ok(interp.text(b"")),
        (b'C', Some(condition)) => {
            let name = condition.name.to_vec();
            Ok(interp.text_built(name))
        }
        (b'D', Some(condition)) => match (&condition.description, &condition.name[..]) {
            (Some(description), _) => {
                let description = description.clone();
                Ok(interp.text_built(description))
            }
            // The one pair this crate cannot answer: the oracle's `NOVALUE`
            // description is the variable's own derived name (measured,
            // `ZUNSETVAR`) and nothing on the read path carries it as far as
            // `novalue_check`. `Raised::description` has the whole note.
            (None, b"NOVALUE") => {
                Err(
                    Loud::builtin_option_object("CONDITION", b'D', "the NOVALUE variable's name")
                        .into(),
                )
            }
            (None, _) => Ok(interp.text(b"")),
        },
        (b'E', Some(condition)) => match condition.code_sub {
            Some(sub) => Ok(interp.text(sub.to_string().as_bytes())),
            None => Ok(interp.text(b"")),
        },
        (b'I', Some(condition)) => {
            let text: &[u8] = if condition.call { b"CALL" } else { b"SIGNAL" };
            Ok(interp.text(text))
        }
        (b'S', Some(condition)) => {
            // Live, not stored: `trapState` reads the trap table by the
            // condition's *own* name, so an `ANY` trap that caught a
            // `SYNTAX` condition reports `OFF` even before it is removed.
            let text: &[u8] = match interp.activation().traps.get(&condition.name) {
                None => b"OFF",
                Some(trap) if trap.delayed => b"DELAY",
                Some(_) => b"ON",
            };
            Ok(interp.text(text))
        }
        _ => {
            let found = optional_string(interp, args, 1).unwrap_or_default();
            Err(Raised::argument_not_in_list(name, 1, VALID, &found).into())
        }
    }
}

/// A converted argument as a 40.x range message spells it in `found "..."`:
/// **the integer the conversion produced, never the value's own rendering.**
///
/// The two are the same for an integer literal and differ for everything
/// else, which is why an alphabet of integer literals cannot see this.
/// Measured on the oracle, rc 216 in every row:
///
/// ```text
/// arg(0.0)                                found "0"          not "0.0"
/// arg('+0')                               found "0"          not "+0"
/// errortext(1e2)                          found "100"        not "1E2"
/// errortext(' 100 ')                      found "100"        not " 100 "
/// sourceline(1e1)                         ("10")             not ("1E1")
/// numeric digits 3; errortext(999999+1)   found "1000000"    not "1.00E+6"
/// ```
///
/// **The last row is the one that needs `NUMERIC DIGITS` crossed with a
/// numeric-looking argument to see at all**, and it is a D15 interaction:
/// the value's rendering is fixed at creation under `DIGITS 3`, while
/// `required_integer` converts under `ARGUMENT_DIGITS` and the message
/// carries what the conversion produced. Holding either axis at its safe
/// value -- an integer literal, or the default `DIGITS` -- hides every row
/// above.
///
/// **This is the opposite choice from [`Raised::argument_not_whole`], and
/// the split is where the conversion succeeded.** 40.12 is raised *because*
/// the conversion failed, so there is no integer and the rendered value is
/// all there is -- measured, `numeric digits 3; z = 1/3; numeric digits 9;
/// errortext(z)` reports `found "0.333"`, the rendering captured at
/// creation, and `errortext(99999999999999999999)` reports all twenty
/// digits. The range checks below it are raised *after* a successful
/// conversion, and they report the result of it. `builtin.rs`'s
/// `length_of`, `position_of` and `count_of` already had it right for the
/// same reason: measured, `numeric digits 3; word('a b', -(999999+1))` is
/// 93.924 `found "-1000000"`.
///
/// [`Raised::argument_not_whole`]: crate::error::Raised::argument_not_whole
fn converted(value: i64) -> String {
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::super::dispatch;
    use crate::error::Failure;
    use crate::{Interp, Invocation, run_program};

    /// Runs `source` as a whole program and hands back the stdout it
    /// produced, having first insisted the run ended cleanly.
    ///
    /// Through `run_program` and not a miniature of it, because every
    /// property below is about state that only exists once activations are
    /// being pushed and popped: which frame `DIGITS()` reads, what a handler
    /// inherits, what dies with a callee.
    fn output(source: &[u8]) -> String {
        let outcome = run_program("/t.rex", source.to_vec(), Invocation::none());
        assert_eq!(
            outcome.exit_code,
            0,
            "stderr: {}",
            String::from_utf8_lossy(&outcome.stderr)
        );
        String::from_utf8(outcome.stdout).expect("ASCII output")
    }

    /// Dispatches `name` with no arguments and renders the answer.
    fn answer(interp: &mut Interp, name: &[u8]) -> Vec<u8> {
        let result = dispatch(interp, name, &[])
            .expect("a builtin name")
            .expect("the call succeeds");
        interp.to_text(result).into_owned()
    }

    /// Runs `source` expecting it to fail, and hands back the exit code and
    /// the whole of stderr.
    ///
    /// The two together rather than either alone: the code says which family
    /// the error came from (216 for the 40.x wrapper, 163 for an operation's
    /// 93.9xx, 120 for a declared gap) and the text says which member.
    fn failure(source: &[u8]) -> (i32, String) {
        let outcome = run_program("/t.rex", source.to_vec(), Invocation::none());
        (
            outcome.exit_code,
            String::from_utf8(outcome.stderr).expect("ASCII stderr"),
        )
    }

    fn raised(name: &[u8], arguments: &[&[u8]]) -> (u16, u16, Vec<Vec<u8>>) {
        let mut interp = Interp::new();
        let values: Vec<Option<rexx_core::ObjRef>> = arguments
            .iter()
            .map(|bytes| Some(interp.text(bytes)))
            .collect();
        let failure = dispatch(&mut interp, name, &values)
            .expect("a builtin name")
            .expect_err("the call fails");
        let Failure::Raised(condition) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        (condition.number, condition.sub, condition.additional)
    }

    /// The default environment is answered before any `ADDRESS` instruction
    /// runs, and the swap is followed three deep -- one toggle cannot tell a
    /// swap from a pop.
    #[test]
    fn address_reports_the_default_and_then_follows_the_swap() {
        assert_eq!(
            output(b"say address()\naddress envA\naddress envB\nsay address()\naddress\nsay address()\naddress\nsay address()\naddress\nsay address()\n"),
            "sh\nENVB\nENVA\nENVB\nENVA\n"
        );
    }

    /// The initial `TRACE` setting is `N`, and each of the nine letters
    /// comes back as itself. `TRACE OFF` is `O` and bare `TRACE` is `N`,
    /// which is the pair the four booleans alone cannot tell apart.
    #[test]
    fn trace_reports_the_setting_it_was_given_and_not_only_its_behaviour() {
        assert_eq!(output(b"say trace()\n"), "N\n");
        assert_eq!(output(b"trace off\nsay trace()\n"), "O\n");
        assert_eq!(output(b"trace off\ntrace\nsay trace()\n"), "N\n");
        assert_eq!(output(b"trace commands\nsay trace()\n"), "C\n");
        assert_eq!(output(b"trace errors\nsay trace()\n"), "E\n");
        assert_eq!(output(b"trace failure\nsay trace()\n"), "F\n");
        assert_eq!(output(b"trace labels\nsay trace('O')\n"), "L\n");
        // The setter answers the *old* setting and installs the new one.
        assert_eq!(
            output(b"trace off\nsay trace('L')\nsay trace('O')\n"),
            "O\nL\n"
        );
    }

    /// `DIGITS`/`FUZZ`/`FORM` read the running activation's own settings,
    /// including after a `NUMERIC` instruction has moved them.
    #[test]
    fn the_numeric_settings_are_read_back_after_they_change() {
        assert_eq!(
            output(b"say digits() fuzz() form()\nnumeric digits 12\nnumeric fuzz 3\nnumeric form engineering\nsay digits() fuzz() form()\n"),
            "9 0 SCIENTIFIC\n12 3 ENGINEERING\n"
        );
    }

    /// A callee's `NUMERIC` does not leak back, so the same builtin answers
    /// differently either side of the call. Without this the three could be
    /// reading one interpreter-wide setting and every single-frame test
    /// would still pass.
    #[test]
    fn the_numeric_settings_are_the_running_activations_own() {
        assert_eq!(
            output(b"numeric digits 12\ncall sub\nsay digits()\nexit\nsub:\nnumeric digits 4\nsay digits()\nreturn\n"),
            "4\n12\n"
        );
    }

    /// `QUEUED` counts what `PUSH`/`QUEUE` wrote and `PULL` has not taken.
    #[test]
    fn queued_counts_the_lines_still_waiting() {
        assert_eq!(
            output(b"say queued()\nqueue 'a'\npush 'b'\nsay queued()\npull v\nsay queued() v\n"),
            "0\n2\n1 B\n"
        );
    }

    /// `GC()` answers 0 and `GC('Force')` answers 1 after really running a
    /// collection -- the count of collections performed is what separates
    /// "returned 1" from "collected".
    #[test]
    fn gc_forces_a_collection_only_when_asked() {
        let mut interp = Interp::new();
        let before = interp.heap.collections_performed();
        assert_eq!(answer(&mut interp, b"GC"), b"0");
        assert_eq!(
            interp.heap.collections_performed(),
            before,
            "GC() must not collect"
        );
        let forced = interp.text(b"Force");
        let result = dispatch(&mut interp, b"GC", &[Some(forced)])
            .expect("a builtin name")
            .expect("the call succeeds");
        assert_eq!(interp.to_text(result).into_owned(), b"1");
        assert!(
            interp.heap.collections_performed() > before,
            "GC('Force') must collect"
        );
        // Only the first byte is examined, and case-insensitively.
        let spelling = interp.text(b"fnord");
        let result = dispatch(&mut interp, b"GC", &[Some(spelling)])
            .expect("a builtin name")
            .expect("the call succeeds");
        assert_eq!(interp.to_text(result).into_owned(), b"1");
    }

    /// `ERRORTEXT` answers the major message inside the range, the null
    /// string for a number the catalogue has no entry for, and 40.903 either
    /// side of it.
    #[test]
    fn errortext_answers_the_major_message_and_refuses_the_range_ends() {
        assert_eq!(
            output(b"say errortext(5)\nsay '['errortext(0)']'\nsay errortext(99)\n"),
            "System resources exhausted.\n[]\nTranslation error.\n"
        );
        assert_eq!(
            raised(b"ERRORTEXT", &[b"-1"]),
            (
                40,
                903,
                vec![b"ERRORTEXT".to_vec(), b"1".to_vec(), b"-1".to_vec()]
            )
        );
        assert_eq!(
            raised(b"ERRORTEXT", &[b"100"]),
            (
                40,
                903,
                vec![b"ERRORTEXT".to_vec(), b"1".to_vec(), b"100".to_vec()]
            )
        );
    }

    /// `SOURCELINE` counts the program's own lines and hands one back
    /// verbatim; the two out-of-range failures are different sub-codes.
    #[test]
    fn sourceline_reads_the_programs_own_text() {
        assert_eq!(
            output(b"say sourceline()\nsay sourceline(1)\n"),
            "2\nsay sourceline()\n"
        );
        // The two failures are different sub-codes with different
        // substitutions, and the whole message is what the oracle is
        // compared against -- both transcripts below were measured on it,
        // against a one-line program, at rc 216.
        assert_eq!(
            failure(b"say sourceline(0)\n"),
            (
                216,
                concat!(
                    "     1 *-* say sourceline(0)\n",
                    "Error 40 running /t.rex line 1:  Incorrect call to routine.\n",
                    "Error 40.14:  SOURCELINE argument 1 must be positive; found \"0\".\n",
                )
                .to_string()
            )
        );
        assert_eq!(
            failure(b"say sourceline(99)\n"),
            (
                216,
                concat!(
                    "     1 *-* say sourceline(99)\n",
                    "Error 40 running /t.rex line 1:  Incorrect call to routine.\n",
                    "Error 40.34:  SOURCELINE argument 1 (\"99\") must be less than or equal to the number of lines in the program (1).\n",
                )
                .to_string()
            )
        );
    }

    /// `ARG`'s four options over a call with an interior omission, plus the
    /// two out-of-range answers a position past the end gives.
    #[test]
    fn arg_reads_the_calls_own_positions_with_omissions_in_place() {
        assert_eq!(
            output(
                b"call sub 'p1',,'p3'\nexit\nsub:\nsay arg()\nsay '['arg(1)']['arg(2)']['arg(3)']['arg(4)']'\nsay arg(1,'E') arg(2,'E') arg(3,'E') arg(4,'E')\nsay arg(1,'O') arg(2,'O') arg(3,'O') arg(4,'O')\nsay '['arg(2,'N')']'\nsay arg(1,'exists')\nreturn\n"
            ),
            "3\n[p1][][p3][]\n1 0 1 0\n0 1 0 1\n[]\n1\n"
        );
    }

    /// An option with no position is the missing-argument error, not the
    /// bad-option one -- the C++ checks for the position first.
    #[test]
    fn an_arg_option_without_a_position_is_a_missing_argument() {
        let mut interp = Interp::new();
        let option = interp.text(b"E");
        let failure = dispatch(&mut interp, b"ARG", &[None, Some(option)])
            .expect("a builtin name")
            .expect_err("argument 1 is required");
        let Failure::Raised(missing) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((missing.number, missing.sub), (40, 5));
        assert_eq!(missing.additional, vec![b"ARG".to_vec(), b"1".to_vec()]);
        assert_eq!(
            raised(b"ARG", &[b"0"]),
            (40, 14, vec![b"ARG".to_vec(), b"1".to_vec(), b"0".to_vec()])
        );
        assert_eq!(
            raised(b"ARG", &[b"1", b"x"]),
            (
                40,
                904,
                vec![
                    b"ARG".to_vec(),
                    b"2".to_vec(),
                    b"AENO".to_vec(),
                    b"x".to_vec()
                ]
            )
        );
    }

    /// The order of `ARG`'s three checks, one probe per adjacent pair.
    ///
    /// Every row is measured on the oracle and every one distinguishes two
    /// orderings a natural implementation could pick. The empty option is
    /// the case that pins the switch's own place in the sequence: it loses
    /// to both checks in front of it and wins against nothing.
    #[test]
    fn args_three_checks_run_in_the_oracles_order() {
        // A bad *type* in position 1 beats everything after it.
        assert_eq!(
            raised(b"ARG", &[b"x", b"q"]),
            (40, 12, vec![b"ARG".to_vec(), b"1".to_vec(), b"x".to_vec()])
        );
        // A bad *range* in position 1 beats the option switch ...
        assert_eq!(
            raised(b"ARG", &[b"0", b""]),
            (40, 14, vec![b"ARG".to_vec(), b"1".to_vec(), b"0".to_vec()])
        );
        // ... and with the position good, that same empty option is the
        // switch's own 40.904. Same two arguments, one changed, opposite
        // answers: this is the pair, not two independent checks.
        assert_eq!(
            raised(b"ARG", &[b"1", b""]),
            (
                40,
                904,
                vec![
                    b"ARG".to_vec(),
                    b"2".to_vec(),
                    b"AENO".to_vec(),
                    b"".to_vec()
                ]
            )
        );
        // With no position at all, an option -- empty or not -- is 40.5.
        let mut interp = Interp::new();
        for spelling in [b"".as_slice(), b"x".as_slice()] {
            let option = interp.text(spelling);
            let failure = dispatch(&mut interp, b"ARG", &[None, Some(option)])
                .expect("a builtin name")
                .expect_err("argument 1 is required");
            let Failure::Raised(missing) = failure else {
                panic!("expected Raised, got {failure:?}");
            };
            assert_eq!((missing.number, missing.sub), (40, 5));
        }
    }

    /// A range check reports the integer the conversion produced, and 40.12
    /// -- raised when that conversion fails -- reports the value's own
    /// rendering. [`converted`] has the oracle transcripts.
    ///
    /// Every argument below is a *text* whose rendering and conversion
    /// differ. An alphabet of integer literals makes the two coincide and
    /// cannot fail this test at all, which is how the wrong choice shipped.
    #[test]
    fn a_range_message_substitutes_the_converted_integer() {
        assert_eq!(
            raised(b"ARG", &[b"0.0"]),
            (40, 14, vec![b"ARG".to_vec(), b"1".to_vec(), b"0".to_vec()])
        );
        assert_eq!(
            raised(b"ARG", &[b"+0"]),
            (40, 14, vec![b"ARG".to_vec(), b"1".to_vec(), b"0".to_vec()])
        );
        assert_eq!(
            raised(b"ARG", &[b"-1.0"]),
            (40, 14, vec![b"ARG".to_vec(), b"1".to_vec(), b"-1".to_vec()])
        );
        for spelling in [b"1e2".as_slice(), b" 100 ".as_slice(), b"100.0".as_slice()] {
            assert_eq!(
                raised(b"ERRORTEXT", &[spelling]),
                (
                    40,
                    903,
                    vec![b"ERRORTEXT".to_vec(), b"1".to_vec(), b"100".to_vec()]
                ),
                "{}",
                String::from_utf8_lossy(spelling)
            );
        }
        // The opposite half, and the reason this is a split rather than one
        // rule: the conversion failed here, so there is no integer to
        // report and all twenty digits come back verbatim.
        assert_eq!(
            raised(b"ERRORTEXT", &[b"99999999999999999999"]),
            (
                40,
                12,
                vec![
                    b"ERRORTEXT".to_vec(),
                    b"1".to_vec(),
                    b"99999999999999999999".to_vec()
                ]
            )
        );
        // `SOURCELINE` needs a running program for its line count, so both
        // of its range messages are asserted whole.
        let (code, stderr) = failure(b"say sourceline(1e1)\n");
        assert_eq!(code, 216, "{stderr}");
        assert!(
            stderr.contains(
                "Error 40.34:  SOURCELINE argument 1 (\"10\") must be less than or \
                             equal to the number of lines in the program (1)."
            ),
            "{stderr}"
        );
        let (code, stderr) = failure(b"say sourceline(0.0)\n");
        assert_eq!(code, 216, "{stderr}");
        assert!(
            stderr.contains("Error 40.14:  SOURCELINE argument 1 must be positive; found \"0\"."),
            "{stderr}"
        );
    }

    /// The D15 crossing: the `NUMERIC DIGITS` a value was *rendered* under
    /// against the integer its conversion produces.
    ///
    /// This is the axis pair the shared block asks for -- numeric-looking
    /// argument crossed with a `DIGITS` setting -- and neither half sees
    /// anything on its own. Under the default `DIGITS`, `999999+1` renders
    /// as `1000000` and the two agree; with `DIGITS 3` and an integer
    /// literal, no conversion is ever wrong.
    ///
    /// The second program of each pair moves `DIGITS` back *after* creating
    /// the value, so a reading that used the current setting rather than the
    /// captured one would also be wrong, and differently.
    #[test]
    fn a_range_message_ignores_the_digits_the_value_was_rendered_under() {
        for (source, expected) in [
            (
                b"numeric digits 3; say errortext(999999+1)\n".as_slice(),
                "Error 40.903:  ERRORTEXT argument 1 must be in the range 0-99; found \"1000000\".",
            ),
            (
                b"numeric digits 3; z = 999999+1; numeric digits 9; say errortext(z)\n".as_slice(),
                "Error 40.903:  ERRORTEXT argument 1 must be in the range 0-99; found \"1000000\".",
            ),
            (
                b"numeric digits 3; say arg(-(999999+1))\n".as_slice(),
                "Error 40.14:  ARG argument 1 must be positive; found \"-1000000\".",
            ),
            (
                b"numeric digits 3; say sourceline(999999+1)\n".as_slice(),
                "Error 40.34:  SOURCELINE argument 1 (\"1000000\") must be less than or equal to \
                 the number of lines in the program (1).",
            ),
            // And the 40.12 half under the same crossing, which does read
            // the captured rendering: `1/3` at `DIGITS 3` is `0.333`, and
            // stays `0.333` after `DIGITS 9`.
            (
                b"numeric digits 3; z = 1/3; numeric digits 9; say errortext(z)\n".as_slice(),
                "Error 40.12:  ERRORTEXT argument 1 must be a whole number; found \"0.333\".",
            ),
        ] {
            let (code, stderr) = failure(source);
            assert_eq!(code, 216, "{stderr}");
            assert!(
                stderr.contains(expected),
                "expected {expected:?} in:\n{stderr}"
            );
        }
    }

    /// `TRACE(setting)` and the `TRACE` instruction disagree about a digit
    /// string, and the disagreement is the oracle's own.
    ///
    /// The instruction tests for a whole number before parsing a setting and
    /// the builtin does not, so the same text is two different errors.
    #[test]
    fn a_digit_string_is_a_bad_letter_to_the_builtin_and_a_skip_count_to_the_instruction() {
        let (code, stderr) = failure(b"say trace('5')\n");
        assert_eq!(code, 232, "{stderr}");
        assert!(
            stderr.contains(
                "Error 24.1:  TRACE request letter must be one of \"ACEFILNOR\"; \
                             found \"5\"."
            ),
            "{stderr}"
        );
        let (code, stderr) = failure(b"trace value 5\n");
        assert_eq!(code, 232, "{stderr}");
        assert!(stderr.contains("Error 24.901:"), "{stderr}");
        // The rejected byte is the first one, not the whole string.
        let (_, stderr) = failure(b"say trace('-3')\n");
        assert!(stderr.contains("found \"-\"."), "{stderr}");
    }

    /// The option letters `CONDITION` answers, read from inside a live
    /// `SIGNAL ON SYNTAX` handler. Outside one they are all empty, so a stub
    /// returning `''` passes any test taken from the top level.
    #[test]
    fn condition_answers_a_syntax_handler_from_inside_it() {
        assert_eq!(
            output(
                b"signal on syntax name h\nsay 1/0\nexit\nh:\nsay '['condition()']['condition('C')']['condition('D')']['condition('E')']['condition('I')']['condition('S')']'\n"
            ),
            "[SIGNAL][SYNTAX][][3][SIGNAL][OFF]\n"
        );
    }

    /// Outside a handler every string option is the null string, and the
    /// bare form is `I` rather than `C`.
    #[test]
    fn condition_is_empty_outside_a_handler_and_defaults_to_the_instruction() {
        assert_eq!(
            output(b"say '['condition()']['condition('C')']['condition('I')']['condition('S')']['condition('E')']['condition('D')']'\n"),
            "[][][][][][]\n"
        );
    }

    /// `I` and `S` are not one bit: re-arming the trap inside its own
    /// handler moves `S` and leaves `I` alone. Turning it off again moves
    /// `S` back, which is what separates "reads the table" from "reports a
    /// second stored constant".
    #[test]
    fn the_instruction_is_stored_and_the_state_is_looked_up_live() {
        assert_eq!(
            output(
                b"signal on syntax name h\nsay 1/0\nexit\nh:\nsay condition('I') condition('S')\nsignal on syntax name h\nsay condition('I') condition('S')\nsignal off syntax\nsay condition('I') condition('S')\n"
            ),
            "SIGNAL OFF\nSIGNAL ON\nSIGNAL OFF\n"
        );
    }

    /// A `CALL ON` handler reports `CALL` and `DELAY`, and the condition
    /// does not survive its return.
    #[test]
    fn a_call_handler_reports_call_and_delay_and_leaves_nothing_behind() {
        assert_eq!(
            output(
                b"call on user uc name uh\ncall r1\nsay '['condition()']['condition('C')']'\nexit\nuh:\nsay condition('I') condition('S') condition('C')\nreturn\nr1:\nraise user uc description 'cd' return 1\n"
            ),
            "CALL DELAY USER UC\n[][]\n"
        );
    }

    /// `R` clears the running activation's copy and nobody else's.
    #[test]
    fn a_reset_dies_with_the_activation_that_asked_for_it() {
        assert_eq!(
            output(
                b"signal on syntax name h\nsay 1/0\nexit\nh:\ncall clearer\nsay '['condition('C')']'\nexit\nclearer:\nsay '['condition('C')']['condition('R')']['condition('C')']'\nreturn\n"
            ),
            "[SYNTAX][][]\n[SYNTAX]\n"
        );
    }

    /// `RAISE ... DESCRIPTION` reaches `CONDITION('D')`, and a raise without
    /// one answers the null string rather than the previous raise's value.
    #[test]
    fn a_description_survives_as_far_as_the_handler() {
        assert_eq!(
            output(
                b"signal on syntax name h\ncall r1\nexit\nh:\nsay '['condition('D')']['condition('E')']'\nexit\nr1:\nraise syntax 40.4 description 'zd' return 1\n"
            ),
            "[zd][4]\n"
        );
    }

    /// The bad-option error, and the null string counted as a bad option
    /// rather than as an omitted one.
    #[test]
    fn condition_rejects_an_unknown_option_and_the_null_string() {
        for (source, found) in [
            (b"say condition('Z')\n".as_slice(), "Z"),
            (b"say condition('')\n".as_slice(), ""),
        ] {
            let (code, stderr) = failure(source);
            assert_eq!(code, 216);
            let echo = String::from_utf8_lossy(&source[..source.len() - 1]).into_owned();
            assert_eq!(
                stderr,
                format!(
                    "     1 *-* {echo}\n\
                     Error 40 running /t.rex line 1:  Incorrect call to routine.\n\
                     Error 40.904:  CONDITION argument 1 must be one of ACDEIORS; found \"{found}\".\n"
                )
            );
        }
    }

    /// The three options whose answer is an object this crate cannot make
    /// fail loudly rather than returning a plausible string.
    #[test]
    fn the_object_valued_options_are_loud() {
        for (source, message) in [
            (
                b"say condition('A')\n".as_slice(),
                "CONDITION option \"A\" answers an Array or .NIL, which is not implemented",
            ),
            (
                b"say condition('O')\n".as_slice(),
                "CONDITION option \"O\" answers a Directory, which is not implemented",
            ),
            (
                b"say arg(1,'A')\n".as_slice(),
                "ARG option \"A\" answers an Array, which is not implemented",
            ),
        ] {
            let (code, stderr) = failure(source);
            assert_eq!(code, crate::NOT_IMPLEMENTED_EXIT, "{stderr}");
            assert!(stderr.contains(message), "{stderr}");
        }
    }

    /// `CONDITION('D')` refuses only the pair it cannot answer. The three
    /// neighbouring cases -- no condition, an interpreter-raised `SYNTAX`,
    /// and a `RAISE` with no `DESCRIPTION` -- all answer the null string and
    /// are asserted elsewhere in this module; this one is `NOVALUE`.
    #[test]
    fn only_novalues_description_is_loud() {
        let (code, stderr) =
            failure(b"signal on novalue name h\nsay zunsetvar\nexit\nh:\nsay condition('D')\n");
        assert_eq!(code, crate::NOT_IMPLEMENTED_EXIT, "{stderr}");
        assert!(
            stderr.contains("CONDITION option \"D\" answers the NOVALUE variable's name"),
            "{stderr}"
        );
        // The adjacent success: the same option, the same handler shape, a
        // condition whose description this crate does carry.
        assert_eq!(
            output(b"signal on syntax name h\nsay 1/0\nexit\nh:\nsay '['condition('D')']'\n"),
            "[]\n"
        );
    }
}
