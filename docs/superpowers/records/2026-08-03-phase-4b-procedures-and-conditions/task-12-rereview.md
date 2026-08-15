# Task 12 re-review — fix round 1 (`a5a979ad..0e14fac4`)

Scoped re-review at `0e14fac4`. Two questions: are C1, I1–I3 and M1–M8 actually fixed, and did
this round introduce new false statements.

Everything below was checked against the tree. Every mutation was made in the working tree with a
scratchpad backup, reverted immediately, and `git diff --quiet` returned 0 afterwards; the final
tree is clean and `phase-4b.txt` and `run.rs` are byte-identical to `HEAD` (md5 re-checked after
each restore).

---

## Verdict

**Phase 4b's gate can close once three sentences are corrected.** The instruments are sound: the
pin is real and at full strength, the classifiers are genuinely hardened rather than documented,
and every measured figure in the document reproduces. Every remaining defect is prose — including
one that is a fresh instance of the exact failure C1 records.

---

## Part 1: per-finding

| finding | verdict |
|---|---|
| **C1** | **fixed** — pin real, at 4a's strength; falsification note now true. One false sentence beside it (N1). |
| **I1** | **fixed in the document, not in the script** — `mutate-4b.sh:14` still carries the pre-amendment wording. |
| **I2** | **fixed, and two new defects introduced** (N2, N4). Both original false statements are gone and correctly replaced. |
| **I3** | **fixed** |
| **M1** | **fixed** |
| **M2** | **fixed** |
| **M3** | **fixed** |
| **M4** | **fixed** |
| **M5** | **fixed for the load-bearing half; second half not addressed** |
| **M6** | **fixed** |
| **M7** | **fixed** |
| **M8** | **fixed** |

### C1 — fixed

The falsification note (`phase-4b-gate.md:63-67`) now names a pin per file and no longer claims a
deletion leaves a variant unwitnessed. That is what the pin does.

**The pin has the same strength as `phase-4a.txt`'s, by the same device.** `read_subset`
(`coverage.rs:419`) returns an ordered `Vec<String>` and both pins are `assert_eq!` against a
committed `&[&str]`, so all three drift shapes are caught. Measured, exit status read unpiped:

| mutation to `phase-4b.txt` | `phase_4b_subset_matches_the_committed_list` |
|---|---|
| append `lang/exit_with_value.rex` | **exit 101, FAILED** |
| swap the first two entries (reorder only) | **exit 101, FAILED** |
| unmodified | exit 0, ok |

Deletion was verified by the controller. So: add, delete and reorder all fire.

### I1 — fixed in the document, not in the script

The criterion's amendment (`phase-4b-gate.md:216-238`) is true of the mechanism: `run_one`
compares both observed statuses against both declared ones and records `NOT AS DECLARED` in
**either** direction (`mutate-4b.sh:415-421`), and the script exits 1 whenever `FAILURES` is
non-empty (`:640-647`). An unexpected catch really does fail as loudly as an unexpected survival.

But the finding was a contradiction *between the document and the script it grades*, and only the
document moved. `mutate-4b.sh:13-14` still opens:

> `# The 4b exit gate's criterion 6: a committed list of one-line mutations to the`
> `# code Phase 4b added, each of which some instrument in this gate must catch.`

That is now a misquotation of criterion 6 and is false about the script's own row 12. The gate
document says the amendment is "recorded beside them rather than left as a contradiction between a
document and the script it grades" — the contradiction survives, one file over.

### I2 — fixed, with two new defects

Both original false statements are gone, and the replacements check out where they are new:

* "**exactly one** commit corrected this step's framing (`ceabe481`)" — **true**. `ceabe481` is
  the only one of the ten that touches Step 3b's framing; `4c8c1f68` adds Steps 3c/3d, and the
  four preamble commits edit the two-exempt-lists block above Step 1.
* "four rewrite the two-exempt-lists preamble" — **true**: `19692bcc` (adds it), `a6c5b759`,
  `676b7504`, `1388a341`.
* "one corrects an unrelated line-continuation claim" — **true**: `877921d9`.
* "Two of the ten change `rust/` source" — **true**: `8d6790c6` and `b23986d9`.

The two defects are N2 and N4 below. Note also that the sentence accounts for 5 of the 7 remaining
plan edits; `4c8c1f68` and `9b7c8bac` go unmentioned. It does not claim exhaustiveness, so this is
incompleteness rather than error.

### I3 — fixed

The duplication is gone, the cross-reference resolves (`SIX ooTest BODIES TURN ON THIS` exists at
`phase-4-exclusions.txt:246`), and the surviving copy still carries all four facts the deleted one
restated: the six body names, "the only assertion failures in the whole base/keyword table", the
oracle re-run in rewritten form, and the comment-handling commit. The replacement paragraph's
account of what the earlier version said matches the diff exactly.

### M1 — fixed

`INSTRUCTION_WITNESSES` 12 rows, `EXPR_WITNESSES` 4 → 16. `phase-4a-gate.md:97` reads "20
`InstructionKind`, 6 `ExprKind`, 26 total", so the 20 is instruction-only and the comparison is
like for like. `loud.rs:80-81` reads *"one row per still-loud arm for the one split variant left —
`\"Call::Qualified\"` alone"*, which the document now quotes accurately instead of calling the
rows coarse.

### M2 — fixed

`PREFIX_COVERAGE` carries exactly **6** `Coverage::Owned` rows and 13 `Witnessed`
(`trace_oracle.rs:537-554`; the 7th `Owned` match at `:630` is the match arm, not a row).
`assertions.rs` carries **35** `unblocked_by: "Phase 5"`. `corpus.rs` and `collect_stress.rs`
contain no phase string at all. The bullet's conclusion — that none of it derives from
`instruction_owner` — survives, which was M2's point.

### M3 — fixed

`phase-4-exclusions.txt:1021-1024` records the whole-arm deletion (`Push | Queue =>
Ok(Flow::Next)`) as `39 of 39` dropping to `38 of 39`. The criterion now states that two of the
three are rows of this script and the third is cited, with its Task 8 provenance and subset size.

### M4 — fixed

Every citation the script now makes to `stem.rs` is accurate: "copy this paragraph into it and drop
the mutant rather than list it as uncaught" (`stem.rs:395`), "`read_stem` vivifies a fresh stem on a
miss" (`:414`), "at every observation point this phase has" (`:416`), the exposure case and "the two
`drop a.` ones" of `an_exposed_stem_aliases_the_callers_entry_not_the_object` (`:419-423`; the test
is at `run.rs:10725`). The weaker hedge is carried across rather than flattened back to "genuinely
equivalent", and the departure from what `stem.rs` asked for is stated.

### M5 — fixed for the load-bearing half; second half not addressed

**Run, not read.** The classifiers were extracted verbatim from `mutate-4b.sh:212-323` and fed
crafted instrument output. 9 suite cases and 7 corpus cases, all correct:

| suite output | exit | verdict |
|---|---|---|
| `300 passed/0 failed` + `0 passed; 0 failed; 2 filtered out` — **the review's own case** | 0 | **INFRA_FAILURE** |
| 3 targets, one of them `0 passed; 0 failed` | 0 | INFRA_FAILURE |
| only 2 of 3 targets reported (fail-fast shape), both non-empty | 101 | INFRA_FAILURE |
| 3 targets, all non-empty, all green | 0 | PASSED |
| 3 targets, one failure | 101 | DIVERGED |
| 3 targets, one failure, exit 0 (gate env lost) | 0 | INFRA_FAILURE |
| 3 targets green, exit 101 | 101 | INFRA_FAILURE |
| no `test result:` line (compile failure) | 101 | INFRA_FAILURE |
| 4 reporting targets | 0 | INFRA_FAILURE |

The partial-sum path is genuinely closed, not documented: `suite_status` iterates per line, returns
`INFRA_FAILURE` on any target with a zero run count, and requires the line count to equal
`SUITE_TARGET_COUNT=3`. `run_suite` gains `--no-fail-fast`, which is what makes the per-target check
reachable. `suite_totals` was split out for the baseline progress line only and is never used for a
decision.

**Not addressed:** M5's second clause — *"a `DIVERGED` suite verdict does not identify which
instrument spoke"*. `run_one` still prints `suite=DIVERGED` and a list of failing test **names**
(`mutate-4b.sh:404-406`, unchanged from `a5a979ad`); the target binary is never named. It is
inferable from the test names, not stated, and criterion 4's "the collector is the instrument that
sees it" still rests on that inference.

### M6 — fixed

`corpus_status` returns `INFRA_FAILURE` for `0 of 0 matching` at exit 0 **and** at exit 101, with a
comment giving the reason. The six other degenerate shapes still classify correctly (table above).

### M7 — fixed

`NOT_IMPLEMENTED_EXIT = 120` at `lib.rs:119`, outside `157..=253`. `spike.rs:111-123`
(`the_loud_failure_code_cannot_be_confused_with_a_rexx_error`) asserts the band with the message
"256 - major lives in 157..=253 for majors 3 to 99" **and** pins the exact loud message beside it,
as the gate document says. `phase-4-exclusions.txt:37-39` does ask Task 12 for the integer by name
and does give the stale-data reason the gate document attributes to it.

### M8 — fixed

Row relabelled, fourth row added, and every number in it re-derives:

| | owners+loud+coverage | run+eval+error | fraction |
|---|---|---|---|
| `4c8c1f68` | 606+609+713 = **1,928** | 12,759+1,872+1,207 = **15,838** | 12.2% |
| `HEAD` | 606+609+761 = **1,976** | 12,762+1,872+1,207 = **15,841** | **12.47% → 12.5%** |

"+3 interpreter lines from `bind_control`'s doc comment" and "+48 harness lines from the pin" both
check out exactly. The assessment's "the gate first measured 1,019 at `4c8c1f68`" is consistent —
`git diff 4c8c1f68 HEAD` over `rust/**/*.rs` adds exactly one `#[test]`.

---

## Part 2: new false statements

### N1. `lang/condition_traps.rex` is the corpus catcher for **one** mutation, not three

`docs/superpowers/plans/phase-4b-gate.md:77-78` — *"criterion 8's only named witness, and the
declared corpus catcher for **three** of criterion 6's mutations"*
`rust/crates/rexx-exec/tests/coverage.rs:553-555` — the same claim, *"three of `mutate-4b.sh`'s
mutations"*, inside the pin's own doc comment.
(Also in `0e14fac4`'s commit message.)

**True value: one.** Six of the twelve rows declare a corpus catch (1, 3, 4, 5, 6, 7). Each was
applied to `run.rs` and run under `REXX_CORPUS_GATE=1`; each diverges on exactly **one** program:

| row | mutation | diverging program |
|---|---|---|
| 1 | `PROCEDURE EXPOSE` aliases nothing | `lang/call_procedure_expose.rex` |
| 3 | `USE ARG` binds each target to the next argument | `lang/use_arg_forms.rex` |
| 4 | a bare `RETURN` does not drop `RESULT` | `lang/call_return.rex` |
| 5 | an `INTERPRET` fragment's echo resolves its own line | `lang/interpret_error_echo.rex` |
| 6 | `SIGL` is one line past the raising clause | **`lang/condition_traps.rex`** |
| 7 | a `CALL ON` trap disarms permanently when it fires | `lang/call_on_trap_rearms.rex` |

All six report `41 of 42 matching`, `mismatches (1)`. The other six rows declare corpus `PASSED`,
so no corpus program catches them at all. The script's own prose names `condition_traps.rex`
exactly once, at `mutate-4b.sh:528` (row 6). Rows 1 and 7 are structurally out of its reach:
`condition_traps.rex` contains no `PROCEDURE`, and its single `call raiser` (`:76`) fires the
`CALL ON` trap once, where row 7 is only observable on a second raise — which is why
`call_on_trap_rearms.rex` exists.

**This is C1's own defect, one round later.** The triple came from the review's C1 text and was
carried into two files without being run, in the paragraph whose subject is a falsification clause
that was reasoned instead of measured.

### N2. `b23986d9` touches **three** of the named paths, not four

`docs/superpowers/plans/phase-4b-gate.md:765` — *"and `b23986d9` four of the same paths."*

The four paths just named are `keyword_assertions.rs`, `keyword.rs`, `extract_keyword.rs` and
`keyword-exempt.txt`. `b23986d9` touches **three** of them — it does not touch
`rexx-extract/tests/extract_keyword.rs`. Under the looser reading ("the same paths" = all six of
`8d6790c6`'s), the intersection is **five**. Neither is four.

### N3. Deleting all nine unpinned entries does **not** leave the suite green

`docs/superpowers/plans/phase-4b-gate.md:903-905` — *"`phase-4b.txt`'s twelve entries were pinned
by nothing, so **nine of them could be deleted with the whole suite staying green**."*

Measured. With all nine removed and only `call_expression`, `use_arg_forms` and `push_queue` left,
`every_in_scope_variant_is_witnessed_by_the_phase_subsets` fails, exit 101:

```
criterion 1's coverage property fails:
InstructionKind: 3 in-scope variant(s) unwitnessed by the phase subsets: Interpret, Signal, Raise
```

Only the **distributive** reading is true — any *one* of the nine, on its own — and only that was
measured. The collective reading a reader takes from "nine of them could be deleted" is false. The
second half is unmeasured too: "the whole suite staying green" was established only for
`condition_traps.rex`; for the other eight the review ran `coverage.rs` alone.

### N4. 480 insertions is the whole commit, not the four `rust/` files

`docs/superpowers/plans/phase-4b-gate.md:763-765` — *"`8d6790c6` touches `keyword_assertions.rs`,
`keyword.rs`, `extract_keyword.rs` and `keyword-exempt.txt` **across 480 insertions**"*.

Those four carry **406** insertions (21 + 112 + 56 + 217). 480 is the commit total across six
files; the other 74 are `l1-coverage.md` (54) and `phase-4-exclusions.txt` (20). In a sentence
whose subject is "two of the ten change `rust/` source", a reader takes 480 as the `rust/` figure.

---

## Minor, not false

* **`mutate-4b.sh:14`** still carries the pre-amendment criterion 6 wording — see I1.
* **Two new hard-wrapped identifiers defeat `grep`**: `phase-4b-gate.md:747-748` breaks
  `` `phase_4b_subset_matches_the_committed_list` `` after the trailing underscore, and
  `phase-4-exclusions.txt:187-188` breaks `` `the_exempt_set_matches_the_current_failures` ``
  the same way. Commit `1388a341`, in this same phase, added a plan instruction to cite short
  greppable fragments precisely because wrapping defeats `grep`.
* **`task-12-report.md:190`** still labels the `4c8c1f68` row "(this gate)" and has no fourth row —
  M8's defect, corrected in the gate document but not in the report. The numbers themselves are
  right for `4c8c1f68`.
* Criterion 6's "**Five** mutations deliberately do not go through the corpus" counts row 2, whose
  corpus survival the gate itself reports as *discovered*, not chosen. Defensible as a description
  of the committed declarations; "deliberately" is generous.

---

## What reproduced exactly

Re-derived, not read back:

* `read_subset` returns an ordered `Vec<String>`; both pins are `assert_eq!` against a committed
  `&[&str]` — so the 4b pin is order-, addition- and deletion-sensitive, measured for the first two.
* The ten intervening commits and their per-file numstats; the four preamble commits and the
  line-continuation commit identified by content, not by title.
* 12 `INSTRUCTION_WITNESSES` / 4 `EXPR_WITNESSES`; 6 `Owned` / 13 `Witnessed` in `PREFIX_COVERAGE`;
  35 `unblocked_by: "Phase 5"`; zero phase strings in `corpus.rs` and `collect_stress.rs`.
* Every line count in the Step 3b table, at `4c8c1f68` and at `HEAD`, and both deltas.
* `NOT_IMPLEMENTED_EXIT = 120`; `spike.rs`'s band assertion and its pinned message.
* All 12 mutation declarations; the sole diverging program for each of the six corpus-catching rows.
* 16 crafted classifier cases against the hardened `corpus_status`/`suite_status`.
