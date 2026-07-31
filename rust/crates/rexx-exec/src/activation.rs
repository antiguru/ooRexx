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

use crate::Interp;
use crate::plan::Plan;
use rexx_core::SlotFrame;
use rexx_num::Settings;
use rexx_parse::Program;
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
    /// **It records the program and not the body, and `run_activation` fills
    /// that gap by hardcoding `&program.main`.** True for every activation 4a
    /// can build, since 4a has one and it runs the main body. False the moment
    /// 4b calls a `::routine`: that activation's body is
    /// `directives[i]`'s, and without a field saying so it would re-run
    /// `main` instead, silently and with the right program. The missing field
    /// is a body selector beside this one, the same thing `BodyKey::directive`
    /// already carries for the plan cache, and adding it is 4b's rather than
    /// speculative scaffolding here.
    pub(crate) program: Rc<Program>,
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
    pub(crate) settings: Settings,
}

impl Activation {
    /// A fresh top-level activation: no run-time bindings yet, default
    /// `NUMERIC` settings. What `Interp::run` starts every program with.
    ///
    /// 4b's `CALL` will need its own constructor that inherits `settings`
    /// from the caller instead of defaulting it, rather than reusing this
    /// one -- a fresh top-level run and a nested call begin from different
    /// starting settings, and folding both into one function would need a
    /// parameter that is `None` on every path this task can reach.
    pub(crate) fn new(program: Rc<Program>, plan: Rc<Plan>, frame: SlotFrame) -> Activation {
        Activation {
            program,
            plan,
            extra: HashMap::new(),
            frame,
            pc: 0,
            settings: Settings::default(),
        }
    }
}

impl Interp {
    pub(crate) fn activation(&self) -> &Activation {
        self.activations.last().expect("a live activation")
    }

    pub(crate) fn activation_mut(&mut self) -> &mut Activation {
        self.activations.last_mut().expect("a live activation")
    }
}
