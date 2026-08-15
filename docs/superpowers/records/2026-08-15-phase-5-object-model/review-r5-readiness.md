# R5 -- implementation-readiness and decomposition

Reviewer R5. Lens: whether this spec can become a plan whose tasks are executable by fresh subagents
that see one brief each and never the whole document.

## What this reviewer could and could not run

**I had no shell in this session.** The tools available to me were file reads only, so I ran no oracle
probe, no `cargo`, no `/bin/grep`, and no negative control. The ground rules' three instruments were
therefore reduced to one and a half: I could read a file and I could compare two files, and I could
not remove a thing and watch a check go red.

Every finding below is labelled. **CONFIRMED** means I read the file this session and the text would
have read differently had the finding been wrong, and I name the file and the line. **PLAUSIBLE**
means I reasoned from what I read without executing anything. No number below is carried forward out
of a document without saying which document.

Files read in full or in the named range this session: the spec, the ground rules,
`2026-08-15-phase-5-inherited-surface.md`, `phase-4-exclusions.txt` (lines 1-963),
`perf-baseline.md` (lines 1-140, 300-699 partly, 700-1080), `rust/bench-baselines/README.md`,
`rust/crates/rexx-bench/src/bin/rexx-bench-suite.rs` (lines 1-440),
`rust/crates/rexx-bench/src/bin/rexx-arms.rs` (lines 1-160),
`rust/crates/rexx-bench/src/bin/rexx-bench-band.rs` (lines 1-140),
`rust/crates/rexx-exec/tests/owners.rs`, `rust/crates/rexx-exec/tests/corpus.rs` (lines 1-310),
`rust/corpus/README.md`, `rust/corpus/phase-4a.txt`, `phase-4b.txt`, `phase-4c.txt`,
`rust/crates/rexx-core/src/body.rs` (lines 1-120), `rust/crates/rexx-core/src/behaviour.rs`,
and in the read-only oracle tree `interpreter/RexxClasses/CoreClasses.orx` (lines 38-197 and
3400-4193), `interpreter/RexxClasses/StreamClasses.orx` (lines 1-90) and
`interpreter/platform/unix/PlatformObjects.orx` (whole file).

**What I searched for and could not reach.** I could not grep, so I cannot claim any enumeration over
the tree is exhaustive. Specifically: I read two ranges of `CoreClasses.orx` and not the middle, so
any claim of the form "the file needs only X" is not available to me and I make none. I did not read
`rexx-exec/src/lib.rs`'s `instruction_owner`/`expr_owner`/`directive_gap`, `run.rs`, `eval.rs`,
`ir.rs`, `activation.rs`, `tests/loud.rs`, `tests/coverage.rs`, `tests/trace_oracle.rs`, or
`rexx_bench::arms`. Where a finding would have been strengthened by one of those, I say so.

---

## Verdict

**Not ready.**

The spec is a good design document and a poor plan input. Its central method -- the native set is
discovered by running `CoreClasses.orx` -- is a loop with no stated bound, no exit condition other
than "it runs", and no per-iteration size limit, and the spec leaves it that way while its own exit
criterion 1 depends on the loop terminating. Independently of that method, the spec omits from its
scope statement constructs that `CoreClasses.orx` visibly requires and that this tree already
attributes to Phase 5, and its exit criterion 2 requires retiring a refusal that its own D31 moves out
of the phase.

---

## 1. Can this be decomposed into tasks at all?

### 1.1 The decomposition the spec implies

Reading the spec as a plan author would, the units it implies are:

* U1. `rexx-classes` crate skeleton: class object, registry, `.environment`/`.local` as directories (D33).
* U2. The flattened method dictionary and the `updateSubClasses` cascade, replacing `BehaviourTable`'s chain walk (D29).
* U3. `resolve`/`invoke` wired into `step`, `eval.rs` and `Op::Send` (D24, D28).
* U4. `::CLASS` naming another class, retiring the first `directive_gap` refusal.
* U5. `::METHOD` and `::ATTRIBUTE` installation.
* U6. `ExprKind::List` as a real Array (D34).
* U7. `REPLY`/`GUARD` translation-time legality (D32).
* U8. `>M>`/`>N>` trace lines with in-crate exact-stderr assertions (D36).
* U9. `rexx-lib`: embedding, sha256 build script, the bootstrap runner (D26, D27).
* U10. The discovery loop: run `CoreClasses.orx`, implement whatever it refuses, repeat.
* U11. `::REQUIRES`, last (D30).
* U12. The `phase-5.txt` corpus subset and the gate.
* U13. The `Setup.cpp` deferral table as a test over the registry (D25).

### 1.2 The units that cannot be independent

**F1 (blocking, CONFIRMED). U10 is not a task, it is an unbounded loop, and the spec's own evidence
shows the loop is avoidable.**

The spec says the native set is "grown one refusal at a time". That phrasing implies each iteration
is small and each is discoverable only by running the previous one. Neither holds.

The executable top level of `CoreClasses.orx` is lines 39 to 126, terminated by `exit` at line 126,
with everything after it a directive. I read that whole range this session. It is enumerable **today**,
statically, without running anything. It contains, in order:

* `use arg rexxPackage` (line 39), so the bootstrap must pass a Package object as an argument.
* `rexxPackage~addClass('LOCALSERVER', .LocalServer)` (line 47) and three more `addClass` calls.
* `.environment~objectname = "The Environment Directory"` (line 55), an **assignment whose target is a
  message send**, and `rexxPackage~objectname = "The REXX Package"` (line 56).
* `.context~package~publicClasses` (line 61).
* `do name over publicClasses` (line 63), `DO OVER` on a collection object.
* `publicClasses[name]` (line 64), `.environment~put(class, name)` (line 65),
  `rexxPackage~addPublicClass(name, class)` (line 66).
* `do name over "nl", "cr", ...` (lines 70-72), `DO OVER` over a **comma list of expressions** with a
  line continuation.
* `.String~defineClassMethod(name~upper, .methods[("string_cls_" || name)~upper])` (line 73), which
  needs the `.methods` environment symbol.
* `.supplier~inheritInstanceMethods(.SupplierMixin)` and its four siblings (lines 80-87).
* `.string~inherit(.Comparable)` and the rest of the real-inherit block (lines 93-119), which is the
  multiple-inheritance cascade D29 is about.
* `call 'StreamClasses.orx' rexxPackage` (line 122) and `call 'PlatformObjects.orx' rexxPackage` (line 124).

Every one of those is a named, sized obligation available to a plan author now. The loop exists
because nobody read those lines, not because the information is only obtainable by execution.

**What a plan author should write instead.** Replace U10 with:

* A **spine** of tasks derived from a static read of `CoreClasses.orx` lines 39-126 and
  `StreamClasses.orx` lines 39-49, each task named for the clause or clause group it makes run, each
  with its own oracle transcript.
* One **residual discovery task** with a stated budget (a number of refusal cycles, or a wall-clock
  bound) and a named artifact it writes when the budget is exhausted, because a subagent has nobody to
  escalate to. The artifact is the deferral table D25 already requires, so this costs nothing new.
* A stated rule for what happens when one discovery cycle turns out to be larger than a task: the
  spec must say whether the subagent implements it, stubs it, or writes it into the table and stops.

**F2 (blocking, CONFIRMED). The discovery instrument changes character partway through and the spec
does not say so, which makes "one refusal at a time" false after U4-U5 land.**

`directive_gap` refuses at install time, before `main` (`phase-4-exclusions.txt`, "EXCLUSIONS -- every
directive except `::ROUTINE`", read this session: the refused forms produce "stdout EMPTY in every
case"). So today's refusal is a *translation-time* event. Once `::CLASS`, `::METHOD` and `::ATTRIBUTE`
install, the file's directives all install and execution reaches line 47, and every subsequent failure
is a *run-time* failure at one clause. The two failure modes are different instruments: the first
enumerates directive forms and stops before any code runs, the second enumerates clauses and reveals
exactly one per cycle. The spec's "grown one refusal at a time" describes the first and is applied to
the whole phase.

**F3 (blocking, CONFIRMED). U9's embedding decision collides with a Phase 7 exclusion, and the spec
does not resolve the collision.**

`call 'StreamClasses.orx' rexxPackage` (CoreClasses.orx:122) is a named CALL matching no internal
label, no builtin and no `::ROUTINE`. `phase-4-exclusions.txt`'s section "EXCLUSIONS -- external
routine resolution, Phase 7" (read this session) assigns exactly that resolution to Phase 7: "Resolving
a name to a file needs the search path ..., file loading, and a second Package object". D26 says the
three files are embedded rather than read from a search path. So the spec requires a CALL on a literal
program name to resolve against an embedded table, and says nothing about how. Two subagents will
build two different mechanisms: one special-cases the three names in `rexx-lib`, one implements
external-file call resolution and thereby lands Phase 7 work inside Phase 5.

The spec must state the contract: what `CALL 'name'` consults, in what order, and whether the embedded
table is visible to a user program's `CALL 'StreamClasses.orx'`.

**F4 (blocking, CONFIRMED). Two units both own `owners.rs`'s pinned literals, and the spec never
names that file.**

`rexx-exec/tests/owners.rs` (read in full this session) carries `EXPECTED_OUT_OF_SCOPE`, the
`variant_counts_match_the_audited_split` hardcoded counts, and `SPLIT_TABLE_PHASES`. Its own closing
comment enumerates the pinned items a task moving a variant into scope must edit **in the same
change**: `EXPECTED_OUT_OF_SCOPE`, `coverage.rs`'s `EXPECTED_SUBSET`, this file's own counts,
`loud.rs`'s `INSTRUCTION_WITNESSES`/`EXPR_WITNESSES`, and `src/lib.rs`'s
`instruction_owner`/`expr_owner`.

U3 (`Message`), U6 (`List`), U4/U5 (the directive arms), U7 (`Reply`, `Guard`) and the residual rows
(`Expose`, `Forward`, `With`, `Call::Qualified`) each move one or more rows across that line. Every one
of them edits the same constants. The spec mentions none of these files. Two subagents landing
`Message` and `List` in parallel collide on `EXPECTED_OUT_OF_SCOPE` and on the same integer literals in
`variant_counts_match_the_audited_split`.

The spec should either serialise those tasks explicitly or state that ownership-table edits are a
single task's job at the end of each unit, with the edit spelled out.

**F5 (blocking, CONFIRMED). U12 has four call sites, not one, and this exact incompleteness has
already shipped twice in this tree.**

`corpus.rs`'s module doc, read this session, records that `phase-4c.txt` "was added, `coverage.rs` read
it, and this runner did not -- so four programs written as differential witnesses were only ever being
*parsed*, and a criterion-1 witness that nothing runs is a witness that cannot fail." Its `read_subset`
doc adds that this call site "was the one `read_subset` caller of four still pinned to a single file".

The spec's whole instruction for U12 is "It extends the existing `corpus.rs` mechanism". A subagent
reading that brief adds `phase-5.txt` and wires it into `corpus.rs`, which is precisely the incomplete
change the file's own history records twice. The brief must name every `read_subset` caller.

---

## 2. Sentences two competent implementers would read differently

**F6 (blocking, CONFIRMED). "the wiring of every class this phase does build is asserted against the
oracle" (D25).**

Reading A: one differential program per class, in `phase-5.txt`, sending `~class`, `~superClass`,
`~isA` and `~metaClass` and comparing stdout against the oracle.
Reading B: an in-crate table of expected answers transcribed from the oracle once, asserted in a unit
test.

The two differ in what happens when the oracle's answer changes and in whether the assertion re-reads
the oracle. Criterion 2 names the corpus route; D25 names neither. `~isA` also takes an argument and
the spec never says which class to pass, so even under reading A two implementers write different
programs. The risks table says "the four-answer oracle assertion above, taken per class, not at the
end", which adds a third axis (per class, per task) without resolving the first two.

Fix: state it as "a `phase-5.txt` program per class sending `~class`, `~superClass`, `~metaClass` and
`~isA(.Object)`, compared byte for byte, added in the same task that creates the class."

**F7 (blocking, CONFIRMED). "a new kind arrives boxed behind `Body::Instance` unless a measurement is
taken and recorded that says widening is worth it for that kind" (Q4).**

Reading A: the default is `Body::Instance`, and widening needs a measurement, so a subagent that wants
a `Body::ClassObj` variant must first run a benchmark.
Reading B: `Body::Instance` is `Vec<(String, ObjRef)>` (read this session at `body.rs:113`), which is
an association list keyed by `String`; a class object's state is not instance variables, so
`Body::Instance` is not a candidate for it at all and the rule does not bind.

The spec's own U1 requires class objects to "live in the arena like everything else". Under reading A a
subagent stores a class in a `Vec<(String, ObjRef)>` and pays a linear scan on every method-dictionary
access. Under reading B it adds a variant and trips the 80-byte assertion on the first commit. The
spec does not say which, and it does not say what a "measurement" is here: which axis, how many rounds,
which binary. Given that `dispatch`, `alloc` and `heapshape` have no Rust number at all until this
phase makes them runnable, the measurement gating the first widening cannot be taken against the axis
that would justify it.

Fix: say what a class object's `Body` is, by name, in the spec.

**F8 (blocking, CONFIRMED). "Run it at each task that touches an execution path" (the bar).**

Reading A: every task in the phase, since dispatch is an execution path and this phase is dispatch.
Reading B: only tasks that change `run.rs`, `eval.rs` or `ir.rs`, excluding tasks confined to
`rexx-classes`.

Criterion 4 restates it as "a two-build sitting per task that touched an execution path", so the gate
inherits the ambiguity. A subagent cannot decide which it is, and the cost is not small: the recorded
command in `perf-baseline.md` (lines 1012-1016, read this session) runs six axes at `--rounds 5`,
two builds, two sizes, two instruments.

Fix: name the predicate mechanically. "A task whose diff touches `rexx-exec/src/` runs the sitting;
a task confined to `rexx-classes/src/`, `rexx-lib/src/`, `tests/` or `corpus/` does not."

**F9 (blocking, CONFIRMED). "It contains at minimum: ..." (criterion 2).**

"At minimum" makes criterion 2 a floor with no ceiling, which is fine for a gate and useless for a
task brief. A subagent asked to write `phase-5.txt` cannot tell whether it has finished. Worse, the
floor itself is not decidable: "one program per mixin in `CoreClasses.orx`" requires enumerating the
mixins, and the spec's own enumeration in "The sentence the roadmap gets wrong" mixes mixins and
"Rexx-level utility classes" in one sentence, with `Comparator`'s subclasses named collectively as
"its six subclasses" rather than by name.

Fix: name the set, and state the closure rule ("`phase-5.txt` is complete when every class in the
registry and every `::CLASS` in `CoreClasses.orx` has a program naming it").

**F10 (non-blocking, CONFIRMED). "`::CLASS`, `::METHOD` and `::ATTRIBUTE` are this phase's, in that
order, because that is the order `CoreClasses.orx` needs them."**

Two sections earlier the same spec writes "its directives are `::METHOD`, `::CLASS` and `::ATTRIBUTE`,
and nothing else" -- a different order for the same three names. So "in that order" reads either as an
implementation order or as a restatement of an order the document has already given differently. The
justification is also not established: the file's first `::METHOD` block (CoreClasses.orx:138 onward,
read this session) is at file scope and attaches to primitive classes the native layer creates, which
argues that file-scope `::METHOD` is needed independently of `::CLASS`, not after it.

**F11 (non-blocking, CONFIRMED). "the four `directive_gap` over-refusals this phase retires".**

`::OPTIONS` is one of the four `directive_gap` Phase 5 arms (survey section 3, read this session), and
D31 removes it from this phase. See F16.

---

## 3. What a task brief must contain that the spec does not supply

A brief carries exact values verbatim. These are the ones the spec has no value for, so the plan
author invents them and the invention is unreviewed.

**F12 (blocking, CONFIRMED). The guard command has no program name, and the axis set it names does not
match the pinned baseline.**

The spec writes: "A two-build sitting, `--build pinned=bench-baselines/pinned/rexx-run-pre-phase-5
--build head=target/release/rexx-run`, interleaved". No binary is named. `rexx-bench-suite`'s argument
parsing (read this session, lines 188-221) accepts `--self-check`, `--engine` and `--pin` and has no
`--build`. The binary that takes `--build` is `rexx-arms` (read this session, lines 100-124).

Worse, the axis sets disagree. `perf-baseline.md:1012-1016` records the pinned rows as taken with
`--axis alloc4c --axis arith --axis compound --axis emptyloop --axis strings --axis varlookup`. D35's
"classic axes" are `arith`, `compound`, `strings`, `varlookup`, `alloc4c` and `rexxcps`. So:

* `emptyloop` has pinned rows and is **not** in the spec's guard set, though it is `Role::Loop` in
  `rexx-bench-suite`'s `AXES` (read this session) and has a wall-clock row in the pre-Phase-5 table.
* `rexxcps` is in the spec's guard set and has **no** pinned `rexx-arms` row to compare against.
  `rexx-arms` resolves a non-`/` axis through `rexx_bench::program_path` (line 132-136), and `AXES`'s
  doc asserts the axis literal against the `bench-programs/` directory listing, where `rexxcps` does
  not appear. PLAUSIBLE, since I did not run it: `--axis rexxcps` fails to open a file, and the only
  spelling that could work is the absolute path into the read-only oracle tree.

Fix: quote the exact command, program name included, with the axis list copied from
`perf-baseline.md:1012-1016`, and say explicitly whether `rexxcps` and `emptyloop` are in or out.

**F13 (blocking, CONFIRMED). "the floor" has no single value at the place the spec cites.**

The spec's value-representation section says "`bench-baselines/README.md` states the floor to read an
`across_builds` movement against". I read that README in full this session. It states two numbers and
no floor: Task 8's `+/-0.74%` bound on code placement, and Task 4c's 7.8% between two builds of the
same source differing by one comment. The sentence that actually states a rule is in
`perf-baseline.md` at line 1037: "A sub-1% movement across builds is not a result."

So a subagent told to read the movement "against the floor stated in `bench-baselines/README.md`"
picks 0.74%, 1% or 7.8%. Fix: write the number and its source into the brief.

**F14 (blocking, CONFIRMED). The `rexx-classes` interface table is three rows of prose, not signatures.**

The table gives "given a receiver's behaviour and an uppercased message name, which method body",
"given a class, a name and a method body, install it and cascade", and "given an environment symbol,
which object". A subagent on either side of that boundary must invent the types. The spec elsewhere
fixes enough to make the invention consequential and still does not write it down:

* `rexx-classes` holds method *ids*, so `resolve` returns something like `MethodId`, which today is
  `MethodId(pub u32)` in `rexx-core/src/behaviour.rs` (read this session) -- but D29 replaces
  `BehaviourTable`, and the spec does not say whether `MethodId` survives.
* "a receiver's behaviour" is `BehaviourId(u16)` today, and D29 says `BehaviourId` "stays a `u16` index
  for the primitive fast path and is derived from the class object". So `resolve` takes either a
  `BehaviourId` or an `ObjRef`, and which one it takes decides whether a user-defined class can be a
  receiver at all.
* `define` takes "a method body" whose id is `rexx-exec`'s. `CoreClasses.orx` line 3980 is
  `self~define("COPY", "return self")`, which defines a method **from a string at run time**. So
  `define`'s "method body" is sometimes a parse of a string produced during execution. The one-way
  dependency the spec is proud of does not obviously survive that, because parsing is `rexx-parse`'s
  and installing is `rexx-classes`'s and the string arrives inside a method running on `rexx-exec`.

Fix: write the three signatures, in Rust, in the spec, and say which crate owns the run-time parse.

**F15 (blocking, CONFIRMED). Message texts and exit codes for the refusals this phase creates are
unspecified.**

`run.rs`'s directive tests assert the four `directive_gap` strings verbatim (survey section 3, read
this session), and `loud.rs` requires the emitted message to end with the owner read out of
`owners.rs`'s tables. So retiring a refusal is a string-level edit in more than one place. The spec
names none of the strings it retires and none of the strings it creates. In particular, D32 requires
`REPLY` and `GUARD` outside a method to answer the oracle's translation-time error. The spec quotes the
oracle transcript for `reply` (Error 99.919, rc 157) but not for `guard` beyond "99.911 in the same
shape", and gives neither the exact `Error 99 running ... Translation error.` first line for `guard`
nor the sub-message text. A subagent has to re-measure, which the ground rules encourage but which
means the spec is not the source of the expected bytes.

Other exact values missing: the file names and `sha256` recording format for the build script (D26);
the name of the deferral table's file and whether it is a `.txt` under `docs/` or a Rust const (D25);
the `phase-5.txt` header's admits/excludes paragraphs, which every other subset file carries and which
`coverage.rs` polices; the test names for the `>M>`/`>N>` assertions (D36 says "in the shape the three
existing `run.rs` indent tests already use" and does not name the new ones).

---

## 4. Ordering

**F16 (blocking, CONFIRMED). The spec states no task order, and the two ordering statements it does
make contradict its own gate.**

The only ordering statements in the document are D30 (`::REQUIRES` lands last), D33 and D34 ("from the
first task", "from the first commit that makes `~` work"), and the Directives section's "in that
order". Everything else is unordered, and the open questions hand the hardest ordering question to the
plan explicitly ("Whether the build order above is load-bearing").

Trace the path the parent asked for, from nothing implemented to criterion 1 passing.

*What must exist before `::CLASS` can be executed at all.* `::CLASS foo subclass Object` needs a class
registry that can answer `Object`, which needs `.environment` populated, which D33 puts in the first
task. It also needs a metaclass, since `::CLASS ... METACLASS` is one of the refused forms. So U1
precedes U4. The spec does not say this; it is derivable, and every derivation is a place two plan
authors differ.

*What must exist before a method body can run.* From `CoreClasses.orx` lines 3400-4193, read this
session: `EXPOSE` (line 3423 and pervasively), `USE STRICT ARG` with an expression default
(`digits=(digits())`, line 3597), `SELF`, `FORWARD` in three distinct forms (`forward message("%")`
line 3438, `forward message "STRING"` line 3582, `forward class (super) continue` lines 3981 and 4008),
a super-qualified send (`self~activate:super`, line 3998), class-scope methods (`::METHOD ... class`,
line 3588), `::ATTRIBUTE` with `get`/`set`/`class` modifiers (lines 4018-4033), internal labels and
`PROCEDURE` **inside** a method body (`adjLeft: procedure` line 4168, `str: procedure` line 4174,
`syntax:` line 4047), and `SIGNAL ON SYNTAX` inside a method (line 4042).

**The spec never mentions `EXPOSE`, never mentions `FORWARD`, and never mentions `DO WITH` /
`LoopKind::With`.** All three are Phase 5 rows in `owners.rs`'s tables, which I read in full this
session: `InstructionKind::Expose`, `InstructionKind::Forward` and `LoopKind::With` are each
`Owner::Phase("Phase 5")`, and the first two appear in `EXPECTED_OUT_OF_SCOPE`. A subagent whose brief
is generated from this spec will not find them, and the gate will.

*What must exist before `.environment` can answer.* D33 says the directories come first, which is
right, but `.environment~objectname = "The Environment Directory"` (CoreClasses.orx:55) is an
**assignment whose target is a message send**. The survey records that shape as a sub-case refusal
inside an implemented construct: "`Loud::expression` reached from an assignment or `PARSE` target",
with the oracle answering `Object "Q" does not understand message "X="`. So criterion 1 needs setter
dispatch, which the spec's dispatch section does not mention. Its dispatch surface is `~`, `~~` and
`[]` by way of `ExprKind::Message`; `name=` setter sends are a separate path.

**F17 (blocking, CONFIRMED). Criterion 1 requires `DO OVER` on a collection, which hides inside an
`Owner::InScope` variant and so has no refusal and no owner row to find it.**

`CoreClasses.orx:63` is `do name over publicClasses` and `StreamClasses.orx:45` is
`do name over publicClasses`. `LoopKind::Over` is `Owner::InScope` in `owners.rs`, and
`phase-4-exclusions.txt`'s DEVIATION 1 says "DO OVER on a string or a number is fine -- each iterates
once, yielding itself", with `DO OVER` on a stem forbidden in the corpus. Iterating a collection
through `SUPPLIER` is a third case that exists in neither statement.

PLAUSIBLE, since I ran nothing: an implementation that keeps the once-yielding-itself behaviour runs
`CoreClasses.orx` line 63 exactly once with `name` bound to the collection, and criterion 1's "runs to
completion" can be satisfied while the environment is populated with one wrong entry. That is a
silent divergence inside a criterion whose whole defence in the spec is that criterion 2 backs it up.

`CoreClasses.orx:70-72` adds a second shape: `do name over "nl", "cr", ...` over a comma-separated list
of expressions with a continuation. Whether `rexx-parse` builds that today I did not check.

**F18 (blocking, CONFIRMED). Criterion 1 names a file the spec's own open question asks whether to
include, and that file is one line.**

Criterion 1: "`CoreClasses.orx`, `StreamClasses.orx` and `PlatformObjects.orx` all run to completion".
Open question: "**Whether `PlatformObjects.orx` is in scope.** ... it is small, and nothing has read it
yet." A mandatory exit criterion and an open question about whether the same thing is in scope cannot
both stand.

I read the file this session. `/home/moritz/dev/repos/ooRexx/interpreter/platform/unix/PlatformObjects.orx`
is exactly one line: `-- Nothing to do currently`. It has no `use arg`, no directive and no executable
clause. The open question is answerable by one read, and answering it removes a third of criterion 1.

**F19 (non-blocking, CONFIRMED). `StreamClasses.orx`'s own top level needs the same collection
iteration plus package methods, and the spec treats it as a downstream consequence.**

`StreamClasses.orx:39-49` (read this session) is `use arg rexxPackage`,
`.context~package~publicClasses`, `do name over publicClasses`, `.environment~put` and
`rexxPackage~addPublicClass`. So the second file's top level is the first file's top level again, and
its `::CLASS` directives use `MIXINCLASS Object` and `::method ... abstract` (lines 54-56). `ABSTRACT`
is a `::METHOD` modifier the spec never names.

---

## 5. Definition of done, per section

The spec has an exit gate for the phase and almost no completion statement per section. Where a
section has none, the plan will guess.

| section | has a "this section is complete" statement? |
|---|---|
| The three layers | No. It states a hazard and two mitigations. Neither says when the layer is done. |
| The native layer (`rexx-classes`) | Partial. Four owned things are listed; nothing says which classes, so completion is D25's discovered set, which is F1. |
| The sourced layer (`rexx-lib`) | Partial. "embeds them", "sha256 recorded", "never edited" are testable. Whether the bootstrap runs at every `rexx-run` start or only under a flag is unstated, and that decides the `startup` axis. |
| Dispatch | No. D24 and D28 say what shape it takes and what it does not do. Nothing says which messages must resolve. |
| The behaviour table | Yes, effectively: the dictionary is flattened at definition time and rebuilt by cascade, with a mixin diamond in `phase-5.txt` from the first commit that defines a class. This is the best-specified section in the document. |
| `.environment` / `.local` / `VALUE` | Partial. "Both routes close here, together" is a done condition for the `VALUE` half. Nothing says which names must resolve. |
| Directives | No. `::CLASS`/`::METHOD`/`::ATTRIBUTE` are named without their modifier surface (`CLASS`, `PRIVATE`, `ABSTRACT`, `GUARDED`/`UNGUARDED`, `GET`/`SET`), all of which appear in `CoreClasses.orx`. |
| Trace | Yes. Two named instruments, one assertion per nesting depth, `ir_dual` agreeing on raw stderr. |
| Value representation | Partial, and see F7. `ExprKind::List` has a crisp done condition; `Body` does not. |
| The bar | No. See F8 and F12. "Re-role them in `rexx-bench-suite`'s `AXES` table in the same commit that removes the refusal" is crisp and is the only crisp part. |
| The gate | Criteria exist. Criterion 2's "at minimum" (F9), criterion 4's "touched an execution path" (F8) and criterion 5's deferral table (no format given) are each undecidable as written. |

**F20 (blocking, PLAUSIBLE). No section states what happens to the tree between tasks.**

This phase turns `rexx-bench-suite` red by design (the survey quotes
`every_blocked_axis_still_fails_on_this_crate`'s own doc), and the spec's answer is "Re-role them ... in
the same commit that removes the refusal". Good. But the same moment breaks `owners.rs`'s counts,
`loud.rs`'s witnesses, `coverage.rs`'s subset, `bif-exempt.txt`'s Phase 5 rows and
`assertions.rs`'s `EXEMPT` set, all of which the survey records as asserting in both directions -- a row
that starts passing is as red as one that fails. The spec inherits that survey and does not carry the
consequence forward into a task. A subagent that lands message sends and runs the gated debug suite
will see failures it was not told to expect and cannot tell from regressions.

---

## 6. Contradictions inside the document

**F21 (blocking, CONFIRMED). Criterion 2 requires retiring `::OPTIONS`; D31 removes `::OPTIONS` from
the phase.**

Criterion 2: `phase-5.txt` "contains at minimum: ... the four `directive_gap` over-refusals this phase
retires". The four `directive_gap` Phase 5 arms are `Requires`, `Options`, `Class` naming another
class, and `Annotate` naming a target (survey section 3, read this session). D31: "`::OPTIONS` and the
`OPTIONS` instruction **leave this phase**". The two sentences cannot both be true.

**F22 (blocking, CONFIRMED). Criterion 2 requires retiring `::ANNOTATE`; the open questions say nobody
has measured what `::ANNOTATE` does.**

Open question: "**`::ANNOTATE`.** It is one of the four `directive_gap` over-refusals and the survey
classes it as an object-model refusal. Nothing here has measured what it does." An exit criterion
requiring the retirement of a refusal whose subject the spec says is unmeasured is a criterion no
subagent can be briefed for.

**F23 (blocking, CONFIRMED). D31 moves work out of the phase to an owner that cannot be spelled.**

`owners.rs`'s `SPLIT_TABLE_PHASES` is `&["4b", "4c", "Phase 5", "Phase 7"]`, and
`assert_owner_strings_are_split_table_phases` fails for any owner string outside that set, with the
message "an owner string outside that set is an unpoliced escape". `InstructionKind::Options` is
`Owner::Phase("Phase 5")` in both `INSTRUCTION_TAGS` and `EXPECTED_OUT_OF_SCOPE`. So D31's
"package-settings unit that can land before, during or after this phase" has no expressible name in
the one table that polices owner strings, and the message `directive_gap` emits is literally
`::OPTIONS is not implemented (Phase 5)`, asserted verbatim by `run.rs`.

Either D31 keeps the "Phase 5" label and only reorders work inside the phase, in which case it is not a
scope change and the spec's "leave this phase" is wrong; or it needs a plan amendment adding a phase
name, which the spec does not schedule.

**F24 (blocking, CONFIRMED). The spec's "two over-refusals" contradicts the survey it declares its
evidence half.**

Spec, Directives section and D31: taking `::OPTIONS` and `OPTIONS` out "deletes two over-refusals".
Survey section 3: "**`::OPTIONS` has no over-refusal** and the exclusions file says why". The spec's
header says "Where the two disagree, the survey is a dated reading of `11638b91e` and this is the
decision" -- but this is not a decision, it is a factual claim about the same measurement, and the two
documents state it oppositely. On the evidence I read, the spec's substance is defensible (the oracle
runs `::options digits 12` at rc 0 and this crate exits 120, per `phase-4-exclusions.txt`) and the
survey's wording is about which refusals `phase-4-exclusions.txt` *declares*. The plan author cannot
tell that from either document.

**F25 (non-blocking, CONFIRMED). The L2 argument is applied to one criterion and not to its neighbour.**

"A gate criterion that cannot be reached is worse than no criterion" demotes the L2 rung, on the
ground that ooTest needs `SysFileExists` and `.File`, which are Phase 7's. `.File` and the stream
classes come from `StreamClasses.orx`, which criterion 1 requires to run to completion. The spec never
asks whether criterion 1 is reachable without Phase 7's platform layer. From
`StreamClasses.orx:54-90`, read this session, the top of the file is mixins raising 93.963 rather than
touching the filesystem, so PLAUSIBLE: the file may install without Phase 7. But the spec does not make
that argument, and it is exactly the argument it made for L2.

**F26 (non-blocking, CONFIRMED). An open question the body already answered.**

"**What plays the oracle for a native method.** ... There is no third case in this phase, but the plan
should say so rather than leave it implied." The body already says it, in the native-layer section: "A
native method ... is a Rust function whose id resolves through the same table, which is exactly how
`CPPCode::resolveExportedMethod` works on the C++ side", and D25's four-answer oracle assertion is the
instrument. Leaving it as an open question invites a plan author to re-decide something decided.

**F27 (non-blocking, CONFIRMED). "The measurement that would justify a send cache does not exist" sits
beside a decision that requires a measurement to revisit.**

D28: "Revisit only against a `dispatch` axis number, which does not exist yet." D35: `dispatch` becomes
a new measurement during this phase. So the condition for revisiting D28 is satisfied inside the phase
that made the decision, and nothing says whether the revisit happens here or in a later phase. A
subagent taking the first `dispatch` reading has no instruction about what to do with it.

---

## 7. Decomposition assessment

The units are correctly *identified* and incorrectly *sized*. U1, U2, U6, U7, U8 and U11 are
task-shaped: each has a boundary, an artifact and something that goes red. U3, U9, U10 and U12 are
not.

**Two units that will collide, named:** any task landing `ExprKind::Message` and any task landing
`ExprKind::List` both edit `EXPECTED_OUT_OF_SCOPE`, `variant_counts_match_the_audited_split`'s
`EXPR_TAGS` counts, `loud.rs`'s `EXPR_WITNESSES` and `lib.rs`'s `expr_owner`, and the spec names none
of those files. They will also both want to add their witness to `phase-5.txt`, whose header has to
state what the subset admits and which the other three subset files show is rewritten by each task
that widens it.

**A second collision, less obvious:** U6 (`List` as a real Array) and U12 (the corpus subset). The
corpus already holds programs parked for this phase. `phase-4a.txt`'s header, read this session, says
`num/digits_rounding.rex`, `num/exponential.rex` and `num/operators.rex` "stay in corpus/num/ for
Phase 5, once List exists". I also read all three subset files and `lang/primitive_classes.rex` --
described in `corpus/README.md` as "`~id` of every class reachable as an environment symbol" -- is in
none of them, so it is a ready-made Phase 5 witness sitting unrun. The spec's criterion 2 enumerates
what `phase-5.txt` must contain and mentions none of the parked programs. Two subagents will
independently decide whether restoring them is their job.

**The interface that is pinned well:** the flattened dictionary and the cascade. D29 names the C++
functions, states the invalidation mechanism, states what `BehaviourId` becomes, and gives a witness
(a mixin diamond, from the first commit that defines a class). If the rest of the spec were written to
that standard this would be `Ready with fixes`.

---

## Summary of blocking issues, by severity

1. F1 -- the discovery method is an unbounded loop and the information to replace it is statically available.
2. F16 -- `EXPOSE`, `FORWARD` and `LoopKind::With` are Phase 5 rows the spec never mentions, and `CoreClasses.orx` needs the first two pervasively.
3. F21/F22/F23 -- criterion 2 requires retiring refusals that D31 removes from the phase and that the open questions say are unmeasured, and D31's new owner cannot be spelled in the table that polices owner strings.
4. F3 -- `CALL 'StreamClasses.orx'` is Phase 7's resolution mechanism by the exclusions file and D26's embedding by this spec, with no stated contract.
5. F4/F5 -- the ownership tables and the `read_subset` callers are shared mutable state across nearly every task, and the spec names neither.
6. F12/F13 -- the guard command has no program name, its axis set disagrees with the pinned baseline in both directions, and "the floor" has no value at the cited location.
7. F17 -- criterion 1 needs `DO OVER` on a collection, which is inside an `Owner::InScope` variant and so announces nothing.
8. F14/F15 -- no signatures, no message texts, no test names, no file names for the artifacts the phase creates.
9. F6/F7/F8/F9 -- four sentences with two readings each, all four load-bearing.
10. F18 -- criterion 1 and an open question disagree about whether `PlatformObjects.orx` is in scope; the file is one comment line.
11. F2/F20 -- the instrument changes character partway through the phase, and nothing tells a subagent which red tests to expect.
