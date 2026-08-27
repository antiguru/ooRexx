## Task 2: `Method~scope`

**Goal.** `corpus/gate-tables/concepts/xscope.rex` agrees.

**Measured.** Oracle **rc 159**: prints `base BASE` and `sub SUB`, then raises
`97.1 Object "The SUB class" does not understand message "BASEONLY"` on the third line, under the
frame `Compiled method "METHOD" with scope "Class".` Crate: rc 120,
`method "SCOPE" of class "Method" is not implemented`.

**What is already there, probed separately.** The row's error tail **already matches byte for byte**:
`say .sub~method("BASEONLY")` on a `::class sub subclass base` is rc 159 with identical stderr on
both sides today, because `~method` reads the class's own dictionary alone. `.Method~id` and
`Method~class` both answer. So this task is `Method~scope` and its `~id`, and nothing else.

**Build.** `MethodClass::scope` answers the scope the method object carries. The previous plan's
Task 21 put `scope: Option<ObjRef>` on `NativeObject` for `MethodClass::newScope`, so the field
exists; what is missing is the reader. State what a method object with no scope yet answers, and
whether any route in this phase can produce one.

**Done when** the row agrees on both engines and a control is recorded: answering the *defining*
class rather than the scope leaves `base` right and `sub` wrong, and the row reddens.

---

