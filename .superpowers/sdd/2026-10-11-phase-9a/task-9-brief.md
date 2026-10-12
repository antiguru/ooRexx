### Task 9: `ReleaseLocalReference` and the `Package` hang

**Files:** `rexx-api/src/ffi.rs` (`ReleaseLocalReference`), the native-call local-reference list,
`rust/crates/rexx-exec/tests/release_local_reference.rs` and its C++ source (new), the `Package` hang's
site.

- [ ] **Step 1:** The test first: commit the probe's C++ source from
      `.superpowers/sdd/queued/2026-09-29-release-local-reference-memory.md`; the test builds it with
      `c++ -shared -fPIC -O1 -I<repo>/api -I<repo>/api/platform/unix` (fails if no `c++`), runs a
      program calling it at 200,000 and 2,000,000 iterations under `memcap 2G`, reads peak RSS from the
      child's rusage, and asserts the second is at most twice the first. It fails today (430 MB).
- [ ] **Step 2:** Implement: a released local reference drops from the call's rooted set so a
      collection inside a long native call can reclaim it, as `NativeActivation::removeLocalReference`
      does. Update `phase-4-exclusions.txt:6179`'s row to delivered.
- [ ] **Step 3:** `base/class/Package` hangs at program end after its summary (scout A section 4).
      Bisect by renaming tests out (the harness's `REXX_SUITE_GROUPS` plus a rename list) to the test
      that leaves the state behind, then fix it with a crate test that reproduces it in one program.
      Per-task check. Commit.

