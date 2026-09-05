# 5c follow-up — ledger

Plan `docs/superpowers/plans/2026-09-04-phase-5c-followup.md`. Append-only.

## Task 0 — receiver override

Controller, inline. Four claims predicted and confirmed (`task-0-report.md`). Sweep of the other
covered classes measured: 105 sharpenable rows, zero would move today; not landed, Moritz's call.
`defaultSize` found observable via `setBufferSize(0)` and written into the plan's Task 1.

Committed `fdf4c6624`; all seven gates 0 (`task-0-report.md`, Gates).

## Task 1 — where the bytes live

Agent `task1-repr-spike`, in three `git archive` extracts under
`claude-build-scratch/5c-followup-task1/` (kept). Report `task-1-report.md`, brief `task-1-brief.md`.
The controller re-read the sha256s, the four `size_of` probes, the perf cells, the RSS readings, the
subclass `EXPOSE` outputs and the `trace i` lines from the raw files; all match the report.

**D80 decided, by measurement**: `Body::Instance` gains `native: Option<Box<BufferState>>`,
`BufferState { bytes: Vec<u8>, capacity, default_size }`. Append loop: object-variable shape at
2.63× the instructions, 11× the cycles, 31× the wall time and 4,068,068 KB against 17,324 KB peak
RSS of the in-body shape; a separate `Body::Buffer` variant loses `pools`/`own`/`name` and refuses a
subclass's `EXPOSE` the oracle answers. `size_of::<Body>()` stays 80 under every shape probed.

**Review question 3 (`defaultSize`)**: carried. **Review question 1 (Task 3 split)**: adopted the
report's §5 — one core-extraction commit, then readers, mutators, caseless, conversion; written into
the plan. **Review question 2 (sweep)**: still Moritz's.

## Task 2 — the constructor carries state

Agent `task2-buffer-state`, committed `2d533034a`; brief `task-2-brief.md`, report `task-2-report.md`.
**Two background-job completion notifications were lost** (the mutation run, then the fast checks):
the agent sat idle each time until the controller's pid waiter noticed and a message woke it. Waiters
keyed on the job's pidfile are now part of every dispatch.

**Controller review.** Code read in full (`dispatch.rs`, `body.rs`, `value.rs`, `string.rs`):
argument conversion precedes `buffer_state_mut` everywhere; `Object~copy` deep-clones the boxed
state (`native_copy` clones the body). Probed on the oracle and the built binary
(`scratchpad/task2-review/oracle/q*.rex`): `buf == 'abc'` is `0` on both (Object identity, not a
string compare); `buf == buf`, `~copy` independence, `~class~id`, `isA`, `hasMethod`, the 97.1
condition — all agree; `'abc' == buf`, `length(buf)`, `buf~request('STRING')`, `buf~makeString` are
`MAKESTRING`-loud, rc 120. **No silent divergence.** `identityHash > 0` differs for every object and
is the known unmatchable, not this task's.

**Gates: G1 G2 G6 G7 = 0; G3 G4 G5 = 101** on `refusal_sites.rs` alone — 19 `run.rs` definition
lines shifted by one; re-derived in the follow-up commit below. The fast-check list in every brief
now includes `--test refusal_sites` whenever `src/*.rs` changes.

**Decision (controller, plan amendment): each task files its own witness.** The differential runs
only `SUBSET_FILES` programs, so a witness in `corpus/unfiled.txt` is a witness nothing runs — and
Task 2's own M3 mutation (growth without doubling) was caught by `mutablebuffer_state.rex` and
nothing else. Leaving both witnesses unfiled across Task 3's five commits would leave that class
of defect ungated for the phase's whole middle. Filed here: `phase-5c.txt` + `EXPECTED_SUBSET_5C`
gain the two programs; `unfiled.txt` loses them; the differential reads `359 of 359` (357 at 5d's
close); the pin's inversion is red exactly on `phase_5c_subset_matches_the_committed_list`; verified
in a clean `git archive` extract with its own target directory. Task 4 keeps the extract
verification of the whole set and the close.

**Carried to the conversion commit**: `'abc' == buf` and `length(buf)` reach the buffer through the
required-string protocol (`MAKESTRING`), while `buf == 'abc'` is Object identity on the oracle —
binding `MAKESTRING` must not turn the latter into a string compare.

Follow-up committed `3f7bc73c4`; **all seven gates 0** (`scratchpad/task2-review/gates/status.txt`,
first line `3f7bc73c4…`, last line `finished`). The tree is green again with the witnesses gated.

## Task 3a — the byte cores leave the builtins

Dispatched at `3f7bc73c4` + this ledger commit; brief `task-3a-brief.md`. A refactor with no
behaviour change: the string algorithms a buffer method needs become plain functions over bytes, on
`delete_range`'s model; the builtin tests and the differential are the controls, one mutation per
extracted core.

Committed `641e76b84`; **all seven gates 0**. Controller review: the string.rs and word.rs diffs read
in full — a mechanical lift, `overlay_bytes` the one rewritten body and covered by M3 and the
splicing rows; 14 mutations, every core caught in `--lib`, spot checks of the raw files match the
report (restored shas identical to after-refactor, M7b 779/0, M1 `358 of 359`, M6 = M6s). Two
findings written into the plan: **the plain `--test corpus` binary is report mode and cannot go red**
— `REXX_CORPUS_GATE=1` is the catcher (my Task 3a brief cited the wrong signal); and
`MutableBuffer~verify` answers `counted` on every path while the builtin's past-the-end zero is an
untagged text (open, `task-3a-report.md` §6.1).

## Between Task 3a and Task 3b — Moritz's profile rulings (2026-09-05, ~08:00)

`[profile.test] opt-level = 3` — G5 keeps `debug-assertions` and stops paying the unoptimised run
time (957 s against 246 s release, measured across three gate runs). `[profile.mutation]` = release
with `lto = "thin"` — mutation harnesses build with `--profile mutation`, in their own
`target/mutation/`, so a mutation run can no longer leave a stale binary in `target/release/`. No
harness parallelisation now; a performance round comes soon, and the crate's 30 ms startup against
the oracle's 4.5 ms (method-dictionary hashing, measured under `perf`) is its first candidate.

Profiles landed at `bce8e56d7`, **all seven gates 0**. G5 read **5 min including the one-off full
rebuild of the test profile**, against 15–20 min before; the whole suite 14 min against 27.
`debug_assertions` confirmed present under `--profile test` and absent under `--release` by
`cargo rustc … --print cfg`; `target/mutation/rexx-run` built with the release binary's sha256
unchanged.

## Task 3b — the readers, over the cores

Dispatched at `bce8e56d7` + this ledger commit; brief `task-3b-brief.md`. Eighteen readers bound over
Task 3a's cores, witness `mutablebuffer_readers.rex` filed in the same commit, mutations under
`--profile mutation`, STRICT corpus as the catcher.

**The agent died at 09:42 on a usage limit** (`You've hit your session limit · resets 11:40am`),
with its work uncommitted: the witness written and oracle-confirmed, byte-identical on both engines
against the WIP build, the BASE negative control read, every refusal measured (208 probe-engine
pairs), eighteen bodies in `dispatch.rs`, `Raised::incorrect_pad` in `error.rs`, the witness filed
and its `sourceline_oracle` companion generated, the report written through §2.3. The interrupted
session also left `target/release/rexx-run` missing -- cargo removes the old output before linking.
The controller backed the tree up to `scratchpad/task3b/wip-backup/`, rebuilt the binary (sha256
`695dd78e…`, the same the agent recorded), re-verified the witness independently, and finished the
task inline: fast checks, the pin inversion, five mutations under `--profile mutation`, the
method-body refresh against the prediction the agent had already written (which matched the
controller's own), the report's remaining sections, the commit and the gates.

Committed `1cefd7daa`; **all seven gates 0** (`scratchpad/task3b/gates/status.txt`, first line
`1cefd7daa…`, last line `finished`; G4 and G5 read `360 of 360 matching`).

The fast checks found one red, the same shape the Task 2 follow-up hit: `corpus/refusal-sites.tsv`
cites each constructor by file and line, and `Raised::incorrect_pad` pushed the 64 error.rs
definitions below it down by 13 lines. Re-derived from the test's own panic by script; the new
`incorrect_pad` send row was probed against the oracle before it was written. Five mutations, five
red, all on the witness: M3 is the only one the zero-argument method-body probe can also see, and
M4 is red by a panic rather than a mismatch, because deleting `verify_bytes`'s start-past-the-end
guard makes the slice index out of range instead of returning a wrong answer. The
`corpus/method-bodies.txt` refresh matched both the agent's and the controller's independently
written predictions: 18 rows loud to answers, none to diverge.

## Task 3c -- the mutators, over the cores

Dispatched at `3c4585cb4`; brief `task-3c-brief.md`. Eleven rows (`insert overlay replaceAt []=
changeStr upper lower translate space delWord delete`) over Task 3a's cores, witness
`mutablebuffer_mutators.rex` filed in the same commit.

Committed `b1cab6dc3`; **all seven gates 0** (`scratchpad/task3c/gates/status.txt`, first line
`b1cab6dc3…`, last line `finished`; G4 and G5 read `361 of 361 matching`). Controller review from
the raw files: the eleven method-body rows move with no new `diverge` (the file's `diverge` total
is 7 before and after), and five of them stay `loud` with their evidence changing from the method's
own name to `MAKESTRING`, which is what a mutator answering the receiver does to a probe that
`say`s the result.

Three readings worth carrying:

* **The brief was wrong that `buf~string` is unavailable.** Task 2 bound `STRING`, and the binding
  is in `dispatch.rs` at `3c4585cb4`, two commits before the agent started. The controller's
  pre-dispatch check of that brief caught a different error in it (it told the agent to work out
  whether a native body can answer its receiver, which `DELSTR` and `APPEND` already do) and missed
  this one. The brief also said `replaceAt` is `overlay_bytes`'s neighbourhood; it splices, and the
  body is `substr_bytes` for the padded front plus the replacement plus the tail.
* **A predicted-red mutation came back green, and that is what found the hole.** The overlay
  pad-default mutation survived because no line of the witness observed a default pad byte for
  `insert`, `overlay`, `replaceAt` or `[]=`: the growth cases reach the padding path but print only
  the length and the capacity. Four `say` lines were added, the same mutation re-ran red, and the
  widened witness is what is committed. The prediction had cited a witness line that did not exist,
  written from the exploration program rather than from the witness.
* **The growth mutation is red but adds no coverage.** Defeating `ensure_capacity` is caught by
  Task 2's `mutablebuffer_state.rex` alone; the without-the-witness run measured it.

`replaceAt` uses the C++ *named* argument overloads, so its refusals are 88.910, 88.911 and 88.912
at rc 168 where the positional ones are 93.92x at rc 163; three new raisers in `error.rs` carry
them. Capacity growth is not one rule: the five `ensureCapacity` call sites pass different
expressions, each observable through `getBufferSize`. Growing: `insert`, `overlay`, `replaceAt`,
`[]=`, `changeStr` (longer branch), `space` (pad over one byte). Not growing: `upper`, `lower`,
`translate`, `delWord`, `delete`.

**Paused here at Moritz's request.** Task 3d (the caseless family) and Task 3e (the conversions,
which flip `say buf`) are not dispatched.

## Task 3d -- the caseless family

Dispatched at `3403c98f7`; brief `task-3d-brief.md`. The eleven caseless rows over the byte cores,
witness `mutablebuffer_caseless.rex` filed in the same commit.

Committed `ac26c1dc2`; **all seven gates 0** (`scratchpad/task3d/gates/status.txt`, first line
`ac26c1dc2…`, last line `finished`; G4 and G5 read `362 of 362 matching`). Controller review from
the raw files: nine files, all eleven caseless rows move to `answers`, the `diverge` total stays 7,
and `refusal-sites.tsv` is genuinely absent from the diff -- the first commit in this family that
adds no `error.rs` constructor.

**The brief's pre-dispatch measurement was the task's spine.** A caseless method is not its twin
with the bytes folded: over `.MutableBuffer~new('axan')`, `pos('an', 1, 3)` is 3 and
`caselessPos('an', 1, 3)` is 0, because the case-sensitive scan reproduces an upstream window
overrun and the caseless one walks probes one at a time. The plan had sketched this task as "a
comparator through the cores", which would have reproduced the overrun in the caseless path. The
agent then bounded the trap by measurement: it reaches `CASELESSPOS`, `CASELESSCONTAINS`,
`CASELESSCOUNTSTR` and `CASELESSCHANGESTR`, and every other name is the fold. Over 24,696 argument
sets, `pos` and `caselessPos` disagree on 122, all of them lower-case data and so all of them the
overrun rather than the fold; its first control for the `countStr`/`changeStr` pair varied nothing
that mattered and it rebuilt the control rather than keeping the 0.

**The mutation instrument was blind and was replaced mid-task.** Both this task and Task 3c recorded
a per-mutant sha of `target/mutation/rexx-run` read straight after `--lib`. `--lib` does not build
that binary, and the differential never runs it -- `corpus.rs`'s `run_rust` calls
`watchdog::run_bounded(.., Invocation::none())` in process. Task 3d's M1 recorded the value Task 3c
reports for its M2, which is what exposed it: two tasks mutating different sources cannot build the
same binary. The instrument is now the sha of the corpus test executable, read after the corpus run
and shown to distinguish two mutants before its numbers were quoted; eight recorded values, eight
distinct. **`task-3c-report.md` is corrected in this commit**; its red and green readings are
unaffected, since those come from the in-process differential and from `method_bodies`.

Seven mutations: six sole-catcher reds (`361 of 362` with the witness, `361 of 361` without), and
M7, the deliberate adds-no-coverage control, which reddens `lang/mutablebuffer_readers.rex` and does
not move the new witness at all. Field level, after the controller asked whether one field was
carrying all six: predicted first-moving line for all six from the code, measured for M1, M4 and M6,
each confirming its prediction over three disjoint groups of lines. M6's prediction had been wrong
in the agent's own arithmetic; it left the prediction file as written and corrected it in the report
rather than editing a prediction after its run.

**M1b is the control shape a later task should copy.** The without-witness run exits 101 on
`every_lang_program_is_run_or_named_unfiled` when the line is merely removed from `phase-5c.txt`,
which reads as a contradiction beside `361 of 361 matching`. Moving the line into
`corpus/unfiled.txt` instead satisfies `corpus.rs:845` and the control then reads rc 0, `361 of 361`,
0 failing tests -- exit status and matching line saying the same thing.

**Falsified, and mine**: the brief asserted `caselessChangeStr`'s method-bodies row would stay `loud`
at `MAKESTRING`. Sent no arguments it raises 93.903 first and never reaches the receiver, so it
moves to `answers` like its siblings. The plan never carried that sentence; Task 3e's brief must not
inherit it.

Refusals: 70 probes, 140 probe-engine pairs, 0 mismatching on all three descriptors.

