# The inclusive category rollup, ours against the oracle

Produce one table: where each interpreter's instructions go per clause on the
same program, by **work category**, both sides, summing to 100% on each side.

## Why this and not the file rollup already taken

A rollup by source file was taken 2026-09-21 and it is not trustworthy as a
comparison. We build `-O3` with fat LTO and one codegen unit; the oracle is
`-O2 -g -DNDEBUG` in a shared library. Inlining therefore moves cost between
files differently on the two sides, and a stdlib file inlined into a hot
function is charged to that file rather than to the work it does.

**Self cost summed per function partitions the program exactly**, and inlined
code is charged to the function it was inlined into, which is the right unit
here: the oracle's equivalent raw-pointer code also sits inside its functions.
So categorise **functions**, not files, and check that your columns sum to each
run's own `summary:` line.

## The data

Both dumps exist. Re-take them if you prefer, but say so.

* ours: `<scratch>/bc-analysis/cg-instr.out`, whole program 20,291,841,264,
  binary `.text` `81d2f383dcc0270c45ce1644895fc3b838f9577e7cc7e0cfaef8d24a7d680532`
* oracle: `<scratch>/reprofile-2026-09-21/cg-oracle-rexxcps.out`, whole program
  10,688,849,215, run as
  `( ulimit -v 8388608; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib valgrind --tool=callgrind /home/moritz/dev/repos/ooRexx/build/bin/rexx <pinned rexxcps> )`

Both ran the pinned `rust/bench-rexxcps/rexxcps.rex`, both printed the same
`200 x 100 iterations of 1000 clauses`, so both did **20,000,000 clauses** and
per-clause figures are directly comparable.

## The categories

Start from these and change them if the code argues for it, saying why:

* **interpretation machinery** -- deciding what runs next and running the small
  things inline. Ours: `run_ops_from` and its instantiations. Theirs:
  `RexxActivation::run`, the `RexxInstruction*::execute` family, the
  `RexxExpression*::evaluate` family.
* **variable access, simple** and **variable access, compound** kept apart.
* **arithmetic and numeric comparison**
* **string and number conversion**
* **PARSE**
* **built-in functions**
* **call and argument handling**
* **allocation and collection, the interpreter's own**
* **the C allocator** -- `malloc`, `free` and their internals
* **libc string and memory** -- `memcpy`, `memset`, `memcmp`
* **everything else**, which must be small or the categories are wrong

## What the table must show

Per category, per side: instructions, share of that side's run, and
**instructions per clause**. Then the per-clause difference, which is where the
480.2 Ir/clause gap lives.

**Name what is in "everything else" if it exceeds about 5%** on either side.

## Two asymmetries to state, not to correct for

* The oracle is a **shared library**, so its cross-library calls go through the
  PLT and it gets no cross-translation-unit inlining. We are a static binary at
  `-O3` with fat LTO and one codegen unit. **Both of those favour us**, so the
  measured gap understates the structural one rather than overstating it.
* `build/` is `-O2`. No ratio taken from it is a sanctioned figure. Say so
  beside any ratio you quote.

## Three findings to check rather than inherit

Each was measured on 2026-09-21 and each belongs in your table if it is right:

1. **The oracle calls the C allocator 286 instructions' worth in the whole run.**
   Ours spends 868,687,634, about 43.4 per clause. Its own `newObject` family
   across seven files is about 71 per clause, against our roughly 77 all in, so
   allocation **volume** looks like a wash and the difference is where it is
   paid. Confirm or correct both halves.
2. **Bounds checking is 0.99% of our run**, measured by finding every branch
   guarding a bounds panic and pricing it per address. That is not a category
   below; it is a slice through several. Do not double count it.
3. `roots.rs`, the register file, is 578,375,148 or 28.9 per clause. The
   oracle's `ExpressionStack` push and pop are raw pointer moves that inline to
   nearly nothing.

## Constraints

* **Do not edit any file in the repository** and do not commit. You have a
  worktree; work there.
* The suite does not pass in a fresh worktree, which is known and recorded. You
  do not need it.
* Oracle runs wrapped as above, from a fresh empty directory you `mkdir`. Read
  `rust/corpus/oracle-crashes.txt` first and never run its entries.
* `grep` here skips binary and ignored files; use `/bin/grep -a` for any count.
* Do not wait on a `pgrep -f` for your own jobs: the waiting shell's argv
  contains the pattern, so the count never reaches zero. Append `finished` to a
  status file and wait on that.

## Report

`.superpowers/sdd/2026-09-22-category-rollup.md`, or `.txt` if your harness
refuses `.md`. **Replies truncate at about 8 KB**: send the table first,
caveats second, everything else last. Quote the command beside every figure and
give the exit status you observed.
