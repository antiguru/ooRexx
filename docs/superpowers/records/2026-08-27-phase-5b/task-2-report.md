# Task 2 report: the behaviour snapshot (D58)

BASE: 2d5614e63 on plan/rust-rewrite. Landed as `f558ea501`, one commit, tree clean after it.

## 1. Re-measuring the premises

### 1.1 `objcla` is a silent wrong answer (re-measured, not taken on trust)

Probe: a copy of `corpus/gate-tables/concepts/objcla.rex` run from a fresh empty directory,
three descriptors read separately, oracle wrapped in `ulimit -v 1048576`.

```
--- oracle ---   rc=0  stdout: before 0 / after 0 / fresh 1   stderr: (empty)
--- ir ---       rc=0  stdout: before 0 / after 1 / fresh 1   stderr: (empty)
--- tree-walker  rc=0  stdout: before 0 / after 1 / fresh 1   stderr: (empty)
```

Confirmed: rc 0 and empty stderr on all three, `after 1` against the oracle's `after 0`. A silent
wrong answer, not a refusal, exactly as the brief states.

### 1.2 Both `~inherit` arms already agree, which is the live walk getting them right

The spec's `~inherit` program and the brief's `~uninherit` program, measured myself against the
oracle rather than copied on trust:

```
inherit:   oracle rc 0  a 1 / b 1 / c mixin-ran          both engines byte-identical
uninherit: oracle rc 0  a 1 / b mixin-ran / c 0 / e trapped 97   both engines byte-identical
```

The brief's four `~uninherit` lines are confirmed. Both arms passing today is not luck: the live
walk of the class graph is exactly the implementation that gets `~inherit` right and `~define`
wrong, which is what makes the second control (copy on `~inherit` too) the mutation no existing row
can see.

### 1.3 The corpus has no `~inherit` witness that constructs an instance

```
$ cd rust/corpus && git ls-files '*.rex' | xargs /bin/grep -ail '~ *inherit\|~ *uninherit'
gate-tables/concepts/usingcl.rex
lang/class_mutator_inherit_position.rex
lang/class_mutator_inherit_position_not_inherited.rex
lang/class_mutator_refusals.rex
lang/class_mutator_uninherit_not_inherited.rex
lang/class_mutators_user_class.rex
lang/class_reflection.rex
lang/class_rexx_defined_inherit.rex
lang/class_rexx_defined_library_inherit.rex
lang/class_rexx_defined_library_no_mutation.rex
lang/class_rexx_defined_uninherit.rex
lang/class_subclass_factory.rex
lang/class_subclass_refusals.rex
lang/library_bootstrap_setup_methods_gone.rex
lang/uninit_class_inherit_runtime.rex
lang/uninit_class_uninherit.rex

$ ... | xargs /bin/grep -ail '~ *new'
(nothing)
```

None of them constructs an instance, so none can witness either D58 arm.

### 1.4 The handle family exists, cited by symbol

`crates/rexx-classes/src/class_graph.rs` carries `pub struct BehaviourHandle(usize)` and
`instance_behaviour_handle`, `class_behaviour_handle`, `has_method_at`, `method_names_at`,
`has_scope_at`, `lookup_at`, `lookup_from_scope_at`, `resolve_super_scope_at`, `version_at`, all
found with `/bin/grep -n`. `crates/rexx-classes/src/lib.rs:41` re-exports `BehaviourHandle`.

The dependency-cycle claim holds: `rexx-classes/Cargo.toml` depends on `rexx-core`, so `rexx-core`
naming a `rexx-classes` type would close a loop.

## 2. What I built

**The change is one field and one enum shape.** `rexx_core::Body::Instance` gains
`behaviour: BehaviourHandle`, filled once at `native_new` from the class's handle at that instant,
and `dispatch.rs`'s `Behaviour::Instance` stops naming a class and names that handle instead. Every
reader of an instance's behaviour already went through `Interp::receiver_behaviour`, so the rule is
stated once, in the type, and not restated at each mutator or at each reader.

The readers this reaches, all of which were resolving live off the class before and answer from the
stored handle now: `Interp::lookup` (both the ordinary and the `:scope`-override arm),
`receiver_has_scope` (`validateScopeOverride`), `super_scope_for` (`superScope`),
`native_has_method` (`~hasMethod`) and `answers_uninit`.

### What moved, and why

`BehaviourHandle` moved from `rexx-classes` to `rexx-core` (`body.rs`, beside `BehaviourId`), and
`rexx-classes` re-exports it, so `rexx_classes::BehaviourHandle` still resolves and no caller
changed. The cycle the brief names is real and this is the direction that does not close it:
`rexx-classes` depends on `rexx-core`. The field stays private with `new`/`index` accessors, so
`rexx-exec` can hold and copy a handle but cannot mint one; only `ClassGraph::alloc_behaviour` does.

### What I rejected

* **A side table in `Interp` from instance to handle.** It is a second store of the same fact,
  needing its own collection discipline, and the collector already traces the body.
* **Storing a raw `usize` in `Body::Instance`** to avoid moving the type. That puts an untyped
  index next to `class: ObjRef` in the one struct where confusing the two is a silent wrong answer.
* **Snapshotting primitives too.** Measured instead: a program cannot move a built-in class's
  behaviour at all. `.String~define('ZORK', .methods~z)` is oracle rc 158, 98.985 "User additions
  are not allowed to the REXX language classes", and `.String~inherit(.Mx)` is trapped 98 at rc 0;
  both already agree byte for byte on both engines. So live and stored answer alike for every
  receiver whose class this crate builds, and the extra field on every text and array object would
  buy nothing.
* **Deleting `rexx-core`'s `BehaviourTable` and `Object::behaviour` and reusing the name.** Both
  look dead and both are out of this task's scope. Measured, so the next reader does not have to:
  `/bin/grep -rn "BehaviourTable" crates/ --include=*.rs` names only its own definition, the
  `pub use` in `lib.rs`, `crates/rexx-core/tests/behaviour.rs`, and two prose mentions in
  `rexx-classes/src/lib.rs`; and `/bin/grep -rnE "\.behaviour\b" crates/ --include=*.rs`, with the
  `instance_behaviour`/`class_behaviour`/`behaviours[` spellings excluded, matches nothing at all,
  so `Object::behaviour` is written by `Heap::alloc_with_uncollected` and read by no one. That is
  why the new type is a separate one rather than a reuse of `BehaviourId`, and why the two sit
  adjacent in `body.rs` with a doc saying which is which.

### What I deleted

`ClassRegistry::lookup_instance_method_from_scope`, `instance_behaviour_has_scope` and
`instance_super_scope` -- the three class-taking instance-side readers -- had exactly one caller
each, all in `Interp`, and all three are now handle-taking (`lookup_from_scope_at`,
`behaviour_has_scope`, `super_scope_at`, plus `has_method_at` and `lookup_at`). Removing them means
the live walk is no longer reachable from a send.

`lookup_instance_method` stays: `ObjectModel::build` uses it at bootstrap to find the `MethodId`
behind each native method, before any instance exists.

## 3. Re-measured after the change

Release binary rebuilt from the committed sources (md5 of `dispatch.rs` and `class_graph.rs`
checked against the backups after the last mutation restore); probes from a fresh empty directory,
absolute paths, three descriptors read separately, oracle under `ulimit -v 1048576`.

| probe | oracle | ir and tree-walker |
|---|---|---|
| `objcla.rex` | rc 0 `before 0` / `after 0` / `fresh 1` | byte-identical |
| `~inherit`, the spec's program | rc 0 `a 1` / `b 1` / `c mixin-ran` | byte-identical |
| `~uninherit`, the brief's program | rc 0 `a 1` / `b mixin-ran` / `c 0` / `e trapped 97` | byte-identical |
| `~delete` after construction | rc 0 `a 1` / `b still-here` / `c 1` / `d still-here` / `e 0` | byte-identical |
| `~define` on a superclass | rc 0 `a 0` / `b 1` / `c 1` / `d late-ran` | `a`/`b`/`c` agree, `d` is rc 120 -- section 6 |
| `~inherit` on a superclass | rc 0 `a 0` / `b 1` / `c mixin-ran` | byte-identical |
| `UNINIT` lost by `~delete` after construction (probe) | rc 0 `a 1` / `b 0` / `uninit-ran` / `c done` | byte-identical |
| `SUPER` and `:scope` across a `~define` | rc 0, three lines | byte-identical |
| `.String~define` | rc 158, 98.985 | byte-identical |
| `.String~inherit` under a trap | rc 0 `trapped 98` | byte-identical |

The `~delete` row is **a second silent-then-loud wrong answer this change closes and nothing asked
for**: before the change this crate answered `c 0` and then raised 97.1 at rc 159 where the oracle
answers `c 1` / `d still-here` at rc 0.

The `UNINIT` row is the reader the brief did not name: the finalizer question is asked of the
object's own behaviour, so an instance registered at construction still runs a finalizer its class
has since dropped.

## 4. The gate row

`REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast`
exits **101**, as designed while 5b rows are red, and prints

```
  agree          loud=no  5b   objcla Object Classes   depth 1 parent typcla provide.xml:84 objcla.rex
  5b: 6 rows, 2 not yet `agree`          (table C)
  5b: 2 rows, 0 not yet `agree`          (table D)
```

The two table-C rows still red are `usesem` (Task 3) and `methodsbyclass` (Task 8). Before this
task there were three; `objcla` was the third.

Table C's own totals for the whole run: `agree: 545`, `diverge-both: 40`, and the phase split
`5a: 135 rows, 0 not yet agree` / `5b: 6 rows, 2` / `5c: 1347 rows, 941`.

`corpus/gate-tables/concepts/objcla.rex` is in `corpus/phase-5b.txt` and in `EXPECTED_SUBSET_5B` in
the same commit, per the phase's amendment.

## 5. The controls, as run

The instrument for a corpus program is
`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus --no-fail-fast`; for a gate row
it is the phase-gate command in section 4. Each mutation was applied to the source, the release
binary rebuilt, both commands run, then the source restored from a copy in the scratchpad (never
`git checkout --`) **and the binary rebuilt again**.

**Baseline, against the tree exactly as committed**: `286 of 286 matching`, corpus status 0; phase
gate status 101 with table C `agree: 545` and `5b: 6 rows, 2 not yet agree`.

| mutation | gate rows that moved | corpus programs that reddened |
|---|---|---|
| **C1** `receiver_behaviour` resolves an instance's behaviour from its class on every send -- `objcla`'s own committed control | `objcla` `agree` -> `diverge-stdout` | `objcla.rex`, `..._delete.rex` |
| **C2** `~inherit` copies the instance behaviour before rebuilding it | **none** | `..._inherit.rex` |
| **C2u** `~uninherit` copies the instance behaviour before rebuilding it | **none** | `..._uninherit.rex` |
| **M3** `~delete` stops copying, so it mutates the behaviour existing instances hold | **none** | `..._delete.rex` |
| **M4** the cascade copies each subclass's behaviour instead of rebuilding it in place | **none** | `..._subclass.rex` |
| **M5** the `UNINIT` question alone goes back to the class | **none** | `..._delete.rex` |
| **M6** `~hasMethod` alone goes back to the class | `objcla` `agree` -> `diverge-stdout` | `objcla.rex`, `..._delete.rex` |
| **M7** the ordinary lookup alone goes back to the class, leaving `~hasMethod` on the snapshot | **none** | `..._delete.rex` |

"gate rows that moved: none" is read as a **line-for-line diff of the whole normalised gate-table
log against the baseline's**, both tables, every phase -- not as a count. For C2, C2u, M3, M4, M5
and M7 the two logs are identical; the normaliser drops libtest's parallel test ordering, thread
ids, blank lines and cargo's build line, and nothing else. For C1 and M6 the only lines that differ
are `objcla`'s verdict, its two descriptor lines, and the totals that follow from it.

### Control 1: `objcla`'s own committed control

C1 is the mutation the row's `control:` field describes -- "rebuild an existing instance's method
lookup from its class on every send, so a method defined after the instance was created answers".
It reddens `objcla` with the exact transcript the row was showing before this task:

```
  diverge-stdout loud=no  5b   objcla Object Classes ... objcla.rex
      oracle rc=0    out="before 0\nafter 0\nfresh 1\n" err=""
      crate  rc=0    out="before 0\nafter 1\nfresh 1\n" err=""
```

so the row is live rather than green over nothing.

### Control 2: copying on `~inherit` too

C2 is the point of the task and the table above is its result: **not one gate row moves**, in
either table and in any phase, and exactly one corpus program reddens -- the `~inherit` witness
this task adds. C2u is the same experiment for `~uninherit` and answers the same way with the
`~uninherit` witness.

Named, since the brief asks for the rows by name in both readings: the rows green before C2 are
every row of both tables that the phase gate reports as `agree` (table C `agree: 545`, table D
`agree: 41`), and every one of them is still `agree` after it. The rows red before and after are
table C's `usesem` and `methodsbyclass` in 5b, the 5c rows the gate reports and does not gate, and
table D's 5c, phase-7 and deferred-parse-error rows. Not one of them changes verdict either.

### "Can fail" is not "adds coverage", checked per row

Every one of the four new corpus programs is the **only** thing in the suite that catches at least
one mutation:

* `..._inherit.rex` -- C2, which nothing else sees at all.
* `..._uninherit.rex` -- C2u, same.
* `..._delete.rex` -- M3, M5 and M7, each of which nothing else sees.
* `..._subclass.rex` -- M4, same.

Where a row is not the only catcher, that is recorded rather than claimed away: under C1 and M6,
`objcla.rex` catches the mutation too, so against those two `..._delete.rex` adds nothing.
`objcla.rex` itself catches no mutation `..._delete.rex` misses; it is in `phase-5b.txt` because
the phase's amendment puts a gate row's probe there in the task that makes the row agree, not
because it is the unique catcher of anything.

### A witness that could not fail, found and replaced

The first `class_behaviour_snapshot_uninit.rex` I wrote had a class **gain** a `UNINIT` by
`~define` after an instance existed, and asserted that the instance answered `a 0` and ran no
finalizer. It was green, it reddened under C1, and **it was blind to M5** -- the mutation of the
one code path it was supposed to witness. The reason: `native_new` registers an object for
`UNINIT` only when its class has one **at construction**, so in that program the object was never
registered and `answers_uninit` was never asked about it at all.

The replacement goes the other way -- the class **loses** its `UNINIT` by `~delete` after the
instance exists, so the object is registered, its own behaviour still answers `UNINIT`, and its
class's no longer does. That is the only shape in which the snapshot and the live read disagree at
that call site.

### A second row that added nothing, and was folded in rather than kept

With the replacement written, `class_behaviour_snapshot_uninit.rex` and
`class_behaviour_snapshot_delete.rex` both used `~delete`, and **every mutation that reddened one
reddened the other**: measured, M3 and M7 reddened both, and only M5 separated them. Two programs
where one nearly dominates the other is what the "adds coverage" rule exists to stop, so they were
merged into a single `class_behaviour_snapshot_delete.rex` that asks three readers of the same
behaviour in turn -- the name (`a`), the send (`b`), and the `UNINIT` question (`c` and the
finalizer). The merged row catches M3, M5 and M7, each alone. Measured on the oracle, rc 0,
`a 1` / `b still-here` / `c 1` / `d 0` / `uninit-ran` / `e done`, five runs of five identical on
the oracle, five of five on `ir` and three of three on `tree-walker`.

### A test that could fail and still did not earn its place

Moving `BehaviourHandle` out of `class_graph.rs` deleted a doc sentence stating that no two classes
ever share a handle -- a real property, and the one the whole asymmetry rests on. I wrote it as an
assertion in `behaviour_wiring.rs` and then ran the mutation for it (**M8**: `define_class` gives a
new class its superclass's instance handle instead of a fresh one). The new test failed, and so did
three tests that were already there:
`define_on_a_superclass_still_reaches_an_existing_subclass_instance`,
`every_cascade_against_a_handle_bumps_its_version_by_exactly_one` and
`two_classes_with_identical_ancestors_can_still_answer_to_different_method_sets`. A fourth
assertion over an already-policed property adds diagnosis wording and no coverage, so the test was
deleted and the sentence went into `ClassGraph`'s own doc, beside the sibling property it was
already carrying.


## 6. Probing past the row

Every shape the brief names, measured against the oracle from a fresh empty directory, three
descriptors, both engines, all after the change.

1. **`~define` on a superclass after construction.** Oracle rc 0, `a 0` / `b 1` / `c 1` /
   `d late-ran`: an existing instance of a *subclass* **does** see it, because only the direct
   recipient's handle is replaced. This crate answers `a 0` / `b 1` / `c 1` and then refuses the
   `d` send at rc 120 -- see finding A below. In the corpus as the first half of
   `..._subclass.rex`, which stops at `hasMethod` for that reason.
2. **`~inherit` on a superclass after construction.** Oracle rc 0, `a 0` / `b 1` / `c mixin-ran`;
   both engines byte-identical. The second half of `..._subclass.rex`, and it does send the
   message, because a mixin's method comes from a `::METHOD` directive and runs.
3. **An instance built before and after the same mutation.** `objcla.rex`'s own
   `before`/`after`/`fresh`, and the `b` and `d` lines of the `~inherit` and `~delete` witnesses.
4. **`~setMethod`'s interaction.** Not reachable yet: `o~setMethod('OWN', .methods~own)` from a
   program context is the oracle's 97.2 "cannot accept private message" at rc 159, and rc 120
   here. It is Task 3's subject (`usesem`), and the per-object scope it creates sits **in front
   of** the snapshot rather than inside it, so the two do not meet in anything this task changed.
5. **A subclass's instances under a parent's mutation.** `..._subclass.rex`, both families in one
   program, with M4 as its control.
6. **`~delete` after construction.** Not on the brief's list, and it was wrong: `c 0` and a 97.1
   at rc 159 against the oracle's `c 1` / `d still-here` at rc 0. Closed by the same change,
   witnessed by `..._delete.rex`, controlled by M3.
7. **The `UNINIT` question.** Also not on the list. `answers_uninit` reads the object's own
   behaviour, so an instance keeps a finalizer its class has dropped. Closed by the same change,
   folded into `..._delete.rex`, controlled by M5.
8. **`~defineMethods` after construction.** Oracle rc 0, `a 0` / `b 1` -- the copying family's
   third member behaves like `~define`. **No corpus witness is possible yet**: the program needs
   `.Directory~new` to build the table, which is rc 120 here.
   `ClassGraph::define_methods` already calls `copy_instance_behaviour`, so the mechanism is in
   place and unwitnessed at the interpreter level. Recorded for whichever task lands
   `.Directory~new`.
9. **A primitive receiver.** `.String~define('ZORK', .methods~z)` is oracle rc 158 / 98.985 and
   `.String~inherit(.Mx)` is trapped 98 at rc 0; both already agree byte for byte on both engines,
   and `corpus/lang/class_rexx_defined_define.rex` and `class_rexx_defined_inherit.rex` are the
   committed witnesses for that lock. So reading a built-in class's behaviour live rather than
   storing a handle on every string and array is not an exception to D58: there is no mutator a
   program can send that would make the two answers differ.
10. **`SUPER` and a `~name:scope` override across a `~define`.** Oracle rc 0, three lines, both
    engines byte-identical. Both readers moved to the stored handle here and neither changed
    answer.

## 7. Two findings this task does not own

### A. A method installed by a runtime `~define` is reflected but cannot be sent

Measured, oracle rc 0: `.K~define('X', "return 'from-source'")` then `.K~new~x` answers
`from-source`. This crate refuses at rc 120, `rexx-exec: method "X" of class "K" is not
implemented (Phase 5)`, on both engines and for a `Method` object argument
(`.K~define('X', .methods~x)`) just the same. The reflection half agrees: `.K~hasMethod('X')` is
`0` and `.K~new~hasMethod('X')` is `1`, byte-identical.

**It is not this task's and it is not new**, measured at the pre-task binary as well. What is new
is that it is *reachable*: `a4ee9284f`'s own commit message says "The compiled body is validated
and not retained. No send this phase can make reaches an instance-side entry", which was true when
it was written and stopped being true when Task 1 landed `~new`. Nothing in the 5b plan or spec
owns it -- `/bin/grep -n "retain\|compiled body\|methna\|not retained"` over both files matches
only three unrelated lines about class-scope retention.

The failure mode is the right one -- a loud refusal, not a silent wrong answer -- so nothing is
being answered wrongly. It is flagged here because a 5a claim's premise expired under a later
task, which is the class of thing this phase keeps asking to be written where the next reader
sees it.

### B. The performance guard is owed and I have not run it

This change lands code in `src/` of `rexx-core`, `rexx-classes` and `rexx-exec`, so 5a's guard asks
for a `rexx-arms` two-build sitting. I did not run one, for a reason that is the guard's own
staleness test rather than a preference:

```
$ git log --oneline 15a1ffa98..HEAD -- rust/crates rust/Cargo.toml rust/Cargo.lock Cargo.toml | wc -l
152
```

The test is that every commit in that list is one the plan's ledger records. This list spans the
whole of 5a and 5b, so no single ledger names it, and the pin's own rule then says rebuild the pin
and restart the baseline rather than reason about it. Both pinned binaries are present
(`bench-baselines/pinned/rexx-run-15a1ffa98`, built 2026-08-20). Rebuilding a pin and restarting a
baseline is a phase-level decision, and the 5b ledger already records that Task 5's sitting is
reserved for the controller. Flagging rather than deciding.

**What the change is likely to cost, stated so it can be checked rather than assumed:** one
`usize` added to `Body::Instance`, and on the send path one fewer indirection, not one more -- the
handle is read out of the body the dispatcher already has in hand, where before it was fetched
from the class registry's `HashMap` on every send. Primitive receivers pay exactly what they paid
before, the same `instance_behaviour_handle` call.

## 8. A measurement hazard met head-on

Four probes in section 6 first came back as **new divergences** that were not real: the mutation
runner restored the sources and did not rebuild, so `target/release/rexx-run` was still the M6
mutant while I read `~delete`-after-construction as broken. The tell was that
`class_behaviour_snapshot_delete.rex` was green in the corpus run at the same moment its own
strict subset "diverged" as a hand-run probe. The runner now rebuilds on restore, every figure in
sections 3, 5 and 6 was re-taken afterwards, and the md5 of both mutated files was checked against
the backups before the final baseline. This is the brief's stale-binary warning arriving in the
direction it does not usually get stated: a stale *mutant* binary reads exactly like a defect.

## 9. The five gates, and the phase gate

All run from `rust/` against the tree as committed, each status read unpiped from a file, never
chained with `&&`.

| command | status | figures |
|---|---|---|
| `cargo fmt --all --check` | **0** | empty output |
| `cargo clippy --workspace --all-targets -- -D warnings` | **0** | no warnings |
| `cargo test --release --workspace --no-fail-fast` | **0** | 1916 passed, 0 failed |
| `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | **0** | 1916 passed, 0 failed; `286 of 286 matching` |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0** | 1917 passed, 0 failed; `286 of 286 matching` |

`command -v memcap` printed `/home/moritz/.local/bin/memcap`, so gate 5 ran under the cap rather
than the `ulimit -v` substitute.

`cargo fmt --all --check` exited **1** on the first attempt, over four blocks the formatter wanted
rewrapped; `cargo fmt --all` fixed them, the diff is whitespace only, and **the two headline
controls were re-run against the reformatted tree** so that the section 5 table describes the
sources actually committed. C1 reddened `objcla` and `..._delete.rex` again; C2 reddened
`..._inherit.rex` and moved no gate row. The baseline after the reformat is the same
`286 of 286 matching` and the same phase-gate status 101.

The phase gate:

```
REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec \
    --test gate_table_c --test gate_table_d --no-fail-fast
```

status **101**, which is the designed value while any 5b row is red. Table C: `5b: 6 rows, 2 not
yet agree`, those two being `usesem` and `methodsbyclass`; before this task there were three, the
third being `objcla`. Table D: `5b: 2 rows, 0 not yet agree`.

## 10. The plan file

Nothing in the plan's Task 2 section was wrong, checked line by line against the tree: the
`ClassGraph` claim, the dependency-cycle claim, the "neither arm exists in the corpus" claim and
the printed `~uninherit` transcript all re-measured true. No correction made to
`docs/superpowers/plans/2026-08-27-phase-5b.md`.
