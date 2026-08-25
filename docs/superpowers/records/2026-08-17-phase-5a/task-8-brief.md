## Task 8: `::CLASS ... METACLASS`, and the rest of `::CLASS`'s option surface

**Goal.** `METACLASS` installs, which is D44's only program-level witness, and `::CLASS`'s remaining
options are pinned rather than incidentally green.

**Build.** The metaclass edge and the merge position `dire.xml` `clasdi` gives it;
`updateSubClasses` rebuilding both behaviours where `updateInstanceSubClasses` rebuilds one; the
class-behaviour side of the cascade.

**Verification, runnable now -- and the program is pinned, not described.** Measured:

```rexx
say .K~classSideHi
::CLASS S MIXINCLASS Class
::METHOD classSideHi
  return "class-side hi"
::CLASS K METACLASS S
```

oracle rc 0, `class-side hi`. **`S`'s method must NOT carry `CLASS`.** A metaclass donates its
*instance* methods to the class side of the classes using it; with `::METHOD classSideHi CLASS` the
oracle answers **rc 159, `97.1`**, measured -- which is how D44's row reads if taken as prose, and how
I wrote it the first time. The `~inherit` route is not an alternative: `.K~inherit(.S)` is rc 158,
`98.943`.

Plus the options that **already agree** and must keep agreeing, each with a table D row and a probe:
`::CLASS K PRIVATE` and `::CLASS K ABSTRACT` are rc 0 with identical stdout on both sides today,
measured. Their *effects* are not all reachable here: `PRIVATE` versus `PUBLIC` on a class is
observable only across a package boundary, which is 5c's -- say that rather than implying the row is
covered.

**What it changes about refusals.** After this task the `::CLASS` refusal message must name nothing,
because nothing is left refused; the arm is deleted rather than narrowed, and the in-crate table that
held it loses its rows in the same commit.

**Done when** the metaclass witness matches byte for byte on both engines, D44's class-behaviour
question has an answer in the tree, and a negative control rebuilding only the instance behaviour
reddens it. Sitting required.

---

