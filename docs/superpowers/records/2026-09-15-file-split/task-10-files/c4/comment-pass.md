# c4 comment pass: `docs/classes.rs` to `docs/classes/coverage.rs`

Rulings on every line `comments10.py` lists (`c4-comments.txt`) and on
`positional.py` (`c4-positional.txt`, no block listed).

(a) comments naming a moved unit: none listed. By hand,
`/bin/grep -rn 'coverage_of\|unconstructible_index\|strip_tags\|collapse(' rust/crates rust/corpus`
outside the two files finds no comment naming any of the four
(`c4-unqualified.txt`).

(b) comments naming a touched file: every hit matches the `coverage.rs`
tail and names `rexx-exec/tests/coverage.rs`, a different file in another
crate (`collect_stress.rs:31`, `loud.rs:130`, `owners.rs:12`, `:291`,
`:510`, `:541`, `corpus/phase-5b.txt:11`, `:13`). True.

(c) positional words in the parent and destination:
* `docs/classes.rs:39` ("beside the hierarchy list"): a position in
  `provide.xml`. True.
* `:84`, `:236` ("below its readbacks", "below the probe's readbacks"):
  positions in a Rexx probe. True.
* `:601` ("following the section's line"): inside `method_rows`, which did
  not move. True.
* `:635` ("Text after the `<xref/>`"): a position in the XML. True.

The `module is src/docs/classes.rs` header line both committed row files
carry comes from `class_set_header` and `class_methods_header`, which stay
in `classes.rs` with the entry points `docs.rs` calls; `extract_docs.rs`
passed in instrument 4 and both committed files are unchanged (the
commit touches nothing under `rust/corpus/`).
