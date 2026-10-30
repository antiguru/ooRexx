**CLOSED 2026-10-30 by Phase 9d Task 8.8e1 (1d8b03220).**

# Three pointers name a POS-window doc the comment prune deleted

Found by file-split Task 7's review (2026-09-25). `find_forward`'s doc in
`rust/crates/rexx-exec/src/builtin/string.rs` was cut from about 40 lines to
two by `ac15d1042` (the comment prune). Three places still point at the
mechanism it described: `rust/corpus/oracle-crashes.txt:78` (read-only),
`builtin/string.rs:123` ("the doc above describes") and
`builtin/string/tests.rs:185`. Since the corpus is read-only, restore the
mechanism to `find_forward`'s doc from `git show ac15d1042^:<path>` (comment
only, no code), minimal, then check the two in-crate pointers read true.
- `block/references.rs:190`, `:197` name `for_each_variable_in_expr`, which does not exist (stale since before file-split BASE, was `block.rs:769`). Fix alongside the POS-window doc item.


## Task 8.8e1 evidence

Restored POS-window and NUL-overrun mechanism from the pre-prune source, and replaced the stale expression visitor name with visit_expr. See `.superpowers/sdd/2026-10-18-phase-9d/task-8.8e1-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/records/e1/`.
