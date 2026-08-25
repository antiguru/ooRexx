# Task 2 report — the reading ledger

**Status:** done. Ledger committed at `237b53f39` as
`docs/superpowers/plans/phase-5-reading-ledger.md`. Base `e52bff06a`. No code, no sitting.

The ledger carries the detail and is the artifact a later reader is meant to find. This report says
what was done, what it cost, and the things that belong in a report rather than in the ledger.

---

## What I read, and to what stopping point

| authority | stopping point | reached |
|---|---|---|
| `fundclasses.xml`, `collclasses.xml`, `utilityclasses.xml`, `streamclasses.xml` | every `cls*` section's **own prose** | yes — extracted mechanically (each `<section id="cls…">` from its opening tag to its first nested section) and read through. `/bin/grep -ac '<section id="cls'` gives 7 / 17 / 35 / 4 and the extractor emitted 63 headers, which is their sum |
| `rexxpg/en-US/classes.xml` | every section of "A Closer Look at Objects" | yes — `:46` to the closing `</chapter>` at `:1500`, all of it |
| `ootest/` | one row per 5a mechanism, naming the group that pins it or none | yes — the mapping is in the ledger. Every group named was opened; the specific tests cited were read |
| the C++ | every `file:line` the spec cites, verified | yes — resolved, not read |

**Revisions, checked rather than assumed.** `oodocs/rexxpg` r13198, `oodocs/rexxref` r13198,
`ootest/` r13178. `svn info oodocs` is `E155007: not a working copy` — the parent carries nothing and
the two subdirectories are separate working copies.

**The C++ tree needed a note.** `/home/moritz/dev/repos/ooRexx` is on branch
`concurrency-dispatch-fixes` at `cb9563364` **with one uncommitted change** — a ten-line hunk in
`NumberStringClass.cpp`'s `copyIfNecessary()` — which `diff -rq` shows is the only file differing
from this worktree's `interpreter/`. It touches nothing cited. `git merge-base cb9563364 e52bff06a`
is `cb9563364` and `git diff cb9563364 e52bff06a -- interpreter/` is empty, so the tracked C++ is the
same at both and every citation resolves identically from either.

---

## Citations found wrong: **four in the C++, three stale in-tree, one false sentence**

### The four C++ ones

1. **`LanguageParser::addMethod :610`** (spec `:138`) — **wrong file.** `LanguageParser.cpp:610` is
   blank and that file has no `LanguageParser::addMethod` definition at all. It is
   **`DirectiveParser.cpp:610`**. This is the worst of the four, because the line number is right, so
   a reader resolving it lands on a plausible-looking blank and may conclude the file moved.
2. **`createClassBehaviour :1119`** (spec `:117`) — lands 21 lines inside the function. The function
   is **`ClassClass.cpp:1098`**, and its sibling in the same cell (`createInstanceBehaviour :1148`)
   *is* the definition line, so the pair is inconsistent.
3. **`processInstall :1268-1298`** (spec `:136`) — the function is `PackageClass.cpp:1227`, and the
   range's end at `:1298` stops one line short of `current_class->activate()` at `:1299`, which is
   the third pass's own call and the thing the row's phrase "three passes" is about.
   **`:1268`-`:1301`**.
4. **`attributeDirective :1457-1850`** (spec `:131`) — the function closes at `:1848`; `:1850` is
   blank. **`:1457`-`:1848`**.

### Three in-tree Rust citations the tree has moved out from under

The spec was written at `b360783cb`; Task 1 landed after.

* `rexx-exec/src/lib.rs:1275`-`:1281` for the `::ANNOTATE` arm is now the `::CLASS SUBCLASS naming a
  namespace` arm. The `::ANNOTATE` arm is **`:1285`-`:1292`**.
* `corpus.rs:244` for `Invocation::none()` is **`:246`**.
* `tests/support/oracle.rs:189` / `:214` — **this is the instructive one.** Both lines still land on
  the did-not-finish subject, so the citation reads as correct; but the code the spec's sentence
  describes (`exit_code: output.status.code().unwrap_or(-1)`) no longer exists. There is a
  `Termination` enum, `did_not_finish` at `:188`, and a panic at `:214`. The only `unwrap_or(-1)` in
  that directory is inside a doc comment at `:148` explaining why it is *not* used. **The structural
  check the spec asks the plan to build already exists.** A stale citation that lands on the right
  neighbourhood is harder to catch than one that lands on a blank line.

### The false sentence the brief predicted

The spec's "What I could not check" says every `ootest` citation in the enumeration is inherited and
unverified. **There are none.** Measured: `sed -n '114,172p'` of the spec piped to
`/bin/grep -ac ootest` is `0`; `testGroup` occurs twice in the whole document and neither occurrence
is in an enumeration row. The one substantive `ootest` claim (`ANNOTATE.testGroup`, spec `:1096`) is
outside the enumeration, and I verified it. Recorded in the ledger, per the plan's route.

**Everything else resolved.** Full list in the ledger, including the ranges I checked and found
*correct* — `classDirective :334-490` and `methodDirective :629-812` both bound their directive's
option switch tightly, and the enumeration's option lists match the parser's arms exactly (`::CLASS`
seven, `::METHOD` twelve, `::ATTRIBUTE` thirteen). I nearly filed `methodDirective` as a repeat of
the `ClassClass.hpp:176-186` defect and it is not one.

---

## The unexercisable-class list, re-derived

Criterion: a class the phase cannot construct because its `init` chain reaches a native entry point
D37 registers and does not implement. Derivation: every `EXTERNAL 'LIBRARY REXX …'` directive in the
three `.orx` files, attributed to its owning `::CLASS`, then each owner's `init` traced.

| class | reason |
|---|---|
| **`Ticker`** | `init` `:1630` calls `self~!createTimer` at `:1659`, **before** its `guard off` `:1662` and `reply` `:1663`. `~new` cannot succeed |
| **`Alarm`** | `init` `:1472` calls `self~!startTimer` at `:1557`, **after** its `reply` at `:1556`. Under D55 the sender gets its object and the native raise lands on the continuation — exit status 0 with a traceback on stderr. **Not the same case as Ticker's**, though the spec treats them identically |
| **`Stream`** | `init` `StreamClasses.orx:154` calls `self~!c_stream_init` at `:165`, unconditional, no `REPLY`. **Not on the spec's list** |
| **`File`** | `init` `:516` reaches `qualifyImpl` at `:637` through `qualifiedPath` `:633`, on the single-argument path. **Not on the spec's list** |

`RexxQueue` looked as though it belonged and does not: its externals sit behind
`if name_queue == .nil` and `if named_queue \= "SESSION"`, and the argument defaults to `"SESSION"`.
Measured on the oracle, bare `.RexxQueue~new` is rc 0 and constructs; bare `.Stream~new` and
`.File~new` are both rc 163 `93.901`, so both are among the spec's 24 rather than its 38.

The planning consequence: the spec's reason for keeping `Stream` and `File` out of the opt-in set is
representativeness. There is a harder second reason — through Phase 5 they cannot be constructed at
all, because the natives their `init`s reach are Phase 7's.

---

## The `DEFERRALS` mapping

`rust/crates/rexx-classes/src/native_classes.rs:171`. **It has six rows. The spec says a reviewer
"read `DEFERRALS` and asked why four classes were deferred."** The two nobody asked about,
`RexxInteger` and `NumberString`, are exactly the two whose mechanism turns out to be unenumerated —
which is the spec's own argument for the instrument, landing again.

| deferral | missing mechanism | owner |
|---|---|---|
| `QueueClass` | `RemoveMethod` of nine names | enumeration `:126` — this deferral is why that row exists |
| `VariableReference` | `HideMethod` of six operators | `:126` |
| `StemClass` | the same six, plus `~inherit`s `.MapCollection` | `:126`; `:123` and the bootstrap for the second half |
| `RexxInfo` | `addToSystem` not `addToEnvironment` | no enumeration row, but owned by the class-set criterion |
| `RexxInteger` | not `.NAME`-reachable, **and** `CLASS_CREATE_SPECIAL(Integer, "String", …)` | **none** |
| `NumberString` | the same pair | **none** |

The deferral reasons themselves resolve: `CLASS_CREATE_SPECIAL` is `memory/RexxMemory.hpp:541`, used
at `classes/IntegerClass.cpp:2066` and `classes/NumberStringClass.cpp:74`.

---

## Mechanisms the reading found that the enumeration does not carry

Eleven, all in the ledger. The four worth reading first:

1. **A native class reporting a different class than the one that created it.** `12345~class~id` is
   `String` on the oracle, not `Integer`; `(1.5)~class~id` likewise. The enumeration's `:148` puts
   `~class`/`~id` in 5a and says only that the protocol answers — an implementation answering
   `Integer` satisfies every word of it and diverges. Crate is rc 120 today.
2. **`defineClassMethod`.** `fundclasses.xml:96`-`:97` comments it out beside
   `mthClassInheritInstanceMethods` (`:107`-`:108`) with the same reason, "setup.cpp: these two are
   special and will be removed at the end of the image build" — D39's two setup methods, named by the
   documentation. The enumeration has a row for one of them and none for the other, though
   `CoreClasses.orx:73` uses it and `2026-08-17-phase-5a.md:1751` relies on it. String count in the
   spec: 0.
3. **The comment-stripping rule is stated for one file and needed in every file the method-row
   denominator names.** Measured, comparing raw counts against `perl -0777 -pe 's/<!--.*?-->//gs'`:
   member `<xref>`s go 255→249, 354→345, 391→374, 54→45 in the four books and 25→24, 34→33, 9→8 in
   `directoryclassmethods.xml`, `queueclassmethods.xml` and `supplierclassmethods.xml` — the last
   three being include files whose `<member>`s the denominator names explicitly. And one `mth*`
   *section* is commented out: `utilityclasses.xml:10062`'s `mthSupplierInit`. The four books' raw
   section counts sum to 1013, which is the number the spec's entity-prefix bullet uses, so **the
   spec's 1013 includes a withdrawn section and the live figure is 1012.**
4. **`~copy` on a class object.** `fundclasses.xml`'s `clsClass` says it is forbidden for Class and
   all class objects; measured, `say .Object~copy` is oracle rc 163 with `93.970` and the
   `*-* Compiled method "COPY" with scope "Class".` frame. The enumeration puts `~copy` in **5b**
   because "it needs an instance to copy" — the class-object arm needs none. Under D46 that is a
   split that is not named as split.

The rest: `.ENDOFLINE` is a Phase-5-claimed environment entry with no row anywhere (crate rc 120
naming Phase 5; `.LINE` and `.RS` were checked and agree, so they are not a gap); two enumeration
authority cells that say "nothing documents it" are wrong (`collclasses.xml:8064`-`:8067` documents
Stem's six hidden operators **and names DEFINE as the mechanism**; `clsDirectory` documents the
Directory entry methods with a worked example); `clsSetCollection` and `clsInputOutputStream` have
literally no methods and need naming as exceptions; `PRIVATE` has three documented calling contexts;
`PackageClass::findClass` makes **two** security-manager calls where criterion 5 asks for exactly one
at directory lookup; three more classes carry a documented no-user-construction sentence with a
Rexx-level route (`RexxContext`, `RexxInfo`, `StackFrame`); and `hashCode` is `ArgUtil`'s shape one
level down — deliberately undocumented, and `.Object~hasMethod("HASHCODE")` is 1.

---

## What the `ootest` mapping is worth, beyond the rows

The rule the plan cares about is the three-signal one, and its second signal was uncheckable before
this. Four results change how a later task should read it:

* **`UNKNOWN` is the row the spec exists for and `ootest` does not pin it.** `::method unknown`
  occurs in three groups tree-wide; the only object-model one is `Object.testGroup:1487`/`:1493`, and
  reading them, both are UNINIT-tracking **scaffolding** — `UNKNOWN` is the instrument, not the
  subject.
* **The `makeString` limb of Required String Values is unpinned too.** `::method makeString` appears
  in no `base/` group at all.
* **Superclass-versus-mixin merge order is unpinned, and upstream has the same defective probe the
  spec found in its own predecessor.** `Class.testGroup:1276` sets up
  `SUBCLASS RoadVehicle INHERIT WaterVehicle`, asserts only `~isSubclassOf`, and the subclass defines
  its own `show_off` — the exact variant that is green under a backwards merge.
* **Native removal and hiding is pinned as a *mechanism* and not as an *image-build application*.**
  `Class.testGroup`'s one-argument `~define` tests are `hideMethod`'s Rexx-level twin, including its
  97.1 observable at `:1002`-`:1004`. But no `assertFalse(…hasMethod…)` anywhere in the tree names a
  natively removed or hidden name, and `Stem.testGroup` has no `hasMethod` call at all. The nearest
  thing is `Queue.testGroup:452`'s `test_stableSort`, which exercises the **composition** of removal
  and re-donation — the only observable form, as the spec says.

Two rows I drafted as "none" and had to correct after checking, which is the recorded hazard behaving
exactly as recorded:

* **`:121`, leftmost-first among several `INHERIT`s, IS pinned.** `CLASS.testGroup`'s `test_inherit`
  asserts `~superClasses` as an *ordered* list and the assertion moves with the directive's mixin
  order — `(.Object, mixin, .Comparable, .Orderable)` at `:401` against
  `(.Object, .Comparable, mixin, .Orderable)` at `:432`. That pair discriminates.
* **`:125`, the cascade, IS pinned** — `test_class_define` with `::class testDefine2 subclass
  testDefine1`, at `:984`-`:986`.

Both were "none" until I opened the file. I have kept the "none" rows that survived that treatment
and said in each what search produced them.

---

## What I did not read

* `provide.xml` and `dire.xml` — not re-read; their rows are the spec author's, carried with their
  stamp.
* Every `mth*` section — out of scope by the stopping point. The measurements above are about their
  markup, not their content.
* `rexxpg`'s other chapters.
* `ootest` test bodies, except the tests a ledger row cites. The mapping opened every group it names.
* The C++ beyond the cited lines. `findClass` (`:1081`-`:1163`) and `processInstall`
  (`:1227`-`:1302`) were read whole because a citation into each needed it.
* `ClassDirective::activate()`, `RexxInstructionForward`, `RexxInstructionReply::execute` and
  `traceMessage` are cited **without** a line, so there was nothing to resolve. I located the first
  (`instructions/ClassDirective.cpp:284`) and not the other three.
* `platform/windows/PlatformObjects.orx` — opened, two lines, `call 'orexxole.cls'`; its consequences
  for the other CI platforms not pursued.
* **No oracle defect is declared.** The only place the three-signal rule's second signal is applied
  is `~copy`, and it comes out *against* a defect finding — `Class.testGroup:1007`'s
  `test_class_copy` pins `93.970`, so it is behaviour to reproduce.

---

## What I ran

Oracle runs used the wrapper, three descriptors read separately, never `2>&1`, from a fresh `mktemp`
directory with absolute paths. Nothing from `rust/corpus/oracle-crashes.txt` was run. Probes:
`.Object~copy`; `~hasMethod` for `HASHCODE`, `IDENTITYHASH`, `DEFINECLASSMETHOD`,
`INHERITINSTANCEMETHODS`, `COPY`; `12345~class~id` and `(1.5)~class~id`; `.line`/`.rs`/`.endofline`
on both engines; the `.environment~supplier` non-class sweep; bare `~new` on `Stream`, `File` and
`RexxQueue`.

**The five gate commands, from `rust/`, each with its own exit status:**

```
cargo fmt --all --check                                          EXIT=0
cargo clippy --workspace --all-targets -- -D warnings            EXIT=0
cargo test --release --workspace                                 EXIT=0
REXX_CORPUS_GATE=1 cargo test --release --workspace              EXIT=0   106 of 106 matching
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast EXIT=0 106 of 106 matching
```

**No sitting.** This task touches only `docs/`, so the release binary the axes measure is
byte-identical and a sitting would measure noise.

**`REXX_PHASE_GATE=5a … cargo test` was not run, and the reason is that it cannot see its subject.**
Measured: `/bin/grep -arn 'REXX_PHASE_GATE\|CLOSED_PHASES' crates/ --include=*.rs` exits **1** — the
variable and the constant do not exist in the tree yet; they arrive with the gate tables in Tasks 4
and 5 and with Task 24. Setting the variable would run a command that exits 0 for a reason unrelated
to what it is supposed to check, which is the shape this project keeps finding in its own
instruments. The count of 5a rows not yet `agree` is therefore not reportable at this task; Task 4 is
the first that can establish it.

## Commit

`237b53f39` — one file, `docs/superpowers/plans/phase-5-reading-ledger.md`, 480 lines added.

---

# Fix round 1

**Status:** all nine closed. Commit **`fa244be1a`** on top of `237b53f39`, one file, 155 insertions /
55 deletions. Gates green, corpus 106 of 106, `REXX_PHASE_GATE=5a` not run.

## The ruling, applied

Every count in the `ootest` mapping is now `/bin/grep -aciE '<pattern>' ootest/ooRexx/<path>` at
r13178 **with the pattern written beside it** wherever it is not the literal token shown. Every
negative is now written as "no line matching `<pattern>` under `<paths>` at r13178" and gives the
patterns, never as "nothing upstream pins this". The mapping's preamble states both rules and says
which rows of the first draft violated them.

## Finding by finding

**1 — HIGH, closed.** Row `:120` rewritten around
`Class.testGroup::test_INIT_INHERIT_UNINHERIT_SUBCLASS_MIXINCLASS_QUERYMIXINCLASS` (`:295`), with the
setup lines (`:299`, `:312`, `:323`, `:326`, `:330`, `:331`) and the discriminating triple
(`:386` / `:402` after `~uninherit` at `:395` / `:413` after re-`inherit` at `:406`). **I reproduced
it independently** rather than take the reviewer's measurement: rc 0, empty stderr,
`SwimCar: I swim now...` / `RGF_VEHICLE_SWIM` / `SwimCar: I swim now...`. The row now says it is a
discriminator a later task can lift into a corpus program, and explains why it and the spec's own
program are complementary rather than contradictory — the mixin wins over a method defined *at* its
base class and loses to one defined *below* it, which is `fundclasses.xml`'s `inherit` rule from both
sides. Both riders fixed: `test_BASECLASS:177` added, and the `:1245` call recorded as inside
`/* … */` at `:1241`-`:1246`. One line number of my own corrected on the way:
`rgf_vehicle~define("SHOW_OFF", …)` is `:306`, not `:311`.

**2 — HIGH, closed.** Row `:136` rewritten. `REQUIRES.testGroup:320` `test_activate` — the requires
file at `:324`, `::class test2 public` declared after `test`, assertion at `:331` — pins pass 3 after
pass 1. `CONSTANT.testGroup:496` `test_expression_activate` — the `::resource activate` at
`:486`-`:494`, `::method activate class` at `:489` reading `::constant d (2 * 2)` at `:493`,
assertion at `:499` — pins pass 2 before pass 3, with `:480`-`:482` pinning real constants before
expression ones. Both opened and read. The row now closes the loop with the ledger's own
`processInstall` correction by naming the three loops at `:1276`, `:1285`, `:1294`.

**3 — MEDIUM, closed.** Row `:126`'s citations corrected on all three coordinates: `:198` in
`::METHOD "test_DEFINE"` (`:188`) and `:227` in `::METHOD "test_DELETE"` (`:218`), with `:226`'s
comment quoted and the observables at `:205` and `:236`-`:237` named as observables. The negative half
now carries its command and its result.

**4 — MEDIUM, closed.** Every count in the table re-run. The four that would not reproduce were taken
under `::method[^;]*\bX\b` — stated now, and they come out 12, 12, 6, 9 again. `~defineMethods` is
stated as `~defineMethods?\(` and `reply` as `\breply\b`. Every other count re-run and unchanged.

**5 — LOW–MEDIUM, closed by doing both.** The "checked and correct" list is split into **(a)**
citations resolving to a definition line and **(b)** point-of-interest citations resolving inside the
named function, with (b) a table naming each citation, its enclosing function *and that function's
own line*, and what the cited line actually is. The three the review named are there; so are
`ClassDirective.cpp:243`/`:273`, `LanguageParser.cpp:3292` and `ExpressionClassResolver.cpp:135`,
which share the shape. I resolved every enclosing function rather than asserting it:
`RexxClass::subclass` `:1562`, `RexxClass::inherit` `:1287`-`:1369`, `ClassDirective::install` `:165`,
`ClassResolver::evaluate` `:124`, `LanguageParser::parseQualifiedSymbol` `:3261`. **A conservation
check confirms every citation in the original list appears in exactly one of the two** — the split
initially dropped `ClassClass.cpp:1148` and the check caught it.

**6 — LOW, closed.** `guard off` `:1554`, `reply` **`:1555`**, blank `:1556`, `self~!startTimer`
`:1557` — two lines before, with a blank between.

**7 — LOW, closed.** `clsRexxContext`'s two documented routes are the `.CONTEXT` environment symbol
and **the `StackFrame` class's `context` method** (`utilityclasses.xml:7540`-`:7545`). The
`~package`-side route was mine and is withdrawn in place.

**8 — LOW, closed.** The derivation now states the `awk` it was actually run with — case-insensitive
and accepting either quote — and names `StreamClasses.orx:510`, `:546`, `:547` as the three
double-quoted directives the single-quoted pattern misses, all `File`'s, with the conclusion
unchanged.

**9 — LOW, closed by labelling.** A new paragraph separates the `Alarm` row's two halves: that
`reply` precedes `!startTimer` is read off `:1555`/`:1557`; that the transcript is "exit status 0
with a traceback on stderr after the main program has finished" is **inherited from D55's wording**,
measured there on two different programs, and *when* a `REPLY` continuation runs in a single-threaded
Phase 5 is a design choice no task has made. The `Alarm` row's author is told to measure it.

## Two things beyond the nine

**Re-checking every remaining negative under the new rule found two more wrong rows**, both flagged by
the review as the ones to open next, and both the same defect as findings 1 and 2:

* **`:141`** said SELF and SUPER are "pinned by nothing that is about them". `USELOCAL.testGroup:119`-
  `:120` assigns `result = 1; rc = 2; self = 3; super = 4; sigl = 5` inside a `use local` method and
  asserts at `:131` that the answer is `"RESULT RC SELF SUPER SIGL"` — the five special variables are
  protected and keep their values. `API/oo/FUNCTION.testGroup:1053` `TestGetAllVariables1` asserts
  `assertSame(self, d['SELF'])` and `assertSame(super, d['SUPER'])` at `:1055`-`:1056`.
* **`:153`** said no group walks the order. `Package.testGroup:782` `test_package_local` sets
  `.local~packageTest = "ABC"` (`:783`), builds a package setting `.context~package~local~packageTest`
  and returning the environment symbol `.packageTest` (`:784`), and asserts `"DEF"` at `:786` — the
  package-local directory beating `.local`, `searchord`'s step 5 over step 6, which is exactly D33's
  amendment. **Reproduced on the oracle**: rc 0, empty stderr, stdout `DEF` / `DEF`.

So four of the mapping's `none` rows were wrong and all four came from the same defect. The rows that
still find nothing — `:124`, `:126`'s image-build half, `:137`'s `INIT`-before-`INHERIT` half, `:139`,
`:146`'s `makeString` limb, `:170` — each now names the patterns and paths, and I widened each before
keeping it: for `:124` from the name to the donated-method-set effect, for `:137` from a name search
to an intersection over files carrying both `init class` and `activate`. Both still empty.

**The reviewer's "not a defect" observation is now item 6a in the ledger**, verified here rather than
taken on report: `RemoveClassMethod` and `HideClassMethod` (`Setup.cpp:360`-`:361`) have **no call
site anywhere in `interpreter/`** — `/bin/grep -arnE '\b(RemoveClassMethod|HideClassMethod)\('` over
`interpreter/` returns only the two `#define` lines, and the non-preprocessor count in `Setup.cpp` is
0, against `RemoveMethod(` 9 and `HideMethod(` 12 running `:792` to `:1404`. The spec's `:126` names
"their class-side twins"; the image build has no user for them. The section heading is corrected to
say so: items 1–11 came from the reading, 6a came from the reviewer.

## What I ran

Oracle probes for finding 1 and for `:153`, both from fresh `mktemp` directories, three descriptors
read separately, absolute paths, nothing from `oracle-crashes.txt`. Every count in the mapping re-run.
Every citation changed resolved at its new location before being written.

```
cargo fmt --all --check                                             EXIT=0
cargo clippy --workspace --all-targets -- -D warnings               EXIT=0
cargo test --release --workspace                                    EXIT=0
REXX_CORPUS_GATE=1 cargo test --release --workspace                 EXIT=0   106 of 106 matching
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast  EXIT=0   106 of 106 matching
```

No `FAILED` result line in any of the three test logs. No sitting: `docs/` only.
`REXX_PHASE_GATE=5a` not run, per the standing answer.

## What I could not close

Nothing from the nine. Two limits stand and are stated in the ledger rather than closed:

* **The `Alarm` transcript prediction** is not runnable in Phase 5 today, by construction. Labelled.
* **The ledger's own denominator.** Finding 1's shape — a search narrower than its sentence — is now
  guarded by stating every search, but stating a search does not make it wide enough. Four rows were
  corrected by widening; a fifth that nobody thinks to widen would look exactly like the rows that
  survived.

---

# Fix round 2

**Status:** all seven closed, plus the observation. Commit **`1b536079a`** on top of `fa244be1a`, one
file, 115 insertions / 31 deletions. Gates green, corpus 106 of 106, `REXX_PHASE_GATE=5a` not run.

## The ruling, applied

*A procedure is not written down until it has been run verbatim, from the raw file, and its output
seen.* Three of the seven defects were procedures that had never been executed as written. Every
command now in the ledger was run in that form before it was written down, and the pattern block's
round trip — extract from this file's raw bytes, run, compare to the stated count — is recorded in
the ledger itself so a later reader can repeat the test rather than trust it.

## Finding by finding

**N1 — HIGH, closed, and it changed two rows.** `ootest/ooRexx/base/class/class.testgroup.cls` is 70
lines written for nothing but the install passes, loaded by `Class.testGroup:962` `test_activate`
(`:964` results directory, `:966` `loadPackage`, `:968` failure assertion, `:970`-`:972` the three
activate flags). I read it in full. It declares `::class class1 subclass class3` (`:12`),
`::class class2` (`:33`) and `::class class3` (`:49`), **each with both an `::method init class` and
an `::method activate class`**, and asserts four things with their own failure messages: every class
object exists inside every activate (`:25`-`:27`, `:43`-`:45`, `:59`-`:61`); **activate order follows
the dependency graph**, class2 → class3 → class1 (`:20`-`:23`, `:38`-`:41`, `:54`-`:57`), with the
file's own comments explaining why (`:11`, `:32`, `:47`-`:48`); an instance is created and sent to
inside activate (`:16`-`:18`); and the prolog runs after every activate (`:5`-`:9`). Row `:136` now
leads with it and demotes `REQUIRES.testGroup:320` and `CONSTANT.testGroup:496` to the incidental
pins they are; row `:137` is rewritten around it. **Ordering was named nowhere in the ledger and now
is.**

**The extension set, which is how that was missed.** Measured with
`find ootest -type f -not -path '*/.svn/*' | sed 's/.*\.//' | sort | uniq -c | sort -rn`: `.testGroup`
409, `.rex` 49, `.cls` 9, `.testUnit` 7, `.oodTestGroup` 2. The preamble now states the five-include
line once and says rows substitute it for `<extensions>`. **Every negative in the table re-run over
all five**: `:124` (both patterns), `:126`'s name filter, `:137`'s phrasing search, `:139`, `:146`,
`:170` — all unchanged; `:137`'s intersection is not — it returns exactly the `.cls`.

**N2 — MEDIUM, closed.** The intersection's first `grep` had no `-E`, so under BRE `[[:space:]]+`
wanted a literal `+` and it matched zero files: the `none` came out of an empty first term, never
reaching `activate`. With `-E` and the five extensions it is **27 files**, of which **one** also
carries an `activate` directive — the `.cls` N1 is about. The row records both the correction and
what the old form did.

**N3 — MEDIUM, closed.** `NR` → `FNR`, and the third path corrected to
`interpreter/platform/unix/PlatformObjects.orx`. **Run verbatim**, and its output is what the table
says: `CoreClasses.orx:1590`/`:1618` under `Alarm`, `:1690`-`:1692` under `Ticker`,
`StreamClasses.orx:510`/`:546`/`:547` under `File`. The old form printed those three as `:4703`,
`:4739`, `:4740` in a 1010-line file, because `CoreClasses.orx` is 4193 lines and is read first —
`file:line` strings shaped exactly like citations, naming lines that do not exist. Both facts are in
the ledger.

**N4 — MEDIUM, closed.** `:167` is `test_issubclassof_non_class`; the test that reaches the
directive-declared hierarchy is `::method test_issubclassof` at **`:146`**, whose four
`AmphibianVehicle` assertions I resolved individually at **`:154`-`:157`**. The decoy at `:1248` sits
under `::CLASS "WasserFahrzeug" SUBCLASS Fahrzeug` (`:1229`) with its own copy of the same four
assertions at `:1255`-`:1258`. (The review cites `:153`-`:156` for the first set;
`/bin/grep -an 'Amphibianvehicle~issubclassof'` gives `:154`-`:157`, which is what the ledger now
says.)

**N5 — LOW, closed.** `:153`'s search returns **three** files, not two:
`base/directives/REQUIRES.testGroup`, `base/keyword/CALL.testGroup` and
`base/rexxutil/Macrospace.testGroup` (`:154`, `:160`, and a `#define` comment at `:83`). Three
different subjects, none of them this row's; the row now enumerates all three.

**N6 — LOW, closed.** List (a)'s criterion no longer claims its members are all definition lines. It
says what it is: each resolves to what the spec's cell names it as — a definition for a cell naming a
function or an enum, and a macro invocation or image-build statement for the `Setup.cpp` members, and
the two `DirectiveParser.cpp` entries are range ends covered by the paragraph after (b).

**N7 — LOW, closed by relocation rather than explanation.** Every `|`-bearing pattern is out of the
table and into a fenced block keyed by row, where no escaping is needed. **Then extracted from the
committed file's raw bytes and run**: `:143` → 32 on `SecurityManager.testGroup`, `:152` → 12 on
`collections/directory.testGroup`, `:154` → 52 on `Package.testGroup`, `:169` → 60 on
`GUARD.testGroup`, `:126` → its four files with `:126-names` over their hits exiting 1, `:124` and
`:137` → no file. A structural check confirms no table row carries an unescaped extra pipe and none
carries an escaped one either.

**The observation — closed as a hole in the stopping point.** The C++ row's stopping point is "every
`file:line` the spec cites, verified", so the denominator is the **spec's** citations; round 1's
conservation check ran against this ledger's own earlier list, which by construction could not see a
citation that list never had. Re-run against the spec — extracting every `File.(cpp|hpp):N` plus
every bare `name :N` and matching against the ledger — exactly two came back unplaced, and both are
now a new list (c):

* **`Setup.cpp:325-329`** — the spec cites it *as an example of a wrong citation* and corrects it to
  `:331`/`:332` in the same sentence. Verified: `:325`-`:329` is the `TheCommonRetrievers` preamble
  and stops one line before the `SELF` put. The spec's correction is right; nothing further owed.
* **`Setup.cpp:795`-`:797`** — **imprecise, and short at the end that matters.** `:795` is blank; the
  comment runs `:796`-`:798`, and the cited range **stops before `:798`**, the `makeString` line —
  which is exactly the contrast the spec's own sentence at `:806` draws. Corrected to
  **`:796`-`:798`**.

Re-run after the edit, both the explicit and the bare citation sets come back **unplaced: none**.

## What I ran

Every command written into the ledger was executed in the form written. The `awk`, the five-extension
searches, the intersection, and every block pattern extracted from the raw file. Nothing was retyped
from memory into the document.

```
cargo fmt --all --check                                             EXIT=0
cargo clippy --workspace --all-targets -- -D warnings               EXIT=0
cargo test --release --workspace                                    EXIT=0
REXX_CORPUS_GATE=1 cargo test --release --workspace                 EXIT=0   106 of 106 matching
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast  EXIT=0   106 of 106 matching
```

No `FAILED` result line in any of the three test logs. No sitting: `docs/` only.
`REXX_PHASE_GATE=5a` not run, per the standing answer.

## What I could not close

Nothing from the seven. One thing worth handing forward rather than closing:

**The axis of the too-narrow search keeps moving, and stating the search does not fix that.** Five
rows of the mapping have now said `none` and been wrong — on which copy of a hierarchy was searched,
on which word a test uses, on whether a test is *about* a thing or merely asserts it, on which step of
an order it pins, and now on which file extension it lives in. **The fifth was written in the round
that was correcting the first two.** What the stated-procedure rule buys is that a later reader can
widen a search they suspect; it does not make any given search wide enough. That is now said in the
ledger, on the axis-by-axis evidence, rather than left as a general caution.

**Two further limits stand**, unchanged: the `Alarm` transcript is a prediction not runnable in Phase
5 by construction, and the ledger's own denominator is a hand-made list of authorities — N1 is what
that limit looks like when it bites, an authority nobody had opened sitting next to one everybody had.

---

# Fix round 3

**Status:** all four closed, plus a fifth defect the round's own fix uncovered. Commit
**`84b62a596`** on top of `1b536079a`, one file, 247 insertions / 30 deletions. Gates green, corpus
106 of 106, `REXX_PHASE_GATE=5a` not run.

**Commands re-run: 101. Reproduced: 101.** Plus the citation extractor's **two negative controls**,
both of which fire. Detail below.

*(An earlier commit of this round, `400d4ad40`, was amended into `84b62a596` before being reported:
its message claimed a round trip that the round trip then falsified. Amending rather than following
up kept a false claim out of an uneditable message.)*

## The structural ruling, applied

*Replace every completeness assertion with the pasted output of the check that supports it.* The
sentence "every C++ `file:line` the spec cites appears in (a), (b), (c) or the found-wrong table" is
gone. In its place the ledger carries the extractor, its invocation, and its **whole 90-row output**,
under a heading that says why: a completeness sentence cannot be checked at a glance and is what
stops the next reader looking. The C++ authority row no longer reads `done`; it reads **done against
a stated extractor**, with that extractor's reach written down and the note that a fourth citation
form would be invisible to it exactly as the third was.

## Finding by finding

**D1 — closed, three parts.**

1. **Placed.** `ClassClass.cpp:988`-`:990`, resolved at the source: the three-line comment inside
   `RexxClass::method` (`:984`) beginning `// we keep the instance methods defined at this level in a
   separate`, which is the sentence the spec quotes. It is a point-of-interest citation and is now a
   list (b) row. The citation was always correct; only its placement was missing.
2. **The extractor reaches the third form.** A bare `:N` continuing an earlier citation in the same
   parenthesis — neither `File.cpp:N` nor `name :N`. **Four further rules turned out to be
   load-bearing and every one of them was wrong at some point while I drafted this**, each caught by
   running the thing rather than reading it, and each is now stated in the script with its reason:
   the carried filename **resets at every line** (without it a `07:06` timestamp is reported as
   `Setup.cpp:06`); it **resets on any filename**, not only a C++ one (without it a bare `:453` after
   an `.orx` name is attributed to the last `.cpp` seen); the ledger's **fenced blocks are blanked**
   before the markers are searched (without it the script's own quoted markers truncate the span it
   is measuring — measured: `### In-tree Rust` resolved inside the script's `MARKS` line and list (c)
   lost 130 lines); and **only the lists' table rows count**, not the prose beside them.
3. **The row's status.** Marked against what the check reaches, as above.

**The fifth defect, which part 2's last rule found: `ClassClass.hpp:176-186` was placed by no entry.**
The spec cites it at `:1360` as its own worked example of a wrong range. It had been *discussed* in
the prose below the tables and never entered anywhere, so the loose check counted it placed. It is
now a (c) row, verified at the header: `:176` is `static RexxClass *classInstance;`, the `ClassFlag`
enum runs `:180`-`:189`, and the cited range does stop at `:186` before `PARENT_HAS_UNINIT` (`:187`)
and `ABSTRACT` (`:188`) — exactly what the spec says. Tightening the check also **re-attributed
three citations correctly**: `inherit() :1322`, `RexxClass::subclass :1631` and
`PackageClass::findClass :1086` had been matching the found-wrong section's *intro prose* that names
them, and now resolve to list (b), where they actually are.

**D2 — closed by removing the filter, not widening it.** The `find` census the ledger already ran
prints, beside the five extensions it named, **`CLS` 1** — `ootest/framework/OOREXXUNIT.CLS`, the
ooTest framework itself, which `--include='*.cls'` does not match because the glob is case-sensitive
(measured: that include exits 1 over `ootest/framework/`; `--include='*.[cC][lL][sS]'` returns the
file) — plus `norex`, `other`, `test1` ×2, `test2`, and four extensionless files, several of them the
fixtures for the search-order tests row `:153` is about. **A filter narrow enough to write down is
narrow enough to be wrong**, so there is none: every search runs over the whole checkout with
`--exclude-dir=.svn`, and every negative was re-run in that form. Nothing changed except `:137`'s
intersection, 27 files to **29**, with the same single member.

**D3 — closed.** Row `:137`'s literal ellipsis is gone, together with the `<extensions>` placeholder
it stood beside, since the filter it substituted for no longer exists. **The measurement first
written here does not support that conclusion, and is corrected in fix round 4 below**: run as
written against the object this round committed, "no line containing an ellipsis also contains
`grep`, `awk`, `find`, `python3` or `perl`" returns a hit — the authority row at `:33`, whose `find`
is inside the word *findings* — and its five-word list omits `sed`, `svn`, `cargo` and `python`,
every one of which appears in a command in that document. The conclusion holds; the stated check is
narrower than the sentence it licenses even where it runs.

**D4 — closed.** Row `:137` now states the disqualifying fact rather than leaving the reader to open
the file: `class.testgroup.cls`'s three `::method init class` bodies are each a bare `nop` (`:14`,
`:35`, `:51`), so the one file that declares both directives asserts nothing about `init`.

**Also fixed, unprompted:** the comment-stripping table gave its counts with the *pattern described*
rather than stated. Both commands are now written out, and the section-keyed half gives its stripped
figures (250 / 346 / **371** / 45) instead of only its raw ones.

## What I ran — 101 commands, 101 reproduced

Every one extracted from the file's raw bytes, never retyped.

| group | n | result |
|---|---|---|
| mapping counts, `/bin/grep -aciE '<pattern>' <file>` | 49 | all match the stated figure |
| block-pattern counts (`:143` 32, `:152` 12, `:154` 52, `:169` 60, `\breply\b` 33, Stem `hasMethod` 0) | 6 | all match |
| tree-wide searches and negatives | 11 | all match; `:124`, `:137`'s phrasing and `:126-names` exit 1 |
| the extension census | 1 | reproduces, and names what a five-extension filter misses |
| the `awk`, run as a script from the file | 1 | `:510 :546 :547` under `File`, `:1590 :1618` under `Alarm`, `:1690`-`:1692` under `Ticker` |
| comment-stripping pairs, 7 member files + 4 section files | 22 | all match |
| class-side twins (item 6a) | 4 | 0 call sites; `RemoveMethod(` 9, `HideMethod(` 12 |
| the spec's enumeration-`ootest` check | 1 | 0, exit 1 |
| the citation extractor | 1 | 90 rows, 0 unplaced |
| the five gate commands | 5 | all EXIT=0, corpus 106 of 106 |

**And the extractor's two negative controls, because a check that cannot fail is decoration:**

* **reword a list heading** → raises on the marker count, naming the missing marker, instead of
  silently merging two lists. **Fires.**
* **delete one table row** (`ClassClass.cpp:988`-`:990`) → its two tokens return `UNPLACED`.
  **Fires** — but only after the search was narrowed to entries. With the whole section in scope it
  did *not* fire, because the paragraph discussing a citation placed it. That failure is what
  produced the fifth defect above.

**Verified against the committed object, not the working tree:** `git show HEAD:…` piped into the
extraction gives a script that, run against the committed ledger, prints 90 rows with 0 `UNPLACED`,
**byte-identical to the block committed inside it**.

```
cargo fmt --all --check                                             EXIT=0
cargo clippy --workspace --all-targets -- -D warnings               EXIT=0
cargo test --release --workspace                                    EXIT=0
REXX_CORPUS_GATE=1 cargo test --release --workspace                 EXIT=0   106 of 106 matching
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast  EXIT=0   106 of 106 matching
```

No `FAILED` line in any test log. No sitting: `docs/` only. `REXX_PHASE_GATE=5a` not run.

## What I could not close

Nothing from the four. Two things handed forward:

**The extractor's reach is written down and is still finite.** It knows three citation forms and
places by line number within a list's entries. A fourth form is invisible to it exactly as the third
was to its predecessors, and a token that happens to appear in an entry under an *unrelated* file
counts as placed. That is in the ledger beside the output, not implied by its absence.

**The pattern of this round is worth naming, because it is the third round in a row of it.** Round 1
put procedures beside claims; round 2 found three that had never been run; round 3 found that the
check behind a completeness claim was itself narrower than the claim, twice over — once on citation
form, once on prose-versus-entries. Each fix was correct and each left the next layer unexamined.
**What changed this round is that the artifact is now the output rather than the sentence**, so the
next such gap is visible in a diff to someone who runs nothing. That is a better failure mode, not an
absence of failure.

---

# Fix round 4

Base `84b62a596`, one file changed in `docs/` plus this report. All seven findings closed. Two
things beyond them are in the diff and are called out below, because both are the same defect as N1
one level down and both were found by running a procedure this document already states.

## Finding by finding

**N1 — closed, by fixing the extractor.** The carry now crosses a line break and is cleared at a
Markdown block boundary or by any filename that has no `:N` of its own. `ClassClass.hpp:180` and
`:189` — the wrapped continuation at spec `:1360`-`:1361` — appear in the output for the first time;
the other two wrapped tokens the re-review named (`:1285` at spec `:1350`, `:331`/`:332` at spec
`:1360`) are cited elsewhere in a reachable form and are deduplicated onto their earlier rows, which
is why the row count moves by two rather than five. The `07:06` → `Setup.cpp:06` false positive stays
suppressed: with the block-boundary reset removed it comes back, measured, as
`(via RexxString::compareToRexx) :06 spec:1401`.

**N2 — closed, by two rules rather than one.** `where()` now matches the explicit form by **file**
and not by line number alone, and the three tables are narrowed to **the row's own citation cell**
rather than the whole row. `Setup.cpp:1285` and `createInstanceBehaviour :1148` are now labelled
`(a)`, which is what this document's own list records. Against the previous round's script each rule
closes one of the two on its own; with both in place the cell narrowing subsumes the file guard, and
that is stated in the script and in the prose rather than left to be discovered — removing the file
guard changes no byte of the output today, measured.

The property the finding actually asks for is order-independence, so it is measured directly: over
**all 24 orderings** of the four lists this output is one string. The previous round's script gives
**eight** distinct outputs over the same 24, and nine of its rows move under a full reversal, against
the two the re-review found under the one permutation it tried.

**N3 — closed.** *"with the extension set above"* is gone. The block now says the nine are run over
`ootest/` at r13178 in the unfiltered form — whole checkout, `--exclude-dir=.svn`, no extension set —
and all nine were re-run in exactly that form, from the file's raw bytes: `:124` exit 1, `:126` its
four files, `:126-names` over their hits exit 1, `:137` exit 1, `:153` its three files, and
`:143` 32, `:152` 12, `:154` 52, `:169` 60.

**N4 — closed, and it is load-bearing now.** The rule is stated in the script and in the prose as
what it is, together with the fact that it was **inert against the previous round's script** and that
relaxing the per-line reset is what changed that. Verified after N1's fix, not before: narrowing the
extension set to C++ gains five rows — `CoreClasses.orx:66`, `:73`, `:92`, `:93` read as
`DirectiveParser.cpp`'s, and the spec's `corpus.rs:244` read through the `Class::method` carry.

**N5 — closed, report side.** The D3 paragraph above now says that its own measurement does not
support its conclusion, with the hit named. Run as written against `84b62a596`'s ledger, the check
returns the authority row at `:33`, whose `find` is inside *findings*, exit 0. What supports the
conclusion instead, measured on the object this round commits: **no line containing an ellipsis
falls inside a fenced block** — the fenced spans and the ellipsis lines are disjoint — and each of
the ten ellipsis lines was read. They are an `svn` error message quoted as text (`:38`), four
elisions inside quoted Rexx test bodies (`:637`, `:640`, `:642`, `:869`), four quotations from the
reference books (`:734`, `:755`, `:757`, `:1034`), and one quotation of this document's own phrase
(`:1023`). None is a procedure to re-run.

**N6 — closed.** `about \`init\`.** And the \`:137\` pattern` — the clause is a sentence again.

**N7 — closed, the same way the C++ row was.** The four-books row now reads **done against a stated
extractor**, and the extractor and its whole output are committed as
[The `cls*` section set, and its output](../../../docs/superpowers/plans/phase-5-reading-ledger.md#the-cls-section-set-and-its-output).
63 rows — 7, 17, 35 and 4 across the four books — each naming the span that was read: the opening tag
to the line before the first nested `<section>`. The script asserts that the narrow pattern and a
case-insensitive, attribute-order-tolerant one find the same sections, so the denominator is a
property of the books rather than of the pattern; **that assertion was run against a break** and
fires (`AssertionError: oodocs/rexxref/en-US/fundclasses.xml`, exit 1) when the pattern is narrowed
to `clsA`. Two of its rows are checkable against citations this document already makes by a different
route and agree: `clsBuffer` opens `utilityclasses.xml:421`, `clsPointer` opens `:6902`.

## Two things beyond the seven

**`Setup.cpp:348-351` was a wrong file on a committed row.** The spec writes
``MethodDictionary::hideMethod`` is ``put(TheNilObject, name)`` (`:348-351`) — a bare `:N`
continuing a **`Class::method`** name rather than a filename — so the previous round's carry
attributed it to the last `.cpp` seen. `Setup.cpp:348-351` is the `StartClassDefinition` macro's
comment; `MethodDictionary.cpp:348`-`:351` is `hideMethod` with exactly the `put(TheNilObject,
methodName)` the spec quotes, read at the source. The row was *placed* only because
`MethodDictionary.cpp`'s `:348` sits in list (a) — the concrete instance of the residue the document
had stated only in the abstract. The third form's antecedent now admits a `Class::method` name;
measured, that changes exactly one row, from the wrong file to the right one, and clearing the carry
there instead drops the token with no row printed, which is the silent form of the same defect.

**The case-sensitivity pair was a negative without its pattern.** *"that include exits 1 on
`ootest/framework/`"* holds for `#!/usr/bin/env rexx`, `OOREXXUNIT.CLS`'s own first line, and does
**not** hold for `OOREXXUNIT` or `ooRexxUnit`, under which the sibling `.cls` files match too and
both forms exit 0. The pattern is now beside the claim. Found by re-running the ledger's own
procedure with a pattern of my choosing and getting a different answer, which is the whole argument
of the section it sits in.

## What I ran — 127 commands, 125 reproduced as stated

Every pattern and script pulled from the file's raw bytes with `sed -n`, never retyped; run from the
repository root, gates from `rust/`. `oodocs/rexxref` and `oodocs/rexxpg` r13198, `ootest/` r13178,
`oodocs/` itself `E155007`, all four checked today.

| group | n | result |
|---|---|---|
| the C++ extractor, its invocation, `cmp` against the committed block, `UNPLACED` count | 3 | at `84b62a596`: 90 rows, byte-identical, no `UNPLACED` |
| its two negative controls | 2 | heading reword raises naming the marker at count 0, exit 1; row deletion returns `:988`/`:990` `UNPLACED` and nothing else moves |
| the nine `\|`-block patterns, unfiltered, `--exclude-dir=.svn` | 9 | as N3 above |
| the extension census and the extensionless enumeration | 2 | `testGroup` 409, `rex` 49, `cls` 9, `testUnit` 7, `oodTestGroup` 2, `CLS` 1, `norex` 1, `other` 1, `test1` 2, `test2` 1; `test_sysfile`, `test_sysfile_readonly`, `search_order`, `lineout` |
| the case-sensitivity pair | 2 | the case-folded half returns the file under every pattern tried; the `--include='*.cls'` **exits 1** half **does not reproduce as stated**, because the sentence names no pattern — the second item above |
| `svn info` on the three trees and on `oodocs` | 4 | r13198 / r13198 / r13178, `E155007` |
| the mapping table's counts | 53 | every one matches, including the four `::method[^;]*\bX\b` arms 12 / 12 / 6 / 9 |
| the row negatives that return file lists | 4 | `:124` exit 1; `:139` its three; `:146` its four; `:170` its one, 6 matching lines |
| `:137`'s intersection | 2 | 29 files, intersection exactly `base/class/class.testgroup.cls` |
| item 6a | 3 | tree-wide returns only the two `#define`s at `Setup.cpp:360`/`:361`; non-preprocessor count 0; `RemoveMethod(` 9 and `HideMethod(` 12, running `:792` to `:1404` |
| comment-stripping pairs, 7 member files + 4 section files | 22 | 255/249, 354/345, 391/374, 54/45, 25/24, 34/33, 9/8; sections 250/250, 346/346, 372/**371**, 45/45 |
| the spec's enumeration-`ootest` check | 1 | 0, exit 1 |
| the `awk` block, run as a script from the file | 1 | `CoreClasses.orx:1590`/`:1618` under `::CLASS 'Alarm'`, `:1690`-`:1692` under `::class Ticker`, `StreamClasses.orx`'s block under `Stream` / `RexxQueue` / `File`, nothing from `PlatformObjects.orx` |
| `:141`'s two citations | 2 | `USELOCAL.testGroup:119`-`:120` and `:131` as quoted; `base/special.variables/` holds only `RESULT_RC_SIGL.testGroup` |
| the `cls` section census at three pattern widths × four books | 12 | 7 / 17 / 35 / 4 at every width |
| the two `cls` opening lines and the two `fundclasses`/`utilityclasses` span boundaries | 4 | `clsBuffer` `:421`, `clsPointer` `:6902`; `fundclasses.xml:51` opens `clsClass` and `:134` is its first nested `<section>`; `utilityclasses.xml:459` is `clsBuffer`'s |
| the report's D3 measurement | 1 | **does not reproduce** — N5 |
| the five gate commands | 5 | below |

```
cargo fmt --all --check                                              EXIT=0
cargo clippy --workspace --all-targets -- -D warnings                EXIT=0
cargo test --release --workspace                                     EXIT=0
REXX_CORPUS_GATE=1 cargo test --release --workspace                  EXIT=0   106 of 106 matching
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast    EXIT=0   106 of 106 matching
```

All five **re-run on the committed tree at `5a466236b`** after the last prose edit, so the statuses
above are the commit's and not a nearby tree's. No `FAILED` line in any of the three test logs.
`REXX_PHASE_GATE=5a …` not run: the constant does not exist in the tree, so it would exit 0 for an
unrelated reason. No performance sitting: `docs/` and `.superpowers/` only, so the release binary the
axes measure is byte-identical.

`.superpowers/` is git-ignored, so this report is untracked, as it was for rounds 1 to 3 — the commit
is the ledger alone, which is what `84b62a596` and its two predecessors are too.

### My own probes, run independently of the document's

* **each of the seven rules in the new extractor broken in turn**, against the committed script and
  against the committed ledger. Every one changes the output except the file guard, and that one is
  labelled a guard in the script and in the prose because of this measurement, not in spite of it;
* **all 24 orderings of the four lists**, on both scripts — one distinct output against eight;
* both extractors **re-derived from the edited file's raw bytes** after every edit, and each output
  block replaced with what its own script prints, then re-derived once more: both are fixpoints;
* `MethodDictionary.cpp:344`-`:353` and `Setup.cpp:346`-`:353` read at the source, which is what
  settles the file column above;
* the D3 check run against both `84b62a596`'s ledger and this round's, and the fenced spans and
  ellipsis lines computed and intersected.

**One probe of mine was wrong and the document was right.** I first counted 6a's instance-side pair
as `RemoveMethod(` 10 and `HideMethod(` 12 + 1, against the row's 9 and 12, by dropping the
`^[^#]*` the row states — so my count included the two `#define` lines the row's own phrase "counted
the same way" excludes. Re-run as written it is 9 and 12. The failure mode is the one this document
is about, pointed the other way: a procedure re-run *not* as written reads as upstream drift.

## What I could not close

* **Whether the authorities listed are all there are.** Unchanged, and unchangeable from inside the
  document; its header says so.
* **Whether a fourth C++ citation form exists.** This round found that the *third form's antecedent*
  had an unenumerated shape, which is the same question one level down and says nothing reassuring
  about the level above it. The residue paragraph now says that.
* **Whether a `cls*` section could be written in a form the new extractor's pattern misses.** The
  cross-width assertion covers attribute order and case; it does not cover a section whose id does
  not begin `cls`, and nothing here can.
* **Whether the readings happened.** Not checkable by any instrument; only the derived facts are.
* **The rows this round did not touch.** Their counts were re-run and reproduce; their prose was
  approved in earlier rounds and is not reopened. No oracle run this round — nothing in the diff
  moves an oracle-derived claim.
