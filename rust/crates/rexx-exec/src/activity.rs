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

//! The state of one execution, apart from the interpreter it runs in.

use std::collections::VecDeque;

use rexx_core::ObjRef;

use crate::activation::Activation;
use crate::clause::ClauseState;
use crate::error::{Failure, FailureSite};
use crate::plan::Package;
use crate::{ActiveCondition, CallContext, FragmentLevel, NativeFrame, PendingTrap};

/// What each execution of the interpreter needs its own copy of.
pub(crate) struct Activity {
    /// A buffer lent out for a builtin call's evaluated argument values.
    pub(crate) value_buffer: Vec<Option<ObjRef>>,
    /// What each outstanding `SETLOCAL` saved, innermost last: the directory
    /// and the whole environment, which `ENDLOCAL` puts back. The oracle keeps
    /// this on the top-level activation and an internal routine's `SETLOCAL`
    /// therefore outlives its return (`platform/unix/ExternalFunctions.cpp`,
    /// and `funct.xml` says otherwise -- measured, the file is wrong).
    pub(crate) locals: Vec<(std::path::PathBuf, Vec<(Vec<u8>, Vec<u8>)>)>,
    /// The activation running right now, held in a field of its own rather
    /// than at the top of [`Activity::suspended`].
    pub(crate) running: Option<Box<Activation>>,
    /// The `TRACE` setting of whatever [`Activity::running`] holds, kept beside
    /// it rather than read through it.
    pub(crate) trace_cache: crate::trace::TraceCache,
    /// The activations that entered before [`Activity::running`], oldest
    /// first, so `suspended.last()` is the running activation's own caller.
    #[expect(
        clippy::vec_box,
        reason = "the box is the same one `running` holds, so suspending and \
                  resuming move a pointer instead of the activation"
    )]
    pub(crate) suspended: Vec<Box<Activation>>,
    /// Boxes whose activations have ended, kept for the next push rather than
    /// returned to the allocator. `Interp::recycle_activation` is what fills
    /// it and `Interp::push_activation` what drains it; both carry the
    /// reasoning.
    #[expect(
        clippy::vec_box,
        reason = "the box is the allocation being kept, so unboxing here would \
                  return the very thing this parks"
    )]
    pub(crate) spare_activations: Vec<Box<Activation>>,
    /// The native activations on the stack, innermost last. Each carries the
    /// objects an extension's handles name (D5), rooted by
    /// [`Activity::object_roots`] for exactly as long as the frame is on this
    /// stack, so a handle that outlives its activation resolves to nothing.
    pub(crate) native_handles: Vec<NativeFrame>,
    /// Frames popped off [`Activity::native_handles`] with the buffers that held
    /// objects cleared, whose allocations the next native call reuses.
    pub(crate) native_spares: Vec<NativeFrame>,
    /// The next [`NativeFrame`] number, wrapping.
    pub(crate) next_native: u32,
    /// The thread context every native call and package hook is handed,
    /// which an extension may keep for as long as this interpreter runs.
    pub(crate) thread: rexx_api::ffi::ThreadContext,
    /// `current_value_indent` and `current_clause_line`, bundled -- see
    /// `ClauseState`'s own doc comment for what the two share, the property
    /// that decides what belongs alongside them, and why they are one field
    /// rather than two.
    pub(crate) clause_state: ClauseState,
    /// The flattened `DO`/`LOOP`s the op driver has open, innermost
    /// last.
    #[expect(
        clippy::vec_box,
        reason = "the box is the point: a pass boundary takes the top out to hand it a &mut Interp beside it, and moves a pointer rather than the header's Numbers"
    )]
    pub(crate) flat_loops: Vec<Box<crate::run::FlatLoop>>,
    /// The innermost open flat loop, held apart from the stack of
    /// the ones enclosing it.
    pub(crate) flat_top: Option<Box<crate::run::FlatLoop>>,
    /// The constructs the op driver has open, innermost last, across
    /// every level of it at once.
    pub(crate) frames: Vec<crate::ir::drive::Frame>,
    /// The level state each call whose callee is running saved, innermost
    /// last.
    pub(crate) call_tails: Vec<crate::run::CallTail>,
    /// The clause regions `Interp::drive`'s levels stopped at a call op,
    /// innermost last.
    pub(crate) parked_calls: Vec<crate::ir::drive::ParkedCall>,
    /// The levels `Interp::drive` left for a callee, innermost last.
    pub(crate) parked_levels: Vec<crate::ir::drive::ParkedLevel>,
    /// The work each primitive method whose Rexx activation is running has
    /// left, innermost last.
    pub(crate) native_tails: Vec<crate::dispatch::NativeTail>,
    /// Boxes a finished loop handed back, so that entering a loop
    /// is a write into an allocation this interpreter already owns. A loop
    /// entered once per two passes is common enough -- `samples/rexxcps.rex`
    /// enters one 140,000 times to run 280,000 passes -- that an allocation
    /// per entry is charged against a saving per pass.
    #[expect(
        clippy::vec_box,
        reason = "these are allocations handed back for reuse, so the box is what is being kept"
    )]
    pub(crate) flat_spares: Vec<Box<crate::run::FlatLoop>>,
    /// A condition raised by `RAISE` whose `CALL ON` handler has not run yet.
    pub(crate) pending_traps: VecDeque<PendingTrap>,
    /// The condition whose handler is running, for `RAISE PROPAGATE` to
    /// re-raise.
    pub(crate) active_condition: Option<ActiveCondition>,
    /// The object a `RAISE ... ADDITIONAL` named, held from the raise until
    /// the condition object is built. **The raise's own value and not a
    /// rebuild of it**: measured, `additional 'JUSTONE'` puts a `String` in
    /// the directory and `additional (.array~new)` an empty `Array`, where
    /// reconstructing from the substitution list gives a one-item `Array` and
    /// nothing at all. It lives on `Activity` rather than on `Raised` because a
    /// `Raised` travels inside a `Failure` on the Rust stack, where no
    /// destructure can root it, and this field is covered by the exhaustive
    /// match in `object_roots`.
    pub(crate) pending_additional: Option<ObjRef>,
    /// The `RESULT` a condition an extension raised names, held from the
    /// raise until the condition object is built, as
    /// [`Activity::pending_additional`] is.
    pub(crate) pending_result: Option<ObjRef>,
    /// The `RC` a condition a registered command handler raised carries, held
    /// as [`Activity::pending_additional`] is: an object where the entry's value
    /// is not a rendering of the command's code, `.nil` for an entry present
    /// with no value.
    pub(crate) pending_rc: Option<ObjRef>,
    /// The condition object a `RAISE PROPAGATE` re-raises in its caller, from
    /// the propagate until a trap takes it.
    pub(crate) reraised_object: Option<ObjRef>,
    /// How many of the levels that `RAISE PROPAGATE` leaves are still to be
    /// left, which add no frame and no origin: the object has its frames.
    pub(crate) reraise_leaving: usize,
    /// **F3, found by review.** The innermost `SELECT CASE`'s own evaluated
    /// `case` text, or `None` inside a plain `SELECT` (or before any
    /// `SELECT`/`SELECT CASE` has run at all) -- the one piece of state an
    /// **absorbed** `WhenCase` needs that nothing else threads to it: a
    /// *listed* `WhenCase` gets `case_text` handed to it directly by
    /// `Select`'s own explicit arm (`run.rs`), but an absorbed one (a
    /// `WhenCase` reached only through ordinary `Op::Clause`'s region
    /// stepping, because it is itself the `THEN` consequence of a
    /// preceding `WHEN`/`WHEN CASE`, `ast.rs`'s own doc comment on
    /// `whens`) has no such hand-off -- it is stepped like any other
    /// instruction, with nothing carrying its enclosing `SELECT CASE`'s
    /// own comparison value along.
    pub(crate) current_case_text: Option<Vec<u8>>,
    /// **F3's own perimeter, found by review -- and corrected twice more,
    /// each correction found by re-verifying the previous one rather than
    /// trusting it.** When an absorbed `WhenCase` (`run.rs`'s own doc
    /// comment on that arm) takes its `Flow::Goto(false_target)` branch,
    /// whatever it lands on -- `END`'s own 7.3, or (F-EX1) `OTHERWISE`'s
    /// own marker *and its whole body*, redirected through `run_
    /// otherwise` -- reports every indent it computes **`self` spaces
    /// higher** than its own ordinary `static_indent` would give, for as
    /// long as this stays non-zero.
    pub(crate) indent_offset: usize,
    /// The absolute printed indent every clause of the **current activation
    /// level** starts from -- `0` for a program's own body, and an
    /// `INTERPRET` fragment's enclosing clause's own printed indent for the
    /// life of that fragment.
    pub(crate) activation_indent: usize,
    /// The clause a `Raised` condition escaped from, as the 1-based line and
    /// the bytes `TRACE` would echo, or `None` if nothing raised.
    pub(crate) failure_site: Option<FailureSite>,
    /// The levels that have already finished failing, innermost first --
    /// `Raised::report`'s echo stack minus its last entry.
    pub(crate) failure_sites: Vec<FailureSite>,
    /// The `StackFrame` of the level [`Activity::failure_site`] belongs to,
    /// built before that level ends.
    pub(crate) failure_frame: Option<ObjRef>,
    /// The `StackFrame`s of the levels in [`Activity::failure_sites`] that have
    /// one, innermost first: what a trapped condition's `STACKFRAMES` holds
    /// above the trapping activation.
    pub(crate) failure_frames: Vec<ObjRef>,
    /// Whether the failure an extension's method or routine call is answering
    /// is the condition it raised while running, which
    /// `NativeActivation::checkConditions` re-raises in the caller, rather
    /// than one its arguments' conversion raised on the way in.
    pub(crate) native_reraise: bool,
    /// The activation a redirection's input reader is sending `LINEIN` from,
    /// for as long as that send runs: the `RedirectionDispatcher` a condition
    /// raised straight into it meets, which takes it and ends the input.
    pub(crate) input_dispatch: Option<crate::activation::ActivationId>,
    /// Whether the dispatcher took a condition during the current send.
    pub(crate) input_dispatch_trapped: bool,
    /// A SYNTAX condition the dispatcher took, which `RedirectionDispatcher::
    /// handleError` hands to the command's own callout to raise once the
    /// command completes.
    pub(crate) input_dispatch_syntax: Option<Failure>,
    /// The innermost of those levels that has a package: its program, whose
    /// file and package a trapped condition names, and its line, which is
    /// the condition's `POSITION` (a native level has none).
    pub(crate) failure_origin: Option<(Package, Option<usize>)>,
    /// Whether the SYNTAX failure has left a level that is not an internal
    /// call, or was re-raised: `Activity::raiseException` marks it
    /// `PROPAGATED` once the raising activation does not trap it.
    pub(crate) failure_propagated: bool,
    /// Whether it was re-raised, so that `POSITION` is no longer the raise's.
    pub(crate) failure_reraised: bool,
    /// The line number every clause echo prints while an `INTERPRET`
    /// fragment is running, overriding the clause's own line in its own
    /// source.
    pub(crate) clause_line_override: Option<usize>,
    /// How many `INTERPRET` fragments are running, counted from zero outside
    /// any of them.
    pub(crate) fragment_depth: usize,
    /// Each running fragment, outermost first.
    pub(crate) fragments: Vec<FragmentLevel>,
    /// The index, in the innermost running fragment's body, of its clause
    /// being stepped.
    pub(crate) fragment_clause: usize,
    /// Whether a line typed at an interactive-debug pause is running.
    /// `RexxActivation::noTracing` includes this, so a pause's own fragment
    /// traces nothing and pauses nowhere. Written only through
    /// [`Interp::replace_debug_pause`], which keeps `trace_cache` in step.
    pub(crate) debug_pause: bool,
    /// Whether a trace line is being delivered to `.TRACEOUTPUT`. A traced
    /// clause inside that delivery writes to the buffer instead of routing
    /// again: the oracle SIGSEGVs in the one shape that reaches this
    /// (`corpus/oracle-crashes.txt` entry 14), and this crate must terminate.
    pub(crate) routing_trace: bool,
    /// Current `eval` recursion depth, and the deepest it has reached.
    pub(crate) depth: usize,
    pub(crate) max_depth: usize,
    /// The depth-1 address of the chain currently being evaluated, kept aside
    /// until that chain turns out to be the deepest one.
    pub(crate) stack_entry: usize,
    /// The two ends of the span, both from the chain that reached
    /// `max_depth`, written together so they can never disagree.
    pub(crate) stack_first: usize,
    pub(crate) stack_deepest: usize,
    /// Whether the instruction about to be stepped is allowed to be a
    /// `PROCEDURE` -- and, read the other way, whether it is the first
    /// instruction executed in its activation.
    pub(crate) procedure_permitted: bool,
    /// What [`Activity::procedure_permitted`] held when the running
    /// [`crate::ir::Op::Clause`] region opened, for the one op that needs it.
    pub(crate) region_procedure_permitted: bool,
    /// The call that entered the running activation: what `USE ARG` reads.
    pub(crate) call_context: CallContext,
    /// `RANDOM`'s generator state: the seed the next call will scramble, or
    /// `None` before any call has drawn one.
    pub(crate) random_seed: Option<u64>,
    /// `TIME('E')`/`TIME('R')`'s anchor: the clock reading (`builtin::
    /// datetime`'s microseconds-since-0001-01-01 unit) elapsed time is
    /// measured from, or `None` before any `E`/`R` call has run.
    /// ```text
    /// zz=time('E'); call burn; call sub; say 'after' time('E')   [sub does n2 = time('R')]
    ///   oracle:  inside 0.725271  inside-after-R 0.000004  after 0.725387
    ///   crate:   inside 33.889432 inside-after-R 0.000005  after 0.000013
    /// ```
    pub(crate) elapsed_anchor: Option<i64>,
    /// Whether a `TIME('R')` (or a clock read going backward) is waiting
    /// to move [`elapsed_anchor`] the next time the clock cache next
    /// refreshes -- `RexxActivation`'s own `elapsedReset` state flag
    /// (`execution/ActivationSettings.hpp:121`), consumed by
    /// `builtin::datetime::now_base_time`'s cache-miss path. See
    /// [`elapsed_anchor`]'s own doc for why the reset is lazy at all.
    pub(crate) pending_elapsed_reset: bool,
    /// The resolved paths whose `::REQUIRES` directives are still installing
    /// -- `Activity`'s own `requiresTable` (`concurrency/Activity.hpp:308`).
    pub(crate) requires_installing: Vec<Box<str>>,
    /// How many Rust frames pin this activity between the scheduler and its
    /// running driver (spec 2026-09-29 section 2.6).
    pub(crate) pin_depth: u32,
    /// The park a primitive method answered, from its answer until the
    /// activity wakes.
    pub(crate) native_park: Option<Box<crate::dispatch::NativePark>>,
    /// A started activity's send, until its first step makes it.
    pub(crate) first_send: Option<Box<crate::scheduler::StartedSend>>,
    /// Where a started activity records its send's outcome.
    pub(crate) root_then: Option<crate::dispatch::Then>,
    /// The floor of the driver a parked activity left, `None` where it parked
    /// outside one.
    pub(crate) drive_floor: Option<usize>,
    /// The floor of the driver a slice left, and the op of the clause it
    /// resumes at.
    pub(crate) sliced: Option<(usize, u32)>,
    /// How main's root driver ended where a loop other than its own ran it.
    pub(crate) root_end: Option<Result<crate::run::Ended, Failure>>,
    /// What `.context~thread` answers, assigned on first use
    /// ([`crate::Interp::activity_number`]).
    pub(crate) number: Option<u32>,
    /// The messages whose send failed with the condition now unwinding,
    /// until its condition object is built.
    pub(crate) failed_sends: Vec<ObjRef>,
    #[cfg(feature = "pinning")]
    pub(crate) pins: crate::pinning::PinStack,
}

impl Activity {
    pub(crate) fn new() -> Activity {
        Activity {
            value_buffer: Vec::new(),
            locals: Vec::new(),
            running: None,
            suspended: Vec::new(),
            spare_activations: Vec::new(),
            thread: rexx_api::ffi::ThreadContext::new(),
            native_handles: Vec::new(),
            native_spares: Vec::new(),
            next_native: 0,
            clause_state: ClauseState::new(),
            flat_loops: Vec::new(),
            flat_top: None,
            flat_spares: Vec::new(),
            frames: Vec::new(),
            call_tails: Vec::new(),
            parked_calls: Vec::new(),
            parked_levels: Vec::new(),
            native_tails: Vec::new(),
            pending_traps: VecDeque::new(),
            active_condition: None,
            pending_additional: None,
            reraised_object: None,
            reraise_leaving: 0,
            pending_result: None,
            pending_rc: None,
            current_case_text: None,
            indent_offset: 0,
            activation_indent: 0,
            failure_site: None,
            failure_sites: Vec::new(),
            failure_frame: None,
            failure_frames: Vec::new(),
            failure_origin: None,
            failure_propagated: false,
            failure_reraised: false,
            native_reraise: false,
            input_dispatch: None,
            input_dispatch_trapped: false,
            input_dispatch_syntax: None,
            clause_line_override: None,
            fragment_depth: 0,
            fragments: Vec::new(),
            fragment_clause: 0,
            debug_pause: false,
            routing_trace: false,
            depth: 0,
            max_depth: 0,
            stack_entry: 0,
            stack_first: 0,
            stack_deepest: 0,
            procedure_permitted: false,
            region_procedure_permitted: false,
            call_context: CallContext::default(),
            random_seed: None,
            elapsed_anchor: None,
            pending_elapsed_reset: false,
            requires_installing: Vec::new(),
            pin_depth: 0,
            native_park: None,
            first_send: None,
            root_then: None,
            drive_floor: None,
            sliced: None,
            root_end: None,
            number: None,
            failed_sends: Vec::new(),
            trace_cache: crate::trace::TraceCache::of(crate::trace::TraceMode::OFF, false),
            #[cfg(feature = "pinning")]
            pins: crate::pinning::PinStack::default(),
        }
    }

    /// Appends every `ObjRef` the activity must hand the collector to `out`,
    /// and names every field that does not need to be handed over.
    pub(crate) fn object_roots(&self, out: &mut Vec<ObjRef>) {
        let Activity {
            // Every push site roots the value as a temp before it lands here.
            value_buffer: _,
            running,
            trace_cache: _,
            suspended,
            // A finished activation's leftovers, overwritten at reuse and read
            // by nothing in between.
            spare_activations: _,
            // Handles for constants that are not heap objects, and the
            // innermost native call, whose frame `native_handles` roots.
            thread: _,
            native_handles,
            // A pop clears the buffers that held objects, and a push
            // overwrites the handle fields before anything reads them, so a
            // spare names nothing to root.
            native_spares: _,
            next_native: _,
            clause_state: _,
            // The `DO OVER` snapshot sits in a register held for the loop's
            // lifetime, and `RootSet` reaches a register as a temp.
            flat_loops: _,
            flat_top: _,
            frames: _,
            // A saved convention is the caller's, whose arguments and
            // receiver that caller's temps and registers root; a lent argument
            // stack holds values its caller's registers root; a parked level's
            // registers stay reserved in the arena.
            call_tails: _,
            parked_calls: _,
            parked_levels: _,
            native_tails,
            // Overwritten at reuse, as `spare_activations` is.
            flat_spares: _,
            pending_traps,
            active_condition: _,
            pending_additional,
            reraised_object,
            reraise_leaving: _,
            pending_result,
            pending_rc,
            current_case_text: _,
            indent_offset: _,
            activation_indent: _,
            failure_site: _,
            failure_sites: _,
            failure_frame,
            failure_frames,
            // A package identity and a line.
            failure_origin: _,
            failure_propagated: _,
            failure_reraised: _,
            // Flags and an activation identity.
            native_reraise: _,
            input_dispatch: _,
            input_dispatch_trapped: _,
            // A condition, whose ADDITIONAL `pending_additional` roots.
            input_dispatch_syntax: _,
            clause_line_override: _,
            fragment_depth: _,
            fragments: _,
            fragment_clause: _,
            debug_pause: _,
            routing_trace: _,
            depth: _,
            max_depth: _,
            stack_entry: _,
            stack_first: _,
            stack_deepest: _,
            procedure_permitted: _,
            region_procedure_permitted: _,
            // A running call's arguments and receiver are the caller's temps;
            // `CallContext::object_roots` is the parked case's other route.
            call_context: _,
            random_seed: _,
            elapsed_anchor: _,
            pending_elapsed_reset: _,
            requires_installing: _,
            locals: _,
            pin_depth: _,
            native_park,
            first_send,
            root_then,
            drive_floor: _,
            sliced: _,
            root_end,
            number: _,
            failed_sends,
            #[cfg(feature = "pinning")]
                pins: _,
        } = self;
        // The raise's own `ADDITIONAL`, alive between the raise and the
        // condition object that will hold it.
        out.extend(*pending_additional);
        out.extend(*pending_result);
        out.extend(*pending_rc);
        out.extend(*reraised_object);
        out.extend(*failure_frame);
        out.extend(failure_frames.iter().copied());
        out.extend(failed_sends.iter().copied());
        match root_end {
            Some(Ok(crate::run::Ended::Returned(value) | crate::run::Ended::Exited(value)))
            | Some(Err(Failure::Exited(value))) => out.extend(*value),
            _ => {}
        }
        for tail in native_tails {
            tail.object_roots(out);
        }
        if let Some(park) = native_park {
            park.object_roots(out);
        }
        if let Some(send) = first_send {
            send.object_roots(out);
        }
        if let Some(then) = root_then {
            then.object_roots(out);
        }
        // Everything a native call has been handed, and the receiver it is
        // writing object variables through. Held here rather than by the
        // collector's other routes because an extension's handle is the only
        // reference to it: nothing on the Rexx side names an object a native
        // method allocated and has not returned yet.
        for frame in native_handles {
            out.extend(frame.locals.roots());
            out.extend([frame.owner, frame.scope, frame.receiver]);
            out.extend(frame.arguments.iter().copied().flatten());
            out.extend(frame.argument_list);
            out.extend(
                [frame.additional, frame.result, frame.condition]
                    .into_iter()
                    .flatten(),
            );
        }
        // Each queued trap's condition object. Destructured rather than
        // reached by field: the match above guards `Activity`'s own fields, and
        // an `ObjRef` added inside this `VecDeque` would otherwise arrive
        // unrooted with nothing to say so.
        for PendingTrap {
            condition: _,
            rc: _,
            description: _,
            object,
            activation: _,
            queued_during_delivery: _,
            request: _,
            fragment_depth: _,
        } in pending_traps
        {
            out.extend(*object);
        }
        // The context objects of the activations on the stack. **The one
        // object an activation owns outright**: everything else it holds is
        // rooted by its slot frame, by `Interp::class_variables`, or -- a
        // send's receiver -- by the temporary `Interp::message_term` takes
        // over the sending clause. A `RexxContext` is created by
        // `Interp::context_object` and stored on the activation, and nothing
        // else refers to it. Handed over here rather than kept rooted per
        // activation because the alternative is a global root whose key has
        // to be minted, replaced and retired as activations come and go, and
        // this pays only when a collection actually happens.
        // `Activation::object_roots` is the same objects' other route, for an
        // activation a `REPLY` has parked.
        out.extend(
            running
                .iter()
                .map(std::ops::Deref::deref)
                .chain(suspended.iter().map(Box::as_ref))
                .filter_map(|activation| activation.context_object),
        );
        // The trapped condition's object, which a `CALL ON` handler's
        // activation and every callee that inherits its `CONDITION()` hold
        // once the queue has handed it over.
        out.extend(
            running
                .iter()
                .map(std::ops::Deref::deref)
                .chain(suspended.iter().map(Box::as_ref))
                .filter_map(|activation| activation.condition.as_ref()?.object),
        );
    }
}
