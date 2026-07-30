# D10 — parser construction: the measurement

**Verdict: D10(b). Hand-written recursive descent above the token stream.
The plan's starting position (a), `chumsky` combinators above a hand-written
scanner, is contradicted on every axis that was measured.**

Measured 2026-07-28 on Linux, against `build/bin/rexx` for values and
`build/bin/rexxc` for syntax errors.
The spike wrote the expression grammar twice over one shared token stream, in
`rust/crates/rexx-parse-spike/`.
Both arms passed the same corpus: 59 expressions whose value the oracle prints,
27 malformed expressions whose error the oracle reports, and 7 expressions the
oracle accepts but the spike's evaluator cannot value.

**Reproducing.**
Task 3.1 Step 5 says to delete the spike crate, and this branch does not carry
it, but deleting it outright would leave every number below with nothing behind
it.
The crate as it stood when these numbers were taken is preserved on the unmerged
branch **`spike/d10`**, commit `afdd3b1b`, whose parent is the commit that added
this document.
From `rust/`:

```sh
cargo test --offline -p rexx-parse-spike            # 7 tests, both arms
cargo run --offline --release -p rexx-parse-spike --example throughput
```

`git show spike/d10 --stat` lists the ten files.
A patch of the same commit also sits at
`.superpowers/sdd/2026-07-28-phase-3-parser/d10-spike.patch`, which is gitignored
and therefore local to the machine that ran the spike.

## The four axes

| | Hand-written | `chumsky` 0.13 |
|---|---|---|
| Grammar lines of code | **277** | 336 (+21%) |
| Error fidelity, 27 cases | **27/27** | 27/27, but only with a hand-written pre-pass (below) |
| Grammar-layer throughput | **1.08–1.15 ms** | 9.3–12.1 ms (median **8.3×** slower) |
| Net new dependencies | **0** | 12 in this workspace, and a C compiler on the build path |

**Axis 2 has since been devalued, and the decision does not depend on it.** After
this spike ran, parse errors were scoped down: match the error number and
sub-number, drop byte-exact message text, drop error 36's position. The plan had
billed error fidelity as the axis "most likely to decide it", and this document's
strongest single argument was that error 36 is unreachable with `chumsky` 0.13.
Both of those are now worth much less than when they were written.

The verdict is unchanged, on the remaining axes alone, but the two remaining
axes are not equally strong and an earlier revision of this paragraph implied
they were.

**Axis 4, the dependency cost, stands on its own and is not an inference.**
Zero net new dependencies against twelve plus a C compiler on the build path,
on a project that gates every phase on five platforms including OpenBSD, is a
measured fact that needs no extrapolation. It carries the decision by itself.

**Axis 3, throughput, points the same way, and its magnitude is not
established.** The 8.3× is real but narrow: it was measured on the spike's
**expression grammar layer alone**, over the 1,912 of `CoreClasses.orx`'s 4,193
lines that parse as expressions, with the shared scanner explicitly subtracted
and the instruction and directive layers never written. Task 3.10's 3.29 ms is
a different measurement of different work: the whole shipped parser, scanning
and clause splitting and 500 directives and some 3,000 nested instructions.
Multiplying the one ratio onto the other number would conflate them, so this
document does not do it and does not state the product. An earlier revision
computed the figure and then disclaimed it in the same sentence, which is worse
than omitting it: a reader keeps the number and drops the caveat. Whatever the
combinator arm would have cost on the whole file, it would be weighed against a
~55 ms budget whose other components are all unmeasured, so whether it is
decisive depends on what bootstrap execution, heap setup and class construction
cost, and nobody knows yet.

Two earlier revisions of this paragraph got this wrong in opposite directions.
The first claimed parse time *is* cold-start time, which is the overreach the
document corrects two sections below. The correction removed that premise and
left the conclusion it had supported standing, so the paragraph asserted
throughput was "decisive by itself" with nothing behind it. Removing a false
premise does not license keeping the confident sentence it was holding up.

This is recorded rather than quietly left because a later reader finding a
decision resting on a criterion the project dropped a day afterwards would be
right to distrust it.

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

This was written while error numbers and sub-numbers were contract on a
byte-exact reproduction, and it said the axis was decisive on its own.
**It is not, and the devaluation at the top of this document governs.** Error 36
is still unreachable with `chumsky` 0.13 and the analysis below still holds; what
changed is that the project stopped gating on error 36's position at all, so an
argument built on it decides nothing now. Left in place because the mechanism is
worth knowing if combinators are ever reconsidered.

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

**Exactly what was run.** `rust/crates/rexx-parse-spike/examples/throughput.rs`,
built with `--release`, over
`interpreter/RexxClasses/CoreClasses.orx` (its default argument, and identical to
`build/bin/CoreClasses.orx`), 7 whole invocations of the program, each doing 50
timed passes per arm after one untimed warm pass:

```sh
cd rust && cargo run --offline --release -p rexx-parse-spike --example throughput
```

`CoreClasses.orx` is 4,193 lines, 3,268 of them non-blank.
The spike parses expressions and not instructions, so it cannot parse the file;
1,912 of those lines parse as expressions under both arms, 72,315 bytes, and the
two arms disagreed on acceptance for **zero** lines.
Only those 1,912 lines are timed, so both arms do identical work on identical
input, and the lines neither arm accepts cost nothing.

| | per pass, 7 runs | MB/s |
|---|---|---|
| Shared scanner alone | 0.62–0.68 ms | 106–116 |
| Hand, scanner included | 1.77–1.79 ms | 40–41 |
| `chumsky`, parser built once | 9.3–12.1 ms | 6.0–7.7 |
| `chumsky`, parser rebuilt per parse | 11.2–11.4 ms | 6.4 |

**What "grammar layer" excludes.** The shared scanner, which is one function
called by both arms, is timed on its own and subtracted from each arm's figure.
Nothing else is subtracted: both figures still include building the AST, and the
combinator figure in the third row excludes only parser *construction*, which the
fourth row prices separately.

That leaves 1.08–1.15 ms for the hand-written arm against 9.3–12.1 ms for the
combinators.
Per-run ratios across the 7 runs were 8.2, 8.3, 8.3, 8.3, 8.7, 8.7 and 10.5, so
the figure to quote is a **median of 8.3×**, not a single number.
The hand-written arm is the stable one; the spread is entirely in the combinator
measurement.
Rebuilding the combinator parser on every call costs only 1.1× on top of building
it once, so the gap is the combinators themselves and not construction overhead.

**This is not the D2 cold-start number.** Under D2 that figure is the time to
parse the whole file, and there is no instruction parser to measure yet.
It is a like-for-like comparison of the two arms on real input, which is what
the axis was for, and it had to be re-measured as a whole-file parse once a
whole-file parser existed.
Task 3.10 is that re-measurement and the next section is where it landed.
This paragraph named a Task 3.11 that the plan never had; the phase's twelve
tasks end at 3.10.

### Task 3.10's later, different measurement: the shipped parser

The number above is the spike's, and the spike is gone.
This is the shipped `rexx-parse` crate, `parse_program`, over whole files,
recorded here because it sits next to the spike's numbers and a later reader
should not confuse the two.

**Measured 2026-07-29, on the same Linux machine as the rest of this document**
(`AMD RYZEN AI MAX+ 395`, `rustc 1.96.1`), `cargo bench --offline -p rexx-parse
--bench parse`, criterion 0.8.2, `sample_size(10)`, 500 ms warm-up, 30 s
measurement ceiling -- matching `rexx-bench`'s and `rexx-num`'s benchmark
settings, so the methodology is comparable. `perf-baseline.md` carries no row
for `rexx-parse` or either `.orx` file, so nothing here is checked against a
value recorded there.

| File | Lines | Bytes | Time per parse | Throughput |
|---|---|---|---|---|
| `interpreter/RexxClasses/CoreClasses.orx` | 4,193 | 141,049 | 2.619–2.636 ms | 51.0–51.4 MiB/s |
| `interpreter/RexxClasses/StreamClasses.orx` | 1,010 | 37,603 | 651.75–659.11 µs | 54.4–55.0 MiB/s |

Two runs of the full benchmark suite agreed within criterion's own noise band
(criterion reported "no change in performance detected" against the first run
as baseline), so the ranges above are the union of both runs rather than one
run's confidence interval.

**The clone.** `parse_program` takes `Vec<u8>` by value, because `Program`
retains the buffer for every node's span. Each sample needs its own owned
copy, and `rust/crates/rexx-parse/benches/parse.rs` uses criterion's
`iter_batched` to clone the file's bytes in an untimed setup closure, so the
timed region holds `parse_program` plus the cheap node-count check below, not
the clone. Measured separately, the clone costs about 1 us on the
141,049-byte file, under 0.1% of the parse -- small enough that including it
would barely move the number. It is excluded anyway, on principle rather than
because it would distort the result: the interpreter's own cold-start path
reads a file once and parses it once, and never pays for a clone at all.

**The assertion.** For both files, most of the content lives inside `::METHOD`,
`::ATTRIBUTE` and `::ROUTINE` bodies, not the main body -- measured,
`CoreClasses.orx`'s main body holds 41 instructions against 2,390 nested inside
its 347 directives' bodies (`StreamClasses.orx`: 7 main, 153 directives, 610
nested). The benchmark asserts all three counts every sample, so a parser that
stopped early -- after the main body, or after building directives without
their bodies -- cannot post a passing number.

Their provenance is not uniform, and stating it as if it were would overclaim.
`src/directive/tests.rs`'s `core_classes_parses` independently pins
`directives.len() == 347` for `CoreClasses.orx`, and
`the_other_shipped_packages_parse` pins `StreamClasses.orx`'s per-kind counts
(7/139/5/2), which sum to the 153 used here, though 153 itself is never
written there. The main-body and nested-instruction counts (41/7 and
2390/610) have no acceptance test anywhere in the tree: Task 3.10 measured
them for the first time and hardcoded them as the benchmark's own change
detector, not as an independently pinned value.

**What the triple does not observe.** All three counts are flat lengths, and
`If`, `Do` and `Select` carry target indices into these vectors rather than
owning nested vectors, so dropping any clause anywhere changes a count -- the
property that makes the triple worth asserting. It is blind to three things a
count cannot see: corrupted control-flow wiring (a jump index pointing at the
wrong instruction, every count unchanged), a body-boundary bug that moves a
clause from one directive's body into the adjacent one (the sum across
directives still holds), and anything inside an `Expr`. That limit is shared
with `directive/tests.rs`'s own acceptance tests, not introduced by this
benchmark, and it is recorded rather than closed: a benchmark is the wrong
place to grow a structural checksum.

**What this number is and is not.** It is `parse_program`'s cost alone, over
the two files the Rust build parses at every interpreter start under D2. It is
**not** cold-start time: bootstrap execution, heap setup and class construction
are not measured anywhere yet, and D10's own history is why this document says
so explicitly rather than leaving it to be inferred -- this document previously
had to be corrected for claiming parser throughput "sets cold-start time
directly," and a milliseconds-small parse number inviting the same
inference by omission would repeat that mistake. The honest statement is: of
the ~55 ms cold-start budget, parsing `CoreClasses.orx` and `StreamClasses.orx`
together costs 3.27–3.30 ms combined (2.619–2.636 ms plus 651.75–659.11 µs) on
this machine, in this build profile. Nothing here says whether the other
components fit, because none of them has been measured.

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

* **Superseded by a scope decision, and kept as background rather than as a
  requirement.** Parse errors no longer have to reproduce the C++ 1:1: the agreed
  line is to match the error number and sub-number, drop byte-exact message text,
  and drop error 36's position substitution entirely. So nothing below is work
  Task 3.3 owes. It is recorded because the measurement was taken and because the
  claim it corrects — "there is no column anywhere in the oracle", which this
  plan's earlier drafts asserted and three review rounds acted on — is false and
  should not be re-asserted.

  Errors 36.901 and 36.902 substitute, for the offending token, the **1-based byte
  offset within its own physical line** and that line's number.
  The C++ stores both in every token, as `SourceLocation`'s `startLine` and
  `startOffset` (`SourceLocation.hpp:52`), and substitutes `getOffset() + 1` and
  `getLineNumber()`.
  A Rust token would **not** need to store them if this were ever gated: a
  whole-file byte offset plus a binary search into the line index Task 3.2's
  `ProgramSource` already builds for `SOURCELINE` yields the same pair, and the
  spike's own quadruple was a convenience rather than a requirement.
  Two things such a derivation would have to get right.
  It has to search on the **token's own** start offset and not the clause's,
  because the line reported is the token's physical line: a comma continuation
  makes 36.901 say "line 3" for a clause whose trace header says line 2.
  And the line index has to place line starts after any `\r`, so that an in-line
  offset on a CRLF file does not count the carriage return.
  No token's text crosses a line break — a literal that tries to is error 6.2 or
  6.3, and comments are not tokens — so a single start offset always lands inside
  one line.
* The offset is **bytes, not characters**, and a tab counts as one.
  `x = "ää" || (a` reports position 15, where the `(` is the 13th character.
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
if it guesses: `x = a.` is a valid stem reference, and `x = f(,)` is a valid call.

**Corrected by Task 3.5: `f(,)` passes ZERO arguments, not two.** Measured with
`arg()` inside the callee: `f(,)` gives **0** and `f(1,)` gives **1**, because
`parseArgList` returns `realcount` and pops trailing omitted arguments. An array
literal does not: `parseFullSubExpression` returns `total`, so `(1,)` is a
two-element array while `f(1,)` is a one-argument call. The same trailing comma
means different things in the two forms, which is exactly the shape of thing a
parser gets wrong by analogy.

Note how the original error was made, because the instrument caused it: a first
probe used `~items`, which counts non-nil elements and therefore cannot tell a
two-element array with a hole from a one-element array. `~size` is the one that
answers the question.
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
naming: under D2 the parse runs at every program start and so sits inside cold
start, which makes 8.3× in the grammar layer a cost paid on every invocation
rather than once.
