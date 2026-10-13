# Phase 9b Task 4 report: the RexxUtil file routines

Status: DONE_WITH_CONCERNS (concerns at the end, none blocking).

Commits:

- `92d2868fc` Phase 9b Task 4: the RexxUtil file and search-path routines
- `0dccdb3b7` Phase 9b Task 4: an array that grows while holding heap strings
- `bf532a205` Phase 9b Task 4: tilde names through streams and DIRECTORY
- `309d16dce` Phase 9b Task 4: comments without structuring semicolons or dashes
- the records commit that carries this report (sha in the reply)

## What was implemented

`SysFileCopy`, `SysFileMove`, `SysIsFileDirectory`, `SysIsFileLink`, `SysGetFileDateTime`,
`SysSetFileDateTime`, `SysTempFileName`, `SysSearchPath` and `SysFileSearch`, in a new
`rust/crates/rexx-exec/src/builtin/rexxutil/files.rs`, each row switched from
`util(.., Owner::Phase9)` to `util_run(..)`. Each body follows its `RexxUtilCommon.cpp` /
`platform/unix/SysRexxUtil.cpp` function and the `SysFileSystem` calls under it, in the oracle's
order of checks.

- **Paths are bytes.** `path_argument` now reads the argument as a `CSTRING` does: 88.909 where it
  has no string value (`requiredString`, no `STRING` fallback; this is what turns
  `TEST_FILEEXISTS_NIL_INVALID` green), cut at its first NUL. It qualifies through one new
  `builtin::platform::qualify_name` (`SysFileSystem::qualifyStreamName` / `canonicalizeName`),
  which works on bytes, resolves `~` from the interpreter's `HOME` and `~user` from `/etc/passwd`,
  answers the null string where the C++ fails (empty name, unknown user, home not absolute), and
  then calls a new byte-level `paths::normalize_bytes` (`paths::normalize` is now a wrapper over
  it). File system calls get the bytes as an `OsStr` (`rexxutil::os_path`) or straight to rustix.
  The existing `SysFileExists`, `SysIsFile`, `SysFileDelete`, `SysRmDir`, `SysMkDir` and
  `SysDumpVariables` go through it too.
- **The same qualifier everywhere the oracle uses `qualifyStreamName`**: `QUALIFY`, `DIRECTORY`,
  `.File`'s `file_qualify`, `.Stream`'s `init`, the stream builtins' table (`resolve_stream`,
  `forget_stream`) and the redirect stream name. Measured before the change: `QUALIFY('~root/a')`
  and `.File~new('~')~absolutePath` answered `<cwd>/~root/a` and `<cwd>/~` here, `/root/a` and
  the home directory on the oracle; `.File~new('')~absolutePath` answered the cwd here and `""` on
  the oracle; a `.Stream` named `~/x` opened `<cwd>/~/x`. The tilde test needed all of these.
  SysFileTree still qualifies on its own (`rexxutil.rs`, Task 5's).
- Argument helpers (`marshalled`, `check_arity`, `required`, `optional_text`) moved from `stems.rs`
  into `rexxutil.rs` unchanged, plus `c_string`, `required_c_string` and `os_path`.
- `SysFileCopy` / `SysFileMove`: `samePaths` by `std::fs::canonicalize`, `EEXIST` for one file named
  twice and for a move onto a regular file, the copy opening, reading 4096 at a time, then setting
  both times to the second and giving a target it created the source's mode. Across file systems
  the move checks it can rename the source in place under a spare name, copies keeping links
  (`copyFileDontDereferenceSymbolicLinks`), and answers `unlink`'s `-1` where the final unlink fails.
- `SysGetFileDateTime` / `SysSetFileDateTime`: `localtime` and `mktime` through chrono's `Local`.
  `sscanf("%4d-%2d-%2d")` and `"%2d:%2d:%2d"` are ported as `scan_triple` (white space before each
  number, sign counted in the width, separators matched exactly, trailing text ignored). `mktime`
  carries out-of-range fields, takes a repeated reading's earlier instant, and a skipped reading at
  the offset before the skip. chrono answers both readings for the instant of a transition itself
  (Zurich `2020-10-25 03:00:00`), so each candidate is kept only where the zone reads it back.
  Both-omitted reads the clock through `Interp::wall_now` (clippy's `disallowed_methods`).
- `SysTempFileName`: fillers counted (none or more than nine answer `""`), a random start from
  `std::hash::RandomState`, the first filler the lowest digit, each candidate qualified, the first
  that `stat` cannot find answered, `""` after a full wrap. A filler argument that is not one byte
  is 40.23 with the oracle's named substitutions (new `Raised::routine_argument_not_a_character`).
- `SysSearchPath`: the variable from the interpreter's environment, the current directory first
  unless the option starts `N`, names starting `/`, `./`, `../` or `~` only qualified, and the
  first entry naming anything ending the search (a directory there answers `""`).
- `SysFileSearch`: `StemHandler` as `Found` (an instance of `Array` itself is emptied by an `EMPTY`
  send and appended to primitively through a new `Interp::array_append`; a stem or stem name is
  written from tail 1 with the count in tail 0 on every exit, tails past the count left), 40.925
  for anything else (new `Raised::not_an_array_or_stem`, its value always `""` as the oracle's
  null substitution renders), `LineReader`'s lines (LF ends a line, one CR before it is dropped, a
  lone CR stays, a last line without LF counts), `mystrstr` (empty needle matches nothing, ASCII
  case fold), and the bad-option 40.900 (new `Raised::routine_user_defined`) cut to the oracle's
  255-byte `snprintf` buffer.

No `unsafe`, no new dependency, no side map.

## Oracle measurements behind it

Every oracle run: `ulimit -v 1048576`, `memcap 4G`, `LD_LIBRARY_PATH` set, `timeout -k 5 20`,
from a fresh `mktemp -d` directory, stdout, stderr and status read separately; the crate the same
under `memcap 4G`. Scripts in the session scratchpad (`o.sh`, `c.sh`, `d.sh`, the last masking the
run directory's name before comparing).

- `rename` of a file onto a non-empty directory answers 39 (`ENOTEMPTY`) on tmpfs, onto an empty one
  21: the kernel's order, which this crate gets by calling `rename` too.
- A copy whose source is a directory answers 21 and leaves an empty target behind.
- `QUALIFY('~nosuchuser/a')` is `""`, as is `.File~new('')~absolutePath`.
- `SysSetFileDateTime` under `TZ=Europe/Zurich`: `2020-03-29 02:30:00` (skipped) sets
  `1585445400` (shows `03:30:00`), `2020-10-25 02:30:00` (repeated) sets `1603585800`, the earlier.
  `'1 :2:3'` does not scan, `' 3: 4: 5'` does, `+2020` does not (width 4 counts the sign),
  `'+202'` sets year 202 and shows `" 202"`, `-202` shows `-202`.
- **A skipped reading in a zone whose daylight saving is the earlier offset** (`Europe/Dublin`,
  `2020-03-29 01:30:00`): the oracle sets `1585441800`, this crate `1585445400`. glibc picks the
  candidate whose `tm_isdst` is set, and chrono does not expose which offset is daylight saving.
  Not in any witness; stated at `make_time`. Concern 1.
- **`SysFileMove` across file systems is an oracle defect**, entry 39 of `oracle-crashes.txt` (a new
  fourth mode there, "undefined answer"): 3 runs answered `32617 1 0`, `32685 1 0`, `32756 1 0`
  and moved nothing. The oracle binary's `temporaryFilename` (`/home/moritz/dev/repos/ooRexx` at
  `cb9563364`) never sets `errInfo` on success and bounds its `snprintf` by 7; this checkout's
  `interpreter/` fixes both (`diff` of the two `SysFileSystem.cpp`), and this crate follows the
  fixed source.
- `samples/scclient` and `samples/sfclient` (carry-in item 6): neither reaches a file routine.
  `scclient.rex` stops at `::requires 'socket.cls'` (43.901, could not find the file) and
  `sfclient.rex` at `::requires 'rxsock' LIBRARY` (98.903, unable to load the library), measured on
  this crate's binary from a fresh directory. The first needs `::REQUIRES` to find `socket.cls`
  (the `.rex`/`.cls` file resolution the dispatch names as 9c's), the second the rxsock library.
  Their lines stay in `suite-9d-set.txt`.

## Tests and results

Commands from `rust/`, every one in the foreground, exit statuses read unpiped. Raw outputs in
`docs/superpowers/records/2026-10-13-phase-9b/task-4/`, each with its commit on the first line.

| check | command | result |
|---|---|---|
| fmt | `cargo fmt --all --check` | exit 0 at `bf532a205` (the last commit changes no Rust) |
| clippy | `CARGO_INCREMENTAL=0 memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` | exit 0 on `92d2868fc`'s code (later commits change only `.rex` and `sourceline_oracle` files) |
| lib + tables | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-exec --lib --test closed_phases --test internal_routines --test owners --test phase_ten_reasons --test refusal_dispositions --test refusal_sites --test no_process_state` | exit 0: lib 1093, then 8, 2, 1, 6, 5, 3, 5 passed |
| refusal sites | `REXX_REFUSAL_SITES_REFRESH=1 CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-exec --test refusal_sites` | re-derived; ignoring the location column the only change is the three new constructors, each `body`, `off-send-surface` |
| sourceline | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-parse --test sourceline_oracle` | exit 0 at `309d16dce`'s tree |
| suite groups | `REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=base/rexxutil/SysFileXXX,base/rexxutil/SysFileDateTime,base/rexxutil/SysSearchPath,base/rexxutil/SysFileSearch,base/rexxutil/platform/unix/tilde,base/class/File,samples/samples,base/security.manager/SecurityManager REXX_SUITE_TABLE=<records>/table.tsv CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` | exit 101 at `92d2868fc` (`run-output.txt`): 7 of 8 groups green; SecurityManager red on 11 tests, see below |
| corpus | `REXX_CORPUS_GATE=1 CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test corpus` | exit 0 at `309d16dce`, `1035 of 1035 matching` (`corpus-output.txt`) |
| collect_stress | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test collect_stress` | exit 0 at `bf532a205`, 37 passed, the L0 subset with the six witnesses included (`309d16dce` changes only comments) |
| others | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test ir_recorded --test trace_oracle --test coverage --test datetime_zone` | exit 0 at `0dccdb3b7`: 28, 41, 21, 22 passed (`other-tests-output.txt`) |
| crate-only | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-exec --lib rexxutil::files` | 1 passed |

Per group, crate passed of tests (`tests.tsv`): `base/class/File` 64 of 64,
`SysFileDateTime` 8 of 8, `SysFileSearch` 14 of 14, `SysFileXXX` 43 of 43, `SysSearchPath` 10 of 10,
`platform/unix/tilde` 1 of 1, `samples/samples` 29 of 29, `SecurityManager` 18 of 29. At Task 2's
start the first six had 62, 0, 0, 19, 0, 0 passed. SecurityManager's `TEST_CALL_PASS_THREE_ARGS`
(this task's) passes; its 11 others are the 6 blocked on `SYSDROPFUNCS` (Task 7) and the 5 failed
listed in `suite-9d-set.txt` (9d). The suite record is at `92d2868fc`. The later commits change only
corpus witnesses, their `sourceline_oracle` files and `oracle-crashes.txt`.

The suite run created and deleted files in `~` (the tilde test's own `SysTempFileName('~/...')`);
none were left (`ls ~ | grep -c test_tilde` printed 0 after it). Five `~/test_tilde*` files my own
early probes left were deleted by literal path.

### Corpus witnesses

Six new programs in `rust/corpus/lang/`, listed in `rust/corpus/phase-9.txt`, each with its
`crates/rexx-parse/tests/sourceline_oracle/<name>.txt` generated by that test's driver from an
empty directory (nothing was left there). Each was run on the oracle first and its stdout read;
each ends with one untrapped error so the rendering is compared. Each makes its files under a
directory it `mkdir`s in the current directory and removes it (the closed directories are
`chmod 700` again first).

- `rexxutil_file_copy_move.rex`: Review Focus 2's shapes (blank, `c3a9` and `ff` names with the
  lossy spelling shown absent, trailing slash, link to a directory, dangling link, a `000`
  directory, an unreadable file), times and mode kept (`stat`), links followed by a copy and kept
  by a move, the errors.
- `rexxutil_file_tests.rex`: the four predicates over every shape plus `.`, `..`, `/`, `""`, NUL,
  `~`, `~root`, `~nosuchuser`; `QUALIFY` and `.File` with tildes and NUL; then `HOME` moved into
  the witness's directory so one file is reached as `~/home.txt` through the routines, `QUALIFY`,
  `.File`, `.Stream`, `LINEOUT`/`LINEIN`/`STREAM` and `DIRECTORY`. Its last line is
  `SysIsFileLink(.nil)`, the 88.909 rendering.
- `rexxutil_file_date_time.rex`: the scan and carry cases, skipped and repeated readings for both
  European and North American transitions with `stat -c %Y` beside them, years 202, -202, 999,
  1800, 1969, the access time kept, both halves omitted, selectors, links, a directory, a
  non-UTF-8 name, a `000` directory, the errors.
- `rexxutil_temp_file_name.rex`: what is compared is each answer's length against the qualified
  template, the count of places that differ (only filler places, each now a digit, `1000` added for
  any other difference), and whether the answer names anything; and `[]` for the empty answers
  (no filler, ten fillers, NUL before the fillers, unknown `~user`, every name taken).
- `rexxutil_search_path.rex`: answers shown relative to the current directory and the arguments
  with the current directory masked.
- `rexxutil_file_search.rex`: line ends, options, NULs, an array that grows while holding strings
  longer than the inline limit, stems, missing and unreadable files, the errors.

Review Focus 5: the date and time witness also runs under `TZ=Europe/Zurich` and
`TZ=America/New_York` on both sides as subprocesses, in
`datetime_zone.rs::the_file_date_time_witness_reads_the_zone_the_oracle_does`, which asserts all
three descriptors per zone and that the two zones' stdout differ. By hand it was also identical
under UTC, Australia/Lord_Howe, Asia/Kolkata, America/Sao_Paulo and Pacific/Chatham, and under
Europe/Dublin it differs on the one skipped-reading line (Concern 1).

### TDD evidence

RED: before the bodies, Task 2's baseline (`task-2-baseline/tests.tsv`) has every test of these
groups that reaches a new routine blocked by `routine "SYSxxx" is not implemented (Phase 9)`, and
`TEST_FILEEXISTS_NIL_INVALID` failed. My first probe (`SysFileExists(.nil)`, `QUALIFY('~root/a')`,
`QUALIFY('~nosuchuserx/a')`) differed: crate rc 0 and `<cwd>/~...` paths, oracle rc 168 with
88.909 and `/root/a`, `""`.

Reds found while writing, each fixed and then identical:

- `SysSetFileDateTime(f, '2020-10-25', '02:30:00')`: the crate set `1603589400` (chrono's second
  candidate), the oracle `1603585800`. Fixed by taking the earlier.
- `'2020-10-25', '03:00:00'` under Zurich: crate `1603587600`, oracle `1603591200`. Fixed by the
  read-back filter.
- The temp-name witness's `q??` NUL line: `QUALIFY` did not cut at NUL, the oracle's does. Fixed in
  `qualify_name`.
- The tilde test replica: `.File~new('~')`, then `.Stream~new('~/...')` opened the wrong path.
  Fixed by routing both through `qualify_name`.

GREEN: all six witnesses identical on all three descriptors, the six suite groups green, corpus
`1035 of 1035`.

### Mutations, each reversed by an exact edit

- Dropping `Found::add`'s `push_temp` of an array value: `collect_stress` stayed green, twice, the
  second time after the witness was given strings longer than the 54-byte inline limit so the
  values live in the heap while the array grows. `array_append` does not allocate a collected
  object, so nothing collects between `text_built` and the write. The hold is kept (as Task 3's
  ruling kept the sort's) and is not shown load-bearing. Restored from a copy; `git diff --stat`
  on the file empty, `grep -c "push_temp(value)"` 1.

## Derived rows

None record these groups: `grep -n -a -i "rexxutil\|samples/" corpus/sim-gate.tsv` names only
`SysSleep`, `concurrency_tests.rs`'s whole-group rows name only `SysSleep` among the RexxUtil groups
and exclude `samples/` whole. `suite-9d-set.txt` drops `base/rexxutil/SysFileXXX
TEST_FILEEXISTS_NIL_INVALID` and `samples/samples TEST_MAKESTRING` (both pass above), with a header
line saying so, since the file no longer equals its awk derivation. `phase-9b-gate.md`'s two line
citations into that file (`:200-204`, `line 198`) were false after the removal and now name the
file without line numbers. `refusal-sites.tsv` re-derived in `92d2868fc`.

## Files changed

- `rust/crates/rexx-exec/src/builtin/rexxutil/files.rs` (new), `files/tests.rs` (new)
- `rust/crates/rexx-exec/src/builtin/rexxutil.rs` (`mod files`, helpers, `path_argument` on bytes)
- `rust/crates/rexx-exec/src/builtin/rexxutil/stems.rs` (helpers moved out, dump path qualified)
- `rust/crates/rexx-exec/src/builtin/platform.rs` (`qualify_name`, `~user`, `QUALIFY`, `DIRECTORY`)
- `rust/crates/rexx-exec/src/paths.rs` (`normalize_bytes`), `lib.rs` (`cwd_bytes`)
- `rust/crates/rexx-exec/src/dispatch/files.rs`, `dispatch/stream.rs`, `redirect.rs`, `run.rs`
  (qualification through `qualify_name`)
- `rust/crates/rexx-exec/src/dispatch/collection.rs` (`Interp::array_append`)
- `rust/crates/rexx-exec/src/error.rs` (three constructors), `internal_routines.rs` (nine rows)
- `rust/crates/rexx-exec/tests/datetime_zone.rs` (the zone-pinned witness test)
- `rust/corpus/lang/rexxutil_*.rex` (six new), `rust/corpus/phase-9.txt`,
  `rust/corpus/oracle-crashes.txt` (header, entry 39), `rust/corpus/refusal-sites.tsv`,
  `rust/corpus/suite-9d-set.txt`, `docs/superpowers/plans/phase-9b-gate.md`
- `rust/crates/rexx-parse/tests/sourceline_oracle/rexxutil_*.txt` (six new)

## Concerns

1. **Skipped readings in a negative-DST zone.** `Europe/Dublin`'s `2020-03-29 01:30:00` is set an
   hour later here than on the oracle. Fixing it needs each offset's `isdst` flag, which chrono
   does not expose, so a TZif reader (or a dependency) would be needed. Not licensed; the
   controller decides.
2. **Wider than the nine routines.** `QUALIFY`, `DIRECTORY`, `.File`, `.Stream`, the stream
   builtins and redirects now qualify through `qualify_name` (tilde, `~user`, NUL cut, `""` on
   failure). Each change is witnessed and the corpus is `1035 of 1035`, but it is a shared path.
   Streams still open their path through `from_utf8_lossy` (`dispatch/stream.rs`
   `qualified_path`), so a stream named with bytes that are not UTF-8 opens the wrong file; the
   witnesses read such files through a copy. Not fixed here.
3. **No callgrind run**, per the dispatch. `resolve_stream` (every stream builtin call) now calls
   `qualify_name` where it called `paths::normalize`: the same walk on bytes plus a NUL scan.
4. **The `push_temp` in `Found::add` is not shown load-bearing** (Mutations above).
5. **`SysIsFile` still excludes block devices**, where the oracle's `isFile` accepts `S_ISBLK`. This
   container has no block device to witness it with (`ls -l /dev | grep ^b` printed nothing), so
   it was left alone.
6. **`~user` reads `/etc/passwd`**, as `USERID()` already does, so an account known only through
   NSS is not found where the oracle's `getpwnam` would find it.

## Fix round 1

Commit `ab305b53e`, against `task-4-review.md` and the Task 4 rulings in `progress.md`. This
section replaces Concerns 1 and 5 and the Mutations paragraph above.

1. **The shared qualifier (ratified) is witnessed.** I ran the reviewer's `shared.rex`,
   `emptyredir.rex` and `nouserredir.rex` on the oracle first, then folded them into a new
   witness, `rust/corpus/lang/rexxutil_qualified_names.rex`. It is listed in `phase-9.txt` and
   works inside its own directory with `HOME` moved into it. It covers:
   - the stream builtins and `.Stream` with a NUL, with an unknown `~user`, and with `''`;
   - redirects to a NUL name and to `~/t.txt`;
   - trapped redirects to `''` and to `~nosuchuser.../x`, both 98.920 with `Unable to open file ""`;
   - an untrapped `''` redirect as its last line.
   Oracle and crate are identical on all three descriptors, rc 158.
2. **The date and time routines read the program's own `TZ`.** They now convert through the new
   `tz-rs` 0.7.3 dependency. It has `#![forbid(unsafe_code)]` and no dependencies of its own, and
   `Cargo.lock` gains only its entry.
   - `program_zone` resolves `TZ` from the interpreter's environment, as glibc does: unset reads
     `/etc/localtime`, a name a zoneinfo file, `:path` that path, and anything else a POSIX rule.
     A value that reads as nothing, the empty one included, is UTC.
   - The zone is read only inside the two routines, and only after the `stat` succeeds. No other
     call path changed.
   - `make_time` takes the offsets a day either side of the reading. A candidate that reads back
     as the same reading wins, and of two the earlier wins. For a skipped reading it takes the
     candidate whose own local time type `is_dst`, else the offset before the skip.
   - **TDD:** `dublin.rex` under `TZ=Europe/Dublin` and `tzenv.rex` under process
     `TZ=America/New_York` are now identical to the oracle. Their lines are now in
     `rexxutil_file_date_time.rex`, which sets `TZ` itself through `VALUE(..., 'ENVIRONMENT')`:
     - Dublin: the 2020, 2021 and 2045 gaps, the edges, and the repeated reading;
     - UTC and Kolkata;
     - the POSIX rule `IST-1GMT0,M10.5.0,M3.5.0/1`, `:Europe/Zurich`, `''` and `No/Such_Zone`.
   - By hand, the whole witness is identical under process `TZ` Zurich, New_York, UTC, Lord_Howe,
     Kolkata, Sao_Paulo, Dublin, Chatham, Casablanca and unset.
   - **Mutants**, each reversed from a copy and confirmed by `cmp`. Both made the witness differ
     from the oracle:
     - the gap rule fixed at the earlier offset, which reproduces the old Dublin answer:
       `1585445400` against `1585441800`;
     - `program_zone` ignoring the program's `TZ`: the Dublin and Kolkata lines.
3. **I dropped the `push_temp` at `files.rs:663`, because it cannot be load-bearing.** `Interp`
   collects only inside `alloc_with` → `collect_if_due`. Between `text_built` and the slot write,
   `array_append` → `array_grow` → `charge_growth` only marks a collection due. So no collection
   can fall in that window, and that is why the removal run stayed green.
4. **The entry-39 test now uses two `Scratch` directories**, each unique per run (pid plus a
   random key) and removed by `Drop`. A failed assertion leaves nothing behind. It still fails
   loudly where `target/` and `TMPDIR` are one file system.
5. **`datetime_zone.rs` compares stdout and stderr as bytes.** It renders them lossily only in the
   failure message.
6. **The `.File~new('n'||'ff'x)~absolutePath` case** is in `rexxutil_qualified_names.rex`. It
   answers raw `6EFF`, which equals both `QUALIFY` and `directory()'/n'||'ff'x`.

Checks at `ab305b53e`, from `rust/`, in the foreground:

| check | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `CARGO_INCREMENTAL=0 memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` | exit 0 |
| the lib and table command above, plus `no_process_state` | exit 0, lib 1093 passed, then 8, 2, 1, 6, 5, 3, 5 |
| `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-parse --test sourceline_oracle` | exit 0 |
| `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-exec --lib rexxutil::files` | 1 passed; no `rexxutil-move-*` left in `target/tmp` or `/tmp` |
| corpus gate command above | exit 0, `1035 of 1035` before this round, now `1036 of 1036 matching` |
| `... --test ir_recorded --test trace_oracle --test coverage --test datetime_zone` | exit 0: 28, 41, 21, 22 passed |
| the suite command above over the same eight groups | exit 101, the same verdicts as before: seven groups green, SecurityManager 18 of 29 (Task 7's 6 and 9d's 5) |
| `collect_stress` command above | exit 0, 37 passed |

The records in `docs/superpowers/records/2026-10-13-phase-9b/task-4/` were re-run at `ab305b53e`:
`run-output.txt`, the suite tables, `corpus-output.txt`, `collect-stress-output.txt` and
`other-tests-output.txt`.

Concerns left: 2 (other lossy path sites, which the controller is queuing), 3 (no callgrind; the
reviewer measured it), 4 is closed by item 3 above, and 6 (`/etc/passwd`). Concern 1 is closed.
