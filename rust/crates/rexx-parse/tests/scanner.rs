//! Scanner behaviour, pinned against `build/bin/rexx` and
//! `build/bin/rexxc`.
//!
//! Every expectation here that names an interpreter behaviour was measured,
//! not inferred. `rexxc` gives the parse verdict without executing, which is
//! the only way to assert the negative direction (*this file parses*);
//! `rexx`'s output is used where both spellings parse and only the printed
//! result distinguishes them, as with `say a/*c*/b` against `say a b`.

use rexx_parse::{
    Operator, ProgramSource, ScanMode, Scanned, SymbolClass, SymbolId, Tag, Token, TokenKind, scan,
};

fn scan_all(text: &str) -> Scanned {
    let source = ProgramSource::new(text.as_bytes().to_vec());
    scan(&source, ScanMode::Program).expect("scans without error")
}

fn scan_ok(text: &str) -> Vec<Token> {
    scan_all(text).tokens
}

fn kinds(toks: &[Token]) -> Vec<Tag> {
    toks.iter().map(|t| t.kind.tag()).collect()
}

/// The decoded value of the first literal token.
fn literal_text(toks: &[Token]) -> String {
    String::from_utf8(literal_bytes(toks, 0)).expect("literal is text in this test")
}

/// The decoded value of the `n`th literal token.
fn literal_bytes(toks: &[Token], n: usize) -> Vec<u8> {
    toks.iter()
        .filter_map(|t| match &t.kind {
            TokenKind::Literal { value } => Some(value.to_vec()),
            _ => None,
        })
        .nth(n)
        .expect("a literal token")
}

/// The operators, in order.
fn operators(toks: &[Token]) -> Vec<Operator> {
    toks.iter()
        .filter_map(|t| match t.kind {
            TokenKind::Operator(op) | TokenKind::Assignment(op) => Some(op),
            _ => None,
        })
        .collect()
}

/// The symbols, in order, as (id, class).
fn symbols(toks: &[Token]) -> Vec<(SymbolId, SymbolClass)> {
    toks.iter()
        .filter_map(|t| match t.kind {
            TokenKind::Symbol { id, class } => Some((id, class)),
            _ => None,
        })
        .collect()
}

/// Scans `text` expecting a refusal, and answers `(code, sub, line)`.
fn scan_err(text: &str) -> (u16, u16, usize) {
    let source = ProgramSource::new(text.as_bytes().to_vec());
    let error = scan(&source, ScanMode::Program).expect_err("does not scan");
    (error.code, error.sub, source.line_of(error.byte))
}

#[test]
fn block_comments_nest() {
    // `1` is a symbol in Rexx, and the blank between `say` and `1` is
    // significant: previous token is a symbol, next character starts a symbol.
    let toks = scan_ok("/* a /* b */ c */ say 1");
    assert_eq!(
        kinds(&toks),
        [Tag::Symbol, Tag::Blank, Tag::Symbol, Tag::Eoc]
    );
}

#[test]
fn double_dash_starts_a_line_comment_but_minus_does_not() {
    // build/bin/rexx: say 1 -- 2  =>  1     (the `-- 2` is a comment)
    //                say 1 - 2    =>  -1
    // No `Blank` in either. In "a -- b" the look-ahead past `a `'s blank finds
    // `-`, which starts neither a symbol, a literal, `(` nor `[`, so the blank
    // is discarded; then `--` truncates the line and yields the clause end.
    assert_eq!(kinds(&scan_ok("a -- b")), [Tag::Symbol, Tag::Eoc]);
    // In "a - b" the same look-ahead discards the first blank, and the blank
    // before `b` is insignificant because the previous token is an operator.
    assert_eq!(
        kinds(&scan_ok("a - b")),
        [Tag::Symbol, Tag::Operator, Tag::Symbol, Tag::Eoc]
    );
    // And with no blanks at all: measured, `say 1-2` prints -1.
    assert_eq!(
        kinds(&scan_ok("1-2")),
        [Tag::Symbol, Tag::Operator, Tag::Symbol, Tag::Eoc]
    );
}

#[test]
fn a_significant_blank_needs_both_sides() {
    // Left side must be a symbol, a literal, `)` or `]`; right side must start
    // a symbol or a literal, or be `(` or `[`.
    assert_eq!(
        kinds(&scan_ok("f (x)")),
        [
            Tag::Symbol,
            Tag::Blank,
            Tag::LeftParen,
            Tag::Symbol,
            Tag::RightParen,
            Tag::Eoc
        ]
    );
    assert_eq!(
        kinds(&scan_ok("f(x)")),
        [
            Tag::Symbol,
            Tag::LeftParen,
            Tag::Symbol,
            Tag::RightParen,
            Tag::Eoc
        ]
    );
}

#[test]
fn a_closing_paren_also_makes_a_following_blank_significant() {
    // Measured: `say ('x') ('y')` prints `x y` and `say ('x')('y')` prints
    // `xy`, so the blank between `)` and `(` is an operator.
    let with_blank = scan_ok("say ('x') ('y')");
    let without = scan_ok("say ('x')('y')");
    assert_eq!(
        kinds(&with_blank)
            .iter()
            .filter(|t| **t == Tag::Blank)
            .count(),
        2
    );
    assert_eq!(
        kinds(&without).iter().filter(|t| **t == Tag::Blank).count(),
        1
    );
}

#[test]
fn a_blank_before_an_operator_is_discarded() {
    // The right side of the rule admits only a symbol, a literal, `(` and
    // `[`, so `a ~ b` carries no blank token at all even though the left side
    // qualifies.
    assert_eq!(
        kinds(&scan_ok("a ~ b")),
        [Tag::Symbol, Tag::Tilde, Tag::Symbol, Tag::Eoc]
    );
    assert_eq!(
        kinds(&scan_ok("a ~~ b")),
        [Tag::Symbol, Tag::DTilde, Tag::Symbol, Tag::Eoc]
    );
}

#[test]
fn a_tab_is_a_blank() {
    // Measured with `a = 1`: `say a<TAB>2` prints `1 2`, and so does
    // `say a<TAB><TAB>2`, so a run of whitespace is one blank token.
    assert_eq!(
        kinds(&scan_ok("say a\t2")),
        [
            Tag::Symbol,
            Tag::Blank,
            Tag::Symbol,
            Tag::Blank,
            Tag::Symbol,
            Tag::Eoc
        ]
    );
    assert_eq!(
        kinds(&scan_ok("say a\t \t2")),
        [
            Tag::Symbol,
            Tag::Blank,
            Tag::Symbol,
            Tag::Blank,
            Tag::Symbol,
            Tag::Eoc
        ]
    );
}

#[test]
fn a_continuation_becomes_a_significant_blank() {
    // build/bin/rexx: say "a"-  /  "b"   =>  a b     (blank, so a concatenation)
    //                 say "a"||-  /  "b" =>  ab      (previous token is `||`)
    assert_eq!(
        kinds(&scan_ok("say \"a\"-\n\"b\"")),
        [
            Tag::Symbol,
            Tag::Blank,
            Tag::Literal,
            Tag::Blank,
            Tag::Literal,
            Tag::Eoc
        ]
    );
    assert_eq!(
        kinds(&scan_ok("say \"a\"||-\n\"b\"")),
        [
            Tag::Symbol,
            Tag::Blank,
            Tag::Literal,
            Tag::Operator,
            Tag::Literal,
            Tag::Eoc
        ]
    );
    // A comma continues a line the same way: measured, `say 'a',` then `'b'`
    // prints `a b`.
    assert_eq!(
        kinds(&scan_ok("say 'a',\n'b'")),
        [
            Tag::Symbol,
            Tag::Blank,
            Tag::Literal,
            Tag::Blank,
            Tag::Literal,
            Tag::Eoc
        ]
    );
}

#[test]
fn a_continuation_with_no_next_line_is_simply_consumed() {
    // Measured: a file whose only line is `say 1,` prints 1. The comma is
    // neither a comma token nor a blank, because there is no line to continue
    // onto.
    assert_eq!(
        kinds(&scan_ok("say 1,")),
        [Tag::Symbol, Tag::Blank, Tag::Symbol, Tag::Eoc]
    );
}

#[test]
fn a_continuation_onto_an_empty_line_ends_the_clause() {
    // Measured: `say "a",` then an empty line then `"b"` prints `a`, and then
    // runs `b` as a command, so the continuation did not reach line 3.
    assert_eq!(
        kinds(&scan_ok("say \"a\",\n\n\"b\"")),
        [
            Tag::Symbol,
            Tag::Blank,
            Tag::Literal,
            Tag::Eoc,
            Tag::Literal,
            Tag::Eoc
        ]
    );
}

#[test]
fn a_comment_separates_tokens_without_inserting_a_blank() {
    // Measured with `a = 1; b = 2`: `say a/*c*/b` prints 12 while
    // `say a b` prints `1 2`. So a comment is neither whitespace nor nothing:
    // dropping it would glue the symbols into one, emitting a blank for it
    // would insert a space the interpreter does not.
    assert_eq!(
        kinds(&scan_ok("say a/*c*/b")),
        [Tag::Symbol, Tag::Blank, Tag::Symbol, Tag::Symbol, Tag::Eoc]
    );
    assert_eq!(
        kinds(&scan_ok("say a b")),
        [
            Tag::Symbol,
            Tag::Blank,
            Tag::Symbol,
            Tag::Blank,
            Tag::Symbol,
            Tag::Eoc
        ]
    );
}

#[test]
fn blanks_and_comments_between_an_operators_two_halves_do_not_split_it() {
    // The look-ahead for a doubled operator character ignores blanks and
    // comments, so these are all one operator. Measured: `say 1 == 1.0`,
    // `say 1 = = 1.0` and `say 1 =/*c*/= 1.0` all print 0, while
    // `say 1 = 1.0` prints 1.
    assert_eq!(operators(&scan_ok("1 == 1.0")), [Operator::StrictEqual]);
    assert_eq!(operators(&scan_ok("1 = = 1.0")), [Operator::StrictEqual]);
    assert_eq!(
        operators(&scan_ok("1 =/*c*/= 1.0")),
        [Operator::StrictEqual]
    );
    assert_eq!(operators(&scan_ok("1 = 1.0")), [Operator::Equal]);
    // Measured the same way: `say 2 < = 2` prints 1, as `say 2 <= 2` does,
    // while `say 2 < 2` prints 0.
    assert_eq!(operators(&scan_ok("2 < = 2")), [Operator::LessThanEqual]);
    assert_eq!(operators(&scan_ok("2 < 2")), [Operator::LessThan]);
}

#[test]
fn a_continuation_between_an_operators_two_halves_does_not_split_it_either() {
    // Measured: `say 1 =-` then `= 1.0` prints 0, and `say 1 =/*` then
    // `*/= 1.0` prints 0. Both reach the second `=` on the next line.
    assert_eq!(operators(&scan_ok("1 =-\n= 1.0")), [Operator::StrictEqual]);
    assert_eq!(
        operators(&scan_ok("1 =/*\n*/= 1.0")),
        [Operator::StrictEqual]
    );
    // And the operator's span covers both halves and the gap between them.
    let toks = scan_ok("1 = = 1.0");
    let operator = toks.iter().find(|t| t.kind.tag() == Tag::Operator).unwrap();
    assert_eq!(operator.span, 2..5);
}

#[test]
fn an_operator_followed_by_equals_is_an_assignment_shortcut() {
    // Measured: `x = 5` then `x += 1` then `say x` prints 6.
    assert_eq!(
        kinds(&scan_ok("x += 1")),
        [Tag::Symbol, Tag::Assignment, Tag::Symbol, Tag::Eoc]
    );
    assert_eq!(operators(&scan_ok("x += 1")), [Operator::Plus]);
    for (text, op) in [
        ("x -= 1", Operator::Subtract),
        ("x *= 1", Operator::Multiply),
        ("x /= 1", Operator::Divide),
        ("x %= 1", Operator::IntDiv),
        ("x //= 1", Operator::Remainder),
        ("x **= 1", Operator::Power),
        ("x ||= 1", Operator::Concatenate),
        ("x &= 1", Operator::And),
        ("x |= 1", Operator::Or),
        ("x &&= 1", Operator::Xor),
    ] {
        assert_eq!(kinds(&scan_ok(text))[1], Tag::Assignment, "{text}");
        assert_eq!(operators(&scan_ok(text)), [op], "{text}");
    }
    // `=` is the assignment itself, and `==` is a comparison, so neither is a
    // shortcut.
    assert_eq!(kinds(&scan_ok("x = 1"))[1], Tag::Operator);
    assert_eq!(kinds(&scan_ok("x == 1"))[1], Tag::Operator);
}

#[test]
fn an_exponent_sign_joins_the_symbol_only_when_digits_follow() {
    // Measured: `say 1e+5` prints 1E+5 and `say 1.5e-3` prints 1.5E-3, so the
    // sign is inside the symbol. With `y = 5`, `say 1e+y` fails with
    // `Nonnumeric value ("1E") used in arithmetic operation` and `say 1e5+y`
    // prints 100005, so there the sign is an operator and the symbol stops
    // before it.
    let toks = scan_ok("1e+5");
    assert_eq!(kinds(&toks), [Tag::Symbol, Tag::Eoc]);
    assert_eq!(scan_all("1e+5").symbols.name(symbols(&toks)[0].0), "1E+5");

    let split = scan_ok("1e+y");
    assert_eq!(
        kinds(&split),
        [Tag::Symbol, Tag::Operator, Tag::Symbol, Tag::Eoc]
    );
    assert_eq!(split[0].span, 0..2, "the symbol is `1e`, without the sign");
    assert_eq!(operators(&split), [Operator::Plus]);

    // A sign at the end of the line has no digits after it either.
    assert_eq!(
        kinds(&scan_ok("1e+")),
        [Tag::Symbol, Tag::Operator, Tag::Eoc]
    );
}

#[test]
fn a_symbol_is_classified_by_its_shape() {
    let cases = [
        (".", SymbolClass::Dummy),
        (".5", SymbolClass::Constant),
        ("1", SymbolClass::Constant),
        ("1.5e3", SymbolClass::Constant),
        (".nil", SymbolClass::DotSymbol),
        (".a.b", SymbolClass::DotSymbol),
        ("abc", SymbolClass::Variable),
        ("a?b!c_d", SymbolClass::Variable),
        ("stem.", SymbolClass::Stem),
        ("stem.i", SymbolClass::Compound),
        ("stem.i.j", SymbolClass::Compound),
        ("a..b", SymbolClass::Compound),
    ];
    for (text, class) in cases {
        let toks = scan_ok(text);
        assert_eq!(kinds(&toks), [Tag::Symbol, Tag::Eoc], "{text}");
        assert_eq!(symbols(&toks)[0].1, class, "{text}");
    }
    // Measured: `say .5` prints `.5` and `say 1.5` prints `1.5`, so a
    // leading-dot number keeps its spelling; `say .nil` prints
    // `The NIL object`, so `.nil` resolves through the environment instead.
    // And `parse value 'p q' with . y` binds `q` to y, which is what the
    // dummy period is for.
    assert_eq!(
        kinds(&scan_ok("parse value 'p q' with . y")),
        [
            Tag::Symbol,
            Tag::Blank,
            Tag::Symbol,
            Tag::Blank,
            Tag::Literal,
            Tag::Blank,
            Tag::Symbol,
            Tag::Blank,
            Tag::Symbol,
            Tag::Blank,
            Tag::Symbol,
            Tag::Eoc
        ]
    );
}

#[test]
fn one_symbol_has_one_id_and_as_many_spans_as_occurrences() {
    // Measured: `aBc = 1` then `say ABC` prints 1, and `sourceline(1)` gives
    // back `aBc = 1`, so the identity folds case and the spelling does not.
    let scanned = scan_all("aBc = 1\nsay ABC\nsay aBc");
    let ids = symbols(&scanned.tokens);
    // `aBc`, `1`, `say`, `ABC`, `say`, `aBc`.
    assert_eq!(ids[0].0, ids[3].0);
    assert_eq!(ids[0].0, ids[5].0);
    assert_eq!(scanned.symbols.name(ids[0].0), "ABC");

    let spans: Vec<_> = scanned
        .tokens
        .iter()
        .filter(|t| matches!(t.kind, TokenKind::Symbol { id, .. } if id == ids[0].0))
        .map(|t| t.span.clone())
        .collect();
    assert_eq!(spans, [0..3, 12..15, 20..23]);

    // The span, not the id, is what recovers the source spelling, and it does
    // so through `span_bytes`, because a span is an absolute offset and cannot
    // be sliced out of a line. Two of these three are not on line 1, which is
    // the case that matters: `spans[1]` starts at byte 12 while line 1 is 7
    // bytes long.
    let source = ProgramSource::new(b"aBc = 1\nsay ABC\nsay aBc".to_vec());
    let spellings: Vec<&[u8]> = spans
        .iter()
        .map(|span| source.span_bytes(span.clone()).expect("a scanner span"))
        .collect();
    assert_eq!(spellings, [&b"aBc"[..], &b"ABC"[..], &b"aBc"[..]]);
    assert_eq!(source.line_of(spans[1].start), 2);
    assert_eq!(source.line_of(spans[2].start), 3);
    // A span from anywhere else may be out of range, and then there are no
    // bytes rather than a panic or a silently clamped answer.
    assert_eq!(source.span_bytes(0..24), None);
    // Including one a caller assembled from two offsets the wrong way round.
    let (start, end) = (3usize, 1usize);
    assert_eq!(source.span_bytes(start..end), None);
}

#[test]
fn doubled_quotes_are_one_quote() {
    assert_eq!(literal_text(&scan_ok("'it''s'")), "it's");
}

#[test]
fn a_literals_value_is_decoded_rather_than_sliced() {
    // Measured: `s = 'it''s'` then `say length(s)` prints 4, and
    // `say "a""b"` prints `a"b` as one string while `say "a" "b"` prints
    // `a b` as two.
    assert_eq!(literal_text(&scan_ok("\"a\"\"b\"")), "a\"b");
    assert_eq!(kinds(&scan_ok("\"a\"\"b\""))[0], Tag::Literal);
    assert_eq!(
        kinds(&scan_ok("\"a\" \"b\"")),
        [Tag::Literal, Tag::Blank, Tag::Literal, Tag::Eoc]
    );
    // The other delimiter is just text.
    assert_eq!(literal_text(&scan_ok("'a\"b'")), "a\"b");
    assert_eq!(literal_text(&scan_ok("''")), "");
    // A literal may hold bytes that are not text at all, which is why the
    // value is bytes.
    let source = ProgramSource::new(b"'\xff\xfe'".to_vec());
    let toks = scan(&source, ScanMode::Program).expect("scans").tokens;
    assert_eq!(literal_bytes(&toks, 0), b"\xff\xfe");
}

#[test]
fn hex_and_binary_literals_pack_down_to_bytes() {
    // Measured: `say c2x('41 42'x)` prints 4142, `say c2x(''x)` prints
    // nothing, `say c2x('1000 0001'b)` prints 81, `say c2x('1'b)` prints 01
    // and `say c2x('101 0101'b)` prints 55. `say 'A'x` prints an empty line,
    // because `'a'x` is the single byte 0A.
    assert_eq!(literal_bytes(&scan_ok("'41 42'x"), 0), b"\x41\x42");
    assert_eq!(literal_bytes(&scan_ok("'A'x"), 0), b"\x0a");
    assert_eq!(literal_bytes(&scan_ok("'a'X"), 0), b"\x0a");
    assert_eq!(literal_bytes(&scan_ok("''x"), 0), b"");
    assert_eq!(literal_bytes(&scan_ok("''b"), 0), b"");
    assert_eq!(literal_bytes(&scan_ok("'1000 0001'b"), 0), b"\x81");
    assert_eq!(literal_bytes(&scan_ok("'1'b"), 0), b"\x01");
    // A short leading group is legal, in binary as in hex.
    assert_eq!(literal_bytes(&scan_ok("'101 0101'b"), 0), b"\x55");
    // A tab groups the same way a blank does.
    assert_eq!(literal_bytes(&scan_ok("'41\t42'x"), 0), b"\x41\x42");
}

#[test]
fn a_hex_marker_must_not_be_the_start_of_a_longer_symbol() {
    // Measured: with `xy = '!'`, `say 'a'xy` prints `a!`, so `'a'` stayed a
    // string and `xy` is a variable abutted to it. A scanner that took the
    // `x` as a marker would print a newline instead.
    let toks = scan_ok("'a'xy");
    assert_eq!(kinds(&toks), [Tag::Literal, Tag::Symbol, Tag::Eoc]);
    assert_eq!(literal_bytes(&toks, 0), b"a");
    assert_eq!(toks[0].span, 0..3);
    // Measured: `say '41'x'42'x` prints AB, so two hex literals abut.
    let pair = scan_ok("'41'x'42'x");
    assert_eq!(kinds(&pair), [Tag::Literal, Tag::Literal, Tag::Eoc]);
    assert_eq!(literal_bytes(&pair, 0), b"A");
    assert_eq!(literal_bytes(&pair, 1), b"B");
}

#[test]
fn a_clause_terminator_is_emitted_once_however_the_clause_ends() {
    // Measured: `say 1;;say 2;` prints 1 then 2, so the doubled semicolon and
    // the trailing one produce no clause of their own.
    assert_eq!(
        kinds(&scan_ok("say 1;;say 2;")),
        [
            Tag::Symbol,
            Tag::Blank,
            Tag::Symbol,
            Tag::Eoc,
            Tag::Symbol,
            Tag::Blank,
            Tag::Symbol,
            Tag::Eoc
        ]
    );
    // Blank lines around a clause produce nothing at all.
    assert_eq!(
        kinds(&scan_ok("\n\n\nsay 1\n\n\n")),
        [Tag::Symbol, Tag::Blank, Tag::Symbol, Tag::Eoc]
    );
    // Neither does a program made only of terminators, or an empty one.
    assert!(scan_ok(";;;").is_empty());
    assert!(scan_ok("").is_empty());
    assert!(scan_ok("\n\n").is_empty());
    assert!(scan_ok("/* just a comment */").is_empty());
    // End of file terminates the last clause even with no newline after it.
    assert_eq!(kinds(&scan_ok("say 1")).last(), Some(&Tag::Eoc));
    assert_eq!(kinds(&scan_ok("say 1\n")).last(), Some(&Tag::Eoc));
}

#[test]
fn a_shebang_line_is_skipped_by_the_scanner_but_kept_by_the_line_index() {
    // Found by differential testing: 494 of 790 files under `ootest/` and
    // `samples/` open with `#!/usr/bin/env rexx`, which `rexxc` accepts. The
    // line stays visible to `sourceline`.
    let text = "#!/usr/bin/env rexx\nsay 1";
    let source = ProgramSource::new(text.as_bytes().to_vec());
    let toks = scan(&source, ScanMode::Program).expect("scans").tokens;
    assert_eq!(
        kinds(&toks),
        [Tag::Symbol, Tag::Blank, Tag::Symbol, Tag::Eoc]
    );
    assert_eq!(toks[0].span, 20..23, "scanning starts on line 2");
    assert_eq!(source.line_count(), 2);
    assert_eq!(source.line(1), Some(&b"#!/usr/bin/env rexx"[..]));
    // Only `#!` at the very start counts. `#` is not a program character, so
    // measured, `x = 1` then `y = #` is error 13.1 on line 2.
    assert_eq!(scan_err("x = 1\ny = #"), (13, 1, 2));
}

#[test]
fn an_interpret_does_not_skip_a_shebang_line() {
    // `ArrayProgramSource::setup` (`ProgramSource.cpp:594`) guards the skip
    // with `interpretAdjust == 0`. Measured both directions:
    // `interpret "#! nothing here"` is error 13 with
    // `Incorrect character in program "#" ('23'X)`, while the identical text
    // as line 1 of a file is accepted and `say "after"` on line 2 prints
    // `after`. Skipping unconditionally would silently accept an empty
    // program.
    let text = "#! nothing here";
    let source = ProgramSource::new(text.as_bytes().to_vec());
    let error = scan(&source, ScanMode::Interpret).expect_err("does not scan");
    assert_eq!((error.code, error.sub), (13, 1));
    assert_eq!(source.line_of(error.byte), 1);
    assert!(
        scan(&source, ScanMode::Program)
            .expect("scans")
            .tokens
            .is_empty(),
        "as a program the whole thing is the skipped line"
    );

    // The two modes agree on everything else, including a `#` that is not at
    // the start of line 1.
    for text in ["say 1", "#!\nsay 1\nsay 2", "say 1\n#! not line one"] {
        let source = ProgramSource::new(text.as_bytes().to_vec());
        let program = scan(&source, ScanMode::Program);
        let interpret = scan(&source, ScanMode::Interpret);
        let same = match (&program, &interpret) {
            (Ok(a), Ok(b)) => kinds(&a.tokens) == kinds(&b.tokens),
            (Err(a), Err(b)) => a == b,
            _ => false,
        };
        // The first of these three differs by construction; the other two must
        // not.
        assert_eq!(same, !text.starts_with("#!"), "{text:?}");
    }
}

#[test]
fn spans_stay_absolute_across_every_line_terminator() {
    // `ProgramSource` resolves CRLF as one terminator and a bare CR as one,
    // so a span's absolute offset has to skip a different number of bytes on
    // each line. This is the invariant `span_bytes` and `TRACE` both rest on.
    let text = "say 1\r\nsay 22\rsay 333";
    let source = ProgramSource::new(text.as_bytes().to_vec());
    let scanned = scan(&source, ScanMode::Program).expect("scans");
    let last: Vec<(usize, &[u8])> = scanned
        .tokens
        .iter()
        .filter(|t| t.kind.tag() == Tag::Symbol)
        .map(|t| {
            (
                source.line_of(t.span.start),
                source.span_bytes(t.span.clone()).expect("a scanner span"),
            )
        })
        .collect();
    assert_eq!(
        last,
        [
            (1, &b"say"[..]),
            (1, &b"1"[..]),
            (2, &b"say"[..]),
            (2, &b"22"[..]),
            (3, &b"say"[..]),
            (3, &b"333"[..]),
        ]
    );
    // A `\n` followed by a `\r` is two terminators with an empty line
    // between them, and an empty line yields no tokens, so the offsets after
    // it must still land.
    let text = "say 1\n\rsay 2";
    let source = ProgramSource::new(text.as_bytes().to_vec());
    let scanned = scan(&source, ScanMode::Program).expect("scans");
    let after: Vec<(usize, &[u8])> = scanned
        .tokens
        .iter()
        .filter(|t| t.kind.tag() == Tag::Symbol)
        .skip(2)
        .map(|t| {
            (
                source.line_of(t.span.start),
                source.span_bytes(t.span.clone()).expect("a scanner span"),
            )
        })
        .collect();
    // Line 2 is the empty one, so the second clause is on line 3.
    assert_eq!(after, [(3, &b"say"[..]), (3, &b"2"[..])]);
}

#[test]
fn every_error_the_scanner_raises_matches_the_interpreters_number_and_line() {
    // Each of these was run through `build/bin/rexxc`, and the number,
    // sub-number and reported line below are what it printed. The line is the
    // clause's, not the offending character's: in the third case the clause
    // starts on line 1 and the unclosed comment opens on line 2, and the
    // interpreter reports line 1.
    let cases: [(&str, (u16, u16, usize)); 18] = [
        ("/* unclosed\nsay 1", (6, 1, 1)),
        ("say 1\nsay 2 /* opened here\nmore", (6, 1, 2)),
        ("say 1,\n/* unclosed", (6, 1, 1)),
        ("say 1\n/* a /* b */ still open", (6, 1, 2)),
        ("say 'abc", (6, 2, 1)),
        ("say \"abc", (6, 3, 1)),
        ("say 1\nsay 2\nsay 'abc", (6, 2, 3)),
        ("say 1,\n\"abc", (6, 3, 1)),
        ("b\u{e4}c = 2", (13, 1, 1)),
        ("say 1\nx @ y", (13, 1, 2)),
        ("z = $foo", (13, 1, 1)),
        ("say c2x(' 41'x)", (15, 1, 1)),
        ("say c2x(' 1010'b)", (15, 2, 1)),
        ("say c2x('4g'x)", (15, 3, 1)),
        ("say c2x('1012'b)", (15, 4, 1)),
        ("say c2x('4 1'x)", (15, 5, 1)),
        ("say c2x('101 01'b)", (15, 6, 1)),
        ("say 1\n::resource data\nnever terminated", (99, 943, 2)),
    ];
    for (text, expected) in cases {
        assert_eq!(scan_err(text), expected, "{text:?}");
    }
    // A 250-character name is fine and 251 is error 30.1, measured both ways.
    let name = "x".repeat(250);
    assert_eq!(kinds(&scan_ok(&format!("{name} = 1")))[0], Tag::Symbol);
    assert_eq!(scan_err(&format!("{name}x = 1")), (30, 1, 1));
}

#[test]
fn a_resource_body_is_copied_verbatim_rather_than_scanned() {
    // Measured: this file gets rc 0 from `rexxc` and the resource holds two
    // lines, so the unmatched quote and unclosed comment inside it are data.
    // A scanner that tokenised the body would raise 6.2 and 6.1.
    let text =
        "say 1\nexit\n::resource data\nthis is 'unmatched and /* unclosed\nline two\n::END\n";
    let source = ProgramSource::new(text.as_bytes().to_vec());
    let scanned = scan(&source, ScanMode::Program).expect("scans");
    assert_eq!(scanned.resources.len(), 1);
    let body = &scanned.resources[0];
    assert_eq!(scanned.tokens[body.directive].kind.tag(), Tag::DColon);
    let lines: Vec<&[u8]> = body
        .lines
        .iter()
        .map(|span| &text.as_bytes()[span.clone()])
        .collect();
    assert_eq!(
        lines,
        [&b"this is 'unmatched and /* unclosed"[..], &b"line two"[..]]
    );
    // The body produces no tokens, so the last token is the directive
    // clause's terminator.
    assert_eq!(kinds(&scanned.tokens).last(), Some(&Tag::Eoc));
}

#[test]
fn a_resource_end_marker_is_a_prefix_match_on_the_upcased_value() {
    // Measured: with `::resource d2 end stop`, a body line
    // `stop is lowercase, no match` does not end the resource but a later
    // `STOP` line does, and the resource then holds one line. The marker came
    // from a symbol, so it is upcased; a literal marker would not be.
    let text = "say 1\nexit\n::resource d2 end stop\nstop is lowercase, no match\nSTOP\n";
    let source = ProgramSource::new(text.as_bytes().to_vec());
    let scanned = scan(&source, ScanMode::Program).expect("scans");
    assert_eq!(scanned.resources.len(), 1);
    assert_eq!(scanned.resources[0].lines.len(), 1);

    // Measured: with `end 'STOP'`, a line beginning `STOPPING?` ends it, so
    // the test is a prefix and not an equality.
    let prefix = "exit\n::resource d2 end 'STOP'\n::END is just data here\nSTOPPING? yes\n";
    let source = ProgramSource::new(prefix.as_bytes().to_vec());
    let scanned = scan(&source, ScanMode::Program).expect("scans");
    assert_eq!(scanned.resources[0].lines.len(), 1);

    // A malformed `::RESOURCE` is left alone: the interpreter rejects it in
    // the directive parser before it reads a line, so the lines after it are
    // ordinary Rexx there too.
    // Measured: `::resource data junk` is error 25.926 on line 2, which is a
    // parse error and not the scanner's to raise.
    let malformed = "exit\n::resource data junk\nsay 2\n::END\n";
    let scanned = scan_all(malformed);
    assert!(scanned.resources.is_empty());
    assert!(
        scanned
            .tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::Symbol { .. })
                && &malformed.as_bytes()[t.span.clone()] == b"say"),
        "the line after the directive was tokenised"
    );
}

#[test]
fn a_ctrl_z_ends_the_program_before_the_scanner_sees_it() {
    // `ProgramSource` truncates there, so what follows cannot raise a scan
    // error. Measured: `say 1` then a line beginning with 0x1A followed by
    // `say 'unclosed` gets rc 0 from `rexxc`.
    let source = ProgramSource::new(b"say 1\n\x1asay 'unclosed\n".to_vec());
    let scanned = scan(&source, ScanMode::Program).expect("scans");
    assert_eq!(
        kinds(&scanned.tokens),
        [Tag::Symbol, Tag::Blank, Tag::Symbol, Tag::Eoc]
    );
}

#[test]
fn a_double_colon_is_one_token_and_a_single_colon_is_another() {
    assert_eq!(
        kinds(&scan_ok("::routine r")),
        [Tag::DColon, Tag::Symbol, Tag::Blank, Tag::Symbol, Tag::Eoc]
    );
    // No blank after the colon: a colon is not one of the four classes that
    // make a following blank significant.
    assert_eq!(
        kinds(&scan_ok("label: nop")),
        [Tag::Symbol, Tag::Colon, Tag::Symbol, Tag::Eoc]
    );
    // A label may be written as a literal too, which is why a label key is
    // not an interned symbol id. Measured: `'MiXeD': nop` gets rc 0 from
    // `rexxc`.
    assert_eq!(
        kinds(&scan_ok("'MiXeD': nop")),
        [Tag::Literal, Tag::Colon, Tag::Symbol, Tag::Eoc]
    );
    assert_eq!(literal_text(&scan_ok("'MiXeD': nop")), "MiXeD");
}

#[test]
fn the_comparison_operators_scan_to_their_own_kinds() {
    let cases = [
        ("a = b", Operator::Equal),
        ("a == b", Operator::StrictEqual),
        ("a \\= b", Operator::BackslashEqual),
        ("a \\== b", Operator::StrictBackslashEqual),
        ("a > b", Operator::GreaterThan),
        ("a >= b", Operator::GreaterThanEqual),
        ("a >> b", Operator::StrictGreaterThan),
        ("a >>= b", Operator::StrictGreaterThanEqual),
        ("a >< b", Operator::GreaterThanLessThan),
        ("a \\> b", Operator::BackslashGreaterThan),
        ("a \\>> b", Operator::StrictBackslashGreaterThan),
        ("a < b", Operator::LessThan),
        ("a <= b", Operator::LessThanEqual),
        ("a << b", Operator::StrictLessThan),
        ("a <<= b", Operator::StrictLessThanEqual),
        ("a <> b", Operator::LessThanGreaterThan),
        ("a \\< b", Operator::BackslashLessThan),
        ("a \\<< b", Operator::StrictBackslashLessThan),
        ("a & b", Operator::And),
        ("a && b", Operator::Xor),
        ("a | b", Operator::Or),
        ("a || b", Operator::Concatenate),
        ("a + b", Operator::Plus),
        ("a - b", Operator::Subtract),
        ("a * b", Operator::Multiply),
        ("a / b", Operator::Divide),
        ("a // b", Operator::Remainder),
        ("a % b", Operator::IntDiv),
        ("a ** b", Operator::Power),
        ("\\a", Operator::Backslash),
    ];
    for (text, op) in cases {
        assert_eq!(operators(&scan_ok(text)), [op], "{text}");
    }
    // Measured: `say 7 % 2` prints 3, `say 7 // 2` prints 1, `say 2 ** 3`
    // prints 8, `say \1` prints 0, `say 1 \= 2` prints 1 and
    // `say 1 \== 2` prints 1.
}

#[test]
fn the_alternative_logical_not_bytes_are_accepted() {
    // The interpreter takes 0xAA and 0xAC as alternative spellings of `\`
    // (`Scanner.cpp:1110`), which is why they are not error 13.1 the way
    // every other byte above 0x7F is.
    for byte in [0xAAu8, 0xACu8] {
        let source = ProgramSource::new(vec![byte, b'a']);
        let toks = scan(&source, ScanMode::Program).expect("scans").tokens;
        assert_eq!(kinds(&toks), [Tag::Operator, Tag::Symbol, Tag::Eoc]);
        assert_eq!(operators(&toks), [Operator::Backslash]);
    }
    // And the same three-way look-ahead applies.
    let source = ProgramSource::new(b"a \xac= b".to_vec());
    let toks = scan(&source, ScanMode::Program).expect("scans").tokens;
    assert_eq!(operators(&toks), [Operator::BackslashEqual]);
}

#[test]
fn every_token_span_lies_inside_the_source_and_is_ordered() {
    // A span is what `TRACE` and `SOURCELINE` slice, so a span that runs past
    // the text or backwards would be a silent corruption rather than a
    // failure. The one span that may overlap its neighbour is a blank made by
    // a line continuation: the interpreter records it after stepping to the
    // next line, so it covers that line's first byte.
    let text = "say 'a',\n'b' || abs(2.5) -- tail\n/* c */ x = .5\n";
    let source = ProgramSource::new(text.as_bytes().to_vec());
    let scanned = scan(&source, ScanMode::Program).expect("scans");
    let end = text.len();
    let mut previous_start = 0;
    for token in &scanned.tokens {
        assert!(token.span.start <= token.span.end, "{token:?}");
        assert!(token.span.end <= end, "{token:?} past {end}");
        assert!(
            token.span.start >= previous_start,
            "{token:?} went backwards"
        );
        previous_start = token.span.start;
    }
}

/// Every string over `alphabet` of length up to `max`, applied to `f`.
fn for_every_string(alphabet: &[u8], max: usize, mut f: impl FnMut(&[u8])) {
    let mut buffer = Vec::new();
    fn walk(alphabet: &[u8], max: usize, buffer: &mut Vec<u8>, f: &mut impl FnMut(&[u8])) {
        f(buffer);
        if buffer.len() == max {
            return;
        }
        for &byte in alphabet {
            buffer.push(byte);
            walk(alphabet, max, buffer, f);
            buffer.pop();
        }
    }
    walk(alphabet, max, &mut buffer, &mut f);
}

#[test]
fn scan_always_answers_with_tokens_or_an_error_number() {
    // The literal packers index by position and rely on their own validation
    // pass having accounted for every character, so a mis-ported bound would
    // be a panic rather than a wrong answer. `scan` must always answer, with
    // tokens or with an error number.
    //
    // The alphabet is the characters that steer the scanner: both quotes, both
    // literal markers, hex and non-hex digits, both whitespace kinds, both
    // continuation characters, the comment delimiters, and a line end.
    let steering = b"'\"xb4g10 \t-,/*\n";
    let mut count = 0;
    for_every_string(steering, 4, |bytes| {
        count += 1;
        let source = ProgramSource::new(bytes.to_vec());
        // Either outcome is fine. Not panicking is the property.
        let _ = scan(&source, ScanMode::Program);
    });
    assert!(count > 50_000, "the sweep actually ran: {count}");

    // And the same for the operator and punctuation characters, which drive
    // the multi-character look-ahead.
    let punctuation = b"=<>\\~:|&%+-*/[](),;.";
    let mut count = 0;
    for_every_string(punctuation, 3, |bytes| {
        count += 1;
        let source = ProgramSource::new(bytes.to_vec());
        let _ = scan(&source, ScanMode::Program);
    });
    assert!(count > 8_000, "the sweep actually ran: {count}");

    // The packers only run on a complete literal, and a complete one is at
    // least six characters, so they need their own sweep over the content.
    let nibbles = b"4g1 \t0";
    let mut count = 0;
    for_every_string(nibbles, 5, |bytes| {
        for marker in [b'x', b'b'] {
            count += 1;
            let mut text = vec![b'\''];
            text.extend_from_slice(bytes);
            text.push(b'\'');
            text.push(marker);
            let source = ProgramSource::new(text);
            let _ = scan(&source, ScanMode::Program);
        }
    });
    assert!(count > 15_000, "the sweep actually ran: {count}");

    // Arbitrary bytes, including the ones that are error 13.1 and the ones
    // that are an alternative logical not.
    for byte in 0u8..=255 {
        for prefix in [&b""[..], b"'", b"/*", b"a", b"1e", b"'41'"] {
            let mut bytes = prefix.to_vec();
            bytes.push(byte);
            let source = ProgramSource::new(bytes);
            let _ = scan(&source, ScanMode::Program);
        }
    }
}
