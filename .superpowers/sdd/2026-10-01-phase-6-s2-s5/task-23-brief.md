### Task 23: The instruments criteria 4 to 7 need

**Files:** `phase-6-gate.md`, a feature-gated sharing-fraction counter in `rexx-exec`, a ping-pong
benchmark under `rust/bench-programs/`, a test of context objects across activities (Task 7's, cited).

- [ ] **Step 1:** Criterion 4, the single-owner audit: an inventory, derived by a committed command
      (`/bin/grep -a -rn 'unsafe impl Send\|unsafe impl Sync\|thread::spawn\|Arc<\|Mutex<\|Atomic'`
      over `rust/crates`), of every cross-thread type and `unsafe impl`, each shown by a type-level
      fact (`ObjRef: !Send`, `RegFrame<'a>`'s borrow) or by a test where none exists.
- [ ] **Step 2:** Criterion 5: D3 frame ownership from the type-level facts and Task 7's context test.
- [ ] **Step 3:** Criterion 6: the sharing-fraction instrument (each object tagged with the last
      activity that resolved it; objects touched by more than one counted), run over the corpus and
      ooTest; record.
- [ ] **Step 4:** Criterion 7: a ping-pong benchmark (message round trip, semaphore post/wait, GUARD
      WHEN handoff) against the oracle, recorded, not gated. Gates; commit.

