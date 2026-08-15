# Task 3.9 report: TRACE source lines (`*-*` only)

Status: DONE.
Commit: `6614ac34` (`Reconstruct the clause text TRACE prints, terminators joined out`).

## What was built

`ProgramSource::join_span(Range<usize>) -> Option<Cow<'_, [u8]>>` in `rust/crates/rexx-parse/src/source.rs`, the terminator-stripping join the brief specifies.
It walks the line index and concatenates each line's intersection with the span, so the bytes dropped are exactly the terminators inside the span and nothing else.
It is ported from `ProgramSource::extract` (`interpreter/parser/ProgramSource.cpp:153`), whose multi-line branch concatenates `getStringLine` results, which are line content with terminators excluded.
The single-line case, which every uncontinued clause hits, returns a borrowed `Cow` equal to `span_bytes`; `None` falls out exactly where `span_bytes` answers `None`.
Bytes throughout, never `str`; no new raw-text accessor was needed, and the forbidden whole-text accessor remains absent.

Eight tests were added to `rust/crates/rexx-parse/tests/sourceline.rs`, kept in that file as directed.
The shared helper `assert_traced` takes measured `*-*` lines as (printed line number, instruction index, stripped text) triples and checks each line against the clause it came from, per line rather than as a sequence, because `trace r` re-traces a loop body per iteration.
It also checks that the printed line number is the clause's first line via `line_of(clause_span.start)`.

## Oracle output captured

All captures were made 2026-07-28 as `( ulimit -v 1048576; build/bin/rexx FILE ) 2>&1 | cat -A` and live in the session scratchpad (`trace_output.oracle.catA`, `probe_a.oracle.catA` through `probe_f.oracle.catA`).
The scratchpad is session-scoped, so the unfiltered captures are reproduced here in full; the value-marker lines are Phase 4's evidence.

`rust/corpus/lang/trace_output.rex` (sets `trace i`, so value markers and program output interleave):

```
     2 *-* x = 1 + 1$
       >L>   "1"$
       >L>   "1"$
       >O>   "+" => "2"$
       >>>   "2"$
       >=>   X <= "2"$
     3 *-* y = x * 3$
       >V>   X => "2"$
       >L>   "3"$
       >O>   "*" => "6"$
       >>>   "6"$
       >=>   Y <= "6"$
     4 *-* if y > 5 $
       >V>   Y => "6"$
       >L>   "5"$
       >O>   ">" => "1"$
       >>>   "1"$
     4 *-*   then$
     4 *-*     say "big"$
       >L>       "big"$
       >>>       "big"$
big$
     5 *-* trace off$
done 6$
```

Probe A (`/* probe A: terminators are inside the clause span */` / `trace r` / `nop;` / `do i = 1 to 2; say i; end` / `trace off`):

```
     3 *-* nop;$
     4 *-* do i = 1 to 2;$
       >K>   "TO" => "2"$
     4 *-*   say i;$
       >>>     "1"$
1$
     4 *-* end$
     4 *-* do i = 1 to 2;$
       >>>     "1"$
       >>>     "2"$
     4 *-*   say i;$
       >>>     "2"$
2$
     4 *-* end$
     4 *-* do i = 1 to 2;$
       >>>     "2"$
       >>>     "3"$
     5 *-* trace off$
```

Probe B (`/* probe B: a label is its own clause, colon included */` / `trace r` / `here: nop; say "two"` / `trace off`):

```
     3 *-* here:$
     3 *-* nop;$
     3 *-* say "two"$
       >>>   "two"$
two$
     4 *-* trace off$
```

Probe C, continuation over LF (`/* probe C: continuation join */` / `trace r` / `say "x",` / `    "y"` / `trace off`):

```
     3 *-* say "x",    "y"$
       >>>   "x y"$
x y$
     5 *-* trace off$
```

Probe D (`trace r` / `say 1,` / `  + 2` / `trace off`):

```
     2 *-* say 1,  + 2$
       >>>   "3"$
3$
     4 *-* trace off$
```

Probe E, probe C's CRLF spelling (every terminator `\r\n`):

```
     2 *-* say "x",    "y"$
       >>>   "x y"$
x y$
     4 *-* trace off$
```

Probe F (`trace r` / `interpret 'nop; say 1'` / `trace off`), for the `Fragment` path:

```
     2 *-* interpret 'nop; say 1'$
       >>>   "nop; say 1"$
     2 *-* nop;$
     2 *-* say 1$
       >>>   "1"$
1$
     3 *-* trace off$
```

Probes C and E confirm the join's shape directly: comma kept, terminator dropped (both bytes of a CRLF), the continuation line's four leading blanks kept, and the `*-*` line number is the clause's first line.
Probe F shows interpreted clauses trace from the fragment's own text (`nop;` with its semicolon), printed on the `INTERPRET` instruction's line, which is the caller's to resolve.

## Clause spans repaired

None.
Every reconstruction matched the oracle byte for byte from the spans Tasks 3.4 and 3.6 already produce, so `src/clause.rs`, `src/ast.rs` and `src/instruction.rs` are unmodified.

That is a finding, not a default, because the tests could have shown otherwise, and these are the constructs whose spans they actually exercised:

* Assignment and keyword clauses terminated by line end (`x = 1 + 1`, `trace off`).
* An `IF` condition keeping its trailing blank and stopping at the start of its terminator (`if y > 5 `).
* A bare `THEN` mid-line carrying no blank on either side, with the interstitial bytes after it in no clause.
* The `THEN` arm as its own clause (`say "big"`).
* Clauses terminated by an explicit `;`, semicolon included (`nop;`, `do i = 1 to 2;`, `say i;`).
* `END` as a clause of its own, line-end terminated.
* A label with its colon (`here:`).
* Comma continuation over LF and over CRLF, spans containing the terminator (pinned: `say 1,` / `  + 2` parsed alone has clause span `0..12`, and `span_bytes` on probe C's clause still contains the `\n`).
* An `INTERPRET` fragment's clauses via `parse_interpret`.

Constructs this task did not exercise: `ELSE` and `OTHERWISE` end bytes (chosen by the same `split_before` call path as `THEN`, but not observed here), `WHEN` conditions, and multi-label clauses (`a: b: nop`).
Task 3.6's unit tests cover those spans against single-line oracle measurements; this task adds no multi-line evidence for them.

## Nothing unreconstructable

Every `*-*` line in every capture reconstructs byte-identically after stripping the line number, the marker and the leading indentation.

## Verification

* `cargo test --offline --workspace --no-fail-fast`: 557 passed, 0 failed.
* `cargo clippy --offline --all-targets -- -D warnings`: clean.
* `cargo fmt` run before committing.

## What went wrong

* Two of my three pre-flight findings were already the coordinator's own defects, and I resolved the brief's self-contradiction (`say "x","y"` vs "four leading blanks kept") by measuring before asking rather than taking either half of the brief on faith.
  Measuring first was right; what was wrong is that my Step 1 probes initially existed only to settle that question, and I nearly filed them as incidental rather than as the captures the task requires.
* The Step 2 red state was a compile error (`join_span` does not exist), not a behavioral failure, so the run never demonstrated `span_bytes` producing wrong text.
  The behavioral trap is pinned differently: two tests assert the raw span still contains the terminator that the join drops, so a future regression to a raw slice fails on content, not on a missing symbol.
* My probe files carry a comment header line the brief's probe listings do not, so my printed line numbers differ from a headerless run of the same probes (probe C prints line 3 here, line 2 in the coordinator's re-measurement).
  The expectations encode my exact files, and the captures above are of those files; anyone re-running the probes must use the scratchpad files, not the brief's listings, or the line-number assertions will look wrong.
* The Files list predicted span repairs and there were none, which means Step 3's repair contingency went unexercised and the "repair the clause, never widen the node" rule remains untested advice in this codebase.
  If a later construct needs it, this task's tests only guarantee the repair will be noticed, not that it will be easy.
* One degenerate corner is deliberately inconsistent: an empty span at the end of a zero-line source joins as an owned empty `Cow` where a one-line source would borrow.
  No clause span can produce it, and papering over it would have cost a special case with no measurable behavior behind it.

## Fix round, 2026-07-29

Review verdict was spec PASS with four items, all addressed in commit `ab07e536`.
The reviewer's defeat experiment also corrected my report upward: replacing `join_span`'s body with `span_bytes` turns four tests red, not the two I credited.

### Item 1: Coverage gap closed

All four flagged constructs were re-measured against the oracle myself, per the standing rule, and all four reproduced the reviewer's strings byte-identically on unmodified code.
Missing tests, not bugs.
The captures live in the scratchpad as `probe_g` through `probe_j` and are reproduced here unfiltered.

Probe G, continued `ELSE` arm (`trace r` / `if 1 = 2 then nop` / `else say 1,` / `    2` / `trace off`):

```
     2 *-* if 1 = 2 $
       >>>   "0"$
     3 *-*   else$
     3 *-*     say 1,    2$
       >>>       "1 2"$
1 2$
     5 *-* trace off$
```

Probe H, continued `OTHERWISE` arm (`trace r` / `select` / `  when 1 = 2 then nop` / `  otherwise say 1,` / `    2` / `end` / `trace off`):

```
     2 *-* select$
     3 *-*   when 1 = 2 $
       >>>     "0"$
     4 *-*   otherwise$
     4 *-*     say 1,    2$
       >>>       "1 2"$
1 2$
     6 *-* end$
     7 *-* trace off$
```

Probe I, three-fragment continuation (`trace r` / `say 1,` / `  2,` / `    3` / `trace off`):

```
     2 *-* say 1,  2,    3$
       >>>   "1 2 3"$
1 2 3$
     5 *-* trace off$
```

Probe J, multi-label clause (`trace r` / `a: b: nop` / `trace off`):

```
     2 *-* a:$
     2 *-* b:$
     2 *-* nop$
     3 *-* trace off$
```

Probe H additionally evidences the `WHEN` condition's trailing blank, which the original round had left to Task 3.6's single-line pins.
Four tests were added through `assert_traced`, one per probe.

### Item 2: Floor under the expectation lists

`assert_traced` now rejects an empty expectation list, and its doc comment states that completeness is the caller's obligation and points at this report as the location of the unfiltered transcripts.
Exhaustiveness stays non-mechanical, as directed, because the transcript is not in the tree.

### Items 3 and 4: Comments

The two structuring semicolons are split, in `join_span`'s doc comment and in the corpus test's comment.
The value-marker list in that comment now says those are the markers that file happens to emit, with the authority stated as "everything except `*-*`", eighteen prefixes.

### The degenerate corner, resolved toward the code

`join_span`'s fast path now covers the empty span, so an empty span borrows even on a zero-line source and the contract reads exactly: borrowed when the span contains no terminator byte, owned when the join dropped something.
The `Cow` contract test pins the corner.
Chosen over the doc-only option because a caller matching on `Cow` needs the exact rule, and one special-cased sentence is a worse contract than one uniform fast path.

### Verification after the fix round

* `cargo test --offline --workspace --no-fail-fast`: 561 passed, 0 failed.
* `cargo clippy --offline --all-targets -- -D warnings`: clean.
* `cargo fmt` run before committing.

### What went wrong, fix round

* My original report undercounted my own tests' strength: the defeat experiment shows four tests red under a `span_bytes` regression where I claimed two.
  I had reasoned about which tests target the join instead of running the replacement, and the lesson is the phase's recurring one: claims about what a test catches are measurements, not inferences.
* All eight original tests joined exactly two fragments, so the join loop's second iteration had never executed until probe I.
  I flagged `ELSE`/`OTHERWISE` as unevidenced myself but did not notice the fragment-count blind spot, which the reviewer did.
