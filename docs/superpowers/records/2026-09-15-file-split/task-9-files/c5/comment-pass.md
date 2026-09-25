# c5 comment pass: `instruction.rs`'s loop header

Source: `c5-comments.txt` (comments7.py, widened word list, parent and
destination) and `c5-positional.txt`. The moved text is the `Inst` members
`create_loop` through `loop_conditional` (parent lines 788-1161 at
`4b7db8f03`), which sat together between `end_name` and `address`. Every
hit, and why it is still true:

* (a): none. No comment outside the moved text names a moved member in
  backticks. (`instruction/tests.rs:1949` names `for_and_conditional` and
  `:688` sits in a test named after `loop_conditional`; neither states a
  place, and both members keep their names.)
* (b), comments in `rexx-exec` naming `instruction.rs` with a member:
  `run.rs:1164` (`message`), `run/indent.rs:279`, `run/tests/indent.rs:265`
  and `:300` (`if_instruction`), `run/settings.rs:73` and `:326`
  (`numeric`), `run/settings.rs:99` (`trace`). None of those members moved;
  each is still in `instruction.rs`.
* (c), a position in Rexx source or clause order, not in this file:
  `instruction.rs:179`, `:181`, `:182` (THEN tested ahead of the label and
  assignment tests: order of the checks in `parse_instruction`, which did
  not move), `:187`, `:190`, `:314`, `:530`, `:607`, `:678`, `:808`,
  `:809`, `:1033`, `:1202` (a value after the condition name), `:1431`,
  `:1462`, `:1463`, `:1985`, `:2018` (raises before: time), `:2032` (after
  an `OTHERWISE`), `:2063`-`:2066` (earlier in the Rexx body, before it
  reaches the calls: time); `loops.rs:147`, `:214`, `:316`, `:372` (tokens
  that follow or come after, in the clause).
* (c), "here" meaning the code it sits in, all unmoved or moved whole:
  `instruction.rs:189`, `:201` (`parse_instruction`), `:425` (a Rexx
  literal `"here"`), `:488` (`message`), `:578` (these variants),
  `:1204`, `:1229` (`raise`), `:1904` (`variable_list`).
* (c), a pointer to text in the same function, which did not move:
  `instruction.rs:765` ("a real one is consumed above": the THEN test in
  `parse_instruction`, still above `keyword` in this file) and `:769` ("an
  arm above": `keyword`'s own match arms, including `KW_DO` and `KW_LOOP`,
  which still call `create_loop`).
* (c), a pointer into the C++: `instruction.rs:1453` and `:1521` ("the
  three constructors above it", "the two constructors above it", in
  `InstructionParser.cpp`), `:2027` (the grammar a WHEN follows).
* (c) `loops.rs:108`, "This test comes BEFORE the WITH one": the order of
  two tests inside `create_loop`, which moved whole.
* positional.py `loops.rs:108-110` naming `expr`: the Rexx syntax
  `DO name OVER expr`, not `Inst::expr`.
* The module doc of `instruction.rs`, "The instruction grammar: one clause
  in, one instruction out", stays true of the module `instruction`, whose
  loop header is now in its child `instruction::loops`.
