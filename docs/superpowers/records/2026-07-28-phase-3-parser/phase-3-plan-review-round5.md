# Phase 3 parser plan — fifth pass

Target: `docs/superpowers/plans/2026-07-28-phase-3-parser.md` at `a2924ee3`.
Also reviewed: `docs/superpowers/plans/2026-07-27-rust-rewrite.md` (parent, edited in `ca4a7525`).
Reviewed: 2026-07-28. Oracle: `build/bin/rexx` and `build/bin/rexxc` in this worktree.
Prior rounds: `.superpowers/sdd/phase-3-plan-review.md` (rounds 1–3),
`.superpowers/sdd/phase-3-plan-review-round4.md` (round 4).

Every finding is tagged **VERIFIED** (I ran it, or read the C++) or **PLAUSIBLE**.
Probes live in the session scratchpad.

---

## Verdict

**NO-GO** — one Critical, two Important, five Minor.

All sixteen round-4 findings were addressed, and fifteen of them cleanly.
Every new number the fix round introduced is exact: I re-measured the nine-row
bare-keyword table, the directive arithmetic, the marker counts, the
`::RESOURCE` terminator, the `samples/` figures, the `SOURCELINE` driver and
every C++ line citation, and not one is wrong.
That is a real improvement on the previous four rounds, where the numbers
themselves kept moving.

The recurrence is the named one, in its purest form yet.
C1's fix added a `ClauseCursor::split_before` implementation to Task 3.6 as
copy-ready code.
Task 3.4's rule 4, three hundred lines above it, correctly describes the C++ as
doing **two independent** adjustments — the finished instruction narrows its own
end, and the new clause moves its own start — and `split_before` collapses them
into a **single partition point**.
A partition cannot reproduce the oracle, which leaves the bytes between the two
adjustments belonging to no clause at all.
So the correction landed in the prose and the code beneath it asserts the
opposite, for the fifth round running.

The fix is roughly six lines and needs no new investigation: the measurements
are all below.

---

## 1. Round-4 findings, one verdict each

| # | finding | verdict |
|---|---|---|
| C1 | nothing produces the clause spans criterion 6 needs | **fixed, and the fix introduced a new Critical** — all four named tasks plus two more now carry it, and `Clause::span`, `Instruction::clause_span` and Task 3.9's Files list are all right; but the `split_before` code the fix added cannot produce the oracle's bytes. See §2. |
| I1 | criterion 1 contradicted its own justification | **fixed** — restated as two separable properties, interstices now *permitted* and constrained to whitespace-class bytes. The continuation parenthetical is gone and replaced with the correct statement. A residual ambiguity remains about *which* span: §3, I1(new). |
| I2 | the significant-blank rule was never stated | **fixed** — stated two-sidedly in Task 3.3 with `Scanner.cpp:726`, `:755–771`, `Token.hpp:595–596`, all three citations VERIFIED at those lines. Both broken assertions corrected, and I re-derived all five Step 2 tests against the C++ rule: `a -- b` → `[Symbol, Eoc]` ✓, `a - b` → `[Symbol, Operator, Symbol, Eoc]` ✓, `f (x)` vs `f(x)` ✓, both continuation cases ✓ (`say "a"-`/`"b"` prints `a b`, `say "a"\|\|-`/`"b"` prints `ab`, re-measured). |
| I3 | directive arithmetic did not add up | **fixed, exactly** — VERIFIED: the two anchored extractions in Step 1 return **9** and **40** when run; the enum has **11** members (`Token.hpp:335–345`); **41** distinct `SUBDIRECTIVE_*`; **7** shared suffixes (`ATTRIBUTE CLASS CONSTANT LIBRARY METHOD NONE ROUTINE`); 11+41−7 = 45. "There is no set of 36 anywhere" is now stated, and the assert is 9 and 40. |
| I4 | `parse_interpret` dropped its source | **fixed** — returns `Fragment { source, instructions }`, named consistently in the file-structure block, the signature, the prose, Task 3.9's Interfaces and the Notes. |
| I5 | two of seven error numbers are not parse errors | **fixed, exactly** — VERIFIED all nine rows by running both binaries: `then` 8.1/rc 248, `else` 8.2/248, `when` 9.1/247, `otherwise` 9.2/247, `end` 10.1/246, `parse` 20.903/236 identical under both; `procedure` rexxc **rc 0** / rexx 17.1 rc 239, `leave` rc 0 / 28.1 rc 228, `iterate` rc 0 / 28.2 rc 228. `RexxActivation.cpp:1161`/`:1214`/`:1250` all carry the cited `reportException` calls. `KEYWORD_CLAUSES` gains the three bare rows with a comment forbidding wrapping them. |
| I6 | `rexxc` never mentioned | **fixed** — adopted in Task 3.1 axis 2, Task 3.3 Step 5, Task 3.7 Step 3, Task 3.8 Step 1, gate criterion 4 and a Notes entry, with the two stream-routing recipes. The `::END` claim is VERIFIED: lowercase `::end` gives `Error 99` / `99.943 Missing ::RESOURCE end marker "::END"` at **rc 157**; uppercase gives rc 0. |
| I7 | two parent criteria silently dropped | **fixed** — `samples/` adopted as its own criterion using `find`, not the glob, with an explicit "NOT samples/*.rex" comment. Parent row 402 now reads "under `samples/`… (301 files)" and "error **line** reporting"; lines 376, 2360 and 2402 all corrected, 2402 now distinguishing executing the samples from parsing them. One residual: §3, M5. |
| M1 | Step 4 asserted a column exists | **fixed** — rewritten around `'…'x`/`'…'b`, round-tripping and byte-indexed spans, with "there are **no column numbers in error messages**". |
| M2 | `TokenCursor` used, never defined | **fixed** — defined in Task 3.3 beside `ParseCtx`, named in the file-structure block and in Task 3.5's Interfaces, built with `TokenCursor::new(clause.tokens.clone())` in both places. `advance` not `next`, with the clippy reason. The struct-literal `Self { pos: range.start, range }` compiles (field order evaluates `range.start` before the move). |
| M3 | Task 3.9's Files contradicted its steps | **fixed** — `src/clause.rs` and `src/ast.rs` added, with a sentence saying why. One omission the new Critical exposes: §3, M3. |
| M4 | criterion 5 had no oracle-side method | **fixed** — driver included. VERIFIED by running it: `SRCLINES: 27` and 27 `SRC` lines for `do_variants.rex`, matching `wc -l`. The prolog-executes caveat and the prefix filter are both stated. |
| M5 | "239 expected trace lines" did not reproduce | **fixed** — VERIFIED: `TRACE.testGroup` is **1,338** lines, **135** carry `*-*`, **243** carry a value marker. All three now in the plan. |
| M6 | value-marker list incomplete | **fixed** — phrased "every marker except `*-*`" in both Task 3.9 and criterion 6. VERIFIED the enumeration: exactly **15** distinct value markers in that file (`>L>` 58, `>>>` 49, `>K>` 33, `>I>` 23, `>=>` 23, `>A>` 20, `>V>` 17, `>O>` 9, `>M>` 4, `>F>` 4, `>E>` 4, `>R>` 3, `>N>` 3, `>C>` 3, `>P>` 1). `>K>` 33 times ✓, and `>K> "TO" => "2"` reproduces from `do i = 1 to 2` ✓. |
| M7 | "Phase 3 owns the *formatting*" | **fixed** — sentence gone; grep for "formatting of traced" returns nothing, and the parent's row 402 dropped "TRACE output formatting" in the same round, so the two documents agree. |
| M8 | criterion 2 made "All 14" stale | **fixed** — criterion 1 says "Every program in `rust/corpus/lang/`", Task 3.7b Step 4 says count the directory. |
| M9 | Step 5 staged an unchanged file | **fixed** — stages only `d10-decision.md`, with `members = ["crates/*"]` as the reason. |

**16 of 16 addressed. 15 clean. C1's fix introduced a new Critical.**

---

## 2. Critical

### C1(new) — `split_before` is a partition, and the oracle is not

**VERIFIED** against `build/bin/rexx` at three whitespace widths and three
keywords, and against the C++.

Task 3.6 hands the implementer this, as code to copy:

> ```rust
> pub fn split_before(&mut self, ctx: &ParseCtx, at: usize) -> Clause {
>     …
>     let cut = ctx.tokens[at].span.start;
>     self.pending = Some(Clause { tokens: at..cur.tokens.end, span: cut..cur.span.end, label: None });
>     Clause { tokens: cur.tokens.start..at, span: cur.span.start..cut, label: cur.label }
> }
> ```

One `cut`, used as both the finished clause's end and the remainder's start.
Every byte of the original clause lands on one side or the other.

**The oracle assigns some bytes to neither side.** Three probes, deliberately
varying the whitespace width, which is the dimension the plan gives no reason to
think matters:

```
source:  if 6 > 5 then   say "big"        (three blanks before `say`)
trace:        2 *-* if 6 > 5 $            <- one trailing blank
              2 *-*   then$               <- NO trailing blank
              2 *-*     say "big"$        <- NO leading blank, though the source has three

source:  when 1 = 1   then    say "w"     (three before `then`, four before `say`)
trace:        3 *-*   when 1 = 1   $      <- THREE trailing blanks
              3 *-*     then$             <- none
              3 *-*       say "w"$        <- none, though the source has four

source:  otherwise   say "o"     /  else   say "b"
trace:        4 *-*   otherwise$   /   3 *-*   else$      <- keyword alone, both
              4 *-*     say "o"$   /   3 *-*     say "b"$ <- no leading blank
```

The indentation after `*-*` is the trace's own, two spaces per nesting level
(`if` 0, `then` 2, `say` 4; `select` 0, `when` 2, `then` 4, `say` 6), which is
why it does not track the source's blank count. The clause text itself starts
exactly at its first token and, for `THEN`/`ELSE`/`OTHERWISE`, ends exactly at
its last.

Task 3.4's rule 4 already says why, correctly:

> `RexxClause::trim` (`Clause.cpp:138`) moves the clause's *start* forward to the
> current token and leaves the end alone, and the instruction that just ended
> narrows its own end separately

Both halves VERIFIED. `RexxClause::trim` at `Clause.cpp:138` does
`first = current; clauseLocation.setStart(l);` — start only.
`RexxInstructionIf` at `IfInstruction.cpp:58–66` does
`setEnd(location.getLineNumber(), location.getOffset())` from the THEN token,
i.e. the THEN's **start**, which is why the condition clause keeps its trailing
blanks.
`RexxInstructionThen` at `ThenInstruction.cpp:76` does
`setLocation(token->getLocation())` — the THEN token's **whole** location, both
ends, which is why `then` has no blank on either side.
Those are two independent adjustments with a gap between them.

**The arithmetic, for `if 6 > 5 then say "big"`.** Tokens (blanks per the rule
Task 3.3 now states): `[0]if [1]Blank [2]6 [3]> [4]5 [5]Blank [6]then [7]Blank
[8]say [9]Blank [10]"big" [11]Eoc`.

* `split_before(6)` → finished `if 6 > 5 ` ✓, pending starts at `then` ✓.
* Then, for the THEN instruction, there are only two choices and both are wrong:
  * `split_before(7)` → finished `then` ✓, pending ` say "big"` ✗ (leading blank).
  * `split_before(8)` → finished `then ` ✗ (trailing blank), pending `say "big"` ✓.

In the `when 1 = 1   then    say "w"` probe the discrepancy is four bytes, so it
is not a one-byte curiosity.

**Task 3.6 contradicts itself two paragraphs later**, which is the shape this
plan keeps producing:

> **Every `Instruction` carries a `clause_span: Range<usize>`**, copied from the
> `Clause` that `next_clause` or `split_before` returned. … It is not the node's
> own extent: a `THEN` is its own `Instruction` whose `clause_span` covers just
> the `then` token

"Copied from what `split_before` returned" and "covers just the `then` token"
cannot both hold, per the arithmetic above.

**Cost of leaving it.** Gate criterion 6 and Task 3.9's acceptance are
byte-identical comparisons; they fail on one of the two `*-*` lines whichever
call the implementer makes. Task 3.9's Files list now (correctly) includes
`src/clause.rs`, so the `split_before` half is repairable there — but the repair
also requires `THEN`/`ELSE`/`OTHERWISE` to set their own `clause_span` end, which
is in `src/instruction.rs`, not in Task 3.9's Files list. So the escape hatch
that C1's fix built reaches most of this defect and not all of it.

**Fix (no further investigation needed).** Replace the single `cut` with two
positions, and say in one sentence that the bytes between them belong to no
clause:

- `split_before(ctx, at)` returns a clause whose `span` ends at an **end byte the
  caller supplies**, and re-presents the remainder starting at
  `ctx.tokens[at].span.start`. Two arguments, not one.
- `IF`/`WHEN` pass the THEN token's **start** byte (so the condition clause keeps
  its trailing blanks — `IfInstruction.cpp:58–66`).
- `THEN`/`ELSE`/`OTHERWISE` pass their own keyword token's **end** byte (so the
  keyword clause carries no blank on either side —
  `ThenInstruction.cpp:76`).
- State the invariant: interstitial blanks belong to no clause, which is exactly
  what gate criterion 1's property 2 already permits.
- Add `src/instruction.rs` to Task 3.9's Files list (M3).

---

## 3. Important

### I1(new) — criterion 1 says "instruction spans" without saying which span

**VERIFIED** by reading it against the rest of the plan.

> 2. **Instructions are ordered.** Consecutive instruction spans are in source
>    order and do not overlap, and the only bytes between one instruction's span
>    and the next are whitespace, comments and `,`/`-` continuations.

And the justification below:

> Property 1 is stated for expressions and property 2 for instructions on
> purpose, because Task 3.1 Step 3b may make instructions a flat chain rather
> than a tree, in which case they are siblings and containment does not apply to
> them. **The criterion holds either way.**

It does not hold either way, unless "instruction span" means
`Instruction::clause_span`. Under Step 3b's tree outcome with node extents, a
`Do` node's span contains its body instructions' spans, so consecutive
instruction spans **do** overlap and property 2 fails on every loop in the
corpus. The whole point of C1 was that a clause span and a node span are
different things, and this criterion — now on its fifth revision — is the one
place that distinction is not made.

The whitespace clause is right, and the C1(new) evidence confirms it: the bytes
between `then` and `say "big"` are blanks, which property 2 already permits.

**Should say:** "Consecutive `Instruction::clause_span`s are in source order and
do not overlap", and the justification should say the criterion is stated over
clause spans *because* those are per-clause under both Step 3b shapes, which is
what makes it shape-independent.

### I2(new) — criterion 6 forces Step 3b's answer, and no task says so

**VERIFIED** by reading Task 3.1, Task 3.6, Task 3.7b and criterion 6 as a
sequence.

Task 3.1 Step 3b presents tree-versus-chain as genuinely open. Task 3.6 says:

> **Five of the 35 rows cannot be written until Task 3.1 Step 3b lands.** `THEN`,
> `ELSE`, `END`, `WHEN` and `OTHERWISE` only exist as nodes of their own under the
> flat instruction chain; under a tree they are absorbed into their parent and
> have no node to name.

Task 3.7b then closes the only other place a span could live:

> So `Program` needs no separate clause list: the spans travel with the
> instructions

And criterion 6 requires the `then` line to be reconstructible from
`Instruction::clause_span`. Compose those three and the tree outcome is
unsatisfiable: absorbed keywords have no `Instruction`, `Program` has no clause
list, so the `then` clause's bytes have nowhere to live and criterion 6 cannot be
met. Confirmed against the C++, which does make `THEN` a real instruction
(`RexxInstructionThen`, its own `*-*` line in every probe above).

Two smaller consequences of the same gap. Task 3.6 Step 4's
`assert_eq!(KEYWORD_CLAUSES.len(), 35)` is unsatisfiable under the tree outcome,
since five keywords would have no node for `parses_to_node_named` to find. And
the sentence "cannot be written until Task 3.1 Step 3b lands" is inert as
ordering advice — 3.1 is the first task, so the shape is always known by 3.6.

**Should say,** in Task 3.1 Step 3b: whichever shape wins, `THEN`, `ELSE` and
`OTHERWISE` must remain instructions of their own carrying their own
`clause_span`, because the oracle traces each as a separate `*-*` clause
(`ThenInstruction.cpp:76`) and criterion 6 gates on it. That is a constraint on
the decision, not the decision itself, and it is worth two sentences at the point
where the choice is made rather than nine tasks later.

---

## 4. Minor

Each is a line or two, and an implementer absorbs them in passing.

**M1 — Task 3.3's `Eoc` model omits end of file.** *"`scan` emits one `Eoc` at
each clause terminator: an explicit `;`, or an end of line that is not
continued."* All five of the task's own Step 2 tests pass strings with no
trailing newline and expect a final `Eoc`, and Task 3.4's rule 1 says "at `;`, at
an uncontinued end of line, **or at end of file**". Add the third terminator to
the model so it matches the tests immediately below it.

**M2 — Task 3.9 Step 1's probe A description is incomplete, and the acceptance
reads as a sequence comparison.** The plan says probe A traces `nop;`,
`do i = 1 to 2;` and `say i;`. VERIFIED, and it also traces `end`, and `trace r`
re-traces the loop header, body and `end` once per iteration: **six** `*-*` lines
for **four** clauses on source line 3. The acceptance — "for every clause … the
text reconstructed from the AST is byte-identical to the corresponding `*-*`
line" — needs one clarifying clause: every traced line must equal the clause it
came from, not the two sequences must be the same length.

**M3 — Task 3.9's Files list omits `src/instruction.rs`.** The C1(new) repair
needs `THEN`/`ELSE`/`OTHERWISE` to set their own `clause_span` end, which is
instruction-parser code. Fix with C1(new).

**M4 — Task 3.9's Interfaces mis-attributes `clause_span`.** *"Consumes:
`Instruction::clause_span`, produced by Task 3.4 and split by Task 3.6."* Task
3.4 produces `Clause::span`; `Instruction::clause_span` is introduced in Task
3.6. Harmless once C1's naming is consistent everywhere, which it now is, but a
reader chasing the field to Task 3.4 will not find it.

**M5 — the plan still narrows one parent criterion without saying so.** Parent
row 402 now reads "`TRACE`'s `*-*` source lines match the oracle byte-for-byte",
unscoped; criterion 6 scopes that to `trace_output.rex` plus two probes.
Narrowing is the right call — reconstructing every clause of all 301 samples buys
little over three files that cover every rule — but round 4's I7 established the
principle that a phase plan narrowing its parent's gate must record it, and this
is the last place it does not.

---

## 5. New claims introduced by the fix round: all verified

Everything the fixer added, measured this session. Nothing wrong.

**C++ citations, all at the cited lines:** `LanguageParser.cpp:1009`
(`nextClause`), `:1072` (`location.setEnd(tokenLocation)`), `:1378`/`:1403`/
`:1465`/`:1494` (all four `trimClause()` calls, and all four are
`previousToken(); trimClause();` for THEN, THEN, ELSE and OTHERWISE
respectively); `InstructionParser.cpp:173–174` (`trimClause(); reclaimClause();`);
`Clause.cpp:138` (`RexxClause::trim`); `IfInstruction.cpp:58–66`;
`ThenInstruction.cpp:58–77` (`setLocation` at :76); `Scanner.cpp:726`
(`previous->isBlankSignificant()`), `:755–771` (the two-sided
`SIGNIFICANT_BLANK` check), `:342–348` (continuation returns
`SIGNIFICANT_BLANK`), `:271` (`locateToken`), `:313–317`
(`truncateLine(); return CLAUSE_EOL`); `Token.hpp:595–596`
(`isBlankSignificant`), `:333–346` (the `DirectiveKeyword` enum, 11 members at
335–345); `KeywordConstants.cpp:52` (`directives[]`), `:363–405`
(`subDirectives[]`), `:417` (`resolveKeyword`); `RexxActivation.cpp:1161`
(`Error_Invalid_leave_iterate`), `:1214` (`Error_Invalid_leave_leave`), `:1250`
(`Error_Unexpected_procedure_call`).

**Counts:** anchored `directives[]` extraction returns **9**, `subDirectives[]`
**40**, both by running the plan's own `sed`/`grep` commands; 41 distinct
`SUBDIRECTIVE_*`; 7 shared suffixes; `keywordInstructions[]` 35; `subKeywords[]`
50; `conditionKeywords[]` 12; `parseOptions[]` 10; 36 distinct `KEYWORD_*` in
`KeywordConstants.cpp` (the pre-existing "36 keyword constants" claim is right);
`samples/*.rex` glob **36**, `find samples -name '*.rex'` **301**, **67,519**
lines; `rust/corpus/lang/` **14**; `do_variants.rex` **27** lines and **27**
`~source` items; `TRACE.testGroup` 1,338 / 135 / 243 and 15 distinct value
markers.

**Behaviour:** the nine-row bare-keyword table, every rc and sub-number;
`x = 1/0` rexxc rc 0 / rexx 42.3 (criterion 4 cites 42.3 — correct);
`::end` 99.943 rc 157 versus `::END` rc 0; the `SOURCELINE` driver runs and
returns 27; both continuation cases print `a b` and `ab`; `>K> "TO" => "2"`
reproduces.

**Axis 4:** `chumsky` 0.13.0's `default = ["std", "stacker"]` VERIFIED in the
offline registry, and `psm`'s `[build-dependencies.cc]` VERIFIED in 0.1.16,
0.1.30 and 0.1.31. The "28 transitive packages" figure is **PLAUSIBLE** —
not independently derivable offline without a resolve — and nothing gates on it.

**The Rexx facts the earlier drafts got wrong all still hold in the current
text:** keywords not reserved and recognition positional (Task 3.6 Step 2, Notes);
`-2 ** 2` is 4 (Task 3.5 Step 1); `f(x)` versus `f (x)` (Task 3.1 Step 1, Task
3.3's blank rule and its test); no column anywhere in the oracle (Task 3.3 Step 4,
Task 3.8 Step 4, criterion 4, and now the parent at lines 376, 402 and 2360 —
with an internal column deliberately retained in `ProgramSource::position` and
explicitly not gated on); positional versus alphabetical tables (Task 3.6 and the
Notes, both directions stated).

**Cross-task naming, scoped per task:** `clause_span` in eight places, one
spelling; `advance` only, no surviving `next`; `TokenCursor` in Task 3.3, 3.5 and
the file-structure block; `Fragment` in six places; `split_before` in six.
No stale name anywhere.

---

## 6. Exit gate, criterion by criterion

"By this phase" = reachable with a parser plus the two oracle binaries, no
executor. "As written" = reachable by an implementer following the plan's own
task interfaces.

| # | criterion | by this phase | as written | why |
|---|---|---|---|---|
| 1 | every `corpus/lang/` program parses **and** tiles | **Yes** | **No** | Parsing: all 14 pass `rexxc`. Tiling is now correctly stated as containment for expressions plus ordering for instructions, with whitespace-only interstices — but "instruction spans" is undefined between `clause_span` and node extent, and under Step 3b's tree outcome the second reading fails on every `DO`. I1(new). |
| 2 | every `Instruction`/`Expr` variant constructed, by enumeration | **Yes** | **Yes** | Parser-only. Now run over corpus **and** `samples/` with `samples/` named primary, which is what makes 52 instruction and 17 expression classes reachable honestly. |
| 3 | every `.rex` under `samples/` round-trips — 301 files, 67,519 lines | **Yes** | **Yes** | Adopted from the parent. Uses `find`, with an explicit warning that the glob is only 36. Oracle side is one loop and the expected answer is "parses" for all 301. |
| 4 | `CoreClasses.orx` + `StreamClasses.orx` parse end to end | **Yes** | **Yes** | 5,203 lines; `rexxc` confirms either alone. |
| 5 | parse-time error number, sub-number, line and substitutions match | **Yes** | **Yes** | "Parse-time" is now defined by `rexxc` rather than by judgement, with the three runtime counter-examples named and their numbers verified (17.1, 28.1, 42.3). The comparison set is well-defined. |
| 6 | `SOURCELINE(n)` for every line of every corpus program | **Yes** | **Yes** | Driver included and verified to run: 27 items for `do_variants.rex`, prolog caveat and prefix filter both stated. |
| 7 | `TRACE`'s `*-*` lines reconstructible for `trace_output.rex` + two probes | **Yes** | **No** | A parser can do it; the data is static. But `split_before` as specified cannot produce both `then` and `say "big"` (C1(new)), and under Step 3b's tree outcome there is no `Instruction` to hold the `then` span at all (I2(new)). Also needs M2's one-clause clarification, since `trace r` prints six lines for four clauses. |
| 8 | throughput on 5,203 lines recorded against ~55 ms, plainly stated | **Yes** | **Yes** | Asks for a number and an honest statement; bench settings match `perf-baseline.md`. |
| 9 | clippy clean, zero `unsafe` | **Yes** | **Yes** | `unsafe_code = "forbid"` at `[workspace.lints.rust]`. `phase-2-gate.md`'s standing qualification still applies: no CI runs the Rust tree, so this remains a local claim. |
| 10 | Phase 2's twelve differential sets still at 0 — 128,368 cases | **Yes** | **Yes** | Unchanged. |

**Tally: 10 of 10 satisfiable by this phase; 8 of 10 as written.** Both failures
are criterion 1's wording and criterion 6's interface, and both trace to the same
place: whether a clause span is a first-class thing an `Instruction` owns
independently of its node extent. The plan now says it is, in seven places out of
eight.

Adopting `samples/` is a genuine strengthening — criterion 2 was the weakest
criterion in round 4 and is now the second strongest.

---

## 7. Fix order

1. **C1(new)** — give `split_before` two byte positions instead of one; state
   that interstitial blanks belong to no clause; have `IF`/`WHEN` pass the THEN
   token's start and `THEN`/`ELSE`/`OTHERWISE` their own token's end. Add
   `src/instruction.rs` to Task 3.9's Files (M3). This must precede Task 3.6.
2. **I2(new)** — two sentences in Task 3.1 Step 3b constraining the shape
   decision, and drop the inert "cannot be written until Step 3b lands".
3. **I1(new)** — one word in criterion 1: `clause_span`.
4. **M1, M2, M4, M5** as edits in place.

Nothing above needs new investigation. Every byte offset, line citation and
count the fixes need is in this file or in §5.
