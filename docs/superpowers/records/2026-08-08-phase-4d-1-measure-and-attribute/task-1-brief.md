### Task 1: Correct the record before anything reads it

Three documents carry a claim that is backwards, and later tasks read them. This task runs first so no measurement is designed against a false premise.

**Files:**
* Modify: `docs/superpowers/plans/perf-baseline.md`, `docs/superpowers/plans/phase-4-exclusions.txt`, `docs/superpowers/plans/2026-07-27-rust-rewrite.md`
* Modify: `.superpowers/sdd/2026-08-04-phase-4c-builtins-and-parse/rexxcps-measurement.md` and `task-5-report.md` if the claim appears there

- [ ] **Step 1: Reproduce the reversal yourself**

Do not take it on faith. From a fresh scratch directory, with `say 1` in a file:

```
for cap in 100000 400000 600000; do
  ( cd "$D" && ulimit -v $cap; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
      /home/moritz/dev/repos/ooRexx/build/bin/rexx "$D/t.rex" >/dev/null 2>&1; echo "oracle $cap -> $?" )
  ( cd "$D" && ulimit -v $cap; \
      /home/moritz/dev/repos/ooRexx-rust-rewrite/rust/target/release/rexx-run "$D/t.rex" >/dev/null 2>&1; echo "rust $cap -> $?" )
done
```

Expected: oracle 0 at all three caps; rust 101, 101, 0.

Confirm the constant is ours and not the oracle's: `/bin/grep -rn INTERPRETER_STACK_BYTES` in both trees. It is at `rust/crates/rexx-exec/src/lib.rs:309` and has no hits in the C++ tree.

- [ ] **Step 2: Correct every occurrence**

`perf-baseline.md` around `:262` is the origin; the others follow it. The corrected statement is that **this crate** reserves 512 MiB of address space before running anything, so under a shared `ulimit -v 1048576` this crate has roughly 500 MB of room and the oracle roughly 1000 -- which is what `rust/CLAUDE.md` already says correctly, and is the wording to align to.

**Correct the claim; do not delete the paragraph.** It is load-bearing for how memory findings under that cap are read.

- [ ] **Step 3: Amend the master plan for 4d's existence**

`2026-07-27-rust-rewrite.md` currently says at `:473` "The Phase 4 row closes when 4c closes", its roadmap row at `:442` uses the phrase "the ratio bar" without ever defining it, and 4d appears nowhere.

Make three changes: say the Phase 4 row closes when 4d closes; **define "the ratio bar" at `:442` as the parity shipping gate of Global Constraints `:39`**, not as 4a's R2 threshold of 1.5, which `:40` scopes to Phase 1; and add 4d to the roadmap as a sub-phase alongside 4a, 4b and 4c.

- [ ] **Step 4: Verify no occurrence survives, and commit**

`/bin/grep -arn "512 MiB\|INTERPRETER_STACK_BYTES\|512 \* 1024" docs/ .superpowers/` and read each hit. Commit.

---

