# Task 23, fix round 1: scoped re-review

Range `87cf62507..HEAD` (`6fbda00fe`), three commits. Scope: the eleven findings of
`task-23-review.md` and nothing else.

**Verdict: CHANGES REQUIRED.** The code is right everywhere I could reach it -- M1's fix is the
oracle's rule and not a shape fitted to the probes, and I could not break it. What fails is prose:
the breadth re-measurement that bounds M1 is wrong in both directions it states, one withdrawal is
claimed but not made, and two of the sentences this round wrote into `src/` are false.

Everything below was run. Probes came from freshly created empty directories with absolute paths,
three descriptors read separately, both engines (`REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker` --
`tw` is not an accepted value and exits 2, which is worth knowing before a sweep reports 73 engine
splits). I did not rebuild the crate; the one check that needs a rebuild says so.

---

# The breadth re-measurement is wrong, and it is the claim that bounds M1

The report says:

> All 73 `::METHOD ... CLASS` pairs from the two embedded files, called with no arguments, three
> descriptors, fresh directory: **7 match, 14 loud, 51 diverge** -- and **all 51 differ only in the
> two `Error` lines** ...

7 + 14 + 51 is 72, not 73. Re-derived at HEAD, twice, with two different call shapes
(`say .Cls~meth()` and `say .Cls~meth`), 73 programs each, oracle and both engines:

| | match | loud | diverge |
|---|---|---|---|
| `say .Cls~meth()` | 7 | 14 | **52** |
| `say .Cls~meth` | 7 | 14 | **52** |

The 73 pairs were extracted the same way (most recent `::class` above each `::method ... class`,
both embedded files, quotes stripped) and the count agrees with the review's 73. `loud` is crate
rc 120; all 14 are `rexx-exec: ...` on stderr (7 x `~new`/`ARG`, 7 x Phase 7 library entry points),
and no row classed `DIVERGE` has a `rexx-exec:` line, so the partition is not sensitive to the
definition.

**The property the report asserts does hold, for all 52.** Stripping every line beginning `Error `
from both stderrs and comparing the remainder, plus stdout compared whole: **52 of 52 identical, 0
differing.** Every traceback frame and every program name matches. That is the real result and it is
a better one than the report claims.

**The sentence explaining the missing pair is false too.**

> The one pair that moved from diverging to matching outright is the `.Validate~number` shape, which
> raises its condition outright rather than through `USE STRICT ARG`.

Nothing moved. The review measured 7/14/52 before the fix and I measure 7/14/52 after it; the fix
changed *what* differs inside the 52, not how many there are. `.Validate~number` **with no
arguments** does not raise 88.902 outright -- it goes through `USE STRICT ARG` like the rest and is
one of the 52:

```
oracle   3697 *-* Method NUMBER with scope "Validate" in package "REXX" (no source available).
            1 *-* say .Validate~number()
         Error 93 running REXX line 3697:  Incorrect call to method.
         Error 93.901:  Not enough arguments for method; 2 expected.
crate    (same two frame lines)
         Error 40 running REXX line 3697:  Incorrect call to routine.
         Error 40.3:  Not enough arguments in invocation of NUMBER; minimum expected is 2.
```

The `.Validate~number('LENGTH', 'abc')` case the review isolated -- **with** arguments -- is the one
that now matches, and it is a corpus row, not a member of this sweep.

Third occurrence of the same number: "**Task 23 did not introduce it; it made 51 programs reach
it**" is 52.

---

# M1's fix, checked as a mechanism

**The C++ the report cites is accurate at every line.** `PackageClass::traceBack` extracts at
`classes/PackageClass.cpp:571`, tests the miss at `:575`, and reaches
`activation->formatSourcelessTraceLine(programName)` at `:589`; the body is
`execution/RexxActivation.cpp:5057`; its three arms are 101.24 / 101.25 / 101.26
(`messages/rexxmsg.xml`, subcodes 024/025/026), matching the crate's numbering.
`LIBRARY_PACKAGE_NAME`'s `classes/PackageClass.hpp:147` is `getProgramName`, and the REXX package
really is created with that literal name and a sourceless `ProgramSource`
(`MemoryObject::createRexxPackage`, `memory/Setup.cpp:215`).

**The banner rule is the oracle's rule, not a fitted shape.** `Activity::generateProgramInformation`
(`concurrency/Activity.cpp:1090`-`:1123`) walks the frames from the top and takes **the first frame
whose `getPackage()` is not null**, using that one frame for both `POSITION` and `PROGRAM`;
`Activity::display` (`:1443`-`:1455`) prints `running <PROGRAM> line <POSITION>` from those two
entries. The crate takes the innermost site that has a line, and uses that same site for both the
number and the name. The two agree because the frame kind with no line in this crate
(`FailureSite::Rendered`, a `Compiled method ...` echo) is `InternalActivationFrame`, whose
`getPackage()` returns `OREF_NULL` (`concurrency/ActivationFrame.cpp:111`-`:113`). So "a `Rendered`
entry decides neither" is the C++'s own behaviour.

**I tried to break it and could not.** Oracle against both engines, three descriptors, fresh
directory:

* Two library frames under a program clause, `.TimeSpan~fromDays('x')` -- identical, and the two
  frames carry different indents (3700 is inside a `DO`, 3095 is not).
* Two library frames under an internal `CALL` under the program (`call sub` / `sub:`) -- identical,
  four echo lines, banner `REXX line 3700`.
* Two library frames under a **user class method** under the program -- identical.
* Two library frames under an `INTERPRET` -- identical, banner still `REXX`.
* A **native frame innermost inside a library frame**: `.Validate~classType('N', 'x', 5)` reaches
  `object~isA(class)` and raises there. Oracle prints `*-* Compiled method "ISA" with scope
  "Object".` above the sourceless `CLASSTYPE` frame and names `REXX line 3810`. Byte-identical on
  both engines -- this is the case that exercises "a `Rendered` entry decides neither".
* `signal on syntax` around a library raise, then `raise propagate`: identical, including the
  positionless `Error 88:` banner.
* Mixed-case scope ids render right (`scope "TraceObject"`, `scope "File"`, `scope "TimeSpan"`).

**The one case that would test the other direction is loud, not answered.** "A library frame under a
user frame under a library frame" is reachable in the oracle through
`.Validate~requestClassType(name, obj, class)`, which sends `obj~request(...)` and so can land in a
user `::method makeString`. The oracle then names the **program's path** and the user clause's line,
with the library frame below it:

```
     4 *-* raise syntax 88.902 array('Z', 'q')
       *-* Compiled method "REQUEST" with scope "Object".
  3819 *-* Method REQUESTCLASSTYPE with scope "Validate" in package "REXX" (no source available).
     1 *-* say .Validate~requestClassType('X', .K~new, .String)
Error 88 running /abs/path/a1.rex line 4:  Invalid argument.
```

The crate cannot get there: every route to a user *instance* is
`rexx-exec: method "NEW" of class "Object" is not implemented (Phase 5)`, rc 120. The rule the crate
implements would give the same answer (the innermost line-bearing site is the user's clause, which
is a `Clause` and so falls back to `site.path`), but that is an argument, not a measurement --
nothing can witness it in this phase. Worth a sentence somewhere, not a change.

`condition('o')` is loud (`CONDITION option "O" answers a Directory, which is not implemented`), so
the oracle's `PROGRAM` entry -- `REXX` -- is not observable through any second channel either.

**The two new corpus rows are byte-identical**, oracle against both engines, three descriptors, and
rc 168 on all six runs. The nested row does carry two frames at two indents, as claimed.

## 101.25, the routine arm

**The stated fact is true and the stated reason is narrower than the arm.** No embedded file
declares a `::ROUTINE`: `/bin/grep -aic "::routine"` is 0 for `CoreClasses.orx`,
`StreamClasses.orx` and the unix `PlatformObjects.orx`, and a leading-whitespace-tolerant pattern
finds nothing either, so the check survives being widened.

But `::ROUTINE` is not what triggers the arm. `isRoutine()` is `activationContext == EXTERNALCALL`
(`execution/RexxActivation.hpp:171`), and an ordinary `CALL 'file'` runs with `EXTERNALCALL`
(`instructions/CallInstruction.cpp:459`). `CoreClasses.orx:122` is `call 'StreamClasses.orx'
rexxPackage` and `:124` is `call 'PlatformObjects.orx'`, so a failure inside either of those
prologues would take the oracle's `isRoutine()` arm with no `::ROUTINE` in sight -- where this
crate's `sourceless_site` would answer 101.26, since `method_identity` is `None` there. That path is
reachable only if the bootstrap itself fails, which is the same bound the crate's own doc already
gives for the 101.26 arm, so nothing observable turns on it. The sentence

> so no activation this crate can put in a sourceless package is a routine

does not follow from its premise and is the one I would strike or re-base.

Nothing else reaches the arm: `::REQUIRES` is loud, and `Package~addRoutine` is 98.984.

---

# Per-finding verdicts

## M1 -- code CLOSED, prose CHANGES REQUIRED
The divergence is gone and the mechanism is the oracle's. The paragraph that bounds it states 51
where 52 is right, three times, and invents a pair that moved. See above.

**Not verified:** the applied-and-inverted control ("making `Interp::sourceless_site` answer `None`
reddens exactly those two rows and leaves `library_bootstrap_state`,
`library_bootstrap_setup_methods_gone` and `string_upper` green"). Confirming it needs a rebuild.

## M2 -- CLOSED at the three named sites, two residual instances of the same shape
All three are genuinely fixed. `class_graph.rs` now has two doc blocks, one per function, neither
contradicting itself; `registry.rs`'s three false sentences are gone and its donor list is right at
every line (`Setup.cpp:775` Queue<-Array, `:861` Table, `:881` StringTable, `:908` Set, `:933`
Directory<-StringTable, `:958` Relation, `:988` Bag<-Relation, all checked against the file);
`native_classes.rs`'s module doc now names the macro. `MethodDict::replace_methods_from` really does
carry the distinction and the cost the three sites forward to.

Two things of the same shape survive **in the file this round edited**:

1. `crates/rexx-classes/src/native_classes.rs:117`-`:118` -- "`Table`, `StringTable`, `Set`, `Directory`,
   `Relation` and `Bag` are the `InheritInstanceMethods` users." **Queue is a seventh.**
   `Setup.cpp:775` is `InheritInstanceMethods(Array)` inside `StartClassDefinition(Queue)`, the
   generated table has it (`out/setup_classes.rs`: `Queue <- Array`), the replay loop donates for it,
   and `registry.rs`'s own doc -- rewritten in the same commit -- lists `Array -> Queue (:775)`
   first. The round rewrote this sentence and did not fix the omission; it also dropped the hedge
   that had made it a claim about which recipients R8 enabled rather than about `Setup.cpp`.
2. `crates/rexx-classes/src/native_classes.rs:371` -- "Replaying in checklist order instead would
   call `inherit_instance_methods` against a donor not yet built." The loop calls
   `donate_instance_methods`. This is the same two-functions-one-name confusion M2 is about, in the
   same file, twelve lines below the `Op::InheritInstanceMethods` arm that was corrected.

## D1 -- CLOSED
`package_name`'s comment is accurate. `Program` carries no path field
(`crates/rexx-parse/src/lib.rs:102`-`:113`), so "`Interp::programs` holds no path per program" is
true; `record_package_class`'s `library_programs` skip is at `environment.rs:963` and does what the
comment says.

## D2 -- CLOSED
The false evidence is struck and the replacement is right: `/bin/grep -inc call StreamClasses.orx`
is 2, both in comments (`:363` "By calling lineout", `:534` "system calls"), and neither survives the
new `"call "`-plus-quote scan. The windows file really does carry `  call 'orexxole.cls'` at line 2.
The scanner is genuinely wider: the old `trim_start().strip_prefix("call ")` could not match
`CALL 'CoreClasses.orx'` and the new lowercased search can.

Nit, comment rule: "an indented `call` was already matched **before this widening**" and "so
'neither file contains the word' **would have been false evidence** for a true property" are both
history. Strike each and the sentences say the same thing about the code as it is, which is the
constraints file's own test for the borderline case.

## m1 -- HALF CLOSED
The `-0.50%` figure is stated honestly: 5,406.48 against 5,433.48 is -0.497%, the report calls it
-0.50% and 27 instructions a pass, says it is under the guard's line, and says it is not a rounding
step. Good.

The attribution is not withdrawn. Report line 537 still reads, unqualified:

> So this is the collector: after the bootstrap the live set holds the library's classes, their
> method objects and `.environment`'s entries, and every collection traces all of it.

and line 555 then says "**What that does not establish, and an earlier draft of this section claimed
it did.**" It is not an earlier draft; it is eighteen lines above, in the same section, unmarked. So
the fix-round bullet's "'So this is the collector' is withdrawn" is false about the text as shipped.
The brief asked for "state the observable and stop there" -- the stopping sentence exists, the
starting one was not deleted.

## m2 -- CLOSED
`CoreClasses.orx:77` is `-- we do a phony inherit to add those directly to the class instance` and
`:93` is `.string~inherit(.Comparable)`, the first `~inherit` of the run. Both corrected sites use
the right numbers.

## m3 -- behaviour CLOSED, one new false sentence
`PackageClass::findClass` is at `classes/PackageClass.cpp:1081` and its order is as the doc now
states. The `::REQUIRES` probe reproduces: the crate is
`rexx-exec: ::REQUIRES is not implemented (Phase 5)` at rc 120 where the oracle answers `K` at rc 0.
I also swept `::class K subclass <X>` for seventeen library class names, public and non-public
(`SupplierMixin`, `ManyItemMixin`, `SetMixin`, `BagMixin`, `LocalServer`, `Ticker`, `Singleton`,
`TraceObject`, `StreamSupplier`, `Comparable`, `Orderable`, `Validate`, `ArgUtil`, `RexxQueue`,
`Stream`, `File`, `DateTime`): **17 of 17 byte-identical on three descriptors**, so "No program can
see the difference today" survives.

The new sentence justifying it does not:

> The two public-class steps need `::REQUIRES`, which is Phase 5c's

The second of the two is `TheRexxPackage->findPublicClass(internalName)`
(`PackageClass.cpp:1109`), whose contents come from the **image build**, not from any import:
`MemoryObject::completeSystemClass` (`memory/Setup.cpp:199`-`:206`) puts every system class into
`TheEnvironment` **and** into `TheRexxPackage` as a public class, in the same two lines. That is why
the sweep above comes back clean -- the crate's `.environment` step is the substitute for step 3,
not a step it skips. Calling step 3 something "this crate has nothing to consult" and pinning it on
`::REQUIRES` gets a correct decision for a wrong reason.

## m4 -- CLOSED
`/bin/grep -rn "production loads one program" crates/` is 0; the sentence is at
`docs/superpowers/records/2026-08-09-phase-4e-ir/task-10-report.md:198`; `plan.rs:415` is the
`debug_assert_eq!`. The uncorrected attempt-1 wording survives at report line 243 but concern 3
explicitly names it and says both attempts cited it, so that one is bookkeeping rather than a
standing claim.

## m5 -- CLOSED, every figure reproduces
Re-ran both binaries myself under `REXX_CORPUS_GATE=1`.

| | class wiring | hierarchy edges | 5a rows not yet `agree` |
|---|---|---|---|
| `gate_table_c-88c89472c7b48c4f` (03:42) | 15 `agree`, 35 `diverge-both`, 13 `diverge-stdout` | 57 `diverge-both` | 110 of 135 |
| HEAD (`gate_table_c-70b9f1c94ba623c9`) | 62 `agree`, 1 `diverge-both` | 57 `diverge-both` | 61 of 135 |

+47 `agree`, all 13 `diverge-stdout` rows gone. The 13 names in the report are exactly the 13 rows,
in that order, and all carry `loud=no`. The single remaining wiring divergence is `RexxInfo`. The
57 hierarchy rows are all waiting on `method "HASITEM" of class "Array"` (the summary block counts
57). The provenance caveat is present and correctly attributed.

## m6 -- CLOSED. ## m7 -- CLOSED
The blank line is gone. `EXEMPT`'s doc no longer says "now that" and the replacement is true.

---

# Comment rules over the diff

* **ASCII:** `git diff | /bin/grep -a '^+' | /bin/grep -acP '[^\x00-\x7F]'` is 0. No em-dashes, no
  smart quotes.
* **Set cardinalities:** none beyond the small enumerated counts the tree uses as house style.
* **Historical framing:** two instances, both in `rexx-lib/src/lib.rs`, listed under D2 above.
* **C++ citations added or changed:** `PackageClass.cpp:575`-`:589` (correct), `:589` (correct),
  `:1081` (correct), `PackageClass.hpp:147` (correct), `RexxActivation.cpp:5057` (correct),
  `RexxBehaviour.cpp:350` (correct, three sites), `ClassClass.cpp:558`-`:586` (correct -- `:586` is
  the closing brace), `ClassClass.cpp:923` (correct), `Setup.cpp:775`/`:861`/`:881`/`:908`/`:933`/
  `:958`/`:988` (all correct), `MethodDictionary.cpp:251` (unchanged, referenced),
  `CoreClasses.orx:77` and `:93` (correct).
  One is off by one: `class_graph.rs` quotes `MethodDictionary *sourceMethods =
  source->instanceMethodDictionary; sourceMethods->setMethodScope(this);` and cites `:560`-`:562`.
  The second statement is on `:563`; `:562` is its comment. The round reformatted this range without
  fixing it.

---

# What to change

1. The breadth paragraph: **52**, not 51, in all three places, and delete the sentence about a pair
   moving from diverging to matching. The finding worth stating is that the classification did not
   move at all and the *content* of all 52 divergences narrowed to the two `Error` lines.
2. Delete "So this is the collector: ..." at report line 537, or re-word the retraction so it does
   not describe the sentence eighteen lines above it as belonging to an earlier draft.
3. `native_classes.rs:117`-`:118`: add `Queue`, or say what set the six are.
4. `native_classes.rs:371`: `donate_instance_methods`.
5. `environment.rs`: `TheRexxPackage`'s public classes do not need `::REQUIRES`; they are what
   `.environment` and the native name table stand in for.
6. `error.rs`: the 101.25 sentence's conclusion is wider than its premise.
7. Optional: `ClassClass.cpp:560`-`:563`; strike the two historical clauses in `rexx-lib`.
