# Phase 4b — final whole-branch review

Range `dc87708d..a1484211`, 96 commits, 84 files. Scope as dispatched: the four
things a per-task review structurally cannot see. **Not** a re-read of the
per-task hunks — thirteen tasks and twenty-plus review passes already covered
those, and a fourteenth pass over the same diff produces confident findings
about code whose context this reviewer lacks.

**Verdict: ready with fixes.** Nothing behavioural is wrong. Every finding below
is a false or stale *statement* — which is this phase's own declared defect
class, and the reason the fixes are worth making before merge rather than after.
Five are cross-task drift of exactly the shape the dispatch named; the rest are
minor.

The verified-state figures were taken as given and not re-run. Everything below
was measured against the tree at `a1484211`.

---

## Critical

None.

---

## Important

### I1. `l1-coverage.md` still says the compound-`DO` gap is unowned, which Task 12 ruled it is not

`docs/superpowers/plans/l1-coverage.md:704-712`

Three false statements in one paragraph:

* "**the gap is unowned**"
* "Nothing schedules compound control variables, so **no phase owns the fix**"
* "which is why **the row sits in `phase-4-exclusions.txt`'s `KNOWN GAPS`**"

Task 12's Step 3c ruling assigned the defect to 4c and moved the row into
`EXCLUSIONS` — `phase-4-exclusions.txt:148`, "EXCLUSIONS -- a compound variable
as a DO control variable, owned by 4c". `KNOWN GAPS` retains only a pointer
(`phase-4-exclusions.txt:1117-1136`).

**This is the dispatch's named defect class, reproduced exactly.** The same fact
had three prose copies. Two were updated by the ruling:

* `rust/crates/rexx-exec/src/run.rs:5281-5285` (`bind_control`'s doc) — corrected,
  and the gate document flags the correction explicitly at
  `phase-4b-gate.md:950-956`.
* `phase-4-exclusions.txt` — the row itself moved.

The third was not. `git log dc87708d..a1484211 -- docs/superpowers/plans/l1-coverage.md`
ends at `b23986d9` (Task 11 review round 2), which precedes the ruling commits
`9b7c8bac`, `4c8c1f68` and `a5a979ad`. The file is one of the four claim
documents the gate rests on, and it now contradicts the gate.

**Fix:** rewrite `l1-coverage.md:704-712` to state the owner and cite the
`EXCLUSIONS` row.

### I2. `tests/support/mod.rs` states a prefix count Task 9 falsified

`rust/crates/rexx-exec/tests/support/mod.rs:35-36`

> "covering the **ten prefixes this crate can emit today** (D17's own
> reachable-from-4a list) plus **the nine it cannot reach yet**"

Measured: `rexx-exec/src/trace.rs` emits **thirteen** — `*-*`, `>>>`, `>K>`,
`>=>`, `>L>`, `>V>`, `>E>`, `>O>`, `>P>`, `>C>`, plus Task 9's `>A>`
(`trace.rs:569`), `>F>` (`:592`) and `>R>` (`:617`). Gate criterion 3 reports
the same split from the other side: 13 witnessed, 6 owned
(`trace_oracle.rs`'s `PREFIX_COVERAGE`, verified 13 `Witnessed` / 6 `Owned`
over the oracle's 19).

Written at `0830a12e` (Task 2b), falsified by `233fbd8b` (Task 9), in a file
Task 9 never had to touch — the drift geometry the gate document itself
describes at `phase-4b-gate.md:901-907`.

It is also a direct breach of `rust/CLAUDE.md:46-48`: a present-tense count of a
mutable in-repo aggregate, in prose, unasserted.

**Fix:** state the property, not the count — the normalisation covers all
nineteen markers `trace_prefix_table` lists, whether or not this crate emits
them yet. That is what the paragraph is actually arguing, and it is true
permanently.

### I3. `coverage.rs` contains a claim and its own falsification, 167 lines apart

`rust/crates/rexx-exec/tests/coverage.rs:439` versus `:606`

At `:439`, justifying `read_subset_unions_two_files_first_seen_order_deduplicated`:

> "`read_subset`'s multi-file union and de-duplication [...] had no test, because
> **every real call site today passes a one-element slice**"

At `:606`, in the same file:

> "the reason `read_subset` takes a slice at all (Task 0's Step 4), and **this is
> the first call site to pass more than one path**."

Every real call site now passes two: `coverage.rs:622`, `corpus.rs:548`,
`collect_stress.rs:127`. Task 0 wrote `:439`; Task 1 made it false and wrote
`:606` saying so, without revisiting the paragraph above it. Four review passes
over `coverage.rs` did not connect them.

The test remains worth keeping — it pins first-seen ordering and dedup
semantics that the real call sites exercise but do not assert. Only the
justification is false.

**Fix:** delete the "every real call site today" clause.

### I4. `read_subset` is three byte-identical hand-maintained copies; one is tested

`rust/crates/rexx-exec/tests/collect_stress.rs:106`,
`rust/crates/rexx-exec/tests/corpus.rs:254`,
`rust/crates/rexx-exec/tests/coverage.rs:419`

All three bodies are byte-identical (diffed). Criterion 1's headline — the union
of `phase-4a.txt` and `phase-4b.txt`, "42 of 42" — is produced by all three
agreeing, and only `coverage.rs`'s copy has a test.

The reason this belongs in a whole-branch review rather than a task review: Task
2b created `tests/support/mod.rs` for precisely this hazard, and its module doc
argues the case in its own words —

> "This function is compared against on two call sites and must give the
> identical answer on both, so it is written once here" (`support/mod.rs:20-22`),
> citing `error.rs`'s duplicated `push_clause` lines, which "drifted until a
> clamp had to be added to both by hand".

`corpus.rs:182` already declares `mod support;`. `coverage.rs` and
`collect_stress.rs` do not. So the phase built the shared home, moved one helper
into it, and left the other three-way duplicated — a decision no single task's
review had both halves in view to make.

**Fix (4c is acceptable):** move `read_subset` into `tests/support/mod.rs` and
add `mod support;` to the two files that lack it. Mechanical, no behaviour
change. If deferred, say so in the 4c plan rather than leaving it implicit.

### I5. The deferred-minors roll-up is incomplete — five items recorded for this review are absent from it

`.superpowers/sdd/2026-08-03-phase-4b-procedures-and-conditions/deferred-minors.txt`
carries 16 rows, all from Tasks 0 and 1. `progress.md` records five more,
**both blocks explicitly addressed to this review**, and neither reached the
file:

* `progress.md:962-967` — "**Minor (deferred), to be triaged by the final
  whole-branch review:**" — three items from Task 7's report.
* `progress.md:998-999` — "**Minor (deferred), for the final whole-branch review
  to triage:**" — two items from Task 7's review.

Triaged below with the rest. Two of the five are already closed in the tree; the
roll-up would have shown that if they had been in it. The mechanism the lead was
worried about — a roll-up nobody reads becoming a silent discard — failed one
step earlier than expected: the roll-up was not the complete set.

**Fix:** append the five, or record in the 4c plan that the roll-up is
Tasks 0–1 only and Task 7's live in `progress.md`.

---

## Minor

### M-a. The Step 3b ratio table's last row is false at the commit it ships in

`docs/superpowers/plans/phase-4b-gate.md:774`

| row | claimed | measured at `a1484211` |
|---|---|---|
| "after this gate's own commits" | harness **1,976** | **1,983** |

Measured: `owners.rs` 606 + `loud.rs` 609 + `coverage.rs` 768 = 1,983. The
interpreter figure (15,841) and the percentage (12.5%) are both correct, so the
argument is untouched. 1,976 was true at `0e14fac4` and stopped being true at
`a1484211` — itself one of "this gate's own commits".

The document warns against this exact thing eleven lines below, at `:783-785`:
"labelling the third row with a commit at which its numbers are false would be
the same defect this step is about."

**Fix:** re-measure at the tip, or replace the row label with the property.

### M-b. "Every enumerated `match` still has no wildcard arm" is literally false

`docs/superpowers/plans/phase-4b-gate.md:423`; `coverage.rs:654`, `:666`

Two `_ => {}` arms, on `InstructionKind` and `ExprKind` respectively. They are
*dispatch* matches (which sub-enum to descend into), not the tagging matches, so
the consequence the gate draws — "a new variant in any of the seven enums is a
compile error here" — still holds through the exhaustive `instruction_tag` and
`expr_tag`. But a new looping or expression-bearing variant would be silently
skipped for sub-enum coverage.

**Fix:** narrow the sentence to the tagging functions, which is what it means.

### M-c. `rust/corpus/README.md`'s "Current programs" omits all twelve of 4b's corpus additions

18 of 50 `corpus/lang/*.rex` files are absent from the table; **12 are 4b's**
(`call_expression`, `call_on_trap_rearms`, `call_procedure_expose`,
`call_return`, `condition_traps`, `deep_nesting_indent_cap`,
`interpret_error_echo`, `loop_retest_blame`, `push_queue`,
`raise_array_substitution`, `signal_forms`, `use_arg_forms`). The other 6
predate `dc87708d`, so the table was already not an inventory — but 4b's own
README edit added a "Phase 4b subset" section describing `phase-4b.txt` and left
the neighbouring programs table alone, which is the same one-of-two-copies shape
as I1–I3.

Nothing asserts README completeness, which is why it drifted silently.

**Fix (4c acceptable):** add a "Phase 4b additions" subsection, or drop the
table's inventory framing.

### M-d. "unavoidably a third copy" now contradicts the gate's own Step 3b — in two places

`rust/crates/rexx-exec/src/lib.rs:662`, `rust/crates/rexx-exec/tests/owners.rs:600`

Both say the third ownership copy is "**unavoidably**" separate. The gate's Step
3b (`phase-4b-gate.md:841-854`) costs the avoidance concretely — "assert
`lib.rs`'s match equals `owners.rs` expanded through `expand_for_witnesses`",
estimated at half a day — and rejects it on value, not on possibility.

This is deferred minor **M5**, and it has two copies rather than the one the
roll-up records.

**Fix:** "separate by construction (production code cannot reach `tests/`);
consolidation is possible and was costed at the 4b gate — see Step 3b."

### M-e. An unasserted gate total in a doc comment

`rust/crates/rexx-exec/tests/keyword_assertions.rs:217` — "730 lines across the
passing bodies today, against 713 distinct assertions". 713 is a figure the gate
reports and nothing here asserts. Anchored to a pinned ooTest revision (r13178),
so lower risk than the usual case, but still the shape `rust/CLAUDE.md:46-48`
forbids.

### M-f. A soft-wrapped identifier renders with a space inside it

`docs/superpowers/plans/phase-4b-gate.md:835-836` —
`` `every_out_of_scope_variant_fails_\nloudly` `` renders as
`every_out_of_scope_variant_fails_ loudly`, which greps to nothing. Cosmetic.

---

## What was checked and found sound

Recorded so a later reader knows these were measured, not skipped.

| claim | source | result |
|---|---|---|
| union subset = 42 (30 + 12, deduplicated) | criterion 1 | ✅ |
| all 35 `assertions.rs` `EXEMPT` rows are `unblocked_by: "Phase 5"` | criterion 2 | ✅ 35/35 |
| `keyword-exempt.txt` = 796 rows = 790 `4c` + 6 defect | criterion 10 | ✅ exact |
| 13 of 19 trace prefixes witnessed; owners named for 6 | criterion 3 | ✅ |
| `loud.rs` = 12 instruction + 4 expr witness rows | criterion 5 | ✅ |
| `NOT_IMPLEMENTED_EXIT` = 120, outside `157..=253`, machine-asserted | criterion 7 | ✅ `spike.rs:111-115` |
| `unsafe_code = "forbid"` at `[workspace.lints.rust]` | criterion 7 | ✅ `Cargo.toml:10-11` |
| `queue.rs`'s three tests | criterion 9 | ✅ |
| `mutate-4b.sh` has 12 declared rows | criterion 6 | ✅ 12 `run_one` call sites |
| 4 ignored = 3 `rexx-num` format + 1 `corpus.rs` probe | assessment | ✅ |
| `ends_with`, not `contains`, on the loud suffix | criterion 5 | ✅ `loud.rs:591-592` |
| `instruction_owner` returns `None` for `InstructionKind::Do` | Step 3c | ✅ |
| the 6 defect rows are the only assertion failures in `base/keyword` | Step 3c | ✅ derived from `RunOutcome::AssertionFailed` |
| every long identifier the gate cites exists in the tree | — | ✅ 28/28 |
| `l1-coverage.md`'s four tables sum internally (896 bodies, 100 passing, 1,773 / 713 assertions, 790 blocked, 145 unextractable, 39 groups) | — | ✅ all six sums exact |
| the `Call::Trap`-is-loud fact | the phase's own example | ✅ all copies now correct |

---

## Deferred-minors triage

16 from the roll-up, plus the 5 it omitted (I5). "Verified" says whether the
finding was re-checked against the tree at `a1484211`.

| # | finding | verified | ruling |
|---|---|---|---|
| **T0-M1** | Step 5's block sits at end of file, not the module doc | yes — `owners.rs:563-606` | **4c** — cosmetic placement |
| **T0-M2** | `expand_for_witnesses` not exhaustiveness-checked against `Call`/`Signal` | yes — `loud.rs:159-175`, catch-all `other => vec![other]` | **4c** — a renamed arm degrades to a silent no-op expansion; real but remote |
| **T0-M3** | `the_two_harnesses_include_this_exact_file` defeated by the regression it names | yes — `owners.rs:545-559` | **4c** — it catches deletion of the `#[path]` line, not a hand-copied table added *beside* it. Fixing needs a different property |
| **T0-M4** | `instruction_arm` returns `String` where `&'static str` suffices | yes — `loud.rs:135-158`, every arm `.to_string()` on a literal | **4c** — mechanical |
| **T0-M5** | "unavoidably a third copy" overstates the constraint | yes — **two** copies, `lib.rs:662` and `owners.rs:600` | **fix before merge** — see M-d; the gate now contradicts it, and the roll-up undercounts it |
| **T0-M6** | `cargo fmt --edition 2024 --check` does not run; affects every brief | yes | **drop — already fixed.** Correct form now in `rust/CLAUDE.md:30-32`, the plan's preamble (`:46`) and `phase-4b-gate.md:286`. No brief carries the broken spelling |
| **T0-M7** | two comments describe a stderr shape that no longer exists | partial — one candidate at `owners.rs:605` reads correctly today | **drop** unless the reporter can name the two lines; not reproducible as stated |
| **T1-1** | two provably-no-op `record_leave_failure` calls in `run_fragment` | subject changed — `run.rs:5640`, `:5649`; `record_leave_failure` is now a one-line residual report (`:3908-3910`) | **drop** — recorded against the pre-Task-2 mechanism, which `43ac5816`/`e4caa7bf` replaced. Re-derive in 4c if it recurs |
| **T1-2** | 16 lines duplicated from `run_activation`'s arms | yes — `run.rs:5624-5657` | **drop** — the duplication is deliberate and documented in place ("Byte-identical in shape [...] deliberately -- same four constructors, same `record_leave_failure` call") |
| **T1-3** | a mutation-kill note naming a mutation that cannot be typed | no | **4c** — a mutation that cannot be applied is an unfalsifiable kill claim; same family as criterion 6's `INFRA_FAILURE` class |
| **T1-4** | a verbatim-preserved doc now false in the present tense | no (as filed) | **fix before merge** — this *shape* is confirmed live three times independently: I2, I3, M-a. Fix those; then re-check whether a distinct fourth instance remains |
| **T1-5** | ungrammatical `Outcome::collections` rewrite | no | **4c** — prose only |
| **T1-6** | nothing pins `phase-4b.txt`'s line list, unlike `EXPECTED_SUBSET` | yes | **drop — already fixed.** `phase_4b_subset_matches_the_committed_list`, `coverage.rs:583`, added by the gate's own fix round (`0e14fac4`). This is the roll-up's staleness made concrete |
| **T1-7** | Step 4's test duplicates an existing assertion without saying so | no | **4c** — see also I3, which is the same test's *other* comment |
| **T1-8** | a tripwire cost claim relying on dead-code elimination | no | **4c** — an unmeasured performance claim; either measure it or delete it, per `rust/CLAUDE.md:47` |
| **T1-9** | the report says "1 ignored" where the workspace has 4 | yes — 4 ignored confirmed | **drop** — a task report is a dated record; the gate reports 4 correctly |
| **T7-a** *(unlisted)* | `RAISE PROPAGATE`'s `active_condition` never cleared | yes | **drop — fixed.** `deliver_pending_trap` clears it in its `Ended::Returned` arm; tests at `run.rs:12078`, `:12204` |
| **T7-b** *(unlisted)* | non-`SYNTAX` `RAISE ... RETURN` searches one level out, missing a grandparent trap | yes | **drop — measured correct.** Task 7's review probe `pb` showed the oracle also declines; the doc calling it a residual was corrected (`run.rs:3086-3093`) |
| **T7-c** *(unlisted)* | `RAISE PROPAGATE`'s report rendering rests on four transcripts, not a family | no | **4c** — an evidence-depth concern, not a known defect; fold into 4c's condition work |
| **T7-d** *(unlisted)* | `run_activated`'s `_program` parameter is dead | yes — `run.rs:6832` | **4c** — test helper, one line |
| **T7-e** *(unlisted)* | `trap_for` is `pub(crate)` with no cross-module caller | yes — `run.rs:2474`; all callers (`:2509`, `:2564`, `:2667`) are in `run.rs` | **4c** — narrow to private |

**Totals:** 2 fix before merge, 12 fix in 4c, 7 drop (4 of them already closed in
the tree — T0-M6, T1-6, T7-a, T7-b).

---

## Could not verify

* **Whether any *further* stale prose copy exists in a file no task touched.**
  The four instances found (I1, I2, I3, M-d) were found by tracing facts known
  to have changed — the compound-`DO` owner, the prefix count, `read_subset`'s
  arity, the third-copy claim. That method only finds drift whose *changed fact*
  is already known. There is no cheap exhaustive check, and this is the phase's
  most-repeated defect, so assume more exist.
* **Task 1 minors 3, 5, 7 and 8, and T0-M7**, as filed — the roll-up records them
  as one-line summaries with no file:line, and locating each would mean the
  fourteenth pass over the task diff this review was scoped away from. Rulings
  above are on the findings as stated.
* **Criterion 1's "only `call_expression`, `use_arg_forms` and `push_queue`
  construct something nothing else in the union does."** Re-measuring means 12
  single-line deletions with a `coverage.rs` run each. The claim is now moot for
  its original purpose — `phase_4b_subset_matches_the_committed_list` pins all
  twelve regardless of which are individually load-bearing.
* **The gate's own figures.** Taken as given per the dispatch; not re-run.
