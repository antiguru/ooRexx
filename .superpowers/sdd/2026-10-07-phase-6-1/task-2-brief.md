### Task 2: DO and LOOP: COUNTER, WITH ... OVER, OVER a stem

**Files:** `run/loops.rs` (`loop_header_plan` `:371-410`, the refusal `:770`, IR fallback `:1333`),
`ir/` lowering of loops, `lib.rs` (`instruction_owner`'s `Do` comment), `run/tests/loops.rs:196-228`,
`tests/concurrency_tests.rs` whole-group expectations naming `DO is not implemented`, `tests/owners.rs`
(`LoopKind::With` tag `:213`, row `:325`), `run/tests/loops.rs:575`, `:600`, `scheduler/tests.rs:1286-1316`
(a refusal raised inside an UNINIT, built from `do counter c over`: give it a refusal that survives
6.1, one of the LIMIT constructors), `tests/ir_recorded_cases/loop-refusals` (its cases become ordinary
cases with the oracle's output), `corpus/refusal-sites.tsv`, corpus programs.

**Interfaces:** None new outside `run/loops.rs` and the IR loop lowering.

- [ ] **Step 1:** Corpus programs from scout A's c1 probes (`c_do_counter_{ctrl,rep,forever,while,until,over}`,
      `c_loop_counter`, `c_loop_label_counter`, `c_do_with_{over_array,index_only,item_only,over_dir,counter}`,
      `c_do_over_stem_{one,three}`), each with the oracle's output. Stem OVER prints in a way deviation
      1 (`docs/superpowers/plans/phase-4-exclusions.txt:844`) licenses: sorted, or a single tail. They fail with
      `DO is not implemented`.
- [ ] **Step 2:** Implement: COUNTER is a variable assigned the iteration count at each iteration's
      start, on every loop kind; `DO WITH INDEX i ITEM v OVER coll` iterates the collection's supplier;
      `DO x OVER stem.` iterates the stem's tails. COUNTER code is emitted only for a loop that names
      one (Review Focus 3). Delete the refusal and update its pins (`run/tests/loops.rs:196-228`, the
      whole-group expectations, `owners.rs`).
- [ ] **Step 3:** Witnesses agree; the per-task check; perf on `emptyloop`, `decloop`, `rexxcps`
      recorded under `## Task 2`. Commit.

