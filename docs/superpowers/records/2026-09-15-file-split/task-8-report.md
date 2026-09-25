# Task 8 report: `rexx-api`

BASE `5c7173fac`. There are two code commits. Every artifact cited is under
`docs/superpowers/records/2026-09-15-file-split/task-8-files/` (`files/`
below). The tooling is Task 7's, adapted, in `files/tools/`.

## Commits

| # | commit | what moved | from, into |
| --- | --- | --- | --- |
| c1 | `60c4f2b15` | `invoke.rs`'s test module | `invoke.rs` to `invoke/tests.rs` |
| c2 | `526f14761` | the per-code converters `TABLE` names, and their helpers | `values.rs` to `values/convert.rs` |

The order follows the brief: the tests-only move first (c1), then the
production move (c2). Each commit message says `git blame -w -C -C -C`
recovers the moved lines.

Line counts use the plan's `find crates -name '*.rs' | xargs wc -l`:

| file | BASE | final |
| --- | --- | --- |
| `rexx-api/src/invoke.rs` | 1144 | 201 |
| `rexx-api/src/values.rs` | 1988 | 1368 |
| new: `invoke/tests.rs`, `values/convert.rs` | | 951, 706 |

## What moved, and what stayed

* **`invoke.rs` (c1).** The inline `#[cfg(test)] mod tests` became
  `mod tests;` and `invoke/tests.rs`, with `move_tests_mod.py`
  (de-indent, verbatim inside literals). The module path is unchanged, so
  every `super::` path inside it still resolves and no test's fully
  qualified name changes.
* **`values.rs` (c2).** The private per-code converters, from
  `arglist_to_native` to `cstring_from_native`, move in source order with
  `move_units.py`, together with the three private helpers only they call
  (`signed`, `unsigned`, `instance`). What stays in `values.rs`: the
  codes, `Failure`, `Class`, `Value`, `Converted`, `descriptor`,
  `CStringPool`, the `Host` trait, `Numeric`, `Constants`, `Conversion`,
  `Activation`, the row types, `TABLE` with its `const fn` row builders,
  every public lookup over the table (`repr`, `rows`, `result_read`,
  `consumes_argument`, `takes_argument_list`, `to_native`, `from_native`),
  and `pointer_string` with its helper `is_nil_spelling` (Departure 1).
  `values.rs` declares `mod convert;` and imports the converters `TABLE`
  names. `convert.rs` imports what it reads from its parent
  (`Class`, `Conversion`, `Failure`, `MAX_WHOLENUMBER`, `RESULT_DIGITS`,
  `Value`, `pointer_string`); a child may read its parent's private items,
  so none of those widened.
* **`rexx-api/tests/values.rs`: out, as ruled.** It is the conversion
  table's own cases, one subject, and splitting an integration test file
  into modules would rename every test in it.
* **`rexx-api/src/ffi.rs`: out, by the plan.** It has no safe region to
  move.

**Visibility, each item with its reader** (Task 6's precedent is
`pub(in crate::<subtree>)` for a depth-2 item read from outside its
subtree; no item here needed it, since every reader is in `values`):

| item | file | visibility | read by |
| --- | --- | --- | --- |
| every `*_to_native` and `*_from_native` converter | `values/convert.rs` | private to `pub(super)` | `TABLE`'s rows in `values.rs`, through its `use convert::{..}` |
| `signed`, `unsigned`, `instance` | `values/convert.rs` | private, unchanged | the converters in `convert.rs` only |

There is no re-export. Clippy `-D warnings` shows no import is unused.

## Pinned items

* **`unsafe`** (`rexx-core/tests/unsafe_sites.rs`, read in full). Its two
  predicates, applied to code with any `//` comment cut off, are an
  opt-in (`allow(unsafe_code)` or `expect(unsafe_code)`) and a use
  (`unsafe {`, `unsafe fn`, `unsafe impl`, `unsafe trait`). Its lists
  name `ffi.rs` and `load.rs` in this crate, and neither was touched.
  `tools/unsafe_count.sh` applies the same predicates to the lines each
  commit took out of its parent and to the whole of each new file: 0
  matching at c1 (944 moved lines, `invoke/tests.rs`) and at c2 (636
  moved lines, `values/convert.rs`) (`files/c<N>/c<N>-unsafe.txt`).
  `verify8.sh` refused a commit whose count was not `NONE`. The test
  itself is in `rexx-core`, which instrument 4 does not run; it runs in
  the gates (G4, G6).
* **Path pins** (`files/path-pins-base.txt`, the brief's two `grep`s per
  file, before any move): no test, source or bench file names
  `src/invoke.rs` or `src/values.rs`, and the corpus names neither.
  `rust/corpus/phase-8.txt:214` names `crates/rexx-api/tests/values.rs`,
  which this task leaves alone, so the pin means what it meant.
* **`refusal-sites.tsv`**: each commit re-derived it with
  `REXX_REFUSAL_SITES_REFRESH=1` and compared every column but column 4,
  and the header lines, with the committed table
  (`files/c<N>/c<N>-refusal-sites.txt`, each showing the table's mtime
  moved): identical, 247 rows, at both commits, and column 4 unchanged
  too. `git status` showed no change to the table.
* `environment_seam.rs` and `dispatch_seam.rs` name neither file.

## Departures from the brief

1. **`pointer_string` stays in `values.rs`.** It sits in the middle of
   the converter list, but it is not a per-code converter: it is the
   public parser `pointer_string_to_native` calls, and
   `rexx-api/tests/values.rs` calls it as `rexx_api::values::pointer_string`.
   Moving it would have needed a `pub use convert::pointer_string;` to
   keep that path, and the brief admits re-exports only where they are
   needed. So `values.rs` keeps every `pub` item, `convert.rs` holds only
   private ones, and `pointer_string`'s private helper `is_nil_spelling`
   stays beside it. I read the brief's "per-code converter functions" as
   the functions `TABLE`'s rows name, plus the private helpers only they
   call. No ruling was asked for. Moving the two functions would be a
   one-commit follow-up if the controller reads it the other way.
2. **A declared reflow in c1.** With four columns freed by the de-indent,
   rustfmt joined the test module's
   `use crate::ffi::{CALL_CONTEXT, ..., seen,}` onto one line and dropped
   the comma before `}`. item-tool keys a `use` by its tokens, so the key
   changed and instrument 2 reported the unit as vanished. Two tooling
   changes admit exactly that. `instruments.py` (and `cumulative.py`)
   drop a comma before `}` from a `use` unit's key, never from its
   tokens. c1 then declares the unit with `--expect-reflow`, which after
   this task's fix admits a comma before `]` or `}` and nothing else.
   Controls B7 and B8 below show that the declaration is what admits it,
   and that a name dropped from the same list still fails.
3. **A module doc on the new production file.** `values/convert.rs` opens
   with two lines of `//!` saying what it holds, as Task 7's `invariants.rs`
   and `counters.rs` did. The new test file has none, as in Tasks 6 and 7.
4. **Inserted lines in parents** (instrument 1 (a) lists each): `mod tests;`
   (c1, `invoke.rs`); a blank line, `mod convert;` and the
   `use convert::{..}` list (c2, `values.rs`). No other file changed at
   either commit (`files/c<N>/c<N>-other-edits.diff` is empty).

## The instruments

Per-commit outputs are in `files/c<N>/`, made by `tools/checks8.sh` over the
working tree before each commit (`tools/prod8.sh`, which runs `verify8.sh` and then
`commit_task8.sh`). `commit_task8.sh` refused to commit unless the staged
`rust/` tree hash equalled the one instrument 4 ran on, and both commits'
hashes matched (`files/instrument4/i4-c<N>.meta`). `verify8.sh` also refused
a commit unless: fmt and clippy passed (`-p rexx-api -p rexx-exec
--all-targets -D warnings`); the `cargo doc` signatures were BASE's; the
refusal-sites columns were identical; the test-module doc run printed
`Generated`; the unsafe count was `NONE`; and `files/c<N>/comment-pass.md`
existed.

| # | 1 (a) unmoved | 1 (b) moved units | 2 tokens | 3 literals | 4 tests | load before / after instrument 4 |
| --- | --- | --- | --- | --- | --- | --- |
| c1 | 200 equal, 1 inserted | 90 of 93, 2 reflowed, 1 declared reflow | 105 of 105 (1 declared) | 105 of 105; 248 literals; 0 control mismatches | identical | 27.46 18.45 14.37 / 49.71 57.76 44.31 |
| c2 | 1352 equal, 16 inserted | 38 of 55 (52 vis), 17 reflowed | 167 of 167 | 167 of 167; 477; 0 | identical | 69.26 58.57 46.84 / 55.36 72.62 62.26 |

"Reflowed" is `WHITESPACE-ONLY` in instrument 1 (b). In c1, two
multi-line `use` lists of the test module were re-broken by rustfmt, with
their trailing commas kept. In c2 there are seventeen `*_from_native`
signatures: `pub(super) ` made each wider than a line, so rustfmt broke
the parameters onto lines of their own and added the trailing comma, which
instrument 2 reports as `TRAILING-COMMA-ONLY` (Task 2's relaxation).
Instrument 4 ran the brief's
`cargo test -j 4 --release -p rexx-api -p rexx-exec --no-fail-fast`. It
gave 1761 test lines in 59 result blocks at BASE and at both commits,
compared per result block and per test (`files/instrument4/`). It ran
with the worktree's default target directory, since
`tests/support/arity.rs` runs `rust/target/release/rexx-run` by path. The
load averages were very high from work outside this sandbox, and the
results were identical all the same. No oracle-backed test timed out.

**`cargo doc --no-deps -p rexx-api`**: 0 warnings at BASE and at both
commits, in all three runs (public, `--document-private-items`, and that
with `--cfg test`). The signatures are identical (`files/c<N>/c<N>-doc-*-sigdiff.txt`,
all empty). For the moved test module, `tests_links.sh` (the `#[test]`
lines stripped, private items, `--cfg test`) gives 0 warnings in
`invoke.rs` and `invoke/` at BASE and at c1 (`files/base/base-tests-links-invoke.txt`,
`files/c1/c1-tests-links.txt`), and the same for `values` at c2. The
intra-doc links in the moved code are enumerated, with the command, in
`files/final/intra-doc-links.txt`: three in `invoke/tests.rs` (to the test
module's own `LONGEST` and `run`, and to `crate::ffi::refusing_stub`), and
none in `values/convert.rs`.

**The comment pass**, per commit (`files/c<N>/comment-pass.md`, from
`comments7.py`'s `c<N>-comments.txt` and `positional.py`'s
`c<N>-positional.txt`), says for each hit why it is still true. It lists:

* every comment in `rust/crates`, and every line in `rust/corpus`, that
  names a moved unit in backticks or names a touched file by path;
* every positional word in the moved text;
* every positional word left in the parent, which is new in this task.

It found no false comment, so nothing was corrected.

## Tooling changes and controls

Every change was made before the first move, except the `use`-key
normalisation, which c1 prompted. Every control set was re-run after
it, before c1 was committed.

* **`comments7.py` lists positional words in the parent too** (Task 7's
  review: a positional comment left in the parent that names no moved
  unit was missed). Section (c) now covers the parent as well as the
  destinations. It also takes the crate from `SPLIT_CRATE`.
* **`instruments.py --expect-reflow` admits a comma before `]` or `}`
  only** (Task 7's review, M2: it also accepted `(x)` becoming `(x,)`,
  a parenthesised expression turned into a 1-tuple).
* **`instruments.py` and `cumulative.py` normalise a `use` unit's key**
  (Departure 2).
* **`unsafe_count.sh`**: the unsafe-sites predicates over the moved lines
  and the new files.
* **Crate parameter**: `SPLIT_CRATE` (default `rexx-exec`) in
  `comments7.py`, `other_edits.py`, `docs.sh`, `tests_links.sh`. The
  per-commit pipeline is new, adapted from Task 7's: `prep8.sh`,
  `tmove8.sh`, `checks8.sh`, `verify8.sh`, `prod8.sh`, `commit_task8.sh`,
  `run_tests8.sh`, `i48.sh`, `rerun8.sh`, `run_controls8.sh`,
  `perf_builds8.sh` and `perf8.sh`.

**Controls**, on trees fresh from `git archive`. They are Task 2's five,
Task 3b's four, Task 4's eight, Task 5's seven, Task 6's eight, Task 7's
ten, and the other-edits controls of Task 5 and Task 6. They were run
four times:

* before any tooling change (`files/controls-*-before.txt`, summary
  `files/controls-before.out`);
* after the gap fixes (`-tooling`);
* after the `use`-key change, before c1 (`-before-first-move`);
* at the end (`files/final/controls-*-final.txt`, summary
  `files/final/controls-final.out`).

Every run gave 5 of 5, 4 of 4, 8 of 8, 7 of 7, 8 of 8 and 10 of 10, with
the other-edits runs at 4 and 8 as expected. This task's own controls are
`tools/controls_task8.py`. Part A ran in each of the last three runs
above, and part B once its commits existed, 15 of 15 at the end
(`files/final/controls-task8-final.txt`):

* A1. Task 7's c4 with `--expect-reflow` and the declared unit's argument
  turned from `(x)` into `(x,)` fails I1b and I2. Task 7's own
  `instruments.py` passes the same trees, which is the gap.
* A2. Task 7's c4 as committed still passes with the flag.
* A3. A positional comment planted in Task 7's c6 parent (`drive.rs`),
  naming no moved unit, is listed. Task 7's `comments7.py` does not list
  it.
* A4. `unsafe_count.sh` finds `unsafe {` planted in a destination's code
  and `unsafe fn` planted in the parent's moved lines, and ignores the
  same text in a `//` comment.
* B (c1, c2 as committed): both pass.
* B1. c1: a string literal inside a moved test's `assert_eq!` arguments
  changes: I1b, I2, I3.
* B7. c1 without its `--expect-reflow`: I1b, I2.
* B8. c1: the reflowed import also loses a name: I2.
* B2. c2: a moved converter's body changes a token: I1b, I2.
* B3. c2: a moved converter is deleted: I2.
* B4. c2: an unmoved `TABLE` row of `values.rs` names a different
  converter: I1a.
* B5. c2: two moved converters swap their doc comments: I1b, I2, I3.
* B6. c2: the `reason = "..."` string inside a moved converter's
  `#[expect(..)]` attribute changes: I1b, I2, I3.
* B9. c2: the string a moved converter passes to `.expect(..)` changes:
  I1b, I2, I3.

**Re-run with the final tooling** on both commit pairs, from `git archive`
trees (`tools/rerun8.sh`, arguments from each commit's `files/c<N>/args`,
output in `files/final/rerun-summary.txt`): both verdicts PASS, and every
output is identical to the committed one.

**Cumulative** (`files/final/cumulative.txt`): for each parent, every BASE
unit is present exactly once at `526f14761`, in the parent or its
destination, identical modulo visibility, or c1's one declared reflow.
**Comments** (`files/final/comments.txt`, each BASE parent against the
final parent plus its new file): no comment line lost. What was added is
the two license headers and `convert.rs`'s two-line module doc.

## Performance

The measure is callgrind's `summary:` minus `libc.so.6` and `ld-linux`, in
two interleaved rounds, on `rexxcps.rex`, `nop.rex` and `dispatch.rex`.
Every binary was built by `tools/perf_builds8.sh` from a `git archive` of
the whole repository. BASE had its own target directory, and c1 then c2
were built in turn in a second one (`files/perf/builds.txt`). The `.text`
sha256 hashes are:

| revision | `.text` sha256 | bytes |
| --- | --- | --- |
| BASE `5c7173fac` | `bc09a49f...` | 2522699 |
| c1 `60c4f2b15` | `bc09a49f...`, BASE's | 2522699 |
| c2 `526f14761`, final | `bc09a49f...`, BASE's | 2522699 |

**Neither commit changes the release `.text` of `rexx-run`.** It is
byte-identical at all three revisions, and it is also Task 7's final
hash.

| program | BASE round 1 | BASE round 2 | final round 1 | final round 2 | change |
| --- | --- | --- | --- | --- | --- |
| rexxcps | 17895032083 | 17895032571 | 17895037718 | 17895027852 | +0.00000% |
| nop | 9340096834 | 9340094197 | 9340100782 | 9340106215 | +0.00009% |
| dispatch | 20461082233 | 20461073539 | 20461081318 | 20461077000 | +0.00001% |

No axis moves more than a thousandth of a percent, far under the brief's
0.5%, and the binaries' `.text` is identical. The differences are inside
the run-to-run spread, which reaches about ten thousand instructions
between two runs of one binary (`rexxcps`'s final rounds). Every program's stdout is identical between the
binaries except `rexxcps`'s wall-clock clauses-per-second line
(`files/perf/stdout/`). The load average was 117.92 89.77 71.27 before
the runs and 99.67 94.79 76.02 after, with 12 runs in parallel
(`files/perf/meta.txt`). Instruction counts do not depend on load. The
first attempt at these runs failed before measuring anything: `perf8.sh`
was given relative paths and `cd`s into its own directory. It was re-run
with absolute paths, and only that run is recorded.

## Gates

Over `41f32aa24`, this report's first commit, whose `rust/` tree is
c2's. `tools/gates.sh` ran them after the commit, with the default target
directory (`rust/target`), so the arity suites ran the binary G3 built.
The tree did not change from start to finish (`tree-changes 0` before and
after, `files/gates/status.txt`). Each status was read unpiped. The counts
come from `tools/test_results.py` over each gate's own log
(`files/gates/test-*.results`), and the 604 comes from that log's
differential report (`files/gates/corpus-differential-*.txt`).

| gate | command | result | load average (1, 5, 15 min) |
| --- | --- | --- | --- |
| G1 | `cargo fmt --all --check` | exit 0 | 15.30, 76.31, 80.89 before the run |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings`, empty target dir | exit 0 | |
| G3 | `cargo build --workspace --all-targets --release`, no cap | exit 0 | |
| G4 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release --no-fail-fast` | exit 0; 135 result blocks; 2663 passed, 0 failed, 4 ignored; `corpus_differential` 604 of 604, STRICT | 48.16, 65.33, 75.72 before; 7.59, 39.71, 62.58 after |
| G5 | `cargo build --workspace --all-targets` (debug) | exit 0 | |
| G6, first run | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` (debug) | exit 101; 135 result blocks; 2663 passed, **1 failed**, 4 ignored; `corpus_differential` 604 of 604, STRICT | 7.22, 39.10, 62.26 before; **269.84, 352.70, 275.78** after |
| G6, rerun | the same command, same commit and target directory (`tools/gate6_rerun.sh`) | exit 0; 135 result blocks; 2664 passed, 0 failed, 4 ignored; `corpus_differential` 604 of 604, STRICT | 7.71, 39.83, 128.52 before; 6.73, 19.84, 93.02 after |

**G6's first run failed on an oracle timeout, in a run that ended at a
load average of 270 (353 over five minutes).** The one failure was
`method_bodies::no_row_started_diverging_or_stopped_answering`
(`files/gates/g6-first-failure.txt`). Its message says that for
`Directory intersection` and `IdentityTable intersection`, "the oracle did
not finish: TimedOut". Under the plan's quiet-machine bullet that is
machine state until a quiet rerun says otherwise. So I waited until the
one-minute load was under 15 and the five-minute load under 40, and re-ran
the whole of G6 rather than the one test, since this is not the flake the
brief allows to be re-run alone. The rerun is green. Compared with the
first run per test and per result block
(`files/gates/g6-first-vs-rerun.compare`), the only difference is that
test and its block. Every figure of G4 and of the G6 rerun matches BASE's.
`introspection_arity::every_unstable_row_is_really_unstable` passed in all
three test runs, so nothing was re-run for it.

## Concerns

1. **Files still over the trigger** after this task:
   * `values.rs` (1368) is one table-driven module: the codes and types,
     the `Host` trait the conversions call back through, `Activation`,
     `TABLE` with its builders, and the public lookups over the table.
     The survey keeps `TABLE` beside its builders.
   * `rexx-api/tests/values.rs` (2114) is out, as ruled.
   * `ffi.rs` (1041) is out, by the plan.
2. **Machine load.** Load averages ran from 27 to over 100 during the
   instrument 4 runs and the performance runs, and above 350 during G6's
   first run, from work outside this sandbox. No instrument 4 test timed
   out, and every comparison was identical. G6's first run lost one
   oracle-backed test to a timeout, and a quiet rerun of the whole gate
   was green (Gates).
3. c1 and c2 carry their `args` files with their commits, so
   `rerun8.sh` reads each commit's own arguments.
