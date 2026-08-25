# Task 8, fix round 1

**Spec compliant, no behavioural defect.** The reviewer re-measured rather than trusting the report:
all nine committed programs match the oracle byte for byte on both engines, plus fourteen shapes it
built itself. Risk by risk it reproduced the merge ordering including the `hasScope` guard and all
four uncovered shapes; verified the metaclass override against `ClassClass.cpp:1586`-`:1591` and the
oracle; confirmed the refusal order in both source orders and against three neighbours; confirmed the
regression progression; and confirmed the deletion reasoning holds under its own analysis.

Two findings from it are worth having in your report as facts rather than as verdicts. The amended
frame rule is discriminated by exactly the pair you predicted: **`.Object~subclass('X', .Object)` is
`99.927` *with* the `Compiled method "SUBCLASS"` frame** -- the same body, sent instead of called. And
its own pass independently caught three of the four citations before `8fb97527c` landed, from the C++
itself; it missed `inherit_mixin`'s `:224` only because that hunk is outside the reviewed diff.

## 1. This is a latent bug, not a doc defect, and it is fixed in the code

`rexx-classes/src/registry.rs:180`-`:190`. `class_of`'s doc says the oracle's `~class` and `~metaClass`
coincide for a class object, because `setOwningClass`'s argument "is always its metaclass"
(`ClassClass.cpp:1615`). Measured, they do not coincide: for `::CLASS T SUBCLASS S METACLASS M1` with
`S` a metaclass, **`.T~metaClass~id` is `S` and `.T~class~id` is `M1`**. `:1615` passes the *named*
metaclass; `:1590` -- the override this task implements as `define_class`'s -- has already moved the
`metaClass` field to the superclass.

**Both accessors read one field, so Task 9's `~class` answers wrongly the moment it is written.**

**Ruling: split the field and pin it with an in-crate test. Do not fix only the sentence.** A
corrected doc plus a note for Task 9 is precisely the arrangement this plan has watched fail: the same
defect shipped three times here, the third from a dispatch that warned about it, and only the
type-level fix held. Nothing differential can witness this today -- every observer needs `~class`,
which is Task 9's -- so the instrument is an in-crate test asserting both values on that shape, and it
should fail if the two fields are collapsed back into one.

If splitting turns out to need something you judge outside this task, stop and say so precisely rather
than half-doing it; that report I will take.

## 2. A false sentence in a tracked plan document

`docs/superpowers/plans/phase-4-exclusions.txt:604`-`:605` says "`.Class` is then the metaclass
whatever the directive said". False in general, by the same measurement: **the superclass becomes the
metaclass.** Your crate's own doc has this right, which is what makes the tracked file the one that
misleads.

## 3. A set size, in the diff that struck the same shape elsewhere

`corpus/phase-5a.txt:254`-`:257`, "The two 97.1 programs are the boundaries..." -- names a size and
then enumerates. Nothing enforces the count. Delete the enumeration; say what puts a program in that
set.

## Minors

* `lib.rs:3733` cites `ClassDirective.cpp:189`/`:223`, the null tests, against `:180`, a
  `reportException`. The reportExceptions are `:191`/`:225`. **Fifth wrong citation on this plan**, and
  not among the four `8fb97527c` fixed.
* The report's `ir_dual` reason is wrong. No `ir_dual_cases` stanza contains `::class` or `::method` at
  all, and `message-sends` sends only to primitives. The reviewer accepts the conclusion for a reason
  you did not give: the table D probe rewrite plus `run_on_both_engines`' unconditional
  engine-agreement assertion is the real instrument, since `corpus.rs` runs IR only. Take that reason
  if you agree with it after checking it.
* "28 commits" is the count at your base; 29 at the sitting, 30 at head. State when it was taken.
* "`arith` ... to six decimals": `ir small` is `1.001283` here against `1.001284` in Task 7. That cell
  is bimodal across `[1.001283..1.001284]` and has been in three sittings now -- say that rather than
  claiming an identity it does not have.
* `corpus/gate-tables/directives/class__abstract__subkeyword.rex` is still a `say 'main'` probe that
  any `ABSTRACT`-ignoring build passes. Your report says this for `PRIVATE`/`PUBLIC` and not for it.
  Say it, and say where enforcement actually lands.
* The brief's witness is pinned as a superset -- `class_metaclass.rex:27` adds `ABSTRACT` to `K`. Note
  it, so nobody reads the committed program as the brief's program.

## The sitting question the reviewer could not answer

**Ruling: `121bc720d` owes no sitting, and prove it rather than assert it.** It is a doc comment in
`src/`, so the release binary should be byte-identical. Build `rexx-run` at `121bc720d~1` and at
`121bc720d` and compare the two sha256 sums. If they match, record the pair and the method; a
byte-identical binary cannot move any axis, which is a stronger statement than "comments do not
matter". If they differ, that is a finding and you should stop and tell me.

## Verification this round owes

* The five gate commands, each status read **unpiped**.
* Item 1's in-crate test shown failing with the fields collapsed and passing split.
* The two sha256 sums for the sitting question.
* Write the report section **before** you commit, and send me the SHA when it lands. I will wait for it.
