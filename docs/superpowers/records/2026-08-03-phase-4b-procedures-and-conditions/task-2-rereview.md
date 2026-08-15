# Task 2 re-review: fix round 1 (`8b7e4053..6c24ab15`)

Scope: only whether F1-F5 are addressed, whether the fix introduced anything
new, and the five things the implementer did beyond the instruction. The task
itself was passed by the prior reviewer and is not re-reviewed.

Tree at review time: clean. Restored after every experiment; `git status
--porcelain` empty before and after each of the six mutations, and empty at the
end.

Every oracle run used
`( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx FILE )`
with stdout, stderr and exit status read as three separate files, compared with
`cmp` per descriptor. `rexx-run` was rebuilt after every mutation and after
every restore — one early batch was run against a stale mutated binary, was
caught, and was re-run clean; every number below is from a rebuilt binary.

---

## Verdicts

| # | Verdict | Evidence |
|---|---|---|
| **F1** | **ADDRESSED** | `Interp::activation_indent` is its own field, both absolute writers of `indent_offset` are untouched, and 6 `OTHERWISE`-inside-a-fragment shapes at bases 0/2/4/6 are byte-identical to the oracle, while the pre-fix shape (mutation M2) reproduces the divergence in 5 of them. |
| **F2** | **ADDRESSED** | `when_indent` goes through `printed_indent`; 6 fragment shapes and 4 no-`INTERPRET` shapes are byte-identical, and reverting that one line (M4) diverges all of them. |
| **F3** | **PARTIALLY ADDRESSED** | All three named comments were rewritten, but two of the three rewrites carry new false statements (N3, N4), and the generalisation staled three further per-site comments, one of which is now wrong inside a fragment (N6). |
| **F4** | **PARTIALLY ADDRESSED** | Recorded in all four places with the seven-shape table, every row of which reproduces — but the rule the record draws from the table is measurably false (N1), and the corpus header's new paragraph is false and its guidance backwards (N2). |
| **F5** | **ADDRESSED** | Re-measured on 5 spellings: rc 243 in every one, identical major line and both message lines; residue is only the two already-recorded gaps. Concern 4 is struck with an accurate correction. |

**New findings:** Important 3 · Minor 4.

---

## Rulings on the five beyond-instruction items

### 1. `pop_search_frame` needed the base — **right, and understated**

The instruction said "every site that adds `indent_offset` today", and this site
added neither offset, so it was genuinely outside the letter of the prescription
and inside its intent: `origin.indent` is an absolute printed indent.

Verified by mutation M1 (drop `+ self.activation_indent`, rebuild, run):

| program (fragment text in quotes) | enclosing | oracle | with M1 |
|---|---|---|---|
| `interpret "do jj = 1 to 1; leave zz; end"` | top level | 0 | 0 (masked) |
| the same | `do z = 1 to 1` | 2 | 0 |
| the same | one plain `do` | 2 | 0 |
| the same | two plain `do`s | 4 | 0 |
| the same | three plain `do`s | 6 | 0 |
| `interpret "select; when 1 = 1 then leave zz; end"` | one `do` | 2 | 0 |
| `interpret "do jj = 1 to 1; do kk = 1 to 1; leave zz; end; end"` | one `do` | 2 | 0 |
| `interpret "do jj = 1 to 1; select; when 1 = 1 then leave zz; end; end"` | two `do`s | 4 | 0 |
| `interpret "iterate zz"` | one `do` | 2 | 2 (no frame popped) |
| `interpret "leave zz"` | one `do` | 2 | 2 (no frame popped) |

Six shapes diverge without the base, not three; the report understates its own
result. The two that do not are the ones where the fragment contains no
construct owning a search frame, so `pop_search_frame` never runs.

**Deliberately not taking `indent_offset` is right, and I tested it rather than
accepting the argument.** Four programs put a non-matching `LEAVE`/`ITERATE`
inside an *escaped* `OTHERWISE` body (so `indent_offset == 4` at the moment the
frame pops) with no `INTERPRET` anywhere, plus one shape that has a fragment
base *and* an escape elevation at once:

| shape | oracle | crate |
|---|---|---|
| `leave zz` in an escaped `OTHERWISE`, one `do` deep | 2 | 2 |
| the same at top level | 0 | 0 |
| the same with a real `DO` around the `LEAVE` | 0 | 0 |
| `iterate zz` in an escaped `OTHERWISE` inside `do i = 1 to 2` | 0 | 0 |
| `interpret "select case 2; …; otherwise leave zz; end"` one `do` deep | 2 | 2 |

None is elevated by 4. If `pop_search_frame` took `indent_offset` all five would
be wrong.

**The fourteen shapes are unchanged.** Structurally: none of the fourteen
contains an `INTERPRET`, and `activation_indent` is written non-zero only by the
`Interpret` arm. Empirically:
`the_corrected_28x_indent_rule_matches_all_fourteen_probed_shapes` is green, and
under the amplification mutation M6 (`activation_indent * 101` at both sites)
only four tests move, all four `INTERPRET` tests, and the fourteen-shape probe
is not among them.

### 2. `indent_offset` must be zeroed for the fragment's duration — **right**

The double-count is real. Mutation M3 (`let saved_offset = self.indent_offset;`,
leaving the field alone) on the exact shape the report names:

```
oracle   5 *-*             say 1/0;      <- 12
M3       5 *-*                 say 1/0;  <- 16
```

Three further shapes diverge under M3 (the same construct at top level, two
`DO`s deep, and with a flat fragment); the one that does not is the shape whose
fragment contains its own `OTHERWISE`, which resets the offset itself.

**"That shape passed before this round" is true** — under M2 (the base back in
`indent_offset`, the pre-fix shape) all three escaped-`OTHERWISE`-around-the-
`INTERPRET` programs are byte-identical to the oracle. So row 3 is a guard row,
not a fourth bug, exactly as claimed.

**The guard row catches it**: M3 turns
`a_fragments_activation_base_survives_every_indent_writer_inside_it` red and
leaves the other 195 lib tests green.

### 3. The F2 generalisation — **correct and complete for clause echoes**

I enumerated the clause-echo indent computations myself rather than reading the
report's count. Every `*-*` byte in the crate comes from `trace::push_clause`,
reached from exactly two places: `Interp::trace_clause` (6 call sites in
`run.rs`) and `error.rs:344` (the report, reading `FailureSite::indent`). The
indents feeding those:

| site | `run.rs` | how it gets its indent |
|---|---|---|
| `WHEN` scan echo | 962/973 | `printed_indent` |
| every ordinary clause | 1468/1473 | `printed_indent` |
| `OTHERWISE` marker | 1704/1709 | `printed_indent` |
| `Simple` block `END` | 1962 | `printed_indent` |
| `DO`/`LOOP` per-pass re-echo | 2209 | `do_indent` = `current_value_indent`, a `printed_indent` result |
| `END` per-pass re-echo | 2257 | same |
| `UNTIL`'s second `DO` re-echo | 2282 | same |
| failure site, ordinary | 1550 | `printed_indent` |
| failure site, `LEAVE`/`ITERATE` origin | 1607 | `printed_indent` |
| failure site, `LEAVE` search residual | 1783 | `static_indent + activation_indent` (the documented exclusion) |
| failure site, `WHILE`/`UNTIL` | 2234/2299 | `loop_indent` = `do_indent + 2`, derived |

Nothing bypasses it. `static_indent` is untouched: the whole non-comment delta
in `run.rs` is 8 call-site rewrites plus the new 3-line `printed_indent`, and
the function's own signature and body do not appear in the diff at all. It is
still a pure function of `(instructions, target)`.

The generalisation also fixed more than the review knew about — see N3 — and its
stated justification is false in one respect, also N3.

### 4. "Moved no existing expectation structurally rather than luckily" — **right, and I tested the claim rather than the outcome**

Structurally: `activation_indent` has exactly two writes, both in the `Interpret`
arm (`mem::replace` in, plain assignment back out), and the restore is
unconditional — `run_fragment`'s result is bound to a local, not `?`-propagated —
so the field is non-zero only strictly during `run_fragment`.

Empirically, mutation M6 multiplies `activation_indent` by 101 at both sites
that add it. If any non-fragment path depended on it, something without an
`INTERPRET` would move. Result: 4 of 196 lib tests fail, all four
`INTERPRET` tests, and the corpus goes 32 → 31 naming
`lang/interpret_error_echo.rex` and nothing else.

One qualification the report does not make: this argument covers **F1 only**.
F2's change to `when_indent` did move a non-fragment answer — see N3 — for the
better.

### 5. Each of four mutations kills exactly one of the two new tests — **confirmed, all four carried out**

| mutation | `…_activation_base_…` | `…_when_scan_…` | other lib tests |
|---|---|---|---|
| M1 `pop_search_frame` loses the base | **FAILED** | ok | 195 ok |
| M2 base back in `indent_offset` | **FAILED** | ok | 195 ok |
| M3 `indent_offset` not zeroed | **FAILED** | ok | 195 ok |
| M4 `when_indent` loses the offsets | ok | **FAILED** | 195 ok |

Neither test is one that any change turns red. Both are oracle-grounded rather
than self-consistent: I ran all four programs (three rows plus the `trace r`
transcript) against the oracle and the crate output is byte-identical to the
oracle on all three descriptors, so the committed literals equal the oracle's
bytes transitively.

Worth recording: **the corpus stays at 32 of 32 under M1, M2 and M3.** All three
new behaviours are witnessed only by the new unit test. That is not wrong — the
unit test asserts whole stderr and is specific — but the gate is not a second
witness for this round the way it was for the task.

---

## New findings

### Important

**N1 — F4's recorded rule is false: a `DO WHILE` or `DO UNTIL` that completes a
pass decrements too, and the decrement crosses the fragment boundary outward.**

All seven table rows reproduce exactly, so the *table* is sound. The sentence
drawn from it — "So it is exactly the two shapes that run out of iterations" —
is not, and neither is `static_indent`'s doc listing "a `DO WHILE` whose
condition goes false" among the shapes that "leave the counter alone". The
measured `DO WHILE` row is a **zero-trip** `do while 0 = 1`. A `DO WHILE` that
runs a body pass and then goes false decrements exactly like the controlled one:

| program, `say 1/0` after the block | oracle | crate |
|---|---|---|
| `n=0; do while n = 0; n = 1; end` one `do` deep | **0** | 2 |
| the same two `do`s deep | **2** | 4 |
| `n=0; do while n < 2; n = n+1; end` (two passes) | **0** | 2 |
| `n=0; do until n = 1; n = 1; end` one `do` deep | **0** | 2 |
| the same two `do`s deep | **2** | 4 |
| `do jj = 1 to 1 while jj < 5 … end` | **0** | 2 |
| `do label q jj = 1 to 1 … end` | **0** | 2 |
| `n=0; do while n = 0; n = 1; leave; end` (left by `LEAVE`) | 2 | 2 |
| `do 0 … end` | 2 | 2 |
| `do jj = 1 to 3; leave; end` | 2 | 2 |
| plain `do … end` block | 2 | 2 |
| two exhausted controlled `DO`s in sequence, two `do`s deep | **0** | 4 |

The last row shows the decrements **accumulate** (4 → 0), which the record also
does not say. The measured rule is: any repetitive `DO`/`LOOP` that completes at
least one body pass and then ends because a control test fails — count
exhausted, `WHILE` false, `UNTIL` true — decrements once, and it stacks. A
zero-trip loop, a loop left by `LEAVE`, and a non-repetitive block do not. The
zero-trip row therefore does *not* show that "the decrement belongs to the
re-test that fails" — a zero-trip's first test also fails; what distinguishes
them is whether a pass completed.

It reaches Task 2's deliverable in the `WHILE`/`UNTIL` shapes as well:
`interpret "n = 0; do while n = 0; n = 1; end; say 1/0"` one `DO` deep reports 2
against the oracle's 0, and the `UNTIL` spelling likewise.

And it reaches it in a direction nothing records — **outward**. A completed loop
*inside the fragment* lowers the **enclosing program's** later clauses:

```
do / do / interpret "do jj = 1 to 1; nop; end" / interpret "say 1/0" / end / end

oracle    4 *-*   say 1/0            4 *-*   interpret "say 1/0"
crate     4 *-*     say 1/0          4 *-*     interpret "say 1/0"
```

so "The fragment base is right; the lexical indent it is added to is not" is
only half of it — the base itself is computed from a `current_value_indent` that
has already drifted. The same shape moves the enclosing `end` under `trace r`.

The false rule is now written into the tree in four places: the qualifier
"EXCEPT after an exhausted controlled or repeat DO" in
`docs/superpowers/plans/phase-4-exclusions.txt:31-32` of the diff, the KNOWN GAP
prose at `phase-4-exclusions.txt` ("exactly the two shapes that run out of
iterations"), the `Interpret` arm comment at `run.rs:806-813`, `static_indent`'s
new `# That last paragraph is measurably false, in exactly one shape` section at
`run.rs:3357-3378` (the section heading itself says "in exactly one shape"), and
`rust/corpus/lang/interpret_error_echo.rex:25-34` with its mirror in
`rust/crates/rexx-parse/tests/sourceline_oracle/interpret_error_echo.txt`.

*Fix:* restate the exception as "after any repetitive `DO`/`LOOP` that completed
a pass and then ended on a failing control test (count exhausted, `WHILE` false,
`UNTIL` true) — and the effect accumulates, and crosses an `INTERPRET` boundary
outward". Add the `WHILE`-after-a-pass and `UNTIL` rows to the table beside the
zero-trip `WHILE` row so the two are not confused again, and drop "in exactly
one shape" from the `static_indent` heading.

**N2 — the corpus witness's new header is false, and its guidance is backwards.**

`rust/corpus/lang/interpret_error_echo.rex:31-34` (and its byte-identical mirror
in the `sourceline_oracle` expectation):

> The failing fragment below has no completed loop before its failing clause, so
> the plain rule applies to it. Anyone adding one to this file should expect it
> to diverge, and should not "fix" the indent to match.

Lines 47-49 of that same file are `do kk = 1 to 1 / interpret "say 'inside a
DO'" / end` — a completed controlled loop, before the failing clause. The first
sentence is simply false about its own file.

The second is false too, and it is the load-bearing one. I added a second
completed controlled loop (`do mm = 1 to 1 / nop / end`) immediately before the
failing `INTERPRET` in a scratch copy: **byte-identical to the oracle on all
three descriptors.** The file is safe because it sits at top level, where the
oracle's counter floors at 0 — the same floor the record itself names two
paragraphs later as the reason the existing 4a test misses this. Wrap the tail
of the file in one `do … end` and it diverges immediately (measured: oracle 2/4,
crate 4/6).

So a future editor is told the file is protected by a property it does not have,
and told to expect a divergence that will not happen — while the property that
does protect it (top level) goes unstated.

*Fix:* "This file runs its failing `INTERPRET` at top level, where the oracle's
counter floors at 0, so the completed `do kk = 1 to 1` above it cannot move
anything. Nesting the tail inside another `DO` would expose the gap."

**N3 — three new comments claim the `WHEN`-scan defect was not live before the
fragment base; measured, it was a live 4a divergence in a program with no
`INTERPRET` at all.**

The three:

* `run.rs` `printed_indent` doc: "That was invisible while `indent_offset` was a
  transient escape elevation … and it became a live divergence the moment 4b's
  Task 2 gave the same machinery an activation base to carry".
* `lib.rs` `indent_offset` doc: "The bound was false **as soon as** a fragment
  base rode the same field".
* `lib.rs` the new `WHEN`-scan test's own doc: "Invisible while `indent_offset`
  was a transient escape elevation … and a divergence the moment a fragment base
  went through the same machinery".

Under M4 (`when_indent` reverted to bare `static_indent`, i.e. the pre-F2 code),
this nine-line program with no `INTERPRET`:

```rexx
trace r
select case 2
  when 2 then
    when 3 then nop
  otherwise
    select
      when 1 = 1 then nop
    end
end
```

```
oracle     7 *-*           when 1 = 1        >>>             "1"
pre-F2     7 *-*       when 1 = 1            >>>         "1"
```

Three more shapes (one `DO` deeper, two `WHEN`s, a `DO` between the `OTHERWISE`
and the inner `SELECT`) diverge the same way. So F2's underlying defect was a
live 4a divergence all along; the fragment base only made someone look.

The same paragraph family repeats the error in a fourth place, and this one
*endorses* the original wrong reasoning rather than replacing it —
`record_failure_site`'s rewritten doc:

> The argument was sound about `indent_offset` alone — an absorbed `SELECT`'s
> escape dispatch is always closed … before another `SELECT`'s `whens` scan runs

It is not sound. Mutation M5 (drop `indent_offset` from `record_failure_site`)
on a `WHEN` *condition* that raises inside a nested `SELECT` inside an escaped
`OTHERWISE`, no `INTERPRET`:

```
oracle   6 *-*           when 1/0 = 1     <- 10
M5       6 *-*       when 1/0 = 1         <-  6
```

That is exactly the "`Select`'s own two direct calls below" case the paragraph
is about. The old comment's conclusion ("always `0` here in practice") was
correctly retracted; its premise was kept and is false.

This matters beyond wording: F2 exists because a false bound ("no corpus or spec
example nests this deeply") stopped someone looking. Replacing it with a
narrower false bound ("it only became live once a fragment base rode the field")
sets up the same failure for whoever reads it next.

*Fix:* state it plainly — the `WHEN` scan omitted an addend that a nested
`SELECT` inside an escaped `OTHERWISE` already needed, with no `INTERPRET`
involved; the fragment base made it easy to hit, not newly wrong. And delete
"The argument was sound about `indent_offset` alone".

### Minor

**N4 — "the one place either offset is applied" is contradicted inside the same
paragraph, and stated without the caveat in the other copy.**
`lib.rs` `indent_offset` doc: "Every site that applies either offset now goes
through `Interp::printed_indent` … so there is no per-site list left to go
stale" — then, two sentences later, "`pop_search_frame` is the one deliberate
exclusion". `lib.rs` `activation_indent` doc carries only the absolute form:
"Added to `static_indent` … by `Interp::printed_indent`, **which is the one
place either is applied**", with no exclusion note at all.
*Fix:* "the one place either offset is applied to a clause echo, with
`pop_search_frame` the single documented exclusion" in both.

**N5 — the new test's doc contradicts its own table.**
`lib.rs:1708-1709`: "Every row below was captured from the oracle before the fix
and every one of the **first three** failed against it" — the table has three
rows, and its third cell says "already right", as does the paragraph directly
beneath ("It passed before this fix"). Verified: row 3 is byte-identical under
M2. Should read "the first two".

**N6 — three per-site comments still open with `+ self.indent_offset`, which no
longer appears in the code they annotate; one of them is now false.**
`run.rs:1599` (`leave_origin`), `run.rs:1696` (`run_otherwise`), `run.rs:1954`
(the `Simple` block's `END`) all begin with a literal `` `+ self.indent_offset` ``
and are followed by a `self.printed_indent(...)` call. The first and third are
stale only in form. The second also says the addend is "`0` on the ordinary 'no
`WHEN` matched' path" — no longer true whenever the `SELECT` is inside a
fragment, where the addend is the activation base (measured: `otherwise` echoes
at 10 for a fragment at base 2 inside an escaped dispatch). This is the same
class F3 named, produced by this round's own generalisation, and the irony is
that F2's stated purpose was that "there is no per-site list left to go stale".

**N7 — `printed_indent`'s doc opens with a self-contradiction.**
"Through 4a the six sites that needed `+ self.indent_offset` each wrote it out,
and one of them — the `WHEN` scan in `Select`'s own arm — did not." Five wrote
it out; the sixth did not.

---

## Constraints and gates

| check | result |
|---|---|
| `cargo test --workspace` | 860 passed, 0 failed |
| `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` | **32 of 32 matching** |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `git status --porcelain` | empty, before and after all six mutations |

* The corpus witness `lang/interpret_error_echo.rex` was additionally run
  directly against the oracle: byte-identical on all three descriptors, inner
  echo at 4, outer at 2.
* **4a error-report expectations unchanged**: the diff touches five files and
  `error.rs` is not among them; no expected-byte literal was edited. The one
  expectation file that did change,
  `rexx-parse/tests/sourceline_oracle/interpret_error_echo.txt`, is 4b Task 2's
  own, and its regeneration is mechanically correct — `count 53` matches
  `wc -l` on the `.rex`, and the body is byte-identical to it.
* **`EXPECTED_SUBSET` neither widened nor deleted**: `tests/coverage.rs` has no
  commits in the range, and `phase_4a_subset_matches_the_committed_list` still
  reads `assert_eq!(subset, EXPECTED_SUBSET, …)` — equality against the literal.
* No `unsafe` added. The C++ tree was not modified. `.Package~new` was never
  instantiated on a repository file. The SF #2018 shape was never run.

## Verified true (spot-checked comment claims not already covered)

* Fragment delta 0: `interpret "do jj = 1 to 1; say 2 & 1; end"` echoes 2 and 0
  at top level, 6 and 4 two `DO`s deep — both byte-identical.
* `CALL` at printed indent 4 into a flat routine echoes the callee's clause at
  **6** on the oracle (`CALL` itself is still Task 3's; rc 120 vs 214).
* The nested-fragment line inheritance: `interpret 'interpret "say 2 & 1"'`
  gives all three echoes line 1 — byte-identical.
* The `WHEN`-scan test's stated reason for using a plain `do` rather than
  `do z = 1 to 1`: the `Controlled` variant diverges by exactly two missing
  `>>>` lines (`"1"` and `"2"`) and nothing else. Confirmed.
* The doc's own pre-fix measurement for `do z = 1 to 1` around
  `interpret "select; when 1 = 1 then nop; end; nop"`: `WHEN` at 2 under M4,
  oracle 4. Confirmed.
* `the_indent_after_a_loop_has_already_exited_is_not_left_over_from_it` does run
  at top level and asserts indent 0, which is what the oracle prints there.
* F5's residue: five spellings, all rc 243, identical
  `Error 13 running … Invalid character in program.`; the crate differs only in
  the absent parse-time fragment echo and `"&1" ('&2'X)` for `"\n" ('0A'X)`.
