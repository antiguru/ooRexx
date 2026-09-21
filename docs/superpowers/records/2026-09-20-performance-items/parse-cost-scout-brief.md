# Scout: where the instructions inside `exec_parse` go

Read-only diagnosis. You produce a report and a candidate list, not a change.

Item 8 of `.superpowers/sdd/queued/2026-09-20-performance-todo.md` says the
dispatch overhead is about 87 of the 1,016 instructions a `rexxcps` clause costs,
and **the other ~929 is work inside the ops**, none of it opened. `exec_parse` is
the largest single non-driver item in that half: **11.19 instructions per
dispatched op, 6.77% of `rexxcps`**, independently recorded at 6.8% in
`bench-programs/README.md`.

That number says where the time is. It does not say what the work is, whether it
is necessary, or what a fix would look like. That is this scout's whole job.

## What `rexxcps` actually parses

`rust/bench-rexxcps/rexxcps.rex`, which is the pinned copy and not `samples/`:

* line 75, in the timed body: `parse value 'Foo Bar' with v1 +5 v2 .`
* lines 78 to 81, in the timed body: `parse var rc p1 (p0) p5` and three more of
  the same shape, each with a **variable pattern** `(p0)`
* lines 117 to 119, in `subroutine:`, which runs 280,000 times: `parse upper arg
  a1 a2 a3 ., a4`, `parse var a3 b1 b2 b3 .`, and a `parse var rc c1 c2 c3`
  inside a `do 1`

So the hot shapes are a positional pattern, a variable pattern, a multi-target
template with a placeholder `.`, `UPPER` on an `ARG` source, and a comma
separating argument templates. That is most of `PARSE`'s surface, which is why
the figure is large.

## The questions, in order

1. **Where do the 11.19 go?** Break `exec_parse`'s self cost down by source line
   with callgrind's line-level output, and say what each hot line is doing:
   template interpretation, string scanning, allocation, variable assignment,
   trace emission that nothing consumes.
2. **How much of it is re-derived on every execution?** A `PARSE` template is
   fixed at parse time and the same clause runs 280,000 times. If the template is
   walked, decoded or validated per execution, that is work a compiled form
   removes. Say what part of the per-execution cost is a property of the template
   rather than of the data, because that part is the candidate.
3. **What does the variable pattern `(p0)` cost**, and is it paid by the clauses
   that do not have one? A per-execution branch on "is this pattern a variable"
   is cheap; re-resolving a name through the variable pool is not.
4. **How much is allocation?** Each target is assigned a string. Count the
   allocations one execution of line 78 performs and say which are unavoidable
   given the language and which are an artifact of how the targets are built.
5. **What does the oracle do here?** The C++ interpreter is a tree-walker and
   still beats us on several axes. Read its `PARSE` implementation and say in one
   paragraph what its per-execution work is, specifically whether it interprets
   the template per execution or works from something prepared once. Do not copy
   its design; say what it does and what that implies about the floor.

## The output I want

A ranked list of candidates, each with:

* what work it removes, stated as a thing the machine stops doing;
* the instructions per `rexxcps` run it would remove, **derived**, with the
  derivation shown, not a share of a share;
* what it would break if done wrong, and which existing test would catch that;
* whether it is local to `exec_parse` or reaches into the parser, the plan, or
  the op stream.

**A candidate with no derived figure is not a candidate.** Project memory records
a sampled 8.4% that turned out to be 1.87% of instructions, and a candidate whose
two halves came out opposite on two instruments. Use retired instructions as the
instrument, since they are deterministic here to eight significant figures, and
say so beside each number.

## Constraints

* **Do not edit any file in the repository** and do not commit.
* You have your own git worktree. Build and measure in it. **The suite does not
  pass in a fresh worktree** and that is a known, recorded, pre-existing
  condition, not something you caused: fourteen `rexx-exec` tests panic with
  `NotFound` because `<checkout-root>/build/lib` has never been built there. You
  do not need the suite. `cargo build --release` and callgrind on the bench
  programs are enough. Do not try to fix it and do not report it as a finding.
* Measure with `valgrind --tool=callgrind`; retired instructions are the
  instrument. Concurrent load does not perturb them, so do not wait for the
  machine to be idle.
* Oracle runs, if you make any, are wrapped as
  `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`
  from a fresh empty directory you `mkdir` yourself, with stdout, stderr and exit
  status compared as three separate descriptors, never `2>&1`. Read
  `rust/corpus/oracle-crashes.txt` first and never run anything like its entries.
* `grep` here is a ugrep wrapper that silently skips binary and ignored files.
  Use `/bin/grep -a` for any count you will quote.
* The C++ tree at the repository root, `samples/`, `build/`, `ootest/`,
  `oodocs/` and `testbinaries/` are read-only.

## Report

Write to `.superpowers/sdd/2026-09-21-parse-cost-scout-report.md`; that path is
git-ignored. If your harness refuses to write it, return it as text and say so.

Return only: the line-level breakdown, the answers to the five questions, the
ranked candidate list with derived figures, and any concern. Quote the command
beside every figure and give the exit status you observed. If a run is still
going when you report, say so rather than describing what it will say.
