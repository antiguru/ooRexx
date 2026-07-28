//! Clause splitting, pinned against `build/bin/rexx` with `trace r`.
//!
//! `trace r` prints one `*-*` line per clause holding exactly the clause's
//! span, so every span expectation here was read off the interpreter rather
//! than reasoned about. A loop re-traces its body per iteration, so the number
//! of `*-*` lines is not the number of clauses and no test here counts them.

use rexx_parse::{Clause, ProgramSource, SourceKind, Tag, Token, TokenKind, scan, split_clauses};

fn tokens_of(text: &str) -> Vec<Token> {
    let source = ProgramSource::new(text.as_bytes().to_vec(), SourceKind::Program);
    scan(&source).expect("scans without error").tokens
}

fn clauses(text: &str) -> Vec<Clause> {
    split_clauses(&tokens_of(text)).expect("splits without error")
}

fn clause_count(text: &str) -> usize {
    clauses(text).len()
}

/// The source text of each clause's span, which is what `trace r` prints.
fn traced(text: &str) -> Vec<&str> {
    clauses(text)
        .iter()
        .map(|c| &text[c.span.clone()])
        .collect()
}

/// The tags of the tokens a clause holds, terminator excluded.
fn shape(text: &str, n: usize) -> Vec<Tag> {
    let toks = tokens_of(text);
    let cs = split_clauses(&toks).expect("splits without error");
    toks[cs[n].tokens.clone()]
        .iter()
        .map(|t| t.kind.tag())
        .collect()
}

#[test]
fn a_trailing_comma_continues_the_clause() {
    // build/bin/rexx, trace r: `say 1,` / `  + 2` traces as one clause,
    // `say 1,  + 2`, and prints 3.
    assert_eq!(clause_count("say 1,\n  + 2"), 1);
    assert_eq!(clause_count("say 1\nsay 2"), 2);
}

#[test]
fn a_colon_makes_a_label_clause() {
    let cs = clauses("here: say 1");
    assert_eq!(cs.len(), 2);
    assert!(cs[0].label.is_some());
    assert!(cs[1].label.is_none());
}

#[test]
fn a_clause_span_includes_its_terminating_semicolon() {
    // build/bin/rexx, trace r:  `nop;` is traced with the semicolon.
    let src = "nop;\nsay 1\n";
    let cs = clauses(src);
    assert_eq!(&src[cs[0].span.clone()], "nop;");
    // An uncontinued end of line is a terminator too, but contributes no bytes.
    assert_eq!(&src[cs[1].span.clone()], "say 1");
}

#[test]
fn a_label_span_includes_its_colon() {
    // build/bin/rexx, trace r:  `here: nop` traces as `here:` then `nop`.
    let src = "here: nop\n";
    let cs = clauses(src);
    assert_eq!(&src[cs[0].span.clone()], "here:");
}

#[test]
fn end_of_file_terminates_the_last_clause_without_a_newline() {
    assert_eq!(traced("say 1"), ["say 1"]);
    assert_eq!(traced("here:"), ["here:"]);
}

#[test]
fn a_semicolon_ends_the_span_and_a_blank_before_it_does_not() {
    // Measured: `say 1 ;` traces with the blank, `say 1;   ` traces without the
    // blanks, because the terminator is the semicolon and not the line end.
    assert_eq!(traced("say 1 ;\n"), ["say 1 ;"]);
    assert_eq!(traced("say 1;   \n"), ["say 1;"]);
}

#[test]
fn a_line_end_terminator_carries_the_rest_of_the_line_into_the_span() {
    // Measured: `say 1 -- trailing comment` and `say 1   ` are both traced in
    // full. The `--` truncates the line for scanning but the terminator's
    // position is still the end of the line's content.
    assert_eq!(
        traced("say 1 -- trailing comment\n"),
        ["say 1 -- trailing comment"]
    );
    assert_eq!(traced("say 1   \n"), ["say 1   "]);
    // A comment ahead of the semicolon lands in the span for the same reason.
    assert_eq!(
        traced("say 1 /* c */ ; say 2\n"),
        ["say 1 /* c */ ;", "say 2"]
    );
}

#[test]
fn a_label_clause_ends_at_the_colon_even_when_a_semicolon_follows() {
    // Measured: `here: ; nop` traces as `here:` then `nop`, so the colon fixes
    // the label's span end although the semicolon is what terminates.
    assert_eq!(traced("here: ; nop\n"), ["here:", "nop"]);
    // And the label's span reaches the colon across whatever sits between.
    assert_eq!(traced("here : nop\n"), ["here :", "nop"]);
    assert_eq!(traced("here /*c*/: nop\n"), ["here /*c*/:", "nop"]);
}

#[test]
fn consecutive_labels_each_become_their_own_clause() {
    // Measured: `a: b: nop` traces as `a:` then `b:` then `nop`.
    let src = "a: b: nop\n";
    assert_eq!(traced(src), ["a:", "b:", "nop"]);
    let cs = clauses(src);
    assert_eq!(cs[0].label, Some(0..1));
    assert_eq!(cs[1].label, Some(2..3));
    assert_eq!(cs[2].label, None);
}

#[test]
fn a_literal_or_a_constant_symbol_labels_a_clause_too() {
    // `isSymbolOrLiteral` (`Token.hpp:580`) ignores the symbol's class.
    // Measured: all four of these trace as a label clause plus `nop`.
    assert_eq!(traced("\"lit\": nop\n"), ["\"lit\":", "nop"]);
    assert_eq!(traced("1: nop\n"), ["1:", "nop"]);
    assert_eq!(traced(".a: nop\n"), [".a:", "nop"]);
    assert_eq!(traced("stem.: nop\n"), ["stem.:", "nop"]);
}

#[test]
fn a_double_colon_is_not_a_label() {
    // Measured: `here:: nop` traces as one clause and then fails with error
    // 35.1, `Incorrect expression detected at "::"`, so it was never split.
    let src = "here:: nop\n";
    assert_eq!(traced(src), ["here:: nop"]);
    assert_eq!(clauses(src)[0].label, None);
}

#[test]
fn a_colon_after_the_first_token_is_not_a_label() {
    // Only the clause's own first token can carry a label, so the colon in
    // `say a: b` stays inside the clause instead of splitting it.
    let src = "say a: b\n";
    assert_eq!(traced(src), ["say a: b"]);
    assert_eq!(
        shape(src, 0),
        [
            Tag::Symbol,
            Tag::Blank,
            Tag::Symbol,
            Tag::Colon,
            Tag::Symbol
        ]
    );
}

#[test]
fn a_terminator_belongs_to_no_clause_token_range() {
    // The `Eoc` and the label's `Colon` are both excluded from `tokens` while
    // both are inside `span`.
    let src = "here: nop;\n";
    let toks = tokens_of(src);
    let cs = split_clauses(&toks).expect("splits without error");
    assert_eq!(cs[0].tokens, 0..1);
    assert_eq!(toks[1].kind, TokenKind::Colon);
    assert_eq!(cs[1].tokens, 2..3);
    assert_eq!(toks[3].kind, TokenKind::Eoc);
    assert_eq!(traced(src), ["here:", "nop;"]);
}

#[test]
fn a_program_with_nothing_to_run_has_no_clauses() {
    // `scan` collapses terminators and drops an empty final clause, so a blank
    // line, a lone comment and a stray semicolon all leave nothing to split.
    assert_eq!(clause_count(""), 0);
    assert_eq!(clause_count("\n\n\n"), 0);
    assert_eq!(clause_count("/* only */\n"), 0);
    assert_eq!(clause_count(";;;\n"), 0);
    // And a leading comment line does not shift the following clause's span.
    let src = "/* only */\nsay 1\n";
    assert_eq!(traced(src), ["say 1"]);
    assert_eq!(clauses(src)[0].span, 11..16);
}

#[test]
fn repeated_semicolons_produce_one_clause_each_not_an_empty_one() {
    // Measured: `say 1;;say 2` traces as `say 1;` then `say 2`, so the second
    // semicolon is not part of either span.
    assert_eq!(traced("say 1;;say 2\n"), ["say 1;", "say 2"]);
}

#[test]
fn a_multi_line_clause_spans_from_its_first_token_to_the_last_line_end() {
    let src = "say 1,\n  + 2\n";
    let cs = clauses(src);
    assert_eq!(cs.len(), 1);
    // Across the line terminator, which the span therefore contains: `trace r`
    // prints `say 1,  + 2`, so rendering the span is not a plain slice.
    assert_eq!(cs[0].span, 0..12);
    assert_eq!(&src[cs[0].span.clone()], "say 1,\n  + 2");
}

#[test]
fn a_clause_holds_every_token_between_its_terminators() {
    let src = "say a b\nnop\n";
    assert_eq!(
        shape(src, 0),
        [
            Tag::Symbol,
            Tag::Blank,
            Tag::Symbol,
            Tag::Blank,
            Tag::Symbol
        ]
    );
    assert_eq!(shape(src, 1), [Tag::Symbol]);
}
