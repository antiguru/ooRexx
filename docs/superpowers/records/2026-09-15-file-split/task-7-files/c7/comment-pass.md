# c7 comment pass: `builtin/string.rs`'s `mod tests`

Source: `c7-comments.txt` (comments7.py) and `c7-positional.txt`. Ruling
(team-lead, 2026-09-25): move it, and record the `phase-4c.txt` reading.

* (b) `corpus/phase-4c.txt:192` (read-only): "measured, removing the
  overrun reddens one test in the workspace and it is the transcribed table
  in builtin/string.rs". The test is
  `builtin::string::tests::the_padding_builtins_answer_the_oracles_own_bytes`,
  now in `builtin/string/tests.rs`. The sentence stays true read as "the
  table transcribed in builtin/string.rs": the transcription is
  `find_forward`'s doc (and the padding builtins' docs), which stay in
  `string.rs`, and the test keeps its fully qualified name, which is how a
  reader finds it. The file is a dated measurement record; not edited.
* The test's own doc, "The oracle transcripts each of these lines came
  from are in the function's own doc comment; this is the same table run
  through `dispatch`": plain prose, no intra-doc link. The functions' doc
  comments stay in `string.rs`, and `dispatch` is still the helper it runs
  through (`use super::super::dispatch`, same module path).
* (a) `string/tests.rs:29, 44`: `call` is in the same module.
* (c) `string/tests.rs:208, 211, 238`: "the overrun above", "here":
  the assertions directly above in the same test, which moved whole.
  `:709`, "every 93.9xx here": the test it sits on. `:764`, "below 0x20":
  a byte value. `:806`, "A raiser this module owns": the raisers the
  `builtin::string` tests exercise; the test was in the same module
  before and after, so the phrase means what it meant.
* positional.py: `string.rs:107-109` ("the scan below") and `:977-983`
  ("every other sized result in this module") are production code about
  production code; no production line moved.
* `tests/builtin_status.rs:494` and `corpus/oracle-crashes.txt:78` name
  production code in `string.rs` (its dispatch rows, `find_forward`'s
  doc), which did not move.
