### Task 6: Message routines

**Files:** `builtin/rexxutil.rs` (or `builtin/rexxutil/messages.rs`), rows, witnesses.

Routines: `SysGetMessage`, `SysGetMessageX`, `SysFormatMessage`, `SysGetErrorText`.

- [ ] **Step 1:** Read the oracle's catalog handling (`catopen`/`catgets` on Unix). Decide each by the
      oracle's observable: a catalog the system provides, a missing catalog, an invalid set or message
      number, insert substitution in `SysFormatMessage`. `SysGetErrorText` is `strerror` text: the witness
      compares the numbers whose text is the same on both sides (same libc), and the record says so.
- [ ] **Step 2:** Implement; switch rows. `TEST_CATALOG_INVALID` and `TEST_GENCAT` keep their Phase 10
      `Rxapi` rows (they reach `RXFUNCQUERY`).
- [ ] **Step 3:** Per-task check with `base/rexxutil/platform/unix/SysGetMessage`,
      `base/rexxutil/SysFormatMessage`, `base/keyword/ADDRESS` through the harness. Commit.

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
