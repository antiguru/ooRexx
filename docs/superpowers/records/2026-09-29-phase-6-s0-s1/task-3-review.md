# Task 3 review -- derived concurrency test list and pinned-park counter

Base 1cd896327, head e076572fc. Reviewer ran, all read-only on the tree (target dir under the
scratchpad, since deleted):

- `cargo fmt --all --check`: exit 0.
- `cargo clippy -p rexx-exec --all-targets --features pinning -- -D warnings`: exit 0.
- `cargo test -p rexx-exec --features pinning --test concurrency_tests -- --skip pinned_parks_over_the_derived_list --skip support`: 5 passed (both `measured::` self-tests, the derivation check, the named-groups check, the detector test).
- Negative control re-run: a scratch root with `ABS.testGroup` plus `::method test_negative_control` / `reply`
  derives exactly `base/bif/ABS.testGroup TEST_NEGATIVE_CONTROL REPLY`. Reproduced.
- Miss hunt: `grep -rlaiE` over `ootest/ooRexx/**/*.testGroup` for REPLY, `~start`, GUARD, semaphore,
  `Sys*Sem`, `.alarm`, `.ticker`, SysSleep, `context~thread` and the TraceObject field names, diffed
  against the derived list's groups. Every grep-only file was a false positive (`~startsWith`,
  `.local~start1`, `EVENTSEMAPHORE` as data, excluded `extensions/`/`samples/`) except
  `base/class/MethodArgs.testGroup` (Important 2). A scratch root holding only that file derives no row.
- Not re-run: the `.text` hash build and the 75 s measurement. Risk 1 was judged by reading the macros
  and every hook site in the diff.

## Spec Compliance

✅ Step 1: derived per test method by a committed command (`tests/concurrency_tests.rs`, `derive`),
held equal to the committed blocks by `the_derived_list_is_the_committed_one`; exclusions carry a
reason each; every group spec section 9 names is asserted present; the negative control is predicted,
run and reproduced.

✅ Step 2: every pinned re-entry kind of spec 2.1 and the brief has a frame kind and a site; every
park point the brief lists has an arrival hook (plus timers and `Sys*Sem`); off-by-default feature
`pinning`; per-interpreter state (a field of `Interp`), no process-global state; no new dependency
(`rayon` was already a dev-dependency); no `unsafe`.

✅ Step 3: the measured table is committed per test and per kind, with refusals recorded.

⚠️ The measured summary's one line counting frames beyond TreeEval/TreeSend/OpExec (38) is made
entirely of an artefact (Important 1).

⚠️ One ooTest group that uses Alarm and Ticker is missing from the list (Important 2).

⚠️ No gate builds the feature (Important 3).

Named risks:

1. Feature-off: ✅. With the feature off, `pinned!` expands to `$body` unchanged, and `pin_enter!`,
   `pin_leave!` and `park_point!` expand to nothing. The `park_point!` arguments (the `match` on
   `guard.condition`, `ParkKind::unimplemented_method(&scope, name)`) are discarded tokens, so they
   are never evaluated. The `Interp` field, its initialiser, its destructure arm, `Outcome::pinning`
   and the `reset`/`take` calls are all `#[cfg(feature = "pinning")]`. `drive.rs` and `loops.rs` change
   only by macro wrapping. Nothing ungated was added to a hot path. This agrees with the report's
   identical `.text` hashes.
2. `Native ~RESULT`: fix now (Important 1).
3. Derivation: helper reach by name, class-reaches-only-INIT, `Message~reply`, GUARD as instruction
   are each sound and stated in the document. One miss, through a class `ACTIVATE` method
   (Important 2). The test-class detection (a subclass of `ooTestCase` or of a test class of the same
   file) misses nothing: the other parents seen (`collectionMethods.testGroup`,
   `SecurityManager.testGroup`, `Orderable.testGroup`, `WindowsEventLog.suite`) are in files with no
   feature use.
4. Changes outside pinning: ✅. Every hunk in `stream.rs`, `dispatch.rs`, `reqstr.rs`, `redirect.rs`,
   `condition.rs`, `loops.rs`, `drive.rs`, `command.rs`, `eval.rs`, `input.rs`, `route.rs`, `run.rs`,
   `call.rs`, `interpret.rs`, `sort.rs`, `library.rs`, `object_protocol.rs` and `rexxutil.rs` is a
   macro wrap or insertion. `?` stays outside every `pinned!` body, and no `pin_enter!`/`pin_leave!`
   pair in `library.rs` has an early return between them. `tests/support/oracle.rs`,
   `tests/watchdog/mod.rs` and `tests/method_bodies.rs` only add the cfg-gated `pinning:` field to
   the `Outcome` literals they build, which the feature-on build needs.
5. `refusal-sites.tsv`: ✅ only line shifts. After stripping `:NNN` from every changed line, each
   remaining line appears exactly twice, once removed and once added.
6. Feature never built by the gates: a finding (Important 3).

## Strengths

- The feature-off expansions are token-identical to the old code, so the `.text` identity follows
  from the macros themselves and does not depend on the optimiser.
- `Pinning` carries an `Option<PinKind>` stack, so a conditional push (`PinKind::native` answering
  `None`) still pops, and `unbalanced` is asserted over every measured run.
- The derivation's detector has its own witness test (quoted text, an assignment to `reply`, a
  helper routine, a class INIT). The negative control reproduces.
- The report's concerns are honest and specific.

## Issues

### Critical

None.

### Important

1. **`rust/crates/rexx-exec/src/pinning.rs:111-123` (`PinKind::native`), shown at
   `docs/superpowers/plans/phase-6-pinning.md:299-300,308`.** The `Native` frame is pushed around the
   park-point native itself (`Message~RESULT`, `native_message_result`), so every `MessageResult`
   arrival reads `... > Native ~RESULT`. The summary line "with a frame other than TreeEval, TreeSend
   or OpExec 38" is exactly those 24 + 14 `MessageResult` arrivals. So the only non-tree pinned
   figure in the baseline is 100% artefact, and a reader or Task 11 would take it as 38 genuinely
   pinned parks. A native that is itself the park point is not a frame between the driver and that
   point, and spec 2.1 lists these natives as parkable, not as re-entries. This table is the input
   S2's plan and Task 11 read, so a documented reading is the weaker option. **Fix:** in
   `PinKind::native`, answer `None` for the parkable natives that have native bodies (today `RESULT`;
   add `WAIT`/`ACQUIRE`/`REQUEST` in the same list when they gain bodies), and rename or comment the
   list to say it holds resumable and parkable natives. Then add a self-test:
   `.message~new('abc','LENGTH')~~send~result` records `MessageResult` with no `Native` frame. Finally re-run
   the measurement and regenerate `## Measured`. Cost: one list and a 75 s run.

2. **Derivation miss: `ootest/ooRexx/base/class/MethodArgs.testGroup`.** Its `::method activate class`
   (lines 58-95) builds `.Alarm~new(1, ...)` and `.Ticker~new(1, ...)` and then cancels them. Every
   test in the group reads the instances it made (`test_*` → `checkRequestString` → `allInstances`).
   So every test depends on Alarm and Ticker working, and it reaches the timer entries that the
   `Ticker` rows of the table refuse at (`ticker_createTimer`, Phase 6). The name-reach rule cannot
   see it: `ACTIVATE` runs when the class is activated and no test names it. Reproduced: a root
   holding only that file derives no row and no exclusion. The spec's wording ("a helper it calls")
   arguably leaves it out, but the criterion is an exit criterion, and this group can pass only once
   Phase 6's timers work. **Fix:** treat a class method named `ACTIVATE` (and `INIT` with the `CLASS`
   option) as reached by every test of the file. Add it to the detector test, regenerate the list and
   re-measure. If it is ruled out instead, commit it as an exclusion row with its reason.

3. **Nothing committed builds `--features pinning`** (plan
   `docs/superpowers/plans/2026-09-29-phase-6-s0-s1.md:53-56` gates:
   `cargo clippy --workspace --all-targets` and `cargo test --workspace`, both without the feature).
   The measured module, both self-tests and every feature-on arm of the macros compile only when
   someone passes the flag by hand. The controller resolutions say later tasks rewrite the driver,
   calls, sends and loops and "must keep the hooks working". Under today's gates a hook that stops
   compiling, or a site that loses its wrap, goes unnoticed until Task 11 tries to re-measure. At
   e076572fc it does build and pass (runs above). **Cheapest fix:** add two gate lines to the plan's
   Global Constraints: `cargo clippy -p rexx-exec --all-targets --features pinning -- -D warnings` and
   `cargo test -p rexx-exec --features pinning --test concurrency_tests measured::a_ measured::an_`
   (0.3 s after the build). The self-tests cover only Interpret, SortComparator, Native and the park
   kinds. A frame kind whose site a later task deletes is still invisible. Extending
   `a_park_records_the_pinned_frames_above_it` with one probe per frame kind that is reachable from
   Rexx (UNKNOWN, FORWARD, an operator method, a trap handler, a `WHILE` calling a method, a nested
   `DO`) would close that.

### Minor

1. **Guarded-method reservation is not a park point.** Spec 2.1 names it one ("parks in a send's
   begin half"); the brief's list omits it, and the report says so. The document's `## Counter` does
   not record that it is left out, so a reader of the table cannot tell. **Fix:** either hook it
   (recommended, the same one-line `park_point!` wherever a guarded method's scope lock is taken) or
   add one row to the park table saying it is not counted.

2. **Parks reached in child processes** (`bug2003_guard_when` runs `rexx` as a command; its row reads
   `error, rc 2 | none`) are not seen. The report says so, but the document does not. A `none` row for
   a GUARD WHEN regression test reads as "reaches no park". **Fix:** note beside that row, or in the
   `## Measured` preamble, that parks in child processes are not counted.

No false or future-describing sentences found in the added comments, `Cargo.toml`, or
`phase-6-pinning.md`. The document holds only commands, the rule the derivation applies, the derived
list, exclusions, site tables and measured data. There are no em-dashes (`grep -c '—'`: 0 in all three
new files).

## Assessment

Task quality: **Needs fixes.** Fix Important 1 and 2, then re-measure, since both change the
committed baseline. Important 3 is a gate-line addition, the controller's to make in the plan. The
hooks themselves are correct and fully compiled out with the feature off.
