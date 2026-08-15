### Task 4: Make the allocation axis measurable

`alloc.rex` stops at `rc=120`, "a message send is not implemented (Phase 5)". Allocation throughput does not need message sends, and 4d-2 will land representation changes -- **landing them with this axis unmeasured is the worst available ordering.**

**Files:**
* Create: `rust/bench-programs/alloc4c.rex`
* Modify: `rust/bench-programs/README.md`, `docs/superpowers/plans/perf-baseline.md`

- [ ] **Step 1: Read `alloc.rex` and find what it uses that 4c lacks**

Run it on the oracle and here, and record both. Identify the exact construct that blocks.

- [ ] **Step 2: Write `alloc4c.rex` over the 4c surface**

It must allocate on the same order and in the same proportions as `alloc.rex` does, using only constructs this crate implements -- string concatenation, compound-variable creation and numeric temporaries all allocate and are all available.

**State in the file's own header what it does and does not preserve from `alloc.rex`.** A variant that avoids the blocking construct by measuring something else is not the same axis, and the honest handling is to say which parts of the original axis it covers.

- [ ] **Step 3: Confirm it runs on both sides and produces identical output**

Byte-identical stdout, exit 0 both sides. An interpreter that computes something different is not faster.

- [ ] **Step 4: Add it to the harness's axis list and re-run, then commit**

---

