## Task 9: the Object and Class reflection protocol

**Goal.** `~class`, `~id`, `~superClass`, `~superClasses`, `~metaClass`, `~isA`, `~isSubclassOf`,
`~method`, `~package` and `~identityHash` answer. `~hasMethod` already does, and `~baseClass` landed
in Task 7 because a differential there needed it.

**Why here.** It is what makes table C's wiring half measurable at all -- measured, `.array~id` is rc
120 today and the first wiring question stops there -- and Tasks 18, 19 and 21 all read state back
through it.

**Build.** Each method against `ObjectClass.cpp`/`ClassClass.cpp`'s behaviour, on both engines.
**`~method` reads the class's own instance dictionary and nothing else** (`ClassClass.cpp:984`): it
raises 97.1 for an inherited or donated name and for a class-side name, measured on both `.K~method("M")`
for a class method and `.Array~method("STRING")`. That is the single most load-bearing detail in the
task, because a build that flattens every scope onto one class answers where the oracle raises.
**`~identityHash` answers with the handle deviation 4 licenses**; identity semantics stay 5c's, and
that split is named here and in the boundary rulings above, where it is recorded as ruled.

**Verification, runnable now.** Oracle-differential per method on a `::CLASS`-declared class and on
the primitive classes, both engines. Measured expected answers on hand: `.K~superClasses` is
`The Object class`, `.K~metaClass` is `The Class class`, `.K~isA(.Class)` is `1`, `.K~method("M")` is
rc 159 with a `Compiled method "METHOD" with scope "Class".` frame (Task 6's line, already agreeing
for an explicit send), `.array~id` is `Array`, `.Array~package~name` is `REXX`. The wiring rows in
table C move from `diverge-status` towards `agree`, and the count is this task's headline number.

**What it cannot see.** Nothing here asks whether a *class* method sits at the right scope; the spec
records that no Rexx-level instrument answers that, and the nearest one is a scope-override send.

**Done when** the wiring rows for the classes this crate registers read `agree`, and **the two
controls this task owns are recorded as run**, which is the first point at which either can flip
something green.

**"The classes this crate registers" is narrower than the criterion, and the gap has an owner.**
`Queue`, `Stem` and `VariableReference` are deferred in `native_classes.rs` for a mechanism no task
owned when this plan was written; **Task 21 owns it now** and lifts them. Their wiring rows stay
non-`agree` through this task and that is expected rather than a regression -- the number this task
reports is the one Task 21 moves again. The controls:

* dropping a class from the registry reddens its **table C wiring row**;
* answering `~method` from the flattened behaviour reddens **this task's own `~method` corpus
  programs** -- `.Array~method("STRING")` and `.K~method("M")` must raise 97.1 with the oracle's frame
  line, `.Array~method("APPEND")` must answer. **Not a table C row**: the plan builds no scope row
  class, for the reason Task 5 records, and these programs are the whole of the guard. They go into
  `phase-5a.txt` in this commit, and the task names which classes they cover -- that named list is the
  extent of the protection against a build that flattens every scope onto one class.

Sitting required.

---

