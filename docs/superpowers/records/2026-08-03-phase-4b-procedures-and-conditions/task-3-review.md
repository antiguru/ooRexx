# Task 3 review: the body selector, `CALL`, `RETURN`, and the shared variable pool

Reviewed commit: `733da737` only. `bc665b40` and `046939f5` are the coordinator's
and are excluded, as are the two `src/run.rs` comment hunks in `733da737` that
the coordinator wrote (the oracle's trace-indent counter, and the
"do not write a fifth construct-shaped rule" note).

## Verdicts

* **Spec compliance: PASS with one documentation defect.** Every requirement in
  the brief is met or deliberately and correctly deviated from with the
  deviation disclosed. The one requirement whose *answer* is wrong as recorded
  is Step 4's `::routine` reachability question — see F1.
* **Task quality: PASS.** No correctness defect found in ~60 differential
  probes against the oracle. Eleven mutations reproduced; three further
  mutations of my own also caught. Nothing vacuous found. The findings are
  comment accuracy plus one forward-looking trap.

Severity counts: **Critical 0, Important 1, Minor 10.**

## Environment and what I ran

* `cargo test --workspace` — **879 passed, 0 failed, 4 ignored.**
* `cargo fmt --all --check` — exit 0. `cargo clippy --workspace --all-targets -- -D warnings` — exit 0.
* `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` — **33 of 33 matching.**
* No `unsafe`; the C++ tree untouched; `git status --porcelain` empty at the end
  of every experiment and at the end of the review.

---

## Claim-by-claim verification (all measured, none taken from the report)

### 1. The shared pool default — CONFIRMED, both directions

`v`, a stem tail, and a rewrite of the caller's own variable all cross the
boundary in both directions; the callee's writes survive the return:

```
callee sees v: caller-v          callee sees stem.1: caller-stem
caller sees v: rewritten-by-callee
caller sees w: callee-w          caller sees stem.2: callee-stem2
```

Byte-identical to the oracle on stdout, stderr and rc.

**The converse — nothing isolates that should not, and everything that should
isolate does.** `NUMERIC DIGITS/FUZZ/FORM` set in a callee do *not* survive its
return while a variable set beside them does (`caller after: 0.3333333`,
`caller sees dg: set-in-callee`), and a callee's `trace off` dies with it.
Byte-identical to the oracle. So the sharing is scoped to the variable pool and
not accidentally extended to `Settings` or `TraceMode`.

### 2. `RESULT` dropped on return, not at the call — CONFIRMED

```
inside sub result= before      <- not cleared on the way in
after sub result= 42
inside bare result= 42         <- still the previous call's value inside the callee
after bare result= RESULT      <- dropped by the bare RETURN, reads as its derived name
```

Byte-identical to the oracle (the run also contained a `symbol()` call, which
is a 4c builtin and correctly fails loudly; the four lines above matched
exactly up to that point).

### 3. Omitted arguments — CONFIRMED, and the evaluation is observable

* `call sub 1/0` → Error 42.3 at rc 214 reported **against the `CALL` clause**,
  byte-identical to the oracle, and `in sub` never printed.
* `call sub 1, , 1/0` → the same, so arguments after an omitted position are
  still evaluated and the omitted position itself is skipped rather than
  half-read.
* `call sub 1, , 3` with a normal callee → identical to the oracle on all three
  descriptors; no partial state observable.
* `call sub 1+1, 'q'` under `trace r` produces **no** value line, matching the
  oracle — so nothing is being traced that should not be.

Oracle-side check of the semantics the code comments cite: `call sub 1,,3` into
three `USE ARG` targets gives `[1] [B] [3] 3` with targets named `a,b,c` — the
omitted position leaves its target unset (rendering as its own derived name),
the ones after it do not shift, and `arg()` returns 3. Matches the substance of
the comment; see M9 on its wording.

### 4. The activation clause-echo stack — CONFIRMED, measured with the caller lexically nested

The discriminating shape (`call one` one `DO` deep; `call two` two `DO`s deep
inside `one:`; `say 1/0` in `two:`):

```
    13 *-*           say 1/0
     8 *-*         call two
     2 *-*   call one
```

Byte-identical to the oracle. Each echo carries its own activation's line
(13, 8, 2) and its own indent (10, 6, 2). `2 x depth` predicts 2/4/6 and is
falsified at the first nested caller. I also reproduced the rule under `trace r`
at every shape I could construct: caller in an `IF ... THEN` body directly, in a
`WHEN` body directly, in an `OTHERWISE` body directly, in a labelled `DO`, and
three constructs deep through `DO`/`IF-THEN-DO`/`SELECT-WHEN` — ten programs,
all byte-identical.

### 5. The two compositions — CONFIRMED, and both reported failure modes are real

`CALL` inside `INTERPRET` (trace and error paths) and `INTERPRET` inside a
callee (trace and error paths): all four byte-identical to the oracle,
including the three-level error stacks

```
     4 *-*   say 1/0            4 *-*   say 1/0
     1 *-* call sub             4 *-*   interpret "say 1/0"
     1 *-* interpret "call sub" 1 *-* call sub
```

* **Clearing `clause_line_override` rather than merely not setting it** —
  mutation M3 (`std::mem::take` → plain read, `run.rs:1725`) makes
  `a_call_inside_a_fragment_echoes_each_activations_own_line` FAIL and leaves
  `an_interpret_inside_a_callee_runs_at_the_callees_own_level` passing. That is
  exactly the reported failure mode: only the fragment-outside direction sees it.
* **Resolving the label against the activation's body, not the stepped body** —
  mutation M9 (`activation_body.labels` → `code.body.labels`, `run.rs:1638`)
  makes the same fragment test FAIL while
  `a_routine_without_procedure_shares_the_callers_pool` still passes. Again
  exactly as reported: every test with no `INTERPRET` in it is blind to it.

### 6. `::routine` reachability — the answer in the tree is WRONG (finding F1)

Measured on the oracle:

| program | oracle | ours |
|---|---|---|
| `call max 1,2` + `::routine max` | `result= 2` (builtin wins) | rc 120, loud `routine "MAX" … (4c)` |
| `call zorkolo` + `::routine zorkolo` | `in the routine` / `result= from-routine`, rc 0 | rc 120, loud `routine "ZORKOLO" … (4c)` |

The builtin-shadowing measurement the tree cites is correct. The **conclusion**
drawn from it is not: a `::routine` whose name is not a builtin *is* reachable,
and the oracle dispatches to it. Detail in F1.

### 7. The depth limit — CONFIRMED; two rows re-measured

* Flat unbounded recursion: ours rc **245**, `Error 11 … Control stack full.` /
  `Error 11.1: Insufficient control stack space; cannot continue execution.`,
  identical report lines to the oracle, 10,000 `call sub` echoes against the
  oracle's ~27,300. Oracle rc also 245.
* Re-bisected native-abort depth on the 512 MiB thread, debug:
  * 5 enclosing `DO` blocks per activation → deepest surviving **5,604** (report: 5,616)
  * 25 enclosing `DO` blocks → deepest surviving **1,402** (report: 1,403)

  Both well below `MAX_ACTIVATION_DEPTH = 10,000`, so the report's central claim
  — the counter is decoration for deeply nested bodies — reproduces.
* The 2 MiB-thread claim also reproduces exactly: a temporary unit test through
  `run_source` survives depth **80** and aborts the test binary at depth **90**,
  which justifies `tests/spike.rs` owning the recursion test.

### 8. The eleven mutations — ALL REPRODUCED, including the brief's trap

Applied by me, one at a time, each restored and verified with `cmp`:

| # | mutation | test | result |
|---|---|---|---|
| M1 | callee gets a fresh, properly popped slot frame | shares-the-callers-pool | **FAILED** |
| M1 | same | runs-its-own-clauses (no variables) | **passed** ← the trap |
| M1 | same | result-is-settled-on-return | **FAILED** |
| M2 | `activation_indent = 2 * depth` | indent-plus-two | **FAILED** |
| M2 | same | one-echo-per-activation | **FAILED** |
| M3 | keep `clause_line_override` instead of clearing | call-inside-a-fragment | **FAILED** |
| M4 | drop the `seal_site_level` call | one-echo-per-activation | **FAILED** |
| M5 | drop the `extra` write-back | interpret-name-survives-return | **FAILED** |
| M6 | falling off the end returns instead of exits | exit-or-fall-off | **FAILED** |
| M7 | `Settings::default()` instead of inheriting | numeric-inherited | **FAILED** |
| M8 | `TraceMode::OFF` instead of inheriting | trace-does-not-survive | **FAILED** |
| M9 | resolve labels against `code.body` | call-inside-a-fragment | **FAILED** |

**The trap, carried out:** with the callee given a fresh, properly popped slot
frame — a correct-looking implementation that isolates every callee —
`a_called_label_runs_its_own_clauses_not_the_main_body` (the witness with no
variables in it) **passes**, while both witnesses that carry variables fail.
The report's claim is exact.

Three further mutations of my own, none of which the report lists, all caught:

| # | mutation | test | result |
|---|---|---|---|
| M10 | clear `RESULT` at the call instead of settling it on return | result-is-settled-on-return | **FAILED** |
| M11 | skip argument evaluation entirely | arguments-evaluated-in-the-caller | **FAILED** |
| M12 | let the literal (`CALL "SUB"`) form reach the label table | quoted-name-never-reaches-labels | **FAILED** |
| M13 | `body_of` always resolves to `main` | body-selector-resolves-a-routine-directive | **FAILED** |

A caution for future mutation work in this file, which cost me one wrong
"passed": `let base_indent = self.current_value_indent;` occurs **twice** in
`run.rs` (`:1003` in the `Interpret` arm, `:1722` in `exec_call`) and
`self.seal_site_level();\n<16 spaces>return Err(failure);` occurs **twice**
(`exec_call` first, `run_fragment` second). A first-occurrence replace silently
mutates the wrong function. My M10 result above is the re-run against a unique
anchor.

### 9. Corpus: 33 of 33, and the new witness is genuinely compared

Constructed check, not a count: mutating the indent base from `+ 2` to `+ 3`
(`run.rs:1723`) drops the gate to **32 of 33** with exactly one mismatch named:

```
  [UNCLASSIFIED] lang/call_return.rex: stderr differ
```

and separately, mutation M1 (isolating callees) also drops it to 32 of 33 with
`lang/call_return.rex: stdout, stderr differ` and a diff showing
`outer sees: CALLER_V` against the oracle's `outer sees: caller-v`. So the
witness's `trace r` transcript *and* its variables are both load-bearing. The
harness compares stderr byte-exactly today — DEVIATION 0's normalisation is
Task 2b's and has not landed.

The witness is also mandatory in two further ways I checked directly: deleting
`lang/call_return.rex` from `phase-4b.txt` makes
`every_in_scope_variant_is_witnessed_by_the_phase_subsets` fail with
`InstructionKind: 1 in-scope variant(s) unwitnessed: Return`, and
`rexx-parse`'s `sourceline_matches_the_interpreter_for_every_corpus_program`
requires the new `sourceline_oracle/call_return.txt` (`count 54`, matching the
54-line program) for every `.rex` in `corpus/lang`.

### 10. `loud.rs` witness rows

`Call::Qualified` (`Phase 5`), `Call::Trap` (`4b`), `Signal` (`4b`) and
`Signal::Trap` (`4b`) rows all still present, and all still fail loudly when
run directly:

```
[call ns:sub]      rc=120  rexx-exec: CALL is not implemented (Phase 5)
[call off error]   rc=120  rexx-exec: CALL is not implemented (4b)
[call on error …]  rc=120  rexx-exec: CALL is not implemented (4b)
[signal there …]   rc=120  rexx-exec: SIGNAL is not implemented (4b)
[signal on error…] rc=120  rexx-exec: SIGNAL is not implemented (4b)
```

And the guard that would catch a future accidental deletion works: removing the
`Call::Qualified` witness row makes `assert_witness_set_is_complete` fail with
"must have exactly one entry per out-of-scope InstructionKind variant".

### 11. `EXPECTED_SUBSET` and the pinned counts

* `tests/coverage.rs` is **not in the commit's file list at all**, so
  `EXPECTED_SUBSET` is untouched. Correct: the new program is in
  `phase-4b.txt`, not `phase-4a.txt`.
* `EXPECTED_OUT_OF_SCOPE` loses exactly one row, `("InstructionKind", "Return", "4b")`.
  `Call` deliberately stays.
* The four pinned counts move `21/8/4/6/1 → 22/7/4/6/1`; only `InScope` and
  `Phase("4b")` change, they change by one in opposite directions, and the five
  still sum to 40. `loud.rs`'s two copies (`20` expected witness tags, `22`
  in-scope) move with them. All deliberate and internally consistent.
* No expected-byte literal was weakened anywhere: the only removed assertion in
  the whole diff is `assert_eq!(expected_instructions.len(), 23)`, replaced by
  `20`. Grepping every removed line in `rust/` for `*-*`, `>>>`, `.to_vec()`
  and `assert_eq` returns that one line and nothing else.

### 12. `Interp::depth` — the substitution is right

`Interp::depth` already exists at `src/lib.rs:1042` and is `eval`'s expression
recursion depth (`max_depth` beside it, `StackSpan`'s subject). The brief's
Interfaces line promised a second field under that name. The quantity Task 3
needs is exactly `self.activations.len()`, which the push/pop in `exec_call`
maintains unconditionally; a parallel field could only ever disagree with it,
and would need updating on the `Err` path too. **Ruling: correct call, correctly
disclosed.** Nothing disagrees now — the coordinator has already recorded the
correction in the plan (`…phase-4b-procedures-and-conditions.md:536`) and in
`progress.md:457-459`, and the only in-tree readers are the three
`activations.len()` sites in `run.rs`.

### 13. Extra differential probing (beyond the brief's list)

About 35 further programs, all byte-identical to the oracle on stdout, stderr
and exit status unless noted:

`RETURN` out of a controlled `DO`; `RETURN` out of `DO FOREVER`; `LEAVE` inside a
callee inside the caller's `DO FOREVER` (correctly does **not** cross the
activation boundary — both raise 28.1 with the same two-level echo);
main-body fall-through into a label; `EXIT` from inside `INTERPRET` inside a
callee; recursion to depth 400 with GC pressure (200 calls each building a
50-piece string); `CALL` from a `SELECT`/`WHEN`; `CALL` from an `OTHERWISE`;
`trace i` across a call; `interpret 'interpret "call sub"'`; `call (nm) 1, 2`
with a `>>>` on the target; `drop`/rebind across a call; three-level
call-in-fragment error stacks; the `Loud::unresolved_call` truncation path with
a 300-character dynamic target (truncates at 128 with `...`, no panic) and with
a multi-byte UTF-8 target (no panic).

Two divergences found, **neither this task's**:

* `trace r` + `do i = 1 to 2` + `call sub` is missing two `>>>` value lines on
  the re-tested pass. Reproduced with **no `CALL` in the program at all**
  (`trace r / do i = 1 to 2 / nop / end / exit`), so it is I31, the open KNOWN
  GAP that `phase-4-exclusions.txt` assigns to Task 9.
* A top-level parse error prints `rexx-exec: 47.2: Unexpected label.` where the
  oracle prints a full report. Pre-existing, and recorded already in
  `run_fragment`'s own doc comment ("`execute`'s own parse arm records the same
  gap for the top-level path").

---

## Findings

### Important

**F1. The `::routine` scoping answer is wrong as recorded, and a later task depends on it.**
`src/activation.rs:79-80`, and the same claim restated at `src/plan.rs:75-76`,
`src/run.rs:1612-1614`, and the report's section 3.

The tree says:

> `::routine max` alongside `call max 1,2` still calls the builtin and reports
> `2`, so a `::routine` **cannot be reached without first answering "is this
> name a builtin"**, which is 4c's.

The measurement is right; the inference is not. Measured on the oracle, a
`::routine` whose name is *not* a builtin is reached and run:

```rexx
call zorkolo
say 'result=' result
exit
::routine zorkolo
say 'in the routine'
return 'from-routine'
```
```
oracle rc=0    in the routine / result= from-routine
ours   rc=120  rexx-exec: routine "ZORKOLO" is not implemented (4c)
```

So the accurate statement is: **a `::routine` is reachable in 4b for every name
that is not a builtin; what needs the builtin table is getting the
builtin-colliding names right, and getting those wrong would be a wrong answer
rather than a loud gap.** Not implementing it in 4b is a defensible scope call —
it costs only a clean loud failure today — but the brief made this the answer
Task 9's `>I>`/`<I<` scope decision reads, and "cannot be reached" is the kind
of claim-that-something-cannot-happen this phase has been burned by before.

*Fix:* restate in all four places as "reachable for any non-builtin name;
deferred because the builtin step in front of it is 4c's and a name that
collides with a builtin would otherwise silently run the wrong routine", and
keep the three inherited differences the report already lists (own variable
pool, `TRACE` does not cross, builtins shadow). No code change needed.

### Minor

**M1. `src/run.rs:1690` — "Empty in every program that has no `INTERPRET` and no `DROP (v)`" is false, and this function is what falsifies it.**
`Plan::build` (`plan.rs:130`) derives names from the body's instructions only,
so `RESULT` is in the plan only when the source mentions it. `exec_call`'s own
`self.slot_of(b"RESULT")` (`run.rs:1746`) therefore grows the frame and inserts
`RESULT` into the caller's `extra` on every call in a program that never says
`result`. Measured with a temporary probe: after
`call sub / say 'x' / exit / sub: / return 1`, `extra` is `["RESULT"]`.
Harmless (the clone is a one-entry map and the write-back keeps it correct), but
the sentence is wrong. *Fix:* "Empty only in a program that never calls
anything, since `RESULT` lands here for any caller that does not name it."

**M2. `src/activation.rs:236-241` — two inaccurate claims in `Interp::trace_mode`'s doc.**
"Every reader is inside a condition that also calls a `&mut self` method in the
same expression … sixteen times over": there are **18** readers, of which **7**
have that shape (all in `run.rs`); the other 11 are in `trace.rs` and are plain
`if !self.trace_mode().X { return; }` guards, one of which
(`tracing_intermediates`, `trace.rs:403`) is not in a condition at all. And
"`TraceMode` is `Copy` for exactly this reason" is a false attribution: the
`#[derive(Copy, …)]` is present unchanged at `ef6745c0`, before this task.
*Fix:* "Seven readers in `run.rs` take the `if self.trace_mode().all && let
Some(..) = self.clause_site(..)` shape; the rest are simple guards.
`TraceMode` was already `Copy`, which is what lets this return a value."

**M3. `src/trace.rs:580-585` — a doc comment was orphaned onto the new helper.**
The four-line doc for `the_three_gates_fire_under_exactly_the_modes_that_should_show_them`
now sits immediately above `activate_empty`, so the test at `:613` has no doc
comment and the helper's doc opens with "…are each gated correctly — silent
under `TraceMode::OFF`…", which describes the test, not the helper.
*Fix:* move the first paragraph back down onto the `#[test]`.

**M4. `src/run.rs:60-65` — the module doc misstates where the fragment side's indent state lives, and "the one place they differ".**
`run_fragment` (`:3312`) does *not* save or restore any indent state; the
`Interpret` arm in `step` (`:1003-1012`) does. And `exec_call` differs from that
arm in a second way the sentence excludes: it writes `base_indent + 2` where the
`Interpret` arm writes `base_indent`. *Fix:* name the `Interpret` arm rather
than `run_fragment`, and say the two differences (`+2` vs `+0`, and clear vs
set) instead of "the one place".

**M5. `src/run.rs:221`, `:264`, `:1672`; `tests/spike.rs:359`, `:382`; `docs/superpowers/plans/phase-4-exclusions.txt:311` — "the oracle's own limit is 27,314 activations" is one sample of a varying quantity.**
Three consecutive oracle runs of the same program gave **27,306 / 27,326 /
27,328** echoes; ours is deterministic at 10,000. The conclusion the number
supports (matching it would ship a counter that never fires) is unaffected.
*Fix:* "about 27,300, and it varies run to run — it is where the native stack
gives out, not a constant."

**M6. `src/lib.rs:475-478` — `Loud::unresolved_call`'s stated contract points at the wrong test.**
"that trailing shape is a contract `loud.rs` pins with an `ends_with`" —
`loud.rs:589`'s `ends_with` applies to out-of-scope *variant* witnesses, and
`unresolved_call` has no witness row there (it is not a variant). The
`ends_with` that actually pins it is `run.rs`'s
`a_quoted_call_name_never_reaches_the_label_table`. *Fix:* name that test.

**M7. `src/run.rs:1735` — the unconditional `extra` write-back is a trap for Task 5.**
`self.activation_mut().extra = callee.extra;` is right for D9r's shared pool and
wrong the moment `PROCEDURE` gives a callee its own frame: the callee's private
run-time bindings would be written into the caller's name map. Nothing marks
this. *Fix:* one sentence at the site — "when Task 5's `PROCEDURE` passes a
different `frame`, this write-back must be skipped" — or condition it on the
frame being the caller's.

**M8. `src/run.rs:640` / `src/lib.rs:1109-1112` — the safety net the comments lean on is `debug_assert_eq!` and is compiled out in release.**
"The assertion below is what turns that into a failure at the first instruction
instead of a debugging session" and "`run_activation`'s own loop asserts exactly
that after every step" read as unconditional guarantees. In a release build the
guarantee is absent and the failure mode described (running the caller's body
against the callee's `pc`) is exactly the silent wrong answer the comment warns
about. *Fix:* say "in debug builds", or promote it to `assert_eq!` — it is one
`usize` compare per instruction.

**M9. `src/run.rs:1660` — "`call sub 1,,3` into three `USE ARG` targets gives `[1] [Q] [3]`" omits the target names.**
`[Q]` is the unset middle target rendering as its own derived name, so it only
reproduces if the middle target is spelled `q`. Reproducing with `a, b, c` (as I
did) gives `[1] [B] [3]`, which reads as a contradiction. *Fix:* "`use arg p, q,
r` gives `[1] [Q] [3]`".

**M10. `rust/corpus/phase-4b.txt:32-33` — rewrap artifact.**
The amended paragraph ends `… goes stale by construction). It` / `# requires` /
`# BOTH the write and the read …`, leaving a two-word line. Cosmetic only.

---

## Vacuity audit

For every assertion the diff adds I asked what degenerate implementation
satisfies it and whether deleting its subject leaves it green.

* The two brief-mandated witnesses are the interesting pair, and the trap is
  real and confirmed above: the variable-free one is green against a wrongly
  isolating implementation. It is **not** vacuous — it fails if the callee runs
  the main body (M13's `body_of`-always-main is a different subject, but a
  callee that re-ran `main` would print `main` first) — it is simply blind to
  the pool question, which is why the second witness exists.
* `the_body_selector_resolves_a_routine_directive_and_rejects_a_bad_index`
  exercises a function whose `Some(index)` branch nothing in production reaches.
  It is not vacuous (M13 kills it) and it is the honest way to keep the branch
  from being untested code; the surrounding docs say so.
* `unbounded_call_recursion_raises_11_1_rather_than_overflowing` asserts three
  independent things (rc, the two verbatim report lines, and an exact echo count
  of 10,000). The echo count is the strongest: it fails if the counter fires at
  the wrong depth *and* if activations are not popped on the way out.
* `a_calls_arguments_are_evaluated_in_the_caller` asserts a failure, which is
  the only observable this phase has for arguments; M11 confirms it is not
  self-satisfying.
* `Loud::missing_body` is the one genuinely unreachable branch. It is disclosed
  (C5), it is a `Loud` rather than a panic for a stated reason, and `body_of`'s
  three cases are unit-tested directly. Acceptable.
* The four pinned counts and the two witness-set counts cross-check each other
  in three files; I verified one direction fails (deleting a witness row) and
  one direction fails (deleting the corpus witness).

## Scope

Nothing beyond the brief. The two things that look like additions are both
forced: `run_source_traced`/`say_output_traced` and the reordering of fourteen
`run.rs` tests plus `trace.rs`'s `activate_empty` and `eval.rs`'s one-line
`activate` are direct consequences of moving `trace_mode` onto `Activation`
(disclosed as C4), and no assertion in any of them changed. `src/error.rs`,
named in the brief's file list, is correctly untouched — `seal_site_level` and
`Raised::report` already build the echo stack, which the byte-identical
three-deep transcripts confirm.
