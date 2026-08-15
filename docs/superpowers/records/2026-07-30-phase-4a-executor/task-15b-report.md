STATUS: DONE

# Task 15b report -- the `base/expressions` assertion harness (`rexx-exec/tests/assertions.rs`)

Scope: the consuming half of Task 15 only. Task 15a (already shipped, see
`task-15a-report.md`) built `rexx-extract`'s `extract_assertions`: one
`AssertionRow` per `self~assertSame` call in `ootest/ooRexx/base/expressions/`
(4,259 of 4,269, 10 dropped and accounted for), each carrying the expression
text, the expected-value text, the `NUMERIC DIGITS`/`FORM` in force, and (for
`CONCATENATION` only) the method's `a`..`g` assignment prelude. My job is
Steps 4-6 of `task-15-brief.md`: build the harness that runs every row through
`rexx_exec`'s public entry point, compares byte for byte, proves the table can
fail, and reports counts -- plus the commit.

Filling this in as I go, per the project's write-the-report-first discipline.
A usage-limit reset has killed six agents on this project mid-flight before;
this file is the guard against being the seventh.

## Plan

1. Read `task-15a-report.md` and `task-15-brief.md` (done, both steps
   numbered 5 noted -- the second is the commit step).
2. Establish what `rexx_exec`'s public API already supports that this row
   set needs: `NUMERIC DIGITS`/`FORM` instructions, plain assignment, `SAY`,
   the arithmetic/comparison/concatenation/logical operators, hex/binary
   string literals, grouping parens (confirmed to collapse to the inner
   `Expr` with no `List` node in `rexx-parse`, so `(a==a) (b==a)` is ordinary
   `Binary`/`Blank` chains, not the out-of-scope `ExprKind::List`).
3. Design the row-to-program translation: `NUMERIC DIGITS n` / `NUMERIC FORM
   f`, the prelude verbatim, then `SAY <expr>` and `SAY <expected>` as two
   separate clauses -- comparing the two rendered lines byte for byte is a
   direct proxy for `assertSame`'s own `expected == actual` (Rexx `==` is
   exact-string identity with no padding, so "the two SAY lines are
   byte-identical" and "`==` holds" are the same fact), and it needs no
   dependency on `==` itself being correct in `rexx-exec`, which is exactly
   the thing byte-for-byte comparison exists to check independently.
4. Classify each row's outcome: PASS (both lines byte-identical), FAIL (both
   lines rendered, differ -- a real divergence), or BLOCKED (the run hit
   `NOT_IMPLEMENTED_EXIT`, i.e. some `ExprKind` this row's text constructs is
   not in 4a's scope) -- with BLOCKED rows tagged by the construct named on
   stderr and, where the design spec's split table names an owner for that
   construct, the owning sub-phase.
5. REPORT vs STRICT modes, modelled on `tests/corpus.rs`'s own solution to
   "a passing `cargo test` line hides everything a test prints": the report
   crosses through an inherited-stderr child process, not `println!`.
6. The falsification proof: perturb one real row's `expected` text and show
   that comparison, and only that comparison, fails.
7. Run, measure, record every number below as it is taken. Commit last.

## Findings

**Harness built and running** (`rexx-exec/tests/assertions.rs`, plus a
test-only `rexx-extract` dev-dependency added to `rexx-exec/Cargo.toml`).
Row-to-program translation: `NUMERIC DIGITS n` / `NUMERIC FORM f`, the row's
prelude verbatim, then `SAY <expr>` and `SAY <expected>` as two separate
clauses, comparing the two output lines byte for byte -- a direct proxy for
`assertSame`'s own `expected == actual` (Rexx `==` is exact-string identity)
that does not depend on `rexx_exec`'s own `==` operator being correct.

**A second extractor defect found while running the harness for real, this
one a genuine modeling gap and not (yet) fixed -- STOPPING to ask before
touching `rexx-extract` further, per this task's own charter.**

`self~expectSyntax(code)` (and `self~assertSyntaxError`/`self~expectCondition`)
set up an expectation, checked by the ooTest framework's own per-method
condition trap, that *some subsequent statement in the same method* raises
that condition -- confirmed by reading `OOREXXUNIT.CLS`'s `expectSyntax`
(`self~conditionExpected = .true; self~conditionName = "SYNTAX"; ...`), and
independently by running the harness and watching it happen: `DIVISION.
testGroup`'s `test_262` is

```
self~expectSyntax(26.11)
Numeric Digits 5
self~assertSame("-5678932" % "-37", 1)
```

and running its extracted row (`say "-5678932" % "-37"` under `DIGITS 5`)
raises the oracle's own Error 26.11 ("Result of % operation did not result
in a whole number") -- **exactly the condition the row is supposed to
raise**, not a value it is supposed to equal 1. `expectSyntax` is one
statement earlier than `NUMERIC DIGITS`, not immediately before the
`assertSame`, so a same-line or previous-line check would have missed it;
the expectation spans the rest of the method, however many statements come
between.

Task 15a's own reasoning for treating `self~expectsyntax`/`self~assert*`
lines as inert -- "no variables assigned, so nothing to extract and nothing
that invalidates later state" (`lib.rs`'s `scan_method_for_assertions`
comment) -- is correct about *variable* state and silent about this: it
changes what a *later* `assertSame` in the same method means, from "expr
equals expected" to "evaluating expr raises this condition", which the row
schema (`expr`/`expected`/`digits`/`form`/`prelude`) cannot represent at
all. This is the same shape as the `CONCATENATION` prelude trap the brief
already named -- a line with no variable side effect that still changes
what a later assertion tests -- just a different mechanism.

**Measured extent**, scanning every method for an `assertSame` that occurs
anywhere after a `self~expectsyntax`/`self~assertsyntaxerror`/
`self~expectcondition` line in the same method body (a throwaway Python
probe mirroring `scan_method_for_assertions`'s own state machine, not
shipped):

| Group | rows affected |
|---|---|
| DIVISION | 60 |
| EXPONENT | 6 |
| REMAINDER | 118 |
| **Total** | **184** of 4,259 extracted rows (4.3%) |

`PRECEDENCE.testGroup` has 139 real `self~expectSyntax` markers -- **not**
`self~assertSyntaxError`, which never appears there at all; this was wrong
when first written here and is corrected, with the reason, in "Review
response" below -- and none of its rows are affected, confirmed by the same
probe (0 contribution): each marker is immediately followed by a bare
assignment that itself raises (`self~expectSyntax(42.3)` then `xre=0/0`),
alone in its own single-purpose method with no `assertSame` anywhere in the
body. Every one of
the 184 either never executes under the oracle at all (an earlier statement
in the method already raised and transferred control out) or *is* the
statement expected to raise; either way, comparing two `SAY` lines for it is
asking a question the row was never testing, and it surfaces in this harness
as a false `ANOMALY` ("a real condition escaped a row the oracle's own suite
asserts passes") that reads exactly like a `rexx-exec` defect but is not
one.

**Proposed fix, in `rexx-extract`, not attempted yet:** treat
`self~expectsyntax`/`self~assertsyntaxerror`/`self~expectcondition` the same
way `scan_method_for_assertions` already treats an unsupported statement
(a `DO` loop, the `SPECIAL::test_37` pseudo-function label) -- set
`blocked_reason` without counting the marker line itself as a drop, so every
`self~assertSame` from that point to the end of the method is accounted as
dropped rather than emitted as a row. That raises `Literals`/`MULTIPLICATION`/
`SPECIAL`'s existing 10 dropped to 194, and drops the row count from 4,259 to
4,075 -- still satisfying the same rows+dropped==calls invariant
`rexx-extract-assertions` already enforces, just with 184 more assertSame
calls correctly classified as unrepresentable rather than mis-represented.

**Why I am asking rather than making this change myself:** the task's own
file-scope rule is "`rexx-extract` if the extractor genuinely needs a
change, ask first and say why 15a did not need it" -- this is exactly that
situation, and I would rather have the change reviewed before it lands than
have Task 15a's already-reviewed, already-shipped extractor edited a second
time without a checkpoint.

## The team lead's ruling: convert, not block

Asked via `SendMessage` to `main`. The ruling, independently re-verified by
the team lead before answering (their own words: "`"-5678932" % "-37"` under
`numeric digits 5` raises Error 26.11 on the oracle, and
`self~expectSyntax(26.11)` carries the number inline in the source"): **the
fix is a conversion, not a block.** Blocking would discard the only 184 rows
in this whole table that exercise the *raise* path -- criterion 2 otherwise
quantifies entirely over expressions that produce a value -- and the
conversion is close to free, because both halves are already in hand: the
expected condition number is in `expectSyntax`'s own argument, and this
harness already detects that a condition escaped (that is what it was
reporting as `ANOMALY`).

Three conditions, and how each is met:

1. **Match the full `major.sub`, not just the major.** `RaiseExpectation`
   (`rexx-extract`) carries both fields; `classify_raise`
   (`rexx-exec/tests/assertions.rs`) compares the pair, and
   [`the_raise_falsification_proof`] perturbs *only* the sub (`26.11` ->
   `26.2`, a real, distinct catalogue entry -- a `DO` repetitor error, not a
   division shape at all) and confirms that alone fails.
2. **Scope the marker's reach, stated precisely.** The expectation spans to
   the end of the method (confirmed: `NUMERIC DIGITS` sits between the
   marker and the assertion in the real `test_262`, so a previous-line
   check would have missed it) and is carried **sequentially, exactly like
   `digits`/`form`**: a second `self~expectSyntax` in the same method
   replaces the expectation for whatever follows *it*, the same
   state-carrying rule `digits`/`form` already need. No method in
   `base/expressions` has two markers today (checked programmatically,
   0 hits) -- proven anyway, by a synthetic test
   (`a_second_expect_syntax_marker_replaces_the_first_for_what_follows_it`),
   because the mechanism is the load-bearing part, not today's corpus.
   `PRECEDENCE`'s 139 `self~assertSyntaxError` calls are unaffected and
   behave differently by construction: reading `OOREXXUNIT.CLS` shows
   `assertSyntaxError`/`assertRuntimeError` call `expectSyntax` *and then
   check the raise within that same call*, so nothing is left pending for a
   later statement -- confirmed both by the framework's own source and by
   the corpus (0 occurrences of `assertSame` following either in the same
   method). `self~expectCondition` (a *named*, non-numeric condition
   expectation) gets neither treatment: this row schema has nowhere to
   carry a bare name, so it **blocks** rather than converts or silently
   passing through -- a forward guard, since it does not occur near an
   `assertSame` in this corpus today either.
3. Conversion was not "more than it looks" -- no block-instead fallback was
   needed.

**Measured, before and after**, same invariant both times (rows + dropped
== independently-counted `self~assertSame` calls):

| | rows | dropped | of which raise-expectation |
|---|---|---|---|
| Before this fix (15a's shipped state) | 4,259 | 10 | 0 (mis-modeled as value rows) |
| After this fix | 4,259 | 10 | 184 (60 DIVISION, 6 EXPONENT, 118 REMAINDER) |

The row/dropped totals are unchanged, as they must be: converting a row's
*meaning* is not the same operation as dropping it, and pinning both counts
in the same test (`base_expressions_expect_syntax_conversion_counts`,
`rexx-extract/tests/extract_assertions.rs`) makes that explicit rather than
leaving two totals that could silently drift apart.

**The transferable lesson, as its own sentence, per the team lead's
request:** Task 15a's reasoning that `self~expectSyntax`/`self~assert*`
lines are inert because *they assign no variables and therefore cannot
invalidate later state* is true, and irrelevant -- the marker does not
change variable state, it changes what a **later assertion means**, and
being true about the thing it checked is exactly what made the reasoning
convincing while missing the thing that mattered. The same shape -- a
statement with no data side effect that still changes a later assertion's
semantics -- is what the `CONCATENATION` prelude trap already was, and is
worth watching for again wherever a harness models a test suite's own
mechanics rather than just its data.

**What shipped in `rexx-extract`** (`src/lib.rs`): a new `RaiseExpectation
{ major: u32, sub: u32 }` type; `AssertionRow::expect_raise: Option<
RaiseExpectation>`, carried through the scan exactly like `digits`/`form`;
`parse_raise_expectation`, parsing `self~expectSyntax(major.sub)` and
blocking (not guessing) on the array form `(major.sub, inserts...)`, which
none of the 19 real occurrences use; and the ordering fix so
`self~expectsyntax`/`self~expectcondition` are checked *before* the generic
"other assertion kind, inert" fallback that would otherwise still swallow
them. Six new tests in `rexx-extract/tests/extract_assertions.rs` (the
basic conversion, unaffected-before-the-marker, the sequential-second-marker
proof, the array-form block, `assertSyntaxError`'s non-leaking behaviour,
`expectCondition`'s forward guard) plus the two whole-corpus pins above.
All 18 of that file's tests pass; `cargo clippy -p rexx-extract --all-targets
-- -D warnings` and `rustfmt --edition 2024 --check` are both clean.

**What shipped in `rexx-exec/tests/assertions.rs`**: `program_for` omits the
second `SAY <expected>` entirely for a raise-expectation row, matching real
Rexx argument-evaluation order (the oracle never reaches `expected` either,
since evaluation of a message send's arguments stops at the first one that
raises); `classify` dispatches on `row.expect_raise` to `classify_value`
(the original two-line comparison) or `classify_raise`; and
`parse_condition_number` recovers `major.sub` from `Outcome::stderr`'s own
oracle-format report line, since `Raised` (the type that would carry those
fields directly) is `pub(crate)` inside `rexx-exec` and this is an
integration test outside the crate -- cross-checked against
`256 - exit_code` rather than trusted alone.

## Full harness run, after the fix

`cargo test -p rexx-exec --test assertions`, REPORT mode (default):
**4,224 of 4,259 rows passing.** Zero `MISMATCH`, zero `RAISE-MISMATCH`,
zero `ANOMALY`. The remaining 35 are all `RUNTIME-BLOCKED`, all in
`Literals.testGroup`, none of them a `rexx-exec` defect:

**Corrected below in "Review response" -- what follows is the pre-review
version and was wrong about which sub-phase unblocks 2 of the 35.** First
hit, not the same thing as what unblocks:

| first-hit construct | count | first hit as |
|---|---|---|
| `a message send` | 33 | Phase 5 |
| `a function call` | 2 | 4b |

All 33 come from `test_hexadecimal`/`test_binary`, comparing a literal
against `self~hex(...)`/`self~bin(...)` (message sends) or, in one case,
`.String~xdigit~x2c` (a chained message send on an environment symbol
beyond the three 4a admits). The 2 come from `test_string_range`, comparing
against `self~runDynamicSource(...)` (a function-position call) -- **but
that method's own prelude re-blocks on a message send on the very next
line after `xrange()`, so 4b landing `Call` would not make either of these
2 rows pass; see "Review response" for why all 35, not 33, are Phase 5's
alone.**

`REXX_ASSERTIONS_GATE=1 cargo test -p rexx-exec --test assertions
assertions_differential` (STRICT mode): **fails**, reporting exactly "35 of
4,259 assertion-table rows are not passing" -- proving the gate is live and
these 35 are the only thing between this table and a clean pass, not a
harness that always exits 0.

## The falsification proofs

Two, one per row shape, per Task 15's brief Step 5 ("perturb an expected
value and confirm that row fails"):

* **Value rows** (`the_falsification_proof`): the first row overall (an
  `ADDITION` row) is confirmed to pass unperturbed, then its `expected` is
  wrapped `({expected}) || 'ZZZ-FALSIFICATION-MARKER'` (parens preserve the
  original's own precedence -- confirmed in `rexx-parse` that a
  parenthesised expression collapses to the inner node with no wrapper, so
  wrapping changes nothing about how the original text evaluates) and the
  same row now reports `Mismatch`. A second, unperturbed row is checked
  afterwards to confirm nothing leaked across the two runs.
* **Raise-expectation rows** (`the_raise_falsification_proof`): `DIVISION.
  testGroup`'s `test_262` (the same witness that found the `expectSyntax`
  gap) passes honestly at `26.11`. Perturbed to an impossible expectation
  (`1.1`, which its own `%` expression could never raise) it reports
  `RaiseMismatch { actual: Some((26, 11)), .. }`. Perturbed to `26.2` --
  same major, different sub, a real and distinct catalogue entry (a `DO`
  repetitor error, not a division shape) -- it *still* fails the same way,
  which is the sharper proof Task 15's item 3 asked for by name: a harness
  that only checked the major would have waved `26.2` through.

Also proven directly (`digits_and_form_are_carried_not_defaulted`): the
same `ADDITION.testGroup` `test_198` row (`DIGITS 5`, `FORM ENGINEERING`)
passes at its real settings and reports `Mismatch` when evaluated at the
*default* settings (`DIGITS 9`, `FORM SCIENTIFIC`) instead -- the harness is
sensitive to the per-row settings it carries, not merely trusting that Task
15a computed them correctly.

## Row count and what remains blocked (Task 15's Step 6)

**4,224 of 4,259 extracted rows pass.** All 35 not passing are unblocked
only by Phase 5 (corrected in "Review response" below -- the pre-review
version of this report split them 33/2 by first-hit construct rather than
by what would actually make them pass). The 10 rows Task 15a
already reported as extraction-blocked (`Literals` 6, `MULTIPLICATION` 2,
`SPECIAL` 2, all `DO`-loop or pseudo-function shapes the row format cannot
represent at all, independent of any sub-phase) are unchanged and are not
re-litigated here; see `task-15a-report.md` for that list. Nothing in this
table is silently dropped: every one of the 4,269 original `self~assertSame`
calls is accounted for as a passing row, a `RUNTIME-BLOCKED` row naming its
sub-phase, or an extraction-blocked drop naming its reason.

## Verification

```
cd rust
cargo test -p rexx-extract && cargo test -p rexx-exec --test assertions
cargo clippy -p rexx-extract -p rexx-exec --all-targets -- -D warnings
rustfmt --edition 2024 --check \
  crates/rexx-extract/src/lib.rs \
  crates/rexx-extract/tests/extract_assertions.rs \
  crates/rexx-exec/tests/assertions.rs
REXX_ASSERTIONS_GATE=1 cargo test -p rexx-exec --test assertions assertions_differential
```

**Superseded by "Review response" below**: after F7's ruling, STRICT is
expected to (and does) **pass** -- all 35 are committed to `EXEMPT` and
correctly attributed, so there is no longer a known gap between STRICT and
green. The verification block in "Review response" is the current one.

## Committed

`7eeb309d`, "Add the base/expressions assertion-table harness (Task 15,
consuming half)" -- exactly the five permitted files (`rust/Cargo.lock`,
`rust/crates/rexx-exec/Cargo.toml`, `rust/crates/rexx-exec/tests/
assertions.rs`, `rust/crates/rexx-extract/src/lib.rs`, `rust/crates/
rexx-extract/tests/extract_assertions.rs`). Nothing from the concurrent
TRACE work (`src/eval.rs`, `src/lib.rs`, `src/run.rs`, `src/trace.rs`,
`tests/trace_oracle/`) was staged or touched -- confirmed by `git status`
immediately before and after the commit.

A second commit follows this section, addressing the team lead's review
(F1, F2, F3, F5 evidence corrections; F6 and F7 rulings). See "Review
response" above for what changed and why; its own verification block is
the current one, superseding the "Verification" section above it.

## Review response, after `7eeb309d`

The reviewer's own perturbation (forcing `test_262`'s carried `DIGITS`
from 5 to 9, which stops it raising entirely rather than merely rendering
differently) is now in the tree as `digits_and_form_are_carried_not_
defaulted`'s counterpart on the raise side -- worth recording since it is,
in the team lead's words, "the strongest single piece of evidence in this
task", and it came from review rather than from me.

### F7 (ruling: exempt, and police the attribution like criterion 5)

Implemented as asked, not as a dynamically-recomputed set: `ExemptRow`/
`EXEMPT` in `rexx-exec/tests/assertions.rs` is a **committed array of the
35 rows**, identified by `group`, `method`, a 1-based `occurrence` within
that `group`+`method` (needed because `test_string_range`'s two rows are
byte-identical in `expr`/`expected` -- only their prelude differs, and
`AssertionRow` does not carry the prelude into this identity check), plus
`expr`/`expected` themselves, checked together. Two mechanisms:

* `the_exempt_set_matches_the_current_blocked_rows` -- runs in every mode,
  unconditionally, and asserts the current not-passing set equals `EXEMPT`
  exactly. This is "assert that set."
* `assertions_differential`'s STRICT mode uses `EXEMPT`, not a fresh
  recomputation, to decide what it may forgive: a row **not** on the list
  that is not passing is a gate failure; a row **on** the list that *is*
  passing is also a gate failure (a stale exemption -- the fix is editing
  `EXEMPT`, which is a diff, not a property the harness silently updates
  itself).

**STRICT now passes**: `REXX_ASSERTIONS_GATE=1 cargo test -p rexx-exec
--test assertions assertions_differential` -- green, 0 EXEMPT-set
violations, confirmed by direct run.

**The mutation this device actually kills, measured rather than
predicted.** Commented out `program_for`'s prelude-writing loop (so
`test_hexadecimal`/`test_binary`'s `tab = .String~tab` line,
`test_string_range`'s `all = xrange()` chain, and -- this is what made the
number far bigger than my first guess -- `CONCATENATION`'s own `a`..`g`
prelude all stop running) and re-ran both new tests:

* `the_exempt_set_matches_the_current_blocked_rows` failed immediately:
  `left: 336, right: 35` (the not-passing count against `EXEMPT`'s
  length).
* `assertions_differential`'s own "EXEMPT-set violations" section named
  345 -- 22 of the 35 committed rows (every pure-literal comparison in
  `test_hexadecimal`/`test_binary` with no `self~` in its own `expr`/
  `expected` text, e.g. `"AB"` vs `"41 42"x`) reported "... now PASSES but
  is still listed in EXEMPT" (the stale-exemption direction), and 323
  previously-passing rows outside the committed list -- almost all
  `CONCATENATION`'s, its unset `a`..`g` rendering as their own names --
  reported "is not passing and is not on the committed EXEMPT list" (the
  unattributed-regression direction). Both directions this device exists
  to catch fired from one mutation. Reverted before running the rest of
  the suite; `cargo test -p rexx-exec --test assertions` back to 5/5 green
  immediately after.

### F6 (ruling: state what actually unblocks a row -- Phase 5, all 35)

Re-read `test_string_range`'s own source: `all = xrange()` (a function
call, first-blocked as 4b's) is immediately followed by `all =
all~changeStr(.String~cr, "")` -- a message send -- in the same prelude.
Implementing 4b's `Call` would move this row's *first* blocker one line
later and no further; it would still fail on the very next line, on
Phase 5. Confirmed the same holds for the row's second occurrence (no
extra `changeStr` lines, but its own `expected` argument is `self~
runDynamicSource(...)`, itself a message send). So every `EXEMPT` entry's
`unblocked_by` is `"Phase 5"`, including these two -- the report line
changed from `(4b)`/`(Phase 5)` per-construct to a single `first hit X (not
implemented); unblocked only by Phase 5` phrasing that keeps the
first-hit fact (real, observed, worth keeping) visibly separate from the
unblocking fact (committed, not recomputed). `owning_subphase` (the
function that conflated the two) is deleted.

### Evidence corrections

* **F1**: fixed. `PRECEDENCE.testGroup` has 139 real `self~expectSyntax`
  markers (not `self~assertSyntaxError`, which never appears there --
  `grep -ci` confirms 0) and 0 conversions, but not because they are
  wrapping calls: each is immediately followed by a bare assignment that
  itself raises (`self~expectSyntax(42.3)` then `xre=0/0`), alone in its
  own single-purpose method with no `self~assertSame` anywhere in the
  body. `self~assertSyntaxError` is `Literals`'s own name, 33 occurrences,
  all wrapping calls (`self~assertSyntaxError((15.1, 1), self~hex(" "))`).
  Corrected in both `rexx-extract/tests/extract_assertions.rs`'s doc
  comment and here.
* **F2**: fixed. `OOREXXUNIT.CLS:1203`/`:1213` show `assertSyntaxError`/
  `assertRuntimeError` call `self~expectSyntax` and then attempt the risky
  statement with **no local check** -- a raise escapes to the same
  per-method trap a bare `expectSyntax` relies on. The doc comment now
  says so, and (per the team lead's "consider") the scanner now **blocks**
  on these two names rather than trusting the generic "other assertion
  kind, inert" fallback to be safe here by luck -- the same forward guard
  `self~expectCondition` already had. Zero behaviour change: confirmed by
  re-running `base_expressions_yields_the_measured_row_and_blocked_counts`
  (still 4,259/10) after the change, since 0 `self~assertSame` calls ever
  follow either name in this corpus. The old test
  (`assert_syntax_error_does_not_leave_a_pending_raise_expectation`,
  asserting a passing row) is replaced by
  `assert_syntax_error_blocks_rather_than_silently_passing_through`
  (asserting a block) plus a new `a_row_before_assert_syntax_error_is_
  unaffected`.
* **F3**: fixed. `parse_raise_expectation`'s doc comment said "19", now
  says "184" in both `rexx-extract/src/lib.rs` and the corresponding
  `expect_syntax_with_message_inserts_blocks_rather_than_guesses` doc in
  `tests/extract_assertions.rs`.
* **F5**: fixed. `collect_all`'s doc comment claimed a `count_assert_same`
  recount that was never implemented; corrected to say plainly that both
  pins (`rows.len()`/`dropped`, and the assertSame-count invariant) live
  only on the extractor's own side, and this crate does not duplicate
  either.
* **F8**: the team lead's own, not mine -- the plan's 56/332
  `CONCATENATION` split is being corrected to 106/282 there. Restated here
  only because the mutation-kill evidence above independently confirms
  the reviewer's underlying point regardless of the exact split: with the
  prelude gone, hundreds of `CONCATENATION` rows fail loudly rather than
  passing silently.

### Re-verification after all of the above

```
cd rust
cargo test -p rexx-extract                                    # 19+3 passed
cargo test -p rexx-exec --test assertions                      # 5 passed
cargo clippy -p rexx-extract -p rexx-exec --all-targets -- -D warnings   # clean
cargo clippy --workspace --all-targets -- -D warnings          # clean
rustfmt --edition 2024 --check \
  crates/rexx-extract/src/lib.rs \
  crates/rexx-extract/tests/extract_assertions.rs \
  crates/rexx-exec/tests/assertions.rs                         # clean
REXX_ASSERTIONS_GATE=1 cargo test -p rexx-exec --test assertions assertions_differential   # now PASSES
```

Counts unchanged throughout, as the team lead asked to confirm: 4,259
rows, 10 extraction-blocked, 184 raise-conversions, 4,224 of 4,259 passing,
35 runtime-blocked (all `Literals`, all now correctly attributed to
Phase 5 alone, all committed to `EXEMPT`).

## Second commit

`8aa18b55`, "Address review: exempt-and-police the Phase 5 gap, fix three
mischaracterizations" -- exactly the three files under review
(`rust/crates/rexx-exec/tests/assertions.rs`, `rust/crates/rexx-extract/
src/lib.rs`, `rust/crates/rexx-extract/tests/extract_assertions.rs`), no
`Cargo.lock` change this round (no new dependency). Confirmed unstaged and
untouched at commit time: `docs/superpowers/plans/phase-4-exclusions.txt`
(the team lead's own F8 fix) and `rust/crates/rexx-exec/src/eval.rs`/
`src/run.rs` (the concurrent TRACE work, including an unrelated fix its own
commit also labelled "F1" -- a different review round's finding about a
`DO n` trace tag, not this task's `PRECEDENCE` mischaracterization).
