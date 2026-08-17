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

//! `DATATYPE`, `SYMBOL`, `VALUE` and `VAR`: the four builtins that ask what a
//! *name* is, rather than what a value is.
//!
//! # A Rexx symbol is classified once and read three different ways
//!
//! `SYMBOL`, `VAR` and `VALUE` all start from the identical question --
//! `LanguageParser::scanSymbol` (`parser/Scanner.cpp:1792`), run on the
//! argument's own text -- and then answer it differently. [`classify`] is
//! that one classification, ported byte for byte from `scanSymbol` rather
//! than from the ANSI grammar or the documentation, because the accepted
//! symbol-character set (`!.?_0-9A-Za-z`, `LanguageParser::characterTable`,
//! `parser/Scanner.cpp:60`) and the exponent-tail lookback it needs are both
//! easier to get wrong from prose than from the scanner that actually runs.
//!
//! * [`SymbolKind::Bad`] -- not a symbol at all: empty, over 250 bytes, or
//!   containing a byte outside the accepted set. `SYMBOL` answers `BAD`;
//!   `VAR` and `DATATYPE('S')` answer `0`/false; `VALUE` raises 40.26.
//! * [`SymbolKind::Numeric`] / [`SymbolKind::Literal`] /
//!   [`SymbolKind::LiteralDot`] -- a constant, never a variable, whatever
//!   the pool holds. `SYMBOL` answers `LIT` unconditionally (measured,
//!   `1abc` -- digit-led but not a number -- is `LIT`, not `BAD`); `VAR`
//!   answers `0`; `VALUE` returns the (upcased) text itself and raises
//!   40.26 if a new value was offered, because none of the three is
//!   assignable (`VariableDictionary::getVariableRetriever`,
//!   `execution/VariableDictionary.cpp:738`, builds a constant retriever for
//!   all three and never checks `exists()` for them).
//! * [`SymbolKind::Name`] / [`SymbolKind::Stem`] / [`SymbolKind::CompoundName`]
//!   -- a real variable, stem or compound. `SYMBOL`/`VAR` ask whether it has
//!   ever been given a value; `VALUE` reads (and optionally writes) it.
//!
//! # `VALUE`'s three-argument form is a different builtin wearing the same name
//!
//! A *present* third argument selects the external-pool form
//! (`value(name, , 'ENVIRONMENT')`), which this crate does not implement --
//! see [`Loud::value_selector`]. The two-argument form
//! (`value('myvar','NEWVAL')`) is 4c's, and the discriminator is the third
//! argument's presence, not its content: `value('myvar',,'')` still selects
//! the external-pool form, because an *empty* argument is a present one.
//!
//! # `DATATYPE`'s options are validated against a fixed 13-letter set
//!
//! `"ABILMNOSUVWX9"` is `StringUtil::dataType`'s own default arm
//! (`classes/support/StringUtil.cpp:1132`), not the documentation's
//! description of the function -- the two agree here, but the error message
//! is what a byte-exact comparison is checked against. Only the option's
//! first byte is read, upcased; an *omitted* second argument selects the
//! one-argument `NUM`/`CHAR` form, and a *present but empty* one is 93.915
//! with `found "?"` -- the oracle substitutes that placeholder for the
//! control byte (`0x00`, an empty string's own terminator) the same way it
//! substitutes one for any other control byte in a report line
//! (`error.rs`'s own `displayable`), so this builtin only has to raise with
//! the raw byte and the existing renderer does the rest.

use rexx_core::ObjRef;
use rexx_num::Number;

use super::{arg, optional_string, required_string};
use crate::error::{Failure, Raised};
use crate::{Interp, Loud, Novalue};

/// What `LanguageParser::scanSymbol` classifies a piece of text as.
///
/// Named after the oracle's own `StringSymbolType` (`classes/StringClass.hpp:56`)
/// less `STRING_BAD_VARIABLE`'s and `STRING_NUMERIC`'s C++ spelling, which
/// this crate's own naming convention would otherwise collide with.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SymbolKind {
    /// Empty, over 250 bytes, or containing a byte outside `!.?_0-9A-Za-z`
    /// in a position the exponent-tail exception does not excuse.
    Bad,
    /// Exactly one trailing period and nothing else: `NAME.`.
    Stem,
    /// At least one period, not only a trailing one: `NAME.TAIL`.
    CompoundName,
    /// A digit- or period-led constant that is not a number, or any other
    /// text starting with a non-symbol byte that only an exponent sign
    /// excuses -- `1abc`, `.foo`, `+1e1+`'s rejected cousin `1e1e`.
    Literal,
    /// The single byte `.`, and only that.
    LiteralDot,
    /// A digit- or period-led constant that also parses as a Rexx number:
    /// `1`, `1.5`, `1E+1`.
    Numeric,
    /// No periods, not digit- or period-led: an ordinary variable name.
    Name,
}

/// The longest text `scanSymbol` still calls a symbol
/// (`LanguageParser::MAX_SYMBOL_LENGTH`, `parser/LanguageParser.hpp:458`).
const MAX_SYMBOL_LENGTH: usize = 250;

/// Whether `byte` is one `LanguageParser::isSymbolCharacter` accepts
/// (`parser/Scanner.cpp:60`'s ASCII half): `!.?_0-9A-Za-z`, and nothing at or
/// above `0x80` -- every high byte is a symbol character in no code page
/// this crate reads.
fn is_symbol_byte(byte: u8) -> bool {
    matches!(byte, b'!' | b'.' | b'?' | b'_' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z')
}

/// Ports `LanguageParser::scanSymbol` (`parser/Scanner.cpp:1792`) onto raw
/// bytes rather than an interned, already-validated source token, because
/// this is the one caller that has to classify text a running program built
/// at any byte value.
///
/// Case does not change the answer -- every test below is `isSymbolCharacter`,
/// `isDigit`, or an explicit `to_ascii_uppercase` on the one exponent-sign
/// byte -- so callers needing the *canonical* (upcased) spelling of a real
/// variable upcase before or after calling this as convenient; `datatype`
/// below does not upcase at all, matching `StringUtil::dataType`'s own
/// case-preserving checks, while `symbol`/`var`/`value` upcase first,
/// matching `VariableDictionary::getVariableRetriever`'s `variable->upper()`.
fn classify(text: &[u8]) -> SymbolKind {
    let len = text.len();
    if len == 0 || len > MAX_SYMBOL_LENGTH {
        return SymbolKind::Bad;
    }

    let mut compound = 0usize;
    let mut have_exponent = false;
    let mut scan = 0usize;
    while scan < len && is_symbol_byte(text[scan]) {
        if text[scan] == b'.' {
            compound += 1;
        }
        scan += 1;
    }

    if scan < len {
        // A non-symbol byte, other than a `+`/`-` closing out an exponent,
        // is bad outright -- and a sign is only that exception when there is
        // a preceding byte for it to look back at (`scan == 0` is the read
        // one byte before the string that the C++'s own pointer arithmetic
        // performs at this exact spot -- undefined there, and unconditionally
        // rejected here, which is what every leading-sign probe measures).
        if scan + 1 >= len {
            return SymbolKind::Bad;
        }
        let byte = text[scan];
        if byte != b'-' && byte != b'+' {
            return SymbolKind::Bad;
        }
        if scan == 0 || !text[scan - 1].eq_ignore_ascii_case(&b'E') {
            return SymbolKind::Bad;
        }
        scan += 1;
        while scan < len {
            if !text[scan].is_ascii_digit() {
                return SymbolKind::Bad;
            }
            scan += 1;
        }
        have_exponent = true;
    }

    if text[0] == b'.' || text[0].is_ascii_digit() {
        if compound == 1 && len == 1 {
            return SymbolKind::LiteralDot;
        }
        if compound > 1 {
            return if have_exponent {
                SymbolKind::Bad
            } else {
                SymbolKind::Literal
            };
        }
        // A fresh scan from the start: digit/period run, then an optional
        // exponent. Independent of `scan` above, which by this point has
        // been driven all the way to `len` on every surviving path.
        let mut s = 0usize;
        while s < len && (text[s].is_ascii_digit() || text[s] == b'.') {
            s += 1;
        }
        if s >= len {
            return SymbolKind::Numeric;
        }
        if text[s].eq_ignore_ascii_case(&b'E') {
            s += 1;
            if s < len && matches!(text[s], b'-' | b'+') {
                return SymbolKind::Numeric;
            }
            while s < len {
                if !text[s].is_ascii_digit() {
                    return SymbolKind::Literal;
                }
                s += 1;
            }
            return SymbolKind::Numeric;
        }
        return SymbolKind::Literal;
    }

    if compound == 0 {
        SymbolKind::Name
    } else if compound == 1 && text[len - 1] == b'.' {
        SymbolKind::Stem
    } else {
        SymbolKind::CompoundName
    }
}

/// A Rexx number parsed from bytes, or `None` for anything that is not one
/// (including bytes that are not UTF-8, which a Rexx number's own alphabet
/// never contains).
fn parse_number(text: &[u8]) -> Option<Number> {
    std::str::from_utf8(text).ok().and_then(Number::parse)
}

/// `StringUtil::validateGroupedSetQuiet` (`classes/support/StringUtil.cpp:974`):
/// whether every byte of `text` is in `set`, in groups of `modulus` bytes
/// optionally separated by a blank or tab -- the first group may be
/// shorter than `modulus`, every later one must match the first group's own
/// remainder exactly, and neither a leading nor a trailing blank is
/// admitted. The empty-string case is the caller's, matching the oracle's
/// own `len == 0 || validateGroupedSetQuiet(...)` at each of its two call
/// sites.
fn grouped(text: &[u8], set: fn(u8) -> bool, modulus: usize) -> bool {
    if matches!(text.first(), Some(b' ') | Some(b'\t')) {
        return false;
    }
    let mut space_found = false;
    let mut residue = 0usize;
    let mut count = 0usize;
    let mut last = 0u8;
    for &byte in text {
        last = byte;
        if set(byte) {
            count += 1;
        } else if byte == b' ' || byte == b'\t' {
            if !space_found {
                residue = count % modulus;
                space_found = true;
            } else if residue != count % modulus {
                return false;
            }
        } else {
            return false;
        }
    }
    if matches!(last, b' ' | b'\t') {
        return false;
    }
    !space_found || (count % modulus) == residue
}

/// The 13 letters `DATATYPE`'s option argument accepts, spelled exactly as
/// `StringUtil::dataType`'s own 93.915 message does.
const DATATYPE_OPTIONS: &str = "ABILMNOSUVWX9";

/// `DATATYPE(string)` / `DATATYPE(string, type)`: what kind of data a string
/// holds, or whether it matches one named type.
///
/// The one-argument form is not a third question -- `RexxString::dataType`
/// (`classes/StringClassMisc.cpp:328`) answers it by running the very same
/// check `'N'` runs and choosing between two words from the boolean, so
/// [`SymbolKind`] plays no part here at all except through option `S`/`V`.
pub(crate) fn datatype(
    interp: &mut Interp,
    _name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let text = required_string(interp, args, 1);
    let Some(option) = optional_string(interp, args, 2) else {
        let is_number = parse_number(&text).is_some();
        return Ok(interp.text(if is_number { b"NUM" } else { b"CHAR" }));
    };
    // An empty option's first byte is `0x00`, which matches none of the
    // thirteen letters below and falls straight to the `_` arm -- the same
    // path a bad letter takes, and the byte `error.rs`'s `displayable`
    // turns into `?` when the report line it landed in gets printed.
    let letter = option.first().copied().unwrap_or(0).to_ascii_uppercase();
    let matched = match letter {
        b'A' => !text.is_empty() && text.iter().all(u8::is_ascii_alphanumeric),
        b'B' => text.is_empty() || grouped(&text, |b| matches!(b, b'0' | b'1'), 4),
        b'I' => parse_number(&text)
            .and_then(|n| n.whole_value(rexx_num::ARGUMENT_DIGITS))
            .is_some(),
        b'L' => !text.is_empty() && text.iter().all(u8::is_ascii_lowercase),
        b'M' => !text.is_empty() && text.iter().all(u8::is_ascii_alphabetic),
        b'N' => parse_number(&text).is_some(),
        b'O' => text.len() == 1 && matches!(text[0], b'0' | b'1'),
        b'S' => classify(&text) != SymbolKind::Bad,
        b'U' => !text.is_empty() && text.iter().all(u8::is_ascii_uppercase),
        b'V' => matches!(
            classify(&text),
            SymbolKind::Name | SymbolKind::Stem | SymbolKind::CompoundName
        ),
        // `W`'s precision is the *running* `NUMERIC DIGITS`, where `I`'s and
        // `9`'s are the two fixed ones -- `StringUtil::dataType`'s `'W'` arm
        // rounds through `NumberString::plus(IntegerZero)` under the current
        // settings rather than through `numberValue`'s own fixed-precision
        // conversion (`classes/support/StringUtil.cpp:1076`-`1097`).
        b'W' => {
            let digits = interp.activation().settings.digits();
            parse_number(&text)
                .and_then(|n| n.whole_value(digits as usize))
                .is_some()
        }
        b'X' => text.is_empty() || grouped(&text, |b| b.is_ascii_hexdigit(), 2),
        b'9' => parse_number(&text)
            .and_then(|n| n.whole_value(rexx_num::DEFAULT_DIGITS as usize))
            .is_some(),
        _ => return Err(Raised::invalid_option(DATATYPE_OPTIONS, &[letter]).into()),
    };
    Ok(interp.text(if matched { b"1" } else { b"0" }))
}

/// Whether `name`'s own slot currently holds a value, growing the slot if it
/// has none yet.
///
/// Growing costs nothing observable: a freshly grown slot has no value
/// either way, so calling this on a name nothing has ever touched changes
/// nothing a later read or write would see -- the property `Interp::
/// slot_of`'s own doc states.
///
/// **Matches two different oracle checks with one shape**, because they
/// turn out to be the same test. `RexxActivation::localVariableExists`
/// (`execution/RexxActivation.hpp:515`) asks whether a simple variable's
/// dictionary entry holds a value; `localStemVariableExists` (`:507`) asks
/// only whether a stem's entry exists *at all* -- but in this crate every
/// stem-installing operation (`read_stem`, `stem_set`, `stem_assign`,
/// `stem_drop`) always leaves a `Body::Stem` object behind, dropped stems
/// included, so "has a value" and "was ever installed" coincide for a
/// bare stem's own slot the same way they do not for a compound tail
/// (`compound_tail_exists`, below, needs the tombstone-aware answer
/// `Interp::stem_get` already computes).
fn slot_has_value(interp: &mut Interp, name: &[u8]) -> bool {
    let slot = interp.slot_of(name);
    let frame = interp.activation().frame;
    interp.variable(frame, slot).is_some()
}

/// Splits `tail_source` -- the text after a compound name's first period --
/// on every remaining period and resolves each piece into `stem_get`/
/// `stem_set`'s joined key, substituting a variable's *current* value for
/// any piece that is not itself a literal.
///
/// `VariableDictionary::buildCompoundVariable`'s non-direct form
/// (`execution/VariableDictionary.cpp:923`, `direct == false`): a piece that
/// is empty or starts with an ASCII digit stands for itself, and any other
/// piece is a plain variable name, read through `Interp::read_by_name` --
/// which itself derives the piece's own (upcased) name when the variable is
/// unset, matching a normal compound read. **Not** `Interp::tail_key`, which
/// resolves a *parsed* compound expression's `SymbolId`-tagged pieces; this
/// one starts from a runtime string with no such tagging; the two agree in
/// shape because both port the same oracle rule.
fn resolve_compound_key(interp: &mut Interp, tail_source: &[u8]) -> Vec<u8> {
    let mut key = Vec::new();
    for (index, piece) in tail_source.split(|&byte| byte == b'.').enumerate() {
        if index > 0 {
            key.push(b'.');
        }
        match piece.first() {
            Some(byte) if !byte.is_ascii_digit() => {
                let value = interp.read_by_name(piece);
                key.extend_from_slice(&interp.to_text(value));
            }
            _ => key.extend_from_slice(piece),
        }
    }
    key
}

/// Whether one compound tail -- `name`, already upcased and classified as
/// [`SymbolKind::CompoundName`] -- currently has a value, tombstones and
/// stem-level defaults included.
///
/// `StemClass::compoundVariableExists`'s own test
/// (`classes/StemClass.hpp:139`): the tail's own entry if it has one
/// (present-but-tombstoned counts as "no"), else the stem's own default if
/// it was ever given one, else "no" -- exactly [`Novalue`]'s two producers
/// already distinguish in `Interp::stem_get`, so this reuses that answer
/// rather than re-deriving it.
fn compound_tail_exists(interp: &mut Interp, name: &[u8]) -> bool {
    let dot = name
        .iter()
        .position(|&byte| byte == b'.')
        .expect("SymbolKind::CompoundName has at least one period");
    let stem_name = name[..=dot].to_vec();
    let key = resolve_compound_key(interp, &name[dot + 1..]);
    let (_, novalue) = interp.stem_get(&stem_name, &key);
    novalue == Novalue::Set
}

/// `SYMBOL(name)`: what kind of symbol `name` is -- `BAD`, `LIT` or `VAR`.
///
/// **The three answers do not line up with [`SymbolKind`] one for one.**
/// `BAD` is [`SymbolKind::Bad`] alone; every constant shape (`Numeric`,
/// `Literal`, `LiteralDot`) is `LIT` unconditionally, because
/// `VariableDictionary::getVariableRetriever` builds a *string* retriever
/// for all three and `isString(variable)` short-circuits the `exists()`
/// check `BUILTIN(SYMBOL)` would otherwise run -- measured, `SYMBOL('1abc')`
/// is `LIT` though `1abc` is not a number, and `SYMBOL('.foo')` is `LIT`
/// though `.foo` classifies as a *variable* retriever (`RexxDotVariable`):
/// its `exists()` is the inherited default, unconditionally `false`
/// (`expression/ExpressionBaseVariable.hpp:61`), so a dot name reaches `LIT`
/// by the other route and no dot-variable subsystem is needed to answer it.
/// Only `Name`/`Stem`/`CompoundName` ever ask the variable pool at all.
pub(crate) fn symbol(
    interp: &mut Interp,
    _name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let text = required_string(interp, args, 1);
    let upper = text.to_ascii_uppercase();
    let result: &[u8] = match classify(&upper) {
        SymbolKind::Bad => b"BAD",
        SymbolKind::Numeric | SymbolKind::Literal | SymbolKind::LiteralDot => b"LIT",
        SymbolKind::Name | SymbolKind::Stem => {
            if slot_has_value(interp, &upper) {
                b"VAR"
            } else {
                b"LIT"
            }
        }
        SymbolKind::CompoundName => {
            if compound_tail_exists(interp, &upper) {
                b"VAR"
            } else {
                b"LIT"
            }
        }
    };
    Ok(interp.text(result))
}

/// `VAR(name)`: whether `name` is a variable (simple, stem or compound) that
/// currently has a value.
///
/// The same three-way split [`symbol`] makes, collapsed to a boolean: every
/// constant shape is unconditionally false for the identical reason
/// `symbol`'s own doc gives (`BUILTIN(VAR)`'s `isString` check is the same
/// short-circuit).
pub(crate) fn var(
    interp: &mut Interp,
    _name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let text = required_string(interp, args, 1);
    let upper = text.to_ascii_uppercase();
    let exists = match classify(&upper) {
        SymbolKind::Bad | SymbolKind::Numeric | SymbolKind::Literal | SymbolKind::LiteralDot => {
            false
        }
        SymbolKind::Name | SymbolKind::Stem => slot_has_value(interp, &upper),
        SymbolKind::CompoundName => compound_tail_exists(interp, &upper),
    };
    Ok(interp.text(if exists { b"1" } else { b"0" }))
}

/// A constant name's read answer: a dot-prefixed name resolved the way
/// `RexxDotVariable::getValue` resolves one, and the upcased text itself for
/// every other constant shape.
///
/// **`VALUE`'s one-argument form and the expression form are the same
/// resolution and not the same answer for three names.**
/// `VariableDictionary::getVariableRetriever` builds an ordinary
/// `RexxDotVariable` for a leading-dot symbol, so this reaches the identical
/// order `.NAME` takes -- but the *expression* `.NIL`/`.TRUE`/`.FALSE` is a
/// `SpecialDotVariable` the parser resolved already, which this route never
/// sees. Measured on the oracle with `::class True` in the file: `say .TRUE`
/// prints `1`, `say value('.TRUE')` prints `The TRUE class`.
fn literal_value(interp: &mut Interp, upper: &[u8]) -> Result<ObjRef, Failure> {
    if upper.first() == Some(&b'.') {
        return interp.dot_variable(upper);
    }
    Ok(interp.text(upper))
}

/// `VALUE(name)` / `VALUE(name, newvalue)`: reads (and optionally writes)
/// the variable, stem or compound `name` names.
///
/// **The third argument's presence, not its value, is the whole
/// discriminator** -- see [`Loud::value_selector`]. Checked first and
/// unconditionally, before `name` is even classified: the oracle's own
/// `BUILTIN(VALUE)` reads all three arguments before doing anything else
/// with any of them (`expression/BuiltinFunctions.cpp:1818`-`1822`).
pub(crate) fn value(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    if arg(args, 3).is_some() {
        return Err(Loud::value_selector().into());
    }
    let text = required_string(interp, args, 1);
    let upper = text.to_ascii_uppercase();
    let newvalue = arg(args, 2);
    match classify(&upper) {
        // `found` is the call's own spelling, not the upcased text just
        // used to classify it -- see `Raised::argument_not_a_symbol`'s own
        // doc for the C++ pointer-aliasing mistake that makes those two
        // different call sites in the oracle and one call site here.
        SymbolKind::Bad => Err(Raised::argument_not_a_symbol(name, 1, &text).into()),
        SymbolKind::Numeric | SymbolKind::Literal | SymbolKind::LiteralDot => {
            if newvalue.is_some() {
                return Err(Raised::argument_not_a_symbol(name, 1, &text).into());
            }
            literal_value(interp, &upper)
        }
        SymbolKind::Name => {
            let old = interp.read_by_name(&upper);
            if let Some(new) = newvalue {
                let slot = interp.slot_of(&upper);
                let frame = interp.activation().frame;
                interp.set_variable(frame, slot, new);
            }
            Ok(old)
        }
        SymbolKind::Stem => {
            let old = interp.read_stem(&upper);
            if let Some(new) = newvalue {
                interp.stem_assign(&upper, new);
            }
            Ok(old)
        }
        SymbolKind::CompoundName => {
            let dot = upper
                .iter()
                .position(|&byte| byte == b'.')
                .expect("SymbolKind::CompoundName has at least one period");
            let stem_name = upper[..=dot].to_vec();
            let key = resolve_compound_key(interp, &upper[dot + 1..]);
            let (old, _) = interp.stem_get(&stem_name, &key);
            // Rooted before `stem_set` runs, not after: when the stem has
            // never been touched, `stem_get` derives `old` as a fresh,
            // slot-less allocation (`stem.rs`'s "no object at all" branch),
            // and `stem_set` then allocates the stem's *first* object on
            // the identical branch -- so without this, `old` is reachable
            // from nowhere the collector walks for the one allocation that
            // matters. The instruction loop's own temps frame (`run.rs`'s
            // `step`) closes this out at the end of the clause.
            interp.roots.push_temp(old);
            if let Some(new) = newvalue {
                interp.stem_set(&stem_name, &key, new);
            }
            Ok(old)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Failure;
    use crate::{Interp, Invocation, run_program};

    /// Runs `source` as a whole program and hands back the stdout it
    /// produced, having first insisted the run ended cleanly. Through
    /// `run_program` rather than a miniature of it, for the reason
    /// `state.rs`'s own `output` gives: every property below is about
    /// activation-scoped variable state.
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

    fn failure(source: &[u8]) -> (i32, String) {
        let outcome = run_program("/t.rex", source.to_vec(), Invocation::none());
        (
            outcome.exit_code,
            String::from_utf8(outcome.stderr).expect("ASCII stderr"),
        )
    }

    use super::super::dispatch;

    fn raised(name: &[u8], arguments: &[&[u8]]) -> (u16, u16, Vec<Vec<u8>>) {
        let mut interp = Interp::new();
        let values: Vec<Option<ObjRef>> = arguments
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

    /// [`classify`] against `SYMBOL.testGroup`'s 26 distinct inputs across
    /// `test001`-`test028` (`test007`/`test015` and `test019`/`test026`
    /// each repeat the other's input), which is the exponent-tail
    /// grammar's own adversarial corpus -- every row was measured on the
    /// oracle by the ooTest suite's authors, not by this task.
    #[test]
    fn classify_matches_every_symbol_testgroup_numbered_case() {
        let cases: &[(&[u8], SymbolKind)] = &[
            (b"1 ", SymbolKind::Bad),
            (b"1 1", SymbolKind::Bad),
            (b"1E1", SymbolKind::Numeric),
            (b"1.E1", SymbolKind::Numeric),
            (b"1.1E1", SymbolKind::Numeric),
            (b"1.1E1.", SymbolKind::Literal),
            (b"1E1.", SymbolKind::Literal),
            (b"1.1E1.1", SymbolKind::Literal),
            (b"+1E1", SymbolKind::Bad),
            (b"1E+1", SymbolKind::Numeric),
            (b"1E+1+", SymbolKind::Bad),
            (b"1E++1", SymbolKind::Bad),
            (b"1E+-1", SymbolKind::Bad),
            (b"1E-+1", SymbolKind::Bad),
            (b"1E1E", SymbolKind::Literal),
            (b"1E+1.", SymbolKind::Bad),
            (b"1E+1E1", SymbolKind::Bad),
            (b"1E1 ", SymbolKind::Bad),
            (b"1+E1", SymbolKind::Bad),
            (b"-1E1", SymbolKind::Bad),
            (b"1E-1", SymbolKind::Numeric),
            (b"1-E1", SymbolKind::Bad),
            (b"1 E1", SymbolKind::Bad),
            (b"1E 1", SymbolKind::Bad),
            (b"1E1E+1", SymbolKind::Literal),
            (b"1Garbage+1", SymbolKind::Literal),
        ];
        for (text, expected) in cases {
            assert_eq!(
                classify(text),
                *expected,
                "{}",
                String::from_utf8_lossy(text)
            );
        }
        // Every `Numeric`/`Literal` row above collapses to `LIT` through
        // `SYMBOL` regardless -- see `symbol`'s own doc for why -- so a
        // second table, keyed on the oracle's own answers, pins the
        // builtin's behaviour rather than only the classifier's.
        let symbol_cases: &[(&[u8], &[u8])] = &[
            (b"1 ", b"BAD"),
            (b"1 1", b"BAD"),
            (b"1E1", b"LIT"),
            (b"1.E1", b"LIT"),
            (b"1.1E1", b"LIT"),
            (b"1.1E1.", b"LIT"),
            (b"1E1.", b"LIT"),
            (b"1.1E1.1", b"LIT"),
            (b"+1E1", b"BAD"),
            (b"1E+1", b"LIT"),
            (b"1E+1+", b"BAD"),
            (b"1E++1", b"BAD"),
            (b"1E+-1", b"BAD"),
            (b"1E-+1", b"BAD"),
            (b"1E1E", b"LIT"),
            (b"1E+1.", b"BAD"),
            (b"1E+1E1", b"BAD"),
            (b"1E1 ", b"BAD"),
            (b"1+E1", b"BAD"),
            (b"-1E1", b"BAD"),
            (b"1E-1", b"LIT"),
            (b"1-E1", b"BAD"),
            (b"1 E1", b"BAD"),
            (b"1E 1", b"BAD"),
            (b"1E1E+1", b"LIT"),
            (b"1Garbage+1", b"LIT"),
        ];
        let mut interp = Interp::new();
        for (text, expected) in symbol_cases {
            let value = interp.text(text);
            let result = dispatch(&mut interp, b"SYMBOL", &[Some(value)])
                .expect("a builtin name")
                .expect("the call succeeds");
            assert_eq!(
                interp.to_text(result).into_owned(),
                expected.to_vec(),
                "{}",
                String::from_utf8_lossy(text)
            );
        }
    }

    /// `SYMBOL.testGroup`'s `test_SYMBOL` method, all nine of its
    /// assertions (five documented examples, then its own "new tests"
    /// comment introduces four more), which exercise the variable pool
    /// rather than the classifier alone: a plain assigned name, a compound
    /// whose tail resolves through a variable, a bare digit constant, an
    /// invalid symbol, and four dot/empty/blank edge cases.
    #[test]
    fn symbol_reads_the_variable_pool_for_names_and_compounds() {
        assert_eq!(
            output(
                b"drop a.3\nj=3\nsay symbol('J') symbol(J) symbol('a.j') symbol(2) symbol('*')\nsay symbol('.') symbol('.a') symbol('') symbol('  ')\n"
            ),
            "VAR LIT LIT LIT BAD\nLIT LIT BAD BAD\n"
        );
    }

    /// The `VAR.testGroup` case, unabridged: an assigned simple variable, a
    /// numeric constant, a compound whose tail resolves through a
    /// variable, an invalid symbol, and an environment symbol (which is
    /// never a variable regardless of what it resolves to).
    #[test]
    fn var_answers_the_documented_example_line_for_line() {
        assert_eq!(
            output(
                b"drop a.3\nj=3\nsay var('J') var(J) var('a.j') var(2) var('*') var('.LOCAL')\n"
            ),
            "1 0 0 0 0 0\n"
        );
    }

    /// A stem with an explicit default makes every tail report as existing,
    /// including tails never individually assigned -- `s.9` itself
    /// classifies as `SymbolKind::CompoundName` (one period, not trailing),
    /// and the D15a rule this pins is that its existence check falls back
    /// to `StemClass::realCompoundVariableValue`'s stem-level default
    /// rather than to `tails.is_empty()`.
    #[test]
    fn a_stems_default_value_makes_every_tail_report_var() {
        assert_eq!(
            output(b"s.='dflt'\nsay symbol('s.9') var('s.9')\n"),
            "VAR 1\n"
        );
        // The adjacent failure: with no default at all, an untouched tail
        // reports LIT/0, which is what tells "the default answered" apart
        // from "every tail is VAR regardless".
        assert_eq!(output(b"say symbol('t.9') var('t.9')\n"), "LIT 0\n");
    }

    /// `DATATYPE`'s one-argument form and its documented option letters,
    /// against the transcripts `DATATYPE.testGroup` and Step 0(b) of the
    /// brief both give.
    #[test]
    fn datatype_answers_the_one_argument_form_and_every_option_letter() {
        assert_eq!(
            output(b"say datatype('12.5') datatype('abc')\n"),
            "NUM CHAR\n"
        );
        for (probe, expected) in [
            (&b"datatype(123,'n')"[..], "1"),
            (b"datatype(123,'NUM')", "1"),
            (b"datatype(123,'NX')", "1"),
            (b"datatype('','B')", "1"),
            (b"datatype('abc','L')", "1"),
            (b"datatype('ABC','U')", "1"),
            (b"datatype('AbC','M')", "1"),
            (b"datatype('abc','A')", "1"),
            (b"datatype('ab 3','A')", "0"),
            (b"datatype('0','O')", "1"),
            (b"datatype('2','O')", "0"),
            (b"datatype('a.b','V')", "1"),
            (b"datatype('1abc','V')", "0"),
            (b"datatype('a.b','S')", "1"),
            (b"datatype('*','S')", "0"),
            (b"datatype('FF','X')", "1"),
            (b"datatype('FG','X')", "0"),
            (b"datatype('1010','B')", "1"),
            (b"datatype('1012','B')", "0"),
            (b"datatype(99999999999999999999,'I')", "0"),
            (b"datatype(123456789012345678,'I')", "1"),
            (b"datatype(1000000000,'9')", "0"),
            (b"datatype(999999999,'9')", "1"),
        ] {
            assert_eq!(
                output(&[b"say ".as_slice(), probe, b"\n".as_slice()].concat()),
                format!("{expected}\n"),
                "{}",
                String::from_utf8_lossy(probe)
            );
        }
    }

    /// The trap the brief's Step 0(b) names: an empty string is `CHAR` in
    /// the one-argument form, but `datatype('','B')` is `1` -- the empty
    /// string is a valid binary string, so the same argument answers
    /// oppositely depending only on whether a type is asked for.
    #[test]
    fn the_empty_string_is_char_but_a_valid_binary_string() {
        assert_eq!(
            output(b"say datatype('') datatype('','B') datatype('','X')\n"),
            "CHAR 1 1\n"
        );
    }

    /// `DATATYPE`'s option is read as one byte, and the omitted argument is
    /// not an empty one: an omitted second argument selects the one-argument
    /// form, while an empty one is 93.915 with `found "?"` -- the NUL byte
    /// an empty option's `getChar(0)` reads, turned into `?` by the report
    /// line's own control-byte substitution (`error.rs`'s `displayable`).
    #[test]
    fn datatype_tells_an_omitted_option_from_an_empty_one() {
        // The omitted form: no second argument at all is the one-argument
        // answer, not an error.
        assert_eq!(output(b"say datatype('')\n"), "CHAR\n");
        let (code, stderr) = failure(b"say datatype('','')\n");
        assert_eq!(code, 163, "{stderr}");
        assert!(
            stderr.contains(
                "Error 93.915:  Method option must be one of \"ABILMNOSUVWX9\"; found \"?\"."
            ),
            "{stderr}"
        );
        let (code, stderr) = failure(b"say datatype(1,'Z')\n");
        assert_eq!(code, 163, "{stderr}");
        assert!(
            stderr.contains(
                "Error 93.915:  Method option must be one of \"ABILMNOSUVWX9\"; found \"Z\"."
            ),
            "{stderr}"
        );
    }

    /// Swept over every byte from `0x80` to `0xFF`: none of `A`, `U`, `L`,
    /// `M` ever answers `1` for one of them, matching the brief's own
    /// Step 1 measurement and confirming `u8::is_ascii_*` needs no help to
    /// reproduce it. `W` is the same claim, and is checked separately
    /// (below) through a live activation, since `'W'` reads the running
    /// `NUMERIC DIGITS` and a bare `Interp::new()` has no activation to
    /// read it from.
    #[test]
    fn no_byte_above_0x7f_is_alphanumeric_upper_lower_or_mixed() {
        let mut interp = Interp::new();
        for byte in 0x80u16..=0xff {
            let byte = byte as u8;
            let text = interp.text(&[byte]);
            for option in [b"A", b"U", b"L", b"M"] {
                let opt = interp.text(option);
                let result = dispatch(&mut interp, b"DATATYPE", &[Some(text), Some(opt)])
                    .expect("a builtin name")
                    .expect("the call succeeds");
                assert_eq!(
                    interp.to_text(result).into_owned(),
                    b"0",
                    "byte {byte:#x} option {}",
                    String::from_utf8_lossy(option)
                );
            }
        }
    }

    /// `W`'s half of the same sweep, through a real program (so a live
    /// activation exists to read `NUMERIC DIGITS` from): every byte at or
    /// above `0x80` is a one-byte string that never parses as a number at
    /// all, so `DATATYPE(byte,'W')` is `0` at both ends of the range.
    #[test]
    fn no_byte_above_0x7f_is_a_whole_number() {
        assert_eq!(
            output(b"say datatype('80'x,'W') datatype('ff'x,'W')\n"),
            "0 0\n"
        );
    }

    /// D15 through `DATATYPE('W')`: a value's own `DIGITS`/`FORM` pair is
    /// fixed at creation, and `'W'` reads the *running* setting, not the
    /// value's captured one -- so moving `NUMERIC DIGITS` back down after
    /// creating a ten-digit value changes `DATATYPE`'s answer even though
    /// the value's own rendering never moves.
    #[test]
    fn datatype_w_reads_the_running_digits_not_the_values_captured_one() {
        assert_eq!(
            output(b"numeric digits 3\nzz = 1/3\nsay zz\nnumeric digits 9\nsay datatype(zz,'W')\n"),
            "0.333\n0\n"
        );
        assert_eq!(
            output(b"z = 999999999\nsay datatype(z,'W')\nnumeric digits 3\nsay datatype(z,'W')\n"),
            "1\n0\n"
        );
    }

    /// `VALUE`'s two-argument form: the write returns the *old* value, and a
    /// caseless lookup finds the same slot a differently-cased name wrote.
    #[test]
    fn value_writes_the_pool_and_returns_the_old_value() {
        assert_eq!(
            output(
                b"myvar = 'ORIGINAL'\nsay value('myvar')\nsay value('myvar','NEWVAL')\nsay value('MYVAR')\n"
            ),
            "ORIGINAL\nORIGINAL\nNEWVAL\n"
        );
    }

    /// An unset name's read is its own upcased spelling, with no error --
    /// measured, `value('nosuchvar')` and not `value('NOSUCHVAR')`, so the
    /// caseless read is exercised as well as the answer.
    #[test]
    fn value_of_an_unset_name_is_its_own_upcased_spelling() {
        assert_eq!(output(b"say value('nosuchvar')\n"), "NOSUCHVAR\n");
    }

    /// `VALUE`'s compound form substitutes a tail piece's *current* value,
    /// the same way reading `a.j` in source does -- not the direct, literal
    /// tail `DROP (v)`'s indirect form uses.
    #[test]
    fn value_substitutes_a_compound_tail_through_the_variable_pool() {
        assert_eq!(
            output(b"drop a.3\nj = 3\nsay value('a.j')\na.3 = 'hit'\nsay value('a.j')\n"),
            "A.3\nhit\n"
        );
    }

    /// `VALUE`'s phase split (brief Step 0(a)): the discriminator is the
    /// third argument's *presence*, and an empty third argument is still
    /// present. On the oracle, `value('myvar',,'')` answers `.MYVAR` (a
    /// pool lookup, not the local `NEWVAL` a crate ignoring the third
    /// argument would answer); this crate declares the whole
    /// external-selector path a gap rather than risk answering the
    /// *local* value silently wrong, so both an empty and a named selector
    /// take the identical loud path -- the pair below is what tells "the
    /// third argument's presence is checked" apart from "the third
    /// argument's emptiness is checked and never reached here".
    #[test]
    fn values_third_argument_is_decided_by_presence_not_content() {
        for source in [
            &b"say value('myvar',,'')\n"[..],
            b"say value('zz',,'NOSUCHPOOL')\n",
        ] {
            let (code, stderr) = failure(source);
            assert_eq!(code, crate::NOT_IMPLEMENTED_EXIT, "{stderr}");
            assert!(
                stderr.contains("VALUE's external-selector form is not implemented"),
                "{stderr}"
            );
        }
        // The adjacent success: an *omitted* third argument -- even with an
        // interior omission at the second position -- stays on 4c's own
        // local-pool path and never reaches the loud one.
        assert_eq!(
            output(b"myvar = 'ORIGINAL'\nsay value('myvar',)\n"),
            "ORIGINAL\n"
        );
    }

    /// A bad symbol is always 40.26, read or write; a valid-but-unassignable
    /// one (a number) is 40.26 only when a new value is offered.
    #[test]
    fn value_raises_40_26_for_a_bad_symbol_and_for_writing_a_constant() {
        assert_eq!(
            raised(b"VALUE", &[b"*"]),
            (
                40,
                26,
                vec![b"VALUE".to_vec(), b"1".to_vec(), b"*".to_vec()]
            )
        );
        assert_eq!(
            raised(b"VALUE", &[b"5", b"x"]),
            (
                40,
                26,
                vec![b"VALUE".to_vec(), b"1".to_vec(), b"5".to_vec()]
            )
        );
        // The adjacent success: reading that same constant, with no new
        // value offered, succeeds and answers the constant itself.
        let mut interp = Interp::new();
        let five = interp.text(b"5");
        let result = dispatch(&mut interp, b"VALUE", &[Some(five)])
            .expect("a builtin name")
            .expect("the call succeeds");
        assert_eq!(interp.to_text(result).into_owned(), b"5");
    }

    /// The 40.26 insert is the call's own spelling, never upcased -- every
    /// witness above (`*`, `5`) has no case to get wrong, so neither can
    /// tell an upcased insert from a verbatim one. These four do have a
    /// case, or a byte no lossy conversion may touch, and each is measured
    /// against the oracle directly:
    ///
    /// ```text
    /// value('ab*')             found "ab*"              (BAD, via the first call site)
    /// value('1e1','x')         found "1e1"               (a constant given a new value, the second)
    /// value('abc.def*','x')    found "abc.def*"
    /// value('a'||'80'x)        found "a" + the raw byte 0x80
    /// ```
    ///
    /// `VALUE.testGroup:88`'s `test008` (`value(.zl)` where `.zl` holds
    /// `'lowercase garbage'`) reaches this same raiser; the fifth row below
    /// is that transcript with the `.zl` indirection resolved, since this
    /// crate has no `.local` to build the indirection through.
    #[test]
    fn value_40_26_substitutes_the_arguments_own_bytes_case_and_all() {
        assert_eq!(
            raised(b"VALUE", &[b"ab*"]),
            (
                40,
                26,
                vec![b"VALUE".to_vec(), b"1".to_vec(), b"ab*".to_vec()]
            )
        );
        assert_eq!(
            raised(b"VALUE", &[b"1e1", b"x"]),
            (
                40,
                26,
                vec![b"VALUE".to_vec(), b"1".to_vec(), b"1e1".to_vec()]
            )
        );
        assert_eq!(
            raised(b"VALUE", &[b"abc.def*", b"x"]),
            (
                40,
                26,
                vec![b"VALUE".to_vec(), b"1".to_vec(), b"abc.def*".to_vec()]
            )
        );
        // The high byte must round-trip raw, not through `from_utf8_lossy`'s
        // replacement character.
        assert_eq!(
            raised(b"VALUE", &[&[b'a', 0x80][..]]),
            (
                40,
                26,
                vec![b"VALUE".to_vec(), b"1".to_vec(), vec![b'a', 0x80]]
            )
        );
        assert_eq!(
            raised(b"VALUE", &[b"lowercase garbage"]),
            (
                40,
                26,
                vec![
                    b"VALUE".to_vec(),
                    b"1".to_vec(),
                    b"lowercase garbage".to_vec()
                ]
            )
        );
    }

    /// A leading-dot literal with no definition reads as its own upcased
    /// spelling; `.NIL`/`.TRUE`/`.FALSE` are the three this crate can answer
    /// beyond that fallback.
    #[test]
    fn value_of_an_undefined_dot_literal_is_its_own_spelling() {
        assert_eq!(output(b"c = '.'\nb = 'B'\nsay value(c||b)\n"), ".B\n");
        assert_eq!(
            output(b"say value('.nil')=.nil\nsay value('.true') value('.false')\n"),
            "1\n1 0\n"
        );
    }
}
