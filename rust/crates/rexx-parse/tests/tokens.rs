use rexx_parse::{Keywords, SymbolTable, TokenKind};

#[test]
fn spellings_differing_only_in_case_are_one_symbol() {
    let mut symbols = SymbolTable::default();
    let a = symbols.intern("abc");
    let b = symbols.intern("ABC");
    let c = symbols.intern("aBc");
    assert_eq!(a, b);
    assert_eq!(a, c);
    assert_eq!(symbols.len(), 1);
}

#[test]
fn a_symbols_name_is_the_upcased_spelling_not_the_source_spelling() {
    // Rexx folds symbol case: `abc = 1` then `say aBc` prints 1. The upcased
    // form is the identity, so that is what comes back out; the source
    // spelling is recovered from the token's span instead.
    let mut symbols = SymbolTable::default();
    let id = symbols.intern("aBc");
    assert_eq!(symbols.name(id), "ABC");
}

#[test]
fn a_compound_tail_is_part_of_the_one_symbol() {
    // The C++ scans the whole dotted name as a single token and resolves the
    // tail at run time, so `stem.i.j` is one symbol and not three.
    let mut symbols = SymbolTable::default();
    let whole = symbols.intern("stem.i.j");
    assert_eq!(symbols.name(whole), "STEM.I.J");
    assert_ne!(whole, symbols.intern("stem"));
    assert_eq!(symbols.len(), 2);
}

#[test]
fn distinct_spellings_get_distinct_ids_in_first_seen_order() {
    let mut symbols = SymbolTable::default();
    let first = symbols.intern("alpha");
    let second = symbols.intern("beta");
    assert_ne!(first, second);
    // Re-interning must not shift an existing id.
    assert_eq!(symbols.intern("ALPHA"), first);
    assert_eq!(symbols.name(second), "BETA");
    assert_eq!(symbols.len(), 2);
}

#[test]
fn keyword_tables_keep_the_cpp_table_order() {
    // An entry's position IS its meaning to the caller, so the position is
    // what this pins, not merely membership.
    let mut symbols = SymbolTable::default();
    let keywords = Keywords::new(&mut symbols);

    assert_eq!(keywords.instructions.len(), 35);
    assert_eq!(keywords.sub_keywords.len(), 50);
    assert_eq!(keywords.conditions.len(), 12);
    assert_eq!(keywords.parse_options.len(), 10);
    assert_eq!(keywords.directives.len(), 9);
    assert_eq!(keywords.sub_directives.len(), 40);

    // First, last and one interior entry of the instruction table.
    assert_eq!(
        keywords.instructions.index_of(symbols.intern("address")),
        Some(0)
    );
    assert_eq!(
        keywords.instructions.index_of(symbols.intern("say")),
        Some(28)
    );
    assert_eq!(
        keywords.instructions.index_of(symbols.intern("when")),
        Some(34)
    );
    // `ANNOTATE` heads the directive table but is not an instruction.
    assert_eq!(
        keywords.directives.index_of(symbols.intern("annotate")),
        Some(0)
    );
    assert_eq!(
        keywords.instructions.index_of(symbols.intern("annotate")),
        None
    );
}

#[test]
fn one_spelling_in_two_tables_keeps_each_tables_own_index() {
    // `VALUE` is entry 46 of subKeywords and entry 7 of parseOptions. The
    // tables share one SymbolId and must not share a position.
    let mut symbols = SymbolTable::default();
    let keywords = Keywords::new(&mut symbols);
    let value = symbols.intern("VALUE");
    assert_eq!(keywords.sub_keywords.index_of(value), Some(46));
    assert_eq!(keywords.parse_options.index_of(value), Some(7));
    assert_eq!(keywords.instructions.index_of(value), None);
}

#[test]
fn interning_a_keyword_from_source_reuses_the_pre_interned_id() {
    // This is the point of pre-interning: the positional keyword test in a
    // later task is an integer comparison, not a case-insensitive compare.
    let mut symbols = SymbolTable::default();
    let keywords = Keywords::new(&mut symbols);
    let before = symbols.len();
    let from_source = symbols.intern("Say");
    assert_eq!(symbols.len(), before, "no new symbol was added");
    assert_eq!(keywords.instructions.index_of(from_source), Some(28));
}

#[test]
fn a_tag_drops_the_payload_but_not_the_class() {
    let mut symbols = SymbolTable::default();
    let id = symbols.intern("x");
    let symbol = TokenKind::Symbol {
        id,
        class: rexx_parse::SymbolClass::Variable,
    };
    let other = TokenKind::Symbol {
        id: symbols.intern("y"),
        class: rexx_parse::SymbolClass::Variable,
    };
    // Two different symbols share a tag, so a tag comparison cannot stand in
    // for an identity comparison.
    assert_eq!(symbol.tag(), other.tag());
    assert_ne!(symbol, other);
    assert_ne!(
        symbol.tag(),
        TokenKind::Literal {
            value: Box::new([])
        }
        .tag()
    );
}

#[test]
fn blank_significance_depends_on_the_class_of_the_token_before_it() {
    let mut symbols = SymbolTable::default();
    let symbol = TokenKind::Symbol {
        id: symbols.intern("f"),
        class: rexx_parse::SymbolClass::Variable,
    };
    assert!(symbol.makes_blank_significant());
    assert!(
        TokenKind::Literal {
            value: Box::new([])
        }
        .makes_blank_significant()
    );
    assert!(TokenKind::RightParen.makes_blank_significant());
    assert!(TokenKind::RightBracket.makes_blank_significant());
    // An operator, a left paren and a clause end all suppress it.
    assert!(!TokenKind::Operator(rexx_parse::Operator::Concatenate).makes_blank_significant());
    assert!(!TokenKind::LeftParen.makes_blank_significant());
    assert!(!TokenKind::Eoc.makes_blank_significant());
    assert!(!TokenKind::Comma.makes_blank_significant());
}
