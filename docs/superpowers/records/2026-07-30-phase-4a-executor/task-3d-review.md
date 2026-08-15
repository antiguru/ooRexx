# Review of Task 3d, commit ae7e8bce

## Verdicts

**Spec compliance: PASS.** All six plan steps done, nothing skipped or
faked. **Code quality: PASS.** 0 blocking, 0 major, 1 minor (comment style
only), plus one informational observation team-lead asked me to think
through explicitly.

Everything below is independently reproduced, not accepted from the report:
every cliff, the mutation test, the shared-budget design argument (tested
against a from-scratch alternative implementation, not just "remove the
guard"), and the const assertion. Read-only throughout: no commits, no
branches, no worktrees; historical states pulled with `git archive` into
scratch copies, never `git checkout` on the reviewed tree.

## Inputs

- `docs/superpowers/plans/2026-07-30-phase-4a-executor.md`, "Task 3d" section.
- `.superpowers/sdd/2026-07-30-phase-4a-executor/task-3d-report.md`.
- `git show ae7e8bce` (full diff, all four files:
  `expr.rs`, `lib.rs`, `tests/deep.rs`, `examples/depth_probe.rs`).
- Direct commands against the current tree (HEAD == `ae7e8bce` for
  `rexx-parse`; confirmed with `git log --oneline ae7e8bce..HEAD --
  rust/crates/rexx-parse/` and `git status --short`, both empty).

## Pressure point 1: is "one shared budget" actually required, or just tidier?

Judged the argument first: the two recursions (grouping parens through
`subterm`, calls/collection access through `arg_list`) differ in code path
and in native stack cost per level (parens' native cliff is 88,800-89,000,
calls' is 91,948-92,337 -- calls cost *less* per level on this thread and
build), but a single stack is what both draw against. Two independent
50,000-deep counters would let a program alternate them, `f((f((...))))`,
past 100,000 real levels of recursion -- comfortably past **both** measured
native cliffs -- and abort exactly as before any counter existed. The
report's own reasoning is correct, and using the *shallower* of the two
native cliffs (parens') as the safety margin for a single shared limit is
the conservative, sound choice: it does not require knowing the relative
per-level costs precisely, which is fragile across compiler versions and
platforms, and it bounds the worst case regardless of the mix.

Then I built the actual alternative in a scratch copy, not just the
"guard removed" mutant, because "two counters" and "no counter for one of
them" are different designs and the plan asked which was defensible against
the *other design*, not against nothing. Added a second field
(`call_depth`), had `arg_list` check/increment/decrement it independently
of `subterm`'s existing `expr_depth`, each capped at 50,000 -- a faithful,
compiling "naive" implementation of the alternative, not a strawman:

```
running 6 tests
test a_shallow_paren_nesting_still_parses_on_a_default_stack_thread ... ok
test a_chain_past_the_pre_fix_cliff_parses_and_drops ... ok
test a_paren_nesting_past_the_native_cliff_raises_11_1_instead_of_aborting ... ok
test the_depth_the_oracle_still_answers_also_parses_and_drops ... ok
test a_call_nesting_past_the_native_cliff_raises_11_1_instead_of_aborting ... ok
test parens_and_calls_share_one_budget_rather_than_one_each ... FAILED
```

Exactly as the plan predicted: a per-construct-alone test (including the
task's own new call-nesting test) passes under two separate 50,000 budgets,
because each construct alone still exceeds *its own* budget. Only the
interleaved test, half the budget in each construct, fails -- with separate
counters, half-plus-100 of each stays under both individual caps, so the
whole thing parses and `expect_err` panics. `parens_and_calls_share_one_
budget_rather_than_one_each` is exactly the test that discriminates the two
designs, and no other test in the file does. Design judgement: **correct**,
and now demonstrated against the specific alternative rather than asserted.

## Pressure point 2: mutation verification, reproduced

Scratch copy of the tree (`git archive ae7e8bce`, plus a symlinked
`interpreter/` so `rexx-inventory`'s build script resolves its relative
path), `arg_list`'s guard replaced with a bare call to `arg_list_inner`
(guard removed entirely, the report's own mutant). Built clean (`cargo
build -p rexx-parse --tests`, exit 0), then reproduced each claim:

- The call test alone: **aborts the binary** -- `thread '<unknown>' has
  overflowed its stack`, signal 6, cargo reports `exit 101`. Confirmed.
- The shared-budget test alone: **fails outright**, not aborts --
  `expect_err` panics because with the guard gone, only the grouping-paren
  arm's half of the nesting is ever counted, so the whole expression
  parses. Confirmed, and distinct in kind from the call test's failure,
  which the report's own wording preserves ("aborts the binary" vs. "fails
  outright") -- a real, checked distinction, not sloppy language.
- The four pre-existing tests, run together with the two new ones in one
  binary: all four print `ok` before the process dies on the aborting one.
  Confirmed they are not riding on anything the mutation broke.

**Is an aborting test distinguishable from an infrastructure failure in
CI?** Partially, and it's worth being honest about the gap. The process
does print an identifiable message (`has overflowed its stack`) and cargo
names the exact binary, so a human reading full output can always tell the
two apart. But a CI system gating purely on exit code, without parsing
stderr text, cannot: a SIGABRT from this cause and an unrelated runner
crash both surface as "the job failed," and my own reproduction shows a
second, concrete cost -- once one test in a binary aborts, *every other
test in that binary stops being reported*, which is strictly worse
diagnostics than a normal failing assertion (which lets its siblings keep
running and reports its own name and message). This is not a defect in
Task 3d: the shipped code, with the fix in place, never aborts, so this
risk only exists for a *future* regression that reintroduces the guard's
removal, and an unmistakable, if noisy, CI failure is arguably the right
outcome for that. But it is a real, structural limitation of testing a
stack-overflow-shaped defect by reintroducing it, worth recording rather
than treating the mutation's "success" as free of downsides.

## Pressure point 3: the cliffs

**Oracle's call cliff**, re-run wrapped (`ulimit -v 1048576`), 3 runs each:

```
n=34,293: 213 213 213   (parses, 43.1 at runtime -- "F" is not a routine)
n=34,500: 213 213 213
n=34,760: 245 245 245   (Error 11.1, "Insufficient control stack space")
```

Confirms `[34,500, 34,760]`, sharp on both sides at this resolution, unlike
the paren cliff's noisy middle -- matches the report exactly, and matches
the direction team-lead flagged: the oracle's call cliff (34.6k) is
*lower* than its own paren cliff (39.9k), while this parser's call cliff is
*higher* than its own paren cliff (92k vs 89k). The two implementations
order the constructs oppositely, so neither "calls cost more" nor "calls
cost less" is a safe intuition to carry to the next recursion found.

**This parser's own native cliffs**, reproduced from the actual historical
states rather than by disabling the shipped counter in place (which,
tried first, gave a *different*, more pessimistic number -- see the aside
below):

- Parens: extracted `expr.rs`/`examples/depth_probe.rs` at `0f33843a`
  (Task 3b's commit, before any counter existed at all) via `git archive`,
  built, ran `paren_sized`: **88,800 parses, 89,000 aborts** -- exact match.
- Calls: extracted the same two files at `6285d98f` (Task 3c's commit --
  paren counter exists, `arg_list` still has none at all), added a
  `nested_calls_sized` mode mirroring `paren_sized`'s thread-size wrapper
  (that mode did not exist yet at this commit), built, ran it: **91,948
  parses, 92,337 aborts** -- exact match, at the precise integers the
  report names, not just within the stated bisection width.

Aside, because it explains a real trap and I want it on record rather than
silently discarded: my first attempt at this measurement didn't extract
old commits, it just raised `MAX_EXPR_DEPTH` to 200,000 in the *current*
tree and disabled the `const` assertion, on the theory that a high enough
ceiling would let native recursion reach the true cliff with the counter
"out of the way." That measured **both** cliffs several hundred to over a
thousand levels *shallower* than the reported figures (parens failing
already at 88,700; calls failing already at 91,900). The counter's own
bookkeeping (`expr_depth`'s field access, compare and increment/decrement,
now shared and hit by both constructs) costs real stack on every level,
so a "raise the ceiling" measurement on the shipped code answers a
different question -- "how deep with the current overhead" -- than "what
was the cliff with zero counter at all," which is what the reported
figures are and what a scratch extraction of the pre-counter commits
correctly answers. This is the same trap M1 names for the *default*-thread
numbers, showing up again on the sized thread when reproduced carelessly;
noting it here because the report does not need to say it (its own
figures are right) but the next person re-measuring on this file should
know the naive method is not neutral.

## Pressure point 4: `MEASURED_NATIVE_CLIFF` and the const assertion

Raised `MAX_EXPR_DEPTH` to 90,000 (above `MEASURED_NATIVE_CLIFF = 88_800`)
in a scratch copy: `cargo build -p rexx-parse` fails at compile time,
`error[E0080]: evaluation panicked: MAX_EXPR_DEPTH must stay below the
measured native cliff...`, exit 101. Confirmed the assertion actually
fires when it should, not just that it typechecks.

## Pressure point 5: M1/M2, folded in and re-measured on the final code

Independently re-measured on the actual shipped binary (not copied from
either report):

```
paren_default:    331 parses, 332 aborts
nested_calls:     341 parses, 342 aborts
```

Both exact matches to the report's M1/M2 corrections. Confirmed the three
in-tree sites all carry these final numbers (`grep` for 331/332/341/342
across `tests/deep.rs`, `examples/depth_probe.rs`, `src/expr.rs`) and that
none of them still assert 337/338 or 349/350 as current fact -- the old
numbers appear only inside explicit "was X before" clauses. Confirmed
Task 3c's own report file (`.superpowers/sdd/.../task-3c-report.md`,
untracked by git -- `.superpowers/` is gitignored, so there is no commit
history to check, only file content) still reads 337/338 and 350/360
verbatim: it was not touched, exactly as the report claims and for the
stated reason.

## Additional checks I made on my own, beyond what was asked

- **The two "only in a review file" facts, spot-checked.** Counted raw
  paren/bracket nesting across every `.rex` file in `rust/corpus/` and
  `rust/corpus-l1/`: 12,103 files (matches the report's count exactly),
  deepest nesting found: **5** (matches exactly). And `select` immediately
  followed by `otherwise` (no `when`), wrapped against the oracle: `Error
  7.1: SELECT on line 1 requires WHEN`, raised at the very first `select`
  clause -- confirms a nesting probe using that shape would report a clean
  parse error at every depth without ever building any nesting, exactly as
  claimed.
- **Whether the shared counter can be bypassed by a different `Parser::new`
  call site.** `expr_depth` lives on `Parser` and resets to 0 whenever a
  fresh `Parser` is built; if some free function build a new one and got
  called *from inside* an expression already being recursed into, the
  counter would be silently reset mid-nesting. Grepped every `Parser::new`
  call site in `expr.rs`: all are either the module's own free-standing
  entry points (`parse_expr`, `parse_arg_list`, `parse_message_term`, etc.)
  or the trial-cursor branch inside `parse_variable_or_message_term`. None
  of `subterm`, `message_subterm`, `arg_list`/`arg_list_inner`, `cascade`
  or `collection_message` -- the methods that actually recurse -- ever call
  `Parser::new`; they all use `self.xxx(...)`. And the one free function
  that reaches expression parsing from inside an instruction
  (`parse_arg_list`, for `CALL name arg, arg`) has exactly one call site,
  in `instruction.rs`, at clause level. So the doc comment's "checked
  rather than assumed" claim about this is correct: nothing resets
  `expr_depth` partway down a real nesting.
- **The `parse_constant_expression` off-by-one, traced rather than taken on
  faith.** Its own `TokenKind::LeftParen` arm calls
  `parser.full_subexpression(...)` directly, bypassing `subterm`'s counted
  arm for that one outermost paren -- confirmed by reading the function.
  Anything nested inside that outer paren re-enters through ordinary
  `subexpression`/`subterm`, which *is* counted, so this cannot compound:
  it is a flat, one-time +1 for `RAISE`/`FORWARD`/`USE ARG`/`ADDRESS WITH`,
  exactly as documented, not an unbounded second bypass.
- **No literal em-dash characters** in the diff (`grep -P '\x{2014}'`
  against `git show`, zero hits) -- the project's `--` convention is
  followed throughout the new prose.
- **Full verification suite, unpiped, read directly from `$?`:**
  - `cargo test -p rexx-parse --no-fail-fast`: exit 0. Summed every
    `test result:` line myself: **398 passed, 0 failed** -- matches the
    report's count exactly, digit for digit.
  - `cargo test --workspace --no-fail-fast` (no `--exclude`, since the
    workspace currently builds `rexx-exec` cleanly -- other agents'
    in-flight work has since made it compile again): exit 0, every binary
    `0 failed`.
  - `cargo clippy -p rexx-parse --all-targets -- -D warnings`: exit 0.
  - `cargo clippy --workspace --all-targets -- -D warnings`: **exit 101**,
    on unrelated dead code in `rexx-exec/src/stem.rs` (methods never
    called yet). Not a Task 3d defect: `cargo clippy --workspace --exclude
    rexx-exec --all-targets -- -D warnings` is clean (exit 0), and
    `rexx-exec` is explicitly called out as having live work in it. The
    report's own claim of a clean workspace-wide clippy was presumably
    true at commit time; it no longer is, for reasons outside this task's
    diff.
  - `cargo fmt -p rexx-parse -- --check`: exit 0.

## Code quality notes

- **Minor, non-blocking:** a handful of the new doc-comment sentences join
  two independent clauses with a semicolon rather than two sentences or
  the project's own `--` convention (e.g. "...existed; `MAX_EXPR_DEPTH`
  ... now stops the recursion..."). Consistent with some existing prose
  elsewhere in this crate, and none of the instances found substitute for
  a genuine violation (a literal quoted oracle message with a semicolon in
  it, "Insufficient control stack space; cannot continue execution.", is
  correctly left as a verbatim quotation, not restructured).
- The `arg_list`/`arg_list_inner` split, and decrementing before the
  caller's `?`, mirrors the existing paren-arm pattern exactly rather than
  inventing a second shape for the same idea -- good consistency.
- `MEASURED_NATIVE_CLIFF` plus a `const _: () = assert!(...)` turning "keep
  A below B" from a comment's promise into a compiler-enforced one is a
  good piece of engineering, and I verified it actually fires (Pressure
  point 4).
- `pub use expr::MAX_EXPR_DEPTH` in `lib.rs` is a small, well-justified
  surface widening: it exists so `tests/deep.rs` can compute half the
  budget without a second hardcoded `50_000` drifting from the real one,
  not for its own sake.
- The prefix-chain gap is left correctly scoped: named once, as *the*
  remaining gap (not paired with the now-closed call gap), with the two
  reasons it wasn't folded into this task (doesn't reach the sized path;
  needs its own oracle cliff and a second, different check site) stated
  rather than implied.

## Files touched by ae7e8bce

`rust/crates/rexx-parse/src/expr.rs`, `src/lib.rs`, `tests/deep.rs`,
`examples/depth_probe.rs` -- exactly the four the plan named (plus
`lib.rs`, justified above), confirmed via `git show ae7e8bce --stat`.
Nothing under `rexx-exec/`, `rexx-extract/` or `rust/corpus/` touched.

## Scratch cleanup

All scratch work lives under this session's scratchpad
(`/tmp/claude-1000/.../scratchpad/review3d/`), including two `git archive`
extractions and one full tree copy with mutations applied. No commits, no
branches, no worktrees created against the actual repository.

**One mistake made and corrected, stated rather than hidden:** an `ln -sf`
meant to (re)point a scratch symlink at the real `interpreter/` directory
instead landed *inside* it, because the scratch target was itself already
a symlink to that directory and `ln -sf` follows an existing directory
target rather than replacing it -- this created
`interpreter/interpreter -> interpreter` in the actual repository. Caught
by a final `git status` before writing this verdict, and removed
immediately (`rm interpreter/interpreter`). `git status` on the real
repository now shows only other agents' in-flight `rexx-exec` files,
nothing from this review.
