# Phase 9d Task 4 report: `json`, `yaml` and `dateparser`

Status: DONE.

## Commits

* `b6f6cd64c` Phase 9d Task 4: a pass boundary keeps the loop enclosing its own (fix, witnesses, rows).
* The records and this report follow in the next commit.

## Root cause

`Interp::flat_loop_step_top` (`rust/crates/rexx-exec/src/run/loops.rs`) takes the stepping loop out
of `Activity::flat_top` while it evaluates the pass's `WHILE` or `UNTIL` test. A test that runs a
routine with a loop of its own opens that loop with `flat_top` empty, so nothing is pushed beneath
it. When the callee's loop closes, it pops the loop enclosing the stepping one into `flat_top`. The
step then overwrote that enclosing loop (`Body`), popped one loop too many (`Done`), or left it for
`unwind_frames` to release as the wrong loop (`Err`). A later pass boundary then found no loop open,
or stepped a stale loop whose op index belongs to another chunk. Now every exit of the step puts an
uncovered enclosing loop back on `flat_loops`, through the `#[cold]` `cover_enclosing_loop`. The
compiler emits nothing wrong: the refusal texts were driver state corruption, not a missing driver
arm.

## Witnesses

Both are pure Rexx with no `::requires`, listed in `rust/corpus/phase-9.txt`, each with its
`crates/rexx-parse/tests/sourceline_oracle/<name>.txt` from the module comment's `.Package~new`
driver, run on a scratch copy.

| program | base `671fd0489` | fix `b6f6cd64c` |
|---|---|---|
| `corpus/lang/loop_test_callee_loop.rex` | `rexx-exec: a compiled a pass boundary with no loop open op has no driver arm`, rc 120 | stdout, stderr and rc identical to the oracle |
| `corpus/lang/loop_test_callee_loop_chunk.rex` | `rexx-exec: a compiled Store op has no driver arm`, rc 120 | stdout, stderr and rc identical to the oracle |

The oracle answers rc 0 on both. `loop_test_callee_loop.rex` reaches the step's `Body`, `Done` and
`Err` exits. Its last section raises in a `WHILE` test after a callee's loop ran, and the oracle
prints `trapped 41.1 Q 1` and `trapped 44.1 1 1` per call. `loop_test_callee_loop_chunk.rex` puts the
enclosing loop in another routine's chunk, so the stale loop's body index lands on a `Store` op.

Each arm of the fix is load-bearing. Each mutant restored one arm to its base line, rebuilt, and ran
both witnesses against the oracle:

| mutant | `loop_test_callee_loop.rex` | `loop_test_callee_loop_chunk.rex` |
|---|---|---|
| A: `Body` assigns `Some(top)` | DIFF, pass-boundary text | SAME |
| B: `Done` pops unconditionally | DIFF, pass-boundary text | DIFF, `Store` text |
| C: `Err` does not cover | DIFF, pass-boundary text | SAME |

The fixed `loops.rs` was restored from a copy, compared with `cmp`, and rebuilt. A debug build (with
`LoopNext`'s `debug_assert_eq` on `flat_top`) also matches the oracle on both witnesses.

The whole suite run attributes the texts: `TEST35_DIRECTIVES` was blocked by the pass-boundary
text, and `TEST47_MULTILINEFLOWCOLLECTIONS` by the `Store` text.

## Rows

* `rust/crates/rexx-exec/tests/support/suite_rows.rs`: `PHASE_9_EXTENSION_DIRS` lists
  `extensions/dateparser/`, `extensions/json/` and `extensions/yaml/`. `phase_10_group` answers
  `None` for groups in those directories.
* `tests/suite.rs` fixture: the `extensions/json/json TEST_INVALID_EMPTY ... group` row moved from
  `PASSING` to `FAILING`. `extensions/rxmath/RxMath TEST_NAN ... group` took its place, and
  `extensions/rxmath/RxMath` joined `GROUPS`.
* `docs/superpowers/plans/phase-9-exclusions.txt`: the json, json_02, yaml, DateParser and
  DateFormatter rows were deleted. Every one was `Phase 10 Extension group`, from
  `grep -E '^extensions/(json|yaml|dateparser)/' docs/superpowers/plans/phase-9-exclusions.txt | awk -F'\t' '{print $3"\t"$4}' | sort | uniq -c`.
  `the_row_files_hold_only_checked_rows` and `gate_groups_9b::the_9b_group_file_is_what_its_derivation_gives`
  pass on the result.
* `rust/corpus/refusal-dispositions.tsv`: the `op_not_driven` GUARD row is reworded. The
  constructor has other sites, so the row stays. It now names the pass-boundary invariant, the
  `yaml.cls` reach, and the two witnesses. `refusal-sites.tsv` cites `lib.rs:877`, which did not
  move, and `refusal_sites` passes without a refresh.

## The three groups

`REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=extensions/json/json,extensions/json/json_02,extensions/yaml/yaml,extensions/dateparser/DateParser,extensions/dateparser/DateFormatter REXX_SUITE_TABLE=docs/superpowers/records/2026-10-18-phase-9d/task-4/groups/table.tsv ... cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite`, exit 0.

| group | oracle passed | crate passed | verdict |
|---|---|---|---|
| `extensions/dateparser/DateFormatter` | 31/31 | 31/31 | green |
| `extensions/dateparser/DateParser` | 112/112 | 112/112 | green |
| `extensions/json/json` | 33/33 | 33/33 | green |
| `extensions/json/json_02` | 227/227 | 227/227 | green |
| `extensions/yaml/yaml` | 50/50 | 50/50 | green |

Record: `docs/superpowers/records/2026-10-18-phase-9d/task-4/groups/`.

## The repository's `yaml.cls`

`extensions/yaml/yaml.cls` and the oracle's `build/bin/yaml.cls` differ (`cmp`: byte 18018, line
478). `docs/superpowers/records/2026-10-18-phase-9d/task-4/yaml-repo-cls/yamlrun.sh` lays out the
crate-side run the way `crate_run_once` does. It copies the framework, the group directory, the
driver files with `ooTest.frm`'s `rxfuncquery` tests removed, `rxregexp.cls`, and the oracle's
`*.cls` into `bin-rexx`. It then places the given `yaml.cls` over the oracle's copy and runs
`testOORexx.rex -f yaml.testGroup -U -V 2` under `memcap 2G`. The fixed binary gives, exit 0:

| `yaml.cls` | tests ran | assertions | failures | errors |
|---|---|---|---|---|
| `extensions/yaml/yaml.cls` | 50 | 759 | 0 | 0 |
| `build/bin/yaml.cls` | 50 | 759 | 0 | 0 |

The two stdouts are in the same directory. Both runs print the same `xsltproc` `RC(127)` trace on
stderr, because this machine has no `xsltproc`.

## Perf

`bench-programs/callgrind.sh -B 0.5 -p "startup interpret parse churn sayloop" -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t4 pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run head=/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t4/rexx-head`
from `rust/`, exit 0. `rexx-head` is the release `rexx` built at `b6f6cd64c`. Table copy:
`docs/superpowers/records/2026-10-18-phase-9d/task-4/callgrind.txt`. Baseline: the first block of
`docs/superpowers/records/2026-10-15-phase-9c/close/callgrind.txt`. No 9d task before this one
recorded a perf check in `progress.md`.

| program | head d% vs pin | 9c close d% vs pin | own delta |
|---|---|---|---|
| startup | +0.0188 | +0.0181 | +0.0007 |
| interpret | -2.1922 | -2.1858 | -0.0064 |
| parse | -0.0876 | -0.1117 | +0.0241 |
| churn | +0.1526 | +0.1653 | -0.0127 |
| sayloop | +0.0336 | +0.1446 | -0.1110 |

Each own delta is within +0.05%, and each figure against the pin is under +0.5%.

## Whole suite against the 9c close

`REXX_SUITE_GATE=1 REXX_SUITE_TABLE=docs/superpowers/records/2026-10-18-phase-9d/task-4/suite/table.tsv ... cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite`,
exit 101 (expected while the phase runs), 259.58 s. The joined
`docs/superpowers/records/2026-10-15-phase-9c/close/suite/tests.tsv` and this run's `tests.tsv` both
have 24523 rows. The only rows that changed:

* `extensions/yaml/yaml TEST35_DIRECTIVES`: row `Phase10(Extension)` -> `-`, crate `blocked: rexx-exec: a compiled a pass boundary with no loop open op has no driver arm` -> `passed`.
* `extensions/yaml/yaml TEST47_MULTILINEFLOWCOLLECTIONS`: row `Phase10(Extension)` -> `-`, crate `blocked: rexx-exec: a compiled Store op has no driver arm` -> `passed`.
* The remaining tests of the five groups: row `Phase10(Extension)` -> `-`, oracle and crate
  `passed` before and after (DateFormatter, DateParser, json, json_02, and the other yaml tests).

No other test's row, oracle result or crate result changed.

## Per-task check

All commands ran from `rust/` with
`CARGO_TARGET_DIR=<scratchpad>/target-9d-t4 CARGO_INCREMENTAL=0 memcap 8G` and `-j 4`.

| command | exit |
|---|---|
| `cargo fmt --all --check` | 0 |
| `cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` | 0 |
| `cargo test -j 4 --release -p rexx-exec --test suite` (ungated: 52 passed, among them `the_row_files_hold_only_checked_rows`, `row_checks::*`, `gate_groups_9b::*`) | 0 |
| `cargo test -j 4 --release -p rexx-exec --test refusal_sites --test refusal_dispositions --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals` | 0 |
| `REXX_CORPUS_GATE=1 cargo test -j 4 --release -p rexx-exec --test corpus` (29 passed, 1 ignored) | 0 |
| `cargo test -j 4 --release -p rexx-parse --test sourceline_oracle` | 0 |
| the three groups through the harness (above) | 0 |
| the whole suite through the harness (above) | 101 |
| `bench-programs/callgrind.sh ...` (above) | 0 |

## Concerns

* `loop_test_callee_loop_chunk.rex` reaches the `Store` text only through its op layout, because the
  stale loop's body index happens to land on a `Store`. After the fix it is a plain differential
  witness. Mutant B shows it still catches the `Done` arm.
