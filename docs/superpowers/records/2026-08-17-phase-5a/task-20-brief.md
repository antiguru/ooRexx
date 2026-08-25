## Task 20: `::ANNOTATE`'s six targets, and the readback

**Goal.** D54 and M1. `::ANNOTATE` naming a target is **in scope**, with `~annotation` and
`~annotations`.

**The claim being retired, and where it lives.** The superseded spec and the old plan's Task 4 say
`::ANNOTATE naming a target` is not an over-refusal "because the oracle refuses it too (99.945, rc
157)". **99.945 is the unknown-target-name case only.** Measured: `::ANNOTATE CLASS K author "moritz"`
with `::CLASS K` declared, read back as `say .K~annotation("AUTHOR")`, is **oracle rc 0, `moritz`**
against **crate rc 120**. `dire.xml` documents six target types with a worked example,
`fundclasses.xml` documents the readback, and `ootest`'s `ANNOTATE.testGroup` has a passing test per
target -- which Task 2's reading is what confirms directly.

**Build.** All six targets -- `ATTRIBUTE CLASS CONSTANT METHOD PACKAGE ROUTINE` -- and
`~annotation`/`~annotations` on the objects that carry them.

**Installing all six is this task's; reading all six back is not, and the split is measured rather
than assumed.** A readback needs a handle on the annotated object, and 5a has one for some of the
targets and none for the rest. Measured on the oracle, each at rc 0 `moritz`:

* **`CLASS`** -- `say .K~annotation("AUTHOR")`, the class object itself.
* **`METHOD`, `ATTRIBUTE`, `CONSTANT`** -- `.K~method("M")~annotation("AUTHOR")` and the same shape
  through `~method("A")` and `~method("C")`. Each rides Task 9's `~method`, which precedes this
  task, and each reads the class's own instance dictionary.
* **`PACKAGE`** -- the only route is `.context~package~annotation("AUTHOR")`, and **the Package
  object and the `RexxContext~package` accessor that reaches it are both Task 21's**, which follows
  this task. Crate today: rc 120, and measured, it falls on the *send*: `say .context` is rc 0
  `a RexxContext` on both sides, the context object having landed with the old plan's Task 6. So this
  task installs the target and **Task 21 owns its readback**, which its "Done when" carries and its
  Build list supplies the accessor for.
* **`ROUTINE`** -- the routes are `.routines["R"]~annotation("AUTHOR")` and
  `Package~findRoutine`; measured, the first answers on the oracle. But a populated `.ROUTINES` is
  5c's by this plan's own handover and `~findRoutine` is in no task here, so **the `ROUTINE` target's
  readback is 5c's**, recorded in Task 24's handover list beside `.ROUTINES` itself.

Both halves of each split are owned, which is what D46 asks; an earlier draft asked this task for
every readback, and the `PACKAGE` and `ROUTINE` ones have no route at this point in the sequence.

**The three places the generalisation lives, each with its own fix:** the superseded spec (already
superseded); the old plan's Task 4 (superseded by this file); and
**`rexx-exec/src/lib.rs:1285`-`1287`**, whose sentence is true of the program it names and sits where
a reader takes it as the reason the whole family is refused. **The refusal is retired, not
re-commented**, and the measurement stays with its sibling beside it.
`docs/superpowers/plans/phase-4-exclusions.txt:420` carries `::annotate routine nosuchrtn` under
"REFUSED BY THE ORACLE BEFORE main" -- **that line is a true measurement and is not the false
statement**; what it is missing is the sibling row for a valid target, which this task adds.

**Verification, runnable now.** Oracle-differential per target on the install, and per readback form
for those whose route exists here, both engines, plus the unknown-target case staying at `99.945`
rc 157 byte for byte.

**Done when** all six targets install on both engines, the `CLASS`, `METHOD`, `ATTRIBUTE` and
`CONSTANT` readbacks agree byte for byte, the `PACKAGE` and `ROUTINE` readbacks are recorded against
Task 21 and 5c respectively, the unknown-target refusal is unchanged, and the three carriers of the
generalisation are fixed. Sitting required.

---

