# Task 7 review: activation-relative slot positions, slots per activation segment

Reviewed: diff 8b253ef97..863d147d1 (code at a0a82f2b7), brief, report, spec 2.1/5/9(8).

**Spec compliance: ✅ with two ⚠️.** Every brief step is done and witnessed. Two controller
requirements hold in debug builds only: "cannot be resolved against another activation's segment
without a check" (the serial check is a `debug_assert_eq!`), and the native frame token is an
identity only modulo 2^32. Neither is violated by any caller today. Both have cheap fixes (I1, I2).

## Spec Compliance

- ✅ Step 1 failing test: `tests/outer_context.rs` runs o9b, o9c and the new `outer_context/o9d.rex`
  against the live oracle, plainly and under collect-every-alloc. Before: rc 120 with the Phase 6
  refusal (quoted in the report). After: `a_kept_outer_context_reaches_its_callers_variables ... ok`
  in the gate logs at a0a82f2b7. G4 (`g4-test-release.txt:3038`) and G6 (`g6-test-debug.txt:3041`)
  both ran with `REXX_CORPUS_GATE=1` (`p6-gates/gates.sh:17,22`), so the test was not skipped.
- ✅ Oracle expectations: the comparison is live (gate-only), not a committed expected file, which
  is how this test file already worked. The report records the oracle command and its full o9d
  stdout.
- ✅ Step 2: `SlotFrame { depth, serial }`, `Target::Slot { frame, index } | Cell`, and one `Segment`
  record per frame holding `start`. `grow_slots_of(frame, n)` works below the top.
- ✅ The debug "targets descend" check is kept as `(to.depth, to_index) < (frame.depth, index)`
  (`roots.rs:557-566`). Within one activity, depth order is push order, which is absolute-position
  order, so this is the old check's exact equivalent and still guarantees that the alias chase
  terminates. No test fires it (M2).
- ✅ Step 3: the refusal is removed, with its message, `unbound_outer`, `refuse_unbound_outer`,
  `governing`, `Activity::outer_caller` and the outer-only branches. The exclusions paragraph
  "STILL DIFFERS ... OWNER: Phase 6" is replaced by a FIXED 2026-09-30 note. That note is true (the
  test above is green) and describes no future work. No `(Phase 6)` text for this residual remains
  in `rust/crates` or in the exclusions file.
- ✅ `refusal-sites.tsv`: unchanged and consistent. At the base, the refusal was a
  `crate::Loud { message }` struct literal. `refusal_sites.rs` scans constructors (`:195`, `:302`,
  `:440`), so this refusal never had a row, and the base TSV has none mentioning surface.rs,
  outer or unbound. See M4 for what this implies for criterion 8.
- ✅ Callgrind against the base: `phase-6-perf.md` "## Task 7", with a recipe, sha256 of every
  binary and a median of three rounds. At round 2, every program is inside the budget.
- ⚠️ Serial check is debug-only (I1). ⚠️ Native id is a wrapping u32 (I2).

## Strengths

- Only the record holds `start`. Every lasting handle is (depth, serial) plus an offset, so the
  REPLY re-push and a future move rewrite one field.
- The per-frame-`Vec` design was measured (assign +6.19%) and dropped, not argued away. Round 2 is
  net faster on every program except extcall (+0.15%).
- The rexx-core tests were mutation-checked, three mutations with the tests they redden named.
  Miri covers five selections at both code commits, with `Compiling` lines checked.
- o9d.rex covers more than the brief asks for: stem, compound through a resolved tail and through
  an unbound tail, drop, `GetAllContextVariables`, a later `INTERPRET` growth in the frame above,
  and a `PROCEDURE EXPOSE` aliasing into the grown outer frame.

## Issues

### Critical

None.

### Important

**I1. The fast-path precondition and the identity check are debug-only, so a violation reads the
wrong frame silently in release.** Location: `rust/crates/rexx-core/src/roots.rs:185-196`, `:210-222`,
`:243-253` (fast paths) and `:401-408` (`segment`).

- **What:** `frame_slot`, `set_frame_slot` and `clear_frame_slot` ignore the handle while
  `indirect == 0` and use `top_start`. `segment()` checks the serial only under `debug_assert_eq!`.
  At the base, the fast path was `frame.start + index`, which is correct for any frame. So this is a
  new silent-wrong-answer mode, even though no caller uses it today.
- **Caller enumeration (named risk 1):** the non-test callers are
  `variables.rs:29,40,52,87,105,130` and `dispatch.rs:2349` (park_reply), `:2520` (resume_reply).
  - `variables.rs` is reached only from `variable`/`set_variable`/`clear_variable`. Their call
    sites (datatype.rs, stem.rs, run.rs, run/call.rs, run/loops.rs, run/interpret.rs, install.rs,
    dispatch.rs, ir/drive.rs, surface.rs) all pass `self.activation().frame` or a frame they have
    just pushed.
  - The running activation's frame is the top segment except under `CallerSwap`, which opens
    `begin_indirect` (`surface.rs:874`, `:888`).
  - `variable_in` (the one suspended-activation read) was switched to `frame_slot_of`.
  - `park_reply` reads the replying method's own frame. `resume_reply` writes a frame it has just
    pushed.
  - G6 runs the whole corpus in debug with `debug_assert_top` live. No caller violates the
    precondition today.
- **Why it matters:** the invariant "the running activation's frame is on top" is exactly what
  Phase 6 changes, because parked activations are swapped in by value (P13). The next caller that
  addresses a non-top frame without `begin_indirect` gets a plausible wrong value. The failure
  class matches the one the old `grow_slots` panic was written to prevent ("a variable that lands
  in another routine's pool").
- **Fix (recommended):**
  - Make the fast path correct by construction instead of asserting it. Cache `top_serial` next to
    `top_start` and take the fast path only when `indirect == 0 && frame.serial == top_serial`.
    Otherwise fall through to the `_of` path.
  - That costs one load from the line `top_start` already occupies and one predictable compare. It
    has no precondition left to violate, and `begin_indirect` for CallerSwap becomes optional.
  - It costs the same as a release `assert!` on the same compare and is strictly better, because a
    violation is served instead of aborting.
  - Promote the check in `segment()` from `debug_assert_eq!` to `assert_eq!`. That record is
    already loaded (`start` and `serial` are adjacent), and it covers the slow paths, `frame_len`
    and `grow_slots_of`.
  - Measure both with the Task 7 callgrind recipe on assign, varlookup, nop, emptyloop and rexxcps
    against a0a82f2b7. A type-level token for "the top frame" was considered and rejected: the
    handle lives in `Activation` and is copied freely, and a token could not stop a stale copy.

**I2. The native frame token can alias a live frame after a u32 wrap.** Location:
`rust/crates/rexx-exec/src/dispatch/library.rs:488-489`,
`rust/crates/rexx-exec/src/dispatch/library/surface.rs:735-742`.

- **What:** `suspended_caller` returns the *newest* `native_handles` entry whose `id` equals the
  token (`iter().rev().find`).
- **How it aliases:** suppose an outer native with id N stays live (it has kept its context and
  called back into Rexx) while 2^32 further native calls happen in that activity. A native nested
  above it can then also carry id N. A use of the outer's kept context while that native is live
  resolves to the inner one's caller. The result is the wrong activation's variables, silently.
- **Feasibility:** it is reachable in principle, for example a Rexx loop of a few billion native
  calls under a long-lived outer native. It is rare, but it is a correctness bug, not only a stale-
  handle one. A context used after its native returned is outside the contract in the oracle too,
  so staleness is not the concern.
- **Cheapest sound fix:**
  - Pack the row and the id into the u64 that rexx-api already carries:
    `(row as u64) << 32 | id as u64`.
  - `suspended_caller` indexes `native_handles[row]` and checks `id == token as u32`.
  - Two live frames cannot share a row, so no two live frames can share a token. `NativeFrame` stays
    256 bytes; a u64 id cost extcall +0.98% in round 1.
  - The lookup becomes O(1) instead of a reverse scan.
- **Test:** set `next_native` to `u32::MAX` before an outer9-style run, which is a cfg(test) knob.

### Minor

- **M1. A comment now names an invariant that no longer exists.**
  `rust/scripts/mutate-4b.sh:499-500` says the mutation "trips a `roots.rs` invariant (`grow_slots
  on a frame that is not the top one`)". That panic and `grow_slots` are both gone. Under the
  2026-09-29 prose rule, delete the parenthetical, or delete the clause and keep "panics
  `corpus_differential`".
- **M2. Nothing exercises the descent assertion.** It is reachable through the public API: alias an
  outer slot to `slot_ref(inner, _)` and then read the outer slot. A `#[cfg(debug_assertions)]
  #[should_panic(expected = "does not descend")]` test in `tests/roots.rs` would prove the assertion
  is live, as the two new debug-only tests do for theirs.
- **M3. The perf record has no verdict line for Task 7.** `phase-6-perf.md` "## Task 7" ends at the
  second wall-clock table. The budget verdict ("every program inside budget at round 2") and the
  wall-clock reading appear only in the report. Earlier tasks' sections state theirs in the record
  (for example, the "Every program is inside its ±4% bar ..." line above it). Add the two sentences
  there.
- **M4. Criterion 8's enumeration cannot see struct-literal refusals.** This refusal
  (`crate::Loud { message }` built with `owned_message(.., Some("Phase 6"))`) was never a
  `refusal-sites.tsv` row. Phase 6's committed command for criterion 8 must therefore grep
  messages or owners, not constructors, or it will under-count. This does not block Task 7; it is
  for whoever writes that command.
- **M5. `SlotRef` stored in the heap can hold a `Slot` target by type.** `Body::Cell(SlotRef)`
  (`rexx-core/src/body.rs:148`) holds only cells in practice, because `promote` always answers
  `Target::Cell`. Stored there, a `Target::Slot` would outlive its frame. This is unchanged from
  the base (a tagged usize). Optionally, a `CellRef` newtype would make it type-level.

### Named risks not raised as issues

- **Risk 3 (REPLY):** the only lasting `SlotFrame` holders are `Activation::frame` and alias
  entries.
  - `resume_reply` pushes a fresh frame and rewrites `activation.frame = frame`
    (`dispatch.rs:2514-2531`).
  - Alias entries saved across the park are asserted to hold no `Target::Slot`
    (`park_reply`'s `frame_aliases == 0`, `dispatch.rs:2342`). `Target::Cell` entries are
    frame-independent.
  - `>name` references hold a cell (`promote` never answers a slot). `VarHome::Slot` is transient
    inside `exec_procedure`. PROCEDURE EXPOSE inside a method is refused (17.1).
  - `NativeFrame::caller` is an activation id, not a stack row.
  - Nothing stale survives a REPLY today. The new serial makes a stale handle *detectable* (in
    debug, and in release with I1's fix), which the base's `{start, depth}` could not do. A
    cross-activity REPLY move must rewrite `depth`, which is the report's concern 2 and Phase 6
    S1's own work.
- **Risk 4 (absolute indices):** `git grep` at 863d147d1 for `frame_starts`, `alias_count`,
  `CELL_TAG`, `grow_slots\b`, `outer_caller`, `unbound_outer`, `native_handles.len() - 1`,
  `native_handles.get(` and `native_handles[` finds:
  - in code, only `dispatch/context.rs:715` (`build_native_frame_at(row)`, a transient row used
    while building a StackFrame, never handed out or stored) and a test;
  - otherwise only historical plan documents and M1.
  - `top_start` is an absolute index, but it is private to `ActivityRoots`, recomputed on
    push, pop and grow, and travels with the struct under a by-value swap (P13).
- **Risk 5:** see Spec Compliance. The check exists and is meaningful; it is untested (M2).
- **Risk 7 (wall clock):** two runs of `wallclock.sh -r 5`, each a median of five.
  - decloop: +8.54% and +2.41%, mean about +5.5%, inside its P10 band of +6.10%.
  - heapshape: +3.10% and +4.37%, mean about +3.7%, inside ±4%. It is positive in both runs, but its
    Ir is -0.41% and the identical-binary control shows +1.25% in run 2.
  - Instruction counts bind (P10/P15) and are negative for both programs. Each excursion appears in
    one run only. **No re-run is needed.** If the controller wants heapshape settled, one more
    three-way run (base, t7b, base2) is enough.

## Assessment

**Task quality: Needs fixes.**

I2 is a real, if rare, silent-wrong-variable path, with an O(1) fix that costs no size. I1 turns a
latent silent wrong answer, which the next Phase 6 task is well placed to trigger, into a correct
fallback for one compare. Both are small and should be measured with the existing callgrind recipe.
M1-M3 are one-line record and test fixes. The design, the refusal removal and the evidence are
otherwise sound.
