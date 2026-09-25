# c5 comment pass: `ir/compile.rs`'s op-stream invariants

Source: `c5-comments.txt` (comments7.py) and `c5-positional.txt`, plus
`/bin/grep -rn -a` over `rust/` (`.rs`, `.md`, `.txt`) for every moved
function's name: only `ir/compile.rs`, `compile/invariants.rs`,
`compile/tests.rs` and `ir/drive.rs` name them.

* (a) `ir/drive.rs:568`, "`compile::assert_region_ops_name_their_clause` is
  what makes the two the same instruction": the path named the function in
  `compile`, where it no longer is, and `compile`'s import of it is private,
  so the path does not reach it from `drive`. **Corrected in this commit**
  to `compile::invariants::assert_region_ops_name_their_clause`, one token
  inserted on one line, no re-wrap (rule in `rules`, checked by
  other_edits.py). The rest of the sentence (the check makes the two the
  same instruction) is unchanged and true: `compile` still calls it on
  every stream.
* (b) `run/loops.rs:1364`, "(`ir/compile.rs`)": register allocation, which
  stays in `compile.rs`.
* (c) and positional.py: "immediately behind" / "immediately followed by"
  in six moved docs describe op positions in a stream, not file positions.
* `compile/tests.rs` imports the checks as `super::assert_...`; that path
  still resolves, through `compile.rs`'s `use invariants::{...}`, so the
  test file is unedited.
* `compile.rs`'s module doc ("the one pass that turns a body into a
  [`Chunk`]") stays true: the pass is still there; the checks it runs on
  its output are in its child.
* `assert_analysis_only_narrows` stays in `compile.rs`: it checks the
  `trace_flow` analysis against the plan, not the op stream, and runs only
  under `debug_assertions`. Its doc does not name the moved checks.
