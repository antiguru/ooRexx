//! `TokenCursor`, which no test outside this crate can reach: it is
//! `pub(crate)`, so an integration test under `tests/` cannot name it.

use super::{Tag, Token, TokenCursor, TokenKind};

fn tok(kind: TokenKind) -> Token {
    Token { kind, span: 0..0 }
}

#[test]
fn a_cursor_visits_only_its_own_range() {
    let tokens = [
        tok(TokenKind::Eoc),
        tok(TokenKind::LeftParen),
        tok(TokenKind::RightParen),
        tok(TokenKind::Eoc),
    ];
    let mut cursor = TokenCursor::new(1..3);
    assert_eq!(cursor.peek(), Some(1));
    assert_eq!(cursor.advance(), Some(1));
    assert_eq!(cursor.advance(), Some(2));
    // The trailing Eoc at index 3 is outside the clause and unreachable.
    assert_eq!(cursor.advance(), None);
    assert_eq!(cursor.peek(), None);
    assert_eq!(cursor.position(), 3);
    assert_eq!(tokens[2].kind.tag(), Tag::RightParen);
}

#[test]
fn stepping_back_re_yields_the_same_token() {
    let mut cursor = TokenCursor::new(4..7);
    assert_eq!(cursor.advance(), Some(4));
    assert_eq!(cursor.advance(), Some(5));
    cursor.back();
    assert_eq!(cursor.advance(), Some(5));
}

#[test]
fn stepping_back_past_the_end_of_an_exhausted_range_still_works() {
    let mut cursor = TokenCursor::new(0..1);
    assert_eq!(cursor.advance(), Some(0));
    assert_eq!(cursor.advance(), None);
    // `pos` is at `range.end`, one past the last token; back must land on it.
    cursor.back();
    assert_eq!(cursor.peek(), Some(0));
}

#[test]
#[should_panic(expected = "TokenCursor::back before start")]
fn stepping_back_before_the_clause_is_a_parser_bug() {
    let mut cursor = TokenCursor::new(3..5);
    cursor.back();
}
