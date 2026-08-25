# Task 1, fix round 1 — controller rulings on the review's nine findings

Review: `.superpowers/sdd/2026-08-17-phase-5a/task-1-review.md`. **Read it first** — it carries the
evidence for each finding, and in several places the exact fix. Verdicts were **spec compliance:
APPROVE**, **task quality: REWORK**. Nothing about the spec half is reopened here.

I have ruled on every finding. Where I overrode the reviewer, or chose between options it offered,
the reasoning is below — apply the ruling, not your own re-derivation of it.

---

## M1 — bound the read, do not merely document it. **Ruled: fix.**

The reviewer offered two routes: bound it, or document that the deadline bounds the child rather than
descriptors the child passed on, and add it to the human-check list beside the in-process half.

**Documenting it is not available, and here is why.** The human check the plan prescribes is that
every committed probe is first run by hand through `rexx-run` under `timeout -s KILL 10`. Run by
hand, stdout is a **terminal**, not a pipe. A grandchild holding an inherited terminal descriptor
blocks nothing: `rexx` exits, the prompt returns, the run looks clean. Under the harness the same
program has stdout on a **pipe**, the grandchild holds the write end, and `read_to_end` blocks until
it closes. **The check cannot see its own subject** — it comes back green on exactly the programs
that hang the suite. A limitation whose stated mitigation is structurally blind to it is not
mitigated, so it gets closed rather than written down.

Implement the route the reviewer describes: hand each reader thread's buffer back over a channel, and
stop waiting on the join once the child is dead **and** the deadline has passed. A possibly-truncated
transcript is free here — the run is already classified as a non-finish, and a non-finish is a
structural failure whose bytes nobody compares.

**Then document the residual honestly**, because one remains: the orphan keeps running and keeps the
descriptor; you have stopped reading, not reaped it. Say that, and say what it costs (a leaked
process, not a hung suite).

Prove it with a test. `address system 'sleep 30 &'` — or whatever shape reproduces it — through
`Oracle::run`, asserting the call returns in well under 30 seconds. **Give it a bound that would fail
if you had not made this change**, and say in the report what the same test does against the code as
it stands now.

## M2 — **Ruled: fix, and make it structural.**

`input_oracle.rs`'s `diffs` reads two identical non-finishes as agreement. This is the one call site
where the brief's own helper was available and unused, and it converts the exact defect the task
exists to fix from a false red into a false green — the worse direction. Use `did_not_finish` on both
sides and make a non-finish a structural failure, red in **both** gate modes, per the global
constraints.

## M3 — **Ruled: fix. The program must name itself.**

A timed-out corpus program currently panics from `support/oracle.rs` with no path in the message, and
`corpus.rs`'s accumulated report is emitted after the loop, so it never prints. Fix at the callers
that hold the path — `check_case`, and the sweep sites in `builtin_status.rs` and
`state_builtin_oracle.rs` — rather than by threading a context argument into `expect_exit_code`, so
the structural line is shaped like the report's other lines and names `rel_path`.

## L1 — **Ruled: fix.** Bind the status `wait()` returns and pass it through instead of hardcoding
`None`. It removes a flaky red on a program finishing within one poll interval of the deadline, and
it makes `classify_termination`'s `(Some(status), _)` arm reachable from its only caller — the arm a
unit test currently pins while nothing can reach it.

## L2 — **Ruled: fix.** `run_with_stdin` must drop `child.stdin` before waiting, matching what
`wait_with_output` did. Today's only caller passes a `File` so nothing is broken, but the method's
signature invites `Stdio::piped()` and that caller would get a ten-second timeout instead of an
answer.

## L3 — **Ruled: fix, using the reviewer's own replacement.** "The switch every oracle-invoking
harness in this crate uses" is checkable and false — `corpus.rs`, `builtin_status.rs` and
`state_builtin_oracle.rs` invoke the oracle unconditionally and gate only the assertion. State it the
honest way instead: the deadline **mechanism** runs in both modes, since those harnesses put every
program through `wait_with_deadline` on a plain `cargo test`, so a broken poll loop or a
misclassified normal exit reddens ungated; only the **kill** is verified under the gate.

## L4 — history framing. **Ruled: split.**

* `Termination`'s "the `exit_code: i32` sentinel **this replaced**" — **remove the history framing,
  keep the content.** The test is whether the framing is removable at zero cost, and it is: *"a named
  type rather than an `exit_code: i32` sentinel, because `status.code().unwrap_or(-1)` gives a signal
  death and a normal `exit -1` the identical representation"* says the same thing about the code as
  it is. The measurement justifying the current design stays; the account of what it replaced goes.
* `oracle_deadline.rs`'s **before state** — **move it to the report and out of the module doc.** Your
  brief said the task "records" it and did not say where; I am ruling that "records" means the report
  and the ledger, which is where `rust/CLAUDE.md`:96 puts history. Keep in the module doc only what
  the test proves and what it does not, stated without reference to any prior implementation.
* `write_and_wait`'s **"the one caller left"** — **remove.** This is a count of a mutable set and goes
  false the moment a later task adds a caller.
* `POLL_INTERVAL`'s route rationale — **keep, unchanged.** The brief asked for the choice to be
  recorded, and design rationale is not "what moved or shrank". The reviewer declined to charge it
  and was right.

## L5 — **Ruled: no change, except the one L4 already covers.**

`rust/CLAUDE.md`:87's rule targets a **mutable in-repo aggregate** — a count no human will
re-verify and that the next task falsifies. "The two plain values", "the two primitives", "two
failing arms", "those two pipes" name immutable arities of a fixed signature; they cannot rot. The
base file is dense with the same shape, and enforcing it here would apply a standard the file does
not hold itself to. The reviewer drew exactly this line and charged only the mutable one. Leave the
rest alone.

## L6 — **Ruled: fix in the report, not in code.** State why no sitting was run — the diff touches
only `rust/crates/rexx-exec/tests/`, so the release binary the axes measure is byte-identical and a
sitting would measure noise. Also run and report
`REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1 cargo test --release --workspace`, noting that it is currently
byte-identical to gate command 4 because neither `REXX_PHASE_GATE` nor `CLOSED_PHASES` exists in
`rust/` yet, and that this stops being true at Task 4.

---

## Verification for this round

All five gate commands, each captured with **its own** exit status, plus the phase-gate command from
L6. Corpus must stay at **106 of 106**. Both the existing `oracle_deadline` test and your new M1 test
must pass under the gate, and you must report the wall time of each — the ungated-versus-gated
`0.00s` / `10.01s` split is what witnesses the deadline, and your M1 test needs an equivalent witness
of its own.

Append your account of this round to `.superpowers/sdd/2026-08-17-phase-5a/task-1-report.md` under a
new heading — do not rewrite what is already there.

**Return only:** status, commit SHAs, one line per finding saying what you did, and anything you
could not close.
