## Task 22: the native entry-point registry

**Goal.** D37. `::METHOD ... EXTERNAL 'LIBRARY REXX name'` resolves at install.

**It binds eagerly, and a failure stops the program before its prologue.** Measured by the old plan:
a file whose prologue is `say "prolog ran"` carrying one external method naming a missing entry point
exits **166** with **empty stdout** and `Error 90.998`; a real entry point exits 0 and prints.
Re-measure on a current build -- `3ee6e4bf7` changed when an `EXTERNAL` directive is refused.

**Build.** A registry in which every `LIBRARY REXX` name the three `.orx` files declare resolves; an
entry this phase does not implement raises **when invoked**, naming its owning phase.
`file_separator` and `file_path_separator` are implemented here as a stated scope addition from
Phase 7, because `StreamClasses.orx`'s constants call them at install. **`::ROUTINE EXTERNAL` naming a
real shared library stays Phase 7's** and its refusal is untouched -- say which of the three
`EXTERNAL` forms this task moves and which it does not, because the old plan's Task 8 fix rounds
produced a wrong answer in exactly that neighbourhood.

**Verification, runnable now.** Oracle-differential on the eager-bind failure, both engines. Then one
corpus program per registered family that **invokes** an unimplemented entry and pins the refusal --
writable because the old plan's Task 7 landed method bodies. This is a task whose subject is a
refusal: the instrument for "an unimplemented entry stops being loud" is the in-crate loud table plus
the invoking corpus programs, because a refusal at rc 120 that the oracle answers cannot be a corpus
row.

**Done when** all three `.orx` files install with no unresolved external, the eager-bind failure
matches byte for byte, invoking an unimplemented entry is loud and names its phase, and a control is
recorded: **binding lazily instead of eagerly** turns the missing-entry program from rc 166 with empty
stdout into a run that prints its prologue, so the program reddens. That control can fire here because
the program is green after this task. Sitting required.

---

