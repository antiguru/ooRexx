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
//! (Task 7) with `Stem`, `Compound`, `DotVariable`'s parse-time names,
//! `Prefix`, the arithmetic operators and the concatenation forms
//! `||` did not already cover, and (Task 8) with the comparison operators
//! (through `rexx-num`'s own comparison entry points, never a hand-written
//! string comparison), the binary logical operators `&`/`|`/`&&`, and
//! `ExprKind::Logical` (the comma-separated conditional list `IF a, b THEN`
//! desugars to), and (Task 4, 4b) `ExprKind::Call`, the internal-function
//! form (`f(...)`/`"f"(...)`) -- see `eval_call`'s own doc for the
//! resolution order and what still falls through to the loud `4c` fallback,
//! and (Task 5, 4b) `ExprKind::VariableReference`, the `>x`/`<x` form, which
//! evaluates to the referenced variable's own value -- the arm's own comment
//! has the measurement, and why `USE ARG >name` does not come through here.
//! `QualifiedCall`, `ClassResolver` and `List` still fail loudly through the
//! existing, exhaustive `form_name`. `DotVariable` resolves through
//! `environment.rs` (Phase 5a, D33), whose own doc has the order and the
//! names it refuses rather than answers.
//!
//! **A function here that opens a temps frame and then evaluates through `?`
//! leaves its own `pop_frame` unreached when that evaluation raises. That is
//! deliberate, and Tasks 10 and 11 should copy it rather than repair it.**
//! `step_in_temps_frame` pops unconditionally with an outer watermark, and
//! `pop_frame` truncates rather than popping one frame, so the skipped inner
//! frames are discarded when the failing instruction returns. Nothing
//! accumulates: an instruction is the granularity at which the temps stack is
//! guaranteed balanced, not an expression.
//!
//! **The other side of that is a frame popped on the failure path, and it is
//! equally safe.** `eval_node`'s binary arm and `eval_prefix` bind their
//! operator's result and pop before returning it, so an *operator's* own raise
//! discards everything the site rooted, where the `?` on an *operand's*
//! evaluation skips past. Both are correct for the same reason -- `pop_frame`
//! truncates to a watermark the site took itself, so it can only discard what
//! that site rooted -- plus one thing the failure path needs on its own:
//! nothing the failure carries away is a root. `Failure::Exited` is the
//! variant that carries an `ObjRef`, and neither the functions `apply_binary`
//! dispatches to nor `apply_prefix`'s own arms build anything but `Raised` and
//! `Loud`.
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

use crate::activation::CallType;
use crate::error::Raised;
use crate::run::{Ended, Resolved};
use crate::value::{canonical_small_int, exact_small_int, within_digits};
use crate::{Code, Failure, Interp, Loud, StackSpan};
use rexx_core::{Body, Decoded, INLINE_BYTES, NotNumeric, ObjRef, is_class_slot};
use rexx_num::{CompareOp, DivOp, Number};
use rexx_parse::{CallTarget, Expr, ExprKind, Operator, PrefixOp, SymbolId};

/// D19's evaluation-depth limit: `eval`'s own recursion, one level per
/// left-deep term, refuses anything past this depth with 11.1 ("Insufficient
/// control stack space") rather than letting the interpreter thread's guard
/// page abort the process silently.
///
/// **Exactly 100,000, the largest depth the oracle is measured to survive**
/// (`phase-4-exclusions.txt`'s Deviation 2: it prints an answer for a
/// 100,000-term expression and SIGSEGVs, no condition, between 100,000 and
/// 150,000). Bounded on both sides, per D19: at least 100,000 is the floor a
/// lower limit would fail (it would refuse programs the oracle accepts), and
/// `INTERPRETER_STACK_BYTES` (512 MiB) divided by this crate's own measured
/// per-level cost (`lib.rs`'s own doc comment on `INTERPRETER_STACK_BYTES` --
/// **1840** bytes/level in debug, re-measured at this task's own
/// implementation time and recorded there with the task it belongs to, the
/// method and the survivable-depth arithmetic, in the same task-anchored
/// form as every other row in that lineage rather than under a calendar
/// date; ~1600 was the figure current before this
/// task's own depth counter added its own few bytes per level) is the
/// ceiling a higher one would fail. A
/// limit *above* the oracle's own cliff would be worse than one below it: it
/// would widen the window where this crate succeeds and the oracle segfaults,
/// which is the opposite of what a differential harness wants.
///
/// `eval` checks `self.depth > MAX_EVAL_DEPTH`, never `>=`: a 100,000-term
/// expression's deepest `eval` call is depth 100,000 exactly, and that is the
/// one depth the oracle is known to handle, so the check has to fire one
/// level past it or the corpus's own passing case would be refused.
///
/// **What this does not do, and must not be documented as doing**: a program
/// can reach a deep tree without ever evaluating it (`exit` before a
/// 700,000-term expression aborts inside `Drop`, with no `eval` call and
/// this counter never in a position to see it) -- that path is closed by
/// `rexx-parse`'s iterative `Drop`, not by this counter.
const MAX_EVAL_DEPTH: usize = 100_000;

/// Which of the three bare-symbol reads an expression is, carried where the
/// `ExprKind` itself is not.
///
/// [`Interp::read_symbol`] and [`Interp::echo_symbol_read`] both dispatch on
/// this, and `crate::ir::Op::Load` carries one because a compiled op holds no
/// borrow of the node it was emitted for -- the same reason
/// `crate::ir::Op::LoopHeaderValue` carries a `HeaderRole` rather than the
/// keyword's own node.
///
/// **The three kinds it does not have are the three that are not a variable
/// read.** `ExprKind::Constant`'s value is its own upcased spelling rather
/// than anything stored, `ExprKind::DotVariable` traces `>E>` and
/// `ExprKind::VariableReference` traces `>O>`; none reaches either function.
/// The handle a Rexx logical value is, as a constant.
///
/// **A comparison's value is the one-byte string `"1"` or `"0"`, and that
/// string inlines into the handle itself** -- so the two handles are fixed bit
/// patterns and `logical` is a `const fn` rather than a call into
/// `Interp::text`, which would run `inline_text`'s loop to rediscover them.
/// Every consumer that only wants the bit can then compare two integers
/// instead of decoding a handle and matching its bytes.
pub(crate) const LOGICAL_TRUE: ObjRef = ObjRef::inline_byte(b'1');
/// See [`LOGICAL_TRUE`].
pub(crate) const LOGICAL_FALSE: ObjRef = ObjRef::inline_byte(b'0');

/// The value a comparison or logical operator answers with.
pub(crate) const fn logical(holds: bool) -> ObjRef {
    if holds { LOGICAL_TRUE } else { LOGICAL_FALSE }
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
    /// [`Interp::eval`]'s own per-node entry: the stack-span bookkeeping and
    /// D19's evaluation-depth limit, in one place because there is now more
    /// than one caller.
    ///
    /// **The second caller is the compiled stream.** A native op evaluates a
    /// node without entering `eval` at all, and for most of them that is
    /// invisible -- an `Op::Const` has no operands to recurse into, so the
    /// depth it would have counted bounds nothing. A call is different: its
    /// *arguments* go back through `eval`, so a call op that skipped this
    /// would start them one level shallower than the tree-walker does and move
    /// the depth at which a deeply nested argument raises 5.3. Calling this is
    /// what keeps the two engines' answer to that identical rather than nearly
    /// so.
    ///
    /// `anchor` is any address inside the caller's own frame; its value is
    /// never read, only its position, which is what makes the span a
    /// measurement of the real stack rather than of the recursion count.
    ///
    /// **The caller owes the matching `self.depth -= 1`** on every exit path,
    /// the raising ones included, exactly as `eval` does below. Not a guard
    /// type, because the trace hook on the way out needs the value in hand, so
    /// both halves would have to be threaded through one anyway.
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
    ///
    /// **Every arm here is additive tracing, never a second evaluation.**
    /// `expr.kind`'s own already-computed pieces (a `SymbolId`'s name, an
    /// operator's spelling) are read directly; nothing re-derives a value
    /// `eval_node` already produced.
    pub(crate) fn trace_intermediate(&mut self, code: &Code<'_>, expr: &Expr, value: ObjRef) {
        if !self.tracing_intermediates() {
            return;
        }
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
            // `echo_symbol_read`'s own doc comment has what each kind owes.
            ExprKind::Variable(id) => self.echo_symbol_read(code, SymbolRead::Simple, *id, value),
            ExprKind::Stem(id) => self.echo_symbol_read(code, SymbolRead::Stem, *id, value),
            ExprKind::Compound(id) => {
                self.echo_symbol_read(code, SymbolRead::Compound, *id, value);
            }
            // `.NIL`/`.TRUE`/`.FALSE` -- `>E>`, measured (this task's
            // report): **not** in the design spec's own "measured reachable
            // from pure-4a code" list, a correction this task found rather
            // than assumed. The other `DotVariable` names all fail loudly
            // before reaching here (`eval_node`'s own arm), so this is
            // exhaustive over what can arrive.
            ExprKind::DotVariable(id) => {
                let tag = code.symbols.name(*id).as_bytes().to_vec();
                let text = self.to_text(value).to_vec();
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
            //
            // Reached because `eval_argument` (`run.rs`) evaluates a
            // reference argument through `eval` on the *whole* reference
            // node rather than on its inner variable: doing the latter
            // traced `>V>   PQ => "val"` here instead, which is this arm's
            // own adjacent measured failure.
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
                let text = self.to_text(value).to_vec();
                self.trace_function(indent, &name, &text);
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
    ///
    /// **The three kinds are three different operations, which is why this
    /// dispatches rather than resolving a slot once and reading it.**
    ///
    /// * A simple variable's miss derives its own upcased spelling and nothing
    ///   more can ever observe the difference, so a `Body::Text` is the whole
    ///   answer.
    /// * A bare stem's miss must come back as a real, shared `Body::Stem`
    ///   (`Interp::read_stem`), because the oracle's `createStemVariable` fires
    ///   on any miss, reads included, and a read's result can be aliased (`b. =
    ///   a.` with `a.` never touched, then `a.1 = 5`, then `say b.1` -> `5`).
    ///   Rendering an unset stem alone cannot tell the two models apart -- both
    ///   give the derived name -- which is exactly how this was missed the first
    ///   time (branch review F4); aliasing is where the object's identity
    ///   becomes observable.
    /// * A compound raises `NOVALUE` on a miss exactly as a simple variable
    ///   does, measured: `signal on novalue` with `say zunset.1` traps, with
    ///   `SIGL` set to the reading clause. A **bare stem** does not -- `say
    ///   zunsetstem.` under the same trap prints the derived name and carries
    ///   on, rc unchanged -- which is why the arm below it has no
    ///   `novalue_check` and the other two do.
    ///
    /// **`at` is the slot a compiler already resolved, and `None` is the
    /// resolution every caller made before there was one.** `crate::ir::compile`
    /// reads it out of the same `Plan` this activation runs with -- the map
    /// `Code::slots` is a view of -- so the two are one answer resolved at two
    /// times rather than two answers. A compound never carries one: its read
    /// goes through the *stem's* slot and a tail key resolved at the read site,
    /// neither of which is this symbol's own slot.
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
                self.novalue_check(novalue)?;
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
                self.tail_key_into(code, id, &mut key);
                let (value, novalue) = self.stem_get_at(stem_name, stem_at, &key);
                self.give_key_buffer(key);
                self.novalue_check(novalue)?;
                Ok(value)
            }
        }
    }

    /// The `>V>` line one bare-symbol read owes, and the `>C>` line a compound
    /// owes in front of it.
    ///
    /// **The one implementation both engines enter**, for the reason
    /// [`Interp::echo_literal`] is one: `eval.rs` emits these as a side effect
    /// of evaluating the expression, and `crate::ir::Op::Load` evaluates
    /// nothing -- so `crate::ir::Op::TraceRead` emits them from a register
    /// instead, and the two must not be able to disagree about what they say.
    ///
    /// `>V>` is tagged with the symbol's own name and shows the value's text.
    /// Measured for a simple variable; a bare stem's own `>V>` is reasoned from
    /// that rather than separately probed, since both produce a tag and an
    /// already-computed value and the line shows the same two things regardless
    /// of which read produced it.
    ///
    /// `>C>` then `>V>` -- measured (`RexxActivation.cpp:4791`-`4802` read
    /// directly): a compound read always announces the fully-resolved name it
    /// used before showing what is stored there, whether or not the tail
    /// actually resolves. The tag is the compound's own *unresolved* source
    /// spelling (e.g. `A.I`); the resolved name is `Code::stem_name`'s answer
    /// -- the read site's own -- concatenated with `tail_key`'s output, which
    /// matches `stem_get`'s own answer exactly when the read site and the stem
    /// object's own name agree, and diverges from it only through aliasing: a
    /// known, narrow gap, not silently assumed correct.
    ///
    /// The gate is asked before anything is rendered, for the reason
    /// `Interp::echo_literal` asks it there: rendering allocates a copy of a
    /// value of any size, and an untraced run must not pay for it.
    #[inline(always)]
    pub(crate) fn echo_symbol_read(
        &mut self,
        code: &Code<'_>,
        read: SymbolRead,
        id: SymbolId,
        value: ObjRef,
    ) {
        if !self.tracing_intermediates() {
            return;
        }
        self.echo_symbol_read_line(code, read, id, value);
    }

    /// The name-building and the lines, out of line behind
    /// [`Interp::echo_symbol_read`]'s gate.
    #[inline(never)]
    fn echo_symbol_read_line(
        &mut self,
        code: &Code<'_>,
        read: SymbolRead,
        id: SymbolId,
        value: ObjRef,
    ) {
        let indent = self.clause_state.current_value_indent;
        let tag = code.symbols.name(id).as_bytes().to_vec();
        if read == SymbolRead::Compound {
            // The slot is not wanted here: this builds the printed name
            // and reads nothing.
            let (stem_name, _) = code.stem(id);
            let mut resolved = stem_name.to_vec();
            resolved.extend_from_slice(&self.tail_key(code, id));
            self.trace_compound_name(indent, &tag, &resolved);
        }
        let text = self.to_text(value).to_vec();
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
            //
            // The interned spelling keeps its leading period and is upcased
            // (`scanner.rs`'s symbol capture includes the whole `.NAME` span
            // before interning), so the match is against
            // `.NIL`/`.TRUE`/`.FALSE`, not `NIL`/etc, and `dot_variable`
            // takes the same dotted, upcased spelling.
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
                // evaluation above leaves them for `step_in_temps_frame` to
                // truncate. The module doc has why both are safe.
                //
                // The result is unrooted from `apply_binary`'s return to
                // whatever the caller of this does with it, and nothing
                // between the two allocates.
                let result = self.apply_binary(*op, left_value, right_value);
                self.roots.pop_frame(frame);
                result
            }

            // The comma-separated conditional list (`IF a, b THEN`, and
            // `WHEN`/`GUARD`/`WHILE`/`UNTIL`'s own versions of the same
            // syntax) -- see `eval_logical_list`'s own doc comment for the
            // short-circuit and sub-number this arm alone can give it.
            ExprKind::Logical(items) => self.eval_logical_list(code, items),

            // `>name`/`<name` in an ordinary value position **decays to the
            // referenced variable's value**, measured: `p = 'orig'; say >p`
            // prints `orig`, and `call sub2 >p` into a plain (non-`>`) `use
            // arg q` binds `orig` and leaves the caller's `p` untouched. So
            // there is no reference *object* to build here.
            //
            // What makes `>` more than a no-op is `USE ARG >name`, and that
            // path never reaches this arm: `Interp::invoke_call` evaluates
            // a call's arguments through `eval_argument` instead, which
            // keeps the caller's slot alongside this same value. Only an
            // argument written `>something` at the call site carries one,
            // which is why passing a plain symbol to `use arg >q` is error
            // 88.928 rather than silently aliasing something.
            ExprKind::VariableReference(inner) => self.eval_node(code, inner),

            // `f(...)`/`"f"(...)` (Task 4, 4b) -- see `eval_call`'s own doc.
            ExprKind::Call { target, args } => self.eval_call(code, target, args),

            // `target~name(...)`, `target~~name(...)` and `target[...]`
            // (Phase 5a) -- see `Interp::message_term` (`dispatch.rs`) for
            // the evaluation order and `Interp::resolve`/`Interp::invoke`
            // for the send itself. The **expression** form only: the
            // message-assignment form is an instruction and never an
            // expression, so `assigned` is `None` here.
            //
            // **91.999 is the expression position's own error and not the
            // send's**, which is why it is raised here rather than inside
            // `message_term`: measured, `::method m class` ending in a bare
            // `return` is rc 0 as a whole clause and 91.999 at rc 165 under
            // `say`.
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

            other => Err(Loud::expression(other).into()),
        }
    }

    /// `ExprKind::Call`: the internal-function form, evaluated for its
    /// value rather than run as a clause of its own.
    ///
    /// **Resolution is four steps -- internal label, builtin, `::ROUTINE`,
    /// then an external Rexx file -- and none of them is in this function.**
    /// `eval_call` only decides the two inputs a `CallTarget` reduces to
    /// (`name`, `search_labels`) below and hands them to `Interp::resolve_call`
    /// (`run.rs`), which both this function and `exec_call` (`CALL`) share, and
    /// which owns all four steps: the label search
    /// (`activation_body.labels.get(name)`), the builtin table, the
    /// `::ROUTINE` lookup, and the 43.1 that stands in for the file search
    /// this crate does not do. Only the fourth is deferred, to **Phase 7**.
    /// Keeping them there rather than here is what stops `CALL length 'abc'`
    /// and `say length('abc')` answering differently. `eval_call` itself owns
    /// no expression-only resolution step; the only thing specific to this
    /// call form is what happens *after* `Interp::invoke_call` returns
    /// (`Ended`'s three cases, below), which `CALL` does not need because it
    /// never produces a value for an enclosing expression to use.
    ///
    /// **The two halves are entered here rather than through their
    /// composition, and uncached**: an expression call has no name of its own
    /// to remember an answer under, where a compiled `CALL` site has its op
    /// position (`crate::ir::Op::Call`).
    ///
    /// **`CallTarget::Literal` never searches the label table, symmetric
    /// with `CALL "SUB"` (Task 3).** Its own doc in `rexx-parse` already
    /// says so; confirmed here, on the oracle, in a clean directory (the
    /// scratchpad root is on the external-routine search path and a stale
    /// `f.rex` there gives a different, wrong answer): with an internal
    /// `f:` label present, `say f(1)` runs it, while `say "f"(1)` is Error
    /// 43.1 rc 213, "Routine not found". **That is this crate's answer too**
    /// (`search_labels = false` below): with the builtin and `::ROUTINE`
    /// steps behind the label search built, "not a label" and "not anything"
    /// are separable, so the condition is the oracle's own rather than a
    /// fabricated one. `a_literal_call_target_never_reaches_the_label_table`
    /// asserts exactly that, five lines of test below this paragraph.
    ///
    /// **`RESULT` is never touched here**, unlike `CALL`: measured, a
    /// caller's `RESULT` is unaffected by `f(1)` appearing in an
    /// expression. Nor does this emit `TRACE I`'s own `>F>`/`>A>` lines --
    /// `eval`'s own `trace_intermediate` hook has no arm for `ExprKind::
    /// Call` yet, and Task 9 is who adds one; this function must not
    /// anticipate it.
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
    ///
    /// **Split so the compiled stream can supply a resolution it kept rather
    /// than making a fresh one**, and split rather than copied because the
    /// three `Ended` arms below are measured behaviour -- 44.1 in particular
    /// is the expression form's own answer and `CALL` has no equivalent -- and
    /// a second copy of them is one free to stop agreeing.
    ///
    /// [`eval_call`]: Interp::eval_call
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
        match self.invoke_call(code, resolved, name, args, CallType::Function)? {
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
    ///
    /// The operand prologue alone: what the operator then does with one value
    /// is [`Interp::apply_prefix`], entered from here and from
    /// `crate::ir::Op::Prefix`.
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
        //
        // The result is unrooted from `apply_prefix`'s return to whatever the
        // caller of this does with it, and nothing between the two allocates.
        let result = self.apply_prefix(op, value);
        self.roots.pop_frame(frame);
        result
    }

    /// `op value` for the prefix operators `+`, `-` and `\`.
    ///
    /// **The one dispatch both engines enter**: `eval_prefix` above evaluates
    /// the operand out of the tree and `crate::ir::Op::Prefix` reads it out of
    /// a register, and everything past that point is this function, so the two
    /// cannot come to disagree about what a prefix operator answers.
    /// [`Interp::apply_binary`]'s arrangement, with one operand.
    ///
    /// `+`/`-` are arithmetic -- measured, `numeric digits 1
    /// ; say -12345` gives `-1E+4`, the same rounding `0 - 12345` gives, so
    /// they are implemented as exactly that rather than a sign flip on the
    /// operand's own digits. `\` is a **text** check, never a numeric one:
    /// measured, `say \'abc'` is 34.901, not 41.1, so a non-numeric operand
    /// is not converted first and does not fail as "nonnumeric".
    ///
    /// **The operand must already be rooted by the caller**, for the reason
    /// [`Interp::concat_values`] states: both arms below allocate the value
    /// they answer with.
    pub(crate) fn apply_prefix(&mut self, op: PrefixOp, value: ObjRef) -> Result<ObjRef, Failure> {
        let result = match op {
            PrefixOp::Plus | PrefixOp::Minus => {
                let number = self.arith_left_operand(op.spelling(), value)?;
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
                    // `\.array` is 97.1 on the oracle, the same message send
                    // the dyadic operators make.
                    None => {
                        if let Some(kind) = self.operator_operand_gap(value) {
                            return Err(Loud::operator_operand(op.spelling(), kind).into());
                        }
                        return Err(Raised::not_logical(&text).into());
                    }
                };
                self.text(flipped)
            }
        };
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
    ///
    /// Both operands already integers small enough to tag, and an operator
    /// whose exact result is an integer too: [`Interp::arith_general`] is a
    /// detour through a representation neither operand is in and the result
    /// does not need. It is a detour that allocates -- `to_number` renders a
    /// `SmallInt` to a `String` and reparses it, `add` builds a digit `Vec`,
    /// and `number` renders that back to a `String` to decide the result is a
    /// small integer after all -- so what this skips is five allocations, not
    /// five instructions.
    ///
    /// [`small_int_arith`] answers `None` for every case where the two paths
    /// could disagree, and `exact_small_int`'s own doc comment has why the
    /// remaining ones cannot.
    ///
    /// **This is the whole of what `crate::ir::Op::Arith`'s quickened arm
    /// runs, entered from there and from `eval_arithmetic` above**, so the
    /// hint that arm reads decides only whether this is *tried*, never what it
    /// answers.
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
    ///
    /// **Both operands must already be rooted by the caller**, because
    /// everything below allocates: `eval_arithmetic` pushes them as temps of
    /// the frame it opened, and `crate::ir::Op::Arith` has them in registers,
    /// which are roots of the region `Interp::run_chunk` reserved.
    ///
    /// The settings are read here rather than passed in, so that an operation
    /// computes under the ones in force at the moment it runs -- D15's rule,
    /// and the reason a caller holding a `digits` from before cannot supply it.
    pub(crate) fn arith_general(
        &mut self,
        op: Operator,
        left_value: ObjRef,
        right_value: ObjRef,
    ) -> Result<ObjRef, Failure> {
        let digits = self.activation().settings.digits();
        let form = self.activation().settings.form();

        let left_number = self.arith_left_operand(op.spelling(), left_value)?;

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

        Ok(self.number(result, saturate_digits(digits), form))
    }

    /// Converts an arithmetic operand to a `Number`, or 41.1 with the
    /// operand's own rendered text as the substitution -- measured, `say
    /// 'abc' + 1` reports `Nonnumeric value ("abc")`, the operand as it
    /// renders, not upcased or otherwise transformed.
    ///
    /// `pub(crate)` since Task 11: a controlled `DO`/`LOOP`'s own
    /// `initial`/`TO`/`BY` need exactly this conversion (measured, `do i =
    /// 'a' to 3` is the identical 41.1 an arithmetic operand's own failure
    /// is) and `run.rs` is where that header is evaluated, not here --
    /// reused rather than a second copy of the same three lines.
    pub(crate) fn arith_operand(&mut self, value: ObjRef) -> Result<Number, Failure> {
        match self.to_number(value) {
            Ok(number) => Ok(number),
            Err(NotNumeric) => {
                let text = self.to_text(value).to_vec();
                Err(Raised::nonnumeric(&text).into())
            }
        }
    }

    /// [`Interp::arith_operand`] for the operand the operator is *sent to*.
    ///
    /// **The asymmetry is the oracle's and is measured**: `.array + 1` is 97.1
    /// where `1 + .array` is 41.1 quoting `"The Array class"`, which this
    /// crate already answers identically. So only the left operand -- and a
    /// prefix operator's only one -- can carry this gap *for an operator*, and
    /// the right one keeps the ordinary 41.1. A controlled `DO` header is not
    /// an operator and does not follow that rule: `Interp::header_number`
    /// checks every position, because each is rounded through a unary
    /// operator of its own.
    ///
    /// Entirely on the failing path: an object of either shape is
    /// [`NotNumeric`] whatever this decides, so a program doing arithmetic on
    /// numbers never reaches the test.
    fn arith_left_operand(&mut self, op: &str, value: ObjRef) -> Result<Number, Failure> {
        match self.to_number(value) {
            Ok(number) => Ok(number),
            Err(NotNumeric) => {
                if let Some(kind) = self.operator_operand_gap(value) {
                    return Err(Loud::operator_operand(op, kind).into());
                }
                let text = self.to_text(value).to_vec();
                Err(Raised::nonnumeric(&text).into())
            }
        }
    }

    /// The shared body of `||`/`Abuttal` (no separator) and `Blank` (one
    /// space).
    ///
    /// `Blank` inserts exactly one space, regardless of how much whitespace
    /// separated the terms in source -- measured, `'a'  'b'` is `a b` whether
    /// one space or several sit between them in the original text, because the
    /// scanner has already collapsed that distinction into "this is a Blank
    /// operator" before the parser ever sees it. That is why the separator is a
    /// caller's argument and not read off the source span.
    ///
    /// **One byte or none, rather than a slice**, and the width is the reason:
    /// a slice of run-time length is joined by a call into `memcpy`, and the
    /// separator this function is handed most often is the empty one, which
    /// pays that call to copy nothing. `Blank`'s single space is the widest
    /// there is for the type to have to hold.
    ///
    /// **Both operands must already be rooted by the caller**, because the join
    /// below allocates and a value held only in a Rust local across an
    /// allocation is invisible to the collector. [`Interp::arith_general`]'s
    /// contract, for the same reason: `eval_node` pushes them as temps of the
    /// frame it opened, and `crate::ir::Op::Binary` has them in registers,
    /// which are roots of the region `Interp::run_chunk` reserved.
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
        //
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
    ///
    /// **Both operands must already be rooted by the caller**, for the reason
    /// [`Interp::concat_values`] states: the result value below allocates.
    ///
    /// Calls `to_number` for the non-strict family only, and never for
    /// strict operators, which never inspect a `Number` at all
    /// (`CompareOp::is_strict`'s short-circuit, `compare.rs`) -- asking for
    /// one anyway would force a needless parse of an operand
    /// `==`/`>>`/... never numerically compares. When `to_number` is
    /// called, it already routes through `Body::Text`'s tri-state `num`
    /// cache (`value.rs`), so an operand already asked about is not
    /// reparsed -- keeping the parse on this side of the call is what the
    /// plan's "do not quietly defeat the cache by using the `&str` entry
    /// point" is about, not a prohibition on calling `to_number` at all.
    fn compare_values(
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
        if let Some(holds) = small_int_compare(op, left_value, right_value, digits) {
            return Ok(logical(holds));
        }
        let strict = is_strict_compare(op);
        let left_number = if strict {
            None
        } else {
            self.to_number(left_value).ok()
        };
        // **Behind the fast path above, because a comparison of renderings
        // never fails and so offers nothing to ride.** Measured, comparing an
        // object against the very text it renders as answers `0` on the
        // oracle and `1` here; see `Loud::operator_operand`.
        //
        // **Behind the left operand's own parse as well, and the assertion is
        // what makes that safe rather than the argument for it.** Every shape
        // this gap names -- a class identity, one of the interpreter's own
        // objects, and a stem redirecting to either -- is a shape
        // `Interp::to_number` answers `NotNumeric` for, so a left operand that
        // produced a `Number` has no gap to report and the lookup this costs
        // is a heap fetch on a value already known to be a number. The
        // ordering the skip must not disturb is the *error's*, and it is
        // undisturbed: nothing between here and the original position can
        // fail, and the arm that could -- `compare_numbers` -- is reached only
        // when both operands parsed, which is exactly when there is no gap.
        debug_assert!(
            left_number.is_none() || self.operator_operand_gap(left_value).is_none(),
            "a left operand that parsed as a number reported an operator gap"
        );
        if left_number.is_none()
            && let Some(kind) = self.operator_operand_gap(left_value)
        {
            return Err(Loud::operator_operand(op.spelling(), kind).into());
        }
        let right_number = if strict {
            None
        } else {
            self.to_number(right_value).ok()
        };

        // **Both operands parsed means the bytes have no reader**, so they are
        // not produced. `compare_decoded`'s own `(Some, Some)` arm is
        // `compare_numbers` and nothing else, and it is handed the same two
        // values this would have passed it -- so this is where the rendering
        // happens rather than what the comparison answers. What it skips is
        // `render` plus `text` on both sides, whose result reaches only the
        // string fallback below.
        //
        // Measured on `samples/rexxcps.rex`: 2,520,002 comparisons reach this
        // point and 1,680,001 of them take this arm, which is 3,360,002
        // renderings not performed.
        if let (Some(left), Some(right)) = (&left_number, &right_number) {
            let holds = rexx_num::compare_numbers(left, right, digits, fuzz, compare_op(op))
                .map_err(Raised::from)?;
            return Ok(logical(holds));
        }
        // Either operand failed to parse, or the operator is strict and neither
        // was parsed at all. Both routes compare the operands' own text, which
        // is what these two renderings are for.
        //
        // **`compare_strings` rather than `compare_decoded`, because a `None`
        // here is an answer and not a question.** `compare_decoded` reads a
        // `None` as "parse this one from the bytes", which is right for a
        // caller that has not tried -- and this one has: an operand is `None`
        // exactly when [`Interp::to_number`] already refused it, or when the
        // operator is strict and no `Number` is ever consulted. Handing that
        // `None` on buys the refused parse a second time, over the same
        // bytes, to reach the same string fallback.
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
            strict || left_number.is_some() || Number::parse_bytes(left_bytes).is_none(),
            "a left operand to_number refused parses from its own rendering"
        );
        debug_assert!(
            strict || right_number.is_some() || Number::parse_bytes(right_bytes).is_none(),
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
    ///
    /// **Both operands must already be rooted by the caller**, for the reason
    /// [`Interp::concat_values`] states: the result value below allocates.
    fn logical_values(
        &mut self,
        op: Operator,
        left_value: ObjRef,
        right_value: ObjRef,
    ) -> Result<ObjRef, Failure> {
        let left_text = self.to_text(left_value).to_vec();
        let left_bool = match logical_value(&left_text) {
            Some(value) => value,
            // The oracle sends `&`/`|`/`&&` to the left operand as a message
            // -- `.array & 1` is 97.1 where this crate's own 34.901 quotes
            // the object's rendering.
            None => {
                if let Some(kind) = self.operator_operand_gap(left_value) {
                    return Err(Loud::operator_operand(op.spelling(), kind).into());
                }
                return Err(Raised::not_logical(&left_text).into());
            }
        };
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

    /// The noun for an operand no operator here can take, or `None` for one
    /// every operator can.
    ///
    /// The shapes are a class object and one of the interpreter's own
    /// objects, and neither is a value the oracle treats as text when an
    /// operator meets it. [`Loud::operator_operand`] carries the measurements
    /// and [`Loud::object_position`] the surfaces that are not operators.
    ///
    /// **Every caller reaches this on a path that was already failing, except
    /// [`Interp::compare_values`] and `Interp::header_number`**: a comparison
    /// of renderings always succeeds, so the first sits behind the
    /// two-small-integer fast path where a loop bound never pays for it, and
    /// the second runs once per loop entry rather than once per iteration.
    ///
    /// Takes one operand and allocates nothing, so it carries no rooting
    /// precondition of its own.
    pub(crate) fn operator_operand_gap(&self, value: ObjRef) -> Option<&'static str> {
        // A small integer, an inline string and `.nil` all leave on this
        // line: only a heap-tagged handle can be either shape.
        let Decoded::Heap { slot, generation } = value.decode() else {
            return None;
        };
        if is_class_slot(slot, generation) {
            return Some("a class object");
        }
        match &self.heap.get(value)?.body {
            Body::Native(_) => Some("one of the interpreter's own objects"),
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
            _ => None,
        }
    }

    /// `left op right` for every binary operator whose two operands are just
    /// values by the time it runs -- concatenation, comparison and logical.
    ///
    /// **The one dispatch both engines enter**: `eval_node`'s own binary arm
    /// evaluates the operands out of the tree and `crate::ir::Op::Binary` reads
    /// them out of registers, and everything past that point is this function,
    /// so the two cannot come to disagree about what an operator answers.
    ///
    /// Arithmetic is **not** here and keeps [`Interp::eval_arithmetic`]:
    /// `**`'s exponent is not converted the way its base is, so it does not
    /// share the operand handling the families below do, and its
    /// compiled site carries a quickening hint no other operator has.
    ///
    /// **Both operands must already be rooted by the caller**, for the reason
    /// [`Interp::concat_values`] states.
    pub(crate) fn apply_binary(
        &mut self,
        op: Operator,
        left: ObjRef,
        right: ObjRef,
    ) -> Result<ObjRef, Failure> {
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
    ///
    /// **Short-circuits on the first element that checks out false**,
    /// unlike `&` (`logical_values`' own doc comment). Measured with `if 0,
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
    ///
    /// **F4, found by review: every element traces its own `>>>`, under
    /// `TRACE R` alone, not only `TRACE I`.** Measured: `trace r` /
    /// `if 1, 1 then say 'x'` gives *three* `>>>` lines --  one per
    /// element (`"1"`, `"1"`) and one more for the list's own overall
    /// result (`"1"`), the third of which was already covered (`eval_
    /// condition`'s own `ConditionTrace::Result`/`Keyword` fires on
    /// whatever this function returns, list or not). The two missing
    /// lines are `trace_result`, **not** an intermediate: comma-list
    /// elements trace through the same `results`-gated path a traced
    /// instruction's own top-level value does (matching `test_case_when`'s
    /// own per-value `>>>` pair, `SELECT CASE`'s comma list, the other
    /// place this crate already has this exact shape), not through
    /// `eval`'s `intermediates`-gated hook -- which is *why* `TRACE R`
    /// alone already shows it on the oracle, and confirms this is not a
    /// second, competing computation of anything `eval`'s own hook
    /// already produces: an element already gets its own `>L>`/`>V>`/
    /// `>O>` etc. under `TRACE I` from that hook (unaffected, still
    /// correct), and *additionally* gets this `results`-gated line,
    /// exactly as `IF`'s own condition gets both an intermediate trace
    /// for its sub-expressions and its own separate `>>>`.
    ///
    /// Fixes `IF`/`WHEN`/`WHILE`/`UNTIL` **all four** in this one place,
    /// since every one of them reaches a comma-list condition only
    /// through this shared function (`eval_node`'s `ExprKind::Logical`
    /// arm, the desugaring point) -- confirmed, not assumed: re-ran the
    /// oracle differential for `IF` with this exact fix in place, byte
    /// for byte (this task's report has the transcript).
    ///
    /// **The failing element still traces before it raises**: measured,
    /// `if 1, 'x' then nop` shows `>>>   "1"` then `>>>   "x"` and only
    /// then 34.6 -- the trace call sits ahead of the `logical_value`
    /// check below, matching that order exactly.
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
///
/// Unreachable in practice, which is the reason saturation is the right
/// choice rather than a guess dressed up as one: `u32::MAX` is about four
/// billion significant figures, and a program that set `NUMERIC DIGITS`
/// anywhere near that would exhaust memory building a single `Number`
/// (`rexx-num` reserves working storage proportional to `DIGITS`) long
/// before precision ever mattered. No corpus program, and no realistic
/// one, can reach the clamp.
///
/// `pub(crate)` since Task 11: a controlled `DO`/`LOOP`'s own control
/// variable is created through `Interp::number` exactly like any other
/// arithmetic result (`run.rs`'s `loop_advance`), and needs the identical
/// narrowing -- reused from here rather than copied.
/// `left op right` as a tagged small integer, or `None` when the general
/// arithmetic path must run instead.
///
/// All seven arithmetic operators are here, each admitted only where its
/// exact `i64` answer is the answer the interpreter gives:
///
/// * `+`, `-` and `*` take two integers to an integer outright.
/// * `%` truncates toward zero and `//` takes the dividend's sign, which is
///   what `i64`'s own `/` and `%` do -- measured, `-7 % 3` is `-2`, `-7 // 3`
///   is `-1` and `7 // -3` is `1`. Neither can need more room than the
///   operand guard below has already allowed: `|left % right|` is at most
///   `|left|` and `|left // right|` is below `|right|`, both of which that
///   guard accepted.
/// * `/` is the one of the seven whose exact result need not be an integer,
///   so it is admitted only when the division leaves no remainder -- `6 / 2`
///   is, `1 / 3` is not.
/// * `**` is admitted for a non-negative exponent whose exact power fits both
///   the tag and `digits`. `NumberString::power` reduces the exponent
///   bitwise, so every intermediate is `base` raised to a prefix of that
///   exponent and therefore no wider than the result itself; it also works at
///   `digits` plus the exponent's own digit count plus one. So a result that
///   needs no rounding is reached without any, and the exact answer is the
///   interpreter's answer.
///
/// **Every case this declines is answered by the general path, so a decline
/// costs speed and never an answer** -- which is why the guards below are
/// free to be stricter than the interpreter wherever stating the exact
/// condition would be harder than the fast path is worth.
///
/// The `checked_*` forms carry two different jobs. On `*` and `**` they are
/// the overflow test, and it is reachable. On `/`, `%` and `//` they are how
/// a **zero divisor** declines, so the 42.3 the general path raises is still
/// what a program sees. On `+` and `-` they are neither: two operands inside
/// the tag cannot overflow `i64`, and the checked form is there only so the
/// arms read alike.
///
/// **Both operands are checked against `digits` before the operation, not
/// just the result afterwards.** Rexx rounds the operands too, so an operand
/// too wide for the precision makes the exact `i64` answer the wrong one --
/// see [`exact_small_int`]'s own doc comment for the measured pair. `**` does
/// *not* round its base (`prepareOperatorNumber` is called there with
/// `NOROUND`), so for that operator the shared guard is stricter than the
/// interpreter rather than matching it.
/// [`Interp::arith_small_int`] for a pair that is not two tagged integers,
/// where at least one operand has to be read out of its bytes.
///
/// **Outlined, and the `match` above never falls into it by accident.** Two
/// tagged operands are the common pair and answer without touching this;
/// everything else arrives here, most of it to be declined. Keeping the
/// scan out of the caller is what stops a clause that cannot use it from
/// paying for it -- measured, folding this into the caller cost between 0.3%
/// and 2.2% more instructions on the four fixed-work benchmark axes.
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
///
/// **A string that spells an integer canonically is that integer**, and
/// [`Interp::literal`] says so already: a source literal whose bytes are
/// exactly some integer's own rendering starts life tagged. A value reaching
/// arithmetic as [`Decoded::Text`] instead has usually come back from a
/// builtin, which does not apply that test -- `substr` answers a string --
/// and it is the same value either way. [`canonical_small_int`] is the same
/// test, so what it admits parses to exactly what `Number::from_i64` would
/// build from the tag; anything it refuses, `05` and `+5` and ` 5 ` among
/// them, falls to the general path where the bytes decide.
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
///
/// A negative exponent leaves the integers -- `2 ** -1` is `0.5` -- and is
/// declined by the conversion rather than by a test of its own. So is an
/// exponent past [`u32`], which no base but `0`, `1` and `-1` could survive
/// anyway; those three would be exact, and they go to the general path with
/// everything else rather than earning an arm of their own.
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
///
/// **The guard on `eval_node`'s own arithmetic arm, and so the one enumeration
/// of the set** -- `crate::ir::compile` decides whether an expression compiles
/// to `crate::ir::Op::Arith` by asking this, rather than by repeating the list
/// where nothing would notice the two drifting apart. A compiler that promoted
/// one operator more than this would run arithmetic on a concatenation.
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
///
/// **The guard on `Interp::apply_binary`'s own comparison arm, and so the one
/// enumeration of the set** -- `compare_op` translates each of these to a
/// `rexx-num` `CompareOp`, and an operator in one list and not the other is a
/// comparison that reaches the translation with nothing to translate to.
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

/// Whether `op` is one of the operators [`Interp::logical_values`] computes.
fn is_logical(op: Operator) -> bool {
    use Operator::*;
    matches!(op, And | Or | Xor)
}

/// Whether `op` has a native op of its own, which is what licenses
/// `crate::ir::compile` compiling it to registers rather than leaving its
/// whole expression to `crate::ir::Op::EvalExpr`.
///
/// **A positive enumeration of the families rather than "everything but the
/// prefix `\`"**, so that an operator added to `rexx_parse::Operator` is
/// not promotable until somebody says it is: the compiler would otherwise emit
/// an op for it, and the driver would reach `Interp::apply_binary` with no arm
/// to answer from.
pub(crate) fn is_native_binary(op: Operator) -> bool {
    is_arithmetic(op) || is_concatenation(op) || is_comparison(op) || is_logical(op)
}

/// The bytes a call target names, and whether an internal label may answer it.
///
/// One place rather than two, because the pair is what decides which routine
/// runs: `CALL 'MAX'` skipping the internal `max:` label is measured
/// behaviour, and the compiled stream has to reach the same answer as
/// `eval.rs` from the same node.
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
///
/// `None` means they do not and the general path must run.
///
/// **This is `RexxInteger::comp` (`interpreter/classes/IntegerClass.cpp:1191`)
/// and not an optimisation of the path below it.** Two integers that both fit
/// `NUMERIC DIGITS` are subtracted directly there, and `NUMERIC FUZZ` -- which
/// only ever enters through `NumberString::comp` -- is never consulted for
/// them. Measured at `DIGITS 9 FUZZ 8`: `100000000 = 100000001` is `0`, while
/// the same pair spelled `100000000.0 = 100000001`, whose left operand is no
/// longer an integer, is `1`. Taking the general path here for a non-zero
/// `FUZZ` would answer `1` for both.
///
/// The guards are each a case where comparing the integers would give a
/// different answer from what the interpreter does, so none of them is
/// defensive.
///
/// * **Both magnitudes must sit inside `NUMERIC DIGITS`**, which is
///   `Numerics::isValid`'s test on either side of that `&&`. Outside it the
///   interpreter falls to `NumberString::comp` and the operands are rounded to
///   that many significant digits first, so at `DIGITS 9` two distinct
///   ten-digit integers compare equal.
/// * **The strict *ordering* operators are excluded**, because they compare
///   strings and not numbers: `9 >> 10` is true where `9 > 10` is false.
///   Strict *equality* is included, because a small integer renders
///   canonically -- no sign on zero, no leading zeros, no exponent -- so two
///   equal renderings mean equal values and the converse holds too.
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
    ///
    /// The refusals are the load-bearing half. A Rexx value's identity is its
    /// bytes, and `05`, `+5` and `5.0` are numerically five but render as
    /// themselves -- so they must reach the general path, where the bytes
    /// decide, rather than being folded into a tag that would render `5`.
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
    ///
    /// Compared against the general path's own output rather than against a
    /// table of expected strings written here. A table would pin the fast
    /// path to my reading of Rexx's rounding rule, and that reading is
    /// exactly what was wrong: a first version of `small_int_arith` checked
    /// only the result against `DIGITS` and answered `975` for `1000 - 25`
    /// at `DIGITS 3`, where the interpreter answers `980`. The general path
    /// is `rexx-num`, which is differentially validated against the oracle;
    /// agreeing with it is the property worth asserting.
    ///
    /// **The `expect` on the general path's own result is an assertion, not
    /// a convenience.** A fast path that accepted an operation the general
    /// path raises on -- a zero divisor, an exponent that overflows -- would
    /// answer where the interpreter reports 42.3 or 26, and that is the shape
    /// this line fails on.
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
    ///
    /// Measured on the interpreter: `-7 % 3` is `-2` (truncated toward zero,
    /// not floored to `-3`), `-7 // 3` is `-1` and `7 // -3` is `1` -- the
    /// remainder takes the *dividend's* sign, not the divisor's.
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
    ///
    /// `1000 - 25` at `DIGITS 3` is `980` on the interpreter, not `975`:
    /// `1000` needs four significant digits, so it is rounded before the
    /// subtraction and the `5` falls off the end (`ootest`'s
    /// `SUBTRACTION::test_147`). The fast path must decline it. `100 - 25`
    /// differs only in the operand's width and must not be declined --
    /// without this half, a guard that refused every subtraction would pass.
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
    ///
    /// Asserting them together is what makes each mean something. A build that
    /// always fell back would answer `.ARRAY` with its own text; one that was
    /// always loud would refuse `.FOO`, which the oracle answers at rc 0.
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
        //
        // **This was a transcription loss, not a measurement gap, and an
        // earlier version of this comment blamed the wrong step.** Task 8's
        // report does carry equal-operand rows: `'9' <<= '9'` is there and
        // would have exposed the `<<=` mutation, and `'9' \== '9'` is an
        // equal-operand negated form. Both were measured and neither reached
        // a test. So the oracle work was sound and the loss happened between
        // the report and the assertions, which is the more likely failure of
        // the two and the one worth guarding: check a report's own table
        // against the test that claims to encode it.
        //
        // Each value below was re-measured against `build/bin/rexx` before
        // being written here rather than derived from the mapping it checks.
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
    ///
    /// **Not nested parentheses.** `rexx-parse`'s own `MAX_EXPR_DEPTH` is
    /// 50,000 and raises the identical 11.1 from the *parser*, before
    /// `eval` ever runs -- a depth test built that way would go green
    /// without this counter firing at all, which is exactly the trap this
    /// task's own brief warns about. A flat chain of same-precedence binary
    /// operators does not increase parser recursion the way a nested
    /// construct does (the precedence-climbing loop that assembles it does
    /// not recurse per term), so 100,000 (and 100,001) terms here never
    /// come near that other limit.
    fn chain(terms: usize) -> Vec<u8> {
        let mut program = b"say 'a'".to_vec();
        for _ in 1..terms {
            program.extend_from_slice(b"||''");
        }
        program.push(b'\n');
        program
    }

    /// The engine both boundary tests below run their chain on, and the
    /// choice is a statement about the subject rather than a convenience.
    ///
    /// `MAX_EVAL_DEPTH` is `eval`'s own counter. `crate::ir::compile` promotes
    /// a chain of native operators to ops that reach the operator with its
    /// operands already in registers, so on the compiled engine this program
    /// never enters `eval` and there is no depth for the counter to count.
    ///
    /// **What that costs is stated rather than implied: the limit is the
    /// tree-walker's, and the compiled engine runs a chain past it.**
    /// Measured -- `say 'a'` followed by 100,000 `||''` is rc 245 on the
    /// tree-walker and rc 0 on the compiled one, and `say 1` followed by
    /// 100,000 `+0` was already that pair before any operator but arithmetic
    /// was promoted.
    fn depth_limited() -> crate::Invocation {
        crate::Invocation::none().with_engine(crate::Engine::TreeWalker)
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
    ///
    /// **Confirming this reaches `MAX_EVAL_DEPTH` and neither of the two
    /// other limits nearby.** Not the parser's own 50,000 (`chain`'s own
    /// doc comment: no nested construct, so no parser recursion to hit).
    /// Not the native guard page either: printed and checked directly, both
    /// halves of this pair report `outcome.stack.max_depth` (via a `dbg!`
    /// run by hand while writing this test, since the assertion below only
    /// needs the boundary case to hold) equal to `MAX_EVAL_DEPTH` and
    /// `MAX_EVAL_DEPTH + 1` respectively -- exactly one term more between
    /// the two, and both values sit at roughly 1600 bytes/level *
    /// 100,000 ~= 160 MB into the 512 MiB stack, nowhere near its own
    /// cliff (`INTERPRETER_STACK_BYTES`'s own doc comment: ~335,000
    /// survivable levels at that per-level cost).
    #[test]
    fn eval_survives_exactly_max_eval_depth_terms_and_prints_the_oracles_own_answer() {
        let outcome = crate::run_program(
            "depth-boundary-at.rex",
            chain(MAX_EVAL_DEPTH),
            depth_limited(),
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
            chain(MAX_EVAL_DEPTH + 1),
            depth_limited(),
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
    ///
    /// The witness is a name no table can ever hold, not a builtin waiting
    /// its turn: the builtin step reads `rexx_inventory`'s own name set, so
    /// any real builtin here would assert where the implemented boundary sits
    /// and go red the day that name landed. Where the boundary sits is
    /// `corpus/builtin-status.txt`'s to record, over every builtin at once.
    ///
    /// Measured on the oracle in a clean directory, which is the only place
    /// this answer is stable: `call zorkolo` reports 43.1 rc 213 there and
    /// runs a stale `zorkolo.rex` at rc 0 in a directory that has one.
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
        assert!(
            String::from_utf8_lossy(&outcome.stderr).contains("4c"),
            "stderr: {:?}",
            String::from_utf8_lossy(&outcome.stderr)
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
///
/// A separate module because every case here runs a whole program through
/// both engines rather than driving `eval` against a hand-built activation,
/// which is what the module above is for.
#[cfg(test)]
mod object_operand_tests {
    use crate::{Engine, Invocation, run_program};

    /// Runs `source` on both engines and hands back `(exit code, stdout,
    /// stderr)`, having first insisted the two engines agree with each other.
    ///
    /// **Both engines, because the check lives in `apply_binary`,
    /// `apply_prefix`, `arith_general` and `compare_values`, all four of
    /// which `crate::ir::Op::Binary`/`Op::Arith`/`Op::Prefix` enter as well.**
    /// A check placed in `eval_node` instead would leave the compiled arm
    /// answering, and only running both arms can tell.
    fn both_engines(source: &[u8]) -> (i32, String, String) {
        let mut answer = None;
        for engine in [Engine::TreeWalker, Engine::Ir] {
            let outcome = run_program(
                "/t.rex",
                source.to_vec(),
                Invocation::none().with_engine(engine),
            );
            let seen = (
                outcome.exit_code,
                String::from_utf8_lossy(&outcome.stdout).into_owned(),
                String::from_utf8_lossy(&outcome.stderr).into_owned(),
            );
            match &answer {
                None => answer = Some(seen),
                Some(first) => assert_eq!(
                    first,
                    &seen,
                    "the two engines disagree on {:?}",
                    String::from_utf8_lossy(source)
                ),
            }
        }
        answer.expect("at least one engine ran")
    }

    /// Every operator the oracle sends to its left operand as a message
    /// refuses loudly, naming the operator and the operand's shape.
    ///
    /// **Each of these answered at rc 0 or raised the wrong condition before
    /// this test existed**, and the pre-Phase-5 build refused the whole
    /// program at rc 120 because `.array` did not resolve at all -- so
    /// resolving the name without this check turned a loud gap into a wrong
    /// answer. The oracle's own answer is in each row's comment.
    #[test]
    fn an_operator_sent_to_an_object_is_loud() {
        // (source, the operator the message must name, the shape it must name)
        let cases: &[(&[u8], &str, &str)] = &[
            // oracle 0 -- identity, not a comparison of renderings
            (
                b"say (.array == 'The Array class')\n",
                "==",
                "a class object",
            ),
            (b"say (.array = 'The Array class')\n", "=", "a class object"),
            (
                b"say (.array \\== 'The Array class')\n",
                "\\==",
                "a class object",
            ),
            // oracle 97.1 at rc 159 -- the operator is a message the class
            // does not answer
            (b"say (.array > .array)\n", ">", "a class object"),
            (b"say (.array + 1)\n", "+", "a class object"),
            (b"say (.array ** 1)\n", "**", "a class object"),
            (b"say (.array & 1)\n", "&", "a class object"),
            (b"say -.array\n", "-", "a class object"),
            (b"say \\.array\n", "\\", "a class object"),
            // oracle 0, "The NIL object" -- Directory answers `+` through its
            // own UNKNOWN, which this crate models nothing of
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
        ];
        for (source, op, kind) in cases {
            let (code, stdout, stderr) = both_engines(source);
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

    /// **R12 at the one numeric surface that is not an operator**: a
    /// controlled `DO` header's `initial`, `TO` and `BY` values.
    ///
    /// `Interp::header_number` rounds each through what is a real unary `+` on
    /// the oracle, so all three answer 97.1 -- and `do i = 1 to .array` is why
    /// "the operand on the right always agrees" is a fact about the binary
    /// operators and not a rule: here every position converts through an
    /// operator of its own.
    ///
    /// Before this, each answered 41.1 at rc 215 quoting the object's
    /// rendering, where the pre-Phase-5 build refused the whole program.
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
        ];
        for (source, role, kind) in cases {
            let (code, stdout, stderr) = both_engines(source);
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
    ///
    /// The control for the test above, in the shape the concatenation family
    /// is the control for the operator one. `FOR` and a bare `DO`'s repeat
    /// count go through `whole_nonneg`, which never asks for a number, and
    /// `NUMERIC DIGITS` renders the value with `to_text` and parses the bytes
    /// in `set_digits_str` -- so all three answer 26.3, 26.2 and 26.5 from the
    /// object's rendering on both sides.
    /// `corpus/lang/environment_object_in_a_loop_header.rex` is the same
    /// property against the live oracle.
    #[test]
    fn a_header_position_that_reads_text_keeps_the_oracles_own_diagnostic() {
        for (source, major) in [
            (&b"do i = 1 to 5 for .array\nend\n"[..], "26.3"),
            (b"do .array\nend\n", "26.2"),
            (b"numeric digits .array\n", "26.5"),
        ] {
            let (code, _stdout, stderr) = both_engines(source);
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
    ///
    /// Measured: `do e over .array` is 98.913 at rc 158, and a directory or a
    /// string table iterates its own entries -- `do e over .environment`
    /// prints `INPUTOUTPUTSTREAM` first. `LoopState::OverOnce` bound the
    /// target once and yielded the object's rendering, so each of these was a
    /// single wrong line at rc 0 where the pre-Phase-5 build refused the
    /// program.
    #[test]
    fn an_object_as_a_do_over_target_is_loud() {
        let cases: &[(&[u8], &str)] = &[
            (b"do e over .array\nsay e\nend\n", "a class object"),
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
            let (code, stdout, stderr) = both_engines(source);
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
    }

    /// **A stem redirects to its default, and every check has to follow.**
    ///
    /// `to_text` and `to_number` chase a `Body::Stem`'s default, so a check
    /// that stopped at the stem handle let the review's original Critical back
    /// in through one assignment: measured, `a. = .array; say (a. == 'The
    /// Array class')` answered `1` where the oracle answers `0`.
    ///
    /// The last row is the control that says the hole was the indirection: a
    /// compound read yields the class handle itself and was loud already.
    #[test]
    fn an_object_reached_through_a_stem_default_is_loud() {
        for source in [
            &b"a. = .array\nsay (a. == 'The Array class')\n"[..],
            b"a. = .array\nsay a. + 1\n",
            b"a. = .array\ndo i = 1 to a.\nend\n",
            b"a. = .array\nsay a.zz + 1\n",
        ] {
            let (code, stdout, stderr) = both_engines(source);
            assert_eq!(
                (code, stdout.as_str()),
                (120, ""),
                "{:?} reported {stderr:?}",
                String::from_utf8_lossy(source)
            );
            assert!(
                stderr.contains("a class object"),
                "{:?} must name the shape it refused, got {stderr:?}",
                String::from_utf8_lossy(source)
            );
        }
    }

    /// A controlled loop's own increment adds to the control variable, and
    /// that variable is the oracle's **left** operand of the implicit `+`.
    ///
    /// Measured, `do i = 1 to 3; say 'iter' i; i = .array; end` prints one
    /// iteration and then raises 97.1; this crate printed the same iteration
    /// and then raised 41.1. The stdout assertion is what pins that the
    /// refusal happens at the increment and not before the body runs.
    #[test]
    fn an_object_assigned_to_a_control_variable_is_loud_at_the_increment() {
        let (code, stdout, stderr) =
            both_engines(b"do i = 1 to 3\nsay 'iter' i\ni = .array\nend\n");
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
    ///
    /// `RaiseInstruction::execute` (`instructions/RaiseInstruction.cpp:270`
    /// -`290`) makes that call once, inside
    /// `if (errorCode->strCompare(SYNTAX))`. Measured: `additional (.array)`
    /// is a 98 execution error at rc 158, and `additional (.environment)`
    /// substitutes `INPUTOUTPUTSTREAM` -- the first entry of the array the
    /// directory converts to -- where rendering the object substituted its own
    /// name into an otherwise correct 40.1.
    ///
    /// **The `ARRAY (...)` form is not this and is deliberately absent.** The
    /// oracle builds a real `ArrayClass` from those elements first
    /// (`:217`-`:239`), so the `requestArray` below gets an array and returns
    /// it unchanged; the elements are rendered, never converted. A check on
    /// that arm refused three programs this crate already matched, which is
    /// what [`a_raise_the_oracle_does_not_array_convert_still_answers`] now
    /// holds it to.
    #[test]
    fn an_object_as_a_raise_syntax_substitution_is_loud() {
        for source in [
            &b"raise syntax 40.1 additional (.array)\n"[..],
            b"raise syntax 40.1 additional (.environment)\n",
        ] {
            let (code, stdout, stderr) = both_engines(source);
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
    ///
    /// Every expected string below is the oracle's own output for that
    /// program, and the `ARRAY` and `USER` ones were **rc 120 for a while**:
    /// the first version of the `RAISE` fix checked the `ARRAY` elements and
    /// checked `ADDITIONAL` under every condition, which refused four
    /// programs this crate had been matching byte for byte. `DESCRIPTION` was
    /// never refused and is here because the arm sits beside the two that
    /// were. It all shipped with the whole suite green, because the control
    /// standing in for it was a counted loop with no `RAISE` in it.
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
            let (code, _stdout, stderr) = both_engines(source);
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
    ///
    /// The boundary is the oracle's own and is measured, not reasoned: the
    /// operator is sent to the **left** operand, so an object on the right
    /// is converted through `stringValue()` exactly as this crate converts
    /// it; and the concatenation family calls `stringValue()` on both sides
    /// whichever operand is an object. Every expected string below is the
    /// oracle's own stdout for that program.
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
            // which is `LoopState::OverOnce`'s own rule and stays true.
            (b"do e over 'abc'\nsay e\nend\n", "abc\n"),
            // A stem whose default is an ordinary value is untouched by the
            // redirect the check now follows.
            (b"a. = .array\nsay a.\n", "The Array class\n"),
            (b"a. = .array\nsay a. || 'x'\n", "The Array classx\n"),
            (b"z. = 5\ndo i = 1 to z.\nsay i\nend\n", "1\n2\n3\n4\n5\n"),
        ];
        for (source, expected) in cases {
            let (code, stdout, stderr) = both_engines(source);
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
    ///
    /// `IF`/`WHEN`/`WHILE` and the comma list check their value's *text* on
    /// both sides -- measured, `if .array then` is 34.1 and `if .array, 1
    /// then` is 34.6 on the oracle, both quoting the object's own rendering.
    /// Extending R12's refusal to them would be an over-refusal of a
    /// diagnostic the two implementations already agree on byte for byte.
    #[test]
    fn a_condition_on_an_object_keeps_the_oracles_own_diagnostic() {
        for (source, major) in [
            (&b"if .array then say 'y'\n"[..], "34.1"),
            (b"if .array, 1 then say 'y'\n", "34.6"),
            (b"do while .array\nend\n", "34.3"),
        ] {
            let (code, _stdout, stderr) = both_engines(source);
            assert_eq!(code, 222, "{:?}", String::from_utf8_lossy(source));
            assert!(
                stderr.contains(&format!("Error {major}:")),
                "{:?} reported {stderr:?}",
                String::from_utf8_lossy(source)
            );
        }
    }
}
