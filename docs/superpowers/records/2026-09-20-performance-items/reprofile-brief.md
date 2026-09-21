# Regenerate the candidate queue from a fresh profile

Five performance commits landed today and `rexxcps` went from 21,147,005,593
instructions to 20,292,230,586. **Every remaining item on
`.superpowers/sdd/queued/2026-09-20-performance-todo.md` is sized against a base
that no longer exists**, and today produced four separate demonstrations that a
stale or ill-formed attribution misleads.

This task produces the next queue. It is a measurement and a ranked list; it
changes no code.

## What landed, so you can recognise work that has already moved

| commit | what | measured |
|---|---|---|
| `7a28f68e6`, `7fbadfecb` | a per-clause `TRACE` setting from a forward dataflow analysis | zero on `rexxcps`, at its ceiling where the setting is a top-level literal |
| `941b5fb82` | a loop's back edge and zero-trip edge in that analysis | -1.78% on an in-loop literal shape |
| `725aa8863` | `Condition` + `JumpUnless` fused into `ConditionJump` | **-1.6747%** |
| `1da095de8` | a bound-slot store for a plain-variable `PARSE` target | **-1.4520%** |
| `bb81ae522` | a `PARSE` trigger's trace shape decided once, not per target | **-0.2340%** |
| `5e765dc5a` | a `PARSE` source read where it lives instead of copied per execution | **-0.6761%** |
| `26ccef4ee` | the driver's ops read through a slice cut to the loop's own bound | -0.0794% `rexxcps`, **-0.864% `varlookup`**, **-0.750% `emptyloop`** |

## Rules for the figures you produce, all of them learned today

* **A file-level share inside an inlined function sizes nothing.** Measured: the
  whole program fell 151,986,716 while `core/src/slice/index.rs` attributed to
  `run_ops_from::<true>` rose from 817,003,219 to 950,003,537. Rank by
  **whole-program totals and a function's own self cost**. If you want to name a
  stdlib file, say which function it is inside and treat the number as a pointer,
  never as a size.
* **Say for every figure whether it is an A/B or an attribution.** An attribution
  is what the machine currently spends; it is not what a fix recovers. Today a
  ceiling measured by deleting the cause overstated by more than two, a ceiling
  that priced one side of a change understated by three, and an attribution's
  implied fix measured **slower** when built.
* **`rexxcps` carries about 0.01% intrinsic spread** because it renders `TIME()`
  into its own output. `emptyloop` and `varlookup` reproduce to 0.0001%. So a
  sub-percent question is decided on the narrow axes, and `rexxcps` is the axis
  that must always be reported rather than the one that can settle a small
  difference.
* **Two candidates that share a cost cannot both be sized against the same base.**
  One task today lost a candidate to this: its predecessor took two of the three
  indexings it was sized against, and it measured +0.0324%. When you rank, say
  which candidates overlap and what the second is worth after the first.

## What to measure

`valgrind --tool=callgrind` at the current HEAD on `rust/bench-rexxcps/rexxcps.rex`,
the pinned copy, and on the `rust/bench-programs/` axes. Interleave, give the
within-build spread, print the sha256 of every binary, and compare the `.text`
of the measured binary against a rebuild of the committed tree in a separate
target directory:

    objcopy -O binary --only-section=.text <binary> <out> && sha256sum <out>

A whole-file sha256 cannot do that job: `[profile.release]` sets `debug = true`,
so the target directory's path is in the debug info and an identical rebuild
elsewhere hashes differently.

## The questions

1. **Where does the program stand now?** Instructions, ops dispatched, ops per
   clause, instructions per op, and instructions per clause, against the recorded
   6.39 ops per clause and 165.4 Ir per op from 2026-09-20. Count ops by summing
   execution counts at the dispatch jump addresses; the method is in
   `2026-09-20-instructions-per-op.md`.
2. **What are the largest functions by self cost**, and which of them has nobody
   opened? The to-do's item 8 lists a numeric cluster, a text-and-number
   conversion cluster and an allocation cluster, all estimated from profile
   shares and none A/B'd. Re-derive them.
3. **Which remaining to-do items still have the size they claim?** Items 6, 9, 10,
   11 and 12, plus PARSE candidate 1, which was 2.82%, then 1.97%, then 1.76%,
   and part of whose remainder is cost that this session's own commits created.
4. **What does the oracle do differently on whatever comes out first?** It is a
   tree-walker and still wins on several axes. One paragraph, not a design.

## The output

A ranked queue, each entry carrying: what work stops happening, the figure with
its evidence class, what would falsify it, and which existing test catches a
mistake. **A candidate with no derived figure is not a candidate.** An entry
whose honest answer is "cannot be sized without building it" says that, and goes
in a separate list rather than being given a number.

Say plainly which of the old items are now dead, and why, so the queue replaces
the old list rather than sitting beside it.

## Constraints

* **Do not edit any file in the repository** and do not commit. You have your own
  worktree; measure there.
* **The suite does not pass in a fresh worktree**: fourteen `rexx-exec` tests
  panic with `NotFound` because `<checkout-root>/build/lib` has never been built
  there. Known, recorded, pre-existing. You do not need the suite. Do not try to
  fix it and do not report it.
* Oracle runs wrapped `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`
  from a fresh empty directory you `mkdir` yourself. Read
  `rust/corpus/oracle-crashes.txt` first and never run its entries. `build/` is
  `-O2`, so an oracle ratio is not a sanctioned figure; say so if you quote one.
* `grep` here is a ugrep wrapper that skips binary and ignored files. Use
  `/bin/grep -a` for any count you quote.
* **Do not wait on a `pgrep -f` for your own jobs.** The waiting shell's argv
  contains the pattern, so the count never reaches zero; two runs deadlocked that
  way today. Append a `finished` line to a status file and wait on that.

## Report

Write to `.superpowers/sdd/2026-09-21-reprofile-report.md`, or `.txt` if your
harness refuses `.md`, and say which. **Replies truncate at about 8 KB**: send
the ranked queue first, the standing figures second, concerns last. Quote the
command beside every figure and give the exit status you observed.
