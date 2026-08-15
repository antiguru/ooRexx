### Task 2: witness the two `settle_block_indent` arguments

Review finding **I4**. `e74780054` added two `settle_block_indent` calls that are part of the fix,
are observable against the oracle, and are pinned by nothing. At `56d9d1c86` they are
`crates/rexx-exec/src/run.rs:6175` (`it.settle_block_indent(true, do_indent)`) and
`crates/rexx-exec/src/run.rs:6244` (`it.settle_block_indent(false, do_indent)`). Task 1 will have
moved those lines; find them by the surrounding `Simple` arm, not by number.

Both arguments are **correct**. The reviewer verified three `trace r` programs (a plain `DO` with a
body, an empty `DO` under a double requeue, nested plain `DO`s) match the oracle byte for byte on
all three descriptors and both engines, and that the pre-fix binary got the indent wrong on two of
them. What is missing is a test that fails when either argument is flipped.

Measured by the reviewer, each flip builds clean and leaves `corpus`, `trace_oracle`, `trace_indent`
and all of `ir_dual` green:

* first call, `true` becomes `false`: a handler delivered at a plain `DO`'s header clause loses two
  columns in its activation, `15 *-*     h2:` becomes `15 *-*   h2:`.
* second call, `false` becomes `true`: the handler delivered at `END`'s clause gains two columns.

#### Steps

1. **Re-measure both flips at the tree you find** (after Task 1) and report the transcripts. Do not
   assume the two columns above survived Task 1's change.

2. **Add a witness per argument.** Two distinct witnesses, one per call, each of which reddens when
   its own argument is flipped and stays green when the other is. The corpus program
   `corpus/lang/do_clause_boundaries.rex` deliberately carries no `TRACE`, which is right for its
   own subject, so the witness belongs in an `ir_dual_cases` row or a `trace_oracle` case rather than
   in that corpus program.

3. **Prove the selectivity by mutation**, not by assertion. Flip each argument in turn, run with
   `--no-fail-fast` under `memcap 8G`, and report every test that caught each flip. A single run
   that stops at the first failing assert does not measure selectivity.

4. **Gates** as in Global Constraints.

---

