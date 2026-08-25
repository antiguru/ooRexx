# Task 2, fix round 1 — rulings on the review

Review: `.superpowers/sdd/2026-08-17-phase-5a/task-2-review.md`. **Spec compliance: APPROVE. Task
quality: REWORK.** Nine findings — two high, two medium, five low. Read it; each carries its
evidence, and several carry the corrected text.

**First, what held**, because it is most of the document and it survived a hard sample: all four C++
corrections are right on **both** halves, all three in-tree Rust corrections likewise, **every one of
the eleven new mechanisms is real, genuinely absent from the enumeration, and 5a's**, and every
measurement outside finding 4 reproduced to the digit. The reviewer reproduced finding 1 against the
oracle itself. This round does not reopen any of that.

---

## The root cause, because five of the nine are one defect wearing five hats

Findings **1, 2, 4, 5 and 8** are all the same thing: **a conclusion recorded without the procedure
that produced it, or with a procedure that does not reproduce it.**

* **1 and 2** record `none` / `not asserted anywhere`. Both are negative claims from a search, and in
  both the search was **narrower than the claim it licensed** — row `:120` searched the
  directive-declared hierarchy and missed the dynamically-built one in the same file; row `:136`
  searched for `ACTIVATE` as a name and missed two explicit pass-ordering assertions.
* **4** states the counting method as `/bin/grep -aciE` and four counts do not reproduce under it;
  one reproduces only under a pattern the ledger never names.
* **5** applies one standard of correctness to the "found wrong" list and a looser one, unstated, to
  the "checked and correct" list.
* **8** states a derivation pattern that, run as written, yields a different denominator than the one
  that produced the table.

**This matters more here than it would anywhere else, because Task 3 re-derives against this
document and enforces its staleness rule.** A count that cannot be reproduced reads as upstream drift
when nothing upstream moved. A `none` that was never really searched hands a later task the
three-signal rule's second signal for free — and the global constraints say **Task 2 is what makes
that signal checkable**. A false `none` is the dangerous direction.

**Ruled, and this is the shape of the whole round: every negative claim and every count carries the
exact command or the exact procedure that produced it**, verbatim enough to paste. Not "searched the
tree" — the pattern, the paths, the revision. A reader must be able to judge **how wide the search
was** without redoing it, and re-run it when they doubt it. Where a count needs `\b` anchors or a
`^::method` prefix to come out, that is the stated method.

**And a `none` that survives is written as what it is:** "no test matching `<pattern>` under
`<paths>` at r13178", never as "nothing upstream pins this". The first is checkable and the second
is a claim no search can support.

---

## Finding-by-finding

**1 — HIGH. Fix.** Row `:120` gets the dynamic hierarchy in
`Class.testGroup::test_INIT_INHERIT_UNINHERIT_SUBCLASS_MIXINCLASS_QUERYMIXINCLASS` (`:295`), with
`:386` / `:402` / `:411`-`:413` as the discriminator — the reviewer measured the oracle answering
`SwimCar: I swim now...` while the mixin is inherited and `RGF_VEHICLE_SWIM` after `~uninherit`.
Record it as a discriminator a later task can lift into a corpus program, which is the row's real
value. Also fix the two riders: `test_BASECLASS:177` is a second assertion over `:119`'s mechanism,
and the `:1245` call is correctly excluded because it sits inside `/* … */`.

**2 — HIGH. Fix.** Row `:136`: `REQUIRES.testGroup:320` `test_activate` pins pass 3 after pass 1, and
`CONSTANT.testGroup:496` `test_expression_activate` pins pass 2 before pass 3. Together they pin the
three-pass structure the row says nothing pins — on the row your own `processInstall` correction is
about.

**3 — MEDIUM. Fix.** `:126`'s headline citation is wrong on all three coordinates: the `define` calls
are `:198` and `:227` (not `:205`/`:236`, which are the `assertFalse`s), and they live in
`test_DEFINE` (`:188`) and `test_DELETE` (`:218`), not `test_issubclassof_non_class` (`:167`). **The
substantive claim survives** — this is the row the ledger leans on hardest, so its citation being
resolvable matters more than most.

**4 — MEDIUM. Fix.** Correct the four counts or state the pattern that produces them, per the ruling
above. Also `~defineMethods` needs `~defineMethods\(` and `reply` needs `\breply\b`.

**5 — LOW–MEDIUM. Fix.** Either apply the "found wrong" criterion to `ClassClass.cpp:1322`,
`:1631` and `PackageClass.cpp:1086` and correct them, or state plainly that the second list uses a
point-of-interest criterion. Do not leave two standards unlabelled in one document — the list is
introduced as "a re-checker should not redo these", which is precisely why it must say what "checked"
meant.

**6 — LOW. Fix.** `Alarm`'s `reply` is `:1555`, two lines before the native call at `:1557` with a
blank between, not `:1556` and not one line. The substantive claim holds and `Ticker`'s five line
numbers are all exact.

**7 — LOW. Fix.** `clsRexxContext`'s documented second route is **the `StackFrame` class's `context`
method** (`utilityclasses.xml:7540`-`:7545`), not a `~package`-side method. Everything else in
finding 10 verified exactly.

**8 — LOW. Fix.** The derivation pattern must match what was run: `StreamClasses.orx` has three
double-quoted `external "library REXX…"` directives. No member is missed and the conclusion stands —
the pattern as written just does not produce the table.

**9 — LOW. Fix by labelling.** The `Alarm` conclusion is sound and is a genuine correction to the
spec's "neither `~new` can succeed" — but *when* the `REPLY` continuation runs in a single-threaded
Phase 5 is a design choice **no task has made**, so "after the main program has finished" is
inherited from D55's two-program measurement rather than derived for this case. Mark it as the
prediction it is, and say the `Alarm` row's author must measure rather than inherit it.

---

## Verification

The five gate commands, each with its own exit status; corpus **106 of 106**. **Do not run
`REXX_PHASE_GATE=5a …`** — your reasoning for declining it was right and is now the standing answer
for every task until the constant exists.

Every citation you change, resolve at its new location before writing it. Every count you restate,
run and paste the command.

Append to `.superpowers/sdd/2026-08-17-phase-5a/task-2-report.md` under "Fix round 1".

**Return only:** status, commit SHAs, one line per finding, and anything you could not close.
