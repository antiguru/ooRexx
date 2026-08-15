# Task 2b review: normalise leading indentation in the differential harnesses

Reviewed `fe09f445..0830a12e` against `task-2b-brief.md` and DEVIATION 0 in
`docs/superpowers/plans/phase-4-exclusions.txt:79`-`155` (its SCOPE paragraph,
`:91`-`97`, read as the specification).

Every claim below was run, not reasoned about. Oracle probes were each run
from a fresh empty subdirectory under the scratchpad, wrapped in the required
`ulimit`/`LD_LIBRARY_PATH` form, with stdout, stderr and exit status read as
three separate descriptors. The C++ tree was read, never modified. Working
tree was clean before and after; the only change left behind is this file.

---

## Verdicts

**1. Spec compliance — CONDITIONAL PASS.** Everything the brief asks for is
present, at the two sites it names and nowhere else, with regeneration
untouched and no figure moved. Two things block an unconditional pass: the
function normalises **more than SCOPE licenses** in one constructible case
(**I1**, proven with real oracle bytes), and the normative DEVIATION 0 row it
rewrites gained a claim that contradicts the sentence three lines above it
(**I2**). Neither needs a redesign.

**2. Task quality — PASS.** `cargo fmt --all --check` exit 0,
`cargo clippy --workspace --all-targets -- -D warnings` exit 0, 891 passed /
0 failed / 4 ignored. All three required negative controls are **live** under
mutation (proven below, not assumed). Comments are accurate except for I2.
No dead code, no duplication, no YAGNI beyond the 19-marker table, whose
generosity is a defensible verbatim transcription of the C++ table rather
than a judgement call. One test-hygiene gap (**M3**).

**Counts: 0 Critical, 2 Important, 6 Minor.**

---

## The central question: what can this hide that DEVIATION 0 does not license?

### Divergence classes proven NOT hidden

A 36-test hostile battery was run against a copy of the shipped function in a
standalone crate outside the repo. 34 passed — i.e. the function correctly
distinguishes each of these:

| # | class | probe |
|---|---|---|
| 1 | a missing line | `adv_missing_line` |
| 2 | an extra line | `adv_extra_line` |
| 3 | an extra **blank** line | `adv_extra_blank_line` |
| 4 | two adjacent lines reordered | `adv_reordered_adjacent_value_lines` |
| 5 | a changed value inside a `>>>` line | `adv_changed_value_inside_result_line` |
| 6 | changed clause text | `adv_changed_clause_text` |
| 7 | a changed line number | `adv_changed_line_number` |
| 8 | a changed marker (`>>>` vs `>L>`) | `adv_changed_marker` |
| 9 | a changed tag marker (`" => "` vs `" <= "`) | `adv_changed_tag_marker_arrow` |
| 10 | a value whose content **begins with spaces inside its quotes** | `adv_value_content_begins_with_spaces_inside_quotes` |
| 11 | an all-space quoted value of a different width | `adv_value_content_is_only_spaces` |
| 12 | internal spacing inside clause text (`say      3`) | `adv_internal_spaces_in_clause_text_preserved` |
| 13 | presence/absence of the trailing newline | `adv_no_trailing_newline_preserved` |
| 14 | `\r\n` vs `\n` | `adv_crlf_line_ending_content_preserved` |
| 15 | non-UTF-8 content bytes (no panic, difference kept) | `adv_non_utf8_bytes_survive` |
| 16 | tab runs (tabs are not spaces, not collapsed) | `adv_tab_is_not_a_space` |
| 17 | a zero-length space run vs a one-space run | `adv_zero_space_run_does_not_grow_a_space` |
| 18 | a marker at an offset other than 7 (7-digit line number) | `adv_marker_beyond_offset_seven_is_not_normalised` |
| 19 | near-miss markers (`***`, `>>V`, `<I>`, `>i>`, …) | `adv_near_miss_markers_are_not_normalised` |
| 20 | both `error.rs::report` banner forms, four variants | `adv_error_banner_untouched` |
| 21 | newline count is invariant (cannot merge/drop/invent lines) | `adv_line_count_is_invariant` |
| 22 | empty input, bare `\n`, `\n\n`, lines shorter than the marker offset, a line truncated exactly at the marker end | `adv_empty_input`, `adv_bare_newline_roundtrips`, `adv_line_shorter_than_marker_offset`, `adv_truncated_exactly_at_marker_end` |
| 23 | an enormous (100 000-byte) space run | `adv_enormous_space_run` |
| 24 | idempotence | `adv_idempotent` |

Item 10 deserves a note because it was the most likely place for this to go
wrong and it does not: on a single-line value the `"` is the first non-space
byte, so `take_while(|&&b| b == b' ')` stops there and the value's own leading
spaces survive. Confirmed against real oracle bytes —
`       >L>   "       >>>   z"` keeps all seven interior spaces.

### Marker set: complete and correct

`TRACE_PREFIXES` (`tests/support/mod.rs:37`-`57`) was compared against the
authority, `interpreter/execution/RexxActivation.cpp:3567`-`3588`. All
nineteen entries match in spelling **and order**, and each carries its C++
enumerator name as a comment. `PREFIX_OFFSET = 7` and `PREFIX_LENGTH = 3`
match `RexxActivation.cpp:3591`-`3595` (`LINENUMBER = 6`,
`PREFIX_OFFSET = LINENUMBER + 1`). No marker is missing; none is wrongly
included.

The two markers this crate cannot emit but which are the most "content-like"
(`+++` from `traceSourceString`, `RexxActivation.cpp:4024`, and `>I>`/`<I<`
from `traceEntryOrExit`, `:3714`) both place their content at
`INSTRUCTION_OVERHEAD = 11` — exactly one space after the marker and no
indent run — so normalising them is a provable no-op rather than a risk.

### The class it DOES hide — see I1

---

## Findings

### IMPORTANT

#### I1 — The normalisation reaches past trace lines: the 2nd and later physical lines of a traced value or clause whose text contains a newline have their **content** collapsed

`rust/crates/rexx-exec/tests/support/mod.rs:124`-`153` (`normalize_line`),
reached from `normalize_stderr` at `:97`-`117`.

`normalize_stderr` splits stderr on `\n` and asks one question per physical
line: are bytes `7..10` a known marker? That question cannot tell a trace
line from **the tail of a quoted value that itself contains a newline**.

Measured, not reasoned. This program is entirely inside 4a/4b's scope — a hex
literal, a concatenation and `TRACE I`:

```rexx
trace i
x = '0a'x || "       >>>   z"
```

Oracle stderr (`cat -A`, run from a fresh empty directory):

```
     2 *-* x = '0a'x || "       >>>   z"$
       >L>   "$
"$
       >L>   "       >>>   z"$
       >O>   "||" => "$
       >>>   z"$
       >>>   "$
       >>>   z"$
       >=>   X <= "$
       >>>   z"$
```

`rexx-run` produces the identical bytes. Four of those ten lines
(`       >>>   z"`) are **not trace lines** — they are the second physical
line of a quoted value — and each carries `>>>` at offset 7 followed by a
space run. `normalize_line` collapses that run, which is the *string's own
data*.

Consequence, demonstrated with the real oracle bytes above as one side and,
as the other, the same bytes with only those continuation lines widened by
two spaces — exactly what a value-rendering or concatenation bug on our side
would produce, leaving the `*-*` source echo byte-identical:

```
adv_HOLE_value_content_after_an_embedded_newline_is_eaten ... FAILED
  HOLE CONFIRMED: a pure string-VALUE content divergence is hidden.
adv_HOLE_minimal ... FAILED
```

`check_case` would report those two runs as matching.

DEVIATION 0's SCOPE (`phase-4-exclusions.txt:91`-`97`) names "the clause
text" and "the value lines' CONTENT" as staying byte-exact. This hides both.
The row's own WHY paragraph (`:119`-`122`) is the reason it matters: value
lines "carry intermediate results and evaluation order that NOTHING else in
the output exposes."

Scope of the exposure, stated fairly: it needs (a) a newline inside traced
text and (b) the bytes after that newline to be ≥ 10 long with a known marker
at 7..10 and a space run after it. No corpus `.rex` embeds a newline in
traced text today (`grep` for `'0a'x` and friends across `rust/corpus`
returns only `errors/parse-errors.tsv`), and the corpus is 33 of 33 either
way. So this is **latent, not active** — which is why it is Important and not
Critical. It is also permanent and silent, which is why it is not Minor.

**Fix (real).** Carry one bit of state across lines in `normalize_stderr`: a
trace record that opens a quote and does not close it before end-of-line
continues onto the next physical line, and lines inside such a region must be
returned untouched. Concretely — count `"` (and, for `*-*` lines, `'`) in the
part of a qualifying line at and after the space run; an odd count opens the
region, the next odd count closes it; skip `normalize_line` while open. That
closes both the value case and the INTERPRET-fragment case (a fragment whose
clause text contains a newline produces `     3 *-* say '`, likewise
unbalanced — measured).

**Fix (minimum acceptable).** If the state machine is judged too much
machinery for a latent hole, then disclose it instead of leaving it silent:
add a rule to DEVIATION 0's SCOPE in the same shape DEVIATION 1 already uses
for `DO OVER` ("Consequence, and it is a rule rather than an observation: NO
CORPUS PROGRAM MAY …") — no corpus program and no `trace_oracle` witness may
put a newline into traced text — and pin a test in `tests/support/mod.rs`
that records the hole. A disclosed limit is a different object from a silent
one.

#### I2 — DEVIATION 0's rewritten row contains a claim that contradicts the sentence three lines above it

`docs/superpowers/plans/phase-4-exclusions.txt:136`-`137`, repeated verbatim
at `rust/crates/rexx-exec/tests/corpus.rs:158`-`159` and in
`task-2b-report.md:144`-`145`.

The row now says:

> `one_two_and_three_enclosing_dos_indent_by_two_four_and_six` (precisely
> this row's own oracle-counter shape)

But `:129`-`131`, three lines above, requires the pinned witnesses to contain
**NO completed loop**, and the counter defect this row documents
(`BaseDoInstruction.cpp:161` vs `:377`, `phase-4-exclusions.txt:99`-`113`)
only fires when a loop **completes at least one body pass** and then ends on
a failing control test. The two statements cannot both be true of the same
test.

The test (`rust/crates/rexx-exec/src/run.rs:6892`-`6906`) runs

```
do i = 1 to 3      do i = 1 to 3       do i = 1 to 3
say 1/0            do j = 1 to 3       do j = 1 to 3
end                say 1/0             do k = 1 to 3
                   end                 say 1/0
                   end                 end / end / end
```

and asserts indent 2 / 4 / 6. The failure is on the **first** iteration; no
pass ever completes; no loop ever ends. It is the correct witness for what
`:129`-`131` asks for, and it is *not* the counter-defect shape. The row's
own measured example of that shape (`:493`-`498`) is a different program.

This project's own record is the reason to rank this Important rather than
Minor: `:124`-`127` and `:526`-`532` record that the indent rule was stated
five different ways and was wrong every time, and this file is asserted by
`coverage.rs`/`loud.rs` precisely because decisions here have gone missing
before.

**Fix.** In both `phase-4-exclusions.txt:136` and `corpus.rs:158`, replace
the parenthetical with what the test is: "plain nested `DO`s around a failing
clause, no completed loop — the shape the sentence above requires".

---

### MINOR

#### M1 — The three pinned witnesses assert the indent *quantity*, not rendered bytes; the byte-level pin exists but is not named

`run.rs:6903`-`6904`, `run.rs:5452`-`5453` and `run.rs:6633`ff all destructure
`FailureSite { indent, .. }` and compare an integer. The brief's claim that
`run.rs`'s unit tests "assert exact `interp.trace` bytes with literal leading
spaces (16 such assertions)" is not true of these three. The implementer did
**not** propagate that claim — `corpus.rs:154` says "asserting an exact
`FailureSite`/trace indent" and the row says "asserting an exact indent",
both accurate — which is the right call and worth recording.

The consequence is unstated, though: nothing in the pinned set pins the
*rendering*. A `push_clause` spacing regression (say, three spaces per level
instead of two) would leave all three green, and on the corpus and
`trace_oracle` paths the normalisation now hides it. It is not unguarded —
`trace.rs:546`-`577`'s `every_formatter_matches_its_own_oracle_transcript`
asserts literal bytes with leading spaces for all six shapes including
indent 4, and is a unit test outside both comparison functions — but nothing
says so.

**Fix.** Name `trace::tests::every_formatter_matches_its_own_oracle_transcript`
in DEVIATION 0's WHAT SURVIVES paragraph as the byte-level half of the
surviving set.

#### M2 — On the `trace_oracle.rs` path the normalisation is a no-op today and strictly removes coverage

All five committed `.expected` files were regenerated from the live oracle
using the recipe in `trace_oracle.rs:78`-`87` and compared:

```
compound_read_write:         RAW ORACLE BYTES (identical)
dotvariable_beyond_the_list: RAW ORACLE BYTES (identical)
keyword_while:               RAW ORACLE BYTES (identical)
prefix_operators:            RAW ORACLE BYTES (identical)
trace_output:                RAW ORACLE BYTES (identical)
```

They are raw oracle bytes, and our own output already matched them
byte-exactly (baseline: 879 passed, 0 failed). So on this path the change
buys nothing now and removes byte-exactness from five witnesses — including
`keyword_while`, whose `WHILE` loop is exactly the counter-defect shape. The
brief names the file, so this is licensed, not a spec violation. But the
report's "Did any previously-failing shape start passing?" section
(`task-2b-report.md:159`-`172`) reasons only about the corpus.

**Fix.** One sentence in the report and in DEVIATION 0's IMPLEMENTED
paragraph: on the `trace_oracle` path the normalisation is currently inert and
is applied for consistency, prospectively.

#### M3 — No negative control covers an over-permissive **merge**

Mutation-testing the shipped controls (each mutation applied to a copy of
`normalize_stderr` in a standalone crate; the six committed tests then run):

| mutation | property broken | committed controls |
|---|---|---|
| `ls.sort()` | line order | `a_reordered_line_still_differs` **FAILED** — live |
| drop every `>>>` line | line presence | `a_missing_line_still_differs` **FAILED** — live |
| truncate each line to 10 bytes | line content | `a_changed_value_still_differs` **FAILED** — live |
| identity function | the deviation buys anything | both positive controls **FAILED** — live |
| return `Vec::new()` | everything | 4 of 6 **FAILED** |
| squeeze every space run in the whole stream | reach outside trace lines | `a_non_trace_line_is_untouched` **FAILED** — live |
| **`ls.dedup()`** | line presence, by merging | **all six pass** |

All three controls the brief requires are genuinely load-bearing — none is
vacuous. The gap is `ls.dedup()`: `base_transcript()`
(`tests/support/mod.rs:170`-`176`) has two distinguishable lines, so a
merge-shaped over-permissiveness is unguarded. A repetitive `DO FOREVER` body
does emit consecutive identical `*-*` re-echoes, so this is not a purely
theoretical shape.

**Fix.** Append a third line identical to the second in `base_transcript()`,
or add one control asserting `normalize_stderr(A ++ A) != normalize_stderr(A)`.

#### M4 — A second, differently-scoped stderr normalisation already exists in the tree and is not mentioned

`rust/crates/rexx-oracle/src/normalize.rs:22`-`35` folds `\r\n` to `\n` and
rewrites the cwd to `<CWD>` on **stderr**, and is consumed by
`rust/crates/rexx-oracle/src/bin/rexx-diff.rs:15`. DEVIATION 0 at
`phase-4-exclusions.txt:96`-`97` says it "must not be cited as precedent for a
second normalisation", and `tests/support/mod.rs:22`-`24` calls itself "the
one normalisation the differential harnesses are allowed to apply to
`stderr`".

Neither statement is wrong about the two harnesses under review — verified by
grep, `normalize_stderr` has exactly two call sites, `corpus.rs:375` and
`trace_oracle.rs:177`-`178`, both stderr comparisons — but `rexx-diff` is a
third differential path with different stderr semantics that applies neither
normalisation, and no file says so.

**Fix.** One clause in DEVIATION 0's IMPLEMENTED paragraph naming
`rexx-diff`'s pre-existing path-and-CRLF normalisation as a separate, older
thing this row does not govern.

#### M5 — Line numbers ≥ 1 000 000 fall out of the normalisation silently

`push_clause` (`trace.rs:254`) formats `{line:>6}`, which overflows to seven
bytes past 999 999, pushing the marker to offset 8. `normalize_line` then
finds no marker at 7..10 and returns the line untouched, so those lines are
compared byte-exactly — safe, but inconsistent with every other line, and
undocumented. Proven by `adv_marker_beyond_offset_seven_is_not_normalised`.

**Fix.** One sentence on `PREFIX_OFFSET`'s doc comment
(`tests/support/mod.rs:59`-`68`).

#### M6 — The +12 accounting is correct but lives only in the task report

Confirmed independently rather than accepted: `support::tests::` appears
exactly 12 times in the full `cargo test --workspace` log — six distinct
names under `Running tests/corpus.rs` and the same six under
`Running tests/trace_oracle.rs` — and the diff adds exactly six `#[test]`
functions and modifies no other test. 879 + 12 = 891. Verified figures:

```
TOTAL passed=891 failed=0 ignored=4
4224 of 4259 rows passing -- REPORT MODE, NOT THE GATE
33 of 33 matching -- REPORT MODE, NOT THE GATE
cargo fmt --all --check       exit 0
cargo clippy --workspace --all-targets -- -D warnings   exit 0
```

Nothing in the code or in the exclusions row records that the headline count
now double-counts six tests, so the next person to compare 891 against a
future figure has to rediscover it.

**Fix.** One sentence in `tests/support/mod.rs`'s module doc.

---

## The flagged concern: the third file

**The implementer is right, and the mechanism is sound.**

Right on the substance: `trace.rs:246`-`252` records that `error.rs::report`
held a second copy of `push_clause`'s four lines, that the two drifted, and
that 4b Task 2 had to clamp one quantity and found the other had not been.
This function is compared *against itself* across two call sites — if the two
copies ever disagreed, one harness would call two streams equal and the other
would not, on the same bytes. That is the same defect shape, in the one place
it would be hardest to notice. Duplicating it would have been the worse call,
and the brief's `Files:` list is a list, not a prohibition.

Sound on the mechanism, verified rather than assumed: Cargo auto-discovers
only direct children of `tests/` as integration-test targets, so
`tests/support/mod.rs` adds no third binary. The full run's target list
confirms it — `Running tests/corpus.rs` and `Running tests/trace_oracle.rs`
appear, `tests/support.rs` does not.

The one real consequence — `#[cfg(test)]` is active in an integration-test
binary, so the six control tests compile and run **twice** — is harmless
(they are pure functions over byte literals, and a failure would simply be
reported twice) and is correctly accounted for in the report. It is recorded
in the wrong place, which is M6, not a problem with the decision.

---

## Verified claims in DEVIATION 0's IMPLEMENTED row

Each claim at `phase-4-exclusions.txt:81`-`89`, checked against the code:

| claim | verdict |
|---|---|
| "`tests/support/mod.rs` … is the normalising function" | True. |
| "shared by `corpus.rs` and `tests/trace_oracle.rs`" | True — `mod support;` at `corpus.rs:174` and `trace_oracle.rs:104`. |
| "applied at both harnesses' own stderr comparison and nowhere else" | True. Exactly two call sites workspace-wide: `corpus.rs:375`, `trace_oracle.rs:177`-`178`. Both are stderr comparisons; `stdout` and `exit_code` at `corpus.rs:369`/`378` and `trace_oracle.rs:175`/`181` are untouched. See M4 for the one caveat (a different, older normalisation elsewhere in the tree). |
| "never on a `.expected` file's own regeneration, which stays raw oracle bytes" | True, and stronger than "there is no regeneration code": all five `.expected` files were regenerated from the live oracle and are byte-identical to the committed ones. |
| "Every figure … before this task landed was obtained under EXACT comparison" | True — the prior line was `if rust.stderr != cpp.stderr`. |
| "none did: the corpus was already 33 of 33 matching" | True, before and after. |
| the three named pinned witnesses "already existed … asserting an exact indent" | True; all three exist and assert an exact indent, as a quantity — see M1. |
| "a unit test's own `assert_eq!` is outside either harness's comparison function and normalisation cannot reach it" | True. |
| "NOT `the_indent_after_a_loop_has_already_exited_is_not_left_over_from_it`: it runs at top level, where the oracle's counter is already clamped at 0 and the correct and incorrect models agree" | **True, on both grounds.** `run.rs:6916`-`6925` runs `do i = 1 to 3 / say i / end / say 1/0`; the failing clause is at top level. Entering the loop takes the oracle's counter to 1; the loop exhausts its count, so `endLoop` (`BaseDoInstruction.cpp:377`) does a bare `unindent()` to 0 — which is also where the absolute-restore path would land. And `unindent()` clamps at 0 (`RexxActivation.hpp:318`), so even a doubled decrement reads 0. The two models cannot be told apart here. It is correctly excluded, and the row is right to say so explicitly. |
| "(precisely this row's own oracle-counter shape)" for `one_two_and_three_…` | **False — see I2.** |

---

## Reproduction

Oracle probes: `.../scratchpad/p1` … `p7` (each a fresh empty directory).
Adversarial battery: `.../scratchpad/adv` (36 tests, standalone crate, a copy
of the shipped `tests/support/mod.rs` as its `lib.rs`).
Mutations: `.../scratchpad/mut_*`, bodies in `.../scratchpad/bodies/`.
Full test log: `.../scratchpad/full.log`.
