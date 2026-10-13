### Task 8: Named semaphores

**Files:** `rust/crates/rexx-exec/src/semaphores.rs` and `semaphores/`, `lib.rs` (`named_semaphore`),
`builtin/rexxutil.rs` (`create_event_sem`, `create_mutex_sem`, the open routines), rows, witnesses.

Routines: `SysOpenEventSem`, `SysOpenMutexSem`, and the named form of `SysCreateEventSem` /
`SysCreateMutexSem` refused at `lib.rs:511`.

- [ ] **Step 1:** Read what a name means in the oracle on Unix (process-local table or a system
      object). If it is cross-process (the daemon or POSIX named semaphores), report it: Phase 10 `Rxapi`
      if the daemon, a Moritz ruling otherwise. If process-local, implement it through a named table that
      holds semaphore objects (Specialization is internal: the table maps names to objects, not ids to
      state).
- [ ] **Step 2:** Witness open-by-name, create twice, close and reopen, and Review Focus 4's two
      activities. Per-task check with the semaphore groups Task 2's file names. Commit.

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
