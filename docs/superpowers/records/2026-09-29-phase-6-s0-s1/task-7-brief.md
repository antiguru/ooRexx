### Task 7: Activation-relative offsets and slots per activation segment

**Files:** `rust/crates/rexx-core/src/roots.rs` (`SlotRef`, `aliases`, `SlotFrame`, `frame_starts`,
`alias_slot` `:300-306`, `resolve_aliased` `:329-342`), `rust/crates/rexx-exec/src/dispatch.rs`
(`take_frame_aliases`/`put_frame_aliases` at `:2342`, `:2503`), `run.rs` (PROCEDURE EXPOSE aliasing
`:1294-1318`, `:1684`), `plan.rs:747` (`grow_slots`), `dispatch/library.rs` (the native-frame index at
`:96`, `:167`, `:215`, `:259`), `dispatch/library/surface.rs` (`in_caller` `:437`,
`suspended_caller` `:783-784`, the unbound-outer refusal at `:802-808`), the exclusions row for the
kept-outer-context residual.

**Interfaces:** Produces slot positions as (activation identity, offset) resolved through the
activation's own record; `ActivityRoots::grow_slots_of(activation, n)` usable on a frame below the
top.

- [ ] **Step 1:** Write the failing test: through a kept outer call context (Phase 8's `outer9`
      forge, `docs/superpowers/records/2026-09-14-phase-8-surface/task-9-probes/`), a SET of a name the
      outer activation never bound matches the oracle (today rc 120, refusal naming Phase 6).
- [ ] **Step 2:** Make each stored position activation-relative, and give each activation its own slot
      segment so a frame below the top can grow. Keep a debug assertion equivalent to today's
      "targets descend" check (`:335-337`) in the new form.
- [ ] **Step 3:** The test of Step 1 passes; remove the refusal and its exclusions row (it is the
      Phase 6 owner the spec's criterion 8 lists), re-derive `refusal-sites.tsv`. Corpus,
      collect-stress, the PROCEDURE EXPOSE and REPLY corpus programs green. Callgrind against the
      base.
- [ ] **Step 4:** Gates; commit.

