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
    COND_ANY, COND_ERROR, COND_FAILURE, COND_HALT, COND_LOSTDIGITS, COND_NOMETHOD, COND_NOSTRING,
    COND_NOTREADY, COND_NOVALUE, COND_PROPAGATE, COND_SYNTAX, COND_USER, KW_ARG, KW_CALL, KW_DO,
    KW_DROP, KW_ELSE, KW_END, KW_EXIT, KW_EXPOSE, KW_FORWARD, KW_GUARD, KW_IF, KW_INTERPRET,
    KW_ITERATE, KW_LEAVE, KW_LOOP, KW_NOP, KW_NUMERIC, KW_OPTIONS, KW_OTHERWISE, KW_PARSE,
    KW_PROCEDURE, KW_PULL, KW_PUSH, KW_QUEUE, KW_RAISE, KW_REPLY, KW_RETURN, KW_SAY, KW_SELECT,
    KW_SIGNAL, KW_THEN, KW_TRACE, KW_USE, KW_WHEN, POPT_ARG, POPT_CASELESS, POPT_LINEIN,
    POPT_LOWER, POPT_PULL, POPT_SOURCE, POPT_UPPER, POPT_VALUE, POPT_VAR, POPT_VERSION,
    SUB_ADDITIONAL, SUB_ARG, SUB_ARGUMENTS, SUB_ARRAY, SUB_BY, SUB_CASE, SUB_CLASS, SUB_CONTINUE,
    SUB_COUNTER, SUB_DESCRIPTION, SUB_DIGITS, SUB_ENGINEERING, SUB_EXIT, SUB_EXPOSE, SUB_FOR,
    SUB_FOREVER, SUB_FORM, SUB_FUZZ, SUB_INDEX, SUB_ITEM, SUB_LABEL, SUB_LOCAL, SUB_MESSAGE,
    SUB_NAME, SUB_OFF, SUB_ON, SUB_OVER, SUB_RETURN, SUB_SCIENTIFIC, SUB_STRICT, SUB_TO, SUB_UNTIL,
    SUB_VALUE, SUB_WHEN, SUB_WHILE, SUB_WITH, parse_instructions,
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
        (KW_ARG, "ARG"),
        (KW_CALL, "CALL"),
        (KW_DO, "DO"),
        (KW_DROP, "DROP"),
        (KW_ELSE, "ELSE"),
        (KW_END, "END"),
        (KW_EXIT, "EXIT"),
        (KW_EXPOSE, "EXPOSE"),
        (KW_FORWARD, "FORWARD"),
        (KW_GUARD, "GUARD"),
        (KW_IF, "IF"),
        (KW_INTERPRET, "INTERPRET"),
        (KW_ITERATE, "ITERATE"),
        (KW_LEAVE, "LEAVE"),
        (KW_LOOP, "LOOP"),
        (KW_NOP, "NOP"),
        (KW_NUMERIC, "NUMERIC"),
        (KW_OPTIONS, "OPTIONS"),
        (KW_OTHERWISE, "OTHERWISE"),
        (KW_PARSE, "PARSE"),
        (KW_PROCEDURE, "PROCEDURE"),
        (KW_PULL, "PULL"),
        (KW_PUSH, "PUSH"),
        (KW_QUEUE, "QUEUE"),
        (KW_RAISE, "RAISE"),
        (KW_REPLY, "REPLY"),
        (KW_RETURN, "RETURN"),
        (KW_SAY, "SAY"),
        (KW_SELECT, "SELECT"),
        (KW_SIGNAL, "SIGNAL"),
        (KW_THEN, "THEN"),
        (KW_TRACE, "TRACE"),
        (KW_USE, "USE"),
        (KW_WHEN, "WHEN"),
    ] {
        assert_eq!(
            keywords.instructions.index_of(symbols.intern(spelling)),
            Some(index),
            "keyword {spelling} is not at index {index}"
        );
    }
    for (index, spelling) in [
        (SUB_ADDITIONAL, "ADDITIONAL"),
        (SUB_ARG, "ARG"),
        (SUB_ARGUMENTS, "ARGUMENTS"),
        (SUB_ARRAY, "ARRAY"),
        (SUB_BY, "BY"),
        (SUB_CASE, "CASE"),
        (SUB_CLASS, "CLASS"),
        (SUB_CONTINUE, "CONTINUE"),
        (SUB_COUNTER, "COUNTER"),
        (SUB_DESCRIPTION, "DESCRIPTION"),
        (SUB_DIGITS, "DIGITS"),
        (SUB_ENGINEERING, "ENGINEERING"),
        (SUB_EXIT, "EXIT"),
        (SUB_EXPOSE, "EXPOSE"),
        (SUB_FOR, "FOR"),
        (SUB_FOREVER, "FOREVER"),
        (SUB_FORM, "FORM"),
        (SUB_FUZZ, "FUZZ"),
        (SUB_INDEX, "INDEX"),
        (SUB_ITEM, "ITEM"),
        (SUB_LABEL, "LABEL"),
        (SUB_LOCAL, "LOCAL"),
        (SUB_MESSAGE, "MESSAGE"),
        (SUB_NAME, "NAME"),
        (SUB_OFF, "OFF"),
        (SUB_ON, "ON"),
        (SUB_OVER, "OVER"),
        (SUB_RETURN, "RETURN"),
        (SUB_SCIENTIFIC, "SCIENTIFIC"),
        (SUB_STRICT, "STRICT"),
        (SUB_TO, "TO"),
        (SUB_UNTIL, "UNTIL"),
        (SUB_VALUE, "VALUE"),
        (SUB_WHEN, "WHEN"),
        (SUB_WHILE, "WHILE"),
        (SUB_WITH, "WITH"),
    ] {
        assert_eq!(
            keywords.sub_keywords.index_of(symbols.intern(spelling)),
            Some(index),
            "sub-keyword {spelling} is not at index {index}"
        );
    }
    // The parse options and the condition names are tables of their own.
    // `ARG`, `PULL` and `VALUE` sit in both the sub-keyword and the
    // parse-option table at different indices, so conflating the two would
    // silently make `PARSE VALUE` mean something else.
    for (index, spelling) in [
        (POPT_ARG, "ARG"),
        (POPT_CASELESS, "CASELESS"),
        (POPT_LINEIN, "LINEIN"),
        (POPT_LOWER, "LOWER"),
        (POPT_PULL, "PULL"),
        (POPT_SOURCE, "SOURCE"),
        (POPT_UPPER, "UPPER"),
        (POPT_VALUE, "VALUE"),
        (POPT_VAR, "VAR"),
        (POPT_VERSION, "VERSION"),
    ] {
        assert_eq!(
            keywords.parse_options.index_of(symbols.intern(spelling)),
            Some(index),
            "parse option {spelling} is not at index {index}"
        );
    }
    for (index, spelling) in [
        (COND_ANY, "ANY"),
        (COND_ERROR, "ERROR"),
        (COND_FAILURE, "FAILURE"),
        (COND_HALT, "HALT"),
        (COND_LOSTDIGITS, "LOSTDIGITS"),
        (COND_NOMETHOD, "NOMETHOD"),
        (COND_NOSTRING, "NOSTRING"),
        (COND_NOTREADY, "NOTREADY"),
        (COND_NOVALUE, "NOVALUE"),
        (COND_PROPAGATE, "PROPAGATE"),
        (COND_SYNTAX, "SYNTAX"),
        (COND_USER, "USER"),
    ] {
        assert_eq!(
            keywords.conditions.index_of(symbols.intern(spelling)),
            Some(index),
            "condition {spelling} is not at index {index}"
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

// ---- DO and LOOP ----

/// The loop header of the first instruction of `text`, which must be a `DO` or
/// a `LOOP`.
fn loop_of(text: &str) -> crate::ast::Loop {
    match &ok(text)[0].kind {
        InstructionKind::Do(body) | InstructionKind::Loop(body) => (**body).clone(),
        other => panic!("{text:?} is not a loop: {other:?}"),
    }
}

/// A one-word name for a loop's shape, so a test can say which of the six the
/// parse produced without spelling out its expressions.
fn loop_shape(text: &str) -> String {
    let body = loop_of(text);
    let kind = match &body.kind {
        crate::ast::LoopKind::Simple => "simple".to_string(),
        crate::ast::LoopKind::Forever => "forever".to_string(),
        crate::ast::LoopKind::Count(None) => "count(-)".to_string(),
        crate::ast::LoopKind::Count(Some(_)) => "count".to_string(),
        crate::ast::LoopKind::Controlled(control) => {
            let mut parts = String::new();
            for entry in &control.order {
                parts.push(match entry {
                    crate::ast::ControlExpr::To => 't',
                    crate::ast::ControlExpr::By => 'b',
                    crate::ast::ControlExpr::For => 'f',
                });
            }
            format!("controlled[{parts}]")
        }
        crate::ast::LoopKind::Over { for_count, .. } => {
            format!("over{}", if for_count.is_some() { "+for" } else { "" })
        }
        crate::ast::LoopKind::With {
            index,
            item,
            for_count,
            ..
        } => format!(
            "with[{}{}]{}",
            if index.is_some() { "i" } else { "" },
            if item.is_some() { "v" } else { "" },
            if for_count.is_some() { "+for" } else { "" }
        ),
    };
    let tail = match &body.conditional {
        None => "",
        Some(c) if c.until => "+until",
        Some(_) => "+while",
    };
    let label = if body.label.is_some() { "@" } else { "" };
    let counter = if body.counter.is_some() { "#" } else { "" };
    format!("{label}{counter}{kind}{tail}")
}

#[test]
fn bare_do_is_a_block_and_bare_loop_is_forever() {
    // The one difference between the two keywords. Both rc 0 under rexxc.
    assert_eq!(loop_shape("do\nend"), "simple");
    assert_eq!(loop_shape("loop\nend"), "forever");
    assert_eq!(names(&ok("do\nend")), ["DO", "END"]);
    assert_eq!(names(&ok("loop\nend")), ["LOOP", "END"]);
}

#[test]
fn every_loop_form_reaches_its_own_shape() {
    // Each of these was checked with rexxc, all rc 0.
    assert_eq!(loop_shape("do 3\nend"), "count");
    assert_eq!(loop_shape("do forever\nend"), "forever");
    assert_eq!(loop_shape("do while 1\nend"), "forever+while");
    assert_eq!(loop_shape("do until 1\nend"), "forever+until");
    // A controlled loop takes the control variable's name as its label when no
    // LABEL clause gave one, which is what `LEAVE i` matches.
    assert_eq!(loop_shape("do i = 1 to 3\nend"), "@controlled[t]");
    assert_eq!(
        loop_shape("do i = 1 to 3 by 2 for 4\nend"),
        "@controlled[tbf]"
    );
    assert_eq!(loop_shape("do i over x\nend"), "@over");
    assert_eq!(
        loop_shape("do i over x for 2 while 1\nend"),
        "@over+for+while"
    );
    assert_eq!(loop_shape("do with index i over x\nend"), "with[i]");
    assert_eq!(loop_shape("do with item v over x\nend"), "with[v]");
    assert_eq!(
        loop_shape("do with index i item v over x for 2\nend"),
        "with[iv]+for"
    );
    assert_eq!(loop_shape("do label a\nend a"), "@simple");
    assert_eq!(loop_shape("loop counter c\nend"), "#forever");
    assert_eq!(loop_shape("do counter c 3\nend"), "#count");
    assert_eq!(loop_shape("do label a counter c 3\nend a"), "@#count");
}

#[test]
fn the_order_of_a_controlled_loops_keywords_is_the_order_written() {
    // Evaluation order is observable, because a control expression can have
    // side effects, so it is recorded rather than normalised.
    assert_eq!(loop_shape("do i = 1 by 2 to 3\nend"), "@controlled[bt]");
    assert_eq!(
        loop_shape("do i = 1 for 4 to 3 by 2\nend"),
        "@controlled[ftb]"
    );
}

#[test]
fn over_is_tested_before_with_so_a_variable_may_be_named_with() {
    // `do with over x` is a DO OVER whose control variable is WITH, because
    // `createLoop` tests `second == OVER` before it tests `first == WITH`.
    // Measured: rc 0 under rexxc, and the shape is `over`, not `with`.
    assert_eq!(loop_shape("do with over x\nend"), "@over");
}

#[test]
fn a_loop_control_variable_must_be_a_variable() {
    // `addVariable` calls `needVariable`, so the number comes from the
    // SPELLING. Measured: `do 1 = 1 to 2` is 31.2 and `do .5 = 1 to 2` is
    // 31.3, where `drop .5` is 31.2 from the class-based test instead.
    assert_eq!(err("do 1 = 1 to 2\nend"), (31, 2));
    assert_eq!(err("do .5 = 1 to 2\nend"), (31, 3));
    assert_eq!(err("do .a = 1 to 2\nend"), (31, 3));
    assert_eq!(err("do 1 over x\nend"), (31, 2));
    assert_eq!(err("do with index 1 over x\nend"), (31, 2));
    assert_eq!(err("do with index .a over x\nend"), (31, 3));
    // The other direction: a stem and a compound are legal control variables.
    assert_eq!(loop_shape("do a. = 1 to 2\nend"), "@controlled[t]");
    assert_eq!(loop_shape("do a.b = 1 to 2\nend"), "@controlled[t]");
}

#[test]
fn a_duplicated_loop_keyword_is_27_902() {
    // Measured with rexxc: rc 229, Error 27.902.
    assert_eq!(err("do i = 1 to 3 to 4\nend"), (27, 902));
    assert_eq!(err("do i = 1 by 2 by 3\nend"), (27, 902));
    assert_eq!(err("do i over x for 1 for 2\nend"), (27, 902));
    // The other direction: one of each is fine.
    assert_eq!(loop_shape("do i = 1 to 3 by 2\nend"), "@controlled[tb]");
}

#[test]
fn a_loop_takes_at_most_one_conditional() {
    // `Error_Invalid_do_whileuntil`. Measured:
    // `do i = 1 to 3 while 1 until 2` is rc 229, Error 27.1.
    assert_eq!(err("do i = 1 to 3 while 1 until 2\nend"), (27, 1));
    assert_eq!(err("do while 1 until 2\nend"), (27, 1));
    // The other direction, and it is not obvious: `while 1 x` is ACCEPTED,
    // because the conditional absorbs `1 x` as a blank concatenation before
    // the end-of-clause check runs. Measured: rc 0.
    assert_eq!(
        loop_shape("do i = 1 to 3 while 1 x\nend"),
        "@controlled[t]+while"
    );
}

#[test]
fn do_forever_allows_only_a_conditional_after_it() {
    // The one reachable use of `parseLoopConditional`'s error argument.
    // Measured: `do forever x` is rc 229, Error 27.901.
    assert_eq!(err("do forever x\nend"), (27, 901));
    assert_eq!(loop_shape("do forever while 1\nend"), "forever+while");
}

#[test]
fn do_with_requires_over_after_its_variables() {
    // Measured: `do with index i x` is rc 229, Error 27.904.
    assert_eq!(err("do with index i x\nend"), (27, 904));
    assert_eq!(err("do with item v item w over x\nend"), (27, 902));
}

#[test]
fn a_simple_do_may_not_have_a_counter() {
    // Measured: `do counter c` is rc 229, Error 27.905, while
    // `loop counter c` is rc 0 because a LOOP always iterates.
    assert_eq!(err("do counter c\nend"), (27, 905));
    assert_eq!(loop_shape("loop counter c\nend"), "#forever");
}

#[test]
fn label_and_counter_names_must_be_symbols() {
    // Measured: `do label` is 20.918 and `do counter "x" 3` is 20.934.
    assert_eq!(err("do label\nend"), (20, 918));
    assert_eq!(err("do counter \"x\" 3\nend"), (20, 934));
}

#[test]
fn a_repeated_label_keyword_becomes_the_count_expression() {
    // `do label a label b` is accepted, because the second LABEL ends the
    // option loop and becomes `DO expr` with the expression `label b`.
    // Measured: rc 0 under rexxc.
    assert_eq!(loop_shape("do label a label b\nend"), "@count");
}

// ---- the data family: DROP, EXPOSE, SAY, PUSH, QUEUE ----

/// The names in the first instruction's variable list, `(name)` marked with
/// parentheses so the two forms cannot be confused.
fn variable_list(text: &str) -> Vec<String> {
    let (instructions, symbols) =
        parse_kind(text, SourceKind::Program).unwrap_or_else(|e| panic!("{text:?}: {e:?}"));
    let list = match &instructions[0].kind {
        InstructionKind::Drop { variables }
        | InstructionKind::Expose { variables }
        | InstructionKind::Procedure { variables } => variables,
        other => panic!("{text:?} has no variable list: {other:?}"),
    };
    list.iter()
        .map(|v| match v {
            crate::ast::VariableRef::Direct(id) => symbols.name(*id).to_string(),
            crate::ast::VariableRef::Indirect(id) => format!("({})", symbols.name(*id)),
        })
        .collect()
}

#[test]
fn say_push_and_queue_take_an_optional_expression() {
    // Measured with rexxc, all rc 0.
    assert_eq!(names(&ok("say")), ["SAY"]);
    assert_eq!(names(&ok("say 1 2")), ["SAY"]);
    assert_eq!(names(&ok("push")), ["PUSH"]);
    assert_eq!(names(&ok("push 1")), ["PUSH"]);
    assert_eq!(names(&ok("queue")), ["QUEUE"]);
    // The trial parse must not eat the keyword: `say(1)` parses a call, finds
    // no message applied and is discarded. Measured: rc 0.
    assert_eq!(names(&ok("say(1)")), ["SAY"]);
    // The other direction of that gate: with a message applied it IS a message
    // instruction, keyword spelling or not.
    assert_eq!(names(&ok("say~length")), ["<message>"]);
}

#[test]
fn drop_and_expose_take_both_variable_spellings() {
    // Measured with rexxc, all rc 0.
    assert_eq!(variable_list("drop a b c."), ["A", "B", "C."]);
    assert_eq!(variable_list("drop (v)"), ["(V)"]);
    assert_eq!(variable_list("drop a (v) b"), ["A", "(V)", "B"]);
    assert_eq!(variable_list("expose a b"), ["A", "B"]);
}

#[test]
fn an_empty_variable_list_names_its_own_instruction() {
    // Measured: `drop` is 20.901 and `expose` inside a method is 20.902, the
    // two sub-numbers being the only difference between the two instructions.
    assert_eq!(err("drop"), (20, 901));
    assert_eq!(err("drop \"x\""), (20, 901));
    assert_eq!(err("expose"), (20, 902));
    assert_eq!(err("expose \"x\""), (20, 902));
}

#[test]
fn the_two_variable_gates_disagree_and_both_are_reproduced() {
    // The direct form tests the symbol's CLASS and the `(name)` form tests its
    // SPELLING, so a constant beginning with a period gets a different number
    // in each. Measured: `drop .5` is 31.2, `drop (.5)` is 31.3.
    assert_eq!(err("drop .5"), (31, 2));
    assert_eq!(err("drop (.5)"), (31, 3));
    // The rest of both gates, measured: a plain number is 31.2 either way and
    // a dot symbol is 31.3 either way.
    assert_eq!(err("drop 5"), (31, 2));
    assert_eq!(err("drop (1)"), (31, 2));
    assert_eq!(err("drop ."), (31, 3));
    assert_eq!(err("drop (.a)"), (31, 3));
}

#[test]
fn an_indirect_variable_reference_must_be_closed() {
    // Measured: `drop (` is 20.906, `drop (v` is 46.901 and `drop (v x` is
    // 46.1.
    assert_eq!(err("drop ("), (20, 906));
    assert_eq!(err("drop (v"), (46, 901));
    assert_eq!(err("drop (v x"), (46, 1));
}

#[test]
fn expose_is_rejected_inside_interpret() {
    // `exposeNew` calls `isInterpret()` first. Measured at RUN time, because
    // rexxc never parses an INTERPRET string: `interpret "expose a"` is
    // rc 157, Error 99.908.
    let (code, sub) = match parse_kind("expose a", SourceKind::Interpret) {
        Ok(_) => panic!("EXPOSE inside INTERPRET must be rejected"),
        Err(e) => (e.code, e.sub),
    };
    assert_eq!((code, sub), (99, 908));
    // The other direction: the same text as a program is accepted, so the gate
    // is the source kind. Measured: `expose a` alone is rc 0 under rexxc.
    assert_eq!(names(&ok("expose a")), ["EXPOSE"]);
}

// ---- PARSE, ARG and PULL, and the template grammar ----

/// A one-line rendering of a `PARSE` instruction: its source, its options and
/// its template, so a test can pin the whole shape.
fn parse_shape(text: &str) -> String {
    let (instructions, symbols) =
        parse_kind(text, SourceKind::Program).unwrap_or_else(|e| panic!("{text:?}: {e:?}"));
    let body = match &instructions[0].kind {
        InstructionKind::Parse(body) | InstructionKind::Arg(body) | InstructionKind::Pull(body) => {
            body
        }
        other => panic!("{text:?} is not a PARSE: {other:?}"),
    };
    let source = match &body.source {
        crate::ast::ParseSource::Arg => "arg".to_string(),
        crate::ast::ParseSource::LineIn => "linein".to_string(),
        crate::ast::ParseSource::Pull => "pull".to_string(),
        crate::ast::ParseSource::Source => "source".to_string(),
        crate::ast::ParseSource::Version => "version".to_string(),
        crate::ast::ParseSource::Var(id) => format!("var:{}", symbols.name(*id)),
        crate::ast::ParseSource::Value(None) => "value:-".to_string(),
        crate::ast::ParseSource::Value(Some(e)) => format!("value:{}", e.shape(&symbols)),
    };
    let mut flags = String::new();
    if body.upper {
        flags.push('U');
    }
    if body.lower {
        flags.push('L');
    }
    if body.caseless {
        flags.push('C');
    }
    let mut out = format!("{source}[{flags}]");
    for entry in &body.template {
        let Some(trigger) = entry else {
            out.push_str(" |");
            continue;
        };
        let kind = match trigger.kind {
            crate::ast::TriggerKind::End => "end",
            crate::ast::TriggerKind::Plus => "+",
            crate::ast::TriggerKind::Minus => "-",
            crate::ast::TriggerKind::Absolute => "=",
            crate::ast::TriggerKind::MinusLength => "<",
            crate::ast::TriggerKind::PlusLength => ">",
            crate::ast::TriggerKind::String => "str",
            crate::ast::TriggerKind::Mixed => "mix",
        };
        out.push_str(&format!(" {kind}"));
        if let Some(value) = &trigger.value {
            out.push_str(&format!("({})", value.shape(&symbols)));
        }
        for target in &trigger.targets {
            match target {
                Some(e) => out.push_str(&format!(" ->{}", e.shape(&symbols))),
                None => out.push_str(" ->."),
            }
        }
    }
    out
}

#[test]
fn every_parse_source_reaches_its_own_variant() {
    // Measured with rexxc, all rc 0.
    assert_eq!(parse_shape("parse arg a"), "arg[] end ->A");
    assert_eq!(parse_shape("parse pull a"), "pull[] end ->A");
    assert_eq!(parse_shape("parse linein a"), "linein[] end ->A");
    assert_eq!(parse_shape("parse source a"), "source[] end ->A");
    assert_eq!(parse_shape("parse version a"), "version[] end ->A");
    assert_eq!(parse_shape("parse var v a b"), "var:V[] end ->A ->B");
    assert_eq!(
        parse_shape("parse value \"a b\" with a b"),
        "value:\"a b\"[] end ->A ->B"
    );
    // The VALUE expression is optional and defaults to the null string.
    // Measured: `parse value with a` is rc 0.
    assert_eq!(parse_shape("parse value with a"), "value:-[] end ->A");
    // The short forms imply UPPER and take no options.
    assert_eq!(parse_shape("arg a"), "arg[U] end ->A");
    assert_eq!(parse_shape("pull a"), "pull[U] end ->A");
}

#[test]
fn the_parse_options_are_recognised_once_each() {
    // Measured: rc 0 for each of these.
    assert_eq!(parse_shape("parse upper arg a"), "arg[U] end ->A");
    assert_eq!(parse_shape("parse lower caseless arg a"), "arg[LC] end ->A");
    // A repeated option is not an option any more, it is an unknown source.
    // Measured: `parse upper upper arg a` and `parse caseless caseless arg a`
    // are both rc 231, Error 25.12.
    assert_eq!(err("parse upper upper arg a"), (25, 12));
    assert_eq!(err("parse caseless caseless arg a"), (25, 12));
    // UPPER and LOWER exclude each other for the same reason.
    assert_eq!(err("parse upper lower arg a"), (25, 12));
}

#[test]
fn parse_rejects_an_unknown_or_missing_source() {
    // Measured: `parse foo a` is 25.12, `parse "x" a` and bare `parse` are
    // both 20.903.
    assert_eq!(err("parse foo a"), (25, 12));
    assert_eq!(err("parse \"x\" a"), (20, 903));
    assert_eq!(err("parse"), (20, 903));
    // PARSE VAR needs a variable. Measured: `parse var` is 20.904 and
    // `parse var 1 a` is 31.2.
    assert_eq!(err("parse var"), (20, 904));
    assert_eq!(err("parse var 1 a"), (31, 2));
    // PARSE VALUE needs its WITH. Measured: `parse value "x" a` is 38.3.
    assert_eq!(err("parse value \"x\" a"), (38, 3));
}

#[test]
fn every_template_trigger_reaches_its_own_kind() {
    // Measured with rexxc, all rc 0.
    // The variables BEFORE a trigger are the ones it assigns, which is why
    // `+(3)` carries A and B and the trailing END trigger carries only C.
    assert_eq!(
        parse_shape("parse arg a b +3 c"),
        "arg[] +(3) ->A ->B end ->C"
    );
    assert_eq!(parse_shape("parse arg 1 a"), "arg[] =(1) end ->A");
    assert_eq!(parse_shape("parse arg =5 a"), "arg[] =(5) end ->A");
    assert_eq!(parse_shape("parse arg -2 a"), "arg[] -(2) end ->A");
    assert_eq!(parse_shape("parse arg <2 a"), "arg[] <(2) end ->A");
    assert_eq!(parse_shape("parse arg >2 a"), "arg[] >(2) end ->A");
    assert_eq!(parse_shape("parse arg +(x) a"), "arg[] +(X) end ->A");
    assert_eq!(
        parse_shape("parse arg \"lit\" a"),
        "arg[] str(\"lit\") end ->A"
    );
    assert_eq!(parse_shape("parse arg (e) a"), "arg[] str(E) end ->A");
    // CASELESS switches the string triggers to the mixed-case comparison.
    assert_eq!(
        parse_shape("parse caseless arg \"lit\" a"),
        "arg[C] mix(\"lit\") end ->A"
    );
    // A lone period consumes a field and assigns nothing.
    assert_eq!(parse_shape("parse arg . a"), "arg[] end ->. ->A");
    // A comma starts the next parse string, and shows up as its own entry.
    assert_eq!(parse_shape("parse arg a, b"), "arg[] end ->A | end ->B");
    // A target may be a compound variable or a message term. Measured, both
    // rc 0.
    assert_eq!(
        parse_shape("parse arg a.b"),
        "arg[] end ->compound:A.[var:B]"
    );
    assert_eq!(parse_shape("parse arg q~x"), "arg[] end ->(msg~ Q \"X\")");
}

#[test]
fn a_template_with_no_variables_after_its_last_trigger_gets_no_end_trigger() {
    // `variableCount > 0` gates the trailing END trigger, so a template ending
    // in a trigger has nothing after it. Measured: rc 0.
    assert_eq!(parse_shape("parse arg a +3"), "arg[] +(3) ->A");
    assert_eq!(parse_shape("parse arg +3"), "arg[] +(3)");
}

#[test]
fn a_trigger_position_may_not_be_a_variable() {
    // The gate `parseNew` applies with `token->isVariable()`. Measured:
    // `parse arg +x a` is 38.2 and `parse arg +"x"` is 38.2, while
    // `parse arg +(x) a` is rc 0, which is the other direction.
    assert_eq!(err("parse arg +x a"), (38, 2));
    assert_eq!(err("parse arg +a. a"), (38, 2));
    assert_eq!(err("parse arg +\"x\""), (38, 2));
    // Missing entirely. Measured: `parse arg +` is 38.901.
    assert_eq!(err("parse arg +"), (38, 901));
    // An empty parenthesised position. Measured: `parse arg +()` is 35.931.
    assert_eq!(err("parse arg +()"), (35, 931));
}

#[test]
fn an_unusable_template_token_is_38_1() {
    // Measured: `parse arg *3` is rc 218, Error 38.1.
    assert_eq!(err("parse arg *3"), (38, 1));
    // A dot symbol is not a variable, so it fails the target gate rather than
    // the trigger one. Measured: `parse arg .a` is 31.3.
    assert_eq!(err("parse arg .a"), (31, 3));
}

// ---- CALL and SIGNAL, including the condition traps ----

/// A rendering of the first instruction when it is a `CALL` or a `SIGNAL`.
fn call_shape(text: &str) -> String {
    let (instructions, symbols) =
        parse_kind(text, SourceKind::Program).unwrap_or_else(|e| panic!("{text:?}: {e:?}"));
    let bytes = |b: &[u8]| String::from_utf8_lossy(b).to_string();
    let trap = |t: &crate::ast::ConditionTrap| {
        format!(
            "{} {}{}",
            if t.on { "on" } else { "off" },
            bytes(&t.condition),
            match &t.label {
                Some(l) => format!(" ->{}", bytes(l)),
                None => String::new(),
            }
        )
    };
    match &instructions[0].kind {
        InstructionKind::Call(call) => match &**call {
            crate::ast::Call::Named {
                name,
                literal,
                args,
            } => format!(
                "call{} {}({})",
                if *literal { "-lit" } else { "" },
                bytes(name),
                args.len()
            ),
            crate::ast::Call::Dynamic { target, args } => {
                format!("call-dyn {}({})", target.shape(&symbols), args.len())
            }
            crate::ast::Call::Qualified {
                namespace,
                name,
                args,
            } => format!(
                "call-ns {}:{}({})",
                symbols.name(*namespace),
                symbols.name(*name),
                args.len()
            ),
            crate::ast::Call::Trap(t) => format!("call-trap {}", trap(t)),
        },
        InstructionKind::Signal(signal) => match &**signal {
            crate::ast::Signal::Label(name) => format!("signal {}", bytes(name)),
            crate::ast::Signal::Value(e) => format!("signal-value {}", e.shape(&symbols)),
            crate::ast::Signal::Trap(t) => format!("signal-trap {}", trap(t)),
        },
        other => panic!("{text:?} is not a CALL or SIGNAL: {other:?}"),
    }
}

#[test]
fn every_call_form_reaches_its_own_variant() {
    // Measured with rexxc, all rc 0.
    assert_eq!(call_shape("call f"), "call F(0)");
    assert_eq!(call_shape("call f 1, 2"), "call F(2)");
    // A literal target keeps its case and never resolves to an internal label.
    assert_eq!(call_shape("call \"f\" 1"), "call-lit f(1)");
    assert_eq!(call_shape("call (e) 1"), "call-dyn E(1)");
    assert_eq!(call_shape("call ns:name 1"), "call-ns NS:NAME(1)");
    // A number is a symbol, so it is a legal call target name. Measured: rc 0.
    assert_eq!(call_shape("call 1"), "call 1(0)");
}

#[test]
fn call_rejects_a_target_that_is_neither_symbol_literal_nor_paren() {
    // `Error_Symbol_or_string_call`. Measured: bare `call` is rc 237, 19.2.
    assert_eq!(err("call"), (19, 2));
    // Measured: `call +1` and `call ,1` are both rc 237, Error 19.2.
    assert_eq!(err("call +1"), (19, 2));
    assert_eq!(err("call ,1"), (19, 2));
    // But `call ~x` is rc 0 and is NOT a CALL at all: the message-term test
    // runs before keyword dispatch, so this is the message `CALL~X`.
    assert_eq!(names(&ok("call ~x")), ["<message>"]);
    // `Error_Symbol_expected_qualified_call`, measured as 20.922.
    assert_eq!(err("call ns:"), (20, 922));
}

#[test]
fn call_on_accepts_fewer_conditions_than_signal_on() {
    // The difference is the whole reason the two share one function with a
    // flag. Measured with rexxc: `call on syntax` and `call on novalue` are
    // rc 231 Error 25.1, while `signal on syntax` is rc 0.
    assert_eq!(err("call on syntax"), (25, 1));
    assert_eq!(err("call on novalue"), (25, 1));
    assert_eq!(err("call on lostdigits"), (25, 1));
    assert_eq!(err("call on nomethod"), (25, 1));
    assert_eq!(err("call on nostring"), (25, 1));
    assert_eq!(
        call_shape("signal on syntax\nsyntax: nop"),
        "signal-trap on SYNTAX ->SYNTAX"
    );
    assert_eq!(
        call_shape("signal on novalue\nnovalue: nop"),
        "signal-trap on NOVALUE ->NOVALUE"
    );
    // Both accept ANY, ERROR, FAILURE, HALT and NOTREADY. Measured: rc 0.
    assert_eq!(
        call_shape("call on any\nany: return"),
        "call-trap on ANY ->ANY"
    );
    assert_eq!(
        call_shape("call on error\nerror: return"),
        "call-trap on ERROR ->ERROR"
    );
    // Neither accepts PROPAGATE. Measured: `signal on propagate` is 25.3.
    assert_eq!(err("signal on propagate"), (25, 3));
    assert_eq!(err("call on propagate"), (25, 1));
    assert_eq!(err("call on foo"), (25, 1));
}

#[test]
fn a_user_condition_carries_its_composed_name() {
    // `USER name` is the condition's own name, built by
    // `concatToCstring("USER ")`. Measured: rc 0.
    assert_eq!(
        call_shape("call on user x\nx: return"),
        "call-trap on USER X ->X"
    );
    // `Error_Symbol_expected_user`, measured as 20.915.
    assert_eq!(err("call on user"), (20, 915));
    assert_eq!(err("call on user \"x\""), (20, 915));
}

#[test]
fn the_on_form_takes_a_name_override_and_the_off_form_takes_nothing() {
    // Measured: rc 0.
    assert_eq!(
        call_shape("call on error name lab\nlab: return"),
        "call-trap on ERROR ->LAB"
    );
    assert_eq!(call_shape("call off error"), "call-trap off ERROR");
    // Measured, each of these: `call on error name` is 19.3,
    // `call on error name lab x` is 21.903, `call off error x` is 21.904,
    // and the sub-keyword must be NAME.
    assert_eq!(err("call on error name"), (19, 3));
    assert_eq!(err("call on error name lab x"), (21, 903));
    assert_eq!(err("call off error x"), (21, 904));
    assert_eq!(err("call on error label lab"), (25, 914));
    // SIGNAL's own sub-keyword error is a different number.
    assert_eq!(err("signal on error label lab"), (25, 915));
    assert_eq!(err("signal on error name lab x"), (21, 903));
    // Missing the ON/OFF condition entirely.
    assert_eq!(err("call on"), (20, 911));
    assert_eq!(err("call off"), (20, 912));
}

#[test]
fn every_signal_form_reaches_its_own_variant() {
    // Measured with rexxc, all rc 0.
    assert_eq!(call_shape("signal lab\nlab: nop"), "signal LAB");
    assert_eq!(call_shape("signal \"lab\"\nlab: nop"), "signal lab");
    assert_eq!(call_shape("signal value 1"), "signal-value 1");
    // An implicit SIGNAL VALUE, where the target is not a symbol or literal.
    assert_eq!(call_shape("signal (e)"), "signal-value E");
    // A label name must be the whole clause, and a number IS a symbol, so
    // `signal 1+1` is not an expression. Measured, both rc 235, Error 21.905.
    assert_eq!(err("signal lab x"), (21, 905));
    assert_eq!(err("signal 1+1"), (21, 905));
    // `Error_Symbol_or_string_signal`, measured as 19.4.
    assert_eq!(err("signal"), (19, 4));
}

// ---- the rest of the procedure family ----

#[test]
fn the_expression_only_instructions_take_what_they_take() {
    // Measured with rexxc, all rc 0.
    assert_eq!(names(&ok("return")), ["RETURN"]);
    assert_eq!(names(&ok("return 1")), ["RETURN"]);
    assert_eq!(names(&ok("exit")), ["EXIT"]);
    assert_eq!(names(&ok("exit 1")), ["EXIT"]);
    assert_eq!(names(&ok("reply")), ["REPLY"]);
    assert_eq!(names(&ok("interpret \"x\"")), ["INTERPRET"]);
    assert_eq!(names(&ok("options \"x\"")), ["OPTIONS"]);
    // INTERPRET and OPTIONS require theirs, and name their own sub-numbers.
    // Measured: `interpret` is 35.912 and `options` is 35.913.
    assert_eq!(err("interpret"), (35, 912));
    assert_eq!(err("options"), (35, 913));
}

#[test]
fn the_interpret_only_rejections_come_from_the_source_kind() {
    // All four are measured at RUN time, because rexxc never parses the
    // string: `interpret "reply 1"` is 99.924, `"forward to 1"` is 99.923,
    // `"guard on"` is 99.912 and `"use local a"` is 99.915.
    for (text, expected) in [
        ("reply 1", (99, 924)),
        ("forward to 1", (99, 923)),
        ("guard on", (99, 912)),
        ("use local a", (99, 915)),
        ("expose a", (99, 908)),
    ] {
        let got = match parse_kind(text, SourceKind::Interpret) {
            Ok(_) => panic!("{text:?} must be rejected inside INTERPRET"),
            Err(e) => (e.code, e.sub),
        };
        assert_eq!(got, expected, "{text:?} inside INTERPRET");
        // The other direction: every one of them is legal in a program.
        assert!(parse(text).is_ok(), "{text:?} as a program");
    }
}

#[test]
fn procedure_takes_only_expose() {
    // `procedure` alone parses, which is the Step 4 table's point: measured,
    // rc 0 under rexxc and Error 17.1 only at run time.
    assert_eq!(names(&ok("procedure")), ["PROCEDURE"]);
    assert_eq!(variable_list("procedure expose a"), ["A"]);
    // Measured: `procedure foo` is 25.17 and `procedure expose` is 20.902.
    assert_eq!(err("procedure foo"), (25, 17));
    assert_eq!(err("procedure expose"), (20, 902));
}

#[test]
fn guard_takes_on_or_off_and_an_optional_when() {
    // Measured with rexxc inside a method, all rc 0.
    assert_eq!(names(&ok("guard on")), ["GUARD"]);
    assert_eq!(names(&ok("guard off")), ["GUARD"]);
    assert_eq!(names(&ok("guard on when 1")), ["GUARD"]);
    // Measured: bare `guard` and `guard foo` are 25.913, while `guard on foo`
    // and `guard on 1` are 25.912 -- a different number for the second
    // keyword.
    assert_eq!(err("guard"), (25, 913));
    assert_eq!(err("guard foo"), (25, 913));
    assert_eq!(err("guard 1"), (25, 913));
    assert_eq!(err("guard on foo"), (25, 912));
    assert_eq!(err("guard on 1"), (25, 912));
}

#[test]
fn every_forward_option_is_accepted_once() {
    // Measured with rexxc inside a method, all rc 0.
    assert_eq!(names(&ok("forward")), ["FORWARD"]);
    assert_eq!(names(&ok("forward to 1")), ["FORWARD"]);
    assert_eq!(
        names(&ok("forward message \"x\" class .a arguments (1) continue")),
        ["FORWARD"]
    );
    assert_eq!(names(&ok("forward array (1,2)")), ["FORWARD"]);
    // Measured, each: `forward to 1 to 2` is 25.917, `forward to` is 35.925,
    // `forward array 1` is 35.924, `forward arguments (1) array (2)` is
    // 25.918, `forward foo` is 25.916 and `forward continue continue` is
    // 25.919.
    assert_eq!(err("forward to 1 to 2"), (25, 917));
    assert_eq!(err("forward to"), (35, 925));
    assert_eq!(err("forward array 1"), (35, 924));
    assert_eq!(err("forward arguments (1) array (2)"), (25, 918));
    assert_eq!(err("forward foo"), (25, 916));
    assert_eq!(err("forward continue continue"), (25, 919));
    assert_eq!(err("forward 1"), (25, 916));
}

#[test]
fn raise_accepts_its_conditions_and_rejects_any() {
    // Measured with rexxc, all rc 0.
    for text in [
        "raise syntax 1",
        "raise error 1",
        "raise failure 1",
        "raise halt",
        "raise novalue",
        "raise propagate",
        "raise user x",
        "raise error 1 description \"d\" additional (1)",
        "raise error 1 array (1,2)",
        "raise error 1 return",
        "raise error 1 return 2",
    ] {
        assert_eq!(names(&ok(text)), ["RAISE"], "{text:?}");
    }
    // ANY is a condition name everywhere else and is NOT raisable: measured,
    // `raise any` is rc 231, Error 25.906, the same as `raise foo`.
    assert_eq!(err("raise any"), (25, 906));
    assert_eq!(err("raise foo"), (25, 906));
    // Measured: bare `raise` is 20.914, `raise user` is 20.915,
    // `raise syntax` with no value is 35.1,
    // `raise error 1 additional (1) array (2)` is 25.909,
    // `raise error 1 return 2 exit 3` is 25.911 and
    // `raise error 1 foo` is 25.907.
    assert_eq!(err("raise"), (20, 914));
    assert_eq!(err("raise user"), (20, 915));
    assert_eq!(err("raise syntax"), (35, 1));
    assert_eq!(err("raise error 1 additional (1) array (2)"), (25, 909));
    assert_eq!(err("raise error 1 return 2 exit 3"), (25, 911));
    assert_eq!(err("raise error 1 foo"), (25, 907));
}

/// A rendering of the first instruction when it is a `USE`.
fn use_shape(text: &str) -> String {
    let (instructions, symbols) =
        parse_kind(text, SourceKind::Program).unwrap_or_else(|e| panic!("{text:?}: {e:?}"));
    match &instructions[0].kind {
        InstructionKind::Use(u) => match &**u {
            crate::ast::Use::Local { variables } => {
                let names: Vec<&str> = variables
                    .iter()
                    .map(|v| match v {
                        crate::ast::VariableRef::Direct(id) => symbols.name(*id),
                        crate::ast::VariableRef::Indirect(_) => "(?)",
                    })
                    .collect();
                format!("local {}", names.join(" "))
            }
            crate::ast::Use::Arg {
                strict,
                allow_optionals,
                targets,
            } => {
                let mut out = String::from(if *strict { "strict arg" } else { "arg" });
                for target in targets {
                    match target {
                        None => out.push_str(" -"),
                        Some(t) => {
                            out.push(' ');
                            if t.alias {
                                out.push('>');
                            }
                            out.push_str(&t.target.shape(&symbols));
                            if let Some(d) = &t.default {
                                out.push_str(&format!("={}", d.shape(&symbols)));
                            }
                        }
                    }
                }
                if *allow_optionals {
                    out.push_str(" ...");
                }
                out
            }
        },
        other => panic!("{text:?} is not a USE: {other:?}"),
    }
}

#[test]
fn every_use_arg_form_reaches_its_own_shape() {
    // Measured with rexxc, all rc 0.
    assert_eq!(use_shape("use arg a"), "arg A");
    assert_eq!(use_shape("use arg a, b"), "arg A B");
    assert_eq!(use_shape("use arg , b"), "arg - B");
    assert_eq!(use_shape("use strict arg a"), "strict arg A");
    assert_eq!(use_shape("use arg a = 1"), "arg A=1");
    assert_eq!(use_shape("use strict arg a, b = 2"), "strict arg A B=2");
    assert_eq!(use_shape("use arg a, ..."), "arg A ...");
    assert_eq!(use_shape("use arg >a"), "arg >A");
    assert_eq!(use_shape("use arg <a."), "arg >stem:A.");
    assert_eq!(use_shape("use arg >a, b"), "arg >A B");
    assert_eq!(use_shape("use arg q~x"), "arg (msg~ Q \"X\")");
    assert_eq!(use_shape("use arg a.b"), "arg compound:A.[var:B]");
    // An empty list is legal. Measured: `use arg` alone is rc 0.
    assert_eq!(use_shape("use arg"), "arg");
    assert_eq!(use_shape("use local a"), "local A");
    assert_eq!(use_shape("use local"), "local ");
}

#[test]
fn use_arg_rejects_what_the_oracle_rejects() {
    // Measured, each of these: `use foo` is 25.905, `use strict foo` is
    // 25.929, `use arg ..., a` is 99.930, `use arg >a = 1` is 99.950,
    // `use arg >a.b` is 20.931, `use arg a b` is 46.902, `use arg a = ` is
    // 35.930 and `use arg 1` is 31.2.
    assert_eq!(err("use foo"), (25, 905));
    assert_eq!(err("use strict foo"), (25, 929));
    assert_eq!(err("use arg ..., a"), (99, 930));
    assert_eq!(err("use arg >a = 1"), (99, 950));
    assert_eq!(err("use arg >a.b"), (20, 931));
    assert_eq!(err("use arg a b"), (46, 902));
    assert_eq!(err("use arg a ="), (35, 930));
    assert_eq!(err("use arg 1"), (31, 2));
    // USE LOCAL has its own list rules: measured, `use local 1` is 31.2,
    // `use local .a` is 31.3 and `use local a.b` is 99.948, because only a
    // simple variable or a stem can be local.
    assert_eq!(err("use local 1"), (31, 2));
    assert_eq!(err("use local .a"), (31, 3));
    assert_eq!(err("use local a.b"), (99, 948));
}

// ---- the settings family: NUMERIC and TRACE ----

/// A rendering of the first instruction when it is a `NUMERIC` or a `TRACE`.
fn setting_shape(text: &str) -> String {
    let (instructions, symbols) =
        parse_kind(text, SourceKind::Program).unwrap_or_else(|e| panic!("{text:?}: {e:?}"));
    match &instructions[0].kind {
        InstructionKind::Numeric {
            setting,
            expression,
        } => {
            let what = match setting {
                crate::ast::NumericSetting::Digits => "digits",
                crate::ast::NumericSetting::Fuzz => "fuzz",
                crate::ast::NumericSetting::FormDefault => "form-default",
                crate::ast::NumericSetting::FormScientific => "form-scientific",
                crate::ast::NumericSetting::FormEngineering => "form-engineering",
                crate::ast::NumericSetting::FormValue => "form-value",
            };
            match expression {
                Some(e) => format!("{what} {}", e.shape(&symbols)),
                None => what.to_string(),
            }
        }
        InstructionKind::Trace(trace) => match trace {
            crate::ast::Trace::Default => "default".to_string(),
            crate::ast::Trace::Setting(s) => {
                format!("setting {}", String::from_utf8_lossy(s))
            }
            crate::ast::Trace::Skip(n) => format!("skip {n}"),
            crate::ast::Trace::Value(e) => format!("value {}", e.shape(&symbols)),
        },
        other => panic!("{text:?} is not a NUMERIC or TRACE: {other:?}"),
    }
}

#[test]
fn every_numeric_form_reaches_its_own_setting() {
    // Measured with rexxc, all rc 0.
    assert_eq!(setting_shape("numeric digits"), "digits");
    assert_eq!(setting_shape("numeric digits 5"), "digits 5");
    assert_eq!(setting_shape("numeric fuzz 1"), "fuzz 1");
    assert_eq!(setting_shape("numeric form"), "form-default");
    assert_eq!(setting_shape("numeric form scientific"), "form-scientific");
    assert_eq!(
        setting_shape("numeric form engineering"),
        "form-engineering"
    );
    assert_eq!(setting_shape("numeric form value 1"), "form-value 1");
    // An implicit FORM VALUE, where what follows FORM is not a symbol.
    assert_eq!(setting_shape("numeric form (e)"), "form-value E");
}

#[test]
fn numeric_rejects_what_the_oracle_rejects() {
    // Measured: bare `numeric` and `numeric "x"` are 20.905, `numeric foo` is
    // 25.15, `numeric form foo` is 25.11 and `numeric form scientific x` is
    // 21.911.
    assert_eq!(err("numeric"), (20, 905));
    assert_eq!(err("numeric \"x\""), (20, 905));
    assert_eq!(err("numeric foo"), (25, 15));
    assert_eq!(err("numeric form foo"), (25, 11));
    assert_eq!(err("numeric form scientific x"), (21, 911));
}

#[test]
fn every_trace_form_reaches_its_own_variant() {
    // Measured with rexxc, all rc 0.
    assert_eq!(setting_shape("trace"), "default");
    assert_eq!(setting_shape("trace r"), "setting R");
    assert_eq!(setting_shape("trace ?r"), "setting ?R");
    assert_eq!(setting_shape("trace ??r"), "setting ??R");
    // Only the first non-`?` character means anything, which is why the long
    // spellings work at all.
    assert_eq!(setting_shape("trace results"), "setting RESULTS");
    assert_eq!(setting_shape("trace \"?r\""), "setting ?r");
    assert_eq!(setting_shape("trace ''"), "setting ");
    assert_eq!(setting_shape("trace 5"), "skip 5");
    assert_eq!(setting_shape("trace -5"), "skip -5");
    assert_eq!(setting_shape("trace +5"), "skip 5");
    assert_eq!(setting_shape("trace - 5"), "skip -5");
    assert_eq!(setting_shape("trace -\"5\""), "skip -5");
    assert_eq!(setting_shape("trace 0"), "skip 0");
    assert_eq!(setting_shape("trace value 1"), "value 1");
    assert_eq!(setting_shape("trace (e)"), "value E");
}

#[test]
fn the_trace_number_gate_is_exactly_the_oracles() {
    // The number test runs BEFORE the option test, so a numeric-looking
    // setting is a skip count and everything else is an option string. Both
    // directions of the boundary are measured against rexxc.
    //
    // Whole and within nine digits, so a skip count:
    assert_eq!(setting_shape("trace 1e2"), "skip 100");
    assert_eq!(setting_shape("trace 123456789"), "skip 123456789");
    // Not whole, or wider than nine digits, so not a number -- and then not a
    // valid option either, because it starts with a digit. Measured: all four
    // are rc 232, Error 24.1.
    assert_eq!(err("trace 1234567890"), (24, 1));
    assert_eq!(err("trace 1e20"), (24, 1));
    assert_eq!(err("trace 1.5"), (24, 1));
    assert_eq!(err("trace 1e-2"), (24, 1));
    // An unknown option letter. Measured: 24.1.
    assert_eq!(err("trace zzz"), (24, 1));
    // Every letter the setting parser knows, which is the other direction of
    // the same gate. Measured: all rc 0.
    for letter in ["a", "c", "l", "e", "f", "n", "o", "r", "i"] {
        let text = format!("trace {letter}");
        assert_eq!(
            setting_shape(&text),
            format!("setting {}", letter.to_uppercase()),
            "{text:?}"
        );
    }
}

#[test]
fn trace_rejects_what_the_oracle_rejects() {
    // Measured: `trace 5 x` and `trace r x` are 21.906, `trace -a` is 26.7 and
    // `trace value` is 35.916.
    assert_eq!(err("trace 5 x"), (21, 906));
    assert_eq!(err("trace r x"), (21, 906));
    assert_eq!(err("trace -a"), (26, 7));
    assert_eq!(err("trace value"), (35, 916));
}
