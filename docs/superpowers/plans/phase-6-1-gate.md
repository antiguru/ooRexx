# Phase 6.1 gate record

Performance base: `e6af1198b` (the commit Task 1 starts from). Budget (spec section 5): a running
total of at most +0.5% beyond the noise band on every program; wall clock within ±4%.

## Task 1

Commits: `66b0f6854` (layout), `16b67c6cc` (behaviour), `12239c974` (perf round 1). `S` is
`/tmp/claude-1000/p61/t1`; each binary from `git archive <sha> rust interpreter`, built with
`CARGO_TARGET_DIR=$S/target-NAME memcap 8G cargo build --release -j 4 -p rexx-exec --bin rexx-run`,
one `Compiling rexx-exec` line each.

| binary | source | sha256 |
|---|---|---|
| base | `e6af1198b` | `2ea19b3ea2875fada8718eaf5b89f828d5ae07587a74feaf9b34681c879c891e` |
| pad1 | base + `layout-pad.py 8` | `f7c3a9d41e0646fc1931281a32aaf002b7a3f762a87386349508c70fe61a5aa1` |
| pad2 | base + `layout-pad.py 48` | `c03d30015583f749cf4344438901f65af16dc34746a67ff6d9c7d9e2549cfac0` |
| pad3 | base + `layout-pad.py 192` | `91653253d9f2c1c7ab8ca690b17460dca4ad4615a3a284401ebec17195f6e6bb` |
| t1 | `16b67c6cc` | `6a31c6fb78068741f10d9ded837051f4c2334cbba04b4b2cb4285f835990a78b` |
| r1 | `12239c974` | `b44aaf39eeacc09b6fe1bcacfb8235d4fb0d3db02ac51e5c000e855c18d8ec3c` |

Noise band, the layout controls against base:

```
bash rust/bench-programs/callgrind.sh -r 3 -j 8 -o $S/cg-pad -p "fibcall fibfunc dispatch dispatchclass sendloop" base=$S/bin/base/rexx-run pad1=$S/bin/pad1/rexx-run pad2=$S/bin/pad2/rexx-run pad3=$S/bin/pad3/rexx-run
```

Exit 0. Every control delta on every program prints as `0.0000`% (largest absolute difference 136
instructions, `sendloop` pad3), so the band is 0 and the budget is +0.5%.

Instructions:

```
bash rust/bench-programs/callgrind.sh -r 3 -j 10 -o $S/cg-final -p "fibcall fibfunc dispatch dispatchclass sendloop" base=$S/bin/base/rexx-run t1=$S/bin/head/rexx-run r1=$S/bin/r1c/rexx-run
```

Exit 0, every spread 0.0000%.

| program | t1 % | r1 % | verdict |
|---|---:|---:|---|
| fibcall | +0.6131 | +0.2044 | inside |
| fibfunc | +0.5642 | +0.2717 | inside |
| dispatch | +0.7929 | +0.0961 | inside |
| dispatchclass | +0.8357 | +0.1013 | inside |
| sendloop | +1.1774 | +0.1427 | inside |

t1 was over on every program: `pop_activation`, inlined into its callers at base, went out of line
(`cgdiff.py` on `sendloop`: `finish_send` -245,000,269, `pop_activation` +365,005,475). Round 1
moved the termination restore to a cold helper, forced `pop_activation` inline and kept
`TraceCache` at its base shape.

Wall clock:

```
PROGRAMS="fibcall fibfunc dispatch dispatchclass sendloop" bash rust/bench-programs/wallclock.sh -r 5 -o $S/wall-final base=$S/bin/base/rexx-run pad2=$S/bin/pad2/rexx-run r1=$S/bin/r1c/rexx-run
```

Exit 0; load average 2.59 at start, 1.47 at end.

| program | pad2 % | r1 % |
|---|---:|---:|
| fibcall | +0.13 | -0.13 |
| fibfunc | +0.51 | +0.13 |
| dispatch | +0.63 | -3.16 |
| dispatchclass | +1.71 | -2.99 |
| sendloop | +0.51 | -2.40 |

Every program inside ±4%.

## Task 2

Commits: `5b7acef35` (behaviour), `9777db504` (perf round 1), `fe765db7b` (witness), `4d46b76b0` (whole-group lines). `S` is
`/tmp/claude-1000/p61/t2`; base and pads are Task 1's binaries (sha256 above, re-checked). Each
Task 2 binary from `git archive <sha> rust interpreter`, built with
`CARGO_TARGET_DIR=$S/target-NAME memcap 8G cargo build --release -j 4 -p rexx-exec --bin rexx-run`,
one `Compiling rexx-exec` line each.

| binary | source | sha256 |
|---|---|---|
| t2 | `5b7acef35` | `b21c6a8e7f2e201312009c1f6be339db75f1d8ab918f9bb5b4c23bdbfd18e1a3` |
| r1 | `9777db504` | `0d08596f68c918280afca14c1d019b6ca00618ee1884b651bd8ab606b6d93d5d` |

Instructions:

```
bash rust/bench-programs/callgrind.sh -r 3 -j 8 -o $S/cg-final -p "emptyloop decloop rexxcps" base=$B/base/rexx-run pad1=$B/pad1/rexx-run pad2=$B/pad2/rexx-run pad3=$B/pad3/rexx-run t2=$S/bin/t2/rexx-run r1=$S/bin/r1/rexx-run
```

(`B` is `/tmp/claude-1000/p61/t1/bin`.) Exit 0. The layout controls print `+0.0000`% on every
program, so the band is 0 and the budget is +0.5%.

| program | t2 % | r1 % | verdict |
|---|---:|---:|---|
| emptyloop | +18.6211 | -0.3229 | inside |
| decloop | +3.1105 | +0.4373 | inside |
| rexxcps | +0.4163 | +0.2741 | inside |

t2 was over: the fast flat header's closure stopped being inlined into `ops_loop_steady`
(`cgdiff.py` on `emptyloop`: `flat_loop_header::{closure#0}` +875,001,750 as a symbol of its own,
`ops_loop_steady` +624,854,607). Round 1 spells the clause out (`enter_clause`, the advance,
`leave_clause`), keeps WHILE/UNTIL/COUNTER as bits of one `PassTest` byte, and moves the driver's
`LoopHeaderValue` work into one filing call.

Wall clock:

```
PROGRAMS="emptyloop decloop rexxcps" bash rust/bench-programs/wallclock.sh -r 5 -o $S/wall base=$B/base/rexx-run pad2=$B/pad2/rexx-run r1=$S/bin/r1/rexx-run
```

Exit 0; load average 0.16 at start, 0.57 at end.

| program | pad2 % | r1 % |
|---|---:|---:|
| emptyloop | -0.23 | +4.16 |
| decloop | +0.84 | -0.84 |
| rexxcps | -2.03 | +0.81 |

**`emptyloop` is outside ±4% on wall clock.** Two more emptyloop-only runs of the same command
gave +4.18 and +4.42. `perf stat -e cycles`, three runs each: base 1.262-1.288 G, Task 1's `r1c`
1.281-1.291 G, Task 2's r1 1.316-1.348 G; branch misses equal (370,386 against 384,551). The cost is
cycles at fewer instructions, and `perf annotate` puts it in `loop_advance` (20.3% of cycles at base,
37.4% at r1, where `cgdiff.py` gives it 49,999,953 fewer instructions). Two further rounds moved it the wrong way:
`DO WITH`'s state out of `LoopState` (+6.5% cycles) and the counter boxed in `FlatLoop` (+10%).
Neither landed. Three rounds are spent; this stops for Moritz's ruling.

Ruling (Moritz, 2026-10-07): the `emptyloop` wall-clock overrun is accepted for now and is
re-checked at Task 12 with a layout control.

Whole-group rows that `4d46b76b0` rewrote. Each failure was visible only once DO COUNTER stopped
refusing, and none of them is caused by the loops. Each test was run alone on both engines from an
ooTest copy with every other `test_*` method renamed. The oracle passes every one; the column says
what ours fails on.

| group | test | ours, alone | cause |
|---|---|---|---|
| TRACE_TraceObject | TEST_VARIABLE | rc 1: `assertSame` 6 against 0, "number of variable related trace objects" (line 162) | no collected TraceObject has a VARIABLE entry |
| TRACE_TraceObject | TEST_CALLER_STACK_FRAME_REPLY_START | rc 1: `assertEquals` at line 706, a count of 0 where 2 is expected | no CALLERSTACKFRAME carries a THREAD entry |
| TRACE_TraceObject | TEST_TRACEOBJECT_COLLECTOR | rc 2: 97.1 at line 276, `.nil` does not understand STARTSWITH | the collector gathers fewer trace lines than the test produces |
| TRACE | TEST_TRACE_LABEL_WITH_FORWARD | rc 1: `test_forwarded3.rex` traces 10 lines against 12 (line 1188) | FORWARD/REPLY `>I>`/`<I<` lines across the replied activity |
| Method | TEST_NEWFILE_CONTEXT_FLOATINGMETHOD, TEST_NEWFILE_CONTEXT_IMPORTEDPACKAGE, TEST_NEW_CONTEXT_OMITTED, TEST_NEW_ARRAY_FROM_FILE, TEST_NEW_FILE_COMPILED | rc 2: three `assertEquals`/`assertOneOrAnother` failures and two 98.971 | Method NEW's file, array and context shapes; the whole run refuses at Method NEW (Phase 9) |

## Task 3

No performance measurement is owed at this task's close (global constraints). Per-task check at
`652826b54`, `S` = `/tmp/claude-1000/p61/t3/gate`: `memcap 8G cargo test -j 4 --workspace
--no-fail-fast` exit 0; `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec --test corpus
--test ir_recorded_oracle` exit 0; `whole_groups` exit 101 on the rows below, which the
whole-group commit rewrites.

Whole-group and group-table rows the whole-group commit rewrites in `tests/concurrency_tests.rs`.
None is a regression: each is a refusal this task removed, or what running past it reaches.

| group | part | was | now | cause |
|---|---|---|---|---|
| RexxContext | whole | refuses at TESTCONDITION01, CONDITION("O") | refuses at TESTCOPY01, `RexxContext~copy` (Phase 9) | `.context~condition` answers (c3), so the run reaches the next refusal |
| RexxContext | rest | assertions 354, failing [TESTRS01] | assertions 357, failing [TESTRS01] | TESTCONDITION01's three `assertEquals` now run and pass |
| TRACE | rest | assertions 117, failing set unchanged | assertions 118 | TEST_TRACE_OPTIONS, the group's one test of a form this task changed (OPTIONS, b1): started and not failing in the dumped run, so its `assertTraceOutput` is the attributed assertion |
| GUARD | derived | refuses at TEST_WHEN_USE_LOCAL_NO_WAIT, USE LOCAL | agrees with the oracle | USE LOCAL runs (b2); the two `DIFFERING` rows are deleted |
| GUARD | outcome table | `GUARD_PASSING` without TEST_WHEN_USE_LOCAL_NO_WAIT | with it | the same; `the_outcome_table_of_the_guard_group_in_both_modes` |

At `61d441781`: `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec --release --test
concurrency_tests whole_groups` exit 0, 6 passed. The whole gated `concurrency_tests` file (same tree
content) is 37 passed, 1 failed: `the_s2_rows_of_the_derived_list_in_both_modes`, TRACE_TraceObject
TEST_TRACEOBJECT_COLLECTOR differing between the modes (rc 2 both, 2 assertions against the oracle's
583). It fails identically, three runs at HEAD and one at `1d308cb9d` (the base tree from `git
archive`, its own target directory), so it predates this task.

### Task 3 fix round 1

`the_s2_rows_of_the_derived_list_in_both_modes`, TRACE_TraceObject TEST_TRACEOBJECT_COLLECTOR,
bisected with each commit's tree from `git archive` in one target directory, a `Compiling rexx-exec`
line each: passes at `3cee3e622` (the row "refused: DO is not implemented", allowed by
`TRACE_INTERLEAVES`) and fails at `5b7acef35` (Task 2's DO COUNTER). Not a regression: the removed
DO refusal lets the test run, and both modes end in the same 97.1 at rc 2 with the same stdout (its
timestamps masked) while the REPLY continuation's trace lines reach stderr in another order. The
row's allowance in `concurrency_tests.rs` now also covers that case (same status, same masked stdout,
the same stderr lines in another order).

Performance, measured because the fix round changes the DO paths. Binaries: base, pads and Task 2's
r1 as recorded above (sha256 re-checked); `t3` from `git archive 012bf8ab1 rust interpreter`, built with
`CARGO_TARGET_DIR=/tmp/claude-1000/p61/t3/target-f memcap 8G cargo build --release -j 4 -p rexx-exec
--bin rexx-run`, one `Compiling rexx-exec` line, sha256
`64138259f4635338f4f942ba37cbec21ae5f5e086e573b5db7fcdb1edc01dbb9`.

```
bash rust/bench-programs/callgrind.sh -r 3 -j 8 -o $S/cg-final2 -p "emptyloop decloop rexxcps" base=$B/base/rexx-run pad1=$B/pad1/rexx-run pad2=$B/pad2/rexx-run pad3=$B/pad3/rexx-run r1=/tmp/claude-1000/p61/t2/bin/r1/rexx-run t3=$S/bin/fr2/rexx-run
```

(`S` is `/tmp/claude-1000/p61/t3`.) Exit 0. The layout controls print `+0.0000`% on every program, so
the band is 0 and the budget is +0.5%.

| program | r1 % | t3 % | verdict |
|---|---:|---:|---|
| emptyloop | -0.3229 | -0.0019 | inside |
| decloop | +0.4373 | -2.8190 | inside |
| rexxcps | +0.2741 | -0.0008 | inside |

The first fix-round binary (`1d7b27de8` with `9172b7085`) measured emptyloop +0.3192% and decloop
+1.0206% against base on the same command: the header and the wide step asked
`operator_message_receiver` on every value. `012bf8ab1` asks it only after a conversion fails.

```
PROGRAMS="emptyloop decloop rexxcps" bash rust/bench-programs/wallclock.sh -r 5 -o $S/wall base=$B/base/rexx-run pad2=$B/pad2/rexx-run t3=$S/bin/fr2/rexx-run
```

Exit 0; load average 1.51 at start, 1.28 at end.

| program | pad2 % | t3 % |
|---|---:|---:|
| emptyloop | +0.47 | +3.96 |
| decloop | -0.84 | -5.88 |
| rexxcps | -0.87 | -0.87 |

Every program inside ±4%; emptyloop's wall clock is the one Moritz ruled on at Task 2.

### Task 3 fix round 3

`t3` from `git archive 0c75907d3 rust interpreter`, built as above, one `Compiling rexx-exec` line,
sha256 `f5ea13e699bd1e7e034b44b4fe9db87825a86e0fddf561fefe058883a66536f8`.

```
bash rust/bench-programs/callgrind.sh -r 3 -j 10 -o $S/cg-h3 -p "emptyloop decloop rexxcps" base=$B/base/rexx-run pad1=$B/pad1/rexx-run pad3=$B/pad3/rexx-run t3=$S/bin/h3/rexx-run
```

Exit 0; the pads print `+0.0000`% on every program, so the band is 0 and the budget +0.5%.

| program | t3 % | verdict |
|---|---:|---:|
| emptyloop | -0.9620 | inside |
| decloop | -2.6728 | inside |
| rexxcps | -0.4940 | inside |

Perf round 1 of this fix round: `dcd1db992` measured emptyloop +0.9614% (3 instructions a pass in
`ops_loop_steady`, where the pass boundary inlined the loop-object release); `0c75907d3` makes the
release `#[cold] #[inline(never)]`.

## Task 4

Commits: `7d233521f` (behaviour), `899db70ac` (SOURCELINE expectations). Per-task check at
`899db70ac`: `cargo fmt` clean; `memcap 8G cargo clippy -j 4 --workspace --all-targets -- -D
warnings` exit 0; `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec --test corpus --test
ir_recorded_oracle` exit 0 (corpus 29 passed, 1 ignored; ir_recorded_oracle 21 passed); `memcap 8G cargo test -j 4 --workspace --no-fail-fast` exit 0.
At `7d233521f` alone that run was red on `rexx-parse --test sourceline_oracle` (no expectations
for the new witnesses, and none for Task 3's `do_object_compare_array`); `899db70ac` adds them.

### Step 3: the three `define` probes

`b6_define_array_subclass_with_primitive`, `b10_define_constant_method` and
`b10_define_attr_method` answer the oracle's `2`, `5` and `A` after b5 (`define` installs the
identity the Method object runs under), so nothing goes under row 9. They are the corpus programs
`define_native_array_subclass.rex`, `define_constant_method.rex` and `define_attribute_method.rex`.

### Step 3: a borrowed native row on a receiver of the wrong type (Task 7's DEVIATION evidence)

With b5, `run` hands a primitive's own row to an instance of another class, and the row reaches a
`receiver_class` guard. Probe texts, scout A's (`/tmp/claude-1000/p61/sa/probes/<name>/p.rex`), each
`say .t~new~go(X)` over `::class t` / `::method go` / `use arg m` / `return self~run(m)`:

| probe | X | ours, default and `REXX_SWITCH_MODE=every` | oracle, 5 runs |
|---|---|---|---|
| `b2_run_list_items_on_inst` | `.list~method('ITEMS')` | rc 120 `a message send to a value that is not a list is not implemented (Phase 5)` | SIGSEGV rc 139, 5 of 5 |
| `b2_run_routine_call_on_inst` | `.routine~method('CALL')` | rc 120 `a message send to a routine object this crate did not build is not implemented (Phase 5)` | SIGSEGV rc 139, 5 of 5 |
| `b2_run_array_items_on_inst` | `.array~method('ITEMS')` | rc 120 `a message send to a value that is not an array is not implemented (Phase 5)` | rc 0, a different number each run (`139900247187392`, `140122939563968`, `139928481144768`, `140276912463808`, `139996166725568`) |

Ours, one run each: `b2_run_supplier_item_on_inst` `a value that is not a supplier`,
`b2_run_package_name_on_inst` `a package object this crate did not build`,
`b2_run_varref_name_on_inst` `a value that is not a variable reference`,
`b2_run_stackframe_name_on_inst` `a stack frame this crate did not build`,
`b2_run_queue_items_on_inst` `a value that is not an array`; each rc 120 with `(Phase 5)`, both
engine modes identical. The routine row is the site `dispatch/executable.rs` `begin_routine` had as
`method_from_source("a routine whose body this crate does not hold")` for a record it does not hold;
it is now a `receiver_class` guard.

### Performance

Task 4 changes the send path (`begin_method` loses the `method_body_gap` read; `own_method_entry`
gains a class-object arm behind `object_methods`), so it is measured against `a3c2c3c0a`. `P` is
`/tmp/claude-1000/p61/t4`; each tree from `git archive <sha> rust interpreter`, every file touched,
built with `CARGO_TARGET_DIR=$P/target-<name> memcap 8G cargo build --release -j 4 -p rexx-exec --bin
rexx-run`, one `Compiling rexx-exec` line each.

| binary | source | sha256 |
|---|---|---|
| base | `a3c2c3c0a` | `21d594986d032d3b440ff11b0164980fa61eea4e5e657ea06245e4f1d225331b` |
| t4 | `7d233521f` | `be0af11d393dbfc543774e81ac02a323f7eb8208ab83fd86857c0ed153c143a5` |

```
bash rust/bench-programs/callgrind.sh -r 3 -j 8 -o $P/cg -p "fibcall fibfunc dispatch dispatchclass sendloop rexxcps emptyloop" base=$P/target-base/release/rexx-run t4=$P/target-head/release/rexx-run
```

Exit 0, every spread at most 0.0001%. No layout pads were run at this commit; the earlier tasks'
pads printed `0.0000`% on these programs, and no delta here is near the budget.

| program | t4 % | verdict |
|---|---:|---|
| fibcall | +0.0002 | inside |
| fibfunc | +0.0003 | inside |
| dispatch | -1.1319 | inside |
| dispatchclass | -1.1938 | inside |
| sendloop | -1.6774 | inside |
| rexxcps | +0.0001 | inside |
| emptyloop | +0.0003 | inside |

## Task 4a

Commits: `d27a9d441` (behaviour, witnesses, table), `08876442c` (`refusal-sites.tsv` re-derived for
the two new raisers).

### Step 1: the truth table

Generated, run and compared by the committed tooling, `T` =
`.superpowers/sdd/2026-10-07-phase-6-1/task-4a-truth-table`, `R` = `/tmp/claude-1000/p61/t4a/fr1c`:

```
python3 -I $T/gen.py $R/probes
bash $T/run.sh oracle $R/probes $R/oracle
bash $T/run.sh crate $R/probes $R/crate rust/target/debug/rexx-run
bash $T/run.sh crate $R/probes $R/base /tmp/claude-1000/p61/t4a/target-base/release/rexx-run
bash $T/compare.sh $R/oracle $R/crate
python3 -I $T/table.py $R/oracle   # and $R/crate, $R/base
```

`gen.py` writes one program per value and context; `run.sh` runs each from its own fresh empty
directory. Values: `gt` `2 > 1`, `dtrue` `.true`, `eq` `1 = 2`, `dfalse` `.false`, `q1` `'1'`, `i1`
`1`, `sum` `0+1`, `cat` `'1'||''`, `left` `left('12',1)`, `str` `1~string`, `tcopy` `.true~copy`,
`a1` `.array~of(1)`, `q0` `'0'`, `a0` `.array~of(0)`, `banana`, `sp1` `' 1'`, `a12`
`.array~of(1,2)`, `s1`/`s0` an object whose `STRING` answers `1`/`0`, `sd` one with the default
`STRING`. Contexts: IF, WHEN, WHILE, UNTIL (`T`/`F`), `\v`, `1 & v`, `if 1, v`; `ifcmp`, `whencmp`,
`whilecmp`, the condition `o = 1` over a user `=` answering the value; SELECT CASE on an object
whose `==` answers the value; DO TO (user `>`, `n` passes of `for 3`); BY (user `<`, the control's
`<`/`>` answering `1 = 1`/`1 = 0`, so `0` passes means descending); and `hasItem`/`index` on Array,
List and Table over items whose `==` answers it. A cell is stdout, or the error code and rc.

Oracle:

| value | if | when | while | until | not | and | list | ifcmp | whencmp | whilecmp | case | doto | by | arr_hasItem | arr_index | lst_hasItem | lst_index | tbl_hasItem | tbl_index |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| gt | T | T | T | T | 0 | 1 | T | T | T | T | T | 0 | 0 | 1 | 1 | 1 | 0 | 1 | k |
| dtrue | T | T | T | T | 0 | 1 | T | T | T | T | T | 0 | 0 | 1 | 1 | 1 | 0 | 1 | k |
| eq | F | F | F | F | 1 | 0 | F | F | F | F | F | 3 | 3 | 0 | The NIL object | 0 | The NIL object | 0 | The NIL object |
| dfalse | F | F | F | F | 1 | 0 | F | F | F | F | F | 3 | 3 | 0 | The NIL object | 0 | The NIL object | 0 | The NIL object |
| q1 | T | T | T | T | 0 | 1 | T | T | T | T | T | 3 | 3 | 1 | 1 | 1 | 0 | 1 | k |
| i1 | T | T | T | T | 0 | 1 | T | T | T | T | T | 3 | 3 | 1 | 1 | 1 | 0 | 1 | k |
| sum | T | T | T | T | 0 | 1 | T | T | T | T | T | 3 | 3 | 1 | 1 | 1 | 0 | 1 | k |
| cat | T | T | T | T | 0 | 1 | T | T | T | T | T | 3 | 3 | 1 | 1 | 1 | 0 | 1 | k |
| left | T | T | T | T | 0 | 1 | T | T | T | T | T | 3 | 3 | 1 | 1 | 1 | 0 | 1 | k |
| str | T | T | T | T | 0 | 1 | T | T | T | T | T | 3 | 3 | 1 | 1 | 1 | 0 | 1 | k |
| tcopy | T | T | T | T | 0 | 1 | T | T | T | T | T | 3 | 3 | 1 | 1 | 1 | 0 | 1 | k |
| a1 | T | T | T | T | 97.1 rc159 | 1 | T | T | T | T | T | 3 | 3 | 1 | 1 | 1 | 0 | 1 | k |
| q0 | F | F | F | F | 1 | 0 | F | F | F | F | F | 3 | 3 | 0 | The NIL object | 0 | The NIL object | 0 | The NIL object |
| a0 | F | F | F | F | 97.1 rc159 | 0 | F | F | F | F | F | 3 | 3 | 0 | The NIL object | 0 | The NIL object | 0 | The NIL object |
| banana | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | 34.4 rc222 | 34.901 rc222 | 34.901 rc222 | 34.6 rc222 | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | 34.905 rc222 | 3 | 3 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 |
| sp1 | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | 34.4 rc222 | 34.901 rc222 | 34.901 rc222 | 34.6 rc222 | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | 34.905 rc222 | 3 | 3 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 |
| a12 | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | 34.4 rc222 | 97.1 rc159 | 34.901 rc222 | 34.6 rc222 | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | 34.905 rc222 | 3 | 3 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 |
| s1 | T | T | T | T | 97.1 rc159 | 1 | T | T | T | T | T | 3 | 3 | 1 | 1 | 1 | 0 | 1 | k |
| s0 | F | F | F | F | 97.1 rc159 | 0 | F | F | F | F | F | 3 | 3 | 0 | The NIL object | 0 | The NIL object | 0 | The NIL object |
| sd | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | 34.4 rc222 | 97.1 rc159 | 34.901 rc222 | 34.6 rc222 | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | 34.905 rc222 | 3 | 3 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 |

This crate, the debug build of the tree at `f691ac33b`. `compare.sh` printed `380` cells, then
`13 by` and `13 doto`: every cell outside DO TO and BY is identical to the oracle on stdout, stderr
and status, and the DO TO/BY cells that differ are Deviation 25's (true ends the loop or counts
down, a non-logical answer is 34.901):

| value | if | when | while | until | not | and | list | ifcmp | whencmp | whilecmp | case | doto | by | arr_hasItem | arr_index | lst_hasItem | lst_index | tbl_hasItem | tbl_index |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| gt | T | T | T | T | 0 | 1 | T | T | T | T | T | 0 | 0 | 1 | 1 | 1 | 0 | 1 | k |
| dtrue | T | T | T | T | 0 | 1 | T | T | T | T | T | 0 | 0 | 1 | 1 | 1 | 0 | 1 | k |
| eq | F | F | F | F | 1 | 0 | F | F | F | F | F | 3 | 3 | 0 | The NIL object | 0 | The NIL object | 0 | The NIL object |
| dfalse | F | F | F | F | 1 | 0 | F | F | F | F | F | 3 | 3 | 0 | The NIL object | 0 | The NIL object | 0 | The NIL object |
| q1 | T | T | T | T | 0 | 1 | T | T | T | T | T | 0 | 0 | 1 | 1 | 1 | 0 | 1 | k |
| i1 | T | T | T | T | 0 | 1 | T | T | T | T | T | 0 | 0 | 1 | 1 | 1 | 0 | 1 | k |
| sum | T | T | T | T | 0 | 1 | T | T | T | T | T | 0 | 0 | 1 | 1 | 1 | 0 | 1 | k |
| cat | T | T | T | T | 0 | 1 | T | T | T | T | T | 0 | 0 | 1 | 1 | 1 | 0 | 1 | k |
| left | T | T | T | T | 0 | 1 | T | T | T | T | T | 0 | 0 | 1 | 1 | 1 | 0 | 1 | k |
| str | T | T | T | T | 0 | 1 | T | T | T | T | T | 0 | 0 | 1 | 1 | 1 | 0 | 1 | k |
| tcopy | T | T | T | T | 0 | 1 | T | T | T | T | T | 0 | 0 | 1 | 1 | 1 | 0 | 1 | k |
| a1 | T | T | T | T | 97.1 rc159 | 1 | T | T | T | T | T | 0 | 0 | 1 | 1 | 1 | 0 | 1 | k |
| q0 | F | F | F | F | 1 | 0 | F | F | F | F | F | 3 | 3 | 0 | The NIL object | 0 | The NIL object | 0 | The NIL object |
| a0 | F | F | F | F | 97.1 rc159 | 0 | F | F | F | F | F | 3 | 3 | 0 | The NIL object | 0 | The NIL object | 0 | The NIL object |
| banana | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | 34.4 rc222 | 34.901 rc222 | 34.901 rc222 | 34.6 rc222 | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | 34.905 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 |
| sp1 | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | 34.4 rc222 | 34.901 rc222 | 34.901 rc222 | 34.6 rc222 | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | 34.905 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 |
| a12 | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | 34.4 rc222 | 97.1 rc159 | 34.901 rc222 | 34.6 rc222 | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | 34.905 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 |
| s1 | T | T | T | T | 97.1 rc159 | 1 | T | T | T | T | T | 0 | 0 | 1 | 1 | 1 | 0 | 1 | k |
| s0 | F | F | F | F | 97.1 rc159 | 0 | F | F | F | F | F | 3 | 3 | 0 | The NIL object | 0 | The NIL object | 0 | The NIL object |
| sd | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | 34.4 rc222 | 97.1 rc159 | 34.901 rc222 | 34.6 rc222 | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | 34.905 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 | 34.901 rc222 |

This crate at the base `24394ca34` (release), for the record of what changed: CASE answered `F`
for every value (text compare against `'x'`); `s1`/`s0` raised 34.1-34.4/34.6 in every condition
context; every collection cell answered false where the oracle answers true or raises.

| value | if | when | while | until | not | and | list | ifcmp | whencmp | whilecmp | case | doto | by | arr_hasItem | arr_index | lst_hasItem | lst_index | tbl_hasItem | tbl_index |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| gt | T | T | T | T | 0 | 1 | T | T | T | T | F | 0 | 0 | 1 | 1 | 1 | 0 | 1 | k |
| dtrue | T | T | T | T | 0 | 1 | T | T | T | T | F | 0 | 0 | 1 | 1 | 1 | 0 | 1 | k |
| eq | F | F | F | F | 1 | 0 | F | F | F | F | F | 3 | 3 | 0 | The NIL object | 0 | The NIL object | 0 | The NIL object |
| dfalse | F | F | F | F | 1 | 0 | F | F | F | F | F | 3 | 3 | 0 | The NIL object | 0 | The NIL object | 0 | The NIL object |
| q1 | T | T | T | T | 0 | 1 | T | T | T | T | F | 0 | 0 | 1 | 1 | 1 | 0 | 1 | k |
| i1 | T | T | T | T | 0 | 1 | T | T | T | T | F | 0 | 0 | 1 | 1 | 1 | 0 | 1 | k |
| sum | T | T | T | T | 0 | 1 | T | T | T | T | F | 0 | 0 | 1 | 1 | 1 | 0 | 1 | k |
| cat | T | T | T | T | 0 | 1 | T | T | T | T | F | 0 | 0 | 1 | 1 | 1 | 0 | 1 | k |
| left | T | T | T | T | 0 | 1 | T | T | T | T | F | 0 | 0 | 1 | 1 | 1 | 0 | 1 | k |
| str | T | T | T | T | 0 | 1 | T | T | T | T | F | 0 | 0 | 1 | 1 | 1 | 0 | 1 | k |
| tcopy | T | T | T | T | 0 | 1 | T | T | T | T | F | 0 | 0 | 1 | 1 | 1 | 0 | 1 | k |
| a1 | T | T | T | T | 97.1 rc159 | 1 | T | T | T | T | F | 0 | 0 | 1 | 1 | 1 | 0 | 1 | k |
| q0 | F | F | F | F | 1 | 0 | F | F | F | F | F | 3 | 3 | 0 | The NIL object | 0 | The NIL object | 0 | The NIL object |
| a0 | F | F | F | F | 97.1 rc159 | 0 | F | F | F | F | F | 3 | 3 | 0 | The NIL object | 0 | The NIL object | 0 | The NIL object |
| banana | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | 34.4 rc222 | 34.901 rc222 | 34.901 rc222 | 34.6 rc222 | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | F | 34.901 rc222 | 34.901 rc222 | 0 | The NIL object | 0 | The NIL object | 0 | The NIL object |
| sp1 | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | 34.4 rc222 | 34.901 rc222 | 34.901 rc222 | 34.6 rc222 | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | F | 34.901 rc222 | 34.901 rc222 | 0 | The NIL object | 0 | The NIL object | 0 | The NIL object |
| a12 | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | 34.4 rc222 | 97.1 rc159 | 34.901 rc222 | 34.6 rc222 | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | F | 34.901 rc222 | 34.901 rc222 | 0 | The NIL object | 0 | The NIL object | 0 | The NIL object |
| s1 | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | 34.4 rc222 | 97.1 rc159 | 1 | 34.6 rc222 | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | F | 34.901 rc222 | 34.901 rc222 | 0 | The NIL object | 0 | The NIL object | 0 | The NIL object |
| s0 | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | 34.4 rc222 | 97.1 rc159 | 0 | 34.6 rc222 | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | F | 34.901 rc222 | 34.901 rc222 | 0 | The NIL object | 0 | The NIL object | 0 | The NIL object |
| sd | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | 34.4 rc222 | 97.1 rc159 | 34.901 rc222 | 34.6 rc222 | 34.1 rc222 | 34.2 rc222 | 34.3 rc222 | F | 34.901 rc222 | 34.901 rc222 | 0 | The NIL object | 0 | The NIL object | 0 | The NIL object |

### Step 4: mutation

Each mutant applied to the working tree, `memcap 8G cargo test -j 4 -p rexx-exec --test truth`
run, the file restored from a copy and `git diff --stat` checked unchanged.

| mutant | result |
|---|---|
| `condition_value` judges `to_text(value)` instead of `truth` | red: `the contexts judge .s1~new differently` (if/when/while/until error, the rest true) |
| `same_item` answers `to_text(answer) == "1"` | red: `the contexts judge 'banana' differently` (the six collection contexts false, the rest error) |

### Performance

`P` is `/tmp/claude-1000/p61/t4a`. Each tree from `git archive <sha> rust interpreter`, built with
`CARGO_TARGET_DIR=$P/target-<name> memcap 8G cargo build --release -j 4 -p rexx-exec --bin
rexx-run`, one `Compiling rexx-exec` line each.

| binary | source | sha256 |
|---|---|---|
| base | `24394ca34` | `1b2ac8d64bcb519844232776df870959be1079d37222e455fc7fb3b957615577` |
| t4a | `d27a9d441` | `6d19eb53f24b39a41f074b879f0277b36306352f6042e87f17d965fc165daa35` |

```
bash rust/bench-programs/callgrind.sh -r 3 -j 8 -o $P/cg2 -p "rexxcps emptyloop decloop dispatch" base=$P/target-base/release/rexx-run t4a=$P/target-head2/release/rexx-run
```

Exit 0, every spread 0.0000%. No layout pads.

| program | t4a % | verdict |
|---|---:|---|
| rexxcps | +0.1847 | inside |
| emptyloop | +0.6478 | **over** (+0.5% budget) |
| decloop | -0.0833 | inside |
| dispatch | +0.1943 | inside |

emptyloop runs no truth judgment (`do i = 1 to n; nop; end`). The difference is 2 instructions a
pass inside `ops_loop_steady` (measured on the same loop at `n = 200000` with `--dump-instr=yes`:
`ChunkTrace` read twice a pass where the base reads it once, and inlined lines of `run.rs`,
`slice/index.rs` and `vec/mod.rs` re-attributed), a register-allocation change in the driver.
Four variants of the head tree, each measured with the same script at `-r 1`, left emptyloop at
+0.6478% every time: the IF quick path written out inline as at the base; `Activation` padded
back to 480 bytes; `open_select_case` `#[inline(never)]`; `register_holds` as a `match`. Stopped
there for a ruling.

```
PROGRAMS="emptyloop decloop rexxcps dispatch" bash rust/bench-programs/wallclock.sh -r 5 -o $P/wall2 base=$P/target-base/release/rexx-run t4a=$P/target-head2/release/rexx-run
```

Exit 0; load average 2.68 at the end.

| program | t4a % |
|---|---:|
| emptyloop | -0.23 |
| decloop | -1.76 |
| rexxcps | -0.92 |
| dispatch | -8.54 |

### Per-task check

At `d27a9d441`: `cargo fmt --all --check` exit 0; `memcap 8G cargo clippy -j 4 --workspace
--all-targets -- -D warnings` exit 0; `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec
--test corpus --test ir_recorded_oracle` exit 0 (corpus 29 passed, 1 ignored; ir_recorded_oracle 21
passed); `memcap 8G cargo test -j 4 --workspace --no-fail-fast` exit 101, the one failure
`refusal_sites` (the two new raisers and moved line numbers). At `08876442c` after
`REXX_REFUSAL_SITES_REFRESH=1`: the workspace run exit 0, and clippy re-run exit 0 (one `Checking
rexx-exec` line). `whole_groups` not run: no whole-group
expectation line changed.

## Task 5

Commits: `59eb57f37` (behaviour, witnesses, exclusions rows, `refusal-sites.tsv`), `bef52e1b6`
(fixtures that do not parse renamed `.cls`), `7824d57df` (perf round 1). `S` is the session
scratchpad's `t5/perf`; each binary from `git archive <sha> rust interpreter`, built with
`CARGO_TARGET_DIR=$S/target-NAME memcap 8G cargo build --release -j 4 -p rexx-exec --bin rexx-run`,
one `Compiling rexx-exec` line each. `base61` is Task 1's base binary (sha256 re-checked).

| binary | source | sha256 |
|---|---|---|
| base | `f1202acc1` | `b469013d2f8b5ea278488f4a626c1a3d0f0b0c54b175026bc43d08ab17c502c6` |
| pad48 | base + `layout-pad.py 48` | `6ece878099c80836cc55b1f986eaad6feaf05abe79352d8b253761be3f67f4ad` |
| t5 | `59eb57f37` | `bde248b6f8330d544375b5347d3c3738d232676066149ffd6e80f7aab3ae90ef` |
| r1 | `7824d57df` | `e13d0e9e5862d3fa2a21ac9ac70c8a067656da107fbc6a3a0e0b8ad63dcebf70` |

Instructions:

```
bash rust/bench-programs/callgrind.sh -r 3 -j 8 -o $S/cg2 -p "startup parse" base61=/tmp/claude-1000/p61/t1/bin/base/rexx-run base=$S/bin/base/rexx-run pad48=$S/bin/pad48/rexx-run t5=$S/bin/t5/rexx-run r1=$S/bin/r1/rexx-run
```

Exit 0. Deltas against `base61`. pad48 against base is +0.0000% on `parse` and +0.0007% on `startup` (+0.0395 against +0.0388), so the band is about 0.

| program | base % | pad48 % | t5 % | r1 % | r1 against base % |
|---|---:|---:|---:|---:|---:|
| startup | +0.0388 | +0.0395 | +0.5292 | +0.2646 | +0.2257 |
| parse | +0.6004 | +0.6004 | +0.6190 | +0.6090 | +0.0085 |

t5 was +0.49% on `startup` against `base`: `ParseError` grew from two words to four, and the
scanner and `Result` lines inlined into `rexx_parse::parse` grew (`callgrind_annotate`: +94k on
`scanner.rs`, +80k on `result.rs`). Round 1 keeps it two words (`byte` and a `u32` end). `parse`'s
running total is over +0.5% at `base` already; Task 5 adds +0.0085%.

Wall clock:

```
PROGRAMS="startup parse" bash rust/bench-programs/wallclock.sh -r 5 -o $S/wall base=$S/bin/base/rexx-run pad48=$S/bin/pad48/rexx-run r1=$S/bin/r1/rexx-run
```

Exit 0; load average 1.98 at start and end.

| program | pad48 % | r1 % |
|---|---:|---:|
| startup | +4.17 | +4.17 |
| parse | +0.00 | +0.78 |

`startup`'s medians are 0.024 s against 0.025 s, one step of the script's resolution; the pad
shows the same step.

`whole_groups` (`REXX_CORPUS_GATE=1 memcap 8G cargo test -j 1 --release -p rexx-exec --test
concurrency_tests whole_groups`, at `7824d57df`): OOM-killed at the 8G cap, peak 8.0G, about nine
minutes in, before the table was written. Not re-run at a higher cap; its expectation lines are
unchanged.

Fix round 1 (`eb3775477`, binary `fr1`, sha256
`58f637e888af95cf4d891cc66d387e46cade4677a3632a80e1bbe3cea9595a24`), the same callgrind command
with `fr1=$S/bin/fr1/rexx-run` added, exit 0: startup +0.2641%, parse +0.6089% against `base61`.

### Task 5 fix round 1, I2: `whole_groups`

Command, `S` the session scratchpad's `t5i2`:

```
REXX_CORPUS_GATE=1 REXX_WHOLE_GROUPS_TABLE=$S/runN/table.tsv memcap 8G /usr/bin/time -v cargo test -j 1 --release -p rexx-exec --test concurrency_tests whole_groups
```

| run | tree | result | wall | max RSS |
|---|---|---|---|---|
| 1 | `ff982eb46` (harness change `93c7c19fe`, rows unchanged) | exit 101, 5 passed, the main test on the rows below only | 4:10 | 878 188 KB |
| 2 | `a945a5ae8` (rows rewritten) | exit 0, 6 passed | 5:14 | 2 385 136 KB |
| 3 | `a945a5ae8`, no rebuild | exit 0, 6 passed | 25:58 | 819 136 KB |

Max RSS is `time -v`'s, the largest of cargo and everything it waited for. Run 2 rebuilt the test
binary and has no line of its own for the test process, so its figure may be rustc's; runs 1 and 3
built nothing. Run 3's wall clock is one oracle run of MutexSemaphore (1 of 30, both parts) that hung
to the 300 s oracle deadline, which made this side's deadline 4 x 300 s for the every-mode run the
row lists as exceeding it. Runs 2 and 3 give the same normal and every cells on every row but
REPLY's, whose runs here matched a different one of the oracle's racing outcomes.

Every moved row, with its cause. Each was attributed by running the harness's copy of the group
(the same renamed-out tests, started lines marked, `-U -V 2`, normal mode, under `memcap 2G` and
`timeout 300`) under `rexx-run` built from these trees (`git archive`, own target directory, one
`Compiling rexx-exec` line each): `pre4` = `ec97e4190`, `t4` = `4c63d5b69`, `t4a` = `f691ac33b`,
`t5` = `93c7c19fe`. At `t5` every copy reproduces the harness's key exactly. `t4` and `t4a` agree on
every copy, so no row moved under Task 4a.

| group | part | was | now | cause and evidence |
|---|---|---|---|---|
| Class | whole | refuses at TEST_CLASS_DEFINE, `define` (Phase 9) | failure, assertions 298, failing [TEST_ACTIVATE TEST_METHODS] | Task 4 runs TEST_CLASS_DEFINE (`pre4` refuses there, `t4` rc 1 with this key); the harness change leaves TEST_SUBCLASSES_GC out, without which the run allocates until the cap (`whole-groups-memory.md`). The failing pair is the old rest row's |
| Class | rest | assertions 202 | not run; rows deleted | the whole run no longer refuses |
| Method | rest | assertions 63 | assertions 64, failing set unchanged | Task 4: TEST_NEW_TWO_ARGS_STRING_NIL (`expectSyntax(93.961)`) refused at `pre4` (a method source that is neither a string nor an array) and runs and passes from `t4`. The old rest, re-derived at `pre4` with it left out: 63 |
| Object | rest | assertions 247 | assertions 249, failing set unchanged | Task 4: TEST_RUN_BAD_METHOD_SOURCE and TEST_SETMETHOD_BAD_METHOD_SOURCE refused at `pre4` with the same refusal and pass from `t4`. The old rest re-derived at `pre4`: 247 |
| ATTRIBUTE | whole | refuses at TESTABSTRACTTWICE, `does not parse here` (Phase 5) | error, assertions 276, failing [TESTDELEGATE TESTMISPLACEDCLASSMETHOD] | Task 5: the parse error is a SYNTAX condition the test traps (`t4a` refuses, `t5` this key). TESTMISPLACEDCLASSMETHOD: see below |
| ATTRIBUTE | rest | assertions 232 | not run; rows deleted | the whole run no longer refuses |
| CONSTANT | whole | refuses at TEST_BAD_NEGATIVE, `does not parse here` | refuses at TEST_EXPRESSION_ACTIVATE, Routine NEW (Phase 9) | Task 5 (`t4a` refuses at TEST_BAD_NEGATIVE, `t5` at TEST_EXPRESSION_ACTIVATE) |
| CONSTANT | rest | assertions 156 | assertions 170, failing set unchanged | Task 5: the rest now leaves out TEST_EXPRESSION_ACTIVATE and TEST_EXPRESSION_SELF instead of the tests that did not parse, which now run and pass |
| METHOD | whole | refuses at TESTABSTRACTEXTERNAL, `does not parse here` | refuses at TESTPACKAGE, StringTable ITEMS (Phase 9) | Task 5 (`t4a` and `t5`, as above) |
| METHOD | rest | assertions 139, last started TESTSTRINGNAME | assertions 179, last started TEST_DELEGATE_TWICE, failing set unchanged | Task 5: the rest leaves out TESTPACKAGE alone |
| CALL | whole | refuses at TEST_INVALID, `does not parse here` | agrees with the oracle; rows deleted, and the rest rows with them | Task 5 (`t4a` refuses, `t5` rc 0, 181 assertions) |
| GUARD | whole | refuses at TEST_INVALID_OPTION_ONOFF, `does not parse here` | agrees with the oracle; rows deleted | Task 5 (`t4a` refuses, `t5` rc 0, 26 assertions) |
| TRACE | whole | refuses at TEST_TRACE_OPTIONAL_ADDITIONAL, `does not parse here` | error, assertions 120, the old rest row's failing set | Task 5 (`t4a` refuses, `t5` this key) |
| TRACE | rest | assertions 118 | not run; rows deleted | the whole run no longer refuses |
| TRACE_TraceObject | whole | refuses at TEST_OBJECT_AND_SCOPE, a send to a method context whose scope no longer defines it | error, assertions 18, the old rest row's failing set and TEST_OBJECT_AND_SCOPE | Task 4 deleted the refusal (`pre4` refuses, `t4` this key). TEST_OBJECT_AND_SCOPE fails `assertEquals(arr~items, 19)` with 0: its TraceObject collector receives no lines, the gap Task 4's review names |
| TRACE_TraceObject | rest | assertions 18 | not run; rows deleted | the whole run no longer refuses |

No row is a regression: every test in a new failing set either failed in the old rest row or
refused before the task that moved it (so it never passed here). The new failures of that second
kind:

- ATTRIBUTE TESTMISPLACEDCLASSMETHOD expects 99.905 for `::attribute 'foo' class` followed by a
  body. The parser here reports 99.937 (attribute body without SET or GET), the oracle 99.905 (CLASS
  without a matching `::CLASS`). Measured on a two-line program: oracle rc 157 with 99.905; `t5`
  rc 157 with 99.937; `pre4` refuses with the same 99.937. Older than Phase 6.1.
- TRACE_TraceObject TEST_OBJECT_AND_SCOPE: the collector gap above.

Still listed and agreeing in both green runs, not moved by Tasks 4, 4a or 5: TIME whole and derived,
CALL derived (the elapsed-clock rows) and REPLY whole and derived every.

Per-task check at `a945a5ae8`: `cargo fmt --all --check` exit 0; `memcap 8G cargo clippy -j 4
--workspace --all-targets -- -D warnings` exit 0; `memcap 8G cargo test -j 4 --workspace
--no-fail-fast` exit 0; `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec --test corpus
--test ir_recorded_oracle` exit 0 (29 passed, 1 ignored; 21 passed).

## Task 5a

Commits: `8e52e14ca` (byte-counting trigger and its tests), `5f5b80b3d` (perf round 1: charge after
the allocation, collect on the next), `0d18b0045` (`refusal-sites.tsv` re-derived, line numbers
only). Report: `.superpowers/sdd/2026-10-07-phase-6-1/task-5a-report.md`. `P` is
`/tmp/claude-1000/p61/t5a`.

### Peak memory

`loopN` is `do i = 1 to N; x = copies('abc', 100000); end` (300 000-byte strings); `loop999` is
`do i = 1 to 1000000; x = copies('abc', 333) || i; end`. Ours: `memcap 2G /usr/bin/time -f "%M %e
%x" timeout -k 5 120 $BIN FILE`, `REXX_SWITCH_MODE=every` for the every cells. Oracle: `( ulimit -v
1048576; LD_LIBRARY_PATH=$ORACLE/lib /usr/bin/time ... timeout -k 5 20 rexx FILE )` from an empty
directory. 5 runs per cell, median max RSS in KB, all exit 0 except where stated.

| program | base normal | base every | t5a normal | t5a every | oracle |
|---|---:|---:|---:|---:|---:|
| loop4000 | 1 191 136 | 1 191 392 | 52 404 | 52 308 | 13 380 |
| loop20000 | OOM at 2G (1 run) | not run | 51 932 | 52 184 | 13 016 |
| loop999 | 90 064 | 89 684 | 55 364 | 55 452 | 20 580 |

Collection counts are not compared with the oracle (licensed divergence).

### Performance

| binary | source | sha256 |
|---|---|---|
| base | `37d874a36` | `66a67962b9e8e4053f88d73167a8762d0b5df585081d210c5dd5227028d5a305` |
| pad48 | base + `layout-pad.py 48` | `e5737b82031b9de2c323c2c3767bb3745c15d27f228fc1a48f676ff2eea4bdf3` |
| t5a | `0d18b0045` | `0736d76c9d03230ca55835d52a52686cfa8b5a90e8022cbb4f600165ca91053a` |

Each from `git archive <sha> rust interpreter`, every file touched, `CARGO_TARGET_DIR=$P/target-NAME
memcap 8G cargo build --release -j 4 -p rexx-exec --bin rexx-run`, one `Compiling rexx-exec` line
each.

```
memcap 8G bash rust/bench-programs/callgrind.sh -r 3 -j 8 -o $P/cg-final -p "alloc alloc4c strings rexxcps emptyloop" base=$P/target-base/release/rexx-run pad48=$P/target-pad48/release/rexx-run t5a=$P/target-final/release/rexx-run
```

Exit 0, every spread 0.0000%, pad48 +0.0000% on every program.

| program | t5a % | verdict |
|---|---:|---|
| alloc | +0.0434 | inside |
| alloc4c | -0.4070 | inside |
| strings | +0.1016 | inside |
| rexxcps | +0.2035 | inside |
| emptyloop | +0.0000 | inside |

`8e52e14ca` alone was +0.80% on strings and +0.89% on rexxcps (the charge read the built `Bytes`,
which then was copied into its slot); round 1's variants are in the report.

```
PROGRAMS="alloc alloc4c strings rexxcps emptyloop" memcap 8G bash rust/bench-programs/wallclock.sh -r 5 -o $P/wall base=$P/target-base/release/rexx-run pad48=$P/target-pad48/release/rexx-run t5a=$P/target-final/release/rexx-run
```

Exit 0; load average 0.74 at start, 1.02 at end. pad48 / t5a %: alloc +0.37 / -0.75, alloc4c
+2.15 / +3.76, strings -1.56 / +0.35, rexxcps -0.41 / +1.17, emptyloop +0.67 / -2.23.

### Per-task check

At `0d18b0045`: `cargo fmt`; `memcap 8G cargo clippy -j 4 --workspace --all-targets -- -D
warnings` clean; `memcap 8G cargo test -j 4 --workspace --no-fail-fast` exit 0 (3066 passed, 4
ignored; `collect_stress` 37 passed); `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec
--test corpus --test ir_recorded_oracle` exit 0 (50 passed, 1 ignored).

### `whole_groups`

`REXX_CORPUS_GATE=1 REXX_WHOLE_GROUPS_TABLE=$P/wgN/table.tsv memcap 8G /usr/bin/time -v cargo test
-j 1 --release -p rexx-exec --test concurrency_tests whole_groups` at `0d18b0045`: run 1 (rebuilt)
exit 0, 6 passed, 6:42, max RSS 2 381 424 KB (rustc included); run 2 (no rebuild) exit 0, 6
passed, 4:10, max RSS 915 712 KB, against 819 136 KB for Task 5 I2's run 3 at `a945a5ae8`. The
harness leaves TEST_SUBCLASSES_GC out of every part, so neither run reaches the loop this task
bounds. Our cells differ between runs 1 and 2 only on REPLY (racing oracle outcomes).

### `loop999`'s cost

`loop999` (`do i = 1 to 1000000; x = copies('abc', 333) || i; end`), 10 interleaved wall runs at
`0d18b0045`: base 1.04 to 1.07 s; t5a 1.02 to 1.03 s in 3 runs and 1.13 to 1.24 s in 7.
Instructions are flat (callgrind at N = 100 000: +0.53%; `perf stat` 14.45 G against 14.40 G at
N = 1 000 000).

### Fix round 1

Code `d0a3d5db3`: `~copy`, arrays (creation, and growth through `array_grow`,
`array_splice_slot` and `array_resize`; a multidimensional extend was left uncharged until fix
round 2) and MutableBuffers (creation and capacity growth) are charged; `Outcome::peak_body_bytes` is `#[cfg(test)]`. Per-task check exit 0
(3070 passed, 4 ignored; corpus gate 50 passed, 1 ignored).

```
memcap 8G bash rust/bench-programs/callgrind.sh -r 3 -j 8 -o $P/cg-fr1 -p "alloc alloc4c strings rexxcps emptyloop" base=$P/target-b2/release/rexx-run fr1=$P/target-fr1/release/rexx-run
```

`b2` = `37d874a36` (sha256 `c9a451e632c9e6d64c924da7760287eefaf8798d6effb75816227228da8ae733`),
`fr1` = `d0a3d5db3` (sha256 `d2ac10ff755a3a5ece5e6b45a9c6ab3627eea2ec6caecd851e66ff45b5e05496`).
Exit 0, spreads 0.0000%: alloc +0.1905, alloc4c -0.3756, strings +0.1356, rexxcps +0.2383,
emptyloop +0.0000; all inside.

Peak RSS, one run each (base / fr1 / oracle, KB): `y~copy` of 300 KB x 3000 898 520 / 52 352 /
13 848; `.array~new(100000)` x 1000 1 582 256 / 54 884 / 22 156; MutableBuffer with three 100 KB
appends x 3000 911 376 / 44 388 / 15 448.

Wall-clock diagnosis (report, Fix round 1): `loop999`'s slow mode survives with the byte trigger
switched off (`COLLECT_BYTES_FLOOR` = 2^60: cycles 2.98 to 3.96 G against base 3.08 to 3.13 G,
instructions flat), so it is not the collections. The extra cycles are in `copies_bytes` and
`memmove`, and they move with the environment's size (heap placement). `array_fill20` shows no
regression on my build of `d9794b41f` (3.01 to 3.05 G against base 3.01 to 3.11 G), so the review's
+35% is one build's code placement.

## Task 6

Commits: `f062f791e` (pause placement, `.DebugInput`, witnesses), `004312db8` (perf round: the
pause ahead of the clause boundary, ITERATE's step out of line). Report:
`.superpowers/sdd/2026-10-07-phase-6-1/task-6-report.md`. `T` is `/tmp/claude-1000/p61/t6`.

### Performance

| binary | source | sha256 |
|---|---|---|
| base | `fe4956b36` | `2764952da672477aa8fae8a4588680a60026c7d5b7bc193cd0b7b06651f633f6` |
| nobranch | `004312db8`'s code without the three R7 sites (`ir/drive.rs` only) | `fa4476b1505e68b07c3916da7597829cc52608b4f276e2955d27e174194f7b1d` |
| head | `004312db8`'s code (release-identical: the commit's `clause.rs` edit is a `debug_assert`) | `ff3665d8d52bb536c2e670535b988e0e5a4808f8521ea4840ee453243e4bf4b1` |

Each in its own target directory with one `Compiling rexx-exec` line; base from `git archive
fe4956b36 rust interpreter` with every file touched.

```
memcap 8G bash rust/bench-programs/callgrind.sh -r 3 -j 4 -o $T/cg9 -p "emptyloop rexxcps" base=$T/target-base/release/rexx-run nobranch=$T/target-nb4/release/rexx-run head=$T/bin-head4/rexx-run
```

Exit 0, spreads 0.0001% at most.

| program | nobranch % | head % | verdict |
|---|---:|---:|---|
| rexxcps | +0.0016 | +0.0514 | inside |
| emptyloop | +0.0000 | +0.6442 | inside with R7 apart |

R7's branch (head against nobranch): rexxcps +0.050%, emptyloop +0.644%, accounted beside the
budget. `f062f791e` was rexxcps +0.8393% and emptyloop -0.3234% (`cg1`).

```
PROGRAMS="rexxcps emptyloop" memcap 8G bash rust/bench-programs/wallclock.sh -r 5 -o $T/wall1 base=$T/target-base/release/rexx-run nobranch=$T/target-nb4/release/rexx-run head=$T/bin-head4/rexx-run
```

Exit 0; load average 1.90 at start, 1.55 at end. nobranch / head %: rexxcps +1.44 / +0.21,
emptyloop -0.68 / +2.50; inside ±4%.

### Per-task check

At `004312db8`: `cargo fmt --all --check` exit 0; `memcap 8G cargo clippy -j 8 --workspace
--all-targets -- -D warnings` exit 0; `memcap 8G cargo test -j 4 --workspace --no-fail-fast` exit 0
(3073 passed, 4 ignored); `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec --test corpus
--test ir_recorded_oracle` exit 0 (50 passed, 1 ignored).

## Task 7

Commit `3b9a80364`. Per-task check at that commit: `cargo fmt --all` clean; `memcap 8G cargo
clippy -j 4 --workspace --all-targets -- -D warnings` exit 0; `memcap 8G cargo test -j 4
--workspace --no-fail-fast` exit 0 (3079 passed, 0 failed); `REXX_CORPUS_GATE=1 memcap 8G cargo
test -j 4 -p rexx-exec --test corpus --test ir_recorded_oracle` exit 0 (corpus 29 passed, 1
ignored; ir_recorded_oracle 21 passed). `git grep -n '"Phase 5"' -- rust/crates` prints only
`closed_phases.rs`'s `CLOSED`.

The disposition test failed first at `6860d4f80` with no table, listing 25 ownerless
constructors and two ownerless `Loud` struct literals; it passes at `3b9a80364`, and the table
without its `receiver_class` row fails naming that constructor. `no_owner_table_names_a_closed_phase`
failed on the `Literals` rows and a `bif_assertions.rs` report string before they were re-homed.
Dispositions, the `Literals` probes and the receiver split's probes are in
`.superpowers/sdd/2026-10-07-phase-6-1/task-7-report.md`. No performance run (refusal paths and
the `owed` owner type only).

### Fix round 1

Commits `732c65404`, `58a558ff1`. At `58a558ff1`: clippy exit 0; workspace debug test exit 0
(3079 passed, 0 failed); gated corpus exit 0 (29 passed, 1 ignored; ir_recorded_oracle 21).
Callgrind against `6860d4f80`, `-r 3`: rexxcps -0.0000%, dispatch +0.0727%, a Directory-read loop
-0.0756% (+0.8013% before `#[cold]` on the receiver refusals). Details in the Task 7 report.

## Task 8

Commits `b7a050d6b` (the mode), `909d87b09` (a refused endless wait ends the program's end).

### The oracle's per-clause time

`rust/bench-rexxcps/rexxcps.rex` on the oracle (`build/`, RelWithDebInfo), five runs from a fresh
directory, `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout
-k 5 60 /home/moritz/dev/repos/ooRexx/build/bin/rexx rexxcps.rex )`: 17,410,653, 17,187,757,
17,229,378, 16,947,286 and 17,328,049 clauses per second, all rc 0. Median 17,229,378, which is
58 ns per clause. The simulation's per-clause quantum is drawn once per run from 29 to 116 ns
(half and twice; `QUANTUM_NANOS` in `rexx-exec/src/sim.rs`). At that rate the oracle's 24 ms
slice is 413,793 clauses.

### Performance

`S` is `/tmp/claude-1000/p61/t8/perf`. Each binary from `git archive <sha> rust interpreter`, every
file touched, built with `CARGO_TARGET_DIR=$S/target-NAME memcap 8G cargo build --release -j 4 -p
rexx-exec --bin rexx-run`, one `Compiling rexx-exec` line each.

| binary | source | sha256 |
|---|---|---|
| base | `0765d19ef` | `14bae372deb870ad0a7ec80910f929be55807327700df8f8ac98a5e27ebe2d6f` |
| head2 | `909d87b09` | `8db9e19626471e2ae106bd14b2ecc753099cb15af65cb7191bc40a0b996d20cc` |

```
memcap 8G bash rust/bench-programs/callgrind.sh -r 3 -j 6 -o $S/cg2 -p "rexxcps emptyloop startup" base=$S/bin/base/rexx-run head=$S/bin/head2/rexx-run
```

Exit 0.

| program | base | head | head % |
|---|---:|---:|---:|
| rexxcps | 17792490814 | 17792513774 | +0.0001 |
| emptyloop | 7761204239 | 7761253168 | +0.0006 |
| startup | 58250145 | 58250393 | +0.0004 |

`b7a050d6b` measured the same way (`cg1`): +0.0001, +0.0006, +0.0001.

```
PROGRAMS="rexxcps emptyloop startup" memcap 8G bash rust/bench-programs/wallclock.sh -r 5 -o $S/wall3 base=$S/bin/base/rexx-run head=$S/bin/head2/rexx-run
```

Exit 0; load average 1.01 at start, 1.00 at end. rexxcps +1.32%, emptyloop -0.45%, startup
-4.00% (25 ms against 24 ms, the timer's millisecond); inside ±4%. Two earlier runs against
`b7a050d6b` at load average 5 to 15 (`wall1`, `wall2`) are not counted.

One callgrind run of `bench-programs/pingpong/pingmsg.rex` per binary, glibc included: base
7,585,612,504, head2 7,585,522,883 (-0.0012%).

### Per-task check

At `909d87b09`: `cargo fmt --all --check` exit 0; `memcap 8G cargo clippy -j 4 --workspace
--all-targets -- -D warnings` exit 0; `memcap 8G cargo test -j 4 --workspace --no-fail-fast` exit 0
(3094 passed, 0 failed, 4 ignored); `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec
--test corpus --test ir_recorded_oracle` exit 0 (corpus 29 passed, 1 ignored; ir_recorded_oracle
21 passed).

### Fix round 1

Commit `95df78c16`. Witnesses: a callback from `SENDFROMANOTHERTHREAD`'s own thread under `sim:1`
is refused (`a_callback_from_another_thread_is_refused_in_the_simulation_mode`: stdout `1`, rc 120,
in 0.12 s); the review's `p3/fifo.rex` (a fifo read by main's command and written by another
activity's) under `REXX_SWITCH_MODE=sim:1` through `rexx-run` is refused in 2.14 s wall, rc 120,
no process left behind; the crate test of the same shape under `block=0.5`.

Callgrind against `0765d19ef` (`$S/cg3`, head3 = `95df78c16`, sha256
`76ea39fb8677f9c6940af402af2f686fba21afe3bb28e24c1db92b5db08f56d8`): rexxcps +0.0001%, emptyloop
+0.0006%, startup +0.0000%. Wall clock `-r 5` (`$S/wall4`, load 1.26 to 1.24): rexxcps +2.22%,
emptyloop +0.22%, startup +4.17% (24 ms against 25 ms); startup alone at `-r 9` (`$S/wall5`):
+0.00%.

Per-task check at `95df78c16`: `cargo fmt --all --check` exit 0; clippy exit 0; workspace debug
test exit 0 (3097 passed, 0 failed, 4 ignored); gated corpus exit 0 (29 passed, 1 ignored;
ir_recorded_oracle 21 passed).

### Fix round 2

Commits `27cbe2510`, `da89562a3`. Witness: the review's `p4/sock2.rex` (an `rxsock` accept in a
started activity that main's later connect ends), loopback, port changed, through the debug
`rexx-run` with `LD_LIBRARY_PATH` at the oracle's `build/lib`: default mode `bind 0 listen 0`,
`connect 0`, `got 1`, rc 0 in 0.24 s; `sim:1` rc 120, `a native call in the simulation mode that
runs longer than its bound`, 2.12 s; `sim:1,block=0.5` the same in 0.62 s. The crate test
`a_native_call_only_another_activity_can_end_is_refused_after_its_bound` runs both modes.

Callgrind against `0765d19ef` (`$S/cg4`, head4 = `da89562a3`, sha256
`191222d8003eead560f5172d95882aa3341fd637882b0201d3c81ae64c18e044`): rexxcps +0.0001%, emptyloop
+0.0006%, startup +0.0009%.

Per-task check at `da89562a3`: `cargo fmt --all --check` exit 0; clippy exit 0; workspace debug
test exit 0 (3098 passed, 0 failed, 4 ignored); gated corpus exit 0 (29 passed, 1 ignored;
ir_recorded_oracle 21 passed).

### Fix round 3

Commit `1f7b73ac0`, sim code only (the `block=` parse and the watcher), so no perf run.
`REXX_SWITCH_MODE=sim:1,block=0` exits 2 with `block is a number of seconds from 0.001 to 86400`.
`quick_native_calls_at_the_smallest_bound_run_alike` (50 `RxCalcSqrt` calls that leave the driver,
`block=0.001`, 20 runs alike) passed in 5 separate runs. Per-task check at `1f7b73ac0`: fmt exit 0;
clippy exit 0; workspace debug test exit 0 (3099 passed, 0 failed, 4 ignored); gated corpus exit 0
(29 passed, 1 ignored; ir_recorded_oracle 21 passed).

## Task 9

Commits `60f21ad1b` (policies, invariants, trace and replay), `296cac74f` (perf round 1: the `gc=`
draw out of line).

### Performance

`S` is `/tmp/claude-1000/p61/t9/perf`. Each binary from `git archive <sha> rust interpreter`, every
file touched, built with `CARGO_TARGET_DIR=$S/target-NAME memcap 8G cargo build --release -j 4 -p
rexx-exec --bin rexx-run`, one `Compiling rexx-exec` line each. `callgrind.sh` now runs
`pingpong/*.rex` as `pingmsg`, `pingguard`, `pingsem`.

| binary | source | sha256 |
|---|---|---|
| base | `6358ca7a6` | `0e18eb55b6ba72406cbbf70e7078e97b278c7a29933b9929f3f483c9a6fe6706` |
| head | `60f21ad1b` | `907123352a6d8e536f020f68bd594a4e83602fdb82fe23c49473fcb19a8a109e` |
| head2 | `296cac74f` | `bef89dcd103d29a73a8bc9d341fe4bf5375700324b47ac1a126ec5bc52bf8356` |

```
memcap 8G bash rust/bench-programs/callgrind.sh -r 3 -j 6 -o $S/cg1 -p "pingmsg pingguard pingsem alloc alloc4c heapshape rexxcps emptyloop" base=$S/bin/base/rexx-run head=$S/bin/head/rexx-run
memcap 8G bash rust/bench-programs/callgrind.sh -r 3 -j 6 -o $S/cg2 -p "pingmsg pingguard pingsem alloc alloc4c heapshape rexxcps emptyloop" base=$S/bin/base/rexx-run head2=$S/bin/head2/rexx-run
```

Both exit 0. The `pingpong` programs' run-to-run spread is at most 0.0001% on either binary in
both runs, inside the budget, so they are gated like the others.

| program | head % | head2 % | head2 spread % | verdict |
|---|---:|---:|---:|---|
| pingmsg | +0.5498 | +0.2005 | 0.0000 | inside |
| pingguard | +0.2261 | +0.2259 | 0.0000 | inside |
| pingsem | +0.1874 | +0.1872 | 0.0001 | inside |
| alloc | +0.5330 | +0.3552 | 0.0000 | inside |
| alloc4c | +0.4536 | +0.0663 | 0.0000 | inside |
| heapshape | +0.6806 | +0.1280 | 0.0000 | inside |
| rexxcps | +0.4839 | +0.0694 | 0.0001 | inside |
| emptyloop | +0.0001 | +0.0000 | 0.0000 | inside |

head was over on pingmsg, alloc and heapshape: `cgdiff.py` put the cost in `collect_if_due` (+92 M
on rexxcps, +7.5 M on pingmsg), which had stopped being inlined into `alloc_with` once the grown
`sim_declines_collection` was inlined into it. Round 1 makes that hook `#[cold] #[inline(never)]`.
At head2 `alloc`'s remaining +0.36% is `core::str::converts::from_utf8` (+66,000,539) and
`alloc_with` (+6,065,969), code this task does not touch; no layout control was run, so it is not
attributed. `pingmsg`'s `message_completed` is +640,000 (eight instructions per completion: the
ready-queue mark and the `sim` test).

```
PROGRAMS="rexxcps emptyloop alloc alloc4c pingpong/pingmsg pingpong/pingsem pingpong/pingguard" memcap 8G bash rust/bench-programs/wallclock.sh -r 5 -o $S/wall1 base=$S/bin/base/rexx-run head2=$S/bin/head2/rexx-run
```

Exit 0; load average 1.81 at start, 3.47 at end. rexxcps -1.88%, emptyloop +0.22%, alloc +1.68%,
alloc4c +0.18%, pingmsg -0.23%, pingsem +0.80%, pingguard +0.82%; inside ±4%. heapshape prints
its own figures and was not timed.

### Per-task check

At `60f21ad1b`: `cargo fmt --all --check` exit 0; `memcap 8G cargo clippy -j 4 --workspace
--all-targets -- -D warnings` exit 0; `memcap 8G cargo test -j 4 --workspace --no-fail-fast` exit 0
(3105 passed, 0 failed, 4 ignored); `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec
--test corpus --test ir_recorded_oracle` exit 0 (corpus 29 passed, 1 ignored; ir_recorded_oracle
21 passed); `refusal_sites` 5 passed and `refusal_dispositions` 3 passed after
`REXX_REFUSAL_SITES_REFRESH=1`. At `296cac74f`: fmt and clippy exit 0, the `sim::` and `uniform_1`
lib tests 19 passed.

### Fix round 1: running totals against the 6.1 base

Commit `474e4bdb9`. `fix1` built as above from `git archive 474e4bdb9 rust interpreter` in
`/tmp/claude-1000/p61/t9f1/target-head`, one `Compiling rexx-exec` line. `base61` is Task 1's base
binary (`e6af1198b`), its sha256 matching the `## Task 1` table.

| binary | source | sha256 |
|---|---|---|
| fix1 | `474e4bdb9` | `cbc6aa343be07ff75e914daef4e0055f7ccd6de23ac943845204d18194d472ca` |
| base61 | `e6af1198b` | `2ea19b3ea2875fada8718eaf5b89f828d5ae07587a74feaf9b34681c879c891e` |

```
memcap 8G bash rust/bench-programs/callgrind.sh -r 2 -j 6 -o /tmp/claude-1000/p61/t9f1/cg1 -p "pingmsg pingguard pingsem alloc alloc4c heapshape rexxcps emptyloop" base=/tmp/claude-1000/p61/t9/perf/bin/base/rexx-run head2=/tmp/claude-1000/p61/t9/perf/bin/head2/rexx-run fix1=/tmp/claude-1000/p61/t9f1/bin/head/rexx-run base61=/tmp/claude-1000/p61/t1/bin/base/rexx-run
```

Exit 0, every spread at most 0.0017%. Percentages are against base61 except the last column.

| program | base (Tasks 1-8) % | head2 % | fix1 (running total) % | fix1 vs head2 % | verdict |
|---|---:|---:|---:|---:|---|
| pingmsg | -0.4541 | -0.2545 | -0.2545 | +0.0000 | inside |
| pingguard | -0.5914 | -0.3665 | -0.3661 | +0.0004 | inside |
| pingsem | -0.4358 | -0.2494 | -0.2485 | +0.0009 | inside |
| alloc | -0.2804 | +0.0738 | +0.0738 | +0.0000 | inside |
| alloc4c | -0.8796 | -0.8139 | -0.8139 | -0.0000 | inside |
| heapshape | +1.0265 | +1.1558 | +1.1558 | -0.0000 | **over** |
| rexxcps | +0.0230 | +0.0924 | +0.0924 | -0.0000 | inside |
| emptyloop | -0.3191 | -0.3191 | -0.3191 | -0.0000 | inside |

heapshape is +1.16% against the 6.1 base, +1.03% of it present before Task 9 (base, `6358ca7a6`).
It is being bisected separately and goes to Moritz for a ruling; this round does not change it.

### Heapshape round 1

Code commit `5214a2089` (live body bytes kept as a running figure; report
`.superpowers/sdd/2026-10-07-phase-6-1/heapshape-round-report.md`). Binaries from `git archive <sha>
rust interpreter`, every file touched, `CARGO_INCREMENTAL=0 memcap 8G cargo build --release -j 4 -p
rexx-exec --bin rexx-run`, one `Compiling rexx-exec` line each. `head` was built in
`/tmp/claude-1000/p61/hsr/target-base`, `r1` in `/tmp/claude-1000/p61/hsr/target`. base61's sha256
matches the `## Task 1` table.

| binary | source | sha256 |
|---|---|---|
| base61 | `e6af1198b` | `2ea19b3ea2875fada8718eaf5b89f828d5ae07587a74feaf9b34681c879c891e` |
| head | `7aedf9501` | `8b154c88a4defebf8aeba56f00f3dfdd56d8dccedbd3d7c4bcee59ddf5194b61` |
| r1 | `5214a2089` | `c9cf373fad59449e71ffe5a7d17dd4e993f4ee1129d228462f62cd741c58343f` |

```
memcap 8G bash rust/bench-programs/callgrind.sh -r 2 -j 6 -o /tmp/claude-1000/p61/hsr/cg1 -p "pingmsg pingguard pingsem alloc alloc4c heapshape rexxcps emptyloop" base61=/tmp/claude-1000/p61/t1/bin/base/rexx-run head=/tmp/claude-1000/p61/hsr/bin/head/rexx-run r1=/tmp/claude-1000/p61/hsr/bin/r1/rexx-run
```

Exit 0, every spread at most 0.0001%. Percentages are against base61 except the last column.

| program | head % | r1 (running total) % | r1 vs head % | verdict |
|---|---:|---:|---:|---|
| pingmsg | -0.2545 | +0.0096 | +0.2647 | inside |
| pingguard | -0.3663 | -0.3664 | -0.0000 | inside |
| pingsem | -0.2494 | -0.2494 | -0.0000 | inside |
| alloc | +0.0738 | +0.2330 | +0.1591 | inside |
| alloc4c | -0.8139 | +0.0607 | +0.8817 | inside |
| heapshape | +1.1559 | +0.0525 | -1.0907 | inside |
| rexxcps | +0.0924 | +0.4409 | +0.3482 | inside |
| emptyloop | -0.3191 | -0.3191 | +0.0000 | inside |

`cgdiff.py` head against r1: every program's change is in `Interp::collect_now` (the sweep, inlined):
heapshape -25.75M (the survivor sums gone), rexxcps +61.99M, alloc +29.39M, alloc4c +27.47M, pingmsg
+4.27M (the freed bodies' bytes read per dead object). Outside it, alloc's `array_of_class` is
+3.00M and pingmsg's `security_arguments_array` and `text_built` +80,000 each (the hold beside each
charge). No call count moved by more than 26 (`free`). The survivor cost moved to a per-dead-object cost, so rexxcps has
0.06% left under the budget.

```
PROGRAMS="rexxcps emptyloop alloc alloc4c heapshape pingpong/pingmsg pingpong/pingsem pingpong/pingguard" memcap 8G bash rust/bench-programs/wallclock.sh -r 5 -o /tmp/claude-1000/p61/hsr/wall2 base61=... head=... r1=...
```

Exit 0, load average 1.06 at start and 1.01 at end. r1 against base61: rexxcps +1.59%, emptyloop
+4.43% (head +5.36%), alloc -1.99%, alloc4c +1.62%, heapshape -0.55%, pingmsg +0.23%, pingsem
+8.94% (head +1.63%), pingguard +5.04% (head +1.68%). A rerun at `-r 11` (`wall3`) gave pingsem
+8.26% (head +4.13%), pingguard +4.20% (head +0.84%), emptyloop +3.48% (head +3.02%). Against a
layout control (head with `layout-pad.py 48`, `wall4`, `-r 11`, against head): pad48 pingsem
+1.59%, pingguard +0.00%; r1 pingsem +3.17%, pingguard +3.28%. r1 and head run the same instructions
on pingsem and pingguard (310 and 114 Ir apart), so the wall-clock gap is not added work; not
attributed further.

## Task 10

The seeded gate: `concurrency_tests.rs` `group_runs::whole_groups::sim_gate`, parts and seed counts
in `rust/corpus/sim-gate.tsv`, exemptions in `rust/corpus/sim-exempt.tsv`, oracle sets in
`rust/corpus/sim-oracle/`. Report: `.superpowers/sdd/2026-10-07-phase-6-1/task-10-report.md`.

### Calibration and budget

Seed 0 of each part under `fifo`, both builds (`REXX_SIM_CALIBRATION=FILE ... cargo test
[--release] -p rexx-exec --test concurrency_tests sim_gate::calibration`, 8 jobs): 196 rows, k and
both wall times per row in `sim-gate.tsv`; summed wall time 14.2 s release, 19.5 s debug; the largest
row DateTime whole, 5.0 s and 8.1 s. Seeds: 70 per row in release; 14 per row in debug on the 128
rows that are derived parts, programs with k at least 1, the gate's own programs and MutexSemaphore
single. Run deadline: calibrated time x 10, at least 60 s.

Budget: release 10 min, debug 2 min. Measured at `b8bba37a2`: release 13720 runs in 126 s (inside
the `whole_groups` filter run, `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 1 --release -p rexx-exec
--test concurrency_tests whole_groups`, exit 0, 14 passed, 5:20 wall); debug at `902de404e` 1792
runs in 12 s (`REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec --test concurrency_tests
sim_gate::`, exit 0, 8 passed).

### Oracle differences at the close run

Reported, not failed (`/tmp/claude-1000/p61/t10/gate-final.txt`), runs per part: every seed of the
parts whose normal-mode run is listed in `DIFFERING` (70 each: Section1, TRACE_TraceObject whole and
derived, TRACE, RAISE whole and derived, METHOD whole, rest and derived, CONSTANT whole and rest,
ATTRIBUTE whole and derived, RexxContext whole and rest, Object whole and rest, Method whole and
rest, Message whole and derived, Class whole, STREAM whole); SysSleep whole 61 and derived 55 (the
exempt TEST_SLEEP_DURATION); REPLY derived 29 and whole 14 (assertion counts after the replying
tests, P41/P86); MutexSemaphore single 20 and `message_notify.rex` 14 (the exempt deadlocks);
`guard_on_lock_passes_between_activities.rex` 16; Ticker derived 9 and whole 8;
`context_moved_by_reply.rex` 8; `main_ends_in_pinned_yield.rex` 3;
`ticker_fires_on_its_replied_activity.rex` 2; MethodArgs derived 1; STREAM derived 1.

### `whole_groups`

At `b8bba37a2`, the command above: exit 0, 14 passed. At `902de404e` it was red on TRACE whole (both
modes): Task 6 (`f062f791e`, then `072e536ac`) had made the `?` tests and TEST_TRACE_NUMERIC_DEBUG pass; the two keys were
rewritten (report, "whole_groups: TRACE whole").

## Task 11

Mutants judged at `8910306e5`. Report: `.superpowers/sdd/2026-10-07-phase-6-1/task-11-report.md`.
Tests: `src/scheduler/tests/mutants.rs` (M9, M10, M11, M12, in the simulation mode) and
`tests/concurrency_tests.rs` `measured::a_slice_deferred_before_a_pinned_park_stays_with_its_activity`
(M7, `--features pinning`). Commands: `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec
--no-fail-fast --lib scheduler::tests::mutants`, and `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4
-p rexx-exec --features pinning --no-fail-fast --test concurrency_tests --
measured::a_slice_deferred_before sim_gate::`.

| mutant | site | verdict | judge |
|---|---|---|---|
| M6 | `switch_to` keeps `SLICE` | equivalent: a pending `SLICE` at `switch_to` only moves the incoming activity's first yield to one the timer could produce (scout C); green under every judge | none |
| M7 | `switch_to` keeps `slice_deferred` | killed: deferred `[SORTWITH, SortComparator]` 1 where 2 | M7 pinning test |
| M9 | `cancel_wait` keeps `when_parked` | killed: rc 120, a ready activity holding a park reason | `a_failed_guard_when_leaves_no_park_for_a_halt_to_end` |
| M10 | `cancel_wait` keeps the guard-queue entry | killed: 98.905 deadlocks, rc 120 | `a_failed_guard_lock_wait_leaves_no_place_in_the_queue` |
| M11 | `cancel_wait` keeps the sleeper | killed: `result The NIL object after 0` | `a_failed_pinned_sleep_leaves_no_deadline_to_end_a_later_wait`, `the_seeded_gate` |
| M12 | `object_roots` drops `failed_sends` | killed: 1 write to a dead handle | `a_dropped_failed_send_is_written_while_alive` |

Found and fixed: `concurrency_tests` did not compile under `--features pinning` since Task 9
(`pinning_table`, E0507).
