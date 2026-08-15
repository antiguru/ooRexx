# Phase 5a -- the native layer and the bootstrap prologue

**Spec:** `docs/superpowers/specs/2026-08-15-phase-5-object-model.md`. It is the binding authority;
this plan is its argument. Decisions D25 to D45 live there and are not restated here.
**Evidence:** `docs/superpowers/specs/2026-08-15-phase-5-inherited-surface.md`, a dated reading of
`11638b91e`.
**Entry:** met. `perf-baseline.md`'s "The pre-Phase-5 baseline" pins the standing at `b029abe77`.
**Exit:** `CoreClasses.orx` translates, installs, and its prologue runs to `exit` on both engines,
with `StreamClasses.orx` and `PlatformObjects.orx` installed on the way.

## What 5a is, and the observation that bounds it

Phase 5 splits three ways. **5a is this plan**: the native class layer, and enough of it to run
`CoreClasses.orx`'s executable prologue. **5b** is Rexx method bodies -- `EXPOSE`, `FORWARD`, scoped
instance variables, `~new` and `init`. **5c** is the sourced library classes and the corpus gate.

**The bound on 5a is that the prologue never runs a Rexx method body.** Read at
`CoreClasses.orx:39-126`, every message it sends lands on a native method: `rexxPackage~addClass`,
`~addPublicClass`, `~objectname=`, `.context~package~publicClasses`, `publicClasses[name]`,
`.environment~put`, `name~upper`, `.String~defineClassMethod`, `~inheritInstanceMethods`,
`~inherit`, and two `CALL`s on program names. `::METHOD` directives are *installed*, and their
bodies are stored and never entered.

So this plan builds classes, behaviours, a registry, directories, directive installation and a
native message send -- and does **not** build instance variables, `EXPOSE`, `FORWARD`, or method
invocation into Rexx code. Those are 5b's, and a task here that reaches for them has mis-scoped.

**There is no oracle transcript for the prologue** (D39). `removeSetupMethods()` deletes
`DEFINECLASSMETHOD` and `INHERITINSTANCEMETHODS` before the image is saved, so no run of the shipped
interpreter reaches that file's `exit`. Every task below that needs an expected answer takes it from
a *separate, small* oracle program exercising the same construct, never from a transcript of the
bootstrap. A task that claims to have diffed the bootstrap against the oracle has not.

## Global Constraints

These bind every task. A task that cannot satisfy one stops and says so rather than working around
it.

**Differential correctness.** A change is right when output matches the C++ oracle byte for byte on
stdout, stderr and exit status -- never when it "looks right". Wrap every oracle run as
`( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib
/home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`, read the three descriptors separately, never
`2>&1`, and probe from a fresh empty directory with absolute paths. Never run a program listed in
`rust/corpus/oracle-crashes.txt`.

**Both engines, every construct, same task.** Decided 2026-08-15. Every construct lands in the
tree-walker and as a compiled op in the task that introduces it, not as `Op::Generic` with a
promotion to follow. `ir_dual` is then a live check from the first commit rather than a check on
delegation.

**The C++ tree is read-only.** `interpreter/`, `samples/`, `build/`, `ootest/`. Read freely, never
write. `ootest/` is an SVN working copy of a different repository and is git-excluded.

**No `unsafe`.** The workspace sets `unsafe_code = "forbid"`; that line is the record of which crates
hold an exception and this plan adds none.

**Ownership moves in one commit.** `owners.rs`'s module doc names five items that move together --
`EXPECTED_OUT_OF_SCOPE`, `coverage.rs`'s `EXPECTED_SUBSET`,
`variant_counts_match_the_audited_split`, `loud.rs`'s witness tables, and `lib.rs`'s
`instruction_owner`/`expr_owner`. A task that moves a construct from out-of-scope to in-scope edits
all five in the same commit. The same applies to `corpus/bif-exempt.txt`,
`corpus/keyword-exempt.txt`, `rexx-exec/tests/assertions.rs`'s `EXEMPT` and
`trace_oracle.rs`'s `PREFIX_COVERAGE` when a row of theirs starts passing.

**The performance guard.** Any task that lands code in `rexx-exec`, `rexx-core` or `rexx-classes`
runs a two-build sitting before it reports done:

```
./target/release/rexx-arms --build pinned=bench-baselines/pinned/rexx-run-pre-phase-5 \
                           --build head=target/release/rexx-run \
    --axis alloc4c --axis arith --axis compound --axis emptyloop --axis strings --axis varlookup \
    --rounds 5 --task <task> --commit <commit> --baseline bench-baselines/phase-5a-arms.tsv
```

The instrument is `instructions:u`. A movement under 1% on an axis is not a finding; at or above 1%
the task records it and says whether it is the change or the layout, with an `arm_ratio` row as
tie-breaker where one is expressible. Do not optimise; this phase's performance work is suspended
and the guard exists so a regression is attributed rather than discovered later.

**The gate commands**, all from `rust/`: `cargo fmt --all --check`; `cargo clippy --workspace
--all-targets -- -D warnings`; `cargo test --release --workspace`; the corpus gate under
`REXX_CORPUS_GATE=1` unfiltered; a debug run `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace
--no-fail-fast`, because `[profile.release]` does not set `debug-assertions` and every
`debug_assert` is compiled out of the release gates.

**Say what a check could not see.** For any claim resting on a check, state what that check would
have done had the claim been false. If the answer is "the same thing", the check is decoration.

---

## Task 1: the behaviour table

**Goal.** Replace `rexx-core::BehaviourTable`'s lookup-time superclass walk with the flattened
method dictionary the oracle builds, in the new `rexx-classes` crate. Data structure and its tests
only; nothing in `rexx-exec` calls it yet.

**Why this is first.** Every other task in this plan resolves a message through it, and its shape is
the one thing in the phase that a plausible-looking wrong answer survives longest.

**What the oracle does, and it is not a chain walk.** `RexxClass::createInstanceBehaviour`
(`ClassClass.cpp:1148`) builds a **flat** dictionary into the behaviour when the class is defined.
`updateInstanceSubClasses` (`:1071`) rebuilds only the instance behaviour;
`updateSubClasses` (`:1036`) rebuilds the instance behaviour **and then** the class behaviour,
because instance methods "may have an impact on metaclasses". Both cascade to every subclass.

**Build:**

* a behaviour holding a flat name-to-method map **plus scope ordering**, because
  `MethodDictionary` carries a `scopeList` and `scopeOrders` beside the name map and
  `FORWARD CLASS(SUPER)` cannot be answered without them;
* `define`, which **copies the behaviour first** -- `defineMethod` (`ClassClass.cpp:819`) says "make
  a copy of the instance behaviour so any previous objects aren't enhanced" -- then replaces the
  entry, then cascades instance-side only;
* `inherit`, which does **not** copy: it appends to the superclass list and cascades both sides in
  place;
* a per-behaviour **monotonic version**, bumped by every cascade. One field and one increment. It is
  what makes D28's "no send cache" cheap to revisit later, and adding it afterwards means finding
  every site that mutates a dictionary.

**Exact expected answers, taken from the oracle by separate probes, not argued.** Write a probe per
row before writing the code, and put the oracle's answer in the test:

| probe | what it pins |
|---|---|
| a class inheriting two mixins that both define one name | merge order under multiple inheritance |
| a diamond: two mixins both inheriting a third that defines the name | which scope wins |
| `~define` on a class with an instance already created | the copy: the old instance must not see it |
| `~inherit` on a class with an instance already created | no copy: the old instance must see it |
| `::class X mixinclass class` inherited into a class | the class-behaviour side, which an instance-method diamond does not reach |

**Verification.** Unit tests in `rexx-classes` carrying those oracle answers as expected values, each
naming the probe it came from. Plus a negative control: make `define` mutate in place instead of
copying, and confirm the third row's test goes red. A test that passes under both behaviours is not
testing the copy.

**Done when** the table answers every row above, the negative control is recorded in the report, and
no code outside `rexx-classes` has changed.

---

## Task 2: class objects, the registry, and the metaclass graph

**Goal.** A class is an object in the arena with an `ObjRef` identity, carrying **two** behaviour
ids (D44), a superclass list, a subclass list for the cascade, and its id string.

**Build:** the class object; the registry mapping a name to a class `ObjRef`; the metaclass edges,
including `.class` being an instance of itself, which is why `BehaviourTable`'s walk needed a
visited set and why the flattened form must terminate; and the primitive classes this plan needs,
grown from what the prologue turns out to require rather than transcribed from `Setup.cpp` (D25).

**The `Setup.cpp` list is a checklist and is derived, not copied** (criterion 7). Add a build-script
arm deriving the class names from `/bin/grep -aE "createInstance\(\)"` over
`interpreter/memory/Setup.cpp`, following `crates/rexx-inventory/build.rs`, whose own doc is "The C++
tree is the source of truth. Nothing here is hand-maintained". A hand-written list stays green while
`Setup.cpp` gains a class. Every derived name is either in the native layer or in a deferral table
with a reason naming what would have to exist; "not needed yet" is not a reason.

**Verification.** The wiring assertion of criterion 3, per class, against the oracle: `~class`,
`~superClass`, **`~superClasses`**, `~isA`, `~metaClass`. `~superClasses` is not optional --
`.DateTime~superClass~id` and `.Array~superClass~id` are both `Object`, so the other four are blind
to every mixin edge, and a build that wired no mixin at all passes without it.

**Done when** every class in the native set answers all five byte-identically to the oracle, and the
deferral table is a test over the derived list rather than a comment.

---

## Task 3: the native message send, and the security seam

**Goal.** `ExprKind::Message` and `InstructionKind::Message` evaluate, resolving through Task 1 and
invoking a **native** method. Both engines, same task.

**Shape, already decided.** D24: `resolve` and `invoke`, two operations, not one fused `send`. D28:
dynamic, no per-call-site cache -- `ir.rs`'s `CallSite` is not extended to sends, and the reason is
Phase 4e's own amendment, that the classic cache is correct *because it needs no guard*, which is
exactly the property a send cache lacks.

`resolve` takes the receiver's behaviour, the uppercased name, **an optional start scope**, and an
optional per-object method table. The start scope is not speculative: `RexxObject::messageSend`'s
scope-override overload (`ObjectClass.cpp:919`) reaches `superMethod(msgname, startscope)`, and
`CoreClasses.orx` uses `forward class(super)` throughout. Build the parameter now even though
`FORWARD` is 5b's; retrofitting a parameter through every call site is the expensive order.

**The security seam (D45), and it is the whole of the security work in this phase.** Dispatch passes
through exactly **one** chokepoint, and a test asserts there is exactly one. No manager object, no
installation path, no Rexx-visible behaviour. The test is the deliverable: a count of call sites that
reach method invocation, asserted at one, so that Phase 7 adds a hook to something that exists rather
than inventing a second mechanism -- which is what D12 warns costs touching every path twice.

**Verification.** Oracle-differential programs sending a native message to each primitive kind, on
both engines, byte for byte. Plus `ir_dual` agreement on raw stderr, which is live from this task
onward.

**Done when** a message send to a native method matches the oracle on all three descriptors on both
engines, and the one-chokepoint assertion exists and fails if a second site is added.

---

## Task 4: `.environment`, `.local`, `.context`, `.methods`

**Goal.** All four become real objects (D33), and `.NAME` resolution answers through them.

**The resolution order is not the obvious one.** `CoreClasses.orx:47-52` names `.LocalServer`,
`.SupplierMixin`, `.ManyItemMixin`, `.SetMixin` and `.BagMixin` -- classes declared later in the same
file **with no `PUBLIC` keyword**. A non-public class is not in `.environment`, so `.NAME` must
consult the running package's own class table **before** `.environment`.

`.context` and `.methods` are not classes and are in no registry: `.context` is the per-activation
reflection object, `.methods` is the package's unattached-method directory. Both exit 120 today with
`an environment symbol is not implemented`, against oracle rc 0 answering `a RexxContext` and
`.METHODS`.

**Close both `VALUE` routes together or neither.** `phase-4-exclusions.txt` records the one-argument
form as **silently wrong at rc 0**: `say value('.LOCAL')` is `The Local Directory` on the oracle and
`.LOCAL` here, while `say .LOCAL` exits 120. One route answers wrongly in silence and the other
refuses; closing one leaves the asymmetry. The three-argument empty-selector form is **not** this and
is not in scope -- its oracle answer is `..LOCAL`.

**Verification.** Oracle-differential on `say .LOCAL`, `say .ARRAY`, `say .METHODS`, `say .context`,
`say value('.LOCAL')`, `say value('.ARRAY')`, and a non-public class resolved by name from inside its
own package. Both engines.

**Done when** the `UNATTRIBUTED:an environment symbol` rows leave `corpus/bif-exempt.txt` and
`expr_owner` gives `ExprKind::DotVariable` an owner, both in this task's commit.

---

## Task 5: the directives that install

**Goal.** `::CLASS`, `::METHOD`, `::ATTRIBUTE` and `::CONSTANT` install. Bodies are stored, never
entered.

**`::CONSTANT` carries a divergence recorded before it is fixed.** Its parenthesised expression form
is accepted here and **never evaluated**, where the oracle evaluates it when the directive installs.
Measured, on a program whose prologue is `say "prolog"`:

```
::CLASS K then ::CONSTANT c (.NoSuchClass~m)   oracle rc 159, stdout empty
                                              this crate rc 0, stdout "prolog"
::CONSTANT sep (...) with no ::CLASS           oracle rc 157, 99.906, structural
```

**A well-formed expression is rc 0 on both sides**, so a probe over valid programs cannot see this;
the witness has to make the expression fail. `StreamClasses.orx:548-549` are this exact form.

**Widen `coverage.rs` first.** `assert_program_has_only_routine_directives`, applied to the union of
`SUBSET_FILES`, panics on any subset program carrying `::CLASS` or `::METHOD` -- which is every
corpus program this task adds. Widen the walker in this task's commit or no corpus program can land.

**Verification.** Oracle-differential per directive, including the failing-expression `::CONSTANT`
witness, on both engines. `::ANNOTATE naming a target` is **not** in scope: the oracle refuses it too
(99.945, rc 157), so it is not an over-refusal and retiring it closes no divergence.

**Done when** the four directives install, the `::CONSTANT` gap's row moves from KNOWN GAPS to CLOSED
DEFECTS in `phase-4-exclusions.txt`, and `coverage.rs` accepts a `::CLASS` in a subset program.

---

## Task 6: the native entry-point registry

**Goal.** `::METHOD ... EXTERNAL 'LIBRARY REXX name'` resolves at install (D37).

**It binds eagerly and a failure stops the program before its prologue.** Measured: a file whose
prologue is `say "prolog ran"` carrying one external method naming a missing entry point exits
**166** with **empty stdout** and `Error 90.998`. Naming a real entry point exits 0 and prints.

**Build** a registry in which every `LIBRARY REXX` name the three `.orx` files declare *resolves*.
An entry this phase does not implement raises **when invoked**, naming its owning phase -- so
install-time behaviour matches the oracle and the divergence sits where the later phase lands.
`CoreClasses.orx` declares five, all timers, Phase 6's. `StreamClasses.orx` declares a large family,
Phase 7's.

**Two are implemented here** as a stated scope addition, because `CoreClasses.orx` cannot reach its
own `exit` without them: `file_separator` and `file_path_separator`, called at install time by
`StreamClasses.orx:548-549`'s constants.

**Verification.** A corpus program per registered family that invokes one unimplemented entry and
pins the refusal, so a registered-but-silent entry cannot masquerade as working. Oracle-differential
on the eager-bind failure shape.

**Done when** all three `.orx` files install without an unresolved external, and invoking an
unimplemented entry is loud and names its phase.

---

## Task 7: the Package object and the setup methods

**Goal.** The prologue's first clauses run. Needs a live Package object with `addClass`,
`addPublicClass`, `objectname=` and `publicClasses`, plus `.methods`, plus the two setup methods.

**The setup methods are removed after the bootstrap** (D39). `removeSetupMethods()`
(`ClassClass.cpp:923`, called at `Setup.cpp:1809`) deletes `DEFINECLASSMETHOD` and
`INHERITINSTANCEMETHODS` from the image. Measured on the shipped oracle:
`.String~hasMethod("DEFINECLASSMETHOD")` is 0, `.Supplier~hasMethod("INHERITINSTANCEMETHODS")` is 0,
`.Class~hasMethod("DEFINE")` is 1. Provide them during the bootstrap and remove them after; keeping
them is a divergence any corpus program can see by sending `~defineClassMethod`.

**Verification.** Oracle-differential on `hasMethod` for all three names after the bootstrap, and on
sending `~defineClassMethod` to a class, which must be `97.1` and rc 159.

**Done when** the prologue reaches its `~inherit` block and the three `hasMethod` answers match.

---

## Task 8: the collection sends the prologue makes

**Goal.** `DO OVER` a collection **object**, the `[]` index message, and Directory `~put`.

`LoopKind::Over` is `Owner::InScope` in `owners.rs` today, which is true only for what Phase 4 can
iterate -- an array. Over an object it must send `makearray` or `supplier`. The prologue also uses
`do name over "nl", "cr", ...`, a comma-separated **expression list** over a line continuation, which
is a different construct from `do name over publicClasses`.

**Verification.** Oracle-differential on both `DO OVER` shapes and on `~put`/`[]` against a Directory,
both engines. The ownership move edits all five `owners.rs` pinned items in this commit.

**Done when** both loop shapes and the index message match the oracle.

---

## Task 9: `rexx-lib` and the bootstrap

**Goal.** The three `.orx` files are embedded, located and run (D26).

**Embed at build time from this repository's own tracked copy**, by a workspace-relative path, the
way `crates/rexx-inventory/build.rs` already reaches the C++ tree. They are never edited; a gap in our
interpreter is fixed in our interpreter. Record each file's sha256 so a drift is a build failure.

**`rexx-lib` owns program-name resolution for the two `CALL`s.** `call 'StreamClasses.orx'` would
otherwise reach the external file search, which `Loud::unresolved_call`'s own doc assigns to Phase 7.
Intercept the three bootstrap names ahead of that search and touch the search itself not at all.

**Verification.** The prologue runs to `exit` on both engines. **There is no oracle transcript to
diff against** -- say so in the report rather than implying one. What is checkable: the exit status,
that stdout is empty, that stderr is empty, and that the post-bootstrap state answers Task 2's five
wiring questions for every class the prologue installs, byte for byte against the oracle.

**Done when** `rexx-run CoreClasses.orx` exits 0 with empty stdout and stderr on both engines.

---

## Task 10: the 5a gate

**Goal.** Prove 5a rather than assert it, and leave 5b a clean tree.

**Build** `rust/corpus/phase-5a.txt` and wire it into every harness that reads a subset list.
`corpus.rs`, `coverage.rs`, `ir_dual.rs` and `collect_stress.rs` each assert against a read of the
corpus directory, so they redden until wired in. **`trace_oracle.rs` has no such guard** and will
silently keep measuring the 4a/4b/4c union -- wire it explicitly and say so.

**The subset contains at minimum:** a native message send to each primitive kind; the five wiring
answers per class in the native set; a mixin diamond that discriminates merge order from a chain
walk; a class-behaviour-side witness, which an instance-method diamond does not provide; the four
directives including the failing-expression `::CONSTANT`; both `VALUE` routes and `say .LOCAL`; both
`DO OVER` shapes; and one unimplemented native entry point per registered family.

**Report, and each of these is a number or a named absence:** the five gate commands; the
unsafe-block count and the list of crate roots carrying `deny` rather than `forbid`, which every
phase exit owes and which the spec's first draft omitted; the guard's `across_builds` movement per
axis against the 1% floor; and every committed table this plan edited.

**Done when** all five gate commands pass, the subset passes byte for byte on both engines, and the
report names what it did **not** cover -- 5b's method bodies, instance variables, `EXPOSE` and
`FORWARD` -- so 5b's plan starts from a stated boundary rather than an assumption.
