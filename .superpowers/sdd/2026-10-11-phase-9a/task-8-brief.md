### Task 8: `NEW` on subclasses of String, Stem, Method, Routine and Message

**Files:** `dispatch/construct.rs:102` (`unbuilt_new`), `dispatch/class_protocol.rs`, `environment.rs`,
`body.rs`, `dispatch.rs`, `hash/stem.rs`, `string.rs`, `value.rs` (scout D section 3's file list), corpus
programs. Reference only: `.superpowers/sdd/2026-10-07-phase-6-1/scout-d-prototype.patch` and that
phase's `scout-d-report.md` (what the prototype lacks).

- [ ] **Step 1:** Corpus programs for each class: `.K~new(...)` with and without an INIT, the
      instance's class, its string value, identity answers (`return this` audit: every method the
      6.1 report lists), `request('STEM')`, `copy`, hash keys in a Table, `sign`, Stem `empty`,
      Method/Routine `scope`, `annotations`, `addRoutine` accepting a descendant, Message with
      `~send`. Also `Class~enhanced` running `NEW` (`.array~enhanced(d, 3)`, oracle `0 Array`) and the
      queued items `2026-10-10-subclass-package`, `-subclass-rexxinfo`,
      `2026-09-30-stringtable-subclass-new-arg`, `2026-09-30-hash-subclass-init-capacity`.
- [ ] **Step 2:** Implement String and Stem first, commit; then Method, Routine, Message and
      `enhanced`, commit; then the queued items, commit. Delete `unbuilt_new`'s refusal for each class
      as it lands. Per-task check per commit, with `Method`, `Routine`, `String`, `Stem`, `Message`,
      `Class` and `CONSTANT` through the harness.

