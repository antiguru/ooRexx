# Task 7 report: `::CLASS ... MIXINCLASS` and `::CLASS ... INHERIT`

Both keywords install on both engines, with the documented merge order. `~baseClass`
answers, so the mixin's own observable is reachable. The `INHERIT` refusal ladder matches
byte for byte, including the native-method traceback frame the install machinery's send
contributes, and it is measured wider than the brief asks: every condition a `::CLASS`
directive can reach, not only the shapes the brief names. (`RexxClass::inherit` raises two
more, `Error_Execution_rexx_defined_class` and `Error_Execution_uninherit`; neither is
reachable from a directive -- the first needs a receiver the image build marked Rexx-defined
and the second needs `~inherit`'s second argument, which `ClassDirective::install` never
passes.)

## What was built

**`rexx-classes`.**

* `ClassGraph::inherit` answers `Result<(), InheritRefusal>` where it used to `assert!`.
  `InheritRefusal` has one variant per `SYNTAX` condition `RexxClass::inherit` reports
  (`ClassClass.cpp:1298`-`:1332`): `NotAMixin` (98.942), `Recursive` (98.944) and
  `BaseClass(ObjRef)` (98.943, carrying the base class because the oracle's message names
  it). The graph refuses; the caller raises, because the messages substitute a class's
  `~defaultName` and the report carries a frame that belongs to whoever sent the message.
* `ClassGraph::base_class` / `ClassRegistry::base_class`, the accessor `~baseClass` reads.
* The `UNINIT` propagation flags: `has_uninit` and `parent_has_uninit` on `ClassDef`,
  oracle's `HAS_UNINIT` and `PARENT_HAS_UNINIT`. `ClassGraph::define` sets `has_uninit`
  for an instance method named `UNINIT`, which is `defineMethod`'s own line
  (`ClassClass.cpp:852`-`:857`). Propagation runs at the three constructors the oracle
  writes it at: `subclass` (`:1634`) and `mixinClass` (`:1525`), which are the two arms of
  `define_class` here, and `inherit` (`:1364`), at the tail of that function and not on a
  refusal.

**`rexx-exec`.**

* `directive_gap` no longer names `MIXINCLASS` or `INHERIT`. The namespace arm generalised
  from `SUBCLASS ns:` to any class reference on the directive carrying a namespace, and its
  message with it (`::CLASS naming a namespace`) -- a message naming only `SUBCLASS` would
  have misnamed the two new shapes it now catches. `METACLASS` keeps its own arm and its own
  message.
* `install_class` takes its `ClassKind` from `class.mixin`, not from the presence of the
  shared `subclass` slot. That is M10, and its discriminator is `~baseClass`.
* `install_class_at` follows `ClassDirective::install`
  (`interpreter/instructions/ClassDirective.cpp:165`-`:229`): resolve the
  `SUBCLASS`/`MIXINCLASS` target, create the class, then walk the `INHERIT` list left to
  right, one `INHERIT` send per entry. Target resolution moved into
  `Interp::resolve_class_target`, which both the subclass slot and the inherit list use, so
  a target that resolves nowhere is one 98.909 rather than two copies of it.
* `Interp::inherit_mixin` raises the refusal, preceded by
  `Interp::blame_native_method(b"INHERIT", scope)` and then `Interp::blame_directive`, which
  is the two-echo shape the failing `::CONSTANT` path already uses. The scope string is read
  from the registry (`root_and_metaclass().1`, then `id_string`) rather than written down.
* `class_install_order`'s dependency set is now every class reference the directive names
  that the file also declares, unqualified -- `class_dependencies`, matching
  `ClassDirective::addDependencies` (`:327`-`:344`) and `checkDependency`'s own
  `isQualified` skip (`:294`-`:310`). Cycle detection generalised from one target to a set
  and is otherwise unchanged.
* `NATIVE_METHODS` gains `("Class", "BASECLASS", 0, native_base_class)`. `BASECLASS` was
  already in `.Class`'s own dictionary (derived from `Setup.cpp:455`'s
  `AddProtectedMethod`); what was missing was an implementation for the `MethodId` it mints.

**`phase-4-exclusions.txt`.** `MIXINCLASS` and `INHERIT` move out of its refused list into the
runs-here list, and a closed-defect entry beside `SUBCLASS`'s records the merge order, the
refusal ladder, which rungs carry the frame, and the corpus witnesses. The namespace rows
gain the `MIXINCLASS ns:` and `INHERIT ns:` spellings with the oracle's 98.987 measured for
each.

**Why `blame_stem_forwarded_operator` is not reused.** Its doc says a caller is "an adapter
that evaluates one operator whose receiver is `value`", and its body returns immediately
unless `is_stem_receiver(value)`. This task's caller is neither: the receiver is a class
object and there is no operator. What both share is the primitive underneath,
`Interp::blame_native_method` in `dispatch.rs`, which is already `pub(crate)` and is what
`inherit_mixin` calls. No list was added to and no rule was bent.

## The five gate commands

Run from `rust/` on the **committed** tree -- `git status` clean at `c1510a936`, nothing edited
between starting the run and reading it -- with each status read **unpiped**: the harness does
`echo "GATE<n> <command> exit=$?"` immediately after each command, with that command's own
output redirected to a file rather than piped, so no pipeline's status can stand in for the
command's.

```text
GATE1 cargo fmt --all --check exit=0
GATE2 cargo clippy --workspace --all-targets -- -D warnings exit=0
GATE3 cargo test --release --workspace exit=0
GATE4 REXX_CORPUS_GATE=1 cargo test --release --workspace exit=0
GATE5 REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast exit=0
```

`/bin/grep -E "^(failures:|test result: FAILED)"` over gates 3, 4 and 5's captured output
matches nothing, so gate 5's `--no-fail-fast` did not hide a failure behind a zero status. The
corpus differential reads **127 of 127 matching** in STRICT mode in both gate 4 and gate 5; it
read 118 before this task, and the nine programs listed below are the difference.

**Why this is the second five-gate run, and the first is not the one reported.** An earlier
run of the same five was also all-zero, but between it and the commit I applied a fourth
mutation to `lib.rs` for the "beyond the brief" section and restored it from a copy -- so the
gated tree and the committed tree were not *provably* the same. They are in fact identical
(`crates/rexx-exec/src/lib.rs` as committed and the copy taken from the gated tree both sha256
to `d5eab40a3ded9ffa0d2b4a9b0cf84b1f42302e13a523e3bc84e625c60c13f753`), but that is a check
after the fact rather than a run, so the run above was made on a tree untouched since.

`memcap` is present on this machine (`/home/moritz/.local/bin/memcap`), checked with
`command -v` rather than assumed, so gate 5 is the `memcap` form and not the `ulimit -v`
substitute.

## The performance sitting

Run at `6aa432f19`, the commit this task lands, against the pin checked below. `rexx-run`'s
own sha256 at that commit is
`4b0a696cf0ea63606f8f321fe7c3b4b2d9669c2d4d5301eaa85204a56bb7726d`.

```bash
./target/release/rexx-arms --build pinned=bench-baselines/pinned/rexx-run-15a1ffa98 \
                           --build head=target/release/rexx-run \
    --axis alloc4c --axis arith --axis compound --axis emptyloop --axis strings --axis varlookup \
    --rounds 5 --task 7 --commit 6aa432f19 --baseline bench-baselines/phase-5a-arms.tsv
```

exit 0, 312 rows appended to `bench-baselines/phase-5a-arms.tsv`, which is the file named for
this pin. Committed as `c1510a936`, separately from the code so that its message can quote the
figures the file holds.

**`pinned>head across_builds instructions:u`**, every row transcribed from the TSV as
`median [min..max]`, `k=5` throughout:

| axis | tw small | ir small | tw large | ir large |
|---|---|---|---|---|
| `alloc4c` | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000001] | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] |
| `arith` | 1.001006 [1.001006..1.001006] | 1.001284 [1.001283..1.001284] | 1.001036 [1.001036..1.001036] | 1.001294 [1.001294..1.001294] |
| `compound` | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] |
| `emptyloop` | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] |
| `strings` | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] |
| `varlookup` | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] |

**`pinned>head across_builds cycles:u`**, the same rows:

| axis | tw small | ir small | tw large | ir large |
|---|---|---|---|---|
| `alloc4c` | 1.052375 [0.909543..1.102125] | 1.001532 [0.972306..1.088736] | 1.021470 [0.967047..1.042255] | 0.972850 [0.940234..0.987893] |
| `arith` | 1.001318 [0.987623..1.006770] | 0.989924 [0.983003..0.992873] | 0.994084 [0.991531..0.998077] | 0.982806 [0.974797..0.985337] |
| `compound` | 0.993062 [0.891517..1.000692] | 0.999198 [0.965846..1.007579] | 1.009435 [0.993595..1.110414] | 0.995977 [0.984059..1.004869] |
| `emptyloop` | 0.994845 [0.992608..1.008931] | 0.977586 [0.968736..0.982780] | 0.998488 [0.985475..1.007121] | 0.976359 [0.971315..1.003872] |
| `strings` | 0.974233 [0.897285..1.064438] | 1.003306 [0.994126..1.035335] | 0.980708 [0.946171..1.004435] | 1.004585 [1.001829..1.064312] |
| `varlookup` | 0.992233 [0.986238..0.997609] | 0.995558 [0.991391..1.000147] | 0.992145 [0.989498..0.998087] | 0.990112 [0.986641..1.001017] |

**What the numbers mean for this task's code: nothing reaches it.** The threshold is 1% on
`instructions:u`, and no benchmark in the axis list executes a single line this task changed --
checked rather than assumed: `/bin/grep -cE '^[[:space:]]*::'` over each of
`bench-programs/{alloc4c,arith,compound,emptyloop,strings,varlookup}.rex` answers `0`, so none
of them installs a directive at all. (`alloc4c.rex` does contain the characters `::`, once, in
a prose comment naming `Heap::Slot` -- which is why the check is anchored to the start of a
clause and not a bare `grep -c '::'`.) So there is no axis on which the change could show up as
work, and the `arith` row is layout.

**One word in `c1510a936`'s message reads stronger than the rows support.** It says the `arith`
ratio "is deterministic across all five rounds"; the `ir small` cell is `1.001283..1.001284`, so
read that as "reproduces to five decimals" rather than "identical". The commit message cannot be
edited, so it is corrected here.

## The pin, checked before the sitting

`bench-baselines/pinned/rexx-run-15a1ffa98` has sha256
`141c3fa968ea774655bc00f3e3b9f61cf1c4b738fd2793831d0055e94f1d074b`, which is the value
`bench-baselines/PINNED.md` records for it. The staleness test:

```bash
git log --oneline 15a1ffa98..HEAD -- rust/crates rust/Cargo.toml rust/Cargo.lock Cargo.toml
```

run with `HEAD` at this task's base, `c27b46ea4`, lists commits from `d5647aa4d` up to
`c27b46ea4`. Each one was looked up by hash in
`.superpowers/sdd/2026-08-17-phase-5a/`, and every one of them is named there -- in a task
report, a review, a fix-round brief or a committed review diff. No foreign commit sits under
the phase, so the pin is live.

## The merge-order program, both engines

```rexx
say .K~m
::CLASS P
::METHOD m CLASS
  return 'parent'
::CLASS M1 MIXINCLASS Object
::METHOD m CLASS
  return 'mixin'
::CLASS K SUBCLASS P INHERIT M1
```

| side | rc | stdout | stderr |
|---|---|---|---|
| oracle | 0 | `parent` | empty |
| `REXX_ENGINE=ir` | 0 | `parent` | empty |
| `REXX_ENGINE=tree-walker` | 0 | `parent` | empty |

The mixin winning is the negative control and it does not happen. The variant where `K`
defines its own `m` answers `own` on all three, which is green under any merge order and is
in the corpus beside it for exactly that reason -- it says the program is measuring the
order rather than the presence.

## The diamond

```rexx
say .Combo~methodx
say .Combo~methody
::CLASS MixinBase MIXINCLASS Object
::METHOD methodx CLASS
  return 'BASE_X'
::METHOD methody CLASS
  return 'BASE_Y'
::CLASS MixinA MIXINCLASS Object INHERIT MixinBase
::METHOD methodx CLASS
  return 'A_X'
::CLASS MixinB MIXINCLASS Object INHERIT MixinBase
::METHOD methodx CLASS
  return 'B_X'
::CLASS Combo SUBCLASS Object INHERIT MixinA MixinB
```

| side | rc | stdout |
|---|---|---|
| oracle | 0 | `A_X` then `BASE_Y` |
| `REXX_ENGINE=ir` | 0 | `A_X` then `BASE_Y` |
| `REXX_ENGINE=tree-walker` | 0 | `A_X` then `BASE_Y` |

The first-listed mixin wins the overridden name, and the shared ancestor's own untouched
method comes through once and undamaged.

**This diamond does not discriminate a chain walk, and the corpus program carries a second
one that does.** See control 2 below: `class_inherit_order.rex` also declares `Combo2`, where
only the *second*-listed mixin overrides the shared ancestor's method. Oracle `B_ONLY`, on
both engines here.

## The three refusal shapes, byte for byte

Read as three descriptors, one process each, from a fresh empty directory. `<dir>` below
stands for the probe directory's absolute path, which both sides print identically because
both are handed the same absolute path.

**`::class d inherit c` with no `c`.** rc 158 on all three sides, stdout empty:

```text
     2 *-* ::class d inherit c
Error 98 running <dir>/inherit_nosuch.rex line 2:  Execution error.
Error 98.909:  Class "C" not found.
```

No frame line, and that is the oracle's own: `ClassDirective::install` resolves each
`INHERIT` target (`:214`-`:219`) before it sends `INHERIT`, so nothing has been sent yet.

**The same with `::class c` declared.** rc 158 on all three sides, stdout empty, and the
report opens with the frame line this task owes:

```text
       *-* Compiled method "INHERIT" with scope "Class".
     3 *-* ::class d inherit c
Error 98 running <dir>/inherit_notmixin.rex line 3:  Execution error.
Error 98.942:  Class "The C class" must be a MIXINCLASS for INHERIT.
```

**`::CLASS K INHERIT M PUBLIC`.** rc 158 on all three sides, stdout empty:

```text
     3 *-* ::CLASS K INHERIT M PUBLIC
Error 98 running <dir>/inherit_public.rex line 3:  Execution error.
Error 98.909:  Class "PUBLIC" not found.
```

`dire.xml`'s "INHERIT must be last" is not a syntax check: the arm consumes to the end of the
clause and `PUBLIC` is read as one more class name.

**The rest of the ladder, measured because the code reaches it and a panic was the
alternative.** Each rc 158, stdout empty, oracle and both engines identical:

| program | report |
|---|---|
| `::CLASS M MIXINCLASS Object` / `::CLASS K INHERIT M M` | frame line, then `Error 98.944:  Class "The K class" cannot inherit from itself, a superclass, or a subclass ("The M class").` |
| `::CLASS P` / `::CLASS M MIXINCLASS P` / `::CLASS K INHERIT M` | frame line, then `Error 98.943:  Class "The K class" is not a subclass of "The M class" base class "The P class".` |
| `::CLASS K INHERIT K` | no frame line, `Error 98.911:  Cyclic inheritance in program "<path>".` |

The last one is the install ordering, not the send: an `INHERIT` target is a dependency
exactly as a `SUBCLASS` target is, so a class inheriting itself cannot be placed. It is the
witness for the `class_install_order` generalisation.

## `~baseClass`

| program | oracle | `ir` | `tree-walker` |
|---|---|---|---|
| `say .M~baseClass`, `M MIXINCLASS Object` | `The Object class` | same | same |
| `say .P~baseClass`, `P` bare | `The P class` | same | same |
| `say .S~baseClass`, `S MIXINCLASS Class` | `The Class class` | same | same |

"same" here is `cmp` over all three descriptors, not a reading of the table.

The third row is Task 8's D44 witness becoming reachable: `::CLASS S MIXINCLASS Class` was
`rc 120` before this task and now installs and agrees.

## `METACLASS` still refuses, and names only what it still refuses

```rexx
say 'main'
::CLASS S MIXINCLASS Class
::CLASS K METACLASS S
```

Oracle rc 0 `main`; this crate rc 120, stdout empty, stderr
`rexx-exec: ::CLASS METACLASS is not implemented (Phase 5)`. The `MIXINCLASS` directive above
it installs -- measured on its own, `say .S~baseClass` answers -- so the message is not
standing in for a second construct.

The namespace arm's message changed with its predicate, for the same reason: it now catches
`MIXINCLASS ns:` and `INHERIT ns:` as well as `SUBCLASS ns:`, and reads `::CLASS naming a
namespace`. Measured, the oracle answers 98.987 `Namespace "NS" not found in package "<path>"`
at rc 158 for all three spellings, so the target is unreachable there too and what diverges is
the report rather than the outcome.

## The UNINIT flag propagation's in-crate test

`crates/rexx-classes/tests/behaviour_wiring.rs`:
`uninit_propagates_through_all_three_constructors`, with
`a_refused_inherit_propagates_no_uninit_flag` beside it.

The test drives `ClassGraph` directly. It asserts the flag arrives through `subclass`,
through `subclass` again one generation further down (which is what the oracle's
`hasUninitDefined() || parentHasUninitDefined()` pair buys over a one-level test), through
`mixinClass`, and through `inherit`; and it asserts the negative for each of them, so a
build that sets the flag on every class fails it.

**No differential and no gate row, and the reason is not that the flag cannot be observed.**
It can be, class-side, with no `~new` anywhere -- see the three programs below. What is
missing is the collector that fires `UNINIT`, which is 5b's. So the in-crate test is the only
instrument, which the brief says and which control 3 measures rather than assumes.

## The three UNINIT programs, for the 5b handover

Each `::METHOD uninit CLASS` says `self~id`. One process each, three descriptors separate.
The "before" column is the pinned `rexx-run-15a1ffa98`, a build that predates this task.

| program | oracle | this crate before | this crate now |
|---|---|---|---|
| `::CLASS K` with the method | rc 0, `main` then `uninit on K` | rc 0, `main` alone | rc 0, `main` alone |
| the same on `::CLASS P` with `::CLASS K SUBCLASS P` | rc 0, `main`, `uninit on K`, `uninit on P` | rc 0, `main` alone | rc 0, `main` alone |
| the same on `::CLASS M MIXINCLASS Object` with `::CLASS K INHERIT M` | rc 0, `main`, `uninit on K`, `uninit on M` | **rc 120, `rexx-exec: ::CLASS MIXINCLASS is not implemented (Phase 5)`** | rc 0, `main` alone |

**The third row is a debt this task creates.** Its shape changed from a loud refusal, which
cannot be mistaken for an answer, to a silent wrong answer at rc 0. 5b owns the firing, so the
gap itself is correct; what changed is that it stopped announcing itself. All three programs
travel to 5b together.

## The three controls, each run

### 1. Table D's mutation 1, relocated here: the M10 narrowing keyed on the slot

**The mutation, spelled so it can be applied.** In `Interp::install_class`, replace

```rust
let kind = if class.mixin {
    ClassKind::Mixin
} else {
    ClassKind::Regular
};
```

with a narrowing that consults the shared `subclass` slot instead of the `mixin` flag and so
cannot tell the two keywords apart -- taking the branch that treats every `::CLASS` naming a
target as an ordinary subclass:

```rust
let kind = if class.subclass.is_some() {
    ClassKind::Regular
} else {
    ClassKind::Regular
};
```

That is what the plan's M10 line describes: `ClassDirective` holds `SUBCLASS` and `MIXINCLASS`
in one slot, so a reader who goes by the slot builds a `MIXINCLASS` directive as a plain
subclass of its target. **A same-polarity substitution -- `subclass.is_some()` where
`class.mixin` stood, keeping both arms -- does not reproduce this**, because a `MIXINCLASS`
directive fills the slot too and would still come out `Mixin`; the reviewer was right to say so.
Rebuilt release, then re-ran gate table D:

```text
before  agree           ::CLASS  MIXINCLASS  subkeyword  class__mixinclass__subkeyword.rex
after   diverge-stdout  ::CLASS  MIXINCLASS  subkeyword  class__mixinclass__subkeyword.rex
          oracle rc=0    out="The Object class\n" err=""
          crate  rc=0    out="The K class\n"      err=""
```

exactly the flip the brief predicts. The `INHERIT` row moves with it, to `diverge-both` --
under the mutation the mixin is no longer a mixin, so the `INHERIT` send is refused 98.942
where the oracle answers `from the mixin`. `REXX_CORPUS_GATE=1 cargo test --release --test
corpus` exits **101** under the mutation, against 0 without it.

**The two table D probes had to gain their discriminators for this to be a control at all.**
`class__mixinclass__subkeyword.rex` and `class__inherit__subkeyword.rex` both said `main` and
nothing else, which agrees with the oracle under any implementation that installs the keyword
and ignores it. They now say `.k~baseClass` and `.k~m` respectively -- the row's own subject,
one line each, which is what `expected_oracle_lines` requires.

### 2. Walking the chain instead of merging reddens the diamond

`ClassRegistry::lookup_class_method` repointed at a `ClassGraph::walk_lookup` that resolves by
walking `superclasses` depth-first at lookup time instead of reading the merged behaviour.
`REXX_CORPUS_GATE=1 cargo test --release --test corpus` exits **101**, and the diamond line is
what moves:

```text
[UNCLASSIFIED] lang/class_inherit_order.rex: stdout differ
    rust:   stdout="M1\nM2\nA_X\nBASE_Y\nBASE_X\nforward\n"
    oracle: stdout="M1\nM2\nA_X\nBASE_Y\nB_ONLY\nforward\n"
```

**The diamond as first written did not catch it, and neither did the mutation I reached for
first.** Two findings, both from running rather than reasoning:

* Removing `cascade_build`'s two `has_scope` guards -- the obvious "stop merging once" mutation
  -- leaves the whole corpus green. `MethodDict::add_method` replaces a same-scope entry in
  place, so re-walking a shared ancestor re-adds its methods at the position they already hold
  and nothing moves. The diamond guard is not observable through method resolution at all with
  this dictionary; what it buys is the scope ordering and the work saved.
* The `Combo` diamond (both mixins override) answers `A_X` under a chain walk too, because the
  walk reaches `MixinA` before `MixinB`. What separates them is a diamond where only the
  *second*-listed mixin overrides: the merge folds the shared ancestor in at the first walk's
  position and lets the later override stand (`B_ONLY`), where the walk finds the ancestor
  through the first mixin and never reaches the second (`BASE_X`). That is `Combo2`, added to
  `class_inherit_order.rex` for this control, and it is the only line of that program that
  moves -- the `A_X` and `BASE_Y` lines above it agree under the mutation.

The mutation also reddens `class_mixinclass.rex`, `method_class_side_lookup.rex` and
`message_send_argument_object_not_a_string.rex`. That is the "can fail is not adds coverage"
answer for this control, and it does not need a second run to see: those are different files,
so whether `class_inherit_order.rex` declares `Combo2` cannot change what they report. The
chain walk would have been caught without `Combo2`. What `Combo2` adds is a witness for a
merge-order defect confined to a diamond -- something no program in the corpus could see
before, since none of the others declares one.

### 3. Dropping the flag propagation from one of the three constructors

One at a time, each rebuilt and run:

```text
drop propagation from subclass     uninit_propagates_through_all_three_constructors FAILED
drop propagation from mixinClass   uninit_propagates_through_all_three_constructors FAILED
drop propagation from inherit      uninit_propagates_through_all_three_constructors FAILED
unmutated                          uninit_propagates_through_all_three_constructors ok
```

And the "can fail is not adds coverage" half: with the `inherit` propagation dropped, a whole
`cargo test --workspace --no-fail-fast` reports exactly one failure, that test. Nothing else in
the workspace can see the flag, which is the claim the brief makes and this measures.

## What instrument catches a regression in what changed about refusals

`MIXINCLASS`'s and `INHERIT`'s rows leave
`every_directive_this_crate_cannot_install_refuses_before_the_first_clause` and
`a_class_keyword_gap_is_raised_inside_the_class_pass`, and a corpus program can never hold
them: rc 120 against the oracle's rc 0 is not a differential row. What replaces them:

* the two forms move into
  `every_directive_this_crate_can_install_leaves_the_program_alone`, whose bound is rc 0,
  `main ran`, empty stderr -- a build that refused either again fails there;
* the corpus programs listed below, which are differential rows the corpus gate reads;
* table D's `MIXINCLASS` and `INHERIT` rows, now `agree` and now carrying discriminators.

**The replacement can fail the same way the deleted rows could.** The deleted rows caught a
build that refuses a directive the oracle installs. The `can_install` rows catch exactly that,
and control 1 shows the corpus and table D catch the opposite failure -- a build that installs
it wrongly -- which the deleted rows could not.

The namespace rows are the honest remainder: `::CLASS naming a namespace` is a loud refusal
where the oracle raises 98.987, and no corpus row can express it. Its instrument is the
in-crate refusal test only.

## Corpus programs added

Every one of them is in `corpus/phase-5a.txt`, in `coverage.rs`'s `EXPECTED_SUBSET_5A`, and --
since every one of them allocates nothing -- in `collect_stress.rs`'s
`NO_ALLOCATION_PROGRAMS`. Each has its
`crates/rexx-parse/tests/sourceline_oracle/<name>.txt` generated with that test's own
documented driver.

| program | what it pins |
|---|---|
| `lang/class_mixinclass.rex` | merge order (`parent`), the own-method control (`own`), `~baseClass` for a mixin, a plain class, a subclass and a mixin of a mixin |
| `lang/class_inherit_order.rex` | leftmost-first in both directions, the diamond, the second diamond control 2 needs, a forward `INHERIT` |
| `lang/class_inherit_not_found.rex` | 98.909, no frame |
| `lang/class_inherit_trailing_keyword.rex` | 98.909 naming `PUBLIC` |
| `lang/class_inherit_not_a_mixin.rex` | 98.942, with the frame |
| `lang/class_inherit_base_class.rex` | 98.943, with the frame |
| `lang/class_inherit_recursive.rex` | 98.944, with the frame |
| `lang/class_inherit_cycle.rex` | 98.911 from the generalised install ordering |
| `lang/class_metaclass_cycle.rex` | 98.911 from the `METACLASS` dependency edge |

## Everything beyond the brief, and what each of them discriminates

A fourth mutation was run for this section rather than for the brief's control list:
`class_dependencies` narrowed back to the `SUBCLASS` slot alone, which is what the
generalisation replaced. `REXX_CORPUS_GATE=1 cargo test --release --test corpus` reads
**124 of 127** under it, and the three that move are named in the table below.


The brief names three refusal shapes; this task measured and committed more, and it committed
more corpus programs than the brief asks for at all. Asked for by the lead, and the honest
answer for each -- "nothing, it was cheap" where that is what it is.

| beyond the brief | what it discriminates that the named shapes do not |
|---|---|
| 98.944, `::CLASS K INHERIT M M` | the only program that constructs `InheritRefusal::Recursive`. A build with the wrong catalogue number or the wrong substitution order on that arm is green without it, and a directive reaches the arm, so before this task the alternative was a panic. |
| 98.943, `::CLASS K INHERIT M` under `M MIXINCLASS P` | the only program that constructs `InheritRefusal::BaseClass`, and the only one that reads the base class the refusal carries -- three substitutions, the third being what `~baseClass` answers. Same panic argument. |
| 98.911, `::CLASS K INHERIT K` | one of the three witnesses for the dependency-set generalisation, measured: with `class_dependencies` narrowed back to the `SUBCLASS` slot alone, this program is `rc 120 rexx-exec: an activation's body selector names no routine body` against the oracle's 98.911. **My prediction for this one was wrong and running it is what said so** -- I expected 98.944 from `inherit(K, K)`, but `K` is never installed, so `resolve_class_target` reaches `Loud::missing_body` first. |
| 98.911, the `METACLASS` pair | the third, and the only one that reaches the `METACLASS` edge: under the mutation it is `rc 120 ::CLASS METACLASS is not implemented (Phase 5)`, which the corpus classifies as a loud failure rather than an unexplained one. |
| `class_inherit_order.rex`'s `Combo2` | control 2's own subject: a merge-order defect confined to a diamond. Measured -- the chain-walk mutation reddens three other programs, so it is not needed to catch *that*, and no other corpus program declares a diamond at all. |
| `class_inherit_order.rex`'s forward `INHERIT` | the second witness for the same generalisation: under that same mutation the whole program is `rc 120` with the same loud message, against the oracle's six lines. |
| `class_mixinclass.rex`'s `~baseClass` rows for a subclass and for a mixin of a mixin | nothing the `MIXINCLASS` and plain rows beside them do not already reach. They were cheap, they are one `say` each in a program that had to exist anyway, and they are the shapes `ClassGraph::define_class`'s `base_class` arms actually compute. |
| `class_mixinclass.rex`'s `own` row | named by the brief as the merge order's negative control, so not an extra. |

## What I could not close

**The `UNINIT` flags are right at the graph's own API and not yet right through the directive
path, and the reason is this crate's install order.** `Interp::install_directives` creates
every class first and attaches methods in a second pass, where the oracle attaches a class's
methods at construction time (`ClassDirective` holds them before `install` runs). So for
`::CLASS P` / `::METHOD uninit` / `::CLASS K SUBCLASS P`, `P`'s `has_uninit` is set after `K`
is constructed, and `K`'s `parent_has_uninit` stays false where the oracle's is true. Nothing
observes it today -- 5b owns the firing -- and nothing in this task's scope closes it: the
oracle's own `defineMethod` does not cascade the flag either, so a propagation added at
`define` would be unfaithful, and the faithful fix is to attach a class's methods before its
dependents are constructed, which is a restructure of `install_directives` well outside this
task. It travels to 5b with the three programs above.

This is the same shape as `phase-4-exclusions.txt`'s standing open row, "A ::CONSTANT
EXPRESSION CANNOT SEND TO A CLASS DECLARED LATER IN THE SAME FILE": the oracle has every class
installed with its methods before it evaluates anything, and this crate does not. The fix that
closes both is the one that attaches a class's methods when the class is constructed. A
narrower fix for either -- evaluating `::CONSTANT` expressions in a later pass, say -- need not
close the other, so this is a shared cause and not a shared fix.

**`ClassDirective::addDependencies` includes `metaclassName` and so does
`class_dependencies`, which is a change to a construct that stays refused -- and it does have a
witness, which the first draft of this line said it did not.** Measured: `::class a metaclass
b` / `::class b metaclass a` is rc 158 with the oracle's own `98.911` bytes on both engines,
against `rc 120 rexx-exec: ::CLASS METACLASS is not implemented (Phase 5)` on the pinned
`rexx-run-15a1ffa98`. It is `corpus/lang/class_metaclass_cycle.rex`. The reasoning that nearly
left it unwitnessed is worth keeping: the construct is refused, so it looked like nothing
observable could depend on the edge -- but the *ordering* failure is diagnosed before any class
is created and therefore before the refusal is reached.

**One process mistake worth recording.** Removing a mutation from `registry.rs` with
`git checkout -- <path>` discarded this task's own uncommitted edits to that file along with the
mutation, and the loss was silent -- the build failed several steps later, not at the checkout.
`rust/CLAUDE.md` already says to restore from a copy rather than from git; this is the second
half of that sentence being true. The edits were re-applied by hand and the file is covered by
the gate runs below rather than by a byte comparison -- there was no copy left to compare
against, which is the point.

The later mutations were restored from a copy instead, and that one *is* checkable:
`crates/rexx-exec/src/lib.rs` as committed and the copy taken from the tree the first
five-gate run saw both sha256 to
`d5eab40a3ded9ffa0d2b4a9b0cf84b1f42302e13a523e3bc84e625c60c13f753`.

---

# Fix round 1

Both Important findings were UNINIT and both are closed. Finding 1 took the narrow branch: it
works, and it does **not** need the install reordering. Finding 2's ruling is taken as written --
the doc says what our flag is, and the assertion that froze a divergence is unpinned.

**The brief's suggested shape for the narrow fix would have encoded a wrong semantics, and this
was measured before building rather than after.** The brief says "set `has_uninit` where a class
method named `UNINIT` is attached". The oracle does not do that:

* `RexxClass::checkUninit` (`ClassClass.cpp:1210`-`:1218`) sets `HAS_UNINIT` from the class's
  **flattened instance** behaviour;
* the class-side spelling instead reaches `hasUninitMethod` and `requiresUninit` at `:1222`,
  which enters the class **object** in the collector's uninit table.

Measured on the oracle, one process each, three descriptors:

| program | oracle |
|---|---|
| `::CLASS P` + instance `::METHOD uninit` + `::CLASS K SUBCLASS P`, then `o = .K~new` | rc 0, `made` then `instance uninit, scope P` |
| `::CLASS K` + instance `::METHOD uninit`, then `o = .K~new` | rc 0, `made` then `instance uninit, scope K` |
| `::CLASS K` + `::METHOD uninit CLASS`, then **two** `.K~new` | rc 0, `made two` then `uninit on K` -- **once** |

The first row is finding 2 confirmed by running rather than by reading: an *inheriting* class
carries `HAS_UNINIT`, which is why its instance runs the ancestor's `uninit`. The third row is
why the brief's shape was not taken: a class-side `uninit` registers one object, the class
itself. Setting `has_uninit` there would have made every instance that class ever creates
register for an `UNINIT` the oracle does not run for it.

## What was built

* **`ClassGraph::check_uninit`** -- the oracle's `checkUninit`, the half this crate can model:
  set `has_uninit` when the **flattened instance** behaviour answers `UNINIT`, whether the class
  defines it or inherits it. One-way and idempotent, like the oracle's. Its doc names the other
  half (`requiresUninit` on the object, `:1222`) as a mechanism this crate does not have, so the
  function is not read as covering it.
* **`ClassGraph::refresh_parent_has_uninit`** -- recomputes `parent_has_uninit` from the current
  superclass list. The oracle has no such function and the doc says why one is needed here: the
  oracle attaches a class's methods while constructing it, so its three constructors read a
  finished parent, and this crate does not.
* **`Interp::install_directives`** runs the pair over the file's classes **in `order`** -- the
  dependency order, so a parent is finished before its child -- after the method pass, `check_uninit`
  first and the propagation second, which is the oracle's order within a class. This is the same
  shape and the same reason as the `refresh_class_behaviour` loop already beside it.

**No reordering of install was needed**, so the stop-and-report branch does not apply. The
two-pass structure stays exactly as it was; what changed is that the flags are computed at a
point where the inputs are complete instead of at a point where they are not.

## The three things owed, on the branch taken

* **The graph-API test is renamed** `uninit_propagates_through_all_three_constructors_at_the_graph_api`,
  and its doc now says it calls `define` before `define_class`, an order the directive install
  never produces, and points at the `rexx-exec` test for the other half. It cannot be read as
  evidence about programs.
* **The handover record** carries that `parent_has_uninit` was `false` for every declarable class
  before this round, with the two-pass install as the reason and `phase-4-exclusions.txt`'s
  `::CONSTANT` row as the precedent -- see "What travels to 5b" below.
* **The three handover programs** carry the finding that `::METHOD uninit CLASS` reaches
  `class_define` and sets neither flag, with the measurement above.

## Finding 2: the doc, and the unpinned assertion

`ClassDef::has_uninit`'s doc no longer claims to be the oracle's `HAS_UNINIT` by way of
`defineMethod` alone. It now names both oracle sites -- `defineMethod` (`:854`) and `checkUninit`
(`:1214`) -- says what the flag means (this class's *instances* need `UNINIT`), cites
`completeNewObject` (`:1892`) as the site that reads it, carries the measurement for the
inheriting case, and states that a class-side `::METHOD uninit CLASS` does not set it and what it
does instead.

`behaviour_wiring.rs`'s `assert!(!g.has_uninit(child))` is no longer a bare assertion that our
answer is right. It is now a pair: `false` **before** `check_uninit` -- a fact about this crate's
API, which does not call it from its constructors -- and the oracle's `true` **after** it, with
the divergence named as a divergence and the oracle measurement cited beside it.

## The in-crate test for the directive path, shown failing without the change

`rexx-exec`'s `the_uninit_flags_are_set_for_the_classes_a_file_declares` installs
`::class Base` with an instance `::method uninit`, `::class Kid subclass Base`,
`::class Grandkid subclass Kid`, and a `Plain`/`Plainkid` pair with no `uninit` anywhere. It
asserts `has_uninit` on all three of the first chain, `parent_has_uninit` on the lower two, and
`false` for both flags across the `Plain` pair.

```text
with the install_directives pass deleted   the_uninit_flags_are_set_for_the_classes_a_file_declares FAILED
with it                                    the_uninit_flags_are_set_for_the_classes_a_file_declares ok
```

`a_class_side_uninit_sets_neither_flag` sits beside it and pins the third measurement above: the
class-side spelling installs on the class side and sets neither flag.

## The three handover programs, before and after

Unchanged, which is the point: they exercise the class-object registration, not the flags.

| program | oracle | crate before this round | crate after, `ir` and `tree-walker` |
|---|---|---|---|
| `::CLASS K` + `::METHOD uninit CLASS` | rc 0, `main`, `uninit on K` | rc 0, `main` alone | rc 0, `main` alone |
| the same on `::CLASS P` with `::CLASS K SUBCLASS P` | rc 0, `main`, `uninit on K`, `uninit on P` | rc 0, `main` alone | rc 0, `main` alone |
| the same on `::CLASS M MIXINCLASS Object` with `::CLASS K INHERIT M` | rc 0, `main`, `uninit on K`, `uninit on M` | rc 0, `main` alone | rc 0, `main` alone |

**And the instance-spelling programs, which are the ones the flags govern**, join the handover
beside them. These are **loud**, not silent, so 5b meets a refusal rather than a wrong answer:

| program | oracle | crate, both engines |
|---|---|---|
| `::CLASS P` + instance `::METHOD uninit` + `::CLASS K SUBCLASS P`, `o = .K~new` | rc 0, `made`, `instance uninit, scope P` | rc 120, `rexx-exec: method "NEW" of class "Object" is not implemented (Phase 5)` |
| `::CLASS K` + instance `::METHOD uninit`, `o = .K~new` | rc 0, `made`, `instance uninit, scope K` | the same rc 120 |
| `::CLASS K` + `::METHOD uninit CLASS`, two `.K~new` | rc 0, `made two`, `uninit on K` | the same rc 120 |

## What travels to 5b

* **Before this round, `parent_has_uninit` was `false` for every class a program could declare,
  always** -- `install_directives` creates every class in one loop and attaches every method in
  the next, so the propagation ran against parents with no methods; and `UNINIT` occurs nowhere in
  the generated `setup_classes.rs`, so no native class carried it either. That is closed now for
  both flags, through the directive path, with an in-crate test that fails without the pass.
  `phase-4-exclusions.txt`'s "A ::CONSTANT EXPRESSION CANNOT SEND TO A CLASS DECLARED LATER IN THE
  SAME FILE" is the standing row with the same two-pass cause; this round did not close that one,
  which needs the methods attached at construction rather than a later pass.
* **`::METHOD uninit CLASS` sets neither flag, and that is correct.** It reaches
  `ClassRegistry::add_class_method` -> `ClassGraph::class_define`, the class side. The oracle's
  matching mechanism is `hasUninitMethod`/`requiresUninit` on the class object
  (`ClassClass.cpp:1222`), which this crate models nowhere -- there is no uninit table and no
  collector to drive it. 5b owns that, and the three class-side programs above are its witnesses.
* **What 5b can rely on now**: `has_uninit(C)` is true exactly when `C`'s flattened instance
  behaviour answers `UNINIT`, and `parent_has_uninit(C)` is true exactly when some ancestor of `C`
  carries either flag -- both for classes a file declares, not only for graph-API callers.

## Minors

* `behaviour_wiring.rs`'s `98.943` in the recursive-inherit doc is now `98.944`, matching the
  twin sentence in `class_graph.rs`.
* `class_references`' "exactly these three" is gone; it now says `checkDependency` asks about the
  same references and no others. `error.rs`'s "on three shapes" is gone; the shapes are listed,
  and the `METACLASS` pair joins them.
* `blame_native_method`'s `pub(crate)` rationale is a rule rather than a list: anything that
  reaches a native method's body and raises from inside it owes the line, and a send is only the
  most obvious way to get there.
* **`METACLASS ns:` is now covered in both directions.** Measured, the oracle answers 98.987 for
  it exactly as for the other three spellings. It has a row in
  `every_directive_this_crate_cannot_install_refuses_before_the_first_clause` and in
  `a_class_keyword_gap_is_raised_inside_the_class_pass`, the `directive_gap` arm's comment says
  the namespace arm is reached before the `METACLASS` one so a qualified metaclass target answers
  there, and `phase-4-exclusions.txt` names the fourth spelling in its refused list and in the
  closed-defect paragraph.

## Control 1's description

Rewritten above, in "Table D's mutation 1", to name the mutation someone else can apply. The
reviewer's objection was right: a same-polarity substitution keeping both arms does not reproduce
the flip, because a `MIXINCLASS` directive fills the shared slot too. The mutation that does is
the one that treats every `::CLASS` naming a target as a plain subclass, which is what M10's own
line describes.

## Fix round 1's five gate commands

Run from `rust/` on the tree this round commits, each status read **unpiped** -- the harness
echoes `GATE<n> <command> exit=$?` immediately after each command with that command's output
redirected to a file rather than piped.

```text
GATE1 cargo fmt --all --check exit=0
GATE2 cargo clippy --workspace --all-targets -- -D warnings exit=0
GATE3 cargo test --release --workspace exit=0
GATE4 REXX_CORPUS_GATE=1 cargo test --release --workspace exit=0
GATE5 REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast exit=0
```

Corpus **127 of 127 matching** in STRICT mode, reported by gate 4 and again by gate 5, unchanged
from the task's own round -- this round adds no corpus program, because nothing it changes is
expressible as a differential row. `/bin/grep -E "^(failures:|test result: FAILED)"` over gates
3, 4 and 5's captured output matches nothing.

## Fix round 1's performance sitting

Run at `c35ba11cc` against the same pin, exit 0, 312 rows appended to
`bench-baselines/phase-5a-arms.tsv` and committed as `d0b7bd151`. Every appended row carries the
`c35ba11cc` tag, checked by cutting the commit column out of the diff.

**`pinned>head across_builds instructions:u`**, transcribed from the TSV as
`median [min..max]`, `k=5` throughout:

| axis | tw small | ir small | tw large | ir large |
|---|---|---|---|---|
| `alloc4c` | 1.000000 [0.999999..1.000000] | 1.000000 [0.999999..1.000000] | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] |
| `arith` | 1.001006 [1.001006..1.001006] | 1.001284 [1.001283..1.001284] | 1.001036 [1.001036..1.001036] | 1.001294 [1.001294..1.001294] |
| `compound` | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] |
| `emptyloop` | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] |
| `strings` | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] |
| `varlookup` | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] | 1.000000 [1.000000..1.000000] |

The conclusion is the task round's, unchanged and for the same checked reason: no benchmark in
the axis list installs a directive, so none of them reaches the pass this round added. The
`arith` cells read what they read at `6aa432f19`, to six decimals.

---

# Fix round 2

Prose, one false sentence, one dead branch. The mechanism was accepted and is untouched.

## 1. The false sentence, replaced by what running it says

The comment claimed that without the pass, every assertion expecting `true` reads `false`. Run,
it does not: `has_uninit(base)` stays `true` because `ClassGraph::define` sets it when the
`::METHOD uninit` is attached, and the test stops at the `kid` row.

Measured, one deletion at a time, each rebuilt and run:

```text
delete the whole pass                     fails at has_uninit(kid)          "its subclass"
delete only check_uninit                  fails at has_uninit(kid)          "its subclass"
delete only refresh_parent_has_uninit     fails at parent_has_uninit(kid)
unmutated                                 ok
```

**Two distinct rows, not three** -- the whole-pass and `check_uninit`-only deletions redden the
same assertion, because `has_uninit(kid)` is asserted before `parent_has_uninit(kid)`. The
brief predicted three; what matters for the claim survives, and is what the comment now says:
each call in the pass is witnessed on its own, and `has_uninit(base)` witnesses neither, so an
assertion on the declaring class alone would not have caught the pass going missing.

## 2. The set-size phrases

Deleted rather than recounted, at all four sites:

* `class_graph.rs`'s `has_uninit` field no longer counts oracle sites. It states the rule --
  the flag is set when an instance method named `UNINIT` becomes reachable from the class,
  whether by being added to it or by being inherited -- and points at the two functions in this
  crate that decide it.
* `has_uninit`'s accessor doc says "see the field for what sets it".
* `refresh_parent_has_uninit`'s doc says "each site that propagates this flag -- see the field
  for which" rather than counting constructors.
* `lib.rs`'s test comment says "the ones that reach it through the flattened behaviour".

And a sweep of everything else written this round found two more of the same shape, both fixed:
"the two halves of that pass" in the comment item 1 replaced, and "each of the four has an
in-crate row" in `phase-4-exclusions.txt`. What is kept is measurements -- "two `.K~new`
instances print `uninit on K` once, not three times" is evidence and keeps its numbers.

## 3. The citation

`ClassClass.cpp:1214` is the third line of `checkUninit`'s comment; the `setHasUninitDefined()`
call is `:1217`. Corrected, and checked by reading the numbered range rather than by trusting
the correction.

## 4. The dead branch

`classes[index]` replaces the `let ... else { continue; }`. Checked before deleting rather than
after: the loop that fills `classes` walks the same `order`, inserts an entry for every index in
it, and nothing between the two loops removes one -- both loops are in `install_directives`, a
few dozen lines apart, so a reader can settle it without leaving the function. The comment now
says that, so the bare index is not a claim a later reader has to re-derive.

**The sitting tagged `c35ba11cc` still stands**, and no fresh one was run. The branch was
unreachable, so no path any benchmark takes changed; and the axis programs install no directive
at all, so they do not reach this pass whether the branch is there or not.

## What round 2 adds to the 5b handover

Two things the re-review established, both worth carrying because 5b will lean on them:

* **`inherit` reaches `checkUninit` too, indirectly** -- `updateSubClasses()` at
  `ClassClass.cpp:1361` into `:1052`. So `:1364`'s propagation is not the whole mixin story, and
  keying this crate's fix on the flattened behaviour is what gets that case right rather than
  luck. Measured: a `MIXINCLASS Object` with an instance `uninit`, inherited by `K`, fires on
  `.K~new`.
* **The post-pass cannot diverge from the oracle's incremental computation.** The oracle
  finishes each class -- create, `INHERIT`, `defineMethods` -- before starting the next, in
  dependency order, so both flags are a fixpoint; recomputing them once at the end reaches the
  same answer. Nine shapes measured and agreed, negatives included.

## Fix round 2's five gate commands

Run from `rust/` on the tree this round commits, each status read **unpiped** -- the harness
echoes `GATE<n> <command> exit=$?` immediately after each command with that command's output
redirected to a file rather than piped.

```text
GATE1 cargo fmt --all --check exit=0
GATE2 cargo clippy --workspace --all-targets -- -D warnings exit=0
GATE3 cargo test --release --workspace exit=0
GATE4 REXX_CORPUS_GATE=1 cargo test --release --workspace exit=0
GATE5 REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast exit=0
```

Corpus **127 of 127 matching** in gate 4 and again in gate 5.
`/bin/grep -E "^(failures:|test result: FAILED)"` over gates 3, 4 and 5's captured output
matches nothing.

**No sitting this round**, per the ruling: the only executable change is replacing an
unreachable `let ... else { continue; }` with the index it always took, and the axis programs
install no directive, so they never enter that loop at all. The sitting tagged `c35ba11cc`
stands.
