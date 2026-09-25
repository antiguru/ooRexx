# c2 comment pass: `gate_table_c.rs` to `table_c/probes.rs`

Rulings on every line `comments10.py` lists (`c2-comments.txt`) and on
`positional.py` (`c2-positional.txt`, no block listed).

(a) comments naming a moved unit:
* `table_c/probes.rs:110` ("The marker is spelled once, in
  `DOCUMENTED_EDGE_MARKER`, because `check_documented_edge` reads the
  oracle's answer back by it"): both names are unchanged and the constant
  is still defined once; `check_documented_edge` (still in
  `gate_table_c.rs`) reads it through the parent's import. True.

(b) comments naming a touched file:
* `support/mod.rs:12`, `gate_walk/mod.rs:12`, `corpus/phase-5c.txt:11`: as
  at c1. True.
* `corpus/docs/class-set.txt:29`, `:38`: derived from string literals in
  `rexx-extract`'s `docs/classes.rs` (re-derived by `extract_docs.rs`);
  "`gate_table_c.rs` derives the instance-arm probe" is now done by
  `method_probe_text` in `table_c/probes.rs`, a module of the
  `gate_table_c` test target that the target's test calls. Not a comment,
  and not editable without changing a literal and re-deriving the file; left,
  and raised as a concern in the report.
* `corpus/gate-tables/README.md:95`: "`gate_table_c.rs`'s
  `class_probe_text`, `edge_probe_text` and `method_probe_text` are the
  definition ...". All three moved to `tests/table_c/probes.rs` in this
  commit, so the file it names no longer holds them. **Corrected in this
  commit**: `gate_table_c.rs`'s becomes `tests/table_c/probes.rs`'s, one
  line, no re-wrap. Declared in `c2/extra` and checked by
  `extra_edits10.py` (`c2-extra-edits.txt`): HEAD's text with that one
  substitution equals the working text.

(c) positional words in the parent and destinations:
* `gate_table_c.rs:331` ("a value this file knows how to read",
  `check_entry_kinds`): `ENTRY_KINDS`, `class_probe_shape`,
  `check_entry_kinds` and `check_edge_endpoints` are still in
  `gate_table_c.rs`. True.
* `:552`, `:618`, `:660`, `:821`, `:887`, `:967`, `:1193`, `:1416`,
  `:1419`: the lines ruled at c1 (`:749`, `:815`, `:857`, `:1018`, `:1084`,
  `:1164`, `:1390`, `:1613`, `:1616` there), each shifted by the move and
  pointing inside its own function, probe or report. True.

By hand: the probe-text string literals that name
`crates/rexx-exec/tests/gate_table_c.rs` moved verbatim into
`table_c/probes.rs` (they are derivation output compared with the committed
probes, and cannot change). They say that file "re-derives this file on
every run", which its test still does, by calling the moved functions.
Raised as a concern in the report with the class-set lines.
