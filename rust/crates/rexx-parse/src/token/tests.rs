//! `TokenCursor`, which no test outside this crate can reach: it is
//! `pub(crate)`, so an integration test under `tests/` cannot name it.
//!
//! Three tests here exercised `TokenCursor::back` and went with the method:
//! the grammar peeks rather than rewinding, so nothing called it.

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
