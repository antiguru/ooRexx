### Task 1: The 9a carry-in

**Files:** `rust/bench-programs/callgrind.sh`, `rust/crates/rexx-exec/src/dispatch/library.rs`
(`push_native_frame`, `take`, `pop_native_frame`), `rust/crates/rexx-exec/src/environment.rs`
(`class_in_program`, `package_local_symbol`), `rust/crates/rexx-exec/src/dispatch/hash.rs`
(`package_local_get`), records named in the carry-in.

- [ ] **Step 1:** `callgrind.sh` reports the whole-process count (libc and ld added back) beside the
      libc-subtracted one, and its comparison table judges by whole-process. Its own test (if
      `rexx-bench` has one for the table) covers both columns.
- [ ] **Step 2:** Build the pin from `a98238be5` into `/home/moritz/dev/repos/ooRexx-p9-pin/`
      (`git archive`, touch the tree, its own target directory, require a `Compiling rexx-exec` line),
      record its sha256 in `phase-9a-gate.md`'s pin section beside the old one, and delete the build
      directory, keeping only the binary.
- [ ] **Step 3:** N1: the native frame is no longer copied by value on push, take and pop (box it, pass
      it by reference, or shrink it back; the fifth memcpy site in `push_native_frame` goes). Measure
      `extcall` against the pin, whole-process: within +0.5%.
- [ ] **Step 4:** m1: a corpus witness that repeats a `.NAME` miss after a package-local write, so the
      memo's `debug_assert!` runs in the debug gate; restore `directory_get`'s view assertion in
      `package_local_get`. m2: `class_in_program` propagates `Err` and `Owed` as `package_find_class`
      does, witnessed. m3, m4: the records and the metaclass pin the carry-in names.
- [ ] **Step 5:** Per-task check, plus `concurrency_tests` (native calls) and `release_local_reference`.
      Commit.

