# Task 4 review: the `Activity` (027ef469c..edacf562d)

**Spec compliance: ✅ compliant, judged against controller ruling P13 (running activity inline,
not `Box<Activity>`).** Task quality: **Approved**. The findings are all Minor, in the records
rather than the code.

## Spec Compliance

- ✅ **Step 1, classification.** Every field in the brief's activity list is on `Activity`, and
  `clause_countdown` and `deadline` stay on `Interp`. Spec section 5's list (`native_handles`,
  `native_spares`, `value_buffer`, `running`/`suspended`, `clause_state`, `frames`, `flat_top`,
  `pending_traps`, `active_condition`, `fragments`, `depth`, `call_context`, indents,
  `random_seed`) is covered. The report has a table with one reason per row.
- ✅ **Step 2, exhaustive destructures.** `Interp::object_roots` (`rust/crates/rexx-exec/src/lib.rs:2389`)
  destructures `activity` with no `..` and calls `activity.object_roots(out)`.
  `Activity::object_roots` (`rust/crates/rexx-exec/src/activity.rs`, the `let Activity { .. } = self`
  block) also has no `..`, and `pins` sits under the same cfg. Every rooted moved field is still
  rooted: `pending_additional`, `pending_result`, `pending_rc`, `reraised_object`, `failure_frame`,
  `failure_frames`, every `native_handles` frame, `pending_traps`' objects, and the context and
  condition objects of `running`/`suspended`. Every moved field that was `_` in the base is still
  `_`, and its comment moved with it. `stem_exposers`, `global_references`, `security_managers` and
  the three class tables are still rooted from `Interp`.
- ✅ **P13.** `Interp::activity: Activity` is inline (`lib.rs:1031`). No box or accessor was added.
- ✅ **Step 3/4.** The measurement is recorded and the gates ran (the report quotes `status.txt`; the
  controller verified it).
- ✅ **Global constraints.** No `unsafe`, no dependency, no `Op::Generic`, and no new process-global
  state. `Activity::new()` initialises every field to exactly its base `Interp::new()` value (I
  compared them field by field).

**`size_of::<Activity>()` = 1040 bytes** (`size_of::<Interp>()` = 6168). The largest single part is
`Option<ActiveCondition>` at 200 bytes. After it come `Option<FailureSite>` (64),
`CallContext` (56), `ClauseState` and `failure_origin` (32 each), and many 24-byte `Vec`s. This is
without the `pinning` feature, which adds a `RefCell<Vec>`.

How it was measured: a scratch `git archive edacf562d rust interpreter` copy, with a `#[cfg(test)]`
probe appended to `activity.rs` that printed `size_of`, run as
`CARGO_TARGET_DIR=$S/tgt cargo test -p rexx-exec --lib size_probe -- --nocapture` with
S=`scratchpad/t4rev`. The worktree was not touched.

So a P13 switch (`mem::swap` of the inline value with a boxed parked one) moves about 2 KB. If
switches ever show up in a profile, boxing `active_condition` (it is `None` except inside a
handler) is the obvious first cut.

## Strengths

- **Round 2 costs nothing measurable.** I checked the callgrind rows in `p6-t4/cg2/*.row`, and they
  match the table: rexxcps base r2 is 17,817,346,062 and r2's median is 17,817,329,203; `nop` is
  +131.
- **The rewrite is mechanical, and I checked that by running a comparison.** For each changed `.rs`
  file, I stripped `.activity.` and compared the whitespace-free token stream with the base. Thirty
  files are identical. The remaining eleven differ only in rustfmt trailing commas, reflowed method
  chains, and, in the comments, oracle citations `Activity::X` that my normalisation had rewritten.
  None of them reorders a statement or moves a borrow scope.
- **The `lib.rs` side is also mechanical.** I took every line removed from `lib.rs` and looked for
  it in `activity.rs` plus the lines added to `lib.rs`. Every removed line is there, except for
  intended `pub(crate)` prefixes and doc paths renamed from `Interp::` to `Activity::`. No comment
  was dropped or re-wrapped.
- **The pinning split is right.** `Pinning` keeps only the report and `PinStack` holds the frame
  stack. `park`/`take` read the running activity's stack. No `RefCell` borrow is held across the
  body of `pinned!`.

## Issues

### Critical
None.

### Important
None.

### Minor

1. **The report's perf summary is false** (`task-4-report.md`, the Performance section). It says
   round 2 has "every program within +/-131 Ir", but `phase-6-perf.md`'s own table shows
   `strings` -8,787, `arith` -5,148, `alloc` -4,364, `decrender` -1,718, `decloop` -1,633,
   `alloc4c` -1,339, `parse` -1,130 and `heapshape` -934. +131 is the largest *increase*, not
   the bound. The committed table is correct, and every figure is a decrease.
   Fix: delete the sentence, or say "no program above +131 Ir". The file is gitignored, so this
   affects only the controller.
2. **The perf doc's build recipe skips a step** (`docs/superpowers/plans/phase-6-perf.md`, `## Task 4`
   code block). `cargo build` writes to `$S/tgt/NAME/release/rexx-run`, but the next two lines hash
   `$S/bin/NAME/rexx-run`. The `mkdir -p $S/bin/NAME && cp ...` line from `p6-t4/build.sh` is not
   recorded, so the commands as written fail. Fix: add the `cp` line.
3. **Commit 8f0437be3's message says the fields "sit where they sat on Interp"**, which is false:
   they are now a nested struct at different offsets. The implementer raised this themselves
   (report concern 2). Since the commit is already made, the fix is no action; do not rewrite
   history for it.

The classification spot checks found nothing wrong under the second-concurrent-activity test:

- `trace_cache` and `debug_pause` mirror the running activation.
- `routing_trace` guards against a recursion on one delivery, and the oracle's equivalent is
  per-thread.
- `thread` is an 8-byte `Rc` handle, so libraries that clone it are unaffected, which spec 2.4
  requires.
- `requires_installing` is the oracle's `Activity::requiresTable`.
- `input_dispatch*` is one send on one activity.
- `max_depth`/`stack_*` measure the Rust stack of the chain that `depth` counts.
- `elapsed_anchor` sits on the activation in the oracle, which is closer to activity than to
  interpreter.
- `random_seed` is per activity in the oracle.

Nothing left on `Interp` holds one execution's state:

- The lent buffers fall back to an empty one when taken twice.
- `text_scratch` cannot be borrowed across `&mut Interp`.
- `deferred` is a work queue.
- `kept_holders` counts across calls.
- `untranslated` is keyed by program.

The root order handed to the collector changed (activity first). Mark order is not observable, and
GC ordering is a licensed divergence.

## Assessment

**Task quality: Approved.** The code matches the brief as amended by P13. It is behaviour-neutral
and costs nothing measurable. Minor 1 and 2 are one-line record fixes that can go in with the next
task.
