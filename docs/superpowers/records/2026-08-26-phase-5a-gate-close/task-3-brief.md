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

**Two things to decide and record**, because a class built at run time meets machinery built for
directive-installed ones: **which package** the new class belongs to, and **whether the REXX_DEFINED
lock applies to it**. Measure both against the oracle rather than reasoning: a user program can ask
`~package~name`, and can try to mutate what it just built.

**Done when** the row agrees on both engines, the two decisions above are measured and recorded, and
a control is recorded: defaulting the superclass to something other than `.Object` reddens
`default-superclass`.

---


## Controller addendum, measured at `637d0dd64` before dispatch

**Treat the brief's "what is already there" paragraph as a claim to re-measure, not a premise.** It
has been wrong twice on this plan, both times in the direction of making the task look smaller
(Task 1's registry paragraph, Task 2's "the field exists, only the reader is missing"). Correct the
plan file where you find it wrong, not this brief and not your report.

**The row's other sends, which the brief does not name, all answer today** -- oracle and `ir`
byte-identical, rc 0:

```
say 'A' .array~superClasses~makeString('L', ' ')   ->  A The Object class The OrderedCollection class
say 'B' .array~baseClass~id                        ->  B Array
say 'C' .array~superClass~id                       ->  C Object
```

So `~superClasses`, `~makeString('L', ' ')`, `~baseClass` and `~superClass` are all present. The
first refusal is `MIXINCLASS`, as the brief says.

**The two decisions are both directly observable, and I measured them on the oracle so you can check
your build against a number rather than an argument.** Re-run them yourself; do not take these on
trust.

*Which package.* **None.** A runtime-built class's `~package` is `.nil`:

```
k = .object~subclass("k")
say 'pkgname' k~package~name
```
oracle **rc 159**, stdout empty, `Error 97.1: Object "The NIL object" does not understand message
"NAME".` So `~package` must answer `.nil`, not the calling program's package. Check what
`.context~package~classes` does and does not now contain, and what `~package` answers for a class a
`::CLASS` directive installed (that one answers the program path -- measured, oracle and `ir` agree).

*Whether the REXX_DEFINED lock applies.* **It does not.** `~inherit` on a runtime-built class is
rc 0:

```
k = .object~subclass("k")
k~inherit(.object~mixinclass("mx"))
say 'inherit-ok' k~superClasses~makeString('L', ' ')  ->  inherit-ok The Object class The mx class
say 'id' k~id                                         ->  id k
say 'string' k~string                                 ->  string The k class
say 'baseclass' k~baseClass~id                        ->  baseclass k
```

The lock itself already exists in the crate and already matches: `.array~inherit(.object)` is
**rc 158** with `Error 98.985: User additions are not allowed to the REXX language classes.`,
byte-identical on oracle and `ir` today. So the risk is the opposite one -- a runtime-built class
inheriting the lock it must not have. Pin both directions.

Note from those four lines: **`~id` keeps the spelling as written** (`k`, not `K`), `~string` is
`The k class`, and a `~subclass`-built class is **its own** `baseClass` while a `~mixinclass`-built
one reports its `baseClass` as the class it was sent to (`Object` in the row).

**Task 2 built `Method~scope` and threads a dictionary slot's scope through `Interp::method_object`.**
A class you build at run time gets methods through `~define`/`~defineMethods`, so
`k~method(name)~scope` must answer the new class. That pair is the one genuine interaction between
this task and an earlier one; probe it.

**The hazard, restated.** Every task on this plan turns a loud refusal into an answer, which is
exactly where a silent wrong answer at rc 0 is introduced, and no gate sees one. Probe past the row.
