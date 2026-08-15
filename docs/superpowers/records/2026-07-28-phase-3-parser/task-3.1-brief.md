## Task 3.1: The D10 spike — decide parser construction

**This task produces a decision and a document, not production code.** Its
output governs every later task's shape, so it goes first and is timeboxed.

**Files:**
- Create: `rust/crates/rexx-parse-spike/` (a scratch crate, deleted at the end)
- Create: `docs/superpowers/plans/d10-decision.md`

**Interfaces:**
- Produces: the decision recorded in `d10-decision.md`, plus a token-stream
  shape that Task 3.3 will implement for real.

Implement **the expression grammar only** — precedence, abuttal
concatenation, message sends (`~`, `~~`), function and array-reference forms,
compound variables — twice over the same hand-written token stream:

1. with `chumsky` 0.13.0 combinators
2. by hand, recursive descent

Do **not** implement all 35 instructions twice. The expression grammar alone
is enough signal; the parent plan says so explicitly and the timebox exists to
stop this becoming Phase 3 itself.

- [ ] **Step 1: Extract the expression corpus**

The L0 corpus at `rust/corpus/lang/` has 14 programs. Pull every distinct
expression shape out of them plus these, which cover the forms that separate
the two approaches:

```rexx
a + b * c                      /* precedence */
a b c                          /* abuttal concatenation */
a || b                         /* explicit concatenation */
obj~method(1, 2)               /* message send */
obj~~method                    /* cascading message send */
arr[i, j]                      /* array reference */
stem.i.j                       /* compound variable */
f(g(h(x)))                     /* nested calls */
-x ** 2                        /* prefix vs power binding */
a = b = c                      /* LEFT-associative. With a=2 b=2 c=1 this is
                               1: (a=b)=c is (2=2)=1 is 1=1 is 1, where
                               a=(b=c) would give 2=(2=1) is 2=0 is 0.
                               All-equal operands cannot tell them apart. */
f(x)                           /* call */
f (x)                          /* NOT a call -- abuttal of f and (x) */
say a""b                       /* NOT abuttal: ""b is an empty BINARY
                               literal, so this prints just a */
```

`f(x)` versus `f (x)` is the one the parent plan singles out as the combinator
hazard, because a blank changes a function call into a concatenation. Verified
they differ. Do not omit it — an earlier draft of this plan did, and it is
precisely the case that separates the two D10 options.

For each, capture the interpreter's answer with a driver that prints the
evaluated result, so both spike implementations are checked against the same
ground truth rather than against each other.

- [ ] **Step 2: Build both implementations**

Same token stream, same AST output type. **Equal effort on the two arms, not
equal wall time** — the implementer has no clock, so a wall-clock timebox is not
a rule it can follow.

An arm stops on the first of two conditions: it reaches a working expression
grammar over the whole Step 1 corpus, or it visibly stalls. Stalled means the
same construct has been attacked three times without the arm getting closer —
the same test still fails, or the fix for one construct broke another that was
passing. Once one arm reaches a working grammar, the other gets the same *number
of attempts* and then stops wherever it is.

Record, in those words, which of the two conditions ended each arm. "The
combinator arm never reached a working expression grammar" is a legitimate and
decisive result and must be recorded as the outcome, not as an incomplete
measurement. Pushing an arm past its stall point destroys the comparison,
because the two attempts then differ in effort rather than in difficulty.

- [ ] **Step 3: Measure the axes**

The parent plan fixes axes 1–3 and no others. Axis 4 is not a measurement of the
two implementations; it is a property of the dependency, already known, and it
belongs in the same document because it decides the same question.

1. **Lines of code** — the whole expression grammar, excluding the shared
   token stream.
2. **Error fidelity** — at each failure site, can the exact interpreter error
   number, sub-number and **line** be produced, along with the substitution
   values the message quotes? Test with deliberately malformed expressions and
   compare against `build/bin/rexxc`, which gives the parse verdict without
   executing the file. This is the axis most likely to decide it,
   because it is what Phase 3's gate checks. There is **no column** anywhere in
   the oracle — do not measure the spike on one, and note that ooRexx locates
   an error by quoting the offending token, so the substitution values are what
   actually pin the position.
3. **Parse throughput on `CoreClasses.orx`** — not on synthetic input. Under
   D2 this number is cold-start time.
4. **Dependency and portability cost.** `chumsky` 0.13.0 pulls **28 transitive
   packages**; the hand-written arm pulls **zero**. `chumsky`'s `default`
   feature set is `["std", "stacker"]`, and `stacker` depends on `psm`, which
   has a `build.rs` with `cc` in its `[build-dependencies]`. So choosing
   `chumsky` puts **a C compiler on the Rust build path**. This branch's CI
   builds five platforms including OpenBSD (`ci/platforms`), so that is a
   portability cost, not a convenience cost. Record it as a fact in
   `d10-decision.md` alongside the three measurements; do not re-derive it.

- [ ] **Step 3b: Decide the AST's shape — tree or flat instruction chain**

The spike builds an AST either way, so settle this while it is cheap. The C++
links instructions into a **chain** (each node points to the next) rather than
nesting them in a tree, and D13's text says "plain owned Rust data inside one
arena object per code body" — the arena half of which an earlier draft of this
plan dropped.

The two shapes give Phase 4 different dispatch loops: a chain walks a `next`
pointer, a tree recurses. Getting it wrong is not a parser fix, it is a Phase 4
rewrite. Record the choice and the reason with the D10 decision.

**One constraint binds the decision, whichever shape wins.** All five of `THEN`,
`ELSE`, `OTHERWISE`, `END` and `WHEN` must remain instructions of their own, each
carrying its own `clause_span`. **None of the five may be absorbed into a parent
node**, however tempting that is under the tree outcome.

The reason is that the oracle traces each as a separate `*-*` clause, so each
needs somewhere to keep its own span, and Task 3.7b keeps no separate clause
list — an absorbed keyword would leave those bytes with nowhere to live.
Measured, all five: `RexxInstructionThen` sets its location to the `THEN` token's
own (`ThenInstruction.cpp:76`), and `trace r` on `if 1 = 1 then say "a"` prints
three `*-*` lines for one source line; `end` prints its own line; `otherwise`
prints alone with no trailing blank; `when 0 = 1 ` prints with its trailing
blanks.

Two of the five are gated directly, and the other three are not, so do not look
for all five in gate criterion 6: `THEN` appears via `trace_output.rex` and `END`
via Task 3.9 Step 1's probe A. `ELSE`, `OTHERWISE` and `WHEN` fall outside
criterion 6's three-file scope but are bound by the same rule, because Task 3.6
Step 4 needs a node named for each of the 35 keywords and the `samples/`
round-trip criterion parses all five in quantity.

This is a constraint on the choice, not the choice itself, and it is stated here
rather than nine tasks downstream because this is where it is cheap to honour.
Task 3.1 is the first task executed, so a shape chosen without it is a shape that
fails Task 3.6 Step 4 five tasks later.

- [ ] **Step 4: Write `d10-decision.md`**

Record the three measurements, the dependency cost from axis 4, which stop
condition ended each arm, the Step 3b shape decision, the overall decision, and
— importantly — what would change it. The parent plan's starting position is
(a): hand-written scanner and clause splitter with `chumsky` above the token
stream. State plainly if the measurements contradict that.

- [ ] **Step 5: Delete the spike crate and commit**

`rust/Cargo.toml` has `members = ["crates/*"]`, so creating and deleting the
spike crate never touches it. The decision document is the only file to stage.

```bash
rm -rf rust/crates/rexx-parse-spike
git add docs/superpowers/plans/d10-decision.md
git commit -m "Decide D10 with measurements"
```

---

