# Item 7: assert the bound before indexing

Against BASE `0e9c9e062`, whose code is `5e765dc5a`.

## Headline

One of the three candidates built is worth keeping. It removes the driver's
per-op bounds check by cutting the op stream to the bound the loop already
tests against, and it measures **-0.864% on `varlookup`, -0.750% on
`emptyloop` and -0.079% on `rexxcps`** in retired instructions.

The other two were built and measured and are **not** in the tree: one bought
nothing on the narrow axes and cost 0.055% on `rexxcps`, the other cost 2.015%
on `emptyloop`.

**The file-level attribution that sized this item does not predict any of
that, and on one axis it points the wrong way.** `core/src/slice/index.rs`
attributed to `run_ops_from::<true>` on `varlookup` **rose** from 817,003,219
(4.65%) to 950,003,537 (5.45%) across the change that cut the whole program by
0.864%. Sizing a further site from that file's share is not available.

## Gates

Run over the committed tree at `26ccef4ee70e5071c06c17a296abe3efc1d3a524`,
unpiped, each status written as it landed, statuses at
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/bounds7/gates.txt`,
whose first line is that sha. Tallies summed from each log's own
`test result:` lines.

| gate | status |
|---|---|
| `cargo fmt --all --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `cargo build --workspace --all-targets --release` | 0 |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release --no-fail-fast` | **101 first, 0 on re-run** |
| `cargo build --workspace --all-targets` | 0 |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | 0 |

Release tests, first run: 133 binaries, 2650 passed / 1 failed / 4 ignored.
Release tests, re-run of the same command over the same commit: 133 binaries,
**2651 passed / 0 failed / 4 ignored**, the expected tally. Debug tests: 133
binaries, **2652 passed / 0 failed / 4 ignored**, the expected tally, first
run.

**The one failure is an oracle flake and not this change.**
`introspection_arity::every_unstable_row_is_really_unstable` asserts that a
row marked `UNSTABLE:` really does not reproduce; it does that by running the
**oracle** twice on a probe and comparing, through
`arity::stable_rows_marked_unstable`, which calls `oracle_command` and nothing
else. `Object~hashCode` answered the same bytes on both of its runs, so the
test reported the marker as wrong. This crate's interpreter is not invoked by
that path at all, and the same binary passed in the debug sweep of the same
commit. Twenty isolated re-runs of that one test: 0 failures. It is worth
someone's attention as a flaky instrument; it is not evidence about item 7.

Fast checks run over this tree before the commit, each status read unpiped
from its own run:

* `cargo fmt --all --check` -- exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` from an **empty**
  `CARGO_TARGET_DIR` -- exit 0, 74 crates checked, `rexx-exec` among them. The
  warm run in the working target finished in 2.14s and is provisional by
  `rust/CLAUDE.md`'s rule; this is the cold one it asks for.
* `REXX_CORPUS_GATE=1 memcap 8G cargo test -p rexx-exec --no-fail-fast`
  (debug) -- exit 0, 49 test binaries, **1578 passed, 0 failed, 1 ignored**,
  summed from each binary's own `test result:` line. This is the crate the
  change is in and the one carrying `corpus`, `ir_dual` and the `debug_assert`
  tripwires that a `--release` gate compiles out.

## The change

`crates/rexx-exec/src/ir/drive.rs`, `Interp::run_ops_from`, and
`crates/rexx-exec/src/ir.rs`'s `Chunk`.

The driver's loop reads one op per iteration through `Chunk::op_at_index`,
which is `self.ops.get(at as usize)` -- a bounds check and an `Option` per op.
The loop's first statement is `if pc >= stop { ... }`, so every read below it
already has `pc < stop`. What was missing was any link between `stop` and the
stream's own length.

`Chunk::ops_upto(stop)` supplies it: it answers `self.ops.get(..stop as
usize)`, so the slice it returns is `stop` long, and `pc < stop` is `pc <
stream.len()`. The per-op read becomes `&stream[pc as usize]` and the compiler
has the bound.

**What establishes it.** The loop guard, which returns or re-enters at the top
for every path that changes `pc`, and the slice's own length. Nothing weaker
is asserted: the entry read refuses exactly the chunks the per-op read used to
refuse, a `stop` past the end of `ops`, and it refuses them with the same
`Loud::chunk_map_too_short`. It is raised earlier -- at entry rather than at
the op that would have run off the end -- so on a malformed chunk it can now
fire where the old code would have returned first. In a well-formed chunk it
is unreachable either way: `Chunk::op_of` carries one entry per instruction
plus a final entry at `ops.len()`, so `op_at(end)` never exceeds `ops.len()`.
Neither spelling can panic.

The `debug_assert` above the read moves to `stream.get(pc as usize)`, keeping
its `Option` form and its meaning.

## Measurement

`valgrind --tool=callgrind`, whole-program `Ir` as callgrind reports it under
`Collected`, two interleaved rounds, every build in its own
`CARGO_TARGET_DIR`. Every run exited 0.

    valgrind --tool=callgrind --callgrind-out-file=<out> <build>/rexx-run <program>

run from `rust/`. Binaries:

| build | `rexx-run` sha256 |
|---|---|
| BASE | `7ae009e5d931865fd25ca50273f26debd56d34824827762547e05929b66095e8` |
| the change | `7782eb9e75bbae3ad6ad4aaafaeae37896b199a03a685e8a986b5d2d0d1b6366` |

The committed tree was rebuilt in a third target directory and its `.text`
section is byte-identical to the measured binary's
(`objcopy -O binary --only-section=.text`, both
`81d2f383dcc0270c45ce1644895fc3b838f9577e7cc7e0cfaef8d24a7d680532`). The two
whole-file sha256s differ, because `[profile.release]` sets `debug = true` and
the target directory's path is in the debug info.

| program | BASE r1 | BASE r2 | change r1 | change r2 | delta |
|---|---|---|---|---|---|
| `rexxcps.rex` | 20,307,371,712 | 20,309,351,216 | 20,291,880,753 | 20,292,580,419 | **-0.0794%** |
| `emptyloop.rex` | 9,998,584,190 | 9,998,580,791 | 9,923,589,894 | 9,923,584,600 | **-0.7501%** |
| `varlookup.rex` | 17,584,579,592 | 17,584,602,232 | 17,432,602,826 | 17,432,605,566 | **-0.8643%** |

Within-build spread: `emptyloop` 3,399 and 5,294 (0.00005%), `varlookup`
22,640 and 2,740 (0.0001%), `rexxcps` 1,979,504 and 699,666 (0.0097%).
`rexxcps` is the noisy one because it renders `TIME()` into its own output;
its two builds' ranges do not overlap.

`emptyloop` and `varlookup` were named as axes that should move, and they
moved by ten times what `rexxcps` did. The change is one instruction pair per
driver loop iteration, and those two programs spend most of their instructions
in that loop while `rexxcps` spends most of its inside the ops.

Four further BASE runs and two further change runs of `rexxcps` were taken
while the design was being settled -- BASE 20,308,496,831 and 20,308,664,166,
the change 20,294,387,082 and 20,291,879,661 -- and agree with the table. The
`varlookup` and `emptyloop` figures for the change reproduced in a second,
separately launched job (17,432,606,812 and 9,923,583,446).

## The two candidates that were built and reverted

Both were complete, compiled clean, and passed
`cargo test -p rexx-exec --no-fail-fast` under `REXX_CORPUS_GATE=1` in debug
(1578 passed, 0 failed, 1 ignored) before being measured. Neither is in the
tree.

### The clause position table

`Chunk::position_at(index)` is `self.positions.get(index).copied()`, one
bounds check per clause, on the same index that `code.body.instructions.get`
has just been checked against one statement earlier. `clause_positions` makes
that table either `body.instructions.len()` long or empty, so cutting it to
`instructions.len()` once at entry makes the second check provable from the
first.

Measured: **+0.0548% on `rexxcps`** (20,304,434,381 and 20,302,248,741 against
the change's 20,291,880,753 and 20,292,580,419), **-0.00001% on `emptyloop`**,
**+0.000006% on `varlookup`**. Removing a bounds check per clause made the
program slower on the axis it was sized on and did nothing at all on the two
axes that had just moved by 0.75% and 0.86%.

A second spelling was built, reading the position at its use rather than
beside the instruction lookup, in case the first had extended a live range
across the clause entry. `rexxcps` 20,299,139,962 and 20,296,450,719 against
the change's 20,298,679,824 and 20,292,244,445 in the same interleave: **also
positive**, so the placement is not what cost it.

### The region slice

The region's ops come from `Chunk::ops_in(pc + 1, end)`, a `Range` `get` that
tests both `pc + 1 <= end` and `end <= ops.len()`. Cutting them from `stream`
instead -- `&stream[pc as usize + 1..]`, which is free under the loop guard,
then a `RangeTo` `get` for the region's own length -- replaces two checks with
one.

Measured, one round: **+2.015% on `emptyloop`** (10,123,592,975 against
9,923,583,446) and **+0.654% on `varlookup`** (17,546,607,330 against
17,432,606,812). The re-slice costs more than the check it removes. Round two
was cancelled and the candidate reverted.

## The remainder, re-derived

`callgrind_annotate --threshold=100`, by file summed over every site it was
inlined into. The sites the brief named, at BASE on `rexxcps`, reproduced
here: `index.rs` in `run_ops_from::<true>` 287,039,771, in `exec_parse`
89,040,030, in `run_ops_from::<true>'2` 35,000,429. The second and third are
untouched by this change.

**No further site is sized from those numbers, and here is why.** On
`index.rs` cost attributed to `run_ops_from::<true>` went from 817,003,219
(4.65% of BASE) to 950,003,537 (5.45% of the change) while the whole program
fell by 151,986,716. The attribution moved the wrong way by 133 million
instructions across a change that removed indexing and nothing else. The same
run's `run_ops_from` self cost fell by 152,000,241, which is the whole program
delta, so the change is localised where it was made; it is the split of that
function's cost across source files that is not to be trusted.

What is left, stated as work rather than as a figure:

* **`exec_parse`.** `index.rs` there is 89,040,030 on `rexxcps` at BASE, with
  25,200,006 at each of `index.rs:184` (the scalar `get`'s own comparison) and
  `index.rs:186`. It is untouched by this change, and PARSE was reworked
  earlier today, so its bounds would have to be re-derived against the current
  shape before anything is built.
* **`code.body.instructions.get(index)`**, one per clause in the driver. No
  bound for it is established anywhere: the index arrives inside the op and
  nothing in the chunk records a limit the compiler could carry. Making it
  provable means giving the op a proof, which safe Rust cannot express without
  the check it would remove.
* **`run_ops_from::<false>`** carries 26,880,000 on `rexxcps` at BASE and gets
  the same change for free, since it is the same function.

## Concerns

* **The estimate was 3.86% and the measured recovery at the largest site is
  0.079% on the axis it was sized on.** The brief asked for a fraction and
  said to report what came out. The fraction on `rexxcps` is about one
  fiftieth. On the narrow axes it is about one fifth.
* **The direction of a bounds-check removal is not predictable from the
  profile.** Three candidates were built by one technique at three sites in
  one function: -0.86%, 0.00%, +2.02% on `varlookup`. Every one of them
  removes a check and adds nothing to a per-op path. Two of the three are
  worse than what they replaced. Anything further under this item has to be
  built and A/B'd, and a prediction written down beforehand was wrong in sign
  for two of the three.
* **A prediction was recorded before the second candidate was measured and was
  falsified**, along with the rival hypothesis recorded beside it. Both
  predicted a saving; the measurement was a cost. That file is
  `scratchpad/bounds7/prediction.txt` in this session's scratchpad and is not
  in the repository.
* **The constraint the brief warned about did not bind.** Nothing here adds a
  test to a per-clause or per-op path. The one landed change adds one slice
  `get` per `run_ops_from` entry, and `run_ops_from` is entered 840,009 times
  on `rexxcps` against 28,722,000 clauses, so the entry cost is about one
  thirty-fourth of a per-clause test. The 840,009 is callgrind's own call
  count for the three instantiations; the 28,722,000 is the execution count of
  a one-instruction line inside the `Op::Clause` arm, which several unrelated
  lines in that arm agree with at 1x and 1.5x.
