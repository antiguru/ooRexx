# Task 23 review: the bootstrap milestone

Range `5b054d4b5..HEAD` (`87cf62507`), five commits, both attempts.

**Spec compliance: PASS** against the brief's "Done when".
**Task quality: CHANGES REQUIRED.**

Findings: **2 major**, **2 moderate**, **7 minor**.

Most serious: an error raised inside a library method prints the interpreter's own source
text and the user program's path where the oracle prints a source-less frame naming the
method, its scope and package `REXX` -- a refusal turned into a wrong answer, on a surface
no gate and no sentence in the tree covers.

Everything below was run. Probes came from freshly created empty directories with absolute
paths, three descriptors read separately, both engines. Every `file:line` was re-read with
`/bin/grep -n` (not `sed -n`, which strips leading blank lines and gave me one wrong
line number I caught and discarded).

---

## What I did not repeat

The controller's own verification: `fmt` clean, `clippy` exit 0, `REXX_CORPUS_GATE=1 cargo
test --release --workspace` exit 0 with 246 of 246 matching, and the hand-run five checks.
I did not rebuild the crate. Two checks below use test binaries already in `target/` and
say so.

---

# Major findings

## M1. An error raised inside a library method diverges on stderr, and nothing records it

**The isolated case**, both engines, rc and stdout identical, stderr differing in exactly
two lines:

```
say .Validate~number('LENGTH', 'abc')
```

```
oracle rc=168   ir rc=168   tw rc=168
--- oracle stderr                     +++ crate stderr (both engines)
-  3700 *-*       Method NUMBER with scope "Validate" in package "REXX" (no source available).
+  3700 *-*       raise syntax 88.902 array(name, number)
-Error 88 running REXX line 3700:  Invalid argument.
+Error 88 running /abs/path/v.rex line 3700:  Invalid argument.
```

The 88.902 line itself matches byte for byte. What diverges is the traceback frame and the
program name in the error line.

**The oracle's mechanism, verified in the C++.** An image-saved package carries no source,
so `PackageClass.cpp:589` calls `RexxActivation::formatSourcelessTraceLine`
(`RexxActivation.hpp:243`, body at `RexxActivation.cpp:5057`), which renders
`Message_Translations_sourceless_method_invocation`
(`messages/RexxErrorMessages.h:745`, `messages/rexxmsg.xml:6525`):
`Method &1 with scope "&2" in package "&3" (no source available).` This crate keeps the
library source, so it renders the clause instead and attributes the frame to the running
program's file.

**It is a refusal that became a wrong answer.** Measured against
`bench-baselines/pinned/rexx-run-15a1ffa98`: before this task `.Validate` was an unbuilt
`.environment` name and every one of these programs was a loud rc-120 refusal. The global
constraints single this class out -- "the corpus gate cannot see a clean refusal becoming a
wrong answer ... and that defect class survived five fix rounds on the old plan's Task 8" --
and require the task that changes what is refused to say what instrument catches a
regression. Nothing in the report, the plan or the source mentions it: `/bin/grep -n` for
`no source available` over `docs/superpowers/plans/2026-08-17-phase-5a.md` and `rust/crates`
finds nothing.

**Breadth, measured.** I extracted every `::METHOD ... CLASS` pair from the two embedded
`.orx` files (73 pairs) and called each with no arguments, three descriptors, from a fresh
directory: **7 match, 14 are loud, 52 diverge on all three descriptors.** All 52 carry the
frame defect.

**Half of those 52 is not this task's, and I checked.** The rc/condition half --
40.3/40.4 `Incorrect call to routine` where the oracle raises 93.901/93.902 `Incorrect call
to method` -- is a pre-existing `USE STRICT ARG` defect. It reproduces on the pinned pre-5a
binary with a purely user-declared class method:

```
say .K~tag(1)
::class K
::method tag class
  use strict arg
  return 'k'
```

`rexx-run-15a1ffa98` gives `Error 40.4 ... maximum expected is 0` at rc 216 where the
oracle gives 93.902 at rc 163. So Task 23 did not introduce it -- but it did make 52 new
programs reach it, and the `.Validate~number` case above shows the frame half standing
alone with the condition half correct.

**What I would want.** The frame defect is the task's and should be named as a divergence
class with its own instrument, the way the `StringTable` order divergence is. Whether to
close it (mark an embedded library package source-less and render
`Method X with scope "S" in package "REXX" (no source available).`, and answer `REXX` as the
program name for a frame in one) or to record it is the controller's call, not mine. What
is not acceptable as it stands is that neither the report nor the tree says it exists.

## M2. The `InheritInstanceMethods` disambiguation is documented backwards in three places

The *code* is right. I verified the C++ distinction the report claims to have found, and it
holds exactly as stated:

* `memory/Setup.cpp:375` -- `#define InheritInstanceMethods(source)
  currentInstanceBehaviour->inheritInstanceMethods(The##source##Behaviour);` reaches
  `RexxBehaviour::inheritInstanceMethods` (`behaviour/RexxBehaviour.cpp:350`), whose body is
  `methodDictionary->replaceMethods(source->getMethodDictionary(),
  source->getOwningClass(), getOwningClass())` -- `MethodDictionary.cpp:251` -- which copies
  and **leaves the donor alone**.
* The Rexx-callable method is bound at `Setup.cpp:461` as
  `RexxClass::inheritInstanceMethodsRexx` (`CPPCode.cpp:708`, body at `ClassClass.cpp:599`),
  which calls `RexxClass::inheritInstanceMethods` (`ClassClass.cpp:558`), whose second line
  is `sourceMethods->setMethodScope(this)` on the **donor's own dictionary**.

And the call sites are the right way round: `native_classes.rs:280` (the `Setup.cpp` replay)
uses `donate_instance_methods`; `dispatch.rs:3625` (the Rexx method) uses
`inherit_instance_methods`.

The documentation is the defect, in three places, all of them describing the fix as though
it had not been made.

**(a) `crates/rexx-classes/src/class_graph.rs:853` -- an orphaned doc block.** Inserting
`donate_instance_methods` above `inherit_instance_methods` reassigned the existing doc
comment. The block that begins

```
/// `inheritInstanceMethods`. Oracle's `RexxClass::inheritInstanceMethods`
/// (`:558-586`) takes `donor`'s own instance-method dictionary **by
/// pointer and rewrites it in place** ...
```

and continues

```
/// This crate does the identical in-place rewrite, not a clone: `donor`
/// really is left with its own instance methods bearing `class`'s scope
/// afterward ...
```

now sits on `donate_instance_methods`, which clones the donor's dictionary and does not
touch it -- the new text is appended to the same block with no separator and says so two
paragraphs later ("**leaving the donor alone**"). One doc block, two contradictory halves,
on the item whose whole point is the distinction. `inherit_instance_methods` is left with no
doc comment at all. This is the insertion-orphans-a-doc-block shape; `fmt` and `clippy` both
pass over it.

**(b) `crates/rexx-classes/src/registry.rs:571` -- three false sentences.**
`ClassRegistry::inherit_instance_methods`'s doc still reads:

* "This crate has no reason to build a second donation mechanism" -- this task built one.
* "The two mechanisms therefore agree on the resulting **name set** ... even though they
  disagree on which class a donated name's scope is attributed to (invisible to those
  probes, and **not queryable through this crate's own public API either**)" -- the task's
  own measurement falsifies it: `.array~at('x')` reported `Compiled method "AT" with scope
  "Queue".`, and two corpus rows caught it.
* "`native_classes.rs`'s replay loop calls this for `Table`, `StringTable`, `Set`,
  `Directory`, `Relation` and `Bag`" -- the replay loop calls `donate_instance_methods`.

**(c) `crates/rexx-classes/src/native_classes.rs:110` -- the module doc states the opposite
of the code.** "`InheritInstanceMethods(source)` operations replay through
[`crate::ClassRegistry::inherit_instance_methods`] (`RexxClass::inheritInstanceMethods`'s
donate-by-own-dictionary semantics) **rather than** `RexxBehaviour::inheritInstanceMethods`'s
bootstrap-only donate-by-flattened-behaviour semantics ... -- see
[`crate::ClassRegistry::inherit_instance_methods`]'s own doc comment for why the two agree
on every name this task's probes check." Every clause of that is now wrong, and it forwards
the reader to (b).

---

# Moderate findings

## D1. `environment.rs:1422` states a reason this change falsified

```rust
// A program's own package. This phase loads one program, so its
// path is the running program's.
Package::Program(_) => self.program_path.clone().into_bytes(),
```

Every run now loads four programs. `package_name` discards the `ProgramId`, so a package
object for `CoreClasses.orx`, `StreamClasses.orx` or `PlatformObjects.orx` would answer the
*user program's* path. `Interp::bootstrap_library`'s own doc names the same risk and argues
it is unreachable ("Nothing hands it out -- a program's own `.context~package` is its own"),
and `record_package_class`'s new `library_programs` skip is what keeps `.Alarm~package~name`
answering `REXX` (verified: byte-identical on both engines). So the behaviour is right; the
comment that a reader would rely on says the opposite of what is true.

## D2. `rexx-lib`'s recursion argument rests on a false measurement, behind a case-sensitive check

`crates/rexx-lib/src/lib.rs`, in
`the_entry_program_calls_exactly_the_other_embedded_programs`:

> **The second half is what makes the run-time recursion guard unnecessary rather than
> forgotten**: ... Measured, neither non-entry file contains the word `call` at all.

False. `/bin/grep -in call` on `StreamClasses.orx` answers 2 -- `:363` ("By calling lineout
directly") and `:534` ("additional system calls"), both inside comments. The property the
sentence supports (no embedded file calls another, so `enter_library_program` needs no depth
guard) does hold today, but the stated evidence does not.

The assertion that would catch a change is narrower than the claim: `quoted_call_targets`
matches only `trimmed.strip_prefix("call ")` followed by `'`, so an upstream `CALL
'CoreClasses.orx'` in a non-entry file leaves the test green. Rexx is case-insensitive, and
the windows `PlatformObjects.orx` -- which this task correctly declines to embed -- contains
exactly that shape (`  call 'orexxole.cls'`), so the day someone embeds it the guard is the
thing that has to hold.

---

# Minor findings

## m1. The perf control isolates the bootstrap, not the collector

Checked against `bench-baselines/phase-5a-arms.tsv`. Every figure in the report's two tables
reproduces exactly from the TSV (`per_pass`, `instructions:u`, `head`): emptyloop ir
373.00 -> 373.00, varlookup 866.00 -> 865.99, arith 23,776.23 -> 24,629.05 (+3.59%),
dispatchclass 6,472.37 -> 6,873.53 (+6.20%), compound 1,929.48 -> 2,135.47 (+10.68%),
alloc4c 3,606.49 -> 4,067.06 (+12.77%), strings 5,433.48 -> 7,136.86 (+31.35%), and the tw
column likewise. The `fixed` rows are as quoted (emptyloop ir 629,425 -> 148,993,113;
strings ir -26,044 -> 148,162,236), and I confirmed the headline independently with `perf
stat -e instructions:u`, three runs each: `say 1` is 149.26M / 149.05M / 148.92M at HEAD
against 585,658 / 585,654 / 585,383 on `rexx-run-15a1ffa98`. +148.4M per run, and every
share in the cost table divides correctly.

Two things the control does not do that the prose says it does.

* **"Suppressing the bootstrap puts every axis back on its pre-task figure to within a
  rounding step."** For emptyloop and alloc4c, yes (373.000017 and 3,606.495076 against
  373.00 and 3,606.494954). For strings it is 5,406.48 against 5,433.48, **-0.50%**. That is
  under the guard's 1% line and does not change the conclusion, but 27 instructions a pass
  is not a rounding step and the report's own table prints both numbers.
* **"So this is the collector."** The control separates *the bootstrap ran* from *the new
  code exists*. It does not separate *the collector traces a bigger live set* from any other
  consequence of a larger resident heap -- a more expensive allocation path, a bigger root
  set walked per allocation. Both explanations predict exactly the observed pattern (flat on
  the two axes that allocate nothing, growing with how much each of the others allocates),
  and no collection count was read. The observable is sound and the concern is the right one
  to raise for Task 24; the mechanism is an inference presented as an attribution.

The control ran 3 of 8 axes, which the report states.

## m2. Two `.orx` citations point at the wrong line

* `dispatch.rs`, `native_inherit_instance_methods`: 'the "phony inherit" `CoreClasses.orx:78`
  names in its own comment'. `:78` is `-- dictionary.`; the phrase is on `:77`.
* `lib.rs`'s `library_bootstrap` field doc and `dispatch.rs`'s `rexx_defined_lock` both say
  "`CoreClasses.orx:88` onwards is a run of `~inherit` clauses". `:88` is blank; the section
  comment is `:89` and the first `~inherit` is `:93` (`.string~inherit(.Comparable)`).

Every other citation I checked holds, including the ones easiest to get wrong:
`RexxBehaviour.cpp:350`, `ClassClass.cpp:558`, `ClassClass.cpp:883`,
`MethodDictionary.cpp:251`, `ObjectClass.cpp:1950`, `ObjectClass.hpp:340`,
`ClassClass.cpp:136`-`:142` (`liveGeneral`'s `PREPARINGIMAGE` / `setRexxDefined`),
`ExpressionMessage.cpp:155`-`:181` (`:181` is exactly `// evaluate the arguments first`, so
the "both checks run before the arguments" claim lands on the right line),
`DoBlockComponents.cpp:233`-`:236`, `StringClass.cpp:1765`, `Setup.cpp:681`,
`HashContents.cpp:489`, `StringClass.hpp:328`, `CoreClasses.orx:47`/`:52`/`:55`/`:63`/`:73`/
`:99`/`:110`/`:122`/`:124`/`:990`, `StreamClasses.orx:45`/`:506`.

## m3. `directive_class`'s doc misstates the C++ order it names

`environment.rs`: "`PackageClass::findClass`'s order, which is `.NAME`'s own minus the
reflection names: the running package's installed classes, then `.environment`, then the
native name table." `PackageClass::findClass` (`PackageClass.cpp:1081`) is installed classes
-> imported public classes -> `TheRexxPackage->findPublicClass` -> package local -> the
directories. The crate implements three of those and skips the two public-class steps.

Unreachable today: I built a two-file probe (`::requires 'dep.rex'` with a public
`::class Comparable` in the dependency, and `::class K subclass Comparable` in the main
file) and the crate is loud -- `rexx-exec: ::REQUIRES is not implemented (Phase 5)` -- where
the oracle resolves to the imported class. So no program can see the difference. The
sentence still states the C++ order incorrectly, and the module doc a few hundred lines
above gets it right ("The steps between those that this crate has nothing to consult are
named rather than skipped silently").

## m4. Report concern 3 cites a comment that does not exist

Both attempts say `Interp::run_activation`'s own comment described the plan-key assertion as
"unpinned by anything else today because production loads one program, so every `ProgramId`
is 0". `/bin/grep -rn` over `crates/` for `production loads one program`, `one program`, and
`unpinned` finds no such comment; the sentence is in
`docs/superpowers/records/2026-08-09-phase-4e-ir/task-10-report.md:198`. The substance holds
-- `Plan::line_at`'s `debug_assert_eq!` (`plan.rs:415`) is now exercised across four
`ProgramId`s per run, and the debug gate is green -- but the citation is to the wrong
artifact.

## m5. The before-figure the report says it could not take was available

The report admits "I did not take either figure before this task". One was reachable without
a rebuild: `target/release/deps/gate_table_c-88c89472c7b48c4f`, built 03:42, between the base
commit (03:03) and the task's first code commit (03:50), and gate table C's probes are
unchanged by this task. Running it under `REXX_CORPUS_GATE=1`:

| | class wiring | hierarchy edges | 5a rows not yet `agree` |
|---|---|---|---|
| before (03:42 binary) | 15 agree, 35 diverge-both, **13 diverge-stdout** | 57 diverge-both | 110 of 135 |
| HEAD | **62 agree, 1 diverge-both** | 57 diverge-both | **61 of 135** |

So the task's real delta is +47 `agree` and the elimination of all 13 silent-wrong-answer
rows. Both of the report's own figures reproduce exactly at HEAD, and I confirmed all 57
hierarchy rows end at `rexx-exec: method "HASITEM" of class "Array" is not implemented
(Phase 5)` (57 of 57 by count, one distinct crate stderr in the block) with each printing
its `child` and `parent` lines first. The one class-wiring diverge is `RexxInfo`, as stated.
Caveat on provenance: that binary's exact source state is inferred from its mtime, so treat
the before-column as approximate.

## m6. `crates/rexx-exec/Cargo.toml` gained a stray blank line before `[lints]`.

## m7. `assertions.rs`'s rewritten `EXEMPT` doc uses "now that" ("their prelude line answers
now that the library bootstrap installs `.String~tab`"). Mild historical framing; strike it
and the sentence says the same thing. Noted only because the same doc correctly removed a
set-size count ("All 35 are") in the same edit.

---

# What I verified and found correct

## The five checks and the bootstrapped state, swept far past the corpus

Zero divergences across every sweep below (oracle vs both engines, three descriptors, fresh
empty directory):

* **`.environment` membership.** All 69 names in `ORACLE_ENVIRONMENT`, one program per name
  (`say .NAME`): **67 identical, 2 loud** -- `ENDOFLINE` and `REXXINFO`, exactly as the
  report claims, and both `rexx-exec: environment symbol ... is not implemented (Phase 5)`
  rather than a wrong answer.
* **Class wiring.** 62 classes x 11 questions (`~id`, `~class`, `~metaClass`,
  `~package~name`, `~superClasses`, `~baseClass`, `~isSubclassOf(.Object)`, three
  `~hasMethod`, `~method('INIT')`): 0 divergences, and no engine split.
* **Instance method dictionaries.** For each of 24 classes I read its full method-name list
  off the oracle (`~methods` supplier) and asked `c~method(name)` for every name -- about
  1,400 lookups: 0 divergences. The same list through `c~hasMethod(name)`: 0 divergences.
* **Class-side method sets.** All 73 `::METHOD ... CLASS` pairs extracted from the two `.orx`
  files, plus five natives, through `~hasMethod`: 0 divergences.
* **Native x library interaction.** 227 rows of `isSubclassOf` / `isA` over
  {Object, Class, String, Array, Queue, Stem, Table, StringTable, Directory, Set, Bag,
  Relation, Supplier, Comparable, Orderable, Collection, MapCollection, OrderedCollection,
  SetCollection, Stream, File, Alarm, Monitor, Message, List} x
  {Object, Comparable, Orderable, Collection, MapCollection, OrderedCollection,
  SetCollection, Class}, plus `~isA` for a string and an array instance: 0 divergences.
* **The 16 `.String` class constants** (`~nl`, `~cr`, `~tab`, `~null`, `~alnum` ... `~xdigit`),
  compared as bytes: identical.
* **`String~UPPER`.** 22 boundary rows in one program (omitted args, 0, -1, 1.5, `'x'`,
  past the end, zero length, negative length, empty receiver, three arguments, a number
  receiver, an `e9`x byte, `.nil`, `1e2`, `' 2 '`, and both sides of `ARGUMENT_DIGITS`):
  identical, and `'abcdef'~upper('0.0')` really does report the unconverted `found "0.0"`
  as the doc says.
* **The locks stay closed for a program.** `.string~inherit(.Comparable)` is 98.985 at rc
  158 with the `Compiled method "INHERIT" with scope "Class".` frame;
  `.Alarm~package~addClass('ZZ', .Object)` is 98.984 at rc 158; `.String~defineClassMethod`
  and `.Supplier~inheritInstanceMethods` are gone. All byte-identical.
* **The `CALL` interception is bootstrap-only.** `call 'CoreClasses.orx'` and `call
  CORECLASSES` are 43.1 at rc 213, byte-identical.
* **No leakage into the program.** `digits()`/`fuzz()`/`form()`/`trace()`/`address()`/`arg()`
  at the first clause, `sourceline()` and `sourceline(1)`, and the settings after a library
  call: all identical. `parse source` unchanged. A user `::class Comparable` still shadows
  the library's, and `~package~name` for it is the file path.
* **Traps.** `signal on syntax` around a library raise gives `trapped 88 2` on both sides;
  `trace r` across a library call echoes only the program's own clause. Only the *untrapped*
  traceback diverges (M1).

## The two licensed divergences, and their bounds

**(a) `StringTable` order.** Real and observable by an ordinary program, as the report says.
Measured:

```
do e over .methods ; say e ; end     ::method zz ::method aa ::method mm ::method qq ::method bb
oracle: BB AA ZZ QQ MM       crate: AA BB MM QQ ZZ
```

The bound holds: **membership matches everywhere I could reach it.** `.methods` with five
unattached methods answers 5 on both; `.routines`, `.resources` and `p~publicClasses` answer
the same names and counts on both; library classes do not leak into a user package's
`~publicClasses`. `hash_collection_indexes` has exactly one caller (`over_items`), which has
exactly one caller shape (`DO OVER`), and `is_hash_collection` tests class identity against
`StringTable` alone, so no other route reaches the sorted order. Every other `StringTable`
accessor a program could use (`p~classes`, `~publicRoutines`, `~makeArray`) is loud.
`corpus/lang/do_over_string_table.rex` is genuinely order-independent -- it sorts its own
concatenation with a hand-written `::routine sorted`, counts passes, and uses `FOR 2` and
`LEAVE`, none of which can see the order. `corpus/README.md` now carries the prohibition,
and it is the right home for it.

**(b) `Directory` keeps the rc-120 refusal, justified on membership.** Both halves verified:
`do e over .local` iterates **10** entries on the oracle, and all ten
`ORACLE_LOCAL` names are loud here (`Phase 7`), so the crate's `.local` is empty; `do e over
.environment` iterates **69** on the oracle against the 67 the crate answers. So iterating
either would be a wrong *count*, not just a wrong order. `.environment~class` and
`.local~class` are `The Directory class` on both sides, and
`eval.rs`'s `an_object_as_a_do_over_target_is_loud` now carries the adjacent `StringTable`
success -- which is the in-crate instrument the global constraints ask for when a refusal
moves.

## Build inputs and pins

All three sha256 pins in `crates/rexx-lib/build.rs` match `sha256sum` on both the read-only
tree and this worktree's own tracked copies, which are byte-identical to each other. The
windows `PlatformObjects.orx` is neither read nor embedded and the crate says so -- correct,
and it is the file that would break D2's recursion argument. `interpreter/` copies unchanged
(`git status` clean at review time and at HEAD).

## Comment rules

Zero non-ASCII bytes in the added lines of the whole diff
(`git diff | /bin/grep -a '^+' | /bin/grep -acP '[^\x00-\x7F]'` is 0), so no em-dashes and no
smart quotes. No set-cardinality violations beyond small enumerated counts the tree already
uses as house style ("the two methods `Setup.cpp` puts on `.Class`"), and one such count was
*removed* by this task ("All 35 are" -> "Every row here is"). One mild historical framing
(m7).

## Committed tables that moved

* `assertions.rs`'s `EXEMPT` lost 22 rows and the pass count goes 4224 -> 4246 of 4259;
  22 is exactly the difference, and the reason given (the `tab = .String~tab` prelude now
  answers) is the right one.
* `collect_stress.rs`'s `NO_ALLOCATION_PROGRAMS` gained four and lost four. The list is
  asserted in both directions at the use site, so neither direction can be a convenience
  edit; the removed entries took their explanatory comment with them and the surviving
  neighbour's comment was correctly re-singularised. The report's admission that it did not
  measure *why* the four departures now allocate stands as an honest, bounded gap -- the
  likely cause is that `directive_class` now reaches `directory_lookup`, but I did not
  measure it either and am not asserting it.

---

# Verdicts

**Spec compliance: PASS.** The brief's "Done when" is met and I checked each clause. The
driver leaves stdout and stderr empty and exits 0 on both engines; all five checks answer
the oracle's bytes and are committed as corpus rows; the `ACTIVATE` control was run and
inverted in attempt 1 (the attempt-2 brief forbade re-running it) and attempt 2 ran five
controls of its own with the informative pair identified as such; the sitting exists in
`phase-5a-arms.tsv` under `23-attempt-2` with a named control arm, and every figure in the
report reproduces from it. The windows `PlatformObjects.orx` is named as unread. Plan line
58 was corrected.

**Task quality: CHANGES REQUIRED**, on M1 and M2.

M1 because a task whose whole subject is "what state every program starts in" introduced a
divergence class reachable by an ordinary program, turned 52 loud refusals into
three-descriptor wrong answers on the way, and left no sentence anywhere saying so -- while
the same report carefully records two smaller divergences it did license. The fix may be
"record it and name the instrument"; it may be "mark the embedded packages source-less". It
is not "leave it undiscovered".

M2 because the report's headline discovery -- two C++ functions with one name -- is now
documented backwards at all three sites a reader would consult, including one where the same
doc block asserts both halves of the contradiction. The code is right, which is what makes
this survivable; it is also what makes it exactly the shape that comes back.

D1 and D2 are one-line prose corrections with real reasons behind them. The minors are
citation and framing repairs.

The rest of the work is strong. Every differential sweep listed above found no wrong answer
outside M1, the two licensed divergences are
bounded where they are claimed to be bounded, the C++ reading behind the fix is accurate at
every line I checked, and the measured before/after on gate table C (15 -> 62 `agree`, 13
silent wrong answers eliminated) is a larger result than the report claims for itself.
