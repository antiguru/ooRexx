# Phase 9b Task 4 review: the RexxUtil file routines

Range `72550bcda..1cbce6cde`. Reviewer runs were all in the foreground. Every interpreter run used `memcap 4G` from a fresh `mktemp -d` directory, and every oracle run also used the `ulimit -v 1048576` / `timeout -k 5 20` wrapper. The scripts and outputs are in the session scratchpad `t4rev/`. The tree was not edited. Mutants were built in a `git archive` copy with their own target dir, which was deleted afterwards.

### Spec Compliance

- ❌ Issues found (one):
  - **The shared path was widened inside the batch.** Global constraints, "Mechanical batches", say that an item needing "a change of a shared path" "is reported and moved by the controller, not widened inside the batch". Instead, `qualify_name` (`builtin/platform.rs:44`) was routed into `QUALIFY`, `DIRECTORY`, `.File` `qualify`, `.Stream` `init`, `resolve_stream`/`forget_stream` (`run.rs`) and the redirect name (`redirect.rs`). Report Concern 2 reports this after the fact. The work is correct (see Q2), it was needed for the brief's own `platform/unix/tilde` group, and it makes stream-heavy code faster. My recommendation is that the controller ratify it, not revert it.
- All nine routines are implemented and switched to `util_run`. `TEST_FILEEXISTS_NIL_INVALID` is green and its `suite-9d-set.txt` line was removed, as the brief requires. `samples/samples TEST_MAKESTRING` was also removed because it passes. Review Focus 2 shapes are pinned in the six witnesses. Review Focus 5 runs under Zurich, plus New_York, on both sides (`tests/datetime_zone.rs`).
- ⚠️ Not verified from the diff: the suite record (`run-output.txt`) is at `92d2868fc`, not HEAD. The later commits change `.rex` files, records and Rust comments only, per the report. I did not re-run the suite.

### Strengths

- Each routine follows the C++ order of checks. I compared `move_file` (`files.rs:196`) line by line with `SysFileSystem::moveFile` (`SysFileSystem.cpp:1680`), and `SysSetFileDateTime`'s structure with `SysRexxUtil.cpp:1400-1452`.
- The paths-as-bytes work is thorough. The nine routines contain no lossy conversion, and every witness carries an `'ff'x` name next to its `'efbfbd'x` lossy decoy.
- The `make_time` read-back filter for chrono's ambiguous answer at a transition instant is subtle and correct, and it is witnessed.

### Answers to the dispatch's questions

**Q1. Europe/Dublin spring-forward gap.** Confirmed on the source and by running both sides.
- What the oracle does: `SysRexxUtil.cpp:1446` sets `newtime->tm_isdst = -1`, then calls glibc 2.43 `mktime`.
- Measured, oracle against crate, both under `TZ=Europe/Dublin`:
  - `2020-03-29 01:30:00` sets `1585441800` on the oracle and `1585445400` here.
  - `2021-03-28 01:30:00` sets `1616891400` and `1616895000`.
  - `2045-03-26 01:30:00` sets `2374101000` and `2374104600`. This date is past the TZif table, in the region governed by the `IST-1GMT0,M10.5.0,M3.5.0/1` footer.
  - The edges `00:59:59` and `02:00:00`, and the repeated `2020-10-25 01:30:00`, are identical on both sides.
- Why they differ: glibc keeps the gap candidate whose own `tm_isdst` is nonzero. Dublin's tzdata flags winter GMT as DST, so glibc lands before the transition. The crate always uses the offset before the skip.
- The fix needs each offset's `isdst`. chrono 0.4.45 has it (`LocalTimeType::is_dst`), but the module is private (`offset/local/tz_info`, 2460 lines). The options:
  - **(a) License it. Cost: none.** POSIX leaves `mktime` in a gap unspecified, and glibc's own comment calls its choice "common practice". It is a narrow, deterministic divergence: only a reading inside a spring-forward gap, only in zones whose tzdata flags the earlier offset as DST.
  - **(b) Write a safe-Rust zone reader.** It would cover the TZif v2 transitions and ttinfo `isdst`, the POSIX footer rule, and glibc's resolution of `TZ` (unset → `/etc/localtime`, `:path`, a zoneinfo name, a bare POSIX string). Estimated at roughly 300-400 lines plus tests. It needs no new dependency and no `unsafe`. It runs only on the `LocalResult::None` path, so it adds no cost to ordinary calls.
  - **(c) A new dependency** (`jiff` exposes `dst()`). This needs Moritz.
- **Recommendation: (a).** Choose (b) only if Moritz also wants the in-program `TZ` divergence below closed, because a zone reader is the one mechanism that fixes both.

**Q1, a related divergence the report does not mention.** Here `SysGetFileDateTime` and `SysSetFileDateTime` read the process's `TZ`, but the oracle reads the program's environment `TZ`.
- Measured with process `TZ=America/New_York`. The program set `TZ` to UTC, then to `Asia/Kolkata`, through `VALUE(..., 'ENVIRONMENT')`.
- Oracle: `2020-06-01 12:00:00`, `17:30:00`, `12:00:00`, `06:30:00`. Crate: `12:00:00` every time.
- This follows the crate-wide stance that TZ is per process (`specs/2026-09-03-phase-5d-foundation.md:289`), so it is not a Task 4 defect. It should be named wherever the Dublin decision is recorded.

**Q2. Scope.** Each shared caller, and how its change is witnessed:
- `QUALIFY` (tilde, `~user`, unknown user, NUL, `""`) and `.File` (tilde, `~user`, unknown user, NUL, `""`): `rexxutil_file_tests.rex`.
- `DIRECTORY` (tilde, unknown user, NUL): the same witness.
- `.Stream~qualify` and `LINEOUT`/`LINEIN`/`STREAM` with `~/`: the same witness.
- **Not in the corpus**, so I probed them on both sides. All were identical on stdout, stderr and rc:
  - stream builtins with a NUL in the name;
  - stream builtins and `.Stream` with an unknown `~user`. The oracle also keys the stream table on `""` (`RexxActivation.cpp:2002` → `QualifiedName` → `qualifyStreamName`), as the crate does;
  - `.Stream~new('')`;
  - redirects to a NUL name, to `~/t.txt`, to `''` and to `~nosuchuser/x`. The last two give `98.920 Unable to open file ""`.
- The `''` and unknown-user redirects are behaviour changes. The old code normalised `''` to the cwd. The new answer matches the oracle but is not pinned by any witness.
- I did not check the `SysDumpVariables` path, which now also gets tilde and the NUL cut, against the oracle.
- `~` with `HOME` unset now answers `""`. I did not run it on the oracle, because the C++ assigns `getenv("HOME")` unchecked and may crash.
- **Q2 verdict:** the changes are correct as far as I probed, but some of them are witnessed only by my ad hoc probes.

**Q3. Entry 39's test with target/ and TMPDIR on one file system.** It fails loudly and does not pass silently. I ran it with `TMPDIR` on the target's ext4 file system: `FAILED`, and the panic at `files/tests.rs:45` says `this test needs ... on different file systems`, with `left: Ok(())`. The cost is that `cargo test --lib` is red on any machine without a tmpfs `/tmp`. That is a Minor item below.

**Q4. Performance.** `callgrind.sh -r 3 -p "dirread sayloop" -B 0.5 pin=<pin> head=<HEAD binary>` exited 0. Whole-process deltas: `dirread` +0.3863%, `sayloop` +0.1860%. Spreads were at most 0.0003%.
- These deltas include Tasks 1 to 3. Neither program resolves a stream.
- No bench program calls a stream builtin (I grepped `bench-programs/*.rex`). So I ran an ad hoc loop by hand under `valgrind --tool=callgrind`: 20000 `LINEOUT`, 20000 `LINEIN`, then `LINES` and a close. Pin: `606478701` and `606640069` Ir. HEAD: `470989043` and `470985197` Ir. **HEAD is −22%.**
- Inclusive `resolve_stream` fell from 246.0M Ir to about 115M Ir. The old `paths::normalize` paid for `cwd_text()` (a lossy `String`), `format!` and `join`. `qualify_name` is still about 22% of that loop, because the oracle qualifies on every call too.
- Within budget.

**Q5. Paths as bytes.** Each of the nine routines is witnessed with an `'ff'x` name next to its `efbfbd` lossy decoy:
- `SysFileCopy` and `SysFileMove`: `rexxutil_file_copy_move.rex:22-25,65`.
- `SysIsFileDirectory` and `SysIsFileLink`: `rexxutil_file_tests.rex`.
- `SysGetFileDateTime` and `SysSetFileDateTime`: `rexxutil_file_date_time.rex:68`.
- `SysTempFileName`: `rexxutil_temp_file_name.rex:21,24`.
- `SysSearchPath`: `rexxutil_search_path.rex:32-33`.
- `SysFileSearch`: `rexxutil_file_search.rex:53-55`.

`files.rs` contains no `from_utf8_lossy`. These path lossy sites remain, found with `grep -rn "from_utf8_lossy\|to_string_lossy\|cwd_text()"` over `rexx-exec/src` and then triaged by hand:
- `DIRECTORY`: `builtin/platform.rs:181` (`cwd_text`), `:189` and `:196`.
- `SysFileTree`, which is Task 5's: `builtin/rexxutil.rs:797`, `:798` and `:841`.
- `.File` methods: `dispatch/files.rs:36` (`path_argument`), `:411` (rename target) and `:556` (search path).
- `.Stream` opens: `dispatch/stream.rs:61` (`qualified_path`), `:555` and `:776`. This is Concern 2's: a stream on a non-UTF-8 name opens the wrong file.
- `CD` command: `command.rs:309`, plus `:567` (`current_dir(cwd_text())`).
- External file read: `install.rs:1859` and `:1866`.

**Q6. GC and `Found::add`'s `push_temp` (`files.rs:663`).** It is not needed.
- `Interp` collects only inside `alloc_with` → `collect_if_due` (`lib.rs:2818-2838`). Between `text_built` and the slot write, `append_slot` → `array_grow` → `charge_growth` only sets `collect_due`. `store_of` returns early for a real Array. `stem_object_write` does not allocate either.
- So no collection can occur in that window. The implementer's `collect_stress` removal run could not have exercised one, by construction. Under stress mode, the next line's `text_built` collects with the previous value already in the array, which tests that the array roots its items, not the hold.
- The hold is dead weight: one temp per matched line, retained until the routine returns.

**Q7. Mutation.** I built two mutants in a `git archive` copy of HEAD. The control, HEAD's binary against the oracle, was identical on all three descriptors for each witness below.
- M1, `qualify_name`'s NUL cut removed (`Some(end) => &name[..end]` → `Some(_) => name`): `rexxutil_file_tests` goes red. `nul 1 [] 1` becomes `nul 0 [<W>/^@x] 0`, `chdir 1 [] 1` becomes `chdir 1 [] 0`, and stderr differs.
- M2, `make_time`'s `.min()` → `.max()`: `rexxutil_file_date_time` goes red (`1603585800` becomes `1603589400`), and so does `rexxutil_temp_file_name` (`0 2 0` becomes `-3 2 0`).
- I did not run the "adds coverage" check, i.e. the same mutants against the suite without the new witnesses.

### Issues

#### Critical (Must Fix)

None.

#### Important (Should Fix)

1. **The shared path was widened inside a mechanical batch, against a global constraint.** See Spec Compliance. The controller should ratify it. The redirect `''` and unknown-user change, and the stream-builtin NUL and unknown-user change, should get witness lines: the four probe programs in `t4rev/prog/shared.rex`, `emptyredir.rex` and `nouserredir.rex` already match the oracle.
2. **The Dublin gap needs a ruling (Q1).** Today it is an unlicensed, unwitnessed divergence, recorded only in a code comment (`files.rs:289` doc) and the report. Licensing is recommended. The in-program `TZ` divergence should be named next to it.

#### Minor (Nice to Have)

1. `files.rs:663`: `push_temp(value)` is shown unnecessary by reading the collector (Q6). Drop it, or keep it consistent with Task 3's ruling, but the record should say "cannot be load-bearing", not "not shown load-bearing".
2. `files/tests.rs:40-50`: the test is loud on a shared file system, which is right, but it makes `--lib` red on non-tmpfs machines. It also uses a fixed `target/tmp/rexxutil-move`, so two concurrent runs collide, and a failed assert leaves both directories behind because cleanup is not in a guard.
3. `tests/datetime_zone.rs`: stdout and stderr are compared through `from_utf8_lossy`. This is harmless for this witness, which prints no raw name, but a byte comparison is the stronger form.
4. `.File~new('n'||'ff'x)~absolutePath` now answers raw bytes where it used to answer U+FFFD bytes. This is correct, but no witness covers it.

### Assessment

**Task quality:** Needs fixes. The code is approved. The fixes are controller rulings (shared-path ratification, the Dublin licence) and witness lines, not code changes.

**Reasoning:** The routines are faithful to the C++, byte-clean, mutation-sensitive, and faster on the stream path than the pin. What remains is the out-of-batch widening, now probe-verified but partly unwitnessed, and an unlicensed `mktime` gap divergence that safe Rust cannot close without a zone reader.
