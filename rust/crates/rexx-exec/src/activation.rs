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

//! One activation: everything about the frame currently executing (D16).
//!
//! Moved here from Task 3's spike, which built this shape; `plan.rs` moved
//! alongside it, since the two are the halves of one design. `blocks:
//! Vec<Block>` is deliberately **not** here, and the reason is that nothing
//! reads it yet, not that its shape is unknown: the design doc's DO/LOOP
//! passage already gives one, naming the control variable's slot,
//! `to`/`by`/`for`, the iteration counter, the block's label and its `end`
//! index. Task 11 is the first code that will actually walk a block, so it
//! should pick the representation against a real reader rather than inherit
//! a guess made here. An earlier version of this comment claimed no
//! definition existed while listing it in the same sentence.
//!
//! **4b's Task 3 is the first task for which more than one of these exists
//! at a time**, and three fields carry the consequences: `body` (which code
//! body this frame runs, no longer assumed to be `main`), `trace_mode`
//! (moved off `Interp`, because a callee's `TRACE` must die with it) and
//! `settings` (which finally has a caller to inherit from). Their doc
//! comments carry the measurements; `Activation::nested` is where all three
//! are set for a callee.

use crate::Interp;
use crate::plan::{BodyKey, Plan, ProgramId};
use crate::trace::TraceMode;
use rexx_core::{ObjRef, SlotFrame};
use rexx_num::Settings;
use rexx_parse::{CodeBody, DirectiveKind, Program};
use std::collections::HashMap;
use std::rc::Rc;

/// How many ended activation boxes [`Interp::recycle_activation`] parks for
/// reuse.
///
/// Small on purpose: what the pool serves is a call and its return, which
/// needs one box back for the next call, and every extra entry is a frame's
/// worth of memory held for a depth the program has already left.
const SPARE_ACTIVATIONS: usize = 4;

/// One enabled condition trap: `SIGNAL ON cond NAME label` or `CALL ON cond
/// NAME label`.
///
/// The two differ only in this `bool`, and the difference is entirely in
/// *when* and *how* the handler runs -- measured, and the two answers are
/// not variations of one another:
///
/// * `SIGNAL ON` transfers control the instant the condition is raised. The
///   rest of the clause never runs: `signal on novalue` with `say zunset`
///   prints nothing before the handler.
/// * `CALL ON` runs the handler as an ordinary internal call at the **next
///   clause boundary**, and the clause that raised completes first. Measured
///   with `zres = one(1)` where `one` raises a trapped `USER` condition and
///   the handler assigns `zres` itself: the program prints the *handler's*
///   value, so the assignment had already stored the routine's before the
///   handler ran.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Trap {
    /// `CALL ON` rather than `SIGNAL ON`.
    pub(crate) call: bool,
    /// The label to transfer (or call) to -- `NAME label`, or the condition's
    /// own name when `NAME` is omitted. `USER foo`'s own default is `FOO`,
    /// measured: `signal on user foo` with a `foo:` label traps there.
    /// **`Rc<[u8]>` and not `Box<[u8]>`, which is a measurement.** A callee
    /// inherits its caller's trap table by clone, so every armed trap's label
    /// was copied on every call. Measured with the marginal method -- a
    /// `call` body run at N and 2N iterations, differenced -- one
    /// `signal on novalue` cost 423 user instructions per call over a program
    /// with no trap armed, and 319 with the label shared instead of copied.
    /// The oracle's own figure for the same pair is 0, so the remainder is
    /// the table and its keys, not this.
    pub(crate) label: std::rc::Rc<[u8]>,
    /// The trap is armed but held while its own `CALL ON` handler runs
    /// (`TrapHandler::disable`/`enable`, `execution/TrapHandler.cpp`).
    ///
    /// **A state and not a removal, and `CONDITION('S')` is the whole of
    /// what a program can see of the difference.** It reports `DELAY`
    /// inside the handler and `OFF` for a trap that is not there at all,
    /// measured in one program -- `call on user uc` with the handler
    /// printing `condition('S')` gives `DELAY`, and the same handler after
    /// `call off user uc` gives `OFF`. Removing and re-inserting reports
    /// `OFF` for the first.
    ///
    /// **That is the only difference, and the wider claim this comment used
    /// to make is false.** It said a removal "resurrects a trap the handler
    /// turned off". It does not: the handler runs as a nested activation
    /// with its own copy of the trap table, so its `CALL OFF` never reaches
    /// the caller's table and there is nothing to resurrect. Measured two
    /// ways -- on the oracle, a handler whose first act is `call off user
    /// uc` is entered again on a second raise; and with the removal restored
    /// here, the same program prints identically except that
    /// `CONDITION('S')` reads `OFF` where it should read `DELAY`.
    /// Commit `f03d69f1`'s message carries the superseded claim and cannot
    /// be edited.
    ///
    /// A delayed trap does not fire, and `Interp::trap_for` is what
    /// enforces that. **`Interp::caller_trap_for` deliberately does not**,
    /// which is the C++'s own line rather than an omission: `raiseCondition`
    /// matches a handler and queues it without asking whether it is delayed,
    /// and `processTraps` is what skips a delayed one. So a condition raised
    /// inside a handler by a routine the handler called is *matched*, then
    /// dropped at the clause boundary when `deliver_pending_traps`'s own
    /// `trap_for` declines it. Measured, and both interpreters agree: a
    /// handler whose first run calls a routine raising the same condition
    /// runs **once**, and the program carries on. An earlier version of this
    /// comment said every such lookup filters, which is one lookup too many.
    pub(crate) delayed: bool,
}

/// The condition a handler running in this activation was entered for: as
/// much of the oracle's condition Directory as `CONDITION()` can be answered
/// from here.
///
/// **Per activation, and that is measured rather than convenient.** The C++
/// keeps it in `settings.conditionObj`, which an internal call copies along
/// with the rest of the settings block and never writes back, so the three
/// observables are:
///
/// ```text
/// handler          condition('C')  ->  SYNTAX
///  call clearer    condition('C')  ->  SYNTAX      inherited
///  call clearer    condition('R')              then condition('C')  ->  ''
/// handler          condition('C')  ->  SYNTAX      the callee's reset died with it
/// ```
///
/// The same copy rule is why a handler's condition does not outlive its own
/// activation: a `SIGNAL ON` handler in a callee leaves the caller reporting
/// nothing once it returns.
///
/// **What is deliberately not here.** `CONDITION('A')` and `CONDITION('O')`
/// answer an `Array` and a `Directory`, neither of which this crate's value
/// model has; `builtin::state`'s `CONDITION` refuses those two options
/// loudly rather than storing something that could only be rendered wrongly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TrappedCondition {
    /// `CONDITION('C')`: the condition's own name, the same bytes
    /// [`Activation::traps`] is keyed by -- `SYNTAX`, `NOVALUE`, `USER UC`.
    pub(crate) name: Box<[u8]>,
    /// `CONDITION('E')`: the part of the condition object's `CODE` item
    /// after the dot, which only a `SYNTAX` condition has one of.
    ///
    /// Measured: `say 1/0` trapped gives `3` (the sub of 42.3), `raise
    /// syntax 40.4` gives `4`, `raise syntax 40` gives `0` -- so a missing
    /// sub is a real zero rather than an absence -- and `NOVALUE`, `USER`,
    /// `ERROR` and `FAILURE` all give the null string.
    pub(crate) code_sub: Option<u16>,
    /// `CONDITION('I')`: the trap that fired was `CALL ON` rather than
    /// `SIGNAL ON`.
    ///
    /// **This is the one thing `CONDITION()` needs that nothing else records.**
    /// `Trap::call` says how a trap *would* fire; this says how the
    /// condition being reported *did*. The two come apart the moment the
    /// handler re-arms its own trap the other way round -- measured, inside
    /// a `CALL ON USER UC` handler, `signal on user uc` leaves
    /// `CONDITION('I')` at `CALL` while `CONDITION('S')` becomes `ON`.
    pub(crate) call: bool,
    /// `CONDITION('D')`: the `RAISE ... DESCRIPTION` value, or `None` when
    /// the raise carried none. See [`Raised::description`] for the one
    /// condition whose description this cannot supply.
    ///
    /// [`Raised::description`]: crate::error::Raised::description
    pub(crate) description: Option<Vec<u8>>,
}

/// A unique identity for one activation, minted when it is pushed and never
/// reused for the life of an `Interp`.
///
/// **Added by fix round 1, because a depth does not identify an activation.**
/// `PendingTrap` first named its target activation by the activation stack's
/// own depth, which is right while that activation is on the stack and wrong
/// the moment it leaves: a later, unrelated call re-enters the same depth and
/// picks up a
/// condition that was never its. Measured both ways -- `call aa` then `call
/// cc`, with `aa` raising a trapped `USER` condition, ran the handler inside
/// `cc`; and a pending condition whose activation is unwound by an error the
/// *caller* traps is dropped by the oracle and was delivered into the next
/// routine by us.
///
/// A counter rather than a pointer or an index for the obvious reason: it
/// stays unique across a pop, which is exactly the case both defects had in
/// common.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) struct ActivationId(pub(crate) u64);

/// The `ADDRESS` environment pair an activation carries: the target a command
/// clause would be sent to, and the one a bare `ADDRESS` swaps back to.
///
/// **A pair and not a stack**, which is the whole of the bare form's
/// behaviour. Measured on the oracle with `envA`, `envB` and then four
/// consecutive bare `ADDRESS`es: `ENVA`, `ENVB`, `ENVA`, `ENVB`. A stack walks
/// backwards out of the pair instead of returning into it, and it parts
/// company almost immediately -- measured by mutation, replacing the swap with
/// `current = alternate.take()`: the alternate is wrong after **one** toggle
/// and the current after **two**. The C++ is the same two words swapping,
/// `RexxActivation::toggleAddress`.
///
/// `None` is the interpreter's default environment, which is **platform
/// supplied** (`Activity::getInstance()->getDefaultEnvironment()`, and `sh`
/// measured on this host) rather than a name this crate chooses. Both fields
/// start `None` because the oracle starts both at that same default:
/// `RexxActivation`'s own constructors set `currentAddress` and then
/// `alternateAddress = currentAddress`. Measured consequence, and the one edge
/// a survey built from working examples misses -- a bare `ADDRESS` before any
/// other `ADDRESS` swaps two equal values and so changes nothing:
///
/// ```text
/// say address()  ->  sh
/// address        ->  (no change)
/// say address()  ->  sh
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct AddressState {
    /// The environment in force. `None` is the platform default.
    ///
    /// **This is what `ADDRESS()` answers**: `BuiltinFunctions.cpp`'s
    /// `ADDRESS` is `context->getAddress()` and nothing else. It is one of
    /// five readers of `settings.currentAddress` in the C++ and the only one
    /// a Rexx program can use without issuing a command --
    /// `corpus/lang/address_env.rex`'s own header has the enumeration.
    /// Rendering `None` means naming the platform default, which
    /// `builtin::state`'s `DEFAULT_ENVIRONMENT` spells and says why.
    pub(crate) current: Option<Rc<[u8]>>,
    /// What a bare `ADDRESS` swaps `current` with.
    pub(crate) alternate: Option<Rc<[u8]>>,
}

impl AddressState {
    /// `ADDRESS env` / `ADDRESS VALUE expr`: the new target becomes current and
    /// the old current becomes the alternate.
    ///
    /// The old *alternate* is discarded, so setting the same name twice leaves
    /// both halves equal and a following bare `ADDRESS` does nothing --
    /// measured, `address envA; address envA` then three bare `ADDRESS`es all
    /// report `ENVA`.
    pub(crate) fn set(&mut self, name: Rc<[u8]>) {
        self.alternate = self.current.take();
        self.current = Some(name);
    }

    /// [`AddressState::set`] from bytes, reusing an `Rc` that already holds
    /// them.
    ///
    /// **The name a `SELECT`-free `ADDRESS env` names is a constant of the
    /// parse tree, and the one `ADDRESS VALUE address()` computes is the name
    /// already in force**, so allocating for either is an allocation whose
    /// result is a copy of something this state is holding. Measured against
    /// the tree as committed, `instructions:u` per `address SH`: 353 before
    /// and 232 after, where a bare `ADDRESS` -- the same bookkeeping with no
    /// name to hand over -- is 230.
    ///
    /// Both arms are what [`AddressState::set`] would leave behind, and
    /// `set_bytes_agrees_with_set` says so by running the two against each
    /// other over every ordering of a small set of names.
    pub(crate) fn set_bytes(&mut self, name: &[u8]) {
        // Setting the name already in force: `set` would leave both halves
        // holding it, and the second one can be the first's own `Rc`.
        if self.current.as_deref() == Some(name) {
            self.alternate = self.current.clone();
            return;
        }
        // Setting the name last swapped out: `set` would make it current and
        // the current one alternate, which is the swap this state already has.
        if self.alternate.as_deref() == Some(name) {
            std::mem::swap(&mut self.current, &mut self.alternate);
            return;
        }
        self.set(Rc::from(name));
    }

    /// Bare `ADDRESS`.
    pub(crate) fn toggle(&mut self) {
        std::mem::swap(&mut self.current, &mut self.alternate);
    }
}

/// How far a routine activation is past the point where it could still
/// announce its own `>I>`, and whether it has.
///
/// **One value where the C++ has two bools**, `traceEntryAllowed` and
/// `traceEntryDone` (`RexxActivation.hpp:634`-`635`). They are read together
/// at every site, only three of their four combinations are reachable, and
/// the one this crate got wrong was reachable only because the pair let the
/// "allowed" half be spent from a place the "done" half could not see.
///
/// **The transitions are a decay driven by clauses stepped, not by
/// instructions in a list, and that difference is the whole defect this
/// models.** The C++ clears `traceEntryAllowed` at the bottom of its own
/// instruction loop (`RexxActivation.cpp:657`-`659`), and an `IF`'s
/// then-clause, a `DO` body's clause and an `INTERPRET`'s clauses are each a
/// separate instruction in that same loop. Here they are nested *inside*
/// their enclosing clause's own step, so a clear driven by the top-level loop
/// never fires before them. Measured -- `call rtn` / `::routine rtn` /
/// `if 1=1 then trace l` / `return`: the oracle writes zero bytes to stderr
/// and this crate wrote the whole `>I>`/`<I<` pair, both at rc 0, until the
/// decay moved to [`Interp::step_in_temps_frame`], which is the one place a
/// clause is stepped at any nesting depth.
///
/// The measured table, one row per shape, `>I>` announced only where marked.
/// Every routine body below ends `return`; the caller is `call rtn`:
///
/// ```text
/// trace l                                        announced
/// /* comment */ then trace l                     announced   (not an instruction)
/// n0 = 0 then trace l                            -
/// lbl: then trace l                              -           (a LABEL is one)
/// if 1=1 then trace l                            -
/// if 1=0 then nop; else trace l                  -
/// do 1; trace l; end                             -
/// do i = 1 to 1; trace l; end                    -
/// select; when 1=1 then trace l; end             -
/// interpret "trace l"                            announced
/// interpret "nop; trace l"                       -
/// n0 = 0 then interpret "trace l"                -
/// if 1=1 then interpret "trace l"                -
/// interpret "interpret 'trace l'"                -
/// ```
///
/// **`INTERPRET` is the one construct that does not simply decay**, and the
/// last four rows are why. The C++ gives an interpret activation its own
/// `traceEntryAllowed` and gates on `tracingLabels() &&
/// parent->isMethodOrRoutine() && parent->traceEntryAllowed &&
/// !parent->traceEntryDone` (`:3644`-`:3652`). So a fragment starts its own
/// count, but only when the `INTERPRET` was the routine's own first clause,
/// and never when the parent is *another fragment* -- an interpret activation
/// is not a method or routine, which is exactly what the last row measures.
/// `Interp::enter_fragment` and `Interp::leave_fragment` are that rule.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum TraceEntry {
    /// No clause of this activation has been stepped yet.
    Pending,
    /// The clause being stepped is this activation's first, so a `TRACE`
    /// executing right now may announce.
    Allowed,
    /// A second clause has begun, or a fragment was entered from a state that
    /// could not announce. Nothing more will be announced and no `<I<` is
    /// owed.
    Spent,
    /// `>I>` has been announced, which is also the precondition for `<I<`.
    ///
    /// The exit half needs this **and** `tracingLabels()` still being in
    /// force at the end: measured, a routine whose body is `trace l` then
    /// `trace off` announces `>I>` and no `<I<` at all.
    Done,
}

impl TraceEntry {
    /// The state one more stepped clause leaves this in.
    ///
    /// `Pending` is spent by the first clause *beginning*, not by its
    /// finishing, which is what makes `Allowed` the state a `TRACE` in that
    /// clause sees. `Done` absorbs, so a routine that announced and then runs
    /// on still owes its `<I<`.
    pub(crate) fn stepped(self) -> TraceEntry {
        match self {
            TraceEntry::Pending => TraceEntry::Allowed,
            TraceEntry::Allowed | TraceEntry::Spent => TraceEntry::Spent,
            TraceEntry::Done => TraceEntry::Done,
        }
    }

    /// Whether a `TRACE` executing right now may announce `>I>`.
    pub(crate) fn may_announce(self) -> bool {
        matches!(self, TraceEntry::Allowed)
    }
}

/// How far one activation is through a `REPLY` ([`Activation::reply`]).
///
/// **Three states rather than two flags**, because `Owed` implies
/// `Issued` and a pair admits a state that means nothing: a body cannot be
/// waiting to continue past a `REPLY` that never ran.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum ReplyState {
    /// No `REPLY` has run here. The ordinary state of every activation.
    None,
    /// A `REPLY` ran and this activation's body has since continued past it.
    /// A second `REPLY` is 98.935, and a `RETURN`/`EXIT` carrying a value is
    /// 98.936/98.937.
    Issued,
    /// A `REPLY` has just handed its value out and the rest of the body is
    /// owed. `Interp::enter_method_body` reads this to park the activation
    /// instead of releasing it, and moves it to `Issued` as it does.
    ///
    /// Named for the debt rather than for the activation's position, because
    /// `Interp::suspended` is the activation stack below the running one and
    /// this is not that.
    Owed,
}

/// One activation: everything about the frame currently executing.
pub(crate) struct Activation {
    /// This activation's own identity, unique for the life of the `Interp`.
    /// See [`ActivationId`] for the two measured defects that made a depth
    /// insufficient.
    pub(crate) id: ActivationId,
    /// The program this frame is running.
    ///
    /// **A liveness anchor, and never borrowed through.** Nothing takes
    /// `&self.activation().program.…` and then calls a `&mut self`
    /// method: that is the `E0502` written out in `run_activation`. This field
    /// exists so that the `Rc` the instruction loop clones into its local has
    /// something to be cloned from, and so that a frame keeps its program
    /// alive independently of `Interp::programs`.
    ///
    /// **It records the program and not the body**, which is why [`body`]
    /// sits beside it: through 4a `run_activation` hardcoded `&program.main`,
    /// true for every activation 4a could build and false the moment an
    /// activation runs anything else. Task 3 replaced the hardcoding with a
    /// read of that field.
    ///
    /// [`body`]: Activation::body
    pub(crate) program: Rc<Program>,
    /// `program`'s own id, the durable identity `Interp::programs` hands out.
    ///
    /// The `Rc` above is a liveness anchor and cannot answer this: two
    /// activations holding the same `Rc` share an id, but an `Rc` is not a
    /// key, and recovering the index by scanning `Interp::programs` is a
    /// reverse lookup that can fail. Carried so that [`Activation::body_key`]
    /// is a field read: it is the other half of the plan and chunk caches'
    /// key, whose first half is [`body`] just below.
    ///
    /// [`body`]: Activation::body
    pub(crate) program_id: ProgramId,
    /// Which of `program`'s code bodies this activation is running: `None` is
    /// `program.main`, `Some(i)` is `program.directives[i]`'s own body.
    ///
    /// **The same shape `BodyKey::directive` carries** (`plan.rs`), decided
    /// together with it on purpose -- a selector here that denoted something
    /// other than the plan cache's key would cache one body's plan under
    /// another body's name, which is a wrong answer rather than a miss.
    /// [`body_of`] is the one function that turns the pair into a
    /// `&CodeBody`, so the two spellings cannot come apart.
    ///
    /// `Some(i)` is a `::ROUTINE` activation built by [`Activation::routine`]
    /// from `Interp::resolve_call`'s third resolution step, or a
    /// `::METHOD`/`::ATTRIBUTE` activation built by [`Activation::method`]
    /// from a resolved message send. [`Entry`] is what tells the two apart
    /// where it matters. The resolution order in
    /// front of the routine step is load-bearing rather than
    /// tidy -- internal label, then builtin, then `::ROUTINE` -- because a
    /// routine name that **collides** with a builtin must go to the builtin.
    /// Measured: `::routine max` alongside `call max 1, 9` still calls the
    /// builtin and reports 9, so a `::routine` search placed in front would
    /// silently run the wrong routine rather than fail.
    ///
    /// A quoted target is a second order and not the same one: measured,
    /// `call 'ZORKOLO'` skips the internal `zorkolo:` label and reaches the
    /// `::routine`, while `call 'MAX' 1, 9` still reaches the builtin.
    ///
    /// The lookup **upcases both sides**, unlike [`CodeBody::labels`].
    /// Measured: `::routine 'zork'` is found by `call zork`, `call 'zork'`
    /// and `call 'ZORK'` alike, and `::routine MiXeD` by `call 'mixed'`.
    /// The builtin step in front of it is the opposite, matched
    /// case-sensitively: `call 'max' 1, 9` is 43.1 where `call 'MAX' 1, 9`
    /// answers 9, which is what lets a `::routine 'max'` be reachable at all.
    ///
    /// [`CodeBody::labels`]: rexx_parse::CodeBody::labels
    pub(crate) body: Option<usize>,
    pub(crate) plan: Rc<Plan>,
    /// Names bound after `plan` was built, and the reason this field exists is
    /// the whole answer to "does a fragment's plan work against the enclosing
    /// plan's name map".
    ///
    /// It does for reads, and it cannot for writes. A fragment that introduces
    /// a name the enclosing body never mentions has to bind that name to a
    /// slot, and the binding has to outlive the fragment. Measured on the
    /// oracle:
    ///
    /// ```text
    /// interpret "zork = 42"      /* ZORK is in no instruction of this body */
    /// interpret "say zork"       /* prints 42 */
    /// ```
    ///
    /// The enclosing `plan` is an `Rc` the activation holds a clone of, so it
    /// is not uniquely owned and cannot be extended; `RootSet::grow_slots`
    /// hands out the *slot* but records no *name* for it. This map is where
    /// the name goes. `DROP (v)` has the identical hole, so this is not a
    /// fragment-only mechanism.
    pub(crate) extra: HashMap<Box<[u8]>, usize>,
    pub(crate) frame: SlotFrame,
    /// Whether this activation is the one that must `pop_slots` [`frame`],
    /// and equivalently whether the pool in it is its own.
    ///
    /// **The whole of D9r's shared-pool default lives in this one bool.** A
    /// callee with no `PROCEDURE` reuses the caller's frame, so it is `false`
    /// and the frame outlives the return; a `PROCEDURE` callee pushes a frame
    /// of its own and sets it `true`. The top-level activation owns its frame
    /// too -- `Interp::run` is what pops that one.
    ///
    /// It also decides a second thing, and getting only the first right is a
    /// silent bug: `Interp::invoke_call` moves the callee's `extra` back into
    /// the caller on return, which is correct exactly when the two shared a
    /// pool. For a `PROCEDURE` callee it would overwrite the caller's own
    /// run-time name bindings with the callee's isolated ones.
    ///
    /// [`frame`]: Activation::frame
    pub(crate) owns_frame: bool,
    /// How this activation was entered -- what an instruction whose legality
    /// depends on the entry reads. [`Entry`] carries the oracle's own table.
    pub(crate) entry: Entry,
    /// What `PARSE SOURCE`'s second word answers while this activation runs.
    ///
    /// A field rather than a function of [`entry`], because [`Entry::Routine`]
    /// is one kind for a `::ROUTINE` reached by `CALL` and for the same body
    /// reached as a function, which answer differently: [`CallType`]'s own
    /// doc has the measured table and the program for each row.
    ///
    /// [`entry`]: Activation::entry
    pub(crate) call_type: CallType,
    /// `Some` exactly for an [`Entry::Method`] activation: what its
    /// `>I>`/`<I<` lines name.
    ///
    /// Carried on the activation rather than recovered from
    /// [`Activation::body`], because neither substitution is in the directive:
    /// [`MethodIdentity`]'s own doc has the measurements.
    pub(crate) method_identity: Option<MethodIdentity>,
    /// Every name an `EXPOSE` in this activation bound to a scope pool on the
    /// receiving object, by the slot index that name resolves to here.
    ///
    /// **Empty for all but a method activation that ran an `EXPOSE`**, and the
    /// emptiness is the fast path: `Interp::variable` and its two siblings ask
    /// this first and fall straight through to the frame when there is nothing
    /// in it.
    ///
    /// **Keyed on the slot index rather than on the name** so that every route
    /// to the variable agrees without re-deriving anything. A plan-resolved
    /// read, an `INTERPRET` fragment's read and a `VALUE('V')` call all reach
    /// `Interp::slot_of` (or the plan's own map, which is the same answer), so
    /// keying on what they all produce is what makes the exposure invisible to
    /// them.
    ///
    /// **Inherited by an internal `CALL` that shares this pool**, measured: a
    /// class method exposing `v`, calling an internal label with no
    /// `PROCEDURE`, and the label assigning `v`, leaves the object variable
    /// changed. A `PROCEDURE` callee starts from an empty list and
    /// `exec_procedure` puts back only what its own `EXPOSE` list names.
    pub(crate) exposed: Vec<(usize, InstanceVar)>,
    /// How far this activation is through a `REPLY`, which is
    /// `ActivationSettings::isReplyIssued` plus the `REPLIED` execution state
    /// as one value.
    ///
    /// [`ReplyState`] has the transitions. Three readers: `REPLY` itself, for
    /// the one-per-invocation rule; `RETURN`/`EXIT` carrying a value, which
    /// has nowhere to go once a reply has been handed to the sender; and
    /// `Interp::enter_method_body`, which parks this activation rather than
    /// releasing it when the body is to continue.
    pub(crate) reply: ReplyState,
    /// Whether no instruction has yet been executed in this activation --
    /// where a label does not count as an instruction.
    ///
    /// `PROCEDURE` and `USE LOCAL` both ask "is this the *first* instruction
    /// executed", and the answer is a run-time property, not a static one.
    /// Measured on the oracle, all four shapes:
    ///
    /// ```text
    /// call sub / sub: procedure                 -> runs
    /// call sub / sub: / lbl2: / procedure       -> runs      (labels do not count)
    /// call sub / sub: nop / procedure           -> 17.1      (a NOP does)
    /// say 'main' / sub: / procedure             -> 17.1      (fell through, no call)
    /// ```
    ///
    /// The second and third together are why this is cleared per instruction
    /// *kind* rather than at the label the call jumped to, and the fourth is
    /// why [`entry`] is a separate field: a body's text cannot say
    /// whether its `PROCEDURE` is reachable, because the same instruction is
    /// legal when called and not when fallen into.
    ///
    /// `run_activation` is the only writer, and it clears this **before**
    /// stepping rather than after -- measured, `sub: interpret "procedure"`
    /// is 17.1, so a fragment does not inherit its host clause's permission.
    /// `Interp::procedure_permitted` carries the value across the one step
    /// that is allowed to use it.
    ///
    /// [`entry`]: Activation::entry
    pub(crate) first_instruction_pending: bool,
    /// How far this activation is past the point where a `>I>` could still be
    /// announced -- `RexxActivation::traceEntryAllowed` and `traceEntryDone`
    /// as one value, because the two are read together everywhere and a
    /// separate pair admits states the C++ never reaches.
    ///
    /// [`TraceEntry`] has the transitions and the measurements behind them.
    pub(crate) trace_entry: TraceEntry,
    pub(crate) pc: usize,
    /// This activation's own `NUMERIC DIGITS`/`FUZZ`/`FORM`.
    ///
    /// Per activation and not one field on `Interp` (measured, in the
    /// design's "The borrow shape"): a callee's own `NUMERIC` setting must
    /// not leak back into its caller once it returns, so each activation
    /// needs its own copy rather than sharing one. Task 6 adds the field,
    /// default-initialised, since 4a's one activation never has a caller to
    /// inherit from. Task 7's arithmetic is its first reader (`eval.rs`'s
    /// `digits()`/`form()` calls, feeding the DIGITS/FORM pair a value is
    /// rendered under at creation); Task 9's `NUMERIC` instruction is what
    /// will first mutate it, and 4b's `CALL` is what will initialise a
    /// callee's from the caller's current value instead of the default.
    /// [`Activation::nested`] is that initialisation.
    pub(crate) settings: Settings,
    /// This activation's own `TRACE` setting (D17).
    ///
    /// **Per activation since Task 3, and one field on `Interp` before
    /// that.** The `Interp` field was a deliberate 4a-only simplification and
    /// said so: 4a has exactly one frame, so there was no `return` for a
    /// callee's `TRACE OFF` to fail to survive. Measured on the oracle, `trace
    /// r` in a caller and `trace off` as the callee's first clause -- the
    /// caller's own next clause is echoed again after the `return`, so the
    /// callee's setting dies with the callee. [`Activation::nested`] inherits
    /// the caller's value at call time; nothing writes back on the way out,
    /// which is the whole of that behaviour.
    pub(crate) trace_mode: TraceMode,
    /// This activation's own `ADDRESS` environment pair.
    ///
    /// **Per activation, inherited by copy at call time, never written back**,
    /// the same one-way rule [`trace_mode`], [`settings`] and [`traps`] follow,
    /// and measured the same way. `address outer` in the main body, then a
    /// called internal routine:
    ///
    /// ```text
    /// main   address()  ->  OUTER
    ///  sub   address()  ->  OUTER      inherited
    ///  sub   address              ->   sh      the caller's alternate came too
    ///  sub   address inner ; address() -> INNER
    /// main   address()  ->  OUTER      unchanged by the callee
    /// main   address              ->   sh      the caller's own alternate
    /// ```
    ///
    /// So both halves of the pair cross into the callee, not just the current
    /// one -- the C++ copies the whole settings block
    /// (`_parent->putSettings(settings)` in the internal-call constructor).
    ///
    /// A `::routine` does **not** inherit it: its constructor sets
    /// `currentAddress` from the instance default and `alternateAddress` from
    /// that, exactly as a top-level activation does. Nothing in this crate
    /// enters a `::routine` body yet, so no code path here depends on that
    /// difference, but it is the same non-inheritance `DIGITS`/`FORM`/`FUZZ`
    /// and `TRACE` show.
    ///
    /// [`trace_mode`]: Activation::trace_mode
    /// [`settings`]: Activation::settings
    /// [`traps`]: Activation::traps
    pub(crate) address: AddressState,
    /// The condition traps enabled in this activation, keyed by the exact
    /// condition name a raise carries (`Raised::condition`) -- `SYNTAX`,
    /// `NOVALUE`, `USER FOO`, ...
    ///
    /// **Per activation, inherited by copy at call time, and never written
    /// back** -- all three measured, and each one separately:
    ///
    /// * *Inherited.* `signal on syntax` in the main body with `say 1/0`
    ///   inside a `PROCEDURE`d routine traps, and the handler reads the
    ///   *routine's* isolated variable pool, so the trap fired in the callee
    ///   rather than after unwinding to the caller. `SIGL` is the callee's
    ///   own raising line there (9 in that probe), not the caller's `call`
    ///   clause.
    /// * *By copy.* The same program with `signal off syntax` as the callee's
    ///   first clause still traps -- in the *caller*, with `SIGL` set to the
    ///   caller's `call` clause -- so turning a trap off in a callee leaves
    ///   the caller's own enabled.
    /// * *Never written back.* Symmetric with [`trace_mode`] and [`settings`]
    ///   just above, and for the same reason: the callee's map dies with its
    ///   frame.
    ///
    /// A trap is **removed when it fires** (measured: a `SIGNAL ON SYNTAX`
    /// handler whose own clause divides by zero gets the ordinary fatal
    /// report, not a second trap; and a `NOVALUE` handler reading a second
    /// unset variable gets that variable's derived name). `SIGNAL ON` inside
    /// the handler re-arms it, also measured.
    ///
    /// [`trace_mode`]: Activation::trace_mode
    /// [`settings`]: Activation::settings
    pub(crate) traps: TrapMap,
    /// The condition `CONDITION()` reports in this activation, or `None`
    /// when no handler has been entered here. [`TrappedCondition`] carries
    /// the measurements for the copy-on-call, never-write-back rule it
    /// follows along with [`traps`].
    ///
    /// [`traps`]: Activation::traps
    pub(crate) condition: Option<TrappedCondition>,
    /// `DATE`/`TIME`'s clock reading, in `builtin::datetime`'s own
    /// microseconds-since-0001-01-01 unit -- the last value this activation
    /// ever read, or `None` before its first one. **This is not itself the
    /// per-clause cache** -- see [`clock_stale`] for that half -- because
    /// `RexxActivation::getTime`'s own lazy `TIME('R')` reset
    /// (`execution/RexxActivation.cpp:3400`-`3406`) needs the *stale* value
    /// still readable one call after the clause that produced it stopped
    /// being current, to anchor the reset to. Overwriting this straight to
    /// `None` on invalidation, an earlier version of this field's own
    /// shape, made that value unrecoverable by the time a reset needed it.
    ///
    /// **Per activation, matching `ActivationSettings::timeStamp` exactly**,
    /// and not one field on `Interp` the way [`Interp::elapsed_anchor`]'s
    /// own divergence is: a nested `CALL`'s own instructions must not
    /// disturb the *caller's* cached reading, and a single `Interp`-wide
    /// field could not tell the two apart. Measured, the shape the shared
    /// brief's own probe is built from: `say time("L") burn() time("L")`,
    /// where `burn` is an internal routine that runs a real CPU burn before
    /// returning. `burn`'s own body steps its own instructions -- a `DO`
    /// clause and a `RETURN` clause, at least two calls into
    /// `step_in_temps_frame` -- through *its own* `Activation`, invalidating
    /// only `clock_stale` on the callee's frame; the caller's is a
    /// different field on a different frame and survives the call
    /// untouched, so the second `time("L")` after `burn()` returns still
    /// reads *this* clause's own cached value. A version of this field
    /// tried first on `Interp` reproduced the oracle's cache in every case
    /// except this one nested-call shape -- and this is that shape.
    ///
    /// [`clock_stale`]: Activation::clock_stale
    /// [`Interp::elapsed_anchor`]: crate::Interp::elapsed_anchor
    pub(crate) cached_clock: Option<i64>,
    /// Whether [`cached_clock`] needs a fresh read before this activation's
    /// clause may trust it -- the per-clause half [`cached_clock`]'s own
    /// doc names, set `true` once per instruction by `step_in_temps_frame`
    /// on whichever activation is executing at the time, mirroring
    /// `RexxActivation::run`'s own `settings.timeStamp.valid = false` set
    /// right after `nextInst->execute()` returns (`RexxActivation.cpp:647`).
    ///
    /// [`cached_clock`]: Activation::cached_clock
    pub(crate) clock_stale: bool,
}

/// How control arrived at an activation.
///
/// **The kinds are kept apart rather than reduced to "was it called"**,
/// because the instructions that ask disagree about which entries they want
/// and neither wants the split that question draws. Measured on the oracle,
/// one program per cell in a clean directory, with the instruction as the
/// entered body's first executed one:
///
/// ```text
/// entered by                       PROCEDURE first   USE LOCAL first
/// the program itself               17.1, rc 239      98.993, rc 158
/// an internal label, by CALL       runs, rc 0        99.910 at parse time
/// an internal label, as a function runs, rc 0        99.910 at parse time
/// a ::ROUTINE, by CALL             17.1, rc 239      98.993, rc 158
/// a ::ROUTINE, as a function       17.1, rc 239      98.993, rc 158
/// a ::METHOD                       17.1, rc 239      runs, rc 0
/// ```
///
/// **A `::ROUTINE` is not an internal call**, which is what the `PROCEDURE`
/// column turns on. 17.1's own sentence is "the first instruction executed
/// after an internal CALL or function invocation"; a `::ROUTINE` body is
/// neither, however it was reached, and neither is a `::METHOD`. Admitting
/// a `PROCEDURE` there pushes a second frame onto an activation that
/// already owns one, which is a corrupted frame stack rather than a wrong
/// answer.
///
/// The `USE LOCAL` column is a different question -- "is this a method
/// invocation" -- and the rows where it says 99.910 never reach the executor
/// at all, since `rexx-parse` refuses a `USE LOCAL` that is not its body's
/// first instruction (`exec_use`'s own doc has the shapes that were tried).
///
/// Every reader matches on this exhaustively and without a wildcard, so an
/// entry kind added here cannot be left unanswered at any of them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Entry {
    /// The program's own activation, which [`Interp::run`] starts.
    ///
    /// [`Interp::run`]: crate::Interp::run
    TopLevel,
    /// A label in the running program's own source, reached by `CALL` or by
    /// a function invocation -- the one entry a `PROCEDURE` is legal in.
    InternalCall,
    /// A `::ROUTINE` directive's body, reached either way.
    Routine,
    /// A `::METHOD` directive's body, reached by a message send.
    Method,
}

/// `PARSE SOURCE`'s second word: the *calling context* the running activation
/// was entered under, which the oracle keeps as `settings.calltype`
/// (`execution/ActivationSettings.hpp:179`) and renders straight into the
/// source string (`RexxActivation.cpp:4601`).
///
/// **This is not [`Entry`] under another name**, and reducing it to one would
/// reintroduce the defect it was written for. Measured on the oracle, one
/// program per row in a clean directory, each printing the second word from
/// the body named:
///
/// ```text
/// the program itself                          COMMAND
/// an internal label, by CALL                  the enclosing activation's
/// an internal label, as a function            the enclosing activation's
/// a ::ROUTINE, by CALL                        SUBROUTINE
/// a ::ROUTINE, as a function                  FUNCTION
/// a ::METHOD, by a message send               METHOD
/// a ::ATTRIBUTE GET or SET, by a message send METHOD
/// ```
///
/// `Entry::Routine` is one kind for every `::ROUTINE` row above and cannot
/// tell them apart, which is why this is carried beside it rather than
/// derived from it: measured, one `::routine` body reached each way in a
/// single program
/// answers `SUBROUTINE` from the `CALL` and `FUNCTION` from the function
/// invocation. And the label rows inherit rather than answering a value of
/// their own -- measured, a `::routine` invoked as a function whose body then
/// `CALL`s an internal label reads `FUNCTION` inside that label -- which is
/// [`Inherited`]'s job and why this field is one of its members.
///
/// An `INTERPRET` fragment has no row because it pushes no activation, so it
/// reads whatever its enclosing one answers; measured all the same, a
/// fragment inside a `::METHOD` body reads `METHOD`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum CallType {
    /// A program run as a command, which is how the `rexx` front end starts
    /// the top-level activation (`RexxStartDispatcher.cpp:104`, `RXCOMMAND`).
    Command,
    /// A `::ROUTINE` body reached by `CALL`.
    Subroutine,
    /// A `::ROUTINE` body reached as a function.
    Function,
    /// A `::METHOD` or `::ATTRIBUTE` body reached by a message send.
    Method,
}

impl CallType {
    /// The word itself, as `PARSE SOURCE` spells it.
    pub(crate) fn token(self) -> &'static [u8] {
        match self {
            CallType::Command => b"COMMAND",
            CallType::Subroutine => b"SUBROUTINE",
            CallType::Function => b"FUNCTION",
            CallType::Method => b"METHOD",
        }
    }
}

/// What a `::METHOD` activation knows about the send that entered it.
///
/// **Two of the three are what `>I>`/`<I<` name**: the substitutions message
/// 101018's method form takes (`rexxmsg.xml:6480`, `Method <q>&1</q> with
/// scope <q>&2</q> in package <q>&3</q>.`), and neither is recoverable from
/// the directive alone -- the oracle's first substitution is
/// `getMessageName()`, the name the *send* used, which is the dictionary key
/// and so differs from the directive's own spelling for an `::ATTRIBUTE`
/// setter; the second is the scope the resolution came from. The third,
/// `receiver`, is what `EXPOSE` binds against, and the scope is read twice
/// over because it decides that as well.
pub(crate) struct MethodIdentity {
    /// The message name, already upcased by the parser -- measured,
    /// `::method MiXeD` announces `"MIXED"` and `::method "quoted"`
    /// announces `"QUOTED"`.
    pub(crate) name: Box<[u8]>,
    /// The defining class, whose `~id` is printed unmodified -- measured,
    /// `::class 'k'` announces `with scope "k"`.
    ///
    /// **Also the pool `EXPOSE` binds into**, which is what makes one object
    /// hold one name at two values at once: the scope is where the method was
    /// declared, not where the send arrived, so a method inherited from a
    /// superclass reaches that superclass's pool on the same receiver.
    pub(crate) scope: ObjRef,
    /// The object the send was addressed to -- `SELF`.
    ///
    /// Here rather than read back out of the `SELF` slot when `EXPOSE` wants
    /// it. The slot is an ordinary variable a body may assign to, so reading
    /// it would make the pool `EXPOSE` binds depend on what the body has done
    /// to a name, and the pool is a property of the send.
    pub(crate) receiver: ObjRef,
}

/// One method body a `REPLY` left owed, off every stack until it is resumed.
///
/// **The slots are copied out rather than left where they were.** A
/// `SlotFrame` nests -- `RootSet::pop_slots` asserts that the frame it closes
/// is the top one -- so an activation cannot keep its frame open while the
/// activations above it close. `slots` is that frame's contents in frame
/// order, and `Interp::resume_reply` pushes a frame of the same length and
/// writes them back.
///
/// `parked` is what keeps every `ObjRef` in all three of the other fields
/// reachable while this sits in the queue; [`Activation::object_roots`] has
/// what goes into it and what it cannot see.
pub(crate) struct DeferredReply {
    pub(crate) activation: Box<Activation>,
    /// The calling convention the method was entered under: what `ARG()`,
    /// `USE ARG` and a send's own caller resolution read. Restored around the
    /// resumed body exactly as `Interp::enter_method_body` restores it around
    /// the first half.
    pub(crate) context: crate::CallContext,
    pub(crate) slots: Vec<Option<ObjRef>>,
    pub(crate) parked: rexx_core::Parked,
}

/// One variable an `EXPOSE` bound: which object's pools hold it, which of that
/// object's pools, and under what name.
///
/// **All three travel together and none of them is derivable at the point of
/// use.** The owner is not `SELF` (a body may reassign that name), the scope
/// is the method's declaring class rather than the receiver's own, and the
/// name is the spelling the pool is keyed on rather than the slot index the
/// activation reached it by.
#[derive(Clone, Debug)]
pub(crate) struct InstanceVar {
    /// The object whose [`rexx_core::ScopePools`] hold this variable.
    ///
    /// **Not the receiver.** A class object is not an arena object, so it has
    /// no `Body::Instance` of its own to hold pools; `Interp` gives it one and
    /// this names that object. A class object is the only receiver an `EXPOSE`
    /// accepts here -- `Loud::expose_receiver` refuses the rest -- so this is
    /// always such an object, and always one `Interp::class_variables` roots.
    pub(crate) owner: ObjRef,
    /// Which of the owner's pools -- the running method's declaring class.
    pub(crate) scope: ObjRef,
    /// The name the pool is keyed on, upcased as every variable name here is.
    pub(crate) name: Box<[u8]>,
}

impl Activation {
    /// A fresh top-level activation: no run-time bindings yet, default
    /// `NUMERIC` settings, `TRACE` off. What `Interp::run` starts every
    /// program with.
    ///
    /// [`Activation::nested`] is the sibling a `CALL` uses, and the two are
    /// deliberately not one function with a flag: a fresh top-level run and a
    /// nested call begin from genuinely different starting settings, and
    /// folding them together would need parameters that are meaningless on
    /// one of the two paths.
    pub(crate) fn new(
        id: ActivationId,
        program: Rc<Program>,
        program_id: ProgramId,
        plan: Rc<Plan>,
        frame: SlotFrame,
    ) -> Activation {
        Activation {
            id,
            program,
            program_id,
            body: None,
            plan,
            extra: HashMap::new(),
            frame,
            owns_frame: true,
            entry: Entry::TopLevel,
            // The word `rexx p.rex` puts here: the front end starts a program
            // as `RXCOMMAND`, whose string is `COMMAND`.
            call_type: CallType::Command,
            method_identity: None,
            exposed: Vec::new(),
            reply: ReplyState::None,
            first_instruction_pending: true,
            trace_entry: TraceEntry::Pending,
            pc: 0,
            settings: Settings::default(),
            // `NORMAL` and not `OFF`: the two behave identically here and a
            // program can tell them apart, measured -- `say trace()` as the
            // first clause of a program with no `TRACE` instruction prints
            // `N`.
            trace_mode: TraceMode::NORMAL,
            address: AddressState::default(),
            traps: TrapMap::default(),
            condition: None,
            cached_clock: None,
            clock_stale: true,
        }
    }

    /// The activation a `CALL` pushes: it starts at `pc`, and it **inherits
    /// every field [`Inherited`] carries** from the caller rather than
    /// defaulting them.
    ///
    /// The struct is the enumeration, so this doc names no list of its own:
    /// a field added there is inherited here by construction, and a contract
    /// repeating the list would say what it carried when it was written.
    /// [`Inherited`]'s own doc states what qualifies a field for it.
    ///
    /// `traps` is the one worth pointing at from here -- see
    /// [`Activation::traps`] for the three probes that pin it, including the
    /// one that separates "inherited" from "checked in the caller after
    /// unwinding", which every two-level program answers the same way and
    /// only a `PROCEDURE`d callee tells apart.
    ///
    /// Every inheritance is measured, and all are one-way -- the callee
    /// starts from the caller's value and never writes back. `numeric digits
    /// 7` in a caller, `numeric digits 3` in the callee: the callee sees 7 on
    /// entry, reports 3 after its own instruction, and the caller still
    /// reports 7 after the `return`. `trace r` in a caller and `trace off` in
    /// the callee: the callee's clauses stop echoing and the caller's resume.
    /// Copying the values in here is what makes both true, since the callee's
    /// own fields are then simply dropped with its frame.
    ///
    /// `frame` is the caller's own `SlotFrame` for a callee with no
    /// `PROCEDURE` (D9r's shared pool, the default Task 3 implemented), so
    /// this constructor does not decide the pool -- and, since Task 5, it
    /// never does: the callee is always born sharing, and its `PROCEDURE`
    /// arm is what swaps in a frame of its own if it has one. Nothing here
    /// inspects the callee's first instruction, because whether that
    /// instruction is a legal `PROCEDURE` is not knowable from the body's
    /// text (`first_instruction_pending`'s own doc has the four measured
    /// shapes).
    ///
    /// **A `::ROUTINE` is not this constructor's shape and takes no
    /// [`Inherited`] at all** -- [`Activation::routine`] is its own, and the
    /// reason it is separate is that a routine inherits **none** of the
    /// fields this one copies. Measured, one probe per field, each with a
    /// caller that set the value and a routine that reads it back:
    ///
    /// ```text
    /// numeric digits 7 / fuzz 2 / form engineering    routine: 9 0 SCIENTIFIC
    /// address system                                  routine: address() = sh
    /// signal on syntax name mytrap, routine has its   the routine's own mytrap
    ///   own mytrap: label and raises 1/0              never runs; the caller's does
    /// inside a SIGNAL ON SYNTAX handler, condition()  routine: the null string
    /// trace r in the caller                           routine: trace() = N,
    ///                                                 none of its clauses echoed
    /// ```
    ///
    /// The pool is a further difference and is not a field of either
    /// constructor: a routine's `frame` is one this crate pushed for it and
    /// `owns_frame` is true, where a `CALL`ed label shares its caller's.
    /// Measured: a caller holding `vv = 'CALLER'` calling a routine that says
    /// `vv` prints the derived name `VV`, the routine's own write to `vv` is
    /// not visible after the return, and `RESULT` and `SIGL` read inside the
    /// routine are their own uninitialised names rather than the caller's
    /// values.
    ///
    /// The parameter list is over clippy's threshold and stays that way: the
    /// three fields that could be bundled are `program`, `program_id` and
    /// `body`, and bundling them would put a type between every reader and
    /// `Activation::program`, which is the field the instruction loop clones
    /// its `Rc` from. [`Inherited`] is the bundle that pays for itself,
    /// because the fields in it share a property rather than a caller.
    #[allow(
        clippy::too_many_arguments,
        reason = "the alternative bundle would wrap the field every instruction loop reads"
    )]
    pub(crate) fn nested(
        id: ActivationId,
        program: Rc<Program>,
        program_id: ProgramId,
        body: Option<usize>,
        plan: Rc<Plan>,
        frame: SlotFrame,
        pc: usize,
        inherited: Inherited,
    ) -> Activation {
        Activation {
            id,
            program,
            program_id,
            body,
            plan,
            extra: HashMap::new(),
            frame,
            owns_frame: false,
            entry: Entry::InternalCall,
            call_type: inherited.call_type,
            method_identity: None,
            exposed: Vec::new(),
            reply: ReplyState::None,
            first_instruction_pending: true,
            trace_entry: TraceEntry::Pending,
            pc,
            settings: inherited.settings,
            trace_mode: inherited.trace_mode,
            address: inherited.address,
            traps: inherited.traps,
            condition: inherited.condition,
            // Not inherited, matching every other field this constructor
            // does not take from `inherited`: a fresh `RexxActivation`
            // constructs a fresh, invalid `RexxDateTime timeStamp`
            // regardless of its caller, and this is that same "start
            // invalid" rather than something `Inherited` should carry.
            cached_clock: None,
            clock_stale: true,
        }
    }

    /// The activation a call into a `::ROUTINE` directive pushes: it starts
    /// at instruction 0 of `directives[body]`'s own body, in a pool of its
    /// own, and it inherits **nothing**.
    ///
    /// **The absence of an [`Inherited`] parameter is the contract**, not a
    /// convenience. [`Activation::nested`]'s own doc carries the six probes
    /// that measure it, one per field it would otherwise have copied, and
    /// there is no arm here that could accidentally start copying one.
    ///
    /// `owns_frame` is true, so `Interp::invoke_call` pops the frame on the
    /// way out and does **not** move `extra` back into the caller. That is
    /// the same pair `PROCEDURE` already sets, and it is right here for a
    /// stronger reason than there: a routine has a different `CodeBody` and
    /// therefore a different [`Plan`], so a name means a different slot index
    /// on each side and a binding carried across would land on an unrelated
    /// variable.
    ///
    /// `call_type` is the one thing the *invocation form* decides rather than
    /// the directive: [`CallType::Subroutine`] for `CALL name` and
    /// [`CallType::Function`] for `name(...)`, which is why it is a parameter
    /// where every other field here is fixed. [`CallType`]'s own doc has the
    /// measurement.
    pub(crate) fn routine(
        id: ActivationId,
        program: Rc<Program>,
        program_id: ProgramId,
        body: usize,
        plan: Rc<Plan>,
        frame: SlotFrame,
        call_type: CallType,
    ) -> Activation {
        Activation {
            id,
            program,
            program_id,
            body: Some(body),
            plan,
            extra: HashMap::new(),
            frame,
            owns_frame: true,
            entry: Entry::Routine,
            call_type,
            method_identity: None,
            exposed: Vec::new(),
            reply: ReplyState::None,
            first_instruction_pending: true,
            trace_entry: TraceEntry::Pending,
            pc: 0,
            settings: Settings::default(),
            trace_mode: TraceMode::NORMAL,
            address: AddressState::default(),
            traps: TrapMap::default(),
            condition: None,
            cached_clock: None,
            clock_stale: true,
        }
    }

    /// The activation a message send into a `::METHOD` body pushes.
    ///
    /// **Everything [`Activation::routine`] settles, settled the same way**,
    /// and each half is measured rather than carried over by analogy:
    ///
    /// * it inherits nothing -- `trace i` in the caller does not echo the
    ///   method's own clauses, and the method's clauses echo at indent 0 even
    ///   when the sending clause sits two `DO` levels deep;
    /// * it owns its frame, because the body is a different [`CodeBody`] with
    ///   a plan of its own;
    /// * `PROCEDURE` as its first instruction is 17.1, the same answer a
    ///   `::ROUTINE` gets ([`Entry`]'s own table has the row).
    ///
    /// What differs is [`Entry::Method`], which `USE LOCAL` reads,
    /// `method_identity`, which the `>I>`/`<I<` pair reads, and
    /// [`CallType::Method`], which `PARSE SOURCE`'s second word reads -- and
    /// that last one is fixed here where [`Activation::routine`] takes it as
    /// a parameter, because a message send is a single form while a routine
    /// call's own form -- `CALL name` against `name(...)` -- decides it.
    pub(crate) fn method(
        id: ActivationId,
        program: Rc<Program>,
        program_id: ProgramId,
        body: usize,
        plan: Rc<Plan>,
        frame: SlotFrame,
        identity: MethodIdentity,
    ) -> Activation {
        Activation {
            id,
            program,
            program_id,
            body: Some(body),
            plan,
            extra: HashMap::new(),
            frame,
            owns_frame: true,
            entry: Entry::Method,
            call_type: CallType::Method,
            method_identity: Some(identity),
            exposed: Vec::new(),
            reply: ReplyState::None,
            first_instruction_pending: true,
            trace_entry: TraceEntry::Pending,
            pc: 0,
            settings: Settings::default(),
            trace_mode: TraceMode::NORMAL,
            address: AddressState::default(),
            traps: TrapMap::default(),
            condition: None,
            cached_clock: None,
            clock_stale: true,
        }
    }

    /// The key this activation's body is cached under, in both the plan
    /// cache and the chunk cache.
    ///
    /// The two halves are [`Activation::program_id`] and
    /// [`Activation::body`], and neither is derived here: `body` is already
    /// the same selector `BodyKey::directive` carries (that field's own doc
    /// says why the two spellings cannot come apart), and `program_id` is the
    /// id the loader issued for `program`. So this is a field read, and a
    /// body entered through any path is looked up under the key its plan was
    /// built under.
    pub(crate) fn body_key(&self) -> BodyKey {
        BodyKey {
            program: self.program_id,
            directive: self.body,
        }
    }

    /// Appends every `ObjRef` this activation holds to `out`.
    ///
    /// **For an activation that is off every stack**, which is what a `REPLY`
    /// leaves behind: while it is on the stack, `SELF` in its own frame and
    /// `Interp::class_variables` root the same objects, and this type is not
    /// walked by the collector at all. Parked, neither holds, so the values
    /// have to be handed to `RootSet::park`.
    ///
    /// **The destructuring is exhaustive and has no `..`**, so a field added
    /// to any of the three types below is a compile error here rather than a
    /// value that silently stops being rooted. It cannot see an `ObjRef`
    /// appearing inside `Settings`, `TrapMap`, `AddressState` or
    /// `TrappedCondition`, none of which holds one; the instrument for that is
    /// `run_program_collect_every_alloc`, which collects at every allocation
    /// and so reaches a missed root as a wrong answer rather than as luck.
    pub(crate) fn object_roots(&self, out: &mut Vec<ObjRef>) {
        let Activation {
            id: _,
            program: _,
            program_id: _,
            body: _,
            plan: _,
            extra: _,
            frame: _,
            owns_frame: _,
            entry: _,
            call_type: _,
            method_identity,
            exposed,
            reply: _,
            first_instruction_pending: _,
            trace_entry: _,
            pc: _,
            settings: _,
            trace_mode: _,
            address: _,
            traps: _,
            condition: _,
            cached_clock: _,
            clock_stale: _,
        } = self;
        if let Some(MethodIdentity {
            name: _,
            scope,
            receiver,
        }) = method_identity
        {
            out.push(*scope);
            out.push(*receiver);
        }
        for (
            _,
            InstanceVar {
                owner,
                scope,
                name: _,
            },
        ) in exposed
        {
            out.push(*owner);
            out.push(*scope);
        }
    }
}

/// Everything a callee starts from its caller's copy of, gathered so
/// [`Activation::nested`] takes one argument for the lot.
///
/// **The grouping is the concept and not a parameter-count workaround.**
/// Each field here is measured to be inherited at call time and measured
/// *not* to be written back on return -- the callee's copy simply dies with
/// its frame -- and each field's own doc comment on `Activation` carries the
/// transcript. They travel as one argument because that pair of properties
/// is what they have in common, which a run of separate parameters between
/// `pc` and the end of the signature would not say.
///
/// A field belongs here when both halves hold. `extra` deliberately does not:
/// it is moved back into the caller on return for a shared-pool callee
/// (`owns_frame`'s own doc comment), which is the opposite of one-way.
pub(crate) struct Inherited {
    pub(crate) call_type: CallType,
    pub(crate) settings: Settings,
    pub(crate) trace_mode: TraceMode,
    pub(crate) address: AddressState,
    pub(crate) traps: TrapMap,
    pub(crate) condition: Option<TrappedCondition>,
}

/// The code body a `(program, selector)` pair denotes: `None` is
/// `program.main`, `Some(i)` is `program.directives[i]`'s own body.
///
/// A free function rather than a method on `Activation`, because the borrow
/// it returns has to outlive every `&mut self` call in `run_activation` --
/// the discipline that function's own doc comment writes out at length. Its
/// caller holds an `Rc<Program>` in a local and passes `&local`, so the
/// `&CodeBody` is rooted in the local and not in `self`.
///
/// `None` on a selector that names a directive with no body of its own,
/// rather than a panic: `Some(i)` can only be built from a resolution
/// step that already looked at `directives[i]`, so a mismatch is an internal
/// inconsistency, and this crate's rule for those is to fail loudly at the
/// caller rather than abort the process here.
///
/// A `::METHOD`'s and a `::ATTRIBUTE`'s bodies are here beside `::ROUTINE`'s
/// because a method activation runs one: `::ROUTINE`, `::METHOD` and
/// `::ATTRIBUTE` are the directive kinds that own a [`CodeBody`], and each
/// field's own doc says when it is `None` (a generating option, for both
/// method forms).
pub(crate) fn body_of(program: &Program, selector: Option<usize>) -> Option<&CodeBody> {
    match selector {
        None => Some(&program.main),
        Some(index) => match &program.directives.get(index)?.kind {
            DirectiveKind::Routine(routine) => routine.body.as_ref(),
            DirectiveKind::Method(method) => method.body.as_ref(),
            DirectiveKind::Attribute(attribute) => attribute.body.as_ref(),
            _ => None,
        },
    }
}

impl Interp {
    /// Mints the next [`ActivationId`]. Every `Activation` this crate pushes
    /// gets its identity from here and nowhere else, which is what makes
    /// "unique for the life of the `Interp`" a property of the code rather
    /// than a convention.
    pub(crate) fn next_activation_id(&mut self) -> ActivationId {
        let id = ActivationId(self.next_activation_id);
        self.next_activation_id += 1;
        id
    }

    pub(crate) fn activation(&self) -> &Activation {
        self.running.as_deref().expect("a live activation")
    }

    pub(crate) fn activation_mut(&mut self) -> &mut Activation {
        self.running.as_deref_mut().expect("a live activation")
    }

    /// The running activation, or `None` where nothing is running.
    ///
    /// For the caller that asks *whether* one is running rather than
    /// assuming it: everything else wants [`Interp::activation`] and its
    /// `expect`.
    pub(crate) fn running_activation(&self) -> Option<&Activation> {
        self.running.as_deref()
    }

    /// The running activation's own caller, or `None` at the outermost
    /// level.
    pub(crate) fn caller_activation(&self) -> Option<&Activation> {
        self.suspended.last().map(Box::as_ref)
    }

    /// [`Interp::caller_activation`] for a writer.
    pub(crate) fn caller_activation_mut(&mut self) -> Option<&mut Activation> {
        self.suspended.last_mut().map(Box::as_mut)
    }

    /// How many activations are live, the running one included.
    ///
    /// This is what `Vec::len` answered while the running activation sat at
    /// the top of the same vector, and every caller reading a *depth* --
    /// `MAX_ACTIVATION_DEPTH`, an `INTERPRET` fragment's queue, the driver's
    /// own entry depth -- means this number and not the suspended count.
    pub(crate) fn activation_depth(&self) -> usize {
        self.suspended.len() + usize::from(self.running.is_some())
    }

    /// Makes `activation` the running one and suspends whatever was.
    ///
    /// **The box comes from [`Interp::recycle_activation`]'s pool when one is
    /// waiting**, so a program whose loop calls a routine allocates the frame
    /// once instead of once per call. Overwriting the spare drops whatever the
    /// last activation left in it, which is the same drop the `Box` would have
    /// run when it was freed -- what is saved is the allocator round trip, and
    /// on a probe whose loop is one `CALL` into a label that returns at once,
    /// `malloc` and `free` were 6.5% of the program between them.
    pub(crate) fn push_activation(&mut self, activation: Activation) {
        let boxed = match self.spare_activations.pop() {
            Some(mut spare) => {
                *spare = activation;
                spare
            }
            None => Box::new(activation),
        };
        // `trace_cache`'s own invariant: the setting travels with whichever
        // activation is running, and this changes which one that is.
        self.trace_cache = boxed.trace_mode;
        if let Some(outer) = self.running.replace(boxed) {
            self.suspended.push(outer);
        }
    }

    /// Keeps an ended activation's box for the next [`Interp::
    /// push_activation`], instead of returning it to the allocator.
    ///
    /// **The contents are left in it and dropped at reuse.** They are the
    /// activation that just ended, so nothing reads them; clearing them here
    /// would be the same drop moved earlier and a write of the whole struct
    /// besides. What they keep alive is one program's `Rc` and one plan's,
    /// both of which `Interp::programs` holds anyway.
    ///
    /// **Capped, because the pool is fed by returns and drained by calls, so a
    /// recursion that unwinds a thousand levels would otherwise leave a
    /// thousand frames parked.** Past the cap the box is simply dropped; the
    /// depth that pays for the pool is the shallow, repeated one.
    pub(crate) fn recycle_activation(&mut self, ended: Box<Activation>) {
        if self.spare_activations.len() < SPARE_ACTIVATIONS {
            self.spare_activations.push(ended);
        }
    }

    /// Ends the running activation and resumes its caller, answering the
    /// activation that ended.
    ///
    /// Answers the box rather than its contents, so that ending an
    /// activation moves a pointer where returning it by value would copy the
    /// whole of it back out.
    pub(crate) fn pop_activation(&mut self) -> Option<Box<Activation>> {
        let ended = self.running.take()?;
        self.running = self.suspended.pop();
        // The resumed caller's setting, or `OFF` where nothing is left to
        // resume -- which is the state `Interp::new` starts in.
        self.trace_cache = self
            .running
            .as_deref()
            .map_or(TraceMode::OFF, |resumed| resumed.trace_mode);
        Some(ended)
    }

    /// The `TRACE` setting in force right now: the *running* activation's.
    ///
    /// Returns a copy rather than a borrow, and `TraceMode` is `Copy` for
    /// exactly this reason. Every reader is inside a condition that also
    /// calls a `&mut self` method in the same expression -- `if
    /// self.trace_mode().all && let Some(..) = self.clause_site(..)` is the
    /// shape, sixteen times over -- and a borrow of `self` held across those
    /// would be the same `E0502` `run_activation`'s doc comment writes out.
    pub(crate) fn trace_mode(&self) -> TraceMode {
        // **The cache is checked here rather than trusted**, which is what
        // closes the set of places that maintain it: any path that changes
        // the running activation or its setting without going through one of
        // them reddens this at the next clause, in a debug run of anything at
        // all.
        debug_assert_eq!(
            self.running_activation()
                .map(|activation| activation.trace_mode),
            Some(self.trace_cache),
            "the cached TRACE setting is not the running activation's"
        );
        self.trace_cache
    }

    /// Sets the running activation's `TRACE`. Only the `TRACE` instruction
    /// and this crate's own tests call it; a callee inherits its starting
    /// value through [`Activation::nested`] instead, never through here.
    pub(crate) fn set_trace_mode(&mut self, mode: TraceMode) {
        self.activation_mut().trace_mode = mode;
        self.trace_cache = mode;
    }
}

/// One of the condition names the language fixes, used as an index into a
/// [`BuiltinTraps`] slot.
///
/// `USER <name>` is not here: its spelling is half program text, so it lives
/// in [`TrapMap`]'s map along with everything else this does not recognise.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Builtin {
    Any,
    Error,
    Failure,
    Halt,
    LostDigits,
    NoMethod,
    NoString,
    NotReady,
    NoValue,
    Syntax,
}

impl Builtin {
    const COUNT: usize = 10;

    /// The slot `name` names, or `None` for a name that gets a map entry.
    ///
    /// **Correctness does not rest on this list being complete.** A spelling
    /// it does not answer for goes to `TrapMap`'s map, which is where every
    /// name went before there were slots at all, so a name missing from here
    /// is slower and not wrong. The list is `CONDITIONS` in `rexx-parse`'s
    /// own token table without the two entries that are not trap keys:
    /// `USER`, which only ever reaches a table as the `USER <name>` form, and
    /// `PROPAGATE`, which is `RAISE`'s.
    fn of(name: &[u8]) -> Option<Builtin> {
        Some(match name {
            b"ANY" => Builtin::Any,
            b"ERROR" => Builtin::Error,
            b"FAILURE" => Builtin::Failure,
            b"HALT" => Builtin::Halt,
            b"LOSTDIGITS" => Builtin::LostDigits,
            b"NOMETHOD" => Builtin::NoMethod,
            b"NOSTRING" => Builtin::NoString,
            b"NOTREADY" => Builtin::NotReady,
            b"NOVALUE" => Builtin::NoValue,
            b"SYNTAX" => Builtin::Syntax,
            _ => return None,
        })
    }
}

/// A slot per [`Builtin`], allocated only once something is armed.
#[derive(Clone, Default)]
pub(crate) struct BuiltinTraps([Option<Trap>; Builtin::COUNT]);

/// The condition traps armed in one activation.
///
/// **Two stores, because the keys come from two places.** Every name the
/// language fixes gets a slot; `USER <name>` and anything [`Builtin::of`]
/// does not recognise gets a map entry. An activation that arms nothing --
/// the overwhelming majority -- holds a null pointer and an empty map, and a
/// callee inheriting its caller's table copies both without allocating.
///
/// **`Rc` rather than inline slots, and a pointer rather than nothing, are
/// two separate measurements.** Inline slots would put a `Trap`-sized cell
/// per condition into `Activation`, whose layout is on the variable-read
/// path: putting the whole table behind an `Rc` once *shrank* `Activation`
/// and cost 0.26% to 1.66% on every benchmark axis, two of which make no
/// calls at all. `Rc` rather than `Box` is what makes the per-call copy free
/// rather than one allocation, since a callee that arms nothing never writes.
#[derive(Clone, Default)]
pub(crate) struct TrapMap {
    builtin: Option<Rc<BuiltinTraps>>,
    user: HashMap<Box<[u8]>, Trap>,
}

impl TrapMap {
    pub(crate) fn get(&self, name: &[u8]) -> Option<&Trap> {
        match Builtin::of(name) {
            Some(which) => self.builtin.as_ref()?.0[which as usize].as_ref(),
            None => self.user.get(name),
        }
    }

    pub(crate) fn get_mut(&mut self, name: &[u8]) -> Option<&mut Trap> {
        match Builtin::of(name) {
            Some(which) => Rc::make_mut(self.builtin.as_mut()?).0[which as usize].as_mut(),
            None => self.user.get_mut(name),
        }
    }

    pub(crate) fn insert(&mut self, name: Box<[u8]>, trap: Trap) {
        match Builtin::of(&name) {
            Some(which) => {
                let slots = self
                    .builtin
                    .get_or_insert_with(|| Rc::new(BuiltinTraps::default()));
                Rc::make_mut(slots).0[which as usize] = Some(trap);
            }
            None => {
                self.user.insert(name, trap);
            }
        }
    }

    pub(crate) fn remove(&mut self, name: &[u8]) {
        match Builtin::of(name) {
            Some(which) => {
                if let Some(slots) = self.builtin.as_mut() {
                    Rc::make_mut(slots).0[which as usize] = None;
                }
            }
            None => {
                self.user.remove(name);
            }
        }
    }

    /// `#[cfg(test)]` because nothing outside the tests asks this: every
    /// caller in the interpreter wants the trap itself, not whether one is
    /// there.
    #[cfg(test)]
    pub(crate) fn contains_key(&self, name: &[u8]) -> bool {
        self.get(name).is_some()
    }
}

#[cfg(test)]
mod address_tests {
    use super::AddressState;
    use std::rc::Rc;

    /// **[`AddressState::set_bytes`] leaves what [`AddressState::set`] would**,
    /// which is the whole of its licence to skip the allocation: it is an
    /// optimisation of `set`, not a second rule about what `ADDRESS` does.
    ///
    /// Driven over every sequence of names drawn from a set small enough that
    /// repeats, alternations and fresh names all occur -- which is what
    /// reaches both of its arms and the fall-through. A `bare` step is a plain
    /// `ADDRESS`, so the toggling that decides which half holds what is in the
    /// sequences too.
    #[test]
    fn set_bytes_agrees_with_set() {
        const NAMES: [&[u8]; 3] = [b"AAA", b"BB", b"C"];
        let mut reached_current = 0;
        let mut reached_alternate = 0;
        for encoded in 0..4usize.pow(6) {
            let mut byte = encoded;
            let (mut fast, mut slow) = (AddressState::default(), AddressState::default());
            for _ in 0..6 {
                let step = byte % 4;
                byte /= 4;
                match step {
                    3 => {
                        fast.toggle();
                        slow.toggle();
                    }
                    name => {
                        let name = NAMES[name];
                        if fast.current.as_deref() == Some(name) {
                            reached_current += 1;
                        } else if fast.alternate.as_deref() == Some(name) {
                            reached_alternate += 1;
                        }
                        fast.set_bytes(name);
                        slow.set(Rc::from(name));
                    }
                }
                assert_eq!(
                    (fast.current.as_deref(), fast.alternate.as_deref()),
                    (slow.current.as_deref(), slow.alternate.as_deref()),
                    "set_bytes and set disagree after sequence {encoded}"
                );
            }
        }
        // **That the situations arise, not that the arms took them.** An arm
        // that tests the wrong half falls through to the allocating path and
        // still answers correctly, so equivalence cannot see it; what this
        // pins is that the sequences above put a name back that is already
        // current, and one that is already the alternate, so the equivalence
        // above is checked over both.
        assert!(
            reached_current > 0 && reached_alternate > 0,
            "the sequences never set a name already held, so the equivalence was not checked \
             over the shapes the shortcuts exist for: current {reached_current}, alternate \
             {reached_alternate}"
        );
    }
}
