# SDD ledger — plan: docs/superpowers/plans/2026-08-27-phase-5b.md

Spec (binding): docs/superpowers/specs/2026-08-27-phase-5b-instances.md, decisions D57-D69 + D59a.
Global constraints: docs/superpowers/records/2026-08-17-phase-5a/global-constraints.md, plus the
5b plan's Global constraints section (phase-5b.txt amendment, the phase-gate command, three added rules).
Base: 4c553f383. Tree clean. `git diff --stat c7345f7a9..HEAD -- rust/` is empty, so the plan's
"all five gates green at c7345f7a9" carries to the base.

## Pre-flight scan

Every named site re-measured against the tree at 4c553f383 before the scan was written.

| # | tasks | produces / consumes | finding |
|---|---|---|---|
| 1 | 0 -> 1,2,3,4,5,7,8,9 | Task 0 creates `corpus/phase-5b.txt`; seven later tasks add entries | clean; Task 0 is first, as the plan orders |
| 2 | 0 self | six reader sites named | all six verified present: `corpus.rs:657`, `coverage.rs:675`, `ir_dual.rs:1185`, `collect_stress.rs:141` (guarded), `trace_oracle.rs:688` (unguarded), `EXPECTED_SUBSET_5A` at `coverage.rs:1212` |
| 3 | 1 self | `receiver_kind` seam, `Object INIT` native, `DEFAULTNAME` absent | verified: `dispatch.rs:1123` `Body::Instance(_) => Err("an instance of a user class")`, `dispatch.rs:375` `("Object","INIT",Arity::Fixed(0),native_no_op)`, `DEFAULTNAME` matches nothing under `crates/rexx-exec/src/` |
| 4 | 1 -> 2,3,5,7,9 | instance receiver kind | ordered; clean |
| 5 | 2 -> 3 | both change the send's method lookup: Task 2 makes the behaviour snapshot a property of where behaviour lives, Task 3 adds the per-object first step in front of it | ordered 2 then 3; Task 3 consumes Task 2's structure. Flagged for the Task 3 dispatch |
| 6 | 3 vs 8 | both add a `[]=`: Task 3 for `StringTable`, Task 8 for `Array` | verified `"[]="` matches nothing in `dispatch.rs` today, and `StringTable` has `[]`, `AT`, `PUT`, `UNKNOWN` (`dispatch.rs:443`-`:448`). Different natives, no conflict. Task 3's brief quotes an `Array`-`[]=` refusal transcript that Task 8 will make stale; Task 3 runs first, so it is only a report-reading hazard |
| 7 | 3 -> 7 | Task 7's `~run` reuses Task 3's restricted-private check (D66) | ordered; clean |
| 8 | 1 -> 5 | Task 1 registers `UNINIT` at construction; Task 5 delivers it | ordered; `lib.rs:6446` `debug_assert!(stats.pending_uninit.is_empty(), ...)` verified present with the comment naming itself as the delivery site |
| 9 | 5 -> 6 | Task 6's DEVIATION pins the crate side that Task 5's delivery 3 changes | ordered, as the plan states |
| 10 | 4 vs 1 | plan's ordering paragraph lists Task 4 among the three that "may land before Task 1" | **finding.** Only Task 4's `FORWARD`-instruction half is Task-1-independent. Both of the spec's replacement `DELEGATE` probes construct with `.K~new` (`say 'main' .K~new~length`, and `k = .K~new`), so Task 4's stated goal -- both table D rows agree over probes that send the delegated message -- needs Task 1. Ruling below |
| 11 | 4,5,8 | three tasks edit gate-table files | Task 4 rewrites two probes under `corpus/gate-tables/directives/` and no `gate_table_d.rs` row text; Task 5 corrects `gate_table_c.rs`'s `obdes` control text; Task 8 changes `methodsbyclass`'s probe and its `oracle_lines`. Different rows, sequential; clean |
| 12 | global constraint vs table D | "a task that makes a gate-table row agree adds that row's probe path to `corpus/phase-5b.txt`" | coherent for table D: `Row::probe_path` (`gate_table_d.rs:146`) derives a real file path, `gate-tables/directives/<directive>__<keyword>__<position>.rex`, and both DELEGATE probes exist on disk |
| 13 | 4,5,7,9 self | four tasks delegate an open scope question to themselves (`FORWARD`'s four unmeasured options, the class-`UNINIT` sweep order, `~run`'s option surface, Task 9's enumeration) | intended by the plan: each names "record the answer either way" as its own output. Not a conflict |
| 14 | 6 self | the DEVIATION's file is "`phase-4-exclusions.txt` or a Phase 5 successor named by this task" | intended: the plan asks the task to say which and why |
| 15 | 9,10 | Task 9 is an audit wanting the rest landed; Task 10 is the flip | ordered last; clean |

Ruling: run Task 4 after Task 1, not before — its two replacement probes both construct an instance with `.K~new`, so the plan's "may land before Task 1" is true only of its `FORWARD` half — costs nothing if wrong, since no two implementers run in parallel and the plan permits either order.

## Progress

Task 0: dispatched (sonnet), BASE 4c553f383, brief task-0-brief.md, report task-0-report.md
Task 0: idled without reporting at 19:51 with five edits and corpus/phase-5b.txt uncommitted and no report file; resumed with the write-report-first and wait-in-long-stretches rules, both now in global-constraints.md
Task 0: minor (deferred): corpus/README.md documents a "Phase 5a subset" section and gains no 5b one. Ruling: out of scope for Task 0 -- README is documentation, not one of the six readers the brief names -- flagged for the final whole-branch review to triage.
Task 0: idled a second time, this time legitimately, waiting on a backgrounded debug test run; told it the run's `| tail -100` makes the status tail's, not cargo's.
Task 1: pre-dispatch check done during Task 0's gate wait. `completeNewObject` at ClassClass.cpp:1882 has the plan's four steps in the plan's order (checkAbstract, setBehaviour(getInstanceBehaviour()), hasUninitDefined -> requiresUninit, sendMessage INIT). `ClassGraph::has_uninit`/`parent_has_uninit`/`check_uninit`/`refresh_parent_has_uninit` all exist and are witnessed at lib.rs:7726-7781. `compile_method_source` at dispatch.rs:3541. `Body::Instance` is live at run.rs:3088 and lib.rs:6094-6167. Both gate-table probes exist on disk. No prerequisite missing.

## Interlude: gate parallelization (Moritz, 2026-08-28)

Moritz: "We gotta parallelize the gate, 1h at each point is too slow, especially as we're adding more tests."
Measured from Task 0's own gate logs: the debug gate is 63 min, of which ir_dual 2218.98s, bif_assertions
1084.79s and assertions 446.50s are 91%. Release gate is 15 min with the same three dominating. Each is one
`#[test]` wrapping a serial `for` loop over independent cases, so libtest's pool cannot reach it. Machine is
32 cores / 124 GB and the gate uses about one. Falsified hypothesis, recorded because it looked certain:
assertions.rs has five tests each recomputing collect_all, which looked like a free 5x -- one test alone runs
102.40s against the binary's 102.52s, so they already overlap and memoizing buys CPU, not wall time.
Decision (Moritz): land it now, before Task 1, and I drive it directly rather than via SDD.
Ruling (Moritz's, not mine): do not vendor ootest/ now -- "too early". It stays a git-ignored svn working copy at r13178 and the 5a constraint to verify it with `svn info` stands. Findings kept in vendor-ootest.md for whenever it is revisited: 14 unversioned paths are oracle test residue that a blind `git add ootest/` would commit as upstream content, and several .testGroup files are deliberately binary with 12 carrying CR bytes, so vendoring needs `ootest/** -text` or every expectation derived from them moves silently.
Sequence agreed: Task 0 commit -> Task 0 review -> rayon gate parallelization (controller-driven) -> Task 1.
Ruling (Moritz's): gate parallelization moves to after 5b lands and before 5c, not before Task 1. Measurements and design stand in gate-parallelization.md; the tree is untouched. Cost recorded there: nine remaining tasks each pay the ~63 min debug gate plus fix rounds.
Sequence now: Task 0 commit -> Task 0 review -> Task 1 ... Task 10 -> gate parallelization -> 5c.
Task 0: all five gates green. Gate 5 (debug, memcap 8G) reran and completed 2026-08-28 22:54 in scratchpad/gate-debug-memcap-2.log: EXIT:0, 102 `test result: ok` blocks, 0 FAILED, corpus `264 of 264 matching` under `mode: STRICT (the gate)`. The agent idled a third time without committing; resumed to append gate 5, fill the self-review, drop the stale placeholder headings, and commit.
Task 0: committed 6fc65c487 "Phase 5b Task 0: corpus/phase-5b.txt and its wiring", 6 files, +40. Review dispatched (sonnet) over 4c553f383..6fc65c487, package review-4c553f383..6fc65c487.diff.
Task 0: complete (commits 4c553f3..6fc65c4, review clean -- spec compliant, quality Approved, 0 Critical, 0 Important, 1 Minor which is the already-ledgered corpus/README.md gap).
Task 1: dispatched (opus), BASE 6fc65c487, brief task-1-brief.md, report task-1-report.md. Model is the most capable available because this is the phase's architecture task: it opens the instance seam, ports completeNewObject, and carries a use-after-free the subset stress run structurally cannot see.
STOPPED by Moritz 2026-08-29, mid-Task-1. State at the stop:
  * Task 0: complete and reviewed clean at 6fc65c487.
  * Task 1: committed f7c38531c "Build ~new, INIT and the instance seam". NOT REVIEWED -- no review package was
    generated and no task reviewer ran. Two uncommitted one-line comment edits remain in the working tree
    (dispatch.rs and collect_stress.rs), both removing a set size from a comment ("four steps" -> "steps",
    "the same two mutations" -> "the same mutations"), which is the no-cardinality rule being applied. They are
    the agent's own in-progress cleanup, not a control mutation, and nothing was restored or reverted.
  * The agent had re-confirmed four gates plus the phase gate and was on the debug corpus gate when it was
    stopped; that run was orphaned and killed, so its result is unknown and unrecorded.
  * Task 1's report is at task-1-report.md and is partial.
To resume: review f7c38531c before building on it, and re-run the debug corpus gate, since no gate-5 result
exists for this commit.

## Resumed 2026-08-30

HEAD is c245dc418 "Update prose", Moritz's own commit of the two comment edits the stop left in the
working tree (dispatch.rs, collect_stress.rs, one line each, the no-cardinality rule). Tree clean.
Gates re-run at c245dc418, not at f7c38531c, since the prose commit is what is on disk:
  * gate 1 `cargo fmt --all --check` -> FMT_EXIT:0
  * gate 2 `cargo clippy --workspace --all-targets -- -D warnings` -> exit 0
  * gate 5 `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` backgrounded,
    log scratchpad/gate5-debug-c245dc418.log. This is the run the stop orphaned.
  * gates 3 and 4 (release, release+corpus) still owed; they queue behind gate 5 for the target lock.
Task 1 review dispatched (opus), package review-6fc65c487..c245dc418.diff, brief
task-1-review-brief.md, report task-1-review.md. Given its own CARGO_TARGET_DIR and told the
worktree is read-only, because a write of its would void the running gate.
Task 1 report read by the controller. It is complete except for a self-review section, which no
heading in it matches; everything else the brief asked for is there, including four controls with
transcripts. Five things it hands forward: `objcla` is now a *silent* diverge-stdout (`after 1` vs
`after 0`) and is Task 2's; the `USE STRICT ARG`-inside-`::METHOD` wrong error pair (crate 40.3 vs
oracle 93.901) and `TRACE`'s value lines not following an `OBJECTNAME` override are both written into
Task 9; `dispatch.rex` became a live `Role::Loop` bench axis owing a first baseline row, Task 10's;
and the performance sitting was **not taken**.
**The performance sitting can now be re-taken.** The report's reason was that the machine offered one
usable counter, `perf stat` scaling the `cycles:u,instructions:u` pair to about 50.00% each, which
`rexx_bench::parse_counters` refuses below 99.995% by design. The machine has since rebooted (`uptime`
reads 9 min at 20:47 on 2026-08-30) and the pair now schedules whole:

```
$ perf stat -x, -e cycles:u,instructions:u -- /bin/true
355077,,cycles:u,334781,100.00,,
146817,,instructions:u,334781,100.00,,
```

So the absence was transient, not a machine property -- the retry rule earning its keep. The sitting
is owed for f7c38531c and is queued behind the three remaining gates, since a benchmark taken under a
running gate measures the gate.
Task 2 pre-dispatch check, run against the tree at c245dc418 while gate 5 ran.
* The `BehaviourHandle` family the plan names is present: `class_graph.rs:117` `pub struct
  BehaviourHandle(usize)`, `:1124` `instance_behaviour_handle`, `:1136` `has_method_at`, `:1158`
  `lookup_at`, `:1177` `resolve_super_scope_at`.
* The layering claim holds and is a real cycle, not a style rule: `rexx-classes/Cargo.toml` depends
  on `rexx-core`, and `rexx-core` depends only on `rexx-num`, so `Body` naming a `rexx-classes` type
  would close a loop. `BehaviourHandle` is a newtype over `usize`, so moving it down is cheap.
* The corpus claim holds. Pattern, recorded beside the claim: from `rust/corpus`,
  `git ls-files '*.rex' | xargs /bin/grep -ail '~ *inherit\|~ *uninherit'` gives 14 files, and
  `/bin/grep -qai '~ *new'` matches none of them.
* `objcla`'s committed control is at `gate_table_c.rs:327`: "rebuild an existing instance's method
  lookup from its class on every send, so a method defined after the instance was created answers".
  That is a description of what Task 1 shipped, which is why the row is red for exactly that reason.
No prerequisite missing.
Task 1 review complete (task-1-review.md): spec compliance clean, quality **CHANGES REQUESTED**,
2 Critical / 4 Important / 3 Minor. The reviewer re-measured all five acceptance criteria and both
controls -- it worked around the read-only worktree with `git archive c245dc418 | tar -x` into scratch,
which is a better instrument than judging a transcript and is worth reusing.

**C1 verified independently by the controller**, release binary against the oracle from a fresh empty
directory, three descriptors, both engines:

```
p1  o~objectName = '123'; say o + 1        oracle rc=159 (97.1)   ir/tw rc=0 out=124
p2  o~objectName = '123'; say (o = 123)    oracle rc=0   out=0    ir/tw rc=0 out=1
```

p2 is the phase's worst shape outright: rc 0 and empty stderr on both sides with different stdout.
The cause is the new `Body::Instance { name: Some(..) }` arm of `heap_to_number` (`value.rs:1160`),
which lets a *named* instance satisfy the numeric paths that `operator_operand_gap` only ever sees
after a failure. An unnamed instance is correct. The reviewer validated the fix (delete the arm;
`datatype(o)` still answers NUM) and separately that the logical and prefix-`\` surfaces need the gap
check moved ahead of `logical_value`.

**C2 verified structurally by the controller.** `native_new` (`dispatch.rs:3961`) calls
`interp.heap.set_uninit(object)`; `Interp::collect`'s `debug_assert!(stats.pending_uninit.is_empty())`
(`lib.rs:6447`) is unchanged and its comment now names `native_new` as the setter while still
asserting nothing sets it. A debug build aborts at rc 101 once an instance whose class defines
`UNINIT` becomes garbage; the oracle is rc 0. The gate is green only because no committed program has
that shape. **This is the seam the plan designed** -- pre-flight scan row 8 saw it and did not rule on
it, which is my miss -- and the plan's Task 5 delivery 1 is literally "replace `Interp::collect`'s
assertion with the delivery".

**Ruling: Task 5 runs immediately after Task 1's fix round, before Task 2.** It closes C2 at its root
rather than papering the assertion over, and it costs nothing in ordering: the pre-flight scan gives
Task 5 exactly one predecessor (Task 1) and one successor (Task 6), and nothing in Tasks 2, 3 or 4
produces anything it consumes. The alternative -- leave the window open across Tasks 2, 3 and 4 -- bets
that none of their probes makes an instance with a `UNINIT` unreachable, and "unlikely" is how this
project gets bitten.
Sequence now: Task 1 fix round -> Task 5 -> Task 2 -> 3 -> 4 -> 6 -> 7 -> 8 -> 9 -> 10 -> gate
parallelization -> 5c.
Gate 5 at c245dc418 finished, exit 0, log scratchpad/gate5-debug-c245dc418.log: 102 `test result: ok`
blocks, 0 `FAILED`, 0 `test result: FAILED`, corpus `270 of 270 matching` under
`mode: STRICT (the gate) -- REXX_CORPUS_GATE is set`. That closes the run the 2026-08-29 stop orphaned
and confirms Task 1's own figure. Gates 3 and 4 are not re-run at c245dc418: the reviewer ran gate 4
independently in its own target dir (`REXX_CORPUS_GATE=1 cargo test --release --locked --workspace
--no-fail-fast`, exit 0, `270 of 270 matching`), and the f7c38531c..c245dc418 delta is two comment
lines. The fix commit re-runs all five from scratch.
Task 1 fix round dispatched (opus), BASE c245dc418, brief task-1-fix-brief.md, report
task-1-fix-report.md.

## Task 1 fix round: the implementer died mid-gate, controller took it over

The fixer hit a weekly account limit at 23:55 on 2026-08-30, after reporting "Pass 2: fmt 0, cold
clippy 0, release workspace 0, gated release 0, phase gate 101. Waiting on the debug gate." State
found: work **uncommitted** (6 modified, 2 untracked), no cargo process alive, `task-1-fix-report.md`
complete at 361 lines except its Status checkboxes, a gates section and the performance sitting.
Backed the working tree up first -- `git diff` plus the two untracked files copied to
scratchpad/fixer-backup-085439 -- before touching anything.
Controller re-ran gate 1 (`FMT_EXIT:0`) and gate 2 (exit 0) and relaunched gate 5,
log scratchpad/gate5-debug-fixround.log.

**Ratification the report explicitly asks for: the array deletion stands.** The fixer found, beyond
its brief, that `heap_to_number`'s `Body::Array` arm parsed the array's joined string value, so an
array rendering as a number was a number to arithmetic and to non-strict comparison. Verified by the
controller against the oracle from a fresh empty directory, three descriptors, both engines, on the
release binary as fixed:

```
a = (1,); say (a = 1)   oracle rc=0 out=0      crate rc=120 loud (was rc=0 out=1 at BASE, per the report)
a = (1,); say a + 1     oracle rc=159 (97.1)   crate rc=120 loud
```

and it predates Phase 5b, which the report claims and which
`git log -S "let bytes = self.array_string_of(value);"` pins to `18626fdb1` (2026-08-21), an ancestor
of `f7c38531c^`. So it is a live silent wrong answer inherited from 5a, in the same function and the
same defect class as C1, with no other task scoped to it. Keeping it also makes `eval.rs:1486`'s
`debug_assert` true, which the brief asked to be reconciled rather than left as a debug/release split.
Both shapes stay divergent either way -- the change is silent to loud, which is this phase's own
ordering -- and no gate row moves.

**Still owed on this round:** gate 5, the performance sitting (the report has no section for it), the
report's Status block and gates section, and the commit. The controller does all four, since the
implementer cannot be resumed.
Task 1 fix round committed by the controller as **f06142745** "Keep an object out of an operator's
left operand", 8 files, +418 -74. All five gates re-run by the controller at the tree as committed
rather than relayed from the implementer's pass 2: fmt 0, clippy 0, release 0, gated release 0
(`271 of 271 matching`), debug memcap 0 (102 `test result: ok`, 0 `FAILED`, `271 of 271 matching`
under `mode: STRICT (the gate)`), phase gate 101 with `5b: 6 rows, 4 not yet agree` (C) and
`5b: 2 rows, 0 not yet agree` (D). The four red rows are still `objcla`, `usesem`, `obdes` and
`methodsbyclass`; `abscla` and `creo` still read `agree`. **The round moves no gate row**, which is
correct -- its subject is a surface no row covers, and the report says so instead of claiming a
witness it does not have.
Report updated with a Status table naming who measured what and a controller gates section; the rest
is the implementer's own text.
Performance sitting running, log scratchpad/sitting.log, `--task 1-fixround-1 --commit f06142745`,
the eight-axis guard command from the 5a plan's guard block (checked against the constraints file's
copy, which the file itself warns can narrow silently; they agree). It lands as its own follow-up
commit, which is the convention the tsv's history shows ("Record Task 23's sitting", "Record fix
round 3's sitting"). Pin verified before running:
`sha256sum bench-baselines/pinned/rexx-run-15a1ffa98` is
`141c3fa968ea774655bc00f3e3b9f61cf1c4b738fd2793831d0055e94f1d074b`, matching `PINNED.md`.
Counters re-probed on a real workload immediately before the sitting and both schedule whole:

```
$ perf stat -x, -e cycles:u,instructions:u -- .../rexx-run .../bench-programs/emptyloop.rex
1527085587,,cycles:u,520315833,100.00,,
9276866256,,instructions:u,520315833,100.00,,
```

That also settles what the earlier shortage cost: the unmultiplexed pair reads 9.2769e9 instructions,
matching Task 1's single-event reading of 9.2766e9, where its multiplexed pair read 9.3622e9. The
scaled estimate was about 0.9% high on a quantity deterministic to eight significant figures -- which
is exactly the size of movement a sitting is meant to detect, and why `parse_counters` refuses it.
Sitting taken and committed as **c65b51641** "Record the fix round sitting, eight axes", 380 rows.
**How the sitting is read, settled from this file's own do-nothing control rather than asserted.**
No 5b task has a sitting -- the last rows before these are `23-fixround-3` at `5cc3267cd` -- so the
delta spans Task 0, Task 1 and the fix round together and nothing is attributable to the fix round
alone. `21-fixround-4-comments` and `21-fixround-5-comments` are two comment-only sittings against
the same pin; across them, per axis/arm/size, instructions move `+0.00` on every one of the fourteen
and cycles move `-2.82` to `+0.88`. So instructions are exact across sittings and cycles are not
readable; the machine also rebooted between the two sittings here and its PMU behaviour changed, so
that control does not even cover this boundary. Instructions read: `emptyloop` ir `-1.30%`/`-1.25%`,
`varlookup` ir `-0.56%`/`-0.54%`, everything else within `+-0.25%` except `dispatchclass` at `+0.22%`
to `+0.45%`. Guard met -- nothing regresses half a per cent over three commits. `dispatchclass` is
the send path's own axis and is where Task 1 put `NATIVE_CLASS_METHODS` and the instance receiver
kind, so it is the axis to watch as 5b keeps landing there. `dispatch.rex` still owes a first
baseline row and stays with Task 10, since adding an axis to the guard is a plan change.

Task 5 pre-dispatch check, against c65b51641. All present: `heap.rs:48/:314/:339`
(`pending_uninit`, `set_uninit`, `clear_uninit`), `class_graph.rs:376/:395/:435`
(`has_uninit`, `check_uninit`, `parent_has_uninit`), and `class_graph.rs:386`-`:389` documenting the
missing `requiresUninit` table that delivery 3 must build. `obdes`'s control text at
`gate_table_c.rs:457` is still the wording the task must correct. **C2 reproduces at this commit**
and is Task 5's "fails on BASE" witness: 200,000 `.K~new` with a `::METHOD uninit` is rc 101 on both
engines in debug, panicking at `lib.rs:6447`, against the oracle's rc 0 `main`; release is rc 0, so
it is a debug/release split as well as a divergence. No prerequisite missing.

## Task 5: built and committed, review dispatched

Task 5 landed four commits on c65b51641: `3ff1055de` "Deliver UNINIT, and characterise the
termination sweep's order", `cf85bfd67` "Make the UNINIT sweep linear in the objects it finalizes",
`eda2a0b94` "Correct three C++ citations, and name the class-object exception in corpus/README.md",
`d708491a7` "Rewrap two UNINIT doc comments and delete a sentence said twice". 24 files, +731 -29.
All three deliveries agree on both engines; `obdes` agrees; corpus `277 of 277 matching`; five
controls run in a scratch copy with every reduced arm at `271 of 271`, so nothing already in the
corpus catches any of them. The debug abort is closed and `lang/uninit_instance_collected.rex` is the
case that fails at BASE in debug (rc 101) *and* silently at BASE in release (a missing `uninit ran`).

**The order question is answered, and the controller verified its citations rather than the
transcripts.** The claimed rule is `strhash(class id) % 17` ascending with entry order as the
tie-break. Every C++ citation holds: `HashCollection.hpp:130` `static const size_t DefaultTableSize
= 17`; `IdentityTableClass.hpp:69` `new_identity_table()` passing it; `RexxMemory.cpp:193`
`uninitTable = new_identity_table()`; `HashContents.cpp:489` `iterateNext` walking
`while (nextBucket < bucketSize) { position = nextBucket++; ... }`, chain first within a bucket; and
`ClassClass.cpp` `RexxClass::getHashValue` returning `id->getHashValue()` with the C++'s own comment
saying why it must be the id string and not the address. That last line is also why instances are not
reproducible, which is D61's whole subject -- measured 19/20 vs 1/20 -- so D61 lapsing over
class-against-class order while staying in force over instances follows from the mechanism rather
than from a transcript.
The new divergence it opened and did not close is likewise citation-checked: `Memory.hpp:107`
`static const size_t SaveStackSize = 10` and `RexxMemory.cpp:306`
`MemoryObject::collectAndUninit(bool clearStack)`. A 10-deep save stack is exactly consistent with
the report's measurement that 0..8 padding clauses diverge and 9 agrees. Written into the plan's
Task 6 for a decision; it is about instances, so D59a's licence does not reach it.

**Owed on this task:** gate 5, which is genuinely running under `setsid` (pid 1192132, started
12:43) and writes `scratchpad/task5/g5.status`; and the performance sitting. Task 5 reports the
sitting failed again with the same `perf stat` message under loadavg 46 and reads that as contention
rather than a machine property. That is consistent with the controller's own successful eight-axis
sitting at 10:37 on a quiet machine, so the controller takes Task 5's sitting once gate 5 and the
review are done and the machine is quiet.
Review dispatched (opus), package review-c65b51641..d708491a7.diff, brief task-5-review-brief.md,
report task-5-review.md, own CARGO_TARGET_DIR, worktree read-only.

## Task 5 review: spec compliance FAIL, fix round briefed

`task-5-review.md`: 2 Critical, 3 Important, 7 Minor. Quality changes requested, and spec compliance
**fails on one of the brief's own named arms** -- the mixin arm the brief said "must agree" does not.

**The order rule survived the hardest test available and is not in the fix round's scope.** The
reviewer predicted bucket order *before running* on id sets the report never used, and every
prediction held on the oracle and on both crate engines: seven ids chosen so bucket order is neither
alphabetical nor declaration order, with a shared bucket and three ids long enough to wrap 64-bit
`size_t`; five mixed-case runtime-built ids; and -- the one that matters most -- **the signed-char
half of the fold, which no transcript in the report or the spec can see**: ids `"\xE9A"` and `"A"`,
where signed predicts A first and unsigned predicts `\xE9A` first, and the oracle answers A first, so
`uninit_bucket`'s `i64::from(byte as i8)` is right. Independent third confirmation from `hashCode`
itself: `'AB'` is `0x0821` = 31*65+66. That is the difference between reproducing a mechanism and
reproducing a transcript.

**CRIT-1 verified by the controller**, release binary against the oracle, fresh empty directory,
three descriptors, both engines:

```
say 'main' / ::class m mixinclass Object / ::method uninit class / say 'uninit on M' / ::class k inherit m

  oracle  rc=0  main / uninit on M / uninit on M
  crate   rc=0  main / uninit on M
```

The oracle fires the inherited finalizer **twice**, once for `K` and once for `M`; we fire it once,
rc 0 with empty stderr on both sides. `check_uninit` is never re-run after an inherit on either path
(`dispatch.rs:4300` does not call it; `lib.rs:5444` calls it *before* the `class.inherit` loop at
`:5447`; `inherit_mixin` at `:5609` not at all). The report's sentence "check_uninit is called from
the same places the oracle calls it" is false, and it is the argument the ordering rests on.

**IMP-2 is why it shipped, and it is the hazard the brief named in its own words.**
`lang/uninit_class_mixin.rex` gives `K` its own `::method uninit class`, so deleting `inherit m`
changes no output on either side -- the witness cannot see its stated subject. **Third instance of
"a witness that cannot fail" in this phase**; Task 1 found two by running its own, this one was
found by review. The brief carried the rule and the round shipped past it, which says the rule in a
brief is not a control -- the same finding as [[dispatch-prose-is-not-a-control]].

IMP-3 is the stale-binary shape: the report's pre-fix figures at 16k and 64k are identical to its
post-fix figures, and the reviewer's interleaved re-measurement makes the real pre-fix numbers much
worse (16,000: 2.6s not 0.30; 200,000: killed at 300s not 49.55). The fix is real and larger than
claimed; the table is wrong.

Fix brief written: `task-5-fix-brief.md`, covering CRIT-1, CRIT-2 (a fixed-point sweep where
`runUninits` makes one pass -- unbounded, the crate is killed at 15s rc 137 with buffered stdout
lost, which is a hang and worse than a wrong answer), IMP-1 (no `processing_uninits` interlock),
IMP-2, IMP-3 and all seven Minors. Dispatch waits for gate 5 at `d708491a7` to finish so the fix
implementer does not contend for the debug target lock; that gate's result is now a data point
rather than the gate, since `d708491a7` is no longer the end of the task.

## Task 5 fix round: four commits, CRIT-1 verified closed by the controller

`b4266cbd6` "Register UNINIT where the oracle registers it, and bound both sweeps", `34246a575`
"Give the three delivery-point fixes corpus witnesses", `e43d2c228` "Make the uninherit row witness
the delivery-time test", `9c0007723` "Unsplice the corpus determinism rule, and record where UNINIT
registers". 24 files, +472 -119. Tree clean; gate 5 running detached.

**CRIT-1 re-measured by the controller** against the release binary built after the fix (14:29),
oracle from a fresh empty directory, three descriptors, both engines. All three shapes now agree,
including a runtime `~inherit` probe the controller wrote rather than took from the review:

```
::class m mixinclass Object / ::method uninit class / ::class k inherit m
   oracle and both engines: main / uninit on M / uninit on M
k = .Object~subclass('K') ; k~inherit(.M)
   oracle and both engines: main / uninit on M / uninit on M
```

Four new corpus rows, one per finding: `uninit_class_inherit_runtime.rex` (CRIT-1's runtime
registration path), `uninit_class_uninherit.rex` (the `~uninherit` direction the fix brief asked be
checked the same way round -- a registration that outlives its method and runs nothing),
`uninit_allocating_finalizer.rex` (CRIT-2), `uninit_nested_collection.rex` (IMP-1). The last one's
comment records that it prints only *whether* the inner finalizer ran inline and never *when*,
because the oracle answers that two ways across runs -- D61 applied correctly to a shape D61 never
named, which is the first time in this phase a task has extended that rule rather than just obeyed it.

---

## Gate parallelization, brought forward (2026-08-31)

Moritz: "Anything running right now? If no, poll the rayon test parallelization forward." Nothing
was running -- all four teammates idle, load 1.68 -- so the controller built it directly, as
`gate-parallelization.md` specifies (controller-driven, not SDD). Its one ordering constraint,
"nothing here starts until Task 0 has committed", was already satisfied at `6fc65c487`.

Landed as one commit on top of `9c0007723`. Five files, +34 -16. All five gates pass on the live
tree; gate 5 exit 0, zero `FAILED`, 103 ok results.

**The headline is 5.0x on gate 4, not 14.3x.** The three sweeps go 823.4s to 57.7s in isolation
(14.3x), but gate 4 as a whole goes 933.54s to 187.67s, because the binaries this does not touch now
dominate it. The gate figure is the one that describes what the phase actually pays. Gate 5 goes
~63 min to 10:24.94.

Full write-up, including the three claims in the design that did not survive contact and the five
acceptance checks answered one by one, is in `gate-parallelization.md`'s new "Built" section. Two
things worth pulling up to the ledger:

* **The design's "one call per file" would have bought zero wall clock** in two of the three
  binaries. Both `assertions.rs` and `bif_assertions.rs` have *two* full-sweep tests, and libtest
  overlaps the tests within a binary, so the untouched sweep would simply have become the new
  bottleneck. Five sites, not three. This is the same "libtest already overlaps them" fact the
  document's own killed-hypothesis section rests on, applied one step further than the design applied
  it.
* **Two of the five acceptance checks needed no work, both because their premise was wrong** rather
  than because the work was easy -- the oracle counter is already atomic (and no sweep builds an
  `Oracle` at all), and no test reads any of the ten `thread_local!` counters. Worth noting against
  the pattern this phase keeps hitting: a check can be satisfied for a reason unrelated to the one
  that motivated writing it, and saying "passed" without saying which would have hidden that.

The performance guard is not implicated: `cargo tree -p rexx-exec --edges normal` names `rayon` zero
times, so the shipped interpreter is unchanged and no `rexx-arms` sitting is owed.

**Still outstanding from Task 5**, unchanged by this: the task-5 agent's fix round is committed and
its report written (`task-5-fix-report.md`, 15:49) but never messaged to the controller -- the fourth
silent finish this phase -- and that fix round has not yet been reviewed. Task 5's own performance
sitting remains reserved for the controller, and `task-2-brief.md`'s prerequisites still need
re-measuring against the post-Task-5 tree before dispatch.

**Correction, same day.** `58b3043a1` was committed with a false sentence in its message -- "the
lockfile churn is one line" -- while actually carrying ~30 unrelated dependency version bumps (138
lines of `Cargo.lock`). Cargo re-resolved the graph when the dev-dependency was added. Corrected in
`2d5614e63`: lock restored to `9c0007723` plus the single `+ "rayon",` line, all five gates re-run
against it and green, and the lock verified not to have moved during those runs. Not amended, so the
false sentence stands in history at `58b3043a1`.

The mechanism is worth carrying forward: `git diff --stat` was read *before* the gates and correctly
showed `Cargo.lock | 1 +`; the bumps appeared *during* the gate runs and the file was staged on the
strength of that earlier reading. A build tool rewriting a tracked file between review and commit is
a different hazard from a concurrent agent doing it, and nothing in the workflow watched for it.

## Task 5 closed pending review; Task 2 dispatched (2026-08-31)

**Phase gate re-read by the controller at `2d5614e63`**, documented command, exit **101** as designed
while 5b rows are red. Table C: 5a 135 rows 0 not-agree, **5b 6 rows 3 not-agree**, 5c 1347 rows 941
not-agree. Table D: 5a 36 rows 0 not-agree, **5b 2 rows 0 not-agree**.

**`obdes` agrees.** That is Task 5's row -- "Object Destruction and Uninitialization" -- and this is
the controller's own reading of the phase gate, not the task's claim. The three 5b rows still red in
table C are `objcla` (Task 2's), `usesem` and `methodsbyclass`.

**Task 5's gate 5 is satisfied on a superset.** The implementer correctly ruled two of its own runs
void (it edited corpus files while they ran, so a compiled `EXPECTED_SUBSET_5B` faced a grown
`phase-5b.txt`) and never landed a clean one. It does not need one: all five gates are green at
`2d5614e63`, which contains the fix round plus the two rayon commits. The implementer also caught its
own vacuous phase-gate invocation -- `REXX_PHASE_GATE=5b` without `REXX_CORPUS_GATE=1` exits 0 in
REPORT MODE, and a 0 there is not a gate reading. Good self-reporting on all three.

`task-5-fix-reviewer` dispatched against `d708491a7..9c0007723`, read-only, working in a `git archive`
extract with its own `CARGO_TARGET_DIR`. Its brief makes the whole-set check on item 2 mandatory
rather than sampled: every new corpus row gets the behaviour it witnesses deleted, to confirm the row
reddens. IMP-2 was a witness that could not fail, and that is the third instance of that shape this
phase.

**Task 2's brief re-measured against the current tree before dispatch**, which is what that step is
for -- three of its four "what is already there" claims had moved:

* **Four of five `class_graph.rs` line numbers had rotted** (Task 5 added to the same file):
  `instance_behaviour_handle` 1124 to 1150, `has_method_at` 1136 to 1162, `lookup_at` 1158 to 1184,
  `resolve_super_scope_at` 1177 to 1203. Only `BehaviourHandle` at 117 held. The brief now cites all
  five **by symbol and not by line**, and says why.
* **The corpus count moved 14 to 16**, Task 5 having added two rows that mention inheritance. The
  brief no longer carries the number: what is load-bearing is that *none* of those files constructs
  an instance, which is still true, and that is now what it states.
* The dependency-cycle claim holds exactly as written: `rexx-classes` depends on `rexx-core`,
  `rexx-core` on `rexx-num` only.
* `objcla` still reads `diverge-stdout loud=no 5b`, re-measured at `2d5614e63`.

## Task 5 fix-round review: IMP-A confirmed and refined (2026-08-31)

`task-5-fix-reviewer` returned 0 Critical, 2 Important, 5 Minor (`task-5-fix-review.md`). It did the
whole-set check the brief made mandatory -- nine mutation arms, each a `git archive` extract with one
edit and its own `CARGO_TARGET_DIR`, binary `cmp`'d against pristine on every arm -- and **every new
corpus row reddens when the behaviour it witnesses is deleted**. IMP-2's failure shape does not
recur. That was the round's main risk and it is closed.

**IMP-A verified independently by the controller, and it is real.** The termination sweep's copy of
the `processing_uninits` interlock is load-bearing and no committed row can see it.

Method: extract of `2d5614e63`, pristine and mutated builds in separate target dirs, `cmp` confirming
the binaries differ, every `uninit_*.rex` plus `obdes.rex` run on both engines and compared
pristine-against-mutated. **All ten rows identical.** Then a probe written here rather than taken
from the review -- `uninit_nested_collection`'s question asked of the *termination* path, the outer
finalizer reached because the program ends rather than by a program-level `GC('force')`:

```
oracle (10 runs, one distinct stdout, rc 0, empty stderr)
  start / end / outer fin / inner built / inner ran inline? 0
crate, interlock intact, both engines:   inner ran inline? 0   rc 0, empty stderr
crate, interlock removed, both engines:  inner ran inline? 1   rc 0, empty stderr
```

**The refinement, which the review did not separate.** The interlock is two halves and only one is
load-bearing:

* the early-return guard `if self.processing_uninits { return Vec::new(); }` -- removing it alone
  changes **nothing**, not on any of the ten rows and not on the probe above, which does catch the
  other half. No witness was found for this half. That is "no witness found", not "unreachable";
  nothing here ran it to a conclusion.
* the assignment `self.processing_uninits = true` -- this is what blocks the *nested* ready-sweep,
  and removing it is what produces the silent wrong answer above.

A first mutation that removed only the guard is why this was nearly recorded as "IMP-A does not
reproduce". The half that matters had to be named before the probe could see it.

**IMP-B confirmed.** `task-5-fix-report.md` ends mid-sentence at "The gate line below is from the
documented command..." with nothing after it, and records **no exit status for any gate**. The
substance is unaffected -- all five gates are green at `2d5614e63`, which contains the round -- but
the round's own record does not say so. To be completed by a controller note appended to that file,
not by rewriting the implementer's words.

**Queued, not dispatched:** a second Task 5 fix round adding the termination-path witness. It touches
`corpus/phase-5b.txt` and `coverage.rs`, which `task-2` is editing right now, so it waits until
`task-2` commits. Any gate run spanning this moment would be void.

## Task 2 verified and closed; item A given an owner (2026-09-01)

`task-2` landed `f558ea501`, tree clean, 22 files, +434 -115. **Verified by the controller against
the tree, not the report.** Phase gate at that commit, documented command, exit 101 as designed:
`objcla` reads `agree`, table C 5b goes **3 not-agree to 2** (`usesem`, `methodsbyclass` remain),
table D 5b stays 0, 5a stays 0 on both tables. The binary was checked newer than every source under
`crates/` before any probe was run against it.

The task's own controls are the strong kind and are worth naming: copying the behaviour on
`~inherit` too moves **no gate row in either table in any phase** and reddens exactly one corpus
program, its new witness. It also caught a witness that could not fail *before* committing -- its
first UNINIT witness had the class gain `UNINIT` after construction, which `native_new` never
registers, so `answers_uninit` was never asked -- and replaced it with the opposite shape. That is
the fourth appearance of that failure mode this phase and the first one caught by the author.

**Item A verified and assigned.** `.K~define('X', "return 'from-source'")` then `.K~new~x`:
oracle rc 0 `from-source`, this crate rc 120 `method "X" of class "K" is not implemented (Phase 5)`
on both engines, three descriptors read separately. Loud, not silent. Given to **Task 9** in
`b1ec122ca`, whose section already says its list "is appended to as later tasks find more". It is not
Task 3's: Task 3 owns the per-object machinery, while this is a class-side `~define` whose effect on
an instance was unmeasurable in 5a. Its sibling `~defineMethods` after construction is recorded in
the same entry as measured but unbuildable until `.Directory~new` exists.

**Item B, the performance guard: the pin is stale and the sitting is mine.** `task-2` was right to
refuse it. `PINNED.md` documents this exact defect from the previous pin -- a guard measuring against
a floor so low that "all of them" are "headroom to burn".

**A correction to my own check.** My first staleness command returned **0 commits**, which reads
exactly like "the pin is current". It was well-formed but resolved relative to the cwd, which was
`rust/`, so it asked about `rust/rust/crates`. From the repo root it is **153**, confirming
`task-2`'s 152 plus its own commit. The cwd-proof form is the `:/` prefix. **What caught it was the
peer's contradicting number, not any check of mine** -- had `task-2` not reported a figure, the 0
would have stood and licensed a sitting against a 153-commit-stale pin, which is precisely what
`PINNED.md` exists to prevent.

**The call:** one drift sitting at `b1ec122ca` against the stale pin first -- the last chance to see
whether Tasks 1, 2 and 5 moved the tree against 5a's start, unattributable across 153 commits but
real -- and then re-pin, so the remaining seven tasks get an attributable guard. Sitting running;
the Task 5 second fix round is briefed and held behind it because it would build and pollute it.

## The performance pin retired, and the drift Moritz ruled on (2026-09-01)

The 5a pin `rexx-run-15a1ffa98` was 153 crate-source commits stale, so a drift sitting was taken
against it before retiring it (`bench-baselines/phase-5b-drift.tsv`, all eight axes, 7 rounds,
quiet machine). Instructions per pass, current tree against the 5a pin:

| axis | tw | ir | | axis | tw | ir |
|---|---|---|---|---|---|---|
| strings | +19.66% | +32.58% | | dispatchclass | +8.27% | +8.48% |
| rexxcps | +13.04% | +16.36% | | compound | +7.85% | +11.26% |
| alloc4c | +9.01% | +13.11% | | arith | +3.23% | +2.34% |
| emptyloop | -1.60% | -2.93% | | varlookup | -0.91% | -1.84% |

**Moritz's ruling, 2026-09-01.** The drift was visible throughout 5a; every per-task gate reading was
honest and under half a percent; twenty-odd such steps compound to this. It is not any one task's
defect and no gate is widened now. **A non-SDD performance iteration round follows the completion of
Phase 5** and owns it.

The general lesson, which is not ooRexx-specific: **a per-step threshold that always passes still
permits unbounded cumulative regression.** The guard was doing exactly what it was designed to do and
could not have caught this, because nothing in it ever compares against a fixed distant point --
which is what a pin is, and why letting one go 153 commits stale removes the only instrument that
would have shown the sum.

**A confound this reading carries, recorded so the later round does not inherit it as fact.** The two
binaries were built with **different compilers**: the 5a pin with `rustc 1.97.1 (2026-07-14)`, the new
one with `rustc 1.98.0 (2026-08-18)`. No share of the table above is attributable to code rather than
to the compiler until `15a1ffa98` is rebuilt with 1.98.0 and one axis re-run against the 1.97.1 pin.
`PINNED.md` carries that experiment.

**Re-pinned** at `f558ea501`, sha256 `857787f797e88dd94ab839d7b66f1e08cd1a08d22c9e7978d05cbfd49f70494e`,
with `phase-5b-arms.tsv` started empty for the remaining tasks' sittings.

**A tool side effect worth knowing.** `rexx-arms` **appends to the `--baseline` file**. The first
drift run aborted on a missing bench program (`rexxcps.rex` lives in `bench-rexxcps/`, not
`bench-programs/`, so it needs a path axis and not a stem) and wrote nothing; the second appended 3
of 8 axes to the tracked `phase-5a-arms.tsv` -- a partial sitting in the canonical record that reads
exactly like a complete one. Removed by filtering the `5b-drift` rows out and confirming the diff
against HEAD was empty, then written to its own file. **Pass `--baseline` a file you intend to
write to.**

## Task 5 second fix round closed (2026-09-01)

`task-5-fix2` landed `3148d8cdd`; controller follow-up in `3290d5763`. Tree clean, all five gates
green at the fix commit, phase gate 101 by design, `obdes` still `agree`, corpus 286 to 287 of 287
matching.

**IMP-A closed, and the controller's two-half split reproduced by a third party.** MF (guard deleted
alone) changes nothing on four probe shapes or either engine; MSET (the `processing_uninits`
assignments deleted, guard kept) gives `inner ran inline? 1` at rc 0 with empty stderr on both
engines. The row is `corpus/lang/uninit_nested_collection_at_exit.rex`.

**The best finding of the round is one nobody asked for: the padding is load-bearing.** The
implementer kept the controller's probe's filler variables and then tested whether they mattered. A
control running the same finalizer body as an ordinary class method answers **1 padded and 0
unpadded on the oracle**, ten runs each. So the smaller row -- which two candidate designs preferred
-- would have *agreed*, would still have reddened under MSET, and its comment would have been false:
the oracle would print 0 because it never collected the instance at all, while the crate printed 0
because the interlock refused. Same verdict, different reason, and no gate could tell them apart.
That is "green over nothing" caught one level deeper than this phase has caught it before, and it is
the answer to the question the IMP-2 shape kept posing.

**The guard half: no witness found, established with a live instrument rather than a deletion.** The
guard body was replaced with a `panic!` and run against all 286 corpus programs plus eight probes
built to provoke re-entry (external routine call, `INTERPRET`, `EXIT` and `RAISE` in a finalizer,
class-side finalizer, unforced collection, D59 class-scope stash). It never fired. The same
instrument on `run_ready_uninits` fires on `uninit_nested_collection.rex`, which is what makes the
negative readable. Guard not deleted.

**Controller rulings.**

* **A cfg(test) test asserting the guard's contract: yes**, dispatched as one more commit. The
  implementer's refusal to write "no corpus row witnesses this" as a *source comment* is correct and
  was affirmed rather than overridden -- that sentence is a mutable in-repo aggregate, which
  `CLAUDE.md` forbids in a comment, and it rots the moment a row is added. A test asserts the
  contract instead of describing the repo.
* **The stale rustdoc link and the spec citation were the controller's to fix**, done in
  `3290d5763`. `registry.rs:527` documented the twin of `lookup_class_method_from_scope` as
  `lookup_instance_method_from_scope`, which `f558ea501` renamed to `lookup_from_scope_at`. The spec
  cited `Heap::clear_uninit`, which was *deleted* rather than renamed. Behaviour-neutrality was
  measured, not argued: the release binary rebuilt with both edits has the same sha256 as without
  them, after a real recompile.
* **The gate-5 count caveat is resolved and benign.** `^running` sections number 103 and
  `^test result:` lines number 103, so every section reports and nothing ran silently. The
  92 + 10 = 102 came from counting *headers*: one `Doc-tests` header covers two sections.

**Open, and not this task's:** two unresolved rustdoc links (`MethodDict::merge`,
`MethodDict::replace_methods_from`) that resolve to functions which exist, fail for a different
reason, and predate this phase.

## Task 5 fully closed; the controller's own ruling was the thing corrected (2026-09-01)

`f65694c8b` adds the guard-contract test. Gates green, phase gate 101, `obdes` agree, 287 of 287
matching, tree clean.

**The implementer corrected the controller's ruling, and was right.** The ruling said to "assert it
answers empty". That would have shipped **the exact vacuous witness this phase has hit four times**:
an empty `Vec<Loud>` is also what a sweep that ran everything successfully answers, so `is_empty()`
alone passes over an implementation with no guard at all. What separates the two cases is whether the
sweep **consumed** its work, since `take_uninit_flagged` and `take_uninit_classes_in_sweep_order`
drain. The test has three arms: the setup really leaves a class pending (so neither arm below is green
over an empty registry); interlocked, the sweep answers empty **and the class is still there**;
un-interlocked, the same call consumes it. The third arm is what gives the second its meaning.

A ruling from the controller is not exempt from the failure mode the controller keeps ruling against.

**Verified independently by the controller**, two builds in `git archive` extracts with their own
target dirs:

| mutation | the unit test | `uninit_nested_collection_at_exit.rex` |
|---|---|---|
| guard deleted | **FAILED**, `left: 0 right: 1` -- the registry drained | unaffected |
| assignment deleted, guard kept | ok | **`inner ran inline? 1`** on both engines, oracle says `0` |

So the two instruments are complementary and neither is redundant, which is the whole justification
for keeping both. The implementer measured this too rather than inferring it.

**One thing the ruling assumed wrongly and the implementer fixed properly.** `install_directives`
alone does not build a usable `Interp`: the un-interlocked arm panicked at
`index out of bounds: the len is 0 but the index is 0` on `self.programs[installed.program.0]`,
because the `ProgramId` handed to it has to name a program `Interp::run` pushes first. The helper now
does the same in the same order -- completing the documented setup rather than working around the
type, which is the right side of that line.

**Comparing failing sets rather than counts.** Its "nothing else catches it" check found 31 failing
tests with the guard intact and 32 without, the extra one being this test -- the 31 being the extract
having no `ootest/` or `oodocs/`, which are git-ignored working copies at the repo root. Comparing
sets rather than a count against zero is what made that readable.

Task 5 is closed: delivery, review, two fix rounds, both halves of the interlock witnessed.

## Task 3 landed; usesem agrees; the recurring gate flake fixed (2026-09-01)

`task-3` landed `4e613c349`, `8bcf6375e`, `d4fac6707`. **Verified by the controller's own phase-gate
run** at `d4fac6707`, exit 101 by design: `usesem` reads `agree`, table C 5b goes **2 not-agree to 1**
(`methodsbyclass`, Task 8's), table D 5b stays 0, 5a stays 0 on both. 5c also fell **941 to 912**
not-agree, which this task does not own and did not claim.

**The silent wrong answer it found was re-measured here with a different program.** A one-off
`UNINIT` attached from inside a method now fires, and a probe the controller wrote -- a second
instance of the same class that never attaches one, so a leak to the class would print twice --
agrees byte for byte with the oracle on both engines, rc 0 and empty stderr.

**A premise in the brief was false, and it was the controller's error.** The brief said
`compile_method_source` "already turns a source string into a method object for `~define`", carried
from the plan and presented as re-measured. Only its *existence* was checked. Its doc comment six
lines above the signature, at the very commit that was checked, ends "for a body no send ever
reaches" -- it validates and discards. Filing the parse as a program of its own turned out to be the
largest single piece of the task. **Verifying that a symbol exists is not verifying what the sentence
about it says**, and the cheap half was checked while the expensive half was passed through on the
plan's authority.

### The gate flake, fixed rather than noted again

`builtin::datetime::tests::time_e_does_not_reset_the_anchor_time_r_does` failed gate 5 for the
**third** recorded time -- 5a Task 14, the gate-close plan's Task 4, and now here at
`e2 = 0.161732, e3 = 0.230071` on a run carrying a 15-minute load average of 28.19. It cost this task
a whole gate-5 rerun, and seven 5b tasks remain, each gating.

The claim was right and the instrument was wrong: two equal burns and `e3 > e2 * 1.5` is the right
assertion only while both burns are scheduled alike. Fixed in `76a669c24` by making the burns
lopsided, the first 200 times the second, and asserting `e3 > e2` -- which under an unmoved anchor is
**monotonicity** rather than a timing coincidence, while a reset makes `e3` measure the small burn
alone.

**The margin is the evidence.** Against a control that makes `E` reset the anchor, built in an
extract with its own target dir: `e2 = 0.091873` against `e3 = 0.000259`, a factor of **355** where
the old form separated the cases by 1.5. It fails on that control and passes without it. Twenty runs
with 32 spinners resident all passed, recorded as *support* and not proof -- load 28.19 was not
recreated.

Test-only: the release binary is byte-identical either way after a real recompile. fmt, clippy and the
release workspace suite are green on the committed state; gates 4 and 5 were not run and are not
claimed.

### Left open by Task 3, all loud, none silent

`methodsbyclass` (Task 8). Item A is now **a one-line change for Task 9**, since a body exists to
copy. `~copy` is unimplemented, so the spec's "a copy carries the object's own scope" has no witness.
`self~setMethod` on a class object is rc 120 where the oracle is rc 0. `setMethod('MM', .nil)` is
rc 120 where the oracle raises 93.974. One synthetic program per compiled source is never reclaimed,
the shape `hold_method_object`'s doc already records for `~define`.

## Task 3 reviewed: one Critical, verified by the controller (2026-09-01)

`task-3-reviewer` returned 1 Critical, 3 Important, 6 Minor over `f65694c8b..d4fac6707`, having
re-run fifteen mutation arms rather than trusting the report's table.

**CRIT-1 reproduced independently, with the controller's own probe**, both engines, three descriptors,
rc 0 and empty stderr on every side:

```
t = .StringTable~new ; t['MM'] = "return 'enhanced'" ; e = .K~enhanced(t)
oracle           a enhanced / b one-off / c enhanced   / d 1
ir, tree-walker  a enhanced / b one-off / c from-class / d 1
```

`c` is a **silent wrong answer**: `unsetMethod` deletes an enhanced method where the oracle reveals
it. `Class~enhanced`'s methods share the per-object dictionary `setMethod` writes, so the crate has
**two levels where the oracle has three** -- the object's own one-offs, then the enhanced behaviour in
a dummy subclass, then the class. The comment justifying the storage quotes `ClassClass.cpp:1457`
about the `.nil` SCOPE, which the crate gets right, and not about which dictionary holds them: a true
citation supporting a different claim than the one it is attached to.

**IMP-1 is the fifth instance of the green-over-nothing shape**, and this one is the purest yet: the
row's own comment says "the send is 97.1 even though the class defines the method" and **the row never
makes that send**. IMP-3 is a sixth. Both were found by running arms, not by reading.

**IMP-2, the unreclaimed synthetic program, measured rather than noted.** `maxrss` under
`REXX_ENGINE=ir`: empty program 16,524 KB; 20,000 `setMethod` under distinct names 385,032 KB against
the oracle's 111,904; **20,000 under one name, each replacing the last, 383,452 KB against the
oracle's 20,588, which is flat**; 20,000 re-attachments of one pre-built `Method` object 18,444 KB,
flat. About 18.4 KB per compiled source, linear at three sizes, and the fourth row is the control
naming `compile_method_source` rather than the per-object dictionary as the cost.

**Two things the reviewer did that are worth keeping as method.** Its wrong-reason pass on the eight
rows the report had not checked produced a sharper result than a redden: with the access scopes
unfiled, `setmethod_private_refusal` does not merely fail, it produces *the other refusal* (98.991 at
rc 158 with the frame) where the oracle has 97.2 at rc 159 with none -- so the row is green because
the private check fires first, which is D66's whole claim. And it **withdrew a finding of its own**:
an "enhanced UNINIT runs at the wrong time" that turned out to be the oracle's `SaveStackSize` window,
which the same probe reproduces on the parent build with a plain `~new` object and which padding
clauses make agree.

Fix round dispatched at `e30055b37`. The brief tells it to read the C++ itself rather than take the
review's citations as premises, and to answer IMP-2 with either a reclaim or an explicit licence
carrying that measurement -- not to leave it as "never reclaimed".

## Task 3 fix round closed (2026-09-01)

Four commits to `ccc6dbd13`, tree clean. **Controller's own phase gate at that commit**: exit 101 by
design, `usesem` `agree`, table C 5b **1 not-agree** (`methodsbyclass`, Task 8's), table D 5b 0, 5a 0
on both. CRIT-1 verified with the probe written before the round, both engines, three descriptors, rc
0 and empty stderr: `a enhanced / b one-off / c enhanced / d 1` on all three sides, and the simpler
face (`unsetMethod` on a name only `enhanced` provided) now leaves `hasMethod` at 1 where it read 0.

**The round corrected the review, and the correction is right.** The review's IMP-2 fourth row --
"20,000 re-attachments of one pre-built `Method` object stay at 18,444 KB", offered as *the control*
identifying `compile_method_source` as the cost -- **could not have run on this crate**. Verified
here: `.K~method('SPARE')` handed to `setMethod` is rc 120,
`a one-off method whose body this crate does not hold`, against oracle rc 0. There is no route to a
pre-built `Method` object, so the figure is from something other than what it names. The conclusion
survived because the fixer replaced the control rather than deleting it: 20,000 of the no-method form
under one name is 17,192 KB and flat, and a forced collection every thousandth iteration moves
382,788 KB only to 377,044 KB, so the cost is `Interp::programs` and is not collectable garbage.

**A reviewer's measurement from a route that does not exist is a new shape for this phase.** Prior
instances were unwitnessed rows and prospective result lines; this is a control that reads as run,
carries a plausible figure, and names an operation the crate refuses at rc 120. The tell was
available cheaply -- one probe -- and nothing in the review's own text flagged it.

**IMP-2 licensed rather than reclaimed**, with the figures recorded at `Interp::record_compiled_body`:
about 18 KB per compiled method source, unbounded, superseded or not; a program that never passes a
source string pays nothing. Reclaiming needs a refcount or a sweep over `programs`, because a
`ProgramId` is both a `Vec` index and half a `BodyKey` and a one-off can replace itself while running.
That is a decision to take on the figures, not a fix round's work.

**Two method notes worth keeping.**

* **A void reading, recorded rather than smoothed.** Re-running the base baseline straight after a
  `cp -a` from a pristine copy printed 298 of 299, because the restored mtimes predate the mutated
  build's artifacts and cargo rebuilt nothing. `touch` and re-run gives 299 of 299. The eight arm
  readings are unaffected, each log showing its crate compiling. Same family as
  "a stale binary outlives its revert".
* **The sitting's cycles column was chased instead of reported.** Instructions are under 0.1%
  everywhere. Cycles had `dispatchclass` several percent slower against the pin -- but the earlier
  sitting's rows in the same file read *faster*, and comparing two sittings is not a measurement here.
  Interleaved on a quiet machine, base against head was identical in instructions and +3.2% to +5.9%
  in cycles. The do-nothing control -- BASE plus two `pub fn`s nothing calls -- moved the same axis up
  to **4.3% the other way**, also at identical instruction counts. Layout, not work, measured rather
  than argued. Neither control wrote to `phase-5b-arms.tsv`.

**A correction the controller owed the agent.** It paused reporting "load from other agents on the
machine". There were none: the 3082% CPU process was its own gate 5's `ir_dual`, using about 30 cores
because the sweeps were parallelized this morning. Gate 5 went from roughly one core to thirty, so an
agent carrying the old model reads its own run as contention. Told, and told not to record it.

**Left open:** `methodsbyclass` (Task 8); the licensed program retention; `~copy`; and four loud
divergences, one new to any list -- `setMethod` with a `Class~method` `Method` object as its second
argument, oracle rc 0 against rc 120 here, which is the same refusal that falsified the review's
control.

## Task 4 landed: FORWARD and DELEGATE, and two rows that now mean something (2026-09-01)

Four commits to `8e891cd5a`, tree clean, five gates green. **Controller's own phase gate** at
`1aa54204b`: exit 101 by design, table C 5b **1 not-agree** (`methodsbyclass`, Task 8's), table D 5b
**0**, 5a 0 on both.

**The central claim verified rather than accepted.** Both table D rows now send: the attribute probe
drives setter and getter through the delegate (`main via-set`) and the method probe reads `main 8`,
and oracle, `ir` and `tree-walker` agree on both. The task's own control is the right shape -- an
extract with `send_to_delegate` made not to send reads `diverge-stdout` on the *replaced* probes and
exits 101, while the same build with the *pre-Task-4* probes restored reads `agree`/`agree` at exit 0.
That is the demonstration that the replacement earns its place, and it is the first time this phase a
task has proved a row's replacement rather than just its addition.

**All six `keyForward` options are built and each has a witness**, so the phase owes none to a later
task -- which was the plan's explicit instruction and the reason the controller measured all six
before dispatch. The task's `CONTINUE` reading is rc 0 where the brief's table said rc 165; both are
the same behaviour through different probes, rc 165 being a continued forward with nothing after it.

**The spec was green over a wrong implementation, and the controller fixed it** (`1aa54204b`). D62's
probe used `d = 'abcdef'` expecting `len 6`, but `'length'` -- the directive's own name -- is also six
characters, so a build reading the directive name instead of the delegate symbol printed `6` and the
row agreed. Measured by the task in an arm: `'abcdef'` agrees under the wrong implementation,
`'abcdefgh'` diverges. Both spec sites now use the eight-character string, which no name in the
program matches. **The prose between those two sites already names this shape** -- "a witness whose
program stops one send short of the thing it names" -- so the document described the trap in the
paragraph above the one it fell into. The committed corpus row already used the discriminating value;
only the spec was stale.

**`dire.xml`'s stated equivalence is not observably exact.** `expose d` plus `forward to (d)` leaves
its own clause on the traceback where `DELEGATE` leaves none, measured over one failing inner method
at rc 214 both sides. Built as a `GeneratedKind::Delegate` instead of the literal expansion, with
`delegate_no_frame.rex` and `forward_frame.rex` committed as that pair. The plan's Build paragraph
said the wrong thing and the task corrected it.

**One divergence found by probing past its own rows**, and it is the thing a reviewer should look at
hardest: a non-continuing `FORWARD` after a `reply` that carried a value is rc 0 with the oracle
printing an `98.937` report on stderr and rc 0 with **empty stderr** here -- same status, same stdout,
a whole report missing. Fixed in `acb015bd4`, with the bare-`reply` arm beside it as the adjacent
success that pins the rule to the replied *value* rather than to the keyword.

**One sitting is owed and could not be taken.** The first, at `d8e48d353`, is recorded and clean:
`instructions:u` under half a percent on all eight axes, loudest `dispatchclass` ir large at +0.379%.
The second aborted because **this machine's PMU stopped scheduling `cycles:u` and `instructions:u`
together** -- confirmed independently by the controller at load 1.72 with no competing `perf`
process: five attempts, the pair fails every time, each event counts alone. This is the same shortage
recorded against `c7345f7a9`, which a reboot cleared. Nothing was written to the baseline. The change
is one bool store per activation and `size_of::<Activation>()` is 384 before and after, read out of
the compiler in two extracts -- a bound, not a substitute. **Retry when the PMU recovers.**

**Two commit-message corrections the task recorded rather than amended:** `164d5c106` claims plan
corrections that are actually in `d8e48d353`, and `acb015bd4` cites `RexxActivation.cpp:1370`-`:1374`
where the check is at `:1367`-`:1369`.

**Also found, both on Task 9's list:** `.Array~of` is unimplemented (oracle rc 0 against rc 120), and
a translation error inside `INTERPRET` reports one traceback clause fewer than the oracle --
established pre-existing two ways, on a sibling refusal with no `FORWARD` in it and on the
pre-Task-4 pinned binary.

## Task 4 reviewed: three Criticals, two silent, both reproduced here (2026-09-01)

`task-4-reviewer` over `ccc6dbd13..8e891cd5a`: 3 Critical, 2 Important, 2 Minor, 1 observation. It
re-ran every arm the report claimed and **all of them reproduce** -- so the task's controls were
honest and the gap is coverage past the rows, which is a different failure from the ones this phase
has been finding.

**CRIT-2, reproduced by the controller with its own probes.** A condition raised by a non-continuing
`FORWARD`'s send is trapped by the *forwarding* method; the oracle unwinds past it.

```
loud    oracle rc 214, stdout empty, stderr 321 bytes (three-line traceback)
        crate  rc 0,   stdout `a INNER-handler`, stderr empty
silent  oracle rc 0, stderr empty, `a OUTER-handler`
        crate  rc 0, stderr empty, `a INNER-handler`
```

One word of stdout apart with matching status and empty stderr on both sides. **The mechanism is the
same phantom the SIGSEGV entry recorded this morning**: `setForwarded(true)` runs before the send, and
`RexxActivation::trap` reads that flag first and drills to the previous non-forwarded frame. A
forwarded activation is a phantom for *condition delivery* as well as for its result -- one C++
decision surfacing as two unrelated-looking defects, a crash shape and a silent trap.

**CRIT-3, also reproduced here.** `forward message('SEEN') arguments (s.)` over a stem is oracle
`a seen 4` against crate `a seen 1`, rc 0 and empty stderr both sides. `forward_arguments` reuses
`operator_operand_gap`, whose predicate is "what no operator can take" rather than "what
`requestArray` cannot answer"; the stem arm is where those differ and nothing refuses.

**CRIT-1** is loud but wrong: `FORWARD CLASS (x)` with a non-ancestor skips the scope-override
validation the equivalent `o~m:.Other` performs correctly on all three sides. `Interp::message_term`
asks two checks, `Interp::exec_forward` asks one.

**OBS-1 questions a controller ruling and is treated as such.** Under mutation arm M6,
`forward_after_reply.rex` did not terminate -- rc 137 after a 60-second kill, measured crate-only,
where the unmutated build is well under a second. The licensed divergence, which is the controller's,
says this crate answers the self-forward shape with a clean rc 245. **If a member of the family hangs
instead, that licence is wrong rather than narrow.** The reviewer refused to construct the unmutated
forbidden program to settle it, which is correct, and the fix round is told the same: investigate on
the crate alone, reach the shape by routes that are not the forbidden literal, and say plainly if
anything hangs.

**IMP-1 is a record defect rather than a behaviour one** -- `acb015bd4`'s `98.937` row is missing from
the report's arm table, which stops at `d8e48d353`'s eleven. The reviewer ran the two missing arms and
both behave, so the fix round records them rather than re-deriving them.

Fix round dispatched at `c9c730ba7`.

## Task 4 fix round closed; the controller's licence was wrong (2026-09-01)

Three commits to `5ad238d81`, plus the controller's spec alignment at `5912d29ce`. Tree clean, all
five gates exit 0 (`315 of 315`), **controller's own phase gate** at `5912d29ce`: exit 101 by design,
table C 5b **1 not-agree** (`methodsbyclass`, Task 8's), table D 5b **0**, 5a 0 on both.

**OBS-1 is real and the licensed divergence was wrong as written -- that licence is the
controller's.** Verified here, crate alone, oracle never invoked, and the boundary is sharper than the
round reported:

| shape | result |
|---|---|
| self-forward, no `reply` | rc 245 in under 0.1 s -- the licence holds |
| self-forward after a **valued** `reply` | 98.937, terminates |
| self-forward after a **bare** `reply` | **does not terminate**, rc 137 at a 30 s kill, both engines, both descriptors empty |

**The controller's first probe used a valued `reply` and did not hang**, which nearly produced a
"does not reproduce" ruling against a true finding. `acb015bd4`'s own 98.937 check fires first and
short-circuits the shape -- one of this task's fixes masking another of its findings. Only the bare
`reply` reaches it. Not deep recursion and not allocation: `VmRSS` flat at 17,760 kB, a `gdb` stack of
24 frames that does not grow, the reply queue refilling as fast as it drains so
`MAX_ACTIVATION_DEPTH` never fires. Present at BASE. The round correctly **invented no bound**, since
every member is a shape the safety rule forbids handing to the oracle, so a cap would be behaviour
with nothing to validate it.

**A premise in the controller's brief was false.** It told the round that the oracle's stem tail order
is hash order. It is not: it is a walk of `CompoundVariableTable`'s balanced tree, deterministic
across five runs and dependent on assignment order. `Body::Stem` holds tails in a `HashMap` with no
insertion order, so reproducing it is a representation change to the interpreter's hottest structure.
The round measured this rather than taking the brief's word, which is why its witness compares
`arg()` only; the ordering divergence is on Task 9's list with its cost.

**Two Criticals were wider than the review had them.** CRIT-1 also hits the `CONTINUE` path and
validates the scope against the `TO` target rather than `SELF`; CRIT-2 has a third face, a caller's
`signal on nomethod` over a forwarding callee, needing `willTrap`'s drill as well as the phantom flag.
A fourth silent divergence was found on CRIT-3's predicate: a multi-line string as `ARGUMENTS` was one
argument here against the oracle's two.

**A second gate row green over a wrong implementation**, found by an eighth arm: the
`::ATTRIBUTE ... DELEGATE` row set the attribute and read it back only through `peek`, so it exercised
the delegating *setter* and never the *getter*. A build registering a plain getter beside the
delegating setter left the corpus at 315 of 315 and table D at 0 not-agree. The row now sends
`k~at` as well; the controller aligned the spec's copy at `5912d29ce`. **That is the second time
D62's probes were green over a wrong implementation and the two failed differently** -- `'abcdef'`
produced the right bytes from the wrong variable, this one never made the send at all.

**Open, and the controller's to decide: the differential harness bounds only the oracle.**
`ORACLE_DEADLINE` kills the oracle subprocess; the crate side runs in-process through `run_program`
with no deadline at all. Since the three hot sweeps are now rayon-parallel, one hung case blocks the
whole `collect` and stalls gate 4 or 5 indefinitely rather than reddening a row -- and a reachable
crate-side hang is no longer hypothetical. Put to Moritz with the options.

## Task 6 dispatched, and Moritz ruled the fifth divergence (2026-09-01)

Dispatched at `6eb234598`, brief `task-6-brief.md`, opus, in the worktree. No second implementer runs
beside it.

**Both divergences were re-measured by the controller before dispatch and both reproduce, silent** --
rc 0 and empty stderr on every side, stdout differing only in the position of one line. Divergence A,
D59a's 5b row, needs a metaclass carrying an instance-side `UNINIT`, because `~setMethod` on a class
object from a program context is `97.2` at rc 159 (Task 3's restricted-private check, reached again).
Oracle `before / class uninit / after` against both crate engines' `before / after / class uninit`.
Divergence B reproduces at the plan's stated threshold: 0..8 padding clauses diverge, 9 agrees.

**The window counts allocations, not clauses, which is new and it decided the task.** Padding of
`zI = 'p'||'q'||'r'||'s'` moves the threshold from 9 clauses to 3. So a ten-deep hold cannot buy
agreement -- reproducing the threshold would need this crate to allocate arena objects one-for-one
with the C++ interpreter's.

**Moritz's ruling: license it, do not build the hold** -- "anything that depends on gc ordering is
inherently unpredictable, and we shouldn't aim for equal behavior. it just needs to be one of the
many correct orderings." **The controller's argument was right and its reason was wrong**, which is
the shape this project keeps hitting: "exact agreement is not available" is a shortfall, and the
ruling is that collection timing is not a specified observable at all, so both orderings are correct
answers to the same program. That reasoning is stronger, it is durable, and it applies to Divergence
A too. The allocation-counting measurement survives as a supporting fact rather than as the basis.

**The boundary was sent with the ruling, because "any correct ordering" is one careless reading away
from swallowing D69.** The licence is over ordering only: the finalizer must still run, after the
object is unreachable and no later than termination. It does not reach a `UNINIT` that never runs
(D69's subject -- an absence is not an ordering), one that runs while the object is still reachable,
or any rc, stderr, or other stdout difference. D59a's own risk table names "the licence widens by
drift"; that sentence is the mitigation.

**A tension the row has to resolve in its own prose**: the licence says the crate's ordering may
legitimately change, and the witness asserts exact bytes on both sides so nothing moves quietly. Both
are wanted, so the DEVIATION and the test doc say that a change to the crate's ordering is a
deliberate edit to the row, where a change on the oracle's side or in rc or stderr is a real finding.

**A near miss worth keeping.** The controller's first two reconstructions of Divergence B did not
reproduce it, and nearly produced a "the plan's table is wrong" ruling against a true finding. Both
put a `say` between `~new` and `drop`, or inside `::method init`; either allocates enough to evict
the object from the ten-slot save stack and the oracle then agrees at every N. Same shape as Task 4's
valued-`reply` probe: a probe one clause off the recorded shape measures something else and reads as
a clean negative. `corpus/lang/uninit_instance_collected.rex`'s header comment is the authority on
the shape.

## Task 6 closed (2026-09-02)

Committed at `09d2e2e57`, three tracked paths, 665 insertions, no `Cargo.lock`. Controller's
comment-policy section landed separately at `cabc32418`; tree clean. All five gates 0 and the sha
pair matched across the run. Phase gate 101 by design: table C 5a 135 rows 0 not-agree, 5b 6 rows
**1** not-agree (`methodsbyclass`, Task 8's); table D 5a 36/0, 5b 2/0, 15 passed. Gates 4 and 5 both
`315 of 315 matching`. Every figure read independently by the controller from the run's own artifacts
before the report arrived, and the two readings agree.

**DEVIATIONS 5 and 6 in `phase-4-exclusions.txt`, witness
`rexx-exec/tests/licensed_divergences.rs`.** The file choice is justified in the report: the only
code reading that file searches its text for `KNOWN GAP:` markers, and `builtin_status.rs`'s
`excluded == 15` comes from `rexx-inventory` rather than the file, so a Phase 5 DEVIATION row cannot
redden a Phase 4 gate.

**Moritz's ruling reframed both rows and the documentation settled them.** `rexxref`'s `obdes`
section -- the one the gate row is named after -- and `rexxpg/classes.xml:447` both say only that
Rexx runs an `UNINIT` *before reclaiming the object's storage*, and neither says when reclamation
happens or in what order across objects. Both quoted from `svn info`-checked working copies at
r13198. So the rows lead with the language's own contract rather than with either the controller's
"exact agreement is not available" or the task's stronger corpus measurement, both of which are kept
as supporting facts. The boundary is written out in full in both rows, textually identical so a diff
shows drift: ordering only, the finalizer must still run, and an absence is D69's subject rather than
this licence's.

**The task's control beat the recommendation it was given.** Built as Control B, the save-stack hold
takes the corpus from `315 of 315` to **4 disagreeing** -- `uninit_instance_collected.rex`,
`uninit_nested_collection.rex`, `class_behaviour_snapshot_delete.rex`, `setmethod_uninit.rex` -- and
*inverts* the divergence, agreeing at N=8 and diverging at 9 and 10. One regression was predicted
from the allocation figures and four were measured; the report records both numbers.

**Two instruments in one task had descriptions written from what the check was *for* rather than
what it *does*, and the pattern is the finding.** The marker check was one-directional while its doc
claimed both halves were protected -- a deleted witness row and a typo'd marker were both invisible,
now closed by a `BTreeSet` equality with a four-row control table. The sha pair hashed three named
files while the claim made for it was about the tree. The second was resolved by narrowing the claim
and **naming the uncovered interval with timestamps** rather than widening mid-run, which would have
certified two disjoint intervals. Both checks were sound; the prose beside them was the defect.

**A detached gate run does not survive the turn that started it**, and this cost three restarts. One
died mid-gate-4 with three green lines on disk and no fourth, while the idle notice described it as
in flight -- indistinguishable from inside the turn. The controller's `ps` reading was challenged and
held on birth-time evidence. The inheritable fix is in the report: each gate's status appended to a
file so "a status nobody read" is not a state the run can be in, and a waiter that exits on the
process vanishing as well as on completion, so a second death reports itself instead of looking like
progress.

**The controller wrote into the plan file while the task was editing it** -- the third instance of
that hazard, and it went through the exemption that "docs outside the crate are safe". They are not:
every brief on this plan tells the implementer to correct the plan where it is wrong, so the brief
hands them the file. The rule is ownership, not location.

## Task 7 closed (2026-09-02)

Committed at `1f57c3d02`, 24 paths, 1291 insertions, 7 deletions, no `Cargo.lock`, tree clean. Five
gates all 0, read from the run's own status files. Gates 3, 4 and 5 each 104 `test result: ok` with
zero `FAILED`; gates 4 and 5 each `323 of 323 matching` -- BASE's 315 plus this task's eight
witnesses. Phase gate 101 by design, every figure reproducing Task 6's exactly.

**Ten native rows and a new `Primitive::Message` arm**: `Object~copy`, `Class~copy`'s 93.970 refusal,
`~run`, `~send`, `~sendWith`, `~start`, `~startWith`, and `Message~result`/`~completed`/`~hasError`.
Eight corpus witnesses, eight controls, each reddening exactly one witness and nothing else in the
323-program union.

**A Done-when criterion could not be met as written, found by the controller's pre-dispatch tree
read rather than by plan review.** The acceptance spelled the `send` witness as
`o~send(.Array~of('M', .Base))`. The behaviour is real -- measured, oracle rc 0, `plain sub` /
`array base` -- but `.Array~of` is not implemented and is Task 9's, so the witness would have been
red for a reason that was not this task's. The array literal `('M', .Base)` is an equivalent route
and answers identically on the oracle and on both engines at BASE. Plan corrected.

**Two defects the task found in its own work.** The message-name array's element count is `lastItem`
and not the slot count -- `('M',)` separates them, oracle 93.946 against the first build's 88.914,
confirmed independently by the controller at rc 163. And a claim about the oracle's duplicate error
report that had been written into the plan, a doc comment and the report *simultaneously*, falsified
by four runs and corrected in all three. A citation audit over every `interpreter/` line number the
task added found **seven wrong**, including `RexxClass::copyRexx` cited at `ClassClass.cpp:497` in
three places where it is `:166`.

**A third instance of the instrument-wider-than-its-coverage pattern**, after Task 6's two. The
tree-hash guard composes `git status --porcelain` with `git diff`; the first names an untracked path
without reading it and the second skips untracked files entirely, so the sixteen untracked files here
-- eight witnesses and eight sourceline expectations -- were **named by the hash and never read by
it**. What actually certifies them is mtimes: gate window 01:20:26 to 01:36:59, every one of the 24
paths older than the start, newest `dispatch.rs` at 01:19:38.

**The task's own named gap, partly closed by the controller.** It verified the 5c row *count* rather
than per-row, noting that a pair moving in opposite directions would not show. A verdict-line diff
between Task 6's and Task 7's phase-gate logs is **empty over the 226 rows those logs list**, with a
negative control confirming the comparison detects a single altered verdict. The logs list 226 of
1,567 rows, so for the remainder the per-phase counts remain the only instrument, and the residual
unwitnessed case is two rows of the *same* phase moving in opposite directions.

**The gate run finished green at 01:36 and sat uncommitted until 06:58**, when the controller read
the status files and woke the task. The status-file harness worked exactly as intended -- the result
was on disk and readable hours later, independent of any turn -- but nothing wakes an idle agent when
a detached run ends. Three of the controller's own background waiters were killed rather than firing
during this task, so polling on its own turns is currently the only reliable read.

## Task 8 closed: multidimensional Array, and the last red 5b row in either table (2026-09-02)

`dcd8b468d`, from BASE `1f57c3d02`. `corpus/gate-tables/concepts/methodsbyclass.rex` agrees on both
engines, so **table C 5b is 6 rows with 0 not yet `agree`** and table D 5b is 2 rows with 0 --
measured under the constraints file's own invocation at exit 0, both tables reporting
`gated by this run: 0 row(s)`. Five gates green: fmt and clippy at exit 0, the two release test runs
1936 passed / 0 failed, the debug `memcap 8G` run 1937 / 0, and the corpus harness **326 of 326
matching** in STRICT mode.

**`Body::Array` became a struct variant carrying `ArrayClass`'s `dimensions` field**, and
`rexx-exec` gained `.Array~new`, `[]=`/`PUT` and `~dimension`, with the subscript list validated the
way `validateIndex` and its two halves validate it: a read past a dimension answers `.nil`, a write
past one reshapes the array and moves what is already there, and the offset takes the **first**
subscript as the fastest-moving one.

**The plan's stated discriminator was false, and the falsification is a transcript rather than an
argument.** It said "`m[1,2]` and `m[2,1]` are distinguishable only under the right mapping". A
transposed mapping transposes the write and the read alike, so the pair swaps places together.
Measured with `multi_dimension_position` accumulating in reverse -- the whole mutation being
`.enumerate()` -> `.enumerate().rev()` -- the plan's own discriminator program is byte-identical to
the oracle on three descriptors and both engines, while the shipped probe diverges on exactly one
line. What discriminates is a reader of the slots in their own order; the probe carries
`~toString('l', ' ')`. Plan corrected in place, including its `Done when`.

**Both controls at exit 101, table C 5b at 1 not-agree**, with the row's own transcript in each: the
committed control (route the setter to a single-index `[]=`) moves the `element` line, and the
transposed mapping moves **only** the `order` line -- the `element` line is identical under it,
which is the same finding from the other side. Control 1 also settles the brief's BASE count: a
single red `methodsbyclass` produces exactly `5b: 6 rows, 1 not yet agree`.

**A comment claim was turned into an assertion mid-task, and the gate run was restarted for it.**
`.array~new(0)` carries a one-element dimensions array whose entry is not the extent; the body's
doc said so and nothing checked it. `array_multidimensional.rex`'s `grew` line now does --
`zero~put('v', 3)` then `~dimension(1)` is `3` where the stored entry is `0`. Mutation-checked:
dropping `native_array_dimension`'s `len() != 1` guard reddens that one program and leaves the 56
others green. The first gate run was stopped rather than left to finish, and everything re-run from
the beginning on the committed tree.

**A citation audit found three of this task's own `interpreter/` line numbers wrong** -- two of them
landing on the comment line directly above the call they named -- plus one pre-existing comment
naming `array_index.rex`, a corpus program that does not exist. All four corrected. The
tree-hash instrument reads the bytes of every tracked-modified and untracked non-ignored path and
says in the report which paths it does not cover.

**Named gaps.** `.Array~subclass('K')~new(2,3)` answers `6` on the oracle and is a loud refusal here
(a `Body::Array` carries no behaviour handle); `Array~dimensions` (plural) stays loud, with its
oracle answer recorded for whoever builds it; `extendMulti`'s product is bounded here at
`MaxFixedArraySize` where the C++ bounds it only in `createMultidimensional`, unmeasured against the
oracle because reaching it needs an allocation the oracle cannot make; and no performance sitting
was run.

**`condition('A')` is not implemented in this crate**, which is why the refusal corpus program
compares error *numbers* and two unit tests carry the substituted argument positions. That is a gap
worth an owner: it is the thing that would let a corpus program compare a substitution
differentially instead of a hand-transcribed assertion.

### Controller verification of Task 8, and a correction against the controller's own brief

`dcd8b468d` verified independently: 22 files, 1091 insertions, 79 deletions, no `Cargo.lock`, tree
clean. **The phase gate was re-run by the controller rather than read from the report** --
`REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test
gate_table_d --no-fail-fast` -- and exits **0**, 14 passed and 15 passed, 0 failed. **5b now has no
red gated row in either table.** The per-phase listing does not appear in a green run because cargo
captures a passing test's stdout; the exit status is the reading, since the gate is built to exit 101
when a gated row disagrees.

**The oracle's slot layout was confirmed independently**: `.array~new(2,3)` written at `[1,1]`,
`[2,1]`, `[1,2]` answers `p11 p21 p12` through `~toString('l',' ')`, so column-major with the first
subscript fastest, as the task reports.

**The task found the plan wrong, and the controller's brief had repeated the error as measured.**
The plan claimed `m[1,2]` and `m[2,1]` "are distinguishable only under the right mapping". They are
not: a consistently transposed mapping transposes the write and the read alike, so the pair swaps
together and each cell still reads back its own value. The task measured this rather than arguing it
-- with `multi_dimension_position` accumulating in reverse, the plan's own discriminator is
byte-identical to the oracle on three descriptors and both engines -- and the probe now carries a
reader of the slots in their own order.

**What the controller actually did wrong is worth naming, because it is the third instance.** The
brief said "the plan's discriminator program answers exactly what the plan states", and that was
true: the oracle's output was measured. The claim passed through unmeasured was the *discriminating
power* -- that the program could tell one mapping from another. A compound claim was checked on its
cheap half and labelled measured as a whole. Same shape as `compile_method_source` and the stem tail
order in Task 3 and Task 4's briefs.

**The acceptance this task was written to fix would have failed in exactly the way it was written to
prevent.** The plan's own diagnosis -- that the committed probe could not see the indexing -- was
right, and its proposed fix inherited the defect it diagnosed. Control 2 shows it from the other
side: a transposed mapping moves **only** the order line, with the element line identical to the
oracle's.

## Phase 5b closed (2026-09-02)

`a9bf7d023` "Phase 5b Task 10: close 5b" and `ad26cb95a` "Record Task 10's sitting against the Phase
5b pin". Tree clean. `CLOSED_PHASES` reads `&["5a", "5b"]` at `gate_tables/mod.rs:344`, verified by
reading the file at HEAD.

**The flip was verified by the controller running the gate tables with `REXX_PHASE_GATE` unset** --
`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d
--no-fail-fast` -- exit **0**, 14 passed and 15 passed, 0 failed, and the banner reads `gated for 5a,
5b`. So 5b's rows are gated for everyone with no environment variable set, which is the property the
flip exists to create and the one a green run cannot otherwise distinguish from a flip that gates
nothing.

**Task 10's own redden control is the instrument that separates those two**, and it fired both ways:
the same crate mutation under the same command is 101 with `"5b"` closed (`gated by this run: 2
row(s)`) and 0 without it (`0 row(s)`). It also proved the DEVIATION witness live by falsifying one
recorded oracle byte and watching the row redden on what the oracle actually printed.

**The sitting, nine axes against `rexx-run-f558ea501` with its sha verified before measuring,
interleaved.** Nothing reaches 1%: largest `+0.700%` (`varlookup` large, ir) and `-0.223%`
(`emptyloop` large, tw). The finding worth keeping is not any single figure but the sign pattern --
**every `ir` cell positive across all nine axes while `tw` is negative on six**. That is a consistent
direction no per-axis threshold can see, and it is the same shape as the drift PINNED.md already
records. Recorded for the post-Phase-5 performance round, not re-litigated here.

**`dispatchclass` stays, decided on a measurement rather than an assertion**: `ir/tw` is 0.951 on
`dispatch` and 0.990 on `dispatchclass`, so the compiled engine buys about 4.9% on an instance send
against about 1.0% on a class send. Two different dimensions, so retiring either would lose one.
`alloc4c` stays, since `alloc.rex` did not go live.

**A brief claim of the controller's was wrong and Task 10 corrected the plan.** Both said
`dispatch.rex`'s first figure would be a new baseline row rather than a comparison. It is a
comparison: `rexx-run-f558ea501` was built at Task 2's commit, *after* Task 1 unblocked the program,
so the pinned binary runs it at rc 0 -- confirmed independently by the controller running the pin.
The controller had verified the three parked programs at **HEAD** and passed through a claim about
what the **pin** did, which is the fourth instance this phase of checking one half of a conjunction
and labelling the whole sentence measured.

**Owed next**, and not part of this phase: the SDD workspace copies to
`docs/superpowers/records/2026-08-27-phase-5b/`, which today holds only `plan-review.md` and
`spec-review.md` against the workspace's fifty-one files. The `review-<base>..<head>.diff` packages
are excluded by the records README, but only after both endpoints of each are verified reachable --
that verification is part of the copy, not an assumption to inherit.
