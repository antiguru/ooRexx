### Task 3: The derived concurrency test list and the early pinned-park counter

**Files:** `rust/crates/rexx-exec/tests/concurrency_tests.rs` or a committed derivation script (the
report says which), a feature-gated counter in `rust/crates/rexx-exec/src/` (feature name in the
report), `docs/superpowers/plans/phase-6-pinning.md` (new).

- [ ] **Step 1:** Derive exit criterion 1's list **per test method** (spec section 9): every ooTest
      test method whose body, or a helper it calls within its group, uses REPLY, `~start`,
      `~startWith`, GUARD, the semaphore classes, `Sys*Sem`, Alarm, Ticker, SysSleep,
      `.context~thread` or TraceObject thread and guard fields. Commit the command and its output,
      with the exclusions and a reason each (rxapi persistent state; RexxQueue, Phase 10; extensions
      and samples, Phase 10). Negative control: add a scratch copy of a group with a REPLY in a new
      method and show the derivation picks it up.
- [ ] **Step 2:** Add a counter, behind a cargo feature that is off by default, recording at each
      **would-be park point** (GUARD ON contended or WHEN, `Message~result`/`~wait`, a semaphore
      wait, SysSleep, REPLY) the kinds of Rust frames between the driver and that point: the pinned
      re-entry kinds of spec 2.1 (sort comparator, conversion, UNKNOWN, FORWARD, operator dispatch,
      trap handler, native-API callback, library entry, stream/PULL/output/trace/redirect wrappers,
      loop header, non-flattened loop, INTERPRET, tree-evaluated expression or send, `Op::Exec`).
      It records; it never changes behaviour. The feature compiles away entirely when off (check
      with a `.text` hash against a build without it).
- [ ] **Step 3:** Run the Step 1 tests under the counter on today's code and record, per test and per
      kind, the would-be pinned parks in `phase-6-pinning.md`. This is the baseline Task 11
      re-measures and the input to S2's plan (spec section 11).
- [ ] **Step 4:** Gates; commit.

