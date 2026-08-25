## Task 13: the three access scopes, and the one chokepoint

**Goal.** `PRIVATE`, `PACKAGE` and `PROTECTED` are three separate limbs with three separate answers,
all passing through D45's single dispatch chokepoint (M3).

**Build.** `checkPrivate`'s limb keyed on the **caller's scope**; `checkPackage`'s keyed on the
**caller's package**; `PROTECTED` routed through the security-manager seam, which is what makes the
seam's placement checkable rather than nominal. The two access checks sit **inside** the chokepoint,
not beside it.

**Verification, and the three arms are not equally reachable -- measured, all three.**

* **`PRIVATE`: a live over-refusal, fully reachable.** `say .K~m` with `::METHOD m CLASS PRIVATE` is
  oracle rc 159, `97.2 Object "The K class" cannot accept private message "M" from this context.`
  against crate rc 120. Its adjacent success -- a `pub` method sending `self~m` from inside the class
  -- is what pins the rule to caller scope rather than to something coincidental. **That program is
  not green here yet and the measurement says which twin is:** with `PRIVATE` on `m` the crate
  refuses the whole file at rc 120 (`a PRIVATE ::METHOD is not implemented`), while the same program
  *without* the keyword is rc 0 `inner` on both sides today. So the private twin is this task's
  target and the plain twin is the neighbour that must stay green -- read as one sentence they look
  like the same claim. Also: a *sibling* class in the same package, which is outside the defining
  scope and must refuse -- measured, oracle rc 159 with the same `97.2` text, and its traceback is
  **two frames**, the `return .K~m` line before the `say .S~poke` line, which is the byte-for-byte
  target. And a subclass's method sending a superclass's private method: **measured, oracle rc 0
  `inner`.** A class-side `::METHOD poke CLASS` on `::CLASS Sub SUBCLASS Base` sending `self~m`,
  where `m` is `Base`'s `PRIVATE` class method, is answered rather than refused. So the limb this
  task builds admits more than the defining class, and a rule written as "only the defining scope"
  would redden this program. The refusal cases above are the sibling and the outside caller.
* **`PACKAGE`: the refusal arm is not reachable in this phase.** Measured, the discriminating program
  needs a second package, which needs `::REQUIRES`, which is 5c's: the oracle answers
  `97.3 ... cannot accept package scope message "M" from a different package caller` and the crate
  answers rc 120 on the directive. The same-package case agrees today and stays agreeing. **So the
  cross-package limb's only instrument here is an in-crate test**, said plainly, and 5c owes the
  differential.
* **`PROTECTED`: agrees today and the differential proves nothing.** Measured, rc 0 with identical
  stdout on both sides; the same holds for `PACKAGE`'s same-package arm, oracle rc 0 `pkg`. The instrument is the chokepoint assertion -- a test that counts dispatch
  chokepoints and fails if a second appears -- and the task states what that test counts and what
  would make it pass while a second dispatch path exists.

**Every measurement here is on a class method**, because reaching an instance method needs `~new`.
This task inherits that limit and restates it rather than implying the instance case is covered; it
joins the old plan's Task 7 debt against 5b.

**What it changes about refusals, and the instrument.** This is the task most likely to turn a clean
refusal into a wrong answer: a private send that the oracle raises on must not start answering. The
corpus gate cannot see that at all if our answer stops being rc 120. The instruments are table D's
`PRIVATE` row, the corpus programs above, and an in-crate test asserting the refusal shape by scope --
named, because "the corpus is green" would not have caught it.

`phase-4-exclusions.txt:3304`'s KNOWN GAP on `PRIVATE`, `GUARD` and `PROTECTED` loses its `PRIVATE`
and `PROTECTED` limbs in this task's own commit; `GUARD`'s goes in Task 16. **The line was cited as
`:3248` until this was checked**, which is the VALUE dot-variable entry; the file lives at
`docs/superpowers/plans/phase-4-exclusions.txt` and the entry's own text is the thing to search for.
Each limb of it states what makes that option *currently safe*, which is the sentence that stops being
true, so the limbs come out with the measurements that replace them rather than being deleted.

**Done when** both `PRIVATE` programs match byte for byte on both engines, the chokepoint test names
what it counts, and a negative control allowing every private send reddens the outside-scope case.
Sitting required.

---

