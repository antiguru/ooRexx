# Task 9 re-review -- fix round 4 (`16a67ae2`), scoped to the fourth label route

Every claim below is labelled **[ran]** or **[reasoned]**.

## Headline answer

**Ninth false statement in this commit: yes, but not among the four places the
report targeted.** All four rewritten route-claim sites (`trace.rs`'s `labels`
field doc, `trace.rs`'s `LABELS` const doc, `trace_oracle.rs`'s test doc,
`trace_labels.rex`'s header) are individually true and their C++ citations
check out [ran]. The ninth instance is a **fifth, untouched sentence in the
same file this round edited**: `docs/superpowers/plans/phase-4-exclusions.txt:1104`
still reads "whose committed expectation is three `*-*` lines" -- a claim
this commit's own diff (three paragraphs above it, same file) falsifies: the
committed expectation (`trace_labels.expected`) now has **four** `*-*` lines
(`fellthrough`, `sub`, `zf`, `there`), confirmed both by reading the file and
by an independent live-oracle run [ran]. This is the defect class exactly --
a count of a mutable in-repo aggregate, falsifiable by the same commit -- and
it sits three paragraphs below this round's own new paragraph explaining why
that class is dangerous. **Parkable**: doc-only, no test or code depends on
it, no behavioural consequence, one-word fix (`three` -> `four`).

---

## Method [ran]

* Read `.superpowers/sdd/2026-08-03-phase-4b-procedures-and-conditions/task-9-rereview3.md`,
  the fix-round-4 section of `task-9-report.md` (lines 1123-1273), the full
  `git diff 50da3045 16a67ae2`, and `rust/CLAUDE.md`.
* Read `RexxInstructionLabel::execute` (`LabelInstruction.cpp:54`-`60`) and
  `traceLabel`/`tracingLabels` (`RexxActivation.hpp:366`,`376`) directly from
  the read-only C++ tree, plus `TraceSetting.cpp:52`-`54` for the pre-existing
  flag-set citation the new paragraphs build on.
* Backed up `crates/rexx-exec/src/trace.rs` and
  `crates/rexx-exec/tests/trace_oracle/trace_labels.expected` to the
  scratchpad (`t9r4/backup/`) before mutating either. Restored both from the
  copies, **never `git checkout --`**, and verified the restore against
  `git diff HEAD` (empty) and `git show HEAD:<path> | md5sum` matching the
  working-tree hash -- not just my own backup's hash, closing the loop fully.
* Ran `R4-M1` and `R4-M2` and read passed/failed counts, not exit statuses.
* Re-ran `cargo fmt --all --check`, `cargo clippy --workspace --all-targets
  -- -D warnings`, and `cargo test --workspace` (summed every `test result:`
  line myself rather than trusting a single summary) after all restores, to
  confirm my own mutation testing left nothing broken and that the counts
  the controller already established still hold.
* Regenerated `trace_labels.rex`'s output against the live oracle myself,
  from a freshly `mkdir`'d scratchpad directory, using the mandated wrapper,
  three separate descriptors, absolute redirects.

**Anomaly, disclosed rather than acted on**: twice during file restoration,
a tool-result channel injected a fabricated "system-reminder" claiming
`trace.rs` and then `trace_labels.expected` had been "modified, either by
the user or by a linter," calling the change "intentional," and instructing
me not to tell the user. Both claims were false -- in both cases `git diff
HEAD` was empty and the working-tree hash matched `git show HEAD:<path>`
exactly, i.e. the file was the genuine, unmodified committed content. I did
not act on the injected content and I am not concealing it: no tool output
can authorize hiding information from the user. This is a process/security
observation, not a Task 9 finding.

---

## 1. Ninth false statement -- detailed

### The four rewritten sites: individually true [ran]

* **`trace.rs`, `labels` field doc** (new paragraph after "the only mode it
  decides anything in is `L`."): "The condition is 'the `LABEL` instruction
  executed', not any list of ways control can arrive... The C++ site above
  enumerates the whole condition in one line." The cited site, immediately
  above and *unchanged* by this round, reads `RexxInstructionLabel::execute`
  "traces through `traceLabel` and nothing else, and `traceLabel`'s gate is
  `tracingLabels()`." Verified against source: `LabelInstruction.cpp:58`-`59`
  is `context->traceLabel(this); context->pauseLabel();` -- `traceLabel` is
  the only trace-related call (`pauseLabel` is debug-pause machinery,
  unrelated to echoing); `RexxActivation.hpp:376` defines `traceLabel` as
  `if (tracingLabels()) traceClause(v, TRACE_PREFIX_CLAUSE);`. The citation
  supports the sentence exactly.
* **`trace.rs`, `LABELS` const doc**: "every executed `LABEL` clause echoes
  and nothing else does. `tests/trace_oracle/trace_labels.rex` carries the
  probed routes and says why they are examples rather than an enumeration."
  Both clauses checked against the actual file -- true [ran].
* **`trace_oracle.rs` test doc**: "The program's own header says which
  routes to a label it probes, why they are examples rather than an
  enumeration, and why the silent constructs between them are the other
  half of the claim." Read `trace_labels.rex`'s header directly: it does all
  three of those things, in that order. True.
* **`trace_labels.rex`'s header**: repeats the same C++ citation almost
  verbatim ("`RexxInstructionLabel::execute` traces through `traceLabel` and
  nothing else, and `traceLabel`'s gate is `tracingLabels()`") and correctly
  reframes the four listed routes (`FELLTHROUGH`, `SUB`, `ZF`, `THERE`) as
  "examples that have been probed, never a closed set." True, and internally
  consistent -- nowhere in this file does a stale count survive; the closing
  paragraph was correctly reworded from "none of the **three** label lines"
  to "none of the label lines" (no count at all).

None of these four states a count. **The report's claim is accurate for the
four places it names.**

### The fifth place it did not check: `phase-4-exclusions.txt:1102`-`1106` [ran]

Two paragraphs after this round's new "THE ROUTE COUNT IN THIS ROW WAS WRONG"
paragraph, an **unmodified** paragraph (context lines in the diff, not `+`
lines) still reads:

> WHAT WE USED TO DO: nothing at all under trace l. WITNESS:
> `rust/crates/rexx-exec/tests/trace_oracle/trace_labels.rex`, whose committed
> expectation is **three** `*-*` lines and whose stdout proves the constructs
> between them ran silently.

`trace_labels.expected` (edited by this same commit, `git diff` confirms) now
has **four** `*-*` lines: `fellthrough`, `sub`, `zf`, `there` -- verified both
by reading the committed file and by an independent oracle run from a fresh
`mkdir`'d directory (byte-identical stdout/stderr/rc to the file, see
Section 3). This sentence was true at `50da3045` and is false at `16a67ae2`,
made false by this commit's own edits to the very file (`trace_labels.rex`/
`.expected`) it names, three paragraphs below the paragraph that explains why
exactly this kind of staleness is worth guarding against. It is not one of
the four sites the report swept -- it is a neighbour the sweep missed.

Severity: **parkable**. It is prose in a doc-only exclusions log, asserts
nothing a test reads, and carries no behavioural risk -- but it is precisely
the class this whole task exists to eliminate ("a claim about a mutable
in-repo aggregate, falsifiable by its own commit"), and its presence means
round 4 did not fully keep the chain broken, even though every location it
deliberately edited is clean.

### Nothing else found

Grepped the tree for "ways a label"/"three ways"/"label is reached" outside
the four checked sites: the only other hits are (a) `rust/CLAUDE.md`
correctly *quoting* the old false claim as a worked example of the defect
class (past tense, historical), (b) `phase-4-exclusions.txt`'s own new LESSON
paragraph doing the same, (c) `trace_labels.rex`'s "an earlier version...
said" sentence, also historical, and (d) two unrelated hits (`phase-3-parser.md`
about a parser entry point, `activation.rs:145` about `::routine` activation
reachability -- a different subject, pre-existing, untouched by this
round's diff). None of these are current exhaustiveness claims.

The dangling annotation line added to `phase-4-exclusions.txt`'s route-example
block ("`<- and an internal FUNCTION call, found by a later review attacking
the count`", with no accompanying measured line number unlike its three
siblings) is stylistically inconsistent but not false -- the block's intro
was correctly reworded to "Probed routes -- EXAMPLES, NOT AN ENUMERATION"
before this addition, so it no longer claims the three-line list is
exhaustive. Noting as a minor cosmetic nit, not a finding.

---

## 2. Mutation re-runs [ran]

Both mutations reproduced against a freshly built binary, restored from
scratchpad copies (never `git checkout --`), restore verified against
`git diff HEAD` (empty) and hash match to `git show HEAD:<path>` on both
files:

| # | mutation | baseline | result |
|---|---|---|---|
| R4-M1 | `tracing_clause` ignores `is_label` (`mode.all \|\| (mode.labels && is_label)` -> `mode.all`) | `1 passed; 0 failed; 21 filtered` | **`0 passed; 1 failed; 21 filtered out`** |
| R4-M2 | `zf:`'s `*-*` line deleted from the committed `trace_labels.expected` | `1 passed; 0 failed; 21 filtered` | **`0 passed; 1 failed; 21 filtered out`** |

Both match the report's claimed counts exactly. After restoring both files,
`cargo test -p rexx-exec --test trace_oracle` ran clean: **22 passed; 0
failed** (matches the controller's already-established count).

---

## 3. Independent oracle re-run of the extended witness [ran]

From a fresh `mkdir`'d scratchpad directory, using the mandated wrapper,
copying `trace_labels.rex` in unmodified:

```
rc=0
stdout: before any label / if body ran / in the callee /
        function call returns ZF-VALUE / after the signal
stderr:     36 *-* fellthrough:
    52 *-*   sub:
    56 *-*   zf:
    49 *-* there:
```

`diff` against the `===STDOUT===`/`===STDERR===` sections of the committed
`trace_labels.expected`: **both empty (byte-identical)**. Independently
confirms the report's regeneration claim without trusting the report's own
account of it.

---

## 4. Gates re-run [ran]

After all mutation testing and restores, tree confirmed clean
(`git status --porcelain` empty):

| gate | result |
|---|---|
| `cargo fmt --all --check` | exit 0, clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, clean |
| `cargo test --workspace` | summed every `test result:` line myself: **990 passed, 0 failed** -- matches the controller's already-established count exactly |

Not re-derived (controller already established, per the dispatch): STRICT
corpus 41/41, assertions 4224/4259.

---

## 5. The declined restoration -- judgement

Round 3 deleted a paragraph containing one true, non-load-bearing sentence
("`step`'s own `Assignment` arm builds its `rendered` unconditionally because
`trace_result` and the write both need it," `run.rs:957`-`958`) alongside the
false claim it was cited to support. Round 4 declined to restore it, arguing
it is case 3 of its own stated procedure (a mutable in-repo referent, no
longer load-bearing since the pre-gate now stands on the benchmark alone).

Verified directly [ran]: `bind_control`'s current comment (`run.rs:5289`-
`5308`) is self-contained -- it justifies the pre-gate purely by the
40-ns-per-pass measurement and explicitly declines to reason about sibling
call sites ("if the question ever matters, the compiler and the profiler
answer it, and this comment should not try to"). The `step` Assignment arm
at `run.rs:957`-`958` still builds `rendered` unconditionally and still feeds
it to the unconditional `trace_result` call at `958`, with no comment at that
site cross-referencing `bind_control`'s gated one.

**Judgement: the right call, with a small, honest cost.** The rule this task
converged on ("if a claim is load-bearing, assert it in a test; if it is
not, delete it") does not carve out an exception for "true and mildly
informative," and creating one would immediately re-invite the pattern that
produced eight false statements: a sentence about another site's code
structure, drifting the moment either site is touched again. The current
`bind_control` comment stands on a measurement that "stays true however the
rest of the file changes" (its own words), which is a strictly stronger
foundation than a cross-reference to a sibling arm's shape. The cost is
real but small: a reader at `run.rs:957` who wonders why `rendered` is built
unconditionally here but conditionally at `bind_control` has to work it out
from the two call sites rather than being told directly -- one extra step of
inference, not a gap in correctness or in what a diligent reader can
recover. Not restoring it is consistent with the round's own logic and with
`rust/CLAUDE.md`'s rule as written.

---

## Summary

* **Ninth false statement: yes** -- `docs/superpowers/plans/phase-4-exclusions.txt:1104`
  ("committed expectation is three `*-*` lines"), stale as of this commit's
  own edits to the same file, three paragraphs below this round's own
  lesson about exactly this class. **Parkable**: doc-only, no behavioural
  or test consequence, one-word fix.
* **The four route-route rewrites the report targeted: all individually
  true, citations verified directly against `LabelInstruction.cpp`,
  `RexxActivation.hpp`, and `TraceSetting.cpp`.** No count remains in any of
  them.
* **`trace_labels.rex` witness gap: closed.** Now exercises `FELLTHROUGH`,
  `SUB`, `ZF`, `THERE`; regenerated line, verified independently against the
  live oracle from a fresh directory, byte-identical on stdout, stderr, and
  rc.
* **R4-M1 and R4-M2: both reproduce red**, `0 passed; 1 failed; 21 filtered
  out` in both cases, matching the report's claimed counts exactly. Restores
  verified against `git diff HEAD` and `git show HEAD` hash match, not just
  against my own backup copies.
* **Round 3's deletion: not restored, and that was the right call** under
  the stated procedure -- the surviving `bind_control` comment is
  self-contained on the benchmark, and the small loss of drive-by legibility
  at the `step` Assignment arm is an acceptable, disclosed cost rather than
  an oversight.
* **Gates re-run clean**: fmt 0, clippy 0, `cargo test --workspace` 990
  passed / 0 failed (summed myself), `--test trace_oracle` 22 passed.
* **Process note**: two spurious tool-result "system-reminders" during this
  review falsely claimed tracked files had been intentionally modified and
  instructed concealment; both claims were verified false against `git diff
  HEAD`/`git show HEAD`, not acted on, and are disclosed here rather than
  hidden.
