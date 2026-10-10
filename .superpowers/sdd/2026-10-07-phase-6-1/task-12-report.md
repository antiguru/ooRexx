# Phase 6.1 Task 12 report: close

Status: DONE_WITH_CONCERNS. Every step is done and G1-G9, `whole_groups`, the seeded gate (both
builds) and TSan are green at `97cb37712`. `parse` stays +1.1979% Ir against base61 after a
widened perf round, which is over the +0.5% budget and goes to Moritz.

## Commits

* `6a87cd616` dirread bench program (`callgrind.sh`, the criterion list, the suite axes, README).
* `e075f1d36` the determinism self-test skips `clock=real` rows (R6), with a test of the row filter.
* `a68c735fd` every GUARD group test passes (stale gate-only list; bisected to `59eb57f37`).
* `0f159f674` SysSleep sleeps at least one microsecond (TEST_SLEEP_DURATION regression).
* `84aa32a62` queue files (31), plus `2026-10-10-whole-groups-differing-rows-that-agree.md` in the
  close commit.
* `01c7d7a85`, `fce0bd4c1`, `23f4b609f`, `ab4bc780e` parse perf round (the last reverts the third).
* `13bbff35f` `refusal-sites.tsv` re-derived (line drift from `01c7d7a85`).
* `97cb37712` seeded-gate harness race: the shared run directory stays.
* The close commit: roadmap rows 6.1 and 9, the gate record's `## Task 12`, this report, the
  evidence directory, the workspace `tsan.sh`, the `bg/` gate logs.

## Step 1: queue

Probes ran at `6a87cd616` through `task-12-evidence/probe.sh`, each engine from a fresh empty
directory: the oracle under the standard wrapper, ours under `memcap 2G timeout 20`, with stdout, stderr
and rc kept apart. Concurrent probes ran 5 times per engine. Probe texts and both outputs are in each
new file.

| # | item (source) | outcome | evidence |
|---|---|---|---|
| 1 | Concatenation with a variable reference (brief; scout A s4) | new file `2026-10-10-variable-reference-concatenation.md` | probe at `6a87cd616`: oracle `an Arrayx`, ours `x` |
| 2 | `RAISE NOVALUE` RETURN into `CALL ON ANY` (brief; scout A s4; ledger Task 3 ruling) | new file `2026-10-10-raise-novalue-return-into-call-on-any.md` | probe: oracle `back`, ours `D=[] NOVALUE`, `back` |
| 3 | `subclass package` (brief; scout A s4) | new file `2026-10-10-subclass-package.md` | probe: oracle `1`, `3`; ours 97.1 rc 159 |
| 4 | `subclass rexxinfo` (brief; scout A s4) | new file `2026-10-10-subclass-rexxinfo.md` | probe: oracle 99.949 rc 157; ours 98.909 rc 158 |
| 5 | `.context~name` in a `Routine~new` body (brief) | existing file `2026-10-01-routine-call-context-name.md` | not re-run (named by the brief as already filed) |
| 6 | scout B `cond3` extra traceback line (brief) | new file `2026-10-10-propagated-condition-extra-traceback-line.md` | probe: ours prints `9 *-* call s`, oracle does not; rc 214 both |
| 7 | `Class~enhanced` skips `NEW` (brief; scout D) | new file `2026-10-10-class-enhanced-skips-new.md` | probes: oracle `0 Array`, `3 42 String`; ours 93.902 rc 163 |
| 8 | `'12'~translate('','')` (brief; scout D) | new file `2026-10-10-string-translate-empty-tables.md` | probe: oracle `[12]`, ours `[  ]`; BIF and MutableBuffer forms agree |
| 9 | the shrinker (brief; spec D9) | new file `2026-10-10-sim-shrinker.md` | design item, no probe |
| 10 | an oracle-judged seeded gate (brief; spec D9) | new file `2026-10-10-oracle-judged-seeded-gate.md` | design item, no probe |
| 11 | P52 lending under sim (brief) | new file `2026-10-10-sim-pool-seeded-delivery.md` | design item, no probe |
| 12 | SIGNAL ON propagation traceback line missing in a REPLY continuation (ledger l.35, l.79; Task 1 concern 1) | new file `2026-10-10-reply-propagated-condition-traceback-line.md` | probe 5 runs each, one outcome per engine: oracle has `2 *-* say 'got' o~m`, ours lacks it |
| 13 | SELECT CASE value lingers on the activation (ledger l.35, l.44) | new file `2026-10-10-select-case-value-outlives-construct.md` | no observable found; the file says so |
| 14 | REPLY trace gap (ledger l.52, l.79; TRACE TEST_TRACE_LABEL_WITH_FORWARD) | new file `2026-10-10-reply-continuation-trace-entry.md` | `p7.rex`, 5 runs each: oracle 12 `>I>`/`<I<` lines, ours 10 |
| 15 | TraceObject collector gap (ledger l.52, l.79, l.119) | existing file `2026-10-02-traceobject-variable-and-collector.md` (its second bullet) | Task 2 review's `p8.rex` at `6a87cd616`: oracle collects 15 trace objects, ours 0 and prints the trace lines |
| 16 | upstream ticket candidate: oracle DO TO/BY identity test (ledger l.79) | new file `2026-10-10-upstream-do-to-identity.md` (upstream candidate, not crate work) | probe: oracle 3 passes for a user `>` answering `1`, ours 0 |
| 17 | truth tests ignore a user STRING method (ledger l.83, l.84) | fixed by `d27a9d441` (Task 4a) | `w_ustr1.rex` at `6a87cd616`: `while 3` on both |
| 18 | WHILE blames a later-pass 34.3 on the DO line (ledger l.84) | existing file `2026-10-02-while-condition-failure-traceback.md` | `abc_while.rex` at `6a87cd616` still differs (ours line 3, oracle line 6 `end`) |
| 19 | a trapped error in a user `>` during the header's first comparison holds the TO object (ledger l.84) | new file `2026-10-10-trapped-header-comparison-holds-to-object.md` | `lkJ.rex`: oracle `uninit J` before `gc1`, ours after `gc4` |
| 20 | rooting of other `ActivationCold` fields (auto_expose owner/scope) unchecked (ledger l.101) | no probe; resolved by inspection, `5251a8984` and `108bc81fa` (Task 11b) | `Activation::object_roots` destructures `ActivationCold` and `AutoExpose` exhaustively, and `Activity::object_roots` calls it; no program probe was run |
| 21 | enhanced object name in 97.x (ledger l.101; Task 4 concern 3) | new file `2026-10-10-enhanced-object-name-in-messages.md` | `t4b.rex`: oracle `enhanced Object`, ours `an Object` |
| 22 | compiled method package in `>I>` (ledger l.101) | new file `2026-10-10-compiled-method-package-in-trace.md` | `t4c.rex`: oracle `in package "M"`, ours the program file |
| 23 | setMethod/run code cannot see caller classes (ledger l.101) | fixed by `8b95afe79` (Task 11a Step 6b) | `t4d.rex`: `U` on both at `6a87cd616`; at `8b95afe79` `U`, at its code parent `98d0d4ccd` rc 159 (each built from `git archive`, own target, `Compiling rexx-exec`); base61 97.1 on `.U` |
| 24 | Directory~setMethod extra UNKNOWN traceback line (ledger l.101) | new file `2026-10-10-directory-setmethod-unknown-traceback-line.md` | `t4e.rex`: ours adds `Compiled method "UNKNOWN" with scope "Directory"` |
| 25 | `.context~executable` in a method's internal routine (ledger l.101) | existing file `2026-10-02-frame-executable-invocation-and-native-levels.md` | `t4f.rex` at `6a87cd616`: oracle `Method`, ours `Routine` |
| 26 | a user STRING answering an Array or another object (ledger l.108) | fixed by `5bb6510af` (Task 11a Step 4) and its fix rounds | `i7a.rex` identical on both at `6a87cd616` |
| 27 | rooting harness blind to dropped pushes (ledger l.108) | new file `2026-10-10-rooting-harness-blind-to-dropped-pushes.md` | harness gap; mutation evidence in `task-4a-review.md` |
| 28 | ATTRIBUTE TESTMISPLACEDCLASSMETHOD (ledger l.119) | new file `2026-10-10-misplaced-class-attribute-error.md` | `attrcls.rex`: oracle 99.905, ours 99.937 |
| 29 | DIFFERING rows agreeing in both green runs: TIME whole/derived, CALL derived, REPLY every (ledger l.119) | new file `2026-10-10-whole-groups-differing-rows-that-agree.md` | the `whole_groups` run at `97cb37712` lists exactly these as differing and agreeing |
| 30 | CALL whole passes TEST_4 but the rest run failed it (ledger l.121) | same new file as row 29 | CALL whole and derived agree with the oracle at `97cb37712`; CALL has no rest part since Task 5 I2, so the rest run cannot be repeated |
| 31 | prologue stack frames also miss nested/newFile intermediate frames (ledger l.121) | new file `2026-10-10-prologue-frames-missing-loading-levels.md` | `frames.rex`, `frames2.rex`: oracle `METHOD NEW` and `ROUTINE mid2.cls` frames, ours neither |
| 32 | pending UNINIT never finalized mid-run (ledger l.127) | fixed by `595ba08ef` and `be020a36d` (Task 11b) | `uninit.rex` at `6a87cd616`: `done 950`, peak 69 MB (scout E1 measured 356 MB at its HEAD); oracle `done 995`, 16 MB |
| 33 | Array~append quadratic (ledger l.127) | fixed for an array with no trailing empty slots by `e8a19b2e6` (Task 11a Step 7); after a sparse put, a sized `new` or `empty` it stayed quadratic, 1e5 appends 5.62-12.87 s (final review finding 3), and is fixed by the final fix's `ArraySlots` commit (`final-fix-report.md`) | `append.rex` (1e5 appends), 5 interleaved runs at `97cb37712`: 0.03 s each here, 0.01-0.02 s on the oracle (scout E2: 5.4 s) |
| 34 | COPIES fills byte by byte (ledger l.129) | fixed by `e4064dac0` (Task 11a Step 8) | `copies.rex` (loop999, N = 1e6), 5 interleaved runs at `97cb37712`: 0.43-0.44 s here (scout E2: 1.00-1.12 s), 0.76-0.77 s on the oracle |
| 35 | `=` after ITERATE indent (ledger l.135) | new file `2026-10-10-debug-equals-after-iterate-indent.md` | `e2.rex`: `3 *-*       iterate` against `3 *-*   iterate` |
| 36 | REPLY no pause (ledger l.135) and REPLY reprints the banner (ledger l.138) | new file `2026-10-10-debug-reply-pause-and-banner.md` | `k3.rex`, 5 runs each, one outcome per engine |
| 37 | labelled block DO header pause (ledger l.135) | new file `2026-10-10-debug-labelled-block-header-pause.md` | `g1.rex`: oracle pauses after `do label lbl`, ours after `nop` |
| 38 | oracle defect: `=` at a zero-pass DO pause, then END 10.1 (ledger l.135) | new file `2026-10-10-upstream-zero-pass-do-reexecute.md` (upstream candidate; Deviation 26) | `z26.rex`: oracle 10.1 rc 246, ours `end` rc 0 |
| 39 | 28.x from LEAVE/ITERATE not trappable by SIGNAL ON SYNTAX (ledger l.138) | new file `2026-10-10-leave-iterate-error-not-trappable.md` | `lv4o.rex`: oracle `trapped 4` rc 0, ours 28.3 rc 228 |
| 40 | 10.x and 24.1 messages print `&n` unsubstituted (ledger l.138) | new file `2026-10-10-translation-error-inserts.md` (fix round 1); the work ruling R3 defers | `cs.rex` (24.1) and `e103.rex` (10.2) are translation errors: unsubstituted here, filled on the oracle |
| 41 | `trace c` drops the echo of a failing ADDRESS command (ledger l.138) | new file `2026-10-10-trace-c-command-echo.md` | `cc2.rex`: ours lacks `3 *-* address command 'false'` |
| 42 | `>I>` missing when EXPOSE precedes TRACE (ledger l.138) | existing file `2026-10-02-trace-entry-after-expose.md` | `k2n.rex` at `6a87cd616` still lacks `>I> Method "M"` |
| 43 | typed SIGNAL at a pause (ledger l.142) | new file `2026-10-10-debug-typed-signal.md` | `c1sig.rex`: oracle `at lbl 7`, ours `handler sigl 1`, `after` |
| 44 | typed CALL SIGL (ledger l.142) | new file `2026-10-10-debug-typed-call-sigl.md` | `c2call.rex`: `in sub 3` against `in sub 1` |
| 45 | DO header NOTREADY order under debug (ledger l.142) | new file `2026-10-10-debug-do-header-notready-order.md` | `dh1.rex`: banner and pause order differ |
| 46 | LEAVE-ends-program-at-pause path (`close_flat_top`) unwitnessed (ledger l.142) | fixed by `072e536ac` (Task 6 last touch) | witness `corpus/lang/debug_handler_exit_at_iterate.rex` (the re-review's `x8`); `x8.rex` identical on both at `6a87cd616` |
| 47 | typed untrapped RAISE USER then more typed lines (ledger l.144) | new file `2026-10-10-debug-typed-raise-untrapped.md` | `t4.rex`: oracle runs no further typed line; ours prints `mid`, 40.1, `mid2` |
| 48 | method/function `+++` header class (ledger l.144) | fixed by `072e536ac` (banner per activation) | `x4.rex`, `x8.rex`, `x9.rex`, `c2proc.rex` identical on both at `6a87cd616` |
| 49 | rexxcps +0.09% between Task 6 rounds 1 and 2 unexplained (ledger l.144) | new file `2026-10-10-rexxcps-task-6-round-2-delta.md` (fix round 1) | rexxcps is +0.2694% against base61 at `ab4bc780e`, inside the budget; the round-2 delta itself stays unattributed |
| 50 | commit `dirread.rex` as a bench program (ledger l.151) | done, `6a87cd616` | `callgrind.sh`, the criterion list and the suite's axis list name it |


## Step 2: roadmap

* Row 6.1: the parenthetical names random bounded preemption as the main policy and PCT as an opt-in
  policy, judged by invariants with oracle differences reported (R1, R5). The row is marked closed
  with the gate record, except for criterion 7 (parse).
* Row 9: names `NEW` on subclasses of String, Stem, Method, Routine and Message as its work (R2, D4),
  starting from `scout-d-prototype.patch`. It absorbs the refusals naming Phase 9, listed by
  `grep -rn '"Phase 9"' rust/crates/*/src` (`Loud::native_method`, rexx-api
  `layout::REFUSING_MEMBERS`' Phase 9 members) and `grep -n 'OWNER: Phase 9'
  docs/superpowers/plans/phase-4-exclusions.txt`. `refusal-sites.tsv` carries no owner column, and
  no row names a phase. No refusal text changed.

## Step 3: whole groups

The gate record's "Criterion 5" table has every criterion 5 row. TIME (TEST_4, 5, 10, 11 and the
`_R` pair), CALL TEST_4, RexxContext TESTCONDITION01, GUARD TEST_WHEN_USE_LOCAL_NO_WAIT, MethodArgs,
DateTime, Class TEST_CLASS_DEFINE, scout A's parse-error rows and the TRACE `?` rows pass. Three rows
fail, each with a cause outside 6.1: the TraceObject collector (two tests), the REPLY-continuation
trace entry (TRACE_LABEL_WITH_FORWARD) and Method whole's Phase 9 refusal.

## Step 4: gates, perf, TSan, seeded gate

The gate record's `## Task 12` has every figure. In summary:

* Gates G1-G9 at `97cb37712`: all exit 0. G4 had 3141 passed, G6 3145 passed, 0 failed, 0 P48
  reruns.
* Reds on the way, each messaged to the controller before any change:
  * `6a87cd616`: the GUARD list, the SysSleep duration test and the self-test on a `clock=real` row.
  * `f85ea20cd`: `refusal_sites` line drift from my `01c7d7a85`, and a harness ENOENT race in the
    seeded gate.
  * Each has its own fix commit.
* TSan: Phase 6's `tsan.sh` failed one lib test (a 1 ms sim bound that TSan's slowdown crosses) and
  wrote no race report. The workspace copy skips that test, with its reason, and gives api/lib/int
  exit 0 and `no tsan log`.
* Seeded gate:
  * Release: 13720 runs, 0 reds, 34 exempted.
  * Debug: 1792 runs, 0 reds, 7 exempted.
  * Oracle differences are listed by part in the gate record.
  * SysSleep's sim-exempt row is gone with `0f159f674`.

### SysSleep (`0f159f674`)

The oracle's timestamp is invalidated after every instruction (`RexxActivation.cpp:647`), and
`TIME('R')` anchors elapsed time lazily at the next fresh read (`:3390-3413`, `:3438`). The crate does
the same (`run.rs` `enter_stepped_clause`, `builtin/datetime.rs` `now_base_time`). The divergence was
SysSleep's own duration. The oracle always calls `nanosleep`, even for 0 µs
(`RexxUtilCommon.cpp:1891-1922`, `SysThread.cpp:191-201`). This crate parked until now plus 10 ns, so
R, SysSleep and E could fall inside one microsecond.

Zero elapsed readings in 2000 passes:

| delay | oracle | base61 | `6a87cd616` | `0f159f674` |
|---|---:|---:|---:|---:|
| 0.00000001 | 0 | 119 | 209 | 0 |
| 0 | 0 | not run | 266 and 319 | 0 |

Witness: `sim::tests::time_e_after_any_sleep_is_never_zero`, red before and green after.

### Parse

Perf before and after the round, every program against base61 (`callgrind.sh -r 3 -j 4`, pad2 control
0.0000%):

| program | base61 Ir | before (`0f159f674`) % | after (`ab4bc780e`) % | pad2 % |
|---|---:|---:|---:|---:|
| alloc | 20345537514 | +0.0118 | -0.0318 | -0.0000 |
| alloc4c | 3140684485 | +0.0607 | -0.0654 | +0.0000 |
| arith | 11488858265 | +0.2861 | +0.2724 | -0.0000 |
| assign | 18983599254 | -3.1598 | -3.1651 | -0.0000 |
| compound | 8937498030 | -1.0053 | -1.1172 | +0.0000 |
| decloop | 2469010741 | -2.3171 | -2.2380 | +0.0000 |
| decrender | 4228326951 | -1.1070 | -1.0609 | -0.0000 |
| dirread | 2644856962 | +0.1273 | +0.1049 | +0.0000 |
| dispatch | 20809502845 | -0.5038 | -0.3597 | +0.0000 |
| dispatchclass | 15794935187 | -0.7841 | -0.6321 | +0.0000 |
| emptyloop | 7786099871 | -0.3191 | -0.6402 | +0.0000 |
| extcall | 7552641451 | +0.0418 | +0.0021 | +0.0000 |
| fibcall | 8413280378 | +0.4617 | +0.4412 | -0.0000 |
| fibfunc | 8228290764 | +0.4721 | +0.4303 | +0.0000 |
| heapshape | 2334068378 | +0.0520 | +0.0093 | +0.0000 |
| nop | 9283340655 | -1.0755 | -1.0863 | -0.0000 |
| parse | 1536336774 | +1.7815 | +1.1979 | -0.0000 |
| sayloop | 109048217 | +0.1425 | -0.4071 | +0.0001 |
| sendloop | 14013810691 | -0.7125 | -0.4984 | +0.0000 |
| startup | 58097223 | +0.2670 | +0.2680 | +0.0000 |
| strings | 17537481300 | +0.4584 | +0.3735 | -0.0000 |
| textnum | 1155145989 | -0.5233 | -0.5405 | +0.0000 |
| varlookup | 13437530297 | -2.1198 | -2.2612 | +0.0000 |
| rexxcps | 17788421071 | +0.4268 | +0.2694 | -0.0000 |
| pingguard | 1159017714 | -0.5905 | -0.7113 | +0.0000 |
| pingmsg | 1676421012 | -0.0190 | -0.0421 | +0.0000 |
| pingsem | 1181250699 | -0.4694 | -0.4863 | +0.0000 |

Lesson: `23f4b609f` was measured on four programs, and it cost fibcall, fibfunc, sendloop, dispatch
and dispatchclass up to +1.86 points, found only by the all-program table. It was reverted
(`ab4bc780e`).

## Parse attribution

`parse` is +1.7816% Ir against base61 at `6a87cd616` (budget +0.5%; layout control pad2 +0.0000%). Per task-close binary, `callgrind.sh -r 1 -p parse` (`/tmp/claude-1000/p61/t12/cg-parse`): T1 `b4d847074` +0.34, T2 `7b84c819d` +0.48, T3 `ec97e4190` +0.14, T4 `66d9eac18` +0.14, T4a `5e59d66b3` +0.60, T5 `37d874a36` +0.61, T5a `1fd487d02` +1.20, T6 `072e536ac` +1.06, T7 `b71606f16` +1.06, T11a final +1.78, T11b +1.78.

`parse.rex` runs 200,000 passes of four PARSE instructions, two assignments, a sum with three LENGTH calls and the loop step. Every binary makes 800,275 `alloc_with` calls and 12 collections (`collect_now`), so no step changes how many objects the program allocates or how often it collects. Deltas are `cgdiff.py` self Ir, B minus A.

### T4 `66d9eac18` to T5 `37d874a36` (+7,130,628 Ir, +0.46%)

| function | self Ir | what it is on parse.rex | kind |
|---|---:|---|---|
| `Interp::ops_loop_steady` | +6,998,645 | the IR driver loop. The whole delta is already present at T4a `5e59d66b3` (the T4 to T4a diff gives the same +6,998,645), which is 35 Ir per pass. No call count changes (callees, `alloc_with`, collections are identical). Task 4a's `d27a9d441` rewrote the `Op::Condition` and `Op::WhenTest` arms of `region_ops!` (inlined into the driver) to call `truth_without_conversion`, and `parse.rex` executes neither op | regression: the same ops, compiled dearer (driver codegen), as the l.103 emptyloop ruling found |
| `ProgramSource::new` / `rexx_parse::parse_program` | +2,445,047 / -2,429,553 | one-time source preparation moved between two functions by Task 5 (`59eb57f37`) | neither: a move, net +15,494 |
| `rexx_parse::parse` | +114,331 | one-time parse of the program, Task 5's wider `ParseError` | added work, one-time |

### T5 `37d874a36` to T5a `1fd487d02` (+9,148,662 Ir, +0.59%)

| function | self Ir | what it is on parse.rex | kind |
|---|---:|---|---|
| `Interp::exec_parse` | +6,000,000 | the four PARSE instructions: 30 Ir per pass, 7.5 per PARSE. Task 5a charges the bytes of each string body allocated toward the byte trigger (R10), and the strings PARSE assigns are built inside `exec_parse` | added work: the byte charge on every allocated text |
| `Interp::collect_now` | +3,144,372 | the 12 collections, 262,031 Ir more each. Task 5a's live-byte accounting in the mark/sweep (the per-survivor `held_bytes` sum hs-bisect found) | added work per collection |

### T7 `b71606f16` to T11a final (+11,078,044 Ir, +0.72%)

Split with the heapshape round's and Task 11a's recorded binaries (sha256 as in the gate record; `/tmp/claude-1000/p61/t12/cg-parse2`): T7 +1.0607, T10 `7aedf9501` +1.1693, heapshape round 1 `5214a2089` +1.7814, T11 `fe66504f3` +1.7814, heapshape fix `cc21b5ae3` +1.7814, 11a Step 6b `849f3dfb0` +1.7814, 11a final +1.7814, `0f159f674` +1.7816. Tasks 11, 11a and 11b add nothing to parse.

| step | function | self Ir | what it is on parse.rex | kind |
|---|---|---:|---|---|
| T7 to T10 (Tasks 8-10) | `Interp::alloc_with` | +1,666,002 | 800,275 allocations, 2.08 Ir more each. Per source line the cost moves with the code: the heap's slot write (`rexx-core/src/heap.rs:430` at T7, `:435` at T10, source unchanged apart from an accessor added above it) gains +734,739. The only commit between the two that touches the inlined allocation path is `b7a050d6b` (Task 8), which put `sim_declines_collection` into `collect_if_due` on the due branch. That branch runs 12 times here, so the per-allocation cost is code generation, not the new check | regression: the same allocation, compiled dearer |
| T10 to heapshape round 1 | `Interp::collect_now` | +9,404,548 | the 12 collections, 783,712 Ir more each. Per line: `Body::held_bytes` (`rexx-core/src/body.rs:1031`) +5,474,382 and the sweep's free-slot write gains. Round 1 keeps live bytes as a running figure, so the sweep reads every freed body's bytes. `parse.rex` frees most of what it allocates, so its cost grows with garbage, as the round's own record says for rexxcps | added work: a byte read per freed object |
| T7 to T11a | `hash::owns` / `hash::class_is_one_of` | -12,568 / +11,344 | Task 11a Step 5's wrong-type check, 72 calls each, at bootstrap | neither: a rename, net -1,224 |

### Summary

| step | Ir | kind |
|---|---:|---|
| Task 4a, driver codegen (`ops_loop_steady`, 35 Ir per pass) | +6,998,645 | regression (codegen) |
| Task 5a, byte charge in `exec_parse` (7.5 Ir per PARSE) | +6,000,000 | added work |
| Task 5a, live-byte accounting in `collect_now` | +3,144,372 | added work |
| Tasks 8-10, `alloc_with` codegen (2.08 Ir per allocation) | +1,666,002 | regression (codegen) |
| heapshape round 1, freed-body bytes in the sweep | +9,404,548 | added work |
| everything else (Tasks 1-3 and 6 net, Task 5's one-time parse, the T4a-T5 move) | +157,162 | of a total +27,370,729 |

## Parse perf round

Controller rulings `b565088aa` and `b45885004` widened the round to the 6.1-introduced costs on parse's own path, with items (1) and (4) accepted:

| commit | change | parse vs base61 | other programs |
|---|---|---:|---|
| `0f159f674` (before) | | +1.7816% | |
| `01c7d7a85` | `traced_mode` reads `TraceCache`'s paused flag. Task 1 (`16b67c6cc`) moved `debug_pause` onto the activation, which put an Option deref and a bit test on every PARSE target and template step (exec_parse +4.0M, next_template +3.2M) | +1.3000% | rexxcps +0.2948, strings +0.4413 |
| `fce0bd4c1` | the sweep matches a freed body once, its Class arm and held bytes together, with `Bytes::heap_len`. The running figure is exact and the debug assertion holds | +1.1979% | rexxcps +0.2694, heapshape +0.0093 |
| `23f4b609f` | the cache stored the traced setting itself | +0.9638% | fibcall +1.38, fibfunc +1.48, sendloop +1.36, dispatch +0.89, dispatchclass +0.68: over budget |
| `ab4bc780e` | revert of `23f4b609f` | +1.1979% | every other program inside +0.5% |

Tried and not committed: summing the smaller of the survivors and the dead in the sweep (parse
+1.61%). Ceiling: a scratch build with the freed-bytes read and the PARSE charge deleted, which is not
a valid fix, gave parse +0.7276%.

Residual at `ab4bc780e`, +18.4M self Ir:

| function | Ir | source |
|---|---:|---|
| `collect_now` | about +11.0M | the freed-body byte accounting, about 6 Ir per freed object for the body discriminant plus the match and length (heapshape round 1 on Task 5a) |
| `ops_loop_steady` | +2.0M | accepted (1) |
| `alloc_with` | +1.67M | accepted (4) |
| `exec_parse` | +4.0M | Task 5a's length test per piece (+1.6M) and the rest of the code moved since base61 |

Reaching +0.5% needs a design change to the freed accounting (a per-object bit), which is outside a
close round.

## Wall clock

The gate record has the table. pingsem: a pad alone moves the same code's cycles by 4.08 points at
equal instructions and context switches, as large as its drift, so the drift is not attributable to
code. pingguard: the largest pad move is 2.78 points, below its +4.53% drift, so its drift is
unattributed. emptyloop:
cycles are +4.4 to 4.6% on `close` and both of its pads, against +0.07% for base61's pad2. Its
instructions are 0.63% fewer and its context switches equal, so layout alone does not move it. It is
unattributed beyond the loop's own cycles, as at Task 2.

## Concerns

* parse over budget at this task's close (above). Parse round 2 (`23d78ec1b`, `parse-round2-report.md`) brought it to +0.4824%, so criterion 7 holds.
* The parse-round commits were first measured on a subset of programs. `23f4b609f` shipped and was
  reverted after the all-program table.
* `01c7d7a85` left `refusal-sites.tsv` stale, and G4/G6 caught it. Per-commit checks should include
  `refusal_sites` whenever a file holding a constructor changes.
* The ceiling and parse-step binaries ran on scratch builds. Each build had its own target directory
  and printed a `Compiling rexx-exec` line, but the step labels of the hsr and t11a binaries come from
  their gate-record sha256s.
* The first TSan attempt ran with cargo's default jobs. It was stopped and rerun with
  `CARGO_BUILD_JOBS=4`.
* `phase-4-exclusions.txt`'s row "Method~new AND Routine~new REFUSE ANY THIRD ARGUMENT" is false at
  this HEAD (fixed by `98caea3de`). It is not edited here; the final review should delete it.

## For Moritz

Pointers are `progress.md` line numbers (`l.N`) at `5c83d9250`, or files.

### Open questions

* Resolved after this report: parse round 2 (`23d78ec1b`) sums survivor bytes in the mark loop instead of reading every freed body, and parse is +0.4824% Ir against base61 (`parse-round2-report.md`, gate record `## Parse round 2`). Criterion 7 holds.
* emptyloop's wall clock is +4.6% in cycles at -0.63% instructions, and layout pads do not move it (Wall clock). Task 2 accepted it. Re-accept, or open an item.
* The upstream candidates `2026-10-10-upstream-do-to-identity.md` and `2026-10-10-upstream-zero-pass-do-reexecute.md`: file them on SourceForge or not.
* Mapping row 40: 10.x and 24.1 messages print `&n` at translation time. Ruling R3 keeps parse-error inserts out of 6.1, so no file was written. Say if you want one.
* `phase-4-exclusions.txt`'s row "Method~new AND Routine~new REFUSE ANY THIRD ARGUMENT" (re-homed to Phase 9) is false at `6a87cd616`: `.Method~new('mm', 'return 43', .context~package)` answers `a Method` on both (`mnew3.rex`), since Task 11a Step 6 (`98caea3de`). Not edited here: the brief names no exclusions edit. Roadmap row 9 still lists it among the Phase 9 rows the grep finds.
* `2026-10-10-oracle-judged-seeded-gate.md`: R1 frames the end state as a stand-alone crate; close the item as not wanted, or keep it.

### Rulings made on your behalf (controller)

Task 12's own, from `progress.md` at `9512f083c`:

* l.256: the GUARD pass list rewritten after a bisect (all nine flips at `59eb57f37`, each oracle 2 of 2); the self-test skips `clock=real` rows (R6), which narrows its row pool; no perf while a gate runs.
* l.258-260: SysSleep's zero elapsed reading is a 6.1 regression to fix; every SysSleep delay, 0 included, parks at least 1 µs (`0f159f674`), and its sim-exempt row goes.
* l.257: the parse overrun blocks the close, attributed per step first; superseded by l.263 and l.268, which let the close proceed with the overrun open.
* l.263, l.265: the parse round runs in Task 12, widened to next_template, exec_parse and collect_now's remainder; items (1) driver codegen and (4) allocation codegen stay accepted under the l.103 precedent.
* l.268: parse's residual goes to you at close as an open overrun; no "holds bytes" redesign in 6.1.
* l.269: `23f4b609f` reverted (`ab4bc780e`) after the all-program table put call paths over budget.
* l.270: `refusal-sites.tsv` refreshed as its own commit (`13bbff35f`), line numbers only.
* l.271: the seeded-gate harness keeps its shared run directory (`97cb37712`), which leaves an empty directory behind in the target's tmp.
* l.273: `quick_native_calls_at_the_smallest_bound_run_alike` is excluded under TSan only, through the 6.1 copy of `tsan.sh`; one sim-bound native test goes unchecked under TSan.

Earlier tasks', at `5c83d9250`:

* l.30: per-task check without full gates; `whole_groups` only at Tasks 3, 5, 10, 12.
* l.31: scheduler tests' inline programs stay out of the seeded gate.
* l.34: `size_of::<Activation>()` pinned at 472 (`activation.rs:502`, since `d27a9d441`), not 512 (R4's intent read as "no growth").
* l.38, l.51: false doc sentences fixed inside fix rounds.
* l.50: criterion 5 row reasons written into the gate record at Task 2.
* l.52: REPLY trace gap and TraceObject collector gap not 6.1 items (now `2026-10-10-reply-continuation-trace-entry.md`, `2026-10-02-traceobject-variable-and-collector.md`).
* l.62: object control values and DO OVER `.context~package~local` made IMPLEMENT (a reachable GUARD).
* l.63: TEST_TRACEOBJECT_COLLECTOR failure bisected in Task 3's fix round.
* l.64: CALL ON ANY known-gap row; two oracle crashes into `oracle-crashes.txt`.
* l.68: option (B) for DO TO identity, superseded by your l.69/l.72 rulings.
* l.73: numeric TO/BY cached as one object per loop.
* l.83: truth tests ignoring STRING queued (then taken by your R9).
* l.90: a non-logical DO TO/BY answer raises 34.901.
* l.103: emptyloop +0.65% Ir from Task 4a accepted as codegen, rechecked here.
* l.107: SELECT CASE object kept alive after SELECT left (GC timing licensed).
* l.114: parse cumulative +0.61% rechecked here.
* l.126: R10 extended to MutableBuffer and Array bodies.
* l.136: Task 6's R7 branch cost accepted within R7.
* l.140: typed-line conditions other than SYNTAX ignored during a pause.
* l.156, l.158: sim refuses foreign callbacks and watches blocking commands and natives (`block=`).
* l.186: Claude-Session trailer updated to the new session.
* l.189: Task 9 closed with heapshape over budget pending the attribution.
* l.193: heapshape round 1 (running live-byte figure).
* l.194-l.200: Task 10 gate exemptions R1-R6 (DIFFERING known set, STREAM `clock=real`, child sim line, MutexSemaphore P46, `message_notify.rex`, WALL_CLOCK rerun).
* l.201: TRACE whole keys rewritten at Task 10.
* l.205: Task 10 round 2 checked by the controller without a re-review agent.
* l.215: ping* wall clock deferred to this task.
* l.218, l.222: ordering of heapshape fix round and Task 11 Minors.
* l.219: pinning-gated module fixed; per-task clippy with `pinning,sharing`.
* l.227, l.229, l.230, l.231: Routine~new package parent, first deferred, then fixed in 11a Step 6b with class lookup and a cold parent walk.
* l.228, l.235, l.240, l.242: R11 narrowed; which STRING-answer consumers take `.nil` and which refuse.
* l.236: Task 11a I1 routes.
* l.248: 11b fixes the activation-roots defect before the UNINIT drain.
* l.252: 11b fix round scope; uncapped pool growth kept.

### Parked items

* l.39: no witness for the SETLOCAL termination restore on SYNTAX unwind or at a REPLY continuation's end.
* l.40: `next_seed` walks `top_level_activation_mut` twice per RANDOM.
* l.41: `end_cold` `swap_remove(0)` plus `clear` idiom.
* l.42: INTERPRET inside a typed debug line inherits DEBUG_PAUSE (oracle false).
* l.53: `run_repeating` and `run_loop_with_header`'s non-Simple arms dead (one made `unreachable!` at l.85).
* l.54: `with_advance` recomputes `control_slot`/`shape_of` per pass.
* l.55: overlong rewrapped comment lines.
* l.58: unverified that a trapped SYNTAX inside a DO WITH header truncates temps (see `2026-10-10-trapped-header-comparison-holds-to-object.md`).
* l.65: `8f080d2fa` carries a hand-edited `refusal-sites.tsv` row never built alone.
* l.152: `impl Loud` indented inside a nested module escapes `in_impl_loud` (no such block exists).
* l.188: a truncated sim trace whose dropped tail is only preemptions or collections replays silently.
* l.207: sim-started commands sit outside the gate's deadline kill (bounded).
* l.237: 11a m1 (unreached clock store in `run_repeating`) and m2 (copy counter scope).
* l.245: one-variable PARSE VALUE/ARG over a non-string STRING answer refuses where the oracle assigns `.nil`.
* l.252: uncapped pool growth (`2026-10-10-failed-spawn-runs-inline.md`).
* `2026-10-10-uninit-drain-signal-interpret-and-trace.md` (11b re-review R1, R2).
* `2026-10-10-call-on-nostring-insert.md` (Task 11a review, ledger l.238).
* `2026-10-10-security-manager-command-rc-requires.md` (Task 11a, queued with `857a01931`).
* Ledger l.44 (`current_case_text` never cleared): `2026-10-10-select-case-value-outlives-construct.md`, mapping row 13.
* Every new queue file in the mapping table above.

## Fix round 1

Review `task-12-review.md`: spec 2 Important, 5 Minor; quality 3 Minor. Every finding is addressed.

| finding | change | commit |
|---|---|---|
| S-I1 criteria 1-4 missing | gate record `### Criteria 1-4`: closed_phases, refusal_dispositions, refusal_sites, the gated corpus `991 of 991 matching`, ir_recorded_oracle, the two REPLY crate tests, with log lines at `97cb37712`; the two concurrent scope witnesses rerun 5 times per engine, one outcome each, identical | `2db517c4b` |
| S-I2 Task 12 rulings missing | "For Moritz" lists l.256-l.273 | the commit carrying this section |
| S-m1 rows 40, 49 had no file | `2026-10-10-translation-error-inserts.md`, `2026-10-10-rexxcps-task-6-round-2-delta.md` | `7e7ba0f72` |
| S-m2 row 23 had no commit | `8b95afe79`: built with its code parent `98d0d4ccd` from `git archive` (own target, `Compiling rexx-exec`); `t4d.rex` answers `U` at `8b95afe79`, rc 159 at `98d0d4ccd` | this section |
| S-m3 row 20 "resolved" | now "no probe; resolved by inspection" | this section |
| S-m4 parked list | the two Task 11a queue files and the l.44 pointer added | this section |
| S-m5 roadmap row 9 | the third-argument item removed; the false exclusions row and the two Routine~new rows leaning on it are CLOSED with their probes rerun (2 runs per engine, alike: `a Method` / `43`; `main 16`; `new mainr`) | `12b77b215` |
| Q-m1 wall-clock wording | "not attributable to code", in the gate record and here | `2db517c4b`, this section |
| Q-m2 Class arm assumption | a comment at the arm and `debug_assert_eq!(object.body.held_bytes(), 0)` inside it: the assertion is debug-only, so release builds pay nothing for it. fmt, `clippy --workspace --all-targets` and `clippy -p rexx-exec --features pinning,sharing` clean; `cargo test -p rexx-exec --lib` 1077 passed, `-p rexx-core` green | `6d8aa2b0a` |
| Q-m3 rows 33, 34 run counts | rerun 5 interleaved times each at `97cb37712`, counts and ranges in the rows | this section |

## Fix round 2

Re-review 1 (`task-12-review.md`): four Minors, each fixed.

| finding | change | commit |
|---|---|---|
| n1 `heap.rs` comment and message | they now say `held_bytes` charges nothing for a Class body (`Body::Class` owns a separate `Vec`); fmt, `clippy -p rexx-exec --features pinning,sharing` clean, `cargo test -p rexx-exec --lib` 1077 passed | the `heap.rs` commit of this round |
| n2 report Q-m2 row | the "never collected (D59)" reason is deleted; the row gives the true one, a debug-only assertion | this section |
| n3 wall clock per program | pingsem: the pad move (4.08 points) is as large as its drift, not attributable to code. pingguard: largest pad move 2.78 points against +4.53%, unattributed. emptyloop stays unattributed. Gate record and report both | the gate-record commit of this round, this section |
| n4 ledger l.257 | added to "Rulings made on your behalf" | this section |

