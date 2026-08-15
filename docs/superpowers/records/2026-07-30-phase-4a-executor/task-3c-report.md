# Task 3c report: a depth counter on the parser's paren recursion

Status: DONE

## Step 1: pin the oracle's cliff, and measure our own on both thread sizes

Oracle (`build/bin/rexx`, wrapped `( ulimit -v 1048576; ... )`), `say
((((...'a'...))))` with N parens, bisected below the brief's 38,000-40,000
bracket:

- N <= 39,900: stable rc 0 across 3 runs.
- N in [39,910, 39,940]: **non-deterministic**, rc 0 or rc 245 (Error 11.1,
  "Insufficient control stack space; cannot continue execution.") on
  different runs of the identical file. This is real jitter (presumably
  ASLR-driven stack-layout variance in the oracle's own C++ recursion), not
  a measurement mistake -- confirmed by rerunning several points 3x each.
- N >= 39,950: stable rc 245 across 3 runs.

So the oracle's cliff is a **bracket with a noisy middle**, [39,900, 39,950],
not a single integer -- consistent with the brief's warning that a prior
task recorded a per-level figure that was an artifact of too wide a bracket;
this one is deliberately reported as a range rather than rounded to a point.

Our own parser, no counter yet, `examples/depth_probe.rs`'s new
`paren_default`/`paren_sized` modes (parse the same `say (((...)))'a'...`
shape, thread with no explicit `stack_size` vs. an explicit 512 MiB one):

- `paren_sized` (512 MiB, debug, this machine): stable success through
  88,800, stable native stack overflow (SIGABRT, no message) from 89,000.
  Matches the spec's own "measured at 90,000" figure closely enough to be
  the same phenomenon on a different machine/rustc build.
- `paren_default` (no explicit `stack_size`, i.e. what `cargo test` gives a
  test): stable success at 337, stable native stack overflow at 338.
  **Two orders of magnitude shallower than the oracle's own cliff**, and
  this is the number the brief asked me to find: it decides whether any
  existing `rexx-parse` test is near it (checked: none is; the deepest
  paren nesting in the corpus and this crate's own tests is nowhere close
  to 337) and it decides what the counter can and cannot protect -- see
  Step 3.

## Step 2 & 3: the counter

Added `MAX_PAREN_DEPTH: u32 = 50_000` and a `paren_depth: u32` field on
`expr.rs`'s `Parser`, incremented/checked/decremented around the single
recursive call in `subterm`'s `TokenKind::LeftParen` arm
(`self.full_subexpression(Terminators::RIGHT)`), raising `self.error(11, 1)`
before descending past the limit rather than after. Chosen inside the
oracle's own reporting range (roughly 25% above its measured [39,900,
39,950] cliff) and comfortably below our own sized-thread native cliff
([88,800, 89,000], leaving a ~40% margin). No `unsafe`; `error.rs` needed no
change at all, since `11.1`'s text ("Insufficient control stack space;
cannot continue execution.") is already in `rexx-inventory`'s generated
table and `ParseError`'s existing message lookup is generic over the code.

Verified directly with `depth_probe`:
- `paren_sized` at 49,999/50,000: parses. At 50,001/100,000/500,000: raises
  `11.1` cleanly (previously all three aborted natively past 89,000).
- `paren_default` at 49,999 through 500,000: still aborts natively, exactly
  as Step 1 predicts -- the counter's limit (50,000) is far above this
  parser's own default-thread native cliff (338), so on a small thread the
  process dies before the check is ever reached at any depth the counter
  would care about. This is a real, stated limitation (see `MAX_PAREN_DEPTH`
  and `paren_default`'s doc comments, and `tests/deep.rs`'s
  `a_shallow_paren_nesting_still_parses_on_a_default_stack_thread`), not an
  oversight: the counter protects a **sized** caller (what D19 gives
  `rexx-exec`'s public entry point), and nothing in this crate promises
  more to a caller who parses on an unsized thread.

**Exact parity is not achievable and is not attempted.** Between roughly
40,000 and 50,000 parens, the oracle already raises 11.1 and this parser
still succeeds -- an acknowledged, unavoidable divergence, stated in
`MAX_PAREN_DEPTH`'s doc comment, because the two cliffs are stack artifacts
of two unrelated implementations more than 2x apart.

`tests/deep.rs` gained two tests: `a_paren_nesting_past_the_native_cliff_
raises_11_1_instead_of_aborting` (100,000 parens, on an explicit 512 MiB
thread matching D19's sized entry point, asserts `(err.code, err.sub) ==
(11, 1)`; previously aborted, now clean) and `a_shallow_paren_nesting_
still_parses_on_a_default_stack_thread` (300 parens, safely below the
337/338 default-thread cliff, documenting rather than closing the gap
above).

## Step 4: the other recursive descents

Checked, using `depth_probe`'s new `prefix_chain`/`nested_calls`/
`nested_do` modes (default 2 MiB thread unless noted):

- **Nested `DO`/`END` blocks**: read `block.rs::translate_block` first --
  it is a single flat `loop`, with open blocks tracked on a
  heap-allocated `Vec<Frame>` (`Block::control`), never recursion per
  nesting level. Then measured rather than trusting the reading alone:
  `do` nested 100,000 deep, one `nop`, 100,000 matching `end`s, parses
  cleanly. **Not exploitable**, and structurally can't be: nothing in
  `translate_block` calls itself.
- **`SELECT`/`WHEN` nesting**: not measured directly (no probe mode
  written for it), but reaches the exact same `translate_block` loop and
  the same `Vec<Frame>` control stack as `DO`, via the same `Control::
  Select`/`SelectCase` frame kinds read while writing Task 3b's report on
  `block.rs`. Expected safe for the same structural reason; **flagged as
  read-but-not-measured** rather than left unstated.
- **Prefix operator chains** (`- - - - ...1`, recursing in
  `message_subterm`, which calls itself directly for the operand and never
  passes through `subterm`'s counter at all): aborts natively between
  1,150 and 1,200 on a default thread. **Not fixed.**
- **Nested function/message calls** (`f(f(f(...'a'...)))`, recursing
  through `subterm`'s `Symbol` arm into `arg_list` and back through the
  full expression chain, a different re-entry path than the grouping-paren
  arm the counter guards): aborts natively between 350 and 360 on a
  default thread -- **shallower than plain grouping parens**, and also not
  fixed.

Both unfixed exposures are real and measured, not fixed here: this task's
scope, both in the plan's task title ("a depth counter on the parser's
subexpression recursion") and in D19's own measured table, is the
grouping-parenthesis recursion specifically, the one construct the oracle
itself reports a condition for rather than crashing. Fixing the other two
would need their own oracle measurements (neither construct's oracle cliff
is known) and, for the prefix-chain case, a second counter in a different
function (`message_subterm`, which never reaches `subterm`'s check for a
pure prefix-operator chain). Recording both as known gaps of the same
shape rather than silently patching or silently dropping them, per the
same discipline Task 3b's report used for `Debug`/`PartialEq`/`Clone`.

## Verification (Step 5)

- `cargo test -p rexx-parse --no-fail-fast`: every binary green, 254 unit
  tests + all integration binaries, including `tests/deep.rs` at 4 tests
  (2 from Task 3b, 2 new).
- `cargo test --workspace --exclude rexx-exec --no-fail-fast`: every binary
  green, 0 failed. `rexx-exec` excluded per the coordination note (two
  other agents' work in flight there); not touched.
- `cargo clippy -p rexx-parse --all-targets -- -D warnings` and
  `cargo clippy --workspace --exclude rexx-exec --all-targets -- -D
  warnings`: both clean.
- `cargo fmt -p rexx-parse --check`: clean (one reformat applied to
  `depth_probe.rs`'s `paren_default`/`paren_sized` match arm before this).

## Files touched

- `rust/crates/rexx-parse/src/expr.rs`: `MAX_PAREN_DEPTH` constant,
  `Parser::paren_depth` field, the check/increment/decrement around
  `subterm`'s grouping-paren recursion. `error.rs` needed no change.
- `rust/crates/rexx-parse/tests/deep.rs`: two new tests plus a module-doc
  paragraph, per above.
- `rust/crates/rexx-parse/examples/depth_probe.rs`: five new modes
  (`paren_default`, `paren_sized`, `prefix_chain`, `nested_calls`,
  `nested_do`) and an extended module doc comment recording every cliff
  measured for this task.

Not touched: `rust/crates/rexx-exec/`, `rust/corpus/`, anything under
`error.rs` (considered, not needed).

## Commit

`6285d98f`, "Add a depth counter on the parser's parenthesis recursion,
raising 11.1". Staged and committed exactly the three files above. `git
status` after the commit still shows other agents' in-progress files under
`rexx-exec/` and `rexx-extract/`, untouched by me.
