# Task 7 report: `rexx-exec/src/builtin/*` and `ir/*` over the limit

BASE `964a6a8db`. There are eight code commits. c1 to c6 came first, then
this report's first commit (`e104bc0ff`) and a first gate run
(`1a12e0059`). c7 and c8 followed team-lead's ruling on `string.rs`
(Departure 1), then this revision of the report and the second gate
run, whose results are below. Every artifact cited is under
`docs/superpowers/records/2026-09-15-file-split/task-7-files/` (`files/`
below). The tooling is Task 6's, adapted, in `files/tools/`.

## Commits

| # | commit | what moved | from, into |
| --- | --- | --- | --- |
| c1 | `43de18ad7` | `numeric.rs`'s test module | `builtin/numeric.rs` to `builtin/numeric/tests.rs` |
| c2 | `2c6abf8cc` | `convert.rs`'s test module | `builtin/convert.rs` to `builtin/convert/tests.rs` |
| c3 | `8905209f8` | `datetime.rs`'s test module | `builtin/datetime.rs` to `builtin/datetime/tests.rs` |
| c4 | `d31dbed5f` | `compile.rs`'s test module | `ir/compile.rs` to `ir/compile/tests.rs` |
| c5 | `ecd4fc295` | the op-stream `assert_*` checks | `ir/compile.rs` to `ir/compile/invariants.rs` |
| c6 | `e59418808` | the driver's `#[cfg(test)]` counters | `ir/drive.rs` to `ir/counters.rs` |
| c7 | `de1bfa3f4` | `string.rs`'s `mod tests` | `builtin/string.rs` to `builtin/string/tests.rs` |
| c8 | `7c8f9aa3b` | `string.rs`'s `mod scan_tests` | `builtin/string.rs` to `builtin/string/scan_tests.rs` |

The order follows the brief: tests-only moves first (c1 to c4), then the
leaf with one production caller (c5), then the counters (c6); c7 and c8,
tests-only, came after the ruling. Each commit
message says `git blame -w -C -C -C` recovers the moved lines.

Line counts use the plan's `find crates -name '*.rs' | xargs wc -l`:

| file | BASE | final |
| --- | --- | --- |
| `builtin/numeric.rs` | 1330 | 603 |
| `builtin/convert.rs` | 2092 | 1011 |
| `builtin/datetime.rs` | 2001 | 1165 |
| `builtin/string.rs` | 2084 | 1174 |
| `ir/compile.rs` | 2409 | 1722 |
| `ir/drive.rs` | 2590 | 2416 |
| new: `numeric/tests.rs`, `convert/tests.rs`, `datetime/tests.rs` | | 732, 1087, 848 |
| new: `string/tests.rs`, `string/scan_tests.rs` | | 893, 36 |
| new: `compile/tests.rs`, `compile/invariants.rs` | | 423, 294 |
| new: `ir/counters.rs` | | 197 |

## Where things went, and why

* **The four `builtin/` files' test modules and `compile.rs`'s** each
  became `mod tests;` (`string.rs`'s second, `mod scan_tests;`) and a file
  of their own under the parent's directory,
  with `move_tests_mod.py` (de-indent, verbatim inside literals). The
  module path is unchanged, so every `super::` path inside them still
  resolves and no test's fully qualified name changes.
* **`ir/compile/invariants.rs` (c5)** takes the nine op-stream checks
  `compile` runs over every stream it emits
  (`assert_trace_ops_open_a_clause_region` through
  `assert_region_ops_name_their_clause`), each widened from private to
  `pub(super)`. `compile.rs` declares `mod invariants;` and imports all
  nine, which is also how `compile/tests.rs`'s unedited
  `use super::{..., assert_...}` still resolves.
  * **`assert_analysis_only_narrows` stays in `compile.rs`**: it checks
    the `trace_flow` analysis against the plan under `debug_assertions`,
    not the op stream, so it is not one of the "op-stream invariant
    checks" the ruling names. I read the ruling narrowly.
  * **`COMPILE_CALLS` and its accessors stay with `compile`**, as the brief
    says: `count_compile_call` is called from `compile` itself, and its
    comment ties it to the chunk-cache test in `golden_tests.rs`. Nothing
    in the code argues for moving it.
* **`ir/counters.rs` (c6)** takes the test-only instrumentation after the
  op loop: every `thread_local!` counter, its recorder (`count_*`,
  `record_frame_floor`), its accessor, and `COUNTING` with
  `suspend_counters`/`resume_counters`. It is declared
  `#[cfg(test)] mod counters;` in `ir.rs`, so it does not exist in a release
  build. `drive.rs` keeps the op loop and `site_resolution_before_arguments`
  (the `impl Interp` block that sat between two counter blocks), and every
  `#[cfg(test)]` call site in both is byte-identical: `drive.rs` gains a
  `#[cfg(test)] use super::counters::{...}` of the recorders and accessors,
  and a `#[cfg(test)] pub(crate) use` of `resume_counters` and
  `suspend_counters`. The counters are a sibling of `drive` under `ir/`,
  not a child, as the brief names the file.
* **`ir/golden_tests.rs`, `builtin/state.rs`, `builtin/datatype.rs`**: out,
  as ruled.

**Visibility, each item with its reader** (Task 6's precedent is
`pub(in crate::<subtree>)` for a depth-2 item read from outside its
subtree; no item here needed that, since every reader is inside `ir`):

| item | file | visibility | read by |
| --- | --- | --- | --- |
| the nine `assert_*` checks | `compile/invariants.rs` | private to `pub(super)` | `compile` (calls all nine); `compile/tests.rs` through `compile.rs`'s import |
| `count_run_chunk_entry`, `count_clause_op_entry`, `count_trace_op_echo`, `count_arith_hint_skip`, `count_call_site_hit`, `count_const_build`, `count_load_constant_build`, `record_frame_floor` | `ir/counters.rs` | private to `pub(super)` | the op loop and `site_resolution_before_arguments` in `drive.rs`, through its `#[cfg(test)] use` |
| `run_chunk_entries`, `clause_op_entries`, `trace_op_echoes`, `arith_hint_skips`, `call_site_hits`, `const_builds`, `load_constant_builds`, `frame_floor_high_water` | `ir/counters.rs` | `pub(crate)` narrowed to `pub(super)` | `drive/tests.rs`'s unedited `use super::{...}`, through `drive.rs`'s import |
| `suspend_counters`, `resume_counters` | `ir/counters.rs` | `pub(crate)`, unchanged | `Interp::bootstrap_library` (`lib.rs:2055`, `:2075`) as `ir::drive::suspend_counters()`, through `drive.rs`'s `pub(crate) use` |
| `counting` and the statics | `ir/counters.rs` | private, unchanged | `counters.rs` only |

The `pub(crate) use` is the only re-export, and it is there because an
existing path (`ir::drive::suspend_counters`) must keep working. Clippy
`-D warnings` shows no import is unused.

## Pinned items

* **`refusal-sites.tsv` column 3 is asserted, not assumed.** Every commit
  re-derived the table with `REXX_REFUSAL_SITES_REFRESH=1` and diffed every
  column other than column 4, column 3 included, for all rows
  (`files/c<N>/c<N>-refusal-sites.txt`, each showing that the table's mtime
  moved), and `verify7.sh` refused to go on unless they were identical.
  The two production moves were first trialled in the test worktree
  (`tools/trial7.sh`, `files/c5/c5-trial-refusal-sites.txt`,
  `files/c6/...`). Every column, column 4 included, was identical at all
  six commits: no moved code constructs a refusal. Nothing needed a ruling.
* **Path pins** (`files/path-pins-base.txt`, the brief's two `grep`s per
  file, before any move):
  * `tests/builtin_status.rs:494`, "The names `src/builtin/string.rs`
    runs", and `corpus/oracle-crashes.txt:78`, "`find_forward`'s own doc
    comment in `rust/crates/rexx-exec/src/builtin/string.rs`": both still
    true, since no production code in `string.rs` moved (`find_forward`
    at `string.rs:86`, its dispatch table in the same file).
  * `corpus/phase-4c.txt:192`, "it is the transcribed table in
    builtin/string.rs": the test it means is
    `builtin::string::tests::the_padding_builtins_answer_the_oracles_own_bytes`,
    which was in `string.rs`'s inline `mod tests` and after c7 is at
    `builtin/string/tests.rs:63`. Departure 1 gives the ruling and the
    reading under which the sentence stays true.
  * `corpus/lang/string_extremes.rex:14` (and its copy in
    `rexx-parse/tests/sourceline_oracle/`), "`integer_object`'s doc in
    builtin/numeric.rs": `integer_object` is production code and stayed
    (`numeric.rs:308`).
  * `run/loops.rs:1364`, "(`ir/compile.rs`)": register allocation, which
    stayed in `compile.rs`.
  * No test, source or bench file names `ir/drive.rs`, `builtin/convert.rs`
    or `builtin/datetime.rs`; the corpus names none of them.
* No file-granular invariant touches these files: `unsafe_sites.rs`,
  `environment_seam.rs` and `dispatch_seam.rs` name none of them.

## Departures from the brief

1. **`builtin/string.rs` waited for a ruling, then moved (c7, c8).**
   Before moving anything I found the third path pin above, which the brief
   does not list. `corpus/phase-4c.txt:192` (read-only) says: "measured,
   removing the overrun reddens one test in the workspace and it is the
   transcribed table in builtin/string.rs". That test,
   `builtin::string::tests::the_padding_builtins_answer_the_oracles_own_bytes`,
   was in `string.rs`'s `mod tests`. I asked team-lead to choose: (a) move
   `mod tests` anyway and record the reading, or (b) leave it. No ruling had
   come by the time c1 to c6 and the first gate run were done, so the first
   version of this report left `string.rs` whole. The ruling was then (a):
   `phase-4c.txt` is a dated measurement record, and the file is not
   edited. **The reading under which the sentence stays true**: "the
   transcribed table in builtin/string.rs" is the oracle table that
   `find_forward`'s doc (and the padding builtins' docs) transcribe, and
   those stay in `string.rs`. The test that runs it through `dispatch` keeps
   its fully qualified name, which is how a reader finds it. Its new file
   is `builtin/string/tests.rs` (line 63). The test's own doc, "The oracle
   transcripts each of these lines came from are in the function's own doc
   comment", is plain prose with no intra-doc link. The docs it points to
   did not move, and `tests_links.sh` finds no warning in `string.rs` or
   `string/` before or after (`files/c7/c7-tests-links.txt`). c8 moved
   `mod scan_tests` the same way, as the ruling asked.
2. **Declared reflow in c4.** In
   `a_trace_op_outside_a_clause_region_is_refused`, the de-indent gave
   rustfmt room to join a two-element array onto one line, and it dropped
   the array's trailing comma. That is a token difference neither Task 6
   relaxation covers. `instruments.py` gained `--expect-reflow=KEY`: for a
   declared unit only, it compares whitespace-stripped text and tokens with
   every comma directly before a closing `)`, `]` or `}` dropped on both
   sides, and fails on anything else (controls 4 and 5 below). Literal
   values are unchanged (instrument 3).
3. **One comment corrected (c5).** `ir/drive.rs:568` named
   `compile::assert_region_ops_name_their_clause`. After c5 that path no
   longer reaches the function from `drive` (`compile`'s import is
   private), so the comment now reads
   `compile::invariants::assert_region_ops_name_their_clause`, which is one
   path segment inserted on one line, with no re-wrap. `other_edits.py`
   checks it as a `subst` rule (`files/c5/rules`). The line is a comment
   in the op loop's arm for `Op::Clause`; a comment does not reach codegen.
4. **Inserted lines in parents** (instrument 1 lists each): `mod tests;`
   (c1 to c4, c7), `mod scan_tests;` (c8); `mod invariants;` and its `use` (c5, `compile.rs`); the two
   `#[cfg(test)]` imports (c6, `drive.rs`); and
   `#[cfg(test)] mod counters;` (c6, `ir.rs`, an `insert` rule).

## The instruments

Per-commit outputs are in `files/c<N>/`, made by `tools/checks7.sh` over the
working tree before each commit (`tools/prod7.sh` then `verify7.sh`, then
`commit_task7.sh`). `commit_task7.sh` refused to commit unless the staged
`rust/` tree hash equalled the one instrument 4 ran on, and every commit's
hashes matched (`files/instrument4/i4-c<N>.meta`). `verify7.sh` also refused
a commit unless the `cargo doc` signatures were BASE's, the refusal-sites
columns were identical, the test-module doc run built (`Generated`), and
`files/c<N>/comment-pass.md` existed.

| # | 1 (a) unmoved | 1 (b) moved units | 2 tokens | 3 literals | 4 tests | load before / after instrument 4 |
| --- | --- | --- | --- | --- | --- | --- |
| c1 | 602 equal, 1 inserted | 31 of 33, 2 reflowed | 77 of 77 | 77 of 77; 699 literals; 0 control mismatches | identical | 1.81 4.71 5.21 / 2.25 5.26 5.40 |
| c2 | 1010 equal, 1 inserted | 36 of 37, 1 reflowed | 90 of 90 | 90 of 90; 1230; 0 | identical | 1.77 4.70 5.20 / 2.76 5.35 5.49 |
| c3 | 1164 equal, 1 inserted | 41 of 42, 1 reflowed | 127 of 127 | 127 of 127; 813; 0 | identical | 1.91 4.79 5.29 / 2.23 5.07 5.34 |
| c4 | 1992 equal, 1 inserted | 22 of 23, 1 declared reflow | 72 of 72 (1 declared) | 72 of 72; 226; 0 | identical | 1.22 3.83 4.85 / 2.63 5.01 5.13 |
| c5 | 1713 equal, 9 inserted | 9 of 9 (vis) | 50 of 50 | 50 of 50; 143; 0 | identical | 2.30 3.89 4.69 / 2.77 5.11 5.14 |
| c6 | 2406 equal, 10 inserted | 28 of 28 (16 vis) | 67 of 67 | 67 of 67; 219; 0 | identical | 3.19 4.61 4.95 / 2.35 5.05 5.20 |
| c7 | 1199 equal, 1 inserted | 25 of 26, 1 reflowed | 96 of 96 | 96 of 96; 1504; 0 | identical | 4.29 4.49 5.73 / 3.68 19.42 15.56 |
| c8 | 1173 equal, 1 inserted | 2 of 2 | 71 of 71 | 71 of 71; 144; 0 | identical | 2.91 17.01 14.93 / 3.89 6.63 10.04 |

"Reflowed" is `WHITESPACE-ONLY` in instrument 1 (b): rustfmt re-wrapped a
signature or a closure once the de-indent gave it four more columns. In
c1 and c2 one signature per file also lost its trailing comma, which
instrument 2 reports as `TRAILING-COMMA-ONLY` (Task 2's relaxation).
Instrument 4 gave 1582 test lines in 50 result blocks at BASE and at every
commit, compared per result block and per test
(`files/instrument4/`). The flake did not fire. The 5- and 15-minute
loads around c7 and c8 were high from other work on the machine; the
results were identical all the same. c1 to c4, c7 and c8 changed no file
outside their parent and destination (`files/c<N>/c<N>-other-edits.diff`
is empty). c5 (`drive.rs`) and c6 (`ir.rs`) changed one other file each,
under a rule `other_edits.py` checked (`files/c<N>/c<N>-other-edits-check.txt`).

**The comment pass**, per commit (`files/c<N>/comment-pass.md`, from
`tools/comments7.py`'s `files/c<N>/c<N>-comments.txt` and
`positional.py`'s `c<N>-positional.txt`). It lists every comment in
`rust/crates` and every line in `rust/corpus` that names a moved unit in
backticks, or names a touched file by path, and every positional word
inside the moved text, and says for each why it is still true. It found
one false comment, at c5 (Departure 3), and it is corrected in c5.

**Tooling changes, each with a control.**

* **`item-tool` keys a macro item that declares statics by those statics**
  (`macro thread_local/RUN_CHUNK_ENTRIES`). `drive.rs` has several
  `thread_local!` blocks, which were all keyed `macro thread_local`, and
  `instruments.py`'s duplicate-key assertion would have refused the file.
  I added the change before the first move, and re-ran every earlier
  task's controls with it (`files/controls-*-before-threadlocal.txt`).
* **`instruments.py`**: `--expect-reflow` (Departure 2).
* **`docsig.py`**: the `cargo doc` gate. Each warning becomes a
  `message | file` pair, without the line number. A move that shifts
  lines keeps the signature. A warning that appears, vanishes, or moves
  to another file changes it. The gate is that all three signatures
  (public, private, private with `--cfg test`) are BASE's. Task 6's gate
  of "0 located in dispatch files" could not carry over, because BASE
  already has private-doc warnings in `numeric.rs`, `datetime.rs`,
  `builtin.rs` and `ir.rs`.
* **`comments7.py`**: the comment-pass listing above.
* **`tests_links.sh` guard**: the first BASE run of it, on a
  `git archive` of `rust/` alone, failed in the build scripts (they read
  the C++ tree) and still printed "0 warnings". `verify7.sh` now refuses a
  commit whose test-module doc run did not print `Generated`. The BASE
  runs were redone on an archive of the whole repository
  (`files/base/base-tests-links-*.txt`).
* **`trial7.sh`**: the refusal-sites trial of a production move in the
  test worktree.

**Controls**, on trees fresh from `git archive`. Task 2's five, Task 3b's
four, Task 4's eight, Task 5's seven, Task 6's eight, and the other-edits
controls of Task 5 and Task 6 were run three times. The first run was
before the first move (`files/controls-*-before.txt`). The second was
after the item-tool change (`-before-threadlocal`). The third was at the
end (`files/final/controls-*-final.txt`, summary in
`files/final/controls-final.out`). The results were 5 of 5, 4 of 4, 8 of 8,
7 of 7 and 8 of 8, with 4 and 8 of the other-edits runs as expected, every
time. This task's own controls, `tools/controls_task7.py`, gave 10 of 10
(`files/final/controls-task7-final.txt`):

1. a moved `thread_local!` changes its value: I1b, I2, I3;
2. a moved `thread_local!` is deleted: I2;
3. two moved `thread_local!` blocks swap the statics they declare, so each
   key's tokens are unchanged and only the comment above differs: I1b;
4. the declared reflow unit also changes a token: I1b, I2, I3;
5. c4 without `--expect-reflow`: I1b, I2, so the declaration is what
   admits the reflow;
6. an unmoved `compile.rs` line changes: I1a;
7. `docsig.py`: a warning that moves to another file changes the
   signature, and the same warning at another line does not;
8. `comments7.py`: a comment in `lib.rs` naming `drive::run_chunk_entries`
   is listed;
9. `other_edits.py` with c5's rule: a second edit in `drive.rs` beside the
   declared substitution fails;
10. `other_edits.py` with c6's rule: an edited, rather than inserted, line
    in `ir.rs` fails.

**Re-run with the final tooling** on every commit pair, from `git archive`
trees (`tools/rerun7.sh`, arguments from each commit's `files/c<N>/args`,
output in `files/final/rerun-summary.txt`), c1 to c8: every verdict is PASS, and every
output is identical to the committed one.

**Cumulative** (`files/final/cumulative.txt`): for each parent, every BASE
unit is present exactly once at `7c8f9aa3b`, in the parent or a
destination. Each is identical modulo visibility, or is c4's one declared
reflow. **Comments** (`files/final/comments.txt`, BASE's seven touched
files against the same files plus the eight new ones at `7c8f9aa3b`): one
comment line was lost, the `drive.rs` line Departure 3 corrects. What
was added is the eight license headers, the two new module docs and the
corrected line. The new test files have no module doc, as in Task 6.

**`cargo doc --no-deps -p rexx-exec`**: 2 warning lines at every commit;
53 with `--document-private-items`, and 52 with `--cfg test` as well. The
signatures are identical to BASE's at every commit
(`files/c<N>/c<N>-doc-*-sigdiff.txt`, all empty). The public run covers
neither the moved test code nor `ir/counters.rs`; the `--cfg test` run
covers `counters.rs`, whose three intra-doc links resolve. For the moved
test modules, `tests_links.sh` (the `#[test]` lines stripped, private
items and `--cfg test`) gives BASE's result at every commit
(`files/base/base-tests-links-*.txt` against `files/c<N>/c<N>-tests-links.txt`):
nothing in `numeric`, `convert`, `string` or `compile`'s test code, and in
`datetime`'s the one BASE warning, `unresolved link to month_name`, at
`datetime.rs:1493` before and `datetime/tests.rs:341` after, the same
line of text. The intra-doc links in the moved code are enumerated in
`files/final/intra-doc-links.txt`, with the command.

## Performance

The measure is callgrind's `summary:` minus `libc.so.6` and `ld-linux`, in
two interleaved rounds. Every binary was built by `tools/perf_builds.sh`
from a `git archive` of the whole repository. BASE had its own target
directory. c4, c5, c6 and later c8 were built in turn in a second target
directory (`files/perf/meta.txt`, `builds.txt`). The `.text` sha256 hashes
are:

| revision | `.text` sha256 | bytes |
| --- | --- | --- |
| BASE `964a6a8db` | `c8244939...` (Task 6's final, as its code is) | 2522699 |
| c4 `d31dbed5f` | `c8244939...`, BASE's | 2522699 |
| c5 `ecd4fc295` | `bc09a49f...` | 2522699 |
| c6 `e59418808` | `bc09a49f...`, c5's | 2522699 |
| c8 `7c8f9aa3b`, final | `bc09a49f...`, c6's | 2522699 |

**c6, the driver commit, does not change the release `.text`**, as the
brief expected: its hash is c5's. The only commit that changes `.text` is
c5. It changes 4058 bytes and leaves the size alone. The text symbols and
their sizes (`nm -S -C`, hashes stripped) are identical between BASE and
c5 (`files/perf/nm-base.txt`, `nm-c5.txt`, `text-diff-base-c5.txt`). The
differing bytes are small displacements (for example 255 to 253), which
is consistent with the assert panics' location data moving to a new file
name in read-only data. I did not bisect further: nothing moved.

The callgrind runs below compare BASE with c6, measured before c7 and c8
existed. The final commit's `.text` is c6's byte for byte (c7 and c8 move
test code only), so the figures stand for the final commit. The c8
binary was built to check that, and was not run again.

| program | BASE round 1 | BASE round 2 | final round 1 | final round 2 | change |
| --- | --- | --- | --- | --- | --- |
| rexxcps | 17895018291 | 17895023736 | 17895013525 | 17895014043 | -0.00004% |
| nop | 9340094775 | 9340098776 | 9340097175 | 9340100531 | +0.00002% |
| assign | 19540359037 | 19540372040 | 19540371703 | 19540361266 | +0.00000% |
| emptyloop | 9285889935 | 9285880731 | 9285898653 | 9285879508 | +0.00004% |
| varlookup | 14842895292 | 14842893850 | 14842894233 | 14842899303 | +0.00001% |
| arith | 11519335804 | 11519337752 | 11519337572 | 11519338553 | +0.00001% |
| compound | 9234402826 | 9234403271 | 9234397870 | 9234414502 | +0.00003% |
| dispatch | 20461074538 | 20461077340 | 20461075546 | 20461078301 | +0.00000% |
| strings | 17737058899 | 17737066891 | 17737061908 | 17737062352 | -0.00000% |

No axis moves more than a ten-thousandth of a percent, far under the
brief's 0.5%, so nothing was bisected. Every program's stdout is
identical between the binaries except `rexxcps`'s wall-clock
clauses-per-second line (`files/perf/stdout/`). The load average was 2.41
3.37 4.45 before the runs and 6.96 6.79 5.72 after, with 12 runs in
parallel.

## Gates

**Second run, the gates of record**, over `d000efebe`, this report's
second commit, whose code is identical to c8. It used the same script
and procedure, and the tree was unchanged from start to finish
(`files/gates-final/status.txt`, `test-*.results`,
`corpus-differential-*.txt`).

| gate | command | result | load average (1, 5, 15 min) |
| --- | --- | --- | --- |
| G1 | `cargo fmt --all --check` | exit 0 | 0.79, 3.98, 8.39 before the run |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings`, empty target dir | exit 0 | |
| G3 | `cargo build --workspace --all-targets --release`, no cap | exit 0 | |
| G4 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release --no-fail-fast` | exit 0; 135 result blocks; 2663 passed, 0 failed, 4 ignored; `corpus_differential` 604 of 604, STRICT | 15.16, 9.47, 9.85 before (G3's build had just finished); 2.03, 6.52, 8.73 after |
| G5 | `cargo build --workspace --all-targets` (debug) | exit 0 | |
| G6 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` (debug) | exit 0; 135 result blocks; 2664 passed, 0 failed, 4 ignored; `corpus_differential` 604 of 604, STRICT | 2.02, 6.44, 8.70 before; 2.20, 5.52, 7.85 after |

Every figure matches BASE's. The flake passed in both runs, so nothing was
re-run.

**First run**, over `e104bc0ff` (code identical to c6), before c7 and c8.
`files/tools/gates.sh` ran it after the commit. The tree did not change
from start to finish: `git status --short` was empty before and after.
The statuses, each read unpiped, are in `files/gates/status.txt`. G3 to
G6 use the default target directory, so the arity suites ran the binary
G3 built.

| gate | command | result | load average (1, 5, 15 min) |
| --- | --- | --- | --- |
| G1 | `cargo fmt --all --check` | exit 0 | 0.94, 4.34, 4.95 before the run |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings`, empty target dir | exit 0 | |
| G3 | `cargo build --workspace --all-targets --release`, no cap | exit 0 | |
| G4 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release --no-fail-fast` | exit 0; 135 result blocks; 2663 passed, 0 failed, 4 ignored; `corpus_differential` 604 of 604, STRICT | 15.54, 9.27, 6.68 before (G3's build had just finished); 1.87, 6.12, 6.26 after |
| G5 | `cargo build --workspace --all-targets` (debug) | exit 0 | |
| G6 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` (debug) | exit 0; 135 result blocks; 2664 passed, 0 failed, 4 ignored; `corpus_differential` 604 of 604, STRICT | 1.80, 6.04, 6.23 before; 2.12, 5.77, 6.37 after |

Every figure matches BASE's. The counts come from `tools/test_results.py`
run over each gate's own log (`files/gates/test-release.results`,
`test-debug.results`). The 604 comes from that log's differential report
(`files/gates/corpus-differential-*.txt`).
`introspection_arity::every_unstable_row_is_really_unstable` passed in
both runs, so nothing was re-run.

## Concerns

1. **Files still over the trigger** after this task: `ir/drive.rs` (2416)
   is the op loop, one `match`, which the ruling does not cut.
   `ir/compile.rs` (1722) is the one compile pass and its emit helpers.
   `builtin/string.rs` (1174), `builtin/datetime.rs` (1165) and
   `builtin/convert.rs` (1011) are each one family of builtins.
   `builtin/convert/tests.rs` (1087) is one set of test cases for that
   family. `ir/golden_tests.rs` (1637) is out by ruling.
2. `ir/counters.rs` keeps each item's own `#[cfg(test)]`, which is now
   redundant under the module's. Removing them would not be a pure move,
   and clippy does not flag them.
3. The `args` files of c1 to c3 (each commit's instrument arguments, which
   `rerun7.sh` reads) are committed with the reports and not with their
   commits. From c4 on, each commit carried its own.
