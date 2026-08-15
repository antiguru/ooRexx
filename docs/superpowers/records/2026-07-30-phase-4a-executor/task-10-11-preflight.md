STATUS: DONE

# Pre-flight audit: Task 10 (IF, SELECT, SELECT CASE) and Task 11 (DO, LOOP)

Auditor: read-only review agent, 2026-07-31. Claims checked against the tree
at 43e18462 (plus the live Task 9 fix round, which this audit did not touch)
and against the oracle (every probe under the ulimit, loop probes under
`timeout 10`). The SF #2018 shape (a FALSE `WHEN` whose empty `THEN` absorbed
the next `WHEN`) was not probed, per instruction; the TRUE-condition variant
in Task 10's brief is a different shape, already measured safe (below).

---

## Task 10 brief

### 1. Claims checked, and which are false

- "Modify: run.rs / tests in-file" — **true now** (run.rs exists since Task
  9). Note the Task 9 fix round is live in that file; dispatch order is the
  lead's call, not a brief defect.
- "`when 1 = 1 then` followed by `when 2 = 2 then nop`, where the second
  WHEN is the first's THEN instruction and is never collected into `whens`"
  — **true and already recorded in-tree**: `ast.rs:771-779` documents the
  `whens` field with exactly this measurement (`LanguageParser.cpp:1319`),
  including that the program is *accepted, rc 0*. Re-confirmed at runtime
  this session: prints nothing, runs to completion, rc 0. The brief asks for
  the test but never states the expected outcome; `ast.rs:776` has it.
- "SELECT CASE compares with `==`: `select case '007'` does not match
  `when 7`" — **confirmed** (prints `other`), and `select case 7` matches
  `when 7`. But the claim is incomplete in a way that matters, see item 3.
- "A SELECT that reaches its END with no WHEN taken is 7.3" — **confirmed**,
  and measured precisely (see Measurements): rc 249, and **the echoed clause
  is the `END`, not the `SELECT`**. The brief does not say which clause is
  echoed, and that is the detail most likely to be silently wrong (item 3).

### 2. Steps that violate constraints if followed literally

- The Step-5 commit block: `git add ... rust/crates/rexx-exec/tests/run_select.rs`
  names an integration-test file that contradicts the brief's own Files
  section and the global in-file `#[cfg(test)] mod tests` rule. **Identical
  defect to Task 9's brief**, which the Task 9 dispatch had to correct.
- "Step 2 to 5 as before" delegates to a step structure the extracted brief
  does not contain. Task 9's implementer got the four dispatch corrections
  spelled out; "as before" reaching a fresh implementer carries nothing.

### 3. What the brief needs and does not say

- **IF raises 34.1 and WHEN raises 34.2 for a bad single-expression
  condition, and must NOT do the checking for a comma-list condition, which
  raises 34.6 itself.** The brief never mentions error 34 at all. The rule
  is load-bearing and lives in two places that do not reach a Task 10
  implementer: `task-8-report.md` ("34.1-34.4 fire when the condition is a
  single expression... 34.6 fires when the bad value is one element of a
  multi-element comma list") and `eval.rs`'s `eval_logical_list` doc comment
  ("The four keyword-specific numbers belong to IF/WHEN/WHILE/UNTIL
  themselves (Tasks 9-11), evaluating a bare condition Expr directly and
  checking its own result"). Confirmed against the oracle in the Task 8
  review (34.1 and 34.2 measured directly). A `WhenCase` value, by contrast,
  is compared, never logical-checked, so it raises nothing of the sort.
- **7.3 must be raised with the `END` as the failing clause** (measured, both
  for plain SELECT and SELECT CASE). If the implementer raises it from the
  `Select` arm's own execution, `failure_site` will record the `select`
  clause and stdout/rc will still match, so **only the stderr echo can catch
  it** — the same trap the lead flagged for `DO UNTIL`. Natural
  implementation: the raise happens when control reaches the `END` (or the
  `When`'s `false_target` chain runs out), which also matches the AST's
  jump-target design.
- **`WhenCase` is a value LIST, an OR of `==` comparisons, and the comma
  means the opposite of what it means in a plain WHEN.** `ast.rs:801-815`:
  `select case 2; when 1, 2 then say 'hit'` prints `hit` (re-confirmed this
  session), while plain `when 1, 2` is 34.6. A test suite that only uses
  single-value CASE whens cannot tell `parseCaseWhenList` semantics from
  `parseLogical` semantics. Also `values` is "at least one, none may be
  omitted (35.934)".
- The runtime semantics of `Then`, `Else { then_exit }`, `Otherwise`, and
  `End` as instructions `step` must now handle (all currently fall to
  `Loud::instruction`), and `If { false_target }` / `When { false_target,
  exit }` jump-target meanings: all documented in `ast.rs:741-830`, none
  mentioned in the brief. An implementer who reads `ast.rs` is fine; the
  brief should say to.

---

## Task 11 brief

### 1. Claims checked, and which are false

- **The Files section is wrong: the depth limit cannot land in `run.rs`.**
  `eval.rs:70-71` (the `eval` wrapper's doc comment): "Task 11 adds the
  limit check to this function, which is why it is the one that owns the
  depth bookkeeping." So Task 11 must modify `eval.rs`, which the Files
  section does not list. The 11.1 raise also implies extending the measured-
  family list in `error.rs`'s catalogue-coverage test (`error.rs`, also
  unlisted) if the project keeps that test exhaustive. Same defect class as
  Task 9's stale signature, but this one would surface as a scope violation
  mid-task.
- **Every depth figure in the brief is stale, by the brief's own rule and by
  two tree moves since.** The ~1600-bytes/level figure is tied to
  "`eval_node` after Task 7 grew to fifteen match arms"; Task 8 added three
  more dispatch arms plus `eval_compare`/`eval_logical`/`eval_logical_list`
  frames, and Task 9 moved the instruction loop into `run.rs`, changing the
  stack mix above `eval`. The brief already says "treat the rule as the
  deliverable and the number as perishable" and "do not quote a
  predecessor's number" — that instruction is correct and everything
  numeric around it (1600, 335,000, 6.5x) should be re-measured at
  implementation. The 100,000 limit and the fire-above-not-at rule are
  policy, not measurements, and stand.
- The Step-2 error-family table: **every row re-confirmed against the oracle
  this session** (third independent account, agreeing with the brief; see
  Measurements). The brief's instruction to re-run it remains good practice.
- "`Over` on a non-stem target: a string and a number each iterate once,
  yielding themselves" — **confirmed** (`abc`, `42`).
- "`LoopKind::With` is Phase 5's" — consistent with the tree
  (`ast.rs` `With` variant exists; nothing in 4a answers `SUPPLIER`).
- "Control expressions are evaluated in `Controlled::order`" — **true**,
  `ast.rs`'s `Controlled { order: Vec<ControlExpr> }` exists with exactly
  that doc rationale.
- 11.1 exists in the catalogue: "Insufficient control stack space; cannot
  continue execution." (rexxmsg message 11, subcode 001), so
  `Raised::syntax(11, 1, ...)` will render real text.

### 2. Steps that violate constraints if followed literally

- Step-5 commit block: `git add ... tests/run_loops.rs` — same forbidden
  integration-test file shape as Tasks 9 and 10.

### 3. What the brief needs and does not say

- **`WHILE` and `UNTIL` are absent from the brief's own test list.** Step 1
  enumerates `Simple`/`Forever`/`Count`/`Controlled`/`Over` and never
  mentions `Loop.conditional` (`ast.rs`: `Option<LoopConditional>`, never
  both). With them come three facts the brief also omits:
  - 34.3 (WHILE) / 34.4 (UNTIL) for a bad single-expression condition, with
    the same division of labour against the comma list's 34.6 as Task 10's
    (source: `task-8-report.md`, `eval_logical_list`'s doc comment; both
    measured, including comma lists under WHILE/UNTIL giving 34.6, in the
    Task 8 review).
  - **A failing UNTIL condition echoes the `END` clause; a failing WHILE
    echoes the `DO` clause** (the lead's measurement, re-confirmed this
    session with exact stderr). Right answers with a wrong echo are exactly
    what stdout/rc differential testing cannot see.
  - The echoed clause for loop-condition failures is **indented two extra
    spaces** in the oracle's report (see Measurements). Nothing in the
    current Rust reporting path produces indentation, and none of Task 12's
    measured cases had any. This will break byte-for-byte stderr agreement
    the first time a loop-condition failure is diffed.
- **The LEAVE/ITERATE name rule is not what classic-Rexx priors suggest, and
  nothing in the briefs or tree states it.** Measured this session:
  - An ordinary clause label does NOT name a loop: `outer: do i = ...` with
    `leave outer` from inside is **28.3**, rc 228 (and `iterate outer` is
    28.4). The names that work are `DO LABEL name` (28.3's own message:
    "label of a current loop or block instruction") and the control
    variable's name, which becomes the label automatically
    (`ast.rs`'s `Loop.label` doc records the auto-label rule).
  - `leave i` from a nested inner loop leaves the **outer** loop, unwinding
    the inner one; `iterate i` from the inner loop iterates the outer
    (`1 1` / `2 1` / `done`).
  - Bare `leave` inside a Simple (non-loop) `do` block is **28.1**; but a
    *labeled* simple block (`do label b ... leave b`) is leavable, rc 0.
  - `leave sel` naming a `select label sel` exits the SELECT (not the
    enclosing loop), so **SELECT belongs on the leavable-block stack** —
    see section 4.
  - Bare `leave` inside a SELECT nested in a loop leaves the loop (the
    brief's own test item, confirmed).
- **`DO ... OVER` a stem works on the oracle and the brief is silent on
  stems entirely.** `a.1='x'; a.2='y'; do i over a.` iterates the tail keys
  — and printed them in **hash order (`2` then `1`), not insertion order**.
  Ten of the seventeen blocked corpus programs are DO-blocked; if any uses
  `DO OVER` on a stem, iteration *order* becomes observable and the oracle's
  order is an implementation artifact of its hash table. This needs a scope
  decision (in or out for 4a?) and, if in, an ordering decision, before an
  implementer invents one silently.
- `DO COUNTER name` (`Loop.counter`, in the AST) and `OVER ... FOR n`
  (`Over.for_count`) both exist in the parse and appear nowhere in the
  brief's test list.
- Bonus row for the Step-2 table, measured: `do i = 1.5 to 3` iterates
  twice printing `1.5` and `2.5` (the brief's "no error" row, now with the
  observable outputs), and `do i = 1 by 0 to 3` starts iterating normally
  (probed with `leave` after the first pass).

### 4. Where the two briefs contradict or fail to coordinate

- **The block stack.** Task 11's commit message claims "the block stack
  LEAVE unwinds", implying Task 11 builds it. But the measured LEAVE rules
  reach Task 10's constructs: `leave sel` must find a SELECT by label, and
  bare LEAVE must unwind *through* SELECT contexts to the innermost loop.
  If Task 10 implements SELECT purely as jump targets with no runtime block
  bookkeeping (which suffices for Task 10's own tests), Task 11 must
  retrofit block entries onto Task 10's freshly written arms — in the same
  file, immediately after. Neither brief mentions the other's stake. Worth
  a line in whichever dispatch goes first.
- Both briefs' Step-5 commit blocks carry the same forbidden-file defect;
  not a contradiction, but the same correction is needed twice.
- No genuine conflict found in what they assert about the tree: Task 10
  needs nothing from `eval.rs`, so Task 11's unlisted `eval.rs` edit does
  not collide with Task 10's scope.

---

## Measurements (oracle, this session)

**SELECT 7.3, exact report** (`select` / `when 0 then say 'a'` / `end`),
rc 249, echoing the END on its own line number:

```
     3 *-* end
Error 7 running .../p1.rexx line 3:  WHEN or OTHERWISE expected.
Error 7.3:  All WHEN expressions of SELECT are false; OTHERWISE expected.
```

SELECT CASE with no match and no OTHERWISE gives the identical 7.3 shape,
also echoing the `end`. The message text matches rexxmsg.xml, so
`Raised::report` will reproduce lines 2-3 verbatim once the raise carries
(7,3) and no substitutions; line 1 is `failure_site`'s to get right (the
END's line and text, NOT the select's).

**DO WHILE vs DO UNTIL failing-condition echo** (the lead's measurement,
confirmed, with one addition — note the two extra spaces before the clause
text, absent from every non-loop error echo measured to date):

```
do until 'x' / end        ->      2 *-*   end            (34.4, rc 222)
do while 'x' / end        ->      1 *-*   do while 'x'   (34.3, rc 222)
```

The indentation is real and inconsistent across error classes: loop-
condition failures indent two spaces; 7.3, LEAVE/ITERATE errors (28.x), and
first-entry control errors (41.1/26.2/26.3, below) do not; a bare `leave`
failing inside a simple `do` block indented two. I could not derive the
full rule from these probes and did not try further — **it needs
characterizing during Task 10/11 implementation, because stderr diffs will
enforce whatever it is.** Raw transcripts are in the scratchpad.

**LEAVE/ITERATE**: full transcript set summarized in Task 11 item 3 above
(28.1, 28.3, 28.4 texts captured; `DO LABEL`, control-variable auto-label,
cross-loop leave/iterate, labeled block, labeled SELECT, bare leave through
SELECT-in-loop).

**The DO control error family, third independent account, agreeing with the
brief's table**: `do i='a' to 3`, `do i=1 to 'x'`, `do i=1 by 'x'` are all
41.1 (rc 215, echoing the `do` clause, unindented); `for 'x'`/`for -1`/
`for 1.5` are 26.3 with the value as substitution (rc 230); `do 'a'`/
`do -1`/`do 2.5` are 26.2 (rc 230); `do i = 1.5 to 3` runs (1.5, 2.5);
`do i = 1 by 0 to 3` loops without error.

**DO OVER**: `'abc'` -> `abc` once; `42` -> `42` once; a stem -> its tail
keys in hash order (measured `2` then `1` against insertion order 1, 2).

**The absorbed-WHEN program** (`when 1 = 1 then` / `when 2 = 2 then nop`):
runs clean, rc 0, matching `ast.rs:776`. The SF #2018 false-WHEN shape was
not approached.

## Not reached

- The depth cliff and bytes-per-level: deliberately not re-measured (the
  brief itself mandates re-measuring at implementation; a stale audit
  number would just be a fifth entry in the graveyard).
- The full indentation rule for error-report clause echoes (observed,
  bounded, not characterized).
- `DO OVER` stem ordering beyond one two-tail probe (enough to prove
  hash-order, not to model it).
- SF #2018 neighborhood, per instruction.
- `SELECT CASE` with expressions (not literals) as WHEN values, `DO WITH`
  (Phase 5), `DO COUNTER` semantics beyond noting the brief omits it.
