# Task 11: the gate -- report

BASE `5ab3028f`.
Commits, in order: `d9f68dd6` (the default flip), `6257baa4` (the `Sync` decision), `e9c76aa1` (the gate document, the two master-plan lines, the anchor's supersession note, and 1786 baseline rows).
Read back from `git log`, working tree clean afterwards.

The deliverable is `docs/superpowers/plans/phase-4e-gate.md`.
This report is what that document does not carry: how the measurements were taken, what went wrong while taking them, and what I could not settle.

## Gates

| gate | result |
|---|---|
| `cargo test --workspace`, dev | 1423 passed, 0 failed, 4 ignored, exit 0 |
| the same, four gate variables STRICT | 1423, exit 0, all four `mode: STRICT` banners present |
| `cargo test --release --workspace` | 1423, exit 0 |
| the same, STRICT, **at the committed tree** | 1423, exit 0, corpus 51 of 51 |
| `cargo fmt --all --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |

**The clippy run's target directory did not exist beforehand and this is recorded as evidence rather than as a claim.**
`ls -d` on the path answered `No such file or directory` before the run; `stat` gives its birth time as `2026-08-11 10:13:58`, inside the run; the run compiled 27 crates and checked 37, `rexx-exec` and `rexx-bench` among them; stderr holds zero `warning` or `error` lines.

Every exit status above was read unpiped, never from a pipeline.

## Method, and the four things it cost to get right

**Every figure comes from `rexx-arms`**, which is the only instrument used. No `perf` was run by hand.

**Three sittings, and the builds in each are the whole design.**

* **The per-commit sitting** (task `11`): five distinct binaries, seven axes, five rounds, one sitting, all cells rotated. Every column of the instruction tables is therefore comparable with every other -- which a set of five separate sittings would not have been.
* **The spike sitting** (task `11-spikes`): head against `QUICKENING = false`, `PatchSlot` on a `Cell<u32>`, and `CALL_SITE_CACHE = false`, five axes.
* **The shipped sitting** (task `11-shipped`): `5ab3028f` against the binary the gate ships, all seven axes, because a flipped default and a changed slot type are a different build.

All 1786 rows are appended verbatim to `rust/bench-baselines/phase-4e-arms.tsv`; nothing in the gate document is a retyped number that is not also a row there.

**Six boundaries were built, five were measured, and the sixth is why.**
`9c20430f` (Task 8) and `e7b8eef8` (Task 7-M2) produce a **byte-identical** `rexx-run` -- `cmp` exits 0, sha256 `36c2b61e...`. That is Task 7-M2 landing the harness and promoting nothing, confirmed by the binary rather than by its commit message, and it is the same relationship `132c3395` and `9da84dc3` already had. So they are one build and one column.

**The archaeology ran in a detached worktree with a shared target directory**, never in the main tree, so the six boundary builds could not disturb the tree under test. Each spike edit was backed up with `cp`, applied, built, and restored from the backup with `sha256sum -c` reporting `OK` -- eight such cycles, every one verified.

**`rexx-arms` refused nothing and needed no argument I had to discover twice.** The only harness surprise was `startup`: it is a `Fixed` workload, so it has no per-pass reduction at all, and its `cycles:u` ratio ranged [0.89617..1.58778] over five rounds on a 600,000-cycle process. Its instruction figure is stable to four decimals. The gate document says so rather than quoting the cycle figure as a result.

## What went wrong while measuring

**The documented oracle wrapper aborts this crate.**
`ulimit -v 1048576` -- the cap `rust/CLAUDE.md` prescribes for oracle runs -- kills `rexx-run` on `rexxcps` with `memory allocation of 402653184 bytes failed`, exit 134. That is the interpreter stack reservation the same file's third bullet describes, met head-on. The bench harness's own 8 GiB cap is what both sides ran under, and the gate document records the failure rather than quietly using the working number.

**A probe of mine could not compile twice before it ran**, both times because `ir::drive` is a private module and its counters are not re-exported. The fix was a temporary `#[cfg(test)] pub(crate) use` in `ir/mod.rs`, restored and sha-verified. The probe was worth the three attempts: without it my criterion 7 verdict would have rested on reading `run_source` and inferring that `Interp::new`'s engine reaches `run_activation`.

**`git revert --no-commit 7d9cfb9a` at head conflicts** in `ir/drive.rs`, which Tasks 9 and 10 rewrote around the enter/leave split. The revert was aborted, the worktree checked clean, and step 3d is reported as unevaluated with the build that would evaluate it named. Hand-resolving that conflict would have produced a fourth driver and a measurement of my own code.

## The unit error I made and caught

My first draft of the residual section put "the op set growing under each promotion, **+45 per pass**" as a **row of 7-M2's itemisation table**.

Two things wrong with it, and the second is worse.

* That table is **per promoted clause** and the `emptyloop` figure is **per loop pass of a body with no promoted clause**. They cannot be added, and the table's total would have been nonsense.
* The +45 was the IR arm's absolute movement 1613 to 1658 across five builds, and **+19 of it is the enter/leave split's own documented cost** -- a trade priced when it landed, not a new finding. Attributing the whole of it to op-set growth would have been a true number with a false cause attached, which is the failure mode this phase has recorded three times.

Corrected to a separate section in its own unit: the gap moves +99 to +140, of which +19 is the split and **+22** is Tasks 8, 9 and 10, with the mechanism named as a candidate and explicitly not claimed, because no build here isolates it.

## Predictions: the two-line version

Full detail is in the gate document. What matters here is which way the errors ran.

**Task 9: five held, two failed.** The two failures are `emptyloop` moving when it was predicted not to, and the patch table being worth 157.75 instructions per pass on `arith` when it was predicted to be at or below noise everywhere. The second is the more interesting: the same bullet that got the magnitude wrong named the mechanism right one sentence earlier -- the hint pays where a site always falls through -- and `arith` and `compound`, the two axes with a site the small-integer path can never serve, are exactly where it pays.

**Task 10: the main prediction failed on four axes of five and the hedge held exactly.** "The axis I would expect to go the wrong way first, if one does, is `strings`, and the mechanism is the extra call boundary" is the most useful sentence any task in this phase wrote before measuring, and it is right down to the axis and the mechanism. Its main prediction -- no movement on the call-free axes -- failed, though every movement is under half a point and they do not point one way.

**Task 10's zero-execution claim about the resolution cache is now measured rather than argued.** `CALL_SITE_CACHE = false` reproduces head's IR-arm per-pass figure **exactly** on `emptyloop`, `arith`, `varlookup` and `compound`. That is what licenses the gate to attribute none of Task 10's movement to the cache, which the brief required and nothing had checked.

## Concerns

* **Criterion 6's first clause is genuinely missing, not merely awkward.** There is no assertion over the compiled op stream of every corpus program; the golden serialiser has exactly one caller and `ir/mod.rs:41` says so in a comment. A promotion that silently stopped firing for a construct the per-construct golden set does not cover would leave every gate in this document green, including the dual-engine sweep -- both arms would agree, because the tree-walker is what the unpromoted path runs. It is cheap to build and I did not build it; scope, not difficulty.
* **Three configurations now exist that nothing re-runs**: `QUICKENING = false`, `CALL_SITE_CACHE = false`, and `Interp::new` on `Engine::Ir`. All three are green today and all three rot silently. The third is the one that matters most, because it is the only thing that has ever run `run.rs`'s exact-stderr trace tests on the compiled arm.
* **The +22 per pass has no mechanism.** I named a candidate and refused to claim it. If it is the driver's `match` widening, it gets worse at Phase 5 rather than better, and it is the item I would cost first.
* **`arith`'s tree-walker arm moved +186.6 instructions per pass at the Task 9 boundary**, on an arm Task 9 does not touch. I have no explanation and the gate document does not offer one. It is visible in the per-pass absolutes in the baseline file.
* **Everything here is one Linux host.** The five-platform requirement is untouched and unchanged.
* **I landed a code change at the gate.** The `Cell<u32>` is small, measured, suite-green in both profiles and re-measured on the shipped binary in a sitting of its own -- but a gate task changing code is a risk, and the brief's instruction to decide rather than leave the atomic in place by default is why I took it. The alternative, keeping the atomic and recording the price, was the other admissible answer.

## What I did not do

* **No oracle benchmark sitting.** Nothing here re-measures the tree-walker against the C++ interpreter; the anchor's figures stand for that pair and the gate says so.
* **No wall clock as an instrument.** The two `rexxcps` cps figures are labelled as 4f's and as one run each.
* **No cache-check assertion built.** Recorded per the brief, with what it would cost and the design question it would force.

---

# Fix round 1: the final review

Commits `7f81c7c9`, `2d5de564`, `b3e0ba6d`, `48eb1bd3`, read back from `git log`.
Suite **1427** passed, 0 failed, 4 ignored, exit 0, in dev and release with all four gates STRICT; corpus 51 of 51; `cargo fmt --all --check` 0; clippy 0 from a target directory that did not exist beforehand (birth time `11:15:30`, inside the run).

**1427 and not 1423** because this round adds four tests, and each exists because something here had no witness.

## The finding that was not prose

The review is right and the fix is at the type level rather than in a habit.

`Side::rust` handed its child `env: Vec::new()`, and `child::run` *adds* to an inherited environment rather than clearing it -- so `REXX_ENGINE` came from the invoking shell and, unset, from `rexx-run`'s own default. At `d9f68dd6` that default moved, so `rexx-bench-suite` and `rexx-bench-band` changed which arm they measure with nothing in their output saying so, against baselines taken on the other one.

**One correction to the review, and it narrows the blast radius rather than widening it.** The review names **three** binaries; it is **two**. `rexx-time` does not use `Side`, does not name `rexx-run`, and takes its whole command line from its caller -- it is this tree's `hyperfine` substitute. Its child inherits the invoking shell's environment, which is the same thing any direct `rexx-run` invocation does and is the intended consequence of the flip rather than a hidden one. The two that hide it are the two that construct `Side::rust` themselves.

The fix names the arm in three places rather than one, because a provenance block is something a caller has to remember to print:

* the constructor takes the arm and there is none without one;
* the label is `rust-ir` or `rust-tw`, so `rexx-bench-band`'s rows carry it in the `side` column they already have -- and `summarise` branches on `oracle`, so old rows reading plain `rust` still reduce;
* `rexx-bench-suite` prints it in the provenance table beside the sha256s, and both binaries take `--engine`, refusing a spelling they do not recognise exactly as `rexx-run` does.

`rexx-arms` now builds its side through the same constructor instead of a `Side` literal, so `REXX_ENGINE` is spelled in one place in the crate.

**The guard was checked for coverage and not only for redness.** Putting `env: Vec::new()` back reddens `a_rust_side_names_its_arm_in_the_environment_and_in_its_label` and **nothing else**: 49 passed, 1 failed, across the whole crate. So it is the sole catcher rather than a second opinion on something already covered.

## The witness that was missing

`rexx-run`'s unset-variable default had nothing checking it, and could not have: `Invocation::into_parts` is `pub(crate)`, so the library's test cannot see the binary's answer, and a binary is a separate crate that cannot reach the library's test.

Both now name `Engine::DEFAULT` and each is pinned to it from its own side. The binary's half needed the decision split out over a `Result` rather than over the process first -- `remove_var` is `unsafe`, this workspace forbids `unsafe`, and libtest runs its cases on threads of one process, so a test that unset the variable was neither writable nor safe if it had been.

## What I disagree with, and it is small

**"Three binaries" is two**, as above.

Everything else in the review reproduces. Two of its prose items were worse than it said and both are now gone rather than corrected in place: `ir/mod.rs`'s "the plan's ten tasks" was a count of a mutable in-repo aggregate *and* wrong, and the review's own "14-program `tests/trace_oracle/` set" is 15 -- which is why the gate document now names that set without counting it.

## What this round did not do

* **No measurement was re-run**, per the brief. Nothing in this round changes a number; the `Side::rust` fix changes which arm a *future* run measures.
* **`phase-4e-anchor.md` and `perf-baseline.md` are not re-measured**, only labelled with the arm that produced them and the command that reproduces it.
