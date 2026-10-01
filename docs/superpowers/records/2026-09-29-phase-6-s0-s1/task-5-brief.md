### Task 5: The per-activity `RootSet` part and a frame arena per activity

**Files:** `rust/crates/rexx-core/src/roots.rs`, `rust/crates/rexx-core/src/frame.rs` (module doc and
invariants), `rust/crates/rexx-core/tests/unsafe_sites.rs` if the grant's text changes, the
`rexx-exec` call sites of the moved methods (derived; `push_temp`, `push_frame`, `pop_frame`,
`push_slots`, `set_frame_slot`, `frame_slot`, `pop_slots`, `temps_len` and the rest).

**Interfaces:** Produces `rexx_core::roots::ActivityRoots` holding `temps`, `frames` (the arena),
`slots`, `aliases`, `alias_count`, `frame_starts`, `parked`, `parked_free`; `RootSet` keeps `globals`
and `cells` and the running activity's `ActivityRoots`; `RootSet::iter` covers every activity's
roots (one today).

- [ ] **Step 1:** Write the restated `frame.rs` grant into its module doc exactly as Task 1's decision
      block words it; the arena type itself changes only in ownership (one per `ActivityRoots`).
- [ ] **Step 2:** Split `RootSet`, keeping the LIFO asserts (`pop_slots` `:225`, `grow_slots` `:415`)
      per `ActivityRoots`. The running activity's part stays one indexed load away.
- [ ] **Step 3:** Corpus, collect-stress, `tests/frame_block.rs` and the unsafe-site test green;
      Miri on `rexx-core`'s lib tests under Stacked Borrows, since `frame.rs` changed (record how
      Miri was run). Callgrind against the base; rounds as in Task 4.
- [ ] **Step 4:** Gates; commit.

