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
    /// `USE ARG`, `USE STRICT ARG` and `USE LOCAL`.
    /// ```text
    /// as the program's own first instruction    98.993, rc 158
    /// as a ::ROUTINE's own first instruction    98.993, rc 158
    /// anywhere else                             99.910, rc 157
    /// ```
    pub(super) fn exec_use(
        &mut self,
        code: &Code<'_>,
        use_: &Use,
        first_instruction: bool,
    ) -> Result<(), Failure> {
        match use_ {
            Use::Local { variables } => {
                // 98.993 is "this is not a method invocation" and 99.910 is
                // "it is one, but this is not its first instruction", so
                // what decides between them is the entry kind rather than
                // whether there was a call. Measured on the oracle: `use
                // local` first in a `::ROUTINE` is 98.993 at rc 158, the
                // same answer the top-level shape gets.
                let method_invocation = match self.activation().entry {
                    Entry::TopLevel | Entry::InternalCall | Entry::Routine => false,
                    Entry::Method => true,
                };
                if first_instruction && method_invocation {
                    self.use_local(code, variables)
                } else if first_instruction {
                    Err(Raised::use_local_outside_method().into())
                } else {
                    Err(Raised::use_local_not_first().into())
                }
            }
            Use::Arg {
                strict,
                allow_optionals,
                targets,
            } => self.exec_use_arg(code, *strict, *allow_optionals, targets),
        }
    }

    /// `USE LOCAL` as a method's first instruction
    /// (`RexxActivation::autoExpose`): the listed names and `SELF`, `SUPER`,
    /// `RC`, `SIGL` and `RESULT` stay local, and every other name binds to the
    /// receiver's pool for the method's scope, a name first met later
    /// included ([`Interp::slot_of`]).
    fn use_local(&mut self, code: &Code<'_>, variables: &[VariableRef]) -> Result<(), Failure> {
        let Some(identity) = self.activation().method_identity.as_ref() else {
            return Err(Raised::use_local_outside_method().into());
        };
        let (scope, receiver) = (identity.scope, identity.receiver);
        let owner = self.pool_owner(receiver)?;
        let mut local: Vec<Box<[u8]>> = [b"SELF".as_slice(), b"SUPER", b"RC", b"SIGL", b"RESULT"]
            .into_iter()
            .map(Into::into)
            .collect();
        for variable in variables {
            let (VariableRef::Direct(id) | VariableRef::Indirect(id)) = variable;
            local.push(code.symbols.name(*id).as_bytes().into());
        }
        let auto = AutoExpose {
            owner,
            scope,
            local: local.into(),
        };
        let activation = self.activation();
        let mut names: Vec<(usize, Box<[u8]>)> = activation
            .plan
            .names
            .iter()
            .chain(activation.extra.iter())
            .map(|(name, slot)| (*slot, name.clone()))
            .collect();
        names.sort_unstable_by_key(|(slot, _)| *slot);
        for (slot, name) in names {
            self.auto_expose_slot(&auto, slot, name);
        }
        self.activation_mut().cold_mut().auto_expose = Some(auto);
        Ok(())
    }

    /// Binds `slot`, named `name`, to `auto`'s pool unless the name is one
    /// `USE LOCAL` keeps local or a compound, which reaches the pool through
    /// its stem.
    pub(crate) fn auto_expose_slot(&mut self, auto: &AutoExpose, slot: usize, name: Box<[u8]>) {
        if shape_of(&name) == NameShape::Compound || auto.local.contains(&name) {
            return;
        }
        self.expose_slot(
            slot,
            InstanceVar {
                owner: auto.owner,
                scope: auto.scope,
                name,
            },
        );
    }

    /// `USE ARG`/`USE STRICT ARG`: bind the call's arguments to this
    /// instruction's targets, positionally.
    fn exec_use_arg(
        &mut self,
        code: &Code<'_>,
        strict: bool,
        allow_optionals: bool,
        targets: &[Option<UseTarget>],
    ) -> Result<(), Failure> {
        let in_method = self.activation().entry == Entry::Method;
        if strict {
            let supplied = self.activity.call_context.arguments.len();
            // The minimum is the position of the last target that must be
            // supplied -- one with no default of its own. A later target
            // carrying a default does not raise it, which is what makes `use
            // strict arg p, q = 'dflt'` legal with one argument.
            let minimum = targets
                .iter()
                .rposition(|target| {
                    target
                        .as_ref()
                        .is_none_or(|target| target.default.is_none())
                })
                .map_or(0, |index| index + 1);
            if supplied < minimum {
                return Err(if in_method {
                    Raised::not_enough_method_arguments(minimum).into()
                } else {
                    let name = Rc::clone(&self.activity.call_context.name);
                    Raised::not_enough_arguments(&name, minimum).into()
                });
            }
            if !allow_optionals && supplied > targets.len() {
                return Err(if in_method {
                    Raised::too_many_method_arguments(targets.len()).into()
                } else {
                    let name = Rc::clone(&self.activity.call_context.name);
                    Raised::too_many_arguments(&name, targets.len()).into()
                });
            }
        }

        for (index, target) in targets.iter().enumerate() {
            let Some(target) = target else { continue };
            // `get` past the end and a `None` inside the list are the same
            // thing to a target: nothing was supplied for this position.
            let argument = self
                .activity
                .call_context
                .arguments
                .get(index)
                .cloned()
                .flatten();
            self.bind_use_target(code, index, target, argument, strict, in_method)?;
        }
        Ok(())
    }

    /// Binds one `USE ARG` target to one argument, or to its default, or to
    /// nothing.
    fn bind_use_target(
        &mut self,
        code: &Code<'_>,
        index: usize,
        target: &UseTarget,
        argument: Option<ObjRef>,
        strict: bool,
        in_method: bool,
    ) -> Result<(), Failure> {
        let position = index + 1;
        if target.alias {
            let Some(argument) = argument else {
                return Err(Raised::variable_reference_omitted(position).into());
            };
            let Some(bound) = self.as_variable_reference(argument) else {
                let found = self.to_text(argument).to_vec();
                return Err(Raised::not_a_variable_reference(position, &found).into());
            };
            let reference = bound.name.clone();
            let slot = bound.home.clone();
            let name = self.use_target_name(code, target)?;
            // **The kinds must match, and the check is before the
            // uninitialised one.** Measured: a target that is both
            // kind-mismatched and already assigned reports the kind error,
            // not 98.995. Compound is not a third kind to handle -- `>p.1`
            // and `>q.1` are both rejected by `rexx-parse` (20.930/20.931),
            // so each side is a simple variable or a stem and nothing else.
            let target_is_stem = shape_of(&name) == NameShape::Stem;
            let reference_is_stem = shape_of(&reference) == NameShape::Stem;
            if target_is_stem != reference_is_stem {
                // Both substitute the *caller's* name, unlike 98.995 just
                // below, which names the target. Measured with a variable
                // whose value differs from its name, so the two cannot be
                // confused: `p = 'value-not-name'` passed as `>p` reports
                // `found "P"`.
                return Err(if target_is_stem {
                    Raised::not_a_stem_variable_reference(position, &reference).into()
                } else {
                    Raised::not_a_simple_variable_reference(position, &reference).into()
                });
            }
            let index = self.slot_of(&name);
            let frame = self.activation().frame;
            // The target must be **currently unset**. `RootSet::slot`
            // resolves through any alias already in force, which is what the
            // repeat case needs: after one `use arg >q`, `Q` reads the
            // caller's variable, so it "has a value" and the second attempt
            // is refused.
            if !self.target_is_uninitialised(&name, frame, index) {
                return Err(Raised::variable_reference_not_uninitialised(&name).into());
            }
            match slot {
                VarRefHome::Cell(cell) => self.roots.activity_mut().alias_slot(frame, index, cell),
                // The same binding `EXPOSE` makes, on this activation's own
                // slot: the target names the caller's object variable rather
                // than any frame storage, so there is nothing to alias to.
                VarRefHome::Instance { owner, scope } => {
                    let var = InstanceVar {
                        owner,
                        scope,
                        name: reference.clone(),
                    };
                    let activation = self.activation_mut();
                    match activation.exposed.iter_mut().find(|(at, _)| *at == index) {
                        Some(bound) => bound.1 = var,
                        None => activation.exposed.push((index, var)),
                    }
                }
            }
            // `>R>`, the alias's own line and the **only** trace line this
            // branch emits: no `>>>` and no `>=>`, because nothing was
            // evaluated and nothing was assigned (`UseInstruction.cpp:164`-
            // `167`, `aliasVariable` then `traceVariableAlias`, and
            // `handleArgument` `return`s before its own `traceResult` for
            // this case). Caller's name first, target's second -- see
            // `trace_alias`.
            self.trace_alias(
                self.activity.clause_state.current_value_indent,
                &reference,
                &name,
            );
            return Ok(());
        }

        // Present: bind the value. Absent: the default if there is one, and
        // otherwise drop the target -- measured, an absent target does not
        // keep whatever it held before.
        let value = match argument {
            Some(argument) => Some(argument),
            None => match &target.default {
                Some(default) => {
                    let value = self.eval(code, default)?;
                    self.roots.activity_mut().push_temp(value);
                    Some(value)
                }
                None => None,
            },
        };
        // A message or bracket term assigns through PARSE's arm; its `drop` is
        // a no-op (`ExpressionBaseVariable.hpp:65`), so an omitted argument
        // without a default leaves it alone.
        if matches!(target.target.kind, ExprKind::Message { .. }) {
            return match value {
                Some(value) => {
                    let indent = self.activity.clause_state.current_value_indent;
                    let rendered = self.result_text(value)?;
                    if let Some(rendered) = &rendered {
                        self.trace_result(indent, rendered);
                    }
                    self.assign_expr_target(code, &target.target, value, indent, None)
                }
                None if strict => Err(if in_method {
                    Raised::missing_method_argument(position).into()
                } else {
                    let call = Rc::clone(&self.activity.call_context.name);
                    Raised::missing_argument(&call, position).into()
                }),
                None => Ok(()),
            };
        }
        let name = self.use_target_name(code, target)?;
        match value {
            Some(value) => {
                // `>>>` then `>=>`, in that order and both at this `USE`
                // clause's own indent -- `handleArgument`'s own
                // `traceResult(argument)` immediately before
                // `retriever->assign(context, argument)`, whose own
                // `traceAssignment` is the second line
                // (`UseInstruction.cpp:74`-`77`, and the default-value arm
                // ten lines below it does the identical pair). Measured
                // under `trace r`: `use arg a, b` on a two-argument call
                // traces `>>>     "1"` and `>>>     "2"` and no `>=>`, which
                // is the gating -- `>>>` is `results`, `>=>` is
                // `intermediates`, so the pair is not one line's worth of
                // conditional.
                let indent = self.activity.clause_state.current_value_indent;
                // Render each result and assignment line at its emission point.
                let rendered = self.result_text(value)?;
                if let Some(rendered) = &rendered {
                    self.trace_result(indent, rendered);
                }
                self.roots.activity_mut().push_temp(value);
                self.assign_by_name(&name, value);
                if let Some(rendered) = self.intermediate_text(value)? {
                    self.trace_assignment(indent, &name, &rendered, value);
                }
                Ok(())
            }
            // `USE STRICT ARG` refuses an omitted position that has no
            // default of its own, where `USE ARG` drops the target
            // (`UseInstruction.cpp:97`-`:111`). The position is the target's,
            // not the last one supplied -- measured, oracle rc 216, `call r
            // , 2` into `use strict arg a, b` reports `argument 1`.
            None if strict => Err(if in_method {
                Raised::missing_method_argument(position).into()
            } else {
                let call = Rc::clone(&self.activity.call_context.name);
                Raised::missing_argument(&call, position).into()
            }),
            None => {
                self.drop_by_name(&name);
                Ok(())
            }
        }
    }

    /// Whether a `USE ARG >name` target is in the uninitialised state the
    /// oracle requires of it: a local variable (`RexxVariable::isAliasable`),
    /// so never one an `EXPOSE` or `USE LOCAL` bound to the object's pool, and
    /// for a stem one named as the variable that owns the slot's storage.
    fn target_is_uninitialised(&self, name: &[u8], frame: SlotFrame, index: usize) -> bool {
        if self.exposure(frame, index).is_some() {
            return false;
        }
        match self.variable(frame, index) {
            None => true,
            Some(value) => {
                let own = self.cell_owner_name(frame, index).unwrap_or(name);
                shape_of(name) == NameShape::Stem && self.is_uninitialised_stem(value, own)
            }
        }
    }

    /// One `USE ARG` target's variable name.
    fn use_target_name(&self, code: &Code<'_>, target: &UseTarget) -> Result<Vec<u8>, Failure> {
        match &target.target.kind {
            ExprKind::Variable(id) | ExprKind::Stem(id) | ExprKind::Compound(id) => {
                Ok(code.symbols.name(*id).as_bytes().to_vec())
            }
            other => Err(Loud::expression(other).into()),
        }
    }
}
