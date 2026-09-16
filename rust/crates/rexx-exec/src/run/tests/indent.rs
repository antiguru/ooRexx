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

// ---- Task 11's own indentation quantity ----

#[test]
fn one_two_and_three_enclosing_dos_indent_by_two_four_and_six() {
    for (source, spaces) in [
        (&b"do i = 1 to 3\nsay 1/0\nend"[..], 2),
        (&b"do i = 1 to 3\ndo j = 1 to 3\nsay 1/0\nend\nend"[..], 4),
        (
            &b"do i = 1 to 3\ndo j = 1 to 3\ndo k = 1 to 3\nsay 1/0\nend\nend\nend"[..],
            6,
        ),
    ] {
        let mut interp = Interp::new();
        run_source(&mut interp, source).unwrap_err();
        let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
        else {
            panic!("not a clause site")
        };
        assert_eq!(indent, spaces, "{source:?}");
    }
}

/// **The test the coordinator's own design review asked for by name**:
/// a raise *after* a loop has already exited, at a shallower lexical
/// depth than the loop's own body -- exactly the shape a live,
/// imperfectly-unwound `Interp` counter would over-indent and a purely
/// static function of `(instructions, index)` cannot, because there is
/// no state left over from the loop's own three completed iterations
/// for anything to have failed to unwind.
#[test]
fn the_indent_after_a_loop_has_already_exited_is_not_left_over_from_it() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"do i = 1 to 3\nsay i\nend\nsay 1/0").unwrap_err();
    let FailureSite::Clause { indent, text, .. } =
        interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(
        indent, 0,
        "top level, after the loop, not the loop's own two"
    );
    assert_eq!(text, b"say 1/0".to_vec());
}

/// `DO`'s own control-setup expressions (`TO`/`BY`/`FOR`/the repeat
/// count/`OVER`'s target) are evaluated **before** the loop's own body
/// is entered, and are unindented even though the `DO` clause they
/// belong to sits at the same lexical position the body's own two
/// spaces would apply to -- the case the brief's own bullet 4 names
/// (`do i = 1 to 'x'` gets none), re-measured across the sibling forms
/// the brief did not enumerate.
#[test]
fn control_setup_expressions_are_unindented_unlike_the_loop_body_they_precede() {
    for source in [
        &b"do i = 1 to 'x'\nsay 1\nend"[..],
        &b"do 1/0\nsay 1\nend"[..],
        &b"do i = 1 to 3 for 1/0\nsay 1\nend"[..],
        &b"do i = 1 to 3 by 1/0\nsay 1\nend"[..],
    ] {
        let mut interp = Interp::new();
        run_source(&mut interp, source).unwrap_err();
        let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
        else {
            panic!("not a clause site")
        };
        assert_eq!(indent, 0, "{source:?}");
    }
}

/// `WHILE`/`UNTIL` are tested **inside** the loop's own frame, unlike
/// the header's control-setup expressions -- measured, `do while 1/0`
/// is indented two at top level, not zero.
#[test]
fn while_and_until_are_indented_inside_the_loops_own_frame() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"do while 1/0\nsay 1\nend").unwrap_err();
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(indent, 2);

    let mut interp = Interp::new();
    run_source(&mut interp, b"do until 1/0\nsay 1\nend").unwrap_err();
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(indent, 2);
}

/// A `SELECT`'s own scan (evaluating each `WHEN`'s condition) is
/// indented two, present even before any `WHEN` matches -- the finding
/// this task made that the brief did not state (measured: `select /
/// when 1/0 then nop / end` indents the failing `WHEN`'s own condition
/// by two, not zero).
#[test]
fn a_whens_own_condition_is_indented_at_the_selects_own_two_spaces() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"select\nwhen 1/0 then nop\nend").unwrap_err();
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(indent, 2);
}

/// A matched `WHEN`'s own `THEN` body indents six (`SELECT`'s two, plus
/// the `WHEN`-`THEN` shape's own four, the same as an `IF`'s matched
/// branch); `OTHERWISE`'s own body indents only four, **not** six --
/// the second finding this task made that the brief did not state,
/// because `OTHERWISE` is not built from the same double-frame `IF`-
/// shaped machinery a `WHEN`'s `THEN` is.
#[test]
fn a_matched_whens_then_body_indents_six_but_otherwises_body_indents_only_four() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"select\nwhen 1 = 1 then say 1/0\nend").unwrap_err();
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(indent, 6);

    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"select\nwhen 1 = 0 then nop\notherwise\nsay 1/0\nend",
    )
    .unwrap_err();
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(
        indent, 4,
        "OTHERWISE's own body, one frame, not the WHEN-THEN shape's two"
    );
}

/// An `IF`'s matched branch is four spaces, for **both** `THEN` and a
/// plain `ELSE` (not only the "else if" chain shape the brief's own
/// example used) -- and an "else if" chain nests two such branches,
/// giving eight.
#[test]
fn an_ifs_matched_then_or_else_branch_indents_four_and_an_else_if_chain_indents_eight() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"if 1 = 1 then say 1/0").unwrap_err();
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(indent, 4, "THEN");

    let mut interp = Interp::new();
    run_source(&mut interp, b"if 1 = 0 then say 2\nelse say 1/0").unwrap_err();
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(indent, 4, "a plain ELSE, not part of an else-if chain");

    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"if 1 = 0 then say 2\nelse if 1 = 1 then say 1/0",
    )
    .unwrap_err();
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(
        indent, 8,
        "the outer ELSE's own four, plus the inner IF's THEN's own four"
    );
}

/// Task 13's own four `static_indent` fixes, found while building
/// `TRACE`'s clause echo -- **not a second computation of the
/// indentation quantity**, a bug in the existing one, for a case
/// Task 10/11 structurally could not exercise: none of `THEN`/`ELSE`/
/// `OTHERWISE`/a `WHEN`'s own `THEN` carries an expression, so none of
/// them can ever raise a condition and become a `FailureSite` -- the
/// only way anything before this task ever asked `static_indent` a
/// question. `TRACE` echoes every stepped instruction, markers
/// included, which is what finally asks.
#[test]
fn all_indents_fills_what_static_indent_computes_for_every_corpus_program() {
    let corpus = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let mut compared = 0usize;
    let mut positions = 0usize;
    let mut directories = vec![corpus];
    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(&directory).expect("a readable corpus directory") {
            let path = entry.expect("a readable directory entry").path();
            if path.is_dir() {
                directories.push(path);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rex") {
                continue;
            }
            let bytes = std::fs::read(&path).expect("a readable corpus program");
            let Ok(program) = parse_program(bytes) else {
                continue;
            };
            let instructions = &program.main.instructions;
            let filled = all_indents(instructions);
            let expected: Vec<usize> = (0..instructions.len())
                .map(|index| static_indent(instructions, index))
                .collect();
            assert_eq!(
                filled.as_ref(),
                expected.as_slice(),
                "{} disagrees",
                path.display()
            );
            compared += 1;
            positions += instructions.len();
        }
    }
    assert!(
        compared > 40 && positions > 500,
        "only {compared} programs and {positions} positions were compared, \
         which is too little of the corpus to have tested anything"
    );
}

#[test]
fn a_then_else_when_then_or_otherwise_markers_own_clause_indents_half_its_bodys() {
    // `IF`'s own `THEN`: `then_start` used to fall into the body's `+4`
    // branch (the branch's own recursive call returns 0 for the very
    // first position of its range) and answer 4, not 2.
    let instructions = instructions_of(b"if 1 = 1 then say 'x'");
    let then_start = if_then_start(&instructions, 0);
    assert_eq!(static_indent(&instructions, then_start), 2, "IF's own THEN");

    // `IF`'s own `ELSE`: `target == false_target` used to fall through
    // this whole arm entirely (`pc = else_end; continue`, which passes
    // straight over the marker's own index in the enclosing walk) and
    // answer 0, the enclosing level, not 2.
    let instructions = instructions_of(b"if 1 = 0 then say 'x'\nelse say 'y'");
    let if_index = 0;
    let InstructionKind::If { false_target, .. } = &instructions[if_index].kind else {
        panic!("index 0 is the IF")
    };
    let else_index = false_target.expect("this IF has an ELSE");
    assert_eq!(static_indent(&instructions, else_index), 2, "IF's own ELSE");

    // A `WHEN`'s own `THEN`, sharing `InstructionKind::Then` with `IF`
    // (`instruction.rs`'s `if_instruction` builds both): `target ==
    // body_start` used to match the loop's own `>=` and answer 6, the
    // body's value, not 4.
    let instructions = instructions_of(b"select\nwhen 1 = 1 then say 'x'\nend");
    let when_index = 1;
    let then_start = if_then_start(&instructions, when_index);
    assert_eq!(
        static_indent(&instructions, then_start),
        4,
        "WHEN's own THEN"
    );

    // `OTHERWISE`'s own clause -- **this one used to abort the process**,
    // not merely answer a wrong number: `target == *otherwise_index`
    // matched neither the `whens` loop nor the body check below it, and
    // fell to `unreachable!("a resolved SELECT's own range holds only
    // its WHENs and OTHERWISE")`. Reproduced against the tree before
    // this fix (`cargo test` aborted this exact test with that message,
    // `run.rs`'s panic site named in the fix's own commit) rather than
    // inferred.
    let instructions = instructions_of(b"select\nwhen 1 = 0 then nop\notherwise\nsay 'y'\nend");
    let otherwise_index = instructions
        .iter()
        .find_map(|i| match &i.kind {
            InstructionKind::Select { otherwise, .. } => *otherwise,
            _ => None,
        })
        .expect("this SELECT has an OTHERWISE");
    assert_eq!(
        static_indent(&instructions, otherwise_index),
        2,
        "OTHERWISE's own marker clause"
    );
}

/// `if_instruction` (`rexx-parse`'s `instruction.rs`) gives both `IF`
/// and a `WHEN`'s own `THEN` the identical shape: the `Then` marker
/// sits immediately after the condition-bearing instruction at
/// `condition_index`. A free function rather than inlined three times
/// in the test above, since all three call sites need the exact same
/// index arithmetic and nothing else about `Instruction` to find it.
fn if_then_start(instructions: &[Instruction], condition_index: usize) -> usize {
    assert!(
        matches!(
            instructions[condition_index].kind,
            InstructionKind::If { .. }
                | InstructionKind::When { .. }
                | InstructionKind::WhenCase { .. }
        ),
        "condition_index must be an IF, a WHEN or a WHEN CASE"
    );
    condition_index + 1
}

/// Parses `source` and returns its top-level instruction list, owned --
/// the marker-index tests above need to read `InstructionKind` fields
/// directly (`false_target`, `otherwise`) rather than only run the
/// program to a raise, since a marker clause cannot raise at all.
fn instructions_of(source: &[u8]) -> Vec<Instruction> {
    parse_program(source.to_vec())
        .expect("test program parses")
        .main
        .instructions
}

/// Nesting a `SELECT` (with a matched `WHEN`) inside a `DO`: two plus
/// two plus four -- confirming the additive model composes across
/// different construct kinds, not only same-kind nesting.
#[test]
fn a_select_nested_inside_a_do_composes_the_two_constructs_own_contributions() {
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"do i = 1 to 3\nselect\nwhen 1 = 1 then say 1/0\notherwise nop\nend\nend",
    )
    .unwrap_err();
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(indent, 8);
}
