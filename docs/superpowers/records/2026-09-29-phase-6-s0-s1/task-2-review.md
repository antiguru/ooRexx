# Task 2 review: benchmarks, callgrind script, Phase 6 base

Range reviewed: `9d863ccc5..cc74f959c`. Not reviewed: `1f652a43d` ("Cut the Phase 6 benchmark
prose...", committed after the range, touching `callgrind.sh`, `cgsum.py`, `child.rs`, `lib.rs`,
all four programs and the record), and the worktree's uncommitted edits to `callgrind.sh`,
`cgsum.py`, `layout-pad.py` plus an untracked `rust/bench-programs/__pycache__/`, which were
present when this review started.

Review scratch: `$W = /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/p6-t2-review`.
No build was done; every probe below used the implementer's base binary
`p6-t2/bin/base/rexx-run`, sha256 `3a82550968f46b4d0a82d38151bda74469d1a89b3980c42d2cdd35a640fa85b0`
(re-hashed by me).

## Spec Compliance

Mostly compliant; one requirement is not in a committed record.

- ✅ Step 1 programs: `fibcall`, `fibfunc`, `sendloop`, `extcall` (`::requires 'orxfunction' LIBRARY`).
  fib(22) is repeated 30x for the `n =` loop-bound convention; a justified deviation.
- ⚠️ Step 1 "record the comparison": the oracle comparison exists only in
  `task-2-report.md`, which is untracked (`git ls-files .superpowers/sdd/2026-09-29-phase-6-s0-s1/`
  prints nothing). No committed file at `cc74f959c` records it (Issue I2).
- ✅ Step 2 script: two or more binaries, interleaved and rotated, `--compress-strings=no`,
  libc and ld-linux subtracted, one table. No worktree or scratch path (Issue M1 on the oracle default).
- ✅ Step 3: base and three layout controls, each its own target dir; each build log has exactly one
  `Compiling rexx-exec` line (`grep -c 'Compiling rexx-exec' p6-t2/logs/build-*.log` → 1 for all
  five); sha256 per binary recorded; noise band per program; five interleaved wall-clock runs.
- ⚠️ Step 3 "quote every command": three figures have no command (Issue M2).
- ✅ Spec section 7 programs all present (`rexxcps`, `emptyloop`, `varlookup` plus the four).

## Strengths

- `cgsum.py` asserts that its per-object sum equals callgrind's `summary:`, so a parse error cannot
  go unnoticed.
- `PROGRAMS` is checked against `*.rex` on disk.
- Every figure I spot-checked against `p6-t2/cg2/summary.tsv` reproduces: rexxcps base median
  17,920,429,749, t1pa 17,924,390,887 (+0.0221%), the extcall excursion 42,000,341 and the
  rexxcps excursion 7,554,142.
- The `nm -S` claim holds. `nm -S … | grep -vi layout_pad`, sorted by size, type and name,
  gives 3869 rows in all four binaries. The only differences are 7 symbols whose `.NNN` suffix is
  renumbered at equal size.
- The excursions are disclosed next to the band rather than hidden by the median.

## Named risks

1. **Instrument validity: the excursions come from the seeded std `HashMap` in `Table::register`.**
   The rate is higher than the record suggests. Command: `$W/ext3.rex` (extcall with
   `n = 300000`), 32 callgrind runs of base, 16 at a time, each from a fresh empty dir, with
   `LD_LIBRARY_PATH=<oracle>/build/lib`, then `cgsum.py`:

   ```
   exlibc min 1,046,422,815  (30 runs within 15,344 of each other)
   e13 1,047,929,159  (+0.144%)
   e23 1,050,630,329  (+0.402%)
   ```

   2 of 32 runs are excursions, and one of them is a different size (+5 Ir per call instead of +14).
   So the distribution has more than one outlier mode. `callgrind_annotate --inclusive=no`
   diffed e13 and e23 against e5. The whole excess is in hashbrown's `raw.rs` and SSE2 group code
   inlined into `<rexx_api::handles::Table>::register` (+600,000 in each, plus 600,000 / 300,000 in
   the SSE2 and intrinsics lines). That is probe-sequence work.
   `Table.entries` is `HashMap<usize, ObjRef>` with the default `RandomState`
   (`rust/crates/rexx-api/src/handles.rs:32`). The keys are deterministic ObjRef bits, so the
   per-process seed is the only nondeterministic input to that probing. I did not control the seed,
   so this is strong localisation, not proof.

   The rexxcps excursion has a candidate of the same kind. `resolve_fixed_call` calls
   `builtin::resolve`, which looks the name up in `rows()`, a `static HashMap<&[u8], u16>` with the
   default `RandomState` (`rust/crates/rexx-exec/src/builtin.rs:671`). An h2 tag collision there
   costs one extra key comparison (`memcmp`) per lookup, which matches "280,003 extra `memcmp`
   calls". The report's "its label lookup is a BTreeMap" is true but leaves out this seeded map on
   the same path. My probe did not reproduce it: `$W/bifs.rex`, 30 distinct builtins × 20,000, 12 runs,
   memcmp calls 306,144–306,191 and exlibc spread 15,503 Ir. Whether it is that map is unconfirmed.

   Judgement: median-of-3 is a sound first-order mitigation, but not a sufficient one.
   - At ~2/32 per run, P(≥2 of 3 contaminated) ≈ 3p² ≈ 1.1% per binary per cell.
   - A contaminated extcall median is +0.14% to +0.42%, the same order as the 0.3% budget.
   - The script computes `spread%` but flags nothing.
   - The **0.0000 band on extcall is honest only for uncontaminated medians**. The record discloses
     the excursions, but "Twelve further callgrind runs of base on `extcall` showed none" leaves a
     reader thinking the rate is far lower than 2 in 32.

   See Issue I1 for the fix.
2. **`LD_LIBRARY_PATH` on every rust axis.** It changes the rust side's environment on all axes:
   ~45 more bytes of env, so the initial stack shifts. ld.so also searches `build/lib` first for
   `libgcc_s.so.1` and `libc.so.6`. `ldd` resolves both to `/usr/lib` with and without it, so only
   the failed opens are added. Both effects touch `startup`'s wall clock and cross-sitting comparison
   with older `rexx-bench` records. The oracle side already carried the same variable
   (`child.rs:44`), so rust against oracle is now more symmetric, not less. Only `extcall` needs it:
   without it, `env -i rexx-run extcall.rex` fails with Error 98.903 "Unable to load library
   'orxfunction'", rc 158. The report discloses this (concern 3). Issue M3.
3. **Oracle comparison: ✅ reproduced.** Each program was extracted from `cc74f959c` and run from a
   fresh empty dir (verified empty after the run). Oracle:
   `(ulimit -v 1048576; LD_LIBRARY_PATH=<oracle>/lib <oracle>/bin/rexx F)`. Rust:
   `env -i LD_LIBRARY_PATH=<oracle>/lib rexx-run F`. All four gave
   `out:same err:same rc:0/0`, with 0 stderr bytes on the oracle.
   The implementer's `oracmp.sh` does the same with separate descriptors and fresh dirs.
4. **Traceability: mostly ✅.** Build commands, both sittings' commands and the tabulation are quoted.
   sha256 is recorded for all five binaries (re-hashed, match), and the Compiling lines are
   asserted and present. Exceptions are in Issue M2.
5. **Paths: ✅ for worktree and scratch.** `here` is derived from `$0`, and `mktemp -d` uses
   `$TMPDIR`. The only absolute path is the oracle default at `callgrind.sh:37` (Issue M1). The
   script refuses to run if `liborxfunction.so` is absent, so a clean checkout on this machine runs.

## Issues

### Critical

None.

### Important

**I1. `rust/bench-programs/callgrind.sh:14-18,123-131`: a bimodal instrument that the table does not flag.**
- What: extcall has an upward outlier mode of +0.14% to +0.42%, at about 2 in 32 runs of an
  unchanged binary (risk 1). The median absorbs one in three and nothing reports when it fails.
- Why: the S0/S1 budget is +0.3% beyond a 0.0000 band. A contaminated base or candidate median
  would read as a budget breach, or would mask a real +0.3% regression.
- Fix: this instrument change is within Task 2's scope and does not move the base. Have the script
  print a marked warning, and exit non-zero, for any cell whose spread exceeds a small fixed
  threshold (e.g. 0.01%, well above every band but startup/sayloop). The stage protocol then re-runs
  that cell. Also state the measured rate in `phase-6-perf.md` in place of or next to "Twelve further
  runs … showed none", with its command.
- Removing the source (a deterministic hasher for `Table.entries`, and possibly `builtin::rows()`)
  is the better fix but changes the interpreter after the base. `rustc-hash` is already a dependency
  of `rexx-core`, so no new external dependency is needed. That change would have to be its own
  measured change, or the base would have to be re-cut. It is the controller's call; I do not
  require it for Task 2.

**I2. Step 1's oracle comparison is not in any committed file.**
- What: the four-line `stdout=same stderr=same rc=0/0` result and its command are only in the
  untracked `task-2-report.md`. `phase-6-perf.md` records only extcall's stdout against the oracle,
  from the wall-clock run.
- Why: the brief says "record the comparison", and SDD records are committed by ruling.
- Fix: commit the report, or add the comparison and its `oracmp.sh` command to `phase-6-perf.md`.

### Minor

**M1. `rust/bench-programs/callgrind.sh:37`: hard-coded oracle path as the `REXX_LIB_DIR` default.**
- Not a worktree or scratch path. It duplicates `child::ORACLE_ROOT` (`child.rs:27`), and the script
  fails loudly if the path is absent, so it is acceptable. Noted only because it is the one absolute
  path in the script.

**M2. `docs/superpowers/plans/phase-6-perf.md:29,31,87-89,108-130`: figures without their command.**
- The `nm -S` equal-sizes claim (true, see Strengths) and the `.text` bytes column (from the
  implementer's `logs/binaries.txt`) have no quoted command.
- "Twelve further callgrind runs of base on `extcall`" has no command. I checked `p6-t2/seed/e*.cg`:
  12 runs, exlibc 9,910,525,081–9,910,533,309, so the claim is true.
- The wall-clock block leaves out the lines that define `S`, `LIB`, `out`, `PROGRAMS` and `prog()`.
  `LIB` matters because it is why the rust arms ran with `LD_LIBRARY_PATH` set. The report calls
  the block "quoted verbatim in the record"; it is abridged. Quote `wall.sh` in full, or commit it.

**M3. `rust/crates/rexx-bench/src/child.rs:57-66` and `lib.rs:83-89`: the library path is set on every axis, but only `extcall` needs it.**
- The environment change alters `startup` and the stack placement of every other axis against
  earlier `rexx-bench` sittings.
- Fix: accept it and name the break in comparability in a committed record (currently only in the
  report), or set it only on the `extcall` axis.
- Also, the test `a_rust_side_sets_no_engine_environment_and_names_its_arm` now passes for any
  environment made only of `LD_LIBRARY_PATH` entries, including an empty one. It no longer asserts
  that the library path is present, so the extcall prerequisite is not tested.

**M4. `phase-6-perf.md:91-99`: bands understate run-to-run spread on `startup` and `sayloop`.**
- The record itself shows round-to-round spread of 0.0139–0.0245% against bands of 0.0045% and
  0.0030%, so "outside its band" on these two programs does not mean a real change.
- This is harmless against a 0.3% budget, and the record already says it. No action is needed
  beyond I1's spread flag covering these cells too.

## Assessment

Task quality: **Needs fixes.** Fix I1 (flag high-spread cells, and state the measured excursion rate
with its command) and I2 (commit the oracle comparison). M2 is a small follow-on of the
quote-every-command rule. The measured figures I checked are correct, and the controls are what
they claim to be.
