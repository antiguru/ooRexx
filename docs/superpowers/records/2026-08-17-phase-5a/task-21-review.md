# Task 21 review

**Spec compliance: CHANGES REQUIRED.** Three of the brief's four "Done when" bullets are met and I
verified each independently. The second -- "the four mutators work on a user class" -- is not: on a
user class, `~inherit` with a non-class position reports the wrong error, and `~defineMethods` handed
`.local` silently does nothing where the oracle raises.

**Task quality: CHANGES REQUIRED.** Two silent wrong answers (rc 0, wrong stdout) on surface this
task introduced, one of them predicted-and-denied by a doc comment the report itself cites as
authority. The rest of the work is strong: the replay is right name for name against a live oracle
measurement I took myself, the lock is byte-exact under all five frames on both engines, all
eighteen corpus programs agree on three descriptors on both engines, and all three controls really
ran.

**Findings: 2 critical, 1 major, 2 moderate, 6 minor.**

Everything below was measured from a fresh empty directory with absolute paths, oracle wrapper
exactly as dispatched, three descriptors read separately, both engines. I did not rebuild.

---

## What I verified that held

Recorded first so the findings are read against it, not instead of it.

* **All eighteen new corpus programs agree with the oracle byte for byte on stdout, stderr and exit
  status, on `REXX_ENGINE=ir` **and** `REXX_ENGINE=tree-walker`.** I ran each one myself with `cmp`
  on all three descriptors -- no indent normalisation, exact bytes. This matters because
  `corpus.rs:246` runs `Invocation::none()` only, so the corpus gate the controller ran covers the
  IR engine alone; the tree-walker half of "both engines, every construct" was unmeasured by the
  gates and is now measured.
* **The five lock frame lines are five distinct lines and each matches.** `DEFINE`, `DEFINEMETHODS`,
  `DELETE`, `INHERIT`, `UNINHERIT`, each `       *-* Compiled method "X" with scope "Class".`
* **The replay is right, checked against the live oracle rather than against the report.** I dumped
  `cls~methods(cls)` for all three new classes off the shipped interpreter and compared to the tables
  in `native_classes_wiring.rs`. `Queue` (32 names), `Stem` (19), `VariableReference`
  (`NAME REQUEST UNKNOWN VALUE VALUE=`) match exactly, including the absence of every removed and
  every hidden name and the presence of `APPEND` and `VALUE=`.
* **The wiring rows read as the report says.** Running gate table C's own eight questions
  (`gate_table_c.rs:576`-`:593`) for each class: `VariableReference` agrees on all eight;
  `Queue` and `Stem` agree on `entry`, `class-of-entry`, `id`, `class`, `superclass`, `metaclass`,
  `isa-class` and differ on `superclasses` alone. Exactly the brief's licensed exception.
* **`~define` preserves method identity and `~defineMethods` always copies -- and the crate matches
  the harder case the corpus does not pin.** Oracle and both engines agree on
  `m = .methods~z; .K~define("Z", m)` -> `1`, then `.K2~define("Z", m)` -> `0` (the object `m` had
  its scope set as a side effect of the first install, so the second one copies), and
  `.K3~define("Z", .K~method("Z"))` -> `0`. This is `MethodClass::newScope`
  (`classes/MethodClass.cpp:183`-`:199`) reproduced exactly, mutation of the caller's object
  included.
* **`~inherit`'s position semantics are right.** Read out of the C++ (`ClassClass.cpp:1353` ->
  `ArrayClass.hpp:247` -> `ArrayClass.cpp:797`, whose own doc comment says "the index is the position
  where this value will be inserted, not the index of where it is inserted after") and confirmed
  against the oracle. All three citations are correct.
* **The three controls really ran.** All three named files hash to exactly the sha256 the report
  records, and `git status` is clean -- so the restores are the committed bytes and not a
  reconstruction. Each mutation is one that could only redden for the reason claimed: control 1
  removes only the flag test, control 2 skips only `Queue`'s removals (leaving hiding untouched, so
  `class_native_hiding.rex` staying green is real separation), control 3 reroutes only
  `ClassGraph::hide` (so both the `Setup.cpp` replay and `~define`'s omitted-argument arm redden,
  which is why `class_mutators_user_class.rex` went with it).
* **Determinism.** Twelve runs each of the four output-bearing new corpus programs on both engines:
  one distinct hash each. `native_keys` sorts before it allocates; `public_classes_table` sorts
  before it allocates; `rexx_defined` is set in `CLASS_DEFINITIONS` order. `which_package`'s
  `HashMap` scan is a unique-match find, so its order cannot matter.
* **The pin.** sha256 of `bench-baselines/pinned/rexx-run-15a1ffa98` matches `PINNED.md`; the pin is
  an ancestor of HEAD; and D9's substitute check holds -- the commits touching crate source since the
  pin (103 now, 101 when the report measured) are all inside `d926c334d..HEAD`, set difference empty.
* **Commit hygiene.** Corpus programs, `phase-5a.txt`, `coverage.rs` and `collect_stress.rs` are one
  commit (36 added, 3 modified); all eighteen `sourceline_oracle` expectations are added, none
  modified. Comments are ASCII throughout, no em-dashes, no historical framing.
* **Almost every C++ citation.** I checked all of them: `ClassClass.cpp` `:136`-`:142`, `:396`,
  `:518`, `:522`, `:536`, `:543`, `:819`, `:823`, `:836`-`:843`, `:840`-`:849`, `:864`, `:923`-`:941`,
  `:952`, `:955`, `:966`-`:971`, `:991`-`:993`, `:1250`, `:1260`-`:1261`, `:1265`, `:1287`, `:1290`,
  `:1298`-`:1301`, `:1350`, `:1353`, `:1379`, `:1382`, `:1391`-`:1394`, `:1401`, `:1407`, `:1424`;
  `MethodDictionary.cpp` `:166`-`:171`, `:200`-`:202`, `:211`, `:221`-`:237`, `:233`, `:334`, `:348`,
  `:350`, `:453`, `:486`; `RexxBehaviour.cpp:444`-`:448`; `RexxBehaviour.hpp:82`; `MethodClass.cpp`
  `:183`, `:219`, `:457`-`:486`; `ArrayClass.hpp:247`; `ArrayClass.cpp:797`; `Setup.cpp` `:205`,
  `:456`, `:457`, `:463`, `:466`, `:478`, `:775`, `:792`-`:804`, `:933`, `:1307`-`:1312`,
  `:1399`-`:1404`, `:1737`, `:1809`; `PackageClass.cpp` `:1401`, `:1406`-`:1419`, `:1563`, `:1570`,
  `:1926`, `:1931`, `:1933`, `:1944`, `:1949`; `ClassDirective.cpp:209`; `ContextClass.cpp:160`,
  `:162`; `HashCollection.cpp:854`; `CoreClasses.orx:99` and `:110`. All correct except the two in
  findings 6 and 7. Notably right: `Setup.cpp:466` really is `AddMethod` and not
  `AddProtectedMethod`, which the comment says.
* **Three doc-comment measurements I re-ran and confirmed byte for byte:** `.K~define("STRING")` then
  `~method` -> `The NIL object`; `.K~uninherit(.Object)` -> 98.942 naming `The Object class`; and
  the rename claim -- with `~objectName=` set on both classes, 98.945 reads
  `Class "renamed" has not inherited class "mixrenamed".` on the oracle and on both engines.

---

## Findings

### C1 (critical). `.context` is a fresh object per evaluation, and this task made that observable

Reproduction, three descriptors, both engines identical:

```rexx
.context~objectName = "tagged"
say .context~objectName
```

Oracle rc 0, stdout `tagged`. Crate rc 0, stdout `a RexxContext`. Also:

```rexx
say (.context~identityHash == .context~identityHash)   /* oracle 1, crate 0 */
c = .context
say (c~identityHash == .context~identityHash)          /* oracle 1, crate 0 */
```

rc 0 with wrong stdout is the project's worst failure mode, and this is a three-line program.

**It is this task's.** Before this commit, `receiver_kind`'s `Body::Native(_)` arm refused every send
to `.context` loudly (the controller's note 3 records the message). This task added
`Primitive::Context` so that `RexxContext~PACKAGE` could resolve, which necessarily turned on every
Object-inherited method for that receiver at the same time -- `~identityHash`, `~objectName`,
`~objectName=`. `~package` itself is fine: it is cached and the corpus proves it. The object the
message is sent *to* is not.

**And the comment that says otherwise is the one the report leans on.**
`crates/rexx-exec/src/environment.rs:675`-`:686`, `Interp::context_object`:

> **A fresh object per evaluation where the oracle caches one per activation**, and the difference is
> not observable in this phase: a `RexxContext` answers no method this crate implements, and identity
> comparison needs `~==`, which is 5b's.

Both premises are now false -- a `RexxContext` answers six methods this crate implements, and
`~identityHash` witnesses the identity without `~==`. D1 cites this same comment ("`Interp::context_object`'s
own doc already calls identity comparison 5b's") as part of its justification for not building `==`.
The half it quoted is true; the sentence it sits in stopped being true in the same commit.

This is the exact instruction the controller's note 2 gave, one level up from where it was applied:
"treat a fresh-object-per-send implementation as the defect it is rather than as an unobservable
difference."

**Fix shape.** Cache one `RexxContext` per activation the way `package_objects` caches a package, or
-- if an activation genuinely has nowhere to root one -- refuse `~identityHash`/`~objectName`/
`~objectName=` on a `Primitive::Context` receiver loudly rather than answering from a throwaway. A
corpus row is available either way once the identity is stable. Whichever route is taken, the
`context_object` doc must stop claiming the difference is unobservable.

### C2 (critical). `~defineMethods` reads an unbuilt directory as an empty one

```rexx
.K~defineMethods(.local)
say "survived"
::class K
```

Oracle rc 163, stderr `Error 93.974:  The method source argument must be a string, array, or method
object.` under the `DEFINEMETHODS` frame. Crate rc 0, stdout `survived`, stderr empty. Both engines.

**Mechanism.** `native_define_methods` accepts a `Directory` receiver and walks it with the new
`Interp::native_keys`, which reads a `Body::Native`'s own map and nothing else. This crate's `.local`
has no entries at all -- its entries are the `unbuilt` table that `Interp::hash_entry_read`
(`environment.rs:719`-`:736`) refuses per name with `directory entry "SYSCARGS" is not implemented
(Phase 7)`. So the crate already knows those entries exist on the oracle and are not built here; the
new walk simply does not ask. The result is a mutation that silently does nothing.

`.environment` escapes only by luck: its entries *are* built, they are class objects, and the
not-a-method check fires, so that send is loud (verified: rc 120 against oracle rc 163). `.local` is
the reachable case today.

Before this commit the same send was a loud refusal, so this is a refusal turned into a wrong
answer, which the global constraints single out as the class no gate can see.

**Fix shape.** Before walking, ask `directory_scope(table)` and refuse loudly if the environment
model holds any `unbuilt` entry in that scope -- the same knowledge `hash_entry_read` already uses,
consulted one level up. D7's "the two collections it *can* walk" is the sentence to correct: it can
walk their maps, which is not the same as being able to read them.

### M1 (major). `~inherit`'s position argument reports the wrong error for a non-class value

```rexx
.K~inherit(.M1)
.K~inherit(.M2, "abc")
```

Oracle rc 158: `Error 98.945:  Class "The K class" has not inherited class "abc".`
Crate rc 158: `Error 98.942:  Class "abc" must be a MIXINCLASS for INHERIT.`
Same exit status, same frame line, same stdout; the message differs. Both engines.

The C++ never type-checks `position`. It uses it in exactly one place,
`superClasses->indexOf(position)` (`ClassClass.cpp:1346`), and a value that is not in the list gives
index 0 and `Error_Execution_uninherit` at `:1350` -- whatever kind of object it was.
`native_class_inherit` (`dispatch.rs`) instead runs the position through `class_receiver` and maps
the failure to `Raised::inherit_needs_a_mixinclass`, which is the *first* argument's error, not the
second's.

`ClassGraph::inherit_at`'s doc has this right -- "A `position` this class does not already inherit is
`InheritRefusal::NotInherited` naming it (`:1350`)" -- so the graph would answer correctly if the
dispatch arm let the value through. The fix is to drop the `class_receiver` conversion for the
position and let `inherit_at` do the identity search; a non-class simply never matches.

D3 built this argument deliberately and gave it a corpus row for the successful shapes. The refusal
shape has none, which is why it shipped. It needs one.

### M2 (moderate). Two doc comments in the files this diff edits are falsified by this diff

Both assert the deferral and the missing mechanism that this task removed.

* `crates/rexx-exec/src/dispatch.rs:940`-`:944`, `Interp::receiver_kind`: "A stem answers `Stem` on
  the oracle, and `rexx_classes::deferred_classes` does not build that class (its `Setup.cpp` block
  hides the comparison methods, and `MethodDict` models no removal), so a stem receiver resolves
  nothing and fails loudly."
* `crates/rexx-exec/src/lib.rs:648`-`:653`, `Loud::receiver_class`: "The reachable case is a stem:
  `rexx_classes::deferred_classes` leaves `.Stem` out because its `Setup.cpp` block hides the
  comparison methods and `MethodDict` models no removal."

`.Stem` is now built (I read `.Stem~method("==")` off the crate) and `MethodDict` now models both
removal and hiding -- the module doc two files away says so. The *conclusion* still holds: a stem
value still has no receiver arm and `a. = 'dflt'; say a.~length` is still rc 120 loud, measured. Only
the reason is now wrong, in the two places a reader would go to find out why.

This is the category the report itself opened (concern 4) and then searched only one instance of.
A `/bin/grep -a "models no removal"` over `crates/` finds both in one pass.

### m1 (minor). Three C++ citations in one sentence, all landing off the code they name

`dispatch.rs`, `method_name_argument`:

> `stringArgument(method_name, "method name")->upper()`
> (`classes/ClassClass.cpp:826`-`:828`, `:963`, `:986`)

* `:826`-`:828` is `}`, a blank line, and the first line of a comment. The call is `:831`-`:832`.
* `:963` is the comment "does not suddenly show up in existing instances of this class." The call in
  `deleteMethod` is `:961`.
* `:986` is the comment "make sure we have a proper name". The call in `RexxClass::method` is `:987`.

The claim is true of all three functions; none of the three pins is on the line. This is the defect
class that has shipped on three consecutive tasks of this plan.

### m2 (minor). `memory/Setup.cpp:1216` is a blank line

`environment.rs`, `running_package_object`: "`RexxContext::getPackage` (`classes/ContextClass.cpp`'s
`getPackage`, bound by `memory/Setup.cpp:1216`)". The binding is
`AddMethod("Package", RexxContext::getPackage, 0);` at `:1218`; `:1216` is empty and `:1213` is the
class-method block above it.

### m3 (minor). Two comments name the size of a set

`crates/rexx-classes/tests/native_classes_wiring.rs`:

* "`Queue`'s own set is `Array`'s, minus **the nine names** `Setup.cpp:792`-`:804` removes"
* "**The six names** `Setup.cpp:1307`-`:1312` hides are absent"

Both name a cardinality where the citation already names the set. The rule is in
`global-constraints.md` ("A comment may not name the size of a set. Name the set; true counts
included"), and this commit applies it correctly elsewhere -- it renames
`the_thirteen_classes_untouched_by_the_prologue...` to `every_class_untouched...` and
`the_twelve_prologue_mutated_classes...` to `every_prologue_mutated_class...` for exactly this
reason.

Borderline, listed but not counted separately: "The five mutators" (`dispatch.rs`) and "**The five
methods that read it are the five that mutate a class**" (`class_graph.rs`). Both enumerate the set
in the same sentence, so the count adds nothing and could go; both are closer to the tolerated "the
two arms of that `if`" shape already in the tree.

### m4 (minor). D4's "what that cannot distinguish" list is short by one

`~define(name, .nil)` also differs on `hasUninitDefined`. In the C++, the `.nil` arm leaves
`methodObject` at `OREF_NULL`, so `if ((MethodClass *)TheNilObject != methodObject)`
(`ClassClass.cpp:852`) is *true* and `:854`-`:857` sets `hasUninitDefined` when the name is `UNINIT`
-- where the omitted-argument arm, which stores `TheNilObject`, does not. The crate's `.nil` arm
calls `delete_instance_method`, which touches no uninit flag.

Not observable this phase (uninit needs an instance, so it is 5b's, same as D43's witness), and I am
not asking for a behaviour change. D4 should record it beside the stored-null-versus-absent point it
already records, since a reader of D4 would otherwise conclude the `.nil` arm was fully analysed.

### m5 (minor). Report section 7 overstates the axis agreement, and its Task 20 column is not one sitting

Reading `bench-baselines/phase-5a-arms.tsv` directly, `across_builds` / `instructions:u`:

* The report says six of eight axes "read exactly what Task 20's sitting read, to six decimal
  places". `arith` does not: ir small `0.989742` -> `0.989743`, ir large `0.989572` -> `0.989573`.
  Five axes are identical, not six. The move is one unit in the sixth decimal and changes nothing.
* Task 20 has three separate `pinned>head` blocks in that file, and the figures in the report's table
  are drawn from different ones (`ir small 1.017433` is from the second block, `tw large 1.015397`
  from the third; the third block reads `ir small 1.017435`). The three blocks agree to about
  0.0001, so the conclusion -- largest move +0.20% on `dispatchclass ir small`, nothing near the 1%
  threshold, three of four `dispatchclass` arms down -- is unaffected. But "task 20" as a column
  label implies one sitting and is not one.

I accept the perf conclusion. No sitting is owed.

### m6 (minor). `drop_method_object` cannot un-root what it forgets

`hold_method_object` roots a method object with `RootSet::add_global`, and `roots.rs` has no
`remove_global`. `drop_method_object` clears `method_objects` but the global root stays, so a
`~delete`d or `.nil`-defined method object is never collectable. Bounded by the number of distinct
(class, name) pairs a program ever defines, and not a divergence -- the map is what `~method` reads,
and it is cleared correctly. Recording it because the doc's sentence ("an entry that stops existing
must not leave its object behind") reads as though the object is released, and it is not.

---

## The nine decisions

| | verdict | why |
| --- | --- | --- |
| D1 | accept | `==` on a `Body::Native` is a loud refusal here, the `~identityHash`-with-`==` substitute rejects the same defect, and the `=`-at-`NUMERIC DIGITS 9` reasoning is right. Caveat: it cites `Interp::context_object`'s doc as authority, and that doc's other half is what finding C1 falsifies. |
| D2 | accept | Verified: `.SUB~hasMethod("M")` is `0` before and after `~define`, so a `~hasMethod` row would have been a pair of zeros any build passes. `~method` is the only class-side reader that moves. |
| D3 | accept the semantics, reject as complete | The insert-at-the-index reading is right in the C++ and against the oracle, and the corpus pins both insertion points. The non-class position refusal is wrong -- finding M1. |
| D4 | accept, incomplete | The three-shapes table is right and measured. The undistinguished-cases list omits `hasUninitDefined` -- finding m4. |
| D5 | accept | Verified: `.Array~package~publicClasses["ORDEREDCOLLECTION"]` is `The OrderedCollection class` on the oracle and no class this crate registers carries that name, so a partial table would answer `The NIL object`. Honest that nothing automated catches it. |
| D6 | accept | Verified: `.K~define("SRC", "say 'x'")` then `~method` prints `a Method` at rc 0 on the oracle; the crate refuses loudly. Nothing here compiles a method body outside a directive. |
| D7 | accept the shape, reject as complete | Sorted-before-allocating is right and I confirmed the output is deterministic over twelve runs; the `supplier_refusal` 97.1 arm matches byte for byte. Reading the two hash collections' maps directly is what produces finding C2. |
| D8 | accept | The rewritten comment is true: `removeSetupMethods` (`ClassClass.cpp:923`-`:941`) walks `.Object`'s subclass tree deleting from each behaviour, and `delete_instance_method` cascades the instance side alone. Replacing a justification the commit made false is the right move, and is the same move findings C1 and M2 are asking for elsewhere. |
| D9 | accept | Re-derived: sha256 matches `PINNED.md`, the pin is an ancestor, and every commit touching crate source since the pin is inside `d926c334d..HEAD` (empty set difference). The substitute reading of the test is the right one. |

## The five concerns

1. **Wrong number in `7d1544f84`'s message.** Accept. The TSV reads 0.988201 -> 0.992210 on that
   cell, four thousandths. Recording it in the report was the correct response to a message that
   cannot be amended.
2. **Two loud refusals with no corpus instrument (D5, D6).** Accept, and the honesty is right. One
   cheap improvement: neither refusal has an in-crate assertion either, so nothing at all would
   notice `Loud::rexx_package_classes` or `Loud::method_from_source` becoming an answer. A test that
   asserts the refusal fires is the "in-crate test only" answer the global constraints explicitly
   allow, and it is better than nothing automated.
3. **`Queue class` and `Stem class` moved from a loud divergence to a silent one.** Accept.
   Licensed by the brief in terms, and I confirmed the divergence is confined to the `superclasses`
   line of each row. Worth noting for Task 23: it propagates into value positions, e.g.
   `.Queue~superClasses~items` is `2` on the oracle and `1` here at rc 0.
4. **`Interp::method_object`'s doc is unwitnessable as written.** Accept the analysis. I would not
   defer it. The task established the fact, the fix is one clause, and finding C1 is what deferring
   a known-false neighbouring sentence costs. Rewrite the negative half around the `~objectName`
   row, which does discriminate, and drop the `=` row's counterfactual.
5. **`==` on an interpreter object stays a loud refusal.** Accept. Verified: `p~publicClasses ==
   p~publicClasses` is rc 120 here, and the `~identityHash` rows carry the same discrimination.

---

## What I did not check

* I did not rebuild, so I did not re-run the controls myself; I verified them by file hash, by the
  logic of each mutation, and by confirming the tree is clean.
* I did not run `gate_table_c` or any cargo command, to avoid touching the shared `target/` while
  other agents may be live. The gate-table claims in report section 5 are instead verified by running
  that file's own probe text (`gate_table_c.rs:576`-`:593`) through both engines and the oracle by
  hand.
* `~defineMethods` with a table holding `.nil` is unreachable this phase (`[]=` on a `StringTable` is
  a loud refusal), so the `None` arm of `define_method_table` is untested by anything I could run.
  It reads correctly against `ClassClass.cpp:1260`-`:1261`.
