### Task 1: Record the rulings and grants in the roadmap, and adopt the flat-loop path

**Files:** `docs/superpowers/plans/2026-07-27-rust-rewrite.md`; `rust/crates/rexx-exec/src/run/loops.rs`,
`ir/compile.rs`, `ir.rs`, `ir/drive.rs`, `lib.rs` (SPIKE markers only);
`.superpowers/sdd/queued/2026-09-26-flatloop-spike-comment.md` (close).

- [ ] **Step 1:** In the roadmap, add to D3's dated note of 2026-09-29 the rulings R2 (`.environment`
      per interpreter licensed for Phase 9), R3 (signal handling as a new `unsafe` site) and R5
      (`libc` as a direct dependency of the signal module), quoting the spec's section 1.1.
- [ ] **Step 2:** Add the Section 1 decision blocks the Global Constraints require before code: the
      `unsafe impl Send` for the interpreter island (spec 2.5, lands in S4; name its module), the
      signal module (R3, lands in S4), and **`frame.rs`'s changed invariants** (one arena per
      activity instead of one per interpreter; LIFO per activity; used by Task 5). Amend row 6's
      "kernel lock" wording to point at D3's note (R1), and the Global Constraints' nightly line for
      R4 (nightly only for the TSan gate, from the installed toolchain).
- [ ] **Step 3:** Adopt the flat-loop path: remove the `REXX_NO_FLAT` toggle and `static FLAT`
      (`run/loops.rs:1303-1308`) so the flat path is unconditional, and remove every "SPIKE"
      marker and the `reason = "spike"` allow (`loops.rs:1287`), keeping each marker's surrounding
      comment's meaning. Derive the marker list with `/bin/grep -a -rn 'SPIKE\|spike' rust/crates/rexx-exec/src`
      before and after, and record both outputs in the task report.
- [ ] **Step 4:** Gates; commit (docs and code in separate commits).

