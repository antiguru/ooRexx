# Phase 4c exit gate — assessment

Phase 4c gave `rexx-exec` a builtin function table and an input model.
It delivered 66 of the 81 builtin names `rexx_inventory::builtins::NAMES` carries, the `40.x` incorrect-call family, the `PARSE` template engine in all eight trigger kinds with the comma fence and the `.` placeholder, every `ParseSource` variant, the `ARG` and `PULL` instruction spellings, the `ADDRESS` instruction's environment-naming forms, `::ROUTINE` dispatch with the `>I>` and `<I<` trace prefixes, and the compound-`DO` control-variable fix, all measured against `build/bin/rexx`.

---

## The criteria

**These are 4b's ten, carried forward with the amendments D14 and the 4c plan name, plus two the plan adds.**
They were fixed in `docs/superpowers/plans/2026-08-04-phase-4c-builtins-and-parse.md` before this task ran anything, which is what makes them criteria rather than descriptions of what happened -- 4b's own gate makes the same distinction and for the same reason.

**Every criterion below was checked against one question before it was written: what degenerate implementation satisfies this, and would deleting its subject leave it green?**
Four criteria in 4a could not fail; 4b's review found more.
The per-criterion notes say, for each one, what its falsification is, and where a criterion cannot see something the notes say so in the criterion rather than in a footnote.

### 1. The named L0 subset — the **union** of `phase-4a.txt`, `phase-4b.txt` and `phase-4c.txt` — runs with zero divergences, and the union satisfies the variant-coverage property

> Every program named by the union of the three subset files matches the oracle on stdout, on stderr and on exit status, under `REXX_CORPUS_GATE=1`.
> And the union constructs at least one instance of every in-scope `InstructionKind`, `ExprKind`, `LoopKind`, `PrefixOp`, `EndStyle` and `Trace` variant, and every `Operator`, where "in-scope" is `tests/owners.rs`'s own table and not a list in this document.

**The union of three, not of two, and until 4c's Task 9 it was two.**
`phase-4b.txt`'s own header excludes `PARSE` by definition, so every witness for every construct 4c added had to land in a third file; a criterion reading only the first two would have measured 4c's delivery at zero and passed.
Task 9 widened `tests/corpus.rs` after finding that Tasks 7 and 8's witnesses were being *parsed* by `coverage.rs` and *run* by nothing.

**What this criterion cannot see, unchanged from 4b.**
The coverage property enumerates variants **individually and asserts nothing whatever about their combinations**.
A variant is witnessed the moment it appears in any program, regardless of what else appears in the same clause.
That is exactly how 4a shipped `a. = 5; say a. + 1` aborting at rc 101 through a byte-identical corpus.
A reader must not take a pass here as evidence that combinations were exercised.

*Falsification:* deleting a program from any subset file fails that file's own committed-list equality -- `phase_4a_subset_matches_the_committed_list`, `phase_4b_subset_matches_the_committed_list`, `phase_4c_subset_matches_the_committed_list`, the last added by Task 7 for exactly the reason 4b's gate found: for nine of `phase-4b.txt`'s twelve entries, deleting that one line left `coverage.rs` green, including one criterion's only witness.
Mutating the interpreter fails the differential, exercised for real under criterion 6.

### 2. The `base/expressions` assertion table, `tests/assertions.rs` — a **regression check**, and this gate states plainly that it is not evidence of 4c's delivery

> Every row of the extracted `base/expressions` table is evaluated byte for byte and never numerically; every row that does not pass is on a committed exempt list naming what would unblock it; and the exempt set is policed in **both** directions.

**Deleting the whole of 4c leaves this criterion green, and that is a prediction this gate makes rather than an observation it reports.**
All 35 rows of `EXEMPT` are `unblocked_by: "Phase 5"` and **no row reads `4c`**, so nothing this sub-phase delivered could move one.
4b's gate predicted this criterion would report the same number at 4c's gate that it reported there, and said that sameness would be the correct result rather than a stall.
It is carried here as a regression check on 4a's and 4b's work, and a reader must not read its pass as 4c evidence.

*Falsification:* removing a row from `EXEMPT` while it still fails, or leaving a row on it after it starts passing, fails `the_exempt_set_matches_the_current_blocked_rows` under `REXX_ASSERTIONS_GATE=1`.
A row compared numerically rather than byte for byte fails `the_falsification_proof`.

### 3. Trace output byte for byte, plus the prefix-coverage measure — and the **indent this criterion does not cover**

> Every committed trace witness's stderr matches the oracle byte for byte under the comparison DEVIATION 0 defines; every witness still emits every prefix the table claims for it; and the fraction of the oracle's nineteen prefixes that is witnessed is a **committed literal that an assertion reads**, with an owning phase named for every prefix that is not.

**The target is 16 of 19.**
`>.>`, `>I>` and `<I<` are 4c's three.
The three that remain each name an owner in `PREFIX_COVERAGE`: `+++` is Phase 7's under D-P, and `>M>` and `>N>` are Phase 5's.
`trace_oracle.rs`'s `WITNESSED_PREFIX_COUNT` is 16, `OUT_OF_SCOPE_PREFIX_COUNT` is 3, and the test asserts their sum equals the oracle's own `trace_prefix_table` size, so neither number can drift on its own.

**THE CORPUS CANNOT PIN A TRACE INDENT, FOR ANY PREFIX, AND THIS CRITERION DOES NOT CLAIM OTHERWISE.**
DEVIATION 0 normalises the run of spaces after the marker, on both sides, in `tests/support/mod.rs`, shared by `corpus.rs` and `trace_oracle.rs`; the committed `.expected` files are normalised at comparison time too.
So an off-by-two indent is invisible to **every corpus-based instrument in this repository**.
A criterion claiming "trace output byte for byte" over the corpus without this sentence would be claiming coverage that does not exist -- 4b's Task 4 C1 was a real, user-visible indent bug and the corpus reported 34 of 34 with it live.

What still carries the indent is three `run.rs` unit tests, outside either harness's comparison function: `one_two_and_three_enclosing_dos_indent_by_two_four_and_six`, `the_corrected_28x_indent_rule_matches_all_fourteen_probed_shapes`, and `an_absorbed_whencases_escaping_false_branch_reports_end_at_its_own_residual_indent`.
Those are shallow-depth and loop-free; **the indent of a 4c construct -- a `PARSE` target's value line, a `>I>` routine entry -- is unpinned**, and this gate says so rather than leaving a reader to infer it from the normalisation.

*Falsification:* fabricating a witness's `.expected` so it no longer contains a prefix it is named for fails `every_witness_still_emits_every_prefix_it_is_named_for` by name; dropping a prefix from the coverage table to flatter the fraction fails the equality against `support::TRACE_PREFIXES`, read from the oracle's own table.

### 4. Collect-on-every-allocation over the **union**, with a **builtin-shaped** negative control

> The union subset passes again under `run_program_collect_every_alloc`, byte-identical to `run_program`; every program in it performs a non-zero number of collections, checked **per program** and not only in aggregate; and a negative control deleting a root a **builtin** holds makes at least one test fail.

**Until this task, no builtin was under the collector at all.**
`tests/collect_stress.rs` read `phase-4a.txt` and `phase-4b.txt` only, and the 4a/4b subset **calls no builtin**, so every allocation the 66 names make was outside `run_program_collect_every_alloc`'s reach -- the same defect 4a's version had when it ran 29 programs and zero call frames.
Task 15 widened the call site to the three-file union.

**The control must not be `mutate-4b.sh` row 9.**
Builtins reuse `resolve_and_run_call`'s argument evaluation, so the obvious root in that window is `self.roots.push_temp(argument.value())`, which is verbatim that row; re-running it would re-test 4b.

**Finding a builtin-shaped one required establishing that the obvious one does not exist.**
A builtin's own *result* holds no root: `resolve_and_run_call`'s builtin arm hands it straight back, rooted by whichever caller receives it exactly as a callee's `RETURN` value is, and that is stated deliberately in the code rather than being an omission.
What exists instead, and what the control deletes, is a root a builtin holds over a value of its own across an allocation it makes itself: `VALUE(name, newvalue)` reads the old value out of the stem and must keep it alive across `stem_set`, because on a never-touched stem `stem_get` and `stem_set` allocate on the same branch.

*Falsification:* a mode that never collects passes the byte-identity check by construction, which is why the per-program non-zero-collections assertion exists; the control is `mutate-4c.sh` row 9 and is run rather than cited.

### 5. Every variant executes or fails loudly, **naming an owner**, at **arm** granularity where a variant is split

> An enumerating test with no wildcard arm asserts that each `InstructionKind`/`ExprKind` variant is in 4c's named set or produces `NOT_IMPLEMENTED_EXIT` with a message naming the owning sub-phase; and where a variant is split across phases, the check is per **arm**.

*Falsification:* removing the phase from a loud message, or letting an unimplemented arm run rather than exit loudly, fails `every_out_of_scope_variant_fails_loudly`; a new variant in any enumerated enum is a compile error, not a silently shrinking check.

### 6. Mutation control: `rust/scripts/mutate-4c.sh`, 4c-shaped mutations, carrying the guard and three additions this phase forced

> A committed list of one-line mutations to the code 4c added, each of which **declares in advance what each of two instruments should say about it, and is measured against that declaration** -- so an unexpected survival and an unexpected catch are both failures.
> The script carries (a) the exact-match, exactly-once application guard, (b) a baseline pass of the unmutated tree before the first mutation and after the last restore, (c) a three-way `PASSED`/`DIVERGED`/`INFRA_FAILURE` classification that never folds an infrastructure failure into either bucket, (d) `--no-fail-fast` on every run, (e) a **binary-count assertion measured from its own baseline**, and (f) a wall-clock timeout per run, classified `INFRA_FAILURE`.

**(d), (e) and (f) are new here and each is a measured defect in this phase rather than a precaution.**

* `cargo test --workspace` stops after the first test binary that fails, so under a mutation the run is truncated at the first catcher and every later binary silently never executes.
  That is invisible in a green baseline.
  Measured at Task 9: three of six "caught by this test and nothing else" claims were false.
* The truncation guard counts `Running`/`Doc-tests` **header** lines, not `test result:` lines, and the two genuinely differ: `Doc-tests rexx_exec` prints two `test result:` blocks from one process.
  The count is **derived from the script's own baseline in the same run**, never written down, because every task that adds a test binary moves it -- 72/73 at Task 9, 75/76 at Task 15.
  The headers are on stderr and the `test result:` lines on stdout, so the two descriptors are captured separately.
* A mutation can hang instead of failing: measured at Task 14, a mutation left `keyword_assertions_differential` running for over nine minutes on a body whose loop the mutation had made unterminating.

**A `PASSED`/`PASSED` declaration needs a written justification at its own row**, naming the instrument that should have caught it and why it cannot.
Without that rule, declaring a mutation a survivor because nothing happens to test it scores "as declared" and reports coverage that does not exist.

*Falsification:* pointing the oracle at a nonexistent path must abort at the first baseline check, before any mutation runs.
A mutation whose pattern no longer matches is fatal, never a skip.

### 7. Zero `unsafe`, clippy clean, `cargo fmt` clean

> `unsafe_code = "forbid"` stands; `cargo clippy --workspace --all-targets -- -D warnings` is clean; `cargo fmt --all --check` from `rust/` is clean.

`cargo fmt --all --check`, **not** `cargo fmt --edition 2024 --check`: `cargo fmt` has no `--edition` flag and that spelling exits 2 before doing any work.

### 8. A condition trap is witnessed by **a value a handler set**, never by an exit code

> At least one differential corpus program traps a condition and proves it by a value the handler wrote into the variable pool -- a value that is neither the flag variable's derived name nor its unset rendering -- and the program is compared to the oracle byte for byte.

Carried forward from 4b unchanged.
The reason it is phrased this way is that the obvious criterion cannot fail: "the program exited 0 after the trap fired" is satisfied by a program that never raised, and in Rexx an unset variable reads as its own uppercased spelling, so a handler flag left unset renders as plausible-looking data rather than as an obvious blank.

**4c adds a second instance of the same trap, and it is worth naming because the shape recurs:** a `PARSE` criterion asserting "exited 0" passes for a program that parsed nothing.
`lang/parse_triggers.rex` and `lang/parse_sources.rex` assert the assigned values, chosen so that an unset target renders as its own derived name and is recognisably wrong.

### 9. `PUSH`/`QUEUE` — 4b shipped this undifferentiated, and this gate records **what closed it**

> The queue's `PUSH`-at-head / `QUEUE`-at-tail order is asserted against oracle-measured values; and 4b's recorded gap -- that no differential instrument could observe what the queue *stored* -- is either still open or is closed by a named instrument that runs today.

**4b's gate shipped `PUSH`/`QUEUE` with their storage verified only in-crate**, and said so deliberately: nothing that reads the queue back existed before 4c, so a differential run could observe what was written and how it traced, never what was stored.

**Task 8 closed it, and the closure is cited to the row that runs.**
`crates/rexx-exec/tests/input_oracle.rs`'s `queue-round-trip` row drives the `rexx-run` binary against the oracle today.
It is cited there rather than to `corpus/lang/pull_queue.rex`, because `tests/corpus.rs` did not read `phase-4c.txt` until Task 9 -- a corpus program is not an instrument until something runs it.

### 10. The `base/keyword` L1 table is policed in both directions — a **measurement this gate reports**, and the rows that fired are attributed to the task that removed them

> `rust/corpus/keyword-exempt.txt`'s committed set matches the current failures exactly, in both directions.
> The pass **rate** is reported and is not a threshold.
> And this gate says **which task removed which rows**, from the file's own git history.

**The 790 this plan was written around is spent, and it was always an upper bound on what 4c fixes rather than a measure of its remaining surface.**
The residue proves it: of the rows left, three read `RAISED` and cannot be made to pass by implementing anything.
Two are `CALL` bodies that fail **under the C++ oracle itself** (`Error 43, Routine not found`, because the extraction dropped the `::routine`s they call) and one is `NUMERIC::test_42`, which exits 3 by falling through into its own `dig:` label.
**No task owned those rows**, so they came off ungated across the family tasks, and the attribution table below is what makes that visible rather than letting the criterion be green by construction.

*Falsification:* the `is not on the committed` direction fires when a body starts failing; the `now PASSES` direction fires when one starts passing.
Both live in `rust/crates/rexx-exec/tests/keyword_assertions.rs`.

### 11. NEW: the builtin implemented/not-implemented boundary is **derived from a live differential run**, in both directions

> `rust/corpus/builtin-status.txt` holds one row per name in `rexx_inventory::builtins::NAMES`, every row derived by running that name's probe through **both** interpreters, and the derivation is asserted equal to the file in both directions -- a row that disagrees is red whichever way it disagrees.

**"Each of the 66 names is recognised" is satisfied by 66 stubs returning the null string, which is why this criterion is phrased over a differential run rather than over a name table.**
The file cannot drift ahead of the executor or lag behind it, because it is not hand-maintained: editing a row on its own only moves the failure.

*Falsification:* **Task 2's Step 5, the interpreter mutation and not the file edit.**
Deleting `LENGTH`'s dispatch arm must flip its row `implemented` -> `loud` on its own.
That is what proves the harness observes the interpreter rather than a name table, and it was run at Task 2 because Task 1, which wrote the harness, had no dispatch to delete.

### 12. NEW: the `base/bif` L1 table — a **measurement**, not a threshold, with the set assertion **ungated**

> Every `self~assertSame` in `ootest/ooRexx/base/bif` is either an extracted row or is accounted for by a named `DropReason`, with `rows + dropped == calls` holding per group; every row that does not pass is on `rust/corpus/bif-exempt.txt` with what would unblock it; and that set is policed in **both** directions **under a plain `cargo test`**.
> The pass rate is reported and is not gated.

**No threshold, because `base/bif` is the whole builtin surface** -- the fifteen names D4 excludes, and everything Phase 5 and Phase 7 own, sit inside it.
A number over that is a progress signal, not a statement about 4c's scope.

**The set assertion is not behind `REXX_BIF_GATE`, and that is the criterion's teeth.**
A measurement nobody gates on, policed by an assertion nobody runs, is coverage that does not exist -- so `the_exempt_set_matches_the_current_failures` runs on every `cargo test`.
`REXX_BIF_GATE=1` adds only a non-zero exit for a caller who wants one.

**The attribution column is derived from `rexx-exec`'s own loud message**, exactly as `keyword-exempt.txt` derives it, so the file cannot drift from `instruction_owner`/`expr_owner`.
The same two limits apply: a derived owner says what a row hits *first*, and `Loud::unresolved_call` carries a fixed `4c` for every builtin name the crate runs nothing for, whichever phase actually owns it.

*Falsification:* `the_falsification_proof` perturbs a passing row's expression and requires exactly that row to fail; `a_raise_row_needs_the_sub_number_too` requires a raise row expecting `40.12` not to be satisfied by a program raising `40.5`, and a row expecting a raise not to be satisfied by a program that raises nothing.
Conservation is falsified by `every_call_is_a_row_or_a_counted_drop` per group, and by `the_row_floor`, which is what stops conservation being satisfied by an extractor that drops everything.

---

## Assessment

Every figure below was taken at commit `1c94e50b`, with the command shown, reading the exit status unpiped.

| # | criterion | result |
|---|---|---|
| 1 | union subset + variant coverage | **MET** — 50 of 50 matching |
| 2 | `base/expressions` table | **MET as a regression check** — 4,224 of 4,259, unchanged from 4a and 4b |
| 3 | trace byte for byte + prefix coverage | **MET, with the indent stated as uncovered** — 16 of 19 |
| 4 | collect-on-every-allocation + builtin-shaped control | **MET** |
| 5 | every variant loud, naming an owner | **MET** |
| 6 | `mutate-4c.sh` | **MET** — 9 of 9 as declared, one declaration corrected to what was measured |
| 7 | no `unsafe`, clippy, fmt | **MET** |
| 8 | a trap witnessed by a handler's value | **MET** |
| 9 | `PUSH`/`QUEUE`'s 4b gap | **CLOSED**, by a named instrument that runs |
| 10 | `base/keyword` L1, both directions | **MET** — 888 of 896 bodies, and the per-task attribution below |
| 11 | builtin status derived from a live differential run | **MET** — 66 implemented, 15 excluded, 81 rows |
| 12 | `base/bif` L1, ungated set assertion | **MET as a measurement** — 4,920 of 4,999 value rows, 184 of 186 raise rows |

### 1. The union subset

`REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` reports **50 of 50 matching** at exit 0, the subset being `phase-4a.txt`'s 30, `phase-4b.txt`'s 12 and `phase-4c.txt`'s 8.
`corpus.rs`'s dated-row convention was followed: the figure is a new row naming the commit it is true at, not an edit of the 47 the previous row carries.

### 2. The `base/expressions` table

`REXX_ASSERTIONS_GATE=1 cargo test -p rexx-exec --test assertions` reports **4,224 of 4,259 rows passing** at exit 0, with 35 RUNTIME-BLOCKED.
All 35 are `unblocked_by: "Phase 5"` and none reads `4c`.
This is the same number 4a and 4b reported, and 4b's gate predicted it would be.
**Deleting the whole of 4c leaves this criterion green.**

### 3. Trace

`WITNESSED_PREFIX_COUNT` is 16 and `OUT_OF_SCOPE_PREFIX_COUNT` is 3, asserted to sum to the oracle's own table.
`>.>`, `>I>` and `<I<` moved from owned to witnessed during 4c; `>M>` and `>N>` are Phase 5's and `+++` is Phase 7's.

**What this gate does not have, stated rather than left implicit: no instrument in this repository pins the indent of any 4c construct.**
Both differential harnesses normalise the run of spaces after the marker on both sides, and the committed `.expected` files are normalised at comparison time, so an off-by-two indent on a `PARSE` target's value line or on a `>I>` routine entry is invisible to all of them.
The three unnormalised indent witnesses are `run.rs` unit tests over 4a-era shapes.

### 4. Collect-on-every-allocation

`tests/collect_stress.rs` now reads the three-file union; before this task it read two, and the 4a/4b subset **calls no builtin at all**, so every allocation the 66 names make was outside the collector's reach.
`the_l0_subset_passes_again_under_collect_on_every_allocation` passes, with a non-zero collection count asserted **per program** over all 50.

**The widening put the builtins under the instrument; it did not, at this gate, find anything.**
No mutation in `mutate-4c.sh` was caught by the subset run — the control that fires is a hand-written case.
That is worth saying plainly, because "the collector now covers the builtins" and "the collector caught a builtin rooting bug" are different claims and only the first is true here.

The control is `mutate-4c.sh` row 9, and finding a builtin-shaped one required establishing that the obvious one does not exist: a builtin's own result holds no root, because `resolve_and_run_call`'s builtin arm hands it back for the caller to root exactly as a callee's `RETURN` value is.
What the row deletes instead is `VALUE(name, newvalue)`'s root over the old stem value across its own `stem_set`, and the test that fires is `values_compound_write_roots_the_old_value_before_the_stems_first_allocation` -- **alone**, over the whole workspace.

### 5. Loud failure with an owner

`every_out_of_scope_variant_fails_loudly` and `tests/owners.rs`'s tables pass as part of the workspace run.

### 6. `mutate-4c.sh`

**9 of 9 mutations behaved exactly as declared**, at exit 0, with the unmutated tree passing both instruments before the first mutation and after the last restore (50 of 50 matching; 75 test binaries, 1,286 passed, 0 failed, both times).

**One declaration was corrected to what was measured rather than left standing as a guess.**
Row 7 -- a `::ROUTINE` resolved before the builtin table -- was declared `PASSED`/`DIVERGED` on the reasoning that the resolution order had only an in-crate witness.
It measured `DIVERGED`/`DIVERGED`: `corpus/lang/routine_dispatch.rex` defines `::routine length` and `::routine 'max'` for exactly this purpose, and the corpus witness is the stronger of the two because it compares against the oracle rather than against this crate's own expectation.
The row now declares what it measures, and the correction is written at the row.

**What the run says about coverage, per finding rather than per round** (every run used `--no-fail-fast`, so "nothing else caught it" is measured rather than truncated):

| row | corpus | suite | the in-crate tests that fired |
|---|---|---|---|
| 1. optional argument reads as omitted | DIVERGED | DIVERGED | 12, across `convert` and `datatype` |
| 2. minimum arity bound off by one | PASSED | DIVERGED | 8, including both `base/bif` tests |
| 3. absolute `PARSE` trigger off by one | DIVERGED | DIVERGED | 5 |
| 4. `.` placeholder consumes no field | DIVERGED | DIVERGED | 2 |
| 5. comma fence does not advance | DIVERGED | DIVERGED | **1, and it is `bif_assertions::the_exempt_set_matches_the_current_failures`** |
| 6. bare `ADDRESS` keeps the current name | DIVERGED | DIVERGED | 4 |
| 7. `::ROUTINE` before the builtin table | DIVERGED | DIVERGED | 1 |
| 8. `TIME('R')` never moves the anchor | PASSED | DIVERGED | 2 |
| 9. `VALUE`'s old stem value unrooted | PASSED | DIVERGED | 1 |

**Row 5 is the clearest thing this gate can say about what criterion 12 bought.**
The comma fence's only in-crate catcher over the whole workspace is the `base/bif` exempt-set assertion this task added; before it, that defect was seen by the corpus and by nothing else.

**Row 8 is `TIME('R')`, and its `PASSED` on the corpus is decision D11 rather than a gap.**
`RANDOM`, `DATE` and `TIME` are barred from every differential corpus program because their answers differ between two runs of the same interpreter, so the construct's only gate is Task 12's own unit tests -- `time_r_resets_relative_to_the_last_reset_not_program_start` and `a_callees_own_time_r_leaks_into_the_caller_after_it_returns` are what fired.

**No row is declared `PASSED`/`PASSED`.**
4b's script carried one, I17's equivalent mutant; 4c produced no equivalent mutant worth running, and the rule requiring a written justification for such a row therefore did not have to be exercised.

### 7. Hygiene

`cargo clippy --workspace --all-targets -- -D warnings` exits 0; `cargo fmt --all --check` from `rust/` exits 0; `unsafe_code = "forbid"` stands.

### 8. Traps and `PARSE`, witnessed by values

Carried forward from 4b unchanged, and 4c's own version of the trap is in place: `lang/parse_triggers.rex` and `lang/parse_sources.rex` assert the assigned values rather than the exit status, chosen so an unset target renders as its own derived name.

### 9. `PUSH`/`QUEUE`

**4b's recorded gap is closed.**
Task 8 gave the round trip its first differential witness, and the closure is cited to the row that runs: the `queue-round-trip` row in `crates/rexx-exec/tests/input_oracle.rs`, which drives the `rexx-run` binary against the oracle.
It is not cited to `corpus/lang/pull_queue.rex`, because `tests/corpus.rs` did not read `phase-4c.txt` until Task 9 and a corpus program is not an instrument until something runs it.

### 10. `base/keyword`

`REXX_KEYWORD_GATE=1 cargo test -p rexx-exec --test keyword_assertions` reports **888 of 896 bodies passing, carrying 1,737 of 1,773 `assertSame` calls**, at exit 0.

**The 790 this plan was written around is spent, and here is which task spent it**, from `corpus/keyword-exempt.txt`'s own git history.
"Removed" is a row that disappeared because the body started passing; "re-attributed" is a row that stayed but whose `unblocked_by` moved.

| commit | task | removed | re-attributed | rows left |
|---|---|---|---|---|
| `fda5bff9` | 4b Task 11 created the file | — | — | 796 |
| `c10e40df` | Task 2, builtin dispatch and arity | 1 | — | 795 |
| `63a9ea9f` | Task 3, the 22 string builtins | 17 | — | 778 |
| `3af26b60` | Task 4, the seven word builtins | 1 | — | 777 |
| `ca0b63f9` | Task 5, the twelve conversion builtins | 1 | — | 776 |
| `510a070e` | Task 6, the seven numeric builtins | 4 | — | 772 |
| `da86c106` | Task 7, the `PARSE` template engine | **655** | — | 117 |
| `81944c76` | Task 8, the input model | 5 | — | 112 |
| `27606888` | Task 9, `ADDRESS` | 1 | 3 (`4c` -> `Phase 7`) | 111 |
| `f03d69f1` | Task 10, the state builtins | 95 | 2 (`4c` -> `defect:`/`RAISED`) | 16 |
| `7891fd4b` | Task 12, `DATE`/`TIME` | 1 | — | 15 |
| `e00528df` | Task 13, `::ROUTINE` dispatch | — | 2 (`4c` -> `RAISED`) | 15 |
| `d372dcdf` | Task 14, the compound-`DO` fix | 7 | — | **8** |

**790 was always an upper bound on what 4c fixes rather than a measure of its remaining surface, and the eight left prove it.**
Three read `RAISED` and **cannot be made to pass by implementing anything**.
`CALL::test_literal` and `CALL::test_expression` fail under the **C++ oracle itself** -- `Error 43, Routine not found`, because the extraction dropped the `::routine`s they call -- and `NUMERIC::test_42` exits 3 by falling through into its own `dig: Return digits()`, whose value becomes the program's exit status; the oracle exits 3 on the same program.
Three more read `Phase 7`: the `ADDRESS` bodies that issue a command or use the `WITH` redirection.
The last two read `4c`, and that label is `Loud::unresolved_call`'s fixed string rather than a real 4c debt -- `CALL::test_on_name` and `CALL::test_9` are waiting on `CHARIN` and `LINEIN`, which are whole exclusions and so Phase 7's.
**No task owned any of these rows**, so they came off ungated across the family tasks; the table above is what stops this criterion being green by construction.

### 11. The builtin status boundary

`corpus/builtin-status.txt` holds 81 rows: **66 `implemented`, 15 `excluded`**, and no `loud` or `divergent` row.
Every row is derived by running that name's probe through both interpreters and asserted equal to the file in both directions, so the file cannot drift ahead of the executor or lag behind it.

The falsification is **Task 2's Step 5, and it is an interpreter mutation rather than a file edit**: deleting `LENGTH`'s dispatch arm flips its row `implemented` -> `loud` on its own.
Task 1, which wrote the harness, could not run it, because there was no dispatch to delete yet; Task 2 ran it.

### 12. `base/bif`

`cargo test -p rexx-exec --test bif_assertions` reports **4,920 of 4,999 value rows passing and 184 of 186 raise rows passing, out of 6,293 `assertSame` calls**, at exit 0 in both REPORT and STRICT mode.

**This is a measurement and this gate does not gate on it.**
`base/bif` is the whole builtin surface: the fifteen names D4 excludes and everything Phase 5 and Phase 7 own sit inside it.

**The teeth are `the_exempt_set_matches_the_current_failures`, and it runs under a plain `cargo test`.**
All 81 failing rows are on `corpus/bif-exempt.txt`: 47 blocked on builtins `builtin-status.txt` calls `excluded`, 22 on message sends in their own operands (Phase 5), 9 on an environment symbol, and 3 on `BEEP`.

**`BEEP` is a real finding and it is not a builtin gap.**
Measured on the oracle, `retc = beep(262, 1)` exits 0 and answers the null string; this crate raises 43.1.
`BEEP` is not a builtin function at all -- it is a routine the interpreter's internal package registers (`interpreter/runtime/InternalPackage.cpp:199`) -- so its absence from `rexx_inventory::builtins::NAMES` is correct and the gap is in what this crate provides beside the builtin table.
Nothing in Phase 4's scope covers it.

**Conservation:** `4,999 rows + 1,294 dropped == 6,293 calls`, asserted per group, with every dropped call carried by one of seventeen named `DropReason`s.
`the_row_floor` is what stops that being satisfied by an extractor that drops everything.

**Three of those seventeen reasons were found by running the rows rather than by reading them**, and each was producing a row that asserted something the interpreter must never produce -- see the task report for the transcripts.
Briefly: six groups hold bytes that are not UTF-8 and a lossy read turns each into a three-byte replacement character; `VALUE`'s setter form assigns while `assertSame`'s own arguments are being evaluated; and `DATE`/`TIME` read the clock when given no input value.

---

## What this gate found

* **`tests/collect_stress.rs` was the last `read_subset` call site reading two files**, so no builtin was under the allocation-stress collector at all. Fixed here.
* **The `base/bif` extraction produced twelve confidently-wrong rows before three drop reasons were added.** Every one of them balanced the conservation invariant, which is exactly the failure mode that invariant cannot see.
* **`::options all <condition>` enables `NOVALUE`**, so the file split that inverts body semantics is **43 of 76** and not the 38 the plan carried. Verified on the oracle.
* **The `base/bif` exempt-set assertion is the only in-crate catcher for the `PARSE` comma fence** over the whole workspace.
* **`BEEP` works under the oracle and raises 43.1 here**, and it is outside the builtin table on both sides.
* **A `PARSE` mutation is caught by two `builtin::datetime` unit tests**, which is not a flake and not a coincidence: `dates_absolute_clock_is_cached_the_same_way` and its neighbour both run `parse value date('T') burn() date('T') with n1 . n2`, so the `.` placeholder is load-bearing for them. Reproduced in all three sweeps. It is worth recording because it is the shape a coverage claim gets wrong in the other direction -- a test's name says nothing about which code it reaches.

## What 4c inherits to Phase 5 and Phase 7

* **The trace indent of every 4c construct is unpinned** (criterion 3). Closing it needs either an unnormalised comparison mode or in-crate exact-stderr assertions for `PARSE` and `>I>`/`<I<`.
* **The eight remaining `keyword-exempt.txt` rows are unowned.** Three cannot be made to pass at all, and the exempt file's header says so; the other five are Phase 7's.
* **`BEEP` and the interpreter's internal-package routines** are provided by neither the builtin table nor `::ROUTINE` resolution.
* **`ExprKind::DotVariable` is loud and carries no owner**, which is why nine `base/bif` rows read `UNATTRIBUTED:an environment symbol`. Giving it a phase would make those rows derived like the rest.
* **The `base/bif` drop table's largest categories are Phase 5's**: 418 calls in bodies that send a message and 467 in bodies whose statements this extractor cannot carry.
