# Task 11 report: close

BASE `7485ff6d5`. Nothing under `rust/` changes in this task. Artifacts are
under `docs/superpowers/records/2026-09-15-file-split/task-11-files/`
(`files/` below). Section 1 and the gates in section 2 were done by the first
Task 11 agent; the G6 rerun is team-lead's; section 3 and this text are the
second agent's, which re-derived the section 1 lists and opened a sample of
its citations (listed at the end of section 1).

## 1. The re-measure

The plan's command, run at BASE from a `git archive` of `7485ff6d5`:

```
cd rust && find crates -name '*.rs' | xargs wc -l | awk '$1 > 1000' | sort -n
```

Output in `files/remeasure/list-7485ff6d5.txt`; the same command at
`ec7824e4a` (Task 1's BASE), `44bbf0959` and `5d84dd8cb` beside it. The
second agent re-ran it at `7485ff6d5`, `ec7824e4a` and `5d84dd8cb` from fresh
archives and got byte-identical lists, and checked the table below against
the `7485ff6d5` list by path and line count with `diff` (no difference).

"Ruled by" names where the file's reason was given. Paths are abbreviated:
`survey` is `.superpowers/sdd/2026-09-15-file-split/task-1-survey.md` (also
committed at `docs/superpowers/records/2026-09-15-file-split/task-1-survey.md`),
`ledger` is `.superpowers/sdd/2026-09-15-file-split/progress.md`, `T<N>` is
`docs/superpowers/records/2026-09-15-file-split/task-<N>-report.md`, and
`T3a` is `.superpowers/sdd/2026-09-15-file-split/task-3a-report.md` (not
under `docs/superpowers/records/`). Crate paths drop `crates/`; `rexx-exec`
is the crate unless another is named.

| file | lines | ruled by | reason it stays |
| --- | --- | --- | --- |
| `rexx-parse/tests/scanner.rs` | 1007 | `.superpowers/sdd/2026-09-15-file-split/task-9-brief.md` "Out"; ledger, Task 9 dispatched; T9 concern 2 | an integration test file; splitting renames its tests |
| `src/builtin/convert.rs` | 1011 | T7 concern 1 | one family of builtins (tests moved out) |
| `src/plan/tests.rs` | 1013 | T5 concern 1 | one test module, moved whole |
| `src/dispatch/object_protocol.rs` | 1023 | T2 concern 4; ledger, Task 6 brief rulings ("out") | one class's own methods |
| `tests/gate_table_c.rs` | 1038 | T10 concern 2 | the test target's tests and what only they read; the one further cut is inside one function's body |
| `rexx-api/src/ffi.rs` | 1041 | plan, Global Constraints (D-U1); survey, "Files where a split would make things worse"; ledger ruling 1 | no safe region to move: both the test stubs and `mod tests` hold `unsafe` in the forms `unsafe_sites.rs` matches |
| `rexx-parse/src/expr.rs` | 1043 | survey, "worse"; Task 9 brief | one recursive-descent expression parser and its precedence table |
| `src/run/tests/conditions.rs` | 1047 | T3a, fix round sizes section; ledger, Task 3a review ruling | one responsibility: when a condition is trapped and delivered |
| `rexx-num/tests/format.rs` | 1051 | survey, "worse"; plan Task 10; ledger ruling 3 | the cases for one function |
| `src/dispatch/package.rs` | 1063 | T6, "Where things went" | `Package`'s readers and writes, one class |
| `src/dispatch/class_protocol.rs` | 1065 | T2 concern 4; ledger, Task 6 brief rulings | one class's own methods |
| `rexx-parse/src/scanner.rs` | 1073 | survey, "worse"; Task 9 brief | one `Scanner` state machine and its literal packers |
| `src/builtin/convert/tests.rs` | 1087 | T7 concern 1 | one set of test cases for one family |
| `src/value.rs` | 1089 | **no ruling on the remainder** (finding 2) | my reading: the D15 value model, one `impl Interp` of value conversions plus the small-integer and parsed-number helpers they call |
| `rexx-parse/src/directive.rs` | 1117 | survey, "worse"; Task 9 brief | one `impl Dir` parser and its keyword index constants |
| `src/eval/tests.rs` | 1138 | T5 concern 1 | one test module, moved whole |
| `src/run/call.rs` | 1145 | T3b concern 2 | one construct |
| `src/builtin/datetime.rs` | 1165 | T7 concern 1 | one family of builtins |
| `src/builtin/string.rs` | 1174 | T7 concern 1 | one family of builtins |
| `src/run/tests/scope.rs` | 1190 | T3a, fix round sizes section; ledger, Task 3a review ruling | variable scoping, one contiguous range |
| `tests/ir_recorded.rs` | 1211 | survey, "worse" and "What the plan's later tasks assume" item 6; plan Task 10 | one set of test cases; its inline tables' move to a data file is queued outside this plan |
| `tests/method_bodies.rs` | 1213 | survey, "worse"; plan Task 10 | one table (D76) and the probe that fills it |
| `src/activation.rs` | 1274 | survey, "What the plan's later tasks assume" item 8; ledger ruling 3; plan Task 5 | one thing with a 56-line test module the survey said to skip |
| `rexx-parse/src/ast.rs` | 1352 | survey, Rank 12; Task 9 brief | one declaration catalogue whose groups reference each other |
| `src/environment.rs` | 1367 | ledger, Task 5 dispatched (SPLIT, seam stays); **T5 concern 1 says it is not one thing** (finding 3) | the seam, the model and `.NAME` resolution, plus native-collection helpers and package string tables that the ruling did not name |
| `rexx-api/src/values.rs` | 1368 | ledger, Task 8 dispatched; T8 concern 1 | one table-driven module; `TABLE` stays beside its builders |
| `rexx-parse/src/directive/tests.rs` | 1505 | Task 9 brief "Out" | one grammar's cases; a topic split renames every test |
| `tests/coverage.rs` | 1509 | survey, "worse"; plan Task 10 | committed data beside the assertions that compare against it |
| `src/eval.rs` | 1592 | survey, Rank 7; T5 concern 1 | one evaluator `impl Interp` |
| `src/ir/golden_tests.rs` | 1637 | survey, "worse"; `.superpowers/sdd/2026-09-15-file-split/task-7-brief.md` ("out (one subject)") | golden transcripts for one subject, `ir::compile` |
| `src/ir/compile.rs` | 1722 | ledger, Task 7 dispatched; T7 concern 1 | the one compile pass and its emit helpers (invariants and tests moved out) |
| `rexx-parse/src/instruction.rs` | 1733 | ledger, Task 9 dispatched; T9 concern 2 | the dispatch, the cursor primitives and the other instructions' grammars, all one `impl Inst` |
| `src/dispatch/tests.rs` | 1801 | T2 concern 4; ledger, Task 6 brief rulings | one test module; splitting renames its tests |
| `rexx-classes/tests/native_classes_wiring.rs` | 1865 | survey, "worse"; plan Task 10 | recorded method sets asserted in the tests that hold them |
| `src/dispatch/buffer.rs` | 1877 | ledger, Task 6 brief rulings; T6, "Where things went" | `MutableBuffer`'s bodies and the per-method argument sets it shares with `String`, after the shared parsers left |
| `src/dispatch/stream.rs` | 1925 | T6, "Where things went" | `.Stream`'s entry points around one state block whose helpers every group calls |
| `rexx-parse/src/instruction/tests.rs` | 2084 | Task 9 brief "Out" | one grammar's cases; a named fixture reader |
| `rexx-api/tests/values.rs` | 2114 | ledger, Task 8 dispatched | one subject; an integration-test split renames every test |
| `src/dispatch/hash.rs` | 2133 | survey, Rank 5; T6, "Where things went" | the store, `Directory`/`StringTable`, the shared hashed surface and its constructors, as ruled |
| `src/run/loops.rs` | 2322 | T3b concern 2 | one construct; the nested and flat paths side by side could be a later cut |
| `src/install.rs` | 2333 | survey, Rank 4 (one module); **T4 concern 3 says it holds more than one thing** (finding 3) | directive installation, library resolution, the executable records and the access-scope rows |
| `src/dispatch/string.rs` | 2364 | T6, "Where things went" | one class's primitive methods and their table, operators included |
| `src/ir/drive.rs` | 2416 | survey, "worse"; ledger, Task 7 dispatched; T7 concern 1 | the op loop, one `match` over `Op` |
| `src/error.rs` | 2452 | survey, "worse" and item 8; ledger ruling 3 | `impl Raised` is an alphabet of condition constructors; the `Failure` block is the one cut the survey allowed, not ruled in |
| `src/dispatch.rs` | 2882 | T2 concern 2; T6 concern 2 | the object model and the native tables (rows stay by Task 2's departure), plus `VariableReference`'s natives, which no child serves |
| `src/lib.rs` | 3210 | ledger, "Task 4 ruling: `Loud` stays"; **T4 concern 2 says it holds more than one responsibility** (finding 3) | `Loud` stays until the refusal scanner keys on the `impl`, not the file |
| `src/run.rs` | 3438 | **T3b concern 1 says it is not one thing** (finding 3) | the loop plus instruction bodies and stream resolution the survey gave no home |

### Against Task 1's list

| commit | files over the trigger |
| --- | --- |
| `5d84dd8cb` | 46 |
| `ec7824e4a` (Task 1's BASE) | 48 |
| `44bbf0959` | 48 |
| `7485ff6d5` (this BASE) | 47 |

The brief says Task 1's list was taken at `5d84dd8cb` with 48 files. The
plan's command at `5d84dd8cb` lists fewer; the files it lacks against
`ec7824e4a` are `rexx-api/src/ffi.rs` (946 lines at `5d84dd8cb`, 1041 at
`ec7824e4a`) and `rexx-api/src/invoke.rs` (981, then 1144), which crossed
the trigger in between (finding 1). The comparison below is against
`ec7824e4a`, the list Task 1 surveyed. `files/remeasure/left.txt`,
`joined.txt` and `stayed.txt` were made with `comm` over the two sorted path
lists.

* **Left**: `rexx-api/src/invoke.rs`, `rexx-bench/src/bin/rexx-bench-suite.rs`,
  `builtin/numeric.rs`, `builtin.rs`, `dispatch/collection.rs`,
  `dispatch/library.rs`, `dispatch/native.rs`, `parse_template.rs`,
  `plan.rs`, `run/tests.rs`, `trace.rs`, `rexx-extract/src/docs/classes.rs`,
  `rexx-parse/src/block.rs`.
* **Joined**, all children the splits created: `builtin/convert/tests.rs`,
  `dispatch/buffer.rs`, `dispatch/class_protocol.rs`,
  `dispatch/object_protocol.rs`, `dispatch/tests.rs`, `eval/tests.rs`,
  `install.rs`, `plan/tests.rs`, `run/call.rs`, `run/loops.rs`,
  `run/tests/conditions.rs`, `run/tests/scope.rs`.
* **Stayed**: every other file in the table above (`files/remeasure/stayed.txt`).

Citations the second agent opened and printed: T2 concern 4 (and 2, 3); T3b
concerns 1 and 2; T4 concerns 2 and 3; T5 concern 1; T6's "Where things went"
bullets for `stream.rs` and `package.rs` (lines 96 and 100); T7 concern 1;
T9 concern 2; T10 concern 2; survey "Rank 12" (line 515, `ast.rs` at 548) and
the survey's enumeration paragraph (lines 16 to 27). Each says what the table
attributes to it. The others are the first agent's and were not reopened.

## 2. The gates

Over `91f6afaba` (the report skeleton; its `rust/` tree is `7485ff6d5`'s), by
`files/tools/gates.sh`, statuses in `files/gates/status.txt`, read unpiped.
`git status --short` was empty before and after (`tree-changes 0`,
`tree-changes-after 0`). Default target directory for G3 to G6.

| gate | command | exit | load (1, 5, 15 min) |
| --- | --- | --- | --- |
| G1 | `cargo fmt --all --check` | 0 | before: 6.00 8.16 12.70 |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings`, empty target dir | 0 | |
| G3 | `cargo build --workspace --all-targets --release` | 0 | |
| G4 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release --no-fail-fast` | 0 | before 5.04 7.84 12.52; after 4.72 6.39 10.67 |
| G5 | `cargo build --workspace --all-targets` | 0 | |
| G6 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **101** | before 4.72 6.39 10.67; after 3.90 6.72 9.91 |
| G6 rerun (team-lead, same commit) | same command | 0 | before 0.17 0.16 0.21; after 9.72 5.85 2.68 (`files/gates/status-rerun.txt`) |

G3's log is one `Finished` line (0.51 s): `rust/target` was already current
for this tree, so G4's arity suites ran a binary built from these sources.

Test counts from `files/tools/test_results.py` (`files/gates/*.results`), and
`test result:` lines counted with `/bin/grep -a -c` in each log:

| run | ok | FAILED | ignored | result blocks | blocks ok | corpus |
| --- | --- | --- | --- | --- | --- | --- |
| G4 release | 2663 | 0 | 4 | 135 | 135 | 604 of 604 |
| G6 debug, first | 2663 | 1 | 4 | 135 | 134 | 603 of 604 |
| G6 debug, rerun | 2664 | 0 | 4 | 135 | 135 | 604 of 604 |

The release and rerun debug counts equal the plan's BASE figures (2663/0/4
release, 2664/0/4 debug, 604 of 604 STRICT), so the failing set is empty.

**The first G6 failure was the oracle's, not ours.** The one failing test
was `corpus_differential` (`tests/corpus.rs`), with one mismatch,
`lang/push_queue.rex` (`files/gates/test-debug.txt:1760`). Our side's trace
is the one the release run matched. The oracle's side stopped at line 33,
`push a`, with `Compiled method "PUSH" with scope "RexxQueue"` and Error 48
(failure in system service), exit 208. team-lead ran the oracle's PUSH
afterwards and it worked: a transient in the oracle's rxapi queue service,
which is machine state, so the whole gate was re-run on the same commit when
the machine was quiet, as the brief directs. `diff` of the two debug runs'
per-test results (`files/gates/test-debug.results`,
`files/gates/test-debug-rerun.results`) differs on exactly these lines: the
`tests/corpus.rs [corpus]` block's `RESULT` line (FAILED passed=28 failed=1
against ok passed=29 failed=0) and `corpus_differential` (FAILED against ok).

## 3. Cumulative performance, built fresh

`d6dd60d90` (Task 2's BASE) against `7485ff6d5`, each built by
`files/tools/build.sh`: a `git archive` of the whole repository at the
commit, every extracted file touched, a fresh `CARGO_TARGET_DIR` of its own,
`cargo build --release -p rexx-exec --bin rexx-run`. Every build log
(`files/perf/build-logs/`) has exactly one `Compiling rexx-exec v0.1.0` line
naming its own archive, and `files/perf/builds.txt` records it with the
exit status, load and `.text` hash. A first attempt archived only `rust/`
and failed in `rexx-lib`'s build script, which reads
`interpreter/RexxClasses/CoreClasses.orx` from the repository root; its logs
are in `files/perf/failed-attempt1/` and it measured nothing.

callgrind (`--cache-sim=no --branch-sim=no`), `summary:` minus `libc.so.6`
and `ld-linux` by `files/tools/cg_objects.py`, which asserts the per-object
costs add up to `summary:`. `files/tools/perf.sh` ran two interleaved rounds
of both binaries on the brief's programs, each run in a fresh empty
directory that was still empty afterwards (every `.result` says `leftover
files in cwd: 0`). The programs were read from `7485ff6d5`'s archive;
`git diff d6dd60d90 7485ff6d5 -- rust/bench-rexxcps rust/bench-programs` is
empty. Every run exited 0.

| program | `d6dd60d90` r1 | `d6dd60d90` r2 | `7485ff6d5` r1 | `7485ff6d5` r2 | r1 | r2 | change (means) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| rexxcps | 17897276821 | 17897287851 | 17895021796 | 17895036456 | -0.01260% | -0.01258% | -0.01259% |
| nop | 9340094695 | 9340098457 | 9340101302 | 9340105654 | +0.00007% | +0.00008% | +0.00007% |
| assign | 19540360342 | 19540361928 | 19540362723 | 19540363644 | +0.00001% | +0.00001% | +0.00001% |
| emptyloop | 9285889130 | 9285882173 | 9285882413 | 9285883596 | -0.00007% | +0.00002% | -0.00003% |
| varlookup | 14842896471 | 14842899742 | 14842899605 | 14842901535 | +0.00002% | +0.00001% | +0.00002% |
| arith | 11519342277 | 11519337321 | 11519344937 | 11519335804 | +0.00002% | -0.00001% | +0.00000% |
| compound | 9234398051 | 9234393159 | 9234395510 | 9234397025 | -0.00003% | +0.00004% | +0.00001% |
| dispatch | 20471081669 | 20471079920 | 20461083822 | 20461078720 | -0.04884% | -0.04886% | -0.04885% |
| strings | 17788061334 | 17788060541 | 17737063413 | 17737068241 | -0.28670% | -0.28667% | -0.28668% |
| startup | 60856349 | 60861846 | 60859727 | 60858247 | +0.00555% | -0.00591% | -0.00018% |
| parse | 1539184444 | 1539180649 | 1543380824 | 1543384849 | +0.27264% | +0.27315% | +0.27289% |

(`files/perf/cumulative/perf-summary.txt`, `perf-table.tsv`, the `.result`
files and `stdout/`.) Every program's stdout is identical between the two
binaries except `rexxcps`'s wall-clock `Performance:` line.

No axis moves more than 0.5%. The axes that move by more than their
round-to-round spread are `strings`, `dispatch` and `rexxcps` (down) and
`parse` (up).

`.text` sha256 (`objcopy -O binary --only-section=.text BIN OUT && sha256sum OUT`):

| revision | `.text` sha256 | bytes |
| --- | --- | --- |
| `d6dd60d90` | `597b63b4f49206ff91bc1b1987298e154a9f32deb282b3d9fc5bf2c4eb0c685d` | 2527323 |
| `7485ff6d5` | `e86350ff9724ed1470f233e726c6000205ded3bb224a4e595e663f12eb3a45e5` | 2522699 |

### Per task, built fresh at every task BASE

No axis crossed the threshold, so the brief did not require a bisect. The
movements above were attributed anyway, because the recorded per-task
figures below do not account for them: every task BASE in the ledger was
built the same way and measured once per program by
`files/tools/chain.sh` (one round; the endpoints are round 1 of the table
above). The step between two consecutive BASEs holds exactly one task's
`rust/` commits (`git log --oneline A..B -- rust` for each pair).

| program | T2 | T3b | T4 | T5 | T6 | T7 | T8 | T9 | T10 | cumulative |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| rexxcps | -0.00008% | +0.00000% | -0.00000% | +0.00008% | -0.01259% | -0.00001% | +0.00001% | -0.00000% | +0.00000% | -0.01260% |
| nop | +0.00002% | -0.00004% | +0.00001% | +0.00006% | -0.00003% | +0.00001% | +0.00001% | +0.00004% | -0.00001% | +0.00007% |
| assign | -0.00002% | +0.00003% | +0.00001% | -0.00003% | +0.00003% | +0.00002% | -0.00001% | +0.00000% | -0.00002% | +0.00001% |
| emptyloop | -0.00008% | -0.00002% | +0.00006% | -0.00002% | +0.00001% | -0.00002% | +0.00005% | +0.00003% | -0.00007% | -0.00007% |
| varlookup | +0.00001% | +0.00002% | -0.00000% | +0.00006% | -0.00007% | +0.00002% | -0.00004% | +0.00005% | -0.00002% | +0.00002% |
| arith | +0.00002% | -0.00008% | -0.00000% | +0.00004% | +0.00001% | -0.00001% | +0.00004% | -0.00000% | +0.00002% | +0.00002% |
| compound | -0.00000% | -0.00002% | -0.00000% | -0.00002% | +0.00005% | -0.00004% | +0.00005% | +0.00009% | -0.00013% | -0.00003% |
| dispatch | -0.00005% | +0.00003% | +0.00005% | -0.00007% | -0.04884% | +0.00002% | +0.00003% | -0.00002% | +0.00001% | -0.04884% |
| strings | +0.00002% | +0.00002% | -0.00005% | +0.00003% | -0.28671% | -0.00002% | -0.00001% | +0.00004% | -0.00002% | -0.28670% |
| startup | +0.00487% | -0.00201% | +0.00256% | -0.00601% | +0.00741% | -0.00273% | +0.00867% | -0.00594% | -0.00127% | +0.00555% |
| parse | -0.00053% | -0.00001% | +0.00008% | +0.00017% | +0.27297% | -0.00006% | +0.00040% | -0.00052% | +0.00014% | +0.27264% |

Absolute figures in `files/perf/chain-table.txt`, per-run results in
`files/perf/chain/`. Every non-`rexxcps` program's stdout was byte-identical
across every binary and round; `rexxcps`'s was identical once its
`Performance:` line was removed.

| BASE | step | `.text` sha256 (first 8) | bytes | same hash recorded in |
| --- | --- | --- | --- | --- |
| `d6dd60d90` | | `597b63b4` | 2527323 | T2 |
| `36bbb3684` | T2 | `f6840a66` | 2527131 | T2, T3b |
| `2f3065b2b` | T3b | `da915d7c` | 2527131 | T3b, T4 |
| `1b851ae96` | T4 | `a0c2d9b6` | 2527131 | T4, T5 |
| `ed8cb3f03` | T5 | `36972bba` | 2527131 | T5, T6 |
| `964a6a8db` | T6 | `c8244939` | 2522699 | T6, T7 |
| `5c7173fac` | T7 | `bc09a49f` | 2522699 | T7, T8, T9 |
| `72f2bc3d8` | T8 | `bc09a49f` | 2522699 | T8, T9 |
| `8a1c43696` | T9 | `e86350ff` | 2522699 | T9, T10 |
| `7485ff6d5` | T10 | `e86350ff` | 2522699 | T10 |

Every fresh hash is one a task report already recorded (found with
`/bin/grep -a -rl` over the records directory). So each recorded hash chain
was a real build at least at its ends, including Task 7's and Task 8's
finals, whose builds Task 9 put in doubt. Task 8 did not change `.text`,
which it claimed on a stale binary and which is now measured; Task 10 did
not change it either. Task 7 changed `.text` without changing its size or
any axis.

### The per-task figures as recorded

`files/perf/per-task-recorded.tsv`, each task report's "change" column; `-`
is an axis the task did not measure.

| task | status | rexxcps | nop | assign | emptyloop | varlookup | arith | compound | dispatch | strings | startup | parse |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 10a | skipped | - | - | - | - | - | - | - | - | - | - | - |
| 2 | valid | -0.00006 | - | - | - | -0.00004 | -0.00007 | -0.00003 | +0.00000 | -0.00002 | - | - |
| 3b | valid | -0.00001 | -0.00002 | -0.00001 | -0.00000 | +0.00001 | -0.00004 | -0.00003 | +0.00002 | -0.00004 | - | - |
| 4 | valid | +0.00002 | +0.00002 | -0.00004 | +0.00008 | -0.00001 | +0.00003 | -0.00006 | -0.00000 | -0.00002 | - | - |
| 5 | valid | -0.00004 | -0.00002 | -0.00000 | -0.00002 | -0.00001 | -0.00001 | +0.00004 | -0.00004 | -0.00001 | - | - |
| 6 | valid | +0.00067 | +0.00008 | +0.00005 | -0.00003 | +0.00000 | -0.00001 | +0.00000 | -0.04881 | -0.28670 | - | - |
| 7 | **void** | -0.00004 | +0.00002 | +0.00000 | +0.00004 | +0.00001 | +0.00001 | +0.00003 | +0.00000 | -0.00000 | - | - |
| 8 | **void** | +0.00000 | +0.00009 | - | - | - | - | - | +0.00001 | - | - | - |
| 9 | valid | +0.00003 | -0.00001 | - | - | - | - | - | - | - | +0.00917 | -0.00006 |
| 10 | text-equal | - | - | - | - | - | - | - | - | - | - | - |
| **sum of valid** | | +0.00061 | +0.00005 | -0.00000 | +0.00003 | -0.00005 | -0.00010 | -0.00008 | -0.04883 | -0.28679 | +0.00917 | -0.00006 |
| **measured cumulative** | | -0.01259 | +0.00007 | +0.00001 | -0.00003 | +0.00002 | +0.00000 | +0.00001 | -0.04885 | -0.28668 | -0.00018 | +0.27289 |

(Sums by `files/tools/sum_recorded.py`.) The recorded sums agree with the
fresh cumulative figure on every axis but `rexxcps` and `parse`, and the
per-BASE table says why for both (findings 4 and 5). `startup`'s recorded
+0.00917% is the size of that axis's round-to-round noise: `d6dd60d90`'s
own rounds differ by 0.00903%.

## 4. Nothing under rust/ changed

This task's writes are the report and `task-11-files/`. The builds and runs
read `git archive` trees in the scratchpad and wrote only there. The shared
checkout had uncommitted `rust/` edits by another agent while section 3 ran,
so the "nothing under `rust/`" check belongs to the commit: it must stage
these explicit paths and `git diff --cached --stat` must show nothing under
`rust/`.

## Findings

1. **The survey's claim that no file crossed the trigger between
   `5d84dd8cb` and its BASE is false.** Survey line 27 says so; the plan's
   command at `5d84dd8cb` lacks `rexx-api/src/ffi.rs` (946 lines) and
   `rexx-api/src/invoke.rs` (981), both over the trigger at `ec7824e4a`
   (1041 and 1144). The brief's "48 files at `5d84dd8cb`" carries the same
   error. Task 1's rulings covered both files, so no ruling is missing.
2. **`value.rs` (1089) has no ruling on its remainder.** T5 concern 1 lists
   it with no reason. Reading: the D15 value model, one `impl Interp` of
   value conversions plus the small-integer and parsed-number helpers they
   call. Not split.
3. **Files over the trigger that their own task's report says are not one
   thing**: `run.rs` (T3b concern 1), `lib.rs` (T4 concern 2, with `Loud`
   held by ruling), `install.rs` (T4 concern 3) and `environment.rs` (T5
   concern 1). Each report names the separable part. Not split here.
4. **Task 6's recorded `rexxcps` figure (+0.00067%) is wrong; the move cost
   nothing there and saved 0.0126%.** Task 6's final round 1 was
   17899771416, 4.76 million over its round 2 (17895014232), and the mean of
   the two gave the recorded figure. Fresh, both rounds at each endpoint
   agree within 15 thousand, and the whole -0.0126% falls in Task 6's step.
   What made Task 6's round 1 high is not known.
5. **`parse` rose 0.273% in Task 6's step, and no task recorded it.**
   `parse` was added to the axes at Task 9, after Task 6. Under the 0.5%
   threshold, so the step was not bisected commit by commit; its
   candidates are Task 6's moves (`git log --oneline ed8cb3f03..964a6a8db
   -- rust`), among them `dispatch/method_arguments.rs` and the `String`
   residue natives. `strings` (-0.287%) and `dispatch` (-0.049%) also fall
   entirely in Task 6's step, as Task 6 recorded.
6. The first G6 failed on an oracle-side transient (section 2); the rerun
   on the same commit passed with the plan's BASE counts.

## Concerns

1. The per-BASE table is one round per revision. Its endpoints' two rounds
   differ by at most 0.00008% on every axis except `parse` (0.00026%) and
   `startup` (0.00903%), so a single-round step of that size is noise; the
   findings above sit well outside it.
2. Section 4's check was not run by this agent (see there).
3. The `rexxcps` outlier in finding 4 is unexplained. If `rexxcps` can
   produce a round 4.76 million high, a two-round mean can hide or invent a
   change of that size (0.027%) on this axis.
