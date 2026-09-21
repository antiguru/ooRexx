# PARSE, the contained candidates: 3, 4 and 5

Read `.superpowers/sdd/2026-09-21-parse-cost-scout-report.md` first, all of it.
It is the diagnosis this work comes from, it carries the figures, the line-level
attribution and the tests that catch each mistake, and it was produced by
measurement rather than reading. Sections "Ranked candidates" 3, 4 and 5 are your
three pieces of work.

Deliberately excluded: candidates 1 and 2. Candidate 1 reaches into the parser,
the plan and the op stream; candidate 2 needs a cursor that re-derives its slice
after every call that can allocate, because the source bytes live in an arena
that moves. **Both are worth more than these three and both are decided later**,
after these land and the profile is retaken. That ordering is deliberate: an
attribution has a shelf life measured in landed changes, and these three move
the denominator the big ones were sized against.

## What you are building

| # | what the machine stops doing | attributed |
|---|---|---|
| 3 | entering a seven-argument general assignment for a target that is a plain variable with a bound slot | 202,926,832 Ir, 0.96% |
| 4 | deciding the trace shape once per target instead of once per `PARSE` | 68,880,034 Ir, 0.33% |
| 5 | indexing the piece three times per target instead of once | 23,986,680 Ir, 0.11% |

All three are local to `parse_template.rs`, with candidate 3 adding an entry
point beside `assign_expr_target` in `run.rs`. No parser change. No op-stream
change. `code.slot_for(id)` already answers the question candidate 3 asks.

**Every figure above is an attribution of what the machine currently spends, not
a measure of what a fix recovers.** This session has twice quoted a figure of the
first kind as though it were the second, and both times the fix banked a
fraction. Candidate 3's number is additionally a division rather than an A/B: it
apportions six lines of `assign_expr_target` between its two callers by call
count. Treat all three as ceilings.

## The order, and the stopping rule

One commit per candidate, in the order 3, 4, 5, each gated. After each, measure.

**A candidate whose measured recovery is under 0.15% of the run is reported, not
committed.** Say what you measured and move to the next one. Three small commits
that each bank something are worth more than one that banks the sum of their
estimates, which is not a thing that exists.

If a measurement surprises you in the other direction, say so with the same
force. The `Condition`/`JumpUnless` fusion that landed this morning was derived
at 0.56% and measured **-1.6747%**, because the derivation priced a removed op
at 20 instructions, which is what a dispatch doing *nothing* costs, and the
fusion also removed the register traffic between the pair. Nothing here is a
dispatch-count argument, so that particular correction does not apply to your
figures; the habit it teaches does.

## The hazard in candidate 3, which is the only one that can be silently wrong

A specialisation for "plain variable with a bound slot" is wrong for a target
that is not one. `parse var s a.b` writes a compound tail; `parse var s stem.`
replaces a stem. Getting it wrong writes the right value into the wrong place
and produces no error at all.

Two things make that survivable, and you should use both rather than either:

* `assign_expr_target`'s `Stem` and `Compound` arms already carry
  `debug_assert!(at.is_none(), ...)` tripwires. **They fire only in the debug
  gate**, which is one of the reasons the debug gate is not redundant with the
  release one.
* `corpus/lang/parse_template.rex` and the compound corpus programs cover the
  shapes, and `corpus/lang/parse_triggers.rex` names the wrong answer each
  template mistake prints.

**Assert the boundary, do not rest on it.** The scout established that the target
shapes `exec_parse` can reach today are `Variable`, `Stem` and `Compound` and
nothing else -- but only because a message term as a `PARSE` target is currently
a refusal, `exit 120`, which the parser accepts and `assign_expr_target` has no
arm for. That is where the boundary sits today, not a property of the language:
the oracle runs `parse value 'aa bb' with o~v1 w2` at rc 0. So a specialisation
resting on the set of shapes must **assert** the set, so that implementing the
message target later reddens a test rather than silently taking a wrong path.
That defect is recorded separately in
`.superpowers/sdd/queued/2026-09-21-parse-message-target-unimplemented.md`;
**it is not yours to fix** and the extent of it is underived.

## What is already settled, so you do not re-derive it

From the scout, each measured:

* **The variable pattern `(p0)` costs nothing.** Replacing it with the literal it
  always holds made the run 5,639,869 instructions *slower*. There is no
  variable-pool lookup on that path.
* **The allocations are required.** `alloc_with` is called 2,520,002 times
  against 2,520,000 predicted from the template shapes; the oracle makes
  4,480,008 for the same targets.
* **`PARSE` is not where we are losing.** It is 14.90% of our run inclusive and
  21.49% of the oracle's, over identical structural work. We are about 1.37x the
  oracle there while the whole run is 1.98x.
* **`PARSE UPPER`/`LOWER` must keep copying**; the upper pass is 165.2 Ir over
  the two argument strings, measured.

If you find one of these is wrong, say so with the probe that shows it.

## How it is judged

**The oracle differential is the arbiter.** Run the corpus. `PARSE` is
everywhere in it.

Gates as `rust/CLAUDE.md` defines them, and **run the debug pair**, which is
where candidate 3's tripwires live:

    cargo build --workspace --all-targets --release                              # outside the cap
    REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release --no-fail-fast # under it
    cargo build --workspace --all-targets                                        # outside
    REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast           # under it

plus `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --
-D warnings`. **`memcap 8G` kills a cold release compile** at rc 137 with a
`Compiling` as the last line, which a status-only reading calls a red gate.
Build outside the cap, test under it. Expected at the current tree: 133 binaries,
2649 passed / 0 failed / 4 ignored release, 2650 / 0 / 4 debug.

Commit before any long run and leave the tree frozen while one is in flight.

## Measurement

`valgrind --tool=callgrind` on `rust/bench-rexxcps/rexxcps.rex`, the pinned copy,
never `samples/`. Two rounds per build, interleaved, each build in its own
`CARGO_TARGET_DIR`, sha256 printed beside every binary. Give the within-build
spread; the last two runs measured 0.0068% and 0.0002%.

**Retake the line-level profile after your last commit** and say which of the
scout's five candidates still has the figure it had. That re-profile is a
deliverable, not a courtesy: candidates 1 and 2 are chosen from it next.

## House rules

`rust/CLAUDE.md` governs. Comments minimal: one-sentence overview, then
parameters, returns, panics and non-obvious properties only. **A comment may
never state the size of a set.** No em-dashes. Never drop or re-wrap an existing
comment; inserting an item under a doc block silently reassigns that doc, and
neither `fmt` nor `clippy` sees it.

No `unsafe`. No new dependencies. No process-global state. Never amend a commit.

## Report

Write to `.superpowers/sdd/2026-09-21-parse-contained-report.md`; if your harness
refuses to write `.md` files, return it as text and say so.

Return only: status, commits, the measured recovery for each candidate against
its attributed figure, gate results with exit statuses, the retaken profile, and
any concern. Quote the command beside every figure and give the exit status you
observed. If a run is still going when you report, say so rather than describing
what it will say.
