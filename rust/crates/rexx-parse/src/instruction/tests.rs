//! The instruction grammar, pinned against `build/bin/rexx` and
//! `build/bin/rexxc`.
//!
//! Every accepted case below was checked with `rexxc`, which is a parse-only
//! oracle, and every rejected case carries the number and sub-number `rexxc`
//! reported. Both directions are tested for every gate: a test that only
//! checks the accepted cases catches one error and misses its opposite.
//!
//! In-crate rather than under `tests/`, because `ParseCtx`, `ClauseCursor` and
//! `parse_instruction` are all `pub(crate)` and an integration test is a
//! separate crate.

use crate::ast::{Instruction, InstructionKind};
use crate::token::{Keywords, ParseCtx, ParseError, SymbolTable};
use crate::{ProgramSource, SourceKind, scan};

use super::{
    KW_ELSE, KW_END, KW_IF, KW_ITERATE, KW_LEAVE, KW_NOP, KW_OTHERWISE, KW_SELECT, KW_THEN,
    KW_WHEN, SUB_CASE, SUB_LABEL, parse_instructions,
};

/// Parses `text` as a whole program and returns every instruction, with the
/// symbol table its ids belong to.
fn parse_kind(text: &str, kind: SourceKind) -> Result<(Vec<Instruction>, SymbolTable), ParseError> {
    let source = ProgramSource::new(text.as_bytes().to_vec(), kind);
    let scanned = scan(&source).expect("the test input scans");
    let result = {
        let ctx = ParseCtx {
            source: &source,
            tokens: &scanned.tokens,
            symbols: &scanned.symbols,
            keywords: &scanned.keywords,
        };
        parse_instructions(&ctx)
    };
    result.map(|instructions| (instructions, scanned.symbols))
}

fn parse(text: &str) -> Result<Vec<Instruction>, ParseError> {
    parse_kind(text, SourceKind::Program).map(|(instructions, _)| instructions)
}

/// The instructions of `text`, which must parse.
fn ok(text: &str) -> Vec<Instruction> {
    parse(text).unwrap_or_else(|e| panic!("{text:?} failed to parse: {e:?}"))
}

/// The error `text` raises, as `(code, sub)`.
fn err(text: &str) -> (u16, u16) {
    match parse(text) {
        Ok(instructions) => panic!(
            "{text:?} parsed into {:?} but an error was expected",
            names(&instructions)
        ),
        Err(e) => (e.code, e.sub),
    }
}

/// The keyword each instruction came from, with the four keyword-less forms
/// spelled out so that a test can name them.
fn names(instructions: &[Instruction]) -> Vec<&'static str> {
    instructions
        .iter()
        .map(|i| {
            i.kind.keyword().unwrap_or_else(|| match &i.kind {
                InstructionKind::Assignment { .. } => "<assign>",
                InstructionKind::Label { .. } => "<label>",
                InstructionKind::Message { .. } => "<message>",
                InstructionKind::Command { .. } => "<command>",
                _ => unreachable!("every keyword form answers `keyword`"),
            })
        })
        .collect()
}

/// The source text of each instruction's `clause_span`, which is what `TRACE`
/// echoes on its `*-*` line.
fn spans(text: &str) -> Vec<&str> {
    let instructions = ok(text);
    instructions
        .iter()
        .map(|i| &text[i.clause_span.clone()])
        .collect()
}

#[test]
fn keyword_indices_still_name_their_own_spellings() {
    // The index constants are positions in a table whose order is load
    // bearing, so each is pinned against the spelling it stands for. A
    // reordering of `INSTRUCTIONS` fails here rather than silently making
    // `IF` parse as `INTERPRET`.
    let mut symbols = SymbolTable::default();
    let keywords = Keywords::new(&mut symbols);
    assert_eq!(
        keywords.instructions.len(),
        35,
        "the keyword instruction table is not 35 entries"
    );
    for (index, spelling) in [
        (KW_ELSE, "ELSE"),
        (KW_END, "END"),
        (KW_IF, "IF"),
        (KW_ITERATE, "ITERATE"),
        (KW_LEAVE, "LEAVE"),
        (KW_NOP, "NOP"),
        (KW_OTHERWISE, "OTHERWISE"),
        (KW_SELECT, "SELECT"),
        (KW_THEN, "THEN"),
        (KW_WHEN, "WHEN"),
    ] {
        assert_eq!(
            keywords.instructions.index_of(symbols.intern(spelling)),
            Some(index),
            "keyword {spelling} is not at index {index}"
        );
    }
    for (index, spelling) in [(SUB_CASE, "CASE"), (SUB_LABEL, "LABEL")] {
        assert_eq!(
            keywords.sub_keywords.index_of(symbols.intern(spelling)),
            Some(index),
            "sub-keyword {spelling} is not at index {index}"
        );
    }
}

// ---- the four clause shapes with no keyword ----

#[test]
fn a_second_token_of_equals_is_an_assignment_whatever_the_first_spells() {
    // Measured: `if = 2; say if` prints 2, so a keyword spelling in the target
    // position is still an assignment.
    assert_eq!(names(&ok("if = 2")), ["<assign>"]);
    assert_eq!(names(&ok("do = 3")), ["<assign>"]);
    assert_eq!(names(&ok("then = 7")), ["<assign>"]);
    assert_eq!(names(&ok("end. = 0")), ["<assign>"]);
    assert_eq!(names(&ok("stem.if = 99")), ["<assign>"]);
    // The shortcut operators are assignments too (`assignmentOpNew`).
    assert_eq!(names(&ok("a += 1")), ["<assign>"]);
    assert_eq!(names(&ok("a ||= 'x'")), ["<assign>"]);
}

#[test]
fn a_keyword_with_no_equals_after_it_is_the_keyword() {
    // The other direction of the same gate: without the `=` these are
    // instructions, not assignments.
    assert_eq!(names(&ok("nop")), ["NOP"]);
    assert_eq!(names(&ok("if 1 then nop")), ["IF", "THEN", "NOP"]);
}

#[test]
fn a_strict_equals_in_the_assignment_position_is_rejected() {
    // `syntaxError(Error_Invalid_expression_general, second)` at
    // `InstructionParser.cpp:185`. Verified with rexxc: `a == 1` is rc 221,
    // Error 35.1.
    assert_eq!(err("a == 1"), (35, 1));
}

#[test]
fn an_assignment_target_must_be_a_variable() {
    // `needVariable` picks its number from the spelling, not the class:
    // verified with rexxc, `1 = 2` is 31.2 and `.a = 2` is 31.3.
    assert_eq!(err("1 = 2"), (31, 2));
    assert_eq!(err(".a = 2"), (31, 3));
    // A stem and a compound both pass, which is the other direction.
    assert_eq!(names(&ok("a. = 2")), ["<assign>"]);
    assert_eq!(names(&ok("a.b = 2")), ["<assign>"]);
}

#[test]
fn an_empty_assignment_right_hand_side_is_35_918() {
    // The sub-number the instruction parser owes `parse_expr`. Verified with
    // rexxc: `r =` is rc 221, Error 35.918.
    assert_eq!(err("r ="), (35, 918));
}

#[test]
fn a_label_clause_is_a_label() {
    assert_eq!(names(&ok("here: nop")), ["<label>", "NOP"]);
    // The colon ends the clause unconditionally, so the `;` belongs to no
    // clause at all. Measured under `trace r`: `here:` then `nop`.
    assert_eq!(spans("here: ; nop"), ["here:", "nop"]);
    // A label may spell a keyword, and the label test runs before any keyword
    // test. Verified with rexxc: `then: nop` is rc 0.
    assert_eq!(names(&ok("then: nop")), ["<label>", "NOP"]);
    assert_eq!(names(&ok("if: nop")), ["<label>", "NOP"]);
}

#[test]
fn a_label_in_interpret_text_is_47_1() {
    // Measured: `interpret "here: nop"` is rc 47 with `found "HERE"`.
    let (code, sub) = match parse_kind("here: nop", SourceKind::Interpret) {
        Ok(_) => panic!("a label in INTERPRET text must be rejected"),
        Err(e) => (e.code, e.sub),
    };
    assert_eq!((code, sub), (47, 1));
    // The other direction: the same text as a program is fine, so the gate is
    // the source kind and not the text.
    assert_eq!(names(&ok("here: nop")), ["<label>", "NOP"]);
}

#[test]
fn a_term_with_a_message_applied_is_a_message_instruction() {
    assert_eq!(names(&ok("q~append(1)")), ["<message>"]);
    assert_eq!(names(&ok("q~~append(1)")), ["<message>"]);
    assert_eq!(names(&ok("q[1] = 2")), ["<message>"]);
    assert_eq!(names(&ok("q[1] += 2")), ["<message>"]);
    assert_eq!(names(&ok("'abc'~length")), ["<message>"]);
}

#[test]
fn a_term_with_no_message_applied_is_a_command() {
    // `parseMessageTerm` returns null unless a `~`, `~~` or `[` was applied,
    // which is why a bare call is a command and not a call.
    assert_eq!(names(&ok("f(1)")), ["<command>"]);
    assert_eq!(names(&ok("'echo hi'")), ["<command>"]);
    assert_eq!(names(&ok("a b c")), ["<command>"]);
    assert_eq!(names(&ok("1 + 2")), ["<command>"]);
}

#[test]
fn a_keyword_survives_the_message_term_trial() {
    // The trial parse must not consume anything when it fails. `if(1)` goes
    // down the general path, parses a call, finds no message applied and is
    // thrown away, after which `if` is still a keyword. Verified with rexxc:
    // `if(1) then nop` is rc 0.
    assert_eq!(names(&ok("if(1) then nop")), ["IF", "THEN", "NOP"]);
}

// ---- control flow, THEN and the one bit of block state ----

#[test]
fn a_bare_then_is_8_1() {
    // `nextInstruction`'s KEYWORD_THEN arm is an unconditional error.
    // Measured: `then` is rc 248, Error 8.1.
    assert_eq!(err("then"), (8, 1));
    // ... and the other direction: after an IF it is the THEN instruction.
    assert_eq!(names(&ok("if 1 then nop")), ["IF", "THEN", "NOP"]);
}

#[test]
fn an_if_with_no_then_is_18_1_and_a_when_is_18_2() {
    // Measured: `if 1 = 1` / `nop` is 18.1, and the same shape under a
    // `SELECT` is 18.2.
    assert_eq!(err("if 1 = 1\nnop"), (18, 1));
    assert_eq!(err("select\nwhen 1 = 1\nnop\nend"), (18, 2));
    // At the end of the body rather than before another clause.
    assert_eq!(err("if 1 = 1"), (18, 1));
}

#[test]
fn an_empty_if_condition_is_35_929() {
    // `parseLogical` raises before `requiredLogicalExpression` can, so the
    // number is 35.929 and not 35.902. Measured, all three: rc 221, 35.929.
    assert_eq!(err("if then nop"), (35, 929));
    assert_eq!(err("if , 1 = 1 then nop"), (35, 929));
    assert_eq!(err("if 1 = 1, then nop"), (35, 929));
    // Measured: a bare `if` is also 35.929.
    assert_eq!(err("if"), (35, 929));
}

#[test]
fn else_and_otherwise_parse_nothing() {
    assert_eq!(
        names(&ok("if 1 then nop\nelse nop")),
        ["IF", "THEN", "NOP", "ELSE", "NOP"]
    );
    assert_eq!(
        names(&ok("select\nwhen 1 then nop\notherwise nop\nend")),
        ["SELECT", "WHEN", "THEN", "NOP", "OTHERWISE", "NOP", "END"]
    );
}

#[test]
fn end_takes_an_optional_name_and_nothing_else() {
    assert_eq!(names(&ok("select\nend")), ["SELECT", "END"]);
    // `isSymbol()` is class-agnostic, so a number is a legal block name as far
    // as the parser is concerned: measured, `select` / `end 1` is Error 10.3, a
    // block-MATCHING error, not a parse error. A literal is not a symbol and
    // is rejected here: `select` / `end "x"` is Error 20.909.
    assert_eq!(names(&ok("select\nend 1")), ["SELECT", "END"]);
    assert_eq!(err("select\nend \"x\""), (20, 909));
    // Anything after the name is 21.909. Measured: `do` / `end a b` is rc 235,
    // Error 21.909.
    assert_eq!(err("select\nend a b"), (21, 909));
}

#[test]
fn nop_takes_nothing() {
    // Measured with rexxc: `nop 1` is rc 232, Error 21.901.
    assert_eq!(err("nop 1"), (21, 901));
    assert_eq!(names(&ok("nop")), ["NOP"]);
}

// ---- rule 4: THEN, ELSE and OTHERWISE end a clause mid-line ----

#[test]
fn a_then_on_the_same_line_makes_three_clauses_with_a_gap() {
    // Measured under `trace r`, and this is the whole reason `split_before`
    // takes two byte positions:
    //     2 *-* if 1 = 1
    //     2 *-*   then
    //     2 *-*     say "a"
    // The condition keeps ALL THREE trailing blanks, `then` carries none on
    // either side despite the four following it, and `say "a"` starts at `say`.
    // The leading blanks on the traced lines are TRACE's own nesting indent,
    // confirmed by `if 1 = 1 then say "a"` printing the same two and four.
    assert_eq!(
        spans("if 1 = 1   then    say \"a\""),
        ["if 1 = 1   ", "then", "say \"a\""]
    );
    // The four blanks between `then` and `say` are in no clause at all.
    let instructions = ok("if 1 = 1   then    say \"a\"");
    assert_eq!(instructions[1].clause_span.end, 15);
    assert_eq!(instructions[2].clause_span.start, 19);
}

#[test]
fn an_if_clause_ends_at_the_start_of_its_terminator_either_way() {
    // Both spellings, because the rule is the START of whatever ended the
    // condition and not "the THEN token". Measured under `trace r`:
    // `if 1 = 1;` with `then` on the next line traces as `if 1 = 1` WITHOUT
    // its semicolon, where `nop;` traces WITH one.
    assert_eq!(
        spans("if 1 = 1;\nthen say \"a\""),
        ["if 1 = 1", "then", "say \"a\""]
    );
    assert_eq!(spans("nop;"), ["nop;"]);
}

#[test]
fn a_then_at_the_end_of_a_line_still_loses_its_trailing_blanks() {
    // The span end moves even when nothing follows on the line, because the
    // THEN takes its location from the keyword token rather than from the
    // clause.
    assert_eq!(spans("if 1 then   \nnop"), ["if 1 ", "then", "nop"]);
}

#[test]
fn else_and_otherwise_trim_the_rest_of_their_line() {
    assert_eq!(
        spans("if 1 then nop\nelse say 2"),
        ["if 1 ", "then", "nop", "else", "say 2"]
    );
    assert_eq!(
        spans("select\nwhen 1 then nop\notherwise say 2\nend"),
        [
            "select",
            "when 1 ",
            "then",
            "nop",
            "otherwise",
            "say 2",
            "end"
        ]
    );
}

#[test]
fn select_takes_an_optional_label_and_an_optional_case() {
    // Verified with rexxc, all three rc 0.
    assert_eq!(
        names(&ok("select label a\nwhen 1 then nop\nend a")),
        ["SELECT", "WHEN", "THEN", "NOP", "END"]
    );
    assert_eq!(
        names(&ok("select case 1\nwhen 1 then nop\nend")),
        ["SELECT", "WHEN", "THEN", "NOP", "END"]
    );
    assert_eq!(
        names(&ok("select label a case 1\nwhen 1 then nop\nend a")),
        ["SELECT", "WHEN", "THEN", "NOP", "END"]
    );
}

#[test]
fn select_rejects_anything_that_is_not_label_or_case() {
    // Measured with rexxc: `select x` is rc 231 Error 25.923, and so is a
    // literal, which cannot be a keyword at all.
    assert_eq!(err("select x\nwhen 1 then nop\nend"), (25, 923));
    assert_eq!(err("select \"x\"\nend"), (25, 923));
    // The name after LABEL must be a symbol. Measured: 20.918 for both a
    // missing name and a literal one.
    assert_eq!(err("select label\nend"), (20, 918));
    assert_eq!(err("select label \"x\"\nend"), (20, 918));
    // CASE's expression is required. Measured: 35.933.
    assert_eq!(err("select case\nend"), (35, 933));
}

#[test]
fn leave_and_iterate_parse_bare() {
    // The Step 4 table's point: measured, `leave` and `iterate` alone are
    // rc 0 under rexxc and only fail at run time, with Error 28.1 and 28.2
    // from `RexxActivation.cpp:1214` and `:1161`. They are NOT wrapped in a
    // loop here, because standing alone is the behaviour under test.
    assert_eq!(names(&ok("leave")), ["LEAVE"]);
    assert_eq!(names(&ok("iterate")), ["ITERATE"]);
    // The named form, which is the only one the oracle accepts at run time.
    // Verified with rexxc: `do label a` / `leave a` / `end a` is rc 0.
    assert_eq!(names(&ok("do label a\nleave a\nend a"))[1], "LEAVE");
}

#[test]
fn leave_and_iterate_name_their_own_errors() {
    // The two keywords differ only in the sub-number, which is why they are
    // separate variants rather than one with a flag. Measured with rexxc:
    // 20.907/21.907 for LEAVE and 20.908/21.908 for ITERATE.
    assert_eq!(err("leave \"x\""), (20, 907));
    assert_eq!(err("leave a b"), (21, 907));
    assert_eq!(err("iterate \"x\""), (20, 908));
    assert_eq!(err("iterate a b"), (21, 908));
}
