# Final spec review: Phase 6.1 (standing in for Moritz)

Spec: `docs/superpowers/specs/2026-10-07-phase-6-1-loose-ends.md` at `fc04e3857`. Read against roadmap row 6.1
(`docs/superpowers/plans/2026-07-27-rust-rewrite.md:658`), the scout reports and the three reviews beside this
file, the queued items, the S2-S5 ledger (`.superpowers/sdd/2026-10-01-phase-6-s2-s5/progress.md`) and the code.
`git diff --stat be19fd06a fc04e3857` touches `.superpowers/` and `docs/` only, so every `be19fd06a` line the
spec cites still holds at `fc04e3857`. No builds, no runs: every check below is a read or a grep, quoted where a
finding rests on it. Paths under `rust/crates/rexx-exec/` unless they say otherwise.

## Verdict

**Plan after fixes.** It is the right Phase 6.1: every queued item the row names has a stage, every census (b)
and (c) group has a disposition, and R1-R4 are applied where they bind. The blocker: section 4 requires the
seeded gate to go red on re-introduced defects, and the gate's own failure list cannot see a value symptom,
which is M11's second route and the symptom of some ledger fixes. The IMPORTANT findings are stale text left
by the hand edits and the clauses of criterion 1 that no command decides. All are prose fixes; the only open input is scout D's sizing, which R2 already waits on.

| class | count |
|---|---|
| BLOCKER | 1 |
| IMPORTANT | 5 |
| MINOR | 12 |

## The earlier reviews: each BLOCKER and IMPORTANT, reflected or dropped

| review | finding | in the spec at `fc04e3857` |
|---|---|---|
| facts B1 | M11 is a mutant the prototype killed, not a defect | reflected: section 4, "The unjudged mutants" |
| facts B2 | b14's Phase 9 rehome has no true reason | R2; D4 and T3 carry it, with stale remnants (I3 below) |
| facts I1 | the 512-byte packing was false | R4; section 2 states a packing that fits (M1: one field unhomed) |
| facts I2 | b13: crashing rows vs garbage rows, and the GUARD half | reflected: D3 |
| facts I3 | no `$debugging` test on the Flowed arm | reflected: section 5 |
| facts I4 | criterion 3 named probes that cannot agree | reflected: 8.3 |
| facts I5 | the 213-site count | rightly dropped: R3 takes the inserts out |
| facts I6 | D5 reversed the 2026-07-28 decision unnamed | R3; D5 names it |
| facts I7 | `classify.py` keys drift on every edit | half: section 3 demotes the scripts; 8.1 still re-runs them (I2) |
| facts I8 | `sl3` is `oracle-crashes.txt` 10b | reflected: section 2 |
| facts I9 | criterion 5 omitted rows | reflected: 8.5 matches scout A's and scout B's attributions |
| facts I10 | `pct:d` has no k | half: the policy carries k, the gate mix does not (I4) |
| dst B1 | nothing shows the gate can go red | half: the paragraph exists; the judge cannot see M11's assertion route or a value-symptom revert (B1) |
| dst B2 | k undefined, replay key broken | as facts I10 |
| dst B3 | legal schedules red; seeds by rule; `SIM_EXEMPT`; `DIFFERING` under sim | seeds by rule and `SIM_EXEMPT` reflected; `DIFFERING` rightly dropped under R1, with B1's consequence |
| dst I1 | PCT's bound does not hold under FIFO | reflected: "Random bounded preemption, not PCT" (Q1) |
| dst I2 | no fairness floor | reflected |
| dst I3 | one PRNG stream couples knobs to the schedule | reflected: one stream per decision kind |
| dst I4 | clock origin and quantum unspecified | reflected: seeded epoch; the quantum is sized in the plan with its measurement |
| dst I5 | replay key incomplete | reflected: harness command, profile and stack size printed |
| dst I6 | no trace, replay, shrinker, regression form | trace, replay and the regression form reflected; shrinker to be queued (M3) |
| dst I7 | oracle set grows during the gate | reflected: read-only, explicit refresh variable |
| dst I8 | self-test too weak; no structural guard | reflected: fresh process, trace hash, inbox breach, clippy (M11) |
| dst I9 | FIFO-only leaves legal orders unexplored | reflected: seeded pick among the activities one event readied |
| plan B1 | the census command cannot fail on its subject | reflected as the disposition test, with the `UNCLASSIFIED` remnant (I2) |
| plan I1 | `Activation` does not fit 512 | R4 |
| plan I2 | pingpong refused by `callgrind.sh`; `gc=q`; alloc | reflected (M5 on the new instrument's spread) |
| plan I3 | perf once at the close | reflected: per stage at T1, T4, T5, T7 plus the cumulative figure |
| plan I4 | inserts had no completion check | rightly dropped: R3 |
| plan I5 | the judge had no rule for today's licences | rightly dropped under R1 for `DIFFERING` and `WALL_CLOCK` (they become reported differences); `SIM_EXEMPT` keeps the deadlock case |
| plan I6 | the outcome cache written during the gate | reflected |
| plan I7 | tier 2 never read | reflected: 8.6 lists every difference the close run reports |
| plan I8 | gate runtime unmeasured | reflected: T7's timed run sets the count |
| plan I9 | G9 and TSan dropped | reflected: 8.8 |
| plan I10 | `Literals` EXEMPT rows had no disposition | reflected: T6 |

## BLOCKER

### B1. The kill check names defects the judge cannot see

Spec, section 4 "The gate can fail": "Criterion 6 requires the seeded gate itself, not a crate test, to go red
on re-introduced defects: at least two scheduler fixes from the Phase 6 S2-S5 ledger reverted one at a time,
and M11 in a group-shaped program under `fail=wait:K` (its early `.nil` result must reach an invariant or an
assertion the program makes)."

Spec, section 4 "Gate": "The gate fails on: a panic, a `scheduler_inconsistency` or other invariant refusal, a
determinism breach, a hang past the run deadline, or a design-limit refusal (inverted pinned wait, immovable
REPLY) in a program whose recorded oracle twin does not deadlock."

Wrong: "an assertion the program makes" is not in the failure list. Under R1 an ooTest assertion failure or a
wrong printed value is an oracle difference, reported and never red, so M11's second route cannot red the gate.
The two ledger fixes are unnamed, so a planner may revert two whose only symptom is a value, and the gate stays
green on both; criterion 6 then asks for a red the judge cannot produce. The ledger holds both kinds. Symptoms
in the failure list: Task 4 I2 "pinned busy activity never preempted ... hangs", Task 8 I1 "timer single
waiter slot (one post wakes only last waiter)", Task 19 "run_started unreachable! crash", Task 21 N1 "halt
readies an already-readied timed waiter: double ready-queue entry". Symptoms outside it: Task 13 I1 "timed
EventSemaphore wait answers 1 after post+reset", Task 7 I1 "notifier failure overwritten by
restore_unwinding". Command: `grep -n -i -E 'fix|defect' .superpowers/sdd/2026-10-01-phase-6-s2-s5/progress.md
| grep -i -E 'schedul|deadlock|hang|panic|park|wake|slice|guard|sleeper|baton'`.

Fix, in section 4: (1) one more red category, "a check the program itself makes fails (a
nonzero ooTest failure or error count, or an assertion line a gate program prints) where the committed oracle
outcome set of that group part has none"; this is the program's self-check licensed by the oracle set, not
oracle equivalence, so R1 stands (Q2 asks Moritz to confirm). (2) Name the two fixes by symptom in the list:
Task 8 I1 (a parked waiter with no wake source, caught by "every parked activity has a wake source") and Task
21 N1 (an activity in `ready` twice, caught by "each activity is in exactly one of running, ready, parked or
finished"), with Task 4 I2 (a hang past the deadline) as the alternate if a revert does not apply cleanly; the
plan records the commit each revert undoes. (3) M11 names its route: the stale sleeper readies an activity that
is parked on a result, which the one-state invariant catches only if it is written as "a ready activity holds
no park reason"; T8 writes it so and asserts the branch is reached.

## IMPORTANT

### I1. Criterion 1 reaches past the row, and its reach is not decidable

Spec, 8.1: "every remaining `Loud` constructor's owner is Phase 9, 10 or 11 with a true reason, or `None` with
a row in `corpus/refusal-dispositions.tsv`."

Wrong: "every remaining" includes class (a), which the row does not name (6.1 is classes (b) and (c)); the
census records class (a) as relabelled wholesale and "not checked against any plan row" (facts B2); no stage
audits it; "a true reason" is a judgement no command decides. D2 says a REHOME quotes the row text, but nothing
says where the quote lives, so even b14's REHOME half (R2's else branch) has no check.

Fix: narrow to the row. "Every constructor the census put in (b) or (c) is deleted, or has owner `None` with a
row, or has owner Phase 9, 10 or 11 with a REHOME row quoting the roadmap text that names it." REHOME rows join
`refusal-dispositions.tsv` (constructor, REHOME, phase, the quoted text), and the disposition test checks that
each such constructor carries that owner. Class (a) stays as the census left it, its caveat cited.

### I2. Criterion 1 re-runs the scripts section 3 says cannot decide it

Spec, section 3: "Scout A's `sites.py` and `classify.py` stay a report: their class tables are hand-written, so
they cannot decide the criterion." Spec, 8.1: "Scout A's census scripts, re-run, report no `UNCLASSIFIED`
site."

Wrong: the sentences disagree, and the second is met only by hand. `PER_SITE` is keyed by `file:line`; scout A
saw two keys drift and one site fall to `UNCLASSIFIED` from three inserted lines (`scout-a-report.md:19-25`).
After T1-T8 nearly every key drifts, so the re-run reports `UNCLASSIFIED` sites until the table is
re-keyed by hand, and a re-keyed table reports zero whatever the code says. The scripts are not in the tree:
`find . -path '*/target' -prune -o -path '*/.claude' -prune -o -name classify.py -print` finds only
`docs/superpowers/records/2026-09-23-driver-spikes/*/classify.py`, a different script; the census ones live in
the `loud-census.md` appendix.

Fix: delete the sentence from criterion 1. The disposition test is the structural replacement section 3 names.

### I3. D4 states no decision; section 3 and T6 still carry the REHOME

Spec, D4: "`setMethod`/`unsetMethod` OBJECT scope and EXPOSE on a non-instance (b14), blocked today by `NEW` on
subclasses of String, Stem, Method, Routine and Message. (Ruling R2.)" Section 3 table: "REHOME Phase 9 | b14
(D4)". T3: "b14 with subclass `NEW` under R2 (or the Phase 9 row amendment)". T6: "D3, D4 and the GUARD
relabels".

Wrong: D4 is a noun phrase; the verb was lost in the hand edit. Section 3's row says REHOME unconditionally
where R2 makes it conditional on scout D's sizing. T6 lists D4 beside T3, so T3 and T6 both own it.
VariableReference's 93.967 is in R2 and in neither D4 nor T3. The Inputs paragraph does not name scout D's
report (`scout-d-new.md` is its brief, at `fc04e3857`).

Fix: D4 reads "b14 is IMPLEMENT in 6.1 with subclass `NEW` on String, Stem, Method, Routine and Message and
VariableReference's 93.967, if `scout-d-report.md` sizes `NEW` at M or less; else REHOME Phase 9 with the row
amended to name them (R2)". Section 3's row: "IMPLEMENT or REHOME per R2, decided by scout D". T6 drops D4.
Inputs names scout D's report.

### I4. `pre:d` needs k, and the gate mix gives none

Spec, section 4 Policies: "`pre:d` takes d preemptions at contended steps chosen uniformly among the first `k`
contended steps (`pre:d,k=N`, k part of the policy string)". Section 4 Gate: "Policy mix: `pre:1`, `pre:2`,
`pre:3`, `uniform:0.01`, `uniform:0.2`, `uniform:1`."

Wrong: the gate's `pre` policies carry no k, so they are undefined as the section defines them, and the replay
line has none to print. The reviews asked for k in the policy (dst B2, facts I10); the half that says where N
comes from was not carried.

Fix: "k per group part is the contended-step count of T7's calibration run with no preemption, committed
beside the seed rule in the gate table; the gate's `pre:d` reads it and the replay line prints `pre:d,k=N`."

### I5. The disposition vocabulary cannot label what T7 adds after T6

Spec, section 3: "a row in a committed table, `corpus/refusal-dispositions.tsv`: constructor, GUARD or
DEVIATION, and the record it cites". Section 4: "named semaphores, rxapi, a wait for a post only a native's own
thread or a command's peer can deliver | refused loudly in sim"; "Any inbox post other than `Posted::Halt` is a
determinism breach, refused loudly."

Wrong: T6 lands the test; T7 then adds `Loud` constructors with no owner that are reachable (so not GUARD by
D2's definition) and not oracle deviations (the default mode runs those programs). The test goes red at T7
with no row kind to write. The same holds today for the design-limit constructors the structural test will
enumerate although the row does not name them: `inverted_wait`, `immovable_reply`, "a wait that nothing left
to run can end", "an array subscript that is an empty slot" (`sed -n '400,910p' src/lib.rs | grep -o -E
'owned_message\([^,]*, None\)|^\s*None,?\s*$'`). Also: the constructors that take the owner as a parameter
(`redirection`, `environment_symbol` and the other `Some(owner)` sites) carry it at the call site, which the
test as worded ("every `Loud` constructor whose owner is `None`") does not read.

Fix: a third kind, LIMIT, citing the spec section or deviations entry that licenses it (sim refusals cite
section 4; the Phase 6 design limits cite their DEVIATIONS entry). Say the table covers every `None`-owner
constructor, class d included, that T6 writes the rows for today's and T7 for its own, and that the test reads
call sites passing `None` as well as definitions.

## MINOR

M1. Section 2's packing homes the anchor, the flags, the seed, the SETLOCAL list and `condition`;
`current_case_text` (`Option<Vec<u8>>`, 24 bytes, `src/activity.rs:158`) is in D1's list and in the table and
has no home. After `condition` (64 bytes) leaves for the box it fits inline; say which.

M2. Section 2 table, `debug_pause`, column "INTERPRET, typed debug line": "true for the typed line's duration".
INTERPRET proper shares the activation and the flag stays false; only the typed line sets it. Split the cell.

M3. D9: "the shrinker and an oracle-judged seeded gate, queued". Neither is in `.superpowers/sdd/queued/` (`ls
.superpowers/sdd/queued | grep -i -E 'shrink|oracle-judged'` prints nothing). "queued by T9", as D9's first
clause already says of the divergences.

M4. Roadmap row 6.1 still reads "deterministic simulation testing with PCT schedules, the gate instrument for
the concurrency groups", and the queued item's gate rule is "every outcome is an oracle outcome or a recorded
divergence". R1 and section 4 supersede both. T9 amends the row's parenthetical (Q1).

M5. Section 5 names D6's branch and D7's `gc=q`, not D7's other default-mode work: a per-switch pick in
`next_runnable` (scout C's own risk line) and the clock seam on every clock read. Name them so T7's figure is
read for what it measures. `callgrind.sh` on `pingpong/*` is also a new instrument: the switch count under
valgrind's slowdown decides the Ir, so T7 first measures the run-to-run spread (as the `arith` spread was) and
gates on pingpong only if the spread sits inside the budget; else recorded, not gated, as Phase 6 did
(`phase-6-gate.md:1035-1042`).

M6. T2 has no perf measurement, and c1 puts COUNTER on every loop kind. If the loop-step op gains a
conditional, `emptyloop` moves (the memory section 5 cites prices a conditional in that arm at 1.25% on
`emptyloop`). Either measure `emptyloop` and `rexxcps` at T2's close or require the design to keep COUNTER off
the step op of a loop that has none.

M7. T5: "if none exists, T5 stops with the measurement", and then nothing. Say the figures go to Moritz under
the stopping rule and that criteria 4 and 7 are read through his ruling (Q4). The recompile route exists: the
driver already recompiles a chunk when its trace setting changes ("Staleness heals -- the chunk is recompiled
under the setting now in force", `src/ir/drive.rs:1899-1901`), so the flowed ops can be lowered differently
under `?` with no default-mode test.

M8. D8's widened scan hits vocabularies that hold `"Phase 5"` with no row using it: `tests/bif_assertions.rs:781`
and `tests/keyword_assertions.rs:668` (`PHASES`), `tests/owners.rs:331` (`SPLIT_TABLE_PHASES`), and the count
assertion at `tests/owners.rs:472`. T6 prunes "Phase 5" from them (scout A section 3: no exempt row uses it)
rather than exempting the files.

M9. Criterion 4's witnesses read stdin (`pa`, `pakinds`, `di`, `eof`, `dbgcall`). The corpus has the form
(`rust/corpus/lang/trace_debug.stdin`); name it. `i2` prints `condition('O')~message`, which carries `&1` under
R3; rewrite it to print code, position and traceback only before it is compared. `c5` needs its second file;
the corpus has `.d` directories (`rust/corpus/lang/library_routine_package_blame.d`).

M10. Criterion 6, "in debug over its recorded subset": say the subset and the seed count come from T7's timed
run and are written in the gate record, so the criterion names its source.

M11. Section 4's clippy rule allows the clock seam and the run deadline. `grep -rn -E
'\.elapsed\(\)|thread::sleep|Instant::now|SystemTime::now' src --include=*.rs` (tests excluded) also finds
`src/sync.rs:189-217` (the loom clock), `src/run.rs:573` (`#[cfg(test)]`), `src/signal.rs:238,270` and
`src/scheduler/pool.rs:43` (sleeps on threads sim does not run). Say the plan derives the allow-list from that
command and gives each site its reason.

M12. `fail=wait:K` is "never in a run compared with the oracle", while the oracle report compares every seeded
outcome. Say M11's `fail=` run is excluded from the report.

## Questions only Moritz can answer

| # | Question | Recommendation |
|---|---|---|
| Q1 | The row and the queued item name PCT. The spec builds random bounded preemption under FIFO with a seeded one-event order, because PCT's priorities reorder the ready queue in ways the oracle's FIFO kernel-lock queue cannot take, and its probability bound needs those priorities (dst I1). Confirm the drop and the row amendment at T9? | Confirm. The invariants hold under any schedule, so what is lost is PCT's bound, not coverage; `uniform:1` plus the fairness floor already reach the schedules PCT would add, and oracle-unreachable orders would only swell the report. |
| Q2 | Under R1, is a failed check the program itself makes (an ooTest failure or error count, a gate program's own assertion line) red when no committed oracle outcome of that group part has it? | Yes. It is the suite's own check, licensed by the oracle set rather than judged by it. Without it the kill check sees only hangs, panics and invariant breaches, and criterion 6 is weaker than the row's "gate instrument" needs. |
| Q3 | Criterion 1: the (b) and (c) constructors (the row), or every constructor's owner audited against a plan row? | The row. Class (a) keeps the census caveat; a Phase 9 census, if Phase 9 wants one, is Phase 9's. |
| Q4 | If T5 finds no branch-free design for D6's pause on the Flowed arm: pay about 0.5% on rexxcps (the whole budget) for correct debug pauses, or queue D6 past 6.1 with `pa`, `pakinds` and `dbgcall` recorded as not delivered? | Let T5 try the recompile-under-debug route first (M7). If it fails, queue D6 rather than spend the budget on a debugging feature, and criterion 4 lists those probes as not delivered beside the measurement. |

## Checked and correct at `fc04e3857`

P89's lines `src/ir/drive.rs:2984-2986` (`elapsed_anchor`, `pending_elapsed_reset`, `random_seed`); the hot
exit `:1905` and the `RegionEnd::Flowed` arm `:1930`; the switch-mode arm `src/scheduler.rs:1990-1999`;
`cancel_wait`'s sleeper withdrawal `:1047-1049`; the `stress_collect` test `src/lib.rs:2645`; the 512 assert
`src/activation.rs:497` and `ActivationFlags` as a `u8` using `FORWARDED`, `GUARDED`, `RESERVED`; `cached_clock: Option<i64>` `:375`
and `condition: Option<TrappedCondition>` `:364`; the debt paragraph `tests/closed_phases.rs:31-38` and `CLOSED`
without Phase 5; the `Literals` EXEMPT rows `tests/assertions.rs:185-281` with `unblocked_by: "Phase 5"` and
the still-blocked check `:740-755`; `scheduler_inconsistency` exists (`src/lib.rs:814`);
`a_reply_continuation_keeps_the_elapsed_clock_and_the_random_seed` (`src/scheduler/tests.rs:1058`);
`whole_groups` with `ORACLE_RUNS`, `WALL_CLOCK`, `EVERY_FORCES_THE_RACE`, `DIFFERING` and `GATE_ENV`
(`tests/concurrency_tests.rs`); `oracle-crashes.txt` 10b ("Exactly one restore per process survives");
`phase-4-exclusions.txt:530` (OWNER Phase 5 for `::REQUIRES`) and `:844` (stem OVER order); `callgrind.sh`'s
`PROGRAMS` without `pingpong/*`; `parse-errors.tsv` read by `rust/crates/rexx-parse/tests/errors.rs:47`; the
origin failure `bg/b6efbfd53/logs/g4-test-release.txt:2210-2213` (REPLY derived every, rc 1); the queued items
D7 and D8 cite, with Moritz's PCT wording and NC-h, NC-i; `phase-6-gate.md` G9 (`:685`, `:750`) and criterion
3's TSan run (`:776`); Phase 6 spec section 7's instrument, noise rule, ±4% wall-clock rule and stopping rule;
criterion 5's row list against scout A's and scout B's ooTest columns; the dispositions of c1-c8 and b1-b18
against scout A section 2; the oracle lines `LanguageParser.cpp:866-876` and `:4129`, `Activity.cpp:3240-3263`,
`RexxActivation.cpp:3455-3460`, `:4640-4657`, `:1494-1500`, `:174`, `:312`, `ActivityManager.hpp:359`.
Staging order holds: T1 before T5, T3 before T6, T4 before T6, T7 before T8, T2-T5 before T6's `closed_phases`.
