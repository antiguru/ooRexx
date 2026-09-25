# c4 comment pass: `ir/compile.rs`'s test module

Source: `c4-comments.txt` (comments7.py) and `c4-positional.txt`.

* (b) `run/loops.rs:1364`, "A loop header's registers are allocated in
  the enclosing scope and released past the whole loop (`ir/compile.rs`)":
  that allocation is `compile`'s production code, which did not move.
* (c) `compile/tests.rs:145, 271, 318`, "refusals above": the refusal
  tests directly above each accepting test, in the same module, which
  moved whole and in order. `:298`, "invented here": the test it sits on.
* No comment outside the test module names one of its tests or helpers
  (`two_symbols`); no comment in `compile.rs` names the test module as
  being in this file. The `COMPILE_CALLS` comment names
  `golden_tests.rs`, which this commit does not touch.
