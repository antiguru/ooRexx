# Task 20: what the controller measured before dispatch

Measured at BASE `7130b222e`, fresh directory per batch, absolute paths, three descriptors read
separately, both sides bounded, `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`.

**The brief's ownership split rests on two premises. Both are false now. Read this before planning.**

## Premise 1: "a populated `.ROUTINES` is 5c's, so the ROUTINE readback is 5c's"

Task 17 populated all three package tables -- an accepted scope widening, because the receiver test
is by class and leaving them empty produced silent rc-0 wrong answers. Measured today:

    say .routines["R"] / say .routines~r    with ::ROUTINE r
      oracle rc 0  `a Routine` twice        crate BOTH ENGINES byte-identical

So `.ROUTINES` is populated here and both spellings of the route resolve. The reason the brief gives
for deferring the `ROUTINE` readback no longer holds.

## Premise 2: the METHOD, ATTRIBUTE and CONSTANT readbacks "ride Task 9's `~method`"

They need more than `~method`. `~method` hands back a Method object; reading an annotation off it
sends that object a message, and **no Method or Routine object can receive any message here**:

    say .K~method("M")~class    oracle rc 0 `The Method class`   crate rc 120 both engines
    say .routines["R"]~class    oracle rc 0 `The Routine class`  crate rc 120 both engines
      rexx-exec: a message send to one of the interpreter's own objects is not implemented (Phase 5)

That is Task 17's parked gap, which it declined as a design call. **It blocks four of the six
readbacks, not one**: METHOD, ATTRIBUTE, CONSTANT and ROUTINE all send to one of those objects.
PACKAGE is blocked separately and is Task 21's. Only the `CLASS` readback sends to a class object,
which works.

## What is measured, and what each readback needs

| target | oracle readback | reachable here? |
| --- | --- | --- |
| `CLASS` | `say .K~annotation("AUTHOR")` -> rc 0 `moritz` | **yes**, a class-object send |
| `METHOD` | `.K~method("M")~annotation("AUTHOR")` -> rc 0 `moritz`, with an *instance* method | needs a Method object to receive a message |
| `ATTRIBUTE` | same shape through `~method("A")` -> rc 0 `moritz` | same |
| `CONSTANT` | same shape through `~method("C")` -> rc 0 `moritz` | same |
| `ROUTINE` | `.routines["R"]~annotation("AUTHOR")` and `.routines~r~annotation(...)`, both rc 0 `moritz` | route resolves; needs a Routine object to receive a message |
| `PACKAGE` | `.context~package~annotation("AUTHOR")` -> rc 0 `moritz` | Task 21's, blocked at the send |

`say .K~annotations` is oracle rc 0 `a StringTable`, a class-object send, so it is reachable here.

**Note the METHOD shape**: `~method` reads the class's own *instance* dictionary, so the annotated
member must be an instance method. With `::METHOD m CLASS` the readback is rc 159 with the
`Compiled method "METHOD" with scope "Class".` frame, which is Task 19's measured contrast and not a
readback at all.

**The install side is uniformly unbuilt.** Every target, and the unknown-target case, is
`rexx-exec: ::ANNOTATE naming a target is not implemented (Phase 5)` at rc 120 today. The brief's
"the unknown-target case staying at 99.945 rc 157 byte for byte" is the target state, not the
current one: today the blanket refusal swallows it before the name is checked. Oracle for
`::ANNOTATE ROUTINE nosuchrtn` is rc 157.

## Ruling

**Install all six targets. Deliver `~annotation` and `~annotations` where the receiver already
works** -- the class object -- and make the unknown-target case the oracle's 99.945 rc 157.

**For the four blocked readbacks: measure the cost first, then decide, and report which way it
went.** The question is narrow: can `~annotation`/`~annotations` be answered on Method and Routine
objects by the same mechanism that already answers `.methods~z` and `.routines~r`? **If that is
contained, build it** -- it turns one readback into five and closes a gap that is otherwise a silent
hole in this task's own deliverable. **If it needs a general instance-receiver mechanism, stop and
say so** -- that is Phase 5b's and I will park these readbacks with 5b named as owner, beside the
`PACKAGE` one that is Task 21's. Do not build a general receiver arm on your own initiative.

Either way the brief's stated reasons must not survive: the `ROUTINE` readback is not deferred
because `.ROUTINES` is empty, and the member readbacks do not follow from `~method` alone.
