# Task 6, fix round 2

Round 1 verified well. The narrowed predicate **holds from the under-firing side** -- the reviewer
attacked it with 12 boundary programs (unset, `String` default, dropped stem, nested-unset,
nested-`String`, self-assigned stem, compound `b.1`) and all are byte-identical on both engines, and
it re-ran your mutation to confirm the un-narrowed predicate reddens the new test. The DO-header site
is genuinely the same call with no duplicated block, and `FOR`, bare repeat, `DOWNTO`, a `WHILE`
bound, `LOOP`, nested `DO`, `TRACE R` and `TRACE I` are all byte-identical. The control was verified
by running it: disabling `blame_stem_forwarded_operator` at its definition reddens exactly 6 of 112,
each by exactly the frame line. The staleness test was re-run against all 22 commits, not a sample.

Five things follow. One is a real gap in the task's own goal; the rest are prose.

## 1. The mechanism covers one error class out of the several the goal names

**This is the important one.** The frame fires only on a `41.1` **conversion** failure. An error the
forwarded method raises *after* converting carries the oracle's frame and not ours. The reviewer's
witnesses:

* `s. = 1` then `say s. / 0` -- `42.3`
* `say s. ** 999999999999` -- `26.8`
* `s. = 'abc'` then `say s. & 1`, and `say \s.` -- `34.901`
* `numeric digits 1` / `s. = '9.9E999999999'` / `do i = s. to 5` -- `42.901`, which is **inside this
  round's own `match` at `run.rs:7295`**, because the `Ok` arm's `round_via_unary_plus` is the same
  forwarded unary `+`.

Not a regression -- the pin behaves the same -- but your task's goal sentence is *an error raised
inside a native method that an operator invoked emits the frame line*, and that is four error classes
unmet. Your report's "nothing I could not close" is false as it stands, and that alone has to change.

**Ruling: fix it, with the same escape hatch that applied to the DO site and that you were right to
take.** If the shape is "carry a flag saying we are evaluating a stem-forwarded operand, and blame on
any raise while it is set" -- one state instead of a predicate at the failure site -- build it, and
give the five witnesses corpus programs on the same terms as the six you already have. If it is not
that, if it needs a mechanism rather than a flag, **stop and report precisely what it needs**, name
the five witnesses in the report and in the corpus-gap record, and I will take that. A precise report
is acceptable; the silent version is not.

Judge honestly which it is. You are the one holding the code.

## 2. Set-size phrases, four of them

`corpus.rs:314`, `phase-5a.txt:178`, `run.rs:7292` are new this round; `phase-5a.txt:171` survived
round 1. A comment may not name the size of a set, true counts included. Measurements keep their
numbers -- the distinction is whether the number came from running something or from counting what you
just wrote.

## 3. A false premise at `eval.rs:1097`

The comment says an `.environment` default refuses. Measured: the oracle answers `+` with `.nil` at
**rc 0**. Correct it to what it does.

## 4. `run.rs:7288`, "each header position"

Loose enough to be read as covering positions the code does not reach. Say which.

## 5. The sitting's ratio claim is false for two rows

"every ratio ... min equal to max" does not hold for `alloc4c/ir/small` (max 1.000001) or
`strings/tw/large` (min 0.998014). The medians are 1.000000 and the layout conclusion stands, so what
needs correcting is the claim, not the conclusion.

## 6. The negative control's transcript, again

`task-6-report.md:398-402` reports the control in one sentence. The reviewer ran it and confirmed the
substance, so nothing is in doubt -- but the transcript is what makes the claim checkable by the next
reader without a reviewer, and this is the second round it has been asked for. Paste it: the mutation,
the six reddened programs with the failing output, and the restored `112 of 112`.

## Verification this round owes

* The five gate commands, each status read **unpiped**.
* If you take finding 1's fix: the five witnesses measured on both engines against the oracle, before
  and after, and the control extended to them.
* If you do not: the precise report of what the mechanism would need, and the witnesses recorded where
  the corpus records its gaps.
