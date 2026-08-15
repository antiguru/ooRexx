# Task 2 re-review round 3: fix round 3 (`e4caa7bf..e5e1481e`)

Scope: the four claims the round-3 text makes about the oracle's clause-echo
indent, every row of the two committed tables, the shapes the new text lists as
unmeasured, a set of shapes nobody has listed, the corpus witness's coherence,
a tree-wide hunt for surviving copies of the retracted rules, and the truth of
every comment this diff adds or changes.

Tree at review time and at the end: `git status --porcelain` empty. **No file
in the repository was modified at any point in this review** — every probe was
a fresh file in the session scratchpad, and the three corpus variants were
copies made with `sed`/`awk` into the scratchpad, never edits in place. No
mutation of the crate was needed this round, so `rexx-run` was built once from
the committed tree and used unchanged throughout.

84 programs measured. Every oracle run was
`( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`
with stdout, stderr and exit status captured as three separate files and
compared per descriptor with `cmp`. The measured indent is the count of spaces
between `*-* ` and the clause text. Unless stated otherwise the observation
point is `say 1/0` three plain `DO`s deep — **static indent 6** — so the floor
at 0 cannot mask a result, and our crate's own column doubles as the static
value because `static_indent` is a pure function of lexical nesting.

---

## Verdicts

| Claim | Verdict | One line |
|---|---|---|
| **1. Qualification** | **SURVIVED** | No falsifying shape. Every loop that completed a pass and ended on a failing control test decremented; every zero-trip loop, `LEAVE`-terminated loop and non-repetitive block did not. The general clause also correctly covers five spellings it does not enumerate. |
| **2. Scope** | **FALSIFIED** | `do label q … end` — a *plain*, non-repetitive `DO` carrying a `LABEL` — **discards** the decrement at its `END`. The committed sentence asserts the opposite in as many words, for both the plain-`DO` and the `IF`-branch-`DO` case. |
| **3. Corpus-file protection** | **SURVIVED on the mechanism, one sub-clause false as written** | All three measurable halves reproduce exactly. "Deleting those three lines changes nothing" is false literally — it drops a line of stdout and moves every later line number — though the indents, which is what the sentence is about, are unchanged. |
| **4. The two tables** | **ALL 28 ROWS REPRODUCE** | Fifteen qualification rows and thirteen scope rows, each re-measured from a freshly written file. Not one row failed. |

**New findings: Important 2 · Minor 3.** Round-2 minors N13 and N14 are
unaddressed and still stand.

---

## 1. Claim 2 is false, and it is the fourth iteration of the same defect

### 1a. The falsifying shape

```rexx
do
do
do
do label q
do kk = 1 to 1
nop
end
end
say 1/0
end
end
end
```

**oracle 6, ours 6 — zero decrements.** Remove the two words `label q` and
nothing else, and it is oracle 4 against ours 6 — one decrement. The two
programs are otherwise byte-identical.

The committed text says, in both of its copies:

> …propagates OUTWARD through the END of a plain DO block and an IF-branch DO
> block — but is DISCARDED at the END of an enclosing repetitive DO/LOOP and at
> the END of an enclosing SELECT.
> — `docs/superpowers/plans/phase-4-exclusions.txt:416-419`

> A decrement propagates into blocks opened after it and outward through the
> `END` of a plain `DO` or an `IF`-branch `DO`, but is **discarded** at the
> `END` of an enclosing repetitive `DO`/`LOOP` and at the `END` of an enclosing
> `SELECT`.
> — `rust/crates/rexx-exec/src/run.rs:3409-3412`

`do label q … end` is a plain `DO` block: it is not repetitive, it runs its
body exactly once, and it is not a `SELECT`. It satisfies the sentence's
antecedent and does the opposite of what the sentence predicts. This is not an
omission from a list — it is a positive universal claim with a counterexample.

### 1b. It is not confined to the plain-`DO` case

Measured, same observation point, static 6 throughout:

| enclosing construct around the qualifying loop | oracle | static | decrements |
|---|---|---|---|
| `do … end` | 4 | 6 | 1 |
| **`do label q … end`** | **6** | 6 | **0** |
| `if 1 = 1 then do … end` | 4 | 6 | 1 |
| **`if 1 = 1 then do label q … end`** | **6** | 6 | **0** |
| `do label q` with the loop a further plain `do` deep inside | **6** | 6 | **0** |

So the `IF`-branch half of the sentence is false in exactly the same way. Both
of the constructs the sentence names as propagating are propagating only when
**unlabelled**, and the word does not appear.

The record already knows `LABEL` is a spelling in play: row 6 of the
qualification table is `do label q jj = 1 to 1`. The table measured a labelled
*repetitive* loop and the sentence generalised to labelled *plain* blocks
without measuring one — the same shape of error as the previous three rounds.

### 1c. It is a genuine discard, not a suppression, and the level restored is the drifted one

Two controls separate the hypotheses:

* **The inner loop does decrement inside the labelled block.** A clause placed
  after the inner loop's `END` but still inside `do label q` prints at oracle 6
  against static 8 — identical to the unlabelled control. The decrement
  happens; the labelled block's `END` throws it away.
* **The labelled block restores the level it *entered* with, not the lexical
  depth.** A qualifying loop *before* the labelled block, then the block, then
  a clause after its `END`: oracle 4 against static 6. The earlier decrement
  survives the labelled block untouched. And a clause *inside* that block
  prints at oracle 6 against static 8, so the drift propagates inward normally.

That is save-and-restore, not reset.

### 1d. The oracle's source says exactly this, and names the whole class

Read from the C++, not inferred:

* `interpreter/instructions/SimpleDoInstruction.cpp:78-89` — a plain `DO`
  creates a `DoBlock` and calls `newBlockInstruction` **only if it has a
  `LABEL`**; otherwise it calls `addBlockInstruction`, which only steps the
  nesting level.
* `interpreter/execution/RexxActivation.hpp:306` —
  `terminateBlockInstruction` is `popBlockInstruction(); settings.traceIndent = _indent;`
  — an **absolute** restore from the value saved in the `DoBlock`.
* `interpreter/instructions/EndInstruction.cpp:158-170` — `OTHERWISE_BLOCK`,
  `LABELED_OTHERWISE_BLOCK` and **`LABELED_DO_BLOCK`** all go through
  `terminateBlockInstruction`. `LOOP_BLOCK` (line 138) does
  `setIndent(doBlock->getIndent())`, the same absolute restore. The unlabelled
  plain `DO` falls to `default:` and does a relative `removeBlockInstruction`.
* `interpreter/execution/RexxActivation.hpp:318` — `unindent()` clamps at 0.
  That is the floor the corpus row relies on.

So the discarding class is precisely **every construct that restores the indent
from a saved `DoBlock`**: any repetitive `DO`/`LOOP`, `SELECT`/`OTHERWISE`, and
**any labelled `DO`**. The record names the first two and explicitly excludes
the third.

*Suggested replacement, measured:* "…propagates OUTWARD through the END of an
**unlabelled** plain DO block and an **unlabelled** IF-branch DO block — but is
DISCARDED at the END of an enclosing repetitive DO/LOOP, an enclosing SELECT,
and **any DO carrying a LABEL, including a non-repetitive one**. The
distinguishing property is whether the construct restores the counter from a
saved value or backs it off relatively; `SimpleDoInstruction.cpp:78-89` creates
the saved block only for a labelled DO."

### 1e. Does the "what is not known" paragraph protect a reader who meets this?

**No.** The paragraph (`phase-4-exclusions.txt:438-447`) is a good instrument
for the *qualification* rule — it names five unmeasured spellings and says
"MEASURE BEFORE EXTENDING ANY CLAIM HERE", so a reader meeting `DO OVER` is
warned off. But it lists no unmeasured *scope* shape, and the scope sentence
five lines above it makes an unhedged positive claim that a labelled plain `DO`
propagates. A reader who meets `do label q` does not find a gap to fill; they
find an answer, and it is wrong. The hedge protects the half of the record that
was already right.

---

## 2. Claim 1 (qualification) survived every shape

No shape I could build satisfies "completed at least one body pass, then ended
because a control test failed" and fails to decrement, and none outside it
decrements. Beyond the fifteen table rows:

| shape | oracle | static | decrements? | rule right? |
|---|---|---|---|---|
| `do i over .array~of(1,2)` (runs out) | 4 | 6 | yes | yes |
| `do i over .array~new` (empty) | 6 | 6 | no | yes |
| `do with index i item v over .array~of(1,2)` | 4 | 6 | yes | yes |
| `do counter c jj = 1 to 1` | 4 | 6 | yes | yes |
| `do jj = 1 to 1; iterate; end` (ends on its final `ITERATE`) | 4 | 6 | yes | yes |
| `do jj = 1 to 2; iterate; end` | 4 | 6 | yes | yes |
| `do jj = 1 to 10 for 2` (`FOR` budget before the `TO` bound) | 4 | 6 | yes | yes |
| `do jj = 1 to 2 for 2` (`FOR` equals the bound) | 4 | 6 | yes | yes |
| `do jj = 1 to 5 while jj < 3` (`TO` and `WHILE`, the `WHILE` ends it) | 4 | 6 | yes | yes |
| `loop label q jj = 1 to 1` (labelled `LOOP`) | 4 | 6 | yes | yes |
| `do label outer jj = 1 to 3` with a qualifying inner and `leave outer` | 6 | 6 | no | yes |
| the same with `if jj > 0 then leave outer` | 6 | 6 | no | yes |
| three / four / five qualifying loops in sequence | 0 / 0 / 0 | 6 | saturates | see below |

**Enumeration gap, unchanged from round 2 and now wider.** The prose gloss
"count exhausted, `WHILE` false, `UNTIL` true" names three spellings; `DO
OVER`, `DO WITH`, `DO COUNTER`, a hit `FOR` budget and a final `ITERATE` all
decrement and none is named. The *general* clause covers them, so the rule is
not falsified — but the corpus header states only the gloss, in em-dashes that
read as the complete list, and 4c implements `DO OVER`.

**"Siblings accumulate" saturates at the floor, and that is not said where the
claim is made.** Three, four and five qualifying loops in sequence at static 6
all give oracle **0**, not −0/−2/−4: `unindent()` clamps. The floor is stated
elsewhere in the same row (line 467) and in the corpus header, but the SCOPE
paragraph's accumulation sentence carries no bound. Minor, because the two
places are close together.

---

## 3. Claim 3 (corpus-file protection)

The committed program is byte-identical to the oracle on stdout, stderr and
exit status (rc 222; outer echo at 2, inner at 4). Lines 47-49 are the
completed `do kk = 1 to 1` wrapping `interpret "say 'inside a DO'"`; lines
51-53 are the failing loop. Both cited spans are correct, and the completed
loop does sit at top level. Measured, all four variants:

| variant | oracle | ours | verdict |
|---|---|---|---|
| as committed | 2 and 4 | 2 and 4 | identical, rc 222 |
| lines 47-49 deleted | 2 and 4 | 2 and 4 | identical — the completed loop contributes nothing where it stands |
| **lines 47-53 wrapped in one more `DO`** | **2 and 4** | **4 and 6** | gap exposed, exactly the numbers claimed |
| lines 51-53 only wrapped | 4 and 6 | 4 and 6 | identical — no gap, exactly as claimed |
| control: 47-53 wrapped, completed loop deleted | 4 and 6 | 4 and 6 | identical |

The mechanism sentence is right and the three predictions are right. The
round-2 error ("the failing `INTERPRET` runs at TOP LEVEL") is gone and its
replacement is accurate.

**Minor — "deleting those three lines changes nothing" is false as written**
(`rust/corpus/lang/interpret_error_echo.rex:31-32` and its mirror at
`…/sourceline_oracle/interpret_error_echo.txt:32-33`). Deleting lines 47-49
removes `inside a DO` from stdout and moves the error report's line number from
52 to 49 on both echoes and the major line. What the measurement supports is
"the file still matches the oracle" / "the indents are unchanged" — which is
the point being made, and is what the round-2 review actually measured. As
phrased it is a sentence reaching one step past its row, in a comment whose
subject is sentences reaching past their rows.

---

## 4. The tables

### 4a. Fifteen-row qualification table — all fifteen reproduce

`say 1/0` one `do` deep, one file per row, oracle | ours:

`do jj = 1 to 1` 0|2 · `do 2` 0|2 · `n=0; do while n = 0; n = 1; end` 0|2 ·
`n=0; do until n = 1; n = 1; end` 0|2 · `do jj = 1 to 1 while jj < 5` 0|2 ·
`do label q jj = 1 to 1` 0|2 · `do forever … leave` 2|2 ·
`n=0; do while n = 0; n = 1; leave; end` 2|2 · `do while 0 = 1` 2|2 ·
`do jj = 1 to 0` 2|2 · `do 0` 2|2 · `do jj = 1 to 3; leave; end` 2|2 ·
`if 1 = 1 then nop` 2|2 · `select … end` 2|2 · plain `do … end` 2|2.

The nine `2|2` rows are byte-identical to the oracle on all three descriptors.

### 4b. Thirteen-row scope table — all thirteen reproduce

Three plain `DO`s deep, static 6, one file per row:

| row | oracle | ours | decrements |
|---|---|---|---|
| one qualifying loop | 4 | 6 | 1 |
| two in sequence | 2 | 6 | 2 |
| two nested | 4 | 6 | 1 |
| two inners in sequence inside a qualifying outer | 4 | 6 | 1 |
| inner inside `do jj = 1 to 3 … leave` | 6 | 6 | 0 |
| inner inside `do forever … leave` | 6 | 6 | 0 |
| inner inside `select`/`when … then do` | 6 | 6 | 0 |
| inner as the `WHEN` body directly | 6 | 6 | 0 |
| inner inside an `OTHERWISE` body | 6 | 6 | 0 |
| inner inside a plain `do … end` | 4 | 6 | 1 |
| inner inside two nested plain `do … end` | 4 | 6 | 1 |
| inner inside `if 1 = 1 then do` | 4 | 6 | 1 |
| loop after a `select … end` at the same level | 4 | 6 | 1 |

### 4c. The two "visibly decrements inside the enclosing construct" witnesses

Both reproduce to the digit: **oracle 6 against static 8** inside the
`LEAVE`-terminated loop, **oracle 12 against static 14** inside the `WHEN`
body. I also measured the two rows the sentence covers but does not cite: the
`do forever … leave` row gives oracle 6 against static 8, and the `OTHERWISE`
row gives oracle 8 against static 10 — both visible, both consistent.

*Minor:* the sentence says "In the LEAVE and SELECT rows the inner loop DOES
decrement; it is visible on a clause still inside the enclosing construct."
That covers five table rows, and one of them — "qualifying inner as the `WHEN`
body directly" — admits no such clause, because the loop *is* the entire body.
The claim is unfalsifiable rather than false there. Worth one qualifying word.

---

## 5. The shapes listed as unmeasured — four of five decrement, and four of five were already measured

`phase-4-exclusions.txt:444-446` lists as still unmeasured: `DO OVER` running
out, `DO WITH`, a loop left by `SIGNAL` or `RETURN`, `ITERATE` on the final
pass, and a `FOR` budget reached before the `TO` bound.

| listed shape | measured now | result |
|---|---|---|
| `DO OVER` running out | yes | **decrements** (oracle 4, static 6) |
| `DO WITH` | yes | **decrements** (4, 6) |
| `ITERATE` on the final pass | yes | **decrements** (4, 6) |
| `FOR` budget before the `TO` bound | yes | **decrements** (4, 6) |
| a loop left by `SIGNAL` | yes | **no observable effect, structurally** |
| a loop left by `RETURN` | yes | **no observable effect, structurally** |

The `SIGNAL` and `RETURN` entries have an answer better than "unmeasured":

* **`SIGNAL` cannot carry it, ever.** `RexxActivation.cpp:2105-2112` clears
  `doStack`, zeroes `blockNest` and sets `settings.traceIndent = 0` on every
  `SIGNAL`. Measured three ways — a loop left by `SIGNAL`, a qualifying loop
  completed *before* a `SIGNAL`, and a `SIGNAL ON SYNTAX` trap firing from
  inside a loop body — all three land at oracle 2 in a `do … end` after the
  label, identical to the no-loop control. This also disposes of "a loop whose
  body raises and is trapped": for `SYNTAX` the trap *is* a `SIGNAL`, and
  `CALL ON SYNTAX` is not legal REXX (Error 25.1), so the resume-inside-the-loop
  variant does not exist for this condition.
* **`RETURN` cannot carry it across a `CALL`.** A qualifying loop in a called
  routine, and a `RETURN` out of a live loop in a called routine, both give
  oracle 6 at the call site — identical to a routine containing only `nop`. The
  counter is per-activation for a `CALL` and shared for an `INTERPRET`.

**Important — the paragraph asserts ignorance the record had already cured.**
Four of these five were measured in `task-2-rereview2.md` §1a, the very
document this fix round responds to, with the same results (`DO OVER` 4, `DO
WITH` 4, `FOR` budget, `iterate`). Writing them into a "WHAT IS NOT KNOWN" list
discards a measurement that was in hand and tells the next reader to redo it.
The list should be the two `SIGNAL`/`RETURN` entries, restated as findings
rather than gaps, plus the scope shapes nobody has tried.

---

## 6. Shapes nobody had listed

| shape | oracle | static | result |
|---|---|---|---|
| **qualifying loop inside `do label q … end`** | **6** | 6 | **falsifies claim 2** (§1) |
| qualifying loop inside `if 1 = 1 then do label q … end` | 6 | 6 | falsifies claim 2 |
| `loop label q jj = 1 to 1` (labelled `LOOP`) | 4 | 6 | decrements; rule right |
| `do jj = 1 to 5 while jj < 3` (`TO` + `WHILE`, `WHILE` ends it) | 4 | 6 | decrements; rule right |
| `LEAVE outer` from an inner loop, both spellings | 6 | 6 | no decrement; rule right |
| loop ending on its final `ITERATE` | 4 | 6 | decrements; rule right |
| loop body raises, `SIGNAL ON SYNTAX` traps | 2 | 2 | counter zeroed by the `SIGNAL` |
| qualifying loop inside an `INTERPRET` fragment, enclosing program continues | 4 | 6 | **crosses outward** — confirms the recorded claim |
| `if 1 = 1 then do kk = 1 to 1 … end` (the loop *is* the `IF` branch) | 4 | 6 | decrements and propagates |
| qualifying loop then a `SELECT` opened after it, observed inside the `WHEN` | 12 | 14 | propagates inward |
| the same, observed after the `SELECT`'s `END` | 4 | 6 | survives the `SELECT` it did not enclose |
| outer loop running three passes around a qualifying inner | 4 | 6 | 1, not 2 — nesting still does not accumulate |

The last two are worth keeping: a `SELECT` discards only a decrement made
*inside* it, and an enclosing loop's own qualification adds one decrement
regardless of how many its body made. Both are consistent with the committed
text.

---

## 7. Stale copies — one survives, in the most-read document of the phase

The two the fix claims to have removed are genuinely gone: the KNOWN GAP row's
lead-in now states the corrected predicate (`phase-4-exclusions.txt:346-352`),
and the table is introduced by "every row measured against the oracle"
(line 358). N10 and N11 are addressed.

**Important — `docs/superpowers/plans/2026-08-03-phase-4b-procedures-and-conditions.md:148`
is a sixth copy and still carries the retracted rule verbatim:**

> **The rule, measured, and stated wrongly twice before this.** Any repetitive
> `DO`/`LOOP` that … decrements the oracle's running counter once, and the
> decrements **stack**. … Two exhausted controlled `DO`s in sequence two `do`s
> deep give oracle 0 against our 4.

Three things are wrong with it: "the decrements **stack**" is the exact claim
round 3 retracted; "stated wrongly twice before this" is now three times; and
the paragraph below it re-tells the round-2 history as if it were current.
The literal witness in the last sentence is true (it is the sibling case), but
it is the same true-witness-under-a-false-generalisation pairing that this
whole sequence of rounds exists to undo.

This is not an untouched file. **Commit `500e1743` in this very review range
edits it**, adding Step 3b at line ~1012. The author was in the file.

Line 588 of the same document points readers at the exclusions row for "the
measured tables, the scope rule, and a list of shapes still unmeasured" — so
the plan now contradicts itself, stating the retracted rule at 148 and
deferring to the corrected one at 588.

Everything else is clean. I searched the whole tree for `exhaust`,
`accumulat`, `stacks`, `at that level`, `two spaces lower`, `later clause`,
`all seven`/`all fifteen`, `re-tested pass` and the test identifier
`the_indent_after_a_loop_has_already_exited_is_not_left_over_from_it` across
`.rs`, `.txt`, `.md`, `.rex` and `.toml` in `docs/`, `rust/` and the phase's
`progress.md`. No other restatement survives; the remaining hits are unrelated
uses of the words, or correct pointers to the exclusions row.

---

## 8. Every claim this diff adds or changes

**Measured true:** all fifteen qualification rows and all thirteen scope rows;
the "a pass completed, not a re-test failed" argument and its
`LEAVE`-after-a-pass evidence; the nested-loop falsifying shape quoted at
`phase-4-exclusions.txt:426-433` (oracle 6 = static 6, reproduced exactly); the
two-in-sequence sibling witness; "siblings accumulate, nesting does not" for
the two named cases; the discard at an enclosing repetitive `DO`/`LOOP`'s `END`
and at a `SELECT`'s `END`; both "visibly decrements inside" witnesses to the
digit; the outward `INTERPRET` crossing; the deliverable-reaching fragment
numbers (one `DO` deep 0 vs 2, two deep 2 vs 4); the delta-0 pair; all three
corpus-variant predictions; the line-span citations 47-49 and 51-53; the floor.

Step 3b's figures, added by `500e1743`, all check out: `owners.rs` +
`loud.rs` + `coverage.rs` = **1,852** lines at `e4caa7bf`; `run.rs` +
`eval.rs` + `error.rs` = 6,706 + 1,560 + 759 = **9,025**; Task 0
(`dc87708d..0197b360`) = **907 insertions**. "Zero interpreter functionality"
is fair — 126 of those insertions are in production `src/lib.rs`, but they are
the owner-string match Step 3b itself describes, not interpreter behaviour.

**False or reaching past their rows:**

* **F1 (Important).** The scope sentence's "propagates OUTWARD through the END
  of a plain DO block and an IF-branch DO block" —
  `phase-4-exclusions.txt:416-419` and `run.rs:3409-3412`. False for a labelled
  plain `DO` and a labelled `IF`-branch `DO`. §1.
* **F2 (Important).** `2026-08-03-phase-4b-procedures-and-conditions.md:148`
  still says the decrements "stack" and that the rule was "stated wrongly twice
  before this". §7.
* **F3 (Important).** `phase-4-exclusions.txt:444-446` lists four shapes as
  unmeasured that `task-2-rereview2.md` §1a had already measured, all of which
  decrement. §5.
* **F4 (Minor).** "deleting those three lines changes nothing" —
  `interpret_error_echo.rex:31-32` and its `.txt` mirror. §3.
* **F5 (Minor).** "In the LEAVE and SELECT rows the inner loop DOES decrement;
  it is visible on a clause still inside the enclosing construct" —
  `phase-4-exclusions.txt:419-423`. Unfalsifiable for the row where the loop is
  the whole `WHEN` body. §4c.

`run.rs:806-823` — the fragment-base comment's new scope paragraph — is the one
copy that escapes F1. It names the discarding constructs without asserting that
anything else propagates, so it is incomplete rather than false. It should
still gain the labelled-`DO` case.

**Round-2 minors not claimed by this round and still standing:** N13, the
inward `INTERPRET` crossing, is still recorded nowhere (re-measured: a
qualifying loop in the enclosing program lowers a *fragment's* base — oracle
2 and 2 against ours 4 and 4, where the no-loop control is 4 and 4 identical).
N14, that the decrement does not cross a `CALL` boundary, is still recorded
nowhere (re-measured, §5). N4-N7 were not in this round's scope.

---

## 9. File coherence, constraints and gates

* `rust/corpus/lang/interpret_error_echo.rex` — **53 lines**.
* `rust/crates/rexx-parse/tests/sourceline_oracle/interpret_error_echo.txt` —
  54 lines: `count 53` followed by content **byte-identical** to the `.rex`
  (`tail -n +2 … | cmp -` exit 0).
* The corpus program matches the oracle on stdout, stderr and exit status
  (rc 222), verified per descriptor.

| check | result |
|---|---|
| `cargo test --workspace` | exit 0, no failures |
| `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` | exit 0, **32 of 32 matching** |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `git status --porcelain` | empty at the start, and empty again after the last measurement — see the note below |

**Note on the working tree at the end.** Every measurement and all four gates
above ran against the committed tree, and `git status --porcelain` was verified
empty immediately after the last probe. Some minutes later — while this report
was being written — `rust/crates/rexx-exec/src/{activation,lib,plan,run,trace}.rs`
appeared as modified, mtimes 14:03-14:04. **None of those edits is mine**: this
review wrote exactly one file, this report, and `.superpowers/` is gitignored
(`.gitignore:19`), which is why the report itself does not show in
`git status`. The changes are a concurrent agent's implementation work in this
same session and have deliberately been left alone rather than restored.

No `unsafe` added or present in anything touched. The C++ tree was read
(`interpreter/instructions/SimpleDoInstruction.cpp`,
`interpreter/instructions/EndInstruction.cpp`,
`interpreter/execution/RexxActivation.{hpp,cpp}`) and **never modified** —
confirmed by the empty `git status` in that repository's own working tree being
irrelevant here, since no write tool was pointed at it. `.Package~new` was never
instantiated. The SF #2018 shape (`select; when 1 = 0 then; when 2 = 2 then
nop; end`) was never run: every `SELECT` probe here has a matching `WHEN` with a
non-empty `THEN`, and the one `OTHERWISE` probe uses `when 1 = 0 then nop`. No
symbol named `x` or `b` precedes a quoted string in any probe. All scratch files
live under the session scratchpad. Every oracle run used `ulimit -v 1048576`
and three separate descriptors; exit statuses were read unpiped.

---

## 10. What the next round has to do

1. Add "unlabelled" to the two scope sentences, or replace them with the
   restore-versus-back-off statement in §1d, which is what the oracle actually
   implements and covers every shape measured in three rounds.
2. Fix `2026-08-03-phase-4b-procedures-and-conditions.md:148`.
3. Replace the four already-measured entries in "WHAT IS NOT KNOWN" with their
   answers, and put the two `SIGNAL`/`RETURN` structural findings there instead.
4. Reword "deleting those three lines changes nothing" to "leaves the file
   still byte-identical to the oracle".

None of these is a code change. The crate's behaviour is unaffected by all of
it: `static_indent` computes the lexical indent, the divergence is 4a's, and
every gate is green.
