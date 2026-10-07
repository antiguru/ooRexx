### Finding Verdicts

- **I1: COUNTER and DO WITH passes keep a temporary rooted to loop end** — ADDRESSED. `count_pass` (run/loops.rs:731, 745) and `with_advance` (2504, 2515, 2532) bracket each pass with `push_frame`/`pop_frame`. `FrameId` is only a temps length mark (roots.rs:352-358), so `pop_frame` truncates temps to the mark and nothing else. The new crate test `counter_and_with_passes_do_not_accumulate_temps` (tests/loops.rs) asserts the high-water mark is flat for COUNTER and equal to the supplier build's own mark for DO WITH. The report names the red runs with each pop removed and shows the peak-memory before/after.
- **Gate record: whole-group failing rows need cause and alone-run** — ADDRESSED. `docs/superpowers/plans/phase-6-1-gate.md` `## Task 2` has a table with one row per newly visible failing test: the alone-run result, the line, and a cause outside the loops. The emptyloop ruling is recorded.
- **M3: `HeaderRole::OverFor` doc** — ADDRESSED. The doc at run/loops.rs:~388 now names both `DO name OVER expr FOR expr` and `DO WITH ... OVER expr FOR expr`.

### Named risk (pop must not drop a value the loop still needs, or run on a path without a push)

- Supplier: `with.supplier` is read at entry and was rooted at setup, before the mark is taken, so truncation to the mark cannot reach it.
- COUNTER value: rooted by the bound variable after `bind_control`. The pop follows the trace echo, which reads the value first. `counter.passes` is a Rust integer, not a temp.
- FOR count: `with.remaining` is a Rust integer. The pop precedes its check and never touches it.
- LEAVE/SIGNAL: neither occurs between the push and the pop in either function. The push is the first statement and each pop is on every non-`?` exit (the AVAILABLE-false early return pops at 2515). The `?` paths skip the pop and leave the temps to outer truncation. Truncating to a mark is idempotent, so a later pop with a lower mark is harmless. Each push has a matching pop on every normal path.
- AVAILABLE is now pushed as a temp inside the frame, so it is rooted across `string_value_text`, which is a small improvement.

### New Breakage in the Fix Diff

None.

### Out-of-Scope Observations

- The `?`-path non-pop relies on outer truncation. That is sound as far as I read it, but I did not verify that a trapped SYNTAX condition inside a DO WITH header truncates the temps. The controlled loop at 2270 has the same shape, so this is not new.

### Verdict

**Fix round:** All findings addressed, no new Critical/Important breakage.
