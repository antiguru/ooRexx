# Phase 5 — the object model

**Status:** design. Replaces `docs/superpowers/specs/2026-08-15-phase-5-object-model.md`, which is
superseded in whole and kept for its record of how the phase is easy to get wrong.
**Entry:** met. Phase 4f closed; `perf-baseline.md`'s "The pre-Phase-5 baseline" pins the standing at
`b029abe77`.
**Blocks:** Phases 6, 7 and 8, which are independent of each other and may run in parallel once this
closes. Delivery order inside Phase 5 therefore matters and is part of this document.
**Decided:** 2026-08-17. Written against `b360783cb`.

## Why there is a second spec

The first one was derived without the ooRexx documentation, which nobody had checked out. It was
reviewed hard — a five-reviewer adversarial panel, then a two-reviewer re-review, then a plan-split
review — and none of that found what one afternoon with `oodocs/` found: two of the documentation's
own object-model sections, **"Defining an UNKNOWN Method"** and **"Required String Values"**, appear
nowhere in the old spec or in the plan derived from it. Both are dispatch. Both are reachable in
surface this crate already runs. Both are, today, **silent wrong answers at rc 0 or a wrong error
rather than a loud refusal** — the failure mode every other instrument in this project exists to
prevent. Measured this session, from a fresh empty directory:

```
say .k~zork(1,2)     with  ::METHOD unknown CLASS returning "unknown:" n "args" a~items
  oracle  rc 0    stdout  unknown: ZORK args 2      stderr empty
  crate   rc 159  stdout  empty                     stderr Error 97.1 ... does not understand "ZORK"

say .k               with  ::METHOD makeString CLASS returning "K says hello"
  oracle  rc 0    stdout  K says hello              stderr empty
  crate   rc 0    stdout  The K class               stderr empty
```

The second is the worse one: **matching exit status, empty stderr on both sides, different stdout.**
No refusal names an owner, no harness reddens, and nothing in the old gate looks at it.

That is the whole argument for this document. A derivation from the design produced a plan whose
gate could not see the things the design had not thought of; a derivation from the *specification*
produces an enumeration that does not depend on having thought of them. So the enumeration below is
taken from the documentation's own structure, the phase boundaries are re-cut along the mechanisms
that structure names, and the gate is two tables whose **rows** come from the documentation and whose
**verdicts** come from running programs against both interpreters.

## Authorities, in order

1. **The documentation**, at `oodocs/` — a git-ignored, read-only checkout.
   * **`rexxref/en-US/provide.xml`, "Objects and Classes" — the specification chapter, and the
     concept floor.** Its section tree is enumerated below and is the row set of gate table C.
   * `rexxref/en-US/dire.xml` — the directive reference; the row set of gate table D.
   * `rexxref/en-US/fundclasses.xml` and its siblings `collclasses.xml`, `utilityclasses.xml`,
     `streamclasses.xml`, and the `*classmethods.xml` includes — the per-class method sets.
   * `rexxpg/en-US/classes.xml` — the Programming Guide's conceptual account. Useful, subordinate to
     the two above, and in one place (the method search order) it states something the
     implementation contradicts.
2. **The implementation** — `/home/moritz/dev/repos/ooRexx/interpreter/`, read-only. Authority for
   what it *does*. This project reproduces it byte for byte, so **where the documentation and the
   implementation disagree, the implementation wins and the disagreement is a row in
   [its own section](#documented-versus-implemented)**.
3. **The test suite** — `ootest/`, a read-only SVN checkout of another project, git-excluded.
   Authority for what upstream pins, which is the third signal in the plan's rule for declaring an
   oracle defect.

**None of the three is a build dependency.** `oodocs/` and `ootest/` are checkouts on a developer's
machine; CI's five platforms have neither. Everything derived from them is **committed as a data
file naming the upstream revision it was derived at**, exactly as `corpus/keyword-exempt.txt` is
("Measured at ooTest r13178"), with the extractor committed beside it. See
[D56](#new-decisions).

## The concept floor

`provide.xml`'s chapter "Objects and Classes" has this section tree, read out of the XML rather than
from a reading of the prose:

```
chapter provide — Objects and Classes
  section typcla   — Types of Classes
    objcla   — Object Classes
    xmixin   — Mixin Classes
    abscla   — Abstract Classes
    xmetac   — Metaclasses
  section xcremet  — Creating and Using Classes and Methods
    usingcl  — Using Classes
    xscope   — Scope
    usesem   — Defining Instance Methods with SETMETHOD or ENHANCED
    methna   — Method Names
    xmeths   — Default Search Order for Method Selection
    unkno    — Defining an UNKNOWN Method
    chsrod   — Changing the Search Order for Methods
    pubpri   — Public, Package-Scope, and Private Methods
    creo     — Initialization
    obdes    — Object Destruction and Uninitialization
    reqstr   — Required String Values
    concurr  — Concurrency
  section classmeth — Overview of Classes Provided by Rexx
    chi              — The Class Hierarchy
    methodsbyclass   — Class Library Notes
```

Every one of those ids is a row of gate table C. Two of them — `unkno` and `reqstr` — have no
counterpart anywhere in the superseded spec or its plan; `usingcl`, `usesem` and `methodsbyclass` are
named in neither the spec nor the correlation review's flattened list of the chapter.

### The enumeration

One row per mechanism, each with the authority that pins it, where the implementation puts it, the
phase this spec assigns it, and what it does **today**, measured. "silent" means the two interpreters
agree on exit status and stderr and disagree on stdout; that is the class this table exists to make
visible.

| mechanism | authority | implementation | phase | today |
|---|---|---|---|---|
| object / mixin / abstract / metaclass as four kinds | `provide.xml` `typcla`,`objcla`,`xmixin`,`abscla`,`xmetac` | `ClassClass.hpp:176-186` flags | 5a (`ABSTRACT` enforcement 5b) | partial |
| a class carries two behaviours, class-side and instance-side | `dire.xml` `clasdi` (metaclass merge position); D44 | `createClassBehaviour :1119`, `createInstanceBehaviour :1148` | 5a | built (Task 2) |
| the metaclass graph and its circularity | `provide.xml` `xmetac` | `RexxClass::createInstance :1854`, `buildFinalClassBehaviour :654` | 5a | built (Task 3) |
| a mixin's base class, and who may inherit it | `provide.xml` `xmixin` | `mixinClass() :1514`, `inherit() :1322` | 5a | refused |
| merge order: a class's own methods precede its superclasses' **and its mixins'** | `provide.xml` `xmeths`; `fundclasses.xml` `mthClassInherit` | `createInstanceBehaviour` merges own dictionary after the superclass loop | 5a | refused |
| merge order among several `INHERIT`s — leftmost first | `dire.xml` `clasdi` | reverse walk + `addFront` | 5a | refused |
| method dictionary: one entry per scope, `addFront`, scope list and scope orders | `provide.xml` `xscope` | `MethodDictionary::addMethod :164`, `addScope :594`, `findSuperMethod :434` | 5a | built (Task 2) |
| `~define` copies the behaviour, `~inherit` mutates it in place | `fundclasses.xml` `mthClassDefine`/`mthClassInherit`; D43 | `defineMethod :819`, `inherit :1287` | 5a | built (Task 2) |
| `inheritInstanceMethods` — donation with no superclass edge | image-build only; `Setup.cpp:1809` removes it | `ClassClass.cpp:558` | 5a | not built |
| the cascade, instance-side vs both sides | D44 | `updateInstanceSubClasses :1071`, `updateSubClasses :1036` | 5a | built (Task 2) |
| **the REXX_DEFINED lock** | `fundclasses.xml` `mthClassDefine`, `mthClassInherit` | `liveGeneral :134` under `PREPARINGIMAGE`; raises 98.985 in `define`, `defineMethods`, `delete`, `inherit`, `uninherit` | 5a | not built |
| `~define` / `~defineMethods` / `~delete` / `~uninherit` / `~enhanced` | one `fundclasses.xml` section each | `ClassClass.cpp:819,518,952,1379,1440` | 5a (`~enhanced` 5b — it builds an instance) | partial |
| **`::CLASS` option surface** — `METACLASS PUBLIC PRIVATE SUBCLASS MIXINCLASS INHERIT ABSTRACT` | `dire.xml` `clasdi`, `<option>` + indexterms | `classDirective` `DirectiveParser.cpp:334-490` | 5a | `SUBCLASS`/`PUBLIC`/`ABSTRACT` built; rest refused |
| **`::METHOD` option surface** — `ATTRIBUTE CLASS PUBLIC PACKAGE PRIVATE GUARDED UNGUARDED PROTECTED UNPROTECTED ABSTRACT DELEGATE EXTERNAL` | `dire.xml` `methd` | `methodDirective :629-812` | 5a (`DELEGATE` 5b) | partial |
| **`::ATTRIBUTE` option surface** — the `::METHOD` set plus `GET` and `SET`, and the rule that `GET`/`SET` may carry a body overriding the generated one | `dire.xml` `attrd` | `attributeDirective :1457-1850` | 5a (`DELEGATE` 5b) | partial |
| **`::CONSTANT` creates an instance method *and* a class method** | `dire.xml` `constantd` | `createConstantGetterMethod :2518` | 5a | not built — no readable method at all |
| `::CONSTANT`'s parenthesised expression runs at install with `self` bound to the class | `dire.xml` `constantd`; `ClassDirective::resolveConstants :243` calling `setScope` | second install pass | 5a | evaluated, not readable |
| `::CONSTANT` forward references to calculated constants are refused; a floating one may not use the parenthesised form | `dire.xml` `constantd` | `resolveConstants` | 5a | the floating rule agrees (99.906) |
| **`::ANNOTATE`'s six targets and `~annotation`/`~annotations`** | `dire.xml` `annotd` indexterms `ATTRIBUTE CLASS CONSTANT METHOD PACKAGE ROUTINE`; `fundclasses.xml` `mthClassAnnotation(s)` | `annotateDirective :1940`, `RexxClass::setAnnotations :343` | 5a | **over-refused** — see [the false claim](#the-annotate-claim) |
| **install is three passes over a dependency-ordered class list; a cycle is 98.911** | `dire.xml` `clasdi` example "CLASS directive deferred processing"; `fundclasses.xml` `mthClassActivate` | `processInstall :1268-1298`, `resolveDependencies :1801` | 5a | ordering and cycles built (Task 8); the three passes are not |
| **class-object initialization: `INIT` at construction, then `INHERIT`, then `ACTIVATE`** | `fundclasses.xml` `mthClassActivate` (explains `ACTIVATE` *by contrast with* `INIT`) | `RexxClass::subclass :1631` sends `INIT`; `ClassDirective::activate()` in pass three | 5a | neither built |
| floating `::METHOD`/`::ATTRIBUTE`/`::CONSTANT` reach `.METHODS` | `dire.xml`, once per directive | `LanguageParser::addMethod :610` | 5a | built (Task 6) |
| **the complete method search order: per-object, own class, superclasses, `UNKNOWN`, NOMETHOD** | `provide.xml` `xmeths` and `unkno` | `messageSend :866`, `processUnknown :1002` | 5a (per-object arm 5b) | steps 2-3 built; **`UNKNOWN` missing, silently wrong** |
| **changing the search order — `~m:scope`, `~m:super`** | `provide.xml` `chsrod` | `messageSend :919` → `superMethod`; `validateScopeOverride :1950` | 5a | built (Task 5) |
| `SELF`, `SUPER` | `rexxpg/classes.xml` `spvar` | `Setup.cpp:325-329` | 5a | built (Tasks 5, 7) |
| **`PUBLIC` / `PACKAGE` / `PRIVATE` as three access scopes** | `provide.xml` `pubpri` | `checkPrivate :609`, `checkPackage :659` | 5a | `PRIVATE` refused; `PACKAGE` agrees in the same-package case |
| `PROTECTED` routes the send through the security manager | `provide.xml` `pubpri` | `processProtectedMethod :976` | 5a seam (D45); semantics with the manager | agrees today |
| scope-keyed instance variables, `EXPOSE`, lazy creation | `provide.xml` `xscope`; D40 | `getObjectVariables :2489` | 5a | built (Task 8) |
| class-scope instance variables | `provide.xml` `xscope`; `TraceObject~activate` | same chain on the class object | 5a | built (Task 8) |
| **Required String Values — `request("STRING")` → `makeString` → NOSTRING → `defaultName`** | `provide.xml` `reqstr`, which lists every context | `requestString :1235`, `requestStringNoNOSTRING :1302`, `defaultName :1760` | 5a | **`defaultName` limb only; `makeString` limb silently wrong** |
| `~objectName`, `~objectName=`, `~string`, `~request` | `fundclasses.xml`, one section each | `ObjectClass.cpp:1696,1733,1760` | 5a | `~objectName=` planned; the rest rc 120 |
| **the `Object`/`Class` native protocol** — `~class`, `~id`, `~superClass`, `~superClasses`, `~metaClass`, `~isA`, `~isSubclassOf`, `~hasMethod`, `~method`, `~copy`, `~identityHash` | `fundclasses.xml` `clsObject`, `clsClass` | `ObjectClass.cpp`, `ClassClass.cpp` | 5a (`~copy` 5b) | only `~hasMethod` answers |
| `.environment`, `.local`, `.context`, `.methods` as objects | `rexxpg/classes.xml` `pubobj`; D33 | `Setup.cpp`, `DirectoryClass` | 5a | built (Task 6) |
| **Directory entry methods** — `.environment~local` answers while `hasMethod("LOCAL")` is 0 | measured; `Setup.cpp:1781` via `setMethodRexx` | `DirectoryClass::setMethodRexx :480`, `unknownValue :591` | 5a | not built |
| **the eight-step environment-symbol search order, including the package-local directory** | `rexxpg/classes.xml` `searchord` | `PackageClass::findClass :1086` | 5a; steps 3 and 5 with 5c | steps 2 and 7 built (Task 6) |
| `.Package` — `addClass`, `addPublicClass`, `publicClasses`, `~name`, `~local`, install | `fundclasses.xml` `clsPackage` | `PackageClass.cpp:1401,1432,1567,1227` | 5a; `~local` 5c | not built |
| the native entry-point registry for `EXTERNAL 'LIBRARY REXX name'` | D37 | eager bind at install | 5a | not built |
| **Initialization of an instance — `~new`, `init`, `self~init:super`** | `provide.xml` `creo` | `completeNewObject :1882` | **5b** | refused |
| **Object Destruction and Uninitialization — `UNINIT` and its propagation flags** | `provide.xml` `obdes`; `rexxpg/classes.xml` `uninit` | `checkUninit :1210`, `ObjectClass.cpp:2579,2604`; flags propagate through `subclass`/`mixinClass`/`inherit` | **5b**, flags carried by 5a's code | not built |
| **per-object methods — `SETMETHOD` and `ENHANCED`, and the scope they create** | `provide.xml` `usesem` | `ObjectClass.cpp:1829,1891`; `checkRestrictedMethod :697` | **5b** | refused |
| `FORWARD`, and therefore `DELEGATE` | `provide.xml` `creo` uses `FORWARD` to define multi-`INIT`; `dire.xml` defines `DELEGATE` as `expose`+`forward to()` | `RexxInstructionForward`; `createDelegateMethod :2438` | **5b** | refused |
| abstract-method and abstract-class enforcement | `provide.xml` `abscla` | `makeAbstract :1754`, `checkAbstract :1741` | **5b** (needs `~new`) | installs, unenforced |
| `~run`, `~send`/`~sendWith`, `~start`/`~startWith` | `fundclasses.xml` `clsObject` | `ObjectClass.cpp:2185` and neighbours | **5b** (`~start`'s concurrency Phase 6) | not built |
| `::REQUIRES` `LIBRARY NAMESPACE`, and namespace-qualified class references `ns:Class` | `dire.xml` `requ`; `clasdi` calls each class name an "optionally-qualified symbol" | `requiresDirective :2779`, `parseClassReference :287` | **5c** | refused |
| `::OPTIONS` and the `OPTIONS` instruction | `dire.xml` `optionsd` | `optionsDirective :948` | **5c** | refused |
| `::RESOURCE`, `.RESOURCES` | `dire.xml` `resourced` | `resourceDirective :2266` | **5c** | installs |
| `::ROUTINE` `EXTERNAL PRIVATE PUBLIC`, `.ROUTINES` | `dire.xml` `routd` | `routineDirective :2565` | **5c** | partial |
| `Package~local`, and the package-local directory as a search step | `fundclasses.xml` `clsPackage`; `rexxpg/classes.xml` `searchord` | `PackageClass` | **5c** | not built |
| **the documented per-class method sets** | `fundclasses.xml`, `collclasses.xml`, `utilityclasses.xml`, `streamclasses.xml`, `*classmethods.xml` | everywhere | **5c** | the acceptance set of gate table C |
| `GUARDED`/`UNGUARDED`, `REPLY`, `GUARD` legality | `provide.xml` `concurr`; `dire.xml` `methd`; D32 | `RexxInstructionReply::execute` | 5a legality, **Phase 6** semantics | over-refused inside a method |
| the `*-* Compiled method "X" with scope "Y".` traceback line | every error transcript through a method frame | error path | 5a | native-send half built (Task 5); the operator half is open |
| `>M>` and `>N>` trace prefixes | `trace_oracle.rs` `PREFIX_COVERAGE` | `traceMessage` | 5a (`>M>` landed), `>N>` with class resolution | partial |

## The mechanism boundaries

Three phases, as decided. Each boundary is a **line between documented mechanisms**, and the reason
it falls there is that no single documented mechanism crosses it.

### 5a — a class exists and answers a message

Everything from a `::CLASS` directive to a method body running, and the objects the install path
needs. It is one mechanism chain because **installing a directive runs Rexx code** and because the
documented method search order is a single ordered list that cannot be split without leaving a step
unowned.

Contents: class construction and the whole class graph; the option surface of `::CLASS`, `::METHOD`,
`::ATTRIBUTE`, `::CONSTANT` and `::ANNOTATE`, minus `DELEGATE`; the three install passes over the
dependency-ordered class list, including **class-object initialization end to end** (`INIT`, then the
`INHERIT` merge, then `ACTIVATE`); the complete method search order including `UNKNOWN` and the
NOMETHOD condition; search-order override; the three access scopes and `PROTECTED`'s seam;
scope-keyed instance variables and `EXPOSE`; the required-string protocol; the four environment
objects, the Directory entry-method mechanism and the environment-symbol search order minus its two
package-crossing steps; the Package object and the class-graph mutators including the REXX_DEFINED
lock; the native entry-point registry.

**Why the line is here and not further back.** The superseded plan put `ACTIVATE` in 5a and the class
object's `INIT` in 5b. `fundclasses.xml` explains `ACTIVATE` by contrast with `INIT` —

> "Because the INIT method is called early in the class construction process, only limited class
> initialization is possible at that time. The activate method is the preferred method for
> initializing a class object."

— and the contrast is the specification. Measured this session, on `::CLASS M MIXINCLASS Object` plus
`::CLASS K INHERIT M` carrying both:

```
oracle rc 0:   K init,     hasMethod MM = 0
               K activate, hasMethod MM = 1
               prologue
crate  rc 120: ::CLASS MIXINCLASS is not implemented (Phase 5)
```

`INIT` fires **before** the `INHERIT` merge and `ACTIVATE` after it. A phase that owns one and not the
other owns neither: the discriminator is the pair.

**Why the line is here and not further forward.** Instance creation is a separate documented
mechanism with its own section, and nothing 5a needs reaches it — established by reading, not
assumed. The only Rexx bodies either `.orx` file runs at install are `TraceObject`'s
`::method activate class` (`CoreClasses.orx:3996`) and the two parenthesised `::CONSTANT`s at
`StreamClasses.orx:548`-`549`. Neither sends `~new`.

The derivation, stated because the first version of it was wrong: **case-insensitively**,
`^[[:space:]]*::constant[[:space:]].*\(` over `CoreClasses.orx`, `StreamClasses.orx` and
`platform/unix/PlatformObjects.orx` returns those two `StreamClasses.orx` lines and nothing else, and
`^[[:space:]]*::(method|attribute)[[:space:]]+["']?(init|activate)["']?` returns that one `activate`
line plus a family of bare `::METHOD init` lines — eleven in `CoreClasses.orx`, four in
`StreamClasses.orx` — **none of which carries `CLASS`**, so every one of them is an *instance* `init`
that runs at `~new` and not at install. A case-**sensitive** version of the same pair of greps sees
one `init` line instead of fifteen and would have supported the same conclusion by luck. And the
check for `CLASS` on those fifteen lines has to run over `cat`ed content, not over a multi-file
`grep`: the filename prefixes are `CoreClasses.orx` and `StreamClasses.orx`, so a second `grep -c
class` down the pipe answers **fifteen** for the filenames rather than **zero** for the directives.
The tables in [the gate](#the-two-gate-tables) are derived by the same kind of extraction and inherit
the same hazard, which is why their verdicts come from running programs and not from matching text.

### 5b — an instance exists

Instance construction and the whole of instance initialization (`~new`, `init`, `self~init:super`
chaining); Object Destruction and Uninitialization; per-object methods (`SETMETHOD`, `ENHANCED`,
`unsetMethod`) and the object-own scope they create; `FORWARD` and therefore `DELEGATE`;
abstract-class enforcement, which is a check inside `~new`; `~copy`; the alternative invocation paths
`~run`, `~send`/`~sendWith`, `~start`/`~startWith` minus their concurrency; and **the instance-side
reading of every 5a limit that could only be measured on a class object**, which Task 7 already
recorded as a debt.

**Why the line is here.** Every item is a mechanism whose first argument is an instance, and none of
them is reachable at all without `~new`. `DELEGATE` is here rather than with its directive because
`dire.xml` defines it *as* `expose` plus `forward to()`, and the documented equivalence is the
specification; splitting `DELEGATE` from `FORWARD` would be the `ACTIVATE`/`INIT` mistake again.

### 5c — the package, and the class library

`::REQUIRES` with `LIBRARY` and `NAMESPACE`, and therefore namespace-qualified class references on
`METACLASS`/`SUBCLASS`/`MIXINCLASS`/`INHERIT`; `::OPTIONS` and the `OPTIONS` instruction; `::RESOURCE`
and `.RESOURCES`; `::ROUTINE`'s option surface and `.ROUTINES`; `Package~local` and the two steps of
the documented environment search order that cross a package boundary; and **the documented per-class
method sets answering**, which is the acceptance set Phases 6, 7 and 8 enter on.

**Why the line is here.** Everything in 5c is a mechanism whose unit is a *package* rather than a
class or an object — `dire.xml`'s remaining four directives, `fundclasses.xml`'s Package Class, and
`rexxpg`'s search-order steps 3 and 5. The superseded spec's reason for deferring `::REQUIRES` was
"the bootstrap does not use it", which is an ordering and not a boundary; the mechanism reason
reaches the same place and also collects `::OPTIONS`, `::RESOURCE`, `NAMESPACE` and `Package~local`,
which the old cut left with no owner at all.

### The bootstrap is a milestone, not a boundary

Running `CoreClasses.orx`, `StreamClasses.orx` and `PlatformObjects.orx` to their prologue's `exit`
is a **milestone inside 5a**, because 5a is the phase that completes every mechanism they need — the
reading in the previous section is the evidence. It keeps everything the superseded plan's Tasks 10
to 14 do: the driver, the Package argument, the embedded files, the sha256 drift check, the
post-bootstrap state checks.

What changes is its standing. It is not the definition of a phase boundary, and it is not the gate.
`CoreClasses.orx` is one file's exercise of a language, and a phase scoped to what it happens to
touch leaves `UNKNOWN`, Required String Values, Object Destruction, `PACKAGE` and `DELEGATE` owned by
nobody — measured: each is absent from the superseded spec and its plan, and each is documented.

## The class-set criterion, which replaces "32 classes"

`2026-07-27-rust-rewrite.md:453` gives Phase 5 the exit clause *"32 classes exist and respond"*.
Measured: `/bin/grep -acE "^::[Cc][Ll][Aa][Ss][Ss]" CoreClasses.orx` is **32**, and the number is that
grep. Measured on the shipped oracle, iterating `.environment` and counting entries that answer
`~isA(.Class)`: **62**. So the criterion is satisfiable with a third of the environment missing, and
it is satisfiable by a build that never runs `StreamClasses.orx`.

**The criterion is replaced by the documented class set.** `provide.xml`'s "The Class Hierarchy"
(`chi`) is an indented `<simplelist>` inside a `$GENERATED` block, one `<member>` per class, five
`&nbsp;` per level of indentation, each carrying `linkend="clsX"` — mechanically extractable, and its
indentation states one superclass-or-mixin edge per class. The wider set is every `<section id="clsX">`
across `fundclasses.xml`, `collclasses.xml`, `utilityclasses.xml` and `streamclasses.xml`.

Diffed against the shipped oracle's `.environment` this session:

| | |
|---|---|
| documented as a class **and** an `.environment` class | everything not named below |
| documented, not an `.environment` class | **`RexxInfo`** — the `.environment` entry is an *instance*: `say .RexxInfo` is `a RexxInfo`, `say .RexxInfo~class` is `The RexxInfo class`, and `EndSpecialClassDefinition(RexxInfo)` (`Setup.cpp:1285`) routes the class to `addToSystem`, whose target is not reachable as an environment symbol — `.environment~entry("SYSTEM")` is `The NIL object`. **`RegularExpression`** — documented in `utilityclasses.xml`, delivered by `::requires "rxregexp.cls"`, and that file is not on this build's search path: measured `43.901 Could not find file "rxregexp.cls" for ::REQUIRES`, rc 213 |
| an `.environment` class, documented nowhere | **`ArgUtil`**, and `provide.xml` carries the reason as an XML comment beside the hierarchy: `<!-- commented ArgUtil (deprecated, won't document) -->` |
| documented in a `cls*` section but absent from the hierarchy list | **`EventSemaphore`**, **`MutexSemaphore`**, **`Singleton`**, `RegularExpression` |

**So the criterion is:** every class named by a `cls*` section across the reference, minus
`RegularExpression` (no library on this build's path) and minus `RexxInfo`-as-an-environment-entry
(whose `.environment` entry must be an instance whose `~class~id` is `RexxInfo`), plus `ArgUtil`, is
an `.environment` class; each answers `~id`, `~class`, `~superClass`, `~superClasses`, `~metaClass`
and `~isA(.Class)` byte-identically to the oracle; each documented edge in the hierarchy list appears
in that class's `~superClasses` answer; and each class answers its documented method set under the
readback rule below. That is gate table C's class half, and it is the Phase 5 exit criterion the
roadmap row is amended to.

**The correlation review's version of this diff recorded four disagreements and 59 agreeing names.**
Re-derived here it is five and 58: `ArgUtil` was missed, and the trio said to be "in the image, absent
from the documented hierarchy" *is* documented — in `utilityclasses.xml` — so the disagreement is
between two parts of the same book rather than between the book and the image.

## The two gate tables

Both are `#[test]`s, so `cargo test --release --workspace` — already a gate command — runs them.
**Every row's verdict is produced by running a program. Every row carries an oracle column.** A table
whose crate column is read out of `directive_gap`'s arms is a restatement of the source in a second
place and catches nothing; `corpus/keyword-exempt.txt` is the precedent that works, and it works
because its `unblocked_by` column is "re-derived on every run" from executed bodies, as its own
header says.

Three verdicts, computed and never written: **agree**, **loud** (the crate refuses at rc 120 naming a
phase), **diverge** — and `diverge` is split into `diverge-loud` (exit status differs) and
**`diverge-silent`** (exit status and stderr match, stdout does not). `diverge-silent` is the verdict
the superseded gate had no way to express and the reason both `UNKNOWN` and `makeString` survived
three reviews.

### Table D — the directive and option surface

**Row set**, derived and committed:

* from `dire.xml`, per `<section id="...">` under the `dire` chapter, the union of that section's
  `<option>NAME</option>` occurrences and its `<indexterm><primary>NAME subkeyword</primary>` entries;
* from `DirectiveParser.cpp`, per directive function, the `SUBDIRECTIVE_*` arms of its own `switch`.

The union is the row set; **a name in one source and not the other is a row of its own**, marked with
which side produced it, because that asymmetry is where the two authorities disagree. Two derivation
artifacts must be handled explicitly rather than silently absorbed, both found while writing this:

* the markup does not distinguish an option from an option's *value*. `::OPTIONS`' `<option>`
  occurrences include `ENGINEERING`, `SCIENTIFIC`, `INHERIT` and `NOINHERIT`, which `dire.xml`'s prose
  says are values of `FORM` and of `NUMERIC` respectively. The row key is therefore
  **(directive, keyword, position)** where position is `subkeyword` or `value-of(subkeyword)`, and the
  parser's nesting is what settles it;
* a section's `<option>` set picks up cross-references to *other* directives' options — `::CLASS`'s
  set contains `CLASS` because its `METACLASS` paragraph describes `::METHOD`'s `CLASS` option. A row
  whose keyword the directive's own parser has no arm for is marked `cross-reference` and carries the
  parser evidence for the marking.

**Columns per row:** the probe program's path; oracle rc, stdout and stderr; crate rc, stdout and
stderr, **on both engines**; the verdict; and which of `CoreClasses.orx` / `StreamClasses.orx` uses
the shape, derived by scanning both files rather than asserted.

**The probe corpus is the row set's own obligation.** One committed program per row, under
`rust/corpus/`, and a row without one fails the table — not "is skipped". Shapes the file scan must
reach, each already known to exist: both keywords on one directive with mixin targets
(`StreamClasses.orx:115`), a quoted class target (`:371`), an install-time `::CONSTANT` sending a
private class method of its own class (`:546`-`:549`), and a directive carrying its body on the same
physical line (`CoreClasses.orx:151`).

**How it fails in the direction that matters.** Three named mutations, each of which the table's own
test asserts flips a stated row:

1. make the crate accept a still-refused keyword silently — the row moves `loud` → `agree` **only if
   the oracle agrees too**; if it does not, the row reads `diverge-silent` and reddens. This is the
   `SUBCLASS`/`MIXINCLASS` shared-slot hazard the plan-split review filed as M10: `ast.rs`'s
   `ClassDirective` holds both in one `subclass` slot separated by `mixin: bool`, so a narrowing
   written against `subclass.is_some()` admits `MIXINCLASS` at rc 0. A refusal-derived table agrees
   with that bug. This one does not, because the `MIXINCLASS` row's probe asks a question a plain
   subclass answers differently — measured, `::CLASS M MIXINCLASS Object` gives `.M~baseClass` =
   `The Object class` while `::CLASS P` gives `.P~baseClass` = `The P class`, both at rc 0 and
   neither needing `~new`.
2. make a row's expected oracle bytes wrong — the row reddens, because they are re-read from the
   oracle on every run and never typed.
3. delete a row's probe — the table reddens on the missing program rather than shrinking.

### Table C — the concept and class surface

**Row set**, derived and committed, in two halves.

*Concepts.* One row per `<section id>` under `provide.xml`'s `provide` chapter, at every nesting
level — the tree printed above. Each row names the corpus program that exercises the mechanism and
carries the same measured columns as table D. A section with no program is a **failing** row.
This half is what would have caught `unkno` and `reqstr`: neither is a directive keyword, so table D
cannot see either, and the superseded gate had only table D's shape.

*Classes and methods.* One row per (class, method) pair, derived from the `cls*`/`mth*` section
structure of `fundclasses.xml`, `collclasses.xml`, `utilityclasses.xml`, `streamclasses.xml` and the
`*classmethods.xml` includes, plus one row per class for the six wiring answers and per documented
hierarchy edge.

**The readback rule, measured this session, because getting it wrong makes the table unbuildable:**

| what to ask | how | measured on the oracle |
|---|---|---|
| does class X have documented **instance** method m | `.X~method("M")` | `.Array~method("APPEND")~class` is `The Method class`; `.Array~method("ZZNOSUCH")` raises 97.1 |
| does class X have documented **class** method m | `.X~hasMethod("M")` | `.Array~hasMethod("OF")` is 1; `.Array~hasMethod("APPEND")` is 0 |

`~method` reads `instanceMethodDictionary`, the methods "defined at this level"
(`ClassClass.cpp:984`, comment at `:988`-`:990`), so it is per-scope and discriminating: measured,
`.Array~method("STRING")` **raises 97.1** although every Array instance answers `STRING`, while
`.String~method("COMPARETO")` answers. A build that flattened every scope onto one class would answer
the first and redden.

**`~instanceMethods` is not the readback**, and the measurement that says so is worth committing
beside the rule: `.Array~instanceMethods` collected into a Set contains `DEFINE`, `ID` and `OF` and
does **not** contain `APPEND` — it answers about the receiver, which is a class object, not about
instances of it. `.Array~instanceMethods(.nil)` is empty. Either would have produced a table that
looked derived and measured the wrong thing.

Which half of the pair a documented method belongs to is derivable: `fundclasses.xml` and its
siblings suffix a method's `<title>` with `(Class Method)`, `(Abstract Method)`, `(Private Method)`
or `(Attribute)` where it applies, and the suffix is systematic — with one case wobble (`Class
method` occurs beside `Class Method`) and one `(Inherited Class Method)`, so the match is
case-insensitive and the four unusual titles are pinned by name in the extractor.

**How it fails in the direction that matters.**

1. Drop a class from the registry — its wiring row cannot produce an answer and reddens.
2. Answer a class's methods from a flattened all-scopes dictionary — `.Array~method("STRING")`
   answers where the oracle raises, and that row reddens.
3. Implement a section's mechanism wrongly but silently — the concept row's program is compared on
   three descriptors, so `makeString` returning the wrong string reddens at rc 0.
4. Delete the `UNKNOWN` step from dispatch — the `unkno` row reddens. Delete the `makeString` limb —
   the `reqstr` row reddens. Both of those are the negative controls that prove the two rows this
   spec exists for are live, and both must be recorded as having been run.

### What the gate must also cover, which the old one did not

* **Dispatch, as a protocol.** Every step of `provide.xml`'s search order gets a row, including the
  `UNKNOWN` step and the NOMETHOD condition beneath it, and the access-scope checks that sit between
  lookup and invocation. `::METHOD unknown` is reachable on a class object with no `~new`, measured
  above, so this is 5a's and not deferrable.
* **String conversion, as a protocol.** `provide.xml` `reqstr` names its contexts — `SAY`; `DO`'s
  `exprr`/`exprf`; substituted compound-variable tails; commands and `ADDRESS`; `ARG`/`PARSE`/`PULL`;
  parenthesised `CALL` targets; `DROP`/`EXPOSE`/`PROCEDURE` lists; `INTERPRET`; `NUMERIC`'s three
  values; `OPTIONS`; `PUSH`/`QUEUE`; `SIGNAL VALUE`; `TRACE VALUE`; builtin arguments; and dyadic
  operators with a string on the left — plus the two different rules for method arguments (String's
  arithmetic/comparison/concatenation methods fall back to `~string`; every other method raises).
  **Every one of those is a dispatch site once objects exist**, and each gets a row. The crate already
  has a partial answer to this question in `eval.rs`'s `object_operand_tests` and in
  `corpus/lang/message_send_argument_object_not_a_string.rex`; those were derived from a coercion
  audit rather than from `reqstr`, and re-deriving them from `reqstr` is a migration item.
* **Superclass-versus-mixin merge order**, which is the one merge rule the bootstrap depends on and
  which no probe in the superseded plan discriminates — it asks for two mixins and a diamond, and both
  are green under an implementation that gets superclass-versus-mixin backwards. `provide.xml`
  `xmeths` states the rule ("places methods of a class **before** methods of its superclasses") and
  `fundclasses.xml`'s `inherit` states its consequence ("Inherited methods can take precedence **only
  over** methods defined at or above the base class"). Measured, and **reachable at class level with
  no `~new`**, which the plan's instance-shaped version of this probe is not: with `::CLASS P`,
  `::CLASS M1 MIXINCLASS Object` and `::CLASS K SUBCLASS P INHERIT M1` each carrying
  `::METHOD m CLASS`, `say .K~m` is **`parent`** at rc 0 — not `M1`. The mixin winning is its negative
  control. It is load-bearing rather than academic: `CoreClasses.orx:93` is
  `.string~inherit(.Comparable)`, and `Comparable`'s `compareTo` (`:1300`, under
  `::CLASS 'Comparable' MIXINCLASS Object Public` at `:1299`) is a real
  `(self~identityHash - other~identityHash)~sign` body rather than `ABSTRACT`, so if the mixin won,
  `'ab'~compareTo('b')` would be an identity-hash sign; measured it is **`-1`**. (The correlation
  review cites `:92` for the `inherit` line; read at the file it is `:93`.)

## Documented versus implemented

Five, each measured this session. In every one the implementation wins.

| the documentation says | the implementation does | consequence |
|---|---|---|
| `provide.xml` `xmeths`: an object acquires its class's methods "at the time of its creation", and superclass methods are "limited to methods that were available when the object was created". `rexxpg/classes.xml` `meths` says the same | a **flat dictionary built at class-definition time**; `~inherit` rebuilds it in place and **is** visible to an existing instance. Measured: with `o = .K~new` then `.K~inherit(.M)`, `o~fromMixin` is `yes` at rc 0; with `.K~define(...)` instead, `o~later` is 97.1 | D29 and D43 follow the implementation. `fundclasses.xml`'s `inherit` section ("Any subsequent change to the instance methods of classobj takes immediate effect") agrees with the C++; the two search-order sections are the ones that are wrong |
| `dire.xml` `clasdi`: "If you specify INHERIT, it must be the last option" | not a syntax check. The `INHERIT` arm consumes to end of clause, so `::CLASS K INHERIT M PUBLIC` is rc 158, `98.909 Class "PUBLIC" not found` | the phase owes that error shape, not a translation-time refusal |
| `provide.xml` `chi` lists `RexxInfo` as a class under `.Object` | `.RexxInfo` in `.environment` is an **instance**; the class object is not an environment entry | the acceptance set carries `RexxInfo` as an instance row, not a class row |
| `provide.xml` `chi` omits `EventSemaphore`, `MutexSemaphore`, `Singleton`, `RegularExpression` | all four have `cls*` sections in `utilityclasses.xml`; the first three are `.environment` classes | the acceptance set is the union of the `cls*` sections, not the hierarchy list; the hierarchy list supplies edges only |
| nothing documents `ArgUtil`, deliberately — `provide.xml` comments it out as "deprecated, won't document" | `.ArgUtil` is an `.environment` class object; `.ArgUtil~class` is `The Class class` | the acceptance set carries it as an undocumented-but-present row with the comment as its citation |

<a id="the-annotate-claim"></a>
### The `::ANNOTATE` claim, which is false where it is generalised

The superseded spec, and Task 4 of the plan derived from it, say: *"`::ANNOTATE naming a target` is
not in scope: the oracle refuses it too (99.945, rc 157), so it is not an over-refusal and retiring it
closes no divergence."* Measured this session:

```
::ROUTINE r ; return 1        with  ::ANNOTATE ROUTINE r author "moritz"
  oracle  rc 0    stdout "prologue"   stderr empty
  crate   rc 120  stderr  rexx-exec: ::ANNOTATE naming a target is not implemented (Phase 5)
```

99.945 is the **unknown-target-name** case only. `dire.xml` documents six target types with a worked
example, `fundclasses.xml` documents `~annotation`/`~annotations` to read them back, and
`ootest`'s `ANNOTATE.testGroup` has a passing test per target. So it is a live over-refusal and
retiring it closes a divergence.

Three places in the tree carry the generalisation and each needs a different fix:

* the superseded spec — superseded here;
* `docs/superpowers/plans/2026-08-15-phase-5a-native-layer.md`, Task 4 — the plan that follows this
  spec rewrites it;
* `rust/crates/rexx-exec/src/lib.rs:1275`-`1277`, the arm comment. Its sentence is *"Resolves its
  target against the accumulated package: measured, `::annotate routine nosuchrtn` is 99.945 rc 157"*
  — true of the program it names, and placed where a reader takes it as the reason the whole family
  is refused.

`docs/superpowers/plans/phase-4-exclusions.txt:420` also carries `::annotate routine nosuchrtn` under
"REFUSED BY THE ORACLE BEFORE main". **That line is a true measurement of the program it names** and
is not the false statement; what it is missing is the sibling row for a valid target, whose absence is
why the file reads as though the family were settled. The plan adds that row.

## D25–D45 disposition

Every decision, one row, no gaps.

| | disposition | why |
|---|---|---|
| **D25** — native/sourced line discovered by running `CoreClasses.orx`; `Setup.cpp` a checklist; wiring asserted through `~class`, `~superClass`, `~superClasses`, `~isA`, `~metaClass` | **amended** | The five wiring questions stand and gain `~id` and the per-class documented method set. The *enumeration* no longer comes from running the file: it is the documented class set above. `/bin/grep -aE "createInstance\(\)"` over `Setup.cpp` — named in criterion 7 as "the enumeration's definition" — yields **C++ type names**, measured: `RexxObject`, `RexxString`, `IdentityTable`, `NumberString`, `WeakReference` and the rest, of which `RexxInfo`, `RexxInteger` and `NumberString` end in `EndSpecialClassDefinition` and are not `.environment` classes at all. Discovery-by-running remains as a supplement, not the definition |
| **D26** — the three `.orx` files executed from this repository's tracked copy, never edited; no image built; D2 measured at exit | **carried** | |
| **D27** — new crates `rexx-classes` and `rexx-lib`; the boundary is a trait; `resolve` takes a start scope and a per-object table | **amended** | `rexx-classes` exists and holds. `resolve`'s inputs widen: the `UNKNOWN` step needs the receiver's own dictionary consulted after the miss, and `PRIVATE`/`PACKAGE` need the **caller's scope and the caller's package**, neither of which the old signature carries. `rexx-lib` does not exist yet |
| **D28** — resolution dynamic, no per-call-site cache; amends D24 | **carried** | |
| **D29** — dictionary flattened at class-definition time, scope ordering retained, cascade, monotonic version stamp | **carried, extended** | Extended by a witness the old table could not produce — see D43 |
| **D30** — `::REQUIRES` lands last in this phase | **superseded** | Replaced by the 5c boundary. "Lands last" is an ordering; the re-cut needs a mechanism, and `::REQUIRES` is the package mechanism, which also collects `NAMESPACE`, `::RESOURCE`, `::OPTIONS` and `Package~local` — all unowned under the old cut |
| **D31** — `::OPTIONS` and the `OPTIONS` instruction stay in this phase, sequenced independently of dispatch | **amended** | They stay in Phase 5 and are now **placed**, in 5c with the other package-settings directives, instead of "whenever the plan finds room" — which produced zero mentions in the plan. The option list in the superseded spec is confirmed against `dire.xml` `optionsd`; `ENGINEERING`/`SCIENTIFIC` and `INHERIT`/`NOINHERIT` are values of `FORM` and `NUMERIC`, not additional options |
| **D32** — `REPLY`/`GUARD` get a run-time legality check, not a translation-time one, at rc 157 with the oracle's bytes; bare `GUARD` keeps its syntactic check | **carried** | Its open question is closed by D55 below |
| **D33** — the four directories are real objects; `.NAME` consults the running package's class table before `.environment`; `VALUE`'s one-argument form resolves through them | **amended** | The summary is steps 2 and 7 of the eight `rexxpg/classes.xml` `searchord` specifies. The **package-local directory** is a step and it wins over both: measured, with `MYTHING` set in all three, `say .MYTHING` is `from package local`. And the directories *do* something D33 does not mention — measured, `.environment~local~class` is `The Directory class` while `.environment~hasMethod("LOCAL")` is **0**, through `DirectoryClass`'s own `methodTable`/`unknownValue` |
| **D34** — `ExprKind::List` becomes a real Array from the first commit that makes `~` work | **carried** | The commit landed (Task 5) and the conversion did not: measured, `(1,)~size` is 2 on the oracle and rc 120 here. Migration item M4 below |
| **D35** — the six guard axes and the 1% floor rule | **carried** | |
| **D36** — the gate is the nine criteria; L2 reported not gated | **superseded** | The gate is the two derived tables plus the criteria restated in [the gate](#the-two-gate-tables) and [the class-set criterion](#the-class-set-criterion-which-replaces-32-classes). L2 stays reported, not gated, for the reason the old spec gives: it cannot start without `SysFileExists`, `.File` and `SysFileTree`, all Phase 7's |
| **D37** — the native entry-point registry; `file_separator` and `file_path_separator` implemented here as a stated scope addition from Phase 7 | **carried** | |
| **D38** — `::CONSTANT` is this phase's, specifically its parenthesised expression form | **amended** | Three documented rules the old text does not carry, all in `dire.xml` `constantd` and all confirmed: a `::CONSTANT` creates **both an instance method and a class method** — measured, `.K~c` and `.K~new~c` are both `42`; the expression form runs **as a method against the class object with `self` bound** (`ClassDirective::resolveConstants :243` calls `setScope`); forward references to calculated constants are refused. The floating-form refusal the old spec measured (99.906, rc 157) is confirmed by the documentation as a rule rather than being an oracle quirk |
| **D39** — the bootstrap has no oracle transcript; the two setup methods are provided during it and removed after | **carried, extended** | Its sibling is the REXX_DEFINED lock, which neither spec nor plan mentions. `liveGeneral` (`ClassClass.cpp:134`) sets `setRexxDefined()` under `PREPARINGIMAGE` and repoints `package` to `TheRexxPackage`; measured, `.Array~package~name` is `REXX` and `.Array~define("ZORK", .methods["Z"])` is rc 158 with `*-* Compiled method "DEFINE" with scope "Class".` and `Error 98.985: User additions are not allowed to the REXX language classes.` A bootstrap that finishes without setting it lets `.string~inherit(...)` succeed at rc 0 where the oracle raises — a divergence any corpus program can see |
| **D40** — `Body::Instance`'s association list replaced by a scope-keyed variable pool; `EXPOSE` is not new machinery | **carried** | Landed in Task 8 |
| **D41** — object identity is not modelled; handle equality, licensed as deviation 4 | **carried** | Observable only once `.IdentityTable` and `~identityHash` exist, which is 5c |
| **D42** — literals pooled by value, one object per distinct literal per compiled unit; not global; fresh per `INTERPRET`; `.true`/`.false` through the value path | **carried** | Its `.true`/`.false` half landed with Task 6 |
| **D43** — an object holds a behaviour reference; `define` copies before mutating, `inherit` mutates in place | **carried, extended** | The old spec's witness was one mutation at a time. Measured this session, the **pair in sequence** discriminates a version-stamped rebuild-in-place from the real thing: with `o = .K~new` first, `define` **then** `inherit` leaves `o~hasMethod("FROMMIXIN")` and `o~hasMethod("LATER")` both `0`, while `inherit` **then** `define` leaves them `1` and `0`. A rebuild-in-place answers `1 0` to both orderings and reddens only the first program, so both orderings are required |
| **D44** — a class carries two behaviours; `updateInstanceSubClasses` rebuilds one, `updateSubClasses` both; `phase-5.txt` needs a class-behaviour witness | **amended** | The witness is reachable from a program **only** through `::CLASS ... METACLASS`. Measured: `::CLASS S MIXINCLASS Class` + `::CLASS K METACLASS S` gives `say .K~classSideHi` = `class-side hi` at rc 0, while the `~inherit` route is refused by the oracle itself — `.K~inherit(.S)` is rc 158, `98.943 Class "The K class" is not a subclass of "The S class" base class "The Class class"`. So **`METACLASS` enters the phase**; the superseded plan kept it refused by design and named it uncovered, which made D44's own witness unreachable |
| **D45** — the security manager is seam only; exactly one chokepoint at dispatch and one at directory lookup, each asserted | **carried, extended** | The dispatch chokepoint is the one `PROTECTED` routes through (`processProtectedMethod :976`), which is what makes the seam's placement checkable rather than nominal, and the access-scope checks (`checkPrivate :609`, `checkPackage :659`) must sit inside it rather than beside it |

**Buckets: carried 13, amended 6, superseded 2, withdrawn 0.** Carried: D26, D28, D29, D32, D34,
D35, D37, D39, D40, D41, D42, D43, D45. Amended: D25, D27, D31, D33, D38, D44. Superseded: D30, D36.

## New decisions

* **D46.** **Phase 5 is cut into 5a, 5b and 5c along documented mechanisms**, as specified above: 5a
  is a class existing and answering a message, 5b is an instance existing, 5c is the package and the
  class library. No documented mechanism crosses a boundary, and the test of a proposed change to the
  cut is whether it splits one.
* **D47.** **"The bootstrap runs" is a milestone inside 5a**, not a phase boundary and not the gate.
  Established by reading both `.orx` files: the only Rexx bodies either runs at install are
  `TraceObject~activate` and two parenthesised `::CONSTANT`s, and none sends `~new`.
* **D48.** **The Phase 5 class criterion is the documented class set**, derived as above, with the
  five recorded disagreements. This amends `2026-07-27-rust-rewrite.md:453`'s "32 classes exist and
  respond", which is `CoreClasses.orx`'s own `::CLASS` count and is satisfiable while thirty classes
  are missing.
* **D49.** **Gate table D exists**, with the row set, columns, probe-corpus obligation and three named
  falsifying mutations specified above.
* **D50.** **Gate table C exists**, with both halves, the measured `~method`/`~hasMethod` readback
  rule, and the four named falsifying mutations — of which the `UNKNOWN` and `makeString` controls
  must be recorded as run, because those two rows are the reason this spec was written.
* **D51.** **The documented method search order is implemented complete in 5a**, including the
  `UNKNOWN` step and the NOMETHOD condition beneath it. `::METHOD unknown` is reachable on a class
  object without `~new`, measured, so no part of it defers to 5b except the per-object first step.
* **D52.** **The required-string protocol is a 5a mechanism** — `request("STRING")`, then the
  receiver's `makeString`, then the NOSTRING condition if trapped, else `defaultName` — and every
  context `provide.xml` `reqstr` lists becomes a dispatch site. The crate's existing object-operand
  answers are re-derived from that section rather than from the coercion audit that produced them.
* **D53.** **There are three access scopes, not one.** `PRIVATE` and `PACKAGE` are separate limbs with
  separate errors (`checkPrivate :609`, `checkPackage :659`), and `PACKAGE` is checked against the
  **caller's package**, which nothing in the crate establishes as a dispatch input today. `PROTECTED`
  is the fourth route and goes through D45's seam.
* **D54.** **`::ANNOTATE` naming a target is in scope**, in 5a, with `~annotation`/`~annotations`. The
  three places carrying the generalised refusal claim are corrected as listed above.
* **D55.** **A `REPLY` or `GUARD` inside a Phase 5 method runs.** This closes the superseded spec's
  open question. Measured: `guard on` inside a class method is **oracle rc 0** against our rc 120, and
  `reply "replied"` followed by `return "returned"` is **oracle exit status 0** with stdout
  `replied` / `after` and a `98.936 RETURN cannot return a value after a REPLY` traceback on stderr,
  against our rc 120. Both are over-refusals today. In a single-threaded Phase 5, `GUARD ON` on an
  uncontended object is a no-op, and `REPLY` hands its value to the sender and lets the method body
  carry on — where a `RETURN` carrying a value then raises `98.936`, which is a run-time legality
  check like D32's. Note the transcript shape a gate case must reproduce: **exit status 0 with a
  traceback on stderr**, because the raise happens after the main program has finished. Phase 6
  replaces the scheduling, not the legality.
* **D56.** **`oodocs/` and `ootest/` are authorities, not build dependencies.** Everything derived
  from either is committed as a data file naming the upstream revision it was derived at, with the
  extractor committed beside it, following `corpus/keyword-exempt.txt`. The gate report records one
  re-derivation run against a present `oodocs/` whose diff against the committed file is empty; the
  test that re-derives **fails** when `oodocs/` is absent unless the run is explicitly marked
  docs-less, because a check that silently skips is the failure mode this project has shipped before.

## Migration items against landed work

Tasks 1 to 8 of `docs/superpowers/plans/2026-08-15-phase-5a-native-layer.md` are complete and their
code is in the tree at `b360783cb`. **Nothing in this spec invalidates any of it.** Nine things change
around it, each stated so the plan that follows can own it rather than discover it.

1. **`::ANNOTATE`'s arm and its comment** (`rexx-exec/src/lib.rs:1275`-`1281`). The refusal is retired,
   not just re-commented; the comment's measurement stays and gains its sibling.
2. **Task 5's dispatch miss arm.** `corpus/lang/message_send_unknown_method.rex` pins 97.1 for a name
   the behaviour does not answer — correct for a receiver with no `UNKNOWN` method, and it is now
   *one branch of two*. The `UNKNOWN` search step goes in front of it and the program gains a sibling
   whose receiver has one.
3. **Task 5's `resolve` signature** gains the caller's scope and the caller's package (D53), and
   Task 5's chokepoint assertion must be the thing `PRIVATE`, `PACKAGE` and `PROTECTED` all pass
   through, not a sibling of them.
4. **`ExprKind::List`** (D34) was bound to Task 5's commit and did not land: measured, `(1,)~size` is
   2 on the oracle and rc 120 here.
5. **Task 6's environment-symbol order** gains the package-local directory as a step above `.LOCAL`
   and `.ENVIRONMENT`, and the four directory objects gain the Directory entry-method mechanism.
6. **Task 6's `.methods` work and Task 7's method-body work** gain the join `dire.xml` states three
   times: a floating `::METHOD`/`::ATTRIBUTE`/`::CONSTANT` is reachable through `.METHODS`, which is
   what `CoreClasses.orx:73` depends on — inside the loop at `:69`-`:74`, and read at the file
   rather than taken from the correlation review, which cites `:66`-`:71`. Two measurements bound it,
   and the second corrects a
   guess I made before running it: `.methods~class` is `The StringTable class`, **and a StringTable
   answers an entry name sent as a message just as a Directory does** — `.methods~z` and
   `.methods["Z"]` both give `The Method class`, an absent name gives `The NIL object` rather than
   raising, and `.methods~hasMethod("Z")` is `0`. Separately, a package with no floating directive at
   all leaves `.METHODS` and `.ROUTINES` resolving to their own literal spellings — rc 0 and
   byte-identical on both engines today, so this is agreement to preserve rather than a gap.
7. **Task 4's `::CONSTANT`** is not finished by installation: measured, `.K~c` is 97.1 here against
   `42` on the oracle, and the directive owes both an instance method and a class method (D38).
8. **Task 8's deferred install** covers the ordering and not the passes. Measured on this build, both
   of them things ruling R26 asked for: a forward `::CLASS B SUBCLASS A` resolves (`say .B~m` is
   byte-identical) and a cycle is byte-identical at 98.911 rc 158. What is not there is the
   **three-pass** structure —
   install all, resolve constants, activate all in construction order — which `fundclasses.xml`
   `mthClassActivate` specifies and which `CoreClasses.orx:2184` needs, since `DateTime` inherits
   `Orderable` from `:3840`.
9. **`corpus/lang/primitive_classes.rex` is in no subset file.** It asserts `~id` across the primitive
   classes and is reached only as a *parse* fixture, by `rexx-parse/src/instruction/tests.rs:1901`;
   nothing runs it against the oracle, and `.array~id` is rc 120 here today. The class half of gate
   table C is where that program's job belongs, and it should not be left looking like coverage.

Two further items from the plan-split review are carried forward unchanged because this spec does not
displace them: the `SUBCLASS`/`MIXINCLASS` shared slot in `ast.rs`'s `ClassDirective` (M10), which
gate table D's oracle column is now the instrument for; and Task 9's and Task 11's verifications,
which name a bootstrap that runs later than they do.

## Risks

| risk | consequence | response |
|---|---|---|
| a derived row set is committed once and the docs move | the table measures a stale specification | D56's revision stamp and the recorded re-derivation run |
| the concept half of table C is satisfied by a program that touches the section's subject without discriminating it | a green row over a wrong implementation | each concept row names its negative control, and the two rows this spec exists for must have theirs recorded as run |
| 5a is now larger than the superseded 5a | it never closes | it is larger by mechanisms that were unowned, not by new work; the two gates inside it (core, then bootstrap) stay, and the added items are individually small — `UNKNOWN` is one search step, `PACKAGE` one limb, `METACLASS` one directive keyword |
| `METACLASS` entering 5a reopens the class-behaviour side late | D44's witness lands after the code it witnesses | it is the *only* program-level route, measured; the alternative is deferring D44's witness explicitly, which the superseded plan did by accident |
| the required-string protocol touches every instruction that takes a string | a long tail across Phase 4 surface | `provide.xml` `reqstr` enumerates the contexts, so the tail is bounded and countable before it starts |
| a documented method set is large and mostly 5c's | table C reads red for most of Phase 5 | that is correct and is the point; the table reports per-row status and only the class half's *wiring* rows gate 5a |

## What I could not check

* **`ootest/` as a gate.** I read no test group in this session. Every `ootest` citation in the
  enumeration above is inherited from `.superpowers/sdd/2026-08-15-phase-5a-native-layer/object-model-correlation.md`
  and is **not independently verified here**. The three-signal rule for declaring an oracle defect
  needs `ootest` read directly, and the plan owes that reading.
* **Concurrency.** `provide.xml` `concurr` is enumerated as a row and not derived. It is Phase 6's,
  and D55 decides only what a `REPLY`/`GUARD` does in a single-threaded Phase 5, on two measured
  programs. Whether `UNGUARDED` has any Phase 5-observable effect is unmeasured.
* **Most of the C++ line numbers.** I read and confirmed `Setup.cpp:185` (`addToSystem`), `:396`
  (`EndSpecialClassDefinition`), `:1285` (`RexxInfo`), `ClassClass.cpp:984` (`RexxClass::method`, with
  its own comment) and `DirectiveParser.cpp:948` (`optionsDirective`). **Every other `file:line` in
  the enumeration is inherited** from the superseded spec and from
  `object-model-correlation.md`, and is not re-verified here. **Both inherited `.orx` citations I did
  check were wrong** — `CoreClasses.orx:66`-`71` for the `.methods` index, which is `:73`, and `:92`
  for `.string~inherit(.Comparable)`, which is `:93` — so the plan should re-read a citation before
  leaning on it rather than treating the inherited ones as sound.
* **Whether `~method` reaches every mixin-donated name.** Measured on two points —
  `.String~method("COMPARETO")` answers, `.Array~method("STRING")` raises — and the mechanism is
  `instanceMethodDictionary` (`ClassClass.cpp:984`). I did not read how `inherit` populates that
  dictionary, so the exact inclusion rule for a donated name is stated as a measurement, not a rule,
  and the extractor's expected column must be validated against the oracle per class rather than
  reasoned from the hierarchy.
* **The crate column throughout.** Every crate reading is from
  `rust/target/release/rexx-run` as built at 07:06 on 2026-08-17, at `b360783cb`. I did not build. A
  task landing between then and the plan will move some of these rows, which is what the tables are
  for.
* **`StreamClasses.orx`'s full shape inventory.** I confirmed its two parenthesised `::CONSTANT`s by
  scanning both files and inherited the other four shapes (both keywords on one directive, a quoted
  target, the private-method constant, the same-line body) from the plan-split review without
  re-reading the lines.
* **`PlatformObjects.orx` on non-unix.** Unix's is one comment line. I checked no other platform, and
  CI runs five.
* **Cold start and D2.** Not measured here; the criterion and its instrument
  (`rexx-bench/src/bin/rexx-time.rs`, `--warmup 10 --runs 50`, against the C++ median of 5.119 ms
  recorded in `perf-baseline.md`) are carried from the superseded spec unchanged.
* **Whether any oracle behaviour recorded here is an upstream defect.** I applied no part of the
  three-signal rule. Everything above is recorded as behaviour to reproduce.
