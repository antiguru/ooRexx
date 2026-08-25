# SDD ledger — plan: docs/superpowers/plans/2026-08-17-phase-5a.md

Branch `plan/rust-rewrite`, worktree `/home/moritz/dev/repos/ooRexx-rust-rewrite` (`GIT_DIR` under
`ooRexx/.git/worktrees/` — this IS a worktree; do not create another).

Plan committed at `d926c334d`. Spec: `docs/superpowers/specs/2026-08-17-phase-5-object-model.md`,
adopted at `4f813b58e` and **binding** — where plan and spec disagree, the spec wins.
Twenty-four tasks. Execution base: `d926c334d`.

Global constraints extracted verbatim to `global-constraints.md` in this directory (plan lines
240–436, covering Global constraints and Delivery order). Every dispatch points at that file rather
than restating it.

## Ruling: this plan gets its own workspace, and why

`.superpowers/sdd/2026-08-15-phase-5a-native-layer/progress.md` is the **superseded** plan's ledger.
It holds `Task 1: complete` through `Task 8: complete` under **that plan's numbering**, and this
plan's numbers start at 1 and are unrelated (the plan says so at its own line 31).

The skill's resume rule is mechanical: tasks with a `Task <N>: complete` line are done, resume at
the first without one. Applied to the old ledger by a controller that has lost its context, it
reports this plan's Tasks 1–8 complete and resumes at 9. **That is the single most expensive
recovery failure the skill names, and it would be reached by following the documented rule
correctly.**

The alternative — one ledger with a loud marker — makes the correct answer depend on a reader
noticing a caveat. This plan's own doctrine on its staleness test settles the choice: *"Both refuse
to be reasoned about, which is the property worth keeping; only one of them is true."* A fresh
ledger makes the mechanical rule give the right answer with no reasoning at all.

**Cost if wrong:** the new plan's pre-execution history (plan review, rework, fix round, re-review,
APPROVE) stays in the old directory rather than moving here. Cross-referenced below, in both
directions, rather than copied — a copy is a second thing that can go stale.

**Pre-execution history for THIS plan lives in `.superpowers/sdd/2026-08-15-phase-5a-native-layer/`:**
`phase5a-plan-review.md`, `plan-rework-brief.md`, `plan-rework-report.md`, `plan-fixround-brief.md`,
`plan-fixround-report.md`, `fixround-rereview.md`, `fixround-rereview-package.diff`, and the ledger
entries after its `Task 8: complete` line. The old ledger's entries **before** that line are the
superseded plan's task execution and describe old numbering.

## Pre-flight conflict scan, run 2026-08-20 before Task 1

The scan the skill asks for is two halves. Both are discharged; the second by me, now, because
nothing else had done it.

**Half one — task-pair conflicts and per-task self-consistency: already run, by the plan review, and
re-verified through APPROVE.** `phase5a-plan-review.md` opens with "Task-by-task: is the stated
verification runnable at that point in the sequence?" across all 24 tasks and raised 7 must-fixes,
every one of them exactly a scan finding: a task building a step the spec assigns to 5c (1), a "Done
when" that cannot be met because no task builds what it reads (3), two controls naming rows no task
creates (4, 5), a mutation that cannot flip at the task that owns it (6), and an unowned split (7).
All seven closed; `fixround-rereview.md` confirms and adds three low findings, closed by hand at
`d926c334d`. The producer/consumer orderings are stated in the plan's own "Delivery order" section
(1 before every probe-committing task; 11→12; 17→21; 9,12→21; 21→23) and were checked in that
review.

**Re-running that as a 24×24 table would not be a second check — it would be the same check by a
reader with less of the spec in hand.** Recorded as discharged with its citation rather than
repeated, and the review loop remains the net for conflicts that only emerge from implementation.

**Half two — does the plan mandate anything the review rubric treats as a defect? Scanned, clean.**
The two rubric items are a test that asserts nothing and verbatim duplication of a logic block.
Searched the plan for mandates of either shape: no hit for `verbatim cop`, `copy … verbatim`,
`duplicat`, `identical cop`, `same code in both`; no hit for `asserts nothing`, `no assertion`,
`does not assert`, `smoke test`. **What that search would have done had the claim been false:** it
would have printed the mandating line with its line number, as `/bin/grep -an` does everywhere else
in this session. It printed nothing on both passes.

The plan runs the opposite risk to the rubric's, and runs it deliberately — it is written against
this project's recorded `gate-criteria-failure-modes`, and several of its own constraints
("Say what a check could not see", "Where a task changes what is refused, it says what instrument
catches a regression") exist to force assertions to have teeth. No conflict to rule on.

**Scan verdict: clean. No rulings required before Task 1.**

## Entry conditions, checked rather than asserted

* Tree clean at `d926c334d`, branch `plan/rust-rewrite`.
* Corpus **106 of 106** under `REXX_CORPUS_GATE=1`, no standing red.
* Performance pin `bench-baselines/pinned/rexx-run-15a1ffa98` — checked below before the first task
  that needs it. Task 1 is `tests/`-only and runs no sitting, so the pin is not on its path.
* Staleness test — `git diff 15a1ffa98 HEAD -- rust/crates rust/Cargo.toml Cargo.toml` empty, and
  `15a1ffa98` an ancestor of HEAD. Verified 2026-08-20 during the fix round; re-checked at the base
  of the first task that lands `src/` code, not here.

---
## Task 1 dispatched — base `d926c334d`

`the run timeout, and what "did not finish" means`. Model: standard. Brief `task-1-brief.md`
(plan lines 437–479), report `task-1-report.md`.

**Ruling: briefs are extracted one at a time, at dispatch, not batched up front.** Extracting all
twenty-four now is one command and saves twenty-three later ones, but a brief is a *copy* of plan
text, and plans get amended mid-execution — this project's named failure mode is exactly a copy
whose original moved. Extracting at dispatch means a brief is never older than the dispatch that
uses it. **Cost if wrong:** twenty-three extra `sed` invocations.

**Carried into the dispatch beyond the brief**, because the brief cannot know it:
* The brief cites `oracle.rs:189`/`:214`. The plan's own constraint is that the task relying on a
  citation re-reads it first, and stale `file:line` is a recorded failure mode here. Told to check
  and to report what it found if they moved.
* `wait-timeout 0.2.0` — offline machine, so "is in this machine's cargo cache" is a claim to prove
  before depending on it. Default to the no-new-dependency route.
* No sitting: `tests/`-only, release binary byte-identical, a sitting would measure noise.
* The oracle wrapper, three descriptors, fresh empty directory — the scratchpad is on the oracle's
  external-routine search path.
* `shell-pipe-exit-status`: in an `&&` chain `cmd | tail` reports tail's status. Capture each gate
  command's own.
* `/bin/grep -a` for counts; bare `grep` is a ugrep wrapper with `-I`.
* No subagents, and no self-dispatched reviewer.

### Task 1 returned DONE — `d5647aa4d`, `1212d7f80`. Verified independently before review.

`Command::output` replaced by a 5 ms `try_wait` poll under a 10 s deadline; `CppOutcome`'s
`exit_code: i32` replaced by `termination: Termination` (`Exited(i32)` / `Signaled` / `TimedOut`)
with `classify_termination(Option<i32>, bool)` a pure function; `did_not_finish` reads the status,
never an exit code; `expect_exit_code()` panics rather than inventing one. Rippled into six consumer
test files.

**I re-ran gates 1–4 myself rather than taking the report's word**, capturing each command's own
exit status rather than a pipeline's -- `fmt=0 clippy=0 test=0 gated=0`, corpus **106 of 106**, and
`/bin/grep -ac "^test result: FAILED"` returns 0 on both test logs.

**The one claim worth its own instrument was the deadline actually firing.** The report offers
`finished in 10.00s`, which is *consistent with* the kill and does not witness it -- a test that
never ran its body reports a duration too. Measured as a difference instead: the `oracle_deadline`
binary reports **`finished in 0.00s` ungated and `finished in 10.01s` gated**. The ten seconds is
the deadline, and the ungated reading is the control that gives it meaning.

**The classification's ambiguous arm resolves correctly**, checked at the source rather than
inferred: `(Some(status), _) => Exited(status)` comes first, so a process that exits normally in the
same instant the deadline expires is recorded by its real exit code and not as a timeout. The race
resolves toward the truth, which is the arm the brief left unstated.

Review dispatched on the most capable model -- this harness is what all twenty-three remaining tasks
commit probes against, and the failure mode to hunt is a check blind to its own subject.

### Task 1 review: spec **APPROVE**, quality **REWORK**. Three medium, six low. Fix round 1 dispatched.

Review at `task-1-review.md`. It reproduced all five gate commands independently and reproduced the
report's own numbers, and its "What I could not check" is honest about the difference between
grepping every construction site and watching a mutation redden.

**It answered the question I set it, and the answer was better than mine.** I had offered the
ungated-vs-gated `0.00s`/`10.01s` split as the witness that the deadline fires. The reviewer's is
stronger and does not depend on a duration at all: `Termination::TimedOut` is constructible from
exactly one place, `classify_termination(None, true)`, whose `true` comes from exactly one caller --
`wait_with_deadline`'s deadline branch -- and `do forever; end` has no way to exit on its own. **The
classification is unreachable except by the kill.** A timing reading is circumstantial; a
reachability argument is not.

**Rulings, all nine, most consequential first.**

**M1 -- ruled fix, and the option to document it was ruled out rather than declined.** The deadline
bounds the child; `read_to_end` returns only when every write end of the pipe closes, so a grandchild
the program leaves behind keeps both reader joins blocked with no bound. Measured by the reviewer
against the real oracle: `address system 'sleep 12 &'` exits immediately and the read blocks
**12011 ms**. A committed probe doing `sleep 3600 &` hangs `cargo test` for an hour despite Task 1.

The reviewer offered documenting it plus the plan's human check as the alternative. **That check
cannot see this failure mode.** It runs the probe by hand under `timeout -s KILL 10`, where stdout is
a **terminal**: an orphan holding an inherited terminal descriptor blocks nothing, `rexx` exits, the
prompt returns, the run looks clean. Under the harness the same program's stdout is a **pipe** and
the orphan holds the write end. **The prescribed mitigation comes back green on exactly the programs
that hang the suite** -- this project's `checks-blind-to-their-own-subject` shape, and it is what
turns "document it" from a legitimate honest-half answer into a non-answer. Bounded instead, with the
residual (a leaked process, not a hung suite) documented, and a test that must fail against the code
as it stands.

**M2 -- ruled fix, structural.** `input_oracle.rs`'s `diffs` compares `termination` fields, so two
runs that both died by signal compare **equal** and the case counts as agreeing. The single call site
where the brief's own helper existed and went unused, and it converts the exact defect the task was
written to kill from a false red into a **false green** -- the worse direction.

**M3 -- ruled fix at the callers.** A timed-out corpus program panics from `support/oracle.rs` naming
no program, and `corpus.rs`'s report is emitted after the loop so it never prints. Fixed where the
path is in hand rather than by threading context into `expect_exit_code`.

**L1 -- ruled fix.** The deadline branch discards the status `wait()` returns and hardcodes `None`.
A program finishing within one poll interval of the deadline is reported `TimedOut` and reddens the
suite on a successful run. It also makes `classify_termination`'s `(Some(status), _)` arm reachable
from its only caller -- **an arm a unit test currently pins while nothing in production can reach it**.

**L2, L3 -- ruled fix.** `run_with_stdin` must drop `child.stdin` before waiting, as
`wait_with_output` did. And "the switch every oracle-invoking harness in this crate uses" is
checkable and false; replaced with the reviewer's honest form -- the deadline *mechanism* runs in
both modes, only the *kill* is gate-verified.

**L5 -- ruled no change, and this is the ruling most likely to look like an oversight.** The
size-of-a-set rule targets a **mutable in-repo aggregate**: a count no human re-verifies and the next
task falsifies. "The two plain values", "two failing arms", "those two pipes" name immutable arities
of a fixed signature and cannot rot. The base file is dense with the same shape, so charging them
here would apply a standard the file does not hold itself to. The reviewer drew that line itself and
charged only `write_and_wait`'s "the one caller left", which **is** mutable. Correct, and adopted.

**L6 -- ruled fix in the report only.** No sitting, and the reason stated rather than implied. The
phase-gate command is currently byte-identical to gate 4, because neither `REXX_PHASE_GATE` nor
`CLOSED_PHASES` exists in `rust/` yet -- **vacuous until Task 4**, and worth recording now so its
first non-vacuous run is not mistaken for its first run.

**L4 -- ruled split, and the general half went into the plan at `7e3f3cb41`.** The brief told the
task to "record" a before state and did not say where; `rust/CLAUDE.md` says comments carry no
history. **Every task that measures the state it is about to change meets this collision**, so per
that file's own rule -- correct the plan, not the message that carries the work, because a dispatch
is read once and then lost -- it is now a global constraint. The split: evidence for the contract a
reader is about to rely on stays; an account of what the code did before goes to the report and the
ledger. Deciding test for a borderline case: **strike the historical framing and see whether the
sentence still says the same thing about the code as it is.**

**Instrument note, and it nearly cost a false clean.** My check for banned comment phrasing was a
line-wise `grep`, and these doc comments are hard-wrapped at ~72 columns, so every phrase worth
searching for splits across a line boundary. It returned one hit, a false positive. Collapsing the
wrap first found two real instances. I sent the reviewer the defect mid-review rather than let it run
the same broken search and confirm my wrong answer; it had found them by searching short fragments,
which it called "luck, not method", and supplied the method that is reliable: collapse each
contiguous comment block to one line for base and head, then `diff`. **Adopted for the rest of the
phase.**

**Brief extraction is now by heading, not by line number.** The plan amendment shifted every line
below it, and `sed -n '240,436p'` would have silently extracted a window that no longer means what it
meant. Both extractions now derive their bounds from `^## Global constraints` and `^## Task N:`.

### Fix round 1 returned DONE — `b0c32c924`, `8017f027a`. Re-review dispatched; I am running the
### red demonstration the round could not close.

All nine findings addressed as ruled, including the one ruled **no change**. The implementer also ran
the reviewer's collapsed-comment-block diff on its own output before committing and caught three more
history-framing spots beyond the four the review flagged -- **the method transferred, which is the
result I wanted from sending it mid-review**.

**What it could not close is the one thing my brief asked for by name:** a red/green demonstration
for M1. The brief said *"give it a bound that would fail if you had not made this change, and say in
the report what the same test does against the code as it stands now"*, and the round substituted the
reviewer's independent oracle measurement for it. That measurement is real evidence about POSIX pipe
semantics; **it is not evidence about this test against this code**, which is the recorded distinction
between a mutation that can fail and a test that adds coverage. Doing it myself rather than spending
a fix round on it.

**Method, and the property that makes it a control rather than a re-run:** a detached worktree at
`1212d7f80` (pre-fix), with the post-fix `oracle_deadline.rs` copied on top and nothing else changed.
**It compiles there, exit 0, no errors** -- so the new test calls no API that did not already exist,
and the only variable between red and green is the fix itself. A test that needed new support code
could not have been run this way at all, and the fact that it did not is what makes the comparison
clean.

Running now. Under the fixed code the binary's two tests take about ten seconds each; the pre-fix run
is already past forty-five seconds with no test line emitted, which is the read blocking on the
grandchild rather than anything about the assertion.

**M2's fix checked at the source against my own ruling**, which said structural means red in *both*
gate modes. It is a bare `assert!` in the comparison loop, before `diffs`, naming the case and both
terminations. Unconditional, so no gate variable can turn it green. Meets the ruling.

**One thing I found reading `wait_with_deadline` that the round did not raise**, held rather than
dispatched, because the re-review may reach it independently and finding it twice is worth more than
finding it once: the reader threads do `let _ = pipe.read_to_end(&mut buf);` and then send `buf`
regardless. **A read that errors partway sends a partial transcript that arrives on time and is
treated as complete.** The direction is safe -- this is the oracle side, so a truncated transcript
produces a spurious *divergence*, never a spurious agreement -- which is why it is a note and not a
finding. If the re-review misses it, I will raise it with that severity attached.

### The M1 red/green, run by me. **Red pre-fix, green post-fix, and the bound discriminates.**

| | wall time | verdict |
| --- | --- | --- |
| pre-fix `1212d7f80` + post-fix test | **30.008 s** | **FAILED** -- `cargo test` rc **101** |
| post-fix `8017f027a` | ~10 s | passes |
| the test's bound | 20 s | -- |

The failure message is the one the fix exists for: *"the run took 30.008207618s to return -- a
program that only leaves a background process holding its own pipe open must not block for that
process's own lifetime"*. Thirty seconds is the backgrounded `sleep`'s own lifetime, which is exactly
the quantity M1 said the harness was hostage to.

**The bound is not a race against either number.** Ten seconds below it is the fixed code's own read
budget expiring; thirty above it is the grandchild's lifetime. A threshold with an order of magnitude
of daylight on both sides discriminates; one wedged against either would have been a flake dressed as
a control.

**The other test in the same binary passed in the same pre-fix run.** That is what makes this a
control rather than a smoke alarm: the red is specific to the mechanism M1 added, not a tree that
fails to build or a harness that cannot find its oracle.

**Recording the exit code separately mattered here.** The wrapper reported `[exited with code 0]`
while `cargo test` itself returned **101** -- the shell's last-command-wins rule, this session's
recorded `shell-pipe-exit-status` hazard, and the reason the run captured `prefix_run=$?` on its own
line instead of trusting the tail.

Worktree removed, `git worktree prune` run, tree clean at `8017f027a`.

### Re-review: **REWORK**. 8 closed, M3 open, 7 new, two high. **Both highs trace to my own ruling.**

Re-review at `task-1-rereview.md`. It checked M1's new mechanism on every axis I named and the
mechanism held: worst case **2 x ORACLE_DEADLINE**, bounded once rather than twice because the two
channels share a clock; truncation provably cannot reach a run that finished, since the timeout arm
sets `code = None` alongside the empty buffers so no path yields `Exited(n)` with an emptied
transcript; and L1 and M1 interact in the ruled direction. It built a standalone `rustc` check to
confirm `recv_timeout(Duration::ZERO)` still returns an already-queued value -- the one place the
shared clock could have silently discarded a delivered transcript.

**It also found, independently, the partial-buffer truncation I was holding**, and placed it better
than I had: pre-existing, unchanged by this round, and the one truncation shape that can reach an
`Exited` run. Found twice, which is what I was holding it for. **Ruled out of scope for Task 1 and
recorded as a known gap** rather than widening the task to cover it.

**N1 and N2 are defects in my ruling, and the wording is worth quoting because it looks harmless.**
I wrote: produce a *"`Mismatch`-shaped structural line naming `rel_path`"*. The implementer did
exactly that. **"Shaped like the report's other lines" routes a structural failure through each
caller's verdict channel, and every verdict channel in this tree is gated or set-matched.**

* `corpus.rs` -- a non-finish becomes a `Mismatch`, and the only assertion is
  `!gate || mismatches.is_empty()`. **Green in report mode**, which is gate command 3 and every plain
  `cargo test`.
* `state_builtin_oracle.rs` -- worse. The non-finish returns through the same `Option<String>` the
  caller matches against `DECLARED_GAPS`, so a declared gap that *starts* timing out is absorbed as
  expected. **Green in every mode**, while the file goes on asserting its differential surface has
  not moved and the line naming the program is interpolated only into a message never printed.

Before the round both were red everywhere, via `expect_exit_code`'s panic. **My ruling converted a
loud correct failure into a silent wrong answer at two of three sites** -- the precise defect class
the plan's own global constraints warn about, introduced by the controller enforcing them.

**`builtin_status.rs` is the site that got it right, and it got it right by not following the
framing** -- an unconditional `assert!` naming the program. That is now the standard for all three,
and the corrected ruling is one sentence: **one event, one treatment, and the operator always learns
which program.**

**Scope call on the two unruled sites.** `parse_version_oracle.rs` and `ir_dual_oracle.rs` still
panic through `expect_exit_code`, naming no program -- correct, since a panic is red everywhere, but
it is *exactly* M3's complaint at two more sites. Ruled **in scope**, framed honestly: my M3 ruling
under-scoped the fix, rather than this being new work. Cost is small; leaving one event with three
treatments in the tree is not.

**This is the fourth instance in this project of the same shape** -- see
[[dispatch-prose-is-not-a-control]] and [[false-justification-rides-correct-decision]]. The decision
to fix M3 at the callers was right. The *description* of the shape to fix it into was wrong, and it
shipped inside a correct decision where nothing was looking for it. The re-review caught it because
it was told to weight new mechanism over corrected findings; a reviewer checking "was M3 closed?"
would have said yes.

**Round 2 requires demonstrations for N1 and N2, not arguments.** The re-review could not execute
them -- it edits nothing. The implementer can, and the last round's substitution of an argument for a
red run is the reason this is stated as a requirement rather than assumed.

### Fix round 2 returned DONE — `77f8e507b`, `63a49c9b9`. Re-review 2 dispatched.

M3/N1/N2 fixed as one root-cause change: `state_builtin_oracle.rs`'s `sweep_one` now asserts
unconditionally instead of returning through the `DECLARED_GAPS`-matched channel; `corpus.rs` keeps
the `Mismatch` for the report line and adds a `structural: bool` with an unconditional pre-gate
assertion; `parse_version_oracle.rs` and `ir_dual_oracle.rs` brought to the same shape. N3-N7 fixed.

**Checked at the source before dispatching the re-review**, since it is the exact thing my last
ruling got wrong: `corpus.rs`'s `assert!(structural.is_empty(), ...)` sits at `:672`, ahead of the
gated `assert!` at `:680`, and takes no gate variable. Unconditional, and first.

**The demonstration design is better than what I asked for, and the reason is worth keeping.** I
asked for a red/green on the non-finish at each site. The obvious probe is `do forever` -- and the
implementer rejected it, because **the in-process crate side has no timeout, so `do forever` would
hang the demonstration itself**, which is Task 1's own honest half turned back on the task. It used
`address system 'sleep 15 &'` instead, and confirmed by measurement that `ADDRESS` refuses loudly
in-process in 0.003 s, so the crate side terminates while the oracle side is held by the grandchild.
**Green at `8017f027a`, red on the current tree, naming the case, both sites.** That is the shape I
wanted and a probe I would have specified wrong.

**My brief contained an error and the round flagged it rather than guessing.** I asked for "all three
`oracle_deadline` tests"; the file has **two** non-unit tests -- the hang test and round 1's
grandchild test. No third was intended; I miscounted, and the round was right to say so rather than
invent one or silently report two. Left for re-review 2 to confirm independently rather than taking
either of our words.

**Open, and mine to decide rather than the implementer's:** whether the before-state measurement that
L4 moved out of the module doc needs a home in this ledger. It does -- recorded here now: at
`1212d7f80`, `do forever; end` under the by-hand wrapper `( ulimit -v 1048576; LD_LIBRARY_PATH=...
timeout -s KILL 10 .../bin/rexx do-forever.rex )` ran the full ten seconds and was killed by the
external `timeout`: **rc 137, both channels empty.**

### Pre-dispatch check on Task 2 found a plan defect. Corrected at `7a2f6a414`.

Task 2 reads `oodocs` and `ootest` end to end, so I resolved them against the tree before dispatching
rather than after. **The global constraint pointed at a path where two of the three do not exist.**

It read: *"`/home/moritz/dev/repos/ooRexx/interpreter/`, `ootest/` and `oodocs/`"* -- one prefix and
three names, which is how a reader parses it and how an implementer would have used it.
`/home/moritz/dev/repos/ooRexx/ootest` **is missing**; both working copies live inside this worktree.
An implementer handed that path finds nothing and either stops or guesses, and this is a task whose
whole output is a record of what was read.

**Everything the task names does resolve, at the corrected paths**, and the revisions match the
stamps the task's own verification asks for, so this was a wrong pointer and not a missing
prerequisite: `oodocs/rexxpg` and `oodocs/rexxref` at **r13198**, `ootest/` at **r13178**, and all
four `cls*` XML files plus `rexxpg/en-US/classes.xml` present.

**Both are git-ignored**, which puts them in the same class as the pinned binary -- machine state no
file in the checkout records -- so the bullet now carries the check (`svn info`) rather than the
assumption, exactly as the pin's entry condition was made to. **`oodocs/` has no `svn info` at its
own top level**; its two subdirectories are separate working copies and carry it, which is worth
stating because checking the parent is the obvious move and it returns nothing.

**This is what the pre-dispatch check is for and what plan review cannot do.** Six review rounds read
this plan, including one that specifically hunted controls whose referents had gone stale, and none
resolved that path -- because reading it is not the same act as running it. The prose is not even
wrong so much as ambiguous, and ambiguity survives every reading and dies on the first `ls`.

### Re-review 2: **8 of 8 ruled findings CLOSED.** REWORK on four new lows. Round 3 dispatched.

The corrected M3 ruling held: the structural channel is unescapable at all five named sites, each
assertion traced individually rather than asserted in aggregate. N7 arm one's preservation is guarded
by an existing test that **would fail had it been written the other way round** -- which is the
difference between a fix and a fix with a witness. The demonstrations' premise was reproduced
independently on this host rather than taken from the report.

**D1 is one sentence carrying two defects, and the second is the better find.** The comment names
`"106 of 106 matching"`, which breaks the size-of-a-set constraint outright and is the class I ruled
fix one round ago. But a non-finish is pushed to `mismatches` and never counted in `matched`, so such
a run prints **`105 of 106`**. The sentence names **the one headline that cannot accompany the event
it describes** -- a comment that is wrong about its own subject, not merely against a style rule.

**D2 is the round's own class landing inside the round's own fix.** "The pre-channel code named the
path in exactly this case" is history, added by the commit that was removing history. **And the
implementer's collapsed-comment pass missed it** while the reviewer's pass over the same six files
found it -- so the method that transferred so well two rounds ago is not self-checking. I asked for
the *why* rather than only the fix; a method that can miss silently needs its failure mode named
before I keep relying on it for twenty-three more tasks.

**D3 looks like a cosmetic label and is not.** `format!("{filename}#{checked}")` counts across every
file, so a hang in the one-stanza `rexxcps` reports `#152`. The misleading number is the visible
half; the decisive half is that `datadriven` 0.9.0 walks `fs::read_dir` **unsorted**, so the same
stanza gets a different label on another machine and adding a case file renumbers everything after
it. **A diagnostic that differs between two people looking at the same failure is worse than no
diagnostic**, because it makes them disagree about what they saw.

**D4's sixth site is mine.** `input_oracle.rs:556`/`:567` reaches `expect_exit_code` with no
`did_not_finish` in front -- the exact complaint M3 was raised about. My round-2 brief named two
"also in scope" sites and this was not one, so the code is not an implementer omission; **my
enumeration was short, twice now, on the same finding.** Ruled in.

**The commit message stays wrong, deliberately.** `63a49c9b9`'s subject claims "everywhere" and one
site falsified it as it was written. Amending reviewed work to correct prose costs more than the
false sentence does, so the ledger carries the correction instead --
[[false-justification-rides-correct-decision]]'s recorded shape, third instance, and the ledger is
the only durable place it can be learned.

### Fix round 3 done at `56caf9d08`. All five gates green — and the runner reported failure anyway.

D1-D4 all closed; I verified each at the source. `checked += 1` survives at `ir_dual_oracle.rs:213`
for the `oracle.invocations() == checked` assertion while `in_file` resets per file at `:206`; the
sixth site's assertion is unconditional and names its case; D1's count is gone and no new count
appeared. A cheap scoped re-review is out anyway -- **skipping it because I checked it myself is the
shortcut this project's history punishes, and my last self-ruling produced two highs.**

**The gate run's own exit status was 1, and every gate inside it was 0.** `fmt=0 clippy=0 test=0
gated=0 memcap=0`, corpus **106 of 106** in all three test logs, zero `FAILED`. The 1 came from the
script's last command -- `grep -c "^test result: FAILED"`, which **exits 1 when it finds no matches**.
The success condition and the failure signal are the same event. This is
[[shell-pipe-exit-status]] inverted: the recorded hazard is a red run reported green, and this is a
green run reported red. **Echoing each gate's own status on its own line is what made it readable
either way**, and a run that had trusted the tail would have chased a phantom regression through a
clean tree.

### Second pre-dispatch defect in Task 2, and this one made a criterion vacuous. Corrected at `e52bff06a`.

The `ootest/` reading row said: **"every `testGroup` the spec's enumeration names"**. Measured: the
enumeration names **none**. Its `authority` column cites `provide.xml`, `dire.xml` and
`fundclasses.xml` and never `ootest`; `testGroup` occurs twice in the whole spec and neither
occurrence is in an enumeration row.

**So the criterion was satisfiable by reading nothing** -- achievable and worthless, which is
[[gate-criteria-failure-modes]]'s first shape, in the plan whose global constraints exist to forbid
it. And the row it gates is the one the plan says makes the three-signal rule's second signal
checkable, so a vacuous pass there licenses every later task to declare oracle defects on an
unchecked signal.

**The spec compounds it.** Its own "What I could not check" reads *"Every `ootest` citation in the
enumeration above is inherited from `object-model-correlation.md` and is not independently
verified"*. **There are none to inherit.** The spec is adopted and binding and this plan does not
edit it, so the correction goes in the plan text and into the ledger Task 2 commits -- the route the
plan already specifies for a citation that moved.

**Ruled: derive the mapping rather than read a list that does not exist.** One row per 5a mechanism
in the enumeration, naming the test group that pins it or recording that none does. More work than
the original phrasing implied, and it is the work the row always existed for. A mechanism with no
test group is a real answer and gets recorded as one.

**Two plan defects found by the same act, before Task 2 was dispatched, neither findable by reading.**
The read-only-trees path was ambiguous rather than wrong; this one was a criterion whose referent set
is empty. **Both took one command.** Six review rounds -- including one hunting specifically for
controls whose referents had gone stale -- passed over both, because resolving a reference is a
different act from reading one. The pre-dispatch check is now the highest-yield instrument in this
loop and runs before every remaining task.

### **Task 1: complete** at `56caf9d08`.

Commits `d5647aa4d`, `1212d7f80`, `b0c32c924`, `8017f027a`, `77f8e507b`, `63a49c9b9`, `56caf9d08`.
Plus three controller commits to the plan: `7e3f3cb41`, `7a2f6a414`, `e52bff06a`.

Three fix rounds. Round 1 closed nine findings, round 2 closed eight and two highs that **my own
ruling caused**, round 3 closed four lows. Re-review 3: **APPROVE, 4 of 4 closed, zero new.** It
read `datadriven` 0.9.0's own source to confirm the `walk` closure runs once per file rather than
inferring it from the API -- the right instinct for a binding whose whole defect was that it did not
reset.

**What Task 1 actually delivers.** No probe can hang the workspace: a deadline on the wait *and* a
bounded read, so a program leaving a grandchild holding the pipe no longer blocks for that
grandchild's lifetime. A run that did not finish is a `Termination`, not a `-1` exit code that every
verdict function read as a status divergence. And a non-finish is a **structural** failure at all six
oracle-invoking sites -- unconditional, red in both gate modes, naming its program.

**All five gate commands exit 0, corpus 106 of 106**, verified by me on the final tree and matched
independently by the re-review.

**The two things worth carrying forward are not fixes.**

**One: my M3 ruling is the most expensive thing that happened here.** "Produce a `Mismatch`-shaped
structural line" reads as a formatting instruction and was a routing instruction -- it sent a
structural failure down each caller's verdict channel, which is gated or set-matched, turning a loud
correct failure into a silent wrong answer at two of three sites. The implementer followed it
exactly. **A controller ruling is code that someone else executes, and it needs the same adversarial
read as code.**

**Two: the collapsed-comment method is not self-checking**, and the failure is in the step after it.
The implementer ran the method correctly, then filtered its output through a keyword list assembled
from phrasings earlier rounds had named -- so a sentence using none of those words passed clean while
failing the rule the words were shorthand for. Saved as
[[collapsed-comment-diff-then-keyword-filter]]: read every line of the collapsed diff; keyword pass
second as a backstop, never first as a filter.

### Task 3 pre-dispatch check: **clean.** Every referent resolves, and the measured claim reproduces.

Run while Task 2 was in flight, because the check has now found a defect in two tasks out of two and
a clean result is worth as much as a finding -- it says the instrument is discriminating rather than
finding things because I am looking hard.

* **`utilityclasses.xml:429` and `:6910`** -- both resolve, both carry *"can only be created using
  the native code application programming interfaces"*, and `/bin/grep -a` finds the sentence at
  **those two lines and nowhere else** in the file. So the plan's "exactly the two classes that carry
  one" is not merely cited, it is bounded: the citation and the exhaustiveness claim are both true.
* **`objectclassmethods.xml`** -- exists, and **nothing under `rexxref/en-US/` references it.** The
  trap the extractor has to handle is real rather than remembered.
* **`rexx-extract`'s "four modes already there"** -- resolves to `bif`, `keyword`, `extract` and
  `extract_assertions`. Worth carrying into the dispatch: **all four read `ootest`**, and the new
  binary reads `oodocs` DocBook XML, so "following the four modes" means following their *structure*,
  not reusing their parsing. `rust/corpus/docs/` does not exist yet; Task 3 creates it.
* **The 62 / 38 / 24 measurement re-run by me on the current oracle**: `.environment` iterated, a bare
  `~new` sent to every entry answering `~isA(.Class)`. **62 class entries, 38 constructible, 24
  raising** -- exactly the plan's figures, rc 0, empty stderr, nothing hung inside a 60 s bound. This
  is the number the whole `covered`/`not-covered` status column rests on, and it is the shape that
  goes stale silently across an oracle swap, so it was worth the two minutes.

**My first probe was wrong and the oracle caught it**: I put `signal on syntax`'s label inside a `DO`
block, which is error 47.2, *"Labels are not allowed within a DO/LOOP block"*. Moved the trap into a
`procedure` and it ran. [[probe-discipline]] holds -- **my own probes remain the least reliable
instrument in this loop**, and the only reason this one did not produce a confident wrong number is
that a malformed probe fails loudly rather than quietly. A probe that had silently counted zero
classes would have "confirmed" nothing and read like agreement.

### Task 2 committed the ledger at `237b53f39`. Review dispatched. It went idle without reporting.

**Second agent this session to finish its work and not return a message** -- the first was Task 1's
reviewer. Both had written their files and committed; only the return was missing. Chasing on the
idle notice rather than waiting is what turned this into a two-minute check instead of a stall, and
it is worth doing every time: **`git log` and the report path answer "did it finish?" without the
agent's cooperation.**

`docs/superpowers/plans/phase-5-reading-ledger.md`, one commit, and the five gate commands each
exit 0 with corpus **106 of 106**.

**The task found what it was built to find.** Four wrong C++ citations, three in-tree Rust citations
the tree has moved out from under, the spec's false sentence the brief predicted, and **eleven
mechanisms the enumeration does not carry** -- which is the whole justification for doing this before
the row sets freeze.

**Its handling of the phase-gate command is better than the previous three rounds', and better than
my own.** Rounds 1 to 3 of Task 1 ran `REXX_PHASE_GATE=5a ... cargo test`, got exit 0, and reported
it "vacuous, as expected". This task **declined to run it**, and gave the reason: `/bin/grep -arn
'REXX_PHASE_GATE\|CLOSED_PHASES' crates/` **exits 1**, so setting the variable runs a command that
exits 0 for a reason unrelated to what it is supposed to check. That is
[[checks-blind-to-their-own-subject]] identified in a command the plan itself mandates, and declining
is the right answer where reporting a green is the tempting one.

**A refinement to my own Task 3 pre-dispatch check, which the reading supplies.** I verified
`/bin/grep -a` finds *"can only be created using the native code application programming
interfaces"* at `utilityclasses.xml:429` and `:6910` **and nowhere else in that file**, and called
the plan's exhaustiveness claim bounded. The ledger's finding 10 confirms the claim survives -- and
adds that **three further classes carry a differently worded sentence of the same force**
(`clsRexxContext`, `clsRexxInfo`, `clsStackFrame`, plus `clsVariableReference` as a fourth shape),
each additionally marked `<!-- new() is forbidden -->`.

**So my check bounded a string and I reported it as bounding a concept.** The plan's claim is about
that sentence and is true; unconstructibility is a concept with synonyms, and no grep for one phrasing
can bound it. **A grep-based exhaustiveness claim is exactly as wide as its pattern**, which is the
same shape as the keyword-filter failure two tasks ago: a pattern assembled from known instances
cannot see the next novel phrasing.

### Task 2 review: spec **APPROVE**, quality **REWORK**. Nine findings; **five are one defect.**

The review sampled hard rather than reading for plausibility, and most of the ledger held: all four
C++ corrections right on **both** halves, all three in-tree Rust corrections likewise, **every one of
the eleven new mechanisms real, absent from the enumeration, and 5a's**, and every measurement
outside one finding reproducing to the digit. It reproduced the first high against the oracle itself
rather than arguing it.

**The root cause behind findings 1, 2, 4, 5 and 8: a conclusion recorded without the procedure that
produced it, or with a procedure that does not reproduce it.**

* Two rows record `none` / `not asserted anywhere`. **Both are negative claims from a search whose
  width was narrower than the claim it licensed.** Row `:120` searched the *directive-declared*
  hierarchy and missed a *dynamically built* copy of the same hierarchy **in the same file**, where
  `:386`/`:402`/`:411` discriminate mixin-versus-superclass precedence exactly. Row `:136` searched
  `ACTIVATE` as a method name and missed two explicit pass-ordering assertions.
* Four counts do not reproduce under the stated `/bin/grep -aciE`; one reproduces only under a
  pattern the ledger never names.
* Two C++ citation lists use two standards of "correct", and the looser one is the list introduced as
  "a re-checker should not redo these".
* The unexercisable derivation's stated pattern, run as written, yields a different denominator than
  the one that produced the table.

**Ruled as a class rather than five times: every negative claim and every count carries the exact
command that produced it** -- pattern, paths, revision, verbatim enough to paste. A surviving `none`
is written as *"no test matching `<pattern>` under `<paths>` at r13178"*, never *"nothing upstream
pins this"*. **The first is checkable and the second is a claim no search can support.**

**Why this document cannot afford it, specifically.** Task 3 re-derives against it and enforces its
staleness rule, so a count that will not reproduce reads as upstream drift when nothing moved. And
the global constraints say **Task 2 is what makes the three-signal rule's second signal checkable** --
so a false `none` hands a later task that signal for free and licenses an oracle-defect claim on an
unchecked one. **False `none` is the dangerous direction**, the same way a false green is.

**Third instance today of one pattern, and it is worth naming because it crosses three different
media.** A keyword filter over comment prose could not see a phrasing nobody had named yet; my grep
for one sentence bounded a string while I reported it as bounding the concept of unconstructibility;
and now a search of `ootest` for one construction shape licensed a claim about every shape. **A
negative result is exactly as wide as the search that produced it, and in all three cases the width
went unrecorded** -- which is why the ruling is about recording the procedure rather than about
searching harder.

### Task 2 fix round 1 at `fa244be1a`. Re-review dispatched.

**The ruling found two more of the defect it was written for.** Re-checking every remaining negative
under the new rule turned up `:141` (SELF and SUPER are pinned -- `USELOCAL.testGroup:119`-`:131`
protects all five special variables, and `API/oo/FUNCTION.testGroup:1055`-`:1056` asserts them
individually) and `:153` (`Package.testGroup:782`-`:786` walks the package-local directory beating
`.local`, which is `searchord`'s step 5 over step 6 and exactly D33's amendment, **reproduced on the
oracle**: rc 0, empty stderr, `DEF`). **Four of the mapping's `none` rows were wrong; the review found
two and the rule found the other two.** That is the difference between fixing findings and fixing a
defect.

It also **verified the reviewer's own observation rather than taking it on report** -- that
`RemoveClassMethod`/`HideClassMethod` have no call site in `interpreter/` -- and recorded the
provenance split in the ledger's section heading, so a later reader can tell which items came from
the reading and which from a review.

**The residual limit is stated honestly and is correct:** *"stating a search does not make it wide
enough. A fifth that nobody thinks to widen would look exactly like the rows that survived."* That is
the true bound on this instrument, and it is better to have it written down than to imply the rule
closed the class.

**I committed the same defect while checking for it, inside the hour.** I reported "no fix round
section" in the report, from `/bin/grep -an "^## Fix round 1"`. The section is there as `# Fix round
1` -- an `#`, not `##`. **My pattern assumed a heading level and I stated the conclusion as though it
had searched for the section.** Same shape as the four `ootest` rows, same shape as the keyword
filter, same shape as my grep for one unconstructibility sentence. **The rule I wrote for the
implementer -- state the pattern beside the claim -- would have caught mine too**, because writing
`^## Fix round 1` next to "no fix round section" makes the gap between them visible at a glance.

### Task 2 re-review: all nine **CLOSED**, seven new. Fix round 2 dispatched.

**The ruling is vindicated where it was aimed: 54 of 54 restated counts reproduce**, 13 negatives
re-run with 10 clean, and both rows the round found on its own verified at source. The re-review
resolved citations rather than reading them and re-ran the procedures rather than checking they
existed.

**And three of the seven new defects are the ruling's own shadow: procedures written down without
being run as written.**

* A stated `grep` has **no `-E`**, so under BRE `[[:space:]]+` wants a literal `+`. Run exactly as
  written it returns **zero files** -- the intersection is empty before `activate` is consulted at
  all. **The conclusion survives and its stated reason does not**, which is the worst version: a
  re-runner sees the empty set the row predicts and stops.
* The unexercisable derivation's `awk` prints `NR`, cumulative across files, not `FNR` -- emitting
  `StreamClasses.orx:4703` for a file of 1010 lines. **It produces strings shaped exactly like
  citations, naming lines that do not exist.**
* Every `|` in a mapping pattern is written `\|`. That is correct **GFM table escaping** and renders
  right -- but a later task reads this file with `sed` or `cat`, and `/bin/grep -E` treats `\|` as a
  **literal pipe**. Pasted from the raw text three counts return 0 and exit 1, **reading as upstream
  drift when nothing moved** -- precisely the misreading the document exists to prevent, arriving
  through the fix for it.

**Ruled: a procedure is not written down until it has been run verbatim, from the raw file, and its
output pasted.** If the output is in front of you, the command ran. "I stated the method" and "the
method works" were quietly the same sentence last round; this separates them, and it is checkable at
a glance.

**N1 is the third instance of the search-width defect and the most valuable finding of the task.**
`ootest/ooRexx/base/class/class.testgroup.cls` is a file upstream wrote **for nothing but this
mechanism**, driven by `Class.testGroup:962`. It pins every class existing before any `activate`
runs, asserted three times over; **activate order following the dependency graph**; an instance being
constructible inside `activate`; and the prolog running last. **No row in the ledger mentions that
ordering is pinned at all.**

**The mechanism of the miss is `--include='*.testGroup'`, when the assertions live in a `.cls`** --
and it is in the rows rewritten to fix the first two instances. So the class was not closed by
naming it, and the honest bound the round itself wrote -- *"stating a search does not make it wide
enough"* -- was right within the hour.

**N4 is a correction re-introduced two rows from where it was made**: finding 3 fixed a
mis-attribution to `test_issubclassof_non_class (:167)`, and the same wrong line landed on row
`:120`. [[correction-rounds-introduce-false-statements]], and the reason the re-review brief says to
weight what a round newly wrote over what it corrected.

**Promoted an "observation" to a fix.** `Setup.cpp:325-329` is cited by the spec and sits in neither
citation list. The C++ row's stopping point is *"every `file:line` the spec cites, verified"*, so a
cited line in no list is **a hole in the stopping point**, not bookkeeping. The conservation check ran
over the old list rather than over the spec -- checking the artifact against its predecessor instead
of against the authority that defines it.

### Task 2 re-review 2: seven **CLOSED**, four new. Round 3 dispatched. Trend 9 -> 7 -> 4.

Every command the ledger states was extracted from raw bytes and re-run; all but one reproduce. The
`\|` hazard is closed **by relocation** rather than by explanation -- patterns moved out of table
cells entirely, and all nine block patterns paste from raw bytes and run. That is the right shape: a
reader who pastes does not read the explanation first.

**Two of the four new defects are the search-width defect on axes nobody had thought of, inside the
fix for the previous instance of it.**

* The conservation extractor handles `File.cpp:N` and bare `name :N`. **`ClassClass.cpp:988`-`:990`
  is a third form** -- a bare `:N` continuing an earlier citation *in the same parenthesis*. Cited by
  the spec, in no list, and the ledger now asserts none are left unplaced.
* The extension glob is **case-sensitive**: `--include='*.cls'` does not match
  `ootest/framework/OOREXXUNIT.CLS`, which is the ooTest framework itself. The paragraph also drops
  files its own command printed -- an enumeration that does not survive its own output.

**So the class is not closable by widening, and this round stops trying.** Six instances, each a
correct search whose result was written as a claim about the world. Recorded as
[[negative-claims-are-as-wide-as-their-pattern]].

**The structural ruling: replace every completeness assertion with the pasted output of the check
that supports it.** An assertion cannot be checked at a glance and rots silently; a committed
derivation can, and a later task diffs it. This is [[prose-cannot-review-a-procedure]] applied to a
document rather than to a rule -- keep the criterion, commit the derived artifact.

**The document already contains the standard it needs to meet, which is why the ruling is a
correction and not a redesign.** Its header says of its own denominator: *"an authority nobody
thought to list is invisible to this document exactly as a mechanism nobody thought to enumerate is
invisible to the spec's table."* That is exactly right. The C++ row's *"every citation appears in one
of the lists"* is the sentence that does not live up to it, and the re-review named the cost
precisely: **the completeness sentence is what stops the next reader looking.**

**D3 is small and is the round's own subject.** One row's command carries a literal `...` where every
other writes `<extensions>`; pasted it is `grep: ...: No such file or directory`, **exit 2**. It was
run with the includes and written with an ellipsis -- the gap between "ran a command" and "wrote the
command down" surviving the round whose ruling was about exactly that gap.

### Task 2 fix round 3 at `84b62a596`. **101 commands re-run, 101 reproduce.** Re-review 3 dispatched.

The structural ruling landed as intended: the sentence claiming every spec C++ citation is placed is
**gone**, replaced by the extractor, its invocation and its whole output as committed rows. The C++
authority row no longer reads `done` -- it reads **done against a stated extractor**, with that
extractor's reach written down **and the note that a fourth citation form would be invisible to it
exactly as the third was.** That last clause is the part that matters: it converts a completeness
claim into a bounded one that says where its own boundary is.

**The round's own fix found a fifth defect, and its shape is instructive.** `ClassClass.hpp:176-186`
-- which the spec cites as **its own worked example of a wrong range** -- had been *discussed in the
prose below the tables* and entered in no list, so the loose check counted it placed. **Prose about a
citation read as placement of it.** Only the rule "count the lists' table rows, not the prose beside
them" separated the two.

**Four extractor rules turned out to be load-bearing and every one was wrong at some point during
drafting**, each caught by running rather than reading:

* the carried filename **resets at every line** -- otherwise a `07:06` timestamp is reported as
  `Setup.cpp:06`;
* it resets on **any** filename, not only a C++ one -- otherwise a bare `:453` after an `.orx` name is
  attributed to the last `.cpp` seen;
* the ledger's fenced blocks are **blanked before markers are searched** -- otherwise the script's own
  quoted markers truncate the span it is measuring, and list (c) silently lost 130 lines;
* only table rows count.

**That is the argument for committing a derived artifact instead of a sentence, made by the artifact
itself.** Every one of those four errors was invisible to reading and fatal to the result;
[[prose-cannot-review-a-procedure]] holds, and this time the procedure was the thing being reviewed.

**It amended rather than followed up, for the right reason.** An earlier commit of the round carried
a message claiming a round trip that the round trip then falsified; it was amended before being
reported. **A false claim in an uneditable commit message is the one copy nobody can correct later**
-- [[false-justification-rides-correct-decision]] -- and amending unreviewed work is the cheap moment
to prevent it.

### Re-review 3: every prior finding **CLOSED**, extractor reproduces **byte for byte**, both negative
### controls **fired**. Seven new, no highs. Round 4 to a **fresh implementer**.

The deliverable works and was tested the right way: the extractor was pulled from the file's raw
bytes, run, and diffed against the committed rows; the controls were **made to fire** rather than
accepted as firing. The re-review also confirmed the fifth defect independently -- at `1b536079a` the
string `ClassClass.hpp:176-186` occurs once, in prose, in no list.

**N1 is the seventh instance of the search-width defect and the first that is *silent*.** The spec's
prose is hard-wrapped, so form 3 -- a bare `:N` continuing an earlier citation -- sometimes has its
filename on the previous line. The per-line reset drops those tokens **without printing a row**, so
the `UNPLACED` verdict cannot see them. Three are in the committed spec.

**That is the distinction worth keeping: a check that misses is safe, a check that lies is not.** The
extractor exists so a reader can see the set instead of trusting a sentence -- and on this shape it
prints a clean derivation with the tokens absent. It costs no conclusion today (all three are cited
elsewhere in a reachable form), so **the defect is the blind spot, not the answer**, and the fix is
the mechanism rather than a paragraph documenting the hole. The re-review's argument for that is
decisive: **a wrap is not a choice the author makes about the citation**, so it is the likeliest shape
of the next one added.

**N2 is a reproducibility defect in the committed artifact.** `where()` returns the first section
carrying a matching `:N` **with no file association**, so two citations the ledger's own list (a)
records as *correct* are labelled `found-wrong` -- and both flip to `(a)` when the section order is
permuted and nothing else changes. **An artifact whose labels depend on the order its sections happen
to be listed in is not a derivation.** Ruled: qualify by file, do not close it by disclosure -- the
existing disclosure covers the mechanism and stops one step short of naming the two rows where it
fired.

**N6 is [[insertions-orphan-doc-blocks]] in prose**: the previous round's bolded insertion replaced a
semicolon with a full stop and left the next clause starting with a lower-case `and`.

**Round 4 goes to a fresh implementer, per protocol, and the protocol's reasoning applies exactly
here.** Rounds 4-5 escalate; the model is already the most capable, so the change available is the
eyes. **Seven instances of one correlated error have come from one author**, and that is the pattern
the rule exists for. Against it: the document is intricate and the incumbent knows it. **The findings
are now localized to one line each**, which is what makes the swap cheap -- a fresh agent doing
mechanical work with exact instructions risks little, where a fresh agent doing design would risk a
lot.

### Round 4 at `5a466236b`, fresh implementer. **127 commands run, 125 reproduce.** Re-review 4 out.

**The fresh implementer paid for itself on N2, by fixing the property instead of the finding.** The
finding named two mislabelled rows found under one permutation. The round measured the thing the
finding was *about*: **over all 24 orderings of the four lists the output is one string**, where the
previous script gives **eight** distinct outputs and moves nine rows under a full reversal -- four
and a half times what the single permutation exposed. It also states that with both new rules in
place the cell narrowing subsumes the file guard, and that removing the guard changes no byte today,
**rather than leaving that to be discovered by whoever deletes it.**

**It found a wrong file on a committed row, which is the first substantive error in the ledger's
data rather than in its prose.** The spec writes ``MethodDictionary::hideMethod`` is
``put(TheNilObject, name)`` (`:348-351`) -- a bare `:N` continuing a **`Class::method` name**, not a
filename -- so the carry attributed it to the last `.cpp` seen and it read as `Setup.cpp:348-351`,
which is a macro comment. `MethodDictionary.cpp:348`-`:351` is `hideMethod` with exactly the quoted
expression.

**The row counted as *placed* only because `MethodDictionary.cpp:348` happens to sit in list (a).**
That is the residue the document had described in the abstract, arriving as a concrete instance: a
coincidence of line numbers across two files made a wrong citation look verified. It is also the
eighth axis of the same defect -- an antecedent form nobody had enumerated -- and the fix admits
`Class::method` names, changing exactly one row.

**Its honesty about the alternative is the part I would keep.** Clearing the carry at that position
instead of extending it *"drops the token with no row printed, which is the silent form of the same
defect"* -- so the two candidate fixes differ not in whether they remove the wrong row but in whether
the reader is told anything. That is the distinction the whole round is built on, applied by the
implementer to its own choice.

**And it caught a negative of its own without its pattern**: *"that include exits 1 on
`ootest/framework/`"* holds for the file's own first line and **not** for `OOREXXUNIT` or
`ooRexxUnit`, under which the sibling `.cls` files match and both forms exit 0. Found by re-running
the ledger's own procedure with a pattern of its own choosing and getting a different answer -- which
is the argument of the section it sits in, turned on the section itself.

**Round 5 is the last before the breaker trips.** The re-review is briefed on that and asked to say
which findings it would actually block on, so the adjudication has its opinion rather than only its
list.

### **Task 2: complete** at `68a44aa1c`. Re-review 4 returned **APPROVE**, 7 of 7 closed.

Commits: `237b53f39`, `fa244be1a`, `1b536079a`, `84b62a596`, `5a466236b`, `68a44aa1c`. Four fix
rounds; the fourth on a fresh implementer. Findings 9 -> 7 -> 4 -> 7 -> **2 low, neither blocking**.

**The re-review earned its verdict rather than asserting it.** It pulled the extractor from the
ledger's raw bytes and `cmp`'d against the committed block -- **no difference, 92 rows, no
`UNPLACED`**. It inserted a reorder line and ran **all 24 orderings**: one distinct output,
byte-identical. It ran the previous round's script under the same harness for contrast: **eight**
distinct outputs, **nine** rows moving under full reversal. It made all three negative controls fire.
And it **broke each of the seven rules the script states**, one at a time, confirming every one
behaves as written -- including the guard that changes nothing today.

**Both remaining findings closed by hand.**

**D2** was two words a wrap had taken out of a comment -- *"any filename with no `:N` own"*, missing
"of its" -- and the same rule is correct in two other places, so it was the one wrong copy of three.
Fixed and committed.

**Verifying that fix mattered more than the fix.** The edit sits **inside a fenced block**, and the
script **blanks fenced blocks before searching for its markers** -- the coupling that cost list (c)
130 lines two rounds ago. Re-ran the extractor: byte-identical, 92 rows. A comment-only edit to a
file that contains its own analyser is not obviously safe, and "obviously safe" is what that class of
defect wears.

**D1 stays uncorrected in the report, deliberately.** It is a stale enumeration in
`task-2-report.md`, which is git-ignored today and copied to `docs/superpowers/records/` when the plan
closes, under an **append-only** policy: an entry that turns out wrong is corrected by a later
document, never by editing the one that was wrong. This entry is that correction. The committed
ledger carries **no ellipsis claim at all**, so nothing durable is affected.

**And D1 contains the ninth axis of this task's defect**, which is worth the line: the check searched
"ellipsis" as **U+2026 only**, so the ASCII `...` form was invisible -- and under it, a line this very
round added *is* an ellipsis line inside a fence. A search for a character class that omits half the
class.

**I committed the tenth myself, in the act of verifying.** The ledger states its invocation as the
script **piped through `sort -k1,1 -k2,2V`**; I pasted the script call without the pipe, got a
diff, and briefly had a "does not reproduce" result that was my own truncation of the stated command.
**Copying part of a stated procedure is the same defect as writing one that was never run** -- and the
fix that has worked all task is the same: the command is in the document, so paste the whole of it.

### Task 3 dispatched — base `68a44aa1c`. The extractors and the committed row sets.

Model: most capable. Multi-file, and it produces the denominators both gate tables are checked
against, so an error here is inherited by every task after it.

**Carried into the dispatch beyond the brief**, each because the brief cannot know it:

* **The 62 / 38 / 24 figures are current** -- I re-measured them on this oracle rather than passing
  the plan's numbers through. Including the probe trap that cost me a run: a `signal on syntax` label
  inside a `DO` block is **error 47.2**, and the trap has to live in a `procedure`.
* **A cross-task hand-off the plan does not carry.** The plan grounds `unreachable` in
  `utilityclasses.xml:429`/`:6910` and says those two classes and no others. I verified that
  **for that sentence** -- and Task 2's reading found **three more classes carrying a differently
  worded sentence of the same force**, plus a fourth of the shape, each with a
  `<!-- new() is forbidden -->` comment. The ledger rules them `not-covered` with a trivial opt-in,
  **not** `unreachable`, because the spec grounds `unreachable` specifically in *"instances come only
  from native code"* and none of the three says that. **Task 3 writes those reasons**, so it needed
  the distinction and the plan alone would have given it the wrong answer.
* Task 2's committed ledger is now an input rather than a sibling: its corrected citations and its
  eleven unenumerated mechanisms bear directly on the same books this task extracts from.
* `rexx-extract`'s four existing modes all read `ootest`; this one reads `oodocs` XML. Follow the
  structure, not the parsing.

**And the phase's dominant defect, stated to the task most exposed to it.** Ten instances across the
first two tasks, each on a new axis -- keyword list, quoted phrase, construction shape, heading level,
file-extension glob, citation form, line wrap, antecedent form, character encoding, partially-pasted
command. **This task's entire deliverable is a set of patterns.** So it carries the two habits that
worked: the pattern beside every negative, and the derivation's *output* committed rather than an
assertion that it passed -- which is where Task 2 ended up after four rounds, and is cheaper to start
with than to arrive at.

### Task 3 returned at `fd8cb5cd0`. Five row sets committed. Review dispatched.

`provide-sections.txt`, `hierarchy-edges.txt`, `class-set.txt`, `class-methods.txt`,
`directive-options.txt`, each with a D56 stamp and each re-derived and compared **in both
directions** by `tests/extract_docs.rs`. **The `covered` sweep's neither-arm set is empty.**

**The cross-task hand-off I carried into the dispatch landed exactly right, and it is the reason to
carry such things rather than let the plan alone drive.** All four classes Task 2's reading found --
`RexxContext`, `RexxInfo`, `StackFrame`, `VariableReference` -- are `not-covered`, each with its own
quoted sentence and `file:line`, each recording *"the reference says the user cannot create one and
names a Rexx-level route instead"*. **`unreachable` is exactly `Buffer` and `Pointer`**, each quoting
*"can only be created using the native code application programming interfaces"* at its own line.
The distinction the plan alone would have collapsed -- the user cannot make one **versus** only
native code makes one -- survives into the committed data with its evidence attached.

**The 37-versus-38 gap I flagged is answered rather than papered over:** the 38th `covered` class is
`ArgUtil`, which the books document nowhere, so it has no method rows to sweep. And the statuses sum:
**38 covered, 23 not-covered, 2 unreachable**, against 63 rows.

**It ran a sweep the brief did not require, and it is the one that could still have found something.**
The gated sweep is scoped to `covered` because an unscoped one is unmeetable -- `Alarm`'s documented
instance methods answer on neither arm. But the **class arm needs no instance**, so every
`not-covered` and `unreachable` row can still be asked there, and a bogus name would answer `0`.
Over the other 25 classes, 552 rows: **every row whose own arm is `class` answered `1`, no
exceptions**, and every `0` is an instance-arm row on a class with no instance. **That is
corroboration of the rows the gate deliberately cannot reach**, obtained by noticing that the reason
for the scope does not apply to half the rows.

**Two departures from the brief's stated derivation, both because running the rule produced wrong
rows** -- which is R32's whole argument, arriving on schedule. Taken literally, *"the `SUBDIRECTIVE_*`
arms"* marks `SCIENTIFIC`, `ENGINEERING`, `INHERIT` and `NOINHERIT` as having no parser arm when the
parser plainly handles them through a `SUBKEY_*` switch nested inside `case SUBDIRECTIVE_FORM:` -- and
it contradicts the spec's own stated purpose for the `position` column, *"the parser's nesting is what
settles it"*. And `::RESOURCE`'s `END` is tested by `!=` outside any `switch`, so the literal rule
gives it **no row at all** though it is both documented and implemented. Both widenings keep the
narrow set recoverable by filtering on a committed `evidence` column rather than discarding it.

**It also deleted a `pub fn` nothing called whose doc asserted a relationship that does not exist**,
found by its own post-commit read of the diff -- **not by a tool, because a `pub fn` is not dead code
to the compiler.** That is the same class as an unrunnable check: the compiler's silence is not
evidence.

### Task 3 review: spec **APPROVE**, quality **REWORK** on one. Round 1 dispatched; F5's half fixed
### by me at `c36f52881`.

The reviewer re-derived rather than read: every derived count reproduces from an independent pass,
both oracle sweeps reproduce row for row, about sixty rows sampled against the book at the line they
cite, and every hard case the brief names re-derived independently from `oodocs`.

**F1 is worth more than its Medium, and the reason it survived is the lesson.** `method_rows` takes a
method's name from its section `<title>`, and the book does not always display that title: five
class-table members carry `xrefstyle="template:<text>"`, which **replaces the rendered name
outright**. Three come out right by accident. Two do not -- `DateTime` and `TimeSpan` name their
constructors `new (Inherited Class Method)` pointing at `mth*Init`, so the row set carries
`DateTime init instance` and **no `DateTime new class`**, while `.DateTime~hasMethod("NEW")` is `1`.

**The emitted rows are true and the missing ones are missing** -- and `DateTime` is `covered`, so
these are exactly the rows Task 5's table C probes.

**Every check the task ran was structurally incapable of seeing it.** The both-directions test
compares the extractor **with itself**: a row never derived has no arm to disagree about, and the
file and the derivation agree perfectly on its absence. The oracle sweeps ask the rows that exist.
**A denominator's defect is invisible to every instrument that takes the denominator as given** --
which is the whole reason R32 made this a task, and the reason the plan demanded the `ArgUtil`
assertion for hierarchy rule 1.

**And the task guarded rule 1 and left this rule unguarded** -- the same shape, one rule apart. The
fix is its own pattern applied once more: assert every member's `xrefstyle` begins `select:title`
with a named exception list, and **see it fire**.

**Three findings are about a reader who has the data and not the report.** `.superpowers/` is
git-ignored until the plan closes, so `ArgUtil`'s deliberate absence from `class-methods.txt` is
explained only somewhere that reader cannot see -- and **a deliberate absence and a lost row look
identical in the data**. The explanation moves into the committed header.

**F4 surfaced something worth knowing beyond this task: CI runs no `cargo` at all.** All three
workflows check out `ootest` and nothing else, so **no test in `extract_docs.rs` runs on any CI
platform** -- the row sets are guarded on one developer machine per gate, not continuously. The
task's own module doc says the true thing; its report claimed four of the tests "run everywhere".

**F5 was mine and is fixed.** The plan still stated the narrow `SUBDIRECTIVE_*` rule the task
correctly departed from, so a Task 4 reviewer checking table D against it would have found a mismatch
the artifact explains and the plan does not. The plan now points at the committed header as the
authority; the spec keeps the narrow phrasing at its own `:857` and is not edited.

### Task 3 fix round 1 at `c0fda0a44`. Re-review dispatched.

`class-methods.txt` 1345 -> **1347**, `covered` 793 -> 794, sweep re-run at 37 classes rc 0 with the
neither-arm set still empty. `xrefstyle` is now read, the two missing rows are emitted, and the rule
that reads it is guarded by an assertion with a named exception list.

**The round corrected the review, and the correction changes the exception list rather than just the
account.** The review said three of the five `template:` members come out right anyway. Two of those
three are **not class-table members at all**: `mthTraceObjectNew` and `mthTraceObjectNotifySet` are
`<member>` elements wrapping an `<xref>` **in running prose** -- inside a `<note>` inside a `<para>`
inside a different section entirely, well past `clsTraceObject`'s head. The extractor reads only a
class section's head, so it never saw them; their `select:title` twins in the real table are what
produced those rows.

So the three that "come out right" are **one because its template and its title agree, two because
they are never read** -- and `TEMPLATE_MEMBERS` correctly holds the three that *are* table members.
**If upstream moves a prose one into a table, the assertion fires and a person decides**, which is
the behaviour worth having. A round that had simply implemented the review's list would have carried
two entries that exempt something the extractor cannot reach.

**I nearly reported the row count as wrong, and the miss was mine.** `/bin/grep -avc '^#'` gives
**1348** against the report's 1347; the difference is the single blank line separating the header
from the data. **"Not a comment" is not the same predicate as "a row"** -- eleventh instance of this
session's dominant defect, in a one-line count I ran to check somebody else's number. Counting
tab-bearing non-comment lines gives 1347 and the report is right.

The pattern that keeps catching this is the same one ruled for the ledger two tasks ago: **say which
predicate produced the number**, beside the number. `wc -l` minus comments is a different claim from
"rows", and only one of them was what I meant.

### Task 3 re-review: **APPROVE**, every finding closed. One low, two nits. Round 2 dispatched.

**The guard was verified the strongest way available**: fired on the **production path**, over a real
book with a genuine sixth `template:` member introduced, at both call sites and on both arms --
unknown style and no style at all. Not a unit test of the assertion, the assertion doing its job on
the thing it guards. Both-directions test fails in both directions. `TEMPLATE_MEMBERS`'s correction
checked at the source: it correctly holds three.

**N2 is labelled a nit and is F3's ruling arriving a second time**, which is why it gets fixed rather
than parked. The header says `section` is the section *the name came from*; for the two new rows the
name came from a class-table member's `xrefstyle`, and following `DateTime new class`'s own citation
lands on `<section id="mthDateTimeInit"><title>init</title>`, **which contains no `new` anywhere**.
Grepping the committed file for `xrefstyle`, `template` or `displays` matches nothing.

**So the artifact carries no rule explaining its own two strangest rows** -- the explanation is in
`TEMPLATE_MEMBERS`'s doc, in the crate. Same principle as `ArgUtil`: `.superpowers/` is git-ignored,
and **a reader holding the data but not the crate cannot tell a derived row from a wrong one.** The
mechanism is right; only its record is missing.

**N1 names a shape worth keeping.** The round deleted a wrong count from the source comment and the
**same wrong count is still in the report** -- *"three of the six sentences wrap"*, which measures as
one under the cited-span reading and four under the full-sentence one, and is three under neither.
**A wrong count removed from one place and left in another is worse than either alone**, because the
two now state different tests and a reader cannot tell which is current. That is
[[correction-rounds-introduce-false-statements]] in its quietest form: the correction was right and
incomplete.

**I passed my own miscount down as a warning rather than hiding it.** "Rows" and "non-comment lines"
differ by the blank line under the header; I briefly had the implementer's figure two rows wrong, and
the dispatch asks it to state the predicate beside every count -- which is the same rule I ruled for
the ledger, now applied to me.

### Task 4 pre-dispatch check: **clean.** Four cited shapes resolve, and the claim the plan asked to
### be confirmed is true for two independent reasons.

* **All four `.orx` shapes read at the file**, and each is what the plan says:
  `StreamClasses.orx:115` is `::CLASS 'InputOutputStream' public MIXINCLASS Object INHERIT
  InputStream OutputStream`; `:371` is a quoted class target, `subclass 'Supplier'`; `:546`-`:549` is
  an install-time `::CONSTANT` sending `.File~getSeparator`, whose `::method getSeparator` is
  `private class` two lines above -- a private class method of its own class, exactly as described;
  and `CoreClasses.orx:151` carries its body on the same physical line as the directive. **The plan
  flagged that the spec inherited three of these without re-reading**, so re-resolving them was the
  point rather than a formality.
* **The corpus claim the plan explicitly refuses to have taken on trust** -- *"`corpus.rs`'s directory
  scan reads only `phase-*.txt`, so a new subtree adds no obligation -- confirm that in this task
  rather than trusting this sentence."* Confirmed, and it holds **twice over**:
  `phase_subset_files_on_disk` reads `corpus/` with a **non-recursive** `read_dir` and filters
  `starts_with("phase-") && ends_with(".txt")`. Either property alone would keep
  `corpus/gate-tables/directives/` out.
* `RAW_STDERR_COMPARISON` is in `corpus.rs`; `Invocation::with_engine` is in `rexx-exec`'s `lib.rs`
  and used by `ir_dual.rs`.

**My first pass nearly produced a confident wrong claim, and the mechanism was truncation.** I
grepped for `read_dir` among four alternatives and piped through `head -12`; no `read_dir` line
appeared, and the natural reading was *"there is no directory scan, `SUBSET_FILES` is a const list"*.
**There is one, at `corpus.rs:599`**, and it is the thing the plan's sentence is about. `head` had cut
it off. [[truncated-output-reads-as-absence]], third instance -- and it is the same family as this
session's dominant defect: **an output narrower than the question, read as an answer to the
question.** Counting matches first (`/bin/grep -acn`) is what caught it, because a count of 1 cannot
be truncated into 0.

### **Task 3: complete** at `094fbbdd8`. Re-review 2 returned **APPROVE**, all three closed.

Commits `a8a3b7f22`, `fd8cb5cd0`, `c0fda0a44`, `658514a42`, `094fbbdd8`, plus my `c36f52881` to the
plan. Two fix rounds. Five row sets under `rust/corpus/docs/`, each stamped and each re-derived and
compared in both directions.

**What Task 3 delivers:** the denominators both gate tables are checked against.
`provide-sections.txt` 21, `hierarchy-edges.txt` 57, `class-set.txt` 63, `class-methods.txt` **1347
data rows** (1432 lines = 84 comment + 1 blank + 1347 -- predicate stated, because I got this number
wrong once already), `directive-options.txt` 79. **The `covered` sweep's neither-arm set is empty.**

**The re-review's check on the round's new test is the best piece of verification this session has
produced, and its result is negative.** The round added
`every_method_rows_origin_line_names_its_section` to guard N2. The reviewer **reverted the
extractor's origin logic to its pre-fix behaviour, regenerated the row set to match, and ran the
suite: all eight integration tests passed.** So the test does **not** catch the defect it was written
for.

**The reason is exact and is worth keeping**: for an override row the section's own line
(`<section id="mthDateTimeInit">`) contains the section id **just as much as** the member's line
does, so a `contains(section)` predicate cannot discriminate the two. The test does catch the offset
bug it names -- the reviewer reproduced that failure verbatim on a relocated copy by forcing
`head_base = 0`.

**Ruled: a known limit, not another round.** N2's *substance* is closed in the data and is verified --
`DateTime new class` now cites `utilityclasses.xml:1281`, which displays `new (Inherited Class
Method)`, and a single-sided drift still reddens through the pre-existing both-directions test. What
remains uncovered is a **coincident regeneration of both sides**, which is that module's disclosed
blind spot for **every** row rather than something these two introduced. Nothing that was guarded
became unguarded. The report does not overclaim, which is why this is a nit rather than a finding.

**[[test-can-fail-is-not-test-adds-coverage]], demonstrated rather than argued.** The test can fail,
and it fails on a real bug. It still adds no coverage of the thing it was written for, and **the only
instrument that could tell the difference was reverting the fix and re-running** -- which the
reviewer did, unprompted.

### Task 5 pre-dispatch check: **clean.** M9's citation resolves and every oracle claim reproduces.

* **M9.** `corpus/lang/primitive_classes.rex` exists, and `rexx-parse/src/instruction/tests.rs:1901`
  is `"primitive_classes"` with the `include_str!` on `:1902`. **Those two lines are the only
  references to it anywhere in `rust/`** -- searched `*.rs` and `*.txt`, so it is in **no
  `phase-*.txt` subset** and the differential never runs it. That confirms the plan's charge exactly:
  a file sitting in `corpus/lang/` that looks like corpus coverage and is reached only as a parse
  fixture.
* **The oracle claims, measured today, rc 0 and empty stderr on the run that mattered:**
  `.Array~hasMethod("ID")` and `("DEFINE")` are both **1**; `.Array~method("APPEND")~class` is
  **`The Method class`**; `.Array~method("STRING")` and `.Array~method("ID")` both raise **97.1**;
  and `.Array~instanceMethods` contains `DEFINE`, `ID`, `OF` and **not** `APPEND`.
* So the plan's "what this task cannot see" holds as stated: `~hasMethod` answers 1 for methods that
  are `Class`'s, while `~method` reads the instance dictionary and raises for them -- **a build that
  moved a class method between scopes is not caught on the class arm.**

**Two probe errors in a row, both mine, both loud.** First: `~instanceMethods` returns **a Supplier**,
not an indexed collection, so `~hasIndex` raised 97.1 -- I had assumed a collection because the plan's
sentence says "contains". Second: `.Array~new("DEFINE",...)` treats its arguments as **dimensions**,
raising 93.906 *"Method argument 1 must be zero or a positive whole number"*; the constructor that
takes elements is `~of`.

**Four probe errors this session and every one failed loudly**, which is the only reason none of them
produced a wrong number. [[probe-discipline]] holds, and the specific lesson repeats: **my errors are
in the Rexx, not in the claim being checked** -- each time the plan was right and my instrument was
wrong. A probe that had silently returned `0` for every `hasIndex` would have "refuted" the plan.


### **Task 4: complete** at `db312da3e`. Gate table D and the shared gate-table harness.

Commits `042dbeda8` (79 probe programs plus both READMEs) and `db312da3e` (the harness and the
table). Full report in `task-4-report.md`. **No sitting: `git diff --stat -- rust/crates/*/src/` is
empty across both commits**, so the release binary the axes measure is byte-identical.

**The number this task establishes: 10.** Predicate -- rows of table D whose committed
`owning_phase` answers `"5a"` and whose verdict is not `agree`, where `agree` means the crate and the
oracle match on exit status, `stdout` and `stderr` with `stderr` compared **raw**. Denominators: 36
rows owned by 5a, 79 rows in the table. Whole-table verdicts: **31 `agree`, 48 `diverge-both`,
nothing else**. Loud: 48. Every task after this one reports against the 10.

The ten are `::ANNOTATE`'s five target rows, `::ATTRIBUTE EXTERNAL`, `::METHOD EXTERNAL`, and
`::CLASS`'s `INHERIT`, `METACLASS` and `MIXINCLASS`.

**`REXX_PHASE_GATE` and `CLOSED_PHASES` now exist, and the mechanism fires.** They were in no file in
`rust/` before this commit, so every earlier task's `REXX_PHASE_GATE=5a ...` run exited 0 for a reason
unrelated to what it checks. Measured, all four combinations: report mode 0; corpus gate alone 0
(no closing phase, `CLOSED_PHASES` empty); phase gate alone 0 (a closing phase without the corpus
gate gates nothing); **both together 101**, naming the ten rows.

**So `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1 cargo test --release --workspace` is red from here until
every 5a row agrees, and that is the mechanism working rather than a standing red.** The five gate
commands are untouched -- none of them sets `REXX_PHASE_GATE` -- and both corpus gate commands exit 0
with the corpus at **106 of 106**. Tasks 5 through 23 should read the count off the report, not off
the exit status. Task 24's "Done when" is only a check at all if this command is red before it.

**The plan's `diverge-status` illustration is wrong, measured, and it will mislead Task 5.** The brief
says a loud `::ANNOTATE` row is `diverge-status`; the plan says the same of table C's wiring rows
(*"`.array~id` is rc 120 today, so most wiring rows read `diverge-status` and `loud`"*). A loud
refusal moves **all three** descriptors: it fires at install time so the program's own `say` never
runs and `stdout` empties, it writes `rexx-exec: ...` where the oracle wrote nothing, and the status
goes 0 to 120. Every one of table D's 48 divergences is `diverge-both`. Table C's wiring rows will be
too.

**Three of the five verdict cells had no witness on the unmutated commit**, which is the "a criterion
that is achievable and worthless" hazard arriving inside the verdict function. The plan's mutation 2
asks for one perturbation of an agreeing row; **I ran three**, one per descriptor, at the single
`Outcome` construction in `lib.rs`: `stdout` -> all 31 agreeing rows become `diverge-stdout`, `stderr`
-> `diverge-stderr`, exit status -> `diverge-status`, and the gated count 10 -> 36 each time. Every
cell has now been produced by a running program. All three reverted; `git diff` over `crates/*/src/`
empty and the release `rexx-run` rebuilt afterwards, because a clean `git status` says nothing about
`target/`.

The status mutation also moved the loud count 48 -> 2 while all 48 rows stayed non-`agree`, which is
the one thing the `loud` column exists to distinguish and the one thing it distinguishes **nothing**
by today: on this commit `loud` and non-`agree` are the same 48 rows, because every table-D
divergence is a refusal and none is a wrong answer.

**Mutation 3 fires in report mode with no environment variable set at all** -- delete a probe and the
table reddens on the missing program, structurally, rather than shrinking to 78 rows. The other
direction too: a probe no row names is *"a probe program no row names, so nothing runs it"*.
**Mutation 1 is named against Task 7 and was not run and not faked**: its discriminator is
`.M~baseClass`, which the crate cannot answer, and table D's `MIXINCLASS` row is already non-`agree`
for an unrelated reason, so the check would do the same thing whether or not the claim is true.

**Four more structural channels, each made to fire:** the two engines disagreeing (an unconditional
`assert!` naming the program, which aborts rather than being collected -- Task 1's shape); a row no
`owning_phase` arm covers; a bootstrap shape the `.orx` scan reaches zero occurrences of; and a
verdict cell with an empty preimage. The one I could not fire is the oracle not finishing, which
would need a hanging probe this project forbids committing; Task 1 verified that channel and this
table shares its `did_not_finish` predicate.

**Two rows of table D can never read `agree` and no task in this plan can make them.** `::CLASS CLASS`
and `::RESOURCE LIBRARY` are the row set's two `cross-reference` rows -- documented, no parser arm --
and **both interpreters refuse them with the same number and sub-number**, 25.901 and 25.926. They
differ only in how a syntax error is rendered, which `phase-4-exclusions.txt`'s 2026-08-20 note
records as decided, gated in both directions against `rexxc` for number/sub/line, and *"not Phase 5
work"* that *"nobody owns"*. They are filed under `deferred-parse-error-rendering`, deliberately not
spelled like a phase so it can never match `REXX_PHASE_GATE`. **Had they been filed under 5a, Task 24
could not have closed the phase and the reason would have been invisible in the count.**

**The `.orx` column is derived, and fixing it removed a false hit.** `gate_tables/orx.rs` lexes both
files -- nesting comments, both quotes, the clause-ending `;` -- and holds no line number or class
name. All four shapes came out where they were read at the file this session: `StreamClasses.orx:115`,
`:371`, `:548`/`:549` (the two `::CONSTANT`s; `:546` is the `private class` method that makes them
hits), and `CoreClasses.orx:151` and eleven more. **Measured with the operand rule off, exactly one
hit was a class name standing where a keyword was looked for** -- `::class "Singleton" mixinclass
class public`, whose `class` is `MIXINCLASS`'s operand -- and it landed on the `::CLASS CLASS` row,
the one row whose own evidence says that directive has no such arm. Skipping consumed operands is
what makes the column true rather than a refinement of it.

**A scan the brief did not name does see the new subtree: `rexx-diff` walks `corpus/` recursively.**
Its self-test is the same binary twice and needs determinism, not agreement -- measured with the
probes in place, **204 programs, 0 divergences, exit 0**. Its cross-implementation invocation now
reports their divergences, which is the table's subject. Both READMEs say which rule binds these
files. The four `phase_subset_files_on_disk` copies (`corpus.rs`, `coverage.rs`, `collect_stress.rs`,
`ir_dual.rs`) are all non-recursive **and** filtered to `phase-*.txt`, and `corpus.rs`'s
`the_differential_reads_every_phase_subset_file` is the standing assertion over that set -- so the
confirmation the plan asked for is a test that already exists, not a sentence.

**The pin's staleness test already fails at `094fbbdd8`, before this task, and the failure is
spurious.** `git diff 15a1ffa98 HEAD -- rust/crates rust/Cargo.toml Cargo.toml` is non-empty: Tasks 1
and 2's files under `rexx-exec/tests/`, and Task 3's `rexx-extract` crate including its `src/`.
**None of the four crates the guard names has a `src/` change**, and `cargo tree -p rexx-exec --edges
normal` does not contain `rexx-extract`, so nothing that changed can reach `rexx-run`. The test as
phrased is over more than what the binary is built from, which is the same defect as the phrasing the
constraints rejected. I did **not** establish by rebuild that the pin's sha256 reproduces: an
incremental rebuild from a working tree gave a different hash, and that cannot separate "the source
moved" from "the build is not bit-reproducible under `debug = true`", so it is not offered as
evidence. The first task that owes a sitting should do the clean-tree rebuild `PINNED.md` prescribes.

**One conflict flagged rather than resolved.** The crate's own loud message for `::METHOD EXTERNAL`
says `(Phase 7)`, while the plan's Task 22 -- a 5a task -- owns `::METHOD ... EXTERNAL 'LIBRARY REXX
name'`. The row is filed `5a` on the plan's authority, and its probe names `LIBRARY REXX
zzz_no_entry`, whose oracle answer is rc 166 / `Error 90.998` -- byte for byte Task 22's own
eager-bind shape. `::ROUTINE EXTERNAL` keeps a real shared-library name and is filed `7`, because the
plan says that form stays Phase 7's.

**What the table cannot see, beyond the two the brief names.** A keyword this crate accepts and
ignores reads `agree` when the oracle shows no observable difference either: `::ATTRIBUTE DELEGATE`
and `::METHOD DELEGATE` both agree today and neither row can tell "installed a delegating accessor"
from "parsed the keyword and dropped it". The owning-phase assignment is unchecked and is read by a
human out of a diff. `::CONSTANT` has no row at all, because it has no keyword in either authority --
and it is the directive the third bootstrap shape is about.

**`.superpowers/` is git-ignored and nothing under it is tracked**, checked with `git check-ignore -v`
and `git ls-files`. The report and this entry are untracked files, as every earlier task's are.
### Task 4 returned at `db312da3e`. Gate table D and the shared harness. Review dispatched.

79 rows, **79 committed probes**, harness in `tests/gate_tables/` (`mod.rs` + `orx.rs`), table in
`gate_table_d.rs`. **Headline, with both predicates stated as asked: 10 of 36 `5a`-owned rows are not
`agree`, out of 79 rows.** "Not `agree`" means crate and oracle differ on at least one of exit
status, stdout or stderr, stderr compared byte for byte; "owned by 5a" means `owning_phase` answers
`"5a"`. **This task establishes the number; every later task reports against it.**

### The phase gate is live, and I measured it firing rather than reading that it would.

For three tasks the sixth command was refused on the grounds that it exits 0 for a reason unrelated to
what it checks. **Task 4 is where it gains a subject**, so the honest thing was to run the A/B rather
than announce the change:

| command | exit | result |
|---|---|---|
| `REXX_CORPUS_GATE=1 cargo test --release --test gate_table_d` | **0** | 14 passed |
| `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1 ...` | **101** | 13 passed, 1 **FAILED** at `gate_table_d.rs:546` |

**A difference, on the same tree, from one environment variable.** That is the whole content of
"the mechanism is live", and it is the check the previous three tasks correctly said they could not
perform.

**Recorded so a later task does not `fix` it: that red is by design.** `CLOSED_PHASES` is empty and
stays empty until Task 24 adds `5a`. Until then the table is a **progress report** and the phase-gate
command is a **counter every task reports**, not a sixth gate that must pass. **The five gate
commands are the ones that must exit 0.** A task that finds this command red and treats it as a
regression will spend a round chasing the design.

**Named forward rather than performed: mutation 1.** Its discriminator is `.M~baseClass`, which the
crate cannot answer until Task 7 -- so applying it here leaves the row non-`agree` **whether or not
the claim is true**, which is decoration by this plan's own constraint. The review is asked to check
that the task *named* it rather than performing a mutation that cannot discriminate.

### Task 4 review: spec **APPROVE**, quality **REWORK**. Two medium, seven low. Round 1 dispatched.

Spec side was clean on every clause -- mutations 2 and 3 run, **mutation 1 named against Task 7 and
not faked**, exclusivity and exhaustiveness hold, `loud` never a verdict, `Raw` on every row, no
oracle bytes typed into the table, and the four `.orx` shapes derived by a scan that reddens if any
reaches nothing.

**Both mediums are in the shared harness Task 5 inherits, and both were demonstrated with measured
A/B tables in a scratch tree rather than argued.**

**M1 -- a row can leave the table with no structural channel.** `read_dir` lists a symlink whatever
its target is, so a probe present by *name* but not canonicalisable passes the set check and then
hits an `else { continue; }` whose comment says it was already reported. Measured: `78 rows`, **no
structural failure, exit 0** -- the exact outcome the missing-probe message promises cannot happen.
On a non-agreeing 5a row: **gated count down by one with nothing red.**

**The consequence is what makes it a medium: Task 24 closes Phase 5a when the gated count reaches
zero, and a dangling-symlink probe contributes zero.** The close criterion becomes satisfiable by
**removing evidence** rather than by making it agree. Same shape as Task 1's M3 ruling -- a
structural failure that finds a channel where nothing is red.

**M2 -- the verdict function reads its channels out of a display list.** `contains(&"exit code")` over
a `Vec<&'static str>` documented as *"which of the three observable channels disagree"*, and table D
is its **only** consumer that reads the labels' content; everything else checks emptiness or prints
them. Measured: **crate exit status wrong on every row, plus a rename of `"exit code"` to `"exit
status"`, and table D reports counts byte-identical to the unmutated tree.** The rename alone leaves
the full workspace gate exiting 0 at 106 of 106.

**`contains` is a positive test, so an unrecognised label is simply not seen** -- the asymmetry is the
defect, and it is [[checks-blind-to-their-own-subject]] with the blindness one crate away from the
check.

### L3 was mine, and it had propagated into every dispatch this phase. Fixed at `45a10c7fe`.

The plan's global constraint said *"the workspace sets `unsafe_code = "forbid"`"*. **It sets
`deny`**, at `rust/Cargo.toml:31` -- changed by Moritz on 2026-08-20 **precisely because `forbid`
cannot be overridden by an inner `#[allow]`, which is exactly what an approved site needs.**

So the new comment's conclusion, that the lint *"rules out"* a construct, is **the one thing `deny`
was chosen not to do**. The substance was right in both places -- no `unsafe` here -- and the reason
given was false, which is [[false-justification-rides-correct-decision]] again, this time inherited
rather than invented: the same sentence sits in `corpus.rs:505` and `support/oracle.rs:46`.

**I had pasted it verbatim into every implementer dispatch of this phase.** The correction went to
the plan rather than the next brief, because a dispatch is read once and lost -- and this is the
clearest instance yet of why that rule exists: the false sentence reached four tasks from one bullet.


### **Task 4 fix round 1: complete** at `0d4a63feb`. Two mediums closed in the shared harness, seven lows.

Review returned **spec compliance APPROVE, task quality REWORK**. One commit, on top of `db312da3e`
and the lead's document-only `45a10c7fe`. Report section "Fix round 1" in `task-4-report.md`.
**Still no sitting**: `git diff --stat -- rust/crates/*/src/` is empty across the round, and every
mutation that touched `src/lib.rs` was reverted with the release `rexx-run` rebuilt afterwards.

**The 5a non-`agree` count is unchanged at 10 of 36**, same predicate as before -- rows whose
committed `owning_phase` answers `"5a"` and whose verdict is not `agree`, `agree` being a three-way
byte match with `stderr` raw. Whole table still 31 `agree`, 48 `diverge-both`. **That M1's fix did
not move the population is the thing worth reporting**: its subject is a row leaving the table
without a verdict, and no committed probe does that.

**M1 reproduced before it was fixed, and the reviewer's account was exact.** A probe replaced by a
symlink to a nonexistent path gave `78 rows`, **no structural failure, exit 0**, and `5a: 35 rows, 9
not yet agree` -- the gated count down by one with nothing red. The `else { continue; }` said the row
had already been reported as missing; it had not, because that check compares derived paths against
`read_dir`'s listing and **`read_dir` lists an entry by name whatever the name resolves to**. The
`Err` arm now pushes a `Structural` naming the row, the path and the io error, and the comment says
the earlier check answers a different question rather than pointing at it. Same symlink now: exit
101 in report mode with no environment variable set.

**M2 is the one worth carrying forward as a shape.** The verdict function recovered its three
channels by `contains` on a `Vec<&'static str>` whose documented job is to be *printed*. The fix is
the typed producer the review asked for, not the fallback assertion: `support/oracle.rs` gains
`DescriptorDiff` (one `bool` per channel) from `descriptor_diff_with`, and the label list becomes
`DescriptorDiff::labels()` -- a rendering of that value rather than a second computation of it, so
every existing caller that prints it or tests it for emptiness is untouched.

**Run in both directions, which is what makes it evidence.** Unmutated: `agree: 31`,
`diverge-both: 48`, 5a 10. The review's combined mutation -- crate exit status wrong on **every** row
*and* `"exit code"` renamed to `"exit status"` -- gave counts **byte-identical to unmutated** before
the fix and gives **`diverge-status: 31`, 5a 36 of 36** after it. The rename **alone** is inert both
ways, which is the other half: renaming a display label must not move a verdict.

**The typed producer moves one hazard rather than removing it, and that half is now asserted.**
`corpus.rs` reads the label list for *emptiness*, so a channel silently dropped from the rendering
would read there as agreement on a real divergence.
`labels_name_exactly_the_channels_that_differ` walks all eight field combinations and holds the
rendering equal to them; dropping the `exit code` label fails it in `gate_table_d` **and** in
`corpus`, naming the combination.

**L1 is latent and now checked.** `run_on_both_engines` asserts both arms report zero
`chunks_refused`, because a body the ir arm refuses runs on the tree-walker and makes "the two
engines agree" a comparison of two tree-walker runs while the row is still labelled a two-engine
measurement. Passes on all 79 probes; fires, naming the program, when the count is forced.

**L2 was a readback that could not fail.** `say form()` with no directive answers `SCIENTIFIC`, so
`::options form scientific` + `form()` distinguished nothing. The `FORM` subkeyword probe now sets
`ENGINEERING` and discriminates (measured `SCIENTIFIC` without, `ENGINEERING` with); the `SCIENTIFIC
value-of(FORM)` probe cannot be fixed that way and says so in its own first comment instead of
carrying a `form()` that looks like evidence. Both re-run by hand under `timeout -s KILL 10` and
through the oracle wrapper.

**L3 was inherited, not invented, and the correction is now in all three carriers.** The workspace
lint is `unsafe_code = "deny"`; `forbid` was replaced on 2026-08-20 **precisely because it cannot be
overridden by an inner `#[allow]`**, which is what a granted site needs. So "rules out" was the one
thing `deny` was chosen not to do. Fixed in `gate_tables/mod.rs`, `corpus.rs` and
`support/oracle.rs`; `/bin/grep -rn 'unsafe_code = "forbid"' crates/` now matches nothing.

**L5's real content: the identical-probe groups are one rule, not three kinds**, which is why the
short prose did not look wrong. Re-derived by `md5sum` after L2's change: `ALL`/`SYNTAX
value-of(ALL)`, the six `<condition>`/`SYNTAX value-of(<condition>)` pairs,
`NUMERIC`/`INHERIT value-of(NUMERIC)`, and now `FORM`/`ENGINEERING value-of(FORM)`. A keyword that
takes a value cannot appear without one and a value cannot appear without its keyword, so the two
rows are exercised by the same clause. `SCIENTIFIC value-of(FORM)` left the set with L2's fix.

**L6, stated rather than implied: every `agree` row of table D is acceptance-only.** No probe here
instantiates or invokes anything, so its three descriptors are the same whether the keyword's effect
is implemented or the keyword is parsed and dropped. **The limitation is forced**, and the reviewer
measured it: `.k~new~m`, an outside call to a `PRIVATE` method, `o~at = 5` and `.k~new` are oracle rc
163 / 159 / 0 / 158 and crate rc 120 on all four. No probe this table could have carried at this
commit would see the difference.

**L7 is carried, not decided.** The `::ROUTINE EXTERNAL` row stays filed `7`, and the consequence is
written where the task drawing that boundary will meet it -- beside the `owning_phase` arm and in the
probe's own first comment: a row's identity is (directive, keyword, position), so the two `EXTERNAL`
forms are **one row** and the probe picks the shared-library one. If the `LIBRARY REXX` form moves
into 5a the way `::METHOD`'s did, that behaviour has no row in this table at all, and closing that
needs the row set to distinguish the forms -- which is `directive-options.txt`'s shape, not this
table's.

**Still open, said rather than implied.** The oracle-did-not-finish channel is still unfired (it
needs a probe this project forbids committing). `owning_phase`'s assignment is still unchecked by
anything, by design. And **`StderrComparison::Raw` still has no runtime witness here** -- the
reviewer flipped every row to `Normalized` and the counts did not move, because no committed probe's
`stderr` differs only in a trace indent. The property is a real input to the verdict and nothing in
this table would notice it being switched off; the first row that turns on a trace transcript is
what witnesses it.
### Task 4 fix round 1 at `0d4a63feb`. Nine closed, controls seen firing. Re-review dispatched.

**5a non-`agree` unchanged at 10 of 36, out of 79**, predicate stated -- and the report checked the
thing worth checking: **M1's fix does not move the row population**, because its subject is a row
leaving the table *without a verdict* and no committed probe does that.

**M2 was fixed structurally rather than patched, and that is the difference between the two available
answers.** The brief offered a fallback -- assert every returned label is one the function knows.
The round took the real one: `support/oracle.rs` gains **`DescriptorDiff`**, one `bool` per channel,
and the old `Vec<&'static str>` becomes `DescriptorDiff::labels()` -- **a rendering of that value
rather than a second computation of it.** Every existing caller that prints the list or tests it for
emptiness is unchanged by construction, because there is now only one place the answer is decided.

**Then it noticed the fix had *moved* the hazard rather than removed it, and closed that too.**
`corpus.rs` reads the label list for **emptiness**, so a channel dropped from the rendering would
make a real divergence look like agreement *there*. `labels_name_exactly_the_channels_that_differ`
walks all eight combinations of the three booleans and holds the rendering equal to them --
**recorded as run**: dropping the `exit code` label fails it in `gate_table_d` **and** in `corpus`.
Finding the displaced risk is the part I did not ask for.

**The controls, all three directions:**

| tree | table D |
|---|---|
| unmutated | `agree: 31`, `5a: 10 not yet agree` |
| exit status wrong everywhere **+** label renamed, **before** the fix | byte-identical to unmutated |
| the same, **after** | `diverge-status: 31`, **`5a: 36 not yet agree`** |
| the rename **alone**, after | unchanged -- **correct: renaming a display label must not move a verdict** |

That last row is the one that makes the pair a control rather than a sledgehammer: the fix has to
make the *status* half visible without making the *rename* itself a verdict change.

**L1's reasoning is sharper than the finding.** `Engine::Ir` runs a body the instruction stream
cannot hold **on the tree-walker instead** -- so a refused body silently turns "the two engines
agree" into a comparison of two tree-walker runs, while the row is still reported as a two-engine
measurement. Latent on all 79 probes; made to fire by adding one to the ir arm's count, which names
the program.

### Task 4 re-review: **APPROVE**, 9 of 9 closed, every control fires. Short round 2 dispatched.

It confirmed each fix by **reverting it in a scratch tree and watching the old behaviour return** --
the dangling symlink gives exit 101 with the fix and 78 rows / no structural / exit 0 without it. That
is the strongest confirmation available and it is now this loop's normal standard rather than its
best case.

**N1 is small and gets fixed because Task 5 copies the pattern.** The missing-probe check pushes a
`Structural` **without removing the row**, so the loop still reaches `canonicalize` and fails with the
same `os error 2` a dangling symlink gives. A genuinely deleted probe reports **twice**, and the
second message asserts the file is listed in the directory when it is not.

**The one fact distinguishing the new arm's case from the old check's is the one fact the message
states without checking** -- and since deleting a probe is the commonest structural failure this
table has, it is the message a reader meets most often.

### The last carrier of the stale `forbid` was my own copy, not the plan.

The re-review's out-of-scope line: `global-constraints.md` still read *"the workspace sets
`unsafe_code = "forbid"`"* after the plan was corrected at `45a10c7fe` and all three in-tree copies
were fixed.

**That file is a copy I extract from the plan per dispatch, and I extracted it before making the
correction.** So the attention lens handed to Task 4's implementer and to both reviewers was stale
while its source was right -- **the derived artifact outliving the fix to its origin**, which is the
same shape as every stale recording this project has recorded, arriving in the mitigation I built to
avoid re-reading the plan.

Re-extracted; it reads `deny`. **The general lesson is not "re-extract more often" -- it is that a
copy needs a rule saying when it is stale**, and this one now has the only rule that holds: it is
re-derived from the plan by heading at every dispatch, so the window is one task wide. What made this
instance visible was a reviewer reading the lens rather than only the code.


### **Task 4 fix round 2: complete** at `fe4c95913`. Re-review returned APPROVE; five items closed.

One low, one report-only low, three nits, on top of `0d4a63feb`. Report section "Fix round 2".
**5a non-`agree` unchanged at 10 of 36**, same predicate. Whole table 31 `agree`, 48 `diverge-both`.
Five gate commands 0, corpus **106 of 106**; `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1` 101 with the ten
named. **No sitting**: `git diff --stat -- crates/*/src/` empty.

**N1 is the round's real content, and it is a defect the fix for M1 introduced.** The `Err` arm's
message said the probe was *listed in the directory*. True of a dangling symlink, false of a deleted
probe -- and a deleted probe reaches that arm too, because the set check reports the row without
removing it from `rows`. **Reproduced: two `Structural` entries for one row, the second asserting a
listing entry that is not there.** Both cases give the same `os error 2`, so the message's one
distinguishing claim was the one thing it did not check. **Deleting a probe is this table's commonest
structural failure -- it is mutation 3 -- so that was the message a reader would meet most often.**
Guarded on `on_disk.contains(&probe)`; measured after, a deleted probe reports once as a missing
probe and a dangling symlink once as listed-but-unresolvable, both exit 101 in report mode.

**The nit worth keeping is that the comment's own example was not its arm's case.** It named a probe
*"whose bytes cannot be reached"*; measured, `chmod 000` canonicalises fine and fails later in
`run_on_both_engines`'s `fs::read`. Still red, so nothing was at risk -- but a comment that
illustrates a path with something that does not take it is the same shape as a check that cannot see
its own subject, one level down. The comment now names the unreadable file as the thing this arm is
*not* about.

**N2 is `correction-rounds-introduce-false-statements` in miniature.** Round 1's L2 fix made
`::OPTIONS FORM` and `::OPTIONS ENGINEERING value-of(FORM)` byte-identical, and round 1's own L5
paragraph states the rule that makes them so -- yet the L6 paragraph two sections later still listed
three readback rows where there are four. **The correction was right and its neighbourhood was not
re-read**, which is the recorded failure mode exactly. Rewritten to derive the set from L5's rule
rather than enumerate it.

**The 93-character doc line is the case where no gate can see the defect.** `cargo fmt` does not
rewrap doc comments, so the only instrument is a person looking -- which is how it arrived, as a nit
rather than a red. Checking for it found two more of this task's own at 81 and 95. Verified per file
over comment lines: **no line this task added exceeds 80 characters**; the four that remain long in
the shared files predate it.

**Worth knowing, and not ours: the copy outlived the source.** `global-constraints.md` still carried
the stale `forbid` sentence after the plan was corrected, because the controller extracts that file
per dispatch and had extracted it before the fix -- **the attention lens handed to this task was
stale while the plan was right.** Re-extracted. That is a fifth carrier of the same claim, after the
plan, `gate_tables/mod.rs`, `corpus.rs` and `support/oracle.rs`, and the only one no reader of the
crate would have found.
### **Task 4: complete** at `fe4c95913`.

Commits `042dbeda8`, `db312da3e`, `0d4a63feb`, `fe4c95913`, plus my `45a10c7fe` to the plan. Two fix
rounds. **Five gates verified by me on the final tree: `fmt=0 clippy=0 test=0 gated=0 memcap=0`,
corpus 106 of 106.**

**What Task 4 delivers:** gate table D over 79 rows with 79 committed probes, **and the shared
gate-table harness Task 5 inherits** -- five mutually exclusive, jointly exhaustive verdict cells,
`loud` as a column and never a verdict, `Raw` stderr on every row, both crate engines in process with
an unconditional agreement assertion before any verdict exists, and a structural channel red in both
modes. **10 of 36 `5a`-owned rows are not `agree`, out of 79** -- predicate: `owning_phase` answers
`"5a"` and crate and oracle differ on exit status, stdout or raw stderr. **This is the number every
later task reports against.**

**Both mediums were in the shared harness, and that is the whole argument for the two rounds.** Table
C would have copied a structural failure that vanishes without a channel, and a verdict function that
recovers its inputs by matching display strings. **Fixing them once here is the difference between
one round and two tables.**

**The best single result of the task is a measurement I did not ask for.** With the crate's exit
status wrong on **every** row *and* one label renamed in a shared file, table D reported counts
**byte-identical to the unmutated tree**, and the full workspace gate exited 0 at 106 of 106.
**Two independent defects cancelling into a clean bill of health** -- and the fix was to make the
string list a *rendering* of a typed value rather than a second computation of it, so there is now
one place the answer is decided.

**The phase gate is live and its red is the design.** `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1` exits
101 while the unphased run exits 0; `CLOSED_PHASES` is empty until Task 24. Recorded here twice
because a later task meeting that red and treating it as a regression will spend a round on it.

### Task 5 returned: `e00ac9daf`, `0e605eb77`. Gate table C. Review dispatched.

**1488 verdict rows plus one assertion row, 237 committed probes**, sharing Task 4's harness. Five
gates exit 0 with corpus 106 of 106. Verdicts on the creating commit: `agree: 32`,
`diverge-both: 1434`, `diverge-stdout: 22`, `loud: 1432` -- **red by design**, since `.array~id` is
rc 120 today.

**5a gated rows: table D 10, table C 135.** And the report caught something about how that number is
obtained: **`cargo test` without `--no-fail-fast` stops at the first failing target**, so the
workspace-level phase-gate run reports only whichever table it reached first. It ran the two tables
separately so **both numbers are measured rather than one inferred** --
[[mutation-runs-need-no-fail-fast]] applied to a counter rather than to a mutation.

**M9's disposal is better than either option the plan offered.** The plan said promote or delete.
`corpus/lang/primitive_classes.rex` asked `~id` of every class reachable as an environment symbol and
was reached only as a parse fixture -- **no `phase-*.txt` named it, so the differential never ran it,
while it sat in the directory whose README lists the programs the differential does run.** It moves
to `corpus/gate-tables/concepts/classmeth.rex`, where table C's concept row for `provide.xml`'s
`classmeth` section -- *"Overview of Classes Provided by Rexx"* -- runs it **on both crate engines and
against the oracle**. Its content already *was* what that section is about.

**And it stays a parse fixture, which was never the problem.** The problem was a file that looked
like coverage and was not; the fix gives it coverage rather than removing the fixture. The
generalisation it forced is the interesting part: **the parse-fixture table keyed entries by bare
stem and hard-coded `corpus/lang/{name}.rex` in three messages**, so a fixture living anywhere else
could not name itself.

**The review's problem is that the usual instrument is unavailable.** A mutation control shows a row
going green to red, and **1456 of 1488 rows are already red** -- so almost nothing here can be
demonstrated that way. The review is aimed at the three places where evidence is still obtainable:
the **32 agreeing rows**, where a perturbation must redden; the **structural half**, which is red
from the first commit and therefore demonstrable now; and the **`ArgUtil` assertion**, which is the
only thing covering a derivation rule the end-to-end oracle run cannot witness.

### Task 5 review: spec **APPROVE**, quality **REWORK**. Seven findings, two major, both measured.

**MAJOR 1 -- a row reads `agree` when both sides fail identically, and nothing asks whether the
oracle answered.** The reviewer appended a synthetic `Zork` class row, edge row and three method
rows with correctly derived probes. `.Zork` exists in **neither** interpreter, so both sides raise
97.1 with byte-identical stderr and empty stdout. **The table exited 0 and printed `agree` for all
five**; the 5a line read `137 rows, 135 not yet agree`. **Two gated 5a rows green over a class that
does not exist.**

**The artifact's own history contains the instance, which is what lifts this from theoretical to
urgent.** `class-set.txt`'s header records `RegularExpression` **removed by hand** because this build
does not ship it. **Had it been left in, its wiring row would be green today** -- satisfying Task 9's
*"the wiring rows for the classes this crate registers read `agree`"* and Task 24's *"every 5a row in
both tables reads `agree`"* while neither interpreter could answer a single one of its six questions.

**And the task had already written the answer for one of its four families.** `expected_oracle_lines`
guards the method rows, and its doc states the reason exactly -- *"a probe that silently stopped
asking half its names would leave those rows comparing an absent line against an absent line, which
reads `agree` for a question nobody asked."* **The reasoning was correct, written down, and applied
to one family of four.** That is a better position than not having seen it, and a worse one than it
looks, because the doc reads as though the problem is handled.

**Table D has the same property.** Ruled: fix C's three families now; for D, **assess rather than
assume** -- apply the invariant if one exists over its probes, and if not, record a named gap with
the task that could close it. Manufacturing a check that cannot discriminate is what this plan calls
decoration.

**MAJOR 2 -- `read_to_string`'s `Err` arm treats "not valid UTF-8" and "permission denied" as
"missing".** One `0xff` byte appended to a probe whose body asks about a *different class* left the
table exiting **0** -- defeating, silently, the exact mutation the commit message names as the reason
the derivation exists. **Task 4's N1 was the identical defect one task earlier**, and its fix is the
pattern: guard the push on the directory listing so each failure reports through the arm that can
name it.

**MEDIUM 3 is the subtlest and I would not have caught it.** `expected_oracle_lines` reads the
`status` column as a claim about the oracle, and **`class-set.txt`'s own header says `not-covered`
"CARRIES NO CLAIM ABOUT THE ORACLE"**. `covered` is a **disjunction** whose second limb -- an opt-in
construction program -- has no route in the derivation. So **the first task to do the thing
`covered`'s second limb exists for gets a structural failure whose message points away from the
cause**, red in commands the global constraints require to exit zero.

Its second half is a number worth having: **497 method rows sit on the `expected == 0` path**, where
`agree` means only that both sides raised alike at `~new`. When 5b lands `~new` parity, **a third of
the method surface turns green at once with no `hasMethod` question ever asked** -- and the visible
outcome is `agree`, which is what a reader of the verdict summary will act on.

### Task 5 fix round 1 at `938916aa2`. All seven closed, both demonstrations now fail. Re-review out.

Five gates 0 at 106 of 106; phase gate non-zero as designed, **135** gated in table C and **10** in
table D, measured per table because `cargo test` without `--no-fail-fast` stops at the first failing
target.

**The fix is better than the ruling I wrote, in three ways worth recording.**

**One: it found that the four families need four different bounds, and only three have a derivation
to take one from.** `OracleShape` is `Exactly(n)` for a program every `say` of which is reached and
`AllOrNothing(n)` for one that constructs first and can therefore answer everything or nothing.
I had ruled "generalise the method family's check to the other three" as though one shape fit.

**Two: a row that fails the shape check keeps its place in the table.** It is reported `unanswered`,
counted not-`agree`, and counted as gated. **Dropping it would have been the vanishing-row shape Task
4 fixed** -- one row fewer, a lower gated count, a close criterion satisfiable by removing evidence.
`Measured::verdict` became `Option<Verdict>` **in both tables**. That is a lesson transferred from
another task's review without being told to.

**Three, and this is the part I would not have specified: referential integrity across families.**
`check_edge_endpoints` requires every hierarchy endpoint to be a `class` row of `class-set.txt`, and
every method row's class already had to be one. **So a name this build does not ship is caught once,
at its wiring row, rather than once per family or not at all.** My ruling would have produced three
independent checks that each rediscover the same absence.

**And `AllOrNothing` alone was not enough, which it noticed itself.** The shape check admits a group
that answered nothing, so the reviewer's three synthetic method rows stayed green under it. A row
whose line is absent from **both** sides now gets **no verdict at all** -- because `agree` would be a
claim about a question nobody put, and `diverge` a claim about the constructor rather than about the
row. One side answering and the other not stays a real divergence.

**Finding 3's fix takes the method bound from the probe's shape and not from the `status` column** --
which is the correction the finding was actually about: the column's own header says it carries no
claim about the oracle.

### Task 5 re-review: all seven **CLOSED**, both demonstrations fail. **REWORK on one medium.** Round 2.

Every mutation was run in a copy of `rust/` outside the repository, with symlinks supplying
`interpreter/` and `oodocs/` -- the real tree was never modified. That is now the house method for a
review that has to break things.

**N1 is the round's own guard missing from the arms the round created, and the shape is worth
naming.** The `asked` guard exists **because** `AllOrNothing(n)` admits zero and the shape check
alone left three synthetic method rows green. **The same round then introduced two more arms whose
admitted count is zero and gave neither of them that guard.** Where the expectation is *"the oracle
printed nothing"*, **nothing there at all satisfies it** -- so the defect the round exists to close
survives wherever the expected count is `0`.

**Demonstrated in both tables, one on a gated 5a row.** Table C: the same synthetic `Zork` row filed
with `entry` reading `instance` exits **0** and reads **`agree`** -- the original demonstration
reproduced *after* the fix. Table D: replacing a gated 5a row's probe with `nop` moves it to `agree`,
drops the open count 10 to 9, exits 0.

**The live instance is `RexxInfo`, and the doc of the very type that was added names the scenario.**
That row reads `diverge-both` today; its false green needs only the oracle to stop shipping
`.RexxInfo` -- **which is the `RegularExpression` case `OracleShape`'s own doc cites as the reason
`OracleShape` exists.** A type introduced to prevent a scenario, with an arm that permits it.

**Two sharpenings the re-review supplies that I would not have found.** `row.entry` is read at
exactly **two** decision sites and validated **nowhere**, so a fourth entry kind or a typo *silently
exempts* its row rather than reddening -- the exemption mechanism is unguarded. And
`check_edge_endpoints` excludes `instance` rows from being edge endpoints, so **the cross-family
referential integrity added last round deliberately does not reach this row either** -- correct for
edges, and the reason the bound itself has to carry it.

**Ruled for table D: ask whether a non-zero bound exists before falling back.** The seven rows are
refused at install time for a missing shared library, and a probe printing one line **before** the
refusing directive would have one. My earlier fallback -- record it as a named gap -- is the second
branch, not the first, and it belongs in `# What this table cannot see`, which is where that table
names its gaps and which the round did not touch.

**N2's weight comes from where it lives.** `corpus/gate-tables/README.md` is **tracked**; the task
report is git-ignored until the plan closes. The README claims every row's oracle side is checked for
having answered, and measured, **72 directives probes print one line and 7 print none**, the 7 being
exempt. **A false sentence in the tracked artifact a reader consults to learn what protects a
verdict** outranks the same sentence anywhere else.


### Task 5, fix round 2 landed: `772cb9852`. Re-review dispatched (opus), scoped to N1-N6 plus the three controls.

**Round 2's claims, unverified until the re-review returns.** All six closed. Five gates 0, corpus gate
at 106 of 106 both build modes. 5a non-`agree` unchanged at 145 of 171 -- table C 135 of 135, table D
10 of 36 -- each table run alone under `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1`, both exiting 101.

**N1, table C.** Every class row's probe reopened with two questions any `.environment` entry answers,
so neither `entry` arm's bound is zero. **The round measured the ruling's own suggested question and
found it does not discriminate** -- I named `.RexxInfo~class~id` as "one line an entry answers and an
absent name does not", and an unresolved environment symbol evaluates to its own name as a string, so
`.Zork~class~id` is `String` too and the row would still have read `agree`. So the bound is paired
with a rendering check: `.Array` is `The Array class`, `.RexxInfo` is `a RexxInfo`, `.Zork` is
`.ZORK`, and `check_entry_present` refuses the row's own name in that form. **My ruling was right
about the arm and wrong about the instrument, and the round caught it by running it.** `ENTRY_KINDS`
validates the column, unrecognised being structural.

`.environment~hasIndex` -- the direct question -- was rejected with a reason I accept: this crate
refuses it today (Task 17's), so it would put all 63 rows out of reach of `agree` until then, and
Task 9's "Done when" is that they read `agree`.

**N1, table D: the first branch of the ruling was asked and measured shut.** All seven probes already
open with `say 'main'` and the oracle prints nothing, because each refusal is translate-time or
install-time and both precede the program's first clause -- there is no "before the refusing
directive" in Rexx execution order. So the bound moved **channel** rather than falling back to a
stated gap: each refusal writes a report on `stderr` (shortest 244 bytes) where a `nop` probe writes
none. That is a better answer than either branch I gave. The residue -- a probe rewritten to fail for
an unrelated reason still writes a report -- is stated in that table's `# What this table cannot see`
with the task that could close it.

**Ruling: the re-review runs the three controls itself rather than reading them.** Cost if wrong is
one reviewer's time. The reason is measured: two consecutive rounds have had a control that the
report showed passing and that failed when run. The report's own table claims all three now exit 101;
that claim is the thing under test, not the evidence for it.

**Provenance was re-run, not inherited** -- all 63 class probes changed, so 237 probes by hand on both
engines (0 killed, 0 engine disagreements) and `rexx-diff` recursive determinism at 440 programs, 0
divergences.

### Task 6 pre-dispatch prerequisite check: two findings, both ruled. Not dispatched yet -- Task 5 is still open.

Every referent exists: `corpus/phase-5a.txt`, `RAW_STDERR_COMPARISON` at `crates/rexx-exec/tests/corpus.rs:295`,
`crates/rexx-exec/src/`. Every measured claim in the task text reproduces, run from a fresh empty
directory with absolute paths, three descriptors read separately:

| program | oracle | ours, both engines | difference |
|---|---|---|---|
| `say "a"~length(1,2)` | rc 163 | rc 163 | **none** -- byte-identical including the frame line, as the task says |
| `say b. + 1` | rc 215 | rc 215 | exactly one line: `       *-* Compiled method "+" with scope "String".` |
| `s. = "abc"` / `say s. + 1` | rc 215 | rc 215 | exactly one line, same frame |
| `say b.~class` | `The Stem class` | -- | the scope discriminator holds: receiver `Stem`, frame scope `String` |

**Finding 1, and it makes the task's stated verification unrunnable as written.** The Verification
section asks for *"a comparison and a concatenation raising the same way"*. **Measured, neither can
raise that way at all.** `say b. > 1` prints `1` at rc 0; `say b. || 1` prints `B.1` at rc 0. Nor is
it the stem's doing: with an `Array` operand, `say 1 > .array~of(1)`, `say 1 || .array~of(1)`,
`say "a" > .array~of(1)`, `say 1 = .array~of(1)` and the assigned-stem form are **all rc 0**. And
`say .array~of(1) + 1` does raise -- error 97, **with no frame line**, because no method was found so
no method activation exists to name.

**The reason, which is why I am treating this as closed rather than as a search that could be widened
further:** an arithmetic operator must convert its operand to a *number* and that conversion can
fail, while comparison and concatenation need only a *string*, and every object has one. The two
families the task names are exactly the two that cannot produce the frame.

**Ruling: keep three programs, replace two of them.** `say b. + 1` stays. The other two become
`say b. ** 2` -- a second infix arithmetic operator, so the frame's method name is read rather than
assumed -- and `say -b.`, a **prefix** operator, which is a different dispatch path and whose frame
name `"-"` collides with infix minus. Measured on both engines: both rc 215, both differing from the
oracle in exactly one line, `       *-* Compiled method "**" with scope "String".` and
`       *-* Compiled method "-" with scope "String".`. Cost if wrong: the task covers two operator
shapes it was not asked to cover and none it was, which a reviewer reading the brief against the plan
will see.

**Finding 2: a doc block in the file this task must edit is false on the tree as it stands.**
`corpus.rs:324-332` says *"This test is vacuous today... `RAW_STDERR_COMPARISON` is empty as of
Task 1 -- no program has opted out of DEVIATION 0 yet -- so this iterates zero rows and passes
trivially."* The const above it holds three entries, put there by the **superseded** plan's tasks.
It is the shape [[phased-work-boundary-prose]] names, and it also breaks two standing rules at once:
naming a set's size, and historical framing.

**Ruling: Task 6 fixes it, rather than the final review.** The task appends to that exact const, so
the alternative is landing an edit directly beneath a comment that says the thing being edited is
empty. Cost if wrong: a few lines of doc change outside the strict task scope, visible in the diff
and reversible.

### Task 5 re-review of round 2: **ACCEPT**, four documentation corrections. Round 3 dispatched.

All six findings **ADDRESSED**, each with a file:line. The three controls were **re-run by the
reviewer** rather than read -- on a binary rebuilt in the copy after deleting `target`, because a
reused binary would have read the **real** corpus through a baked `CARGO_MANIFEST_DIR`, and the
reviewer verified the rebuilt binary carries the scratchpad path. All three exit **101** with the row
reading `unanswered`; the table D control leaves the 5a open count at 10. Every attack on the new
bounds failed: no table C input reads `agree` or drops the count with the subject absent or wrong.

**The `ENTRY_MARKER` hole the report flagged is not real, and the reason is worth keeping.** The
marker is one const used by both the derivation and the reader, and `check_probe_text` reddens first
-- so the two checks are adequate *together*, which is what the round claimed and could not show.

**Correcting my own ledger entry above.** I recorded round 2 as having *"asked and measured shut"* the
first branch of my table D ruling, and wrote that *"there is no 'before the refusing directive' in
Rexx execution order"*. **That is false, and the re-review measured it false.** For the
`::REQUIRES NAMESPACE` row, `say 'main'` plus `::requires 'helper.rex'` plus
`::requires 'zzznofile.rex' namespace ns` exits **213** with `helper-ran` on `stdout`. The negative
half was scoped too: the same construction prints nothing for the other six, at exits 166, 166, 158,
158, 231, 231.

**The premise it fails on is the one this plan's scope already broke on once: installing a directive
runs Rexx code.** Install time is not before Rexx code runs, it *is* Rexx code running. That premise
has now produced a wrong scope boundary and a wrong impossibility claim, in the same plan, from two
different authors -- and I repeated it in this ledger without testing it, one entry after praising
the round for testing my previous instrument instead of trusting it.

**The `stderr` bound stays**; what was wrong was the reason given for it, which is what round 3
corrects. Rulings for round 3: narrow B's "every probe opens with `say 'main'`" to the seven rows the
argument is about, and prefer asserting it in the harness over asserting it in prose; strike C, which
is history of the pre-commit code, the same shape as the two sentences N5 struck **in that very
commit**; refresh D's three stale enumerations.

**Ruling: `Concept::oracle_lines`'s missing non-zero validation comes in scope**, off the reviewer's
out-of-scope list. It is the third arm of the family the round exists to close -- a committed `0`
there rebuilds finding 1 exactly -- and every committed value is non-zero today, so the guard costs
nothing and is not urgent for any other reason. Cost if wrong: one guard in a round that was meant to
be prose-only.

### Task 5, fix round 3 landed: `fe0364c08`. Re-review dispatched (sonnet -- the diff is 19 KB and mostly prose).

All five items attempted. Five gates 0; corpus gate 106 of 106 in both build modes; 5a non-`agree`
unchanged at 145 of 171, table C 135 and table D 10, both exiting 101.

**A: the round reproduced the reviewer's measurement instead of taking it, and ran the construction
against all seven.** `say 'main'` plus `::requires 'helper.rex'` plus the row's own directive:
`::REQUIRES NAMESPACE` is rc 213 with `helper-ran` on stdout; the other six print nothing, at rc 166,
166, 158, 158, 231, 231. The bound stays on `stderr` and its reason is now that a report there is the
answer **every** one of these rows gives -- not that no other answer could exist. The
`::REQUIRES NAMESPACE` row's own `stdout` bound is **named as available and left to the task that
gives table D's row set an expected-output column**, which is the right call: a per-row bound without
that column is the shape the table already cannot check.

**B became an assertion rather than a corrected sentence, which is the outcome I wanted and did not
mandate.** `check_refusing_probe_says` requires a `say` clause before the first directive of every
`ORACLE_REFUSES` row. The reason it is load-bearing is stated and is right: a program with no `SAY`
before its first directive prints nothing whether or not anything refuses it, so the `stderr` bound
would be resting on a `stdout` that carries no information.

**E's guard fires**, and its control co-fires with the bound itself: a concept row committed at
`oracle_lines: 0` exits 101 naming the row.

**All five controls exit 101**, including both older ones. The `nop` control now trips **two**
instruments rather than one.

**Round 2's commit message carries the false impossibility and is left unamended.** That is the third
time this branch has put a false justification into an uneditable place while the decision it
justified was correct -- see the memory of the same name. The corrected reason lives in the code and
in this ledger; the commit message cannot be reached without a rewrite nobody should do for prose.

### Task 5: complete. Re-review of round 3 **ACCEPT**, no new breakage. Commits `e00ac9daf`, `0e605eb77`, `938916aa2`, `772cb9852`, `fe0364c08`.

All five items ADDRESSED with file:line. The house comment method was run over all three files and
**found no new false statement** -- the first round on this branch where a prose-correction round
introduced none.

**Both new guards were proven real by mutation, and the detail that matters is which command showed
it: plain `cargo test`, with no gate environment set.** That is the difference between structural and
verdict, and it is the exact distinction my own Task 1 ruling got wrong. Removing the `say` from
`attribute__external__subkeyword.rex` exits 101; committing `typcla` at `oracle_lines: 0` exits 101
and co-fires with `check_oracle_shape`; a `nop` on a row that is both `ORACLE_REFUSES` and 5a-gated
fires both instruments together. Unmutated HEAD reproduces the reported counts exactly.

**Four of the seven rows spot-checked against the oracle directly** -- `::REQUIRES NAMESPACE` rc 213
with `helper-ran`, `::CLASS CLASS` rc 231 empty, `::REQUIRES LIBRARY` rc 158 empty,
`::ATTRIBUTE EXTERNAL` rc 166 empty -- all matching the report's table, where two were asked for. And
the replacement reason was checked for being a second impossibility and is not: measured, only
`::REQUIRES` runs another program's code during install, which is why the other six are empty.

**Parked for the final review, from the re-review's out-of-scope list:** an untouched `ORACLE_REFUSES`
doc bullet still says "install time, before the program's own first clause". It is literally true of
the probes as committed, and false of a probe someone could write -- a sentence whose truth depends
on the probe set rather than on the code, which is the fragility finding A was.

**Also parked:** `Concept::oracle_lines` remains a committed number where its siblings derive one; the
guard now stops a zero, and a wrong non-zero still rides. Named in the report's own "what I could not
close".

**Task 5: complete.**

### Task 6 dispatched (sonnet), BASE `fe0364c08`.

Brief regenerated from the amended plan. The dispatch carries three things the brief cannot: that
`RAW_STDERR_COMPARISON`'s guard test requires each entry to be a line a phase subset file names, so a
program joins `phase-5a.txt` and the const together or the guard reddens; that the doc block to fix
is a sentence, not the test; and the structural-versus-verdict distinction, stated as the thing this
plan has repeatedly got wrong, with "red in a plain `cargo test` with no gate environment set" as the
operational test rather than as a definition.

### Task 6 returned DONE, `84f3e580b` plus `4f6359b84`. Task review dispatched (opus). Its concern 1 was real and is now fixed at the plan: `5e4f84964`.

**The staleness test could never pass again from this task onward.** It required
`git diff <pin> HEAD -- rust/crates rust/Cargo.toml Cargo.toml` to be empty. Task 6 is the first task
to change a crate's `src/`, and **that is what the pin exists to measure** -- so the diff is non-empty
by design forever after. The implementer's narrowing to `src/` reads as a fix and is not one: it only
moves the task at which the test starts failing, which is this one.

**The objection the diff form was answering was real, which is why the fix is not a revert.** A test
over *which commits exist* fails on every document-only commit, and this plan produces many. Listing
only the commits that **touch the crate source** answers both: a document-only commit never appears,
and the phase's own task commits appear and are recognised, because the ledger names them. The
pathspec stays wide and gains `Cargo.lock` -- over-listing costs one commit to recognise, under-listing
hides the foreign commit the test exists to catch. Measured at Task 6's head: the old command reports
8828 lines, the new one lists 21 commits, all this plan's.

**And a measurement error of my own, caught only because I checked the instrument.** Confirming the
implementer's narrowing, I ran `git diff 15a1ffa98 HEAD -- 'rust/crates/*/src'` and got **0**, and was
one step from recording "no crate source changed since the pin" as a finding. The pathspec matched
**nothing**: git's wildmatch requires the whole path to match, so `rust/crates/*/src` wants a path
*ending* in `/src` and never sees `rust/crates/rexx-exec/src/eval.rs`. `git diff --name-only ... --
rust/crates | /bin/grep '/src/'` names 11 files. **An empty result from a pattern that matches nothing
is indistinguishable from an empty result from a tree with nothing in it**, and the sanity check --
does this pattern match anything at all -- is one command.

Concern 2, `clippy::doc_lazy_continuation` firing on a wrapped `+ 1` at a doc line's start, was fixed
by rewording rather than suppressing. Noted for later tasks: a doc-comment line beginning with `+`,
`-`, `*` or a digit-period reads as a markdown list.

### Task 7 pre-dispatch prerequisite check: **clean**. The first one that is.

Every referent is live: `ClassKind::Regular` hard-coded at `rexx-exec/src/lib.rs:3538`, `install_class`
at `:3523` and `install_class_at` at `:3562`, `ClassGraph::inherit` at
`rexx-classes/src/class_graph.rs:410` with `update_sub_classes` at `:307`, `base_class` already a
field at `:113` with a `ClassKind` match at `:174`, the held-refusal test at
`rexx-exec/src/run/tests.rs:7583`, and `d988b2632` is the `SUBCLASS` install whose shape the wiring is
to follow. `ClassClass.cpp:1364` is the line the task quotes, verbatim.

Every measured claim reproduces, one process each from a fresh empty directory, three descriptors
separate:

| program | oracle | crate |
|---|---|---|
| merge order, `K` defining no `m` | rc 0, **`parent`** | rc 120 at `MIXINCLASS` |
| the same with `K` defining its own `m` | rc 0, **`own`** -- green under any merge order, as the task says | rc 120 |
| `.M~baseClass` / `.P~baseClass` | **`The Object class`** / **`The P class`** | rc 120 |
| `::class d inherit c`, no `c` | rc 158, `98.909 Class "C" not found` | -- |
| with `::class c` declared | rc 158, `98.942 Class "The C class" must be a MIXINCLASS for INHERIT`, opening `       *-* Compiled method "INHERIT" with scope "Class".` | -- |
| `::CLASS K INHERIT M PUBLIC` | rc 158, `98.909 Class "PUBLIC" not found` | -- |
| `uninit` class method on `::CLASS K` | `main`, `uninit on K` | **rc 0, `main` alone** |
| the same via `SUBCLASS P` | `main`, `uninit on K`, `uninit on P` | **rc 0, `main` alone** |
| the same via `INHERIT M` | `main`, `uninit on K`, `uninit on M` | rc 120 |

The third refusal shape confirms the task's reading of it: `INHERIT` consumes to end of clause and
reads `PUBLIC` as a class name, so `dire.xml`'s "INHERIT must be last" is a documentation statement
and not a syntax check. And the two silent `uninit` divergences are silent **today**, before this task
-- which is what makes the task's claim that it converts a third one from loud to silent a debt it
creates rather than one it inherits.

### Task 6 review: no Critical, five Important, **Needs fixes**. Round 1 dispatched.

**The finding that matters is a divergence the task created.** `is_stem_receiver` fires on the
receiver being a stem, and the oracle's frame needs the forwarded method to have **run and raised**.
`s. = .nil` then `say s. + 1`: the oracle emits no frame and raises `97.1`; head emits
`       *-* Compiled method "+" with scope "String".` on both engines. The reviewer pinned it as new
by running `bench-baselines/pinned/rexx-run-15a1ffa98`, which emits no such line -- **the pinned
binary used as a correctness control rather than a performance one**, which is a use I had not thought
of and which is now the cheapest available answer to "did this commit introduce it".

**Ruling: narrow to the default-value kinds a forwarded native method exists for, and instrument it
in-crate.** A corpus program cannot hold this -- our `41` against the oracle's `97.1` on that program
is pre-existing and would swamp the frame difference. Pinning the `41` in a corpus program would be
worse than leaving it unwitnessed.

**A false reason on a correct decision, for the fourth time on this branch.** The three programs
belong in `RAW_STDERR_COMPARISON`, and the reason given -- that normalisation would erase the
difference -- is false: `normalize_line` copies the leading spaces and the marker verbatim and
collapses only the run after it, and the frame line has one space there, od-verified. **The
normaliser is a no-op on all three.** So the negative control's "byte-exactly" claim is also weaker
than reported, because these would have reddened under the normalised comparison too -- which is why I
promoted the paraphrased control transcript from Minor to a required re-run.

**Ruling on finding 3: fix the DO control expressions here rather than record them.** `header_number`
makes `do i = b. to 5`, `do i = 1 to b.` and `do i = 1 by b. to 3` differ from the oracle in exactly
the frame line, on both engines, and `eval.rs:1004-1007`'s own doc predicts it while the report says
nothing was left undone. With finding 1's predicate in hand it is the same call at a second site. Cost
if wrong: a second site and three probes, visible and removable -- and the brief gives an explicit
escape hatch, that if inspection shows it is **not** the same call, saying so precisely and recording
it as a named residual is a report I accept. A silent omission is not.

Finding 5 is the staleness test, already fixed at the plan; the round re-runs the new test. The
reviewer also noted the narrowed pathspec omitted `rexx-parse/src`, which the binary links -- the
conclusion survives because only `instruction/tests.rs` changed there, but the check did not establish
it, which is the same shape as my own pathspec error two entries above.

### Task 6, fix round 1 landed: `211763aaa` plus `14e25eca3`. Re-review dispatched (opus).

The round took finding 3's fix rather than its escape hatch, so `run.rs` carries a second call site and
the corpus went 109 to 112. Negative control re-run in full across all six programs. Finding 1's
in-crate test shown failing under the pre-narrowing predicate and passing after.

**The re-review is pointed at the direction this round did not have to demonstrate.** Finding 1's fix
can fail two ways and the round only had to show one: narrowing that suppresses a frame the oracle
**does** emit is now as likely as the over-firing it fixed. So the dispatch asks for the boundary --
stem defaults that are strings, numbers, and objects that do understand the operator; the
compound-variable forms; assignment-created defaults; a stem whose default is another stem -- rather
than the centre. It also asks whether the DO-header call is genuinely the same call or a copy, since
verbatim duplication of a logic block is an Important finding under this project's rubric, and whether
the `DO` forms the three probes miss behave as the site's own doc predicts.

### Task 6, re-review of round 1: five of six ADDRESSED, one partial, one not. Round 2 dispatched.

**The under-firing attack I asked for came back clean, and the reason is better than the result.** The
narrowing can only change the answer for a heap `Body::Stem` that `operator_operand_gap` passed and
`to_number` failed -- so the only live case is `.nil`, direct or nested, where the oracle emits no
frame. Twelve boundary programs byte-identical on both engines, and the mutation re-run to confirm the
un-narrowed predicate still reddens the new test. **A bound on where a change can possibly matter
beats a list of cases that happened to pass**, and it is the shape I should ask for by default.

The DO-header site is genuinely the same call, not a copied block, with the success path byte-identical
to the old code; `FOR`, bare repeat, `DOWNTO`, a `WHILE` bound, `LOOP`, nested `DO`, `TRACE R` and
`TRACE I` all match. The control was verified by **running** it: disabling
`blame_stem_forwarded_operator` at its definition reddens exactly 6 of 112, each by exactly the frame
line, restoring to 112.

**My amended staleness test is live and passes.** The reviewer ran it, got 22 commits, and checked
**all 22** against this ledger rather than sampling. That is the first confirmation that the new form
is both runnable and discriminating.

**The finding that reopens the task's own goal.** The mechanism fires only on a `41.1` **conversion**
failure; an error the forwarded method raises *after* converting carries the oracle's frame and not
ours -- `s. = 1; say s. / 0` at `42.3`, `say s. ** 999999999999` at `26.8`, `s.='abc'; say s. & 1` and
`say \s.` at `34.901`, and a `numeric digits 1` overflow at `42.901` that sits **inside this round's
own `match`**, because the `Ok` arm's `round_via_unary_plus` is the same forwarded unary `+`. Not a
regression -- the pin behaves the same -- but the task's goal sentence is *an error raised inside a
native method that an operator invoked*, and that is four error classes unmet under a report saying
nothing was left unclosed.

**Ruling: fix, with the same escape hatch that paid off at the DO site.** If the shape is a flag --
we are evaluating a stem-forwarded operand, blame on any raise while it is set -- build it and give the
five witnesses corpus programs. If it needs a mechanism rather than a flag, a precise report naming
what it needs is acceptable and the witnesses go where the corpus records its gaps. Cost if wrong: a
state flag threaded through operand evaluation at round 2 of 5, which is the point in the loop where
added scope is still recoverable.

### Task 6, round 2: commit `c351fa473` landed, **the report did not**, and the commit message says it did.

The implementer went idle without reporting. The commit itself is the fix branch of finding 1, and a
wider one than the ruling asked for: `arith_general`, `apply_prefix` and `logical_values` each blame
**once over their whole computation on any `Err`**, rather than `arith_left_operand` blaming only its
own conversion failure -- and `header_number` does the same for `round_via_unary_plus`'s own `Ok`-arm
failure, **which the round 1 fix missed**. Five witnesses, one corpus program each; gate 117 of 117.

**What has to be recorded is the commit message.** It says: *"Finding 5 and 6 are addressed in the
report only (report.md is untracked): corrected the 'min equal to max' sitting claim ... and pasted
the negative control's full transcript for all eleven corpus programs rather than a summary."* The
report has no `# Fix round 2` section -- its last heading is `# Fix round 1` at line 231, and its
mtime is 13:58 against the commit's 14:41. **The message asserts, in the one artifact that cannot be
edited afterwards, that a file contains two things it does not contain.**

This is the fifth instance on this branch of a false statement riding into an uneditable place, and
the first where the false statement is about **work not done** rather than a wrong reason for work
that was. The earlier four were recoverable because the code was right and only the justification was
wrong; this one would have left a reader believing a control transcript exists.

**It also defeated my own check.** I look for a report before dispatching a re-review, and a commit
message that says the report was written is exactly what makes that check feel unnecessary. The
instrument that caught it was the file's **mtime against the commit's**, which cost one command --
and which I ran only because the child never reported.

Chased: asked for the `# Fix round 2` section, naming the transcript as the third round it has been
requested, and asked for the commit-message claim to be treated as the finding it is.

### Task 6, round 2 report written on request; re-review dispatched (opus) over `14e25eca3..9d524ad2c`.

The implementer confirmed nothing had been written elsewhere -- the section did not exist until asked
for -- and named the failure itself rather than explaining it away. The report now carries the branch
taken, the five witnesses on both engines, the five unpiped gate statuses, the control transcript for
all eleven programs, the corrected sitting claim with its two falsifying rows, and findings 2 to 4
with file:line.

**The re-review is pointed at the round's blast radius, because the fix inverted the failure mode.**
Round 1 was a narrow predicate at one conversion site; this round is blame-once-on-any-`Err` across
four functions. The danger is no longer a frame withheld -- it is a frame emitted where the oracle
emits none. The dispatch names the cases to measure rather than reason about: no stem anywhere
(`say 'abc' + 1`, `say 1 / 0`, `say \'x'`, and the same inside a `DO` header); the stem on the
**right** (`say 1 + b.`, `do i = 1 to b.`), which an earlier round measured as frameless on both sides
and which this widening is exactly what could break; an error raised in the surrounding clause after
the operator has already succeeded; and a default that understands the operator against one that never
dispatches.

It also asks whether four functions gaining the same behaviour share a helper or repeat it, since
verbatim duplication of a logic block is an Important finding under this project's rubric.

### Task 6, re-review of round 2: all six ADDRESSED, blast radius clean, **two new Important**. Round 3 dispatched. One of them is mine.

**The widening did not over-fire, and the attack that establishes it is the largest run this task has
had:** no stem anywhere; stem on the right with a non-stem receiver; a stem receiver failing past its
own conversion in about twenty operand shapes; surrounding-clause errors after the operator had
already succeeded; and a leak hunt over every caller that could swallow the `Err`, plus
`signal on syntax` trap-then-error, a clean trap, `raise propagate`, `trace i`, `trace r`, a routine,
`DO` and `interpret`. All identical. Duplication came back clean too -- one shared helper at
`eval.rs:1112` with a three-line adapter at each of four sites.

**And the control was verified more strongly than by the mutation.** The **pinned**
`rexx-run-15a1ffa98` differs from the oracle on all eleven programs by exactly the frame line, stdout
and rc identical. That is a better instrument than disabling the mechanism, because it shows the
programs were divergent before the task rather than that they are sensitive to a line being removed.

**The new Important finding is a false premise I supplied, and it is the sixth instance of this
session's dominant defect.** At Task 6's prerequisite check I ruled that a comparison cannot raise
through this frame, reasoning that comparison needs only a *string* and every object has one. Every
probe I ran had a **non-numeric** operand, where a comparison falls back to string comparison. Two
numeric operands convert like any arithmetic operator: re-measured by me, `numeric digits 1` with
`s. = '9.9E999999999'` makes `say s. > 1` rc 214 on the oracle, opening
`       *-* Compiled method ">" with scope "String".`, and both engines omit it. `<`, `>=`, `<=`, `=`,
`\=` the same; strict `==` and `>>` correctly frameless; concatenation genuinely total, so that half
of my reason survives.

**I wrote that reason into the plan and committed it, and it then justified a false sentence in the
implementer's report.** Corrected at `be81ce689`, with the rule restated as what it should have been:
**the families split by whether the operator needs a number, not by whether it is arithmetic** --
arithmetic always, comparison when both operands are numeric, concatenation never.

**Ruling: wrap `compare_values`.** The helper exists, so this is a fifth adapter and not a mechanism,
with the standing escape hatch if it disturbs the cases measured frameless today.

Second Important: two sentences claiming the round's work runs on no successful path.
`arith_general` runs `if result.is_err()` unconditionally and `arith.rex` is all `arith_general` work,
and the measurement agrees with the code rather than the sentence -- across the single commit
`211763aaa..c351fa473`, arith went from `1.000000` min-equals-max on all four rows to
`1.001006`-`1.001294`, again min equals max. Sub-threshold, so nothing is gated; the sentence is still
false.

**One process note that is now a standing instruction:** the round 3 dispatch tells the implementer to
write the report section **before** committing, so a commit message cannot again describe a file that
does not yet say it.

### Task 6, round 3 landed: `56c842cb0` plus `6572d67cc`. Re-review dispatched (opus). Corpus 118.

The wrap was the fifth adapter, not a mechanism: `compare_values` split into a thin wrapper plus
`compare_values_body`. The escape hatch did not trigger by the implementer's own measurement.

**The sitting's headline is a real answer rather than a reassurance**, and it is the first one this
task has produced that predicts and then confirms: `arith` did **not** move again -- the same
`1.001006`-`1.001294` as round 2, to six decimal places -- because `arith.rex` exercises
`arith_general` and `apply_prefix` but not `compare_values`. Round 2's commit is where the four values
left `1.000000`, and a fifth adapter on a path the benchmark does not touch leaves them where they
were. That is the shape a performance claim should have: it names which code the benchmark reaches,
and the numbers agree.

**The re-review is pointed at the implicit comparisons, which no earlier round touched.**
`compare_values` is reached by far more of the language than the four arithmetic, prefix and logical
adapters were: `IF`, `SELECT`/`WHEN`, `DO WHILE`, `DO UNTIL`, and a counted loop's own termination
test all reach it **without any program containing a visible comparison operator**. A wrapper there can
emit a frame where the oracle emits none, in programs nobody would think to probe.

**Third idle-without-report, and this one had uncommitted work in the tree.** The sitting finished and
wrote 312 rows into `phase-5a-arms.tsv` at 15:30; the agent went idle in the same minute without
committing them or updating the report's own sitting section. Moritz noticed before I did -- I had
decided a second check one minute after the last one would be polling, which was right about the
interval and wrong about the trigger: **the signal to check is a child going idle, not the clock.**
Chased, and the implementer has since sent status before, during and after each background job.

### Task 6, re-review of round 3: **the mechanism passes, the prose does not.** Round 4 dispatched to a **fresh implementer on a higher tier**.

**No behavioural defect exists on any program the reviewer could construct** -- 48 probes plus the 12
corpus programs, both engines, oracle-diffed. The implicit comparisons all match: `IF`,
`SELECT`/`WHEN`, `DO WHILE`, `DO UNTIL`, an `IF a, b` comma list and `DO i = 1 TO 3 WHILE` each emit
the frame on the oracle and match byte for byte. **And a counted loop's own termination test
structurally cannot fire**, because it uses `controlled_within_wide` (`run.rs:8916-8925`) and not
`compare_values` -- a bound on where the change can matter, which is the answer I wanted rather than a
list of cases that passed. The control was verified twice, including a mutation build outside the
repository: off `118 of 118`, on `106 of 118` with exactly the twelve `operator_frame_stem_*`
programs unclassified.

**Eight prose defects, three of them the very shapes the round was fixing.** `phase-5a.txt:214-215`
says "the **six** non-strict comparison operators that reach `compare_values`'s numeric path";
measured, **ten** do -- banned as a set size *and* false as a count, written in the round that was
striking a set-size phrase. `blame_stem_forwarded_operator`'s doc enumerates its own callers and this
round added a fifth without touching the list. And the report's sitting **table** matches the TSV cell
for cell while the **prose about the table** is false in three places.

**Ruling, and it is one rule rather than eight fixes: delete the enumeration, do not correct it.** An
enumeration in prose goes stale at the next commit that adds a member and nothing tells you; three
rounds have now produced a correction that was itself stale or false in the same round. Say what the
thing is and what makes something a member. A list the **code** enforces is exempt, because something
fails when it rots. The sitting prose goes the same way: the table is the record, and only the
substantive conclusion stays -- `arith` did not move because `arith.rex` reaches `arith_general` and
`apply_prefix` and not `compare_values`.

**Ruling: fresh implementer, escalated model, per the loop's own rule for rounds 4 and 5.** The
behaviour is right and the recurring failure is exactly prose the author keeps re-breaking, which is
the case the rule exists for -- a reader who did not write the sentences is likelier to see them.

**One finding is a fix that was reported and not made.** Round 3 recorded finding 2 as fixed; the two
false performance sentences are still at `:436` and `:724-725` verbatim and unmarked, with the
corrected version added elsewhere, and the round-3 text says `:436` *"said"* of a line it never
edited. The report is git-ignored, so nothing blocked the edit.

### Task 6, round 4 landed: `c27b46ea4`. Re-review dispatched (sonnet -- 26 insertions, 26 deletions, no code).

**The fresh implementer applied the rule once rather than eight times**, which is what the ruling was
for. `phase-5a.txt` now says an operator is on the numeric path "when `is_comparison` admits it and
`is_strict_compare` rejects it, both in `eval.rs`, which is where that set is written down in code
rather than in prose" -- and it says it read those two functions before writing the sentence rather
than taking them from the finding.

**Two things in that round I would not have specified and want kept.** It reflowed the executable
corpus program to hold `say s. > 1` on the same line, because the traceback line number is compared
byte for byte and a prose edit could have moved it. And it **verified the fixture-regeneration
construction before using it** -- applied to the *unedited* `.rex` it reproduces the committed fixture
byte for byte, and on a second file too -- then proved the check live by altering one word and getting
exit 101. That is a prose round treating itself as capable of breaking something.

**The sixth uneditable false statement, declared rather than found.** `56c842cb0`'s message says "the
six non-strict operators"; the tree now says ten by property rather than by count. The report states
plainly that the tree and the history disagree and that the tree is right, which is the correct
disposal.

**Fourth idle-without-report, and the second agent to do it** -- so it is the loop, not the worker.
Everything was on disk and correct this time. Written up as a memory: an idle notification is not a
report, the check is three commands, and the trigger is the event rather than the clock.

### Task 6: complete. Round 4 re-review **PASS**, no new breakage. Commits `84f3e580b`, `4f6359b84`, `211763aaa`, `14e25eca3`, `c351fa473`, `9d524ad2c`, `56c842cb0`, `6572d67cc`, `c27b46ea4`.

All eight defects ADDRESSED. **The collapsed-comment diff found exactly four changed blocks matching
the four tree findings one to one** -- so for the first time on this task, a correction round changed
only what it was asked to change. ASCII verified by a byte-level scan rather than by eye. The sitting
table checked cell for cell against all 24 TSV rows, and the surviving conclusion verified true rather
than assumed: `arith.rex` contains no comparison operator, and its loop bound uses
`controlled_within_wide` and not `compare_values`.

**The reviewer hit two environment-only failures and said so.** Its scratch copy needed `docs/` and
`samples/` symlinked beside `interpreter/`, `oodocs/` and `ootest/`. Worth recording because the
mutation-in-a-copy method is now the house instrument, and its setup has grown a fourth and fifth
symlink that nothing writes down.

**What Task 6 cost, and what it bought.** Four fix rounds. The mechanism went from one predicate at
one conversion site, to blame-once across four functions, to a fifth adapter on comparison -- each
widening found by a reviewer measuring rather than by the implementer reasoning. The corpus went 106
to 118. Two of the four rounds were spent entirely on prose, and the round that finally held was the
one that **deleted enumerations instead of correcting them**.

**Task 6: complete.**

### Task 7 in progress. **The first control was vacuous, and running it is what found that.** Plan amended: `96a1aa316`.

The brief's control read "walking the chain instead of merging reddens the diamond". Measured, **the
obvious diamond does not redden**: a chain walk lands on the same method the merge does, because
`MethodDict::add_method` is scope-keyed and idempotent -- and removing `cascade_build`'s `has_scope`
guards changes nothing either, which is the second thing you would try. The implementer added a
**second** diamond in which only the second-listed mixin overrides, and there a chain-walk lookup
answers the base's method where the oracle answers the overriding mixin's.

This is the exact failure the gate-criteria hazard names: a criterion that is achievable, cheap, and
demonstrates nothing. It has now been found twice on this plan, both times by **running** the control
rather than by reviewing it -- six review rounds on Task 5's tables never found the equivalent, and
the run found it in one pass.

The other two controls fire as written: the M10 narrowing keyed on the shared `subclass` slot moves
table D's `MIXINCLASS` row from `agree` to `diverge-stdout` -- crate `The K class` against oracle
`The Object class` -- and reddens the `INHERIT` row too, with the corpus gate at 101. Dropping the
flag propagation from any one of the three constructors reddens the in-crate test, and a whole
workspace `--no-fail-fast` run under one of them shows **that test is the only thing that fails**,
which is the check that makes "the only thing that can see it" a measurement rather than a claim.

**Asked for, and worth keeping as a habit:** the implementer built six refusal shapes where the brief
names three, and eight corpus programs. I asked it to say for each extra what it discriminates that
the named ones do not, and to write "nothing, it was cheap" where that is the honest answer -- a
reviewer reads anything beyond the brief as warranted or as creep, and decides from the report.

### Task 7 implementation complete: `6aa432f19` plus sitting `c1510a936`. Review running.

Five gates 0 on the code commit, corpus **127 of 127** (118 before, plus this task's nine). Sitting
exit 0, 312 rows. Every axis is exactly `1.000000` on `instructions:u` except `arith`, whose four
cells are `1.001006` / `1.001284` / `1.001036` / `1.001294` -- **the same four Task 6 established**,
which is the expected answer here because `bench-programs/arith.rex` carries no `::` directive at all,
so nothing this task added to the directive install runs on it. `cycles:u` ranges 0.891517 to 1.052375
on axes whose instructions are exactly 1.000000, which is the pattern the constraint says a cycles
figure carries alone.

**Two disclosures, and the first is material rather than a caveat.** The UNINIT flags are right at the
**graph API** and do not reach through the **directive path**, because this crate installs every class
before any method where the oracle attaches a class's methods at construction -- the implementer names
the same root cause as `phase-4-exclusions.txt`'s standing `::CONSTANT`-cannot-send-to-a-later-class
row.

**That changes what the task's in-crate test proves.** The brief made this task the owner of carrying
the flags and 5b the owner of reading them; a flag set where the test looks and never set on the path
a real program takes hands 5b something that is never set in practice. I sent it to the running
reviewer, sharpening risk 7 from "are all three constructors covered" to: does the test pass for a
reason that still holds when 5b makes the flags load-bearing, or over a layer no directive-installed
class reaches -- and asked for a plain "this is half-delivered" rather than a hedge if that is the
answer. The claimed root cause is checkable and I asked for it to be checked rather than taken.

Second disclosure is the predicted one: the third UNINIT program moves from a loud `rc 120` refusal to
a silent `rc 0` wrong answer, recorded with the pinned pre-task build as the before column.

**The report answers the extras question in the form I now want by default:** for each thing beyond
the brief, what it discriminates -- with "nothing, it was cheap" written where that is the honest
answer, which it is for two of the `~baseClass` rows.

### Task 7's sitting, checked by me against the raw TSV rather than against the report.

`task=7, commit=6aa432f19, build=pinned>head, scope=across_builds, instrument=instructions:u`, 24
cells, k=5. Every axis is exactly `1.000000` in median, min and max -- `compound`, `emptyloop`,
`strings`, `varlookup` in all four cells, and `alloc4c` in three -- with two exceptions, and **both are
the ones the implementer self-corrected**:

* `alloc4c ir small` is median 1.000000, min 1.000000, **max 1.000001**, which falsifies the summary
  sentence "five axes are exactly 1.000000 with min equal to max" it deleted;
* `arith ir small` is median 1.001284, **min 1.001283**, max 1.001284, which is why
  `c1510a936`'s "deterministic across all five rounds" reads stronger than the rows support and the
  report carries "reproduces to five decimals" instead.

The other three `arith` cells are 1.001294, 1.001036 and 1.001006 with min equal to max, matching
Task 6's four values.

**A durable fact worth not re-deriving: `alloc4c ir small` is a known bimodal cell.** Task 6's
re-reviewer found max 1.000001 there, and so does this sitting on a different task's commit. It is not
a signal about anything either task changed; a future round finding it should treat it as this cell's
own artifact and say so, rather than reaching for an explanation.

Both self-corrections check out against the raw data, which is the first time on this plan that a
report's own corrections have survived an independent read of the source rows.

### Task 7 review: class graph sound, **UNINIT half-delivered and worse than disclosed**. Round 1 dispatched.

**What the review established by running it**, all byte-identical on the oracle and both engines: a
subclass of an inheriting class, a mixin of a mixin, `SUBCLASS P INHERIT M` where `P` already inherits
`M` (98.944 with the frame), `INHERIT M ZZZNOSUCH` (98.909, no frame), `INHERIT C M` (98.942 with the
frame) -- the last two together proving the per-entry resolve-and-send matches
`ClassDirective.cpp:165-229` -- and cycles through inherit edges and mixinclass edges alike.
`ClassGraph::inherit`'s check order maps one to one onto `ClassClass.cpp:1298-1332`, and M10 was
verified in the code rather than from the report.

**Finding 1 is the disclosure, checked and found larger.** `install_directives` creates every class in
one loop and attaches every method in the next, and `UNINIT` occurs zero times in the generated
`setup_classes.rs` -- so `parent_has_uninit` is `false` for **every class a program can declare,
always**. The only writer is a direct `ClassGraph` call, in an order the directive path never produces.
**And the three handover programs use `::METHOD uninit CLASS`, which routes through `add_class_method`
to `class_define` and sets nothing**, so the other flag is unreachable for them too. The in-crate test
passes over a layer no directive-installed class reaches: an instrument that cannot fail in the way
that matters, which is the hazard this plan has a named memory for.

**Finding 2 is worse in kind than finding 1, because it freezes the divergence.** `ClassDef::has_uninit`
is documented as the oracle's `HAS_UNINIT` and is not: the oracle also sets it in `checkUninit`
(`ClassClass.cpp:1210-1218`) from the **flattened** behaviour via `subclass` (`:1626`), so an
inheriting class has it set there and not here -- and `behaviour_wiring.rs` carries
`assert!(!g.has_uninit(child))`, **asserting our wrong answer is correct**. The site the flag exists
for reads exactly it (`:1892`), so 5b following that doc under-registers silently.

**Ruling: narrow fix with the escape hatch that has now worked three times.** Set `has_uninit` where a
class method named `UNINIT` is attached, and run the propagation as a pass after methods are attached.
If it cannot be done without reordering install itself, stop and report -- that reordering is a
different subsystem and is the standing `phase-4-exclusions.txt` row's own root cause. Three things are
owed on either branch: the in-crate test must name that it exercises the graph API and not the
directive path; the handover must carry that the flag is false for every declarable class today; and
the three programs must carry the `class_define` finding so 5b starts from it.

**Ruling: unpin the assertion.** An assertion that quietly freezes a wrong answer is worse than no
assertion, because it gets cited as a decision.

**One item to clarify rather than fix.** The reviewer could not reproduce control 1 from its
description -- a literal same-polarity substitution would not produce the reported flip; the observable
requires building a `MIXINCLASS` directive as a plain subclass, which is what the plan's own line
describes. It judged the control genuine and both polarities witnessed. The description has to name a
mutation someone else can apply and get the same result.

### Task 7, fix round 1 landed: `c35ba11cc`. Re-review dispatched (opus). Sitting running separately.

Five gates 0, corpus 127 of 127 unchanged -- the round adds no corpus program, on the claim that
nothing it changes is expressible as a differential row. I asked the re-reviewer to judge that claim
rather than accept it.

**My ruling named a mechanism and was wrong, and the implementer measured it before building.** I
wrote "set `has_uninit` where a class method named `UNINIT` is attached". The oracle does not:
`checkUninit` sets the flag from the **flattened instance** behaviour, while the class-side spelling
reaches `hasUninitMethod`/`requiresUninit` (`ClassClass.cpp:1222`) and registers the class **object**
in the collector's uninit table. The discriminating probe is worth keeping: **two `.K~new` instances
under a class-side `::METHOD uninit CLASS` print `uninit on K` once, not three times** -- so the
class-side spelling sets the flag on nothing, and my instruction taken literally would have
over-registered every instance such a class ever makes.

What was built instead is `ClassGraph::check_uninit` mirroring the oracle's function, plus
`refresh_parent_has_uninit`, run over a file's classes in dependency order **after** methods are
attached, following the `refresh_class_behaviour` loop already there. No install reordering.

**The general lesson, now a memory.** Every ruling I made this session that named an **outcome** held;
every one that named a **mechanism or a probe** was wrong -- `.RexxInfo~class~id` as a discriminator,
"comparison cannot raise", and this. The asymmetry is structural: an outcome ruling is checkable by
whoever holds the code, a mechanism ruling arrives with authority attached and nothing to test but the
code built from it. Name the site only when I have run something there, and say so in the same
sentence so it reads as evidence rather than instruction.

**The re-review is told my brief may have been the wrong one and asked to re-take the probe itself**,
because the alternative is a wrong semantics resting on a plausible measurement nobody re-ran. Its
sharpest assigned target is a **mixin carrying an instance `::METHOD uninit`** -- the case the brief's
citation and the fix each cover from a different side -- and whether recomputing as a post-pass can
differ from the oracle's incremental computation on a forward `INHERIT`, a diamond, or a class that
both subclasses and inherits.

### Task 7, re-review of round 1: **mechanism ACCEPTED**, prose owed. Round 2 dispatched.

**My brief was wrong and the re-review settled it by measurement, with the control I had not asked
for.** Re-taking the probe from a fresh empty directory: the class-side `::METHOD uninit CLASS` with
two `.K~new` prints `uninit on K` once, and **the same program with an *instance* `uninit` prints
twice**. That second program is the difference between a suggestive measurement and a conclusive one,
and neither the implementer nor I ran it.

**Two facts established that 5b will lean on.** `inherit` **does** reach `checkUninit`, indirectly via
`updateSubClasses()` at `ClassClass.cpp:1359` into `:1052` -- so the brief's `:1364` citation was not
the whole mixin story, and the fix gets that case right *because* it keys on flattened behaviour
rather than on the propagation. And the post-pass **cannot** diverge from the oracle's incremental
computation: the oracle finishes each class -- create, `INHERIT`, `defineMethods` -- before starting
the next, in dependency order, so both flags are a fixpoint the pass recomputes. Nine shapes measured
and agreed.

**"No differential row could witness this" holds, and was checked rather than taken:** nothing outside
tests reads either flag, `rexx-core`'s `Object::has_uninit` is a different flag this crate never sets,
and every observing program needs `~new`, which is `rc 120` on both engines.

**The round's own mutation sentence is false**, which is the item to fix first: `lib.rs:5411-5413`
claims that without the pass every assertion expecting `true` reads `false`. Run, `has_uninit(base)`
still reads true because `ClassGraph::define` sets it, and the test fails at the `kid` row. The
replacement is stronger than the original: deleting the whole pass, deleting only `check_uninit`, and
deleting only `refresh_parent_has_uninit` each redden the test **at a different row**, so both halves
are independently witnessed.

**Four new set-size phrases, in the round that struck two elsewhere.** That is the fourth consecutive
round across two tasks. The rule is the one that finally held on Task 6: delete the enumeration, do
not recount it. One of the four is contestable as well as banned -- "two oracle sites set it" against
four `setHasUninitDefined()` call sites, two tautological -- which is the argument for deleting rather
than correcting in one line.

**Ruling on the dead branch and the sitting:** `lib.rs:3547-3549`'s `else { continue; }` is
unreachable because `:3444` inserts every `order` index, so removing it changes no path the benchmarks
take and the sitting tagged `c35ba11cc` stands. No fresh sitting for a prose round -- and if the branch
turns out to be reachable, that is a different finding and the round stops rather than deleting it.

### Task 7, fix round 2 landed: `381e6ff31`, with round 1's sitting rows at `d0b7bd151`. Re-review dispatched (sonnet).

Five gates 0, corpus 127 of 127, zero `failures:` and zero `test result: FAILED` across all three
captured outputs. No sitting this round, per the ruling, with the reason recorded: the only executable
change is the unreachable branch becoming the index it always took, and no axis program installs a
directive so none enters that loop.

**My replacement sentence was wrong too, and the implementer ran it rather than adopting it.** I wrote
that three deletions each redden the test at a different row; there are **two** -- deleting the whole
pass and deleting only `check_uninit` both stop at `has_uninit(kid)`, which is asserted first. The
substantive claim survives and is what the comment now carries: each call is independently witnessed,
and `has_uninit(base)` witnesses neither, which is exactly what the original false sentence obscured.

**I passed on an unrun measurement while correcting someone for an unrun measurement.** The provenance
is the error: a re-review reported it, I restated it as fact in a brief. A finding arrives with the
reviewer's authority the way a ruling arrives with mine, so anything I repeat into a brief I own --
run it, or attribute it with "check before relying on it". Added to the ruling memory.

**The self-sweep found more than my list did, which is the habit to keep.** I named four set-size
phrases; sweeping the round turned up **two more**, one of them inside the very comment item 1 was
replacing. A named list of instances is never the complete set; only re-reading everything written
against the **rule** finds the rest. Four rounds across two tasks have now established that.

The re-review is told my prediction was wrong and to verify the **measured** version that shipped
rather than the one in my brief -- and to check the sitting rows' min and max rather than the median,
since `alloc4c ir small` is the known bimodal cell that has read max 1.000001 in two earlier sittings
and would falsify any "min equal to max" summary.

### Task 7: complete. Round 2 re-review **ACCEPT**; the one new Minor was my citation and is fixed in the report. Commits `6aa432f19`, `c1510a936`, `c35ba11cc`, `381e6ff31`, `d0b7bd151`.

The dead branch was confirmed unreachable by reading both loops rather than by trusting the claim; the
three deletions were re-run independently and match the shipped comment exactly; and clippy was re-run
after `touch`ing the changed files, so its green is a real re-check rather than a stale cache -- a
check on the instrument rather than only with it.

**The Minor was `ClassClass.cpp:1359`, which I relayed into a brief without printing the line.** It is
`:1361`; `:1359`-`:1360` are the comment. Verified by me this time, and then the implementer verified
it again before applying, and checked the other half of the same citation while the lines were on
screen -- `:1052` is `checkUninit();` inside `updateSubClasses`, correct as written. One occurrence,
report only, no commit owed.

**Third relayed-claim error in three rounds, and all three were line-number citations** --
`ClassClass.cpp:1214` (a comment; the setter is `:1217`), the three-rows mutation claim, and `:1359`.
A `sed -n` settles any of them in one command, which is less than the clause of attribution would have
cost. Recorded: a cited line number in someone else's report is unverified until I have printed that
line.

**Two practices from this task are now expected rather than optional**, because each caught something
in a round that had already been reviewed: sweeping your own round against the **rule** rather than
against the list of named instances, and **running** a correction handed to you instead of adopting
it. The second caught two errors of mine that would otherwise have shipped.

**Task 7: complete.**

### Task 8 pre-dispatch prerequisite check: clean. Two of my own probes were wrong, both the same shape.

Every measured claim reproduces, one process each from a fresh empty directory, three descriptors
separate, crate rebuilt at Task 7's head:

| program | oracle | crate, both engines |
|---|---|---|
| the metaclass witness, `S`'s method without `CLASS` | rc 0, `class-side hi` | rc 120 at `::CLASS METACLASS` |
| the same with `::METHOD classSideHi CLASS` | rc **159**, `97.1 Object "The K class" does not understand message "CLASSSIDEHI"` | rc 120 |
| `.K~inherit(.S)` | rc **158**, `98.943 Class "The K class" is not a subclass of "The S class" base class "The Class class"` | rc 120, `method "INHERIT" of class "Class" is not implemented` |
| `::CLASS K PRIVATE` | rc 0 | rc 0, identical stdout |
| `::CLASS K ABSTRACT` | rc 0 | rc 0, identical stdout |

The `~inherit` refusal still stands after Task 7, which is right: Task 7 installed the **directive**,
not the method.

**And the design claim checks out at the source.** `updateSubClasses` (`ClassClass.cpp:1036`) calls
`createInstanceBehaviour` then `createClassBehaviour`, with the comment saying the class behaviour is
built second *"because the added methods may have an impact on metaclasses"* -- which is exactly the
merge position this task is about. `checkUninit()` at `:1052` sits inside it, confirming Task 7's
citation.

**Two probe failures of mine in one command, both "the pattern matched nothing".**
`grep "void RexxClass::updateSubClasses"` found nothing because the source has **two spaces** after
`void`; and `grep -rn clasdi oodocs/*.xml` found nothing because the glob does not recurse -- it is in
`oodocs/rexxref/en-US/dire.xml`. Had I stopped at either, I would have reported a live referent
missing. **An empty result is a claim about the pattern until the pattern is shown to match something
it should.** This is now the third and fourth instance today, after the git pathspec.

### Task 8 in progress. Two findings handed back to me, both ruled; and a **Task 7 regression** found by Task 8.

**Ruling: both gate table C corrections stand.** `xmetac` and `typcla` named Task 8 as the task that
can first run their control, and both probes read `~id` and `~class~id`, which are **Task 9's** --
measured `rc 120 method "ID" of class "Class" is not implemented`, so neither row could reach `agree`
here. The corrected sentences name Task 9 and say what installs at Task 8, which is the right split: a
control's task and a mechanism's task are different questions and the row now answers both.

**Ruling: `typcla` needs no abstract discriminator, and its control was vacuous as written.** It named
"make `::CLASS ... ABSTRACT` a no-op" as a mutation that would redden the row -- it cannot, because the
probe asks each class its `~class~id` and an abstract class answers exactly as a plain one does. The
struck half is right, and the reason the row needs no replacement is that **`abscla` already is the row
for abstract enforcement**, at 5b, because the check lives inside `~new`. A discriminator in `typcla`
would duplicate it. **Third vacuous control found on this plan, all three by running rather than by
reading.**

**A silent wrong answer that Task 7 introduced and Task 7's review did not find.**
`::CLASS S MIXINCLASS Class ABSTRACT` was `rc 0` printing `main ran` at Task 7's head, against the
oracle's `98.990 rc 158` -- measured on the pinned build before this task touched anything. It became
reachable *when Task 7 landed MIXINCLASS*, and it is the only effect of `ABSTRACT` a program can
observe before `~new`. Task 7's review measured many shapes the committed programs did not cover, but
they were all `MIXINCLASS`/`INHERIT` shapes; **option combinations were not among them**, and that is
the gap in how I framed those risks. Closed here for one `if` over a flag the task had to build
anyway, which is the right call -- with the report required to record it as a Task 7 regression closed
in Task 8, so the history says where it came from.

Standing deferral, not this task's: `::CLASS K METACLASS Singleton` is `98.908` here against the
oracle's rc 0, because `.Singleton` is a deferral this registry does not build -- the same shape as
`::class k subclass singleton`, which diverges identically, so the metaclass keyword adds nothing.

### Task 8 implementation complete: `bb6d46466` code, `96773c448` sitting. One doc edit ruled before review.

Five gates 0 at the committed head after every control was reverted, corpus **135 of 135**, staleness
test run with one `grep` per commit across all 28.

**The `strings tw small` cell does not reproduce.** A second sitting to a throwaway baseline reads
1.000000 in median, min and max for all four `strings` cells while reproducing `arith`'s four values to
six decimals, so the committed TSV keeps exactly one sitting for the commit and the cell is that
sitting's own artifact. **Decided by re-running, not by the reachability argument it already had** --
which is the right order, because the argument would have been just as persuasive if the cell had been
real.

**A corpus program was deleted rather than added, and mutation is why.** Two candidate witnesses, run
against "check after the override" and "delete the check": the plain one caught nothing the corner one
did not, and the corner one caught a mutation the plain one missed. One program, strictly more caught.
Subtraction is the rarer move and nothing in the process asks for it.

**Ruling: `blame_native_method`'s rule is mine and it over-predicts, so it changes.**
`RexxClass::subclass` is the body of the `~subclass` method and `99.927` raises inside it, so the rule
as written -- anything reaching a native method's body and raising from inside it owes the frame --
predicts a line the oracle does not emit. **The discriminator is whether the oracle got there by a
send:** `ClassDirective::install` calls `subclass()` directly, and reaches `INHERIT` through
`sendMessage`, which is why Task 7's refusals carry the line and Task 8's three do not. I generalised
that rule from two callers that both happened to arrive by send.

The edit lands as its own commit before the review dispatches, so the reviewer sees one head. The
implementer was right not to edit a doc it did not own, and right to leave the measurement in
`Raised::bad_metaclass`'s doc for a reader arriving from that side.

Also to be checked at review: the claim that **no `ir_dual_cases` stanza is warranted** because
directive install runs before either engine takes over, so a stanza would discriminate nothing -- with
every program run on both engines against the oracle with `cmp` on all three descriptors instead.

### Correcting two things I wrote above, both corrected by the implementer rather than by me.

**"All three vacuous controls were found by running" is false.** The third was found by **reading** --
the `~class~id` line was noticed while checking whether the row could reach `agree` at all. The honest
version, and it is still the point: two were found by running, and the third was found during an
inspection that **a run had forced**. Nobody was reviewing that arm on its own account.

**"The pinned-build measurement as the before column" was the wrong instruction**, and taking it
literally would have understated the regression. The pinned `rexx-run-15a1ffa98` **predates Task 7**,
so it answers `rc 120 ::CLASS MIXINCLASS is not implemented` -- correct by refusal, never reaching the
bug. The regression is visible only against Task 7's own head, so the implementer built one: a
throwaway worktree at `d0b7bd151`, release build, measured, worktree removed. The progression is what
makes "regression" the right word:

```
oracle                     rc 158  98.990  Class S is a metaclass and cannot be made ABSTRACT.
pinned rexx-run-15a1ffa98  rc 120  ::CLASS MIXINCLASS is not implemented (Phase 5)
Task 7 head, d0b7bd151     rc 0    main ran                     <-- the regression
HEAD                       rc 158  98.990, byte-identical
```

**The general point I had not made: the pinned build answers "was this here before the phase", not
"was this here before the previous task".** Those are different questions, and a phase where each task
changes the same surface needs the second one answered by a build at the previous task's head.

**And the `typcla` ruling was accepted for a reason worth keeping.** Mine replaced the implementer's
because its argument was about **reachability** -- "I do not believe a discriminator exists before
`~new`" -- which is the mutable kind that rots, and mine is about the **table's own shape**: `abscla`
is the row for abstract enforcement, so a discriminator in `typcla` would duplicate a row rather than
cover a gap. It went into the control sentence rather than the report, because the next person asking
"should this row get one?" reads the arm.

Third commit `6754ae2a2`, `tests/`-only, gates re-run at it, corpus 135 of 135.

### `121bc720d` states the send condition. Two rulings, and a dispatch error of mine.

**The rule now reads: the line is owed wherever the oracle reached the failing native method's body by
a message send, whatever put the send there.** `ClassDirective::install` carries both sides on one
directive -- `classObject->sendMessage` for `INHERIT` at `:230`, direct calls to `subclass()` and
`mixinClass()` at `:205` and `:200` -- so the discrimination is measurable in one place.

**The implementer checked my generalisation against the C++ rather than assuming it covered the third
caller, and said why: this plan's own record is that my mechanism rulings are the ones that turn out
wrong.** That is the first time that record has been applied to me by someone else instead of by me
afterwards. It holds: `StemClass::processUnknown` reaches the method with `value->messageSend`
(`StemClass.cpp:280`), so the stem-forwarded operator genuinely is a send.

**And the sharpening is better than the rule I asked for.** A **source-level** send term is not what
the rule asks about -- there is no `~` anywhere in `say b. + 1`, and it is still a send at the C++
level. That is exactly the misreading the next reader would make, so it goes in the doc rather than
only in the report.

**Ruling: fix all four citations in one follow-up, including the two that are Task 7's.** Leaving a
known-wrong citation for a reviewer to rediscover spends a review seat on something already known, and
the finding would arrive with less evidence than is already in hand. `Interp::inherit_mixin` cites
`ClassDirective.cpp:224`, which is `{`; the send is `:230` -- **the fourth wrong line citation on this
plan**. `Raised::class_not_found` cites `:214`-`:219` where the resolution is `:221`-`:222` and its
report `:225`. The implementer's own two ranges open on the comment above their mechanism, which is
not wrong and is the shape that becomes wrong at the next inserted line.

**My error: I dispatched Task 8's review against `6754ae2a2` before `121bc720d` landed**, having
mistaken the `typcla` commit for the doc edit I had explicitly asked to be told the SHA of. Risk 5's
subject was therefore outside the reviewer's diff and it would have reported the rule unamended. Told
it to read the file at HEAD, and that a citation-only commit follows and is out of its scope. **The
instruction was right and I did not wait for the answer to it** -- asking for a SHA and then not
waiting for it is the same as not asking.

### `8fb97527c`: four citations pointed at the lines their sentences are about. Gates 0, corpus 135 of 135, comments only so no sitting.

```
inherit_mixin       ClassDirective.cpp:224 -> :230    :224 is `{`
class_not_found     :214-:219 -> :222 and :230        the sentence claims the lookup precedes the
                                                      send, so it now names both
abstract_metaclass  :246-:249 -> :247-:249            :246 is the comment above `if (isAbstract())`
install_class_at    ClassClass.cpp:1753 -> :1754      :1753 is the ` */` above the function
```

Each printed before the edit and again after it.

**The implementer flagged an interaction I would have hit as a finding to adjudicate:** two of those
four are **inside** the range the running review is reading, so its diff shows the old values while
HEAD has the new ones. Told the reviewer not to report them -- and to say so anyway if its own pass had
already caught them, because that is information about the pass that the fix erases.

**A boundary worth naming, in its words rather than mine.** It had defaulted to "another task's text,
not mine to touch", *"which treats authorship as the boundary when the real boundary is whether the
evidence is already in hand"*. That is the better rule and it generalises past citations.

**It also declined to widen a commit I had scoped**, having checked that the source-level-send
distinction was already in `blame_native_method`'s doc -- `not "who wrote a send term"`, and a stem
forwarding an operator reaching the method through `value->messageSend` with no send term in sight. It
offered to sharpen it as its own commit instead of folding it in. That is the right handling of a
scoped instruction, and it is the opposite failure from the one I made an hour ago by not waiting for
a SHA I had asked for.

### Task 8 review: **spec compliant, no behavioural defect**, three Important prose findings and six minors. Round 1 dispatched.

The reviewer re-measured rather than trusting the report: nine committed programs byte for byte on both
engines plus fourteen shapes of its own, and all eight risks reproduced -- the merge ordering with its
`hasScope` guard and four uncovered shapes, the metaclass override against `ClassClass.cpp:1586`-`:1591`
and the oracle, the refusal order in both source orders and against three neighbours, and the
regression progression with row 3 taken from source because `abstract_` had no reader at `d0b7bd151`.

**The amended frame rule is discriminated by exactly the predicted pair.**
`.Object~subclass('X', .Object)` is `99.927` **with** the `Compiled method "SUBCLASS"` frame -- the same
body, sent instead of called. That is the cleanest possible confirmation: one body, two arrival routes,
two different answers.

**And the reviewer's own pass caught three of the four citations before `8fb97527c` landed**, from the
C++ itself, missing only the one outside its diff. I asked it to say so whether or not the fix had
pre-empted it, because a fix erases the evidence about whether the pass would have found it -- and it
would have.

**Ruling: finding 1 is a latent bug and is fixed in the code, not in the sentence.** `class_of`'s doc
claims `~class` and `~metaClass` coincide for a class object; measured, for
`::CLASS T SUBCLASS S METACLASS M1` with `S` a metaclass, `.T~metaClass~id` is `S` and `.T~class~id` is
`M1`. `:1615` passes the *named* metaclass while `:1590` -- the override this task implements -- has
already moved `metaClass` to the superclass. **Both accessors read one field, so Task 9's `~class`
answers wrongly the moment it is written.** A corrected doc plus a note for Task 9 is exactly the
arrangement recorded as failing three times here, the third shipped from a dispatch warning about it;
only the type-level fix held. Nothing differential can witness it -- every observer needs `~class`,
which is Task 9's -- so the instrument is an in-crate test that fails if the fields are collapsed back.

**Ruling on the sitting question the reviewer could not answer:** `121bc720d` owes none, and the round
proves it rather than asserting it -- build `rexx-run` at `121bc720d~1` and at `121bc720d`, compare the
sha256 sums. A byte-identical binary cannot move any axis, which is a stronger statement than "comments
do not matter", and it is two commands.

Minors include a **fifth wrong citation** (`lib.rs:3733` citing the null tests against a
`reportException`), a wrong `ir_dual` reason with the reviewer supplying the right one, a commit count
taken at the base rather than at head, and `class__abstract__subkeyword.rex` still being a `say 'main'`
probe any `ABSTRACT`-ignoring build passes -- said in the report for `PRIVATE`/`PUBLIC` and not for it.

### Task 8, fix round 1 landed: `8a88dc63d`. Sitting owed and running. All eight brief items closed.

**Item 1 fixed at the type level, and it is wider than the finding.** The review's row needed
`METACLASS`; re-measured, `::CLASS T2 SUBCLASS S` with `S` a metaclass parts `~metaClass` (`S`) from
`~class` (`Class`) **with no `METACLASS` keyword anywhere**. So the split is a property of **deriving
from a metaclass**, and naming `METACLASS` only chooses which value the `~class` side holds. That is
the sentence Task 9 needs; the narrower one would have let it think the keyword was the trigger.
`ClassDef` carries `owning_class` beside `metaclass`, bound before the override where `metaclass` is
bound after. The test goes through `install_directives` rather than a hand-built graph -- the Task 7
failure this plan already paid for -- and **both collapses fail it at different assertions**, each run
confirmed `running 1 test` so neither was a filter matching nothing.

**A doc falsified by the round's own commit, found by the implementer and missed by the review and by
me:** `a_bare_class_directive_subclasses_object` still said `SUBCLASS`/`METACLASS`/`INHERIT` are
"still `directive_gap` above". All three install.

**My ruling that this round owed no sitting was wrong, and the predicate the implementer built is what
refutes it.** It measured rather than reasoning: `.text` goes **1205001 to 1205177 bytes** across
`8a88dc63d~1..8a88dc63d`. So the predicate **discriminates** -- it held for `121bc720d` and fails here,
which is what makes it a rule rather than an excuse, and that pair is worth more than either half.

**The general lesson, in its words, and it is better than mine.** A stale **enumeration** at least
looks stale on rereading, because the members are there to count. A stale **reason** reads as fresh,
because the sentence carrying it is still well-formed and still true of what it was written about. The
`.text` argument was correct for a comments-only commit and stayed attached after the round grew a
field split. The check is the citation rule one level up: **state the predicate and apply it, rather
than restating the conclusion it produced last time.**

**And its direct verification beat my inference on the `cycles` half.** I deduced "nothing changes
size, therefore nothing moves" from a partial table; it compared the section table and program headers
-- 43 sections, every name, address, offset and size identical, every segment's offset, vaddr, filesz
and memsz identical. Measured, not deduced, and my inference would have been wrong had anything I
could not see moved.

**Open for Moritz:** whether the no-sitting-owed method belongs in `rust/CLAUDE.md` under Gates, where
that section already carries this shape of rule. The implementer's instinct is right -- a future task
looks there, not in a closed task's report -- and I declined to edit `CLAUDE.md` on a peer's request.
Draft parked in the Task 8 report.

### Task 8, fix round 1 sitting: `c9523023c`. No axis moves. Re-review dispatched over `6754ae2a2..c9523023c`.

**I verified the rows myself against the TSV rather than reading the summary**, and they are exact: of
24 `pinned>head across_builds instructions:u` cells for `8a88dc63d`, exactly six are non-flat -- the
four `arith` cells and two single-ulp `alloc4c` cells. `strings tw small` is flat on all three
statistics, a third reading that closes it as the first sitting's own artifact.

**The predicate now has one pass and one failure behind it**, which is the part of this round worth
keeping: `121bc720d` held it, `8a88dc63d` fails it at `.text` 1205001 to 1205177 bytes. **A rule with
only confirming instances is indistinguishable from a rule that always says yes**, and the confirming
half was cheap only because the failing half was measured too.

**The shape was predicted before it was measured:** no axis program installs a `::CLASS` directive, so
`define_class` runs only over the bootstrap classes, once, before any measured pass -- the added field
write lands in the regression fit's fixed term and never in `per_pass`.

**Checked the one load-bearing unverified claim in the hand-off:** that the only commit after
`8a88dc63d` is the TSV and no gate reads it. It holds -- nothing under `rust/crates` reads a baseline
TSV except `rexx-arms` itself plus a doc comment -- and the search was **not vacuous**, since the
pattern matches on `phase-4e-arms.tsv` references. Four probes have been lost to empty-pattern results
this session, so the non-vacuity is worth stating with the result.

`rust/CLAUDE.md` remains untouched and the four-line draft sits in the report under a NOT APPLIED
heading, in that file's house style, carrying the rule, the one-path requirement, the reproducibility
control, `--dump-section` over `--only-section` with why the latter passes by not looking, and the
measured pass/fail pair as its evidence. Open for Moritz.

### Task 8, re-review of round 1: **code and instrument accepted, prose not**. Round 2 dispatched, prose only.

The field split was attacked with **18 shapes on the oracle and then read back out of `ClassGraph` in a
copied tree** -- no divergence. A metaclass from a metaclass, `METACLASS` naming the superclass, a
middle-generation metaclass, a later-declared metaclass, a mixin with a metaclass, a grandparent on the
class side, and `~metaClass`/`~class` on `.Object`, `.Class`, `.String`, `.Method`. It also closed a
hazard neither of us had raised: `newRexx`'s `isPrimitiveClass()` branch cannot bite, because `.Class`
is the only primitive carrying `META_CLASS`.

The test is real -- plain `cargo test`, no gate variable, both collapses failing at the two different
assertions with `running 1 test` printed each time. **And every C++ citation new in this diff checks
out: the first round on this plan with none wrong.**

**N2 is my sentence and it is false.** I wrote *"the split is a property of deriving from a metaclass,
and naming `METACLASS` only chooses which value the `~class` side holds"*, told the implementer to lift
my phrasing, and it does not survive. Necessity holds; **sufficiency does not** --
`::class MC mixinclass class`, `::class Z subclass Class` and `::CLASS M3 SUBCLASS MC METACLASS MC`
each derive from a metaclass and each coincide. The exact rule: **they part iff the superclass is a
metaclass and is not the named-or-inherited metaclass.**

**The counterexample was two lines above the sentence in the report's own table** -- `S metaclass Class
S class Class` -- and it is the first directive of the committed test's own program. Both of us read
past it. **A generalisation written directly above its own counterexample is the sharpest form of this
defect I have seen**: the evidence was not missing, it was adjacent.

**N1 is the mirror image: my brief had the oracle's write order right and the round inverted it.**
`:1590` runs *before* `:1615` and writes a different location, the local `meta_class` never being
reassigned. Six copies, one in an uneditable commit message.

**N3 I verified myself, having repeated the wrong number to Moritz.** Not "fourth sitting for that
cell" but **six**; not "three earlier sittings" but five. The TSV holds **eight** sittings and
`arith ir small` has spanned `[1.001283..1.001284]` in six, from `c351fa473` onward. Ruled: **say the
span and the commits, never the count** -- the count is the part that keeps going wrong across three
tasks now.

Also worth having: `arith` did not move once but twice -- `84f3e580b` reads 1.000542, `211763aaa` back
to 1.000000, and `c351fa473` onward at 1.001283/4. The "it moved at round 2" story was a simplification.

### Task 8, fix round 2 landed: `4729e5d3a`. **PAUSED HERE at Moritz's instruction.**

Prose only, and I verified it independently rather than accepting the claim: `git diff -U0
c9523023c..4729e5d3a -- rust/crates` filtered to lines that are not comment lines is **0 lines**. Five
gates 0, corpus 135 of 135, tree clean.

**N2's exact rule now has its coinciding rows kept beside it, which is the part that makes it hold.**

```
::class MC MIXINCLASS Class           ~metaClass Class  ~class Class   same
::class Z  SUBCLASS Class             ~metaClass Class  ~class Class   same
::CLASS M3 SUBCLASS MC METACLASS MC   ~metaClass MC     ~class MC      same
::CLASS T  SUBCLASS MC METACLASS M1   ~metaClass MC     ~class M1      part
::CLASS T2 SUBCLASS MC                ~metaClass MC     ~class Class   part
::CLASS K  METACLASS M1               ~metaClass M1     ~class M1      same
```

The three coinciding rows stay **because my false shape predicts a divergence for each of them** -- a
counterexample retained as a standing refutation rather than deleted once the sentence was fixed.

**The sharpest statement of how we both missed it, in its words:** the refuting row was not merely
nearby, it was *inside the evidence* -- FR1.1's own table, two lines above the false sentence, and the
first directive the committed test installs. *"I had the data, formatted it, and read the sentence
instead of the table."* A refuting row sitting inside the support for the claim it refutes is a
different failure from not having looked.

**N1 corrected from the C++ rather than from either of our texts:** `:1590` writes the field to the
superclass, `:1613` reads the already-overridden field, `:1615` hands `setOwningClass` the local that
the field write never touched -- two locations, not a race over one.

**N3 re-derived rather than accepted**, and the report now carries the span and the six commits with no
count at all, noting the median flips between the two adjacent values while min and max stay pinned.
Third time on this task that deleting a count beat correcting it.

**It also caught a defect I would not have:** its first rewrite of the test's doc used the oracle
probe's `MC`/`M3` names while the test installs `S`/`M1` -- a table sending a reader after classes the
program does not declare. Rebuilt against the program's own names, with a column marking which rows the
test asserts and which it only documents.

## Resume point

* **Next step:** dispatch a scoped re-review of `c9523023c..4729e5d3a`. Prose-only diff; a cheaper tier
  is right. Its targets: whether the `iff` is stated correctly in all five copies, whether N1's
  execution-order sentence matches the C++, and whether the round introduced any new set-size,
  enumeration or historical phrasing -- four rounds across three tasks have done exactly that inside a
  round striking the same shape.
* **Then:** Task 8 closes, and Task 9 is next -- the Object and Class reflection protocol, which is the
  task that consumes `owning_class` and the corrected rule.
* **Open for Moritz, 1:** whether the byte-equivalence method goes into `rust/CLAUDE.md` under Gates.
  Four-line draft parked in the Task 8 report under a NOT APPLIED heading, in that file's house style,
  carrying the rule, the one-path requirement, the reproducibility control, `--dump-section` over
  `--only-section`, and the measured pass/fail pair.
* **Open for Moritz, 2:** whether the sufficiency counterexample `S` should become an **asserted** row
  in the in-crate test rather than a documented one. It is a code change, so the round left it; making
  it asserted would put my false generalisation under a test that fails if anyone restates it.
* Tasks 9 through 24 remain, with the ordering constraints unchanged: 11 before 12, 17 before 21,
  9 and 12 before 21, and 21 before 23.

### Resumed. Task 8 round 2 re-review dispatched (sonnet). Task 9 pre-dispatch prerequisite check: **clean**.

**Tooling note: the superpowers plugin cache is gone from this machine**, so `review-package` and
`task-brief` no longer exist. Both are now done by hand -- the package is
`git log --oneline` + `git diff --stat` + `git diff -U10` into one uniquely named file, which is what
the skill prescribes as the fallback, and briefs are `sed -n` over the plan's task section. Task 9's
and Task 10's briefs were extracted that way.

**Task 9's referents and measurements, all reproducing.** One process each, fresh empty directory,
three descriptors separate, crate rebuilt at HEAD:

| probe | oracle | crate, both engines |
|---|---|---|
| `.array~id` | rc 0, `Array` | rc 120, `method "ID" of class "Class" is not implemented` |
| `.K~superClasses` / `~metaClass` / `~isA(.Class)` / `.Array~package~name` | rc 0, `The Object class`, `The Class class`, `1`, `REXX` | rc 120 at `SUPERCLASSES` |
| `.K~method("M")` for a **class** method | rc 159, `97.1`, opening `       *-* Compiled method "METHOD" with scope "Class".` | rc 120 |
| `.Array~method("STRING")` -- a donated name | rc 159, same frame | rc 120 |
| `.Array~method("APPEND")` -- its own | rc 0, `a Method` | rc 120 |
| `.Array~hasMethod("APPEND")` | rc 0, **`0`** | rc 0, `0` -- **already agreeing byte for byte** |

**The task's own load-bearing detail checks out at the source.** `ClassClass.cpp:984` is
`MethodClass *RexxClass::method(RexxString *method_name)`, and the comment immediately below it says
the instance methods defined at that level live in a separate dictionary retrieved directly -- which is
exactly the claim that `~method` reads the class's own dictionary and nothing else. The `.Array` pair
demonstrates it in both directions: `APPEND` answers, `STRING` raises.

`~hasMethod` answering `0` for `.Array~hasMethod("APPEND")` is right rather than a contradiction of the
brief: on a **class object** the question is whether the class object itself responds, and `APPEND` is
an instance method.

The three deferrals the task names are real and documented in `native_classes.rs` -- `QueueClass`,
`StemClass` and `VariableReference`, each with `Setup.cpp`'s donation or hiding as the stated reason,
which is the mechanism Task 21 owns.

**Task 9 is not dispatched yet:** Task 8 may still take a round 3, and an implementer there would
collide with one here.

### Correcting the tooling note: **the plugin files are not missing, they moved.**

They are at `/home/moritz/.claude/plugins/cache/claude-plugins-official/superpowers/6.3.0/...`, with
`review-package`, `task-brief` and `sdd-workspace` all present and executable. I wrote "the plugin
cache is gone from this machine" after searching only under `.claude-personal/plugins`, which no longer
exists.

**The config directory has now moved in both directions during this one session.** Earlier the path
`.claude/plugins` failed and the files were under `.claude-personal`; now `.claude-personal/plugins`
fails and they are under `.claude`. So neither location is the answer -- **locate the scripts each
time rather than caching the path**, and the seventh instance this session of a negative claim being
exactly as wide as the search behind it.

Verified the scripts run, and **my hand-extracted Task 9 brief is byte-identical to the script's
output**, so nothing done during the gap needs redoing.

### Task 8: complete. Round 2 re-review **PASS**. Commits `bb6d46466`, `96773c448`, `6754ae2a2`, `121bc720d`, `8fb97527c`, `8a88dc63d`, `c9523023c`, `4729e5d3a`.

All of N1's six copies addressed, one of them **by deletion rather than restatement** -- `registry.rs`
dropped the order claim instead of re-asserting it, which the reviewer accepted as silent-but-not-false
since it is stated correctly elsewhere. All five `iff` copies carry the necessity-not-sufficiency form
with the "and is not the named-or-inherited metaclass" clause and the counterexamples beside them.

**The reviewer re-measured all six rows itself and then walked the `iff`'s truth table by hand against
`RexxClass::subclass`'s actual tests** (`:1586` superclass-is-metaclass, `:1566`-`:1568`
default-metaclass), confirming both directions. That is stronger than re-running the rows: the rows
show the rule fits six cases, the truth table shows why it fits all of them.

**Second consecutive round with zero wrong citations**, every one printed. Comment-prose pass found
nothing beyond the five intended sites. And clippy was re-run cold after `touch`ing the changed files,
because the first pass was suspiciously fast -- the house rule catching a warm-cache green.

**Task 8: complete.**

### Task 9 returned complete: `4e9a0369f`, `18626fdb1`, `3e695182c`. Review dispatched (opus). I checked the sittings myself first.

**Two sittings recorded, and the pair is what makes the performance claim readable** -- `4e9a0369f`
before the fix and `18626fdb1` after:

| axis | before the fix | after |
|---|---|---|
| `strings ir` | 1.010616 | **1.009685** |
| `strings tw` | 1.006302 | 1.005749 |
| `alloc4c ir` | 1.002602-1.002706 | 1.001156-1.001203 |
| `alloc4c tw` | 1.001743-1.001789 | 1.000775-1.000795 |
| `arith` | 1.001159-1.001461 | **1.001006-1.001294** |
| `compound`, `emptyloop`, `varlookup` | 1.000000 | 1.000000 |

**Something the report did not claim and the numbers show: `arith` returned to exactly the standing
values** carried since Task 6 -- 1.001006 / 1.001284 / 1.001036 / 1.001294 -- having been higher before
the fix. So the `Copy` change removed the added work from every path it was on, not only from
`strings`. That is stronger evidence for the attribution than the axes that stayed flat.

**The report's headline `+0.9685%` is the `ir` arm, which is the worst one**, so it is the honest number
rather than a flattering pick. It is also **0.0315 percentage points under the guard**, for a task whose
subject is reflection and should not touch string rendering at all. I have asked the reviewer to rule
specifically on whether that is a pass or a finding dressed as a pass, and whether `try_text`'s array
arm -- which the report says carries the residual +0.89% -- can be off the string path entirely rather
than one line on it.

**The criterion is not met and the report says so unprompted.** Eleven registered classes read
`diverge-stdout`, each on exactly one probe line, `superclasses`, with the missing entries being
`CoreClasses.orx`'s own `~inherit` calls that `native_classes.rs` already assigns to **Task 13**. The
other 38 fail at `.NAME`: three are the deferrals the brief names as Task 21's, 35 are
prologue-defined.

**Control 1 came with the reason it is not vacuous, which I had not asked for:** the phase-gated command
**exits 101 without the mutation too**, so the exit status is not the evidence -- the verdict counts
moving 14/11/38 to 13/11/39 are. That is the distinction that separates a real control from one that
reads green on an unrelated failure.

**And the round self-reported five guessed C++ citations plus a test comment asserting an unmeasured
oracle answer** -- it claimed `say (.Object~superClasses = '')` answers `1` by reasoning from identity
comparison; the oracle answers `0`. Both fixed in `18626fdb1`. That is five more guessed citations on a
plan that had five before, so the reviewer prints every citation in the diff.

### Task 9 review: **Approved**. One Important is mine and is fixed at the plan (`c7117ef82`); the other is a citation. Round 1 dispatched.

**The criterion was unsatisfiable when it was written, and the phrase hid the reason.** "The wiring rows
for the classes this crate registers read `agree`" carved out only `Queue`, `Stem` and
`VariableReference` -- the gap that was known. **A class can be registered and still need the prologue
for its wiring row to match**, and eleven are: each differs on exactly one probe line, `superclasses`,
by exactly the `CoreClasses.orx` `~inherit` targets, work `native_classes.rs` assigned to **Task 13**
before Task 9 started. The criterion now asks that the rows this task's protocol can carry read `agree`
and that the rest are each **attributed** to the task that moves them, so a non-`agree` row is a
regression only if it belongs to neither Task 13 nor Task 21.

**The other Important is the sixth guessed citation on this task, and where it sat is the finding.**
`dispatch.rs:1397` cites `PackageClass::getName`, which does not exist -- it is `getProgramName`,
`PackageClass.hpp:147`, bound at `Setup.cpp:1189`. **It is the only citation in the diff without a line
number, and the only one that was wrong.** A citation with no line number is one nobody printed, which
is a cheaper tell than checking them all.

**`+0.9685%` ruled a pass**, both sittings matched against the TSV to six decimals by the reviewer as
well as by me. **0.03 points of headroom remain**, so the next task adding an arm to
`to_text`/`text_len`/`try_text` crosses the guard -- that goes into Task 10's dispatch, not only into a
report.

**Risk 1 came back clean from a direction I had asked for and a shape neither of us named:** 31 extra
names probed both ways, including `DEFINECLASSMETHOD` and `INHERITINSTANCEMETHODS`, which
`RexxClass::removeSetupMethods` strips after `Setup.cpp` adds them.

**An unreported divergence the round found and did not write down**, now item 2 of the fix round:
`.Array~method('APPEND')~class` is rc 120 against the oracle's `The Method class`, and `native_method`
mints a fresh object where the oracle returns the same one -- `==` is `1` there and is not here. Not
this task's to implement; this task's to record.

**And a control that used a hint instead of a test:** the "not an inlining threshold" claim was made
with `#[inline]`. `#[inline(always)]` is the test. Cheap to re-run, and the claim does not stand on the
hint.

### Task 9, fix round 1 landed: `5fd2002cf` plus sitting `40ea10305`. Re-review dispatched (sonnet).

**The citation fix generalised the way I hoped and did not say.** I gave the tell -- a citation with no
line number is one nobody printed -- and the round swept the diff for **that shape rather than that
name**, finding two more bare citations: `RexxObject::classObject` at `ObjectClass.cpp:1814`, whose body
is `behaviour->getOwningClass()` and is therefore *why* `~class` reads the owning class, and a bare
`MethodArguments.hpp` resolved to `:136` and `:161`. One bare reference stays bare because it is a
whole-file negative claim, **now recorded as wide as its pattern with a control showing the pattern can
match** -- which is the discipline applied to itself.

**Item 2's disposal is better reasoned than my instruction.** I asked where the identity half belongs;
it filed it as 5c's **with the method rows** and argued explicitly that it is *not* deviation 4's,
because the oracle's `1` is `RexxClass::method` handing back the object the dictionary already holds
(`ClassClass.cpp:991`) -- so it needs `MethodDict` to hold method objects, the same thing `.Method`'s
protocol needs. **Filing it under deviation 4 would have put it where nothing reaches it.**

**"Real work, not noise" was wrong about the mechanism, and the correction is what Task 10 needs.**
`strings.rex` builds no array, so the arm's body never runs. Per-pass counts make it exact:
**`try_text`'s one arm is 48 instructions per pass and everything else this task added is 4**, the delta
identical on both engines, which is what says it is the shared value model rather than one engine's
path.

**The headroom is now a number a task can budget against rather than a percentage.** The `ir` arm's 1%
ceiling is **53.70 instructions per pass** and this task stands at **52.00** -- **1.7 per pass left**.
And it is priced: `to_text` + `text_len` cost 4 per pass together, `try_text` alone cost 48. **So it is
a `try_text` arm specifically that nobody can afford next**, and a task needing one must buy it back
rather than absorb it. That goes into Task 10's dispatch verbatim.

**`#[inline(always)]` reproduces 1.009685 to six decimals**, so the claim survives the test that
`#[inline]` could not make.

**And a defect no item named, invisible to the differential by construction.** `class_package.rex`
printed this file's absolute path twice, which `corpus/README.md` forbids in the same words it uses to
exclude `PARSE SOURCE`'s third word. **No gate could catch it, because both interpreters get the same
path.** Projected away, and the replacement pins strictly more: whether the name equals that word,
whether it equals `REXX`, and that `.Array`'s does not. Printing the path asserted none of the three.

### Task 9: complete. Round 1 re-review **approved**, one transcription minor sent back for a one-line fix, no further re-review. Commits `4e9a0369f`, `18626fdb1`, `3e695182c`, `5fd2002cf`, `40ea10305`.

Every item verified at the source. `TOSTRING` was built and run live against the oracle and both engines
on five shapes -- a declared class, a primitive, a class-side send, a nonexistent name and the wrong
arity -- all byte-identical. The one apparent divergence, `.Array~superClasses~toString` missing
`OrderedCollection`, is `~makeString`'s existing gap and therefore **Task 13's known `superclasses`
issue rather than anything new**.

**The headroom arithmetic holds, checked against the TSV:** 1% of pinned `strings`/`ir` at
5369.518137 is 53.70; the task stands at 52.00; **1.7 instructions per pass remain**. `try_text` alone
is 48 per pass, `to_text` plus `text_len` 4 together.

**The deliberately-bare citation holds and was checked the right way:** `\.package\b` matches nothing in
`CoreClasses.orx` or `StreamClasses.orx` while `\.array\b` matches both -- so the pattern can match, and
the negative is as wide as it claims.

**"No gate could catch it" was verified rather than accepted:** `corpus.rs` compares both interpreters
on the same file in the same run, so identical-but-path-dependent output never diverges.

**The minor is median-versus-max**, the same column confusion that produced a false "min equal to max"
two tasks ago: the report's table quoted `alloc4c`/`ir`/small as `1.001204`, which is `value_max`, where
the median is `1.001203`. Ruled: **when a table quotes one figure per cell, name the column in the
header** and the confusion has nowhere to live.

### Task 10 pre-dispatch prerequisite check: clean. One instrument error of mine.

`dispatch.rs:540` is `pub(crate) fn resolve(&mut self, receiver, name, start_scope)` -- so the two
parameters this task adds, the caller's scope and the caller's package, are genuinely absent.
`SmallInt` is a `Decoded` variant at `rexx-core/src/handle.rs:138`. `ir_dual` is present. And both
roadmap citations are exact: the patchable-slot sentence is at `2026-07-27-rust-rewrite.md:492`, and the
superseded spec does cite `:482`.

**My own error: `head -6` on the first grep hid `dispatch.rs:540`, and I was one step from reporting
that the send path has no `resolve` at all.** The pattern matched it; the pipe dropped it. Third
instance this session of truncated output reading as absence, and the tell is that a `head` on a search
whose *completeness* is the question answers a different question than the one asked.

### Task 9 closed. The one-cell fix exposed two more, of a different kind.

Naming the column did not just fix `alloc4c`/`ir`/small's `1.001204` to `1.001203`. Re-reading every
numeric table column by column found **two further wrong cells, wrong a different way**: `alloc4c`'s
`tw large` and `ir large` were correct **medians of a pre-commit scratch sitting**, not of the
`18626fdb1` sitting the table's own caption named.

**So the two failure modes sit side by side: the right row's wrong column, and the right column's wrong
run.** Neither is catchable by a careful reader, because every figure is a real number from a real row.
A caption naming a commit does not constrain where the cells came from.

**And a claim fell with them.** *"Reproduces to six decimals on every axis"* was true of the run it was
written from and false of the run it named -- the `5fd2002cf` and `18626fdb1` sittings disagree in the
sixth decimal on exactly those two `alloc4c` cells, which is that axis's noise floor on a delta of a
thousandth rather than anything moving. Now stated with its exception and with what the exception means.

**The rule, in the implementer's words, and it is better than my instruction:** *a table of measurements
states its column and its source in the header, or it is not a table of measurements.* All four numeric
tables now carry one. Recorded as a memory.

**Task 9: complete.** `4e9a0369f`, `18626fdb1`, `3e695182c`, `5fd2002cf`, `40ea10305`, plus my plan
amendment `c7117ef82`.

### Task 10 in progress. Two pushbacks, both accepted; the plan said the wrong thing and is amended at `fc31fb82a`.

**"`resolve` gains the caller's scope" was wrong, and the parameter would have had nothing to put in
it.** I verified the implementer's reading rather than taking it: `RexxObject::checkPrivate` is at
`ObjectClass.cpp:609`, `:616` is `RexxObject *sender = activation->getReceiver();` and `:628` is
`RexxClass *scope = method->getScope();`. **So the caller contributes a receiver and the scope comes
from the resolved method.** Citations exact. The plan now says receiver, with the reasoning recorded
beside it.

**The roadmap amendment's evidence checks out too:** `ir.rs:1060` is `Message { index: u32 }`, with no
`site` field -- so there is no slot for a send to be patched into, and the sentence claiming the IR
"founds OO dispatch for Phase 5 by making a call site a patchable slot" does not describe this tree.
Amending rather than recording-why-it-survives is the right branch.

**Ruling on the interning: state the layer, do not dress it up.** The pool and the identity land; the
method dictionary stays keyed by string. D24's constraint as the spec words it is *selectors interned at
compile time*, which is met -- keying the dictionary by selector is a further change nobody has costed.
The report says what landed, what did not, and what the remaining half would buy, so a later task
decides it on evidence rather than rediscovering the gap.

**Two things sent as the finishing conditions.** The per-pass delta against 52.00 must be reported, not
only the ratio -- 53.5 is inside the guard and out of room, which is a different sentence from "under
1%", and this task added a behaviour arm to the hottest path plus a wider signature. And its tables get
read closely because Task 11 budgets against them, so: column and run named in the header, per the rule
the previous task's three wrong figures produced.

### Task 10 implementation complete: `64c78d0c6` roadmap, `b55a721d1` code, `207756aae` sitting. Review dispatched (opus). I verified the sitting rows myself.

All 24 cross-build `instructions:u` cells for `b55a721d1` checked against the TSV: nothing crosses 1%.
`strings ir small` is **1.009685**, identical to Task 9's, and the absolutes give pinned
8053980111 against head 8131979701 on `ir small`. **This task's own contribution is -0.000180 per pass**,
below the pinned build's own run-to-run spread -- so nothing measurable was added, and
**1.695467 instructions per pass remain** of the 53.695182 ceiling.

**The report states a limit on its own acceptance, and it is the sharpest thing in it.** No axis
exercises the send path -- `dispatch.rex` is rc 120 on `~new` -- so the sitting says *nothing added is on
an executed path*, **not** *the send path did not get slower*. The brief called the sitting this task's
real acceptance, so **the brief's own acceptance is weaker than the brief said**, and I have asked the
reviewer to rule on whether a send-exercising probe is owed here rather than left to the task that first
makes one runnable.

**The third mutation did more than pass.** It established that the existing `SELF`/`SUPER` test
*cannot* catch a receiver bound from `resolution.scope`, because its method sits on the class the send
names -- so an inherited class method is what makes receiver-versus-scope visible, and a re-derived
receiver is a wrong answer nothing else in the tree sees. That is a measurement over the whole suite
rather than a claim about the new test.

**The stale-binary hazard fired and was caught by hash**, first time on this plan: after the mutation
runs `target/release/rexx-run` was still the m3 mutant at `9ec0c7eb...`, and the rebuild at the commit
gives `2fed9841...`, matching the binary the interim sitting measured.

**Two self-corrections worth keeping.** A ratio-comparison paragraph written by eye named the wrong
differing cells -- running the comparison found three, not two, and one it had named is identical in both
runs. And `libtest`'s panic lines came from the **mutants**, whose line numbers shift when a mutation
deletes lines, so two of four assertion citations were off by three; all four re-read at HEAD.

**A trap recorded for Task 13:** `Caller::package`'s `None` means "no activation" while
`Interp::package_objects`' `None` key means "the `REXX` package". Two `None`s, different meanings.

### Task 10 review: **Needs fixes**, three Important. Round 1 dispatched. Three of the minors are statements I repeated upward.

**Correcting myself first.** Three claims I passed on from the report are false:

* *"below the pinned build's own run-to-run spread"* -- the contribution is **0.000180** against a spread
  of **0.000092**, so it is twice the spread, not below it. The conclusion that nothing measurable was
  added survives on the ratio being identical to six decimals; the reason I gave for it does not.
* `expr.rs:867`/`:933` for the two `ExprKind::Message` constructors are **`:866`/`:932`**.
* *"four `~` characters"* in `alloc4c.rex` is a `grep -c` **line** count: **8 occurrences on 4 lines**.
  I described a line count as a character count while making a point about anchoring counts.

**Item 1 is a real divergence whose comment is worse than its value.** `Entered::Label` gets
`receiver: None`, but `RexxActivation::internalCall` passes **the caller's receiver**
(`RexxActivation.cpp:3313`, assigned at `:474`) and only `internalCallTrap` passes `OREF_NULL`
(`:3343`). Measured on the oracle: `CALL inner` inside a class method reaching `self~priv` is **rc 0**;
from a `CALL ON ERROR` handler, **rc 159 / 97.2**; from the top level the same, as control. **The new
comment asserts the wrong value is right**, and Task 13 reads the comment before the code -- which is
exactly what landing this signature early was supposed to prevent.

**Item 2, ruled: make "no activation" a variant, both sides.** `Caller::package`'s `None` and
`Interp::package_objects`' `None` are the same type with opposite meanings, and the only guard is a
`debug_assert` **no in-tree path can falsify**. This plan's record on asking a comment to carry an
invariant is three shipped instances, the third from a dispatch that warned about it.

**Item 3 is against the brief as much as against the task, and the reviewer did the thing that proves
it.** The brief called the sitting this task's real acceptance and the sitting cannot see the send path
-- **and the figure was achievable all along**: interleaved, three rounds, an `s~length` loop at
**1.00155** ir / 1.00153 tw and a `.K~m` class-method loop at **1.00090** ir / 1.00119 tw. Green. A
reviewer producing the measurement the task's own acceptance was missing is the strongest form of that
finding, because it removes the argument that it could not be had.

**And the `SmallInt` arm is a pure no-op today** -- every consumer folds it, and the oracle shows a small
integer indistinguishable from the equivalent string across every question asked, both behaviours
enumerating the same methods. That is fine, because D24 asks for the arm structurally; the rationale
comment is what claims more than the arm does.

**One minor outranks its severity because of where it lives:** roadmap `:494`'s "nothing reads it" is
false -- `rexx-classes/tests/behaviour_wiring.rs:807`-`:825` reads it -- and the roadmap is tracked.

### Task 10, fix round 1 committed: `1d7bc4b95`. Sitting owed and running. Plan amended at `936b5687f`.

Five gates green, corpus 143 of 143, both tables' 5a counts unchanged again. One test more than round 1:
`a_trap_handler_and_an_internal_call_do_not_carry_the_same_receiver`.

**The `.text` predicate was used to conclude a sitting IS owed, which is its first use in that
direction.** `size -A` gives 1,221,865 bytes at `5fd2002cf` against 1,226,713 at this head, with both
sha256 sums stated. Every previous use of that predicate on this plan concluded "no sitting needed"; a
predicate that has only ever said no is one nobody has seen work.

**Item 1 landed at the type level and found a third arm.** `CallEntry { Written, Trap }` with a pure
`entered_receiver`, and the `::ROUTINE` route **measured rather than assumed**: rc 159 / 97.2, so a
routine does not inherit the receiver either. My brief named two arms; the decision is three-way.

**Item 2's `debug_assert` was deleted rather than kept, and what replaced it carries a control.**
`plan::Package { Rexx, Program(ProgramId) }` and `CallerPackage { NoActivation, Package(Package) }` make
both absences variants, and the accessors stay alive under
`#[allow(dead_code, reason = "read by Task 13's PRIVATE check")]` -- **with the allow itself controlled**:
stripping the attributes yields exactly `methods 'receiver' and 'package' are never used`. An `allow` is
normally invisible to every check in the tree, so establishing that it is load-bearing and names the
right thing closes a hazard nothing else in the process looks at. And the round-1 report's "trap for
whoever writes the PACKAGE check" is now **closed rather than documented**, because the ruling made both
sides variants.

**The four false statements are corrected in place in the earlier sections, not only in the fix-round
section**, each saying what it said first. That is the disposal Task 6 got wrong in the opposite
direction, where two false sentences stayed verbatim while a corrected version was added elsewhere.

**And my own instruction was narrower than the data.** I said the `tw` arm is the one to reason from;
five rounds shows `send_call` exact on **both** arms (min = median = max) while the two *send* probes
show single-round excursions of about 0.4% on `tw` as well. The correct statement is the one the report
now makes: medians over five rounds are the figures, and a single round of either arm is not evidence.

### Task 10, fix round 1 complete: `1d7bc4b95` code, `0d8203280` sitting. Re-review dispatched (sonnet). Sitting verified by me.

24 cross-build `instructions:u` cells for `1d7bc4b95`: twelve flat -- `compound`, `emptyloop`,
`varlookup` -- and twelve matching the standing pattern, nothing at or above 1%, `strings`/`ir`/small
still 1.009685. Per-pass 51.999693 against the 53.695178 guard, **1.695485 left**.

**The item 3 figures came back with medians over five rounds:** **+3.03** per native send, **+11.95** per
method send, **+18.00** per internal `CALL`, the last exact on both arms. And a sharpening on the
attribution I would not have made: **the probe's `CALL` is top-level, so both builds compute the same
value** -- which is what makes "the cost appears when the decision is computed rather than folded" a
statement about the computation rather than about the answer.

**A finding of my own that no review named, and I have sent it to be verified rather than asserted.**
The `task` column of `phase-5a-arms.tsv` holds `6`, `7`, `8`, `9`, `9-fixround-1`, `10`,
`10-fixround-1`. **Tasks 6, 7 and 8 recorded their fix-round sittings under the bare task number**,
distinguished only by the commit column -- Task 8's implementer chose that deliberately and called it
this plan's convention -- **while Tasks 9 and 10 introduced a `-fixround-N` suffix.** So a later query
filtering `task == "9"` sees one sitting where `task == "8"` sees several, and the file's own
`README.md` and `PINNED.md` both mention the column. **The rows are measurements and must not be
rewritten**; the question is where a reader is told both spellings exist. Ruling deferred to the
re-review's confirmation, because a claim about a committed data file's contents is exactly the kind I
have been wrong about this session.

### Task 10, round 1 re-review: substantially solid. Round 2 dispatched -- three documentation fixes, no behaviour. Task 10 closes on it.

**Everything load-bearing was reproduced rather than read.** The four-arm receiver decision confirmed by
the reviewer's own oracle probes; the test reddened **structurally** when the `Trap` arm was mutated in a
sandbox; the `allow` control real, naming exactly `methods 'receiver' and 'package' are never used` at
the accessors' `impl` block rather than at the fields; all five C++ citations correct; and **+18 per
`CALL` independently reproduced by `perf stat` against a sha256-verified rebuild at 17.999847.**

**None of round 1's four false statements survives anywhere in the report**, checked for orphan copies
as well as for the wrong figures -- which is the disposal Task 6 failed in the other direction.

Two defects survive, both documentation:

* **A false sentence in new code.** `plan.rs:53` says `Interp::class_packages` keys on `Package`; it is
  `HashMap<ObjRef, ProgramId>` at `lib.rs:2347`, untouched by the diff.
* **A stale claim in the report's "What I could not close"**, still asserting that `resolve`'s
  `debug_assert` stands behind `Caller`'s fields when this round's own item 2 deleted it -- **one
  paragraph from the neighbour that was explicitly closed out.** The section a reader most trusts to be
  current is the one least likely to be re-read when the thing it describes gets closed.

**And my task-column framing was imprecise, which the re-review corrected.** Task `9` bare also holds
two commits, so "one sitting versus several" was wrong. The exact asymmetry: **only the reviewer-driven
fix round moved to a `-fixround-N` suffix, and only for Tasks 9 and 10**; Tasks 6, 7 and 8 recorded
theirs under the bare number. Neither `README.md` nor `PINNED.md` documents either spelling.

**Ruling: document both spellings in `rust/bench-baselines/README.md`, name the commit column as the
reliable discriminator, and do not touch the committed rows.** They are measurements; rewriting them to
a convention decided afterwards would be editing data to fit a rule that did not exist when it was
taken.

**One process note carried into the round:** the re-review could not confirm gate 5's exit code because
it ran as a background job rather than the shell's own child. All five gates run as this shell's
children so each `$?` belongs to its command.

### Task 10: complete. Round 2 `5957d6726`. **PAUSED HERE at Moritz's instruction: stop after Task 10.**

Verified by me at the stopping point: HEAD `5957d6726`, tree clean, 76 commits on the branch since the
phase pin, `phase-5a.txt` at 304 lines, and `bench-baselines/README.md` documenting both `task` column
spellings with the commit column named as the discriminator.

**Item 2 was worse than the brief knew, and re-reading the whole section is what found it.** The brief
named one stale item; there were three. The worst asserted that *"the instrument that would say the send
path did not get slower does not exist in this plan"* -- **which round 1's own item 3 built.** That is
not a stale sentence, it is a false claim of non-existence that would have talked the next reader out of
doing something already done. All three corrected in place, each saying what it said first.

**The lesson is the section, not the sentence.** "What I could not close" is the section a reader most
trusts to be current and the one least likely to be re-read when the thing it describes gets closed. A
brief naming one item in it will find one item; re-reading it found three.

**The `.text` predicate got its most careful use yet, and a control.** `.text` sha256 and byte count
identical on both sides while the whole-file sums differ -- which is exactly why the predicate names
`.text` and not the file. One section did move, `.data.rel.ro`, and rather than guess, the same comment
text was placed at the **end** of `plan.rs`, where it shifts no item's line number, and recovered the
pre-round section byte for byte. So what moved is line-number data for the code below the insertion. No
sitting owed, none run.

**Gate 5's status is now the shell's own child**, which is the one thing the previous re-review could not
confirm.

## Resume point

* **Next:** Task 11 -- the Array and the Directory that 5a's own mechanisms send to. Run the
  pre-dispatch prerequisite check first; it has found defects in four of the eight tasks it has been
  applied to.
* **Budget:** 1.695485 instructions per pass remain of the 53.695178 ceiling on `strings`/`ir`. A
  `try_text` arm costs 48 per pass and is unaffordable; `to_text` plus `text_len` cost 4 together.
* **Send and call paths:** Tasks 12, 13 and 21 report against Task 10's baseline -- about 3 per native
  send, about 12 per method send, +18 per internal `CALL` -- or state they do not touch them.
* **Task 13 owes:** the eleven registered classes whose wiring rows need the prologue's `~inherit`
  calls. **Task 21 owes:** `Queue`, `Stem`, `VariableReference`.
* **Ordering still binding:** 11 before 12; 17 before 21; 9 and 12 before 21; 21 before 23.
* **Open for Moritz, unchanged:** whether the byte-equivalence method goes into `rust/CLAUDE.md` under
  Gates (four-line draft parked in the Task 8 report under a NOT APPLIED heading), and whether the
  metaclass sufficiency counterexample should become an **asserted** row in Task 8's in-crate test
  rather than a documented one.

---

## Task 11 -- the Array and the Directory that 5a's own mechanisms send to

**Two follow-ups from Moritz closed before this task started.** Both were the open items the previous
resume point left him.

* **The sitting requirement is a question, not a proof obligation** (`3a03ca4b9`). His words: "some
  changes cannot possibly affect performance, and such changes can just ignore the requirement... the
  goal was not reproducible Rust builds." The constraint now asks "can this change affect what
  executes?", says that a comment, a test, a corpus program or a document plainly cannot and runs no
  sitting, and records why comparing binaries is the wrong instrument so nobody rebuilds that
  apparatus. **This retires the `rust/CLAUDE.md` question as moot**: the byte-equivalence method is not
  going into the gates, and the four-line draft parked in the Task 8 report stays NOT APPLIED.
* **The metaclass counterexample is now an asserted row** (`21cde29af`), his call. Added, then measured:
  **the pair cannot fail.** On that row `Class` is both the superclass and the metaclass a bare
  declaration inherits, so both fields read the same object and no rule over those two inputs can
  separate them. Verified by collapsing `registry.rs`'s `class_of` onto the `metaclass` field -- the
  test fails at `T~class` and never reaches the new pair; `registry.rs` restored and `diff -q`
  confirmed identical. Labelled `stated, cannot fail` in the code and in the doc table, with `T`/`T2`
  named as what actually guards the split. A counterexample to a sentence is not automatically a test.

**Pre-dispatch prerequisite check: four structural claims held, every measurement reproduced, one
premise was stale.**

* Held: `owners.rs:251` has `ExprKind::List` at `Owner::Phase("Phase 5")`, `:389` carries the pinned
  row, `:259` has `LoopKind::Over` already `Owner::InScope`, and `body.rs:292` is the `<= 80` assert.
* Reproduced: `say (1,)~size` is oracle rc 0 `2` against crate rc 120 `a parenthesised list is not
  implemented (Phase 5)` on **both** engines; `do i over 'abc'` is rc 0 `abc` on the oracle and both
  engines.
* **Ruling -- the task does not add a value kind, and the plan said it did** (`fc7e452bc`). Measured:
  `Body::Array(Vec<ObjRef>)` is `body.rs:109` and **Task 9 already allocates one** at `dispatch.rs:1382`
  for `~superClasses`, and a directory's store is already `NativeObject`'s `entries` map. The task adds
  *methods* over both. Both carried constraints stay stated, because a variant added anyway would trip
  them, but the expected outcome is that neither is exercised and the report says so in a sentence.
  **Cost if wrong:** none to the code; the cost of leaving it was a report justifying a boxing decision
  the task never reached, which is the false-justification shape that has now ridden a correct decision
  five times.
* Carried into the dispatch: the **Task 17 boundary**. Task 17 owns the Directory entry-method
  mechanism and depends on this task's `~put`, so this task builds `~put`/`[]`/`~at` and not
  entry-name-as-message dispatch.

**BASE for the review package:** `fc7e452bc`. Implementer dispatched on opus.

**Two later tasks got their oracle-side prerequisite check while Task 11 ran**, because the oracle's
answers do not move when our tree does. Both found a hole.

* **Task 12's programs did not determine their own exit status** (`52fc1d94a`). The plan promised
  `say .k~zork(1,2)` at oracle rc 0 with `unknown: ZORK`, and named the method's `use arg n, a` but
  not what it does with the result. Measured all three shapes: a body that only `say`s gives
  **rc 165** and `Error 91.999: Message "ZORK" did not return a result.` because `say` demands one;
  the body **returning** `'unknown:' n` gives rc 0 `unknown: ZORK`; and the args arm reads best as a
  bare statement, `.k~zork(1,2)` with the method printing `a~items`, oracle rc 0 `unknown: ZORK
  args 2`. **Cost if left:** the implementer writes the obvious program, gets rc 165 against a plan
  promising rc 0, and either reports the plan wrong or quietly changes the target.
* **Task 13 asked for a measurement it did not carry** (`aba8b555e`). It named "a subclass's method
  sending a superclass's private method, whose answer is measured rather than assumed" and gave no
  answer. Measured: **oracle rc 0 `inner`.** A class-side method on `::CLASS Sub SUBCLASS Base`
  sending `self~m`, where `m` is `Base`'s `PRIVATE` class method, is answered rather than refused, so
  a limb written as "only the defining scope" would redden it. The refusals are the sibling and the
  outside caller, and the **sibling's traceback is two frames**, the inner `return .K~m` line before
  the outer `say .S~poke` line, which is the byte-for-byte target. Also reproduced unchanged: the
  outside caller at rc 159 `97.2`, its `self~m` twin at rc 0 `inner`, `PROTECTED` at rc 0, and
  `PACKAGE`'s same-package arm at rc 0 `pkg`.

**Ruling on both:** amend in place and keep the ordering. Neither hole is a reason to re-sequence;
both are the plan failing to say which program it means, which is the shape the prerequisite check
exists to catch. **Cost if wrong:** a program shape pinned here that the task later finds a better
form of, which is a cheap edit.

## Task 11: complete

Commits `f4b21eadb` (the implementation) and `6f3434e88` (two performance amendments the sitting
found). Report: `task-11-report.md`. Gates: all five green at `6f3434e88`, corpus **149 of 149**,
up from 143 of 143.

**Built:** an Array's `[]`/`AT`/`SIZE`/`ITEMS`, a Directory's `[]`/`AT`/`PUT`, `ExprKind::List` as a
real `.Array`, and `DO OVER` an Array. `ExprKind::List` moved to `Owner::InScope` in all five pinned
items; `LoopKind::Over` was already in scope and was not touched. Six corpus programs added.

**The ruling held: no `Body` variant was added**, so neither `body.rs:292`'s size assertion nor Q4's
boxing rule was exercised, and the report says so in a sentence rather than justifying a decision the
task did not reach. `Body::Array`'s payload did widen, from `Vec<ObjRef>` to `Vec<Option<ObjRef>>`,
and that is forced: measured, `(1,,3)~items` is `2` where `(1,.nil,3)~items` is `3` and both are
`~size` `3`, so an empty slot and a slot holding `.nil` are different values that `~at` answers
identically. Both are three words, so `size_of::<Body>()` did not move.

**Two oracle rules the task had to read and no plan document carried.**

* **Every trace value line and every error-message substitution of an object renders through
  `stringValue()`, not through the string value a string context asks for.** Measured both halves:
  `trace i` over `a = (1,,3)` prints `>>>   "an Array"` where `say '<'||a||'>'` prints its items on
  two lines, and `say 1 + (1,2)` is 41.1 quoting `"an Array"`. The contrast that bounds it: an
  instruction or builtin that **converts** first and quotes the converted string keeps the joined
  text -- `translate('abc','x','y',(1,2))` is 40.23 quoting `"1` / `2"`, and `numeric digits (3,)`
  really does set DIGITS to 3. `Interp::string_value_text` is the one place the split lives.
* **`FOR` is consulted after the item is bound**, which is
  `RexxInstructionDoOverFor::iterate`'s own `checkOver(...) && checkFor()`
  (`instructions/DoOverInstruction.cpp:279`). `do e over a for 2` on a four-item array leaves the
  control variable at the **third** item. This closed a pre-existing defect no test covered:
  `do e over 'abc' for 0` left the variable unset here against the oracle's `abc`.

**Two refusals added on purpose, each with its own instrument, because the corpus gate cannot see
either.** A directory index the oracle's own directory holds and this crate does not build is loud
**per directory** -- measured, `.local['STDOUT']` is an object and `.environment['STDOUT']` is `.nil`.
And a name a receiver's behaviour does not answer is loud when that behaviour answers `UNKNOWN`,
because the oracle forwards there: `.environment~nosuch` is `The NIL object` at rc 0, so 97.1 would
be a wrong answer a program can trap. Task 17 still owns the `UNKNOWN` mechanism itself.

**A new oracle crasher, recorded as `corpus/oracle-crashes.txt` entry 6.** `a = (1,2); say a~at((,2))`
is SIGSEGV rc 139, 3 runs of 3: `ArrayClass::validateIndex` spreads a lone array argument by **item
count** with **slot array** (`classes/ArrayClass.cpp:1219`-`:1226`), so an array whose leading slot
is empty and whose item count is one hands `validateSingleDimensionIndex` a null subscript. The
neighbours are clean and bound it -- `a~at((1,))` answers, `a~at((1,,3))` is 93.926, `a~at((,))` is
93.901. No differential row can cover the crashing shape, so `dispatch.rs`'s
`an_expanded_index_of_one_empty_slot_is_loud` is the whole instrument.

**The sitting, and what the control actually established.** `strings`/`ir` is flat: +51.999691 per
pass against the 53.695178 ceiling, leaving 1.695487 where the brief carried 1.695485. The first
sitting at `f4b21eadb` read `strings`/tw at +1.371%, above the threshold, so the interleaved control
ran -- the same source built twice differing by one comment -- and came back **exactly 1.000000 on
every axis and both arms**. That is not the change being exonerated: `instructions:u` is deterministic
under source perturbation, so the control **cannot fail on that instrument** and what it established
was the instrument's determinism. Layout excluded, a bisection found two real causes, both fixed in
`6f3434e88` and both on programs containing no list and tracing nothing: an arm of its own for
`ExprKind::List` in `eval_node`'s match cost `strings` 43 instructions per pass (that match is the
tree-walker's whole expression dispatch), and `string_value_text` inlined into the two
`#[inline(always)]` trace gates cost a further 29. After both, no axis reaches 1%. A residual of 10
and 25 per pass is bounded but **not attributed**: the bisection stopped at `dispatch.rs`, which does
not compile in isolation against the new payload.

**Two divergence families found and left open, both pre-existing** (`.Array~superClasses` has
returned an array since Task 9, so every position was already reachable). `NUMERIC DIGITS`/`FUZZ`/
`FORM VALUE` quote the joined text where the oracle quotes `an Array`, and closing it means splitting
`Settings::set_digits_str`'s parse input from its message substitution -- an `activation.rs` API
change, since `numeric digits (3,)` proves the two must part. And `interpret` of a value holding a
newline reaches `lib.rs`'s already-documented parse-error gap by a new route. `builtin::whole_number`
and the 41.1/26.8/26.2/26.3 sites **were** closed, because their value and message paths were already
separate.

**One item outside the pinned five had to move**: `spike.rs`'s two loud witnesses were `say (1, 2)`
and both went red the moment the list evaluated. They are `ExprKind::ClassResolver` now, the sixth
witness that file has had, and its doc records the sequence rather than predicting which form lasts.

**Task 11 implementer returned DONE_WITH_CONCERNS at `f92e00b86`** -- three commits, `f4b21eadb`
implementation, `6f3434e88` two performance fixes the sitting found, `f92e00b86` the sitting rows.
Verified by me before dispatching the review, not taken from the report:

* Gates: `cargo fmt --all --check` exit 0, `cargo clippy --workspace --all-targets -- -D warnings`
  exit 0, and `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` exit 0 with 98
  `test result: ok` lines, no `failures:` and no `FAILED`. The corpus report block reads
  `mode: STRICT (the gate)` and **149 of 149 matching**, up from 143.
* **The GC consequence of widening `Body::Array` to `Vec<Option<ObjRef>>` is handled**:
  `body.rs:325` is `out.extend(items.iter().filter_map(|item| *item))`, so a filled slot is traced
  and an empty one contributes nothing. This was the one claim I would not have let a report carry
  alone, because a missed trace is a use-after-free no test in this suite would show.
* **The "these divergences predate this task" claim holds, and I checked it the way that could have
  falsified it** -- by reaching the family with no list in the program at all.
  `numeric digits .Array~superClasses` is rc 230 on both sides with the oracle saying
  `found "an Array"` and both crate engines saying `found "The Object class"`. `~superClasses` has
  answered an array since Task 9, so the route is Task 9's and this task inherits it rather than
  opening it.

**One concern of the implementer's is a live finding I am holding for the reviewer.** It reports
moving `spike.rs`'s two loud witnesses as "the sixth item beyond the pinned five", and
`owners.rs:590` still reads "every one of the **five** items below". Either that sentence is now
false or the move was not one of its items. This is the universal-quantifier-over-an-enumerable-set
tell, and the comment rule against naming a set's size would have prevented the sentence.

**Task 11 review: spec compliance PASS, task quality PASS WITH FINDINGS.** Two findings block, both
about claims rather than behaviour, and both in prose this commit itself wrote.

* **B1 -- four instruction figures computed by three different subtractions.** The `List`-arm and
  trace-gate costs baked into `eval.rs`'s and `value.rs`'s doc comments are not all the marginal
  reading the sentences claim: two are, `strings` 43 is `D - A` and `varlookup` 7 is `C - D`. And the
  decomposition contradicts itself -- 43 + 29 is exactly the whole move, so the two causes account for
  everything, while the next paragraph reports 33 and 12 unattributed. **Ruling: the marginal reading
  throughout** (`strings` 39 / `varlookup` 4 for the arm, `strings` 29 / `varlookup` 11 for the gates),
  each comment saying the two are not additive, "costs both nothing" deleted because the outlined
  shape sits 12 and 33 above `HEAD~1`, and the bisection variants either committed to the TSV under
  their own label or declared unrecoverable. **Cost if wrong:** a reader re-deriving a figure from the
  wrong subtraction, which is what the finding already caught.
* **B2 -- the rewritten `Raise` ownership sentence asserts a state a run contradicts.** Verified by me,
  not taken from the review: `raise syntax 93.900 additional (1,2)` is oracle rc 163 `Error 93.900:
  1.` against crate **rc 120** `an array as a RAISE ADDITIONAL value is not implemented (Phase 5)`,
  while the `array (1,2)` spelling is byte-identical on both sides. The comment says `ADDITIONAL
  (a, b)` "is implemented" and that both spellings reach identical oracle bytes. The refusal predates
  this task -- reachable at Task 9 through `additional (.Array~superClasses)` -- so the finding is the
  claim, but this commit rewrote exactly that sentence and owed it a run.
  **Ruling: honest bookkeeping unconditionally, behaviour fix under a cap.** An arm-grained row and
  owner for `ADDITIONAL <array>` must land, so the refusal is owned and `loud.rs`'s witness machinery
  covers it. The behaviour fix is authorized **only** if wiring the value through the expansion the
  `raise.array` arm already performs is contained: no new mechanism, a corpus row, both engines
  byte-identical. Otherwise stop and say what the larger change would have been. **Cost if wrong:**
  either a refusal that stands one task longer than it had to, or a `RAISE` change inside the Array
  task, which is why the cap is on it rather than a preference.

**Ruled not owed:** the `owners.rs:590` "five items" sentence. The reviewer's disposition is that a
variant move must update all five and that claim is a necessary condition which still holds; the
`spike.rs` witnesses are not ownership data, so the move was not a sixth item. My earlier suspicion
that the sentence had gone false was wrong, and the block stays as it is.

**One measurement error of my own, in the same shape this project keeps hitting.** I reported the TSV
had no `task` `11` rows after a `grep` whose pattern used spaces where the file has tabs. It has them.
A negative claim is exactly as wide as its pattern.

Fix round 1 dispatched to the same implementer.

## Task 11 fix round 1: complete

Commits `071835c9c`, `91935800d`, `fefe33597`. Review: `task-11-review.md` (spec
compliance PASS, task quality PASS WITH FINDINGS, two blocking). All five gates green,
corpus **152 of 152**, up from 149 of 149.

**B2 was a behaviour defect wearing a prose finding's clothes, and the honest bookkeeping
turned out not to be needed.** The comment this task rewrote said `ADDITIONAL (a, b)` was
implemented because the parenthesised list is; measured, it was rc 120 where the oracle
reports at rc 216, while `array (1,2)` was already byte-identical. The fix is contained to
the arm that existed: `RexxObject::requestArray` answers an array **unchanged** rather
than compacting it, so its slots are the substitution list -- which is what the `ARRAY`
arm already builds. So the arm-grained ownership row the ruling required as the floor was
not needed, and the corrected sentences say what does have no code: an `ADDITIONAL` value
under `SYNTAX` that is a class object or one of the interpreter's own, refused through
`Loud::object_position`, which is not reached through `Loud::instruction` and so reads no
owner -- the shape `lib.rs` already documents for `Expose`.

**Three keyword value lines were wrong for the reason the task's own report already gave
about every other trace line, and the review did not name them.** `traceKeywordResult` is
handed the object at the condition keyword, at `DESCRIPTION` and at `ADDITIONAL`, and each
separately converts for its own data. Found by re-reading the fix's neighbourhood rather
than the line the finding named, which is the rule this plan put in place after fix rounds
introduced new false statements around their own fixes.

**The first two corpus programs I wrote for B2 witnessed nothing, and the controls are what
said so.** A trapped ladder printing `rc'.'condition('E')` stayed fully green under a
mutation closing the empty slot up *and* under one rendering the traced object as its
string value: a trap exposes only the number and sub, the substituted message exists only
in the untrapped report, and `condition('A')` would answer the array itself, which this
crate refuses. The rewrite is three untrapped, traced programs in `RAW_STDERR_COMPARISON`,
and all five mutations behind them redden the corpus. **"Can fail" was not the question;
"does this program contain the channel the fix is visible on" was.**

**B1 was three subtractions across four figures and one asserted zero.** Two of the
published per-cause numbers were not the quantity their sentence claimed, and
`eval_cold`'s "costs both nothing" asserted a zero the same bisection read as 33 and 12
instructions per pass. The bisection is now six binaries, each rooted at the same commit
and each carrying one change, measured in one interleaved sitting under
`task` `11-bisection` with the `build` column naming each variant -- so a reader subtracts
the committed rows instead of trusting a sentence. Marginal: the `eval_node` arm 39 and 4,
the trace gates 29 and 11; they sum to 67 and 15 against a whole move of 72 and 16, which
the comments now say. **And this round put a wrong number in on its own first pass**: 38
for the arm's `strings` figure, a misreading of the two rows, corrected at `fefe33597`
with the commit message saying so.

**What the layout control actually establishes, now written down where it was over-read.**
A rebuild differing by one comment produces identical machine code, so an
`instructions:u` ratio of exactly 1.000000 was forced: the control cannot fail on that
instrument. The counter licenses reading a non-zero move as executed work; the control
does not. The report's "on this instrument layout is not a term at all" is gone.

Observations O1, O2, O3, O4, O5, O7 and O8 fixed; **O6 skipped on the coordinator's
instruction** (the `owners.rs` five-item claim is a necessary condition and still true).
The residual in the bisection is derivable now and still unattributed.

**Fix round 1 closed both findings, verified independently.** Commits `071835c9c`, `91935800d`,
`fefe33597`.

* **B1 CLOSED.** I recomputed all four figures off the committed `11-bisection` rows rather than
  reading the report's table: on `per_pass`/tw, `b - c` is 39.000076 (`strings`) and 3.999885
  (`varlookup`), `b - d` is 29.000050 and 10.999892, `e - a` is 32.999268 and 11.999993, `b - a` is
  71.999695 and 15.999905, and the comment control `f - e` is 0.000104 and -0.000006. Every figure now
  in a comment is the subtraction its sentence claims. The round put a wrong number in on its own first
  pass, 38 for the arm's `strings` figure, and corrected it at `fefe33597` -- the fourth time on this
  plan that a correction round has introduced a new false number while removing an old one.
* **B2 CLOSED, and the behaviour fix was taken rather than the bookkeeping.** Verified by me on four
  shapes, both engines, three descriptors: `additional (1,2)`, `array (1,2)`, `additional (1,,3)` and
  `additional ('R',,'X')` are all byte-identical to the oracle. Gates on `fefe33597`: fmt 0, clippy 0,
  gated release suite 0 with 98 `test result: ok`, corpus **152 of 152** STRICT.
* The re-review also killed one of the implementer's own concerns: the object-valued `ADDITIONAL`
  refusal **does** have an in-crate test, `eval.rs:3669`'s
  `an_object_as_a_raise_syntax_substitution_is_loud`, confirmed load-bearing by mutating the gate.
  The report's "no in-crate test" sentence is the false claim, not the code.

**Fix round 2 dispatched, over a finding the re-review ruled out of scope and I ruled in.** Measured:
`raise syntax 93.900 array ((1,2),3)` is oracle `Error 93.900:  an Array.` against crate
`Error 93.900:  1` newline `2.` -- **a wrong answer, not a refusal** -- while the same program spelled
`additional ((1,2),3)` agrees, because the path this round built names the object and the older `ARRAY`
arm joins its elements. **Ruling: in scope.** A nested array as a substitution item needs a list inside
a list, so the shape was unreachable before `ExprKind::List` evaluated, which makes it this task's to
own. I told the implementer to **establish that by building `f4b21eadb~1`** rather than take my
reasoning, because this is an unreachability claim and those have been wrong here before, and to fix
the rendering either way. **Cost if wrong:** a rendering fix landing in the Array task that a later
task would otherwise have made, which is cheap; the cost of the other ruling was shipping a wrong
answer where a refusal used to stand.

## Task 11 fix round 2: complete

Commits `3983105e1` and `0ef493ae7`. Re-review closed B1 and B2; two items left, both
closed. All five gates green, corpus **154 of 154**, up from 152 of 152.

**The nested-array shape is this task's, and the coordinator was right to ask for the run rather
than the argument.** A nested substitution item needs a list inside a list, so at
`f4b21eadb~1` both `raise syntax 93.900 array ((1,2),3)` and the `additional` spelling are rc 120
`a parenthesised list is not implemented (Phase 5)` on both engines. The task made the shape
reachable and owns it, so this is a fix and not a recorded divergence. (`f4b21eadb~1` is
`aba8b555e`, and the binary run was built from `21cde29af`'s crates; `git diff --stat` over the
crate pathspec between the two is empty and the commits between them touch only the plan.)

The `ARRAY` arm rendered its elements with `to_text`, so a nested item reported and traced its
inner elements joined where the oracle names the object. The arm builds its rendering once, so
one substitution to `Interp::string_value_text` fixes the report and both `>A>` lines, and it is
the renderer the `ADDITIONAL` arm already reaches. **The two arms share the renderer and not the
slot accessor**, which is the construct rather than a compromise: an `ARRAY` list's elements come
from the parse and there is no array object to read slots off. 13 programs across the nested
neighbourhood, both engines, all byte-identical; two corpus rows, one per spelling; both
mutations redden.

**A "no test covers X" sentence was wrong for the third time on this task, and the pattern is now
named.** Fix round 1 said the object-valued `ADDITIONAL` refusal had no in-crate test;
`eval.rs`'s `an_object_as_a_raise_syntax_substitution_is_loud` covers exactly it and predates the
task. Auditing every sentence of that class found a second: round 1's claim that no test covered
the `DO OVER ... FOR` defect. `run/tests.rs`'s `do_over_for_0_skips_the_single_non_stem_iteration`
covered the shape and asserted the half that was right (the body does not run); nothing asserted
the control variable afterwards, and that test's doc called the rule a judgement call, which
stopped being true when round 1 measured `checkOver(...) && checkFor()`. Doc corrected and the
missing assertion added. **In all three cases a test existed and the sentence was written from
memory of what this task added rather than from a search of the test files** -- the check that
works is grepping for the construct before writing the sentence.

**A measurement-methodology finding, recorded because it bears on how this plan's reports may be
read.** The *pinned* build's own per-pass figure moved between sittings on two axes for the same
binary and program -- `arith`/`ir` 23764.5222 against 24059.5738, `alloc4c`/`ir` 3591.3984 against
3594.6850 -- while the within-sitting `head - pinned` difference held to a ten-thousandth
(+2.9981 against +3.0008, and +8.9985 against +9.0012). So **a per-pass figure is comparable
within a sitting and not across sittings on those axes**, which is what the interleaving rule
already says and is now measured. Every subtraction the Task 11 report performs is within one
sitting, including the six-build bisection, so no claim there depends on the comparison this
invalidates.

Round 2 moved nothing measurable: every `across_builds` figure equals round 1's to three decimal
places, nothing reaches 1%, and `strings`/`ir` is +52.0003 against the 53.695178 ceiling.

**Fix round 2 closed the finding, and the round-2 re-review found the two false statements the round
itself introduced.** Commits `3983105e1`, `0ef493ae7`.

* **The nested rendering is fixed and I verified it on four shapes**, both engines, three descriptors:
  `array ((1,2),3)`, `additional ((1,2),3)`, `array ((1,,3),3)` and the doubly nested
  `array (((1,2),3),4)` are all `Error 93.900:  an Array.` byte for byte. Gates exit 0, corpus
  **154 of 154**.
* **My in-scope ruling held, and the implementer established it rather than taking my reasoning.**
  Both programs are rc 120 loud on `f4b21eadb~1` -- a nested substitution item needs a list inside a
  list, so this task made the shape reachable. It also caught the provenance wrinkle that the binary
  came from `21cde29af`'s crates, and I checked the same thing independently: `git diff 21cde29af
  aba8b555e -- rust/` is empty, so that binary is `f4b21eadb~1`'s crate source.
* **The prose audit is why the round-2 re-review was scoped at prose.** Two new false statements, both
  introduced by the round that was correcting others, both now corrected by me in place with what they
  said first: (1) "every figure is the round-1 sitting's to three decimal places" is false for two of
  twelve published percentages, `alloc4c`/`ir` +0.271% against +0.270% and `arith`/`tw` +0.151%
  against +0.150%; (2) "that class of sentence has now been wrong three times on this task" counted
  Task 10's send-path episode as the third, which is another task's and which Task 10's own re-review
  recorded ADDRESSED. Two on this task, not three. **This is the fifth round on this plan to add a
  false statement while removing one, and the first where both additions were counts.**
* Confirmed true by the re-review, each by running: the renderer change touches no non-array element
  (12 probes, all byte-identical), `an_object_as_a_raise_syntax_substitution_is_loud` predates the task
  at `e5cce2546`, `do_over_for_0_skips_the_single_non_stem_iteration` now asserts the control variable
  and is load-bearing under a mutation that reorders `loop_advance`'s `OverItems` arm, and the two arms
  share the renderer but not the slot accessor because an `ARRAY` list's elements come from the parse
  where `ADDITIONAL`'s one value is the array.

**One methodology finding this task produced, and it outlives the task.** The pinned build's own
per-pass figure moves between sittings for the same binary -- `arith`/`ir` 23764.5222 against
24059.5738, `alloc4c`/`ir` 3591.3984 against 3594.6850 -- while the within-sitting `head - pinned`
difference holds to a ten-thousandth. **Per-pass is comparable within a sitting, not across sittings.**
Every subtraction this task publishes is within one sitting, the six-build bisection included, so no
claim here depends on it. Later tasks quoting a per-pass figure from an earlier task's row are the
exposure.

**Task 11: complete.** `0ef493ae7`, spec compliance PASS, task quality PASS with both blocking
findings closed and both re-reviews clean on their scopes. Corpus 143 -> 154. Open and carried
forward: the two divergence families in report section 6 (`NUMERIC DIGITS`/`FUZZ`/`FORM VALUE` object
substitution, `INTERPRET` of a value holding a newline), the unattributed bisection residual of 33 and
12 instructions per pass, and oracle-crashes entry 6, unfiled, which is Moritz's call.

## Resume point

* **Next:** Task 12 -- `UNKNOWN` and the NOMETHOD condition. **Paused here at Moritz's instruction.**
  Its program shapes were pinned at `52fc1d94a` before this pause; run the pre-dispatch prerequisite
  check against the tree as it stands after Task 11, because `a~items` now exists and the args-array
  arm's starting measurement will have moved.
* **Budget:** `strings`/`ir` is +52.0003 against the 53.695178 ceiling, leaving 1.6949 instructions
  per pass.
* **Ordering still binding:** 17 before 21; 9 and 12 before 21; 21 before 23.

---

## Task 12 -- `UNKNOWN`, and the NOMETHOD condition

**Pre-dispatch prerequisite check found the task's own starting measurement stale, and Task 11 is why**
(`c18b877e3`). The plan said the crate answers **rc 159, `97.1 does not understand "ZORK"`** for
`say .k~zork(1,2)` with an `UNKNOWN` method -- "a wrong error where the oracle answers". Re-measured,
both engines: the crate is **rc 120**, `rexx-exec: the UNKNOWN forward for message "ZORK" is not
implemented (Phase 5)`. The gate is `lib.rs:828`, its test `dispatch.rs:2674`, and `git log -S` puts
both in **`f4b21eadb`**, Task 11's implementation commit.

**Task 11 was right to add it, which is why this is an amendment and not a finding against it.** Once
`ExprKind::List` evaluated, the send reached dispatch for the first time; without an `UNKNOWN` step the
crate would have answered 97.1 where the oracle answers, a wrong answer newly reachable. It refused
instead. **So Task 12 removes a gate rather than correcting an error** -- a different edit in a
different place, and an implementer working from the old sentence would have gone looking for a 97.1
site to fix.

Also re-measured and carried into the dispatch: the no-argument spelling `say .k~zork` is the same rc
120 against oracle rc 0; `corpus/lang/message_send_unknown_method.rex` (`say 'abc'~nosuchmsg`) still
agrees at rc 159 `97.1` on both sides, so it is the other branch rather than a casualty; and the
NOMETHOD keyword, `COND_NOMETHOD`, its trappable-condition lists and `Builtin::NoMethod` all already
exist on the parse side, which is not the same as the raise side working.

**BASE for the review package:** `c18b877e3`. Implementer dispatched on opus.

**Oracle-side prerequisite checks for Tasks 14 and 15, run while Task 12 was building.** Every claim
in both tasks holds on the oracle exactly as written, including the message text:

* Task 14: `say .k` with a class-side `makeString` returning `'K says hello'` is rc 0 `K says hello`.
  `.K~request("STRING")` is `The NIL object`, and `.K~string` and `.K~objectName` are both
  `The K class`.
* Task 15: `.K~a = 5` then `say .K~a` with `::METHOD a CLASS ATTRIBUTE` is rc 0 `5`; the same shape
  with `::ATTRIBUTE b CLASS` is rc 0 `7`; `::METHOD m CLASS ABSTRACT` **installs** at rc 0, and the
  send is rc 163 `Error 93.965:  Method M is ABSTRACT and cannot be directly invoked.`

The crate halves of these were **not** re-measured here on purpose: Task 12's implementer is running
gates, which rebuilds `target/release/rexx-run`, and a probe against a binary being replaced is not a
measurement. Both tasks get their crate-side check at their own dispatch, which is where a moved
starting point matters -- and on this plan two of the last three starting points had moved.

**Task 12 implementer returned DONE_WITH_CONCERNS at `8cdc40820`** -- five commits, one of them a fix
for a wrong answer it shipped and caught itself. Verified by me before dispatching the review:

* Gates exit 0, 98 `test result: ok`, differential corpus **156 of 156**.
* Byte-identical on three descriptors, both engines, all of: `say .k~zork(1,2)` with a returning
  `UNKNOWN` (rc 0 `unknown: ZORK`); the args-array arm printing `a~items` (rc 0 `unknown: ZORK
  args 2`), which is what Task 11 had to precede this for; `signal on nomethod` trapping a miss
  through an internal `CALL` and in the main body (rc 0 `trapped`); a receiver with no `UNKNOWN`
  (rc 159).
* **The case that caught its wrong answer now matches**, and I ran it rather than taking it: `signal on
  nomethod` in the main body with the miss inside a `::METHOD` body is oracle rc 159 `97.1 Object "The
  K class" does not understand message "ZORK".` on both engines. A `::METHOD` activation does not
  inherit the main body's trap.

**How that defect was found is the part worth keeping.** It read `reportNomethod` as a stack walk,
implemented one, and **three probes agreed**. The fourth did not. It found it because a mutation of
the *correct* code left the corpus green -- so the instrument, not the reasoning, is what caught it.
Three agreeing probes on a rule about scope are not evidence the scope is right.

**Ruling on the stale sentence it found in the binding spec** (`4b836188a`).
`specs/2026-08-17-phase-5-object-model.md:25` still shows the crate answering rc 159 `97.1` for
`.k~zork(1,2)`. **The measurement block stays as it is.** It is dated evidence for why the second spec
exists, framed "Measured this session", and rewriting it would falsify the record of what the argument
was made from -- the same append-only rule the records directory runs on. What a reader needs is where
each row stands, so a note appended below gives each its owner: the `UNKNOWN` row went to rc 120 in
Task 11 and to rc 0 matching in Task 12, and the `makeString` row is unchanged and **Task 14's**.
**Cost if wrong:** a reader takes the dated block as current, which the appended note is what prevents.

**Task 12 review: spec compliance PASS, task quality PASS with required corrections.** Nothing about
the behaviour is wrong. The reviewer checked the corrected trap rule against
`Activity::checkCondition`'s break at the first Rexx activation, ran **eight** probes across the
neighbourhood the fix round did not name -- a miss in a `::ROUTINE` body, method-from-method, a trap
established inside the method body, `~unknown` as the missing name, handler re-entry, `call on error`,
`call on any` against `signal on any`, and `UNKNOWN` beating an armed trap -- all byte-identical on
three descriptors on both engines, and confirmed the stack-walk mutation reddens `condition_nomethod
.rex` alone. The D50 control moves both `unkno` and `xmeths` to `diverge-both`.

**What blocks is two comments that state something false, both confirmed by me:**

* `corpus/phase-5a.txt:359-361` still carries the withdrawn mechanism -- "The offer walks the whole
  activation stack before it degrades, so a caller's NOMETHOD handler beats a callee's SYNTAX handler".
  **The fix commit never touched that file**, which is the lesson: a behaviour fix and the prose
  describing that behaviour live in different files, and reverting the mechanism in code does not
  revert the sentence that sold it.
* `dispatch.rs:1140` says the unrooted-array control makes a row "panic in `to_text`"; measured, it is
  a **loud refusal at rc 120 from the receiver guard**, in release and debug both. The root's
  justification survives; the named symptom is wrong, and a reader reproducing the control would look
  for the wrong thing.

**Folded in as mandatory rather than optional: five set-size counts in new comments**, which the
Global Constraints forbid outright. **Ruling on the TSV:** 312 duplicate-key rows against the README's
documented `commit` discriminator get **distinct keys, not deleted rows** -- relabel one sitting the
way Task 11's bisection took `11-bisection`, and extend the README in the same commit if its
convention does not reach this case. **Cost if wrong:** a label a later task has to interpret, against
the alternative of destroying a measurement.

Fix round 1 dispatched to the same implementer, with the sitting answered in a sentence rather than run,
since no executed code changes.

**Fix round 1 closed clean, and the round-1 re-review found no new false statement** -- the first round
on this plan to correct prose without adding any. Verified by me before dispatching it: duplicate-key
rows **0**, and `cut -f2-` over the TSV before and after the relabel is byte-identical, so only column
1 moved and no measurement was rewritten. Gates exit 0, corpus **156 of 156** unmoved.

* **The README is now true of the file it describes.** It had called `commit` "the reliable
  discriminator", which was false for this file; it now states `(task, commit)`, carries a runnable
  duplicate-key check that prints `0`, and drops a dated enumeration its own command contradicted. The
  re-review ran both documented commands exactly as written.
* **The implementer was right and the reviewer was wrong on the perf figure**, and the reason is that
  two different quantities were being named. Widest ratio *above* 1 in the `12-prev` sitting is
  `1.000000032` on `varlookup`/`ir`/`large`, which is the review's figure, confirmed verbatim; widest
  *departure from 1 in either direction* is `0.999999741` on `alloc4c`/`ir`/`large`, a departure over
  eight times larger. "Six decimal places" is honest and "seven" overstates. **The implementer
  disagreed with a review and named both quantities rather than picking one**, which is what made the
  adjudication a computation instead of an argument.
* **Ruled: the comment rule does not reach pre-existing text on a touched line.** The `exec_call`
  phrase predates this task and names an architecturally fixed fact (D24's two functions) rather than a
  driftable count. Widening a fix round into unrelated prose sharing a line is the scope creep the
  round avoided everywhere else.

**Task 12: complete.** `977cf4b9e`, spec compliance PASS, task quality PASS. Corpus 154 -> **156**.
Table C `unkno` and `xmeths` both `agree`, 117 -> 115 not-agree, and D50's deletion control recorded as
run with both rows going `diverge-both`. The task spent **none** of the perf budget: `strings`/`ir` at
+52.0001 against the 53.695178 ceiling. Open and carried: `CALL ON NOMETHOD` is a parse refusal at rc
120 against the oracle's 25.1 at rc 231, generic to this crate's parse errors and uncovered by the
corpus; and `.environment~nosuch`'s refusal moved to Task 17's missing piece with one in-crate test as
its only instrument.

---

## Task 13 -- the three access scopes, and the one chokepoint

**Pre-dispatch prerequisite check: every arm re-measured against the current tree, and nothing had
moved.** Both engines, three descriptors: the outside caller is crate rc 120 `a PRIVATE ::METHOD is not
implemented (Phase 5)` against oracle rc 159 `97.2`; the `self~m` twin is the same refusal against
oracle rc 0 `inner`; **the same program with `PRIVATE` removed agrees today on both engines**, which is
the neighbour that must stay green; the sibling and the subclass are both rc 120 here, against oracle
rc 159 and oracle rc 0 respectively; `PROTECTED` and `PACKAGE`'s same-package arm both agree at rc 0.
This is the first task in a while whose starting state was exactly what the plan said.

**The instruments it names already exist, which I checked rather than assumed.**
`tests/dispatch_seam.rs` is D45 site one, with `dispatch_passes_through_exactly_one_chokepoint` at
`:164` and `the_scan_can_tell_a_present_token_from_an_absent_one` as its control -- so the chokepoint
assertion is a test to keep holding, not one to build. The security-manager seam is cited at
`dispatch.rs:36` against `getEffectiveSecurityManager()->checkProtectedMethod`.

**One citation was wrong and I fixed it** (`1a2ec9664`). The plan sent the implementer to
`phase-4-exclusions.txt:3248` for the `PRIVATE`/`GUARD`/`PROTECTED` known gap; that line is the VALUE
dot-variable entry and the gap is at **`:3304`**, in `docs/superpowers/plans/`. The amendment names the
directory and says to search the entry's text instead, because a line number into a document of that
size is the citation most likely to drift. It also carries forward what the entry is for: each limb
states what makes that option *currently safe*, so the limbs come out with the measurements that
supersede them rather than being deleted.

**BASE for the review package:** `1a2ec9664`. Implementer dispatched on opus.

**Task 13 implementer returned DONE_WITH_CONCERNS at `6b1ef369d`**, four commits, and **it corrected
the plan on two points rather than building what the plan said.** Both corrections verified by me, both
amended (`fd04468f0`):

* **The oracle refuses the lookup, not the send.** The plan put the `PRIVATE`/`PACKAGE` checks inside
  the dispatch chokepoint. Measured by me, both engines: `say .K~m` with `m` a `PRIVATE` class method
  **and an `UNKNOWN` class method present** is oracle rc 0 `unknown: M`, not 97.2 -- and it stays rc 0
  when that `UNKNOWN` is itself declared `PRIVATE`, so **the `UNKNOWN` lookup carries no access check
  at all.** A refusal sited at the chokepoint could reach neither answer. The checks live in
  `Interp::resolve`; only `PROTECTED` sits at the seam.
* **Table D's `PRIVATE` row was never an instrument for this task.**
  `gate-tables/directives/method__private__subkeyword.rex` is `say 'main'` plus `::method m private` --
  an *instance* method, never sent -- so it witnesses that the directive installs and nothing about
  access. Table C's `pubpri` is the instrument, and **both** of its controls redden it, allow-all and
  refuse-all alike, so neither blanket answer passes.
* **And my own framing was wrong in the same paragraph.** I had written that the corpus gate "cannot
  see that at all" once our answer stops being rc 120. It can: our refusal becomes the oracle's own
  97.2, so the program is an ordinary differential row. The true and narrower statement is that the
  corpus cannot see a sending position none of its programs occupies.

Gates verified by me: exit 0, 98 `test result: ok`, corpus **159 of 159**. The README's documented
duplicate-key check still prints **0** with Task 13's rows in place.

**Sent to the review as the question I most need answered:** several `pinned>head` medians under
`task` 13 sit above 1% -- `emptyloop`/`tw` 1.0471, `compound`/`tw` 1.0290, `varlookup`/`tw` 1.0243,
`strings`/`ir` 1.0214, `alloc4c`/`ir` 1.0186 -- against a rule that calls a 1% ratio a finding. **What
the guard actually compares is the thing to establish from the repository**, because if the threshold
applied to accumulated drift since the pin then earlier tasks would already have breached it. Paired
with the implementer's first concern: the send path now costs about **+14 per native send and +26 per
class-method send** against Task 10's +3 and +12, unconditional, with the shape that removes it named
(flags on the method, from the dictionary entry `rexx-classes` already returns) and declined as a
cross-crate change it would not take unreviewed.

**One measurement error of my own, again a projection rather than a pattern.** I read "three medians
per key" off the TSV and suspected duplicate rows; the projection had dropped a discriminating column.
The documented check prints 0. A projection that hides a discriminator is the same failure as a grep
pattern that cannot match -- the tool answers a narrower question than the one asked.

**Task 13 review: spec compliance PASS, task quality PASS WITH REQUIRED FIXES.** Every "done when"
clause was re-verified independently, both `pubpri` controls redden that row **and nothing else**
(whole-table diff, 53 to 52 `agree`), the order-invariant claims reproduce including the debug-only
panic, and the `PACKAGE` in-crate test fails under its mutation while the corpus stays green.

**The perf alarm was mine, and it was a projection error again.** Every figure I flagged as over 1%
was **`cycles:u`, not `instructions:u`** -- my `awk` filtered `scope` and never `instrument`. Task 13's
widest `instructions:u` ratio is `strings`/`ir` **1.009685**, under the line. **The plan already warned
about exactly this**: "A `cycles:u` figure from this bench is not a result on its own", with three
prior sittings where the loudest `cycles:u` rows were +2.020%, -5.497% and +2.792% while
`instructions:u` stayed flat to a thousandth. The document was right and my filter was wrong. Second
projection error on this same file in one task, and the general shape is now clear: **a projection that
drops a discriminating column fails exactly like a grep pattern that cannot match** -- the tool answers
a narrower question than the one asked, and answers it confidently.

**One real ambiguity did come out of it, now settled** (`63628e8ec`). The ratio the bench emits under
`pinned>head` is head over pinned: **accumulated drift for the whole phase, not the task's delta**, and
**no code applies the 1% line at all** -- it is prose in the plan. Read as accumulated drift it has
already been crossed twice without stopping anything, which I measured per task: Task 9 at 1.010616,
Task 11 at 1.013710, Tasks 12 and 13 at 1.009685. So the rule governs a task's **own contribution**,
which one interleaved sitting yields, and the accumulated ratio is context rather than a gate.

**The send-path cost was reproduced independently from two binaries in one tree** -- +13.999 per native
send, +25.989 per class-method send, +52.015 for two sends, zero on a sendless loop -- and **declining
the cross-crate change was ruled right.** The residual is that the figure has no committed referent,
which the fix round is asked to close with rows or one sentence.

**What blocks, both handed to fix round 1:**

* **F1: a "nothing covers X" claim whose evidence is false three ways** -- the quoted `/bin/grep`
  lacks `-E`, so run as written it matches nothing; with `-E` it returns three hits rather than one;
  and the line citation points at an unrelated assertion. **The conclusion survives.** Fourth instance
  of that sentence shape on this plan, and **the first where the command rather than the memory was the
  flaw**.
* **F2: `PRIVATE` on `::ATTRIBUTE` changed in both directions with no instrument of any kind.** The
  refusing side now **matches the oracle byte for byte** (rc 159 `97.2`, where we were rc 120), so it
  is expressible as a corpus row and is not one; the allowed side swaps which loud refusal a program
  gets and nothing exercises the private form. The Global Constraint requires a task changing what is
  refused to name the instrument, and to say "an in-crate test only" if that is the honest answer --
  here the honest answer was **nothing**.

**Fix round 1 closed both findings, and the round-1 re-review found two false prose claims -- both of
the two shapes this plan keeps producing.** Corrected by me in the report, each saying what it said
first.

* **"the mutation reddens exactly the two instruments added for it and nothing else in the
  workspace"** is false. It also fails `collect_stress.rs`'s
  `the_l0_subset_passes_again_under_collect_on_every_allocation`, reproduced twice and confirmed
  passing on the clean tree. **A claim about the whole workspace was made from two runs**, which is the
  general error; the coverage claim it was supporting survives.
* **"the second-closest cell in the file to 1% after `strings`"** is false. Task 11's `strings`/`tw`
  1.013710 and `alloc4c`/`tw` 1.011324 and 1.011034, and Task 9's `strings`/`ir` 1.010616, are all
  wider. It is the closest among **this task's own rows**. The forward-looking point -- the next
  instruction added to the send path crosses 1% there -- stands on the axis's own margin and never
  needed the ranking.

**What the round delivered beyond the findings is the thing worth keeping.** `bench-programs/
dispatchclass.rex` gives the send path **its own guard axis** -- the blind spot Task 10 discovered and
could only record. The re-review confirmed the program exercises a class-method send in its hot loop,
that the quoted figures are `instructions:u` and not `cycles:u`, and that the base/changed/pinned
arithmetic is exact. This task's own contribution on that axis is about **0.41%**.

**And the corpus program's load-bearing discriminator was verified by mutation, not argued.** Its
comment claims `CONDITION('E')` reading `2` rather than `1` is what separates a real access check from
a build that dropped the accessor and let the name-miss report stand. The re-review dropped the private
accessor in `install_attribute` and read `E` as **`1`**. The discriminator discriminates.

**Task 13: complete.** `df799fee0`, spec compliance PASS, task quality PASS with both findings closed.
Corpus 156 -> **160 of 160**. Table C's `pubpri` `agree` with both controls recorded as reddening it
and nothing else in that table. `PRIVATE` and `PROTECTED` limbs retired from the known-gap entry at
`phase-4-exclusions.txt:3304`; `GUARD`'s stays for Task 16. Send path costs **+13.999 per native send
and +25.989 per class-method send**, reproduced from two binaries in one tree, accepted with the
removing shape named (flags on the method, from the dictionary entry `rexx-classes` already returns)
and deliberately not taken as an unreviewed cross-crate change.

**Carried forward:** the cross-package `PACKAGE` limb has no differential in this phase and its only
instrument is an in-crate test, which 5c owes; every measurement here is on a class method, because
instance methods need `~new`, which is 5b's; and a bodyless private attribute read from an *allowed*
caller is a separate gap, oracle rc 0 `A` against our loud refusal for want of the instance variable.

## Resume point

* **Next:** Task 14 -- required string values, the other mechanism the spec was rewritten for. Its
  oracle side is already checked and holds exactly; the **crate side needs re-measuring at dispatch**,
  since `makeString` is the row the spec note now assigns to it.
* **Then:** Task 15, and **Moritz has set the stop after 15.**
* **Ordering still binding:** 17 before 21; 21 before 23.

---

## Task 14: required string values

**Bound moved.** Moritz has extended the stop from after Task 15 to **after Task 16**. The resume
point above is superseded on that one point only.

### Pre-dispatch prerequisite check

Every claim the brief makes about the tree holds, and the two measured claims were re-run at
`df799fee0` rather than trusted. Three findings the brief does not carry:

1. **The crate side of the headline divergence, re-measured live.** `say .k` with `::METHOD makeString
   CLASS` returning `"K says hello"` is oracle rc 0 `K says hello` against crate rc 0 `The K class`,
   **on both engines**, matching status and empty stderr on both sides, differing on stdout alone.
   This is the silent shape the task exists for and it is real today, not inferred from the spec.

2. **The Rexx-level face starts LOUD, not wrong.** `.K~request("STRING")` is **rc 120**
   `rexx-exec: method "REQUEST" of class "Object" is not implemented (Phase 5)` on both engines,
   where the oracle answers rc 0 `The NIL object` / `The K class` / `The K class` for
   `~request("STRING")` / `~string` / `~objectName`. So this task starts from *two different*
   states: the protocol is a silent wrong answer, its Rexx-level face is a loud refusal. The brief
   names the first and is silent on the second, and they need different care -- the face's own
   corpus rows can be written against a refusal that already names an owner.

3. **`Interp::to_text` (`value.rs:746`) is total and infallible, and `reqstr` is neither.** It
   returns `Cow<[u8]>` with no `Result`, and its `Body::Native` arm is
   `Cow::Borrowed(native.rendered())` -- that arm is exactly where `The K class` comes from.
   `reqstr` dispatches (`makeString` is a method send that can run Rexx code) and can fail (NOSTRING).
   `required_string` (`builtin.rs:854`) calls `to_text` directly, so **builtin arguments -- one of the
   contexts the brief names -- go through a total function today**. Reconciling those two is this
   task's central design question and the brief does not pose it.

Supporting facts, all checked: `NATIVE_METHODS` (`dispatch.rs:214`-`:288`) has no `STRING`,
`REQUEST`, `OBJECTNAME`, `OBJECTNAME=` or `MAKESTRING` row at `Object`; `MAKESTRING`/`TOSTRING`
exist at `Array` only. The generated class setup does declare `Request`/`ObjectName`/`ObjectName=`,
which is why the names resolve and then meet the not-implemented gate rather than a name miss.
`ClassRegistry::default_name` (`rexx-classes/src/registry.rs:167`) already answers the protocol's
last limb. Table C's `reqstr` row (`gate_table_c.rs:470`-`:481`) exists, is `phase: "5a"`, carries
`oracle_lines: 6`, and names **both** controls as Task 14's in its own `control` field -- the brief
and the table agree. `object_operand_tests` (`eval.rs:3358`) and
`corpus/lang/message_send_argument_object_not_a_string.rex` both exist; that module's own doc says
the object check lives in four places -- `apply_binary`, `apply_prefix`, `arith_general` and
`compare_values` -- which is the surface the re-derivation has to cover, not just `apply_binary` as
the brief's verification line says.

**Ruling: dispatch as written, with findings 2 and 3 carried into the dispatch.** No plan amendment.
Finding 1 confirms the brief. Finding 2 adds a starting state the brief omits but does not contradict
it. Finding 3 is a design question the implementer must answer, and the brief's own wording
(`apply_binary`'s object check, which this protocol makes unavoidable) already points at it. Cost if
wrong: the implementer picks a shape for the fallible/total boundary that a later task has to redo.

### Advance checks run while Task 14's implementer works, and one plan amendment

These are the halves of Tasks 15's and 16's premises that need no crate binary, run now because the
tree is being changed by an implementer and the oracle is not. **The crate half of each is
deliberately not run here**: the only binary available is either mid-build or the pin at
`15a1ffa98`, which predates Tasks 1 to 13 and cannot answer "what does the crate do today". Both
tasks' crate-side claims get re-measured at their own dispatch, the way Task 14's just were.

**Amendment.** Task 16's text cited `phase-4-exclusions.txt:3248` for the `GUARD` limb it retires.
Wrong line, and **the same defect I already corrected once on this plan** -- Task 13's citation had
it too, and the fix there did not sweep for other instances. The `GUARD` known-gap entry is at
`:3304`; `:3248` is inside a `VALUE`/dot-variable entry. Corrected in place. I then swept every
`phase-4-exclusions.txt:<n>` citation in the plan: `:3304` (Task 13, already right), `:517` (a
`::CONSTANT` known gap, right), `:420` (the `::annotate routine nosuchrtn` row, right). One wrong of
four, now zero.

**Task 16, oracle side, both programs confirmed exactly as the brief states them.** `guard on` in a
class method body is rc 0, stdout `guarded`, empty stderr. And `reply "replied"` / `say "after"` /
`return "returned"` is **exit status 0** with stdout `replied` then `after` and a `98.936 RETURN
cannot return a value after a REPLY.` traceback on stderr -- the exit-status-0-with-a-traceback pair
the brief warns a gate case must reproduce. **One thing the brief does not say: that traceback
carries the program's absolute path**, so whatever instrument pins this transcript has to handle the
path the way the corpus harness does, and a hand-written in-crate byte comparison would not.

**Task 16's `CoreClasses.orx` citations hold.** `:1554` is `guard off` and `:1663` is `reply`, in the
two bootstrap bodies the brief names.

**Task 15, oracle side, all three live rows confirmed exactly.** `::METHOD a CLASS ATTRIBUTE` with
`.K~a = 5` then `say .K~a` is rc 0 `5`. `::ATTRIBUTE b CLASS` with `7` is rc 0 `7`. And `::METHOD m
CLASS ABSTRACT` prints `installed` first and then refuses the send at rc 163 with `93.965 Method M is
ABSTRACT and cannot be directly invoked.` -- so the brief's "installs at rc 0 and is refused only when
sent" is confirmed by the `installed` line surviving in stdout, not inferred from the status.

**Task 15's six agreeing options, oracle side.** `PUBLIC`, `PACKAGE`, `GUARDED`, `UNGUARDED`,
`PROTECTED` and `UNPROTECTED` on a class method are each rc 0 with the body's own output and empty
stderr. That is one side of the brief's "identical stdout on both sides today" and **not the claim** --
the claim is about the pair and the crate half is what can have moved under Tasks 9 to 13. Task 15
re-measures it.

**The amendment is in the working tree and deliberately NOT committed yet.** Committing it now would
move HEAD under a live implementer and put a controller's doc commit inside the `df799fee0..HEAD`
range Task 14's review package is generated over, which is the range the reviewer reads as the task's
diff. It is committed on its own after Task 14 closes. Recorded here so it cannot be lost if this
context is compacted: **`docs/superpowers/plans/2026-08-17-phase-5a.md` line 1506, `:3248` to
`:3304`.**

### REXXCPS: the fixed-count copy already exists, and what measuring it showed

**Moritz recommended copying `rexxcps` to a fixed-count benchmark. That work is already in the tree**
and has been since 2026-08-20: `rust/bench-rexxcps/rexxcps.rex`, REXXCPS 2.2 with `count=200` and
`averaging=100` as constants and the `do trial=1 to 2` loop deleted. Verified by reading the
implementation rather than its README: the stock `samples/rexxcps.rex` has `count=100`,
`averaging=100`, the trial loop, and the self-adjustment `count=(1%total + 1) * count` at its line
163; the canonicalised copy has none of that. So the shape is **200 x 100 x 1000**, between the two
Moritz suggested. Both sides print `Averaged: 200 x 100 iterations of 1000 clauses`, which is the
README's own stated check that the two did identical work, and it holds.

**Keeping 200 x 100 rather than raising it to 200 x 200.** REXXCPS 2.2's adjustment exists to get a
run past one second of wall clock, and both sides already clear that: 1.17 s oracle and 1.98 s crate
in the committed baseline. Doubling `averaging` doubles the runtime for no resolution the instrument
can use, and it would make the new rows incomparable with the committed ones.

**Measured now, interleaved, three rounds, 8 GiB cap on every side.** The crate arms are the pinned
`15a1ffa98` binary (sha256 verified against `PINNED.md`) and the working tree's `rexx-run` as the
Task 14 implementer had just rebuilt it at 21:04:55 -- **not HEAD**, and labelled as such. That
binary's sha256 was captured before and after the run and did not change, so all its rows are one
build.

| side | median cps | spread |
|---|---:|---:|
| oracle | 16,983,508 | 3.10% |
| pinned `15a1ffa98`, ir | 10,217,307 | 0.29% |
| Task 14 WIP, ir | 10,408,703 | 1.52% |
| pinned `15a1ffa98`, tw | 8,225,714 | 1.11% |
| Task 14 WIP, tw | 8,334,177 | 2.31% |

oracle/pinned-ir **1.662x**; ir is **1.242x** the tree-walker. The pinned figure reproduces the
committed baseline's 10,203,946 median to within 0.13% and the oracle's to within 0.06% on a single
earlier pass (17,351,863 against a recorded 17,350,930), so the program and the harness are behaving.

**The finding that changes the plan: cps cannot resolve what the arms guard resolves.** The oracle's
own figure moved 3.10% across three interleaved rounds, and its single earlier pass read 17,351,863
against a 16,983,508 median here -- about 5% between two sittings minutes apart. The whole
accumulated Phase 5a drift on the arms axes is at most **+0.99%** on `instructions:u`. So the
`+1.87%` ir and `+1.32%` tw that the WIP build reads *faster* than the pinned one here are **inside
this instrument's noise and are not a result.** They are recorded as not-a-result on purpose.

**So the axis is added on `instructions:u`, not on cps.** That is the reason `rexx-arms` reports
instructions in the first place, and a `Fixed` workload gives exactly the absolute counts and the
`pinned>head` ratio with no per-pass figure to misread. The cps number stays what it already is: a
crate-against-oracle headline for the end-of-phase `rexx-bench-suite` snapshot, quoted with its
interval, never differenced against another section.

### Plan amendment: the guard's axis list, and what was missing from it

**Moritz's call: rexxcps joins the guard for Tasks 15 and 16.** Both new axes are now written into the
Global Constraints command block -- `--axis dispatchclass --axis bench-rexxcps/rexxcps.rex` -- with a
paragraph saying which task each starts at, that neither is retroactive, and that adding an axis is
not the event that restarts the baseline (a change of pin is, and the pin is unchanged).

**The larger finding, which is why the amendment is worth more than one flag.** `dispatchclass` had
been a guard axis for two tasks **without appearing anywhere in the plan**. It was carried in Task 13's
dispatch prose and nowhere else, so a later task that did not inherit that prose would have run a
narrower guard and reported green over an axis it never measured, with nothing outside the dispatch
able to tell. That is this session's own recorded hazard about dispatch prose not being a control,
found in the tree rather than in the abstract.

**The command line was run before the paragraph describing it was written.** Pinned binary given as
both builds, into a scratch baseline: exit 0, 16 rows, `axis` column reading `rexxcps` rather than the
path, `size` reading `small` alone, scopes `absolute`/`arm_ratio`/`across_builds`, and **zero
`per_pass` rows**, which is what the `Fixed` classification promises. The harness refuses `--rounds 1`
with `1 rounds is not a median with a spread; 3 is the least this harness reports`.

**And that run measured the axis's noise floor: the tightest of any axis here.** Same binary both
sides, `pinned>head` on `instructions:u` came out **1.000016** (ir) and **0.999984** (tw) -- about two
thousandths of a percent, against the 3.10% cps spread on the same program. Absolute counts are
**18,995,016,694** instructions on ir and **23,968,311,451** on tw, several times the largest existing
axis. A longer deterministic run sharpens the ratio where a longer timed run only blurs it, so this is
the axis best able to see a small regression, not the worst.

**Still uncommitted, with the earlier citation fix.** Two edits now wait on Task 14 closing:
`:3248` to `:3304` at what was line 1506, and this guard-block amendment. Both are in
`docs/superpowers/plans/2026-08-17-phase-5a.md` and go in one docs commit once the implementer's
range is closed.

### Upstream filings, 2026-08-22, and a third pending edit

Moritz filed four tickets and commented on one, all confirmed by reading them back from the Allura
API rather than from the drafts: **SF #2082** (`DATE` day-zero reaching `monthNames[-1]`), **#2083**
(`StringUtil::pos` scanning one past its window, carrying both the wrong answer and the `CHANGESTR`
crash), **#2084** (the same `CALL ON` condition name queued twice in one clause), **#2085**
(`ArrayClass::validateIndex` taking `items()` alongside `data()`). All open, all created 2026-08-22 by
`antiguru`, titles verbatim as drafted. The trace-indent finding went in as **post 8 on SF #1850**
"TRACE issues" (status accepted) rather than as a new ticket, because that ticket's own second section
already covered the same code path's other symptom.

**Worth knowing before any follow-up on #1850:** its post 6 is Erich's **r12719**, 2023-08-16,
"DO/LOOP/SELECT no longer double-indent trace output". A trace-indent defect there has already been
fixed once, so the new comment can read as covered. What separates them is that r12719 fixed a
**double**-indent and this is an **under**-indent of two columns, measured on 5.3.0 built 30 Jul 2026,
a build that contains r12719.

**`rust/corpus/oracle-crashes.txt` is updated and is now the third edit waiting on Task 14 to close.**
Its entries 2, 3, 5 and 6 each said "not filed upstream"; all four now carry their SF number, and the
file has no `not filed` line left. Checked by grep after the edit, and the five SF numbers it now
holds are #2018, #2082, #2083, #2084, #2085.

**Pending docs commit, once the implementer's range is closed:** the `:3248` to `:3304` citation fix,
the guard-block axis amendment, and this crash-list update. Three edits, one commit.

**Asked and answered: there is no second, unfiled `SELECT` segfault.** The exclusion list carries no
`SELECT` crash entry at all -- zero lines matching a crash word (`segv|segfault|sigsegv|crash`)
together with a select word (`select|when|otherwise|dangl`), and zero occurrences of `dangling` in the
whole file; its only `SELECT` row is line 3016, `SELECT CASE`, which **agrees**. `oracle-crashes.txt`
has exactly one `SELECT` entry, entry 1, already attributed to #2018. Repo-wide, `dangling` appears
only as "dangling THEN" in Phase 3 parser records, which is a parse shape this crate handles, and as
"dangling reference" in review prose. **The likely referent is #2018's own open question** -- jfaucher
asked whether the parser should reject `when <cond> then` immediately followed by `when`, the way
Josep Maria's parser does with Error 9.1. That is a language-design call left with the maintainers, not
an unfiled defect.

### Task 14: implementer reported DONE_WITH_CONCERNS, review dispatched

Five commits, `df799fee0..127fb74b8`:

* `b305019ec` the protocol plus every dispatch site, 940 insertions across 20 files
* `93099cf99` `~string`, `~request`, `~objectName`, `~objectName=` and `String~makeString`
* `b15552ec3` nine corpus programs, the in-crate refusal tests, the `object_operand_tests` re-derivation
* `6f7899514` two of its own new corpus programs that would **hang** under the control they exist to catch
* `127fb74b8` the sitting

**Gates verified by me, not taken from the report.** At `127fb74b8`: fmt 0, clippy 0,
`REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` exit 0, 98 `test result: ok`,
corpus **169 of 169** (160 before), zero `test result: FAILED` and zero `panicked`.

**The design decision on my finding 3.** `to_text` stays total and infallible; the protocol is a
separate fallible pre-step returning an `ObjRef`, gated by a monotonic latch whose soundness a
debug-only byte comparison checks. Rejected: a fallible `to_text`, a `Cow`-returning dispatcher, and
hoisting the conversion. **The latch is the part I do not yet believe**, and it is the review's first
attention item: a cache that misses is safe and a cache that lies is a wrong answer, the check is
debug-only so release has nothing checking it, and a check that has only ever passed is
indistinguishable from one that cannot fire. The review is asked whether it was ever inverted.

**`6f7899514` is the commit worth keeping from this task.** Two of the new corpus programs armed
`SIGNAL ON SYNTAX` above the clauses they measure and set the loop counter below them, so under the
deletion control the handler read an uninitialised `n`, raised 41.1 from inside itself, re-armed, and
wedged. **A witness that hangs under the mutation it exists to catch is not a witness** -- and it was
found by running the control rather than by reading the program. That is the "can fail" versus "adds
coverage" hazard caught in the act.

Its `dispatch.rs` hunk is a **pure doc-comment edit** with no behaviour, which I checked before
dispatching the review, and its commit message does not mention it. Minor, flagged to the reviewer
rather than ruled on here.

**Concerns carried into the review.** A builtin fetching arguments out of position order would
convert in a different order from the oracle, described as a **residual, unenumerated** -- but the
builtin table in this tree is finite and enumerable, so the review is asked to settle whether any
builtin does this today, which turns a concern into either a correctness finding or a non-issue.
Three `reqstr` contexts stay refused as Phase 7 or 5c. `~request` for unmodelled `MAKE` methods and
`~objectName=` on a string stay loud by decision.

**Sitting, reported and to be checked:** widest own contribution `compound`/ir **1.007835**, under the
line. `emptyloop` used as an in-sitting layout control at `pinned>base` exactly 1.000000 and
`pinned>head` 0.992022, on the argument that a change adding work cannot make an axis go down. The
review is asked whether that is a control or a rationalisation, and to confirm the rexxcps axis is
**absent** here -- it joins at Task 15, so its absence is correct and its presence would be the
anomaly.

### Task 14 fix round 2: landed, verified by me, and the agent died before reporting it

The implementer's process was killed at about 00:49 on 2026-08-23, one minute after it committed
`04cc77f3d`. **No completion record, so nothing about this round came from the agent** -- every
statement below is something I checked in the tree.

`04cc77f3d` "Witness the trap-arming site, and re-derive the raw-position table". Gates at that
commit: fmt 0, clippy 0, `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` exit 0,
98 `test result: ok`, corpus **171 of 171**, zero `FAILED` and zero `panicked`.

**All eight findings from round 2's dispatch are addressed in the tree.** A: the new program is
registered at `phase-5a.txt:424`. B: `converting the whole list` is gone. C: `coverage.rs` reads
cleanly, the sweep corruption is repaired. E: the accumulated table is labelled `highest ratio per
axis over both arms and both sizes`, which is what it projects. F: the report head reads 171, and each
round's own evidence table correctly records what that round measured rather than being stale. G: gone.
H: `builtin.rs:1551` asserts the table is non-empty, `:1569` refuses an empty per-builtin list.

**Finding A got the right answer, which is worth recording because the wrong answer was available.**
`required_string_nostring_no_directive.rex` declares nothing at all, so `SIGNAL ON` is the only thing
that can arm the protocol -- every earlier trap-armed program also declared a class with a
`makeString`, and a directive installs before the first clause runs, so the latch was already set by
the time the `SIGNAL ON` executed. Proved live by disabling the arming site: it diverges on stdout on
both engines **while every other trap-armed program in the corpus still matches.** That contrast is
the witness; a program that merely goes red is not.

**And the round took the re-review's correction rather than restating its concern.** The blocker it
had cited for not re-deriving the raw-position table did not exist -- `tests/gate_tables/orx.rs:79`
already reads `interpreter/` at test time -- so the table is now re-derived on every test run, panics
rather than skipping when the tree is absent, states its own blind spot as a set asserted against the
source, and was proved live three ways.

### Two rulings

* **Ruling: finding D is closed.** Its one surviving instance, "called from both directive
  installers", is at report `:767` -- in the **report**, not a comment. The set-size constraint binds
  comments; no code comment carries the shape. Cost if wrong: a report sentence names a set size,
  which no rule forbids.
* **Ruling: the `emptyloop` layout swing is accepted and the sitting stands.** `emptyloop` read
  `0.989284`, 1.1% in the **fast** direction on an axis this change cannot touch, which makes the
  widest own contribution (`compound`/ir at 1.007835) uninterpretable rather than clean -- and the
  report says exactly that. Accepted: the contribution is under the 1% line, and an instrument whose
  layout noise exceeds a sub-1% signal is a property of the instrument, not a defect in the change.
  Cost if wrong: a sub-1% regression on this task is invisible, and the next task on the same axes
  inherits it.

### The session that died, and the plan amendment it produced

**Moritz asked whether something ran without a memory cap. It did, and it was the subagent's own
comparison script.** `cmp.sh` bounded the oracle at line 8 with `ulimit -v 1048576` and
`timeout -s KILL 10`, and bounded the crate at line 15 with `timeout -s KILL 20` and **no memory limit
at all**. Two crate runs hit that 20-second SIGKILL, so the crate looped for the full 20 seconds
unbounded on a machine with 124 GB of RAM and 94 GB of swap. `memcap` appears 15 times in that agent's
transcript, so it was capping where it thought to; this site it did not. **No kernel OOM record is
recoverable** -- `dmesg` shows nothing and the cgroup files are unreadable from here -- so the
mechanism is established and the kernel event is not, and it is written down that way.

**A second defect rides on the first: `rc 137` is `128 + SIGKILL`, which a `timeout -s KILL`, a
`memcap` and a kernel OOM all produce.** The script read all three as one event and printed `DIVERGE`
for what was a timeout.

**Amended into the plan's global constraints, because dispatch prose is not a control and this gap was
in mine.** Every wrapper in that document caps the oracle and nothing said to cap `rexx-run`, so a
differential probe ran one side under a 1 GiB address-space limit and the other under none -- which is
not a differential measurement. Crate-side runs now carry both bounds, and a task reporting rc 137
says which of the three produced it.

**Fourth pending edit, same deferred commit:** the crate-side bound in global constraints, the
within-sitting qualification on the rexxcps noise floor with the 33.97 instructions-per-pass
`dispatchclass` span, the `:3248` to `:3304` citation fix, the guard-block axis amendment, and the
`oracle-crashes.txt` SF numbers.

### Task 14: COMPLETE

**Task 14: complete.** `f0b7c1a` range head after three fix rounds and two re-reviews. Spec compliance
PASS from the first review and never reopened. Corpus **160 -> 171 of 171**. Gate table C's `reqstr`
row `agree` with both of D50's controls run and recorded: the deletion control, and mutation 3
reddening at rc 0 with empty stderr on both sides on stdout alone.

Commits: `b305019ec` `93099cf99` `b15552ec3` `6f7899514` `127fb74b8` (original), `13219ab92`
`1d87d90cc` (round 1), `04cc77f3d` (round 2), `e8717a8b7` (round 3), and mine: `11bfec85c` (F1/F2),
`123ae0bc8` (the five doc corrections), `bc35d38` (F3).

**The one behaviour defect this task introduced, and how it was caught.** `VALUE`'s second argument
was converted where the oracle fetches it raw, giving rc 0 and empty stderr on both sides with stdout
alone differing -- the exact silent shape the task exists to remove. **Nothing in the harness could
see it.** It was found because the reviewer refused to accept "residual, unenumerated" for a set the
code can enumerate, enumerated the 81 `BUILTIN(x)` blocks, and hit it. The lesson is not about
`VALUE`: an unenumerated residual over an enumerable set is where the defect was hiding.

**Three instances of one pattern on this task, which is worth more than any of them.** The latch's
debug check claimed a coverage it did not have. Two corpus programs hung under the control they
existed to catch. A corpus program's header claimed a route it could not see. **All three asserted
coverage from a green run rather than from an inversion**, and each was settled by breaking the thing
and watching the witness move. That is now the standing method on this plan.

### Rulings at close

* **Ruling: F1 and F2 fixed by me rather than by a round-4 escalation.** The loop's rule sends rounds
  four and five to a fresh implementer on a higher tier; that rule exists for an implementer who is
  stuck, and this was one false doc sentence and six escaped backslashes with exact locations. Cost if
  wrong: a controller edit lands without an implementer's own self-review, which is why the F1
  sentence was re-derived by running the inversion rather than taken from the review.
* **F1 verified by running it, not by reading the finding.** Exemption disabled, `--no-fail-fast`
  corpus run: exactly `required_string_builtin_raw_argument.rex` and
  `required_string_nostring_no_directive.rex` redden. The doc now names that set by what its members
  do and says which check established it.
* **F2 was wider than reported.** The review named three sites; each string literal had a *second*
  continuation line with the same escaped backslash, so six lines needed the fix, not three. Found by
  grepping for the pattern after applying the named three.
* **Ruling: F3 fixed rather than parked.** A comment asserting "every directive installer" when two
  installs do not arm is a false statement in code, and `arm_reqstr_for`'s own doc already carried the
  right qualifier, so the tree disagreed with itself about one set.
* **Ruling: F5 and F6 parked.** F5's "widest movement in the whole sitting" is a `per_pass` figure
  where the `fixed` scope moved further, but the reviewer established that scope is known noise across
  all three sittings and nothing rests on the phrase. F6 is three comments narrating their own edit
  history against the strike test, in a shape the reviewer found pervasive and pre-existing -- that is
  a project-wide question and not this task's to settle. Cost if wrong: a report sentence overstates
  its own scope, and the strike-test sweep stays owed.
* **F4 corrected the plan, and the plan was wrong because I wrote it.** My own amendment asserted rc
  137 has three producers including an address-space breach. Measured: `ulimit -v` gives **rc 134**
  here (a Rust allocation abort) and **rc 251** on the oracle (a graceful `Error 5 System resources
  exhausted`), while `memcap` and a time kill both give **137**. So rc 137 has two producers, and the
  address-space bound is **not symmetric between the two interpreters** -- it makes them fail
  differently, which a three-descriptor harness reads as a divergence the interpreters did not cause.
  The bullet now prefers `memcap` for that reason.

**Next:** Task 15, the `::METHOD`/`::ATTRIBUTE` option surface. Oracle side already verified exactly
(`5`, `7`, and rc 163 `93.965` with `installed` surviving in stdout). **The crate side of its six
agreeing rows needs re-measuring at dispatch** -- that claim is about the pair, and the crate half is
what Tasks 9 to 14 could have moved. Its sitting now includes `dispatchclass` and
`bench-rexxcps/rexxcps.rex`. **Then Task 16, and Moritz has set the stop after 16.**

## Task 15: `::METHOD` and `::ATTRIBUTE`'s option surface

### Pre-dispatch prerequisite check

Every measured claim in the brief was re-run at `979522f74` rather than trusted, with crate runs bounded
by `memcap 1G` per this morning's amendment. **All hold.**

**The three live rows.** `::METHOD a CLASS ATTRIBUTE` with `.K~a = 5` then `say .K~a` is oracle rc 0
`5` against crate **rc 159** with an `Error 97` traceback. `::ATTRIBUTE b CLASS` with `7` is oracle
rc 0 `7` against crate **rc 120**, `a generated ::ATTRIBUTE accessor is not implemented (Phase 5)`.
`::METHOD m CLASS ABSTRACT` is oracle **rc 163** with `93.965 Method M is ABSTRACT and cannot be
directly invoked.` against crate **rc 120**, `a ::METHOD with no body of its own is not implemented
(Phase 5)`. **On the third, `installed` reaches stdout on both sides**, so the directive already
installs correctly here and only the send diverges. Both crate refusals name an owner, so this task
replaces loud refusals rather than fixing wrong answers.

**The six agreeing rows, re-measured because the claim is about the pair.** `PUBLIC`, `PACKAGE`,
`GUARDED`, `UNGUARDED`, `PROTECTED` and `UNPROTECTED` on a class method are each rc 0 with the body's
own output and empty stderr, **MATCH on both engines**. Tasks 9 to 14 could have moved our half and
did not.

**Four findings the brief does not carry, all in the dispatch:**

1. **Table D's probes already exist, so this task changes verdicts and not the row set.** There are
   committed probes for both directives under `corpus/gate-tables/directives/`, named
   `{directive}__{keyword}__{position}.rex`, covering all six agreeing options and both `get` and
   `set`. The brief's "one probe per table D row" is already satisfied.
2. **`owning_phase` (`gate_table_d.rs:231`) already assigns every row this task touches**, via a
   catch-all putting `::ANNOTATE`/`::ATTRIBUTE`/`::CLASS`/`::METHOD` options in 5a, with `DELEGATE`
   carved out to 5b at `:237` and its reason at `:202`. Nothing to re-litigate.
3. **Table C cannot see the abstract-method enforcement, and its own row says so.** The `abscla` row
   puts abstract-*class* enforcement in 5b because the check lives inside `~new`, and says the
   abstract-*method* half is 5a and **"has no arm of this section's probe"**. So the instruments are
   the table D rows plus in-crate tests, exactly as the brief says, and "an in-crate test only" may be
   the honest answer for the abstract-send refusal.
4. **The `GET`/`SET` overriding-body case already has corpus coverage** in
   `lang/method_attribute_body.rex` and `lang/method_attribute_set_body.rex`.

**Ruling: dispatch as written.** No plan amendment. Findings 1 and 4 narrow the work rather than
changing it; 2 and 3 confirm the brief's own scope statements against the tree. Cost if wrong: the
implementer adds probes for rows that already have one, which review would catch as duplication.

**Carried into the dispatch as the pattern to avoid:** Task 14's three instances of asserting coverage
from a green run. The brief's control -- a getter generated without a setter -- must be proved live by
running it, with before and after quoted.

### Task 15: implementer reported DONE_WITH_CONCERNS, review dispatched

Six commits, `979522f74..dd01df2b4`. Gates verified by me at that commit: fmt 0, clippy 0,
`REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` exit 0, 98 `test result: ok`,
corpus **177 of 177** (171 before), zero `FAILED` and zero `panicked`.

**The guard axis I made mandatory two tasks ago earned its place on its second outing.** The first
sitting **breached the 1% line at +1.86% on `dispatchclass`** -- the send-path axis Task 13 created
for exactly this and which had one sitting of history. The implementer measured four attributions,
found none of them was the knob, and then **changed the design** rather than writing a note: it split
`generated_methods` out of `method_bodies`, and the final sitting reads **+0.047%** with the other
seven axes at 1.000000. That is the outcome the axis existed to produce, and it is the first time on
this plan a guard breach has forced a redesign instead of an attribution paragraph.

**Four concerns, the first three routed to the review as priorities.**

1. **The abstract raise has no mutation witness of its own.** This is Task 14's pattern for the fourth
   time on this plan: a behaviour whose coverage no inversion has tested. The review is asked to
   delete the raise and run the workspace under `--no-fail-fast` to find out whether anything goes red.
   **Table C cannot be that instrument** -- its own `abscla` row says the abstract-method half has no
   arm of that section's probe -- so if nothing else catches it, the honest answer is an in-crate test
   named as the only instrument.
2. **The two-table separation is load-bearing with only a comment defending it.** On this plan a
   constraint defended by prose has failed repeatedly and only type-level or test-level enforcement has
   held. The review is asked whether an enforcement is feasible here, and told not to demand a
   redesign if the answer is genuinely no.
3. **"30 of 33 probes byte-identical"** -- the summary does not say what the other three are. Either
   legitimate non-identity, or a divergence described as something else.
4. Stem and compound attribute variables remain loud refusals, so the generated-accessor refusal
   **narrowed rather than disappeared**; and every measurement is class-side, because instance
   receivers need `~new`, which is 5b's. Both are consistent with earlier tasks and are for the review
   to confirm rather than for me to accept.

Also carried into the review: seven axes reading **exactly** 1.000000 is a strong claim to check
rather than accept, no conclusion may be drawn across two sittings, and every projection keeps its
`instrument` column.

**Three stale background waiters from this task's own sittings have now notified**, each replaying the
same result. Recorded so a later reader does not read them as three separate finishes.

### Task 15: COMPLETE

**Task 15: complete** at `b1016203c`. Spec compliance PASS from the first review and never reopened.
Task quality closed after three fix rounds and one scoped re-review. Corpus **171 -> 177 of 177**,
gates green, tree clean.

Commits: `9a422b998` `0fc365d3d` `d7bf45383` `aed89a3e7` `64bfa30d6` `dd01df2b4` (original),
`e9f1e830b` `df48d53f0` (round 1), `eb85dfdd6` `e243c8755` `6cd464849` (round 2), `b1016203c` (round 3).

**The guard axis paid for itself.** The first sitting breached at **+1.86% on `dispatchclass`**, the
implementer measured four attributions, none was the knob, and it **changed the design** -- splitting
`generated_methods` out of `method_bodies` -- rather than writing an attribution paragraph. Final
contribution +0.047%, and the re-review reproduced the 600-row sitting digit for digit. First guard
breach on this plan to force a redesign.

**Two of the implementer's four concerns were false, and the reviewer killed them by mutation.**
"No test asserts the two-table separation" -- folding the tables back with the order swapped fails a
named test on its own while the corpus stays green. "The abstract raise has no mutation witness" --
replacing it with `Ok(None)` fails an in-crate test and takes the corpus to 176 of 177. **Both were
written from a green run**, the fourth task running to produce that shape. The separation's assertable
half then landed as the type-level form the plan has learned to prefer:
`const _: () = assert!(size_of::<InstalledMethodBody>() == 16)` at `lib.rs:3217`, proved live by E0080.

**Ruling taken back and decided: the loud refusal.** `::METHOD a CLASS DELEGATE p ATTRIBUTE` was oracle
rc 159 `Object "P"` against crate rc 159 `Object "The K class"` -- **status, error number and sub-number
all matching, only the substituted receiver wrong.** My round-1 dispatch said "do not implement
`DELEGATE`" meaning the semantics; the implementer read it as excluding the reviewer's cheaper
alternative too, which is a fair reading of what I wrote, and handed the decision back. Ruled: install
the setter key so the send refuses loudly. Reasons on the record -- the plan's standing trade is that a
refusal is visible and a wrong answer is not; giving the send something to refuse is not modelling
delegation, which stays 5b's; and it converts what 5b inherits from a silent defect into a loud one.
**Cost stated as a cost:** rc 120 now differs from the oracle's rc 159 where the name miss agreed with
it, and that agreement was the defect. Instrument is an in-crate test only, verified in the strong
direction -- under the revert mutation only the setter row fails and the workspace stays 177 of 177.

**The accumulated drift disclosure, which I found rather than the review.** The report's "every axis
within 9 ppm" was the own contribution and correct; the **accumulated** `pinned>head` figures were
undisclosed and two are over 1%: `dispatchclass` 1.012257 to 1.012597 and `rexxcps` 1.018638 ir /
1.015430 tw. Attribution derived and then confirmed by the re-review: **`dispatchclass` crossed during
Task 14** (1.009288 -> 1.011803, Task 14's own contribution 1.002149 to 1.002492), so Task 14 owns it;
**`rexxcps` has Task 15 rows only** because the axis joined the guard at Task 15, so its reading is an
unattributable position -- which the amendment I wrote for that axis had already licensed in advance.
A control firing correctly rather than being found stale.

**Third instance of the sweep hazard.** Round 1's own set-size sweep commit *introduced*
"beside the two fields", and re-running the sweep in round 3 found the shape in more places than round
1 had classified. Task 14's counterpart did the same across eight sites. The durable change is that
round 3's report states the sweep as a command plus a per-line classification of what it still returns,
rather than as a claim about what was swept.

**Also worth keeping:** `cargo doc` caught a broken intra-doc link that fmt, clippy and the test suite
all pass over, and it caught a second one this task had introduced. And N6's fix measured the `BOTH`
body refusal as `Error 99.937`, which caught the implementer's own first-draft guess of 99.934 before
it shipped.

**Parked with rulings:** N5, a `cycles:u` interval quoted from `small` rows where the range across
sizes is wider. The lookup order remains the one half of the perf fix nothing can assert. No instance
receiver anywhere, because `~new` is 5b's. Stem and compound accessor variables still refuse loudly, so
the generated-accessor refusal narrowed rather than disappeared. `DELEGATE`+`ATTRIBUTE` still diverges,
now loudly, with an in-crate test as its only instrument and the oracle's own answer recorded in the
comment for 5b.

**Next: Task 16, `REPLY` and `GUARD` inside a method -- the last task before Moritz's stop.**

## Task 16, fix round 1 re-review: CHANGES REQUESTED

Scope `7eabdfb59..8610d6334`, report at `task-16-rereview.md`. All seven of round 1's findings
**closed on their substance**. Ten new defects, every one a sentence in a comment or in the report:
no behaviour defect, no dead instrument, no number that gates anything.

**The top finding was the controller's own commit `8610d6334`.** I had flagged it in the dispatch as
an unchecked inference about the oracle's reason; it was worse than that -- false in both directions,
and two probes kill it without needing the C++. Re-run by me, three runs each: `guard on when w = 0`,
already true and satisfied by nobody, is `Error 99.913` at rc 157 *even inside a method that exposes
something else*; `guard on when v = v`, which no activity's change could falsify, answers rc 0. The
check is a reference test on which names the expression mentions
(`LanguageParser.cpp:2024`, `InstructionParser.cpp:2640`-`:2646`). The maintainers' comment at the
raise site gives their motivation, which the entry now quotes as motivation rather than as the rule.
The same pass caught a second error of mine: the ON/OFF sentence had the flag backwards --
`guardWait` releases the scope either way (`GuardInstruction.cpp:147`-`:150`). Fixed at `08965a822`.

**Ruling (defect B):** drop the enforcement clause rather than convert the reader names to intra-doc
links. `cargo doc` is not a gate, so a warning there lands among nineteen others and fails nothing --
a check blind to its own subject. Cost if wrong: the doc can still go stale silently, which is what
it already does; we lose nothing we had.

**Ruling (defect D):** add `corpus/lang/method_guard_when_not_logical.rex` rather than narrow the
sentence to name 34.902 as a third sole-instrument row. Building the instrument makes the claim true
and buys a real differential row; corpus 185 -> 186. Cost if wrong: one corpus program to delete.

**Ruling (defect F):** delete the marker count, keep the site list, do not ship a corrected count.
`grep -c` cannot see this claim's subject at all -- one marker wrapping to two lines pays for one
marker missing, so the number could match with a site unmarked.

**The set-size sweep did not relocate the shape.** First round on this plan where that holds, and the
reviewer verified it rather than taking it: four genuine set-size phrases removed, none reintroduced.
The three surviving enumerations with no instrument are parked as such.

Fix round 2 dispatched fresh (round 1's implementer did not survive compaction), brief at
`task-16-fixround-2-brief.md`, covering B through J.

## Task 16, fix round 2: DONE, one item back for round 3

Commits `51103413a` (the corpus witness) and `5bc6aca0a` (B, C, E, J). Gates re-run by me at
`5bc6aca0a` rather than taken: fmt 0, clippy 0, `REXX_CORPUS_GATE=1 cargo test --release
--workspace --no-fail-fast` exit 0, **186 of 186 matching**, 98 `test result: ok`, zero FAILED,
zero panicked. I also measured D's shape independently before dispatch and again from the
implementer's own program: oracle rc 222, `Error 34.902`, byte-identical on both engines and all
three descriptors.

**The implementer's own concern falsified a sentence it shipped, which is the useful outcome of
the round.** C's fix made `Interp::deferred`'s doc say the two-order/five-order gap "depends on how
the shape is written, not on which sitting ran it". It then reconstructed the shape the 18/12
reading came from -- the original scratch file is gone, in git and everywhere else -- and measured
**five** orders. Same described shape, two on one sitting and five on another, with no way to tell
whether the reconstruction is faithful or the original was undersampled.

**Ruling:** drop the causal clause. Rest the claim on the shape that reproduces (five orders, twice,
three rows identical), which is all the exclusion needs, and state in the doc that the 18/12 reading
came from a program that was not preserved. That is the finding rather than a hedge: defect C existed
because a number was recorded without its program, and this round measured what that costs -- the
number is now unverifiable and its best reconstruction disagrees with it. Cost if wrong: we under-claim
a mechanism we never established.

**A live instrument earned its place.** The implementer tried the new corpus program in
`collect_stress.rs`'s `NO_ALLOCATION_PROGRAMS`; the both-directions assertion caught that it does
allocate, so it stayed out. That list cannot be padded by assumption.

Round 2 also left a false count in its own commit subject ("six sentences", addresses four). Not
amended -- correct call; round 3's commit message carries the correction so it lives in git rather
than in an ignored report.

## Task 16, fix round 3 and the corpus-total fix

`49e4c9e27` replaced the last comment quoting the live full-corpus total with the names of what the
98.936/98.937 mutation reddens. The implementer re-ran that mutation on the 186-entry tree rather
than taking my numbers and got mine exactly, and **found a second false claim in the same sentence
that I had not flagged**: it said the corpus mismatch was "the only red thing in the workspace",
which the same mutation refutes. I widened its negative check across the whole rust tree afterwards:
the other `N of M` readings are subset counts or commit-pinned history, neither of which goes stale
when the corpus grows, so the defect is closed -- but the implementer's stated pattern was wider
than what it checked, and the true claim is that no comment quotes the *full* corpus total live.

`52682fe94` carried out the round-3 ruling. `Interp::deferred`'s doc now rests on the shape that
reproduces and states the 18/12 reading as a sitting whose program was never preserved, with no
claim about why the two differ. The implementer found the retracted causal claim in **two** report
places rather than the one I pointed at, and kept the one fact that survives without a mechanism:
`A-after` follows `B-replied` in 8 of 30 runs of one shape and 0 of 30 of the other, stated as a
fact about two samples. Round 2's own commit-subject miscount is corrected in `52682fe94`'s body,
so it lives in git rather than only in a git-ignored report.

Final scoped re-review of `08965a822..52682fe94` dispatched, scoped to load-bearing defects only.

## Task 16: complete at `65d333a59`

Final scoped re-review of `08965a822..52682fe94` returned CHANGES REQUESTED with two defects, both
false numbers, both one line, both verified by me and fixed in place rather than sent back for a
fourth round.

**A reviewer certified a wrong line number, and the next reviewer caught it.** Two run.rs docs put
`truthValue(Error_Logical_value_guard)` at `GuardInstruction.cpp:168`; the call is at `:167` and
`:184`, and `:168` is the opening brace. The first re-review had explicitly certified `:168` as
correct. What made it findable was that `exec_guard`'s own doc brackets the loop as `:167`-`:185`
three sentences earlier, so one doc block named the same line two ways. Fixed at `65d333a59`.

**H's fix reproduced the shape H was raised for.** "19 warnings total" was computed as 4 + 15 --
the two categories that were looked at -- rather than by counting the command's warnings, which are
21. The two it misses are `redundant explicit link target` in `rexx-exec` itself, the crate the
sentence goes on to clear. Confirmed independently: `cargo doc --workspace --no-deps` prints 29
`^warning` lines, of which eight are per-crate roll-ups, leaving 21 = 15 + 4 + 2. Report corrected.

B through J are otherwise all CLOSED, each re-derived by running rather than read: D measured a third
independent time, F's marker sites re-resolved, G's four ratios recomputed from the TSV, E's pointer
checked against entry 7's own text.

**Gates at `65d333a59`**, run by me: fmt 0, clippy 0, `REXX_CORPUS_GATE=1 cargo test --release
--workspace --no-fail-fast` exit 0, **186 of 186 matching**, 98 `test result: ok`, zero FAILED, zero
panicked. Tree clean.

Task 16: complete.

## Task 17 dispatched

BASE `65d333a59`. Brief at `task-17-brief.md`, controller notes at `task-17-controller-notes.md`.
All four of the task's measured claims re-run by me at `08965a822` and reproducing; the two crate
refusals come from different routes, which the plan does not say.

## Task 17: complete at `dd83eecdc`

Review returned **PASS on both verdicts** -- spec compliance with one required correction, task
quality clean. Fix round 1 closed it. Gates re-run by me at `dd83eecdc`: fmt 0, clippy 0,
`REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` exit 0, **190 of 190 matching**,
98 `test result: ok`, zero FAILED, zero panicked, tree clean. Corpus 186 -> 190.

**Ruling: the widened scope stands.** The implementer filled `.ROUTINES` and `.RESOURCES` as well as
the `.METHODS` the brief asked for, and flagged it rather than hiding it. Admitting a `StringTable`
receiver admits all three tables because the receiver test is by class, so the alternative was two
empty tables answering `The NIL object` at rc 0 where the oracle answers `a Routine` -- a silent wrong
answer, this project's worst failure mode. I measured all four table probes on both sides before
ruling: byte-identical, and the no-directive control still resolves to the literal spellings. Cost if
wrong: work done a task early, in a task that owns the mechanism anyway.

**The required fix was Task 16's defect again**, one task later: a C++ citation naming lines that do
the described thing on a branch the sentence's own example never takes. `DirectiveParser.cpp:875` and
`:880` sit inside `if (externalname != OREF_NULL)` at `:860`; a plain `::method z attribute` takes the
`else` at `:889`. Verified by me before dispatch and again after the fix, along with `:2418`, `:2474`
and the `CoreClasses.orx:65` neighbour. **Two tasks running, the citation defect has appeared in both.**

**My own recurring error showed up in someone else's report.** "Three cells at or above 1%" is six
once `size` is kept as a column -- a projection that drops a discriminator and then reports as though
the question had been asked of the whole set. The sitting tables themselves kept `size`; only the
sentence over them was wrong. Same shape as my `pinned>head` slip earlier on this plan.

**Parked:** a `Method` or `Routine` object still cannot receive a message (`.methods~z~class` is
rc 120 against the oracle's `The Method class`) -- loud, and instance receivers are 5b's. `.RESOURCES`
can have no corpus row, because `every_corpus_program_tiles` requires every byte of a program in
`corpus/lang` to be tiled by a clause node and a `::RESOURCE` body is not clauses; its sole instrument
is `dispatch.rs`'s `the_package_tables_hold_what_their_directives_declare`, recorded as sole.

**Upstream defect found and confirmed three ways** -- implementer, reviewer, and me independently.
`StringHashCollection::unknown` reads `arguments[0]` without consulting `argCount`
(`HashCollection.cpp:1026`), so `d~"A="()` stores the evaluation stack; with nothing evaluated before
it, the entry holds `STDQUE`. `setEntry` at `:854` documents that a value-less send is meant to
*remove* the entry, so the missing check is what stops it reaching its own documented path. Written up
at `upstream-tickets.md` with a one-line fix and a duplicate-check note. Filing is Moritz's.

**Stop here.** Tasks 18-24 are not authorised.

## Task 18: implementation and fix round 1

Commits `f880a4753` (pre-existing clippy), `34cd90a4a`, `423526a86`, `88bab13f1`, my `4761541c0`
(plan amendments), then `98d608364`, `d7020ed9f`, `fe8d51cb0`. Corpus **190 -> 202**. Gates re-run by
me at each stop: green, `202 of 202 matching`, 98 `test result: ok`, zero FAILED, zero panicked.

**Two of my own inputs were wrong and the implementer caught both.**

*The scope-override substitute.* My controller notes repeated the plan's claim that `self~init:super`
was available because the old plan's Task 5 landed it. True of `Interp::resolve`'s `start_scope`
parameter, false of the expression that reaches it: rc 120 on both engines. Measured with and without
the line -- K's superclass is `.Object`, whose class `INIT` is a no-op, so the oracle's transcript is
byte-identical without it. Plan amended at `4761541c0` in both places it appeared.

*The stale guard command.* `global-constraints.md`, the file every implementer is handed as binding,
carried the **six-axis** sitting command after the plan gained `dispatchclass` (Task 13) and
`bench-rexxcps/rexxcps.rex` (Task 15). Task 18's sitting ran six axes for that reason. The plan's own
paragraph warns that a guard axis living only in a dispatch is not a control; this was the same defect
one layer over, in the file that outranks a dispatch. Fixed, and the file now says the plan is the
authority and it is not. **The omission hid nothing** -- the contribution arm is flat on both missing
axes -- but a guard that can silently narrow is not a guard.

**Ruling on the moved compiler:** `PINNED.md` records `rustc 1.97.1`, head now builds under `1.98.0`,
which arrived on its own because `rust-toolchain.toml` pins the channel. Pin stays, baseline does not
restart: the 1% line is asked of the contribution arm, whose two sides build under the same compiler,
so the gate quantity is unaffected; the accumulated `pinned>head` ratio is a position, not a gate.
Rebuilding the pin would not reproduce its sha256 and would discard the accumulated series to repair
an advisory figure. Recorded in the plan's guard block. Cost if wrong: accumulated readings from
Task 18 on are not attributable to any task's work, which they already were not.

**Ruling on the duplicate-member divergence, and it paid.** The review found it recorded only in a
source comment, disposed of as "left for whichever task owns 99.902" -- a task that does not exist --
while being a silent wrong answer with no instrument. I ruled: measure the cost, build it if contained
because the sibling case (99.903, duplicate `::ROUTINE`) already lives in the same walk, stop and
report if not. It was contained. `Interp::check_member_keys` now mirrors
`LanguageParser::checkDuplicateMethod`, with the keys a directive claims lifted out of
`install_method`/`install_attribute` so the check and the install read one enumeration.

**99.905 accepted although it exceeded the ruling.** The implementer also implemented the `CLASS`
keyword with no `::CLASS` above it -- the same C++ function's other arm, another rc 0 silent wrong
answer -- and flagged it rather than slipping it in. Refusing it would have required inventing
behaviour for a loose class-side member the oracle never permits. Verified by me byte-identical.

**The citation defect has now shipped in three consecutive tasks**: a true claim pinned to a line on a
branch the sentence's own example never takes. Here, `:1875` (the value-omitted arm) cited for
`::constant c 5`, which takes `:1911`. See [[citation-names-the-wrong-branch]].

## Task 18: complete at `9a0f249e9`

Three fix rounds. Final re-review: **CHANGES REQUESTED on one number and nothing else** -- section
13.3's restore witness quoted a `lib.rs` sha256 matching no state in the range or its history. The
reviewer searched every version of the file, every other file the range touches at each commit, and
every tracked file under `rust/` at HEAD. The state it was meant to witness is correct; I verified
`b256ab48...` at HEAD and corrected the report. A digest whose whole job is to let a reader confirm
the mutations were undone cannot do it if it identifies no state.

**Over-refusal held.** The reviewer built thirty shapes to break the new checks and none refused
anything the oracle accepts, including the upcasing edge (`::CLASS "e-acute"` pairs) where a byte-wise
key could have diverged, names shared across all four kinds, and a member named like its own class.

**Corpus 190 -> 204, and three divergence classes closed that the brief never mentioned:** 99.902
duplicate members, 99.905 a `CLASS` keyword with no `::CLASS`, and 99.901/99.942 duplicate class and
resource. Every one was rc 0 with wrong output. The enumeration that exposed the last two has been in
`rexx-parse/src/directive.rs`'s module doc since Phase 3, listing exactly which subcodes the caller
owes; nobody had checked it against what the crate answers. **Candidate follow-on: take each such
enumeration in the tree and test it against the oracle.**

**Two witnesses that could not fail, caught by their own mutations.** The `::CLASS` corpus row first
paired `::CLASS a` with `::CLASS "A"`, which discriminates nothing because the tokenizer upcases a
symbol so both store `A`; and a control's comment named `R` as a class and a routine when the routine
was `f`. Both left the suite at 204 of 204 under the mutation meant to redden them. See
[[test-can-fail-is-not-test-adds-coverage]].

**The bimodal cell is settled**: `dispatchclass tw small` was 0.998019 in fix round 1 and is 1.000002
in fix round 3 over a build carrying strictly more code. A bimodal sample falling the other way, not
an effect.

## Task 19: complete at `a39a91ebd`

**It was not the bookkeeping task the dispatch expected, and the reason is worth keeping.** The
controller had measured all six of the brief's crate-side readings at BASE and found every one
matching, so the notes said this might be verification and coverage alone. I re-measured all six and
confirmed them. But the brief's **Build** list carried an item the notes did not probe -- the
parenthesised form running *as a method against the class object with `self` bound* -- and that was
not built. Probing it found **four divergences at BASE on both engines**, two of them silent rc-0
wrong answers: `::CONSTANT c (self)` answered the string `SELF`, and
`::CONSTANT c (self~hasMethod("D"))` answered `0` where the oracle answers `1`. The readings the
notes took were of the *observations the brief listed as diverging*; the gap was in the *mechanism it
listed as owed*, and a mechanism nobody has built shows up only in probes nobody has written yet.

`SELF`, `SUPER`, the calling convention's receiver (so a `PRIVATE` class method is reachable) and its
argument list (empty, not the running program's) now all come from
`ClassDirective::resolveConstants`' shape. `SUPER` reuses the function a class-side `::METHOD`
already reads, which was checked against the oracle *before* the change rather than assumed.

**The control fires, and the count is the finding.** Installing the getter class-side only leaves the
corpus at **206 of 207** with `class_constant_instance_method.rex` alone reddening, 97.1 with the
`Compiled method "METHOD" with scope "Class".` frame where the oracle answers `a Method`.
`class_constant_values.rex` stays green under it, which is the brief's reason `.K~c` cannot be the
control's subject -- confirmed by running, not by argument. Seven mutations in all. Every row of the
two new rc-0 programs has a mutation that reddens that row alone; the failure program is reached only
by the `SELF` mutation, which reddens its sibling too, so its marginal coverage over that sibling is
not demonstrated by any mutation I ran.

**One divergence has no corpus expression at all.** The argument-list rule needs a command line, and
the corpus harness passes none, so `tests/input_oracle.rs` carries it -- and the mutation that
reinstates the old behaviour leaves the corpus at 207 of 207 while reddening that test alone.
See [[test-can-fail-is-not-test-adds-coverage]].

**A citation I invented and caught before committing**: the new doc named `Interp::invoke_method`,
which does not exist -- the binder is `Interp::enter_method_body`. `/bin/grep -n` on the symbol
returned only my own two new lines. A rustdoc intra-doc link to a missing item is not a compile
error, so no gate here would have found it. See [[recorded-tool-answers-go-stale]].

**Moving a KNOWN GAP row left a dangling referent.** The paragraph beneath it opened "IS A DIFFERENT
QUESTION"; the question it was different *from* had just left the section. Nothing in that file's own
gate can see that, and it is a shape any row move can produce.

**One red seen once and not reproduced.** The first gated workspace run after the code change
reported `error: 1 target failed: -p rexx-exec --lib`, its failure names cut off by a `tail` I should
not have piped. 26 subsequent runs against the same source -- three cargo lib runs, three workspace
gates, ten direct runs of each of the two `rexx_exec` lib test binaries -- were all exit 0. Cause
unknown; other agents share this checkout's `target/`, which would explain it, but that is untested.

**The sitting's contribution arm is flat to a millionth on every axis** -- no bench axis declares a
class -- while `cycles:u` over the same builds spans 0.979953 to 1.035493. `dispatchclass ir small`
moved 0.2% in the accumulated `pinned>head` position and `ir large` did not; the figures it moved
against are **task `18-fixround-3`'s** rows (`0baa2ccae`), not task `18`'s, which carry six axes and
no `dispatchclass` at all. **`dispatchclass ir small` is not one of the cells this ledger records as
bimodal** -- those are `alloc4c ir small` and `dispatchclass tw small` -- so what the record supports
is that this axis has produced such a cell on its other arm, and not that this cell is one. The
tie-breaker the plan names is the contribution arm, and it is flat.

**Fix round 1 at `7130b222e`.** Three comment-accuracy defects, all the same shape: a claim true
before the change and contradicted by a program the change itself added. None behavioural, none
reachable by a gate.

**The one worth carrying is `sourceline_oracle.rs`.** Its module comment said the regeneration driver
takes the fallback path on `trace_numeric_request.rex` "and the primary path everywhere else". Ran
that driver instrumented to report its path over every program in `corpus/lang`: **90 of 213 take
the fallback** (commit `7130b222e`'s message says 91 -- I counted the listing by eye, then counted it
with `wc -l` twice; the message cannot be amended and nothing committed carries the number), because
constructing a package runs the prolog and installs the directives, so every
witness program whose purpose is a failure takes it. The finding was "your new file also takes it";
the sentence was already wrong about most of the corpus. **A prose fix would have been the same
defect one file later**, so the list is gone and the condition it rested on is asserted over every
corpus program instead -- no CR, no `CTRL-Z`, and a final terminator except on the one file whose
name says it has none -- with all three assertions inverted to prove they can fail. That is
[[recorded-tool-answers-go-stale]] and [[prose-cannot-review-a-procedure]] in one place: the way to
find it was to run the procedure over the whole set, not to read the sentence.

**A correction can go stale inside its own round.** The review's D1 transcript quoted line numbers
from the very corpus file D2 was about to edit. Quoting them in the D1 fix would have made it false
the moment the D2 fix landed. The comment quotes a probe instead.

**Two of my own claims the reviewer corrected, both checked before accepting**: the accumulated
sitting figures I attributed to "Task 18" are task `18-fixround-3`'s (`0baa2ccae`) -- task `18`'s own
rows carry six axes and no `dispatchclass`; and `dispatchclass ir small` is not one of the cells this
ledger records as bimodal (`alloc4c ir small` and `dispatchclass tw small` are). Also that the
corpus gate is itself single-engine (`corpus.rs:246`, `Invocation::none()`), so the new
`input_oracle.rs` row is no worse covered than a corpus row -- my report had framed it as uniquely
exposed. The tree now says which engine that file compares, where a reader of the row will find it.

## Task 19: complete at `7130b222e`

Review returned **PASS on both verdicts**; fix round 1 closed three comment defects. Gates re-run by
me at the close: fmt 0, clippy 0, `REXX_CORPUS_GATE=1 cargo test --release --workspace
--no-fail-fast` exit 0, **207 of 207 matching**, 98 `test result: ok`, zero FAILED, zero panicked,
tree clean. Corpus 204 -> 207.

**My pre-dispatch check was right about what it measured and wrong about what it implied.** All six
of the brief's claims already matched at BASE, closed by Task 18, so I dispatched this warning
against decoration. The implementer probed the one Build-list item **I had not** -- the parenthesised
form running as a method against the class object with `self` bound -- and found four divergences,
two of them silent rc-0 wrong answers: `::CONSTANT c (self)` answered the string `SELF`, and
`(self~hasMethod("D"))` answered `0` for the oracle's `1`. Six claims matching said nothing about the
seventh item on the list. **A pre-dispatch sweep bounds what is already true, never what is left.**

**The control fires on both halves**: class-side getter only gives 206 of 207, reddening
`class_constant_instance_method.rex` alone, while `class_constant_values.rex` stays green under it --
which confirms by running, not by argument, that `.K~c` could not have been the control's subject.

**All three review defects were one shape: a claim true before the change, falsified by the change's
own new witness.** The instructive one is D3, where the report *already knew* the new program takes
the fallback path and argued correctly that it was faithful, while the comment three files away
asserting nothing else does was left standing. Knowing a fact and updating the sentence that denies
it are separate acts.

**D3 was far wider than the finding, and the fix is the right shape.** Instrumented over
`corpus/lang`, **90 of 213** programs take the fallback, not one: constructing a package runs the
prolog and installs the directives, so every witness program whose purpose is a failure takes it. The
implementer replaced the list with the rule and **asserted the condition** over every corpus program
-- no CR, no `CTRL-Z`, a final terminator on every file but the one whose name says it has none --
then inverted all three to prove they can fail. I verified the conditions independently (213 programs,
zero CR, zero `CTRL-Z`, exactly `no_trailing_newline.rex`) and inverted the CR assertion myself: it
fails naming the file and the reason. Restored byte-identically, md5 checked against HEAD. See
[[prose-cannot-review-a-procedure]].

**Two of its disclosures worth keeping.** It refused my D1 transcript's line numbers because they
pointed into the file D2 was about to edit -- the D1 fix would have been false the moment the D2 fix
landed. And it disclosed that its own commit message says 91 where the count is 90; nothing committed
carries the number, because the comment states a rule rather than a count.

**Ruling: closed without a further agent round.** The fix round is comments plus assertions, both
verdicts already PASS, and I ran the verification a scoped re-review would have run -- the conditions,
the inversion, and the gates.

## Task 20: complete at `4a519518d`

Review: **spec compliance PASS, task quality CHANGES REQUIRED** on ten findings. Fix round 1 closed
eight; F9 and F10 were plan rows and mine, committed at `4a519518d`. Final re-review: **CLOSE**, no
load-bearing defect surviving, every fix checked against the tree rather than the report. Gates at
the close, run by me: fmt 0, clippy 0, `REXX_CORPUS_GATE=1 cargo test --release --workspace
--no-fail-fast` exit 0, **210 of 210 matching**, 98 `test result: ok`, zero FAILED, zero panicked,
tree clean. Corpus 207 -> 210.

**All six `::ANNOTATE` readbacks are delivered, and every ownership premise the brief rested on was
false.** `PACKAGE` was handed to Task 21 because `.context~package` was said to be the only route --
`.K~package` reaches it. `ROUTINE` was deferred to 5c because a populated `.ROUTINES` was 5c's --
Task 17 populated it. And the member readbacks were said to ride Task 9's `~method` alone, when each
also needs the Method object to receive a message. **The third premise was mine**, introduced in the
notes correcting the first two. Task 20 measured the cost of the receiver gap, found `Method` and
`Routine` could become receivers by the arms `.Package` already had, and built it.

**The task shipped two silent wrong answers and no gate saw either.** `~method` answered a fresh
object per send, so `identityHash` comparisons were `0` for the oracle's `1` and `~objectName=`
forgot the name -- rc 0 with wrong stdout over a tree the corpus called `210 of 210`. **What found it
was the implementer re-measuring a sentence it had already written claiming the difference was
unobservable.** Not an instrument.

**And the fix for that introduced a determinism defect.** `for (target, pairs) in staged` drained a
std `HashMap`, so annotation tables were allocated in per-process random order and `~identityHash`
answered the handle: I measured two distinct outputs over twelve runs, the reviewer ten. **The shipped
binary was not run-to-run deterministic**, which is the property the whole differential method rests
on, and nothing was flaky only because no corpus program happens to print such a value. Fixed with
`BTreeMap`s; verified by me at one output over twelve on each engine with the engines agreeing, and
the reviewer inverted the new control live -- under a mutation back to `HashMap` the same six tables
appear at the same six arena indices in a different assignment, which is an iteration-order
difference and nothing else.

**A data-loss event worth carrying:** the implementer destroyed roughly two hundred lines of its own
uncommitted work with `git checkout -- crates/rexx-exec/src/environment.rs` and rebuilt from its
transcript. I checked for recovery: never staged, no dangling blob holds it, unrecoverable. The
reviewer read that file's diff line by line against its history because the gates cannot see what a
reconstruction most easily drops. See [[git-checkout-destroys-uncommitted-work]].

**Open, unscoped, larger than this task:** what else in the crate answers from an arena index or a
hash iteration order. Neither the implementer nor the reviewer surveyed it, and I did not either --
my one attempt to scope it with `grep` over-matched `Vec` iteration and produced a number that means
nothing, which is [[probe-discipline]] holding. It belongs on the follow-on list beside Task 18's
enumeration sweep.

**Still Task 21's, measured at the close:** `.context~package~name` is rc 120 here against oracle
rc 0, and `.K~package~findRoutine("R")` is rc 120 against `a Routine`.

## Run boundary

Tasks 16 through 20 complete. Corpus **185 -> 210**. Tasks 21-24 are not authorised.

## Run resumed: Tasks 21-24 authorised

Authorised 2026-08-24 to run the plan to its end, whole-branch review included. BASE for Task 21 is
`4a519518d`.

**Pre-dispatch prerequisite check, run against the tree and both interpreters before composing the
brief, found two defects in the plan's own Task 21 text.**

* **Its headline verification line does not reproduce as written.** `.Array~define("ZORK",
  .methods~z)` alone is oracle rc 159 / `97.1 Object ".METHODS" does not understand message "Z"`, not
  the rc 158 / `98.985` the plan states: `.METHODS` holds the running package's own methods and a file
  declaring none has none, so the argument fails before `DEFINE` is sent. Adding `::method z` gives
  the plan's stated result exactly. An implementer following the plan would have reproduced the 97.1
  and had to decide whether the plan or the oracle was wrong.
* **`.context~package~name` answers the program's absolute file path**, measured, so the most obvious
  witness for the accessor this task exists to build cannot be a corpus row: the corpus compares
  stdout byte for byte across checkouts. Ruled: witness it by `~class` and by
  `.context~package == .context~package`, the second of which also fails against a fresh-object-per-
  send implementation, which is the shape of the silent wrong answer Task 20 shipped.

Also fixed a stale corpus count in `global-constraints.md` before handing it over: it said the corpus
was 106 of 106, which was true when written and has been false since Task 1. Replaced with the rule
rather than a number, per [[no-set-cardinality-in-prose]].

Confirmed present and not to be rebuilt: `.Array~package~name` already answers `REXX` on the crate,
and `.methods~z` already resolves. Confirmed absent: no `98.985` and no REXX_DEFINED anything in the
crate, and all five of the task's probes refuse loudly at rc 120.

### Task 21 review and fix round 1

Review: **spec compliance CHANGES REQUIRED, task quality CHANGES REQUIRED**, 2 critical, 1 major,
2 moderate, 6 minor. The review verified rather than reasoned, and its "what held" section is the
larger half: the native replay checked name for name against a live oracle dump it took itself, all
eighteen corpus rows re-run on **both** engines, all three controls confirmed genuinely inverted with
the recorded sha256s matching committed bytes, determinism over twelve runs, the pin, and about a
hundred C++ citations checked one at a time.

**Both criticals are silent wrong answers, and I reproduced both before spending a fix round.**
`.context~objectName = "tagged"` then `say .context~objectName` is `tagged` on the oracle and
`a RexxContext` here, both rc 0; `.K~defineMethods(.local)` is oracle rc 163 `93.974` and rc 0
`survived` here. Both are refusals this task turned into answers.

**C1 is the sharpest instance yet of [[new-witness-falsifies-its-neighbours]].** Adding
`Primitive::Context` so `~package` could resolve necessarily switched on every Object-inherited
method for that receiver at once. The doc comment asserting the difference was unobservable was
**cited as authority by a decision made in the same diff**, and both of its premises died in that
diff. My own note 2 had given the rule and the implementer applied it correctly one level down, to
the Package object.

Citations landing off the line they name is now **four consecutive tasks**. Fix instruction was to
run `/bin/grep -a "models no removal"` rather than patch the two sites the review named, because a
list I hand over is not a control and the search is.

**Machine-level event, mid fix round: the volume holding the repos hit 0 bytes free.** A write
truncated `run/tests.rs` to 262144 bytes; the implementer restored it from the committed blob and I
verified independently that `git hash-object` equals `git rev-parse HEAD:<path>` (d3c7d0cf2) at
363975 bytes with the path clean. It reclaimed 4.1G by deleting `target/doc` and
`target/debug/incremental`, both regenerable caches. **The consumer is not this project**:
`/home/moritz/dev/repos` is 1.5T of a 1.7T volume and `materialize` alone is 1.4T. Told it to make no
further deletions and to stop rather than free space, and to flag any odd perf axis rather than
attribute it, since a full volume with a just-deleted incremental cache is exactly the environment
difference that manufactures a fake finding.

Resolved the same day by Moritz, outside this worktree: the volume now reads 814G free at 51%. The
measurement stands as taken (55 agent worktrees under another project's `.claude/worktrees`, the
largest 263G, each carrying its own Rust `target/`); nothing in this project was the cause and
nothing here needed reclaiming beyond the two regenerable caches. Told the implementer to discard any
perf figure taken while the volume was full rather than reason about it.

## Task 21: complete at `7c23bf891`

Review: **spec compliance CHANGES REQUIRED, task quality CHANGES REQUIRED**, then **five fix rounds**.
Gates at the close, all five run by me on the committed tree: fmt 0, clippy 0 with zero warnings,
release corpus gate exit 0, debug `memcap 8G --no-fail-fast` exit 0, **232 of 232 matching** in both
gate modes, 98 `test result: ok`, zero FAILED, zero panicked, tree clean. Corpus **210 -> 232**.

**The behavioural work was right after round 1.** Rounds 2 through 5 were almost entirely about
whether the sentences describing it were true. That is the finding of this task and it is worth more
than the surface it built.

**One real regression after round 1, and it is instructive.** The C1 fix cached a `RexxContext` per
activation and rooted it by a sweep in `Interp::collect_now`. The soundness argument was that an
activation is running, suspended or parked, and each state has a mechanism. **Both halves true, the
conclusion false**: the sweep hangs off the *collection site*, not off the state, and `GC('force')`
called `Heap::collect` directly. Three lines diverged rc 0 against rc 120. Nothing caught it because
`gc(` and `.context` appeared in **disjoint sets of corpus programs** -- the same shape that hid the
parked route, which had no differential row at all until round 3 wrote one.

**The fix that held is the one a test enforces.** Round 3 turned the rule into
`heap_collect_is_called_from_collect_now_alone`, and round 4 widened its needle and then **built a
UFCS bypass that compiles, reddens `class_context_gc.rex`, and passes the assertion green** -- so the
doc now lists what the scan cannot see instead of claiming a class. That is
[[dispatch-prose-is-not-a-control]] resolved the only way it has ever resolved here.

**Four correction rounds each introduced a false sentence while fixing one, and every instance was an
added justification whose argument was right.** F1 is the sharpest: two real `~identityHash` values,
measured through variables in round 1, became a false exhibit when pasted into a comment as Rexx
literals, because unary minus is arithmetic and both collapse to `-1.40404878E+14` at `DIGITS 9`. I
had verified those numbers myself, in the variable form, and did not catch the transcription.
Round 5's governing rule -- **prefer deleting to rewriting** -- closed three of four items by
striking a clause, and a struck clause cannot rot.

**Two rulings recorded.** Round 4 kept the same implementer against the process's escalate-on-round-4
rule, on the grounds that it was already on the most capable model and the corrections were to prose
written with the measurements in hand. And concern 4's deletion was ordered over the implementer's
objection, then **falsified the sentence above it** -- a deletion cannot introduce a false new
statement, but it can strip the support from a true old one. Both halves of that are now in
[[new-witness-falsifies-its-neighbours]]'s territory.

**Inherited by later tasks:** `Queue`'s and `Stem`'s `~superClasses` close at Task 23 and are a
licensed silent divergence until then, propagating into value positions
(`.Queue~superClasses~items` is 2 on the oracle and 1 here, rc 0); two refusals with no corpus
instrument by construction; `==` on an interpreter object still refuses loudly; D43's witness and the
`~define(name, .nil)` flattened-behaviour question both need an instance and are 5b's; a latent
rooting window in `resume_reply` that nothing allocates into today; and two comments deliberately not
reached into, recorded so the next reviewer does not re-raise them.

## Task 22: complete at `5b054d4b5`

Review: **spec compliance PASS, task quality CHANGES REQUIRED** (4 major, 7 minor), then two fix
rounds. Gates at the close, all five run by me: fmt 0, clippy 0, release gate 0, debug
`memcap 8G --no-fail-fast` 0, **240 of 240 matching** in both modes, 99 `test result: ok`, zero
FAILED, zero panicked, tree clean. Corpus **232 -> 240**.

**Every major was a false sentence; the reviewer could not falsify the implementation.** It probed
the registry beyond what the task wrote and everything matched the oracle byte for byte on three
descriptors and both engines.

**The refusal split is the thing this task had to get right and did.** One function draws the
boundary and both the installer and the gap-message writer read it, so the form that binds and the
forms that refuse cannot come apart. `::ROUTINE` stays whole because routines resolve against a
*third* table (`rexx_routines[]`, exporting `Directory`/`Filespec`/`Beep`), measured: for a routine
`LIBRARY REXX file_separator` is rc 166 while `LIBRARY REXX Filespec` is rc 0.

**It caught a silent wrong answer before shipping it.** A `LIBRARY REXX` entry point checks its own
argument count from inside its activation, so too many arguments is **88.922** with **no source
line** in the traceback, not the 93.902 a primitive reports. Without modelling that, extra arguments
would have been ignored and `/` answered at rc 0.

**Two findings about method are worth more than the fixes.**

* **A sweep built from its own input can only return its input.** Round 1's needle for stale
  invocable-kind counts was assembled from strings the review had quoted; it returned exactly the
  four it was handed and missed a fifth, in the doc of the very function whose `match` that round
  edited. Round 2 rebuilt the needle from the *concept* (any quantifier within eighty characters of
  `kind`/`invocable`) and **validated it at three commits** -- base, after round 1, HEAD. The
  base-commit run is the useful one: the missed sentence was already false before this task, so it
  was an inherited defect two reviews had walked past. I reimplemented the needle independently and
  got the same trend; my absolute counts differ by one throughout because my pattern matches `all`
  inside `call`. See [[negative-claims-are-as-wide-as-their-pattern]].
* **A stale string hard-wrapped across two `///` lines is invisible to grep.** The one live instance
  of a deleted refusal message was found by collapsing comment blocks first. See
  [[grep-misses-wrapped-prose]] and [[collapsed-comment-diff-then-keyword-filter]].

**An instrument I am adopting from the implementer:** to show a comment-only round changed no
codegen, compare the `.text` section hash across a forced rebuild rather than the whole binary.
Comments always move debug info, so a whole-binary comparison proves nothing; `.text` identical is
what supports skipping a sitting.

**Also measured, and new:** shared libraries in this tree do load, and the library name is
case-sensitive -- `LIBRARY rexxutil` gives 90.998 (opened, entry missing) where `LIBRARY REXXUTIL`
gives 98.903 (not loaded). That falsified a caveat in `staged_gap` whose ground had been wrong since
before this task.

**Inherited:** `::ATTRIBUTE EXTERNAL` is the only 5a-owned gate table D row not at `agree`, with the
fix shape recorded; `::ROUTINE ... 'LIBRARY REXX <routine>'` is oracle rc 0 against rc 120 here and
no gate row sees it; the two implemented entry points answer unix values unconditionally; the timer
and queue phase assignments are the implementer's own, not the roadmap's, and cost one word each if
wrong.

### Ruling before Task 23 dispatch: the scope-override send is Task 23's

**Pre-dispatch check found a blocker the plan does not assign.** `CoreClasses.orx` is rc 120 on the
crate: `a message scope override on "The StringTable class" is not implemented (Phase 5)`, no frame.
Located: `CoreClasses.orx:3993` declares `TraceObject subclass StringTable` and its **class-side**
`::method activate class` runs `self~activate:super` at `:3998` before assigning `option = 'N'`.
Installing a class runs `ACTIVATE`, so the send is on the install path -- and `.TraceObject~option`
being `N` is the brief's own **check 3**, the one that separates a bootstrap that ran `ACTIVATE` from
one that did not. The check cannot pass while the send refuses.

Measured, so the boundary is not guessed: a program whose **uncalled** body holds `self~init:super`
is rc 0 and byte-identical to the oracle, so this is not a translation-time gap. Only the executed
path matters, and the bootstrap executes this one. Minimal differential recorded in the notes:
oracle rc 0 `base+k`, crate rc 120 both engines.

**The plan contradicts itself.** Line 58 lists "the scope-override send (`~m:super`)" among old
Task 5's deliveries -- false, measured. Lines 150 and 1699 say the opposite and are right: Task 5
landed `Interp::resolve`'s `start_scope` parameter, not the expression reaching it. Line 2131 assigns
`self~init:super` to 5b, but as *instance construction chaining*, which is a different question from
whether the send form exists.

**Ruling: Task 23 builds the send** -- the expression form reaching the `start_scope` parameter that
already exists, on both engines, with the oracle's errors for a non-superclass scope. 5b keeps `~new`,
`init` chaining and `UNINIT`. **Cost if wrong:** the task grows by one expression form; if the send
turns out to need the whole superclass-search machinery instead of the existing parameter, the
implementer reports BLOCKED and I re-rule rather than let Task 23 absorb unbounded 5b work. Line 58
is corrected as part of the task, since it claims a delivery that does not exist.

### Task 23 attempt 1: the install half, and the five mechanisms the prologue still needs

Commits `8f7a757e4` (the scope override) and `36c5903c5` (`rexx-lib` and the install differential).
Gates green at both: `fmt`, `clippy -D warnings`, release suite, release corpus gate, debug corpus
gate under `memcap 8G`. Corpus **242 of 242**, from 240.

**The ruling held and cost what it said it would.** The scope override is one expression form:
`Interp::message_term` validates the scope and passes it to the `start_scope` parameter that already
existed, and `rexx-classes` gained one accessor beside the class-side one it already had. Two
refusals rather than one, because the oracle has two -- 88.914 for a scope that is not a class object
and 93.957 for a class the receiver's behaviour was never given, both raised before the arguments are
evaluated. Two corpus rows, `_chain` and `_not_a_scope`.

**What that unlocked is bigger than the send.** `CoreClasses.orx` and `StreamClasses.orx` now
**install completely and byte-identically to the oracle**, every `::` directive of both, on both
engines. `tests/bootstrap_install_oracle.rs` composes their directive sections into one program and
runs that differential. The brief's **check 3** (`.TraceObject~option` is `N`) and **check 5** (the
setup methods gone, `DEFINE` present) pass there, against the oracle rather than against a copy of
this crate.

**The brief's control was run and inverted, live.** Suppressing the `ACTIVATE` pass in
`Interp::install_directives` and rebuilding gives `OPTION` where the oracle gives `N`, on both
engines, and reddens the new differential; restoring from a scratchpad copy leaves the file
sha256-identical and the check green. The `rexx-lib` sha256 pin was inverted the same way, in both
directions, on the shipped `build.rs`.

**Where it stopped, and why that is a ruling and not a slow implementer.** The prologue needs five
mechanisms the brief's build list does not name, enumerated with their measurements in the plan's
Task 23 section. The blocking one is `DO OVER` a `StringTable`, which is `CoreClasses.orx:63` and
`StreamClasses.orx:45` and has no route around it. **The mechanism behind it is hash-collection
iteration order** -- an order the oracle fixes and this plan assigns to nobody -- and it is
reproducible in principle (`HashContents`' bucket order, modelled and matched against a seven-class
probe) but only for a table whose insertion sequence this crate already matches. That is true for a
package's public classes and false for `.environment`. Implementing it with any other order turns a
clean rc-120 refusal into a silent wrong answer, which the global constraints forbid and which no
corpus row can catch.

**Three of the five are one shape and are not landable alone**: `defineClassMethod`,
`inheritInstanceMethods`, the open `REXX_DEFINED` lock and the write-open REXX package are all
*bootstrap-time* interpreter state. Offering any of them to an ordinary program diverges from the
oracle, so each is unwitnessable until a bootstrap exercises it -- which is why attempt 1 landed none
of them rather than landing four mechanisms with no witness.

**Two method notes.**

* **The new differential adds no measured coverage, and it says so.** Four mutations were applied and
  run with the file moved out of `tests/`: the corpus catches three (`ACTIVATE` suppressed,
  `ACTIVATE` reversed, the two `has_scope` arms swapped) and `native_classes_wiring.rs` catches the
  fourth. What it does that nothing else does is run the upstream files' own bytes through both
  interpreters. See [[test-can-fail-is-not-test-adds-coverage]].
* **No sitting for `36c5903c5`**, and the reason was measured rather than asserted: `rexx-lib` is a
  dev-dependency, and the release `rexx-run`'s `.text` hashes identically across forced rebuilds with
  and without it. The first attempt at that check compared two builds cargo had not actually rebuilt
  -- 0.04s and 0.02s -- which is [[checks-blind-to-their-own-subject]] inside the very check meant to
  license skipping the guard. `touch`ing a source and reading the 27.6s build times is what made it a
  measurement.

### Task 23 attempt 2: the bootstrap runs

Moritz's challenge to attempt 1's blocker 1 was right and it resized the task. Both prologue loops
are order-insensitive -- one distinct key into `.environment` and one into the package per pass, and
no output -- so the bootstrap needs the iteration and not the oracle's hash order. `DO OVER` a
`StringTable` is built with sorted key order, the order divergence is recorded beside the iteration,
and no corpus row depends on it.

**Narrowed from the ruling, and the reason is a measurement**: `Directory` (`.environment`,
`.local`) keeps the rc-120 refusal where `StringTable` iterates. A `StringTable` this crate hands out
holds exactly what the running program's directives put in it, so its membership is the oracle's
(measured, three unattached `::METHOD`s answer `3` on both); `.environment` and `.local` are modelled
as a subset, so iterating one would differ in **membership** and not only in order -- measured,
`.local` iterates ten entries on the oracle and none here. Nothing in the bootstrap iterates either.
Cost if wrong: one line in `ObjectModel::iterable_collection_class`.

**Four things the enumeration did not name**, each found by running rather than by reading, and each
now in the plan's Task 23 section:

* `Setup.cpp`'s `InheritInstanceMethods` macro is `RexxBehaviour::inheritInstanceMethods` and
  `Class~inheritInstanceMethods` is `RexxClass::inheritInstanceMethods` -- **different functions,
  same name**, and this crate had one implementation serving both. The macro copies from the donor;
  the method rewrites the donor. Latent until the bootstrap rebuilt a donor's behaviour, then
  `.array~at('x')` reports scope `"Queue"`. Two corpus rows caught it,
  `array_index_refusals.rex` and `array_make_string_refusals.rex`.
* Directive class-name resolution read the native table alone, so `StreamClasses.orx:506`'s
  `inherit Comparable` could not see what `CoreClasses.orx`'s prologue had put in `.environment`.
* `removeSetupMethods`'s walk cannot use the ordinary cascade for `.Object` or `.Class`; the two
  bootstrap-only merges have to be redone after it.
* Every per-run instrument needed a boundary. `Outcome::collections` is counted from the end of the
  bootstrap, and the compiled engine's `#[cfg(test)]` counters are suspended across it -- suspended
  rather than zeroed, because every test reads a delta taken outside `execute` and a counter zeroed
  inside it makes that subtraction meaningless.

**Two committed tables moved and both are decisions rather than churn.** `assertions.rs`'s `EXEMPT`
lost 22 rows: `test_hexadecimal`/`test_binary` open with `tab = .String~tab`, which answers now that
the prologue's `defineClassMethod` has run, so every row in either method whose own text carries no
`self~` send started passing. `collect_stress.rs`'s zero-collection set gained three `REXX_DEFINED`
refusals and the scope-override refusal and lost four programs that now allocate.

**The cost has two halves and the sitting found the second one.** The fixed half is the bootstrap:
148.4 million `instructions:u` per run, which the sitting's `fixed` intercept gains on every axis and
which Task 24's cold-start framing already covers. The per-pass half is the library being
**resident**: `emptyloop` and `varlookup` are flat to 0.00% and every allocating axis moves --
`arith` +3.59%, `dispatchclass` +6.20%, `compound` +10.68%, `alloc4c` +12.77%, `strings` +31.35% --
and a control build with `bootstrap_library` not called puts each back on its pre-task figure, so it
is the collector tracing the library and not any other change here. **Task 24 inherits that and its
framing does not cover it.** The release suite also got much slower: the slowest test binary is 469 s
and the whole command stopped fitting in the ten-minute window it used to finish inside, no
before-and-after total taken. Profiled: the fixed half is `MethodDict::add_method`'s hashing during
the install cascade, not the parse and not the removal walk -- both measured, the second by deleting
it (6.9 million).

**The staleness test has a pathspec trap and I walked into it once.** Run from `rust/`,
`git log 15a1ffa98..HEAD -- rust/crates ...` answers **0 commits**, because a git pathspec is
relative to the working directory; from the repository root it answers 119, every one recorded. A
zero read as "no foreign commits" would have been [[negative-claims-are-as-wide-as-their-pattern]]
inside the very check that guards the pin.

**Check 2 has a better instrument than the corpus row: gate table C's own wiring half.** Read under
`REXX_CORPUS_GATE=1` -- a progress report and not a gate, since 5a is not closed until Task 24 --
its class wiring rows read **62 `agree`, one `diverge-both`** -- the one being
`RexxInfo`, which `.environment` holds as an instance rather than a class. `Queue` and `Stem` are
among the 62, which is exactly the pair Task 21's "Done when" excepted and recorded against this
task. The 57 hierarchy edge rows all still diverge and none of it is this task's: every one ends at
`method "HASITEM" of class "Array" is not implemented`, and each now prints its `child` and `parent`
lines first. Neither figure was taken before this task, so neither is a delta.

**Five controls, each applied and inverted**, with the four mutated files restored from scratchpad
copies and `sha256sum` diffed against the pre-control listing. Controls 1 (the `StringTable`
iteration yields nothing) and 2 (`remove_setup_methods` not called) are the informative pair: each
reddens exactly the rows about the mechanism it broke. Controls 3 (the lock left closed), 4
(`String~upper` folding to lower) and 5 (the `CALL` interception removed) redden *everything*,
because each breaks the bootstrap itself -- worth stating rather than reading as strength. The brief
said not to re-run attempt 1's `ACTIVATE` control and I did not.

**Follow-up on the hash order, and it strengthens the ruling rather than changing it.** Moritz asked
what table ooRexx actually uses, since Rust seeds `HashMap` precisely to stop anyone depending on an
order. It is open hashing with **no seed**, and the controller's reading of the mechanism held. What
neither of us had is that **reproducibility depends on the key type, not on the collection class**:
the bucket is `key->getHashValue() % bucketSize`, which is content-derived for a string
(`31*h + byte`) or a number, and `RexxObject::identityHash` -- `((uintptr_t)this) ^ UINTPTR_MAX`,
the address -- for everything else. I re-measured rather than taking it: ten oracle runs of an
eight-object `IdentityTable` printed in iteration order give **five** distinct outputs, each a
rotation of one cyclic sequence, where the controller measured four; ten runs of an eight-key
`StringTable` give one. So for an object-keyed collection **the oracle does not agree with itself**
and matching its order is impossible rather than expensive.

**Written down where the project looks**, per [[record-findings-where-the-project-looks]]:
`rust/corpus/README.md`'s "The one rule: determinism" already said "no addresses, no iteration over
an unordered collection", and this case is both at once with nothing in the program saying so. The
rule now names it: no corpus program may print the iteration order of a collection whose keys are
objects rather than strings or numbers -- which covers an `IdentityTable` outright and a `Table`,
`Set`, `Bag` or `Relation` whenever what went into it was not a string. `Interp::
hash_collection_indexes` and the plan's Task 23 row carry the same split, the plan's narrowed from
"the order is deterministic" to "the string-keyed order is deterministic".

### Task 23 fix round 1

Spec compliance PASS, task quality CHANGES REQUIRED: 2 major, 2 moderate, 7 minor, all addressed.

**M1 was a refusal that became a wrong answer, and the ruling was to close it.** A condition raised
inside a library method printed this crate's clause echo where the oracle prints
`Method &1 with scope "&2" in package "&3" (no source available).`, and named the running program's
absolute path where the oracle names `REXX`. `FailureSite::Sourceless` is the fix: a level with a
line and an indent like a clause and a catalogue message where the clause text would be, with the
report's banner taking its name from the innermost line-bearing site. Re-measured the reviewer's
way, all 73 library class-method pairs: **7 match, 14 loud, 52 diverge -- the same classification the
review measured before the fix -- and all 52 differ only in the two `Error` lines**, checked by
stripping them and comparing the rest. Two corpus rows, one of them the
two-frame nested case; corpus **248 of 248**.

**The other half is inherited and I reproduced it rather than trust the review**: `use strict arg`
gives 40.3/40.4 where the oracle gives 93.901/93.902, on the pinned pre-5a binary, with a purely
user-declared class method. Task 23 made 52 programs reach it and did not create it.

**Three lessons the round is worth recording for.**

* **M2 is [[insertions-orphan-doc-blocks]] again, and mine.** Inserting `donate_instance_methods`
  above `inherit_instance_methods` silently reassigned the latter's doc comment to the former, and I
  then appended text saying the opposite two paragraphs down. One block, two contradictory halves,
  on the item whose whole point is the distinction; `fmt` and `clippy` pass over it. Six instances
  were recorded on one plan before this one.
* **D2 and m1 are the same defect in two media**: a true claim propped up by evidence that does not
  survive being run. "Neither non-entry file contains the word `call`" was false (`grep -in` answers
  2, both in comments) for a property that does hold; "so this is the collector" was an inference
  where a control had separated something narrower. Both struck rather than patched.
* **m5 is the more interesting one: the before-figure I said I could not take was sitting in
  `target/`.** A test binary built between the base commit and the first code commit still runs, and
  running it turns "I did not measure the before" into **+47 `agree` and the elimination of all 13
  silent-wrong-answer rows** in gate table C. `target/` is not a record, but it is evidence, and a
  stale build of a differential harness is a before-state that costs one command. I re-ran it rather
  than copy the reviewer's numbers, and reproduced them exactly.

### Fix round 1, second finding: a gate line that was written before the run

The controller ran the five gates at `86d73d269` and the gated workspace command exited **101**:
`collect_stress.rs:448`, the zero-collection set drifted by exactly the two corpus rows this round
added. `ceb72f0c3` registers them.

**The defect worth recording is the report line, not the drift.** The report claimed five green
gates for `cfffc7899` and `87cf62507`. It was not a stale result carried forward from an earlier
tree -- **the run never happened at those commits**. I wrote the gate block while a chain was in
flight, meaning to fill in the statuses; the chain was killed when the controller asked me to commit,
and the block stayed as written. A prospective claim sat in the report in the exact shape of a
measurement.

This is [[verification-commands-that-do-not-run]] with the failure moved one step later: there the
command was malformed and never executed, here the command was fine and the *reading* was
fabricated. Both produce a green line that reads identically to a real one. `rust/CLAUDE.md` already
says "Read every exit status unpiped, and confirm the command actually ran"; what it did not say,
and what this adds, is **write the gate line only from output you are looking at** -- never leave a
placeholder in the shape of a result.

Two consequences recorded rather than promised:

* The report's attempt-2 gate section is **retracted** in place, naming the two commits, saying what
  *was* seen printed (`fmt`, `clippy`, and `246 of 246` from `cargo test -p rexx-exec --test corpus`
  run directly), and saying which three commands were never run at those commits.
* `248 of 248` had the same provenance error even though the number is real: it came from the corpus
  differential invoked directly, not from the gated workspace command, and the report said the
  latter. `cargo test` stops at the first failing binary, so at `86d73d269` the gated run never
  reached the corpus harness at all.

### Fix round 2: the miscount, and where it came from

**The breadth figure was 51 and is 52**, in three places, and my own arithmetic gave it away: 7 + 14
+ 51 is 72 against 73 pairs. The cause is worth more than the number. The sweep read its subject
list with `while read` from a file written by `'\n'.join(...)`, which leaves **no trailing
newline**, so the last line -- `File writeChars` -- was never read and landed in no bucket. `wc -l`
agrees with `while read` and reports 72, so the two instruments that could have caught it agree with
each other and both undercount.

**The finding that was actually there is stronger than the one I claimed.** I wrote that a pair had
moved from diverging to matching. Nothing moved: the review measured 7/14/52 before the fix and the
re-review measures 7/14/52 after it. What changed is the *content* -- stripping every line beginning
`Error ` leaves **52 of 52 identical**, where before the fix each carried a wrong frame and a wrong
program name as well. Re-derived here over all 73 with the newline added.

**And the collector retraction was false about its own document.** The bullet said "an earlier draft
of this section claimed it did"; the unqualified sentence was eighteen lines above it, in the same
shipped text. Struck rather than re-worded, per the round's rule: a clause that is not there cannot
rot. See [[correction-rounds-introduce-false-statements]] -- a retraction is a correction, and this
round's own correction was the false statement.

## Task 23: complete at `571275f0b`

Two attempts, one BLOCKED, two fix rounds. Gates at the close, all five run by me on the committed
tree with `--no-fail-fast`: fmt 0, clippy 0, release gate 0, debug `memcap 8G` 0, **248 of 248
matching** in both modes, 102 `test result: ok`, zero FAILED, zero panicked, tree clean.
Corpus **240 -> 248**.

**The Rust interpreter now boots ooRexx's own Rexx-written class library at interpreter start.**
`CoreClasses.orx` and `StreamClasses.orx` install completely and the prologue runs to its `exit`;
the bootstrapped state is byte-identical to the oracle on both engines. Verified by me directly:
`.Alarm~id`, `.Stream~id`, `.Comparable~id`, `.Queue~superClasses`, `.Stem~superClasses`,
`.TraceObject~option`, the two `hasMethod` rows and `.Supplier~method("ALLITEMS")`. Task 21's two
deferred `~superClasses` rows closed here as it predicted.

**Gate table C went from 15 `agree` to 62, and all 13 `diverge-stdout` rows are gone** -- the
silent-wrong-answer class. That delta exists only because the reviewer found a before-figure the
report had written off: a `gate_table_c` test binary in `target/` built between the base commit and
the first code commit. Provenance is its mtime, so the before column is approximate.

**A whole divergence class closed that the brief did not name.** An error raised inside a library
method printed the library's own source text and **the user program's absolute path**; the oracle
prints a sourceless frame and names the package. 52 of 73 library class-methods carried it, and the
path leak meant no corpus row could pin it on any checkout. Ruled close-it-do-not-license-it, and it
is closed: the reviewer verified the fix implements the oracle's own rule
(`Activity::generateProgramInformation` takes the first frame whose `getPackage()` is non-null and
uses that one frame for both position and name) and could not break it across seven nestings. All 52
now differ only in the two `Error ` lines, which is the pre-existing `USE STRICT ARG` defect and not
this task's.

**My own error, and the one that mattered most.** Attempt 1 reported `DO OVER` a `StringTable` as
blocked on hash-collection iteration order, and I carried that up as the blocker without checking
whether the bootstrap needed the *order* or only needed to *iterate*. Moritz challenged it. Both
prologue loops are order-insensitive -- each pass puts one distinct key into `.environment` and one
into the package, and the prologue prints nothing. The wall was not a wall, and the task shipped a
day later. See [[rule-the-observable-not-the-mechanism]]: "the mechanism behind it is hash order" was
true of *matching* the oracle and false as a description of the blocker.

Measuring it properly then gave something better than either reading:
[[oorexx-hash-iteration-order]]. ooRexx is unseeded, the opposite of Rust; string-keyed order is
fixed, and object-keyed order **does not match itself across runs** because the bucket is an address.
`rust/corpus/README.md` now carries the prohibition that follows.

**Three reporting failures from one implementer, none of them in the code.** Twice it went idle with
the entire round uncommitted -- 20 files once, 17 the next -- and once it reported five green gates
for a tree where two of the five had never run: a results block written as a placeholder while a
chain was in flight, killed, and shipped in the shape of a result
([[prospective-result-lines]]). The `248 of 248` in it was real and printed by a different command
than the one it was credited to. It disclosed the mechanism itself and added the constraints rule.
**Every one of these was caught by running the gates myself on the committed tree**, which is the
only reason none reached the branch looking green.

**Perf: the cost is real and the attribution was withdrawn.** ~148M `instructions:u` per run, and
per-pass movement from flat on the two non-allocating axes to +31.35% on `strings`. "So this is the
collector" is withdrawn: the control build separates *the bootstrap ran* from *the new code exists*,
no collection count was read, and a larger resident heap has other costs predicting the same
pattern. The observable stands and Task 24's cold-start measurement inherits it as an open question.

**Inherited:** the 57 hierarchy-edge rows all end at one missing method, `Array~hasItem`; the
`USE STRICT ARG` condition defect (40.4 where the oracle raises 93.902) is pre-existing, proven on
the pinned pre-5a binary, and now reachable from 52 library methods; `Directory` iteration stays a
loud refusal on membership grounds, not order.

## Task 24: complete at `433ac60fc`

Review: **spec compliance CHANGES REQUIRED, task quality CHANGES REQUIRED** (1 major, 3 minor), one
fix round. Gates at the close, all five run by me with `--no-fail-fast`: fmt 0, clippy 0, release
gate 0, debug `memcap 8G` 0, **248 of 248 matching** in both modes, 102 `test result: ok`, zero
FAILED, zero panicked, tree clean.

**Spec compliance is CHANGES REQUIRED because the brief's first clause is unmeetable, and that is
the task's finding rather than a defect in it.** The gate was measured instead of asserted and the
measurement says **5a is not closed**: five rows owned by 5a do not agree with the oracle, each
stopping at a 5a mechanism no task of this plan built. I ran all five by hand; every one is a
genuine rc 120 naming its mechanism.

| row | stops at | filed to |
|---|---|---|
| `concepts/usingcl.rex` | `Class~mixinClass`, `Class~subclass` behind it | Task 7 |
| `concepts/xscope.rex` | `Method~scope` | Task 9 |
| `concepts/methna.rex` | `~define(name, 'source')` | Task 21 |
| `classes/rexxinfo.rex` | `.REXXINFO` environment symbol | standing deferral |
| `directives/attribute__external__subkeyword.rex` | `::ATTRIBUTE EXTERNAL` | Task 22 |

**The flip is deliberately not committed**, and the reasoning is sound: five standing reds would
make `global-constraints.md`'s "any red is a regression" false for every later task, and the first
thing a later task would need is a list of names to read past -- the mechanism that constraint
exists to prevent. The exact line, its file, the five rows and the reasoning are in the plan's own
Task 24 section instead.

**It declined the easy exit.** The `::ATTRIBUTE EXTERNAL` row is filed 5a *by omission* -- table D's
catch-all sweeps `::ATTRIBUTE` into 5a and its doc enumerates every other exception it makes.
Re-filing it to Phase 7 would have closed a gate row by narrowing what the gate covers. See
[[gate-criteria-failure-modes]]: this is the adversarial read applied by the implementer to its own
task.

**The best-designed control of the run.** Rather than breaking something global it reverted one
mechanism -- `native_superclasses` truncated with `.take(1)` -- and 40 of 1488 rows reddened while
1448 were unchanged. Its own note is the general lesson: *a control that reddens everything
witnesses far less than one that reddens a nameable set*, which is a direct correction of Task 23's
three controls, each of which reddened everything by breaking the bootstrap.

**The major finding was a false negative, and the reviewer could not find a sixth failing row** --
it re-derived the set of five three independent ways without taking a number from the report. The
report said a control build "still needs" running while reading the file that contains it: the last
156 rows of `phase-5a-arms.tsv` are `23-attempt-2-control-no-bootstrap` at a commit whose `src/`
diff against HEAD is empty. Verified by me. It already separates the costs -- `strings`, per pass,
ir: pin 5,369.53, bootstrap suppressed 5,406.48, shipped 7,136.86 -- so the movement is per-pass
with the resident library and a `per_pass` figure cannot carry a fixed 148M. **The consequence that
mattered was the dropped handoff**: Task 23 passed forward a decision about the collector's cost
model with a large resident set, and the boundary lists had no place for a cost, so it fell out. It
is now carried.

**And a divergence no gate row can see**, found by running the catch-all audit in the direction the
report had not: `("::ROUTINE", "EXTERNAL")` is filed to Phase 7 for both spellings because a row's
identity is (directive, keyword, position) and its probe picks the shared-library form. So
`::routine r external 'LIBRARY REXX Filespec'` is oracle rc 0 against rc 120 here -- the same
`LIBRARY REXX` spelling D37 moved into 5a for `::METHOD` -- and unlike `::ATTRIBUTE EXTERNAL`'s, no
row of either table fails on it. Named in the 5b/5c boundary.

## Plan complete

All 24 tasks executed. Corpus **185 -> 248** across this run, `phase-5a.txt` **51 -> 193**.
Gate table C class wiring **15 -> 62 `agree`**, all 13 `diverge-stdout` rows eliminated.
**5a is not closed**, on the five rows above.

### The guard's blind axis, found by a question rather than by a check

Moritz asked how `rexxcps` moved, after I quoted a seven-axis per-pass table for an eight-axis guard.
It is the axis that moved **most** in Task 23 and the only one the fixed/per-pass decomposition
cannot see.

Read from `phase-5a-arms.tsv`: `rexxcps` accumulated position against the pin went **1.0187 -> 1.1654**
on the ir arm and 1.0151 -> 1.1320 on tw across Task 23, i.e. about +14 percentage points. Absolute,
ir: 18,995,149,153 -> 22,136,792,605, a gain of **3.14 billion instructions**. The bootstrap's fixed
cost is ~148 million, which is 0.8% of that total, so **roughly three billion of it is per-pass** --
the same resident-library cost that reads +31.35% on `strings`.

**Why no per-pass figure exists for it:** `rexxcps` runs at one size only, where the other seven axes
run at two. The harness fits `fixed` (intercept) and `per_pass` (slope) across the two sizes, so a
single point has no line to fit and `rexxcps` carries only `absolute`, `across_builds` and
`arm_ratio` rows. Confirmed by listing scopes per axis in the baseline.

**Why that is unlucky rather than merely incomplete:** `rexxcps` is the classic clauses-per-second
benchmark and the most realistic workload in the guard -- a mixed program rather than a targeted
microbench. The synthetic axes got a clean attribution and the representative one got a single
number nobody decomposed. On a realistic workload the resident library costs about **16%**, not the
low single digits the flat axes suggest.

Handed to whoever takes the collector cost decision: either add a second size to `rexxcps` so the
guard can decompose it, or record the asymmetry so 5b and 5c know the representative axis is the one
the decomposition cannot attribute.
