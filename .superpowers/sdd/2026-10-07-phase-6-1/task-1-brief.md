### Task 1: Activation scope

**Files:** `rust/crates/rexx-exec/src/activation.rs` (struct at `:254`, assert at `:497`,
`ActivationFlags`, `Inherited` at `:931`, constructors `new`, `nested` `:643`), `activity.rs` (remove
`locals` `:34`, `active_condition` `:120`, `current_case_text` `:158`, `debug_pause` `:228`,
`elapsed_anchor` `:267`, `pending_elapsed_reset` `:274`; rename `random_seed` `:258` to
`random_source`), `run/condition.rs` (`:365-369`, `:398`, `:506-616`, `:988`), `run/call.rs`
(`:901`, `:946`), `dispatch.rs` (`:3171` method activations), `builtin/datetime.rs` (`:777-788`,
`:857-876`), `builtin/numeric.rs` (`next_seed` `:560-582`), `lib.rs` (SETLOCAL push/pop `:2545-2570`,
roots near `:1893`), `run/condition.rs`, `run/select.rs`, `trace.rs`, `run/interpret.rs`,
`ir/drive.rs` (delete P89's lines `:2984-2986`), `scheduler/tests.rs:1058`, `corpus/phase-6-1.txt`
and its registrations, `corpus/oracle-crashes.txt` (entry 10b).

**Interfaces:** Produces on `Activation`: the elapsed anchor (`i64`, 0 unset) and `cached_clock` as
an `i64` with a sentinel; flag bits `ELAPSED_RESET` and `DEBUG_PAUSE` in `ActivationFlags`;
`current_case_text` inline; a cold box `Option<Box<ActivationCold>>` holding `condition:
Option<TrappedCondition>` (moved from its inline field, `activation.rs:364`, already per activation
and copied by `nested`), `active_condition: Option<ActiveCondition>` (moved from `activity.rs:120`; the
one slot `RAISE PROPAGATE` reads, kept apart from `condition` for the reason at
`run/condition.rs:365-369`), `random_seed` and `locals`, allocated on first use. `Activity` keeps one
generator, `random_source` (renamed from `random_seed`), lazily seeded by `initial_seed`; a top-level
activation's first unseeded draw seeds its own `random_seed` from it, advancing it once (the oracle's
`Activity::getRandomSeed`, `Activity.cpp:469-475`). Accessors `Interp::top_level_activation_mut()`
(walks `running` then `suspended` past `Entry::InternalCall` frames) for RANDOM and SETLOCAL. Task 6
reads `DEBUG_PAUSE`.

- [ ] **Step 1: Witnesses first.** Turn scout B's scope probes into corpus programs under
      `rust/corpus/lang/` (single-activity) and crate tests in `scheduler/tests.rs` (concurrent): `c2`,
      `c5` (with its second file in a `.d` directory), `int`, `rnd`, `sl1`, `sl6`, `cond1`, `cond2`,
      `ct2`, `ct4`, and a `debug_pause` witness with no flowed clause (a routine called from a typed
      line under `trace ?r` traces its own clauses; stdin through a `.stdin` file, the
      `corpus/lang/trace_debug.stdin` form). Time-printing probes are rewritten as predicates first.
      Concurrent: `reply` as predicates (`m before` is 0; `m after` below main's elapsed time) and
      `sl2`, each 5 runs on the oracle with counts in the test's doc comment. Every SETLOCAL witness
      restores at most once per process. Add Review Focus 2's depth-2000 witness, an unseeded-RANDOM predicate (a method
      called twice prints `random(1,1000000)` unseeded each time; the two values differ, on both
      engines), and Review Focus 1's REPLY probes, each run 5 times on the oracle: a method that (a)
      `call time 'r'`, replies, then prints a `time('e')` predicate; (b) seeds `random`, replies, draws
      again; (c) `setlocal`, replies, `endlocal` (`sl2`); (d) traps a condition with `CALL ON`, replies
      in the handler, `raise propagate` in the continuation; (e) `select case` with a REPLY inside a
      WHEN and an absorbed WHEN after it. The `debug_pause` witness: scout B's `dbgcall` shape with a
      routine (not an internal call) so no flowed clause is involved, its expected pause lines taken
      from the oracle. Run them: each fails as scout B recorded.
- [ ] **Step 2: Layout.** Move the fields as the Interfaces block says; check
      `size_of::<Activation>() == 512` builds before any behaviour change (a commit of its own with
      the program outputs unchanged).
- [ ] **Step 3: Rules.** Constructors: program, routine, method, external and started activations
      start fresh; `nested` (internal call, `CALL ON`) copies anchor, cached clock, reset flag, `condition` and
      `active_condition` from the caller (a `CALL ON` handler's activation gets the trapped condition
      as its `active_condition`); methods and routines start with neither; `DEBUG_PAUSE` is set only on
      the activation running a typed debug line, for that line's duration (`run/interpret.rs:256`,
      `trace.rs:57`), and is false for INTERPRET proper and every callee; INTERPRET and debug
      lines run in the current activation; REPLY moves the box. RANDOM and SETLOCAL use
      `top_level_activation_mut`. An activation ending with a non-empty SETLOCAL list restores its
      oldest entry. `SELECT CASE` stores its value on the activation for the construct's duration.
      Delete P89's copy lines; rewrite `a_reply_continuation_keeps_the_elapsed_clock_and_the_random_seed`
      to the oracle's rule (the continuation keeps the method's own clock and seed; main's are its own).
- [ ] **Step 4:** Witnesses pass and agree with the oracle. Extend `oracle-crashes.txt` entry 10b with
      `sl3`'s route (a routine's termination restore followed by main's ENDLOCAL). The per-task check.
- [ ] **Step 5: Perf.** `fibcall`, `fibfunc`, `dispatch`, `dispatchclass`, `sendloop` against the base.
      Record in `docs/superpowers/plans/phase-6-1-gate.md` (new, section `## Task 1`). Commit.

