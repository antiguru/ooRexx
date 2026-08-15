STATUS: DONE

# Pre-flight audit: Task 13 (Trace) and Task 16 (gate harnesses)

Auditor: read-only review agent, 2026-07-31. Claims checked against the tree
and the oracle; every oracle call under the ulimit.

**Scoping answer, up front: yes, most of the continuous instrument can be
built now — but the instrument the lead is describing is mostly Task 14,
not Task 16.** The corpus subset file already exists (`rust/corpus/
phase-4a.txt`, 26 entries); what is missing is Task 14's harness
(`tests/corpus.rs` does not exist), and nothing in a differential runner
depends on the phase being complete: it compares oracle against rexx-run,
both fully defined today, so it reports a partial implementation honestly
by construction. Of Task 16 proper, the enumerations and set assertions can
be built now with stable content; the mutation script's infrastructure can;
the gate assessment cannot and should not. Details in the scoping section
at the end.

---

## Task 13 (Trace)

### 1. Claims checked against the tree

- **The 19-prefix claim is right.** `RexxActivation.hpp:89-109` enumerates
  exactly 19 prefixes, `TRACE_PREFIX_CLAUSE` (0) through
  `TRACE_PREFIX_INVOCATION_EXIT` (18); the `TRACE_OUTPUT_*` values at 30+
  are tagging, not prefixes, and do not disturb the count. The brief's
  reachable set (8 named + "a prefix-operator line") matches the design
  spec's criterion 3, which names all nine including `>P>`.
- **The trace sink matches what is built**: `Interp` has `trace: Vec<u8>`
  (lib.rs:502) separate from `out`, flowing to `Outcome.stderr`, exactly
  D17's measured stderr/stdout split. The brief's Step 3 "emitting to the
  trace sink (stderr)" is consistent with the tree.
- **`rexx-parse/tests/sourceline_oracle/` exists** (dir plus
  `sourceline_oracle.rs` reader), so the committed-expectations pattern the
  brief cites is real. `trace.rs` and `tests/trace_oracle/` do not exist
  yet; the Create lines are accurate.
- **The one big false premise is D17's "from the start", and it makes the
  Files section wrong.** D17 decided "the dispatch loop emits a trace event
  per evaluation step from the start" precisely so trace would not be a
  retrofit. The tree has **zero trace emission points**: nothing in
  `eval.rs` or `run.rs` writes to the trace sink (the only writer is
  `execute`'s error path). Tasks 7, 8 and 9 built the dispatch loop with no
  event hook, so Task 13 IS the retrofit D17 forbade — it must thread an
  event through `eval_node`'s arms (about eighteen of them now) and
  `step`'s, i.e. **modify `eval.rs` and `run.rs`, neither of which the
  Files section lists** (same defect class as Task 11's missing `eval.rs`).
  `run.rs` is also the file under active contention. And
  `InstructionKind::Trace` execution (the `TRACE R`/`TRACE I` instruction
  itself, `Trace` enum at ast.rs:1291, 4 variants) is a `step` arm in
  `run.rs` that no step of the brief mentions adding.

### 2. Constraint violations if followed literally

- The Step 5 commit block adds `rust/crates/rexx-exec/tests/trace.rs`, an
  integration test the Files section's own "in-file `#[cfg(test)] mod
  tests`" line contradicts. **Third brief running with this defect.** One
  nuance for the dispatch to rule on: the pattern the brief cites as
  precedent (`sourceline_oracle`) itself uses an integration reader test in
  rexx-parse, from Phase 3, before the in-file rule. Either the reader test
  is ruled a public-surface test (fine as `tests/trace.rs`, then the Files
  section is what needs correcting) or the in-file rule applies (then the
  git add line does). The brief currently asserts both.

### 3. What the brief needs and does not say

- **The indentation rule.** The oracle's `*-*` lines are indented by block
  nesting, and the error report's clause echo shares that indentation
  (measured in the Task 10/11 pre-flight: loop-condition failure echoes
  carry two extra spaces; other error classes none). Task 10 is
  characterising the per-construct counting right now. Task 13's brief
  predates the discovery entirely — Step 2's "spacing, quoting and
  indentation are unspecified anywhere but the oracle" is true but
  understates that a sibling task is *concurrently* deriving the same rule
  for the error path. What Task 13 inherits: the per-construct nesting
  counts from Task 10's work. What it must not do: re-derive them from its
  own probes and diverge. What needs coordinating: the indentation helper
  will be shared between trace formatting and `failure_site`'s echo, so
  whichever task lands second must use the first one's, and the dispatch
  should say which file owns it.
- **The expected-line count, verified with a correction waiting to
  happen.** The brief carries no number (good). The design spec's D17
  paragraph records 34 blocks / 342 lines / 128 `*-*` / 214 value-or-marker
  — **independently reproduced exactly**, with an end-marker-aware count
  over anchored lowercase `::resource` directives. But the spec's own
  anchoring lesson has a second chapter: the file also contains six
  uppercase `::RESOURCE` blocks (line 1205 on) that a case-sensitive scan
  misses — three are embedded `.rex` program sources, and three are
  `*_expected` blocks holding **32 more lines of expected trace output**
  (`>I>`/`<I<` invocation prefixes, 4b+ territory). A future case-
  insensitive recount will get 37 expected-output blocks / 374 lines and
  conclude the spec is wrong. No 4a impact (all of it is collected by
  4b/4c), but the figure should be annotated before it fights its third
  recount.

### 4. Contradictions with Task 16 / shipped tasks

- Task 16's coverage enumeration includes the `Trace` enum, whose witnesses
  need Task 13's execution. Ordering 13-before-16 is already the plan's;
  no conflict, just a dependency worth stating in 16's dispatch.
- No contradiction with the shipped sinks/reporting: the brief's stderr
  claim matches Task 9/12's built reality.

---

## Task 16 (gate harnesses)

### 1. Claims checked against the tree

- **"Create `docs/superpowers/plans/phase-4-exclusions.txt`" is stale.**
  The file exists (8,939 bytes, amended this session) and its content
  *exceeds* the brief's Step 4: the 15 whole exclusions (named, two phases),
  the 3 partial rows (VALUE, ADDRESS(), QUEUED), a DEVIATIONS section with
  the stem-order row **plus** a second deviation (the 11.1 evaluation-depth
  row, parity-vs-deviation carefully split), **plus** a KNOWN GAPS section
  (4 rows) with a deliberately asymmetric amendment rule the brief never
  mentions. Following Step 4 literally would *regress* the file. What Task
  16 still owes on this front: not writing the file but **asserting its
  sets** in the harness (Step 1's "asserted the way the exclusions file
  is") and keeping the gate's "66 of 81 builtins" phrasing consistent with
  it. What is done: everything else in Step 4.
- **Step 5's path is wrong.** `docs/superpowers/plans/phase-4a-gate.md`
  does not exist. The seven criteria live in the design spec,
  `docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md`, section
  "4a exit gate" (line 446, criteria 1-7 present and numbered).
  `phase-4a-gate.md` is evidently the file Task 16 *writes*, in the shape
  of the existing `phase-3-gate.md` — the dispatch should say "assess the
  design spec's criteria, writing plans/phase-4a-gate.md", or the
  implementer will hunt for a nonexistent input.
- Verified true, each against the tree: `keyword()` maps both `When` and
  `WhenCase` to `"WHEN"` (ast.rs:912, exact); **40** `InstructionKind`
  variants and **15** `ExprKind` variants (counted); `LoopKind` (6),
  `EndStyle` (6), `Trace` (4), `PrefixOp`, `Operator` all exist;
  `Operator::Backslash` exists (token.rs:229) and a dyadic `\` is **error
  35.1** on the oracle (probed: `Incorrect expression detected at "\"`), so
  the owner-string-not-witness rule for it is correctly reasoned; every
  AST field the Step 3 mutation list names exists (`If::false_target`,
  `When::exit`, `Loop::end`, `Controlled::order`). `rust/scripts/` does not
  exist yet (Create implies mkdir; fine).

### 2. Constraint violations if followed literally

- **None found, including the place I expected one.** The Files section
  creates `tests/coverage.rs` and `tests/loud.rs` — integration tests — but
  the in-file rule covers *private subjects*, and these tests' subjects are
  public cross-crate surface: rexx-parse's AST enums and `run_program`'s
  `Outcome`. `tests/spike.rs` is the standing precedent. Flagging the
  reasoning so it can be overruled rather than discovered. No git add
  defect: the brief's Step 6 is just "Commit."

### 3. What the brief needs and does not say

- `rust/corpus/phase-4a.txt` exists with 26 entries (this also makes Task
  14's own "Create" line stale, noted in passing though 14 is outside this
  audit). Task 16's Step 1 quantifies witnesses "in the subset" — that
  subset is now a concrete committed file, and the brief should point at
  it.
- The not-implemented exit code: `loud.rs` should test against the named
  constant `NOT_IMPLEMENTED_EXIT`, not a literal — the exclusions file
  itself (lines 34-39) records why writing the number anywhere that
  nothing recompiles goes stale silently. The band argument (outside
  157..253) lives only in that file.
- Step 2's "names its owner" needs the loud message's owner strings to
  come from the same policed set as Step 1's owner arms, or the two
  enumerations can drift apart naming different phases for one variant.
  Nothing in either step says they share a table.

### 4. Contradictions

- None hard between 13 and 16 beyond the Trace-witness dependency above.
  With 14: Task 16 Step 1's witness rows point into the subset that Task
  14's harness runs; if 16's coverage test runs witnesses itself, the two
  harnesses re-run the same programs — the design spec reads as criterion 1
  being ONE harness (subset runs + coverage property over it), so 16
  should consume 14's runner rather than grow a second one. Worth one line
  in the dispatch.

---

## The scoping question: build now or gate day?

**What the lead actually wants formalized is Task 14's harness, and it can
be built today.** The differential runner has no dependency on phase
completeness: for each subset entry it runs oracle and rexx-run and
compares bytes — a 17/26-failing report is exactly as honest as a
0/26-failing one, which is why the ad-hoc shell loop it replaces has been
useful. The subset file exists; the committed-expectations pattern
(sourceline_oracle) exists; the only design decision is that the always-on
mode must **report** (N of M, per-program status, exit 0) while a strict
mode (env flag or `#[ignore]`-gated test) asserts zero divergences — that
strict switch is the only genuinely gate-day part. The design spec's
criterion 1 requires the gate to run it strict; nothing requires the
interim runs to.

Of Task 16 proper, split by whether content is stable before 10/11/13 land:

- **Build now, zero churn:** the no-wildcard coverage enumeration (Step 1).
  The out-of-4a owner assignment is complete and stable ("the assignment is
  complete today", verified: 40 + 15 variants), the exclusions set
  assertion has a final-shaped file to assert, and the no-wildcard match
  starts paying immediately — a new variant added during 10/11/13 becomes a
  compile error in the harness rather than a silent gap, which is worth
  more *during* those tasks than after. 4a-scope witness rows can name
  their programs now; only "witness must pass" is gate-day strictness, same
  report-vs-strict split as above.
- **Build now, the stable half:** `loud.rs` (Step 2) for the variants that
  will STILL be loud at gate (the 4b/4c/Phase-5/Phase-7 sets). Those rows
  never churn, and they are the criterion's actual point ("closes a surface
  larger than 4a's own"). Rows for 4a-owned variants would flip from loud
  to executed as each task lands — skip them entirely; at gate they are
  covered by the coverage enumeration instead.
- **Build the skeleton now, grow with the code:** the mutation script
  (Step 3). The apply/revert/guard loop plus the three patterns whose
  target code exists today (Abuttal-as-Blank, `=` as `==`, created-digits/
  created-form formatting). The If/When/Loop/LEAVE patterns are added as
  Tasks 10/11 land — and the unapplied-pattern guard is precisely what
  makes incremental growth safe, since a pattern committed before its code
  exists fails loudly instead of reporting phantom coverage.
- **Gate day, correctly:** Step 5's assessment (it narrates final
  measurements), and flipping every report-mode switch to strict.

**Is there a reason not to run a gate harness continuously? None
fundamental.** The one real friction is tables that encode *current* state
churning on every task; the resolution is to encode *gate* state and run in
report mode until the gate, which is what every split above does. A
secondary caution: a continuously-green report has none of a gate's force,
so the gate-day flip to strict must actually happen — the report mode
should print its own mode loudly so nobody mistakes a report for the gate.

## Not reached

- The `>P>`-and-friends witness programs themselves (Task 13's Step 1 work,
  not this audit's); no trace-output bytes were captured.
- Whether `TRACE R`/`TRACE I` parsing is complete in rexx-parse (the Trace
  enum exists with 4 variants; parse-side completeness was Phase 3's gate).
- Task 14/15's briefs in full (out of scope; only their intersection with
  the scoping question was checked).
- The forwarded-test `::RESOURCE` blocks' runtime semantics (characterized
  by content only: `>I>`/`<I<` invocation-prefix expectations).
