# c1 comment pass: `builtin/numeric.rs`'s test module

Source: `c1-comments.txt` (comments7.py) and `c1-positional.txt`. Every line
listed there, with why it is still true after the move.

* (a) `numeric/tests.rs:76`, "[`call`], for the cases ...": `call` moved in
  the same module, `builtin::numeric::tests`, so the link names what it named.
* (b) `corpus/lang/string_extremes.rex:14`, "`integer_object`'s doc in
  builtin/numeric.rs": `integer_object` is production code and did not move
  (`numeric.rs:308`).
* (c) The positional words inside `numeric/tests.rs` (lines 20, 316, 461,
  490, 516, 606, 693: "here", "not here", "later", "above", "below") each
  refer to the test they sit in or its neighbours. The module moved whole,
  in order, so every neighbour is where it was.
* positional.py: `numeric.rs:349-355`, "the one place in this file where
  the answer depends on which *representation* the target has": a claim
  about production code; removing the tests from the file cannot add a
  second such place.
* No comment in `numeric.rs` names the test module or a test.
