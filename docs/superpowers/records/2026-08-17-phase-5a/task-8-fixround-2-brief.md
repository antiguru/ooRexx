# Task 8, fix round 2 -- prose only

**Code and instrument accepted.** The reviewer attacked the field split with 18 shapes on the oracle
and then read the same shapes out of `ClassGraph` in a copied tree: **no divergence**. That includes a
metaclass derived from a metaclass, `METACLASS` naming the superclass, a middle-generation metaclass, a
metaclass declared later in the file, a mixin with a metaclass, a grandparent on the class side, and
`~metaClass`/`~class` on `.Object`, `.Class`, `.String` and `.Method`. It also established that
`newRexx`'s `isPrimitiveClass()` branch cannot bite us, because `.Class` is the only primitive carrying
`META_CLASS`.

**The test is real**: plain `cargo test`, no gate variable, both collapses failing at the two different
assertions with `running 1 test` and `Compiling rexx-classes` printed each time. **And every C++
citation new in this diff checks out** -- the first round on this plan with no wrong citation.

Change no code this round.

## N1. The oracle's write order is stated backwards, in six places

`:1590` runs **before** `:1615`, not after. The mechanism is that `:1590` writes a different location
and the local `meta_class` is never reassigned. Copies at `class_graph.rs:148`, `class_graph.rs:271`,
`registry.rs:192`, `lib.rs:5420`, `phase-4-exclusions.txt:609`, and repeated in the report's FR1.1.

**My brief had this right and the round inverted it**, so read the C++ rather than either text: print
`:1585`-`:1620` and write what the lines do in the order they run.

`8a88dc63d`'s commit message carries a copy that cannot be edited. Record that in the report beside the
correction, as this plan does with the others.

## N2. The general rule is mine, and it is false

I told you to lift my phrasing -- *"the split is a property of deriving from a metaclass, and naming
`METACLASS` only chooses which value the `~class` side holds"* -- and it does not survive. **Necessity
holds; sufficiency does not.** Measured counterexamples, each deriving from a metaclass and each
coinciding: `::class MC mixinclass class`, `::class Z subclass Class`, and
`::CLASS M3 SUBCLASS MC METACLASS MC`.

**The exact rule: they part iff the superclass is a metaclass *and is not* the named-or-inherited
metaclass.**

Copies at `registry.rs:190`, `class_graph.rs:147`, `class_graph.rs:158` ("wrong on every class derived
from a metaclass"), `lib.rs:5417`, and report FR1.1. **The counterexample was two lines above it in
your own report's table** -- `S metaclass Class  S class Class` -- and it is the first directive of the
committed test's own program. Both of us read past it.

This is the sentence Task 9 builds on, so state it in the exact form and put the counterexample beside
it, not the general shape.

## N2b, N3 and the rest

* **N2b.** `phase-4-exclusions.txt:604` still says a named `METACLASS` is discarded "altogether", while
  `:608`-`:610`, added this round, show it surviving as `~class`.
* **N3.** The sitting counts are false and I repeated them: not "fourth sitting for that cell" but
  **six**, and not "three earlier sittings recorded" but **five**. I checked the TSV myself: it holds
  eight sittings, and `arith ir small` has spanned `[1.001283..1.001284]` in six of them, from
  `c351fa473` onward. One copy is in `c9523023c`'s message and cannot be edited. **Say the span and
  the commits rather than a count** -- the count is what keeps going wrong, and the span is the claim
  anyone cares about.
* **N4.** `registry.rs:188` "every class it builds" -- `.Object` is built with `None`.
* **N5.** `lib.rs:5435` -- `~request` also reads `owningClass` (`ObjectClass.cpp:1916`), so `class_of`
  has a second oracle-side reader.
* **N6.** "no axis program installs a `::CLASS`" is false of `rexx-bench`'s `PROGRAMS`:
  `bench-programs/dispatch.rex:9`. Check whether it is true of the six **measured axes** and narrow the
  sentence to what you measured, naming the set you checked.
* **N7.** `8a88dc63d`'s message closes "No sitting... comments-only", which its own successor refutes.
  FR1.6 records the error but not that a copy landed in a commit message.
* **N8.** The report cites `lib.rs:5467`/`:5466`; the assertions are `:5469`/`:5468`.
* **N9.** `lib.rs:5442`'s "Either way" names and enumerates a set size.

## Verification this round owes

* The five gate commands, each status read **unpiped**.
* For N1 and N2, the C++ lines and the counterexample programs printed in the report, not summarised.
* No sitting: prose only, and the predicate says why -- but state which side of it this round falls on
  rather than asserting the conclusion.
* Report section **before** the commit; send me the SHA.
