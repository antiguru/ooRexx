# Item 7: assert the bound before indexing

Item 7 of `.superpowers/sdd/queued/2026-09-20-performance-todo.md`, and the
section "Item 7 sized on `rexxcps` at last" appended to it today. Read both.

## The size, measured today rather than inherited

`callgrind_annotate --threshold=100` over a run of `5e765dc5a`, whole program
20,313,581,507 instructions, each file summed over every site it was inlined
into:

| file | instructions | share |
|---|---|---|
| `core/src/slice/index.rs` | 784,260,164 | **3.86%** |
| `core/src/slice/iter/macros.rs` | 671,356,756 | 3.30% |
| `core/src/num/uint_macros.rs` | 287,088,300 | 1.41% |

The largest `index.rs` sites: **`run_ops_from::<true>` 287,039,771**,
`exec_parse` 89,040,030, `run_ops_from::<true>'2` 35,000,429, the rest spread
thin. Per axis it was already measured at **4.68% of `varlookup`** and **3.29% of
`emptyloop`**, so unlike most items this one has a figure on the narrow axes too.

**`iter/macros.rs` is iterator machinery, not bounds checks. Do not add the two.**
It is listed so you do not size this item by summing them.

**This is an attribution over the whole indexing operation**, address computation
and bounds check together, so what an assertion removes is the check and its
panic path, not the figure. Three separate results today showed an attribution
overstating what a fix recovers. Expect a fraction and report what you get.

## The constraint that decides the whole design

**A conditional in the driver's `Op::Clause` arm costs about 0.5% on `rexxcps`
and about 1.25% on `emptyloop` whatever it does.** That is measured, in five
shapes, and it is why the previous TRACE speculation was reverted at a
measured +0.501% for its guard against a -0.543% saving.

So an assertion added to a per-clause or per-op path **pays that tax on every
program** and has to beat it before it banks anything. The win has to come from
hoisting the bound to where it is established once and letting the compiler
elide the checks under it: once per region entry, once per chunk, once per
template, not once per access.

**If you find yourself adding a test to the hot path to remove a test from the
hot path, stop and report that.** It is the shape that has already cost this
project one reverted commit.

## What is available and what is not

`unsafe` is granted only in `rexx-api/src/ffi.rs`, `src/load.rs` and
`rexx-core/src/bytes.rs`. **`get_unchecked` is not available to you**, and no
part of this task is a reason to widen that grant. The technique here is to make
the bound provable, not to assert it away.

Project memory records two things about this technique: it is worth several
percent on `Index`'s panic path, and **a formatted message on the assertion costs
more than the checks it removes**. Prefer a bare assertion, and measure if you
disagree.

## Ordering, which cost a candidate two tasks ago

Several sites share one cost. A change at `run_ops_from` may remove what a change
at `exec_parse` was sized against, and the order you do them in will then decide
which one gets the credit. That already happened today: a candidate measured
**+0.0324% slower** because its predecessor had taken two of the three indexings
it was sized against.

So: **one commit per site, and re-derive the remaining sites after each.** If you
prefer a different order, say why in the report and give what the second one
would have been worth at the other order.

## How it is judged

**The oracle differential is the arbiter.** `corpus_differential` has been 604 of
604 STRICT on the last three commits; keep it there.

An assertion that can fire turns correct behaviour into a panic. Indexing already
panics out of bounds, so asserting the *same* bound changes nothing observable;
asserting a **stronger** bound than the code guarantees is a crash on valid
input. State for each assertion what establishes it, and prefer the debug gate to
catch a wrong one: `Interp::enter_clause`'s `debug_assert` and the temps-frame
watermark are compiled out of everything a `--release` gate runs.

Gates as `rust/CLAUDE.md` defines them, builds outside the cap, tests under
`memcap 8G`:

    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo build --workspace --all-targets --release
    REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release --no-fail-fast
    cargo build --workspace --all-targets
    REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast

Expected at BASE: 133 binaries, 2651 passed / 0 failed / 4 ignored release,
2652 / 0 / 4 debug. **`memcap 8G` kills a cold release compile** at rc 137 with
`Compiling` as the last line, which a status-only reading calls a red gate. A
clippy finishing in under a second against a warm target is provisional; re-run
it from an empty `CARGO_TARGET_DIR` before quoting it. Sum test tallies from each
log's own `test result:` lines, never from a summary, because `cargo test` stops
at the first failing binary.

Commit before any long run and leave the tree frozen until the status file says
finished.

## Measurement

`valgrind --tool=callgrind` on `rust/bench-rexxcps/rexxcps.rex`, the pinned copy.
Interleaved rounds, own `CARGO_TARGET_DIR` per build, sha256 beside each binary,
within-build spread given. BASE is 20,307,384,766.

**Measure `emptyloop.rex` and `varlookup.rex` too, and here they are not
controls** -- they carry 3.29% and 4.68% of this cost and should move. A program
that should benefit and does not is as informative as one that should not and
does.

## House rules

`rust/CLAUDE.md` governs. Comments minimal: one-sentence overview, then
parameters, returns, panics and non-obvious properties only. **A comment may
never state the size of a set.** No em-dashes. Never drop or re-wrap an existing
comment; inserting an item under a doc block silently reassigns that doc.

No new dependencies. No process-global state. Never amend a commit; a follow-up
commit is the answer.

## Report

Write to `.superpowers/sdd/2026-09-21-bounds-check-report.md`, or the same path
with `.txt` if your harness refuses `.md`, and say which.

**Your replies to me truncate at about 8 KB. Put gate statuses first**, then
commits, then the measurement per site, then the re-derived remainder, then
concerns. Quote the command beside every figure and give the exit status you
observed. If a run is still going when you report, say so rather than describing
what it will say.
