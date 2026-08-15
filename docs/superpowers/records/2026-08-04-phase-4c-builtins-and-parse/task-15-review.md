# Task 15 review — the `base/bif` L1 harness, the 4c subset, `mutate-4c.sh`, and the gate

Range `284ef543..89debc85`, four commits, 11 files, +3687/-22.
Reviewed against `task-15-brief.md` and `task-15-report.md`.

## Verdicts

* **Spec compliance: ✅ MET.**
  Every deliverable the brief names exists, runs, and does what the brief asked of it.
  Every global constraint holds.
  Every figure in the report's Step 7 table reproduced independently, to the digit.
* **Task quality: high, with two Important findings.**
  Both self-raised concerns that contradict the brief are correct and the brief is wrong.
  The three new drop reasons are genuine, not coverage discarded to flatter a number.
  The substituted `TIME('R')` catchers fire and the brief's declared one does not, exactly as reported.
  What is missing is a pin on this task's own Step 4 change, and a correction written back into the plan.

Findings: **0 Critical, 2 Important, 4 Minor.**

---

## The six self-raised claims, verified

### 1. The two contested figures — the implementer is right, the brief is wrong, both times

**Block comments: 25, not 120-135.**
An independent nesting-aware scanner (tracking `/* */` depth, `--` to end of line, and quoted strings, over raw bytes) gives **24 `assertSame` inside block comments** (`LINES.testGroup` 23, `CHARS.testGroup` 1) and **1 behind a `--`** (`DATE.testGroup`), for 25 — and 6,268 in code.
`6,268 + 24 + 1 = 6,293` closes exactly.
That matches the committed `DropReason::InsideComment` literal of `(3 bodies, 25 calls)` in `rust/crates/rexx-extract/tests/extract_bif.rs:230`.
The brief's "over a hundred assertions that never run" is off by roughly a factor of five.

**`::options novalue`: 43 of 76 files, not 38.**

```
$ /bin/grep -a -l -i '^[[:space:]]*::options.*novalue' *.testGroup | wc -l      -> 38
$ /bin/grep -a -h -i '^[[:space:]]*::options' *.testGroup | sort | uniq -c
     35 ::options novalue error
      5 ::options all syntax
      2 ::options novalue syntax
      1 ::options novalue error
$ /bin/grep -a -l -iE '^[[:space:]]*::options[[:space:]].*(novalue|all)' *.testGroup | wc -l  -> 43
```

And the mechanism is real, measured on the oracle from a fresh empty directory:

```
$ ( ulimit -v 1048576; LD_LIBRARY_PATH=.../lib .../rexx /abs/o1.rex )   # say abc + ::options all syntax
exit=158, stderr: Error 98.986:  Reference to unassigned variable "ABC".
$ ( ... /abs/o2.rex )                                                    # say abc, no directive
exit=0, stdout: ABC
```

So `::options all <cond>` does enable NOVALUE.
Matching only the `novalue` spelling would have applied the symbol-equals-own-name resolution in `BEEP`, `CONDITION`, `DATE`, `LINEOUT` and `XRANGE`, where the idiom raises — the dangerous direction.
**The plan must be corrected; see Important 2.**

### 2. The `is_symbol_char` reuse defect — fixed, and `base/keyword` did not regress

`rust/crates/rexx-extract/src/bif.rs:139-141` now has its own predicate (`alphanumeric || _ . ! ?`) with the difference stated in its doc comment.
The `keyword.rs` diff is **visibility only** — five `fn` became `pub(crate)`, nothing else — so no behaviour there could move.

```
$ REXX_KEYWORD_GATE=1 cargo test -p rexx-exec --test keyword_assertions      # exit 0
888 of 896 bodies passing, carrying 1737 of 1773 assertSame calls
```

Identical to the `1c519dfc` baseline. No regression.

### 3. The three new drop reasons are genuine, and the direction is right

Verified by deleting their subject: I disabled `usable`'s two checks and the `SideEffectingAssertion` block in a scratch copy of `bif.rs`, re-ran the extractor and the harness, and restored (`git status` clean afterwards).

| | rows | passing | failing |
|---|---|---|---|
| committed | 4,999 | 4,920 | 79 |
| three reasons disabled | 5,017 | 4,926 | 91 |

So **18 rows come back** (`C2X` +1, `COMPARE` +1, `COPIES` +1, `DELSTR` +8, `INSERT` +1, `VALUE` +6) and **12 of them are MISMATCHes**.
The 11 `NonUtf8Source` calls land in `C2X`/`COPIES`/`DELSTR`/`INSERT`, all four of which `iconv -f UTF-8 -t UTF-8` rejects.
The 1 `ClockDependent` call is `COMPARE.testGroup:605`, `assertSame(COMPARE(SUBSTR(TIME(),1,2)||'N', RIGHT(TIME('H'),2,'0'), ...), '0')` — two genuine clock reads, barred by D11.
The 6 `SideEffectingAssertion` calls are `VALUE.testGroup`'s setter chain.
None of this is coverage discarded: each is a row asserting something no interpreter should be measured against.
(The gate's word for the number is off; Minor 1.)

### 4. `BEEP` — a real gap, correctly recorded

```
$ ( ulimit -v 1048576; ... rexx /abs/o3.rex )   # retc = beep(262,1); say '['retc']'; say length(retc)
exit=0, stdout: "[]" then "0"
$ cargo run -q --bin rexx-run -- /abs/b.rex
exit=213, stderr: Error 43.1:  Could not find routine "BEEP".
```

Confirmed on both sides.
It is recorded in three durable places: the three `ANOMALY` rows in `rust/corpus/bif-exempt.txt`, that file's own header (with the `InternalPackage.cpp:199` citation), and `docs/superpowers/plans/phase-4c-gate.md:331-334` and `:357`.
Nothing in Phase 4 owns it, and the gate says so. Correct handling.

### 5. The headline's population — stated, but only in prose

Gate §12 gives the denominator in the same sentence as the headline (`out of 6,293 assertSame calls`), states conservation (`4,999 + 1,294 == 6,293`), and names the largest drop categories as Phase 5's at `:359`.
That is honest.
The summary table row is thinner; Minor 3.

### 6. The `TIME('R')` catchers — the substitutes fire, the brief's named one does not

Applied mutation row 8 (`interp.elapsed_anchor = Some(stale);` -> `let _ = stale;`) and ran `cargo test -p rexx-exec --lib`:

```
test result: FAILED. 505 passed; 2 failed
    builtin::datetime::tests::a_callees_own_time_r_leaks_into_the_caller_after_it_returns
    builtin::datetime::tests::time_r_resets_relative_to_the_last_reset_not_program_start
```

Exactly the two the script's row-8 comment names, and `two_time_r_reads_in_one_clause_answer_identically` passes under the mutation, precisely as the report says.
The brief's declared catcher was wrong and the correction is written at the row (`rust/scripts/mutate-4c.sh:556-563`). Tree restored clean.

---

## Findings

### Important 1 — Step 4's own change is unpinned: reverting it leaves the whole workspace green

`rust/crates/rexx-exec/tests/collect_stress.rs:128-132`

This task's Step 4 deliverable is the third entry in `collect_stress.rs`'s `read_subset` call, and criterion 4's whole amendment rests on it ("Task 15 widened the call site to the three-file union", `phase-4c-gate.md:72`).
Nothing asserts it.

Measured, by deleting the subject — I removed the single line `&corpus_dir.join("phase-4c.txt"),` and ran the full workspace:

```
$ cargo test --offline --workspace --no-fail-fast     # exit 0
1286 passed, 0 failed, 75 binaries, 0 FAILED blocks
```

Byte-identical to the unmutated result.
`the_l0_subset_passes_again_under_collect_on_every_allocation`'s four assertions — non-empty subset, no mismatches, no zero-collection program, non-zero total — all still hold over the 42-program 4a/4b union, and `phase_4c_subset_matches_the_committed_list` lives in `coverage.rs` and pins the *file's contents*, not which harness reads it.

**Failure scenario:** a later task edits the `read_subset` list (or reverts it during a merge), the builtins silently leave the collector's reach again, and criterion 4 reports MET over a subset that calls no builtin — which is verbatim the defect this criterion was amended to close, and verbatim what `coverage.rs:500-516` records happening to `phase-4a.txt`.
`corpus.rs` and `coverage.rs` carry the same hazard, but for them a shrunken subset moves a reported "N of N" figure that other criteria read; here nothing moves at all.

**Fix:** one assertion in `collect_stress.rs` — the subset must contain a named builtin-calling program (`lang/state_builtins.rex` is the obvious one), or its length must equal the sum of the three committed lists.
Cheap, and it is the same device `EXPECTED_SUBSET` already is one level up.

### Important 2 — the two corrected figures were not written back into the plan

`docs/superpowers/plans/2026-08-04-phase-4c-builtins-and-parse.md:1751` and `:1780`

Both still read as they did before this task measured them:

* `:1751` — "roughly **6,150-6,170 in live method bodies, 120-135 inside block comments**"; measured 6,268 in code and 25 commented.
* `:1780` — "**38 of the 76 files** carry it … in the other 38 files it evaluates to its own uppercased name"; measured 43 and 33.

The `::options` correction reached the gate (`:348`) and `bif.rs`'s module doc (`:63-64`).
The block-comment correction reached only the task report and the pinned literal in `extract_bif.rs`.

**Failure scenario:** this is the project's own recorded rule — "When a plan or brief is wrong, correct the plan, not the message that carries the work", earned when a `trace r` divergence recorded only in a review summary was absent from the next brief.
4c is the last sub-phase so no brief regenerates from these lines, but the plan is the durable artifact a Phase 5 reader consults, and `:1780`'s figure is the one that decides, in five files, whether a row is resolved in the direction that asserts a value the interpreter must never produce.
A Phase 5 extractor built from the plan reintroduces exactly the defect this task avoided.

**Fix:** edit both lines to the measured values, saying what was removed and why, as the plan's own convention requires.

### Minor 1 — "twelve confidently-wrong rows" understates the measured 18

`docs/superpowers/plans/phase-4c-gate.md:347`

Measured (§3 above): disabling the three reasons restores **18** rows, of which **12** are MISMATCHes and 6 pass.
"Twelve" is defensible as the mismatch count and indefensible as the row count; the sentence reads as the latter.
The claim it supports — that conservation certified all of them — holds either way.
Say "eighteen rows, twelve of them reported as mismatches" and it is checkable.

### Minor 2 — criterion 12's "with what would unblock it" is not enforced for the derived-outcome attributions

`rust/crates/rexx-exec/tests/bif_assertions.rs:820-821`

`every_exempt_attribution_is_a_known_phase_or_a_declared_outcome` whitelists `MISMATCH`, `RAISE-MISMATCH` and `ANOMALY` alongside the four phase names.
`MISMATCH` is not "what would unblock it" — it is "this row is wrong".
So a genuine value divergence found later can be added to `bif-exempt.txt` with attribution `MISMATCH`, and criterion 12 reports MET with no phase owning it.

Not live today: the 81 committed rows are 47 `4c`, 22 `Phase 5`, 9 `UNATTRIBUTED:an environment symbol`, 3 `ANOMALY`, and **zero** `MISMATCH` — which is a genuinely strong result and worth stating in the gate, because it means no value divergence is being absorbed.
Absorbing one would still show in a diff, which is why this is Minor rather than Important.

### Minor 3 — the assessment table's row 12 drops the denominator

`docs/superpowers/plans/phase-4c-gate.md:198`

Reads `4,920 of 4,999 value rows, 184 of 186 raise rows` with no `of 6,293`.
The report's own framing — "98% of the 79% that could be lifted" — appears nowhere in the gate.
§12's prose carries the denominator two lines in, so a reader who reads the section is not misled; a reader who skims the table is.

### Minor 4 — `has_novalue_option` would read `::options novalue off` as enabling NOVALUE

`rust/crates/rexx-extract/src/bif.rs:533-542`

The predicate is "any word on the `::options` line equals `novalue` or `all`", so a disabling spelling reads as enabling.
Not present at ooTest r13178 — the corpus holds only `novalue error`, `novalue syntax` and `all syntax` — and the direction is conservative (it drops rows rather than resolving them wrongly).
But the doc comment states the contract as "whether the file carries a directive that makes reading an unassigned symbol raise", which that spelling falsifies.
Either narrow the match to the condition-trap forms or say in the doc that the disabling spelling is unhandled and absent at r13178.

---

## What was checked and found correct

**Deliverables.** All five present: `crates/rexx-extract/src/bif.rs`, `crates/rexx-exec/tests/bif_assertions.rs`, `rust/corpus/bif-exempt.txt`, `rust/scripts/mutate-4c.sh`, `docs/superpowers/plans/phase-4c-gate.md`, plus `crates/rexx-extract/tests/extract_bif.rs` (not in the brief's file list, and it is where conservation is asserted).

**The set assertion is ungated and runs.** `the_exempt_set_matches_the_current_failures` carries no `#[ignore]` and no `gate_mode()` guard; it reads `collect()` and `committed_exempt()` unconditionally and asserts set equality **and attribution equality** in both directions (`bif_assertions.rs:466-518`).
`cargo test -p rexx-exec --test bif_assertions` runs 5 tests with no env var and it is one of them.

**Conservation is asserted, not printed.** `extract_bif.rs:174-191` asserts `rows + dropped == calls` **per group**, over `keyword::count_assert_same` as an independent denominator.
`the_drop_reasons_account_for_every_call_outside_the_population` pins all seventeen reasons including the six standing at zero, and asserts `EXPECTED.len() == DropReason::ALL.len()` so a new variant is a red test rather than a silently unlisted bucket.
`the_row_floor` (4,000 against 4,999 actual) is a real floor.
`collect()` panics on a missing `ootest/` rather than skipping (`bif_assertions.rs:147-152`), so the harness cannot be vacuous by absence.

**The denominator is exact.** Independently counted: 6,293 `self~assertSame`, 5 `assertSameList`, and the 424-call tail is exactly `assertTrue` 232 + `assertEquals` 116 + `assertFalse` 76 = **424**, stated at `extract_bif.rs:27-28` as out of scope. (The corpus also carries 13 `assertIsA`, 12 `assertFile`, 8 `assertFail`, 7 `assertNotSame`, 4 `assertNotEquals`, which nothing claims either.)

**`the_falsification_proof` has teeth** and is paired with its adjacent success — it perturbs the first passing row's expression inside fresh parentheses (avoiding the regrouping trap `assertions.rs` hit), requires a `Mismatch`, then requires the unperturbed row to still pass.
`a_raise_row_needs_the_sub_number_too` is constructed rather than found and covers both negatives (wrong sub, and no raise at all).

**The mutation script.** All nine `OLD` patterns occur **exactly once** in the tree today (checked programmatically); none is a semantic no-op.
`--no-fail-fast` on every suite run; `timeout ${RUN_TIMEOUT}` on both instruments with exit 124 -> `INFRA_FAILURE`; binary count read from `Running`/`Doc-tests` headers on **stderr** and pass/fail from **stdout**, in separate files, never `2>&1`; `BASELINE_BINARIES` established by the script's own first baseline and never hardcoded; `require_clean` refuses a dirty tree; restore is a file copy, not `git checkout --`; no row is `PASSED`/`PASSED` so the justification rule was not exercised, and the rule is written down anyway (`:106-109`).
Row 9 is genuinely distinct from `mutate-4b.sh` row 9 (`push_temp(argument.value())` in `run.rs`) — it deletes `interp.roots.push_temp(old)` in `builtin/datatype.rs`, a root a builtin holds across its own allocation, as criterion 4 requires.
The `INFRA_FAILURE` paths in `corpus_status`/`suite_status` survive `set -euo pipefail` — verified with a reduced reproduction rather than argued.

**The gate's strongest coverage claim reproduces.** Applied mutation row 5 (the comma fence) and ran the full workspace:

```
$ cargo test --offline --workspace --no-fail-fast     # exit 101
failing tests: the_exempt_set_matches_the_current_failures
binaries: 75   (no truncation)
```

One catcher over the whole workspace, and it is the harness this task added. Tree restored clean.

**Criterion 10's attribution table is exact.** Row counts of `rust/corpus/keyword-exempt.txt` at each of the thirteen commits the gate names: 796, 795, 778, 777, 776, 772, 117, 112, 111, 16, 15, 15, 8 — every removal count in the table reconciles.

**Wiring.** `corpus.rs:461-465` and `coverage.rs:673-677` read the three-file union; `coverage.rs:557/626/638` are the three deliberate per-phase pins; `collect_stress.rs:128-132` is this task's fix. `phase-4c.txt` holds 8 programs, including `lang/state_builtins.rex` and `lang/builtin_argument_range.rex`, so the union does gain builtin-calling programs per the brief's trap 3.
`corpus/README.md` gained five 4c rules (the four the brief names plus the `::`-directive one), and its cited guard `assert_program_has_only_routine_directives` exists at `coverage.rs:151`.

**Constraints.** Zero `unsafe` in the new files; zero em-dashes in any new `.rs`, `.sh` or corpus file (the 12 in `corpus/README.md` are all pre-existing — the diff's added lines contain none); the one comment removed in `collect_stress.rs` was a stale history block whose central claim this change made false, which the tree's own rule requires correcting rather than hedging.

## Every figure, with its command and unpiped exit status

All from `rust/`, stdout and stderr as separate files, exit status read unpiped.

| figure | command | result | exit |
|---|---|---|---|
| workspace suite | `cargo test --workspace --no-fail-fast` | 1,286 passed, 0 failed; 75 header lines (stderr), 76 `test result:` (stdout) | 0 |
| corpus gate | `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` | `50 of 50 matching` | 0 |
| assertions gate | `REXX_ASSERTIONS_GATE=1 cargo test -p rexx-exec --test assertions` | `4224 of 4259 rows passing` | 0 |
| keyword gate | `REXX_KEYWORD_GATE=1 cargo test -p rexx-exec --test keyword_assertions` | `888 of 896 bodies … 1737 of 1773` | 0 |
| bif gate | `REXX_BIF_GATE=1 cargo test -p rexx-exec --test bif_assertions` | `mode: STRICT`; `4920 of 4999 value rows … 184 of 186 raise rows` | 0 |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | clean (rexx-extract and rexx-exec recompiled in this run) | 0 |
| format | `cargo fmt --all --check` | clean | 0 |
| builtin status | `corpus/builtin-status.txt` row census | 81 rows: 66 `implemented`, 15 `excluded` | — |
| trace prefixes | `trace_oracle.rs` | `WITNESSED_PREFIX_COUNT` 16, `OUT_OF_SCOPE_PREFIX_COUNT` 3 | — |
| bif exempt | `corpus/bif-exempt.txt` row census | 81 rows: 47 `4c`, 22 `Phase 5`, 9 `UNATTRIBUTED:…`, 3 `ANOMALY` | — |

Every one matches the implementer's report exactly.

`mutate-4c.sh` was **not** run end to end — nine mutations against two instruments, one of which is the full workspace, is roughly twenty full suite runs.
Instead its declarations were verified by reading each mutation, by confirming all nine patterns still match exactly once, and by running rows 5 and 8 against their declared instruments (both as declared).
⚠️ Cannot verify from this review: the observed status pair for rows 1, 2, 3, 4, 6, 7 and 9, and the per-row in-crate catcher counts in the gate's table other than rows 5 and 8.
