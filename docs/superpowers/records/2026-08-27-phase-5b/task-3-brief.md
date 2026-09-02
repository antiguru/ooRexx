# Task 3: per-object methods, their scopes, and the restricted-private check

**Goal.** `corpus/gate-tables/concepts/usesem.rex` agrees on both engines.

**BASE:** the commit named in your dispatch. Read `global-constraints.md` and the 5a constraints it
points at, `rust/CLAUDE.md`, the plan's Task 3 section, and D66 and D67 in the spec.

## Measured by the controller at `3290d5763` -- re-measure it, it is a claim and not a premise

All four of the plan's Task 3 claims still hold, which is worth saying because Task 2's brief had
three of four rot under it before dispatch. Cited by symbol rather than line, for the same reason.

**`usesem` itself**, oracle from a fresh empty directory, three descriptors read separately:

```
oracle            rc 0   setmethod one-off / not-shared 0 / enhanced enhanced / still-not-shared 0
crate, both       rc 120  rexx-exec: method "SETMETHOD" of class "Object" is not implemented (Phase 5)
```

**`[]=` exists for no receiver kind.** `/bin/grep -an '"\[\]="' rust/crates/rexx-exec/src/dispatch.rs`
matches nothing -- that is the pattern, recorded beside the claim so its width is visible. The
`StringTable` entries in the native table are `[]`, `AT`, `PUT` and `UNKNOWN`, with no setter.

**The syntax parses, so this is a build item and not a parser one.** `a = (7,8,9)` then `a[2] = 5`:

```
oracle            rc 0    8 / 5
crate, both       rc 120  8, then method "[]=" of class "Array" is not implemented (Phase 5)
```

The read prints first on both sides, which is what says the parse is fine and the send is missing.

**`compile_method_source` exists** in `rexx-exec/src/dispatch.rs` and already turns a source string
into a method object for `~define`. It is what the string and string-array forms need here.

## Build

`setMethod`, `unsetMethod`, `enhanced`, and the object's own scope. `usesem` also needs
`.StringTable~new`, `StringTable`'s `[]=`, and `Class~enhanced`.

**The scope argument is D67 and is the part most easily got silently wrong.** `FLOAT`, the default,
is **one pool per object**, shared by all that object's `FLOAT` one-offs and separate from the
class's; `OBJECT` shares the class's. One `FLOAT` method cannot distinguish "one pool per object"
from "one pool per method" -- the spec measures the difference with two, and so must you.

**The restricted-private check is D66 and has two refusals, not one.** From a program context the
ordinary private check refuses first at dispatch:
`97.2 ... cannot accept private message "SETMETHOD" from this context.` at rc 159, with no method
frame on the traceback. `checkRestrictedMethod` (`ObjectClass.cpp:697`) refuses from a class method
of a class the object is not an instance of: `98.991 Method SETMETHOD may only be invoked from a
method of the same object or one of its classes.` at rc 158, **with** a
`Compiled method "SETMETHOD" with scope "Object".` frame. **The frame is the discriminator**, not the
number. `send` and `sendWith` are not in the trio and answer from a program context.

**The per-object first step of the search order is this task's too, and `usesem` cannot see it.**
`usesem` defines `EXTRA` only in the object's scope, so an implementation searching the class first
and the object second prints byte-identical output. The spec's discriminating program shadows a class
method of the same name and carries `unsetMethod` with it.

## Done when

* `usesem` agrees on both engines under
  `REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast`
  (which exits 101 by design while other 5b rows are red), and its probe path is in
  `corpus/phase-5b.txt` and `EXPECTED_SUBSET_5B` in the same commit;
* `corpus/phase-5b.txt` carries the precedence program, the two `FLOAT` methods sharing a pool with a
  second instance seeing an uninitialised name, and **both** restricted-private refusals;
* **two controls are recorded as run**: `usesem`'s own committed control (put a `setMethod`
  definition in the class's dictionary so it reaches the class's other instances) reddens `usesem`;
  and searching the class before the object reddens the new precedence program **while every gate row
  that agreed before the mutation still agrees after it** -- phrase that against the rows that were
  green before, by name, not against "every gate row", since two 5b rows are red for unrelated
  reasons. That second control is re-run at Task 10.

## Hazards this phase keeps hitting, named so you can defeat them

* **A witness that cannot fail** has appeared four times here. For every row you add, delete the
  behaviour it witnesses and confirm the row reddens, in both directions, with transcripts.
* **A witness that passes for the wrong reason** -- the harder version, found in Task 5's second fix
  round. A candidate row there would have agreed with the oracle while its comment was false about
  why, because the oracle never reached the state the row described. When your row agrees, check that
  it agrees *for the reason you think*: construct the control that distinguishes your explanation
  from the nearest wrong one.
* **"Can fail" is not "adds coverage."** Run your mutation against the corpus *without* the new
  program first. If an existing row catches it, say so and say what yours adds.
* **Probe past the row.** This task changes *the send*, which every corpus program uses. Look for
  shapes no row covers: `setMethod` on an instance whose class later gains the same method, a
  one-off shadowing an inherited method, `unsetMethod` for a name never set, and `enhanced` with a
  scope that collides.
* **No comment states the size of a set**, and no comment states a mutable in-repo aggregate. ASCII
  only, no em-dashes, no historical framing.

## Gates and records

All five gates from `rust/`, each status read unpiped from its own file, **never chained with `&&`**
(a chain reports only the last command's exit), plus the phase-gate command above. The three hot
sweeps are parallelized now: gate 4 is about three minutes and gate 5 about ten.

If your change touches `src/`, a performance sitting is owed. The pin is
`bench-baselines/pinned/rexx-run-f558ea501` and the baseline file is
`bench-baselines/phase-5b-arms.tsv`, which is empty apart from its header. **`rexx-arms` appends to
whatever `--baseline` names**, so do not point it at `phase-5a-arms.tsv` or `phase-5b-drift.tsv`.
`rexxcps` is not in `bench-programs/`; it needs a path axis.

Write your report to `task-3-report.md`, file first and append as you go. Message the controller when
you finish.
