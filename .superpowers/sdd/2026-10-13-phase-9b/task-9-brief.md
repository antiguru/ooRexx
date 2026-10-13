### Task 9: Package and library sites

**Files:** `rust/crates/rexx-exec/src/dispatch/native.rs` (`handle_set`), `dispatch/library.rs`
(classic-style rows), the `LIBRARY rexxutil` package-name path, `phase-4-exclusions.txt:6123-6135`.

- [ ] **Step 1:** `LIBRARY rexxutil` as a package name loads the RexxUtil package as the oracle does
      (98.903 today); witness `::requires 'rexxutil' library` and a routine call through it; update the
      exclusions rows.
- [ ] **Step 2:** `handle_set`, Moritz's option 3 (ledger 2026-10-12): `HANDLE:0`, `HANDLE:1`, `HANDLE:2`
      through std's stdin, stdout and stderr, no `unsafe`; any other number stays a loud refusal. Witness
      the read shape only (`oracle-crashes.txt` entry 12a forbids the write).
- [ ] **Step 3:** The classic-style library rows: confirm with `git grep` over `ootest`, `testbinaries`,
      `extensions`, `samples` that nothing on Linux reaches them; if so, they keep their refusal with a
      GUARD disposition naming the search; if something does, implement up to the `.STDQUE` call, which
      stays Phase 10 `Rxapi`.
- [ ] **Step 4:** Per-task check. Commit.

## Review Focus

1. **Stems that are not plain arrays.** A stem with a default value, a non-numeric `stem.0`, missing
   tails below `stem.0`, or tails that are not whole numbers, passed to `SysStemSort`, `SysStemCopy`,
   `SysStemDelete` or `SysStemInsert`. The oracle's answer (error number or rc) is the expected one.
   Task 3 pins each shape in its witness.
2. **Paths that are not simple.** Names with spaces, non-ASCII bytes, a trailing slash, a symlink to a
   directory, a dangling symlink, and a directory without read permission, through the file routines and
   `SysFileTree`. Task 4 and Task 5 each pin the shapes that apply to them.
3. **An oracle crash inside RexxUtil.** `SysStemSort` has a known one-line oracle crash (upstream
   SF #2068, project memory). Task 3 reads `oracle-crashes.txt` and records the shape there if it is not
   listed, and the crate's answer for that shape is pinned by a crate-only test with its reason.
4. **Concurrency.** `SysWait`, `SysFork`, `SysCreatePipe` and the named semaphores used from two
   activities at once. Task 7 and Task 8 run each witness under the scheduler's simulation mode as well
   as normally (`concurrency_tests`' S2 list where it applies).
5. **Time zones and clocks.** `SysGetFileDateTime` and `SysSetFileDateTime` round-trip under a non-UTC
   `TZ`, and across a daylight-saving boundary. Task 4 runs its witness with `TZ=Europe/Zurich` on both
   sides.

---
