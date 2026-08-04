# Phase 4b exit gate — assessment

Phase 4b made `rexx-exec`'s activation stack real. It delivered `INTERPRET`, the
error report's per-activation echo stack, `CALL`/`RETURN` (the `Named` and
`Dynamic` arms), `PROCEDURE`, `PROCEDURE EXPOSE`, `USE ARG`/`USE STRICT ARG` and
the `>name` variable reference, `ExprKind::Call`'s internal-routine expression
form, `SIGNAL` to a label and `SIGNAL VALUE`, `SIGNAL ON`/`OFF`, `CALL ON`/`OFF`,
`RAISE`, `NOVALUE`, `SIGL`, the trap table, `PUSH`/`QUEUE`, three new trace
prefixes (`>A>`, `>F>`, `>R>`) and `TRACE L`, all measured against
`build/bin/rexx`.

---

## The criteria, written before anything was measured

This section was written and saved before a single gate was run for this
document, per the task's own Step 1: **a criterion written after the measurement
is a description of what happened.** Everything under "Assessment" below was
added afterwards.

Ten criteria. Seven carry forward from `phase-4a-gate.md` with the amendments D14
requires; three are new, and each of the three exists because 4b built something
4a's criterion set had no way to see.

**Every criterion below was checked against one question before it was written:
what degenerate implementation satisfies this, and would deleting its subject
leave it green?** Four criteria in 4a could not fail. Criterion 6's predecessor
was satisfied by `/bin/true`; criterion 4 had no subject until the mode it named
was built; and `mutate-4a.sh` itself once reported 9 of 9 mutations caught with
the oracle binary absent, because any non-zero exit counted as a catch. The
per-criterion notes below say, for each one, what its falsification is.

### 1. The named L0 subset — the **union** of `phase-4a.txt` and `phase-4b.txt` — runs with zero divergences, and the union satisfies the variant-coverage property

> Every program named by the union of `rust/corpus/phase-4a.txt` and
> `rust/corpus/phase-4b.txt` matches the oracle on stdout, on stderr and on exit
> status, under `REXX_CORPUS_GATE=1`. And the union constructs at least one
> instance of every in-scope `InstructionKind`, `ExprKind`, `LoopKind`,
> `PrefixOp`, `EndStyle` and `Trace` variant, and every `Operator`, where
> "in-scope" is `tests/owners.rs`'s own table and not a list in this document.

**The union, not `phase-4a.txt` alone.** D6 created `phase-4b.txt` beside it
rather than growing 4a's file, so a regression can still be attributed to a
sub-phase. `phase-4a.txt`'s own header excludes `INTERPRET`, `CALL`, `PROCEDURE`,
`SIGNAL`, `RAISE`, `PUSH` and `QUEUE` **by definition**, so every witness for
every construct 4b added had to land in the second file; a criterion reading only
the first would have measured 4b's delivery at zero and passed.

**What this criterion cannot see, stated in the criterion rather than discovered
after it.** The coverage property enumerates variants **individually and asserts
nothing whatever about their combinations.** A variant is witnessed the moment it
appears in any program, regardless of what else appears in the same clause. This
is not a hypothetical limit: it is exactly how 4a shipped `a. = 5; say a. + 1`
aborting the process at rc 101 through a 29-of-29 byte-identical corpus, nine
per-task reviews, seven gate criteria and a nine-mutation script. `Stem` had a
witness, arithmetic had witnesses, and `Stem` *as an arithmetic operand* had
none, because nothing in the coverage model ever asked for one. 4b's own Task 10
was dispatched to add combination witnesses and its review deleted two of the
three it wrote as redundant, which is a real result and is reported below rather
than smoothed over. **A reader must not take a pass here as evidence that
combinations were exercised.**

*Falsification:* deleting a program from either subset file fails its file's own
committed-list equality — `phase_4a_subset_matches_the_committed_list` for
`phase-4a.txt`, `phase_4b_subset_matches_the_committed_list` for
`phase-4b.txt`; mutating the interpreter fails the differential, exercised for
real under criterion 6.

**The second of those two pins did not exist when this criterion was first
written, and the falsification claimed here was false without it.** The original
wording said a deleted `phase-4b.txt` entry would "leave a variant unwitnessed".
Measured, one line at a time across all twelve: **nine of the twelve leave
`coverage.rs` green**, because the coverage property needs only *some* program to
construct each variant and by the end of 4b most variants have several
witnesses. Only `call_expression`, `use_arg_forms` and `push_queue` construct
something nothing else in the union does. The worst case was
`lang/condition_traps.rex` — **criterion 8's only named witness, and the declared
corpus catcher for three of criterion 6's mutations** — whose deletion left the
corpus reporting `41 of 41 matching` at exit 0, `coverage.rs` green and
`collect_stress` green: three criteria still reporting MET with one criterion's
entire subject removed, and the headline shrinking 42 → 41 while still reading
as a clean sweep. **A "N of N matching" harness cannot notice a missing program.**
The pin closes it, and is verified to fail by name.

### 2. The `base/expressions` assertion table, `tests/assertions.rs`, with D14's Phase 5 amendment

> Every row of the extracted `base/expressions` table is evaluated byte for byte
> and never numerically; every row that does not pass is on a committed exempt
> list naming what would unblock it; and the exempt set is policed in **both**
> directions, so a row that starts passing is as red as a row that starts
> failing.

**D14's amendment, and it is stated here up front rather than discovered.**
4a's wording contemplated a blocked row being unblocked by 4b or 4c and never
named Phase 5, which made the criterion as literally written unable to pass
within 4a at all. It is amended here: **all 35 rows of `tests/assertions.rs`'s
`EXEMPT` are `unblocked_by: "Phase 5"`.** The consequence, stated as a
prediction this gate is making rather than as an observation it will report:
**this exempt list cannot light up at this gate, and it cannot light up at 4c's
either.** Nothing 4b or 4c delivers moves a single one of those 35 rows. A
reader should expect this criterion to report the same number at 4c's gate that
it reports here, and that sameness is the correct result, not a stall.

**`tests/assertions.rs` is named explicitly, every time, because there are now
two exempt lists and they behave oppositely.** Task 11 added
`rust/corpus/keyword-exempt.txt`, whose 796 rows are **designed to fire at 4c's
gate** — see criterion 10. A criterion that said only "the exempt list" would be
ambiguous about which property it claims.

*Falsification:* removing a row from `EXEMPT` while it still fails, or leaving a
row on it after it starts passing, fails
`the_exempt_set_matches_the_current_blocked_rows` under
`REXX_ASSERTIONS_GATE=1`. A row compared numerically rather than byte for byte
fails `the_falsification_proof`.

### 3. Trace output byte for byte, plus D14's coverage measure — and what DEVIATION 0 removed from this criterion's reach

> Every committed trace witness's stderr matches the oracle byte for byte under
> the comparison DEVIATION 0 defines; every witness still emits every prefix the
> table claims for it; and the fraction of the oracle's nineteen prefixes that
> is witnessed is a **committed literal that an assertion reads**, with an
> owning phase named for every prefix that is not.

**This criterion no longer witnesses indentation, and saying so is half its
text.** DEVIATION 0 (4b Task 2b) normalises the run of spaces between a trace
line's prefix field and its content, on both sides, for stderr only, in
`tests/support/mod.rs`, shared by `corpus.rs` and `trace_oracle.rs`. A criterion
phrased as though it still checked indent would be claiming coverage the harness
**gave up on purpose**, and the deviation's own row records what that cost:
Task 4's C1 was a real, user-visible indent bug — a failure to restore
`Interp::current_value_indent` across a nested activation — and the corpus
reported 34 of 34 with it live.

So this criterion states its own reach exactly:

* **Normalised, and therefore *not* witnessed here:** leading indentation on
  stderr.
* **Unnormalised, and therefore still witnessed:** the three pinned shallow-depth
  (nesting ≤ 3, no completed loop) indent witnesses that live as `run.rs` unit
  tests, outside either harness's comparison function —
  `one_two_and_three_enclosing_dos_indent_by_two_four_and_six`,
  `the_corrected_28x_indent_rule_matches_all_fourteen_probed_shapes`, and
  `an_absorbed_whencases_escaping_false_branch_reports_end_at_its_own_residual_indent`.
  **Not** `the_indent_after_a_loop_has_already_exited_is_not_left_over_from_it`,
  which runs at top level where the oracle's counter is clamped at 0 and the
  correct and incorrect models agree.
* **Where this criterion's power now lives, and it is not a consolation prize:**
  value-line **content** and line **order** stay byte-exact, along with exit
  status, stdout, catalogue text, clause text and line numbers. `>>>`, `>V>`,
  `>O>`, `>L>`, `>F>` and `>A>` carry intermediate results and evaluation order
  that nothing else in the output exposes. Every defect Task 9 found and fixed —
  the re-tested pass's missing `>>>` pair, `EXIT`'s missing value line, `RAISE
  ... ARRAY`'s element order, the omitted array position closing up — was a
  **content or order** difference and was caught, not normalised away.

*Falsification:* fabricating a witness's `.expected` so it no longer contains a
prefix it is named for fails
`every_witness_still_emits_every_prefix_it_is_named_for` by name; dropping a
prefix from `PREFIX_COVERAGE` to flatter the fraction fails the equality against
`support::TRACE_PREFIXES`, read from the oracle's own `trace_prefix_table`;
changing the count without changing the table fails the literal.

### 4. Collect-on-every-allocation over the **union**, with an **activation-shaped** negative control

> The union subset passes again under `run_program_collect_every_alloc`,
> byte-identical to `run_program`; every program in it performs a non-zero number
> of collections, checked **per program** and not only in aggregate; and a
> negative control deleting a root that a **call** holds makes at least one
> program fail.

**Carried forward verbatim, this criterion could not fail for anything 4b
built, and that is why it is not carried forward verbatim.** 4a's version passed
having exercised **zero call frames** — its subset is 4a's 29 programs, none of
which contains a `CALL`. Two changes make it a 4b criterion:

1. It reads the **union**, so 4b's programs — calls, arguments, exposure,
   traps — are stressed too.
2. Its negative control must delete **a root a call holds**, specifically the
   argument list's root between an argument's evaluation and the callee's
   `USE ARG`. 4a's control deletes `eval_arithmetic`'s `push_temp(left_value)`;
   re-running that control here would re-test 4a and report a pass that means
   nothing about anything 4b added.

*Falsification:* a mode that never collects passes the byte-identity check by
construction, which is why the per-program non-zero-collections assertion exists;
the negative control is what proves the mode can observe a missing root at an
activation boundary.

### 5. Every variant executes or fails loudly, **naming an owner**, at **arm** granularity where a variant is split

> An enumerating test with no wildcard arm asserts that each
> `InstructionKind`/`ExprKind` variant is in 4b's named set or produces
> `NOT_IMPLEMENTED_EXIT` with a message naming the owning sub-phase; and where a
> variant is split across phases, the check is per **arm**, not per variant.

**The owner in the message is Task 0's, and until Task 0 landed the claim was
false.** The 4a plan asserted that loud messages name the owning sub-phase; they
never did — the emitted text was `format!("{name} is not implemented")` and named
no phase at all.

**Arm granularity is the amendment 4b forced.** `InstructionKind::Call` has four
`rexx_parse::Call` arms; Task 3 implemented `Named` and `Dynamic`, Task 7
implemented `Trap`, and only `Qualified` (`CALL ns:name`) is still owed — to
Phase 5. A variant-grained check reading the coarse `Call` tag alone would say
that no program in the 4b subset may contain a `CALL` at all, which is not what
the table means.

*Falsification:* removing the phase from a loud message, or letting an
unimplemented arm run rather than exit loudly, fails
`every_out_of_scope_variant_fails_loudly`; a new variant in any enumerated enum
is a compile error, not a silently shrinking check.

### 6. Mutation control: `rust/scripts/mutate-4b.sh`, 4b-shaped mutations, carrying 4a's guard

> A committed list of one-line mutations to the code 4b added, each of which
> **declares in advance what each of two instruments should say about it, and is
> measured against that declaration** — so an unexpected survival and an
> unexpected catch are both failures. The script carries (a) the exact-match,
> exactly-once application guard, (b) a baseline pass of the unmutated tree
> before the first mutation and after the last restore, and (c) a three-way
> `PASSED`/`DIVERGED`/`INFRA_FAILURE` classification that never folds an
> infrastructure failure into either "caught" or "not caught".

**AMENDED, and the amendment is the point rather than a loosening.** The obvious
wording — "each of which some instrument in this gate must catch" — is the one
this criterion started with, and **the script contradicts it on purpose**: row 12
is I17's genuinely equivalent mutant, declared to survive *both* instruments,
and the brief asked for it to be run rather than cited. Under the original
wording that row is a criterion violation; under this one it is a falsifiable
claim, because being *caught* now fails the script just as loudly as an
unexpected survival. A criterion that forced every mutation to be caught would
have made the only honest way to record an equivalent mutant — running it — into
a rule break, and the alternative (dropping it) leaves the equivalence claim
untestable. Criteria 2 and 4 carry their own amendments for the same reason;
this one is recorded beside them rather than left as a contradiction between a
document and the script it grades.

**The guard is reused; the mutations are not.** Re-running 4a's nine mutations
here would test 4a. Every mutation in `mutate-4b.sh` targets code that did not
exist before this sub-phase.

**Five mutations deliberately do not go through the corpus,** four of them for a
correct reason — the queue's storage and order (criterion 9), a dropped GC root
(criterion 4), and an omitted argument's `>A>` line, none of which a differential
run can observe. Classifying those against the corpus would report `PASSED` —
not caught — which is a true statement about the corpus and a misleading one
about the tree. They are classified against the crate's own unit suite instead,
which **asserts a non-zero test-run count per target and requires every target
to have reported**: `cargo test` exits 0 when it matches nothing (`0 passed;
0 failed; N filtered out`, status 0), so a harness reading only the exit status
cannot tell "passed" from "does not exist" — and summing the targets cannot tell
it either, since one busy target masks an empty one.

*Falsification:* pointing the oracle at a nonexistent path must abort at the
first baseline check, before any mutation runs, rather than reporting a number.
This is checked for real below, because it is the exact defect 4a's script
shipped.

### 7. Zero `unsafe`, clippy clean, `cargo fmt` clean

> `unsafe_code = "forbid"` stands; `cargo clippy --workspace --all-targets --
> -D warnings` is clean; `cargo fmt --all --check` from `rust/` is clean.

`cargo fmt --all --check`, **not** `cargo fmt --edition 2024 --check`: `cargo
fmt` has no `--edition` flag, that spelling exits 2 before doing any work, and a
task in this phase ran it and reported formatting clean.

**`NOT_IMPLEMENTED_EXIT`'s band, confirmed here because
`phase-4-exclusions.txt` asks this task for it by name** ("The specific integer
is Task 12's to confirm"). The value is **120** (`rexx-exec/src/lib.rs`), and
what the exclusions file actually argues for is the *band*: the code must sit
outside `157..=253`, where `256 - major` lives for majors 3 to 99, or a
not-implemented failure is indistinguishable from a raised condition. **120 is
outside it, and that is machine-asserted rather than confirmed by this
sentence** — `tests/spike.rs`'s
`the_loud_failure_code_cannot_be_confused_with_a_rexx_error` asserts
`!(157..=253).contains(&NOT_IMPLEMENTED_EXIT)` and pins the exact loud message
beside it. So the integer is recorded here once, as the exclusions file asked,
and the property it has to keep is guarded where it cannot go stale — which is
the same file's own reason for not writing the number into itself.

### 8. A condition trap is witnessed by **a value a handler set**, never by an exit code

> At least one differential corpus program traps a condition and proves it by a
> value the handler wrote into the variable pool — a value that is neither the
> flag variable's derived name nor its unset rendering — and the program is
> compared to the oracle byte for byte.

**This criterion exists because the obvious one cannot fail.** "The program
exited 0 after the trap fired" is satisfied by a program that never raised at
all. It is worse than that in Rexx specifically: an unset variable reads as its
own uppercased spelling, so a handler flag left unset renders as
plausible-looking data rather than as an obvious blank. A witness value must
therefore be chosen so that a failure prints something recognisably wrong.

*Falsification:* an implementation that never traps produces a fatal report and
no output at all; one that traps but leaves the handler's write out produces the
derived name; one that transfers at the raise rather than at the clause boundary
reorders the accumulated segments. All three are corpus divergences, and the
second and third are mutations under criterion 6.

### 9. `PUSH`/`QUEUE`'s interleaved order is pinned at unit level against oracle-measured values, **and this gate records that the construct ships undifferentiated**

> The queue's `PUSH`-at-head / `QUEUE`-at-tail order is asserted against values
> measured from the oracle; a separate test proves the instruction arms actually
> write into the running interpreter's queue; and this gate states plainly that
> no corpus program can observe the queue's contents.

**The recording is part of the criterion, not a footnote to it.** Nothing that
reads the queue back — `PULL`, `PARSE PULL`, `QUEUED()` — exists before 4c, so a
differential run can observe what was *written* and how it *traced*, and never
what was *stored*. A green gate would otherwise let a reader conclude the queue
is differentially verified. It is not, and 4b ships it that way on purpose.

*Falsification:* three mutations, and the split between them is the point.
**Two are rows of `mutate-4b.sh` and were run for this document**: collapsing
`push` into `queue` fails the order test, and deleting the two
`Queue::push`/`Queue::queue` call sites while keeping evaluation and tracing
leaves the **corpus green** and fails only the interpreter-level unit test. The
**third is cited, not re-run** — deleting the whole instruction arm fails the
corpus, because the expression stops being evaluated and traced — and its
provenance matters: it was measured at 4b's Task 8 against a **39**-program
subset (`39 of 39` dropping to `38 of 39`), not against the 42 this gate
reports. The mechanism is unchanged by the subset growing, but the numbers in
`phase-4-exclusions.txt`'s own row are Task 8's, not this gate's.

### 10. The `base/keyword` L1 table is policed in both directions — a **measurement this gate reports**, not a threshold it passes

> `rust/corpus/keyword-exempt.txt`'s committed set matches the current failures
> exactly, in both directions: a body that starts passing is as red as a body
> that starts failing. The pass **rate** is reported and is not a threshold.

**This is the second exempt list, and it is the opposite of criterion 2's.**
Of its 796 rows, **790 are attributed `4c` and are designed to fire at 4c's own
gate** — each body starting to pass and failing the set assertion until it is
removed. The other 6 are `defect:compound-do-control-variable` and fire whenever
that defect is fixed, which is unscheduled (see the ruling below). **The 790 are
the point, not the 6**: a criterion phrased around the defect rows would leave a
reader thinking the file is inert at 4c's gate when 790 of its rows are precisely
what fires there.

**No threshold.** A strict gate on a table nobody has looked at turns a
measurement into a blocker. What is gated is the set assertion; what is reported
is the number.

*Falsification:* the `is not on the committed` direction fires when a body starts
failing; the `now PASSES` direction fires when one starts passing. Both live in
`rust/crates/rexx-exec/tests/keyword_assertions.rs`.

---

## Assessment

**Ten criteria. Eight are met. One (criterion 2) is met carrying an inherited
criterion defect, the same one 4a recorded and the same one Phase 2's gate had.
One (criterion 3) is met weakly, and its weakness is now a measured number
rather than an admission.** One new gap was found by this gate's own mutation
script and is recorded in `phase-4-exclusions.txt` rather than left in this
document alone.

Every figure below was produced by running the command named, with **each exit
status read unpiped**. Nothing here is carried forward from another task's
report.

**The workspace total is 1,020 and not 1,019, and the difference is this gate's
own fix round.** The gate first measured 1,019 at `4c8c1f68`; the review then
showed criterion 1's falsification claim was false for `phase-4b.txt` (nine of
its twelve entries were pinned by nothing), and closing it added
`phase_4b_subset_matches_the_committed_list` to `coverage.rs`. **Every figure in
this table was re-run after that addition**, and only the workspace count moved.

### The tree, in one run

| command | exit | result |
|---|---|---|
| `cargo test --offline --workspace --no-fail-fast` | 0 | **1,020 passed, 0 failed, 4 ignored** |
| `cargo fmt --all --check` (from `rust/`) | 0 | clean |
| `cargo clippy --offline --workspace --all-targets -- -D warnings` | 0 | clean |
| `REXX_CORPUS_GATE=1 … --test corpus` | 0 | **42 of 42 matching** |
| `REXX_ASSERTIONS_GATE=1 … --test assertions` | 0 | **4,224 of 4,259 rows**, 35 RUNTIME-BLOCKED |
| `REXX_KEYWORD_GATE=1 … --test keyword_assertions` | 0 | **100 of 896 bodies, 713 of 1,773 calls** |
| `rust/scripts/mutate-4b.sh` | 0 | **12 of 12 mutations behaved exactly as declared** |

The 4 ignored are the three `rexx-num` format tests Phase 3's gate recorded
(they allocate 2-3 GB and say so in their own `#[ignore]` reasons) plus one
probe in `corpus.rs` that only runs as a child process of another test.

**Phase 2's and Phase 3's qualification still stands: no CI builds or tests the
Rust tree, so every figure in this document is a local claim.**

### 1. The union subset, zero divergences, full variant coverage — MET

**42 of 42 matching**, verified for this document. The union is `phase-4a.txt`'s
30 programs and `phase-4b.txt`'s 12, read by `read_subset` in first-seen order
with repeats dropped; `coverage.rs`, `corpus.rs` and `collect_stress.rs` all
read the same union, and `phase_4a_subset_matches_the_committed_list` still pins
4a's own 30 entries by exact equality so 4b could not have quietly shrunk them.

`every_in_scope_variant_is_witnessed_by_the_phase_subsets` passes over the
union. Every enumerated `match` still has no wildcard arm, so a new variant in
any of the seven enums is a compile error here rather than a silently shrinking
check.

**The combinations limit is not hypothetical, and 4b measured its own version of
it.** Task 10 was dispatched specifically to add combination witnesses — a raise
inside a routine inside a loop, an exposed stem mutated across calls, a trap that
resumes and raises again. Its own fix round deleted **two of the three**: every
mutation that made `raise_in_routine_in_loop.rex` or
`expose_stem_across_calls.rex` fail also failed an *existing single-construct*
program, so neither added coverage a simpler witness did not already have. Only
`call_on_trap_rearms.rex` survived, and it earns its place — it is the declared
catcher for mutation 7 below. The honest reading is that **combining constructs
is not automatically coverage**, and that 4b found this by mutation rather than
by assuming either way.

### 2. `tests/assertions.rs` — MET, with the inherited criterion defect

**4,224 of 4,259 rows passing, 35 RUNTIME-BLOCKED**, `REXX_ASSERTIONS_GATE=1`
exit 0, all five tests green.

**That is the identical figure the 4a gate reported, and the criterion predicted
it.** All 35 exempt rows are `unblocked_by: "Phase 5"`; nothing 4b delivered
could move one, and nothing 4c delivers will either. The sameness is the
prediction coming true, not a stalled measurement — and stating it in the
criterion *before* running it is what makes that distinguishable.

The inherited defect is unchanged and is restated rather than re-discovered:
the criterion's literal wording contemplates a row unblocked by 4b or 4c, so it
cannot pass STRICT within Phase 4 at all without `EXEMPT`, a committed
positionally-keyed escape. D14's amendment names Phase 5, which is what makes
the wording honest; it does not remove the defect, and the harness's own module
doc says so.

### 3. Trace output and its coverage measure — MET, weakly, with the weakness measured

`the_trace_surfaces_coverage_is_thirteen_of_nineteen_with_owners_for_the_rest`
passes: **13 of the oracle's 19 prefixes are witnessed at the end of 4b, up from
10 at the 4a gate.** The three new ones are Task 9's — `>A>`, `>F>`, `>R>`. The
other six each name an owner: `+++` and `>.>` (4c), `>I>` and `<I<` (4c,
alongside `::routine` dispatch), `>M>` and `>N>` (Phase 5).

The number is a committed literal that four assertions read, not a printed line:
the prefix set is checked equal to `support::TRACE_PREFIXES` (the nineteen read
from `RexxActivation.cpp`'s own `trace_prefix_table`, an enumeration **outside
this repository**), the `Witnessed` subset is checked equal to
`CLAIMED_PREFIXES`, which
`every_witness_still_emits_every_prefix_it_is_named_for` has already tied to
what the committed `.expected` files actually contain, and both counts must add
up to the whole table. So the fraction cannot be improved by dropping a prefix.

**Why "weakly".** 13 of 19 is a real measure and it is not 19 of 19; and
DEVIATION 0 means this criterion no longer witnesses indentation at all outside
the three pinned `run.rs` unit tests named in the criterion. What it does
witness — value-line content, line order, exit status, stdout, catalogue text,
clause text, line numbers — is where every trace defect 4b actually found lived,
which is the argument for the deviation and not a consolation for it.

### 4. Collect-on-every-allocation over the union — MET, and this time with call frames

`the_l0_subset_passes_again_under_collect_on_every_allocation` passes over the
**union**, byte-identical to `run_program`, with the per-program non-zero
collection assertion intact. 4a's version of this criterion ran 29 programs and
**zero call frames**; this one runs 42 including every `CALL`, `PROCEDURE
EXPOSE`, `USE ARG` and trap witness 4b added.

**The activation-shaped negative control fires**, and it is mutation 9 of the
script rather than a hand-run claim: deleting `run.rs`'s
`self.roots.push_temp(argument.value());` — the argument list's root, held
between an argument's evaluation and the callee's own `USE ARG` — leaves
`REXX_CORPUS_GATE=1` at **42 of 42** and fails the suite. That asymmetry is the
whole point: a dropped root is invisible to a differential run, so the corpus's
silence here is correct and the collector is the only instrument that can speak.

`a_clause_value_survives_the_handler_its_boundary_runs` (Task 7's fix round 4)
covers the other activation-shaped rooting window — a clause value created and
consumed *across* a `CALL ON` handler — in all three `Flow` shapes that carry a
value.

### 5. Every variant executes or fails loudly, naming an owner — MET

`loud.rs`'s three tests pass. `every_out_of_scope_variant_fails_loudly` runs a
witness per out-of-scope variant through `run_program` and asserts both
`NOT_IMPLEMENTED_EXIT` and that stderr **ends with** the exact suffix
`" is not implemented (OWNER)"` — `ends_with`, not `contains`, so a message
merely mentioning the owner's bytes somewhere does not satisfy it.

The witness list stands at **12 `InstructionKind` rows** — 16 including the four
`ExprKind` ones, and the 20 it is compared against below is likewise
instruction-only, so the trend is like for like. The rows are **arm**-grained,
not coarse: `loud.rs`'s own module doc says "one row per still-loud arm", which
for the one split variant left means `Call::Qualified` alone. Down from 20 at the
4a gate as 4b moved constructs in scope: `Interpret` (Task 1), `Return` and two
`Call` arms (Task 3), `Procedure`/`Use` (Task 5), `Signal`/`Raise` and
`Call::Trap` (Task 7), `Push`/`Queue` (Task 8). A witness for a variant that
moves in scope must be **deleted**, not left stale, or
`assert_witness_set_is_complete` fails the other way.

### 6. `mutate-4b.sh` — MET

**12 of 12 mutations behaved exactly as declared**, exit 0, with both baselines
green before the first mutation and after the last restore (**42 of 42** corpus,
**325 passed / 0 failed** suite, both times). The tree was verified clean
afterwards.

**The guard was attacked the way the branch review attacked 4a's, and it holds.**
`corpus.rs`'s `oracle_root()` was repointed at a nonexistent path and the script
re-run: it **exited 1 at the very first baseline check, before touching a single
mutation**, printing the oracle's own missing-binary message rather than
reporting a number. `corpus.rs` was then restored from a scratchpad copy (not
`git checkout --`), `git status` confirmed clean, and the corpus gate re-verified
at 42 of 42.

**Two mutations were declared wrong and the declarations were corrected to what
was measured**, which is recorded here because a script whose declarations are
edited to match its results is worth nothing unless the edits are visible:

* **Row 2** (a callee does not inherit its caller's traps) was declared
  DIVERGED/DIVERGED and measured **PASSED/DIVERGED**. This is a genuine finding,
  not a mis-declaration to paper over: **trap inheritance into a callee has no
  differential witness.** It now has its own `KNOWN GAPS` row, with the
  mechanism — every raise a corpus program makes inside a routine is a
  `RAISE ... RETURN`, which unwinds the routine before delivery, so the matching
  table is always the caller's own live one and never the inherited copy.
* **Row 5** (an `INTERPRET` fragment's echo resolves its own line) was declared
  DIVERGED/PASSED and measured **DIVERGED/DIVERGED** — caught by ten
  `run.rs`/`trace_oracle` tests as well as the corpus. Declaration corrected.

**One mutation could not be assessed and the script refused to score it**, which
is the guard working rather than a mutation being unlucky. An earlier attempt at
a `PROCEDURE`-shaped row pointed the callee's activation back at the caller's
frame while a fresh frame was already pushed; that trips a `roots.rs` invariant
and panics `corpus_differential` **before** it prints its `N of M matching`
line. `corpus_status` classified it `INFRA_FAILURE` and the script aborted.
Calling that a catch is exactly the defect 4a's first script shipped.

The twelve, and which instrument sees each:

| # | mutation | corpus | suite |
|---|---|---|---|
| 1 | `PROCEDURE EXPOSE` aliases nothing | DIVERGED | DIVERGED |
| 2 | a callee does not inherit its caller's traps | PASSED | DIVERGED |
| 3 | `USE ARG` binds each target to the next argument | DIVERGED | DIVERGED |
| 4 | a bare `RETURN` does not drop `RESULT` | DIVERGED | DIVERGED |
| 5 | an `INTERPRET` fragment's echo resolves its own line | DIVERGED | DIVERGED |
| 6 | `SIGL` is one line past the raising clause | DIVERGED | DIVERGED |
| 7 | a `CALL ON` trap disarms permanently when it fires | DIVERGED | DIVERGED |
| 8 | an omitted argument position traces no `>A>` line | PASSED | DIVERGED |
| 9 | the argument list's root is dropped (**criterion 4's control**) | PASSED | DIVERGED |
| 10 | `PUSH` inserts at the tail, like `QUEUE` | PASSED | DIVERGED |
| 11 | `PUSH`/`QUEUE` evaluate and trace but store nothing | PASSED | DIVERGED |
| 12 | **I17**: `DROP` of a stem as a plain slot clear | PASSED | PASSED |

**I17 is run rather than cited, and it survives both instruments as declared.**
The scoping document expected this mutant to become pinnable once 4b landed,
because nothing in 4a could hold a second reference to a stem. Both halves of
that are wrong: `b. = a.` already shares the object in 4a on both interpreters,
**and** the mutant is equivalent anyway, because "the slot holds a fresh empty
stem" and "the slot is unset" are not distinguishable through a second
reference — `a.1 = 'orig'; b. = a.; drop a.; say a.1 b.1` prints `A.1 orig`
either way. Declaring it a survivor makes the reclassification falsifiable: if it
ever starts being caught, I17 needs revisiting rather than the row being edited.

### 7. Zero `unsafe`, clippy clean, `fmt` clean — MET

`unsafe_code = "forbid"` stands at `[workspace.lints.rust]`. `cargo clippy
--offline --workspace --all-targets -- -D warnings` and `cargo fmt --all
--check` both exit 0, verified for this document.

### 8. A condition trap witnessed by a value a handler set — MET

`corpus/lang/condition_traps.rex` is the witness, compared to the oracle byte
for byte as part of the 42. It accumulates `ZWITNESS` across five blocks —
`SIGNAL ON SYNTAX` with `SIGL`, a re-armed trap under a second label, `SIGNAL ON
NOVALUE` on both a simple variable and a compound tail, `CALL ON USER` resolving
at the clause boundary, and `SIGNAL OFF` leaving the last raise untrapped — and
prints the whole string, so **a block that silently did not run removes its own
segment** rather than leaving the output unchanged.

The values are chosen so that no segment equals any variable's derived name or a
prefix of one. That is the specific vacuity this criterion exists to close: an
unset Rexx variable renders as its own uppercased spelling, so a handler flag
left unset reads as plausible data rather than as an obvious blank.

Three of the twelve mutations attack it and all three are caught: **6** (`SIGL`
one line off, which moves four numbers in the witness string at once), **7** (a
`CALL ON` trap disarming like a `SIGNAL ON` one), and **1** (`PROCEDURE EXPOSE`,
which the handler-visibility blocks depend on).

### 9. `PUSH`/`QUEUE` order pinned at unit level, and the construct ships undifferentiated — MET

The order is oracle-measured and pinned by `queue.rs`'s three tests:
`interleaved_push_and_queue_match_the_oracle_order` (`push "a"`, `queue "b"`,
`push "c"` leaves `c`, `a`, `b`), `queue_alone_is_plain_fifo` (the adjacent
success, which is what stays green if `push_front` quietly became a second
`push_back`), and
`push_and_queue_actually_write_into_the_running_interpreters_queue`, which runs
a program through `Interp::run_activation` and reads `Interp::queue` back.

**The recording is made, and it is executed rather than asserted.** Mutations 10
and 11 are the split, and both came out exactly as declared:

* Discarding the stored line while keeping evaluation and tracing (11):
  **corpus 42 of 42, suite fails.** No corpus program can observe what the queue
  stored.
* Collapsing `PUSH` into `QUEUE` (10): **corpus 42 of 42, suite fails.** Same.
* The exclusions file's own third measured claim — deleting the whole
  instruction arm — *is* caught differentially, because the expression stops
  being evaluated and traced at all.

So `PUSH`/`QUEUE` ships with its **wiring and its trace** differentially
verified and its **storage** verified only in-crate. That closes when 4c lands
its first `PARSE PULL`, `PULL` or `QUEUED()` corpus program.

### 10. The `base/keyword` L1 table, policed in both directions — MET as a policing criterion; the number is reported

`REXX_KEYWORD_GATE=1` exits 0 with all seven tests green, including
`the_exempt_set_matches_the_current_failures` in both directions.

**The measurement, reported and not gated:** **1,773 of 2,441 exact-spelling
`assertSame` calls extracted into 896 bodies (72.6%); 100 bodies pass, carrying
713 assertions.** The remaining 796 bodies are **790 `4c` plus 6
`defect:compound-do-control-variable`**, counted from the committed file at this
commit.

**The denominator's spelling matters and is stated:** 2,441 is the count of
calls spelled *exactly* `assertSame`. The obvious prefix match gives 2,561 and
silently classifies 120 `assertSameList` calls — a different method — as dropped
`assertSame` calls.

Three qualifications belong beside that number, and each one weakens it:

* **A `4c` attribution says what a body hits *first*, not what would make it
  pass.** Four bodies are known to differ, and they are not merely blocked —
  they are not equivalent to the method they came from. `CALL::test_expression`,
  `CALL::test_literal` and `CALL::test_on_name` fail **under the C++ oracle
  itself** (`Error 43, Routine not found`), because they call `::routine`s the
  standalone program does not carry; `NUMERIC::test_42` exits 3 because its body
  falls through into its own `dig: Return digits()`.
* So **the 790 are an upper bound on what landing 4c would fix, not a measure of
  4c's remaining surface.** The stronger claim was in `l1-coverage.md` and was
  corrected; it is not restated here.
* **`base/keyword` has zero Phase 5 dependency.** That is a genuine finding and
  safe to report: not one body in this table is blocked by the object model, so
  4b owes this table nothing further.

**This is the second exempt list and it behaves oppositely to criterion 2's.**
`tests/assertions.rs`'s 35 rows cannot light up at this gate or 4c's;
`rust/corpus/keyword-exempt.txt`'s 790 `4c` rows are **designed to fire at 4c's
gate**, each body starting to pass and failing the set until it is removed. Both
directions live in `rust/crates/rexx-exec/tests/keyword_assertions.rs` — grep the
fragments `now PASSES` and `is not on the committed`, which is how they are cited
here rather than by line number, since line numbers move whenever that file is
edited.

**I27 is not this gate's criterion and no figure from `TRACE.testGroup` is used
above.** That file yields 239, 342, 374, 393 or 437 expected trace-output lines
under five defensible anchorings, and the group is not runnable as extracted.
Criterion 3's own measured, named subset — 13 of 19 prefixes — is used instead.

---

## Step 3c: the ruling on the compound-`DO` gap

**Option 2. The gap is assigned to 4c, which moves it out of `KNOWN GAPS` and
into `EXCLUSIONS`, and makes it 4c's own gate criterion.** The row now reads
"EXCLUSIONS -- a compound variable as a DO control variable, owned by 4c" in
`docs/superpowers/plans/phase-4-exclusions.txt`.

**What the divergence is.** `do cv.j = 1 to 5` is legal Rexx and the oracle
iterates it, assigning the compound `CV.J` on every pass. This crate never binds
it: `bind_control` (`rexx-exec/src/run.rs`) writes the control variable through
`slot_of`, a flat name-to-slot lookup, so `CV.J` becomes the literal name of one
simple variable and no tail is resolved — while the same executor resolves the
same name correctly in `say cv.j` one line later. It is **not** a parse gap:
`cv.j` is a single symbol token and the parser interns `"CV.J"` whole. The
`LEAVE`/`ITERATE`/`END` forms naming a compound all dispatch **correctly**;
three probes narrow it to exactly one mechanism.

**Why an owner rather than the other two options.**

* **Not option 1, fix it here.** `run.rs` is outside this task's file list and
  so is `rexx-parse`, and the recorded cost includes a `rexx-parse` signature
  change (`Controlled::control` carrying the `VariableRef` shape an assignment
  target already does). Landing an interpreter behaviour change in the same
  commit that measures the tree would invalidate every figure in this document —
  the mutation table, the 42, the L1 counts — and would do it with no review
  round behind it. **The gate reports the tree 4b ships; it does not move it.**
* **Not option 3, ship it as an open gap.** That is what all eleven previous 4b
  tasks did, and every one of them was right, because `DO` was in none of their
  file lists. `phase-4-exclusions.txt` **is** in this task's, and it is the file
  where an owner is recorded, so the excuse does not transfer to the gate. An
  obligation with no owner is what let this drift from 4a through eleven tasks.
* **4c and not Phase 5**, because nothing here needs the object model. This is
  classic Rexx, the resolution already exists in the crate, and 4c is the last
  sub-phase at which a Phase 4 construct's own defect can be closed inside Phase
  4. `instruction_owner` returns `None` for `InstructionKind::Do` — implemented,
  not deferred — so Phase 4 must not close having claimed `DO` and shipped a
  known, witnessed wrong answer inside it with nobody named.

**Two rows described one defect, and the ruling merged them.** "A COMPOUND
CONTROL VARIABLE IS STORED IN THE WRONG PLACE" (Task 9's, carrying the `>C>`
trace symptom and the rexx-parse cost) and "A COMPOUND VARIABLE AS A DO CONTROL
VARIABLE IS NOT SUPPORTED" (Task 11's, carrying the six L1 bodies and the three
narrowing probes) were separate `KNOWN GAPS` entries for the same
`bind_control` mechanism. **The split is part of why it drifted unowned**: either
one read as the whole of it. A pointer stays in `KNOWN GAPS` so a reader who
knows it by either name finds where it went.

**The notice mechanism already exists and needs nothing new.** Six ooTest bodies
turn on this and they are the **only assertion failures in the whole
`base/keyword` table** — every other non-passing body there fails loudly with an
unimplemented construct. When the fix lands, all six start passing and
`the_exempt_set_matches_the_current_failures` goes red until they are removed
from the committed set. That is 4c's gate criterion for this row, and it is
automatic.

**Assigning the owner does not change `keyword-exempt.txt`, and must not.**
Those six rows read `defect:compound-do-control-variable` and stay that way:
their attribution is derived from the **outcome kind** —
`RunOutcome::AssertionFailed` maps unconditionally to that one constant in
`keyword_assertions.rs` — because these bodies *run and disagree* rather than
failing loudly with a phase-named message. Rewriting them to `4c` would
contradict the derivation the exempt file's own header states and would turn the
set assertion red immediately. **An owner records who owes the fix; it does not
reclassify how the harness sees the failure.**

---

## Step 3b: does per-sub-phase ownership attribution earn its keep?

**Recommendation for the 4c plan: keep the attribution, and spend the
consolidation budget on the third copy's *prose* instead. Do not act on this in
4b.**

### The ratio, re-measured at three points

| commit | harness (`owners`+`loud`+`coverage`) | interpreter (`run`+`eval`+`error`) | harness as % of interpreter |
|---|---|---|---|
| `e4caa7bf` (mid-4b) | 1,852 | 9,025 | **20.5%** |
| `5b2de07a` (after Task 11) | 1,928 | 15,838 | **12.2%** |
| `4c8c1f68` (**the tree this gate assessed**) | 1,928 | 15,838 | **12.2%** |
| after this gate's own commits | 1,976 | 15,841 | **12.5%** |

**The third row is the tree that was assessed, not the commit this document
lands in, and the fourth row exists because those are different.** The gate's
own work moves both totals: `bind_control`'s corrected doc comment adds 3
interpreter lines, and the fix round's `phase_4b_subset_matches_the_committed_
list` adds 48 harness lines. The fraction moves 12.2% → 12.5%, which changes
nothing about the argument — a gate's own instrument growing is not the
attribution surface tracking construct count — but labelling the third row with
a commit at which its numbers are false would be the same defect this step is
about.

**The third point is identical to the second, and that is a fact about the
commits rather than a measurement that failed to move**: the ten commits between
`5b2de07a` and `4c8c1f68` changed none of the six files — `git diff --stat` over
exactly those paths is empty.

**That is the whole of the claim, and an earlier version of this paragraph
overreached it in two ways that a reader would have taken as fact.** It said the
ten are "plan and documentation edits, including three that corrected this very
step's own framing". Neither half survives checking. **Two of the ten change
`rust/` source**: `8d6790c6` touches `keyword_assertions.rs`, `keyword.rs`,
`extract_keyword.rs` and `keyword-exempt.txt` across 480 insertions, and
`b23986d9` four of the same paths. The tree moved substantially between Task 11
and this gate — just not in the six files this ratio measures, which is the only
thing the empty diff shows. And **exactly one** commit corrected this step's
framing (`ceabe481`, which re-measured the ratio); of the remaining plan edits,
four rewrite the two-exempt-lists preamble and one corrects an unrelated
line-continuation claim.

**The trend runs against the argument consolidation was raised on.** Across 4b
the harness grew **4%** (1,852 → 1,928) while the interpreter it attributes grew
**76%** (9,025 → 15,838). A fixed cost that shrinks as a fraction while its
subject nearly doubles is a *weaker* case for consolidation than the mid-4b
snapshot suggests, and quoting only the 20.5% figure would put a thumb on the
scale.

**What the third data point at 4c's gate would have to look like to change the
recommendation.** 4c adds `PARSE` in all forms, 66 builtins, `ADDRESS`,
`VALUE`'s variable-access form and four condition/argument builtins — a large
interpreter increment with **very few new out-of-scope variants**, since most of
4c's work moves variants *in* scope, which *deletes* `loud.rs` witness rows. So
the null hypothesis is that the fraction falls again, to roughly 8-9%. **The
recommendation should change only if the harness grows super-linearly in the
interpreter** — concretely, if the three files exceed ~2,600 lines (a 35% jump
against 4a-to-4b's 4%) or if the fraction *rises* at all. A rise would mean the
attribution surface has started tracking construct count rather than sitting
flat, which is the only shape under which "one sub-phase at a time" is genuinely
paying per construct.

### The three things asked about

**1. The third copy.** Ownership data lives in `tests/owners.rs`'s tag tables,
in `loud.rs`'s witness rows (each carrying an `owner` string), and in
`src/lib.rs`'s `instruction_owner`/`expr_owner` — production code cannot reach a
test module, so the third copy is structural, not laziness.

**It is guarded, and the guard is real**: `every_out_of_scope_variant_fails_
loudly` runs each witness through `run_program` and asserts stderr **ends with**
`" is not implemented (OWNER)"`, so `loud.rs`'s owner strings and `lib.rs`'s
emitted messages are cross-checked on every run. Verified by mutation in this
phase.

**Cost of "make `owners.rs` the single source and assert `lib.rs`'s match equal
to it": higher than it sounds, and the reason is a shape mismatch rather than
effort.** The two encode different things. `owners.rs` is **variant**-grained
and ternary (`InScope` / `Phase(..)` / `Unreachable`); `lib.rs` is
`Option<&'static str>` and **arm**-grained — `InstructionKind::Call` reads
`Owner::Phase("Phase 5")` coarsely in `owners.rs` while `lib.rs` splits
`Named`/`Dynamic`/`Trap` → `None` from `Qualified` → `Some("Phase 5")`.
`loud.rs` already carries the reconciliation (`instruction_arm` and
`expand_for_witnesses`, both hand-maintained). A straight equality assertion is
therefore **not available**; the honest version is "assert `lib.rs`'s match
equals `owners.rs` expanded through `expand_for_witnesses`", which makes the
hand-maintained expansion table the new single point of truth and moves the
duplication rather than removing it. **Estimated at half a day, and it buys less
than it costs** — the data is already guarded.

**2. Are the phase strings load-bearing at all?** Mostly not, and the
differential corpus does make the "4b is done" claim better and more directly —
42 of 42 against the oracle says more than any owner string. What would actually
break under a binary implemented/not-implemented with the reason in the message
and no phase attribution:

* `keyword_assertions.rs` would lose its derived `unblocked_by` column. This is
  the one real loss, and it is not small: **790 of the 796 exempt rows are
  attributed by parsing the owner out of the loud message**, which is what makes
  that file impossible to drift from `instruction_owner` and what makes "a
  blocker moving between phases goes red here" true. Replacing it with
  hand-written phases would recreate exactly the stale-data problem the
  attribution exists to prevent.
* `coverage.rs`/`loud.rs`'s policing that an owner names a phase from the split
  table would go, and with it the check that caught the `ExprKind` ownership
  ambiguity at the 4a gate.
* Nothing else **derived from `instruction_owner`** — but "no other file
  mentions a phase" would be false, and a 4c planner inventorying this surface
  needs the difference. `trace_oracle.rs`'s `PREFIX_COVERAGE` carries
  `Coverage::Owned("4c")`/`Owned("Phase 5")` for six prefixes, and
  `assertions.rs`'s `EXEMPT` carries 35 `unblocked_by: "Phase 5"` rows. Both are
  **hand-written constants policed against a committed literal**, not values
  parsed out of a loud message, so removing the phase attribution from
  production would not touch either — they would keep working and keep needing
  hand maintenance. `corpus.rs` and `collect_stress.rs` read no phase string at
  all.

So the phase strings are load-bearing **for one consumer**, and that consumer is
the 796-row file 4c's gate fires on. **Removing them before 4c's gate would be
the wrong order.**

**3. Which defects the attribution caused.** Visible by the end of the phase:

* `loud.rs` deleting the witness covering `Call::Qualified` and `Call::Trap`
  when `Call` landed.
* Four occurrences of a witness implemented out from under a test.
* The `>I>`/`<I<` owner question, whose *existence* is an attribution artifact.
* The corpus-subset union plumbing (Task 0: **907 insertions with zero
  interpreter functionality**).
* `INSTRUCTION_WITNESSES`' own count comment going stale **four separate
  times** — "20", then "19", then "17/16", then "14/14" — each correction
  itself a count that the next task falsified. It is now stated as a property
  with the arithmetic asserted a few lines below rather than as a number in
  prose.

**And the sharpest instance, because it was measured rather than predicted.**
One stale fact — "`Call::Trap` is loud", false since Task 7 — had **three** prose
copies, found by three different agents across two tasks: `tests/owners.rs`
(Task 8's review), `tests/loud.rs`'s `INSTRUCTION_WITNESSES` doc (Task 8's
implementer, while fixing the first), and `src/lib.rs` under `ExprKind::Call`
(Task 8's re-review). **Task 7's own five review passes saw none of them**,
because each copy sits in a file Task 7 never had to touch.

**That is the finding that should drive 4c's decision.** The stderr assertion
guards the **data**; nothing guards the **prose about** the data, and the prose
is where all three copies were. The guarded duplication is cheap and has never
produced a wrong answer. The unguarded commentary around it is what rotted, four
times in one constant's doc comment and three times in one sentence about
`Call::Trap`. **So the consolidation worth costing for 4c is not merging the
three tables — it is deleting the prose that restates them.** That is
cheaper (no shape reconciliation, no new single point of truth), it targets the
copies that actually went stale, and it follows the rule this repo already has:
if a claim about a mutable in-repo aggregate is load-bearing, assert it; if it
is not, delete it.

**Not acted on in 4b, deliberately.** Consolidating the ownership harness while
the phase depends on it is the collision D5 warns about, and it would move every
gate figure between here and now.

---

## What this gate created, and what it found

* `docs/superpowers/plans/phase-4b-gate.md` (this document) and
  `rust/scripts/mutate-4b.sh` did not exist; both are new.
* `mutate-4b.sh` runs **two instruments per mutation** and has each row declare
  what both should say, where 4a's script ran the corpus alone and treated its
  silence as failure. That is what made rows 2, 8, 9, 10 and 11 expressible at
  all: five of the twelve mutations are invisible to a differential run, and
  four of those five are invisible for a *correct* reason.
* **One new gap found and recorded**: a callee's inherited trap table has no
  differential witness (`phase-4-exclusions.txt`, `KNOWN GAPS`). Found by
  mutation row 2 declaring the wrong thing and being measured.
* **One harness hole found by this gate's own review and closed**:
  `phase-4b.txt`'s twelve entries were pinned by nothing, so nine of them could
  be deleted with the whole suite staying green.
  `coverage.rs`'s `phase_4b_subset_matches_the_committed_list` closes it, the
  4b equivalent of the pin 4a's own branch review added for `phase-4a.txt` — the
  same hole, one sub-phase later, found the same way.
* **One ruling made**: the compound-`DO` control-variable divergence is 4c's,
  and its two `KNOWN GAPS` rows are merged into one `EXCLUSIONS` row.
* **One comment corrected outside this task's nominal file list**, because this
  commit's own change falsified it: `bind_control`'s doc comment
  (`rexx-exec/src/run.rs`) said the defect was "recorded as a KNOWN GAP", which
  stopped being true the moment the row moved. A doc-comment-only change, no
  behaviour; flagged here rather than left for a reader to trip over, per the
  standing rule that a comment stating something false must be corrected rather
  than hedged.

## What went wrong, so the next gate expects it

* **Two of twelve mutation declarations were wrong.** Both were corrected to
  what was measured, and one of them was a real finding rather than a slip. A
  script whose declarations are edited to match its output is worthless unless
  the edits are visible, which is why both appear in criterion 6 above and in
  the script's own comments.
* **The first `PROCEDURE`-shaped mutation broke the harness rather than the
  behaviour**, panicking `corpus_differential` before it could print a
  comparison. The `INFRA_FAILURE` class caught it and the script aborted rather
  than scoring it. A mutation that breaks the instrument has not been tested,
  and 4a's first script would have called it a catch.
* **The compound-`DO` defect was recorded twice, in two `KNOWN GAPS` rows, and
  the duplication helped it stay unowned** — either row read as the whole
  defect, and neither named an owner. Recording a gap twice is worse than
  recording it once, and the second copy is the one that hides the first's
  incompleteness.
* **And this document then did the same thing inside the row that says so.**
  The merged `EXCLUSIONS` row's new ownership paragraph re-listed all six ooTest
  bodies, called them the only assertion failures in `base/keyword` and cited
  the oracle re-run — every one of which the row's own body already said a few
  paragraphs down. Caught by review, not by the author writing the sentence
  immediately above it. Naming a defect is not the same as being immune to it.
* **This gate claimed a protection it did not have.** Criterion 1's
  falsification said a deleted `phase-4b.txt` entry would leave a variant
  unwitnessed. Measured: **nine of twelve would not**, and deleting criterion
  8's only witness left three criteria still reporting MET while the headline
  shrank 42 → 41. The claim was plausible because it is *true of
  `phase-4a.txt`* — 4a's own branch review had built exactly that pin — and it
  was carried across to a second file without being re-run. **A falsification
  clause is a claim like any other and needs the same measurement**, which is
  the one place this document reasoned instead of running, having said in its
  own criteria section that reasoning is what fails here.
* **Two false sentences shipped in Step 3b, both attached to a true
  measurement.** The empty six-file diff is real and reproduces; the
  characterisation next to it ("plan and documentation edits", "three that
  corrected this step's framing") was wrong on both counts — two of the ten
  commits change `rust/` source, and exactly one touched this step. The
  measurement was checked and the sentence summarising its context was not,
  which is this project's most-repeated defect shape and the reason
  `rust/CLAUDE.md` forbids mutable in-repo aggregates in prose.
