# Task 1 report — the run timeout, and what "did not finish" means

Base `d926c334d`, worktree `/home/moritz/dev/repos/ooRexx-rust-rewrite`. Touches `tests/` only.

## The cited lines

The brief cites `oracle.rs:189`/`:214`. Re-read against the file as it stood at the start of this
task, both citations were still exactly right: line 189 was `run_with`'s
`exit_code: output.status.code().unwrap_or(-1),` and line 214 was the identical line in
`run_with_stdin`. Both are gone now -- replaced by the `Termination` classification below -- but the
citations were accurate at the moment they mattered.

## The timeout route: polling, no new dependency

`cargo add -p rexx-exec --dry-run --offline wait-timeout@0.2.0` succeeds (exit 0) against
`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wait-timeout-0.2.0`, so the crate is a real
option and not ruled out by the no-network constraint. It was not taken. `wait-timeout`'s
`wait_timeout` only replaces the poll loop; this harness still needs its own threads reading
`stdout`/`stderr` concurrently while the wait is outstanding, for the same reason
`write_and_wait`'s doc already gives for the write side (a chatty program can fill a pipe's kernel
buffer and block until something drains it). A dependency that removes one loop and leaves the
harder half of the mechanism exactly as it is is not what "buys something" means, so the harness
polls `Child::try_wait` on a 5ms interval (`support::oracle::POLL_INTERVAL`) and calls `kill()` at
the 10-second deadline (`support::oracle::ORACLE_DEADLINE`, matching this project's own
`timeout -s KILL 10` convention by hand). This reasoning is recorded as a doc comment on
`POLL_INTERVAL` in `rust/crates/rexx-exec/tests/support/oracle.rs`, not only here.

## Shape of the classification function

```rust
pub enum Termination {
    Exited(i32),   // ran to completion
    Signaled,      // died from a signal nobody here sent
    TimedOut,      // wait_with_deadline killed it
}

pub fn classify_termination(code: Option<i32>, deadline_exceeded: bool) -> Termination {
    match (code, deadline_exceeded) {
        (Some(status), _) => Termination::Exited(status),
        (None, true) => Termination::TimedOut,
        (None, false) => Termination::Signaled,
    }
}
```

A pure function of the two primitives the brief names, not of a live `ExitStatus`: `ExitStatus` has
no portable public constructor (`ExitStatus::from_raw` is Unix-only and still needs a real wait
status from the kernel), so a signal-death test could not build one to call this on. Taking the two
primitives keeps both failing arms plain unit tests.

`CppOutcome` now carries `pub termination: Termination` in place of `pub exit_code: i32`. A new
`did_not_finish(&CppOutcome) -> bool` is `!matches!(termination, Termination::Exited(_))` -- tested
on the status, never on a synthesised exit code, since both failing arms carry `code: None` and
nothing about an exit code distinguishes them. A new `CppOutcome::expect_exit_code(&self) -> i32`
returns the code on `Exited` and panics naming the actual `Termination` otherwise; every existing
differential caller in the tree assumed a normal exit already (`cpp.exit_code`, no check), so this
converts that assumption from a silent `-1` (read as a divergence) into a loud structural failure
naming what happened, without requiring every caller to be rewritten around `did_not_finish` today.

**Four unit tests**, all in `support::oracle::tests`:
* `classify_termination_reports_a_normal_exit_regardless_of_the_deadline_flag` -- `Exited` wins
  whatever `deadline_exceeded` says (paired success case, `rust/CLAUDE.md`'s "pair a refusal with
  its adjacent success").
* `classify_termination_reports_timed_out_when_the_harness_killed_it` -- `(None, true)`.
* `classify_termination_reports_signaled_when_nothing_here_killed_it` -- `(None, false)`.
* `did_not_finish_is_true_for_both_failing_arms_and_false_for_a_normal_exit`.

The brief asks for one test per failing arm; both are there, plus the paired normal-exit case and a
direct test of `did_not_finish` itself.

## The deadline mechanism

`Oracle::run`, `run_with` and `run_with_stdin` all now spawn with `stdout`/`stderr` piped, hand the
`Child` to a new private `wait_with_deadline`, which spawns two reader threads (one per output pipe,
reading to end concurrently -- the same shape `wait_with_output` already uses, needed for the same
reason) and polls `try_wait` on the main thread until either the child exits or `ORACLE_DEADLINE`
passes, at which point it kills and reaps the child and reports `TimedOut`. `write_and_wait` (the
single-blocking-wait helper) stays, but only for `rexx-run` in `tests/input_oracle.rs`: the oracle's
own three methods needed the poll loop and `write_and_wait`'s single `wait_with_output` cannot
express one, while `rexx-run` run as a subprocess carries no deadline requirement in this task's
scope, so it keeps the simpler blocking form.

## Before state, run once by hand

```
mkdir -p <fresh dir>; printf 'do forever\nend\n' > hang.rex
time ( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
       timeout -s KILL 10 /home/moritz/dev/repos/ooRexx/build/bin/rexx <abs path>/hang.rex \
       > out.txt 2> err.txt )
```

Result: `real 0m10.002s`, shell reports `Killed`, exit 137, `out.txt` and `err.txt` both empty. This
is the state `Command::output()` (no deadline of its own) would reproduce forever with the
`timeout` wrapper removed -- there is no surviving in-repo code path that still calls it that way to
run as a counter-test, so this by-hand run is the only place the before state is recorded. It is
also recorded in `rust/crates/rexx-exec/tests/oracle_deadline.rs`'s module doc.

## Verification, runnable now

New file `rust/crates/rexx-exec/tests/oracle_deadline.rs`,
`a_program_that_never_finishes_reddens_at_the_deadline_instead_of_hanging`, gated on
`REXX_CORPUS_GATE` (it costs the full `ORACLE_DEADLINE` and needs the oracle). It runs the same
`do forever; end` through `Oracle::run` and asserts: `Termination::TimedOut`; `did_not_finish` true;
both channels empty; wall time `>= ORACLE_DEADLINE` and `< ORACLE_DEADLINE * 3` (a ceiling wide
enough to admit "returned at all" without pinning an exact schedule); one oracle invocation. In both
gated gate runs below it reported `finished in 10.00s` and passed -- the hang reddened at the
deadline rather than hanging the binary.

## The five gate commands, from `rust/`

1. `cargo fmt --all --check` -- exit `0`.
2. `cargo clippy --workspace --all-targets -- -D warnings` -- exit `0`, from a compile that recompiled
   `rexx-classes`, `rexx-core`, `rexx-parse` and `rexx-exec` fresh (not a reused warm result).
3. `cargo test --release --workspace` -- exit `0`. `oracle_deadline.rs`'s one test prints its
   `*** SKIPPED ***` line and reports `ok` in `0.00s` (ungated, matching every sibling oracle test's
   skip pattern), so this command does not pay the 10-second cost.
4. `REXX_CORPUS_GATE=1 cargo test --release --workspace` -- exit `0`. Corpus: `106 of 106 matching`.
   `oracle_deadline.rs`: `13 passed; 0 failed`, `finished in 10.00s`.
5. `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` (debug profile; `memcap` was
   present with `/sys/fs/cgroup` mounted) -- exit `0`. Corpus: `106 of 106 matching`.
   `oracle_deadline.rs`: `13 passed; 0 failed`, `finished in 10.00s`.

Each status above is the exit code of the command itself, captured directly (`echo "EXIT: $?"`
immediately after, or the background-task exit code for the one run that exceeded the interactive
timeout), never read off a piped `tail`.

## Oracle invocation count, before and after

**Structural check**: every call-site file that invokes `oracle.run`/`run_with`/`run_with_stdin`
has the identical number of such call expressions before (`d926c334d`) and after this task --
`corpus.rs`, `builtin_status.rs`, `parse_version_oracle.rs`, `state_builtin_oracle.rs`,
`ir_dual_oracle.rs` each still have exactly 1, `input_oracle.rs` still has 2. What this check could
not see: a change to how many times a call site executes inside its own loop (e.g. an early
`return` or an added iteration) -- it only counts source-level call expressions, not runtime calls.

**Runtime check**, which does see that: every existing assertion in the tree that compares
`oracle.invocations()` against a literal or a derived count still passed under both gated gate runs
(4 and 5 above, both exit 0, 0 failed) -- `parse_version_oracle.rs` (`== 1`),
`state_builtin_oracle.rs` (`== CASES.len()`), `input_oracle.rs` (`>= 15` and `== CASES.len()`, plus
a second `== 1` in its unreadable-console test), `ir_dual_oracle.rs`, and
`builtin_status.rs`'s own committed literal `run.oracle_invocations == 66`, which is the strongest
single data point: a number written down before this task still matches after it. My own new test
adds one more oracle invocation of its own (`assert_eq!(oracle.invocations(), 1)`), scoped to its
own `Oracle` instance and not counted against any other file's total.

## What this does not cover

The crate side of a gate-table row runs in-process (`Invocation::with_engine`) and cannot be killed
by anything short of the whole test process dying with it -- no mechanism landed here changes that.
The plan's answer is a human check: every probe program a later task commits is run by hand through
`rexx-run` under `timeout -s KILL 10` first, and the task says it did. `oracle_deadline.rs`'s module
doc states this explicitly so it is not implied.

## What I could not check

* Whether `wait-timeout` would behave differently under a real deadline than the hand-rolled poll
  loop -- not run, since it was not taken as the implementation; the offline-cache and `cargo add
  --dry-run` check only establishes that it *could* have been taken.
* A machine without `memcap`/cgroups available (gate 5's alternate `ulimit -v` path) -- this
  session had `memcap` present throughout, so the substitute path was not exercised.
* Whether the 5ms poll interval or the 3x deadline ceiling in `oracle_deadline.rs` hold up under a
  more heavily loaded machine than this session's -- both are generous relative to what was
  measured (a single `try_wait` and `kill`/`wait` pair completing well inside either bound), but
  neither was stress-tested under contention.

## Commits

Working-tree diff before commit: `rust/crates/rexx-exec/tests/support/oracle.rs` (the mechanism),
new `rust/crates/rexx-exec/tests/oracle_deadline.rs` (the verification), and six existing oracle
consumers (`builtin_status.rs`, `corpus.rs`, `input_oracle.rs`, `ir_dual_oracle.rs`,
`parse_version_oracle.rs`, `state_builtin_oracle.rs`) updated to the new `CppOutcome` shape. Commit
SHAs are in the message back to the dispatcher.

## Fix round 1

Base for this round `1212d7f80` (the reviewed head). Rulings applied from
`task-1-fixround-1-brief.md`; evidence and exact-fix suggestions from `task-1-review.md`.

### M1 — the deadline bounded the wait, not the read

Fixed, using the read-bound-via-channel route the ruling names. `wait_with_deadline` in
`support/oracle.rs` no longer joins its two reader threads: each sends its buffer down an
`mpsc::channel` instead, and the function gives that channel its own budget of `ORACLE_DEADLINE`,
measured fresh from the moment the process-wait loop above ends rather than chained off whatever
time that loop had left -- a fast-exiting direct child with a slow grandchild (the demonstrated
shape) would otherwise get almost none. If either channel does not answer within that budget, both
transcripts are discarded and the run is forced to `Termination::TimedOut` regardless of what the
direct child's own exit status was, since an incomplete transcript is not something any caller
should compare bytes against. The residual is documented at the same doc comment, honestly: a
grandchild that still holds the descriptor keeps running, and the reader thread stays parked in
`read` for as long as that process does (or forever) -- a leaked background process and a leaked
background thread in the test binary, not a hung suite. `Child` has no handle on a process it never
spawned, so there is nothing further here to kill.

**Test, in `oracle_deadline.rs`**:
`a_background_process_holding_the_pipe_open_does_not_hang_the_run`, gated the same way as the
existing test. `address system 'sleep 30 &'` then `say 'done'`: the direct `rexx` process exits at
once, the backgrounded `sleep` inherits and holds the stdout descriptor. Asserts the call returns in
under 20 seconds (the bound that would fail without this fix -- an unconditional `join()` blocks for
the backgrounded process's own lifetime, 30 seconds here, unboundedly for a longer-lived one) and
that the result is `Termination::TimedOut`. **Against the code as it stood before this fix**, this
program's `Oracle::run` call would not have returned before the backgrounded `sleep` exited: the old
`join()` reads to EOF, and EOF does not arrive until every write end of the pipe closes, the
grandchild's included. Not reproduced by literally reverting and re-running (out of scope to redo the
mechanism twice), but the mechanism removed -- an unconditional `.join()` on both reader threads --
is exactly what the review measured against the real oracle outside this harness (12011 ms for a
`sleep 12 &`, scaling with the backgrounded duration).

### M2 — `input_oracle.rs`'s `diffs` read two non-finishes as agreement

Fixed. `command_line_arguments_and_the_console_agree_with_the_oracle`'s loop now asserts
`!did_not_finish(&rust) && !did_not_finish(&cpp)` before calling `diffs`, unconditionally (this
whole test only runs under the gate already, so "red in both gate modes" is satisfied by the check
firing regardless of any further mode split within it). A run that does not finish is now a
structural failure naming the case, its `why`, and both `Termination`s, rather than a silent pass
when both sides happen to fail the same way.

### M3 — a timed-out program reddened without naming itself

Fixed at the three callers that hold a name, per the ruling, not by giving `expect_exit_code` a
context argument:

* `corpus.rs`'s `check_case` checks `did_not_finish(&cpp)` before calling `descriptor_diffs_with`
  (which would otherwise reach `expect_exit_code`'s path-less panic) and returns a `Mismatch` naming
  `rel_path` and the `Termination`, shaped like every other row this file's report prints.
* `builtin_status.rs`'s `measure` asserts `!did_not_finish(&cpp)` before `descriptor_diffs`, naming
  `name` in the panic.
* `state_builtin_oracle.rs`'s `sweep_one` checks `did_not_finish(&cpp)` before `descriptor_diffs` and
  returns a report line naming `case.name`, the same `Option<String>` shape its caller already
  expects for any other divergence.

None of these paths fire against the current 106-program corpus or any existing probe set -- no
program in the committed corpus times out -- so this is a code-review-verified fix rather than one
with its own red/green demonstration; the ruling for M2 and M3 did not ask for one the way M1's did.

### L1 — the deadline branch discarded a real exit status

Fixed. The kill branch now binds `child.wait()`'s own result (`let status = child.wait().ok();`) and
passes `status.and_then(|s| s.code())` into `classify_termination` instead of hardcoding `None`. A
process that exits in the gap between the last `try_wait` and the deadline check is now classified
`Exited(n)` from its real status rather than `TimedOut` from an assumed one. The paired unit test
`classify_termination_reports_a_normal_exit_regardless_of_the_deadline_flag`
(`classify_termination(Some(1), true) == Exited(1)`) already existed from Task 1's first pass and
this fix is what makes that combination reachable from `wait_with_deadline`'s own call site rather
than only from a hand-constructed pair of arguments.

### L2 — `run_with_stdin` did not close a piped stdin before waiting

Fixed. `run_with_stdin` now does `drop(child.stdin.take())` immediately after spawning, before
calling `wait_with_deadline` -- a no-op for the `File`/`Stdio::null()` shapes its one live caller
passes (neither hands back a `child.stdin` to take), and the fix for `Stdio::piped()`, which the
method's signature also accepts and which would otherwise leave the child waiting on input that
never arrives.

### L3 — the new file's stated gate reason was false for three harnesses

Fixed, using the reviewer's own replacement framing. `oracle_deadline.rs`'s "# The gate" section no
longer claims every oracle-invoking harness in the crate shares its reason for gating. It now says
`corpus.rs`, `builtin_status.rs` and `state_builtin_oracle.rs` already put every program through
`wait_with_deadline` on a plain `cargo test`, gating only their own assertion, so the deadline
mechanism itself would already redden ungated in those three; what only this file checks, and only
under the gate, is that the kill itself fires, which costs the full deadline to prove.

### L4 — history framing in new doc comments

Fixed, all four instances the review named, tense-and-word changes only, no content lost:

1. `Termination`'s doc: "the `exit_code: i32` sentinel this replaced" / "gave ... read ... was" →
   "rather than an `i32` exit code" / "gives ... reads ... is".
2. `wait_with_deadline`'s "Why this exists": "hung whichever test called it forever, and hung
   `cargo test --workspace`" → "hangs whichever test calls it, and hangs `cargo test --workspace`".
3. `write_and_wait`'s doc: "the one caller left" (a mutable-set count, the instance the review held
   hardest) → "its caller in `tests/input_oracle.rs`".
4. `oracle_deadline.rs`'s module doc: "what `Oracle::run` called directly, with no deadline) would
   have" → "the way an undeadlined `Command::output` does".

The before-state paragraph the review's L4 flagged as a plan-versus-house-rule question is resolved
the way the ruling picked: moved out of the module doc entirely (L4's second bullet superseded the
review's own "reframe rather than move" recommendation, so the ruling governs). The doc now says only
what the test proves and what it does not, plus a pointer to this report for the control; the
measurement itself -- `rc 137`, ten seconds, both channels empty -- stays in this report's "Before
state" section above, unedited.

`expect_exit_code`'s "every existing differential caller in this tree assumes the oracle ran to
completion" -- the instance the review flagged but did not charge -- is left as it was; the ruling's
nine findings did not include it.

### L5 — set cardinalities in new comments

**Ruled no change**, and none made, except the one instance L4 already counted (`write_and_wait`'s
"the one caller left", above). New comments added this round keep the same shape the base file
already uses freely for fixed-arity counts ("both channels", "either side", "the two channels").

### L6 — the performance guard's "and says why" was not said

Stated here rather than in code, per the ruling. This task's diff touches only
`rust/crates/rexx-exec/tests/` in both rounds (`git diff --stat -- rust/Cargo.toml Cargo.toml` and
`-- rust/crates/*/Cargo.toml` are empty against both `d926c334d` and `1212d7f80`), so the release
binary the performance axes measure is byte-identical to what it was before this task started, and a
sitting would measure noise rather than the change -- no sitting was run for either round, on that
basis.

`REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1 cargo test --release --workspace`: exit `0`, corpus `106 of
106 matching`, 94 `test result: ok` blocks, `0 failed`, identical in substance to gate command 4's
run (`diff` of the two logs with per-test timings stripped shows only test-interleaving-order
differences, the same 94 blocks and the same pass/fail counts) -- expected, since neither
`REXX_PHASE_GATE` nor `CLOSED_PHASES` exists anywhere in `rust/` yet, so this command is currently
byte-identical in effect to gate command 4. That stops being true at Task 4.

### Verification for this round

All five gate commands, plus L6's phase-gate command, run fresh after the fixes above:

1. `cargo fmt --all --check` -- exit `0`.
2. `cargo clippy --workspace --all-targets -- -D warnings` -- exit `0`.
3. `cargo test --release --workspace` -- exit `0`; `oracle_deadline.rs` prints `*** SKIPPED ***` for
   both gated tests and reports `14 passed`, `finished in 0.00s`.
4. `REXX_CORPUS_GATE=1 cargo test --release --workspace` -- exit `0`. Corpus `106 of 106 matching`.
   `oracle_deadline.rs`: `14 passed; 0 failed`, `finished in 10.01s` (the two ~10-second tests run
   concurrently in the same binary under the default test harness).
5. `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` (`memcap` present) -- exit
   `0`. Corpus `106 of 106 matching`. `oracle_deadline.rs`: `14 passed; 0 failed`,
   `finished in 10.01s`.
6. `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1 cargo test --release --workspace` -- exit `0`, corpus `106
   of 106 matching`, substance identical to (4).

**Per-test wall time**, since gate command 4's binary-level `10.01s` alone does not distinguish "two
tests each bounded at ~10s, run concurrently" from some other split:
`REXX_CORPUS_GATE=1 cargo test --release --test oracle_deadline -- --test-threads=1 --nocapture`
(sequential, so each test's cost adds rather than overlaps) reports `14 passed`,
`finished in 20.02s` -- the twelve unit tests cost negligible time, so this is ~10.01s for
`a_program_that_never_finishes_reddens_at_the_deadline_instead_of_hanging` and ~10.01s for
`a_background_process_holding_the_pipe_open_does_not_hang_the_run`, each independently witnessing its
own deadline.

### What I could not close

* **M1's negative demonstration is not a red/green pair against this harness's own commit history.**
  I did not revert `wait_with_deadline` to its pre-fix `.join()` form and re-run the new test to watch
  it fail (and, per the mechanism, hang rather than fail -- the same property the original
  `oracle_deadline.rs` test's own doc already names for a deleted deadline). The review's own
  independent measurement against the real oracle (12011 ms for `sleep 12 &`, scaling with duration)
  is the evidence that the removed mechanism -- an unconditional `.join()` -- has the defect; I did
  not re-derive that evidence a second time inside this harness.
* **The read-bound's own race**, symmetric to L1's but on the read side: a channel send that lands in
  the same instant `recv_timeout`'s deadline expires is unexercised, the same way L1's original race
  was unmeasured in the first review. Not reproduced.
* **`memcap`'s absence** (gate 5's `ulimit -v` substitute path) -- `memcap` was present throughout
  this round too.
* **Mutation testing of the new read-bound or the L1 status-binding fix** -- not run; the same
  constraint the original review noted (no file edits during review) does not apply to me, but no
  mutation pass was requested for this round and none was run.

## Fix round 2

Base for this round `8017f027a` (round 1's head, re-reviewed). Rulings applied from
`task-1-fixround-2-brief.md`; evidence from `task-1-rereview.md`. Verdict on the prior round:
**REWORK**, eight of nine round-1 rulings closed, M3 open, seven new findings (N1-N7), two high.

### M3 / N1 / N2 -- one fix: a structural failure gets its own unconditional channel

The round-1 fix for M3 routed a non-finish through each caller's **verdict** channel -- the exact
channel every one of them gates or set-matches -- which is what converted a failure that was red in
both modes into one that was green in report mode at `corpus.rs` and green in every mode at
`state_builtin_oracle.rs`. Fixed at all three named sites, plus the two "also in scope" sites the
ruling named, taking `builtin_status.rs`'s original approach (an unconditional `assert!` naming the
program) as the standard everywhere:

* **`state_builtin_oracle.rs`'s `sweep_one`** no longer returns the non-finish as `Some(String)` --
  the exact channel `every_state_builtin_case_matches_the_oracle_except_the_declared_gaps` matches
  against `DECLARED_GAPS`. It now asserts unconditionally, naming `case.name`, before that channel is
  ever reached.
* **`corpus.rs`** keeps the `Mismatch` so the report line still renders (the "report vs strict" split
  the file is built around), but `Mismatch` gained a `structural: bool` field, `true` only for the
  `did_not_finish` branch. `corpus_differential` now collects the structural subset from
  `mismatches` and asserts it empty **unconditionally, before** the existing gated
  `!gate || mismatches.is_empty()` assertion -- so report mode (a plain `cargo test`, the mode most
  runs use) reddens on a non-finish exactly like the gated mode already did.
* **`parse_version_oracle.rs`** and **`ir_dual_oracle.rs`** (the ruling's "also in scope": they still
  reached `expect_exit_code`'s path-less panic, which is red everywhere but was the third of three
  different treatments of one event) now each assert unconditionally too, naming the program --
  `parse_version_oracle.rs` names the fixed probe directly (the file runs exactly one), and
  `ir_dual_oracle.rs` threads a `label` (the stanza file's name plus a running stanza count) into
  `render_oracle` so its assertion names which stanza.

**Demonstrated, not merely argued**, per the round's own requirement. For each of N1 and N2: a
program that exits quickly in-process but leaves the oracle non-finishing --
`address system 'sleep 15 &'` then `say 'done'` (`ADDRESS` refuses loudly in 0.003s in-process,
measured directly against `target/release/rexx-run`, confirmed `ADDRESS is not implemented` --
`do forever; end` was rejected for this because `check_case`/`sweep_one` both run the crate's
in-process side first, with no timeout of its own, and would have hung the demonstration itself, not
merely the mechanism under test).

* **N1.** A `git worktree add <temp-dir> 8017f027a` (the pre-round-2 commit), with a temporary
  `#[test]` appended to that copy's `corpus.rs` calling `check_case` directly against a scratch
  directory holding the probe above, then reproducing that commit's actual
  `!gate || mismatches.is_empty()` assertion with `gate = false`. Ran under
  `CARGO_TARGET_DIR` pointed at this tree's own `target/` (so only the touched test binary rebuilds):
  **passes, `10.01s`** -- silently green in report mode, before the fix. The identical probe and
  assertion shape (now using the new `structural` field) added temporarily to the current tree's
  `corpus.rs` and run the same way: **panics, `10.01s`**,
  `structural=["hang.rex"]` -- red, after the fix. Both temporary tests removed before committing;
  the worktree removed (`git worktree remove --force`); `git diff --stat` confirms only the intended
  fix remains in `corpus.rs`.
* **N2.** Same worktree, a temporary `#[test]` calling `sweep_one` directly with a synthetic
  `Case { name: "fixround2_demo_gap", source: <the probe above> }` and reproducing
  `every_state_builtin_case_matches_the_oracle_except_the_declared_gaps`'s own set-equality logic
  against a **local** synthetic `declared_gaps` set containing only that one name (never touching the
  real `CASES`/`DECLARED_GAPS`, since that second array also feeds
  `every_declared_gap_names_a_case_and_fails_loudly`'s in-process, undeadlined run -- adding a
  hanging source there for real would have risked exactly the hang the first demonstration avoided).
  Before the fix: **passes, `10.01s`** -- the non-finish is absorbed as the declared gap it shares a
  name with. After the fix, calling `sweep_one` directly with the same case (no set-matching needed,
  since the fixed function now asserts unconditionally inside itself): **panics, `10.01s`**, naming
  `fixround2_demo_gap` and stating "a structural failure ... never a declared gap". Both temporary
  tests removed before committing.

### N3 -- L3's replacement text was still false for two of the three files

Fixed, dropping the "gating only their own assertion" clause: `builtin_status.rs` and
`state_builtin_oracle.rs` gate nothing at all, not merely their own assertion. The conclusion
("the mechanism would already redden ungated") is unchanged and, if anything, stronger without the
false clause.

### N4 -- the mutable-set count L4 removed reappeared nine lines away

Fixed. `run_with_stdin`'s comment named "this method's only caller passes today" (a count with a
phase-status "today" attached); it now names the caller directly:
`input_oracle.rs`'s `an_unreadable_console_is_end_of_input`.

### N5 -- history framing in the new test, plus a dangling pointer

Fixed both. "The bound that would fail without the fix this test proves" -> "The bound that would
fail if the read were joined unconditionally". The module doc's pointer at "the task's own report"
(unnamed, unfollowable from the source) is dropped rather than named -- the report file lives under
`.superpowers/`, which is not tracked, so a permanent source comment pointing at it would point at
something a different checkout does not have.

### N6 -- `ORACLE_DEADLINE`'s doc still read as the bound on one invocation

Fixed. The constant's doc now states both budgets it names (the wait, and the read that follows it,
independently) and says a run can cost up to twice it, reconciling that against the `timeout -s KILL
10` it grounds itself in: the by-hand figure bounds the process, not a descriptor the process hands to
one of its own, which is exactly the gap the doubled bound closes.

### N7 -- two diagnostics quietly lost with the join, both fixed

* A child that dies from a genuine signal while a descendant holds the pipe no longer gets
  relabelled `TimedOut`: the read-timeout arm now only synthesises `code = None,
  deadline_exceeded = true` when the process-wait loop's own classification was `Exited` --  an
  existing `Signaled` (or `TimedOut`) is left alone rather than overwritten.
* `recv_timeout`'s `Timeout` and `Disconnected` are now matched separately. A `Disconnected` --
  meaning a reader thread panicked before it could send -- panics here too, naming the path and
  which pipe, restoring the diagnostic the pre-channel `.join()`-based code gave instead of folding
  it into a silent `TimedOut`.

### Verification for this round

All five gate commands plus the phase-gate command, fresh after every fix above:

1. `cargo fmt --all --check` -- exit `0`.
2. `cargo clippy --workspace --all-targets -- -D warnings` -- exit `0`.
3. `cargo test --release --workspace` -- exit `0`.
4. `REXX_CORPUS_GATE=1 cargo test --release --workspace` -- exit `0`. Corpus `106 of 106 matching`.
   `oracle_deadline.rs`: `14 passed; 0 failed`, `finished in 10.01s` (its two non-unit tests -- the
   original hang test and round 1's M1 grandchild test -- run concurrently in one binary, both still
   bounded at `ORACLE_DEADLINE`, unaffected by this round's changes).
5. `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` (`memcap` present) -- exit
   `0`. Corpus `106 of 106 matching`. `oracle_deadline.rs`: `14 passed; 0 failed`,
   `finished in 10.01s`.
6. `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1 cargo test --release --workspace` -- exit `0`. Corpus `106
   of 106 matching`. Still vacuous, as L6 recorded: neither `REXX_PHASE_GATE` nor `CLOSED_PHASES`
   exists in `rust/` yet.

`corpus_differential`, `the_status_file_matches_a_live_differential_run`,
`every_state_builtin_case_matches_the_oracle_except_the_declared_gaps`,
`parse_version_still_answers_what_the_oracle_answers` and
`every_recorded_expectation_is_still_what_the_oracle_produces` all still pass under the gate --
the new unconditional checks in each do not fire against anything in the real corpus, probe or
stanza sets, matching round 1's own note that none of this fires against current content.

Collapsed-comment-block diff (`8017f027a` vs this round's head) run over every file this round
touched, not only the ones named: no new task numbers, "used to", or other history framing beyond
the instances N3-N5 name and fix above.

**Note on "all three `oracle_deadline` tests"**: the brief's verification section asks for three;
`oracle_deadline.rs` has two non-unit tests (the original hang test, and round 1's grandchild test).
N1 and N2's demonstrations were built as temporary tests in `corpus.rs` and `state_builtin_oracle.rs`
and explicitly removed before committing, per "a temporary program plus a run you report and then
remove is fine" -- so no third test was added to `oracle_deadline.rs` this round. Flagging the
mismatch rather than guessing a third test was wanted somewhere.

### What I could not close

* **The read bound's own race** (a `send` landing in the same instant `recv_timeout` expires,
  symmetric to L1's original race) is still unexercised -- not reproduced this round either.
* **`memcap`'s absence** -- present throughout this round, so the `ulimit -v` substitute path remains
  unexercised.
* **Whether `progress.md` needs the moved before-state measurement** -- the re-review flagged this as
  a controller call, not an implementer one; unchanged this round.
* **Mutation testing** of the `structural` field, the unconditional asserts, or the N7 classification
  fixes -- not run; not requested this round.

## Fix round 3

Base for this round `63a49c9b9` (round 2's head, re-reviewed). Rulings from
`task-1-fixround-3-brief.md`; evidence and replacement text from `task-1-rereview-2.md`. Verdict on
round 2: **8 of 8 ruled findings CLOSED** (including M3+N1+N2 as one, traced individually at all five
sites); REWORK only for four new low defects, D1-D4.

### D1 -- a new comment named the size of the corpus, with a false example inside it

Fixed. `corpus_differential`'s new comment named `"106 of 106 matching"` as the headline a
non-finish must not be allowed to produce -- a count, in the one file whose own `SUBSET_FILES` doc
already says "a number a reader might eyeball is not a check", and the wrong count besides (a
non-finish is never added to `matched`, so such a run would print `105 of 106`, not `106 of 106`).
Replaced with "a fully-matching corpus" -- naming the set's property rather than its size, which
removes the false example along with the count, since the sentence no longer names any specific
headline for either a true or a false run to match against.

### D2 -- a new comment was history about the shape this very round was fixing, and my own pass missed it

Fixed: deleted "The pre-channel code named the path in exactly this case
(`.join().unwrap_or_else(|_| panic!(...))`)" from the `Disconnected`-handling comment in
`support/oracle.rs`. The surrounding two sentences carry the whole argument on their own.

**My round-2 report's claim was wrong, and here is why the pass missed it.** I ran the
collapsed-comment diff correctly -- the sentence the reviewer found was present in that round's diff
output, which I did read. What I did with that output was the gap: I scanned it with a keyword
`grep` built from the specific phrasings earlier rounds had already named ("Task 1", "used to", "no
longer", "any more", "this replaced"), rather than reading every new sentence on its own merits
against the rule's actual test ("strike the framing -- does the sentence still say the same thing
about the code as it is?"). "The pre-channel code named the path in exactly this case" uses none of
those words, so it passed a keyword scan clean while failing the test the words are only ever a
shorthand for. A pattern list built from prior findings can only catch a recurrence of an already-seen
shape; it cannot catch a new one wearing different words. This round's own passes (see below) read
every diff line's prose rather than filtering it through an accumulated keyword list, for exactly
that reason.

### D3 -- the stanza label used a running total, and the more decisive half was that it was not reproducible

Fixed with the ruled shape: a per-file counter (`in_file`) reset inside `datadriven::walk`'s outer
closure, incremented per stanza inside `file.run`'s inner closure, and used for the label; `checked`
is untouched and keeps counting the running total `oracle.invocations() == checked` needs. Did not
reach for `TestCase::line_number` (confirmed private in `datadriven` 0.9.0, as the ruling said).

### D4 -- "everywhere" was falsified by one remaining site, and the comment claimed it too

Fixed both halves. `ir_dual_oracle.rs`'s comment no longer claims "the shape every oracle-invoking
harness in this crate uses for the same event" -- reworded to describe only what this call site does,
which cannot go false as other call sites change. The sixth site,
`input_oracle.rs`'s `an_unreadable_console_is_end_of_input` (`oracle.run_with_stdin` at `:556`,
`expect_exit_code` at what was `:567`), now asserts `!did_not_finish(&cpp)` first, naming the test,
the same shape as its sibling call site in the same file (`:449`, M2's fix). Per the ruling, the
commit message that said "everywhere" is not amended; this round's own commit and this report are
where the correction lives.

### Verification for this round

All five gate commands plus the phase-gate command, fresh after every fix above:

1. `cargo fmt --all --check` -- exit `0`.
2. `cargo clippy --workspace --all-targets -- -D warnings` -- exit `0`.
3. `cargo test --release --workspace` -- exit `0`.
4. `REXX_CORPUS_GATE=1 cargo test --release --workspace` -- exit `0`. Corpus `106 of 106 matching`.
   `oracle_deadline.rs`: `14 passed; 0 failed`, `finished in 10.01s` -- both of its two non-unit tests
   (there are two, not three; the brief's own correction this round confirmed it).
5. `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` (`memcap` present) -- exit
   `0`. Corpus `106 of 106 matching`. `oracle_deadline.rs`: `14 passed; 0 failed`,
   `finished in 10.01s`.
6. `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1 cargo test --release --workspace` -- exit `0`. Corpus `106
   of 106 matching`. Still vacuous, as recorded in every prior round.

Collapsed-comment-block diff run over **every file the whole task has touched** (`oracle.rs`,
`oracle_deadline.rs`, `corpus.rs`, `state_builtin_oracle.rs`, `parse_version_oracle.rs`,
`ir_dual_oracle.rs`, `input_oracle.rs`, `builtin_status.rs`), against the original base `d926c334d`,
not just this round's diff -- per the brief's instruction that D2 shows the pass can miss when it is
scoped too narrowly. This time each file's new content was read in full rather than filtered through
a keyword list first (see D2 above for why that distinction matters); a keyword pass was run
afterward only as a second check, not the only one. One pre-existing hit survived the read
(`oracle_deadline.rs`'s "it would no longer be true that this test..." ceiling-comment, a logical
hypothetical already reviewed and left alone in rounds 1 and 2); nothing else in any of the eight
files carries history framing, a task number, or a set cardinality beyond the fixed-arity kind L5
already ruled out of scope.

### What I could not close

Unchanged from round 2's list: the read bound's own send/recv_timeout race; `memcap`'s absence (still
present throughout); whether `progress.md` needs the moved before-state (still the controller's call);
mutation testing (still not requested).
