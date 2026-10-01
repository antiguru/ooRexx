# Final review slice B: records and claims (0d911ab7c..70030c6ed)

Read-only review of records and claims at 70030c6ed. Ten findings (B1-B10); none is a code defect, B5 is witnessed by a run. Checked-consistent and not-reached sections at the end.

## Findings

### B1. phase-6-perf.md:447-448, superseded Task 2 figure

Sentence: "Task 2's own three padding controls put `dispatch`'s band at -3.58% (`pad2`), narrower than what a real (non-padding) code change produces."

Evidence: the record's own base wall-clock table (phase-6-perf.md:181) gives `dispatch` pad2 d% = -1.07. The -3.58% is from task-2-report.md:91, Task 2's first base measurement before ruling P8 moved the base to `1754a3b5a` and re-measured. Also "three padding controls" is wrong for wall clock: only pad2 was run under `wallclock.sh` (phase-6-perf.md:164).

Fix: delete the sentence.

### B2. phase-6-gate.md:14, "recorded without a control" contradicts the perf record

Sentence: "Wall clock, recorded without a control (P19): ..."

Evidence: phase-6-perf.md:1272-1275, the Task 11 wall run it cites, includes `base2=`, "a copy of the base binary, the identical-binary control"; the table has a `base2 d%` column. P19 forbids layout control (function order / PGO), not a measurement control. The figures themselves match (decrender +5.68, sendloop +10.14, textnum +8.79, dispatch +8.94 inside 14.91), and Task 11's callgrind medians and spreads recompute exactly from `p6-t11/cg/summary.tsv`.

Fix: minimal replacement "Wall clock, recorded only, no layout control (P19): ...", or delete "without a control".

### B3. 2026-07-27-rust-rewrite.md:332 (D-U2), stale line citations

Sentence: "Today's native call holds `Rc` clones across the C call (`dispatch/library.rs:86`, `:94`, `:155`, `:165`)".

Evidence: written at 66073d0a2/9d863ccc5 against the pre-Task-4 file. At 70030c6ed `library.rs:94` is `let mut strings = CStringPool::new();`, `:155` is a fn signature line, `:165` is `let program = self.library_code_program(code);`. The clones are now at `:86` (`Rc::clone(&binding.library)`), `:95` (`self.activity.thread.clone()`), `:157` (`.map(Rc::clone)`), `:168` (`self.activity.thread.clone()`); further `thread.clone()` sites at `:218`, `:264`. Task 4 (0a4c5f46a) moved `thread` into `Activity`, shifting the lines.

Fix: delete the parenthesised line list (the sentence stands without it), or replace with `:86`, `:95`, `:157`, `:168`.

### B4. phase-6-pinning.md:282-293 (`## Counter`, pinned-frames table), stale against Tasks 8 and 9

The section states the counter's current frame kinds and sites; it was last written at 027ef469c (Task 3) and Task 11's `## S1 close` did not touch it.

(a) Row `| Native | ... except SEND, SENDWITH, START, STARTWITH, NEW, CALL, CALLWITH and the park points RESULT, WAIT, ACQUIRE |` is false. `pinning.rs:112-116` at 70030c6ed exempts only `RESULT`, `WAIT`, `ACQUIRE` (`const PARKING`); Task 9's C1 fix (7092dc75a/9e09cbc85) dropped the other names. Fix: replace the site cell with "`Interp::invoke`'s native and implemented-external arms, except the park points `RESULT`, `WAIT`, `ACQUIRE`".

(b) `PinKind::Delegate` (pinning.rs:89, pushed at `dispatch.rs:2444` in `send_to_delegate`, added by Task 9 C1) has no row. Fix: add `| Delegate | \`Interp::send_to_delegate\` |`.

(c) Row `| TreeEval | Op::EvalExpr |` omits the argument-evaluation sites Task 8's I3 fix added: `run/call.rs:332` (`invoke_builtin_call`), `:397` (`begin_invoke_call`), `:1239` (`arguments_before_failure`). Fix: extend the cell to "`Op::EvalExpr`; a call's non-leaf argument in `run/call.rs`".

### B5. ir/drive.rs:2425-2426 (`Interp::drive` doc), "every callee" is false

Sentence: "Runs `root`'s body for the activation on top of the stack, and the body of every callee a call op of it enters, on this one Rust frame."

Evidence: a call op in a region that is not `TOP` runs its callee recursively (`drive.rs:1232-1247`: `if $top { begin_call_tree ... }` else `run_call_tree`, "its callee's body run on this Rust stack", drive.rs:1990/2105/2202). Root-body call ops reach non-`TOP` regions through `run_ops` -> `ops_loop::<false, false>` (drive.rs:2824), e.g. the body of a labelled plain `DO` (`flat_loop_start` declines it, loops.rs:1319, into `run_loop_with_header`). Run on the gate's cap-lifted binary (`p6-t11/bin/cap/rexx-run`, MAX_ACTIVATION_DEPTH 100M), `f: procedure; arg n; if n = 0 then return 'bottom'; do; r = f(n - 1); end; return r` at depth 200,000 prints `bottom` rc 0; the same with `do label blk` aborts rc 134, "has overflowed its stack". At the shipped cap of 10,000 both finish, so this is a documentation defect only.

Fix: replace with "Runs `root`'s body for the activation on top of the stack, and the body of every callee a call op of a driven region enters, on this one Rust frame." (or delete the "and the body of every callee ..." clause).

Same overbreadth, test doc: `rexx-exec/tests/concurrency_tests.rs:676-678` "calls inside a plain `DO` block run on the driver's frame" holds only for an unlabelled block (the witnesses use only unlabelled ones). Fix: "an unlabelled plain `DO` block".

### B6. run.rs:2753-2756, falsified by Task 10's plain-DO flattening

Sentence: "The boundaries the construct does owe are opened elsewhere: a plain `DO`'s header and `END` clauses in `run_loop_with_header`'s own `LoopKind::Simple` arm, and a repeating loop's clauses in `run_repeating`."

Evidence: pre-existing comment (3bea85d86, 2026-08-15). Since Task 10, an unlabelled plain `DO` is started by `flat_loop_start` -> `flat_block_start` (loops.rs:1319-1320, 1457-1476, which opens the block's own clause "as `run_loop_with_header`'s `Simple` arm runs it"); only a labelled one reaches the `Simple` arm. A repeating loop that flattens likewise opens its pass clauses through `flat_loop_step`, not `run_repeating`.

Fix: delete the sentence (the bold lead sentence and its pointer to `leave_clause_without_boundary` stand alone).

### B7. clause.rs:29-32 (`Deadline::CLAUSES_PER_CHECK` doc), describes a mechanism that does not exist

Sentence: "Clauses between two visits to [`Interp::countdown_reached`], with or without a deadline: any interpreter can receive a request or a completion, so the cold path always runs on this cadence (design section 4)."

Evidence: added by 98d9fae33 (Task 6, S0). At 70030c6ed no request or completion channel exists in `rexx-exec` (no inbox, request or completion type; `/bin/grep -a -rn -i 'inbox\|struct Request\|enum Request\|Completion\b' rust/crates/rexx-exec/src` hits only unrelated prose). `countdown_reached` (clause.rs:281-296) checks only the deadline and reloads the countdown. The "because" clause is a future mechanism stated as present.

Fix: delete ": any interpreter can receive a request or a completion, so the cold path always runs on this cadence (design section 4)", leaving "Clauses between two visits to [`Interp::countdown_reached`], with or without a deadline."

### B8. Set cardinalities in new prose (true today, against the rule)

- install.rs:2293-2294 "for `Routine~call` and the two rows beside it" (Task 9). The rows are `executable.rs:154-158` `RESUMABLE_METHODS` (`[]`, `CALL`, `CALLWITH`). Fix: "for the rows of `executable.rs`'s `RESUMABLE_METHODS`".
- eval.rs:577 "none of `function_value`'s three arms can apply to it" (Task 8, 4822d01e0). Fix: "none of `function_value`'s arms can apply to it".

Checked and not flagged: call.rs:954-958 "five pieces ... four of them ... the fifth" is pre-existing (d37641150) and still matches the five saved fields at call.rs:960-992.

### B9. phase-6-gate.md:89-90, REPLY ordering stated as a fixed oracle order

Sentence: "`REPLY` ordering: the oracle runs a `REPLY` continuation's output before the caller continues; this crate runs it after. Pre-existing."

Evidence: the oracle's order is scheduling-dependent, not "before". Run from a fresh dir, `o = .c~new; say o~m; say 'done'` with `::method m; reply 'answered'; say 'continuation'`: oracle prints `answered done continuation` 3 runs of 3, the same as this crate (head binary). With a 200,000-pass empty loop before `say 'done'` the oracle prints `continuation` before `done` in 2 runs of 3 and after it in 1; this crate prints `done` first every time. The two task-10 reviews disagree on `reply2` for the same reason (task-10-review.md:136 "identical to the oracle", task-10-rereview.md:44 "the oracle interleaves `bg` lines before `done`").

Fix: replace with "`REPLY` ordering: the oracle runs a `REPLY` continuation concurrently with its caller, so its output can come before the caller's later output; this crate runs it after the caller. Pre-existing."

### B10. phase-6-gate.md:94, "status lines verbatim" is a selection

Sentence: "Run at `6a621dbbd` by `.../p6-gates/gates.sh`; status lines verbatim:"

Evidence: the run's `status.txt` (`scratchpad/p6-t11c/gates/status.txt`) also has `started 2026-10-01T11:08:31+02:00` and five `load ...` lines between the G lines; the block omits them. Every line it does show matches, and the G4/G6 totals recount exactly (2803/0, 2805/0 from `test result` lines of `g4-test-release.txt`, `g6-test-debug.txt`).

Fix: replace "status lines verbatim" with "its result lines, verbatim".

## Checked and found consistent

- Gate record refusal commands (phase-6-gate.md:65-67) re-run at 70030c6ed: the first command's non-test hits are exactly the table's rows (lib.rs:679/688/696; the `Sys*Sem` routines; native.rs:116-120; layout.rs:384-385); `refusal-sites.tsv` count 0; the wrapped-occurrence command lists the table's files plus `run.rs`, `handle.rs` and test files, as lines 81-83 say. 1f9be8ea5..70030c6ed changes only the three docs, so the record's `1f9be8ea5` holds at 70030c6ed.
- Task 11 callgrind table: every base/head median and spread recomputed from `p6-t11/cg/summary.tsv` (exact). Binary hashes match `p6-t11/bin`. Wall table matches `wall.out`.
- Gate perf bullets (fibfunc +1.88 over, others inside; decrender/sendloop/textnum over, dispatch inside 14.91) agree with phase-6-perf.md Task 11.
- Task 10 tables (`p6-t10/cg`, `cg-fr`): fibfunc/nop medians and the spread claims recompute. Task 9 `cg-full2`, front-end `cg-r1`, Task 8 fix `cg` deltas recompute for fibcall, fibfunc, sendloop, rexxcps, dispatch, nop.
- Recursion-depth programs: `depth 9999 rc 11` for call/func/send on base and head binaries, re-run.
- Pinning `## S1 close` table: matches `p6-t11/tgt/pin/tmp/pinning-table.md`; Task 3 sums (32, 48, 38, 1478, ...) recompute from `## Measured`.
- Oracle-crashes 20 and 21: oracle rc 139 (20 after `start`, 21 and the `directory` variant with empty stdout); this crate prints `start`/`caught 11 SYNTAX`/empty line rc 0, and `1` rc 0.
- Second region expansion: the front-end paragraph (phase-6-perf.md:1005-1007) still matches `ops_loop` (two `clause_region!` expansions, drive.rs:3050 and :3180); no text claims the second expansion is gone.
- Scheduler: `Scheduler` declares only `spawn()`; no comment or doc names `run_until_park`, `stop_the_world`, `wake` etc. in code (the spec's section 5 mention is out of scope). `exec_suspends` and `ExecOutcome::Park` docs say Park is loud; nothing says it parks.
- "no bench program has a plain DO": gone from the tree.
- Ruling citations in docs (P10, P15, P16, P17, P19, P21, P22) match the ledger's text.
- Every `Type::member` named in added comments exists; `Activity::`, `ActivityRoots::`, `FrameArena::`, `Interp::` ones exist on that type.
- D-U4 figures: `GUARD` = 65,536 cells (frame.rs:50), so the 512 KiB floor holds; `unsafe_sites.rs` exists and lists `bytes.rs`/`frame.rs`.
- Em-dashes: the only one on an added line (2026-07-27-rust-rewrite.md:217) is pre-existing text on a line whose new part is the appended parenthetical.

## Not reached

- phase-6-perf.md Task 4-7 tables and Task 6's prose figures (`wall2`/`wallA` sendloop +11.15/+7.96, the `wallctl` "every program inside 4%"): their scratch dirs (`p6-t4`..`p6-t7`) no longer exist, so not recomputed.
- Task 9 bullets' per-send figures (422, -216, -154, -40 instructions per send), the 5.76 billion `cgdiff` figure, and drive.rs:2836's "5.2% fewer" (Task 8's measurement, not re-measured after the front-end round reshaped `ops_loop_steady`).
- The gate's cap-lifted timings (4,000,000-depth aborts, 300 s non-finish, 0.10/0.22/0.69 s).
- New comments outside the listed focus files were read once for false/stale/forward/cardinality; not every one was checked against its code (e.g. plan.rs:158 "the cache never replaces an entry", lib.rs:1092 "`plans` only inserts").
- pinning.md `## Counter` park-point table: sites `Interp::invoke`'s refusals now sit in `begin_invoke`/`begin_invoke_other` (dispatch.rs:2088, :2168), reached from `invoke`; judged true enough not to flag.
