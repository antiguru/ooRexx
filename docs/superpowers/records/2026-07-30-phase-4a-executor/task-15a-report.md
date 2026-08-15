# Task 15a report -- the `rexx-extract` assertion-row extractor

Scope, per the team lead's dispatch: the extractor half of Task 15 only.
Build a `rexx-extract` mode that emits one row per `base/expressions`
assertion -- expression text, expected value, `NUMERIC DIGITS` in force, and
(where needed) an assignment prelude -- rather than continuing to render
whole test methods as standalone `.rex` programs. The falsification check,
the byte-for-byte comparison, and `rexx-exec/tests/assertions.rs` are the
harness half and are explicitly not mine: 4a has no evaluator yet, and the
file scope for this task is `rust/crates/rexx-extract/` only (not
`rexx-parse`, not `rexx-exec`, not `rust/corpus/`, all in flight from other
agents in this worktree).

Filling this in as I go; this is the skeleton written before touching code,
per the project's write-the-report-first discipline.

**Status: complete, verified, uncommitted pending this report.** All three
things the brief called out as the hard part are done and each has its own
test: sequential `NUMERIC DIGITS`/`FORM` carry-through (not read in
isolation), the `CONCATENATION` assignment prelude attached to every row
that needs it (not silently dropped, which would pass while testing
nothing), and per-group produced/dropped counts reported and pinned as a
test rather than left as a percentage. Nothing in this task's scope was
skipped. What is explicitly *not* done, because it was never this task's
scope: the byte-for-byte comparison, the falsification check ("perturb an
expected value and confirm the row fails"), and
`rexx-exec/tests/assertions.rs` -- all need a real evaluator and belong to
the harness half, per the team lead's dispatch. A usage-limit reset
happened mid-session; the team lead's follow-up reported this report file
as absent and the work as uncommitted at that point. By the time this
session resumed and checked, this file was present in the working tree
(written progressively while the code was being built, per the
write-the-report-first discipline) but still uncommitted, and none of the
code had been staged or committed either -- consistent with "uncommitted"
even if the "no report" half of that description no longer matched what
was on disk. Re-verified everything below from scratch (tests, clippy,
fmt, and the real-corpus run) rather than trusting anything from before
the reset, and did not touch the code itself except to re-run it.

## Plan

1. Read the `Task 15` section of the plan and the file structure it names,
   and independently re-derive *why* the current rendering executes
   nothing, since the brief states the conclusion ("main body is empty")
   without walking through the directive-ordering reasoning -- confirm
   under the oracle rather than trust the phrasing.
2. Read every `.testGroup` file under `ootest/ooRexx/base/expressions/`
   (11 files, 4,269 `assertSame` calls per the brief) to learn the actual
   source shape: how `NUMERIC DIGITS` appears, how an assertion's expected
   value and expression text are spelled, and which groups need a prelude
   (variables assigned before the assertion) versus which are self-contained
   literal arithmetic.
3. Design the row format and the sequential-scan-carrying-DIGITS algorithm,
   and design the prelude representation for CONCATENATION specifically.
4. Implement the extraction mode in `rust/crates/rexx-extract/src/lib.rs`
   and wire it into `src/bin/rexx-extract.rs`.
5. Run it over all of `base/expressions`, report counts produced and
   dropped per group (the brief's explicit ask: a shortfall against the
   4,269/1,226/388 figures is the finding, not a rounding error).
6. Test: at minimum, a file that changes `NUMERIC DIGITS` mid-way, to prove
   the setting is carried sequentially rather than pattern-matched in
   isolation (this is the brief's own named risk: getting it wrong silently
   produces rows that test the wrong precision and still pass).
7. Write everything up here, stage only `rust/crates/rexx-extract/` paths,
   commit once.

## Findings from reading `ootest/ooRexx/base/expressions/*.testGroup` (11 files, 13,731 lines)

* **Why the current rendering executes nothing, confirmed independently.**
  `render()` wraps a method body in `::routine main public`. A `::ROUTINE`
  directive is not auto-invoked by loading the file -- and since the file's
  *first* line is that directive (nothing precedes it), the package's
  prolog is empty. Running it directly runs zero statements; nothing calls
  `main`. This matches "prints nothing, exits 0" without needing `self` to
  be broken inside a routine (it would be, too, but that's moot -- the body
  never runs at all).
* **`self~assertSame` count matches the brief exactly**: 4,269 total,
  1,226 in PRECEDENCE, 388 in CONCATENATION. Verified with
  `grep -c self~assertSame`, group by group.
* **Every single `self~assertSame(...)` call, all 4,269, is syntactically
  uniform**: starts the line (after trim), balanced parens, exactly one
  top-level comma, nothing trailing after the close-paren, never spans more
  than one physical line. Verified computationally (a throwaway Python
  probe mirroring the planned Rust scan, not shipped). So the *parsing* of
  an individual call is not where the risk is; the risk is entirely in
  carrying `NUMERIC` state and prelude assignments correctly across lines.
* **Only `CONCATENATION` mixes assignments with `assertSame` in the same
  method** -- checked all 11 files programmatically for "method has both a
  bare assignment line and an `assertSame` call"; `CONCATENATION` is the
  only hit (3 methods, `test_1`/`test_2`/`test_3`, each a 7-line `a`..`g`
  prelude, `test_3` adds an 8th variable `nb`). This confirms the brief's
  framing that CONCATENATION is the one exception, and that a general
  "accumulate assignment lines seen so far in this method as prelude"
  design doesn't need a `CONCATENATION`-specific branch.
* **A pre-flight finding, not in the brief: `NUMERIC FORM` is exactly as
  load-bearing as `NUMERIC DIGITS`, in 5 of the 11 groups.**
  `ADDITION` (197 occurrences), `SUBTRACTION` (299), `MULTIPLICATION` (71),
  `DIVISION` (135), `REMAINDER` (144) all set `Numeric Form ENGINEERING`
  before some assertions and `Numeric Form SCIENTIFIC` before others, in
  the same shape as the `DIGITS` example the brief gives (e.g. `ADDITION`
  `test_198`: `Numeric Form ENGINEERING` + `Numeric Digits 5` then
  `self~assertSame(9999999999999 + 9999999999999, 20.000E+12)` --
  `20.000E+12` is only the right answer in engineering notation). A row
  carrying `DIGITS` alone and defaulting `FORM` to `SCIENTIFIC` would not
  silently pass here the way an uncaught `CONCATENATION` prelude would --
  it would just fail outright, since scientific and engineering notation
  render differently -- but it would fail *for the wrong reason*, reading
  as an evaluator defect (`rexx-exec`'s `FORM` handling) rather than what
  it actually is (a missing extractor field), across roughly 850 rows.
  **Decision: the row schema carries `form` alongside `digits`,
  defaulting to `Scientific`, tracked with exactly the same sequential
  carry-and-reset-per-method rule.** `NUMERIC FUZZ` does not need the same
  treatment: it never appears in `base/expressions` (checked, 0
  occurrences) and could not matter even if it did, since it only widens
  `=`/`<`/`>`'s tolerance and `assertSame` compares with `==`, which FUZZ
  never affects.
* **All `NUMERIC DIGITS` values found are plain literal integers, 1 to
  100** (exact values seen: 1, 2, 3, 4, 5, 6, 9, 11, 15, 16, 18, 20, 30, 50,
  100), matching the brief's "from 1 to 100" precisely. All `NUMERIC FORM`
  values are exactly the literal words `SCIENTIFIC` or `ENGINEERING` (never
  `NUMERIC FORM VALUE expr`). `NUMERIC` settings are per-method (each
  `::method` is a fresh activation with default `DIGITS 9`/`FORM
  SCIENTIFIC`/`FUZZ 0`) -- confirmed no package-level `::OPTIONS` sets a
  different default anywhere in these 11 files; the three `::OPTIONS` lines
  that exist (`novalue error`, `all syntax`) are all the file's last
  directive and control unrelated behaviour.
* **Everything else that isn't a `NUMERIC` setting, a bare assignment, or
  an assertion call, catalogued across all 11 files** (I scanned every line
  of every method that contains at least one `assertSame`, for anything
  that isn't one of those three shapes): a handful of `DO`/`END` loops (5
  methods in `Literals`, 1 in `MULTIPLICATION`), one method in `SPECIAL`
  (`test_37`) that defines and calls a local label as a pseudo-function
  (`f1: Arg a; ...; Return a`, called as `f1(3)`) with a semicolon-joined
  multi-clause prelude line, and bare trailing `return` statements in a few
  methods that are always the last line (harmless, no assertion follows
  them). Predicted fallout, worked out by hand before writing the Rust
  scanner so there is an independent number to check it against: **10
  `assertSame` calls blocked** (`Literals` 6 across 5 methods,
  `MULTIPLICATION` 2 in one method, `SPECIAL` 2 in `test_37`), so **4,259
  of 4,269 rows extractable**. `self~assertTrue`/`assertEquals`/
  `expectSyntax`/etc. calls (other assertion kinds, not `assertSame`) are
  treated as inert: they don't assign variables, so they don't block or
  change state, they just don't produce a row.
* **Design choice on what a row's `expr`/`expected` fields hold:** raw
  source text of `assertSame`'s two positional arguments, verbatim,
  unparsed -- not a pre-stripped string literal. Both are ordinary Rexx
  expressions (method-call arguments are always expressions; a quoted
  literal and a bare signed number are both just expressions that happen to
  be constant), and evaluating "the expected side" is exactly the kind of
  interpretation that needs a real evaluator, which is the harness's job
  and needs `rexx-exec`, not mine. Handing over raw text for both sides
  keeps the extractor purely mechanical and leaves comparison semantics
  (byte-for-byte, per the brief's Step 4) entirely to the harness.

## A real bug found by my own invariant check, in the pre-existing shared `extract()`

Every extraction mode's own internal consistency check (`rows.len() + sum(blocked.dropped) == independently-counted assertSame calls`, run as an assertion inside `rexx-extract-assertions`'s own binary before it prints each group's row) panicked on the first real-corpus run:

```
MULTIPLICATION.testGroup: 186 rows + 2 dropped != 1050 assertSame calls
```

864 calls unaccounted for. Traced it to `extract()` (the pre-existing, already-tested function this whole crate is built on, not something this task wrote): its `::method` name parsing only stripped double quotes (`.trim_matches('"')`), but `MULTIPLICATION.testGroup` names eight of its methods with single quotes instead (`::method 'test_15_bit'`, `'test_16_bit'`, `'test_9_digit'`, `'test_31_bit'`, `'test_32_bit'`, `'test_18_digit'`, `'test_63_bit'`, `'test_64_bit'`, each with 108 `assertSame` calls -- 8 x 108 = 864, exactly the shortfall). The un-stripped leading `'` made `name.to_ascii_lowercase().starts_with("test")` false, so `push_if_test` silently dropped the entire method. `PRECEDENCE.testGroup`'s `'test_='` is single-quoted too, though it uses `assertTrue` rather than `assertSame` so it did not show up in this particular count.

This is a real, pre-existing defect in code this task did not write, reachable from the *old* program-rendering mode too (confirmed: re-ran the existing `rexx-extract` binary over `base/expressions` before and after the fix -- `MULTIPLICATION.testGroup` went from "151 total, 143 total methods found" to "151 total, 151 found" -- see numbers below). Fixed with a one-line change, `.trim_matches('"')` to `.trim_matches(['"', '\''])`, plus a comment recording the measurement and a regression test (`a_single_quoted_method_name_is_still_recognised_as_a_test` in `tests/extract.rs`).

**Known side effect, out of my file scope to fix:** `docs/superpowers/plans/l1-coverage.md` is a previously-committed report from the old `rexx-extract` binary and still shows the pre-fix, undercounted number (`MULTIPLICATION.testGroup | 143 | 143 | 100.0%`, line 313). That file is under `docs/`, outside this task's file scope (`rust/crates/rexx-extract/` only), so it is now stale relative to the tool that produced it and I have not touched it -- flagging this to the team lead rather than fixing it myself.

## Final measured counts, `ootest/ooRexx/base/expressions/` (via `rexx-extract-assertions --suite`)

| Group | assertSame calls | Rows | Dropped |
|---|---|---|---|
| ADDITION | 304 | 304 | 0 |
| COMPOSITE | 28 | 28 | 0 |
| CONCATENATION | 388 | 388 | 0 |
| DIVISION | 373 | 373 | 0 |
| EXPONENT | 123 | 123 | 0 |
| Literals | 45 | 39 | 6 |
| MULTIPLICATION | 1050 | 1048 | 2 |
| PRECEDENCE | 1226 | 1226 | 0 |
| REMAINDER | 297 | 297 | 0 |
| SPECIAL | 29 | 27 | 2 |
| SUBTRACTION | 406 | 406 | 0 |
| **Total** | **4269** | **4259** | **10** |

Blocked methods, all confirmed by hand before writing the scanner (see "Findings" above), so there is an independent number this matches:

* `Literals::test_hexadecimal_single` -- `do hex over "..."~makeArray("")` (2 dropped)
* `Literals::test_hexadecimal_double` -- `do first over "..."~makeArray("")` (1 dropped)
* `Literals::test_binary_single` -- `do bin = 0 to 1` (1 dropped)
* `Literals::test_binary_nibble` -- `do first = 0 to 1` (1 dropped)
* `Literals::test_binary_ones` -- `do power = 1 to 64` (1 dropped)
* `MULTIPLICATION::test_bug1339` -- `do m = 2 to 3` (2 dropped)
* `SPECIAL::test_37` -- `h=''; lta=lt; c.lta=f1(3)+f1(4);` (2 dropped; a local label called as a pseudo-function)

Every group matches the number the invariant demands exactly (rows + dropped == independently-counted `self~assertSame` occurrences); this is enforced as a permanent test
(`every_assert_same_in_base_expressions_is_a_row_or_an_accounted_for_drop`) so a future regression in either `extract()` or `extract_assertions` shows up immediately rather than as a silent undercount. A second test
(`base_expressions_yields_the_measured_row_and_blocked_counts`) pins these exact totals, matching the brief's specific ask to report 1,226 for `PRECEDENCE` and 388 for `CONCATENATION` and to treat any shortfall as the finding rather than a rounding error.

## What ships

* `rust/crates/rexx-extract/src/lib.rs`: the one-line `extract()` fix (single-quote stripping) with its comment; `Form`, `AssertionRow`, `BlockedMethod`, `AssertionExtraction`, `extract_assertions`, and the private scanning/parsing helpers; `find_test_groups` (a directory walk, factored out so both binaries can share it without disturbing the existing `rexx-extract.rs` binary's own `walk()`).
* `rust/crates/rexx-extract/src/bin/rexx-extract-assertions.rs`: a reporting CLI (`--suite <dir>`) that runs the new mode over a suite and prints the per-group table above, the blocked list, and enforces the rows+dropped==calls invariant as a hard assertion. It does not write a data file: the row set is meant to be produced by calling `extract_assertions` directly against the already-checked-in `.testGroup` sources at the point of use (in the harness half, later), not by serialising a pre-baked artifact this task would have to design a format for and that `rexx-exec`/`rust/corpus` (both out of this task's file scope) would have to consume.
* `rust/crates/rexx-extract/tests/extract.rs`: one new regression test for the single-quote fix, alongside the two pre-existing tests (still passing, unchanged).
* `rust/crates/rexx-extract/tests/extract_assertions.rs`: 11 tests -- 9 synthetic (digits/form sequential carry, prelude attachment and its absence, partial blocking, the harmless-trailing-`return` non-report, other-assertion-kind inertness, non-`test`-method exclusion, group-label passthrough) plus the two whole-corpus tests above.

## Verification run

`cd rust && cargo test -p rexx-extract && cargo clippy -p rexx-extract --all-targets -- -D warnings && cargo fmt --check -p rexx-extract`: 14 tests (3 + 11), 0 failed; clippy clean; fmt clean. Confirmed the existing `rexx-extract` binary (the old program-rendering mode, untouched apart from benefiting from the shared `extract()` fix) still runs cleanly over the same suite: `11 groups, 2888 test methods, 2847 extractable (98.6%)`.

## Pre-flight deviation from the brief, flagged rather than silently applied

The brief's Step 1 names only `NUMERIC DIGITS` as state to carry. Reading all 11 files first (before writing the scanner) found `NUMERIC FORM` is exactly as load-bearing in 5 of them (`ADDITION` 197, `SUBTRACTION` 299, `MULTIPLICATION` 71, `DIVISION` 135, `REMAINDER` 144 occurrences) -- e.g. `ADDITION.testGroup`'s `test_198` sets `Numeric Form ENGINEERING` before an assertion whose expected value, `20.000E+12`, is only correct in that notation. A row that dropped `FORM` and defaulted to `SCIENTIFIC` would not silently pass (scientific and engineering notation render visibly differently, so it isn't the same silent-pass trap `CONCATENATION`'s prelude is), but it would fail for the *wrong* reason across roughly 850 rows -- reading as a `rexx-exec` `FORM`-handling defect when the real problem would be a missing extractor field. Decision: `form` ships alongside `digits` in every row and `Form`'s test, tracked with the identical sequential carry-and-reset-per-method rule, with its own dedicated test (`form_changing_mid_method_is_carried_sequentially`). `NUMERIC FUZZ` gets no such treatment: confirmed 0 occurrences in `base/expressions`, and it could not matter even if present, since it only widens `=`/`<`/`>`'s tolerance and `assertSame` compares with `==`.


