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
