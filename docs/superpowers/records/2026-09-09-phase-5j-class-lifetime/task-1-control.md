# Phase 5j Task 1 — the negative control, prediction and result

The mutation: remove the `target.class_id().is_some() ||` term from the weak-clearing pass in
`crates/rexx-core/src/heap.rs`, leaving the predicate as it stood before this task.

## The prediction, written before the run

1. `a_weak_reference_to_a_live_class_answers_the_class` reddens on both engines, printing
   `live class: NIL` and `declared: NIL`.
2. The `live object: an Object` line stays correct inside that failure — it is the control that says
   the defect is about class identities and not about weak references.
3. No other test in the workspace reddens, because nothing before this task asserted a weak
   reference to a class. Named as the part most likely to be falsified.
4. The rest of the suite is unchanged.

## The result

**1. CONFIRMED.** The failure is

```
assertion `left == right` failed: TreeWalker
  left: "live class: NIL\nlive object: an Object\ndeclared: NIL\n"
 right: "live class: TEMPC\nlive object: an Object\ndeclared: DECL\n"
```

**On both engines, by two instruments rather than one.** The assertion stops at `TreeWalker`, so
this run witnesses only that engine. The `Ir` half is witnessed separately: `rexx-run` on the
unfixed build, whose default engine is the IR, printed `live class: NIL` / `declared: NIL` against
the oracle's `TEMPC` / `DECL`.

**2. CONFIRMED.** `live object: an Object` is correct on the failing side.

**3. FALSIFIED as stated, and the reason was mine.** The first run reddened a second test,
`sourceline_matches_the_interpreter_for_every_corpus_program`:

```
crates/rexx-parse/tests/sourceline_oracle/weakref_class.txt: no oracle expectation for corpus
program weakref_class (No such file or directory (os error 2)); regenerate per the module comment
```

That is not the mutation. A new `corpus/lang/*.rex` owes a `sourceline_oracle/<name>.txt`
expectation, and this task had not generated one — it would have failed with the fix in place too.
Generated with the module's own driver and the count checked against the file (21 lines, primary
`~source` path, not the fallback). **The control was then re-run from a clean tree to measure
claim 3 rather than argue it**, and the second run is what the claim rests on.

The near miss is worth recording: `cargo test --test corpus` passed after the file was added, and
`--test class_weak_reference` passed, so two targeted runs both said the tree was healthy. Only the
whole-workspace control saw the third test that a new corpus program obliges.

**4. CONFIRMED** on the second run, alongside 3.

## The clean run's reading

Second run, mutation applied to an otherwise clean tree:
`grep -c "^test result: FAILED" control2.log` answers **1**, and the failing test is
`a_weak_reference_to_a_live_class_answers_the_class` alone. Claim 3 rests on that run, not on the
first.
