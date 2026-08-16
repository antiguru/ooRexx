# Phase 5a -- the object model runs Rexx code

**Spec:** `docs/superpowers/specs/2026-08-15-phase-5-object-model.md`, the binding authority. D25 to
D45 live there and are not restated.
**Evidence:** `docs/superpowers/specs/2026-08-15-phase-5-inherited-surface.md`, a dated reading of
`11638b91e`.
**Entry:** met. `perf-baseline.md`'s "The pre-Phase-5 baseline" pins the standing at `b029abe77`.
**Exit:** a driver program that calls `CoreClasses.orx` with a Package object runs it to `exit` on
both engines, with `StreamClasses.orx` and `PlatformObjects.orx` installed on the way.

**Revised once, after a four-reviewer panel demolished the first draft's scope.** The reports are in
`.superpowers/sdd/2026-08-15-phase-5a-native-layer/` and committed under
`docs/superpowers/records/`. What changed is below, because a plan that hides why it was rewritten
invites the same mistake.

## The premise that failed, and the boundary that replaces it

The first draft claimed 5a could be "the native layer only", on the ground that
`CoreClasses.orx`'s prologue sends every message to a native method and `::METHOD` directives are
"installed, and their bodies are stored and never entered".

**Half of that is true and the load-bearing half is false.** The prologue's own sends really do all
land on native methods -- including `DO OVER`, whose protocol is `requestArray` and which for a
primitive-behaviour receiver takes a C++ virtual with no dispatch at all. But **installing a
directive runs Rexx code**: `PackageClass::install` sends `ACTIVATE` to every class it installs, and
`CoreClasses.orx:3996` defines `::method activate class` on `TraceObject` as a Rexx body using
`EXPOSE`, class-scope instance variables and `self~activate:super`.

Measured on a four-line user program, `::CLASS K` plus `::METHOD activate CLASS` that says something:

```
oracle   rc 0    "ACTIVATE ran, zz = 42"      then   "prologue clause 1"
ours     rc 0                                        "prologue clause 1"
```

The activate output **precedes the main program's first clause**, and this crate does not run it at
all -- a live divergence nobody had recorded.

**So 5a includes Rexx method-body invocation, `EXPOSE` and class-scope instance variables.** It has
to; `CoreClasses.orx` cannot install without them. One consolation: `self~activate:super` is a
scope-override send, so `resolve`'s start-scope parameter is required here rather than being built
speculatively for 5b.

**The boundary now.** 5a is everything needed for the three `.orx` files to install and
`CoreClasses.orx`'s prologue to complete. **5b** is the rest of the object surface -- `~new` and
`init`, `FORWARD`, per-object `~setMethod`, and the collection classes' own Rexx method bodies.
**5c** is the library classes and the corpus gate.

**Two spec decisions are deliberately not 5a's, and saying so is the point.** **D41**'s
`identityHash` rule and **D42**'s literal pooling are both observable only once `.IdentityTable` and
`~identityHash` exist, which is 5c; 5a has no instrument that could witness either, so building them
here would be unverifiable work. **One half of D42 does bind here**: `.true`/`.false` and the
environment symbols go through the value path rather than the literal path, which is Task 6's, since
that is where they are created. A later plan that finds these unassigned should read this paragraph
rather than assume they were forgotten.

## Global Constraints

**Differential correctness.** A change is right when output matches the C++ oracle byte for byte on
stdout, stderr and exit status. Wrap every oracle run as `( ulimit -v 1048576;
LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib
/home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`, read three descriptors separately, never
`2>&1`, probe from a fresh empty directory with absolute paths. Never run a program in
`rust/corpus/oracle-crashes.txt`.

**"Byte for byte" is not what the corpus harness does, and Task 1 fixes that.** `descriptor_diffs`
(`rexx-exec/tests/support/oracle.rs`) compares `normalize_stderr(rust)` against
`normalize_stderr(cpp)`, and `tests/support/mod.rs` carries a committed test asserting
`normalize_stderr(indent_0) == normalize_stderr(indent_2)`. The erasure of a two-column indent
difference is deliberate and pinned. Until Task 1 lands an unnormalised mode, no task may claim a
byte-for-byte stderr comparison.

**Both engines, every construct, same task.** Every construct lands in the tree-walker and as a
compiled op in the task that introduces it, not as `Op::Generic` with a promotion to follow.

**The C++ tree is read-only.** `interpreter/`, `samples/`, `build/`, `ootest/`. `ootest/` is an SVN
working copy of a different repository and is git-excluded.

**No `unsafe`.** The workspace sets `unsafe_code = "forbid"` and this plan adds no exception.

**Ownership moves in one commit.** `owners.rs`'s module doc names five items that move together:
`EXPECTED_OUT_OF_SCOPE`, `coverage.rs`'s `EXPECTED_SUBSET`,
`variant_counts_match_the_audited_split`, `loud.rs`'s witness tables, and `lib.rs`'s
`instruction_owner`/`expr_owner`. The same applies to `corpus/bif-exempt.txt`,
`corpus/keyword-exempt.txt`, `assertions.rs`'s `EXEMPT` and `trace_oracle.rs`'s `PREFIX_COVERAGE`
when a row starts passing. **`coverage.rs`'s `every_in_scope_variant_is_witnessed_by_the_phase_subsets`
demands a subset-file witness the moment a variant moves**, which is why Task 1 exists and why every
later task adds its witness to `phase-5a.txt` in its own commit.

**The performance guard.** Any task landing code in `rexx-exec`, `rexx-core`, `rexx-classes` or
`rexx-lib` runs a two-build sitting before reporting done:

```
./target/release/rexx-arms --build pinned=bench-baselines/pinned/rexx-run-pre-phase-5 \
                           --build head=target/release/rexx-run \
    --axis alloc4c --axis arith --axis compound --axis emptyloop --axis strings --axis varlookup \
    --rounds 5 --task <task> --commit <commit> --baseline bench-baselines/phase-5a-arms.tsv
```

Instrument `instructions:u`. Under 1% on an axis is not a finding; at or above 1% the task says
whether it is the change or the layout, with an `arm_ratio` row as tie-breaker. **The axes call none
of the new crates**, so for a task that only adds `rexx-classes` code this run witnesses "the classic
paths did not get slower" and nothing more -- say that rather than implying it measured the new work.

**The gate commands**, from `rust/`: `cargo fmt --all --check`; `cargo clippy --workspace
--all-targets -- -D warnings`; `cargo test --release --workspace`; the corpus gate under
`REXX_CORPUS_GATE=1` unfiltered; and a debug run `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace
--no-fail-fast`, because `[profile.release]` does not set `debug-assertions`.

**The oracle can be wrong, and this phase is where that is most likely.** ooRexx's OO extensions are
the least-exercised part of the language, and a surprising inconsistency may be an upstream defect
rather than a property to reproduce. Three signals together justify recording a behaviour instead of
building it: it is **inconsistent** in a way no rule explains (`left` and `right` disagreeing at the
same argument); **ootest does not pin it**, which is now checkable at `ootest/`; and the neighbouring
code is already upstream-flagged (`CoreClasses.orx:1305` cites `[bugs:1406]`). One signal is not
enough -- `~define` copying the behaviour and `~inherit` not is also surprising, and its comment
proves it deliberate. Where a task takes this route it records the three signals and files the defect
upstream if it is clear; `phase-4-exclusions.txt` deviations 0 and 3 are the precedent for both
directions.

**Say what a check could not see.** For any claim resting on a check, state what that check would
have done had the claim been false. If the answer is "the same thing", it is decoration.

---

## Task 1: the harness prerequisites

**Goal.** Make it possible for any later task to add a corpus program and to claim a byte-for-byte
stderr comparison. Nothing in this task changes interpreter behaviour.

**Why first.** Three separate committed mechanisms block the second task otherwise.

**Build:**

* **An unnormalised comparison mode.** `descriptor_diffs` normalises stderr on both sides. Add a mode
  that compares raw bytes, opt-in per program, and leave the normalised path as the default so
  Deviation 0 in `phase-4-exclusions.txt` is untouched. The spec requires this and the first draft of
  this plan assigned it to nobody.
* **A widened directive walker.** `coverage.rs`'s `assert_program_has_only_routine_directives`,
  applied to the union of `SUBSET_FILES`, panics on any subset program carrying `::CLASS` or
  `::METHOD` -- which is every program this plan adds.
* **`rust/corpus/phase-5a.txt`, wired into every harness that reads a subset list.** `corpus.rs`,
  `coverage.rs`, `ir_dual.rs` and `collect_stress.rs` each assert against a read of the corpus
  directory and redden until wired in. **`trace_oracle.rs` has no such guard** and will silently keep
  measuring the 4a/4b/4c union; wire it explicitly and say in the report that it was the one site
  nothing would have caught.

**Verification.** With an empty `phase-5a.txt`, all five gate commands pass. Then a single
throwaway program carrying `::CLASS K` is accepted by the widened walker where it previously
panicked -- run it before the change to see the panic, and say so. For the unnormalised mode, a
program whose only difference from the oracle is two columns of trace indent must FAIL under the new
mode and PASS under the default; both directions, or the mode is unwitnessed.

**Done when** the five gate commands pass and the three mechanisms are demonstrated by the
before/after runs above rather than asserted.

---

## Task 2: the behaviour table

**Goal.** Replace `rexx-core::BehaviourTable`'s lookup-time superclass walk with the flattened
dictionary the oracle builds, in a new `rexx-classes` crate. Data structure and tests only.

**What the oracle does.** `createInstanceBehaviour` (`ClassClass.cpp:1148`) builds a flat dictionary
at class-definition time. `updateInstanceSubClasses` (`:1071`) rebuilds the instance behaviour only;
`updateSubClasses` (`:1036`) rebuilds the instance behaviour and then the class behaviour. Both
cascade to every subclass.

**Build:** a behaviour holding a flat name-to-method map **plus scope ordering**, because
`MethodDictionary` carries `scopeList` and `scopeOrders` beside the name map and a scope-override
send cannot be answered without them; `define`, which **copies the behaviour first** (D43;
`ClassClass.cpp:819`: "make a copy of the instance behaviour so any previous objects aren't
enhanced") then cascades instance-side only; `inherit`, which does **not** copy and cascades both
sides in place; **`inheritInstanceMethods`, which donates methods without creating a superclass
edge**; and a per-behaviour monotonic version bumped by every cascade, which is what makes D28's
no-send-cache decision cheap to revisit.

**Exact expected answers, from oracle probes written before the code.** Two mixins defining one name,
for merge order. A diamond, for which scope wins. `~define` with an instance already created -- the
old instance must NOT see it. `~inherit` with an instance already created -- it MUST. A
`mixinclass class`, for the class-behaviour side.

**The test that the first draft got wrong, twice.** A class-graph assertion cannot see
`inheritInstanceMethods`. Measured: `.Supplier~superClasses` is `The Object class` alone while
`.array~of(1,2)~supplier~hasMethod("ALLITEMS")` is 1, and `.Set~superClasses` and `.Bag~superClasses`
are byte-identical despite different donated content. **So every wiring test in this plan asserts a
method SET, not only the class graph.**

**Verification.** Unit tests carrying those oracle answers, each naming its probe. Two negative
controls: make `define` mutate in place and confirm the `~define` row reddens; make
`inheritInstanceMethods` a no-op and confirm the method-set row reddens. A test passing under both
behaviours is not testing anything.

**Done when** every row answers, both negative controls are recorded, and nothing outside
`rexx-classes` changed.

---

## Task 3: class objects, the registry, and the metaclass graph

**Goal.** A class is an arena object with an `ObjRef` identity, two behaviour ids (D44), a superclass
list, a subclass list for the cascade, and its id string.

**Build** the class object, the name-to-class registry, the metaclass edges including `.class` being
an instance of itself, and the primitive classes the prologue turns out to need -- grown by running
it (D25), not transcribed.

**The `Setup.cpp` list is derived, not copied.** Add a build-script arm deriving class names from
`/bin/grep -aE "createInstance\(\)"` over `interpreter/memory/Setup.cpp`, following
`crates/rexx-inventory/build.rs`. Every derived name is either in the native layer or in a deferral
table whose reason names what would have to exist; "not needed yet" is not a reason.

**Verification.** Per class, against the oracle: `~class`, `~superClass`, `~superClasses`, `~isA`,
`~metaClass`, **and the class's instance method set**. The method set is not optional -- see Task 2.

**Done when** every class in the native set answers all six byte-identically and the deferral table
is a test over the derived list.

---

## Task 4: the directives that install

**Goal.** `::CLASS`, `::METHOD`, `::ATTRIBUTE` and `::CONSTANT` install. Bodies are stored. The
`ACTIVATE` protocol is Task 9's, not this one's.

**`::CONSTANT` carries a divergence recorded before it is fixed.** Its parenthesised expression form
is accepted here and never evaluated, where the oracle evaluates it at install. Two shapes, two
different exit codes, and the plan's first draft quoted one number for both:

```
::CLASS K  then  ::CONSTANT c (.NoSuchClass~m)   oracle rc 159, stdout empty, 97.1 on the expression
::CONSTANT sep (...) with no ::CLASS             oracle rc 157, stdout empty, 99.906, structural
both, on this crate                              rc 0, prologue runs
```

**A well-formed expression is rc 0 on both sides**, so the witness must make the expression fail.

**`::ANNOTATE naming a target` is not in scope**: the oracle refuses it too (99.945, rc 157), so it
is not an over-refusal and retiring it closes no divergence.

**Verification.** Oracle-differential per directive on both engines, including the failing-expression
`::CONSTANT` witness. Add each witness to `phase-5a.txt` in this commit.

**Done when** the four directives install and the `::CONSTANT` row moves from KNOWN GAPS to CLOSED
DEFECTS in `phase-4-exclusions.txt`.

---

## Task 5: the message send, the scope override, and the dispatch seam

**Goal.** `ExprKind::Message` and `InstructionKind::Message` evaluate against a **native** method.
Both engines.

**Shape.** D24: `resolve` and `invoke`, not a fused `send`. D28: dynamic, no per-call-site cache;
`ir.rs`'s `CallSite` is not extended to sends.

**`resolve` takes a start scope, and it is needed in this phase, not 5b.** `RexxObject::messageSend`'s
override (`ObjectClass.cpp:919`) reaches `superMethod(msgname, startscope)`, and `CoreClasses.orx`'s
`activate` uses `self~activate:super`, which installs before the prologue runs.

**The security seam, site one of two (D45).** Dispatch passes through exactly one chokepoint, with a
test asserting exactly one. No manager object. **Site two is Task 6's**; the first draft of this plan
put the seam only here and called it "the whole of the security work", which left the directories
with no seam at all.

**Verification.** Oracle-differential sends to each primitive kind, plus a `~msg:super` send, both
engines, with `ir_dual` live from here on. For the chokepoint, state what the test counts and what
would make it pass while a second dispatch path exists.

**`>M>` is this task's, not Task 7's.** Amended after the task ran: the prefix fires for *any*
message send (`RexxExpressionMessage::evaluate` and `RexxInstructionMessage::execute` both call
`traceMessage`), so a native send under `TRACE I` emits it and leaving it out would turn a loud gap
into a wrong trace. `trace_oracle.rs`'s `PREFIX_COVERAGE` row and its witness moved here.

**Done when** native sends and one scope-override send match the oracle on both engines.

---

## Task 6: the four directories, and the seam's second site

**Goal.** `.environment`, `.local`, `.context` and `.methods` become real objects (D33).

**Resolution order is not the obvious one.** `CoreClasses.orx:47-52` names classes declared later in
the same file **with no `PUBLIC` keyword**, so `.NAME` must consult the running package's own class
table **before** `.environment`.

**Close both `VALUE` routes together.** The one-argument form is silently wrong at rc 0 --
`say value('.LOCAL')` is `The Local Directory` on the oracle and `.LOCAL` here -- while `say .LOCAL`
exits 120. The three-argument empty-selector form is not this and is not in scope.

**The security seam, site two.** `.local`/`.environment` lookup passes through exactly one
chokepoint, asserted the same way as Task 5's.

**Verification.** Oracle-differential on `say .LOCAL`, `say .ARRAY`, `say .METHODS`, `say .context`,
both `value()` forms, and a non-public class resolved by name from inside its own package.

**Done when** the `UNATTRIBUTED:an environment symbol` rows leave `corpus/bif-exempt.txt`,
`expr_owner` gives `ExprKind::DotVariable` an owner, and both seam sites are asserted.

---

## Task 7: invoking a Rexx method body

**Goal.** A message that resolves to a `::METHOD` body enters it, binds `self`, runs, and returns.
Both engines.

**This is in 5a because `ACTIVATE` is.** Without it `CoreClasses.orx` cannot install.

**Scope:** entering and leaving a method activation, `self`, `RETURN` with and without a value, and
the trace prefixes a method entry produces. **Not** `~new`/`init`, **not** `FORWARD`, **not**
per-object methods -- those are 5b's.

**`>M>` landed in Task 5**, which is where the construct that emits it landed; its
`PREFIX_COVERAGE` row and witness are already in place. What this task owes is the trace a **Rexx
method activation** adds around it -- the `>I>`/`<I<` invocation lines and the callee's own clause
echoes -- and `ir_dual` cannot police any of their indents: both engines format trace through one
shared `trace.rs`, so a wrong indent is wrong identically on both arms. The instrument is in-crate
exact-stderr assertions in the shape of the existing indent tests in `rexx-exec/src/run/tests.rs`
(`a_message_sends_two_indents_are_the_oracles_own_and_normalisation_cannot_see_them` is Task 5's
own), with expected bytes **captured from the oracle**, not typed.

**Verification.** Oracle-differential on a class method and an instance method returning a value,
under no trace and under `TRACE I`, both engines, using Task 1's unnormalised mode for the traced
cases.

**Done when** a Rexx method body runs and its trace matches the oracle byte for byte, indent
included.

---

## Task 8: `EXPOSE` and scope-keyed instance variables

**Goal.** D40's representation, and the instruction that reaches it.

**`Body::Instance(Vec<(String, ObjRef)>)` is replaced.** An object holds one variable pool per
scope -- `RexxObject` chains a `VariableDictionary` per scope, created lazily, found by linear walk
(`ObjectClass.cpp:2489`). A pool is a full variable pool: measured, **an instance variable can be a
stem with tails, iterable by `DO OVER`**. The Rust shape is a small scope-to-pool association over
the storage an activation already uses.

**`EXPOSE` is not new machinery**: it binds names in the running method to that method's scope pool,
structurally what `PROCEDURE EXPOSE` already performs.

**Class-scope instance variables are what `ACTIVATE` uses**, so a class object carries a pool too.

**Verification.** Oracle-differential: a subclass and its parent each exposing the same name and
holding different values simultaneously; an exposed stem with tails; an exposed variable surviving
across two calls; a class method exposing a class-scope variable.

**Done when** the same-name-different-scope program matches the oracle, and a negative control
collapsing the two scopes into one pool reddens it.

---

## Task 9: the `ACTIVATE` protocol

**Goal.** Installing a class sends it `ACTIVATE`, before the program's first clause.

**Measured** on `::CLASS K` plus `::METHOD activate CLASS`: the oracle prints the activate output
first, then the prologue's; this crate prints only the prologue's. Record the divergence in
`phase-4-exclusions.txt` before closing it.

**Verification.** Oracle-differential on that program and on one where `activate` uses `EXPOSE` and
`self~activate:super`, both engines. Then `.TraceObject~option` after a bootstrap must be `N` and not
`OPTION` -- the shipped image carries the activated value, which is the discriminator between an
activate that ran and one that did not.

**Done when** activate output precedes the prologue's on both engines, byte for byte.

---

## Task 10: the native entry-point registry

**Goal.** `::METHOD ... EXTERNAL 'LIBRARY REXX name'` resolves at install (D37).

**It binds eagerly and a failure stops the program before its prologue.** Measured: a file whose
prologue is `say "prolog ran"` carrying one external method naming a missing entry point exits **166**
with **empty stdout** and `Error 90.998`. A real entry point exits 0 and prints.

**Build** a registry where every `LIBRARY REXX` name the three `.orx` files declare resolves; an entry
this phase does not implement raises **when invoked**, naming its owning phase. `file_separator` and
`file_path_separator` are implemented here as a stated scope addition, because `StreamClasses.orx`'s
constants call them at install.

**Verification.** Oracle-differential on the eager-bind failure. Then a corpus program per registered
family that **invokes** an unimplemented entry and pins the refusal -- which is writable only because
Task 7 landed method bodies, and would not have been under the first draft's scope.

**Done when** all three `.orx` files install with no unresolved external and invoking an unimplemented
entry is loud and names its phase.

---

## Task 11: the Package object and the setup methods

**Goal.** `addClass`, `addPublicClass`, `objectname=`, `publicClasses`, and the two setup methods.

**The setup methods are removed after the bootstrap** (D39). `removeSetupMethods()`
(`ClassClass.cpp:923`, called at `Setup.cpp:1809`) deletes `DEFINECLASSMETHOD` and
`INHERITINSTANCEMETHODS`. Measured on the shipped oracle:
`.String~hasMethod("DEFINECLASSMETHOD")` is 0, `.Supplier~hasMethod("INHERITINSTANCEMETHODS")` is 0,
`.Class~hasMethod("DEFINE")` is 1, and `.String~defineClassMethod(...)` is rc 159 with 97.1.

**Verification.** Oracle-differential on those four answers after a bootstrap.

**Done when** the prologue reaches its `~inherit` block and the four answers match.

---

## Task 12: the collection sends the prologue makes

**Goal.** `DO OVER` a collection object, the `[]` index message, and Directory `~put`.

**The protocol is `requestArray`**, and for a primitive-behaviour receiver it takes a C++ virtual
with no dispatch. The prologue also uses `do name over "nl", "cr", ...`, a comma-separated expression
list over a line continuation, which folds into a plain Array and is a different construct from
`do name over publicClasses`.

**Verification.** Oracle-differential on both `DO OVER` shapes and on `~put`/`[]` against a Directory,
both engines. The `LoopKind::Over` ownership move edits all five `owners.rs` pinned items here.

**Done when** both loop shapes and the index message match the oracle.

---

## Task 13: `rexx-lib`, the bootstrap driver, and the run

**Goal.** The three `.orx` files are embedded, located, and run (D26).

**`CoreClasses.orx` is not a main program and the first draft's exit condition was wrong.** Measured:
running it directly is **rc 159** on the oracle, `Object "REXXPACKAGE" does not understand message
"ADDCLASS"`, because `use arg rexxPackage` with no argument leaves the symbol as its own name. The
bootstrap **calls** the file with a Package object, the way `Setup.cpp:1795` passes `TheRexxPackage`
as a single argument. This task builds that driver and says what supplies the argument.

**Embed at build time** from this repository's tracked copy by a workspace-relative path, following
`crates/rexx-inventory/build.rs`. Record each file's sha256 so a drift is a build failure. **Never
edit them.**

**`rexx-lib` owns program-name resolution for the two `CALL`s**, intercepting the three bootstrap
names ahead of the external file search, which is Phase 7's.

**Verification, and its limits stated rather than implied.** **There is no oracle transcript for the
prologue** (D39): `removeSetupMethods()` deletes two methods it calls, so no shipped-oracle run
reaches its `exit`. What is checkable: the driver's exit status, empty stdout, empty stderr, and the
post-bootstrap state answering Task 3's six wiring questions for every class the prologue installs.
**Add a seventh check that the first draft lacked**: `.TraceObject~option` is `N`, which separates a
bootstrap that ran `ACTIVATE` from one that did not, and the donated-method sets on `.Supplier`,
`.Set`, `.Bag` and `.Relation`, which separate a real `inheritInstanceMethods` from a no-op.

**Done when** the driver exits 0 with empty stdout and stderr on both engines and all seven
post-bootstrap checks match the oracle.

---

## Task 14: the 5a gate

**Goal.** Prove 5a rather than assert it, and leave 5b a stated boundary.

**The subset** in `phase-5a.txt` must by now contain, added task by task: native sends to each
primitive kind; the six wiring answers plus donated-method sets per class; a mixin diamond
discriminating merge order from a chain walk; a class-behaviour-side witness; the four directives
including the failing-expression `::CONSTANT`; both `VALUE` routes and `say .LOCAL`; a Rexx method
body under `TRACE I` in unnormalised mode; the same-name-different-scope `EXPOSE` program; the
`ACTIVATE` ordering program; one unimplemented native entry per family; and both `DO OVER` shapes.

**Report, each a number or a named absence:** the five gate commands; the unsafe-block count and the
crate roots carrying `deny` rather than `forbid`, which every phase exit owes; the guard's
`across_builds` movement per axis against the 1% floor, with the note that the axes call none of the
new crates; and every committed table this plan edited.

**Done when** all five gate commands pass, the subset passes on both engines with the traced cases
unnormalised, and the report names what 5a did **not** cover -- `~new` and `init`, `FORWARD`,
per-object `~setMethod`, and the collection classes' own Rexx bodies -- so 5b starts from a stated
boundary rather than an assumption.
