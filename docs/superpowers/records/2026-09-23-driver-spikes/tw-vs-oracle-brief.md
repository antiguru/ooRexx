# Why does the oracle's tree walker need ~40% of our tree walker's instructions?

## The question (Moritz)

Both are tree walkers. On `rexxcps` the oracle runs 534.44 instructions per
clause, our restored tree-walker 1274.02 (with libc), the IR 1010.95
(`docs/superpowers/records/2026-09-23-driver-spikes/tree-walker.md`, category
table). "There shouldn't be any structural reason left." Find the reasons,
concretely, per construct, with the instructions accounted for on both sides.
This is an investigation: no interpreter changes to ship.

## Why the existing category table is not the answer

It compares categories assigned by function-name rules to two different code
bases. The oracle's "interpretation machinery" is 98.29 per clause against our
walker's 584.09, but the oracle's expression evaluation may be filed under
arithmetic or variable access by those rules while ours is filed as
machinery. **Do not argue from that table.** Check how the rollup's rules file
the oracle's `evaluate` methods before citing any category, and prefer the
matched per-construct method below.

## Method: matched constructs, then matched call paths

1. **Per-construct cost.** Write micro-programs, each a counted loop
   (`do i = 1 to N`) around exactly one construct, and an empty-loop control
   with the same N. Cost per construct = (program - control) / N. At least:
   `nop`; `x = y`; `x = 'abc'`; `x = x + 1`; `x = a || b`;
   `if a = b then nop`; `a.i = x` and `x = a.i`; `x = length(y)`;
   `call r` of an internal routine that returns; `parse var s a b c`;
   `x = y * 1.5` under default digits. Plus the empty loop's own
   per-iteration cost. Measure on three engines: the oracle, our tree-walker
   (`REXX_ENGINE=tree-walker`, feature on), our IR (same binary). Callgrind,
   `summary:` minus libc and ld-linux for ours; for the oracle, report both
   with and without libc and say what its libc share is, since the oracle's
   libc use differs (strings, allocation). Check determinism with two rounds.
2. **For the constructs with the largest oracle-to-walker gap (at least four,
   and always `x = x + 1`, `x = y`, the empty loop, and `if a = b`)**, take a
   callgrind inclusive call tree of one program run on each side and align
   them: every function entered per construct execution, with instructions
   per execution. Read the oracle's source (C++ at
   `/home/moritz/dev/repos/ooRexx/` -- `interpreter/instructions/`,
   `interpreter/expression/`, `interpreter/execution/RexxActivation.cpp`) and
   ours, and for each surplus name what our side does that theirs does not,
   with `file:line` on both sides.
3. **Classify each surplus** into one of, or a new class you name:
   * Rust-enforced checks (bounds, overflow, `RefCell`, `Option::expect`);
   * error propagation (`Result` threading vs C++ exceptions);
   * representation (how a value, a string, a number, a variable is stored
     and reached; tagged `ObjRef` vs pointers; small ints; string rendering);
   * missing caching the oracle does (e.g. the oracle's literal/number
     caches, cached variable retrievers, line numbers on the instruction);
   * extra work the oracle does not do at all (rooting, trace/deadline
     checks, bookkeeping), and whether that work is required for our
     semantics or incidental;
   * compiler/codegen (spills, failed inlining), with the evidence.
   Give the per-construct instruction totals per class, so the answer to
   "where do the other 60% go" is a table, not a list.

Prior art, read before starting, do not re-derive:
* project memory `oorexx-oracle-is-a-tree-walker.md` (the oracle's clause loop
  and four differences recorded 2026-08-08; several may since be fixed --
  check each against today's code and say which still hold);
* `docs/superpowers/records/2026-09-20-performance-items/2026-09-22-category-rollup.md`
  and `2026-09-21-bounds-check-share.md` (bounds checks are ~1% of rexxcps).

**Rule the observable, not the mechanism**: every "because" in the report must
rest on something you ran or a line you read and cite. Where you infer, say
"inferred".

## Where you work

* Worktree: pre-authorised to run `git switch -c investigate/tw-vs-oracle
  95d8cd7ed` (the `spike/tree-walker` head) when HEAD is not that commit and
  `git status --short` shows only `.claude/`. Verify HEAD afterwards.
* Build with `--features tree-walker` (check the exact feature and package
  names in `rust/crates/rexx-exec/Cargo.toml`) into your own
  `CARGO_TARGET_DIR` under your scratch:
  `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/round3/tw-vs-oracle/`.
* Binding rules: `../spikes/common.md` beside this directory (standing rules,
  oracle-run discipline, `oracle-crashes.txt`, `/bin/grep -a`, no star-glob
  `rm`, commit messages via `-F` with the trailers named there). The C++ tree
  is read-only. Never set `NUMERIC DIGITS` above 1000. A symbol `x` or `b`
  immediately followed by a quoted string is a hex/binary literal: do not write
  `x'...'` or `b"..."` by accident in micro-programs.
* Do not dispatch subagents.

## Report

Committed to your branch at
`docs/superpowers/records/2026-09-23-driver-spikes/tw-vs-oracle.md`, micro-
programs and scripts beside it in `tw-vs-oracle-files/`, and a copy at
`<scratch>/report.md` if your harness allows it. Reply under 8 KB: the
per-construct table (construct, oracle, walker, IR, walker/oracle), then the
class totals, then the three largest individual causes with `file:line` on
both sides, then what does NOT explain it (things you checked and ruled out),
then concerns. Branch and head commit.
