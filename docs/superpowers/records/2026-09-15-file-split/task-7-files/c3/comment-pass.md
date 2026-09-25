# c3 comment pass: `builtin/datetime.rs`'s test module

Source: `c3-comments.txt` (comments7.py) and `c3-positional.txt`.

* (a) Every hit is inside `datetime/tests.rs` and names a test or helper of
  the same module (`failure`, `date_366_is_valid_only_in_a_leap_year`,
  `every_date_output_letter_is_accepted`,
  `time_r_resets_relative_to_the_last_reset_not_program_start`,
  `the_empty_option_renders_as_the_empty_string_not_a_placeholder`,
  `output()`): each moved with the module, keeps its name, and the
  intra-doc links resolve in the same module `builtin::datetime::tests`.
* (c) and positional.py: every "above", "below", "earlier", "later",
  "here" inside `datetime/tests.rs` refers to the test it sits in, a
  sibling test, or a byte range (`0x20`). The module moved whole and in
  order, so every sibling is where it was.
* No comment in `datetime.rs` names a test, or the test module as being in
  this file.
* The one test-module intra-doc warning at BASE (`tests_links.sh`,
  datetime.rs:1493, inside the test module) is expected to be the same
  warning located in `datetime/tests.rs`; see `c3-tests-links.txt`.
