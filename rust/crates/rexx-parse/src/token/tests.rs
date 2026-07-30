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
    let mut visited = Vec::new();
    while let Some(i) = cursor.advance() {
        visited.push(tokens[i].kind.tag());
    }
    // The cursor yields indices and never reads the token array, so the check
    // that matters is that its indices SELECT the intended tokens: the two
    // inside the range and neither Eoc. An earlier revision ended with
    // `assert_eq!(tokens[2].kind.tag(), Tag::RightParen)`, which compares the
    // array literal with itself and was the only use of `tokens` at all.
    assert_eq!(visited, [Tag::LeftParen, Tag::RightParen]);
    // The trailing Eoc at index 3 is outside the clause and unreachable.
    assert_eq!(cursor.peek(), None);
    assert_eq!(cursor.position(), 3);
}
