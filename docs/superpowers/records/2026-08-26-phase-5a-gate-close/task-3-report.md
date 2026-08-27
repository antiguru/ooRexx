# Task 3: `Class~subclass` and `Class~mixinClass`

**Base** `637d0dd64`. Branch `plan/rust-rewrite`.

**Commits.** `7d04f72a9` (the implementation, the two corpus programs, the unit test, the three
comment corrections and `oracle-crashes.txt` entry 8) and `edc9e69c3` (the plan correction). The
five gates below ran on the tree those two commits contain: `git diff HEAD` is empty.

## What the row does now

`rust/corpus/gate-tables/concepts/usingcl.rex`, three descriptors read separately, oracle against
both engines:

```
$ ( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx <row> )
rc 0
mixin-base Object
subclass-of Array
superclasses The Array class The Persistence class
default-superclass Object
```

`REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker` are byte-identical to that on stdout, stderr and exit
status. Measured with a `cmp`-per-descriptor script (`scratchpad/t3probe/diff.sh`), which reads the
two crate runs in separate temporary directories and never merges descriptors.

Gate table C reads the row `agree loud=no`, printed by

```
$ REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c -- --nocapture
  agree          loud=no  5a   usingcl Using Classes  depth 1 parent xcremet provide.xml:407 usingcl.rex
```

Before the change the same row was `rc 120`, `rexx-exec: method "MIXINCLASS" of class "Class" is not
implemented (Phase 5)`, on both engines.

## The two decisions, measured

Both are directly observable from a Rexx program, so neither is a judgment call. I re-ran the
controller's addendum probes rather than taking them on trust, and both reproduce.

### Which package: **none**. `~package` answers `.nil`.

```
k = .object~subclass("k")
say 'pkgname' k~package~name
```
oracle **rc 159**, stdout empty, stderr
`Error 97.1:  Object "The NIL object" does not understand message "NAME".`

The C++ says why: `subclassRexx` and `mixinClassRexx` forward `OREF_NULL` as the package
(`classes/ClassClass.cpp:1546`, `:1496`) where `ClassDirective::install` forwards its own
(`instructions/ClassDirective.cpp:200`, `:205`), `subclass` stores it at `:1582`, and `getPackage`
is `resultOrNil(package)` (`:1735`).

**This was the one place the change could have shipped a silent wrong answer at rc 0.**
`Interp::package_object_for` read an *absence* from `class_packages` as "the REXX package", so a
class built at run time would have answered `The REXX Package`. The fix is a third state with one
spelling: `plan.rs`'s new `ClassPackage` enum, `Program(ProgramId)` or `Null`, with absence still
meaning REXX. The three answers side by side, oracle rc 0 and both engines byte-identical:

```
k = .object~subclass("k")
say 'pubK' .context~package~publicClasses['K']~id        ->  pubK K
say 'pub-runtime' .context~package~publicClasses['DUP']  ->  pub-runtime The NIL object
say 'pkg-nil' k~package                                  ->  pkg-nil The NIL object
say 'directive-pkg' .K~package~name                      ->  <the program's own path>
say 'array-pkg' .Array~package~name                      ->  array-pkg REXX
::CLASS K PUBLIC
```

A runtime-built class also enters **no** package class table. Oracle rc 0:
`.context~package~classes~items` is `0` both before and after `k = .object~subclass("k")`.
(That probe cannot be run against the crate -- `Package~classes` is an unrelated Phase 5 gap and
refuses loudly at rc 120 -- so the crate side is pinned through `~publicClasses` above instead.)

### Whether the `REXX_DEFINED` lock applies: **it does not**, in either direction.

```
k = .object~subclass("k")
k~inherit(.object~mixinclass("mx"))
say 'inherit-ok' k~superClasses~makeString('L', ' ')  ->  inherit-ok The Object class The mx class
```
oracle **rc 0**. The C++ is the reason on both halves: none of the five mutators' `isRexxDefined()`
tests is inside `RexxClass::subclass`, and the flag is written by `RexxClass::liveGeneral` at
image-save time (`:136`-`:142`) rather than by any constructor.

The opposite direction is pinned too, and it already held: `.array~inherit(.object)` is **rc 158**,
`Error 98.985:  User additions are not allowed to the REXX language classes.`, byte-identical on
oracle and both engines. `corpus/lang/class_rexx_defined_inherit.rex` is that half;
`corpus/lang/class_subclass_factory.rex` is this one.

**The `library_bootstrap` question is unobservable and I built no machinery for it.**
`Interp::install_class` sets `REXX_DEFINED` on a class installed while the interpreter's own library
is running. The factory could have done the same, but nothing can reach it:
`/bin/grep -in "~subclass\|~mixinclass" interpreter/RexxClasses/CoreClasses.orx
interpreter/RexxClasses/StreamClasses.orx interpreter/platform/unix/PlatformObjects.orx` matches
nothing, so no library source builds a class by message. A branch nothing can execute is a branch no
test can witness, so the cheaper arm is taken: the factory never sets the flag, which is also what
`RexxClass::subclass` does.

## What was built

`dispatch.rs` gains two `NATIVE_METHODS` rows at `Arity::Fixed(3)` -- the count `memory/Setup.cpp`
declares (`:470`, `:474`) -- and one factory behind both, because `RexxClass::mixinClass` is
`RexxClass::subclass` under the mixin flag (`:1519`) with the base class taken from the receiver
(`:1522`), and `rexx_classes::ClassGraph::define_class` already makes both of those from
`ClassKind::Mixin`. That function already had the whole graph: superclass wiring, the mixin base
class, the metaclass override for a class derived from a metaclass, and `parent_has_uninit`. The
factory is the order around it.

`class_factory` runs `RexxClass::subclass`'s own order, and each boundary in that order is
observable:

1. **the metaclass**, argument 2, defaulting to the receiver's own (`:1566`-`:1569`) and tested
   before anything else (`:1572`);
2. **a `NEW` send to the metaclass** (`:1579`), which is where the class id's own refusals live;
3. the class is built, and recorded as belonging to no package;
4. **the enhancing class methods**, argument 3, merged into the class method dictionary
   (`:1602`-`:1608`);
5. `checkUninit`, the `INIT` send (`:1628`, `:1631`), the parent-uninit propagation (`:1634`).

**The `NEW` send is modelled rather than made.** This crate implements no `Class~new` at all, so the
factory constructs the class where the oracle sends. A metaclass whose `NEW` resolves anywhere other
than `.Class` decides what gets built and this crate cannot build it, so it refuses **loudly**
rather than building the one `.Class` would have. That case is reachable and I measured it: with
`::CLASS MyMeta SUBCLASS Class` and a `::METHOD new CLASS` whose body is `forward class (super)`,
`.object~subclass("k", .MyMeta)~id` is oracle rc 0 answering `id k`, against
`rexx-exec: method "NEW" of class "MYMETA" is not implemented (Phase 5)` at rc 120 here. A body that
returns something which is *not* a class object crashes the oracle -- see below.

**The class id's refusals carry two traceback frames**, and that is the send, not the error. Because
the checks live inside `RexxClass::newRexx` (`:1786`), the oracle's report is `Compiled method "NEW"
with scope "Class".` above `Compiled method "SUBCLASS" with scope "Class".` above the sending
clause.
`class_id_argument` blames the `NEW` frame on every way of failing -- the shape `Interp::invoke` and
`Interp::blame_request` already use -- so a `makeString` that raises inside the conversion reports
`REQUEST`, `NEW`, `SUBCLASS`, which is what the oracle reports. All three shapes are byte-identical
on both engines.

Files touched:

* `rust/crates/rexx-exec/src/dispatch.rs` -- two table rows, `native_subclass`,
  `native_mixin_class_factory`, `class_factory`, `factory_metaclass`, `class_id_argument`,
  `required_class_id`, `enhance_class_methods`, `install_enhancing_methods`.
* `rust/crates/rexx-exec/src/plan.rs` -- the `ClassPackage` enum.
* `rust/crates/rexx-exec/src/lib.rs`, `environment.rs` -- `class_packages`' value type,
  `record_packageless_class`, and `package_object_for`'s third answer.
* `rust/corpus/lang/class_subclass_factory.rex`, `class_subclass_refusals.rex`, their
  `crates/rexx-parse/tests/sourceline_oracle/*.txt`, and their rows in `corpus/phase-5a.txt` and
  `EXPECTED_SUBSET_5A`.
* `rust/corpus/oracle-crashes.txt` -- entry 8, below.
* `rust/crates/rexx-exec/src/dispatch.rs`'s test module --
  `a_metaclass_with_its_own_new_is_loud`, below.

## An oracle crash, found while building this

`RexxClass::subclass` casts the answer of the `NEW` send without checking it and dereferences it on
the next statement:

```c++
RexxClass *new_class = (RexxClass *)meta_class->sendMessage(GlobalNames::NEW, class_id, p);
// ...
new_class->setPackage(package);            // ClassClass.cpp:1579, then :1582
```

A metaclass may carry its own `NEW` and nothing constrains what it returns, so

```rexx
say .object~subclass("k", .MyMeta)~id
::CLASS MyMeta SUBCLASS Class
::METHOD new CLASS
  return 'x'
```

is **SIGSEGV, rc 139, three runs of three**, both descriptors empty. The neighbour bounds it: the
same `::METHOD new CLASS` reached directly, `.MyMeta~new("k")~id`, is a clean
`Error 97.1: Object "x" does not understand message "ID".` at rc 159, so it is the cast inside
`subclass` and not the overridden `NEW`, and shortening the program to the direct send measures the
wrong thing.

Written up as `corpus/oracle-crashes.txt` entry 8, with the wrapper it was measured under and a
"do not run this to confirm it" note in the file's own house style. **No upstream ticket filed** --
that is Moritz's call, and this is one signal.

This crate cannot reach the shape: `factory_metaclass` refuses a metaclass whose `NEW` resolves
anywhere other than `.Class` before it builds anything.

## False or narrowed comments corrected

Each was found by running something in the neighbourhood of the change, not by rereading.

* **`dispatch.rs`'s `native_id`: "`::class Foo` makes `.Foo~id` `Foo`" is false.** Measured, oracle
  and `ir` agreeing: `::class Foo` makes it `FOO` and `::class "Foo"` makes it `Foo` -- an unquoted
  directive name is upcased with every other symbol before the id is taken. It matters here because
  `.object~subclass("Foo")~id` **is** `Foo`, so the two spellings now sit next to each other.
* **`error.rs`'s `bad_metaclass`: "No `Compiled method` frame, measured" is now half the story.**
  It was true of the route it was measured on and reads as a property of the error. The directive
  still carries none; the same 99.927 reached by `.object~subclass("k", .Object)` carries
  `Compiled method "SUBCLASS" with scope "Class".` The pair is what says the frame follows the
  route.
* **`class_graph.rs`'s `refresh_parent_has_uninit`: "its only caller" and "the directive install".**
  The directive is no longer the only route into `RexxClass::subclass`. The argument the doc makes still holds for both, so only the two clauses
  naming the caller changed.

## Probing past the row

Every task on this plan turns a loud refusal into an answer, so the question is not whether the row
agrees but whether anything now answers at rc 0 where it should not. The final run was over the row,
the two new corpus programs and every hand-written probe -- `echo $ALL | wc -w` -> **56** -- against
the oracle on both engines, three descriptors read separately with `cmp` per descriptor. The script
is `scratchpad/t3probe/diff.sh`, whose per-program loop runs the oracle under `ulimit -v 1048576`
and each engine under `timeout -s KILL 20`, each in its own fresh directory. (The unbounded-loop
probe under Concerns 6 is deliberately outside that set: it peaks at 4 GB and would perturb anything
running beside it.)

What the probes cover beyond the row: `~id` case, `~string`, `~baseClass` for both kinds and for a
mixin of a mixin, `~superClass`, `~superClasses`, `~metaClass`, `~class`, `~isA`, `~isSubclassOf`,
`~objectName=` on a class the factory built, the whole mutator set on one (`~inherit`, `~uninherit`,
`~define`, `~defineMethods`), `~method(name)~scope` on all three write routes, `~annotation`, an
inherited class-side `INIT` and one that raises, `~subclass` sent from inside a method body,
`~subclass` from inside an `INIT` that is itself building a class, the empty and mixed-case id, the
metaclass argument in every shape (omitted, `.nil`, a non-class, a non-metaclass class, an explicit
`.Class`, a runtime-built metaclass, and one a `::CLASS ... METACLASS` supplied), the enhancing
table, two classes built under the same id, a renamed class appearing in a refusal's own
substitution, `trace i` and `trace r` over the send, `INTERPRET` around it, and the argument
refusals.

**Eight diverge, each on both engines, and every one is a loud `rc 120` refusal -- never an
answer.** Six are pre-existing Phase 5 gaps this change does not touch; two are refusals this task
chose. (Three further probes hit `StringTable~items`, `==` on a class object and `condition('O')`,
also pre-existing gaps; I rewrote those to ask the same question another way and they then agree.
The `=` row below is the one I left asking, so that the gap is on the record.)

| probe | oracle | crate | whose |
| --- | --- | --- | --- |
| a metaclass with its own `::METHOD new CLASS` | rc 0 | `method "NEW" of class "MYMETA"` | this task |
| `.object~subclass("k", .Class, .environment)` | rc 163, 93.974 | `a directory whose entries this crate does not fill` | this task |
| `.context~package~classes` | rc 0 | `method "CLASSES" of class "Package"` | pre-existing |
| `forward class (super)` in a class-side `INIT` | rc 0 | `FORWARD is not implemented` | pre-existing |
| `.K~new` on a `::CLASS` | rc 0 | `method "NEW" of class "Object"` | pre-existing |
| `.methods['MM']` | rc 163, 93.924 | `method "[]" of class "String"` | pre-existing |
| `~queryMixinClass` | rc 0 | `method "QUERYMIXINCLASS" of class "Class"` | pre-existing |
| `(a = b)` on two class objects | rc 0 | `the operator \`=\` applied to a class object` | pre-existing |

The `.environment` row is `~defineMethods`' existing `Loud::unreadable_collection`, reached through
the new third argument: this crate models `.environment` as a subset of the oracle's, so walking it
would mutate from a different set rather than raise.

`~queryMixinClass` is the direct reader of the mixin flag this factory now sets, and it refuses for
a `::CLASS ... MIXINCLASS` exactly as it does for one built here, so the gap is the directive's as
much as the send's and is nobody's on this plan. The row it would have covered is covered instead by
`~baseClass`, which parts the two kinds and which both new corpus programs read.

## Controls

### The plan's named control, and what it actually reddens

The plan said "defaulting the superclass to something other than `.Object` reddens
`default-superclass`". **That is wrong about which line reddens, and I have corrected the plan file**
(`docs/superpowers/plans/2026-08-26-phase-5a-gate-close.md`, Task 3). `~subclass` has no superclass
default to change -- the superclass is the receiver -- so the mutation the phrase describes is
"build the class under `.Object` rather than under the receiver", which is also gate table C's own
registered control for this row.

Applied (`Some(class)` -> `Some(root)` in `class_factory`), rebuilt, and re-run against the tree as
committed after the two witness rows below were added. `REXX_ENGINE=ir` on the row:

```
mixin-base Object
subclass-of Object                                 <- reddens
superclasses The Object class The Persistence class <- reddens
default-superclass Object                          <- UNCHANGED
```

because the row asks that last line of `.object~subclass("plain")`, whose receiver already is
`.Object`. Gate table C reads the row `diverge-stdout`, and
`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` prints `259 of 261 matching`.

The mutation that *does* redden all four lines, `default-superclass` among them, is building the
class under its **metaclass** (`Some(class)` -> `Some(metaclass)`). `REXX_ENGINE=ir` on the row:

```
mixin-base Class
subclass-of Class
superclasses The Class class The Persistence class
default-superclass Class
```

### Inverted live

With the first control reverted from a scratchpad copy (never `git checkout --`) and rebuilt, the
row, `class_subclass_factory.rex` and `class_subclass_refusals.rex` are all byte-identical to the
oracle on both engines again -- `ALL AGREE (3 programs, both engines)` from `diff.sh`. So the
redness above was caused by the mutation and not by the harness.

### A control of my own: does the new corpus add coverage, or only fail?

A red mutation proves a test can fail. It does not prove the test catches anything the existing
suite would have missed. The candidate: **delete `interp.record_packageless_class(id);`** from
`class_factory`, which makes `~package` on a runtime-built class answer `The REXX Package`. The row
does not ask `~package`, so `usingcl` stays green under it.

| run | command | result |
| --- | --- | --- |
| mutation, new programs registered | `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | `260 of 261 matching`, `[UNCLASSIFIED] lang/class_subclass_factory.rex: stdout differ` |
| mutation, the two rows removed from `corpus/phase-5a.txt` | same command | **`259 of 259 matching`** |
| mutation, the two rows removed, whole gated release suite | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | one failure only: `phase_5a_subset_matches_the_committed_list`, which is my own control setup (the rows are still in `EXPECTED_SUBSET_5A`) and not a catch. Every other binary passes. |

So nothing that existed before this task catches the dropped package record;
`class_subclass_factory.rex` is the only instrument for it. Both mutations were reverted from a
scratchpad copy and the tree rebuilt before the gates below.

The same first control gives the same reading for the row's own subject:
`259 of 261 matching`, and the two failures named are **exactly** the two programs this task added --
so the pre-existing corpus does not catch the superclass mutation either.

### A second control of my own, on the branch no corpus row can reach

The metaclass `NEW` check is the one place where dropping code produces a **silent wrong answer at
rc 0** rather than a refusal: the crate would build the class `.Class` would have built and hand it
back as though the metaclass's own `NEW` had made it. No corpus row can carry that, because the
oracle answers the shape at rc 0 and this crate refuses it. So the instrument is a unit test,
`dispatch::tests::a_metaclass_with_its_own_new_is_loud`, with the answering row beside it (the same
metaclass with no `NEW` of its own builds and answers `id k`, matching the oracle).

Deleting the check and running `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast`:

```
exit 101
test dispatch::tests::a_metaclass_with_its_own_new_is_loud ... FAILED
test result: FAILED. 740 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
261 of 261 matching
```

**One failing test in the whole gated release run, and it is the new one.** The corpus is untouched
at `261 of 261`, both gate tables are untouched, and every other binary passes -- so the test does
not merely fail, it is the only thing that catches this. Restored from
`scratchpad/backup-t3/dispatch-final.rs` before the gates below.

## The corpus the task adds

Two programs, registered in `corpus/phase-5a.txt` and `EXPECTED_SUBSET_5A`, each with its
`crates/rexx-parse/tests/sourceline_oracle/<name>.txt` regenerated by the `.Package~new` driver that
test's module comment carries. Both agree with the oracle byte for byte on all three descriptors on
both engines, and both were written against the oracle first.

* **`corpus/lang/class_subclass_factory.rex`**, rc 0. What gets built and what it answers: the id's
  spelling, the rendering, `~baseClass` for both kinds, `~superClass`, `~superClasses`, `~metaClass`,
  `~class`, `~isA`, `~isSubclassOf`; the three package answers side by side; the mutator set on a
  class the factory just built, `~method(name)~scope` read back after `~define` and after
  `~defineMethods`; a chained `~subclass` and `~mixinClass`; the metaclass argument explicit and
  omitted, the omitted one read off a `::CLASS ... METACLASS` receiver so that "the receiver's own"
  and "always `.Class`" are different answers; the enhancing table, holding an `INIT` so that the
  merge's precedence over the `INIT` send is an answer and not an argument; and an inherited
  class-side `INIT` printing once for the `::CLASS` install and once for the class `~subclass` built
  under it.

  **Two of those rows are there because the file's own comment claimed them and nothing in the file
  showed them.** Every receiver in the first draft had `.Class` for a metaclass, so "the metaclass
  defaults to the receiver's own" was satisfied by a build that always used `.Class`; and the
  enhancing table held no `INIT`, so "merged before the `INIT` send" was satisfied by a build that
  merged after. Found by reading the comment against the output rather than against the code.
* **`corpus/lang/class_subclass_refusals.rex`**, rc 168. A trapped ladder over the argument
  refusals, in the shape `class_reflection_argument_ladder.rex` uses: 88.901, 88.909, 99.927 in four
  spellings, 93.902 for both messages, the answering rows that stop "refuse everything" from
  passing, and `~inherit`'s own 98.943 reached through a mixin the file built on `.Array`, which is
  what says the mixin flag and the base class took. It ends untrapped on the 88.901 so the frame
  pair -- `NEW` under `SUBCLASS` -- is compared as bytes.

`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` prints **261 of 261 matching**,
up from 259 at `637d0dd64`.

## The interaction with Task 2

`Method~scope` on a class the factory built answers the new class, through both write routes.
Measured, oracle rc 0 and both engines byte-identical:

```
k = .object~subclass("kk")
k~define('Y', .methods~z)
say 'scope' k~method('Y')~scope~id        ->  scope kk
k~defineMethods(.methods)
say 'dm-scope' k~method('Z')~scope~id     ->  dm-scope kk
mx = .object~mixinclass("mixy")
mx~define('W', .methods~z)
say 'mx-scope' mx~method('W')~scope~id    ->  mx-scope mixy
```

and `k~hasMethod('Y')` is `0` in both, because `~hasMethod` on a class object asks its class
behaviour and `~define` writes the instance side. The enhancing third argument is the mirror image:
`k~hasMethod('Z')` is `1` and `k~method('Z')` raises 97.1 under a `Compiled method "METHOD" with
scope "Class".` frame, byte-identical.

## What the plan file now says that it did not

`docs/superpowers/plans/2026-08-26-phase-5a-gate-close.md`, Task 3. Two edits, both from
measurement:

* **Added**, because the plan sized the task from the row and the row uses one of three arguments:
  the metaclass argument and its precedence over the class id, the enhancing class methods and their
  precedence over the `INIT` send, and the `NEW` send that owns the class id's refusals and their
  second traceback frame. Each with the probe that shows it.
* **Replaced**, the control sentence, for the reason under "Controls" above. The replacement says
  what the old mutation actually reddens and what mutation reddens the line the old sentence named,
  rather than restating the old sentence in different words.

The plan's "what is already there" paragraph was **not** wrong this time. `Class~baseClass`,
`Class~superClass` and `Array~makeString` do all answer at `637d0dd64`, and so does `~superClasses`,
which the controller's addendum had already added. The `~~inherit` twiddle the paragraph flags as
untested by the refusal was checked once the sends resolved: the row's third line is
`superclasses The Array class The Persistence class` and it agrees.

## Concerns

1. **`Class~queryMixinClass` is still a loud refusal**, and it is the one message that reads the
   mixin flag directly. It refused before this task for a `::CLASS ... MIXINCLASS` too, so the gap
   is the directive's as much as the send's and closing it is outside this task, but it means the
   flag's only differential witnesses here are `~baseClass` and `~inherit`'s 98.943. Both are in
   `class_subclass_factory.rex` and `class_subclass_refusals.rex`.
2. **A metaclass with its own `NEW` refuses loudly where the oracle answers rc 0.** Deliberate: the
   oracle builds the class object by sending `NEW`, this crate implements no `NEW`, and answering
   would mean building the class `.Class` would have built and calling it the user's. It is `rc 120`
   at `NOT_IMPLEMENTED_EXIT`, never a 97.1 and never a wrong answer at rc 0.
   `dispatch::tests::a_metaclass_with_its_own_new_is_loud` is the only instrument for it, and the
   second control above is what says so.
3. **`.object~subclass("k", .Class, .environment)` refuses loudly where the oracle raises 93.974.**
   Also deliberate, and it is `~defineMethods`' existing behaviour reached through a new door.
4. **`cargo clippy` ran with a warm target directory.** The run that followed the last Rust edit
   reported `Checking rexx-exec ... Finished in 1.89s`, so it did re-lint the crate this task
   changed; the final rerun in the gate table below is fully cached at 0.07s, which is honest --
   nothing but `.rex` corpus files and their `sourceline_oracle` expectations changed after it, and
   clippy lints neither. It did not re-lint the crates this task did not touch. `rust/CLAUDE.md`
   asks for a clean target directory at a *phase* boundary, which this is not.
5. **The session scratchpad is shared between teammates.** `scratchpad/gates/` already held another
   agent's `g1.log`/`g3.log`/`run.sh` when I got there, so my own gate run writes to
   `scratchpad/task3-gates/` instead. The figures below come from that directory.
6. **A class this crate builds is never freed, and `~subclass` is the first unbounded route to
   building one.** A `::CLASS` directive is bounded by the source text; a send in a loop is not, and
   a class identity lives outside the arena so the collector never reaches it. Measured with
   `/usr/bin/time -v` on `do i = 1 to 200000; k = .object~subclass("k"); end`, both sides printing
   `built k` at rc 0:

   | | peak RSS | elapsed |
   | --- | --- | --- |
   | `REXX_ENGINE=ir` | **4,085,904 kB** | 0:06.78 |
   | oracle under `ulimit -v 4194304` | **109,100 kB** | 1:02.21 |

   About 20 kB per class here, because `ClassGraph::define_class` allocates two behaviours and
   cascades the ancestor dictionary into each. The oracle collects the unreachable ones. The
   *answers* agree byte for byte on all three descriptors; only the memory does not, which is the
   licensed OOM-divergence axis rather than a wrong answer, and no corpus program is anywhere near
   it. Recorded rather than fixed: pruning the registry needs a collector that reaches class
   identities, which is not this task.

## Gates

All five run from `rust/` on the tree as committed, each status read unpiped out of its own file
(`scratchpad/task3-gates/g<N>.status`, written by `scratchpad/task3-gates/run.sh`).

| | command | exit |
| --- | --- | --- |
| G1 | `cargo fmt --all --check` | **0** |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| G3 | `cargo test --release --workspace --no-fail-fast` | **0** |
| G4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | **0** |
| G5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0** |

`command -v memcap` answers `/home/moritz/.local/bin/memcap`, so the `ulimit` substitute was not
needed. `memcap` is on G5 alone, which is `rust/CLAUDE.md`'s placement and Task 2's measurement:
`memcap 8G cargo test --release` dies at exit 137 during compilation.

Figures, each beside the command that printed it:

* `/bin/grep -ac 'FAILED' <log>` -> **0** for each of G3, G4 and G5, and `/bin/grep -ac '^---- '
  <log>` -> **0** for each, which is the failure-block count rather than the word.
* `/bin/grep -a 'of 261 matching' <log>` -> **`261 of 261 matching`** for both G4 and G5. The same
  program set without this task's two rows printed **`259 of 259 matching`** -- measured, in the
  coverage control above -- so the two added programs are the whole difference.
* G2's own output is one `Finished` line and no `Checking` line, because nothing Rust changed after
  the run that produced `Checking rexx-exec ... in 1.89s`. See Concerns 4.

**One figure that is not stable and should not be quoted: the count of `test result: ok` lines.**
It read 102 on an earlier G3 run of this same command during this task and 103 on the final one,
and only one of those is my new unit test
(`740 passed` -> `741 passed` in `rexx-exec`'s lib). The other came from two parallel targets
interleaving on one pipe -- the log has a bare `s` on its own line where a `running 0 tests` was
split -- so the count measures the scheduler, not the suite. Both runs list the same 91 test
binaries and the same 10 doc-test targets.
