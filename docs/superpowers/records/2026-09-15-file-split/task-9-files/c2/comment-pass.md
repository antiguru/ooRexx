# c2 comment pass: `token.rs`'s symbol interning

Source: `c2-comments.txt` (comments7.py, widened word list, parent and
destination) and `c2-positional.txt`. The moved text is BASE `token.rs:47`-`110`
(`SymbolId`, `SymbolTable` and their `impl` blocks), which sat between
`ParseError` and `Operator`. Every hit, and why it is still true:

* (a) `token.rs:156`, `TokenKind::Symbol`'s doc, "the identity is the
  `SymbolId`": names the type, not a place; `SymbolId` is still reached as
  `token::SymbolId` through the re-export.
* (a) `token/symbols.rs:12`: the new module doc.
* (a) `token/symbols.rs:18`, `:23`, `:25`, `:33`: the moved docs of
  `SymbolId`, `SymbolId::index` and `SymbolTable`, naming each other and
  `SymbolTable::len`, all of which moved together into this file; `:33`
  also names `ProgramSource` and `Program`, which are reached by name, not
  by position.
* (b): none.
* (c), a position in Rexx source, in the token stream or in time, not in
  this file: `token.rs:36` (the start of the clause), `:170` (end of
  file), `:188` (an operator immediately followed by `=`), `:244` (a blank
  following this token), `:297` and `:509` (`scan` builds the tables
  before it reads any source), `:519` (the end of its clause), `:537` (the
  end of the range).
* (c) `token.rs:324`, "here the order is load-bearing": "here" is these
  Rust tables as against the C++ ones; the tables are still in `token.rs`
  at this commit (c3 moves them, with this comment).
* (c) in `token/symbols.rs`: none.
* positional.py `token.rs:36-38`, "start of", naming `name`: a false
  match. The comment is `ParseError::byte`'s doc, "The name reads like the
  latter", where "name" is the field's name, not `SymbolTable::name`.
* The module doc of `token.rs`, "Tokens, symbol interning, the keyword
  tables, and the context every `parse_*` function is handed", stays true
  of the module `token`, which still holds symbol interning, in its child
  `token::symbols`, re-exported as `token::{SymbolId, SymbolTable}`.
