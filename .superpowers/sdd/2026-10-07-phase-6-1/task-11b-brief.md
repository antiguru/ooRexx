### Task 11b: Deferred items, scheduling and finalization

**Evidence:** `scout-e1-report.md` items 1 and 6. Ruling R11.

**Rule:** a blocking command or native never runs inline on the baton when that can hang the
process; pending UNINITs run at activation return as the oracle runs them (`RexxActivation.cpp:705`,
`NativeActivation.cpp:1361`, `Activity.cpp:249/324/3440`), so a loop creating finalizable objects
runs in bounded memory.

**Files:** `command.rs`, `scheduler.rs` (`exit_for_native`, `exit_for_block`), `scheduler/pool.rs`,
`input.rs`; `dispatch.rs` (activation return, `run_ready_uninits`), `run/call.rs`, `lib.rs`
(`collect_now` and its false comment about when the oracle runs UNINITs); `phase-6-1-gate.md`.

- [ ] **Step 1: Full pool.** With every pool thread busy, a blocking command or native parks its
      activity for a worker (or the pool grows) instead of running inline; the scout's 64-activity
      probe ends (the oracle, given memory for 64 thread stacks, ends 5/5). Sim mode keeps its bound;
      the new park is a recorded scheduling event. Test with explicit switch points.
- [ ] **Step 2: UNINIT drain.** Pending UNINITs run at activation return; the scout's probe prints
      the oracle's count (`done 995` order of magnitude) and peak RSS stays within a small multiple of
      the no-UNINIT control. Corpus programs whose output order changes are named and witnessed
      against the oracle (5 runs each). The check at activation return is one test on the common
      path: measure it.
- [ ] **Step 3:** The per-task check; `whole_groups` and the seeded gate in release (UNINIT timing
      feeds both: update their committed outcomes only where the oracle agrees); perf against the
      task's base and base61, recorded under `## Task 11b`. The queued item
      `2026-10-09-full-pool-inline-command-hang.md` gets its `RESOLVED by` line. Commit.

