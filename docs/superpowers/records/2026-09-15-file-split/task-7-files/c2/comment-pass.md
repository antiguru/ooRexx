# c2 comment pass: `builtin/convert.rs`'s test module

Source: `c2-comments.txt` (comments7.py) and `c2-positional.txt`.

* (a) `convert.rs:733`, "the ordering
  `the_call_layer_is_checked_before_the_operation_layer` pins": names the
  test, not a file. The test moved with its name and fully qualified name
  unchanged (`builtin::convert::tests::...`), and still pins that ordering.
* (a) `convert/tests.rs:62, 67, 73, 84`: intra-doc links to `call_at`,
  `call`, `answer`, `raised`, each in the same module as before
  (`builtin::convert::tests`).
* (c) The positional words inside `convert/tests.rs` (lines 93, 94, 135,
  187, 188, 285, 288, 553, 628, 635, 928, 1036, 1049): "below" and "above"
  refer either to byte values (`0x80`, `0x20`, one tenth) or to code in
  the same test or module, which moved whole and in order; "here" is the
  test it sits on.
* positional.py: `convert.rs:971-975`, "the start/end shortcut below",
  "leave the loop immediately": production code about production code; no
  production line moved.
* No comment in `convert.rs` names the test module as being in this file.
