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
//!
//! A child of [`super`] for the reason [`super::rexx_info`] is one: the rows
//! point at [`super::NativeMethod`]s, whose parameter list names types only
//! that module can.
//!
//! # A context is a live handle and a frame is a snapshot
//!
//! `RexxContext` holds a `RexxActivation *` and every one of its readers calls
//! `checkValid` first (`ContextClass.cpp:147`), so a context whose activation
//! has ended raises `98.981`. A `StackFrameClass` holds no activation at all --
//! its constructor copies the type, name, executable, target, arguments,
//! traceback line, line number and invocation id out of the activation
//! (`RexxActivation::createStackFrame`, `RexxActivation.cpp:5006`) -- so it
//! answers those after the frame is gone.
//!
//! Measured on the oracle, rc 0, both objects captured inside `snapshot('p1',
//! 'p2')` and read after it returned: `f~name` is `SNAPSHOT`, `f~line` is the
//! line the frame was built at, `f~traceLine` is that clause's own echo,
//! `f~type` is `ROUTINE`, `f~target` is `The NIL object`, `f~arguments~items`
//! is `2`, and `c~name` is `Error 98.981 Target RexxContext is no longer
//! active.` So the two representations here are different in kind and not
//! merely in lifetime: a frame is built from [`Snapshot`]'s entries and a
//! context is resolved back to its activation on every send.
//!
//! # Which activation a context names
//!
//! [`Interp::context_depth`] scans the activation stack for the one whose
//! `Activation::context_object` is this receiver, and its `None` is
//! `checkValid`'s null pointer. Nothing else can answer it: the object is
//! created by `Interp::context_object` and stored on the activation, and an
//! activation that has returned no longer stands anywhere.
//!
//! # `~stackFrames` is the whole current stack, not the receiver's own
//!
//! `RexxContext::getStackFrames` delegates to `activity->generateStackFrames`
//! (`concurrency/Activity.cpp:1141`), which walks the **activity's** frame
//! chain from the innermost. Measured: a context captured in `r` and sent
//! `~stackFrames` from inside a routine `r` called two levels deeper answers
//! that deeper stack, not `r`'s. The `skipFirst` argument removes the native
//! `stackFrames` frame itself, which this crate never pushes.

use super::{Arity, Cleared, Failure, Interp, Loud, NativeMethod, ObjRef, Raised, hash};
use crate::activation::{Activation, Entry};
use crate::plan::ProgramId;
use rexx_core::{BehaviourId, Body};

/// `RexxContext`'s and `StackFrame`'s instance methods. Chained into
/// `ObjectModel::build` beside [`super::NATIVE_METHODS`].
///
/// Every row is declared at a parameter count of zero
/// (`Setup.cpp:1218`-`:1232` and `:1706`-`:1717`), so an argument is refused
/// before a body runs -- measured, `.context~digits(1)` and
/// `.context~stackFrames[1]~type(1)` are both `93.902 Too many arguments in
/// invocation of method; 0 expected.` on the oracle and here.
pub(super) static NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    ("RexxContext", "ARGS", Arity::Fixed(0), context_args),
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
///
/// `AddClassMethod("New", ..., A_COUNT)` at `Setup.cpp:1213` and `:1702`, both
/// bodies `reportException(Error_Unsupported_new_method, getId())` and nothing
/// else -- the same site `Pointer` and `Buffer` share. Measured, oracle rc 163
/// both: `NEW method is not supported for the RexxContext class.` and the same
/// for `StackFrame`.
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
///
/// **The `NativeObject` entry map is the storage rather than a struct of its
/// own**, because every field is an `ObjRef` and the collector already traces
/// those entries -- a parallel side table keyed by the frame's own handle
/// would be a set of roots nothing walks, which is project memory
/// `oorexx-rust-fresh-allocations-in-a-closure` one layer up.
mod key {
    pub(super) const TYPE: &[u8] = b"TYPE";
    pub(super) const NAME: &[u8] = b"NAME";
    pub(super) const LINE: &[u8] = b"LINE";
    pub(super) const INVOCATION: &[u8] = b"INVOCATION";
    pub(super) const TARGET: &[u8] = b"TARGET";
    pub(super) const ARGUMENTS: &[u8] = b"ARGUMENTS";
    pub(super) const CONTEXT: &[u8] = b"CONTEXT";
    pub(super) const TRACE_LINE: &[u8] = b"TRACELINE";
}

/// One activation's fields as `StackFrameClass`'s constructor copies them,
/// gathered before anything is allocated.
///
/// **Read out of the activation in one pass and allocated from afterwards**,
/// which is the borrow shape rather than a preference: every value below is
/// built with `&mut Interp` and the activation is reached through `&self`.
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
    /// Where in the activation stack the `RexxContext` `receiver` belongs,
    /// counted from the innermost -- `0` is the running activation.
    ///
    /// `None` is `RexxContext::checkValid`'s null `activation`
    /// (`ContextClass.cpp:147`): a context whose activation has ended stands
    /// nowhere, and every reader on that class turns it into `98.981`.
    ///
    /// **A scan rather than a map from object to activation**, because the
    /// answer has to go stale exactly when the activation ends and a map
    /// would have to be pruned there. The depth is bounded by the call depth
    /// and this is an introspection path.
    fn context_depth(&self, receiver: ObjRef) -> Option<usize> {
        self.frames()
            .position(|activation| activation.context_object == Some(receiver))
    }

    /// The activation the `RexxContext` `receiver` names, or `98.981`.
    fn context_frame(&self, receiver: ObjRef) -> Result<&Activation, Failure> {
        let depth = self
            .context_depth(receiver)
            .ok_or_else(|| Failure::from(Raised::context_not_active()))?;
        self.frame_at(depth)
            .ok_or_else(|| Failure::from(Raised::context_not_active()))
    }

    /// `RexxActivation::getIdntfr` (`execution/RexxActivation.cpp:94`): this
    /// activation's `~invocation`, minted on the first ask.
    fn invocation_of(&mut self, depth: usize) -> Option<u32> {
        if let Some(found) = self.frame_at(depth)?.invocation {
            return Some(found);
        }
        self.next_invocation += 1;
        let minted = self.next_invocation;
        self.frame_at_mut(depth)?.invocation = Some(minted);
        Some(minted)
    }
}

/// The whole of `RexxContext::checkValid` at a reader that answers straight
/// out of the activation: the depth, or `98.981`.
fn depth_of(interp: &Interp, receiver: ObjRef) -> Result<usize, Failure> {
    interp
        .context_depth(receiver)
        .ok_or_else(|| Raised::context_not_active().into())
}

/// `RexxContext::getDigits`: `activation->digits()`, the setting **in force**
/// at that context.
///
/// **Not the default, which is what `.RexxInfo~digits` answers** -- the two
/// classes give one method name two meanings, and the difference is measured:
/// oracle rc 0, after `numeric digits 5; numeric form engineering; numeric
/// fuzz 2`, `.context~digits .context~form .context~fuzz` is `5 ENGINEERING 2`
/// where `.RexxInfo~`'s three are `9 SCIENTIFIC 0`.
fn context_digits(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let digits = interp.context_frame(receiver)?.settings.digits();
    Ok(Some(interp.counted(digits as usize)))
}

/// `RexxContext::getFuzz`: `activation->fuzz()`, in force -- see
/// [`context_digits`].
fn context_fuzz(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let fuzz = interp.context_frame(receiver)?.settings.fuzz();
    Ok(Some(interp.counted(fuzz as usize)))
}

/// `RexxContext::getForm`: `activation->form()` as one of the two global
/// names, in force -- see [`context_digits`].
fn context_form(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let form = interp.context_frame(receiver)?.settings.form();
    let text: &[u8] = match form {
        rexx_num::Form::Scientific => b"SCIENTIFIC",
        rexx_num::Form::Engineering => b"ENGINEERING",
    };
    Ok(Some(interp.text(text)))
}

/// `RexxContext::getLine`: `activation->getContextLine()`, the line of the
/// clause that context is **currently executing**.
///
/// [`crate::activation::ClauseSnapshot`] is where the value is kept and why it
/// cannot be derived from `Activation::pc`.
fn context_line(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let depth = depth_of(interp, receiver)?;
    let line = interp.clause_of(depth).line;
    Ok(Some(interp.counted(line)))
}

/// `RexxContext::getName`: `activation->getCallname()` -- the name this
/// context was invoked under, and the program's own path at the top level.
///
/// **The name as the caller wrote it.** Measured, oracle rc 0: `call MiXeD`
/// into `::routine MiXeD` answers `MIXED` (the parser upcased the symbol) and
/// `call 'lower'` into `::routine 'lower'` answers `lower`.
fn context_name(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let name = interp.context_frame(receiver)?.invoked_as().to_vec();
    Ok(Some(interp.text_built(name)))
}

/// `RexxContext::getInvocation`: `activation->getIdntfr()`.
fn context_invocation(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let depth = depth_of(interp, receiver)?;
    let id = interp
        .invocation_of(depth)
        .ok_or_else(|| Failure::from(Raised::context_not_active()))?;
    Ok(Some(interp.counted(id as usize)))
}

/// `RexxContext::getThread`: `activation->getActivity()->getIdntfr()`.
///
/// **A constant, and it is already wrong.** `Activity::getIdntfr`
/// (`concurrency/Activity.cpp:108`) mints one id per *system thread* off a
/// counter of its own, so the first thread to ask is `1`. This crate runs
/// every activation on the one interpreter thread, and `Object~start` does
/// **not** refuse: it runs the send, so a method that has `REPLY`d is on
/// another thread on the oracle and on the same one here. Measured, rc 0 and
/// stderr identical, a `::method` that replies and then reads
/// `.context~thread`: the oracle answers a different id from the main
/// thread's `1` and this answers `1`. The id itself is not fixed -- it counts
/// system threads the interpreter has touched -- so what diverges is that it
/// differs at all, not which number it is.
fn context_thread(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    depth_of(interp, receiver)?;
    Ok(Some(interp.counted(1)))
}

/// `RexxContext::getInterpreter`:
/// `activation->getActivity()->getInstance()->getIdntfr()`.
///
/// **A constant, and unlike [`context_thread`] it is correct**: the id is
/// minted per `InterpreterInstance` off a third counter, and a `rexx-run`
/// process creates one instance, so there is one instance to have an id and
/// `1` is its. Nothing a program can do makes a second -- `~start` makes a
/// second *thread* within the one instance, which is why the two rows part.
/// Measured, rc 0 both sides, a `::method` that `REPLY`s and then reads both:
/// `interp= 1` on the oracle and here, where `thread=` differs.
fn context_interpreter(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    depth_of(interp, receiver)?;
    Ok(Some(interp.counted(1)))
}

/// `RexxContext::getRS`: `activation->getContextReturnStatus()`, which is
/// `.nil` unless `settings.isReturnStatusSet()`.
///
/// **`.nil` is the answer for every program this crate can run, and the state
/// it reads is the one a command clause writes.** `settings.returnStatus` is
/// set by `RexxActivation::command`; a command clause is D12/Phase 7's and
/// fails loudly here, which is the same position `.RS` itself is in
/// (`Interp::rexx_variable`, `environment.rs`). So this is not a constant
/// standing in for something unimplemented -- it is the unset branch, and
/// nothing this crate runs can take the other one.
fn context_rs(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    depth_of(interp, receiver)?;
    Ok(Some(ObjRef::NIL))
}

/// `RexxContext::getCondition`: a **copy** of the condition object the
/// activation is handling, and `.nil` when it is handling none.
///
/// **The `.nil` arm is answered and the other refuses.** Building the object
/// is `CONDITION('O')`'s own job and that is not implemented -- `builtin::
/// state::condition` refuses option `O` loudly with the same message -- so
/// answering `.nil` inside a handler would be a wrong answer where the oracle
/// hands back a `Directory`. Measured, oracle rc 0: `.context~condition` is
/// `The NIL object` outside a handler.
fn context_condition(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if interp.context_frame(receiver)?.condition.is_some() {
        return Err(Loud::builtin_option_object("CONDITION", b'O', "a Directory").into());
    }
    Ok(Some(ObjRef::NIL))
}

/// `RexxContext::getPackage`: the package of the program **the receiver's own
/// activation** belongs to (`ContextClass.cpp:160`).
///
/// **The one route to a running program's package object**, which is why it
/// is a row of its own: measured, `.Array~package~name` is `REXX`, so a class
/// the bootstrap registered reaches the interpreter's own package and not this
/// one.
///
/// **The same object `~package` answers for a class that program declared**,
/// because both go through `Interp::package_object`'s cache under one key.
/// Measured, oracle rc 0, with `::class K public`: `.context~package ==
/// .K~package` is `1`, `.context~package == .context~package` is `1`, and
/// `.context~package == .Array~package` is `0`.
fn context_package(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let program = interp.context_frame(receiver)?.program_id;
    Ok(Some(
        interp.package_object(crate::plan::Package::Program(program)),
    ))
}

/// `RexxContext::getArgs`: the arguments the context's activation was entered
/// with, as a fresh `Array`.
///
/// An omitted position holds its place as an empty slot, the same hole
/// `arg(n, 'A')` reports. Measured, oracle rc 0: `call rtn 'aa', , 'cc'` reads
/// `~items` `2` and `~size` `3` inside `rtn`, with `a[2] == .nil` true.
fn context_args(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let arguments = interp.context_frame(receiver)?.invoked_with().to_vec();
    Ok(Some(array_of_slots(interp, arguments)))
}

/// `RexxContext::getVariables`: `activation->getAllLocalVariables()`, a fresh
/// `Directory` of the context's own variable pool.
///
/// **Only the names that have a value**, which is
/// `VariableDictionary::getVariableDirectory`'s own test -- a name the plan
/// reserved and nothing assigned, and a name a `DROP` cleared, are both
/// absent. Measured, oracle rc 0, on a program whose only assignments are
/// `me`, `vv` and a stem: the directory has `~items` `3`.
///
/// **A stem is one entry under its own trailing-period name and a compound is
/// none.** Measured: with `a. = 'def'` and `a.1 = 'one'`, the keys are `A.`
/// and `B` alone, `d['A.']~class~id` is `Stem`, and `d~hasIndex('A.1')` is
/// `0`. That falls out here rather than being special-cased: this crate's plan
/// binds a compound-shaped name to a slot of its own that no read or write
/// ever fills (`plan::CompoundName`, whose reads go through the stem's slot),
/// so the value test excludes it.
///
/// **`SELF` and `SUPER` appear in a method**, measured -- the oracle's local
/// dictionary holds them and so does this crate's frame, since
/// `Interp::enter_method_body` writes them into slots like any other name.
fn context_variables(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let depth = depth_of(interp, receiver)?;
    let mut entries = Vec::new();
    {
        let activation = interp
            .frame_at(depth)
            .ok_or_else(|| Failure::from(Raised::context_not_active()))?;
        let named = activation
            .plan
            .names
            .iter()
            .map(|(name, slot)| (&**name, *slot))
            .chain(activation.extra.iter().map(|(name, slot)| (&**name, *slot)));
        for (name, slot) in named {
            if let Some(value) = interp.variable_in(activation, slot) {
                entries.push((name.to_vec(), value));
            }
        }
    }
    hash::directory_of(interp, entries).map(Some)
}

/// `RexxContext::getExecutable`: `activation->getExecutable()`, the `Method`
/// or `Routine` object this context is running.
///
/// **The object the rest of the interpreter already hands out for that same
/// executable, not a fresh one**, and the identity is measured: oracle rc 0,
/// inside `::routine r`, `.context~executable == .routines['R']` is `1`, and
/// inside `::method m` of `::class kk`, `.context~executable ==
/// .kk~method('M')` is `1`. So a `::ROUTINE` answers out of the `.ROUTINES`
/// table and a `::METHOD` out of `Interp::method_objects`; only a program's
/// own main section has no existing cache and gets
/// [`Interp::program_routine_objects`].
fn context_executable(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let depth = depth_of(interp, receiver)?;
    executable_at(interp, depth).map(Some)
}

/// [`context_executable`]'s answer for the activation at `depth`.
fn executable_at(interp: &mut Interp, depth: usize) -> Result<ObjRef, Failure> {
    let activation = interp
        .frame_at(depth)
        .ok_or_else(|| Failure::from(Raised::context_not_active()))?;
    let program = activation.program_id;
    match (
        activation.entry,
        activation.body,
        &activation.method_identity,
    ) {
        (Entry::Method, _, Some(identity)) => {
            let scope = identity.scope;
            let name = identity.name.to_vec();
            interp.method_executable(scope, &name)
        }
        (Entry::Routine, Some(directive), _) => {
            let name = routine_entry_name(interp, program, directive)?;
            let table = interp
                .package_string_table(program, crate::environment::PackageTable::Routines)
                .ok_or_else(|| Loud::receiver_class("a routine whose package has no table"))?;
            match interp.heap.get(table).map(|held| &held.body) {
                Some(Body::Native(native)) => native
                    .entry(&name)
                    .ok_or_else(|| Loud::receiver_class("a routine with no package entry").into()),
                _ => Err(Loud::receiver_class("a package table this crate did not build").into()),
            }
        }
        _ => Ok(interp.program_routine_object(program)),
    }
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
        .get(program.0)
        .ok_or_else(|| Failure::from(Loud::receiver_class("a program this crate did not load")))?;
    match source.directives.get(directive).map(|found| &found.kind) {
        Some(rexx_parse::DirectiveKind::Routine(routine)) => Ok(routine.name.to_ascii_uppercase()),
        _ => Err(Loud::receiver_class("a routine activation with no directive").into()),
    }
}

/// `RexxContext::getStackFrames`: every frame of the running stack, innermost
/// first.
///
/// **The activity's frames and not the receiver's own**, which is the
/// delegation `Activity::generateStackFrames` makes and is measured: a context
/// captured in one routine answers the stack in force where it is *sent*, not
/// where it was made.
///
/// **`skipFirst` has nothing to remove here.** The oracle passes `true` to
/// drop the native `stackFrames` invocation's own frame; this crate runs a
/// native method inside the sending activation and pushes none, so the
/// innermost frame is already the sender's.
fn context_stack_frames(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    depth_of(interp, receiver)?;
    let depths: Vec<usize> = (0..interp.frames().count()).collect();
    let frame = interp.roots.push_frame();
    let mut slots = Vec::with_capacity(depths.len());
    for depth in depths {
        let object = build_frame(interp, depth)?;
        interp.roots.push_temp(object);
        slots.push(Some(object));
    }
    let array = interp.alloc_with(
        BehaviourId::ARRAY,
        Body::Array {
            dimensions: None,
            slots,
        },
    );
    interp.roots.pop_frame(frame);
    interp.roots.push_temp(array);
    Ok(Some(array))
}

/// One `StackFrame` for the activation at `depth` --
/// `RexxActivation::createStackFrame` (`RexxActivation.cpp:5006`).
fn build_frame(interp: &mut Interp, depth: usize) -> Result<ObjRef, Failure> {
    let invocation = interp.invocation_of(depth);
    // Created here when the activation never asked for one, which is what
    // `createStackFrame`'s own `getContextObject()` does -- see
    // `Interp::context_object_at`.
    let context = interp.context_object_at(depth);
    let mut snapshot = read_snapshot(interp, depth)?;
    let frame = interp.roots.push_frame();
    let kind = interp.text(snapshot.kind);
    interp.roots.push_temp(kind);
    let name = interp.text_built(std::mem::take(&mut snapshot.name));
    interp.roots.push_temp(name);
    let line = interp.counted(snapshot.clause.line);
    let invocation = match invocation {
        // `StackFrameClass::getInvocation` answers `.nil` for a zero id,
        // which is the frame kinds a native activation builds and which this
        // crate has none of; the id is minted here rather than left at zero.
        None => ObjRef::NIL,
        Some(id) => interp.counted(id as usize),
    };
    interp.roots.push_temp(invocation);
    let trace_text = trace_line_text(interp, &snapshot);
    let trace_line = interp.text(&trace_text);
    interp.roots.push_temp(trace_line);
    let arguments = array_of_slots(interp, std::mem::take(&mut snapshot.arguments));
    interp.roots.push_temp(arguments);
    let class = interp.object_model().stack_frame;
    let object = interp.native_instance(class);
    let entries = [
        (key::TYPE, kind),
        (key::NAME, name),
        (key::LINE, line),
        (key::INVOCATION, invocation),
        (key::TARGET, snapshot.target.unwrap_or(ObjRef::NIL)),
        (key::ARGUMENTS, arguments),
        (key::CONTEXT, context.unwrap_or(ObjRef::NIL)),
        (key::TRACE_LINE, trace_line),
    ];
    for (name, value) in entries {
        let held = interp
            .heap
            .get_mut(object)
            .expect("just allocated and rooted");
        let Body::Native(native) = &mut held.body else {
            unreachable!("allocated as Body::Native by native_instance")
        };
        native.set_entry(name, value);
    }
    // `StackFrameClass::stringValue`/`makeString`, which answer the traceback
    // line where `defaultName` answers `a StackFrame` -- the one native class
    // in this crate that separates the two. Without it an `Array` holding a
    // frame joins on the default name: measured, oracle rc 0,
    // `say .array~of(f)` is the traceback line where `f~objectName` is
    // `a StackFrame`.
    let held = interp
        .heap
        .get_mut(object)
        .expect("just allocated and rooted");
    let Body::Native(native) = &mut held.body else {
        unreachable!("allocated as Body::Native by native_instance")
    };
    native.set_string_value(&trace_text);
    interp.roots.pop_frame(frame);
    // Re-pushed above the frame that has just closed, which is the shape
    // `Interp::line_array` and `array_of_texts` already have: the temp
    // `native_instance` took for this object was inside that frame, so
    // without this the frame comes back to its caller rooted by nothing.
    interp.roots.push_temp(object);
    Ok(object)
}

/// The activation at `depth` read out in one pass, before anything allocates.
fn read_snapshot(interp: &Interp, depth: usize) -> Result<Snapshot, Failure> {
    let activation = interp
        .frame_at(depth)
        .ok_or_else(|| Failure::from(Raised::context_not_active()))?;
    // `RexxActivation::createStackFrame`'s own cascade, whose whole value set
    // is `StackFrameClass.cpp`'s `FRAME_*` constants. `FRAME_INTERPRET` needs
    // an activation for a fragment, which `run_fragment` does not push, and
    // `FRAME_COMPILE` belongs to the parser
    // (`LanguageParser::createStackFrame`, `parser/LanguageParser.cpp:866`).
    // The phase's `found-not-fixed-register.md` is where which kinds this
    // crate reaches is recorded, because that is a boundary that moves.
    let (kind, target) = match activation.entry {
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
    Ok(Snapshot {
        kind,
        name: activation.invoked_as().to_vec(),
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
///
/// **Built by the same function every `*-*` echo is** ([`crate::trace::
/// push_clause`]), so the two cannot come to disagree about the six-wide line
/// number field, the marker or the indent. The trailing newline is the echo
/// sink's and not part of the string: measured, `f~traceLine` is
/// `    40 *-* f = .context~stackFrames` with nothing after it.
fn trace_line_text(interp: &Interp, snapshot: &Snapshot) -> Vec<u8> {
    let text = interp
        .programs
        .get(snapshot.program.0)
        .and_then(|program| {
            let span = crate::activation::body_of(program, snapshot.body)?
                .instructions
                .get(snapshot.clause.index)?
                .clause_span
                .clone();
            program.source.join_span(span)
        })
        .map(std::borrow::Cow::into_owned)
        .unwrap_or_default();
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

/// A fresh `Array` over `slots`, an omitted position left empty.
fn array_of_slots(interp: &mut Interp, slots: Vec<Option<ObjRef>>) -> ObjRef {
    let array = interp.alloc_with(
        BehaviourId::ARRAY,
        Body::Array {
            dimensions: None,
            slots,
        },
    );
    interp.roots.push_temp(array);
    array
}

/// The value one `StackFrame` snapshot entry holds.
///
/// The refusal is an internal inconsistency rather than a program's doing: a
/// receiver reaches here only by resolving one of these names at
/// `StackFrame`, and every frame this crate hands out is built by
/// [`build_frame`] with all of them filled.
fn frame_entry(interp: &Interp, receiver: ObjRef, name: &[u8]) -> Result<ObjRef, Failure> {
    match interp.heap.get(receiver).map(|held| &held.body) {
        Some(Body::Native(native)) => native
            .entry(name)
            .ok_or_else(|| Loud::receiver_class("a stack frame this crate did not build").into()),
        _ => Err(Loud::receiver_class("a stack frame this crate did not build").into()),
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
