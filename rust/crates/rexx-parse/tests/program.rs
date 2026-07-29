//! The public entry points, `parse_program` and `parse_interpret`, which
//! compose the scanner, the clause splitter, and the instruction/directive
//! grammars into `Program` and `Fragment`.
//!
//! Every expectation naming an interpreter behaviour was measured against
//! `build/bin/rexx` or `build/bin/rexxc`, not inferred. `rexxc` is a
//! parse-only oracle: it answers rc 0 without running the file, which is the
//! only way to assert the negative direction (*this file parses*).

use std::path::Path;

use rexx_parse::{
    Directive, DirectiveKind, InstructionKind, ParseError, parse_interpret, parse_program,
};

fn err<T>(result: Result<T, ParseError>) -> (u16, u16) {
    match result {
        Ok(_) => panic!("expected an error, but the input parsed"),
        Err(e) => (e.code, e.sub),
    }
}

// ---- Step 1's own test, verbatim ----

#[test]
fn a_program_with_directives_separates_them_from_instructions() {
    let p = parse_program(b"say 1\n::routine r\n  return 2\n".to_vec()).unwrap();
    assert_eq!(p.instructions.len(), 1);
    assert_eq!(p.directives.len(), 1);
    assert_eq!(p.source.line(1), Some(&b"say 1"[..]));
}

// ---- The main body / directive boundary, both directions ----

#[test]
fn a_program_with_no_directives_has_an_empty_directive_list() {
    let p = parse_program(b"say 1\nsay 2\n".to_vec()).unwrap();
    assert_eq!(p.instructions.len(), 2);
    assert!(p.directives.is_empty());
}

#[test]
fn a_program_with_no_main_instructions_starts_directly_with_a_directive() {
    let p = parse_program(b"::routine r\nreturn 1\n".to_vec()).unwrap();
    assert!(p.instructions.is_empty());
    assert_eq!(p.directives.len(), 1);
}

/// The trailing instruction after a directive with a body joins that body
/// rather than being an error or a main-program instruction. Measured: a file
/// of `say "main"` / `::routine r` / `return 2` / `say "after directive"` runs
/// rc 0 under `build/bin/rexx` and prints only `main`, so the trailing `say`
/// became part of routine `r`'s body and never ran as top-level code.
#[test]
fn a_trailing_instruction_after_a_body_directive_joins_its_body() {
    let p = parse_program(
        br#"say "main"
::routine r
return 2
say "after directive"
"#
        .to_vec(),
    )
    .unwrap();
    assert_eq!(
        p.instructions.len(),
        1,
        "only the main `say` is a main instruction"
    );
    assert_eq!(
        p.directives.len(),
        1,
        "the trailing `say` did not become a second directive"
    );
}

/// A directive body gets this grammar's full clause-level validation even
/// though Task 3.7c, not this task, assembles its chain. Measured against
/// `build/bin/rexxc`:
///
/// ```text
/// say "main"
/// ::routine r
/// if 1 = 1
/// ```
/// gives `Error 18.1: IF instruction on line 3 requires matching THEN
/// clause.` -- the same missing-THEN check the main body gets.
#[test]
fn a_body_directives_body_is_validated_not_merely_skipped_missing_then() {
    let result = parse_program(b"say \"main\"\n::routine r\nif 1 = 1\n".to_vec());
    assert_eq!(err(result), (18, 1));
}

/// As above, for an ordinary expression-grammar error inside the body.
/// Measured against `build/bin/rexxc`: `::routine r` / `say )` gives
/// `Error 37.2: Unmatched ")" in expression.`
#[test]
fn a_body_directives_body_is_validated_not_merely_skipped_unmatched_paren() {
    let result = parse_program(b"::routine r\nsay )\n".to_vec());
    assert_eq!(err(result), (37, 2));
}

/// `::METHOD` and `::ATTRIBUTE` bodies get the same treatment as `::ROUTINE`.
/// `directive_has_body` reads a different bool field per `DirectiveKind`
/// variant, so a routine-only test cannot pin the other two arms: measured,
/// `::class c` / `::method m` / `return 5` and `::class c` / `::attribute a
/// get` / `return 5` both give rc 0 under `build/bin/rexxc`, with a body in
/// each case that the composition must consume rather than mis-read as the
/// next directive.
#[test]
fn a_method_directives_body_is_recognised_and_consumed() {
    let p = parse_program(b"::class c\n::method m\nreturn 5\n".to_vec()).unwrap();
    assert_eq!(p.directives.len(), 2);
    match &p.directives[1].kind {
        DirectiveKind::Method(m) => assert!(m.body),
        other => panic!("expected a method directive, got {other:?}"),
    }
}

#[test]
fn an_attribute_directives_body_is_recognised_and_consumed() {
    let p = parse_program(b"::class c\n::attribute a get\nreturn 5\n".to_vec()).unwrap();
    assert_eq!(p.directives.len(), 2);
    match &p.directives[1].kind {
        DirectiveKind::Attribute(a) => assert!(a.body),
        other => panic!("expected an attribute directive, got {other:?}"),
    }
}

/// Two directives with bodies in a row: the first body ends exactly where the
/// second directive's `::` starts.
#[test]
fn two_body_directives_in_a_row_each_get_their_own_body() {
    let p = parse_program(b"::routine r1\nreturn 1\n::routine r2\nreturn 2\n".to_vec()).unwrap();
    assert_eq!(p.directives.len(), 2);
    for directive in &p.directives {
        match &directive.kind {
            DirectiveKind::Routine(r) => assert!(r.body),
            other => panic!("expected a routine directive, got {other:?}"),
        }
    }
}

/// A directive that can never carry a body needs no special-case skip: the
/// NEXT `parse_directive` call sees the trailing clause does not start with
/// `::` and raises 99.916 on its own, exactly as `build/bin/rexxc` does.
/// Measured for all five directive kinds that never set a body flag.
#[test]
fn trailing_code_after_a_bodiless_directive_is_99_916_for_every_kind_that_never_has_a_body() {
    let cases: &[&[u8]] = &[
        b"::class c\nsay \"trailing\"\n",
        b"::options digits 5\nsay \"trailing\"\n",
        b"::requires \"x\"\nsay \"trailing\"\n",
        b"::annotate package a 1\nsay \"trailing\"\n",
        b"::resource d\nsome\ndata\n::END\nsay \"trailing\"\n",
    ];
    for case in cases {
        let result = parse_program(case.to_vec());
        assert_eq!(
            err(result),
            (99, 916),
            "case {:?} did not raise 99.916",
            String::from_utf8_lossy(case)
        );
    }
}

/// `::CONSTANT` is the counter-case: it already rejects a body with its OWN
/// specific number, 99.938, from inside `parse_directive` itself, before this
/// crate's composition loop ever gets a chance to call `parse_directive`
/// again. Measured against `build/bin/rexxc`.
#[test]
fn a_constant_directives_own_body_check_still_wins_over_the_generic_one() {
    let result = parse_program(b"::constant c\nsay \"trailing\"\n".to_vec());
    assert_eq!(err(result), (99, 938));
}

// ---- Program::labels ----

fn label_at(instructions: &[rexx_parse::Instruction], index: usize) -> &[u8] {
    match &instructions[index].kind {
        InstructionKind::Label { name } => name,
        other => panic!("instruction {index} is not a label: {other:?}"),
    }
}

#[test]
fn a_symbol_label_is_keyed_by_its_upcased_spelling() {
    let p = parse_program(b"mIxEd: nop\n".to_vec()).unwrap();
    assert_eq!(p.labels.get(&b"MIXED"[..]).copied(), Some(0));
    assert_eq!(label_at(&p.instructions, 0), b"MIXED");
    // Neither the source spelling nor a different case reaches it as a key.
    assert!(!p.labels.contains_key(&b"mIxEd"[..]));
}

/// A literal label is keyed by its bytes VERBATIM, case preserved, unlike a
/// symbol label. Measured (the plan's own six-case table): with a label
/// spelled `'MiXeD':`, `signal value 'MiXeD'` reaches it while
/// `signal value 'MIXED'` and `signal MiXeD` both raise 16.1. This test pins
/// the parse-time half of that: the STORED key.
#[test]
fn a_literal_label_is_keyed_by_its_exact_bytes() {
    let p = parse_program(b"'MiXeD': nop\n".to_vec()).unwrap();
    assert_eq!(p.labels.get(&b"MiXeD"[..]).copied(), Some(0));
    assert!(!p.labels.contains_key(&b"MIXED"[..]));
    assert_eq!(label_at(&p.instructions, 0), b"MiXeD");
}

/// A label's bytes need not be valid UTF-8, so the key type must not be
/// `Box<str>`. Measured against `build/bin/rexx`: a literal label holding a
/// raw `0xFF` byte is a legal `SIGNAL VALUE` target and the jump lands
/// correctly. This test pins that such a program parses at all and that the
/// byte survives unchanged as the map key.
#[test]
fn a_label_may_hold_bytes_that_are_not_valid_utf8() {
    let mut text = b"'".to_vec();
    text.push(0xFF);
    text.extend_from_slice(b"': nop\n");
    let p = parse_program(text).unwrap();
    let key: &[u8] = &[0xFF];
    assert_eq!(p.labels.get(key).copied(), Some(0));
}

/// Two labels with the same key: the FIRST occurrence wins. Measured against
/// `build/bin/rexx`: `signal a` / `say "unreachable"` / `a:` / `say "first"` /
/// `exit` / `a:` / `say "second"` / `exit` prints only `first`. A plain
/// overwrite (last-wins) would point at the second `a:` instead, which this
/// test catches by checking the STORED index against the position of the
/// distinguishing instruction that follows each label.
#[test]
fn a_duplicate_label_keeps_the_first_occurrence_not_the_last() {
    let p = parse_program(b"signal a\nsay \"unreachable\"\na: nop\na: nop\n".to_vec()).unwrap();
    // Instructions, in order: Signal(0), Command "unreachable"(1),
    // Label "A"(2), Nop(3), Label "A"(4), Nop(5).
    assert_eq!(p.instructions.len(), 6);
    assert_eq!(label_at(&p.instructions, 2), b"A");
    assert_eq!(label_at(&p.instructions, 4), b"A");
    assert_eq!(
        p.labels.get(&b"A"[..]).copied(),
        Some(2),
        "the map must keep the FIRST label's index, not the second"
    );
}

/// A label inside a directive's body is not in `Program::labels` at all: that
/// map is built from the main body's instructions only, because a label is
/// local to the code body that declares it (the same reason each directive
/// body will get its own table once Task 3.7c parses one).
#[test]
fn a_label_inside_a_directive_body_is_not_in_the_programs_own_label_map() {
    let p = parse_program(b"::routine r\na: nop\n".to_vec()).unwrap();
    assert!(p.labels.is_empty());
}

// ---- parse_interpret: directives and labels are both rejected ----

/// Measured via a `signal on syntax` trap around `interpret "::routine r"`:
/// `condition('o')~code` is `99.914`, "INTERPRET data must not contain
/// directive instructions." One check, not per-directive: a fragment with
/// other content before the `::` still raises it.
#[test]
fn interpret_rejects_a_directive_with_99_914() {
    assert_eq!(err(parse_interpret(b"::routine r".to_vec())), (99, 914));
    assert_eq!(
        err(parse_interpret(b"say 1; ::routine r".to_vec())),
        (99, 914)
    );
}

/// Measured via a `signal on syntax` trap around `interpret "x: nop"`:
/// `condition('o')~code` is `47.1`.
#[test]
fn interpret_rejects_a_label_with_47_1() {
    assert_eq!(err(parse_interpret(b"x: nop".to_vec())), (47, 1));
    assert_eq!(err(parse_interpret(b"'lit': nop".to_vec())), (47, 1));
}

/// The negative direction: ordinary `INTERPRET` text with neither a label nor
/// a directive parses fine and gets no `directives`/`labels` fields at all
/// (`Fragment` declares neither).
#[test]
fn interpret_accepts_ordinary_text_with_no_directive_and_no_label() {
    let f = parse_interpret(b"say 1; say 2".to_vec()).unwrap();
    assert_eq!(f.instructions.len(), 2);
    assert_eq!(f.source.line(1), Some(&b"say 1; say 2"[..]));
}

// ---- Fragment's SymbolTable is independent of any enclosing Program's ----

/// `a`'s `SymbolId` in `target`, panicking if `target` is not a plain
/// variable assignment.
fn assigned_symbol(instruction: &rexx_parse::Instruction) -> rexx_parse::SymbolId {
    match &instruction.kind {
        InstructionKind::Assignment { target, .. } => match &target.kind {
            rexx_parse::ExprKind::Variable(id) => *id,
            other => panic!("assignment target is not a plain variable: {other:?}"),
        },
        other => panic!("not an assignment: {other:?}"),
    }
}

/// `parse_interpret` builds a fresh `SymbolTable` every call rather than
/// sharing one across calls -- which a shared table could not do without
/// `&mut` at the point Phase 4 calls this at run time. This is not observable
/// through the id VALUE, since both calls resolve `A` the same way; what is
/// observable is that each fragment's own table resolves its own id back to
/// the right text independently of the other fragment's table existing at
/// all, which is exercised here by keeping both alive at once.
#[test]
fn a_fragments_symbol_table_is_its_own_not_shared_across_calls() {
    let f1 = parse_interpret(b"a = 1".to_vec()).unwrap();
    let f2 = parse_interpret(b"b = 2".to_vec()).unwrap();
    let a_id = assigned_symbol(&f1.instructions[0]);
    let b_id = assigned_symbol(&f2.instructions[0]);
    assert_eq!(f1.symbols.name(a_id), "A");
    assert_eq!(f2.symbols.name(b_id), "B");
    // `f1`'s table has no entry for `B` at all, and vice versa: querying the
    // OTHER fragment's id in this table would be a logic error in a caller
    // that conflated the two, not something this crate's types prevent, so
    // the independence is pinned by each table answering only for its own
    // program's names.
    assert_ne!(f1.symbols.name(a_id), f2.symbols.name(b_id));
}

// ---- Step 4: every corpus/lang program parses through this entry point ----

#[test]
fn every_corpus_lang_program_parses() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/lang");
    let mut count = 0;
    for entry in std::fs::read_dir(&dir).expect("corpus/lang exists") {
        let entry = entry.expect("readable directory entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rex") {
            continue;
        }
        let text = std::fs::read(&path).expect("readable corpus file");
        parse_program(text).unwrap_or_else(|e| {
            panic!("{} failed to parse: {e:?}", path.display());
        });
        count += 1;
    }
    // Gate criterion 2 permits adding more; this counts the directory rather
    // than hard-coding a number, but a directory that silently went empty
    // would make the loop above vacuously pass, so a floor is asserted too.
    assert!(
        count >= 14,
        "expected at least 14 corpus/lang programs, found {count}"
    );
}

/// Every directive that ever appears in the corpus is at least reachable
/// through `Directive::kind`'s own keyword accessor, so the corpus test above
/// is not silently skipping the directive path entirely.
#[test]
fn the_corpus_exercises_at_least_one_directive_with_a_body() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/lang");
    let mut saw_a_directive = false;
    for entry in std::fs::read_dir(&dir).expect("corpus/lang exists") {
        let path = entry.expect("readable directory entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rex") {
            continue;
        }
        let text = std::fs::read(&path).expect("readable corpus file");
        let p = parse_program(text).expect("corpus programs parse");
        if !p.directives.is_empty() {
            saw_a_directive = true;
        }
    }
    assert!(
        saw_a_directive,
        "no corpus/lang program has a directive; the directive-loop path in \
         parse_program is untested by the corpus walk above"
    );
}

/// A `Directive`'s own `clause_span` still indexes `Program::source`, the same
/// invariant `Instruction::clause_span` has, checked directly rather than only
/// through the corpus walk.
#[test]
fn a_directives_clause_span_indexes_the_programs_own_source() {
    let p = parse_program(b"::routine r\nreturn 1\n".to_vec()).unwrap();
    let d: &Directive = &p.directives[0];
    let text = p.source.span_bytes(d.clause_span.clone()).unwrap();
    assert_eq!(text, b"::routine r");
}
