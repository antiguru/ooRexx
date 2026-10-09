### Task 11: The unjudged mutants

**Files:** `scheduler/tests.rs`, `tests/concurrency_tests.rs`, `scheduler.rs` (`cancel_wait`
`:1034-1049`, `switch_to` `:976-977`), `activity.rs:607` (`object_roots`), `environment.rs:1241-1247`
(a dead-handle write counter under `cfg(test)`), `phase-6-1-gate.md`.

- [ ] **Step 1:** M11: a crate test under `fail=wait:K` in which a started message sleeps while main's
      `SysSleep` wait fails with 11.1 and is trapped, then main waits on `~result`; asserts the result
      is the slow method's and that `cancel_wait`'s sleeper branch was reached. Run it with the
      `sleepers.retain` deleted: red. Restore.
- [ ] **Step 2:** M7 through the pinning report under `every` (scout C's schedule: A defers a slice in
      a pinned frame then parks pinned; B runs pinned); the deferred and pinned-yield counts change
      under the mutant.
- [ ] **Step 3:** M9 under `halt@K` and `fail=wait:K` (a GUARD WHEN wait fails trappably, the program
      parks on `~result`, a halt reaches `wake_for_halt`); M10 under `fail=wait:K` (a guard-lock wait
      fails while another activity holds the guard; a later send). A defect either shows is fixed here, witnessed per the Global Constraints' found-defect rule.
- [ ] **Step 4:** M12 with the dead-handle write counter and a `gc=q` run of a program whose main drops
      the message first (and a Message subclass with UNINIT); judged by the counter: 0 unmutated; above 0 under the mutant kills it; 0 under the mutant
      records it equivalent with the program named. M6 recorded as
      equivalent under any outcome judge, with scout C's argument.
- [ ] **Step 5:** Each mutant run with `REXX_CORPUS_GATE=1 --no-fail-fast` and restored; the table in
      `## Task 11`; the per-task check. Commit.

