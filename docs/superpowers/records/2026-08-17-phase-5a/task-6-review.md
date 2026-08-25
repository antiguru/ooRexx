# Task 6 review: the native-method frame in a traceback, the operator half

Base `fe0364c08` .. head `4f6359b84`, both commits.

## Spec Compliance

**Issues found**, but the brief's literal "Done when" is met: the three named programs
(`say b. + 1`, `say b. ** 2`, `say -b.`) match the oracle byte for byte on both engines,
each joins `rust/corpus/phase-5a.txt` and `RAW_STDERR_COMPARISON` in the fix commit, the
const's doc block is corrected, and the sitting is recorded.

Verified independently, not taken from the report:

* **The three programs, and eleven forms they do not cover.** Ran the oracle wrapper
  (three descriptors, fresh empty directory, absolute paths) and `target/release/rexx-run`
  under `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`. `say b. + 1` matches, and so do
  `say b. * 2`, `say b. / 2`, `say b. - 2`, `say b. % 1`, `say b. // 1`, `say +b.`,
  `x = b. + 1`, `if b. + 1 = 2 then nop`, `say b.1 + 1`, `b.1 = "abc"; say b.1 + 1`. The
  frame's method name is `Operator::spelling`/`PrefixOp::spelling`
  (`rust/crates/rexx-parse/src/token.rs:243`, `.../ast.rs:91`), whose arithmetic arms are
  `+ - * / % // **` -- the oracle's own operator message names -- so the name is read, not
  assumed, for every arithmetic shape and not only the three probed.
* **The binary is not stale.** `target/release/rexx-run` (12:56:39) postdates both
  `eval.rs` (12:55:45) and `dispatch.rs` (12:45:51), and the working tree's line numbers
  match the diff exactly, so the probes above ran the head build.
* **The corpus arithmetic.** Program lines across the four `SUBSET_FILES`
  (`corpus.rs:589`): 31 + 12 + 12 + 54 = **109**. 106 + 3, nothing silently left the set.
  `phase-5a.txt` and `EXPECTED_SUBSET_5A` are +3 entries with zero deletions.
* **`coverage.rs`'s six lines are warranted and mandatory**, not scope creep: three entries
  plus their comment in `EXPECTED_SUBSET_5A`, which
  `phase_5a_subset_matches_the_committed_list` (`coverage.rs:941`) asserts equal to
  `phase-5a.txt`. Omitting them is a structural red. The global constraint's
  ownership-moves-in-one-commit rule names this file explicitly.
* **The three `sourceline_oracle/*.txt` fixtures are also mandatory**, not extra:
  `crates/rexx-parse/tests/sourceline_oracle.rs` covers "every line of every
  `rust/corpus/lang/` program". Counts 8/8/7 match the `.rex` files' own line counts, and
  the bodies match line for line. All three crash their prolog, so the driver takes its
  documented `SIGNAL ON SYNTAX` fallback; the fallback is faithful here because none of the
  three has CRLF, an embedded `CTRL-Z`, or a missing final newline.
* **The corrected doc block is true and constraint-clean.** `corpus.rs:333-338` states a
  present-tense property, names no count, and carries no historical marker; struck of
  "rather than an iteration over zero rows" it still says the same thing.
* **The `DO`-header claim in the pre-existing doc is right about the code.**
  `arith_left_operand` has exactly two callers (`eval.rs:817` prefix, `eval.rs:950`
  `arith_general`); `Interp::header_number` (`run.rs:7254`) does not call it, so this
  change cannot fire in a `DO` header. See Important 3 for what that costs.
* **A trapped condition cannot leave a stale frame.** `blame_native_method` sets
  `failure_site`; `lib.rs:1903` records that a trapped condition clears `failure_site` and
  `failure_sites`, and `run.rs:4025` takes it. The new call site inherits that, so
  `signal on syntax` over a stem operand cannot decorate a later traceback.

**Cannot verify from the diff:** the five gate commands and the "98 test binaries, 0
FAILED" claim (not re-run, per instruction); the negative control (this checkout is
read-only, so the `if false &&` edit cannot be reproduced here); that the three
`sourceline_oracle` fixtures came from the live oracle driver rather than from the `.rex`
files. The controller should treat the last as cheap to re-derive.

## Strengths

* **The placement is genuinely shared.** One predicate in one function reached by both
  engines through `eval_arithmetic`/`eval_prefix` and `Op::Arith`/`Op::Prefix`; no
  per-engine duplication, no `Op::Generic`. My probes confirm both engines emit identical
  bytes on all eleven extra forms.
* **The scope-versus-receiver distinction the controller flagged is correct.** The frame
  names `.String`'s id read through the object model (`dispatch.rs:375`), not the
  receiver's class, and `say b. + 1`'s bytes prove it: `b.~class` is `The Stem class`, the
  frame says `String`. A build that named the receiver's class would redden all three
  corpus programs.
* **The frame is on the failing path only.** The new check sits inside the
  `Err(NotNumeric)` arm behind `operator_operand_gap`, so no succeeding arithmetic touches
  it.
* **The gap-then-stem ordering is right.** A stem whose default is a class object is caught
  by `operator_operand_gap` first and gets its loud refusal, not a frame.
* **The doc block correction is a real correction**, not a reshuffle: it removes a false
  vacuity claim, a count, and historical framing in one sentence.

## Issues

### Critical (Must Fix)

None.

### Important (Should Fix)

**1. `is_stem_receiver` over-fires: we now emit a frame the oracle does not.**
`rust/crates/rexx-exec/src/eval.rs:1038-1042`, doc claim at `1060-1061`.

Measured, three descriptors, fresh directory:

```
s. = .nil
say s. + 1
```

| | first stderr line | rc |
|---|---|---|
| oracle | `     2 *-* say s. + 1` (no frame), then `Error 97.1: Object "The NIL object" does not understand message "+".` | 159 |
| head, both engines | `       *-* Compiled method "+" with scope "String".` | 215 |
| `bench-baselines/pinned/rexx-run-15a1ffa98` (pre-5a) | `     2 *-* say s. + 1` (no frame) | 215 |

The pinned pre-5a build emits no frame line here; head does. **This commit adds a stderr
line the oracle does not emit.** The oracle's frame exists only when the stem's forward
lands on a native method that *runs and raises*; with a `.nil` default the send finds no
`+` at all, dispatch fails before any activation, and there is no frame. The predicate
"is the receiver a stem at all" cannot see that.

The doc states the over-broad rule as a measured conclusion: "so the test cannot be 'does
the default fail to parse' and must be 'is the receiver a stem at all'" (`eval.rs:1060-1061`).
The premise (unset `b.` and `String`-defaulted `s.` behave alike) is true; the conclusion
does not follow, because it ignores a default that answers no operator at all. The same
overstatement is cross-referenced from `dispatch.rs:863` and paraphrased in
`corpus/phase-5a.txt`.

Why it matters beyond the one program: the corpus cannot catch this. `s. = .nil; say s. + 1`
already diverges on the error number (41 vs 97), so no row changes colour, and the next task
that fixes 97.1 for object-valued stem defaults inherits a wrong frame it did not add.

Fix: narrow the predicate to a stem whose default value is the shape whose native operator
would actually run (a string/number value), and rewrite `eval.rs:1053-1061` to state that
rule rather than "is the receiver a stem at all". If narrowing is deferred, the limitation
belongs in the report and the ledger with the `.nil` repro, not left as a doc claiming the
predicate is exactly right. (`.object~new` and `.array~of` are loud-refused this phase, so
`.nil` is today's only reachable witness -- that is a phase accident, not a bound.)

**2. The stated reason for the `RAW_STDERR_COMPARISON` entries is false.**
`rust/crates/rexx-exec/tests/corpus.rs:308-312`, repeated in `rust/corpus/phase-5a.txt`
("because the frame's own leading whitespace and its absent line number are part of the
bytes").

The comment says the default comparison "would collapse the frame's own leading run of
spaces". It would not. `normalize_line` (`crates/rexx-exec/tests/support/mod.rs:222-243`)
copies `line[..PREFIX_OFFSET + PREFIX_LENGTH]` verbatim -- the seven leading spaces *and*
the marker -- and collapses only the space run **after** the marker, "and *only* that first
run". `od`-verified, the oracle's frame line is 7 spaces, `*-*`, exactly **one** space, then
content, so normalisation is a no-op on it; and `blame_native_method`'s own doc records that
the line carries no indent at any depth, so it is always one space. The neighbouring clause
line (`     8 *-* say b. + 1`) is likewise one space. **Normalisation cannot change any byte
of these three programs' stderr**, so raw mode buys nothing here and the negative control
would have reddened in either mode.

The entries themselves are fine -- raw is strictly stronger, and the brief mandates them
("Each program joins `phase-5a.txt` and `RAW_STDERR_COMPARISON` in this commit"), so this is
**plan-mandated** and should stay. What must change is the recorded reason, in both places:
say that the brief requires raw comparison for the frame's bytes, not that normalisation
would eat them. A reader trusting the current sentence would believe the default comparison
is unsafe for `*-*` lines generally, which is exactly the sort of false-but-load-bearing
justification `RAW_STDERR_COMPARISON` exists to prevent one level up.

**3. An operator-invoked frame the oracle emits and we still do not: the `DO` header.**
`rust/crates/rexx-exec/src/run.rs:7254` (`header_number`), unreferenced by the change.

Measured, both engines, oracle stderr's first line vs ours:

| program | oracle | head |
|---|---|---|
| `do i = b. to 5` | `       *-* Compiled method "+" with scope "String".` | absent |
| `do i = 1 to b.` | same | absent |
| `do i = 1 by b. to 3` | same | absent |

rc 215 on both sides; the frame line is the **only** difference. (`for` and the repetitor
are 26.x on both sides with no frame, and match.) This is **not a regression** -- it
diverged before the change too -- but it is squarely inside the task's own goal sentence
("an error raised inside a native method that an *operator* invoked"), and
`arith_left_operand`'s own pre-existing doc (`eval.rs:1004-1007`) already says why: each
`DO` header position "is rounded through a unary operator of its own", which is precisely
what makes the oracle raise a frame there.

It is recorded nowhere. The report's "What this task does not close" names only the 5b scope
case and asserts "Nothing in this task's own scope was left undone"; Task 7's brief covers
an explicit `~inherit` send, not this. Under the global constraint "Say what a check could
not see", three programs one keystroke away from the ones just committed, differing in
exactly the line this task exists to produce, must be either fixed (route `header_number`'s
initial/to/by rounding through the same frame) or written down as knowingly open. Silence
is the one option the plan rules out.

**4. Two global-constraint violations in new comment prose.**

* `rust/crates/rexx-exec/src/eval.rs:1055-1056`: "one of the **two** object shapes no
  operator answers" names the size of a set. The constraint is verbatim: "A comment may not
  name the size of a set. Name the set; true counts included." It goes stale the moment a
  third shape joins `operator_operand_gap`. Fix: "one of the object shapes no operator
  answers".
* `rust/crates/rexx-exec/src/dispatch.rs:861`: "`pub(crate)` **since Phase 5a Task 6**: ..."
  is historical framing, and it fails the constraint's own deciding test -- strike "since
  Phase 5a Task 6" and the sentence says the same thing about the code as it is, so the
  framing was decoration. Fix: drop the clause; the timeless half ("an arithmetic operator's
  left operand can raise from inside this same shape of frame, and `eval.rs` is a different
  module") is the whole justification.

(The `// Task 6:` labels in `phase-5a.txt`, `coverage.rs` and the three `.rex` headers are
the established convention in those files -- `// Task 8, ruling R26:` and friends sit
directly above them -- and are not this finding.)

**5. The performance guard's staleness test failed as written, and was reasoned about
rather than obeyed.** Report's "Staleness check"; `bench-baselines/phase-5a-arms.tsv`.

The constraint's test is `git diff <pin-commit> HEAD -- rust/crates rust/Cargo.toml
Cargo.toml` being empty, and it says in as many words: "the pin must then be rebuilt and the
baseline restarted **rather than reasoned about**". The diff is not empty (the report says
8828 lines; every earlier 5a task added files under `rust/crates/*/tests/`). The implementer
narrowed the path spec to the four crates' `src/` and declared the pin valid.

The narrowing is very probably what the constraint's author intended -- the same paragraph
argues for "a test phrased over *what the crate source is*" -- but it is a deviation from a
binding control, and it also **left out a crate the pinned binary actually links**: the
narrowed spec covers `rexx-exec`, `rexx-core`, `rexx-classes`, `rexx-lib` only, while
`rust/crates/rexx-parse/src` also changed between the pin and head. Checked: the only file
touched there is `instruction/tests.rs`, a `#[cfg(test)]` module with no effect on the
release binary, so the conclusion survives -- but the check as run did not establish that,
and the report's sentence "zero commits have touched any of those four crates' `src/`" is
true of a set that is narrower than the binary's inputs.

Fix is a controller ruling, not a redone sitting: amend the constraint's path spec to the
crates and directories that actually build `rexx-run` (`rexx-parse/src` included, `tests/`
excluded), so the test passes exactly when the pin is still right. Re-running the sitting
would reproduce these numbers.

### Minor (Nice to Have)

* `rust/crates/rexx-exec/src/eval.rs:1053` -- "**The one** predicate ... needs" is the same
  cardinality shape as Important 4's first bullet, one step softer.
* `rust/corpus/phase-5a.txt` (new block, "**The three** vary the operator") -- names a set
  size in a comment; goes stale if a fourth operator program joins the block.
* `rust/crates/rexx-exec/src/eval.rs:1030` -- "That path is `Interp::arith_operand`'s, not
  this one, **and unchanged here**." "Unchanged here" describes the edit, not the code;
  struck, the sentence is unharmed.
* **The report's performance explanation is refuted by its own rows.** It says "the change
  adds no work to any path they exercise", but `arith`'s `pinned>head across_builds`
  `instructions:u` is 1.000453/1.000552 (small) and 1.000466/1.000552 (large) with
  `value_min == value_max` -- deterministic, not noise, ~3M instructions on 6.5e9.
  `arith_left_operand` is on the arith axis's hot path even though the new branch is not
  taken, so this is codegen/layout in a function the axis runs. Below the 1% threshold, so
  correctly "not a finding" per the constraint; but the stated mechanism is wrong and should
  read as a layout effect rather than as an absence of added work.
* **The negative control's transcript is partly paraphrased.** The header
  (`106 of 109 matching`), the owner tally and the panic line read as real harness output,
  but program 1's rust/oracle strings are elided with `...` and programs 2 and 3 are summarised
  as "(same shape, "**")". The claim "all three reddened **byte-exactly** ... missing exactly
  the frame line, nothing else" therefore rests on the implementer's summary for two of the
  three. Paste the three mismatch blocks unedited next time. (Given Important 2, note also
  that the control would have fired under the normalised comparison too, so it does not
  witness the raw-mode opt-in.)

## Assessment

**Task quality: Needs fixes.**

The core mechanism is right and better-verified than the brief asked for -- eleven operator
forms beyond the three probes match the oracle byte for byte on both engines, and the
scope-versus-receiver subtlety the brief warned about is handled correctly. But the
predicate is broader than the oracle's rule and now emits a `Compiled method` line the
oracle does not for a stem with a `.nil` default (proved against the pre-5a pinned binary),
while its doc asserts that predicate is exactly right; the reason recorded for the
`RAW_STDERR_COMPARISON` entries is false against `normalize_line`'s actual behaviour; and
three `DO`-header programs differ from the oracle in exactly the line this task exists to
produce, recorded nowhere.
