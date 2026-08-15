# P-B: executability review of `docs/superpowers/plans/2026-08-15-phase-5a-native-layer.md`

**Lens:** executability under subagent-driven-development. A task is executable when a competent
implementer, seeing only the extracted brief, the interfaces the controller hands it and the Global
Constraints, can produce the right artefact and decide for itself whether it succeeded.

**Severity counts: 2 critical, 15 major, 11 minor** — 28 findings, plus one recorded non-finding
(PB-24). All are labelled CONFIRMED or PLAUSIBLE at the point they are stated; exactly one, PB-26,
is PLAUSIBLE, and one, PB-03, is CONFIRMED in its textual half and PLAUSIBLE in its consequence.
The index at the end carries the same labels.

---

## 0. What the implementer actually sees, established rather than assumed

The controller runs
`/home/moritz/.claude/plugins/cache/claude-plugins-official/superpowers/6.3.0/skills/subagent-driven-development/scripts/task-brief`.
Its whole body is one `awk` program:

```
!infence && /^#+[ \t]+Task[ \t]+[0-9]+/ { intask = ($0 ~ ("^#+[ \t]+Task[ \t]+" n "([^0-9]|$)")) }
intask { print }
```

So the brief is **exactly the `## Task N` section and nothing else**. This is not inference: three
briefs already exist in this directory, generated from this plan, and I read all three.
`task-1-brief.md` opens at `## Task 1: the behaviour table` and ends at the `---` before Task 2.

**PB-01 (major, CONFIRMED). The plan's own bounding section reaches no implementer.**
"What 5a is, and the observation that bounds it" — which carries *the prologue never runs a Rexx
method body*, *a task here that reaches for `EXPOSE`/`FORWARD`/instance variables has mis-scoped*,
and *there is no oracle transcript for the prologue* (D39) — is above `## Task 1` and therefore in no
brief. The last of those three is the single most expensive thing an implementer can get wrong: it
is the error that got a whole spec draft rewritten, and the plan says so in that unreachable
section. Tasks 7 and 9 each restate a fragment of it in their own text; Tasks 2, 3, 5 and 8 do not,
and any of them could plausibly try to diff against a bootstrap transcript.
*What this finding would look like if wrong:* the extractor would emit the file preamble, or the
generated briefs in this directory would contain it. Neither is the case.
*Fix:* either fold the three sentences into the Global Constraints the controller passes verbatim,
or repeat the no-transcript sentence in every task whose verification says "oracle".

---

## 1. Per-task

### Task 1 — the behaviour table

**Executable from the brief alone? No.** It must invent, and every invention is unreviewed:

* the type name (the brief calls it "a behaviour"; `rexx-core` already has `BehaviourTable` and
  `BehaviourEntry`, so the new name collides or shadows);
* whether it reuses `rexx_core::MethodId` and `rexx_core::BehaviourId`, or defines its own — the
  brief names neither type, and the spec's interface table (`resolve(...) -> MethodId`) is not in
  the brief;
* the signatures of `define`, `inherit` and lookup — the brief describes them in prose only;
* the version field's type and whether it is per-behaviour or global ("a per-behaviour **monotonic**
  version" fixes neither width nor overflow behaviour);
* the representation of "scope ordering" — `scopeList` plus `scopeOrders` is quoted as the C++
  shape, not specified as the Rust one;
* the crate's `Cargo.toml`, including whether it carries `[lints] workspace = true` (see PB-08);
* every file path in `crates/rexx-classes/`;
* five oracle probe programs and five test names.

**PB-02 (major, CONFIRMED). Goal and Done-when contradict each other.**
Goal: "**Replace** `rexx-core::BehaviourTable`'s lookup-time superclass walk … in the new
`rexx-classes` crate." Done-when: "no code outside `rexx-classes` has changed." You cannot replace a
`rexx-core` item without changing `rexx-core`. The implementer will resolve this silently, and the
two resolutions differ.
The tree makes the consequence concrete. I grepped for every use of `BehaviourTable` outside its own
module (`/bin/grep -rn "BehaviourTable" --include=*.rs .`): the only hits are
`crates/rexx-core/src/lib.rs`'s re-export and `crates/rexx-core/tests/behaviour.rs`. **Nothing in
`rexx-exec` calls it.** So `rexx-core::behaviour.rs` is dead except for its own test file, D29 says
its chain walk is "replaced, not extended" and D43 says "Only `BehaviourTable`'s chain-walking lookup
goes" — and **no task in this plan deletes it**. Task 1 is forbidden to; Tasks 2-10 never mention it.
5a will end with two behaviour implementations, one of them dead, and the deletion assigned to
nobody.

**Independently verifiable? Partly, and the plan does not admit the part that is not.**
Quoted: "Unit tests in `rexx-classes` carrying those oracle answers as expected values, each naming
the probe it came from. Plus a negative control: make `define` mutate in place instead of copying,
and confirm the third row's test goes red."

The negative control decides row 3 and only row 3. Rows 1 and 2 — merge order under multiple
inheritance, and which scope wins in a diamond — are the rows the brief itself says "a
plausible-looking wrong answer survives longest", and they have **no** control. Worse, they are not
checkable against the oracle in isolation at all: the oracle answers "the program printed `B`", and
the unit test asserts `resolve(behaviour, "M") == some MethodId`. The bridge between those two
statements is the implementer's own model of which `MethodId` corresponds to which Rexx method. A
wrong merge order plus a matching wrong bridge is green. Real validation arrives only when a Rexx
program runs through the structure, which is Task 3 at the earliest.

**PB-03 (major; CONFIRMED that the plan offers exactly one negative control for exactly one of five
rows — that is a read of its own text; PLAUSIBLE that rows 1, 2 and 5 are therefore unvalidatable in
isolation, which is reasoning I did not run). Task 1's verification is deferred and the plan does not
say so.** The controller's brief presents five oracle-pinned rows as if all five were decided here.
One is. *Fix:* say in Task 1 that rows 1, 2 and 5 are pinned but not *validated* until Task 3, and
move the discriminating diamond's end-to-end witness into Task 3's Done-when rather than Task 10's
subset list.

**One task or several?** One, but its verification has two independent failure modes (the copy
semantics; the merge order), and only one is instrumented. Splitting is not the fix — instrumenting
the second is.

---

### Task 2 — class objects, the registry, and the metaclass graph

**PB-04 (major, CONFIRMED). Task 2 cannot be verified before Task 3 exists.**
Verification, quoted: "The wiring assertion of criterion 3, per class, against the oracle: `~class`,
`~superClass`, **`~superClasses`**, `~isA`, `~metaClass`." Every one of those is a message send.
`ExprKind::Message` is `Owner::Phase("Phase 5")` in `crates/rexx-exec/tests/owners.rs:380` and is
built in **Task 3**. Done-when — "every class in the native set answers all five byte-identically to
the oracle" — is therefore unreachable at Task 2 and the implementer has no way to know that from its
brief. It will either build dispatch (Task 3's work, unreviewed as such) or report BLOCKED.

**PB-05 (major, CONFIRMED). The derived class list is derived in the half that does not matter and
hand-written in the half that does.**
I ran the plan's own enumeration: `/bin/grep -acE "createInstance\(\)" interpreter/memory/Setup.cpp`
answers **31**, and the 31 lines are the C++ class symbols `RexxClass`, `RexxInteger`, `RexxString`,
… `MutexSemaphoreClass`. The checklist Task 2 owes is over **Rexx** class names, because the five
wiring answers are Rexx expressions. The mapping is irregular: `RexxString`→`String` and
`RexxObject`→`Object` drop the prefix, but `RexxContext`→`RexxContext` and `RexxInfo`→`RexxInfo`
keep it; `ArrayClass`→`Array` drops the suffix, but `IdentityTable`→`IdentityTable` and
`StringTable`→`StringTable` have none. No rule generates it. So the bridge from the derived list to
the checked list is hand-maintained and unguarded — which is exactly what criterion 7 exists to
prevent, reinstated one step downstream of the guard.

Worse, two of the 31 have no Rexx name at all. Measured this session against the oracle
(`do n over .array~of(...) ; say n "->" .environment~hasIndex(n~upper) ; end`, rc 0):
`NumberString -> 0` and `Integer -> 0`; every other name I probed answered 1. So `RexxInteger` and
`NumberString` cannot answer `~class`/`~superClass`/`~superClasses`/`~isA`/`~metaClass` at all, and
Task 2's Done-when ("every class in the native set answers all five") either quietly excludes them —
undetectably, since the exclusion lives in whatever the implementer wrote — or blocks.
*What this check could not see:* I probed 31 spellings I guessed at. A class reachable only under a
name I did not guess would read as absent here; the guarded claim is only about `Integer` and
`NumberString`, whose C++ symbols I took from the grep above.

**PB-06 (minor, CONFIRMED). The build script's home is unstated.** "Add a build-script arm deriving
the class names … following `crates/rexx-inventory/build.rs`". `crates/rexx-inventory/build.rs`
reaches the C++ tree by `"../../../interpreter/memory/..."`-shaped relative constants and writes to
`OUT_DIR`. "Following" is ambiguous between *add an arm to `rexx-inventory`'s build script* and
*write `rexx-classes/build.rs` in the same style*. The first puts Task 2 into a crate outside the
perf guard's three; the second duplicates the `rerun-if-changed` discipline. Also, a build script
cannot shell out to `/bin/grep`; the plan gives the enumeration as a grep and no extraction rule for
what to capture from a matching line.

**Independently verifiable?** The deferral-table half is: "the deferral table is a test over the
derived list rather than a comment" is a real, checkable shape. The wiring half is not (PB-04).

**One task or several?** Three independent failure modes: the class objects and metaclass graph; the
registry; the derived checklist and deferral table. The third shares nothing with the first two and
is the one with a self-contained verification. I would split it out and let it land before or after
without ordering constraint.

---

### Task 3 — the native message send, and the security seam

**PB-07 (critical, CONFIRMED). Task 3 needs `phase-5a.txt`, which is Task 10.**
Task 3 moves `InstructionKind::Message` and `ExprKind::Message` from `Owner::Phase("Phase 5")` to
`Owner::InScope`. `crates/rexx-exec/tests/coverage.rs:727`'s
`every_in_scope_variant_is_witnessed_by_the_phase_subsets` then requires a program **in a phase
subset file** that constructs each. The subset files are pinned:
`const SUBSET_FILES: &[&str] = &["phase-4a.txt", "phase-4b.txt", "phase-4c.txt"]` appears four times
(`corpus.rs:472`, `coverage.rs:513`, `ir_dual.rs:1181`, `collect_stress.rs:137`), and each is
asserted against a directory read (`phase_subset_files_on_disk()`, which filters
`name.starts_with("phase-") && name.ends_with(".txt")` and sorts). Each of 4a/4b/4c additionally has
its exact line list pinned by `coverage.rs`'s `EXPECTED_SUBSET`, `EXPECTED_SUBSET_4B` and
`EXPECTED_SUBSET_4C`.

So Task 3's implementer has three options and the brief names none: put a `~` program into
`phase-4c.txt` and edit `EXPECTED_SUBSET_4C` (semantically wrong — 4c's own header excludes the
construct); create `phase-5a.txt` and edit all four `SUBSET_FILES` consts plus `trace_oracle.rs:661`'s
inline `["phase-4a.txt", "phase-4b.txt", "phase-4c.txt"]` (that is Task 10's entire Build section,
executed at Task 3 by a subagent that has never read Task 10); or leave the variants out of scope
and fail Task 3's own goal. **Task 10 as written then has nothing to build.**
The same trap catches Tasks 4, 5, 6 and 8 — Task 5's brief even says "or no corpus program can land"
while providing no file for one to land in.
*Fix:* move "create `rust/corpus/phase-5a.txt` and wire the five readers" out of Task 10 and make it
Task 1's or a new Task 0's, with Task 10 keeping only the population and the report.

**PB-08 (major, CONFIRMED). The one-chokepoint assertion has no mechanism and no negative control.**
Quoted: "a test asserts there is exactly one … a count of call sites that reach method invocation,
asserted at one, so that Phase 7 adds a hook to something that exists". *How* is left entirely to
the implementer. The only precedent in the tree for a test that reads source text is
`owners.rs:563`'s `the_two_harnesses_include_this_exact_file`, which does
`text.contains("#[path = \"owners.rs\"]")`. A count built the same way is a substring count over a
`.rs` file: it goes green on a rename, cannot see a call reached through a trait object or a closure,
and cannot distinguish a second chokepoint from a second *mention*. Task 1 gets an explicit negative
control; the deliverable the brief itself calls "the whole of the security work in this phase" gets
none. Under the project's own rule — *what would this check have done had the claim been false?* —
a text-count chokepoint test would have done the same thing.

**PB-09 (major, CONFIRMED). Criterion 5's other half is in no task.** D45 and gate criterion 5 read:
"Dispatch **and `.local`/`.environment` lookup** each pass through exactly one chokepoint … a test
asserts there is exactly one **per site**." Task 3 covers dispatch. Task 4 builds `.environment`,
`.local`, `.context` and `.methods` and its Verification and Done-when say nothing about a
chokepoint. No other task mentions one. Half of a numbered exit criterion is unassigned.

**PB-10 (major, CONFIRMED). D24's three surviving forward constraints are named nowhere in the
plan.** The spec: "selectors interned at compile time, a `SmallInt` behaviour arm, a receiver in the
calling convention. **The first three are this phase's and the plan owes a task for each**." I
grepped the plan for `SmallInt` (0 hits), `selector` (1 hit, unrelated — it is the `VALUE`
three-argument selector in Task 4) and `receiver` (1 hit, "resolve takes the receiver's behaviour").
The `SmallInt` arm is not optional decoration for Task 3: its Verification is "sending a native
message to each primitive kind", and an integer held inline has no `BehaviourId` today
(`rexx_core::BehaviourId` has exactly `STRING`, `ARRAY`, `OBJECT`, `STEM` —
`crates/rexx-core/src/body.rs:25-28`). The implementer must invent the whole primitive-kind→behaviour
mapping.

**PB-11 (minor, CONFIRMED). "Both engines" needs an env var the plan never names.**
`crates/rexx-exec/src/bin/rexx-run.rs:44` reads `REXX_ENGINE`, accepting `"ir"` and `"tree-walker"`;
`Engine::DEFAULT` is `Engine::Ir` (`crates/rexx-exec/src/invocation.rs:160`). Every task's
Verification says "on both engines" and no task says how. This is discoverable, which is why it is
minor, but six briefs each pay the discovery.

**Independently verifiable?** The oracle-differential half, yes. The chokepoint half, no (PB-08).

**One task or several?** At least three failure modes: `ExprKind::Message` evaluation;
`InstructionKind::Message` execution; the chokepoint assertion. The third is a different artefact
with a different instrument and shares no code path — it should be its own task, which would also
force its mechanism to be specified.

---

### Task 4 — `.environment`, `.local`, `.context`, `.methods`

**Facts check out.** I ran all four claims. Ours vs oracle, measured this session:

```
say .LOCAL           ours rc 120 "rexx-exec: an environment symbol is not implemented"  | oracle rc 0 "The Local Directory"
say .METHODS         ours rc 120 same                                                   | oracle rc 0 ".METHODS"
say .context         ours rc 120 same                                                   | oracle rc 0 "a RexxContext"
say value('.LOCAL')  ours rc 0 ".LOCAL"                                                 | oracle rc 0 "The Local Directory"
```

The plan's asymmetry argument is exactly right and the Verification list is well chosen.

**PB-12 (major, CONFIRMED). The Done-when is not achievable for four of its nine rows.**
Quoted: "**Done when** the `UNATTRIBUTED:an environment symbol` rows leave `corpus/bif-exempt.txt`".
There are nine such rows (`/bin/grep -av "^#" corpus/bif-exempt.txt | awk -F'\t' '{print $2}' | sort |
uniq -c` → 47 `4c`, 3 `ANOMALY`, 22 `Phase 5`, 9 `UNATTRIBUTED:an environment symbol`). Five are
closable here — I read them: `LENGTH::test025` is `length(...)`; `REVERSE::test017`/`test021` are
`reverse(.funnyconst)`; `REVERSE::test026` is `value(reverse(.okajsnah))`; `VALUE::test019` is
`value(.a)`. The other four are `STREAM::test_relative_file_exists`, `…exists2`,
`…not_exists`, `…not_exists2`, and their bodies (`ootest/ooRexx/base/bif/STREAM.testGroup:954`
onward) read `.ooRexxUnit.directory.separator` and then call `stream(file, 'c', 'query exists')`.
`corpus/builtin-status.txt:91` marks `STREAM` **excluded**. So once `.NAME` resolves, those four hit
the STREAM call instead — and the file's own header says the derived owner is `4c` "for every
builtin name rexx-exec runs nothing for". The rows **change attribution; they do not leave.** The
implementer discovers this only after the work is done, and then has to rule on it alone.

The file's header also warns that its category counts are pinned by
`every_exempt_attribution_is_a_known_phase_or_a_declared_outcome`, and that the repair for a red is
"rule on the row first … and then move the number to match, deliberately." Task 4's brief gives it
neither the rule nor the authority.

**PB-13 (minor, CONFIRMED). Giving `ExprKind::DotVariable` an owner is not free.** Done-when also
requires "`expr_owner` gives `ExprKind::DotVariable` an owner, both in this task's commit."
`owners.rs:580-624`'s pinned list item 5 says `lib.rs`'s `expr_owner` is the third copy of ownership
data and that `loud.rs`'s `every_out_of_scope_variant_fails_loudly` compares it to the tables — but
also records, measured, that "An owner written onto a variant this crate implements is data no
execution path reads, and **nothing covers it**". If Task 4 succeeds, `DotVariable` becomes
implemented, and the owner it is told to add is by that file's own account uncovered. The brief asks
for an edit whose only witness the tree says does not exist.

**One task or several?** Two failure modes that do not share an instrument: the four directory
objects and `.NAME` resolution order; and the `VALUE` one-argument route. The plan is right to close
them together (the asymmetry argument), so I would keep one task and split the Done-when into two
separately-checkable clauses.

---

### Task 5 — the directives that install

**Executable from the brief alone? Mostly, with two named inventions.**

**PB-14 (minor, CONFIRMED). `directive_gap` is never named.** Task 5 must delete or narrow
`crates/rexx-exec/src/lib.rs:1043`'s arm — `DirectiveKind::Class(class) if class.subclass.is_some()
|| class.metaclass.is_some() || !class.inherit.is_empty() => gap("::CLASS naming another class",
"Phase 5")`. The brief mentions neither `directive_gap`, nor `Loud`, nor `loud.rs`'s witness rows,
which must be *deleted* when a gap closes or `assert_witness_set_is_complete` fails the other way
(`owners.rs`, pinned item 4). The Global Constraints cover the owners.rs five; they do not name
`directive_gap`.

**PB-15 (minor, CONFIRMED). The `::CONSTANT` row already exists.** The brief says the divergence is
"recorded before it is fixed"; `docs/superpowers/plans/phase-4-exclusions.txt:2231` already carries
`A ::CONSTANT DIRECTIVE'S PARENTHESISED EXPRESSION IS NEVER EVALUATED…` under KNOWN GAPS. An
implementer reading "recorded before it is fixed" as an instruction adds a second row. Say "the row
is at `phase-4-exclusions.txt:2231`; move it."

**PB-16 (minor, CONFIRMED). "Widen the walker" is under-specified and reaches further than stated.**
`assert_program_has_only_routine_directives` (`coverage.rs:151`) rejects any non-`ROUTINE` directive
because "`each_instruction` descends into a routine's body and into no other kind of directive body".
Widening it means making `each_instruction` descend into `::METHOD`, `::ATTRIBUTE` and `::CONSTANT`
bodies — which changes what coverage counts as witnessed for **every** variant, not just the new
ones, and can flip `every_in_scope_variant_is_witnessed_by_the_phase_subsets` in either direction.
The brief says "widen the walker" and stops.

**Independently verifiable?** Yes, and well: "Oracle-differential per directive, including the
failing-expression `::CONSTANT` witness, on both engines", plus the observation that a valid
expression is rc 0 on both sides so the witness must fail. That last sentence is the best piece of
verification design in the plan — it names precisely what a naive probe could not see.

**One task or several?** Four directives, four independent failure modes, plus the walker widening
and the exclusions-file move. I would split `::CONSTANT` (which is a divergence closure with its own
witness and its own exclusions row) from `::CLASS`/`::METHOD`/`::ATTRIBUTE` (which are installation).

---

### Task 6 — the native entry-point registry

**PB-17 (major, CONFIRMED). "Family" is undefined, and it is what the gate is stated in terms of.**
Verification: "A corpus program per registered family that invokes one unimplemented entry and pins
the refusal". Task 10 repeats it: "one unimplemented native entry point per registered family". The
plan never defines a family. I enumerated the declarations
(`/bin/grep -aoiE "library rexx [a-z_]+"` over the two files): `CoreClasses.orx` declares
`alarm_startTimer`, `alarm_stopTimer`, `ticker_createTimer`, `ticker_waitTimer`, `ticker_stopTimer`;
`StreamClasses.orx` declares names with prefixes `file_`, `stream_`, `rexx_*_queue`, `query_`, plus
the unprefixed `handle_set`, `std_set`, `qualify` and `this_file_case_sensitive`. Any partition of
that set satisfies "one per family", including a partition of size one. The criterion is satisfiable
by construction and cannot fail. *Fix:* name the families in the plan, or restate the criterion as
"one program per *file* that declares externals, invoking one unimplemented entry from each".

**PB-18 (minor, CONFIRMED). Task 6 rewrites a Phase 7 refusal without saying so.**
`crates/rexx-exec/src/lib.rs:1020` is `DirectiveKind::Method(method) if method.external.is_some() =>
gap("::METHOD EXTERNAL", "Phase 7")`. Task 6's goal is that this form resolves at install. The brief
names neither the arm nor its `Phase 7` owner, and the owner label is a contract `loud.rs` pins with
an `ends_with`.

**PB-19 (minor, CONFIRMED). The two natives Task 6 implements are reached through a `PRIVATE` class
method.** `StreamClasses.orx:546-549`:

```
::method getSeparator private class external "LIBRARY REXX file_separator"
::method getPathSeparator private class external "LIBRARY REXX file_path_separator"
::constant separator (.File~getSeparator)
::constant pathSeparator (.File~getPathSeparator)
```

Evaluating those two constants at install time means invoking a **private class method** from the
directive-install context. `PRIVATE` appears nowhere in the plan. Task 3's `resolve` parameter list
(receiver behaviour, uppercased name, optional start scope, optional per-object table) has no
place to express caller scope, so if privacy has to be enforced the parameter is missing and
"retrofitting a parameter through every call site is the expensive order" — Task 3's own argument,
applied to the thing Task 3 left out.

**Independently verifiable?** The eager-bind shape, yes — the brief carries the exact measured
answer (rc 166, empty stdout, `Error 90.998`). The family criterion, no (PB-17).

**One task or several?** Two: the registry and the eager-bind semantics; and the two implemented
natives, which are a Phase 7 scope addition with a different oracle and a different risk.

---

### Task 7 — the Package object and the setup methods

**Executable? It must invent the largest single thing in the plan.** "Needs a live Package object
with `addClass`, `addPublicClass`, `objectname=` and `publicClasses`, plus `.methods`, plus the two
setup methods." Not given: the Rust type, where it lives, how `publicClasses` relates to Task 5's
`::CLASS` installation, what `.methods` holds and who populates it (Task 5 stores method bodies;
Task 7 claims `.methods` — the boundary is drawn nowhere), and the exact bytes of
`objectname` defaults.

**PB-20 (major, CONFIRMED). Nothing says how `rexxPackage` gets bound.**
`CoreClasses.orx:39` is `use arg rexxPackage`. `crates/rexx-exec/src/bin/rexx-run.rs:88-105` joins
every word after the program path into the single argument string a Rexx program sees, with no
option parsing at all. So `rexx-run CoreClasses.orx` supplies no object. Task 7 says the prologue's
first clauses run; Task 9 says `rexx-run CoreClasses.orx` exits 0. Neither says who constructs the
Package object or how it reaches `use arg`. See PB-22, which is the same hole seen from the exit
condition.

**Independently verifiable? Yes, and it is the best-instrumented task in the plan.** "Oracle-
differential on `hasMethod` for all three names after the bootstrap, and on sending
`~defineClassMethod` to a class, which must be `97.1` and rc 159." That check would come out
differently if the setup methods were left installed, which is exactly the property being asserted.

**One task or several?** Three failure modes: the Package object; `.methods`; the provide-then-remove
of the two setup methods. The third has its own oracle answer and its own risk (leaving them
installed is a divergence any program can see) and should be separable.

---

### Task 8 — the collection sends the prologue makes

**PB-21 (major, CONFIRMED). `ExprKind::List` is orphaned between Task 3 and Task 8, and neither names
it.** Measured this session:

```
do n over "a","b","c"   ours rc 120  stderr: rexx-exec: a parenthesised list is not implemented (Phase 5)
                        oracle rc 0  stdout: a\nb\nc
```

So Task 8's second `DO OVER` shape is blocked by `ExprKind::List`, which is
`("ExprKind", "List", "Phase 5")` in `owners.rs:379`. D34 assigns it to "the first commit that makes
`~` work" — Task 3 — and says it must become a **real Array**, because `(1,)~size` is 2 and
`(1,,)~size` is 3. `ExprKind::List` appears **zero** times in the plan (`/bin/grep -ac "ExprKind::List"`
→ 0; the spec has 2). Neither Task 3 nor Task 8 owes it, and D34's discriminator — the trailing-comma
arity — is in no task's verification, so a string-approximation implementation passes both.
The same grep found `ClassResolver`, `QualifiedCall`, `Call::Qualified` and `LoopKind::With` at zero
in the plan; each is a `Phase 5` row in `EXPECTED_OUT_OF_SCOPE`. Some are legitimately 5b/5c's, but
the plan never says which, so a 5a implementer meeting one has no ruling.

**PB-22 (minor, CONFIRMED). Task 8's ownership-move instruction has no subject.**
Quoted: "The ownership move edits all five `owners.rs` pinned items in this commit." But
`crates/rexx-exec/tests/owners.rs:247` already reads
`LoopKind::Over { .. } => ("Over", Owner::InScope)`, and `Over` is absent from
`EXPECTED_OUT_OF_SCOPE` (I read the whole list; its `LoopKind` row is `With`). Task 8's own text says
so — "`LoopKind::Over` is `Owner::InScope` in `owners.rs` today" — and then instructs an edit anyway.
The real move Task 8 needs is `ExprKind::List`, which it never names (PB-21).

**Independently verifiable?** Yes for the loop shapes and `~put`/`[]`; the ownership clause is
unverifiable because it has no subject.

**One task or several?** Two: the two `DO OVER` shapes (one needs `makearray`/`supplier` sends, the
other needs `ExprKind::List`) and the Directory `~put`/`[]` messages. They fail independently.

---

### Task 9 — `rexx-lib` and the bootstrap

**PB-23 (critical, CONFIRMED). Task 9's Done-when contradicts the Global Constraint it ships
under.**
Done-when: "`rexx-run CoreClasses.orx` exits 0 with empty stdout and stderr on both engines."
Global Constraints, first paragraph: "A change is right when output matches the C++ oracle byte for
byte on stdout, stderr and exit status."

Measured this session, from a fresh empty directory, under the mandated wrapper, three descriptors
read separately:

```
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  /home/moritz/dev/repos/ooRexx/build/bin/rexx \
  /home/moritz/dev/repos/ooRexx-rust-rewrite/interpreter/RexxClasses/CoreClasses.orx )

rc 159
stdout: (empty)
stderr:     47 *-* rexxPackage~addClass('LOCALSERVER', .LocalServer)
        Error 97 running .../CoreClasses.orx line 47:  Object method not found.
        Error 97.1:  Object "REXXPACKAGE" does not understand message "ADDCLASS".
```

So on the **same command line**, the oracle is rc 159 with three lines of stderr and this crate is
required to be rc 0 with none. The plan's own preamble explains why (the file needs a Package
argument that only `rexximage` supplies), but that explanation is in the unreachable section
(PB-01). A Task 9 implementer sees only: a Done-when demanding rc 0 and empty stderr, and a Global
Constraint demanding byte-for-byte agreement with an oracle that answers rc 159 and three lines. It
cannot satisfy both and cannot see which governs. It will either build an argument-injection path
nobody specified, or report BLOCKED, or — worst — quietly special-case the filename.

This is also a permanent, unrecorded divergence: after 5a, `rexx-run FILE` and `rexx FILE` disagree
for at least one real file, and nothing in `phase-4-exclusions.txt` licenses it. Contrast the
project's own discipline on `MAX_EVAL_DEPTH` and on OOM, both of which are written down as licensed
deviations.
*Fix:* give the bootstrap its own entry point (a flag, an env var, or an internal test hook), state
it in Task 9, and keep `rexx-run CoreClasses.orx` byte-identical to the oracle. If the divergence is
wanted, it needs a deviation row.

**PB-24 (minor, CONFIRMED, and it is a non-finding worth recording). The `.orx` files are tracked
here.** `git ls-files --error-unmatch` names all three of
`interpreter/RexxClasses/CoreClasses.orx`, `interpreter/RexxClasses/StreamClasses.orx`,
`interpreter/platform/unix/PlatformObjects.orx`. Task 9's "this repository's own tracked copy" claim
holds, and the `rexx-inventory/build.rs` precedent (relative constants plus `rerun-if-changed`) is a
real one.

**PB-25 (minor, CONFIRMED). The sha256 mechanism is unspecified and takes a policy decision.**
"Record each file's sha256 so a drift is a build failure." The workspace's only external dependency
is `datadriven`, and `rexx-exec/Cargo.toml`'s comment calls it "the only dependency this workspace
takes from outside it". A build-time sha256 needs `sha2` (present in the offline cache as
`sha2-0.10.9` and `sha2-0.11.0`, alongside `sha2-asm-0.6.4`), whose default features reach
`cpufeatures`. Under `unsafe_code = "forbid"` that is a decision, not a detail. The plan also does
not say where the expected digests are written, nor what a maintainer does when `interpreter/`
legitimately updates.

**PB-26 (major, PLAUSIBLE). The performance guard does not require a sitting on the task most likely
to move the numbers.** The rule is "Any task that lands code in `rexx-exec`, `rexx-core` or
`rexx-classes`". Task 9 lands `rexx-lib`. If the bootstrap runs at interpreter startup — D26 says
the files are *executed* and no image is built, and D2's threshold is "measured at exit" — then every
one of the six axes gains the bootstrap's cost and the 1% floor is blown on all six at once. The
plan never says whether the bootstrap runs on every `rexx-run` invocation or only on demand, and
that single unanswered question decides whether Task 9 is a no-op for the guard or the largest
regression in the phase. The spec flags the same hazard from the other side (`rexx-bench-suite`'s
`Role::Offset` doc becoming false, `write_axes` subtracting a now-large offset from every guarded
axis); neither appears in the plan (`/bin/grep -ac "Role::Offset"` → 0).

**Independently verifiable?** The honest half is: "**There is no oracle transcript to diff against**
-- say so in the report rather than implying one. What is checkable: the exit status, that stdout is
empty, that stderr is empty, and that the post-bootstrap state answers Task 2's five wiring
questions." That is good design. It is undermined by the exit status being the one that contradicts
the oracle (PB-23).

**One task or several?** Three: embedding plus digests; program-name interception for the two
`CALL`s; and running the bootstrap. The middle one touches `Loud::unresolved_call`'s neighbourhood
and fails independently of the other two.

---

### Task 10 — the 5a gate

**PB-27 (major, CONFIRMED). Nothing in the plan builds the unnormalised comparison mode, and the
gate is stated in terms of it.** Gate criterion 2 in the spec is explicit: "`corpus.rs`'s comparison
runs stderr through `normalize_stderr`. Criterion 2 says *byte for byte*, so the subset needs an
unnormalised comparison mode, or the criterion is weaker than it reads. **Build the mode.**"
`crates/rexx-exec/tests/corpus.rs:127` carries `# DEVIATION 0: leading indentation on stderr is
normalised`, and `crates/rexx-exec/tests/support/mod.rs:155` is the `normalize_stderr` it uses.
I grepped the whole plan for `normali` — **zero hits** (`/bin/grep -ain "normali"` exits 1). Task 10's
Done-when reads "the subset passes byte for byte on both engines", and the harness that decides it
does not compare byte for byte. This is the project's signature failure mode, in the task whose
entire purpose is to prove the phase: the check runs, exits 0, and cannot see indentation
divergence — which is precisely what the spec's trace section says is at stake.

**PB-28 (minor, CONFIRMED). `phase-5a.txt` gets no committed-contents pin.** 4a, 4b and 4c each have
one (`EXPECTED_SUBSET`, `EXPECTED_SUBSET_4B`, `EXPECTED_SUBSET_4C`, with
`phase_4a/4b/4c_subset_matches_the_committed_list`), and `owners.rs`'s pinned item 2 is exactly that
device. Task 10 says "Build `rust/corpus/phase-5a.txt` and wire it into every harness that reads a
subset list" and names the five readers correctly — but not the pin. Without it, a program can be
removed from `phase-5a.txt` later and the whole suite stays green while the headline shrinks, which
is the defect `corpus.rs:436-470`'s own comments describe having happened twice before.

**PB-29 (major, CONFIRMED). Task 10's unsafe/lints report is blind to the case this plan creates.**
Report item: "the unsafe-block count and the list of crate roots carrying `deny` rather than
`forbid`". This plan adds two crate roots. Every existing crate carries `[lints] workspace = true` —
I checked all eight `crates/*/Cargo.toml`. **Nothing in the workspace asserts that a crate does so**:
I grepped every `.rs` for `lints.workspace` and `unsafe_code` and the only hits are two prose
comments (`corpus.rs:396`, `support/oracle.rs:46`). A `rexx-classes` or `rexx-lib` whose
`Cargo.toml` omits the `[lints]` stanza inherits **neither** `forbid` nor `deny`, compiles clean, and
appears in Task 10's report as neither — because the report enumerates roots carrying `deny`, and
this root carries nothing. Had the claim "no crate has an unsafe exception" been false in this
particular way, the report would have looked identical.
*Fix:* Task 1 and Task 9 each state `[lints] workspace = true` explicitly, and Task 10 reports the
list of roots that do **not** inherit workspace lints, not the list carrying `deny`.

**Independently verifiable?** The five gate commands, yes. The subset's adequacy, no: the subset is
chosen by the same process that wrote the implementation, and the spec itself flags that ("a subset
chosen by the same person who wrote the implementation is a weak instrument"). The plan does not
answer it.

**One task or several?** Two, and they have opposite risk profiles: wiring the subset file into five
harnesses (mechanical, verifiable by five red-then-green tests) and populating it (judgement, and
the thing the spec says is weak). Per PB-07 the wiring must move earlier regardless.

---

## 2. The dependency table, pair by pair

Built from the tasks' own text plus the tree. Each row is *what one produces* against *what the
other consumes*.

| producer → consumer | shared file or interface | produced | consumed | gap |
|---|---|---|---|---|
| T1 → T3 | the flattened behaviour type in `rexx-classes` | a type, `define`, `inherit`, lookup, a version field — **no names or signatures stated** | `resolve(behaviour, name, start_scope, per_object)` | T3 must guess T1's spelling; the plan states neither side's signature |
| T1 → T2 | `BehaviourId` / behaviour ids | T1 forbidden to touch `rexx-core` | T2 needs two behaviour ids per class (D44) | who allocates ids is unstated |
| T2 → T3 | the class registry | name → class `ObjRef` | `~class`/`~superClass` answers | fine |
| T3 → T2 | message send | `ExprKind::Message` evaluation | **T2's own verification** | **inverted: T2 cannot be verified before T3** (PB-04) |
| T2 → T4 | the registry | class objects by name | `.ARRAY` resolution | fine |
| T3 → T4 | send | `.environment~put` | Task 4's Verification | fine |
| T2/T5 → T4 | the running package's class table | T5 installs `::CLASS` | `.NAME` consults it **before** `.environment` | T4 needs T5's table but is ordered before T5 |
| T5 → T3 | `MethodId → body` table | `::METHOD` bodies stored | `invoke` needs a body table | T3 is ordered first and has no bodies to invoke; "native method" is the escape, but the table's owner is unstated |
| T3 → T5 | `::CONSTANT` expression evaluation | expression eval incl. `~` | install-time evaluation | fine |
| T5 → T6 | `::METHOD ... EXTERNAL` install | directive install | eager bind | fine |
| T6 → T5 | `directive_gap`'s `::METHOD EXTERNAL` arm | — | both tasks must edit `directive_gap`; neither names it | collision (PB-14, PB-18) |
| T3 → T8 | `ExprKind::List` | **nobody produces it** | T8's second `DO OVER` shape | **orphan** (PB-21) |
| T4 → T7 | `.methods` | T4 makes it a real object | T7 says "plus `.methods`" | duplicated ownership, boundary undrawn |
| T5 → T7 | method objects in `.methods` | bodies stored | `.methods[…]` yields a Method | who populates `.methods` is unstated |
| T7 → T9 | the Package object | "a live Package object" | `use arg rexxPackage` binding | **neither says how it is bound** (PB-20, PB-23) |
| T5/T6/T9 → T6 | `.File` class | T5's `::CLASS` in `StreamClasses.orx` | `.File~getSeparator` at install | `.File` is D11's Phase 7; the line is undrawn |
| T3/T4/T5/T6/T8 → T10 | `rust/corpus/phase-5a.txt` + four `SUBSET_FILES` consts + `trace_oracle.rs:661` | T10 | every task that lands a corpus program | **inverted; T10's build section is consumed by five earlier tasks** (PB-07) |
| T3 → T10 | `every_in_scope_variant_is_witnessed_by_the_phase_subsets` | T3 moves `Message` in scope | a witness program in a subset file | same inversion, enforced by a test |
| T9 → guard | `rexx-bench-suite`'s `Role::Offset`, `write_axes` | bootstrap cost at startup | the six guarded axes | unaddressed (PB-26) |

---

## 3. The plan-wide questions

### Does the stated order work?

**No, in two places.**

Tracing from nothing: T1 (a crate nothing links) → T2 (class objects; **verification blocked on T3**)
→ T3 (send; **blocked on T10's subset file, by a test**) → T4 (directories; needs T5's package class
table for the non-public-class rule it itself states) → T5 (directives) → T6 (externals) → T7
(Package) → T8 (collections; **needs `ExprKind::List`, which no task produces**) → T9 (bootstrap;
**needs an argument-injection path no task produces**) → T10 (gate; **its Build section was already
forced at T3**).

The two hard inversions are PB-07 (T3 needs T10) and PB-04 (T2 needs T3). PB-21 and PB-20 are not
inversions but holes — work no task produces at any point.

### Is Task 1 verifiable in isolation?

**Partly, and the deferral is not admitted.** Three of the five probe rows — the two-mixin merge, the
diamond, and the class-behaviour-side mixin — cannot be validated against the oracle by a unit test,
because the oracle's answer is "the program printed X" and the unit test's subject is a `MethodId`.
The bridge between them is the implementer's model, so a consistently-wrong model is green. Only the
`~define` copy row has a negative control, and it is the row least likely to be got wrong. Task 3
is indeed the first thing that can observe the structure; the plan places the discriminating diamond
in Task 10's subset list, which is nine tasks of work after the shape is fixed. This is Task 1's own
stated risk — "the one thing in the phase that a plausible-looking wrong answer survives longest" —
left uninstrumented by the task that names it. (PB-03.)

### Where is the two-build sitting ceremony, and where is it real?

The rule fires on `rexx-exec`, `rexx-core` or `rexx-classes`.

* **Ceremony — T1, T2.** Both land only in `rexx-classes`, which at that point is a workspace member
  nothing links. `target/release/rexx-run` is built from unchanged sources, so the sitting measures
  code placement and machine drift and nothing else. The plan's own floor rule then invites the task
  to spend a paragraph attributing a ≥1% reading to "the layout" — a conclusion known in advance.
  Recording "no code reachable from `rexx-run` changed; sitting skipped" would be the honest form.
* **Real — T3, T4, T8.** T3 adds arms to the expression evaluator and a new op to the compiled
  stream; T4 adds a resolution step to `.NAME`; T8 touches the loop path. These three can move
  `varlookup`, `emptyloop` and `arith` for reasons that are the change and not the layout, and the
  sitting is the only thing that would attribute them.
* **Mixed — T5, T6, T7.** Install-time code, off every guarded axis, but each edits files the hot
  path shares. Cheap, worth keeping.
* **Missing — T9 (PB-26).** `rexx-lib` is not one of the three named crates, so the rule as written
  does not require a sitting on the one task that could add work to every program's startup. Whether
  it does depends on a question the plan never answers.

Not a finding, checked: `bench-baselines/phase-5a-arms.tsv` does not exist, but
`crates/rexx-bench/src/bin/rexx-arms.rs:151-170` creates it with a header on first use. Two smaller
gaps in the quoted command: `--task <task> --commit <commit>` are unfilled placeholders, and nothing
tells the implementer to `cargo build --release` first even though `--build head=target/release/rexx-run`
requires it.

### What is missing entirely?

Work 5a's own exit condition requires that no task covers:

1. **The argument-injection path that binds `use arg rexxPackage`** — PB-20/PB-23. Without it Task 9
   cannot reach rc 0; with it, `rexx-run` diverges from the oracle and nothing licenses that.
2. **The unnormalised comparison mode** the spec's criterion 2 says in as many words to build —
   PB-27. Zero occurrences of `normali` in the plan.
3. **The `.local`/`.environment` chokepoint and its assertion**, half of gate criterion 5 — PB-09.
4. **`ExprKind::List` as a real Array**, D34, required by Task 8's own second `DO OVER` shape and by
   the `(1,)~size == 2` discriminator — PB-21.
5. **D24's three surviving forward constraints** — interned selectors, a `SmallInt` behaviour arm, a
   receiver in the calling convention — which the spec says "the plan owes a task for each" — PB-10.
   The `SmallInt` arm is a precondition of Task 3's own verification.
6. **Deletion of `rexx-core::BehaviourTable`'s chain walk**, which D29 and D43 both require and which
   Task 1's Done-when forbids — PB-02.
7. **A committed-contents pin for `phase-5a.txt`** — PB-28.
8. **An explicit `[lints] workspace = true` on the two new crates**, plus a report shaped to see its
   absence — PB-29.
9. **`::ANNOTATE naming a target`.** Task 5 rules it out of scope on the ground that the oracle
   refuses it too. True, but the spec's answer is "what this phase owes is matching the oracle's
   refusal **bytes**, not removing ours" — our refusal today is `directive_gap`'s, at a different rc
   and a different shape from the oracle's 99.945/rc 157. Task 5 retires the obligation without
   discharging it, and no task picks it up.

---

## 4. What I searched for, and what these searches could not reach

* The extraction mechanism: read the `task-brief` script itself and the three already-generated
  briefs, rather than inferring from the SKILL.
* Subset wiring: `/bin/grep -rn "SUBSET_FILES|phase-4a.txt|phase-4b.txt|phase-4c.txt"` over
  `--include=*.rs`, then read each hit's surrounding assertion. A harness reading the corpus
  directory under a name I did not grep for would be invisible here; I cross-checked against the
  plan's own list of five and found the same five.
* Plan coverage of spec obligations: `/bin/grep -ac` for `ExprKind::List`, `ClassResolver`,
  `QualifiedCall`, `Call::Qualified`, `LoopKind::With`, `SmallInt`, `normali`, `Role::Offset`,
  `apply_binary`, `PREFIX_COVERAGE`, `hyperfine`. A construct the plan refers to only by prose
  description would read as absent; I read the plan end to end as well, which is how the
  `LoopKind::Over`/`ExprKind::List` confusion in Task 8 surfaced.
* Lints inheritance: read all eight `crates/*/Cargo.toml`, then `/bin/grep -rn` for
  `lints.workspace|unsafe_code` over every `.rs`. A CI-side check outside `rust/` would be missed;
  I did not search `.github/` or `ci/`.
* Oracle probes: five programs, each from a fresh `mktemp -d` under the scratchpad, absolute paths,
  the mandated `ulimit`/`LD_LIBRARY_PATH` wrapper, stdout/stderr/rc read as three descriptors, never
  `2>&1`. `CoreClasses.orx` is not in `corpus/oracle-crashes.txt`, which I read before running it.
* One claim of mine was wrong mid-review and is recorded rather than dropped: I first concluded
  `rexx-run` had no engine selector, from reading `main`'s argument handling. It has one —
  `REXX_ENGINE`, at `bin/rexx-run.rs:44` — and the finding shrank from "unrunnable" to "unnamed"
  (PB-11).
* Not attempted: I did not run the negative control for PB-07 (creating `rust/corpus/phase-5a.txt`
  and watching four tests redden), because the ground rules forbid modifying the repository. That
  finding rests on reading `assert_eq!(SUBSET_FILES, phase_subset_files_on_disk(), …)` and the
  filter `name.starts_with("phase-") && name.ends_with(".txt")` in all four harnesses, plus each
  harness's own doc comment stating that this is the behaviour it is there to produce. It is the one
  major finding in this report whose confirmation is a source read rather than a run.

---

## 5. Findings index

| id | sev | label | task |
|---|---|---|---|
| PB-07 | critical | T3 needs T10's subset file, enforced by a test | 3, 10 |
| PB-23 | critical | T9's Done-when contradicts the byte-for-byte Global Constraint; oracle is rc 159 | 9 |
| PB-01 | major | the plan's bounding section, incl. D39, reaches no brief | all |
| PB-02 | major | T1 goal vs Done-when contradict; `BehaviourTable` never deleted | 1 |
| PB-03 | major | T1's verification is deferred and unadmitted; control covers 1 of 5 rows | 1 |
| PB-04 | major | T2 cannot be verified before T3 | 2 |
| PB-05 | major | derived class list is C++ symbols; the Rexx-name bridge is hand-written; 2 of 31 unnameable | 2 |
| PB-08 | major | one-chokepoint test: no mechanism, no negative control | 3 |
| PB-09 | major | criterion 5's `.local`/`.environment` chokepoint is in no task | 3, 4 |
| PB-10 | major | D24's three forward constraints named nowhere; `SmallInt` blocks T3's own verification | 3 |
| PB-12 | major | T4's Done-when unachievable for 4 of 9 rows (STREAM is `excluded`) | 4 |
| PB-17 | major | "registered family" undefined; the criterion cannot fail | 6, 10 |
| PB-21 | major | `ExprKind::List` orphaned; blocks T8's second `DO OVER`; D34's discriminator untested | 3, 8 |
| PB-26 | major | guard rule omits `rexx-lib`, the task most able to move every axis | 9 |
| PB-27 | major | no task builds the unnormalised comparison mode; "byte for byte" is measured by a normaliser | 10 |
| PB-29 | major | new crates can inherit no lints at all; the report is shaped not to see it | 1, 9, 10 |
| PB-06 | minor | build-script home and extraction rule unstated | 2 |
| PB-11 | minor | `REXX_ENGINE=tree-walker` unnamed | all |
| PB-13 | minor | `expr_owner` edit for `DotVariable` has no witness, by `owners.rs`'s own account | 4 |
| PB-14 | minor | `directive_gap`/`loud.rs` edits unnamed | 5 |
| PB-15 | minor | `::CONSTANT` KNOWN GAP already exists at `phase-4-exclusions.txt:2231` | 5 |
| PB-16 | minor | "widen the walker" under-specified; reaches all coverage | 5 |
| PB-18 | minor | T6 rewrites a `Phase 7` refusal arm without saying so | 6 |
| PB-19 | minor | `PRIVATE` class method at install time; `resolve` has no caller-scope parameter | 6 |
| PB-22 | minor | T8's ownership move has no subject (`Over` already `InScope`) | 8 |
| PB-25 | minor | sha256 mechanism unspecified; takes a dependency policy decision | 9 |
| PB-28 | minor | `phase-5a.txt` gets no committed-contents pin | 10 |
| PB-20 | major | nothing binds `use arg rexxPackage` (same hole as PB-23, seen from T7) | 7 |
| PB-24 | — | non-finding: the three `.orx` files are tracked; T9's premise holds | 9 |
