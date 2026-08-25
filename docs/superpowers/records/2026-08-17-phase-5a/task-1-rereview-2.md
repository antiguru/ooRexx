# Task 1, fix round 2 — re-review

Scope `8017f027a..63a49c9b9` (`77f8e507b` the classification and prose fixes, `63a49c9b9` the
structural channel). Read-only; nothing in the tree was edited except this file. The standard is
`task-1-fixround-2-brief.md`'s rulings; round 1's approved work is not reopened.

## Verdict

**REWORK**, narrowly and for comments only. **Every ruled finding is closed**, and the two that
mattered are closed in the way the corrected ruling asked for: the structural channel is now
unescapable at all five named sites, I traced each one and name the assertion below, and the N7
arm-one preservation is guarded by an existing test that would fail if it had been written the other
way round. The demonstrations' premise reproduces on this host, independently measured.

The four new defects are all **low**: one breaks a stated global constraint outright (a comment
naming the size of the corpus), one is another instance of the history-framing class this very round
was fixing, one is a diagnostic label that cannot be resolved by the reader it was added for, and one
is a claim of "everywhere" that one remaining site falsifies. Each is a one- or two-line edit; none
touches the mechanism. I am calling REWORK rather than APPROVE because the controller ruled the
identical class (N4, round 1) "fix" one round ago, and because the label defect landed in what this
round newly specified — which is this project's measured pattern.

## What I ran

| command | result |
| --- | --- |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --release --workspace` | exit 0; corpus `106 of 106 matching -- REPORT MODE`; 94 `test result: ok`; 0 `FAILED`; `oracle_deadline` `14 passed`, `finished in 0.00s` (both non-unit tests skip ungated) |
| `REXX_CORPUS_GATE=1 cargo test --release --workspace` | exit 0; corpus `106 of 106 matching`; 94 `test result: ok`; 0 `FAILED`; `oracle_deadline` `14 passed`, `finished in 10.01s` |
| collapsed-comment-block diff, base vs head, **all six** changed files | one new defect the implementer's own pass missed — D2 below |
| `rustc`-built probe of `mpsc::recv_timeout` in three states | `Ok` for sent-then-dropped (at 5 s *and* at `Duration::ZERO`), `Err(Disconnected)` for a panicked-before-send thread, `Err(Timeout)` for a live holder — see N7 |
| the demonstration's probe against the oracle by hand, fresh empty dir, `timeout -s KILL` | direct child exits rc 0 printing `done` in **0.010 s**; the same run read *through a pipe* returns in **15.010 s** — the non-finish the demonstration needs is real |
| the same probe against `target/release/rexx-run` | rc 120 in **0.003 s**, `rexx-exec: ADDRESS is not implemented (Phase 7)` |
| `git diff --stat 8017f027a..63a49c9b9 -- . ':!rust/crates/rexx-exec/tests'` | empty — tests-only, so no performance sitting, and the report says why (`task-1-report.md`, L6, amended to "either round") |
| `/bin/grep -ra 'REXX_PHASE_GATE\|CLOSED_PHASES' rust/crates rust/corpus` | empty — the phase-gate command is still vacuous, as recorded |

## The ruled findings

* **M3 + N1 + N2 — CLOSED.** One event, one treatment, at all five sites; each is unconditional and
  each names the program. Traced individually:
  * `corpus.rs` — `check_case:369` is the **only** non-finish exit and it is the **only**
    `Mismatch` constructed with `structural: true`; the other constructor (`:421`) is downstream of
    that early return, so a non-finish cannot reach it. `Mismatch` is built in exactly two places
    (`grep -n Mismatch corpus.rs`), both inside `check_case`, whose sole caller is
    `corpus_differential:649`. That function has **no early return at all** — the only pre-assertion
    exits are panics (`!subset.is_empty()`, `canonicalize`, `read_subset`) — so every route reaches
    `:667-678`, the unconditional `assert!(structural.is_empty(), …)`, and it stands **before** the
    gated `!gate || mismatches.is_empty()` at `:680`. `emit_uncaptured` runs before both, so the report still
    renders. Red with the gate unset (verified: `corpus_differential` runs and prints in report
    mode, gate command 3's log) and with it set.
  * `state_builtin_oracle.rs` — the file's only `oracle.run` is `:482`, and `:492-498` asserts
    unconditionally on `case.name` before the `Option<String>` the caller set-matches against
    `DECLARED_GAPS` can be produced. The file contains no `GATE`/`gate_mode` reference at all, so
    both modes are the same mode here; measured, its test runs in report mode (0.84 s in gate 3's
    log). A declared gap that starts timing out is now impossible to absorb.
  * `builtin_status.rs` — unchanged, still `:235-240`, still ungated (18 tests, 0.39 s in report
    mode). The standard the other four were brought to.
  * `parse_version_oracle.rs` — `:137-142`, before `descriptor_diffs` and therefore before
    `expect_exit_code`.
  * `ir_dual_oracle.rs` — `:140-146`, inside `render_oracle`, before `expect_exit_code:148`. I
    checked the absorption route the brief warned about: this file's "expected set" is the
    `not-oracle-bytes` marker, whose `assert_ne!(ours, from_oracle, …)` would **pass** on a
    non-finish (empty oracle transcript differs from ours). The new assertion fires first, so that
    route is closed. I also confirmed `datadriven` 0.9.0 contains no `catch_unwind`: a panic inside
    the `file.run` closure propagates immediately rather than being folded into its accumulated
    `failures`.
* **N3 — CLOSED**, and the replacement text is now true rather than merely less false. Measured, not
  read: in report mode `builtin_status` runs 18 tests in 0.39 s and `state_builtin_oracle` 14 in
  0.84 s — they really do put their programs through `wait_with_deadline` ungated — and neither file
  mentions `REXX_CORPUS_GATE`. `corpus.rs` runs ungated too. No new clause was introduced.
* **N4 — CLOSED.** `support/oracle.rs:316-318` names `input_oracle.rs`'s
  `an_unreadable_console_is_end_of_input`. Verified that is the sole caller of `run_with_stdin`
  (`input_oracle.rs:556`, one hit tree-wide), so the name is right and no count or "today" survives.
* **N5 — CLOSED**, both halves. `oracle_deadline.rs:189` is "The bound that would fail if the read
  were joined unconditionally", and the module doc's unfollowable pointer is gone ("measured by hand
  rather than run here"). Dropping rather than naming is one of the two options the ruling offered.
  *Not charged, but worth the controller's eye:* the report justifies the drop with "the report file
  lives under `.superpowers/`, which is not tracked", which is true today and incomplete — SDD
  records are copied to the tracked `docs/superpowers/records/<plan>/` when a plan closes. The
  action is still right (that path does not exist yet, so naming it would dangle now); only the
  stated reason is narrower than the full picture.
* **N6 — CLOSED, and the arithmetic checks out.** `ORACLE_DEADLINE`'s doc now names both budgets and
  says an invocation is bounded "at up to **twice** this". The code agrees: the wait loop breaks at
  `start.elapsed() >= ORACLE_DEADLINE` (`:472`), then `read_deadline = Instant::now() +
  ORACLE_DEADLINE` (`:501`) is a *fresh* budget shared by both `recv_timeout` calls (`:503`, `:505`,
  both `saturating_duration_since` the same instant). So 10 + 10, and `wait_with_deadline`'s own
  "the pair together are bounded by it once, not twice" is about the two channels and does not
  contradict it. The `timeout -s KILL 10` reconciliation is stated the way the ruling asked ("bounds
  only the process, not a descriptor it hands to one of its own").
* **N7 arm one — CLOSED, and the inversion the brief asked about is not there.**
  `support/oracle.rs:542-548` synthesises `TimedOut` **only** when
  `classify_termination(code, deadline_exceeded)` is already `Exited(_)`. Walked all four inputs: a
  clean `Exited(0)` plus a held pipe → overwritten → `TimedOut` (the M1 hole stays closed); a
  killed-at-deadline `TimedOut` → preserved; a genuine `Signaled` → preserved (the ruled fix); an
  `Exited(n)` bound after the kill by L1's `child.wait()` → overwritten, correctly, since the
  transcript is unreadable. **This is witnessed permanently and it discriminates**:
  `a_background_process_holding_the_pipe_open_does_not_hang_the_run` asserts
  `outcome.termination == Termination::TimedOut` on exactly the exited-cleanly-plus-held-pipe shape,
  so the inverted condition (`if !matches!(…, Exited(_))`) would fail it with `Exited(0)`. It passes
  under the gate at 10.01 s, measured in my own run.
* **N7 arm two — CLOSED, verified by execution rather than by reading std.** `:515-520` matches
  `Disconnected` separately and panics naming the path and which pipe, on the test's own thread, so
  it reaches the operator as a test failure. The false-positive the brief asked about does not
  exist: my `rustc` probe on this toolchain returns `Ok(7)` for a channel whose sender was dropped
  **after** a successful send, and `Ok(8)` for the same at `Duration::ZERO` — queued values are
  delivered before disconnection is reported. `Disconnected` therefore implies the reader thread
  died before its `send`, and since both `read_to_end` and `send` are `let _ =`-swallowed, that
  means a panic. `matches!` on a non-`Copy` `Result` binds nothing and so does not move it; the
  later `match (stdout_result, stderr_result)` still compiles, which the build settles.
* **"Do not fix" — respected.** `git diff 8017f027a..63a49c9b9 -- support/oracle.rs` has **no** hunk
  touching `read_to_end`, `thread::spawn` or `send(buf)`; the reader-thread bodies at `:456-465` are
  byte-identical to the base. The partial-buffer path is untouched.
* **The brief's "all three `oracle_deadline` tests" — the implementer is right and this is a
  controller error.** `grep -c '#\[test\]' oracle_deadline.rs` is **2**, and the file has no
  `mod tests`. The `14 passed` in both gate runs is those two plus twelve unit tests that ride in
  from `support/` (5 in `support/oracle.rs`, 7 in `support/mod.rs`), which the per-test listing in
  my gate-4 log confirms by name. Both wall times are in the table above: 10.01 s for the pair
  concurrently under the gate, 0.00 s ungated (both skip).

**Tally: 8 of 8 CLOSED (M3+N1+N2 as one, N3, N4, N5, N6, N7 arm one, N7 arm two).**

## The demonstrations

**The design discriminates, and its premise reproduces.** I re-ran the probe by hand: the direct
child exits rc 0 printing `done` in 0.010 s, while a reader on its stdout pipe waits **15.010 s** for
EOF. So under `wait_with_deadline` the wait loop classifies `Exited(0)` almost immediately and the
read budget then expires at 10 s — a genuine non-finish, reached through the ordinary
`Oracle::run` path, which is what both call sites needed. The rejection of `do forever; end` is also
sound and not an excuse: the crate side runs in-process with no timeout of its own, and I measured
`rexx-run` refusing `ADDRESS` in 0.003 s, so the probe exercises the oracle side while leaving the
in-process side instant.

Both demonstrations reproduce the *caller's own logic* rather than calling the real test function —
`corpus_differential` hard-codes its corpus directory and
`every_state_builtin_case_matches_the_oracle_except_the_declared_gaps` uses the committed `CASES`.
That is the right call for the second one (a hanging source in `CASES` would also feed the
undeadlined in-process `every_declared_gap_names_a_case_and_fails_loudly`, as the report says), and
it means the demonstrations prove the **assertions' behaviour** but not their **placement**. The
placement is what I traced by reading, above, and it is correct.

**Nothing permanent witnesses either fix, and that is a licensed gap rather than a defect** — the
ruling said "a temporary program plus a run you report and then remove is fine". What it costs is
worth stating for the controller: deleting `corpus.rs`'s structural assertion, flipping its
`structural: true` to `false`, or deleting `sweep_one`'s assertion would leave all five gate commands
green, because no committed program can produce a non-finish (the corpus must stay 106 of 106).
`oracle_deadline.rs` witnesses that the *event* is manufacturable and classified; nothing witnesses
that a call site reddens on it. A permanent witness is achievable if the controller ever wants one:
`check_case` takes `corpus_dir`/`rel_path` and `sweep_one` takes a `&Case`, and `corpus.rs` already
uses the `#[ignore]` + child-`cargo test` pattern for precisely this "prove it rather than assert it"
problem (`demonstrate_the_report_reaches_a_plain_cargo_test`). Cost would be ~10 s, gate-only.

## New defects

### D1 (low) — a new comment names the size of the corpus, in the file whose own doc rejects that

`corpus.rs:659-666`, the comment on the new structural assertion: a report-mode run "cannot let a
program that stopped finishing pass as `"106 of 106 matching"` just because the corpus gate itself
was not requested."

The global constraint is unconditional: **"A comment may not name the size of a set. Name the set;
true counts included. Measurements keep their numbers."** 106 is the size of a mutable set that the
same document requires to grow ("The corpus is **106 of 106** … and must stay there and grow"). This
is the class the controller ruled "fix" one round ago as N4, in the same task.

**Concrete failure.** Task 4 lands its first probe in `corpus/phase-5a.txt`. The headline every run
prints becomes `107 of 107 matching`, and this comment names a string no run produces — in the file
whose `SUBSET_FILES` doc already says "A number a reader might eyeball is not a check". The
in-tree precedent for keeping a number (`crates/rexx-exec/src/lib.rs:1415`, "105 of 106") sits inside
a fenced *measured* block introduced by "Measured, by putting a single positional rule …", which is
the licensed exception; this sentence is a hypothetical about a future run, not a measurement.

Secondary, and the reason the sentence does not survive a careful reader either: a non-finish is
pushed to `mismatches` and never counted in `matched`, so such a run prints `105 of 106`, not
`106 of 106`. The failure N2 described was "prints `105 of 106 matching`, exits 0". As written the
comment names the one headline that cannot accompany the event it describes.

**Fix:** name the set, not the count — "…pass as a fully-matching corpus just because the corpus gate
itself was not requested."

### D2 (low) — a new comment is history about the shape this round replaced

`support/oracle.rs:511-512`: "The pre-channel code named the path in exactly this case
(`.join().unwrap_or_else(|_| panic!(...))`)".

This is the L4/N5 class, added by the round that was fixing it, and it fails the ruling's own test:
strike the historical framing and the block still says the same thing about the code as it is,
because the last sentence carries the whole argument ("folding it into a `TimedOut` classification
instead would trade a named harness bug for a silent misclassification"). An account of "the shape
that was replaced" is exactly what the constraint sends to the report, and the report already
carries it (Fix round 2, N7).

**Concrete failure.** A reader in six months greps for `.join().unwrap_or_else` in this file, finds
nothing, and cannot tell whether the comment describes code that was removed, code in another file,
or code that never existed here. It also falsifies the round's own verification claim — the report
says the collapsed-comment diff found "no … history framing beyond the instances N3-N5 name and fix
above"; my collapsed-comment diff over all six files found this one.

**Fix:** delete the sentence and its parenthetical; the surrounding two sentences are complete.

### D3 (low) — `ir_dual_oracle.rs`'s new label cannot be resolved to a stanza, and is not stable

`ir_dual_oracle.rs:209`: `let label = format!("{filename}#{checked}");`, with `checked` incremented at
`:208`. `checked` is the **running total across every file** `datadriven::walk` reads — it is the
same counter the `oracle.invocations() == checked` assertion needs at `:243`, so it cannot be a
within-file index. `render_oracle`'s new doc nevertheless says "`label` identifies the stanza".

**Concrete failure.** `tests/ir_dual_cases/` holds 20 files and 197 `program` stanzas. The oracle
hangs on the single stanza in `ir_dual_cases/rexxcps`; the panic reads
`tests/ir_dual_cases/rexxcps#152: the oracle did not finish: TimedOut`. The operator opens a
one-stanza file and looks for its 152nd stanza. For `assignment-and-say` (27 stanzas) the number
gives no help at all in locating which one hung, while the format implies it does.

Worse, it is not reproducible: `datadriven` 0.9.0's `test_files` (`src/lib.rs:286-301`) walks
`fs::read_dir` with **no sort**, so file order is filesystem order. The same stanza gets a different
number on another machine, and adding or removing a case file renumbers every stanza after it.

**Fix:** one line — a per-file counter reset inside the `walk` closure beside
`let filename = file.filename.clone();`, incremented in the inner closure and used for the label,
leaving `checked` to keep counting invocations. (`TestCase::line_number` exists in `datadriven` but
is private, so a `file:line` label is not available.) Alternatively drop the `#n` and let the
filename plus the panic's `Termination` stand.

### D4 (low) — "everywhere" is falsified by one remaining site

`63a49c9b9`'s subject is "Give a structural failure its own unconditional channel, **everywhere**",
and its body says "that shape is now used **everywhere the same event can occur**". One site is left:
`input_oracle.rs:556` runs `oracle.run_with_stdin(&abs, …)` and `:567` reaches
`cpp.expect_exit_code()` inside an `assert_eq!` with no `did_not_finish` check in front of it. The
same test's sibling in the same file (`:449-457`, M2's fix) does have one, which is why a
file-granularity reading of the claim passes and a call-site reading does not. `ir_dual_oracle.rs`'s
new comment inherits the same overreach: "the shape **every** oracle-invoking harness in this crate
uses for the same event".

**Concrete failure.** The oracle hangs or is killed on `unreadable.rex`; the failure is
`oracle run did not exit normally, so it has no exit code: TimedOut`, raised from inside
`support/oracle.rs`, naming no program — which is precisely the complaint M3 was raised about. It is
red (the panic is unconditional wherever the gate lets the test run), so nothing goes green; what is
wrong is the claim, and the claim is in an uneditable commit message.

**Note this site was never ruled** — the round-2 brief named exactly two "also in scope" sites and
this is not one of them, so the code is not an implementer omission. What the round can still fix is
the ir_dual comment's "every"; whether to add the sixth assertion is the controller's call, and it is
two lines.

## What I could not check

* **Gate command 5** (`REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast`, the
  debug build). I ran fmt, clippy and gate commands 3 and 4, and the phase-gate command's vacuity
  premise. The report's figures for command 5 are unverified by me; commands 3 and 4 reproducing
  exactly — 94 `ok` blocks, `106 of 106`, `10.01s`, 0 `FAILED` — is some evidence the rest of the
  table is honest.
* **The two gate-only harnesses in report mode.** `parse_version_oracle.rs` and `ir_dual_oracle.rs`
  return before `locate()` when the gate is unset (measured: both finish in 0.00 s in gate 3's log),
  so their new assertions are unreachable ungated. That satisfies the brief's "red with the gate
  unset" only vacuously — the oracle is never invoked there, so no structural event exists to be
  green. Pre-existing to both files, untouched by this round, and the same treatment round 1 gave
  `input_oracle.rs`'s wholesale skip: noted, not charged.
* **A reader thread actually panicking.** `Disconnected` is unreachable through any path I can name
  (`read_to_end`'s and `send`'s errors are both swallowed), so the new panic at `:515-520` is a
  diagnostic for a harness bug that would have to be introduced first. I proved the *channel*
  semantics by execution; I could not produce the panic through `wait_with_deadline` itself without
  editing it.
* **The read bound's own race** — a `send` landing in the same instant `recv_timeout` expires.
  Unexercised, as both the report and round 1 say. Under the round-2 code its outcome is an
  occasional spurious `TimedOut`, which is now red at all five converted sites rather than absorbed
  at two of them.
* **Mutation testing** of the `structural` field, the five assertions, or the N7 arm-one condition.
  None run. The arm-one condition is the one place where a mutation *would* be caught today, by
  `a_background_process_holding_the_pipe_open_does_not_hang_the_run`; the other four would not be,
  which is the "unwitnessed" point above.
* **Whether `progress.md` needs the moved before-state measurement.** Unchanged from round 1's
  re-review; still a controller call, not an implementer omission.
