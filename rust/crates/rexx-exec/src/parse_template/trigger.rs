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
    /// Evaluates one trigger's operand, if it has one, and moves the cursor.
    pub(super) fn apply_trigger(
        &mut self,
        code: &Code<'_>,
        trigger: &ParseTrigger,
        source: &SourceText,
        cursor: &mut Cursor,
        indent: usize,
    ) -> Result<(), Failure> {
        // `End` is the only kind with no operand, so every other arm below
        // can ask for one. A missing operand on a kind that needs one is not
        // representable from a program that parsed -- `parse_template`
        // (`rexx-parse`) fills `value` on every trigger it builds except
        // `End`, and refuses a trigger with nothing after it (38.901) -- and
        // it is reported as the gap it would be rather than defaulted to a
        // position.
        let operand = match trigger.kind {
            TriggerKind::End => {
                cursor.move_to_end();
                return Ok(());
            }
            _ => match &trigger.value {
                Some(operand) => operand,
                None => return Err(Loud::parse_trigger_operand().into()),
            },
        };
        let value = self.eval_in_parse(code, operand)?;
        self.roots.activity_mut().push_temp(value);
        // `>>>` for the operand, after `eval`'s own `>L>`/`>V>` and before
        // the conversion that can fail: `integerTrigger`
        // (`ParseTrigger.cpp:143`-`153`) traces and only then converts, so
        // the operand's own value line is emitted even on the 26.4 path.
        // **Copied only where a copy is needed**, which is neither of the two
        // arms below. `trace_result` returns at once unless `results` is on,
        // and a search reads the bytes where they already are: `Cursor` is a
        // local of `exec_parse` rather than a field, so a shared borrow of
        // the value can be live while it is written to. The unconditional
        // `to_vec` here was 5.2% of every allocation `samples/rexxcps.rex`
        // made -- one per trigger, on a program whose templates are mostly
        // numeric triggers that never look at the bytes at all.
        if self.trace_mode().results {
            let rendered = self.traced_string_value_text(value)?;
            self.trace_result(indent, &rendered);
        }
        match trigger.kind {
            // Convert before borrowing the source because conversion can allocate.
            // Both source and needle bytes are borrowed only for the search.
            TriggerKind::String => {
                let needle = self.required_string_value(value)?;
                self.roots.activity_mut().push_temp(needle);
                let needle = self.render(needle);
                cursor.search(source.bytes(self), needle.text(self));
                Ok(())
            }
            TriggerKind::Mixed => {
                let needle = self.required_string_value(value)?;
                self.roots.activity_mut().push_temp(needle);
                let needle = self.render(needle);
                cursor.caseless_search(source.bytes(self), needle.text(self));
                Ok(())
            }
            _ => {
                // 26.4, and the conversion is bounded by the **active**
                // `NUMERIC DIGITS` rather than by a fixed width
                // (`integerTrigger`'s own `requestUnsignedNumber(result,
                // number_digits())`). Measured: `numeric digits 2; parse
                // value d with p +(100) q` is 26.4 on the oracle, as are a
                // fractional operand, a non-numeric one and a negative one.
                // The substitution is the operand's own rendering -- measured
                // `found "1E2"` for `+(1e2)`, which is D15's rule showing
                // through rather than a re-rendering here.
                let Some(offset) = self.whole_nonneg(value) else {
                    // The one place the operand's own rendering is still
                    // owned, and it is the raise's substitution.
                    return Err(Raised::syntax(26, 4, vec![self.to_text(value).to_vec()]).into());
                };
                // Clamped rather than checked: an offset past `usize` is
                // indistinguishable from one past the string's own end, and
                // every operation below treats those the same way.
                let offset = usize::try_from(offset).unwrap_or(usize::MAX);
                match trigger.kind {
                    TriggerKind::Plus => cursor.forward(offset),
                    TriggerKind::Minus => cursor.backward(offset),
                    TriggerKind::Absolute => cursor.absolute(offset),
                    TriggerKind::PlusLength => cursor.forward_length(offset),
                    TriggerKind::MinusLength => cursor.backward_length(offset),
                    // Handled above; the operand kinds and the movement
                    // kinds partition `TriggerKind`, and this arm is what
                    // keeps that a compile-time exhaustive match rather than
                    // an assumption.
                    TriggerKind::End | TriggerKind::String | TriggerKind::Mixed => {}
                }
                Ok(())
            }
        }
    }
}
