# c5 comment pass: `docs/classes.rs` to `docs/classes/methods.rs`

Rulings on every line `comments10.py` lists (`c5-comments.txt`) and on
`positional.py` (`c5-positional.txt`, no block listed).

(a) comments naming a moved unit: none listed. By hand, every moved name
unqualified across `rust/crates` and `rust/corpus` (`c5-unqualified.txt`,
command inside): outside `docs/classes.rs` and `docs/classes/` no line names
one; inside `docs/classes.rs` every hit is code (the import, `method_rows`,
and the test module's calls), none a comment.

(b) comments naming a touched file: none listed.

(c) positional words in the parent and destination:
* `docs/classes.rs:38`, `:83`, `:235`, `:546`: as ruled at c4 (`:39`,
  `:84`, `:236`, `:601` there), shifted. True.
* `docs/classes/methods.rs:81` ("Text after the `<xref/>`", `Listed`'s
  `trailing` field): a position in the XML. True.

The test module in `classes.rs` still exercises `names_of` and
`displayed_names`, now reached through the parent's import; its tests keep
their names (`docs::classes::tests::*`), and no test comment names the file
they live in. The `module is src/docs/classes.rs` header lines are
unchanged, as at c4.
