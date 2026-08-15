# Task 2 re-review round 2: fix round 2 (`6c24ab15..e4caa7bf`)

Scope: only whether N1, N2 and N3 are addressed, whether the committed
fifteen-row table reproduces, whether the corpus witness stayed coherent, and
whether any comment this diff adds or changes is false.

Tree at review time: clean. Two mutations applied and reverted (`M4`
`when_indent` back to bare `static_indent`; `M5` `record_failure_site` drops
`indent_offset`), `rexx-run` rebuilt after each mutation and after each
restore. `git status --porcelain` empty before, between and after.

Every oracle run was
`( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx FILE )`
with stdout, stderr and exit status captured as three separate files and
compared with `cmp` per descriptor. 78 programs measured. The measured indent
is the count of spaces between `*-* ` and the clause text.

---

## Verdicts

| # | Verdict | One line |
|---|---|---|
| **N1** | **PARTIALLY ADDRESSED** | The *qualification* predicate (which loops decrement) is correct and survived every shape I could build. The *scope* and *accumulation* claims are false: nested qualifying loops do not accumulate, because the decrement is discarded at the `END` of an enclosing repetitive loop and at the `END` of an enclosing `SELECT`. The corrected rule was also written five lines below an uncorrected copy of the old narrow rule, and the sentence introducing the fifteen-row table still says "all seven measured". |
| **N2** | **PARTIALLY ADDRESSED** | The two false sentences are gone and the floor mechanism is the right explanation, but the replacement says the file is protected because "the failing `INTERPRET` runs at TOP LEVEL" — it does not; it runs inside `do kk = 1 to 1` at printed indent 2, which the same file states 19 lines earlier. The guidance sentence is also true only under one of two readings of "the tail". |
| **N3** | **ADDRESSED** | Both replacement claims reproduce exactly under mutation, with no `INTERPRET` in the program. |

**Fifteen-row table: all fifteen rows reproduce.** No row failed.

**New findings:** Important 3 · Minor 4. Prior-round minors N4-N7 all still
stand (not claimed by this round).

---

## 1. Is the new N1 rule correct and complete?

### 1a. The qualification predicate survives

The committed rule's predicate — *a repetitive `DO`/`LOOP` that completes at
least one body pass and then ends because a control test fails decrements
once; a zero-trip loop, a loop left by `LEAVE`, and a non-repetitive block do
not* — held on every shape I built. `say 1/0` observed three plain `DO`s deep
(static 6) unless noted, so the floor at 0 cannot mask anything:

| shape | oracle | crate | qualifies? | rule right? |
|---|---|---|---|---|
| `do i over .array~of(1,2)` (runs out) | 4 | *rc 120* | yes | yes |
| `do i over .array~new` (empty) | 6 | *rc 120* | no | yes |
| `do with index i item v over .array~of(1,2)` | 4 (at depth 1: 0) | *rc 120* | yes | yes |
| `do i over 'a b c'~makearray(' ')` | 0 (depth 1) | *rc 120* | yes | yes |
| `loop jj = 1 to 1` (`LOOP` spelling) | 0 (depth 1) | 2 | yes | yes |
| `loop 2` | 0 (depth 1) | 2 | yes | yes |
| `loop forever … leave` | 6 | 6 | no | yes |
| `do 1+1` (expression count) | 0 (depth 1) | 2 | yes | yes |
| `do jj = 1 to 10 for 2` (`FOR` budget hit) | 0 (depth 1) | 2 | yes | yes |
| `do jj = 1 to 3 for 1` | 4 | 6 | yes | yes |
| `do jj = 1 to 3 for 0` (zero-trip via `FOR`) | 6 | 6 | no | yes |
| `do jj = 1 to 3 by -1` (zero-trip) | 6 | 6 | no | yes |
| `do jj = 3 to 1` (zero-trip) | 6 | 6 | no | yes |
| `do jj = 1 to 3 while 0 = 1` (zero-trip) | 6 | 6 | no | yes |
| `do jj = 1 to 3 until 1 = 1` (one pass) | 4 | 6 | yes | yes |
| `n=0; do forever until n = 1; n = 1; end` | 4 | 6 | yes | yes |
| `do jj = 1 to 4 by 2` | 4 | 6 | yes | yes |
| `do 1` | 4 | 6 | yes | yes |
| `do jj = 1 to 2; iterate; end` | 4 | 6 | yes | yes |
| `do jj = 1 to 5` (five passes) | 4 | 6 | yes — **still one decrement** | yes |
| `do jj = 1 to 3; if jj = 2 then leave; end` (pass completes, **then** `LEAVE`) | 6 | 6 | no | **yes** |
| `n=0; do while n < 9; n=n+1; if n = 3 then leave; end` | 6 | 6 | no | **yes** |
| `do counter c jj = 1 to 1` | 0 (depth 1) | *rc 120* | yes | yes |

The two `LEAVE`-after-a-completed-pass rows are the sharpest test of the
rule's conjunction, and it gets them right: "a pass completed" alone is not
sufficient, the exit must also be a failing control test. The five-pass row
shows the decrement is one per loop, not one per pass.

`SIGNAL` out of a loop and a `SIGNAL ON SYNTAX`-trapped body **cannot be
measured**: a label is illegal inside a `DO`/`LOOP` block (Error 47.2), so
every `SIGNAL` target is at top level, where the floor at 0 masks the answer.
Both measured at oracle 4 = static, i.e. nothing observable. `RETURN`/`EXIT`
out of a loop leaves no later clause in the same activation; measured across
the `CALL` boundary instead, and the answer is that it does not cross — see
1c.

**Enumeration gap, not falsity:** `DO OVER`, `DO WITH`, `DO COUNTER`, a hit
`FOR` budget and `DO FOREVER UNTIL` all decrement and none is named. The
general clause covers them; the enumeration "count exhausted, `WHILE` false
and `UNTIL` true" does not, and 4c implements `DO OVER`. The `do forever …
leave … end | 2 | 2` table row also reads as "`DO FOREVER` never does it",
which `n=0; do forever until n = 1; n = 1; end` (oracle 4, crate 6)
contradicts.

### 1b. The accumulation claim is false — this is the substantive defect

Four places in the tree now say the decrements accumulate:

* `phase-4-exclusions.txt:393` — "THE DECREMENTS ACCUMULATE"
* `phase-4-exclusions.txt:248` (the ABSOLUTE-indent bullet) — "whose decrement also accumulates"
* `run.rs:812` — "and the effect accumulates"
* `static_indent`'s doc — "the decrements **accumulate**"
* `interpret_error_echo.rex:29` and its mirror — "the effect stacks"

The literal witness each cites is true — two exhausted controlled `DO`s **in
sequence** at one level, two `DO`s deep, give oracle 0 against our 4
(re-measured; and three in sequence three deep give oracle 0 against our 6).
But the generalisation is false, because accumulation is a property of
*sibling* loops only:

| program (all observed three plain `DO`s deep, static 6) | oracle | crate | decrements |
|---|---|---|---|
| one qualifying loop | 4 | 6 | 1 |
| two qualifying loops **in sequence** | 2 | 6 | 2 |
| two qualifying loops **nested** (`do jj = 1 to 1` around `do kk = 1 to 1`) | **4** | 6 | **1, not 2** |
| two qualifying inners in sequence inside a qualifying outer | **4** | 6 | **1, not 3** |
| qualifying inner inside `do jj = 1 to 3 … leave` | **6** | 6 | **0** |
| qualifying inner inside `do forever … leave` | **6** | 6 | **0** |
| qualifying inner inside `select`/`when … then do` | **6** | 6 | **0** |
| qualifying inner as the `WHEN` body directly | **6** | 6 | **0** |
| qualifying inner inside an `OTHERWISE` body | **6** | 6 | **0** |
| qualifying inner inside a plain `do … end` | 4 | 6 | 1 |
| qualifying inner inside **two** nested plain `do … end` | 4 | 6 | 1 |
| qualifying inner inside `if 1 = 1 then do … end` | 4 | 6 | 1 |
| qualifying loop *after* a `select … end` at the same level | 4 | 6 | 1 |

The inner loop in the `LEAVE` and `SELECT` rows **does** decrement — it is
visible on a clause still inside the enclosing construct (measured: oracle 6
against static 8 inside the `LEAVE`-terminated loop; oracle 12 against static
14 inside the `WHEN` body). The decrement is then **discarded** at the
enclosing construct's `END`.

**The scope rule the record is missing.** A qualifying loop's decrement
applies to later clauses in its own block, propagates into blocks nested
inside it afterwards, and propagates outward through the `END` of a plain `DO`
block and an `IF`-branch `DO` block — but it is discarded at the `END` of an
enclosing repetitive `DO`/`LOOP` and at the `END` of an enclosing `SELECT`.
So the record's two scope statements are each half-right and jointly
incoherent: `run.rs:811` and the corpus header say "every later clause **at
that level**" (too narrow — it also reaches outer levels), while
`phase-4-exclusions.txt:401` says it "crosses a fragment boundary outward"
(true, but says nothing about the boundaries that stop it).

Under the brief's falsification criterion — *a shape that satisfies the
description and does not decrement* — the falsifying shape is

```rexx
do jj = 1 to 3
  do kk = 1 to 1
  nop
  end
leave
end
```

The `do kk = 1 to 1` completes a pass and ends on a failing control test, and
no clause after the outer `end` is moved by it (oracle 6 = static 6).

### 1c. The crossing claims

* **Outward across an `INTERPRET` boundary — TRUE.** `do` / `do` /
  `interpret "do jj = 1 to 1; nop; end"` / `interpret "say 1/0"` / `end` /
  `end`: oracle prints both echoes at 2, crate at 4. Without the second
  `INTERPRET`, oracle 2 against crate 4. Three plain `DO`s deep, oracle 4
  against crate 6.
* **Inward across an `INTERPRET` boundary — TRUE, and recorded nowhere.**
  `do` / `do` / `do jj = 1 to 1; nop; end` / `interpret "say 1/0"`: oracle
  prints both echoes at 2, crate at 4. So the enclosing program's drift also
  reaches *into* the fragment's base. `run.rs:817` says "it reaches this base
  from both sides" but names the inside-the-fragment case as the second side;
  `phase-4-exclusions.txt` records only "OUTWARD".
* **Across a `CALL` boundary — it does NOT cross.** `do`/`do`/`do`/`call sub`/
  `say 1/0` with `sub:` containing `do kk = 1 to 1; nop; end; return`: oracle
  6, identical to the control with no loop in the routine. The counter is
  per-activation for a `CALL` and shared for an `INTERPRET`. Unrecorded, and
  it is Task 3's problem.

### 1d. The floor

Confirmed, and it clamps the counter rather than only the printed value: two
completed loops at top level followed by `do` / `say 1/0` / `end` prints at 2,
not 0 — so the level went `0 → 0 → 0 → 1`, not `0 → −1 → −2 → −1`. All four
floor probes are byte-identical to the oracle on all three descriptors.
`the_indent_after_a_loop_has_already_exited_is_not_left_over_from_it` does run
at top level (`run.rs:6450`) and asserts 0, which is what the oracle prints.

---

## 2. The fifteen-row table

Re-measured from scratch, `say 1/0` one `do` deep, each row its own file.
**All fifteen reproduce**, oracle and crate columns both:

`do jj = 1 to 1` 0/2 · `do 2` 0/2 · `n=0; do while n = 0; n = 1; end` 0/2 ·
`n=0; do until n = 1; n = 1; end` 0/2 · `do jj = 1 to 1 while jj < 5` 0/2 ·
`do label q jj = 1 to 1` 0/2 · `do forever … leave` 2/2 ·
`n=0; do while n = 0; n = 1; leave; end` 2/2 · `do while 0 = 1` 2/2 ·
`do jj = 1 to 0` 2/2 · `do 0` 2/2 · `do jj = 1 to 3; leave; end` 2/2 ·
`if 1 = 1 then nop` 2/2 · `select … end` 2/2 · plain `do … end` 2/2.

The deliverable-reaching claims beside the table also reproduce:
`interpret "do jj = 1 to 1; nop; end; say 1/0"` one `DO` deep gives oracle 0
against crate 2 and two `DO`s deep oracle 2 against crate 4; the `WHILE` and
`UNTIL` spellings of the fragment give the same; and the delta-0 pair
(`interpret "do jj = 1 to 1; say 2 & 1; end"` echoing 2 and 0 at top level,
6 and 4 two `DO`s deep) is byte-identical.

---

## 3. N2's replacement claim

The corpus witness is byte-identical to the oracle on all three descriptors
as committed (rc 222, outer echo at 2, inner at 4).

**Half one — "the failing `INTERPRET` runs at TOP LEVEL" — FALSE.** The
failing `INTERPRET` is line 52, inside `do kk = 1 to 1` on lines 51-53. Its
printed indent is 2, which the oracle transcript shows and which the *same
file* asserts at lines 13-14: "The INTERPRET sits inside one DO, so it echoes
at 2." The replacement therefore contradicts a claim 19 lines above it in the
same comment.

The mechanism the sentence describes is nevertheless the right one, applied to
the wrong construct: what sits at top level is the **completed** `do kk = 1 to
1` at lines 47-49. Its decrement takes the top-level counter below 0, the
floor absorbs it, and the failing `INTERPRET`'s own level is computed fresh
from 0. Measured directly: deleting lines 47-49 leaves the file byte-identical
to the oracle, so the completed loop contributes nothing where it stands.

**Half two — "Nesting the tail of this file inside another `DO` exposes the
gap at once: measured, 2 and 4" — true under one reading, false under the
other.**

| what is wrapped in an extra `do … end` | oracle | crate | verdict |
|---|---|---|---|
| lines **47-53** (both loops) | 2 and 4 | 4 and 6 | gap exposed; "2 and 4" is exactly right |
| lines **51-53** (the failing loop — "the tail") | 4 and 6 | 4 and 6 | **byte-identical; no gap** |
| control: 47-53 wrapped, completed loop deleted | 4 and 6 | 4 and 6 | byte-identical |

"The tail of this file" most naturally reads as the failing construct, which
is the reading that does not expose anything. The wrap has to start at the
*completed* loop, and the sentence does not say so. A future editor following
it literally will measure a match and conclude the record is wrong.

*Fix:* "What protects THIS file is that the completed `do kk = 1 to 1` on
lines 47-49 sits at TOP LEVEL, where the oracle's counter floors at 0 and its
decrement is absorbed. Wrapping lines 47-53 in one more `DO` — the completed
loop included — exposes the gap at once: oracle 2 and 4, ours 4 and 6.
Wrapping only lines 51-53 does not."

---

## 4. N3's replacement claim

**Reproduced exactly, both halves, and the fourth site too.**

`M4` (`when_indent` reverted to bare `static_indent`, `rexx-run` rebuilt), on
a program with no `INTERPRET` at all:

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
oracle     7 *-*           when 1 = 1        (indent 10)
M4         7 *-*       when 1 = 1            (indent  6)
```

Byte-identical to the oracle on all three descriptors with the mutation
reverted. So the `WHEN`-scan defect was a live 4a divergence before any
fragment base existed, exactly as the new comments say.

The widening claim also reproduces: under `M4`, a plain `do` around
`interpret "select; when 1 = 1 then nop; end; nop"` prints the `WHEN` at 2
where the oracle prints 4.

`M5` (`record_failure_site` computes `static_indent + activation_indent`,
dropping `indent_offset`), on the same nesting with a raising `WHEN`
*condition* and no `INTERPRET`:

```
oracle   6 *-*           when 1/0 = 1     (indent 10)
M5       6 *-*       when 1/0 = 1         (indent  6)
```

So `record_failure_site`'s rewritten doc — "That was false about
`indent_offset` alone, before any fragment base existed" — is true, and the
retraction of the old premise is correct.

---

## 5. File coherence

* `rust/corpus/lang/interpret_error_echo.rex` is **53 lines**.
* `rust/crates/rexx-parse/tests/sourceline_oracle/interpret_error_echo.txt` is
  54 lines: `count 53` followed by content **byte-identical** to the `.rex`
  (`cmp` on the tail, exit 0).
* The corpus program is byte-identical to the oracle on stdout, stderr and
  exit status (rc 222).

---

## 6. Comment claims measured

Every empirical claim the diff adds or changes was measured. True: the
fifteen-row table; the `WHILE`/`UNTIL`-after-a-pass rows; the zero-trip,
`LEAVE` and non-repetitive negatives; the "distinguishing property is a pass
completed" argument (the `LEAVE`-after-a-pass rows are what actually prove it,
and they are not in the table); the literal two-in-sequence accumulation
witness; the outward fragment crossing; the deliverable-reaching fragment
numbers; the delta-0 pair; both `M4` claims; the `M5` claim; the floor; and
the dropped "in exactly one shape" heading qualifier.

False or misleading, all introduced or left standing by this round:

**Important**

* **N8 — "the failing `INTERPRET` runs at TOP LEVEL"** (corpus header line 32
  and its mirror). False, and contradicts line 13-14 of the same file. This is
  the third round in a row in which a correction to a false statement was
  itself false. Detail in §3.
* **N9 — the accumulation generalisation is false for nesting**, in five
  places (§1b). The record has no statement of where a decrement stops, and
  the two scope statements it does have contradict each other.
* **N10 — the KNOWN GAP row still opens with the pre-fix narrow rule.**
  `phase-4-exclusions.txt:346-349`, five lines above the corrected table:
  "A DO that terminates BY EXHAUSTING ITS ITERATIONS decrements the oracle's
  indent counter one time too many". This is the sentence "THE RULE" at 380
  exists to replace, and it was outside every hunk of the diff. A reader who
  stops at the table's lead-in gets the narrow rule; the same row then
  explains at 385-391 why the narrow rule is wrong.

**Minor**

* **N11 — "all seven measured against the oracle"** (`phase-4-exclusions.txt:359`)
  introduces a table that this diff grew to fifteen rows. Unchanged context
  line, now false.
* **N12 — "Nesting the tail of this file inside another DO"** is true only if
  "the tail" starts at the completed loop (§3). Measured both readings.
* **N13 — the inward crossing is unrecorded.** The enclosing program's drift
  reaches into a fragment's base (measured, §1c), and
  `phase-4-exclusions.txt` records only "OUTWARD".
* **N14 — the `CALL` boundary is unrecorded.** The decrement does not cross it
  (measured, §1c), which is a Task 3 fact the row would be the right place for.

**Prior-round minors, all still standing** (this round did not claim them, but
three of the four sit inside hunks this diff edited):

* N4: `activation_indent`'s doc still says `printed_indent` "is the one place
  either is applied" with no `pop_search_frame` note, while `indent_offset`'s
  doc says "no per-site list left to go stale" and then names the exclusion.
* N5: `lib.rs:1713` still says "every one of the first three failed against
  it" over a three-row table whose third cell says "already right".
* N6: `run.rs:1610`, `run.rs:1707` and `run.rs:1973` still open with a literal
  `` `+ self.indent_offset` `` that no longer appears in the code they
  annotate; `run.rs:1707`'s "`0` on the ordinary no-`WHEN`-matched path" is
  still false inside a fragment.
* N7: `printed_indent`'s doc still opens "the six sites that needed
  `+ self.indent_offset` each wrote it out, and one of them … did not" — this
  diff edited the sentence's tail and left the contradiction.

---

## Constraints and gates

| check | result |
|---|---|
| `cargo test --workspace` | all green, 0 failed (255 + 196 + the rest) |
| `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` | **32 of 32 matching** |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `git status --porcelain` | empty, before and after both mutations and at the end |

No `unsafe` added. The C++ tree was not touched. `.Package~new` was never
instantiated on a repository file. The SF #2018 shape
(`select; when 1 = 0 then; when 2 = 2 then nop; end`) was never run — the
`select case 2` shapes here have a matching `WHEN`, and every one exited
cleanly (rc 0 or 214). All scratch files under the session scratchpad. Every
oracle run under `ulimit -v 1048576` with three separate descriptors.
