//! `TokenCursor`, which no test outside this crate can reach: it is
//! `pub(crate)`, so an integration test under `tests/` cannot name it.
//!
//! Three tests here exercised `TokenCursor::back` and went with the method:
//! the grammar peeks rather than rewinding, so nothing called it.

use super::{SymbolTable, Tag, Token, TokenCursor, TokenKind};

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

/// `SymbolId::index` promises a dense, zero-based index into the table that
/// interned it, which is what lets a caller use a `Vec` instead of a
/// `HashMap<SymbolId, _>` on the variable-lookup path.
///
/// The guarantee is a property of `intern` -- it assigns `SymbolId(names
/// .len())` before pushing -- so it is pinned here rather than left implicit.
/// A change that made ids sparse, or based them on anything but insertion
/// order, would still satisfy every other test in this crate and would fail
/// only inside a consumer's `Vec`, as either a panic or a silently wrong slot.
#[test]
fn symbol_ids_are_dense_and_zero_based() {
    let mut symbols = SymbolTable::default();
    let names = ["ALPHA", "beta", "Gamma", "DELTA", "epsilon"];

    let ids: Vec<_> = names.iter().map(|n| symbols.intern(n)).collect();

    let indices: Vec<usize> = ids.iter().map(|id| id.index()).collect();
    assert_eq!(indices, (0..names.len()).collect::<Vec<_>>());

    // `len` is what a caller sizes its `Vec` by, so it has to be exactly one
    // past the largest index rather than merely at least that.
    assert_eq!(symbols.len(), names.len());

    // Case-insensitive interning must not consume an index: re-interning any
    // spelling returns the id the first one got, so the range stays dense.
    assert_eq!(symbols.intern("alpha").index(), 0);
    assert_eq!(symbols.intern("BETA").index(), 1);
    assert_eq!(symbols.len(), names.len());

    // The index addresses the same entry `name` does, which is the property
    // that makes a parallel `Vec` line up with the table at all.
    for (id, expected) in ids.iter().zip(names) {
        assert_eq!(symbols.name(*id), expected.to_ascii_uppercase());
        assert_eq!(id.index(), ids[id.index()].index());
    }
}
