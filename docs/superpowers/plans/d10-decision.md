# D10 — parser construction: the measurement

**Verdict: D10(b). Hand-written recursive descent above the token stream.
The plan's starting position (a), `chumsky` combinators above a hand-written
scanner, is contradicted on every axis that was measured.**

Measured 2026-07-28 on Linux, against `build/bin/rexx` for values and
`build/bin/rexxc` for syntax errors.
The spike wrote the expression grammar twice over one shared token stream, in
`rust/crates/rexx-parse-spike/`, and was deleted once these numbers were
recorded.
Both arms passed the same corpus: 59 expressions whose value the oracle prints,
27 malformed expressions whose error the oracle reports, and 7 expressions the
oracle accepts but the spike's evaluator cannot value.

## The four axes

| | Hand-written | `chumsky` 0.13 |
|---|---|---|
| Grammar lines of code | **277** | 336 (+21%) |
| Error fidelity, 27 cases | **27/27** | 27/27, but only with a hand-written pre-pass (below) |
| Grammar-layer throughput | **1.10–1.24 ms** | 9.6–10.6 ms (**8.6×** slower) |
| Net new dependencies | **0** | 12 in this workspace, and a C compiler on the build path |

Lines of code counts non-blank, non-comment lines outside the licence banner, in
the one file that holds each arm's grammar.
The shared token stream (507 lines), the shared AST (138), the shared evaluator
used only for checking against the oracle (151), and the 79 lines both arms
import are all excluded from both columns.

## Which stop condition ended each arm

The hand-written arm **reached a working expression grammar over the whole
corpus**, on the first run after it compiled: 59/59 values, 27/27 errors, 7/7
accepted.
It needed no fix rounds against the corpus at all.
Part of why is that it could mirror the oracle's own control flow directly, one
Rust function per `LanguageParser` method, so each error site sits where the
oracle's does.
That is an advantage of the approach rather than an artefact of writing it first:
both arms were written from the same reading of the same C++, and only one of the
two idioms can take that shape.

The combinator arm also **reached a working expression grammar**, on its fourth
attempt, with one qualification recorded below.
It did not stall, in the sense the task defines, because every attempt but one
moved the count.

* Attempt 1, the first build that ran: **23/27 errors**.
  `a +`, `a ||` and `\` reported 35.918 instead of naming the operator, and
  `(a[1` reported the outer parenthesis where the oracle reports the inner
  bracket.
* Attempt 2, replacing `or_not()` with an explicit omitted-argument parser and
  adding a position-preferring `merge` to the error type: **23/27, nothing
  moved**.
* Attempt 3, `require_term`: a rewinding lookahead that checks for the term
  before `pratt` ever tries to parse it, so the operator can name itself:
  **25/27**.
  It regressed `a + * b`, which named `+` where the oracle names `*`.
* Attempt 4, teaching `require_term` to distinguish a terminator from a token
  that merely cannot start a term, plus `unmatched_opener`, a hand-written
  bracket-balance pre-pass over the token stream: **27/27**.

**The qualification.** `(a[1` was attacked twice and was never fixed inside the
combinators.
Attempt 4 reaches 27/27 by computing error 36 in a hand-written loop over the
tokens, before the grammar runs.
So the honest statement of the fidelity result is: *the combinator arm cannot
produce error 36, and reaches parity only by delegating that error to
hand-written code.*

### Why error 36 is out of reach

`repeated()`, `or_not()` and `choice()` all rewind a partially-consumed
alternative and throw its error away.
The bracket error in `(a[1` is raised inside a message cascade, which is a
`repeated()`, so it never reaches the caller and the outer parenthesis wins.
chumsky 0.13 has no cut or commit combinator: `grep -rE 'fn cut|Committed|
no_backtrack' src/*.rs` over the crate source finds nothing.
There is therefore no lever to keep the inner error, and the error type's
`merge` is not called at all, because chumsky has already discarded the loser.

This is the axis Phase 3's gate checks, and error numbers **and sub-numbers** are
contract, so it is decisive on its own.

### Errors 36.901 and 36.902 do contain a position, and it is a byte offset

This corrects a fact the task brief states.
There is no column in `condition('o')`, but two of the messages the parser must
produce substitute one:

```
Error 36.901:  Left parenthesis "(" in position 5 on line 3 requires a corresponding right parenthesis ")".
Error 36.902:  Square bracket "[" in position 7 on line 2 requires a corresponding right square bracket "]".
```

`LanguageParser::errorPosition` (`LanguageParser.cpp:4114`) substitutes
`tokenLocation.getOffset() + 1` and `tokenLocation.getLineNumber()`, so the
value comes straight off the offending token.
It counts **bytes, not characters**: `x = "ää" || (a` reports position 15, where
the `(` is the 13th character and the 15th byte.
A tab counts as one.

## Axis 3 in detail, and what it is not

`CoreClasses.orx` is 4,193 lines, 3,268 of them non-blank.
The spike parses expressions and not instructions, so it cannot parse the file;
1,912 of those lines parse as expressions under both arms, 72,315 bytes, and the
two arms disagreed on acceptance for **zero** lines.
Those lines were then timed, 50 passes each, after one untimed warm pass.

| | per pass | MB/s |
|---|---|---|
| Shared scanner alone | 0.59–0.66 ms | 110–123 |
| Hand, scanner included | 1.76–1.83 ms | 39–41 |
| `chumsky`, parser built once | 9.6–10.6 ms | 6.8–7.6 |
| `chumsky`, parser rebuilt per parse | 11.4–12.9 ms | 5.6–6.3 |

Subtracting the shared scanner leaves the layer D10 is about: 1.10–1.24 ms
against 9.6–10.6 ms, a ratio of **8.6×**.
Rebuilding the combinator parser on every call costs only 1.1–1.2× on top of
building it once, so the gap is the combinators themselves and not construction
overhead.

**This is not the D2 cold-start number.** Under D2 that figure is the time to
parse the whole file, and there is no instruction parser to measure yet.
It is a like-for-like comparison of the two arms on real input, which is what
the axis was for, and it must be re-measured as a whole-file parse when Task
3.11 exists.

## Axis 4, the dependency cost

Recorded rather than re-derived, per the task.
`chumsky` 0.13.0 pulls 28 transitive packages against the hand-written arm's
zero; in this workspace, which already has some of them, adding it locked **12
new packages**: `chumsky`, `stacker`, `psm`, `object`, `ar_archive_writer`,
`hashbrown`, `foldhash`, `equivalent`, `allocator-api2`,
`unicode-segmentation`, `windows-link`, `windows-sys`.
`stacker` → `psm` has a `build.rs` with `cc` in its build-dependencies, so
choosing `chumsky` puts a **C compiler on the Rust build path**, and
`ci/platforms` builds five platforms including OpenBSD.

One thing the task did not know: `pratt`, the module that makes the precedence
table tractable, is **not** in `chumsky`'s default feature set.
`default = ["std", "stacker"]`, and `pratt = []` must be enabled explicitly.

## The hazard the plan bet on does not discriminate

The parent plan singles out `f(x)` against `f (x)` as the combinator hazard.
It is not one.
The oracle keeps them apart in its **scanner**: a blank becomes a `TOKEN_BLANK`
operator when the previous token was a symbol, a literal, `)` or `]`
(`RexxToken::isBlankSignificant`, `Token.hpp:595`) and the next real character
starts a symbol, a quote, `(` or `[` (`Scanner.cpp:754`).
Once the token stream carries that token and nothing above it skips whitespace,
both constructions get the case right for free, and both did, on the first
attempt.

The case is still worth keeping in the corpus, because it fails loudly for any
parser that pads or skips whitespace.
It just is not evidence about D10, and the axis that actually separated the two
arms was error fidelity.

## Step 3b — the AST's shape

**A flat instruction chain in one arena per code body, not a tree.**

```rust
pub struct CodeBody {
    pub instructions: Vec<Instruction>,
    pub first: Option<InstructionId>,
}

pub struct Instruction {
    pub kind: InstructionKind,
    pub clause_span: (usize, usize),
    pub next: Option<InstructionId>,
}
```

Nesting lives in the indices a block instruction holds, never in owned children.

* It is what the oracle does.
  `RexxInstruction` has `RexxInstruction *nextInstruction; // the next
  instruction object in the assembled chain` (`RexxInstruction.hpp:103`), and
  `RexxActivation` walks it (`RexxActivation.cpp:583`, `:640`, `:724`).
  Control flow that jumps — `SIGNAL`, `ITERATE`, `LEAVE`, the `END` of a `DO` —
  maps one to one instead of needing translation.
* It satisfies Step 3b's constraint by construction rather than by discipline.
  `THEN`, `ELSE`, `OTHERWISE`, `WHEN` and `END` are each an ordinary link in the
  chain with their own `clause_span`, and there is no parent node for them to be
  absorbed into.
  Under a tree the natural shape is `If { then: Box<Instruction> }`, and the
  constraint then has to be remembered at every one of the five sites.
* It matches the arena idiom D1 already settled and `rexx-core` already uses, so
  `Vec` plus an index newtype replaces a chain of `Box`.
* Phase 4's dispatch becomes `while let Some(id) = next`, which is also the
  natural place to emit one `*-*` line per clause span.

Verified on the oracle that all five keywords trace as clauses of their own, so
all five need somewhere to keep a span:

```
$ build/bin/rexx t1.rex          # trace r; if 1 = 2 then say "a"; else say "b"
     2 *-* if 1 = 2
       >>>   "0"
     3 *-*   else
     3 *-*     say "b"
```

`if 1 = 1 then say "a"` likewise prints three `*-*` lines for one source line;
`when 0 = 1 ` prints with its trailing blank, `otherwise` prints alone without
one, and `end` prints its own line.

The cost of the chain is that an `InstructionId` is not type-checked against the
body it indexes.
`InstructionId(u32)` is a newtype and Phase 4 should debug-assert the bound, the
same trade D1 already accepted for object handles.

## The token stream Task 3.3 has to build

Settled by the spike and by reading the oracle's scanner.
This is the part of the spike worth keeping, since the crate itself is gone.

* Every token carries `(start_line, start_offset, end_line, end_offset)`.
  Lines are 1-based, offsets are **0-based byte offsets within the token's own
  physical line**, and errors 36.901 and 36.902 substitute `start_offset + 1`
  and `start_line`.
* The line reported is the token's own physical line, not the clause's first.
  A comma continuation makes 36.901 say "line 3" for a clause whose trace header
  says line 2.
* `Blank` is a token, emitted under the two-part rule quoted above.
  Nothing above the scanner may skip it.
* Abuttal has **no** token.
  The oracle synthesises a zero-length operator when a symbol, a literal or `(`
  appears where an operator was expected (`LanguageParser.cpp:2878`).
* `,` **and `-`** at end of line are clause continuations, and a continuation
  acts as a significant blank.
  `x = 5 -` with `3` on the next line is `5 3`, not `2`.
  `--` is a line comment, which is why `v = 1 --1` leaves `v` as `1`.
* A closing quote followed immediately by `x`/`X`/`b`/`B`, with no further symbol
  character, makes the literal hexadecimal or binary.
  This is why `a''b` is just `a` and why `"["x` is a hex literal and an error.
* Terminator sets are a **runtime parameter** of the expression grammar, and a
  parenthesised subexpression drops the enclosing set
  (`LanguageParser.cpp:3777`).
  The blank operator additionally is not an operator when the next real token is
  a terminator, which is what stops `do i = 1 to 3` concatenating `3` with `TO`.

Two places where the spike is deliberately simpler than what Task 3.3 will
build, so that this document does not read as a specification of them.
The spike's symbol names are owned `String`s, where Task 3.3 interns them.
The spike's expression nodes carry no span of their own, only the token
locations needed for the errors above, where the real AST holds a byte range per
node for `SOURCELINE`, error reporting and `TRACE`.
Neither changes any measurement here, because both arms held the same types.

Grammar facts the corpus pinned:

* Every dyadic operator is left-associative, `**` included.
  `2 ** 3 ** 2` is 64, not 512, and `a = b = c` with `2 2 1` is 1, not 0.
* Prefix `+`, `-` and `\` bind above `**`.
  `-2 ** 2` is 4 and `-2 ** -2` is 0.25.
* `~` must be followed by a symbol or a string.
  `a~[3]` is error 19.909, but `a~1` parses, because a number is a symbol.
* `abs ('2.5')` is `ABS 2.5`, with one blank.
  The task brief says `ABS2.5`; the blank operator inserts a blank and the
  abuttal operator does not.

## The error corpus, with the oracle's answers

Every case was run through `build/bin/rexxc` as the line `x = <expr>` on line 3
of a file, so the byte positions below are positions within that line.
`36.901` and `36.902` carry `[position|line]`; the rest carry the quoted token,
where the message has one.

| Expression | Oracle |
|---|---|
| `)` | 37.2 |
| `]` | 37.901 |
| `a[1]]` | 37.901 |
| `1 2 3)` | 37.2 |
| `a b )` | 37.2 |
| `(a))` | 37.2 |
| `a +` | 35.1 `+` |
| `a \|\|` | 35.1 `\|\|` |
| `a \|\| \|\| b` | 35.1 `\|\|` |
| `a + * b` | 35.1 `*` |
| `**2` | 35.1 `**` |
| `~a` | 35.1 `~` |
| `a %% b` | 35.1 `%` |
| `a \(1 = 2)` | 35.1 `\` |
| `()` | 35.1 `(` |
| `\` | 35.901 `\` |
| `(a` | 36.901 `[5\|3]` |
| `((a)` | 36.901 `[5\|3]` |
| `f(a b` | 36.901 `[6\|3]` |
| `a[` | 36.902 `[6\|3]` |
| `a[1` | 36.902 `[6\|3]` |
| `(a[1` | 36.902 `[7\|3]` |
| `a~` | 19.909 |
| `a~~` | 19.909 |
| `a~b~` | 19.909 |
| `'unterminated` | 6.2 |
| `a~"b` | 6.3 |

Two results that look like errors and are not, and that a parser will get wrong
if it guesses: `x = a.` is a valid stem reference, and `x = f(,)` is a valid call
with two omitted arguments.
`x = a b if` is also valid, because keywords are not reserved.
`x = -` is 35.918 rather than 35.901, because the trailing `-` is a continuation
and the expression is empty.

## What would change this decision

* **A cut or commit combinator in `chumsky`.**
  If a partially-consumed alternative's error survived `repeated()`, `or_not()`
  and `choice()`, the error-36 objection largely dissolves and the question is
  worth reopening.
  This is the single technical change that would do it.
* **Phase 3's gate dropping substitution-value fidelity** and asking only for the
  major number.
  Then only the throughput and dependency axes remain, and both are softer.
* Nothing in the lines-of-code axis, in either direction.
  The instruction layer is 35 keywords of mostly straight-line token matching,
  which is where combinators offer least, so the +21% is more likely to widen
  than to close.

The throughput gap will not change the answer on its own, but it is worth
naming: under D2 the parse is cold-start time on every program run, so 8.6× in
the grammar layer is a cost paid on every invocation, not once.
