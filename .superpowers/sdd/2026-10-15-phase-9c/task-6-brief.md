### Task 6: The `rexx-run` switch

**Files:** every file `git grep -l 'rexx-run' -- rust` lists at the task's start, recorded with the
command in the report. At `39baa699f` that includes `src/bin/rexx-run.rs` (deleted), the test files
using `CARGO_BIN_EXE_rexx-run`, `tests/support/group_runner.rs:582` (`run_crate_process`, sim mode),
`tests/support/arity.rs:399`, `rexx-bench/src/bin/rexx-bench-suite.rs:597-658` and
`rexx-bench-suite/report.rs:175,185`, `rexx-bench-band.rs:143`, `rexx-arms.rs:15`,
`rexx-bench/src/child.rs:50,58`, `bench-programs/wallclock.sh:13,64,65`, `bench-baselines/PINNED.md`,
`rust/CLAUDE.md:49`, `corpus/README.md:295`, `corpus/phase-4c.txt:40`, `rexx-exec/src/lib.rs:2859-2860`,
`rexx-parse/tests/deep.rs:13`, and the samples lines of `rust/corpus/suite-9d-set.txt`.

- [ ] **Step 1:** Delete `rexx-run.rs`. Switch every caller to `rexx` (`CARGO_BIN_EXE_rexx`,
      `target/release/rexx`). A sentence that described `rexx-run` describes `rexx`, or is deleted if
      it is history. `PINNED.md`'s lines that record which binary a historical pin was are kept, and
      listed in the report.
- [ ] **Step 2:** Audit every assertion that relies on `rexx-run`'s own exit status 2 or usage text
      (`rexx` answers usage with rc 255) or on a literal binary path, with the grep command in the
      report. Each is changed to `rexx`'s answer or shown unaffected.
- [ ] **Step 3:** Run `samples/scclient`, `samples/sfclient`, `samples/scserver` and
      `samples/sfserver` through the harness and act on the result as spec section 3 says: a test
      that passes leaves `suite-9d-set.txt`, and a test that still fails stays there with its
      cause named in the report.
- [ ] **Step 4:** `git grep -n 'rexx-run' -- rust` prints only the kept `PINNED.md` lines. Per-task
      check, plus every test binary the switch touched and `cargo build -p rexx-bench`. Commit.

