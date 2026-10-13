# Phase 9b Task 7 review: the RexxUtil process and session routines

Diff `464e5ab03..fe5191171`, reviewed from `review-464e5ab03..fe5191171.diff`. Every run below came
from a scratch copy of `fe5191171`'s tree (`git archive` of `rust/`, `interpreter/`,
`docs/superpowers/plans/`, `extensions/rxregexp/`, plus a symlink to `ootest/`), with its own
`CARGO_TARGET_DIR`. Nothing in the worktree was edited.

### Spec Compliance

- ✅ Spec compliant. Step 1: every routine is classified, and `SysFork` was measured on the oracle
  and reported, not written (`process.rs` `fork`, `oracle-crashes.txt` entry 46). Step 2: the ten
  rows switched (`internal_routines.rs` hunk), and `SysDropFuncs` unblocks SecurityManager. Step 3:
  the `SysCreatePipe` witness runs under the simulation mode, and `SysWait` runs under it in
  `tests/rexxutil_process.rs`. Step 4: SecurityManager went through the harness (`suite-table.tsv`).
  Task 2's baseline names no other group that is blocked on these routines
  (`grep -a -i "SysQueryProcess\|SysCreatePipe\|...\|SysFork" task-2-baseline/table.tsv tests.tsv`
  prints only SecurityManager rows).
- ⚠️ SecurityManager is 23 of 29, not 24: `TEST_REQUIRES_HANDLE` fails on a defect outside RexxUtil.
  It is confirmed and sized under Q1. The controller has to decide whether it is fixed here or
  moved.

### Strengths

- The C++ was ported closely. `PTIME`'s millisecond carry and `%2d` hours both have unit tests
  (`process/tests.rs` `ptime_carries_milliseconds_and_splits_the_clock`). `PGID` is
  `getpgid(getppid())`, as in the C++, and `pgid_is_the_parents_group` pins it in a process group
  of its own, which the corpus witness cannot do.
- `SysWait` is kept out of the corpus harness, whose programs share one process, and is tested in a
  `rexx-run` child instead. That is the right call.
- The witness masks each nondeterministic value by its relation to `/proc`, not by deleting it, so
  `PID`/`PPID`/`PGID`/`PPRIO` and the pipe flags all stay checked against the oracle.
- The `SysFork` measurement (entry 46, the threaded fork never ending) is useful input for the
  grant decision.

### Answers to the controller's questions

**Q1. TEST_REQUIRES_HANDLE.** Confirmed in both code bases.
- Oracle: `PackageClass::loadRequires` (`interpreter/classes/PackageClass.cpp:1319`) and
  `PackageClass::newRexx` (`:184`) resolve `fullName`/`resolvedName` from the **original** target
  *before* calling `InterpreterInstance::loadRequires`. `PackageManager::loadRequires`
  (`package/PackageManager.cpp:716-766`) then lets the manager change `shortName` (`:722`), which
  is used only for the cache and the macrospace, and sends the second event with that
  pre-resolved name (`:756`).
- Crate: `Interp::load_requires` (`rust/crates/rexx-exec/src/install.rs:617`) shadows `name` with
  the manager's replacement and resolves from it (`:625-627`).
- Reproduced with the implementer's `probes/requires-handle.rex`: the oracle's second event names
  `.../R_handle.cls`, and the crate's names `.../R_replaced.cls`.
- **Fix size:** one function, `load_requires`. Three lines: bind `let original = name;` before the
  first check, and pass `original` to `resolve_requires` and `resolve_search` at `:626-627`.
  Applied in the scratch copy, the probe printed the oracle's three lines, and the SecurityManager
  group went to **24 of 29**: `TEST_REQUIRES_HANDLE` passed, and the five still failing are exactly
  the 9d-set five (suite table from the copy). Its three callers are the `::REQUIRES` install
  (`install.rs:566`), `load_package` (`:928`, `.Package~loadPackage`) and `load_package_global`
  (`:934`, `.Package~new`). All three map onto C++ paths that pre-resolve from the original, so
  the fix is right for all three. It changes behaviour only where a security manager replaces a
  `REQUIRES` name.
- A related divergence for the same fix round: the crate's later `requires_file_not_found(name)`
  sites (`install.rs:631`, `:643`, `:650`) report the replaced name, where the C++ reports the
  original `target` (`PackageClass.cpp:1328`).
- **Recommendation: fix it in Task 7's fix round, with a witness.** The change is three lines, the
  witness already exists (the probe; print `endsWith` booleans, not paths), and it closes the
  group this task was dispatched to close. The global constraint "a change of a shared path is
  reported and moved" is the controller's to waive. If it is not waived, add a
  `suite-9d-set.txt` line owned by 9c (`::requires` resolution), and the group stays red until 9c.
  The fix round must run the corpus gate, since the path is shared.

**Q2. unsafe.** None added.
- `git diff 464e5ab03 fe5191171 | grep "^+.*unsafe"` matches only prose (the report,
  `oracle-crashes.txt`, the `Loud::unsafe_grant` name and doc, the refusal-sites row). The new code
  uses only safe `rustix`/`std` calls. `crates/rexx-core/tests/unsafe_sites.rs` scans for the
  `unsafe` token in code, so a new site would have reddened it.
- `rustix-1.1.4/Cargo.toml` defines `termios = []` and `pty = ["fs"]`, so neither pulls in a crate.
- `git diff --stat 464e5ab03 fe5191171 -- rust/Cargo.lock` is empty. Cargo.lock records packages,
  not features, so "unchanged" is accurate and there was nothing to record. The `unsafe` inside
  rustix is the dependency's, as it already was for `fs`/`process`.
- No safe `getrusage` exists: `grep -rn "fn getrusage" rustix-1.1.4/src` prints nothing.

**Q3. SysWait/SysCreatePipe safety.**
- Both are safe Rust APIs: `rustix::process::wait` and `std::io::pipe` + `fcntl_setfd`/`setfl` +
  `into_raw_fd`.
- `SysWait` is `waitpid(-1)`. It reaps **any** child of the process: another activity's
  `ADDRESS` command child, a child of another interpreter in the same process (the corpus harness
  runs programs on threads of one process, and Phase 6 isolated interpreters do the same), or the
  embedding host's `std::process::Child`. The victim's `Child::wait` then fails with `ECHILD`. In
  the crate, a command answers `UNKNOWN_COMMAND` (127) with `FAILURE` (`command.rs:702-705`), and
  `SysCls` answers `-1`.
- It is not a memory-safety hazard: no `kill` follows a reaped pid in the crate
  (`grep -rn "\.kill()" src` prints nothing). It is an isolation hazard across interpreters, and
  inherent to the routine. The oracle is the same process-wide, and one interpreter in one
  process behaves as the oracle does (entry 45).
- `SysWait` also blocks holding the baton, as the oracle's native routine holds the kernel lock.
- `SysCreatePipe` leaks two descriptors without `FD_CLOEXEC` per call, which every later child of
  the process inherits, the harness's oracle children included. This is the oracle's semantics,
  and harmless for the witness, whose `head -n 1` does not wait for EOF.
- Minor, below.

**Q4. PTIME/PMEM.**
- Self vs children matches. `/proc/self/stat` fields 14/15 (`utime`/`stime`) are the thread
  group's own totals, all threads included and children excluded (children are fields 16/17),
  which is what `RUSAGE_SELF` reports. Field offsets 11/12 after `)` are correct
  (`process.rs` `Usage::of_this_process`).
- Units: both are converted to (seconds, microseconds). The crate's value has 1/`CLK_TCK`
  resolution (10 ms), where the oracle's has microseconds of the same cputime.
- `PMEM`: both are in KB, but `ru_maxrss` also carries the pre-`exec` high-water mark, which
  `VmHWM` does not.
- The 10 ms ticks are acceptable. Neither value repeats between runs on either side, and the
  witness masks `PTIME`'s digits while still checking its format byte for byte. The report states
  both differences (concern 4).

**Q5. SysCls/SysGetKey.**
- The pty tests prove the byte read without a line end, the echo bytes (`k` with `ECHO`, nothing
  with `NOECHO`), and that the local modes are restored, **on the success path only**.
- Restoration after a failed read is not witnessed. `read_key` restores unconditionally, because
  nothing returns between the set and the restore, so it holds by construction. But a mutant that
  restores only on `Ok(1)` would survive every test.
- The `'FF'x` end-of-input answer is tested only through `key_text`, not through `read_key` on a
  closed pty.
- No restoration on a terminating signal. The oracle's handler (`SysRexxUtil.cpp:204-208`)
  restores and re-raises into itself, so it is no model. Report concern 6 says the crate installs
  none.
- `SysCls`'s bytes are proven by the corpus witness, not by a pty: `[H[2J[3J` matches the oracle's
  output with `TERM=xterm`.

**Q6. Sim witnesses: run, not just claimed.**
- I re-ran the witness from fresh directories. Oracle rc 216, crate rc 216, stdout identical, and
  the oracle's stdout `cmp`-identical to the committed `witness.oracle.out`. Stderr differs only
  in the run directory.
- Under five fresh seeds (`sim:11`, `sim:12,uniform:1`, `sim:13,pct:3,k=12`, `sim:14,pre:2,k=12`,
  `sim:15,uniform:0.5,gc=0.5`): rc 216, stdout same, with 3 to 16 switches each.
- The `SysWait` sim runs live in `rexxutil_process.rs`, which ran in the mutation check below.

**Q7. The witnesses fail without the change.** I ran two mutants in the scratch copy, each by an
exact edit:
- `SysWait`'s no-child answer `NO_CHILD` → `0`: `sys_wait_with_no_child_answers_minus_one`
  FAILED (`left: "0\n0\n0\n"`, `right: "-1\n-1\n-1\n"`).
- `SysCreatePipe`'s `O_NONBLOCK` put on the write end instead of the read end: the corpus witness
  differs at `n flags 04000 01` → `00 04001` and at `Nonblocking flags`.

### Issues

#### Critical (Must Fix)

None.

#### Important (Should Fix)

None in Task 7's own code. `TEST_REQUIRES_HANDLE` (Q1) is a pre-existing defect in
`install.rs:617-627` that this task exposed. It should be fixed or moved by ruling.

#### Minor (Nice to Have)

1. `process/tests.rs`: no test that the settings are restored after a failed read, and none of
   `read_key`'s `None` path. One fix: close the controller before typing, then assert
   `key == None` and `after == before`.
2. `process.rs` `wait`: the doc does not say that it reaps children of other activities and other
   interpreters in the process, which is the property a reader needs. Entry 45 and the test doc
   say part of it.
3. `process.rs` `cls` doc: "its raw wait status -- measured, `256` where ...". This is a
   ` -- ` (the global prose rule) and a record of a measurement (the comment rule in
   `rust/CLAUDE.md`). Drop the clause after the dash.
4. `rexxutil.rs:1` module doc: the re-wrap leaves the line ending in `--`. Pre-existing text, but
   now a two-line split.
5. `SysCreatePipe` in the corpus witness leaks 14 inheritable descriptors per corpus run into the
   test process. Harmless today. Note it if a later witness waits for EOF on a pipe.

### Assessment

**Task quality:** Approved

**Reasoning:** The routines port the C++ closely and use only safe APIs. Cargo.lock and the
`unsafe` count are genuinely unchanged. The witness reproduces against the oracle, including under
fresh simulation seeds, and both mutants I ran were caught. The one open item,
`TEST_REQUIRES_HANDLE`, is a confirmed three-line fix in `load_requires` that took the group to
24/29 in a scratch copy. I recommend landing it in this task's fix round, with the probe as a
witness.
