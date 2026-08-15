# Task 3.1 report — the D10 spike

**Status: DONE_WITH_CONCERNS.**
The decision is unambiguous and every axis was measured, but three things in the
brief turned out to be wrong and one of them is a requirement on Task 3.3.

**Commits on `plan/rust-rewrite`:**

* `43c4ba5f` — "Decide D10 with measurements". One file,
  `docs/superpowers/plans/d10-decision.md`.
* `4abd61f4` — the follow-up asked for by the coordinator: the exact throughput
  reproduction, the per-run ratio spread, a pointer to the preserved spike, and
  the token-offset refinement below.

**Commit on the side branch `spike/d10`: `afdd3b1b`** — "Preserve the D10 spike so
its measurements can be audited".
Parent is `43c4ba5f`; not merged, and `plan/rust-rewrite` does not carry the
crate.
It was built with a temporary index (`GIT_INDEX_FILE`), `git write-tree`,
`git commit-tree` and `git update-ref`, so `HEAD`, the real index and the working
tree were never switched — another agent is working in this worktree under
`docs/`, and a `git checkout` would have moved files under it.

The spike crate is deleted from the working tree and `rust/Cargo.lock` is byte
for byte its committed content, verified two ways: `git diff --stat
rust/Cargo.lock` returns nothing, and its SHA-256 is `57bbb894…` both before the
spike was created and after it was removed.
The same commit is also written out as a patch at
`.superpowers/sdd/2026-07-28-phase-3-parser/d10-spike.patch`, 2,740 lines; that
directory is matched by `.gitignore:19`, confirmed with `git check-ignore -v`, so
it does not dirty the tree.

**Before archiving, the restored crate was re-run to confirm the archive is the
exact state the numbers came from: 7 tests, all green.**

**Tests: 7 spike tests green, both arms — 59 value cases against `build/bin/rexx`,
27 error cases against `build/bin/rexxc`, 7 accept-only cases, plus a
terminator-context case and an arm-agreement case. Whole workspace green,
`cargo clippy --offline --all-targets -- -D warnings` clean.**

## The verdict

**D10(b): hand-written recursive descent above the token stream.** The plan's
starting position (a) is contradicted on all four axes.

| | Hand | `chumsky` 0.13 |
|---|---|---|
| Grammar lines of code | **277** | 336 (+21%) |
| Error fidelity, 27 cases | **27/27** | 27/27 only with a hand-written pre-pass |
| Grammar-layer throughput | **1.08–1.15 ms** | 9.3–12.1 ms (median **8.3×**) |
| Net new dependencies | **0** | 12, and a C compiler on the build path |

## Stop conditions

* Hand arm: **reached a working expression grammar**, first run after it
  compiled, no fix rounds against the corpus.
* Combinator arm: **reached a working expression grammar** on attempt 4, with the
  qualification that error 36 is computed by a hand-written bracket-balance loop
  and not by the combinators.
  It did not stall; the counts moved on attempts 3 and 4.
  Attempt 2 moved nothing.

Attempt by attempt, on the 27 error cases:

* Attempt 1, first build that ran: **23/27**.
  `a +`, `a ||` and `\` gave 35.918 instead of naming the operator, because
  `pratt` parses the right-hand side itself and its failure is a bare "unexpected
  end of input" that the furthest-error rule then lets win.
  `(a[1` named the outer parenthesis where the oracle names the inner bracket.
* Attempt 2, an explicit omitted-argument parser in place of `or_not()`, plus a
  position-preferring `merge` on the error type: **23/27, nothing moved**.
* Attempt 3, `require_term` — a rewinding lookahead that checks for the term
  before `pratt` can try it, so the operator names itself: **25/27**.
  Regressed `a + * b`, which named `+` where the oracle names `*`.
* Attempt 4, `require_term` distinguishing a terminator from a token that merely
  cannot start a term, plus `unmatched_opener`, a hand-written bracket-balance
  pre-pass: **27/27**.

**The single strongest argument in the decision, recorded here as well as in the
document: error 36 is unreachable with `chumsky` 0.13.**
`repeated()`, `or_not()` and `choice()` all rewind a partially-consumed
alternative and throw its error away.
The bracket error in `(a[1` is raised inside a message cascade, which is a
`repeated()`, so it never reaches the caller and the outer parenthesis wins.
There is no cut or commit combinator to stop that:
`grep -rE 'fn cut|Committed|no_backtrack' src/*.rs` over the 0.13.0 crate source
finds nothing.
The custom error type's `merge` is not even called, because chumsky has already
discarded the loser, so there is no lever at that level either.
Attempt 4 reaches 27/27 only by computing error 36 in a hand-written loop over the
tokens before the grammar runs, which is to say the combinators do not produce it
at all.

## Axis 3: exactly what was run

`rust/crates/rexx-parse-spike/examples/throughput.rs`, from `rust/`:

```sh
cargo run --offline --release -p rexx-parse-spike --example throughput
```

* **Release**, not debug.
* Input is `interpreter/RexxClasses/CoreClasses.orx`, the example's default
  argument, byte-identical to `build/bin/CoreClasses.orx`.
* 7 whole invocations of the program; each does **50 timed passes** per arm after
  **one untimed warm pass**.
* Of 3,268 non-blank lines, the **1,912** that parse as expressions under both
  arms are timed, 72,315 bytes. The two arms disagreed on acceptance for zero
  lines, which is itself a cross-check.
* **What is excluded from "grammar layer":** only the shared scanner, which is one
  function both arms call, timed on its own and subtracted from each arm's total.
  Nothing else. Both figures still include building the AST. The prebuilt
  combinator row additionally excludes parser *construction*, which the fourth row
  prices separately at 1.1× on top — so the gap is not construction overhead.

| | per pass, 7 runs |
|---|---|
| Shared scanner alone | 0.62–0.68 ms |
| Hand, scanner included | 1.77–1.79 ms |
| `chumsky`, parser built once | 9.3–12.1 ms |
| `chumsky`, parser rebuilt per parse | 11.2–11.4 ms |

Grammar layer: **1.08–1.15 ms** against **9.3–12.1 ms**.
Per-run ratios were 8.2, 8.3, 8.3, 8.3, 8.7, 8.7 and 10.5, so the figure to quote
is a **median of 8.3×**; my first message said 8.6×, which was a single run.
The hand-written arm is the stable side and the spread is all in the combinator
measurement.
This is **not** the D2 cold-start figure and cannot be until Task 3.11 exists,
because the spike has no instruction parser and therefore cannot parse the file.

## Concerns

**1. The oracle does expose a position, and it is a byte offset. Task 3.3 has to
be able to produce it.**
The brief says there is no column anywhere in the oracle and not to measure on
one.
That holds for `condition('o')`, but errors 36.901 and 36.902 substitute a
character position: `Left parenthesis "(" in position 5 on line 3 requires...`.
`LanguageParser::errorPosition` (`LanguageParser.cpp:4114`) substitutes
`tokenLocation.getOffset() + 1` and `getLineNumber()`, and the value counts
**bytes**, not characters: `x = "ää" || (a` reports position 15 where the `(` is
the 13th character.
**On the coordinator's refinement: agreed, and my original wording overstated
it.** A stored quadruple is not needed.
A whole-file byte offset plus a binary search into the line index Task 3.2's
`ProgramSource` already builds for `SOURCELINE` yields both the physical line and
the in-line byte offset, and it is exactly equivalent, not an approximation.
The spike stored the quadruple only because it had no `ProgramSource` to search.
Take the derivation approach and record it in Task 3.3.

Two things the derivation has to get right, and both are cheap.
It must search on the **token's own** start offset, never the clause's, because
the line reported is the token's physical line — which is precisely the
continuation case the coordinator measured, where the main message says line 2 and
the substitution says line 3.
And the line index must place each line's start **after** any `\r`, so that an
in-line offset on a CRLF file does not count the carriage return; the C++ never
hits this because it slices lines before scanning them.
Nothing else is at risk: no token's text crosses a line break, because a literal
that tries to is error 6.2 or 6.3 and comments are not tokens, so a single start
offset always lands inside one line.

I have rewritten this bullet in `d10-decision.md` to say derive-don't-store, so
Task 3.3 does not inherit the overstatement.

**2. `abs ('2.5')` is `ABS 2.5`, not `ABS2.5`.**
The brief states the latter.
The blank operator inserts one blank and the abuttal operator does not; a blank
is present here, so the result carries a blank.
The corpus case is correct in the committed document.

**3. The hazard the plan bet on does not discriminate between the two options.**
`f(x)` against `f (x)` is settled entirely in the scanner: the oracle emits a
`TOKEN_BLANK` when the previous token was a symbol, literal, `)` or `]`
(`Token.hpp:595`) and the next real character starts a symbol, quote, `(` or `[`
(`Scanner.cpp:754`).
Both arms got it right on the first attempt, because neither skips whitespace.
Keep the case in the corpus, since it fails loudly for any parser that pads, but
it is not evidence about D10.
The axis that actually decided it was error fidelity, which is also the axis the
Phase 3 gate checks, so the plan's instinct about *which axis matters* was right
even though its instinct about *which case* was not.

## Smaller findings, all in the committed document

* `pratt`, the chumsky module that makes the precedence table tractable, is
  **not** a default feature. `default = ["std", "stacker"]` and `pratt = []`.
* Axis 3 is not the D2 cold-start figure and cannot be until Task 3.11 exists.
  It times 1,912 real `CoreClasses.orx` lines that both arms accept as
  expressions, 72,315 bytes; the two arms disagreed on acceptance for zero lines.
  Rebuilding the combinator parser per call costs only 1.1–1.2× on top of
  building it once, so the 8.6× is the combinators and not construction.
* `-` at end of line is a clause continuation exactly like `,`, and a
  continuation acts as a significant blank: `x = 5 -` with `3` on the next line
  is `5 3`, not `2`. `--` is a line comment, which is why `v = 1 --1` leaves `v`
  as `1`.
* Error 37 has two distinct sub-numbers: 37.2 for a stray `)`, 37.901 for a
  stray `]`.
* `~` requires a symbol or string after it, so `a~[3]` is 19.909, but `a~1`
  parses because a number is a symbol.
* `x = a.`, `x = f(,)` and `x = a b if` all parse. `x = -` is 35.918 and not
  35.901, because the trailing `-` is a continuation and the expression is empty.
* The unmatched opener that 36.901/36.902 name is the **innermost still
  unmatched** one.

## Step 3b

**Flat instruction chain in one arena per code body, not a tree.**
`Vec<Instruction>` with `next: Option<InstructionId>`, nesting held as indices.
Reasons: it is what the oracle does
(`RexxInstruction.hpp:103`, walked in `RexxActivation.cpp:583`), it satisfies the
five-keyword constraint by construction rather than by discipline because there
is no parent node for `THEN`, `ELSE`, `OTHERWISE`, `WHEN` or `END` to be absorbed
into, it matches the arena idiom D1 settled, and Phase 4's dispatch becomes
`while let Some(id) = next`, which is also where one `*-*` line per clause span
naturally hangs.
Verified all five trace as separate clauses myself rather than taking the brief's
word: `if 1 = 2 then say "a"` / `else say "b"` under `trace r` prints `*-*` lines
for `if 1 = 2`, `else` and `say "b"` on their own.

## Two procedural notes

`HEAD` was `a9bbb2a1` when I committed, not the `b61d586a` the brief names.
A concurrent commit, "Intern symbols in Task 3.3, and rule out hash-consing the
AST", landed in between.
I read it and added a short paragraph to the decision document saying where the
spike is deliberately simpler than what Task 3.3 will build — owned `String`
symbol names rather than interned ones, and no per-node span — so that the two
documents do not contradict each other.

The spike is preserved twice over, as described at the top: branch `spike/d10`
(`afdd3b1b`) and the gitignored patch at
`.superpowers/sdd/2026-07-28-phase-3-parser/d10-spike.patch`.
The branch is the one to use, because it can be run rather than only read.
A third copy sits in this session's scratchpad as `rexx-parse-spike-archive/`; it
is the same bytes and will not outlive the session.
