# Task 2 review: the error report carries a stack of sites, and the clause echo saturates at 40

Reviewed diff: `review-1c6cc329..addf320f.diff` (commit `addf320f`).
Tree at review time: clean, restored after every experiment (`git status --porcelain` empty).

---

## Verdicts

**1. Spec compliance — MET, with one requirement partially met.**

Every step of the brief was carried out, including the two the brief warned had
already been built wrong twice (the fragment line override and the first-wins
race). The one shortfall is in the *Interfaces* clause itself — "each entry
carrying its own line number and its own **absolute printed indent**". The line
is right in every shape I measured. The indent is right for every shape the
implementer probed and wrong for two classes they did not (F1, F4 below), both
measured against the live oracle with three-line programs.

**2. Task quality — GOOD, accept with the F1/F2/F3 fixes.**

Test hygiene is the best I have seen in this tree. Every new assertion is
oracle-grounded byte-for-byte rather than self-consistent; I carried out all
four of the implementer's own mutations plus the clamp mutation myself, and all
five turn both a unit test and a named corpus program red. No vacuous
assertion found. The substantive defect is that the task reused
`Interp::indent_offset` for a second, longer-lived quantity (which the brief
told it to do) without auditing the two sites that write that field
*absolutely*, and without correcting the three doc comments whose truth
depended on the field having only one meaning.

**Counts:** Critical 0 · Important 5 · Minor 8.

---

## Per-requirement table

| # | Requirement (brief) | Verdict |
|---|---|---|
| R1 | `Raised` carries a stack, innermost first | **met** — verified against the oracle for `INTERPRET`, nested `INTERPRET`, multi-clause fragments, `LEAVE`/`ITERATE` escaping a fragment |
| R2 | each entry carries its own line | **met** — `i1`/`i2`/`i5`/`q3`/`l1`–`l3` byte-identical |
| R3 | each entry carries its own **absolute printed indent** | **partially met** — F1 and F4 |
| R4 | `static_indent` signature unchanged, clamp not inside it | **met** — `static_indent` untouched in the diff |
| R5 | extend `Interp::indent_offset`, not a parallel mechanism | **met** — and this reuse is the direct cause of F1/F2/F3 |
| R6 | I12: amend the KNOWN GAP row with the measured rule | **met** (M4, M6 are inaccuracies inside it) |
| R7 | I11: leave `failure_site`'s first-wins guard alone, name Task 7 | **met** — guard byte-identical; Task 7 named at `run.rs:2917-2920` and `lib.rs:886-892` |
| R8 | Step 1: capture the four named oracle expectations | **met** — I reproduced all four independently |
| R9 | Step 2: say the cap expectation fails today, as a 4a defect | **met** — report §2, measured on a throwaway build of `1c6cc329` |
| R10 | Step 3: one-element case byte-identical | **met** — no expected-byte literal in `error.rs` changed; only `ClauseSite` construction moved |
| R11 | Step 3: do not walk `Interp::activations` | **met** |
| R12 | Step 4: fragment entry at delta 0 | **met** for the shapes measured (see R3) |
| R13 | Step 4: the traced `INTERPRET` `*-*` echo | **met** — byte-identical on all three descriptors |
| R14 | Step 5: clamp `*-*` only, one rule at both formatters | **met** — there is now exactly one `*-*` formatter in the crate |
| R15 | Step 5b: `ParseError` → 27.901 at rc 229, built once, top level checked | **met** — obstacle independently confirmed |
| R16 | Step 6: suite + gate green | **met** — 32 of 32; `cargo fmt --all --check` 0; `cargo clippy --workspace --all-targets -- -D warnings` 0 |
| R17 | Step 7: `phase-4a.txt` + `EXPECTED_SUBSET` amended together, not widened | **met** |
| R18 | Step 8: KNOWN GAP amended, CLOSED DEFECTS section added | **met** |
| G1 | no `unsafe`, C++ tree untouched | **met** — `git diff --name-only` touches no `interpreter/`, `samples/`, `build/`, `ootest/` path |

---

## Claims verified by running

Every oracle run used the required wrapper
`( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx FILE )`
with stdout, stderr and exit status read as three separate descriptors.

### The 40-column cap

Oracle, `N` nested plain `do` blocks around `say 1/0`, files written flush left:

| depth | 1 | 2 | 17 | 18 | 19 | 20 | 21 | 25 | 30 |
|---|---|---|---|---|---|---|---|---|---|
| oracle | 2 | 4 | 34 | 36 | **38** | **40** | **40** | **40** | **40** |
| `rexx-run` at `addf320f` | 2 | 4 | 34 | 36 | **38** | **40** | **40** | **40** | **40** |

Matching at every depth either side of 20, and stdout/stderr/rc all identical.

Value lines are not clamped. `trace r`, 25 nested `DO`s, `yy = 7`:

```
oracle    27 *-* <40 spaces>yy = 7        >>> <52 spaces>"7"
rexx-run  27 *-* <40 spaces>yy = 7        >>> <52 spaces>"7"
```

The cap is on the total, applied after the activation base. 18/19/20 nested
`DO`s around `interpret "do jj = 1 to 1; say 1/0; end"`, oracle and crate
byte-identical in all three:

| outer `DO`s | `interpret` echo | fragment echo | uncapped fragment |
|---|---|---|---|
| 18 | 36 | 38 | 38 |
| 19 | 38 | 40 | 40 |
| 20 | 40 | **40** | 42 |

### Two formatters, one rule

`grep` over `crates/*/src` finds exactly one non-test site that writes the
`*-*` prefix: `trace.rs:254`, inside `push_clause`, which clamps with
`indent.min(MAX_CLAUSE_INDENT)`. `error.rs`'s `Raised::report` calls it. So it
is not a constant typed twice; it is one formatter. Neither surface was
missed — the trace `*-*` and the report `*-*` are literally the same code
path, and the deep-nest corpus program exercises the report half while
`t25.rex` exercises the trace half.

`MAX_CLAUSE_INDENT` load-bearing: setting it to `usize::MAX` turns
`error::tests::the_clause_echo_saturates_at_forty_columns` red **and** the
corpus to `31 of 32`, naming `lang/deep_nesting_indent_cap.rex`.

### The echo stack

Oracle vs `rexx-run`, all three descriptors, all byte-identical:

| program | shape | result |
|---|---|---|
| `interpret "say 2 & 1"` | two levels | OK |
| `interpret 'interpret "say 2 & 1"'` | three levels | OK |
| the same, two `DO`s deep | three levels at indent 4 | OK |
| `interpret "nop; say 3; say 2 & 1"` | multi-clause fragment, still 2 echoes | OK |
| `interpret "do jj = 1 to 1; say 2 & 1; end"` | fragment's own `DO` | OK |
| `interpret "if 1 = 1 then say 2 & 1"` inside a `DO` | matched `IF` contributes 4 | OK |
| `interpret "leave outer"` / `"iterate outer"` / `"leave"` | `LEAVE` escaping fragment text | OK |
| `say 1/0` inside three nested `DO`s | one echo, not four | OK |

### The traced `INTERPRET` echo

`trace r` / `zz = 'nop'` / `interpret zz`. Oracle rc 0, stdout empty, stderr:

```
     2 *-* zz = 'nop'
       >>>   "nop"
     3 *-* interpret zz
       >>>     "nop"      <- (shown collapsed; the real line is >>>   "nop")
     3 *-* nop
```

`rexx-run` produces the identical five lines, rc 0, empty stdout. All three
descriptors compared separately and independently.

### `ParseError` → 27.901 at rc 229, and the two remaining divergences

Both divergences are real, and both are asserted **as they are**:

```
p1.rex:  say 1 / interpret "do forever then"
oracle   rc 229   1 *  2 *-* do forever then
                        2 *-* interpret "do forever then"
                        Error 27 ... line 2:  Invalid DO or LOOP syntax.
                        Error 27.901: ... found "THEN".
rust     rc 229   1 *        (fragment clause echo absent)
                        2 *-* interpret "do forever then"
                        Error 27 ... line 2:  Invalid DO or LOOP syntax.
                        Error 27.901: ... found "&1".
```

`p6.rex` (the same `INTERPRET` two `DO`s deep) confirms the parse-time echo
carries **no** indent: the oracle prints the fragment clause at 0 and the
`INTERPRET` at 4. `interpret "if"` is 35.929 at rc 221 with no substitution, so
that pair matches on everything except the missing echo.

`lib.rs::a_fragment_that_does_not_parse_raises_the_oracles_condition` asserts
the whole stderr byte string including `found "&1"` and the single echo, so
neither divergence can shrink unnoticed — closing either one turns the test red
and forces the expectation to be updated deliberately.

The declined clause-end guess is justified: the counterexample is real.
`interpret "do jj = 1 to 1; do forever then; end"` echoes `do forever then;` on
the oracle — the inner clause with its terminator, not the rest of the text —
so to-end-of-source would be silently wrong here and right for `p1`.

The top-level obstacle is real too: `rexx_parse::parse` (`lib.rs:216`) is
private, `parse_program` takes `Vec<u8>` by value, and `ProgramSource::new`
takes ownership, so `execute` genuinely cannot resolve a line without cloning
the whole text before every parse.

### Corpus is 32 of 32, and both new programs are compared

`REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` → `32 of 32 matching`.

I did not accept the count. I built my own witness check by mutating the
implementation and reading which program the report names:

| mutation | unit tests turned red | corpus |
|---|---|---|
| `MAX_CLAUSE_INDENT = usize::MAX` | `the_clause_echo_saturates_at_forty_columns` | `31 of 32`, `lang/deep_nesting_indent_cap.rex` |
| `seal_site_level` a no-op | `a_raise_inside_a_fragment_reports_both_clauses` | `31 of 32`, `lang/interpret_error_echo.rex` |
| `base_line = None` | that one + `interpret_traces_the_text_it_is_about_to_run` | `31 of 32`, `lang/interpret_error_echo.rex` |
| `base_indent = current_value_indent + 2` | same two | `31 of 32`, `lang/interpret_error_echo.rex` |
| no base at all | same two | `31 of 32`, `lang/interpret_error_echo.rex` |

Each new slot is therefore genuinely exercised, and each is the *only* program
its own mutation names.

### `EXPECTED_SUBSET`

`tests/coverage.rs:512` gained exactly one entry,
`"lang/deep_nesting_indent_cap.rex"`, in the same commit as the `phase-4a.txt`
line it pins. `phase_4a_subset_matches_the_committed_list` still reads
`assert_eq!(subset, EXPECTED_SUBSET, ...)` — equality against the literal, not
length, not a subset test. Not widened, not deleted.

### The `CALL` indent rule (measured, not implemented)

All seven rows of the report's table reproduce exactly:

| program | caller printed indent | callee depth | inner echo | outer echo(es) |
|---|---|---|---|---|
| `c3` | 0 | 0 | line 4, **2** | line 1, 0 |
| `c1` | 0 | 1 | line 5, **4** | line 1, 0 |
| `c6` | 2 | 0 | line 6, **4** | line 2, 2 |
| `c7` | 2 | 2 | line 8, **8** | line 2, 2 |
| `c2` | 4 | 0 | line 8, **6** | line 3, 4 |
| `c5` | 4 | 1 | line 9, **8** | line 3, 4 |
| `c4` | 0, nested calls | 0 | line 7, **4** | line 4 at 2, line 1 at 0 |

The discriminating case holds: `c2` is a call two `DO`s deep into a flat
callee and gives **6**, where "2 × depth" predicts 2. Base = calling clause's
printed indent + 2, plus the callee's own `static_indent`, fits every row.
Major line names the innermost entry (`c4` → line 7, `c2` → line 8), which is
also what the new `error.rs` unit test's expected bytes encode.

### The four mutations and the doc-comment table

I carried out all four and read the actual output rather than the test result.
Probe A = the unit test's shape (two `DO`s, `INTERPRET` on line 3); probe B =
`interpret "do jj = 1 to 1; say 2 & 1; end"` alone on line 1.
`(line, indent)` per echo, innermost first:

| variant | A | B |
|---|---|---|
| correct | `[(3,6), (3,4)]` | `[(1,2), (1,0)]` |
| level never sealed | `[(3,6)]` — one echo | `[(1,2)]` — one echo |
| no line override | `[(1,6), (3,4)]` — line **1** | `[(1,2), (1,0)]` — **identical** |
| callee `+2` base | `[(3,8), (3,4)]` — indent **8** | `[(1,4), (1,0)]` — inner at 4 |
| no base at all | `[(3,2), (3,4)]` — indent **2** | `[(1,2), (1,0)]` — **identical** |

This is the table in `lib.rs:1600-1606`, cell for cell. The corrected claim is
correct: exactly two of the four survive the top-level shape, and for the two
unrelated reasons stated. All four die against both the corpus witness and at
least one unit test.

### The newline-in-`INTERPRET` note (report concern 4)

**Not reproducible as stated — see F5.** The crate does not accept it.

---

## Findings

### Important

**F1 — `indent_offset` has two meanings now, and two sites write it absolutely.**
`rust/crates/rexx-exec/src/run.rs:1698` (`run_otherwise`) does
`self.indent_offset = 0;`, and `rust/crates/rexx-exec/src/run.rs:1263`
(the absorbed `WhenCase` false escape) does `self.indent_offset = 4;`. Both
were correct while the field was a transient escape elevation that only ever
went `0 → 4 → 0`. This task made the same field carry an `INTERPRET`
fragment's activation base for the whole life of the fragment, so both writes
now destroy that base.

Measured, three-line program, no `CALL`:

```rexx
do z = 1 to 1
interpret "select; when 1 = 0 then nop; otherwise nop; end; say 1/0"
end
```

```
oracle       2 *-*   say 1/0          <- fragment base 2
rexx-run     2 *-* say 1/0            <- base lost by run_otherwise's reset
```

Under `trace r` the same program shows every clause after the `OTHERWISE`
two spaces short, and the `SELECT CASE` variant
(`select case 2; when 2 then; when 3 then nop; otherwise nop; end; nop`) shows
`otherwise` at 6 where the oracle prints 8 and its body at 8 where the oracle
prints 10 — the `= 4` write, same cause. The identical `SELECT`s outside a
fragment match the oracle exactly, so this is specific to the new overload.

This is not a regression (before this task no fragment clause was echoed at
all), but it is a live divergence in the property the brief's Interfaces
clause names, and Task 3 will inherit it: a called routine whose body contains
an `OTHERWISE` will lose its `+2` base for every clause after it.

*Fix:* give the activation base its own field — `Interp::activation_indent`,
set by the `Interpret` arm and (next) by `CALL`, added alongside
`indent_offset` at every site that adds `indent_offset` today. That keeps the
escape elevation's absolute set/reset correct and is the field Task 3 needs
anyway. Minimal alternative: make the escape additive
(`self.indent_offset += 4` at `:1263`, `self.indent_offset -= 4` at `:1698`),
but that leaves one field carrying two quantities and will need untangling at
Task 3 regardless.

**F2 — `when_indent` omits `self.indent_offset`.**
`rust/crates/rexx-exec/src/run.rs:941`:
`let when_indent = static_indent(&code.body.instructions, when_index);` —
the only clause-echo indent computation in the file that does not add
`self.indent_offset`. Measured under `trace r`,
`do z = 1 to 1 / interpret "select; when 1 = 1 then nop; end; nop" / end`:
oracle prints the `WHEN` at 4, `rexx-run` at 2. Same for a two-`WHEN` scan.

`lib.rs:860-871` already discloses that the `WHEN` scan does not add the
offset, but bounds the consequence with "No corpus or spec example nests this
deeply" — which was true of the escape elevation and is false of a fragment
base. A plain `SELECT` inside an `INTERPRET` inside one `DO` is not deep
nesting.

*Fix:* `let when_indent = static_indent(&code.body.instructions, when_index) + self.indent_offset;`
(or `+ self.activation_indent` under F1's preferred shape), and rewrite the
bound at `lib.rs:868`.

**F3 — three doc comments made false by this task and not corrected.**
The task corrected seven comments about `source: None` (all seven accurate — I
checked each), and missed the three whose truth depended on `indent_offset`
having exactly one producer.

* `rust/crates/rexx-exec/src/run.rs:1518` — "`indent_offset` is only ever
  non-zero while a *different* `SELECT`'s own absorbed-escape dispatch is still
  open ... **so it is always `0` here in practice**." False since this commit:
  it is the fragment base for the whole life of any `INTERPRET`.
* `rust/crates/rexx-exec/src/lib.rs:851` — "set by the absorbed `WhenCase`'s
  own false branch ... **explicitly restored to `0` by `run_otherwise`**".
  There is a second setter now (`run.rs:819`) and the restore-to-`0` is F1's
  defect.
* `rust/crates/rexx-exec/src/lib.rs:868` — "No corpus or spec example nests
  this deeply". Falsified by a three-line program (F2).

This project has removed six false statements from this tree in this phase;
these are three more, written *by* the change that falsified them.

*Fix:* rewrite all three alongside F1/F2. The `lib.rs` field doc needs a new
paragraph naming both quantities and both producers.

**F4 — an unrecorded pre-existing divergence that this task's own model
asserts over.** The oracle's `*-*` indent is not a pure function of lexical
depth: a `DO` loop that terminates **by exhausting its count** leaves the
indent two spaces lower for every subsequent clause at that level. Measured
with no `INTERPRET` anywhere:

```rexx
do
do jj = 1 to 1
nop
end
say 1/0      <- oracle prints at 0, rexx-run at 2
end
```

Which preceding block shapes do it (`say 1/0` one `do` deep, oracle vs crate):

| preceding block | oracle | rust |
|---|---|---|
| `do jj = 1 to 1 ... end` | **0** | 2 |
| `do 2 ... end` | **0** | 2 |
| `do forever ... leave ... end` | 2 | 2 |
| `do while 0 = 1 ... end` | 2 | 2 |
| `do jj = 1 to 0 ... end` (zero-trip) | 2 | 2 |
| `if 1 = 1 then nop` | 2 | 2 |
| `select ... end` | 2 | 2 |

It is the same re-tested-pass mechanism the KNOWN GAP row already records for
the two missing `>>>` value lines, and it is a second, unrecorded symptom of
it. It reaches this task's own deliverable directly: `interpret "do jj = 1 to
1; nop; end; say 1/0"` one `DO` deep gives oracle 0 and `rexx-run` 2 on the
report's innermost echo, and two `DO`s deep gives 2 vs 4.

`static_indent`'s own doc (`run.rs:3281-3284`) states the contrary as the
design decision at its centre — "computed fresh from the flat instruction list
every time, never carried on a running `Interp` counter ... a clause's
indentation never depends on which iteration of an enclosing loop is currently
running". That is measurably false. The existing test
`the_indent_after_a_loop_has_already_exited_is_not_left_over_from_it`
(`run.rs:6323`) does not catch it because it runs at top level, where the
oracle's counter is already clamped at 0 — the same "at indent 0 the base is 0"
blind spot the implementer identified in their own mutation table.

Not this task's to fix — it is 4a, and no part of the brief asks for it. But
it is this task's to *record*, because this task wrote the settled-sounding
model that hides it: `run.rs:806-813` ("the base is the enclosing clause's
printed indent exactly, with no bump of its own"),
`phase-4-exclusions.txt:258-264` ("plus whatever nests them INSIDE the
fragment"), and `interpret_error_echo.rex`'s header.

*Fix:* add a KNOWN GAP row tying it to the existing controlled-loop re-test
row, with the seven-shape table above; and add "except after an exhausted
controlled or repeat `DO`" to the three places that state the fragment rule as
complete.

**F5 — report concern 4 is not reproducible; the crate does not accept a
newline in `INTERPRET` text.** Measured on four spellings
(`interpret "nop" || "0a"x || "nop"`, `interpret "say 1" || "0a"x || "say 2"`,
the same via a variable, and a trailing-newline-only form): `rexx-run` raises
**13.1 at rc 243** in every case, matching the oracle on exit status,
condition, major line and major message. The only residue is the two gaps this
task already records — the missing parse-time fragment echo and the unfilled
`&1`/`&2` substitutions:

```
oracle    Error 13.1:  Incorrect character in program "\n" ('0A'X).
rust      Error 13.1:  Incorrect character in program "&1" ('&2'X).
```

*Fix:* strike concern 4 from the report, or restate it as "a newline in
`INTERPRET` text is a further instance of the recorded parse-echo and
substitution gaps, not a separate one". Nothing needs recording that is not
already recorded.

### Minor

**M1 — stale corpus figure in the new CLOSED DEFECTS section.**
`docs/superpowers/plans/phase-4-exclusions.txt:427`: "takes the corpus from
31 of 31 to 30 of 31". Measured: **32 of 32 → 31 of 32**. The row is the one
place a future reader would check the fix is load-bearing, and the numbers do
not match anything they can run.
*Fix:* change to `32 of 32` / `31 of 32`.

**M2 — "prints its full 50" uses "prints" in two senses in one sentence.**
`docs/superpowers/plans/phase-4-exclusions.txt:413`: "the `*-*` echo prints 40
while the `>>>` line for the same clause prints its full 50". The `*-*` 40 is a
printed column count; the `>>>` 50 is the `indent` argument, whose printed
column is **52** (measured). `trace.rs:236` carries the disambiguating
parenthetical (`push_prefixed_blanks`'s own `3 + indent`, so 53 blanks after
the prefix); this copy does not.
*Fix:* add the same parenthetical, or write 52.

**M3 — `interpret_error_echo.rex:9` says "the last line of this file".**
The failing `INTERPRET` is on line **41** of a 42-line file. The header's own
point (the echoes carry the enclosing clause's line, whatever it is) survives,
but the locating detail is wrong and a reader editing the file would trust it.
*Fix:* "the second-to-last line", or drop the locator.

**M4 — the KNOWN GAP row hands Task 3 a mechanism it must not use.**
`docs/superpowers/plans/phase-4-exclusions.txt:274`: "the mechanism is already
there (Interp::indent_offset for the base, **Interp::clause_line_override for
the line**, seal_site_level for the first-wins boundary)". A called routine is
in the same source file and its clauses carry their **own** lines — measured,
`c2.rex`'s innermost echo is line 8, the callee's line, not the caller's 3.
Setting the override for `CALL` would be actively wrong. The same row states
the `CALL` line rule correctly two paragraphs above, so a careful reader will
notice the contradiction; a faithful one will not.
*Fix:* "`Interp::clause_line_override` is **not** wanted for `CALL` — a callee
resolves its own lines from the same source; it exists only because a fragment
has a source of its own."

**M5 — `corpus_differential`'s own convention not followed.**
`rust/crates/rexx-exec/tests/corpus.rs:483-496` asks each task that re-runs the
gate to add a dated row ("recording it is the instruction, not merely leaving
the older row intact"). Task 2 re-ran and got `32 of 32`; no row was added.
*Fix:* add "Expected result at commit `addf320f`: **32 of 32 matching**,
`phase-4a.txt`'s 30 plus `phase-4b.txt`'s 2."

**M6 — the `Option<&ProgramSource>` follow-up is recorded only in code and in
the report.** It is not in `phase-4-exclusions.txt`, where every other deferred
item in this phase lives, so the next reader of that file will not see it.
*Fix:* one line under KNOWN GAP or a new FOLLOW-UPS row, naming the shape
(collapse to `&ProgramSource`, which also makes `clause_site` infallible and
removes the `Option` from `LeaveOrigin::site`).

**M7 — `base_line` resolves and discards the clause text.**
`rust/crates/rexx-exec/src/run.rs:818`:
`self.clause_site(source, instruction).map(|(line, _)| line)` calls
`join_span` and allocates a `Vec<u8>` of the whole `INTERPRET` clause on every
fragment execution, only to drop it.
*Fix:* `source.map(|s| self.clause_line_override.unwrap_or_else(|| s.line_of(instruction.clause_span.start)))`,
or split a `clause_line` helper out of `clause_site`.

**M8 — `MAX_CLAUSE_INDENT` is `pub(crate)` but read only inside `trace.rs`.**
`rust/crates/rexx-exec/src/trace.rs:239`. The intra-doc link at `:244` resolves
for a private item too; `error.rs:697` names it in prose only.
*Fix:* drop `pub(crate)`.

---

## Independent rulings the coordinator asked for

### The second corpus program — **sound reasoning, well chosen, keep it**

The argument checks out empirically, not just rhetorically. I ran every
echo-stack mutation and watched which program the gate names: it is
`lang/interpret_error_echo.rex` every time, and never
`lang/interpret_dynamic.rex`. Without the new program all four mutations leave
the gate green at `31 of 31` — the task's central deliverable would ship with
its only witnesses being in-crate unit tests, which is exactly the shape the
brief's own Step 7 warns about.

It is well chosen for a reason beyond "it raises". It varies all three
quantities that a wrong implementation confuses at once — the line (41, not
1), the enclosing indent (2, not 0), and the fragment's own nesting (its own
`DO`, so inner 4 vs outer 2) — which is what makes it discriminate rather than
merely fail. Its three successful `INTERPRET`s above the failing one, including
a fragment inside a fragment, keep it from being an error-path-only file. The
header states which wrong answer each field produces, which is the right
standard for a corpus witness.

One caveat carried into F4: the program's own header states the fragment
indent rule as complete, and the rule has an exception.

### Retaining `Option<&ProgramSource>` — **right call for this task, but it
needs an owner, not just a comment**

Collapsing it touches 10 signatures in `run.rs` and roughly 31 call sites, and
would cascade into `LeaveOrigin::site`, `clause_site`'s return type and eight
`if let Some(...)` guards. Doing that in the same commit as the echo stack
would have buried the substantive change under a mechanical one and made the
byte-identity argument for the one-element case much harder to check. The
implementer's judgement is right, and correcting the seven comments rather than
deleting them is the better half of the call — I checked all seven and all
seven are accurate.

The hazard is real though, and worth naming precisely: those eight `if let
Some(...)` guards can now only take one branch, so any future path that
reintroduces a `None` will *silently drop a clause echo* instead of failing.
That is the same failure mode this project keeps finding — a branch that
cannot be wrong today and fails quietly the day it can. Hence M6: the follow-up
belongs in `phase-4-exclusions.txt` with an owner, not only in a doc comment
that the collapse itself would delete.

---

## Verification commands run

```
cargo test --workspace                                  all green (194 lib + 255 + …, 0 failed)
REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus  32 of 32 matching
cargo fmt --all --check                                 exit 0
cargo clippy --workspace --all-targets -- -D warnings    exit 0
git status --porcelain                                  empty, before and after every mutation
```

Exit statuses read unpiped. No `unsafe` added. The C++ tree was not modified;
`.Package~new` was never instantiated on a repository file; the SF #2018 shape
was never run.
