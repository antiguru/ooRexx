# Task 2 review (61280aaeb..ed0923fcc)

### Spec Compliance

- ✅ Spec compliant: `rust/corpus/suite-9b-groups.txt` is the spec section 4 derivation (routine list = section 4's
  list plus `SysFileTree`, `SysFileXXX`, checked name by name against spec lines 390-395), minus the row-derived
  Phase 10 group; the test re-derives it; the starting verdicts are recorded with command and wall time.
- ⚠️ Cannot verify from the raw output: the "Wall time 29 s" and "Exit status 101" in `phase-9b-gate.md:24`.
  `run-output.txt` has the 26.78 s build and `finished in 2.05s` and ends in `error: test failed` (consistent with
  101) but carries no wall-clock or status line.

### Checks run

- Real grep: `/usr/bin/grep -rlawiE '<ROUTINES>' ootest/ooRexx` (GNU grep 3.12, not the ugrep shell function) gives
  20 files: the 18 listed groups, `base/rexxutil/Macrospace.testGroup`, `base/rexxutil/platform/unix/SysFileCopyMove.sh`.
  Same 20 without `-a`, under `LC_ALL=C` and `LC_ALL=C.UTF-8`, and as the union of 32 single-routine `grep -rlawi` runs.
- Word-boundary semantics: an independent Python re-implementation (ASCII `[A-Za-z0-9_]` lookarounds, `re.I`, every
  regular file, symlinks skipped) gives the same 20 files; no routine-name occurrence in `ootest/ooRexx` has a
  non-ASCII byte adjacent to it, so the Rust ASCII-only word rule and grep's UTF-8 locale rule cannot disagree on
  this tree. `find ootest/ooRexx -type l` is empty, so the not-following-symlinks choice is moot here.
- `SysFileCopyMove.sh` is referenced by nothing in `ootest/` (grep hit only `.svn/wc.db`), so dropping non-group
  hits loses no group.
- Phase 10 exclusion is derived: `phase_10_group` (`suite.rs` new mod, `phase_10_group`) reads `read_rows()` over
  `suite_rows::FILES` (exclusions, licences, not-run) and `expected_set_in`; Macrospace's rows are in
  `docs/superpowers/plans/phase-9-exclusions.txt`. No group name is hand-listed in the test.
- Staleness: a missing `ootest/ooRexx` panics in `files_below` (`read_dir ... unwrap_or_else(panic!)`); an empty tree
  derives an empty list and fails the equality. The test is a plain `#[test]`, not env-gated.
  `CARGO_INCREMENTAL=0 timeout 300 memcap 8G cargo test -j 4 -p rexx-exec --test suite gate_groups_9b`: exit 0,
  2 passed.
- Baseline: recounting `task-2-baseline/tests.tsv` per group (passed / blocked / failed / not started) reproduces
  every row of the table in `phase-9b-gate.md:34-53`; `verdicts.tsv` has 16 red, 2 green, `equal` throughout.
- Owners: SysFileTree's 17 failures are exactly the shapes Task 5 (`### Task 5: SysFileTree`) lists.
  `SysFileXXX.TEST_FILEEXISTS_NIL_INVALID` is named in Task 4 (`### Task 4: File routines`). SecurityManager's 5
  failures have no 9b owner (below).

### Strengths

- The derivation is re-run, not re-listed: the header's command is rebuilt from `ROUTINES` and asserted
  (`parse`/`command`), so the const and the recorded command cannot drift apart.
- `has_word` handles a later in-line match after an earlier non-word one (`xSysStemSortx SysStemSort(`) and the
  `SysGetMessage`/`SysGetMessageX` prefix case, both pinned in `a_word_match_is_grep_dash_w_dash_i`.
- The baseline record keeps every raw artifact the table is computed from.

### Issues

#### Critical (Must Fix)

None.

#### Important (Should Fix)

- `phase-9b-gate.md:56` "The remaining red groups are the 9b work." is false for
  `base/security.manager/SecurityManager`. Its 5 failed tests (`TEST_ENVIRONMENT_HANDLE`,
  `TEST_METHOD_HANDLE_NO_RESULT`, `TEST_METHOD_HANDLE_RESULT`, `TEST_METHOD_PASS`,
  `TEST_STREAM_PASS_BIF_STREAM_OPEN`) test security-manager ENVIRONMENT/METHOD/STREAM events
  (`SecurityManager.testGroup:250,267,280,295,373`), call no RexxUtil routine, and are listed in
  `rust/corpus/suite-9d-set.txt:200-204`, which makes them Phase 9d's. No 9b plan task owns them (Task 7 only claims
  the `SysDropFuncs` blocks). As written, Task 10 Step 2's criterion "every Task 2 group passes under the harness
  apart from rows the exclusion test accepts" cannot be met by 9b. Fix: delete or correct the sentence, and the
  controller rules how the 9b close judges this group (9d-set tests accepted at 9b close, or the group judged on
  its 9b tests only). Ruling (b) keeps the group in; it does not say whose its non-RexxUtil failures are.

#### Minor (Nice to Have)

- `suite-9b-groups.txt:170` "in path order": the list is byte order (`S` before `p`); `grep ... | sort` in the
  default `en_US.UTF-8` locale puts `platform/...` before `SysDumpVariables`. Say `LC_ALL=C sort` order.
- `suite-9b-groups.txt:165` says "at the revision in suite-revision.txt" but the test does not check the checkout's
  revision; it re-derives over whatever is checked out. Not a staleness hole (a changed tree fails the equality),
  just an unenforced sentence.
- Not this task's diff, for the controller: `SysFileXXX.TEST_FILEEXISTS_NIL_INVALID` is owned twice, by 9b Task 4
  and by `suite-9d-set.txt:198` (the 9d awk excludes the `SysFileTree` group but not this 9b witness spec section 4
  names).
- Wall time and exit status in the record are not in the committed raw output (see ⚠️).

### Assessment

**Task quality:** Needs fixes

**Reasoning:** The derivation and its test are correct against a real GNU grep and an independent re-implementation,
and the baseline table is exact; one recorded sentence is false and hides a gate group whose failures no 9b task owns.
