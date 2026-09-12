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

//! The builtin functions: which names are builtins, how many arguments each
//! takes, and the one dispatch every implementation hangs off.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use rexx_core::{Decoded, ObjRef};

use crate::error::{Failure, Raised};
use crate::value::Rendered;
use crate::{Interp, Loud};

/// Crate-visible because the eight base conversions are methods on `String`
/// as well as builtins -- `AddMethod("B2X", RexxString::b2x, 0)` and the rest
/// of `memory/Setup.cpp:634`-`:641` -- and the cores here are what keep the
/// two forms from coming to disagree about a grouping rule or a sign.
pub(crate) mod convert;
/// Crate-visible because `String~dataType` is the same test `DATATYPE` is --
/// `AddMethod("Datatype", RexxString::dataType, 1)` (`memory/Setup.cpp:584`)
/// -- and the thirteen letters are what the two forms must not come to
/// disagree about. The refusal is not shared: only the letter test is.
pub(crate) mod datatype;
mod datetime;
/// Crate-visible because `String~sign` is the same computation `SIGN` is --
/// `RexxString::sign` is `ArithmeticMethod(Sign(), "SIGN")`
/// (`classes/StringClass.cpp:1084`) -- and [`numeric::sign_of`] is what keeps
/// the builtin and the method from coming to disagree.
pub(crate) mod numeric;

/// The builtins that read or move the interpreter's own platform state.
mod platform;
mod state;
/// Crate-visible because the builtins are not the only place this
/// interpreter searches a haystack for a byte: a `PARSE` template's
/// one-byte string pattern wants [`string::find_byte`] too, and a second
/// copy of a scan whose correctness argument is as delicate as that one's
/// is the wrong way to give it one.
pub(crate) mod string;
pub(crate) mod word;

/// What a builtin's code looks like: the interpreter, the row's own name and
/// the already-evaluated arguments.
type Run = fn(&mut Interp, &'static [u8], Args<'_>) -> Result<ObjRef, Failure>;

/// One builtin this crate runs: its name, its arity, and the code.
struct Builtin {
    /// The name as `BuiltinFunctions.cpp` spells it, which is upper case.
    /// Compared against the call site's own bytes without upcasing either
    /// side -- see [`dispatch`] for the measurement that makes that the rule
    /// rather than a shortcut.
    name: &'static [u8],
    /// The fewest arguments the oracle accepts, which is also the number of
    /// leading positions that may not be omitted (40.3 and 40.5 below).
    min: usize,
    /// The most the oracle accepts, or `None` for a variadic builtin.
    max: Option<usize>,
    /// The code, taking this row's own [`name`] as its second argument.
    run: Run,
}

/// Every builtin this crate runs, with the arity `check_arity` enforces
/// before the code is entered.
const IMPLEMENTED: &[Builtin] = &[
    Builtin {
        name: b"ABBREV",
        min: 2,
        max: Some(3),
        run: string::abbrev,
    },
    Builtin {
        name: b"ABS",
        min: 1,
        max: Some(1),
        run: numeric::abs,
    },
    Builtin {
        // Zero arguments, not one: `check_args` with a maximum of 0, so
        // measured, `say address(1)` is 40.4 naming a maximum of 0 rather
        // than ignoring the extra.
        name: b"ADDRESS",
        min: 0,
        max: Some(0),
        run: state::address,
    },
    Builtin {
        // Measured: `endlocal()` with nothing outstanding answers 0, and
        // `endlocal(1)` is 40.4 naming a maximum of 0.
        name: b"ENDLOCAL",
        min: 0,
        max: Some(0),
        run: platform::endlocal,
    },
    Builtin {
        name: b"QUALIFY",
        min: 1,
        max: Some(1),
        run: platform::qualify,
    },
    Builtin {
        name: b"SETLOCAL",
        min: 0,
        max: Some(0),
        run: platform::setlocal,
    },
    Builtin {
        name: b"USERID",
        min: 0,
        max: Some(0),
        run: platform::userid,
    },
    Builtin {
        name: b"ARG",
        min: 0,
        max: Some(2),
        run: state::arg,
    },
    Builtin {
        name: b"B2X",
        min: 1,
        max: Some(1),
        run: convert::b2x,
    },
    Builtin {
        // A minimum of 1, not 2: the second string is optional and the pad
        // does the work on its own, so measured, `c2x(bitand('ffff'x))` is
        // `FFFF`.
        name: b"BITAND",
        min: 1,
        max: Some(3),
        run: convert::bitand,
    },
    Builtin {
        name: b"BITOR",
        min: 1,
        max: Some(3),
        run: convert::bitor,
    },
    Builtin {
        name: b"BITXOR",
        min: 1,
        max: Some(3),
        run: convert::bitxor,
    },
    Builtin {
        name: b"C2D",
        min: 1,
        max: Some(2),
        run: convert::c2d,
    },
    Builtin {
        name: b"C2X",
        min: 1,
        max: Some(1),
        run: convert::c2x,
    },
    Builtin {
        // Two rows, one implementation: see `Builtin::run` for why the name
        // travels as an argument and what a program can see of the
        // difference.
        name: b"CENTER",
        min: 2,
        max: Some(3),
        run: string::center,
    },
    Builtin {
        name: b"CENTRE",
        min: 2,
        max: Some(3),
        run: string::center,
    },
    Builtin {
        name: b"CHANGESTR",
        min: 3,
        max: Some(4),
        run: string::changestr,
    },
    Builtin {
        name: b"COMPARE",
        min: 2,
        max: Some(3),
        run: string::compare,
    },
    Builtin {
        // A minimum of 0 and a maximum of 1: the bare form is
        // `CONDITION('I')`, measured -- `BUILTIN(CONDITION)` opens
        // `int style = 'I'`.
        name: b"CONDITION",
        min: 0,
        max: Some(1),
        run: state::condition,
    },
    Builtin {
        name: b"COPIES",
        min: 2,
        max: Some(2),
        run: string::copies,
    },
    Builtin {
        name: b"COUNTSTR",
        min: 2,
        max: Some(2),
        run: string::countstr,
    },
    Builtin {
        // A minimum of 1, not 2: the second argument is the type option,
        // and it is optional -- measured, `datatype('12.5')` succeeds with
        // no second argument at all, answering `NUM`/`CHAR` rather than
        // 40.3. `DATATYPE_Min`/`DATATYPE_Max`, `BuiltinFunctions.cpp:780`.
        name: b"DATATYPE",
        min: 1,
        max: Some(2),
        run: datatype::datatype,
    },
    Builtin {
        // A minimum of 0 and a maximum of 5: `date()` and `date('S')` both
        // succeed with no other arguments at all, and `DATE_Min`/
        // `DATE_Max` (`BuiltinFunctions.cpp:1003`-`1004`) name 5 as the top
        // -- `osep`/`isep`, positions 4 and 5, following the input/output
        // style pair. This module's own doc comment on the count check
        // names the conditional-required quirk `(min, max)` cannot express
        // for this exact builtin: supplying `option2` (position 3) without
        // `indate` (position 2) is 40.5, not legal, and `date::date` checks
        // that itself.
        name: b"DATE",
        min: 0,
        max: Some(5),
        run: datetime::date,
    },
    Builtin {
        // A minimum of 1, not 2: `DELSTR`'s start position is optional and
        // defaults to 1, so measured, `say delstr('abcdef')` deletes the
        // whole string rather than raising.
        name: b"DELSTR",
        min: 1,
        max: Some(3),
        run: string::delstr,
    },
    Builtin {
        name: b"D2C",
        min: 1,
        max: Some(2),
        run: convert::d2c,
    },
    Builtin {
        name: b"D2X",
        min: 1,
        max: Some(2),
        run: convert::d2x,
    },
    Builtin {
        name: b"DIGITS",
        min: 0,
        max: Some(0),
        run: state::digits,
    },
    Builtin {
        // A minimum of 2 where `DELSTR`'s is 1: `DELWORD`'s start word is
        // required, so measured, `say delword('a b')` is 40.3 naming a
        // minimum of 2 where `say delstr('abcdef')` succeeds.
        name: b"DELWORD",
        min: 2,
        max: Some(3),
        run: word::delword,
    },
    Builtin {
        name: b"ERRORTEXT",
        min: 1,
        max: Some(1),
        run: state::errortext,
    },
    Builtin {
        name: b"FORM",
        min: 0,
        max: Some(0),
        run: state::form,
    },
    Builtin {
        name: b"FORMAT",
        min: 1,
        max: Some(5),
        run: numeric::format,
    },
    Builtin {
        name: b"FUZZ",
        min: 0,
        max: Some(0),
        run: state::fuzz,
    },
    Builtin {
        // A maximum of 1 where its four neighbours here take none: the
        // argument is the `"force"` spelling, and `BUILTIN(GC)` rejects
        // anything whose first byte is not `f` or `F`.
        name: b"GC",
        min: 0,
        max: Some(1),
        run: state::gc,
    },
    Builtin {
        name: b"INSERT",
        min: 2,
        max: Some(5),
        run: string::insert,
    },
    Builtin {
        name: b"LASTPOS",
        min: 2,
        max: Some(4),
        run: string::lastpos,
    },
    Builtin {
        name: b"LEFT",
        min: 2,
        max: Some(3),
        run: string::left,
    },
    Builtin {
        // `say length()` is 40.3 with a minimum of 1 and `say
        // length('abc','x')` is 40.4 with a maximum of 1, both measured, rc
        // 216.
        name: b"LENGTH",
        min: 1,
        max: Some(1),
        run: string::length,
    },
    Builtin {
        name: b"LOWER",
        min: 1,
        max: Some(3),
        run: string::lower,
    },
    Builtin {
        // `max: None` for the same reason `XRANGE`'s row gives -- `MAX_Max`
        // is `argcount` (`BUILTIN(MAX)`) -- and measured,
        // `max(1,2,3,4,5,6,7,8)` is 8 rather than 40.4.
        name: b"MAX",
        min: 1,
        max: None,
        run: numeric::max,
    },
    Builtin {
        name: b"MIN",
        min: 1,
        max: None,
        run: numeric::min,
    },
    Builtin {
        name: b"OVERLAY",
        min: 2,
        max: Some(5),
        run: string::overlay,
    },
    Builtin {
        name: b"POS",
        min: 2,
        max: Some(4),
        run: string::pos,
    },
    Builtin {
        name: b"QUEUED",
        min: 0,
        max: Some(0),
        run: state::queued,
    },
    Builtin {
        // A minimum of 0: `random()` is a call with no arguments at all and
        // answers a number in 0..999.
        name: b"RANDOM",
        min: 0,
        max: Some(3),
        run: numeric::random,
    },
    Builtin {
        name: b"REVERSE",
        min: 1,
        max: Some(1),
        run: string::reverse,
    },
    Builtin {
        name: b"RIGHT",
        min: 2,
        max: Some(3),
        run: string::right,
    },
    Builtin {
        name: b"SIGN",
        min: 1,
        max: Some(1),
        run: numeric::sign,
    },
    Builtin {
        name: b"SOURCELINE",
        min: 0,
        max: Some(1),
        run: state::sourceline,
    },
    Builtin {
        name: b"SPACE",
        min: 1,
        max: Some(3),
        run: string::space,
    },
    Builtin {
        name: b"STRIP",
        min: 1,
        max: Some(3),
        run: string::strip,
    },
    Builtin {
        name: b"SUBSTR",
        min: 2,
        max: Some(4),
        run: string::substr,
    },
    Builtin {
        name: b"SUBWORD",
        min: 2,
        max: Some(3),
        run: word::subword,
    },
    Builtin {
        name: b"SYMBOL",
        min: 1,
        max: Some(1),
        run: datatype::symbol,
    },
    Builtin {
        // A minimum of 0 and a maximum of 3: `TIME_Min`/`TIME_Max`
        // (`BuiltinFunctions.cpp:1315`-`1316`). `option2` (position 3)
        // requires `intime` (position 2) the same way `DATE`'s own
        // conditional pair does, measured, `time(,,'n')` is 40.5 naming
        // argument 2.
        name: b"TIME",
        min: 0,
        max: Some(3),
        run: datetime::time,
    },
    Builtin {
        // A maximum of 1, and the argument *sets* the mode while the answer
        // is the mode that was in force -- measured, `trace l` then `say
        // trace('O')` prints `L`.
        name: b"TRACE",
        min: 0,
        max: Some(1),
        run: state::trace,
    },
    Builtin {
        // Six, not four: `start` and `range` are ooRexx's own extension to
        // the classic four-argument `TRANSLATE`, and measured, a seventh
        // argument is 40.4 naming a maximum of 6.
        name: b"TRANSLATE",
        min: 1,
        max: Some(6),
        run: string::translate,
    },
    Builtin {
        name: b"TRUNC",
        min: 1,
        max: Some(2),
        run: numeric::trunc,
    },
    Builtin {
        name: b"UPPER",
        min: 1,
        max: Some(3),
        run: string::upper,
    },
    Builtin {
        // `VALUE_Min`/`VALUE_Max`, `BuiltinFunctions.cpp:1812`-`1813`: the
        // new-value and selector positions are both optional, so
        // `value('name')` alone is legal.
        name: b"VALUE",
        min: 1,
        max: Some(3),
        run: datatype::value,
    },
    Builtin {
        name: b"VAR",
        min: 1,
        max: Some(1),
        run: datatype::var,
    },
    Builtin {
        name: b"VERIFY",
        min: 2,
        max: Some(5),
        run: string::verify,
    },
    Builtin {
        name: b"WORD",
        min: 2,
        max: Some(2),
        run: word::word,
    },
    Builtin {
        name: b"WORDINDEX",
        min: 2,
        max: Some(2),
        run: word::word_index,
    },
    Builtin {
        name: b"WORDLENGTH",
        min: 2,
        max: Some(2),
        run: word::word_length,
    },
    Builtin {
        // The search phrase is argument 1 and the string searched is argument
        // 2, which is the reverse of every other builtin here that takes both
        // (`BUILTIN(WORDPOS)`, `expression/BuiltinFunctions.cpp`). Measured,
        // the message numbers them the same way round: `wordpos('a','a b','q')`
        // is `WORDPOS argument 3 must be a whole number`.
        name: b"WORDPOS",
        min: 2,
        max: Some(3),
        run: word::word_pos,
    },
    Builtin {
        name: b"WORDS",
        min: 1,
        max: Some(1),
        run: word::words,
    },
    Builtin {
        name: b"X2B",
        min: 1,
        max: Some(1),
        run: convert::x2b,
    },
    Builtin {
        name: b"X2C",
        min: 1,
        max: Some(1),
        run: convert::x2c,
    },
    Builtin {
        name: b"X2D",
        min: 1,
        max: Some(2),
        run: convert::x2d,
    },
    Builtin {
        // `max: None` because `XRANGE_Max` is `argcount` itself
        // (`BUILTIN(XRANGE)`), so no call is ever too long -- measured,
        // `xrange('a','b','c','d','e','f','g','h')` is eight bytes rather
        // than 40.4. Its minimum is 0, so `xrange()` is a call with no
        // arguments at all and answers all 256 bytes.
        name: b"XRANGE",
        min: 0,
        max: None,
        run: convert::xrange,
    },
];

/// The builtin names Phase 4 dispatches, as a set built once.
fn in_scope() -> &'static HashSet<&'static str> {
    static NAMES: OnceLock<HashSet<&'static str>> = OnceLock::new();
    NAMES.get_or_init(|| rexx_inventory::builtins::in_scope().into_iter().collect())
}

/// Whether `name` is a builtin function name.
pub(crate) fn is_builtin(name: &[u8]) -> bool {
    std::str::from_utf8(name).is_ok_and(|name| in_scope().contains(name))
}

/// The builtin names Phase 4 excludes **outright**, as a set built once --
/// the same derivation [`in_scope`] uses, from the complement
/// `rexx_inventory::builtins::wholly_excluded()`. A *partially* excluded name
/// is not here: its in-scope form still dispatches, and its excluded form is
/// loud from inside the builtin's own code.
fn wholly_excluded() -> &'static HashSet<&'static str> {
    static NAMES: OnceLock<HashSet<&'static str>> = OnceLock::new();
    NAMES.get_or_init(|| {
        rexx_inventory::builtins::wholly_excluded()
            .into_iter()
            .collect()
    })
}

/// Whether `name` is a builtin function name Phase 4 runs nothing for.
pub(crate) fn is_excluded_builtin(name: &[u8]) -> bool {
    std::str::from_utf8(name).is_ok_and(|name| wholly_excluded().contains(name))
}

/// [`resolve`] composed with [`run`], for the unit tests in this module's
/// children.
#[cfg(test)]
pub(crate) fn dispatch(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
) -> Option<Result<ObjRef, Failure>> {
    let target = resolve(name)?;
    Some(run(interp, name, target, args))
}

/// Which builtin a resolved name runs, decided once where the name is
/// resolved rather than found again on every call.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum BuiltinTarget {
    /// The row of [`IMPLEMENTED`] that runs this name.
    Row(u16),
    /// A name this crate declares in scope and runs nothing for.
    Gap,
}

/// Every implemented builtin's row, keyed by the bytes a call site spells.
fn rows() -> &'static HashMap<&'static [u8], u16> {
    static ROWS: OnceLock<HashMap<&'static [u8], u16>> = OnceLock::new();
    ROWS.get_or_init(|| {
        IMPLEMENTED
            .iter()
            .enumerate()
            .map(|(at, builtin)| {
                let at = u16::try_from(at).expect("the builtin table is far under u16");
                (builtin.name, at)
            })
            .collect()
    })
}

/// Resolves `name` to what will run it, or `None` when it is not a builtin at
/// all and resolution should carry on to a `::ROUTINE`.
pub(crate) fn resolve(name: &[u8]) -> Option<BuiltinTarget> {
    // **The row map is asked first, and that ordering is the second half of
    // the saving.** Every row's name is in scope -- asserted by
    // `every_implemented_row_names_an_in_scope_builtin`, which is what makes a hit
    // here conclusive without asking the set as well. So a call to a builtin this
    // crate runs, which is every call any benchmark here makes, hashes the
    // name once rather than twice.
    if let Some(row) = rows().get(name) {
        return Some(BuiltinTarget::Row(*row));
    }
    // A miss is not yet an answer: the name may still be in scope with no row
    // here, and that difference is `Gap`. This is the cold half.
    is_builtin(name).then_some(BuiltinTarget::Gap)
}

/// Runs the builtin `target` names over already-evaluated arguments.
pub(crate) fn run(
    interp: &mut Interp,
    name: &[u8],
    target: BuiltinTarget,
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let BuiltinTarget::Row(row) = target else {
        // A builtin name this crate has no code for. It reads as the same
        // declared gap an unresolved name gets, and deliberately so: a
        // builtin *is* a routine as far as the message is concerned, the
        // owning phase is the same one, and `tests/keyword_assertions.rs`
        // reads the owner out of exactly this shape.
        return Err(Loud::unresolved_call(name).into());
    };
    let builtin = &IMPLEMENTED[row as usize];
    check_arity(builtin, args)?;
    // **The required-string protocol runs here, once, over the argument
    // list** -- after the 40.x count checks and before the builtin body,
    // which is where the measurements in
    // `Interp::required_string_arguments` put it. The row's own name goes
    // with it, because which positions are exempt is a fact about the
    // builtin.
    match interp.required_string_arguments(builtin.name, args)? {
        Some(converted) => (builtin.run)(
            interp,
            builtin.name,
            Args {
                values: &converted,
                objects: args,
            },
        ),
        None => (builtin.run)(
            interp,
            builtin.name,
            Args {
                values: args,
                objects: args,
            },
        ),
    }
}

/// The 1-based argument positions `name` fetches **raw**, which the
/// required-string protocol must leave alone, or an empty slice.
pub(crate) fn raw_argument_positions(name: &'static [u8]) -> &'static [usize] {
    match RAW_ARGUMENT_POSITIONS.iter().find(|(row, _)| *row == name) {
        Some((_, positions)) => positions,
        None => &[],
    }
}

/// The whole of what [`raw_argument_positions`] answers, as a table, so a
/// second builtin joining it is one row rather than a second branch.
static RAW_ARGUMENT_POSITIONS: &[(&[u8], &[usize])] = &[(b"VALUE", &[2])];

/// One builtin call's arguments, in each of the readings a builtin needs of
/// them.
#[derive(Copy, Clone)]
pub(crate) struct Args<'a> {
    /// Each supplied argument through the protocol, converted in position
    /// order before the body ran.
    values: &'a [Option<ObjRef>],
    /// The objects the argument expressions produced.
    objects: &'a [Option<ObjRef>],
}

impl<'a> Args<'a> {
    /// How many argument positions the call wrote, omissions included --
    /// the oracle's `argcount`.
    fn len(self) -> usize {
        self.objects.len()
    }

    /// The object at 1-based `position`, unconverted, which is what a 40.x
    /// message names.
    fn object(self, position: usize) -> Option<ObjRef> {
        self.objects.get(position - 1).copied().flatten()
    }

    /// Every position from 1-based `from` onwards, converted -- what a
    /// variadic builtin walks.
    fn values_from(self, from: usize) -> &'a [Option<ObjRef>] {
        &self.values[from - 1..]
    }
}

/// The 40.x incorrect-call checks every builtin shares, in the order the
/// oracle applies them.
/// ```text
/// say substr('abc')             40.3  Not enough arguments in invocation of SUBSTR; minimum expected is 2.
/// say length('abc','x')         40.4  Too many arguments in invocation of LENGTH; maximum expected is 1.
/// say substr('abc',,2)          40.5  Missing argument in invocation of SUBSTR; argument 2 is required.
/// ```
fn check_arity(builtin: &Builtin, args: &[Option<ObjRef>]) -> Result<(), Failure> {
    if let Some(max) = builtin.max
        && args.len() > max
    {
        return Err(Raised::too_many_arguments(builtin.name, max).into());
    }
    if args.len() < builtin.min {
        return Err(Raised::not_enough_arguments(builtin.name, builtin.min).into());
    }
    match args[..builtin.min].iter().position(Option::is_none) {
        Some(index) => Err(Raised::missing_argument(builtin.name, index + 1).into()),
        None => Ok(()),
    }
}

// ---- reading arguments ----

/// The precision the oracle converts a builtin's numeric arguments under.
const ARGUMENT_DIGITS: usize = 18;

/// The argument at 1-based `position`, or `None` if the call did not supply
/// one there.
fn arg(args: Args<'_>, position: usize) -> Option<ObjRef> {
    args.values.get(position - 1).copied().flatten()
}

/// The rendered bytes of the argument at 1-based `position`, which the
/// caller knows is present.
fn required_string(interp: &mut Interp, args: Args<'_>, position: usize) -> Vec<u8> {
    let value = arg(args, position).expect("check_arity admitted this required argument");
    interp.to_text(value).into_owned()
}

/// The argument at 1-based `position` prepared for a shared borrow, for a
/// builtin that reads more than one string or reads one across a further call
/// on the interpreter.
fn required_render(interp: &mut Interp, args: Args<'_>, position: usize) -> Rendered {
    let value = arg(args, position).expect("check_arity admitted this required argument");
    interp.render(value)
}

/// The rendered bytes of an optional argument.
fn optional_string(interp: &mut Interp, args: Args<'_>, position: usize) -> Option<Vec<u8>> {
    let value = arg(args, position)?;
    Some(interp.to_text(value).into_owned())
}

/// An argument the builtin declared as an integer, converted the way the
/// oracle's `optional_integer`/`required_integer` macros do.
#[inline]
fn whole_number(
    interp: &mut Interp,
    name: &[u8],
    args: Args<'_>,
    position: usize,
) -> Result<Option<i64>, Failure> {
    let Some(value) = arg(args, position) else {
        return Ok(None);
    };
    // **A tagged integer already is the answer**, when it is small enough
    // that `whole_value` would round nothing -- which is what `whole_i64`
    // decides. Going the long way builds a `Number` out of the tag and takes
    // the `i64` straight back out of it. A `None` here means only that the
    // rounding rule has to run, so the general path below still does.
    if let Decoded::SmallInt(small) = value.decode()
        && let Some(whole) = rexx_num::whole_i64(small, ARGUMENT_DIGITS)
    {
        return Ok(Some(whole));
    }
    // `to_number` hands back an owned `Number`, so the borrow of `interp` is
    // over before `to_text` below needs its own.
    if let Ok(number) = interp.to_number(value)
        && let Some(whole) = number.whole_value(ARGUMENT_DIGITS)
    {
        return Ok(Some(whole));
    }
    // **`stringValue()` on the *object*, not on the value the conversion
    // above read**, which is the rule for every object substitution in an
    // error message: `reportException` is handed the object and the catalogue
    // renders it. Measured, three descriptors: `substr('abcdef',(1,2))` is
    // 40.12 `found "an Array"` where `substr('abcdef',(2,))` answers `bcdef`,
    // so the same argument converts through one rendering and is quoted
    // through the other -- and, with the required-string protocol in front,
    // `substr(.A, .B)` where `.B`'s class-side `makeString` answers `'x'` is
    // 40.12 `found "The B class"` and not `found "x"`.
    let named = args.object(position).unwrap_or(value);
    let found = interp.string_value_text(named);
    Err(Raised::argument_not_whole(name, position, &found).into())
}

/// An argument the builtin declared as a pad, which must be exactly one
/// byte.
fn pad_byte(
    interp: &mut Interp,
    name: &[u8],
    args: Args<'_>,
    position: usize,
) -> Result<Option<u8>, Failure> {
    let Some(value) = arg(args, position) else {
        return Ok(None);
    };
    // **The string value, and quoted as the string value too** -- the
    // opposite of [`whole_number`] above, because `padArgument` converts with
    // `stringArgument` and then quotes the *converted string* rather than the
    // object. Measured, `translate('abc','x','y',(1,2))` is 40.23 `found "1`
    // and `2"` on two lines, where `substr('abcdef',(1,2))` quotes
    // `"an Array"`.
    let found = interp.to_text(value).into_owned();
    match found.as_slice() {
        [byte] => Ok(Some(*byte)),
        _ => Err(Raised::argument_not_a_pad(name, position, &found).into()),
    }
}

// ---- range-checking converted arguments ----

/// A converted argument used as a length: zero or positive.
#[inline]
fn length_of(value: i64) -> Result<usize, Failure> {
    usize::try_from(value).map_err(|_| Raised::invalid_length(value.to_string().as_bytes()).into())
}

/// A converted argument used as a position: strictly positive.
#[inline]
fn position_of(value: i64) -> Result<usize, Failure> {
    match usize::try_from(value) {
        Ok(position) if position > 0 => Ok(position),
        _ => Err(Raised::invalid_position(value.to_string().as_bytes()).into()),
    }
}

/// A converted argument used as a repetition or replacement count: zero or
/// positive. `method_position` is the position the oracle's message names,
/// which is the *operation's* own numbering rather than the call's.
#[inline]
fn count_of(value: i64, method_position: usize) -> Result<usize, Failure> {
    usize::try_from(value).map_err(|_| {
        Raised::argument_not_non_negative(method_position, value.to_string().as_bytes()).into()
    })
}

// ---- building results ----

/// A result buffer of exactly `len` bytes' capacity, or the condition the
/// oracle raises when the allocator refuses.
fn fresh_buffer(len: usize) -> Result<Vec<u8>, Failure> {
    let mut out = Vec::new();
    out.try_reserve_exact(len)
        .map_err(|_| Failure::from(Raised::system_resources()))?;
    Ok(out)
}

/// **`try_reserve`, not `try_reserve_exact`, and the difference is the whole
/// point of lending the buffer.** An exact reservation resizes the shared
/// buffer to precisely one call's need, so two callers wanting different
/// lengths reallocate on every alternation and the lending buys nothing:
/// measured on `bench-programs/strings.rex`, whose loop asks `changestr` for
/// 43 bytes and then joins 46, the buffer was reallocated twice per iteration
/// and that was the whole of what the program still allocated. Growing
/// amortised lets the capacity settle at the longest result a program asks
/// for and stay there.
fn buffer(interp: &Interp, len: usize) -> Result<Vec<u8>, Failure> {
    let mut out = interp.take_result_buffer();
    out.try_reserve(len)
        .map_err(|_| Failure::from(Raised::system_resources()))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The three partial rows are builtin names, and the whole exclusions are
    /// not.
    #[test]
    fn the_partial_exclusions_are_builtin_names_and_the_whole_ones_are_not() {
        for name in rexx_inventory::builtins::PARTIALLY_EXCLUDED {
            assert!(
                is_builtin(name.as_bytes()),
                "{name} is excluded only in part, so its in-scope form must dispatch"
            );
        }
        for name in rexx_inventory::builtins::wholly_excluded() {
            assert!(
                !is_builtin(name.as_bytes()),
                "{name} is excluded outright, so nothing here may claim it"
            );
        }
        assert!(is_builtin(b"LENGTH"));
        assert!(!is_builtin(b"length"), "the table is not case-folded");
        assert!(!is_builtin(b"ZORKOLO"));
        assert!(!is_builtin(&[0xff, 0xfe]), "and does not need valid UTF-8");
    }

    /// [`resolve`] partitions the in-scope set exactly, and answers `None`
    /// for everything outside it.
    #[test]
    fn resolve_partitions_the_in_scope_set_into_rows_and_gaps() {
        let mut rows_seen = 0;
        let mut gaps_seen = 0;
        for name in in_scope() {
            let bytes = name.as_bytes();
            match (rows().get(bytes), resolve(bytes)) {
                (Some(row), Some(BuiltinTarget::Row(answered))) => {
                    assert_eq!(*row, answered, "{name} resolved to the wrong row");
                    rows_seen += 1;
                }
                (None, Some(BuiltinTarget::Gap)) => gaps_seen += 1,
                (row, answered) => {
                    panic!("{name}: row map says {row:?}, resolve says {answered:?}")
                }
            }
        }
        assert_eq!(
            rows_seen,
            IMPLEMENTED.len(),
            "every row is reachable by name"
        );

        // **`BuiltinTarget::Gap` is unreachable today, and this is where that
        // is written down.** Phase 4's in-scope set and this crate's table are
        // currently the same 66 names, so no call can produce it -- found by
        // this test failing an earlier assertion that demanded a gap exist.
        assert_eq!(
            gaps_seen,
            0,
            "an in-scope builtin has no row: {} in scope against {} rows",
            in_scope().len(),
            IMPLEMENTED.len()
        );

        assert_eq!(resolve(b"ZORKOLO"), None, "not a builtin at all");
        assert_eq!(resolve(b"length"), None, "the table is not case-folded");
        assert_eq!(
            resolve(&[0xff, 0xfe]),
            None,
            "and does not need valid UTF-8"
        );
    }

    /// Every implemented row names a real in-scope builtin.
    #[test]
    fn every_implemented_row_names_an_in_scope_builtin() {
        for builtin in IMPLEMENTED {
            let name = std::str::from_utf8(builtin.name).expect("a builtin name is ASCII");
            assert!(
                is_builtin(builtin.name),
                "{name} has an implementation but is not an in-scope builtin name"
            );
            assert!(
                builtin.max.is_none_or(|max| max >= builtin.min),
                "{name} has a maximum below its minimum"
            );
        }
    }

    /// `dispatch` answers `None` for a name that is not a builtin, which is
    /// the answer resolution needs to carry on past this step.
    #[test]
    fn dispatch_declines_a_name_that_is_not_a_builtin() {
        let mut interp = Interp::new();
        assert!(dispatch(&mut interp, b"ZORKOLO", &[]).is_none());
        assert!(
            dispatch(&mut interp, b"CHARIN", &[]).is_none(),
            "a whole exclusion is not a builtin name here either"
        );
    }

    fn never_run(_: &mut Interp, _: &'static [u8], _: Args<'_>) -> Result<ObjRef, Failure> {
        unreachable!("check_arity never runs the builtin")
    }

    /// A builtin with a required argument in a middle position, which is the
    /// only shape that can reach 40.5. `LENGTH` takes one argument and a lone
    /// omitted argument is a trailing omission that never arrives, so its own
    /// row cannot produce that sub-code.
    fn substr_arity() -> Builtin {
        let row = IMPLEMENTED
            .iter()
            .find(|builtin| builtin.name == b"SUBSTR")
            .expect("SUBSTR has a row");
        Builtin {
            run: never_run,
            ..*row
        }
    }

    /// The three incorrect-call sub-codes and their substitutions, against
    /// the oracle transcripts in [`check_arity`]'s own doc.
    #[test]
    fn the_arity_checks_answer_the_oracles_own_sub_codes() {
        let value = ObjRef::small_int(1).expect("1 is a small int");

        let failure =
            check_arity(&substr_arity(), &[Some(value)]).expect_err("one argument is too few");
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (40, 3));
        assert_eq!(raised.additional, vec![b"SUBSTR".to_vec(), b"2".to_vec()]);

        let five = [Some(value); 5];
        let failure = check_arity(&substr_arity(), &five).expect_err("five arguments are too many");
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (40, 4));
        assert_eq!(raised.additional, vec![b"SUBSTR".to_vec(), b"4".to_vec()]);

        let failure = check_arity(&substr_arity(), &[Some(value), None, Some(value)])
            .expect_err("argument 2 is required");
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (40, 5));
        assert_eq!(raised.additional, vec![b"SUBSTR".to_vec(), b"2".to_vec()]);

        // The adjacent success: an omission *past* the required positions is
        // not an error at all -- measured, `say substr('abc',2,)` prints
        // `bc`.
        check_arity(&substr_arity(), &[Some(value), Some(value), None])
            .expect("that call is legal");
    }

    /// The maximum is checked before the required positions, which is the one
    /// ordering a program can tell apart.
    #[test]
    fn too_many_arguments_wins_over_a_missing_required_one() {
        let value = ObjRef::small_int(1).expect("1 is a small int");
        let failure = check_arity(
            &substr_arity(),
            &[None, Some(value), Some(value), Some(value), Some(value)],
        )
        .expect_err("five arguments are too many");
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (40, 4));
    }
    /// The oracle's own source is what says which argument positions are
    /// fetched raw, and this re-derives [`RAW_ARGUMENT_POSITIONS`] from it.
    #[test]
    fn the_raw_argument_table_re_derives_from_the_oracles_own_source() {
        /// The blocks that pass positions on without naming a constant for
        /// them, which the derivation below cannot classify. See this test's
        /// own doc for what ruled them.
        const EXTRA_ARGUMENT_BLOCKS: &[&str] = &["MAX", "MIN"];
        /// The accessors that run the protocol on the position they fetch.
        const CONVERTING: &[&str] = &[
            "required_string",
            "optional_string",
            "required_integer",
            "optional_integer",
            "required_big_integer",
            "optional_big_integer",
            "optional_pad",
        ];
        /// The accessors that are `stack->peek` and convert nothing.
        const RAW: &[&str] = &["get_arg", "optional_argument", "arg_exists", "arg_omitted"];

        let path = std::path::PathBuf::from(
            "/home/moritz/dev/repos/ooRexx/interpreter/expression/BuiltinFunctions.cpp",
        );
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "cannot read {} -- the builtin bodies this table derives from are part of \
                 the read-only C++ tree, so a missing one means the tree moved rather than \
                 that the fact is gone: {e}",
                path.display()
            )
        });

        let lines: Vec<&str> = text.lines().collect();
        let mut derived: Vec<(String, Vec<usize>)> = Vec::new();
        let mut blocks = 0usize;
        let mut extra_argument_blocks: Vec<String> = Vec::new();
        let mut at = 0usize;
        while at < lines.len() {
            let Some(name) = lines[at]
                .strip_prefix("BUILTIN(")
                .and_then(|rest| rest.trim_end().strip_suffix(')'))
            else {
                at += 1;
                continue;
            };
            // Brace-matched from the line after the signature, which is the
            // opening `{`, so a nested block cannot end the body early.
            let mut depth = 0isize;
            let mut end = at + 1;
            while end < lines.len() {
                depth += isize::try_from(lines[end].matches('{').count()).expect("a short line");
                depth -= isize::try_from(lines[end].matches('}').count()).expect("a short line");
                if depth == 0 && lines[end].contains('}') {
                    break;
                }
                end += 1;
            }
            let body = lines[at + 1..end.min(lines.len())].join("\n");
            blocks += 1;
            if body.contains("stack->arguments(") {
                extra_argument_blocks.push(name.to_string());
            }

            // `const size_t <NAME>_<arg> = N;`, the block's own position
            // constants. `Min` and `Max` are the arity pair and not positions.
            let mut positions: Vec<(String, usize)> = Vec::new();
            for line in body.lines() {
                let Some(rest) = line.trim_start().strip_prefix("const size_t ") else {
                    continue;
                };
                let Some((declared, value)) = rest.split_once('=') else {
                    continue;
                };
                let Some(argument) = declared.trim().strip_prefix(&format!("{name}_")) else {
                    continue;
                };
                if argument == "Min" || argument == "Max" {
                    continue;
                }
                let Ok(value) = value.trim().trim_end_matches(';').trim().parse::<usize>() else {
                    continue;
                };
                positions.push((argument.to_string(), value));
            }

            let mentions = |family: &[&str], argument: &str| {
                family.iter().any(|macro_name| {
                    body.contains(&format!("{macro_name}({name}, {argument})"))
                        || body.contains(&format!("{macro_name}({name},{argument})"))
                })
            };
            let mut raw_only: Vec<usize> = positions
                .iter()
                .filter(|(argument, _)| mentions(RAW, argument) && !mentions(CONVERTING, argument))
                .map(|(_, value)| *value)
                .collect();
            raw_only.sort_unstable();
            if !raw_only.is_empty() {
                derived.push((name.to_string(), raw_only));
            }
            at = end + 1;
        }

        assert!(
            blocks > 0,
            "no BUILTIN(x) block parsed out of {} -- the file's shape moved",
            path.display()
        );
        extra_argument_blocks.sort();
        assert_eq!(
            extra_argument_blocks,
            EXTRA_ARGUMENT_BLOCKS
                .iter()
                .map(|name| (*name).to_string())
                .collect::<Vec<_>>(),
            "the set of blocks that pass positions on without a constant has moved, and this \
             derivation cannot classify those positions -- run the probes this test's doc names \
             against the new member before widening the list"
        );

        // Restricted to what this crate can reach, for the reason the doc
        // gives.
        let mut derived: Vec<(String, Vec<usize>)> = derived
            .into_iter()
            .filter(|(name, _)| IMPLEMENTED.iter().any(|row| row.name == name.as_bytes()))
            .collect();
        derived.sort();
        let mut committed: Vec<(String, Vec<usize>)> = RAW_ARGUMENT_POSITIONS
            .iter()
            .map(|(name, positions)| {
                (
                    String::from_utf8_lossy(name).into_owned(),
                    positions.to_vec(),
                )
            })
            .collect();
        committed.sort();
        assert_eq!(
            derived,
            committed,
            "RAW_ARGUMENT_POSITIONS disagrees with {} -- a position the oracle fetches raw and \
             this table does not exempt is a silent wrong answer, and one it exempts and the \
             oracle converts is another",
            path.display()
        );
    }

    /// Every row of [`RAW_ARGUMENT_POSITIONS`] names a builtin this crate
    /// implements, at a position that builtin can be given, and the table is
    /// not empty.
    #[test]
    fn every_raw_argument_row_names_a_builtin_and_a_position_it_can_take() {
        assert!(
            !RAW_ARGUMENT_POSITIONS.is_empty(),
            "RAW_ARGUMENT_POSITIONS is empty, which passes every arm below \
             vacuously -- the oracle has at least one raw argument position and \
             `the_raw_argument_table_re_derives_from_the_oracles_own_source` \
             is what says which"
        );
        for (name, positions) in RAW_ARGUMENT_POSITIONS {
            let builtin = IMPLEMENTED
                .iter()
                .find(|row| row.name == *name)
                .unwrap_or_else(|| {
                    panic!(
                        "RAW_ARGUMENT_POSITIONS names {}, which is not an implemented builtin",
                        String::from_utf8_lossy(name)
                    )
                });
            assert!(
                !positions.is_empty(),
                "{} has an empty raw-position list, which is what omitting the row means",
                String::from_utf8_lossy(name)
            );
            for position in *positions {
                assert!(
                    builtin.max.is_none_or(|max| *position <= max),
                    "{} declares a raw position {position} past its own maximum",
                    String::from_utf8_lossy(name)
                );
            }
        }
    }
}
