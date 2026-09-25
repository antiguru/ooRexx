# c6 comment pass: `instruction.rs`'s ADDRESS grammar

Source: `c6-comments.txt` (comments7.py, widened word list, parent and
destination) and `c6-positional.txt`. The moved text is the `Inst` members
`address`, `address_with`, `output_option` and `redirect_target`, which sat
together between `end_name` and `numeric`. Every hit, and why it is still
true:

* (a): none.
* (b): the same seven `rexx-exec` comments as c5's, naming `instruction.rs`
  with `message`, `if_instruction`, `numeric` and `trace`, none of which
  moved.
* (c) in `instruction.rs`: the same comments c5's pass ruled on, with the
  same text, less the two that moved into `address.rs`
  (`c6-parent-hits-vs-c5.txt` compares the two lists with line numbers
  dropped: the only difference is those two lines). Each ruling of c5's
  holds, since none of those comments points at the ADDRESS members. One
  correction to c5's pass: it did not list `instruction.rs:576` (at c5),
  "an optional expression to the end of the clause" in `keyword`'s SAY,
  PUSH and QUEUE comment, which is a position in the Rexx clause.
* (c) `instruction/address.rs:44`-`45`, "what follows is either that
  keyword or the end of the clause": tokens of the ADDRESS clause, inside
  `address`, which moved whole.
* positional.py `address.rs:44-47` naming `command` and `keyword`: the
  English words ("The command expression", "that keyword"), not
  `Inst::command` or `Inst::keyword`.
* The module doc of `instruction.rs` stays true of the module
  `instruction`, whose ADDRESS grammar is now in `instruction::address`.
