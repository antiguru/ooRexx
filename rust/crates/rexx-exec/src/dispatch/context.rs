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

//! `RexxContext`'s and `StackFrame`'s readers -- `classes/ContextClass.cpp`
//! and `classes/StackFrameClass.cpp`, bound by `memory/Setup.cpp:1211`-`:1238`
//! and `:1700`-`:1723`.

use super::{Arity, Cleared, Failure, Interp, Loud, NativeMethod, ObjRef, Raised, hash};
use crate::activation::{Activation, Entry};
use crate::plan::ProgramId;
use rexx_core::{BehaviourId, Body};

/// `RexxContext`'s and `StackFrame`'s instance methods. Chained into
/// `ObjectModel::build` beside [`super::NATIVE_METHODS`].
pub(super) static NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    ("RexxContext", "ARGS", Arity::Fixed(0), context_args),
    ("RexxContext", "COPY", Arity::Fixed(0), context_copy),
    (
        "RexxContext",
        "CONDITION",
        Arity::Fixed(0),
        context_condition,
    ),
    ("RexxContext", "DIGITS", Arity::Fixed(0), context_digits),
    (
        "RexxContext",
        "EXECUTABLE",
        Arity::Fixed(0),
        context_executable,
    ),
    ("RexxContext", "FORM", Arity::Fixed(0), context_form),
    ("RexxContext", "FUZZ", Arity::Fixed(0), context_fuzz),
    (
        "RexxContext",
        "INTERPRETER",
        Arity::Fixed(0),
        context_interpreter,
    ),
    (
        "RexxContext",
        "INVOCATION",
        Arity::Fixed(0),
        context_invocation,
    ),
    ("RexxContext", "LINE", Arity::Fixed(0), context_line),
    ("RexxContext", "NAME", Arity::Fixed(0), context_name),
    ("RexxContext", "PACKAGE", Arity::Fixed(0), context_package),
    ("RexxContext", "RS", Arity::Fixed(0), context_rs),
    (
        "RexxContext",
        "STACKFRAMES",
        Arity::Fixed(0),
        context_stack_frames,
    ),
    ("RexxContext", "THREAD", Arity::Fixed(0), context_thread),
    (
        "RexxContext",
        "VARIABLES",
        Arity::Fixed(0),
        context_variables,
    ),
    ("StackFrame", "ARGUMENTS", Arity::Fixed(0), frame_arguments),
    ("StackFrame", "CONTEXT", Arity::Fixed(0), frame_context),
    (
        "StackFrame",
        "EXECUTABLE",
        Arity::Fixed(0),
        frame_executable,
    ),
    (
        "StackFrame",
        "INVOCATION",
        Arity::Fixed(0),
        frame_invocation,
    ),
    ("StackFrame", "LINE", Arity::Fixed(0), frame_line),
    (
        "StackFrame",
        "MAKESTRING",
        Arity::Fixed(0),
        frame_trace_line,
    ),
    ("StackFrame", "NAME", Arity::Fixed(0), frame_name),
    ("StackFrame", "STRING", Arity::Fixed(0), frame_trace_line),
    ("StackFrame", "TARGET", Arity::Fixed(0), frame_target),
    ("StackFrame", "TRACELINE", Arity::Fixed(0), frame_trace_line),
    ("StackFrame", "TYPE", Arity::Fixed(0), frame_type),
];

/// `RexxContext`'s and `StackFrame`'s **class** methods, chained into
/// `ObjectModel::build` beside [`super::NATIVE_CLASS_METHODS`].
pub(super) static NATIVE_CLASS_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    (
        "RexxContext",
        "NEW",
        Arity::Counted,
        super::native_unsupported_new,
    ),
    (
        "StackFrame",
        "NEW",
        Arity::Counted,
        super::native_unsupported_new,
    ),
];

/// The entry keys one `StackFrame`'s snapshot is stored under, on the
/// `NativeObject` the frame is.
mod key {
    pub(super) const TYPE: &[u8] = b"TYPE";
    pub(super) const NAME: &[u8] = b"NAME";
    pub(super) const LINE: &[u8] = b"LINE";
    pub(super) const INVOCATION: &[u8] = b"INVOCATION";
    pub(super) const TARGET: &[u8] = b"TARGET";
    pub(super) const ARGUMENTS: &[u8] = b"ARGUMENTS";
    pub(super) const CONTEXT: &[u8] = b"CONTEXT";
    pub(super) const EXECUTABLE: &[u8] = b"EXECUTABLE";
    pub(super) const TRACE_LINE: &[u8] = b"TRACELINE";
}

/// One activation's fields as `StackFrameClass`'s constructor copies them,
/// gathered before anything is allocated.
struct Snapshot {
    kind: &'static [u8],
    name: Vec<u8>,
    clause: crate::activation::ClauseSnapshot,
    body: Option<usize>,
    program: ProgramId,
    target: Option<ObjRef>,
    arguments: Vec<Option<ObjRef>>,
}

impl Interp {
    /// Where in the running activity's stack the `RexxContext` `receiver`
    /// belongs, counted from the innermost -- `0` is the running activation.
    fn context_depth(&self, receiver: ObjRef) -> Option<usize> {
        self.frames()
            .position(|activation| activation.context_object == Some(receiver))
    }

    /// `RexxActivation::getIdntfr` (`execution/RexxActivation.cpp:94`): this
    /// activation's `~invocation`, minted on the first ask.
    pub(crate) fn invocation_of(&mut self, depth: usize) -> Option<u32> {
        if let Some(found) = self.frame_at(depth)?.invocation {
            return Some(found);
        }
        self.next_invocation += 1;
        let minted = self.next_invocation;
        self.frame_at_mut(depth)?.invocation = Some(minted);
        Some(minted)
    }
}

/// `read`'s answer for the activation the `RexxContext` `receiver` names, at
/// its depth in its own activity, which is swapped in where it is idle.
/// `RexxContext::checkValid` (`classes/ContextClass.cpp:145`) tests only that
/// the activation has not ended, so `98.981` is the answer where no activity
/// holds it.
fn at_context(
    interp: &mut Interp,
    receiver: ObjRef,
    read: impl FnOnce(&mut Interp, usize) -> Result<Option<ObjRef>, Failure>,
) -> Result<Option<ObjRef>, Failure> {
    if let Some(depth) = interp.context_depth(receiver) {
        return read(interp, depth);
    }
    let owner = interp
        .idle_context_owner(receiver)
        .ok_or_else(|| Failure::from(Raised::context_not_active()))?;
    let answer = interp.with_idle_activity(owner, |interp| {
        let depth = interp
            .context_depth(receiver)
            .expect("the owner holds the context's activation");
        read(interp, depth)
    })?;
    if let Some(object) = answer {
        interp.roots.activity_mut().push_temp(object);
    }
    Ok(answer)
}

/// The activation at `depth`, or `98.981`.
fn frame(interp: &Interp, depth: usize) -> Result<&Activation, Failure> {
    interp
        .frame_at(depth)
        .ok_or_else(|| Failure::from(Raised::context_not_active()))
}

/// `RexxContext::getDigits`: `activation->digits()`, the setting **in force**
/// at that context.
fn context_digits(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    at_context(interp, receiver, |interp, depth| {
        let digits = frame(interp, depth)?.settings.digits();
        Ok(Some(interp.counted(digits as usize)))
    })
}

/// `RexxContext::getFuzz`: `activation->fuzz()`, in force -- see
/// [`context_digits`].
fn context_fuzz(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    at_context(interp, receiver, |interp, depth| {
        let fuzz = frame(interp, depth)?.settings.fuzz();
        Ok(Some(interp.counted(fuzz as usize)))
    })
}

/// `RexxContext::getForm`: `activation->form()` as one of the two global
/// names, in force -- see [`context_digits`].
fn context_form(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    at_context(interp, receiver, |interp, depth| {
        let text: &[u8] = match frame(interp, depth)?.settings.form() {
            rexx_num::Form::Scientific => b"SCIENTIFIC",
            rexx_num::Form::Engineering => b"ENGINEERING",
        };
        Ok(Some(interp.text(text)))
    })
}

/// `RexxContext::getLine`: `activation->getContextLine()`, the line of the
/// clause that context is **currently executing**.
fn context_line(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    at_context(interp, receiver, |interp, depth| {
        let line = interp.clause_of(depth).line;
        Ok(Some(interp.counted(line)))
    })
}

/// `RexxContext::getName`: `activation->getCallname()` -- the name this
/// context was invoked under, and the program's own path at the top level.
fn context_name(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    at_context(interp, receiver, |interp, depth| {
        let name = frame(interp, depth)?.invoked_as().to_vec();
        Ok(Some(interp.text_built(name)))
    })
}

/// `RexxContext::getInvocation`: `activation->getIdntfr()`.
fn context_invocation(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    at_context(interp, receiver, |interp, depth| {
        let id = interp
            .invocation_of(depth)
            .ok_or_else(|| Failure::from(Raised::context_not_active()))?;
        Ok(Some(interp.counted(id as usize)))
    })
}

/// `RexxContext::getThread`: `activation->getActivity()->getIdntfr()`.
fn context_thread(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    at_context(interp, receiver, |interp, _| {
        let number = interp.activity_number();
        Ok(Some(interp.counted(number as usize)))
    })
}

/// `RexxContext::getInterpreter`:
/// `activation->getActivity()->getInstance()->getIdntfr()`.
fn context_interpreter(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    at_context(interp, receiver, |interp, _| Ok(Some(interp.counted(1))))
}

/// `RexxContext::getRS`: `activation->getContextReturnStatus()`, which is
/// `.nil` unless `settings.isReturnStatusSet()`.
fn context_rs(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    at_context(interp, receiver, |interp, depth| {
        let rs = frame(interp, depth)?.rs;
        Ok(Some(match rs {
            Some(rs) => interp.text(rs.to_string().as_bytes()),
            None => ObjRef::NIL,
        }))
    })
}

/// `RexxContext::getCondition`: a **copy** of the condition object the
/// activation is handling, and `.nil` when it is handling none.
fn context_condition(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    at_context(interp, receiver, |interp, depth| {
        match frame(interp, depth)?
            .condition()
            .and_then(|condition| condition.object)
        {
            Some(object) => interp.condition_copy(object).map(Some),
            None => Ok(Some(ObjRef::NIL)),
        }
    })
}

/// `RexxContext::getPackage`: the package of the program **the receiver's own
/// activation** belongs to (`ContextClass.cpp:160`).
fn context_package(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    at_context(interp, receiver, |interp, depth| {
        let program = frame(interp, depth)?.program_id;
        Ok(Some(
            interp.package_object(crate::plan::Package::Program(program))?,
        ))
    })
}

/// `RexxContext::getArgs`: the arguments the context's activation was entered
/// with, as a fresh `Array`.
fn context_args(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    at_context(interp, receiver, |interp, depth| {
        let arguments = frame(interp, depth)?.invoked_with().to_vec();
        Ok(Some(array_of_slots(interp, arguments)))
    })
}

/// `RexxContext::getVariables`: `activation->getAllLocalVariables()`, a fresh
/// `Directory` of the context's own variable pool.
fn context_variables(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    at_context(interp, receiver, |interp, depth| {
        local_variables(interp, depth).map(Some)
    })
}

/// `RexxActivation::getAllLocalVariables`: a `Directory` of every variable
/// the activation at `depth` has set, a stem under its name with the period.
pub(crate) fn local_variables(interp: &mut Interp, depth: usize) -> Result<ObjRef, Failure> {
    let entries = local_variable_entries(interp, depth)?;
    hash::directory_of(interp, entries)
}

/// [`local_variables`]'s pairs, name and value, with no `Directory` built.
pub(crate) fn local_variable_entries(
    interp: &Interp,
    depth: usize,
) -> Result<Vec<(Vec<u8>, ObjRef)>, Failure> {
    let activation = interp
        .frame_at(depth)
        .ok_or_else(|| Failure::from(Raised::context_not_active()))?;
    let named = activation
        .plan
        .names
        .iter()
        .map(|(name, slot)| (&**name, *slot))
        .chain(activation.extra.iter().map(|(name, slot)| (&**name, *slot)));
    let mut entries = Vec::new();
    for (name, slot) in named {
        if let Some(value) = interp.variable_in(activation, slot) {
            entries.push((name.to_vec(), value));
        }
    }
    Ok(entries)
}

/// `RexxContext::copyRexx`, whose whole body is the 93.970 raise.
fn context_copy(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let target = interp.string_value_text(receiver);
    Err(Raised::copy_not_supported(&target).into())
}

/// `RexxContext::getExecutable`: `activation->getExecutable()`, the `Method`
/// or `Routine` object this context is running.
fn context_executable(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    at_context(interp, receiver, |interp, depth| {
        executable_at(interp, depth).map(Some)
    })
}

/// [`context_executable`]'s answer for the activation at `depth`.
pub(crate) fn executable_at(interp: &mut Interp, depth: usize) -> Result<ObjRef, Failure> {
    match interp.frame_at(depth) {
        Some(activation) if activation.entry == Entry::InternalCall => {
            internal_executable_at(interp, depth)
        }
        _ => caller_executable_at(interp, depth),
    }
}

/// [`executable_at`] for an internal call at `depth`.
#[cold]
#[inline(never)]
fn internal_executable_at(interp: &mut Interp, depth: usize) -> Result<ObjRef, Failure> {
    // An internal call runs its caller's code, and answers its caller's
    // executable: measured, oracle rc 0, an internal routine called from a
    // `::METHOD` answers that `Method`. Each internal call keeps the answer,
    // so a stack's frames, built level by level, walk a chain of internal
    // calls once.
    let mut found = depth;
    while let Some(activation) = interp.frame_at(found)
        && activation.entry == Entry::InternalCall
    {
        if let Some(kept) = activation.cold.as_deref().and_then(|cold| cold.executable) {
            return Ok(kept);
        }
        found += 1;
    }
    let executable = caller_executable_at(interp, found)?;
    let activity = &mut interp.activity;
    for activation in activity
        .running
        .iter_mut()
        .chain(activity.suspended.iter_mut().rev())
        .skip(depth)
        .take(found - depth)
    {
        activation.cold_mut().executable = Some(executable);
    }
    Ok(executable)
}

/// [`executable_at`] for the activation at `depth`, which is not an
/// internal call where there is one.
fn caller_executable_at(interp: &mut Interp, depth: usize) -> Result<ObjRef, Failure> {
    let activation = interp
        .frame_at(depth)
        .ok_or_else(|| Failure::from(Raised::context_not_active()))?;
    let program = activation.program_id;
    match (
        activation.entry,
        activation.body,
        &activation.method_identity,
    ) {
        (Entry::Method, Some(directive), Some(identity)) => {
            if let Some(executable) = activation.cold.as_deref().and_then(|cold| cold.executable) {
                return Ok(executable);
            }
            let (scope, name, receiver) =
                (identity.scope, identity.name.to_vec(), identity.receiver);
            let body = crate::InstalledMethodBody { program, directive };
            let executable = interp.running_method_executable(scope, &name, receiver, body)?;
            // Kept on the activation, which roots it: measured, oracle rc 0,
            // a method that unsets its own entry answers the same object
            // before and after a forced collection.
            if let Some(activation) = interp.frame_at_mut(depth) {
                activation.cold_mut().executable = Some(executable);
            }
            Ok(executable)
        }
        // A `::CONSTANT` expression's level has no body and carries its own.
        (Entry::Method, None, _) => activation
            .cold
            .as_deref()
            .and_then(|cold| cold.executable)
            .ok_or_else(|| Loud::unknown_receiver("a method level with no body").into()),
        (Entry::Routine, Some(directive), _) => {
            if let Some(executable) = activation.cold.as_deref().and_then(|cold| cold.executable) {
                return Ok(executable);
            }
            // A routine built from source text is a synthetic directive in no
            // package table, and is its own executable: measured, oracle rc
            // 0, `.routine~new('r', 'return .context~executable~class~id')~call`
            // answers `Routine`.
            let synthetic = interp
                .programs
                .get(program)
                .and_then(|source| source.directives.get(directive))
                .is_some_and(|declared| declared.clause_span.is_empty());
            if synthetic && let Some(object) = routine_object_running(interp, program, directive) {
                return Ok(object);
            }
            let name = routine_entry_name(interp, program, directive)?;
            let table = interp
                .program_string_table(program, crate::environment::PackageTable::Routines)?
                .ok_or_else(|| Loud::unknown_receiver("a routine whose package has no table"))?;
            interp
                .native_entry(table, &name)
                .ok_or_else(|| Loud::unknown_receiver("a routine with no package entry").into())
        }
        _ => interp.program_routine_object(program),
    }
}

/// A live `Routine` object whose `~call` enters `directive` of `program`.
fn routine_object_running(interp: &Interp, program: ProgramId, directive: usize) -> Option<ObjRef> {
    interp
        .executable_sources
        .iter()
        .find(|(object, record)| {
            record.routine == Some((program, directive)) && interp.heap.get(**object).is_some()
        })
        .map(|(object, _)| *object)
}

/// The `.ROUTINES` key one `::ROUTINE` directive is filed under: its declared
/// name, upcased, exactly as `package_table_entries` files it.
fn routine_entry_name(
    interp: &Interp,
    program: ProgramId,
    directive: usize,
) -> Result<Vec<u8>, Failure> {
    let source = interp
        .programs
        .get(program)
        .ok_or_else(|| Failure::from(Loud::missing_body()))?;
    match source.directives.get(directive).map(|found| &found.kind) {
        Some(rexx_parse::DirectiveKind::Routine(routine)) => Ok(routine.name.to_ascii_uppercase()),
        _ => Err(Loud::unknown_receiver("a routine activation with no directive").into()),
    }
}

/// `RexxContext::getStackFrames`: every frame of the stack of the context's
/// activity, innermost first.
fn context_stack_frames(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    at_context(interp, receiver, |interp, _| {
        let levels = live_levels(interp);
        let frame = interp.roots.activity_mut().push_frame();
        let mut slots = Vec::with_capacity(levels.len());
        for level in levels {
            let object = build_live_frame(interp, level)?;
            interp.roots.activity_mut().push_temp(object);
            slots.push(Some(object));
        }
        let array = interp.alloc_charged(
            BehaviourId::ARRAY,
            Body::Array {
                dimensions: None,
                slots: slots.into(),
            },
        );
        interp.roots.activity_mut().pop_frame(frame);
        interp.roots.activity_mut().push_temp(array);
        Ok(Some(array))
    })
}

/// One level of the running stack as `Activity::generateStackFrames` walks
/// it: an activation, a native call, or an `INTERPRET` fragment.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum LiveLevel {
    /// The activation at this depth.
    Activation(usize),
    /// This row of [`Activity::native_handles`].
    Native(usize),
    /// This row of [`Activity::method_levels`].
    Method(usize),
    /// This row of [`Activity::internal_levels`].
    Internal(usize),
    /// This row of [`Activity::fragments`].
    Fragment(usize),
    /// This row of [`Activity::native_tails`]: a sent primitive method that
    /// entered the activation listed before it.
    Entering(usize),
}

/// Every level of the running stack, innermost first: before each
/// activation, the native calls it made and then the fragments running in
/// it, each innermost first, and after it the sent primitive methods that
/// entered it. A primitive's level is listed only where its method is found
/// ([`Interp::running_primitive`]).
pub(crate) fn live_levels(interp: &mut Interp) -> Vec<LiveLevel> {
    let mut levels = listed_levels(interp);
    levels.retain(|level| match *level {
        LiveLevel::Method(row) => {
            let level = &interp.activity.method_levels[row];
            level.resolution.is_some() || {
                let (receiver, name) = (level.receiver, level.name.clone());
                interp.running_primitive(receiver, &name).is_some()
            }
        }
        _ => true,
    });
    levels
}

/// [`live_levels`] before primitive levels are checked.
fn listed_levels(interp: &Interp) -> Vec<LiveLevel> {
    let mut levels = Vec::new();
    for (depth, activation) in interp.frames().enumerate() {
        let id = activation.id;
        let running = &interp.activity.internal_levels[..interp.activity.internal_depth];
        for (row, level) in running.iter().enumerate().rev() {
            if level.caller == Some(id) {
                levels.push(LiveLevel::Internal(row));
            }
        }
        for (row, native) in interp.activity.native_handles.iter().enumerate().rev() {
            if native.caller == Some(id) {
                levels.push(LiveLevel::Native(row));
            }
        }
        for (row, level) in interp.activity.method_levels.iter().enumerate().rev() {
            if level.caller == Some(id) {
                levels.push(LiveLevel::Method(row));
            }
        }
        for (row, level) in interp.activity.fragments.iter().enumerate().rev() {
            if level.owner == id {
                levels.push(LiveLevel::Fragment(row));
            }
        }
        levels.push(LiveLevel::Activation(depth));
        levels.extend(interp.entering_tails(id).map(LiveLevel::Entering));
    }
    levels
}

/// The `StackFrame` of one live level.
pub(crate) fn build_live_frame(interp: &mut Interp, level: LiveLevel) -> Result<ObjRef, Failure> {
    match level {
        LiveLevel::Activation(depth) => build_frame(interp, depth),
        LiveLevel::Native(row) => build_native_frame_at(interp, row),
        LiveLevel::Method(row) => build_method_level_frame(interp, row),
        LiveLevel::Internal(row) => build_internal_frame_at(interp, row),
        LiveLevel::Fragment(row) => build_fragment_frame(interp, row),
        LiveLevel::Entering(row) => interp
            .entering_frame_at(row)
            .map(|(frame, _)| frame)
            .ok_or_else(|| Loud::unknown_receiver("a native level no send made").into()),
    }
}

/// The `StackFrame` of the primitive method in row `row` of
/// [`Activity::method_levels`]: no line, no invocation and no context, and
/// its `Compiled method` line as its traceback.
fn build_method_level_frame(interp: &mut Interp, row: usize) -> Result<ObjRef, Failure> {
    let level = &interp.activity.method_levels[row];
    let (resolution, receiver) = (level.resolution, level.receiver);
    let (name, arguments) = (level.name.clone(), level.arguments.clone());
    let resolution = resolution.or_else(|| interp.running_primitive(receiver, &name));
    let (scope_name, executable) = match resolution {
        Some(found) => (
            interp.scope_id(found.scope),
            interp.running_native_executable(found.scope, found.method, &name, receiver),
        ),
        None => (interp.scope_id(ObjRef::NIL), None),
    };
    Ok(build_native_level_frame(
        interp,
        None,
        &name,
        receiver,
        &arguments,
        &Raised::compiled_method_line(&name, &scope_name),
        executable,
    ))
}

/// The `StackFrame` of the running fragment in row `row`, stopped on its
/// clause being stepped, or the one that entered the fragment above it.
fn build_fragment_frame(interp: &mut Interp, row: usize) -> Result<ObjRef, Failure> {
    let level = &interp.activity.fragments[row];
    let (line, owner) = (level.line, level.owner);
    let inner = interp.activity.fragments.get(row + 1);
    let clause = inner.map_or(interp.activity.fragment_clause, |inner| inner.outer_clause);
    // The fragment's own clause's indent: the `INTERPRET` clause of the
    // owner's next fragment, else the owner's clause, live or as it was left
    // for a call.
    let indent = match inner {
        Some(inner) if inner.owner == owner => inner.indent,
        _ => match interp.running_activation() {
            Some(running) if running.id == owner => {
                interp.activity.clause_state.current_value_indent
            }
            _ => interp
                .frames()
                .find(|activation| activation.id == owner)
                .map_or(0, |activation| activation.clause.indent),
        },
    };
    let level = &interp.activity.fragments[row];
    let text = level
        .fragment
        .as_ref()
        .and_then(|fragment| {
            let span = fragment.body.instructions.get(clause)?.clause_span.clone();
            fragment.source.join_span(span)
        })
        .map(std::borrow::Cow::into_owned)
        .unwrap_or_default();
    let owner = level.owner;
    let mut trace_text = Vec::new();
    crate::trace::push_clause(&mut trace_text, line, indent, &text);
    while trace_text.last() == Some(&b'\n') {
        trace_text.pop();
    }
    let depth = interp
        .frames()
        .position(|activation| activation.id == owner);
    let executable = depth.and_then(|depth| executable_at(interp, depth).ok());
    let invocation = fragment_invocation(interp, row);
    build_ended_frame(
        interp,
        b"INTERPRET",
        b"",
        line,
        &[],
        &trace_text,
        executable,
        Some(invocation),
    )
}

/// The `~invocation` of the fragment in row `row` of
/// [`Activity::fragments`], minted on the first ask and kept for the
/// fragment's life, as an interpret activation keeps its own.
fn fragment_invocation(interp: &mut Interp, row: usize) -> u32 {
    if let Some(minted) = interp.activity.fragments[row].invocation {
        return minted;
    }
    interp.next_invocation += 1;
    let minted = interp.next_invocation;
    interp.activity.fragments[row].invocation = Some(minted);
    minted
}

/// One `StackFrame` for the activation at `depth` --
/// `RexxActivation::createStackFrame` (`RexxActivation.cpp:5006`).
pub(crate) fn build_frame(interp: &mut Interp, depth: usize) -> Result<ObjRef, Failure> {
    build_frame_with(interp, depth, true)
}

/// [`build_frame`], with the arguments left out where `arguments` is false:
/// an internal call a tail-less `RAISE` left has had its argument list
/// cleared by the time the oracle's frame is taken (`RexxActivation::
/// raiseExit`). Measured, oracle rc 0: a call with two arguments reports
/// none in the condition object.
pub(crate) fn build_frame_with(
    interp: &mut Interp,
    depth: usize,
    arguments: bool,
) -> Result<ObjRef, Failure> {
    let invocation = interp.invocation_of(depth);
    // Created here when the activation never asked for one, which is what
    // `createStackFrame`'s own `getContextObject()` does -- see
    // `Interp::context_object_at`.
    let context = interp.context_object_at(depth);
    let executable = executable_at(interp, depth).ok();
    let mut snapshot = read_snapshot(interp, depth)?;
    if !arguments {
        snapshot.arguments.clear();
    }
    let frame = interp.roots.activity_mut().push_frame();
    if let Some(executable) = executable {
        interp.roots.activity_mut().push_temp(executable);
    }
    let kind = interp.text(snapshot.kind);
    interp.roots.activity_mut().push_temp(kind);
    let name = interp.text_built(std::mem::take(&mut snapshot.name));
    interp.roots.activity_mut().push_temp(name);
    let line = interp.counted(snapshot.clause.line);
    let invocation = match invocation {
        // `StackFrameClass::getInvocation` answers `.nil` for a zero id,
        // which is the frame kinds a native activation builds and which this
        // crate has none of; the id is minted here rather than left at zero.
        None => ObjRef::NIL,
        Some(id) => interp.counted(id as usize),
    };
    interp.roots.activity_mut().push_temp(invocation);
    let trace_text = trace_line_text(interp, depth, &snapshot);
    let trace_line = interp.text(&trace_text);
    interp.roots.activity_mut().push_temp(trace_line);
    let arguments = array_of_slots(interp, std::mem::take(&mut snapshot.arguments));
    interp.roots.activity_mut().push_temp(arguments);
    let entries = [
        (key::TYPE, kind),
        (key::NAME, name),
        (key::LINE, line),
        (key::INVOCATION, invocation),
        (key::TARGET, snapshot.target.unwrap_or(ObjRef::NIL)),
        (key::ARGUMENTS, arguments),
        (key::CONTEXT, context.unwrap_or(ObjRef::NIL)),
        (key::TRACE_LINE, trace_line),
        (key::EXECUTABLE, executable.unwrap_or(ObjRef::NIL)),
    ];
    let set = if executable.is_some() {
        &entries[..]
    } else {
        &entries[..entries.len() - 1]
    };
    Ok(frame_object(interp, frame, set, &trace_text))
}

/// The `StackFrame` holding `entries`, whose values the caller rooted inside
/// `frame`, which this closes.
fn frame_object(
    interp: &mut Interp,
    frame: rexx_core::FrameId,
    entries: &[(&'static [u8], ObjRef)],
    trace_text: &[u8],
) -> ObjRef {
    let class = interp.object_model().stack_frame;
    let object = interp.native_instance(class);
    let held = interp
        .heap
        .get_mut(object)
        .expect("just allocated and rooted");
    let Body::Native(native) = &mut held.body else {
        unreachable!("allocated as Body::Native by native_instance")
    };
    native.reserve_entries(entries.len());
    for &(name, value) in entries {
        native.set_entry(name, value);
    }
    // `StackFrameClass::stringValue`/`makeString`, which answer the traceback
    // line where `defaultName` answers `a StackFrame` -- the one native class
    // in this crate that separates the two. Without it an `Array` holding a
    // frame joins on the default name: measured, oracle rc 0,
    // `say .array~of(f)` is the traceback line where `f~objectName` is
    // `a StackFrame`.
    native.set_string_value(trace_text);
    interp.roots.activity_mut().pop_frame(frame);
    // Re-pushed above the frame that has just closed, which is the shape
    // `Interp::line_array` and `array_of_texts` already have: the temp
    // `native_instance` took for this object was inside that frame, so
    // without this the frame comes back to its caller rooted by nothing.
    interp.roots.activity_mut().push_temp(object);
    object
}

/// The `StackFrame` of an `INTERPRET` fragment of the running activation,
/// executing `line`, whose traceback line is `trace_text`: `FRAME_INTERPRET`
/// with no name, target or arguments, and an invocation and a context of its
/// own (`RexxActivation::createStackFrame`, `RexxActivation.cpp:5006`). The
/// context is one whose activation has ended, as the fragment's has by the
/// time anything reads it; the executable is the running activation's.
pub(crate) fn build_interpret_frame(
    interp: &mut Interp,
    line: usize,
    trace_text: &[u8],
) -> Result<ObjRef, Failure> {
    let executable = executable_at(interp, 0).ok();
    let running = interp.running_activation().map(|activation| activation.id);
    let invocation = interp
        .activity
        .fragments
        .len()
        .checked_sub(1)
        .filter(|&row| Some(interp.activity.fragments[row].owner) == running)
        .map(|row| fragment_invocation(interp, row));
    build_ended_frame(
        interp,
        b"INTERPRET",
        b"",
        line,
        &[],
        trace_text,
        executable,
        invocation,
    )
}

/// A `StackFrame` of type `kind` for a level that has ended, built from its
/// parts: `invocation`, or one of its own where that is `None`, no target,
/// and a context whose activation has ended. `executable` is what
/// `~executable` answers, `None` where this crate does not know it.
#[allow(
    clippy::too_many_arguments,
    reason = "each is one entry of the frame it builds"
)]
pub(crate) fn build_ended_frame(
    interp: &mut Interp,
    kind: &[u8],
    name: &[u8],
    line: usize,
    arguments: &[Option<ObjRef>],
    trace_text: &[u8],
    executable: Option<ObjRef>,
    invocation: Option<u32>,
) -> Result<ObjRef, Failure> {
    let frame = interp.roots.activity_mut().push_frame();
    if let Some(executable) = executable {
        interp.roots.activity_mut().push_temp(executable);
    }
    let kind = interp.text(kind);
    interp.roots.activity_mut().push_temp(kind);
    let name = interp.text(name);
    interp.roots.activity_mut().push_temp(name);
    let line = interp.counted(line);
    let invocation = invocation.unwrap_or_else(|| {
        interp.next_invocation += 1;
        interp.next_invocation
    });
    let invocation = interp.counted(invocation as usize);
    interp.roots.activity_mut().push_temp(invocation);
    let context = interp.ended_context_object();
    let trace_line = interp.text(trace_text);
    interp.roots.activity_mut().push_temp(trace_line);
    let arguments = array_of_slots(interp, arguments.to_vec());
    interp.roots.activity_mut().push_temp(arguments);
    let mut entries = vec![
        (key::TYPE, kind),
        (key::NAME, name),
        (key::LINE, line),
        (key::INVOCATION, invocation),
        (key::TARGET, ObjRef::NIL),
        (key::ARGUMENTS, arguments),
        (key::CONTEXT, context),
        (key::TRACE_LINE, trace_line),
    ];
    if let Some(executable) = executable {
        entries.push((key::EXECUTABLE, executable));
    }
    Ok(frame_object(interp, frame, &entries, trace_text))
}

/// The `StackFrame` of a native method or routine that `target` (`.nil` for a
/// routine) was sent with `arguments`, with no line or invocation and its
/// `Compiled method` or `Compiled routine` line as its traceback
/// (`InternalActivationFrame::createStackFrame`,
/// `concurrency/ActivationFrame.cpp:100`, and `NativeActivation::
/// createStackFrame`, `execution/NativeActivation.cpp:3620`). A routine's
/// context is the calling activation's and a method's is `.nil`: measured,
/// oracle rc 0, `SysSleep`, `FILESPEC` and an `EXTERNAL` routine answer one,
/// `SUBSTR` and an `EXTERNAL` method none. `caller` is `None` for a method,
/// and for a routine the depth of the activation that called it.
pub(crate) fn build_native_level_frame(
    interp: &mut Interp,
    caller: Option<usize>,
    name: &[u8],
    target: ObjRef,
    arguments: &[Option<ObjRef>],
    trace_text: &[u8],
    executable: Option<ObjRef>,
) -> ObjRef {
    let context = match caller {
        Some(depth) => interp.context_object_at(depth).unwrap_or(ObjRef::NIL),
        None => ObjRef::NIL,
    };
    let frame = interp.roots.activity_mut().push_frame();
    let kind = interp.text(if caller.is_none() {
        b"METHOD"
    } else {
        b"ROUTINE"
    });
    interp.roots.activity_mut().push_temp(kind);
    let name = interp.text(name);
    interp.roots.activity_mut().push_temp(name);
    let trace_line = interp.text(trace_text);
    interp.roots.activity_mut().push_temp(trace_line);
    let arguments = array_of_slots(interp, arguments.to_vec());
    let mut entries = vec![
        (key::TYPE, kind),
        (key::NAME, name),
        (key::LINE, ObjRef::NIL),
        (key::INVOCATION, ObjRef::NIL),
        (key::TARGET, target),
        (key::ARGUMENTS, arguments),
        (key::CONTEXT, context),
        (key::TRACE_LINE, trace_line),
    ];
    entries.extend(executable.map(|executable| (key::EXECUTABLE, executable)));
    frame_object(interp, frame, &entries, trace_text)
}

/// The `Method` object a native method level runs: the dictionary entry
/// `name` resolves to for `target`, which is what `Class~method` answers.
/// Measured, oracle rc 0: a `CHARIN` frame's executable is
/// `.Stream~method('CHARIN')` and a `SORTWITH` frame's
/// `.Array~method('SORTWITH')`.
pub(crate) fn native_method_executable(
    interp: &mut Interp,
    target: ObjRef,
    name: &[u8],
) -> Option<ObjRef> {
    let resolution = interp.lookup(target, name, None)?;
    interp.method_executable(resolution.scope, name).ok()
}

/// The `StackFrame` of the native call in row `row` of
/// [`Activity::native_handles`] -- `NativeActivation::createStackFrame`
/// (`execution/NativeActivation.cpp:3620`): no line and no invocation, the
/// calling activation's context, and the `Compiled routine` or `Compiled
/// method` line as its traceback.
///
/// # Panics
/// If no native call is in that row.
pub(crate) fn build_native_frame_at(interp: &mut Interp, row: usize) -> Result<ObjRef, Failure> {
    let native = &interp.activity.native_handles[row];
    let caller = native.caller;
    let (method, name, receiver, scope, arguments, code, ran) = (
        native.method,
        native.name.clone(),
        native.receiver,
        native.scope,
        native.arguments.clone(),
        native.code,
        native.executable,
    );
    let (kind, target, trace_text) = if method {
        let scope = interp.scope_id(scope);
        (
            &b"METHOD"[..],
            receiver,
            Raised::compiled_method_line(&name, &scope),
        )
    } else {
        (
            &b"ROUTINE"[..],
            ObjRef::NIL,
            Raised::compiled_routine_line(&name),
        )
    };
    let depth = caller_depth(interp, caller)?;
    let context = interp.context_object_at(depth);
    let frame = interp.roots.activity_mut().push_frame();
    let kind = interp.text(kind);
    interp.roots.activity_mut().push_temp(kind);
    let name = interp.text_built(name);
    interp.roots.activity_mut().push_temp(name);
    let trace_line = interp.text(&trace_text);
    interp.roots.activity_mut().push_temp(trace_line);
    let arguments = array_of_slots(interp, arguments);
    interp.roots.activity_mut().push_temp(arguments);
    let executable = if method {
        let name = interp.to_text(name).to_vec();
        interp.method_executable(scope, &name).ok()
    } else {
        interp.routine_executable(ran, code)
    };
    let mut entries = vec![
        (key::TYPE, kind),
        (key::NAME, name),
        (key::LINE, ObjRef::NIL),
        (key::INVOCATION, ObjRef::NIL),
        (key::TARGET, target),
        (key::ARGUMENTS, arguments),
        (key::CONTEXT, context.unwrap_or(ObjRef::NIL)),
        (key::TRACE_LINE, trace_line),
    ];
    entries.extend(executable.map(|executable| (key::EXECUTABLE, executable)));
    Ok(frame_object(interp, frame, &entries, &trace_text))
}

/// The `StackFrame` of the internal package routine in row `row` of
/// [`Activity::internal_levels`], built as a failing one's is.
///
/// # Panics
/// If no internal level is in that row.
fn build_internal_frame_at(interp: &mut Interp, row: usize) -> Result<ObjRef, Failure> {
    let level = &interp.activity.internal_levels[row];
    let name = match level.name {
        crate::activity::LevelName::Upper => level.row.name.to_ascii_uppercase().into_bytes(),
        crate::activity::LevelName::Row => level.row.name.as_bytes().to_vec(),
        crate::activity::LevelName::Called => level.called.clone(),
    };
    let arguments = level.arguments[level.from..].to_vec();
    let (caller, row, given) = (level.caller, level.row, level.executable);
    let depth = caller_depth(interp, caller)?;
    let executable = match given {
        ObjRef::NIL => interp.internal_routine_object(crate::internal_routines::index_of(row)),
        given => given,
    };
    Ok(build_native_level_frame(
        interp,
        Some(depth),
        &name,
        ObjRef::NIL,
        &arguments,
        &Raised::compiled_routine_line(&name),
        Some(executable),
    ))
}

/// The depth of the running activation `caller` names. Refuses one that is
/// not running: [`live_levels`] lists a native or internal level only beside
/// its caller.
fn caller_depth(
    interp: &Interp,
    caller: Option<crate::activation::ActivationId>,
) -> Result<usize, Failure> {
    let depth = interp
        .frames()
        .position(|activation| Some(activation.id) == caller);
    debug_assert!(
        depth.is_some(),
        "a level's caller {caller:?} is not running"
    );
    depth.ok_or_else(|| {
        Loud::unknown_receiver("a stack frame whose calling activation is no longer live").into()
    })
}

/// The activation at `depth` read out in one pass, before anything allocates.
fn read_snapshot(interp: &Interp, depth: usize) -> Result<Snapshot, Failure> {
    let activation = interp
        .frame_at(depth)
        .ok_or_else(|| Failure::from(Raised::context_not_active()))?;
    // `RexxActivation::createStackFrame`'s own cascade, whose whole value set
    // is `StackFrameClass.cpp`'s `FRAME_*` constants. `FRAME_INTERPRET` is a
    // fragment's, which `run_fragment` pushes no activation for and
    // `live_levels` lists beside them, and `FRAME_COMPILE` belongs to the parser
    // (`LanguageParser::createStackFrame`, `parser/LanguageParser.cpp:866`).
    // The phase's `found-not-fixed-register.md` is where which kinds this
    // crate reaches is recorded, because that is a boundary that moves.
    let (kind, target) = match activation.entry {
        Entry::TopLevel
            if matches!(
                activation.call_type,
                crate::activation::CallType::Subroutine
                    | crate::activation::CallType::Function
                    | crate::activation::CallType::Requires
            ) =>
        {
            (&b"ROUTINE"[..], None)
        }
        Entry::TopLevel => (&b"PROGRAM"[..], None),
        Entry::InternalCall => (&b"INTERNALCALL"[..], None),
        Entry::Routine => (&b"ROUTINE"[..], None),
        Entry::Method => (
            &b"METHOD"[..],
            activation
                .method_identity
                .as_ref()
                .map(|identity| identity.receiver),
        ),
    };
    let arguments = activation.invoked_with().to_vec();
    // A `::REQUIRES` prologue's frame is named for its package, measured.
    let name = match (activation.entry, activation.call_type) {
        (Entry::TopLevel, crate::activation::CallType::Requires) => {
            interp.program_display_name(activation.program_id).to_vec()
        }
        _ => activation.invoked_as().to_vec(),
    };
    Ok(Snapshot {
        kind,
        name,
        clause: interp.clause_of(depth),
        body: activation.body,
        program: activation.program_id,
        target,
        arguments,
    })
}

/// `RexxActivation::getTraceBack`: this frame's own clause as `TRACE` prints
/// it -- `PackageClass::traceBack` (`classes/PackageClass.cpp:561`) over the
/// instruction's source location and the trace indent in force.
fn trace_line_text(interp: &mut Interp, depth: usize, snapshot: &Snapshot) -> Vec<u8> {
    let traceable = interp
        .programs
        .get(snapshot.program)
        .is_none_or(|program| program.source.is_traceable());
    // The interpreter's own library has no source to show, as
    // `Interp::sourceless_site` reports it.
    let library = interp.library_programs.contains(&snapshot.program);
    let text = if library {
        interp.sourceless_line(depth, crate::LIBRARY_PACKAGE_NAME)
    } else if !traceable {
        let package = interp.program_display_name(snapshot.program).to_vec();
        interp.sourceless_line(depth, &package)
    } else {
        interp
            .programs
            .get(snapshot.program)
            .and_then(|program| {
                let span = crate::activation::body_of(program, snapshot.body)?
                    .instructions
                    .get(snapshot.clause.index)?
                    .clause_span
                    .clone();
                program.source.join_span(span)
            })
            .map(std::borrow::Cow::into_owned)
            .or_else(|| directive_clause_text(interp, depth, snapshot))
            .unwrap_or_default()
    };
    let mut out = Vec::new();
    crate::trace::push_clause(
        &mut out,
        snapshot.clause.line,
        snapshot.clause.indent,
        &text,
    );
    while out.last() == Some(&b'\n') {
        out.pop();
    }
    out
}

/// The clause of the directive a level that runs no body is stopped on
/// ([`crate::activation::ActivationCold::directive_clause`]), whose clause
/// index names no instruction.
#[cold]
#[inline(never)]
fn directive_clause_text(interp: &Interp, depth: usize, snapshot: &Snapshot) -> Option<Vec<u8>> {
    let directive = interp.frame_at(depth)?.cold.as_deref()?.directive_clause?;
    let program = interp.programs.get(snapshot.program)?;
    let span = program.directives.get(directive)?.clause_span.clone();
    program
        .source
        .join_span(span)
        .map(std::borrow::Cow::into_owned)
}

/// A fresh `Array` over `slots`, an omitted position left empty.
fn array_of_slots(interp: &mut Interp, slots: Vec<Option<ObjRef>>) -> ObjRef {
    let array = interp.alloc_charged(
        BehaviourId::ARRAY,
        Body::Array {
            dimensions: None,
            slots: slots.into(),
        },
    );
    interp.roots.activity_mut().push_temp(array);
    array
}

/// What a `StackFrame` method refuses a receiver of another class with
/// (Deviation 28).
const NOT_A_FRAME: &str = "a value that is not a stack frame";

/// The value one `StackFrame` snapshot entry holds.
fn frame_entry(interp: &Interp, receiver: ObjRef, name: &[u8]) -> Result<ObjRef, Failure> {
    match interp.heap.get(receiver).map(|held| &held.body) {
        Some(Body::Native(native)) => native
            .entry(name)
            .ok_or_else(|| Loud::receiver_class(NOT_A_FRAME).into()),
        _ => Err(Loud::receiver_class(NOT_A_FRAME).into()),
    }
}

/// `StackFrameClass::getType`: `PROGRAM`, `INTERNALCALL`, `ROUTINE` or
/// `METHOD` -- see [`read_snapshot`] for the two of the C++'s six this crate
/// cannot reach.
fn frame_type(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    frame_entry(interp, receiver, key::TYPE).map(Some)
}

/// `StackFrameClass::getName`: the routine or method name this frame was
/// invoked under, and the program's own path for a `PROGRAM` frame.
fn frame_name(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    frame_entry(interp, receiver, key::NAME).map(Some)
}

/// `StackFrameClass::getLine`: the line the frame was executing when the
/// snapshot was taken.
fn frame_line(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    frame_entry(interp, receiver, key::LINE).map(Some)
}

/// `StackFrameClass::getInvocation`.
fn frame_invocation(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    frame_entry(interp, receiver, key::INVOCATION).map(Some)
}

/// `StackFrameClass::getTarget`: the receiver of the send, and `.nil` on any
/// frame that is not a `METHOD` one.
fn frame_target(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    frame_entry(interp, receiver, key::TARGET).map(Some)
}

/// `StackFrameClass::getArguments`, whose `OREF_NULL` answers an empty array
/// rather than `.nil`.
fn frame_arguments(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    frame_entry(interp, receiver, key::ARGUMENTS).map(Some)
}

/// `StackFrameClass::getExecutable`: the `Method` or `Routine` the frame's
/// level was running, `.nil` for a `COMPILE` frame. A frame built without
/// one refuses: a package hook's, one built from a failure's site rather
/// than from an activation, and one whose [`executable_at`] failed.
fn frame_executable(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    match interp.heap.get(receiver).map(|held| &held.body) {
        Some(Body::Native(native)) => native
            .entry(key::EXECUTABLE)
            .map(Some)
            .ok_or_else(|| Loud::frame_without_executable().into()),
        _ => Err(Loud::receiver_class(NOT_A_FRAME).into()),
    }
}

/// `StackFrameClass::getContext`: the `RexxContext` of the activation this
/// frame was taken from -- which may since have ended, and then raises
/// `98.981` on its own next send while this frame goes on answering.
fn frame_context(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    frame_entry(interp, receiver, key::CONTEXT).map(Some)
}

/// `StackFrameClass::getTraceLine`, which `String` and `MakeString` are bound
/// to as well (`Setup.cpp:1713`, `:1716`, `:1717` -- one function under three
/// names, so the three cannot disagree).
fn frame_trace_line(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    frame_entry(interp, receiver, key::TRACE_LINE).map(Some)
}
