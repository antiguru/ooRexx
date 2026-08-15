# Task 6 review: `SIGNAL` to a label, and `SIGNAL VALUE`

Reviewed at `53b75f90` against `375b662d`. Every oracle run below was made from a
fresh `mkdir`ed directory under the session scratchpad, one program per
directory, absolute paths for every redirect, `( ulimit -v 1048576;
LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx FILE )`, stdout/stderr/rc read
as three separate descriptors. Every experiment that touched the tree was
reverted and `git status --porcelain` confirmed empty afterwards.

## Verdict 1 -- spec compliance: **NOT MET** (one Step 1 shape skipped, one
## behaviour of the implemented construct missing)

| Requirement | Result |
| --- | --- |
| Step 1: `SIGNAL` out of a nested `DO` | **met** -- measured, transcript in report, reproduces exactly |
| Step 1: `SIGNAL` out of a `SELECT` | **not met** -- never measured, never tested, absent from the corpus witness (I3) |
| Step 1: `SIGNAL` to a label not in the current body | **met** -- 16.1, rc 240, reproduces exactly |
| Step 1: `SIGNAL` from a called routine to a label in that routine's body | **met** (same body at this phase; the report says so correctly) |
| Step 1: `SIGNAL` from a called routine to a label in the caller's body | **met** -- reproduces exactly, and the report's explanation of *why* is right |
| Step 1: `SIGNAL` out of an `INTERPRET` fragment | **met** -- both the success and the failure twin reproduce exactly |
| Step 1: `SIGNAL VALUE` naming no label | **met** -- number, empty string, ordinary string, lowercase-vs-upcased all reproduce |
| Step 2: failing tests transcribed from those measurements | **met** -- 8 tests, negative control run, and I confirmed two independent mutants each kill a distinct test |
| Step 3: implement `Signal::Label` and `Signal::Value` | **not met in full** -- `SIGL` is not set (C1) |
| Step 4: suite, `owners.rs`, commit | **met** -- fmt clean, clippy clean, `cargo test --workspace` green (68 `test result: ok`, 0 failures), corpus 37/37 REPORT **and** `REXX_CORPUS_GATE=1` STRICT |
| `Signal::Trap` keeps failing loudly with its own witness row | **met** -- row present at `loud.rs:258`; `signal off error`, `signal on error` and `signal on error name handler` all exit 120 with `rexx-exec: SIGNAL is not implemented (4b)` |
| In-scope witness uses `SIGNAL label`, not `SIGNAL ON` | **met** -- `signal_forms.rex` uses `signal past_loop`, `signal value target`, `signal nowhere`; no `SIGNAL ON` anywhere |
| A `Flow` variant added only where the existing ones do not fit | **met** -- the decision is correct; the argument given for it is not (I1/I2) |
| Nothing beyond scope | **met** -- no expected-byte literal changed, no unrelated file touched |

## Verdict 2 -- task quality: **NEEDS WORK**

The implementation is correct on everything it was measured against, and the two
non-obvious things the brief warned about (resolve against the *activation's*
body; do not let a `SIGNAL` ride `Goto` out of a fragment) are both handled
right. What is wrong is the evidence: the single most-argued decision in the
task has no test that fails if it is reverted, and the sentence offered as its
measurement provably does not discriminate.

Counts: **1 Critical, 3 Important, 4 Minor.**

---

## Step 1 measurements, re-run independently

One line each; all reproduce the report's transcripts byte for byte unless noted.

1. **`SIGNAL` out of a nested `DO`** (`if i = 2`, second pass) -- oracle rc 0,
   stdout `before\ni= 1\nreached there\n`; the loop's later iterations and the
   clause after `END` never run; the `SIGNAL` clause traces with no `>>>` line.
   Against `rexx-run`: stdout and rc identical, stderr differs by exactly the
   two lines `>>>     "1"` / `>>>     "2"` -- the pre-existing `Controlled`
   retrace gap, nothing else.
2. **`SIGNAL` out of a `SELECT`** -- oracle rc 0, stdout
   `before\nreached there\n`, `after select` never runs; trace shows `select`,
   the `when`, `then`, `signal there`, then the label. `rexx-run` matches
   **byte for byte on all three descriptors.** Not in the report.
3. **`SIGNAL` to a label not in the body** -- rc 240, stdout `before\n`, stderr
   `Error 16 ... line 2:  Label not found.` / `Error 16.1:  Label "NOWHERE" not
   found.` `rexx-run` matches on all three.
4. **`SIGNAL` from a called routine to a label in the caller's body** -- rc 0,
   stdout `in sub\ncaller label reached\n`, stderr empty; the clause after
   `CALL` never runs. `rexx-run` matches. The report's explanation is correct:
   the callee shares the caller's body at this phase, and the program ends
   because the callee's activation falls off the end.
5. **`SIGNAL` out of an `INTERPRET` fragment** -- rc 0, stdout
   `before\nreached there\n`; fragment clause echoes at the enclosing
   `INTERPRET`'s line, then the enclosing label. Failure twin: rc 240, both
   clauses echo innermost-first at line 2. `rexx-run` matches both.
6. **`SIGNAL VALUE` naming no label** -- `123` -> `Label "123" not found.`,
   `''` -> `Label "" not found.`, `'there'` with `there:` present ->
   `Label "there" not found.`, all rc 240, no shape check of any kind. The
   success form traces `>K>   "VALUE" => "THERE"`. `rexx-run` matches all four.
7. **Quoted vs bare (report's 2b)** -- `signal "sub"` with `sub:` present is
   16.1 naming `sub` verbatim; `signal Sub` and `signal "SUB"` both resolve.
   `rexx-run` matches all three.

Also re-derived independently: **47.2** (label inside `DO`/`END`), **47.3**
(label as an `IF`'s `THEN` body), both rc 209 exactly as the report says -- and
**47.4**, `Labels are not allowed within a SELECT block`, which the report and
the code comment do not mention (M2).

## Ruling: `Flow::Signal(usize)` versus reusing `Flow::Goto`

**The decision is right. The argument printed beside it is wrong in two
places, and nothing in the suite defends it.**

The mechanism is exactly as claimed. `run_fragment` (`run.rs:4118`) steps the
fragment with `run_bounded(&code, 0, code.body.instructions.len(), ...)`, and
`run_bounded`'s absorption test is `Flow::Goto(target) if target >= start &&
target <= end` (`run.rs:3071`, `end` **inclusive**). With `start == 0` any
escaping target `<= fragment_len` is swallowed and re-interpreted as an index
into the fragment's own instruction array. `Flow::Signal` does not match that
guard, so it falls to `other => return Ok(other)`.

I verified the forwarding by reading every consumer, and there are **five**
catch-alls on the path, not the four the comment lists:

* `run_bounded` `run.rs:3072` `other => return Ok(other)` -- forwards.
* `do_body_outcome` `run.rs:3586` `other => Ok(Some(other))` -- forwards;
  `run_loop`'s `Simple` arm (`:3124`) and `run_repeating` (`:3432`) both return
  it unchanged.
* `leave_select` `run.rs:2934` `other => Ok(other)` -- forwards. The
  `OTHERWISE` redirect just above it (`run.rs:1314`) is `if let Flow::Goto(target)`,
  so a `Signal` correctly cannot be mistaken for the marker jump.
* `run_fragment` `run.rs:4162` `other => Ok(other)` -- forwards.
* **`If`'s own true-branch arm, `run.rs:1106-1109`** `other => Ok(other)` --
  forwards, and is not named in the doc comment's list (M2).

Consumed only at `run_activation`'s top-level dispatch (`run.rs:663`), plus the
test miniature `run_activated` (`run.rs:5292`). `exec_call` never sees one: a
callee's `SIGNAL` is consumed inside the callee's own `run_activation` and the
callee ends as `Ended::Exited` (`run.rs:2514`), which is what makes measurement 4
come out right.

### The program that breaks under `Goto` reuse

The report's own headline measurement does **not** discriminate. I patched the
two `Ok(Flow::Signal(target))` sites to `Ok(Flow::Goto(target))`, rebuilt, and
re-ran:

* `interpret "signal there"` with `there:` at body index 4 and a 1-instruction
  fragment -- **output identical to the oracle.** No collision: `4 > 1`.
* My `g1`:

  ```rexx
  say 'A'
  interpret "nop; signal here; say 'WRONG BRANCH RAN'"
  here:
  say 'landed correctly'
  exit
  ```

  target index 2, fragment length 3, `2 <= 3` -> absorbed. Oracle and the
  shipped build print `A` / `landed correctly`. Under `Goto` reuse the build
  prints `A` / **`WRONG BRANCH RAN`** / `landed correctly` -- a silent wrong
  jump into the fragment's own instruction space, exactly the predicted failure.
* My `g2` (label at body index 0, `interpret "signal top"`): oracle and shipped
  build print `at top` twice, rc 0. Under `Goto` reuse the fragment's own
  `signal` is at fragment index 0, the jump targets itself, and the process
  **hangs** (killed at rc 124).

So the risk is real and reachable. But two of the three statements offered for
it are not:

* `run.rs:199-202` -- "Measured: `interpret "signal there"` reaches an enclosing
  `there:` ..., **which only holds if nothing along the way can mistake the
  escaping jump for one of its own**." Measured: it holds anyway. That program
  reaches `there:` with `Goto` reuse in place.
* `run.rs:195-196` (and twice in the report) -- "coincidental overlap is the
  **common case, not the exception**". Unmeasured, and false for every program
  either document exhibits. Overlap needs `label_index <= fragment_len`, which
  typical one-clause fragments and typical label positions do not satisfy.

The sufficient and true reason is narrower: the two index spaces are unrelated,
any overlap is a silent wrong jump rather than a crash, and overlap is
reachable in ordinary programs (`g1`, `g2`).

### And nothing tests it

**With `Flow::Signal` collapsed into `Flow::Goto`, the whole tree stays green:**
`cargo test -p rexx-exec` -> 252 lib tests pass, every integration target passes,
and `REXX_CORPUS_GATE=1` corpus reports **37 of 37 matching**. The design
decision that gets three paragraphs of doc comment has no witness at all.

## Ruling: the loop-gap witness choice

**Sound. It does not dodge a divergence this task should have surfaced.**

* The gap is genuinely pre-existing and genuinely disclosed --
  `run.rs:3633`, `LoopState::Controlled`, citing `DoBlock::checkControl`.
* It is genuinely `SIGNAL`-independent: I ran `trace r` / `do i = 1 to 3` /
  `say 'i=' i` / `end` with no `SIGNAL` anywhere; stdout and rc match, stderr
  is short by exactly the same `>>>` pairs, three times over.
* Most importantly, the gap is **not hiding a `SIGNAL` defect**. I ran the
  report's original second-pass shape (`if i = 2`) end to end: stdout matches,
  rc matches, and the stderr diff is *only* the two known lines -- no missing
  `end` echo, no wrong landing line, nothing `SIGNAL`-shaped.
* The alternative was worse. Keeping the second-pass witness would have meant
  either committing an expected-byte literal that encodes the crate's known
  wrong output as correct, or landing a red test. Routing around it and saying
  so in both the corpus header and the unit-test doc comment is the right call,
  and it matches what `interpret_error_echo.rex` already does.

The one thing I would add: neither the report nor any comment records the
positive fact I just measured -- that the second-pass shape diverges *only* by
the known two lines. Without it the next reader cannot tell "routed around a
known gap" from "routed around a gap that might be covering something". Worth
one sentence in the witness's header.

## Findings

### Critical

**C1. `SIGNAL` does not set `SIGL`, and the gap is disclosed nowhere.**
`rust/crates/rexx-exec/src/run.rs:1674-1696` (both `Signal` arms).

```rexx
say 'sigl before:' sigl
signal there
say 'no'
there:
say 'sigl after:' sigl
```

Oracle: `sigl before: SIGL` / `sigl after: 2`. `rexx-run`: `sigl before: SIGL` /
`sigl after: **SIGL**`. Same for `SIGNAL VALUE` (oracle `3`, crate `SIGL`).

This is a **silent wrong answer**, not a loud gap -- the category `corpus.rs`'s
own module doc says the tree does not have ("Every mismatch today is a *clean
loud failure*: nothing produces a wrong answer"). `SIGL` appears nowhere in
`docs/superpowers/plans/phase-4-exclusions.txt`, `rust/corpus/README.md`,
`phase-4a.txt`, `phase-4b.txt`, or any source comment; the only occurrences in
the tree are in `corpus-l1/`, which runs nothing.

Fairness: the root is **shared with Task 3** -- an internal `CALL` is also a
transfer of control to a label and the oracle sets `SIGL` for it too (measured:
oracle `in sub sigl: 2`, crate `SIGL`). Task 6 extends an existing undisclosed
divergence to a second construct rather than creating one. It is Critical
because it is undisclosed and because this task is the one declaring `SIGNAL`
implemented and adding a corpus witness that enumerates "what this program's
differential run pins" without mentioning it.

Fix, minimum: record it as a known divergence in
`docs/superpowers/plans/phase-4-exclusions.txt` and in `signal_forms.rex`'s
header, and name an owner. Task 7 is the natural home -- `SIGL` is also set on a
trapped condition, which is Task 7's -- but that has to be written down now, not
inferred. Fix, proper: set `SIGL` at the point of transfer in both `SIGNAL` arms
and in `resolve_and_run_call`; the clause line is already available through
`self.clause_site(source, instruction)` and the slot machinery already exists
(`self.slot_of(b"RESULT")`, `run.rs:2524`). Measure the fragment and nested
cases first -- do not assume the enclosing `INTERPRET`'s line.

### Important

**I1. The `Flow::Signal`-vs-`Goto` decision has no test that can fail.**
`rust/crates/rexx-exec/src/run.rs:1675`, `:1695`.
Patching both to `Ok(Flow::Goto(target))` leaves all 252 lib tests green, every
integration target green, and the STRICT corpus at 37/37. None of the eight new
tests touches the collision case: `signal_escapes_an_interpret_fragment_to_
reach_an_enclosing_label` (`run.rs:8600`) uses a 1-instruction fragment and a
target at index 4, which cannot collide.
Fix: add the two programs above as unit tests -- `g1` asserting stdout is
`A\nlanded correctly\n` (a `Goto` build prints `WRONG BRANCH RAN` in the middle),
and a bounded variant of `g2`. Both are three lines and both die instantly under
the reuse.

**I2. Two false/unsupported statements in `Flow::Signal`'s own doc comment.**
`rust/crates/rexx-exec/src/run.rs:199-202` and `:195-196`; the same two claims
appear in `task-6-report.md` section 4 and in "Did `Flow` need a new variant".
`:199-202` says the `interpret "signal there"` result "only holds if nothing
along the way can mistake the escaping jump for one of its own" -- measured, it
holds under `Goto` reuse too. `:195-196` calls coincidental overlap "the common
case, not the exception", which is unmeasured and false for every program cited.
Fix: delete both clauses and state the reason that is actually load-bearing --
the index spaces are unrelated, `run_bounded`'s guard is `target <= end` with
`start == 0` so any overlap is absorbed silently, and overlap is reachable
whenever the target's index is no greater than the fragment's length (cite the
`g1`/`g2` programs, which become the tests in I1).

**I3. `SIGNAL` out of a `SELECT` was never measured, never tested, and is not in
the corpus witness.** It is a named item on this task's Step 1 list.
`rust/corpus/lang/signal_forms.rex` covers `DO` only; the eight unit tests cover
`DO`, `INTERPRET` and a called routine, never `SELECT`. I measured it (oracle rc
0, `before` / `reached there`, `after select` never runs) and `rexx-run` matches
byte for byte on all three descriptors -- so this is a hole in the record and in
the test set, not a behavioural defect. It matters because `leave_select`
(`run.rs:2934`) is one of the forwarding sites the design argument depends on
and it is the only one with no witness.
Fix: add the transcript to the report and one unit test asserting the traced
`SELECT` escape; optionally fold a `SELECT` block into `signal_forms.rex`.

### Minor

**M1. Stale line reference, stale on the day it was written.**
`rust/crates/rexx-exec/src/run.rs:190-191` cites `run.rs:2153-2154` for the
`CALL` label-resolution fix. In the shipped tree those lines are
`assign_by_name`'s body; the real site is `run.rs:2264-2268`. The number was
correct at `92e80218` (where the brief quotes it) and this very diff moved it.
`resolve_signal_target`'s own copy at `:2175` hedges with "in the tree this task
started from"; this one does not.
Fix: drop the numbers, name `resolve_and_run_call` alone -- `phase-4b.txt`'s own
header already warns that positional references go stale by construction.

**M2. Two incomplete enumerations in the same doc comment.**
`rust/crates/rexx-exec/src/run.rs:204-210`: "**Nesting inside `DO`/`LOOP` or `IF`
needs no equivalent care** ... (47.2, 47.3, measured), so a `SIGNAL` target can
never sit strictly inside a range `run_bounded` is currently absorbing a `Goto`
into." The conclusion is universal but the evidence covers two of the three
enclosing constructs: `SELECT` also runs its `WHEN` and `OTHERWISE` bodies
through `run_bounded` (`run.rs:1329`, `:2894`). The conclusion happens to be
true, because a label inside a `SELECT` is **47.4** -- measured, rc 209,
`Labels are not allowed within a SELECT block; found "INSIDE"` -- but 47.4 is
cited nowhere. This is precisely the "sound inference from a true premise
missing a second premise" shape `CLAUDE.md`'s Method section names.
Second, `:212-215` lists the forwarding catch-alls as
`run_bounded`/`do_body_outcome`/`leave_select`/`run_fragment`; `If`'s own arm at
`run.rs:1106-1109` is a fifth.
Fix: add 47.4 and `SELECT` to the first list, and `If`'s arm to the second.

**M3. `run_activated`'s `unreachable!` message no longer describes its range.**
`rust/crates/rexx-exec/src/run.rs:5285`: "run_bounded never escapes a top-level
program's own `[0, len)` range". Since this task's loop, `start` is the last
`SIGNAL` target rather than 0, so the range is `[start, len)`. The arm is still
genuinely unreachable -- every construct at index `>= start` returns a `resume`
`> start` -- but the stated reason no longer matches the code.
Fix: say `[start, len)` and keep the argument (a construct's resume point is
always at or after its own index, and `start` is always a label index).

**M4. Citation that does not carry the claim.**
`rust/crates/rexx-exec/src/run.rs:1664-1666`: "`name` is already upcased for a
bare symbol and verbatim for a quoted one (`rexx-parse`'s own `Signal` doc)".
`rexx-parse`'s `Signal` doc (`crates/rexx-parse/src/ast.rs:1127-1131`) says only
"`SIGNAL label` and `SIGNAL "label"`" and documents no case rule at all. The
claim itself is true -- I measured all three spellings -- but the pointer is
empty.
Fix: either drop the parenthetical and cite the measurement, or add the case
rule to `ast.rs`'s `Signal::Label` doc and keep the pointer.

## Things I checked that are clean

* **Corpus is genuinely 37/37 in STRICT mode**, and `signal_forms.rex` is
  genuinely *compared*, not counted. My first probe was wrong (mutating the
  `.rex` changes both interpreters alike, so it stays matching). The right probe
  is to break the implementation: deleting the `trace_keyword` call in the
  `Signal::Value` arm drops the gate to `36 of 37 matching`, `mismatches (1):
  [UNCLASSIFIED] lang/signal_forms.rex: stderr differ`, `test corpus_differential
  ... FAILED`. Restored afterwards.
* **`Signal::Trap`'s witness row survives and is live.** `loud.rs:258` still
  carries `tag: "Signal::Trap"`, and `every_out_of_scope_variant_fails_loudly`
  (`loud.rs:552`) actually runs it and asserts both `NOT_IMPLEMENTED_EXIT` and
  the exact `" is not implemented (4b)"` suffix. Direct runs of `signal off
  error`, `signal on error` and `signal on error name handler` all give rc 120
  with `rexx-exec: SIGNAL is not implemented (4b)`.
* **The corrected `loud.rs` entry count is right.** `INSTRUCTION_TAGS` has
  exactly **16** coarse `Owner::Phase` instruction tags and 24 `InScope`;
  `INSTRUCTION_WITNESSES` has exactly **17** rows; `14 + 2 + 1 = 17` is
  consistent with both. The historical figures in the corrected comment check
  out against git: `0197b360` (4a gate) 20, `a462e3e9` (Task 1) 19, `733da737`
  (Task 3) 18, `8b87195b` (Task 5) 16. The staleness claim is also true -- the
  line still read "20 entries -- 18 coarse" at Task 5 while
  `assert_witness_set_is_complete`'s copy was correctly at 16/18.
* **Label resolution against the activation's body is genuinely pinned.**
  Replacing `resolve_signal_target` in the `Signal::Label` arm with a direct
  `code.body.labels` lookup kills
  `signal_escapes_an_interpret_fragment_to_reach_an_enclosing_label` and nothing
  else. Restored afterwards.
* **No pre-existing expected-byte literal changed.** `git diff` removes no line
  containing `b"`, `assert_eq` or `.to_vec()` from `run.rs`; the only edit to
  existing test code is `run_activated`'s loop restructure, which is a pure
  wrapper (every non-`Signal` arm still returns on its first firing, so
  behaviour for the ~250 `SIGNAL`-free tests is unchanged). All 36 previously
  listed corpus programs still match.
* **Only `signal_forms.txt` was added under `sourceline_oracle/`** -- no
  existing expectation file changed, consistent with the report's
  regenerate-everything-and-check-idempotent claim.
* **`owners.rs` data is unchanged** (`InstructionKind::Signal(_) => ("Signal",
  Owner::Phase("4b"))`), only a comment added, and the comment is accurate.
* **`corpus/README.md` was correctly skipped** -- `interpret_error_echo`,
  `call_return`, `call_expression`, `call_procedure_expose` and `use_arg_forms`
  are all likewise absent from it, so this matches established practice.
* **Gates**: `cargo fmt --all --check` exit 0; `cargo clippy --workspace
  --all-targets -- -D warnings` exit 0; `cargo test --workspace` exit 0, 68
  `test result: ok` lines, zero failures. All exit statuses read unpiped.
* **`error.rs`'s `label_not_found` doc** is accurate in every particular I
  tested, including the quoted/bare case asymmetry.

## Tree state

`git status --porcelain` empty after every experiment and at the end of the
review. Nothing committed.
