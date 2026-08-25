# Task 10, fix round 1

Three Important. Risks 3, 4 and 8 came back verified, including that mutation 3's reasoning holds and
that the amended roadmap sentence is true of this tree. The selector work is met at the layer D24 asks
for and its identity is **contained** -- no `Selector` or `Selector::same` anywhere in `rexx-exec` or
`rexx-classes` -- and the per-parse pool matches the oracle for the right reason
(`LanguageParser.cpp:788`-`:793`, `RexxMemory.cpp:164`).

## 1. `Entered::Label` carries the wrong receiver, and the comment says it is right

`run.rs:5437`-`:5443`. `RexxActivation::internalCall` passes **the caller's receiver**
(`execution/RexxActivation.cpp:3313`, assigned at `:474`); only `internalCallTrap` passes `OREF_NULL`
(`:3343`). Measured on the oracle: `CALL inner` inside a class method reaching `self~priv` is **rc 0**;
the same send from a `CALL ON ERROR` handler is **rc 159 / 97.2**; from the top level, rc 159 / 97.2 as
the control.

So an internal `CALL` inherits the receiver and a trap does not. Split them, and **fix the comment
first** -- it currently asserts the wrong value is correct, which is worse than the value, because
Task 13 reads this field and will read the comment before the code.

Latent today. That is not a reason to defer it: the whole point of landing this signature one task
early was to stop Tasks 12 and 13 rewriting each other, and a field that is wrong when they arrive
defeats that.

## 2. Two `None`s with different meanings, and the guard cannot fail

`dispatch.rs:381`-`:398` and `:648`-`:658`. `Caller::package`'s `None` means "no activation";
`Interp::package_objects`' `None` key means "the `REXX` package" (`lib.rs:2354`, `environment.rs:627`
and `:669`). Same type, opposite meanings, and the only guard is a `debug_assert` **no in-tree path can
falsify** -- `Interp::caller` cannot produce Some-receiver-with-None-package, and `no_caller` is
None/None.

**Ruling: make "no activation" a variant.** An enum, both sides. That is the type-level fix, and this
plan's record on the alternative is unambiguous: the same defect shipped three times when a comment or
a dispatch note was asked to carry an invariant, the third time from a dispatch warning about it.

While you are there: the reviewer judges the `debug_assert`'s actual job to be keeping the accessor out
of `dead_code`. If that is what it is for, say so or find another way to keep the field alive -- an
assertion that cannot fail, standing where a reader expects a check, is worse than no assertion.

## 3. The send-path figure was achievable, and the reviewer took it

Plan-mandated, and the finding is against the brief as much as against you. The brief called the sitting
this task's real acceptance; the sitting cannot see the send path. **A send-exercising measurement was
available the whole time**, and the reviewer ran it -- interleaved, three rounds, `instructions:u`:

* an `s~length` loop: **1.00155** ir, **1.00153** tw;
* a `.K~m` class-method loop: **1.00090** ir, **1.00119** tw.

Green. **Take these numbers into the report as this task's own**, re-run them yourself so they are
yours rather than quoted, and say what they do and do not cover. They are not axis rows and they do not
consume the 1.695467-per-pass budget; they are the evidence that the path this task reshaped did not
slow, which the axis sitting cannot give.

## Minors, and three of them are false statements I repeated

* **"Smaller than the pinned build's own run-to-run spread" is false**: 0.000180 against 0.000092. I
  repeated it upward. Say what the two numbers are.
* `expr.rs:867`/`:933` are off by one -- the constructors are at `:866`/`:932`. I repeated these too.
* **"All four `~`" is a `grep -c` line count**: 8 occurrences on 4 lines, all in the comment. Say
  occurrences or say lines, and say which.
* **The `SmallInt` arm is a pure no-op today** -- every consumer folds it (`dispatch.rs:618`, `:1366`),
  and the oracle probe shows a small integer indistinguishable from the equivalent string across
  `~class~id`, class identity, `~length`, `~isA`, `~hasMethod`, `~reverse`, arithmetic, `=`/`==`, with
  both behaviours enumerating the same methods. **That is fine** -- D24 asks for the arm structurally --
  but the rationale comment claims more than the arm does. `dispatch.rs:288`-`:303`'s "the route rather
  than the answer" was already true of the base arm, so the split buys neither.
* Gate 5's extra test is `rexx-core/src/bytes.rs:320`'s pre-existing `cfg(debug_assertions)` test, not a
  `debug_assert`.
* Roadmap `:494`'s "nothing reads it" is false -- `rexx-classes/tests/behaviour_wiring.rs:807`-`:825`
  reads it. That is a **tracked** file, so it is the one minor here that outranks its severity.
* `dispatch.rs:840`'s `.expect` is unreachable and `receiver` stays in scope, so the stated
  "by construction" property is not enforced.
* The spec at `:1179` still quotes the pre-amendment roadmap text.

## Verification this round owes

* The five gate commands, each status read **unpiped**.
* For item 1, the three oracle measurements re-run yourself, and a test that fails when a trap and an
  internal call are given the same receiver.
* For item 2, the enum, with whatever kept the field alive stated plainly.
* For item 3, your own interleaved run of both probes.
* A sitting only if `.text` changes -- state which side of the predicate you land on, with the two
  sha256 sums if you claim it does not.
* Report section **before** the commit; send me the SHA.
