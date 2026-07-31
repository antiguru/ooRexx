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

//! Expression evaluation, parts one and two: terms, arithmetic,
//! concatenation, comparison and logic.
//!
//! `eval`/`eval_node`/`stack_span` moved here from Task 3's spike, extended
//! (Task 7) with `Stem`, `Compound`, `DotVariable`'s three admissible names,
//! `Prefix`, the seven arithmetic operators and the two concatenation forms
//! `||` did not already cover, and (Task 8) with the twelve comparison
//! operators (through `rexx-num`'s `compare_decoded`, never a hand-written
//! string comparison), the three binary logical operators `&`/`|`/`&&`, and
//! `ExprKind::Logical` (the comma-separated conditional list `IF a, b THEN`
//! desugars to). Every other `ExprKind` -- `Call`, `QualifiedCall`,
//! `Message`, `ClassResolver`, `List`, `VariableReference`, and any
//! `DotVariable` beyond the three -- still fails loudly through the
//! existing, exhaustive `form_name`.
//!
//! **Six functions here open a temps frame and then use `?`, so a raised
//! condition leaves their own `pop_frame` unreached. That is deliberate, and
//! Tasks 10 and 11 should copy it rather than repair it.** `eval_prefix`,
//! `eval_arithmetic`, `concat`, `eval_compare`, `eval_logical` and
//! `eval_logical_list` are the six. `step_in_temps_frame` pops
//! unconditionally with an outer watermark, and `pop_frame` truncates rather
//! than popping one frame, so the skipped inner frames are discarded when the
//! failing instruction returns. Nothing accumulates: an instruction is the
//! granularity at which the temps stack is guaranteed balanced, not an
//! expression.
//!
//! The alternative was measured and rejected rather than left untried. A
//! `Drop` guard cannot be written here at all, because it would have to hold
//! `&mut RootSet` across `self.eval(&mut self)` and that is two live `&mut`
//! borrows; the only escapes are a raw pointer, which is `unsafe` for no
//! strict need, and putting the `RootSet` behind a `RefCell`, which relaxes
//! this crate's borrow discipline to fix something that is not a defect.
//!
//! What this does depend on is every instruction loop routing through
//! `step_in_temps_frame`. A future caller that evaluates in a loop *outside*
//! instruction context and carries on past an `Err` would accumulate for
//! real, and would do it silently, since nothing asserts temps balance. Only
//! test helpers do that today.
//!
//! **Arithmetic, comparison and logic can all fail two different ways, and
//! this module is where that split first matters.** An unimplemented form
//! is `Loud`, unchanged. A real Rexx condition -- `1/0`, `'abc' + 1`, `2 **
//! 'x'`, `\'abc'`, `if 1, 'x' then` -- is `Raised` (`error.rs`), and both
//! convert into the one type `step` and everything above it propagate,
//! `Failure`.

use crate::error::Raised;
use crate::{Code, Failure, Interp, Loud, StackSpan};
use rexx_core::{NotNumeric, ObjRef};
use rexx_num::{CompareOp, DivOp, Number, compare_decoded};
use rexx_parse::{Expr, ExprKind, Operator, PrefixOp, compound_parts};

impl Interp {
    /// Evaluates one expression node, and keeps the depth bookkeeping D19
    /// needs.
    ///
    /// Split from `eval_node` so that the depth is decremented on every exit
    /// path including the `?` ones, without a guard type that would need to
    /// hold a borrow of `self` across the recursive call. Task 11 adds the
    /// limit check to this function, which is why it is the one that owns the
    /// counter.
    ///
    /// The stack probe: the address of a local here, recorded at the first
    /// level and at the deepest. Taking a raw pointer and casting it to
    /// `usize` is safe code, so this needs no `unsafe`, and measuring the real
    /// function rather than a replica of it is the whole reason to do it here.
    /// The two ends are written **together**, when the maximum is beaten, so
    /// they always describe one call chain; `StackSpan`'s doc has the
    /// measurement that made that necessary.
    pub(crate) fn eval(&mut self, code: &Code<'_>, expr: &Expr) -> Result<ObjRef, Failure> {
        let probe = 0u8;
        let here = &probe as *const u8 as usize;

        self.depth += 1;
        if self.depth == 1 {
            self.stack_entry = here;
        }
        if self.depth > self.max_depth {
            self.max_depth = self.depth;
            self.stack_first = self.stack_entry;
            self.stack_deepest = here;
        }

        let value = self.eval_node(code, expr);
        self.depth -= 1;
        value
    }

    fn eval_node(&mut self, code: &Code<'_>, expr: &Expr) -> Result<ObjRef, Failure> {
        match &expr.kind {
            ExprKind::Literal(bytes) => Ok(self.text(bytes)),
            // A constant's value is its own upcased spelling, which is
            // observable rather than incidental: `say 1e5` prints `1E5`.
            ExprKind::Constant(id) => Ok(self.text(code.symbols.name(*id).as_bytes())),
            // A bare stem read is not a new operation (D15a, `stem.rs`'s own
            // module doc): it goes through the exact same slot read every
            // variable uses, unset or not, so `Stem` and `Variable` share
            // this arm rather than each getting their own copy of it.
            ExprKind::Variable(id) | ExprKind::Stem(id) => {
                let (value, _novalue) = self.read(code, *id);
                Ok(value)
            }

            // `id` names the *whole* compound (its interned spelling is the
            // full dotted text); `compound_parts` decomposes it into the
            // stem's own name and the tail pieces `tail_key` (`stem.rs`,
            // Task 5) resolves into the one key `stem_get` looks up.
            ExprKind::Compound(id) => {
                let (stem_name, _tails) = compound_parts(code.symbols.name(*id));
                let key = self.tail_key(code, *id);
                Ok(self.stem_get(stem_name.as_bytes(), &key))
            }

            // The three admissible names (D15, "Expression evaluation"):
            // `.nil`, `.true`, `.false`. Anything else is Phase 5's
            // (environment symbols beyond these three) and falls through to
            // the loud failure below. The interned spelling keeps its
            // leading period and is upcased (`scanner.rs`'s symbol capture
            // includes the whole `.NAME` span before interning), so the
            // match is against `.NIL`/`.TRUE`/`.FALSE`, not `NIL`/etc.
            ExprKind::DotVariable(id) => match code.symbols.name(*id) {
                ".NIL" => Ok(ObjRef::NIL),
                // `.true`/`.false` need no representation of their own
                // (D15): they are the one-byte strings "1" and "0", built
                // fresh here the same way any other text value is.
                ".TRUE" => Ok(self.text(b"1")),
                ".FALSE" => Ok(self.text(b"0")),
                _ => Err(Loud::expression(&expr.kind).into()),
            },

            ExprKind::Prefix { op, operand } => self.eval_prefix(code, *op, operand),

            // Concatenation (D15's "Expression evaluation": `Abuttal`,
            // `Blank`, `||`, over bytes). `Abuttal` and `||` join directly;
            // `Blank` inserts exactly one space, regardless of how much
            // whitespace separated the terms in source -- measured, `'a'
            // 'b'` is `a b` whether one space or several sit between them
            // in the original text, because the scanner has already
            // collapsed that distinction into "this is a Blank operator"
            // before the parser ever sees it.
            ExprKind::Binary {
                op: op @ (Operator::Concatenate | Operator::Abuttal | Operator::Blank),
                left,
                right,
            } => {
                let separator: &[u8] = if *op == Operator::Blank { b" " } else { b"" };
                self.concat(code, left, right, separator)
            }

            // Arithmetic (D15's "Expression evaluation": `+ - * / % // **`,
            // through `rexx-num` under the settings in force *now* -- the
            // rendering of the result is fixed at creation, D15's own rule,
            // but the DIGITS/FORM an operation computes *under* is always
            // the activation's current ones, read fresh on every call.
            ExprKind::Binary {
                op:
                    op @ (Operator::Plus
                    | Operator::Subtract
                    | Operator::Multiply
                    | Operator::Divide
                    | Operator::IntDiv
                    | Operator::Remainder
                    | Operator::Power),
                left,
                right,
            } => self.eval_arithmetic(code, *op, left, right),

            // The twelve comparison operators (D15's "Expression
            // evaluation": numeric-or-string `= \= <> >< > < >= <= \> \<`,
            // and strict `== \== >> << >>= <<= \>> \<<`), all through
            // `rexx-num`'s `compare_decoded` -- see `eval_compare`'s own
            // doc comment for why no string comparison is written here.
            ExprKind::Binary {
                op:
                    op @ (Operator::Equal
                    | Operator::BackslashEqual
                    | Operator::GreaterThan
                    | Operator::BackslashGreaterThan
                    | Operator::LessThan
                    | Operator::BackslashLessThan
                    | Operator::GreaterThanEqual
                    | Operator::LessThanEqual
                    | Operator::StrictEqual
                    | Operator::StrictBackslashEqual
                    | Operator::StrictGreaterThan
                    | Operator::StrictBackslashGreaterThan
                    | Operator::StrictLessThan
                    | Operator::StrictBackslashLessThan
                    | Operator::StrictGreaterThanEqual
                    | Operator::StrictLessThanEqual
                    | Operator::LessThanGreaterThan
                    | Operator::GreaterThanLessThan),
                left,
                right,
            } => self.eval_compare(code, *op, left, right),

            // `&`/`|`/`&&`: always evaluate and check both operands, never
            // short-circuited (measured, see `eval_logical`'s own doc
            // comment) -- the opposite of `ExprKind::Logical`'s own rule
            // just below.
            ExprKind::Binary {
                op: op @ (Operator::And | Operator::Or | Operator::Xor),
                left,
                right,
            } => self.eval_logical(code, *op, left, right),

            // The comma-separated conditional list (`IF a, b THEN`, and
            // `WHEN`/`GUARD`/`WHILE`/`UNTIL`'s own versions of the same
            // syntax) -- see `eval_logical_list`'s own doc comment for the
            // short-circuit and sub-number this arm alone can give it.
            ExprKind::Logical(items) => self.eval_logical_list(code, items),

            other => Err(Loud::expression(other).into()),
        }
    }

    /// `+`/`-`/`\`, the three prefix operators (D15's "Expression
    /// evaluation"). `+`/`-` are arithmetic -- measured, `numeric digits 1
    /// ; say -12345` gives `-1E+4`, the same rounding `0 - 12345` gives, so
    /// they are implemented as exactly that rather than a sign flip on the
    /// operand's own digits. `\` is a **text** check, never a numeric one:
    /// measured, `say \'abc'` is 34.901, not 41.1, so a non-numeric operand
    /// is not converted first and does not fail as "nonnumeric".
    fn eval_prefix(
        &mut self,
        code: &Code<'_>,
        op: PrefixOp,
        operand: &Expr,
    ) -> Result<ObjRef, Failure> {
        let frame = self.roots.push_frame();
        let value = self.eval(code, operand)?;
        self.roots.push_temp(value);

        let result = match op {
            PrefixOp::Plus | PrefixOp::Minus => {
                let number = self.arith_operand(value)?;
                let digits = self.activation().settings.digits();
                let form = self.activation().settings.form();
                let result = if op == PrefixOp::Plus {
                    Number::zero().add(&number, digits)
                } else {
                    Number::zero().sub(&number, digits)
                }
                .map_err(Raised::from)?;
                self.number(result, saturate_digits(digits), form)
            }
            PrefixOp::Not => {
                let text = self.to_text(value).to_vec();
                let flipped = match logical_value(&text) {
                    Some(true) => b"0",
                    Some(false) => b"1",
                    None => return Err(Raised::not_logical(&text).into()),
                };
                self.text(flipped)
            }
        };

        self.roots.pop_frame(frame);
        Ok(result)
    }

    /// The seven arithmetic operators, sharing one operand-evaluation and
    /// error-conversion path.
    ///
    /// **`**`'s exponent is not evaluated the same way its base is, and
    /// that asymmetry is the fact being reproduced, not a shortcut**
    /// (measured: `2 ** 'x'` and `2 ** 2.5` both give 26.8, `'y' ** 2` and
    /// `'y' ** 'x'` both give 41.1 -- the base's failure always wins, and
    /// checked first). The base goes through `arith_operand`, exactly like
    /// every other operator's operands, and a conversion failure is 41.1.
    /// The exponent goes through `to_number` directly: on `NotNumeric` it
    /// is 26.8 with the exponent's own text as the substitution (there is
    /// no `Number` for `rexx-num`'s own `ArithError::PowerExponentNotWhole`
    /// to carry in that case); on a `Number` that parses but is not whole,
    /// `Number::pow` raises `PowerExponentNotWhole` itself and `Raised`'s
    /// `From<ArithError>` carries it through unchanged.
    fn eval_arithmetic(
        &mut self,
        code: &Code<'_>,
        op: Operator,
        left: &Expr,
        right: &Expr,
    ) -> Result<ObjRef, Failure> {
        let frame = self.roots.push_frame();
        let left_value = self.eval(code, left)?;
        self.roots.push_temp(left_value);
        let right_value = self.eval(code, right)?;
        self.roots.push_temp(right_value);

        let left_number = self.arith_operand(left_value)?;
        let digits = self.activation().settings.digits();
        let form = self.activation().settings.form();

        let result = if op == Operator::Power {
            let exponent = match self.to_number(right_value) {
                Ok(number) => number,
                Err(NotNumeric) => {
                    let text = self.to_text(right_value).to_vec();
                    return Err(Raised::power_exponent_not_whole(&text).into());
                }
            };
            left_number.pow(&exponent, digits)
        } else {
            let right_number = self.arith_operand(right_value)?;
            match op {
                Operator::Plus => left_number.add(&right_number, digits),
                Operator::Subtract => left_number.sub(&right_number, digits),
                Operator::Multiply => left_number.mul(&right_number, digits),
                Operator::Divide => left_number.div(&right_number, digits, DivOp::Divide),
                Operator::IntDiv => left_number.div(&right_number, digits, DivOp::IntegerDivide),
                Operator::Remainder => left_number.div(&right_number, digits, DivOp::Remainder),
                _ => unreachable!("eval_node only dispatches the seven arithmetic operators here"),
            }
        }
        .map_err(Raised::from)?;

        let value = self.number(result, saturate_digits(digits), form);
        self.roots.pop_frame(frame);
        Ok(value)
    }

    /// Converts an arithmetic operand to a `Number`, or 41.1 with the
    /// operand's own rendered text as the substitution -- measured, `say
    /// 'abc' + 1` reports `Nonnumeric value ("abc")`, the operand as it
    /// renders, not upcased or otherwise transformed.
    fn arith_operand(&mut self, value: ObjRef) -> Result<Number, Failure> {
        match self.to_number(value) {
            Ok(number) => Ok(number),
            Err(NotNumeric) => {
                let text = self.to_text(value).to_vec();
                Err(Raised::nonnumeric(&text).into())
            }
        }
    }

    /// The shared body of `||`/`Abuttal` (no separator) and `Blank` (one
    /// space). Established here rather than retrofitted: a value held only
    /// in a Rust local across an allocation is invisible to the collector,
    /// so both operands are pushed to `RootSet` before the join, which
    /// allocates, runs -- the same discipline the concatenation arm always
    /// used, now shared by all three operators that need it.
    fn concat(
        &mut self,
        code: &Code<'_>,
        left: &Expr,
        right: &Expr,
        separator: &[u8],
    ) -> Result<ObjRef, Failure> {
        let frame = self.roots.push_frame();
        let left_value = self.eval(code, left)?;
        self.roots.push_temp(left_value);
        let right_value = self.eval(code, right)?;
        self.roots.push_temp(right_value);

        let mut bytes = self.to_text(left_value).to_vec();
        bytes.extend_from_slice(separator);
        bytes.extend_from_slice(&self.to_text(right_value));
        let joined = self.text(&bytes);

        // `joined` is unrooted from here to the caller's own `push_temp`,
        // and nothing between the two allocates.
        self.roots.pop_frame(frame);
        Ok(joined)
    }

    /// The twelve comparison operators, all through `rexx-num`'s
    /// `compare_decoded` -- **no string comparison is written here**, per
    /// the plan's own instruction and this crate's standing rule against a
    /// second copy of logic `rexx-num` already owns (the same rule
    /// `compare.rs`'s own module doc states for its three entry points).
    ///
    /// Calls `to_number` for the non-strict family only, and never for
    /// strict operators, which `compare_decoded` never even inspects a
    /// `Number` for (`is_strict()`'s short-circuit, `compare.rs:154`) --
    /// asking for one anyway would force a needless parse of an operand
    /// `==`/`>>`/... never numerically compares. When `to_number` is
    /// called, it already routes through `Body::Text`'s tri-state `num`
    /// cache (`value.rs`), so an operand already asked about is not
    /// reparsed -- passing its *bytes* to `compare_decoded` alongside the
    /// already-parsed `Number` is what the plan's "do not quietly defeat
    /// the cache by using the `&str` entry point" is about, not a
    /// prohibition on calling `to_number` at all.
    fn eval_compare(
        &mut self,
        code: &Code<'_>,
        op: Operator,
        left: &Expr,
        right: &Expr,
    ) -> Result<ObjRef, Failure> {
        let frame = self.roots.push_frame();
        let left_value = self.eval(code, left)?;
        self.roots.push_temp(left_value);
        let right_value = self.eval(code, right)?;
        self.roots.push_temp(right_value);

        let left_bytes = self.to_text(left_value).into_owned();
        let right_bytes = self.to_text(right_value).into_owned();

        let strict = is_strict_compare(op);
        let left_number = if strict {
            None
        } else {
            self.to_number(left_value).ok()
        };
        let right_number = if strict {
            None
        } else {
            self.to_number(right_value).ok()
        };

        let digits = self.activation().settings.digits();
        let fuzz = self.activation().settings.fuzz();
        let holds = compare_decoded(
            &left_bytes,
            left_number.as_ref(),
            &right_bytes,
            right_number.as_ref(),
            digits,
            fuzz,
            compare_op(op),
        )
        .map_err(Raised::from)?;

        let result = self.text(if holds { b"1" } else { b"0" });
        self.roots.pop_frame(frame);
        Ok(result)
    }

    /// `&`/`|`/`&&`. **Both operands are always evaluated and checked,
    /// never short-circuited**, and the left is checked before the right
    /// when both are bad -- measured: `say (0 & 'x')` and `say (1 | 'x')`
    /// both raise 34.901 on `"x"` even though the result is already
    /// decided by the first operand alone, and `say ('y' & 'x')` (both
    /// bad) reports `"y"`, the left operand's own text, not `"x"`. The
    /// opposite rule from `ExprKind::Logical`'s comma list, just below,
    /// which does short-circuit -- both are implemented exactly as
    /// measured rather than made to agree with each other.
    fn eval_logical(
        &mut self,
        code: &Code<'_>,
        op: Operator,
        left: &Expr,
        right: &Expr,
    ) -> Result<ObjRef, Failure> {
        let frame = self.roots.push_frame();
        let left_value = self.eval(code, left)?;
        self.roots.push_temp(left_value);
        let right_value = self.eval(code, right)?;
        self.roots.push_temp(right_value);

        let left_text = self.to_text(left_value).to_vec();
        let left_bool = logical_value(&left_text).ok_or_else(|| Raised::not_logical(&left_text))?;
        let right_text = self.to_text(right_value).to_vec();
        let right_bool =
            logical_value(&right_text).ok_or_else(|| Raised::not_logical(&right_text))?;

        let holds = match op {
            Operator::And => left_bool && right_bool,
            Operator::Or => left_bool || right_bool,
            Operator::Xor => left_bool != right_bool,
            _ => unreachable!("eval_node only dispatches And/Or/Xor here"),
        };

        let result = self.text(if holds { b"1" } else { b"0" });
        self.roots.pop_frame(frame);
        Ok(result)
    }

    /// `ExprKind::Logical`: the comma-separated conditional list `IF a, b,
    /// c THEN` (and `WHEN`/`GUARD`/`WHILE`/`UNTIL`'s own versions)
    /// desugars to, "a logical AND of its parts" (`ast.rs`'s own doc
    /// comment).
    ///
    /// **Short-circuits on the first element that checks out false**,
    /// unlike `&` (`eval_logical`'s own doc comment). Measured with `if 0,
    /// (1/0) then nop` followed by `say 'reached'`, which prints `reached`
    /// and exits 0, and `if 1, 0, (1/0) then nop`, which does the same with
    /// three elements. Every element up to and including the first false one
    /// is evaluated left to right and checked to be exactly `0`/`1`; nothing
    /// after it is touched.
    ///
    /// The probe is `(1/0)` rather than the `'x'` an earlier version of this
    /// comment cited, because `'x'` cannot tell the two candidate rules
    /// apart: a literal evaluates harmlessly, so skipping only the *check*
    /// would produce the same clean run. `(1/0)` raises 42.3 the moment it is
    /// evaluated, and `if 1, (1/0)` does exactly that (rc 214), so what is
    /// skipped is the evaluation.
    ///
    /// **Always 34.6, the per-element message, never 34.1/34.2/34.3/34.4**
    /// -- measured, `if 1, 'x' then` and `if 'x', 1 then` both give 34.6,
    /// "Value of logical list expression element must be exactly...",
    /// regardless of position, while a *single*, non-list condition
    /// (`if 'x' then`, no comma) gives 34.1 instead. The four
    /// keyword-specific numbers belong to `IF`/`WHEN`/`WHILE`/`UNTIL`
    /// themselves (Tasks 9-11), evaluating a bare condition `Expr`
    /// directly and checking its own result rather than going through this
    /// arm at all -- this function has no instruction context to prefer
    /// one of those numbers over 34.6, and the oracle's own rule does not
    /// ask it to: 34.6 is what a list element gets independent of which of
    /// the five keywords built the list.
    fn eval_logical_list(&mut self, code: &Code<'_>, items: &[Expr]) -> Result<ObjRef, Failure> {
        let frame = self.roots.push_frame();
        let mut holds = true;
        for item in items {
            let value = self.eval(code, item)?;
            self.roots.push_temp(value);
            let text = self.to_text(value).to_vec();
            let item_holds =
                logical_value(&text).ok_or_else(|| Raised::logical_list_element(&text))?;
            if !item_holds {
                holds = false;
                break;
            }
        }

        let result = self.text(if holds { b"1" } else { b"0" });
        self.roots.pop_frame(frame);
        Ok(result)
    }

    pub(crate) fn stack_span(&self) -> StackSpan {
        StackSpan {
            max_depth: self.max_depth,
            bytes: self.stack_first.saturating_sub(self.stack_deepest),
        }
    }
}

/// Narrows `Settings::digits()` (`u64`) to `Body::Num`'s `created_digits`
/// (`u32`) by saturating rather than rejecting or panicking.
///
/// Unreachable in practice, which is the reason saturation is the right
/// choice rather than a guess dressed up as one: `u32::MAX` is about four
/// billion significant figures, and a program that set `NUMERIC DIGITS`
/// anywhere near that would exhaust memory building a single `Number`
/// (`rexx-num` reserves working storage proportional to `DIGITS`) long
/// before precision ever mattered. No corpus program, and no realistic
/// one, can reach the clamp.
fn saturate_digits(digits: u64) -> u32 {
    u32::try_from(digits).unwrap_or(u32::MAX)
}

/// A logical value is *exactly* the one-byte string `0` or `1` -- text, not
/// numeric, no coercion (D15's "Expression evaluation"; measured, `' 1 '`,
/// `'01'`, `'1.0'` and `''` are each not logical, error 34). Shared by
/// prefix `\`, `&`/`|`/`&&` and `ExprKind::Logical`'s per-element check,
/// which differ only in *which* sub-number they raise on `None`, never in
/// what counts as logical.
pub(crate) fn logical_value(text: &[u8]) -> Option<bool> {
    match text {
        b"0" => Some(false),
        b"1" => Some(true),
        _ => None,
    }
}

/// `Operator` -> `rexx-num`'s `CompareOp`, for the eighteen `Operator`
/// variants `eval_node` dispatches to `eval_compare`. `CompareOp` has only
/// twelve variants: `\=`/`<>`/`><` (`BackslashEqual`/`LessThanGreaterThan`/
/// `GreaterThanLessThan`) all mean `NotEqual` (`compare.rs`'s own doc
/// comment: the interpreter's operator table repeats one method pointer for
/// all three), and the four backslash-negated forms invert their positive
/// counterpart's sense rather than getting a `CompareOp` of their own: `\>`
/// ("not greater than") is `LessEqual`, `\<` is `GreaterEqual`, and their
/// strict siblings `\>>`/`\<<` map the same way onto `StrictLessEqual`/
/// `StrictGreaterEqual`. Verified against the oracle (this task's report),
/// not derived from the names alone -- `\>>`/`\<<` are strict-family byte
/// comparisons, where "greater"/"less" do not carry the numeric intuition
/// their spelling suggests.
fn compare_op(op: Operator) -> CompareOp {
    use Operator::*;
    match op {
        Equal => CompareOp::Equal,
        BackslashEqual | LessThanGreaterThan | GreaterThanLessThan => CompareOp::NotEqual,
        GreaterThan => CompareOp::Greater,
        LessThan => CompareOp::Less,
        GreaterThanEqual => CompareOp::GreaterEqual,
        LessThanEqual => CompareOp::LessEqual,
        BackslashGreaterThan => CompareOp::LessEqual,
        BackslashLessThan => CompareOp::GreaterEqual,
        StrictEqual => CompareOp::StrictEqual,
        StrictBackslashEqual => CompareOp::StrictNotEqual,
        StrictGreaterThan => CompareOp::StrictGreater,
        StrictLessThan => CompareOp::StrictLess,
        StrictGreaterThanEqual => CompareOp::StrictGreaterEqual,
        StrictLessThanEqual => CompareOp::StrictLessEqual,
        StrictBackslashGreaterThan => CompareOp::StrictLessEqual,
        StrictBackslashLessThan => CompareOp::StrictGreaterEqual,
        other => unreachable!(
            "eval_node only dispatches the eighteen comparison operators here, got {other:?}"
        ),
    }
}

/// Whether `op` is one of the eight strict comparison operators -- decided
/// directly off `Operator` rather than off `CompareOp`, because
/// `CompareOp::is_strict` is private to `rexx-num` (`compare.rs`) and this
/// crate has no other need for a public one: strictness is only ever asked
/// here, to decide whether calling `to_number` is worth it at all before
/// `compare_decoded` is reached (a strict comparison never looks at a
/// `Number`, so parsing one first would be pure waste).
fn is_strict_compare(op: Operator) -> bool {
    use Operator::*;
    matches!(
        op,
        StrictEqual
            | StrictBackslashEqual
            | StrictGreaterThan
            | StrictLessThan
            | StrictGreaterThanEqual
            | StrictLessThanEqual
            | StrictBackslashGreaterThan
            | StrictBackslashLessThan
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::{BodyKey, ProgramId};
    use crate::{Activation, error::Failure};
    use rexx_parse::{InstructionKind, Program, parse_program};
    use std::collections::HashMap;
    use std::rc::Rc;

    /// Pushes a fresh top-level activation for `program`, the same setup
    /// `Interp::run` does, so a test can drive `eval` through a live
    /// activation without running the instruction loop -- `step`'s
    /// `Assignment` arm only handles `ExprKind::Variable` targets today
    /// (`Stem`/`Compound` are Task 9's dispatch), so a program that
    /// assigns a stem cannot simply be run; tests that need one set it up
    /// directly through `stem_assign`/`stem_set` instead.
    fn activate(interp: &mut Interp, program: Program) -> Rc<Program> {
        let program = Rc::new(program);
        let id = ProgramId(interp.programs.len());
        interp.programs.push(Rc::clone(&program));
        let plan = interp.plan_for(
            BodyKey {
                program: id,
                directive: None,
            },
            &program.main,
            &program.symbols,
        );
        let frame = interp.roots.push_slots(plan.len());
        interp
            .activations
            .push(Activation::new(Rc::clone(&program), plan, frame));
        program
    }

    /// Parses `source` (one `SAY` of an expression), activates it, and
    /// evaluates that expression -- the one piece of machinery almost
    /// every test below needs.
    fn eval_source(interp: &mut Interp, source: &[u8]) -> Result<ObjRef, Failure> {
        let program = parse_program(source.to_vec()).expect("test program parses");
        let program = activate(interp, program);
        let expr = match &program.main.instructions[0].kind {
            InstructionKind::Say {
                expression: Some(expr),
            } => expr,
            other => panic!("expected a SAY with an expression, got {other:?}"),
        };
        let code = Code {
            body: &program.main,
            symbols: &program.symbols,
            slots: &HashMap::new(),
        };
        interp.eval(&code, expr)
    }

    fn eval_text(interp: &mut Interp, source: &[u8]) -> Vec<u8> {
        let value = eval_source(interp, source)
            .unwrap_or_else(|failure| panic!("expected {source:?} to evaluate, got {failure:?}"));
        interp.to_text(value).to_vec()
    }

    /// `eval_source`, against the activation already on top of the stack
    /// rather than pushing a fresh one.
    ///
    /// For a test that has already bound state into a frame -- via
    /// `stem_assign`/`stem_set`, or `activation_mut().settings` -- and
    /// wants to evaluate against it. Calling `activate` (hence
    /// `eval_source`) a *second* time here would push a second, empty
    /// frame that shadows the one already set up, exactly the trap
    /// `stem.rs`'s own multi-clause test found
    /// (`a_multi_level_tail_joins_its_pieces_with_a_period`): the parsed
    /// `Program` does not need to be remembered by `Interp` at all for a
    /// one-off eval, so it is a plain local here, never an `Rc`.
    fn eval_in_place(interp: &mut Interp, source: &[u8]) -> Result<ObjRef, Failure> {
        let program = parse_program(source.to_vec()).expect("test program parses");
        let expr = match &program.main.instructions[0].kind {
            InstructionKind::Say {
                expression: Some(expr),
            } => expr,
            other => panic!("expected a SAY with an expression, got {other:?}"),
        };
        let code = Code {
            body: &program.main,
            symbols: &program.symbols,
            slots: &HashMap::new(),
        };
        interp.eval(&code, expr)
    }

    fn eval_in_place_text(interp: &mut Interp, source: &[u8]) -> Vec<u8> {
        let value = eval_in_place(interp, source)
            .unwrap_or_else(|failure| panic!("expected {source:?} to evaluate, got {failure:?}"));
        interp.to_text(value).to_vec()
    }

    // ---- terms ----

    #[test]
    fn a_literal_is_its_own_bytes() {
        let mut interp = Interp::new(false);
        assert_eq!(eval_text(&mut interp, b"say 123"), b"123");
    }

    #[test]
    fn a_constant_is_its_own_upcased_spelling() {
        // say 1e5 -> 1E5 (measured against the oracle; D15's own example).
        let mut interp = Interp::new(false);
        assert_eq!(eval_text(&mut interp, b"say 1e5"), b"1E5");
    }

    #[test]
    fn a_variable_reads_through_the_plan() {
        let mut interp = Interp::new(false);
        let program = parse_program(b"say x".to_vec()).expect("test program parses");
        let program = activate(&mut interp, program);
        let five = interp.text(b"5");
        let slot = interp.slot_of(b"X");
        let frame = interp.activation().frame;
        interp.roots.set_slot(frame, slot, five);
        let expr = match &program.main.instructions[0].kind {
            InstructionKind::Say {
                expression: Some(expr),
            } => expr,
            other => panic!("expected a SAY with an expression, got {other:?}"),
        };
        let code = Code {
            body: &program.main,
            symbols: &program.symbols,
            slots: &HashMap::new(),
        };
        let value = interp.eval(&code, expr).unwrap();
        assert_eq!(&*interp.to_text(value), b"5");
    }

    #[test]
    fn a_bare_stem_reads_through_the_same_path_as_a_variable() {
        // w. = 'wd' ; say w. -> wd
        let mut interp = Interp::new(false);
        activate(
            &mut interp,
            parse_program(b"nop".to_vec()).expect("test program parses"),
        );
        let wd = interp.text(b"wd");
        interp.stem_assign(b"W.", wd);
        assert_eq!(eval_in_place_text(&mut interp, b"say w."), b"wd");
    }

    #[test]
    fn a_compound_read_resolves_its_tail_and_looks_it_up() {
        // a.1 = 'x' ; say a.1 -> x
        let mut interp = Interp::new(false);
        activate(
            &mut interp,
            parse_program(b"nop".to_vec()).expect("test program parses"),
        );
        let x = interp.text(b"x");
        interp.stem_set(b"A.", b"1", x);
        assert_eq!(eval_in_place_text(&mut interp, b"say a.1"), b"x");
    }

    #[test]
    fn the_three_admissible_dot_variables() {
        let mut interp = Interp::new(false);
        assert_eq!(eval_text(&mut interp, b"say .nil"), b"The NIL object");
        assert_eq!(eval_text(&mut interp, b"say .true"), b"1");
        assert_eq!(eval_text(&mut interp, b"say .false"), b"0");
    }

    #[test]
    fn a_dot_variable_beyond_the_three_fails_loudly() {
        let mut interp = Interp::new(false);
        let failure = eval_source(&mut interp, b"say .foo").unwrap_err();
        assert!(matches!(failure, Failure::Loud(_)), "got {failure:?}");
    }

    // ---- prefix ----

    #[test]
    fn prefix_plus_and_minus_are_arithmetic_not_a_sign_flip() {
        // numeric digits 1 ; say -12345 -> -1E+4 ; say +12345 -> 1E+4
        // (measured: the same rounding `0 - 12345`/`0 + 12345` gives).
        // `NUMERIC` is not run through `step` (Task 9's instruction), so
        // `DIGITS` is set directly on the activation the same way Task 9's
        // own implementation eventually will.
        let mut interp = Interp::new(false);
        activate(
            &mut interp,
            parse_program(b"nop".to_vec()).expect("test program parses"),
        );
        interp
            .activation_mut()
            .settings
            .set_digits_str("1")
            .unwrap();
        assert_eq!(eval_in_place_text(&mut interp, b"say -12345"), b"-1E+4");
        assert_eq!(eval_in_place_text(&mut interp, b"say +12345"), b"1E+4");
    }

    #[test]
    fn prefix_not_flips_a_logical_value() {
        let mut interp = Interp::new(false);
        assert_eq!(eval_text(&mut interp, b"say \\1"), b"0");
        assert_eq!(eval_text(&mut interp, b"say \\0"), b"1");
    }

    #[test]
    fn prefix_not_on_a_non_logical_value_raises_34_901() {
        // say \'abc' -> Error 34.901
        let mut interp = Interp::new(false);
        let failure = eval_source(&mut interp, b"say \\'abc'").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (34, 901));
        assert_eq!(raised.additional, vec!["abc".to_string()]);
    }

    // ---- arithmetic ----

    #[test]
    fn the_seven_arithmetic_operators() {
        let mut interp = Interp::new(false);
        assert_eq!(eval_text(&mut interp, b"say 1+2"), b"3");
        assert_eq!(eval_text(&mut interp, b"say 5-3"), b"2");
        assert_eq!(eval_text(&mut interp, b"say 2*3"), b"6");
        assert_eq!(eval_text(&mut interp, b"say 7/2"), b"3.5");
        assert_eq!(eval_text(&mut interp, b"say 7%2"), b"3");
        assert_eq!(eval_text(&mut interp, b"say 7//2"), b"1");
        assert_eq!(eval_text(&mut interp, b"say 2**3"), b"8");
    }

    #[test]
    fn divide_by_zero_raises_42_3() {
        // say 1/0 -> Error 42.3, rc 214
        let mut interp = Interp::new(false);
        let failure = eval_source(&mut interp, b"say 1/0").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (42, 3));
    }

    #[test]
    fn remainder_by_zero_also_raises_42_3() {
        // say 1//0 -> Error 42.3, rc 214 (the same DivideByZero as /)
        let mut interp = Interp::new(false);
        let failure = eval_source(&mut interp, b"say 1//0").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (42, 3));
    }

    #[test]
    fn a_nonnumeric_operand_raises_41_1_with_its_own_text() {
        // say 'abc' + 1 -> Error 41.1, "Nonnumeric value (\"abc\")"
        let mut interp = Interp::new(false);
        let failure = eval_source(&mut interp, b"say 'abc'+1").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (41, 1));
        assert_eq!(raised.additional, vec!["abc".to_string()]);
    }

    #[test]
    fn a_non_numeric_power_exponent_raises_26_8_not_41_1() {
        // say 2 ** 'x' -> Error 26.8, "found \"x\""
        // 'y' ** 2     -> Error 41.1 (the base's ordinary nonnumeric path)
        let mut interp = Interp::new(false);
        let failure = eval_source(&mut interp, b"say 2**'x'").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (26, 8));
        assert_eq!(raised.additional, vec!["x".to_string()]);

        let base_failure = eval_source(&mut interp, b"say 'y'**2").unwrap_err();
        let Failure::Raised(base_raised) = base_failure else {
            panic!("expected Raised, got {base_failure:?}");
        };
        assert_eq!((base_raised.number, base_raised.sub), (41, 1));
    }

    #[test]
    fn a_number_created_by_arithmetic_renders_under_the_digits_that_made_it() {
        // numeric digits 1 ; x = 15 + 0 ; x is 20, so x + 6 is 3E+1 while
        // 15 + 6 is 2E+1 -- D15's own SmallInt-admissibility transcript,
        // reachable end to end through eval now rather than constructed by
        // hand as Task 4 had to.
        let mut interp = Interp::new(false);
        activate(
            &mut interp,
            parse_program(b"nop".to_vec()).expect("test program parses"),
        );
        interp
            .activation_mut()
            .settings
            .set_digits_str("1")
            .unwrap();
        assert_eq!(eval_in_place_text(&mut interp, b"say 15+0"), b"2E+1");
        assert_eq!(eval_in_place_text(&mut interp, b"say 15+6"), b"2E+1");
    }

    // ---- concatenation ----

    #[test]
    fn the_three_concatenation_forms() {
        let mut interp = Interp::new(false);
        assert_eq!(eval_text(&mut interp, b"say 'a'||'b'"), b"ab");
        assert_eq!(eval_text(&mut interp, b"say 'a' 'b'"), b"a b");

        // Abuttal: two adjacent terms with no operator and no whitespace.
        // Not `'a'('b')` -- a quoted literal directly followed by `(...)`
        // is call syntax (`CallTarget::Literal`), measured: `say
        // 'a'('b')` is Error 43.1, "Could not find routine \"a\"", not
        // concatenation. `x'b'` (a variable directly followed by a
        // literal) is the real Abuttal shape, matching the oracle
        // transcript this task's report already verified (`x = 'a'; say
        // x'b'` -> `ab`).
        let program = parse_program(b"say x'b'".to_vec()).expect("test program parses");
        let program = activate(&mut interp, program);
        let a = interp.text(b"a");
        let slot = interp.slot_of(b"X");
        let frame = interp.activation().frame;
        interp.roots.set_slot(frame, slot, a);
        let expr = match &program.main.instructions[0].kind {
            InstructionKind::Say {
                expression: Some(expr),
            } => expr,
            other => panic!("expected a SAY with an expression, got {other:?}"),
        };
        let code = Code {
            body: &program.main,
            symbols: &program.symbols,
            slots: &HashMap::new(),
        };
        let value = interp.eval(&code, expr).unwrap();
        assert_eq!(&*interp.to_text(value), b"ab");
    }

    // ---- comparison ----

    #[test]
    fn the_thirteen_line_comparison_transcript() {
        // The plan's own measured block, verbatim (task-8-report.md has
        // the oracle run). The first row is the one that discriminates the
        // real string rule from "blank-pad the shorter on the right".
        let mut interp = Interp::new(false);
        assert_eq!(eval_text(&mut interp, b"say (' a' = 'a')"), b"1");
        assert_eq!(eval_text(&mut interp, b"say ('09'x'a' = 'a')"), b"1");
        assert_eq!(eval_text(&mut interp, b"say ('a' = 'a'||'09'x)"), b"1");
        assert_eq!(eval_text(&mut interp, b"say ('a' = 'a ')"), b"1");
        assert_eq!(eval_text(&mut interp, b"say ('a b' = 'a  b')"), b"0");
        assert_eq!(eval_text(&mut interp, b"say ('' = ' ')"), b"1");
        assert_eq!(eval_text(&mut interp, b"say ('01' = '1')"), b"1");
        assert_eq!(eval_text(&mut interp, b"say (' 1 ' = 1)"), b"1");
        assert_eq!(eval_text(&mut interp, b"say ('a' = 1)"), b"0");
        assert_eq!(eval_text(&mut interp, b"say ('10' >> '9')"), b"0");
        assert_eq!(eval_text(&mut interp, b"say ('10' > '9')"), b"1");
        assert_eq!(eval_text(&mut interp, b"say ('a' << 'a ')"), b"1");
        assert_eq!(eval_text(&mut interp, b"say ('01' == '1')"), b"0");
    }

    #[test]
    fn backslash_negated_and_synonym_forms() {
        // Not in the plan's own transcript; measured separately (report)
        // to pin the Operator -> CompareOp mapping, particularly that
        // \>/\< invert the *positive* comparison's sense rather than
        // getting a CompareOp of their own.
        let mut interp = Interp::new(false);
        assert_eq!(eval_text(&mut interp, b"say ('9' \\> '10')"), b"1");
        assert_eq!(eval_text(&mut interp, b"say ('9' \\< '10')"), b"0");
        assert_eq!(eval_text(&mut interp, b"say ('9' \\>> '10')"), b"0");
        assert_eq!(eval_text(&mut interp, b"say ('9' \\<< '10')"), b"1");
        assert_eq!(eval_text(&mut interp, b"say ('9' <> '10')"), b"1");
        assert_eq!(eval_text(&mut interp, b"say ('9' >< '10')"), b"1");
        assert_eq!(eval_text(&mut interp, b"say ('9' \\= '10')"), b"1");
        assert_eq!(eval_text(&mut interp, b"say ('9' \\== '9')"), b"0");
        assert_eq!(eval_text(&mut interp, b"say ('9' >>= '9')"), b"1");
        assert_eq!(eval_text(&mut interp, b"say ('8' >>= '9')"), b"0");
        assert_eq!(eval_text(&mut interp, b"say ('10' <<= '9')"), b"1");

        // **Equal operands, and every row above is unequal ones.** Seven of
        // the eighteen mapping rows survived a mutation run against the whole
        // suite: `\>` to `Less`, `\<` to `Greater`, `\>>` to `StrictLess`,
        // `\<<` to `StrictGreater`, `<<=` to `StrictLess`, `>=` to `Greater`,
        // and `<=` to `Less`. Equality is the only case that separates
        // `LessEqual` from `Less`, so unequal operands cannot tell those pairs
        // apart however many of them a test lists, and non-strict `>=`/`<=`
        // appeared in no test at all.
        //
        // The gap started as a measurement gap and became a test gap: the
        // fourteen oracle measurements behind the rows above have the same
        // blind spot, no equal-operand case for any negated form. Each value
        // below was measured against `build/bin/rexx` before being written
        // here, not derived from the mapping it checks.
        assert_eq!(eval_text(&mut interp, b"say ('9' \\> '9')"), b"1");
        assert_eq!(eval_text(&mut interp, b"say ('9' \\< '9')"), b"1");
        assert_eq!(eval_text(&mut interp, b"say ('9' \\>> '9')"), b"1");
        assert_eq!(eval_text(&mut interp, b"say ('9' \\<< '9')"), b"1");
        assert_eq!(eval_text(&mut interp, b"say ('9' <<= '9')"), b"1");
        assert_eq!(eval_text(&mut interp, b"say ('9' <= '9')"), b"1");
        assert_eq!(eval_text(&mut interp, b"say ('9' >= '9')"), b"1");
    }

    #[test]
    fn a_comparison_reuses_an_already_parsed_num_cache() {
        // Not a behaviour difference visible from the answer alone (the
        // numeric family gives the same result whether or not the cache
        // was already warm) -- what this actually exercises is that
        // eval_compare, reading `x` back out through a variable, still
        // takes the numeric path on an object whose `num` cache this test
        // itself already filled, rather than `to_number` inside
        // eval_compare somehow needing a cold object to work at all.
        // 007 and 7 comparing numerically equal (not "0" -- a byte compare
        // would disagree with the leading zero) is what proves the
        // numeric path, not the string one, ran.
        let mut interp = Interp::new(false);
        activate(
            &mut interp,
            parse_program(b"nop".to_vec()).expect("test program parses"),
        );
        let x = interp.text(b"007");
        interp.to_number(x).expect("007 parses, filling its cache");
        let slot = interp.slot_of(b"X");
        let frame = interp.activation().frame;
        interp.roots.set_slot(frame, slot, x);
        assert_eq!(eval_in_place_text(&mut interp, b"say (x = '7')"), b"1");
    }

    #[test]
    fn strict_equality_compares_bytes_where_the_ordinary_form_compares_value() {
        // The contrast is the test: the same two operands answer 0 under `==`
        // and 1 under `=`, so this pins that `==` reaches `CompareOp`'s strict
        // row rather than the ordinary one.
        //
        // **It is deliberately not named for skipping `to_number`, which was
        // this test's previous name and claim.** That claim is unobservable
        // from any result, and its stated failure story ("would compare 1 == 1
        // and answer 1") is wrong: `compare_decoded` returns on
        // `op.is_strict()` before reading either `Number`, so routing a strict
        // operator through `to_number` changes nothing a program can see.
        // Proved by mutation, not by reading -- hardwiring `is_strict_compare`
        // to `false` left the whole suite green. `is_strict_compare` gating
        // the parse is a real saving and its own doc comment gives that
        // honest, performance rationale; no behavioural test can guard it.
        let mut interp = Interp::new(false);
        assert_eq!(eval_text(&mut interp, b"say ('01' == '1')"), b"0");
        assert_eq!(eval_text(&mut interp, b"say ('01' = '1')"), b"1");
    }

    // ---- logical ----

    #[test]
    fn the_three_logical_operators_and_their_truth_tables() {
        let mut interp = Interp::new(false);
        assert_eq!(eval_text(&mut interp, b"say (1 & 1)"), b"1");
        assert_eq!(eval_text(&mut interp, b"say (1 & 0)"), b"0");
        assert_eq!(eval_text(&mut interp, b"say (0 | 0)"), b"0");
        assert_eq!(eval_text(&mut interp, b"say (1 && 1)"), b"0");
        assert_eq!(eval_text(&mut interp, b"say (1 && 0)"), b"1");
    }

    #[test]
    fn logical_operators_raise_34_901_on_a_non_logical_operand() {
        // ' 1 ' & 1 -> Error 34.901, found " 1 " (and '01'/'1.0'/'' alike,
        // measured in the report -- one representative here, the same
        // check `logical_value` gives all four).
        let mut interp = Interp::new(false);
        let failure = eval_source(&mut interp, b"say (' 1 ' & 1)").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (34, 901));
        assert_eq!(raised.additional, vec![" 1 ".to_string()]);
    }

    #[test]
    fn logical_operators_do_not_short_circuit() {
        // say (0 & 'x') -> still raises on 'x', even though 0 already
        // decides the AND -- both operands are always evaluated and
        // checked. Contrast with ExprKind::Logical's comma list, below.
        let mut interp = Interp::new(false);
        let and_failure = eval_source(&mut interp, b"say (0 & 'x')").unwrap_err();
        assert!(
            matches!(and_failure, Failure::Raised(_)),
            "got {and_failure:?}"
        );
        let or_failure = eval_source(&mut interp, b"say (1 | 'x')").unwrap_err();
        assert!(
            matches!(or_failure, Failure::Raised(_)),
            "got {or_failure:?}"
        );
    }

    #[test]
    fn logical_operators_check_the_left_operand_first() {
        // say ('y' & 'x') -> reports "y", the left operand, when both are
        // bad -- confirms evaluation order rather than assuming it.
        let mut interp = Interp::new(false);
        let failure = eval_source(&mut interp, b"say ('y' & 'x')").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!(raised.additional, vec!["y".to_string()]);
    }

    // ---- ExprKind::Logical (the comma list) ----

    /// Parses `if <cond> then nop`, extracts the `IF`'s own `condition`
    /// (an `ExprKind::Logical` for a comma list, an ordinary `Expr`
    /// otherwise) and evaluates it directly. The only way to build one:
    /// comma-list syntax is gated to `IF`/`WHEN`/`GUARD`/`WHILE`/`UNTIL` in
    /// the parser (`ast.rs`'s own doc comment on `ExprKind::Logical`), and
    /// none of those instructions run yet -- Tasks 9-11's `step` dispatch
    /// does not reach `IF` -- so a real program can only ever hand the
    /// parsed condition to this test directly, never through `run`.
    fn eval_condition(interp: &mut Interp, source: &[u8]) -> Result<ObjRef, Failure> {
        let program = parse_program(source.to_vec()).expect("test program parses");
        let code = Code {
            body: &program.main,
            symbols: &program.symbols,
            slots: &HashMap::new(),
        };
        let condition = match &program.main.instructions[0].kind {
            InstructionKind::If { condition, .. } => condition,
            other => panic!("expected an IF, got {other:?}"),
        };
        interp.eval(&code, condition)
    }

    fn eval_condition_text(interp: &mut Interp, source: &[u8]) -> Vec<u8> {
        let value = eval_condition(interp, source)
            .unwrap_or_else(|failure| panic!("expected {source:?} to evaluate, got {failure:?}"));
        interp.to_text(value).to_vec()
    }

    #[test]
    fn a_comma_list_is_an_and_of_its_parts() {
        // if 1, 1, 1 then -> true ; if 1, 0, 1 then -> false (measured
        // against the oracle's own THEN/ELSE branch taken).
        let mut interp = Interp::new(false);
        assert_eq!(
            eval_condition_text(&mut interp, b"if 1, 1, 1 then nop"),
            b"1"
        );
        assert_eq!(
            eval_condition_text(&mut interp, b"if 1, 0, 1 then nop"),
            b"0"
        );
    }

    #[test]
    fn a_comma_list_element_failure_raises_34_6_not_34_901() {
        // if 1, 'x' then -> Error 34.6, found "x" (checked left to right:
        // if 'x', 1 then also gives 34.6 on "x", the first element).
        let mut interp = Interp::new(false);
        let failure = eval_condition(&mut interp, b"if 1, 'x' then nop").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (34, 6));
        assert_eq!(raised.additional, vec!["x".to_string()]);

        let first_bad = eval_condition(&mut interp, b"if 'x', 1 then nop").unwrap_err();
        let Failure::Raised(first_bad) = first_bad else {
            panic!("expected Raised, got {first_bad:?}");
        };
        assert_eq!(first_bad.additional, vec!["x".to_string()]);
    }

    #[test]
    fn a_comma_list_short_circuits_on_the_first_false_element() {
        // The opposite of `&`'s own rule
        // (logical_operators_do_not_short_circuit, above).
        //
        // **`(1/0)` and not `'x'`, and the difference is the whole strength of
        // this test.** A skipped `'x'` only shows the *check* was skipped: a
        // literal evaluates harmlessly, so `if 0, 'x'` passes just as well
        // against an implementation that evaluates every element and then
        // stops checking. `(1/0)` cannot be evaluated without raising 42.3, so
        // it separates the two. Measured on the oracle: `if 0, (1/0) then nop`
        // and `if 1, 0, (1/0) then nop` both exit 0, while `if 1, (1/0)` exits
        // 214 with Error 42.3 -- that last one is the control, and without it
        // this test would pass against an evaluator that raised nothing ever.
        let mut interp = Interp::new(false);
        // An activation, which the `'x'` version of this test did not need:
        // division reads the frame's own `NUMERIC DIGITS`, so the control case
        // below reaches `activation()` where a literal never would.
        activate(
            &mut interp,
            parse_program(b"nop".to_vec()).expect("test program parses"),
        );
        assert_eq!(
            eval_condition_text(&mut interp, b"if 0, (1/0) then nop"),
            b"0",
            "the second element must never be evaluated, not merely left unchecked"
        );
        assert_eq!(
            eval_condition_text(&mut interp, b"if 1, 0, (1/0) then nop"),
            b"0",
            "short-circuits at the second element, never reaching the third"
        );
        let reached = eval_condition(&mut interp, b"if 1, (1/0) then nop");
        let Err(Failure::Raised(raised)) = reached else {
            panic!("an element that IS reached must raise, or the two cases above prove nothing");
        };
        assert_eq!((raised.number, raised.sub), (42, 3));
    }
}
