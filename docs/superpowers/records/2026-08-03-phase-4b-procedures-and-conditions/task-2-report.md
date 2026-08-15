# Task 2 report: the error report carries a stack of sites, and the clause echo saturates at 40

Commit: `addf320f2b3c868165017ac123250fe5b359017b`.

Baseline confirmed green before any change: `cargo test --workspace` 853 passed / 0 failed
/ 4 ignored, corpus `30 of 30 matching`, assertion table `4224 of 4259`, tree clean at
`1c6cc329`.

---

## 1. The oracle transcripts, and the exact programs

Every capture below used the required wrapper, reading stdout, stderr and the exit status
as three separate descriptors:

```bash
( ulimit -v 1048576; \
  LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE ) 1>"$out" 2>"$err"; rc=$?
```

Indent counts below are the number of spaces between the eleven-byte `%6d *-* ` prefix and
the clause text, measured by script rather than by eye.

### 1a. The two-level INTERPRET case

`i1.rex`

```rexx
say 1
interpret "say 2 & 1"
```

```text
RC 222   stdout: 1
     2 *-* say 2 & 1                     (indent 0)
     2 *-* interpret "say 2 & 1"         (indent 0)
Error 34 running /abs/i1.rex line 2:  Logical value not 0 or 1.
Error 34.901:  Logical value must be exactly "0" or "1"; found "2".
```

`i2.rex` — the same nested one level deeper:

```rexx
say 1
interpret 'interpret "say 2 & 1"'
```

```text
RC 222
     2 *-* say 2 & 1
     2 *-* interpret "say 2 & 1"
     2 *-* interpret 'interpret "say 2 & 1"'
Error 34 running /abs/i2.rex line 2:  Logical value not 0 or 1.
```

**Three entries for three levels, all carrying line 2.** `i5.rex` repeats it inside two
`DO`s (`interpret 'interpret "say 2 & 1"'` on line 3): all three echoes at line 3, all
three at indent 4.

### 1b. The unit the stack counts is a *level*, not a block

This is the measurement the plan's original one-line rule did not state, and it is why
`FailureSite`'s doc now says "one per activation-like level".

| program | echoes |
|---|---|
| `say 1/0` inside three nested `DO`s | **1** |
| `interpret "say 2 & 1"` | **2** |
| `interpret 'interpret "say 2 & 1"'` | **3** |

A multi-clause fragment does not add entries either — `i7.rex`,
`interpret "nop; say 3; say 2 & 1"`, echoes twice, not four times.

### 1c. A fragment's indent base is delta 0, and the fragment's own nesting adds on top

`i8.rex`

```rexx
say 1
interpret "do jj = 1 to 1; say 2 & 1; end"
```

```text
     2 *-*   say 2 & 1;      (indent 2 = enclosing 0 + the fragment's own DO)
     2 *-* interpret "do jj = 1 to 1; say 2 & 1; end"   (indent 0)
```

`q3.rex`, the same with an `IF` inside the fragment and one `DO` outside it:

```rexx
do kk = 1 to 1
  interpret "if 1 = 1 then say 2 & 1"
end
```

```text
     2 *-*       say 2 & 1   (indent 6 = enclosing 2 + a matched IF's own 4)
     2 *-*   interpret "if 1 = 1 then say 2 & 1"        (indent 2)
```

So the base is the enclosing clause's **absolute printed indent, with no bump of its own**,
and `static_indent` inside the fragment adds to it exactly as it would anywhere else. Note
the inner clause text keeps its `;` (`say 2 & 1;`) — the fragment's own clause span ends
there.

### 1d. A raise inside a `DO` inside a called routine, and the probe that discriminates

`CALL` is Task 3's, so these are recorded for it rather than asserted here. All four
combinations, plus a two-level call:

| program | shape | innermost echo | outer echoes |
|---|---|---|---|
| `c3.rex` | `call` at indent 0, callee flat | `4 *-*   say 1/0` (2) | `1 *-* call sub1` (0) |
| `c1.rex` | `call` at indent 0, callee one `DO` deep | `5 *-*     say 1/0` (4) | (0) |
| `c6.rex` | `call` at indent 2, callee flat | `6 *-*     say 1/0` (4) | `2 *-*   call sub1` (2) |
| `c7.rex` | `call` at indent 2, callee two `DO`s deep | `8 *-* …` (8) | (2) |
| `c2.rex` | `call` at indent 4, callee flat | `8 *-*       say 1/0` (6) | `3 *-*     call sub1` (4) |
| `c5.rex` | `call` at indent 4, callee one `DO` deep | `9 *-* …` (8) | (4) |
| `c4.rex` | two nested calls, all flat | `7 *-*     say 1/0` (4) | `4 *-*   call sub2` (2), `1 *-* call sub1` (0) |

**The rule: a callee's base is the calling clause's printed indent + 2**, and each callee
clause prints base + its own `static_indent`. Every row fits; `2 x depth` fits none of the
`c2`/`c5`/`c6`/`c7` rows. `c2.rex` is the discriminating probe the brief named — a call two
`DO`s deep into a flat callee — and it gives 6 where `2 x depth` predicts 2.

`c4.rex` also pins the major line: it reports `line 7`, the **innermost** entry's line, not
the outermost.

**This is a different quantity from `INTERPRET`'s.** A fragment gets +0 and a routine gets
+2, measured on the same day against the same binary. That is now stated in
`phase-4-exclusions.txt` for Task 3 to consume.

### 1e. The 40-column cap

`d<N>.rex`: N nested `do vN = 1 to 1` around `say 1/0`, written with **no source
indentation at all**, so the printed indent cannot be the file's own.

| depth | 1 | 2 | 17 | 18 | 19 | 20 | 21 | 25 | 30 |
|---|---|---|---|---|---|---|---|---|---|
| oracle | 2 | 4 | 34 | 36 | 38 | **40** | **40** | **40** | **40** |

`s25.rex` (25 plain `do` blocks, no control variable) gives 40 too, so the cap is not
specific to `Controlled` loops.

**Value lines are not capped**, which is the measurement that rules out clamping inside
`static_indent` or `push_indent`. `t25.rex` (`trace r`, 25 nested `DO`s, `yy = 7`):

```text
    27 *-*                                         yy = 7      -> *-* indent 40
       >>>                                                     "7"
                                                               -> 53 blanks after the
                                                                  prefix, i.e. indent 50
```

**The cap is on the total, applied after an activation base is added.** 18/19/20 nested
`DO`s around `interpret "do jj = 1 to 1; say 1/0; end"`:

| outer `DO`s | `interpret` echo | fragment echo | uncapped fragment |
|---|---|---|---|
| 18 | 36 | 38 | 38 |
| 19 | 38 | 40 | 40 |
| 20 | 40 | **40** | 42 |

### 1f. A fragment that fails to parse

`p1.rex` — `say 1` / `interpret "do forever then"`:

```text
RC 229   stdout: 1
     2 *-* do forever then
     2 *-* interpret "do forever then"
Error 27 running /abs/p1.rex line 2:  Invalid DO or LOOP syntax.
Error 27.901:  Incorrect data following FOREVER keyword on the loop; found "THEN".
```

`interpret "if"` is 35.929 at rc 221 in the same shape.

**The parse-time echo carries no indent.** `p6.rex`, the same `INTERPRET` two `DO`s deep,
prints the fragment clause at **indent 0** and the `INTERPRET` at 4; `p7.rex`, where the
failing clause is nested inside the fragment's own `DO`, still prints it at 0 (and as
`do forever then;`, with the terminator). So this is not the activation base under another
name, and I did not model it as one.

A top-level syntax error (`p4.rex`) prints one echo, also at indent 0, and its own line.

---

## 2. Running the new expectations, and which ones failed

The cap expectation failed at the baseline, and **this is a 4a defect this task closes,
not a regression this task introduced.** I built the baseline commit `1c6cc329` in a
throwaway git worktree and ran its `rexx-run` against the same probe files:

| depth | oracle | baseline `rexx-run` | after this task |
|---|---|---|---|
| 18 | 36 | 36 | 36 |
| 19 | 38 | 38 | 38 |
| 20 | 40 | 40 | 40 |
| 21 | 40 | **42** | 40 |
| 25 | 40 | **50** | 40 |
| 30 | 40 | **60** | 40 |

The `INTERPRET` echo expectations also failed at the baseline, as expected — that half was
Task 1's deliberate hand-off, not a defect found here.

---

## 3. How I satisfied myself the indent rule holds at more than one shape

Not by nesting depth alone, because depth-only variation is what the brief warns hid a
defect for a whole plan revision. Four axes:

* **Enclosing depth**: fragment bases measured at absolute indent 0 (`i1`, `i8`), 2 (`i4`,
  `q3`, `r2`), 4 (`i5`), 36, 38 and 40 (`cap1`–`cap3`).
* **Nesting shape inside the fragment**: flat (`i1`), a `DO` (`i8`), a matched `IF`/`THEN`
  (`q3`, which contributes 4 rather than 2 and so would expose an implementation that
  hard-coded "two per level").
* **Construct kind for the enclosing clause**: plain `DO` blocks and `Controlled` `DO`s
  give the same indents (`s25` vs `d25`), and source indentation is irrelevant (every
  `d<N>.rex` is written flush left and still prints 2N).
* **Failure kind**: an arithmetic raise, a logical-value raise, a `LEAVE`/`ITERATE` that
  escapes fragment text (`q2`, `q4`), an expression failure *before* the fragment runs
  (`q1`, correctly one echo), and a parse failure.

Then I mutated the implementation and measured which shapes each mutation survives, rather
than reasoning about it. Four mutations, each built and run:

| mutation | two `DO`s deep, `INTERPRET` on line 3 | `interpret …` alone on line 1 |
|---|---|---|
| level never sealed | one echo, not two | one echo, not two |
| no line override | inner echo at line **1** | **identical to correct** |
| a called routine's `+ 2` base | inner echo at **8** | inner at 4 |
| no base at all | inner echo at **2** | **identical to correct** |

**This corrected a claim I had already written into a doc comment.** I first wrote that
three of the four survive at top level; measuring showed it is two, and for two unrelated
reasons that both look like success — at indent 0 the base is 0 so omitting it changes
nothing, and with the `INTERPRET` on line 1 the enclosing line and the fragment's own line
are both 1 so overriding it changes nothing. Varying only the depth would have caught the
first and not the second. The comment now carries the measured table.

All four mutations also fail the committed corpus witness
`rust/corpus/lang/interpret_error_echo.rex`, whose `INTERPRET` is at indent 2 on a line
well past 1.

---

## 4. What I did about the two clause-echo formatters

**I made there be one.** `trace.rs`'s own doc already said the two were "one quantity with
two formatters, not two quantities"; `error.rs`'s `Raised::report` now calls
`trace::push_clause` instead of holding a byte-identical second copy of the same four
lines. The clamp is a single `indent.min(MAX_CLAUSE_INDENT)` inside `push_clause`, and
`MAX_CLAUSE_INDENT` is a documented constant carrying the measurement.

This is stronger than the shared helper or shared constant the brief allowed: with one
formatter the two cannot disagree at all, rather than being kept in agreement by a shared
number that either could stop using. `static_indent` is untouched, its signature unchanged,
and `push_prefixed_blanks` (every value line) is untouched, so `>>>` stays uncapped as
measured.

Verified the clamp is load-bearing: setting `MAX_CLAUSE_INDENT` to `usize::MAX` takes the
corpus from `32 of 32` to `31 of 32`, naming `lang/deep_nesting_indent_cap.rex` with the
exact byte difference (50 spaces vs 40).

---

## 5. Whether the top-level syntax path could share the `ParseError` conversion

**The mapping: yes, and it is built once.** `impl From<&ParseError> for Raised` in
`error.rs` is `Raised::syntax(error.code, error.sub, Vec::new())` and has nothing
fragment-specific in it. `Loud::parse` is gone.

**The report: no, and the obstacle is concrete rather than a preference.**
`Raised::report`'s major line names a source line; the line comes from
`ParseError::line(&source)`; and `parse_program` takes `text: Vec<u8>` **by value** and
returns only the `ParseError` on the failure path, so by the time `execute`'s parse arm
runs, the `ProgramSource` that could answer has been built and dropped inside the parser.
There is no route back to it: `rexx-parse` exposes `ProgramSource::new` and `scan`, but the
`parse(&ProgramSource)` composition that turns a source into a `Program` is private, so the
only way to answer is to clone the entire program text before every parse in order to serve
a path that runs only on syntax errors. Closing it properly is a `rexx-parse` signature
change — return the source alongside the error, or make that composition public — which is
outside this task's file list. That is written into `execute`'s own parse arm rather than
left to be rediscovered.

Both paths also share a second gap, so duplicating the mapping would not have helped
either of them: the failing clause never became an `Instruction`, and `ParseError` carries
the clause's **start** byte with **no end**, so there is no span to echo. I deliberately
did not guess one. Taking the text to end-of-source is right for a single-clause fragment
and silently wrong for `interpret "do jj = 1 to 1; do forever then; end"`, whose oracle
echo is `do forever then;` and not the rest of the text (measured, `p7.rex`). A rule that
is right sometimes and quietly wrong otherwise is worse than an absence that is recorded.

What the fragment path now produces, against the oracle:

| | oracle | before | after |
|---|---|---|---|
| exit code | 229 | 120 | **229** |
| condition | 27.901 | none (loud) | **27.901** |
| enclosing echo(es) | present, right indent | absent | **present, matching** |
| major line | matching | absent | **matching** |
| failing fragment clause echo | present | absent | absent |
| sub-message substitution | `found "THEN"` | n/a | `found "&1"` |

The last two are recorded in `phase-4-exclusions.txt` with their measurements, and the unit
test asserts them **as they are** rather than as the oracle has them, so neither can shrink
unnoticed. The `&1` is `ParseError` carrying no substitution values, which `rexx-parse`'s
own module note already records as a deliberate decision that Phase 4 owes back.

---

## 6. Implementation summary

* `Raised::report` takes `ClauseSite { path, sites: &[FailureSite] }` and emits one echo per
  entry, innermost first; the major line names `sites.first()`'s line. The one-element case
  is byte-identical — **every pre-existing expectation in `error.rs` is unchanged**, only
  the construction shape moved, which is the evidence rather than a new test asserting it.
* `Interp::failure_site` stays exactly as it was (first-wins, guard untouched, never
  cleared — inherited item **I11**). `Interp::failure_sites: Vec<FailureSite>` holds levels
  already sealed. `run.rs`'s new `seal_site_level` moves one into the other and carries the
  comment naming **Task 7** as the owner of clearing both.
* `Interp::clause_line_override` supplies the line while a fragment supplies the text;
  `clause_site` became a method to read it (its "free function, not a method" reason was
  corrected, not deleted).
* The `Interpret` arm sets `indent_offset` to the enclosing clause's absolute indent —
  **set, not added**, since that value already contains any offset in force — and saves and
  restores both fields, which is also what makes a nested fragment inherit the outer
  fragment's line.
* `run_fragment` passes `Some(&fragment.source)` and seals its level on all three error
  paths.
* The stack is never resolved by walking `Interp::activations`, per the brief.

### Comments corrected rather than left standing

`source: None` now has no caller. I kept the `Option` — collapsing it is a mechanical
change across every signature that threads it, which is a restructuring rather than this
task's — but corrected every comment that claimed a `None` caller exists or that a site
inside a fragment is unresolvable: `run_activation`'s gap block, `step`'s `source`
paragraph, `step_in_temps_frame`'s first-wins paragraph, `LeaveOrigin::site`,
`run_fragment`'s own doc, `clause_site`, and `spike.rs`'s reference to the deleted
`Loud::parse`. **Collapsing the `Option` is worth doing and is flagged here rather than
done.**

---

## 7. Corpus

Two programs, not one.

* `lang/deep_nesting_indent_cap.rex` — the brief's own, in **`phase-4a.txt`** (25 plain
  `DO` blocks, nothing outside that file's header). `EXPECTED_SUBSET` in
  `tests/coverage.rs` was amended in the same commit; the assertion was neither widened nor
  deleted.
* `lang/interpret_error_echo.rex` — in **`phase-4b.txt`**, a raise inside a fragment inside
  a `DO`. **This one is beyond the brief's file list and I added it deliberately**: without
  it the task's central deliverable has no live differential witness at all, since
  `interpret_dynamic.rex` was written to raise and trace nothing precisely because this gap
  existed. Flagging it rather than assuming the widening was wanted.

So the figure is **32 of 32**, not the `31 of 31` the brief predicted; the extra one is the
4b witness above.

Both were confirmed **compared, not merely counted**, two ways:

* The brief's own pattern: appending a program known to diverge (a `CALL`) gave
  `31 of 32 matching` with `[CALL] lang/zzz_temp_divergence_probe.rex: stderr, exit code
  differ` naming it; reverted, and `git status` confirms nothing was left behind.
* Stronger, and specific to each new program: breaking the clamp names
  `deep_nesting_indent_cap.rex`; each of the three echo-stack mutations names
  `interpret_error_echo.rex`.

Both needed a committed `SOURCELINE` expectation
(`rexx-parse/tests/sourceline_oracle/<name>.txt`), generated with that file's own documented
driver. **I ran the driver on copies in the scratchpad, never on the files in the tree**,
because the driver instantiates `.Package~new` on its argument and doing that inside the
repository is forbidden.

---

## 8. Documentation

* `phase-4-exclusions.txt`'s **KNOWN GAP** row (inherited item **I12**) amended to carry the
  measured rule in full: the unit is a level and not a block, both fragment echoes carry the
  enclosing clause's line, the fragment's indent base is delta 0, a called routine's is
  +2 with the four-row measurement table, and the major line names the innermost entry. It
  records the `INTERPRET` half as closed here and the `CALL` half as Task 3's, and keeps the
  history of the reason that was already falsified once. The still-open parse-echo and
  substitution gaps are stated with their measurements.
* A new **CLOSED DEFECTS** section for the 40-column cap: what the oracle does, what we did,
  why the clamp is not in `static_indent`, and which test goes red if it is removed. It is a
  section rather than a deletion because a fixed divergence leaves behind the temptation to
  "simplify" the fix.

---

## 9. Test output

```
cargo test --workspace          858 passed; 0 failed; 4 ignored   (baseline: 853/0/4)
REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus
                                32 of 32 matching                 (baseline: 30 of 30)
cargo fmt --all --check         exit 0
cargo clippy --workspace --all-targets -- -D warnings
                                exit 0
```

Exit statuses read unpiped in every case. No `unsafe` was added or needed; the C++ tree was
not touched.

Tests added: `error.rs` — `the_report_echoes_one_line_per_level_innermost_first`,
`the_clause_echo_saturates_at_forty_columns`,
`a_parse_error_becomes_the_condition_the_oracle_raises`; `lib.rs` —
`a_raise_inside_a_fragment_reports_both_clauses`,
`a_fragment_that_does_not_parse_raises_the_oracles_condition`. Tests extended:
`run.rs`'s `interpret_traces_the_text_it_is_about_to_run` now asserts the whole oracle
transcript instead of stopping one line short, and pins the fragment echo's indent.

Each new assertion was checked against the "what degenerate implementation satisfies this"
question, and every one has a named mutation that turns it red, listed above or in its own
doc comment. `a_parse_error_becomes_the_condition_the_oracle_raises` deliberately does not
assert `Raised::condition`: that field is still `expect(dead_code)` awaiting `SIGNAL ON`,
and a test-only read would fulfil the expectation in `cfg(test)` builds only, turning the
annotation into a warning under `--all-targets` without giving the field the genuine reader
its doc comment is waiting for.

---

## 10. Concerns

1. **The corpus figure is 32 of 32, not 31 of 31**, because I added a second program. See
   section 7 — it is the only live differential witness for the echo stack, but it is a
   scope decision the coordinator may want to reverse.
2. **`Option<&ProgramSource>` now has no `None` caller.** Kept, comments corrected, and
   flagged as a follow-up rather than collapsed in this task.
3. **Two parse-error divergences remain**, both bounded and both recorded: the failing
   fragment clause is not echoed (needs a clause span on `ParseError`), and a parse error's
   sub-message leaves `&1` unfilled (needs substitution values `ParseError` deliberately
   does not carry). Neither is a regression; both are strictly closer to the oracle than the
   rc-120 loud failure they replace.
4. ~~**A newline inside `INTERPRET` text is error 13.1 on the oracle and we accept it.**~~
   **Struck — the claim was wrong** (review F5, confirmed by re-measuring). The crate raises
   13.1 at rc 243 on all four spellings I checked, matching the oracle on exit status,
   condition, major line and major message. The only residue is concern 3's two gaps: the
   parse-time fragment echo is missing and the substitutions read `"&1" ('&2'X)` where the
   oracle has `"\n" ('0A'X)`. Nothing here needs recording that concern 3 does not already
   record. **How I got it wrong is the useful part**: I ran the probe *before* Step 5b landed,
   read rc 120, and never re-ran it afterwards — the same stale-probe failure mode this
   project keeps meeting, in a concern I raised rather than in code I wrote.
5. **The `CALL` indent rule in section 1d is measured but not implemented**, by design —
   Task 3 owns it. The mechanisms it needs all exist and are named in the amended KNOWN GAP
   row.
6. **I could not identify the discrete set of "eleven existing error-report witnesses"** the
   brief names. What I can state: every pre-existing assertion on report bytes passes with
   its expected bytes untouched — the `error.rs` diff shows only construction-shape changes,
   and `spike.rs`'s `a_raised_condition_reports_the_failing_clause` (the end-to-end
   one-echo report) is unmodified and green.

---

# Fix round 1 (review F1, F2, F3, and the recording halves of F4 and F5)

Commit: `6c24ab1587b18f6938c1b7f10e17489b773a496a`.
Base for this round: `8b7e4053` (the plan correction withdrawing the `indent_offset`
reuse instruction).

**Every finding was reproduced against the live oracle before anything was changed.** All
three code findings reproduced exactly as reported, and F5's counter-measurement stood up
against my own claim.

## F1 — the activation base has its own field

`indent_offset` had two producers that write it **absolutely** — `= 4` at the absorbed
`WhenCase` false escape, `= 0` at the end of `run_otherwise` — which was correct while it
carried only a transient escape elevation and destroyed a fragment base the moment it
carried one too. Reproduced, no `CALL` anywhere:

```rexx
do z = 1 to 1
interpret "select; when 1 = 0 then nop; otherwise nop; end; say 1/0"
end
```

```text
oracle       2 *-*   say 1/0
before       2 *-* say 1/0        <- base destroyed by run_otherwise's reset
after        2 *-*   say 1/0
```

Took the reviewer's preferred fix rather than the minimal one: **`Interp::activation_indent`
is its own field**, and `indent_offset` means one thing again.

**Two things beyond the literal instruction, both measured.**

*First, a third site needed the base — `pop_search_frame`.* The instruction was "added
alongside `indent_offset` at every site that adds `indent_offset` today", and that site adds
neither. It resets a `LEAVE`/`ITERATE` search's residual indent to a construct's own lexical
position, which is an absolute printed indent and so needs the base. Measured on three
shapes, all reporting the `LEAVE` at 2 where we printed 0:

| program (all inside `do z = 1 to 1`) | oracle | before | after |
|---|---|---|---|
| `interpret "do jj = 1 to 1; leave zz; end"` | 2 | 0 | 2 |
| `interpret "select; when 1 = 1 then leave zz; end"` | 2 | 0 | 2 |
| `interpret "do jj = 1 to 1; do kk = 1 to 1; leave zz; end; end"` | 2 | 0 | 2 |

It takes `activation_indent` and deliberately **not** `indent_offset` — its contract is
restoring the value saved when the frame was pushed, and Task 11's fourteen-point probe
settled that. Adding only the base leaves all fourteen shapes byte-identical, because the
base is `0` in every one.

*Second, `indent_offset` has to be zeroed for the fragment's duration, not just left alone.*
The enclosing clause's printed indent **already contains** whatever escape elevation was in
force, so setting `activation_indent` from it while leaving `indent_offset` at `4` counts
the elevation twice. Measured on an `INTERPRET` inside an escaped `OTHERWISE`'s own body one
`DO` deep: the oracle prints the fragment's clause at 12, and the double-counting version
prints 16. That shape passed *before* this round and had to keep passing, which is why it is
row three of the new regression test rather than a fourth bug.

## F2 — `when_indent`, and why it is now impossible to reintroduce

Reproduced: `do z = 1 to 1` around `interpret "select; when 1 = 1 then nop; end; nop"` under
`trace r` printed the `WHEN` at 2 where the oracle prints 4.

Rather than adding the missing addend at that one site, I introduced
**`Interp::printed_indent`** — `static_indent(target) + activation_indent + indent_offset` —
and routed all six sites through it. F2 exists *because* six sites open-coded the addition
and one was missed; a missing addend is not something a reader notices, so the fix is to
leave nothing to notice. `static_indent`'s own signature and purity are untouched, and the
clamp is still nowhere near it. `pop_search_frame` is the single documented exception.

## F3 — the three doc comments, plus the one the fix itself obsoleted

* `record_failure_site`'s "so it is always `0` here in practice" — rewritten. The underlying
  argument about escape dispatches was sound; what was wrong was reading it as "the addend
  is always zero here", which the fragment base falsified. It now says the addend is
  emphatically not always zero and that nothing may assume it is.
* `indent_offset`'s "explicitly restored to `0` by `run_otherwise`" — **true again**, because
  the second producer moved out. Kept, with its reference to `step_in_temps_frame`'s indent
  computation updated to `printed_indent`.
* `indent_offset`'s "No corpus or spec example nests this deeply" — the paragraph is gone,
  and the replacement says *why*: the narrowness it disclosed is gone with F2's fix, the
  `WHILE`/`UNTIL` sites were never really exceptions (they read a `printed_indent` result),
  and `pop_search_frame` is the one deliberate exclusion.
* `activation_indent`'s own doc records both quantities, both producers, the measured
  delta-0-vs-plus-2 difference from `CALL`, and the double-counting measurement.

## F4 — recorded, not fixed

I re-measured all seven shapes independently rather than copying the table. Every row
reproduces. `say 1/0` one `do` deep, after each preceding block:

| preceding block | oracle | ours |
|---|---|---|
| `do jj = 1 to 1 ... end` (controlled) | **0** | 2 |
| `do 2 ... end` (repeat count) | **0** | 2 |
| `do forever ... leave ... end` | 2 | 2 |
| `do while 0 = 1 ... end` | 2 | 2 |
| `do jj = 1 to 0 ... end` (zero-trip) | 2 | 2 |
| `if 1 = 1 then nop` | 2 | 2 |
| `select ... end` | 2 | 2 |

Exactly the two shapes that run out of iterations. The zero-trip row is the one that shows
the decrement belongs to the re-test that fails, not to the construct. I also confirmed it
reaches this task's deliverable: `interpret "do jj = 1 to 1; nop; end; say 1/0"` one `DO`
deep reports its innermost echo at 2 against the oracle's 0, and two `DO`s deep at 4
against 2.

Recorded in four places: a new KNOWN GAP paragraph tied to the existing controlled-loop
re-tested-pass row (same mechanism, second symptom); the "except after an exhausted
controlled or repeat `DO`" qualifier on the fragment rule in the exclusions row, in the
`Interpret` arm, and in `interpret_error_echo.rex`'s header. `static_indent`'s doc gets a
`# That last paragraph is measurably false, in exactly one shape` section — it stated purity
as the design decision at its centre, and the honest correction is to keep the design and
name the exception, since closing the gap means modelling the oracle's counter rather than
making the function impure.

I also kept the reviewer's observation about *why* the existing test misses it: it runs at
top level, where the oracle's counter is already 0 and cannot go lower. That is the same
blind spot that hid two of this task's own four mutations — met twice in one task, which is
worth more than either instance alone.

## F5 — concern 4 struck

The reviewer is right and I was wrong. Re-measured on four spellings: the crate raises 13.1
at rc 243 in every one, matching the oracle on exit status, condition, major line and major
message. The residue is only concern 3's two recorded gaps. Concern 4 in the original report
is struck through with the correction and with how it happened: I ran the probe *before*
Step 5b landed, read rc 120, and never re-ran it after the fix.

## Regression tests added

Two, both `run_program`-level and both asserting the **whole** stderr so that a wrong indent
on either echo fails rather than only the one being probed:

* `a_fragments_activation_base_survives_every_indent_writer_inside_it` — three rows: the
  `OTHERWISE` reset, the `LEAVE` search reset, and the escaped-`OTHERWISE`-around-the-
  `INTERPRET` guard against double counting.
* `a_when_scan_inside_a_fragment_echoes_at_the_fragments_own_indent` — the full `trace r`
  transcript, byte-identical. Uses a plain `do` block rather than `do z = 1 to 1` on purpose:
  a `Controlled` loop's re-tested pass omits two `>>>` lines (the KNOWN GAP row), and this
  test is not the place to encode that.

**Neither is vacuous, and each is specific.** Four mutations, each run:

| mutation | `..._activation_base_...` | `..._when_scan_...` |
|---|---|---|
| `pop_search_frame` loses the base | **FAILED** | ok |
| the base goes back into `indent_offset` (the pre-fix shape) | **FAILED** | ok |
| `indent_offset` not zeroed for the fragment | **FAILED** | ok |
| `when_indent` loses the offsets | ok | **FAILED** |

Each mutation kills exactly one of the two, so neither is a test that any change turns red.

## Verification

```
cargo test --workspace          860 passed; 0 failed; 4 ignored   (was 858/0/4)
REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus
                                32 of 32 matching                 (unchanged)
cargo fmt --all --check         exit 0
cargo clippy --workspace --all-targets -- -D warnings
                                exit 0
```

**The F1 fix moved no existing expectation**, and the reason is structural rather than
lucky: `activation_indent` is `0` for every program that is not running a fragment, so
adding it at a site cannot change a 4a answer. 858 → 860 is the two new tests and nothing
else.

`interpret_error_echo.rex` grew by eleven comment lines (the F4 qualifier), so its committed
`SOURCELINE` expectation was regenerated with that file's own documented driver, run on a
scratchpad copy rather than on the file in the tree.

## Not touched, deliberately

The eight Minors, per the coordinator's instruction that they are deferred to the
whole-branch review. One note for whoever applies them: **M3's proposed replacement text
needs recomputing.** It says the failing `INTERPRET` is on line 41 of a 42-line file and
suggests "the second-to-last line"; the F4 qualifier above moved it, and the file is now 53
lines. The header sentence M3 flags is still inaccurate and still unfixed.
