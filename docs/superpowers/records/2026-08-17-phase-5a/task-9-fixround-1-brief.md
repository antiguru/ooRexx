# Task 9, fix round 1

**Approved.** The build is spec compliant and the one criterion it misses was unsatisfiable as the
brief worded it -- that is mine to fix, not yours, and I am amending the plan rather than asking you
for more.

What the review established by running it: all eight new corpus programs byte-identical on both
engines against the live oracle; 14/11/38 reproduced with all 25 class names; and the eleven differing
rows differ on **exactly** the `superclasses` line by **exactly** the `CoreClasses.orx` `~inherit`
targets, with fourteen line citations checked. Risk 1 came back clean from both directions over 31
extra names, including `DEFINECLASSMETHOD` and `INHERITINSTANCEMETHODS`, which `RexxClass::removeSetupMethods`
strips after `Setup.cpp` adds them -- a shape neither of us named. Risk 2 clean, including four parting
shapes your corpus omits.

**The `+0.9685%` is a pass and not a finding dressed as one**, on the reviewer's own check of both
sittings against the TSV to six decimals. Two things to carry: you reported the over-threshold reading
rather than hiding it, which is why the pair is legible at all; and **there are 0.03 points of headroom
left**, so the next task that adds an arm to `to_text`/`text_len`/`try_text` crosses the guard. That
goes in your report as a stated consequence, not as a footnote.

## 1. A sixth guessed citation, in the one place a line number was missing

`dispatch.rs:1397` cites `PackageClass::getName`, which does not exist. The method is
`PackageClass::getProgramName` -- `classes/PackageClass.hpp:147`, bound at `memory/Setup.cpp:1189`.

**It is the only citation in the diff without a line number, and it is the only one that was wrong.**
That is worth more than the fix: a citation without a line number is one nobody printed. Add the line
numbers.

## 2. A method object answers nothing, and that is an unreported divergence

`.Array~method('APPEND')~class` is `rc 120` where the oracle answers `The Method class`. And
`native_method` mints a **fresh** object where the oracle returns the same one -- `==` is `1` there and
is not here.

I am not asking you to implement `.Method`'s protocol; that is not this task. **Record both, with the
measurements, where the corpus records its gaps**, and say which task the identity half belongs to. A
divergence found and not written down is the one nobody finds twice.

## 3. Prose and enumerations

* `dispatch.rs:1930` enumerates "the receiver kinds a program can put on the left of a send" and your
  own `~method` adds one the list does not have. **Delete the enumeration** rather than extending it --
  that is the rule that finally held on the previous two tasks.
* `corpus/lang/class_method_own_dictionary.rex:12` enumerates the covered classes in a comment. Third
  copy of that shape.
* `value.rs:669` says "the only caller"; there are three -- `:601`, `:729`, `:1047` -- and `to_number`'s
  does not go through `Redirect`.
* `corpus/phase-5a.txt:288` is a one-word wrap artifact (`# one`).

## 4. Two claims in the report that need correcting rather than defending

* **The mandated coverage list stops at row 10.** Rows 11 to 14 are missing -- the `::ATTRIBUTE` pair,
  which the reviewer verified: `A`/`A=` answer, `B`/`B=` raise. The brief says that named list is the
  whole extent of the protection, so a list that stops early is the protection stopping early.
* **"Real work, not noise" is asserted.** `strings.rex` builds no array, so the arm's *body* never runs.
  The cost must therefore be in `try_text`'s enum dispatch, which your report never says. Say what the
  cost actually is; the conclusion survives, the reason has to be the real one.

## 5. Two controls to sharpen, both cheap

* The "not an inlining threshold" control used `#[inline]`, which is a **hint**. `#[inline(always)]` is
  the test. Re-run it, or drop the claim.
* Control 2's two-mutation necessity was verified by logic and not by running. You ran the mutations;
  say in the report which program each one reddens, so the necessity is a measurement.

## 6. `TOSTRING`, left loud beside `MAKESTRING`

Same C++ function and arity (`Setup.cpp:733`-`:734`). Either implement it beside `MAKESTRING` -- if it
is the same function, say so and measure it -- or record why it stays refused. What is not acceptable
is the pair being split with no reason on record.

## Verification this round owes

* The five gate commands, each status read **unpiped**.
* For item 2, the measurements as run, both engines.
* For item 5, the `#[inline(always)]` result and the per-mutation program names.
* No sitting unless you change code that `.text` reaches -- state which side of that predicate this
  round falls on. Items 2 and 6 could put you on the code side; if so, run one.
* Report section **before** the commit; send me the SHA.
