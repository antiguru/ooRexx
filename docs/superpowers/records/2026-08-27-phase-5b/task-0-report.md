# Task 0 report: `corpus/phase-5b.txt` and its wiring

## What was built

* `rust/corpus/phase-5b.txt` created, empty (header only), in `phase-5a.txt`'s
  original shape (matching the file as Task 1 of Phase 5a committed it at
  `3fa2aac1c8`, before any 5a task had appended a program). Header text:
  states the union it stands in (`phase-4a.txt`, `phase-4b.txt`,
  `phase-4c.txt`, `phase-5a.txt`), the one-path-per-line format read by the
  same `read_subset`, and that Task 0 only wires the file in and adds no
  corpus program of its own.

* `"phase-5b.txt"` appended to the four guarded `SUBSET_FILES` literals:
  * `rust/crates/rexx-exec/tests/corpus.rs` (was `:657`, `phase-5a.txt` row)
  * `rust/crates/rexx-exec/tests/coverage.rs` (was `:675`)
  * `rust/crates/rexx-exec/tests/collect_stress.rs` (was `:141`)
  * `rust/crates/rexx-exec/tests/ir_dual.rs` (was `:1185`)

* `"phase-5b.txt"` appended to the unguarded literal in
  `rust/crates/rexx-exec/tests/trace_oracle.rs`
  (`every_live_witness_emits_its_prefix_and_is_run_by_the_corpus`, was
  `:688`).

* `EXPECTED_SUBSET_5B` added beside `EXPECTED_SUBSET_5A` in
  `rust/crates/rexx-exec/tests/coverage.rs`, as an empty slice (the file is
  committed empty), with a new test `phase_5b_subset_matches_the_committed_list`
  mirroring `phase_5a_subset_matches_the_committed_list`.

No corpus program was added. No file outside these six sites plus the new
corpus file was touched (in particular `corpus/README.md`'s own "Phase 5a
subset" section, which documents `phase-5a.txt`, was left alone: the brief's
"Build" section names exactly six sites and README.md is not one of them).

## Base commit

`4c553f383` (branch `plan/rust-rewrite`), matching the brief.

## Control (both arms), recorded as run

All edits below were made on the working tree, run, and then restored from
scratchpad backups (`/tmp/.../scratchpad/backup/{corpus,coverage,collect_stress,ir_dual,trace_oracle}.rs`,
taken after the six-site wiring was in place). `git diff --stat` for each file
was checked to be a single-line addition again after each restore.

### Arm 1: removing `phase-5b.txt` from a guarded literal reddens that binary

Removed the `"phase-5b.txt",` line from `SUBSET_FILES` in
`rust/crates/rexx-exec/tests/corpus.rs`, then ran:

```
$ cd rust && cargo test -p rexx-exec --test corpus --no-fail-fast
```

Result: **exit 101** (read unpiped, via `echo "EXIT:$?"` appended to the
redirected log). Relevant excerpt:

```
test the_differential_reads_every_phase_subset_file ... FAILED
...
thread 'the_differential_reads_every_phase_subset_file' (28904) panicked at crates/rexx-exec/tests/corpus.rs:690:5:
assertion `left == right` failed: the corpus differential does not read every phase subset file in rust/corpus/. A file missing from SUBSET_FILES is a phase whose programs are never run against the oracle, and the run stays green over whatever is left -- the headline shrinks and nothing asserts on it
  left: ["phase-4a.txt", "phase-4b.txt", "phase-4c.txt", "phase-5a.txt"]
 right: ["phase-4a.txt", "phase-4b.txt", "phase-4c.txt", "phase-5a.txt", "phase-5b.txt"]

failures:
    the_differential_reads_every_phase_subset_file

test result: FAILED. 16 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 30.03s
error: test failed, to rerun pass `-p rexx-exec --test corpus`
EXIT:101
```

Restored `corpus.rs` from the scratchpad backup, `git diff --stat` showed the
file back to its single-line-addition state, and re-ran the same command:
`test result: ok. 17 passed; 0 failed; 1 ignored; ...` / `EXIT:0`.

### Arm 2: removing `phase-5b.txt` from the unguarded `trace_oracle.rs` literal reddens nothing

Removed the `"phase-5b.txt",` line from the literal inside
`every_live_witness_emits_its_prefix_and_is_run_by_the_corpus` in
`rust/crates/rexx-exec/tests/trace_oracle.rs`, then ran:

```
$ cd rust && cargo test -p rexx-exec --test trace_oracle --no-fail-fast
```

Result: **exit 0**. Full tail:

```
test pull_queue_covers_the_line_reading_sources_in_both_modes ... ok
test use_arg_alias_covers_the_alias_prefix_and_its_results_level_gate ... ok
test trace_output_covers_clause_result_assignment_literal_variable_and_operator ... ok
test message_send_covers_the_message_result_line ... ok
test compound_read_write_covers_the_resolved_compound_name ... ok
test trace_labels_covers_the_labels_only_mode ... ok
test dotvariable_beyond_the_list_covers_the_spec_correction ... ok
test call_arguments_covers_the_argument_prefix_at_every_position_shape ... ok
test prefix_operators_covers_plus_and_backslash ... ok
test controlled_loop_covers_the_control_variables_own_value_lines ... ok
test exit_value_covers_the_exit_instructions_own_result_line ... ok
test parse_placeholder_covers_the_dummy_prefix_and_the_two_modes_disagreement ... ok
test every_live_witness_emits_its_prefix_and_is_run_by_the_corpus ... ok

test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
EXIT:0
```

`every_live_witness_emits_its_prefix_and_is_run_by_the_corpus` itself stayed
green, which is the "reddens nothing" property the brief names. **This arm is
honest but currently unfalsifiable**: `phase-5b.txt` is committed empty by
construction, so removing it from the literal's read list drops zero bytes
from the concatenation this test scans, and the test would have stayed green
even with the wiring simply missing. It does not yet demonstrate that a
*future* removal (once a later 5b task has put a live witness in the file)
would go undetected -- it demonstrates that today's empty file changes
nothing when dropped, which is the whole of what can be measured before any
task has appended a program. The property the brief is naming becomes
falsifiable only once `phase-5b.txt` carries a `Coverage::WitnessedLive` row,
which is outside this task's scope.

Restored `trace_oracle.rs` from the scratchpad backup, `git diff --stat`
showed the file back to its single-line-addition state, and re-ran the same
command: `test result: ok. 32 passed; 0 failed; ...` / `EXIT:0`.

`git status --porcelain` after both restores showed exactly the intended
change set: the five modified test files (one line each) plus the new
`rust/corpus/phase-5b.txt`, nothing else.

## Five gate commands, run from `rust/`, each status read unpiped

1. `cargo fmt --all --check`
   Exit 0, no output.

2. `cargo clippy --workspace --all-targets -- -D warnings`
   Exit 0. Tail: `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 4.23s`.

3. `cargo test --release --workspace --no-fail-fast`
   Exit 0. `grep -c "^test result: ok"` on the redirected log: **102**;
   `grep -c "^test result: FAILED"`: **0**.

4. `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast`
   Exit 0. Same **102** `test result: ok` blocks, **0** FAILED. The corpus
   differential's own STRICT-mode line (`tests/corpus.rs`'s `corpus_differential`):
   `mode: STRICT (the gate) -- REXX_CORPUS_GATE is set` / `264 of 264 matching`,
   with `phase-5b.txt` in the union (empty, so it contributes no programs and
   the count is unchanged from before this task). Gate table D's two
   `DELEGATE` rows for phase 5b still show `agree loud=no` (pre-existing,
   unrelated to this task -- the plan itself names these as green over a
   missing mechanism).

5. `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast`
   Exit 0. `grep -c "^test result: ok"` on the redirected log: **102**;
   `grep -c "^test result: FAILED"`: **0**. Same corpus-differential STRICT
   line as gate 4: `mode: STRICT (the gate) -- REXX_CORPUS_GATE is set` /
   `264 of 264 matching`. This is the debug-build run, first attempt was
   killed by the harness (not by memcap's OOM: no `EXIT:` line was ever
   written, so the wrapping shell itself was reaped, not the child process
   at its own exit) while sitting inside `ir_dual.rs`'s
   `both_engines_agree_across_every_population`, which is a long sweep and
   slow in an unoptimized build; retried detached from the tool's own
   background-task supervisor (`nohup` + `disown`), polled at 60s intervals
   through a `Monitor` until-loop, and it completed on that run.

## Self-review

* **Completeness against the brief.** All six named sites carry
  `"phase-5b.txt"`: the four guarded `SUBSET_FILES` literals
  (`corpus.rs`, `coverage.rs`, `collect_stress.rs`, `ir_dual.rs`), the
  unguarded literal in `trace_oracle.rs`, and `EXPECTED_SUBSET_5B` plus its
  pinning test in `coverage.rs`. `rust/corpus/phase-5b.txt` exists, is
  empty apart from its header, and is in `phase-5a.txt`'s original
  (Task-1-era) shape. No corpus program was added. Both control arms were
  run for real (not just described) and restored from scratchpad backups,
  verified green again by rerunning the same command and by `git diff
  --stat`. All five gate commands ran and are green, each with its own
  figure quoted beside it.
* **YAGNI.** Nothing was added beyond the six sites plus the corpus file:
  `corpus/README.md` was deliberately left untouched (not one of the
  brief's six sites; flagged to the coordinator and recorded there as a
  deferred minor for the final review, not something to fix in this task).
  No corpus program, no additional test beyond the one pinning test the
  brief asks for.
* **Comment rule compliance.** Every comment added (the corpus file's
  header, the doc comment on `EXPECTED_SUBSET_5B`) is ASCII, has no
  em-dashes, no historical framing, and names no set's cardinality (the
  corpus file's own header describes format and provenance, not a count;
  `EXPECTED_SUBSET_5B`'s doc comment says why the constant exists, not how
  many entries are in it -- it has none).
* **Test hygiene.** `phase_5b_subset_matches_the_committed_list` mirrors
  the shape of the four existing `phase_*_subset_matches_the_committed_list`
  tests exactly (same `read_subset` call, same `assert_eq!` against a named
  constant, same failure-mode wording adapted to `5b`/`phase-5b.txt`). No
  test was written to describe a property already covered by an existing
  assertion, and no test's own passing depends on an assumption not also
  checked by the control.

