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
use crate::Activation;
use crate::plan::{BodyKey, ProgramId};
use rexx_parse::{Program, parse_program};

/// Pushes a fresh top-level activation for `program`, the same setup
/// `Interp::run` does, so a test can drive `step` through a live
/// activation without the full instruction loop. Copied rather than
/// shared, matching every other test module in this crate (`eval.rs`,
/// `plan.rs`, `stem.rs` each keep their own).
fn activate(interp: &mut Interp, program: Program) -> Rc<Program> {
    let program = Rc::new(program);
    let program_id = ProgramId(interp.programs.len());
    interp.programs.push(Rc::clone(&program));
    let plan = interp.plan_for(
        BodyKey {
            program: program_id,
            directive: None,
        },
        &program.main,
        &program.symbols,
        &program.source,
    );
    let frame = interp.roots.push_slots(plan.len());
    let id = interp.next_activation_id();
    interp.push_activation(Activation::new(
        id,
        Rc::clone(&program),
        program_id,
        plan,
        frame,
    ));
    program
}

/// Parses `source`, activates it, and runs its whole body -- through
/// `Interp::run_activation` itself, not through a miniature of it.
fn run_source(interp: &mut Interp, source: &[u8]) -> Result<Option<ObjRef>, Failure> {
    let program = parse_program(source.to_vec()).expect("test program parses");
    let program = activate(interp, program);
    run_activated(interp, &program)
}

/// `run_source`, with the program's directives installed first.
fn run_source_with_directives(
    interp: &mut Interp,
    source: &[u8],
) -> Result<Option<ObjRef>, Failure> {
    let program = parse_program(source.to_vec()).expect("test program parses");
    let program = Rc::new(program);
    let program_id = ProgramId(interp.programs.len());
    interp.programs.push(Rc::clone(&program));
    interp.install_directives(program_id, &program)?;
    let plan = interp.plan_for(
        BodyKey {
            program: program_id,
            directive: None,
        },
        &program.main,
        &program.symbols,
        &program.source,
    );
    let frame = interp.roots.push_slots(plan.len());
    let id = interp.next_activation_id();
    interp.push_activation(Activation::new(
        id,
        Rc::clone(&program),
        program_id,
        plan,
        frame,
    ));
    run_activated(interp, &program)
}

/// [`run_source_with_directives`], keeping what the program printed.
fn say_output_with_directives(interp: &mut Interp, source: &[u8]) -> Vec<u8> {
    run_source_with_directives(interp, source).expect("test program runs");
    std::mem::take(&mut interp.out)
}

/// `run_source`'s second half, split out so `run_source_traced` can put a
/// `TRACE` setting on the activation between the push and the run.
fn run_activated(interp: &mut Interp, _program: &Program) -> Result<Option<ObjRef>, Failure> {
    interp.run_activation().map(Ended::value)
}

fn say_output(interp: &mut Interp, source: &[u8]) -> Vec<u8> {
    run_source(interp, source).expect("test program runs");
    std::mem::take(&mut interp.out)
}

/// `run_source`, with `TRACE R` already in force for the activation it
/// pushes.
fn run_source_traced(interp: &mut Interp, source: &[u8]) -> Result<Option<ObjRef>, Failure> {
    let program = parse_program(source.to_vec()).expect("test program parses");
    let program = activate(interp, program);
    interp
        .set_trace_mode(crate::trace::mode_from_setting(b"r").expect("R is a valid TRACE setting"));
    run_activated(interp, &program)
}

fn say_output_traced(interp: &mut Interp, source: &[u8]) -> Vec<u8> {
    run_source_traced(interp, source).expect("test program runs");
    std::mem::take(&mut interp.out)
}

mod address;
mod assignment;
mod branch;
mod conditions;
mod directives;
mod indent;
mod iterate_leave;
mod loops;
mod message;
mod numeric;
mod result;
mod routines;
mod scope;
mod signal;
