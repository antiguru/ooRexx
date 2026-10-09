### Task 9: Policies, invariants, decision trace and replay

**Files:** `sim.rs`, `scheduler.rs` (switch-mode arm `:1990-1999`, `next_runnable` `:1591-1630`,
`spawn` `:497-511`, the readying sites `:361-368`, `:587-595`), `tests/` (sim tests).

**Interfaces:** `Policy::{Fifo, Pre { d, k }, Uniform { p }, Pct { d, k }}` with `order: Order::{Fifo,
OneEvent}` (default `OneEvent`) and a fairness floor `F`; a contended step is a clause boundary where
`ready` is non-empty; `fn check_invariants(&self) -> Result<(), Loud>` on the scheduler; the trace as a
`Vec<Decision>` (`Preempt(step)`, `Pick(index)`, `Collect(alloc)`) with a stable hash;
`sim:...,trace=FILE` writes it, `sim:replay=FILE` replays it.

- [ ] **Step 1:** Tests first: `pre:1,k=N` on a two-activity program reaches both interleavings over a
      seed range; a polling activity (no park point) under every policy ends because of the floor;
      `uniform:1,order=fifo` matches `EveryOpportunity`'s outcome on the switch-mode tests whose output reads no clock; a recorded trace
      replays to the same output and trace hash; two separate processes of one seed give the same
      hash (Review Focus 5, the program of Task 8 Step 1 plus a native routine call).
- [ ] **Step 2:** Policies: `pre:d,k=N` takes d preemptions at contended steps drawn uniformly among the
      first k; `uniform:p` preempts each contended step with probability p; `pct:d,k=N` (R5) assigns
      seeded priorities at spawn, lowers the running activity's priority at d-1 change points over the
      first k contended steps, and picks the highest-priority ready activity; every policy forces a
      preemption after F contended clauses, F sized to the oracle's 24 ms slice at its measured clause
      rate (Task 8 Step 2's figure). `order=fifo` turns off the one-event pick.
- [ ] **Step 3:** Invariants checked at every switch in sim: each activity in exactly one of running,
      ready, parked or finished; a ready activity holds no park reason; one baton holder; guard holders
      and waiters consistent with the guard queues; every parked activity has a wake source (a post, a
      deadline, a guard, a halt). A violation is a `scheduler_inconsistency` refusal naming the
      invariant.
- [ ] **Step 4:** The trace, its hash in `SimReport`, write and replay. The per-task check; perf on
      `pingpong/pingmsg`, `pingguard`, `pingsem` (extend `callgrind.sh`'s `PROGRAMS` to them; measure
      their run-to-run spread first and gate on them only if it sits inside the budget, else record),
      `alloc`, `alloc4c`, `heapshape`, `rexxcps`, `emptyloop` under `## Task 9`. Trace `trace=H` joins the `rexx-sim:` line. Commit.

