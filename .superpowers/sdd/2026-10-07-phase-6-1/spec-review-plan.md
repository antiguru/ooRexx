# Spec review, lens `plan`: Phase 6.1

Spec: `docs/superpowers/specs/2026-10-07-phase-6-1-loose-ends.md` (uncommitted draft) at `be19fd06a`.
Builds ran in `git worktree add /tmp/claude-1000/p61/srp/wt be19fd06a`, `CARGO_TARGET_DIR=/tmp/claude-1000/p61/srp/target`,
both removed afterwards.

## BLOCKER

### B1. Exit criterion 1's census command cannot fail on its subject

Spec 8.1: "The committed census command shows classes (b) and (c) empty". Section 3: "A committed command
re-derives the census (scout A's `sites.py` and `classify.py`) and shows classes (b) and (c) empty."

`classify.py` does not read an owner from the code. It maps constructor names and `file:line` keys to a
class through two hand-written tables (`loud-census.md:216-275`, `DEFAULT` and `PER_SITE`; the script's own
comment: "The mapping is read from the source by hand"). So:

- Editing `DEFAULT['receiver_class']` from `'b'` to a `d-` class empties (b) with no code change.
- `PER_SITE` is keyed by line. Scout A already saw two keys drift and one site fall to `UNCLASSIFIED`
  (`scout-a-report.md:18-25`). After 6.1's edits most keys drift. A drifted key falls to its constructor's
  `DEFAULT`, and for `run.rs:1197` (OPTIONS) that default is a `d-guard` constructor, so a still-present
  (b) site would leave (b) silently. `UNCLASSIFIED` is not (b) or (c), so the criterion as worded passes
  over it.
- After 6.1 the GUARD and DEVIATION constructors also carry owner `None`, which is what class (c) means
  ("names no phase"). Only the hand table tells them apart.

Fix: make the criterion structural. A test (beside `closed_phases`) that enumerates every `Loud`
constructor whose owner is `None` and requires each to be listed in one committed disposition table
(constructor, disposition GUARD or DEVIATION, and the record it cites: an `oracle-crashes.txt` line, an
exclusions row, or the probe names), failing on any unlisted constructor and any listed one that no longer
exists. Keep `classify.py` as a report, and add "zero UNCLASSIFIED" to criterion 1 if it stays.

## IMPORTANT

### I1. The `Activation` layout in section 2 does not fit 512 bytes

Spec section 2: "`size_of::<Activation>() == 512` holds (`activation.rs:497`): the clock anchor inline as an
`i64` with 0 for unset, the two flags as bits in `ActivationFlags`, and the seed, the SETLOCAL list and the
condition in one cold box allocated on first use." Scout B said this was unchecked ("Whether 8 inline bytes
still fit in 512 must be checked with the assert; I did not build it", `scout-b-report.md:103-104`).

Run: added `probe_anchor: i64` and `probe_cold: Option<Box<[u64; 4]>>` after `cached_clock` in
`activation.rs`, replaced the assert with `const _: [(); 0] = [(); size_of::<Activation>()];`, then
`CARGO_TARGET_DIR=/tmp/claude-1000/p61/srp/target memcap 8G cargo check -j 4 -p rexx-exec`:
`expected an array with a size of 0, found one with a size of 528`. With the `i64` alone: `520`. The
unmodified assert with both fields: `error[E0080]: evaluation panicked: assertion failed:
size_of::<Activation>() == 512`.

The spec's layout is 528. The performance budget (section 5) leans on "one word" inline growth on the
hot call path (fibcall, fibfunc, dispatch, sendloop), the programs Phase 6 already closed over budget and
accepted (`phase-6-gate.md:1278-1284`).

Fix: name what shrinks. Candidates visible in the struct: `cached_clock: Option<i64>` (16 bytes) to an `i64`
with a sentinel saves 8; `condition: Option<TrappedCondition>` already sits on `Activation`
(`activation.rs` struct) and may absorb `active_condition` instead of the cold box. Or decide the size
moves to 520/528 and say so, with T1 measured (see I3). Open question Q1.

### I2. D7's default-mode cost is measured on a program the instrument refuses

Spec section 5: "D7 adds a branch per switch in `next_runnable`, measured on the ping-pong benchmark."
The instrument is "as Phase 6 spec section 7": callgrind plus wall clock. `callgrind.sh` refuses it:

    bash bench-programs/callgrind.sh -r 1 -o $SCRATCH/srp-cg -p pingpong/pingmsg x=/bin/true
    not in PROGRAMS: pingpong/pingmsg
    rc=2

(`callgrind.sh:40-55`: `PROGRAMS` is the top-level `*.rex` only.) Phase 6 recorded ping-pong by
`wallclock.sh` only, "Recorded, not gated" (`phase-6-gate.md:1035-1042`), and the wall-clock layout noise
is ±4% (Phase 6 spec section 7), so a +0.5% budget cannot be decided on it.

D7 also adds default-mode work the section does not name: the `Interp::now()` seam on every clock read
(section 4 table), and `gc=q` on every allocation unless it folds into the existing `stress_collect` test in
`collect_if_due` (`lib.rs:2644-2649`), which `alloc`, `alloc4c` and `heapshape` exercise.

Fix: commit a callgrind command for the ping-pong programs (extend `callgrind.sh` to `pingpong/*` or
give the `valgrind` line), state that `gc=q` hangs off `stress_collect`, and name `alloc` among D7's
programs.

### I3. Performance measured once at the close; D1 is the risky change and lands first

Spec section 5: "measured once at the close"; stopping rule "three rounds". D1 (T1) is the only change on
the per-call path and is the first stage; with a single measurement at T9, a T1 regression is found after
T2-T8 are built on top of it, and a round then means reworking T1 under seven stages. Phase 6 measured
per stage (Phase 6 spec section 7, "measured once per stage"), and its per-call programs closed over budget
by ruling. Per-step thresholds also hide cumulative drift, so the close figure must stay; the issue is
the missing early one.

Fix: measure at T1's close (fibcall, fibfunc, dispatch, sendloop, dispatchclass), at T5's (rexxcps,
emptyloop) and T7's (I2's programs), each against the 6.1 base, plus the cumulative figure at T9. Also
say whether the Phase 6 wall-clock rule (±4%) applies.

### I4. D5's inserts have no completion check

Spec D5: "with the message inserts filled"; O2: "touch 213 parser sites". Criterion 4 checks scout B's
`i2` 1-8 and `interp`, which exercise a handful of error codes. Nothing decides whether the other sites
pass their values, so "inserts filled" cannot fail outside eight probes.

Also unnamed: roadmap row 3 records "message text and substitutions deliberately not reproduced
(2026-07-28 scope decision)" (`2026-07-27-rust-rewrite.md:654`). O2 reverses a recorded ruling and should
say so.

Fix: a committed check that every parser raise site whose message template carries `&N` supplies N
values (a test over the error table and the `rexx-parse` sites), plus one corpus program per reachable
error code that has inserts. Or narrow D5 to the traceback line and the insert sites the probes and ooTest
rows reach (O2's alternative). Open question Q2.

### I5. The sim gate's judge has no rule for today's licences

Spec section 4, tier 1: "a nonzero ooTest failure or error count where no oracle run had one, an rc no
oracle run had". The existing judge also accepts P86 count-only differences, the `DIFFERING` rows and the
P48 `WALL_CLOCK` rerun (`scout-c-report.md` section 3, `concurrency_tests.rs:3312-3355`). A `DIFFERING`
row fails tier 1 under every seed as written; a `WALL_CLOCK` row meets a virtual clock whose per-clause
quantum the spec does not set.

Fix: say that the existing verdict's licences apply per seed, and give the quantum (or its derivation)
in the spec, since a too-large quantum makes legal programs with elapsed-time bounds fail tier 1.

### I6. The outcome-set cache is written during the gate

Spec section 4: "its outcome set is sampled, committed per group, and only grows; an outcome outside it
triggers extra oracle runs before it is judged." A gate run that grows a committed file changes the tree
under the gate and makes a verdict depend on the cache's state, so commit and seed no longer replay a
verdict.

Fix: the gate reads the committed set read-only; extra oracle runs for an unseen outcome are written to
a side file in the run's output, and the committed set grows only through an explicit refresh variable
(the `REXX_INTROSPECTION_ARITY_REFRESH` pattern). The replay line names the cache's commit too.

### I7. Tier 2 has no disposition, so criterion 6 never reads it

Spec 8.6 gates tier 1 only; section 4 says tier 2 "is reported". Nothing says where, or what happens to a
tier-2 outcome.

Fix: criterion 6 adds "every tier-2 outcome of the close run is listed in the gate record with a
disposition (a legal schedule, argued; or a queued defect)".

### I8. Gate runtime is unmeasured and O6 multiplies it

O6: 32 seeds per group, and criterion 6 runs them "in release and debug". At the S5 close the
`group_runs` binary alone took `finished in 825.39s` in G4
(`.superpowers/sdd/2026-10-01-phase-6-s2-s5/bg/42b29493c/logs/g4-test-release.txt`, the longest `finished
in`), with two crate modes per group part. 32 seeds replace those two. Virtual time shortens the sleeping
groups, but the CPU-bound groups scale with the seed count.

Fix: T7 measures one seed's cost per group in both builds and the spec sets the seed count from it (for
example the full list in release, a fixed subset in debug), with the gate's wall time stated as a budget.

### I9. Criterion 8 drops G9

Spec 8.8: "Gates G1-G8 as at the Phase 6 close". The S5 close ran G9 loom
(`phase-6-gate.md`, S5 close status lines: `G9 loom exit 0`). T7 changes the timer thread's arming and the
idle path (`clause.rs`, `timer.rs`), which loom covers.

Fix: G1-G9. Say whether the Phase 6 TSan run (its criterion 3) is repeated, since T7 touches the timer
and pool paths it covered.

### I10. `tests/assertions.rs`'s Phase 5 EXEMPT rows have no disposition

D8 widens `closed_phases` to the owner tables in `tests/`. `tests/assertions.rs:185-281` holds 12 EXEMPT
rows with `unblocked_by: "Phase 5"` (the `Literals` `runDynamicSource` rows), and the module polices that
each still fails (`assertions.rs:740-755`). The widened scan fails on them, and no stage gives them a
disposition: they are not census sites, and section 3's table covers census groups only.

Fix: T6 gives each row a disposition like a refusal (implement what `runDynamicSource` needs in the
harness, or re-home with a true reason). Open question Q3.

## MINOR

### M1. Section 4 is written as if O3-O7 were answered

Section 4: "From scout C, with Moritz's choices in section 6 applied." Section 6: "Answered by Moritz
before planning". They are recommendations, not answers. Fix: "with section 6's recommendations applied";
the planner re-reads section 4 after the answers.

### M2. `dbgcall` may not pass at T1

Criterion 3 counts `dbgcall` as a D1 scope witness and T1 delivers D1. The probe types `call s` at a pause
and expects pauses after each of `s`'s clauses and, by D6's rule, after the CALL's return
(`scout-b-report.md`, pause table: "`call sub` (after `return`, in the caller) ... pause"). The CALL-return
pause is T5's. Fix: T1's witness for `debug_pause` is a probe with no flowed clause, and `dbgcall` moves to
T5's witnesses, or T5 precedes T1's close.

### M3. NC-h and NC-i have no stage

Section 3 closes the queued NC-h gap; T6's row does not list it, and NC-i (lower-case `Owner:`), from the
same queued item (`2026-09-29-closed-phases-owner-gaps`), is not mentioned. Fix: T6 delivers both.

### M4. The scope probes print times

Criterion 3: "each scout B scope probe agrees with the oracle". `reply` prints `m after 0.031429..` on the
oracle and varies by run; `c2`, `c5` and `int` are already predicates. Fix: state that witnesses are
rewritten as predicates (as `c2` is) before they are compared.

### M5. Parse-path programs are not named for D5

Scout B: "A larger `ParseError` widens the parser's `Result`; box it if parse benchmarks move." `startup`
parses the embedded `.orx` files and `parse` is in `callgrind.sh`. Section 5 names programs for D1, D6 and
D7 but none for D5. Fix: name `startup` and `parse` for D5.

### M6. M11's witness is slow

Section 4: M11 "fixed with the prototype as its witness". The prototype scans recursion depth on a 64 MiB
stack for about 25 s in release (`scout-c-report.md` section 4); G6 runs it in debug. Fix: the witness uses
`fail=wait:K` (O7) or a fixed depth.

### M7. Defects the mutant runs find have no owner

Section 4 runs M9 and M10 under the knobs. If either is killed, that is a scheduler defect to fix, and only
M11's fix is planned. Fix: T8 owns any fix a mutant run calls for, or the spec says such a find is queued
and the mutant recorded as killed.

### M8. "queued as their own items" is not true yet

D9: scout A section 4's divergences and `cond3`'s traceback are "queued as their own items". Of those, only
`.context~name` in a `Routine~new` body has a queued file (`2026-10-01-routine-call-context-name.md`); the
others (concatenation with a variable reference, `RAISE NOVALUE` into `CALL ON ANY`, `subclass package`,
`subclass rexxinfo`, `cond3`'s extra traceback line) have none in `.superpowers/sdd/queued/`. Fix: queue
them with the spec, or make it a T9 deliverable.

### M9. Criterion 5's command and escape hatch

Criterion 5 re-runs "the Phase 6 whole-groups table" without naming the command, and "carries a recorded
reason" does not say where the reason is recorded. Scout B's `TIME` TEST_5 and TEST_11 are unattributed
and absent from the list. Fix: name the command and its output file; reasons go in the gate record, each
naming a cause outside 6.1's items; list TEST_5 and TEST_11 as rows to explain.

### M10. PCT's step bound is not in the replay key

Section 4: `pct:d` "places d preemption points uniformly over the run's clause steps". Scout C draws them
over 1..k, with k from `k=` or the previous run's step count. Fix: the committed seed list carries k per
group part, and the replay line prints it.

## Dependency check (brief's examples)

- T3 (b5) before T6 (b13's DEVIATION record): held; the oracle-crash stand-in is reachable on ours only
  after b5 (`scout-a-report.md`, b13 row).
- T1 before T5 (`debug_pause`): held, with M2's exception.
- T7 before T8: held.
- T4 before T6 (`phase-4-exclusions.txt:530` rewritten when b3 lands): held.

Every queued item named by roadmap row 6.1 has a stage: `phase-5-refusal-labels` (T6),
`do-with-over-refusal` and `use-arg-message-term` (T2), `elapsed-clock-per-routine-and-reset` and
`setlocal-scope` (T1), the two debug items (T5), `interpret-syntax-traceback-line` (T4),
`unjudged-scheduler-mutants` (T8), `seeded-random-switch-mode` (T7, T8). Every census (b)/(c) group of
scout A section 2 (c1-c8, b1-b18) has a disposition in section 3.

## OPEN QUESTIONS for Moritz

| # | Question | Recommendation |
|---|---|---|
| Q1 | `Activation` grows to 520 or 528 bytes under D1, or an existing field shrinks to hold 512 (I1)? | Shrink `cached_clock` to a sentinel `i64` and reuse `condition` for the propagated condition if T1 shows it equivalent; measure at T1 either way. |
| Q2 (O2) | Inserts: in 6.1 across all 213 sites, or not? | Differs from the spec: land the traceback line and a generic insert from the error token (most templates' `&1` is the token at the error, which `ParseError` already locates); per-site values only where a probe or ooTest row shows a non-token insert. That reverses the 2026-07-28 Phase 3 ruling partially and should be ruled as such. |
| Q3 | The 12 `Literals` EXEMPT rows naming Phase 5 (I10): implement `runDynamicSource` in the harness, or re-home? | Probe whether the interpreter already runs them outside the harness; if yes, it is a harness gap to fix in T6, else re-home to Phase 9 with the reason. |
| Q4 (O5) | Tier 1 fails, tier 2 reported. The queued item, in Moritz's words, says "every outcome is an oracle outcome or a recorded divergence". | Agree with the tiers, but only with I7: every tier-2 outcome gets a written disposition at the close, which keeps the queued item's rule as a review duty rather than a sampling-luck gate. |
| Q5 (O6) | Seeds and policy mix. | Do not fix 32 before T7 measures one seed's cost per group (I8). |
| Q6 | Performance per stage or once at the close (I3)? | Per stage for T1, T5 and T7, plus the cumulative close figure. |
