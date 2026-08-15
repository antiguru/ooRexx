# Task 3.9 fix-round re-review: `review-6614ac34..ab07e536.diff`

Reviewed: commit `ab07e536` only (the fix round). The original implementation
(`6614ac34`) is not re-reviewed here.

## Method

Independent verification, not a re-read of the prior review or report.
Built a scratch harness (`probe-harness` in the session scratchpad, path
dependency on `rexx-parse`) to call `join_span`/`span_bytes`/`line_span`
directly.
Ran an exhaustive contract checker over every `(start, end)` span for twenty
synthetic sources chosen to cover every case category in the brief (single
terminator byte, mid-CRLF-ending span, LF-CR two-terminator empty line,
source of only terminators, trailing terminator with no line after it,
Ctrl-Z truncation, zero-line source, `Interpret` source, spans starting on a
terminator byte).
Ran the required defeat experiment (`join_span`'s body replaced with
`self.span_bytes(span).map(Cow::Borrowed)`) against the live tree, then
reverted with `git checkout -- rust/crates/rexx-parse/src/source.rs`.
Re-measured probe G against a fresh oracle capture of my own to check the two
comment claims the brief flagged as unchecked.
`git status --short` is clean; `cargo test --offline --workspace
--no-fail-fast` and `cargo clippy --offline --all-targets -- -D warnings` are
clean at `ab07e536` after the revert.

## Item 1: coverage (four probes)

Addressed.
Did not re-measure H, I, J against the oracle myself (the coordinator
already did, byte for byte); did re-measure G myself (see below) and ran the
defeat experiment against all four.

**New finding (Important):** probe J (`a: b: nop`, multi-label) provides no
regression protection for `join_span`'s terminator-stripping. Defeating
`join_span` to a raw `span_bytes` fallback and running the suite turns G, H,
and I red but leaves J green, because none of J's three clause spans (`a:`,
`b:`, `nop`) contain a continuation or a terminator byte: the raw span and
the joined span are byte-identical for all three regardless of whether the
join logic exists at all. J is doing its stated job (proving multi-label
spans are extracted correctly) and that job needed no oracle-measured
multi-line evidence, but it does not exercise the join the round's Item 1
was dispatched to cover evidence for. This is not a defect in the shipped
diff -- a multi-label clause on one line has nothing to join -- but it means
the round's actual "does a defeated join get caught" coverage is 3 of 4 new
tests, not 4 of 4, and that fact appears nowhere in the report or the
review.

**Full defeat-experiment output** (`cargo test -p rexx-parse --test
sourceline` with `join_span` defeated):
```
failures:
    a_continued_clause_joins_without_its_terminator
    a_continued_clause_span_contains_the_terminator_it_joins_out
    a_crlf_continuation_drops_both_terminator_bytes
    join_span_borrows_on_one_line_and_agrees_with_span_bytes_about_none
    probe_g_an_else_arm_continues_like_any_other_clause
    probe_h_an_otherwise_arm_continues_like_any_other_clause
    probe_i_a_three_fragment_continuation_drops_every_terminator

test result: FAILED. 18 passed; 7 failed; 0 ignored; 0 measured; 0 filtered out
```
7 of 25 total tests go red: the same 4 pre-existing ones the prior review
found, plus G, H, and I. J (`probe_j_a_multi_label_clause_is_one_clause_per_label`)
stays green under the defeat, as does `trace_output_rex_reconstructs_every_traced_line`,
`probe_a_...`, `probe_b_...`, and `interpret_fragment_clauses_reconstruct_too`
(none of those contain a continuation either, consistent with the earlier
review).

**Probe G's two unverified comment claims, checked against a fresh oracle
capture** (`trace r` / `if 1 = 2 then nop` / `else say 1,` / `    2` /
`trace off`, run as `( ulimit -v 1048576; build/bin/rexx FILE )`):
```
     2 *-* if 1 = 2 $
       >>>   "0"$
     3 *-*   else$
     3 *-*     say 1,    2$
       >>>       "1 2"$
1 2$
     5 *-* trace off$
```
Both true: there is no `*-*` line for `nop` anywhere in the transcript (the
condition is false, the THEN arm never runs and is never traced), and the
reconstructed clause text for the `*-*     else` line is exactly `else`
(the leading three spaces in the raw line are TRACE's own indentation, not
part of the clause), with nothing before or after it -- no blank on either
side, as the comment claims.

## Item 2: the empty-list floor

Addressed, and fires as documented. `assert_traced`'s first statement is
`assert!(!traced.is_empty(), "an empty expectation list checks nothing")`,
unconditional, no branch or cfg guards it, so a call with `&[]` panics before
touching the parsed program. Confirmed by direct reading; this is a
one-line, unconditional macro call with no ambiguity worth a live re-run.

Confirmed, as the brief anticipated, that a caller CAN trivially satisfy the
floor while still omitting lines: a one-entry list passes the `is_empty`
check and the per-line loop just checks fewer things. This is exactly the
scoped limitation the brief and the doc comment both call out ("a floor, not
a mechanism," completeness stays the caller's obligation) -- not a new
defect, just confirming the floor does no more and no less than advertised.

## Items 3 and 4: comment fixes

Addressed, both.

* Structuring semicolons: grepped both touched comment blocks
  (`src/source.rs:188-204`, the `join_span` doc; `tests/sourceline.rs:239-248`,
  the `trace_output_rex_...` test). Zero structuring semicolons remain in
  either. The one semicolon still present in the `join_span` doc
  (`` a terminating `;`. ``) is a backtick-quoted literal Rexx semicolon
  character, not prose structure, and was never the flagged instance.
* Marker-list disclaimer: the comment now reads "The markers THIS file
  happens to emit are >L>, >V>, >O>, >>> and >=>. That is not the set: the
  authority is 'everything except *-*', eighteen prefixes...". Against the
  given authority (nineteen prefixes total, `*-*` plus eighteen others), the
  arithmetic is right: eighteen, not nineteen.

## Item 5: the `join_span` contract

Addressed, and exact in both directions as far as this review can construct
counterexamples for.

**Exhaustive check, not spot checks.** Built twenty synthetic
`ProgramSource`s (one terminator byte alone in LF and CR form; mid-CRLF
boundary; LF-CR with the empty line it produces; CRLF-pair then more lines;
sources of only LF terminators and only CRLF terminators; a trailing
terminator with nothing after it, in LF, CRLF, and bare-CR form; a leading
terminator; empty text; a single byte; a source mixing every terminator kind
in one buffer; the three-fragment shape; a Ctrl-Z-truncated source; an
`Interpret` source, empty and non-empty; the zero-line `Program` source) and
checked, for **every** `(start, end)` pair with `0 <= start <= end <= len`
(the backwards and past-the-end cases are separately covered by the
existing `None`-agreement behaviour, unchanged by this diff), that:
* `Cow::Borrowed` implies the span contains no terminator byte, and vice
  versa (ground truth for "terminator byte" computed independently, from
  `line_span`, never from `join_span` itself).
* `Cow::Owned` implies the joined bytes differ from the raw span bytes, and
  vice versa.

Result: 0 violations across every span of every source (hundreds of spans
total, full output available in the scratch harness). I could not construct
a counterexample in either direction, including every category the brief
named by name (a span of exactly one terminator byte, a span ending mid-CRLF,
a span over an LF-CR sequence, a source of only terminators, a Ctrl-Z
truncated source, a span whose start byte is itself a terminator).

**The new early return.** Proved by construction, not just read: for any
source with at least one line, the pre-existing fast path (`line_span(line_of(span.start))`
bounds-checked against the span) already returned `Borrowed` for every empty
span before this fix round, because `line_of` always resolves to a valid,
existing line whenever `line_count() >= 1`, and an empty span trivially
satisfies `line.start <= span.start && span.end <= line.end` at that line.
The walk that follows, when reached with an empty span, always appends
nothing from any line (every intersection has `start == end`) and always
returns `Cow::Owned(empty)` -- there is no other value it could have
produced. So the early return changes the `Cow` variant (`Owned` ->
`Borrowed`) only for the one source shape where the old fast path had
nothing to bounds-check against: the zero-line `Program` source. It never
changes a byte value anywhere, and it never bypasses a walk that would have
produced anything other than an empty result. No masking, no regression.

## Verification

* `cargo test --offline --workspace --no-fail-fast`: all suites pass, 0
  failed, at `ab07e536` after reverting the defeat experiment.
* `cargo clippy --offline --all-targets -- -D warnings`: clean.
* `git status --short`: clean.

## Verdict

Spec compliance: PASS. All five items from the review are addressed, and the
one area under suspicion (`join_span`'s contract and its new early return)
held up under exhaustive, code-level, independently-computed verification --
no counterexample found in either direction. One new Important finding: the
fix round's fourth new test (multi-label, probe J) does not exercise the
join at all, so the round's actual defeat-detected coverage for its own
stated goal is 3 of 4 new tests, not 4. Not a blocker; worth a line in the
next report so it doesn't get counted as regression coverage it isn't.
