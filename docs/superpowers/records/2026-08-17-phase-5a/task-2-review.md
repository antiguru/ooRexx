# Task 2 review — the reading ledger

Scope `e52bff06a..237b53f39`, one commit, one new file
(`docs/superpowers/plans/phase-5-reading-ledger.md`, 480 lines added, no code).

**Spec compliance: APPROVE.**
**Task quality: REWORK.**

Nine findings: two high, two medium, five low. The high ones are both in the `ootest` mapping and
are both the same shape — a row recorded as `none`/`not asserted` where upstream carries a
discriminating test. That mapping is the one thing this task adds that nothing else supplies, and a
false `none` is the dangerous direction: it hands a later task the three-signal rule's second signal
where the signal is not there.

Everything else held up under a hard sample. All four C++ corrections are right on **both** halves.
All three in-tree Rust corrections are right on both halves. Every one of the eleven new mechanisms
is real, is genuinely absent from the enumeration, and is 5a's. Every measurement I re-ran outside
finding 4 reproduced to the digit. The five gate commands reproduce with the exit statuses reported.

---

## Findings, most severe first

### 1. HIGH — `ootest` row `:120` records `none`; upstream carries a discriminating pair

The row reads **none** for "merge order: a class's own methods precede its superclasses' **and its
mixins'**", and adds: *"Searched the tree for a test naming precedence between a superclass's method
and a mixin's; the only hits are scope-override error tests (`93.957`) … which are a different
question."*

The row is built entirely around the **directive-declared** `AmphibianVehicle` at
`ootest/ooRexx/base/class/Class.testGroup:1276`. The same file builds a **second, dynamic** copy of
that hierarchy in `::method "test_INIT_INHERIT_UNINHERIT_SUBCLASS_MIXINCLASS_QUERYMIXINCLASS"`
(`:295`), and that one discriminates:

* `rgf_vehicle~define("SWIM", …)` at `:312` puts `SWIM` on the superclass chain;
* `rgf_water_vehicle = rgf_vehicle~mixinclass(…)` defines its own `swim` at `:326`;
* `rgf_amphibian_vehicle = rgf_road_vehicle~subclass(…)` then `~inherit(rgf_water_vehicle)`
  (`:330`-`:331`);
* `:386` asserts `avo~swim` is the **mixin's** answer, `:402` asserts that after `~uninherit` it is
  the **superclass chain's**, and `:411`-`:413` asserts it flips back on re-`inherit`.

Measured on the oracle from a fresh empty directory, reproducing that hierarchy: `swim` is
`SwimCar: I swim now...` while the mixin is inherited and `RGF_VEHICLE_SWIM` after `uninherit`
(rc 0, empty stderr). An implementation with superclass-versus-mixin backwards answers
`RGF_VEHICLE_SWIM` at `:386` and the test goes red. `:384` covers the row's other half the same way:
the amphibian's own `SHOW_OFF` beats `rgf_vehicle`'s inherited one.

Two smaller inaccuracies ride along: "the only assertion over it is `test_issubclassof`" omits
`test_BASECLASS:177` (`.AmphibianVehicle~baseclass`, which is row `:119`'s mechanism), and the
`.AmphibianVehicle~new("SwimCar")~show_off` at `:1245` that looks like a third is inside a `/* … */`
block (`:1241`-`:1246`), so it never runs — that part of the row's reasoning is right.

**Consequence.** The global constraints say "Task 2 is what makes the second signal checkable". A
later task applying the three-signal rule to a merge-order divergence reads `none`, gets signal 2 for
free, and may record oracle behaviour as a defect. Separately, the merge-order task is told there is
no upstream witness and will write one from scratch, when `:379`/`:386`/`:402` is a discriminator it
could lift straight into a corpus program.

### 2. HIGH — `ootest` row `:136` says the three passes are "not asserted anywhere"; both groups it names assert pass ordering

The row reads: *"the cycle only … The **three passes** are not asserted anywhere; `ACTIVATE` appears
in `base/directives/REQUIRES.testGroup` (7) and `CONSTANT.testGroup` (5) **as a method name, not as a
pass-ordering claim**."* Both named groups contain an explicit pass-ordering assertion.

* `base/directives/REQUIRES.testGroup:320` `test_activate` writes a requires file
  `("::class test public", "::method activate class", ".local~activatetest = .test2",
  "::class test2 public")` (`:324`) and asserts at `:331` that `.local~activateTest` is the `test2`
  class object. `.test2` is declared **after** `test` in the same file, so `activate` resolving it
  requires that every class be created before any `activate` runs — pass 3 after pass 1. A
  create-then-activate-each implementation fails it.
* `base/directives/CONSTANT.testGroup:496` `test_expression_activate` carries the comment
  *"expression constants are evaluated at class creation time / they should already be availabe to
  the class activate() method"* and asserts `4` through a `::method activate class` (`:489`) reading
  `::constant d (2 * 2)` (`:493`) — pass 2 (`resolveConstants`) before pass 3 (`activate`). The
  neighbouring `:480`-`:482` pins real constants before expression constants.

Together those two pin the three-pass structure the row says nothing pins.

**Consequence.** Identical to finding 1, on the row the spec's own `processInstall :1268-1298`
correction is about. It also makes the correction in the ledger's own C++ table less useful than it
should be: the ledger correctly extends the range to `:1301` to include
`current_class->activate()`, then tells a later reader nothing upstream exercises it.

### 3. MEDIUM — `:126`'s own headline citation is wrong on all three coordinates

The row says the `.nil`-tombstone mechanism is pinned by *"`.test_a~define("testmethod")` at `:205`
and `:236` (inside `test_issubclassof_non_class`, `:167`) with the comment "make it unaccessible for
new instances""*. At `ootest` r13178:

| the ledger says | what is there |
|---|---|
| `.test_a~define("testmethod")` at `:205` | `:205` is `self~assertFalse(o2~hasmethod("TESTMETHOD"))`. The `define` call, with that verbatim comment, is `:198` |
| … and at `:236` | `:236` is the same `assertFalse`, on `.test_b2` not `.test_a`. The second `define` call is `:227` |
| inside `test_issubclassof_non_class`, `:167` | `:167` is `::method test_issubclassof_non_class`, whose whole body is `expectSyntax(88.914)` / `x = .vehicle~issubclassof("object")`. The `define` calls are in `::METHOD "test_DEFINE"` (`:188`) and `::METHOD "test_DELETE"` (`:218`) |

The comment text is quoted verbatim from `:198`, so the file was opened; the line numbers were taken
from the observables and the enclosing method was mis-attributed.

**Consequence.** `:126` is the row the ledger leans on hardest — it is the mechanism the `DEFERRALS`
table produced, and the ledger's whole argument for the reading instrument. A later task resolving
the citation lands in an unrelated argument-type error test and has to re-derive the claim, or
concludes it is unsupported. The substantive claim survives — `:198`/`:227` are one-argument
`~define`s, and `:1002`-`:1004`'s 97.1 observable is exactly where the row says it is.

### 4. MEDIUM — four `ootest` counts do not reproduce under the method the ledger states

The mapping says: *"the counts beside them are `/bin/grep -aciE` measurements over that file."* For
`base/directives/METHOD.testGroup`, rows `:130` and `:142` give `PRIVATE` 12, `PACKAGE` 12,
`PROTECTED` 6, `UNGUARDED` 9. Measured at r13178:

| pattern | PRIVATE | PACKAGE | PROTECTED | UNGUARDED |
|---|---|---|---|---|
| `-aciE` as stated | 80 | 83 | 29 | 18 |
| `-acE` (case-sensitive) | 6 | 8 | 2 | 1 |
| `-aciE '\bX\b'` | 19 | 26 | **6** | 11 |
| `-aciE '^::method[^"]*\bX\b'` | 4 | 5 | 2 | 5 |
| ledger | 12 | 12 | 6 | 9 |

Only `PROTECTED` reproduces, and only under a pattern the ledger does not state. Two other cells are
under-documented rather than wrong: `~defineMethods` 12 needs `~defineMethods\(` (the bare word gives
13), and `reply` 33 needs `\breply\b` (the bare word gives 51).

For contrast, every other count I re-ran reproduced exactly under the stated method: `MIXINCLASS`
52/67, `METACLASS` 50/58, `ABSTRACT` 16/12, `98.911` 5, `98.985` 7, `~isSubclassOf` 23/20,
`~metaClass` 9/4, `~superClasses` 8/2, `~baseClass` 8/3, `~define(` 29, `~uninherit(` 9,
`~enhanced(` 2/3, `.methods` 4/3, `objectName` 11 and 10, `identityHash` 24, `::annotate` 76,
`~annotation(s)` 70, `::attribute` 118, `ABSTRACT` 50, `EXTERNAL 'LIBRARY` 14/14/10, `93.965` 8/1,
`::constant` 76, `ACTIVATE` 7/5, `addClass|addPublicClass` 52, `~setEntry|~entry(` 32/12,
`guard on|off` 60, `.context` 47, `>M>` 4, `Compiled method` 6, `NOMETHOD` 3/2/2.

**Consequence.** The header says the staleness check against these stamps is Task 3's. Task 3
re-deriving these four rows at the same `ootest` revision gets a disagreement with no upstream
change, which reads as staleness and is not.

### 5. LOW–MEDIUM — the "checked and correct" C++ list uses a looser criterion than the "found wrong" list, unstated

The ledger calls `createClassBehaviour :1119` **wrong** on this reasoning: it lands inside the
function, and *"its sibling in the same cell … *is* the definition line, so the pair is inconsistent
as written."* That standard is not applied to the list it says a re-checker should not redo:

* `ClassClass.cpp:1322` — spec `:119` cites `inherit() :1322`. `RexxClass::inherit` is `:1287`, the
  function runs to `:1369`, and `:1322` is `// ok, now we need to have a common base class.`. Its
  sibling in the same cell, `mixinClass() :1514`, **is** a definition line. Identical shape.
* `ClassClass.cpp:1631` — spec `:137` cites `RexxClass::subclass :1631`. `RexxClass::subclass` is
  `:1562`; `:1631` is the `sendMessage(GlobalNames::INIT, result)` inside it.
* `PackageClass.cpp:1086` — spec `:153` cites `PackageClass::findClass :1086`. `findClass` is
  `:1081`; `:1086` is `findInstalledClass`, which the ledger's own finding 9 lists as *step one* of
  eight.

Each is defensible as a point-of-interest citation; none is defensible under the criterion the ledger
used four paragraphs earlier. **Consequence:** the list is introduced as "listed because a re-checker
should not redo them", so a later task takes them on trust and resolves three of them into function
interiors while expecting definition lines — which is the failure mode the ledger says makes
`createClassBehaviour :1119` worth correcting.

### 6. LOW — `Alarm`'s `reply` line is off by one, and "one line before" is wrong

The unexercisable table says *"The `reply` is at `:1556`, one line *before* the native call."* At
`interpreter/RexxClasses/CoreClasses.orx`: `guard off` is `:1554`, `reply` is **`:1555`**, `:1556`
is blank, `self~!startTimer(numdays, alarmtime)` is `:1557`. Two lines before, with a blank between.
The substantive claim — the `reply` precedes the native call, unlike `Ticker`'s — holds, and
`Ticker`'s own line numbers (`:1630`, `:1659`, `:1662`, `:1663`, `:1690`) are all exact.

### 7. LOW — finding 10 names the wrong Rexx-level route for `clsRexxContext`

The ledger's parenthetical is *"obtainable via `.CONTEXT` or a Method/Routine `~package`-side
method"*. `utilityclasses.xml:7540`-`:7545` says instances *"can only be
obtained via the `.CONTEXT` environment symbol, or by invoking the `context` method of the
`StackFrame` class"*. No `~package`-side route is documented. `.CONTEXT` works, so the opt-in program
the finding recommends is unaffected; the reader looking for the second route is not.

Everything else in finding 10 verified exactly: the three sentences exist at the claimed places
(`clsRexxContext` "They cannot be directly created by the user."; `clsRexxInfo` "Only one instance …
other instances cannot be created or copied."; `clsStackFrame` "StackFrame instances cannot be
directly created by the user."), all three carry a `new()`-forbidden XML comment (`:8004` for
RexxInfo, which is not in the first lines of its section), `clsVariableReference` is the fourth of the
shape, and the `not-covered`-versus-`unreachable` distinction is the right one: the spec grounds
`unreachable` in *"instances come only from native code"* (`utilityclasses.xml:429`, `:6910`,
both confirmed at those exact lines with their sections opening at `:421` and `:6902`), and none of
the three says that.

### 8. LOW — the unexercisable derivation's stated pattern misses the two externals its own criterion exempts

The derivation is *"every `EXTERNAL 'LIBRARY REXX …'` method directive"* — single-quoted, as written.
`StreamClasses.orx` has three double-quoted ones: `:510` `external "library REXX"`, and `:546`/`:547`
`external "LIBRARY REXX file_separator"` / `"…file_path_separator"` — precisely the two the criterion
names as the ones D37 implements. Measured, all three are `File`'s and `File` is already on the list
for `file_qualify`, so no member is missed and the conclusion stands. A later task re-running the
derivation as written gets a different denominator than the one that produced the table.

### 9. LOW / informational — the `Alarm` conclusion is a prediction, labelled "measured at the source"

D55 says a `REPLY` hands its value to the sender and the body carries on, and names the transcript
shape "exit status 0 with a traceback on stderr, because the raise happens after the main program has
finished" — verified at spec `:1260`-`:1269`. The ledger's inference that `Alarm~new` therefore
succeeds is sound and is a genuine correction to the spec's "neither `~new` can succeed". But *when*
the continuation runs in a single-threaded Phase 5 is a design choice no task has made, so "after the
main program has finished" is carried over from D55's two-program measurement rather than derived for
this case. Whoever writes the `Alarm` row should measure rather than inherit it.

---

## Observations, not defects

* **The class-side twins are never called.** The spec's `:126` names "`RemoveMethod` / `HideMethod`,
  **and their class-side twins**". `RemoveClassMethod`/`HideClassMethod` are defined at
  `memory/Setup.cpp:360`-`361` and, measured, there is **no call site** anywhere in `Setup.cpp` —
  the twenty-one calls are Queue's nine `RemoveMethod`s (`:792`-`:804`) and two blocks of six
  `HideMethod`s (`:1307`-`:1312`, `:1399`-`:1404`). The ledger was not asked for this, but it is the
  kind of thing the reading instrument exists to catch, and it would save the `:126` task from
  building machinery the image build has no user for.
* **Set cardinalities.** The ledger writes "comments out four `<member>`s", "`::CLASS`'s seven",
  "`::METHOD`'s twelve", "`::ATTRIBUTE`'s thirteen", "the same six". I do not treat these as
  violations: the constraint as written is scoped to comments ("A comment may not name the size of a
  set"), every one of these is a measurement, and each names its set in the same sentence or the
  adjacent one. Flagging them would be the over-application the project has already paid for once.
* **`REXX_PHASE_GATE` not run — the reasoning is right.** Reproduced:
  `/bin/grep -arn 'REXX_PHASE_GATE\|CLOSED_PHASES' crates/ --include=*.rs` exits **1**, and the two
  names appear nowhere else in `rust/`. Setting the variable would produce a green run whose green
  has nothing to do with what the variable selects — the "checks blind to their own subject" shape.
  Declining and saying so, rather than running it and reporting `EXIT=0`, is the correct call.

---

## Exactly which rows I sampled and resolved

**Authority rows.** All six, for reader / stopping point / revision — all six carry all three.
Revisions confirmed independently: `svn info oodocs` is `E155007: … not a working copy`;
`oodocs/rexxpg` **13198**, `oodocs/rexxref` **13198**, `ootest` **13178**. The C++ row confirmed by
`git log` (`cb9563364`, branch `concurrency-dispatch-fixes`), `git status` (one modified file),
`git diff -U0` (10 insertions / 1 deletion in `copyIfNecessary()` at `@@ -3661`),
`git merge-base cb9563364 e52bff06a` = `cb9563364`, `git diff cb9563364 e52bff06a -- interpreter/`
empty, and `diff -rq` naming `NumberStringClass.cpp` as the only difference. The
`rexxpg/en-US/classes.xml` stopping point checked: `<chapter id="classes">` at `:46`, `</chapter>` at
`:1500`, file is 1500 lines.

**C++ "found wrong" — all four, both halves.**
`LanguageParser.cpp:610` blank and no `addMethod` definition in that file; `DirectiveParser.cpp:610`
is the definition. `ClassClass.cpp:1119` is `// Object is a special case, since it is top dog.`;
`createClassBehaviour` is `:1098`, `createInstanceBehaviour` is `:1148`. `PackageClass::processInstall`
is `:1227`; `:1298` is the `get(i)` line and `:1299` is `current_class->activate()`; the class block
is `:1268`-`:1301` and the three loops are `:1276`, `:1285`, `:1294`. `attributeDirective` is `:1457`,
closes at `:1848`, `:1850` blank.

**In-tree Rust "found wrong" — all three, both halves.**
`lib.rs:1275`-`:1277` is the `::CLASS SUBCLASS naming a namespace` arm; the `::ANNOTATE` comment is
`:1285`-`:1287` and the arm `:1288`-`:1292` with `gap(…)` at `:1291`. `corpus.rs:244` is `.to_str()`,
`:246` is `Invocation::none()`. `oracle.rs`: `did_not_finish` at `:188`, its `!matches!` body at
`:189`, `expect_exit_code`'s panic at `:214`, and the only `unwrap_or(-1)` in that directory is the
doc comment at `:148`.

**"Checked and correct" C++ list — sampled 60 citations across every file it names.**
`ClassClass.cpp` `:134 :343 :518 :558 :654 :819 :952 :984 :1036 :1071 :1210 :1287 :1322 :1379 :1440
:1514 :1631 :1741 :1754 :1854 :1882`; `ObjectClass.cpp` `:609 :659 :697 :866 :919 :976 :1002 :1235
:1302 :1696 :1733 :1760 :1829 :1891 :1950 :2185 :2489 :2579 :2604`; `MethodDictionary.cpp` `:164
:348 :434 :594`; `DirectiveParser.cpp` `:287 :334-490 :629-812 :948 :1940 :2266 :2438 :2518 :2565
:2779`; `PackageClass.cpp` `:1086 :1432 :1944 :2169`; `DirectoryClass.cpp` `:480 :591`;
`LanguageParser.cpp` `:1801 :3292`; `ClassDirective.cpp` `:243 :257 :273 :284`;
`RexxActivation.hpp:357`; `ExpressionClassResolver.cpp:135`; `Setup.cpp` `:185 :331 :332 :360-361
:371-372 :396 :688 :792-804 :1285 :1307-1312 :1399-1404 :1781 :1809`; `ClassClass.hpp:180-189`.
All resolve; three carry the criterion problem in finding 5. The two range notes verified exactly:
`classDirective` runs to `:495` and `::CLASS`'s seven `SUBDIRECTIVE_` arms are `:396 :408 :420 :431
:443 :456 :477`; `methodDirective` runs to `:942`, its twelve arms are `:674 … :796`, `:812` is the
blank before `default:` at `:814`; `::ATTRIBUTE`'s thirteen are `:1500 … :1627`.

**`.orx` and XML list.** `CoreClasses.orx` `:73 :93 :151 :987 :1299 :1300 :1557 :1590 :1659 :1690
:2184 :3840 :3974 :3976 :3996`; `StreamClasses.orx` `:115 :371 :546 :547 :548 :549`; `provide.xml`
`:59 :838 :843 :844 :845 :849 :839/:904`; `utilityclasses.xml` `:429 :6910`. All resolve.

**The false sentence.** `sed -n '114,172p'` of the spec piped to `/bin/grep -ac ootest` is **0**,
exit 1. `testGroup` occurs at spec `:822` and `:1096` only, neither in an enumeration row (the table
is `:114`-`:172`, confirmed row by row). The sentence is at spec `:1342`-`:1344`. The spec is
unmodified in this commit — the ledger records rather than edits, as the plan specifies. The
`ANNOTATE.testGroup` claim at `:1096` verified.

**`ootest` mapping.** All **29** distinct group paths named in the mapping confirmed to exist under
`ootest/ooRexx/`. **32 of the 41** mapping rows resolved at line or count level: `:116 :118 :119
:120 :121 :122 :123 :124 :125 :126 :127 :128 :130 :131 :132/133/134 :135 :136 :137 :139 :140 :142
:143 :145 :146 :147 :148 :150 :151 :152 :169 :170 :171`. Line-level checks that resolved exactly:
`Class.testGroup` `:976 :977-982 :984-986 :996-1000 :1002-1004 :1007 :1012 :1276 :1444-1445`;
`CLASS.testGroup` `:355 :379 :386 :390 :401 :404 :415 :421 :432 :438 :449`; `Object.testGroup`
`:1487 :1493`; `Message.testGroup:372`; `Orderable.testGroup:441`; `Queue.testGroup:452`;
`MethodArgs.testGroup:137`; `VarRef.testGroup:134`; `TRACE_TraceObject.testGroup:455-468`;
`environmentEntries.testGroup` `:69 :76`. Absence claims re-run: `inheritInstanceMethods` **0** files
tree-wide; `::method unknown` in exactly SysUnicode / Object / SecurityManager; `::method makeString`
in exactly API/oo/METHOD, API/oo/FUNCTION, extensions/json, base/class/Array; `Compiled method` in
exactly `incorrectCharacters.testGroup`; `93.957` in exactly FORWARD / Object / Message;
`Stem.testGroup` has no `hasMethod`; no `assertFalse(…hasMethod…)` tree-wide names any of the 21
`Setup.cpp` removed/hidden names. **Coverage over the 5a rows is complete** — I walked spec `:114`-
`:172` row by row and every row whose phase column names 5a has a mapping row; `:149` is correctly
absent as 5b.

**Unexercisable list.** All four members plus the exclusion, at source and on the oracle.
`Ticker` `:1630 :1659 :1662 :1663 :1690`; `Alarm` `:1470 :1472 :1554 :1555 :1557 :1590`;
`Stream` `:127 :154 :165`; `File` `:506 :516 :522 :527 :633 :637 :647`; `RexxQueue` `:439 :440 :443
:445 :447 :448`. The `CoreClasses.orx` externals belong to `Alarm` and `Ticker` and to no other
class, confirmed against the `::CLASS` line numbers. `platform/unix/PlatformObjects.orx` is one
comment line, `platform/windows/` is two lines with `call 'orexxole.cls'`, and those four are the
only `.orx` files in `interpreter/`. Oracle: `.RexxQueue~new` rc 0 / `constructed: RexxQueue` /
empty stderr; `.Stream~new` and `.File~new` both rc 163 with `Error 93.901: Not enough arguments for
method; 1 expected.` D55 read at spec `:1260`-`:1269`; the spec's carried open question read at
`:1178` and its Stream/File representativeness reason at `:505`-`:507`.

**`DEFERRALS`.** All six rows against `rexx-classes/src/native_classes.rs:171` — the table is at that
line and has exactly `RexxInteger`, `NumberString`, `RexxInfo`, `QueueClass`, `VariableReference`,
`StemClass`, and each row's stated mechanism matches its committed `reason`. Queue's nine
`RemoveMethod` names match `Setup.cpp:792`-`:804` exactly; the six `HideMethod`s match `:1307`-`:1312`
and `:1399`-`:1404`. `CLASS_CREATE_SPECIAL` is `memory/RexxMemory.hpp:541` with `id` as its second
parameter, used at `classes/IntegerClass.cpp:2066` and `classes/NumberStringClass.cpp:74`, both
passing `"String"`. The "lies about its identity" comment is `memory/Setup.cpp:296`. The spec's "four
classes were deferred" is at `:788` and its four are named at `:579`.

**All eleven new mechanisms.** (1) oracle `12345~class~id` and `(1.5)~class~id` both `String` rc 0,
crate rc 120 `method "CLASS" of class "Object" is not implemented (Phase 5)`; D25's "not
`.environment` classes at all" at spec `:1120`. (2) `fundclasses.xml:96`/`:97` and `:107`/`:108` carry
the identical comment; `defineClassMethod` string count in the spec is **0**; `CoreClasses.orx:73`
inside the loop at `:70`-`:74`; plan `:1751`; oracle `hasMethod("DEFINECLASSMETHOD")` and
`("INHERITINSTANCEMETHODS")` both **0**. (3) oracle `say .Object~copy` rc 163, stdout empty, stderr
carrying both `*-* Compiled method "COPY" with scope "Class".` and `Error 93.970`, and
`Class.testGroup:1007` `test_class_copy` is `expectSyntax(93.970)` / `c = .array~copy`. (4) the
`.environment~supplier` sweep reproduces exactly seven non-class entries — `ENDOFLINE` (String),
`ENVIRONMENT`, `FALSE`, `LOCAL`, `NIL`, `REXXINFO`, `TRUE`; crate rc 120 on `.endofline~c2x`, rc 0
and agreeing on `.line` and `.rs`; `searchord` names `.endofline` at `rexxpg/classes.xml:880` and its
step-8 list is `.RS .LINE .METHODS .ROUTINES .RESOURCES .CONTEXT`. (5) `collclasses.xml:8064`-`:8067`
reads exactly as quoted, naming Stem, the six methods and DEFINE; `clsDirectory`'s worked example is
`:4031`-`:4033`, inside the `cls*` section (`:3943`) and before its first nested `mth*` section, and
`clsStringTable` carries the twin example plus *"StringTable does not provide methods setMethod and
unsetMethod"*. (6) **every one of the seven comment-stripping rows reproduces to the digit** —
255/249, 354/345, 391/374, 54/45, 25/24, 34/33, 9/8 — as do the section counts 250/346/372/45
summing to **1013**, the spec's use of 1013 at `:429`, and `mthSupplierInit` at
`utilityclasses.xml:10062` inside a comment opened at `:10061`, so the live figure is 1012. The
`provide.xml` grounding also holds: `$GENERATED` START `:839` / END `:904`, exactly one comment
strictly inside, at `:843`-`:845` wrapping the member at `:844`, 60 raw against 59 stripped, reason
comment outside at `:838`, first `&nbsp;` at `:849`. (7) both sentences and both
`(no class or instance methods)` literals confirmed. (8) `rexxpg/classes.xml:1192`-`:1239` has exactly
three `<listitem>`s with the `self~class~allocateAccountNumber` example as the third. (9)
`findClass` `:1081`-`:1163` with `checkLocalAccess` `:1137` and `checkEnvironmentAccess` `:1154`, all
eight steps at the lines listed; criterion 5's wording at spec `:1198`-`:1199` quoted exactly;
`searchord` has eight items whose first is the constants and whose last is the Rexx-defined symbols,
neither inside `findClass`. (10) covered in finding 7. (11) `fundclasses.xml:102`/`:103`;
`hasMethod("HASHCODE")` is **1**; the class-set criterion's `ArgUtil` row at spec `:372` / `:1078`.

**The five gate commands**, from `rust/`, reproduced at `237b53f39`:

```
cargo fmt --all --check                                             EXIT=0
cargo clippy --workspace --all-targets -- -D warnings               EXIT=0
cargo test --release --workspace                                    EXIT=0
REXX_CORPUS_GATE=1 cargo test --release --workspace                 EXIT=0   106 of 106 matching
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast  EXIT=0   106 of 106 matching
```

No failures or panics in either corpus run. No sitting is owed: the diff touches only `docs/`.

Oracle runs used the wrapper with three descriptors read separately, never `2>&1`, from fresh
`mktemp` directories with absolute paths. Nothing in `rust/corpus/oracle-crashes.txt` was run. No
file was edited except this review.

---

## What I could not check

* **The ledger's own denominator, which is the hole it names in its own header.** I sampled the rows
  it wrote; I did not read the four books' `cls*` prose end to end, so I cannot say whether a twelfth
  mechanism is sitting in a section the reading passed over. Findings 1 and 2 show the instrument
  this review *can* apply — open the file the row names — and it found two misses; it cannot find a
  mechanism no row names.
* **Whether the eleven are the eleven.** Each one I checked is real. Whether the reading that
  produced them was exhaustive over its stopping point is exactly what the ledger says it cannot
  assert either.
* **`provide.xml` and `dire.xml`.** Carried with the spec author's stamp in both documents. I
  confirmed the spec's reading table (`:815`-`:822`) says `done` for both with the same stopping-point
  wording the ledger repeats, and did not re-read either chapter.
* **Whether the three in-tree Rust citations were correct when the spec was written.** The ledger
  frames them as "three that the tree has moved out from under" and attributes the movement to
  Task 1. I verified they are wrong **today**; I did not resolve them against `b360783cb` to confirm
  they were right then. If one was already wrong, the framing is off but the correction is not.
* **The nine mapping rows I did not resolve at line or count level** — `:117 :129 :138 :141 :144
  :153 :154 :155 :160` beyond confirming their groups exist and, for `:160`, their counts. Given
  findings 1 and 2, the `none`/partial verdicts among them (`:141`, `:153`) are the ones a re-checker
  should open next.
* **Whether `test_INIT_INHERIT_UNINHERIT_SUBCLASS_MIXINCLASS_QUERYMIXINCLASS` passes upstream.** I
  did not run ooTest. I reproduced its hierarchy on the oracle and confirmed the answers its
  assertions demand, which is what makes it a discriminator; I did not confirm the suite is green.
* **The `Alarm` prediction.** Not runnable in Phase 5 today, by construction — see finding 9.
