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
use crate::plan::Plan;
use crate::trace::TraceMode;
use rexx_core::SlotFrame;
use rexx_num::Settings;
use rexx_parse::{CodeBody, DirectiveKind, Program};
use std::collections::HashMap;
use std::rc::Rc;

/// One activation: everything about the frame currently executing.
pub(crate) struct Activation {
    /// The program this frame is running.
    ///
    /// **A liveness anchor, and never borrowed through.** Nothing takes
    /// `&self.activations.last().program.…` and then calls a `&mut self`
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
    /// **Always `None` today, and that is a resolution-order fact rather than
    /// an unfinished field.** A `::routine`'s body is present in the AST
    /// (`DirectiveKind::Routine`'s own `body: Option<CodeBody>`, `Some` for
    /// every non-external routine), so `Some(i)` is representable and
    /// [`body_of`] resolves it -- but nothing in 4b's `CALL` can *construct*
    /// one, because a named call resolves internal label, then builtin, then
    /// external, and a same-file `::routine` sits behind the builtin step.
    /// Measured on the oracle: `::routine max` alongside `call max 1,2` still
    /// calls the builtin and reports `2`, so a `::routine` cannot be reached
    /// without first answering "is this name a builtin", which is 4c's.
    ///
    /// Task 3's report records the three ways a `::routine` activation is
    /// measurably *not* an internal label's -- its own variable pool, `TRACE`
    /// not crossing into it, and builtins shadowing it -- because whoever
    /// sets `Some(i)` owes all three, and none of them falls out of this
    /// field.
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
    pub(crate) fn new(program: Rc<Program>, plan: Rc<Plan>, frame: SlotFrame) -> Activation {
        Activation {
            program,
            body: None,
            plan,
            extra: HashMap::new(),
            frame,
            pc: 0,
            settings: Settings::default(),
            trace_mode: TraceMode::OFF,
        }
    }

    /// The activation a `CALL` pushes: it starts at `pc`, and it **inherits**
    /// `settings` and `trace_mode` from the caller rather than defaulting
    /// them.
    ///
    /// Both inheritances are measured, and both are one-way -- the callee
    /// starts from the caller's value and never writes back. `numeric digits
    /// 7` in a caller, `numeric digits 3` in the callee: the callee sees 7 on
    /// entry, reports 3 after its own instruction, and the caller still
    /// reports 7 after the `return`. `trace r` in a caller and `trace off` in
    /// the callee: the callee's clauses stop echoing and the caller's resume.
    /// Copying the values in here is what makes both true, since the callee's
    /// own fields are then simply dropped with its frame.
    ///
    /// `frame` is the caller's own `SlotFrame` for a callee with no
    /// `PROCEDURE` (D9r's shared pool, the default this task implements), so
    /// this constructor does not decide the pool -- its caller does, and
    /// Task 5's `PROCEDURE` is what will ever pass a different one.
    pub(crate) fn nested(
        program: Rc<Program>,
        body: Option<usize>,
        plan: Rc<Plan>,
        frame: SlotFrame,
        pc: usize,
        settings: Settings,
        trace_mode: TraceMode,
    ) -> Activation {
        Activation {
            program,
            body,
            plan,
            extra: HashMap::new(),
            frame,
            pc,
            settings,
            trace_mode,
        }
    }
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
/// `None` on a selector that names something other than a routine with a
/// body, rather than a panic: `Some(i)` can only be built from a resolution
/// step that already looked at `directives[i]`, so a mismatch is an internal
/// inconsistency, and this crate's rule for those is to fail loudly at the
/// caller rather than abort the process here.
pub(crate) fn body_of(program: &Program, selector: Option<usize>) -> Option<&CodeBody> {
    match selector {
        None => Some(&program.main),
        Some(index) => match &program.directives.get(index)?.kind {
            DirectiveKind::Routine(routine) => routine.body.as_ref(),
            _ => None,
        },
    }
}

impl Interp {
    pub(crate) fn activation(&self) -> &Activation {
        self.activations.last().expect("a live activation")
    }

    pub(crate) fn activation_mut(&mut self) -> &mut Activation {
        self.activations.last_mut().expect("a live activation")
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
        self.activation().trace_mode
    }

    /// Sets the running activation's `TRACE`. Only the `TRACE` instruction
    /// and this crate's own tests call it; a callee inherits its starting
    /// value through [`Activation::nested`] instead, never through here.
    pub(crate) fn set_trace_mode(&mut self, mode: TraceMode) {
        self.activation_mut().trace_mode = mode;
    }
}
