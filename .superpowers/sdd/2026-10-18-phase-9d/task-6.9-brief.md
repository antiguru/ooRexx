# Task 6.9

* **Task 6.9: conditions.** Clusters: `raise-propagate-from-method`, `raise-result-entry`,
  `call-on-trap-argument`, `signal-trap-missing-label`, `reply-thread-error-report-line-ends`.
  Review Focus 4 applies: one witness each through INTERPRET, REPLY and a propagated condition.
  It also owns two tests Task 6.3 moved past their first failure: `base/bif/CONDITION
  TEST_CONDITION_SIGNAL`, where the NOMETHOD (and NOSTRING) condition object has no `ADDITIONAL`
  entry, and `base/keyword/SIGNAL TEST_SIGNAL_ANY_CONDITION_LOSTDIGITS`, where `RAISE PROPAGATE`
  reaches the caller as SYNTAX (Task 6.3 report).
  Task 6.4's propagate fix (`run/condition.rs`) closed `raise-propagate-from-method` and
  `TEST_SIGNAL_ANY_CONDITION_LOSTDIGITS`. This instance still witnesses that fix through INTERPRET
  and REPLY (Review Focus 4). It also takes the condition-object fields Task 6.4 measured as
  differing: PACKAGE/PROGRAM and POSITION on a NOTREADY object, `CALL ON` STACKFRAMES, a propagated
  object's POSITION, and MESSAGE on a bare 93 object (`docs/superpowers/records/2026-10-18-phase-9d/t6.4/probes/d`).
  It also takes the traceback line of a propagated NOMETHOD, which names the original send where
  the oracle names the `raise propagate` line (Task 6.4 review M7).
  And `raise user x return 5` inside a replied method, which raises 98.936 at the RAISE on the
  oracle and nothing here (Task 6.4 re-review).
  `raise-result-entry` covers `RAISE PROPAGATE RETURN expr`, whose value `run/condition.rs` ignores.
  Task 6.5 measured one consequence: on a write-only stream, `query position write line` answers
  the null string where the oracle answers `ERROR:9 Bad file descriptor`
  (`docs/superpowers/records/2026-10-18-phase-9d/t6.5/r1/probes/write_only_query_result/`).

## The Task 6 steps
### Task 6 (template): one 9d set cluster

The controller writes each instance as `Task 6.<n>: <cluster>`, carrying the cluster's tests, its
groups, its likely files and any Review Focus line `clusters.md` marks for it. Each instance has
these steps:

- [ ] **Step 1:** Write one corpus witness per distinct cause, and show each red against the oracle.
      For the streams cluster, one witness uses a name holding a `'80'x` byte (Review Focus 3).
      For the traceback cluster, one witness covers each of INTERPRET, REPLY and a propagated
      condition (Review Focus 4).
- [ ] **Step 2:** Fix the crate. Where the oracle is wrong, stop and report. The controller takes the
      Deviation to Moritz before any licence row is written.
- [ ] **Step 3:** The witnesses match the oracle and the cluster's tests pass. The whole suite run
      shows no other changed row. Remove the cluster's lines from `suite-9d-set.txt`. Record the
      instance's disposition of any queued item or exclusion row it closes. Run the perf check
      where it applies, then commit.

