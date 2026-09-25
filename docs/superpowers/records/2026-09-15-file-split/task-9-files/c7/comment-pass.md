# c7 comment pass: `instruction.rs`'s PARSE grammar

Source: `c7-comments.txt` (comments7.py, widened word list, parent and
destination) and `c7-positional.txt`. The moved text is the `Inst` members
`parse_instruction_body`, `parse_option`, `parse_template` and
`trigger_position`, which sat together between `arg_list` and
`variable_list`. Every hit, and why it is still true:

* (a): none.
* (b): the same seven `rexx-exec` comments as c5's and c6's, naming
  `instruction.rs` with `message`, `if_instruction`, `numeric` and
  `trace`, none of which moved.
* (c) in `instruction.rs`: the same comments, with the same text, as c6's
  list (`c7-parent-hits-vs-c6.txt`: no difference with line numbers
  dropped), so c6's rulings, which are c5's, hold; none of them points at
  the PARSE members.
* (c) in `instruction/parse.rs`: none.
* positional.py: nothing.
* The module doc of `instruction.rs` stays true of the module
  `instruction`, whose PARSE grammar is now in `instruction::parse`.
