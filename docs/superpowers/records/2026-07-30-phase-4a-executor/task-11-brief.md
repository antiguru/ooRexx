### Task 11: `DO` and `LOOP` in every variant

**Spec:** "Control flow", D19's depth policy.

**Files:**
- Modify: `rust/crates/rexx-exec/src/run.rs`
- Modify: `rust/crates/rexx-exec/src/eval.rs` — **the evaluation-depth counter belongs here, not in `run.rs`.** `eval.rs:70-71` already says "Task 11 adds the limit check to this function", and the recursion the limit guards is `eval`'s.
- Modify: `rust/crates/rexx-exec/src/error.rs` — the 11.1 catalogue-family test lives beside the other catalogue tests. 11.1 is present in the generated catalogue as "Insufficient control stack space".
- Test: a `#[cfg(test)] mod tests` inside each file it touches

- [ ] **Step 1: Write the failing tests** — `Simple`, `Forever`, `Count`, `Controlled` with every combination of `TO`/`BY`/`FOR`, and `Over` on a **non-stem** target (measured: a string and a number each iterate once, yielding themselves). `LoopKind::With` is Phase 5's and takes the loud-failure path, because `DO WITH` sends `SUPPLIER` and nothing in 4a answers a message.

**`DO OVER` on a stem is out of scope and no test may use it.** That is not a judgement to re-make here: it is DEVIATION 1 in `phase-4-exclusions.txt`, which states the rule as "no corpus program may contain `DO OVER` on a stem", because the oracle walks a balanced tree and we use a hash map, so the orders are two different deterministic orders. `DO OVER` on a stem *does* work on the oracle and iterates in hash order, so it is easy to write a test for by accident.

**`COUNTER` and `OVER ... FOR` are in the AST and absent from this list.** Decide each explicitly, implement or take the loud path, and say which in the report. Do not leave them to fall through a catch-all.

**`WHILE` and `UNTIL` are missing from this list and are yours.** With them come **34.3** (`WHILE`) and **34.4** (`UNTIL`) for a single-expression condition that is not `0` or `1`. As with `IF` and `WHEN`, do not check a comma-list condition yourself: it raises **34.6** from inside `eval_logical_list`.

**Which clause the report echoes distinguishes a correct loop from one that evaluates its condition in the wrong place, and nothing else does.** Measured: `do until 'x'; end` echoes **`end`**, because `UNTIL` is tested at the bottom, while `do while 'x'; end` echoes the **`do`** line. A loop that evaluates `UNTIL` eagerly still produces the right values and the right exit code, and only the echoed clause reveals it.

Control expressions are evaluated in `Controlled::order`, which Phase 3 recorded because an expression can have side effects.

`LEAVE` and `ITERATE`, bare and by label, including from inside a `SELECT` nested in a loop. **Measured, and it is not what classic-Rexx intuition suggests:**

* An **ordinary clause label does not name a loop.** `outer: do i = 1 to 3` followed by `leave outer` is **28.3**, and `iterate outer` is **28.4**.
* What does work is `DO LABEL name`, and the **control variable's automatic label**: `leave i` or `iterate i` from inside a nested loop reaches the *outer* loop and unwinds the inner one.
* Bare `leave` in a simple `DO` block is **28.1**, but a **labelled** simple block is leavable.
* `leave sel` exits a `SELECT` **only when that `SELECT` was written `SELECT LABEL sel`**. An ordinary clause label in front of a `SELECT` does not make it leavable, exactly as it does not for a loop: that form is 28.3. The distinction matters to you specifically, because you are designing the block stack and the question is which constructs put a *named* frame on it.

**That last one is a coordination point with Task 10, and Task 10 has since shipped, so here is what it actually left you.** `LEAVE` must find and unwind through Task 10's `SELECT`s by label. Read the committed `run.rs` rather than reasoning from this plan; the three facts below were established after this section was first written, and two of them were mutation-verified.

* **`run_bounded` forwards a `Flow` it does not own, outward and unchanged** (`other => return Ok(other)`, and both the `If` and `Select` callers forward untouched). That is settled, not a contingency: a mutation that swallowed an unowned variant was applied and killed. So a `Flow` variant you add for `LEAVE`/`ITERATE` will propagate out of a nested `IF`/`SELECT` without either arm interfering.
* **But `run_bounded` absorbs any `Goto` whose target lands INSIDE its own range**, forwards or backwards, and does not tell the arm that produced it. Its doc guarantees the propagation case, which is the case that does not bite. The one that bites is yours: a `DO` inside an `IF`'s `THEN`, with an `ITERATE` in the loop body, computes a jump to the loop top, which is inside the `IF`'s range, so the `IF`'s `run_bounded` takes it directly and your `Do` arm re-enters as a **first entry** with its counter reset. That is exactly criterion 6's "`LEAVE` unwinds one block too few" mutation arriving by a route no document mentions. **So the invariant you must hold is: every block-stack change a `LEAVE`/`ITERATE` implies must be complete before the `Goto` is returned**, because you cannot rely on seeing that `Goto` again. State in your report which of your tests pins it.
* **`error.rs` is in your Files list for two reasons, not one.** The 11.1 catalogue entry is the obvious one. The other is `Raised::report`, which is where the clause-echo indentation belongs, and which nothing else points you at.

**The clause-echo indentation is yours, and it was recorded in four places with four different owners before it was recorded here.** Both other named owners are closed tasks, so this is the only statement of it that will reach an implementer. What the oracle does, measured, and re-verified by a reviewer including the two rows that were originally inferred:

* Two spaces per open block frame. One enclosing `DO` gives two, two gives four, three gives six.
* An `IF`'s `THEN` counts as **two** frames: `if 1=1 then say 2 & 1` indents four.
* A `SELECT` adds one more, so a `say` inside a true `WHEN`'s `THEN` indents six. `SELECT CASE`'s `THEN` is also six, and an `ELSE IF` chain is eight.
* `do i = 1 to 'x'` gets **none**, because its control expression is evaluated before the block is entered.

`Raised::report` emits no indentation today, so every corpus program that raises inside a block currently differs from the oracle on stderr, and that is the largest single source of remaining divergence. Two warnings. **The quantity is shared with `TRACE`'s own `*-*` indentation (Task 13), so put it somewhere Task 13 can use rather than inline in the report formatter** -- one quantity, two formatters. And Task 10's report concludes the depth is **derivable from the AST statically, with no runtime block stack**; every probe behind the measured rule was a raising `say`, and the claim that `TRACE` indents identically is asserted rather than measured, so **measure the trace half yourself before sharing the mechanism**.

**Two more things this section did not say.** `MAX_EXPR_DEPTH` in `rexx-parse` is 50,000 and raises the **same** 11.1 from the parser, so a depth test built from nested parentheses goes green without `eval`'s counter ever firing: build the depth test from a left-deep operator chain, not from parentheses. And `End`/`EndStyle` for `DO`/`LOOP` are yours and appear nowhere in the steps below.

- [ ] **Step 2: Measure the `DO` control error family before implementing it**

Measured, and note the paraphrase an earlier draft used was wrong in a way that would have pointed your probes at the one case that never raises:

| clause | oracle |
|---|---|
| `do i = 'a' to 3`, `do i = 1 to 'x'`, `do i = 1 by 'x'` | 41.1 |
| `do i = 1 to 3 for 'x'`, `for -1`, `for 1.5` | **26.3**, the `FOR` count |
| `do 'a'`, `do -1`, `do 2.5` | **26.2**, the `DO` repetitor |
| `do i = 1.5 to 3` | **no error** — a non-whole *control* value is legal |
| `do i = 1 by 0 to 3` | **no error**, loops forever — behaviour to reproduce, not an error to catalogue |

Re-run each row yourself and put the table in the report; two accounts of this family have already disagreed.

- [ ] **Step 3: Implement**, including the depth counter D19 requires.

Its limit is bounded on **both** sides: at least 100,000, the oracle's largest measured passing depth, and below what Task 3's stack size and per-frame cost allow. An upper bound alone is satisfied by a limit of 20,000, which diverges on every program between there and 100,000.

**Task 3 measured these and the answer is not the one this plan first recorded, so inherit the corrected numbers.** The interpreter thread is **512 MiB**. **`eval` binds the stack, and the figure moves whenever `eval`'s shape does — treat the rule as the deliverable and the number as perishable.** After Task 7 grew `eval_node` to fifteen match arms it went from ~783 to **~1600 bytes per level in debug**, roughly 335,000 survivable levels, still clearing D19's 100,000 minimum by more than three times. That is the **fourth** value this figure has taken in two days: ~820, then ~850, then ~783 once three tree walks became iterative, now ~1600. Every one was correct for the code it measured. **Re-measure at implementation and do not quote a predecessor's number.**

The historical figure, superseded: ~783 bytes per level with roughly 685,000 levels in debug. Measured on the current tree and independently confirmed: 600,000 and 684,000 levels pass, 700,000 aborts at rc 134.

| what runs | deepest surviving | bytes/level |
|---|---|---|
| parse and drop only | no cliff below 4,000,000 | under 134 |
| parse, plan and drop | 3,354,442 | ~160 |
| all four, including `eval` | 684,618 | **~783** |

`Plan::note` is now the crate's own remaining recursion at ~160 bytes per level, and is a candidate for the same worklist treatment if anything ever needs it.

**This number has moved twice in a day, and the reason is worth more than the number.** It was first recorded as ~820, then corrected to ~850 by a finer bisection, and both are now void — because Task 3b made three tree walks iterative and thereby *removed the recursions those measurements were measuring*. A measurement of the code is only valid for the code it measured. **Re-run the bisection when you implement this**, and treat the table above as the shape of the answer rather than the answer.

Two lessons recorded alongside it. The ~820 figure was wrong in a way visible without re-measuring: its table had parse-and-drop costing *less* per level than parse-plan-and-drop, so adding a phase appeared to make each level cheaper, which no model of sequential phases produces — incoherence in a table is a finding even before you check the arithmetic. And `eval` binding again at ~783 is now the strongest of the three figures, because an independent in-`eval` probe reported 784 and the external bisection agrees to 0.2 per cent, which is two methods rather than one.

**The recommendation of 100,000 is unchanged; only its justification moved**, and the headroom is now about 6.5x rather than 6x.

**Set the limit at exactly 100,000, raising 11.1 for anything deeper**, and mind the off-by-one: a 100,000-term expression *reaches* depth 100,000, so the check must fire **above** 100,000 rather than at it, or the one depth the oracle is known to survive becomes the first one we refuse. Higher limits reproduce nothing that exists and only widen the window where we succeed while the oracle SIGSEGVs.

**And know what this limit cannot do.** A counter in `eval` does not close the abort path, because a program can reach a deep tree without evaluating it at all: `exit` followed by a 700,000-term expression aborts inside the `Drop`, with nothing evaluated and no counter in `rexx-exec` in a position to see it. That path is closed by Task 3b's iterative `Drop`, not here. Do not write a doc comment claiming this limit makes deep expressions safe.

- [ ] **Step 4: Verify** — plus an expression at a depth the oracle handles comfortably, and a **test that reaches the 11.1 raise**, since no differential program can cross our limit without also crossing the oracle's cliff, and without that test the depth path is untested by construction.

> **That test cannot be a plain `#[cfg(test)]` unit test, and writing one is the trap this step used to set.** The only sized stack in the workspace is inside `run_program` (`lib.rs`), the public entry point; a libtest thread gets 2 MiB by default. At roughly the per-level cost quoted above, `eval` dies natively somewhere around a thousand levels, far below the limit you are trying to trigger, and it dies as a **guard-page abort with no message**, which is precisely the silent death D19's limit exists to prevent. Worse, the cheapest-looking way to make a failing test pass is to lower the limit, which would quietly defeat the whole decision.
>
> So drive it through `run_program`, which is what puts you on the sized thread. `tests/spike.rs` and `tests/corpus.rs` both already do this. If that means this one test is an integration test while the rest of your work is unit-tested in place, that is correct and not a constraint violation: the constraint forbids integration-testing a **private** subject, and `run_program` is public cross-crate surface. Say in your report how you confirmed the test reaches your counter rather than the parser's 50,000 or the guard page.

The oracle's cliff is between **100,000 and 150,000** terms, not at 200,000: 100,000 prints its answer, and both 150,000 and 200,000 exit 139. A corpus rule phrased against 200,000 would admit a 150,000-term program that SIGSEGVs.

- [ ] **Step 5: Commit**

```bash
git add rust/crates/rexx-exec/src/run.rs rust/crates/rexx-exec/src/eval.rs rust/crates/rexx-exec/src/error.rs
git commit -F <message-file>
```

---

