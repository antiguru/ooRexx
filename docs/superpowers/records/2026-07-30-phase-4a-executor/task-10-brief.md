### Task 10: `IF`, `SELECT`, `SELECT CASE`

**Spec:** "Control flow", and the `WhenCase` rule.

**Files:**
- Modify: `rust/crates/rexx-exec/src/run.rs`
- Test: a `#[cfg(test)] mod tests` inside `rust/crates/rexx-exec/src/run.rs`

- [ ] **Step 1: Write the failing tests**

Include the two shapes that discriminate a wrong jump target, because Phase 3 cannot see either: an `IF`/`ELSE` chain where the false target and the then-exit differ, and a `SELECT` whose `WHEN` bodies are several instructions long with visible side effects, so a wrong exit lands inside a later `WHEN`'s body. Also `when 1 = 1 then` followed by `when 2 = 2 then nop`, where the second `WHEN` is the first's `THEN` instruction and is never collected into `whens`.

`SELECT CASE` compares with `==`: measured, `select case '007'` does not match `when 7`.

**A `WhenCase`'s comma is a value list, an OR of `==` tests, and this is the opposite of a plain `WHEN`'s comma.** Measured: `select case 2` with `when 1, 2 then say 'hit'` prints `hit`, while a plain `when 1, 2` on a non-logical value raises 34.6. The two commas parse into the same-looking node and mean opposite things, so an implementer who handles one and reuses it for the other gets a silently wrong answer rather than a failure. `ast.rs:801-815` records the distinction.

**Do not check a comma-list condition yourself.** A single-expression condition that is not `0` or `1` raises **34.1** under `IF` and **34.2** under `WHEN`. A comma list raises **34.6** from inside `eval_logical_list`, which already does it, and re-checking the result would replace 34.6 with 34.1. Measured across all four keywords, and the rule is that the sub-number is decided by the clause being a list at all, not by which element failed: `if 'x', 1 then` is 34.6, not 34.1.

A `SELECT` that reaches its `END` with no `WHEN` taken is **7.3**, and **the clause it echoes is the `END`, not the `SELECT`** (measured, rc 249). Raise it from the wrong arm and stdout and the exit code still match; only the stderr echo shows it.

`when 1 = 1 then` followed by `when 2 = 2 then nop` is **accepted, rc 0** (`ast.rs:776`, re-confirmed at run time). Do not confuse it with the *false*-condition variant, which segfaults the oracle and is upstream bug SF #2018, not ours to reproduce.

**The clause echo is indented by block nesting depth, two spaces per level, and `Raised::report` does not do this yet.** Measured: one enclosing `DO` gives two spaces, two gives four, three gives six; a `SELECT` contributes as well, and `do i = 1 to 'x'` gets none because the control expression is evaluated before the block is entered. This is the same indentation `TRACE` applies, so Task 13 shares it. It is unreachable today only because no block instruction exists; **this task is what makes it reachable**, and the first corpus program with a failure inside a block will diverge on stderr. Characterise the per-construct counting during implementation, since it is not simply one level per keyword.

- [ ] **Step 2: Run to watch them fail**

- [ ] **Step 3: Implement**

- [ ] **Step 4: Verify** — `cargo test`, plus each construct run under both interpreters through `rexx-run`, comparing stdout, stderr and exit code.

- [ ] **Step 5: Commit**

```bash
git add rust/crates/rexx-exec/src/run.rs
git commit -F <message-file>
```

---

