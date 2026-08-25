# Task 7 review: `::CLASS ... MIXINCLASS` and `::CLASS ... INHERIT`

Diff reviewed: `c27b46ea4..6aa432f19` (the implementation commit and the plan-doc commit beneath
it). `c1510a936`, the performance sitting, is out of scope and was not read.

## Spec Compliance

**Spec compliant on everything the brief's "Done when" enumerates, with one half-delivered item
that the brief's letter permits and its intent does not** -- the UNINIT flags. Details in Issues 1
and 2.

Point by point against the brief:

* **Merge order.** Correct, and correct for the shapes the committed programs do not cover. The
  mechanism is `ClassGraph::cascade_build` (`rust/crates/rexx-classes/src/class_graph.rs:312`-`:340`,
  unchanged by this task): superclasses are walked `.rev()`, each ancestor merged before the class's
  own dictionary, and a later merge wins. Since `install_class_at` appends each `INHERIT` entry
  left to right and the `SUBCLASS` target is `superclasses[0]`, the resulting precedence is own >
  explicit superclass > leftmost mixin > later mixins, which is what the brief asks for.
  I ran the four uncovered shapes the dispatch names, three descriptors, fresh directory, oracle
  and both engines -- all byte-identical:
  * a subclass of a class that inherits, and a mixin of a mixin (`MA MIXINCLASS Object INHERIT MB`)
    reached through a further subclass: `P_m / MB_n / MB_m / MA_n / MB_m / MA_n`, rc 0 on all three;
  * `::CLASS K SUBCLASS P INHERIT M` where `P` already inherits `M`: 98.944 with the frame line,
    rc 158, identical on all three;
  * `::CLASS K INHERIT M ZZZNOSUCH` (first entry resolves and is sent, second does not resolve):
    98.909, **no** frame, identical -- which is the per-entry interleave, not resolve-all-then-send;
  * `::CLASS K INHERIT C M` (non-mixin first): 98.942 **with** the frame, identical.
* **`install_class_at`'s order inside the directive** matches `ClassDirective::install`
  (`interpreter/instructions/ClassDirective.cpp:165`-`:229`) as read: resolve the shared
  `SUBCLASS`/`MIXINCLASS` slot, create, then loop the `INHERIT` list resolving and sending one entry
  at a time. The two probes above are what distinguish that from the resolve-all shape.
* **Dependency set.** `class_dependencies` (`rust/crates/rexx-exec/src/lib.rs`, new) is the shared
  slot, `METACLASS` and every `INHERIT` entry, unqualified -- matching `addDependencies`
  (`ClassDirective.cpp:327`-`:344`) and `checkDependency`'s `isQualified` skip. Cycle detection
  generalised from one target to a set: it still terminates (a node is pushed only in state `None`,
  every pushed node reaches the top of the stack and is given a state before its pusher is
  revisited, `Done` is absorbing) and still blames the walk's root. Measured: a cycle through
  `INHERIT` edges (`::CLASS A INHERIT B` / `::CLASS B INHERIT A`) and one through `MIXINCLASS`
  edges (`::CLASS A MIXINCLASS B` / `::CLASS B MIXINCLASS A`) are both 98.911 blaming the first
  directive, byte-identical to the oracle on both engines. A duplicated dependency
  (`INHERIT M M`) pushes the same index twice and is absorbed by the `Done` check at the loop
  head, so nothing is ordered twice.
* **M10.** `Interp::install_class` keys `ClassKind` on `class.mixin`, not on the slot
  (`lib.rs`, `install_class`). Verified in the code, not from the mutation report. Nothing else in
  the changed path keys a kind decision off `subclass.is_some()`; the slot is used only to resolve
  the target, which is correct for both keywords. The parser guarantees `mixin => subclass.is_some()`
  (`rexx-parse/src/directive.rs:464`-`:467`, 19.913 when the name is missing), so `define_class`'s
  `expect` on the mixin arm is unreachable.
* **`~baseClass`.** `native_base_class` is registered at `Class`, answers `ClassSide` from the
  graph and refuses an `Instance` receiver loudly. Its differential rows are in
  `corpus/lang/class_mixinclass.rex`.
* **The refusal ladder.** Six shapes committed, the brief's three among them; every one is a corpus
  program, so the corpus gate is the standing instrument. `ClassGraph::inherit`'s check order maps
  onto `RexxClass::inherit` (`ClassClass.cpp:1298`-`:1332`) one for one, including the class-side /
  instance-side split of the two `Error_Execution_baseclass` tests. The oracle's explicit
  `this == mixin_class` test has no counterpart, and does not need one: a class's own behaviour
  carries its own scope, so `behaviour_has_scope(class, Class, class)` answers the same `Recursive`.
  Substitution order matches the C++ `reportException` argument order in all three.
* **The `98.942` frame line.** Raised by `Interp::inherit_mixin` via `blame_native_method` +
  `blame_directive`, with the scope read from the registry rather than written down. The choice of
  `blame_native_method` over `blame_stem_forwarded_operator` was tested against the latter's own
  stated rule -- I read that doc (`eval.rs:1099`-`:1101`, "A caller is an adapter that evaluates one
  operator whose receiver is `value`") and the new caller genuinely fails it. No list was appended
  to. See Minor 3 for the one doc consequence.
* **`METACLASS` still refuses**, with its own message, and the message names only `METACLASS`.
* **UNINIT flags:** carried at the three constructors and covered by the in-crate test, which is
  the brief's literal requirement. See Issue 1 for what that does and does not deliver.

**What I could not verify from the diff**, for the controller:

* the five gate commands and the 127-of-127 corpus figure -- not re-run, per dispatch;
* the four mutation runs. Control 1's mutation text is described as "`ClassKind` decision rewritten
  to consult `class.subclass.is_some()`", and a literal same-polarity substitution of that condition
  would **not** produce the reported flip (a `MIXINCLASS` directive would still be built `Mixin`);
  the reported observable (`.k~baseClass` answering `The K class`) requires the mutation that builds
  a `MIXINCLASS` directive as a plain subclass, which is the pre-task hard-coding and is what the
  plan's M10 paragraph (`2026-08-17-phase-5a.md:738`) actually describes. The control is genuine and
  matches the brief's prediction; only its one-line description is ambiguous. Worth noting that
  **both polarities are witnessed by committed programs**: the other one reddens
  `class_mixinclass.rex`'s `.K~baseClass` row, which the report itself calls a row that discriminates
  nothing.
* the generated `sourceline_oracle/*.txt` fixtures (nine new files) -- content matches the corpus
  programs line for line and the `count` header matches the file length in each, which is as far as
  reading takes it.

## Strengths

* **The uncovered merge-order shapes hold.** The dispatch's list of things the committed programs do
  not reach -- a mixin inheriting a mixin, a subclass of a class that inherits, the same mixin twice,
  `INHERIT` beside `SUBCLASS` -- all agree byte for byte with the oracle on both engines. The
  implementation is right for the general case and not only for its own witnesses.
* **The install sequence is the oracle's literally**, per-entry resolve-and-send rather than
  resolve-all-then-send. That difference is observable (`INHERIT M ZZZNOSUCH` versus `INHERIT C M`
  produce different reports, one with a frame and one without) and the crate reproduces both.
* **The refusal ladder went wider than the brief and each extra rung is a differential row**, so the
  standing instrument for 98.943 and 98.944 is the corpus gate rather than prose. Given the code
  reaches those arms, the alternative was a panic, which is the right reason to have gone wider.
* **`class_references` is one membership site** feeding both the namespace predicate and the
  dependency set, so the two can never disagree about what a `::CLASS` names.
* **The recorded oracle measurements I spot-checked reproduce exactly.** `::class foo mixinclass
  ns:other` and `::class foo inherit ns:other` are both 98.987 rc 158 with empty stdout, as the
  exclusions file now records.
* **Refused-inherit tests assert the graph is untouched**, not just the error value
  (`ancestors` unchanged, no donated method, no flag) -- that is the half a `should_panic` test could
  never assert, and converting them bought it.
* **The report's own corrections are load-bearing and honest**: the wrong prediction for
  `::CLASS K INHERIT K` (98.944 expected, `missing_body` measured) is recorded as having been found
  by running, and the `cascade_build` `has_scope` mutation that turned out to change nothing is
  reported as a dead control rather than quietly dropped.

## Issues

### Critical (Must Fix)

None.

### Important (Should Fix)

**1. The UNINIT flags are delivered at the graph API only; through the directive path
`parent_has_uninit` cannot be set at all, and for the brief's own three handover programs neither
flag is set.** Files: `rust/crates/rexx-classes/src/class_graph.rs` (`define_class`, `inherit`,
`define`), `rust/crates/rexx-exec/src/lib.rs:3437`-`:3467` (`install_directives`),
`rust/crates/rexx-classes/tests/behaviour_wiring.rs` (`uninit_propagates_through_all_three_constructors`).

Plainly: **this half is half-delivered.** The in-crate test passes over a layer no directive-installed
class reaches. Three separate facts, each checked rather than reasoned:

* `install_directives` creates **every** class in the `for index in &order` loop
  (`lib.rs:3442`-`:3446`) and attaches **every** method in the positional loop after it
  (`lib.rs:3452`-`:3468`). So at the moment `define_class`/`inherit` reads a parent's flags, no class
  declared in the file has any method yet.
* No bootstrap class supplies one either: `UNINIT` does not occur in the generated
  `setup_classes.rs` at all, so the registry never holds a class with `has_uninit` set when a file's
  classes are constructed.
* Therefore `parent_has_uninit` is `false` for every class a Rexx program can declare, always. The
  only writer that can ever set it is a direct `ClassGraph` call -- which is what the test does, in
  an order (`define` before `define_class`) that the directive path never produces.
* And the brief's three handover programs all use `::METHOD uninit CLASS`, which routes
  `install_method` -> `add_class_method` -> `class_define`, which sets **nothing**. `has_uninit` is
  reachable through the directive path only for an *instance*-side `::METHOD uninit`.

**The stated root cause checks out.** The two-pass shape is as described, and
`docs/superpowers/plans/phase-4-exclusions.txt:519`-`:541` already carries a row with the same root
cause ("the ORACLE has every class installed, with its methods, before it evaluates any ::CONSTANT
expression, and this crate evaluates each one as it walks the directive list"). So the disclosure is
accurate as far as it goes.

Why it still matters: the brief made this task the owner of *carrying* the flags precisely so that 5b
starts from a working flag rather than hunting for one. What 5b receives is a flag that is correct at
an API no program reaches and constant-`false` at the path every program takes -- so 5b cannot use it
without doing the `install_directives` restructure first, which is the work this task declares out of
scope. The report's disclosure says the flags are "not yet right through the directive path", which
understates it: it is not a wrong value in one shape, it is an unreachable write for the whole
propagation half.

How to fix: nothing inside this task's scope makes the flag live, so the fix is a record fix plus a
handover fix -- state in the report and in Task 24's handover that (a) `parent_has_uninit` is
unwritable through the directive path, (b) `has_uninit` is unwritable for a class-side `::METHOD
uninit`, and (c) the restructure that closes both is the same one the `::CONSTANT` row needs.
Whether that is enough, or whether the flags should have waited for 5b, is the controller's call --
the brief's letter ("an in-crate test that the flag propagates through all three constructors") is
met.

**2. `ClassDef::has_uninit` is documented as the oracle's `HAS_UNINIT` and is not that flag.**
`rust/crates/rexx-classes/src/class_graph.rs`, the `has_uninit` field doc ("oracle's `HAS_UNINIT`
class flag, which `RexxClass::defineMethod` sets for a method of that name").

The oracle sets `HAS_UNINIT` in two places, not one. Besides `defineMethod` (`ClassClass.cpp:854`),
`RexxClass::checkUninit` (`:1210`-`:1218`) sets it whenever the class's **flattened**
`instanceBehaviour` holds `UNINIT` -- and `checkUninit` is called from `subclass` (`:1626`),
`define` (`:508`), `defineMethods` (`:543`) and `inheritInstanceMethods` (`:585`). So for
`::CLASS P` with an instance `::METHOD uninit` and `::CLASS K SUBCLASS P`, the oracle's `K` has
`HAS_UNINIT` **set**; this crate's `has_uninit(K)` is `false`, and
`behaviour_wiring.rs`'s `assert!(!g.has_uninit(child), "the subclass defines none of its own")` pins
that divergent value as if it were the oracle's.

Why it matters rather than being cosmetic: the site the flag exists for reads exactly this flag and
not the parent one -- `ClassClass.cpp:1892`, `if (hasUninitDefined()) obj->requiresUninit();` at
instance creation. A 5b implementer porting that line against this field, guided by its doc, registers
too few objects and gets a silent wrong answer -- the exact failure mode this phase's constraints
exist to prevent. The crate already has the object-side half waiting (`Heap::set_uninit`,
`rexx-core/src/heap.rs:314`), so the reader is close.

How to fix: either model `checkUninit`'s first half (set `has_uninit` when the rebuilt instance
behaviour holds `UNINIT`), or narrow the field's doc to what it is -- "this class defines `UNINIT`
in its own instance dictionary" -- drop the `HAS_UNINIT` equation and the "so its instances have to
be registered" consequence, and record the gap with Issue 1's for 5b.

### Minor (Nice to Have)

**3. `rust/crates/rexx-classes/tests/behaviour_wiring.rs:566` says `98.943` where the condition is
`98.944`.** The sentence is the twin of the one in `class_graph.rs`'s `inherit` doc, which this same
commit corrected from `98.943` to `98.944`; the test below it now asserts
`Err(InheritRefusal::Recursive)`, whose catalogue number is 98.944 (`error.rs`,
`recursive_inherit`). One-word fix.

**4. Two comments name a set's size.** `lib.rs`'s `class_references` doc ends "which asks
`checkDependency` about exactly these three", and `error.rs`'s `cyclic_inheritance` doc was edited
from "on two shapes" to "on three shapes". The first also enumerates the members immediately above
the code that enumerates them. The enumeration is arguably exempt (the function body *is* the
membership rule), the count is not. Say "about the references `class_references` yields" and drop
the numeral before the list.

**5. `dispatch.rs:862`-`:867`'s `pub(crate)` rationale for `blame_native_method` now under-describes
its callers.** It names an arithmetic operator's left operand and a controlled `DO` header's
numeric position, both in `eval.rs`; the directive-initiated `INHERIT` send in `lib.rs` is a third
caller in a third module and the doc does not admit one exists. Nothing false was added -- the doc
was not touched -- but the enumeration became partial the moment this task landed, which is the
shape the constraint warns about. Replace the two situations with the rule (any module that raises
from inside a native method's own frame).

**6. The namespace arm now also swallows `METACLASS ns:`, and the record does not say so.**
`class_names_a_namespace` asks every reference including `metaclass` -- deliberately, and its own doc
says so -- and it sits above the `METACLASS` arm, so `::class k metaclass ns:x` reports `::CLASS
naming a namespace` rather than `::CLASS METACLASS`. `phase-4-exclusions.txt`'s new paragraph says
the namespace message covers "SUBCLASS, MIXINCLASS and INHERIT" and does not mention `METACLASS`,
and the in-crate refusal tests cover the three but not the fourth spelling. Both messages are loud
rc-120 refusals, so nothing observable changes; the record is what is incomplete.

## Assessment

**Task quality:** Needs fixes.

The construct itself is done well and holds up under adversarial probing: the merge order is right
for shapes no committed program reaches, the install sequence reproduces the oracle's per-entry
resolve-and-send rather than an equivalent-looking alternative, the generalised dependency set
catches cycles through mixin and inherit edges alike, and M10 is written against the flag in the
code and not only in the mutation report. What is not delivered is the UNINIT half: the flags are
correct at an API no directive-installed class reaches, `parent_has_uninit` cannot be `true` for any
class a program declares, and `has_uninit` is not the oracle's `HAS_UNINIT` under a doc that says it
is -- so 5b inherits a flag it cannot read rather than a witness it can start from. Issues 1 and 2
are record-and-doc fixes plus a controller decision about scope, not a rewrite of the construct.
