# Review P-C -- facts, and conformance to the spec

**Subject:** `docs/superpowers/plans/2026-08-15-phase-5a-native-layer.md` (committed at `e72a2427f`).
**Authority:** `docs/superpowers/specs/2026-08-15-phase-5-object-model.md`.
**Reviewer lens:** every factual claim verified against the tree and the oracle; then D25-D45 built
from the spec and checked against the plan's ten tasks.

Every oracle run below used exactly
`( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`
from a fresh empty directory
(`.../scratchpad/pc/probes`), with stdout, stderr and exit status read as three descriptors and
never `2>&1`. "This crate" means `rust/target/release/rexx-run`, the `b029abe77` binary the ground
rules name. Nothing in the repository was modified.

Counts: **6 high, 7 medium, 5 low.**

---

## HIGH

### H1. D45's second chokepoint is dropped, and the plan asserts the drop is complete -- CONFIRMED

The plan, Task 3: *"**The security seam (D45), and it is the whole of the security work in this
phase.** Dispatch passes through exactly **one** chokepoint, and a test asserts there is exactly
one."*

D45 has two sites, not one: *"Dispatch **and `.local`/`.environment` lookup** each pass through
exactly one chokepoint a manager could later hook, and a test asserts there is exactly one **per
site**."* Criterion 5 repeats it: *"Dispatch and `.local`/`.environment` lookup each route through
exactly one chokepoint."*

Task 4 is the task that builds `.environment`, `.local`, `.context` and `.methods`. I read all of it:
it names no chokepoint, no count assertion, and nothing about a manager hook. So the
`.local`/`.environment` half is in neither Task 3 nor Task 4 -- and because Task 3 declares itself
*the whole of the security work in this phase*, 5b and 5c inherit nothing either. Criterion 5 cannot
be met by the phase as planned.

This is the failure mode the assignment names: a decision silently dropped, and here it is dropped
under a sentence claiming completeness.

**What the plan's own check would do had the claim been false:** the same thing. Task 3's
one-chokepoint test counts dispatch sites; it is structurally blind to how many places reach
`.local`.

### H2. Criterion 2's unnormalised comparison mode is assigned to no task, and Task 10 asserts byte-for-byte through the normalising harness -- CONFIRMED

The plan, Task 10 done-when: *"the subset passes byte for byte on both engines"*.

The spec names two harness prerequisites for criterion 2 and says the plan owes a task for each:

> * `corpus.rs`'s comparison runs stderr through `normalize_stderr`. Criterion 2 says *byte for
>   byte*, so the subset needs an unnormalised comparison mode, or the criterion is weaker than it
>   reads. **Build the mode.**
> * `coverage.rs`'s `assert_program_has_only_routine_directives` ... **Widen the walker.**

The plan implements the second (Task 5: *"Widen `coverage.rs` first"*) and never mentions the first.

Verified: `/bin/grep -an "normalize_stderr" rust/crates/rexx-exec/tests/corpus.rs
rust/crates/rexx-exec/tests/support/*.rs` gives
`crates/rexx-exec/tests/support/oracle.rs:302: if super::normalize_stderr(&rust.stderr) !=
super::normalize_stderr(&cpp.stderr)`. The corpus comparison is normalised today, and no task in
this plan changes that. Task 10's "byte for byte" is therefore a claim its own instrument cannot
support -- it would report a pass on a program whose only divergence is inside what
`normalize_stderr` collapses.

### H3. `::ANNOTATE` is ruled out of scope on a premise I measured false -- CONFIRMED

The plan, Task 5: *"`::ANNOTATE naming a target` is **not** in scope: the oracle refuses it too
(99.945, rc 157), so it is not an over-refusal and retiring it closes no divergence."*

The oracle figures are right. I ran `say "hi"` + `::annotate routine nosuchrtn`:

```
oracle   rc 157, stdout empty
         stderr: `     2 *-* ::annotate routine nosuchrtn` / `Error 99 ... Translation error.`
                 / `Error 99.945:  ::ANNOTATE target routine "NOSUCHRTN" not found.`
this crate  rc 120, stdout empty,
         stderr: `rexx-exec: ::ANNOTATE naming a target is not implemented (Phase 5)`
```

**There is a divergence.** rc 157 against rc 120, and entirely different stderr. "Not an
over-refusal" is true; "closes no divergence" is false, and the plan's conclusion rests on the false
half. The spec had this right and the plan reverses it: *"what this phase owes is matching the
oracle's refusal bytes, not removing ours."* Criterion 2 asks for a witness for each of
`directive_gap`'s four Phase 5 arms, `::ANNOTATE` among them.

### H4. D34 is silently dropped, and its own timing rule places it inside 5a -- CONFIRMED

D34: *"`ExprKind::List` is a **real Array** from the first commit that makes `~` work."* The spec's
reasoning: *"`(1,)~size` is 2 and `(1,,)~size` is 3, which no string approximation reproduces, and
which is unobservable until dispatch lands and observable immediately after."*

Task 3 of this plan is the commit that makes `~` work. `ExprKind::List` appears nowhere in the plan
-- I searched the plan text for `List`, `ExprKind::List`, `Array literal` and `D34`.

The tree confirms this is a live obligation, not a formality:
`rust/crates/rexx-exec/tests/owners.rs:238` reads
`ExprKind::List(_) => ("List", Owner::Phase("Phase 5"))`, and `EXPECTED_OUT_OF_SCOPE` carries
`("ExprKind", "List", "Phase 5")`. So the moment Task 3 lands, `(1,,)~size` becomes observable and
answers wrongly, with nothing in the plan noticing.

### H5. Eight Phase-5-owned AST variants fall into no sub-phase under the plan's stated boundary -- CONFIRMED

The plan: *"**5b** is Rexx method bodies -- `EXPOSE`, `FORWARD`, scoped instance variables, `~new`
and `init`. **5c** is the sourced library classes and the corpus gate."*

I enumerated `EXPECTED_OUT_OF_SCOPE`'s Phase 5 rows from `rust/crates/rexx-exec/tests/owners.rs`
itself (lines 364-460), not from the spec's copy:

```
InstructionKind: Call::Qualified, Expose, Options, Message, Guard, Reply, Forward
ExprKind:        QualifiedCall, ClassResolver, List, Message
LoopKind:        With
```

Against the plan's three buckets: 5a takes `InstructionKind::Message` and `ExprKind::Message`
(Task 3). 5b takes `Expose` and `Forward` by name. That leaves **`Call::Qualified`, `Options`,
`Guard`, `Reply`, `QualifiedCall`, `ClassResolver`, `List` and `LoopKind::With`** matching neither
"Rexx method bodies" nor "sourced library classes and the corpus gate".

Four of those carry named spec decisions: D31 (`::OPTIONS` and the `OPTIONS` instruction *"stay in
this phase"*), D32 (`REPLY`/`GUARD` run-time legality check), D30 (`::REQUIRES` *"lands last in this
phase"*), D34 (`List`, see H4). The plan's boundary sentence is where they disappear, and it is
stated as a definition of 5b and 5c rather than as an open remainder -- so a reader of 5b's future
plan will not find them either.

### H6. Task 4's done-when demands a state that contradicts Task 4's goal -- CONFIRMED

The plan, Task 4: *"**Done when** the `UNATTRIBUTED:an environment symbol` rows leave
`corpus/bif-exempt.txt` and `expr_owner` gives `ExprKind::DotVariable` an owner, both in this task's
commit."*

The two conjuncts pull opposite ways, and the tree says which way each pulls:

* `rust/crates/rexx-exec/tests/owners.rs:220` reads
  `ExprKind::DotVariable(_) => ("DotVariable", Owner::InScope)`.
* `rust/crates/rexx-exec/src/lib.rs:1219-1229`: `expr_owner` returns `None` for
  `ExprKind::DotVariable`. `Some("Phase 5")` is reserved for `QualifiedCall`, `ClassResolver`,
  `Message` and `List`.
* `rust/crates/rexx-exec/tests/loud.rs:436-452`,
  `every_out_of_scope_variant_fails_loudly`: for every witness of a phase-owned tag, `if
  outcome.exit_code != NOT_IMPLEMENTED_EXIT` is a failure.

So giving `DotVariable` an owner requires a committed witness program that still exits 120 on a
`.NAME` -- which is exactly what Task 4 exists to stop. Meanwhile the exempt rows *can* leave:
`corpus/bif-exempt.txt:40-45` says those rows *"read a `.NAME` the file never sets -- REVERSE's
`.funnyconst`, VALUE's `.a` -- which the oracle renders as the symbol's own uppercased text"*, and
that is fallback behaviour, not `.File`. The plan has read the spec's *"`expr_owner` gives
`ExprKind::DotVariable` no phase today, which is why the refusal is unowned"* as an instruction to
add an owner, when the refusal is what goes away.

---

## MEDIUM

### M1. The second `::CONSTANT` shape diverges here too, and the plan's table hides it -- CONFIRMED

The plan, Task 5:

```
::CLASS K then ::CONSTANT c (.NoSuchClass~m)   oracle rc 159, stdout empty
                                              this crate rc 0, stdout "prolog"
::CONSTANT sep (...) with no ::CLASS           oracle rc 157, 99.906, structural
```

Both oracle rows are exactly right -- I reproduced them (rc 159 with `Error 97.1: Object
".NOSUCHCLASS" does not understand message "M"`; rc 157 with `Error 99.906: A ::CONSTANT directive
with an expression requires a matching ::CLASS directive`).

But the second row gives no "this crate" column, and it needs one. Running
`say "prolog"` + `::CONSTANT sep (.NoSuchClass~getThing)` through
`rust/target/release/rexx-run`: **rc 0, stdout `prolog`, stderr empty.** The structural 99.906 check
is a second, independent divergence -- we do not raise it at all. `phase-4-exclusions.txt:2231-2266`
records only the expression-evaluation gap (I read the whole row); nothing records this one, and
Task 5's done-when (*"the `::CONSTANT` gap's row moves from KNOWN GAPS to CLOSED DEFECTS"*, singular)
does not reach it. The table's asymmetric presentation is what makes it invisible.

### M2. The ownership-move obligation is attached to the task where nothing moves, and omitted from the task where two things do -- CONFIRMED

Task 8: *"`LoopKind::Over` is `Owner::InScope` in `owners.rs` today"* -- correct,
`owners.rs:247` reads `LoopKind::Over { .. } => ("Over", Owner::InScope)`. Task 8 then says *"The
ownership move edits all five `owners.rs` pinned items in this commit."* There is no ownership move
in Task 8: `Over` is already `InScope` and stays `InScope`.

Task 3, which moves `InstructionKind::Message` (`owners.rs:175`, `Owner::Phase("Phase 5")`) and
`ExprKind::Message` (`:239`, same) into scope and must therefore delete their
`EXPECTED_OUT_OF_SCOPE` rows, their `loud.rs` witnesses, the `variant_counts_match_the_audited_split`
counts and their `expr_owner`/`instruction_owner` entries, says nothing about the five items in its
own done-when. The Global Constraint covers it generically; the per-task obligation is on the wrong
task.

### M3. `DO OVER` an object sends neither `makearray` nor `supplier` on the path the prologue takes -- CONFIRMED

Task 8: *"Over an object it must send `makearray` or `supplier`."*

`interpreter/instructions/DoBlockComponents.cpp:230-247` is the `DO OVER` setup:

```
    if (isArray(result))      { array = ((ArrayClass *)result)->makeArray(); }
    else                      { array = result->requestArray(); ... }
```

and `RexxInternalObject::requestArray` (`interpreter/classes/ObjectClass.cpp:1646-1666`) branches on
`isBaseClass()`: a primitive receiver gets a **direct C++ `makeArray()` call, no message send at
all**; only a Rexx subclass sends, and it sends `REQUEST` with argument `ARRAY`
(`GlobalNames::REQUEST, GlobalNames::ARRAY`), not `MAKEARRAY` and never `SUPPLIER`.

The prologue's `do name over publicClasses` (`CoreClasses.orx:63`) has a StringTable receiver -- a
base class -- so it takes the no-send branch. An implementation built to "send `makearray` or
`supplier`" gets the protocol wrong in both halves. `supplier` has no basis in this path.

### M4. D24's three surviving forward constraints get no task, though the spec says the plan owes one each -- CONFIRMED

Spec: *"D24's forward constraints for Phase 5 ... are five ... D28 disposes of the last two by
declining the cache. **The first three are this phase's and the plan owes a task for each**"* --
selectors interned at compile time, a `SmallInt` behaviour arm, a receiver in the calling convention.
Listed again under Open questions.

I searched the plan for `intern`, `SmallInt`, `calling convention`, `receiver in` and `D24`. Task 3
cites D24 only for the `resolve`/`invoke` split. None of the three appears. They are also not
assignable to 5b or 5c under the plan's boundary sentence (H5).

### M5. Criterion 4 and criterion 9 land in no sub-phase, and Task 10's report list omits both -- CONFIRMED

Task 10 lists what the report owes: *"the five gate commands; the unsafe-block count and the list of
crate roots carrying `deny` rather than `forbid` ...; the guard's `across_builds` movement per axis
against the 1% floor; and every committed table this plan edited."* That is criteria 6 and 8.

Criterion 4 (`>M>` and `>N>` pinned by in-crate exact-stderr assertions, bytes captured from the
oracle, `PREFIX_COVERAGE` rows updated) appears nowhere in the plan except as a passing mention of
`trace_oracle.rs`. Verified live:
`rust/crates/rexx-exec/tests/trace_oracle.rs:627` is `(">M>", Coverage::Owned("Phase 5"))` and `:631`
is `(">N>", Coverage::Owned("Phase 5"))` -- still owed, still unassigned.

Criterion 9 (cold start measured against `build/bin/rexx` with
`rexx-bench/src/bin/rexx-time.rs --warmup 10 --runs 50`, closing D2) is the criterion most obviously
triggered by Task 9's bootstrap landing, and the plan does not mention it -- nor the spec's two
consequential warnings about `rexx-bench-suite`'s `Role::Offset` doc becoming false and `write_axes`
subtracting a suddenly-large offset from every guarded axis.

Neither fits "Rexx method bodies" or "the sourced library classes and the corpus gate".

### M6. `trace_oracle.rs`'s failure mode is not the one the plan describes -- CONFIRMED

Task 10: *"**`trace_oracle.rs` has no such guard** and will silently keep measuring the 4a/4b/4c
union."*

The first half is right. `/bin/grep -an "read_dir\|phase_subset_files_on_disk"
rust/crates/rexx-exec/tests/trace_oracle.rs` returns nothing, while `corpus.rs:482`, `coverage.rs`,
`ir_dual.rs:1198` and `collect_stress.rs:181` each define `phase_subset_files_on_disk()` and assert
`SUBSET_FILES` against it. The plan's four-plus-one split is correct.

The second half is not. `trace_oracle.rs:661` uses the hardcoded list inside
`every_live_witness_emits_its_prefix_and_is_run_by_the_corpus`, whose body concatenates the three
files and then asserts that each `Coverage::WitnessedLive` row's program appears in that text,
failing with *"{rel_path} witnesses {prefix:?} but is in no phase subset file"*. A Phase 5 witness
listed only in `phase-5a.txt` makes that test go **red, loudly**, not silently green. The harness
does not "measure" the union; it membership-checks against it. The remedy the plan prescribes is
right; the reason given for it is wrong, and a task author who trusts the reason will look for a
silent-pass that is not there.

### M7. Six spec obligations addressed to "the plan" are unanswered -- CONFIRMED by reading both documents

The spec's Open questions section ends each item by assigning it to the plan. Unanswered here:

* *"What a `REPLY` or `GUARD` inside a Phase 5 method does"* -- reachable the moment Task 5's
  `::METHOD` installs.
* *"Which `CoreClasses.orx` classes this phase leaves unexercisable ... The list belongs in the
  plan."* (`Alarm` and `Ticker`.)
* *"What plays the oracle for a native method ... The plan should say there is no third case."*
* *"The roadmap's first-listed reason for the IR ... needs amending or a recorded reason it
  survives."*
* *"Where the `phase-5.txt` subset's programs come from ... a subset chosen by the same person who
  wrote the implementation is a weak instrument."* Task 10 lists contents and says nothing about who
  chooses them.
* D33's *"What each directory must *hold* is not enumerated here and some of it is another phase's
  -- `.File` is Phase 7's by D11 -- so the plan draws that line."* Task 4 draws no such line.

Also unassigned: the spec's method-frame traceback divergence (*"the oracle's stderr opens with
`*-* Compiled method "+" with scope "String".` and this crate's does not ... **The plan owes a
task**"*).

---

## LOW

### L1. `owners.rs`'s five pinned items are not in its module doc -- CONFIRMED

Plan: *"`owners.rs`'s module doc names five items that move together"*. The module doc is
`owners.rs:12-43` and names none of them. The list is a plain `//` comment block at `:580-624`
(*"any task moving an `InstructionKind`, `ExprKind` or `LoopKind` variant into scope ... must update
every one of the five items below"*). The five items themselves are quoted correctly, item 4 being
`INSTRUCTION_WITNESSES`/`EXPR_WITNESSES` which the plan calls "loud.rs's witness tables". Only the
location is wrong. (The spec makes the same slip; the plan inherits it.)

### L2. The committed-table list drops `corpus/builtin-status.txt` -- CONFIRMED

The spec's list of tables Phase 5 must edit ends with *"`corpus/builtin-status.txt`, derived from a
live differential"*. The plan's Global Constraints list is `owners.rs`'s five, `corpus/bif-exempt.txt`,
`corpus/keyword-exempt.txt`, `assertions.rs`'s `EXEMPT` and `trace_oracle.rs`'s `PREFIX_COVERAGE`.
`builtin-status.txt` exists (`ls rust/corpus/*.txt`) and is gone from the plan;
`keyword-exempt.txt`, which the spec does not list, is added. The substitution is unexplained.

### L3. `rexx-inventory/build.rs`'s paths are crate-relative, not workspace-relative -- CONFIRMED

Plan, Task 9: *"by a workspace-relative path, the way `crates/rexx-inventory/build.rs` already
reaches the C++ tree"*. That build script uses
`const MESSAGES_XML: &str = "../../../interpreter/messages/rexxmsg.xml";` -- three levels up from
`rust/crates/rexx-inventory`, i.e. resolved against the build script's own working directory, not the
workspace root. The doc quote the plan gives -- *"The C++ tree is the source of truth. Nothing here
is hand-maintained"* -- is verbatim (`build.rs:3-4`, continuing *"and nothing generated is written
into `src/`"*).

### L4. The `forbid` line is not itself the record of exceptions -- CONFIRMED

Plan: *"The workspace sets `unsafe_code = "forbid"`; that line is the record of which crates hold an
exception"*. `rust/Cargo.toml:11-22` says the opposite of the second clause: *"the choice of `forbid`
versus `deny` **at a crate root** is the record of whether that crate has been granted an unsafe
exception"*, and *"An approved site does not relax this line"*. The workspace line is uniform; the
record is per-crate-root. The plan's conclusion ("this plan adds none") is unaffected.

### L5. Nothing in this crate reaches an external file search today -- CONFIRMED

Plan, Task 9: *"`call 'StreamClasses.orx'` would otherwise reach the external file search, which
`Loud::unresolved_call`'s own doc assigns to Phase 7."* The doc quote is accurate --
`rust/crates/rexx-exec/src/lib.rs:595` reads *"External routine resolution is **Phase 7's**"* -- but
the same doc says the one step this crate skips *is* the file search, and that a name matching
nothing is *"the oracle's own Error 43.1 (`Raised::routine_not_found`)"*, not `unresolved_call`.
Measured: `say "before"` + `call 'StreamClasses.orx'` through `rexx-run` gives **rc 213**, stdout
`before`, stderr `Error 43.1: Could not find routine "StreamClasses.orx"`. So Task 9's interception
sits ahead of a 43.1 raise, not ahead of a search that exists; the shape of the change is different
from what the sentence describes.

---

## Conformance table, D25-D45

Built from the spec's own "Decisions recorded here" section, read in file order.

| D | plan's treatment |
|---|---|
| D25 native/sourced line discovered, `Setup.cpp` a checklist, wiring via five questions | **implements** -- Task 2 |
| D26 three `.orx` executed from tracked copy, never edited; D2 measured at exit | **partial** -- Task 9 does the embedding; D2's measurement is in no task (M5) |
| D27 new crates, boundary is a trait, `resolve` takes start scope + per-object table | **partial** -- Task 3 has both `resolve` parameters; the trait for running a body is unnamed (5b's, plausibly, but unstated) |
| D28 dynamic, no per-call-site cache | **implements** -- Task 3 |
| D29 flattened, scope-ordered, cascade, monotonic version | **implements** -- Task 1, all four bullets |
| D30 `::REQUIRES` lands last in this phase | **silent** -- no sub-phase (H5) |
| D31 `::OPTIONS` and `OPTIONS` stay in this phase | **silent** -- no sub-phase (H5) |
| D32 `REPLY`/`GUARD` run-time legality check | **silent** -- no sub-phase (H5) |
| D33 four directories real, `.NAME` before `.environment`, one-arg `VALUE` | **implements**, minus the "what each directory holds" line (M7) |
| D34 `ExprKind::List` a real Array from the first `~` commit | **silent** -- **H4** |
| D35 six guard axes, 1% floor, per-task sitting | **implements** -- Global Constraints |
| D36 the gate is the nine criteria | **partial** -- criteria 4, 5 (half), 9 unassigned |
| D37 native entry-point registry, `file_separator`/`file_path_separator` here | **implements** -- Task 6 |
| D38 `::CONSTANT` expression form | **implements**, minus the 99.906 shape (M1) |
| D39 no oracle transcript; setup methods provided then removed | **implements** -- Task 7, and stated again in the plan's preamble |
| D40 `Body::Instance` replaced by a scope-keyed pool; `EXPOSE` is one task | **deferred to 5b**, consistent with the plan's bound |
| D41 object identity not modelled | **silent** -- no task needs it; no contradiction |
| D42 literals pooled per compiled unit, three constraints | **silent** -- no sub-phase |
| D43 object holds a behaviour reference; `define` copies, `inherit` does not | **implements** -- Task 1 |
| D44 a class carries two behaviours | **implements** -- Task 2 ("two behaviour ids (D44)"), and Task 1's fifth probe row is the class-behaviour witness D44 asks for |
| D45 seam only, one chokepoint **per site** | **contradicts** -- **H1** |

No decision is contradicted outright except D45; the failures are drops.

---

## What I checked and found sound

Everything below came out exactly as the plan states it.

**C++ citations.** All seven line numbers land on the construct named.
`ClassClass.cpp:819` is `RexxClass::defineMethod`'s definition line; `:923` is
`RexxClass::removeSetupMethods`, doc-commented *"Remove the special class methods that are defined
just for image building"*; `:1036` is `updateSubClasses`; `:1071` is `updateInstanceSubClasses`;
`:1148` is `createInstanceBehaviour`. `ObjectClass.cpp:919` is the scope-override `messageSend`
overload and its body reaches `superMethod(msgname, startscope)` at `:928`. `Setup.cpp:1809` is
`TheClassClass->removeSetupMethods();`, sitting after the `runProgram` at `:1798` and before
`saveImage` at `:1812`.

**Quoted comment fragments.** *"make a copy of the instance behaviour so any previous objects aren't
enhanced"* is verbatim at `ClassClass.cpp:860-861` (also at `:531`, in `defineClassMethod`), inside
the function the plan cites. *"may have an impact on metaclasses"* is verbatim at `:1049-1050`; the
plan's surrounding paraphrase says "instance methods" where the C++ says "the added methods", which
does not change the claim.

**Cascade asymmetry.** `updateInstanceSubClasses` (`:1080-1084`) recurses into
`updateInstanceSubClasses` -- instance side only, as Task 1's `define` bullet says.
`updateSubClasses` (`:1039-1053`) clears and rebuilds both behaviours, instance first, and recurses
into `updateSubClasses`. `RexxClass::inherit` (`:1340-1361`) does `superClasses->addLast(mixin_class)`
with no copy and then calls `updateSubClasses()` -- Task 1's `inherit` bullet, exactly.

**Every measured transcript.** All reproduced this session, three descriptors separate:

| probe | result |
|---|---|
| `say "prolog ran"` + `::METHOD zz EXTERNAL 'LIBRARY REXX nosuchentry'` | oracle **rc 166**, stdout **empty**, `Error 90.998: Unable to find external method "nosuchentry"` |
| same with `alarm_startTimer` | oracle **rc 0**, stdout `prolog ran` |
| `::CLASS K` + `::CONSTANT c (.NoSuchClass~m)` | oracle **rc 159**, stdout empty, `Error 97.1`; this crate **rc 0**, stdout `prolog` |
| `::CONSTANT sep (...)` with no `::CLASS` | oracle **rc 157**, `Error 99.906 ... requires a matching ::CLASS directive` (structural, as stated) |
| `.String~hasMethod("DEFINECLASSMETHOD")` / `.Supplier~hasMethod("INHERITINSTANCEMETHODS")` / `.Class~hasMethod("DEFINE")` | **0 / 0 / 1** |
| `.String~defineClassMethod("ZZ", .nil)` | oracle **rc 159**, `Error 97.1: Object "The String class" does not understand message "DEFINECLASSMETHOD"` |
| `say value('.LOCAL')` | oracle rc 0 `The Local Directory`; this crate rc 0 `.LOCAL` |
| `say .LOCAL` | oracle rc 0 `The Local Directory`; this crate **rc 120** `an environment symbol is not implemented` |
| `say .context` / `say .methods` | oracle rc 0 `a RexxContext` / `.METHODS`; this crate rc 120, both |
| `say value('.LOCAL',,'')` | oracle rc 0 **`..LOCAL`** -- the plan is right that this is a different gap and out of scope |
| `::annotate routine nosuchrtn` | oracle rc 157, `Error 99.945` (see H3 for our side) |

**`CoreClasses.orx:47-52`, every name.** Lines 47, 49, 50, 51, 52 name `.LocalServer`,
`.SupplierMixin`, `.ManyItemMixin`, `.SetMixin`, `.BagMixin` (line 48 is a comment). All five are
declared later in the same file and **none carries `PUBLIC`**: `::class "SupplierMixin"` (`:172`),
`::class "ManyItemMixin"` (`:218`), `::CLASS 'SetMixin'` (`:411`), `::CLASS 'BagMixin'` (`:557`),
`::CLASS "LocalServer"` (`:974`). The plan's inference -- `.NAME` must consult the package's own
class table before `.environment` -- follows.

**The prologue's bound.** `use arg rexxPackage` is `CoreClasses.orx:39` and `exit` is `:126`; the
plan's `:39-126` is exact. `PackageClass::addClassRexx`/`addPublicClassRexx` are C++
(`PackageClass.cpp:1926`, `:1944`, registered at `Setup.cpp:1188`), so the plan's claim that the
prologue's messages land on natives holds at the two least obvious receivers.

**Task 6's scope addition is exactly two, not approximately two.**
`/bin/grep -ain "^::constant.*(" CoreClasses.orx StreamClasses.orx` returns exactly
`StreamClasses.orx:548:::constant separator (.File~getSeparator)` and
`:549:::constant pathSeparator (.File~getPathSeparator)`, whose targets at `:546-547` declare
`file_separator` and `file_path_separator`. No third install-time-evaluated constant exists in either
file. `CoreClasses.orx` declares exactly five `EXTERNAL` methods (`:1590`, `:1618`, `:1690`, `:1691`,
`:1692`) and all five are timers, as stated.

**The `Setup.cpp` derivation.** `/bin/grep -aE "createInstance\(\)"
interpreter/memory/Setup.cpp` returns 31 lines, one class each, in the order the spec quotes.
Task 2's proposed build-script arm is well-founded and the enumeration is clean enough to parse.

**In-repo citations, other than those under findings.** `coverage.rs:151-166`'s
`assert_program_has_only_routine_directives` filters `d.kind.keyword() != "ROUTINE"` and is called at
`:753` on the union of `SUBSET_FILES` (`:513`, `["phase-4a.txt", "phase-4b.txt", "phase-4c.txt"]`) --
so it does panic on any `::CLASS`, `::METHOD`, `::ATTRIBUTE` or `::CONSTANT`, and Task 5's "or no
corpus program can land" is right. `rust/Cargo.toml:113-124`'s `[profile.release]` sets `debug`,
`lto` and `codegen-units` and **not** `debug-assertions`, so the plan's reason for the extra debug
gate run holds. `unsafe_code = "forbid"` is at `Cargo.toml:22`. `ootest/` is in `.gitignore:6` and
`ootest/.svn` exists. `phase-4-exclusions.txt:2678` is *"KNOWN GAP: VALUE ON A DEFINED ENVIRONMENT
NAME IS SILENTLY WRONG AT RC 0"*, and `:2231-2266` is the `::CONSTANT` row with a transcript matching
the plan's.

**The `rexx-arms` command would run as written.** `rexx-arms.rs:101-121` parses `--build` (repeated,
`LABEL=PATH`), `--axis` (repeated), `--rounds`, `--task`, `--commit` and `--baseline`; every flag the
plan uses exists and none it omits is required. `--baseline` on a non-existent path creates the file
with a header (`:150-160`), so `bench-baselines/phase-5a-arms.tsv` being new is fine.
`bench-baselines/pinned/rexx-run-pre-phase-5` exists, as do `target/release/rexx-arms` and
`target/release/rexx-run`. Axis names resolve through `rexx_bench::program_path` to
`bench-programs/<name>.rex`, and `alloc4c.rex`, `arith.rex`, `compound.rex`, `emptyloop.rex`,
`strings.rex`, `varlookup.rex` are all present. The command is the one
`perf-baseline.md:1011-1014` recorded for the pinned baseline, plus the second `--build`. The floor
rule's provenance checks out too: `bench-baselines/README.md:32,35` carries both the 7.8% and the
`+/-0.74%` figures the spec cites.

**Harness guards.** `corpus.rs:482/501`, `coverage.rs`, `ir_dual.rs:1198/1464` and
`collect_stress.rs:181/204` each read the corpus directory and assert `SUBSET_FILES` against it;
`trace_oracle.rs` has no `read_dir` anywhere. The plan's four-plus-one split is correct (only its
description of the consequence is not -- M6).

## What I searched for and could not reach

* I searched the plan text for `List`, `SmallInt`, `intern`, `receiver`, `REQUIRES`, `OPTIONS`,
  `REPLY`, `GUARD`, `>M>`, `>N>`, `cold start`, `hyperfine`, `rexx-time`, `builtin-status`,
  `normalize`, `chokepoint` and `.local`. Absences reported above are absences of those terms; a
  task could in principle cover one of them under wording none of those terms reaches.
* I did not build. Every Rust claim is read from source or run through the pre-built
  `b029abe77` binary the ground rules point at, so a claim about a test's *runtime* behaviour
  (H6's `loud.rs` mechanism, H2's normalisation) is read from the code rather than executed.
* I did not run a negative control on Task 1's proposed `define`-copy test, because the code does not
  exist yet. Task 1 prescribes one and prescribes it correctly.
* I did not attempt to run `CoreClasses.orx` under either interpreter. The spec's D39 argument that
  no shipped-oracle run reaches its `exit` is supported here only indirectly, through the three
  `hasMethod` answers plus `Setup.cpp:1809` preceding `saveImage` -- which is the same evidence the
  spec offers, not an independent one.
