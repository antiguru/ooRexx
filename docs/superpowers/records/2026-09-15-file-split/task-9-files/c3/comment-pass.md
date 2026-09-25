# c3 comment pass: `token.rs`'s keyword tables

Source: `c3-comments.txt` (comments7.py, widened word list, parent and
destination) and `c3-positional.txt`. The moved text is `KeywordSet`,
`Keywords`, their `impl` blocks, the plain comment introducing the tables
(parent lines 323-326, outside any unit, moved as `LINES:323-326`; its text
is `cmp`-identical at its destination, `c3-floating.txt`) and the table
constants, which sat together between `Token` and `ParseCtx`. Every hit, and
why it is still true:

* (a) `token/keywords.rs:76`, "since `index_of` returns a position": the
  tables' comment names `KeywordSet::index_of`, which moved into the same
  file with it.
* (b): none.
* (c), a position in Rexx source, in the token stream or in time, not in
  this file: `token.rs:38` (the start of the clause), `:172` (end of
  file), `:190` (immediately followed by `=`), `:246` (a blank following
  this token), `:283` (`ParseCtx::keywords`: interned before `scan` reads
  any source), `:293` (the end of its clause), `:311` (the end of the
  range); `token/keywords.rs:48` (`Keywords`: built before any source is
  read).
* (c) `token/keywords.rs:75`, "here the order is load-bearing": these Rust
  tables as against the C++ ones, and the comment moved with the tables
  it describes, directly above them as before.
* positional.py `token.rs:282-285` naming `Keywords` with "before":
  `ParseCtx::keywords`'s doc; "before" is time (before `scan` reads any
  source), and `Keywords` is named as a type.
* positional.py `token/keywords.rs:48-49` naming `spelling` and
  `token/keywords.rs:74-77` naming `position`: false matches on the English
  words ("spelling tables", "returns a position"), not `Operator::spelling`
  or `TokenCursor::position`.
* The module doc of `token.rs` stays true of the module `token`, which
  holds the keyword tables in its child `token::keywords`, re-exported as
  `token::{KeywordSet, Keywords}`.
