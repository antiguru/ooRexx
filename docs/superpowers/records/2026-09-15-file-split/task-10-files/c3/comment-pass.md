# c3 comment pass: `gate_table_c.rs` to `table_c/checks.rs`

Rulings on every line `comments10.py` lists (`c3-comments.txt`) and on
`positional.py` (`c3-positional.txt`, no block listed).

(a) comments naming a moved unit:
* `table_c/checks.rs:42` (`ENTRY_KINDS`'s doc: "The `entry` values
  [`class_probe_shape`] and [`check_edge_endpoints`] know how to read"):
  both moved into the same file with it, so both links resolve there
  (`c3-testdoc.txt`: no warning). True.

(b) comments naming a touched file: `support/mod.rs:12`,
`gate_walk/mod.rs:12`, `corpus/phase-5c.txt:11`, `corpus/docs/class-set.txt:29`
and `:38`, as ruled at c2. `README.md:95` no longer names
`gate_table_c.rs` (corrected at c2).

(c) positional words in the parent and destinations:
* `table_c/checks.rs:67` (`check_entry_kinds`: "a value this file knows how
  to read"): `ENTRY_KINDS`, which is that set, and its two readers
  `class_probe_shape` and `check_edge_endpoints` are all in
  `table_c/checks.rs` now, so "this file" is true of the file the sentence
  is in. True.
* `table_c/checks.rs:487` ("The `ArgUtil` assertion, carried here as a
  wiring row"): the function that carries it is here, and the table reports
  it as a wiring row. True.
* `table_c/checks.rs:247` ("rather than here", `run_probe`), `:313` ("the
  check below", `probe_set`), `:355` ("Returning quietly here",
  `check_probe_text`): each points inside its own function, which moved
  whole. True.
* `gate_table_c.rs:390`, `:470`, `:696`, `:919`, `:922`: inside the two
  tests, as ruled at c1. True.

By hand: `check_probe_text`'s failure message, a string literal and not a
comment, says "The derivation in gate_table_c.rs is the definition of this
program"; the derivation is in `table_c/probes.rs` since c2. Left (a runtime
message is behaviour) and raised as a concern in the report.

The probe narrowing in `table_c/probes.rs` (`ENTRY_MARKER`,
`DOCUMENTED_EDGE_MARKER`, `class_probe_entry_questions`,
`class_probe_class_questions`, `pub(crate)` to `pub(super)`) changes no
comment.
