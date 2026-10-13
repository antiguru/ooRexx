### Task 7: Process and session routines

**Files:** `builtin/rexxutil.rs` (or `builtin/rexxutil/process.rs`), rows, witnesses, unit tests.

Routines: `SysCls`, `SysGetKey`, `SysQueryProcess`, `SysSetPriority`, `SysUtilVersion`, `SysLoadFuncs`,
`SysDropFuncs`, `SysFork`, `SysWait`, `SysCreatePipe`.

- [ ] **Step 1:** Classify each: observable from a program (corpus witness), terminal-only (`SysCls`,
      `SysGetKey`: unit test against a pseudo-terminal or the escape bytes written, spec section 4), or
      needing `unsafe`/the daemon (report, do not write). `SysFork` in a process with interpreter threads:
      measure what the oracle does and report before implementing.
- [ ] **Step 2:** Implement the rest; switch rows. `SysDropFuncs` unblocks `base/security.manager/SecurityManager`.
- [ ] **Step 3:** Review Focus 4 for `SysWait`/`SysCreatePipe`: witnesses run normally and under the
      simulation mode.
- [ ] **Step 4:** Per-task check with `base/security.manager/SecurityManager` and the groups the
      routines' names reach (Task 2's file) through the harness; re-derive derived rows. Commit.

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
