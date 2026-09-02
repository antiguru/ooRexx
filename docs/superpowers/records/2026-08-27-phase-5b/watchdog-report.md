# The crate-side watchdog -- report

**Base:** `6656d5a4f`. **Date:** 2026-09-02. Branch `plan/rust-rewrite`, worktree
`/home/moritz/dev/repos/ooRexx-rust-rewrite`.

No `unsafe` was added, and none was wanted. No `interpreter/` citation was added, so there was
nothing to audit on that front.

---

## 1. The subject, re-measured rather than taken from the brief

Both engines, crate alone, release `rexx-run`
`4b3a8542f828da867e6b212c40373162627fc8cafc31fcc6db00c901ff7f832f`, launched directly with `$!`
captured and `/proc/<pid>/stat` fields 14 and 15 sampled over four seconds:

| engine | state after 5 s | utime delta / 4 s | stime delta | `VmRSS` | threads |
|---|---|---|---|---|---|
| `ir` | alive | **400 ticks** | 0 | 17,844 kB | `rexx-run` S in `futex_do_wait`, `rexx-interp` **R** |
| `tree-walker` | alive | **401 ticks** | 0 | 17,828 kB | same |

100% of a core with a flat resident set on both. The controller's figures reproduce: 401 ticks
against 400 and 401 here, and a resident set within a few kB of the 17,816 kB recorded.

### Three measurement traps, not two

The brief carries two. A third caught me and a fourth caught the controller a second time, and both
are about a *wrapper* standing between the reader and the subject.

* **`/usr/bin/time -v timeout ... <binary>` reports `timeout`'s rusage** -- the brief's, not
  re-encountered because I launched the subject directly.
* **A valued `reply` does not reproduce the hang** -- the brief's, and now a committed case
  (`a_valued_reply_ends_the_same_program`) rather than a warning.
* **`REXX_ENGINE=tree` is a rejection, not an answer.** My first arm ran that spelling and the
  process exited in under a second at rc 2. Read from the status alone that is "the tree-walker
  terminates"; it is `rexx-run` refusing an unrecognised engine name, and the spelling is
  `tree-walker` (`bin/rexx-run.rs`'s `engine_from`).
* **Stdout is buffered, so a killed run shows none of it.** `Outcome` collects `stdout` in a `Vec`
  and hands it back at the end (that type's own doc records the cost), so a `SIGKILL` discards
  everything the program printed. `stdbuf -o0` does not change it -- Rust's `std::io::Stdout` is not
  libc stdio. A reader reproducing this will see an empty stdout and conclude the program never got
  going.

### What the program actually does

`say 'returned'` **runs.** `o~m` answers at the `REPLY`, the main body finishes normally, and only
then does `execute` reach `Interp::run_deferred_replies`, whose queue each drained body refills.
`lib.rs`'s own comment above that drain says so -- it runs "after the main body's own report and
after its exit status is settled". So the non-termination is entirely *after* the program's last
clause, and `the_self_forward_ends_at_its_deadline_on_both_engines` asserts `stdout ==
"returned\n"` under a deadline.

---

## 2. What was built

### Layer 1 -- a deadline the interpreter honours, off by default

* `Invocation::with_deadline(Duration)` (`crates/rexx-exec/src/invocation.rs`). A field, for the
  reason the `Engine` field's own doc gives and which transfers word for word: an environment
  variable is read once per process, and the harnesses that run a population call `run_program`
  directly.
* `crate::clause::Deadline` and `Interp::count_clause_against_deadline`
  (`crates/rexx-exec/src/clause.rs`). One decrement and one branch per clause; the clock, the reload
  and the "is there a deadline at all" question all sit behind the zero test in a `#[cold]`
  `Interp::countdown_reached`.
* `Failure::Deadline` (`crates/rexx-exec/src/error.rs`), a variant of its own rather than a reuse of
  `Loud`. `Interp::offer_to_trap` declines every failure that is not `Failure::Raised`, so it is not
  trappable by construction; a `CALL ON` trap never sees a failure at all.
* `DEADLINE_EXIT = 121` and `DEADLINE_REPORT` (`crates/rexx-exec/src/lib.rs`). A different number
  from `NOT_IMPLEMENTED_EXIT`, because the two mean opposite things to a harness: a loud refusal is a
  row this crate is *allowed* to be blocked on and several harnesses classify it that way.
* One place decides the status, below everything that runs clauses: `execute`'s
  `if interp.deadline_expired()` guard, after `run_deferred_replies` and after the `UNINIT` sweep.
  It is there rather than in the three match arms because `Interp::run_one_uninit` discards a raised
  condition and an `EXIT` on purpose (the oracle's dispatcher does), and a deadline reaching it would
  otherwise vanish silently. `a_finalizer_that_outlives_the_deadline_is_still_reported` is the
  witness, with its own control.
* `run_deferred_replies` **stops draining** on a deadline. Without that the loop keeps popping
  entries each drained body refills and merely accumulates failures.

**The site list is enforced by a type, not by a rule.** `Interp::enter_clause` and
`Interp::enter_stepped_clause` both take a `clause::DeadlineCounted`, which only
`count_clause_against_deadline` can produce and which nothing outside `clause.rs` can construct. A
site that opens a clause without counting it does not compile. That shape was chosen because that
file's own module doc records four rounds of exactly the defect a hand-maintained site list
produces -- and section 4 below is that defect reproduced on the first attempt, measured.

### Layer 2 -- an outer bound in the harness

`crates/rexx-exec/tests/watchdog/mod.rs`: `run_bounded` spawns a thread, calls `run_program` with
layer 1 armed at `ROW_DEADLINE`, and waits on a channel for `ROW_ABANDON`. On a timeout it
synthesises an `Outcome` with `DEADLINE_EXIT` and a `rexx-watchdog: ` stderr line, which is how the
two layers are told apart afterwards.

Applied at every `run_program` call in `ir_dual.rs`, `assertions.rs`, `bif_assertions.rs` and
`corpus.rs`.

**A disconnect is not a timeout.** A panic inside `run_program` is resumed on the watchdog thread,
drops the sender and arrives as `Disconnected`; that is re-panicked rather than reported as a slow
row.

**`corpus.rs` reads a crate-side non-finish as structural**, the same way it already reads an
oracle-side one -- red in every mode rather than only under the gate, because a run that did not
finish produced no answer to compare. `watchdog::did_not_finish` is `ends_with` for layer 1 and
`starts_with` for layer 2, because `execute` appends its report *after* whatever the program wrote:
`a_run_that_traced_before_its_deadline_still_reads_as_unfinished` is the witness, and it was run
against a `starts_with` mutation and reddened at exactly that assertion.

---

## 3. Each layer's reach, stated as what it does not catch

Both statements live in the code -- `Deadline`'s doc and `watchdog/mod.rs`'s module doc.

**Layer 1 catches** a run that keeps executing clauses.

**Layer 1 does not catch:**

* a run parked on a wait that executes no further clause;
* a spin inside one builtin, or inside one clause's own evaluation -- the check runs when a clause
  *begins*, and a clause that never ends is never re-entered;
* the parse, which `execute` does before it builds an `Interp`.

The `UNINIT` sweep and the reply drain both run clauses and are *inside* the bound.

**Layer 2 catches** whatever layer 1 cannot see, because it is not looking at the interpreter.

**Neither layer survives a native stack overflow**, and one is reachable from Rexx -- section 11.
Layer 2 abandons a thread; it cannot outlive the process that thread takes with it.

**What layer 2 costs.** A Rust thread cannot be killed. The abandoned thread keeps running -- at
100% of a core for a spin, competing with the rest of the sweep for the life of the test binary --
and keeps its interpreter thread's 512 MiB stack reservation. **The process does still exit**: a
detached thread does not hold `main` open, and
`layer_two_returns_for_a_run_layer_one_is_not_watching` witnesses that by abandoning a spinning
thread inside a binary that then has to reach its own exit for the suite to report at all.

**Layer 2 is defensive and untested against a real case** -- section 9.

---

## 4. Where the check sits, and the alternative that is cheaper and wrong

The clause boundary is not the cheapest place. The controller asked whether the check could move off
the per-clause path, naming the resource-guard site and the loop back-edges. I built both and
measured them.

**Attempt one -- the cycles as I could enumerate them**: `run_repeating`'s own iteration, the
`Flow::Signal` transfer, the reply drain, and the `MAX_ACTIVATION_DEPTH` site. It read **-0.001% on
`emptyloop`-ir**, which looked like the answer and was not: the check was not running. Four of the
five deadline cases hung, killed at 25 s. A `do forever` reaches neither `run_repeating` nor the
compiled stream's own loop.

**Attempt two -- `Interp::loop_advance`**, which is the single funnel every repeating loop passes
however it is driven, plus the same `Flow::Signal` and reply-drain sites. Every deadline case
passed, and it is genuinely cheaper:

One interleaved run, both arms and `6656d5a4f` measured together, best of three:

```text
                  clause boundary        at the cycles
emptyloop  ir         +1.078%               +0.809%
emptyloop  tree       +1.375%               +0.688%
varlookup  ir         +0.691%               +0.346%
varlookup  tree       +0.857%               +0.172%
median of 16          +0.304%               +0.077%
```

**It is still wrong, and one program shows it.** `zs = 'interpret zs'` then `interpret zs` iterates
no loop, transfers by no `SIGNAL` and resumes no reply, so a bound honoured at the cycles never sees
it. Measured, both engines, 30 ms deadline:

| where the check is | `interpret zs` |
|---|---|
| clause boundary (shipped) | `DEADLINE_EXIT`, `rexx-exec: the run exceeded its deadline`, under 0.5 s |
| at the cycles | **SIGABRT** -- `thread 'rexx-interp' has overflowed its stack`, the test binary dies |

So the cycle enumeration is not merely unenforceable, it is **false**, and the failure mode is the
worst available: the process aborts and takes the whole test binary with it. That is the same defect
`clause.rs`'s module doc records four rounds of, reproduced on the first attempt in an afternoon.
"Every clause is counted" is a property the compiler checks; "every unbounded run passes through one
of these cycles" is a claim about control flow that nothing checks and that I got wrong.

**The choice is therefore between 1.376% and a claim I have measured to be false**, and the shipped
answer is the clause boundary. The alternative is recorded here with its numbers rather than left
implicit, and the row that discriminates the two is committed:
`every_unbounded_shape_this_crate_can_take_is_bounded`'s `INTERPRET recursion` entry, whose doc says
plainly that a regression there aborts the test binary rather than failing it.

**A note on the residual.** `emptyloop` is the pure clause loop and so the worst axis by
construction: 100 million instructions on the compiled stream and 150 million on the tree-walker
over 25 million passes carrying two clauses each -- two and three instructions per clause, which is
the `sub`-and-branch and the branch the caller keeps. Amortising further buys nothing: on a run with
no deadline `countdown_reached` is entered once per `NO_DEADLINE_SPACING` = 2^32-1 clauses, so
`CLAUSES_PER_CHECK` is not on the default path at all. The only route to literally zero is not
compiling the check into the shipped build, which would make every gated suite measure a binary
other than the one that ships.

## 5. What the default path pays

`instructions:u` over `bench-programs`, `--release`, each arm its own binary, interleaved, best of
three, against `6656d5a4f`. `before` is `460ec0f7ef...`, `shipped` is `fb8f984c20...`.

```text
program        engine          BASE            shipped          delta
emptyloop      ir         9,277,170,539    9,377,104,913      +1.077%
emptyloop      tree      10,902,204,040   11,052,196,741      +1.376%
varlookup      ir        16,511,174,830   16,625,181,533      +0.690%
varlookup      tree      33,269,040,193   33,554,408,491      +0.858%
arith          ir        12,571,034,591   12,583,812,472      +0.102%
arith          tree      13,760,894,751   13,764,926,063      +0.029%
compound       ir        10,810,542,525   10,840,571,734      +0.278%
compound       tree      14,850,740,110   14,925,857,817      +0.506%
strings        ir        21,582,904,422   21,618,965,281      +0.167%
strings        tree      32,607,703,253   32,724,811,229      +0.359%
dispatch       ir        31,754,554,429   31,794,835,219      +0.127%
dispatch       tree      33,384,748,501   33,494,687,653      +0.329%
dispatchclass  ir        27,714,158,177   27,738,139,350      +0.087%
dispatchclass  tree      27,986,138,234   28,042,085,633      +0.200%
alloc4c        ir         4,097,899,646    4,105,898,305      +0.195%
alloc4c        tree       5,796,912,748    5,820,043,342      +0.399%
```

**Median +0.304%, worst +1.376%.** (The section 4 table is a separate interleaved run and its
figures differ in the third digit; a column there must not be read against a row here.)

**The layout control, which is what makes the rest attributable.** An arm carrying the `Invocation`
and `Interp` fields and no check at all reads **+0.002%, +0.002%, -0.001%, +0.001%** on
`emptyloop`/`varlookup`/`compound`/`strings`. Struct layout is not in any of the numbers above.

**Two shapes cost more and were rejected on the number**, both measured the same way: widening the
clause unit itself to carry the failure (`emptyloop` ir +2.966%, `varlookup` ir +5.640%,
`compound` +2.544%, `strings` +2.392%), and one `Option` test with a `checked_sub` per clause
(+1.887%, +1.036%, +0.418%, +0.208%).

**What the 1.376% buys**: the one crate-side non-termination anyone has found becomes a red row
instead of a stalled gate, on a check the compiler will not let a future clause site skip.
The number is stated in `Deadline`'s own doc as well as here.

---

## 6. Corrections to the brief

Written into `watchdog-brief.md` as well, per the rule that a correction belongs where the next
reader will look.

* **`corpus.rs` is not one of the rayon sweeps.** `rayon` appears in `ir_dual.rs`, `assertions.rs`
  and `bif_assertions.rs` and in no other file under `crates/rexx-exec/tests/`; `corpus.rs` runs its
  331 programs serially. The bound is still worth having there -- a hang stalls the serial loop and
  so the gate -- but "stalls the whole `collect`" is the wrong reason for that one file.
* The brief's `lib.rs:6737` and `clause.rs:473` were close enough to find the sites; both have moved
  with this change and are not restated.

---

## 7. Control 1 -- the witness is live

Run against the shipped implementation, not an earlier one. The mutation deletes
`countdown_reached`'s expiry arm, so the bound never fires. Mutated test binary
`89c9bcce4c1c0fcb581cbd7cc163a7c0ec503451a70980e47cc71f4a43ed423f`, each case bounded so the control
could not stall the session:

| case | mutated | restored |
|---|---|---|
| `the_self_forward_ends_at_its_deadline_on_both_engines` | **hangs**, killed at 45 s, rc 137 | ok |
| `every_unbounded_shape_this_crate_can_take_is_bounded` | **fails** at 30 s, rc 101 -- layer 2 answered where layer 1 should have | ok |
| `a_deadline_a_run_finishes_inside_changes_nothing` | **still green** in 2 s | ok |

The third row is the negative control: the mutation did not simply break the file.

Restored from a `cp` copy, rebuilt, and the whole file re-run: **10 passed** in 29.84 s.

**A hazard worth recording, because it nearly produced a false restore.** After the restore the
binary at `target/debug/deps/deadline-895b35913fbe7771` still held the *mutated* build, and a
`sha256sum` of that path read unchanged. Cargo had rebuilt -- into
`deadline-b55bd35ee6804813`, a different metadata hash -- and the path I had memorised no longer
named the current binary. Take the binary path from cargo's own `Running tests/...` line, never from
a previous run's.

## 8. Control 2 -- no false positive

**The values chosen: `ROW_DEADLINE` = 60 s for layer 1, `ROW_ABANDON` = 90 s for layer 2.**

**What the slowest legitimate case actually takes.** `run_bounded` was temporarily instrumented to
time itself and print every row over 50 ms, and the four sweeps were run under the debug gate's own
conditions:

```
REXX_CORPUS_GATE=1 memcap 8G cargo test --test corpus --test assertions \
    --test bif_assertions --test ir_dual -p rexx-exec --no-fail-fast -- --nocapture
```

exit 0, all four green. **51,084 rows above 50 ms; median 154 ms, p99 196 ms, slowest 294 ms** -- an
assertion row. The corpus's own programs are below that.

Separately, before the harnesses were wired up: debug `rexx-run` over the 331 programs the corpus
differential runs, stdin at `/dev/null`, wall clock including process startup -- slowest **198 ms**
run one at a time, **299 ms** with 32 running at once (`corpus/lang/class_inherit_recursive.rex`
and `corpus/lang/operator_frame_stem_plus.rex` respectively).

So the deadline sits about **200 times** the slowest case anyone has measured. The instrumentation
was removed before the gates; it is not in the committed file.

The `500 ms` deadline `deadline.rs`'s own cases use is a different number for a different job -- it
keeps that file's runtime down, and every program it is applied to is written to not terminate.

## 9. Control 3 -- layer 2 is reached, and what that does and does not prove

`layer_two_returns_for_a_run_layer_one_is_not_watching` reaches layer 2's arm and asserts the answer
came from layer 2 (`rexx-watchdog: `) rather than layer 1.

**It is not a case layer 1 cannot see.** It is the self-forward with no deadline set. No crate-side
program is known that layer 1 cannot see, so **layer 2 is proved as a mechanism and is untested
against a subject of its own.** That is written into `watchdog/mod.rs`'s own doc in the same terms.

The candidate for a real subject -- a parked shape -- was looked for and not found: the parked shape
the oracle has, `GUARD ... WHEN` with a false expression (`oracle-crashes.txt` entry 7), is refused
here at `Loud::guard_when_false` before it can park.

## 10. "A sweep containing that program reddens that row and completes"

`a_sweep_containing_the_self_forward_reddens_one_row_and_completes` runs a three-element population
through `rayon`'s `par_iter().collect()` -- the same construct and the same `run_bounded_with` code
path the four sweeps use -- with the self-forward in the middle. The hang's row carries
`DEADLINE_EXIT` and `DEADLINE_REPORT`; its two neighbours' answers are asserted, so a `collect` that
gave up would fail rather than pass with a row missing.

**I did not inject the program into a committed sweep population, and that is deliberate.** Both
populations that would take it are also run against the oracle: `corpus/`'s programs by
`tests/corpus.rs`, and every `ir_dual_cases` stanza by `tests/ir_dual_oracle.rs`, whose whole
purpose is to re-measure those recordings against the running interpreter. The self-forward family
is in `corpus/oracle-crashes.txt` because it takes the oracle's C++ stack out at rc 139. Adding it
to either population would put it in front of the oracle on the next run, and a temporary local
injection would leave that landmine one `git add` away.

---

## 11. A finding outside this task: `INTERPRET` recursion aborts the process

Found while testing the placement, and it is not the watchdog's to fix.

```rexx
zs = 'interpret zs'
interpret zs
```

Measured 2026-09-02 on the crate alone, both engines, **no deadline set**: stdout empty, stderr
`thread 'rexx-interp' (...) has overflowed its stack` / `fatal runtime error: stack overflow,
aborting`, **rc 134**. An `INTERPRET` fragment grows the native stack without growing
`activation_depth`, so `MAX_ACTIVATION_DEPTH` never fires -- structurally the same blind spot as the
self-forward's, with a worse ending. `on_interpreter_thread`'s own doc already names a stack
overflow as the one failure it cannot report; this is a Rexx program that reaches it.

**I did not run this against the oracle.** It is not in `oracle-crashes.txt` and the oracle is
expected to answer it with a clean `Error 5`, but establishing that is a differential measurement
this task did not own. Whoever picks it up should note it is an unbounded recursion and bound the
oracle run.

Under a deadline the crate answers cleanly, which is why the shape is committed as the row that
pins where the check has to sit.

---

## 12. What I did not do

* **`rexx-run` has no way to set a deadline.** The library takes one per run; the binary does not
  expose it, by command line or by environment variable. That was outside the brief and a probe on
  the command line still needs `timeout -s KILL`. If it is wanted, `bin/rexx-run.rs`'s
  `engine_from_environment` is the shape it would take, for the reason that function's own doc
  gives: one program per process is the one case where a variable is enough.
* **Layer 2 is applied at the four harnesses the brief named and nowhere else.** Every other
  in-process caller of `run_program` under `crates/rexx-exec/tests/`, and every one inside
  `src/**/tests.rs`, is unbounded. `tests/gate_tables/mod.rs` is the notable one: the gate-table
  probes run in process, so a hang there would stall the phase gate exactly the way one in a sweep
  would have. `collect_stress.rs` is a second, and it needs its own number rather than this one --
  it runs under collect-on-every-allocation, where a legitimate program is orders of magnitude
  slower and `ROW_DEADLINE` would be a false positive.
* **I did not put the self-forward into a committed sweep population**, section 10.
* **The parse is not bounded by either layer.**
* **Layer 2 has no real subject**, section 9.
* **I did not fix the `INTERPRET` stack overflow**, section 11, and did not measure the oracle's
  answer to it.
* **No `unsafe`**, and none was wanted.
