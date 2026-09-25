# c1 comment pass: `gate_table_c.rs` to `table_c/rows.rs`

Rulings on every line `comments10.py` lists (`c1-comments.txt`) and on
`positional.py` (`c1-positional.txt`, no block listed).

(a) comments naming a moved unit: none listed.

(b) comments naming a touched file:
* `rexx-exec/tests/support/mod.rs:12`, `rexx-parse/tests/gate_walk/mod.rs:12`:
  match the `mod.rs` tail and describe their own files. True.
* `corpus/phase-5c.txt:11`: names `gate_table_c.rs`'s
  `every_closed_phase_this_table_owns_rows_for_is_gated`, which stays in
  `gate_table_c.rs`. True.
* `corpus/docs/class-set.txt:29`, `:38`: derived from string literals in
  `rexx-extract/src/docs/classes.rs` (not comments, and re-derived by
  `extract_docs.rs`); they say `gate_table_c.rs` derives the instance-arm
  probe and reads the `method-owner` column. The derivation
  (`method_probe_text`) has not moved at c1, and `read_classes` reads the
  column from the `gate_table_c` test target's own module. True.
* `corpus/gate-tables/README.md:95`: names `gate_table_c.rs`'s
  `class_probe_text`, `edge_probe_text` and `method_probe_text`, none of
  which moves at c1. True at c1; c2 moves them (ruled there).

(c) positional words in the parent and destinations:
* `table_c/rows.rs:74` ("carries below its readbacks"): a position inside
  the Rexx probe, not in a file. True.
* `gate_table_c.rs:400` ("a value this file knows how to read"):
  `check_entry_kinds`, `ENTRY_KINDS`, `class_probe_shape` and
  `check_edge_endpoints` are all still in `gate_table_c.rs`. True.
* `gate_table_c.rs:749` ("rather than here"), `:815` ("the check below"),
  `:857` ("Returning quietly here"), `:1084` ("the filters below"), `:1164`
  ("before anything runs"), `:1390` ("every `say` below it"), `:1613`
  ("Said here"), `:1616` ("`unanswered` above"): each points inside its own
  function body, or into the probe or the report; nothing they point at
  moved. True.
* `gate_table_c.rs:1018` ("carried here as a wiring row"): the ArgUtil
  assertion is carried by this table. True.

By hand: the full path `crates/rexx-exec/tests/gate_table_c.rs`, which
`comments10.py`'s tails do not include, appears only in probe-text string
literals (`gate_table_c.rs` at BASE `:512`, `:615`, `:663`, `:682`, `:692`)
and in the committed probes under `corpus/gate-tables/` that those literals
derive (`files/path-pins-base.txt`). They are compared byte for byte with
the derivation, so they are not editable; they say the file re-derives the
probe on every run, which the test in it does. No comment corrected.
