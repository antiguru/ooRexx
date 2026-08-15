STATUS: DONE

# Spec and plan audit, revision 10 / eight-plus amendments

Read-only audit of `docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md`
(revision 10) against `docs/superpowers/plans/2026-07-30-phase-4a-executor.md`
and against the tree at `4ce5aae8`. No crate touched.

**Eight findings, ordered by consequence.** Two would change what a pending
implementer builds. One is in a gate criterion. The rest are drift that costs a
reader time rather than a task its correctness.

The headline: **the documents have kept up with the numbers remarkably well** —
every stack figure I could check is current, including the two that were
superseded twice today — and the drift that remains is almost entirely in
*names and task numbers*, not in measurements. That is the opposite of the
failure mode I expected to find.

---

## 1. HIGH. Task 6 specifies a `Plan` that contradicts the `Plan` Task 3 built, in the paragraph that says it is inheriting it

Task 6 is **pending**, so this decides what gets built.

Plan, Task 6 Interfaces:

> Produces: `Plan { slots: HashMap<Box<[u8]>, usize>, len: usize }`,
> `build_plan(&CodeBody, &SymbolTable) -> Plan`

Tree, `rexx-exec/src/lib.rs:424`, committed by Task 3:

```rust
struct Plan {
    names: HashMap<Box<[u8]>, usize>,
    by_symbol: HashMap<SymbolId, usize>,
}
```

Three differences, and the second is the one that matters:

* `slots` is called `names`. Cosmetic.
* **`by_symbol` is absent from the Interfaces entirely.** That is the field the
  hot path uses: evaluation reaches a slot through the `SymbolId` the AST
  already carries, rather than hashing a byte string on every variable access.
  D16 opens by motivating the whole design with "variable lookup is 8.1% of
  runtime on the realistic mixed benchmark and 32.2% on stem-heavy code", so an
  implementer who builds the Interfaces literally puts a byte-string hash on
  precisely the path that costing is about.
* `len: usize` is a method in the built version. Cosmetic.

What makes it a contradiction rather than staleness is the paragraph
immediately below, added by revision 7:

> **`extra` is not optional and Task 3 already proved why.** … Task 3 built
> this; you are inheriting its shape, not inventing one.

The claim of inheritance is true of `extra` and false of `Plan` in the same
breath, so an implementer has no way to tell which half to trust.

D16 itself is *ambiguous* rather than wrong, which is probably how this
happened. It says "One `HashMap<Box<[u8]>, usize>` per plan, through which both
a `SymbolId` and a compound's tail pieces resolve" — and "through which a
`SymbolId` resolves" reads either as "at plan-build time, yielding an index"
(what Task 3 built) or as "on every access" (what Task 6's Interfaces says).
Both readings fit the sentence. Task 6's Interfaces resolved the ambiguity in
the direction that costs runtime, without noticing there was one.

**Suggested fix:** make Task 6's Interfaces quote the built struct, and add one
clause to D16 saying the name map is consulted at build time and the per-symbol
map is what evaluation indexes.

## 2. HIGH. Gate criterion 7 names the wrong task, and the spec calls the spike by two different numbers

Spec criterion 7, the gate itself:

> 7. **Zero `unsafe`, `clippy -D warnings` clean, `cargo fmt` clean**, and the
>    **Task 1 spike** committed with its findings written down.

The plan:

> `### Task 1: rexx-parse gives the main body a CodeBody`
> `### Task 3: The borrow-shape spike`

Task 1 is not a spike. The spike is Task 3, and it is committed as `5b5ccaf6`.

The spec is inconsistent with itself, not merely with the plan. **Three sites
say Task 1** — lines 309, 311 and 461 — and **two say Task 3** — lines 219
("Task 3's spike runs a fragment *and* the variable pool") and 255 ("Task 3's
spike measured the parenthesis axis"). The renumbering happened and missed
three, one of which is a gate criterion.

Line 309 is the oddest, because it justifies Task 1 by appealing to itself:

> `Fragment` has no labels at all — while the **Task 1 spike** runs a
> `Fragment`.

That paragraph exists to explain why Task 1 (the `CodeBody` change) is needed,
and its reason is a spike that is not Task 1.

Consequence is small in practice, since anyone checking criterion 7 will find
the spike, but a gate criterion naming a task that is a different task is the
one category where "a reader will work it out" is not good enough.

## 3. MEDIUM-HIGH. D19's two-cliffs table describes behaviour Task 3c removed, in the present tense

Spec, D19:

> | 85,000 parens | rc 245 | rc 0 |
> | 90,000 parens | rc 245 | **rc 134, SIGABRT, no message** |
>
> So on parens we diverge in **both** directions: we succeed from 40,000 to
> 85,000 where the oracle raises `Insufficient control stack space`, and we
> **abort past 90,000** where the oracle still raises.

Both halves are now false of the tree. `MAX_EXPR_DEPTH` is 50,000, so:

* we succeed from 40,000 to **50,000**, not to 85,000 — the divergence band is
  less than a fifth of the width stated;
* we do **not** abort past 90,000. Measured on the shipped binary during Task
  3c's review: 50,001 and 100,000 parens both raise `11.1` cleanly.

The section reads as current state — the column is headed "ours, debug, sized
thread" with no as-of marker — and the paragraph after it draws a live
conclusion from it. The measurement was correct when taken; the table needs an
"as of, before Task 3c" line, or replacing with the post-fix behaviour.

This is the same shape as the defect Task 3d fixed inside `depth_probe.rs`, one
document up.

## 4. MEDIUM. Neither document records the constraint that decides `by_symbol`'s shape

`rexx_parse::SymbolId` is a newtype over a **private** `u32` with no accessor,
so nothing outside `rexx-parse` can turn one into a `Vec` index. That is why
`by_symbol` in finding 1 is a `HashMap` rather than the array D16's
integer-slots framing implies.

Grepped: the string appears nowhere in `docs/superpowers/`. It lives only in
Task 3's report. Task 6 is the task that will hit it, and its options are to add
`SymbolId::index()` to `rexx-parse` or to keep the hash deliberately — a
decision worth making rather than rediscovering under time pressure.

## 5. MEDIUM. Task 9 does not name `clear_slot`, the operation Task 2b added specifically to unblock it

Task 2b's stated justification was that the gap "**does** block plain `DROP a`
on a simple variable in Task 9". `clear_slot` landed for that reason in
`4ce5aae8`.

Task 9's text says only:

> `DROP` of a variable, a tail, a whole stem and the `(v)` indirect form

with no reference to `clear_slot` and no note that writing `ObjRef::NIL` is
wrong. The measurement that makes it wrong — `y = .nil; drop y; say y` printing
`Y`, not `The NIL object` — is in Task 2b's report and `clear_slot`'s doc
comment, both of which a Task 9 implementer might not open.

This is the "a fact recorded outside the task body does not reach the
implementer" pattern the plan's own preamble warns about, since briefs are
extracted per task.

## 6. LOW-MEDIUM. "Three recursions, three cliffs" is now four, plus one unguarded

Spec:

> **Phase 3's parser has the same exposure, and it is now measured: three
> recursions, three cliffs, on a default 2 MiB stack.**

Task 3d added a fourth to the same family, the `arg_list` descent for
`f(f(f(…)))`, now sharing `MAX_EXPR_DEPTH`. And a fifth is known and unguarded:
prefix chains in `message_subterm`, aborting at 1,150-1,200 on a default
thread, which the exclusions file carries as a known gap.

The table also gives the paren cliff as "~85,000 debug on the sized thread"
where Task 3c bisected it to [88,800, 89,000].

## 7. LOW. Renamed and misnamed symbols

* **`parse_subterm` does not exist.** The function is `subterm`
  (`expr.rs:1045`). Spec's recursion table and plan lines 307 and 361 both use
  the longer name.
* **`MAX_PAREN_DEPTH` no longer exists**, renamed to `MAX_EXPR_DEPTH` by Task 3d
  when the budget became shared. Plan lines 375, 388 and 390 still use it.

Both sit in text for **completed** tasks, so nothing will be built wrong. The
cost is a reader grepping for a symbol that is not there.

## 8. LOW. The exclusions section now describes three sections while still numbering two

Revision 10 added the known-gap third status at line 484. Line 486 still opens
"The file carries a **second section** for semantic deviations", so the reading
order introduces the third status before the second. No claim is wrong; the
numbering just no longer helps.

---

## What I checked and found correct

Worth recording, since the point of an audit is also to say where drift is not.

* **Every stack figure agrees across documents and matches the tree.** The plan
  carries 512 MiB, ~783 bytes per `eval` level and ~685,000 survivable levels,
  with an explicit note that the earlier ~820 and ~850 are void because Task 3b
  removed the recursions they measured. That is the corrected set, not either
  superseded one.
* **The exit code appears as a number in no document.** The exclusions file
  names `NOT_IMPLEMENTED_EXIT` and argues from the 157..253 band instead, so
  Task 12 changing the value breaks nothing.
* **The oracle cliffs agree**: [39,900, 39,950] parens and [34,500, 34,760]
  calls appear identically in the plan, the exclusions file and my own
  measurements.
* **Criterion 1's variant arithmetic is right**: 20+9+4+6+1 = 40
  `InstructionKind` variants, and 9+6 = 15 `ExprKind` variants. Both counts
  match the enums in the tree.
* **`BuiltinFunctions.cpp:3042`** is `builtinTable[]` and holds exactly 81
  entries, so `81 - 15 = 66` derives.
* **D16's `extra` bullet, D19's parity-versus-deviation split, and the
  exclusions file's deviation row 2** all agree with each other and with the
  tree.
* **The `Body::String` references** in both documents are describing its
  deletion, not asserting it exists. Correct as written.

## Method and limits

Grep sweeps for numbers appearing in more than one document, then each
disagreement read in context on both sides before being called a finding. Symbol
names checked against the tree by grep rather than by memory. Nothing here rests
on a measurement I did not either take today or take earlier in this session and
cite.

I did **not** re-run the oracle for figures both documents already agree on and
that I measured earlier today; finding 3's claim that 50,001 parens raises 11.1
is from Task 3c's review, on the shipped binary. I did not audit the L1
coverage, `rexxcps` or trace-prefix sections against their sources, which are
Phase 4b and 4c surfaces and outside what I have context on.
