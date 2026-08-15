# Task 12 report -- the Phase 4b exit gate

**Status: DONE.** Committed `a5a979ad` (the gate), `0e14fac4` (fix round 1) and `a1484211` (fix
round 2), all hashes read back with `git log` after committing. Tree clean.

**Created:** `docs/superpowers/plans/phase-4b-gate.md`, `rust/scripts/mutate-4b.sh`.
**Modified:** `docs/superpowers/plans/phase-4-exclusions.txt`,
`rust/crates/rexx-exec/tests/coverage.rs` (fix round, C1),
and `rust/crates/rexx-exec/src/run.rs` for one doc comment -- see "The one file outside the nominal
list" below.

---

## Fix round 2

All five findings measured before acting; **all five reproduced, none disputed.**

* **N1 (mine to carry, the lead's to originate).** "criterion 8's only named witness, and the
  declared corpus catcher for **three** of criterion 6's mutations" is false; **the true value is
  one.** Measured by applying each of the six corpus-catching mutations in turn: every one diverges
  on exactly one program (`41 of 42` in all six), so no program can be the catcher for more than
  one. Row 6 (`SIGL` off by one) names `lang/condition_traps.rex`; rows 1 and 7 cannot reach it.
  **This is C1's own defect repeating one round later** -- in the paragraph whose entire subject is
  a falsification clause that was reasoned instead of measured. The triple came from the previous
  review's C1 text and neither of us ran it. Corrected in `phase-4b-gate.md` and `coverage.rs`;
  **`0e14fac4`'s commit message carries it uncorrectably**, which is why it is recorded here.
* **N3.** "nine of them could be deleted with the whole suite staying green" is true only
  **distributively**. Measured collectively: removing all nine fails `coverage.rs` at exit 101,
  `3 in-scope variant(s) unwitnessed: Interpret, Signal, Raise`. The document now states which
  reading holds and what was actually run for which program.
* **N2/N4.** `b23986d9` touches **three** of the four `rust/` paths, not four (all but
  `extract_keyword.rs`); `8d6790c6`'s four `rust/` paths carry **406** insertions, not 480 -- 480 is
  the six-file total, the other 74 being `l1-coverage.md` (54) and `phase-4-exclusions.txt` (20).
* **I1.** `mutate-4b.sh`'s header still opened with the pre-amendment criterion 6 wording, so the
  contradiction the gate document claims to have removed survived one file over. Fixed, and the
  header now says its own previous sentence was the false one.
* **M5's second half, taken rather than skipped**, because criterion 4's central claim rested on it.
  `run_one` now prints the failing **target binary** for every suite divergence. Measured: row 9's
  is **`tests/collect_stress.rs` and nothing else**, row 8's is `tests/trace_oracle.rs` alone,
  row 10's is `unittests src/lib.rs` alone. "The collector is the instrument that sees a dropped
  argument-list root" is now an observation rather than an inference from test names.
* Minors: two hard-wrapped identifiers unwrapped so `grep` finds them; criterion 6's "five
  mutations **deliberately**" now separates the four by design from row 2, whose corpus survival
  this gate **discovered**.

## Fix round 1

All findings were verified independently before acting; **none was disputed** -- every one
reproduced. The largest is C1, and it was a real hole rather than a wording problem:

* **C1.** Criterion 1's falsification claimed a deleted `phase-4b.txt` entry would leave a variant
  unwitnessed. Measured one line at a time across all twelve: **nine of the twelve leave
  `coverage.rs` green.** Worst case `lang/condition_traps.rex` -- criterion 8's only witness --
  whose deletion left the corpus at `41 of 41 matching`, exit 0, with three criteria still
  reporting MET. **Fixed with the pin, not with a weakened claim**:
  `coverage.rs`'s `phase_4b_subset_matches_the_committed_list`, verified to fail by name. Workspace
  moved 1,019 -> 1,020 and **every figure was re-run**; only that count changed.
  *Why the false claim was plausible: it is true of `phase-4a.txt`, whose pin 4a's own branch review
  built, and it was carried to a second file without being re-run.*
* **I1.** Criterion 6's wording is contradicted by row 12 (I17's equivalent mutant, declared to
  survive both instruments, which the brief asked for). Amended so an unexpected **catch** fails as
  loudly as an unexpected survival -- which is what makes the equivalence falsifiable.
* **I2.** Two false sentences beside a true measurement in Step 3b. Two of the ten intervening
  commits **do** change `rust/` source (`8d6790c6`, 480 insertions; `b23986d9`), and **exactly one**
  -- not three -- corrected this step's framing. The empty six-file diff itself is real and
  unchanged.
* **I3.** The merged `EXCLUSIONS` row listed the six ooTest bodies twice, inside the row whose own
  ruling closes with "recording a gap twice is worse than recording it once". De-duplicated.
* **M5/M6.** Classifier guards hardened and **verified against the exact degenerate outputs the
  review constructed** (7 cases, all correct): `suite_status` now checks the run count **per target**
  and requires every target to have reported; `run_suite` gains `--no-fail-fast`; `corpus_status`
  rejects `0 of 0 matching`.
* **M1, M2, M3, M4, M7, M8** all corrected as reported.

---

## What was measured

Every command was run from `rust/`, with **each exit status read unpiped**. Measured three times --
at `4c8c1f68`, after the `run.rs` comment fix, and after the fix round -- with **only the workspace
count changing**, and only because the fix round added a test.

| command | exit | result |
|---|---|---|
| `cargo test --offline --workspace --no-fail-fast` | 0 | **1,020 passed, 0 failed, 4 ignored** |
| `cargo fmt --all --check` | 0 | clean |
| `cargo clippy --offline --workspace --all-targets -- -D warnings` | 0 | clean |
| `REXX_CORPUS_GATE=1 … --test corpus` | 0 | **42 of 42 matching** |
| `REXX_ASSERTIONS_GATE=1 … --test assertions` | 0 | **4,224 of 4,259 rows**, 35 RUNTIME-BLOCKED |
| `REXX_KEYWORD_GATE=1 … --test keyword_assertions` | 0 | **100 of 896 bodies, 713 of 1,773 calls** |
| `rust/scripts/mutate-4b.sh` | 0 | **12 of 12 as declared** |

`mutate-4b.sh` was re-run **after** the commit, against the committed tree: same result, tree clean
afterwards. The 42 is 30 entries from `phase-4a.txt` plus 12 from `phase-4b.txt`, no overlap
(checked with `comm`).

These reproduce the state verified independently in an isolated worktree, so the two agree.

## Step 1: criteria before measurement

The criteria section was written and saved before a single gate was run for this document. Ten
criteria: seven carried from `phase-4a-gate.md` with D14's amendments, three new (a condition-trap
criterion, a queue criterion, and the `base/keyword` policing criterion). Each carries an explicit
*Falsification* note answering "what degenerate implementation satisfies this, and would deleting
its subject leave it green?".

**Outcome: eight met; one met carrying an inherited criterion defect; one met weakly.**

* **Criterion 2** (`tests/assertions.rs`) is met carrying the same defect 4a recorded: the literal
  wording contemplates a row unblocked by 4b or 4c and cannot pass STRICT inside Phase 4 without
  `EXEMPT`. The criterion **predicted** its own result before measurement -- all 35 rows are Phase
  5's, so the figure had to be identical to 4a's, and it is (4,224 of 4,259). Saying so beforehand
  is what makes "identical" distinguishable from "stalled".
* **Criterion 3** (trace) is met weakly at **13 of 19** prefixes witnessed, a committed literal four
  assertions read -- the prefix set is checked against `support::TRACE_PREFIXES`, read from the
  oracle's own `trace_prefix_table`, so the fraction cannot be improved by dropping a prefix. The
  criterion states in its own text that DEVIATION 0 removed indentation from its reach, names the
  three pinned unnormalised `run.rs` witnesses that remain, and says where its power now lives
  (value-line content and line order, byte-exact).

The four 4b-specific vacuity shapes the brief named are all addressed in the criteria's own text:
the trap criterion asserts a value the handler set (never an exit code, and never a value that
could be a derived name); criterion 4 reads the union and uses an activation-shaped control; the
coverage criterion states the combinations limit itself; the queue criterion requires the gate to
record that the construct ships undifferentiated.

## Step 2: `mutate-4b.sh`

Reuses 4a's three guards unchanged in intent (`apply_mutation`'s exactly-once requirement,
`require_baseline_pass` before and after, and a `PASSED`/`DIVERGED`/`INFRA_FAILURE` classification
that aborts rather than scoring an infrastructure failure). **The mutations are all 4b's own** --
every `OLD` string is in code that did not exist before this sub-phase.

**The substantive change from 4a's script: two instruments per mutation, and every row declares what
both should say.** 4a's ran the corpus alone and treated its silence as a failure. That is the
wrong verdict for five of these twelve -- the dropped argument-list root, the queue's storage, the
queue's order, an omitted argument's `>A>` line, and a callee's inherited trap table are all
invisible to a differential run, and four of the five are invisible for a *correct* reason. The
suite path additionally **asserts a non-zero test-run count**, because `cargo test <name>` exits 0
when it matches nothing.

Includes both things the brief asked for: the **activation-shaped negative control** (row 9,
deleting `run.rs`'s `push_temp(argument.value())` -- the argument list's root between evaluation and
the callee's `USE ARG`; corpus stays at 42 of 42, the collector catches it) and **I17's
reclassification with its real mechanism** (row 12, run rather than cited, declared to survive both
instruments and doing so).

**The guard was attacked the way the branch review attacked 4a's.** `oracle_root()` repointed at a
nonexistent path: the script **exited 1 at the first baseline, before touching a mutation**.
`corpus.rs` was then restored from a scratchpad copy -- never `git checkout --` -- `git status`
confirmed clean, corpus re-verified at 42 of 42.

**Two declarations were wrong and were corrected to what was measured.** Both corrections are
visible in the script's own comments and in the gate document, because a script whose declarations
are edited to match its output is worthless unless the edits are visible.

**One mutation could not be assessed and the script refused to score it.** An earlier
`PROCEDURE`-shaped row pointed the callee's activation at the caller's frame while a fresh frame was
pushed; that trips a `roots.rs` invariant and panics `corpus_differential` **before** it prints its
`N of M matching` line. Classified `INFRA_FAILURE`, script aborted. 4a's first script would have
called it a catch.

## NEW FINDING: a callee's inherited trap table has no differential witness

Row 2 was declared DIVERGED/DIVERGED and measured **PASSED/DIVERGED**. Deleting
`let traps = caller.traps.clone();` leaves `REXX_CORPUS_GATE=1` at **42 of 42** and fails five
`run.rs` unit tests including
`a_trap_is_inherited_by_a_callee_and_fires_in_the_callees_own_activation`.

**Mechanism, checkable from the corpus programs' own text rather than argued:** every raise a corpus
program makes inside a routine is a `RAISE ... RETURN`, which unwinds the routine like a `RETURN`
before the condition is delivered -- so the trap that matches is always the **caller's own live
table**, never the inherited copy. `lang/call_on_trap_rearms.rex`'s own header states this about its
own `raiser`, and `lang/condition_traps.rex`'s block 4 is the same shape.

Nothing is wrong; it needs nothing from a later phase (every construct involved is 4b's and works);
it is an **unwritten witness** rather than a blocked one. Recorded as a `KNOWN GAPS` row, which the
file's own rules explicitly permit without amendment.

## Step 3c: the ruling

**Option 2. The compound-`DO` control-variable divergence is 4c's.** The row moved out of
`KNOWN GAPS` into a new `EXCLUSIONS -- a compound variable as a DO control variable, owned by 4c`
section, which makes it 4c's own gate criterion.

* **Not option 1 (fix it here):** `run.rs` and `rexx-parse` are outside this task's file list, and
  the recorded cost includes a `rexx-parse` signature change. Moving the tree in the commit that
  measures it would invalidate every figure above, with no review round behind it. The gate reports
  the tree 4b ships; it does not move it.
* **Not option 3 (open gap):** that is what all eleven previous tasks did, correctly, because `DO`
  was in none of their file lists. `phase-4-exclusions.txt` **is** in this one's, and it is where an
  owner is recorded.
* **4c, not Phase 5:** nothing here needs the object model, the resolution already exists in the
  crate, and 4c is the last point at which a Phase 4 construct's own defect can close inside Phase
  4. `instruction_owner` returns `None` for `InstructionKind::Do` -- implemented, not deferred.

**Two `KNOWN GAPS` rows described one defect, and the ruling merged them.** Task 9's (the `>C>`
symptom and the rexx-parse cost) and Task 11's (the six L1 bodies and the three narrowing probes)
were separate entries for the same `bind_control` mechanism. **The split is part of why it drifted
unowned:** either row read as the whole defect. A pointer stays in `KNOWN GAPS` naming both former
titles.

**Enforcement already exists and is automatic:** the six `defect:` rows in `keyword-exempt.txt` go
red the moment the fix lands, via `the_exempt_set_matches_the_current_failures`.

**`keyword-exempt.txt` deliberately does not change, and must not.** Those six rows' attribution is
derived from the **outcome kind** -- `RunOutcome::AssertionFailed` maps unconditionally to
`defect:compound-do-control-variable` in `keyword_assertions.rs` -- because these bodies *run and
disagree* rather than failing loudly with a phase-named message. Rewriting them to `4c` would
contradict the derivation the exempt file's own header states and turn the set assertion red
immediately. An owner records who owes the fix; it does not reclassify how the harness sees the
failure.

## Step 3b: recommendation, not acted on

**Keep the attribution. Spend the consolidation budget on the third copy's PROSE instead.**

| commit | harness | interpreter | fraction |
|---|---|---|---|
| `e4caa7bf` (mid-4b) | 1,852 | 9,025 | **20.5%** |
| `5b2de07a` (after Task 11) | 1,928 | 15,838 | **12.2%** |
| `4c8c1f68` (the tree assessed) | 1,928 | 15,838 | **12.2%** |
| after this gate's own commits | 1,976 | 15,841 | **12.5%** |

The fourth row exists because the tree assessed and the commits this gate lands in are different:
`bind_control`'s doc fix adds 3 interpreter lines and the fix round's new pin adds 48 harness lines.

Both brief endpoints reproduce exactly. **The third point is identical to the second because
`git diff --stat` over exactly those six files between them is empty** -- the ten intervening
commits are plan and documentation edits. Across 4b the harness grew **4%** while the interpreter
grew **76%**, which runs against the argument consolidation was raised on.

**What would change the recommendation at 4c's gate:** the three files exceeding ~2,600 lines (a 35%
jump against 4a-to-4b's 4%), or the fraction **rising** at all rather than falling to the expected
8-9%. 4c moves most variants *in* scope, which *deletes* `loud.rs` rows, so falling is the null
hypothesis.

**The third copy** is guarded (`every_out_of_scope_variant_fails_loudly` asserts stderr **ends
with** `" is not implemented (OWNER)"`, cross-checking `loud.rs`'s strings against `lib.rs`'s
messages). **The suggested fix is not a straight equality**: `owners.rs` is variant-grained and
ternary, `lib.rs` is arm-grained and binary, and `loud.rs` already carries a hand-maintained
reconciliation (`expand_for_witnesses`). Asserting one against the other makes that expansion table
the new single point of truth -- it *moves* the duplication. ~half a day, and it buys less than it
costs.

**The phase strings are load-bearing for exactly one consumer, and it is the file 4c's gate fires
on:** 790 of `keyword-exempt.txt`'s 796 rows derive `unblocked_by` by parsing the owner out of the
loud message, which is what makes that file impossible to drift from `instruction_owner`. Removing
the strings before 4c's gate would be the wrong order. Nothing else reads a phase string.

**The finding that should drive the decision:** the stderr assertion guards the **data**; nothing
guards the **prose about** the data, and the prose is where every rot happened -- `Call::Trap`'s
stale sentence in three files that Task 7's five review passes all missed, and
`INSTRUCTION_WITNESSES`' count comment going stale four separate times, each correction itself a
count the next task falsified. **So the consolidation worth costing is deleting the prose that
restates the tables, not merging them.** Cheaper, targets the copies that actually rotted, and
follows the rule the repo already has.

**Not acted on in 4b**, per the brief.

## Step 3d: the L1 result, reported and not gated

**1,773 of 2,441 exact-spelling `assertSame` calls extracted into 896 bodies (72.6%); 100 bodies
pass, carrying 713 assertions.** The remaining 796 are **790 `4c` + 6 `defect:`**, counted from the
committed file. The denominator's spelling is stated (2,441 exact; the prefix match gives 2,561 and
silently counts 120 `assertSameList` calls as dropped `assertSame` calls).

All three qualifications are beside the number: a `4c` attribution says what a body hits **first**
(four bodies are not even equivalent to the methods they came from -- three fail under the C++
oracle itself, one exits 3 through `dig: Return digits()`); **the 790 are an upper bound on what
landing 4c would fix**, not a measure of 4c's remaining surface; and `base/keyword` has **zero Phase
5 dependency**.

**No figure from `TRACE.testGroup` is used anywhere in the gate.** Criterion 3's own measured, named
subset (13 of 19 prefixes) is used instead, per the brief's I27 instruction.

## The one file outside the nominal list

`bind_control`'s doc comment in `rexx-exec/src/run.rs` said the defect was "recorded as a KNOWN GAP
in `phase-4-exclusions.txt`". **This commit's own change falsified it** when the row moved to
`EXCLUSIONS`. Corrected -- doc comment only, no behaviour -- rather than left for a reader to trip
over, per the standing rule that a false comment must be corrected rather than hedged. Every figure
above was re-measured afterwards and none moved. The alternative (4a's precedent: flag it and leave
it) would have meant shipping a knowingly false comment in the same commit that made it false.

I checked every other citation the move could have falsified: `run.rs`'s two other `KNOWN GAP`
references are to the indent-counter and Controlled-`>>>` rows and are unaffected, and
`keyword-exempt.txt`/`keyword_assertions.rs` cite the file only for owner strings.

## Concerns

0. **The gate claimed a protection it had not measured, and the review found it rather than the
   gate.** C1's falsification clause was reasoned across from `phase-4a.txt` instead of run against
   `phase-4b.txt`. This document's own criteria section says reasoning is what fails on this project,
   and the one clause that was reasoned is the one that was wrong. The lesson worth carrying to 4c's
   gate: **a falsification clause is a claim like any other and needs the same measurement as the
   criterion it protects** -- and "the previous sub-phase's file behaves this way" is exactly the
   shape of premise that has been wrong here before.

1. **`mutate-4b.sh` is slower than 4a's** -- twelve mutations x two instruments x a rebuild each,
   roughly ten minutes. That is the price of the asymmetric declarations, and I judged it worth
   paying since five of the twelve rows exist only because the second instrument does. Worth knowing
   before someone runs it expecting 4a's runtime.
2. **Criterion 1's combinations limit is stated but not closed.** Task 10 was dispatched to close it
   and its own review deleted two of three witnesses as redundant, which is an honest result rather
   than a failure -- but it means the gap 4a's whole-branch review exploited is still open in kind,
   and no instrument in this gate would find its 4b equivalent.
3. **The new trap-inheritance gap is cheap to close and I did not close it**, because writing a
   corpus program means capturing a fresh oracle expectation and moving criterion 1's own subset at
   the moment that subset is being measured. It is a good first candidate for 4c, or for a small
   follow-up before 4c starts.
