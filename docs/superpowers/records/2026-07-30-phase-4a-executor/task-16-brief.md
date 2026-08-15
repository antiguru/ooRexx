### Task 16: The gate harnesses

**Spec:** the 4a exit gate, all seven criteria.

**Files:**
- Create: `rust/crates/rexx-exec/tests/coverage.rs`, `tests/loud.rs`, `rust/scripts/mutate-4a.sh`
- **`docs/superpowers/plans/phase-4-exclusions.txt` ALREADY EXISTS** and is ahead of this plan; Step 4 below says what is still owed and why writing it would regress it

- [ ] **Step 1: The coverage enumeration** — a macro-generated match with **no wildcard arm** over `InstructionKind`, `ExprKind`, `LoopKind`, `PrefixOp`, `EndStyle`, `Trace` and `Operator`. Every variant carries either a witness program in the subset or the phase that owns it, and the test fails on a variant carrying neither.

**Take variant identity from the variant, never from `keyword()`.** `InstructionKind::keyword()` maps **both** `When` and `WhenCase` to `"WHEN"` (`ast.rs:912`), so a test keyed on it lets any `WHEN` silently satisfy `WhenCase` — the coverage number stays green while a variant goes unwitnessed. Found by a gap analysis whose own first run made exactly that mistake.

**`Operator::Backslash` carries an owner string, not a witness.** It cannot appear in a `Binary` node by design: `\` is prefix-only and a dyadic one is error 35.1. Demanding a witness would demand a program that cannot exist, which is the `LoopKind::With` shape one enum over.

The owner arm is an escape unless it is policed, so: the owner string must be one of the phases named in the spec's split table or its "assigned elsewhere" paragraph, and the **set** of out-of-4a variants is asserted, the way the exclusions file is. Otherwise a variant that turns out hard can be relabelled Phase 5's instead of getting a witness. The assignment is complete today — 40 `InstructionKind` variants (20 in 4a, 9 in 4b, 4 in 4c, 6 in Phase 5, 1 in Phase 7) and 15 `ExprKind` variants (9 in scope, 6 failing loudly) — so the assertion costs nothing to add now. Without the owner arm this criterion demands a witness for `LoopKind::With`, which needs Phase 5, and the criterion written to close a blindness finding would itself be unsatisfiable.

- [ ] **Step 2: The loud-failure enumeration** — for every `InstructionKind` and `ExprKind` variant, either 4a executes it or it produces the not-implemented exit code and names its owner. One test closes a surface larger than 4a's own.

- [ ] **Step 3: The mutation control** — a committed list of one-line mutations, each of which the subset must catch: off-by-one on `If::false_target`, on `When::exit`, on `Loop::end`; `Controlled::order` in fixed To/By/For order; `Abuttal` as `Blank`; `=` as `==`; `LEAVE` unwinding one block too few; formatting with the current digits, and with the current form, instead of the created pair.

The script **exits non-zero on an unapplied pattern**. That guard fired in four separate Phase 3 tasks, and without it a stale pattern reports coverage that does not exist. This is the one criterion a `cargo test` cannot be, since it edits the source it tests.

- [ ] **Step 4: `phase-4-exclusions.txt` ALREADY EXISTS and is ahead of this step. Do not write it, and do not follow this step literally.** It holds the 15 whole exclusions, the 3 partial rows, a **separate deviations section** (a deviation is permanent and chosen, an exclusion is work assigned to a later phase, and filing one as the other is how a deviation stops being reviewed), **and** a KNOWN GAPS section this step never described, pinned asymmetrically on purpose: adding a row needs no permission, removing one needs an owner or a measurement. The file has gained rows during the phase, including a second deviation and several gaps. Writing it from this step would **regress** it.
>
> What this task still owes the file is the **set assertion in the harness**, so the enumerated exclusions cannot drift from the file, and syncing the 66-of-81 phrasing. Nothing else.

- [ ] **Step 5: Assess every criterion, writing `docs/superpowers/plans/phase-4a-gate.md`.** That file does **not** exist and is this step's output, not its input; the seven criteria live in the design spec's "4a exit gate" section. Follow the shape of `phase-3-gate.md`: state what was measured, and where a criterion is met but weak, say so. A gate that reports only "met" is worth less than one that says which of its criteria could not have failed.

- [ ] **Step 6: Commit.**
