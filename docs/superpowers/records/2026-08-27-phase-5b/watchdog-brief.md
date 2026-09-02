## The crate-side watchdog

**Ruled by Moritz, 2026-09-02**, after the self-forward's non-termination was characterised. Phase 5b
is closed; this is a follow-on task, not a 5b task.

**BASE:** `6656d5a4f`. Tree clean.

---

## Why, and exactly what it must catch

`support::oracle`'s `ORACLE_DEADLINE` bounds the **oracle subprocess**. The crate side runs
**in process** through `run_program` (`lib.rs:6737`, via `on_interpreter_thread`) with no deadline at
all. Since the hot sweeps are rayon-parallel, one non-terminating case blocks the whole `collect` and
**stalls a gate indefinitely rather than reddening a row**.

That is no longer hypothetical. Measured by the controller at `6656d5a4f`, crate alone, both engines:

```rexx
o = .K~new
o~m
say 'returned'
::class K
::method m
  reply
  forward message('M')
```

runs for ever. Two threads: `rexx-run` parked in `futex_do_wait` joining, and `rexx-interp` in state
`R (running)` at **401 user ticks per 4 s — 100% of a core** (`/proc/<pid>/stat` fields 14 and 15,
sampled over an interval), `VmRSS` flat at 17,816 kB. **It is a spin, not a park**, and no activation
depth grows, so `MAX_ACTIVATION_DEPTH` never fires.

**Two measurement traps, both of which caught the controller. Do not repeat them.**

* **A *valued* `reply` does not reproduce it.** `reply 1` terminates in 0.03 s. Only a **bare**
  `reply` reaches the shape.
* **`/usr/bin/time -v timeout ... <binary>` reports `timeout`'s rusage, not the binary's**, because
  `timeout` forks and GNU time's `wait4` sees only its direct child. It printed `0.00 user` over
  8.00 s and the controller ruled the shape a deadlock on that basis. Launch the subject directly,
  capture `$!`, and sample `/proc/<pid>/stat`.

---

## Build two layers, and be exact about which catches what

The review this follows spent a day on instruments whose description outran their coverage. **State
each layer's reach in its own doc, in terms of what it does not catch.**

### Layer 1 — a deadline the interpreter honours, off by default

A deadline field on `Invocation`, checked at the clause boundary (`clause.rs:473`'s `enter_clause`
is the existing hook; `dispatch.rs:2334`'s `MAX_ACTIVATION_DEPTH` check is the precedent for a
resource guard).

**`Invocation` is the right home and its own `Engine` field says why** — read that doc before
choosing an environment variable instead. A variable is read once per process, so it cannot give two
arms inside one `cargo test`, and the harnesses that run a population call `run_program` directly
rather than spawning an interpreter.

Requirements:

* **Off by default.** `Invocation::none()` must mean exactly what it means today, and no shipped
  behaviour may change. The default path must not pay for the check — measure that claim rather than
  asserting it.
* **Not a panic.** A panic poisons a rayon worker and takes the sweep with it. Return an `Outcome`
  the harness can render as a row failure, distinguishable from any answer a program could produce
  itself.
* **Not a Rexx condition.** This is a harness bound, not language behaviour. It must not be
  catchable, must not appear as an error number, and must never fire when the deadline is unset.

**What layer 1 catches:** a run that keeps executing clauses. That is the one crate-side
non-termination anyone has found.

**What layer 1 does not catch, and the doc must say so:** a run parked on a wait that executes no
further clause, and a spin *inside* a single builtin or a single clause's evaluation. Neither reaches
a clause boundary, so no clause-boundary check can see them.

### Layer 2 — an outer bound in the harness

So that a case layer 1 cannot see still fails a row rather than stalling a sweep. Apply it in the
rayon sweeps (`ir_dual.rs`, `assertions.rs`, `bif_assertions.rs`, and `corpus.rs`).

**Correction, measured 2026-09-02 at `6656d5a4f`: `corpus.rs` is not one of the rayon sweeps.**
`rayon` appears in `ir_dual.rs`, `assertions.rs` and `bif_assertions.rs` and in no other file under
`crates/rexx-exec/tests/`; `corpus.rs` runs its 331 programs serially. It is still worth the bound --
a hang there stalls the serial loop and so the gate -- but "stalls the whole `collect`" is the wrong
reason for that one file.

**Be honest about layer 2's own limit.** A Rust thread cannot be killed, so a hung case's thread keeps
running after the harness gives up on it. Say in the doc what the surviving thread costs — at 100% of
a core it competes with the rest of the sweep — and whether the process still exits.

**Whether a crate-side *parked* shape exists at all is an open question, not a premise.** The known
parked shape, `GUARD ... WHEN` with a false expression, blocks the *oracle*
(`corpus/oracle-crashes.txt` entry 7); this crate refuses it at `Loud::guard_when_false`. If you can
construct no parked crate-side case, then layer 2 is **defensive and untested against a real case**,
and it must be documented that way rather than credited with coverage it has not demonstrated.

---

## Done when

* The program above, run under a deadline, produces a bounded distinguishable outcome instead of
  running for ever — measured, with the transcript.
* A sweep containing that program **reddens that row and completes**, rather than stalling.
* Default behaviour is unchanged: five gates each 0, `331 of 331 matching`, both gate tables' 5b rows
  0 not-agree, and the phase gate 0.
* **Three controls, each recorded as run:**
  1. **The witness is live** — remove the deadline check and the hang test hangs again (bound it, so
     the control does not stall your own run).
  2. **No false positive** — the slowest legitimate program in the corpus, and the debug `memcap`
     gate's slowest case, do not trip the chosen value. Say what value you chose and what the
     slowest observed case actually takes; a deadline chosen without that figure is a number.
  3. **Layer 2 is reached** — a case that layer 1 cannot see still fails its row. If you cannot
     construct one, say so and mark layer 2 untested rather than inventing a case that only looks
     like one.

## Rules

* Never run a program in `corpus/oracle-crashes.txt` and never construct one of those shapes. The
  self-forward family is in that file — it is listed there for the **oracle**, and running it on this
  crate is what this task is for, but the oracle must never see it.
* No `unsafe`. Stop and say so rather than reach for it.
* Three descriptors read separately, never `2>&1`; both engines.
* A gate run does not survive the turn that starts it: statuses to a file, a waiter that exits on the
  process vanishing, and read them in the turn you commit.
* Tree hash before the first gate and after the last, covering untracked files' bytes.
* Audit every `interpreter/` citation you add.
* Commit with `git commit -F <file>`, naming paths explicitly; confirm `Cargo.lock` is absent.
* Report to `.superpowers/sdd/2026-08-27-phase-5b/watchdog-report.md`. Say plainly what you did not do.
