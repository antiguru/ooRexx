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
use crate::{Novalue, planned_code};
use rexx_parse::{Program, parse_interpret, parse_program};

/// `indent_of` answers what `static_indent` answers, for every index.
#[test]
fn indent_of_answers_what_static_indent_answers_at_every_index() {
    let source = b"if 1 = 1 then\n  do i = 1 to 2\n    say i\n  end\nelse\n  nop\nselect\n  when 1 = 0 then nop\n  otherwise\n    say 'o'\nend\n";
    let program = parse_program(source.to_vec()).expect("test program parses");
    let plan = Plan::build(
        &program.main,
        &program.symbols,
        Some(&program.source),
        BodyKind::Plain,
    );
    let instructions = &program.main.instructions;
    assert!(instructions.len() > 8, "the program lost its shape");

    let expected: Vec<usize> = (0..instructions.len())
        .map(|index| crate::run::static_indent(instructions, index))
        .collect();
    let first: Vec<usize> = (0..instructions.len())
        .map(|index| plan.indent_of(instructions, index))
        .collect();
    assert_eq!(first, expected);
    // The stored table itself, not only what the accessor answers: an
    // accessor that recomputed on every call would satisfy the two
    // assertions above while the table it reads from stayed empty.
    assert_eq!(
        plan.indents.as_ref(),
        expected.as_slice(),
        "what the table actually holds"
    );
    // Not every index the same value, or the assertions above hold for a
    // memo that always answers with slot zero.
    assert!(
        expected
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len()
            > 2,
        "this program does not distinguish enough indent levels to test with: {expected:?}"
    );
}

/// `line_at` answers what `ProgramSource::line_of` answers, for every
/// index.
#[test]
fn line_at_answers_what_line_of_answers_at_every_index() {
    let source =
        b"nop\nsay 1\nif 1 = 1 then\n  nop\nelse\n  nop\ndo i = 1 to 2\n  say i\nend\nsay 'done'\n";
    let program = parse_program(source.to_vec()).expect("test program parses");
    let plan = Plan::build(
        &program.main,
        &program.symbols,
        Some(&program.source),
        BodyKind::Plain,
    );
    let instructions = &program.main.instructions;
    assert!(instructions.len() > 8, "the program lost its shape");

    let expected: Vec<usize> = instructions
        .iter()
        .map(|instruction| program.source.line_of(instruction.clause_span.start))
        .collect();
    let answered: Vec<usize> = instructions
        .iter()
        .enumerate()
        .map(|(index, instruction)| plan.line_at(instruction, &program.source, index))
        .collect();
    assert_eq!(answered, expected);
    assert_eq!(
        plan.lines.as_ref(),
        expected.as_slice(),
        "what the table actually holds"
    );
    // Not one line repeated, or a table that answered its first entry
    // everywhere would pass the two assertions above.
    assert!(
        expected
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len()
            > 5,
        "this program does not distinguish enough lines to test with: {expected:?}"
    );
}

/// A plan built with no source carries no table, and `line_at` still
/// answers -- which is the fragment's case and the whole of why the
/// accessor has a fallback arm.
#[test]
fn line_at_falls_back_when_the_plan_was_built_without_a_source() {
    let source = b"nop\nsay 1\nsay 2\n";
    let program = parse_program(source.to_vec()).expect("test program parses");
    let plan = Plan::build(&program.main, &program.symbols, None, BodyKind::Plain);
    assert!(plan.lines.is_empty(), "no source, so no table");
    for (index, instruction) in program.main.instructions.iter().enumerate() {
        assert_eq!(
            plan.line_at(instruction, &program.source, index),
            program.source.line_of(instruction.clause_span.start),
            "index {index}"
        );
    }
}

/// `clause_line_at` answers what `clause_line` answers, at every index,
/// with the override unset and with it set.
#[test]
fn clause_line_at_answers_what_clause_line_answers_and_the_override_still_wins() {
    let source =
        b"nop\nsay 1\nif 1 = 1 then\n  nop\nelse\n  nop\ndo i = 1 to 2\n  say i\nend\nsay 'done'\n";
    let program = parse_program(source.to_vec()).expect("test program parses");
    let plan = Plan::build(
        &program.main,
        &program.symbols,
        Some(&program.source),
        BodyKind::Plain,
    );
    let code = planned_code(&program, &plan);
    let mut interp = Interp::new();

    for (index, instruction) in program.main.instructions.iter().enumerate() {
        assert_eq!(
            interp.clause_line_at(&code, index, instruction, Some(&program.source)),
            interp.clause_line(Some(&program.source), instruction),
            "index {index}"
        );
    }

    interp.clause_line_override = Some(4242);
    for (index, instruction) in program.main.instructions.iter().enumerate() {
        assert_eq!(
            interp.clause_line_at(&code, index, instruction, Some(&program.source)),
            Some(4242),
            "the override outranks the table at index {index}"
        );
    }

    // `source: None` answers `None` whatever the override says, exactly
    // as `clause_line` does -- the two must not come apart on that arm
    // either.
    interp.clause_line_override = None;
    for (index, instruction) in program.main.instructions.iter().enumerate() {
        assert_eq!(
            interp.clause_line_at(&code, index, instruction, None),
            None,
            "index {index}"
        );
    }
}

/// The table matches `ProgramSource::line_of` for every instruction of
/// every corpus program that parses.
#[test]
fn build_fills_what_line_of_computes_for_every_corpus_program() {
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
            let plan = Plan::build(
                &program.main,
                &program.symbols,
                Some(&program.source),
                BodyKind::Plain,
            );
            let expected: Vec<usize> = program
                .main
                .instructions
                .iter()
                .map(|instruction| program.source.line_of(instruction.clause_span.start))
                .collect();
            assert_eq!(
                plan.lines.as_ref(),
                expected.as_slice(),
                "{} disagrees",
                path.display()
            );
            compared += 1;
            positions += program.main.instructions.len();
        }
    }
    assert!(
        compared > 40 && positions > 500,
        "only {compared} programs and {positions} positions were compared, \
         which is too little of the corpus to have tested anything"
    );
}

/// Pushes a fresh top-level activation for `program`, the same setup
/// `Interp::run` does, so these tests can drive `slot_of`/`Plan` through
/// a live activation without running the whole instruction loop.
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
    interp.push_activation(crate::Activation::new(
        id,
        Rc::clone(&program),
        program_id,
        plan,
        frame,
    ));
    program
}

/// The measured bug this fix closes (Task 6 fix dispatch): before it,
/// every one of the first four bodies below built an *empty* plan --
/// `Plan::note`'s `_ => {}` dropped `Stem`/`Compound` outright, and
/// `Plan::build`'s own `_ => {}` meant an instruction it did not
/// explicitly list (a `DROP`, a controlled `DO`) never even reached
/// `note` at all. This asserts the plan's actual *contents* -- which
/// names ended up in `plan.names` -- not merely that resolution still
/// works afterwards through the `extra`/`grow_slots` fallback, which is
/// the exact bar the dispatch set: neutering `Plan::build` to return an
/// empty plan unconditionally must fail this test. It does not fail the
/// pre-existing tests below, whose own assertions run through
/// `slot_of`, which still gives the right *answer* via that fallback
/// even when the plan is empty -- only the plan's own contents catch
/// the regression.
#[test]
fn build_registers_every_name_a_stem_or_compound_touches() {
    let cases: &[(&[u8], &[&str])] = &[
        // say a.b -- the stem "A." and the tail-piece variable "B",
        // neither of which `note`'s old match handled at all.
        (&b"say a.b"[..], &["A.", "B"][..]),
        // a.1 = 'x' -- a bare digit tail is a constant (D15a), so only
        // the stem itself needs a slot.
        (b"a.1 = 'x'", &["A."]),
        // q. = 1 -- a bare Stem assignment target, dropped by the same
        // `_ => {}` a Compound was.
        (b"q. = 1", &["Q."]),
        // say v -- unaffected by this fix, kept as the control case:
        // if this one broke, the fix broke something that already
        // worked, not just left something unfixed.
        (b"say v", &["V"]),
        // drop a.b.c -- `Drop`'s own `_ => {}` in the pre-fix `build`
        // meant this never reached `note` at all, compound or not.
        // Both tail pieces are letter-led, so both are variables.
        (b"drop a.b.c", &["A.", "B", "C"]),
        // do i = 1 to 5 / end -- a controlled loop's control variable,
        // which never went through `note` before either: `Do` was not
        // one of `build`'s three explicitly handled kinds.
        (b"do i = 1 to 5\nend", &["I"]),
    ];
    for (source, expected_names) in cases {
        let program = parse_program(source.to_vec()).expect("test program parses");
        let plan = Plan::build(
            &program.main,
            &program.symbols,
            Some(&program.source),
            BodyKind::Plain,
        );
        assert!(
            !plan.names.is_empty(),
            "{:?} must build a non-empty plan",
            String::from_utf8_lossy(source)
        );
        for name in *expected_names {
            assert!(
                plan.names.contains_key(name.as_bytes()),
                "{:?}: expected {name:?} in the plan, got {:?}",
                String::from_utf8_lossy(source),
                plan.names.keys().collect::<Vec<_>>()
            );
        }
    }
}

/// The table above only asserts that each expected name is *present*,
/// so an over-wide `build` -- one that registers every name in the
/// program, or anything else besides the right set -- would still pass
/// it. That is exactly the shape of gap that let the original `_ =>
/// {}` catch-all through unnoticed: presence checks cannot fail on
/// extra entries, only on missing ones.
#[test]
fn build_registers_exactly_the_expected_set_not_merely_a_superset() {
    let cases: &[(&[u8], &[&str])] = &[(b"leave lbl", &[]), (b"say .nil", &[])];
    for (source, expected_names) in cases {
        let program = parse_program(source.to_vec()).expect("test program parses");
        let plan = Plan::build(
            &program.main,
            &program.symbols,
            Some(&program.source),
            BodyKind::Plain,
        );

        // The reserved names are in every plan, so they are checked
        // for presence once and then set aside; what each case is about
        // is the body's *own* names, and that comparison stays exact.
        for reserved in [b"RESULT".as_slice(), b"RC".as_slice(), b"SIGL".as_slice()] {
            assert!(
                plan.names.contains_key(reserved),
                "{:?}: every plan holds {}",
                String::from_utf8_lossy(source),
                String::from_utf8_lossy(reserved)
            );
        }
        let mut actual: Vec<&[u8]> = plan
            .names
            .keys()
            .map(|k| &**k)
            .filter(|name| !matches!(*name, b"RESULT" | b"RC" | b"SIGL"))
            .collect();
        actual.sort();
        let mut expected: Vec<&[u8]> = expected_names.iter().map(|n| n.as_bytes()).collect();
        expected.sort();

        assert_eq!(
            actual,
            expected,
            "{:?}: expected the plan's own key set to be exactly {expected_names:?}",
            String::from_utf8_lossy(source)
        );
    }
}

/// A method body's plan holds `SELF` and `SUPER`, and no other body's
/// does.
#[test]
fn only_a_method_body_holds_self_and_super() {
    let source = b"nop\n::routine r\n  nop\n::class k\n::method m\n  x = 1\n";
    let program = Rc::new(parse_program(source.to_vec()).expect("test program parses"));
    let mut interp = Interp::new();
    let program_id = ProgramId(interp.programs.len());
    interp.programs.push(Rc::clone(&program));

    let position = |wanted: fn(&DirectiveKind) -> bool| {
        program
            .directives
            .iter()
            .position(|directive| wanted(&directive.kind))
            .expect("the program has this directive")
    };
    let routine = position(|kind| matches!(kind, DirectiveKind::Routine(_)));
    let method = position(|kind| matches!(kind, DirectiveKind::Method(_)));

    for (selector, what, expected) in [
        (None, "the main body", false),
        (Some(routine), "::ROUTINE r", false),
        (Some(method), "::METHOD m", true),
    ] {
        let body = crate::activation::body_of(&program, selector)
            .unwrap_or_else(|| panic!("{what} has a body"));
        let plan = interp.plan_for(
            BodyKey {
                program: program_id,
                directive: selector,
            },
            body,
            &program.symbols,
            &program.source,
        );
        for name in [b"SELF".as_slice(), b"SUPER".as_slice()] {
            assert_eq!(
                plan.slot_of(name).is_some(),
                expected,
                "{what}: {} in the plan, which holds {:?}",
                String::from_utf8_lossy(name),
                plan.names.keys().collect::<Vec<_>>()
            );
        }
    }

    // Registered after the body's own names, so the method body's own `X`
    // is still slot 0 -- the property that let the two be added without
    // renumbering anything a plan already answered.
    let body = crate::activation::body_of(&program, Some(method)).expect("::METHOD m has a body");
    let plan = interp.plan_for(
        BodyKey {
            program: program_id,
            directive: Some(method),
        },
        body,
        &program.symbols,
        &program.source,
    );
    assert_eq!(plan.slot_of(b"X"), Some(0));
}

/// `build` records a compound's split under **the compound's own id**.
#[test]
fn build_records_a_compounds_split_under_the_compounds_own_id() {
    let source = b"drop dd.jj; do aa.ii = 1 to 2; say v.i.7; end";
    let program = parse_program(source.to_vec()).expect("test program parses");
    let plan = Plan::build(
        &program.main,
        &program.symbols,
        Some(&program.source),
        BodyKind::Plain,
    );

    let variable = |text: &str, at: Option<usize>| TailPiece::Variable {
        name: text.as_bytes().into(),
        at,
    };
    let constant = |text: &str| TailPiece::Constant(text.as_bytes().into());

    let mut found: Vec<(&str, &CompoundName)> = Vec::new();
    for instruction in &program.main.instructions {
        match &instruction.kind {
            InstructionKind::Drop { variables } => {
                let (VariableRef::Direct(id) | VariableRef::Indirect(id)) = variables[0];
                found.push((symbols_name(&program, id), expect_entry(&plan, id)));
            }
            InstructionKind::Do(loop_) | InstructionKind::Loop(loop_) => {
                let LoopKind::Controlled(controlled) = &loop_.kind else {
                    panic!("expected a controlled loop, got {:?}", loop_.kind);
                };
                let id = controlled.control;
                found.push((symbols_name(&program, id), expect_entry(&plan, id)));
            }
            InstructionKind::Say {
                expression: Some(expr),
            } => {
                let ExprKind::Compound(id) = expr.kind else {
                    panic!("expected a compound expression, got {:?}", expr.kind);
                };
                found.push((symbols_name(&program, id), expect_entry(&plan, id)));
            }
            _ => {}
        }
    }

    let expected: Vec<(&str, CompoundName)> = vec![
        (
            "DD.JJ",
            CompoundName {
                stem: b"DD.".as_slice().into(),
                stem_at: Some(0),
                tails: vec![variable("JJ", Some(1))].into(),
            },
        ),
        (
            "AA.II",
            CompoundName {
                stem: b"AA.".as_slice().into(),
                stem_at: None,
                tails: vec![variable("II", None)].into(),
            },
        ),
        (
            "V.I.7",
            CompoundName {
                stem: b"V.".as_slice().into(),
                stem_at: Some(3),
                tails: vec![variable("I", Some(4)), constant("7")].into(),
            },
        ),
    ];
    // The slot numbers above are the pass's own, in the order it assigned
    // them: `DD.` then `JJ` for the `DROP`, `AA.II` whole for the control
    // variable, then `V.` and `I` for the `SAY`. Asserted here so that a
    // reader can see where 1 and 4 come from, and so that a change to the
    // assignment order fails on the map rather than only on the pieces.
    let mut names: Vec<(&[u8], usize)> = plan
        .names
        .iter()
        .map(|(name, slot)| (&**name, *slot))
        .collect();
    names.sort_by_key(|(_, slot)| *slot);
    assert_eq!(
        names,
        vec![
            (b"DD.".as_slice(), 0),
            (b"JJ".as_slice(), 1),
            (b"AA.II".as_slice(), 2),
            (b"V.".as_slice(), 3),
            (b"I".as_slice(), 4),
            // `Plan::build` registers these after the body's own names,
            // so they land at the end and the numbers above are unmoved.
            (b"RESULT".as_slice(), 5),
            (b"RC".as_slice(), 6),
            (b"SIGL".as_slice(), 7),
        ]
    );
    let expected: Vec<(&str, &CompoundName)> = expected
        .iter()
        .map(|(name, entry)| (*name, entry))
        .collect();
    assert_eq!(found, expected);
}

/// One symbol recorded by `note_compound_name` and by `bind` keeps the
/// slots, whichever order the pass reaches them in.
#[test]
fn a_control_variable_does_not_take_the_slots_off_a_compound_already_seen() {
    let source = b"say v.i; do v.i = 1 to 2; end";
    let program = parse_program(source.to_vec()).expect("test program parses");
    let plan = Plan::build(
        &program.main,
        &program.symbols,
        Some(&program.source),
        BodyKind::Plain,
    );

    let InstructionKind::Say {
        expression: Some(expr),
    } = &program.main.instructions[0].kind
    else {
        panic!(
            "expected a SAY first, got {:?}",
            program.main.instructions[0].kind
        );
    };
    let ExprKind::Compound(id) = expr.kind else {
        panic!("expected a compound expression, got {:?}", expr.kind);
    };
    // The same id in both positions is the whole premise, so it is
    // checked rather than assumed: `bind` addresses `compounds` by the
    // control variable's id, and if that were a different symbol from the
    // `SAY`'s there would be no overwrite to guard against.
    let (InstructionKind::Do(loop_) | InstructionKind::Loop(loop_)) =
        &program.main.instructions[1].kind
    else {
        panic!(
            "expected a DO second, got {:?}",
            program.main.instructions[1].kind
        );
    };
    let LoopKind::Controlled(controlled) = &loop_.kind else {
        panic!("expected a controlled loop, got {:?}", loop_.kind);
    };
    assert_eq!(controlled.control, id);

    assert_eq!(
        expect_entry(&plan, id).tails.as_ref(),
        [TailPiece::Variable {
            name: b"I".as_slice().into(),
            at: Some(1),
        }]
    );
}

/// The other build order for one symbol, which rests on the other
/// filler's rule.
#[test]
fn a_compound_seen_after_the_control_variable_still_gets_its_slots() {
    let source = b"do v.i = 1 to 2; end; say v.i";
    let program = parse_program(source.to_vec()).expect("test program parses");
    let plan = Plan::build(
        &program.main,
        &program.symbols,
        Some(&program.source),
        BodyKind::Plain,
    );

    let (InstructionKind::Do(loop_) | InstructionKind::Loop(loop_)) =
        &program.main.instructions[0].kind
    else {
        panic!(
            "expected a DO first, got {:?}",
            program.main.instructions[0].kind
        );
    };
    let LoopKind::Controlled(controlled) = &loop_.kind else {
        panic!("expected a controlled loop, got {:?}", loop_.kind);
    };
    let id = controlled.control;

    let last = program.main.instructions.last().expect("a body");
    let InstructionKind::Say {
        expression: Some(expr),
    } = &last.kind
    else {
        panic!("expected a SAY last, got {:?}", last.kind);
    };
    let ExprKind::Compound(say_id) = expr.kind else {
        panic!("expected a compound expression, got {:?}", expr.kind);
    };
    assert_eq!(say_id, id);

    // `V.I` whole takes slot 0 from `bind`, then `V.` and `I` take 1 and
    // 2 from `note_compound_name`. The piece's slot is 2, so the entry
    // that survived is the second one.
    assert_eq!(
        expect_entry(&plan, id).tails.as_ref(),
        [TailPiece::Variable {
            name: b"I".as_slice().into(),
            at: Some(2),
        }]
    );
}

/// **A compound tail piece can be bound in `extra` rather than in the
/// plan, and this is the shape that reaches it.**
#[test]
fn a_tail_piece_with_no_plan_slot_binds_in_extra() {
    let mut interp = Interp::new();
    let program =
        parse_program(b"do za.zi = 1 to 3\nnop\nend".to_vec()).expect("test program parses");
    let (InstructionKind::Do(loop_) | InstructionKind::Loop(loop_)) =
        &program.main.instructions[0].kind
    else {
        panic!(
            "expected a DO first, got {:?}",
            program.main.instructions[0].kind
        );
    };
    let LoopKind::Controlled(controlled) = &loop_.kind else {
        panic!("expected a controlled loop, got {:?}", loop_.kind);
    };
    let id = controlled.control;

    let plan = Plan::build(
        &program.main,
        &program.symbols,
        Some(&program.source),
        BodyKind::Plain,
    );
    assert_eq!(
        plan.slot_of(b"ZI"),
        None,
        "the plan binds the whole ZA.ZI and nothing named ZI"
    );
    assert_eq!(
        expect_entry(&plan, id).tails.as_ref(),
        [TailPiece::Variable {
            name: b"ZI".as_slice().into(),
            at: None,
        }]
    );

    let program = activate(&mut interp, program);
    let plan = Plan::build(
        &program.main,
        &program.symbols,
        Some(&program.source),
        BodyKind::Plain,
    );
    let code = planned_code(&program, &plan);
    // Unset, so the piece derives its own spelling -- the ordinary
    // uninitialised read, reached here through `extra` and growth.
    let key = interp
        .tail_key(&code, id)
        .expect("the tail pieces are strings");
    assert_eq!(key, b"ZI");
    assert!(
        interp.activation().extra.contains_key(b"ZI".as_slice()),
        "the piece must be recorded in extra, which is the source a \
         precomputed slot would have skipped"
    );
}

/// **A compound's stem can be bound in `extra` rather than in the plan,
/// and this is the shape that reaches it.**
#[test]
fn a_stem_with_no_plan_slot_binds_in_extra() {
    let mut interp = Interp::new();
    let program =
        parse_program(b"do za.zi = 1 to 3\nnop\nend".to_vec()).expect("test program parses");
    let (InstructionKind::Do(loop_) | InstructionKind::Loop(loop_)) =
        &program.main.instructions[0].kind
    else {
        panic!(
            "expected a DO first, got {:?}",
            program.main.instructions[0].kind
        );
    };
    let LoopKind::Controlled(controlled) = &loop_.kind else {
        panic!("expected a controlled loop, got {:?}", loop_.kind);
    };
    let id = controlled.control;

    let plan = Plan::build(
        &program.main,
        &program.symbols,
        Some(&program.source),
        BodyKind::Plain,
    );
    assert_eq!(
        plan.slot_of(b"ZA."),
        None,
        "the plan binds the whole ZA.ZI and nothing named ZA."
    );
    assert_eq!(expect_entry(&plan, id).stem_at, None);

    let program = activate(&mut interp, program);
    let plan = Plan::build(
        &program.main,
        &program.symbols,
        Some(&program.source),
        BodyKind::Plain,
    );
    let code = planned_code(&program, &plan);
    let (stem_name, stem_at) = code.stem(id);
    assert_eq!(stem_name, b"ZA.");
    assert_eq!(stem_at, None);

    // Nothing has been written, so the tail derives its own name from the
    // read site's spelling -- the ordinary uninitialised compound read,
    // reached here through `extra` and growth.
    let key = interp
        .tail_key(&code, id)
        .expect("the tail pieces are strings");
    let (value, novalue) = interp.stem_get_at(stem_name, stem_at, &key);
    assert_eq!(novalue, Novalue::Unset);
    assert_eq!(&*interp.to_text(value), b"ZA.ZI");
    assert!(
        interp.activation().extra.contains_key(b"ZA.".as_slice()),
        "the stem must be recorded in extra, which is the source a \
         precomputed slot would have skipped"
    );
}

fn symbols_name(program: &Program, id: SymbolId) -> &str {
    program.symbols.name(id)
}

fn expect_entry(plan: &Plan, id: SymbolId) -> &CompoundName {
    plan.compound(id)
        .expect("the pass recorded this compound's split under its own id")
}

/// **`Plan::bind` records the stem's slot for a stem-shaped name and not
/// for a compound-shaped one**, and the pair is what makes that a decision
/// rather than a coincidence of one spelling.
#[test]
fn bind_keeps_a_stem_shaped_names_own_slot_as_its_stems() {
    let source = b"zt. = 'v'\ndo zs. = 1 to 2\nnop\nend\ndo aa.ii = 1 to 2\nnop\nend";
    let program = parse_program(source.to_vec()).expect("test program parses");
    let plan = Plan::build(
        &program.main,
        &program.symbols,
        Some(&program.source),
        BodyKind::Plain,
    );

    let mut found: Vec<(&str, Option<usize>)> = Vec::new();
    for instruction in &program.main.instructions {
        let id = match &instruction.kind {
            InstructionKind::Assignment { target, .. } => match target.kind {
                ExprKind::Stem(id) => id,
                ref other => panic!("expected a stem target, got {other:?}"),
            },
            InstructionKind::Do(loop_) | InstructionKind::Loop(loop_) => {
                let LoopKind::Controlled(controlled) = &loop_.kind else {
                    panic!("expected a controlled loop, got {:?}", loop_.kind);
                };
                controlled.control
            }
            _ => continue,
        };
        found.push((symbols_name(&program, id), expect_entry(&plan, id).stem_at));
    }

    assert_eq!(
        found,
        vec![("ZT.", Some(0)), ("ZS.", Some(1)), ("AA.II", None)]
    );
}

#[test]
fn a_tail_piece_and_a_plain_variable_share_one_slot() {
    // b = 2 ; say a.b -> A.2 ; a.2 = 'hit' ; say a.b -> hit
    // An implementer who gives tail pieces their own slots gets A.B.
    let mut interp = Interp::new();
    let program = parse_program(b"say a.b".to_vec()).expect("test program parses");
    let id = match &program.main.instructions[0].kind {
        InstructionKind::Say {
            expression: Some(expr),
        } => match expr.kind {
            ExprKind::Compound(id) => id,
            ref other => panic!("expected a compound expression, got {other:?}"),
        },
        other => panic!("expected a SAY with an expression, got {other:?}"),
    };
    let program = activate(&mut interp, program);

    let two = interp.text(b"2");
    let b_slot = interp.slot_of(b"B");
    let frame = interp.activation().frame;
    interp.roots.set_frame_slot(frame, b_slot, two);

    let plan = Plan::build(
        &program.main,
        &program.symbols,
        Some(&program.source),
        BodyKind::Plain,
    );
    let code = planned_code(&program, &plan);
    let key = interp
        .tail_key(&code, id)
        .expect("the tail pieces are strings");
    assert_eq!(key, b"2");
    // `A.` shares its "2" slot with the plain variable `B`'s value,
    // through `a.2`'s own key, which is exactly what `key` resolved to.
    let a2_slot = interp.slot_of(b"A.2");
    assert_ne!(
        a2_slot, b_slot,
        "A.2 is a different variable name from B, so a different slot -- \
         what must share a slot is the tail KEY '2' with the variable B's VALUE, \
         not the names themselves"
    );

    let uninit = interp.stem_get(b"A.", &key).0;
    assert_eq!(&*interp.to_text(uninit), b"A.2");

    let hit = interp.text(b"hit");
    interp.stem_set(b"A.", &key, hit);
    let after = interp.stem_get(b"A.", &key).0;
    assert_eq!(&*interp.to_text(after), b"hit");
}

#[test]
fn a_runtime_name_grows_the_frame() {
    // v = 'X' ; x = 1 ; drop (v) ; say x  ->  X
    // X may not appear in the body at all, so the plan cannot have a
    // slot for it: this program never mentions X in its own text.
    let mut interp = Interp::new();
    let program = parse_program(b"nop".to_vec()).expect("test program parses");
    activate(&mut interp, program);

    // `RESULT`, `RC` and `SIGL` are in every plan (see `Plan::build`), so
    // a body with no variables of its own holds exactly those and nothing
    // more -- which is still the precondition this test needs: `X` is not
    // among them, so reaching it has to grow the frame.
    assert_eq!(
        interp.activation().plan.len(),
        3,
        "a body with no variables of its own holds only the reserved names"
    );
    assert!(
        interp.activation().plan.slot_of(b"X").is_none(),
        "the name this test grows the frame for must not already have a slot"
    );

    let one = interp.number(
        rexx_num::Number::parse("1").unwrap(),
        9,
        rexx_num::Form::Scientific,
    );
    let x_slot = interp.slot_of(b"X");
    let frame = interp.activation().frame;
    interp.roots.set_frame_slot(frame, x_slot, one);

    // `drop (v)` resolves its target *by the current value of v*, not by
    // the literal text "v" -- here that value is "X", so this names the
    // same slot `x_slot` does, exactly like `DROP (v)` would at run time.
    let dynamic_slot = interp.slot_of(b"X");
    assert_eq!(dynamic_slot, x_slot);
    assert!(
        interp.activation().extra.contains_key(b"X".as_slice()),
        "a name the plan never saw must grow into extra, not silently \
         miss or panic"
    );
}

#[test]
fn names_are_keyed_upcased_but_tail_values_are_not() {
    // The two rules live in different decision blocks (D16 vs D15a) and
    // are easy to swap. A NAME is upcased before a `SymbolId` even
    // exists for it -- the *tokenizer* does that (`SymbolTable::intern`),
    // not `Plan` or `slot_of`, which never see a lowercase spelling to
    // begin with: `v.i`, however the source writes it, interns as
    // "V.I", and `compound_parts` decomposes the piece as "I", already
    // upcase. A tail VALUE is different: whatever the piece variable's
    // current *value* renders as, verbatim and case-sensitively.
    let mut interp = Interp::new();
    let program = parse_program(b"say v.i".to_vec()).expect("test program parses");
    let id = match &program.main.instructions[0].kind {
        InstructionKind::Say {
            expression: Some(expr),
        } => match expr.kind {
            ExprKind::Compound(id) => id,
            ref other => panic!("expected a compound expression, got {other:?}"),
        },
        other => panic!("expected a SAY with an expression, got {other:?}"),
    };
    // The compound's own interned spelling is already upcased regardless
    // of how the source wrote it -- the fact D16's "keyed by upcased
    // name" rests on, checked here rather than assumed.
    assert_eq!(program.symbols.name(id), "V.I");

    let program = activate(&mut interp, program);
    let abc = interp.text(b"abc");
    let i_slot = interp.slot_of(b"I");
    let frame = interp.activation().frame;
    interp.roots.set_frame_slot(frame, i_slot, abc);

    let plan = Plan::build(
        &program.main,
        &program.symbols,
        Some(&program.source),
        BodyKind::Plain,
    );
    let code = planned_code(&program, &plan);
    let key = interp
        .tail_key(&code, id)
        .expect("the tail pieces are strings");
    // The tail VALUE "abc" survives verbatim, lowercase and all -- not
    // upcased to "ABC", which is the distinct rule D15a states.
    assert_eq!(key, b"abc");
}

#[test]
fn a_fragments_plan_resolves_against_the_enclosing_frame() {
    // interpret "newvar = 7" ; say newvar + 1  ->  8 (measured on the
    // oracle). A fragment's plan is never cached (BodyKey has no
    // fragment arm) and its bindings land in the enclosing frame via
    // `extra`, not in a frame of its own.
    let mut interp = Interp::new();
    let program = parse_program(b"nop".to_vec()).expect("test program parses");
    activate(&mut interp, program);

    let fragment = parse_interpret(b"newvar = 7".to_vec()).expect("fragment parses");
    let (slots, plan) = interp.fragment_plan(&fragment);
    // The table is sized by the fragment's own symbol table and is mostly
    // `None`; what the fragment *names* is the count of bound entries.
    assert_eq!(
        slots.iter().flatten().count(),
        1,
        "the fragment names exactly one variable"
    );

    let enclosing_slot = interp.slot_of(b"NEWVAR");
    let fragment_slot = slots.iter().flatten().next().copied().expect("one entry");
    // **The remapped plan names the same slot**, which is the half the
    // compiled path depends on: a chunk's ops carry plan slots, and a plan
    // still numbering the fragment's own names would write into whatever
    // the enclosing frame happens to hold at that index.
    assert_eq!(
        plan.names.get(b"NEWVAR".as_slice()).copied(),
        Some(enclosing_slot),
        "the remapped plan's name map still holds the fragment's own \
         local numbering"
    );
    assert_eq!(
        plan.by_symbol.iter().flatten().copied().collect::<Vec<_>>(),
        vec![enclosing_slot],
        "the remapped plan's by_symbol disagrees with the translation \
         `Code::slots` is built from"
    );
    assert_eq!(
        fragment_slot, enclosing_slot,
        "the fragment's own id must resolve to the SAME slot the \
         enclosing body would use for the same name"
    );
}
/// **What may change the `TRACE` setting under a running chunk**, which is
/// the whole of what lets `ir::compile` decide a clause's value echoes at
/// compile time instead of gating each one.
#[test]
fn a_body_that_can_reach_the_trace_setting_is_the_one_that_says_so() {
    let retraces = |source: &[u8]| {
        let program = parse_program(source.to_vec()).expect("test program parses");
        let plan = Plan::build(&program.main, &program.symbols, None, BodyKind::Plain);
        plan.trace_events()
            .iter()
            .any(|event| *event != TraceEvent::Keeps)
    };

    // The instruction, in each of its forms.
    assert!(retraces(b"trace i"));
    assert!(retraces(b"zv = 'i'\ntrace value zv"));
    // The builtin, which sets the *running* activation's own setting where
    // every other call cannot reach it.
    assert!(retraces(b"zg = trace('i')"));
    assert!(retraces(b"call trace 'i'"));
    assert!(retraces(b"zg = TRACE('i')"));
    // Its argument list is no shelter: the walk reaches nested calls.
    assert!(retraces(b"zg = length(trace('i'))"));
    // Text this cannot read.
    assert!(retraces(b"interpret zv"));
    assert!(retraces(b"call (zv) 1"));

    // The adjacent successes: a call, an assignment and a loop that name
    // no route to the setting.
    assert!(!retraces(b"zg = length('ab')"));
    assert!(!retraces(b"call charout , 'x'"));
    assert!(!retraces(b"do zi = 1 to 3\nzs = zi + 1\nend"));
    assert!(!retraces(b"say 'x'"));
    // A *variable* spelled TRACE is not a call to it, but blocking one
    // costs only the optimisation -- so this row records which way the
    // guard falls rather than asserting it must not block.
    let _ = retraces(b"trace_var = 1");
}
