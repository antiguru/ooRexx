# Phase 3 parser plan — sixth pass, reviewing the round-5 fix

Target: the fix commit `094a7c45` ("Give the clause split two byte positions instead of one")
against `docs/superpowers/plans/2026-07-28-phase-3-parser.md`.
Specification reviewed against: `.superpowers/sdd/phase-3-plan-review-round5.md` §7.
Reviewed: 2026-07-28. Oracle: `build/bin/rexx`, `build/bin/rexxc`, and the C++ tree in this worktree.
Every finding below is tagged **VERIFIED** (I ran it, or read the C++ at the cited line).

---

## Verdict

**GO**, with two one-line edits to land before Task 3.1 and Task 3.4 are dispatched.
Neither needs investigation and neither warrants a seventh review round.

Eight of eight round-5 findings were addressed and six are cleanly fixed.
The `split_before` signature change is correct: I re-derived the token indices, both call
sites, the `assert!` at all three boundaries and the two C++ citations, and every one holds.
The fix's most suspect claim — that `END` and `WHEN` trace as clauses of their own — is
**true**, measured independently rather than generalised from `THEN`.
The `assert!` is correct including the case the round-5 fix order did not mention.
The worked example's token indices follow from the plan's own two-sided blank rule, and the
two `split_before` calls compose to the three spans the oracle prints.

The recurrence recurs, twice, and both times as a survivor rather than a new error.
Task 3.4 still says clause spans are derivable from a token sub-range — four lines below the
paragraph the fix added to say they are not.
Task 3.1 Step 3b constrains three keywords while Task 3.6 Step 4 now needs five and cites
Step 3b as its authority.
Both are single-line edits and both are already stated correctly somewhere else in the plan,
which is why this is a GO rather than a sixth NO-GO: the plan cannot produce a wrong parser
that passes the gate, it can only produce avoidable rework.

Two things I found by probing dimensions the plan gives no reason to think matter.
`TRACE.testGroup` contains **sixteen** distinct `>X>` markers, not fifteen — `>.>`
(`TRACE_PREFIX_DUMMY`) is missed by exactly the character class the plan warns about, and
`<I<` is missed by any `>…>` shape at all.
And a continued clause's `*-*` text is not a contiguous byte range: `say "x",` / `"y"` traces
as `say "x","y"`, the newline gone.
Nothing in the gate's scope exercises either, so both are Minor.

---

## 1. The eight round-5 findings, one verdict each

| # | finding | verdict |
|---|---|---|
| C1(new) | `split_before` is a partition and the oracle is not | **fixed** — signature is `split_before(ctx, at, end_at)`; the finished clause ends at `end_at`, the remainder restarts at `ctx.tokens[at].span.start`. Caller rules stated per keyword with the right C++ citations, the gap invariant stated in three places (Task 3.4 rule 4, the doc comment, Notes), and a worked example whose indices and both calls I re-derived and confirmed. `src/instruction.rs` added to Task 3.9's Files. One survivor contradicts it four lines above: N1. |
| I1(new) | criterion 1 said "instruction spans" without saying which | **fixed** — property 2 is now over `Instruction::clause_span`, and the justification was rewritten to say *why* that makes it shape-independent (node extents nest, clause spans do not). The claim that property 2 "already permits" interstitial whitespace is **correct**: VERIFIED, the criterion's own text has said "the only bytes between … are whitespace, comments and `,`/`-` continuations" since round 4. |
| I2(new) | criterion 6 forces Step 3b's answer and no task says so | **partially fixed** — the constraint is now at the decision point, and the inert "cannot be written until Step 3b lands" is gone. But Step 3b binds `THEN`, `ELSE` and `OTHERWISE`; Task 3.6 Step 4 needs **five** nodes and says they are covered "because Step 3b is constrained … see that step". `END` and `WHEN` are not in that step. N2. |
| M1 | `Eoc` model omitted end of file | **fixed** — all three terminators, with the reason spelled out and Task 3.4 rule 1 cross-referenced. Task 3.4 rule 1 does list all three (line 609). |
| M2 | probe A description incomplete; acceptance read as a sequence comparison | **partially fixed** — the sequence half is fixed well, and the replacement number is right where round 5's was wrong: VERIFIED **seven** `*-*` lines for **three** clauses on probe A's line 3 (round 5 said six for four; it is seven for three). The description half survives: Step 1 still says "probe A traces `nop;`, `do i = 1 to 2;` and `say i;`" and still omits `end`, which line 1501 now mentions fifteen lines later. N5. |
| M3 | Task 3.9's Files omitted `src/instruction.rs` | **fixed** — added, with a sentence saying why the end byte is repaired there and nowhere else. |
| M4 | Task 3.9's Interfaces mis-attributed `clause_span` | **fixed** — "introduced in Task 3.6 and set from the `Clause::span` that Task 3.4 produces", plus an explicit "do not go looking for `clause_span` in Task 3.4". |
| M5 | the plan narrowed a parent criterion silently | **fixed** — criterion 6 now records the narrowing, names the principle round 4 established, and justifies the scope. |

**8 of 8 addressed. 6 cleanly fixed, 2 partially. No fix introduced a new defect of its own.**

### What I checked rather than took on trust

* `ThenInstruction.cpp:76` is `setLocation(token->getLocation())`, and the comment two lines
  above reads "the location was set to the full clause containing the THEN, but we want just
  the location of the THEN keyword for this". VERIFIED, exactly as the plan says.
* `IfInstruction.cpp:58–66` is `setEnd(location.getLineNumber(), location.getOffset())` from
  the THEN token, with the comment "the THEN is traced on its own, but using the start of the
  THEN gives a fuller picture". VERIFIED — start, not end, as the plan says.
* **`WHEN` independently, not by analogy.** `when 1 = 1   then    say "w"` traces
  `when 1 = 1   ` with all three trailing blanks, then `then` alone, then `say "w"` with no
  leading blank. `when 0 = 1` traces its condition alone and never traces the `then`. So
  `WHEN` behaves as `IF` does and does get its own `*-*` clause. The claim holds.
* **`END`.** VERIFIED as its own `*-*` clause in a `DO` loop. Worth knowing: a `SELECT`'s
  `END` is **not** traced when a `WHEN` body ran, because control leaves past it — which the
  fix's per-line acceptance now tolerates and a count-based one would not. The M2 fix is
  right for a reason the fix does not state.
* **The `assert!`, at all three boundaries.** `cur.span.contains(&end_at) || end_at == cur.span.end`
  is correct: `Range::contains` excludes the end, so the disjunct is required and is not
  redundant. In the worked example's first call `end_at` is strictly interior; in the second
  it is interior to the pending clause's span. I could construct no legitimate call it
  rejects.
* **The worked example.** `[0]if [1]Blank [2]6 [3]> [4]5 [5]Blank [6]then [7]Blank [8]say
  [9]Blank [10]"big" [11]Eoc` follows from the plan's own two-sided blank rule for all six
  blank positions, including the two that are correctly absent (`6 > ` and `> 5`). Call 1
  yields `if 6 > 5 ` and a pending clause `6..12`; call 2 asserts `8 ∈ 6..12`, yields `then`
  with no blank either side, and leaves `say "big"` starting at `say`. Internally consistent
  with the signature as written.
* **The Rexx facts earlier drafts got wrong.** Keywords not reserved and positional
  (Task 3.6 Step 2, Notes) — untouched and still right. `-2 ** 2` is 4 (Task 3.5) — untouched.
  `f(x)` vs `f (x)` (Task 3.3 test) — untouched. No column anywhere — untouched, and the
  fix added none.

---

## 2. New findings

### Important

#### N1 — Task 3.4 still says clause spans are derivable from a token sub-range

**VERIFIED** against the fix's own worked example.
Task 3.4, line 647, four lines below the paragraph the fix added:

> So `split_clauses` must produce clauses that Task 3.6's cursor can cut further — `tokens`
> is a range, **`span` is derivable from any sub-range of it**, and nothing in `Clause` is
> shared or interned.

That is now false, and it is false in precisely the case C1(new) was about.
The `THEN` clause the fix produces has `tokens: 6..8` — it must, because `at` is the restart
token and `split_before(ctx, 7, …)` would restart the remainder at the blank and give
` say "big"` a leading blank — while its `span` stops at token 6's end.
A span derived from tokens `6..8` would run to token 7's end and yield `then ` with the
trailing blank the oracle does not print.
The condition clause happens to be derivable (token 5's end equals the `THEN`'s start), which
is what makes the sentence look true.

**Fix, one sentence:** `span` is derivable for the clauses `split_clauses` itself produces,
but a clause the cursor cuts may have a `span` narrower than its `tokens` — which is why
`split_before` takes the end byte explicitly.
Worth adding in the same breath: a clause returned by `split_before` carries no `Eoc` token,
so an instruction parser must not require one.

#### N2 — Step 3b constrains three keywords; Task 3.6 Step 4 needs five and cites Step 3b for all five

**VERIFIED** by reading the two passages against each other and against criterion 6's scope.

Task 3.1 Step 3b:

> `THEN`, `ELSE` and `OTHERWISE` must remain instructions of their own, each carrying its own
> `clause_span`.

Task 3.6 Step 4:

> **All 35 rows are writable, including the five keyword clauses.** `THEN`, `ELSE`, `END`,
> `WHEN` and `OTHERWISE` each get a node of their own regardless of which shape Task 3.1
> Step 3b chose, because Step 3b is constrained to keep `THEN`, `ELSE` and `OTHERWISE` as
> separate instructions carrying their own `clause_span` — **see that step**. `END` and
> `WHEN` follow the same rule for the same reason.

The cross-reference does not hold: the step it points at binds three of the five.
Task 3.1 is the **first** task executed, so this is the one place where the omission is
expensive — an implementer who picks the tree shape and folds `END` into `Do` satisfies
Step 3b as written and then fails Task 3.6 Step 4's 35-row assertion and criterion 6.

And criterion 6 needs `END` specifically, not by analogy: probe A's `end` is a traced `*-*`
line inside the criterion's own three-file scope, VERIFIED.
Conversely `ELSE` and `OTHERWISE` are **not** in criterion 6's scope — `trace_output.rex`
contains neither, and neither probe does — so Step 3b's stated justification ("criterion 6
gates on reconstructing each of them") is the wrong reason for two of the three keywords it
names.
The right reason for all five is Task 3.6 Step 4's table, which requires a node named after
every one of the 35 keywords, plus criterion 6 for `THEN` and `END`.

**Fix, one line:** make Step 3b's list `THEN`, `ELSE`, `END`, `WHEN` and `OTHERWISE`, and
give the reason as "Task 3.6 Step 4 asserts a node per keyword for all 35, and criterion 6
gates on the `then` and `end` clause texts".
Same edit to the matching Notes entry at line 1761, which also lists three.

### Minor

**N3 — `TRACE.testGroup` has sixteen `>X>` markers, not fifteen, and the plan's own warning
misses two shapes.** VERIFIED: `grep -oE '>.>'` returns sixteen distinct markers. The
sixteenth is `>.>` — `TRACE_PREFIX_DUMMY` in `RexxActivation.cpp:3572` — appearing three
times, at lines 244, 651 and 709, from `arg 1 a .`. It is missed by the same `[A-Za-z=]`
class the plan warns about, so criterion 6's parenthetical is right in kind and incomplete in
fact. The file also contains `<I<` (`TRACE_PREFIX_INVOCATION_EXIT`, e.g. lines 354, 385, 941)
and `+++` (line 223, `+++   "RC(30)"`), neither of which any `>…>` pattern finds. The
interpreter's table at `RexxActivation.cpp:3567–3587` has **nineteen** prefixes and
`TRACE.testGroup` exercises **all nineteen**, so the honest figure is "eighteen markers other
than `*-*`, and the file has every one of them". Task 3.9's enumeration at line 1430 lists
fifteen and is incomplete in the same sentence that says "Do not enumerate them: the obvious
list is incomplete" — which is a better argument than the count it accompanies. The 243
value-marker line count is consistent with the pattern it was derived from; the complete
pattern gives 266. Nothing gates on any of these numbers.

**N4 — `*-*` reconstruction from a single `Range<usize>` cannot reproduce a continued
clause.** VERIFIED, varying a dimension the plan does not consider: `say "x",` / `"y"` traces
as `say "x","y"` and `say "p"-` / `     "q"` as `say "p"-     "q"` — the continuation
character kept, the next line's leading blanks kept, the **newline removed**. So
`&source[clause_span]` contains a `\n` the traced line does not, and the Notes' unqualified
"`TRACE`'s `*-*` line prints `clause_span`" is false for any continued clause. No file in
criterion 6's scope contains a continuation, so nothing in this phase can fail on it. One
sentence in Task 3.9: a clause spanning lines traces as its covered lines joined without
their newlines, which a single range plus a join rule reproduces, and no gated file exercises
it.

**N5 — M2's other half survives.** Task 3.9 Step 1 still reads "Measured, probe A traces
`nop;`, `do i = 1 to 2;` and `say i;` **with their semicolons**", omitting `end`, which is a
fourth traced clause and the one round 5 flagged. The per-line paragraph fifteen lines below
now names it. Add it to the enumeration too.

**N6 — the doc comment dropped an invariant it still relies on.** The removed text said "The
remainder keeps the original terminating byte, so its span still includes any `;`". The code
still does `..cur.span.end`, so the behaviour survives and the reason does not. One clause,
restored.

**N7 — "tiles" is used in two senses.** Criterion 1 requires each program to "tile", defined
as containment plus ordering with whitespace interstices permitted. The new Notes entry says
"Clause spans do not tile the source", meaning they are not a partition. Both are correct and
the words collide. Say "partition" in the Notes entry.

**N8 — the `assert!` could catch the bug C1(new) was about, and does not.** It bounds
`end_at` to the clause but permits `end_at > ctx.tokens[at].span.start`, which is exactly the
overlapping-span mistake a caller reverting to a single cut point would make.
`assert!(end_at <= ctx.tokens[at].span.start)` states the invariant the criterion depends on.
A strengthening, not a defect.

---

## 3. Contradiction-elsewhere scan, scoped per task

Each of the three changes, scanned inside the task that owns it rather than document-wide.

* **`split_before`'s signature.** Task 3.6 is internally consistent: struct field doc, method
  doc, body, worked example and the `clause_span` paragraph all agree, and the
  `clause_span`-versus-`split_before` tension round 5 flagged is explicitly resolved at line
  892. Task 3.4, which owns rule 4, contradicts it at line 647 — **N1**. Task 3.7b (line
  1240) and Task 3.9 (line 1462) mention `split_before` only in ways the change does not
  touch. No other task calls it.
* **Criterion 1's property 2.** Inside the exit gate the change is complete: the property,
  its justification and the interstice paragraph now agree, and the new paragraph explains the
  shape-independence rather than asserting it. The only collision is terminological — **N7**.
* **Task 3.6 no longer deferring five rows.** Inside Task 3.6 nothing else depends on the
  deferral: Step 2's family list already carries all five keywords in family 1, Step 1's
  extraction is unconditional, and the `KEYWORD_CLAUSES` sketch shows no placeholder. The
  dependency that survives is upstream, in the task the deferral used to wait for — **N2**.

---

## 4. Exit gate, criterion by criterion

"By this phase" = reachable with a parser plus the two oracle binaries, no executor.
"As written" = reachable by an implementer following the plan's own task interfaces.

| # | criterion | by this phase | as written | why |
|---|---|---|---|---|
| 1 | every `corpus/lang/` program parses **and** tiles | **Yes** | **Yes** | I1(new) closed the last hole: property 2 is over `clause_span`, the reason it is shape-independent is stated, and interstitial whitespace is permitted, which is what the mid-line split requires. |
| 2 | every `Instruction`/`Expr` variant constructed, by enumeration | **Yes** | **Yes** | Unchanged, and still the strongest it has been. `samples/` primary, corpus residual. |
| 3 | every `.rex` under `samples/` round-trips — 301 files, 67,519 lines | **Yes** | **Yes** | Unchanged. `find`, not the 36-file glob. |
| 4 | `CoreClasses.orx` + `StreamClasses.orx` parse end to end | **Yes** | **Yes** | Unchanged. |
| 5 | parse-time error number, sub-number, line and substitutions match | **Yes** | **Yes** | Unchanged; `rexxc` defines the comparison set. |
| 6 | `SOURCELINE(n)` for every line of every corpus program | **Yes** | **Yes** | Unchanged; driver verified in round 5. |
| 7 | `TRACE`'s `*-*` lines reconstructible for `trace_output.rex` + two probes | **Yes** | **Yes, once N2 lands** | `split_before` can now produce all three spans of a mid-line `then`, and the acceptance is per-line rather than per-sequence, which is what makes a loop and a skipped `SELECT` `END` survivable. It still needs an `Instruction` to hold the `end` clause's span, and Step 3b does not yet require one. N4 bounds the claim to non-continued clauses, which every gated file is. |
| 8 | throughput on 5,203 lines recorded against ~55 ms | **Yes** | **Yes** | Unchanged. |
| 9 | clippy clean, zero `unsafe` | **Yes** | **Yes** | Unchanged; still a local claim, no CI on the Rust tree. |
| 10 | Phase 2's twelve differential sets still at 0 — 128,368 cases | **Yes** | **Yes** | Unchanged. |

**Tally: 10 of 10 satisfiable by this phase; 10 of 10 as written once N2's one-line edit
lands, 9 of 10 without it.**
Criterion 1, which failed "as written" in each of the last two rounds, is now clean.

---

## 5. Fix order

1. **N2** — add `END` and `WHEN` to Step 3b's constraint and to the matching Notes entry, and
   give the reason as Task 3.6 Step 4's per-keyword assertion plus criterion 6's `then` and
   `end`. Must precede Task 3.1, which is the first task executed.
2. **N1** — one sentence in Task 3.4 line 647: derivability holds for the clauses
   `split_clauses` produces, not for a clause the cursor cut. Must precede Task 3.4.
3. **N3–N7** in place, in any order. **N8** at the implementer's discretion.

No further review round is warranted.
Nothing above needs investigation: every number, citation and byte position is in this file.
