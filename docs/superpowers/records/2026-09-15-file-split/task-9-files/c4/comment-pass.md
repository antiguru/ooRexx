# c4 comment pass: `token.rs`'s parse context and token cursor

Source: `c4-comments.txt` (comments7.py, widened word list, parent and
destination) and `c4-positional.txt`. The moved text is `ParseCtx`,
`TokenCursor` and `TokenCursor`'s `impl` block, which ended `token.rs`
before its `mod tests;`. Every hit, and why it is still true:

* (a): none listed. Read by hand as well (`/bin/grep -rn
  '//.*`\(ParseCtx\|TokenCursor\)' rust/crates`, outside the new file), two
  comments name the moved types unqualified: `clause.rs:21`, "Index range
  into the `ParseCtx::tokens` slice", which names the field, not a place;
  and `token/tests.rs:1`, "`TokenCursor`, which no test outside this crate
  can reach: it is `pub(crate)`", still true: the struct is `pub(crate)`
  and `token.rs` re-exports it `pub(crate)`, so `tests.rs`'s
  `use super::TokenCursor` reaches it and nothing outside the crate can.
* (b): none.
* (c), a position in Rexx source, in the token stream or in time, not in
  this file: `token.rs:35` (the start of the clause), `:169` (end of
  file), `:187` (immediately followed by `=`), `:243` (a blank following
  this token); `token/cursor.rs:40` (`ParseCtx::keywords`: interned before
  `scan` reads any source), `:50` (the end of its clause), `:68` (the end
  of the range).
* positional.py `token.rs:35-37` naming `start` and `token.rs:169`
  naming `end`: false matches on the English words ("the start of the
  clause", "end of file"), not `TokenCursor::start` or `TokenCursor::end`.
  `token/cursor.rs:39-42` naming `spelling`: the same ("reserved
  spelling"), not `Operator::spelling`.
* The module doc of `token.rs` stays true of the module `token`, which
  holds the context every `parse_*` function is handed in its child
  `token::cursor`, re-exported as `token::{ParseCtx, TokenCursor}`.
