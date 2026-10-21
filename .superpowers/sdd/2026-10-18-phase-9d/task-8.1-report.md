# Task 8.1 report: crashes and activation lifetime

Status: DONE_WITH_CONCERNS. Base `f1480269a`. Code `fb86ce94f` and `e909de2d4`; records in the
commit that adds this report. Records: `docs/superpowers/records/2026-10-18-phase-9d/t8.1/`.

## Items

### 2026-10-14-routine-start-call-panic.md and gate:phase-9b1-gate.md:115 (closed, `fb86ce94f`)

Part 1. `Routine~start('CALL')` reached `begin_call` on a fresh activity, which read the caller's
activation for the label callee's program and selector and for `::OPTIONS NUMERIC INHERIT`, and
panicked `a live activation`, rc 101. The three reads moved into the label arm, and the routine arm
passes the caller's settings only when an activation is running (`run/call.rs`).

Witness `corpus/lang/routine_start_call.rex`: arguments, `PARSE SOURCE` call type, the routine's
own `NUMERIC DIGITS`, a routine with no result, one that raises 40.1 (its report on stderr), and two
starts from a label. Red at base (rc 101, `probes/routine-start/routine_start_call.base.*`),
identical to the oracle on all three descriptors at head, 3 runs. Under the simulation mode
(`sim:1`, `sim:7,fifo`, `sim:3,uniform:0.2`, `sim:5,pct:3,k=40`) stdout is identical to the wall
run and stderr differs only by the `rexx-sim:` summary line.

Part 2 is not fixed. The value is not held by a call: any expression register keeps its last
value until the register is reused. `probes/routine-start/retkeep4.rex`: a native send's result
(`.array~new`), a `::routine`'s and a method's, each wrapped in a `WeakReference`, all stay
uncleared across a churn loop run inside a `CALL`; the oracle clears all three. Collection timing
is licensed (GC ordering ruling), so the observable is memory only, bounded by one value per
register. Clearing needs per-pc liveness or a clear at every clause end, both on the run path
(a conditional in `Op::Clause` costs about 0.5% on rexxcps). Proposed owner: the performance round.

### 2026-10-20-collect-stress-raise-additional-array.md (closed, `fb86ce94f`)

`eval_list` (the tree evaluator's parenthesised list) built the array and traced its `>>>` line
before rooting it; `trace i` over `raise syntax 40.4 description (7,8)` freed it. It now pushes
the array as a temp (`eval.rs`), as `build_list` already holds it in a register.

The panic hid two more, each red at base under collect-on-every-allocation (`stress.txt`):

* `reply_as_the_last_clause.rex`: a `REPLY` ending an `IF` or `SELECT` arm leaves no split, and
  `release_method_activation` held the replied value in a local while `empty_continuation` built
  the spawner's frame. The value is parked across the filing (`dispatch.rs`).
* `package_option_writes.rex`: under `::OPTIONS TRACE L`, the value a routine returns was held only
  in `ended` while its `<I<` line was delivered. `trace_invocation_exit` now takes a closure for
  that value and parks it only on the path that delivers (`run/settings.rs`, both callers).

After these the L0 subset has no panic. Two programs still diverge under the stress mode, and they
diverge identically at base (`stress.txt`): `call_site_routine_copy_lifetime.rex` (stress prints
`answered then raised Y 1 0 0` where plain prints `answered then answered 93900`) and
`call_site_fresh_routine_raises.rex` (`function 3 raised 98.941` where plain answers 3). Both are
call sites keeping a routine across a collection, Task 7.1's area. Proposed owner: Task 8.3, which
holds the call-site items. So `collect_stress` exits 101 at head as it did at base.

`uninit_drain_trace_line.rex` joins `COLLECTION_TIMED_PROGRAMS`: collecting at every allocation
runs the `UNINIT` inside the earlier clause, so it prints `0` there.

### 2026-10-19-sigterm-in-trace-delivery.md (closed, no code change)

The halt is served inside the delivery: `request_halt` queues it for the destination's activation
and the next clause boundary raises it. `probes/sigterm/s4.rex`, a destination whose `LINEOUT`
spins under `SIGNAL ON HALT`, prints `after 1` rc 0 on the oracle, base and head, 10 of 10 each
under `timeout -s TERM 1` (`s4.term-1s.txt`). The item's shape (`sig.rex`) keeps both interpreters
running (rc 137 after `-k 5`): the untrapped HALT is reported, and the report's lines go to the
same spinning destination. The oracle prints `3 *-* x = 1` on stdout first, which this crate does
not, because an untrapped failed delivery raises in the traced clause here (entry 51, Task 6.10).
Witness `signals.rs` `sigterm_reaches_a_spinning_trace_destination`, which also passes at base.

### 2026-10-19-trace-destination-recursion-oom.md (closed, `fb86ce94f`)

Each report line's delivery failed deep in the destination and left that failure's sites and
frames on the activity, where the next failure appended to them: 431 MB peak and rc 134 under the
oracle's cap at a depth of 800 (`probes/recursion-oom/d800.capped.txt`). `deliver_trace_line_as`
now copies the failure levels before a delivery (objects rooted as temps) and restores them where
the failure leaves no trace (the line goes to stdout or stderr); a failure queued for the traced
clause keeps its levels as before (`environment/route.rs`, `run.rs`). Head: 44 MB, rc 214, the
report on stdout. Witness `tests/trace_delivery_memory.rs`, run under `ulimit -v 1048576`; the
base binary aborts on the same program.

The item's `rec.rex` now ends at rc 245, 78 MB peak, but after 318 s (`rec.head-full.txt`):
each of the 11.1 report's ten thousand lines re-runs the destination's recursion to the limit, and
the unwinding builds a `StackFrame` per level (malloc-bound in a profile). This follows the
oracle's model, where `Activity::display` sends every traceback line through `traceOutput`; the
oracle segfaults first (entries 51, 52). The parent of Task 6.10 printed `after`, rc 0, because it
swallowed the failure. Making the shape fast needs either a cheaper unwinding capture or a rule
that a report stops retrying a destination that failed, which the oracle does not have.

### 2026-10-10-uninit-drain-signal-interpret-and-trace.md (closed, `e909de2d4`)

A `SIGNAL` out of an `INTERPRET` is the fragment's `RETURNED` exit (`signalTo`,
`RexxActivation.cpp:2096-2100`), so it drains the readied `UNINIT`s as a normal end does
(`run/interpret.rs`). Inline, that arm cost interpret +0.06% whole process; `e909de2d4` moves it to
a cold function.

The trace half already held at base: a delivered line's sends to the `TraceObject` are native
returns, which drain. The item's counts (`done 0` against 674 and 216) are collection timing: no
collection runs in those loops here.

Witnesses, each with a control (`probes/uninit-drain/`):

* `corpus/lang/uninit_drain_interpret_signal.rex` prints `drained at the signal 1` on both, red at
  base (`0`); `u4c.rex`, the same without the `INTERPRET`, prints `0` on both.
* `corpus/lang/uninit_drain_trace_line.rex` prints `drained at a trace line 1` on both, and at
  base; `tbc.rex`, the same without `trace a`, prints `0` on both. It traces inside the loop body
  because the oracle under-indents the clause after a completed loop (the upstream traceIndent
  defect), which the corpus would compare.

### 2026-10-19-guarded-trace-destination-spin.md (open, reported)

Measured (`probes/guarded-spin/`): `spin.rex` prints `done 1` rc 0 on the oracle 8 of 8, and head
is killed at 6 s; `m2.rex`, two started activities tracing into one guarded destination whose
`LINEOUT` calls `syssleep 0.01`, prints `lines 10 results A B` on the oracle 8 of 8, and head ends
rc 120 with `a pinned wait for a guard that only an activity pinned below it can end is not
implemented`. A trace delivery runs pinned inside the traced clause, so while it holds the
destination's guard it can neither take a pinned yield (another delivery would wait on that guard
above it) nor park. Lifting that needs trace emission to become a resumable point of the driver
rather than a nested Rust call, at every site that emits a line. Not tractable here. Proposed
owner: Phase 10, with the continuation activities of the Phase 6 design.

## Commands

All from `rust/`, `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/claude-1000/t81-target`, at
`e909de2d4`'s tree.

| command | result |
|---|---|
| `cargo fmt --all --check` (`checks-fmt.txt`) | 0 |
| `memcap 16G cargo clippy -j 4 -p rexx-exec -p rexx-parse --all-targets -- -D warnings` (`checks-clippy.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-exec --lib --test collect_stress --test signals --test trace_delivery_memory --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions` (`checks-touched.txt`) | 101: `--lib` 1169 passed; `collect_stress` 71 passed, 1 failed (the two call-site programs above); every other binary passed |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus` (`checks-corpus.txt`) | 0, 29 passed, 1 ignored |
| `memcap 16G cargo test -j 4 --release -p rexx-parse --test sourceline_oracle` (`checks-srcl.txt`) | 0 |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../t8.1/suite/table.tsv memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` (`suite/`) | 0; `diff <(sort t7.1/r3/suite/tests.tsv) <(sort t8.1/suite/tests.tsv)` empty |

## Perf

`callgrind.txt`: `bench-programs/callgrind.sh -B 0.5 -j 4 -o .../cg-9d/t8.1-run2 -p "startup
interpret parse churn sayloop" pin=.../rexx-run parent=.../cg-9d/t7.1-r4/rexx-head
head=.../cg-9d/t8.1/rexx-head`, exit 0. Own delta (head minus parent, against the pin):

| program | whole process | libc subtracted |
|---|---|---|
| startup | -0.0020 | +0.0003 |
| interpret | +0.0046 | +0.0090 |
| parse | -0.0002 | 0.0000 |
| churn | -0.0004 | 0.0000 |
| sayloop | -0.0011 | +0.0002 |

Against the pin every program stays under +0.5% (largest churn +0.1538%).

## Records

`docs/superpowers/records/2026-10-18-phase-9d/t8.1/`: `closed.tsv` (the gate line), `callgrind.txt`,
`checks-*.txt`, `stress.txt`, `suite/`, `probes/` (each probe with its `head`, `oracle` and, where it
shows red, `base` descriptors). The queued files carry their `CLOSED` lines, and the inventory rows'
last two columns read `closed:<sha>`.

## Concerns

1. `collect_stress` stays red on `call_site_routine_copy_lifetime.rex` and
   `call_site_fresh_routine_raises.rex`, identically at base: proposed for Task 8.3.
2. The guarded-destination item stays open: proposed for Phase 10.
3. `rec.rex` ends but takes 318 s: the quadratic retry follows the oracle's model.
4. Routine part 2 (registers keep their last value) is not fixed: proposed for the performance
   round.
5. The batch file asks for the inventory rows' last two columns; that is `reason` and `ruled`, so
   each row's reason text is replaced by `closed:<sha>`. Earlier closed rows set `proposed` and
   `ruled` instead.
