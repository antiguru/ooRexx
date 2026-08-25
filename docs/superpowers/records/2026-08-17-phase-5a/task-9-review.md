# Task 9 review: the Object and Class reflection protocol

Base `4729e5d3a`, head `3e695182c`. Read-only review; nothing in the checkout was modified but this
report, and `git status` is clean with HEAD still at `3e695182c`. All
probing was done from a fresh directory of my own with an out-of-tree `CARGO_TARGET_DIR`, because the
release binary in `rust/target/release/` predates the head commit by six minutes and is therefore an
attribution build, not this head.

## Spec Compliance

**Spec compliant on the build, with one criterion the brief words in a way this task cannot satisfy.**

Every method the brief names answers on both engines, and every expected answer the brief puts on the
table holds. I re-ran all eight new corpus programs against the live oracle, three descriptors read
separately, both engines, from a fresh directory:

| program | rc | verdict |
|---|---|---|
| `class_reflection.rex` | 0 | byte-identical, both engines |
| `class_package.rex` | 0 | byte-identical, both engines |
| `class_method_own_dictionary.rex` | 159 | byte-identical, both engines |
| `class_method_class_side_raises.rex` | 159 | byte-identical, both engines |
| `class_reflection_argument_ladder.rex` | 168 | byte-identical, both engines |
| `array_make_string.rex` | 0 | byte-identical, both engines |
| `array_make_string_refusals.rex` | 163 | byte-identical, both engines |
| `array_unknown_method.rex` | 159 | byte-identical, both engines |

The brief's own measured rows all reproduce: `.K~superClasses` is `The Object class`, `.K~metaClass`
is `The Class class`, `.K~isA(.Class)` is `1`, `.Array~package~name` is `REXX`, `.array~id` is
`Array` (from the derived table C probe), and `.K~method('M')` is rc 159 carrying
`*-* Compiled method "METHOD" with scope "Class".` ahead of the 97.1.

**`~identityHash`** answers the handle, as deviation 4 licenses, and the in-crate test is the whole
instrument, as the brief anticipated. Measured, `.Class~superClasses~identityHash > 0` is `1` here and
`0` on the oracle -- the oracle's answer is address-derived and negative -- which is the same
deviation seen from another angle and not a new one.

### The unmet "Done when" -- the implementer's account is correct, and the remainder is another task's

I reproduced the wiring count independently by running `gate_table_c` at head: **14 `agree`, 11
`diverge-stdout`, 38 `diverge-both`**, and the 25 named classes in the report's two lists are exactly
the 25 the run produces. So the headline number is real.

The eleven `diverge-stdout` rows are registered classes, so the criterion as the brief words it is
genuinely not met. The account of why is correct in every particular I could check:

* diffing the derived probes for `String`, `Array`, `Set` and `Message` one by one on both engines,
  each differs on **exactly** the `superclasses` line and nowhere else, stderr identical, rc 0 both
  sides;
* the missing entries are exactly `CoreClasses.orx`'s `~inherit` targets, and all fourteen line
  citations in the report's table are correct in both copies of that file
  (`interpreter/RexxClasses/CoreClasses.orx` and `build/bin/CoreClasses.orx` agree);
* `native_classes.rs`'s module doc (lines 81-102) already says, in terms, that full post-prologue
  correctness for these classes -- "their `~superClasses`, their complete flattened method set" -- is
  **Task 13's** and is not asserted there. That predates this task.
* the 38 `diverge-both` rows all fail at `.NAME` with
  `rexx-exec: environment symbol ".X" is not implemented (Phase 5)`, and the 38 names are exactly the
  three deferrals the brief names plus the 35 the report lists.

So the gap is Task 13's (the prologue) and Task 21's (the three deferrals), and nothing in it is work
this task declined. What is wrong is the criterion: the brief carves out only
`Queue`/`Stem`/`VariableReference` and did not anticipate that a registered class's `~superClasses`
also depends on the prologue. See Issues, Important #2.

### Additions beyond the brief -- both necessary, neither scope creep

* **`Array~makeString`.** `corpus/gate-tables/classes/array.rex` -- a *derived* file the pre-existing
  `gate_table_c.rs` regenerates and compares in both directions -- asks
  `say 'superclasses' .Array~superClasses~makeString('L', ' ')`. No wiring row can reach `agree`
  without it. Necessity established independently of the report's claim.
* **`Package~name`.** The brief itself names `.Array~package~name` is `REXX` as a measured expected
  answer.

`array_make_string_refusals.rex` goes further than either needs, but implementing `makeString` means
owning its refusals, and the ladder pins the two places the oracle's declared arity and its body
disagree. Reasonable.

### Named risk 1: `~method` reads the class's own instance dictionary, attacked from both sides

I probed 31 further names beyond the corpus, both directions, oracle and both engines, and found
**zero divergence**. The set deliberately included the shapes most likely to break a dictionary
model:

* names in Setup.cpp's own `.Array`/`.Class`/`.Object`/`.String`/`.Method`/`.Package`/`.Supplier`/
  `.WeakReference`/`.Routine` blocks that this crate implements nothing for (`AT`, `ITEMS`, `SUBSTR`,
  `METHODS`, `SUBCLASS`, `ENHANCED`, `REQUEST`, `SOURCE`, `SETSECURITYMANAGER`, `AVAILABLE`, `VALUE`,
  `CALL`, `ISABSTRACT`, `SETMETHOD`, `OBJECTNAME=`): oracle answers `a Method`, so does this build;
* operator and suffix names (`==`, `[]`, `[]=`, `||`, `+`): both answer;
* `UNKNOWN`, `.Class~method('NEW')` and `.Array~method('OF')`: both raise 97.1 (class-side or absent);
* **the image-build casualties.** `RexxClass::removeSetupMethods` (`classes/ClassClass.cpp:923`)
  strips `DEFINECLASSMETHOD` and `INHERITINSTANCEMETHODS` from `instanceMethodDictionary` after
  `Setup.cpp:460`-`:461` add them. A model derived from `Setup.cpp` alone would answer here where the
  oracle raises. This build raises, on `.Class` and on `.Object`, on both engines.

The one direction where the two do part is not the dictionary: the oracle returns the *same* method
object on repeated sends and this build mints a fresh one (`(.Array~method('APPEND') ==
.Array~method('APPEND'))` is `1` on the oracle). It is unobservable today because every send to a
method object refuses loudly -- see Minor #4.

### Named risk 2: the `~class` / `~metaClass` split

`native_class` reads `ClassRegistry::class_of`, which is `ClassGraph::owning_class`
(`crates/rexx-classes/src/registry.rs:206`); `native_metaclass` reads `ClassRegistry::metaclass`
(`:211`). Right field in each. I ran the parting shapes the corpus does not cover -- `::CLASS T
SUBCLASS MC METACLASS M1`, `::CLASS M3 SUBCLASS MC METACLASS MC`, `::class MC MIXINCLASS Class`,
`::class Z SUBCLASS Class`, alongside the corpus's `T2 SUBCLASS MC` -- and every `~class`,
`~metaClass`, `~superClass`, `~isA` and `~isSubclassOf` row matched the oracle byte for byte on both
engines.

`~request` does not answer here, so there is nothing to be inconsistent with. The rubric's citation
checks out: `ObjectClass.cpp:1916` is
`Protected<RexxString> class_id = behaviour->getOwningClass()->getId()->upper();` inside
`RexxObject::requestRexx`, so when `~request` lands it must read `class_of` and not `metaclass` --
which is what `native_class` already does.

### Named risk 4: the two controls

**Control 1 is a real control, and the report's reasoning about the exit status is correct.** I
verified the load-bearing half: `gate_table_c` asserts `gated.is_empty()`
(`crates/rexx-exec/tests/gate_table_c.rs:1962`), and the run at head reports **`5a: 135 rows, 117 not
yet agree`**. So under `REXX_PHASE_GATE=5a` the command fails with or without the mutation, exactly as
the report says, and the discriminating signal has to be the verdict counts and the named row. I
confirmed the unmutated side of the comparison (14/11/38) from my own run; I did not re-apply the
mutation, so the 13/11/39 half rests on the report.

**Control 2's logic holds, and the two-mutation claim is right for a checkable reason.** Under
`has_own_instance_method` -> `has_method`, `.Array~method('STRING')` answers because `STRING` is in
`.Array`'s flattened instance behaviour, so `class_method_own_dictionary.rex` reddens; `M` in
`class_method_class_side_raises.rex` is a `::METHOD ... CLASS` and is absent from `K`'s instance
behaviour flattened or not, so only the second mutation (`class_has_method`) reaches it. One mutation
genuinely cannot redden both programs. I did not run the mutations.

### What I could not verify, and what the controller should check

* **The two sittings' provenance and the four attribution builds.** The rows in
  `bench-baselines/phase-5a-arms.tsv` match the report's table to six decimals for both commits, and
  the axes at 1.000000 are 1.000000 in the file. The four attribution builds are not in the file by
  design, so their figures rest entirely on the report; reproducing them means rebuilding four
  binaries and re-running the sitting.
* **Control 1's mutated run** and **Control 2's two mutated runs** (logic checked, runs not repeated).
* **The five gate commands.** I ran `cargo fmt --all --check` (exit 0) and
  `cargo clippy --workspace --all-targets -- -D warnings` (exit 0, no warning lines) myself, and
  `gate_table_c` under `REXX_CORPUS_GATE=1` (exit 0). The three suite-wide commands and the 143-of-143
  corpus count rest on the report; the eight new programs I verified by hand.
* **The staleness test.** I did not re-derive the 34/36-commit list or re-hash the pinned binary.

## Strengths

* **The load-bearing detail is right in a way that survives adversarial probing.** 31 extra `~method`
  names, both directions, including the two methods `removeSetupMethods` deletes after `Setup.cpp`
  installs them -- a model built from `Setup.cpp` alone gets those wrong, and this one does not.
* **The refusal surfaces are exact, not approximate.** Every error number and substitution I checked
  against `RexxErrorCodes.h`/`RexxErrorMessages.h` is right: 88.901
  `Missing argument; argument &1 is required.`, 88.909 `Argument &1 must have a string value.`, 88.914
  `Argument &1 must be an instance of the &2 class.`, 93.902
  `Too many arguments in invocation of method; &1 expected.`, 93.915
  `Method option must be one of "&1"; found "&2".` The named-versus-numbered substitution split
  (`stringArgument`'s two overloads, `runtime/MethodArguments.hpp:136` and `:161`) is modelled as two
  constructors rather than smeared into one.
* **`makeString`'s argument order matches the C++ body statement for statement**: option first
  (`optionalOptionArgument(format, 'L', ARG_ONE)`), option validity next, then the `C`-plus-separator
  maxarg refusal, then the separator's own string check inside the `L` arm only. I read
  `ArrayClass::toString` and the order is the oracle's.
* **`ARRAY_DEFAULT_NAME` is the right distinction, not a guess.** Measured: `say a + 1` reports
  `Object "an Array" does not understand message "+".` while `a~objectName`, `a~defaultName` and
  `a~string` are all `an Array`, and a string context joins the items instead. Both halves of that
  pair are pinned by corpus programs that sit next to each other.
* **The gap discipline holds under pressure.** Every array method this task did not implement refuses
  loudly and specifically -- `ITEMS`, `[]`, `APPEND`, `TOSTRING` each name themselves at rc 120 --
  rather than answering from the wrong place.
* **The corrected `eval.rs` comment is now correct.** Measured: `say (.Object~superClasses = '')` and
  the `==` form are both `0`, and `(.Array~package == .String~package)` is `1`.
* **Two de-enumerations went the right way**: `ObjectModel`'s field doc stopped listing its three
  classes, and `try_text`'s doc stopped naming the number of causes of `None`.

## Issues

### Critical

None.

### Important

**1. `crates/rexx-exec/src/dispatch.rs:1397` cites a C++ function that does not exist.**
`/// `Package~name`: the package's own name -- `PackageClass::getName`.` There is no
`PackageClass::getName`. The method is `PackageClass::getProgramName`
(`classes/PackageClass.hpp:147`), bound as `AddMethod("Name", PackageClass::getProgramName, 0)` at
`memory/Setup.cpp:1189`; grepping `*.cpp` and `*.hpp` for `PackageClass::getName` returns nothing.
This is the sixth instance on this task of the defect the task self-reported five of, and the one
citation in the diff that is not a line number is the one that is wrong -- a name with no line number
looks unfalsifiable and so does not get re-read. Fix: name `PackageClass::getProgramName` and give it
the `memory/Setup.cpp:1189` binding or the `classes/PackageClass.hpp:147` definition. Every other C++
citation in the diff is correct; the full list I read is at the end of this file.

**2. Plan-mandated: the brief's "Done when" cannot be met by this task, and the narrowing paragraph
carves out the wrong set.** The criterion is "the wiring rows for the classes this crate registers read
`agree`". Eleven registered classes cannot reach `agree` until the prologue runs, because their
`~superClasses` gains a `CoreClasses.orx` `~inherit` target, and `native_classes.rs:81-102` assigned
exactly that to Task 13 before this task started. The brief's own narrowing names only
`Queue`/`Stem`/`VariableReference`. So the criterion was unsatisfiable when it was written, and the
implementer is right to report it unmet rather than to have tried to close it. The controller should
amend the criterion -- either to "the classes this crate registers *and whose superclass list the
prologue does not extend*", or by moving the wiring-row criterion to Task 13 -- and should not read
this as an incomplete task. I verified the whole chain: 14/11/38 reproduced, the eleven differ on
exactly one line, that line's missing entries are exactly the `~inherit` targets at the cited
`CoreClasses.orx` lines, and the ownership predates this task.

### Minor

**3. `crates/rexx-exec/src/dispatch.rs:1930` enumerates a set the code decides, and the enumeration
is now wrong.** "The rows are the receiver kinds a program can put on the left of a send: a string,
`.nil`, an array and a package object." A class object is one too (the same test's own subject), and
`~method` -- added by this very task -- puts a **method object** there as well. A universal claim over
a set the code can enumerate, of the shape that has cost this plan repeated fix rounds. Fix: say what
the rows are (the non-class-object receiver kinds these rows cover) without claiming they exhaust the
set.

**4. A method object answers nothing, and the report does not say so.** `~method` makes a new receiver
kind reachable and `receiver_kind` has no arm for it, so it falls into
`Body::Native(_) => Err("one of the interpreter's own objects")`. Measured:
`say .Array~method('APPEND')~class` is rc 120 loud here and `The Method class` on the oracle;
`.Array~method('APPEND')~identityHash` likewise. Loud, so licensed, but the report's "What I could not
close" lists `~instanceMethod`, `~methods`, `~subClasses` and `~request` and omits this, which is the
one the task itself created. Note also the latent divergence behind it: the oracle answers the *same*
method object on repeated sends (`==` is `1`) where `native_method` mints a fresh
`Body::Native` each call, so whichever task makes a method object a receiver has to fix identity as
well as the methods. Fix: add the row to the report and to the ledger.

**5. `crates/rexx-exec/src/value.rs:669` says "the only caller" and there are three.**
`array_string_of` is called from `text_len` (`:601`), `to_text` (`:729`) and `to_number` (`:1047`),
and the third reaches it after matching `Body::Array(_)` directly rather than through `Redirect`. The
safety argument survives -- every caller has just seen the body -- but the sentence is false as
written. Fix: "every caller has just matched this handle's body as `Body::Array`".

**6. The report's mandated coverage list is incomplete.** The brief says the named list of classes the
`~method` programs cover "is the extent of the protection", and the report's list gives `K` as `OWN`
from a `::METHOD` and `SIDE` from a `::METHOD ... CLASS`. The second commit added an `::ATTRIBUTE`
pair and an `::ATTRIBUTE ... CLASS` pair, and the program's rows 11-14 ask them: measured, `A` and
`A=` answer and `B` and `B=` raise. The commit message mentions them; the report's list, and the
report's own row table, stop at row 10. Fix: extend both to rows 11-14.

**7. `corpus/lang/class_method_own_dictionary.rex:12` enumerates the covered classes in a comment.**
The rows below it are the list, and the report carries the list as the brief requires, so the comment
is the third copy and the one nothing checks. Borderline against the constraint rather than a clear
breach -- the code that decides membership is nine lines below -- but it is the shape to prefer
pointing at.

**8. `corpus/phase-5a.txt:288` has a comment line reading `# one`.** A wrap artifact mid-sentence; the
sentence reads correctly but the line does not. Cosmetic.

**9. The "not an inlining threshold" control is weaker than stated.** `#[inline]` is a hint that for a
private function in the same codegen unit changes little; `#[inline(always)]` is the attribute that
could have produced a different number. The conclusion may well be right, but the experiment as run
cannot distinguish "inlining is not the mechanism" from "the attribute did nothing".

**10. `MAKESTRING` answers and `TOSTRING` does not**, though `memory/Setup.cpp:733`-`:734` bind both
names to the same `ArrayClass::toString` at the same arity and the doc comment cites both. Measured,
`.Class~superClasses~toString` is `The Object class` on the oracle and rc 120 loud here. Correct scope
discipline, and the refusal is loud, so this is an observation rather than a defect -- but it is a
one-row gap sitting beside a row that landed.

**11. The report's "it is real work, not noise" for the `strings` residual is asserted, not shown.**
`bench-programs/strings.rex` builds no array and sends no message, so the `Body::Array(_) => None`
arm's *body* never executes on that axis; whatever the arm costs has to be in `try_text`'s enum
dispatch, on every one of the millions of calls the axis makes. That is a coherent mechanism and
probably the right one, but the report does not state it, and the three axes reading 1.000000 cannot
witness it either way because none of them calls `try_text`. Fix: say the mechanism, or say it is
unexplained.

## The C++ citations in this diff, all of them, read

Every added line in the diff carrying a C++/`.orx` reference, checked at the cited line:

| citation | verdict |
|---|---|
| `classes/ArrayClass.cpp:1841` | correct -- `ArrayClass::makeString()`, body forwards to `toString(OREF_NULL, OREF_NULL)` |
| `classes/ArrayClass.cpp:1856` | correct -- `ArrayClass::toString(RexxString*, RexxString*)` |
| `classes/ClassClass.cpp:984` | correct -- `MethodClass *RexxClass::method(RexxString*)` |
| `classes/ClassClass.cpp:987` | correct -- `stringArgument(method_name, "method name")->upper()` |
| `classes/ClassClass.cpp:385` | correct -- `RexxClass::getId()` |
| `classes/ClassClass.cpp:419` | correct -- `RexxClass::getMetaClass()` |
| `classes/ClassClass.cpp:441` | correct -- `RexxClass::getSuperClass()` |
| `classes/ClassClass.cpp:458` | correct -- `RexxClass::getSuperClasses()`, body is `superClasses->copy()` |
| `classes/ClassClass.cpp:1692` | correct -- `RexxClass::isSubclassOf`, asks `isCompatibleWith` on the receiver |
| `classes/ClassClass.cpp:1732` | correct -- `RexxClass::getPackage()` |
| `classes/ObjectClass.cpp:286` | correct -- `RexxObject::isInstanceOfRexx`, asks `isInstanceOf` after `classArgument` |
| `classes/ObjectClass.cpp:1157` | correct -- `RexxObject::stringValue()`, sends `GlobalNames::OBJECTNAME` |
| `classes/ObjectClass.cpp:1760` | correct -- `RexxObject::defaultName()`, and the enhanced arm is where the comment says |
| `runtime/MethodArguments.hpp:727` | correct -- `inline void classArgument(RexxObject*, RexxClass*, const char*)` |
| `memory/Setup.cpp:733`-`:734` | correct -- `MakeString` and `ToString`, both `ArrayClass::toString`, arity 2 |
| `runtime/MethodArguments.hpp` (no line, `stringArgument` overloads) | correct -- `:136` and `:161`, differing in exactly that substitution |
| `CoreClasses.orx:93`, `:97` | correct -- `.string~inherit(.Comparable)`, `.array~inherit(.OrderedCollection)` |
| **`PackageClass::getName`** (dispatch.rs:1397) | **wrong -- no such function; it is `PackageClass::getProgramName`** |

The five citations the task self-reported as guessed (`ClassClass.cpp:596/1626/1707/1721/1932`) do not
appear anywhere in the diff, and the five replacements (`385/419/441/458/1732`) are all correct. The
report's fourteen `CoreClasses.orx` line citations are all correct too.

## Verdict on the +0.9685%

**A pass, not a finding dressed as one.** The guard is "under 1% on an axis is not a finding", the
loudest `instructions:u` row is `strings`/ir at 1.009685, and I confirmed that number and every other
row of the report's table against `bench-baselines/phase-5a-arms.tsv` -- both sittings are in the
file, both labelled with the commit whose build they measured, and `compound`, `emptyloop` and
`varlookup` do read 1.000000 to six decimals on all four arms.

Three things qualify it. First, the change that got it there is real and in the right direction:
making `Redirect` `Copy` took `strings`/ir from 1.010616 -- over the threshold, and reported rather
than hidden -- to 1.009685, and `alloc4c`/ir from 1.002706 to 1.001203. Second, the array arm cannot
be taken off the string path while `try_text` stays total: dropping it sends a `Body::Array` into that
match's `unreachable!`, and folding it into a `_ => None` falsifies the documented meaning of `None`
for a `Body::Instance`. The report considered and rejected both, and I found no third shape. Third,
and this is the part the controller should carry forward: at 0.9685% the axis has 0.03 points of
headroom, the mechanism is one arm in a function the axis calls millions of times, and the report's
own narrowing implies the next task that adds an arm to `to_text`, `text_len` or `try_text` crosses the
guard. That is worth recording in the ledger as a standing constraint on the value model, not just as
this task's number.

The "it is the change and not the layout" conclusion is right about attribution -- adding and removing
the line moves the figure reproducibly to six decimals -- but the report never says what the work
actually is, and the mechanism it implies ("real work") is not the arm's body, which never runs on
that axis. See Minor #11.

## Assessment

**Task quality: Approved.**

The behaviour is right where it is hardest to be right: 31 `~method` names beyond the corpus probed
across both directions, including the two the oracle deletes after `Setup.cpp` installs them, and zero
divergence; the `~class`/`~metaClass` split correct on parting shapes the corpus does not even cover;
all eight new corpus programs byte-identical on three descriptors on both engines; the headline
14/11/38 reproduced independently, and its shortfall correctly diagnosed as another task's. The one
Important code finding is a wrong C++ function name in a doc comment, and the other is a defect in
the brief rather than in the work.
