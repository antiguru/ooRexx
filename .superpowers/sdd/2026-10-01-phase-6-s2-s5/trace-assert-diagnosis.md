# Trace-analysis debug assertion in whole_groups: diagnosis

Branch `plan/rust-rewrite`, HEAD 2496c5f72. Diagnosis only: no tracked file
changed. Binaries were built into their own target dirs under
`/tmp/claude-1000/p6-trace/` (HEAD from the worktree, base and the fix from
`git archive` copies, touched after extraction; every build below printed
`Compiling rexx-exec`).

Symptom (`/tmp/claude-1000/p6-lead/dbg-whole.log`, run dir
`rust/target/tmp/whole-groups-1656614-40/run/r00/`):

```
thread 'rexx-interp' panicked at crates/rexx-exec/src/ir/drive.rs:3543:29:
assertion `left == right` failed: the trace analysis answered ChunkTrace(0) for instruction 12, and the setting in force when it ran is not that one
  left: ChunkTrace(0)
 right: ChunkTrace(51)
```

r00 is the TRACE group: its only testGroup that differs from `ootest/` is
`base/keyword/TRACE.testGroup`, where line 1175 is
`::method SKIPPED_test_trace_label_with_forward`. Rerunning that whole group
through a debug HEAD `rexx-run` (`/tmp/claude-1000/p6-trace/scripts/one.sh`,
laid out like `whole-groups/alone.sh`, stdin `/dev/null`) gives the same
panic, rc 101.

## 1. The test

I ran every `::method test...` in the r00 copy alone with `-t NAME` through
the debug build, and printed every run that did not exit 0:

```
$ for t in $(grep -io '^::method *"\?test[A-Za-z0-9_?]*' $G | awk '{print $2}' | tr -d '"'); do
    bash one.sh $DEBUG_REXX_RUN $S $G "$t"; done | grep -v 'rc=0 $'
test_trace_optional_additional rc=120
test_trace_value_missing rc=120
test_trace_drop rc=1
...
test_trace_?i rc=1
test_trace_numeric_debug rc=101 trace analysis answered ChunkTrace(0) for instruction 12
```

**TEST_TRACE_NUMERIC_DEBUG** is the only one that trips it. Instruction 12 of
that method body is `self~assertTraceOutput(...)`, the clause after
`trace off`. The body is: `t = ...` (0), `.DebugInput~...` (1), `start = .line;`
(2), `trace ?a` (3), seven `nop`s (4-10), `trace off` (11), then the assert (12).

## 2. Minimal reproduction

`m2.rex`:

```
trace ?a
nop
trace off
say 'after'
```

```
$ $DEBUG_REXX_RUN m2.rex </dev/null
       +++ "LINUX COMMAND /tmp/claude-1000/p6-trace/m2.rex"
     2 *-* nop
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++

thread 'rexx-interp' panicked at crates/rexx-exec/src/ir/drive.rs:3528:29:
assertion `left == right` failed: the trace analysis answered ChunkTrace(0) for instruction 3, and the setting in force when it ran is not that one
  left: ChunkTrace(0)
 right: ChunkTrace(51)
rc=101
```

It does not depend on stdin: the same panic fires with `</dev/null`, with a single
blank line, and with each of `trace 0`, `trace 1`, `trace 2`, `trace -1` and
`trace value 1 * 1` as the only input line. `trace value '?a'` in place of
`trace ?a` (`m4.rex`) panics the same way.

## 3. Oracle against our release build

For `m2.rex` there is **no observable divergence**, only the tripwire. With
stdin from `/dev/null`, stdout, stderr and rc are each identical, in 4 runs out of 4:

```
ours rc=0 / oracle rc=0
stdout identical
stderr identical
```

The oracle's stderr ends `     3 *-* trace off` / `     4 *-* say 'after'`. It
keeps tracing after `trace off`, because a TRACE instruction is ignored under
interactive debug.

**The same defect diverges observably once the setting traces values.**
`m11.rex`:

```
trace ?i
nop
trace off
say 1+2
```

```
oracle rc=0, stdout "3", stderr:
       +++ "LINUX COMMAND /tmp/claude-1000/p6-trace/m11.rex"
     2 *-* nop
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
     3 *-* trace off
     4 *-* say 1+2
       >L>   "1"
       >L>   "2"
       >O>   "+" => "3"
       >>>   "3"
ours (release) rc=0, stdout "3", stderr: the same without the >L>, >L> and >O> lines
```

`trace value '?i'` (`m10.rex`) gives the same diff. Our clause echo is right
because it falls back to the run-time gate (`stale`, drive.rs:1791). The value-echo
ops are not: emission dropped them because the analysis claimed `OFF` at
`say 1+2`.

TEST_TRACE_NUMERIC_DEBUG's own release failure (`alone.txt` line 37) has a
**different cause**. With the fix below it still fails. Its actual output
traces `nop /* 3 */` and `nop /* 5 */`, so the lines that
`.DebugInput~destination(...)` supplies never reach the pauses. That fits
`debug_pause_after_clause` (run/interpret.rs:202), which reads through
`input_line` (input.rs:330, stdin). I did not investigate this further.

**A second divergence, separate from the assert, found on the way.** We do not
pause after a clause whose `Op::Exec` returns `ExecOutcome::Done(flow)`,
which includes TRACE and NUMERIC. drive.rs has one pause site (1905), on the
hot exit of `clause_region!`. The `RegionEnd::Flowed` path (drive.rs:1383,
1929) settles the flow without asking it. Input `say 'p1'`, blank, `say 'p2'`, blank, `say 'p3'`, blank:

```
m7 (trace ?a / say 'c1' / trace off / say 'c3' / say 'c4')
  ours:   c1 p1 c3 p2 c4 p3
  oracle: c1 p1 p2 c3 p3 c4
m8 (same with numeric digits 9 in place of trace off): identical pair
m9 (all SAYs): ours == oracle, c1 p1 c2 p2 c3 p3 c4
```

C++ pauses at the end of each instruction's `execute`; TraceInstruction.cpp:162
and :186 do it for an ignored TRACE. Base 1754a3b5a also gives
`c1 p1 c3 p2 c4 p3` for m7.

## 4. Root cause

`crates/rexx-exec/src/ir/trace_flow.rs:357`, in `apply`:

```rust
TraceEvent::Sets(trace) => Setting::Known(trace),
```

A literal `TRACE` produces its own setting whatever pool it arrives in. At run
time it does not when the setting in force is interactive debug.
`run/settings.rs:65`, `apply_trace_request`:
`if self.trace_mode().debug && !self.activity.debug_pause { return; }`,
which is TraceInstruction.cpp:153-163.

The analysis believes it has excluded debug. `literal_trace_event`
(trace/format.rs:348) answers `TraceEvent::Unknown` for every debug setting.
`analyse` (trace_flow.rs:247) refuses a debugging *entry*. So no `Known`
pool is ever debug. But debug can begin **inside** the body, at an
`Unknown` event: `trace ?a`, `trace value '?a'`, `trace('?a')`, or
`INTERPRET`. Its pool is `Setting::Unknown`. The next literal `TRACE` turns
that `Unknown` into `Known(its text)`, a claim that is false whenever the
`Unknown` was debug. In m2: `trace ?a` gives `Unknown` and `trace off` gives
`Known(ChunkTrace(0))`, but at run time `trace off` is ignored and `say`
runs under `?A`, which is `ChunkTrace(51)`. The tripwire is drive.rs:1777-1786
in `open_clause!`, reached from line 3528 (`Op::Clause`) and line 3543
(`Op::CallingClause`, the test's assert clause).

`Setting::Unknown` conflates two bottoms: "paths disagree, but each is a
source-fixed non-debug setting", and "something set a setting the source does
not fix, possibly debug". Only the second makes a following `TRACE`
ineffective.

## 5. Phase 6 or pre-existing

Pre-existing, and not a concurrency defect. `trace_flow.rs` was added at
7a28f68e6, an ancestor of base, and is unchanged between 1754a3b5a and HEAD.
`git diff 1754a3b5a HEAD -- trace_flow.rs` is empty, and `settings.rs`
differs only in the `self.activity.` field moves.

A base build reproduces it. The base build used `git archive 1754a3b5a rust interpreter extensions` into
`/tmp/claude-1000/p6-trace/base`, `find ... -exec touch {} +`, and
`CARGO_TARGET_DIR=/tmp/claude-1000/p6-trace/base-target`:

```
   Compiling rexx-exec v0.1.0 (/tmp/claude-1000/p6-trace/base/rust/crates/rexx-exec)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.88s
   Compiling rexx-exec v0.1.0 (/tmp/claude-1000/p6-trace/base/rust/crates/rexx-exec)
    Finished `release` profile [optimized + debuginfo] target(s) in 1m 02s

$ base-target/debug/rexx-run m2.rex </dev/null
thread 'rexx-interp' panicked at crates/rexx-exec/src/ir/drive.rs:616:25:
assertion `left == right` failed: the trace analysis answered ChunkTrace(0) for instruction 3, and the setting in force when it ran is not that one
  left: ChunkTrace(0)
 right: ChunkTrace(51)
rc=101
m10 base release stderr == HEAD release stderr
m11 base release stderr == HEAD release stderr
```

The gate only met it now because whole_groups runs the TRACE group in a debug
build. The release gate compiles the `debug_assert` out.

## 6. Proposed fix

Split the bottom so that a `TRACE` is obeyed after a disagreement and ignored
after a possible debug. The change goes only in `trace_flow.rs`: a `Disagrees` variant from
the meet of two different `Known`s, `Unknown` absorbing in the meet, and
`Sets` from an `Unknown` pool staying `Unknown`. Consumers are unaffected:
drive.rs:1778 and compile.rs:1908 read only `Known`, and `echoes_*` treat
`Disagrees` like `Unknown`. Full diff, tested in
`/tmp/claude-1000/p6-trace/fix` (copy of HEAD):

```diff
@@ pub(crate) enum Setting {
     Known(ChunkTrace),
-    /// The lattice's bottom: two paths here disagree, or one of them ran
-    /// something whose setting the source does not fix.
+    /// Two paths here leave settings the source fixes and that disagree.
+    /// Neither is interactive debug, so a `TRACE` reached from here is obeyed.
+    Disagrees,
+    /// The lattice's bottom: a path here ran something whose setting the
+    /// source does not fix, which may be interactive debug, under which a
+    /// `TRACE` instruction is ignored.
     Unknown,
@@ fn meet
             (Setting::Known(one), Setting::Known(two)) if one == two => Setting::Known(one),
-            _ => Setting::Unknown,
+            (Setting::Unknown, _) | (_, Setting::Unknown) => Setting::Unknown,
+            _ => Setting::Disagrees,
@@ echoes_values / echoes_keyword (both)
-            Setting::Unreached | Setting::Unknown => true,
+            Setting::Unreached | Setting::Disagrees | Setting::Unknown => true,
@@ fn apply
         TraceEvent::Keeps => pool,
+        // **A `TRACE` instruction is ignored under interactive debug**
+        // (`Interp::apply_trace_request`), and only `Unknown` may be debug:
+        // `literal_trace_event` answers `Unknown` for every debug setting and
+        // `analyse` refuses a debugging entry.
+        TraceEvent::Sets(_) if pool == Setting::Unknown => Setting::Unknown,
         TraceEvent::Sets(trace) => Setting::Known(trace),
```

plus a new unit test, and `Unknown` -> `Disagrees` in the expectations of three
existing tests (`a_trace_i_inside_a_loop_settles_the_body_and_not_its_ends`,
`a_leave_carries_the_setting_from_before_the_trace_after_it`,
`a_label_answers_the_meet_over_the_whole_body`). Their `Known` entries are
unchanged, so no precision is lost:

```rust
/// A `TRACE` after something that may have entered interactive debug is
/// ignored if it did, so it settles nothing.
#[test]
fn a_trace_after_an_unknown_setting_settles_nothing() {
    assert_eq!(
        settings_of(b"trace value '?a'\nnop\ntrace off\nsay 'b'", TraceMode::NORMAL),
        vec![off(), Setting::Unknown, Setting::Unknown, Setting::Unknown]
    );
}
```

The scratch copy has been deleted, so the hunks above are the only record of
the diff. rustfmt has not been run on it.

Two smaller fixes were tried and rejected, both in the same copy:

- `Sets` from any `Unknown` pool stays `Unknown`, with no new variant. This is
  sound, but it fails `a_trace_i_inside_a_loop_settles_the_body_and_not_its_ends`:
  the hot loop body drops from `Known(I)` to `Unknown`, because the header's
  bottom there is a disagreement and not debug.
- Refusing any body that has both an `Unknown` and a `Sets` event fails
  `interpret_leaves_what_follows_it_unknown` and
  `the_trace_builtin_leaves_what_follows_it_unknown`, which lose `Known`
  answers *before* the `Unknown`.

Verification of the proposed fix:

```
$ cargo test -p rexx-exec --lib trace          (fix copy)
test result: ok. 46 passed; 0 failed; ... 962 filtered out
new test with the one `Sets(_) if pool == Unknown` line deleted:
  left: [Known(ChunkTrace(0)), Unknown, Unknown, Known(ChunkTrace(0))]
 right: [Known(ChunkTrace(0)), Unknown, Unknown, Unknown]
test result: FAILED. 0 passed; 1 failed
m2 fix-debug rc=0 0 asserts    m2 fix-release == oracle (rc 0)
m4 fix-debug rc=0 0 asserts    m4 fix-release == oracle (rc 0)
m10 fix-debug rc=0 0 asserts   m10 fix-release == oracle (rc 0)
m11 fix-debug rc=0 0 asserts   m11 fix-release == oracle (rc 0)
r00 TRACE group whole, fix debug build: rc=120, 0 "trace analysis" lines;
  its last stderr line is "rexx-exec: test does not parse here: 21.906 ...",
  the same as HEAD's release build on the same copy
```

Not run: the `whole_groups::each` gate itself, the full `cargo test`, clippy,
and the perf benches. The change touches only bodies that contain a `TRACE`.
TEST_TRACE_NUMERIC_DEBUG still fails with the fix (Failures: 1), for the
`.DebugInput` reason in section 3. The missing-pause divergence in section 3 is
a separate fix in `clause_region!`'s `Flowed` path, and is not proposed here.
