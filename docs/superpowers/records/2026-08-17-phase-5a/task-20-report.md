# Task 20 report: `::ANNOTATE`'s six targets, and the readback

**Status after fix round 1: DONE_WITH_CONCERNS.** All six targets install and **all six readbacks are delivered**,
which is more than the brief's split and more than the controller's ruling asked for. Nothing was
parked with 5b or with Task 21. Three things a reader should carry away before the detail: I
destroyed one source file mid-task with a careless `git checkout --` and rebuilt it; the `PACKAGE`
readback turned out to be reachable here, which falsifies a premise the brief and the notes share;
and this task shipped two silent wrong answers that no gate saw -- one I found before reporting, one
the review found after (F5, the annotation tables' allocation order), both fixed.

BASE `7130b222e`. Commits, oldest first:

| SHA | what |
| --- | --- |
| `d6b2a5394` | the install and the readback, `rexx-core` and `rexx-exec` `src/` |
| `0d0a7391d` | three corpus programs and their registrations |
| `6808a1901` | the record corrections, `phase-4-exclusions.txt` and `gate_table_d.rs` |
| `b4ded3f66` | the sitting, both arms, eight axes |
| `301842eaa` | one `Method` object per dictionary entry, fixing a divergence this task made reachable |
| `36b8992ca` | the sitting re-run at the commit that ships |
| `196da092d` | the two live-plan rows this task's measuring falsified |
| `3d025bcdf` | **fix round 1**: allocate the annotation tables in a fixed order (F5), with F4/F6/F8 in those two files |
| `7e253fae1` | **fix round 1**: F1, F2, F3, F6, F7 -- the prose the late fix falsified, the corpus header, the cardinalities |
| `949c374f8` | **fix round 1**: the sitting, both arms, eight axes, at `7e253fae1` |

---

## The open question, answered: contained, and it was built

The controller asked one thing: **can `~annotation`/`~annotations` be answered on `Method` and
`Routine` objects by the same mechanism that already answers `.methods~z` and `.routines~r`?**

**Yes, by the arms `.Package` already has.** `dispatch.rs` decides a receiver's class in
`receiver_kind`, which already had a `Body::Native` arm per native class -- `Package`, `Directory`,
`StringTable` -- each testing the object's stored class handle against the object model's. A `Method`
object and a `Routine` object are `Body::Native` too, so each needed **one such arm, one
`receiver_behaviour` fold, and one `NATIVE_METHODS` row per method**. That is the identical shape,
not a generalisation of it: `Body::Instance(_)` still answers `Err("an instance of a user class")`
and no general instance-receiver arm was added.

What it cost, beyond those arms:

* **`rexx-core`: one field.** `NativeObject` gained `annotations: Option<ObjRef>`, traced in
  `Body::trace`. A `Method` object cannot be asked which directive it came from, so it carries a
  handle on its table instead.
* **`rexx-exec`: one `Interp` field and one key enum.** `annotations: HashMap<Annotated, ObjRef>`,
  with `environment::Annotated` naming a package, a class object, one entry of a class's method
  dictionary, one entry of the file's unattached table, or a `::ROUTINE` directive.
* **Two `Primitive` variants**, which made every exhaustive match over it a compile error until it
  said what a `Method` and a `Routine` answer -- which is how `~class`, `~objectName` and
  `~objectName=` got decided rather than inherited.

**It turned one readback into six, not five.** The `PACKAGE` readback is reachable here today.

## The two premises that were still standing, and the third

The controller's notes killed the brief's two premises. A third one, in **both** documents, is also
false:

> the only route is `.context~package~annotation("AUTHOR")`, and the Package object and the
> `RexxContext~package` accessor that reaches it are both Task 21's

**`.K~package` is the other route, and it works here.** `Class~PACKAGE` has had a `NATIVE_METHODS`
row since the reflection protocol landed, and the package object has been a receiver since
`Package~NAME`. Measured at BASE, three descriptors:

```
say .K~package~annotation('AUTHOR')     with ::class K + ::annotate package author 'moritz'
  oracle   rc 0   moritz
  crate    rc 120 rexx-exec: method "ANNOTATION" of class "Package" is not implemented (Phase 5)
```

That is a `NATIVE_METHODS` gap, not a missing receiver -- two rows away, in the same table as the
other six. So the `PACKAGE` target's readback is **this task's and is delivered**; Task 21 still owns
`RexxContext~package`, which is the route `.context~package` needs and which is still rc 120 here.

## What the oracle does, measured

Every row below is a probe from a fresh empty directory, absolute paths, three descriptors read
separately, both sides bounded, and both `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`. Every one
of them now matches byte for byte on all three descriptors.

### The six readbacks

| target | readback | answer |
| --- | --- | --- |
| `CLASS` | `.K~annotation("AUTHOR")` | the value |
| `METHOD` | `.K~method("M")~annotation("AUTHOR")` | the value |
| `ATTRIBUTE` | `.K~method("A")~...` and `.K~method("A=")~...` | the value, both halves |
| `CONSTANT` | `.K~method("C")~annotation("AUTHOR")` | the value |
| `ROUTINE` | `.routines["R"]~...` and `.routines~r~...` | the value, both spellings |
| `PACKAGE` | `.K~package~annotation("AUTHOR")` | the value |

and the unattached table: `.methods~m~annotation("X")` for a `::METHOD` ahead of the first `::CLASS`.

### The name lookup and the absent name

`~annotation` **upcases** the name it is given -- `getAnnotation` reads `annotations->entry(name)`
and `StringHashCollection::entry` upcases the index (`classes/support/HashCollection.cpp:824`).
Measured, oracle rc 0: `.K~annotation("AUTHOR")` and `.K~annotation("author")` both answer `moritz`.
A name no `::ANNOTATE` recorded is `The NIL object`, which is `resultOrNil` in each of the three Rexx
stubs.

### The table is live, and that is the part a snapshot fails

`RexxClass::getAnnotations` (`classes/ClassClass.cpp:325`) and `BaseExecutable::getAnnotations`
(`execution/BaseExecutable.cpp:378`) create the table on the first ask and **store it**. Measured,
oracle rc 0 in all three shapes:

```
.K~annotations~put('v','N')            then .K~annotation('N')            -> v
.K~method('M')~annotations~put('v','N') then .K~method('M')~annotation('N') -> v
.routines~r~annotations~put('v','N')   then .routines~r~annotation('N')   -> v
```

The method row is the sharp one: the oracle's `~method` retrieves one object out of the dictionary
and answers it every time, so an addition survives to the next `~method` send. **A build answering a
fresh table per ask answers `The NIL object` on all three.** That is why the design stores one
`StringTable` per annotated thing and hands out a handle on it rather than a copy.

The control is in the same corpus program: `.K~method('P')~annotation('EXTRA')` after an addition to
`M`'s table is `The NIL object` on both sides.

### The refusal, and the resolution order behind it

`::ANNOTATE target &1 "&2" not found.` (`messages/RexxErrorMessages.h:715`), 99.945 at rc 157 with
stdout empty and the `::ANNOTATE` clause echoed. Measured for all five keywords that take a name:
`class`, `routine`, `method`, `attribute`, `constant`.

**The target is resolved against the part of the file already walked, not against the file.**
Measured, oracle rc 157 with 99.945:

* `::ANNOTATE CLASS K` standing **above** `::CLASS K`;
* `::ANNOTATE METHOD m` under `::CLASS B` when `m` was declared under `::CLASS A`.

**Two targets test the claimant's kind, not just its name.** Measured, oracle rc 157: `::ANNOTATE
ATTRIBUTE a` where `a` is a plain `::METHOD a`, and `::ANNOTATE CONSTANT c` where `c` is a plain
`::METHOD c`. Those are `isAttribute()` (`parser/DirectiveParser.cpp:2164`) and `isConstant()`
(`:2081`).

**The instance dictionary is searched before the class one.** Measured, oracle rc 0: a class carrying
both `::METHOD m CLASS` and `::METHOD m`, with `::ANNOTATE METHOD m` under them, annotates the
instance one. `::ANNOTATE METHOD` naming an attribute's getter annotates the getter alone --
`~method("A=")~annotation("X")` is `The NIL object` -- where `::ANNOTATE ATTRIBUTE` annotates both
halves. A class-side `::ATTRIBUTE a class` is reachable, which is
`processAttributeAnnotations`' second lookup.

**Accumulation.** A second `::ANNOTATE` of one target adds to the first's pairs, and a repeated name
takes the later value. Measured, oracle rc 0: `::annotate class K a 1 b 2` then `::annotate class K a
3` leaves `A` at `3` and `B` at `2`.

### Where in the install the class's annotations become visible

Measured, oracle rc 0, and this crate matches: a class-side `::METHOD init` reading
`self~annotation("A")` prints `The NIL object`, and a class-side `::METHOD activate` doing the same
prints the value. That is `ClassDirective::install`'s own order -- the class is built (and sent
`INIT`) at `:200`/`:205`, `setAnnotations` is at `:243`, and `ACTIVATE` is a later pass
(`classes/PackageClass.cpp:1299`). A `::CONSTANT` expression can read one too:
`::constant c (.K~annotation('A'))` answers the value on both sides.

## What this closed that the records had as standing costs

Six shapes that `phase-4-exclusions.txt` and `staged_gap`'s doc carried as losses now match the
oracle byte for byte, both engines:

```
::routine r / ::annotate routine r / ::class a subclass zzznotaclass   98.909 rc 158
::class a / ::constant kk (1/0) / ::routine r / ::annotate routine r   42.3   rc 214
::routine r / ::annotate routine r / a duplicate ::ROUTINE pair        99.903 rc 157
::routine r / ::annotate routine r / a class-less ::constant (1/0)     99.906 rc 157
::annotate routine nosuchrtn above a duplicate ::ROUTINE pair          99.945 rc 157
::annotate routine nosuchrtn above a class-less ::constant (1/0)       99.945 rc 157
```

The first four were the "THE COST IS A REFUSAL WHEREVER THE ORACLE WOULD HAVE CARRIED ON" and "WHAT
R33 COST" rows; the last two were the R33 fold's own refusals.

## The three carriers of the generalisation

* **The superseded spec** -- already superseded, nothing to do.
* **The old plan's Task 4** -- superseded by the brief, nothing to do.
* **`rexx-exec/src/lib.rs`** -- the arm the brief cites at `:1285`-`:1287` was at `:1449`-`:1455` at
  BASE (`directive_gap`'s `DirectiveKind::Annotate(annotate) if !Package` arm, with the
  `::annotate routine nosuchrtn is 99.945 rc 157` sentence above it). **The refusal is retired, not
  re-commented**: the arm is gone, and the measurement it carried now lives at
  `Raised::missing_annotation_target`, beside the behaviour it is evidence for.

`docs/superpowers/plans/phase-4-exclusions.txt` needed more than the sibling row the brief asked for,
because retiring the refusal falsified the sentences around it:

* the run-by-the-oracle list gained the sibling row -- `::annotate class|routine|method|attribute|
  constant <name>, target above it`;
* the refused-here-loudly list lost `::annotate routine nosuchrtn`, which is no longer refused here;
* the over-refusal block's **title** named `::ANNOTATE` and no longer does;
* the cost block lost its three `::annotate` rows;
* the two R33 blocks say what those shapes answer now;
* a `CLOSED DEFECTS` entry records the six targets, what the R33 shapes answer, and the one shape
  that is **still** refused.

`gate_table_d.rs`'s ownership doc said the `ROUTINE` row's readback was 5c's. It is 5a's, and the
corpus carries it.

## A divergence this task made reachable, found late and fixed

**The first build answered a fresh `Method` object per `~method` send, and that is observable
through two of the oracle's own methods.** Measured, oracle rc 0:

```
(.K~method("M")~identityHash = .K~method("M")~identityHash)   1     first build: 0
.K~method("M")~objectName = "x" then say .K~method("M")       x     first build: a Method
```

Both are **rc 0 with wrong stdout**, not refusals, and no gate saw either: the corpus was 210 of 210
over the build that produced them. I found them while checking a sentence I had already written into
this report, which had called the identity difference unobservable.

`RexxClass::method` retrieves the method out of `instanceMethodDictionary` and answers it
(`classes/ClassClass.cpp:984`, the retrieval at `:991`), so the oracle answers one object per
dictionary entry. `Class~method` now does the same -- `Interp::method_object`, cached by
`(class, name)` and rooted the way a package object is -- and `~objectName=` stores on a `Method` and
a `Routine` instead of refusing. Both rows above match now, and so does the control
(`say .K~method("P")` after renaming `M` prints `a Method` on both sides).

**What is still divergent, and is licensed.** `~identityHash`'s own *values* differ: this crate
answers the handle where the oracle derives its answer from an address (`native_identity_hash`
carries deviation 4's licence). One consequence is now reachable that was not before:
`(.K~method("M")~identityHash = .K~method("P")~identityHash)` is `1` on the oracle and `0` here,
because the oracle's two addresses agree at `NUMERIC DIGITS 9` -- measured, its raw values are
`-140064606165617` and `-140064606168321`. The same shape was already reachable at BASE on a class
object, whose `~identityHash` worked there. No corpus program prints an `identityHash`.

## What each claim rests on, and what each instrument could not see

**The corpus** -- three programs, all three differential against the live oracle:

* `corpus/lang/directive_annotate_targets.rex` -- the six targets and their handles, the upcased
  lookup, the absent name, `~annotations`' rendering, and `~class` on a `Method` and a `Routine`.
* `corpus/lang/directive_annotate_table_is_live.rex` -- the additions through `~annotations`, on a
  class and on a method, with a second method as the control.
* `corpus/lang/directive_annotate_missing_target.rex` -- 99.945 for a target declared **below** its
  `::ANNOTATE`.

**What the corpus cannot see, and what covers it instead.** A program refuses once, so five keywords
plus two kind filters need seven programs. `run/tests.rs`'s
`an_annotate_target_the_package_does_not_hold_is_the_oracles_own_refusal` carries the six the corpus
does not, on both engines. That is a deliberate cost trade, not a limitation: each of those refusals
**is** expressible as a corpus row, because this crate and the oracle agree on it.
`an_annotate_target_resolves_the_way_the_directive_walk_accumulates` covers the resolution rules a
single readback program cannot separate, and
`a_class_answers_one_method_object_per_dictionary_entry` is the instrument for the identity property
the late fix is about -- its `~identityHash` half cannot be a corpus row at all, and its
`~objectName=` half is asserted beside it rather than as a program of its own.

**Mutation checks, each run against the whole gated suite with `--no-fail-fast`** -- so the answer is
"what caught it", not "did something catch it":

| mutation | caught by |
| --- | --- |
| drop the `isAttribute()`/`isConstant()` filter | the new refusal test **only** |
| search the class dictionary before the instance one | the new resolution test **only** |
| rebuild the annotation table per ask instead of keeping it | the new resolution test **and** the corpus, `208 of 210` |

The first two are the "can fail is not adds coverage" check: nothing else in the suite, corpus
included, sees either. The third says both new corpus programs earn their place.

**A GC defect the stress suite caught, and nothing else would have.** The first build allocated the
annotation table inside `native_method` while the `Method` object it was about to attach it to was
unrooted. `collect_stress`'s `the_l0_subset_passes_again_under_collect_on_every_allocation` panicked
at `dispatch.rs`'s `a live value` tripwire; the release corpus gate was green over the same tree. The
fix routes the allocation through `Interp::native_instance`, which pushes the object as a temporary
first.

**What the gates could not have seen.** The corpus gate cannot see a clean refusal becoming a wrong
answer, which is why the `~objectName=` row is an in-crate test and says so. `cargo doc --workspace
--no-deps` was run and reports no unresolved intra-doc link in `rexx-exec` or `rexx-core`; it is not
a gate, so that is a one-time check rather than a standing one.

## Gates

Run from `rust/`, each read unpiped:

```
cargo fmt --all --check                                     exit 0
cargo clippy --workspace --all-targets -- -D warnings        exit 0
REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast   exit 0, 210 of 210 matching
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast   exit 0, 210 of 210 matching
```

The corpus was 207 of 207 at BASE and is 210 of 210, the three added programs being the difference.

Gate table D, under `REXX_CORPUS_GATE=1`: `5a: 36 rows, 2 not yet agree`, and the two are
`::ATTRIBUTE EXTERNAL` and `::METHOD EXTERNAL`, which Phase 7 owns. No `::ANNOTATE` row is `loud` any
more.

## The sitting

Pin checks first: `15a1ffa98` is an ancestor of `HEAD`; the pinned binary's sha256 is
`141c3fa968ea774655bc00f3e3b9f61cf1c4b738fd2793831d0055e94f1d074b`, matching `PINNED.md`; and
`git log --oneline 15a1ffa98..HEAD -- rust/crates rust/Cargo.toml rust/Cargo.lock Cargo.toml` lists
only this phase's own task commits, which the ledger records. The command was checked against the
plan's guard block (`docs/superpowers/plans/2026-08-17-phase-5a.md:311`) and agrees with
`global-constraints.md`'s copy: eight axes, five rounds.

**Both arms, `instructions:u`, `--rounds 5`, and both re-run at the commit that ships**
(`301842eaa`) after the `Method`-object fix landed, because a guard measured against a tree nobody
ships is not a guard. `b4ded3f66` holds the first pair, at `6808a1901`; `36b8992ca` holds these.

Accumulated, against the phase-start pin (`head` over `pinned`, so above 1 is slower):

| axis | tw | ir |
| --- | --- | --- |
| alloc4c | 1.00377 | 1.00481 |
| arith | 0.99714 | 0.98974 |
| compound | 1.00697 | 1.01047 |
| emptyloop | 0.99543 | 0.99202 |
| strings | 1.00918 | 1.01341 |
| varlookup | 0.99716 | 0.99426 |
| dispatchclass | 1.01537 | 1.01743 |
| rexxcps | 1.01659 | 1.02031 |

**Contribution, interleaved, against BASE `7130b222e` built from a temporary worktree** -- this is
the arm that decides:

| axis | tw | ir |
| --- | --- | --- |
| alloc4c, arith, compound, emptyloop, strings, varlookup | 1.000000 | 1.000000 |
| dispatchclass, small | 0.999090 | 0.999077 |
| dispatchclass, large | 0.999082 | 1.001081 |
| rexxcps | 1.000011 | 0.999984 |

Six axes are bit-identical on every cell; `dispatchclass` retires **0.09% fewer** instructions than
BASE on three of its four cells and 0.11% more on the fourth, and `rexxcps` is within 0.002% of 1.
**Nothing is at or above 1%, so there is nothing to attribute to the change or to the layout.** The
four axes that read above 1% against the pin -- `dispatchclass`, `rexxcps`, `strings`, `compound` --
read the same against it at Task 19, so they are inherited and not this task's; the six
bit-identical rows are the control that says so, because machine drift would have moved them too.

**What the contribution arm cannot see**: no bench program sends `~method`, checked --
`/bin/grep -rn "method(" bench-programs/*.rex bench-rexxcps/*.rex` matches nothing -- so the caching
this task added is not on any axis's executed path, and these figures say only that its *layout*
cost nothing.

`cycles:u` is not quoted as a result, per the plan. The rows are in
`bench-baselines/phase-5a-arms.tsv` under task 20.

## Fix round 1

Ten findings, `.superpowers/sdd/2026-08-17-phase-5a/task-20-review.md`. All ten are addressed except
F9 and F10, which are the controller's.

**F5 was the only behavioural one and it was real.** The staging map
[`Interp::install_directives`] drains to allocate one annotation table per target was a std
`HashMap`, so the tables landed at per-process random arena indices and `~identityHash` answers the
handle. Measured on the shipped binary, one program printing six annotation tables' `~identityHash`,
twelve runs: **three distinct outputs**; the same program with the `::ANNOTATE` directives removed
was identical on all twelve. Both that map and `attach_directive_annotations`' per-name map are
`BTreeMap`s now, `AnnotatedSite` deriving `Ord`, and the same sweep is **one output over twelve on
each engine**.

**The control is two runs of one program in one process**, `run/tests.rs`'s
`two_runs_in_one_process_allocate_the_annotation_tables_alike`. That shape is what sees it: `std`
seeds each `HashMap` from a thread-local counter, so a second map in a thread iterates differently
from the first -- which is why every in-process test stayed green while the binary varied per run.
Verified live rather than reasoned: it fails against the `HashMap` and passes against the
`BTreeMap`. The program annotates a target of every shape a separate table is built for.

**F1, F2 -- my own late fix falsifying prose it did not sweep**, and the controller is right that
this is the shape to sit with rather than the individual sentences. `301842eaa` cached one `Method`
object per dictionary entry and updated `dispatch.rs`'s neighbouring doc, and left the two sentences
*inside* `environment.rs` that were the justification for keeping and rooting the annotation table,
and the whole `WHAT REMAINS REFUSED` block in the records file this task was sent to correct. All
three are rewritten from a re-measurement: `~objectName=` now stores through a **second** fetch on a
`Method`, a `Routine`, an unattached `Method` and a `Package`, byte-identical to the oracle on both
engines, with `.K~method("P")` unrenamed as the control. What genuinely remains refused is every
name in either dictionary with no `NATIVE_METHODS` row, and `~identityHash`'s licensed values.

**F3, F4, F6, F7, F8** -- the corpus header's count and its miscounted set, three C++ citations
re-pinned and each checked against the path its own example takes (`isAttribute()` at
`DirectiveParser.cpp:2140`, the instance getter's check, which is what a plain `::METHOD` target
trips; `processAttributeAnnotations` at its definition `:2131`; `createMethod`'s `true` at `:1775`;
and `processAnnotation`'s `put` at `:2259`), the four set cardinalities named by member instead,
`coverage.rs`'s doc restated as the property its assertion checks rather than a count of a match's
arms, and `staged_gap`'s "no longer" struck.

**Fix round 1's sitting**, at `7e253fae1`, both arms, eight axes, five rounds. Contribution against
BASE: six axes `1.000000` on every cell, `dispatchclass` 0.99907 to 0.99909 across its four (the
cell that read 1.001081 before the ordering change is now 0.999071), `rexxcps` within 0.001% of 1.
Accumulated against the pin matches `301842eaa` to four decimal places on every axis.

**Gates re-run at `949c374f8`**: `cargo fmt --all --check` exit 0; `cargo clippy --workspace
--all-targets -- -D warnings` exit 0; `REXX_CORPUS_GATE=1 cargo test --release --workspace
--no-fail-fast` exit 0, `210 of 210 matching`; the debug gate exit 0, `210 of 210 matching`.

## Concerns

1. **Two silent wrong answers shipped from this task, and neither was found by an instrument.** The
   first, `~method` handing out a fresh object per send, I found by re-measuring a sentence I had
   already written into this report. The second, F5's allocation order, the review found by asking
   whether the binary was deterministic at all -- a question no test in this workspace asks. The
   pattern is that both were **rc 0 with wrong output over a tree the corpus called `210 of 210`**,
   and in both cases the reason nothing was flaky was that no program in the corpus happens to print
   the value. `two_runs_in_one_process_allocate_the_annotation_tables_alike` closes F5's half; the
   general question -- what else in this crate answers from an arena index or a hash order -- is
   open and larger than this task.

2. **I destroyed `crates/rexx-exec/src/environment.rs` mid-task and rebuilt it from scratch.** After
   a mutation experiment I ran `git checkout -- crates/rexx-exec/src/environment.rs` to undo the
   mutation, which reverted the file to `HEAD` and took roughly two hundred lines of this task's own
   work with it. I reconstructed it from the edits in my own transcript; the gated suite, the
   differential probes and the corpus are green over the reconstruction, and the file was reviewed
   line by line afterwards -- but **the reconstruction has no independent witness that it is
   identical to what was lost**, only that it is correct. This is the hazard the project's own record
   already names ("restore from a copy rather than from git"); the copy existed for `lib.rs` and not
   for this file. Nothing else in the tree was touched by the mistake.

3. **The `PACKAGE` premise.** Both the brief and the controller's notes state that the only route to
   the package object is `.context~package`, and both assign the `PACKAGE` readback elsewhere on that
   basis. `.K~package` is the second route and works here. I delivered the readback rather than
   parking it, because the brief's Build list is "all six targets and `~annotation`/`~annotations` on
   the objects that carry them" and the package object is one of them.

   I corrected the **two forward-looking** places in the live plan myself (`196da092d`), because a
   wrong handover row is what the next task reads: the reading table's `::ANNOTATE` row, and 5c's
   handover list, which carried the `ROUTINE` readback. **I did not rewrite Task 20's own brief
   section**, whose ownership-split paragraphs are now historical rather than forward-looking, and
   **Task 21's brief is yours to check** -- what it still owns is `RexxContext~package`, not the
   `PACKAGE` annotation readback, and its "Done when" may name the latter.

4. **A scope widening, stated so it is a decision and not a discovery.** Making `Method` and
   `Routine` receivers turns every name in those classes' dictionaries into a resolvable message.
   Names with a `NATIVE_METHODS` row answer -- `~class`, `~string`, `~isA`, `~hasMethod`,
   `~objectName`, `~isNil`, `~identityHash`, `~init` -- and every other name is now a **loud gap
   naming Phase 5** where it used to be the blanket "a message send to one of the interpreter's own
   objects". That is D37's rule applied as `.Package` already applies it, but it is a wider surface
   than "the readback", and I did not probe every name in either dictionary against the oracle.

5. **How the first of the two wrong answers was found.** `~method`
   answering a fresh object per send made `(.K~method("M")~identityHash =
   .K~method("M")~identityHash)` answer `0` where the oracle answers `1`, and
   `.K~method("M")~objectName = "x"` forget the name -- both rc 0 with wrong stdout, over a tree the
   corpus called `210 of 210`. `301842eaa` fixes it. **What found it was re-measuring a sentence I
   had already written into this report**, not any instrument: the report said the identity
   difference was unobservable, and checking that claim before shipping it is what produced the
   probe. The general lesson is the project's own -- a summary of a measurement is a new claim and
   needs the measurement re-run -- and it applies to a *design* claim as much as to a prose one.

6. **What remains divergent on `~identityHash`, and is licensed.** The values themselves differ, and
   one consequence is newly reachable: `(.K~method("M")~identityHash =
   .K~method("P")~identityHash)` is `1` on the oracle and `0` here, because the oracle's two
   addresses agree at `NUMERIC DIGITS 9`. The same shape was already reachable on a class object at
   BASE. 5c owns what an identity answer means.
