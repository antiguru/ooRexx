# Task 11 review: the Array and the Directory 5a's own mechanisms send to

Reviewed `aba8b555e..f92e00b86` (`f4b21eadb`, `6f3434e88`, `f92e00b86`) against
`task-11-brief.md`, the plan's Global Constraints
(`docs/superpowers/plans/2026-08-17-phase-5a.md:240`-`:445`) and the tree at
`f92e00b86`.

## Verdicts

**Spec compliance: PASS.** Every construct the brief names is built and matches
the oracle on three descriptors on both engines: `(1,)~size` is `2`,
`(1,2,3)~items` is `3`, both `DO OVER` shapes iterate, and a Directory's
`~put`/`[]`/`~at` answer. The method rows that moved are the named ones --
`[]`/`AT`/`SIZE`/`ITEMS` on `.Array` and `[]`/`AT`/`PUT` on `.Directory` -- and
every other documented method on either receiver still refuses loudly, measured
(`.environment~items`, `.environment~hasIndex`, `.environment['K']='v'` and
`.local~items` are each `method "X" of class "Directory" is not implemented
(Phase 5)`). `ExprKind::List` moved to `Owner::InScope` and all five pinned
`owners.rs` items moved with it in one commit. `LoopKind::Over` was already in
scope and was not touched. No `Body` variant was added; no boxing decision was
reached. Nothing outside the brief's scope was implemented.

**Task quality: PASS WITH FINDINGS.** The code is sound and the tests are real:
the GC consequence of widening `Body::Array`'s payload is handled correctly and
has its own instrument, the three added refusals each name an in-crate test and
each of those tests carries answering rows that a blanket-refusal build would
redden, and the new corpus programs are covered by the collect-on-every-allocation
sweep as well as the differential gate. What is wrong is the performance
narrative and two ownership comments: the per-cause instruction figures written
into two source doc comments cannot be derived from the report's own bisection
table and the report's own decomposition contradicts its own residual, and the
`RAISE` ownership comment this commit rewrote asserts something a run of the
tree contradicts.

## What I ran

* All five gate commands from `rust/`, statuses read from unpiped exit codes:
  `cargo fmt --all --check` **0**; `cargo clippy --workspace --all-targets --
  -D warnings` **0**; `cargo test --release --workspace` **0**;
  `REXX_CORPUS_GATE=1 cargo test --release --workspace` **0**, `149 of 149
  matching`; `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace
  --no-fail-fast` **0**, `149 of 149 matching`. `target/release/rexx-run` was
  rebuilt before any probe, so no probe read a stale binary.
* My own differential probe programs, run from a fresh empty directory
  with absolute paths, three descriptors captured separately, never `2>&1`,
  oracle under `ulimit -v 1048576` and `timeout -s KILL 10`, and both crate
  engines. Batches: the brief's own rows; the Array and Directory refusal
  ladders and their answering neighbours; the builtins that quote a rejected
  argument, each given an array;
  the other value positions (`DO`, `FOR`, `NUMERIC DIGITS/FUZZ/FORM`, `SIGNAL
  VALUE`, `ADDRESS VALUE`, `RAISE ADDITIONAL`/`ARRAY`, `IF`, `WHILE`,
  `SELECT CASE`, `PARSE VALUE`, `INTERPRET`, an argument to a `CALL`); the
  trace surface under `TRACE I`/`TRACE R` for arrays, nested arrays, class
  objects and both directories; `UNKNOWN` on a user class and on a class side.
  Every one matched except the families section 6 of the report already names
  and the refusals section 5 names.
* **I did not run anything in `rust/corpus/oracle-crashes.txt`.**
* Staleness test from the repository root: `git log --oneline
  15a1ffa98..HEAD -- rust/crates rust/Cargo.toml rust/Cargo.lock Cargo.toml`
  lists 43 commits and **every one of them is named in `progress.md`** (checked
  by grepping the ledger for each SHA in a loop, no misses).
  `git merge-base --is-ancestor 15a1ffa98 HEAD` holds, and
  `sha256sum bench-baselines/pinned/rexx-run-15a1ffa98` matches `PINNED.md`.
  The pin is live.
* Arithmetic re-derived from `bench-baselines/phase-5a-arms.tsv` rather than
  from the report's tables.

## The seven flagged items, ruled

1. **`Body::Array`'s payload widening -- holds, and the GC consequence is
   right.** `rexx-core/src/body.rs:325` is now
   `out.extend(items.iter().filter_map(|item| *item))`, so every filled slot is
   still traced and an empty slot reaches nothing. `tests/trace.rs`'s new
   `an_array_reaches_past_an_empty_slot_and_not_through_it` asserts the walk
   crosses a hole rather than stopping at it, and the pre-existing
   `an_array_reaches_every_element_including_duplicates` still asserts the exact
   output vector -- so a trace arm that dropped filled slots (the
   use-after-free) is red in a plain `cargo test`, ungated. `size_of::<Body>()`
   genuinely did not move, but not for the reason the report gives: see
   observation O4. Task 9's allocation site
   (`dispatch.rs:1492`-`:1510`, `native_superclasses`) maps `Some` over the
   class handles and keeps its "every item is a class identity, so the
   allocation cannot collect one out from under this array" note; correct.
   Every other `Body::Array` reader was updated consistently -- `text_len` goes
   through `Redirect::Array`, `try_text` answers `None`, `heap_to_number` parses
   the joined string, and `over_items`/`array_index`/`array_slots` each take
   items or slots deliberately.
2. **The two open divergence families -- the "pre-existing" claim holds, and I
   checked it rather than accepting it.** Both were re-measured on routes that
   do not go through `ExprKind::List`: `a = .Array~superClasses; numeric digits
   a` is rc 230 on both sides with the oracle quoting `"an Array"` and this
   crate quoting `"The Object class"`, and
   `a = 'say 1' || x2c('0a') || 'say 2'; interpret a` is rc 243 on both sides
   with the oracle reporting the newline and this crate printing the literal
   `&1`/`&2`. Neither route touches code this diff changed, so the families
   predate the task. The task's new route makes each *program* newly divergent
   (`numeric digits (1,2)` was rc 120 loud before and is now a wrong message),
   which the report states. `NUMERIC FUZZ` and `NUMERIC FORM VALUE` are in the
   same family and the report names all three.
3. **The control could not fail, and the bisection arithmetic does not hold.**
   The percentage table checks out row for row against the TSV and no axis
   reaches 1% (worst is `strings`/`ir` at +0.969% and `strings`/`tw` at
   +0.940%), and the budget claim checks out: `strings`/`ir` head reads
   5421.5180 at `6f3434e88` against 5421.5179 at Task 10 and 5421.5180 at Task
   9, so this task spends none of the standing budget on the guard's own axis.
   The per-cause attribution does **not** check out -- finding **B1**. The
   control's vacuity is observation **O1**.
4. **The oracle crasher -- refusal is the right disposition and the instrument
   witnesses it.** There is no oracle behaviour to match, which is the same
   ground entry 5 stands on, so refusing rather than answering is right, and
   `Loud::array_index_hole` prints no owner suffix because the construct is
   implemented. `an_expanded_index_of_one_empty_slot_is_loud` asserts the exact
   stderr and rc 120 for three spellings **and** four neighbouring rows that
   answer or raise cleanly (`a~at((1,))` is `1`, `a[(2,)]` is `2`,
   `a~at((1,,3))` is 93.926, `a~at((,))` is 93.901), so a build that refused the
   whole spread, or that read the item count as the size, reddens rather than
   passes. I verified independently that **nothing runs the program against the
   oracle**: the three sources appear only in `dispatch.rs`'s own test module,
   whose `both_engines` calls `crate::run_program` for the two crate engines and
   never the oracle; `at((` matches nothing under
   `crates/rexx-exec/tests/ir_dual_cases/`, nothing in `corpus/`, and nothing in
   any `sourceline_oracle` file. I also confirmed by reading `array_index` that
   the refusal cannot fire on a shape the oracle answers: the spread produces a
   single `None` element only when the item count is one and slot one is empty,
   which is exactly the crashing shape, and `~at(,)` arrives as no argument at
   all (witnessed by `array_index_refusals.rex`'s row 3 matching the oracle's
   93.901).
5. **The `DO OVER ... FOR` fix has a test and was in scope.** `do e over 'abc'
   for 0` leaving the control variable at `abc` is a row of
   `corpus/lang/array_do_over.rex` (`f = 'pre'` then `do f over b for 0` then
   `say 'after zero' f`), and I measured the program clean on both engines.
   Closing it was not drive-by: `LoopState::OverOnce` had to be replaced
   wholesale to iterate an array, and the `FOR`-after-bind order is the same
   `checkOver && checkFor` the new state implements, so the old order could not
   have been preserved. The report names it, which is what the plan asks of a
   side effect.
6. **The `spike.rs` move was not a sixth ownership item, so no `owners.rs`
   amendment is owed here.** `owners.rs:587`-`:632` says a variant move "must
   update every one of the five items below in the same change, or one of the
   tests above (or in `coverage.rs`/`loud.rs`) fails". That is a
   necessary-condition claim about the five and it is still true. `spike.rs`'s
   two witnesses are not ownership data: they need *some* refused form, and that
   file's own doc already states the property rather than predicting a witness
   and already records having been reddened by Task 5's `~` landing -- an
   earlier ownership move. So the five-item block's implied boundedness was
   already inaccurate before this task and this commit did not make it worse.
   Recorded as observation **O6** rather than a finding. The report's own count
   of witnesses is wrong: observation **O2**.
7. **Report not in git history -- ignored as instructed.**

## Findings that block

### B1. The two per-cause instruction figures in `eval.rs` and `value.rs` are not derivable from the bisection, and the report's decomposition contradicts its own residual

* `rust/crates/rexx-exec/src/eval.rs:670`-`:672` (`Interp::eval_cold`'s doc):
  "an arm of its own for `ExprKind::List` cost the `strings` axis **43**
  instructions per pass on the tree-walker arm and the `varlookup` axis **4**
  ... where answering it from behind the existing catch-all **costs both
  nothing**."
* `rust/crates/rexx-exec/src/value.rs:682`-`:683`
  (`Interp::string_value_text`'s doc): "inlined into `Interp::intermediate_text`
  and `Interp::result_text` ... it cost the `strings` axis **29** instructions
  per pass on the tree-walker arm and the `varlookup` axis **7**."

The report's own bisection table is the only evidence for these, and the
marginal costs it supports are different numbers. Writing the table's rows as
A = `HEAD~1`, B = `f4b21eadb` as committed, C = minus the `List` arm, D = minus
the trace gates, E = `6f3434e88`'s outlined shape:

| quantity | `varlookup` | `strings` |
|---|---|---|
| B - A (the whole move) | 16.000 | 72.000 |
| B - C (marginal cost of the `List` arm) | 4.000 | **39.000** |
| B - D (marginal cost of the trace gates) | **11.000** | 29.000 |
| E - A (residual after both fixes) | 12.000 | 33.000 |

Two of the four published figures are the marginal readings (`varlookup` 4 =
B - C, `strings` 29 = B - D). The other two are not: `strings` 43 is D - A and
`varlookup` 7 is C - D. Three different subtractions across four figures, and
neither doc comment says which.

The decomposition is also self-contradictory. 43 + 29 = 72, which is exactly
B - A, the entire move -- so the report attributes the whole regression to the
two causes and then, in the next paragraph, states that 33 (`strings`) and 12
(`varlookup`) are left unattributed after both were fixed. Both cannot be true.
On `varlookup` the published figures sum to 11 against a measured total of 16
while a residual of 12 is claimed, over-attributing by 7. And `eval.rs`'s "costs
both nothing" is contradicted by E - A: the outlined shape sits 12 and 33
instructions per pass above `HEAD~1` on the two axes it names, which is precisely
what the report's "the residual is not attributed" paragraph says was not
placed.

Nothing in the repository lets a later reader re-derive any of it: the bisection
variants and the control appear nowhere in `phase-5a-arms.tsv` (the only `build`
values under `task` `11` are `pinned`, `head` and `pinned>head`, and the string
`control` appears nowhere in the file).

**Why it blocks.** The plan lets measurements keep their numbers in comments
precisely because a comment's number is evidence a reader relies on; two of
these four numbers are not the quantity the sentence claims, and one sentence
asserts a zero the same bisection reports as 12 and 33.

**Fix.** State one subtraction and use it for all four figures -- the marginal
reading (B - C and B - D) is the one that matches what the sentence claims,
i.e. `strings` 39 / `varlookup` 4 for the arm and `strings` 29 /
`varlookup` 11 for the gates -- say in each comment that the two are not
additive, drop "costs both nothing" in favour of the outlined shape's own
residual, and either commit the bisection rows to the TSV under a `task` label
of their own or say in the report that they are not recoverable.

### B2. The rewritten `Raise` ownership comment asserts a state a run contradicts

* `rust/crates/rexx-exec/src/lib.rs:1733`-`:1738` (`instruction_owner`):
  "`RAISE` is likewise whole: `ADDITIONAL (a, b)` is a parenthesised list, which
  is an *expression* and is implemented, and `ARRAY (a, b)` reaches the
  identical oracle bytes".
* `rust/crates/rexx-exec/tests/owners.rs:150`-`:155`: "... and `ExprKind::List`
  is in scope. So there is no `RAISE` shape whose gap belongs to `RAISE`."

Measured, three descriptors, both engines, fresh empty directory:

```text
raise syntax 93.900 additional (1,2)
  oracle  rc 163   Error 93.900:  1.
  crate   rc 120   rexx-exec: an array as a RAISE ADDITIONAL value is not implemented (Phase 5)

raise syntax 93.900 array (1,2)
  oracle  rc 163   Error 93.900:  1.
  crate   rc 163   byte-identical to the oracle
```

So `ADDITIONAL (a, b)` is *not* implemented, the gap now belongs to `RAISE`'s
own refusal (`run.rs:4558`, `Loud::object_position("a RAISE ADDITIONAL value",
kind)`), and the two spellings that are byte-identical on the oracle are not
byte-identical here -- which is the opposite of what the sentence offers its
measurement in support of. `InstructionKind::Raise(_) => None` in
`instruction_owner` and `("Raise", Owner::InScope)` in `owners.rs` assert the
same false thing, and nothing checks it: `loud.rs`'s
`every_out_of_scope_variant_fails_loudly` only walks the phase-owned rows, and
`owners.rs:625`-`:626` says outright that an owner on an implemented variant is
"data no execution path reads".

The **refusal itself predates this task** and I checked that rather than
assuming it: `raise syntax 93.900 additional (.Array~superClasses)` is rc 120
with the same message and rc 163 `Error 93.900:  The Object class.` on the
oracle, and a bare `additional a` is a 35.1 parse error on both sides, so the
site was reachable at Task 9 through the parenthesised single expression. The
finding is therefore the claim and not the behaviour -- but this commit rewrote
exactly this sentence in both files and had to re-read it against the tree,
which is the per-task rule the plan's Global Constraints put in place of a
citation check.

**Fix.** Either give `Raise` an arm-grained row and owner for the
`ADDITIONAL <array>` shape, or expand the array into the substitution list the
way the `raise.array` arm already does (which would make both spellings match
the oracle and make the comment true as written). Do not leave the sentence
standing.

## Observations that do not block

* **O1. The sitting's control is vacuous on its instrument, and the conclusion
  drawn from it does not follow.** A rebuild differing by one comment line
  produces identical machine code, so `instructions:u` had to come back exactly
  1.000000 whether or not layout can move that counter. The report says plainly
  that the control cannot fail, which is the right disclosure, and then
  concludes "on this instrument layout is not a term at all" -- a conclusion the
  control cannot support. The conclusion happens to be true for a different
  reason (an instruction count is a function of the executed path, not of
  addresses), and the actual attribution came from the bisection, so nothing
  downstream rests on it. Worth striking the sentence rather than the control.
* **O2. "the sixth witness that file has had" is one too many.** Both of
  `spike.rs`'s docs enumerate their own sequence and each lists five:
  `do i = 1 to 3` / `call "sub"` / a message send / `ExprKind::List` /
  `ExprKind::ClassResolver`, and `+` / `=` / `~` / `ExprKind::List` /
  `ExprKind::ClassResolver`. Report prose only; the file itself says nothing
  about a count.
* **O3. The per-axis table folds `small` and `large` into one column on a claim
  that is not quite true.** `alloc4c`/`tw` is 1.004569 at `small` and 1.004452
  at `large` (+0.457% against +0.445%), so the two do not agree to three
  decimals on every row. Both are far under the threshold, so no verdict moves.
* **O4. The `size_of::<Body>()` claim is true but its stated verification cannot
  establish it.** `rexx-core/src/body.rs:302` is
  `const _: () = assert!(size_of::<Body>() <= 80)`, an inequality, so a
  successful build says nothing about whether the size moved. What does say it
  is the type: a `Vec<T>` is three words for every sized `T`, so replacing
  `Vec<ObjRef>` with `Vec<Option<ObjRef>>` cannot change the variant's width.
  The report's "8 bytes per element" is right -- `ObjRef` is a plain
  `pub struct ObjRef(u64)` (`rexx-core/src/handle.rs:108`) with no niche, so
  `Option<ObjRef>` is 16 bytes.
* **O5. Two comment claims name the size of a set, and one of them I could not
  match to a set of that size.** `dispatch.rs:394` says a `StringTable`
  "answers the same three method names", a count of a set `NATIVE_METHODS`
  enumerates and a fourth `Directory` row would silently falsify.
  `lib.rs:809` says "the same reason `run.rs`'s two carve-outs print no
  suffix"; `owned_message(.., None)` occurs once in the crate (`lib.rs:812`,
  `array_index_hole`, reached from `dispatch.rs`), and the suffix-less
  constructors reached from `run.rs` are `missing_body`, `op_not_driven` and
  `call_op_off_its_node` -- three, and all internal-inconsistency messages
  rather than carve-outs. Name the set instead.
* **O6. `owners.rs`'s five-item block implies a bounded failure set that is not
  bounded.** "or one of the tests above (or in `coverage.rs`/`loud.rs`) fails"
  omits `spike.rs`, which this task reddened and which Task 5 reddened before
  it. Pre-existing, and the ruling in item 6 above is that this commit owes no
  amendment; worth a sentence next time that block is edited.
* **O7. `Array~at` clones the whole slot vector to read one element.**
  `dispatch.rs:1656`-`:1660` (`array_slots`) does `items.clone()` and
  `native_array_at` calls it per subscript, so `a[i]` is O(size) in both time
  and allocation. Correctness is unaffected and no bench axis allocates an
  array, so the sitting cannot see it; the doc's reason for cloning (the caller
  needs `interp` afterwards) is real but `native_array_size` and
  `native_array_at` only need a count and one slot.
* **O8. Report prose: "all four rows in `.Array`'s own dictionary".**
  `NATIVE_METHODS` has six `Array` rows; `MAKESTRING`/`TOSTRING` predate the
  task. The table underneath is right about which four moved.

## What a check here could not see

* The corpus gate cannot see any of the three added refusals, nor the `RAISE
  ADDITIONAL` refusal in **B2** -- a refusal the oracle does not share is not a
  differential row. Each added refusal names an in-crate test; the `RAISE` one
  names none, which is why the comment claiming it away matters.
* My probes cannot establish the absence of a divergence, only its presence.
  Had this task got the empty-slot distinction wrong, `~items`, `DO OVER` and
  the string join would each have shown it; had it got the rooting wrong, the
  collect-on-every-allocation sweep over `phase-5a.txt` (now including the six
  new programs) would have shown a plain-versus-stress mismatch. Both are live
  instruments, and both are green.
* I did not re-derive the bisection or the control by building the variants;
  **B1** is an internal-consistency finding against the report's own table and
  the committed TSV, not a claim that the true costs are 39 and 29.
* I made no edits, so no mutation was applied to the tree. Other agents are live
  in this worktree and an edit-and-revert has no witness. Where a mutation would
  have been the check, I read the test for whether a wrong build reddens it
  instead, and said so per test above.
