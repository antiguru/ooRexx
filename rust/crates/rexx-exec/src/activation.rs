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

use crate::Interp;
use crate::options::ConditionSyntax;
use crate::plan::{BodyKey, Plan, ProgramId};
use crate::trace::TraceMode;
use rexx_core::{NameMap, ObjRef, SlotFrame};
use rexx_num::Settings;
use rexx_parse::{CodeBody, DirectiveKind, Program};
use rustc_hash::FxHashMap;
use std::rc::Rc;

/// How many ended activation boxes [`Interp::recycle_activation`] parks for
/// reuse.
const SPARE_ACTIVATIONS: usize = 4;

/// One enabled condition trap: `SIGNAL ON cond NAME label` or `CALL ON cond
/// NAME label`.
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
    pub(crate) delayed: bool,
}

/// The condition a handler running in this activation was entered for: as
/// much of the oracle's condition Directory as `CONDITION()` can be answered
/// from here.
/// ```text
/// handler          condition('C')  ->  SYNTAX
///  call clearer    condition('C')  ->  SYNTAX      inherited
///  call clearer    condition('R')              then condition('C')  ->  ''
/// handler          condition('C')  ->  SYNTAX      the callee's reset died with it
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TrappedCondition {
    /// `CONDITION('C')`: the condition's own name, the same bytes
    /// [`Activation::traps`] is keyed by -- `SYNTAX`, `NOVALUE`, `USER UC`.
    pub(crate) name: Box<[u8]>,
    /// `CONDITION('E')`: the part of the condition object's `CODE` item
    /// after the dot, which only a `SYNTAX` condition has one of.
    pub(crate) code_sub: Option<u16>,
    /// `CONDITION('I')`: the trap that fired was `CALL ON` rather than
    /// `SIGNAL ON`.
    pub(crate) call: bool,
    /// `CONDITION('D')`: the `RAISE ... DESCRIPTION` value, or `None` when
    /// the raise carried none. See [`Raised::description`] for the one
    /// condition whose description this cannot supply.
    pub(crate) description: Option<Vec<u8>>,
    /// `CONDITION('O')`: the condition object, built when the condition is
    /// trapped rather than when it is asked for. **Eagerly, and that is the
    /// observable rather than a preference:** its `STACKFRAMES` is the stack
    /// as it stood at the raise, and a `CALL ON` handler runs in a frame that
    /// did not exist then, so building it on the ask would report a stack
    /// that never was.
    ///
    /// An `ObjRef` here is reachable only through this struct, so
    /// [`Activation::object_roots`] names it -- the exhaustive destructure
    /// there guards `Activation`'s own fields and would not have caught a new
    /// one inside this type.
    pub(crate) object: Option<ObjRef>,
}

/// A unique identity for one activation, minted when it is pushed and never
/// reused for the life of an `Interp`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) struct ActivationId(pub(crate) u64);

/// The `ADDRESS` environment pair an activation carries: the target a command
/// clause would be sent to, and the one a bare `ADDRESS` swaps back to.
/// ```text
/// say address()  ->  sh
/// address        ->  (no change)
/// say address()  ->  sh
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct AddressState {
    /// The environment in force. `None` is the platform default.
    pub(crate) current: Option<Rc<[u8]>>,
    /// What a bare `ADDRESS` swaps `current` with.
    pub(crate) alternate: Option<Rc<[u8]>>,
}

/// What one activation remembers about an interactive-debug session.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct DebugState {
    /// `wasSourceTraced`: the banner is printed once per activation.
    pub(crate) source_traced: bool,
    /// `wasDebugPromptIssued`: so is the prompt.
    pub(crate) prompt_issued: bool,
    /// `settings.traceSkip`: how many further pauses to skip, from a numeric
    /// `TRACE` given at a pause.
    pub(crate) skip: i64,
    /// `isDebugBypassed`: a setting changed from inside a pause ends that
    /// pause without reading another line.
    pub(crate) bypass: bool,
    /// The setting a negative skip count is suppressing, put back when the
    /// count runs out. Suppression replaces the setting outright rather than
    /// riding a flag the sink would have to test: measured, testing one cost
    /// `bench-programs/varlookup.rex` 4.4% in `instructions:u`.
    pub(crate) saved: Option<TraceMode>,
}

/// The `ADDRESS ... WITH` table an activation carries, keyed by the upcased
/// environment name (`settings.ioConfigs`, a `StringTable`).
pub(crate) type IoConfigs = NameMap<Box<[u8]>, Rc<rexx_parse::AddressIo>>;

impl AddressState {
    /// `ADDRESS env` / `ADDRESS VALUE expr`: the new target becomes current and
    /// the old current becomes the alternate.
    pub(crate) fn set(&mut self, name: Rc<[u8]>) {
        self.alternate = self.current.take();
        self.current = Some(name);
    }

    /// [`AddressState::set`] from bytes, reusing an `Rc` that already holds
    /// them.
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
    Done,
}

impl TraceEntry {
    /// The state one more stepped clause leaves this in.
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
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum ReplyState {
    /// No `REPLY` has run here. The ordinary state of every activation.
    None,
    /// A `REPLY` ran and this activation's body has since continued past it.
    /// A second `REPLY` is 98.935, and a `RETURN`/`EXIT` carrying a value is
    /// 98.936/98.937.
    Issued,
    /// A `REPLY` ran here and the rest of the body has yet to move to a new
    /// activity.
    Owed,
}

/// The clause an activation is executing, as `StackFrame~line`,
/// `~traceLine` and `RexxContext~line` report it.
#[derive(Copy, Clone)]
pub(crate) struct ClauseSnapshot {
    /// `RexxActivation::getContextLineNumber`
    /// (`execution/RexxActivation.cpp:2941`), whose answer with no current
    /// instruction is `1`.
    pub(crate) line: usize,
    /// The `*-*` echo's own indent: `PackageClass::traceBack`'s
    /// `indent * INDENT_SPACING`, already doubled here as
    /// `Interp::printed_indent` produces it.
    pub(crate) indent: usize,
    pub(crate) index: usize,
}

impl Default for ClauseSnapshot {
    fn default() -> ClauseSnapshot {
        ClauseSnapshot {
            line: 1,
            indent: 0,
            index: 0,
        }
    }
}

/// One activation: everything about the frame currently executing.
pub(crate) struct Activation {
    /// This activation's own identity, unique for the life of the `Interp`.
    /// See [`ActivationId`] for the two measured defects that made a depth
    /// insufficient.
    pub(crate) id: ActivationId,
    /// The program this frame is running.
    pub(crate) program: Rc<Program>,
    /// `program`'s own id, the durable identity `Interp::programs` hands out.
    pub(crate) program_id: ProgramId,
    /// Which of `program`'s code bodies this activation is running: `None` is
    /// `program.main`, `Some(i)` is `program.directives[i]`'s own body.
    pub(crate) body: Option<usize>,
    pub(crate) plan: Rc<Plan>,
    /// Names bound after `plan` was built, and the reason this field exists is
    /// the whole answer to "does a fragment's plan work against the enclosing
    /// plan's name map".
    /// ```text
    /// interpret "zork = 42"      /* ZORK is in no instruction of this body */
    /// interpret "say zork"       /* prints 42 */
    /// ```
    pub(crate) extra: NameMap<Box<[u8]>, usize>,
    pub(crate) frame: SlotFrame,
    /// Whether this activation is the one that must `pop_slots` [`frame`],
    /// and equivalently whether the pool in it is its own.
    pub(crate) owns_frame: bool,
    /// How this activation was entered -- what an instruction whose legality
    /// depends on the entry reads. [`Entry`] carries the oracle's own table.
    pub(crate) entry: Entry,
    /// What `PARSE SOURCE`'s second word answers while this activation runs.
    pub(crate) call_type: CallType,
    /// `Some` exactly for an [`Entry::Method`] activation: what its
    /// `>I>`/`<I<` lines name.
    pub(crate) method_identity: Option<MethodIdentity>,
    /// Every name an `EXPOSE` in this activation bound to a scope pool on the
    /// receiving object, by the slot index that name resolves to here.
    pub(crate) exposed: Vec<(usize, InstanceVar)>,
    /// How far this activation is through a `REPLY`, which is
    /// `ActivationSettings::isReplyIssued` plus the `REPLIED` execution state
    /// as one value.
    pub(crate) reply: ReplyState,
    /// Whether the `REPLY` that ran here carried a value.
    pub(crate) replied_a_value: bool,
    /// What a `REPLY` leaves here while [`ReplyState::Owed`].
    pub(crate) replied: Option<Box<Replied>>,
    /// Whether this activation is performing the send of a `FORWARD` that
    /// does not `CONTINUE`, which makes it a phantom for condition delivery;
    /// whether the method is guarded; and whether this activation holds its
    /// receiver's guard lock for its scope.
    pub(crate) flags: ActivationFlags,
    /// Whether no instruction has yet been executed in this activation --
    /// where a label does not count as an instruction.
    /// ```text
    /// call sub / sub: procedure                 -> runs
    /// call sub / sub: / lbl2: / procedure       -> runs      (labels do not count)
    /// call sub / sub: nop / procedure           -> 17.1      (a NOP does)
    /// say 'main' / sub: / procedure             -> 17.1      (fell through, no call)
    /// ```
    pub(crate) first_instruction_pending: bool,
    /// How far this activation is past the point where a `>I>` could still be
    /// announced -- `RexxActivation::traceEntryAllowed` and `traceEntryDone`
    /// as one value, because the two are read together everywhere and a
    /// separate pair admits states the C++ never reaches.
    pub(crate) trace_entry: TraceEntry,
    pub(crate) pc: usize,
    /// This activation's own `NUMERIC DIGITS`/`FUZZ`/`FORM`.
    pub(crate) settings: Settings,
    /// What `.RS` answers here: 0 after a command that succeeded, 1 after one
    /// that raised `ERROR` and -1 after one that raised `FAILURE`. `None`
    /// until a command runs, which `.RS` answers as its own name rather than
    /// as a number (`RexxActivation::rexxVariable`, `isReturnStatusSet`).
    pub(crate) rs: Option<i32>,
    /// Which conditions this activation raises as SYNTAX errors --
    /// `::OPTIONS <condition> SYNTAX` on its own package, minus whatever a
    /// `SIGNAL ON`/`OFF` here has turned off since.
    pub(crate) condition_syntax: ConditionSyntax,
    /// This activation's own `TRACE` setting (D17).
    pub(crate) trace_mode: TraceMode,
    /// The interactive-debug bookkeeping this activation owns, which is why
    /// debug is scoped to a call: measured, a `trace ?r` inside an internal
    /// routine leaves the caller at `N`.
    pub(crate) debug: DebugState,
    /// This activation's own `ADDRESS` environment pair.
    /// ```text
    /// main   address()  ->  OUTER
    ///  sub   address()  ->  OUTER      inherited
    ///  sub   address              ->   sh      the caller's alternate came too
    ///  sub   address inner ; address() -> INNER
    /// main   address()  ->  OUTER      unchanged by the callee
    /// main   address              ->   sh      the caller's own alternate
    /// ```
    pub(crate) address: AddressState,
    /// The `ADDRESS ... WITH` configurations in force here, keyed by the
    /// **upcased** environment name -- measured, `address sh with output stem
    /// o.` still redirects after a later `address SH`. `None` is the table
    /// the oracle has not created yet (`settings.ioConfigs == OREF_NULL`),
    /// which is every program that never writes one.
    ///
    /// Behind an `Rc` so an internal call inherits it by refcount and a write
    /// in the callee clones first: that is `checkIOConfigTable`'s own lazy
    /// copy (`RexxActivation.cpp:1651`), and it is what makes the
    /// inheritance one-way.
    pub(crate) io_configs: Option<Rc<IoConfigs>>,
    /// The condition traps enabled in this activation, keyed by the exact
    /// condition name a raise carries (`Raised::condition`) -- `SYNTAX`,
    /// `NOVALUE`, `USER FOO`, ...
    pub(crate) traps: TrapMap,
    /// What this activation holds only once a handler, `RANDOM` or
    /// `SETLOCAL` has run in it, allocated on the first.
    pub(crate) cold: Option<Box<ActivationCold>>,
    /// `DATE`/`TIME`'s clock reading, in `builtin::datetime`'s own
    /// microseconds-since-0001-01-01 unit -- the last value this activation
    /// ever read, or [`NO_CLOCK`] before its first one. **This is not itself
    /// the per-clause cache** -- see [`clock_stale`] for that half -- because
    /// `RexxActivation::getTime`'s own lazy `TIME('R')` reset
    /// (`execution/RexxActivation.cpp:3400`-`3406`) needs the *stale* value
    /// still readable one call after the clause that produced it stopped
    /// being current, to anchor the reset to.
    pub(crate) cached_clock: i64,
    /// `TIME('E')`/`TIME('R')`'s anchor, `settings.elapsedTime`: the reading
    /// elapsed time is measured from, in [`cached_clock`]'s unit, or
    /// [`NO_CLOCK`] before this activation's first `E` or `R`.
    pub(crate) elapsed_anchor: i64,
    /// The innermost `SELECT CASE`'s value here, which an absorbed `WHEN`
    /// compares against; `None` inside a plain `SELECT`.
    pub(crate) current_case_text: Option<Vec<u8>>,
    /// Whether [`cached_clock`] needs a fresh read before this activation's
    /// clause may trust it -- the per-clause half [`cached_clock`]'s own
    /// doc names, set `true` once per instruction by `Op::Clause`'s region
    /// on whichever activation is executing at the time, mirroring
    /// `RexxActivation::run`'s own `settings.timeStamp.valid = false` set
    /// right after `nextInst->execute()` returns (`RexxActivation.cpp:647`).
    pub(crate) clock_stale: bool,
    /// The `RexxContext` this activation's `.CONTEXT` answers, built on the
    /// first ask and kept for the rest of the activation --
    /// `RexxActivation::getContextObject`, which fills its own field the
    /// same way.
    pub(crate) context_object: Option<ObjRef>,
    /// The `Message` whose held send this activation made last and that has
    /// not completed: `RexxActivation`'s single `notifyObject`
    /// (`execution/RexxActivation.cpp:4630`), told of a `SYNTAX` failure.
    pub(crate) notify_message: Option<ObjRef>,
    /// The clause this activation is executing. [`ClauseSnapshot`] carries
    /// where it is written and why it is carried rather than derived.
    pub(crate) clause: ClauseSnapshot,
    /// This activation's `~invocation` id, minted on the first ask.
    pub(crate) invocation: Option<u32>,
    /// The name this activation was invoked under -- `settings.messageName`,
    /// which `StackFrame~name` and `RexxContext~name` both answer.
    pub(crate) call_name: Option<Rc<[u8]>>,
    /// The arguments this activation was entered with, the other half of
    /// [`call_name`]'s snapshot -- `StackFrame~arguments` and
    /// `RexxContext~args`.
    pub(crate) call_arguments: Option<Rc<[Option<ObjRef>]>>,
    /// The streams the builtins have opened by name in this activation,
    /// keyed by the **qualified** path, so `./a.txt` and `a.txt` are one
    /// entry. Only a program or method activation owns one; every other
    /// kind resolves through its nearest such ancestor.
    pub(crate) streams: NameMap<Box<[u8]>, ObjRef>,
}

/// How control arrived at an activation.
/// ```text
/// entered by                       PROCEDURE first   USE LOCAL first
/// the program itself               17.1, rc 239      98.993, rc 158
/// an internal label, by CALL       runs, rc 0        99.910 at parse time
/// an internal label, as a function runs, rc 0        99.910 at parse time
/// a ::ROUTINE, by CALL             17.1, rc 239      98.993, rc 158
/// a ::ROUTINE, as a function       17.1, rc 239      98.993, rc 158
/// a ::METHOD                       17.1, rc 239      runs, rc 0
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Entry {
    /// The program's own activation, which [`Interp::run`] starts.
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
/// ```text
/// the program itself                          COMMAND
/// an internal label, by CALL                  the enclosing activation's
/// an internal label, as a function            the enclosing activation's
/// a ::ROUTINE, by CALL                        SUBROUTINE
/// a ::ROUTINE, as a function                  FUNCTION
/// a ::METHOD, by a message send               METHOD
/// a ::ATTRIBUTE GET or SET, by a message send METHOD
/// ```
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
    /// The prologue of a file a `::REQUIRES` loaded, which
    /// `PackageClass::runProlog` calls under `GlobalNames::REQUIRES`
    /// (`classes/PackageClass.cpp:2136`). Measured, oracle rc 0: `parse
    /// source` in such a prologue answers `LINUX REQUIRES <its own path>`.
    Requires,
}

impl CallType {
    /// The word itself, as `PARSE SOURCE` spells it.
    pub(crate) fn token(self) -> &'static [u8] {
        match self {
            CallType::Command => b"COMMAND",
            CallType::Subroutine => b"SUBROUTINE",
            CallType::Function => b"FUNCTION",
            CallType::Method => b"METHOD",
            CallType::Requires => b"REQUIRES",
        }
    }
}

/// What a `::METHOD` activation knows about the send that entered it.
pub(crate) struct MethodIdentity {
    /// The message name, already upcased by the parser -- measured,
    /// `::method MiXeD` announces `"MIXED"` and `::method "quoted"`
    /// announces `"QUOTED"`.
    pub(crate) name: Rc<[u8]>,
    /// The defining class, whose `~id` is printed unmodified -- measured,
    /// `::class 'k'` announces `with scope "k"`.
    pub(crate) scope: ObjRef,
    /// The object the send was addressed to -- `SELF`.
    pub(crate) receiver: ObjRef,
}

/// An activation's flags in one byte: forwarded, `ActivationSettings::
/// isGuarded`, `objectScope == SCOPE_RESERVED`, the elapsed clock's pending
/// reset (`elapsedReset`), and `debugPause`.
#[derive(Clone, Copy, Default)]
pub(crate) struct ActivationFlags(u8);

/// Every call pushes one, so the size is pinned: a field that grows it has to
/// pay for itself on the call path (ruling R4).
const _: () = assert!(size_of::<Activation>() == 480);

impl ActivationFlags {
    const FORWARDED: u8 = 1;
    const GUARDED: u8 = 2;
    const RESERVED: u8 = 4;
    const ELAPSED_RESET: u8 = 8;
    const DEBUG_PAUSE: u8 = 16;

    /// A method activation's flags: `guarded`, and holding its lock where
    /// `reserved`.
    #[inline]
    pub(crate) fn method(guarded: bool, reserved: bool) -> ActivationFlags {
        ActivationFlags(u8::from(guarded) << 1 | u8::from(reserved) << 2)
    }

    pub(crate) fn forwarded(self) -> bool {
        self.0 & Self::FORWARDED != 0
    }

    pub(crate) fn set_forwarded(&mut self, forwarded: bool) {
        self.set(Self::FORWARDED, forwarded);
    }

    pub(crate) fn guarded(self) -> bool {
        self.0 & Self::GUARDED != 0
    }

    pub(crate) fn reserved(self) -> bool {
        self.0 & Self::RESERVED != 0
    }

    pub(crate) fn set_reserved(&mut self, reserved: bool) {
        self.set(Self::RESERVED, reserved);
    }

    /// Whether a `TIME('R')`, or a clock read going backward, waits to move
    /// the elapsed anchor at the next fresh clock read.
    pub(crate) fn elapsed_reset(self) -> bool {
        self.0 & Self::ELAPSED_RESET != 0
    }

    pub(crate) fn set_elapsed_reset(&mut self, reset: bool) {
        self.set(Self::ELAPSED_RESET, reset);
    }

    /// Whether this activation is running a line typed at a debug pause:
    /// `RexxActivation::debugPause`, which `inDebug` and `noTracing` read.
    pub(crate) fn debug_pause(self) -> bool {
        self.0 & Self::DEBUG_PAUSE != 0
    }

    pub(crate) fn set_debug_pause(&mut self, paused: bool) {
        self.set(Self::DEBUG_PAUSE, paused);
    }

    fn set(&mut self, bit: u8, on: bool) {
        if on {
            self.0 |= bit;
        } else {
            self.0 &= !bit;
        }
    }
}

/// [`Activation::cached_clock`] and [`Activation::elapsed_anchor`] before
/// their first reading: the oracle's own `elapsedTime == 0`.
pub(crate) const NO_CLOCK: i64 = 0;

/// The part of an activation most never use, boxed off the call path.
#[derive(Default)]
pub(crate) struct ActivationCold {
    /// The condition `CONDITION()` reports in this activation, or `None`
    /// when no handler has been entered here. [`TrappedCondition`] carries
    /// the measurements for the copy-on-call, never-write-back rule it
    /// follows along with [`Activation::traps`].
    pub(crate) condition: Option<TrappedCondition>,
    /// The condition `RAISE PROPAGATE` re-raises here, `settings.conditionObj`.
    pub(crate) active_condition: Option<Rc<crate::ActiveCondition>>,
    /// `RANDOM`'s state for a top-level activation, `None` before its first
    /// draw.
    pub(crate) random_seed: Option<u64>,
    /// What each outstanding `SETLOCAL` saved, innermost last: the directory
    /// and the whole environment.
    pub(crate) locals: Vec<(std::path::PathBuf, Vec<(Vec<u8>, Vec<u8>)>)>,
}

impl ActivationCold {
    /// What an internal call or `CALL ON` handler starts from: the caller's
    /// two conditions, and neither its seed nor its `SETLOCAL` list, which
    /// stay on the top-level activation.
    pub(crate) fn inherited(&self) -> Option<Box<ActivationCold>> {
        if self.condition.is_none() && self.active_condition.is_none() {
            return None;
        }
        Some(Box::new(ActivationCold {
            condition: self.condition.clone(),
            active_condition: self.active_condition.clone(),
            ..ActivationCold::default()
        }))
    }
}

/// What a `REPLY` leaves its activation until the activation ends.
pub(crate) struct Replied {
    /// What the caller resumes with.
    pub(crate) value: Option<ObjRef>,
    /// The activity the rest of the body moved to, once the split ran.
    pub(crate) continuation: Option<Box<crate::scheduler::Idle>>,
}

impl Replied {
    pub(crate) fn object_roots(&self, out: &mut Vec<ObjRef>) {
        out.extend(self.value);
        if let Some(continuation) = &self.continuation {
            continuation.object_roots(out);
        }
    }
}

/// One variable an `EXPOSE` bound: which object's pools hold it, which of that
/// object's pools, and under what name.
#[derive(Clone, Debug)]
pub(crate) struct InstanceVar {
    /// The object whose [`rexx_core::ScopePools`] hold this variable.
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
            rs: None,
            extra: NameMap::default(),
            frame,
            owns_frame: true,
            entry: Entry::TopLevel,
            // The word `rexx p.rex` puts here: the front end starts a program
            // as `RXCOMMAND`, whose string is `COMMAND`.
            call_type: CallType::Command,
            method_identity: None,
            exposed: Vec::new(),
            reply: ReplyState::None,
            replied_a_value: false,
            replied: None,
            flags: ActivationFlags::default(),
            first_instruction_pending: true,
            trace_entry: TraceEntry::Pending,
            pc: 0,
            settings: Settings::default(),
            condition_syntax: ConditionSyntax::default(),
            // `NORMAL` and not `OFF`: the two behave identically here and a
            // program can tell them apart, measured -- `say trace()` as the
            // first clause of a program with no `TRACE` instruction prints
            // `N`.
            trace_mode: TraceMode::NORMAL,
            debug: DebugState::default(),
            address: AddressState::default(),
            io_configs: None,
            traps: TrapMap::default(),
            cold: None,
            cached_clock: NO_CLOCK,
            elapsed_anchor: NO_CLOCK,
            current_case_text: None,
            clock_stale: true,
            context_object: None,
            notify_message: None,
            clause: ClauseSnapshot::default(),
            invocation: None,
            call_name: None,
            call_arguments: None,
            streams: NameMap::default(),
        }
    }

    /// The activation a `CALL` pushes: it starts at `pc`, and it **inherits
    /// every field [`Inherited`] carries** from the caller rather than
    /// defaulting them.
    /// ```text
    /// numeric digits 7 / fuzz 2 / form engineering    routine: 9 0 SCIENTIFIC
    /// address system                                  routine: address() = sh
    /// signal on syntax name mytrap, routine has its   the routine's own mytrap
    ///   own mytrap: label and raises 1/0              never runs; the caller's does
    /// inside a SIGNAL ON SYNTAX handler, condition()  routine: the null string
    /// trace r in the caller                           routine: trace() = N,
    ///                                                 none of its clauses echoed
    /// ```
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
            extra: NameMap::default(),
            frame,
            owns_frame: false,
            entry: Entry::InternalCall,
            rs: inherited.rs,
            call_type: inherited.call_type,
            method_identity: None,
            exposed: Vec::new(),
            reply: ReplyState::None,
            replied_a_value: false,
            replied: None,
            flags: {
                let mut flags = ActivationFlags::default();
                flags.set_elapsed_reset(inherited.elapsed_reset);
                flags
            },
            first_instruction_pending: true,
            trace_entry: TraceEntry::Pending,
            pc,
            settings: inherited.settings,
            condition_syntax: inherited.condition_syntax,
            trace_mode: inherited.trace_mode,
            debug: DebugState::default(),
            address: inherited.address,
            io_configs: inherited.io_configs,
            traps: inherited.traps,
            cold: inherited.cold,
            // The caller's stamp, invalid: `RexxActivation`'s internal-call
            // constructor copies the settings and sets `timeStamp.valid =
            // false` (`RexxActivation.cpp:241`), so a reset the caller left
            // pending anchors to the caller's last reading.
            cached_clock: inherited.cached_clock,
            elapsed_anchor: inherited.elapsed_anchor,
            current_case_text: None,
            clock_stale: true,
            context_object: None,
            notify_message: None,
            clause: ClauseSnapshot::default(),
            invocation: None,
            call_name: None,
            call_arguments: None,
            streams: NameMap::default(),
        }
    }

    /// The activation a call into a `::ROUTINE` directive pushes: it starts
    /// at instruction 0 of `directives[body]`'s own body, in a pool of its
    /// own, and it inherits **nothing**.
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
            extra: NameMap::default(),
            frame,
            owns_frame: true,
            entry: Entry::Routine,
            // Not inherited: measured, a `::ROUTINE` called after a failing
            // command answers `.RS` as its own name, where an internal label
            // called from the same place answers 1.
            rs: None,
            call_type,
            method_identity: None,
            exposed: Vec::new(),
            reply: ReplyState::None,
            replied_a_value: false,
            replied: None,
            flags: ActivationFlags::default(),
            first_instruction_pending: true,
            trace_entry: TraceEntry::Pending,
            pc: 0,
            settings: Settings::default(),
            condition_syntax: ConditionSyntax::default(),
            trace_mode: TraceMode::NORMAL,
            debug: DebugState::default(),
            address: AddressState::default(),
            io_configs: None,
            traps: TrapMap::default(),
            cold: None,
            cached_clock: NO_CLOCK,
            elapsed_anchor: NO_CLOCK,
            current_case_text: None,
            clock_stale: true,
            context_object: None,
            notify_message: None,
            clause: ClauseSnapshot::default(),
            invocation: None,
            call_name: None,
            call_arguments: None,
            streams: NameMap::default(),
        }
    }

    /// The activation a message send into a `::METHOD` body pushes.
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
            extra: NameMap::default(),
            frame,
            owns_frame: true,
            entry: Entry::Method,
            rs: None,
            call_type: CallType::Method,
            method_identity: Some(identity),
            exposed: Vec::new(),
            reply: ReplyState::None,
            replied_a_value: false,
            replied: None,
            flags: ActivationFlags::default(),
            first_instruction_pending: true,
            trace_entry: TraceEntry::Pending,
            pc: 0,
            settings: Settings::default(),
            condition_syntax: ConditionSyntax::default(),
            trace_mode: TraceMode::NORMAL,
            debug: DebugState::default(),
            address: AddressState::default(),
            io_configs: None,
            traps: TrapMap::default(),
            cold: None,
            cached_clock: NO_CLOCK,
            elapsed_anchor: NO_CLOCK,
            current_case_text: None,
            clock_stale: true,
            context_object: None,
            notify_message: None,
            clause: ClauseSnapshot::default(),
            invocation: None,
            call_name: None,
            call_arguments: None,
            streams: NameMap::default(),
        }
    }

    /// The key this activation's body is cached under, in both the plan
    /// cache and the chunk cache.
    pub(crate) fn body_key(&self) -> BodyKey {
        BodyKey {
            program: self.program_id,
            directive: self.body,
        }
    }

    /// The name this activation was invoked under, empty for one that has
    /// not started running -- see [`Activation::call_name`].
    pub(crate) fn invoked_as(&self) -> &[u8] {
        if let Some(identity) = &self.method_identity {
            return &identity.name;
        }
        self.call_name.as_deref().unwrap_or_default()
    }

    /// The arguments this activation was entered with, the other half of
    /// [`Activation::invoked_as`].
    pub(crate) fn invoked_with(&self) -> &[Option<ObjRef>] {
        self.call_arguments.as_deref().unwrap_or_default()
    }

    /// The condition `CONDITION()` reports here.
    pub(crate) fn condition(&self) -> Option<&TrappedCondition> {
        self.cold.as_deref()?.condition.as_ref()
    }

    /// Replaces [`Activation::condition`], answering the one it held.
    pub(crate) fn replace_condition(
        &mut self,
        condition: Option<TrappedCondition>,
    ) -> Option<TrappedCondition> {
        if condition.is_none() && self.cold.is_none() {
            return None;
        }
        std::mem::replace(&mut self.cold_mut().condition, condition)
    }

    /// The cold part, allocated on the first ask.
    pub(crate) fn cold_mut(&mut self) -> &mut ActivationCold {
        self.cold.get_or_insert_with(Box::default)
    }

    /// Appends every `ObjRef` this activation holds to `out`.
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
            rs: _,
            method_identity,
            exposed,
            reply: _,
            replied_a_value: _,
            replied,
            flags: _,
            first_instruction_pending: _,
            trace_entry: _,
            pc: _,
            settings: _,
            condition_syntax: _,
            trace_mode: _,
            address: _,
            // Parse data behind an `Rc`: an `AddressIo` holds expressions and
            // symbol ids, never an `ObjRef`. What a redirection *evaluates*
            // to is a temp of the command that built it.
            io_configs: _,
            // Plain bookkeeping: counters and flags, no `ObjRef` among them.
            debug: _,
            traps: _,
            cold,
            cached_clock: _,
            elapsed_anchor: _,
            current_case_text: _,
            clock_stale: _,
            context_object,
            notify_message,
            clause: _,
            invocation: _,
            call_name: _,
            call_arguments,
            streams,
        } = self;
        out.extend(*context_object);
        out.extend(*notify_message);
        if let Some(replied) = replied {
            replied.object_roots(out);
        }
        // The trapped condition's own object, destructured rather than
        // reached through a field: the exhaustive match above guards this
        // struct's fields, and an `ObjRef` added to `TrappedCondition` would
        // otherwise arrive unrooted with nothing to say so.
        if let Some(TrappedCondition {
            name: _,
            code_sub: _,
            call: _,
            description: _,
            object,
        }) = cold.as_deref().and_then(|cold| cold.condition.as_ref())
        {
            out.extend(*object);
        }
        // The table's streams. An activation is a root, and a stream only
        // the table holds is reachable through nothing else.
        out.extend(streams.values().copied());
        // The convention's own values, which a moved `REPLY` continuation
        // also roots from its `CallContext`: this activation holds a
        // refcount clone of the same slice.
        out.extend(
            call_arguments
                .iter()
                .flat_map(|args| args.iter().flatten().copied()),
        );
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
pub(crate) struct Inherited {
    pub(crate) call_type: CallType,
    pub(crate) settings: Settings,
    /// `.RS`, which an internal call starts from its caller's copy of.
    pub(crate) rs: Option<i32>,
    pub(crate) condition_syntax: ConditionSyntax,
    pub(crate) trace_mode: TraceMode,
    pub(crate) address: AddressState,
    pub(crate) io_configs: Option<Rc<IoConfigs>>,
    pub(crate) traps: TrapMap,
    /// The caller's `CONDITION()` and `RAISE PROPAGATE` conditions
    /// ([`ActivationCold::inherited`]).
    pub(crate) cold: Option<Box<ActivationCold>>,
    pub(crate) cached_clock: i64,
    pub(crate) elapsed_anchor: i64,
    pub(crate) elapsed_reset: bool,
}

/// The code body a `(program, selector)` pair denotes: `None` is
/// `program.main`, `Some(i)` is `program.directives[i]`'s own body.
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

    /// The activation stack innermost first -- the order
    /// `Activity::generateStackFrames` (`concurrency/Activity.cpp:1141`)
    /// walks it and the order `RexxContext~stackFrames` answers in.
    pub(crate) fn frames(&self) -> impl DoubleEndedIterator<Item = &Activation> {
        self.activity
            .running
            .iter()
            .map(std::ops::Deref::deref)
            .chain(self.activity.suspended.iter().rev().map(Box::as_ref))
    }

    /// The stream table of the activation that owns one: the running frame
    /// when it is a program or method, else the nearest such ancestor.
    /// Internal calls, `PROCEDURE` routines, `::ROUTINE`s and `INTERPRET`
    /// all borrow their caller's, which is measured -- a routine advances
    /// the caller's read position where a method call does not.
    pub(crate) fn stream_table(&self) -> Option<&NameMap<Box<[u8]>, ObjRef>> {
        self.frames()
            .find(|frame| matches!(frame.entry, Entry::TopLevel | Entry::Method))
            .map(|frame| &frame.streams)
    }

    /// [`Interp::stream_table`] for a caller that inserts or removes. **By the
    /// same predicate rather than by depth**: `frame_at_mut` counts backwards
    /// into a vector that is oldest first, and an off-by-one there would put
    /// one activation's streams on its caller -- a wrong table with nothing to
    /// notice it, since both are plausible.
    pub(crate) fn stream_table_mut(&mut self) -> Option<&mut NameMap<Box<[u8]>, ObjRef>> {
        if self
            .activity
            .running
            .as_deref()
            .is_some_and(|frame| matches!(frame.entry, Entry::TopLevel | Entry::Method))
        {
            return self
                .activity
                .running
                .as_deref_mut()
                .map(|frame| &mut frame.streams);
        }
        self.activity
            .suspended
            .iter_mut()
            .rev()
            .map(Box::as_mut)
            .find(|frame| matches!(frame.entry, Entry::TopLevel | Entry::Method))
            .map(|frame| &mut frame.streams)
    }

    /// The innermost activation that is not an internal call: the one
    /// `RANDOM`'s seed and `SETLOCAL`'s list belong to (the oracle's
    /// `isInternalLevelCall` delegation to `parent`).
    pub(crate) fn top_level_activation_mut(&mut self) -> Option<&mut Activation> {
        if self
            .activity
            .running
            .as_deref()
            .is_some_and(|frame| frame.entry != Entry::InternalCall)
        {
            return self.activity.running.as_deref_mut();
        }
        self.activity
            .suspended
            .iter_mut()
            .rev()
            .map(Box::as_mut)
            .find(|frame| frame.entry != Entry::InternalCall)
    }

    /// The activation at `depth`, counted as [`Activity::frames`] counts: `0`
    /// is the running one.
    pub(crate) fn frame_at(&self, depth: usize) -> Option<&Activation> {
        self.frames().nth(depth)
    }

    /// The depth of the innermost activation that is not an internal call.
    pub(crate) fn first_non_internal_depth(&self) -> usize {
        self.frames()
            .position(|activation| activation.entry != Entry::InternalCall)
            .unwrap_or(0)
    }

    /// How many of the running `INTERPRET` fragments run in the activations
    /// from the running one out to the one at `depth`.
    pub(crate) fn fragments_within(&self, depth: usize) -> usize {
        let levels: Vec<ActivationId> = self.frames().take(depth + 1).map(|a| a.id).collect();
        self.activity
            .fragments
            .iter()
            .rev()
            .take_while(|level| levels.contains(&level.owner))
            .count()
    }

    /// The clause the activation at `depth` is stopped on.
    pub(crate) fn clause_of(&self, depth: usize) -> ClauseSnapshot {
        if depth == 0 && self.activity.running.is_some() {
            return ClauseSnapshot {
                line: self.activity.clause_state.line(),
                indent: self.activity.clause_state.current_value_indent,
                index: self.activity.clause_state.clause_index(),
            };
        }
        self.frame_at(depth)
            .map(|activation| activation.clause)
            .unwrap_or_default()
    }

    /// [`Interp::frame_at`] for a caller that has to write.
    pub(crate) fn frame_at_mut(&mut self, depth: usize) -> Option<&mut Activation> {
        // **The two indexings are separate code and this is what holds them
        // equal.** `frames` counts forwards from the running activation and
        // this counts backwards into a vector that is oldest first, so an
        // off-by-one here would write one activation's `~invocation` id onto
        // its caller -- a wrong answer with nothing to notice it, since both
        // are plausible numbers.
        let expected = self.frame_at(depth).map(|activation| activation.id);
        let found = match depth.checked_sub(usize::from(self.activity.running.is_some())) {
            None => self.activity.running.as_deref_mut(),
            Some(below) => match self.activity.suspended.len().checked_sub(below + 1) {
                None => None,
                Some(index) => self.activity.suspended.get_mut(index).map(Box::as_mut),
            },
        };
        debug_assert_eq!(
            found.as_ref().map(|activation| activation.id),
            expected,
            "frame_at and frame_at_mut disagree about the activation at depth {depth}"
        );
        found
    }

    pub(crate) fn activation(&self) -> &Activation {
        self.activity.running.as_deref().expect("a live activation")
    }

    pub(crate) fn activation_mut(&mut self) -> &mut Activation {
        self.activity
            .running
            .as_deref_mut()
            .expect("a live activation")
    }

    /// The running activation, or `None` where nothing is running.
    pub(crate) fn running_activation(&self) -> Option<&Activation> {
        self.activity.running.as_deref()
    }

    /// The running activation's own caller, for a writer, or `None` at the
    /// outermost level.
    pub(crate) fn caller_activation_mut(&mut self) -> Option<&mut Activation> {
        self.activity.suspended.last_mut().map(Box::as_mut)
    }

    /// The activation whose trap table answers for a condition raised right
    /// now: the running one, or, while that one is a phantom performing a
    /// non-continuing `FORWARD`'s send, the innermost frame beneath it that
    /// is not.
    pub(crate) fn trap_frame(&self) -> Option<&Activation> {
        let running = self.running_activation()?;
        if !running.flags.forwarded() {
            return Some(running);
        }
        self.activity
            .suspended
            .iter()
            .rev()
            .map(Box::as_ref)
            .find(|activation| !activation.flags.forwarded())
    }

    /// How many activations are live, the running one included.
    pub(crate) fn activation_depth(&self) -> usize {
        self.activity.suspended.len() + usize::from(self.activity.running.is_some())
    }

    /// Makes `activation` the running one and suspends whatever was.
    pub(crate) fn push_activation(&mut self, activation: Activation) {
        let boxed = match self.activity.spare_activations.pop() {
            Some(mut spare) => {
                *spare = activation;
                spare
            }
            None => Box::new(activation),
        };
        // `trace_cache`'s own invariant: the setting travels with whichever
        // activation is running, and this changes which one that is.
        self.activity.trace_cache =
            crate::trace::TraceCache::of(boxed.trace_mode, boxed.flags.debug_pause());
        // The clause the activation being suspended is stopped on, which is
        // what its own `StackFrame` reports for as long as it stays
        // suspended: `Activity::clause_state` is about to start describing the
        // callee's clauses instead. Taken here rather than at each of the
        // sites that push, for the reason `Interp::run_activation` takes the
        // calling convention there: this is the one place every push passes.
        let clause = ClauseSnapshot {
            line: self.activity.clause_state.line(),
            indent: self.activity.clause_state.current_value_indent,
            index: self.activity.clause_state.clause_index(),
        };
        if let Some(mut outer) = self.activity.running.replace(boxed) {
            outer.clause = clause;
            self.activity.suspended.push(outer);
        }
    }

    /// Keeps an ended activation's box for the next [`Interp::
    /// push_activation`], instead of returning it to the allocator.
    pub(crate) fn recycle_activation(&mut self, ended: Box<Activation>) {
        if self.activity.spare_activations.len() < SPARE_ACTIVATIONS {
            self.activity.spare_activations.push(ended);
        }
    }

    /// Ends the running activation and resumes its caller, answering the
    /// activation that ended.
    pub(crate) fn pop_activation(&mut self) -> Option<Box<Activation>> {
        let mut ended = self.activity.running.take()?;
        // An activation ending with `SETLOCAL`s outstanding restores the
        // oldest (`RexxActivation.cpp:1494-1500`); one a `REPLY` moves on
        // ends later, on its continuation's activity.
        if let Some(cold) = ended.cold.as_deref_mut()
            && !cold.locals.is_empty()
            && ended
                .replied
                .as_ref()
                .is_none_or(|replied| replied.continuation.is_none())
        {
            let oldest = cold.locals.swap_remove(0);
            cold.locals.clear();
            self.restore_local_environment(oldest);
        }
        self.activity.running = self.activity.suspended.pop();
        // The resumed caller's setting, or `OFF` where nothing is left to
        // resume -- which is the state `Interp::new` starts in.
        self.activity.trace_cache = match self.activity.running.as_deref() {
            Some(resumed) => {
                crate::trace::TraceCache::of(resumed.trace_mode, resumed.flags.debug_pause())
            }
            None => crate::trace::TraceCache::of(TraceMode::OFF, false),
        };
        Some(ended)
    }

    /// The `TRACE` setting in force right now: the *running* activation's.
    pub(crate) fn trace_mode(&self) -> TraceMode {
        // **The cache is checked here rather than trusted**, which is what
        // closes the set of places that maintain it: any path that changes
        // the running activation or its setting without going through one of
        // them reddens this at the next clause, in a debug run of anything at
        // all.
        debug_assert_eq!(
            self.running_activation()
                .map(|activation| activation.trace_mode),
            Some(self.activity.trace_cache.mode()),
            "the cached TRACE setting is not the running activation's"
        );
        self.activity.trace_cache.mode()
    }

    /// Sets the running activation's `TRACE`. Only the `TRACE` instruction
    /// and this crate's own tests call it; a callee inherits its starting
    /// value through [`Activation::nested`] instead, never through here.
    pub(crate) fn set_trace_mode(&mut self, mode: TraceMode) {
        self.activation_mut().trace_mode = mode;
        self.activity.trace_cache = crate::trace::TraceCache::of(mode, self.debug_pause());
    }
}

/// One of the condition names the language fixes, used as an index into a
/// [`BuiltinTraps`] slot.
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
#[derive(Clone, Default)]
pub(crate) struct TrapMap {
    builtin: Option<Rc<BuiltinTraps>>,
    /// **Boxed, and `None` rather than empty.** A `SIGNAL ON`/`CALL ON` for a
    /// name that is not one of the builtin conditions is rare, and every
    /// activation carried this map whether or not it had one: inline the
    /// `HashMap` is 48 of `TrapMap`'s 56 bytes, and `Interp::push_activation`
    /// copies the whole `Activation` on every call. `TrapMap` is 16 bytes now
    /// and `Activation` 416 rather than 456.
    #[allow(clippy::box_collection)]
    user: Option<Box<FxHashMap<Box<[u8]>, Trap>>>,
}

impl TrapMap {
    pub(crate) fn get(&self, name: &[u8]) -> Option<&Trap> {
        match Builtin::of(name) {
            Some(which) => self.builtin.as_ref()?.0[which as usize].as_ref(),
            None => self.user.as_ref()?.get(name),
        }
    }

    pub(crate) fn get_mut(&mut self, name: &[u8]) -> Option<&mut Trap> {
        match Builtin::of(name) {
            Some(which) => Rc::make_mut(self.builtin.as_mut()?).0[which as usize].as_mut(),
            None => self.user.as_mut()?.get_mut(name),
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
                self.user
                    .get_or_insert_with(Box::default)
                    .insert(name, trap);
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
                if let Some(user) = self.user.as_mut() {
                    user.remove(name);
                }
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
