### Task 15: the `base/bif` L1 harness, the 4c subset, `mutate-4c.sh`, and the gate

**Files:**
* Create: `crates/rexx-extract/src/bif.rs`, `crates/rexx-exec/tests/bif_assertions.rs`, `rust/corpus/bif-exempt.txt`, `rust/scripts/mutate-4c.sh`, `docs/superpowers/plans/phase-4c-gate.md`
* Modify: `rust/corpus/phase-4c.txt`, `rust/corpus/README.md`, `crates/rexx-exec/tests/coverage.rs`, `crates/rexx-exec/tests/corpus.rs`, `crates/rexx-exec/tests/collect_stress.rs`

- [ ] **Step 0: What the extractor must model, measured 2026-08-05 -- read this before writing any code**

**D12's reuse decision holds, but not as a drop-in.** Without the three additions below, only **39.9% of calls extract correctly and the rest are silently wrong rather than dropped**, which the conservation invariant cannot see.

**The denominator is not what a naive scan gives.** `^[[:space:]]*::method` case-insensitively gives 5,462; **three of those sit inside `/* … */` block comments** (`CHARS.testGroup`, `LINES.testGroup` ×2), so live method directives are **5,459**.
All 6,293 `assertSame` calls reconcile by location: roughly **6,150-6,170 in live method bodies, 120-135 inside block comments, 5 in `::routine` bodies, 1 behind a `--`**.
The block-comment band is a range because ooRexx block comments **nest** and a same-line `/* … */` is easy to mis-detect; the total reconciles exactly either way.
**A line-oriented scan extracts over a hundred assertions that never run.**

**The composition, by body (a body counts in every category it touches):**

| category | bodies | calls |
|---|---|---|
| total live | 4,237 | 6,169 |
| **self-contained, no dependency** | **2,169 (51.2%)** | **2,464 (39.9%)** |
| local variable, simple literal RHS | 402 | 1,474 |
| local variable, computed RHS | 372 | 922 |
| `.local~` fixture set in another body | 962 | 969 |
| `self~` attribute | 24 | 126 |
| in-file `::routine` | 4 | 7 |
| `NUMERIC` in body | 363 | 565 |
| loop or conditional around the assertion | 113 | 531 |
| `expectSyntax` present | 200 | 210 |

**Add exactly three capabilities, all lookup or carry-forward -- no expression evaluator:**

1. **File-scoped fixture resolution.** Collect `.local~NAME =` per file and resolve `.NAME`, **stem-aware**: `WORD.testGroup:276` does `v8. = .v8` then `word((v8.10),4)`, and exact-name matching misses it. That case defeated the classifier that produced these numbers on its first pass and overstated self-contained by 25 bodies. Buys 962 bodies.
2. **`NUMERIC` carry-forward within the body.** 363 bodies, and **362 of them set `NUMERIC` *before* the first `assertSame`** -- the dangerous ordering. `base/expressions` already needed this and it is the category with a precedent for being got wrong silently, because such bodies' operands are literals and they therefore *look* self-contained. `ABBREV.testGroup:526` is the shape: both operands literal, `Numeric Digits 1` the only thing that changes the answer.
3. **`expectSyntax` routing**, per Step 2.

**Then drop the rest explicitly and count it: 418 bodies / 1,103 calls (17.9%)** -- computed local RHS, loops, `self~` attributes, in-file `::routine` calls. Bounded, nameable, covered by `rows + dropped == calls`. Drop the block-comment calls too, and rule deliberately on the 5 in `::routine` bodies.

**Two categories nobody thought to ask about, and the first is the larger risk:**

* **`::options novalue` is a file-level directive that inverts body semantics.** **38 of the 76 files** carry it. Under it an unassigned symbol **raises**; in the other 38 files it evaluates to its own uppercased name. **Same body text, opposite meaning, and the deciding directive is outside the body.** Roughly 1,325 bodies / 2,027 calls live under it. Verified: none of the seven word groups carry it, which is exactly why they can write `word(nv,1)` = `'NV'`.
* **The NOVALUE idiom itself** -- assertions that read a never-assigned symbol and rely on symbol-equals-own-name, e.g. `BITAND.testGroup:185`'s `assertSame(bitand('3', nv2.3), '02'x||'V2.3')`. Resolvable from the body's own bytes **only if the extractor knows the rule**; to a naive reader it looks like unresolved indirection. **Bounded at ≥178 firm, ≤307 loose** -- the residue contains false positives from instruction forms (a `PARSE` target read later), and the gap was not closed.

**Confidence, stated so it can be checked rather than trusted.** Firm: the reconciliation, the 5,459/5,462 split, the 362-of-363 `NUMERIC` ordering, the 38-of-76 `::options` split, the `expectSyntax` 200/210. Softer: the resolvable/not split turns on a "simple versus computed RHS" rule, and that rule is conservative, so the mechanically-resolvable set is probably slightly larger than stated.

- [ ] **Step 1: Extract `base/bif` by reuse**

D12: no third extractor. Preserve the conservation invariant `rows + dropped == calls` and the `DropReason` detail field.
**Match the assertion token case-insensitively, never by prefix**, and **scan `::method` case-insensitively** -- `DATE`/`TIME` spell 50 of them `::METHOD`.
**Use `/bin/grep -a`** for any count over this directory; six groups contain non-UTF-8 bytes.

Measured population: **6,293 `assertSame`** across 73 of 76 files, 5 `assertSameList`, 1,230 `expectSyntax`, and a 424-call `assertTrue`/`assertEquals`/`assertFalse` tail the extractor drops.
State the drop in the header and let the conservation invariant carry the number.

- [ ] **Step 2: `expectSyntax` couples, more sharply than in `base/expressions` -- measured 2026-08-05, do not re-derive**

**The mechanism, read from `ootest/framework/OOREXXUNIT.CLS` rather than inferred.**
`expectSyntax` (`:1322`) **wraps nothing**; it is a pure state-setter on the TestCase instance.
The checking is a frame up in the framework's own `doTheTest` (`:1563`), which installs `signal on any name exceptionHandler` **in the framework method, not in the test method**, then runs the body as `.message~new(self, methodName)~send`.
A matching condition returns immediately (`:1599`); `check4ConditionFailure` (`:1589`) is reached only if the body ran to completion.

**So a raise inside a test body abandons the rest of that body.**
The expectation is method-scoped, not file-scoped -- `clearCondition` is called only from `Assert~init` and `TestSuite` builds one instance per test method -- so the coupling is strictly intra-body.

**The consequence for the extractor.** The dominant shape puts the raiser **inside `assertSame`'s own argument list**, so arguments evaluate, the raise fires, and `assertSame` is never entered:

```rexx
::method "test_21"                       -- C2D.testGroup:129
   self~expectSyntax(40.5)
   self~assertSame(C2D(,-1), '-1')
```

Verified on the oracle: `c2d(,-1)` is **40.5 at rc 216**. **`'-1'` is not an expected value** -- nothing ever compares anything to it. The body asserts one thing: that the call raises 40.5.

An extractor that ignores `expectSyntax` emits *"`C2D(,-1)` equals `'-1'`"* -- **confidently backwards**, and `rows + dropped == calls` balances, so the conservation invariant certifies it.
That is worse than a dropped row: it points an implementer at a return value where the required behaviour is a raise.

**The rule: a `::method` body containing `expectSyntax` yields no `assertSame` rows from any call at or after that line.**
Route those bodies to an error-expectation path keyed on the syntax code, and count the suppressed calls as **dropped** so the invariant still closes over them.

**The bound: 200 bodies contain both; 199 have the `assertSame` at or after the `expectSyntax`, carrying 205 calls.**
Of the 199, **189** have the raiser inside `assertSame`'s argument list and **10** are a separate `ret = <call>` clause.
Exactly one body asserts before expecting (`LINEOUT.testGroup:207`), which is the correct ordering and the only instance of it.
No body has more than one `expectSyntax`.

**Segment bodies at the next `::` directive of ANY kind, not at the next `::method`.**
Getting this wrong is how the count above was first got wrong, by me: `CONDITION.testGroup`'s `test_novalue_override` is three lines, and a `^::method` scan swallows the three following `::routine` bodies into it, inventing a 201st coupled body and five phantom reachable calls.
`assertSame` inside a `::routine` is a **trap handler's** assertion and genuinely runs.

**The unreachability is conditional; the hazard is not.**
If an expected condition failed to raise, the body would run on and the `assertSame` would execute -- but `check4ConditionFailure` then fails the test anyway.
So in neither case is that `(expected, actual)` pair a statement about correct behaviour.

- [ ] **Step 3: Build the harness, with the set assertion ungated**

`REXX_BIF_GATE=1` selects STRICT reporting, as `REXX_KEYWORD_GATE` does.
**But the both-direction set assertion must run under a plain `cargo test`, not behind the env var.**
That is where `keyword_assertions.rs`'s teeth are, and putting `bif-exempt.txt`'s equivalent behind the flag leaves it policed by nothing anyone runs.

The exempt set's attribution is **derived from the loud message**, not hand-written.

- [ ] **Step 4: Wire `phase-4c.txt` into all three harnesses**

All three of `tests/corpus.rs`, `tests/coverage.rs` and `tests/collect_stress.rs` originally hardcoded `phase-4a.txt` and `phase-4b.txt` at their `read_subset` call sites.
**All three must read the three-file union**, or every 4c witness is inert and D6 is undischarged.
**Re-derive each call site rather than trusting a line number** -- the citations this step once carried have all moved.
Find them by `read_subset(&[` in each file, verified 2026-08-07: `corpus.rs` reads three files, `coverage.rs` reads three at its union site (and one file each at the three per-phase pins, which is deliberate), `collect_stress.rs` reads two.

Add `phase_4c_subset_matches_the_committed_list` to `coverage.rs` -- the pin 4b's gate found missing for `phase-4b.txt`, where **nine of twelve entries were deletable with everything green**, including one criterion's only witness.

**Two thirds of this step are already done, by the tasks that needed them.**
Task 7 added the `coverage.rs` pin when it created `phase-4c.txt`.
Task 9 added `phase-4c.txt` to `tests/corpus.rs` after finding that Tasks 7 and 8's witnesses were being *parsed* by `coverage.rs` and *run* by nothing, while `corpus.rs`'s own module doc said otherwise -- one of its mutations was invisible to the whole suite until the wiring landed.
Deferring the wiring to this task was the error: a witness that does not run is not a witness, and four of them sat inert across three tasks.
**What is left here is `tests/collect_stress.rs`**, plus confirming the other two rather than redoing them.
The corpus gate moved **42 -> 47** at Task 9, and Tasks 10-14 have since taken it to **50 of 50**, measured 2026-08-07 at `1c519dfc` under `REXX_CORPUS_GATE=1` (`phase-4a.txt` 30 + `phase-4b.txt` 12 + `phase-4c.txt` 8).
Any criterion quoting 42 or 47 is stale.
**`corpus.rs`'s dated-row convention binds here:** add a new row naming the commit the number is true at, do not edit the 47 in place.

**Corpus rules for 4c, written into `corpus/README.md` beside the `DO OVER` one:** no `RANDOM`, no `DATE`, no `TIME` (D11); `QUEUED()` single-program only; no dependence on external routine resolution (Task 13); no `DO OVER` on a stem (D3).

- [ ] **Step 5: Write `mutate-4c.sh`**

Carry 4a's and 4b's guard: exact-match, exactly-once application; a baseline before the first mutation and after the last restore; three-way `PASSED`/`DIVERGED`/`INFRA_FAILURE` that never folds an infrastructure failure into either bucket; a non-zero test-run count per target, because `cargo test <name>` exits 0 when it matches nothing.

**`--no-fail-fast` is mandatory, and its absence has already produced false coverage claims in this phase.**
`cargo test --workspace` stops after the first test binary that fails, so under a mutation the run is **truncated at the first catcher** and every later binary silently never executes.
That is invisible in a green baseline -- the truncation only happens when a mutation bites -- so the flag looks unnecessary right up to the moment it matters.
Measured at Task 9: three of six "caught by this test and nothing else" claims were false, and re-running with `--no-fail-fast` found `corpus_differential` catching two of them and `keyword_assertions::the_exempt_set_matches_the_current_failures` catching a third.
**Assert the binary count**, baseline against mutated, so a truncated run is an `INFRA_FAILURE` rather than a survivor or a clean catch.

**Count `Running`/`Doc-tests` header lines, not `test result:` lines, and the two genuinely differ.**
Measured at Task 9's re-review: that green run gave **72** header lines and **73** `test result:` lines.
Neither is wrong -- `Doc-tests rexx_exec` prints **two** `test result:` blocks from one process, a normal doctest run and a `compile_fail` one reported separately.
The header line is the process count and is what a truncation guard must compare; `test result:` double-counts that one process and will read as off-by-one forever.
The headers are on **stderr** and the `test result:` lines on **stdout**, so the guard must capture the two descriptors separately.

**Do not hardcode the number.** Re-measured 2026-08-07 at `1c519dfc`: **73** headers and **74** `test result:` lines, so the constant Task 9 would have written is already wrong, and every task that adds a test binary breaks it again.
**The guard must measure its own baseline in the same run** -- count the headers of the pre-mutation baseline it already performs, then require the mutated run to produce that same count -- so the figure is derived, never maintained.
A count recorded in this plan or in the gate document is a dated observation, not an input to the guard.

**Earlier tasks' uniqueness claims were taken with the truncating command and are not to be trusted as stated.**
Tasks 5, 6 and 7 each recorded a "nothing else catches this" result; Task 8's scoped re-review used `--no-fail-fast` and stands, and Task 9's were re-run.
Tasks 10-14 were dispatched with `--no-fail-fast` mandatory, and Task 14's coverage proof used a stronger instrument still -- `git archive` of the pre-fix commit, built and run against the new test's own program, which *is* the defect rather than an approximation of it.
Prefer that instrument wherever the defect has a commit.
Re-verify the rest here rather than inheriting them -- the failure is one-directional (it over-credits a new test with unique coverage, never under-credits), so no *correctness* conclusion rests on it, but the coverage story does.

**A mutation can hang instead of failing, and an unbounded run reads as a stall rather than a result.**
Measured at Task 14: a mutation to the executor left `keyword_assertions_differential` running for over nine minutes, on `DO::test_DO_standardTest2P`, whose extracted body loops until a condition the mutation had made unreachable.
`cargo test` has no per-test timeout, so the whole guard sits there and no bucket is ever assigned.
**Give each mutated run a wall-clock timeout** (`timeout`(1) around the `cargo test` invocation), and classify a timeout as `INFRA_FAILURE` -- never as `DIVERGED`, which would credit the mutation with a catch the suite did not actually make, and never as `PASSED`.

**Declare each mutation's outcome per instrument in advance**, so an unexpected catch fails as loudly as an unexpected survival.

**A `PASSED`/`PASSED` declaration needs a written justification in the script**, naming the instrument that *should* have caught it and why it cannot.
4b's row 12 was a genuine equivalent mutant; without this rule, declaring a mutation a survivor because nothing happens to test it scores "as declared" and reports coverage that does not exist.

Suggested shapes, each needing an instrument named: a builtin's optional argument ignored; an arity bound off by one; a `PARSE` trigger boundary off by one; a `.` placeholder assigning instead of discarding; the comma fence treated as a trigger; `ADDRESS`'s swap keeping the old name; `::routine` resolved before the builtin table; `TIME('R')` not resetting.
**`TIME('R')` is barred from the corpus by D11**, so its declared catcher must be Task 12's unit test -- name it.

- [ ] **Step 6: Write the gate document, criteria first**

**Write the criteria before running anything.** Carry 4b's ten forward with these amendments:

* **Criterion 2** (`tests/assertions.rs`) will report **the same 4,224 of 4,259 with 35 RUNTIME-BLOCKED** as 4a and 4b did.
  All 35 are `unblocked_by: "Phase 5"` and no row is `4c`.
  **State plainly that deleting the whole of 4c leaves this criterion green**, so it is carried as a regression check and not as evidence of 4c's delivery.
* **Criterion 3**'s target is **16 of 19**: `>.>`, `>I>`, `<I<` are 4c's; `+++` is Phase 7's under D-P.
  Verified 2026-08-07: `trace_oracle.rs`'s `WITNESSED_PREFIX_COUNT` is 16 and `OUT_OF_SCOPE_PREFIX_COUNT` is 3, and the test asserts their sum is 19.
  **State the limit this criterion does not cover: the corpus cannot pin a trace *indent*, for any prefix.**
  Both differential harnesses normalise the run of spaces after the marker, on both sides, and the committed `.expected` files are normalised at comparison time too, so an off-by-two indent is invisible to every corpus-based instrument.
  Only an in-crate exact-stderr assertion sees one.
  A criterion claiming "trace output byte for byte" over the corpus is claiming coverage that does not exist; name the in-crate tests that carry the indent, or say the indent is unpinned.
* **Criterion 10**'s `4c` rows in `keyword-exempt.txt` fire here, and the number this plan was written around is spent.
  **790 was the count when 4c began. Measured 2026-08-07 at `1c519dfc`, the file holds 8 data rows, of which 2 read `4c`** (`CALL::test_on_name`, `CALL::test_9`), 3 read `Phase 7` (all `ADDRESS`), and 3 read `RAISED`.
  The gate is **888 of 896 bodies passing, carrying 1737 of 1773 `assertSame` calls**, under `REXX_KEYWORD_GATE=1`.
  **790 was always an upper bound on what 4c fixes, not a measure of its remaining surface**, and the residue proves it: the three `RAISED` rows are the two `CALL` bodies that fail **under the C++ oracle itself** (`Error 43, Routine not found`, because the extraction dropped the `::routine`s they call) plus `NUMERIC::test_42`, which exits 3 by falling through into its own `dig:` label.
  Those three cannot be made to pass by implementing anything.
  **No task owned these rows**, so they came off ungated across the family tasks.
  **Say which task removed which**, from the git history of `corpus/keyword-exempt.txt`, or the criterion is green here by construction.
* **New: the builtin-status criterion**, both directions, whose falsification is Task 2's Step 5 -- the interpreter mutation, not the file edit.
* **Record that 4b's queue gap is closed, and say by what.** 4b's gate shipped `PUSH`/`QUEUE` with their storage verified only in-crate.
  Task 8 gave the round trip its first differential witness: `corpus/lang/pull_queue.rex` pushes and pulls, and a live `queue-round-trip` row in `crates/rexx-exec/tests/input_oracle.rs` runs it against the oracle today rather than waiting for this task.
  **Cite the closure to the row that runs, not to the corpus program**, which `tests/corpus.rs` does not read until Step 4 above.
* **New: `base/bif`**, reported as a measurement and **not gated on a threshold**, with the set assertion ungated per Step 3.

**Ask of every criterion: what degenerate implementation satisfies this, and would deleting its subject leave it green?**
Four traps specific to 4c:

* **"Each of the 66 names is recognised" is satisfied by 66 stubs returning `''`.** The criterion must assert a value per builtin against the oracle. Task 1's differential harness is what makes this real rather than nominal.
* **A `PARSE` criterion asserting "exited 0" passes for a program that parsed nothing.** Assert the assigned values, chosen so an unset target renders as its own derived name and is recognisably wrong.
* **No builtin is under the collector at all until this task fixes it.**
  Measured at Task 2: the 42-program stress subset calls **no builtin**, so every allocation the 66 add is outside `run_program_collect_every_alloc`'s reach.
  The union must gain at least one program per family that allocates, or criterion 4 passes over a subset that never exercises the code 4c added -- the same defect 4a's version had when it ran 29 programs and zero call frames.
* **Criterion 4's collector control must delete a root a *builtin* holds.**
  Because builtins reuse `resolve_and_run_call`'s argument evaluation, the obvious root in that window is `self.roots.push_temp(argument.value())` -- at `run.rs:3259` when this was written, `run.rs:3574` as of 2026-08-07, so find it by the expression, not the line -- which is **verbatim `mutate-4b.sh` row 9**.
  Re-running it re-tests 4b, which is what the criterion's own second sentence forbids.
  The control must target a root the **builtin's own result** holds between allocation and the caller's use.
* **A "reported, not gated" measurement can still be vacuous** if the set assertion behind it is behind the env var.

- [ ] **Step 7: Run everything; record each figure with its command and unpiped exit status**

---

## Explicitly not in scope

* **`ExprKind::List`** -- Phase 5's (D7).
* **`::method`, `::class`, `::requires`, `::attribute`, `::options`** -- Phase 5's. Only `::ROUTINE` is carved out (D-R), and Task 13 makes the rest fail loudly.
* **`QualifiedCall`** -- Phase 5's; namespaces come from `::REQUIRES`.
* **External routine resolution** (a `.rex` file found on the search path) -- Phase 7's, recorded by Task 13.
* **Command dispatch, `ADDRESS ... WITH` redirection, the platform-supplied default environment, and `+++`** -- Phase 7's (D18, D-P).
* **`TRACE ?`'s interactive pause and its banner lines** -- Phase 7's (D-P).
* **The fifteen excluded builtins** (D4), and `VALUE`'s external-selector form.
* **I32-I35**, unowned and unchanged: prefix-operator recursion outside the depth budget; recursive `Debug`/`PartialEq`/`Clone` on `Expr`; the depth counter protecting a sized caller only; `Plan::by_symbol` as a `HashMap` where D16 wants a `Vec` index.
* **The `DO OVER`-on-a-stem traversal-order deviation** (D3).
