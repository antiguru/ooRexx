### Task 3: Stem routines

**Files:** `rust/crates/rexx-exec/src/builtin/rexxutil.rs` (or a new `builtin/rexxutil/stems.rs` if the
file passes about 1500 lines), `internal_routines.rs` rows, corpus witnesses.

Routines: `SysStemCopy`, `SysStemDelete`, `SysStemInsert`, `SysStemSort`, `SysDumpVariables`.

- [ ] **Step 1:** For each routine, read its `SysRexxUtil.cpp`/`RexxUtilCommon.cpp` body and its ooTest
      group; write a corpus program covering its argument forms, error numbers and Review Focus 1's stem
      shapes; run it on the oracle first and read the stdout.
- [ ] **Step 2:** Implement each as the oracle answers; switch its row from `util(.., Owner::Phase9)` to
      `util_run(..)`. Stems are reached through the existing stem API, not by text munging.
- [ ] **Step 3:** Review Focus 3: the SF #2068 shape is in `oracle-crashes.txt`; the crate's answer for
      it is pinned crate-only.
- [ ] **Step 4:** Per-task check with `base/rexxutil/SysStemCopy`, `SysStemDelete`, `SysStemInsert`,
      `SysStemSort`, `SysDumpVariables` through the harness; re-derive derived rows. Commit.

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
