STATUS: DONE

# Task 6 review: the resolution plan

Reviewing `ca005497`, "Resolve a body's variables once, keyed by name, cached on
Interp". Verified from a clean export of the commit, not the working tree.

Recovered after a session died mid-review; every finding below was re-verified
against a fresh export rather than carried over on trust.

## Verdicts

**Spec compliance: PASS with two Important gaps.** Everything the task moves is
correct and `slot_of`'s three-source resolution is exactly D16's. But the
upfront pass does not visit the two constructs D16 singles out, so for any body
using them the plan is empty and every name is created by run-time growth —
which is the algorithm D16 explicitly contrasts itself against.

**Code quality: PASS.** Clean extraction, borrows handled deliberately, comments
that carry reasoning. The coverage gap below is about what the tests reach, not
about how the code reads.

**2 Important, 2 Minor. Nothing Critical**, because no program produces wrong
output today: the fallback path yields the same slots, just lazily.

Baseline: 20 unit tests in `rexx-exec`, all passing on the clean export.

---

## I1. The upfront pass never visits `Stem` or `Compound`, so for those bodies the plan is empty

`Plan::note` matches `Variable`, `Prefix`, `Binary`, then `_ => {}`.
`Plan::build` matches `Assignment`, `Say`, `Interpret`, then `_ => {}`. Neither
`ExprKind::Stem` nor `ExprKind::Compound` is reached, and neither is
`InstructionKind::Do`.

Measured, by printing what `Plan::build` returns for five bodies:

```
PLAN "say a.b"                    -> len 0  names []
PLAN "a.1 = 'x'"                  -> len 0  names []
PLAN "q. = 1"                     -> len 0  names []
PLAN "say v"                      -> len 1  names ["V"]
PLAN "do i = 1 to 3\nsay i\nend"  -> len 1  names ["I"]
```

The last row is not the counter-example it looks like: `I` is there because of
the `say i` inside the loop, not because the control variable was noted.

**Why this is a spec finding rather than an optimisation note.** D16 says the
plan is "built by one upfront pass, at first execution ... it is not populated
lazily one name at a time", and says so because "a lazy design threads a 'seen
this name?' check through every site that touches a variable". For a body
containing a stem or a compound — which is most real Rexx — that is what
happens: the plan contributes nothing and `Interp::slot_of` falls through to
`extra` and `grow_slots`, one name at a time, on first touch. Growth is
supposed to be the exception; here it is the only path.

Two second-order consequences worth stating:

* The performance argument that motivates the whole design is unrealised
  exactly where it was measured. D16 opens with "8.1% of runtime on the
  realistic mixed benchmark and **32.2% on stem-heavy code**" — and stem-heavy
  code is precisely the case that now gets no plan at all.
* `push_slots(plan.len())` allocates zero slots for those bodies, so D16's "a
  frame starts at the plan's length rather than being exactly it" is vacuously
  true rather than descriptive.

**Both `_ => {}` arms carry Task 3's own promises that Task 6 would close
them**, moved across verbatim: "Task 6 makes this pass exhaustive over
`InstructionKind`, which is the point at which the omission would start to
matter" and "Task 6 covers `Stem`, `Compound` and the rest, where D16's rule
that a tail piece lands on the *same* slot as a same-named variable is what
`names` exists for." I wrote those in the spike as a handover; they are now
comments in Task 6 promising Task 6 will do something.

**On the dispatch's third pressure point:** the tail-piece property *does* hold.
`b = 2; say a.b` resolves the piece by reading variable `B` through
`read_by_name` → `slot_of`, and a plain `B` elsewhere goes through the same
name-keyed `slot_of`, so they share a slot and `A.2` is produced. It holds
because resolution is name-keyed, not because the plan arranged it. Nobody gets
`A.B`.

## I2. The upfront pass is pinned by no test at all

Defeated the mechanism: made `Plan::build` return `Plan::default()`
unconditionally, deleting the entire pass.

```
test plan::tests::a_fragments_plan_resolves_against_the_enclosing_frame ... FAILED
test result: FAILED. 19 passed; 1 failed
```

**19 of 20 tests pass with the task's central deliverable removed**, and the one
that fails is about fragments rather than about the pass. `plan.rs` has four
tests; none constructs a `Plan::build` result and asserts its contents, and none
asserts a plan is non-empty for an ordinary body.

That is what let I1 through: the property the tests check is reachable by the
fallback, so an empty plan satisfies them. It also means a future regression in
the pass has nothing to fail against.

The cheapest fix is one test asserting `Plan::build` on a small body yields the
names that body mentions, which would have failed on `say a.b` today.

## m1. The accessor decision is recorded as open, and it closed two minutes earlier

`Plan`'s doc comment: "Recorded as an open `rexx-parse` amendment rather than
made here ... **Kept as a hash for now, pending that decision.**" It even
predicts the spelling — "exposing it as `SymbolId::index()` would cost nothing
new".

`SymbolId::index()` landed as `180875a9` at 22:01:56; this commit is 22:03:54,
and `git merge-base --is-ancestor 180875a9 ca005497` confirms it was in the tree
this was built on.

Keeping the `HashMap` is defensible and I would not argue with it. What needs
correcting is the comment describing the decision as pending on something that
already exists — a reader is told to go and make a change that has been made.

For the dispatch's question about `Option<usize>`: it does not arise, since
`by_symbol` stayed a `HashMap<SymbolId, usize>`. It arises the moment anyone
does swap, and the reason still holds — keywords, labels and constants share the
table, so a dense `Vec` has holes that must stay distinguishable from slot 0.

## m2. The `blocks` deferral is right; its stated reason is not

`Activation` carries `program`, `plan`, `extra`, `frame`, `pc`, `settings`, and
deliberately not `blocks: Vec<Block>`. The file says `Block`'s "only real
definition" comes from a task that has not run, and that "Task 11 adds the field
when it can give `Block` a real shape".

**`Block` is specified.** The spec's Control flow section, line 382: "Loop state
is a per-activation `Vec<Block>` holding the control variable's slot, the `to`,
`by` and `for` values, the iteration counter, the block's label and its `end`
index." That is a shape, not a blank.

And the asymmetry is not principled: **nothing reads `settings` either.**
Grepped — no use of `.settings` anywhere outside `activation.rs`. So the file
added one unused field and deferred another, on a reason that does not
distinguish them.

**My judgement on the question asked: yes, `Activation` is coherent without
`blocks`.** Nothing in the crate can execute a loop, so no path needs loop
state, and a field with no reader freezes a guess that its first real use would
otherwise shape. I would keep the deferral and fix the reason: not "Block is
undefined" but "no code reads it yet, and Task 11's first real use should pick
the representation".

---

## What I verified and found correct

* **`slot_of`'s three sources, in D16's order**, and nothing writes into the
  `Rc`: plan first, then `extra`, then `grow_slots` with the name recorded in
  `extra`. Both halves of `plan.slot_of(name).or_else(|| extra.get(name))` are
  reachable — the plan half by any plain variable the pass saw, the `extra` half
  by every stem or compound name and by `DROP (v)`.
* **The fragment plan** resolves a fragment's own ids against the enclosing
  frame through `slot_of`, and is returned rather than cached, which is what
  `BodyKey` having no fragment arm requires.
* **The extraction is faithful.** `Plan`, `BodyKey`, `ProgramId` and
  `Activation` moved out of the spike into `plan.rs` and `activation.rs` per
  the crate layout, without rewriting the shape Task 3 established.
* **The tail-piece property holds** (see I1), so no program gets `A.B`.

## Method

Clean export of `ca005497` via `git archive`; the working tree was not used, and
`7a628261`'s later `stem.rs` fix is therefore not in scope. Two experiments, both
in the export and discarded after: printing `Plan::build`'s output for five
bodies, and neutering `Plan::build` to measure what the suite notices. Nothing
in the repository was modified.

The CONCATENATION silent-pass figure is 56 strict rows, not 388 (`03c10606`);
noted because I had read the superseded version, and not relied on here.
