## Task 21: the mutators, native removal and hiding, the REXX_DEFINED lock, and the Package object

**Goal.** `~define`, `~defineMethods`, `~delete`, `~uninherit`; **`Setup.cpp`'s native `RemoveMethod`
and `HideMethod`, and the deferrals they hold shut**; the REXX_DEFINED lock that refuses all five on
the interpreter's own classes; and the Package object the bootstrap is handed.

**Why after Task 17.** The lock's probe sends `.methods~z`, which is Task 17's.

**Why the native removal and hiding land here rather than in the registry task that met them.** They
are the same `MethodDictionary` operations `~delete` and a `.nil` entry are the Rexx-level face of, so
building them apart from the mutators would build them twice; and **hiding is dispatch's `UNKNOWN`
limb** -- `MethodDictionary::hideMethod` is `put(TheNilObject, name)`, the spec's own reading -- so it
needs Task 12, which precedes this task and would not have preceded Task 9.

**Build.** The four mutators on a user class, with `define` copying the behaviour before mutating and
`inherit` mutating in place (D43); **the REXX_DEFINED lock** (`liveGeneral` sets `setRexxDefined()`
under `PREPARINGIMAGE` and repoints `package` to `TheRexxPackage`), raising `98.985` in `define`,
`defineMethods`, `delete`, `inherit` and `uninherit`; and `Package` with `addClass`,
`addPublicClass`, `publicClasses`, `~name` -- **`~annotation`/`~annotations` on the package object
are Task 20's and are already delivered**, reached through `.K~package`, which this paragraph did not
consider -- **together with `RexxContext~package`, which is what remains unreachable**. Measured,
`say .context` is rc 0 `a RexxContext` on both sides and the crate's rc 120 falls on the send, so
`.context~package~annotation(...)` has no route until this task supplies one, and neither of the
nearby accessors is it: Task 9's `Class~package` reaches the REXX package rather than the running
one (`.Array~package~name` is `REXX`, measured), and `CoreClasses.orx:47` uses a bound `rexxPackage`
variable rather than `.context~package`, read at the file. `~enhanced` is 5b's -- it builds an
instance.

**And the native layer's own removal and hiding, which the enumeration gained after this plan was
first written.** `MethodDict` models neither, and that is what `rexx-classes/src/native_classes.rs`'s
`DEFERRALS` names as the reason `QueueClass`, `StemClass` and `VariableReference` are not built:

* **removal is a deletion**, `Setup.cpp:792`-`:804` taking names back off Queue's donated Array set;
  **hiding is a `.nil` tombstone**, `:1307`-`:1312` and `:1399`-`:1404` over the comparison operators
  of `VariableReference` and `Stem`, and a hidden name routes to `UNKNOWN` rather than answering.
* **The operations are already derived and are sitting unread.** `build.rs` emits
  `Op::RemoveInstanceMethod` and `Op::HideInstanceMethod` into `CLASS_DEFINITIONS` in each block's own
  order -- so this task replays what is already there and adds no extraction.
* **Lifting those deferrals is the point**, because the spec puts `Queue`, `Stem` and
  `VariableReference` inside Task 24's wiring criterion and **no `.orx` file this plan embeds declares
  any of them** -- measured, the only matching `::CLASS` lines are `CircularQueue` and `RexxQueue` --
  so nothing else here can supply them. `RexxInfo` stays deferred and stays outside the class half of
  the criterion, by the spec's own exclusion: its `.environment` entry is an instance.
* **And Task 23 needs `.Queue` to exist, not merely to be gated on it.** `CoreClasses.orx` declares
  `::CLASS 'CircularQueue' subclass queue public`, so the bootstrap's own install list names the
  deferred class. That is the hard reason this task precedes Task 23, beside the soft one.
* **`Queue`'s and `Stem`'s rows do not finish here, for the ordinary reason no collection's row
  does.** Measured on the shipped image, `.Stem~superClasses` is `The Object class` and
  `The MapCollection class` and `.Queue~superClasses` is `The Object class` and
  `The OrderedCollection class`; those mixin edges come from `CoreClasses.orx:110` and `:99`, so they
  arrive with the bootstrap, and Task 23's post-bootstrap re-run of the wiring half is where those
  rows close -- exactly as `.Array`'s does. `.VariableReference~superClasses` is `The Object class`
  alone, so its row closes here. Said because a reader otherwise takes a red `Stem` or `Queue` row
  after this task for a regression.

**Verification, runnable now.** Measured: `.Array~package~name` is `REXX`, and
`.Array~define("ZORK", .methods~z)` is oracle **rc 158** with
`*-* Compiled method "DEFINE" with scope "Class".` and
`98.985 User additions are not allowed to the REXX language classes.`, against crate rc 120. Plus the
mutators on a *user* class, where they must succeed, and `.K~hasMethod` before and after each.

**The removal and hiding have a differential, and it does not wait for an instance.** The spec says
the composition of native removal and sourced donation is what is observable and that a native
removal is not observable on its own -- that is true of `~hasMethod`, and **false of `~method`**,
which reads the class's own instance dictionary. Measured, all on the shipped oracle at rc 0 unless
stated, crate rc 120 throughout because `.STEM` is not even an environment symbol here:

* `.Stem~method("==")` and `.VariableReference~method("==")` are **`The NIL object`** -- the tombstone
  read back, the hiding visible as itself;
* `.Queue~method("SORT")` and `.Queue~method("MAKESTRING")` raise **rc 159 `97.1`** while
  `.Array~method("SORT")` and `.Array~method("MAKESTRING")` both answer `a Method` -- removal, read
  against the class the set was donated from, name for name;
* `.Queue~method("APPEND")` answers `a Method`, which is the decoy the pair needs: without it a build
  that donated nothing to Queue at all would pass the two rows above.

These are corpus programs of this task, on both engines, and they need Task 9's `~method`.

**Two things this task builds that no differential can see, said plainly.**

* **The setup methods.** `removeSetupMethods()` deletes `DEFINECLASSMETHOD` and
  `INHERITINSTANCEMETHODS` before any user program runs, so the shipped oracle answers `0` to
  `hasMethod` for both and there is **no user-level program that can compare them**. Their only
  instruments are Task 23's post-bootstrap state check and in-crate tests. A differential cannot be
  written and the task must not imply one.
* **D43's discriminating witness is 5b's.** The pair-in-sequence -- `o = .K~new` first, then `define`
  then `inherit` against `inherit` then `define` -- is what separates a version-stamped
  rebuild-in-place from the real thing, and it needs an instance. Here the readback is class-side
  only, and the pair joins 5b's debt list explicitly.

**Done when** the lock reproduces `98.985` byte for byte on both engines including its frame line, the
four mutators work on a user class, `Queue`, `Stem` and `VariableReference` are registered and their
wiring rows read `agree` for every question the pre-bootstrap state can answer -- `Queue`'s and
`Stem`'s `~superClasses` excepted and recorded against Task 23, and **three controls are recorded as
run**:

* **the lock rather than the copy**: removing the REXX_DEFINED check makes `.Array~define("ZORK", …)`
  succeed at rc 0 where the oracle raises, and the program reddens;
* **skip the replay for one class**: `.Queue~method("SORT")` then answers `a Method` where the oracle
  raises `97.1`, and that program reddens;
* **model hiding as removal**: `.Stem~method("==")` then raises `97.1` where the oracle answers
  `The NIL object`, and that program reddens. This is the control that separates the two operations
  from each other, and it is why they are not built as one.

**The control an earlier draft named here could not fire, and the reason generalises.** It was "skip
the copy in `define` and the class-side row reddens". `define`'s copy exists so that objects created
*before* the mutation do not see it -- with no instances, nothing holds the old behaviour, and copy
and mutate-in-place are indistinguishable from any class-side program. That is D43's own finding, and
it is why its witness is 5b's; a control over it here would have done the same thing whether or not
the copy were there. Sitting required.

---

