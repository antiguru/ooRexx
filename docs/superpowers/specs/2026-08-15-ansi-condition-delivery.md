# DO/END clause boundaries and condition delivery, against ANSI X3.274

**What this is for.** Phase 7 owns every condition the standard delays to a clause boundary, because
`ERROR` and `FAILURE` need command issuance and `NOTREADY` needs streams. This document is the read
of the standard that phase designs against, taken while the `DO` clause-boundary work was fresh; the
roadmap's Phase 7 entry summarises its consequences and points here for the quotes.

**The bar for this project remains ooRexx, not ANSI.** Where the two disagree, this crate matches
ooRexx and the disagreement is recorded rather than treated as a defect. Known disagreements: 8.2.4
drains a boundary where ooRexx and this crate deliver one condition and do not re-check; and
8.3.6/8.3.6.1 **read as** putting a `DO UNTIL`'s test in the re-entered `DO` clause, where both
interpreters attribute it to `END` -- that second one is inference from the notation plus the tracing
rules, not from any sentence saying so, and Q5 below says exactly how far the text goes.

**Scope limit, stated first because it is easy to miss.** The transcripts that prompted this read use
`RAISE` and a `USER` condition. Neither exists in the standard, so those programs are rejected before
they run and say nothing about conformance either way. The findings below bind the conditions ANSI
does define.

## Sources actually read

* `https://www.rexxla.org/rexxlang/standards/j18pub.pdf`, 414,740 bytes, the X3J18 pre-publication
  draft. `pdftotext` (no `-layout`) gives 11,786 lines; `-layout` gives 9,091. Page citations below
  are the document's own printed page numbers, taken from the page-break markers in the text
  conversion. (`pdfinfo` reports 181 pages for this file, not 10; the warning in the brief did not
  reproduce with this poppler build. Either way the page count is not the document's own paging.)
* The errata, `https://www.rexxla.org/rexxlang/standards/stan_err.html`. **I read it in full.** Its
  effect is recorded under "Errata" below.

Everything quoted below is from the draft, not the published standard. rexxla's claim of functional
equivalence is taken on trust, not verified.

## Threshold finding: the transcripts are not ANSI programs

The standard has **no `RAISE` instruction** and **no `USER` condition**. The grammar for `CALL ON` is
exhaustive (6.3.2.38, 6.3.2.39, p. 35):

> ```
>   callon_spec      := 'ON' (callable_condition | Msg25.1)             (6.3.2.38)
>                    ['NAME' (taken_constant | Msg19.3)]
>                    | 'OFF' (callable_condition | Msg25.2)
>      callable_condition:= 'ERROR' | 'FAILURE'                         (6.3.2.39)
>                    | 'HALT' | 'NOTREADY'
> ```

`RAISE` does not appear in `keyword_instruction` (6.3.2.9) and the string "RAISE" occurs in the
document only as the notation function `#Raise`. So `call on user zx name h` is Msg25.1 ("CALL ON
must be followed by one of the ...", 8.2.1) and `raise user zx return 5` is a syntax error under the
standard. Every one of transcripts A-D is rejected before it runs.

Consequently the honest verdict for A, B, C and D is **NOT ADDRESSED**, and I say so again below.
What follows is the standard's machinery for the four conditions it *does* define, which is the only
thing that can be measured against these transcripts, plus the places where the crate's behaviour
looks wrong for the ANSI conditions themselves.

---

## Q1. Are DO and END clauses in their own right?

**Answer: clauses in their own right. The whole group is one *instruction*, not one clause.**

Definition (3.1.5, p. 1):

> "**clause**: A section of the program, ended by a semicolon. The semicolon may be implied by the
> end of a line or by some other constructs."

Definition (3.1.30, p. 2):

> "**instruction**: One or more clauses that describe some course of action to be taken by the
> language processor."

Grammar (6.3.2.11-6.3.2.13, p. 35):

> ```
> group             := do | if | select                               (6.3.2.11)
>   do              := do_specification (ncl | Msg21.1 | Msg27.1)     (6.3.2.12)
>                   [instruction_list] do_ending
>      do_ending    := 'END' [VAR_SYMBOL] ncl                         (6.3.2.13)
> ```

with `ncl := null_clause+` (6.3.2.2) and `null_clause := ';' [label_list]` (6.3.2.3). The `ncl` after
`do_specification` and the `ncl` after `'END'` are exactly the semicolons that 3.1.5 makes clause
terminators. So `DO` is a clause, `END` is a clause, and `instruction := group | ...` (6.3.2.6) makes
`DO...END` a single instruction spanning them.

Confirmed independently by 8.2.4 (p. 73), which enumerates clauses that do not pause in interactive
trace:

> "When tracing interactively, pauses occur after the execution of each clause except for CALL, DO
> the second or subsequent time around the loop, END, ELSE, EXIT, ITERATE, LEAVE, OTHERWISE, RETURN,
> SIGNAL, THEN and null clauses."

That sentence presupposes DO and END are clauses that *are executed* and *reach* clause termination;
they are exempted only from the pause, not from the boundary. (8.3.4 makes the same point for labels,
p. 83: "The execution of a label has no effect, other than clause termination activity and any
tracing.")

## Q2. When is a `CALL ON` condition delivered?

**Answer: at the end of the clause in which the condition arose.** 8.4.1 (p. 96):

> "/\* All CALL actions are initially delayed until a clause boundary. \*/"

and, at the head of `#Raise` (p. 95):

> "/\* If there is no argument, this is an action which has been delayed from the time the condition
> occurred until an appropriate clause boundary. \*/"

The boundary itself is 8.2.4 "Clause termination" (p. 72):

> "At the end of each clause there is a check for conditions which occurred and were delayed. It is
> acted on if this is the clause in which the condition arose.
> ```
> do t=1 to 4
> #Condition=WORD('HALT FAILURE ERROR NOTREADY',t)
> /* HALT can be established during HALT handling. */
> do while #PendingNow.#Condition.#Level
> #PendingNow.#Condition.#Level = '0'
> call #Raise
> end
> end
> ```"

Non-normative confirmation, A.8.4.1 (p. 152):

> "Note that delayed conditions are raised at the end of the clause in which they occurred."

Contrast, 8.4.1 (p. 96): "/\* SIGNAL actions occur as soon as the condition is raised. \*/" -- `SIGNAL
ON` does not wait for a boundary. Only `CALL ON` does.

Note the scope: the boundary loop covers **HALT, FAILURE, ERROR, NOTREADY only**. SYNTAX, NOVALUE and
LOSTDIGITS are `SIGNAL`-only under 6.3.2.39/6.3.2.68 and never delayed.

## Q3. One per boundary, or drain?

**Answer: the standard drains, and explicitly permits more than one delivery at a single boundary.**

The 8.2.4 code quoted above is a `do t=1 to 4` over the four condition names, each with an inner
`do while #PendingNow...`. So at one clause boundary the standard delivers, in the fixed order
**HALT, FAILURE, ERROR, NOTREADY**, every condition that is pending at the current level. It is not
"at most one per boundary".

The inner `do while` is a genuine drain only for HALT -- the comment says so ("HALT can be established
during HALT handling"), and 8.4.1 (p. 96) explains why the other three cannot re-arm themselves:

> "/\* Events within the handler are not stacked up, except for one extra HALT while a first is being
> handled. \*/
> ```
> EventLevel = #Level
> if #Enabling.#Condition.#Level == 'DELAYED' then do
> if #Condition \== 'HALT' then return
> ```"

`#Enabling` is set to `'DELAYED'` when the event is recorded and back to `'ON'` only after
`interpret 'CALL' #TrapName...` returns, so a *same-name* re-occurrence inside a handler is silently
dropped. A *different-name* condition raised inside a handler is recorded normally, and is delivered
at the same boundary if its name comes later in `HALT FAILURE ERROR NOTREADY`, otherwise at the next
boundary.

**Internal tension worth knowing about.** 8.2.4's prose says delivery happens only "if this is the
clause in which the condition arose", but its code tests a per-*level* flag with no record of which
clause set it. For a condition raised by a handler that is itself running at a boundary, the two
readings disagree about whether the later condition may slip to the following clause. The standard
does not resolve this.

## Q4. What is SIGL when a `CALL ON` handler is entered?

**Answer: the line of the clause in which the condition occurred.** 7.5.3 "The value of a label"
(p. 59):

> "Whenever a matching label is found, the variables SIGL and .SIGL are assigned the value of the line
> number of the clause which caused the search for the label. In the case of an invocation resulting
> from a condition occurring that shall be the clause in which the condition occurred.
> ```
> Var_Set(#Pool, 'SIGL', '0', #LineNumber)
> Var_Set(0 , '.SIGL', '0', #LineNumber)
> ```"

The prose and the code agree *only because* Q2 puts delivery at the end of the originating clause:
`#LineNumber` is set at clause initialization (8.2.3, p. 72, "The state variable #LineNumber is set to
the line number of the clause") and is still the originating clause's value during that clause's
termination. Where the two can be pulled apart -- delivery at a *later* clause's boundary -- the
normative sentence is the prose: **the clause in which the condition occurred**, not the clause at
whose boundary the handler was entered. This is the crux of the adversarial section below.

## Q5. Does `DO UNTIL`'s test belong to the END clause?

**Answer: the standard says the opposite -- it belongs to the re-executed DO clause, and is evaluated
after the END clause has finished.** 8.3.6 (pp. 80-81) lays the loop out as straight-line notation:

> ```
> IterateLabel:
> if #Contains(do_specification, untilexpr) then do
> Value = #Evaluate(untilexp, expression)
> ...
> #Execute(do_instruction, instruction_list)
> TraceOfEnd:
> call #Goto #Iterate.#Loop /* to IterateLabel */
> ```

and 8.3.6.1 "DO loop tracing" (p. 81) pins those labels to clauses:

> "the DO instruction shall be traced when it is encountered and again each time the IterateLabel (see
> section 8.3.6) is encountered. The END instruction shall be traced when the TraceOfEnd label is
> encountered."

Tracing the source of a clause happens at clause initialization (8.2.3, p. 72: "The clause is traced
before execution"), and that is the same step that sets `#LineNumber`. So the required order per
iteration is: body, **END clause** (traced at `TraceOfEnd`), END's clause termination, **DO clause
re-entered** (traced at `IterateLabel`, `#LineNumber` := DO's line), *then* the `UNTIL` expression is
evaluated inside that DO clause. A condition arising in the `UNTIL` expression therefore has
`#LineNumber` = the **DO**'s line under the standard.

This is read from the notation plus 8.3.6.1, not from a sentence that says "UNTIL belongs to the DO
clause". There is no such sentence. Flagging that as inference.

Corroborating, 8.2.4's pause-exemption list exempts "DO the second or subsequent time around the
loop", which only makes sense if the DO clause is re-entered each iteration; and A.8.2.4 (p. 150):
"most expressions on a DO specification are not re-evaluated on each iteration of the loop so the
corresponding clause is not paused after, except on the first iteration."

---

## Verdicts on A, B, C, D

| | Verdict | Why |
|---|---|---|
| A | **NOT ADDRESSED** | `RAISE`/`USER` are not in the standard; the program is Msg25.1 + syntax error. |
| B | **NOT ADDRESSED** | same |
| C | **NOT ADDRESSED** | same |
| D | **NOT ADDRESSED** | same |

The *shapes* are worth separating from the verdicts, because the crate presumably uses one delivery
mechanism for both ooRexx-only and ANSI conditions:

* **A's shape -- a pending condition delivered at the DO clause's own boundary, before the body --
  is exactly what the standard's model produces** for a condition that arose *in the DO clause*
  (e.g. from a function call in a `WHILE`/`TO` expression). DO is a clause (Q1), its termination runs
  before the body's first clause (Q2). Nothing in A's shape is anomalous.
* **B's shape -- delivery at the END's own boundary -- likewise follows** from END being a clause.
* **C's shape -- output overtaking a requeued handler -- is not contradicted.** In the standard a
  condition first recorded during a handler that is itself running at clause N's boundary can be
  missed by that boundary's fixed-order sweep and picked up at clause N+1's termination, i.e. after
  clause N+1 has executed and printed. Same observable ordering.
* **D's shape contradicts the standard's attribution of the UNTIL test**, see Q5 and the adversarial
  list.

## Errata

Read in full. Three items touch this area:

* **8.3.8 and 8.3.22 (Corrections)** -- "should say 'At this point the clause termination occurs and
  then:' before the changes to #Level and #Pool. (This achieves compatibility with most existing
  implementations about whether any exceptions signaled are raised in the caller or the callee.)"
  This *adds* a clause boundary at the callee's level on `RETURN`/`EXIT`, before the level pops.
  It changes the answer to Q2 for the last clause of a routine: a condition delayed inside a callee is
  delivered **inside the callee**, at the RETURN clause's boundary, with SIGL at the callee's line.
* **8.2.4 (Corrections)** -- prepends `if #InhibitTrace > 0 then #InhibitTrace = #InhibitTrace - 1`.
  Trace bookkeeping only; no effect on condition delivery.
* **9.5.3 (Clerical)** -- `ConditionInstruction` to `ConditionInstruction.#Level`; affects the
  CONDITION built-in's state, not delivery.

Nothing in the errata touches 7.5.3 (SIGL), 8.4.1's delay rules, 6.3.2.38/39, or 8.3.6/8.3.6.1.
No errata item changes the answers to Q1, Q3, Q4 or Q5.

---

## Adversarial: where matching ooRexx may still be wrong

Each of these is checkable with **ANSI conditions only** -- no `RAISE`, no `USER` -- so they are
inside the standard's scope, unlike A-D. I did not run any of them; the brief forbade running the
interpreter.

1. **"At most one delivery per boundary" contradicts 8.2.4.** If the crate's rule from transcript C is
   general, then a clause that leaves two *different* ANSI conditions pending (say a failing command
   raising ERROR and a stream operation raising NOTREADY in the same clause) must deliver **both** at
   that clause's termination under 8.2.4's `do t=1 to 4`. A one-per-boundary engine will run the
   second handler one clause too late. This is the single most likely real divergence.
2. **Delivery order is fixed by name, not by arrival.** 8.2.4 sweeps `HALT FAILURE ERROR NOTREADY` in
   that order. An engine that delivers in raise order, or FIFO, diverges when two names are pending
   together.
3. **SIGL should name the originating clause, not the delivering one.** 7.5.3's prose is normative and
   says "the clause in which the condition occurred". Reachable without `RAISE`: let an ERROR handler
   itself issue a failing command. The second ERROR occurs at a line *inside the first handler*; ANSI
   requires the second handler's SIGL to be that line. An engine that reports `#LineNumber` at the
   moment of delivery will report the interrupted clause's line instead. Transcripts A-C show the
   crate taking SIGL from the delivery boundary (`G ran 5` where the condition was raised at line 14),
   so this is the behaviour to test.
4. **Same-name conditions must not stack.** 8.4.1: "Events within the handler are not stacked up,
   except for one extra HALT while a first is being handled." A condition of the same name occurring
   while its own handler runs is **discarded**, because `#Enabling` is `'DELAYED'` until the handler
   returns. If the crate queues it and delivers it at the next boundary, that contradicts 8.4.1. The
   HALT carve-out is exactly one extra, guarded by `if #PendingNow.#Condition.EventLevel then return`.
5. **`DO UNTIL` attribution.** Per Q5 the standard puts the UNTIL evaluation in the re-entered DO
   clause after the END clause completes, so `SIGL` would be the DO's line (4 in transcript D), not
   the END's (6). ooRexx and the crate report 6. Also check the *trace* order, which 8.3.6.1 pins
   independently of conditions: END traced at `TraceOfEnd`, then DO traced again at `IterateLabel`,
   then the UNTIL expression evaluated and traced.
6. **RETURN/EXIT carry a clause boundary at the callee's level** (errata to 8.3.8/8.3.22). A condition
   delayed by the callee's final clause must be handled in the callee, before the level pops. Worth
   a direct test: `CALL ON ERROR` established at the outer level, a callee whose last clause is a
   failing command, and check which level's handler state and which SIGL are used.
7. **`CALL ON` re-enables automatically after the handler returns** -- 8.4.1's `#Enabling.#Condition.#Level = 'ON'`
   is executed *after* `interpret 'CALL' ...`, so a condition occurring after the handler returns is
   trapped again without the program doing anything.
8. **The handler's result is discarded.** 8.4.1, p. 95: "The instruction \"interpret 'CALL'
   #TrapName.#Condition.#Level\" below does not set the variables RESULT and .RESULT; any result
   returned is discarded." Transcripts B and D use `return` with no expression, so they do not
   exercise it.
9. **Scope check.** `CALL ON NOVALUE`, `CALL ON SYNTAX` and `CALL ON LOSTDIGITS` are ooRexx extensions
   (6.3.2.39 lists four names). Those three are not in 8.2.4's boundary loop at all, so whatever the
   crate does for them is unconstrained by the standard -- but it should not accidentally change the
   timing of the four that *are* constrained.
