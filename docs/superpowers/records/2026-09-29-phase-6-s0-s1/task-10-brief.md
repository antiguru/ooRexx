### Task 10: Plain DO flattened, and the `Op::Exec` outcome channel

**Files:** `rust/crates/rexx-exec/src/run/loops.rs` (`flat_loop_start` `:1356`), `ir/drive.rs`
(`Op::LoopRun` `:1864`, the `Fallback` arm `:1903-1915`, `Op::Exec` `:1651`), `run.rs`
(`exec_instruction` `:541`, `exec_guard` `:1842`, `exec_reply`).

**Interfaces:** Produces `enum ExecOutcome { Done, Park(ParkReason), Split }` (names final in the
report) returned from `Op::Exec` arms to the driver; S1 returns only `Done`, and the driver's handling
of `Park` and `Split` is written and unit-tested with a test-only arm.

- [ ] **Step 1:** Witness first: a corpus program with a function call and a send inside
      `if ... then do ... end`, a SIGNAL out of a plain DO, and a LEAVE-free plain DO in a loop body;
      identical on the oracle. The Task 3 counter shows the calls no longer under a
      non-flattened-loop frame.
- [ ] **Step 2:** `LoopKind::Simple` takes the flat path: its body runs inline, END is a no-op, and it
      pushes no loop pass. `With` keeps falling back.
- [ ] **Step 3:** Introduce the scheduler seam (spec 2.1): an internal trait with `spawn`,
      `run_until_park`, `park(reason)`, `unpark(activity)`, `yield_at_slice`, `exit_for_native`,
      `exit_for_block`, `post_completion`, `request_baton` and `stop_the_world`, and a
      single-activity implementation in which only `run_until_park` and `stop_the_world` do anything
      (the spec lists the seam under S0; it lands here, beside its first user). The `Op::Exec` arms
      return an outcome to the driver instead of `()`, and the driver handles `Park` (park the
      activity at this clause) and `Split` (hand the continuation to a new activity) through the
      seam, exercised in S1 only by a unit test with a test-only instruction
      arm, since no activity can park yet.
- [ ] **Step 4:** Corpus, collect-stress green; callgrind.
- [ ] **Step 5:** Gates; commit.

