# Task 5 report: the other `rexx-exec/src/` top-level files

BASE `1b851ae96`. Eleven code commits, then this report's commit. Every
artifact cited is under
`docs/superpowers/records/2026-09-15-file-split/task-5-files/` (`files/`
below). The tooling is Task 4's, adapted, in `files/tools/`.

## Commits

| # | commit | what moved | child `wc -l` | parent before, after |
| --- | --- | --- | --- | --- |
| c1 | `c32e442c1` | `value.rs`'s test module, to `value/tests.rs` | 965 | 2046, 1089 |
| c2 | `711ba036f` | `plan.rs`'s test module, to `plan/tests.rs` | 1013 | 1820, 818 |
| c3 | `5de681a23` | `builtin.rs`'s test module, to `builtin/tests.rs` | 363 | 1319, 965 |
| c4 | `c9e6cf3ef` | `eval.rs`'s `tests`, to `eval/tests.rs` | 1138 | 3338, 2206 |
| c5 | `838c11687` | `eval.rs`'s `object_operand_tests`, to `eval/object_operand_tests.rs` | 620 | 2206, 1592 |
| c6 | `efa2a8316` | `parse_template.rs`'s test module, to `parse_template/tests.rs` | 573 | 1516, 956 |
| c7 | `facf866a4` | the version constants, from `parse_template.rs` to a new crate-root `version.rs` | 81 | 956, 889 |
| c8 | `8a59de05c` | `environment.rs`'s test module, to `environment/tests.rs` | 267 | 2571, 2314 |
| c9 | `fe7b204ff` | output routing and trace-line delivery, to `environment/route.rs` | 246 | 2314, 2088 |
| c10 | `d81fdab00` | the interpreter's object identities, to `environment/identities.rs` | 745 | 2088, 1367 |
| c11 | `61478d685` | `trace.rs`'s format half, to `trace/format.rs` | 619 | 1464, 869 |

Line counts are the plan's `find crates -name '*.rs' | xargs wc -l`, at
BASE and at `61478d685`. Each commit message says `git blame -w -C -C -C`
recovers the moved lines. The order is the brief's: the tests-only moves,
then `parse_template.rs`, `environment.rs`, `trace.rs`.

## What moved, file by file

* **`value.rs`, `plan.rs`, `builtin.rs`**: the test module only.
  `builtin.rs`'s `IMPLEMENTED` table stays. `builtin.rs`'s "for the unit
  tests in this module's children" stays true, since `builtin/tests.rs` is
  one of them.
* **`eval.rs`**: the two test modules, one commit each. The doc comment on
  `object_operand_tests` stays on its `mod object_operand_tests;`
  declaration. The evaluator `impl Interp` is not cut.
* **`parse_template.rs`**: the test module (c6), then the block from
  `PLATFORM` through `const fn part` (c7), moved verbatim into `version.rs`.
  `is_blank` and everything below it stays.
* **`environment.rs`**: the test module (c8); then `local_route` through
  `deliver_trace_line` (c9); then `record_package_class` through
  `executable_context_package` plus `library_routine_root_key`,
  `program_routine_root_key` and `annotation_root_key` (c10). The last three
  are called only by moved members. What stays: `EnvScope`, `mod env_seam`,
  the environment model and its tables and types (`EnvironmentModel`,
  `Annotated`, `PackageTable`, `TableValue`), `build_environment` through
  `directory_lookup`, the class-lookup steps and `.local`
  (`installed_class` through `set_directory_entry`), the package string
  tables, the native-collection helpers, `running_program`,
  `package_root_key`, `package_local_root_key`, `package_table_root_key`,
  `package_table_entries` and `default_object_name`. Those five free
  functions stay because members that stay call them, or code outside the
  module reaches them as `crate::environment::`.
* **`trace.rs`**: the block from `TraceMode` through `push_operator` (c11),
  moved verbatim into `trace/format.rs`. It holds the mode types and their
  parsing, and the byte layout of each prefix's line, including
  `push_clause`. The emission `impl Interp`, `Announced` and the test module
  stay.

## Pinned items and path mentions

* **`tests/environment_seam.rs`.** `mod env_seam`, `directory_lookup` and
  `set_directory_entry` stay in `environment.rs`. `env_seam::admit(` and
  `env_seam::directory(` each occur exactly twice, all in
  `src/environment.rs`, at every commit:
  `/bin/grep -rn 'env_seam::admit(\|env_seam::directory(' rust/crates/rexx-exec/src`
  gives four lines, all in `environment.rs`. The test passed in
  instrument 4 at every commit. The moved code reaches the seam only as
  `env_seam::Access::Direct`/`Resolve` and through `directory_lookup`, both
  private items of the parent that a child module can see.
* **`tests/support/mod.rs`** named `trace.rs` three times, as the file
  holding `push_clause`, `push_prefixed_blanks`, `push_quoted` and
  `push_quoted_tag`, and "every formatter". All of those moved to
  `trace/format.rs`, so c11 changes all three mentions to name that file
  (departure 2).
* **Prose naming a file for a moved item**, found with
  `/bin/grep -rn '<file>' rust/crates/*/tests rust/crates/*/src rust/corpus`
  before each file:
  * c10: `install.rs`'s comment pointing at `environment.rs`'s
    `record_package_class`, and `dispatch.rs`'s two doc comments saying
    `environment.rs` builds the package and `Method` objects. They now name
    `environment/identities.rs`.
  * c7: `dispatch/rexx_info.rs`'s intra-doc link `[`parse_template::VERSION`]`
    is now `[`version::VERSION`]` (departure 3).
  * Left alone because they are still true: `eval.rs`'s own "`eval.rs`'s
    `object_operand_tests`", and the same phrase in `corpus/phase-5a.txt`
    and in `corpus/lang/environment_object_in_a_raise.rex` and
    `environment_object_in_a_loop_header.rex`. The module is still declared
    in `eval.rs` and is still `eval::object_operand_tests`. The corpus `.rex`
    files may not be edited in any case (`rust/CLAUDE.md`, the
    `sourceline` recording). `tests.rs`'s "`environment.rs`'s
    `package_table_entries`" is also left alone, since that function stays.
    So is `run.rs:2348`'s "`trace.rs`'s own doc comment", which is on a
    member that stays.
  * The specs under `docs/superpowers/` are records of their day and were
    left alone.
* **`refusal-sites.tsv`** was re-derived at every commit
  (`files/c<N>/c<N>-refusal-sites.txt`, each showing that the table's mtime
  moved). Every column except column 4 is identical row for row across all
  247 rows, and the header lines are identical, at every commit. Each
  production move was first tried in a scratch worktree, and the trial
  refresh changed column 4 only. So nothing needed the controller's ruling.
  Column 4 changed at two commits:
  * c7: `mod version;` in `lib.rs` shifts the `lib.rs` `Loud` rows.
  * c11: `raised_invalid_trace_letter` and
    `raised_numeric_trace_interactive_only` move to `trace/format.rs`.

  Task 4 found that column 3 depends on where a constructor is defined
  (its free-function rule). Measured here, it did not change: both
  `raised_*` functions were already defined outside `error.rs`/`lib.rs`
  before c11, and c11's refresh leaves their column 3 as it was
  (`files/c11/c11-refusal-sites.txt`).
* `dispatch_seam.rs`'s `CLEARANCE_CONSUMERS` lists `src/dispatch/rexx_info.rs`,
  which c7 edits by renaming paths only (no mention of the seam's token is
  added or removed); the list is unchanged and every `dispatch_seam` test
  passed in instrument 4 at every commit. `unsafe_sites.rs` grants files in
  `rexx-api` and `rexx-core` only, none of them touched.

## Departures from the brief and the survey

1. **`trace.rs`'s test module stays in `trace.rs`.** The brief rules the
   format half out and does not name the tests. It reads two private
   associated consts, `TraceMode::LABELS` and `TraceMode::ALL`, which now
   live in `trace/format.rs`. Both became `pub(super)`, visible to `trace`
   and its descendants, which is the narrowest visibility that compiles.
2. **`push_clause` moved with the format half**, and the prose in
   `tests/support/mod.rs` was corrected, which is the brief's second option.
   Keeping it back would have split the format functions, and
   `push_prefixed_blanks`, named in the same sentence, would have had to
   stay too.
3. **The version constants' callers were renamed rather than re-exported.**
   `dispatch/rexx_info.rs` (its import, eight paths, one intra-doc link),
   `trace.rs` (`PLATFORM`) and `environment.rs` (`LINE_END`) now name
   `version`. A `pub(crate) use` in `parse_template.rs` would have kept the
   version constants on the PARSE module's path, which is what the ruling
   takes them off. `files/tools/other_edits.py` checks each edit against a
   declared rule (`files/c7/c7-other-edits-check.txt`): HEAD's text with
   `parse_template` replaced by `version` equals the new text. For
   `rexx_info.rs` the check also sorts each run of `use` lines, because
   rustfmt moved the renamed import. `parse_template/tests.rs` gained one
   import of the constants its tests read, and `lib.rs` gained the `mod`
   declaration; both are inserted lines only.
4. **`trace.rs` keeps every `crate::trace::` path** through
   `pub(crate) use format::{..}`, since the paths are used across the
   crate. It also has a plain `use format::{..}` for the four functions only
   `trace.rs` calls. `make_displayable`, the one of those four that was
   private, became `pub(super)`.
5. **c4's declared edit.** After the de-indent,
   `the_small_int_fast_path_answers_what_the_general_path_answers`'s
   `Operator::Divide` arm fits on one line. rustfmt then drops the block
   braces around the arm body and ends the arm with a comma: a token change
   that rustfmt makes and the gate requires. It is declared to the
   instruments (`--expect-edit`). The diff, de-indented, is
   `files/c4/c4-declared-edit.txt`: two statements re-wrapped, `{`/`}`
   gone, `,` added. Literal values are identical (instrument 3 passes on
   the unit).
6. **Environment boundaries re-derived at BASE by key.** The identities
   cluster follows the survey's range: every member from
   `record_package_class` through `executable_context_package`. That range
   includes `package_find_class` and its siblings, which answer
   `Package~findClass` and still read `.local`/`.environment` through
   `directory_lookup`. It excludes `running_program`, which `.NAME`
   resolution uses. The types the moved members key on (`Annotated` and
   the rest) stayed, because `install.rs` and `lib.rs` name them as
   `environment::Annotated` and so on. Moving them would have needed
   re-exports or more path edits, for no change in what the child does.
7. **Imports.** Each new child imports the names it needs through
   `use super::{..}`, the `dispatch/` children's style. The parent keeps
   imports that only a child now uses through that path (for example
   `rexx_parse::Trace` in `trace.rs`). They count as used, so nothing was
   narrowed and instrument 1 (a) has no import-narrowing blocks this time.
8. **No test's fully qualified name changed.** Each moved test module
   keeps its name (`value::tests`, `eval::object_operand_tests`, ...).
   Instrument 4 compares names and found them identical.

## Comment and doc edits

* c7: `rexx_info.rs`'s link, departure 3.
* c10: `install.rs` and `dispatch.rs`'s three mentions, each now naming
  `environment/identities.rs` (`files/c10/c10-other-edits-check.txt`).
* c11: `tests/support/mod.rs`'s three mentions
  (`files/c11/c11-other-edits-check.txt`).
* Added: a license header in each new file; a module doc in each new
  non-test file (`version.rs`, `environment/route.rs`,
  `environment/identities.rs`, `trace/format.rs`); and a `//` comment above
  each new `mod` declaration in a parent.

`files/final/comments.txt` compares, for each parent, BASE's comment lines
with the parent plus its new files at `61478d685`. No comment line was lost
in any of them. The lines added are exactly the headers, docs and `mod`
comments just listed. The edits in the three files outside the parents are
the substitutions above, checked by rule.

## The instruments

Per-commit outputs are in `files/c<N>/`, made by `tools/checks.sh` over the
working tree before each commit. The tree the tests ran on is the tree that
was committed: `tools/commit_task5.sh` refused to commit unless the staged
`rust/` tree hash equalled the one instrument 4 ran on, and every commit's
hashes matched (`files/instrument4/i4-c<N>.meta`).

| # | 1 (a) unmoved | 1 (b) moved units | 2 tokens | 3 literals | 4 tests | load before / after instrument 4 |
| --- | --- | --- | --- | --- | --- | --- |
| c1 | 1088 equal, 1 inserted | 28 of 29, 1 reflowed | 89 of 89 | 89 of 89; 426 literals; 0 control mismatches | identical | 4.01 5.00 5.47 / 2.17 4.89 5.31 |
| c2 | 817 equal, 1 inserted | 22 of 25, 3 reflowed | 65 of 65 | 65 of 65; 372; 0 | identical | 2.75 4.32 5.06 / 4.14 5.48 5.41 |
| c3 | 964 equal, 1 inserted | 10 of 11, 1 reflowed | 58 of 58 | 58 of 58; 287; 0 | identical | 3.45 5.26 5.34 / 4.02 5.55 5.51 |
| c4 | 2205 equal, 1 inserted | 59 of 60, 1 declared | 142 of 143, 1 declared | 143 of 143; 975; 0 | identical | 2.04 4.49 5.14 / 3.14 5.47 5.65 |
| c5 | 1591 equal, 1 inserted | 14 of 16, 2 reflowed | 84 of 84 | 84 of 84; 521; 0 | identical | 3.24 5.35 5.60 / 2.97 5.14 5.52 |
| c6 | 955 equal, 1 inserted | 18 of 18 | 79 of 79 | 79 of 79; 409; 0 | identical | 2.54 4.75 5.37 / 2.27 5.36 5.56 |
| c7 | 888 equal, 1 inserted | 20 of 20 (1 with visibility) | 62 of 62 | 62 of 62; 155; 0 | identical | 2.46 4.76 5.34 / 2.39 5.13 5.41 |
| c8 | 2313 equal, 1 inserted | 5 of 6, 1 reflowed | 122 of 122 | 122 of 122; 696; 0 | identical | 1.70 4.56 5.20 / 2.21 4.99 5.32 |
| c9 | 2085 equal, 3 inserted | 9 of 9 | 117 of 117 | 117 of 117; 573; 0 | identical but the known flake, re-run in isolation: passed | 2.21 4.65 5.19 / 2.73 5.27 5.38 |
| c10 | 1364 equal, 3 inserted | 36 of 36 | 109 of 109 | 109 of 109; 502; 0 | identical | 2.28 4.40 5.05 / 4.81 5.59 5.43 |
| c11 | 860 equal, 9 inserted | 50 of 50 | 102 of 102 (3 with visibility) | 102 of 102; 600; 0 | identical | 4.06 5.13 5.27 / 2.95 5.68 5.69 |

"Reflowed" means rustfmt re-wrapped the unit after the four-space de-indent.
It is whitespace-only to instrument 1 (b), and token-identical and
literal-identical to instruments 2 and 3. Instrument 4 gave 1582 test lines
in 50 result blocks at BASE and at every commit, compared per result block
and per test (`files/instrument4/`). **At c9** one test differed:
`introspection_arity::every_unstable_row_is_really_unstable` FAILED, the
oracle-only flake the brief names. It was re-run in isolation on the same
worktree and tree hash and passed
(`files/c9/c9-i4-flake-rerun.txt` and `.log`). Every other line of c9's
comparison was identical.

Instruments 1 to 3 are Task 4's. The instruments and the tools around them
changed as follows, and each change has a control below:

* **`item-tool`**: a `static` whose value is an array of anything other
  than tuples produced **no unit at all**. The row branch matched every
  array and pushed a unit only per tuple. `environment.rs`'s
  `ORACLE_ENVIRONMENT` and `ORACLE_LOCAL` were therefore invisible to
  instruments 2 and 3 and to the cumulative check under Task 4's tool. Now
  such a static is one unit. The fix uses a let-chain, so the tool's edition
  went to 2024. Neither static moved. The item-tool output of every earlier
  task's control trees gains these units, and all controls still pass.
* **`instruments.py`**:
  * A moved test module's keys take the module's own name
    (`object_operand_tests::`) instead of a fixed `tests::`.
  * A test unit that rustfmt re-wrapped to a different line count is no
    longer re-indented line by line. The old code indexed past the end of
    the line list and crashed on c2's first run. Such a unit cannot be
    byte-identical anyway, and it falls to the whitespace-only comparison,
    with instruments 2 and 3 deciding. On re-run, c1's instrument 1 output
    differs from the committed one in one line only: the byte offset quoted
    for its one reflowed unit (1 instead of 373), with the same verdict
    (`files/final/rerun/`).
* **`cumulative.py`** and **`positional.py`**: the same module-name prefix.
  `cumulative.py`'s declared set now comes only from `SPLIT_DECLARED`, with
  no dispatch default.
* **New tools**:
  * `move_tests_mod.py`: Task 4's test mover for a module that need not
    close the file.
  * `other_edits.py`: every changed file needs a declared rule, and each
    rule must hold.
  * `span_of.py`, `set_imports.py`, `declare_env_child.py`,
    `version_edits.py`, and the per-commit drivers `tests_move.sh`,
    `move_version.sh`, `move_env_child.sh`, `move_trace_format.sh`,
    `verify.sh`, `commit_task5.sh`.
  * `docs.sh` and `tests_links.sh` now cover the parent and every file
    under its directory.

**Controls.** Task 2's five controls (its c1 pair), Task 3b's four (its c8
pair) and Task 4's eight (its c1, c3, c5 and seal pairs) ran with this
task's tooling on trees fresh from `git archive`. They ran before the first
move (`files/controls-*-before.txt`), after each tooling fix
(`-before-itemfix`, `-before-reflowfix`), and at the end
(`files/final/controls-*-final.txt`): 5 of 5, 4 of 4 and 8 of 8 every time.
This task's own controls, `tools/controls_task5.py`, give 7 of 7 at the end
(`files/final/controls-task5-final.txt`):

1. c2: a re-wrapped unit whose line count changed gains a token. Caught by
   I1b and I2.
2. c5: a moved test vanishes from `eval/object_operand_tests.rs`. Caught by
   I2, which it can only be if that file's keys carry the module's name.
3. c5: a space inside a moved test's string literal. Caught by I3.
4. c4: a test other than the declared one changes a token, so the
   declaration masks nothing else. Caught by I1b and I2.
5. c7: a moved const's literal changes. Caught by I1b, I2 and I3.
6. c9: an unmoved name in `ORACLE_LOCAL` changes. Caught by I1a, I2 and I3.
   With Task 4's `item-tool` only I1a reports it
   (`files/final/controls-task5-item-tool-before-fix.txt`).
7. c9: a moved member changes a token. Caught by I1b and I2.

My first expectation for controls 3 and 5 was wrong: I expected I1b and I2
to catch a space added inside a literal. Both are whitespace-blind by
design, and Task 4's control 1 had already recorded that; I3 is the
instrument that catches it. Control 3 now expects I3 and control 5 changes a
character instead. `tools/controls_other_edits.sh` reproduces c7's tree in a
scratch worktree. `other_edits.py` passes on the tree as committed and fails
on each of three planted defects: a renamed path that reaches another
constant, a changed line in an insert-only file, and a changed file with no
rule (`files/final/controls-other-edits-final.txt`).

**Re-run with the final tooling** on every commit pair, from `git archive`
trees (`tools/rerun.sh`, `files/final/rerun/`): every verdict is PASS, and
the outputs are identical to the committed ones except c1's byte offset
above.

**Cumulative** (`files/final/cumulative.txt`): for each parent, every BASE
unit is present exactly once at `61478d685` in the parent or its new files,
identical modulo visibility and the relaxations above, or a declared edit.
The declared edits are c4's test and c7's two renamed paths, in
`build_environment` and `debug_source_string`.

**`cargo doc --no-deps -p rexx-exec`**: 2 warning lines at every commit.
With `--document-private-items` there are 53, and with `--cfg test` as well
52, the same warning texts at every commit (the parent-and-children counts
are in each `c<N>-doc*.txt`). `tools/tests_links.sh` strips the `#[test]`
lines and documents with `cfg(test)`. It finds the same warnings in each
parent and its children at BASE (`files/base/tests-links-*.txt`) and after
each commit (`c<N>-tests-links.txt`). The only change is where they sit:
`eval.rs`'s five, which are in `object_operand_tests`, are in
`eval/object_operand_tests.rs` from c5. The intra-doc links in the moved
test modules are enumerated in `files/final/intra-doc-links.txt`, with the
command.

## Performance

callgrind `summary:` minus `libc.so.6` and `ld-linux`, two interleaved
rounds. BASE was built in its own worktree and target directory, and
`61478d685` in the instrument-4 worktree's. The `.text` sha256 is
`a0c2d9b6...` for BASE, which is the same as Task 4's c5, since the code is
identical, and `36972bba...` for `61478d685` (`files/perf/text-hashes.txt`,
`commands.txt`).

| program | BASE round 1 | BASE round 2 | final round 1 | final round 2 | change |
| --- | --- | --- | --- | --- | --- |
| rexxcps | 17897283261 | 17897264508 | 17897267547 | 17897267465 | -0.00004% |
| nop | 9340096370 | 9340098232 | 9340092544 | 9340098954 | -0.00002% |
| assign | 19540359181 | 19540362789 | 19540361767 | 19540358788 | -0.00000% |
| emptyloop | 9285883657 | 9285877531 | 9285874142 | 9285883676 | -0.00002% |
| varlookup | 14842896116 | 14842896643 | 14842899674 | 14842889473 | -0.00001% |
| arith | 11519338483 | 11519335858 | 11519335389 | 11519337431 | -0.00001% |
| compound | 9234391657 | 9234394552 | 9234395978 | 9234398150 | +0.00004% |
| dispatch | 20471082969 | 20471075370 | 20471072277 | 20471071345 | -0.00004% |
| strings | 17788063727 | 17788056683 | 17788055416 | 17788062892 | -0.00001% |

No axis moves more than 0.0001%, well under the brief's 0.5%, so nothing
was bisected. The spread between two runs of one binary is the same size.
Every program's stdout is identical between the two binaries except
`rexxcps`'s wall-clock clauses-per-second line. Load average: 1.20 3.69
4.92 before the runs, 3.93 5.11 5.27 after.

## Gates

Run after this report's commit, results in the next.

## Concerns

1. **Files still over the trigger after this task:**
   * `eval.rs` (1592): one evaluator `impl`, left whole by the survey and
     the brief.
   * `environment.rs` (1367): the seam, the model and `.NAME` resolution,
     plus two groups the ruling did not name. One is the native-collection
     helpers (`native_instance` through `set_native_entry`); the other is
     the package string tables. Either could be a later cut.
   * `value.rs` (1089).
   * `eval/tests.rs` (1138) and `plan/tests.rs` (1013): each one test
     module, moved whole.
2. **`item-tool` was blind to a `static` array of non-tuples** in every
   earlier task (tooling change 1). Task 4's `lib.rs` split and Task 2's and
   3b's splits may have moved such a static. If one did, instrument 1 (a)
   or (b) still compared its lines with `cmp`, but instruments 2 and 3 and
   the cumulative check never saw it. I did not re-audit those tasks.
3. **The known flake hit once** (c9), and passed when re-run in isolation.
4. **The parents keep imports that only a child uses** through `super::`
   (departure 7). It compiles cleanly and matches `dispatch/`, but it means
   a parent's import list no longer shows what the parent itself uses.
