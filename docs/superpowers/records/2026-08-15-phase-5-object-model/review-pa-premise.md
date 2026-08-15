# Review P-A: does the prologue ever run a Rexx method body?

**Subject:** `docs/superpowers/plans/2026-08-15-phase-5a-native-layer.md`.
**Lens:** the plan's scope boundary rests on one assertion, at lines 16-23 of the plan:

> **The bound on 5a is that the prologue never runs a Rexx method body.** Read at
> `CoreClasses.orx:39-126`, every message it sends lands on a native method [...] `::METHOD`
> directives are *installed*, and their bodies are stored and never entered.

**Verdict: HOLDS WITH EXCEPTIONS.** The narrow claim is true and I could not break it: every message
sent by `CoreClasses.orx:39-126` lands on a native method, and the `DO OVER` case the brief expected
to break it does not. The *second* sentence is false, and it is the load-bearing one. Installing
`CoreClasses.orx`'s directives enters a Rexx method body -- `TraceObject`'s `::method activate class`
-- **before the prologue's first clause runs**, and that body uses `EXPOSE`, scoped instance
variables on a class object, and a super-qualified send. The plan's own Exit criterion
("`CoreClasses.orx` translates, installs, and its prologue runs to `exit`") is therefore not
reachable under the scope the plan states, because it cannot reach the prologue at all.

All oracle runs used, exactly:
`( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib
/home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`, three descriptors read separately, from the
fresh empty directory
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/cd8e0d76-f9df-4fc9-9792-2e8e29a6d20b/scratchpad/pa-probes`,
absolute paths, never `2>&1`. Every `grep` quoted below is `/bin/grep -a`.

---

## 0. The discriminator, and how I established it

The brief asked me to find a runtime discriminator on Method objects. **`~source`, `~package`,
`~scope`, `~isGuarded` and `~class` are all useless for this**, and the reason is worth recording
because it is the trap: the saved image strips method source, so *every* image-resident method
answers `~source~items` 0, `~package~name` `REXX`, `~class~id` `Method`. Measured (probe `p07.rex`):
`.Supplier~method("ALLITEMS")`, which is Rexx from `CoreClasses.orx:175`, and
`.Directory~method("MAKEARRAY")`, which is C++, are indistinguishable on all five. A method defined
in the probe program itself answers `~source~items` 1 and its own file as the package, so the
accessors work -- they just cannot see into the image.

The discriminator that does work is **the syntax-condition traceback frame text** (`p10.rex`):

| frame text | meaning |
|---|---|
| `*-* Compiled method "X" with scope "Y".` (no line number) | native (C++ / external) |
| `NNN *-* Method X with scope "Y" in package "REXX" (no source available).` | Rexx-defined |

It has to be raised from inside a `procedure`; raised at program top level the method frame is
absent from `~traceback` (measured, `p15.rex` versus `p16.rex`, same sends, different frames). It
also does not survive `INTERPRET` (`p13.rex`). Both of those are exactly the "check that cannot see
its subject" shape, so I record them.

**Positive controls in the same run** (`p16.rex` row 16, `p17.rex` rows 11-12): `.Array~union`
resolves to `1131 *-* Method UNION with scope "OrderedCollection" in package "REXX"`, `.Set~new~union`
to `477 *-* Method UNION with scope "Set"`, `.Bag~new~putAll` to `707 *-* Method PUTALL with scope
"Bag"`. The last two matter most: those are Rexx methods installed by `inheritInstanceMethods`, the
very mechanism `CoreClasses.orx:80-87` uses, and the discriminator sees them. Had the prologue's
sends been reaching Rexx methods, this instrument would have said so.

---

## 1. My own enumeration of `CoreClasses.orx:39-126`

Built by reading the file, not from the plan's list. What I searched *for*: the explicit `~` send;
the `~~` cascade send; `[` `]` index syntax; operator sends, specifically `||`; message-assignment
(`recv~name = value`, which sends `NAME=`); the `DO OVER` implicit array protocol; dot-variable
resolution; `CALL` on a program name; and `USE ARG`. What my terms cannot reach: a Rexx method
arrived at through an `UNKNOWN` method (`CoreClasses.orx:1443` defines one, on `Monitor`, which is
not a receiver anywhere in these lines), and anything that only dispatches under a non-default
`TRACE` setting (`DoBlockComponents.cpp:229` calls `traceKeywordResult` on the `OVER` value, and
tracing a value can send `STRING`/`MAKESTRING`).

| line(s) | receiver | message | verdict | evidence |
|---|---|---|---|---|
| 39 | -- | `USE ARG` sends nothing | n/a | read |
| 47-52 | `rexxPackage` (Package) | `ADDCLASS` | NATIVE | `p16` row 1: `Compiled method "ADDCLASS" with scope "Package"` |
| 47-52 | -- | `.LocalServer` `.SupplierMixin` `.ManyItemMixin` `.SetMixin` `.BagMixin` | not a send | `RexxDotVariable` resolution, no dispatch |
| 55, 56 | Directory / Package | `OBJECTNAME=` | NATIVE | `p17` rows 5-6: `Compiled method "OBJECTNAME=" with scope "Object"` |
| 61 | `.context` (RexxContext) | `PACKAGE` | NATIVE | `p16` row 4 |
| 61 | Package | `PUBLICCLASSES` | NATIVE | `p16` row 3 |
| 63 | result of `PUBLICCLASSES` | `DO OVER` array protocol | NATIVE, and normally no dispatch at all | section 3 |
| 64 | StringTable | `[]` | NATIVE | `p17` row 1: `Compiled method "[]" with scope "StringTable"` |
| 65 | `.environment` (Directory) | `PUT` | NATIVE | `p16` row 7 |
| 66 | Package | `ADDPUBLICCLASS` | NATIVE | `p16` row 2 |
| 70-72 | `RexxExpressionList` result (Array) | `DO OVER` array protocol | NATIVE, no dispatch | section 3 |
| 73 | String | `UPPER` (twice: `name~upper` and `(...)~upper`) | NATIVE | `p16` row 9 |
| 73 | String | `\|\|` (the concatenation operator, a send the plan's list omits) | NATIVE | `p17` rows 3-4: `Compiled method "\|\|" with scope "String"` |
| 73 | `.methods` (StringTable) | `[]` (a second index send the plan's list omits) | NATIVE | `p17` row 7 |
| 73 | `.String` (Class) | `DEFINECLASSMETHOD` | NATIVE | `Setup.cpp:460` `AddProtectedMethod("DefineClassMethod", RexxClass::defineClassMethod, 2)` |
| 80-87 | `.supplier` `.relation` `.bag` `.set` (Class) | `INHERITINSTANCEMETHODS` | NATIVE | `Setup.cpp:461` `AddProtectedMethod("InheritInstanceMethods", RexxClass::inheritInstanceMethodsRexx, 1)` |
| 93-119 | `.string` `.array` `.list` `.queue` `.identityTable` `.table` `.stringTable` `.directory` `.relation` `.set` `.bag` `.stem` `.message` (Class) | `INHERIT` | NATIVE | `p16` row 10: `Compiled method "INHERIT" with scope "Class"` |
| 122, 124 | -- | `CALL` on a program name | not a method send | section 4 |
| 126 | -- | `EXIT` | not a send | read |

`DEFINECLASSMETHOD` and `INHERITINSTANCEMETHODS` cannot be probed at runtime -- `p16` rows 11-12 and
`p24` rows 4-5 answer `97.1`, which independently **confirms the plan's D39 claim** that
`removeSetupMethods()` deletes them (`ClassClass.cpp:923-940`, called at `Setup.cpp:1809`). Their
nativeness comes from `Setup.cpp` instead, which is the only source available.

**So on the narrow claim I found nothing.** The plan's boundary paragraph nonetheless under-lists the
sends: `||`, the second `~upper`, `.methods[...]`, and the `DO OVER` protocol are all absent from it.
Each turns out native, so nothing changes, but a list presented as "every message it sends" should be
the list.

---

## 2. Order dependence: I checked, and it cuts the safe way

The brief warned that a send early in the prologue may reach a native method where the same send
later reaches a Rexx one, because the prologue installs Rexx methods as it goes. All the
classifications in section 1 were measured against **the shipped image**, i.e. the *latest* state,
after every `inheritInstanceMethods` and every `inherit` in the file has already taken effect. That
is the conservative direction: `createInstanceBehaviour` (`ClassClass.cpp:1145-1170`) merges
superclasses first and the class's own dictionary last, so an inherited mixin method can only ever be
*overwritten* by the class's own native one, never the reverse. A send that resolves native in the
final image resolved native earlier too.

The measurement confirms the mechanism rather than assuming it: `.Directory~new~put(1)` and
`.StringTable~new["A","B"]` still answer `Compiled method ... with scope "Directory"` /
`"StringTable"` **after** `CoreClasses.orx:105-106` gave both classes `MapCollection`, whose
`Collection` ancestor defines Rexx `makeArray` (`:767`), `supplier` (`:753`), `hasIndex`, `hasItem`,
`items`, `equivalent`, `disjoint`, `difference`, `interSection`, `subSet`, `union` and `xor`.

---

## 3. `DO OVER` -- the case the brief expected to break it, and it does not

**The protocol is `requestArray`, not `makearray` and not `supplier`.** `OverLoop::setup`
(`DoBlockComponents.cpp:221-256`) evaluates the target and then:

```
if (isArray(result))  array = ((ArrayClass *)result)->makeArray();
else                  array = result->requestArray();
```

`RexxInternalObject::requestArray` (`ObjectClass.cpp:1646-1667`) takes the C++ virtual `makeArray()`
**with no message dispatch at all** when `isBaseClass()` holds, and only falls back to
`sendMessage(REQUEST, ARRAY)` for non-primitive behaviours. `isBaseClass()` is
`behaviour->isPrimitive()` (`ObjectClass.cpp:216-219`), and the only thing that clears that flag is
`RexxBehaviour::copy` (`RexxBehaviour.cpp:214-226`). `~inherit` does not copy: `updateSubClasses`
(`ClassClass.cpp:1035`) and `updateInstanceSubClasses` (`:1069`) clear and rebuild the *existing*
behaviour object in place.

Both prologue receivers are safe, and by two independent routes:

* **`publicClasses` (line 63) is a `StringTable`** -- measured, `p11.rex`,
  `.context~package~publicClasses~class~id` is `StringTable`. Not an array, so `requestArray` runs;
  the behaviour is primitive, so no dispatch happens. And *if* it dispatched anyway, both hops land
  native: `.Object~new~request(1,2,3)` gives `Compiled method "REQUEST" with scope "Object"` (`p24`
  row 1) and `.StringTable~new~makearray(9)` gives `Compiled method "MAKEARRAY" with scope
  "StringTable"` (`p16` row 5).
* **`do name over "nl", "cr", ...` (lines 70-72) is not a multi-collection loop.** `OverLoop` holds a
  single `target` (`DoBlockComponents.hpp:155-176`); the comma list is folded by
  `LanguageParser::parseFullSubExpression` (`LanguageParser.cpp:2754-2796`) into a
  `RexxExpressionList`, which evaluates to an `ArrayClass`. `isArray(result)` is therefore true and
  the loop takes the `makeArray()` fast path -- no send, and none of `SUPPLIER`, `MAKEARRAY` or
  `REQUEST` is involved.

`CoreClasses.orx` does define `makearray`/`supplier`/`allItems` on `Collection` (`:753`, `:767`),
`MapCollection` (`:1292`), `SupplierMixin` (`:175`, `:210`) and `CircularQueue` (`:1861`, `:1890`).
**None of them is in the path**, for the reason in section 2.

One caveat on the plan's Task 8, which says `DO OVER` "over an object [...] must send `makearray` or
`supplier`": that is the wrong protocol name and the wrong shape. It is `requestArray`, it prefers a
non-dispatching C++ path, `SUPPLIER` is never involved (that is `DO WITH`, `DoBlockComponents.cpp:343`),
and the failure mode when conversion fails is `Error_Execution_noarray`, not a `97.1`.

---

## 4. The two `CALL`s

* **`call 'PlatformObjects.orx' rexxPackage` is a no-op on this platform.**
  `/home/moritz/dev/repos/ooRexx/interpreter/platform/unix/PlatformObjects.orx` is one comment line,
  `-- Nothing to do currently`. No directives, no prologue.
* **`call 'StreamClasses.orx' rexxPackage`.** Its prologue (lines 39-50) is a copy of
  `CoreClasses.orx:61-67` -- `.context~package~publicClasses`, `do name over publicClasses`,
  `publicClasses[name]`, `.environment~put`, `rexxPackage~addPublicClass`. Every one is classified
  native in section 1, and those classifications were taken from the shipped image, i.e. from a state
  at least as Rexx-rich as the one that prologue runs in.
* **Its *install*, however, evaluates two `::CONSTANT` expressions.** `StreamClasses.orx:548-549`,
  `::constant separator (.File~getSeparator)` and `::constant pathSeparator (.File~getPathSeparator)`.
  `.File~getSeparator` is `::method getSeparator private class external "LIBRARY REXX file_separator"`
  (`:546`), so the *message* lands native. Measured on the oracle: `say .File~separator` is `/` and
  `say .File~pathSeparator` is `:`, rc 0, empty stderr (`p23.rex`). The plan's Task 6 already
  anticipates this and takes `file_separator`/`file_path_separator` into 5a.

The three files carry only four directive keywords -- `/bin/grep -aoih "^::[a-z]*" CoreClasses.orx
StreamClasses.orx | sort -u` gives `::attribute`, `::class`, `::constant`, `::method` (in both
cases). No `::REQUIRES`, `::OPTIONS`, `::ANNOTATE`, `::RESOURCE`, and no use of the `METACLASS`
keyword (`/bin/grep -ain "::requires\|::options\|::annotate\|metaclass\|::resource"` hits only two
comment lines describing `Singleton`). So `::CLASS`/`::METHOD`/`::ATTRIBUTE`/`::CONSTANT` really is
the whole install surface -- Task 5's list is right.

---

## Findings

### F1 -- CONFIRMED, and it is the one that matters. Installing `CoreClasses.orx` enters a Rexx method body, before the prologue.

`PackageClass::install` (`PackageClass.cpp:1268-1301`) makes three passes over the package's classes:
create, `resolveConstants`, then **`current_class->activate()` for every class**.
`ClassDirective::activate` (`ClassDirective.cpp:284-289`) is one line:
`classObject->sendMessage(GlobalNames::ACTIVATE, result)`.

`CoreClasses.orx:3993-4003` is:

```
::class "TraceObject" subclass StringTable public
::method    activate    class
  expose option collector counter makeStringMethod notify
  self~activate:super
  option   ='N'
  collector=.nil
  makeStringMethod = .nil
  notify   =.nil
  counter  =0
```

`/bin/grep -ain "::method *activate\|::METHOD *ACTIVATE" CoreClasses.orx StreamClasses.orx` returns
`CoreClasses.orx:3996` and nothing else, so `TraceObject` is the only class in either file that
overrides it; every other class reaches the native default (`p20` rows 1-2: `Compiled method
"ACTIVATE" with scope "Class"` for both `.Object` and a fresh user class).

**Measured, two ways.**

*It runs, and it runs before the prologue* (`p18.rex`, a four-line user program, stdout in order):

```
  [ACTIVATE ran for K, self= K ]
prologue line 1
K counter is: 42
```

rc 0, stderr empty. The class's `::method activate class` -- using `expose n`, `self~activate:super`,
and an assignment to the exposed variable -- ran to completion *before* the program's first `say`,
and the exposed value survived into the prologue.

*It ran in the shipped image* (`p19.rex`): `.TraceObject~counter` is `0` and `.TraceObject~option` is
`N`, rc 0. Those are precisely the values `activate` assigns, and the only other assignment site
(`::method new class`) requires an instance that was never created. **Negative control, same run:**
a class whose exposed class variable is never assigned answers with the variable's own uppercased
name -- `.N2~v` prints `V` (`p20.rex`, last line). So `N` is not what an unrun `activate` looks like.

**What this costs the plan.** The plan's line 22-23, "`::METHOD` directives are *installed*, and their
bodies are stored and never entered", is false for `CoreClasses.orx`; so is Task 5's "Bodies are
stored, never entered". And because `activate` runs *before* the prologue, the Exit criterion --
"`CoreClasses.orx` translates, installs, and its prologue runs to `exit`" -- cannot be met by a build
that lacks the machinery. Concretely, reaching `CoreClasses.orx:39` requires, inside 5a:

* entering a Rexx method body with a receiver and a scope -- the plan assigns "method invocation"
  to 5b;
* `EXPOSE` -- assigned to 5b by name;
* an object-variable pool **scoped to a class object** (the receiver of a class method is the class),
  five variables at scope `TraceObject` -- "scoped instance variables", assigned to 5b by name;
* a super-qualified send, `self~activate:super`. Task 3 already builds `resolve`'s optional start
  scope, so this one is half-covered -- but Task 3 says that resolve "invok[es] a **native** method",
  and here the *outer* send is to a Rexx one.

Not required: `FORWARD` (`TraceObject`'s `::method new class` uses `forward class (super) continue`
but `new` is not sent at install), and no `~new`/`init` on `TraceObject`.

**What the plan would have to change.** Skipping `TraceObject` is not available: `PackageClass::install`
activates every class in the package unconditionally, and dropping the class changes
`.environment`/`publicClasses` contents that Task 2's and Task 9's wiring checks compare byte for
byte. So either (a) `EXPOSE`, class-scope object variables and Rexx-body invocation move into 5a as a
new task ordered before Task 9, and the 5a/5b line is redrawn as "5b is `FORWARD`, `~new`/`init`, and
*instance*-side variable scoping"; or (b) 5a's Exit criterion drops to "`CoreClasses.orx` translates
and its directives install as far as `ACTIVATE`", and running the prologue moves to 5b -- which
guts 5a, since Tasks 7, 8 and 9 exist to run the prologue. (a) looks like the smaller change.

Either way, **Task 5's verification is currently blind to this**: it checks directive install
per-directive against the oracle on small programs. Had this finding been false, that check would
have done the same thing. A witness that would see it is a one-class program whose `::method activate
class` writes an exposed variable, with the expected answer taken from `p18.rex`'s ordering.

### F2 -- CONFIRMED. `::CONSTANT (expr)` is not an expression evaluation; it is a synthesized Rexx method run against the class.

`ClassDirective::resolveConstants` (`ClassDirective.cpp:257-277`) builds a `RexxCode` of
`ConstantDirective` instructions, wraps it in `new MethodClass(GlobalNames::CONSTANT_DIRECTIVE, ...)`,
sets its scope to the class object "so that super will get set correctly", and calls
`code->run(activity, classObject, ...)`. So `StreamClasses.orx`'s two constants also enter a Rexx
method body -- a synthesized one, with `self` bound to `.File` and `super` live.

This is smaller than F1: the two expressions in the tree use neither `self`, `super` nor `EXPOSE`, so
evaluating them inline in the installing activation reproduces the oracle's bytes here. But it is a
second place where the plan's "bodies are stored and never entered" is not what the oracle does, and
if the shortcut is taken it is a divergence that should be recorded next to the `::CONSTANT` row Task
5 already tracks, rather than left implicit.

### F3 -- CONFIRMED. Task 4's own expected answer is produced by a Rexx method body in the oracle.

Task 4 verifies against `say .LOCAL`. Measured (`p25.rex`, rc 0): `The Local Directory`. That string
is set at `CoreClasses.orx:990`, inside `::METHOD initInstance` of the `LocalServer` class -- a Rexx
body with `expose input output error`, which the oracle reaches from
`Interpreter::initLocal` (`Interpreter.cpp:227-238`) via
`localServer->messageSend(new_string("INITINSTANCE"), ...)`, after `LOCALSERVER~NEW` runs the Rexx
`::METHOD init` (`Interpreter.cpp:207-217`). The same method body populates `STDIN`, `INPUT`,
`DEBUGINPUT`, `STDOUT`, `OUTPUT`, `STDERR`, `ERROR`, `TRACEOUTPUT` and `STDQUE` in `.local`, using
`.stream~new`, `.monitor~new` and `.RexxQueue~new`.

This is outside `CoreClasses.orx:39-126`, so it does not falsify the lens claim -- instance startup
happens after the image is loaded. It matters because Task 4's `.local` is not a thing 5a can build
the way the oracle builds it, and the plan does not say which way it will be built. If 5a hardcodes
`The Local Directory` and an empty-ish `.local`, that passes Task 4's stated check while diverging
from the oracle on `.local~allIndexes`, `.stdout`, `.output`, `.input`. Task 4 should say which, and
if it hardcodes, that belongs in `phase-4-exclusions.txt` as a KNOWN GAP with `LocalServer~initInstance`
named as what would close it.

### F4 -- CONFIRMED, in the plan's favour. Task 6's eager-bind measurement reproduces.

I re-ran it because it governs whether install can complete at all. A program whose prologue is
`say "prolog ran"` carrying `::method m class external "LIBRARY REXX no_such_entry_point_xyz"` exits
**166**, stdout **empty**, stderr `Error 90.998: Unable to find external method
"no_such_entry_point_xyz"` (`p21.rex`). The package-level `::routine` form is the same rc with
`90.999` and "external routine" (`p22.rex`). Note that `PackageClass.cpp:1247` carries the comment
"native methods and routines are lazy resolved on first use, so we don't need to process them here",
which reads as contradicting this; the measurement wins, and a task that trusts the comment instead
will build the wrong thing.

### F5 -- PLAUSIBLE. The plan's boundary list is not the enumeration it claims to be.

`||` at line 73, the second `~upper` at line 73, `.methods[...]` at line 73, and the `DO OVER`
protocol at lines 63 and 70 are all sends (or dispatch decisions) the paragraph's list omits. All
four are native, so no task changes. I label this PLAUSIBLE rather than CONFIRMED because "the list
is incomplete" is a reading of the plan, not a measurement -- the *nativeness* of each is CONFIRMED
above.

---

## What I could not reach

* An `UNKNOWN`-mediated Rexx method. I searched the prologue's receivers and none of them is
  `Monitor`, the only class in either file with `::METHOD unknown` (`CoreClasses.orx:1443`), but I
  did not enumerate `UNKNOWN` methods on the native classes from `Setup.cpp`.
* Anything that dispatches only under a non-default `TRACE`. `DoBlockComponents.cpp:229` traces the
  `OVER` value and value tracing can send `STRING`/`MAKESTRING`; the bootstrap runs untraced, so this
  is unexercised rather than absent.
* `interpreter/platform/windows/PlatformObjects.orx`. I read only the unix file, which is the one
  this build's `CALL` at line 124 resolves.
* The image-build path itself. Every runtime measurement here is against the *shipped* image, plus
  `p18.rex`, which reproduces the install-time ordering on a user program. There is no oracle
  transcript of the bootstrap (the plan's D39 is right about that), so F1's "it ran during the image
  build" rests on the two-way inference in that finding, not on a transcript.
