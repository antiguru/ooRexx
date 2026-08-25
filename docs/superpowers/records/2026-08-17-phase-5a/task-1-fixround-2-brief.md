# Task 1, fix round 2 — rulings on the re-review

Re-review: `.superpowers/sdd/2026-08-17-phase-5a/task-1-rereview.md`. Verdict **REWORK**: eight of
nine rulings closed, **M3 open**, seven new defects, two of them **high**. Read it — its evidence is
precise and in several places it hands you the fix.

**Before anything else: N1 and N2 are my fault, not yours.** My M3 ruling said to produce a
"`Mismatch`-shaped structural line naming `rel_path`". You implemented exactly that, and *that
phrasing is the defect*: it routed a **structural** failure through each caller's **verdict** channel,
and every verdict channel in this tree is either gated or set-matched. So a run that did not finish
became green — in report mode at `corpus.rs`, and in **every** mode at `state_builtin_oracle.rs`.
Do not read the round as sloppy work; read the corrected ruling below and apply it.

---

## M3 + N1 + N2 — one fix. **Ruled: a structural failure gets its own unconditional channel.**

`builtin_status.rs` is the one site of the three that got it right, and it did so by taking the third
route: an **unconditional `assert!` that names the program**. That is the standard for all of them.

* **`state_builtin_oracle.rs`** — the non-finish must not be returnable through the same
  `Option<String>` the caller matches against `DECLARED_GAPS`. Today a declared gap that starts timing
  out is absorbed as expected and the file goes on claiming its surface has not moved, **in both gate
  modes**. Separate channel or an unconditional `assert!` in `sweep_one`; it must be impossible to
  match a structural failure against `DECLARED_GAPS`.
* **`corpus.rs`** — keep the `Mismatch` so the report line still renders, **and** add an
  unconditional assertion over the structural subset, before the gated one. Collect non-finishes into
  their own collection and assert it empty regardless of `gate`. Report mode is the mode most runs
  use, and the global constraint on structural failures is unconditional.

**Also in scope, because it is the same defect at two more sites and my ruling under-scoped it rather
than this being new work:** `parse_version_oracle.rs` and `ir_dual_oracle.rs` still reach
`expect_exit_code` and panic from inside `support/oracle.rs`, naming no program. That is red
everywhere, so it is *correct* — but it is the exact complaint M3 was raised about, and after this
round the tree would otherwise treat one event three different ways. Bring them to the same shape.

**One event, one treatment, and the operator always learns which program.** That is the sentence to
implement against.

## N3 — **Ruled: fix, with the reviewer's own wording.** The clause "gating only their own assertion"
is checkable and false: `builtin_status.rs` and `state_builtin_oracle.rs` gate nothing. This is the
same shape L3 charged, one round later, in L3's own replacement text. Use the form that omits the
clause: those harnesses put every program through `wait_with_deadline` on a plain `cargo test`.

## N4 — **Ruled: fix.** "This method's only caller passes today" is a mutable-set count with a
phase-status "today" attached, nine lines from where you correctly deleted "the one caller left".
Name the caller.

## N5 — **Ruled: fix.** "The bound that would fail without the fix this test proves" is history, and
it comes out at zero cost: *"the bound that would fail if the read were joined unconditionally"* says
the same thing about the code as it is. **While you are there, resolve the dangling pointer the
reviewer noted but did not charge:** the module doc points at "the task's own report" without naming
it, which a reader of the source cannot follow. Name the file or drop the pointer.

## N6 — **Ruled: fix.** `ORACLE_DEADLINE`'s doc still reads as the bound on an invocation, and after
your change an invocation can cost twice it. State the sum at the constant — a run that also leaves
its pipes held costs this twice, once for the wait and once for the read — and reconcile it with the
by-hand `timeout -s KILL 10` the doc grounds itself in, which is bounded at ten while the harness is
bounded at twenty.

## N7 — **Ruled: fix both arms.**

* A child that genuinely **died from a signal** while a descendant held the pipe is now reported
  `TimedOut`, because the timeout arm overwrites `code`/`deadline_exceeded` unconditionally. No
  verdict changes — both are non-finishes — but `Signaled`'s own doc says "a crash, not a timeout",
  and a crash reported as a timeout sends a reader to the wrong place. Preserve an existing
  non-finish classification instead of overwriting it.
* A reader thread that panics disconnects its channel, which the `_` arm reads as a timeout. The
  pre-round code panicked naming the path. `recv_timeout` distinguishes `Timeout` from `Disconnected`
  — match them separately and restore the named panic.

## Not charged, and do not "fix" it

The partial-buffer path — `let _ = pipe.read_to_end(&mut buf)` sending what it has after a mid-stream
error — **is pre-existing, unchanged by this round, and out of scope for Task 1.** Both the reviewer
and I found it independently. It is the one truncation shape that can reach an `Exited` run, and it
did so before this task began. Leave it; I am recording it as a known gap rather than widening Task 1
to cover it.

---

## Verification for this round

**Every one of N1 and N2 needs a demonstration, not an argument.** The re-review could not execute
them because it edits nothing; you can. For each, produce the red: commit nothing permanent, but show
that a non-finish at that site is caught now and was not before. A temporary program plus a run you
report and then remove is fine — say exactly what you did so it can be repeated.

All five gate commands, each with **its own** exit status, plus the phase-gate command. Corpus stays
**106 of 106**. All three `oracle_deadline` tests pass under the gate; report each one's wall time.

Run the collapsed-comment-block diff over your own output before committing — it caught three things
for you last round, and this round adds N3, N4 and N5, which are all that shape.

Append to `.superpowers/sdd/2026-08-17-phase-5a/task-1-report.md` under a new "Fix round 2" heading.

**Return only:** status, commit SHAs, one line per finding, and anything you could not close.
