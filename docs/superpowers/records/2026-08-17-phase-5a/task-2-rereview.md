# Task 2, fix round 1 — re-review

Scope `237b53f39..fa244be1a`, one commit, one file (`docs/superpowers/plans/phase-5-reading-ledger.md`,
155 insertions / 55 deletions, no code).

**Verdict: REWORK.** All nine findings are closed and the two rows the round found on its own are
right at the source and on the oracle. The rework is not about the nine: it is that the central
question this round was ruled on — *do the stated procedures reproduce?* — comes out with three that
do not, and that a cheap widening of one stated search turns up a **dedicated upstream test of the
install passes** that the round missed on the very row it rewrote for exactly that reason.

---

## Per-finding

| # | | verdict |
|---|---|---|
| 1 | HIGH — row `:120` `none` | **CLOSED** — but the row's new `(:167)` is wrong; see N4 |
| 2 | HIGH — row `:136` "not asserted anywhere" | **CLOSED** — both citations resolve and say what the row claims; but see N1 |
| 3 | MEDIUM — `:126`'s headline citation | **CLOSED** — all three coordinates correct |
| 4 | MEDIUM — four counts do not reproduce | **CLOSED** — every count in the mapping re-run, all reproduce |
| 5 | LOW–MEDIUM — two unstated criteria | **CLOSED** — split into (a)/(b) with both criteria named; conservation holds; see N6 for a scope overstatement |
| 6 | LOW — `Alarm`'s `reply` line | **CLOSED** — exact |
| 7 | LOW — `clsRexxContext`'s second route | **CLOSED** — exact |
| 8 | LOW — the derivation's quote style | **CLOSED** on the pattern; the replacement command mis-numbers its own output, N3 |
| 9 | LOW — the `Alarm` prediction | **CLOSED** by labelling |
| 6a | reviewer's observation, now an item | **VERIFIED**, and wider than stated |

---

## New defects, most severe first

### N1 — HIGH. `Class.testGroup:962` `test_activate` and `base/class/class.testgroup.cls` are the direct upstream pin of the three passes, and rows `:136` and `:137` do not name them

`base/class/Class.testGroup:962` is `::method test_activate`. It creates a results directory
(`:964`), calls `.context~package~loadPackage("class.testgroup.cls")` (`:966`), and asserts at
`:968` that the loaded package set no error and at `:970`-`:972` that all three of its classes'
`activate` methods ran.

`ootest/ooRexx/base/class/class.testgroup.cls` (70 lines, same directory) is written for nothing but
this. It declares `::class class1 subclass class3`, `::class class2`, `::class class3`, each with
**both** an `::method init class` and an `::method activate class`, and its activate bodies assert:

* **every class object exists before any `activate` runs** — each of the three activates checks
  `.class1~isa(.class)`, `.class2~isa(.class)` and `.class3~isa(.class)`. That is pass 1 complete
  before pass 3 begins, asserted three times over, and it is strictly stronger than
  `REQUIRES.testGroup:320`, which pins one forward reference;
* **activate order follows the dependency graph** — `class2` first (no dependency), then `class3`,
  then `class1` (which subclasses `class3`), each direction asserted by name with its own failure
  message;
* **an instance can be created and a method sent inside `activate`** — class1's activate does
  `instance = self~new` then `instance~foo`, commented "this should not give an error";
* **the package prolog runs after every `activate`**, asserted at the top of the file.

Two consequences. Row `:136` cites `REQUIRES.testGroup:320` and `CONSTANT.testGroup:496` — both real,
both incidental — while the file upstream wrote *for this mechanism* goes unnamed; a later task
implementing `processInstall`'s three loops writes a witness from scratch and never learns that
upstream also pins activate **ordering**, which no row anywhere in the ledger mentions. Row `:137`'s
sentence "no test group carries both an `::method init class` and an `::method activate`" is true
only by file extension: this file has three classes that each carry both.

**The mechanism of the miss is `--include='*.testGroup'`.** `ootest/` holds `.rex`, `.cls`,
`.testUnit` and `.oodTestGroup` files as well, and the assertions here live in the `.cls`. Re-running
the row's own intersection with those extensions added returns exactly one file — this one. It is
the third instance of "the search was narrower than the sentence it licensed", in the rows the round
rewrote for the first two. A bare `ACTIVATE` count over `base/class/Class.testGroup` — the same
instrument the first draft ran on the other two groups — is **3** and lands on `:962`, `:965`, `:969`.

### N2 — MEDIUM. Row `:137`'s stated intersection is vacuous: its first `grep` matches nothing

The row states the negative as *"measured by intersecting
`/bin/grep -aril --include='*.testGroup' '::method[[:space:]]+.?init[[:space:]]+class'` with
`/bin/grep -aqiE '::method[[:space:]]+.?activate'`, which is empty"*.

The first command has no `-E`. Under BRE, `[[:space:]]+` requires a literal `+` and `.?` requires
any character followed by a literal `?`. Run exactly as written it returns **0 files**, so the
intersection is empty before `activate` is consulted at all. Add `-E` and it returns **26 files**,
and the intersection over those is still empty — so the *conclusion* survives, and only under the
`*.testGroup` restriction N1 is about.

This is the "checks blind to their own subject" shape, and it is the defect the round's own ruling
names: a stated procedure that does not support its claim is worse than the unstated one it replaced,
because a re-runner sees the empty set the row predicts and stops.

### N3 — MEDIUM. The unexercisable derivation's `awk` prints cumulative `NR`, so every line number after the first file is wrong

The stated command ends `{ print FILENAME":"NR" <= "cls }`. `NR` is cumulative across input files;
`FNR` is the per-file number. `CoreClasses.orx` is 4193 lines, so run as written over the three files
the derivation names, `StreamClasses.orx`'s three double-quoted externals — the table's `:510`,
`:546`, `:547` — print as

```
StreamClasses.orx:4703 <= ::CLASS "File" public inherit Comparable Orderable
StreamClasses.orx:4739 <= ::CLASS "File" public inherit Comparable Orderable
StreamClasses.orx:4740 <= ::CLASS "File" public inherit Comparable Orderable
```

`StreamClasses.orx` is 1010 lines. The paragraph introduces this as *"the derivation, mechanically,
with the pattern stated so it reproduces the table"*, and it produces `file:line` strings that look
exactly like citations and name lines that do not exist. `FNR` fixes it. (Minor, same paragraph: the
third file is `interpreter/platform/unix/PlatformObjects.orx`, not under `RexxClasses/`, so the file
list cannot be pasted from either directory as written.)

The finding-8 fix itself is good: the awk *does* find all three double-quoted directives, they are
all `File`'s, and the conclusion is unchanged.

### N4 — MEDIUM. Row `:120`'s new `test_issubclassof (:167)` is the mis-attribution finding 3 corrected, re-introduced two rows away

`:167` is `::method test_issubclassof_non_class`, whose whole body is `expectSyntax(88.914)` /
`x = .vehicle~issubclassof("object")`; it never mentions `AmphibianVehicle`. The method that reaches
the directive-declared hierarchy is `::method test_issubclassof` at **`:146`** (four `assertTrue`s
over `.Amphibianvehicle`, `:153`-`:156`). A second, byte-similar `::method test_issubclassof` at
`:1248` sits after `::CLASS "WasserFahrzeug"` (`:1229`) and before `::CLASS Vehicle` (`:1260`), so it
is a method of `WasserFahrzeug` and is not the test.

The first draft of the row named the method with **no** line number; this round added the wrong one.
The substantive claim — that the directive-declared copy does not discriminate merge order — holds:
`issubclassof` and `baseclass` say nothing about method precedence, and the `~show_off` call at
`:1245` is inside `/* … */` at `:1241`-`:1246`, confirmed.

### N5 — LOW. Row `:153`'s account of what its search returns is incomplete

The row says the stated command *"returns only the `::REQUIRES` file search order and `CALL`'s
function search order"*. Run as written it also returns three hits in
`base/rexxutil/Macrospace.testGroup` — the macro search order at `:154` and `:160`, and a `#define`
comment at `:83`. A third different subject, so the conclusion is unaffected; but the row's
enumeration is a checkable claim and it is wrong, and a re-runner has to decide whether the extra
file means the row is stale.

### N6 — LOW. List (a)'s stated criterion is false of its `Setup.cpp` members

(a) is introduced as *"Resolve to the definition line of the named function or the named
declaration"*, and the paragraph above it claims *"no citation is left in a list whose criterion is
unstated"*. `Setup.cpp` `:331`, `:332`, `:688`, `:792-804`, `:1285`, `:1307-1312`, `:1399-1404`,
`:1781` and `:1809` are statements and macro invocations inside the image build, not definitions or
declarations; `DirectiveParser.cpp:490` and `:812` are range ends (explained in the paragraph that
follows, so those are covered in substance). Consequence is small — the spec's cells for those name
macro calls rather than functions, so no reader expects a signature — but the criterion sentence
overstates what the split achieved.

### N7 — LOW, a hazard rather than a defect. Every `|` in a mapping-table pattern is written `\|`

Verified that `\|` occurs **only** inside table rows and never in prose, so it is GFM table escaping
and the rendered patterns are correct. But `/bin/grep -E` treats `\|` as a **literal pipe**
(measured: `grep -cE 'a\|b'` matches `a|b` and not `ab`), and a later task reads this document with
`sed`/`cat`, not a renderer. Pasted from the raw file, `guard (on\|off)` returns 0 and exits 1,
`~setEntry\|~entry\(` returns 0, `addClass\|addPublicClass` returns 0 — three counts reading as
upstream drift when nothing moved, which is precisely the misreading the round's preamble says this
document is where it would happen.

---

## Observation, not a defect this round introduced

`Setup.cpp:325-329` is cited by the spec at `:1359` and appears in **neither** list (a) nor (b). The
C++ authority row's stopping point is *"every `file:line` the spec cites, verified"*. Present at
`237b53f39` too — the old list carried `:331 :332` and no `:325-329` — so this is inherited, not new,
and the conservation check the round ran was over the old list rather than over the spec.

---

## Exactly which counts and negatives I re-ran, and what they returned

### Counts — 54 re-run, 54 reproduce

All are `/bin/grep -aciE '<pattern>' ootest/ooRexx/<path>` at `ootest` r13178 (`svn info ootest` →
`Revision: 13178`, confirmed; `oodocs/rexxref` and `oodocs/rexxpg` both r13198).

**The four finding-4 counts, under the newly stated `::method[^;]*\bX\b`:** `private` **12**,
`package` **12**, `protected` **6**, `unguarded` **9** in `base/directives/METHOD.testGroup` — all
four reproduce, and row `:142` reuses the same pattern for two of them.

**The two under-documented ones:** `~defineMethods?\(` **12** and `\breply\b` **33**. Both reproduce.

**Everything else in the table:** `MIXINCLASS` 52/67 · `METACLASS` 50/58 · `ABSTRACT` 16/12 ·
`98\.911` 5 · `~baseClass` 8/3 · `~define\(` 29 · `~uninherit\(` 9 (twice) · `98\.985` 7 ·
`~enhanced\(` 2/3 · `::attribute` 118 · `ABSTRACT` 50 · `93\.965` 8/1 · `::constant` 76 ·
`ACTIVATE` 5 · `::annotate` 76 · `~annotations?\b` 70 · `\.methods\b` 3/4 ·
`EXTERNAL[[:space:]]+['"]LIBRARY` 14/14/10 · `~setEntry|~entry\(` 32/12 · `objectName` 11/10 ·
`~isSubclassOf` 23/20 · `~metaClass` 9/4 · `~superClasses` 8/2 · `~identityHash` 24 · `\.context` 47 ·
`\.environment\b` 4 · `addClass|addPublicClass` 52 · `guard (on|off)` 60 · `>M>` 4 ·
`hasMethod` in `Stem.testGroup` **0** · `Compiled method` in `incorrectCharacters.testGroup` **6**.

**Item 6a's three:** class-side twins under `^[^#]*\b(RemoveClassMethod|HideClassMethod)\(` in
`Setup.cpp` **0**; `RemoveMethod(` **9**; `HideMethod(` **12**, first `:792`, last `:1404`.

### Negatives and derivations — 13 re-run, 10 reproduce cleanly, 3 do not

| stated procedure | result |
|---|---|
| `:124` `inheritInstanceMethods` tree-wide | exit **1** ✓ ; widened to `.rex`/`.cls`/`.testUnit`/`.oodTestGroup` — still exit 1 |
| `:124` the effect search (`supplier\|.set\|…~hasMethod`) | exit **1** ✓ |
| `:126` `assertFalse\([^)]*hasm(ethod)?` | exactly `API/oo/METHOD`, `base/class/Class`, `base/class/Object`, `base/directives/ATTRIBUTE` ✓ |
| `:137` the `init class` × `activate` intersection | **✗ vacuous** — first grep is BRE with an ERE pattern, 0 files; with `-E`, 26 files and still empty (N2) |
| `:137` `init…before…(inherit\|activate)` | exit **1** ✓ |
| `:139` `::method .?unknown` | exactly `SysUnicode`, `base/class/Object`, `SecurityManager` ✓ ; widened to the other extensions — same three |
| `:140` `:super` occurrences | exactly once each in `Class.testGroup`, `Object.testGroup`, `ADDRESS.testGroup` ✓ |
| `:146` `::method .?makeString` | exactly `API/oo/METHOD`, `API/oo/FUNCTION`, `extensions/json`, `base/class/Array` ✓ ; widened — same four |
| `:153` `search *order\|searchord` | **✗ incomplete** — REQUIRES and CALL as stated, plus three hits in `Macrospace.testGroup` (N5) |
| `:170` `Compiled method` | exactly `incorrectCharacters.testGroup` ✓ ; widened — same one |
| 6a `\b(RemoveClassMethod\|HideClassMethod)\(` over `interpreter/` | two `#define` lines only ✓ — and **wider than stated**: `/bin/grep -arn` for the two names over the whole repository with no `--include` and no word boundary returns the same two lines and nothing else |
| the unexercisable `awk` | finds all three double-quoted externals, all `File`'s ✓ — **✗ line numbers are cumulative `NR`** (N3) |
| the `\|` escaping convention | verified confined to table rows; `grep -E` treats `\|` as a literal pipe (N7) |

### Citations resolved at their source

**`ootest/ooRexx/base/class/Class.testGroup`** — row `:120`'s whole setup and discriminator: `:295`
(`::method "test_INIT_INHERIT_UNINHERIT_SUBCLASS_MIXINCLASS_QUERYMIXINCLASS"`), `:299`, `:306`,
`:312`, `:323`, `:326`, `:330`, `:331`, `:333`, `:384`, `:386`, `:395`, `:402`, `:406`, `:413` — all
exact, and read in context: `:386` asserts `avo_name || swim_string`, `:402` asserts
`rgf_vehicle_swim` after `~uninherit`, `:413` asserts the mixin's again after re-`inherit`. Finding
3's four: `:188` `::METHOD "test_DEFINE"`, `:198` the one-argument `~define` with its verbatim
comment, `:218` `::METHOD "test_DELETE"`, `:227` its `~define`, plus `:205`, `:226`, `:236`, `:237`.
Also `:146`/`:167`/`:172`/`:177`/`:976`/`:1002`-`:1005`/`:1007`/`:1012`/`:1241`-`:1246`/`:1276`/
`:1444`-`:1445`/`:1248`, and `:962`-`:974` for N1.

**Finding 2's two.** `REQUIRES.testGroup` `:320` `::method test_activate`, `:324` the `file~create`
with `::class test2 public` declared after `test`, `:331` the assertion. `CONSTANT.testGroup` `:486`-
`:494` the `::resource activate`, `:489` `::method activate class`, `:493` `::constant d (2 * 2)`,
`:496` `::method test_expression_activate` with both comment lines at `:497`-`:498`, `:499` the
`assertSame(4, …)`, `:480`-`:482` real-before-expression. Both rows say what the ledger claims.

**The two rows the round found on its own.** `USELOCAL.testGroup:119`-`:120` (the `use local` class
method assigning `result = 1; rc = 2; self = 3; super = 4; sigl = 5;`) and `:131`
(`assertEquals("RESULT RC SELF SUPER SIGL", testValue)`) — exact, with `:121`-`:130` in between as
the row implies. `API/oo/FUNCTION.testGroup:1053` `::method TestGetAllVariables1`, `:1055`/`:1056`
the two `assertSame`s — exact. `base/special.variables/` holds only `RESULT_RC_SIGL.testGroup`.
`Package.testGroup:782` `::method test_package_local`, `:783`, `:784`, `:786` — exact.

**The (b) point-of-interest table** — every enclosing function and every cited line resolved in
`/home/moritz/dev/repos/ooRexx/interpreter/`: `RexxClass::inherit` `:1287`, closing `:1369`, `:1322`
`// ok, now we need to have a common base class.` · `RexxClass::subclass` `:1562`, `:1631`
`new_class->sendMessage(GlobalNames::INIT, result);` · `PackageClass::findClass` `:1081`-`:1163`,
`:1086` `findInstalledClass(internalName)` · `ClassDirective::install` `:165`, `:243`
`classObject->setAnnotations(annotations)`; `resolveConstants` `:257`, `:273`
`code->setScope(classObject)` · `LanguageParser::parseQualifiedSymbol` `:3261`, `:3292` the
`new ClassResolver(…)` · `ClassResolver::evaluate` `:124`, `:135` the `traceClassResolution` call.

**List (a), sampled across every file it names** — all 19 `ClassClass.cpp` entries, `ObjectClass.cpp`
`:609 :976 :1002 :1950 :2604`, all four `MethodDictionary.cpp`, all twelve `DirectiveParser.cpp`,
`PackageClass.cpp` `:1432 :1944 :2169`, both `DirectoryClass.cpp`, `LanguageParser.cpp:1801`,
`RexxActivation.hpp:357`, `ClassClass.hpp:180`-`:189`, and every `Setup.cpp` entry. All resolve to
what the list says except in the criterion sense of N6.

**Conservation over the finding-5 split, re-derived independently from `237b53f39`:** every citation
in the old single list appears in exactly one of (a) and (b), and nothing new appears in either.
`ClassClass.cpp:1148` is in (a), as the report says.

**Findings 6, 7, 8's sources.** `CoreClasses.orx` `:1554` `guard off`, `:1555` `reply`, `:1556`
blank, `:1557` `self~!startTimer(numdays, alarmtime)`, plus `:1470 :1472 :1590 :1630 :1659 :1662
:1663 :1690` — exact. `utilityclasses.xml:7540`-`:7545` names the `.CONTEXT` environment symbol and
the `context` method of the `StackFrame` class, and nothing `~package`-side — exact.
`StreamClasses.orx` `:115 :127 :154 :165 :371 :439 :440 :443 :445 :506 :510 :516 :522 :527 :546 :547
:548 :549 :633 :637 :647` — all exact. `MethodDictionary::hideMethod` `:348`-`:351` is the whole
function with `put(TheNilObject, methodName)` at `:350`. `PackageClass::processInstall` `:1227`, the
three loops `:1276 :1285 :1294`, `:1299` `current_class->activate()`.

### Oracle runs

Two, each from a fresh `mktemp -d`, absolute paths, three descriptors read separately, never `2>&1`,
under the `ulimit -v 1048576` / `timeout -s KILL 10` wrapper. Nothing from `oracle-crashes.txt`.

* Row `:120`'s discriminator, rebuilt from `:299`-`:333`: **rc 0**, stderr empty, stdout
  `SwimCar: I swim now...` / `RGF_VEHICLE_SWIM` / `SwimCar: I swim now...`. Reproduces the ledger's
  three answers exactly.
* Row `:153`'s `test_package_local`, rebuilt from `:783`-`:787`: **rc 0**, stderr empty, stdout
  `DEF` / `DEF`.

### The five gate commands, re-run from `rust/` at `fa244be1a`

```
cargo fmt --all --check                                             EXIT=0
cargo clippy --workspace --all-targets -- -D warnings               EXIT=0
cargo test --release --workspace                                    EXIT=0
REXX_CORPUS_GATE=1 cargo test --release --workspace                 EXIT=0   106 of 106 matching
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast   EXIT=0   106 of 106 matching
```

No `FAILED` line in any of the three test logs. `REXX_PHASE_GATE=5a …` not run and not asked for —
the constant does not exist in the tree, so the command would exit 0 for an unrelated reason. No
sitting is owed: the diff touches `docs/` only.

---

## What I could not check

* **Whether the eleven-plus-one mechanisms are all there are.** Unchanged from the first review, and
  N1 is the shape of what this instrument can find: an authority nobody opened. I opened one file
  type nobody had searched and it held a dedicated test. I did not read the four books or `ootest`
  through, so a second `.cls`, `.rex` or `.testUnit` witness elsewhere would look exactly like the
  rows that survived.
* **Whether `class.testgroup.cls`'s assertions pass upstream.** I read it and resolved its loader; I
  did not run ooTest. Its bodies are `if … then call setError`, so a failure would surface through
  `Class.testGroup:968`'s message rather than as a raise.
* **The rows this round did not touch** — `:117`, `:121`, `:123`, `:125`, `:129`, `:133`/`:134`,
  `:144`, `:145`, `:151`. Verified in the first review; I re-ran `:140`'s `:super` negative only
  because it was cheap.
* **`provide.xml`, `dire.xml`, and every `mth*` section.** Out of this round's diff.
* **The `Alarm` transcript prediction.** Not runnable in Phase 5, by construction, and now labelled
  as the prediction it is.
* **Whether the three in-tree Rust citations were right when the spec was written.** Same limit the
  first review recorded; unchanged by this round.

No file was edited except this review. No subagents were dispatched.
