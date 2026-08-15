# Task 3.4 report: Clause splitting

Status: **DONE_WITH_CONCERNS**.
Everything the brief asked for is implemented, tested and differential-tested against the oracle.
The one concern is a visibility contradiction in the brief that I resolved rather than asked about; it is described in full below.

## Commits

| SHA | What |
|---|---|
| `5eec1fcb` | `src/clause.rs`, the `lib.rs` re-export, and `tests/clause.rs` |

Base was `cdda308e`.
Test summary: **78 tests pass** in `rexx-parse` (13 `sourceline`, 13 `tokens`, 35 `scanner`, 17 `clause`), `cargo clippy --offline --all-targets -- -D warnings` clean, `cargo fmt -p rexx-parse` applied, zero `unsafe`.

## The brief contradiction I resolved

The brief declares `pub(crate) struct Clause` and, three lines above, names `rust/crates/rexx-parse/tests/clause.rs` as the test file.
An integration test in `tests/` is a separate crate, so it cannot name a `pub(crate)` item; only one of the two can hold.
I made `Clause` and `split_clauses` `pub` and re-exported them from `lib.rs`, which is what every other item in this crate already does (`Token`, `Scanned`, `ParseCtx`, `TokenCursor` are all `pub` and re-exported), and the crate has no unit tests in `src/` at all.
The alternative, keeping `pub(crate)` and moving the tests into `src/clause.rs`, would contradict the brief's explicit file list and the crate's convention at once.
This changes a visibility annotation and no behaviour.
If Task 3.9 lives outside `rexx-parse`, as "Task 3.9 reconstructs `*-*` lines from it" suggests, `pub` is required anyway.

Two smaller decisions in the same area:

* I kept `#[derive(Clone, Debug)]` exactly as the brief gives it. I had briefly added `PartialEq, Eq`, then removed them again because no test compares a whole `Clause`; the tests compare `Range`s.
* I kept the specified `Result<Vec<Clause>, ParseError>` even though no input can currently reach an `Err`. The doc comment says so as a fact about today's rules, and records *why*: the one label error the C++ raises, 47.1 for a label in `INTERPRET` text (`InstructionParser.cpp:156`, measured below), needs the source kind, which `split_clauses(&[Token])` is not given. **Task 3.6 owns that error**, because `ParseCtx` carries `source`.

## What the code does

`split_clauses` walks the token vector. For each position that is not an `Eoc` it finds the terminating `Eoc`, then peels labels off the front of that physical clause:

* Ordinary clause: `tokens = start..eoc`, `span = tokens[start].span.start .. tokens[eoc].span.end`, `label = None`.
* Label clause: `tokens = start..start+1`, `span = tokens[start].span.start .. colon.span.end`, `label = Some(start..start+1)`, then continue from `start+2` with the same `eoc`, because `a: b: nop` is three clauses.

The label test is `start + 1 < limit && tokens[start] is Symbol|Literal && tokens[start+1] is Colon`.
`isSymbolOrLiteral` (`Token.hpp:580`) ignores the symbol's class, so a constant, a dot-symbol and a stem all label a clause; all four measured.
No blank can sit between a label and its colon, so the colon is at `start + 1` exactly: a blank is only a token when the next real character starts a symbol, a literal, a `(` or a `[`, and `:` is none of those. That is also where the C++ looks, with `nextToken` rather than `nextReal`.

`span` is never computed from the token range, per the brief's warning.
The label clause is the case that already proves the two must be separate: for `here: ; nop` the terminator is the `;`, not the colon, yet the span ends at the colon.

### The one C++ reading that changed my implementation

`RexxClause::trim` (`Clause.cpp:138`) moves the clause *start* and leaves the end, so a naive port would give the label clause the whole physical clause's span.
It does not, because `labelNew` (`InstructionParser.cpp:2792`-`2811`) overrides the instruction's end unconditionally:

```cpp
    // use the name object for tracking the location.
    SourceLocation location = colonToken->getLocation();
    // the clause ends with the colon.
    newObject->setEnd(location.getEndLine(), location.getEndOffset());
```

"Unconditionally" is the operative word, and it is why `here: ; nop` traces as `here:` and not `here: ;`.
The brief's rule 3 says the colon "terminates the clause when tokens follow it", which is true of *what starts next* but not of the label's own span end.
Probe p4 below is the measurement that pinned this.

### The line-end terminator's extent, which I checked rather than assumed

`sourceNextToken` (`Scanner.cpp:745`-`751`) does `location.setEndOffset(currentLength)` for `CLAUSE_EOL`, and `truncateLine` (`LanguageParser.hpp:162`) is `lineOffset = currentLength`, which moves the *offset*, not the length.
So a `--` comment does not shorten the line: the terminator still sits at the end of the line's content and the comment is inside the clause's span.
Measured (probe p6) rather than left as a reading: `say 1 -- trailing comment` traces in full.
Task 3.3's Rust `truncate_line` agrees, so nothing needed changing.

## Every oracle probe, with raw output

All run as `build/bin/rexx FILE` on a scratchpad file whose line 1 is `trace r`; the `trace r` clause itself is not traced.
Output is piped through `cat -A`, so `$` is end of line.

### Round 1: the span rules

```
=== p1 ===  nop;$ / say 1$
     2 *-* nop;$
     3 *-* say 1$
       >>>   "1"$
1$
=== p2 ===  here: nop$
     2 *-* here:$
     2 *-* nop$
=== p3 ===  here: nop; say "two"$
     2 *-* here:$
     2 *-* nop;$
     2 *-* say "two"$
       >>>   "two"$
two$
=== p4 ===  here: ; nop$
     2 *-* here:$
     2 *-* nop$
=== p5 ===  here:$ / nop$
     2 *-* here:$
     3 *-* nop$
=== p6 ===  say 1 -- trailing comment$
     2 *-* say 1 -- trailing comment$
       >>>   "1"$
1$
=== p7 ===  say 1   $
     2 *-* say 1   $
       >>>   "1"$
1$
```

p4 is the decisive one: the span ends at the colon although the `;` is the terminator.
p6 and p7 confirm the line-end terminator carries the rest of the line, comment included.

### Round 2: label shapes and continuations

```
=== q1 ===  "lit": nop$
     2 *-* "lit":$
     2 *-* nop$
=== q2 ===  1: nop$
     2 *-* 1:$
     2 *-* nop$
=== q3 ===  a: b: nop$
     2 *-* a:$
     2 *-* b:$
     2 *-* nop$
=== q4 ===  say 1,$ /   + 2$
     2 *-* say 1,  + 2$
       >>>   "3"$
3$
=== q5 ===  here : nop$
     2 *-* here :$
     2 *-* nop$
=== q6 ===  here /*c*/: nop$
     2 *-* here /*c*/:$
     2 *-* nop$
=== q7 ===  say 1;;say 2$
     2 *-* say 1;$
       >>>   "1"$
1$
     2 *-* say 2$
       >>>   "2"$
2$
=== q8 ===  .a: nop$
     2 *-* .a:$
     2 *-* nop$
=== q9 ===  stem.: nop$
     2 *-* stem.:$
     2 *-* nop$
```

q4 is worth flagging for Task 3.9: a multi-line clause's span contains the line terminator, and `trace r` prints the line fragments concatenated with the terminator dropped, `say 1,  + 2`.
Rendering the span is therefore not a plain byte slice. The span itself is `0..12` and is correct; the join is Task 3.9's.

### Round 3: terminator edge cases

```
=== r1 ===  say 1 ;$
     2 *-* say 1 ;$
       >>>   "1"$
1$
=== r2 ===  say 1;   $
     2 *-* say 1;$
       >>>   "1"$
1$
=== r3 ===  say 1   (no trailing newline)
     2 *-* say 1$
       >>>   "1"$
1$
=== r4 ===  here:   (no trailing newline)
     2 *-* here:$
=== r5 ===  /* only */$ / say 1$
     3 *-* say 1$
       >>>   "1"$
1$
=== r6 ===  here:: nop$
     2 *-* here:: nop$
Error 35 running .../r6.rex line 2:  Invalid expression.$
Error 35.1:  Incorrect expression detected at "::".$
=== r7 ===  say 1 /* c */ ; say 2$
     2 *-* say 1 /* c */ ;$
       >>>   "1"$
1$
     2 *-* say 2$
       >>>   "2"$
2$
```

r1 against r2 is the pair that pins "the terminator, not the line end": the blank before `;` is in, the blanks after it are out.
r6 confirms `::` is not a label; one clause, then a runtime 35.1.

### Round 4: a colon that is not a label, and stray semicolons

```
=== t1 ===  say a: b$
     2 *-* say a: b$
     2 *-* say a: b$
Error 98 running .../t1.rex line 2:  Execution error.$
Error 98.987:  Namespace "A" not found in package ".../t1.rex".$
=== t2 ===  ;;;$ / say 1$
     3 *-* say 1$
       >>>   "1"$
1$
=== t3 ===  say a b$ / nop$
     2 *-* say a b$
       >>>   "A B"$
A B$
     3 *-* nop$
```

t1's colon is a namespace qualifier, not a label, and the clause is not split; the doubled `*-*` line is the error traceback re-printing the failing clause, not a second clause.

### The `INTERPRET` label error, recorded for Task 3.6

```
$ build/bin/rexx s1.rex        # s1.rex: interpret "here: nop"
     1 *-* here: nop
     1 *-* interpret "here: nop"
Error 47 running .../s1.rex line 1:  Unexpected label.
Error 47.1:  INTERPRET data must not contain labels; found "HERE".
```

**Error 47.1.** `split_clauses` cannot raise it: it has no source kind.

## Differential test: 42 clauses, byte-identical

A single file exercising every shape above at once, no loops and no `IF`, so trace order equals clause order and one `*-*` line corresponds to one clause.
Its 30 lines cover bare and semicolon-terminated clauses, a blank before and after the `;`, an indented clause, a trailing `--` comment, a block comment before the terminator, five label spellings, a label alone on its line, stacked labels, a label followed by `;`, a doubled `;;`, assignments, a blank and a comment before the colon, a `,` continuation, and a namespace colon.

Compared the `*-*` texts against `String::from_utf8_lossy(&bytes[c.span])` with line terminators stripped, dropping the untraced `trace r` clause:

```
$ wc -l oracle2.txt mine2.txt
 42 oracle2.txt
 42 mine2.txt
$ diff oracle2.txt mine2.txt && echo "IDENTICAL"
IDENTICAL
```

The first attempt of this had 41 oracle lines against my 42, because `say a: b` aborted the run before the last clause.
Re-read rather than skimmed: the missing line was the trailing `nop` the interpreter never reached, and lines 1 to 41 already matched.
Moving the failing clause out gave the clean 42-of-42 above.

## Structural sweep: 1,039,513 clauses

A throwaway integration test (written, run, deleted; not committed) walked every `.rex`, `.orx`, `.cls`, `.testGroup` and `.testUnit` file in the repository, scanned it and split it, asserting:

* the token range is non-empty, never goes backwards, and holds no `Eoc`;
* `span.start` equals the first token's start, `span.start < span.end`, and `span.end <= text.len()`;
* `span.start >= ` the previous clause's `span.end`, so spans never overlap (they may have gaps, which gate criterion 1 permits);
* `ProgramSource::span_bytes(span)` is `Some`;
* a labelled clause's `label` equals its `tokens`, is one token long, and is followed by a `Colon`;
* every token is accounted for: `sum(clause.tokens.len()) + eoc_count + label_count == tokens.len()`.

```
12990 files, 12986 scanned, 1039513 clauses, 1781 labels
test corpus_invariants ... ok
```

The 4 unscanned files are scan errors, skipped rather than asserted on; Task 3.3 already covers those.

## Mutation checks on the test file

Three mutations, to confirm the tests would not pass with the logic subtly wrong.

| Mutation | Failures |
|---|---|
| `span_end` taken from the token before the terminator instead of the terminator | 5 of 17 |
| label never recognised (`labelled = false && ...`) | 6 of 17 |
| label clause's span stops at the name instead of the colon | 6 of 17 |

The first is the exact defect the brief warns about, deriving the span from the token range, and `a_clause_span_includes_its_terminating_semicolon`, `a_semicolon_ends_the_span_and_a_blank_before_it_does_not`, `a_line_end_terminator_carries_the_rest_of_the_line_into_the_span`, `repeated_semicolons_produce_one_clause_each_not_an_empty_one` and `a_terminator_belongs_to_no_clause_token_range` all catch it.
`src/clause.rs` was restored from a copy after each; `git diff --stat` confirmed a clean tree before the final run.

## What this task deliberately did not do

* **Rule 4.** No `THEN`/`ELSE`/`OTHERWISE` handling and no `split_before`. The clauses this produces are plain `Range`s over a shared token slice with nothing interned or aliased, so Task 3.6 can construct any sub-range and set any span end.
* **Error 47.1** for a label in `INTERPRET`, which needs `ParseCtx::source`.
* **Rendering a span.** `trace r` joins a multi-line clause's line fragments and drops the terminators; that is Task 3.9's, and probe q4 is the measurement it will need.

## Concerns for the coordinator

1. **`Clause` and `split_clauses` are `pub`, not `pub(crate)`.** The reason is above. If the intent really was `pub(crate)`, the brief's test file has to move into `src/`.
2. **The always-`Ok` `Result`.** Kept because the brief specifies it as a shared interface. If Task 3.6 turns out not to need it, it is a one-line change with no callers outside this crate yet.
3. **Task 3.9 needs q4.** A clause span can contain a line terminator, and `trace r` does not print it. Whoever writes 3.9 should start from `say 1,` / `  + 2` rather than from a single-line clause, or the `*-*` reconstruction will emit an embedded newline.
4. **Task 3.6 owes error 47.1.** Recorded here because this task found it and cannot raise it.
