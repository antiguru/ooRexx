## Task 15: `::METHOD` and `::ATTRIBUTE`'s option surface

**Goal.** Close the two directives' option surfaces, minus `DELEGATE` (5b's, because `dire.xml`
defines it *as* `expose` plus `forward to()` and `FORWARD` is 5b's).

**What is actually missing, measured, because most of this surface already agrees.** `PUBLIC`,
`PACKAGE`, `GUARDED`, `UNGUARDED`, `PROTECTED` and `UNPROTECTED` on a class method are all rc 0 with
identical stdout on both sides today. Three things are not:

* **`::METHOD a CLASS ATTRIBUTE`** generates a getter and a setter: `.K~a = 5` then `say .K~a` is
  oracle rc 0 `5`, crate **rc 159 `97.1`**.
* **`::ATTRIBUTE b CLASS`**'s generated accessors: oracle rc 0 `7`, crate **rc 120**. `GET`/`SET` may
  each carry a body overriding the generated one, which the old plan's Task 7 already runs.
* **`ABSTRACT` enforcement on a method.** Measured, `::METHOD m CLASS ABSTRACT` **installs at rc 0 on
  both sides** and is refused only when sent: oracle rc 163,
  `93.965 Method M is ABSTRACT and cannot be directly invoked.`, crate rc 120. **This is the
  abstract-method half of what was one 5b cell, and it is reachable with no instance** -- ruled, the
  cell corrected, and both halves owned: the method arm here, the class arm (a check inside `~new`)
  in 5b.

**Verification, runnable now.** Oracle-differential per option on both engines, one probe per table D
row, including the six that already agree -- because the risk in this task is not the missing three,
it is that closing them changes an agreeing row. `UNGUARDED` in particular **agrees and has no
Phase-5-observable effect in the reachable shape**; keep its row and say that, rather than treating
green as coverage.

**What it changes about refusals.** The `::ATTRIBUTE` generated-accessor refusal and the abstract-send
refusal both disappear. Neither is expressible as a corpus row today, so the instruments that replace
them are the two table rows plus the in-crate refusal tests, and the task states that the replacement
can fail the same way the deleted rows could.

**Done when** the three live rows agree byte for byte on both engines, the six agreeing rows still
agree, and a control that generates a getter without a setter reddens the attribute row. Sitting
required.

---

