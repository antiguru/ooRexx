# Task 3 report -- derived concurrency test list and pinned-park counter

Base 1cd896327. Status: DONE_WITH_CONCERNS.

## Commits

(pending)

## Step 1: derivation

Committed as `rust/crates/rexx-exec/tests/concurrency_tests.rs` (`derive`), held equal to the
`## Derived list` and `## Exclusions` blocks of `docs/superpowers/plans/phase-6-pinning.md` by
`the_derived_list_is_the_committed_one`; commands in that document's `## Commands`.
`the_derived_list_holds_the_groups_the_spec_names` asserts every group spec section 9 names is in
the list. Exclusions: rxapi persistent state (none matched), RexxQueue (Phase 10), `extensions/`
and `samples/` (Phase 10 recompiles), each with its reason in the block.

Choices a reader should check:
- Helper reach is by name within the group file: a `::METHOD`/`::ROUTINE`/`::RESOURCE` whose name
  appears as a word, and a class named reaches only its `INIT`. The first draft let a named class
  reach all its methods; that pulled every `METHOD.testGroup` test in through `methodTest`'s
  replying methods, so it was narrowed.
- `Message~reply`/`~replyWith` are matched beside `~start`/`~startWith` (not in the spec's list;
  they start a thread the same way). The rows carrying it show it in their feature column.
- GUARD means the instruction, not the `GUARDED` method option.

## Negative control

Prediction (written before running): a scratch root holding only a copy of
`base/bif/ABS.testGroup` (not in the derived list) with `::method test_negative_control` / `reply`
appended derives exactly one row, `base/bif/ABS.testGroup TEST_NEGATIVE_CONTROL REPLY`, and no
exclusions; the unmodified copy derives no row.

Result: as predicted. Command, from `rust/`, with `S` the scratch directory:

```
REXX_CONCURRENCY_ROOT=$S/nc-reply cargo test -p rexx-exec --test concurrency_tests \
  the_derived_list_is_the_committed_one -- --nocapture
```

`nc-plain` printed `derived:` and `excluded:` with no rows; `nc-reply` printed
`base/bif/ABS.testGroup TEST_NEGATIVE_CONTROL REPLY` and no exclusions.

## Step 2: counter

Feature `pinning` of `rexx-exec` (`src/pinning.rs`), off by default. Macros `pinned!`,
`pin_enter!`/`pin_leave!` and `park_point!` expand to their body or to nothing without it; with it,
a `Pinning` field on `Interp` (a `RefCell` stack of frame kinds and a map of arrivals) records at
each park point the frames on the stack, and `Outcome::pinning` carries the report. The
per-kind site tables are in `phase-6-pinning.md`'s `## Counter`.

Beyond the brief's list: a `Native` frame, pushed around every native and implemented-external
method except the natives spec 2.1 makes resumable (`SEND`, `SENDWITH`, `START`, `STARTWITH`,
`NEW`, `CALL`, `CALLWITH`), because spec P6-4 counts any native re-entry as pinning. The frame
names the message.

Self-tests (feature on): `measured::a_park_records_the_pinned_frames_above_it` (top-level
SysSleep has no frame; INTERPRET, sortWith comparator and GUARD WHEN are seen) and
`measured::an_unimplemented_wait_is_a_park_point` (Message~wait, EventSemaphore~wait,
SysWaitEventSem). The measurement asserts every run ends with no frame pushed.

## .text hashes

`cargo build --release -p rexx-exec --bin rexx-run` in fresh target dirs, 20 `Compiling` lines each;
`objcopy -O binary --only-section=.text` then `sha256sum`:

- base 1cd896327: `ad20a4d1a1cc2f1f33c315ad126fa2c6ee9e4ef7dddf70e1102c3677708a361e`
- this change, feature off: `ad20a4d1a1cc2f1f33c315ad126fa2c6ee9e4ef7dddf70e1102c3677708a361e`
- final tree before commit, feature off (rebuilt after the last `pinning.rs` edit, 1 `Compiling` line): `ad20a4d1a1cc2f1f33c315ad126fa2c6ee9e4ef7dddf70e1102c3677708a361e`
- control, same tree, `--features pinning`: `cf95704d0a85fe4c56c11ea17d328945b02f762bf4c967ded3989226d0b5e8bf`

## Step 3: measured table

`phase-6-pinning.md` `## Measured`: summary by park kind and frame stack, then per test (outcome
and each park row). Command in that document's `## Commands`; run took 75 s in release.

Read of the summary: every arrival's frame stack is one of `TreeSend`, `TreeEval`, `OpExec`
(which S1 removes) except `MessageResult`, whose stack holds `Native ~RESULT` -- that native is
itself the park point, not a frame above it, so it is an artefact of the Native frame being pushed
around the park-point native. No sort comparator, conversion, UNKNOWN, FORWARD, operator, trap,
native-API, wrapper, loop or INTERPRET frame appears at any arrival in these tests.

The first run left `ooTest.frm`'s `rxfuncquery` probes in, so most runs ended refused at
`RXFUNCQUERY` (Phase 10) in the summary; the committed table is the second run with those probes
removed as `api_group_tests.rs` does. That run also exposed the semaphore mapping's wrong method
name (`MutexSemaphore~REQUEST`; the method is `ACQUIRE`), fixed with a self-test.

## Gates

(pending)

## Concerns

- `MessageResult` rows show `Native ~RESULT` as a frame: the Native frame is pushed around every
  native, including the park-point natives themselves. Task 11 should read that frame as not
  pinning, or the push should skip the park-point natives.
- Parks inside child processes (`bug2003_guard_when` runs `rexx` as a command) are not seen.
- Many tests refuse before their concurrent part runs (Message START/REPLY family, timers, GUARD
  WHEN that must wait), so the table under-counts what those tests reach once S1/S2 land.
- The guarded-method reservation at a send is not a park point here (the brief's list omits it).
- The derivation is lexical: a message name built at run time (`.message~new(o, 'START')`) or a
  helper reached through another file is not seen.

## Fix round 1

Predictions, written before running:
- Negative control re-run: `nc-reply` (ABS plus a replying `test_negative_control`) derives exactly
  `base/bif/ABS.testGroup TEST_NEGATIVE_CONTROL REPLY`; `nc-plain` derives nothing.
- ACTIVATE control: a scratch root holding only `base/class/MethodArgs.testGroup` derives every
  `test*` method of its test class, each row's features including `Alarm` and `Ticker`, and no
  exclusion.

Results: both as predicted. `nc-plain` no rows; `nc-reply` exactly the one row; `nc-activate` the
ten `TEST_REQUEST_STRING_*` rows of `MethodArgs.testGroup` (its `::method test` count is ten by
`grep -ic '^ *::method *.\?test'`), each `Alarm,Ticker`, no exclusion. Same command as the first
control, `REXX_CONCURRENCY_ROOT=$S/<dir>`.

Changes:
- I1: `PinKind::native` answers `None` for `RESULT`, `WAIT`, `ACQUIRE` as well as the resumable
  natives. Self-test `measured::a_park_point_native_is_not_a_frame_above_itself`
  (`m~send; m~result` records MessageResult with no Native frame); it failed before the change
  (`[[TreeSend, Native("RESULT")]]`) and passes after.
- I2: a class method `ACTIVATE` or `INIT` (the `CLASS` option on its directive) is reached by every
  test of the file; the detector test carries one. `MethodArgs.testGroup` now derives.
- I3: `measured::a_park_under_each_frame_kind_records_it`, one probe per frame kind reachable
  from Rexx: Unknown, Forward, Operator, TrapHandler, LoopHeader, NestedLoop, Conversion,
  TreeEval, TreeSend, OpExec, Interpret, OutputWrapper, PullWrapper, TraceWrapper,
  RedirectWrapper (SortComparator and Native stay in `a_park_records_the_pinned_frames_above_it`).
  Not probed: StreamWrapper (a stream BIF resolves its name to the interpreter's own stream
  object), NativeApiCallback and LibraryEntry (need a native library).
- Minors: `## Counter` states the guarded-method reservation and child-process parks are not
  counted; a run with no in-process park reads "no park reached in-process".
- Re-measured; `## Measured` replaced. Every arrival's frames are now within TreeEval, TreeSend,
  OpExec (the summary line reads 0 beyond them).
- `.text` of `rexx-run`, feature off, at the fix tree before commit:
  `ad20a4d1a1cc2f1f33c315ad126fa2c6ee9e4ef7dddf70e1102c3677708a361e` (unchanged).

The controller's `p6-gates/gates.sh` G8 line passes two test names to `cargo test` before `--`,
which cargo rejects (`unexpected argument 'measured::an_'`). Gates here ran from a copy with the
filters after `--`: `... --test concurrency_tests -- measured::a_ measured::an_`.
