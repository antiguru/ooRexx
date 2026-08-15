### Task 7-M: Find and remove the per-clause cost -- BLOCKS TASKS 8, 9 AND 10

**Decided 2026-08-10 by Moritz, after Task 7 refuted the `varlookup` discharge condition.**

**The premise behind that condition was backwards.** It held that the driver's cost is paid per body-range *entry* and therefore dilutes as a range holds more ops. Task 7 measured it: the cost is per promoted **clause** and **zero** per entry. `varlookup`'s IR-minus-tree-walker went **60.5 to 138.5 instructions per body clause**, +78, taking the axis from 1.0321 to **1.0725** on instructions and 1.0523 on wall with the IR arm slower in 9 pairs of 9.

**So promotion makes criterion 4 monotonically worse, and Tasks 8, 9 and 10 each add promoted clauses.** That is why this task blocks them.

**Two of the three recorded remedies are eliminated by the number itself**, which is what a refuted prediction is worth: hoisting the op range so it travels with `BodyEngine` is per-*entry* work and the per-entry delta is zero; a single-op fast path does not apply to regions that hold three and four ops. Only **reconsidering the two-level shape** survives as a recorded remedy -- but before redesigning, this task asks the cheaper question: **what are the 78 instructions, and how much of them is removable?** 4b-M is the precedent and it is a good one: it found four fifths of the cost on the path both arms take and removed it, which no amount of arguing had found.

**This task promotes nothing.** If removing a cost requires changing what a promotion emits, say so and stop.

**If the answer is that the per-clause cost is inherent to the shape, say it plainly.** That is the finding that sends the phase to a redesign, and reporting it is this task's success rather than its failure.

### The tree-walker's share of the sharing rule: keep extracting, record the cost

**Decided 2026-08-10 by Moritz.** Task 7 regresses the **tree-walker** arm by 0.2% to 1.3% on every axis whose loop body holds an assignment or a `SAY`, purely from extracting `step`'s two arms into the functions the ops share. A probe writing them out inline restores the base figures exactly on both axes.

**That probe is not the remedy, and the reason is the phase's central rule:** two implementations of one semantics is the defect the dual-engine gate exists to catch, and it has caught three Critical defects here already. Paying under 1.3% on the tree-walker to keep one implementation is the trade, and it is taken deliberately.

**Recorded so nobody rediscovers it as a mystery.** The mechanism is that extraction changes LLVM's inlining inside `step`; three `inline` permutations were measured and `#[inline]` is the best available. No mechanism below "the inlining changed" has been named, and none is claimed.

## Global Constraints

* **The C++ tree at `/home/moritz/dev/repos/ooRexx/` is the oracle and is read-only.** Never modify `interpreter/`, `samples/`, `build/`, `ootest/`.
* **Wrap every oracle invocation:** `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx /abs/path/FILE )`. Use `ulimit -v 8388608` for benchmark workloads.
* **Three programs crash the oracle; never run them:** `select; when 1 = 0 then; when 2 = 2 then nop; end`; `say date('M','0','D')`; any `NUMERIC DIGITS` above 1000.
* **Run every oracle probe from a fresh empty subdirectory you `mkdir` yourself, with absolute paths.** The scratchpad root is on the oracle's external-routine search path and a leftover `.rex` file gets called as an external routine.
* **A symbol named `x` or `b` followed by a quoted string parses as a hex or binary literal.** Use `n1`, `cv`, `zz` in probes.
* **Use `/bin/grep -a`, never bare `grep`** -- the wrapper is ugrep with `-I` and silently skips binary files.
* **Read stdout, stderr and exit status as separate descriptors. Never `2>&1`.** Never read a cargo exit code from a pipeline.
* **Never run cargo from the repo root.** Run from `rust/`.
* **`cargo fmt --all --check`**, not `cargo fmt --edition 2024 --check`, which is an error rather than a check.
* **`cargo clippy --workspace --all-targets -- -D warnings`.** A warm target directory makes a green provisional; run from a clean target at the phase gate.
* **`cargo test <name>` exits 0 when it matches nothing.** Read the run count.
* **`cargo test --release` is a distinct gate** -- `lto = "fat"` changes behaviour, which `5253a674` recorded.
* **Mutation runs need `--no-fail-fast`**, or the suite stops at the first catcher and "nothing else caught it" is unmeasured.
* **No `unsafe`.** The workspace sets `unsafe_code = "forbid"`.
* **No em-dashes in comments; use `--`.** No counts of mutable in-repo aggregates in prose.
* **Markdown: one sentence per line, `*` bullets, first-word-only capitalisation including after a colon.**
* **Back up with `cp`, restore from the backup, verify with `sha256sum -c`.** Never `git checkout --`, never `git add -A`, never `git reset --hard`, never force-push.
* **Benchmark comparisons interleave between arms within one sitting.** Never read a comparison across two separate runs.
* **Commit first, then read the hash back with `git log`, then quote it.**


---

## What is already measured, so you do not repeat it

**The instrument.** `perf stat -e instructions:u`. It reproduces to eight significant figures across
builds whose wall clock moves several per cent, and it has a demonstrated zero: two independent
builds of identical source read 1.2e-9 apart.

**Wall clock cannot answer attribution here, and this is measured rather than feared.** Building the
same tree with one comment line added gives a **byte-identical `.text`** -- same sha256, same size,
same load address -- and two runs of it still differ by several per cent. So the noise bounds
*repeated runs of one binary*, not just comparisons between two. A wall-clock claim needs both arms
interleaved in one sitting; anything else says nothing below about 10%.

**The number you are chasing.** `varlookup`, IR minus tree-walker, per **body clause**:

| | instructions |
|---|---:|
| before Task 7 | 60.5 |
| after Task 7 | **138.5** |

**Per body *entry*: zero change.** That is what refuted the amortisation prediction and eliminated
two of the three remedies -- hoisting the op range onto `BodyEngine` is per-entry work, and a
single-op fast path does not apply to regions holding three and four ops.

**Where the +78 is NOT.** `Op::Const` and `Op::TraceLiteral` execute **zero times inside any measured
loop** -- counted directly at `n` and `2n`: one execution on `emptyloop`, one on `strings`, zero on
`varlookup`, `arith`, `compound`, `alloc4c` and `startup`. `varlookup` has **no literal in an
assignment's value position at all**. So the cost is in the clause machinery -- `Op::Clause`,
`Op::TraceClause`, `Op::Store`, `Op::EvalExpr` and the driver arms that run them. **Do not open by
profiling `Const`.**

**Earlier per-pass attribution, for orientation** (`emptyloop`, IR minus TW per `DO`-body pass): 63
before the frame stack, 153 after it, 96 after `settle` was inlined, 96 unchanged through Task 6, 99
after Task 4c. The frame stack is the single largest thing anyone has landed on this axis.

**One cost is deliberate and is not yours to remove.** The tree-walker arm pays 0.2 to 1.3% because
`step`'s arms were extracted into the functions the ops share. An inline probe restores it exactly,
and that probe is **two implementations of one semantics** -- the defect the dual-engine gate exists
to catch, which it has caught three times in this phase. Moritz decided to keep the extraction and
pay the cost. If you find a way to recover the inlining **while keeping one implementation**, that is
in scope; writing the code twice is not.

## What ends this task

1. **The +78 broken down**: which ops and which driver arms, with a number against each. Not a story.
2. **The removable part removed**, measured on the same instrument.
3. **A statement of what remains and whether it is inherent to the two-level shape.**

**This task promotes nothing.** If removing a cost requires changing what a promotion emits, say so
and stop.

**"It is inherent to the shape" is a success for this task, not a failure.** The phase would rather
learn the design is wrong now than after Tasks 8, 9 and 10 have each added promoted clauses to a
regression that widens with every one of them.
