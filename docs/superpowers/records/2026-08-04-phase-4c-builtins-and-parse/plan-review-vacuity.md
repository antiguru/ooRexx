# Phase 4c plan review — checks that cannot fail

Reviewed: `docs/superpowers/plans/2026-08-04-phase-4c-builtins-and-parse.md` (830 lines) against the
tree at `6450e216` (`plan/rust-rewrite`).

Nothing in the repository was modified. Every claim below that names a file, line or count was read or
run; the commands are quoted where the number is load-bearing.

The question asked of every proposed check: **what degenerate implementation satisfies this, and would
deleting its subject leave it green?**

---

## Summary of what I found

Eighteen findings. Three are BLOCKERs and all three land on Task 1's derived builtin-status harness —
the plan's central anti-skew mechanism, which everything else leans on. The harness is well-motivated
and copies the right model, but as specified it **has no differential content, no negative control, and
its subject is deleted by Task 13 of the same plan.**

The single sharpest fact: **`Loud::unresolved_call` is the only producer of the string the classifier
keys on, and Task 13 Step 3 removes it.** From Task 13 onward, deleting the entire `builtin/` module
tree leaves `corpus/builtin-status.txt` validating unchanged.

---

## 1. Task 1's derived builtin-status harness

### F1 (BLOCKER) — Task 13 deletes the classifier's only subject; after it, deleting the whole builtin table leaves the file green

**What the plan specifies.** Task 1 Step 1:

> * stderr ends with `" is not implemented (4c)"` and exit is `NOT_IMPLEMENTED_EXIT` -> `loud`
> * the name is in `coverage.rs`'s `EXCLUDED_BUILTINS` -> `excluded`
> * anything else (including a `40.3` "not enough arguments" raise) -> `implemented`

**Where that string comes from today.** `crates/rexx-exec/src/lib.rs:496`:

```rust
fn unresolved_call(name: &[u8]) -> Loud {
    ...
    Loud { message: owned_message(&format!("routine \"{shown}\""), Some("4c")) }
}
```

reached from the one site `crates/rexx-exec/src/run.rs:3219`:

```rust
let Some(target) = target else {
    return Err(Loud::unresolved_call(name).into());
};
```

That is the **only** producer of `... is not implemented (4c)` on the call path. Task 2 Step 4 confirms
it is the mechanism the status harness rides on: *"`resolve_and_run_call` tries the label table, then
`builtin::dispatch`, then falls through to `Loud::unresolved_call`. **Leave the loud fallback in
place** -- Task 13 replaces it with 43.1."*

**Task 13 Step 3 then removes it:** *"Replace `Loud::unresolved_call` with a real 43.1 raise. Measured
from a clean directory: `Error 43.1 rc 213, "Routine not found"`."*

**Consequence.** From the moment Task 13 lands, a name that resolves to nothing produces
`Error 43.1 ... Routine not found` at rc 213. That is not `NOT_IMPLEMENTED_EXIT`, its stderr does not
end with the loud suffix, and it therefore falls into the classifier's catch-all arm — **`implemented`**.

The degenerate implementation, stated concretely: **delete `crates/rexx-exec/src/builtin/` entirely and
the `builtin::dispatch` call from `resolve_and_run_call`.** Every one of the 66 in-scope names now
raises 43.1; the classifier reads 66 × `implemented`; `corpus/builtin-status.txt` as committed at the
end of Task 12 reads 66 × `implemented`; the set assertion passes in both directions; Step 2's
`implemented + loud == 66` and `total == 81` both pass. **The file's subject is gone and the file is
green.** This is exactly the shape the brief calls "a criterion with no subject" — except worse,
because here the subject is removed by a later task of the same plan.

**It is silent when it happens.** By Task 13 all seven family tasks have landed, so the committed file
already reads 66 × `implemented` and zero `loud`. Nothing flips, nothing goes red, and Task 13 has no
"re-run Task 1's status harness" step at all (Tasks 3–6 and 10–12 each have one; Task 13 does not). The
mechanism dies without a diff.

**And the gate criterion built on it inherits the vacuity.** Task 15 Step 6 adds *"A new criterion for
the derived builtin-status file, policed in both directions, whose falsification is Task 1's Step 3."*
That criterion is evaluated after Task 13, i.e. at the exact point the file can no longer distinguish
an implemented builtin from an absent one.

**Fix, and it is one line of test code.** Add a **negative control** to `builtin_status.rs`: a synthetic
name that is definitely not a builtin (`ZZ_NOT_A_BUILTIN`) run through the identical probe path, with
an assertion that it classifies `loud`. That control goes red the instant Task 13 lands, which is the
notice this whole design is supposed to provide. Then decide deliberately: either the classifier's
`loud` arm is widened to include the 43.1 shape (which reintroduces the problem, since a *genuine*
unresolved routine also raises 43.1 — so it would have to key on the name being in
`rexx_inventory::builtins::NAMES` and *still* not resolving), or the harness is retired at Task 13 with
that stated in the gate rather than left as a green file that measures nothing.

The honest alternative, which is cheaper: make the classifier's `implemented` verdict require the
**absence of a 43.1 raise as well as the absence of the loud message**, and pin that in a test.

---

### F2 (BLOCKER) — the harness passes having run zero programs; Step 3's falsification cannot detect it

**Step 3 as written:**

> Delete one row from `corpus/builtin-status.txt` and confirm the test fails **by name**.
> Then hand-edit one row from `loud` to `implemented` and confirm the other direction fails.

**Both mutations are mutations of the committed file.** Neither touches the classifier and neither
touches the interpreter. Consider this classifier, which runs no program at all:

```rust
fn classify(name: &str) -> Status {
    if EXCLUDED.contains(&name) { Status::Excluded }
    else if DISPATCHED_NAMES.contains(&name) { Status::Implemented }
    else { Status::Loud }
}
```

* At Task 1 time (`DISPATCHED_NAMES` empty) it produces 66 `loud`, 15 `excluded`, 0 `implemented` —
  Step 2's expected result exactly.
* Deleting a row from the file: measured set has 81 keys, committed has 80 → fails by name. **Step 3's
  first falsification passes.**
* Flipping a row from `loud` to `implemented`: measured says `loud`, committed says `implemented` →
  fails the other direction. **Step 3's second falsification passes.**
* `total == 81`, `implemented + loud == 66` → both pass.

So a harness that never invokes `run_program` satisfies every check Task 1 specifies. This is the
"harness that counted `diff` output without checking either side produced any" shape, and the "mutation
script that reported 9 of 9 caught with the oracle absent" shape, in one.

It is not a contrived implementation, either — it is the *fast* one. 81 in-process interpreter runs per
test invocation is real cost, and the table lookup is the obvious optimisation someone reaches for when
the test gets slow.

**The plan's own model already solves this and the plan did not copy that part.**
`keyword_assertions.rs` carries three constructed witnesses that pin the harness to the *interpreter's*
behaviour rather than to the file: `the_falsification_proof` (perturb a passing body's operand and
require exactly that body to fail), `a_body_whose_assertions_never_run_is_not_a_pass`, and
`an_assertion_inside_a_loop_is_checked_on_every_pass`. All three mutate the **subject**, not the
committed set. `builtin_status.rs` gets none.

**Fix.** Step 3 needs a third and fourth falsification, both of which mutate something other than the
file:

* the negative control from F1 (a synthetic non-builtin name must classify `loud`);
* an **adjacent success**: after Task 2, `LENGTH` must classify `implemented` *and* a name in the same
  family that has not landed yet must classify `loud`, asserted in the same test — which is the pairing
  the repo's own "pair a refusal with its adjacent success" rule asks for and which no table-lookup
  classifier survives once the dispatch table and the status file diverge.

Also assert that the probe loop actually executed `NAMES.len()` programs, and that the count is
non-zero — the cheap version of the `mutate-4b.sh` per-target run-count guard, one level down.

---

### F3 (BLOCKER) — the status file records *resolution*, never a *value*; 66 stubs returning `''` satisfy it, and the gate criterion built on it is the trap the plan names three bullets later

Task 15 Step 6 lists, correctly, as the first of three concrete traps:

> **"Each of the 66 names is recognised" is satisfied by a stub returning `''` for all 66.**
> The criterion must assert a **value per builtin**, captured from the oracle, or it is `/bin/true`
> with 66 rows.

And then, five lines earlier in the same step, adds:

> **A new criterion for the derived builtin-status file**, policed in both directions, whose
> falsification is Task 1's Step 3.

**Those are the same criterion.** The status harness's `implemented` verdict means precisely "this name
did not produce the loud message" — which is "recognised". A tree in which all 66 builtins are
`fn(..) -> ObjRef { interp.alloc_empty_string() }` produces 66 × `implemented`, and the criterion
reports MET.

Worse, the builtin-status harness **never invokes the oracle**. `corpus/builtin-status.txt` is a record
of this crate's self-report about itself. `keyword-exempt.txt`, the model it copies, is also
self-derived — but its rows are keyed to bodies whose *pass* condition is an oracle-defined assertion
from `ootest`. There is no analogue here.

**Fix.** State in the gate, in the criterion's own text, that the builtin-status criterion measures
**resolution and the boundary record only, not correctness**, and that correctness is criterion N's
(the per-builtin value criterion) and the corpus's. Otherwise a reader takes "66 of 66 implemented,
policed in both directions" as evidence 66 builtins work, which it is not. The 4b gate did exactly this
for criterion 9 ("this gate records that the construct ships undifferentiated") and it is the right
precedent.

---

### F4 (MINOR) — the classification rule contradicts Task 1's own count assertion

Step 1 says a name in `coverage.rs`'s `EXCLUDED_BUILTINS` classifies `excluded`. Step 2 says the result
is *"66 `loud`, 15 `excluded`, 0 `implemented`"* and asserts `implemented + loud == 66`.

`EXCLUDED_BUILTINS` (`crates/rexx-exec/tests/coverage.rs:701`) has **18** entries, not 15 — the last
three are `VALUE`, `ADDRESS`, `QUEUED`, the *partial* rows, which are **in scope**. `coverage.rs:757`
derives `in_scope = names.len() - (EXCLUDED_BUILTINS.len() - 3)` for exactly this reason.

So the rule as written yields 18 `excluded` and `implemented + loud == 63`. The assertion fails on
first run. The likely repair is a **second, hand-written 15-name list** in `builtin_status.rs`, with
nothing policing it against `coverage.rs`'s 18 — a new drift surface in the file whose whole purpose is
to eliminate drift. Specify the derivation instead: `EXCLUDED_BUILTINS` minus the three partial names,
with the three named as a constant that `coverage.rs` and `builtin_status.rs` share.

### F5 (MINOR) — the `excluded` arm runs nothing and cannot fail

`excluded` is decided by a name-table lookup. 15 (or 18) of 81 rows never consult the interpreter, and
would still read `excluded` if someone accidentally implemented `USERID`. That is not harmful, but it
means the file's row count overstates how much of it is derived: 66 rows are measured, 15 are copied.
If the gate quotes "81 rows, derived", say which 66.

### F6 (IMPORTANT) — the probe form is unguarded, and the wrong one classifies all 81 as `implemented`

Step 1 says the harness "runs a one-line program through `run_program` that calls it with zero
arguments". It does not say what that program is.

`length()` as a whole clause is not a call in Rexx — it is a **command expression**, dispatched to the
environment. `InstructionKind::Command` is Phase 7's; the loud message would end `(Phase 7)`, not
`(4c)`; the classifier's first arm does not match; every name lands in the catch-all and classifies
`implemented`. Committed once, the file validates forever and measures nothing.

The plan's own global constraints already flag the neighbouring hazard (`say '['x']'` is error 15.3;
`b2x`/`x2d` probes) but not this one. Write the probe program into the plan (`say NAME()`, or
`zz = NAME()`), and let the F1/F2 negative control catch a regression in it.

### F7 (IMPORTANT) — what "anything else -> implemented" also swallows

Beyond 43.1 (F1) and a command dispatch (F6), the catch-all arm classifies as `implemented`:

* **a parse failure** — a name whose zero-argument spelling does not parse (exit non-zero, no loud
  suffix);
* **a panic** — `run_program` is in-process, so a panic aborts the test rather than being classified,
  which is survivable; but a panic *caught* anywhere on that path and turned into a non-zero exit reads
  as `implemented`;
* **a hang** — no timeout is specified anywhere; an infinite loop in one builtin stalls the harness
  rather than failing it;
* **a raise of any other error number**, including 40.x sub-codes that indicate the *wrong* builtin ran.

Only the last is intentional. The rule "keys on the loud message, treats every other outcome as
implemented" is a **default-open** classifier, and every default-open classifier in this project's
history has been the vacuous one. A default-closed rule — an explicit list of accepted outcomes
(`exit 0`, or a 40.x raise naming this builtin) with everything else classified `unclassified` and
failing the test — costs one match arm and cannot swallow F1, F6 or a parse failure.

### F8 (IMPORTANT) — Task 2's dispatch name-set assertion contradicts Task 2's own scope, and the resolution decides whether F1 bites

Task 2 Step 3: *"**Assert the dispatch's name set equals `rexx_inventory::builtins::NAMES` minus
`EXCLUDED_BUILTINS`.**"* — while *"Only `LENGTH` is implemented in this task"*.

Two readings, and the plan does not choose:

* **(A) `dispatch` has arms only for implemented names.** Then the assertion is false at Task 2 (1 name
  vs 66) and cannot be written until Task 12. Unimplemented names return `None`, fall through, and rely
  on `Loud::unresolved_call` — so F1 bites exactly as described.
* **(B) `dispatch` has 66 arms from Task 2, the unimplemented ones raising their own loud message.**
  Then the assertion holds from Task 2 and the loud producer survives Task 13. But this contradicts
  *"A family task's diff is one new file plus one line in `builtin/mod.rs`'s dispatch"* (an arm would be
  *changed*, not added) and contradicts the `Option` contract's stated purpose (*"`None` means 'not a
  builtin name', which is what lets resolution fall through to `::routine` in Task 13"* — a builtin
  name must never fall through, since builtins shadow `::routine`).

Reading (B) is architecturally forced by the shadowing requirement and would incidentally repair F1.
**Say so explicitly**, and add the step Task 13 is missing: re-run the status harness after the 43.1
change and record that nothing moved *because* the loud producer for builtin names is the dispatch arm
and not the fallback.

### F9 — what Step 2's count assertions do and do not catch

Credit where due: `total == 81` **does** close the empty-set hole (a harness that enumerated nothing
fails it), and `NAMES.len() == 81` is already asserted in `coverage.rs:728`, so the new assertion is a
duplicate of an existing one — "can fail is not adds coverage".

What the counts do **not** catch, and none of it is caught elsewhere in Task 1: a classifier that runs
nothing (F2); a probe that never reaches the builtin (F6); a name classified `implemented` because it
crashed (F7); a builtin implemented as a stub (F3); and the whole table's deletion after Task 13 (F1).

**The mutation Step 3 would miss, stated as the brief asks:** revert one family's `builtin/<family>.rs`
to stubs returning `''` — the status file is unchanged and green. Or, post-Task-13, delete
`builtin/mod.rs`'s dispatch call from `resolve_and_run_call` — the status file is unchanged and green.

---

## 2. The gate criteria in Task 15 Step 6

The plan pre-emptively names three traps (the `''` stub, PARSE-asserting-exit-0, and the collector
control re-testing 4b). Those are good and correct. Here are the ones it did not name.

### F10 (IMPORTANT) — criterion 4's collector control has no subject distinct from 4b's; the plan's own interface guarantees it

Task 15 Step 6:

> **Criterion 4's collector control must delete a root that a *builtin* holds** — an argument between
> evaluation and the builtin's own use. Re-running 4b's activation-shaped control here re-tests 4b.

But Task 2's Interfaces section says builtins consume *"`resolve_and_run_call`'s existing argument
evaluation, **unchanged**"*. There is exactly one root held in that window, `run.rs:3259`:

```rust
let argument = self.eval_argument(code, expr)?;
self.roots.push_temp(argument.value());
```

and that line **is** `mutate-4b.sh` row 9 — the 4b control, verbatim:

```
run_one "9. the argument list's root is dropped (activation-shaped control)" "${RUN_RS}" PASSED DIVERGED \
'                    let argument = self.eval_argument(code, expr)?;
                    self.roots.push_temp(argument.value());' \
'                    let argument = self.eval_argument(code, expr)?;'
```

So the criterion as worded instructs the implementer to delete the same line, and it will be satisfied
by re-running 4b's control under a new name — the precise failure the criterion's own second sentence
forbids. The criterion has no subject unless a different one is named.

**The 4c-shaped window does exist**, and the plan's global constraints point straight at it: *"Every new
allocation site goes through `Interp::alloc_with` ... **4c adds more allocation sites than 4a and 4b
combined** — every builtin returning a string allocates."* The control should delete a root a builtin
holds **inside its own body** — an intermediate that survives across a second allocation (e.g. the
accumulator in `TRANSLATE`, `SPACE` or `CHANGESTR`, held while the result buffer is allocated). That is
a window 4b could not have, and it is the one `collect_stress` is the only instrument for.

### F11 (IMPORTANT) — criterion 2 (`tests/assertions.rs`) cannot fail for anything 4c builds, and the plan predicts the number, which removes its last content

Task 15 Step 6: *"**Criterion 2** (`tests/assertions.rs`) will report **the same 4,224 of 4,259 with 35
RUNTIME-BLOCKED** that 4a and 4b reported. All 35 are `unblocked_by: "Phase 5"`. **That sameness is the
correct result**."*

Apply the review question. 4c's subject is 66 builtins, `PARSE`, `ADDRESS` and `::routine`. **Delete all
of it** and criterion 2 still reports 4,224 of 4,259 with 35 blocked — indeed it reports it more
reliably, since there is no 4c code to regress. I verified the premise: all 35 `EXEMPT` rows carry
`unblocked_by: "Phase 5"` (`crates/rexx-exec/tests/assertions.rs:347+`), and no row is attributed `4c`,
so no `base/expressions` row is blocked on a builtin at all.

This is structurally identical to the 4b defect the 4b gate *amended*: *"Carried forward verbatim, this
criterion could not fail for anything 4b built, and that is why it is not carried forward verbatim"*
(criterion 4). Criterion 4 was given a 4b-shaped subject; criterion 2 is carried into 4c with only a
prediction attached.

**A criterion that cannot move is not automatically a criterion that cannot fail** — this one can fail
on a *regression* in code 4c does not touch, and that is worth something. But its only 4c-specific
content is the prediction, and a prediction that is confirmed by doing nothing is a tautology. Two
honest options, either acceptable:

* label it explicitly in the 4c gate as **a regression check carried forward, not a 4c measurement**,
  so a reader does not count it toward 4c's evidence; or
* give it a 4c-shaped subject by running the same instrument over the new `base/bif` rows, which is
  what criterion N (base/bif) is for — in which case say that criterion 2 is subsumed and stop quoting
  its number as a 4c figure.

### F12 (IMPORTANT) — the `base/bif` criterion is "reported, not gated", and the both-direction policing only bites if the set test is *ungated*

Task 15 Step 3 says *"`REXX_BIF_GATE=1`, the same shape as `REXX_KEYWORD_GATE`. A body that starts
passing must be as red as one that starts failing."* Step 6 says the criterion is *"reported as a
measurement and **not gated on a threshold**"*.

**Where `keyword_assertions.rs`'s teeth actually are.** Two tests, and only one is gated:

* `keyword_assertions_differential` — gated: `assert!(!gate || unaccounted.is_empty(), ...)`.
* `the_exempt_set_matches_the_current_failures` — **`#[test]` with no gate check at all**
  (`keyword_assertions.rs:411`). Its doc says so explicitly: *"Polices the committed exempt set itself,
  **in every mode, independent of `GATE_ENV`**."*

The both-direction policing is the *ungated* one. If `bif_assertions.rs` copies "the same shape as
`REXX_KEYWORD_GATE`" and puts the set assertion behind `REXX_BIF_GATE`, then a criterion that is
"reported, not gated" is policed by **nothing that anyone runs**: `cargo test --workspace` does not set
the env var, and the gate document says the criterion is not gated. `bif-exempt.txt` becomes a
committed file no check reads, and the criterion is a printed number — the shape the 4b gate's own
criterion 3 doc calls out: *"A printed number that no assertion reads cannot fail."*

**Fix, and it is a one-sentence addition to Task 15 Step 3:** the set-equality test runs unconditionally,
in every mode, exactly as `the_exempt_set_matches_the_current_failures` does; only the report and the
non-zero exit are behind `REXX_BIF_GATE`.

### F13 (IMPORTANT) — `bif-exempt.txt`'s "attribution derived from the loud message" collapses to a constant for the population that most needs attributing

Task 15 Step 3: *"The exempt set's attribution is **derived from the loud message**, not hand-written."*

That derivation is `parse_loud`, which extracts the owner from `... is not implemented (OWNER)`. After
Task 13 there is no loud message for an unresolved name — there is a 43.1 raise, which
`keyword_assertions.rs`'s classifier maps to `RunOutcome::Raised` → attribution `"RAISED"`, a constant
with no owner.

**Who lands in that bucket.** Measured, `ootest/ooRexx/base/bif` has **76** `.testGroup` files, of which
nine are the test groups of *excluded* builtins — `CHARIN`, `CHAROUT`, `CHARS`, `LINEIN`, `LINEOUT`,
`LINES`, `QUALIFY`, `RXQUEUE`, `STREAM` — plus `BEEP.testGroup` and `FILESPEC.testGroup`, whose subjects
are **not in `rexx_inventory::builtins::NAMES` at all** (I checked the generated 81-name list).
Eleven of 76 files, ~14% of the group, are permanently unpassable in Phase 4 and, post-Task-13, all
attribute to the single string `RAISED`.

`keyword_assertions.rs`'s own module doc already names this weakness for its `defect:` rows: *"the set
test compares a constant against a file holding the same constant. What still has teeth there is
*membership*, not the label."* That is true and it is a real (if weak) property — but the 4c plan
claims *derived* attribution for `bif-exempt.txt` without noting that the largest single bucket will not
be derived from anything. State the limit in the harness's own doc, the way `keyword_assertions.rs`
does, and consider deriving an owner for the excluded names from `EXCLUDED_BUILTINS` (which does carry
Phase 7 / Phase 10 attribution in `phase-4-exclusions.txt`) rather than letting them fall into `RAISED`.

### F14 (IMPORTANT) — criterion 3's `>I>`/`<I<` flip breaks the chain that makes `Witnessed` mean anything

Task 13 Step 6: *"Flip `>I>`/`<I<` to `Witnessed` and verify."* Task 13 Step 5: *"The absolute path
makes any committed expectation host-dependent, so **the witness lives in the live corpus, not in
`tests/trace_oracle/`**."*

Those two cannot both hold as the file is built. `the_trace_surfaces_coverage_is_..._with_owners_for_the_rest`
(`trace_oracle.rs:582`) asserts the `Witnessed` subset equals `CLAIMED_PREFIXES`, and
`every_witness_still_emits_every_prefix_it_is_named_for` (`:421`) asserts `CLAIMED_PREFIXES` equals the
union of prefixes actually **present as bytes in the committed `tests/trace_oracle/*.expected` files**.
A `Witnessed` row with no `.expected` behind it fails the second test. That chain is the whole reason
the 13-of-19 number is not self-certifying, and its doc says so: *"That chain is what stops 'witnessed'
from being a claim this file makes about itself."*

The predictable repair — adding a third `Coverage` variant (`WitnessedInCorpus`) with nothing behind it —
makes 2 of criterion 3's 16 unbacked, and is precisely the H3 attack the chain was built for
(*"`keyword_while.rex` replaced with a straight-line program, `.expected` regenerated ... both still
'correct' in the sense that they agree with each other"*).

**Decide it in the plan, not at implementation time.** Either normalise the absolute path in
`tests/support/mod.rs` (a DEVIATION row, priced like DEVIATION 0) so `>I>`/`<I<` can have a real
`.expected`, or leave them `Owned("4c")`-shaped with a new variant that is *chained to the corpus
subset file* by an assertion that the named corpus program exists and contains the prefix. Do not flip
them to `Witnessed` with the witness outside the file set `Witnessed` is defined against.

### F15 (IMPORTANT) — the `::routine` corpus witnesses collide with `coverage.rs`'s no-directives guard, and the collision degrades into a missing witness

`coverage.rs:150` panics on any program in the subset union carrying a `::` directive:

```rust
assert!(p.directives.is_empty(), "{} has a `::` directive, which this walker does not follow into ...")
```

and `every_in_scope_variant_is_witnessed_by_the_phase_subsets` runs it over every program in the union.
Task 15 Step 4 puts `phase-4c.txt` in that union (*"The union of all three subset files is what every
harness reads"*).

Task 13 needs at least two corpus programs containing `::routine`: the `>I>`/`<I<` witness (Step 5) and
the shadowing witness (`call max 1, 9` with `::routine max` present). Both will trip the guard.

The plan does not mention widening the walker. The cheapest-looking resolution — **keep the `::routine`
programs out of `phase-4c.txt`** — is the damaging one: a program outside the subset file is read by
`corpus.rs`, `collect_stress.rs` and `coverage.rs` **not at all**, so the `>I>`/`<I<` witness stops
being compared to the oracle and the "`::routine` resolved before the builtin table" mutation loses its
declared catcher. Name the widening as a Task 13 (or Task 15) step and price it.

### F16 (IMPORTANT) — the mutation script's declaration mechanism lets any un-instrumented mutation be scored a success

`mutate-4b.sh`'s rule — *"a mutation is a FAILURE of this script when its observed pair of statuses
differs from its declared pair"* — is correct, and 4b earned its one `PASSED/PASSED` row (I17) with a
documented equivalence argument plus a stated trigger for revisiting it.

Task 15 Step 5 carries the mechanism (*"Declare each mutation's expected outcome per instrument in
advance, so an unexpected catch fails as loudly as an unexpected survival"*) **without the rule that
constrains it**: nothing in the plan forbids declaring a row `PASSED/PASSED`. So any mutation for which
no instrument exists can be made to "behave exactly as declared" by writing down that it survives, and
the script exits 0 reporting "N of N as declared".

Two of the eight suggested mutations are already in that position by the plan's own rules:

* **`TIME('R')` not resetting.** D11 forbids `TIME` in any corpus program, so `corpus=PASSED` is
  forced by rule. The only possible instrument is the unit test Task 12 describes but does not specify
  (*"its unit test must pin the reset semantics, not a value. Two probes inside one second cannot
  distinguish a live clock read from a cached one; construct the probe so they can"* — an instruction,
  not a check). If that test ends up asserting only that `TIME('R')` returns a number, the mutation is
  `PASSED/PASSED` and scores as success.
* **`RANDOM`-shaped rows** are in the same position for the same reason (D11), as is anything touching
  `QUEUED()` beyond a single program.

**Fix.** Add to Task 15 Step 5: a `PASSED/PASSED` declaration requires a written equivalence or
no-observation-point argument **in the row's own comment**, plus a stated event that would falsify it —
the shape `mutate-4b.sh` row 12 already models. And require that every row with a `DIVERGED` on some
instrument names the **failing target binary** (`suite_failing_targets` already exists and 4b's gate
records why: *"a bare 'suite DIVERGED' does not say which instrument spoke"*).

### F17 (MINOR) — the mutation script's suite target list, and the two new criteria with no mutation evidence

`mutate-4b.sh` hardcodes `SUITE_TARGET_COUNT=3` and
`cargo test ... --lib --test trace_oracle --test collect_stress`. If 4c carries that list unchanged,
**`tests/builtin_status.rs` and `tests/bif_assertions.rs` never run under any mutation** — the two
criteria Task 15 Step 6 adds have zero mutation evidence, which is the one form of evidence this
project trusts. `require_clean` likewise guards only `run.rs` and `queue.rs`; 4c mutates
`builtin/*.rs`, `parse_template.rs` and `plan.rs`. Name the target list and the guarded paths in the
plan so they are not left to be inferred.

Judgement on the eight suggested mutation shapes, per the brief's question ("is there a plausible tree
in which this mutation's pattern matches nothing, or matches something other than intended?"):

| suggested mutation | pattern risk | observable by |
|---|---|---|
| a builtin's optional argument ignored | low (family file, one site) | corpus, if a 4c corpus program passes that optional argument — **needs to be written into `phase-4c.txt`, not assumed** |
| an arity bound off by one | **moderate** — an arity *table* of 66 `(min, max)` rows makes `(1, Some(1))` occur many times; the exactly-once guard will refuse to apply it unless the pattern includes the name | unit test asserting the 40.3 bytes |
| a `PARSE` trigger boundary off by one | low | corpus + unit |
| `.` placeholder assigns instead of discarding | low | corpus (values) **and** trace `>.>`/`>=>` |
| comma fence treated as a trigger | **moderate** — likely a `None =>` match arm, which occurs in many `match`es over `Option`; needs surrounding context in the pattern | corpus |
| `ADDRESS`'s swap keeps the old name | low | corpus, **only if** a program does a two-deep swap and prints `ADDRESS()` — needs both Task 9 and Task 10 |
| `::routine` resolved before the builtin table | **high** — a reordering, not a substitution; and see F15, its corpus witness may not exist | corpus, if F15 is resolved |
| `TIME('R')` not resetting | low | **nothing specified** — see F16 |

The arity-table and comma-fence rows are the two where the exactly-once guard will most plausibly fire
as `UNAPPLIED PATTERN` (fatal, by design) rather than as a silent skip. That is the guard working, and
the guard does exist to be carried — `apply_mutation` at `mutate-4b.sh:193` requires
`content.count(old) == 1` and exits 1 otherwise. Note it in the plan so the implementer writes
name-qualified patterns from the start.

---

## 3. The fifteen tasks' "verify" steps

### F18 (IMPORTANT) — no task's verify step runs the differential, so a family task can land a builtin that diverges from the oracle and stay green

`corpus.rs:571` is the only place divergence is fatal:

```rust
assert!(!gate || mismatches.is_empty(), "STRICT ({GATE_ENV}) mode: ...")
```

`gate` is `REXX_CORPUS_GATE`. **Plain `cargo test --offline --workspace` reports divergences and exits
0.** Every task's verify step in this plan is `cargo test --offline --workspace`, `cargo fmt --all
--check`, and clippy (Task 1 Step 7 is the only one that spells it out; Task 2 Step 6, Tasks 3–6/10–12
Step 5, Task 8 Step 4, Task 9 Step 3 and Task 13 Step 6 are bare headings reading "Verify and commit"
with no body at all).

So the project's own definition of correctness — byte-for-byte against `build/bin/rexx` — is **not
checked at task granularity anywhere in Phase 4c**. It is first checked at Task 15 Step 7, after all 66
builtins, `PARSE`, `ADDRESS` and `::routine` have landed. Add `REXX_CORPUS_GATE=1` to the verify step
of every task from 7 onward (the first task that adds a corpus witness) and to the family tasks once
`phase-4c.txt` has any builtin program in it.

Two things that *do* fire automatically and are worth stating in the plan so they are not mistaken for
breakage: `the_exempt_set_matches_the_current_failures` and `assert_witness_set_is_complete` /
`in_scope_counts_match_the_audited_split` (`loud.rs:366`, `:427`, which pin `12`, `4`, `31`, `11` as
literals) will go red as constructs move in scope. See F19.

### F19 (IMPORTANT) — the seven family tasks' Step 4 cannot fail: it commits whatever changed

> **Step 4: Re-run Task 1's status harness and commit the flipped rows**

There is no expectation to compare against. A task that implements 20 of its 23 names flips 20 rows,
commits them, and is green. The harness's own failure message (*"re-run and commit
`corpus/builtin-status.txt`"*) instructs precisely this. A file whose only obligation is to be edited
into agreement with reality can never contradict reality; its value is entirely in a human reading the
diff, and nothing requires that reading.

The plan already knows the right shape and applies it to mutations: **declare the expected outcome in
advance.** Each family task knows its exact name list. Step 4 should read: *"state the rows you expect
to flip before running; the set that actually flips must equal it exactly, and a mismatch in either
direction is a task failure."* That is free — the list is in the task's own brief — and it converts the
step from a ratchet into a check.

### F20 (MINOR) — nobody owns `keyword-exempt.txt`'s 790 rows, and criterion 10 is therefore green at the gate by construction

`the_exempt_set_matches_the_current_failures` is **ungated** and runs on every `cargo test`. Its 790
`4c` rows start passing as builtins land — the 4b gate says so explicitly (*"designed to fire at 4c's
gate"*). They will actually fire at Tasks 3, 4, 5, 6, 10, 11, 12 and 13, seven or eight times, each
time turning a plain `cargo test --workspace` red.

No task in this plan has a step for it. Task 14 Step 3 removes the **six** `defect:` rows and nothing
else. So an implementer meets a red test with no instruction and reconciles the file ad hoc, seven
times. By Task 15 the file has been hand-edited into agreement and criterion 10 reports MET having been
made true by the tasks it is meant to measure.

The information is not lost — it is consumed early, which is arguably fine — but the gate should say
so, and one task should own the reconciliation with a **declared expected row count per family** (same
device as F19). Otherwise "790 rows fired" is a number nobody can attribute.

### F21 (MINOR) — Task 14's "automatic success signal" does not assert that the six bodies pass

> **That red test is this task's success signal**, and it is automatic.

The signal is that `the_exempt_set_matches_the_current_failures` goes red. But it goes red for **three**
different reasons, and only one of them is the fix working: a body that starts passing, a body whose
attribution *changes* (`defect:compound-do-control-variable` → `4c` or `RAISED`), and a body that starts
failing. All three are cured by editing the file, and Step 3 ("Remove the six rows") cures the first
two identically. Assert the six by name — `RunOutcome::Pass` for each — before removing their rows.

### F22 (MINOR) — the `base/bif` denominators in D12 count `ls` entries, not `.testGroup` files

D12 and Task 15 Step 1 both state *"78 files"* and *"31,162 lines"*. Measured, from
`ootest/ooRexx/base/bif`:

```
$ ls *.testGroup | wc -l                    → 76
$ find . -name '*.testGroup' | wc -l        → 76
$ ls | wc -l                                → 78     # + ARG_TEST.rex + lineout/
$ cat *.testGroup | wc -l                   → 31115
$ grep -oiE '\bassertSame\b' *.testGroup | wc -l   → 5441   ✓ matches the plan
$ grep -oE 'assertSameList' *.testGroup | wc -l    → 5      ✓ matches the plan
$ grep -oiE '\bexpectSyntax\b' *.testGroup | wc -l → 1021   ✓ matches the plan
```

`find_test_groups` (`rexx-extract/src/lib.rs`) globs `*.testGroup` recursively, so the harness will
report 76. The `assertSame` / `assertSameList` / `expectSyntax` figures the plan's decisions actually
turn on are all correct; only the file and line denominators are off, and they are off because they
count `ls` output. Correct them in the plan so the first person to run the extractor does not read a
76-vs-78 mismatch as an extraction defect.

---

## 4. The seven family tasks' shared Step 1 and Step 3

**Can probe-derived tests pass while the implementation is wrong?** Yes, in three ways, and the plan
closes none of them:

1. **The probe and the implementation share an author and a mistake.** Step 1 says *"Commit the table as
   the test's expected values, with the probe program beside each row."* The probe program is committed
   as text; **nothing re-runs it against the oracle.** If a probe is run from the scratchpad root rather
   than a fresh subdirectory (the plan's own global constraint warns this changes 44.1/rc 212 into
   43.1/rc 213), or under a `NUMERIC DIGITS` the author forgot they set, the wrong value is transcribed
   and the implementation is then written to match it. Test green, oracle disagrees. The corpus is the
   only cross-check and it is not run until Task 15 (F18). **Cheapest fix: one corpus program per family
   in `phase-4c.txt`, landed by the family task, so the differential re-derives the family's behaviour
   from the oracle on every run.**

2. **The status harness cannot see it** (F3) — an `implemented` row says nothing about a value.

3. **What the probe table cannot see, beyond the plan's own list** (it names optional-argument positions
   and pad characters):

   * **A value's captured rendering (D15).** `say f(x)` cannot distinguish a result that captured its
     digits/form pair at creation from one that renders at `SAY` time — the two agree unless
     `NUMERIC DIGITS` or `FORM` changes *between* the builtin's return and the use. The plan makes D15 a
     global constraint and names `FORMAT`, `TRUNC`, `D2X`, `X2D` as digits-shaped, but no probe shape in
     Step 1 can observe it. Add one: `x = f(...)` then `numeric digits N` then `say x`.
   * **Number-versus-string operand identity.** `length(1.0)` and `length('1.0')` are different programs
     in Rexx; so are `f(v)` where `v = 1/3` under two different `DIGITS`. Step 1 probes literals.
   * **Trace output.** A builtin call emits value lines under `trace i`; the probe table is stdout-only.
     `>F>` is already `Witnessed`, so nothing forces a re-check, and a builtin whose result traces
     wrongly ships silently.
   * **stderr and exit status on the success path.** The global constraint says read three descriptors;
     Step 1's bullet list mentions neither for the base case.
   * **Which sub-code wins when two argument rules are violated at once** — the plan probes single
     violations (Task 2 Step 1) and the family tasks inherit that.
   * **The trailing-comma form.** Step 1 covers `f(,2)`; `f(2,)` is a different arity question.

Step 2 (*"Read the ooTest group for each name"*) is the right instinct and is the strongest thing in
these seven tasks — it is an enumeration **outside this repository**, which is the axis the repo's own
CLAUDE.md identifies as the one that does not rot. It is not, however, a check: nothing asserts that a
case in the group was covered. Task 15's `bif-exempt.txt` is where that becomes visible, which is the
plan's stated intent (*"a case it covers that the implementation misses will surface at Task 15 as an
exempt row that should not be there"*) — but see F12, which is what decides whether anyone is looking.

---

## What I would change before building, in order

1. **F1/F2** — add a negative control (a synthetic non-builtin name that must classify `loud`) and an
   adjacent-success pair to `builtin_status.rs`, and resolve F8 so the `loud` producer survives Task 13.
   Without this the plan's central anti-skew mechanism dies silently at Task 13.
2. **F3** — state in the gate that the builtin-status criterion measures resolution, not correctness.
3. **F10** — repoint criterion 4's collector control at a root a builtin holds *inside its own body*.
4. **F12** — require `bif_assertions.rs`'s set-equality test to run ungated.
5. **F19** — make the family tasks declare their expected flipped rows before running the harness.
6. **F14/F15** — decide the `>I>`/`<I<` witness question and the directives-in-the-union question in the
   plan, not at implementation time.
7. **F16** — forbid an unargued `PASSED/PASSED` declaration in `mutate-4c.sh`.
8. **F18** — put `REXX_CORPUS_GATE=1` in the verify step of every task that adds a corpus witness.
