# Task 1, fix round 1 — re-review

Scope `7e3f3cb41..8017f027a` (`b0c32c924` the mechanism, `8017f027a` the call sites). Read-only;
nothing in the tree was edited. The standard is `task-1-fixround-1-brief.md`'s rulings; the prior
round's approved work is not reopened.

## Verdict

**REWORK.** Eight of the nine rulings are closed, several of them well — M1's read bound is real,
its test discriminates, and L1/L2/L3/L4/L5/L6 are each done as ruled. M3 is not closed: the name the
ruling asked for is there, but the change that added it **converted a structural failure that was
red in both gate modes into one that is green** — green in report mode at `corpus.rs`, and green in
*every* mode at `state_builtin_oracle.rs`. That is the one direction the global constraints forbid
("Structural failures are always red, in both modes, in every phase … an oracle run that did not
finish. Nothing about the implementation can turn one of these green"), and it is the same
false-green direction M2 existed to close, reintroduced two files away in the same commit.

## What I ran

| command | result |
| --- | --- |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `REXX_CORPUS_GATE=1 cargo test --release --workspace` | exit 0; corpus `106 of 106 matching`; 94 `test result: ok` blocks; `oracle_deadline` `14 passed`, `finished in 10.01s` |
| `REXX_CORPUS_GATE=1 cargo test --release --test oracle_deadline -- --test-threads=1 --nocapture` | exit 0; `14 passed`, `finished in 20.01s` |
| collapsed-comment diff, base vs head, all six files | see L4/L5 below |
| `rustc`-built probe of `recv_timeout(Duration::ZERO)` on a queued value | `Ok(42)` — see "the shared clock is safe" below |

The report's numbers reproduce, including the `20.02s`/`20.01s` sequential split that witnesses the
new test's own ~10 s independently of the existing one's.

## The ruled findings

* **M1 — CLOSED.** The read is bounded, by the channel route the ruling named
  (`support/oracle.rs:495-517`). The residual is documented at the same doc comment and is the right
  residual (a leaked grandchild and a parked thread, not a hung suite). The test exists, is gated the
  same way, and discriminates — details under "M1's new mechanism, checked hard".
* **M2 — CLOSED.** `input_oracle.rs:449-457` asserts `!did_not_finish` on **both** sides before the
  only `diffs` call in the file (`:459`; there is no second call site). Two `Signaled` runs can no
  longer read as agreement. The file skips wholesale without `REXX_CORPUS_GATE` (`:415`), so the
  assertion is reachable only under the gate — pre-existing to this file and not something the ruled
  fix could change; noted, not charged.
* **M3 — OPEN.** All three callers do name their program (`corpus.rs:359` names `rel_path`,
  `builtin_status.rs:236` names `name`, `state_builtin_oracle.rs:489` names `case.name`), and the
  naming reaches the screen at `builtin_status` (panic) and at `corpus` (report line, printed in both
  modes). But two of the three replaced a panic with a value the caller's existing accounting
  absorbs, and the redness the review explicitly credited the old code with is gone. See N1 and N2.
* **L1 — CLOSED.** `support/oracle.rs:481-483` binds `child.wait()`'s status and passes
  `status.and_then(|s| s.code())`. The kill path still classifies `TimedOut` (proved by the existing
  test passing at 10.01 s, whose `Termination::TimedOut` assertion would fail if the SIGKILL status
  leaked through as an `Exited`), and `classify_termination`'s `(Some(status), _)` arm is now
  reachable from its only caller.
* **L2 — CLOSED.** `drop(child.stdin.take())` at `support/oracle.rs:327`, before
  `wait_with_deadline`. Correctly a no-op for the live `File` caller.
* **L3 — CLOSED**, with one new false clause in the replacement text — see N3.
* **L4 — CLOSED** for all four ruled instances, verified by the collapsed-comment method rather than
  by the report's claim: `Termination`'s doc is now "rather than an `i32` exit code … gives … reads …
  is"; `wait_with_deadline`'s "Why this exists" is present tense; `write_and_wait` says "for its
  caller in `tests/input_oracle.rs`"; `oracle_deadline.rs`'s parenthetical is gone and the before
  state is out of the module doc, with the measurement intact in the report's "Before state, run once
  by hand" (`task-1-report.md`:83). `POLL_INTERVAL`'s route rationale is unchanged, as ruled. Two new
  instances of the same class arrived with the new prose — N4 and N5.
* **L5 — CLOSED (no change made).** Verified rather than assumed: the collapsed-comment diff shows
  `classify_termination`'s doc block (the one carrying "the two plain values" / "the two primitives"
  / "both failing arms") is untouched, and "those two pipes" survives inside the rewritten
  `wait_with_deadline` block. A line-wise `grep` for those phrases returns 0 or 1 because they are
  hard-wrapped; the collapsed diff is what settles it.
* **L6 — CLOSED.** The reason is stated in the report (tests-only diff, byte-identical release
  binary, so a sitting would measure noise) and both halves check out independently:
  `git diff --stat 7e3f3cb41..8017f027a -- . ':!rust/crates/rexx-exec/tests'` is empty, as is the
  `Cargo.toml`/`Cargo.lock` diff against `d926c334d`. The phase-gate command's vacuity claim also
  checks out — `grep -rn 'REXX_PHASE_GATE\|CLOSED_PHASES' rust/` finds nothing.

**Tally: 8 CLOSED, 1 OPEN (M3).**

## M1's new mechanism, checked hard

* **Two budgets, and the worst case.** The wait keeps `ORACLE_DEADLINE`; the read gets a fresh
  `ORACLE_DEADLINE` measured from after the wait (`:495`). The two channels share one clock
  (`:497`, `:499`, both derived from `read_deadline`), so the pair is bounded once. Worst case per
  oracle invocation is therefore **≈ 2 × ORACLE_DEADLINE ≈ 20 s**, reached by a program that neither
  exits nor closes its pipes (`address system 'sleep 3600 &'` followed by `do forever`). See N6 for
  where the docs still read as though 10 s bounded an invocation.
* **Can a merely-slow run now be forced to `TimedOut`?** In practice no. The reader threads run from
  the moment of spawn, so after the child exits `read_to_end` returns at EOF; to miss a 10 s
  recv budget the pipe must still be held by a surviving descendant, or the thread must be starved
  for ten seconds. A legitimately long program is still bounded by the *wait*, exactly as before this
  round. The failure mode if it ever did fire is worth naming because N2/N1 compound it: a spurious
  read timeout is a non-finish, which is now silently absorbed at two of the three call sites.
* **The shared clock is safe.** The second `recv_timeout` can be handed `Duration::ZERO` when the
  first consumed the whole budget. I checked that this does not discard an already-delivered
  transcript: `recv_timeout(Duration::ZERO)` on a channel with a queued value returns `Ok`, measured
  with a standalone `rustc` build against this toolchain. So a slow stdout plus a long-since-arrived
  stderr classifies as the finish it is.
* **Truncation cannot reach a run that finished.** The timeout arm (`:509-514`) sets `code = None`
  and `deadline_exceeded = true` alongside the two empty buffers, so the outcome is `TimedOut` by
  construction — there is no path producing `Exited(n)` with an emptied transcript. Every consumer
  that compares bytes either checks `did_not_finish` first or reaches `expect_exit_code`, which
  panics on a non-finish (`descriptor_diffs_with` calls it unconditionally, `support/oracle.rs`). The
  pre-existing partial-buffer path (`let _ = …read_to_end(&mut buf)` ignores a mid-stream error and
  sends what it has) is unchanged by this round and is the one truncation shape that could reach an
  `Exited` run; it did so before the round too.
* **L1 and M1 interact in the ruled direction, not against each other.** The read timeout overwrites
  `code`, so a real `Exited(0)` from the L1 binding is discarded in favour of `TimedOut` when the
  transcript is late — which is what the M1 ruling asked for ("a possibly-truncated transcript … the
  run is already classified as a non-finish"). The reverse order would be the bug and is not what the
  code does.
* **Leaked threads.** Only in the pathological case: on every normal run both threads send and exit,
  so nothing accumulates across hundreds of invocations. A pathological run leaks one grandchild, two
  parked threads and the two pipe read ends until that grandchild dies. Stated in the doc, honestly,
  minus the descriptors (nit, folded into N7).
* **Does the new test's bound discriminate?** Yes. Against the pre-fix code the call returns only
  when the backgrounded `sleep 30` exits, so `elapsed < Duration::from_secs(20)`
  (`oracle_deadline.rs:198`) fails at ~30 s — and the `Termination::TimedOut` assertion fails too,
  since the pre-fix path returned the direct child's `Exited(0)`. Against the head the run costs
  ~10 s (the sequential `20.01s` for the two ~10 s tests is the evidence, given the existing test's
  own `elapsed >= ORACLE_DEADLINE` / `< ORACLE_DEADLINE * 3` bracket). It is not a bound loose enough
  to pass either way. It bounds only from above, so a mutation that shrank the read budget to zero
  would still pass *this* test — the corpus harness is what would catch that, not this file.

## New defects

### N1 (high) — a declared-gap case that stops finishing is now green in every mode

`state_builtin_oracle.rs:489` returns the non-finish as `Some(String)` — the same channel the file
uses for an ordinary divergence. The caller pushes the case name into `differing` (`:525`) and then
asserts only that `differing` and `DECLARED_GAPS` are the same set (`:543`). `DECLARED_GAPS`
(`:74-78`) names `arg_option_array`, `condition_additional`, `condition_object`.

**Concrete failure.** The oracle's run of `condition_object` starts timing out — the oracle binary is
rebuilt, the host is loaded past ten seconds, the memory limit kills it, anything. `sweep_one`
returns `Some("condition_object did not finish: TimedOut …")`, `differing` contains
`condition_object`, it *is* a declared gap, so `unexpected` and `closed` are both empty and the
assertion passes. `transcripts` is only interpolated into the failure message, so the line naming the
program is never printed. The file goes on claiming "the state builtins' differential surface has not
moved" while one of its cases was not compared at all — **in both gate modes, including the full
gate.** Before this round the same event panicked out of `expect_exit_code` inside
`descriptor_diffs_with`, which is red everywhere.

**Fix shape:** keep the named line, but return it on a separate structural channel (a second `Vec`,
or an `assert!` in `sweep_one` like the one `builtin_status.rs:236` already uses) so it cannot be
matched against `DECLARED_GAPS`.

### N2 (high) — a corpus program that stops finishing is now green under a plain `cargo test`

`corpus.rs:359` returns a `Mismatch` for a non-finish. `corpus_differential`'s only assertion is
`assert!(!gate || mismatches.is_empty(), …)` (`:648`), so in report mode — gate command 3, and every
developer's plain `cargo test` — a non-finish is **not red**. It renders through `build_report` as
`[UNCLASSIFIED] <path>: the oracle did not finish: TimedOut …`, indistinguishable in kind from a
verdict divergence, under a banner that tells the reader the headline is "a progress signal for the
tasks still landing".

**Concrete failure.** A Task 4–24 probe lands in `corpus/phase-5a.txt` and hangs the oracle.
`cargo test --release --workspace` prints `105 of 106 matching`, exits 0, and costs 10–20 s more per
run than it did. Nothing in the crate asserts the headline is 106 (`SUBSET_FILES`' own doc says as
much: "A number a reader might eyeball is not a check"). The gated commands 4 and 5 still catch it,
so this is a degradation of signal rather than permanent blindness — but the global constraint is
unconditional, and report mode is the mode most runs use.

**Fix shape:** keep the `Mismatch` for the report line, and add an unconditional assertion over the
structural subset — e.g. collect non-finishes into their own `Vec<String>` and
`assert!(structural.is_empty(), …)` before the gated assertion.

N1 and N2 have one root cause: the ruling's "`Mismatch`-shaped structural line" was implemented by
routing the structural failure through each caller's *verdict* channel, and each caller's verdict
channel is exactly the thing that is gated or set-matched. `builtin_status.rs` took the third route —
an unconditional `assert!` that names the program — and is the one site of the three that satisfies
both halves of the ruling at once. It is also worth noting that the two oracle-invoking harnesses the
ruling did not name (`parse_version_oracle.rs:132`, `ir_dual_oracle.rs:134`) still reach
`expect_exit_code` and still panic, so after this round the tree treats the same structural event
three different ways, and the strictest treatment is at the sites nobody ruled on.

### N3 (low) — L3's replacement text is false about two of the three files it names

`oracle_deadline.rs:36-39`: "`corpus.rs`, `builtin_status.rs` and `state_builtin_oracle.rs` put every
program they own through `wait_with_deadline` on a plain `cargo test` already, **gating only their own
assertion**". Measured: `builtin_status.rs` and `state_builtin_oracle.rs` contain zero occurrences of
`REXX_CORPUS_GATE`/`GATE_ENV` — they gate *nothing*, and their assertions are unconditional. Only
`corpus.rs` matches the clause. The conclusion drawn from it ("the mechanism would already redden
ungated") stays true and is in fact stronger than the sentence claims, so this is a checkable-and-
false clause rather than a wrong conclusion — the same shape L3 charged, at smaller magnitude. The
reviewer's own wording ("those harnesses put every program through `wait_with_deadline` on a plain
`cargo test`") did not contain the clause.

### N4 (low) — the mutable-set count L4 removed from `write_and_wait` reappears nine lines away

`support/oracle.rs:318-319`: "A no-op for the `File`/`Stdio::null()` shapes **this method's only
caller passes today**". "Only caller" is the size of a mutable set and "today" is the phase-status
framing; both go false the first time a later task adds a second caller to `run_with_stdin` — which
is precisely the argument the ruling accepted for deleting "the one caller left". The global
constraint is stricter still ("A comment may not name the size of a set. Name the set; true counts
included"). Same fix as the one already applied nine lines down: name the caller.

### N5 (low) — history framing in the new test's comment

`oracle_deadline.rs:192`: "The bound that would fail **without the fix this test proves**". "The fix"
is an account of a change to this repository, and it comes out at zero cost by the ruling's own test:
*"The bound that would fail if the read were joined unconditionally: the backgrounded `sleep` runs
thirty seconds, so …"* says the same thing about the code as it is. Related nit, not charged: the
module doc now points at "the task's own report" for the control without naming it, which a reader of
the source cannot resolve.

### N6 (low) — `ORACLE_DEADLINE`'s own doc still reads as the bound on an invocation

`support/oracle.rs:99-105` says the constant is "How long an oracle invocation may run before
`wait_with_deadline` kills it", and grounds it in the `timeout -s KILL 10` a person applies by hand.
After this round an invocation can take twice that, and the by-hand check the constant claims to
restate is bounded at ten seconds while the harness's own path is bounded at twenty.
`wait_with_deadline`'s doc states the second budget but never states the sum, and
`oracle_deadline.rs`'s module doc says only "a bounded amount of wall time". One sentence at the
constant — "a run that also leaves its pipes held can cost twice this, once for the wait and once for
the read" — closes it.

### N7 (low) — two diagnostics quietly lost with the join

Both from `support/oracle.rs:509-514`:

* The timeout arm overwrites `code`/`deadline_exceeded` unconditionally, so a child that genuinely
  **died from a signal** while a descendant held the pipe is reported `TimedOut` rather than
  `Signaled` — a crash reported as a timeout. Both are non-finishes, so no verdict changes; only the
  diagnostic does, and `Termination::Signaled`'s own doc ("a crash, not a timeout") is what makes the
  distinction worth keeping.
* A reader thread that panics now disconnects its channel, which the `_` arm reads as a timeout →
  `TimedOut` with empty transcripts. The pre-round code panicked with `"stdout reader thread panicked
  for {path}"`. Unlikely to fire, but it is a named diagnostic replaced by a silent misclassification,
  and under N1/N2 that misclassification can now be absorbed as green.

## What I could not check

* **N1 and N2 by execution.** Both would need a program committed to the corpus or to
  `state_builtin_oracle.rs`'s `CASES`, and this review edits nothing. They rest on reading
  `corpus.rs:648` (`!gate || mismatches.is_empty()`) and `state_builtin_oracle.rs:543` (set equality
  against `DECLARED_GAPS`) against the early returns at `:359` and `:489`, plus the confirmed fact
  that `descriptor_diffs_with` calls `cpp.expect_exit_code()` unconditionally, which is what made the
  same event red before the round.
* **The read bound's own race**, symmetric to L1's — a `send` landing in the same instant
  `recv_timeout` expires. Unexercised, as the report says. Its outcome would be an occasional
  spurious `TimedOut`, which N1/N2 would then absorb rather than redden at two of three sites.
* **Gate commands 3 and 5.** I ran fmt, clippy, gate 4 and the per-test sequential split; I did not
  re-run the plain `--release` workspace run or the `memcap 8G --no-fail-fast` debug run. The report's
  figures for those are unverified by me, though gate 4 reproducing exactly (94 `ok` blocks,
  `106 of 106`, `10.01s`) is some evidence the rest of the table is honest.
* **Whether the ledger should also carry the moved before state.** The L4 ruling says "records" means
  the report **and** the ledger; the round's diff is tests-only, and `progress.md` is the controller's
  file. The report holds the measurement intact; whether `progress.md` needs the same row is a
  controller call, not an implementer omission.
* **Mutation testing** of the read bound, the L1 status binding, or the three new `did_not_finish`
  checks. None run; the two high findings above are what a mutation pass over the call sites would
  most likely have surfaced first.
