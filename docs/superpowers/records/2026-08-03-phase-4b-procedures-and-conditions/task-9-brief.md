### Task 9: Trace for calls, and the Controlled-loop `>>>` gap

**Files:**
- Modify: `rust/crates/rexx-exec/src/trace.rs` (`>A>`, `>F>`, `>R>`, the activation indent base)
- Modify: `rust/crates/rexx-exec/src/run.rs` (the two missing `>>>` lines on a Controlled loop's re-tested pass)
- Modify: `rust/crates/rexx-exec/tests/trace_oracle.rs` -- including **`CLAIMED_PREFIXES`**, which is asserted against the witness union and breaks the moment a prefix lands. It was at `:233` when this plan was written and `:254` at `e72cc19f`; find it rather than trusting either number.
- Modify: `docs/superpowers/plans/phase-4-exclusions.txt`

**Interfaces:**
- Consumes: Task 3's activation stack; `static_indent`, whose signature does not change; Task 2's clamp, which this task must not duplicate or contradict.

**Why:** the trace surface is where 4a's four late divergences were found, all by probing adjacent shapes rather than by the table.

**The measured `trace r` transcript for a two-argument call:**

```
     2 *-* call sub 1, 2
     5 *-*   sub:
     5 *-*   procedure
     6 *-*   use arg a, b
       >>>     "1"
       >>>     "2"
     7 *-*   return a + b
       >>>     "3"
       >>>   "3"
     3 *-* say 'done'
       >>>   "done"
done
     4 *-* exit
```

Four things to read off it: the callee's `sub:` **label clause is echoed**, and so is `procedure`; callee clauses sit at the caller's indent plus two; `use arg` emits one `>>>` per argument; and **the return value is traced twice**, once at the callee's indent and once at the caller's.

**Inherited items this task pays for:**

* **I14, corrected twice.** 4b's prefixes are `>A>` (ARGUMENT, both call forms), `>F>` (FUNCTION, expression form only) and `>R>` (ALIAS). `>R>` is a **RESULTS**-level prefix, not intermediates-only: at `trace i` the call site shows `>O>   ">" => "PP"` and `>A>   "orig"` (the *value*, not the name), then the callee shows `>R>     "PP" => "Q"`; at `trace r` the `>R>` line is still emitted; at `trace l` only the label clause appears. **`>I>`/`<I<` are real and belong to `::routine`/`::method` under `TRACE LABELS`** -- the C++ gate is `tracingLabels() && isMethodOrRoutine()`. An internal-label call with `trace l` first emits nothing, with or without `PROCEDURE`; `trace l` in the caller targeting a `::routine` emits nothing. So their owner follows Task 3 Step 4's answer about whether 4b reaches a `::routine`. **Write the answer into `phase-4-exclusions.txt` as "`::routine`/`::method` under TRACE LABELS", not "method invocation".**
* **I31.** A Controlled (`TO`-style) loop's re-tested pass omits two `>>>` value lines. Measured, cause read from `DoBlock::checkControl` (`interpreter/.../DoBlock.cpp`) rather than inferred, and costed at about twenty lines plus re-verification of bound-before-test, `FOR` and `ITERATE` -- half a day, not a rewrite. **Close it here.** An overstated cost is how a cheap fix stays open, and this row was corrected once for exactly that.

- [ ] **Step 1: Regenerate the five existing trace expectations to prove the harness still round-trips**

`tests/trace_oracle.rs`'s module doc carries the regeneration command, and all five were verified byte-identical in 4a.

- [ ] **Step 2: Record `>I>`/`<I<` as NOT 4b's -- Task 3 settled it by measurement**

`>I>`/`<I<` require `trace l` on a `::routine` or `::method`, so the question is whether 4b reaches one.

**Task 3 first recorded "not reachable at all" and that was wrong; its review caught it.** The `max` witness -- `::routine max` alongside `call max 1,2`, where the builtin wins -- is the one shape where "behind the builtin step" and "unreachable entirely" predict identical bytes. Measured with a non-builtin name, `call zorkolo` with a `::routine zorkolo` runs on the oracle at rc 0.

**The true position: a `::routine` is reachable in 4b for any non-builtin name, and 4b deliberately does not implement it.** The deferral is a scope call, not an impossibility -- getting builtin-colliding names right needs 4c's table, and a wrong answer there would silently run the wrong routine rather than fail loudly. Today the construct fails loudly naming `4c`, which is correct behaviour.

So these two prefixes are out of scope **by decision**, not by unreachability, and the row must say which. Write it as "`::routine`/`::method` under TRACE LABELS, deferred with `::routine` dispatch itself to 4c", and carry Task 3's three measured differences that 4c will meet: a `::routine` activation has its own variable pool, builtins shadow it, and **`TRACE` does not cross into it at all** -- a caller's `trace r` echoes none of its clauses.

Not as "method invocation", and not as "Phase 5's".

**Corrected 2026-08-04, before Task 9 was dispatched.** This step previously ended with a third instruction to write the row as "unreachable until 4c's builtin step exists" -- which contradicts the paragraph above it, contradicts the measurement that settled the question, and is the exact error the next paragraph warns against. It was removed rather than reconciled. A duplicate of the three-measured-differences sentence went with it.

The first revision of this plan reached a wrong conclusion here by probing `trace i` on an internal label, an instrument that could never have produced these prefixes. The lesson generalises and is worth keeping in the row: absence under one instrument is not evidence of ownership.

- [ ] **Step 3: Commit the new expectations, and update `CLAIMED_PREFIXES`**

- [ ] **Step 4: Implement `>A>`, `>F>` and `>R>`**

- [ ] **Step 5: Add the activation indent base, per D2r**

The base is the **calling clause's printed indent** plus the delta, not two times the depth. `static_indent` stays a pure function of the flat instruction list. Task 2 clamped the `*-*` echo at 40 columns and the value lines are **not** clamped -- measured, at nesting depth 25 `*-*` tops out at 40 while `>>>` runs to 52. Do not extend the clamp to value lines and do not re-implement it.

- [ ] **Step 6: Close I31's two missing `>>>` lines, and re-verify bound-before-test, `FOR` and `ITERATE`**

**Still yours after DEVIATION 0.** That deviation normalises leading *indentation*; I31 is two value lines that are **absent**, which is a content difference normalisation does not touch. Same C++ path -- the failing-control-test exit -- different symptom. A note earlier in this phase wrongly claimed the two closed together.

**Two more absent-value-line gaps arrived from Task 7 and are also this task's.** Added 2026-08-04; without this they lived only in a review summary, and one of them had already gone missing once that way.

*`EXIT <expr>` emits no `>>>` value line.* Measured at `e72cc19f`, both sides rc 0:

```text
trace r          oracle stderr:              ours:
say 'a'               2 *-* say 'a'               2 *-* say 'a'
exit 0                  >>>   "a"                   >>>   "a"
                      3 *-* exit 0                3 *-* exit 0
                        >>>   "0"                  (nothing)
```

Task 7 reproduced this on a three-line program containing no conditions, which places it in `InstructionKind::Exit`'s arm. **It constrains corpus programs**: any `trace r` witness containing `exit <value>` will diverge until it is closed, which is why `condition_traps.rex` deliberately contains none.

*Four both-DIFF probes in Task 7's round-4 A/B*, described there as a controlled `DO`'s per-pass `>>>` control-variable lines under `trace r`. **Determine by measurement whether these are I31 or a third gap, assuming neither.** "Same C++ path, different symptom" has already been wrong once on this exact pair.

Closing fewer than three is allowed. Whatever is not closed gets a KNOWN GAP row with an owner and a measured transcript, and the choice is stated -- the failure mode here is a gap dropping silently, not a gap deferred.

- [ ] **Step 7: Add a coverage measure to the trace table**

D14's criterion 3 amendment. The honest statement today is that the five witnesses verify what they cover and the trace surface's coverage is measured by nothing. Produce a number: which of the nineteen prefixes have a committed expectation, which do not, and which are out of scope with an owner. **A printed number that no assertion reads cannot fail** -- assert the count against a committed literal.

- [ ] **Step 8: Run the suite, remove I31's KNOWN GAP row, commit**

Removing a KNOWN GAP row needs the gap closed and a witness in the tree.

---

