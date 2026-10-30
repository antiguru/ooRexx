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
    /// Pushes one `::METHOD` body's activation and the [`CallTail`] its
    /// sender's level state goes into, up to the point the body would run.
    ///
    /// [`CallTail`]: crate::run::CallTail
    #[expect(
        clippy::too_many_arguments,
        reason = "the send's own parts, and the receiver's behaviour where the send has it"
    )]
    pub(super) fn begin_method(
        &mut self,
        _cleared: Cleared,
        installed: crate::InstalledMethodBody,
        resolution: Resolution,
        behaviour: Option<Behaviour>,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<(), Failure> {
        let Some(program) = self.programs.get(installed.program).map(Rc::clone) else {
            return Err(Loud::missing_body().into());
        };
        // Every read is `get`, not an index: an `InstalledMethodBody`
        // always names a real directive with a body, but this crate's rule for an internal
        // inconsistency is a loud refusal rather than a panic, and
        // `body_of`'s own `?` already follows it.
        let Some(directive) = program.directives.get(installed.directive) else {
            return Err(Loud::missing_body().into());
        };
        let Some(body) = body_of(&program, Some(installed.directive)) else {
            return Err(Loud::missing_body().into());
        };
        let guarded = self.sends_guarded(
            resolution.method,
            crate::guards::directive_guarded(&directive.kind),
        );
        // D19/I6, the same guard `Interp::invoke_call` takes and for the same
        // reason: a method that sends itself a message is an unbounded
        // recursion, and it must become a reportable condition rather than a
        // native abort.
        let probe = 0u8;
        if self.activation_depth() >= MAX_ACTIVATION_DEPTH || self.stack_exhausted(&raw const probe)
        {
            return Err(Raised::insufficient_stack().into());
        }
        let plan = self.plan_for(
            BodyKey {
                program: installed.program,
                directive: Some(installed.directive),
            },
            body,
            &program.symbols,
            &program.source,
        );
        let frame = self.roots.push_slots(plan.len());
        let (self_slot, super_slot) = (plan.self_slot, plan.super_slot);
        let reserve_at_once =
            guarded && !plan.traces_entry_early && !self.activities.guards.is_live();
        let callee_id = self.next_activation_id();
        let super_scope = match behaviour {
            Some(behaviour) => self.super_scope_in(behaviour, resolution.scope),
            None => self.super_scope_for(receiver, resolution),
        };
        // **The calling convention, entered before anything reads it.** The
        // receiver goes in here and is read back out below, so the identity
        // the activation carries, the `SELF` the body reads and the caller a
        // send inside the body resolves as all name one field rather than
        // each taking its own copy of this function's argument.
        let arguments = self.shared_arguments(args);
        let name = self.activity.invocation_name(name);
        let saved_context = std::mem::replace(
            &mut self.activity.call_context,
            crate::CallContext {
                name: Rc::clone(&name),
                arguments,
                receiver: Some(receiver),
            },
        );
        // **Shadows the parameter**, which is what makes the routing a
        // property of scope rather than of a comment: past this line the name
        // `receiver` is the convention's own field, so the bindings below
        // cannot read the argument even by accident.
        let receiver = self
            .activity
            .call_context
            .receiver
            .expect("the calling convention replaced directly above carries the receiver");
        // The package the `::METHOD` was declared in, exactly as a
        // `::ROUTINE` starts from its own. `running_activation` rather than
        // `activation` because a send made outside any activation has no
        // caller for `::OPTIONS NUMERIC INHERIT` to read.
        let start = self.package_start(
            installed.program,
            self.running_activation().map(|caller| &caller.settings),
        );
        let mut callee = Activation::method(
            callee_id,
            program,
            installed.program,
            installed.directive,
            plan,
            frame,
            MethodIdentity {
                name,
                scope: resolution.scope,
                receiver,
            },
        );
        if let Some(start) = start {
            start.apply(&mut callee);
        }
        // Dictionary mutation must not replace the Method this invocation
        // entered. Existing registered objects use the activation's GC root.
        if let Some(executable) = self.executable_objects.get(resolution.method).copied() {
            callee.cold_mut().executable = Some(executable);
        }
        // With one activity the lock is the activation's flag alone, taken
        // here unless a package-traced `>I>` has to come first.
        let reserve_at_once = reserve_at_once && !callee.trace_mode.labels;
        callee.flags = crate::activation::ActivationFlags::method(guarded, reserve_at_once);
        self.push_activation(callee);
        self.trace_package_invocation_entry();
        if guarded && !reserve_at_once {
            self.reserve_for_method(crate::guards::GuardKey {
                object: receiver,
                scope: resolution.scope,
            });
        }

        // `SELF` and `SUPER`, the two locals `RexxActivation::run` sets on a
        // method activation before its first instruction
        // (`RexxActivation.cpp:535`-`536`). Measured in a class method of
        // `::class K`: `say self` is `The K class` and `say super` is
        // `The Class class`.
        let self_slot = match self_slot {
            Some(slot) => slot,
            None => self.slot_of(b"SELF"),
        };
        self.set_variable(frame, self_slot, receiver);
        let super_slot = match super_slot {
            Some(slot) => slot,
            None => self.slot_of(b"SUPER"),
        };
        // `.nil` for the topmost scope, which is what `superScope` answers
        // there.
        self.set_variable(frame, super_slot, super_scope.unwrap_or(ObjRef::NIL));

        // The same level state `Interp::invoke_call` saves and restores, set
        // to the same values its `::ROUTINE` arm uses -- that function's own
        // comment enumerates the pieces and carries the measurement for
        // each. A method's clause echoes are at indent 0 whatever the sending
        // clause's own indent was: measured, a send from inside two nested
        // `DO` blocks echoes the method's clauses at 0.
        let saved_clause_state = self.save_clause_state();
        let saved_base = std::mem::replace(&mut self.activity.activation_indent, 0);
        let saved_offset = std::mem::take(&mut self.activity.indent_offset);
        let saved_line = std::mem::take(&mut self.activity.clause_line_override);
        self.activity.call_tails.push(crate::run::CallTail::method(
            saved_context,
            saved_clause_state,
            saved_base,
            saved_offset,
            saved_line,
        ));
        Ok(())
    }
}

#[cfg(test)]
mod tests;
