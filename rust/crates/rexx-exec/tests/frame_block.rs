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

//! The register frame arena's block size, as an embedding sets it.

use rexx_exec::{FrameBlock, Invocation, Outcome, run_program, run_program_collect_every_alloc};

const PATH: &str = "<frame block test>";

/// Each level holds a heap string in a register across the call below it.
const RECURSIVE: &[u8] = b"say length(f(30))\nexit\n\
f: procedure\n  arg n\n  if n = 0 then return ''\n  \
return 'a string longer than any inline text' || n || f(n - 1) || 'z'\n";

fn run(block: FrameBlock, collect: bool) -> Outcome {
    let invocation = Invocation::none().with_frame_block(block);
    if collect {
        run_program_collect_every_alloc(PATH, RECURSIVE.to_vec(), invocation)
    } else {
        run_program(PATH, RECURSIVE.to_vec(), invocation)
    }
}

/// With one frame start per block, every level's registers sit in a block
/// below the current one while the collector runs after every allocation.
#[test]
fn registers_in_every_block_stay_rooted_under_the_smallest_block() {
    let reference = run(FrameBlock::DEFAULT, false);
    assert_eq!(
        reference.exit_code,
        0,
        "{:?}",
        String::from_utf8_lossy(&reference.stderr)
    );
    assert_eq!(reference.stdout, b"1161\n");

    let smallest = FrameBlock::new(FrameBlock::MIN).expect("in range");
    let stressed = run(smallest, true);
    assert!(stressed.collections > 0);
    assert_eq!(
        (stressed.exit_code, &stressed.stdout, &stressed.stderr),
        (reference.exit_code, &reference.stdout, &reference.stderr)
    );
}

#[test]
fn a_block_size_outside_the_range_is_refused_before_any_run() {
    assert_eq!(FrameBlock::new(0), None);
    assert_eq!(FrameBlock::new(FrameBlock::MAX + 1), None);
    assert!(FrameBlock::new(FrameBlock::MAX).is_some());
}
