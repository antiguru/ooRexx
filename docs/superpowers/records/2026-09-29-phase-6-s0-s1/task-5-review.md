# Task 5 review: per-activity roots (0993b9457..5ecf22e37)

**Spec compliance: ✅ compliant**, with one open Step 3 item (⚠️ Miri, being run separately) and three
Minor issues. Task quality: **Approved**.

## Spec Compliance

- ✅ `ActivityRoots` holds `temps`, `frames`, `slots`, `aliases`, `alias_count`, `frame_starts`,
  `parked`, `parked_free` (`rust/crates/rexx-core/src/roots.rs:66-91`). `RootSet` keeps `globals`
  and `cells` and holds the running activity's part inline (`roots.rs:55-62`). That satisfies P3 and
  the inline half of P13. `activity()` and `activity_mut()` are `#[inline(always)]` field projections,
  with no load added.
- ✅ Each `ActivityRoots::new` creates its own `Rc::new(FrameArena::new(..))` (`roots.rs:237`).
  `FrameArena`'s code is unchanged: `git diff 0993b9457 5ecf22e37 -- rust/crates/rexx-core/src/frame.rs`
  touches only `//!` and `//` lines.
- ✅ Step 1. The module doc (`frame.rs:12-38`) restates D-U4's invariant per arena: properties 1-5
  "for each arena", LIFO as a logical property, the cross-arena release case, and the move clause. The
  grant comment is dated and cites D-U4. `tests/unsafe_sites.rs` is unchanged and needs no change
  because it keys on file paths (`:91`, `:111`), not on the grant text.
- ✅ Step 2. The LIFO asserts are byte-identical and now per `ActivityRoots`: `pop_slots` at
  `roots.rs:356-360` and `grow_slots` at `:456-463`, with the same messages.
- ⚠️ Step 3. Miri has not run on this tree. Per the dispatch it is running separately, and its result
  is to be appended to the report. The corpus, collect-stress and unsafe-site tests passed through
  G4/G6 (controller-verified).
- ✅ Callgrind recorded against the Phase 6 base `1754a3b5a`, the same base Task 4 uses.

### Named risks

1. **`Rc<FrameArena>`.** The `Rc` is not new: base `RootSet` already had `frames: Rc<FrameArena>` and
   `frames()` returning a clone. Both clones outside tests are locals held for one chunk's
   duration: `run_chunk` (`rexx-exec/src/ir/drive.rs:377`) and `INTERPRET`'s fragment
   (`run/interpret.rs:73`). Each reserves from and releases to the handle it took, and never
   re-fetches `roots.activity().frames()`. So a release goes back to the arena the frame came from
   even if the running `ActivityRoots` were swapped in between. That is the property D-U4 wants, and
   the `Rc` provides it. No second activity can reach the arena, because no clone is stored in any
   structure; the only other holder is the test file `rexx-core/tests/roots.rs`. A `mem::swap` of
   `ActivityRoots` under P13 moves only the `Rc` pointer. The `FrameArena` and its blocks stay put,
   so a clone held across the swap still names the right (swapped-out) arena. It is not stale.
   `RegFrame` raw pointers point into blocks, not into `ActivityRoots`, which is consistent with P13.
   `set_frame_block`'s `strong_count == 1` assert is unchanged. **Clear.** (See Minor 1 on the
   module doc's wording about moves.)
2. **Aliasing across the split.** A non-cell alias target is an absolute position in the same
   `ActivityRoots`' `slots`, since `alias_slot` stores into `self.aliases` a `SlotRef` from
   `slot_ref`. A cell target is `CELL_TAG`-tagged and indexes the interpreter-wide `cells`, so it
   means the same thing from any activity. That is the intended sharing for a `>name` reference.
   With one `ActivityRoots` today, no alias can refer into another's slots. With two, nothing in the
   types stops a `SlotRef` or `SlotFrame` taken from activity A being used after a switch to B: it
   would index B's `slots`. That stays memory-safe because of the `assert!(position < slots.len())`
   in `at`, `write`, `frame_slot` and `set_frame_slot`, but it is logically wrong. Plan Task 7
   (activation-relative offsets) owns exactly these structures, so this is a note for Task 7, not a
   Task 5 defect. **Clear for this task.**
3. **`RootSet::iter` sources.** The base yields globals, temps, `frames.iter()`, slots, cells and
   parked. The head yields globals, cells, then `activity.iter()`, which is temps, `frames.iter()`,
   slots and parked (`roots.rs:229-235`, `:471-477`). All six sources are present and none is
   dropped. Only the order changed, and mark order is not observable (licensed, as the report notes).
   **Clear.**
4. **LIFO asserts.** Verbatim and per `ActivityRoots`. **Clear.**
5. **`frame.rs`.** The code is byte-identical and `unsafe_sites.rs` is consistent. The doc states the
   invariant with no narrative. **Clear**, apart from Minor 1.
6. **Mechanical call sites.** For every changed `.rs` file except `roots.rs` and `frame.rs`, I took
   the base and head texts, deleted `.activity()` and `.activity_mut()`, mapped `ActivityRoots::` to
   `RootSet::`, and compared them as whitespace-separated token streams (trailing commas ignored, to
   absorb rustfmt). The residue:
   - `rexx-core/src/lib.rs`: the `ActivityRoots` re-export.
   - `rexx-core/tests/roots.rs`: two `set_frame_block` reflows, and one
     `alias_slot(.., roots.slot_ref(..))` hoisted into `let target = roots.slot_ref(outer, 0);`.
     `slot_ref` is `&self` and pure, so the hoist changes nothing.
   - `rexx-exec/src/lib.rs`: a `set_frame_block` reflow.
   - `rexx-exec/src/run.rs:~1690`: rustfmt turned a lengthened match arm into a braced block.

   No logic changed. **Clear.**
7. **`phase-6-perf.md` `## Task 5`.** The recipe is complete, including the `cp` step and the
   `Compiling rexx-exec` count. I checked against `$S/p6-t5`:
   - Both build logs have 1 `Compiling rexx-exec` line.
   - `sha256sum` of `bin/base` and `bin/r1` matches the table and `cg1/binaries.txt`.
   - Every row of the Ir table matches `cg1/table.txt`.
   - The report's summary figures (±0.0006% band, heapshape −0.1058%, rexxcps −0.0004%, +196
     largest) match the table.

   **Clear**, apart from Minor 2, which concerns the report and not the doc.

## Strengths

- A narrow split. Everything that can reach a cell (`promote`, `slot_value`, `set_slot_value`,
  `frame_slot`, `set_frame_slot`, `clear_frame_slot`, `at`, `write`) stays on `RootSet`, and its
  bodies change only by `self.x` → `self.activity.x`.
- Moved methods keep their doc comments and asserts verbatim, and the rustdoc links were retargeted.
- The call-site rewrite really is mechanical. The token diff above leaves nothing unexplained.

## Issues

### Critical

None.

### Important

None.

### Minor

1. **`rust/crates/rexx-core/src/frame.rs:32-34`: the move clause describes a move the design does not
   make.** "A `FrameArena` value may move with its activity; its blocks never do, and no `RegFrame` is
   live across the move, since it borrows the arena." `ActivityRoots` holds `Rc<FrameArena>`. Moving
   an activity moves the `Rc` handle, and the `FrameArena` value stays in the `Rc` allocation. The
   report says so itself: "moving an `ActivityRoots` moves only the handle". Under P13 a `RegFrame`
   *is* live across an `ActivityRoots` swap: `run_chunk` holds `registers` for the whole chunk. That
   is safe because the arena does not move, not because no frame is live. The sentence copies D-U4's
   wording ("the struct moves"), which the brief told the implementer to follow, and it is still true
   as a statement of what the unsafe code tolerates. So this is a wording mismatch with the `Rc`
   design, not a safety gap. **Fix:** either delete the sentence (the prose rule's fix), or have the
   controller rule that the grant's own text stands. Do not reword it here without D-U4 changing.
2. **Report only (`task-5-report.md`, Performance): the heapshape attribution is wrong.** The report
   calls heapshape's −3,469,487 Ir (−0.1058%) a "timed program" effect. I ran
   `python3 rust/bench-programs/cgdiff.py $S/cg1/heapshape.base.r1.cg $S/cg1/heapshape.r1.r1.cg`.
   It shows −3,463,979 of that in `Interp::collect_now` self, and −48,769 in the old `RootSet::iter`
   chain's `next`. So the change is how the root iteration is inlined into the collector: `iter` is
   now globals+cells chained onto `activity.iter()`. The perf doc measures `TIME()` noise between
   heapshape runs at about 100 Ir (`phase-6-perf.md:147-158`), and the three rounds have 0.0000%
   spread. The perf doc itself makes no causal claim, so nothing committed under `docs/` is wrong.
   **Fix:** correct the sentence in the report, or drop the parenthetical.
3. **`rust/corpus/lang/class_context_reply.rex:4` (and its copy
   `rust/crates/rexx-parse/tests/sourceline_oracle/class_context_reply.txt:5`): stale method name.**
   "`park_reply` into `RootSet::park` into `Activation::object_roots`". `park` is now
   `ActivityRoots::park`. The six source comments the report lists were retargeted; this one was
   missed because it sits in a `.rex` file. **Fix:** `RootSet::park` → `ActivityRoots::park` in both
   files, together, because the sourceline oracle fixture mirrors the program text.

### Notes for later tasks (not defects here)

- A `SlotFrame` or `SlotRef` does not carry its activity, and the `RootSet` slot methods always
  address `self.activity`. Task 7 should make it impossible to use one activity's handle against
  another's slots after a switch, or should state why no such path exists.
- `Rc` makes `ActivityRoots` `!Send`, as `RootSet` already was. This matters when D-U2's island
  `Send` is argued.

## Assessment

**Task quality: Approved.** The split matches the brief and rulings P3/P13, root coverage is
complete, the LIFO asserts are intact, `frame.rs`'s code is untouched, and the call-site rewrite is
token-for-token mechanical. The Minor items are prose (one in a committed module doc, one in the
report, one in a corpus comment). Step 3 stays open until the separate Miri run is recorded.
