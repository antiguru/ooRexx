### Task 12: The 4b gate

**Files:**
- Create: `docs/superpowers/plans/phase-4b-gate.md`
- Create: `rust/scripts/mutate-4b.sh`
- Modify: `docs/superpowers/plans/phase-4-exclusions.txt`

**Why:** D13 and D14.

**Write each criterion so it can fail.** Four criteria in 4a could not, and each was caught late: criterion 6's predecessor was satisfied by `/bin/true`; criterion 4 had no way to fail at all until it was rewritten, then no *subject* until the mode it named was built; `strict_comparison_never_calls_to_number` returned before reading either operand; the CONCATENATION rows would have passed while testing nothing. **And `mutate-4a.sh` itself reported 9 of 9 caught with the oracle absent**, because any non-zero exit counted as a catch. It has since gained `require_baseline_pass` and a `subset_status` distinguishing PASSED, DIVERGED and INFRA_FAILURE. Reuse that guard; the mutations are not reusable.

**Four 4b-specific vacuity shapes to write against:**

* **A trap criterion asserting the program exited 0.** A program that never raised also exits 0. Assert a value the handler set, neither the flag's derived name nor its unset rendering.
* **Criterion 4 carried forward verbatim.** It passes today with zero call frames exercised. Carried unchanged it cannot fail *for the thing 4b adds*. It needs the subset union and an **activation-shaped negative control**: 4a's control deletes `eval_arithmetic`'s `push_temp(left_value)`. A 4b control must delete a root a *call* holds -- the argument list between evaluation and the callee's `USE` -- or it re-tests 4a and reports a pass that means nothing.
* **A coverage criterion that enumerates variants** says nothing about combinations, which is how 4a's two Criticals survived everything. State that limit in the criterion's own text.
* **Criterion 3 must state what DEVIATION 0 removed from its reach.** Leading indentation on stderr is normalised before comparison, so criterion 3 no longer witnesses indent at all except through the pinned shallow-depth witnesses Task 2b keeps. A criterion that reads as though it still checks indentation would be claiming coverage the harness gave up deliberately. Say which witnesses are unnormalised, and say that value-line **content** and line **order** remain byte-exact -- that is where the criterion's power now lives.
* **A queue criterion.** Task 8 has no differential witness at all. Any criterion covering it must assert unit-level order against oracle-measured values, and the gate must record that the construct ships undifferentiated.

**State up front rather than discover:** criterion 2's exempt list cannot light up at this gate or 4c's.

**There are now two exempt lists, and only one of them is inert.** That sentence was written when `tests/assertions.rs`'s 35 rows were the only exempt set, and it remains true of them -- all 35 are `unblocked_by: "Phase 5"`. Task 11 added a second, `rust/corpus/keyword-exempt.txt`, and it is the opposite: **it is designed to fire at 4c's gate.** The contrast to carry:

* `tests/assertions.rs`, 35 rows, all `unblocked_by: "Phase 5"` -- **cannot** light up at this gate or 4c's.
* `rust/corpus/keyword-exempt.txt`, 796 rows -- **790 of them are `4c` and fire at 4c's own gate**, each body starting to pass and failing the set. Both directions live in `rust/crates/rexx-exec/tests/keyword_assertions.rs`: **grep for the fragments `now PASSES` and `is not on the committed`**, not for a line number and not for the whole sentence. The line numbers move whenever that file is edited, which a fix round would do; the full sentences are long single-line literals that rustfmt may re-wrap across a continuation, which defeats a whole-sentence `grep -F`. Short fragments survive both, because rustfmt breaks between words and not within them. The other **6** rows are `defect:compound-do-control-variable` and fire whenever that defect is fixed, which is unscheduled.

The 790 are the point, not the 6. A criterion phrased around the defect rows would leave a reader thinking the file is inert at 4c's gate when 790 of its rows are precisely what fires there. Name the file whenever a criterion refers to "the exempt list": the two behave oppositely, and a criterion naming neither is ambiguous about which property it claims.

**I27 is not this gate's criterion.** The 342 expected trace-output lines in `TRACE.testGroup` are 4b's and 4c's to satisfy, but the group is not runnable as extracted, and the same file yields 239, 342, 374, 393 and 437 under five defensible anchorings -- three recounts have already gone astray. If this gate uses any figure from that file, **state which scan produced it**. Prefer a named, measured subset.

- [ ] **Step 1: Write the gate document before running anything**

Criteria first, results second. A criterion written after the measurement is a description of what happened.

- [ ] **Step 2: Write `mutate-4b.sh` with 4a's guard and 4b-shaped mutations**

Include the activation-shaped negative control, and I17's reclassification with its real mechanism.

- [ ] **Step 3: Run every gate, reading each exit status unpiped**

Full suite; `REXX_CORPUS_GATE=1`; `REXX_ASSERTIONS_GATE=1`; `REXX_KEYWORD_GATE=1`; `cargo clippy -- -D warnings`; `cargo fmt --check`; the mutation script.

- [ ] **Step 3b: Assess whether per-sub-phase ownership attribution earns its keep, and cost the consolidation**

Measured at `e4caa7bf`, mid-4b: `tests/owners.rs`, `tests/loud.rs` and `tests/coverage.rs` are **1,852 lines** against **9,025** lines of interpreter source in `run.rs`, `eval.rs` and `error.rs`. Task 0 was **907 insertions with zero interpreter functionality**, and existed only because variants move in scope one sub-phase at a time.

**Re-measure both totals at gate time, because that ratio has already moved against the argument it was raised to support.** At `5b2de07a`, after Task 11: the same three test files are **1,928** lines (606 + 609 + 713) against **15,838** lines of the same three sources (12,759 + 1,872 + 1,207). So across 4b the harness grew **4%** while the interpreter it attributes grew **76%**, and the attribution surface fell from **20.5%** of interpreter size to **12.2%**. A cost that shrinks as a fraction while the thing it covers doubles is a weaker case for consolidation than the mid-4b snapshot suggests, and quoting only the earlier figure would put a thumb on the scale. Report the trend, both endpoints, and what the third data point at 4c's gate would have to look like to change the recommendation.

This is a real question, not a rhetorical one, and 4c is the last sub-phase that could act on the answer. Report on three things:

* **The third copy.** Ownership data lives in `tests/owners.rs`, in `loud.rs`'s witness rows, and in a match in `src/lib.rs` -- production code cannot reach a test module. Task 0 flagged this and it is still hand-maintained. It is *guarded* (the stderr assertion catches drift, verified by mutation), so it is duplication rather than rot. Cost the fix: make `owners.rs` the single source and assert `src/lib.rs`'s match equal to it, rather than maintaining both.
* **Whether the phase strings are load-bearing at all.** They exist so a gate can say "4b is done" -- but the differential corpus makes that claim better and more directly. A binary implemented/not-implemented, with the reason in the message and no phase attribution, would delete most of this surface. Say what would actually break.
* **Which defects the attribution caused.** By this point the phase's record will show it. The ones visible at Task 2 were: `loud.rs` deleting the witness covering `Call::Qualified` and `Call::Trap` when `Call` lands; four occurrences of a witness implemented out from under a test; the `>I>`/`<I<` owner question, whose *existence* is an attribution artifact; and the corpus-subset union plumbing.

  **Added 2026-08-04, and it is the sharpest instance so far because it was measured rather than predicted.** One stale fact -- "`Call::Trap` is loud", false since Task 7 -- had **three** prose copies, and they were found by three different agents across two tasks: `tests/owners.rs` (Task 8's review), `tests/loud.rs`'s `INSTRUCTION_WITNESSES` doc (Task 8's implementer, while fixing the first), and `src/lib.rs` under `ExprKind::Call` (Task 8's re-review). **Task 7's own five review passes saw none of them**, because each copy sits in a file Task 7 never had to touch. The stderr assertion guards the *data*; nothing guards the *prose about* the data, and the prose is where all three copies were. Cost the consolidation against this: the guarded duplication is cheap, and the unguarded commentary around it is what actually rotted.

**Do not act on the answer in 4b.** This step produces a costed recommendation for the 4c plan. Consolidating the harness while ten tasks depend on it is the collision D5 warns about, and it would move every gate figure between here and now.

- [ ] **Step 3c: Rule on the compound-`DO` gap. Do not inherit it.**

4b closes with **one known divergence from the oracle that is not a missing feature**: a compound variable used as a `DO` control variable is never bound. `bind_control` (`rexx-exec/src/run.rs`, find it by name) writes the control variable through `slot_of`, a flat name-to-slot lookup, so `cv.j` becomes the literal name of one variable and no tail is resolved -- while the same executor resolves the same name correctly one line later. It is **not** a parse gap: `cv.j` is a single symbol token and the parser interns `"CV.J"` whole. Provenance is 4a's; `instruction_owner` returns `None` for `InstructionKind::Do`, meaning implemented, not deferred. **Ownership is unassigned**, which is why it has drifted this far.

Six ooTest bodies turn on it and they are the only assertion failures in the whole `base/keyword` table. `phase-4-exclusions.txt`'s `KNOWN GAPS` row carries the two-line witness and the three probes narrowing it to exactly one mechanism.

Every previous task was right to record rather than fix it, because `DO` was not in their file lists. **This gate has no such excuse**: a sub-phase that ships a known, witnessed, unowned divergence in a construct it claims to have delivered must say so in its own gate document, in one of three forms. Pick one and justify it:

1. **Fix it here.** One function, and the six bodies become the regression test that proves it.
2. **Assign it an owner** -- 4c or Phase 5 -- which moves the row from `KNOWN GAPS` into `EXCLUSIONS` and makes it that phase's gate criterion.
3. **Ship it as an open gap**, named in the gate document as a criterion **met weakly**, with the reason no owner was assigned.

What is not acceptable is a gate document that does not mention it. That is the failure this step exists to prevent, and the reason it is a step rather than a note: an obligation recorded only in a controller's ledger reaches nobody.

- [ ] **Step 3d: Record Task 11's L1 result correctly, and do not gate on it**

The `base/keyword` table is a **measurement this gate reports**, not a threshold it passes. Report: **1,773 of 2,441 exact-spelling `assertSame` calls extracted into 896 bodies (72.6%); 100 bodies pass, carrying 713 assertions.** The remaining 796 bodies are 790 `4c` plus the 6 above.

Three qualifications belong beside that number, all measured during Task 11's review and each one weakening it:

* **A `4c` attribution says what a body hits *first*, not what would make it pass.** Four bodies are known to differ: `CALL::test_expression`, `CALL::test_literal` and `CALL::test_on_name` fail under the **C++ oracle itself** (`Error 43, Routine not found`) because they call `::routine`s the standalone program does not carry, and `NUMERIC::test_42` exits 3 because its body falls through into `dig: Return digits()`.
* So the 790 are an **upper bound on what landing 4c would fix**, not a measure of 4c's remaining surface. Do not restate the stronger claim; it was in `l1-coverage.md` and was corrected.
* `base/keyword` has **zero Phase 5 dependency**. That is a genuine finding and safe to report.

- [ ] **Step 4: Assess each criterion honestly, including the ones met weakly**

4a's gate recorded five met, one met with an inherited criterion defect, and one met weakly with an open gap. That is what an honest gate looks like. A seven-of-seven with no qualifications, after a sub-phase this size, is a claim about the instruments rather than the code.

- [ ] **Step 5: Commit the gate document, read the hash back, record it in the ledger**

---

## Explicitly not in scope, and not promised

* **`tests/assertions.rs` moves not at all.** All 35 exempt rows need Phase 5.
* **`Plan::by_symbol` stays a `HashMap`** (I35). D16's shape wants a `Vec` index, and `SymbolId::index()` landed so the swap is this crate's decision. Variable lookup is 8.1%/32.2% of runtime, so it deserves its own measurement rather than arriving as a side effect. `Option<usize>` is still required because keywords, labels and constants share the `SymbolTable`, so a dense `Vec` has holes.
* **The parser's recursion cliffs** (I32, I33, I34). Prefix-operator chains recurse in `message_subterm` outside the shared depth budget, aborting between 1,150 and 1,200 levels on a default 2 MiB thread, and the oracle's cliff for that construct has never been measured. `Debug`, `PartialEq` and `Clone` on `Expr` are still recursive, cliffs at 2,000/2,050 and 2,100/2,200; the trigger is the first test that formats or compares a deep tree. The depth counter protects a sized caller only -- the native abort arrives at 331 parens or 341 calls. Task 3 Step 1 measures 4b's contribution; it does not fix them.
* **`Heap::alloc`'s friendly name.** Still bypasses the stress hook. Parked in 4a by choice.
* **A `DO`/`LOOP` temps frame growing for its whole run.** Parked in 4a by choice.
* **Everything 4c owns** (I23, I24, I29, I30): `PARSE` in all forms, the 66 in-scope builtins, `ADDRESS` and `ADDRESS()`, `VALUE`'s variable-access form, `QUEUED()`, `ARG()`, `CONDITION()`, and the `rexxcps.rex` gate question -- D8, it auto-adjusts its loop count from measured wall-clock time, so it cannot be the byte-for-byte differential the parent plan assumes.
* **The five 4c-only open decisions**: D4 (11 of 15 excluded builtins genuinely blocked; `QUALIFY` not blocked at all; `USERID`/`SETLOCAL`/`ENDLOCAL` blocked because `std::env::set_var` is `unsafe` in edition 2024 and the workspace forbids it), D7, D8, D11 (measured, unseeded `RANDOM` is deterministic across separate processes, so "the values differ between runs" is not evidence of randomness), D12 for `base/bif`.
