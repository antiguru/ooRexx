## Task 1: the run timeout, and what "did not finish" means

**Goal.** No probe can hang the workspace, and a run that did not finish normally is a structural
failure rather than a verdict.

**Why first.** There is no timeout anywhere in `crates/rexx-exec/tests/support/`: `oracle.rs` builds
every outcome from `command.output()`, which waits forever. The spec's own structural check names a
mechanism that does not exist (ruling R31). And `oracle.rs:189`/`:214` build
`exit_code: output.status.code().unwrap_or(-1)`, so a run killed by a signal already yields three
descriptors and an exit code of `-1`, which any verdict function reads as `diverge-status` -- a
divergence where there should have been a failure.

**Build:**

* A deadline on all three of `Oracle::run`, `run_with` and `run_with_stdin`. Two routes, both
  checked: poll `Child::try_wait` on a short interval and `kill()` at the deadline, with **no new
  dependency**; or `wait-timeout 0.2.0`, which **is** in this machine's cargo cache. Take the first
  unless the second buys something, and record which and why.
* `CppOutcome` gains an explicit termination, not a sentinel exit code. Classification is a **pure
  function of `(Option<i32>, deadline_exceeded: bool)`** -- `None` is a signal death, `true` is the
  harness's own kill, `Some(n)` is a normal exit -- so both failing arms are unit-testable without
  synthesising an `ExitStatus`, which is not portable to construct.
* A helper the tables use in Task 4 and Task 5: `did_not_finish(&CppOutcome) -> bool`, tested on the
  status and never on the exit code.

**Verification, runnable now.** A program whose only content is `do forever; end` is run through the
new path and **reddens at the deadline instead of hanging**; the same program under the old path is
what the task records as the before state, run once by hand under `timeout -s KILL 10` rather than in
the suite. The two classification arms get a unit test each. Then the whole existing suite: all five
gate commands, corpus unchanged at 106 of 106, and the oracle invocation count unchanged.

**What this does not cover, and it is the honest half.** The crate side of a gate-table row runs
**in-process** (`Invocation::with_engine`, as the spec requires and as `ir_dual` does), and an
in-process run cannot be killed. **A probe that hangs the crate still hangs `cargo test`.** No
mechanism here changes that; what the plan asks instead is that every probe program committed by any
later task is first run by hand through `rexx-run` under `timeout -s KILL 10`, and the task says it
did. That is a human check and is written down as one.

**Done when** the hanging program reddens, both classification arms have a test, and the five gate
commands pass. No sitting: `tests/` only.

---

