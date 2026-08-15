# R2 -- the gate, the criteria, and every instrument the spec proposes

Reviewer R2. Lens: find a criterion that cannot fail, an instrument that cannot see its subject, and
a claim of verification that verifies nothing.

Every finding below is labelled **CONFIRMED** (I ran something that would have come out differently
had the finding been wrong) or **PLAUSIBLE** (I reasoned from code I read).

## What I ran

Commands whose output any finding rests on, all run this session:

* `sha256sum` on the three `.orx` files in both checkouts.
* `/bin/grep -aoE "^[[:space:]]*::[A-Za-z]+"` over `CoreClasses.orx`, `StreamClasses.orx`,
  `PlatformObjects.orx`; `/bin/grep -aiE "^::class"` over `CoreClasses.orx`.
* `/bin/grep -an "createInstance"` over `interpreter/memory/Setup.cpp`.
* `rust/target/release/rexx-run` on a copy of `CoreClasses.orx` in a fresh empty directory.
* `rust/target/release/rexx-run` on `bench-programs/{dispatch,alloc,heapshape}.rex`.
* `rexx-run` on `/home/moritz/dev/repos/ooRexx/samples/rexxcps.rex`, then `perf stat -e
  instructions:u` on the same, three times.
* The oracle, wrapped as the ground rules require, on two hand-written mixin-diamond programs in a
  fresh empty directory.
* A throwaway crate compiling an **unmodified copy** of `rexx-exec/tests/support/mod.rs` with three
  probe tests added, to exercise `normalize_stderr` on `>M>`/`>N>` lines.
* A throwaway crate with a path dependency on `rexx-core`, printing `size_of` for `Body`, `Object`
  and `Bytes`.
* `command -v hyperfine`.
* `awk` over `rust/bench-baselines/pre-phase-5-arms.tsv` for its distinct `axis` values.

Nothing in the repository was modified. Probes ran from fresh directories under the session
scratchpad.

## What I searched *for*, and what those terms cannot reach

`size_of::<`, `SUBSET_FILES`, `directive_gap`, `floor`, `hyperfine`, `rexxcps`, `PREFIX_COVERAGE`,
`>M>`, `>N>`, `createInstance`, `^::class`, `assert_program_has_only_routine_directives`, `AXES`,
`Role::Blocked`. All exhaustive searches used `/bin/grep -a`.

Two blind spots I could not close. First, an instrument for one of these criteria could exist under a
name none of those terms reaches, and I would not have found it. Second, `.superpowers/sdd/` is
git-ignored and the default `grep` wrapper skips it; I searched it explicitly only for
`directive_gap`.

Three things I could not verify and did not assume:

* Whether `rexx-arms` accepts `--axis /home/moritz/dev/repos/ooRexx/samples/rexxcps.rex`. Its module
  doc says "an `--axis` entry holding a `/` is a path", and I did not run it.
* `size_of::<Slot>()` directly -- `Slot` is private to `rexx-core::heap`. I measured
  `size_of::<(Object, u32)>()` instead, which is `Slot::Live`'s payload.
* I did not run the workspace test suite.

---

## Part 1 -- the numbered exit criteria

### Criterion 1 -- the three `.orx` files run to completion, on both engines

**1. What makes it RED.** Any refusal or raised condition while running them. It is red today:
copying `CoreClasses.orx` into a fresh directory and running
`rust/target/release/rexx-run ./CoreClasses.orx` gives **rc 120**, stderr
`rexx-exec: ::CLASS naming another class is not implemented (Phase 5)`, stdout empty. **CONFIRMED**,
and it reproduces the spec's own "Verified in this session" claim exactly.

**2. What it does not trip on.**

**G7 (MEDIUM, CONFIRMED). `PlatformObjects.orx` is a single comment line.**
`/home/moritz/dev/repos/ooRexx/interpreter/platform/unix/PlatformObjects.orx` is 27 bytes, one line,
`-- Nothing to do currently`. `/bin/grep -an "::"` over it finds nothing. So one of the three files
criterion 1 names cannot go red for any object-model defect whatever -- it is satisfied by an
interpreter that can skip a comment. The spec's open question "Whether `PlatformObjects.orx` is in
scope. ... it is small, and nothing has read it yet" is answered by one `cat`, and the answer changes
what criterion 1 is worth. The Windows file is two lines; a platform gate would still be about
nothing.

**G19 (MEDIUM, CONFIRMED). "Runs to completion" is satisfied while a directive is silently
ignored.** `StreamClasses.orx` carries `::constant` directives (two, by
`/bin/grep -aoE "^[[:space:]]*::[A-Za-z]+"`), and `rexx-exec/src/lib.rs`'s `directive_gap` returns
`None` for `DirectiveKind::Constant(_)` -- installed and ignored, not implemented. So
`StreamClasses.orx` can reach its end with `::CONSTANT` doing nothing at all and criterion 1 is
green. The same argument generalises: `directive_gap`'s `None` arms cover `Annotate`, `Attribute`,
`Class`, `Constant`, `Method`, `Resource` and `Routine`, and a build that installs every one and
implements none runs all three files to completion.

The spec anticipates half of this ("A bootstrap that runs to completion proves the file did not
raise") and leans on criterion 2. That is right as far as it goes, but the spec's own directive
section lists `::CLASS`, `::METHOD` and `::ATTRIBUTE` as this phase's and does not mention
`::CONSTANT` at all -- so criterion 1 requires a file whose directive set is not in the phase's
directive list.

**3. Does the instrument exist.** **No instrument is named.** Criterion 1 states an outcome and no
test, no file, no command. Nothing in the tree runs `CoreClasses.orx`; I had to copy it to a
scratch directory by hand to reproduce the spec's own figure. Criterion 1 is incomplete until the
spec says where that check lives and who writes it. Compare criterion 2, which at least names
`corpus.rs`'s mechanism.

### Criterion 2 -- the `phase-5.txt` corpus subset, "byte for byte" on three descriptors

**1. What makes it RED.** A program in the subset whose stdout, exit status, or *normalised* stderr
differs from the oracle's, under `corpus.rs`'s `corpus_differential` with `REXX_CORPUS_GATE=1`.

**2. What it does not trip on.**

**G1 (HIGH, CONFIRMED). "Byte for byte ... on stdout, stderr and exit status" is false for stderr,
and the criterion inherits exactly the hole criterion 3 exists to patch.**
`corpus.rs`'s `check_case` calls `support::oracle::descriptor_diffs`, which at
`tests/support/oracle.rs:302` compares
`super::normalize_stderr(&rust.stderr) != super::normalize_stderr(&cpp.stderr)` -- not raw bytes.
`normalize_stderr` collapses the space run after any of the nineteen `trace_prefix_table` markers to
one space, and `>M>` and `>N>` are both in that list (`tests/support/mod.rs`, `TRACE_PREFIXES`).

I confirmed this by running the code rather than reading it. I copied `tests/support/mod.rs`
unmodified into a throwaway crate and added three probes. All ten tests passed, including:

* `an_off_by_two_indent_on_an_m_line_normalises_away` -- `>M>   "CLASS" => "The Array class"` and
  `>M>     "CLASS" => "The Array class"` normalise equal.
* `an_off_by_two_indent_on_an_n_line_normalises_away` -- same for `>N>`.
* `a_content_change_on_an_m_line_is_still_seen` -- the control: changing the *value* on the same line
  still differs after normalisation, so the two above are not passing because normalisation is a
  no-op.

So the spec's claim about `tests/support` is **correct**, and the consequence the spec does not draw
is that criterion 2's own wording is wrong. A `phase-5.txt` program that traces a message send with a
two-column indent error passes criterion 2. The spec explains the hole in its Trace section and then
writes "byte for byte ... on stderr" into criterion 2 anyway.

**G13 (MEDIUM, PLAUSIBLE). Criterion 2's coverage set is chosen by the thing under test.**
"an instance of each class the native layer creates" -- and D25 makes the native layer "whatever
running `CoreClasses.orx` turns out to require, grown one refusal at a time". So a smaller native
layer produces a smaller criterion 2. Combined with criterion 5, an implementation that builds a
minimal native set and defers the rest with a reason satisfies criteria 1, 2 and 5 together, and
nothing in the gate says the native set has to be adequate for anything except `CoreClasses.orx`.
That may be the intent of D25, but the criterion reads as coverage of the object model and is
coverage of whatever got built.

The "one program per mixin in `CoreClasses.orx`" half is better -- the file is an external
enumeration -- but see G14.

**G14 (MEDIUM, CONFIRMED). "Mixin" names two different sets, and the spec's own prose uses the
wrong one.** `/bin/grep -aiE "^::class"` over `CoreClasses.orx` gives every `::CLASS` line. The
classes actually declared `MIXINCLASS` are `MessageNotification`, `AlarmNotification`, `Collection`,
`OrderedCollection`, `MapCollection`, `SetCollection`, `Comparable`, `Comparator`,
`DescendingComparator`, `CaselessComparator`, `CaselessDescendingComparator`, `ColumnComparator`,
`InvertingComparator`, `NumericComparator`, `CaselessColumnComparator`, `Orderable` and `Singleton`.

`SupplierMixin`, `ManyItemMixin`, `SetMixin` and `BagMixin` are **plain `::class` declarations with
no `MIXINCLASS` keyword** -- verified with two lines of context each (lines 172, 218, 411, 557). The
spec's own opening section lists all four among "its `::CLASS` directives are mixins". So two task
authors reading criterion 2 will build different subsets, and the spec's prose points at the larger,
wrong one.

**G8 (MEDIUM, CONFIRMED). Criterion 2 requires a witness for something D31 removes from the
phase.** Criterion 2 asks for "the four `directive_gap` over-refusals this phase retires". Enumerated
from `rexx-exec/src/lib.rs`'s `directive_gap` itself, the arms carrying `"Phase 5"` are
`DirectiveKind::Requires(_)`, `DirectiveKind::Options(_)`,
`DirectiveKind::Class(..)` with `subclass`/`metaclass`/non-empty `inherit`, and
`DirectiveKind::Annotate(..)` with a non-`Package` target. **D31 says `::OPTIONS` leaves this
phase.** So the phase retires three of the four, and criterion 2 as written either cannot be met or
silently re-imports `::OPTIONS`.

**3. Does the instrument exist.** Partly, and the missing part is not named.

**G3 (HIGH, CONFIRMED). `coverage.rs` will refuse every program criterion 2 requires.**
Adding `rust/corpus/phase-5.txt` immediately reddens four `SUBSET_FILES` pins --
`tests/corpus.rs:470`, `tests/coverage.rs:513`, `tests/ir_dual.rs:1181`,
`tests/collect_stress.rs:137` -- each asserted against a directory listing. That part is
self-enforcing and good.

But `tests/coverage.rs:151` defines `assert_program_has_only_routine_directives`, which **panics**
for any directive whose keyword is not `ROUTINE`, and `tests/coverage.rs:753` applies it to every
program in the union of `SUBSET_FILES`. Criterion 2 requires programs carrying `::CLASS`,
`::METHOD` and `::ATTRIBUTE` -- one per mixin, plus the directive over-refusals. Every one of them
panics `coverage.rs` the moment `phase-5.txt` names it.

The spec says the gate "extends the existing `corpus.rs` mechanism" and never mentions
`coverage.rs`, whose walker must be widened to descend into class and method bodies before criterion
2 can hold a single program. The criterion is incomplete until the spec says who does that.

### Criterion 3 -- `>M>`/`>N>` pinned by in-crate exact-stderr assertions, and `ir_dual` agrees on raw stderr

**1. What makes it RED.** The in-crate assertions go red if the emitted bytes differ from the
hand-written expectation. `ir_dual` goes red if the two engines produce different stderr for the same
program.

**2. What it does not trip on.**

**G2 (HIGH, CONFIRMED). The `ir_dual` half of criterion 3 cannot see the defect criterion 3 exists
for.** I read what `ir_dual` diffs. `compare` (`tests/ir_dual.rs:1393`) runs one `Case` twice --
`Engine::TreeWalker` then `Engine::Ir` -- and compares `tw.stderr != ir.stderr` raw, and
`render_both_engines`/`compare_inline_cases` do the same with `from_utf8_lossy`. So the spec's claim
that `ir_dual` "diffs raw stderr between the two engine arms and therefore sees indent" is **true as
stated**. What the spec then does with it is not.

`ir_dual` compares *our two engines against each other*. It has no oracle. Its own module doc says
so: "**The one thing it cannot see is work the two engines share.**" A trace line's indent is a
`usize` parameter threaded from the interpreter into `trace.rs`'s `push_prefixed_blanks`
(`push_indent(out, 3 + indent)`) -- one formatter, both arms. An off-by-two indent on `>M>` produced
*identically by both engines* leaves `ir_dual` green by construction. That is precisely the defect
the 4c gate handed forward and that criterion 3 exists to close.

`ir_dual` would catch a `>M>` line the IR arm re-emits differently from the tree-walker, which is a
real and different defect (its module doc records `Op::Const` and `Op::TraceRead` as the cases). It
is not evidence for indent correctness. Listing it as half of criterion 3 makes the criterion look
twice as strong as it is; the load-bearing half is the hand-written in-crate assertions alone.

**G12 (MEDIUM, PLAUSIBLE). Nothing requires the in-crate expected bytes to have come from the
oracle.** `>M>` and `>N>` have no emitter in `rexx-exec/src/` today (`/bin/grep -rn '">M>"' src/`
finds nothing), so their expected bytes will be authored, not measured. `ir_dual`'s case files carry
an explicit tripwire against exactly this -- `both_engines_agree_on_every_case_file` refuses to run
under `REWRITE`, on the ground that regenerating an oracle-measured expectation from the
implementation "would turn an oracle-measured expectation into a self-consistent one, in a diff that
looks like any other expectation update". Criterion 3's instrument has no such guard and no stated
provenance requirement.

Worse, there is no *other* instrument that could catch it. `tests/trace_oracle.rs` compares against
committed `.expected` files and runs both sides through `support::normalize_stderr` (lines 240-241),
so it cannot see indent either. **The tree has no oracle-compared instrument for trace indent at
all.** The three `run/tests.rs` unit tests the spec points at
(`one_two_and_three_enclosing_dos_indent_by_two_four_and_six:2693`,
`the_corrected_28x_indent_rule_matches_all_fourteen_probed_shapes:2437`,
`an_absorbed_whencases_escaping_false_branch_reports_end_at_its_own_residual_indent:967`) all exist,
so the *shape* is real; their strength is entirely the care taken when their bytes were written down.

**G11 (MEDIUM, CONFIRMED). Satisfying criterion 3 leaves `trace_oracle.rs` declaring `>M>` and
`>N>` out of scope, and nothing goes red.** `tests/trace_oracle.rs`'s `PREFIX_COVERAGE` carries
`(">M>", Coverage::Owned("Phase 5"))` at line 627 and `(">N>", ...)` at 631, with committed literals
`WITNESSED_PREFIX_COUNT = 16` and `OUT_OF_SCOPE_PREFIX_COUNT = 3`, and
`OWNER_PHASES = &["Phase 5", "Phase 7"]`. `the_trace_surfaces_coverage_is_sixteen_of_nineteen_...`
checks the prefix set against `support::TRACE_PREFIXES`, the `Witnessed` rows against
`CLAIMED_PREFIXES`, the two counts against their literals, and every owner against `OWNER_PHASES`.

If Phase 5 emits `>M>` and `>N>` and pins them only in `run/tests.rs`, every one of those checks
still passes: the rows stay `Owned("Phase 5")`, the counts still add up, and `"Phase 5"` is still a
permitted owner. The phase would exit with two prefixes it implemented still declared out of scope,
and the file whose own doc says "A phase that has finished cannot own a prefix -- whatever it owned
is witnessed by then" would say the opposite. Criterion 3 should require the rows to move and the
literals with them; as written it does not, and no test notices.

**G21 (LOW). "one per nesting depth" is unbounded.** How many depths? DEVIATION 0 talks about
"nesting depth <= 3"; criterion 3 does not say. A task author writing one assertion satisfies the
letter.

**3. Does the instrument exist.** The `run/tests.rs` shape exists. `ir_dual` exists. The
`trace_oracle.rs` bookkeeping G11 describes must be *changed*, and the criterion does not say so.

### Criterion 4 -- no classic axis regressed against the pinned baseline beyond the floor

**1. What makes it RED.** An `across_builds` movement on `arith`, `compound`, `strings`,
`varlookup`, `alloc4c` or `rexxcps` exceeding "the floor", in a `rexx-arms` two-build sitting.

**2 and 3 together, because the criterion's problems are all about applicability.**

**G4 (HIGH, CONFIRMED). "The floor" is not defined anywhere a task author could apply it, and the
spec sends them to the wrong file.** The spec says twice that `bench-baselines/README.md` "states the
floor". `/bin/grep -arn "floor"` over `rust/bench-baselines/README.md` returns **nothing** -- the
word does not appear in that file. What it does say is that an `across_builds` row's code-placement
component was "bounded at `+/-0.74%` on an axis the change it was measuring could not reach".

The rule actually lives in `perf-baseline.md:1033`, which repeats the same false attribution
("`bench-baselines/README.md` states the floor and where it came from") and then offers a task author
three different numbers in four lines: `±0.74%` (Task 8's bound), "A sub-1% movement across builds is
not a result", and "Task 4c read 7.8% between two builds of the same source differing by one
comment". Two people can apply criterion 4 to a 3% regression and disagree in good faith: above 1%,
so it is a result; below 7.8%, so it is inside the placement noise the same paragraph cites. The
criterion needs one number and a rule, and the spec needs to cite the file that has them.

**G4b (HIGH, CONFIRMED). The classic-axis list and the pinned baseline's axis set differ in both
directions.** `awk -F'\t' 'NR>1{print $3}' rust/bench-baselines/pre-phase-5-arms.tsv | sort -u` gives
`alloc4c`, `arith`, `compound`, `emptyloop`, `strings`, `varlookup`.

* `emptyloop` **is** pinned and is **not** in the spec's classic list. It is the axis where the
  pinned rows record the IR arm costing 13.4% more instructions than the tree-walker -- the one
  `perf-baseline.md` singles out as worth knowing before Phase 5 starts -- and criterion 4 does not
  guard it.
* `rexxcps` **is** in the spec's classic list and has **no row in the pinned baseline**, and is not
  an entry in `rexx-bench-suite`'s `AXES` or a file in `rust/bench-programs/` (which holds
  `alloc4c`, `alloc`, `arith`, `compound`, `dispatch`, `emptyloop`, `heapshape`, `startup`,
  `strings`, `varlookup`). It is `/home/moritz/dev/repos/ooRexx/samples/rexxcps.rex`, referenced by
  the suite as a path constant. So "regressed **against the pinned baseline**" is unanswerable for
  `rexxcps` from the pinned rows; only the live pinned-build arm of a two-build sitting can supply
  it, and the criterion does not say that.

**G18 (MEDIUM, CONFIRMED mechanism). `rexxcps` sizes its own workload from wall clock, so an
instruction-count instrument is measuring a moving target.** `rexxcps.rex:96-128` runs the timing
loop twice: `if total>1 | trial=2 then leave`, else `count=(1%total + 1) * count`. The workload is
therefore a function of how fast the binary under test is.

Today it does not bite, and I measured that rather than assuming it. `rexx-run` on `rexxcps.rex`
exits 0 and prints `Averaged: 100 x 100 iterations of 1000 clauses (over 2.1s)`; three
`perf stat -e instructions:u` runs gave 19,314,858,060 / 19,314,857,530 / 19,316,308,850 -- a spread
of 0.008%. 2.1s is over the one-second threshold, so no recalibration happens and a *slower* build
recalibrates no more than this one does.

The hazard is asymmetric and latent: if either build in a sitting crosses the one-second boundary
(a large speedup, or a faster machine), the two sides run different amounts of work and the
`across_builds` ratio is meaningless. Nothing in `rexx-arms` reads the `Averaged: N x M` line, so
that happens silently. Criterion 4 should either drop `rexxcps` or require the iteration counts to
be asserted equal across the two builds.

**G22 (LOW). "per task that touched an execution path" has no decision procedure.** Phase 5 touches
the executor in nearly every task. Either the criterion means "every task", in which case say so, or
it means something narrower that nobody can apply consistently.

### Criterion 5 -- every class `Setup.cpp` creates is in the native layer or in the deferral table with a reason

**1. What makes it RED.** A name in `Setup.cpp`'s `createInstance()` sequence present in neither.

I checked the spec's quoted list against the tool's own definition rather than against the spec.
`/bin/grep -an "createInstance" interpreter/memory/Setup.cpp` gives, in file order, `RexxClass`,
`RexxInteger`, `RexxString`, `RexxObject`, `PointerClass`, `BufferClass`, `ArrayClass`,
`TableClass`, `IdentityTable`, `RelationClass`, `StringTable`, `DirectoryClass`, `SetClass`,
`BagClass`, `ListClass`, `QueueClass`, `NumberString`, `MethodClass`, `RoutineClass`,
`PackageClass`, `RexxContext`, `StemClass`, `SupplierClass`, `MessageClass`, `MutableBuffer`,
`WeakReference`, `StackFrameClass`, `RexxInfo`, `VariableReference`, `EventSemaphoreClass`,
`MutexSemaphoreClass`. **The spec's list and order match this exactly.** That is a point in the
spec's favour and I record it as such.

**2. What it does not trip on.**

**G5b (MEDIUM, PLAUSIBLE). A deferral row with a vacuous reason.** "with a reason" is not a
checkable predicate. An empty native layer plus thirty-one rows reading "not required by
`CoreClasses.orx`" satisfies criterion 5 outright. Criteria 1 and 2 are the backstop, but criterion 5
itself is satisfiable by writing rows, which is the degenerate implementation the project's own rule
asks about.

**3. Does the instrument exist -- and this is the important one.**

**G6 (HIGH, PLAUSIBLE). The spec does not say where the third list comes from, and the obvious
answer makes the test satisfiable by one edit.** The spec says "`rexx-classes` carries that table as
a test over its own registry". A test comparing `registry ∪ deferral` needs something to compare
*to*. If the `Setup.cpp` list is a literal in the same crate -- which is what "carries that table"
most naturally reads as, and is what the spec's own prose copy models -- then the check is two
in-repo lists and a class dropped from both is one edit away.

This repository has a considered position on exactly that shape and the spec ignores it.
`corpus.rs`'s `the_differential_reads_every_phase_subset_file` compares its literal against a
directory listing, "so the assertion below cannot be satisfied by a copy of `SUBSET_FILES` edited in
the same change". `ir_dual.rs`'s `the_sweep_runs_every_ootest_suite_a_sibling_harness_runs` goes
further: "Pinning the list against a second list in the same file does not rule that out, because
both are one edit away. The sibling harness sources are not."
`rexx-bench-suite`'s `verify_axis_list` does the same against `bench-programs/`.

And the right mechanism already exists in this tree: `crates/rexx-inventory/build.rs` derives Rust
tables from the C++ source at build time (`../../../interpreter/messages/rexxmsg.xml`,
`../../../interpreter/expression/BuiltinFunctions.cpp`) with `cargo::rerun-if-changed` on both, and
its own doc says "The C++ tree is the source of truth. Nothing here is hand-maintained." Criterion 5
should require the `Setup.cpp` list to be derived the same way. As written it is incomplete, and the
default reading gives a test that cannot see its own subject.

### Criterion 6 -- cold start with hyperfine

**G10 (MEDIUM). It is a task, not a criterion, and the spec says so.** "**This is a measurement, not
a pass/fail**" is a complete admission that it cannot go red. Nothing about the delivered code makes
it fail. It belongs in the task list with an obligation to record the number and to close D2 on it;
listing it among exit criteria makes the gate look like it has six teeth when it has five. My answer
to the question posed: **task**.

**G9 (MEDIUM, CONFIRMED). Its named instrument is not present and the roadmap already replaced
it.** `command -v hyperfine` finds nothing on this machine. `perf-baseline.md:97` states hyperfine
"is not installed in this environment and cannot be installed (no network)" and
`crates/rexx-bench/src/bin/rexx-time.rs` "is the substitute"; `2026-07-27-rust-rewrite.md:1361` says
"hyperfine is an external binary and is not available on every platform this gate must run on, so the
workspace carries its own". The spec's "which is what D2 asks for" is true of D2's own sentence at
`:151` and stale against everything the project did afterwards.

There is a second-order trap the criterion walks into. `perf-baseline.md:1063` says "D2's C++ figure
is 5.1 ms from hyperfine" -- but lines 97-111 of the same file show that number came from
`rexx-time --warmup 10 --runs 50` (median 5.119 ms), because hyperfine was unavailable. A task
author told to "measure with hyperfine and compare against D2's figure" is being pointed at a tool
that is not here and a number that is mis-attributed. Criterion 6 should name `rexx-time`, the exact
invocation, and the 5.119 ms it is compared against.

---

## Part 2 -- the other instruments

### The `~class` / `~superClass` / `~isA` / `~metaClass` oracle assertion (D25)

**It fires.** These answers reach stdout, and `descriptor_diffs` compares stdout byte-exact with no
normalisation, so a wrong metaclass link shows up. That is a real instrument and the cheapest one in
the spec.

**G25 (MEDIUM, PLAUSIBLE). It does not shadow the wiring the spec itself calls most likely to be got
wrong.** The spec says a wrong link "surfaces much later as `~class` answering the wrong object, **or
a method resolving through the wrong dictionary**", and then offers these four answers as "the
observable shadow of the wiring". All four are class-*graph* queries: which class, which superclass,
which metaclass, is-a. **None of them reads the flattened method dictionary.** The second failure
mode in the spec's own sentence has no instrument in this response. A fifth probe would cover it --
for each class, the receiver-side answer of a method the class inherits rather than defines -- and
the spec should say so.

### The deferral table as a test over its own registry

Covered as **G6** above. HIGH, PLAUSIBLE.

### The in-crate exact-stderr assertions for `>M>` and `>N>`

Covered as **G12** (no provenance tripwire) and **G11** (leaves `trace_oracle.rs` stale). Both
MEDIUM.

### The claim that `ir_dual` "sees indent"

Covered as **G2**. HIGH, CONFIRMED. The claim is literally true and does not support the use the
criterion makes of it.

### The `size_of::<Body>()` assertion as a guard against quiet widening

**The half that fires is genuinely strong, and I confirmed it is tight rather than assuming it.**
`body.rs:134` asserts `size_of::<Body>() <= 80`. A throwaway crate with a path dependency on
`rexx-core` printed:

```
Body   = 80
Object = 88
Bytes  = 56
Object+u32 (Slot::Live payload) = 96
```

So `Body` sits **exactly** on its bound, `Bytes` exactly on `bytes.rs:63`'s `<= 56`, and
`Slot::Live`'s payload exactly on `heap.rs:31`'s `<= 96`. `Object` is
`{ behaviour: BehaviourId, body: Body, has_uninit: bool }` -- it already carries a behaviour handle,
so there is no slack to absorb a new per-object field either. **One added byte anywhere in the object
graph trips one of the three assertions**, and being `const _: () = assert!(...)` they are
re-evaluated at every compile and cannot go stale. The spec's characterisation is right.

Is there a widening that leaves it green? Only one that is not a widening: a new variant whose
payload fits inside 80 bytes (`Body::Class(Box<...>)` at 8 bytes, say) costs the arena nothing and
correctly does not trip. That is the intended behaviour, not a hole.

**G16 (MEDIUM, PLAUSIBLE). The other half of the risk row has no instrument at all.** The response
column reads "the `size_of` assertion trips; **a widening needs a recorded measurement**". The first
clause is an instrument. The second is a sentence. Raising `80` to `88` is one character, in the same
commit as the widening, and nothing anywhere requires a measurement to exist, to be cited, or to
compare against the pinned baseline. This is the "fix stated beside the criterion" shape: the
assertion's whole job is to force a decision, and the decision has no record and no reviewer hook.
The spec should name where such a measurement lands -- `bench-baselines/*.tsv` and a decision entry
-- and criterion 4's sitting should be required for it.

Minor: the risk row says "the `size_of` assertion", singular. There are three, and which one catches
a given widening depends on where the field is added. Naming one invites a task author to check that
one.

### The "mixin diamond in `phase-5.txt` from the first commit" risk response

**G5 (HIGH, CONFIRMED). The stated witness cannot fail, and I measured it against the oracle.**
The risk is "the flattened dictionary is built as a chain walk", consequence "multiple inheritance
resolves differently from the oracle". The response is "a mixin diamond in `phase-5.txt`".

A diamond alone does not distinguish the two algorithms. What distinguishes them is *the same method
name reachable by two branches*. Two programs, run through the wrapped oracle from a fresh empty
directory:

```rexx
d = .D~new                     d = .D~new
say d~m1 d~m2                  say d~who
::class M1 mixinclass Object   ::class M1 mixinclass Object
::method m1                    ::method who
  return 'm1'                    return 'from M1'
::class M2 mixinclass Object   ::class M2 mixinclass Object
::method m2                    ::method who
  return 'm2'                    return 'from M2'
::class D inherit M1 M2        ::class D inherit M1 M2
```

The left one is a diamond -- `D` reaches `Object` by three paths -- and the oracle prints `m1 m2` at
rc 0. **A linearised chain walk over `[D, M1, M2, Object]` prints the same thing.** The right one
prints `from M1` at rc 0, and that answer is `createInstanceBehaviour`'s merge order and nothing
else. Only the second is a witness.

So the risk response as written is satisfiable by a program that cannot fail. It needs to say
"a method name defined on two branches of a diamond, with the oracle's answer for which one wins".

Two further problems with the same row:

* **It is blocked by G3.** Such a program cannot enter `phase-5.txt` until `coverage.rs`'s directive
  walker is widened, because it carries `::CLASS` and `::METHOD`.
* **"from the first commit that defines a class" is not reachable.** The program needs
  `::CLASS ... INHERIT` and `MIXINCLASS`, which is the `::CLASS naming another class` refusal --
  the phase's *first* task by the spec's own open question. So the witness cannot predate the
  mechanism it tests. The spec's `DateTime` example (`::CLASS 'DateTime' public inherit Comparable
  Orderable`, which I confirmed in the file, with both mixins rooted at `Object`) is a real diamond
  but lives inside `CoreClasses.orx`, which cannot run until the phase is nearly done.

### The per-task two-build performance sitting

Covered as **G4**, **G4b**, **G18**, **G22**. The mechanism itself exists and works:
`rust/target/release/rexx-arms` and `rexx-bench-suite` are both built, and
`rust/bench-baselines/pinned/rexx-run-pre-phase-5` is staged (14,191,256 bytes). The criterion built
on top of it is what is underspecified.

**One instrument here is genuinely strong and the spec undersells it.** D35 says `dispatch`, `alloc`
and `heapshape` must be re-roled "in the same commit that removes the refusal, so the suite's
blocked-axis assertion stays true rather than red". That assertion is real and it fires:
`rexx-bench-suite.rs`'s test at line ~1190 filters `AXES` for `Role::Blocked`, asserts the set is
non-empty, and then *runs each blocked program* through the capped wrapper and asserts
`!completed.succeeded()`. I confirmed all three still refuse today -- `rexx-run` on
`bench-programs/{dispatch,alloc,heapshape}.rex` gives rc 120 with
`rexx-exec: a message send is not implemented (Phase 5)` for each. So the moment any of them starts
working, that test goes red and forces the re-role. This is the one place in the spec where the
stated response is a check that would have done something different had the claim been false.

### The build-script sha256 check on the `.orx` files

**G15 (MEDIUM, CONFIRMED facts).** Three separate problems.

**The files are already in this repository, byte-identical, under version control.** I compared both
checkouts:

| file | sha256 (both trees) |
|---|---|
| `interpreter/RexxClasses/CoreClasses.orx` | `c00727ab66e0fd862f12a7c023656c948259837b8432fa0d149bd6938fe2ff47` |
| `interpreter/RexxClasses/StreamClasses.orx` | `b61440a82bd3b2c198a84f44f8f9f0057aed847750c7b9e2c7fc05a819b92bba` |
| `interpreter/platform/unix/PlatformObjects.orx` | `04f89bb9b1ff2f5ee06f774f6b1b4159699198cc7da48c8703a9a0ab7fe8435e` |

D26's stated purpose is "so that a drift is a build-time failure rather than a silent divergence".
Between the in-repo copy and the oracle checkout's copy there is nothing to drift: they are two
checkouts of the same upstream file, and `git status` already detects a local edit to the in-repo
one. The check as motivated is guarding a divergence version control already guards.

**The spec does not say which tree the build script reads, and the answer matters.**
`rexx-inventory/build.rs` reads the **in-repo** tree by relative path with `cargo::rerun-if-changed`
on each input -- portable, and it is why `rexx-inventory` builds anywhere the repository is checked
out. If `rexx-lib`'s build script instead reads `/home/moritz/dev/repos/ooRexx` absolutely, as "the
read-only oracle tree" implies, then a **shipping crate** becomes unbuildable on any machine without
that second checkout. Today the oracle path is hardcoded only in `tests/support/oracle.rs` -- in
tests. Moving it into a build script is a step change in what the workspace requires to compile, and
the spec should decide it explicitly rather than leave it to the task.

**The sha is falsifiable by the act of committing.** An edit to a `.orx` plus a regenerated sha in
the same commit passes. D26's "**They are never edited**" therefore has no instrument at all -- it is
enforced by the same sentence that states it. Reading the files from the in-repo tracked copies and
letting `git` be the pin is both cheaper and stronger.

---

## Part 3 -- the risk table, row by row

| row | is the response an instrument that fires? |
|---|---|
| `CoreClasses.orx` opens semantic gaps in bulk | **No.** "expected, and the roadmap says so. Triage per gap" is an acknowledgement, correctly placed in the risk column and wrongly placed in the response column. Nothing fires. That is defensible for a schedule risk, but it should not sit under a heading that reads as mitigation. (G24, LOW) |
| the discovered native set is wired wrong | **Partly.** The four-answer assertion fires on the class graph and is cheap and real. It does not observe the flattened dictionary, which is the second half of the same row's own stated consequence. (G25, MEDIUM) |
| `Body` widens quietly | **Half.** The `size_of` assertion is tight and re-evaluated at every compile -- measured `Body = 80`, `Object = 88`, `Slot::Live payload = 96`, all exactly on their bounds. "a widening needs a recorded measurement" has no instrument. (G16, MEDIUM) |
| the flattened dictionary is built as a chain walk | **No, as written.** A diamond with no method-name collision is satisfiable and undetectable -- measured on the oracle: `m1 m2` from the diamond, `from M1` only when both branches define the same name. The witness also cannot exist before `::CLASS ... INHERIT` works, and cannot enter the corpus until `coverage.rs` is widened. (G5, HIGH) |
| the crate boundary is in the wrong place | **The direction is free; the width is unguarded.** Cargo rejects a dependency cycle, so "`rexx-classes` depends on nothing in `rexx-exec`" is enforced by the build for free and needs no response. What the row is actually about -- "two operations and one table" -- has none: nothing stops `rexx-classes` growing a tenth `pub fn`, and "reaching past it is a spec amendment" is a sentence. A `pub` surface assertion, or the interface as a trait `rexx-exec` holds by, would fire. (G20, LOW) |
| a Phase 5 refactor costs 30% on a classic axis | **Inherits criterion 4 entire.** With "the floor" undefined, `rexxcps` unpinned and self-calibrating, `emptyloop` unguarded, and "touched an execution path" undecidable, a 30% regression would be caught -- 30% is above any candidate floor -- but a 5% one would be argued about. (G4, G4b, G18, HIGH/MEDIUM) |

---

## Part 4 -- one omission the gate has, that the roadmap requires

**G17 (MEDIUM, CONFIRMED).** `2026-07-27-rust-rewrite.md:32` reads: "**Every phase exit reports the
unsafe-block count** (...) **and the list of crate roots carrying `deny` rather than `forbid`**.
Either growing without a corresponding decision block in Section 1 fails the gate." Phase 5 creates
two new crates, `rexx-classes` and `rexx-lib`, each of which will carry one of those two attributes
at its root. The spec's gate has no such criterion. This is a standing phase-exit obligation the spec
silently drops.

---

## Part 5 -- one thing addressed to the human partner, challenging a settled call

The gate being a `phase-5.txt` corpus subset is settled and I am not asking to overturn it. But the
decision imports a property the spec then spends a section working around, and the spec does not say
so in the criterion:

`corpus.rs`'s comparison **is not byte-for-byte on stderr**, by design and for good reasons
(DEVIATION 0). The spec identifies the resulting hole precisely in its Trace section -- "an off-by-two
indent on a new trace line is invisible to every corpus instrument in the tree" -- and then writes
"compared byte for byte against the oracle on stdout, stderr and exit status" into criterion 2. Both
sentences are in the same document.

Two ways out, both cheap:

1. Correct criterion 2's wording to say what the instrument does, and let criterion 3 carry the
   indent question explicitly. This is the honest minimum.
2. Better: give `phase-5.txt` an **un-normalised** stderr arm. DEVIATION 0 exists because the
   oracle's own indent counter is restored inconsistently on two loop-exit paths
   (`BaseDoInstruction.cpp:161` vs `:377`), which is a property of *completed loops*. A `phase-5.txt`
   program that traces a message send at fixed nesting depth with no completed loop is outside that
   carve-out entirely and could be compared raw. That would give the phase the first
   oracle-compared indent instrument the tree has ever had, which is worth more than the in-crate
   assertions criterion 3 currently rests on.

---

## Summary

| id | severity | label | one line |
|---|---|---|---|
| G1 | HIGH | CONFIRMED | criterion 2's "byte for byte ... stderr" is false; `descriptor_diffs` normalises, and `>M>`/`>N>` indent normalises away |
| G2 | HIGH | CONFIRMED | criterion 3's `ir_dual` half compares engine to engine, so a shared indent defect leaves it green |
| G3 | HIGH | CONFIRMED | `coverage.rs:151`/`:753` panics on any subset program with `::CLASS`/`::METHOD`; criterion 2 requires them and the spec never mentions it |
| G4 | HIGH | CONFIRMED | "the floor" is not in the file the spec cites, and the file that has it offers three different numbers |
| G4b | HIGH | CONFIRMED | `emptyloop` is pinned and unguarded; `rexxcps` is guarded and unpinned |
| G5 | HIGH | CONFIRMED | the mixin-diamond risk response is satisfiable by a witness that cannot fail (measured on the oracle) |
| G6 | HIGH | PLAUSIBLE | the deferral-table test has no stated external third list; the default reading is two in-repo lists |
| G7 | MEDIUM | CONFIRMED | `PlatformObjects.orx` is 27 bytes, one comment line |
| G8 | MEDIUM | CONFIRMED | criterion 2's "four `directive_gap` over-refusals this phase retires" contradicts D31 |
| G9 | MEDIUM | CONFIRMED | criterion 6 names hyperfine; it is absent and the roadmap replaced it with `rexx-time` |
| G10 | MEDIUM | -- | criterion 6 is a task, not a criterion, and admits it |
| G11 | MEDIUM | CONFIRMED | satisfying criterion 3 leaves `>M>`/`>N>` declared `Owned("Phase 5")` with nothing red |
| G12 | MEDIUM | PLAUSIBLE | criterion 3's expected bytes have no oracle-provenance tripwire, unlike `ir_dual`'s |
| G13 | MEDIUM | PLAUSIBLE | criterion 2's coverage set is chosen by the implementation under test |
| G14 | MEDIUM | CONFIRMED | "mixin" is ambiguous; four `*Mixin` classes are plain `::class`, and the spec calls them mixins |
| G15 | MEDIUM | CONFIRMED | the `.orx` files are already in-repo byte-identical; the sha256 check's tree is unstated and its "never edited" rule has no instrument |
| G16 | MEDIUM | PLAUSIBLE | the `size_of` assertion is tight and real; "a widening needs a recorded measurement" has no instrument |
| G17 | MEDIUM | CONFIRMED | the roadmap's per-phase `unsafe`/`deny`-roots report is missing from the gate |
| G18 | MEDIUM | CONFIRMED | `rexxcps` self-calibrates its workload from wall clock; nothing asserts the two builds ran the same amount |
| G19 | MEDIUM | CONFIRMED | criterion 1 names no instrument, and `::CONSTANT` installs-and-is-ignored so "runs to completion" is weak |
| G20 | LOW | PLAUSIBLE | the crate-boundary response: direction is free from cargo, interface width is unguarded |
| G21 | LOW | -- | "one per nesting depth" is unbounded |
| G22 | LOW | -- | "per task that touched an execution path" has no decision procedure |
| G23 | LOW | -- | the `>N>`/`ClassResolver` correction is deferred to "whatever gate document repeats the old claim" -- no owner, no named file, against the project's own rule to correct the plan where the next reader will see it |
| G24 | LOW | -- | risk row 1's response is an acknowledgement in a mitigation column |
| G25 | MEDIUM | PLAUSIBLE | the four-answer assertion reads the class graph and never the flattened dictionary, which is the other half of its own row's consequence |

**Counts: 6 HIGH, 14 MEDIUM, 5 LOW. 15 CONFIRMED, 6 PLAUSIBLE, 4 unlabelled (they are readings of
the spec's own text, not claims about the tree).**

**What held up.** Three of the spec's claims survived adversarial checking and I record them
because a review that only finds faults is not a measurement: the `Setup.cpp` class list and its
order are quoted exactly right; `tests/support` really does hide an indent error and `ir_dual`
really does diff raw stderr, both as stated; and the `size_of` guard family is tight rather than
slack, with `Body`, `Bytes` and `Slot::Live` all sitting exactly on their asserted bounds. D35's
blocked-axis re-role obligation is backed by a live test that runs each blocked program and would
turn red the moment one succeeded -- the strongest instrument named anywhere in the spec.
