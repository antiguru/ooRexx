# Task 7, fix round 1 -- re-review

Scope: verdict each finding from `task-7-fixround-1-brief.md`, and inspect `c1510a936..c35ba11cc`.
Nothing else. The performance-sitting commit after this head is out of scope and was not looked at.

**Round verdict: ACCEPT the mechanism, with prose corrections owed.** The deviation from my brief is
correct and my brief was wrong -- I re-took the discriminating probe and it reproduces. `check_uninit`
is the oracle's function, the post-pass in dependency order gives the oracle's answer on every shape I
could construct (nine measured), the new test is structurally red without the pass and both halves of
the pass are independently witnessed, and the assertion no longer freezes a divergence. What is owed is
four prose defects, one of them a **false statement about the new test's own negative control**.

---

## 1. Is `check_uninit` the oracle's function?

Read against `interpreter/classes/ClassClass.cpp:1210`-`:1226` directly:

```cpp
1215:    if (instanceBehaviour->methodLookup(GlobalNames::UNINIT) != OREF_NULL)
1217:        setHasUninitDefined();
1222:    if (hasUninitMethod())
1224:        requiresUninit();
```

`ClassGraph::check_uninit` (`rust/crates/rexx-classes/src/class_graph.rs:295`) reads
`behaviour_handle(class, Side::Instance)`'s dict -- the **flattened** instance behaviour, which is what
`instanceBehaviour` is after `createInstanceBehaviour` -- and only sets. **One-way and idempotent, and
the oracle's is too.** The doc's disclaimer of the second half (`:1222`, `hasUninitMethod` ->
`requiresUninit` on the class *object*) is accurate, and its claim that `RexxObject::checkUninit`
(`ObjectClass.cpp:2604`) is a different function that *does* clear is true -- verified, its `else`
branch calls `removedUninit()`.

**Where the oracle calls it, including the case the brief and the fix each covered from one side.**
`subclass` at `:1628` is one site; the brief cited it. The others that matter here:

| site | reaches `checkUninit`? |
|---|---|
| `subclass` (`:1628`) | directly |
| `defineMethods` (`:508`) -- what `ClassDirective::install` uses for a class's `::METHOD`s | directly |
| `inherit` (`:1287`) | **yes, indirectly**: `updateSubClasses()` at `:1359` calls `checkUninit` at `:1052` and cascades it to every subclass |
| `mixinClass` (`:1516`) | via the `subclass` it calls |

So `inherit`'s `hasUninitDefined() || parentHasUninitDefined()` at `:1364` is **not** the whole of the
mixin story -- it sets `PARENT_HAS_UNINIT` only, and `HAS_UNINIT` for the inheriting class arrives
through `updateSubClasses`. That is the case the brief flagged as most likely to be wrong, and it is the
case the fix gets right, because the fix keys on the flattened behaviour rather than on the edge.

Measured, oracle, fresh empty dir, three descriptors read separately:

```
::class M mixinclass Object / ::method uninit / ::class K inherit M      rc 0, "uninit via M for K"
```

An instance of `K` runs the mixin's `uninit`, which only happens if `K` itself carries `HAS_UNINIT`
(`completeNewObject`, `:1892`, reads exactly `hasUninitDefined()` on the class being `new`ed). The fix
answers `true` here -- confirmed by running, below.

Every line number the new prose cites was checked in the C++: `:854`, `:1210`-`:1218`, `:1222`, `:1364`,
`:1525`, `:1628`, `:1634`, `:1892`, `ObjectClass.cpp:2604`, `ClassDirective.cpp:327`-`:344`. All correct
**except `:1214`** -- see the prose section.

## 2. Does the post-pass in dependency order give the oracle's answer?

**Reasoned, then run.** The oracle's `ClassDirective::install` finishes one class entirely -- create,
then each `INHERIT` send, then `defineMethods` -- before starting the next, and the package installs
class directives in dependency order. So at the moment a class's `PARENT_HAS_UNINIT` is computed, its
parent and every mixin it inherits are already final, and its own `HAS_UNINIT` is recomputed by
`defineMethods`' `checkUninit` after its methods land. Both flags therefore equal a *fixpoint* the
oracle happens to reach incrementally: `HAS_UNINIT(C)` iff `C`'s final flattened instance behaviour
answers `UNINIT`; `PARENT_HAS_UNINIT(C)` iff some superclass of `C` carries either flag. The fix
computes that same fixpoint in one pass over `order`. The flags are monotone and no behaviour ever
shrinks, so recomputing later cannot lose a set the oracle made.

The one structural gap that could have broken this -- `defineMethods` (`:508`) does **not** call
`updateInstanceSubClasses`, so a subclass created before its parent's methods land would never be
rebuilt -- is unreachable in dependency order, in the oracle and here alike.

**Run rather than argued.** Nine shapes, each measured on the oracle for the observable
(`HAS_UNINIT` is observable: it decides whether an instance's `uninit` fires) and each asserted against
this crate's flags. Eight I added as throwaway tests in an out-of-repo copy of `rust/`; the first row is
the committed test's own `Base`/`Kid`/`Grandkid` chain. Every row agreed.

| shape | oracle | crate |
|---|---|---|
| `P` + instance `uninit`; `K subclass P` | `uninit for K` | `has(K)`, `parent(K)` |
| forward: `K subclass P` declared **before** `P`, `P` has `uninit` | `uninit for K` | `has(P)`, `has(K)`, `parent(K)` |
| mixin: `M mixinclass Object` + `uninit`; `K inherit M` | `uninit via M for K` | `has(M)`, `has(K)`, `parent(K)` |
| forward mixin: `K inherit M` declared **before** `M` | `uninit for K` | same |
| three generations off a mixin: `A inherit M`, `B subclass A`, `C subclass B` | `uninit for A`, `B`, `C` | `has`+`parent` on A/B/C, `parent(M)` false |
| **middle** class declares it: `A`; `B subclass A` + `uninit`; `C subclass B` | `uninit for C`, `B` only | `has(A)` false, `has(B)` true `parent(B)` false, `has(C)`+`parent(C)` |
| both edges: `P`; `M mixinclass P` + `uninit`; `K subclass P inherit M` | `uninit for K` only | `has(P)` false, `has(M)` true, `has(K)`+`parent(K)` |
| diamond: `M`+`uninit`; `A inherit M`; `B mixinclass Object`; `D subclass A inherit B` | `uninit for D` | `has(D)`, `parent(D)` |
| mixin of a mixin: `M`+`uninit`; `N mixinclass M`; `K inherit N` | `uninit for N`, `K` | `has`+`parent` on N and K |

No divergence found. The negative rows are real negatives -- `has_uninit(A)` in the middle-class shape
and `has_uninit(P)` in the both-edges shape are asserted `false` and read `false`, so the pass is not
setting the flag on everything.

## 3. The over-registration hazard my brief would have created

**My brief's ruling was wrong and the implementer's reading is right.** Re-taken on the oracle, from a
fresh empty dir:

```
::class K / ::method uninit class          , two `.K~new`   -> "uninit on K"   ONCE   (rc 0)
::class K / ::method uninit  (instance)    , two `.K~new`   -> "instance uninit" TWICE (rc 0)
```

The second line is the control the first one needs: two instances *do* produce two prints when the flag
is set, so "once" for the class-side spelling means the instances were not registered and the single
print is the class object itself (`hasUninitMethod` -> `requiresUninit`, `:1222`). Had the brief's shape
been built, every instance of every class with a class-side `uninit` would have been registered for an
`UNINIT` the oracle does not run for it.

The fix does not do that, verified two ways: `ClassGraph::class_define` sets no flag, and `check_uninit`
reads `Side::Instance`, where a `::METHOD uninit CLASS` never lands.
`a_class_side_uninit_sets_neither_flag` (`rust/crates/rexx-exec/src/lib.rs:5470`) pins it.

On both engines the probe is loud, not silent: `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker` both give
rc 120 and `rexx-exec: method "NEW" of class "Object" is not implemented (Phase 5)` on stderr with
`main ran` on stdout, for every one of these programs.

## 4. The new directive-path test

`the_uninit_flags_are_set_for_the_classes_a_file_declares` (`rust/crates/rexx-exec/src/lib.rs:5422`).

Verified by mutation, in an out-of-repo copy, with a **plain `cargo test` and no gate environment
variable set** -- so these are structural failures, not verdict ones:

| mutation | result |
|---|---|
| whole pass (`lib.rs:3546`-`3552`) deleted | FAILED at the `has_uninit(kid)` row |
| only `self.classes().check_uninit(class);` deleted | FAILED at the `has_uninit(kid)` row |
| only `self.classes().refresh_parent_has_uninit(class);` deleted | FAILED at the `parent_has_uninit(kid)` row |

**Both halves of the pass are independently witnessed**, which is more than the report claimed. The
committed tree passes.

## 5. The unpinned assertion

`rust/crates/rexx-classes/tests/behaviour_wiring.rs:628`-`:645` now asserts `false` **before**
`g.check_uninit(child)` and `true` **after**, with the divergence named as a divergence and the oracle
measurement cited. It no longer freezes a wrong answer.

The sentence naming the divergence is true: `has_uninit(child)` is `false` at that point in this crate
because `define_class` does not call `check_uninit`, and the oracle's is `true` because `subclass` calls
`checkUninit` at `:1628` -- both verified, the second by running. The test's rename to
`..._at_the_graph_api` and its doc's "this is not evidence about what a Rexx program gets" discharge the
first of the brief's three owed items.

**One thing worth recording rather than fixing:** the graph API's divergence is real but inert today.
A direct `define_class` caller does get `has_uninit` where the oracle would have it set. The only such
caller is the generated `setup_classes.rs`, where `UNINIT` occurs nowhere, so no image class is affected.
If 5b adds a class-graph caller outside `install_directives`, it has to call `check_uninit` itself.

## 6. Findings, verdicted

| # | finding | verdict |
|---|---|---|
| 1 | flags could never be set through the directive path | **ADDRESSED** -- `rust/crates/rexx-exec/src/lib.rs:3546`-`3552`, test at `:5422`, graph-API test renamed at `rust/crates/rexx-classes/tests/behaviour_wiring.rs:600` |
| 2 | `has_uninit` documented as `HAS_UNINIT` and is not it; assertion pinned the divergence | **ADDRESSED** -- `rust/crates/rexx-classes/src/class_graph.rs:143`-`:165` and `:283`-`:299`; assertion pair at `behaviour_wiring.rs:628`-`:645` |
| m1 | `98.943` should be `98.944` | **ADDRESSED** -- `behaviour_wiring.rs:566`; no stale `98.943` remains in that file |
| m2 | two comments name a set size | **ADDRESSED but regressed** -- both originals are gone (`lib.rs:1218`, `error.rs:503`-`:508`); three new size mentions appeared, see below |
| m3 | `pub(crate)` rationale names two of three callers | **ADDRESSED** -- `rust/crates/rexx-exec/src/dispatch.rs:862`-`:868` states the rule first ("anything that reaches a native method's body and raises from inside it owes this line") and the examples that follow are marked non-exhaustive. The three callers are `eval.rs:1120`, `dispatch.rs:548`, `lib.rs:3740`; the rule admits all three |
| m4 | `METACLASS ns:` uncovered | **ADDRESSED** -- rows in both `run/tests.rs` tables, the `directive_gap` arm comment, and `phase-4-exclusions.txt`. Oracle re-measured: `::class foo metaclass ns:other` is 98.987 rc 158, stdout empty |
| clarify | control 1's description | rewritten in the report; not re-verified here (report-only, out of the diff) |

## 7. New prose defects

House method: contiguous comment blocks collapsed to one line for base and head over all seven changed
`.rs` files, `diff`ed, every new line read without a keyword filter.

**(a) A false statement, `rust/crates/rexx-exec/src/lib.rs:5411`-`:5413`.** The new test's doc says
`install_directives` runs the pass "once the methods are in; **without that pass every assertion below
that expects `true` reads `false` instead**, for every program that can be written." That is false and
the mutation run above disproves it: with the whole pass deleted the test fails at the `kid` row,
*after* `assert!(interp.classes().has_uninit(base), "the declaring class")` has already passed --
`ClassGraph::define` sets `has_uninit` on the declaring class itself (oracle's `defineMethod`, `:854`),
with no pass involved. The rows that do depend on the pass are the ones about *inherited* `UNINIT` and
the `parent_has_uninit` ones; the declaring class's own `has_uninit` is not one of them. Fix: say which
rows the pass decides rather than "every assertion below that expects `true`".

**(b) A set size, plus its enumeration, `rust/crates/rexx-classes/src/class_graph.rs:151`.**
"**Two oracle sites set it and this crate has both**", followed by both members. This is the shape the
house rule forbids, and it is new this round. It is also contestable as a count:
`setHasUninitDefined()` appears at four call sites in the C++ (`:856`, `:1217`, `:1642`, `:1841`), two of
which are the tautology `if (new_class->hasUninitDefined()) new_class->setHasUninitDefined();`. Say what
puts a class in the set (its flattened instance behaviour answers `UNINIT`, or a method of that name is
being added) and point at `checkUninit` and `defineMethod`.

**(c) The same size twice more.** `class_graph.rs:275`, "See the field for **the two sites** that set
it." And `class_graph.rs:308`-`:309`, "the propagation **the three constructors** do reads a finished
parent". The "three constructors" phrasing is inherited from the unchanged `parent_has_uninit` field doc
and the test name, so it is at least consistent -- but the sentence carrying it is new this round.
`lib.rs:5440`, "The class that declares it, and **the two** that inherit it", is the mildest: the two
assertions directly under it enforce the list, which is the rule's own exemption.

**(d) An imprecise citation, `class_graph.rs:155`.** `checkUninit` is cited as `(`:1214`)`. `:1214` is
the third line of a comment inside that function; the line that tests is `:1215` and the line that sets
is `:1217`. The neighbouring `check_uninit` doc cites the same function correctly as `:1210`-`:1218`.

**Soft, not a defect:** `class_graph.rs:307`-`:308`, "The oracle attaches a class's methods while it
constructs the class", is loose -- `ClassDirective::install` attaches them *after* `subclass()` returns,
which is exactly why `subclass`'s own `checkUninit` at `:1628` does not see them. The consequence the
sentence draws (the propagation reads a *finished parent*) is nevertheless true, because the parent's
whole directive install precedes the child's.

**Nothing else.** No historical framing ("used to", "no longer", "previously") in any new line -- struck
and re-read, each new sentence still says the same thing about the code as it is. No non-ASCII in any
added line (the four non-ASCII characters in `lib.rs` are at `:268`, `:292`, `:492`, `:1785`, all
pre-existing). No `unsafe`. Every measurement number in the new prose that I could re-take, I re-took,
and all of them hold: 98.987 for `METACLASS ns:` and for `SUBCLASS ns:`, 98.911 rc 158 stdout-empty for
`::class a metaclass b` / `::class b metaclass a` echoing the first directive, `98.944` for the
recursive-inherit case, and `ClassDirective::addDependencies` (`:327`-`:344`) asking `checkDependency`
about `metaclassName`, `subclassName` and each inherits entry and nothing else.

**Trivial:** `lib.rs:3547`-`3549`'s `let Some(class) = classes.get(index).copied() else { continue; };`
is dead -- the loop at `:3444` inserts every index in `order` into `classes`, so the `else` arm cannot
be taken. It would silently skip a class rather than panic if that ever stopped holding. Not worth a
round on its own.

## 8. "No differential row could witness this round's change"

**True.** Checked rather than accepted:

* Nothing outside tests reads `has_uninit` or `parent_has_uninit`. `/bin/grep` over `rust/crates` finds
  the accessors used only in `class_graph.rs`, `registry.rs`, and assertions.
* `rexx-core`'s `Object::has_uninit` (`heap.rs:306`-`:346`) is a *different* flag, on the heap object,
  and `rexx-exec` never sets it -- `lib.rs:4308`-`:4318` says so and asserts the resurrection list stays
  empty.
* Any program that could observe an `UNINIT` needs an instance. `~new` refuses loudly on both engines:
  rc 120, `rexx-exec: method "NEW" of class "Object" is not implemented (Phase 5)`. Measured on four of
  the probes above, `ir` and `tree-walker` alike.

So the flags are inert in every byte a differential row compares, and an in-crate test is the only
instrument that can see this round's change. The report's claim stands.

## 9. Gates

Run by me, unpiped, on an out-of-repo copy of `rust/` with `target` deleted and
`interpreter/ oodocs/ ootest/ docs/ samples/` symlinked in:

```text
cargo fmt --all --check                                    exit 0
cargo clippy --workspace --all-targets -- -D warnings      exit 0
REXX_CORPUS_GATE=1 cargo test --workspace --no-fail-fast   exit 0
```

The third of those printed `mode: STRICT (the gate) -- REXX_CORPUS_GATE is set` and `127 of 127
matching`, which corroborates the report's corpus claim.
`/bin/grep -E "^(failures:|test result: FAILED)"` over its output matched nothing.

**What I did not re-run:** the report's gates 3 and 4 are the `--release` forms and gate 5 adds
`memcap 8G`. I ran the debug profile without the memcap, which exercises the same tests and the same
corpus comparison but is not a byte-identical repeat of the report's five commands. A performance
sitting was running concurrently, so no timing was taken and none is reported.

## 10. What is owed

Prose only -- the mechanism, the test and the assertion are all sound.

1. `lib.rs:5411`-`:5413` -- the false claim about which assertions depend on the pass. **Must fix**; it is
   a statement about the round's own negative control, and it is the kind of sentence that gets cited.
2. `class_graph.rs:151` -- "Two oracle sites set it ... and both". Rewrite as a membership rule.
3. `class_graph.rs:275` and `:308` -- "the two sites", "the three constructors". Same rule.
4. `class_graph.rs:155` -- `:1214` -> `:1215` or `:1217`.
