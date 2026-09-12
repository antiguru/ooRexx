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

use crate::activation::CallType;
use crate::error::Raised;
use crate::run::CallEntry;
use crate::run::{Ended, Resolved};
use crate::value::{canonical_small_int, exact_small_int, within_digits};
use crate::{Code, Failure, Interp, Loud, StackSpan};
use rexx_core::{Body, Decoded, INLINE_BYTES, NotNumeric, ObjRef};
use rexx_num::{CompareOp, DivOp, Number};
use rexx_parse::{CallTarget, Expr, ExprKind, Operator, PrefixOp, SymbolId};

/// D19's evaluation-depth limit: `eval`'s own recursion, one level per
/// left-deep term, refuses anything past this depth with 11.1 ("Insufficient
/// control stack space") rather than letting the interpreter thread's guard
/// page abort the process silently.
const MAX_EVAL_DEPTH: usize = 100_000;

/// Which of the three bare-symbol reads an expression is, carried where the
/// `ExprKind` itself is not.
pub(crate) const LOGICAL_TRUE: ObjRef = ObjRef::inline_byte(b'1');
/// See [`LOGICAL_TRUE`].
pub(crate) const LOGICAL_FALSE: ObjRef = ObjRef::inline_byte(b'0');

/// The value a comparison or logical operator answers with.
pub(crate) const fn logical(holds: bool) -> ObjRef {
    if holds { LOGICAL_TRUE } else { LOGICAL_FALSE }
}

/// What an arithmetic operator's left operand turned out to be.
enum ArithOperand {
    Number(Number),
    Send(ObjRef),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum SymbolRead {
    /// `ExprKind::Variable`: one slot, and the derived name when it is unset.
    Simple,
    /// `ExprKind::Stem`: one slot too, but a miss allocates a real
    /// `Body::Stem` there rather than deriving a name (`Interp::read_stem`).
    Stem,
    /// `ExprKind::Compound`: a stem slot and a tail key resolved at the read
    /// site, neither of which is the symbol's own slot -- which is why a
    /// compound is the kind a compiled read has no slot for.
    Compound,
}

impl Interp {
    /// Evaluates one expression node, and keeps the depth bookkeeping D19
    /// needs.
    pub(crate) fn enter_eval_node<T>(&mut self, anchor: *const T) -> Result<(), Failure> {
        let here = anchor as usize;

        self.depth += 1;
        if self.depth == 1 {
            self.stack_entry = here;
        }
        if self.depth > self.max_depth {
            self.max_depth = self.depth;
            self.stack_first = self.stack_entry;
            self.stack_deepest = here;
        }

        // D19's evaluation-depth limit, checked **after** the bookkeeping
        // above so a run that trips it still gets an accurate `StackSpan`
        // (the deepest level reached is this one, not the one before it).
        // `> MAX_EVAL_DEPTH`, not `>=`: a 100,000-term left-deep expression
        // reaches depth exactly 100,000 and the oracle is measured to
        // survive and print an answer at that depth
        // (`phase-4-exclusions.txt`'s Deviation 2), so the check must fire
        // one level *above* the one the oracle is known to handle, or the
        // one depth we are supposed to match becomes the first one we
        // refuse.
        if self.depth > MAX_EVAL_DEPTH {
            self.depth -= 1;
            return Err(Raised::insufficient_stack().into());
        }
        Ok(())
    }

    pub(crate) fn eval(&mut self, code: &Code<'_>, expr: &Expr) -> Result<ObjRef, Failure> {
        let probe = 0u8;
        self.enter_eval_node(&raw const probe)?;
        let value = self.eval_node(code, expr);
        self.depth -= 1;
        // `TRACE I`'s own value events (D17), and the **single insertion
        // point** for all of them: `eval` was already split from
        // `eval_node` so every exit path -- every `?`-propagated one
        // included -- passes through here, so a post-order event with the
        // value already in hand needs no threading through `eval_node`'s
        // own arms at all. Only `Ok` gets one: a raise has no value to
        // show, and the oracle's own trace never shows one for a failing
        // sub-expression either (the *clause* echo already happened, and
        // the raise's own report is what follows).
        if let Ok(v) = &value {
            self.trace_intermediate(code, expr, *v);
        }
        value
    }

    /// `>L>`/`>V>`/`>E>`/`>C>`/`>O>`/`>P>`/`>F>`, dispatched on `expr.kind`
    /// (`>A>` is not here: an argument's line belongs to the *call site*
    /// that evaluated it, not to the argument expression's own node --
    /// `Interp::invoke_call`, `run.rs`, owns it) --
    /// `eval`'s own hook, called once per node with `value` already
    /// computed. A no-op immediately when `!self.tracing_intermediates()`
    /// (`TRACE I` only; `TRACE R` never reaches any of these, measured),
    /// so the match below only ever runs its own `to_text` cost under `I`.
    #[inline]
    pub(crate) fn trace_intermediate(&mut self, code: &Code<'_>, expr: &Expr, value: ObjRef) {
        if !self.tracing_intermediates() {
            return;
        }
        self.trace_intermediate_event(code, expr, value);
    }

    /// [`Interp::trace_intermediate`]'s body, entered only under `TRACE I`.
    #[inline(never)]
    fn trace_intermediate_event(&mut self, code: &Code<'_>, expr: &Expr, value: ObjRef) {
        let indent = self.clause_state.current_value_indent;
        match &expr.kind {
            // `Constant`'s own shape is reasoned from `Literal`'s measured
            // one, not independently probed (this crate's own report says
            // so) -- both are `>L>` with no tag.
            // `echo_literal` rather than an open-coded render plus
            // `trace_literal`, because the compiled stream's own
            // `Op::TraceLiteral` emits the identical line from a register and
            // the two must not be able to disagree about it.
            ExprKind::Literal(_) | ExprKind::Constant(_) => self.echo_literal(value),
            // The three bare-symbol reads, all through the one function
            // `crate::ir::Op::TraceRead` enters from a register -- for the
            // reason `echo_literal` above is one function: a compiled read
            // emits nothing of its own, so the line has to come from an op,
            // and the two emissions must not be able to disagree.
            // `echo_symbol_read`'s own doc comment has what a compound owes
            // that this does not emit.
            ExprKind::Variable(id) | ExprKind::Stem(id) | ExprKind::Compound(id) => {
                self.echo_symbol_read(code, *id, value);
            }
            // `.NIL`/`.TRUE`/`.FALSE` -- `>E>`, measured (this task's
            // report): **not** in the design spec's own "measured reachable
            // from pure-4a code" list, a correction this task found rather
            // than assumed. The other `DotVariable` names all fail loudly
            // before reaching here (`eval_node`'s own arm), so this is
            // exhaustive over what can arrive.
            ExprKind::DotVariable(id) => {
                let tag = code.symbols.name(*id).as_bytes().to_vec();
                let text = self.string_value_text(value);
                self.trace_dotvar(indent, &tag, &text);
            }
            // `>P>`, not `>O>` -- a prefix operator's line is its own, which
            // is why this is not the arm below with a different operator type.
            // `echo_prefix_op` rather than an open-coded render plus
            // `trace_prefix_op`, because the compiled stream's own
            // `crate::ir::Op::TracePrefix` emits the identical line from a
            // register and the two must not be able to disagree -- the reason
            // `echo_literal` above is one function.
            ExprKind::Prefix { op, .. } => self.echo_prefix_op(*op, value),
            // Every binary operator, arithmetic, comparison, logical and
            // concatenation alike, traces as `>O>` -- not independently
            // probed for every family, reasoned from the arithmetic
            // transcript this task's report has (`RexxBinaryOperator`'s own
            // base `evaluate` is where the oracle's `traceOperator` call
            // sits, shared by every operator subclass the same way this one
            // `match` arm is shared here).
            // `echo_operator` rather than an open-coded render plus
            // `trace_operator`, because the compiled stream's own
            // `crate::ir::Op::TraceOperator` emits the identical line from a
            // register and the two must not be able to disagree -- the reason
            // `echo_literal` above is one function.
            ExprKind::Binary { op, .. } => self.echo_operator(*op, value),
            // `>name`/`<name` traces as an **operator**, not as the read of
            // the variable it names: `>O>   ">" => "PQ"`, the referenced
            // variable's own *name* as the operator's value
            // (`VariableReferenceOp::evaluate`, `VariableReferenceOp.cpp:110`
            // -`118`, read directly -- `getVariableReference` never reads the
            // variable's value at all, so there is no `>V>` line to go with
            // it). Measured at both spellings and in both positions: `say
            // >pq`, `say <pq`, `call sub >pq` and `call sub <pq` all trace
            // `>O>   ">" => "PQ"`, the `<` form included -- the oracle's own
            // call passes the literal `">"` regardless of which byte was
            // written.
            ExprKind::VariableReference(inner) => {
                let (ExprKind::Variable(id) | ExprKind::Stem(id)) = &inner.kind else {
                    // `rexx-parse` admits nothing else inside a reference
                    // (20.930 at parse time), and `eval_node`'s own arm
                    // fails loudly on anything that arrives regardless, so
                    // there is no value here to have traced.
                    return;
                };
                let name = code.symbols.name(*id).as_bytes().to_vec();
                self.trace_operator(indent, b">", &name);
            }
            // `>F>`, the **function form only** -- `trace_function`'s own
            // doc comment has the measured pair that separates it from
            // `CALL`. The tag is the name as the call site spells it: a
            // symbol target arrives already upcased and a literal one
            // verbatim (`rexx-parse`'s own `CallTarget`), which is the same
            // pair of spellings `eval_call` resolves against.
            ExprKind::Call { target, .. } => {
                let name = match target {
                    CallTarget::Symbol(id) => code.symbols.name(*id).as_bytes().to_vec(),
                    CallTarget::Literal(bytes) => bytes.to_vec(),
                };
                let text = self.string_value_text(value);
                self.trace_function(indent, &name, &text);
            }
            // `>F>` too, tagged with the routine name **alone** -- measured,
            // `trace i` over `say w:wroutine()` traces
            // `>F>   WROUTINE => "wroutine ran"`, with no qualifier on the
            // tag, which is why this shares the arm above's emitter rather
            // than `>N>`'s.
            ExprKind::QualifiedCall { name, .. } => {
                let name = code.symbols.name(*name).as_bytes().to_vec();
                let text = self.string_value_text(value);
                self.trace_function(indent, &name, &text);
            }
            // `>N>`, tagged `namespace:class` -- `traceClassResolution`
            // (`RexxActivation.hpp:358`), whose tag is the two symbols joined
            // by a colon.
            ExprKind::ClassResolver { namespace, name } => {
                let mut tag = code.symbols.name(*namespace).as_bytes().to_vec();
                tag.push(b':');
                tag.extend_from_slice(code.symbols.name(*name).as_bytes());
                let text = self.string_value_text(value);
                self.trace_namespace(indent, &tag, &text);
            }
            // **`>M>` is not here**, and that is the one value prefix this
            // hook does not own: the message-assignment form is an
            // instruction that never evaluates its term as an expression, so
            // a hook arm would print the line for one of the two forms only.
            // `Interp::message_term` (`dispatch.rs`) emits it instead, at the
            // same point in the sequence this hook would have.
            ExprKind::Message { .. } => {}
            // A comma list (`ExprKind::Logical`) has no value line of its
            // own -- each element is a full `eval` call in its own right
            // (`eval_logical_list`), and traces itself through this same
            // function when it runs. Every other unimplemented `ExprKind`
            // never reaches here at all (`eval_node`'s own loud fallback).
            _ => {}
        }
    }

    /// One bare symbol's read: the whole of what `eval_node`'s own
    /// `Variable`/`Stem`/`Compound` arms do, entered from there and from
    /// `crate::ir::Op::Load`.
    pub(crate) fn read_symbol(
        &mut self,
        code: &Code<'_>,
        read: SymbolRead,
        id: SymbolId,
        at: Option<usize>,
    ) -> Result<ObjRef, Failure> {
        match read {
            SymbolRead::Simple => {
                let (value, novalue) = self.read_at(code, id, at);
                self.novalue_check(novalue, value)?;
                Ok(value)
            }
            SymbolRead::Stem => Ok(self.read_stem_at(code.symbols.name(id).as_bytes(), at)),
            // `id` names the *whole* compound (its interned spelling is the
            // full dotted text); the split decomposes it into the stem's own
            // name and the tail pieces `tail_key` (`stem.rs`) resolves into
            // the one key `stem_get` looks up.
            SymbolRead::Compound => {
                debug_assert!(
                    at.is_none(),
                    "a compound read was handed a slot, and the slot it reads is the stem's"
                );
                let (stem_name, stem_at) = code.stem(id);
                // Borrowed and handed back, rather than a fresh key per read
                // -- see `Interp::key_buffer`. The `?` below is why the return
                // is placed before it: an early exit there would lose the
                // buffer, which is safe but forfeits the reuse.
                let mut key = self.take_key_buffer();
                if let Err(failure) = self.tail_key_into(code, id, &mut key) {
                    self.give_key_buffer(key);
                    return Err(failure);
                }
                // `>C>`, here rather than in `echo_symbol_read_line`, so that
                // the tail is resolved once per reference -- that function's
                // own doc comment has the reason.
                if self.tracing_intermediates() {
                    let tag = code.symbols.name(id).as_bytes().to_vec();
                    let mut printed = stem_name.to_vec();
                    printed.extend_from_slice(&key);
                    let indent = self.clause_state.current_value_indent;
                    self.trace_compound_name(indent, &tag, &printed);
                }
                let (value, novalue) = self.stem_get_at(stem_name, stem_at, &key);
                self.give_key_buffer(key);
                self.novalue_check(novalue, value)?;
                Ok(value)
            }
        }
    }

    /// The `>V>` line one bare-symbol read owes.
    #[inline(always)]
    pub(crate) fn echo_symbol_read(&mut self, code: &Code<'_>, id: SymbolId, value: ObjRef) {
        if !self.tracing_intermediates() {
            return;
        }
        self.echo_symbol_read_line(code, id, value);
    }

    /// The name-building and the lines, out of line behind
    /// [`Interp::echo_symbol_read`]'s gate.
    #[inline(never)]
    fn echo_symbol_read_line(&mut self, code: &Code<'_>, id: SymbolId, value: ObjRef) {
        let indent = self.clause_state.current_value_indent;
        let tag = code.symbols.name(id).as_bytes().to_vec();
        let text = self.string_value_text(value);
        self.trace_variable(indent, &tag, &text);
    }

    pub(crate) fn eval_node(&mut self, code: &Code<'_>, expr: &Expr) -> Result<ObjRef, Failure> {
        match &expr.kind {
            ExprKind::Literal(bytes) => Ok(self.literal(bytes)),
            // A constant's value is its own upcased spelling, which is
            // observable rather than incidental: `say 1e5` prints `1E5`.
            ExprKind::Constant(id) => Ok(self.literal(code.symbols.name(*id).as_bytes())),
            // The three bare-symbol reads, each through the one function
            // `crate::ir::Op::Load` enters with the slot already resolved.
            // They are three *different* operations sharing one entry point,
            // not one operation under three spellings -- `read_symbol`'s own
            // doc comment has what separates them -- so the dispatch is on a
            // kind rather than absent.
            ExprKind::Variable(id) => self.read_symbol(code, SymbolRead::Simple, *id, None),
            ExprKind::Stem(id) => self.read_symbol(code, SymbolRead::Stem, *id, None),
            ExprKind::Compound(id) => self.read_symbol(code, SymbolRead::Compound, *id, None),

            // **The parser's own names never reach resolution and every
            // other one does.** `LanguageParser`'s constructor installs a
            // `SpecialDotVariable` retriever for `.NIL`, `.TRUE` and `.FALSE`
            // (`parser/LanguageParser.cpp:781`-`783`), so those three are
            // parse-time constants in the oracle too. Measured: with `::class
            // True` in the file, `say .TRUE` still prints `1` where `say
            // value('.TRUE')` prints `The TRUE class`, because only the
            // second goes through `getVariableRetriever`.
            ExprKind::DotVariable(id) => match code.symbols.name(*id) {
                ".NIL" => Ok(ObjRef::NIL),
                // `.true`/`.false` need no representation of their own
                // (D15): they are the one-byte strings "1" and "0", built
                // fresh here the same way any other text value is.
                ".TRUE" => Ok(LOGICAL_TRUE),
                ".FALSE" => Ok(self.text(b"0")),
                other => {
                    let name = other.as_bytes().to_vec();
                    self.dot_variable(&name)
                }
            },

            ExprKind::Prefix { op, operand } => self.eval_prefix(code, *op, operand),

            // Arithmetic (D15's "Expression evaluation": `+ - * / % // **`,
            // through `rexx-num` under the settings in force *now* -- the
            // rendering of the result is fixed at creation, D15's own rule,
            // but the DIGITS/FORM an operation computes *under* is always
            // the activation's current ones, read fresh on every call. Ahead
            // of the arm below, which is every *other* binary operator, so
            // that arithmetic reaches `eval_arithmetic`: `**`'s exponent is
            // not converted the way its base is, and that asymmetry has
            // nowhere to live in a shared operand path.
            ExprKind::Binary { op, left, right } if is_arithmetic(*op) => {
                self.eval_arithmetic(code, *op, left, right)
            }

            // Concatenation, comparison and `&`/`|`/`&&`
            // (D15's "Expression evaluation"), which share this operand
            // prologue exactly: **both operands are always evaluated, left
            // then right, and never short-circuited** -- measured, `say (0 &
            // 'x')` raises 34.901 on `"x"` though the result is already
            // decided. What each operator then does with two values is
            // `apply_binary`, entered from here and from
            // `crate::ir::Op::Binary`.
            ExprKind::Binary { op, left, right }
                if is_native_binary(*op) && !is_arithmetic(*op) =>
            {
                let frame = self.roots.push_frame();
                let left_value = self.eval(code, left)?;
                self.roots.push_temp(left_value);
                let right_value = self.eval(code, right)?;
                self.roots.push_temp(right_value);
                // Bound rather than propagated with `?`, so the frame is
                // popped on the failure path too -- an operator's own raise
                // discards its operands here, where the `?` on an operand's
                // evaluation above leaves them for `Op::Clause`'s region to
                // truncate. The module doc has why both are safe.
                let result = self.apply_binary(*op, left_value, right_value);
                self.roots.pop_frame(frame);
                result
            }

            // The comma-separated conditional list (`IF a, b THEN`, and
            // `WHEN`/`GUARD`/`WHILE`/`UNTIL`'s own versions of the same
            // syntax) -- see `eval_logical_list`'s own doc comment for the
            // short-circuit and sub-number this arm alone can give it.
            ExprKind::Logical(items) => self.eval_logical_list(code, items),

            // `>name`/`<name` answers a `VariableReference`, the variable
            // itself rather than its value -- measured, oracle rc 0:
            // `vr = 5; o = >vr; say o~class~id` is `VariableReference`, and
            // `o~value = 7` writes `vr`.
            ExprKind::VariableReference(inner) => self.variable_reference(code, inner),

            // `f(...)`/`"f"(...)` (Task 4, 4b) -- see `eval_call`'s own doc.
            ExprKind::Call { target, args } => self.eval_call(code, target, args),

            // `target~name(...)`, `target~~name(...)` and `target[...]`
            // (Phase 5a) -- see `Interp::message_term` (`dispatch.rs`) for
            // the evaluation order and `Interp::resolve`/`Interp::invoke`
            // for the send itself. The **expression** form only: the
            // message-assignment form is an instruction and never an
            // expression, so `assigned` is `None` here.
            ExprKind::Message {
                target,
                name,
                super_class,
                args,
                cascade,
            } => self
                .message_term(
                    code,
                    &crate::dispatch::MessageTerm {
                        target,
                        name,
                        super_class: super_class.as_deref(),
                        args,
                        cascade: *cascade,
                        assigned: None,
                    },
                )?
                .ok_or_else(|| Raised::no_result(name).into()),

            _ => self.eval_cold(code, expr),
        }
    }

    /// Every expression form the match above does not name: `(a, b, ...)`, and
    /// the ones that fail loudly.
    #[inline(never)]
    fn eval_cold(&mut self, code: &Code<'_>, expr: &Expr) -> Result<ObjRef, Failure> {
        match &expr.kind {
            // `RexxExpressionList::evaluate` (`expression/ExpressionList.cpp:83`),
            // which builds an array of the list's own length and fills the
            // positions that were written.
            ExprKind::List(items) => self.eval_list(code, items),
            // `ns:Name`, the qualified class lookup
            // (`ClassResolver::evaluate`,
            // `expression/ExpressionClassResolver.cpp:123`). The `>N>` line
            // it owes is this hook's sibling below, in `trace_intermediate`.
            ExprKind::ClassResolver { namespace, name } => {
                let namespace = code.symbols.name(*namespace).as_bytes().to_vec();
                let name = code.symbols.name(*name).as_bytes().to_vec();
                let package = self.running_program().ok_or_else(Loud::missing_body)?;
                self.namespace_class(package, &namespace, &name)
            }
            // `ns:name(...)`, resolved against that namespace's public
            // routines alone -- so none of `resolve_call`'s four steps runs,
            // and a builtin of the same name is never reached. Measured,
            // `rexx:length('abc')` is 43.902 rather than `3`.
            ExprKind::QualifiedCall {
                namespace,
                name,
                args,
            } => {
                let namespace = code.symbols.name(*namespace).as_bytes().to_vec();
                let name = code.symbols.name(*name).as_bytes().to_vec();
                let package = self.running_program().ok_or_else(Loud::missing_body)?;
                let resolution = self
                    .namespace_routine(package, &namespace, &name)
                    .map(Resolved::Routine);
                let resolved = self.resolved_after_arguments(code, resolution, args)?;
                self.eval_call_resolved(code, resolved, &name, args)
            }
            other => Err(Loud::expression(other).into()),
        }
    }

    /// `ExprKind::List`: a fresh `.Array` whose slots are the list's
    /// positions.
    #[inline(never)]
    fn eval_list(&mut self, code: &Code<'_>, items: &[Option<Expr>]) -> Result<ObjRef, Failure> {
        let frame = self.roots.push_frame();
        let indent = self.clause_state.current_value_indent;
        let mut slots = Vec::with_capacity(items.len());
        for item in items {
            let Some(expr) = item else {
                slots.push(None);
                continue;
            };
            let value = self.eval(code, expr)?;
            self.roots.push_temp(value);
            if let Some(rendered) = self.intermediate_text(value) {
                self.trace_argument(indent, &rendered);
            }
            slots.push(Some(value));
        }
        let array = self.alloc_with(rexx_core::BehaviourId::ARRAY, Body::array(slots));
        if let Some(rendered) = self.result_text(array) {
            self.trace_result(indent, &rendered);
        }
        self.roots.pop_frame(frame);
        Ok(array)
    }

    /// `ExprKind::Call`: the internal-function form, evaluated for its
    /// value rather than run as a clause of its own.
    fn eval_call(
        &mut self,
        code: &Code<'_>,
        target: &CallTarget,
        args: &[Option<Expr>],
    ) -> Result<ObjRef, Failure> {
        let (name, search_labels) = call_target_name(code, target);
        // Held until the arguments have run -- `Interp::resolved_after_
        // arguments` has the C++ citation and the two measurements.
        let resolution = self.resolve_call(name, search_labels);
        let resolved = self.resolved_after_arguments(code, resolution, args)?;
        self.eval_call_resolved(code, resolved, name, args)
    }

    /// [`eval_call`]'s second half: everything after the resolution.
    pub(crate) fn eval_call_resolved(
        &mut self,
        code: &Code<'_>,
        resolved: Resolved,
        name: &[u8],
        args: &[Option<Expr>],
    ) -> Result<ObjRef, Failure> {
        // **A builtin goes straight to its own entry point**, which is the
        // same call `invoke_call` would make and answers the value this
        // function wants: none of the three arms below can apply to it, since
        // it runs no activation and so can neither exit nor return nothing.
        // What the detour cost is the `Ended` -- wider than a register pair,
        // so built in memory here and read back out one line later, on the
        // path every `length(...)`/`substr(...)` in a program takes.
        if let Resolved::Builtin(target) = resolved {
            return self.invoke_builtin_call(code, target, name, args);
        }
        // `CallType::Function`: this is the function-invocation route, and a
        // `::ROUTINE` reached this way answers `FUNCTION` as `PARSE SOURCE`'s
        // second word where the same body reached by `CALL` answers
        // `SUBROUTINE`. Measured in one program, the same routine both ways.
        match self.invoke_call(
            code,
            resolved,
            name,
            args,
            CallType::Function,
            CallEntry::Written,
        )? {
            // `EXIT` inside the routine, or the routine falling off its own
            // end, ends the whole program exactly as it does when the same
            // routine is reached through `CALL` (`Interp::invoke_call`'s
            // own doc, `run.rs`). Propagated as `Failure::Exited` because
            // `eval`'s own return type is a plain `ObjRef` with no `Flow` to
            // carry the event through instead -- see that variant's own doc
            // (`error.rs`) for why every intervening `?` needs no special
            // handling to still unwind every nested `CALL` correctly.
            Ended::Exited(value) => Err(Failure::Exited(value)),
            Ended::Returned(Some(value)) => Ok(value),
            // Measured on the oracle: a routine reached through the
            // expression form and returning nothing (a bare `RETURN`) is
            // Error 44.1 rc 212, "No data returned from function "NAME""
            // -- the expression form's own answer to "nothing to use here",
            // which `CALL` never has to give since its own value only ever
            // reaches `RESULT`, unset or not.
            Ended::Returned(None) => Err(Raised::no_data_returned(name).into()),
        }
    }

    /// `+`/`-`/`\` (D15's "Expression evaluation"), over an operand this
    /// evaluates out of the tree.
    fn eval_prefix(
        &mut self,
        code: &Code<'_>,
        op: PrefixOp,
        operand: &Expr,
    ) -> Result<ObjRef, Failure> {
        let frame = self.roots.push_frame();
        let value = self.eval(code, operand)?;
        self.roots.push_temp(value);
        // Bound rather than propagated with `?`, so the frame is popped on the
        // failure path too -- the shape `eval_node`'s own binary arm has, and
        // the module doc has why both it and the `?` above are safe.
        let result = self.apply_prefix(op, value);
        self.roots.pop_frame(frame);
        result
    }

    /// `op value` for the prefix operators `+`, `-` and `\`.
    pub(crate) fn apply_prefix(&mut self, op: PrefixOp, value: ObjRef) -> Result<ObjRef, Failure> {
        let result = self.apply_prefix_body(op, value);
        // Blamed on any failure, the same reason `Interp::arith_general`
        // gives: measured, `s. = 'abc'; say \s.` is 34.901 with the frame,
        // even though `\` never converts its operand to a number at all --
        // the frame belongs to the forwarded activation, not to which of
        // its checks raised.
        if result.is_err() {
            self.blame_stem_forwarded_operator(op.spelling().as_bytes(), value);
        }
        result
    }

    /// [`Interp::apply_prefix`]'s own computation, wrapped by it for the
    /// reason [`Interp::arith_general_body`] is.
    fn apply_prefix_body(&mut self, op: PrefixOp, value: ObjRef) -> Result<ObjRef, Failure> {
        let result = match op {
            PrefixOp::Plus | PrefixOp::Minus => {
                let number = match self.arith_left_operand(op.spelling(), value)? {
                    ArithOperand::Number(number) => number,
                    ArithOperand::Send(target) => {
                        return self.send_operator(op.spelling(), target, &[]);
                    }
                };
                self.lostdigits_check(&number, value)?;
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
                // `\.array` is 97.1 on the oracle, the same message send the
                // dyadic operators make, and it is asked ahead of the truth
                // test for the reason [`Interp::logical_values_body`] states.
                if let Some(target) = self.operator_message_receiver(value) {
                    return self.send_operator(op.spelling(), target, &[]);
                }
                if let Some(kind) = self.operator_operand_gap(value) {
                    return Err(Loud::operator_operand(op.spelling(), kind).into());
                }
                let text = self.to_text(value).to_vec();
                let flipped = match logical_value(&text) {
                    Some(true) => b"0",
                    Some(false) => b"1",
                    None => return Err(Raised::not_logical(&text).into()),
                };
                self.text(flipped)
            }
        };
        Ok(result)
    }

    /// The seven arithmetic operators, sharing one operand-evaluation and
    /// error-conversion path.
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

        // The two halves in the order they are tried, which is the whole of
        // what this function decides once its operands are values. Both are
        // entered from `crate::ir::Op::Arith` as well, which is why they are
        // functions rather than the two blocks they used to be: the compiled
        // op has its operands in registers rather than in nodes, and
        // everything past that point is the same arithmetic.
        let value = match self.arith_small_int(op, left_value, right_value) {
            Some(value) => value,
            None => self.arith_general(op, left_value, right_value)?,
        };

        self.roots.pop_frame(frame);
        Ok(value)
    }

    /// `left op right` on the small-integer path, or `None` when the general
    /// path must run instead.
    pub(crate) fn arith_small_int(
        &self,
        op: Operator,
        left_value: ObjRef,
        right_value: ObjRef,
    ) -> Option<ObjRef> {
        let digits = self.activation().settings.digits();
        // **The tagged pair is tested first and on its own**, exactly as it
        // was before the spelled operand was admitted. Reading a spelled one
        // costs a scan of its bytes, and a clause whose operands are both
        // already tagged must not pay for a widening it cannot use.
        match (left_value.decode(), right_value.decode()) {
            (Decoded::SmallInt(left_int), Decoded::SmallInt(right_int)) => {
                small_int_arith(op, left_int, right_int, digits)
            }
            _ => spelled_int_arith(op, left_value, right_value, digits),
        }
    }

    /// `left op right` through `rexx-num`, the path every operand shape
    /// reaches and the one [`Interp::arith_small_int`] falls through to.
    pub(crate) fn arith_general(
        &mut self,
        op: Operator,
        left_value: ObjRef,
        right_value: ObjRef,
    ) -> Result<ObjRef, Failure> {
        let result = self.arith_general_body(op, left_value, right_value);
        // **Blamed on any failure past this point, not only the left
        // operand's own conversion.** The oracle's forwarded native method
        // runs to completion or fails; whichever of its own steps raises --
        // the base's conversion, the argument's, or the arithmetic itself --
        // is still inside that one activation, each of them past
        // `arith_left_operand`'s own return. Measured: `s. = 1; say s. /
        // 0` is 42.3 with the frame, `s. = 1; say s. ** 999999999999` is
        // 26.8 with the frame (the base converts fine; the *exponent*
        // fails), and `s. = 1; say s. + .array` is 41.1 with the frame even
        // though it is the *right* operand's own conversion that fails.
        // The receiver test is `blame_stem_forwarded_operator`'s own, so
        // this site gates on failure alone and hands it every failing path
        // here, leaving a non-stem receiver for that predicate to discard.
        if result.is_err() {
            self.blame_stem_forwarded_operator(op.spelling().as_bytes(), left_value);
        }
        result
    }

    /// [`Interp::arith_general`]'s own computation, wrapped by it so every
    /// failing path -- not only [`Interp::arith_left_operand`]'s -- is
    /// blamed once, in the one place that already knows the receiver.
    fn arith_general_body(
        &mut self,
        op: Operator,
        left_value: ObjRef,
        right_value: ObjRef,
    ) -> Result<ObjRef, Failure> {
        let digits = self.activation().settings.digits();
        let form = self.activation().settings.form();

        let left_number = match self.arith_left_operand(op.spelling(), left_value)? {
            ArithOperand::Number(number) => number,
            ArithOperand::Send(target) => {
                return self.send_operator(op.spelling(), target, &[Some(right_value)]);
            }
        };

        // The argument of the operator method the receiver answers, converted
        // after the receiver itself is accepted -- `apply_binary`'s own doc
        // has the citations, and `StringClass::arith` reports **`otherObj`**
        // rather than the conversion when the converted string is not
        // numeric. Measured, rc 215: `'2' + .K` with a class-side
        // `makeString` returning `'xx'` is 41.1 `Nonnumeric value ("The K
        // class")`.
        let converted = self.required_string_value(right_value)?;

        let result = if op == Operator::Power {
            let exponent = match self.to_number(converted) {
                Ok(number) => number,
                Err(NotNumeric) => {
                    let text = self.string_value_text(right_value);
                    return Err(Raised::power_exponent_not_whole(&text).into());
                }
            };
            // The base alone: measured, `1.23456789 ** 1` at DIGITS 3 is
            // 98.972 on the base and `2 ** 1234` is 26.8 on the *exponent*'s
            // own whole-number conversion, which runs first and never
            // reaches LOSTDIGITS.
            self.lostdigits_check(&left_number, left_value)?;
            left_number.pow(&exponent, digits)
        } else {
            let right_number = match self.to_number(converted) {
                Ok(number) => number,
                Err(NotNumeric) => {
                    let text = self.string_value_text(right_value);
                    return Err(Raised::nonnumeric(&text).into());
                }
            };
            // Left first, which is the oracle's order: measured, `987654321 +
            // 123456789` at DIGITS 3 names 987654321.
            self.lostdigits_check2(&left_number, left_value, &right_number, right_value)?;
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

        Ok(self.number(result, saturate_digits(digits), form))
    }

    /// Converts an arithmetic operand to a `Number`, or 41.1 with the
    /// operand's own rendered text as the substitution -- measured, `say
    /// 'abc' + 1` reports `Nonnumeric value ("abc")`, the operand as it
    /// renders, not upcased or otherwise transformed.
    pub(crate) fn arith_operand(&mut self, value: ObjRef) -> Result<Number, Failure> {
        match self.to_number(value) {
            Ok(number) => Ok(number),
            Err(NotNumeric) => {
                let text = self.string_value_text(value);
                Err(Raised::nonnumeric(&text).into())
            }
        }
    }

    /// [`Interp::arith_operand`] for the operand the operator is *sent to*.
    fn arith_left_operand(&mut self, op: &str, value: ObjRef) -> Result<ArithOperand, Failure> {
        match self.to_number(value) {
            Ok(number) => Ok(ArithOperand::Number(number)),
            Err(NotNumeric) => {
                if let Some(target) = self.operator_message_receiver(value) {
                    return Ok(ArithOperand::Send(target));
                }
                if let Some(kind) = self.operator_operand_gap(value) {
                    return Err(Loud::operator_operand(op, kind).into());
                }
                let text = self.string_value_text(value);
                Err(Raised::nonnumeric(&text).into())
            }
        }
    }

    /// Renders the operator-forwarded native-method frame for `op` when
    /// `value` is a stem whose forward reaches a method that actually runs
    /// -- a no-op otherwise.
    pub(crate) fn blame_stem_forwarded_operator(&mut self, op: &[u8], value: ObjRef) {
        if !self.is_stem_receiver(value) {
            return;
        }
        let scope = self.string_class();
        let scope_id = self.classes().id_string(scope).to_string();
        self.blame_native_method(op, &scope_id);
    }

    /// Whether `value` is a stem whose forwarded operator would actually
    /// reach a native method to run: true for an unset stem (its own name
    /// is a `String`) and for one whose default is itself `String`/
    /// `Number`-valued; false for `.nil` and for anything
    /// [`Interp::operator_operand_gap`] already named.
    fn is_stem_receiver(&self, value: ObjRef) -> bool {
        let Decoded::Heap { .. } = value.decode() else {
            return false;
        };
        match self.heap.get(value).map(|object| &object.body) {
            Some(Body::Stem { default: None, .. }) => true,
            Some(Body::Stem {
                default: Some(default),
                ..
            }) => self.stem_default_is_string_or_number(*default),
            _ => false,
        }
    }

    /// Whether `value` is `String`/`Number`-valued, chasing a nested stem's
    /// own default the way [`Interp::to_number`] and
    /// [`Interp::operator_operand_gap`] both do. `.nil` and a class object
    /// answer no arithmetic operator on the oracle, so a forward landing on
    /// either never reaches a method to raise from.
    fn stem_default_is_string_or_number(&self, value: ObjRef) -> bool {
        match value.decode() {
            Decoded::Nil => false,
            Decoded::SmallInt(_) | Decoded::Text(_) => true,
            Decoded::Heap { .. } => match self.heap.get(value).map(|object| &object.body) {
                Some(Body::Class { .. }) => false,
                Some(Body::Text { .. }) | Some(Body::Num { .. }) => true,
                Some(Body::Stem { default: None, .. }) => true,
                Some(Body::Stem {
                    default: Some(default),
                    ..
                }) => self.stem_default_is_string_or_number(*default),
                _ => false,
            },
        }
    }

    /// The shared body of `||`/`Abuttal` (no separator) and `Blank` (one
    /// space).
    fn concat_values(
        &mut self,
        left_value: ObjRef,
        right_value: ObjRef,
        separator: Option<u8>,
    ) -> Result<ObjRef, Failure> {
        // Both operands' bytes are read through shared borrows, which can be
        // live at once -- and that is the whole change here. The left operand
        // used to be copied into an owned buffer for no reason except that
        // reading the right one needed its own `&mut`, so a two-operand join
        // allocated three times: the copy, the buffer it grew into, and the
        // result. It allocates once now, into a buffer sized before anything
        // is written and handed straight to the value.
        let left_rendered = self.render(left_value);
        let right_rendered = self.render(right_value);
        let left_bytes = left_rendered.text(self);
        let right_bytes = right_rendered.text(self);
        // **A join that fits the object's own inline bytes is built on the
        // stack and never touches the lent buffer at all.** `text_built` ends
        // such a join by copying it into the value and handing the buffer
        // straight back, so everything the `Vec` did for it -- the take, the
        // reservation, the length bookkeeping, the return -- was setup for a
        // container the result does not end up in. The bound is the one
        // `text_built` branches on, so this arm and its own are the same
        // arm -- what differs is that the buffer is never taken.
        let total = left_bytes.len() + usize::from(separator.is_some()) + right_bytes.len();
        if total <= INLINE_BYTES {
            let mut buffer = [0u8; INLINE_BYTES];
            let (head, rest) = buffer.split_at_mut(left_bytes.len());
            head.copy_from_slice(left_bytes);
            let rest = match separator {
                Some(byte) => {
                    let (slot, rest) = rest.split_at_mut(1);
                    slot[0] = byte;
                    rest
                }
                None => rest,
            };
            rest[..right_bytes.len()].copy_from_slice(right_bytes);
            return Ok(self.text(&buffer[..total]));
        }
        // **Built in the lent buffer and finished with `text_built`**, which
        // is what makes the one remaining allocation conditional rather than
        // certain. Only a result too long to live inline reaches here, and it
        // keeps the buffer, which is the same trade `text_built` always makes.
        // Measured with `heaptrack` on `bench-programs/strings.rex`, where
        // this was one of the two allocations left per iteration and 99.6% of
        // what that program allocated was freed with nothing allocated in
        // between.
        let mut bytes = self.take_result_buffer();
        bytes.reserve(left_bytes.len() + usize::from(separator.is_some()) + right_bytes.len());
        bytes.extend_from_slice(left_bytes);
        if let Some(byte) = separator {
            bytes.push(byte);
        }
        bytes.extend_from_slice(right_bytes);
        let joined = self.text_built(bytes);

        Ok(joined)
    }

    /// The comparison operators (D15's "Expression evaluation":
    /// numeric-or-string `= \= <> >< > < >= <= \> \<`, and strict
    /// `== \== >> << >>= <<= \>> \<<`), all through `rexx-num`'s
    /// `compare_numbers` and `compare_strings` -- **no comparison rule is
    /// written here**, per the plan's own instruction and this crate's
    /// standing rule against a second copy of logic `rexx-num` already owns
    /// (the same rule `compare.rs`'s own module doc states for its entry
    /// points). Those two are `compare_decoded`'s own arms, called directly
    /// because this function has already decided which arm applies: each
    /// arm's own doc comment says so, and says what a caller buys by
    /// choosing.
    fn compare_values(
        &mut self,
        op: Operator,
        left_value: ObjRef,
        right_value: ObjRef,
    ) -> Result<ObjRef, Failure> {
        let result = self.compare_values_body(op, left_value, right_value);
        // Blamed on any failure, the same reason `Interp::arith_general`
        // gives -- and comparison does have one, corrected from this task's
        // own earlier premise: `compare_numbers` converts both operands to
        // a `Number` and can overflow when both are numeric, exactly as an
        // arithmetic operator's operands can. Measured: `numeric digits 1;
        // s. = '9.9E999999999'; say s. > 1` is rc 214 with the frame; the
        // same overflow with `1 > s.` (the receiver `1`, not a stem) is the
        // identical rc 214 with no frame, because `is_stem_receiver`
        // answers `false` for the plain-number receiver regardless of which
        // operand overflowed. Strict `==`/`>>` and the non-numeric fallback
        // (`compare_strings`) never reach `compare_numbers` at all, so they
        // stay rc 0 and frameless, unaffected by this wrapping.
        if result.is_err() {
            self.blame_stem_forwarded_operator(op.spelling().as_bytes(), left_value);
        }
        result
    }

    /// [`Interp::compare_values`]'s own computation, wrapped by it for the
    /// reason [`Interp::arith_general_body`] is.
    fn compare_values_body(
        &mut self,
        op: Operator,
        left_value: ObjRef,
        right_value: ObjRef,
    ) -> Result<ObjRef, Failure> {
        // Every `&mut` read this comparison needs happens first -- the two
        // parses, and the two settings -- so that the byte reads below can be
        // shared borrows taken together. `to_number` hands back an owned
        // `Number`, so nothing here outlives its own statement, and the
        // operand copies the old order forced are gone.
        let digits = self.activation().settings.digits();
        let fuzz = self.activation().settings.fuzz();
        // **Two tagged small integers answer this comparison themselves**,
        // where the general path renders both operands to text first --
        // measured with `heaptrack` on `samples/rexxcps.rex`, that rendering
        // was the largest single allocation width in the interpreter, at
        // 4,220,217 allocations of nineteen bytes on a 400,000-clause run
        // (`SMALL_INT_MAX` has nineteen digits, which is what pre-sizes them).
        if fuzz == 0
            && let Some(holds) = small_int_compare(op, left_value, right_value, digits)
        {
            return Ok(logical(holds));
        }
        let strict = is_strict_compare(op);
        // **The `Result` is kept rather than turned into an `Option`.** A
        // `Number` is forty bytes and `Result::ok` moves one, so the two calls
        // here moved eighty bytes for nothing: every read below is `is_ok`,
        // `is_err` or a borrow out of the `Ok` arm, and none of them needs an
        // `Option`. Measured on `samples/rexxcps.rex`, -0.531% of its retired
        // instructions and -0.60% of its cycles over six alternating runs --
        // the first pair of runs read -5%, which was the cold start and not
        // this.
        let left_number = if strict {
            Err(NotNumeric)
        } else {
            self.to_number(left_value)
        };
        // **Behind the fast path above, because a comparison of renderings
        // never fails and so offers nothing to ride.** Measured, comparing an
        // object against the very text it renders as answers `0` on the
        // oracle and `1` here; see `Loud::operator_operand`.
        debug_assert!(
            left_number.is_err() || self.operator_operand_gap(left_value).is_none(),
            "a left operand that parsed as a number reported an operator gap"
        );
        if left_number.is_err()
            && let Some(kind) = self.operator_operand_gap(left_value)
        {
            return Err(Loud::operator_operand(op.spelling(), kind).into());
        }
        let right_number = if strict {
            Err(NotNumeric)
        } else {
            self.to_number(right_value)
        };

        // **Both operands parsed means the bytes have no reader**, so they are
        // not produced. `compare_decoded`'s own `(Some, Some)` arm is
        // `compare_numbers` and nothing else, and it is handed the same two
        // values this would have passed it -- so this is where the rendering
        // happens rather than what the comparison answers. What it skips is
        // `render` plus `text` on both sides, whose result reaches only the
        // string fallback below.
        if let (Ok(left), Ok(right)) = (&left_number, &right_number) {
            // The **numeric** arm only: measured, `1.23456789 = 'abc'` at
            // DIGITS 3 is rc 0 on the oracle, because a comparison that falls
            // through to the string rule converts no operand at all. A strict
            // operator never reaches here either.
            self.lostdigits_check2(left, left_value, right, right_value)?;
            let holds = rexx_num::compare_numbers(left, right, digits, fuzz, compare_op(op))
                .map_err(Raised::from)?;
            return Ok(logical(holds));
        }
        // Either operand failed to parse, or the operator is strict and neither
        // was parsed at all. Both routes compare the operands' own text, which
        // is what these two renderings are for.
        let left_rendered = self.render(left_value);
        let right_rendered = self.render(right_value);
        let left_bytes = left_rendered.text(self);
        let right_bytes = right_rendered.text(self);
        // The claim the line below rests on, checked rather than argued: an
        // operand `to_number` refused is one whose own rendering does not
        // parse either, so the arm `compare_decoded` would have reached after
        // reparsing is the arm taken here. A strict operator is exempt because
        // it consults no `Number` at all.
        debug_assert!(
            strict || left_number.is_ok() || Number::parse_bytes(left_bytes).is_none(),
            "a left operand to_number refused parses from its own rendering"
        );
        debug_assert!(
            strict || right_number.is_ok() || Number::parse_bytes(right_bytes).is_none(),
            "a right operand to_number refused parses from its own rendering"
        );
        let holds = rexx_num::compare_strings(left_bytes, right_bytes, compare_op(op));

        let result = logical(holds);
        Ok(result)
    }

    /// `&`/`|`/`&&`. **Both operands are always evaluated and checked,
    /// never short-circuited**, and the left is checked before the right
    /// when both are bad -- measured: `say (0 & 'x')` and `say (1 | 'x')`
    /// both raise 34.901 on `"x"` even though the result is already
    /// decided by the first operand alone, and `say ('y' & 'x')` (both
    /// bad) reports `"y"`, the left operand's own text, not `"x"`. The
    /// opposite rule from `ExprKind::Logical`'s comma list, below, which
    /// does short-circuit -- both are implemented exactly as measured rather
    /// than made to agree with each other. The evaluation half of that rule
    /// belongs to whoever calls this: the check half is here, and it checks
    /// the operand it was handed second even when the first already decided
    /// the answer.
    fn logical_values(
        &mut self,
        op: Operator,
        left_value: ObjRef,
        right_value: ObjRef,
    ) -> Result<ObjRef, Failure> {
        let result = self.logical_values_body(op, left_value, right_value);
        // Blamed on any failure, the same reason `Interp::arith_general`
        // gives: measured, `s. = 1; say s. & 'x'` is 34.901 with the frame
        // even though it is the *right* operand's own text that fails the
        // logical check.
        if result.is_err() {
            self.blame_stem_forwarded_operator(op.spelling().as_bytes(), left_value);
        }
        result
    }

    /// [`Interp::logical_values`]'s own computation, wrapped by it for the
    /// reason [`Interp::arith_general_body`] is.
    fn logical_values_body(
        &mut self,
        op: Operator,
        left_value: ObjRef,
        right_value: ObjRef,
    ) -> Result<ObjRef, Failure> {
        // **Ahead of the truth test, not behind it.** The oracle sends
        // `&`/`|`/`&&` to the left operand as a message -- `.array & 1` is
        // 97.1 where this crate's own 34.901 quotes the object's rendering --
        // so an operand this names refuses however it renders, and a
        // rendering that is itself `0` or `1` is not an answer: measured,
        // oracle rc 159, `o & 1` for an instance named `'1'` is
        // `97.1 Object "1" does not understand message "&".`
        if let Some(kind) = self.operator_operand_gap(left_value) {
            return Err(Loud::operator_operand(op.spelling(), kind).into());
        }
        let left_text = self.to_text(left_value).to_vec();
        let left_bool = logical_value(&left_text).ok_or_else(|| Raised::not_logical(&left_text))?;
        let right_text = self.to_text(right_value).to_vec();
        let right_bool =
            logical_value(&right_text).ok_or_else(|| Raised::not_logical(&right_text))?;

        let holds = match op {
            Operator::And => left_bool && right_bool,
            Operator::Or => left_bool || right_bool,
            Operator::Xor => left_bool != right_bool,
            other => return Err(Loud::binary_operator(other).into()),
        };

        let result = logical(holds);
        Ok(result)
    }

    /// The object an operator is **sent to as a message**, or `None` for an
    /// operand the operator converts itself.
    pub(crate) fn operator_message_receiver(&self, value: ObjRef) -> Option<ObjRef> {
        // **`.nil` is an operator receiver.** It answers `==`, `\\==`, `=`,
        // `<>` and `><` by identity and understands no other operator, so an
        // ordering or arithmetic operator against it is 97.1 -- measured,
        // `.nil < 'x'` and `.nil + 1` both report `Object "The NIL object"
        // does not understand message`. Reaching that through the send is
        // what `RexxObject`'s own operator methods do; converting `.nil` to
        // its string value instead makes it equal to the text `The NIL
        // object` and orderable against anything.
        if value == ObjRef::NIL {
            return Some(value);
        }
        // A small integer and an inline string leave on this line, which is
        // what keeps this off the cost of a comparison between two numbers.
        let Decoded::Heap { .. } = value.decode() else {
            return None;
        };
        match &self.heap.get(value)?.body {
            // **A class object is an operator receiver on the same terms
            // `.nil` is.** `Setup.cpp`'s `Class` block declares `=`, `==`,
            // `\\=`, `\\==`, `<>` and `><` and no other operator, so the six
            // are `Object`'s identity test and every other operator is a name
            // the behaviour does not hold -- measured, oracle rc 0,
            // `.array = .array` is `1` and `.array = 'The Array class'` is
            // `0`; oracle rc 159, `.array > .array` and `.array + 1` are both
            // `97.1 Object "The Array class" does not understand message`.
            Body::Class { .. } => Some(value),
            Body::Instance { .. } => Some(value),
            Body::Stem {
                default: Some(default),
                ..
            } => self.operator_message_receiver(*default),
            Body::VarRef(reference) => self
                .referenced_value(reference)
                .and_then(|referenced| self.operator_message_receiver(referenced)),
            _ => None,
        }
    }

    /// Sends an operator to its left operand as a message, for a receiver
    /// [`Interp::operator_message_receiver`] named.
    fn send_operator(
        &mut self,
        spelling: &str,
        receiver: ObjRef,
        args: &[Option<ObjRef>],
    ) -> Result<ObjRef, Failure> {
        let caller = self.caller();
        let name = spelling.as_bytes();
        match self.send_message(receiver, name, None, args, caller)? {
            Some(result) => Ok(result),
            None => Err(Raised::no_result(name).into()),
        }
    }

    /// The noun for an operand no operator here can take, or `None` for one
    /// every operator can.
    pub(crate) fn operator_operand_gap(&self, value: ObjRef) -> Option<&'static str> {
        // A small integer, an inline string and `.nil` all leave on this
        // line: only a heap-tagged handle can be either shape.
        let Decoded::Heap { .. } = value.decode() else {
            return None;
        };
        match &self.heap.get(value)?.body {
            Body::Class { .. } => Some("a class object"),
            Body::Native(_) => Some("one of the interpreter's own objects"),
            // An array *does* have a string value -- `ArrayClass::makeString`
            // joins its items -- so concatenation, which never asks here,
            // agrees with the oracle. An operator that does ask never gets
            // that string value: `.Array` defines no arithmetic or logical
            // method, so measured, oracle rc 159, `a = (1,); say a + 1` is
            // 97.1; and where a comparison finds one it is `Object`'s own
            // identity test, so `say (a = 1)` is `0` at rc 0 rather than a
            // comparison of `1` against `1`.
            Body::Array { .. } => Some("an array"),
            // The oracle sends the operator as a message here too, and it is
            // the send that fails: measured, oracle rc 159, `o + 1` on an
            // instance is `97.1 Object "a K" does not understand message
            // "+".`, and the same after `o~objectName = '123'` -- the
            // rendering being numeric does not make it a conversion.
            Body::Instance { .. } => Some("an instance of a user class"),
            // **The redirect every conversion here takes, taken here too.**
            // A stem with a default answers *as* that default -- `to_text`
            // and `to_number` both chase it -- so a check that stopped at the
            // stem handle would let `a. = .array; say (a. == 'The Array
            // class')` answer 1 where the oracle answers 0, which is this
            // gap's original shape reached through one assignment. Measured.
            // A compound read (`a.zz`) yields the class handle itself and
            // needs no redirect, which is what says the hole was the
            // indirection rather than the operator arm.
            Body::Stem {
                default: Some(default),
                ..
            } => self.operator_operand_gap(*default),
            // The redirect one arm up, for the same reason: every operator
            // reaches the referenced value through `UNKNOWN`, so it is that
            // value the oracle sends to. Measured, oracle rc 159:
            // `zz = .array~new(2); say (>zz) + 1` is
            // `97.1 Object "an Array" does not understand message "+".` --
            // the array's own answer, substituting the array rather than the
            // reference.
            Body::VarRef(reference) => match self.referenced_value(reference) {
                Some(referenced) => self.operator_operand_gap(referenced),
                // An unset variable reads as its own name, a string.
                None => None,
            },
            _ => None,
        }
    }

    /// `left op right` for every binary operator whose two operands are just
    /// values by the time it runs -- concatenation, comparison and logical.
    pub(crate) fn apply_binary(
        &mut self,
        op: Operator,
        left: ObjRef,
        right: ObjRef,
    ) -> Result<ObjRef, Failure> {
        // **`reqstr`'s dyadic-operator context is the operand on the
        // *right*.** The section names "Rexx dyadic operators when the
        // receiving object (the object to the left of the operator) is a
        // string", and the receiver's own method is what converts its
        // argument -- `StringClass::concatRexx`, `stringComp` and `andOp` all
        // open with `otherObj->requestString()` (`classes/StringClass.cpp:653`,
        // `:774`, `:925`). The left operand is the receiver and is not
        // converted at all: measured, `.array + 1` is 97.1 where `1 + .array`
        // is 41.1, and `eval.rs`'s `object_operand_tests` is that half.
        if let Some(target) = self.operator_message_receiver(left) {
            return self.send_operator(op.spelling(), target, &[Some(right)]);
        }
        // **A string is never equal to `.nil`, whatever its text.**
        // `RexxString::primitiveIsEqual` (`classes/StringClass.cpp:674`)
        // returns false for `TheNilObject` before it looks at any bytes, so
        // `'The NIL object' == .nil` is `0` where comparing string values
        // would make it `1`. Only the equality operators do this: measured,
        // `'a' < .nil` is `0` and `'a' || .nil` is `aThe NIL object`, both of
        // which do use the string value.
        if right == ObjRef::NIL && is_equality(op) {
            return Ok(logical(is_inequality(op)));
        }
        let right = self.required_string_value(right)?;
        match op {
            Operator::Concatenate | Operator::Abuttal => self.concat_values(left, right, None),
            Operator::Blank => self.concat_values(left, right, Some(b' ')),
            op if is_comparison(op) => self.compare_values(op, left, right),
            Operator::And | Operator::Or | Operator::Xor => self.logical_values(op, left, right),
            op => Err(Loud::binary_operator(op).into()),
        }
    }

    /// `ExprKind::Logical`: the comma-separated conditional list `IF a, b,
    /// c THEN` (and `WHEN`/`GUARD`/`WHILE`/`UNTIL`'s own versions)
    /// desugars to, "a logical AND of its parts" (`ast.rs`'s own doc
    /// comment).
    fn eval_logical_list(&mut self, code: &Code<'_>, items: &[Expr]) -> Result<ObjRef, Failure> {
        let frame = self.roots.push_frame();
        let indent = self.clause_state.current_value_indent;
        let mut holds = true;
        for item in items {
            let value = self.eval(code, item)?;
            self.roots.push_temp(value);
            let text = self.to_text(value).to_vec();
            self.trace_result(indent, &text);
            let item_holds =
                logical_value(&text).ok_or_else(|| Raised::logical_list_element(&text))?;
            if !item_holds {
                holds = false;
                break;
            }
        }

        let result = logical(holds);
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
#[inline(never)]
fn spelled_int_arith(
    op: Operator,
    left_value: ObjRef,
    right_value: ObjRef,
    digits: u64,
) -> Option<ObjRef> {
    let left_int = small_int_operand(left_value)?;
    let right_int = small_int_operand(right_value)?;
    small_int_arith(op, left_int, right_int, digits)
}

/// The integer an operand already is, whether it carries the tag or spells
/// one.
fn small_int_operand(value: ObjRef) -> Option<i64> {
    match value.decode() {
        Decoded::SmallInt(int) => Some(int),
        Decoded::Text(inline) => canonical_small_int(&inline),
        _ => None,
    }
}

fn small_int_arith(op: Operator, left: i64, right: i64, digits: u64) -> Option<ObjRef> {
    if !within_digits(left, digits) || !within_digits(right, digits) {
        return None;
    }
    let value = match op {
        Operator::Plus => left.checked_add(right),
        Operator::Subtract => left.checked_sub(right),
        Operator::Multiply => left.checked_mul(right),
        Operator::IntDiv => left.checked_div(right),
        Operator::Remainder => left.checked_rem(right),
        Operator::Divide if left.checked_rem(right) == Some(0) => left.checked_div(right),
        Operator::Power => small_int_power(left, right),
        _ => None,
    }?;
    exact_small_int(value, digits)
}

/// `base ** exponent` in `i64`, or `None` when [`small_int_arith`] must
/// decline.
fn small_int_power(base: i64, exponent: i64) -> Option<i64> {
    base.checked_pow(u32::try_from(exponent).ok()?)
}

pub(crate) fn saturate_digits(digits: u64) -> u32 {
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

/// `Operator` -> `rexx-num`'s `CompareOp`, for the `Operator` variants
/// `apply_binary` dispatches to `compare_values`. **Several operators share
/// one `CompareOp`**: `\=`/`<>`/`><` (`BackslashEqual`/`LessThanGreaterThan`/
/// `GreaterThanLessThan`) all mean `NotEqual` (`compare.rs`'s own doc
/// comment: the interpreter's operator table repeats one method pointer for
/// them), and `\>`/`\<` with their strict siblings `\>>`/`\<<` invert their
/// positive counterpart's sense rather than getting a `CompareOp` of their
/// own: `\>` ("not greater than") is `LessEqual`, `\<` is `GreaterEqual`,
/// `\>>` is `StrictLessEqual` and `\<<` is `StrictGreaterEqual`. `\==` is
/// **not** among them -- it has `StrictNotEqual` to itself -- and `\=` is one
/// of the operators sharing `NotEqual` above. Verified against the oracle
/// (this task's report), not derived from the names alone -- `\>>`/`\<<` are
/// strict-family byte comparisons, where "greater"/"less" do not carry the
/// numeric intuition their spelling suggests.
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
            "apply_binary only dispatches the comparison operators here, got {other:?}"
        ),
    }
}

/// Whether `op` is one of the operators [`Interp::eval_arithmetic`]
/// computes.
pub(crate) fn is_arithmetic(op: Operator) -> bool {
    use Operator::*;
    matches!(
        op,
        Plus | Subtract | Multiply | Divide | IntDiv | Remainder | Power
    )
}

/// Whether `op` is one of the operators [`Interp::concat_values`] joins bytes
/// for: `||`, the abuttal the parser synthesises between two adjacent terms,
/// and the blank between two terms with whitespace in it.
fn is_concatenation(op: Operator) -> bool {
    use Operator::*;
    matches!(op, Concatenate | Abuttal | Blank)
}

/// Whether `op` is one of the operators [`Interp::compare_values`] compares
/// under, the numeric-or-string family and the strict one.
fn is_comparison(op: Operator) -> bool {
    use Operator::*;
    matches!(
        op,
        Equal
            | BackslashEqual
            | GreaterThan
            | BackslashGreaterThan
            | LessThan
            | BackslashLessThan
            | GreaterThanEqual
            | LessThanEqual
            | StrictEqual
            | StrictBackslashEqual
            | StrictGreaterThan
            | StrictBackslashGreaterThan
            | StrictLessThan
            | StrictBackslashLessThan
            | StrictGreaterThanEqual
            | StrictLessThanEqual
            | LessThanGreaterThan
            | GreaterThanLessThan
    )
}

/// Whether `op` asks whether two values are the same, as against how they
/// order.
fn is_equality(op: Operator) -> bool {
    use Operator::*;
    matches!(
        op,
        Equal
            | BackslashEqual
            | StrictEqual
            | StrictBackslashEqual
            | LessThanGreaterThan
            | GreaterThanLessThan
    )
}

/// Which half of [`is_equality`] answers `1` for a value that is not `.nil`.
fn is_inequality(op: Operator) -> bool {
    use Operator::*;
    matches!(
        op,
        BackslashEqual | StrictBackslashEqual | LessThanGreaterThan | GreaterThanLessThan
    )
}

/// Whether `op` is one of the operators [`Interp::logical_values`] computes.
fn is_logical(op: Operator) -> bool {
    use Operator::*;
    matches!(op, And | Or | Xor)
}

/// Whether `op` has a native op of its own, which is what licenses
/// `crate::ir::compile` compiling it to registers rather than leaving its
/// whole expression to `crate::ir::Op::EvalExpr`.
pub(crate) fn is_native_binary(op: Operator) -> bool {
    is_arithmetic(op) || is_concatenation(op) || is_comparison(op) || is_logical(op)
}

/// The bytes a call target names, and whether an internal label may answer it.
pub(crate) fn call_target_name<'a>(code: &Code<'a>, target: &'a CallTarget) -> (&'a [u8], bool) {
    match target {
        CallTarget::Symbol(id) => (code.symbols.name(*id).as_bytes(), true),
        CallTarget::Literal(bytes) => (bytes, false),
    }
}

/// Whether `op` is one of the eight strict comparison operators -- decided
/// directly off `Operator` rather than off `CompareOp`, because
/// `CompareOp::is_strict` is private to `rexx-num` (`compare.rs`) and this
/// crate has no other need for a public one: strictness is only ever asked
/// here, to decide whether calling `to_number` is worth it at all before
/// `compare_decoded` is reached (a strict comparison never looks at a
/// `Number`, so parsing one first would be pure waste).
/// Whether two tagged small integers settle `op` between them, and how.
fn small_int_compare(op: Operator, left: ObjRef, right: ObjRef, digits: u64) -> Option<bool> {
    let (Decoded::SmallInt(left), Decoded::SmallInt(right)) = (left.decode(), right.decode())
    else {
        return None;
    };
    // `i128` so the bound itself cannot overflow at the `DIGITS` a program may
    // set, and so `abs` has a value for `i64::MIN`.
    let bound = 10i128.checked_pow(u32::try_from(digits).ok()?)?;
    if i128::from(left).abs() >= bound || i128::from(right).abs() >= bound {
        return None;
    }
    Some(match compare_op(op) {
        CompareOp::Equal | CompareOp::StrictEqual => left == right,
        CompareOp::NotEqual | CompareOp::StrictNotEqual => left != right,
        CompareOp::Greater => left > right,
        CompareOp::Less => left < right,
        CompareOp::GreaterEqual => left >= right,
        CompareOp::LessEqual => left <= right,
        CompareOp::StrictGreater
        | CompareOp::StrictLess
        | CompareOp::StrictGreaterEqual
        | CompareOp::StrictLessEqual => return None,
    })
}

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

    /// An operand that spells an integer reads as the same integer the tag
    /// carries, and one that spells it any other way does not.
    #[test]
    fn an_operand_that_spells_an_integer_reads_as_that_integer() {
        for value in [0i64, 1, -1, 9, -9, 10, -10, 1234, -1234, 999_999] {
            let tagged = ObjRef::small_int(value).expect("inside the tagged range");
            assert_eq!(small_int_operand(tagged), Some(value), "tagged {value}");
            let spelled = value.to_string();
            let text = ObjRef::inline_text(spelled.as_bytes()).expect("at most seven bytes");
            assert_eq!(small_int_operand(text), Some(value), "spelled {spelled}");
        }
        for spelling in ["05", "+5", " 5", "5 ", "5.0", "-0", "1e2", "", "5x", "-"] {
            let text = ObjRef::inline_text(spelling.as_bytes()).expect("at most seven bytes");
            assert_eq!(small_int_operand(text), None, "{spelling:?}");
        }
    }
    use super::*;
    use crate::plan::{BodyKey, ProgramId};
    use crate::{Activation, error::Failure};
    use rexx_parse::{InstructionKind, Program, parse_program};
    use std::rc::Rc;

    /// Pushes a fresh top-level activation for `program`, the same setup
    /// `Interp::run` does, so a test can drive `eval` through a live
    /// activation without running the instruction loop. `step`'s
    /// `Assignment` arm has handled all three targets (`Variable`/`Stem`/
    /// `Compound`) since Task 9, so this is no longer needed to work around
    /// a gap in `run` -- it stays because this module's tests are about
    /// `eval` itself, isolated from `step`'s surrounding dispatch, the same
    /// reason `run.rs`'s own test module keeps its own copy of the same
    /// helper rather than sharing one through `run`.
    fn activate(interp: &mut Interp, program: Program) -> Rc<Program> {
        let program = Rc::new(program);
        let program_id = ProgramId(interp.programs.len());
        interp.programs.push(Rc::clone(&program));
        let plan = interp.plan_for(
            BodyKey {
                program: program_id,
                directive: None,
            },
            &program.main,
            &program.symbols,
            &program.source,
        );
        let frame = interp.roots.push_slots(plan.len());
        let id = interp.next_activation_id();
        interp.push_activation(Activation::new(
            id,
            Rc::clone(&program),
            program_id,
            plan,
            frame,
        ));
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
            slots: &[],
            plan: None,
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
            slots: &[],
            plan: None,
        };
        interp.eval(&code, expr)
    }

    fn eval_in_place_text(interp: &mut Interp, source: &[u8]) -> Vec<u8> {
        let value = eval_in_place(interp, source)
            .unwrap_or_else(|failure| panic!("expected {source:?} to evaluate, got {failure:?}"));
        interp.to_text(value).to_vec()
    }

    // ---- the small-integer fast path ----

    /// Every answer the fast path gives is the answer the general path
    /// gives, over a grid of operands, precisions and both `FORM`s.
    #[test]
    fn the_small_int_fast_path_answers_what_the_general_path_answers() {
        use rexx_num::Form;

        let operands: [i64; 25] = [
            0,
            1,
            -1,
            2,
            -2,
            3,
            5,
            -5,
            7,
            -7,
            25,
            -25,
            99,
            100,
            999,
            -999,
            1000,
            -1000,
            1001,
            12345,
            -12345,
            1 << 30,
            -(1 << 30),
            rexx_core::SMALL_INT_MAX,
            rexx_core::SMALL_INT_MIN,
        ];
        let precisions: [u64; 9] = [1, 2, 3, 5, 9, 15, 18, 19, 20];
        let forms = [Form::Scientific, Form::Engineering];

        // Pinned to `is_arithmetic` in the direction that is a correctness
        // claim: nothing listed here is outside the set `eval_arithmetic`
        // dispatches. The other direction is not asserted and does not need
        // to be -- an arithmetic operator missing from this list is one
        // `small_int_arith`'s own `_` arm declines, which costs speed and
        // cannot cost an answer.
        let ops = [
            Operator::Plus,
            Operator::Subtract,
            Operator::Multiply,
            Operator::Divide,
            Operator::IntDiv,
            Operator::Remainder,
            Operator::Power,
        ];
        assert!(ops.iter().all(|op| is_arithmetic(*op)));

        let mut interp = Interp::new();
        // Per operator rather than one total: `+` alone reaches the fast path
        // thousands of times, so an aggregate count is satisfied by a guard
        // that admits nothing else. Each operator has to be seen going fast
        // on its own.
        let mut compared = [0usize; 7];
        for (index, op) in ops.into_iter().enumerate() {
            for left in operands {
                for right in operands {
                    for digits in precisions {
                        let Some(fast) = small_int_arith(op, left, right, digits) else {
                            continue;
                        };
                        let fast_text = interp.to_text(fast).to_vec();

                        let left_number =
                            Number::parse(&left.to_string()).expect("an i64 spelling parses");
                        let right_number =
                            Number::parse(&right.to_string()).expect("an i64 spelling parses");
                        let result = match op {
                            Operator::Plus => left_number.add(&right_number, digits),
                            Operator::Subtract => left_number.sub(&right_number, digits),
                            Operator::Multiply => left_number.mul(&right_number, digits),
                            Operator::Divide => {
                                left_number.div(&right_number, digits, DivOp::Divide)
                            }
                            Operator::IntDiv => {
                                left_number.div(&right_number, digits, DivOp::IntegerDivide)
                            }
                            Operator::Remainder => {
                                left_number.div(&right_number, digits, DivOp::Remainder)
                            }
                            Operator::Power => left_number.pow(&right_number, digits),
                            other => unreachable!("{other:?} is not on the fast path"),
                        }
                        .expect("no arithmetic error on the general path either");

                        // Both `FORM`s, because the fast path never reads
                        // `FORM` at all: the claim being tested is that under
                        // its own guard the two forms cannot disagree, which
                        // only an assertion over both can carry.
                        for form in forms {
                            let general =
                                interp.number(result.clone(), saturate_digits(digits), form);
                            let general_text = interp.to_text(general).to_vec();
                            assert_eq!(
                                String::from_utf8_lossy(&fast_text),
                                String::from_utf8_lossy(&general_text),
                                "{left} {op:?} {right} at DIGITS {digits}, FORM {form:?}"
                            );
                        }
                        compared[index] += 1;
                    }
                }
            }
        }
        // The grid is mostly refusals at the low precisions, and `/` and `**`
        // decline most of what they are handed by construction, so the floor
        // is the one every operator clears rather than one scaled to the
        // widest.
        for (index, op) in ops.into_iter().enumerate() {
            assert!(
                compared[index] > 100,
                "{op:?} reached the fast path only {} times over the grid",
                compared[index]
            );
        }
    }

    /// `/` is the one arithmetic operator whose exact answer need not be an
    /// integer, so the fast path takes it only when the division comes out
    /// even -- and must take it then, or the guard is just a refusal.
    #[test]
    fn division_goes_fast_only_when_it_is_exact() {
        let mut interp = Interp::new();

        assert!(small_int_arith(Operator::Divide, 1, 3, 9).is_none());
        assert!(small_int_arith(Operator::Divide, 7, 2, 9).is_none());

        let fast = small_int_arith(Operator::Divide, 6, 2, 9).expect("6 / 2 is exact");
        assert_eq!(&*interp.to_text(fast), b"3");
        let fast = small_int_arith(Operator::Divide, -1000000, 1000, 9).expect("this is exact too");
        assert_eq!(&*interp.to_text(fast), b"-1000");
    }

    /// A zero divisor leaves the fast path for all three division operators,
    /// so the 42.3 the general path raises is still what a program sees.
    #[test]
    fn a_zero_divisor_leaves_the_fast_path() {
        for op in [Operator::Divide, Operator::IntDiv, Operator::Remainder] {
            assert!(
                small_int_arith(op, 7, 0, 9).is_none(),
                "{op:?} by zero was answered on the fast path"
            );
        }
    }

    /// The sign rule for `%` and `//` with a negative operand, which is the
    /// half of this candidate a wrong `i64` intuition would get wrong
    /// silently.
    #[test]
    fn integer_division_truncates_toward_zero_and_the_remainder_follows_the_dividend() {
        let mut interp = Interp::new();
        let cases: [(Operator, i64, i64, &[u8]); 8] = [
            (Operator::IntDiv, -7, 3, b"-2"),
            (Operator::IntDiv, 7, -3, b"-2"),
            (Operator::IntDiv, -7, -3, b"2"),
            (Operator::IntDiv, 7, 3, b"2"),
            (Operator::Remainder, -7, 3, b"-1"),
            (Operator::Remainder, 7, -3, b"1"),
            (Operator::Remainder, -7, -3, b"-1"),
            (Operator::Remainder, 7, 3, b"1"),
        ];
        for (op, left, right, expected) in cases {
            let fast = small_int_arith(op, left, right, 9)
                .unwrap_or_else(|| panic!("{left} {op:?} {right} should go fast"));
            assert_eq!(
                &*interp.to_text(fast),
                expected,
                "{left} {op:?} {right} at DIGITS 9"
            );
        }
    }

    /// `**` takes a whole non-negative exponent whose exact power fits, and
    /// nothing else -- a negative exponent leaves the integers, and a power
    /// too wide for `DIGITS` would have to render exponentially.
    #[test]
    fn power_goes_fast_only_for_an_exact_non_negative_exponent() {
        let mut interp = Interp::new();

        assert!(small_int_arith(Operator::Power, 2, -1, 9).is_none());
        assert!(small_int_arith(Operator::Power, 2, 1_000_000_000, 9).is_none());
        // 2 ** 30 is 1073741824, ten digits, so under DIGITS 9 it rounds and
        // renders as 1.07374182E+9.
        assert!(small_int_arith(Operator::Power, 2, 30, 9).is_none());

        let fast = small_int_arith(Operator::Power, 2, 30, 10).expect("ten digits is enough");
        assert_eq!(&*interp.to_text(fast), b"1073741824");
        let fast = small_int_arith(Operator::Power, -3, 3, 9).expect("a negative base is fine");
        assert_eq!(&*interp.to_text(fast), b"-27");
        let fast = small_int_arith(Operator::Power, 0, 0, 9).expect("Rexx defines this as 1");
        assert_eq!(&*interp.to_text(fast), b"1");
    }

    /// The measured pair the guard exists for, and its neighbour that must
    /// still go fast.
    #[test]
    fn an_operand_too_wide_for_the_precision_leaves_the_fast_path() {
        assert!(small_int_arith(Operator::Subtract, 1000, 25, 3).is_none());

        let mut interp = Interp::new();
        let fast = small_int_arith(Operator::Subtract, 100, 25, 3)
            .expect("both operands fit three digits, and so does the result");
        assert_eq!(&*interp.to_text(fast), b"75");
    }

    // ---- terms ----

    #[test]
    fn a_literal_is_its_own_bytes() {
        let mut interp = Interp::new();
        assert_eq!(eval_text(&mut interp, b"say 123"), b"123");
    }

    #[test]
    fn a_constant_is_its_own_upcased_spelling() {
        // say 1e5 -> 1E5 (measured against the oracle; D15's own example).
        let mut interp = Interp::new();
        assert_eq!(eval_text(&mut interp, b"say 1e5"), b"1E5");
    }

    #[test]
    fn a_variable_reads_through_the_plan() {
        let mut interp = Interp::new();
        let program = parse_program(b"say x".to_vec()).expect("test program parses");
        let program = activate(&mut interp, program);
        let five = interp.text(b"5");
        let slot = interp.slot_of(b"X");
        let frame = interp.activation().frame;
        interp.roots.set_frame_slot(frame, slot, five);
        let expr = match &program.main.instructions[0].kind {
            InstructionKind::Say {
                expression: Some(expr),
            } => expr,
            other => panic!("expected a SAY with an expression, got {other:?}"),
        };
        let code = Code {
            body: &program.main,
            symbols: &program.symbols,
            slots: &[],
            plan: None,
        };
        let value = interp.eval(&code, expr).unwrap();
        assert_eq!(&*interp.to_text(value), b"5");
    }

    #[test]
    fn a_bare_stem_reads_through_the_same_path_as_a_variable() {
        // w. = 'wd' ; say w. -> wd
        let mut interp = Interp::new();
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
        let mut interp = Interp::new();
        activate(
            &mut interp,
            parse_program(b"nop".to_vec()).expect("test program parses"),
        );
        let x = interp.text(b"x");
        interp.stem_set(b"A.", b"1", x);
        assert_eq!(eval_in_place_text(&mut interp, b"say a.1"), b"x");
    }

    /// The names the parser resolves, which reach no directory at all.
    #[test]
    fn the_three_parse_time_dot_variables() {
        let mut interp = Interp::new();
        assert_eq!(eval_text(&mut interp, b"say .nil"), b"The NIL object");
        assert_eq!(eval_text(&mut interp, b"say .true"), b"1");
        assert_eq!(eval_text(&mut interp, b"say .false"), b"0");
    }

    /// Every other name goes through the resolution order, and each of its
    /// outcomes is reachable from an expression: an environment entry, a name
    /// nothing here answers that the oracle does, and a name neither answers.
    #[test]
    fn a_dot_variable_beyond_the_three_resolves_falls_back_or_is_loud() {
        let mut interp = Interp::new();
        assert_eq!(eval_text(&mut interp, b"say .array"), b"The Array class");
        assert_eq!(eval_text(&mut interp, b"say .foo"), b".FOO");
        let failure = eval_source(&mut interp, b"say .stdout").unwrap_err();
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
        let mut interp = Interp::new();
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
        let mut interp = Interp::new();
        assert_eq!(eval_text(&mut interp, b"say \\1"), b"0");
        assert_eq!(eval_text(&mut interp, b"say \\0"), b"1");
    }

    #[test]
    fn prefix_not_on_a_non_logical_value_raises_34_901() {
        // say \'abc' -> Error 34.901
        let mut interp = Interp::new();
        let failure = eval_source(&mut interp, b"say \\'abc'").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (34, 901));
        assert_eq!(raised.additional, vec![b"abc".to_vec()]);
    }

    // ---- arithmetic ----

    #[test]
    fn the_seven_arithmetic_operators() {
        let mut interp = Interp::new();
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
        let mut interp = Interp::new();
        let failure = eval_source(&mut interp, b"say 1/0").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (42, 3));
    }

    #[test]
    fn remainder_by_zero_also_raises_42_3() {
        // say 1//0 -> Error 42.3, rc 214 (the same DivideByZero as /)
        let mut interp = Interp::new();
        let failure = eval_source(&mut interp, b"say 1//0").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (42, 3));
    }

    #[test]
    fn a_nonnumeric_operand_raises_41_1_with_its_own_text() {
        // say 'abc' + 1 -> Error 41.1, "Nonnumeric value (\"abc\")"
        let mut interp = Interp::new();
        let failure = eval_source(&mut interp, b"say 'abc'+1").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (41, 1));
        assert_eq!(raised.additional, vec![b"abc".to_vec()]);
    }

    #[test]
    fn a_non_numeric_power_exponent_raises_26_8_not_41_1() {
        // say 2 ** 'x' -> Error 26.8, "found \"x\""
        // 'y' ** 2     -> Error 41.1 (the base's ordinary nonnumeric path)
        let mut interp = Interp::new();
        let failure = eval_source(&mut interp, b"say 2**'x'").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (26, 8));
        assert_eq!(raised.additional, vec![b"x".to_vec()]);

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
        let mut interp = Interp::new();
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
        let mut interp = Interp::new();
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
        interp.roots.set_frame_slot(frame, slot, a);
        let expr = match &program.main.instructions[0].kind {
            InstructionKind::Say {
                expression: Some(expr),
            } => expr,
            other => panic!("expected a SAY with an expression, got {other:?}"),
        };
        let code = Code {
            body: &program.main,
            symbols: &program.symbols,
            slots: &[],
            plan: None,
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
        let mut interp = Interp::new();
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
        let mut interp = Interp::new();
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
        // compare_values, reading `x` back out through a variable, still
        // takes the numeric path on an object whose `num` cache this test
        // itself already filled, rather than `to_number` inside
        // compare_values somehow needing a cold object to work at all.
        // 007 and 7 comparing numerically equal (not "0" -- a byte compare
        // would disagree with the leading zero) is what proves the
        // numeric path, not the string one, ran.
        let mut interp = Interp::new();
        activate(
            &mut interp,
            parse_program(b"nop".to_vec()).expect("test program parses"),
        );
        let x = interp.text(b"007");
        interp.to_number(x).expect("007 parses, filling its cache");
        let slot = interp.slot_of(b"X");
        let frame = interp.activation().frame;
        interp.roots.set_frame_slot(frame, slot, x);
        assert_eq!(eval_in_place_text(&mut interp, b"say (x = '7')"), b"1");
    }

    #[test]
    fn strict_equality_compares_bytes_where_the_ordinary_form_compares_value() {
        // The contrast is the test: the same two operands answer 0 under `==`
        // and 1 under `=`, so this pins that `==` reaches `CompareOp`'s strict
        // row rather than the ordinary one.
        let mut interp = Interp::new();
        assert_eq!(eval_text(&mut interp, b"say ('01' == '1')"), b"0");
        assert_eq!(eval_text(&mut interp, b"say ('01' = '1')"), b"1");
    }

    // ---- logical ----

    #[test]
    fn the_three_logical_operators_and_their_truth_tables() {
        let mut interp = Interp::new();
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
        let mut interp = Interp::new();
        let failure = eval_source(&mut interp, b"say (' 1 ' & 1)").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (34, 901));
        assert_eq!(raised.additional, vec![b" 1 ".to_vec()]);
    }

    #[test]
    fn logical_operators_do_not_short_circuit() {
        // say (0 & 'x') -> still raises on 'x', even though 0 already
        // decides the AND -- both operands are always evaluated and
        // checked. Contrast with ExprKind::Logical's comma list, below.
        let mut interp = Interp::new();
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
        let mut interp = Interp::new();
        let failure = eval_source(&mut interp, b"say ('y' & 'x')").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!(raised.additional, vec![b"y".to_vec()]);
    }

    // ---- ExprKind::Logical (the comma list) ----

    /// Parses `if <cond> then nop`, extracts the `IF`'s own `condition`
    /// (an `ExprKind::Logical` for a comma list, an ordinary `Expr`
    /// otherwise) and evaluates it directly, bypassing `run`. The only way
    /// to build one: comma-list syntax is gated to `IF`/`WHEN`/`GUARD`/
    /// `WHILE`/`UNTIL` in the parser (`ast.rs`'s own doc comment on
    /// `ExprKind::Logical`). `IF`/`WHEN`/`WHILE`/`UNTIL` all run since
    /// Tasks 10/11, so a real program can reach one through `run` too --
    /// this helper stays because it isolates `eval`'s own handling of the
    /// condition from `step`'s surrounding dispatch, not because `run`
    /// cannot reach one at all.
    fn eval_condition(interp: &mut Interp, source: &[u8]) -> Result<ObjRef, Failure> {
        let program = parse_program(source.to_vec()).expect("test program parses");
        // Activated like `eval_source`'s own programs, where this used to
        // evaluate against a bare `Interp`: `eval`'s own intermediate-value
        // gate reads the *running activation's* `TRACE` since Task 3 moved
        // `trace_mode` off `Interp`, so there has to be one.
        let program = activate(interp, program);
        let code = Code {
            body: &program.main,
            symbols: &program.symbols,
            slots: &[],
            plan: None,
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
        let mut interp = Interp::new();
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
        let mut interp = Interp::new();
        let failure = eval_condition(&mut interp, b"if 1, 'x' then nop").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (34, 6));
        assert_eq!(raised.additional, vec![b"x".to_vec()]);

        let first_bad = eval_condition(&mut interp, b"if 'x', 1 then nop").unwrap_err();
        let Failure::Raised(first_bad) = first_bad else {
            panic!("expected Raised, got {first_bad:?}");
        };
        assert_eq!(first_bad.additional, vec![b"x".to_vec()]);
    }

    #[test]
    fn a_comma_list_short_circuits_on_the_first_false_element() {
        // The opposite of `&`'s own rule
        // (logical_operators_do_not_short_circuit, above).
        let mut interp = Interp::new();
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

    // ---- D19's evaluation-depth limit ----

    /// A left-deep chain of `terms` `SAY`-able terms, joined by `||''` --
    /// the concatenation analogue `records_the_stack_cost_of_one_eval_frame`
    /// (`tests/spike.rs`) already establishes recurses exactly once per
    /// term (`outcome.stack.max_depth == TERMS`, asserted there), reused
    /// here rather than a fresh arithmetic chain invented for this test:
    /// one already-measured relationship between term count and `eval`
    /// depth is worth more than two unrelated ones.
    fn chain(terms: usize) -> Vec<u8> {
        let mut program = b"say 'a'".to_vec();
        for _ in 1..terms {
            program.extend_from_slice(b"||''");
        }
        program.push(b'\n');
        program
    }

    /// The same chain with one **call** in it, which is what makes the whole
    /// expression reach `eval` at all.
    fn eval_walked_chain(terms: usize) -> Vec<u8> {
        let mut program = b"say 'a'".to_vec();
        for term in 1..terms {
            // `substr('x', 1, 0)` is the empty string, so the chain's value is
            // unchanged and only its *shape* differs from `chain`'s.
            if term == 5 {
                program.extend_from_slice(b"||substr('x',1,0)");
            } else {
                program.extend_from_slice(b"||''");
            }
        }
        program.push(b'\n');
        program
    }

    /// **A native chain past the limit runs, and that is the point of the
    /// limit rather than a hole in it.**
    #[test]
    fn a_native_chain_past_the_eval_limit_runs() {
        let outcome = crate::run_program(
            "depth-native.rex",
            chain(MAX_EVAL_DEPTH + 1),
            crate::Invocation::none(),
        );
        assert_eq!(
            outcome.exit_code,
            0,
            "stderr: {:?}",
            String::from_utf8_lossy(&outcome.stderr)
        );
        assert_eq!(outcome.stdout, b"a\n");
        assert!(
            outcome.stack.max_depth < MAX_EVAL_DEPTH,
            "a native chain recursed {} levels into eval, so this test no longer \
             says that a compiled chain does not recurse",
            outcome.stack.max_depth
        );
    }

    /// **Why this goes through `run_program` and not a direct `eval` call.**
    /// The only sized stack in the workspace is inside `run_program`
    /// (`lib.rs`'s own `INTERPRETER_STACK_BYTES`); a `cargo test` thread's
    /// default 2 MiB is far smaller than what this depth needs, and `eval`
    /// would die natively, as an unreported guard-page abort, long before
    /// reaching either boundary this pair tests -- precisely the silent
    /// failure D19's limit exists to prevent, so a plain `#[cfg(test)]`
    /// unit test calling `eval` directly cannot exercise this at all. This
    /// is a unit test file, not `tests/`, but the subject it is testing
    /// (`run_program`'s own observable behaviour at the limit) is public
    /// cross-crate surface, so an integration-shaped test of it is not the
    /// thing the crate's own testing rule forbids (that rule is about
    /// integration-testing a *private* subject, and there is not one
    /// here) -- it simply lives beside the counter it defends rather than
    /// in a separate file, since nothing about reaching `run_program`
    /// requires a different module.
    #[test]
    fn eval_survives_exactly_max_eval_depth_terms_and_prints_the_oracles_own_answer() {
        let outcome = crate::run_program(
            "depth-boundary-at.rex",
            eval_walked_chain(MAX_EVAL_DEPTH),
            crate::Invocation::none(),
        );
        assert_eq!(
            outcome.exit_code,
            0,
            "stderr: {:?}",
            String::from_utf8_lossy(&outcome.stderr)
        );
        assert_eq!(
            outcome.stdout, b"a\n",
            "the oracle's own answer for this exact depth (Deviation 2, phase-4-exclusions.txt)"
        );
        assert_eq!(outcome.stack.max_depth, MAX_EVAL_DEPTH);
    }

    /// **The off-by-one this task's own brief calls out by name**: one term
    /// past the boundary the previous test pins must raise, not merely
    /// "eventually" refuse something deeper. Kills a `>=` in place of `eval`'s
    /// own `>` check, which would refuse the boundary case above instead of
    /// this one.
    #[test]
    fn eval_raises_11_1_exactly_one_term_past_max_eval_depth() {
        let outcome = crate::run_program(
            "depth-boundary-past.rex",
            eval_walked_chain(MAX_EVAL_DEPTH + 1),
            crate::Invocation::none(),
        );
        assert_eq!(
            outcome.exit_code,
            245,
            "256 - 11, stderr: {:?}",
            String::from_utf8_lossy(&outcome.stderr)
        );
        assert_eq!(
            outcome.stdout, b"",
            "SAY never runs: the whole expression must finish evaluating first, \
             and this one cannot"
        );
    }

    // ---- ExprKind::Call (Task 4, 4b): the internal-function expression
    // form ----

    /// The task brief's own Step 1: an internal routine called from inside
    /// an expression, not a `CALL` clause of its own, returns its value into
    /// the enclosing arithmetic.
    #[test]
    fn an_internal_function_returns_its_value_into_an_expression() {
        let outcome = crate::run_program(
            "call-expr-basic.rex",
            b"say f(1) + 1\nexit\nf: return 41\n".to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(
            outcome.exit_code,
            0,
            "stderr: {:?}",
            String::from_utf8_lossy(&outcome.stderr)
        );
        assert_eq!(outcome.stdout, b"42\n");
    }

    /// A name that resolves to nothing -- not a label of the calling body,
    /// not a builtin and not a `::ROUTINE` -- raises the oracle's own 43.1
    /// rather than succeeding, crashing, or reporting a gap.
    #[test]
    fn an_unresolvable_name_raises_43_1() {
        let outcome = crate::run_program(
            "call-expr-builtin.rex",
            b"say zorkolo('abc')\n".to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(outcome.exit_code, 213);
        assert!(
            String::from_utf8_lossy(&outcome.stderr)
                .contains(r#"Error 43.1:  Could not find routine "ZORKOLO"."#),
            "stderr: {:?}",
            String::from_utf8_lossy(&outcome.stderr)
        );
        assert_eq!(outcome.stdout, b"");
    }

    /// The neighbouring case the one above cannot pin on its own: a builtin
    /// Phase 4 excludes **outright** is not "resolves to nothing", and must
    /// stay loud rather than joining it at 43.1. The oracle answers `CHARIN`,
    /// so a condition here would let a program expecting one pass against a
    /// gap.
    #[test]
    fn a_wholly_excluded_builtin_stays_loud_rather_than_raising_43_1() {
        let outcome = crate::run_program(
            "call-expr-excluded.rex",
            b"say charin('nosuch.txt')\n".to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(outcome.exit_code, crate::NOT_IMPLEMENTED_EXIT);
        assert_eq!(
            String::from_utf8_lossy(&outcome.stderr),
            "rexx-exec: routine \"CHARIN\" is not implemented (Phase 7)\n",
            "the refusal names the phase the exclusion table gives it"
        );
    }

    /// **`CallTarget::Literal` never searches the label table**, symmetric
    /// with `CALL "SUB"` (Task 3). Measured independently on the oracle in a
    /// clean directory (never the scratchpad root, which sits on the
    /// external-routine search path and holds a stale `f.rex` -- the first
    /// attempt at exactly this measurement found it and reported the wrong
    /// answer): `say "f"(1)` with `f:` present is Error 43.1 rc 213, "Routine
    /// not found", where `say f(1)` runs the label instead. That is this
    /// crate's answer too now that the builtin and `::ROUTINE` steps behind
    /// the label search exist: "not a label" and "not anything at all" are
    /// separable, so the condition is the oracle's own rather than fabricated.
    #[test]
    fn a_literal_call_target_never_reaches_the_label_table() {
        let outcome = crate::run_program(
            "call-expr-literal.rex",
            b"say \"f\"(1)\nexit\nf: return 41\n".to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(outcome.exit_code, 213);
        assert!(
            String::from_utf8_lossy(&outcome.stderr)
                .contains(r#"Error 43.1:  Could not find routine "f"."#),
            "stderr: {:?}",
            String::from_utf8_lossy(&outcome.stderr)
        );
        assert_eq!(outcome.stdout, b"", "the label must not have run");
    }

    /// **Step 3's own measurement**: a routine reached through the
    /// expression form and returning nothing -- a bare `RETURN`, not the
    /// same event as falling off the routine's own end without one (the next
    /// test) -- raises Error 44.1 at rc 212. Measured on the oracle in a
    /// clean directory: `say f(1)` into `f: return` gives exactly this,
    /// "No data returned from function "F"." with the label's own upcased
    /// spelling.
    #[test]
    fn a_routine_returning_no_value_in_expression_form_raises_44_1() {
        let outcome = crate::run_program(
            "call-expr-no-data.rex",
            b"say f(1)\nexit\nf: return\n".to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(outcome.exit_code, 212, "256 - 44");
        let stderr = String::from_utf8_lossy(&outcome.stderr);
        assert!(
            stderr.contains("Function or message did not return data.")
                && stderr.contains("No data returned from function \"F\"."),
            "stderr: {stderr:?}"
        );
        assert_eq!(
            outcome.stdout, b"",
            "SAY never runs: the argument raises before the clause completes"
        );
    }

    /// **Measured on the oracle**, in a clean directory: `EXIT` inside a
    /// routine reached through the expression form ends the whole program
    /// exactly as it does through `CALL`, at rc 5, with no stdout (`SAY`
    /// never completes) and no stderr (an `EXIT` is not a condition, so
    /// nothing is reported). This exercises `Failure::Exited`'s whole reason
    /// for existing (`error.rs`): `eval`'s own return type has no `Flow` to
    /// carry the event through the way `CALL`'s instruction form does, so it
    /// travels as this `Failure` variant instead, unwound by every
    /// enclosing `?` with no special handling needed, until `execute`
    /// (`lib.rs`) reads it back as an ordinary successful exit.
    #[test]
    fn an_exit_inside_a_routine_reached_by_expression_call_ends_the_whole_program() {
        let outcome = crate::run_program(
            "call-expr-exit.rex",
            b"say f(1)\nexit 9\nf: exit 5\n".to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(outcome.exit_code, 5, "stderr: {:?}", outcome.stderr);
        assert_eq!(outcome.stdout, b"");
        assert_eq!(outcome.stderr, b"");
    }

    /// **Review finding M4, Task 4 fix round 1.** `Failure::Exited` has two
    /// producers -- an `EXIT` instruction (the test above) and a routine
    /// falling off its own end (`run_activation`'s own `Ok(Ended::Exited(
    /// None))` when its instruction loop runs out) -- and only the first had
    /// a test. The two are genuinely distinct events reaching the same
    /// `Ended::Exited` arm in `eval_call`, and the risk this test closes is
    /// specific: a version that only checked for an explicit `Exit` value
    /// while treating "no more instructions" as `Ended::Returned(None)`
    /// would misroute this case into `Raised::no_data_returned` (Error
    /// 44.1) instead of silently ending the program -- a real, measured
    /// divergence (`error.rs`'s `no_data_returned` doc has the "**Not** the
    /// same path" note this pins). Measured on the oracle in a clean
    /// directory: `say f(1)` into `f: nop` at the very end of the file (no
    /// `RETURN`, nothing after it) gives rc 0, empty stdout, empty stderr --
    /// `SAY` never runs because the whole program ends before its argument
    /// finishes evaluating, exactly as the explicit-`EXIT` case does.
    #[test]
    fn a_routine_falling_off_its_own_end_in_expression_form_also_ends_the_whole_program() {
        let outcome = crate::run_program(
            "call-expr-fall-off.rex",
            b"say f(1)\nexit\nf: nop\n".to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
        assert_eq!(outcome.stdout, b"");
        assert_eq!(outcome.stderr, b"");
    }

    /// **Measured**: a caller's `RESULT` is unaffected by `f(1)` appearing in
    /// an expression, unlike `CALL`, which settles it on every return
    /// (`Interp::invoke_named_call`'s own doc, `run.rs`). `result = 'before'`
    /// survives `zz = f(1)` untouched.
    #[test]
    fn an_internal_functions_expression_form_does_not_touch_result() {
        let outcome = crate::run_program(
            "call-expr-result.rex",
            b"result = 'before'\nzz = f(1)\nsay result\nexit\nf: return 99\n".to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(
            outcome.exit_code,
            0,
            "stderr: {:?}",
            String::from_utf8_lossy(&outcome.stderr)
        );
        assert_eq!(outcome.stdout, b"before\n");
    }
}

/// **R12: an operator whose left operand is an object this phase can build
/// and send no message to.**
#[cfg(test)]
mod object_operand_tests {
    use crate::{Invocation, run_program};

    /// Runs `source` and hands back `(exit code, stdout, stderr)`.
    fn run_source(source: &[u8]) -> (i32, String, String) {
        let outcome = run_program("/t.rex", source.to_vec(), Invocation::none());
        (
            outcome.exit_code,
            String::from_utf8_lossy(&outcome.stdout).into_owned(),
            String::from_utf8_lossy(&outcome.stderr).into_owned(),
        )
    }

    /// Every operator the oracle sends to its left operand as a message
    /// refuses loudly, naming the operator and the operand's shape.
    #[test]
    fn an_operator_sent_to_an_object_is_loud() {
        // (source, the operator the message must name, the shape it must name)
        let cases: &[(&[u8], &str, &str)] = &[
            // **No `.array` row here**, and that is the property rather than
            // an omission: a class object is an operator *receiver*
            // (`Interp::operator_message_receiver`), so every operator
            // reaches it as a message and none of them reports a gap.
            // `a_class_objects_operators_are_sent_as_messages` is where they
            // are asserted, and the rows below are the control -- a change
            // that widened past class handles would move one of them.
            (
                b"say (.environment + 1)\n",
                "+",
                "one of the interpreter's own objects",
            ),
            // oracle 0 -- two distinct tables rendering the same text
            (
                b"say (.methods == .routines)\n::method m\n  return 1\n::routine r\n  return 2\n",
                "==",
                "one of the interpreter's own objects",
            ),
            // An array, which `~superClasses` puts in a program's hands.
            // oracle 97.1 at rc 159, `Object "an Array" does not understand
            // message "+"` -- `.Array` defines no arithmetic or logical
            // method, so the send finds nothing.
            (b"say (.Object~superClasses + 1)\n", "+", "an array"),
            (b"say (.Object~superClasses & 1)\n", "&", "an array"),
            // oracle 0 on both, measured -- `Object`'s own identity
            // comparison, which this crate does not model. **A build
            // converting through the array's string value answers `1`**: the
            // array is `.Object`'s own superclass list, which holds nothing,
            // so the items joined are the empty string. Measured, three
            // descriptors, `say (.Object~superClasses = '')` and the `==`
            // form: `0` on the oracle at rc 0.
            (b"say (.Object~superClasses = '')\n", "=", "an array"),
            (b"say (.Object~superClasses == '')\n", "==", "an array"),
            // An array whose joined items *are* a number, which the rows
            // above cannot reach: `.Object~superClasses` holds nothing, so
            // its string value is empty and parses as no number whatever
            // this crate does with it. Measured, oracle: `a = (1,)` then
            // `say a + 1` and `say (a & 1)` are 97.1 at rc 159, and
            // `say (a = 1)` is `0` at rc 0 -- `Object`'s identity test, not
            // a comparison of `1` against `1`.
            (b"a = (1,)\nsay (a + 1)\n", "+", "an array"),
            (b"a = (1,)\nsay (a = 1)\n", "=", "an array"),
            (b"a = (1,)\nsay (a & 1)\n", "&", "an array"),
            (b"a = (1,)\nsay \\a\n", "\\", "an array"),
            // A package object, which `~package` puts in a program's hands
            // and which reaches the same arm `.environment` does.
            // oracle 97.1 at rc 159
            (
                b"say (.Array~package + 1)\n",
                "+",
                "one of the interpreter's own objects",
            ),
            // oracle 1 -- one package object, measured:
            // `(.Array~package == .String~package)`
            (
                b"say (.Array~package == .String~package)\n",
                "==",
                "one of the interpreter's own objects",
            ),
        ];
        for (source, op, kind) in cases {
            let (code, stdout, stderr) = run_source(source);
            let expected = format!(
                "rexx-exec: the operator `{op}` applied to {kind} is not implemented (Phase 5)\n"
            );
            assert_eq!(
                (code, stdout.as_str(), stderr.as_str()),
                (120, "", expected.as_str()),
                "{:?}",
                String::from_utf8_lossy(source)
            );
        }
    }

    /// A class object answers the comparison operators by **identity** and
    /// refuses every other one the way the oracle does, at both spellings.
    #[test]
    fn a_class_objects_operators_are_sent_as_messages() {
        // Identity and not a comparison of renderings, which is what the
        // `'The Array class'` rows pin: measured, oracle rc 0.
        for (source, expected) in [
            (&b"say (.array = .array)\n"[..], "1"),
            (b"say (.array == .array)\n", "1"),
            (b"say (.array = 'The Array class')\n", "0"),
            (b"say (.array == 'The Array class')\n", "0"),
            (b"say (.array \\= .array)\n", "0"),
            (b"say (.array \\== .array)\n", "0"),
            (b"say (.array <> .string)\n", "1"),
            (b"say (.array >< .string)\n", "1"),
            // The message spelling of the same six.
            (b"say .Array~'='(.Array)\n", "1"),
            (b"say .Array~'=='(.Array)\n", "1"),
            (b"say .Array~'\\='(.Array)\n", "0"),
            (b"say .Array~'\\=='(.Array)\n", "0"),
            (b"say .Array~'<>'(.String)\n", "1"),
            (b"say .Array~'><'(.String)\n", "1"),
            // Concatenation never asked the gap and still does not, which is
            // the control for the receiver change: measured, oracle rc 0.
            (b"say (.array || 'x')\n", "The Array classx"),
            (b"say (.array 'x')\n", "The Array class x"),
        ] {
            let (code, stdout, stderr) = run_source(source);
            assert_eq!(
                (code, stdout.as_str(), stderr.as_str()),
                (0, format!("{expected}\n").as_str(), ""),
                "{:?}",
                String::from_utf8_lossy(source)
            );
        }

        // Every other operator, at rc 159 rather than this crate's own
        // refusal. Measured, oracle: `Object "The Array class" does not
        // understand message`.
        for (source, message) in [
            (&b"say (.array > .array)\n"[..], ">"),
            (b"say (.array < .string)\n", "<"),
            (b"say (.array + 1)\n", "+"),
            (b"say (.array ** 1)\n", "**"),
            (b"say (.array & 1)\n", "&"),
            (b"say -.array\n", "-"),
            (b"say \\.array\n", "\\"),
            (b"say .Array~'+'(1)\n", "+"),
        ] {
            let (code, stdout, stderr) = run_source(source);
            assert_eq!(
                (code, stdout.as_str()),
                (159, ""),
                "{:?}",
                String::from_utf8_lossy(source)
            );
            assert!(
                stderr.contains(&format!(
                    "Error 97.1:  Object \"The Array class\" does not understand message \
                     \"{message}\"."
                )),
                "{:?} reported {stderr:?}",
                String::from_utf8_lossy(source)
            );
        }
    }

    /// **R12 at the one numeric surface that is not an operator**: a
    /// controlled `DO` header's `initial`, `TO` and `BY` values.
    #[test]
    fn an_object_in_a_do_headers_numeric_position_is_loud() {
        let cases: &[(&[u8], &str, &str)] = &[
            (b"do i = .array to 5\nend\n", "initial", "a class object"),
            (b"do i = 1 to .array\nend\n", "TO", "a class object"),
            (b"do i = 1 to 5 by .array\nend\n", "BY", "a class object"),
            (
                b"do i = .environment to 5\nend\n",
                "initial",
                "one of the interpreter's own objects",
            ),
            (
                b"do i = .Object~superClasses to 5\nend\n",
                "initial",
                "an array",
            ),
        ];
        for (source, role, kind) in cases {
            let (code, stdout, stderr) = run_source(source);
            let expected = format!(
                "rexx-exec: {kind} as a DO header's {role} value is not implemented (Phase 5)\n"
            );
            assert_eq!(
                (code, stdout.as_str(), stderr.as_str()),
                (120, "", expected.as_str()),
                "{:?}",
                String::from_utf8_lossy(source)
            );
        }
    }

    /// The header positions that read the value's **text** rather than
    /// converting it, which both implementations answer alike.
    #[test]
    fn a_header_position_that_reads_text_keeps_the_oracles_own_diagnostic() {
        for (source, major) in [
            (&b"do i = 1 to 5 for .array\nend\n"[..], "26.3"),
            (b"do .array\nend\n", "26.2"),
            (b"numeric digits .array\n", "26.5"),
        ] {
            let (code, _stdout, stderr) = run_source(source);
            assert_eq!(code, 230, "{:?}", String::from_utf8_lossy(source));
            assert!(
                stderr.contains(&format!("Error {major}:")),
                "{:?} reported {stderr:?}",
                String::from_utf8_lossy(source)
            );
        }
    }

    /// **`DO OVER` hands its target to `requestArray`**, which is neither
    /// `stringValue()` nor an operator, so it falls outside every boundary the
    /// two tests above draw.
    #[test]
    fn an_object_as_a_do_over_target_is_loud() {
        let cases: &[(&[u8], &str)] = &[
            (
                b"do e over .environment\nsay e\nend\n",
                "one of the interpreter's own objects",
            ),
            (
                b"do e over .local\nsay e\nend\n",
                "one of the interpreter's own objects",
            ),
            (
                b"do e over .environment for 2\nsay e\nend\n",
                "one of the interpreter's own objects",
            ),
        ];
        for (source, kind) in cases {
            let (code, stdout, stderr) = run_source(source);
            let expected = format!(
                "rexx-exec: {kind} as a DO header's OVER target is not implemented (Phase 5)\n"
            );
            assert_eq!(
                (code, stdout.as_str(), stderr.as_str()),
                (120, "", expected.as_str()),
                "{:?}",
                String::from_utf8_lossy(source)
            );
        }
        // **A class object is no longer one of them, and that is the point of
        // the pair.** It reaches `requestArray`, answers no `MAKEARRAY`, and
        // raises the oracle's own `Error_Execution_noarray` naming itself --
        // measured, oracle rc 158, `Unable to convert object "The Array
        // class" to a single-dimensional array value.` So what is left
        // refusing above is exactly the two directories this crate models as
        // a subset, which is a MEMBERSHIP difference rather than a missing
        // conversion.
        let (code, stdout, stderr) = run_source(b"do e over .array\nsay e\nend\n");
        assert_eq!((code, stdout.as_str()), (158, ""));
        assert!(
            stderr.contains(
                "Error 98.913:  Unable to convert object \"The Array class\" to a \
                 single-dimensional array value."
            ),
            "a class object must raise the oracle's own noarray, got {stderr:?}"
        );

        // The adjacent success, which is what makes the refusals above about
        // the *directories* rather than about `Body::Native`: the one
        // collection this crate does iterate is a `StringTable`, and the
        // count is order-independent for the reason
        // `Interp::hash_collection_indexes` gives.
        assert_eq!(
            run_source(
                b"n = 0\ndo e over .methods\n  n = n + 1\nend\nsay n\n::method a\n::method b\n"
            ),
            (0, "2\n".to_string(), String::new())
        );
    }

    /// **A stem redirects to its default, and every check has to follow.**
    #[test]
    fn an_object_reached_through_a_stem_default_answers_as_the_object() {
        // Measured, oracle rc 0 and rc 159: the redirect reaches the class
        // itself, so the identity test and the 97.1 are the oracle's.
        for (source, code, stdout, message) in [
            (
                &b"a. = .array\nsay (a. == 'The Array class')\n"[..],
                0,
                "0\n",
                None,
            ),
            (b"a. = .array\nsay a. + 1\n", 159, "", Some("+")),
            (b"a. = .array\nsay a.zz + 1\n", 159, "", Some("+")),
        ] {
            let (actual, out, stderr) = run_source(source);
            assert_eq!(
                (actual, out.as_str()),
                (code, stdout),
                "{:?} reported {stderr:?}",
                String::from_utf8_lossy(source)
            );
            if let Some(message) = message {
                assert!(
                    stderr.contains(&format!(
                        "Error 97.1:  Object \"The Array class\" does not understand message \
                         \"{message}\"."
                    )),
                    "{:?} reported {stderr:?}",
                    String::from_utf8_lossy(source)
                );
            }
        }

        // The control, and a gap this crate still has: a `DO` header converts
        // through `Interp::header_number`, which asks
        // `Interp::operator_operand_gap` and never the send, so this refuses
        // where the oracle answers 97.1 at rc 159.
        let (code, stdout, stderr) = run_source(b"a. = .array\ndo i = 1 to a.\nend\n");
        assert_eq!((code, stdout.as_str()), (120, ""), "reported {stderr:?}");
        assert!(
            stderr.contains("a class object"),
            "must name the shape it refused, got {stderr:?}"
        );
    }

    /// A controlled loop's own increment adds to the control variable, and
    /// that variable is the oracle's **left** operand of the implicit `+`.
    #[test]
    fn an_object_assigned_to_a_control_variable_is_loud_at_the_increment() {
        let (code, stdout, stderr) = run_source(b"do i = 1 to 3\nsay 'iter' i\ni = .array\nend\n");
        assert_eq!(
            (code, stdout.as_str(), stderr.as_str()),
            (
                120,
                "iter 1\n",
                "rexx-exec: a class object as a controlled DO's control variable is not \
                 implemented (Phase 5)\n"
            )
        );
    }

    /// `RAISE ... ADDITIONAL` hands its value to `requestArray` -- **under a
    /// `SYNTAX` condition and nowhere else**.
    #[test]
    fn an_object_as_a_raise_syntax_substitution_is_loud() {
        for source in [
            &b"raise syntax 40.1 additional (.array)\n"[..],
            b"raise syntax 40.1 additional (.environment)\n",
        ] {
            let (code, stdout, stderr) = run_source(source);
            assert_eq!(
                (code, stdout.as_str()),
                (120, ""),
                "{:?} reported {stderr:?}",
                String::from_utf8_lossy(source)
            );
            assert!(
                stderr.contains("a RAISE ADDITIONAL value"),
                "{:?} must name its position, got {stderr:?}",
                String::from_utf8_lossy(source)
            );
        }
    }

    /// **The `RAISE` over-refusal control, and the reason it is its own
    /// test.**
    #[test]
    fn a_raise_the_oracle_does_not_array_convert_still_answers() {
        let cases: &[(&[u8], i32, &str)] = &[
            // The ARRAY form: the oracle has an array already, so the object
            // is rendered into the substitution like any other value.
            (
                b"raise syntax 40.1 array (.array)\n",
                216,
                "External routine \"The Array class\" failed.",
            ),
            (
                b"raise syntax 40.1 array (.environment)\n",
                216,
                "External routine \"The Environment Directory\" failed.",
            ),
            (
                b"raise syntax 40.1 array (.array, 'b')\n",
                216,
                "External routine \"The Array class\" failed.",
            ),
            // Not a SYNTAX condition, so `requestArray` is never reached.
            (
                b"signal on user zork name got\nraise user zork additional (.array)\ngot:\nsay 'trapped'\n",
                0,
                "",
            ),
            // DESCRIPTION is a different keyword and never array-converted.
            (
                b"raise syntax 40.1 description (.array)\n",
                216,
                "External routine \"&1\" failed.",
            ),
        ];
        for (source, expected_code, expected_in_stderr) in cases {
            let (code, _stdout, stderr) = run_source(source);
            assert_eq!(
                code,
                *expected_code,
                "{:?} reported {stderr:?}",
                String::from_utf8_lossy(source)
            );
            assert!(
                stderr.contains(expected_in_stderr),
                "{:?} must still answer the oracle's own bytes, got {stderr:?}",
                String::from_utf8_lossy(source)
            );
        }
    }

    /// **The anti-over-refusal control, and it is not optional.** Making
    /// *every* operator loud would satisfy the test above and refuse a pile
    /// of programs the oracle runs at rc 0 with bytes this crate already
    /// matches.
    #[test]
    fn an_object_the_operator_is_not_sent_to_still_answers() {
        let cases: &[(&[u8], &str)] = &[
            // the concatenation family, object on the left
            (b"say .array || 'x'\n", "The Array classx\n"),
            (b"say .array'x'\n", "The Array classx\n"),
            (b"say .array 'x'\n", "The Array class x\n"),
            (b"say .environment || 'x'\n", "The Environment Directoryx\n"),
            // an object on the right of an operator sent to a string
            (
                b"say ('a StringTable' == .methods)\n::method m\n  return 1\n",
                "1\n",
            ),
            (b"say (1 == .methods)\n::method m\n  return 1\n", "0\n"),
            // and the plain rendering both `.NAME` routes owe
            (b"say .LOCAL\n", "The Local Directory\n"),
            (b"say value('.LOCAL')\n", "The Local Directory\n"),
            (b"x = .array; say x\n", "The Array class\n"),
            // `DO OVER` on a string still iterates once yielding itself,
            // which is `Interp::over_snapshot`' own rule for a target that is
            // not an array.
            (b"do e over 'abc'\nsay e\nend\n", "abc\n"),
            // A stem whose default is an ordinary value is untouched by the
            // redirect the check now follows.
            (b"a. = .array\nsay a.\n", "The Array class\n"),
            (b"a. = .array\nsay a. || 'x'\n", "The Array classx\n"),
            (b"z. = 5\ndo i = 1 to z.\nsay i\nend\n", "1\n2\n3\n4\n5\n"),
        ];
        for (source, expected) in cases {
            let (code, stdout, stderr) = run_source(source);
            assert_eq!(
                (code, stdout.as_str(), stderr.as_str()),
                (0, *expected, ""),
                "{:?}",
                String::from_utf8_lossy(source)
            );
        }
    }

    /// A condition is not an operator, and the oracle's answer for one is
    /// this crate's already.
    #[test]
    fn a_condition_on_an_object_keeps_the_oracles_own_diagnostic() {
        for (source, major) in [
            (&b"if .array then say 'y'\n"[..], "34.1"),
            (b"if .array, 1 then say 'y'\n", "34.6"),
            (b"do while .array\nend\n", "34.3"),
        ] {
            let (code, _stdout, stderr) = run_source(source);
            assert_eq!(code, 222, "{:?}", String::from_utf8_lossy(source));
            assert!(
                stderr.contains(&format!("Error {major}:")),
                "{:?} reported {stderr:?}",
                String::from_utf8_lossy(source)
            );
        }
    }

    /// A stem whose default value answers no operator at all -- `.nil` --
    /// never reaches a method to run, so its own forwarded failure carries
    /// no traceback frame, unlike `b.` (no default) and a `String`-defaulted
    /// stem, both of which do.
    #[test]
    fn a_nil_defaulted_stem_carries_no_operator_frame() {
        let (code, stdout, stderr) = run_source(b"s. = .nil\nsay s. + 1\n");
        assert_eq!(code, 159, "{stderr:?}");
        assert!(
            stderr.contains(
                r#"Error 97.1:  Object "The NIL object" does not understand message "+"."#
            ),
            "{stderr:?}"
        );
        assert_eq!(stdout, "");
        assert!(
            !stderr.contains("Compiled method"),
            "a `.nil`-defaulted stem's own arithmetic failure must carry no \
             operator-forwarded frame -- no method ever ran to raise from -- \
             but got {stderr:?}"
        );
    }

    /// An operator on an instance is the message send the oracle makes, and
    /// a name that spells a number never converts the receiver.
    #[test]
    fn an_operator_on_an_instance_is_the_send_the_oracle_makes() {
        let prologue = "o = .K~new\no~objectName = '1'\n";
        let epilogue = "::CLASS K\n";
        for (expression, spelling) in [
            ("o + 1", "+"),
            ("o - 1", "-"),
            ("o * 2", "*"),
            ("o / 2", "/"),
            ("o // 2", "//"),
            ("o % 2", "%"),
            ("o ** 2", "**"),
            ("(o > 0)", ">"),
            ("(o < 2)", "<"),
            ("(o >= 0)", ">="),
            ("(o <= 2)", "<="),
            ("(o >> '0')", ">>"),
            ("(o << '2')", "<<"),
            ("(o \\> 0)", "\\>"),
            ("(o \\< 2)", "\\<"),
            ("(o & 1)", "&"),
            ("(o | 0)", "|"),
            ("(o && 1)", "&&"),
            ("(\\o)", "\\"),
            ("(-o)", "-"),
            ("(+o)", "+"),
        ] {
            let source = format!("{prologue}say {expression}\n{epilogue}");
            let (code, stdout, stderr) = run_source(source.as_bytes());
            assert_eq!(code, 159, "{expression}: {stderr:?}");
            assert_eq!(stdout, "", "{expression}");
            assert!(
                stderr.contains(&format!(
                    "Object \"1\" does not understand message \"{spelling}\"."
                )),
                "{expression} reported {stderr:?}"
            );
        }
        for (expression, expected) in [
            ("(o = 1)", "0\n"),
            ("(o == '1')", "0\n"),
            ("(o \\= 1)", "1\n"),
            ("(o \\== '1')", "1\n"),
            ("(o <> 1)", "1\n"),
            ("(o >< 1)", "1\n"),
            ("(o || 'q')", "1q\n"),
            ("(o 'q')", "1 q\n"),
            ("(1 = o)", "1\n"),
            ("('q' || o)", "q1\n"),
        ] {
            let source = format!("{prologue}say {expression}\n{epilogue}");
            let (code, stdout, stderr) = run_source(source.as_bytes());
            assert_eq!(code, 0, "{expression}: {stderr:?}");
            assert_eq!(stdout, expected, "{expression}");
        }
        let source = format!("{prologue}if o then say 'yes'\nelse say 'no'\n{epilogue}");
        let (code, stdout, stderr) = run_source(source.as_bytes());
        assert_eq!(code, 0, "{stderr:?}");
        assert_eq!(stdout, "yes\n");
    }

    /// A value [`Interp::operator_operand_gap`] names parses as no number,
    /// which is what makes [`Interp::compare_values`]'s skip past that gap
    /// sound rather than merely cheap -- and the same for
    /// [`Interp::operator_message_receiver`], which
    /// [`Interp::arith_left_operand`] asks only where [`Interp::to_number`]
    /// has already refused. An operand that produced a `Number` and was also
    /// a send target would have its operator applied to the wrong side.
    #[test]
    fn a_value_the_operator_gap_names_parses_as_no_number() {
        let mut interp = crate::Interp::new();
        let one = interp.text(b"1");
        let array = interp.alloc_with(
            rexx_core::BehaviourId::ARRAY,
            rexx_core::Body::array(vec![Some(one)]),
        );
        let class = interp
            .classes()
            .lookup("Object")
            .expect("the Object class is registered");
        let behaviour = interp.classes().instance_behaviour_handle(class);
        let named = interp.alloc_with(
            rexx_core::BehaviourId::OBJECT,
            rexx_core::Body::Instance {
                class,
                behaviour,
                name: Some(b"123".to_vec().into_boxed_slice()),
                pools: rexx_core::ScopePools::new(),
                own: None,
                native: None,
            },
        );
        // A stem is in the gap's set only through its default, so it needs
        // one that is itself in the set.
        let aliased = interp.alloc_with(
            rexx_core::BehaviourId::STEM,
            rexx_core::Body::Stem {
                name: b"A.".to_vec().into(),
                default: Some(array),
                tails: rexx_core::NameMap::default(),
            },
        );
        for value in [array, named, aliased] {
            let rendering = interp.to_text(value).into_owned();
            assert!(
                rexx_num::Number::parse_bytes(&rendering).is_some(),
                "{value:?} renders as {rendering:?}, which is not a number"
            );
            assert!(interp.operator_operand_gap(value).is_some(), "{value:?}");
            assert_eq!(
                interp.to_number(value),
                Err(rexx_core::NotNumeric),
                "{value:?}"
            );
        }
        // The send targets among them, which is the half `arith_left_operand`
        // rides. `array` is in the gap's set and is not one, so this is not
        // the same assertion in different words.
        assert!(interp.operator_message_receiver(named).is_some());
        assert!(interp.operator_message_receiver(array).is_none());
    }
}
