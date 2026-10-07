# Phase 6.1: loose ends of closed phases

**Status:** agreed with Moritz 2026-10-07 after three review rounds (`.superpowers/sdd/2026-10-07-phase-6-1/spec-review-{facts,dst,plan}.md`) and his rulings R1-R4 (section 6). Roadmap row 6.1
(`docs/superpowers/plans/2026-07-27-rust-rewrite.md:658`) is the exit contract, read through this
document.

**Inputs.** Three scout reports in `.superpowers/sdd/2026-10-07-phase-6-1/`, each at `be19fd06a`:
`scout-a-report.md` (every class (b) and (c) loud refusal, run against the oracle), `scout-b-report.md`
(the silent wrong answers: activation scope, debug pauses, INTERPRET tracebacks), `scout-c-report.md`
(deterministic simulation testing). The loud census:
`.superpowers/sdd/2026-10-01-phase-6-s2-s5/loud-census.md`. Queued items named by the roadmap row:
`.superpowers/sdd/queued/<name>.md`. Oracle source paths are relative to
`/home/moritz/dev/repos/ooRexx/interpreter`.

---

## 1. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | **Activation-scoped state moves to `Activation`.** `elapsed_anchor`, `pending_elapsed_reset`, `random_seed`, `locals` (SETLOCAL), `active_condition`, `current_case_text` and `debug_pause` leave `Activity`. | The oracle keeps each per activation (scout B, field table). A probe per field shows the wrong answer (`c2`, `c5`, `int`, `reply`, `rnd`, `sl1`, `sl2`, `sl6`, `cond1`, `cond2`, `ct2`, `ct4`, `dbgcall`). REPLY moves the `Activation` box, so P89's copy lines (`ir/drive.rs:2984-2986`) go. |
| D2 | **Every class (b) and (c) refusal gets one of four dispositions**, per scout A section 2: IMPLEMENT, REHOME (to Phase 9, 10 or 11, quoting the row text that makes it that phase's), DEVIATION (owner none, with an exclusions or oracle-crashes row), GUARD (unreachable; owner none; the probes that tried are its evidence). | The roadmap row. |
| D3 | **Receiver type guards (`Loud::receiver_class`, b13) split as scout A splits them.** The guards a wrong-type native row reaches are a DEVIATION, owner none; the `Err(kind)` sites from `receiver_kind` and the "this crate did not build" sites are GUARD. | A native row on a receiver of the wrong type crashes the oracle or answers memory-dependent garbage (scout A b13: SIGSEGV 5 of 5 for List ITEMS, Supplier ITEM, Package NAME, VariableReference NAME, StackFrame NAME, Routine CALL and a Supplier subclass; Array and Queue ITEMS answer an address, Table ITEMS `0`, Method SCOPE empty, Class ID 91.999). A refusal is the stand-in. The crashing rows go in `corpus/oracle-crashes.txt`; the others in one deviation row. |
| D4 | **`setMethod`/`unsetMethod` OBJECT scope and EXPOSE on a non-instance (b14)**, blocked today by `NEW` on subclasses of String, Stem, Method, Routine and Message. *(Ruling R2.)* | Rehoming to Phase 9 has no true reason: no ooTest file subclasses those classes (facts review B2), so Phase 9 could close with b14 still refusing. |
| D5 | **Parse errors report as SYNTAX conditions everywhere**: a main program (c8), a loaded file (b3: `::REQUIRES`, `Package~new`, an external call, `Method/Routine~newFile`) and an INTERPRET fragment, each with the oracle's traceback line; c8's existing witness is `corpus/errors/parse-errors.tsv` through `rexxc`. Inserts: ruling R3. | One `ParseError`-to-condition path serves all three. The oracle raises the error in the failing source with its clause as the traceback line (`LanguageParser.cpp:866-876`) and fills inserts at the raising site (`:4129-4171`). Roadmap row 3 records message substitutions as deliberately not reproduced (2026-07-28 scope decision, `2026-07-27-rust-rewrite.md:654`); filling any reverses part of that ruling. |
| D6 | **Debug pauses follow the oracle's per-instruction rule**, and pause input is read from `.DebugInput`. | Scout B items 2 and 3: the pause is missing after NUMERIC, TRACE, ADDRESS, DROP, PARSE PULL, a CALL's return and INTERPRET, and present after THEN where the oracle has none. `Activity::traceInput` sends `LINEIN` to `.local~DEBUGINPUT` (`Activity.cpp:3240-3263`). |
| D7 | **A deterministic simulation mode, `REXX_SWITCH_MODE=sim:SEED[,policy][,knobs]`, and a seeded gate judged by the crate's own invariants.** Oracle differences under a seed are reported, never gating. | Moritz, 2026-10-07 (queued `2026-10-07-seeded-random-switch-mode`; ruling on scope: "Sim and invariant, but don't block on the Oracle being absolutely equivalent. Treat it as a tool for as long as we're building the implementation"). The Phase 6 close saw a release-only failure that wall-clock inputs made unreplayable. Section 4. |
| D8 | **`closed_phases` polices Phase 5**, its scan widens to the owner tables in `tests/`, and a structural test ties every ownerless `Loud` constructor to a committed disposition record. | The roadmap row. Scout A section 3: `tests/owners.rs`, `tests/assertions.rs`'s EXEMPT rows, `bif_assertions.rs` and `keyword_assertions.rs` name Phase 5 and are not scanned today. |
| D9 | **Out of scope:** the silent divergences scout A met (its section 4) and scout B's `cond3` traceback, each queued as its own item by T9; the shrinker and an oracle-judged seeded gate, queued; the tree-walker (stays on `spike/tree-walker`, Moritz 2026-10-07); `RXSIODTR`. | Not refusals and not named by the row. |

---

## 2. Activation scope (D1)

The oracle's rule (scout B, "Oracle rule"): settings live in `ActivationSettings`. An internal call and
`CALL ON` copy the caller's settings in and nothing back; INTERPRET shares them (copy in, copy back);
a method, routine, external program or started activity starts fresh. `RANDOM` and `SETLOCAL` act on
the nearest top-level activation (`isInternalLevelCall`, `RexxActivation.cpp:3455-3460`;
`pushEnvironment`, `:4640-4657`). An activation ending with a non-empty SETLOCAL list restores its
oldest entry (`:1494-1500`).

| field | program, routine, method, external, started | internal call, `CALL ON` | INTERPRET, typed debug line | REPLY continuation |
|---|---|---|---|---|
| `elapsed_anchor`, `pending_elapsed_reset` | fresh | copied from the caller, with the caller's stale `cached_clock` | shared | moves with the box |
| `random_seed` | own, seeded from the activity's generator on first use (the oracle seeds at creation, `RexxActivation.cpp:174`, `:312`; lazy seeding is unobservable, since the activity seed is clock- and pid-derived) | none: walks to the nearest top-level activation | shared | moves |
| `locals` | own list; restored at the activation's end | none: walks to the nearest top-level activation | shared | moves; restored at the continuation's end |
| `active_condition` | none | copied; a `CALL ON` handler gets the trapped condition | shared | moves |
| `current_case_text` | none | none | shared | moves |
| `debug_pause` | false | false | true for the typed line's duration | n/a |

`size_of::<Activation>() == 512` holds (`activation.rs:497`). Adding an `i64` anchor gives 520 and an
`i64` plus a cold box 528 (plan review I1, a `cargo check` with the probe fields). So T1 shrinks what
it adds: `cached_clock: Option<i64>` becomes an `i64` with a sentinel (saves 8), the flags become bits
in `ActivationFlags`, the propagated condition reuses the existing `condition: Option<TrappedCondition>`
field where T1 shows them equivalent, and the seed and SETLOCAL list go in one cold box allocated on
first use. Ruling R4 keeps 512: the existing `condition` field joins the cold box.

Witnesses: each scout B probe, rewritten as a predicate where it prints a time (as `c2` is), becomes a
corpus program where it is single-activity, and a crate test where it is concurrent (`reply`, `sl2`).
`dbgcall` needs T5's CALL-return pause, so it is T5's witness; T1's `debug_pause` witness has no
flowed clause. `scheduler/tests.rs`'s
`a_reply_continuation_keeps_the_elapsed_clock_and_the_random_seed` is rewritten to the oracle's rule.
`sl3` aborts the oracle (`free(): invalid pointer`, 5 of 5 runs): it is `oracle-crashes.txt` entry 10b
(one restore per process survives) reached through the termination restore; T1 extends 10b with that
route. Every SETLOCAL witness restores at most once per process, termination restores included.

## 3. Refusals (D2-D5, D8)

Dispositions from scout A section 2, by group:

| disposition | groups |
|---|---|
| IMPLEMENT | c1 DO/LOOP `COUNTER` on every loop kind, `DO WITH ... OVER`, `DO x OVER stem.` (order licensed by deviation 1, `phase-4-exclusions.txt:844`); c2 `USE ARG` into a message or bracket term (PARSE's `assign_expr_target` arm); c3 `.context~condition` (`Interp::condition_copy`); c4 `CONDITION('D')` after `RAISE NOVALUE` answers `''`; c5, c6 `::ATTRIBUTE` and `DELEGATE` on a stem or compound name; c8 and b3 parse errors (D5); b1 `OPTIONS` (evaluate and trace the expression); b2 `USE LOCAL` as a method's first instruction; b4 a method source that is neither string nor array (93.961 or 93.974); b5 a body for every Method object this crate hands out (directive, constant, attribute, delegate, native row), so `setMethod`, `run`, `enhanced` and `define` accept them; b6 a class method from source text; b7, b8 `.context~executable` in one-off methods and `Routine~new` bodies; b10 objects as DO header values, control variables, `FORWARD ARGUMENTS` and `RAISE ... ADDITIONAL` (route through the operator send and `requestArray`) |
| REHOME Phase 9 | b14 (D4) |
| DEVIATION, owner none | b13 receiver type guards (D3) |
| GUARD, owner none | c7 `Refused::Raised`; b9 the remaining `context.rs` routine sites; b11 DO OVER an interpreter directory; b12 operators on objects (198 probes, none refused); b15 `.Class` Setup methods; b16 `.STREAM` before bootstrap; b17 embedded `.orx` parse; b18 the owed Phase 5 placeholder |

Each IMPLEMENT group gets a corpus program per scout A probe, agreeing with the oracle, and the
refusal's constructor is deleted with its pins (scout A section 3 lists them). A GUARD keeps its
constructor with owner `None` and a doc comment citing its probes. Once b5 lands, a native row on a
wrong-type receiver reaches b13's guards, which is the oracle-crash stand-in.

`closed_phases` (D8): `CLOSED` gains `"Phase 5"`; the debt paragraph (`closed_phases.rs:31-38`) goes;
the scan covers `src/` and the owner tables in `tests/` (`owners.rs`, `assertions.rs`'s EXEMPT rows,
`bif_assertions.rs`, `keyword_assertions.rs`). The `Literals` EXEMPT rows naming Phase 5
(`assertions.rs:185-281`, `runDynamicSource`) get a disposition like a refusal: T6 probes whether the
interpreter runs them outside the harness; if it does, the harness gap is fixed, else they are re-homed
with a true reason. `phase-4-exclusions.txt:530` is rewritten when b3 lands. The queued gaps NC-h and
NC-i (`2026-09-29-closed-phases-owner-gaps`) are closed.

**The disposition test.** A test beside `closed_phases` enumerates every `Loud` constructor whose owner
is `None` and requires each to have a row in a committed table, `corpus/refusal-dispositions.tsv`:
constructor, GUARD or DEVIATION, and the record it cites (an `oracle-crashes.txt` entry, an exclusions
row, or probe names). It fails on an unlisted constructor and on a row whose constructor no longer
exists. Scout A's `sites.py` and `classify.py` stay a report: their class tables are hand-written, so
they cannot decide the criterion.

## 4. Deterministic simulation (D7)

From scout C, revised by the DST review (`spec-review-dst.md`) and Moritz's scope ruling (D7).

**Mode.** `SwitchMode::Sim(SimConfig)` beside `AtClause` and `EveryOpportunity`, reached only through
the existing switch-mode arm (`scheduler.rs:1990-1999`), so the default mode gains no per-clause
branch. An in-crate PRNG (splitmix64 seeding xoshiro256\*\*). The seed derives one independent stream
per decision kind (schedule, order, clock, collection, children), so turning a knob on or off does not
re-roll the schedule.

**Nondeterminism under one seed** (scout C section 1):

| source | under sim |
|---|---|
| preemption point | the policy decides at each contended clause boundary (another activity ready) |
| which ready activity runs | FIFO, except that among the activities one event readied (a post, a release) the order is a seeded pick: the oracle's OS threads reach its kernel lock in any order |
| sleeps, timers, semaphore timeouts, `TIME`, `DATE`, `time('e')` | one clock seam; virtual time starts at a seeded fixed epoch, advances a seeded quantum per clause and jumps to the next deadline when every activity waits; the timer thread arms no slices. Knobs: `clock=midnight`, `clock=real` |
| native-call pool | bound 0: every native call, command and stdin read runs on the baton; P52 lending is not exercised |
| `RANDOM` without a seed | drawn from the PRNG |
| collection timing | `gc=q` collects at an allocation with probability q, folded into the existing `stress_collect` test (`lib.rs:2645`) |
| HALT | `halt@K`; real signals stay real |
| wait failure | `fail=wait:K` fails the K-th pinned wait with 11.1; never in a run compared with the oracle |
| file system, environment, `TZ`, child processes, run deadline | real; a child `rexx` gets a seed from the children stream; `SysFileTree` order is the file system's |
| named semaphores, rxapi, a wait for a post only a native's own thread or a command's peer can deliver | refused loudly in sim |

The quantum's range is sized from the oracle's measured per-clause time, and stated in the plan with
its measurement.

**Policies.** Random bounded preemption, not PCT: PCT's probability bound needs random priorities,
which would break the oracle's FIFO order, so no bound is claimed. `pre:d` takes d preemptions at
contended steps chosen uniformly among the first `k` contended steps (`pre:d,k=N`, k part of the
policy string); `uniform:p` preempts each contended step with probability p; `uniform:1` is every
opportunity with the virtual clock. Every policy keeps a fairness floor: a forced preemption after F
contended clauses, F sized to the oracle's 24 ms slice at its measured clause rate, so a polling
activity is never starved into a hang the oracle cannot produce.

**Invariants.** In sim the scheduler checks itself at every switch: each activity is in exactly one
of running, ready, parked or finished; the baton has one holder; every guard holder and waiter is
consistent with the guard queues; every parked activity has a wake source (a post, a deadline, a
guard, a halt). A violation is a `scheduler_inconsistency` refusal. Any inbox post other than
`Posted::Halt` is a determinism breach, refused loudly. Clippy's `disallowed_methods` forbids
`Instant::now` and `SystemTime::now` outside the clock seam and the run deadline.

**Trace and replay.** A run records its decision trace (the contended-step indices where it
preempted, each order pick, each collection point). `sim:trace=FILE` replays a trace. The replay
line printed for a failure is a harness command (test filter plus
`REXX_SIM_ONLY=group:part:seed:policy`) that reruns the same entry point, stack size and environment;
the report also prints build profile and stack size. `rexx-run` prints one `rexx-sim:` stderr line
with seed, policy, step and switch counts. The harness compares raw stderr today, so masking it is new
work, and covers the lines a child `rexx` writes into the parent's stream. A found bug lands as a crate test
with explicit switch points and an assertion that it reaches the defect's site, so a later commit that
moves the schedule fails loudly rather than passing.

**Gate.** Programs: each group part of Phase 6 criterion 1, the `corpus/phase-6.txt` programs and the
scheduler test programs. Seeds derived by rule (`seed_i = splitmix(hash(group, part), i)`), never
hand-listed; the count is set from T7's timed run of one seed per group in both builds and recorded
with the gate's wall-time budget. Policy mix: `pre:1`, `pre:2`, `pre:3`, `uniform:0.01`,
`uniform:0.2`, `uniform:1`. The gate fails on: a panic, a `scheduler_inconsistency` or other invariant
refusal, a determinism breach, a hang past the run deadline, or a design-limit refusal (inverted
pinned wait, immovable REPLY) in a program whose recorded oracle twin does not deadlock. A deadlock
the oracle reaches in the same interleaving (P46, MutexSemaphore TEST_EXCLUSION) is listed in a
committed `SIM_EXEMPT` table with its oracle evidence. A determinism self-test reruns a sample of
group parts under one seed in a fresh process and compares the decision-trace hash and the outcome.

**Oracle report.** Each seeded outcome is compared with the committed oracle outcome set of its
group part (read-only during the gate; it grows only through an explicit refresh variable). A
difference is written to the gate record, never failed on.

**The gate can fail.** Criterion 6 requires the seeded gate itself, not a crate test, to go red on
re-introduced defects: at least two scheduler fixes from the Phase 6 S2-S5 ledger reverted one at a
time, and M11 in a group-shaped program under `fail=wait:K` (its early `.nil` result must reach an
invariant or an assertion the program makes). The origin failure (REPLY under every opportunity, rc 1,
release only, `bg/b6efbfd53/logs/g4-test-release.txt:2210-2213`) is tried under `uniform:1` and the
result recorded.

**The unjudged mutants** (scout C section 4): M11 is killed by scout C's prototype (the shipped
`cancel_wait` withdraws the sleeper, `scheduler.rs:1047-1049`; deleting that line makes `m~result`
answer `.nil` early); it lands as a crate test using `fail=wait:K` that asserts the sleeper branch is
reached. M7 is witnessed through the pinning report under `every`. M9 and M10 run under `halt@K` and
`fail=wait:K`; a defect either finds is fixed in T8. M12 is judged with a dead-handle write counter
under test and a `gc=q` run. M6 is recorded as equivalent under any outcome judge.

## 5. Performance

Instrument, noise rule and wall-clock rule (±4%) as Phase 6 spec section 7, with `bench-programs/` as
committed; `callgrind.sh` is extended to the `pingpong/*` programs. Budget: a running total against
the Phase 6.1 base commit of at most +0.5% beyond the noise band on every program. D6 adds a branch on the `RegionEnd::Flowed` arm (`ir/drive.rs:1930`), which has no `$debugging` test today; any conditional there is priced at ~0.5% on rexxcps (`oorexx-per-clause-branch-costs`), the whole budget. T5 therefore finds a design with no new branch on that arm in the default mode (for example the pause decided where the existing hot-exit test already runs, or the flowed ops re-routed only while debugging); if none exists, T5 stops with the measurement. Measured at T1's
close (fibcall, fibfunc, dispatch, dispatchclass, sendloop), T4's (startup, parse: a wider
`ParseError`), T5's (rexxcps, emptyloop) and T7's (`pingpong/pingmsg`, `pingguard`, `pingsem`, alloc, alloc4c, heapshape), each
against the 6.1 base, and the cumulative figure on every program at T9. Stopping rule as Phase 6:
three rounds per stage, then the figures go to Moritz.

## 6. Rulings

Moritz, 2026-10-07, on the open items the spec reviews left:

| # | Question | Ruling |
|---|---|---|
| R1 | D7's scope | Sim and invariants; oracle differences reported, never gating. "Treat it as a tool for as long as we're building the implementation, but eventually we'll be stand alone ideally." |
| R2 | b14 cannot be rehomed with a true reason (D4) | Into 6.1 with subclass `NEW` on String, Stem, Method, Routine and Message (and VariableReference's 93.967), if a scout sizes `NEW` at M or less; else Phase 9's row is amended to name these refusals as its work. |
| R3 | Parse-error message inserts | Not in 6.1. The 2026-07-28 decision stands; 6.1 lands the traceback line for all three paths. A later reversal goes through a raising API that cannot be called without the inserts its message needs. |
| R4 | `size_of::<Activation>() == 512` | Kept. The existing `condition` field, the seed and the SETLOCAL list go in one cold box allocated on first use, measured on the T1 programs. |

Settled from the reviews: FIFO with seeded one-event order; inline pool; `fail=wait:K` ships, sim
only; seeded clock origin; trace and replay in 6.1, shrinker queued; seeds by rule; b13's
non-crashing rows are a deviation (the oracle reads a wrong-type object; the value is an accident of
layout).

## 7. Staging

| Stage | Delivers |
|---|---|
| T1 Activation scope | D1, its witnesses, P89's lines removed; perf at close. |
| T2 Loop and argument refusals | c1, c2, c3, c4, c5, c6, b1, b2, b10. |
| T3 Method objects | b4, b5, b6, b7, b8; b14 with subclass `NEW` under R2 (or the Phase 9 row amendment). |
| T4 Parse errors | c8, b3, the INTERPRET traceback line (D5, R3); perf at close. |
| T5 Debug | D6, with `dbgcall`; perf at close. |
| T6 Labels | D3, D4 and the GUARD relabels; b18; the disposition test and table; `closed_phases` with Phase 5, the widened scan, the `Literals` rows, NC-h and NC-i. |
| T7 Simulation mode | D7's mode, streams, clock seam, policies, invariants, trace and replay, knobs, report; the seed cost measurement; perf at close. |
| T8 Simulation gate | The gate, `SIM_EXEMPT`, oracle report, self-test, the gate-can-fail check; M11's witness; the mutant judgements and any fix they call for. |
| T9 Close | The queued items of D9; section 8. |

## 8. Exit criteria

1. **No refusal names Phase 5 or no phase without a disposition.** `closed_phases` (with Phase 5, over
   `src/` and the `tests/` owner tables) and the disposition test pass; every remaining `Loud`
   constructor's owner is Phase 9, 10 or 11 with a true reason, or `None` with a row in
   `corpus/refusal-dispositions.tsv`. Scout A's census scripts, re-run, report no `UNCLASSIFIED` site.
2. **Every IMPLEMENT group agrees with the oracle** through its corpus programs.
3. **D1:** the scope witnesses `c2`, `c5`, `int`, `rnd`, `sl1`, `sl2`, `sl6`, `cond1`, `cond2`, `ct2`,
   `ct4` and the `debug_pause` witness agree with the oracle (5 runs each for the concurrent ones);
   `reply` is compared as predicates (`m before` is 0; `m after` is below main's elapsed time). `sl3`
   aborts the oracle and `cond3` is out of scope.
4. **D5, D6:** scout B's `pa`, `pakinds`, `di`, `eof`, `dbgcall`, `interp`, `i2` 1-8 and `interpreply`
   probes agree, with inserts compared as the 2026-07-28 decision allows (R3).
5. **ooTest:** the whole-groups run (`concurrency_tests.rs` `whole_groups`, `REXX_CORPUS_GATE=1`) is
   re-run and its table recorded in the gate record; every row the scouts attribute to a 6.1 item (TIME
   TEST_4, TEST_10, the `_R` pair; CALL TEST_4; RexxContext TESTCONDITION01; GUARD
   TEST_WHEN_USE_LOCAL_NO_WAIT; TRACE_TraceObject `test_object_and_scope` and
   TEST_CALLER_STACK_FRAME_REPLY_START; Method whole and derived; MethodArgs TEST_REQUEST_STRING_*;
   TRACE TEST_TRACE_LABEL_WITH_FORWARD; DateTime TEST_BRUTE_FORCE; Class TEST_CLASS_DEFINE; scout A's
   parse-error rows; the TRACE `.DebugInput` rows) passes, or has a reason in the gate record naming a
   cause outside 6.1's items. TIME TEST_5 and TEST_11, unattributed today, get a recorded reason.
6. **Simulation:** the seeded gate is green in release, and in debug over its recorded subset; the
   gate went red on each re-introduced defect of section 4; the determinism self-test passes; every
   oracle difference the close run reports is listed in the gate record; every mutant of section 4 is
   judged with a run or recorded as equivalent with its argument. The gate is no evidence for P52
   lending, which bound 0 does not run.
7. **Performance** within section 5's budget.
8. **Gates** G1-G9 as at the Phase 6 close, with `REXX_CORPUS_GATE=1`, and the Phase 6 TSan run
   (its criterion 3) repeated, since T7 touches the timer and idle paths.
