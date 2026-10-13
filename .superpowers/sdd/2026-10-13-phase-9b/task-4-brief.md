### Task 4: File routines

**Files:** `builtin/rexxutil.rs` (or `builtin/rexxutil/files.rs`), `internal_routines.rs` rows, corpus
witnesses.

Routines: `SysFileCopy`, `SysFileMove`, `SysIsFileDirectory`, `SysIsFileLink`, `SysGetFileDateTime`,
`SysSetFileDateTime`, `SysTempFileName`, `SysSearchPath`, `SysFileSearch`; and the implemented
`SysFileExists` case `SysFileXXX.TEST_FILEEXISTS_NIL_INVALID`.
`TEST_FILEEXISTS_NIL_INVALID` is 9b's although `suite-9d-set.txt` lists it (controller ruling, 2026-10-13, ledger .superpowers/sdd/2026-10-13-phase-9b/progress.md):
once it is green, this task removes its line from `suite-9d-set.txt`.

- [ ] **Step 1:** As Task 3 Step 1, with Review Focus 2's path shapes and Review Focus 5's `TZ` run.
      Witness programs create their files under a directory they `mkdir` in the current directory and
      remove them at the end.
- [ ] **Step 2:** Implement; switch rows. `SysTempFileName`'s answer is random: the witness prints a
      property (length, the filled template positions, that the file does not exist), and the record says
      what is compared.
- [ ] **Step 3:** Per-task check with `base/rexxutil/SysFileXXX`, `SysFileDateTime`, `SysSearchPath`,
      `SysFileSearch`, `platform/unix/tilde`, `base/class/File`, `samples/samples`,
      `base/security.manager/SecurityManager` (`TEST_CALL_PASS_THREE_ARGS`) through the harness;
      re-derive derived rows. Commit.

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
