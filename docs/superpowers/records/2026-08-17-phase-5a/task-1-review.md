# Task 1 review — the run timeout, and what "did not finish" means

Base `d926c334d`, head `1212d7f80` (`d5647aa4d` the mechanism, `1212d7f80` the verification).
Read-only review; nothing in the tree was edited.

## Verdicts

* **Spec compliance: APPROVE.** Every item the brief names is present, at the values it names, and
  each one I could check independently checked out — including the two the brief singles out
  (`did_not_finish` tested on the status and never on an exit code; the classification a pure
  function of `(Option<i32>, bool)`).
* **Task quality: REWORK.** Three medium findings, all in the same place: the harness's new
  guarantee is narrower than what the code and its docs claim, and the one verdict function that
  now reads `Termination` directly treats "neither side finished" as agreement. None of them is a
  wrong answer for a caller that exists today; all three are latent in a harness that twenty-three
  probe-committing tasks are about to lean on, which is why they are worth closing now rather than
  when a probe finds them.

## What I ran, and what it showed

| command | result |
| --- | --- |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --release --workspace` | exit 0; corpus `106 of 106 matching` (report mode); `oracle_deadline` 13 passed, `finished in 0.00s` (skipped) |
| `REXX_CORPUS_GATE=1 cargo test --release --workspace` | exit 0; corpus `106 of 106 matching` |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | exit 0; corpus `106 of 106 matching` |
| `REXX_CORPUS_GATE=1 cargo test --release --test oracle_deadline -- --nocapture` | exit 0; 13 passed, `finished in 10.01s` |

All five gate commands reproduce. The report's own numbers reproduce.

**Is `finished in 10.00s` evidence the deadline fired, or merely consistent with it?** On its own,
merely consistent — a program that ran ten seconds and exited normally would print the same line.
What makes it evidence is the assertion beside it: the test asserts
`outcome.termination == Termination::TimedOut`, and `TimedOut` is constructible from exactly one
place in the tree — `classify_termination(None, true)` at `support/oracle.rs:170` — whose
`deadline_exceeded: true` is passed from exactly one place, `wait_with_deadline`'s deadline branch.
Grepped: no other production site produces it. Combined with `do forever; end` having no way to
exit on its own, the classification cannot be reached except by the kill. That is a real witness,
not decoration.

**What would `oracle_deadline.rs` have done had the deadline never fired?** It would have hung —
`oracle.run` would not return, and `cargo test` would sit on it forever. So the test cannot pass for
another reason (an erroring program gives `Exited(n)` and fails the first assertion; a missing
oracle makes `locate()` fail loudly; a missing file fails the `fs::write`/`canonicalize`
`unwrap_or_else`; the gate variable being unset makes it print `*** SKIPPED ***` and prove nothing,
which is the house convention and is stated in the skip line). Its failure mode for a *deleted*
mechanism is a hang rather than a red — the one thing it cannot itself catch, and unavoidable
without a second deadline wrapped around the first.

## Findings, most severe first

### M1 (medium) — the deadline bounds the wait, not the read: a probe that leaves a grandchild still hangs the suite

`wait_with_deadline` returns only after `stdout_thread.join()` and `stderr_thread.join()`, and each
thread is blocked in `read_to_end`, which returns when **every** write end of the pipe is closed —
not when the direct child dies. `kill()` reaches the direct child only (the `sh -c … exec` wrapper
makes that child `rexx` itself, correctly). Any process the program leaves behind holds the
inherited descriptor and keeps both joins blocked, with no bound at all.

Demonstrated against the real oracle, from a fresh empty directory, program
`address system 'sleep 12 &'` / `say 'done'`, run through the same
`sh -c 'ulimit -v 1048576 && exec "$0" "$@"'` wrapper the harness builds and captured to EOF the way
the reader threads do: `rc=0`, output `done`, **elapsed 12011 ms** — the Rexx program exited
immediately and the read blocked for the grandchild's whole lifetime. The same shape without a Rexx
interpreter: `out=$(sh -c 'sleep 4 & echo hello; exit 0')` took 4003 ms.

So `address system 'sleep 3600 &'` in a committed probe hangs `cargo test --workspace` for an hour
despite Task 1, and `do forever` plus such a grandchild hangs it forever: the kill fires at ten
seconds and the join never returns.

This is **not a regression** — `Command::output`/`wait_with_output` read to EOF the same way, on top
of an unbounded `waitpid`. It is a gap between the mechanism and the claims made for it:
`wait_with_deadline`'s doc says the poll "calls `kill()` the moment `ORACLE_DEADLINE` has passed",
`oracle_deadline.rs`'s module doc says the call "**returns** … inside a bounded amount of wall
time", and the report's "What this does not cover" names only the in-process crate half. The brief's
goal sentence is "No probe can hang the workspace"; this path is neither closed nor named.

Either bound it (hand each reader thread's buffer back over a channel and stop waiting on the join
once the child is dead and the deadline has passed, accepting a possibly-truncated transcript on a
run already classified as a non-finish), or say in both docs that the deadline bounds the child, not
descriptors the child passed on, and add it to the human-check list beside the in-process half.

### M2 (medium) — `input_oracle.rs`'s `diffs` now reads two non-finishes as agreement

`diffs()` was changed from `rust.exit_code != cpp.exit_code` to `rust.termination != cpp.termination`
(`input_oracle.rs:405`). If both sides fail to finish the same way — both `Signaled`, the case where
`rexx-run` and the oracle both die from a signal — the comparison is equal, `channels` is empty, and
the case is counted as **agreeing**. Nothing in the loop at `input_oracle.rs:428` guards it: there is
no `did_not_finish` check on either side.

The global constraints say an oracle run that did not finish is a structural failure, always red, in
both modes. Here it is green. This is the one call site where the brief's helper was available and
was not used, and it is the site the brief's own motivation describes — "a run killed by a signal …
which any verdict function reads as `diverge-status`". The new type converts that particular verdict
function's answer from a false red into a false green, which is the worse of the two.

Not a regression (`-1 == -1` compared equal before), and the probability of both interpreters dying
by signal on one case is low. The fix is one line: `assert!(!did_not_finish(&rust) && !did_not_finish(&cpp), …)`
before the comparison, or a `did_not_finish` arm in `diffs` that pushes a channel name.

### M3 (medium) — a timed-out program reddens without naming itself

`corpus.rs:check_case` calls `descriptor_diffs_with` for every case, which now calls
`cpp.expect_exit_code()` unconditionally (`support/oracle.rs:525`). A corpus program that times out
therefore panics with `oracle run did not exit normally, so it has no exit code: TimedOut` — red in
both gate modes, which is the shape the brief wants — but the message names no program, the panic
location is `support/oracle.rs`, and `corpus.rs`'s accumulated report is emitted *after* the loop, so
it never prints. The operator learns that one of the corpus programs stopped finishing, not which.
The same holds for `builtin_status.rs` and `state_builtin_oracle.rs`, which sweep many programs per
test.

Concretely: add a `do forever; end` program to `corpus/phase-5a.txt`, and the gate run goes red after
ten seconds with a message that could name the file and does not. `expect_exit_code` cannot know the
path; the caller can. Either check `did_not_finish` in `check_case`/`sweep_one`/`measure` and produce
a `Mismatch`-shaped structural line naming `rel_path`, or give `expect_exit_code` a context argument.

### L1 (low) — the deadline branch throws away a real exit status

```rust
let _ = child.kill();
let _ = child.wait();
break (None, true);
```

`child.wait()` here returns the process's actual status, and the code discards it and asserts
`None`. A program that exits between the last `try_wait` and the deadline check — anything finishing
at ~9.99 s under a loaded `--no-fail-fast` debug run — is killed as a no-op (the kill lands on a
zombie), `wait()` reports its true `Exited(0)`, and the harness still reports `TimedOut`, whereupon
`expect_exit_code()` panics and the suite goes red on a run that completed successfully. Flaky
structural red, not a wrong answer.

`classify_termination`'s `(Some(status), _)` arm — the one the paired unit test pins — exists exactly
for this and is unreachable from the only caller, because the caller hardcodes `None`. Binding the
status from `wait()` and passing it through would make that arm live and remove the race in one line.

### L2 (low) — `run_with_stdin` no longer closes a piped stdin before waiting

`Child::wait_with_output` begins with `drop(self.stdin.take())` (verified in the local std source,
`library/std/src/process.rs:2473`), so the old `.output()` path handed the child EOF before waiting.
`wait_with_deadline` never touches `child.stdin`, and `child` lives until the function returns. A
caller passing `Stdio::piped()` to `run_with_stdin` — which the method's own doc invites, since it
takes an arbitrary `Stdio` — now gets a child blocked reading stdin, killed at ten seconds and
classified `TimedOut`. The live caller (`input_oracle.rs:541`) passes a `File`, so nothing is broken
today. `run_with` does take and drop its `sink` correctly; only `run_with_stdin` is exposed.

### L3 (low) — the new file's stated reason for its gate is false for three harnesses

`oracle_deadline.rs`'s module doc: "Gated on `REXX_CORPUS_GATE`, the switch every oracle-invoking
harness in this crate uses, for the reason they all give: an offline checkout is not asked to produce
an oracle." Measured: `corpus.rs`, `builtin_status.rs` and `state_builtin_oracle.rs` invoke the
oracle unconditionally and gate only the assertion — `builtin_status.rs` and `state_builtin_oracle.rs`
do not mention `REXX_CORPUS_GATE` at all, and my ungated run shows both the corpus report and
`builtin_status`'s table being produced. That unconditional shape is the "structural failures are red
in both modes" convention the global constraints require, so "every … harness" is wrong about the
files that matter most to the claim.

The consequence for the brief's structural-vs-verdict point is smaller than it looks, and the honest
statement was available: the deadline **mechanism** runs in both modes — those three harnesses put
every corpus and status program through `wait_with_deadline` on a plain `cargo test`, so a broken
poll loop or a misclassified normal exit reddens ungated — while only the **kill** is verified under
the gate. Worth saying that way rather than by an "every" that is checkable and false.

### L4 (low) — history framing in new doc comments, against `rust/CLAUDE.md`:96

The rule is "Comments say what the code does, not how it got there. No task numbers, no 'used to', no
account of what moved or shrank", with a carve-out: "a measurement justifying the *current* design
stays, because it is evidence for the contract."

No task numbers survive in the added lines (the `Task 1's own brief` string in `support/oracle.rs` is
pre-existing — line 378 of the `d926c334d` blob — not this diff's doing).

**The test I applied**, instance by instance: is the history framing removable at zero cost to the
content? If the same sentence carries the same facts in the present tense, the framing was
decoration and the rule bites. If removing it loses a fact, the carve-out protects it. In every
instance below the framing came out free, so the fix is a tense change and a few deleted words, not
a deleted paragraph — the substance in each is genuinely evidence for the contract and should stay.

1. **`Termination`'s doc** — "kept as a named type rather than the `exit_code: i32` sentinel this
   replaced: `output.status.code().unwrap_or(-1)` **gave** a signal death and a normal `exit -1` the
   identical representation, and a verdict function comparing exit codes **read** the former as a
   divergence … rather than the failure it **was**." The facts here are about `unwrap_or(-1)`, which
   is a permanently true statement about that expression, not about this repository's past: "A named
   type rather than an `i32` exit code: `status.code().unwrap_or(-1)` **gives** a signal death and a
   normal `exit -1` the identical representation, and a verdict function comparing exit codes
   **reads** the former as a divergence … rather than the failure it **is**." Nothing lost, and
   "this replaced" is gone.
2. **`wait_with_deadline`'s "Why this exists"** — "`Command::output` … blocks on `waitpid` with no
   deadline of its own -- so a program that never exits … **hung** whichever test called it forever,
   and **hung** `cargo test --workspace` with it." `Command::output` still behaves exactly that way;
   the past tense makes a timeless fact about a std function read as a changelog entry. Present
   tense costs nothing.
3. **`write_and_wait`'s doc** — "so this stays the simpler, blocking form for **the one caller
   left**." This is the instance I would hold hardest, because it breaks two rules at once and is
   the only one that can go *false*: "left" is the shrink narration, and "the one caller" counts a
   **mutable** set — the moment any later task calls `write_and_wait` a second time, the comment
   lies. `rust/CLAUDE.md`:87's own rationale ("no human will verify the number still matches") is
   about exactly this, and it is why this instance is a real defect where L5's fixed-arity counts
   are contestable. Naming the set fixes both: "for its caller in `tests/input_oracle.rs`".
4. **`oracle_deadline.rs`'s module doc, first clause** — "rather than blocking forever the way
   `Command::output` (**what `Oracle::run` called directly, with no deadline**) would have." The
   comparison against an undeadlined `Command::output` is the content; the parenthetical is the
   changelog. "rather than blocking forever the way an undeadlined `Command::output` does" keeps
   the whole point.

**The plan-versus-house-rule question, and why I do not think you have to rule on it.** The brief
asked that the before state be recorded ("the same program under the old path is what the task
records as the before state, run once by hand under `timeout -s KILL 10`") and did not say where.
`oracle_deadline.rs`'s module doc records it as "**The before state** … there is no surviving code
path that still calls `Command::output` without a deadline for this file to run as a counter-test:
… ran the full ten seconds and was killed by the external `timeout`: rc 137, both channels empty."

My judgment is that the **module doc is the right home** and there is no conflict to resolve,
because the two rules bite on different halves of that paragraph:

* The **measurement** — ten seconds, rc 137, both channels empty, under the wrapper `rust/CLAUDE.md`
  itself prescribes — is squarely inside the carve-out, and it is doing a job in this file that it
  does nowhere else. This test's own failure mode for a *deleted* mechanism is a hang, not a red
  (see "What would `oracle_deadline.rs` have done"), so it cannot carry its own negative control in
  code. That paragraph *is* the negative control. A reader who asks "what does this test's subject
  look like unbounded?" has no other source. The task report's copy is a different artifact doing a
  different job — an account of what the task did — so this is not duplication for its own sake.
* The **framing** around it — "The before state", "there is no surviving code path … for this file
  to run as a counter-test" — is about the repository and about what the implementer could not do,
  and it comes out free: "**The control, measured by hand rather than run here**, because no path in
  this crate reaches the oracle without a deadline: the same program under `( ulimit -v 1048576; …
  timeout -s KILL 10 … )` runs the full ten seconds and is killed by the external `timeout` — rc
  137, both channels empty. That is what this test's subject looks like unbounded." Every fact
  survives; nothing narrates a change.

So the brief's "record the before state" and `rust/CLAUDE.md`'s "not how it got there" are jointly
satisfiable, and I would ask for the reframing rather than a move or a deletion. If you disagree and
want the measurement out of the file, the report already holds it — but the file then loses its only
statement of what the mechanism is worth, which I would count as the worse trade.

**One weaker instance, same class, flagged rather than charged.** `expect_exit_code`'s doc says
"every existing differential caller in this tree assumes the oracle ran to completion". That is not
history, it is a phase-status claim about a mutable aggregate — `rust/CLAUDE.md`:95's "the one that
actually rots" — and Task 4 or Task 5 landing the first `did_not_finish` check falsifies it. The
sentence's next clause ("a `did_not_finish` check belongs before this call wherever a timeout or a
crash is a live possibility") states the property and cannot go stale, so the fix is to keep that
and drop the census.

**On the instrument.** These are hard-wrapped at ~72 columns, so a phrase-level `grep` over the diff
cannot see them; my own first pass found them only because I searched for short fragments
(`hung`, `replaced`, `called directly`, `would have`) rather than phrases, which is luck, not method.
For the record, the method that is reliable: collapse each contiguous comment block to one line for
the base blob and for the head, then `diff` the two collapsed files — that yields exactly the new and
changed comment prose, per file, with no wrap to defeat the search. I ran it over every file the diff
touches, not only the two named. It surfaced no history framing beyond the instances above.

### L5 (low, and possibly not a defect at all) — set cardinalities in new comments

`rust/CLAUDE.md`:87 forbids naming the size of a set. New comments carry "the two plain values",
"the two primitives", "two failing arms" (twice), "those two pipes". **But the base file is dense
with the same shape** — "three observable channels" (twice), "the two modes", "the two interpreters",
"both output pipes" — so enforcing this here would be applying a standard the file does not hold
itself to, and every instance names an immutable arity, not a mutable aggregate. Recorded so the
decision is yours rather than silently made either way.

**The one cardinality I do charge is in L4, not here**: `write_and_wait`'s "the one caller left"
counts a *mutable* set and can go false when a later task adds a second caller, which is the
difference between this bucket and that one. The distinction is the rule's own rationale — a count
no human will re-verify — and it is worth applying rather than treating every numeral alike.

### L6 (low) — the performance guard's "and says why" is not said

The global constraint: a task touching only `tests/`, `corpus/` or `docs/` runs no sitting **and says
why** — the release binary the axes measure is byte-identical, so the sitting would measure noise.
The diff does touch only `rust/crates/rexx-exec/tests/` (confirmed against the stat and against
`git diff --stat -- '*Cargo.toml' '*Cargo.lock'`, which is empty), and the report's header says
"Touches `tests/` only", but the reason is never stated and no sitting section appears. Also not
reported: `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1 cargo test --release --workspace`, which the
constraints ask every task to run. That one is currently vacuous — neither `REXX_PHASE_GATE` nor
`CLOSED_PHASES` exists anywhere in `rust/`, so the command is byte-identical to gate command 4, which
I ran — but the phrase "every task" will stop being vacuous at Task 4.

## Spec compliance, item by item

* **A deadline on all three of `Oracle::run`, `run_with`, `run_with_stdin`.** `run` delegates to
  `run_with` (`support/oracle.rs:252`), and both `run_with` and `run_with_stdin` now spawn and hand
  the `Child` to `wait_with_deadline`. Yes.
* **No new dependency; both routes checked; record which and why.** No `Cargo.toml`/`Cargo.lock`
  change in the diff. `wait-timeout 0.2.0` is genuinely in this machine's cache
  (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wait-timeout-0.2.0`, confirmed by
  listing it), so the alternative was real and the reasoning for declining it — that it replaces the
  poll loop and leaves the concurrent-reader half untouched — is correct and is recorded at
  `POLL_INTERVAL`'s definition, not only in the report. The polling interval is justified where it
  sits and the loop **sleeps** (`thread::sleep(POLL_INTERVAL)`), it does not busy-wait.
* **`CppOutcome` gains an explicit termination, not a sentinel.** Field `exit_code: i32` removed,
  `termination: Termination` added. Because the field was removed rather than kept alongside, the
  compiler proves the consumer sweep complete — a clean clippy over `--all-targets` is the check,
  not a grep.
* **Classification a pure function of `(Option<i32>, deadline_exceeded: bool)`, both failing arms
  unit-testable.** Exactly that signature, no `ExitStatus` anywhere in it; the reason given
  (`ExitStatus::from_raw` is Unix-only and needs a kernel wait status) is right.
* **`did_not_finish(&CppOutcome) -> bool`, tested on the status and never on the exit code.**
  Verified literally: the body is `!matches!(outcome.termination, Termination::Exited(_))` — no
  numeric comparison — and its test builds `CppOutcome`s by setting `termination` directly, never a
  code. Yes.
* **The hanging program reddens instead of hanging.** Reproduced: 10.01 s, exit 0, with the
  witnessing argument above.
* **Both classification arms get a test.** Present, plus a paired normal-exit case and a direct
  `did_not_finish` test.
* **Corpus unchanged at 106 of 106, oracle invocation count unchanged.** Both reproduced by me in
  all three suite runs. The source-level call-expression count per consumer file is identical at base
  and head (checked directly against the base blobs), and the tree's own `invocations()` assertions —
  `builtin_status.rs`'s committed literal `66` among them — passed under both gated runs.
* **The cited lines `oracle.rs:189`/`:214`.** Re-checked against `d926c334d`'s blob: both lines are
  `exit_code: output.status.code().unwrap_or(-1),`, in `run_with` and `run_with_stdin` respectively.
  The report says it re-read them and what it found, and the report is right.
* **No `unsafe`.** No added line contains the token; no dev-dependency added.
* **`tests/` only, no sitting.** True of the diff (see L6 for the missing sentence).

## Do the two classification unit tests add coverage?

Yes, and I checked the specific question rather than the general one. "Would they fail against the
old code" is trivially true (the function does not exist there), so the question that matters is
whether anything else pins the behaviour:

* `Termination::Signaled` is produced or asserted **nowhere else in the tree** — grepped. No test
  makes either interpreter die from a signal (`corpus/oracle-crashes.txt` is never run). Its unit
  test is the only witness of that arm, in either gate mode.
* `Termination::TimedOut` is otherwise witnessed only by `oracle_deadline.rs`, which is gated. Under
  `cargo test --release --workspace` — the mode most runs use — the unit test is the only thing that
  would catch the two `None` arms being swapped.
* The paired normal-exit test pins `(Some(n), true) => Exited(n)`, a combination no caller can
  currently produce (see L1). It makes the function total and guards the arm ordering; it is not
  redundant with the differential tests, which only ever exercise `(Some(n), false)`.

## What I could not check

* **Mutation testing of the classification and the poll loop.** The brief for this review forbade
  editing files, so I could not invert an arm or delete the `kill()` and watch which tests redden. The
  coverage argument above is from grepping every construction and assertion site, not from a red run.
* **Whether the 5 ms poll costs measurable suite time.** Arithmetic only: each oracle run now returns
  up to one interval late, so a few hundred invocations per gated suite is bounded by roughly a
  second, and I did not build the base to measure a before/after. Both gated runs completed with no
  behaviour change, which does not speak to duration.
* **The grandchild hang inside the harness itself (M1).** I demonstrated the mechanism against the
  real oracle through an equivalent shell capture, not through `Oracle::run`, because reproducing it
  in-harness needs a committed probe or an edit to a test file. The POSIX behaviour is not in doubt;
  the in-harness timing is untested.
* **The race in L1 firing in practice.** Not reproduced — it needs a program landing within one poll
  interval of the deadline. The code path is plain to read, but the probability under this project's
  workloads is unmeasured.
* **`memcap`'s absence.** Gate command 5 ran with `memcap` present, as the report says; the
  `ulimit -v` substitute path is unexercised here too.
* **Two things the collapsed comment pass surfaced that I have no basis to judge.** (i)
  `expect_exit_code`'s "every existing differential caller in this tree assumes the oracle ran to
  completion" is true today and is meant to stop being true — Task 4 and Task 5 are where
  `did_not_finish` gets its first callers. Whether the sentence is expected to be revised then, or
  is meant to read as a standing invariant, is a plan question and not mine. (ii) `POLL_INTERVAL`'s
  "so it **was** a real option and not ruled out by the no-network constraint" is past tense about a
  decision rather than about code; the brief explicitly asked for the route choice to be recorded,
  and I read design rationale as outside `rust/CLAUDE.md`:96's "what moved or shrank". I did not
  charge it, and note it so the omission is deliberate rather than missed.
* **Whether any Phase 5a probe will actually use `ADDRESS`.** M1's severity depends on it; I read no
  Phase 5a task briefs other than Task 1's.
