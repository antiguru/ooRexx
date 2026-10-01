# Task 2 re-review: fix round 1

Range: `cc74f959c..72efa5121`, including `1f652a43d`. Nothing was built and no tests were run, because
the gates are green at `d1086bb6e` per the controller. Every probe used the implementer's binaries under
`p6-t2/r2/bin/` and copies of their summaries. The copies are in
`$W = <scratchpad>/p6-t2-rereview`. No implementer output directory was written to.

## Original findings

| finding | status | where |
|---|---|---|
| I1 bimodal instrument, no flag | Addressed | source removed (`1754a3b5a`); flag at `rust/bench-programs/callgrind.sh:131-138`; rate with command at `phase-6-perf.md:136-145` |
| I2 oracle comparison uncommitted | Addressed | `phase-6-perf.md:37-53` |
| M1 hard-coded oracle path | Unchanged, accepted as Minor | `callgrind.sh:33`; `wallclock.sh:31` adds the same default |
| M2 figures without command | Addressed | `nm -S` `phase-6-perf.md:32-35`, `.text` `:20`, repeat runs `:138-141`, wall clock `:163-166` (`wallclock.sh` committed) |
| M3 library path on every axis; test asserted no presence | Addressed | `rust/crates/rexx-bench/src/child.rs:65-84`; test `child.rs:347-362` |
| M4 bands understate spread | Addressed | covered by SPREAD; startup spread is now 0 |

## Rulings

| ruling | status | evidence |
|---|---|---|
| P8 every seeded map becomes rustc-hash; base moves; re-measure with three controls | Addressed | see risk 1; base `1754a3b5a`, pad1-3 rebuilt, one `Compiling rexx-exec` line in each of the four `r2/logs/build-*.log` |
| P9 extcall-only path; comparison and commands committed; spread > 0.01% fails | Addressed for rexx-bench (see N1 for `rexx-bench-band`) | `child.rs:83`; `callgrind.sh:135-140` |
| P10 wall-clock bar | Addressed | `phase-6-perf.md:198-199` |
| P11 hasher delta recorded | Addressed | `phase-6-perf.md:90-134` |

## Named risks

1. **FxHash conversion: clean.**
   - I ran my own enumeration, wider than the report's: all crates, not only `rexx-*`, with tests
     included. `git grep -nE '(^|[^A-Za-z_])Hash(Map|Set)\b' 72efa5121 -- 'crates/*/src/*.rs' 'crates/*/src/**/*.rs'`,
     with comment lines removed, gives the two lines the report names plus `plan/tests.rs:50,93`.
     Both named lines are legitimate. `NameMap` names `NameHasher = rustc_hash::FxBuildHasher`
     (`rexx-core/src/lib.rs:45`). `cloned` is generic over `S`.
   - `git grep` for `RandomState|DefaultHasher|ahash|IndexMap|DashMap|hashbrown|foldhash|BuildHasher|HashMap as|HashSet as`
     finds only the `NameHasher` alias and a doc comment. No aliased import escapes the pattern.
   - Outside `src/`, only `tests/` integration files still use std maps. All 31 `src/**/*tests.rs`
     files excluded by the report's `:!*tests.rs` are declared under `#[cfg(test)]`.
   - `git diff 1f652a43d 1754a3b5a -- crates` has three kinds of `+`/`-` line: type/constructor
     renames, `use` lines, and the `rustc-hash` manifest entries. No `sort`, `collect` or iteration
     line changed. So no ordering path that existed before can have been removed. Iterated maps that
     reach output (`NativeObject::keys`) went from random order to fixed order.
2. **Cargo.lock: no new package.** Across the whole range there are three `+ "rustc-hash",`
   dependency lines. The existing `[[package]] rustc-hash 2.1.2` (`Cargo.lock:600`) satisfies `"2.1.2"`.
3. **SPREAD flag: works, and does not fire on TIMED residue.** I copied each summary to `$W` and ran
   `callgrind.sh -T -o $W/<copy> NAME=BIN...` on it.
   - Old `cg2`: `SPREAD extcall pad2 0.4238%`, four `startup` lines (0.0114-0.0169%) and
     `SPREAD rexxcps pad2 0.0422%`, exit 1.
   - New `r2/cg` and `r2/rep32`: exit 0 with no SPREAD line. rexxcps prints 0.0001%, and heapshape
     prints 0.0000% (100 Ir). Both are two orders of magnitude under 0.01%.
   - The exit status is the heredoc's status (`callgrind.sh:140`), so `-T` exits 1 on a flag too.
4. **Library path: extcall only in rexx-bench, and asserted both ways.**
   - `Side::rust` env is empty.
   - `for_axis` is applied in the suite (`rexx-bench-suite.rs:318`) and in the arms
     (`arms.rs:503-504`). The criterion bench extends `library_paths` per program
     (`benches/interpreter.rs:41-44`).
   - `only_extcall_gets_a_library_path` checks equality on extcall and emptiness on every other
     `PROGRAMS` and `NOT_BENCHMARKED` name.
   - The one rust `Side` not routed through `for_axis` is N1.
5. **`1f652a43d` trim: no comment became false.**
   - `.rex` headers match their `n` (30, 30, 5,000,000, 3,000,000).
   - `callgrind.sh` and `cgsum.py` headers match the code.
   - The two rexx-bench comments that `1f652a43d` made about the library path were rewritten
     again in `44c465281` and are true at HEAD.
   - The trim changed four `.rex` files' comment bytes. No figure from before the trim survives,
     except the P11 table's `9d863ccc5` column; see the note below.
6. **Every figure has its command; old figures are gone.** I recomputed from the implementer's
   files:
   - hasher table: all 23 rows, from the quoted Python.
   - base spread: rexxcps 20,659, heapshape 100.
   - control deltas: +5,722 / -9,942 / -11,030 / +100.
   - repeat-run lines: from the quoted awk.
   - `cgdiff.py` excerpts: rexxcps r25→r3 +33 calls each, getenv +15477; heapshape self −14/+56/+58.
   - sha256 and `.text` bytes for all four binaries.
   - `nm -S` diff: 0 lines of 3865 for pad1-3. pad1 carries 9 `layout_pad` symbols, so the
     exclusion is live.
   - oracle comparison: four same/same/0/0 lines.
   - wall-clock table: re-tabulated from a copy with `-T`, and it matches the saved table.
   - wall-clock prose: 0 non-empty `.err` files of 235; only heapshape and rexxcps have more than
     one distinct stdout; extcall's oracle stdout equals base; load figures match `binaries.txt`.
   - Timestamps: `cg` 16:58 and `wall` 17:16 local fall between `44c465281` (16:51) and
     `e4928c929` (17:19), as line 3 states.
   - Old figures: the sitting-2 table, `t1pa` and the Task 1 comparison are gone. `9d863ccc5`
     appears only in the P11 section.
   - `d1086bb6e`: after normalising `lib.rs:N`, the `-` and `+` sides of `refusal-sites.tsv` are
     identical.

Checked, not filed: the P11 table's `9d863ccc5` column ran the `.rex` files as they stood at
`0696153b4`. Four of them lost comment bytes in `1f652a43d`. The difference is a one-time lexer cost
and far below the table's printed 0.01% precision.

## New issues

### Critical

None.

### Important

None.

### Minor

**N1. `rust/crates/rexx-bench/src/bin/rexx-bench-band.rs:153`: `Side::rust(...)` is not passed through `for_axis`.**
- `--axes extcall` is accepted, because `resolve` maps any name to `program_path`. The rust child
  inherits the caller's environment (`timing.rs:53-54` only adds entries), so extcall runs only when
  the caller has exported `LD_LIBRARY_PATH`.
- Without it the run stops with Error 98.903 (`Unable to load library "orxfunction"`, rc 158). I
  reproduced this with `env -u LD_LIBRARY_PATH p6-t2/r2/bin/base/rexx-run $W/ext3.rex`. With the
  oracle lib set, it runs with rc 0.
- The failure is loud: band prints the error and exits FAILURE. extcall is not in band's default
  axes.
- Before this round it worked, because every rust side carried the path. The report lists the
  suite, `rexx-arms` and the criterion harness as `for_axis` users; band is not listed.
- Fix: `let rust = Side::rust(rust_binary, arm);` → build it per axis inside the `for entry in &axes`
  loop with `.for_axis(&axis)`.

**N2. `rust/bench-programs/wallclock.sh:53-63`: exit status and stdout are not checked.**
- A run that fails is tabulated as a (short) time. `callgrind.sh` records `rc` and compares stdout;
  this script records neither.
- The committed sitting is clean: I checked that stderr is empty and stdout is consistent (risk 6).
  The next sitting has no such guard unless someone repeats that check by hand.

Observation, not filed: `callgrind.sh:79` and `wallclock.sh:57,60` put `LD_LIBRARY_PATH` on every
program for every arm. This is symmetric within a sitting. P9 addressed M3, which named rexx-bench,
so the two scripts are outside its letter.

## Verdict

**Approved.** Every original finding and ruling is addressed. N1 and N2 are Minor. Neither affects a
committed figure or the base.
