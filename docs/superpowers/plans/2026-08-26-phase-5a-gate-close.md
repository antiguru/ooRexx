# Phase 5a gate close

**Goal.** Make the five gate-table rows owned by 5a that do not `agree` with the oracle agree, then
flip `5a` into `CLOSED_PHASES` so every 5a row in both tables gates rather than reports.

**Where this comes from.** `docs/superpowers/records/2026-08-17-phase-5a/task-24-report.md` measured
the gate rather than asserting it and found 5a open on exactly five rows. That report carries the
flip's own line, the rows, and the reasoning for leaving it out; this plan closes the rows and takes
the flip. The set of five was re-derived three independent ways by that task's reviewer, which is why
this plan starts from it rather than re-measuring the set.

**Base.** `8b4a7459d`, corpus 251 of 251, all five gates green.

## Global constraints

`docs/superpowers/records/2026-08-17-phase-5a/` has the constraints this project runs under and they
bind unchanged. The ones this plan leans on hardest:

* **Differential correctness.** stdout, stderr and exit status byte for byte against the oracle, on
  `REXX_ENGINE=ir` **and** `REXX_ENGINE=tree-walker`. Three descriptors read separately, never
  `2>&1`. Probes from a fresh empty directory with absolute paths. The crate bootstraps on every run,
  so bound crate probes with `timeout -s KILL 20`.
* **A silent wrong answer is the worst defect here** and no gate sees it. Every task below turns a
  loud refusal into an answer, which is exactly when one is introduced.
* **The five gate commands** at the end of every task, `--no-fail-fast`, each status read unpiped,
  and **the command that printed a figure quoted beside the figure**.
* Comments: ASCII only, no em-dashes, no historical framing, and no comment states the size of a set.
* Never `git checkout -- <path>` on a file you have edited; `cp` to the scratchpad and restore from
  the copy. Never `rm` with a star glob. Commit with `-F`, name paths explicitly, never amend.
* **Prefer deleting to rewriting.** Every false sentence the previous plan shipped arrived as an
  added justification whose argument was right.

**Measured at `8b4a7459d`, by the controller, before this plan was written.** Each row's *first*
missing piece is what its refusal names; what follows behind it was probed separately and is listed
per task, so no task is sized from an assumption.

---

## Task 1: `::ATTRIBUTE EXTERNAL`

**Goal.** `corpus/gate-tables/directives/attribute__external__subkeyword.rex` agrees.

**Measured.** Oracle **rc 166**, stderr echoing the directive's own line 3, then
`Error 90 running <path> line 3:  External name not found.` and
`Error 90.998:  Unable to find external method "GETzzz_no_entry".` Crate: rc 120,
`::ATTRIBUTE EXTERNAL is not implemented (Phase 7)`.

**Build.** `GET` and `SET` are **prepended to the procedure name**, not appended to the method name --
`concatToCstring` appends its receiver to its argument (`classes/StringClass.cpp:1405`-`:1416`), and
the previous plan's Task 22 recorded the measurement. Resolve `GET`+procedure against the
`LIBRARY REXX` entry-point registry Task 22 built, raise 90.998 naming the composed name on the miss,
and the `SET` half after it. `::METHOD ... ATTRIBUTE EXTERNAL` is the same mechanism
(`parser/DirectiveParser.cpp:867`, `:1678`) and both spellings must move together or neither.

**The registry exports no `GET*`/`SET*` entry at all** -- measured,
`/bin/grep -n "INTERNAL_METHOD(GET" interpreter/runtime/NativeMethods.h` matches nothing, and the
`SET` form likewise.

**That does not make every `::ATTRIBUTE ... EXTERNAL 'LIBRARY REXX x'` a 90.998, and this paragraph
used to say it did.** The prefix is unconditional only for `ATTRIBUTE_BOTH` and for the `::METHOD`
spelling. A `::ATTRIBUTE ... GET` or `... SET` prepends only where the decoded procedure IS the
default -- the C++ asks `internalname == procedure` over two strings `commonString` interned
(`parser/DirectiveParser.cpp:1737`, `:1802`) -- so an explicit third word that differs from the
upcased name resolves unchanged. Measured, oracle: `::attribute at get external 'LIBRARY REXX
file_separator'` is **rc 0**, and with `class` on it `.k~at` answers `/`. Task 1 binds those forms
too, through the machinery the `::METHOD` form already used, so the resolving half is answered
rather than left a refusal.

**Done when** the row agrees on both engines, `::METHOD ... ATTRIBUTE EXTERNAL` agrees too, and a
control is recorded: appending `GET` instead of prepending it names `zzz_no_entryGET` and the row
reddens.

---

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
exists. State what a method object with no scope yet answers, and whether any route in this phase can
produce one.

**This paragraph used to say the reader was all that was missing, and that was wrong by one write.**
`Interp::method_object`, the `Class~method` route, minted a method object and left its scope empty,
so a reader alone would have answered `.nil` for **every** name a class's own dictionary holds and
the row's first two lines with it. The scope was written only where `method_new_scope` ran, which is
the `~define` family. Measured by Task 2 and recorded in `lang/method_scope.rex`.

**Done when** the row agrees on both engines and a control is recorded: answering the *defining*
class rather than the scope leaves `base` right and `sub` wrong, and the row reddens.

---

## Task 3: `Class~subclass` and `Class~mixinClass`

**Goal.** `corpus/gate-tables/concepts/usingcl.rex` agrees.

**Measured.** Oracle **rc 0**, four lines: `mixin-base Object`, `subclass-of Array`,
`superclasses The Array class The Persistence class`, `default-superclass Object`. Crate: rc 120 at
`MIXINCLASS`, and `SUBCLASS` behind it.

**What is already there, probed separately.** `Class~baseClass`, `Class~superClass` and
`Array~makeString` all answer today. The row's `~~inherit` twiddle reaches `SUBCLASS` first, so the
twiddle itself is untested by the refusal and must be checked once the sends resolve.

**Build.** `RexxClass::subclass` and `RexxClass::mixinClass` as the factory protocol reached by
message rather than by directive -- the same protocol `::CLASS` installs through, which is why the
previous plan's enumeration filed them at 5a. Both send `INIT` to the class they build
(`classes/ClassClass.cpp:1631`).

**Both take three arguments and the row uses one, so the row does not size the task.** `Setup.cpp`
declares each at 3 (`:470`, `:474`) and each extra argument is reachable and observable from a Rexx
program this crate can already run. Measured at `637d0dd64`, oracle:

* the **metaclass** is argument 2, defaulting to the receiver's own, and its test runs *before* the
  class id's -- `.object~subclass(, .Object)` is 99.927 and not the 88.901 its omitted id would earn,
  while `.object~subclass(, .Class)` is that 88.901. A supplied `.nil` is not an omission and is its
  own 99.927;
* the **enhancing class methods** are argument 3, reachable through `.methods`, merged into the class
  method dictionary before the `INIT` send -- so an enhancing `INIT` is the one that runs;
* the class id's own refusals are raised **inside a `NEW` send to the metaclass**
  (`classes/ClassClass.cpp:1576`), so their reports carry two traceback frames:
  `Compiled method "NEW" with scope "Class".` above `Compiled method "SUBCLASS" with scope "Class".`
  A metaclass carrying its own `NEW` intercepts the construction entirely, which is a class this
  crate has no `NEW` to build.

**Two things to decide and record**, because a class built at run time meets machinery built for
directive-installed ones: **which package** the new class belongs to, and **whether the REXX_DEFINED
lock applies to it**. Measure both against the oracle rather than reasoning: a user program can ask
`~package~name`, and can try to mutate what it just built.

**Done when** the row agrees on both engines, the two decisions above are measured and recorded, and
a control is recorded that reddens the row.

**The control this task shipped with was wrong about which line it reddens, and the measurement
replaces it.** It read "defaulting the superclass to something other than `.Object` reddens
`default-superclass`". The mutation that phrase describes -- build the class under `.Object` rather
than under the receiver, which is also gate table C's own registered control for this row -- leaves
`default-superclass` **green**, because the row asks that line of `.object~subclass("plain")`, whose
receiver already is `.Object`. Measured at `REXX_ENGINE=ir`: it reddens `subclass-of` and
`superclasses` and the row reads `diverge-stdout`. Building the class under its *metaclass* is what
reddens all four lines, `default-superclass` among them. `~subclass` has no superclass default to
change: the superclass is the receiver.

---

## Task 4: `.RexxInfo`

**Goal.** `corpus/gate-tables/classes/rexxinfo.rex` agrees.

**Measured, and it is smaller than "build the RexxInfo class".** Oracle **rc 159**: it prints
`entry a RexxInfo` and `class-of-entry RexxInfo`, then raises
`97.1 Object "a RexxInfo" does not understand message "ID"` on the third line -- because
**`.RexxInfo` is an instance, not a class**. Also measured: `.RexxInfo~class~id` is `RexxInfo`,
`.RexxInfo~class~superClass~id` is `Object`, and `.RexxInfo~string` is `a RexxInfo`.

So the row needs a `RexxInfo` **class object that no environment symbol reaches**, and a pre-built
**instance** of it under `.RexxInfo` -- which is exactly what the spec says twice: `RexxInfo` is
`addToSystem`-only (`EndSpecialClassDefinition`), and only the instance is `addToEnvironment`'d
(`memory/Setup.cpp:1737`).

**Build.** The class, off the registry's environment-reachable path; the instance in `.environment`;
and nothing else.

**The instance's unbuilt surface refuses LOUDLY, at rc 120, and never with 97.1.** The oracle
understands most of what this task does not build: measured, oracle rc 0, `.RexxInfo~digits` is `9`,
`.RexxInfo~languageLevel` is `6.06`, `.RexxInfo~form` is `SCIENTIFIC`, `.RexxInfo~fuzz` is `0`,
`.RexxInfo~internalDigits` is `18` and `.RexxInfo~objectName` is `a RexxInfo`; of that list only `ID`
and `FILESEPARATOR` answer `hasMethod` `0`. So a 97.1 for `~version` or `~digits` would be a silent
wrong answer about the language, which is the worst defect class here and one no gate sees. `~id`
keeps its genuine 97.1, because there the oracle really does not understand the message.

**Done when** the row agrees on both engines, `DEFERRALS`' `RexxInfo` entry is retired or narrowed to
what still stands, and a control is recorded: registering the class under `.RexxInfo` instead of the
instance makes `entry` read `The RexxInfo class` and the row reddens.

---

## Task 5: `~define` with source text

**Goal.** `corpus/gate-tables/concepts/methna.rex` agrees.

**Measured.** Oracle **rc 0**, four lines: `id COST`, then `The Method class` three times for
`~method("%")`, `~method("TYPE")` and `~method("type")`. Crate: rc 120,
`a method built from source text is not implemented`.

**What is already there, probed separately.** `~define` with a **method object** works today;
`~method(...)~class` answers `The Method class`. What is missing is compiling a method body from a
string outside a `::METHOD` directive.

**Build.** `MethodClass::newMethodObject` compiles anything that is not already a method object
(`classes/MethodClass.cpp:457`-`:486`). The row also pins that **the dictionary key is the upcased
name** while a quoted name keeps its spelling as the method's own name: `~method("TYPE")` and
`~method("type")` both answer for a name defined as `"type"`, and `"%"` is a name no symbol could
hold.

**Say what a compiled body can and cannot do in this phase**, since the body is real Rexx and
reaches the interpreter: the row's own bodies are `return` of a literal. A body that reaches an
unbuilt mechanism must refuse loudly rather than answer wrongly.

**Done when** the row agrees on both engines, the upcasing pair is pinned by a corpus row of this
task's own, and a control is recorded: keeping the as-written spelling as the dictionary key makes
`~method("TYPE")` raise where the oracle answers, and the row reddens.

---

## Task 6: the flip

**Goal.** `5a` in `CLOSED_PHASES`, and the five gate commands green with the tables' gating arm live.

**Build.** The one line, at `rust/crates/rexx-exec/tests/gate_tables/mod.rs`. The previous plan's
Task 24 verified that `CLOSED_PHASES` is the switch it appears to be, by reddening the identical rows
two ways -- through the constant and through `REXX_PHASE_GATE=5a` -- so this task inherits that and
does not re-derive it.

**Done when** all five gate commands exit 0 with the flip committed, and a negative control is
recorded **in the shape Task 24 established**: revert one mechanism a 5a row depends on, confirm the
gate command exits non-zero, and confirm it reddens **a nameable set rather than everything**. A
control that reddens the whole table has usually broken the bootstrap and witnesses far less.

**If a sixth row appears** once the five are closed -- a row whose owning phase is 5a and whose
verdict is not `agree` -- that is a finding and the report says so. Do not re-file a row to another
phase to make the gate pass: that closes a gate row by narrowing what the gate covers, which Task 24
declined to do for `::ATTRIBUTE EXTERNAL` and this plan will not do either.
