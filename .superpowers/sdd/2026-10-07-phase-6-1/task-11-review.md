# Task 11 review: the unjudged mutants

Reviewed `4d72280f4..fe66504f3` (`8910306e5` work, `fe66504f3` report). Paths are under
`rust/crates/rexx-exec/` unless they say otherwise. Mutants M7, M9 and M12 were re-run by the
reviewer in `git archive fe66504f3` copies (`rust/` plus `interpreter/`, which the build scripts read)
at `/tmp/claude-1000/p61/t11r/`, own target dir, `CARGO_INCREMENTAL=0`, `-j 4`, under `memcap`. The
copy was removed afterwards, and each revert was checked byte-equal to the commit's file.

Reviewer runs (each log had exactly one `Compiling rexx-exec` line):

* `REXX_CORPUS_GATE=1 memcap 8G timeout 1500 cargo test -j 4 -p rexx-exec --no-fail-fast --lib
  scheduler::tests::mutants`:
  * unmutated: 4 passed;
  * M9 (`when_parked = false;` deleted in `cancel_wait`): only the M9 test red, at
    `sim:1,uniform:1,fail=wait:1,halt@20`. It answered rc 120, stdout `caught 11.1`, stderr
    `the scheduler found a ready activity holding a park reason`;
  * M12 (`out.extend(failed_sends...)` deleted in `object_roots`): only the M12 test red, at
    `sim:1,gc=1: a write to a dead handle`, left 1, right 0. The write-count assertion (`ended + 1`)
    passed before it, so the reach held.
* `... --features pinning --test concurrency_tests -- measured::a_slice_deferred_before` under M7
  (`slice_deferred = false;` deleted in `switch_to`): red at `concurrency_tests.rs:977`. The stdout
  assertion passed, and deferred `[Native("SORTWITH"), SortComparator]` was 1 where 2 was expected.
* M9 probe with a mutant debug `rexx-run`, running the M9 test's program
  (`memcap 2G timeout 20`, one run per spec, sim is deterministic):

  | spec | with `halt@K` | without the halt |
  |---|---|---|
  | `sim:1,order=fifo,fail=wait:1` | unmutated output, rc 0 | not run |
  | `sim:1,uniform:1,fail=wait:1` | rc 120, park-reason refusal | rc 120, park-reason refusal |
  | `sim:2,uniform:1,fail=wait:1` | rc 120 (halt@60) | rc 120 |
  | `sim:2,pre:2,k=40,fail=wait:1` | `result The NIL object` / `main halted`, rc 0 | `result slow done`, rc 0 |
  | `sim:3,uniform:0.3,fail=wait:1` | `result The NIL object` / `main halted`, rc 0 | `result slow done`, rc 0 |

### Spec Compliance

* ✅ Step 1, M11: `src/scheduler/tests/mutants.rs:84-99` reuses the seeded gate's
  `tests/sim_gate/m11_stale_sleeper.rex` through `include_str!`, which the carry asked for. It asserts
  the slow method's result and `sleeps == 1`, the `cancel_wait` sleeper reach. The M11 site
  (`cancel_wait`'s `retain`, not `:1361`/`:1377`) is chosen and recorded (report, Sites).
* ✅ Step 2, M7: `tests/concurrency_tests.rs:946-979` runs scout C's schedule under
  `EveryOpportunity`. The deferred and pinned-yield counts move under the mutant (reviewer run).
* ✅ Step 3, M9 and M10: M9 at `mutants.rs:101-135` (`halt@K` plus `fail=wait:1`) and M10 at
  `mutants.rs:137-166`. The reviewer confirmed the M9 kill, but see Minor 1 on which spec makes it.
  No scheduler defect was found. The one defect reported, the `pinning` E0507, was reported and fixed.
* ✅ Step 4, M12: dead-handle counter at `src/environment.rs:1245-1250`, judged by
  `mutants.rs:168-196` under `gc=1`/`gc=0.1`. M6 is recorded as equivalent with scout C's argument.
  The UNINIT variant is absent and the report says why (see judgment 3).
* ✅ Step 5: the `## Task 11` table is in `docs/superpowers/plans/phase-6-1-gate.md:1068-1090`. Per-task
  check lines are in the report. ⚠️ The reviewer did not re-run the workspace test or clippy.
* ✅ Carry: no git checkout/restore. Each mutant is an exact-text edit with a revert (report).

### Strengths

* Each test asserts a reach that `note_cancelled_wait` records before any withdrawal
  (`scheduler.rs:1188`). So the reach still holds under the mutant, and a red cannot come from a
  wait that never happened.
* M12 has two assertions. One requires one more write than the succeeding twin (the condition
  reached the message). The other requires no dead write. The kill is the second one, with the first
  one green, which is exactly the brief's counter judgment.
* M11 reuses the gate program instead of copying it.
* The M7 parity explanation is correct and checkable. With B's pinned visits alternating
  deferral and yield, a carried `slice_deferred` shifts B's phase by one. Per-key totals can see that
  shift only when the visit count is odd. Under the mutant, the reviewer saw deferred 1 / yields
  2 against 2 / 1, the shift scout C names (B's first visit takes `pinned_yield`). The tuning
  therefore exposes the mechanism to a totals-only instrument and does not swap in a different
  mechanism. The `[TreeSend, SORTWITH, SortComparator]` deferral of 1 is main's deferral, the reach.

### Issues

**Critical:** none.

**Important:** none.

**Minor:**

1. `src/scheduler/tests/mutants.rs:119-125` and report M9 row. Under the mutant, the first red spec
   is `sim:1,uniform:1`, and there the kill does not need the halt. The same rc 120 refusal appears
   with `halt@20` removed (reviewer table above). The invariant check catches main ready with the
   stale `when_parked` at its ordinary unpark, so this is not the brief's mechanism ("a halt reaches
   `wake_for_halt`"). The halt-dependent kills are `pre:2,k=40` and `uniform:0.3`: with the halt they
   answer `result The NIL object`, and without it `result slow done`. The loop panics at spec 2, so
   under the mutant those specs never run. The verdict "killed" holds either way. Still, the test's
   doc comment and the report's M9 row, which quotes the spec with `halt@20`, attribute the kill to
   the halt. Fix: put the `pre:2` and `uniform:0.3` specs first, or collect every spec's failure
   before asserting. Then state in the report row that `uniform:1` refuses without the halt.
2. `tests/concurrency_tests.rs:942-946`. The doc comment names B's first two visits. It does not say
   that the per-key totals see M7 only because B's pinned-visit count is odd. A later change to
   `sortWith`'s comparison count would leave the test green and blind to M7. The exact-count
   assertions would go red in that case only if the unmutated counts move too. Fix: one sentence
   stating the odd-count requirement.
3. `src/environment.rs:1246-1249`. The counter resolves through `heap.get`, which runs the `sharing`
   instrument's `resolved`. The `get_mut` that follows on the same slot leaves the tag unchanged, so
   the counts do not move. `Heap::peek` exists for walks the sharing instrument must not count, and
   using it would make that independent of `resolved`'s idempotence. Fix: `self.heap.peek(object)`.

Judgments asked for:

* **Killed verdicts (1).** M7, M9 and M12 are red under the mutant and green unmutated (reviewer
  runs). M7 and M12 are red for the brief's reason. M9 is red first for a halt-independent reason
  (Minor 1), and the halt reason is present but unreached under the mutant. M10 and M11 were not
  re-run. Their reach assertions are sound by reading.
* **M7 (2).** It still witnesses M7's mechanism (Strengths). The one-clause comparator changes
  parity, not the path.
* **M12 (3).** Sound. The brief's rule is that above 0 under the mutant kills it. The plain dropped
  send reaches above 0, so the UNINIT variant could only have mattered for an "equivalent" verdict.
  The report records why the variant cannot be built here (Phase 9 refusal, 93.902). The reviewer
  did not check the oracle outputs quoted for it.
* **Counters (4).** `NATIVE_ENTRY_WRITES`, `native_entry_writes`, `CANCELLED_WAITS`,
  `CancelledWaits`, `cancelled_waits`, `note_cancelled_wait` and its call, and `GuardTable::queues`
  are each `#[cfg(test)]`. A release build compiles none of them. `queues` reads `waiting` only, and
  the counter reads before the write, so they do not change behaviour (Minor 3 is a style point).
* **Pinning fix (5).** `concurrency_tests.rs:1588` `mode.clone()` is correct and minimal: the
  `Option<SwitchMode>` is no longer `Copy` since Task 9, and the closure is `Fn` across rayon rows.

### Assessment

**Task quality:** Approved
