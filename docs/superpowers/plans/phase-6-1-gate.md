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
Callgrind against `6860d4f80`, `-r 3`: rexxcps -0.0000
### Fix round 1

Commits `732c65404`, `58a558ff1`. At `58a558ff1`: clippy exit 0; workspace debug test exit 0
(3079 passed, 0 failed); gated corpus exit 0 (29 passed, 1 ignored; ir_recorded_oracle 21).
Callgrind against `6860d4f80`, `-r 3`: rexxcps -0.0000%, dispatch +0.0727%, a Directory-read loop
-0.0756% (+0.8013% before `#[cold]` on the receiver refusals). Details in the Task 7 report.
