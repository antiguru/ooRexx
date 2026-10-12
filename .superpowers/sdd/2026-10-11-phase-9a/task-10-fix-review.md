# Task 10 fix review (9cd9ebd7c..39b78a0e3)

## Verdicts

1. TESTDELEGATE fix: PASS. Class-side Method objects are keyed (defining class, true, name); instance side
   unchanged (owner, false, name). Only reflection touches it: `method_object`/`hold_method_object` are called
   from `instanceMethod(s)`, `Class~method`, `defineMethod(s)`, `defineClassMethod`, `enhanced`,
   `RexxContext~executable`; no send/dispatch path reads `method_objects`. GC: the cache is pruned in the
   same `freed_classes` retain (lib.rs:3134, key.0 is the defining class) and the object is rooted by
   `class_owns(scope)` as before; `defineClassMethod` now holds under the class-side key
   (identities.rs:537), so `instanceMethod` after `defineClassMethod` answers the defined object.
2. Witness: PASS. Oracle run from /tmp/claude-1000/p9/rv10, rc 0. stdout (9 lines) includes `0 1` then
   `1 0` for the first/second packages (guarded vs protected unguarded): the second block reads its own
   flags, which is the shape that broke TESTDELEGATE; the third and fourth print `1 1 0` and `0 0 1`.
3. Method rexxc rows: PASS. Spec section 3 and criterion 4 (spec lines 148, 357-359) name
   TEST_NEW_ARRAY_FROM_FILE and TEST_NEW_FILE_COMPILED as 9b/9c witnesses. They are recorded through the
   existing DIFFERING mechanism (the `rest` rows already carried the same pair), which `sim_gate` reads via
   `failing_unswitched`; no exemption, no suite-9d-set entry, no gate edit.
4. Rows re-derived: PASS on the evidence available. The 101-then-0 sequence in the report shows the
   ATTRIBUTE whole rows were corrected only after the gate named them stale; the new strings
   (`assertions 92`/`303`, last started, failing sets) are the runner's output form. I did not rerun.
5. Prose: one violation (Minor 1).

## Findings

Critical: none.

Important: none.

Minor:
1. rust/crates/rexx-exec/src/lib.rs:1428 (rewrapped doc comment) ends line with ` -- see`; the rule bans
   ' -- ' and the line is an added line in this diff, so it counts as new prose. Failure: a prose grep over
   the diff's added lines flags it. Fix: replace with a period or a colon.
2. Instance-side identity across a subclass (`.D~new~instanceMethod('M') == .C~method('M')`, oracle 1,
   crate 0) is unchanged and reported as open in task-10-fix-report.md; it is the same defect class on the
   other side and the witness does not cover it. Not a regression.
3. introspection.rs `method_object_for` ClassSide arm passes `scope` to `method_object`, which calls
   `class_owns(scope, ..)`, panicking if `scope` is not a live class. The `.nil` scope with no
   `executable_objects` entry (the object-level setMethod entries `object_level_entries` yields) would
   reach it. My probe (setMethod on a class from a class method, then `instanceMethod`/`instanceMethods(.nil)`
   on the 23:30 `rexx-run`) did not reach the arm: the crate answered `The Object class` and an empty
   supplier where the oracle answers Method and `R`, the same as on the instance side, so that gap predates
   the fix. Unverified; if setMethod ever registers a source-built method without an `executable_objects`
   entry this becomes a panic instead of a wrong answer.
4. suite-9d-set.txt still lists METHOD and ATTRIBUTE TESTDELEGATE though both pass (reported as left
   open, derived list; correct to leave but it now overstates what 9d needs).
