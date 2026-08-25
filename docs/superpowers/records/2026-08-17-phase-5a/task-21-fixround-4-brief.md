# Task 21, fix round 4

Re-review: `.superpowers/sdd/2026-08-17-phase-5a/task-21-rereview-3.md`. Every item of round 3 was
done and both instruments are real. What this round fixes is **three false sentences the correction
round itself added**, one in each file whose prose it rewrote, plus two blemishes and one instrument
that is narrower than it claims.

**Ruling on escalation.** The process says round 4 gets a fresh implementer on a more capable model.
You are already on the most capable model available, and this round is corrections to prose you wrote
with the measurements still in hand, so a fresh agent would re-derive context to no benefit. You keep
it. Cost if wrong: a defect you are blind to survives, which is what the re-review is for.

## The three false sentences

**F1. `environment.rs:1021`-`:1022` -- the exhibit does not reproduce.** I ran it:

    say (-140404878001713 = -140404878167489) (-140404878001713 == -140404878167489)   ->  1 1
    say -140404878001713                                                               ->  -1.40404878E+14
    a = "-140404878001713"; b = "-140404878167489"; say (a = b) (a == b)                ->  1 0

Unary minus is arithmetic, so each literal is evaluated at `NUMERIC DIGITS 9` and both become the
same string before `==` sees them. **The paragraph's argument is right and its exhibit is wrong**, and
the exhibit is the half a reader checks. The values arrive from `~identityHash` as strings in every
real use. Use the variable form, or `corpus/lang/class_context_package.rex`'s shape, which makes the
same argument with no literals.

**F2. `dispatch_seam.rs:185`-`:188` -- "where no gate here can see it" is false of its own exhibit.**
`class_context_gc.rex`, which this task added one round earlier, is exactly such a gate, and fix
round 2 measured it red under precisely that door. `class_context_reply.rex` is a second. The true
claim is narrower: a door **no corpus program reaches**. This is the falsified-neighbour pattern with
the arrow reversed -- the row added to close the hole falsifies the sentence written beside it.

**F3. `lib.rs:376` -- one universal replaced by another.** "Non-zero under an ordinary `run_program`"
is false in the opposite direction to the "Always `0`" it replaced: two statements in the tree say
most programs collect zero times, one of them thirty lines above it. "**Can be non-zero**" is the
fix. The edit also left a stranded `What` at the end of `lib.rs:379` -- rewrap.

## Blemishes

* **`phase-5a.txt` opens with a set cardinality** -- "Three rows here are the context object's own
  identity" -- added by the same round that was fixing exactly that shape, and a fourth context row
  falsifies it. Name the set. In the same sentence, "one per state the object has to survive in"
  does not map onto what follows: row 1 is not a state and row 2 covers two.
* **`coverage.rs:1144` names the weaker half.** It calls `class_context_gc.rex` "surviving a forced
  collection while running", where your own `phase-5a.txt` clause says "running **or suspended**" and
  the row's header calls the suspended half the one the single-activation rows cannot reach.

## The instrument is narrower than its doc claims

`heap_collect_is_called_from_collect_now_alone` pins the literal text `heap.collect(&`, so a real
bypass can pass it green with the sweep gone. The reviewer named concrete ones:
`self.heap.collect(roots)` where `roots` is already a `&RootSet` (no `&`), UFCS
`Heap::collect(&mut self.heap, &self.roots)`, a rebinding through `let h = &mut self.heap;`, a call
rustfmt wraps across lines (the walker scans line by line), and a collection introduced inside
`rexx-core` itself, which the walker never reads.

**Take the file's own precedent**: `dispatch_seam.rs`'s module doc already carries a "what that
leaves it unable to see, stated rather than argued away" list. The new test's doc has no such list
while claiming the needle names the whole set. Add the list, or widen the needle and accept extra
matches. Do not leave the doc claiming a class when the test pins a spelling.

## One number that should not be a number

`class_context_reply.rex`'s comment records that the loud variant "gave two distinct outputs over
twenty oracle runs". The reviewer reconstructed that variant and measured **three** over twenty
(12/6/2). Neither number is wrong; a count of distinct outputs over twenty samples of a race is not a
stable quantity to write down. Say the race is real and reproducible, keep the twenty runs as the
method, drop the count.

## Not changing

The past-tense clauses in `class_context_reply.rex` and `phase-5a.txt` ("it had no differential row")
stay. Corpus headers carry round stamps by convention here and both claims are true of their time.
Ruling recorded so the next reviewer does not re-raise it.

## How to close

All five gates. If you change `src/`, a sitting is owed. Invert any assertion you widen.
`cp` before mutating, restore from the copy, never `git checkout --`. Append to the report.
