# Task 8 review: `::CLASS ... METACLASS`, and the rest of `::CLASS`'s option surface

Range reviewed: `d0b7bd151..6754ae2a2` for risks 1, 2, 3, 4, 6, 7, 8; `121bc720d` (the commit the
controller named mid-review) for risk 5. `8fb97527c`'s four citation corrections are excluded as
directed -- two of them are inside this range, and what the pass saw of them is recorded under Minor.

Everything below that says "measured" was run by me in this checkout: oracle under the standard wrapper, three descriptors read separately, `cmp` on the raw
bytes, from a fresh empty directory with the programs held in a second directory. The crate side is
`rust/target/release/rexx-run`, built 20:47 against `6754ae2a2` (20:44) with only `dispatch.rs`'s doc
comment outstanding at the time, so its behaviour is the reviewed head's.

Nothing in this checkout was mutated. No mutation of the tree was re-run, so section 6 of the report
is unverified by construction (see "What I could not verify").

## Spec Compliance

**Spec compliant.**

The brief's four demands, checked one at a time:

* **The metaclass witness matches byte for byte on both engines.** Independently measured, not taken
  from the report: all nine committed programs -- `corpus/lang/class_metaclass.rex`,
  `class_metaclass_class_method_does_not_donate.rex`, `class_metaclass_superclass_wins.rex`,
  `class_metaclass_not_found.rex`, `class_metaclass_not_a_metaclass.rex`, `class_metaclass_cycle.rex`,
  `class_abstract_metaclass.rex`, `class_abstract_metaclass_subclass.rex`,
  `class_abstract_metaclass_after_inherit.rex` -- agree with the oracle on exit status, stdout and
  stderr under `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker` alike.
* **`S`'s method does not carry `CLASS`.** `class_metaclass.rex:24` is a bare `::METHOD classSideHi`;
  the `CLASS` spelling is a separate program pinning 97.1 rc 159, and I reproduced both.
* **D44's class-behaviour question has an answer in the tree.** `ClassGraph::cascade_build`'s
  `Side::Class` arm (`rust/crates/rexx-classes/src/class_graph.rs:410`-`:416`) is the oracle's
  `createClassBehaviour` (`ClassClass.cpp:1116`-`:1129`) line for line, including the
  `hasScope(metaClass)` guard, and `Interp::install_class_at`'s post-attach loop
  (`rust/crates/rexx-exec/src/lib.rs:3537`, comment at `:3519`-`:3536`) now says why the rebuild is on
  the class side. `update_sub_classes` (`class_graph.rs:458`) rebuilds instance then class, which is
  `updateSubClasses`' documented order (`ClassClass.cpp:1036`, comment at `:1043`-`:1046`).
* **The refusal arm is deleted, not narrowed.** `directive_gap`'s `class.metaclass.is_some()` arm is
  gone from `rust/crates/rexx-exec/src/lib.rs`, and its two rows leave `run/tests.rs` in the same
  commit. `/bin/grep -rn "CLASS METACLASS" rust/ docs/` finds only `docs/superpowers/records/`
  (pattern control: the same pattern matches in the task report). The namespace-qualified spelling is
  not orphaned -- measured, `::class foo metaclass ns:other` still reaches the namespace gap
  (`rc 120`, `::CLASS naming a namespace is not implemented (Phase 5)`) and the two `metaclass ns:`
  rows remain at `run/tests.rs:7606` and `:7781`.

`PRIVATE`/`PUBLIC` are handled as the brief asks -- stated as not covered rather than implied to be,
report section 3 -- and `ABSTRACT` gained the one effect that *is* reachable, `98.990`, with three
programs behind it.

**What the controller should check.** The mutation table (report section 6) and the five gate
statuses were not re-run here; and `121bc720d` lands a doc comment in `src/` with no sitting, which
the performance guard's own escape clause (`tests/`, `corpus/`, `docs/`) does not literally cover.
That is the controller's call, not a finding against this task.

## Strengths

* **The corner shapes hold up.** I built fourteen shapes the committed programs do not cover and
  every one matches the oracle on both engines: a metaclass derived from a metaclass
  (`::CLASS M2 SUBCLASS M1` used as `K`'s metaclass, rc 0); a metaclass donation colliding with the
  superclass's own class-side method of the same name (the metaclass wins, both sides); `SUBCLASS` +
  `METACLASS` + `INHERIT` on one directive; keyword order reversed in the source
  (`subclass zzznosub metaclass zzznometa` is still 98.908); a resolvable metaclass with an
  unresolvable subclass (98.909, so the ordering claim is right for the neighbouring shape too);
  `metaclass object inherit zzznotaclass` (99.927 ahead of INHERIT); `::class k metaclass k`
  (98.911); a metaclass declared later (99.927 naming it); `::CLASS K SUBCLASS Class ABSTRACT`
  (98.990); `ABSTRACT` after a *successful* `INHERIT` on a metaclass (98.990, which the committed
  after-inherit program does not reach); a lowercase directive (`::class s mixinclass class abstract`
  names `S`); and a quoted target (`metaclass "zzzm"` names `ZZZM`).
* **The `hasScope` subtlety is reproduced without being noticed as such.** `::CLASS T SUBCLASS S
  METACLASS M1` with `S` a metaclass answers 97.1 on the oracle, because `S` is already a scope in
  `T`'s class behaviour by the time the metaclass merge is considered. The crate answers 97.1 too, on
  both engines -- the guard at `class_graph.rs:412` is what earns that.
* **The table D probe rewrite is worth more than the report claims for it.**
  `corpus/gate-tables/directives/class__metaclass__subkeyword.rex` went from `say 'main'` to
  `say .k~classSideHi`, and `gate_tables/mod.rs:195`-`:210`'s `run_on_both_engines` asserts the two
  engines agree **unconditionally, in every mode**. That makes the rewrite, not any report prose, the
  committed instrument that holds the tree-walker to the IR for metaclass donation. Measured: the
  probe is rc 0 `from the metaclass` on the oracle and on both engines.
* **The deleted program's reasoning is sound, and I checked it rather than the wording.** The plain
  shape (`::CLASS M MIXINCLASS Object` + `::CLASS K METACLASS M`) reddens under "delete the check"
  but not under "check after the override" -- `K`'s superclass is `.Object`, so no override runs and
  the check still fires. The kept corner shape (`::CLASS S MIXINCLASS Class METACLASS Object`)
  reddens under both, because the override makes `.Class` the metaclass and a late check then passes.
  Strictly more coverage in one fewer program. The retired shape still matches the oracle here
  (measured, 99.927 `"The M class"`, both engines), so nothing regressed with it.
* **The 98.990 regression story is real.** Rows 1, 2 and 4 I measured directly: oracle 98.990 rc 158;
  `bench-baselines/pinned/rexx-run-15a1ffa98` rc 120 `::CLASS MIXINCLASS is not implemented (Phase
  5)`; head byte-identical to the oracle on both engines. Row 3 I verified from source instead of
  from a build: at `d0b7bd151`, `git grep abstract_ -- rust/crates` outside `rexx-parse` is empty, so
  the keyword was parsed and dropped and the program could only have been rc 0 `main ran`. Loud
  refusal to silent wrong answer to match, exactly as reported.
* **The pin control was run and it holds.** `sha256sum` of the pinned binary matches `PINNED.md`, and
  every commit `git log --oneline 15a1ffa98..6754ae2a2 -- rust/crates rust/Cargo.toml
  rust/Cargo.lock Cargo.toml` lists is in `progress.md` (checked mechanically, none missing).
* **The `blame_native_method` rule is now stated as a rule and it survives the discriminating
  probe.** See risk 5 below.

## Issues

### Critical

None.

### Important

**1. `rust/crates/rexx-classes/src/registry.rs:180`-`:190` states something this task's own new code
path falsifies.** The doc on `class_of` claims that for a class object the oracle's `~class` and
`~metaClass` coincide, "because a class's own `behaviour->setOwningClass` argument is always its
metaclass (`ClassClass.cpp:1615`)". `:1615` passes the **local** `meta_class` -- the value the
directive named, or the superclass's default -- while `:1590`, which this task implements as
`define_class`'s override, has already repointed the `metaClass` *field* at the superclass. The two
therefore differ exactly in the case Task 8 made reachable. Measured on the oracle:

```rexx
say .T~metaClass~id .T~class~id .T~isMetaClass    /* -> S M1 1 */
::CLASS S MIXINCLASS Class
::CLASS M1 MIXINCLASS Class
::CLASS T SUBCLASS S METACLASS M1
```

`class_of` and `metaclass` both read one field (`registry.rs:191`, `:196`), so when Task 9 lands
`~class` on this accessor it will answer `S` where the oracle answers `M1`. Not observable today --
`class_of` has only test callers and neither method is implemented -- which is why this is Important
and not Critical. The doc's own measurement (`.string~class~id`) is a case where they coincide, so it
cannot witness the general claim. Fix: restate the doc as the oracle's two values with the condition
that separates them (`ClassClass.cpp:1586`-`:1591` moves the field, `:1615` does not move the owning
class), and either split the field now or record the divergence where Task 9 will read it.

**2. `docs/superpowers/plans/phase-4-exclusions.txt:604`-`:605` makes a false claim about the
oracle.** "deriving from a metaclass discards a named METACLASS altogether, since `.Class` is then
the metaclass whatever the directive said". The first half is right; the second is only right when
the superclass *is* `.Class`. Measured, same program as above: `.T~metaClass~id` is `S`, not `Class`.
The crate's own doc has it right (`class_graph.rs:237`-`:238`, "takes `superclass` as its own
metaclass"), so this is the record disagreeing with the code, in a file later tasks read as the
authority on what was measured. Fix: "the superclass becomes the metaclass, whatever the directive
said -- `.Class` where the directive derived from `.Class`".

**3. `rust/corpus/phase-5a.txt:254`-`:257` names the size of a set and then enumerates it.** "The two
97.1 programs are the boundaries of the donation: the class-side spelling on the metaclass donates
nothing, and a metaclass named on a directive that derives from one is discarded." This is the
construction the global constraint forbids and the shape the report's own section 8 says it struck
elsewhere in this diff ("a cardinality immediately followed by its own enumeration. The enumeration
stays, the count goes"), so it is one instance surviving the sweep that removed its twin. Nothing
enforces the count -- `EXPECTED_SUBSET_5A` enforces the file's whole list, not this subset of it.
Fix: "The 97.1 programs are the boundaries of the donation: ...". Plan-mandated constraint.

### Minor

* **`rust/crates/rexx-exec/src/lib.rs:3733`** cites `(ClassDirective.cpp:180 against :189 and
  :223)`. `:180` is the `reportException` itself; `:189` and `:223` are the null tests, whose
  `reportException`s are at `:191` and `:225`. This one is **not** among the four `8fb97527c`
  corrects, so it stands as a finding.

  **What the pass caught before that commit landed, since the controller asked.** Reading the C++ for
  risks 3 and 5 turned up three of the four independently, from the source rather than from the fix:
  `Raised::class_not_found`'s `ClassDirective.cpp:214`-`:219` (I read `:212`-`:232` and recorded that
  the lookup is `:222` and the send `:230`, so the cited range names the loop header and a comment);
  `Raised::abstract_metaclass`'s `:246` (`:247` is `if (isAbstract())`, `:249` the call, `:246` the
  comment above); and `install_class_at`'s `ClassClass.cpp:1753` (`:1754` is the function, `:1753`
  the doc comment's closing ` */`). The fourth, `Interp::inherit_mixin`'s `:224`, the pass did **not**
  reach on its own -- that doc is outside the diff's hunks and I saw it only in the working tree. So
  the pass caught every citation that the diff itself put in front of it, including the one carried in
  as unchanged context from the previous task, and missed the one it never read.

  Every other C++ citation this task added I checked and they are exact:
  `ClassDirective.cpp:165`-`:249`, `:180`, `:230`, `:205`, `:200`; `ClassClass.cpp:744`-`:747`,
  `:1036`, `:1116`-`:1129`, `:1514`-`:1519`, `:1566`-`:1575`, `:1572`, `:1586`-`:1591`, `:1761`;
  `StemClass.cpp:280`.
* **The report's `ir_dual` justification names an instrument that does not cover the case.** "the
  send that observes the result travels the same dispatch path the existing `message-sends`
  population already runs on both engines" -- measured, no stanza in `tests/ir_dual_cases` contains
  `::class` or `::method` at all (swept every file; the pattern matches 6 times in
  `corpus/lang/class_metaclass.rex`, so it is not a dead pattern), and `message-sends` sends only to
  primitive receivers. The conclusion still stands, for a better reason the task itself produced --
  the gate-table probe rewrite and `run_on_both_engines`' unconditional engine-agreement assertion.
  Worth correcting in the record so the next task does not lean on the stated reason.
* **The report's "28 commits" is the count at the base, not at the sitting.** Measured: 28 at
  `d0b7bd151`, 29 at `bb6d46466` (where the sitting ran), 30 at `6754ae2a2`. The control's outcome is
  unaffected -- the extra entries are the task's own and both are in `progress.md`.
* **"`arith` is the four values Task 6 and Task 7 each recorded, to six decimals"** is off by one in
  the last digit for one cell: `ir small` median is `1.001284` in Task 7's two sittings and
  `1.001283` here (Task 6's `c351fa473` reads `1.001283`). The ranges overlap; the argument is
  unaffected.
* **`corpus/gate-tables/directives/class__abstract__subkeyword.rex` is still `say 'main'` /
  `::class k abstract`,** which any build that drops `ABSTRACT` on the floor passes -- the same
  defect the METACLASS probe was rewritten to fix. The report says this for `PRIVATE`/`PUBLIC` and
  for table C's `typcla`, but not for this table D probe, where the honest sentence is now available
  (the discriminating shape needs a metaclass, and lives in the corpus instead).
* **The brief's witness program is pinned as a superset, not verbatim:** `class_metaclass.rex`'s `K`
  directive carries `ABSTRACT` (`class_metaclass.rex:27`) and the plain shape is pinned under
  different names by the table D probe. Coverage is complete and the extra keyword is argued for in
  the file's own comment; recorded only because the brief said "the program is pinned, not
  described".
* **Neighbouring shape that stays a declared gap:** `::CLASS K SUBCLASS P MIXINCLASS Object METACLASS
  S` is oracle 25.901 rc 231, and this crate is rc 120 `rexx-exec: 25.901: Invalid subkeyword found.`
  The pinned pre-phase build answers identically, so it is standing, not this task's.

## The named risks, one by one

1. **Merge position.** Reproduced. `update_sub_classes` builds instance then class; `cascade_build`'s
   class arm merges the metaclass's flattened instance behaviour under the same `hasScope` guard as
   `createClassBehaviour`. The order-independence argument in `lib.rs:1059`-`:1066` is correct: the
   class side reads each ancestor's own dictionary (walked) and the metaclass's instance behaviour,
   which `add_instance_method` has already finalised for every class in the file. All four shapes the
   dispatch named behave identically on both sides -- metaclass derived from a metaclass, the name
   collision (metaclass wins), a metaclass declared later, and the keyword combinations, except that
   `SUBCLASS`+`MIXINCLASS` on one directive is a parse-level 25.901 the oracle rejects too.
2. **`is_metaclass` and the override.** Verified against `ClassClass.cpp:1586`-`:1591` and against the
   oracle, not against the report: `new_class->metaClass = this` sets the *superclass*, which is what
   `define_class` does, and `setMetaClass()` is conditioned on the superclass alone -- measured,
   `.K~isMetaClass` is `0` and `.S~isMetaClass` is `1` for `::CLASS S MIXINCLASS Class` +
   `::CLASS K METACLASS S`. The `.Class` seed is placed before any class is derived from `.Class`
   (`native_classes.rs:311`). One consequence went unrecorded: see Important 1 and 2.
3. **The three refusals and their order.** `98.908` ahead of `98.909` holds in both source orders;
   `98.909` beats the metaclass validity check when the metaclass resolves and the subclass does not;
   `99.927` beats `INHERIT`'s `98.909`; `98.990` is last, and beats nothing. All measured on the
   oracle and matched by both engines. `99.927` is rc 157 because it is a translation error raised
   from install, as the doc says.
4. **The four-row progression.** Real. Rows 1, 2 and 4 measured; row 3 established from the source at
   `d0b7bd151`, where `abstract_` has no reader outside `rexx-parse`. The pinned build does refuse,
   which is why it cannot be the "before" for the previous task and the report does not use it as one.
5. **The frame-line rule** (`rust/crates/rexx-exec/src/dispatch.rs:852`-`:880`, `121bc720d`). It
   states a rule -- "the line is owed wherever the oracle reached the failing native method's body by
   a message send" -- and names no caller list. Both citations check out: `ClassDirective.cpp:230` is
   `classObject->sendMessage(GlobalNames::INHERIT, ...)`, `:205` and `:200` are direct calls to
   `subclass()`/`mixinClass()`, and `StemClass.cpp:280` is `value->messageSend(...)`, so the stem
   caller fits the amended rule rather than being an exception to it. This task's three refusals fit:
   98.908 is raised in `install`'s own body, 99.927 inside `subclass()` reached by a call, 98.990
   inside `makeAbstract()` reached by `classObject->makeAbstract()` (`:249`) -- none carries a frame,
   measured. **And the case that discriminates the amended rule from the one it replaced:** the same
   native body reached by a send. Measured, `say .Object~subclass('X', .Object)` is 99.927 rc 157
   with `       *-* Compiled method "SUBCLASS" with scope "Class".` at the top of stderr. The old
   rule predicted a frame for the directive route and was wrong; the amended rule predicts both. Note
   for whoever lands `~subclass` as a message: nothing in the crate enforces that its caller then
   pays the line.
6. **The deleted program.** The reasoning holds; worked through above under Strengths.
7. **No `ir_dual_cases` stanza.** Acceptable conclusion, wrong stated reason (Minor above). The part
   of the argument that is right is that install is engine-independent; the part that matters and was
   not said is that the rewritten table D probe already asserts engine agreement unconditionally.
   Note for the record: `corpus.rs` runs `Invocation::none()`, which is `Engine::Ir`
   (`invocation.rs:160`), so the corpus alone does not hold the tree-walker for anything.
8. **The two gate-table C control sentences.** Both claims verified. Measured, both probes are rc 120
   `rexx-exec: method "ID" of class "Class" is not implemented (Phase 5)` at this head, so neither row
   can read `agree` in Task 8, and the plan's Task 9 (`2026-08-17-phase-5a.md:1068`-`:1071`) owns
   `~class`, `~id` and `~metaClass`. And the struck mutation genuinely cannot fire: the oracle answers
   `abstract AB Class` beside `object OC Class`, so a build ignoring `ABSTRACT` produces identical
   bytes for `typcla`.

## What I could not verify

* The five gate statuses, the 135-of-135 corpus run, `fmt` and `clippy` -- not re-run, per the
  dispatch. The arithmetic behind 135 does check out from the manifests (31 + 12 + 12 + 80).
* Every row of report section 6's mutation table. Applying a mutation means mutating the checkout,
  which this review may not do. The two mutations behind the deleted program are consistent with the
  code as read, and the brief's negative control (`Side::Class` -> `Side::Instance`) must redden
  `class_metaclass.rex` given `cascade_build`'s shape, but neither was run.
* The confirming second performance sitting, which went to a throwaway file.
* Whether `121bc720d` owes a sitting.

## Assessment

**Task quality: Needs fixes.** The mechanism is right -- I probed fourteen shapes beyond the
committed nine and every one matches the oracle byte for byte on both engines, including the
`hasScope` corner that decides whether a metaclass donation reaches a class at all. What needs fixing
is prose that will be read as authority: `registry.rs`'s `class_of` doc is now false in the exact
case this task enabled and will make Task 9's `~class` answer wrongly, the exclusions record
over-generalises the override to `.Class`, and one count-plus-enumeration survived the sweep that
struck its twin.
