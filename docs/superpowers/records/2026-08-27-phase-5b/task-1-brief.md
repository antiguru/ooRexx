## Task 1: `~new`, `INIT`, and the instance seam

**Goal.** `corpus/gate-tables/concepts/creo.rex` and `abscla.rex` agree.

**Measured.** Both are rc 120 today, refused at `method "NEW" of class "Object" is not implemented
(Phase 5)`. Oracle for `creo.rex` is rc 0, `type a savings account` / `balance 1000.00` /
`rate 6.25`; for `abscla.rex` it is rc 158, stdout `declared AB Class`, stderr the `98.989 Class AB
is ABSTRACT and cannot be directly created.` transcript with a `Compiled method "NEW" with scope
"Object".` frame.

**What is already there, probed separately.** `Body::Instance(ScopePools)` exists and is what a class
object's own variables already use; `receiver_kind` (`dispatch.rs:1123`) refuses it with
`Err("an instance of a user class")`, which is the seam. `("Object", "INIT", Arity::Fixed(0),
native_no_op)` is already in the native table (`dispatch.rs:375`).
`~objectName`/`~objectName=`/`~string`/`~class`/`~isA` all answer for a class receiver and need only a
second receiver kind. **`~defaultName` answers for no receiver at all** -- `DEFAULTNAME` appears
nowhere in `crates/rexx-exec/src/` outside doc comments, and `.K~defaultName` is rc 120 on both
engines today against the oracle's `The K class` -- so it is built here from zero.
**Re-measure all of this.**

**Build.** `RexxClass::completeNewObject` (`ClassClass.cpp:1882`) in its four steps, in order:
`checkAbstract`, set the behaviour from the class's instance behaviour, register `UNINIT` if the
class or a parent defines it, send `INIT` with `~new`'s arguments. The registry already carries the
`UNINIT` flags (`has_uninit`, `parent_has_uninit`) that step three reads; Task 5 owns what happens
with them afterwards.

Also in this task, because they are the same receiver kind and each is a silent-wrong-answer risk on
its own: an instance's `~string`, `~defaultName`, `~objectName` and `~objectName=`, its `~class` and
`~isA`. Measured, `.Object~new~string` is `an Object` and an instance of `K` is `a K`; setting
`~objectName` changes `~string` and `~objectName` and leaves `~defaultName` answering `a K`.

**5b owes only the constructions its own rows make** (D57, and the spec's native-`~new` split):
`Object` and user classes here, `StringTable` in Task 3, `Array` in Task 8, and **the Message object
`~start` answers** in Task 7 -- *not* `.Message~new`, which refuses `93.901` on the oracle and is 5c's.
Twenty-three of the fifty-nine `*__instance.rex` classes refuse a bare `~new` on the oracle and are
5c's; do not build them and do not treat their rows as this phase's business. `Message` is one of the
twenty-three, which is why the two obligations have to be said apart.

**The rooting hazard is this task's, and it needs a targeted witness rather than the subset run.**
`Interp::pool_owner`'s doc comment (`run.rs:3077`) records that a running send's receiver is rooted
only by the `SELF` slot, which a method body may assign over, and names "the task that creates
instances (`~new`)" as the one that can settle it. This task makes it reachable, and it is a
use-after-free rather than a wrong answer.

**The subset-wide stress run cannot see it**, two ways: `collect_stress.rs`'s subset test walks
`SUBSET_FILES`, which contains no gate-table probe, so neither of this task's rows is in it; and no
ordinary program assigns over `SELF`. The construct is legal and reachable -- measured, oracle rc 0,
`a kept 192 clobbered`:

```rexx
o = .K~new
say 'a' o~go
::CLASS K
::METHOD init
  expose v
  v = 'kept'
::METHOD go
  expose v
  self = 'clobbered'
  t = ''
  do i = 1 to 100
    t = t || i
  end
  return v length(t) self
```

`collect_stress.rs` already holds targeted cases for exactly this class of hazard; this is one more
beside them.

**Done when**

* both rows agree on both engines under the phase-gate command;
* a program that assigns over `SELF` inside a method body and then allocates enough to force a
  collection agrees on three descriptors, is in `corpus/phase-5b.txt`, and passes under
  `run_program_collect_every_alloc` as a targeted case beside `collect_stress.rs`'s existing ones;
* **`corpus/phase-5b.txt` carries the instance-naming witness.** No gate row sends `~string`,
  `~defaultName`, `~objectName`, `~objectName=`, `~class` or `~isA` to an *instance* -- `creo.rex`
  sends `~type`/`~balance`/`~rate`, `abscla.rex` sends to a class, and Object's instance method row is
  `hasMethod` calls and is 5c's -- so six methods this task calls a silent-wrong-answer risk would
  otherwise have no witness at all. Oracle for the instance receiver, rc 0: `an Object`, then `a K`
  for `~string`/`~defaultName`/`~objectName`, then after `~objectName = 'zed'` both `~string` and
  `~objectName` answer `zed` while `~defaultName` still answers `a K`;
* and **two controls are recorded as run**: dropping `checkAbstract` from the `~new` path makes
  `abscla` construct (the row's own committed control), and not sending `INIT` after the object is
  built reddens `creo`.

---

