# Task 4 review: `RETURN`, `EXIT`, `PUSH` and `QUEUE` get an op (`0d058c086`)

**Spec compliance: PASS.**
**Code quality: PASS, with one Major prose finding and five smaller ones.**

Everything below was measured in a detached worktree at `0d058c086` with its own
`CARGO_TARGET_DIR`, and a control worktree at the parent `54afba8a1`. No repository file
was edited; the mutation and probe work was done in the worktree and restored from a
private backup verified with `sha256sum -c`.

---

## 1. The shape decision, verified in the code

The brief said this was Critical if any leg were false. All three legs hold, read off the
**parent** commit's `run.rs`:

| leg | evidence at `54afba8a1` |
|---|---|
| `RETURN`/`EXIT` differ only in the `Flow` constructor | `run.rs:2180`-`2193` and `run.rs:1469`-`1506` are the same statements in the same order -- `eval`, `roots.push_temp`, `result_text` gated on `Some`, `trace_result`, `Some(value)` -- and differ in `Flow::Return(value)` against `Flow::Exit(value)`. Nothing else. |
| `PUSH`/`QUEUE` differ only in the queue end | `run.rs:2275`-`2292` was **already** one arm with `if matches!(.., Push{..}) { queue.push } else { queue.queue }`. |
| the two groups differ in three things | (a) trace: `result_text` (`trace.rs:978`, `trace_mode().results.then(|| to_text(..))`) is reached only inside the `Some` arm, so a bare `RETURN`/`EXIT` traces nothing, against `to_text(..).to_vec()`/`Vec::new()` with `trace_result` called **unconditionally**, so a bare `PUSH`/`QUEUE` traces the null string it queues; (b) side effect: none against a queue write; (c) region end: `Flow::Return`/`Flow::Exit` against `Flow::Next`. |

So "four ops" writes each pair twice and "one op with a tag" rests on a premise the code
refutes. **Two tagged ops is the honest shape and the implementer's argument is sound.**
The reasoning is in the doc comments as the brief required: `ReturnKeyword` and
`QueueKeyword` in `run.rs`, `Op::Return`/`Op::Queue` in `ir/mod.rs`, and both `compile`
arms.

One small note on leg (a): `result_text` gates on `results` and `trace_result` gates on
`results` again, so the `result_text` gate is redundant as a trace gate; what it actually
buys on the `RETURN` side is skipping the render when tracing is off. The report's
description is accurate as a description of the code either way.

---

## 2. Riskiest item 1 -- the corpus edit and the regenerated golden: **PASS**

* `corpus/lang/push_queue.rex` is **39 lines before and after**. `diff` over `cat -n`
  output shows exactly lines 25-29 changed, and all five are inside the header comment
  block. **No traced line number moved**, which is the property the file's own trace
  output depends on.
* `crates/rexx-parse/tests/sourceline_oracle/push_queue.txt` is **40 lines before and
  after**, `count 39` unchanged, and differs only at lines 26-30 -- the same five lines,
  offset by the count header.
* Stronger check: the golden's body with the count header stripped is **byte-identical to
  the program source**. So the regenerated file contains the edit and nothing else; there
  is no room in it for a laundered behaviour change.
* The corrected comment is **true on all three of its claims**:
  * *the gap is closed*: measured. Oracle, from a fresh empty directory,
    `trace r`/`say 'a'`/`exit 3` -> rc 3 with `>>>   "3"`. This crate, `REXX_ENGINE=tree-walker`
    and `REXX_ENGINE=ir`, prints the identical four stderr lines at rc 3.
  * *as `condition_traps.rex`'s own header records*: that header says, verbatim,
    "**That gap is closed as of 4b Task 9**", and names `tests/trace_oracle/exit_value.rex`.
  * *pinned by `tests/trace_oracle/exit_value.rex`*: the file exists and is an
    oracle-compared trace case for `exit 1 + 1` under `trace r`.

The old comment was therefore stale rather than load-bearing, and correcting it in a
five-for-five line budget was the right way to do it.

## 3. Riskiest item 2 -- the pre-existing expectation moved 1 -> 2: **PASS**

`ir::drive::tests::a_body_entered_under_trace_r_echoes_its_promoted_clause_from_the_chunk`.
Verified by probe rather than by reading. A temporary test in the worktree ran the same
harness over five callee bodies and counted `trace_op_echoes()`:

| callee body | echoes |
|---|---:|
| `if 1 = 1 then nop` + `return` (the committed program) | 2 |
| `return` alone | 1 |
| `if 1 = 1 then nop` alone | 1 |
| `nop` + `return` | 1 |
| `say 'x'` + `return` | 2 |

So the second echo is the `RETURN`'s, for exactly the reason the message now gives, and
nothing else about the counter moved (`nop` is still unpromoted, the `THEN` body still
contributes nothing). Probe removed; file restored and `sha256sum -c` OK.

---

## 4. The other named checks

**The activation-stack `debug_assert` is meaningful and not sidestepped.**
`Op::Return`'s `break 'region Ok(RegionEnd::Flowed(flow))` (`ir/drive.rs:1070`) lands on
`RegionEnd::Flowed(flow) => (flow, end)` (`drive.rs:1355`) and falls straight into
`debug_assert_eq!(self.activations.len(), depth, ...)` at `drive.rs:1457` -- the same path
`Op::Call`'s flowing arm already takes, with no early `return` between them. It is
*meaningful* because `returned_value` only pushes a temp, traces and builds a `Flow`: the
activation is popped by `apply_flow` above `run_chunk`, so the depth at the assert
genuinely is unchanged rather than the assert being weakened to accommodate the new op.
Debug binaries running `RETURN` inside a `SELECT`, inside `DO FOREVER`, inside a
`::ROUTINE` and `EXIT` from inside a counted loop did not trip it.

**`RETURN` bare against `RETURN ''`.** `compile` allocates no register and emits no value
op for the bare form; the golden pin reads `src=-` over a two-op region; the driver's
`src.map` yields `None`; `returned_value` traces nothing for `None`. The case row pins the
caller's side, and I re-measured that row against the oracle: `after empty: ` (trailing
blank) against `after bare: RESULT`, byte for byte.

**The shared halves are entered, not copied.** `ir/drive.rs` contains no `trace_result`,
no `queue.push`/`queue.queue`, and no `Flow::Return(`/`Flow::Exit(` construction for these
ops -- the only `Flow::Exit(` in the file is the pre-existing `ClauseOutcome::Ended` line.
Both engines call `Interp::returned_value` (`run.rs:1502`, `run.rs:2180`, `drive.rs:1069`)
and `Interp::queue_evaluated` (`run.rs:2269`, `drive.rs:1101`).

**Rows the report says were already in the tree.** Each checked by opening the file, not
by trusting the list: `trace-settings` has `err>      7 *-*   return` with no value line
after it; `calls` has `10 *-*     return zn + 1` with `>V>     RESULT => "8"` and a bare
`7 *-*   return`, the two-indent `>>>` pair, and the `EXIT`-from-a-called-label /
`::ROUTINE` row at `rc> 3`; `pull_queue.rex` does `push`/`queue` then `pull`/`parse pull`.
The enumeration is accurate.

---

## 5. Gates and independent behaviour evidence

| gate | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, zero `^warning`/`^error` lines |
| `memcap 8G cargo test --workspace --no-fail-fast` | **1476 passed, 0 failed, 4 ignored** -- matches the report |
| `const _: () = assert!(size_of::<Op>() == 16)` | live and holding; negative control at `== 12` gives `error[E0080]: evaluation panicked` at `ir/mod.rs:83`, exit 101, exactly as reported |

Two environment caveats for anyone reproducing in a worktree: the gitignored `ootest/` and
`rust/corpus-l1/` must be reachable at their usual relative paths (27 tests fail otherwise)
and `target/debug/rexx-run` must exist (1 further test fails otherwise). Neither is related
to this diff.

Beyond the suite:

* **Every committed row re-measured against the C++ oracle.** All four stanzas of
  `tests/ir_dual_cases/return-and-queue` were extracted, run under the wrapper from a
  directory created empty, and rendered back into the file's own `rc>`/`out>`/`err>` form:
  **all four match the committed expectation byte for byte**, including the trailing blank
  in `out> after empty: `.
* **HEAD against the parent binary on the whole L0 corpus**: 70 programs x 2 engines = 140
  comparisons of stdout+stderr+rc, **0 divergences**.
* **HEAD against the parent binary on a 400-program L1 sample**: **0 divergences**.
* **Ten adversarial three-way probes on shapes no committed row holds** -- `RETURN` from
  each `WHEN` of a `SELECT`, `EXIT` from a counted loop inside a called label, `PUSH`
  inside an `IF`'s `THEN` and `ELSE`, a compound tail and a bare stem pushed under
  `trace i`, `interpret "exit 3"`, `RETURN` inside `DO FOREVER`, a `::ROUTINE` holding both
  a `RETURN` and an `EXIT`, `PUSH`/`QUEUE` in loops with bare forms and full read-back,
  nested calls settling `RESULT` through two levels, and `push 1/0` under `SIGNAL ON SYNTAX`
  -- **stdout, stderr and exit status identical to the oracle on the IR engine in all ten**,
  and identical to the parent binary on both engines.

---

## 6. Findings

**1. Major -- `Interp::returned_value`'s doc attributes the unrooted window to `EXIT`
alone, on a function that now also serves `RETURN`.**
`run.rs` (the `returned_value` doc): "**The rooting here is shorter than the value needs,
and that is `EXIT`'s window rather than a general one.**", and the paragraph under it is
phrased entirely in `Flow::Exit` terms ("`step_in_temps_frame` pops it before `Flow::Exit`
has even reached `run_activation`"). But `Interp::apply_flow` (`run.rs:1332`-`1344`) takes
`root_exit_value` on **both** arms, and its own comment says "`EXIT`, a top-level `RETURN`,
a `RAISE` with an `EXIT` tail and a handler's own exit all arrive as one of these two
variants"; `docs/superpowers/plans/phase-4f-record.md` records that rooting `Flow::Exit`
alone left three harnesses panicking *because a top-level `RETURN` reaches `exit_code_for`
too*. On the old `EXIT` arm the sentence read as "EXIT against other evals"; on a function
shared by both keywords it reads as "EXIT against RETURN", which the tree's own code and
record refute. This is the recurring defect class -- a comment claiming a discrimination
its own subject cannot make. Fix is one clause: name `EXIT` **and a top-level `RETURN`**.

**2. Minor -- the case file's `src: Option` justification overgeneralises to
`PUSH`/`QUEUE`.**
`tests/ir_dual_cases/return-and-queue` header: "A bare form takes no register at all --
`src` is `None` and not a register holding a null string, because the two are different
instructions". That is true and oracle-pinned for `RETURN` (row 3). For `PUSH`/`QUEUE` it
is not: a bare `queue` and `queue ''` store the same line and trace the same `>>>   ""`,
and no row in this file or the workspace can distinguish `Op::Queue { src: None }` from one
handed a register holding the null string. `Op::Queue`'s own doc in `ir/mod.rs` is honest
about this ("`src` is an `Option` for the reason `Op::Say`'s is"); the case-file header
borrows `RETURN`'s discrimination to justify all four.

**3. Minor -- two cardinality-in-prose violations in the new case file.**
"`tests/trace_oracle/exit_value.rex` and **three** `run::tests` indent witnesses redden
with it" and "that mutation reddens the extracted-program sweep, the branch-shape table and
**a dozen** `run::tests` besides". The plan's constraint is that a comment may not name the
size of a set. The report names the three `run::tests` explicitly, so replacing the count
with the names is free; "a dozen" is not a measured number anywhere (M7's row records 19
tests reddened in total, not twelve `run::tests`).

**4. Minor -- three copies of the same compile arm and three of the same driver preamble.**
`compile.rs`'s `Say`, `Return` and `Queue` arms are the same ~30 lines (mark, `op_index`,
`echoes`, `Op::Clause`, `push_echo`, the `match expression` that allocates and calls
`push_value`, `close_region`, `release`) differing only in the terminal `ops.push`;
`drive.rs`'s `Op::Say`/`Op::Return`/`Op::Queue` arms each repeat the same
`debug_assert_names_the_clause` + arity `debug_assert` + `src.map(|register| { debug_assert!(chunk.holds_register(..)); temp_at(..) })`.
The report's own argument for two ops rather than four was to avoid writing a pair's arm
twice, and the compile side ends up written twice regardless. A
`push_expression_clause(..) -> Option<u16>` helper would collapse all three; the file
already extracts `push_value`, `push_echo` and `close_region`, so the convention exists.

**5. Minor -- the arity `debug_assert` does not pin the keyword against the instruction
kind.** `Op::Return`'s assert accepts `Return { .. } | Exit { .. }` with matching arity, so
an op tagged `Exit` sitting on a `RETURN` clause passes it; same for `Op::Queue`. That is
exactly mutation M1, which is caught only by the case row and the golden pins. Folding the
keyword into the same `matches!` would make the stream self-checking at no run-time cost in
release.

**6. Minor -- `docs/superpowers/plans/2026-08-12-remaining-promotion-survey.md` is
falsified and untouched.** Its "Already promoted" list omits these four, and its
"Promotable ... **One expression then a `Flow`**" list still names `RETURN`, `EXIT`, `PUSH`
and `QUEUE`. The commit updated the plan file but not the survey the plan was written from.
Mitigating: no sibling task in this plan touched the survey either, so this is drift the
plan has been accumulating rather than something this task started.

**7. Nit -- `corpus/lang/push_queue.rex`'s header enumeration is now incomplete.** It still
says reading the order back "needs a construct this program's own subset does not admit",
naming `lang/pull_queue.rex` and `input_oracle.rs`'s `queue-round-trip` row;
`ir_dual_cases/return-and-queue` is now a third place and the strongest one. The five-line
budget the task correctly imposed on itself explains the omission, but the edit could have
spent its five lines on this instead.

**8. Nit -- `tests/ir_dual.rs`'s module doc lists the shared halves the comparison cannot
see (`run_loop`, `assign_evaluated`, `say_evaluated`, `invoke_call`) and does not add
`returned_value`/`queue_evaluated`.** The list does not claim to be exhaustive, and the new
case file's M4 result is precisely an instance of the property that doc describes, so this
is at worst a missed opportunity.

Nothing in the diff uses an em-dash; no test or assertion was deleted (0 removed `#[test]`,
5 added); the `+5` test count is exactly the five new golden pins.

---

## 7. Adjudication of the implementer's five concerns

1. **`Op::Queue` earns less than `Op::Return`.** Agreed and correctly flagged. It is
   uniformity with `Op::Say` rather than a profile result, which the brief asked for; the
   one thing it buys beyond uniformity is M4's unique catch, which is real.
2. **The `EXIT` row overlaps `exit_value.rex`.** Keep the row. It carries the keyword tag's
   discrimination -- I reproduced M1: tagging an `EXIT`'s op `ReturnKeyword::Return` fails
   the row with `left: ""` / `right: "not reached\n"`, exactly the signature the row's
   comment claims -- and it reaches the `>>>` at a callee's indent, where `exit_value.rex`
   is DEVIATION-0-normalised and runs one engine.
3. **M8's real catcher is a `debug_assert`, unmeasured in release.** Honest and correct,
   and the risk is small: the row's load-bearing discrimination is M7 (`after empty: RESULT`
   where the oracle prints `after empty: `), which is a byte comparison that survives a
   release build. No action needed.
4. **The `SIGNAL ON` indent divergence is unfiled.** Upheld, and independently reproduced:
   handler clauses one indent deep, both engines byte-identical, the `say 1/0` control
   diverging identically -- **and byte-identical when run against a binary built from the
   parent commit**, so it is provably pre-existing rather than this task's. Recording it in
   the plan file (which this commit does) rather than filing it is acceptable; if it is ever
   decided to be permanent, `docs/superpowers/plans/phase-4-exclusions.txt` is where it
   belongs.
5. **One measured unique catch, both directions. REPRODUCED.** With `queue_evaluated`'s
   `trace_result` call removed and `rexx-run` rebuilt: `memcap 8G cargo test --workspace
   --no-fail-fast` gives **1475 passed, 1 failed, 4 ignored**, and the single red test is
   `both_engines_agree_on_every_case_file`. With `tests/ir_dual_cases/return-and-queue`
   moved out of the directory and the same mutation still in place: **1476 passed, 0 failed,
   4 ignored** -- the whole workspace green. Tree restored, `sha256sum -c` OK, `rexx-run`
   rebuilt afterwards. The claim is exactly right, including the reason: both engines take
   the mutation, so only the oracle-measured expected block can see it.
