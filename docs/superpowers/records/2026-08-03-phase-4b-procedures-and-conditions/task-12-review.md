# Task 12 review — the Phase 4b exit gate

Reviewed at `a5a979ad` against `task-12-brief.md` and `task-12-report.md`.
Deliverables: `docs/superpowers/plans/phase-4b-gate.md`, `rust/scripts/mutate-4b.sh`,
`docs/superpowers/plans/phase-4-exclusions.txt`, plus a doc-comment-only edit to
`rust/crates/rexx-exec/src/run.rs`.

Every mutation and every corpus-list edit below was made in the working tree with a
scratchpad backup and reverted immediately; `git status` is clean and `run.rs`,
`phase-4a.txt` and `phase-4b.txt` are byte-identical to `HEAD`.

---

## Verdicts

**Spec compliance: MEETS THE BRIEF, WITH ONE REQUIRED FIX.**
All five steps and both sub-steps are discharged, in the order the brief demanded, and
the three artefacts exist with the content asked for. The four named 4b vacuity shapes
are addressed in the criteria's own text; criterion 4 carries a genuine
activation-shaped negative control (measured, below); the ruling in Step 3c picks one
of the three permitted dispositions and justifies the two it did not pick; Step 3d
reports the L1 figure with all three qualifications and gates on nothing; and no figure
from `TRACE.testGroup` appears anywhere. The required fix is C1: the brief's governing
instruction — *"for each of the ten criteria, ask whether deleting its subject would
leave it green"* — is not satisfied for criterion 1's own stated falsification, and the
gate document asserts a protection that does not exist.

**Quality: HIGH.** Nearly every number reproduces exactly, several of them by
independent re-derivation rather than by re-reading the report. The document is honest
in the direction that is usually flattered — it volunteers a wrong declaration, an
aborted mutation, a deleted-two-of-three combination result, and a comment its own
commit falsified. The defects below are prose that outran its measurement (I2, M1, M2,
M3), one internal inconsistency the gate's own standard would have caught elsewhere
(I1), and one duplication the ruling itself argues against (I3). None of the ten MET
verdicts is wrong on the merits; one falsification note is.

---

## Findings

### Critical

**C1. Criterion 1's falsification is false for `phase-4b.txt`, and 4b's twelve programs
are pinned by nothing. Deleting criterion 8's only witness leaves the whole gate
green.** `docs/superpowers/plans/phase-4b-gate.md:63-66`

The criterion states: *"deleting a program from either subset file fails
`phase_4a_subset_matches_the_committed_list` (for 4a's) or leaves a variant unwitnessed
(for 4b's)."* The second half is false. Measured, removing one line from
`rust/corpus/phase-4b.txt`:

| removed entry | `coverage.rs` |
|---|---|
| `lang/interpret_dynamic.rex` | green |
| `lang/interpret_error_echo.rex` | green |
| `lang/call_return.rex` | green |
| `lang/call_expression.rex` | **red** |
| `lang/call_procedure_expose.rex` | green |
| `lang/use_arg_forms.rex` | **red** |
| `lang/signal_forms.rex` | green |
| `lang/condition_traps.rex` | green |
| `lang/push_queue.rex` | **red** |
| `lang/raise_array_substitution.rex` | green |
| `lang/loop_retest_blame.rex` | green |
| `lang/call_on_trap_rearms.rex` | green |

Nine of twelve are held by nothing. The worst case is `lang/condition_traps.rex`, which
is criterion 8's *only* named witness and mutations 1, 6 and 7's declared corpus
catcher. With that line removed:

* `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` → **exit 0, "41 of 41 matching"**
* `cargo test -p rexx-exec --test coverage` → 9 passed
* `cargo test -p rexx-exec --test collect_stress` → 2 passed

So criteria 1, 4 and 8 all continue to report MET with criterion 8's subject deleted,
and the headline number silently shrinks from 42 to 41 while still reading as a clean
sweep. Criterion 4's falsification (*"the negative control is what proves the mode can
observe a missing root"*) and criterion 8's (*"All three are corpus divergences"*) both
route through a corpus whose 4b half can be emptied without any test noticing.

The contrast is exact and confirms the omission rather than excusing it: deleting
`lang/arith_digits.rex` from `phase-4a.txt` fails **both**
`phase_4a_subset_matches_the_committed_list` **and**
`every_in_scope_variant_is_witnessed_by_the_phase_subsets`. 4a built the pin precisely
for this; 4b's file did not get one, and the gate document claims the protection anyway.

Fix: either add the `phase-4b.txt` equivalent of
`coverage.rs:529`'s committed-list equality, or amend criterion 1's falsification note
to say that only three of the twelve entries are load-bearing for variant coverage and
that the rest are unpinned. The first is a few lines and closes it; the second at least
stops the document claiming coverage it does not have.

### Important

**I1. Criterion 6's own wording is contradicted by the script it grades, and the
assessment marks it MET without amendment.** `docs/superpowers/plans/phase-4b-gate.md:198`
vs `rust/scripts/mutate-4b.sh:543`

Criterion 6 requires *"a committed list of one-line mutations to the code 4b added, **each
of which some instrument in this gate must catch**"*. Row 12 (`I17: DROP of a stem as a
plain slot clear`) is declared `PASSED PASSED` — caught by neither instrument, on
purpose, and the brief asked for it. The criterion needed the same explicit amendment
criterion 2 and criterion 4 got ("each of which behaves exactly as declared"), and the
assessment at `:433-437` reports MET without noting that the literal wording no longer
describes the artefact. This is the same defect class the gate is otherwise good at
naming — criterion 2 is flagged for exactly it — applied to text this task wrote itself.

**I2. Step 3b's third-data-point explanation contains two false statements.**
`docs/superpowers/plans/phase-4b-gate.md:678-682`

The load-bearing measurement is **true and reproduces**: `git diff --stat` over the six
files between `5b2de07a` and `4c8c1f68` is empty, there are exactly ten intervening
commits, and the whole table re-derives exactly (1,852/9,025 = 20.5%; 1,928/15,838 =
12.2% at both later points). The sentence that follows does not:

* *"They are plan and documentation edits"* — two of the ten are not. `8d6790c6` ("Task 11
  review round 1") changes `rust/crates/rexx-exec/tests/keyword_assertions.rs`,
  `rust/crates/rexx-extract/src/keyword.rs`, `rust/crates/rexx-extract/tests/extract_keyword.rs`
  and `rust/corpus/keyword-exempt.txt` — 480 insertions. `b23986d9` changes four of the
  same paths. A reader is told the tree did not move between Task 11 and the gate; it
  moved substantially, just not in those six files.
* *"including three that corrected this very step's own framing"* — exactly one does.
  `ceabe481` is titled "Re-measure Task 12 Step 3b's ratio". The other four plan edits
  (`19692bcc`, `a6c5b759`, `676b7504`, `1388a341`) all rewrite the two-exempt-lists
  preamble, not Step 3b, and `877921d9` corrects an unrelated line-continuation claim.

**I3. The merged `EXCLUSIONS` row records the six-body notice mechanism twice,
near-verbatim, in the same row whose ruling argues against exactly that.**
`docs/superpowers/plans/phase-4-exclusions.txt:186-197` and `:246-256`

Both paragraphs name the same six bodies, both say they are the only assertion failures
in `base/keyword`, both say each was re-run under the C++ oracle in its rewritten form,
and both cite the comment-handling commit. The gate's own closing line is *"Recording a
gap twice is worse than recording it once, and the second copy is the one that hides the
first's incompleteness"* (`phase-4b-gate.md:832-833`). The merge is otherwise clean —
the pointer left in `KNOWN GAPS` names **both** former titles, so a reader who knows the
defect as "A COMPOUND CONTROL VARIABLE IS STORED IN THE WRONG PLACE" finds it, and every
measured transcript, probe and cost paragraph from both old rows has a home in the new
one. The only content that did not survive is the old row's file path for `bind_control`
(`rexx-exec/src/run.rs`), which the gate document supplies instead.

### Minor

**M1. "The witness list stands at 12" counts one of the two witness tables.**
`docs/superpowers/plans/phase-4b-gate.md:426`. `INSTRUCTION_WITNESSES` has 12 rows and
`EXPR_WITNESSES` has 4; the list is 16. The 4a gate's figure it compares against
("20") was likewise `InstructionKind` only, so the *trend* is right. Separately,
`loud.rs:80` describes its rows as "one row per still-loud **arm**", so calling them
"coarse phase-owned tags" is the opposite of what the file says — they coincide at 12
only because `Call` happens to have exactly one still-loud arm.

**M2. "`corpus.rs`, `assertions.rs`, `trace_oracle.rs` and `collect_stress.rs` never read
a phase string" is false as written.** `docs/superpowers/plans/phase-4b-gate.md:748-749`.
`trace_oracle.rs:535-575` carries `Coverage::Owned("4c")` / `Owned("Phase 5")` for six
prefixes and asserts every owner against its own `OWNER_PHASES`; `assertions.rs` carries
35 `unblocked_by: "Phase 5"` rows that `the_exempt_set_matches_the_current_blocked_rows`
reads. The *conclusion* survives — neither derives its strings from `instruction_owner`,
so removing phase attribution from the loud messages would not break them — but the
sentence understates the phase-string surface a 4c planner would inventory.

**M3. Criterion 9's falsification says "three separate mutations"; two are in the
script.** `docs/superpowers/plans/phase-4b-gate.md:266-271`. Rows 10 and 11 are run here;
the third ("deleting the whole instruction arm") is cited from
`phase-4-exclusions.txt:1023-1026`, where it was measured at Task 8 against a 39-program
subset ("39 of 39 → 38 of 39"), not the 42 this gate reports. The assessment at `:541-543`
discloses the provenance correctly; the criterion's own text does not.

**M4. `mutate-4b.sh`'s I17 comment states "It is genuinely equivalent" as fact, and is a
fourth prose copy of an argument already committed at Task 5.**
`rust/scripts/mutate-4b.sh:531-542` vs `rust/crates/rexx-exec/src/stem.rs:391-428`. The
`stem.rs` doc already carries the reclassification, the `keep. = a.` transcript, the
"built and run both ways" note and the exposure check — and instructs a future script to
*"copy this paragraph into it and drop the mutant rather than list it as uncaught"*. The
script does the opposite (lists it as a declared survivor), which is the better choice
and is what the brief asked for, but the departure is not noted, the source is not cited,
and this lands in the same commit whose Step 3b recommends *deleting* prose that restates
guarded data. `stem.rs`'s own hedge ("at every observation point this phase has") is also
stronger than the script's flat "genuinely equivalent".

**M5. `suite_status`'s non-zero-run-count guard is aggregate, not per instrument, and a
`DIVERGED` suite verdict does not identify which instrument spoke.**
`rust/scripts/mutate-4b.sh:219-244`. `suite_counts` sums every `test result:` line, and
`run_suite` omits `--no-fail-fast`, so cargo stops at the first failing target and the
sum is partial. Exercised against crafted output: `300 passed / 0 failed` in one target
plus `0 passed; 0 failed; 2 filtered out` in another classifies **PASSED**. This matters
for criterion 4, whose whole claim is that the *collector* is the instrument that sees a
dropped activation root — the script only records "suite DIVERGED". Verified by hand and
the claim holds: with row 9's mutation applied, `--lib` reports 301 passed / 0 failed and
`collect_stress` fails `the_l0_subset_passes_again_under_collect_on_every_allocation`
with a panic at `crates/rexx-exec/src/value.rs:125` — "a live value". Nothing else fails.

**M6. `corpus_status` returns `PASSED` for "0 of 0 matching" with exit 0.**
`rust/scripts/mutate-4b.sh:185-198`. Unreachable today only because `corpus.rs:553`
asserts a non-empty subset and `corpus.rs:230` asserts the oracle binary exists — both in
a file the script does not own and that a mutation could plausibly touch. Every other
degenerate shape I fed the classifiers is handled correctly: no matching line →
`INFRA_FAILURE` at either exit status; "42 of 42" with non-zero exit → `INFRA_FAILURE`;
"41 of 42" with exit 0 (the gate env lost) → `INFRA_FAILURE`; no `test result:` line →
`INFRA_FAILURE`; `0 passed; 0 failed; N filtered out` → `INFRA_FAILURE`.

**M7. The exclusions file's standing obligation to this task is not discharged in the
gate document.** `docs/superpowers/plans/phase-4-exclusions.txt:37-41` says of
`NOT_IMPLEMENTED_EXIT`'s band: *"The specific integer is Task 12's to confirm."* The gate
document never mentions it. The property itself is machine-asserted at
`rust/crates/rexx-exec/tests/spike.rs:113` (`!(157..=253).contains(&NOT_IMPLEMENTED_EXIT)`),
so nothing is actually at risk — but the file Task 12 edited asks Task 12 for a sentence
it did not write.

**M8. Step 3b's third row is labelled `4c8c1f68` ("this gate") but the gate commit is
`a5a979ad`.** `docs/superpowers/plans/phase-4b-gate.md:676`. At `a5a979ad` the
interpreter total is 15,841, not 15,838 — the three lines the `bind_control` doc-comment
fix added. The fraction is unchanged at 12.2%, and the document states its measurement
commit plainly at `:307-309`, so this is labelling rather than error.

---

## What reproduced exactly

Re-derived independently, not read back from the report:

* **42 = 30 + 12, no overlap.** `phase-4a.txt` 30 non-comment entries, `phase-4b.txt` 12,
  `comm -12` empty. All three of `corpus.rs:550`, `coverage.rs:569` and
  `collect_stress.rs:129` read the same two-file union.
* **The 4 ignored tests are exactly the four named**: `rexx-num/tests/format.rs:995`,
  `:1006`, `:1020` (2.1/3.0/2.1 GB) and `corpus.rs:584` (the child-process probe).
* **35 `EXEMPT` rows, all `unblocked_by: "Phase 5"`** in `tests/assertions.rs` — so
  criterion 2's prediction that the figure cannot move at this gate or 4c's is sound.
* **796 = 790 `4c` + 6 `defect:compound-do-control-variable`**, tallied on the second
  tab-separated field of `rust/corpus/keyword-exempt.txt`.
* **13 `Witnessed` / 6 `Owned`** in `PREFIX_COVERAGE`, with owners exactly as the gate
  lists them (`+++`, `>.>`, `>I>`, `<I<` → 4c; `>M>`, `>N>` → Phase 5).
  `phase-4a-gate.md:59` confirms ten at the 4a gate, so "up from 10" is right.
* **The L1 figures, by running the gate**: `REXX_KEYWORD_GATE=1 … --test keyword_assertions`
  exits 0, 7 tests, and prints *"100 of 896 bodies passing, carrying 713 of 1773
  assertSame calls"*. 1,773/2,441 = 72.6%.
* **2,441 / 2,561 / 120.** Counted over `ootest/ooRexx/base/keyword`'s 42 `.testGroup`
  files: 2,441 occurrences of the method name `assertSame`, 2,561 with the prefix match,
  and the difference is exactly the 120 `assertSameList` calls. (The literal spelling
  splits 1,931 `assertSame` / 510 `AssertSame`, so "spelled exactly `assertSame`" means
  "the method, not `assertSameList`" rather than case-exact — the number is right.)
* **Zero Phase 5 dependency in `base/keyword`**, from the gate's own machine-produced
  "not passing, by what would unblock it" breakdown: only `4c` (790) and the defect (6).
* **The whole line-ratio table**, at all three commits, plus the empty six-file diff and
  the ten intervening commits (see I2 for what is wrong in the sentence *around* it).
* **Mutation 2 fails exactly five `run.rs` unit tests** including
  `a_trap_is_inherited_by_a_callee_and_fires_in_the_callees_own_activation`, as the new
  `KNOWN GAPS` row states.
* **Criterion 4's control is real**, and it is the collector that catches it — see M5.
* **`unsafe_code = "forbid"`** at `rust/Cargo.toml:11`.
* **The `KNOWN GAPS` section's own rules permit the trap-inheritance row without
  amendment** (`phase-4-exclusions.txt:575-582`: adding needs none, removing does, and
  assigning an owner is one of the two things that justifies removal). The report's claim
  here is accurate, and so is the ruling's authority to move the compound-`DO` row.
* **The Step 3c derivation claim.** `RunOutcome::AssertionFailed` maps unconditionally to
  `"defect:compound-do-control-variable"` at `keyword_assertions.rs:251-253`, and
  `Blocked` derives its owner from the loud message — so "rewriting those six rows to
  `4c` would turn the set assertion red immediately" is correct, and
  `the_exempt_set_matches_the_current_failures` is a real automatic notice.
* **Criterion 8's witness is genuinely non-vacuous.** `corpus/lang/condition_traps.rex`
  accumulates `START/SYNTAX-AT-n/RAISED-AT-n/NOVALUE-AT-n/TAIL-AT-n/USER-CALLED-AT-n/RAISER-RETURNED`
  into `zwitness` and prints the whole string; no segment is any variable's derived name
  or a prefix of one, and a block that does not run removes its own segment. The handler
  at `trap_user:` writes a segment, so the assertion is on a value a handler set.
* **The `run.rs` edit is doc-comment-only** — one hunk, six lines changed inside
  `bind_control`'s `///` block, no code.
* **`now PASSES` and `is not on the committed`** each occur exactly once in
  `keyword_assertions.rs`, so criterion 10's citation-by-fragment works as described.

## Could not verify

* **That exactly two mutation declarations were edited to match measurement.**
  `mutate-4b.sh` is created whole in `a5a979ad`, so no pre-correction state exists in
  history and the claim is unfalsifiable from the tree. The *form* of the admission is
  adequate — rows 2 and 5 each carry it in their own comments, and both are restated in
  criterion 6's assessment, which is what the brief asked for. Its *completeness* rests
  on the author's word. Nothing in the tree contradicts it.
* **The report's oracle-absent attack as performed.** `oracle_root()` is hardcoded at
  `corpus.rs:201`, so reproducing it means editing a read-only-by-convention test file. I
  exercised the script's own `corpus_status`/`suite_status`/`suite_counts` against crafted
  instrument output instead, which covers the same failure modes and one the report did
  not try (see M5, M6). The guard holds everywhere except the unreachable `0 of 0` case.
* **The trap-inheritance row's mechanism as a standing property.** It is true today: the
  only two `RAISE`s inside a *called routine* anywhere in the 42 are
  `lang/condition_traps.rex:82` and `lang/call_on_trap_rearms.rex:66`, and both are
  `RAISE … RETURN` (`condition_traps.rex:58`'s raise is inside a `SIGNAL ON` handler
  label, which is the same activation, not a routine). But it is a universal quantifier
  over an in-repo mutable set, which this repo's own rule says to assert rather than
  write down. Its only guard is mutation row 2's declared `PASSED/DIVERGED` pair going
  `NOT AS DECLARED` if a corpus program ever gains the shape — which is a real guard, but
  it lives in a script no `cargo test` runs.
