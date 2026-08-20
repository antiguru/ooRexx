# The Phase 5 reading ledger

**What this is.** Which authorities behind
[`docs/superpowers/specs/2026-08-17-phase-5-object-model.md`](../specs/2026-08-17-phase-5-object-model.md)
have been read end to end, by whom, to which stopping point, at which revision. Written at Task 2 of
`docs/superpowers/plans/2026-08-17-phase-5a.md`, which is deliberately early: the spec's enumeration
of mechanisms has no denominator, nothing derives it, and no check can say it is complete. The only
instrument that has ever found a missing mechanism on this project is reading an authority end to
end. Doing that before the gate tables' row sets are frozen means a mechanism the reading finds can
still become a row rather than a surprise.

**What this ledger cannot see, said here rather than in a report nobody rereads.** Its own
denominator is a hand-made list of authorities. It narrows the hole the spec names at
["What supplies each check's denominator"](../specs/2026-08-17-phase-5-object-model.md#what-supplies-each-checks-denominator); it does not
close it. An authority nobody thought to list is invisible to this document exactly as a mechanism
nobody thought to enumerate is invisible to the spec's table.

**Standing.** A row that is `not done` is not a defect. **A row whose stamp is older than the
checkout is** — that staleness check is Task 3's, run against the stamps below, because it needs the
checkout present and D56's docs-less rule.

**The spec is adopted and this task does not edit it.** Where the reading found one of its citations
or claims wrong, the correction is recorded here, which is the route the plan specifies.

---

## The authority rows

| authority | read end to end means | reader | status | revision |
|---|---|---|---|---|
| `oodocs/rexxref/en-US/provide.xml` | every section of the `provide` chapter | spec author, 2026-08-17 | **done** — that reading produced the concept floor | r13198 (`svn info oodocs/rexxref`, checked 2026-08-21) |
| `oodocs/rexxref/en-US/dire.xml` | every one of its nine directive sections | spec author, 2026-08-17 | **done** | r13198 |
| `oodocs/rexxref/en-US/fundclasses.xml`, `collclasses.xml`, `utilityclasses.xml`, `streamclasses.xml` | every `cls*` section's own prose — not every `mth*` | Task 2, 2026-08-21 | **done** — every `<section id="cls…">`'s prose from its opening tag to its first nested section, extracted mechanically and read; findings below | r13198 |
| `oodocs/rexxpg/en-US/classes.xml` | every section of "A Closer Look at Objects" | Task 2, 2026-08-21 | **done** — the whole chapter, `:46` to its closing `</chapter>` at `:1500`; every section listed below was read | r13198 (`svn info oodocs/rexxpg`) |
| `ootest/` | one row per 5a mechanism in the spec's enumeration, naming the test group that pins it or recording that none does | Task 2, 2026-08-21 | **done** — the mapping is [below](#the-ootest-mapping); every test group named there was opened | r13178 (`svn info ootest`) |
| the C++ | not readable end to end; the bounded stopping point is **every `file:line` the spec cites, verified** | Task 2, 2026-08-21 | **done** — every citation resolved; the wrong ones are [below](#citations-found-wrong) | tracked `interpreter/` at `cb9563364`, identical to this worktree's at `e52bff06a` (see the note) |

**`oodocs/` itself has no `svn info`** — `svn: E155007: … is not a working copy`, measured. Its two
subdirectories are separate working copies and carry the revision, so a check run against the parent
returns nothing and reads like absence. Check `oodocs/rexxpg` and `oodocs/rexxref`.

**The C++ revision, stated so it can be checked rather than assumed.** The spec names
`/home/moritz/dev/repos/ooRexx/interpreter/` as the authority. That checkout is on branch
`concurrency-dispatch-fixes` at `cb9563364`, **plus one uncommitted change** —
`interpreter/classes/NumberStringClass.cpp`, a ten-line hunk in `copyIfNecessary()` around `:3658`,
which is the only file differing between that tree and this worktree's `interpreter/` (measured by
`diff -rq`). It touches nothing any citation below names. And `git merge-base cb9563364 e52bff06a` is
`cb9563364` with `git diff cb9563364 e52bff06a -- interpreter/` empty, so the **tracked** C++ is the
same content at both, and every citation below was resolved against content a later reader can get
from either. The uncommitted hunk is machine state no file in the checkout records; it is written
down here because that is the only place it can be.

---

## Citations found wrong

Resolved, not read. Each entry names what the cited location actually is and where the subject lives.

### C++

| the spec cites | what is there | the right one |
|---|---|---|
| `LanguageParser::addMethod :610` (spec `:138`) | **wrong file.** `LanguageParser.cpp:610` is blank; `LanguageParser.cpp` contains no `LanguageParser::addMethod` definition at all | **`DirectiveParser.cpp:610`** — `void LanguageParser::addMethod(RexxString *name, MethodClass *method, bool classMethod)`. The line number is right and the file is not; the spec writes the class-qualified form, so a reader resolves it in the wrong file |
| `createClassBehaviour :1119` (spec `:117`) | a comment *inside* the function — `// Object is a special case, since it is top dog.` The function is `ClassClass.cpp:1098` | **`ClassClass.cpp:1098`**, matching its sibling `createInstanceBehaviour :1148`, which *is* the definition line. The pair is inconsistent as written |
| `processInstall :1268-1298` (spec `:136`) | resolves inside `PackageClass::processInstall`, defined at **`:1227`**. The range's end at `:1298` falls one line short of `current_class->activate()` (`:1299`) — the third pass's own call, and the pass the row's phrase "three passes" is about | **`PackageClass.cpp:1268`-`:1301`** for the class block, or `:1227` for the function. The three loops are `:1276`, `:1285`, `:1294` |
| `attributeDirective :1457-1850` (spec `:131`) | `:1850` is blank; the function closes at `:1848` | **`DirectiveParser.cpp:1457`-`:1848`** |

**Checked and correct, listed because a re-checker should not redo them.** `ClassClass.hpp:180-189`
(`ClassFlag`); `ClassClass.cpp` `:558 :654 :819 :952 :1036 :1071 :1148 :1210 :1287 :1322 :1379
:1440 :1514 :1631 :1741 :1754 :1854 :1882 :984 :134 :343 :518`; `ObjectClass.cpp` `:609 :659 :697
:866 :919 :976 :1002 :1235 :1302 :1696 :1733 :1760 :1829 :1891 :1950 :2185 :2489 :2579 :2604`;
`MethodDictionary.cpp` `:164 :348 :434 :594`; `DirectiveParser.cpp` `:287 :334-490 :629-812 :948
:1940 :2266 :2438 :2518 :2565 :2779`; `PackageClass.cpp` `:1086 :1432 :1944 :2169`;
`DirectoryClass.cpp` `:480 :591`; `LanguageParser.cpp` `:1801 :3292`;
`instructions/ClassDirective.cpp` `:243 :257 :273`; `RexxActivation.hpp:357`;
`ExpressionClassResolver.cpp:135`; `Setup.cpp` `:185 :331 :332 :360-361 :371-372 :396 :688 :792-804
:1285 :1307-1312 :1399-1404 :1781 :1809`.

Two of those deserve a note because the shape that made `ClassClass.hpp:176-186` wrong is present and
harmless in them. `classDirective :334-490` bounds a function running to `:495`, and
`methodDirective :629-812` bounds one running to `:942` — both stop at the end of the directive's
**option switch**, which is what the rows are about. `methodDirective`'s twelve `SUBDIRECTIVE_` arms
run `:674` to `:796` and the cited end at `:812` is the blank line after the last one, before the
`default:` arm at `:814`. Both ranges are tight around their subject rather than short of it, and the
enumeration's option lists match the arms exactly: `::CLASS`'s seven (`:396`-`:477`), `::METHOD`'s
twelve, `::ATTRIBUTE`'s thirteen (`:1500`-`:1627`).

### In-tree Rust — three that the tree has moved out from under

The spec was written against `b360783cb`; Task 1 has landed since. These resolve to the wrong place
today, and one of them resolves to a *neighbouring* line about the same subject, which is the worse
case because it reads as correct.

| the spec cites | what is there now | the right one |
|---|---|---|
| `rexx-exec/src/lib.rs:1275`-`:1277` and `:1275`-`:1281`, the `::ANNOTATE` arm and its comment (spec `:1104`, `:1283`) | `:1275`-`:1277` is the `::CLASS SUBCLASS naming a namespace` arm | **`crates/rexx-exec/src/lib.rs:1285`-`:1292`** — comment `:1285`-`:1287`, arm `:1288`-`:1292`, `gap("::ANNOTATE naming a target", "Phase 5")` at `:1291` |
| `corpus.rs:244` for `Invocation::none()` (spec `:628`) | `.to_str()` | **`crates/rexx-exec/tests/corpus.rs:246`** |
| `tests/support/oracle.rs:189` and `:214`, said to "build the outcome with `exit_code: output.status.code().unwrap_or(-1)`" (spec `:715`) | both lines still land on the did-not-finish subject, but **the code the sentence describes no longer exists**: there is a `Termination` enum, `did_not_finish` at `:188` tests `!matches!(outcome.termination, Termination::Exited(_))`, and `expect_exit_code` panics at `:214` rather than synthesising `-1`. The only `unwrap_or(-1)` left in that directory is inside a doc comment at `:148` explaining why it is not used | the structural check the spec asks for **exists**, as `did_not_finish` (`:188`). The spec's paragraph describing the hazard is history, not a live finding |

`rexx-parse/src/instruction/tests.rs:1901`, `phase-4-exclusions.txt:420`, and the roadmap's `:36`,
`:453` and `:492` all resolve. The roadmap's `:482`, which the superseded spec cited, is blank —
confirming the correction the spec already carries.

### `.orx` and XML

Every one resolves: `CoreClasses.orx` `:73 :93 :151 :987 :1299 :1300 :1557 :1590 :1659 :1690 :2184
:3840 :3974 :3976 :3996`; `StreamClasses.orx` `:115 :371 :546 :547 :548 :549`; `provide.xml` `:59
:843 :844 :845 :849`; `utilityclasses.xml` `:429` (inside `<section id="clsBuffer">`, opened `:421`)
and `:6910` (inside `clsPointer`, opened `:6902`).

### One false sentence in the spec's own "What I could not check"

> "Every `ootest` citation in the enumeration above is inherited from `object-model-correlation.md`
> and is **not independently verified here**."

**There are none to inherit.** Measured: `sed -n '114,172p'` over the spec — the enumeration table —
piped to `/bin/grep -ac ootest` is **0**. The word `testGroup` occurs twice in the whole document,
at `:822` (the reading table this ledger supersedes) and at `:1096` (the `::ANNOTATE` section), and
neither occurrence is in an enumeration row. The enumeration's `authority` column cites
`provide.xml`, `dire.xml`, `fundclasses.xml` and the sibling books, and never `ootest`.

The spec's one substantive `ootest` claim is at `:1096` — *"`ootest`'s `ANNOTATE.testGroup` has a
passing test per target"* — and it is **outside** the enumeration. Checked: the group is
`ootest/ooRexx/base/directives/ANNOTATE.testGroup`, `::annotate` occurs 76 times in it and
`~annotation`/`~annotations` 70 times.

---

## The `ootest` mapping

One row per 5a mechanism in the spec's enumeration — every row whose `phase` column names `5a` for
the mechanism or for a named arm of it. **A mechanism with no test group is a real answer**, and it
is what makes the plan's three-signal rule checkable: until this mapping existed, its second signal
("`ootest` does not pin it, checked at `ootest/` directly") could not be applied at all.

Each row cites the group by path under `ootest/ooRexx/`. Every group named was opened; the counts
beside them are `/bin/grep -aciE` measurements over that file.

| spec line | mechanism | test group that pins it |
|---|---|---|
| `:116` | object / mixin / abstract / metaclass as four kinds | `base/directives/CLASS.testGroup` (`MIXINCLASS` 52, `METACLASS` 50, `ABSTRACT` 16) and `base/class/Class.testGroup` (67, 58, 12) |
| `:117` | a class carries two behaviours, class-side and instance-side | `base/directives/CLASS.testGroup` and `base/class/Class.testGroup`, through `METACLASS` — the same route D44's witness has to take |
| `:118` | the metaclass graph and its circularity | `base/directives/CLASS.testGroup` — `98.911` 5, `METACLASS` 50 |
| `:119` | a mixin's base class, and who may inherit it | `base/class/Class.testGroup` (`~baseClass` 8), `base/directives/CLASS.testGroup` (3) |
| `:120` | merge order: a class's own methods precede its superclasses' **and its mixins'** | **none.** The shape exists — `base/class/Class.testGroup:1276` is `::CLASS AmphibianVehicle SUBCLASS RoadVehicle INHERIT WaterVehicle` — but the only assertion over it is `test_issubclassof`, and `AmphibianVehicle` **defines its own `show_off`**. That is exactly the variant the spec establishes is green under an implementation that has superclass-versus-mixin backwards. Searched the tree for a test naming precedence between a superclass's method and a mixin's; the only hits are scope-override error tests (`93.957`) in `base/keyword/FORWARD.testGroup`, `base/class/Object.testGroup` and `base/class/Message.testGroup`, which are a different question |
| `:121` | merge order among several `INHERIT`s — leftmost first | **pinned at the scope-list level, not at method precedence.** `base/directives/CLASS.testGroup`'s `test_inherit` (`:355`) asserts `~superClasses` as an **ordered** list, and the assertion moves when the directive's mixin order moves: `::class test inherit rexx:comparable 'orderable'` gives `(.Object, .Comparable, .Orderable)` (`:386`), `inherit testMixin rexx:comparable 'orderable'` gives `(.Object, mixin, .Comparable, .Orderable)` (`:401`, `:415`), and `inherit comparable 'testMixin' rexx:orderable` gives `(.Object, .Comparable, mixin, .Orderable)` (`:432`, `:449`). Those last two differ only in the mixins' declared order, so the pair discriminates leftmost-first. What it does not reach is which mixin's *method body* wins: `base/class/Orderable.testGroup:441`'s `::class TestOrderable inherit Orderable Comparable` defines its own `compareTo` |
| `:122` | method dictionary: one entry per scope, `addFront`, scope list and scope orders | `base/class/Class.testGroup` — `~define(` 29, `~uninherit(` 9 |
| `:123` | `~define` copies the behaviour, `~inherit` mutates it in place | `base/class/Class.testGroup`'s `test_class_define` (`:976`) is the D43 witness upstream already has: `t1 = .testDefine1~new`, then `.testDefine1~define('test1', …)`, then `t1~hasMethod('test1')` still **false** while a freshly built `t2` answers true (`:977`-`:982`). `test_define_delete` (`:1012`) is its `delete` twin |
| `:124` | `inheritInstanceMethods` — donation with no superclass edge | **none.** The name occurs **zero** times in `ootest`, measured over the whole tree. Consistent with its being image-build-only and removed by `Setup.cpp:1809` |
| `:125` | the cascade, instance-side vs both sides | `base/class/Class.testGroup`'s `test_class_define` (`:976`), against `::class testDefine2 subclass testDefine1` (`:1444`-`:1445`): `.testDefine1~define('test1', …)` and then `t3 = .testDefine2~new` answering `test1` with `123` (`:984`-`:986`) is the cascade to a subclass. Later in the same test, redefining on `testDefine1` changes what a **new** `testDefine2` instance answers (`:996`-`:1000`). The class-side-versus-both-sides half of the row — `updateSubClasses` against `updateInstanceSubClasses` — is not separated by any test |
| `:126` | native method removal and hiding at image build | **the mechanism yes, its image-build application no.** The `.nil`-tombstone *mechanism* is pinned at the Rexx level by `base/class/Class.testGroup`'s one-argument `~define`: `.test_a~define("testmethod")` at `:205` and `:236` (inside `test_issubclassof_non_class`, `:167`) with the comment "make it unaccessible for new instances", and `.testDefine2~define('TEST1')` and `newClass~define('newMethod')` in `test_class_define` (`:976`) and `test_define_delete` (`:1012`). That is the same `put(TheNilObject, name)` `MethodDictionary::hideMethod` (`:348`-`:351`) performs, reached through `define` — which is exactly how `collclasses.xml:8064`-`:8067` describes Stem's six. Its **observable** is pinned too, with the error code: `test_class_define:1002`-`:1004` hides an inherited `test1` on the subclass with `.testDefine2~define('TEST1')` and asserts the send then raises **97.1**, which is the row's `.nil`-to-`UNKNOWN` limb. What is **not** pinned is the application at image build: measured over the whole tree, no `assertFalse(…hasMethod…)` anywhere names a natively removed or hidden name, and `base/class/Stem.testGroup` contains no `hasMethod` call at all. The nearest thing is the **composition**, for Queue only — `base/class/Queue.testGroup:452` `test_stableSort` runs `a = .queue~of` then `a~stableSort`, which is `Setup.cpp`'s `RemoveMethod("stableSort")` composed with `CoreClasses.orx`'s re-donation through `OrderedCollection`, and which the spec establishes is the only observable form |
| `:127` | the REXX_DEFINED lock | `base/class/Class.testGroup` — `98.985` 7 |
| `:128` | `~define` / `~defineMethods` / `~delete` / `~uninherit` / `~enhanced` | `base/class/Class.testGroup` (`~defineMethods` 12, `~uninherit(` 9, `~enhanced(` 2), `base/class/Object.testGroup` (`~enhanced(` 3) |
| `:129` | `::CLASS` option surface | `base/directives/CLASS.testGroup` |
| `:130` | `::METHOD` option surface | `base/directives/METHOD.testGroup` — `PRIVATE` 12, `PACKAGE` 12, `PROTECTED` 6, `UNGUARDED` 9, `EXTERNAL 'LIBRARY` 14 |
| `:131` | `::ATTRIBUTE` option surface | `base/directives/ATTRIBUTE.testGroup` — `::attribute` 118, `ABSTRACT` 50, `EXTERNAL 'LIBRARY` 14, and the `DELEGATE` assertions at `:733`-`:756` |
| `:132`, `:133`, `:134` | `::CONSTANT`, its parenthesised form, its forward-reference refusal | `base/directives/CONSTANT.testGroup` — `::constant` 76, `ACTIVATE` 5, and `test_expression_self` at `:457` for `:133`'s "with `self` bound to the class" |
| `:135` | `::ANNOTATE`'s six targets and `~annotation`/`~annotations` | `base/directives/ANNOTATE.testGroup` — `::annotate` 76, `~annotation(s)` 70. The spec's own citation, now verified |
| `:136` | install is three passes; a cycle is 98.911 | **the cycle only.** `base/directives/CLASS.testGroup` (`98.911` 5). The **three passes** are not asserted anywhere; `ACTIVATE` appears in `base/directives/REQUIRES.testGroup` (7) and `CONSTANT.testGroup` (5) as a method name, not as a pass-ordering claim |
| `:137` | class-object initialization: `INIT`, then `INHERIT`, then `ACTIVATE` | **none.** No test group runs the `INIT`-before-merge / `ACTIVATE`-after-merge discriminator the spec pins its 5a/5b boundary on |
| `:138` | floating `::METHOD`/`::ATTRIBUTE`/`::CONSTANT` reach `.METHODS` | `base/directives/METHOD.testGroup` (`.methods` 3), `base/class/Class.testGroup` (4) |
| `:139` | the complete method search order, including `UNKNOWN` and NOMETHOD | **none for the `UNKNOWN` step itself, and this is the row the spec exists for.** `::method unknown` occurs in exactly three groups tree-wide; the only object-model one is `base/class/Object.testGroup:1487` and `:1493` (`unknown` and `unknown class`), and reading them, both are **scaffolding** — their whole body is `if name == "UNINIT" then .UninitTracker~recordUninit(self)`, i.e. `UNKNOWN` is the instrument for UNINIT tests, not their subject. The other two are `base/security.manager/SecurityManager.testGroup` and a Windows Unicode group. NOMETHOD is asserted in `base/keyword/SIGNAL.testGroup` (3), `base/keyword/RAISE.testGroup` (2) and `base/bif/CONDITION.testGroup` (2) — as a *condition*, not as dispatch's last step |
| `:140` | changing the search order — `~m:scope`, `~m:super` | **pinned, but through the alternative send paths rather than the `:super` syntax.** `base/class/Object.testGroup:1159`-`:1253` and `base/class/Message.testGroup:700`-`:912` are families of `*_override_from_nonself*` tests asserting `93.957` — which is `validateScopeOverride` (`ObjectClass.cpp:1950`), this row's own implementation citation — reached through `~send`/`~sendWith`/`~start`/`~startWith`. `base/class/Message.testGroup:372` is `test_super_override`. The **`~m:super` source syntax** is what is thin: `:super` occurs once each in `base/class/Class.testGroup`, `base/class/Object.testGroup` and `base/keyword/ADDRESS.testGroup`, and nowhere else |
| `:141` | `SELF`, `SUPER` | **no dedicated group.** `base/special.variables/` holds only `RESULT_RC_SIGL.testGroup`, and both variables are used throughout the suite while pinned by nothing that is about them. The nearest thing is `base/directives/CONSTANT.testGroup:457` `test_expression_self`, which pins `self` bound to the class object inside a parenthesised `::CONSTANT` — that is enumeration row `:133`'s mechanism, not this one's |
| `:142` | `PUBLIC` / `PACKAGE` / `PRIVATE` as three access scopes | `base/directives/METHOD.testGroup` — the `PRIVATE` (12) and `PACKAGE` (12) arms |
| `:143` | `PROTECTED` routes the send through the security manager | `base/security.manager/SecurityManager.testGroup` — `~setEntry`/`~entry(` 32 |
| `:144` | scope-keyed instance variables, `EXPOSE`, lazy creation | `base/keyword/EXPOSE.testGroup` |
| `:145` | class-scope instance variables | `base/keyword/TRACE_TraceObject.testGroup:455`-`:468` — `::attribute baseline class` and `::method init class` with `expose a b baseline`, which is the shape `CoreClasses.orx:3996`'s `::method activate class` needs |
| `:146` | Required String Values — `request("STRING")` → `makeString` → NOSTRING → `defaultName` | **split, and the `makeString` limb is unpinned.** `base/class/MethodArgs.testGroup` carries the receiver-by-receiver arm — `test_request_string_class` (`:137`), `_object`, `_string`, `_method`, `_routine`, `_package`, `_message`, `_stream`, `_mutablebuffer`, `_file` — and `base/keyword/VarRef.testGroup:134` has `test_request_nostring`. But `::method makeString` appears in **no** `base/` group at all — only `API/oo/METHOD.testGroup`, `API/oo/FUNCTION.testGroup`, `extensions/json/json.testGroup` and `base/class/Array.testGroup` (as Array's own method, not as the protocol). The limb the spec was written for is the one upstream does not pin either |
| `:147` | `~objectName`, `~objectName=`, `~string`, `~request` | `base/class/Object.testGroup` (`objectName` 11), `ooRexx/doc/rexxref/chapter5/Section1.testGroup` (10) |
| `:148` | the `Object`/`Class` native protocol | `base/directives/CLASS.testGroup` (`~isSubclassOf` 23, `~metaClass` 9, `~superClasses` 8) and `base/class/Class.testGroup` (20, 4, 2), `base/class/Object.testGroup` |
| `:150` | `~identityHash` — the message-answering arm, which is 5a's | `base/class/Object.testGroup` — 24 |
| `:151` | `.environment`, `.local`, `.context`, `.methods` as objects | `base/runtime.objects/environmentEntries.testGroup` (its `test_local_entries` `:69` and `test_local_monitors` `:76`), `base/class/RexxContext.testGroup` (`.context` 47) |
| `:152` | Directory entry methods | `base/class/collections/directory.testGroup` (`~setEntry`/`~entry(` 12), `base/class/Directory.testGroup` |
| `:153` | the eight-step environment-symbol search order | **partially.** `base/runtime.objects/environmentEntries.testGroup` covers `.local`'s contents and `base/bif/VALUE.testGroup` the `.environment` step; no group walks the order as an order |
| `:154` | `.Package` — `addClass`, `addPublicClass`, `publicClasses`, `~name`, install | `base/class/Package.testGroup` — `addClass`/`addPublicClass` 52 |
| `:155` | the native entry-point registry for `EXTERNAL 'LIBRARY REXX name'` | `base/directives/METHOD.testGroup` (14), `base/directives/ATTRIBUTE.testGroup` (14), `base/directives/ROUTINE.testGroup` (10) |
| `:160` | abstract-**method** enforcement | `base/directives/ATTRIBUTE.testGroup` (`93.965` 8), `base/directives/METHOD.testGroup` (1) |
| `:169` | `GUARDED`/`UNGUARDED`, `REPLY`, `GUARD` legality | `base/keyword/GUARD.testGroup` (`guard on`/`off` 60), `base/keyword/REPLY.testGroup` (`reply` 33) |
| `:170` | the `*-* Compiled method "X" with scope "Y".` traceback line | **none in the object model.** The string occurs in exactly one test group tree-wide, `base/source.file/incorrectCharacters.testGroup` (6), which is about source encoding |
| `:171` | `>M>` trace prefix | `base/keyword/TRACE.testGroup` — 4 |

**What this mapping cannot see.** A group named here pins its mechanism *somewhere*; the row does not
claim the group's assertions are sufficient, and for `:120`, `:121` and `:126` the reading is that
they are demonstrably not. Conversely a "none" row means the searches listed found nothing, over the
patterns named — it is a statement about those searches, not a proof of absence.

---

## The unexercisable-class list, re-derived

The spec carries this as a carried-and-corrected open question and requires re-derivation rather than
inheritance. **The criterion used:** a class the phase cannot construct an instance of, because its
`init` chain reaches a native entry point D37 registers and does not implement. D37 implements
`file_separator` and `file_path_separator` and nothing else, so every other
`EXTERNAL 'LIBRARY REXX …'` raises when invoked.

**The derivation, mechanically:** every `EXTERNAL 'LIBRARY REXX …'` method directive in
`CoreClasses.orx`, `StreamClasses.orx` and `platform/unix/PlatformObjects.orx`, attributed to its
owning `::CLASS`, then each owning class's `init` traced to see whether it reaches one.
`platform/unix/PlatformObjects.orx` is one comment line and declares nothing. In `CoreClasses.orx`
the externals belong to `Alarm` and `Ticker` and to no other class.

| class | file | `init` | native reached | why it is on the list |
|---|---|---|---|---|
| **`Ticker`** | `CoreClasses.orx` | `:1630` | `self~!createTimer` at `:1659`, declared `:1690` as `EXTERNAL 'LIBRARY REXX ticker_createTimer'` | the call is **before** the method's `guard off` (`:1662`) and `reply` (`:1663`), so the raise reaches the sender and `~new` cannot succeed |
| **`Alarm`** | `CoreClasses.orx` | `:1472` | `self~!startTimer(numdays, alarmtime)` at `:1557`, declared `:1590` as `alarm_startTimer` | **and this is not the same case as Ticker's.** The `reply` is at `:1556`, one line *before* the native call. Under D55 a `REPLY` hands its value to the sender and the body carries on, so in Phase 5 the sender gets its Alarm back and the native raise lands on the continuation — the D55 transcript shape, a traceback on stderr after the main program has finished, rather than a failed `~new`. The spec treats the two classes identically and concludes "neither `~new` can succeed"; measured at the source, that holds for `Ticker` and not for `Alarm` |
| **`Stream`** | `StreamClasses.orx` | `:154` | `self~!c_stream_init(stream_name)` at `:165`, declared `:127` as `stream_init` | unconditional, no `REPLY` in front of it. **The spec's list does not carry `Stream`** |
| **`File`** | `StreamClasses.orx` | `:516` | `self~qualifyImpl(path)` at `:637`, via `self~qualifiedPath` (`:633`) which `init`'s `dir == .nil` branch calls; declared `:647` as `file_qualify` | unconditional for the single-argument form, no `REPLY`. **The spec's list does not carry `File`** |

**`RexxQueue` is not on the list, and looked as though it should be.** Its `init` (`StreamClasses.orx:445`)
reaches `self~class~create` and `self~class~open` — both `EXTERNAL 'LIBRARY REXX'` at `:440` and
`:443` — but only through `if name_queue == .nil` and `if named_queue \= "SESSION"`, and the
argument defaults to `"SESSION"`. So `.RexxQueue~new` takes neither branch. It then runs
`self~objectname = named_queue`, which is the enumeration's `:147`. Measured on the oracle,
`x = .RexxQueue~new; say "constructed:" x~class~id` is rc 0 with `constructed: RexxQueue` and empty
stderr, which corroborates the source reading rather than resting on it.

**What this changes for the criterion, and it is not what the list's name suggests.** `Stream` and
`File` are already outside the spec's covered-38: measured, bare `.Stream~new` and `.File~new` are
both **rc 163, `Error 93.901: Not enough arguments for method; 1 expected.`**, so both are among the
24 that raise. The spec discusses both at
["Getting an instance, and what the criterion covers"](../specs/2026-08-17-phase-5-object-model.md#getting-an-instance-and-what-the-criterion-covers)
and gives only the representativeness reason (`'nosuch_zz.txt'` constructs against a file that does
not exist). The derivation above adds a second and harder reason: **through Phase 5 they cannot be
constructed at all**, because the natives their `init`s reach are Phase 7's. An opt-in construction
program for either would be red for a reason that is D37's rather than the class library's.

---

## The `DEFERRALS` mapping

The spec calls the deferral table "the only instrument that has produced a member of this class
without someone reading an authority end to end", and asks for it to be treated as a standing input
to the enumeration. A row per deferral: the concrete missing mechanism, and the enumeration row that
owns it or the statement that none does.

The table is `rust/crates/rexx-classes/src/native_classes.rs:171`.

**The spec says a reviewer "read `DEFERRALS` and asked why four classes were deferred". The table has
six rows.** `RexxInteger` and `NumberString` were never asked the question, and they are the two
whose mechanism turns out to be unenumerated.

| deferral | the concrete missing mechanism | who owns it |
|---|---|---|
| `QueueClass` | `RemoveMethod` of `Dimension`, `Dimensions`, `Fill`, the four sort methods, `makeString`, `toString` | enumeration `:126`. This deferral is *why* that row exists |
| `VariableReference` | `HideMethod` of `=`, `==`, `\=`, `\==`, `<>`, `><` | enumeration `:126` |
| `StemClass` | the same six `HideMethod`s, **and** a second independent reason: `~inherit`s `.MapCollection` from `CoreClasses.orx` | `:126` for the hiding; `:123` and the bootstrap milestone for the `~inherit` |
| `RexxInfo` | registered by `addToSystem`, not `addToEnvironment`; `.REXXINFO` is a live instance, not the class | **no enumeration row** — but it is owned, by the class-set criterion, which excludes it explicitly and requires the `.environment` entry to be an instance whose `~class~id` is `RexxInfo` |
| `RexxInteger` | two mechanisms. (a) not `.NAME`-reachable — `value('.INTEGER')` answers the literal `".INTEGER"`. (b) **`CLASS_CREATE_SPECIAL(Integer, "String", RexxIntegerClass)`** — the class object is constructed with the *id* `"String"`, so an Integer value's `~class` answers `String`, not `Integer` | **none.** See below |
| `NumberString` | the same two — `CLASS_CREATE_SPECIAL(NumberString, "String", RexxClass)`, with the C++'s own comment "the number string class lies about its identity" | **none.** See below |

The deferral reasons resolve: `CLASS_CREATE_SPECIAL` is defined at `memory/RexxMemory.hpp:541` and
its second parameter is the class's `id`; the two uses are `classes/IntegerClass.cpp:2066` and
`classes/NumberStringClass.cpp:74`.

---

## Mechanisms the reading found that the enumeration does not carry

The point of the exercise. Each is measured, and each is reachable in surface this crate already
runs.

### 1. A native class whose instances report a different class than the one that created them

`CLASS_CREATE_SPECIAL(name, id, className)` (`memory/RexxMemory.hpp:541`) builds the class object
with an `id` that need not be its own name. Two classes use it, and both pass `"String"`. Measured
from a fresh empty directory, three descriptors read separately:

```
say 12345~class~id      oracle rc 0  stdout String   crate rc 120  "method \"CLASS\" of class \"Object\" is not implemented (Phase 5)"
say (1.5)~class~id      oracle rc 0  stdout String   crate rc 120  same
```

The enumeration's `:148` puts `~class` and `~id` in 5a and says only that the protocol answers.
Nothing anywhere says that for an integer or a number-string value the answer is a *different class's
name*, and an implementation that answers `Integer` would satisfy every word of `:148` and diverge.
The spec's D25 row knows `RexxInteger` and `NumberString` "are not `.environment` classes at all",
which is a different fact about them.

### 2. `defineClassMethod`, the other of D39's two setup methods

`fundclasses.xml`'s `clsClass` method list comments out four `<member>`s, each with its reason in the
comment. Two of them are commented with the same sentence — **"setup.cpp: these two are special and
will be removed at the end of the image build"** — and they are `mthClassDefineClassMethod`
(`fundclasses.xml:97`, comment at `:96`) and `mthClassInheritInstanceMethods` (`:108`, comment at
`:107`). Those are D39's two setup methods, the ones `Setup.cpp:1809`'s `removeSetupMethods()`
removes.

The enumeration carries `inheritInstanceMethods` as row `:124`. **It carries no row for
`defineClassMethod`**, whose string count in the spec is 0. It is not obscure: `CoreClasses.orx:73`
is `.String~defineClassMethod(name~upper, .methods[…])`, inside the loop at `:69`-`:74` that the
spec's own migration item 6 is about. `docs/superpowers/plans/2026-08-17-phase-5a.md:1751` uses it as
a post-bootstrap check, so the plan owns the mechanism while the enumeration does not enumerate it —
which means it has no owning phase and no gate row. Measured: `.Object~hasMethod("DEFINECLASSMETHOD")`
and `("INHERITINSTANCEMETHODS")` are both **0** on the oracle, so the plan's instrument is sound.

### 3. `~copy` on a class object is 5a-reachable and documented as a refusal

`fundclasses.xml`'s `clsClass` prose: *"Note that the copy method is forbidden for Class and all other
class objects, and will result in an error."* Measured:

```
say .Object~copy
  oracle  rc 163   stdout empty
          stderr   *-* Compiled method "COPY" with scope "Class".
                   Error 93.970:  COPY method is not supported for object The Object class.
```

The enumeration's `:149` assigns `~copy` to **5b** with the reason *"it needs an instance to copy"*.
The class-object arm needs no instance, is a documented refusal, and carries the operator-frame
traceback line that is `:170`'s own subject. Under D46 a split mechanism must have both halves named
and owned; this one is split and is not named as split.

**Upstream pins it**, so this is behaviour to reproduce and not a candidate oracle defect:
`ootest/ooRexx/base/class/Class.testGroup:1007` is `::method test_class_copy`, whose whole body is
`self~expectSyntax(93.970)` followed by `c = .array~copy`. That is the second of the three signals
failing, checked at `ootest/` directly.

### 4. `.ENDOFLINE` — a Phase-5-claimed environment entry with no row

`rexxpg/classes.xml` `searchord` names `.endofline` as one of the global environment's Rexx-supplied
objects. Measured on the oracle by iterating `.environment~supplier`, the entries that do **not**
answer `~isA(.Class)` are `ENDOFLINE` (String), `ENVIRONMENT` (Directory), `FALSE` (String), `LOCAL`
(Directory), `NIL` (Object), `REXXINFO` (RexxInfo instance) and `TRUE` (String). The class-set
criterion covers classes; `:151` covers `.environment`, `.local`, `.context` and `.methods`; D42
covers `.true`/`.false`; the criterion covers `RexxInfo` explicitly. **`.ENDOFLINE` is covered by
none of them**, and the crate refuses it naming this phase:

```
say .endofline~c2x      oracle rc 0  stdout 0A
                        crate  rc 120  "rexx-exec: environment symbol \".ENDOFLINE\" is not implemented (Phase 5)"
```

`.LINE` and `.RS`, the other two `searchord` step-8 symbols with no enumeration row, were checked and
are **not** a gap: `say .line` is `1` and `say .rs` is `.RS` on both engines, rc 0.

### 5. Two authority cells that say "nothing documents it" and are wrong

Both matter beyond tidiness, because the spec's denominator argument turns on them: *"a mechanism with
no documented section is outside every denominator in this document."*

* **`:126`, native removal and hiding**, whose authority cell reads *"**nothing documents it**; found
  by reading `Setup.cpp`. `fundclasses.xml`'s `define` section describes the *effect* of hiding
  without naming the mechanism"*. `collclasses.xml:8064`-`:8067`, in `clsStem`'s own prose, reads:
  *"In addition to the methods defined in the following, the Stem class **removes** the methods `=`,
  `==`, `\=`, `\==`, `<>`, and `><` **using the DEFINE method**."* That names the class, names the
  six methods, and names the mechanism — and the six are exactly `Setup.cpp:1399`-`:1404`'s
  `HideMethod` calls. The mechanism is documented for `Stem`; it is undocumented for `Queue` and
  `VariableReference`.
* **`:152`, Directory entry methods**, whose authority cell reads *"measured; `Setup.cpp:1781` via
  `setMethodRexx`"*. `collclasses.xml`'s `clsDirectory` prose documents it with a worked example —
  `mydir~name = "Mike"` is *"same as `mydir~put("Mike", "NAME")`"* and `say mydir~name` is *"same as
  `say mydir['NAME']`"* — and `clsStringTable` carries the same example plus the sentence that
  StringTable has no `setMethod`/`unsetMethod`, which is the pair's discriminator.

### 6. The comment-stripping rule is stated for one file and needed in every file the denominator names

The spec gives the hierarchy extractor three rules, of which rule 1 is *"strip XML comments before
reading `<member>`s"*, and grounds it in `provide.xml`'s `chi` block containing exactly one comment,
the `ArgUtil` member.

**That grounding was checked and holds.** The `$GENERATED` block is delimited by
`provide.xml:839` (`START`) and `:904` (`END`); strictly between them there is exactly one XML
comment, at `:843`-`:845`, wrapping the `ArgUtil` `<member>` at `:844`; and the block holds 60 raw
`<member>`s against 59 after comment stripping, which is the spec's whole 59-versus-60 arithmetic.
The *reason* text — `<!-- commented ArgUtil (deprecated, won't document) -->`, which the spec quotes
— is a **separate** comment at `:838`, one line **above** `START` and therefore outside the block; an
extractor pointed at the block never sees it, which is why the assertion is stated over the member
comment rather than over that sentence.

**The same hazard is live in every file the method-row denominator names, and the spec's denominator
table does not carry the rule.** Measured, comparing each file's raw count against its count after
`perl -0777 -pe 's/<!--.*?-->//gs'`:

| file | `<member><xref linkend="mth…"` total | outside comments |
|---|---|---|
| `fundclasses.xml` | 255 | 249 |
| `collclasses.xml` | 354 | 345 |
| `utilityclasses.xml` | 391 | 374 |
| `streamclasses.xml` | 54 | 45 |
| `directoryclassmethods.xml` | 25 | 24 |
| `queueclassmethods.xml` | 34 | 33 |
| `supplierclassmethods.xml` | 9 | 8 |

The last three are `*classmethods.xml` include files, whose `<member>`s the denominator names
explicitly. And the section-keyed half has the hazard too, once: `<section id="mth…">` counts
250 / 346 / 372 / 45 across the four books, of which **`utilityclasses.xml`'s `mthSupplierInit`
(`:10062`) sits inside a comment opened at `:10061`**. Their sum is 1013 — the number the spec's
entity-prefix bullet uses — so **the spec's own 1013 includes the withdrawn section and the live
figure is 1012.**

### 7. Two classes whose documented method set is empty by construction

`collclasses.xml`'s `clsSetCollection` — *"This is a tagging MIXIN class only and does not define any
methods of its own"* — and `streamclasses.xml`'s `clsInputOutputStream` both print the literal
`(no class or instance methods)`. A method-row extractor needs them as named exceptions or it
produces a class with no rows and no reason.

### 8. `PRIVATE` has three documented calling contexts, not one

`rexxpg/classes.xml` `public` enumerates them: from a method owned by the same class as the target
(including other instances of that class or of a subclass); from a method defined at the same class
scope, the worked example being a class-side method calling an instance-side `private` one; and
**from an instance to a private *class* method of its own class**, the worked example being
`self~class~allocateAccountNumber` inside an instance `init`. The enumeration's `:142` records
`PRIVATE` as refused today and does not carry the shape; the third context in particular is the one
an implementation checking only "same scope" would get wrong.

### 9. The security manager has two chokepoints at directory lookup, not one

Criterion 5, restated in the spec's inventory of the old gate criteria, asks for *"exactly one
chokepoint at dispatch and one at `.local`/`.environment` lookup, each asserted by a test that fails
if a second appears."* Read at the implementation, `PackageClass::findClass` (`:1081`-`:1163`) makes
**two** separate security-manager calls — `securityManager->checkLocalAccess` at `:1137`, before
`.local`, and `securityManager->checkEnvironmentAccess` at `:1154`, before `.environment`. A test
written to the criterion's letter fails against the oracle's own shape. The eight steps that function
walks are `findInstalledClass` `:1086`, `findPublicClass` `:1095`, `TheRexxPackage->findPublicClass`
`:1109`, `packageLocal` `:1126`, `checkLocalAccess` `:1137`, `getLocalEnvironment` `:1145`,
`checkEnvironmentAccess` `:1154`, `TheEnvironment->entry` `:1162` — **and they are not the eight
`searchord` lists**, whose first step (the `.nil`/`.true`/`.false` constants) and last (the
Rexx-defined symbols `.RS`, `.LINE`, `.METHODS`, `.ROUTINES`, `.RESOURCES`, `.CONTEXT`) are outside
`findClass` entirely, while the two security-manager steps are outside the documentation entirely.
The count agreeing is a coincidence, and `:153`'s phrase "the eight-step … search order" reads as
though one set were both.

### 10. Three more classes the reference says the user cannot construct

The spec's `unreachable` status is grounded in the reference's sentence *"can only be created using
the native code application programming interfaces"*, which `clsBuffer` (`utilityclasses.xml:429`)
and `clsPointer` (`:6910`) carry, and it says those two carry the status and nothing else does.
**That claim survives the reading** — but three further classes carry a *differently worded* sentence
of the same force, each naming a Rexx-level route: `clsRexxContext` (*"cannot be directly created by
the user"*, obtainable via `.CONTEXT` or a Method/Routine `~package`-side method), `clsRexxInfo`
(*"Only one instance … can be obtained via the `.RexxInfo`, other instances cannot be created or
copied"*), and `clsStackFrame` (*"StackFrame instances cannot be directly created by the user"*,
obtainable via `.CONTEXT` or from a condition object created for a trapped condition). All three
additionally carry `<!-- new() is forbidden -->` comments.
They are `not-covered` with a documented, trivial opt-in program rather than `unreachable`, and the
extractor task should know that before it writes their reasons. `clsVariableReference` is the fourth
of this shape — *"It can only be created using a variable reference term. Calling the new method to
create a VariableReference instance is not allowed."* — which is the book agreeing with the route the
spec found by running.

### 11. `hashCode` — `ArgUtil`'s shape, one level down

`fundclasses.xml:102` comments out `mthClassHashCode` with the reason *"won't document hashCode"*,
exactly as `provide.xml:843`-`:845` comments out the `ArgUtil` `<member>` and `:838` carries its
reason. Measured, `.Object~hasMethod("HASHCODE")` is **1**. The class-set criterion has an explicit
`ArgUtil` row for the class-level case; the method half has the same case and no equivalent, and
the both-directions check cannot see it, because a name that is never emitted has no arm to
disagree about.

---

## What was not read

* **`provide.xml` and `dire.xml`** were not re-read here. Their rows are the spec author's, carried
  with the stamp they were taken at.
* **Every `mth*` section.** The stopping point for the four books is each `cls*` section's **own
  prose**, by the plan's wording, and that is what was read. The `mth*` sections are the method-row
  extractor task's subject, and the measurements above are about their markup rather than their
  content.
* **`rexxpg`'s other chapters.** Only "A Closer Look at Objects" was in scope.
* **`ootest` test bodies, except where a row above cites a line.** The mapping is a mapping; it
  opened every group it names and read the specific tests it cites, and it did not read the groups
  through.
* **The C++ beyond the cited lines.** The stopping point is the citations, and it is a bounded
  stopping point precisely because the implementation is not readable end to end. `findClass`
  (`:1081`-`:1163`) and `processInstall` (`:1227`-`:1302`) were read whole because a citation into
  each turned out to need it.
* **`ClassDirective::activate()`** (`instructions/ClassDirective.cpp:284`), `RexxInstructionForward`,
  `RexxInstructionReply::execute` and `traceMessage` are cited by the spec **without** a line, so
  there was nothing to resolve; the first was located and the other three were not.
* **`platform/windows/PlatformObjects.orx`** was opened — two lines, `call 'orexxole.cls'` — and its
  consequences for the other four CI platforms were not pursued.
* **No oracle defect is declared anywhere in this ledger.** The `ootest` mapping is what makes the
  three-signal rule's second signal checkable, and the only place that signal is applied is finding 3,
  where it comes out *against* a defect finding: `~copy` on a class object is pinned upstream, so it is
  behaviour to reproduce. Nothing here applies the first or third signal to anything.
