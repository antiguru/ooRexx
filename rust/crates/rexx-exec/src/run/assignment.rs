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

use super::*;

impl Interp {
    /// Everything one assignment does once its value has been evaluated:
    /// `>>>`, then the write and the lines the write itself produces.
    #[inline]
    pub(crate) fn assign_evaluated(
        &mut self,
        code: &Code<'_>,
        target: &Expr,
        value: ObjRef,
        at: Option<usize>,
    ) -> Result<(), Failure> {
        self.roots.activity_mut().push_temp(value);
        // `>>>` fires before the assignment itself
        // (`RexxInstructionAssignment::execute`: evaluate, trace, *then*
        // assign), which matters only in that the traced value can never be
        // affected by the write it precedes.
        // Reads `current_value_indent` rather than recomputing
        // `static_indent(index)` independently -- the clause unit already
        // computed exactly this value (`indent_offset` included, F-EX1's own
        // correction to F3) for this same instruction right before the clause
        // ran, and a second computation of the identical quantity is how the
        // two drift, which is exactly what happened here before this fix: this
        // site's own copy never learned about the offset when the field was
        // added.
        let indent = self.activity.clause_state.current_value_indent;
        // The entry gate excludes a result line when evaluation enabled tracing.
        if self.activity.clause_state.instructions_traced_at_entry
            && let Some(rendered) = self.result_text(value)?
        {
            self.trace_result(indent, &rendered);
        }
        self.assign_expr_target(code, target, value, indent, at)
    }

    /// Writes `value` into the slot a plain-variable target is already bound
    /// to, and traces the write. `rendered` is the assigned value's own text
    /// where a `>=>` line will print it, and `None` where none will.
    ///
    /// Panics in a debug build when `id` does not name a simple variable: the
    /// slot written here is the whole name's, which a stem or a compound tail
    /// does not have.
    #[inline]
    pub(crate) fn assign_bound_variable(
        &mut self,
        code: &Code<'_>,
        id: SymbolId,
        slot: usize,
        value: ObjRef,
        rendered: Option<&[u8]>,
        indent: usize,
    ) {
        debug_assert!(
            matches!(
                shape_of(code.symbols.name(id).as_bytes()),
                NameShape::Simple
            ),
            "a bound slot was written for {}, which is not a simple variable",
            code.symbols.name(id),
        );
        let frame = self.activation().frame;
        self.set_variable(frame, slot, value);
        // The name is reached only where a line will print it, which is what
        // the general path below pays on every call.
        if let Some(rendered) = rendered {
            self.trace_assignment(indent, code.symbols.name(id).as_bytes(), rendered, value);
        }
    }

    /// Writes `value` through one assignment *target expression*, and traces
    /// the write.
    pub(crate) fn assign_expr_target(
        &mut self,
        code: &Code<'_>,
        target: &Expr,
        value: ObjRef,
        indent: usize,
        at: Option<usize>,
    ) -> Result<(), Failure> {
        self.roots.activity_mut().push_temp(value);
        match &target.kind {
            ExprKind::Variable(id) => {
                let name = code.symbols.name(*id).as_bytes();
                let slot = match at {
                    Some(slot) => slot,
                    None => self.slot_of(name),
                };
                let frame = self.activation().frame;
                self.set_variable(frame, slot, value);
                if let Some(rendered) = self.intermediate_text(value)? {
                    self.trace_assignment(indent, name, &rendered, value);
                }
            }
            // `stem. = expr`: replace-and-rebind (D15a), through the
            // library `stem_assign` already builds -- this arm is the
            // dispatch, not new stem logic.
            ExprKind::Stem(id) => {
                // The tripwire `crate::ir::drive`'s `Op::Store` arm carries
                // for the compiled side, here where **both** engines pass:
                // a slot handed to this arm was resolved against the symbol's
                // own id and is silently shadowed below, so a caller that
                // started supplying one would write through the entry's slot
                // and never learn that its own was ignored.
                debug_assert!(
                    at.is_none(),
                    "a stem write was handed a slot, and the slot it writes comes from the entry"
                );
                let name = code.symbols.name(*id).as_bytes();
                let at = code.compound(*id).and_then(|entry| entry.stem_at);
                self.stem_assign_at(name, at, value);
                if let Some(rendered) = self.intermediate_text(value)? {
                    self.trace_assignment(indent, name, &rendered, value);
                }
            }
            // `a.b = expr`: resolve the tail key the same way reading
            // `a.b` would (`eval_node`'s own `Compound` arm), then
            // mutate that one tail in place through `stem_set`.
            ExprKind::Compound(id) => {
                // `read_symbol`'s own compound tripwire, on the writing
                // side. A compound-shaped name **can** reach
                // `Plan::by_symbol`: `Plan::bind` puts every name it binds
                // whole there, and `note_loop` and `note_parse` both call it
                // with spellings that may be compound-shaped. So a caller
                // reaching for a slot by this symbol's id can find one, and it
                // is not the slot this arm writes.
                debug_assert!(
                    at.is_none(),
                    "a compound write was handed a slot, and the slot it writes is the stem's"
                );
                // **Borrowed, not copied.** `Code::stem` and `SymbolTable::
                // name` both answer with the lifetime of the `Code`, which is
                // the program rather than this `Interp`, so neither needs an
                // owned copy to survive the `&mut self` calls below. The
                // `Variable` arm above has always passed its name borrowed;
                // this arm copied both, and measured with `heaptrack` on
                // `samples/rexxcps.rex` that cost two allocations per compound
                // write.
                let tag = code.symbols.name(*id).as_bytes();
                let (stem_name, stem_at) = code.stem(*id);
                let mut key = self.take_key_buffer();
                if let Err(failure) = self.tail_key_into(code, *id, &mut key) {
                    self.give_key_buffer(key);
                    return Err(failure);
                }
                self.stem_set_at(stem_name, stem_at, &key, value);
                // **The resolved name is built only when a line will print
                // it.** `trace_compound_name` returns at once unless
                // intermediates are on, so joining the stem to the tail key
                // ahead of that check allocated a name to discard. This is the
                // rule the value rendering below follows -- `trace.rs`'s own
                // doc comment gives the reasoning for the value half, and the
                // name half is the same argument.
                if self.tracing_intermediates() {
                    let mut resolved = stem_name.to_vec();
                    resolved.extend_from_slice(&key);
                    self.trace_compound_name(indent, tag, &resolved);
                }
                if let Some(rendered) = self.intermediate_text(value)? {
                    self.trace_assignment(indent, tag, &rendered, value);
                }
                self.give_key_buffer(key);
            }
            // `RexxExpressionMessage::assign` (`expression/ExpressionMessage.cpp:232`):
            // the term's `NAME=` sent with the value ahead of its own
            // arguments, each argument traced as a result.
            ExprKind::Message {
                target: receiver,
                name,
                super_class,
                args,
                ..
            } => {
                let receiver = self.eval(code, receiver)?;
                self.roots.activity_mut().push_temp(receiver);
                let scope = match super_class {
                    Some(super_class) => {
                        let scope = self.eval(code, super_class)?;
                        self.roots.activity_mut().push_temp(scope);
                        Some(scope)
                    }
                    None => None,
                };
                let mut values = vec![Some(value)];
                for arg in args {
                    let evaluated = match arg {
                        Some(expr) => {
                            let evaluated = self.eval(code, expr)?;
                            self.roots.activity_mut().push_temp(evaluated);
                            Some(evaluated)
                        }
                        None => None,
                    };
                    if let Some(shown) = evaluated
                        .map(|value| self.result_text(value))
                        .transpose()?
                        .flatten()
                    {
                        self.trace_result(indent, &shown);
                    } else if evaluated.is_none() {
                        self.trace_result(indent, b"");
                    }
                    values.push(evaluated);
                }
                let mut setter = name.to_vec();
                setter.push(b'=');
                self.validate_scope_override(receiver, scope)?;
                let caller = self.caller();
                let answer = pinned!(
                    self,
                    crate::pinning::PinKind::TreeSend,
                    self.send_message(receiver, &setter, scope, &values, caller)
                )?;
                if let Some(answer) = answer
                    && let Some(shown) = self.intermediate_text(answer)?
                {
                    self.trace_assignment(indent, &setter, &shown, answer);
                }
            }
            other => return Err(Loud::expression(other).into()),
        }
        Ok(())
    }

}
