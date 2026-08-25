## Task 18: the three install passes, and class-object initialization

**Goal.** M8. Install is three passes over a dependency-ordered class list -- **install all, resolve
constants, activate all in construction order** -- and a class object is initialized `INIT`, then the
`INHERIT` merge, then `ACTIVATE`.

**Why the pair and not either half.** `fundclasses.xml` explains `ACTIVATE` by contrast with `INIT`
("Because the INIT method is called early in the class construction process, only limited class
initialization is possible at that time"), and the contrast is the specification. The old cut put
`ACTIVATE` in 5a and class `INIT` in 5b, which is a mechanism split nobody had written down.

**Verification, runnable now -- and the spec's own program is not.** The spec pins the discriminator
as `::METHOD init CLASS` containing `forward class (super) continue`. **`FORWARD` is 5b's**, so that
program cannot be a 5a differential. Measured, `self~init:super` -- a scope-override send, which the
old plan's Task 5 landed -- reproduces the transcript exactly:

```rexx
say "prologue"
::CLASS M MIXINCLASS Object
::METHOD mm CLASS                      -- CLASS-side, and this is load-bearing
  return 1
::CLASS K INHERIT M
::METHOD init CLASS
  self~init:super
  say "K init,     hasMethod MM =" self~hasMethod("MM")
::METHOD activate CLASS
  say "K activate, hasMethod MM =" self~hasMethod("MM")
```

oracle rc 0: `K init, hasMethod MM = 0`, `K activate, hasMethod MM = 1`, `prologue`. **`INIT` fires
before the `INHERIT` merge and `ACTIVATE` after it; the discriminator is the pair, and a phase that
owns one and not the other owns neither.** `M`'s method must carry `CLASS`: `self` in a class-side
`init`/`activate` is the class object, so with a plain `::METHOD mm` both lines read `0` and the
transcript discriminates nothing. This task needs Task 7's `MIXINCLASS`/`INHERIT`, which is why it
sits here.

**The second discriminator, for the passes rather than the initialization, measured by me:**

```rexx
say .A~c
::CLASS A
::CONSTANT c (.B~m)
::CLASS B
::METHOD m CLASS
  return "from B"
```

oracle rc 0 `from B`; **crate rc 159, `97.1 Object "The B class" does not understand message "M"`**.
A constant expression reaching a class declared later resolves on the oracle because constants are a
**second pass over an already-installed list**, and fails here because they are not. It needs no
bootstrap, which is what the old plan's Task 9 verification got wrong.

Also: `.TraceObject~option` is `N` and not `OPTION` on the shipped image, which separates an activate
that ran from one that did not -- but that reading is **after a bootstrap** and therefore Task 23's,
not this task's.

**Done when** both discriminators match byte for byte on both engines, and two controls are recorded:
firing `ACTIVATE` before the merge reddens the first, and resolving constants inside the class pass
reddens the second. Sitting required.

---

